//! `POST /api/v4/governance/admin/assign-jury` — admin backstop to seat a
//! jury panel with reputation gating + concurrent-cap.
//!
//! Phase 5b task 57 replaces the Phase 4 "not target AND not reporter AND not
//! deleted AND accepted_application" eligibility filter with the reputation-
//! gated form: INNER JOIN `reputation_snapshot` on `jury_eligible = true`
//! and a concurrent-cap subquery excluding jurors already at the
//! `jury.max_concurrent_assignments` cap. All five caps come from
//! `governance_config` via `ConfigCache`; `PANEL_SIZE` is now
//! `jury.panel_size`.
//!
//! The case is flipped `Open | ThresholdMet | EmergencyRemove` →
//! `JurySelection`; the plan's shorthand `InPanel` maps to the enum's
//! `JurySelection` variant (decision-queue #7). Per [99 ADR-013] the
//! status match is exhaustive; no `_ =>` catchall.
//!
//! All DB writes run inside one `run_transaction` so the 5 juror inserts +
//! the case update + the 6 log entries land atomically. Partial execution
//! would leave the case in a half-assembled panel state.
//!
//! ## Small-pool fallback
//!
//! Instances bootstrapping a community may not yet have 5 jury-eligible
//! persons with snapshot rows. When the strict filter returns fewer than
//! `jury.panel_size` candidates, the handler consults
//! `jury.fallback_on_small_pool`:
//! - `true` (seeded default) — re-run the Phase 4 filter without the
//!   reputation gate; a `warn!` log entry fires so operators notice.
//! - `false` — return `NotFound` so the admin learns the pool is too
//!   small rather than silently seating an unqualified panel.

use crate::governance::{
  actor_pseudonym_helper,
  config::{self, ConfigCache, Scope},
  governance_log::{self, ENTRY_KIND_JURY_CONSTRAINT_RELAXED},
  jury_common::panel_has_sponsor_majority_cluster,
};
use actix_web::web::{Data, Json};
use diesel::{
  ExpressionMethods, QueryDsl, QueryableByName, SelectableHelper,
  sql_query,
  sql_types::{Array, BigInt, Integer, Nullable},
  insert_into,
  update,
};
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_api_common::governance::{AdminAssignJury, AdminAssignJuryResponse};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::source::governance::{
  jury_assignment::JuryAssignmentInsertForm,
  jury_constraint_violation_log::JuryConstraintViolationLogInsertForm,
  moderation_case::ModerationCase,
};
use lemmy_db_schema_file::{
  PersonId,
  enums::{CaseStatus, JuryAssignmentStatus, JuryConstraintRelaxationReason},
  schema::{jury_assignment, jury_constraint_violation_log, local_user, moderation_case, person},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::{connection::get_conn, utils::functions::random};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::{Value, json};
use tracing::warn;

pub async fn admin_assign_jury(
  Json(data): Json<AdminAssignJury>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminAssignJuryResponse>> {
  is_admin(&local_user_view)?;

  let admin_id = local_user_view.person.id;
  let admin_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), admin_id).await?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let data_for_tx = data;
  let pseudonym_for_tx = admin_pseudonym.clone();

  let outcome = conn
    .run_transaction(|conn| {
      async move { process_assignment(conn, pseudonym_for_tx, data_for_tx).await }.scope_boxed()
    })
    .await?;

  Ok(Json(outcome))
}

/// Body of the `run_transaction` closure. Kept under the workspace
/// `large_futures` lint threshold by matching the submit_jury_vote split.
async fn process_assignment(
  conn: &mut diesel_async::AsyncPgConnection,
  admin_pseudonym: String,
  data: AdminAssignJury,
) -> LemmyResult<AdminAssignJuryResponse> {
  // ConfigCache lives for the whole assignment transaction. Same shape as
  // submit_jury_vote::process_vote.
  let mut cache = ConfigCache::new();

  // 1. Read the case for the status guard + exclusion IDs.
  let case: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(data.case_id))
    .select(ModerationCase::as_select())
    .first(conn)
    .await?;

  // 2. Exhaustive status match per [99 ADR-013].
  match case.status {
    CaseStatus::Open | CaseStatus::ThresholdMet | CaseStatus::EmergencyRemove => {}
    CaseStatus::JurySelection
    | CaseStatus::InReview
    | CaseStatus::Decided
    | CaseStatus::Appealed
    | CaseStatus::Closed
    | CaseStatus::AdminReview => return Err(LemmyErrorType::NotFound.into()),
  }

  // 3. Read panel_size from config for this assignment + downstream check.
  let panel_size =
    config::get_int(&mut cache, &mut (&mut *conn).into(), Scope::Instance, "jury.panel_size")
      .await?;

  // 4. Select eligible jurors via the 3-phase diversity-aware algorithm.
  //    Falls back to the Phase 4 unfiltered shape if the strict filter
  //    under-fills after the R1 cooldown relaxation and
  //    `jury.fallback_on_small_pool` is true. v1-JM-b Task 5 evolves this
  //    caller to cascade panel_size + consume the ConstraintRecord.
  let (eligible, _constraint_record) =
    select_eligible_jurors(conn, &case, panel_size, None, &mut cache).await?;
  let eligible_count = i64::try_from(eligible.len())
    .map_err(|_e| LemmyErrorType::Unknown("eligible count overflow".to_string()))?;
  if eligible_count < panel_size {
    return Err(LemmyErrorType::NotFound.into());
  }

  // 5. Insert JuryAssignment rows with status=Selected (task 64 activation).
  //    Phase 4 used Accepted directly for v0 testability. Task 64 adds the
  //    accept/decline handshake so jurors transition Selected → Accepted via
  //    `accept_jury_assignment` (or → Declined via `decline_jury_assignment`).
  //    submit_jury_vote still filters on status=Accepted, so a decline that
  //    is not replaced in time will fail voting — exactly the intended v0
  //    behaviour.
  // Task 5 populates `selected_under_constraints` with the JSONB
  // generated from `_constraint_record.to_json()`. Keeping None here
  // preserves Task 4 as a pure refactor of the selector; the write path
  // remains the v0 shape until Task 5 upgrades it.
  let forms: Vec<JuryAssignmentInsertForm> = eligible
    .iter()
    .map(|person_id| JuryAssignmentInsertForm {
      case_id: data.case_id,
      person_id: *person_id,
      status: JuryAssignmentStatus::Selected,
      selected_under_constraints: None,
    })
    .collect();
  insert_into(jury_assignment::table)
    .values(&forms)
    .execute(conn)
    .await?;

  // 6. Flip case → JurySelection (plan shorthand "InPanel").
  update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
    .set(moderation_case::status.eq(CaseStatus::JurySelection))
    .execute(conn)
    .await?;

  // 7. One governance_log "jury_assigned" row per juror. Each payload
  //    carries the per-juror pseudonym so the modlog stays pseudonymous
  //    per [99 ADR-015].
  for person_id in &eligible {
    let juror_pseudonym =
      actor_pseudonym_helper::get_or_create(&mut conn.into(), *person_id).await?;
    governance_log::append(
      &mut conn.into(),
      "jury_assigned",
      json!({
        "case_id": data.case_id.0,
        "juror_pseudonym": juror_pseudonym,
      }),
      Some(admin_pseudonym.clone()),
    )
    .await?;
  }

  // 8. Single panel_assembled marker — canonical record of Open→panel.
  governance_log::append(
    &mut conn.into(),
    "panel_assembled",
    json!({
      "case_id": data.case_id.0,
      "juror_count": panel_size,
    }),
    Some(admin_pseudonym.clone()),
  )
  .await?;

  Ok(AdminAssignJuryResponse {
    case_id: data.case_id,
    assigned_person_ids: eligible,
  })
}

/// v1-JM-b per-assignment constraint-application record. Captures which
/// constraints fired or were relaxed during panel selection; the caller
/// writes it both into every `jury_assignment.selected_under_constraints`
/// JSONB column and into the extended `panel_assembled` governance_log
/// payload.
#[derive(Debug, Clone, Default)]
pub(crate) struct ConstraintRecord {
  pub no_majority_from_same_sponsor_cluster: &'static str,
  pub geographic_diversity_preferred: &'static str,
  pub no_recent_juror_repeat: &'static str,
  pub no_same_endorsement_chain: &'static str,
  pub relaxations_fired: Vec<&'static str>,
}

impl ConstraintRecord {
  /// JSON serialisation used for both the per-assignment JSONB column and
  /// the extended panel_assembled governance_log payload. Keys match PRD
  /// §5.1 constraint names verbatim; values are bounded status tokens so
  /// no user-supplied text can leak into the column or log.
  #[expect(
    dead_code,
    reason = "called at v1-JM-b Task 5 jury_assignment.selected_under_constraints insert"
  )]
  pub(crate) fn to_json(&self) -> Value {
    json!({
      "no_majority_from_same_sponsor_cluster": self.no_majority_from_same_sponsor_cluster,
      "geographic_diversity_preferred": self.geographic_diversity_preferred,
      "no_recent_juror_repeat": self.no_recent_juror_repeat,
      "no_same_endorsement_chain": self.no_same_endorsement_chain,
    })
  }
}

/// Select eligible jurors via the v1-JM-b 3-phase diversity-aware
/// algorithm, returning the seated panel alongside a [`ConstraintRecord`]
/// summarising which constraints applied and which were relaxed during
/// assembly.
///
/// - **Phase 1 — pool build**: `run_extended_eligibility_query` consults
///   the reputation gate, the community-wide concurrent-cap, the
///   cross-community total-assignment cap
///   (`jury.max_concurrent_assignments_per_juror_total`), and the recent-
///   juror cooldown (`jury.constraints.juror_cooldown_days`). If the pool
///   comes back < `panel_size`, R1 fires: cooldown is dropped and the
///   query re-runs.
/// - **Phase 2 — sample + sponsor-cluster check**: sample `panel_size`
///   jurors via SQL `ORDER BY random() LIMIT N`; if
///   `panel_has_sponsor_majority_cluster` violates, re-roll up to
///   `jury.constraints.max_retries_before_relax` times. On exhaustion, R2
///   drops the geographic-diversity bias, R3 drops the cluster constraint
///   entirely.
/// - **Phase 3 — geographic-diversity soft bias**: if
///   `jury.constraints.geographic_diversity_preferred` is true and more
///   than one cluster-passing sample survives, the higher-
///   `geographic_diversity_score` sample wins.
///
/// Each R1/R2/R3 event writes BOTH a `jury_constraint_violation_log` row
/// AND a `governance_log` `jury_constraint_relaxed` entry inside the
/// caller's surrounding `run_transaction`, so a panel-assembly rollback
/// reverts the relaxation audit as well (PRD §8.3 + §12.2 atomicity).
///
/// The terminal fallback when pool < `panel_size` post-R1 AND
/// `jury.fallback_on_small_pool = true` defers to
/// [`legacy_select_eligible_jurors`] (Phase 4 unfiltered shape). That
/// path returns `relaxations_fired = ["legacy_fallback"]` so the caller
/// knows the returned panel did not go through the 3-phase cascade.
///
/// `pub(crate)` so `admin_emergency_remove` and `decline_jury_assignment`
/// can reuse it for post-facto / replacement selection. The
/// `panel_size` argument is computed by the caller (via
/// `config::get_int_cascade` in v1-JM-b Task 5, or via `get_int` on
/// `jury.panel_size` for the v0-compatible callers).
pub(crate) async fn select_eligible_jurors(
  conn: &mut diesel_async::AsyncPgConnection,
  case: &ModerationCase,
  panel_size: i64,
  exclude_person_ids: Option<&[PersonId]>,
  cache: &mut ConfigCache,
) -> LemmyResult<(Vec<PersonId>, ConstraintRecord)> {
  // Constraint toggles — JM-a seeded defaults cover every key.
  let cooldown_enabled = config::get_bool(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "jury.constraints.no_recent_juror_repeat",
  )
  .await?;
  let cooldown_days = config::get_int(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "jury.constraints.juror_cooldown_days",
  )
  .await?;
  let cluster_constraint_enabled = config::get_bool(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "jury.constraints.no_majority_from_same_sponsor_cluster",
  )
  .await?;
  let geo_pref_enabled = config::get_bool(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "jury.constraints.geographic_diversity_preferred",
  )
  .await?;
  let max_retries = config::get_int(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "jury.constraints.max_retries_before_relax",
  )
  .await?;
  let max_concurrent = config::get_int(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "jury.max_concurrent_assignments",
  )
  .await?;
  let max_concurrent_per_juror_total = config::get_int(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "jury.max_concurrent_assignments_per_juror_total",
  )
  .await?;
  let fallback_allowed = config::get_bool(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "jury.fallback_on_small_pool",
  )
  .await?;

  let mut record = ConstraintRecord {
    no_majority_from_same_sponsor_cluster: if cluster_constraint_enabled {
      "applied"
    } else {
      "disabled"
    },
    geographic_diversity_preferred: if geo_pref_enabled { "applied_soft" } else { "disabled" },
    no_recent_juror_repeat: if cooldown_enabled { "applied" } else { "disabled" },
    no_same_endorsement_chain: "disabled",
    relaxations_fired: Vec::new(),
  };

  // Excluded IDs set: case.target + case.creator + caller-provided.
  let mut excluded: Vec<PersonId> = Vec::new();
  if let Some(t) = case.target_person_id {
    excluded.push(t);
  }
  if let Some(r) = case.creator_id {
    excluded.push(r);
  }
  if let Some(xs) = exclude_person_ids {
    excluded.extend_from_slice(xs);
  }

  // Over-fetch the candidate pool so Phase 2 re-rolls have fresh samples
  // to choose from without re-querying. 10x panel size covers the worst-
  // case cluster-pressure cascade; capped at 200 so the pool never
  // becomes pathological on an instance with a very large reputation-
  // snapshot set.
  let pool_limit = panel_size.saturating_mul(10).min(200);

  // ============================================================
  // PHASE 1 — pool build with optional cooldown filter
  // ============================================================
  let mut current_cooldown_days = cooldown_enabled.then_some(cooldown_days);
  let mut pool = run_extended_eligibility_query(
    conn,
    case.community_id,
    &excluded,
    max_concurrent,
    max_concurrent_per_juror_total,
    current_cooldown_days,
    pool_limit,
  )
  .await?;

  if i64::try_from(pool.len()).unwrap_or(i64::MAX) < panel_size && current_cooldown_days.is_some() {
    tracing::info!(
      case_id = case.id.0,
      pool_size = pool.len(),
      target = panel_size,
      "jury selection R1: dropping no_recent_juror_repeat (small_pool)"
    );
    write_constraint_relaxation(
      conn,
      case,
      "no_recent_juror_repeat",
      JuryConstraintRelaxationReason::SmallPool,
      json!({
        "phase": "pool_build",
        "dropped_constraint_name": "no_recent_juror_repeat",
      }),
      i32::try_from(pool.len()).unwrap_or(i32::MAX),
      i32::try_from(panel_size).unwrap_or(i32::MAX),
    )
    .await?;
    record.no_recent_juror_repeat = "relaxed_small_pool";
    record.relaxations_fired.push("R1");
    current_cooldown_days = None;
    pool = run_extended_eligibility_query(
      conn,
      case.community_id,
      &excluded,
      max_concurrent,
      max_concurrent_per_juror_total,
      current_cooldown_days,
      pool_limit,
    )
    .await?;
  }

  // If still short after R1: legacy fallback if permitted; else return the
  // under-sized pool so the caller can surface NotFound.
  if i64::try_from(pool.len()).unwrap_or(i64::MAX) < panel_size {
    if !fallback_allowed {
      return Ok((pool, record));
    }
    warn!(
      case_id = case.id.0,
      community_id = ?case.community_id.map(|c| c.0),
      pool_size = pool.len(),
      panel_size,
      "jury pool below panel_size post-R1 — legacy Phase 4 fallback per jury.fallback_on_small_pool=true",
    );
    let legacy = legacy_select_eligible_jurors(conn, &excluded, panel_size).await?;
    record.relaxations_fired.push("legacy_fallback");
    return Ok((legacy, record));
  }

  // ============================================================
  // PHASE 2 — sample with re-roll on sponsor cluster
  // PHASE 3 — soft geographic-diversity bias when multiple passing samples
  // ============================================================
  let mut chosen: Option<Vec<PersonId>> = None;
  let mut best_score: f64 = -1.0;
  let mut current_geo_enabled = geo_pref_enabled;

  let attempts = usize::try_from(max_retries.max(1)).unwrap_or(1);
  for _attempt in 0..attempts {
    let sample = sample_panel(
      conn,
      case.community_id,
      &excluded,
      max_concurrent,
      max_concurrent_per_juror_total,
      current_cooldown_days,
      panel_size,
    )
    .await?;
    if i64::try_from(sample.len()).unwrap_or(i64::MAX) < panel_size {
      // Random sampler ran dry — pool must have become exhausted between
      // phases (transaction isolation still holds; other writers' rows
      // wouldn't show, so this is defensive against pool = {} edge cases).
      continue;
    }

    let violates = if cluster_constraint_enabled && record.no_majority_from_same_sponsor_cluster == "applied" {
      panel_has_sponsor_majority_cluster(conn, &sample).await?
    } else {
      false
    };
    if violates {
      continue;
    }

    if current_geo_enabled {
      let score = geographic_diversity_score(conn, &sample).await.unwrap_or(0.0);
      if score > best_score {
        best_score = score;
        chosen = Some(sample);
      } else if chosen.is_none() {
        chosen = Some(sample);
      }
    } else {
      chosen = Some(sample);
      break;
    }
  }

  if let Some(panel) = chosen {
    return Ok((panel, record));
  }

  // ============================================================
  // R2 — drop geographic_diversity_preferred soft bias; retry the sampler
  // ============================================================
  if cluster_constraint_enabled {
    tracing::info!(
      case_id = case.id.0,
      "jury selection R2: dropping geographic_diversity_preferred bias (cluster_pressure)"
    );
    write_constraint_relaxation(
      conn,
      case,
      "geographic_diversity_preferred",
      JuryConstraintRelaxationReason::ClusterPressure,
      json!({
        "phase": "panel_sample",
        "dropped_constraint_name": "geographic_diversity_preferred",
      }),
      i32::try_from(pool.len()).unwrap_or(i32::MAX),
      i32::try_from(panel_size).unwrap_or(i32::MAX),
    )
    .await?;
    record.geographic_diversity_preferred = "relaxed";
    record.relaxations_fired.push("R2");
    current_geo_enabled = false;

    for _attempt in 0..attempts {
      let sample = sample_panel(
        conn,
        case.community_id,
        &excluded,
        max_concurrent,
        max_concurrent_per_juror_total,
        current_cooldown_days,
        panel_size,
      )
      .await?;
      if i64::try_from(sample.len()).unwrap_or(i64::MAX) < panel_size {
        continue;
      }
      let violates = panel_has_sponsor_majority_cluster(conn, &sample).await?;
      if !violates {
        // Keep current_geo_enabled as false so a future read of the record
        // reflects R2 fired (relaxations_fired already lists "R2").
        let _ = current_geo_enabled;
        return Ok((sample, record));
      }
    }
  }

  // ============================================================
  // R3 — last resort: drop the sponsor-cluster constraint entirely
  // ============================================================
  warn!(
    case_id = case.id.0,
    "jury selection R3: dropping no_majority_from_same_sponsor_cluster (cluster_pressure_exhausted)"
  );
  write_constraint_relaxation(
    conn,
    case,
    "no_majority_from_same_sponsor_cluster",
    JuryConstraintRelaxationReason::ClusterPressureExhausted,
    json!({
      "phase": "panel_sample",
      "dropped_constraint_name": "no_majority_from_same_sponsor_cluster",
    }),
    i32::try_from(pool.len()).unwrap_or(i32::MAX),
    i32::try_from(panel_size).unwrap_or(i32::MAX),
  )
  .await?;
  record.no_majority_from_same_sponsor_cluster = "relaxed";
  record.relaxations_fired.push("R3");
  let last_sample = sample_panel(
    conn,
    case.community_id,
    &excluded,
    max_concurrent,
    max_concurrent_per_juror_total,
    current_cooldown_days,
    panel_size,
  )
  .await?;
  Ok((last_sample, record))
}

/// Row shape for the extended eligibility query. Module-scope because
/// workspace lints deny `items-after-statements`.
#[derive(QueryableByName)]
struct ExtendedEligibilityRow {
  #[diesel(sql_type = Integer)]
  id: i32,
}

/// Build the Phase-1 eligibility pool via a single parameterised
/// `sql_query`. v1-JM-b extends the Phase 5b `run_strict_eligibility_query`
/// with two optional constraints: a recent-juror cooldown (NOT EXISTS
/// subquery against `jury_assignment.responded_at`) and a cross-community
/// total-active-assignment cap (NOT IN GROUP BY / HAVING).
///
/// Parameter order:
/// - `$1` — community_id as nullable `Int4`
/// - `$2` — excluded person ids as `Int4[]`
/// - `$3` — `jury.max_concurrent_assignments` (per-community, `BigInt`)
/// - `$4` — `jury.max_concurrent_assignments_per_juror_total`
///   (instance-wide total, `BigInt`)
/// - `$5` — `jury.constraints.juror_cooldown_days` (`BigInt`); the query
///   short-circuits the cooldown predicate when `$5 <= 0`
/// - `$6` — `LIMIT` (`BigInt`); the caller picks a pool_limit larger than
///   `panel_size` for Phase-2 re-roll headroom
async fn run_extended_eligibility_query(
  conn: &mut diesel_async::AsyncPgConnection,
  community_id: Option<lemmy_db_schema::newtypes::CommunityId>,
  excluded: &[PersonId],
  max_concurrent: i64,
  max_concurrent_per_juror_total: i64,
  cooldown_days: Option<i64>,
  limit: i64,
) -> LemmyResult<Vec<PersonId>> {
  let community_bind: Option<i32> = community_id.map(|c| c.0);
  let excluded_bind: Vec<i32> = excluded.iter().map(|p| p.0).collect();
  // When cooldown is disabled the caller passes `None`; bind `0` so the
  // SQL predicate `($5 <= 0 OR ...)` short-circuits.
  let cooldown_bind: i64 = cooldown_days.unwrap_or(0);

  let sql = "\
     SELECT p.id AS id \
     FROM person p \
     INNER JOIN local_user lu ON lu.person_id = p.id \
     INNER JOIN reputation_snapshot rs \
       ON rs.person_id = p.id \
       AND rs.community_id IS NOT DISTINCT FROM $1 \
     WHERE p.deleted = false \
       AND lu.accepted_application = true \
       AND rs.jury_eligible = true \
       AND p.id <> ALL($2) \
       AND p.id NOT IN ( \
         SELECT ja.person_id FROM jury_assignment ja \
         INNER JOIN moderation_case mc ON mc.id = ja.case_id \
         WHERE ja.status IN ('Selected', 'Accepted') \
           AND mc.community_id IS NOT DISTINCT FROM $1 \
         GROUP BY ja.person_id \
         HAVING count(*) >= $3 \
       ) \
       AND p.id NOT IN ( \
         SELECT ja2.person_id FROM jury_assignment ja2 \
         WHERE ja2.status IN ('Selected', 'Accepted') \
         GROUP BY ja2.person_id \
         HAVING count(*) >= $4 \
       ) \
       AND ($5 <= 0 OR NOT EXISTS ( \
         SELECT 1 FROM jury_assignment ja3 \
         WHERE ja3.person_id = p.id \
           AND ja3.responded_at IS NOT NULL \
           AND ja3.responded_at > now() - ($5 || ' days')::interval \
       )) \
     ORDER BY random() \
     LIMIT $6";

  let rows: Vec<ExtendedEligibilityRow> = sql_query(sql)
    .bind::<Nullable<Integer>, _>(community_bind)
    .bind::<Array<Integer>, _>(excluded_bind)
    .bind::<BigInt, _>(max_concurrent)
    .bind::<BigInt, _>(max_concurrent_per_juror_total)
    .bind::<BigInt, _>(cooldown_bind)
    .bind::<BigInt, _>(limit)
    .load(conn)
    .await?;
  Ok(rows.into_iter().map(|r| PersonId(r.id)).collect())
}

/// Re-run the Phase-1 query with `LIMIT = panel_size` to draw a fresh
/// random sample, without caching the intermediate 10x pool inside Rust.
/// SQL `ORDER BY random()` re-randomises on every call, so re-sampling in
/// a re-roll loop naturally produces independent samples. Much cheaper
/// than Rust-side `rand::shuffle` + avoids adding a direct `rand` dep.
async fn sample_panel(
  conn: &mut diesel_async::AsyncPgConnection,
  community_id: Option<lemmy_db_schema::newtypes::CommunityId>,
  excluded: &[PersonId],
  max_concurrent: i64,
  max_concurrent_per_juror_total: i64,
  cooldown_days: Option<i64>,
  panel_size: i64,
) -> LemmyResult<Vec<PersonId>> {
  run_extended_eligibility_query(
    conn,
    community_id,
    excluded,
    max_concurrent,
    max_concurrent_per_juror_total,
    cooldown_days,
    panel_size,
  )
  .await
}

/// Row shape for [`geographic_diversity_score`].
#[derive(QueryableByName)]
struct DistinctCommunityRow {
  #[diesel(sql_type = BigInt)]
  distinct_count: i64,
}

/// v1-JM-b PRD §5.3 Phase 3 soft geographic-diversity score. Returns the
/// fraction `COUNT(DISTINCT moderation_case.community_id) / panel_size`
/// across the sample's prior jury-assignment history: `0.0` when every
/// sampled juror has only served one community (or no community at all —
/// the bootstrapping case); up to `1.0` when every juror has served a
/// distinct community. Used as a re-roll tiebreaker in Phase 2 of
/// [`select_eligible_jurors`].
///
/// Per PRD §OQ-V1-JM-02 lean, the full timezone-aware heuristic is v1.5;
/// v1-JM-b ships this community-distinct stub and gets the soft-bias
/// plumbing out of the way.
#[expect(
  clippy::as_conversions,
  clippy::cast_precision_loss,
  reason = "distinct_count is COUNT(DISTINCT community_id) and sample.len() ≤ jury.panel_size (≤ 20 per PRD §5.3); neither approaches 2^53"
)]
async fn geographic_diversity_score(
  conn: &mut diesel_async::AsyncPgConnection,
  sample: &[PersonId],
) -> LemmyResult<f64> {
  if sample.is_empty() {
    return Ok(0.0);
  }
  let ids_bind: Vec<i32> = sample.iter().map(|p| p.0).collect();
  let row: DistinctCommunityRow = sql_query(
    "SELECT COUNT(DISTINCT mc.community_id) AS distinct_count \
     FROM jury_assignment ja \
     INNER JOIN moderation_case mc ON mc.id = ja.case_id \
     WHERE ja.person_id = ANY($1) \
       AND mc.community_id IS NOT NULL",
  )
  .bind::<Array<Integer>, _>(ids_bind)
  .get_result(conn)
  .await?;
  Ok((row.distinct_count as f64) / (sample.len() as f64))
}

/// Emit a constraint-relaxation audit inside the caller's
/// `run_transaction`: (1) a `jury_constraint_violation_log` row for the
/// queryable modlog index, AND (2) a `governance_log`
/// `jury_constraint_relaxed` entry for the tamper-evident hash-chain
/// record. Both writes land atomically with the panel seating, per PRD
/// §8.3 / §12.2.
///
/// `actor_pseudonym = None` because R1/R2/R3 relaxations are system-
/// level, not attributed to any admin. `AdminOverride` (PRD §8.3) is
/// distinct and not reached by this path; future sub-phases that wire an
/// admin-initiated override will call `write_constraint_relaxation` with
/// a populated actor pseudonym argument.
async fn write_constraint_relaxation(
  conn: &mut diesel_async::AsyncPgConnection,
  case: &ModerationCase,
  constraint_name: &str,
  reason_code: JuryConstraintRelaxationReason,
  metadata: Value,
  pool_size_at_relax: i32,
  panel_size_target: i32,
) -> LemmyResult<()> {
  let form = JuryConstraintViolationLogInsertForm {
    case_id: case.id,
    constraint_name: constraint_name.to_string(),
    reason_code,
    relaxation_metadata: Some(metadata.clone()),
    pool_size_at_relax,
    panel_size_target,
  };
  insert_into(jury_constraint_violation_log::table)
    .values(&form)
    .execute(conn)
    .await?;

  // `reason_code` serialises to its snake_case shape per the enum's serde
  // rename_all attribute. The fallback to Null is belt-and-braces — a
  // fixed-shape enum cannot actually fail to_value at runtime.
  let reason_code_json =
    serde_json::to_value(reason_code).unwrap_or(Value::Null);
  governance_log::append(
    &mut conn.into(),
    ENTRY_KIND_JURY_CONSTRAINT_RELAXED,
    json!({
      "case_id": case.id.0,
      "constraint_name": constraint_name,
      "reason_code": reason_code_json,
      "pool_size_at_relax": pool_size_at_relax,
      "panel_size_target": panel_size_target,
      "metadata": metadata,
    }),
    None,
  )
  .await?;

  Ok(())
}

/// Phase 4 eligibility filter — "not deleted AND accepted_application AND
/// not in `excluded`". Preserved so the post-R1 small-pool fallback path
/// has a well-defined relaxed set. Does NOT consult `reputation_snapshot`;
/// does NOT apply the concurrent-cap.
///
/// The `excluded` slice carries the full exclusion set built in
/// [`select_eligible_jurors`] (case.target + case.creator + caller-provided
/// ids from `admin_emergency_remove` / decline-replacement paths); dropping
/// it would let fallback re-seat already-removed or already-picked jurors.
async fn legacy_select_eligible_jurors(
  conn: &mut diesel_async::AsyncPgConnection,
  excluded: &[PersonId],
  panel_size: i64,
) -> LemmyResult<Vec<PersonId>> {
  let mut query = person::table
    .inner_join(local_user::table)
    .filter(person::deleted.eq(false))
    .filter(local_user::accepted_application.eq(true))
    .into_boxed();
  for person_id in excluded {
    query = query.filter(person::id.ne(*person_id));
  }

  query
    .order(random())
    .limit(panel_size)
    .select(person::id)
    .load::<PersonId>(conn)
    .await
    .map_err(Into::into)
}
