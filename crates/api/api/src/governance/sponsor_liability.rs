//! Sponsor-liability applier.
//!
//! Invoked by `submit_jury_vote::process_vote` at step 8.5 (between sanction
//! insert and case-flip-to-Decided) when the decided case produced a sanction
//! (`NoAction` short-circuits before this call). For each active sponsor of the
//! target, computes a reputation delta against `EndorsementStrength`, applies
//! founder-vs-regular multiplier, rounds half-to-even, clamps to the honour-price
//! floor, writes `reputation_event`, and emits `sponsor_liability_applied` (plus
//! `sponsor_liability_clamped` if the floor clamp fired) to `governance_log`.
//!
//! ## v1-SL-d split
//!
//! `apply_sponsor_liability` is a thin wrapper around two pub(crate) helpers:
//!
//! - `compute_sponsor_liability` — pure read; returns `Vec<SponsorDelta>`.
//! - `fire_sponsor_liability` — DB writes only; iterates `SponsorDelta` slice.
//!
//! This split lets the v1 `submit_jury_vote` rewrite call
//! `compute_sponsor_liability` at vote-tally time (to know whether active
//! sureties exist) without writing anything, then defer the actual writes to
//! SL-c's scheduler at grace-window expiry.
//!
//! ## Watch 10 — PII discipline
//!
//! Governance-log payloads carry `sponsor_pseudonym` (UUID string), never raw
//! `sponsor_id` / `target_person_id` / any `PersonId(i32)`. The pseudonym is
//! sourced via `actor_pseudonym_helper::get_or_create`.
//!
//! ## Watch 3 — exhaustive match
//!
//! `severity_for_action` enumerates every `SanctionAction` variant; NO `_ =>`
//! wildcard, so adding a new variant forces a compile-time decision about its
//! severity bucket.
//!
//! `liability_severity_from_case_severity` enumerates every `CaseSeverity`
//! variant; NO `_ =>` wildcard per ADR-013.
//!
//! ## GOTCHAs (from plan §11.1)
//!
//! - **56a** (severity for `Restoration` → Minor): restorative sanctions imply
//!   the target violated norms, so the sponsor chain still bears softest-bucket
//!   accountability. v1 may refine `Restoration` into specific variants with
//!   their own severities.
//! - **56c** (integer-math drift): `round_ties_even` is banker's rounding on
//!   stable Rust 1.77+; avoids the `as i64` truncate-toward-zero bias across
//!   many sponsors.
//! - **56d** (zero-sponsor early return): returns `Ok(vec![])` from compute
//!   (empty vec) so the wrapper returns `Ok(0)` without any writes, preserving
//!   Phase 4's golden-path test (target with no sureties) unchanged.
//! - **56e** (transaction scope): the `conn` is the outer `run_transaction`
//!   connection. DO NOT nest `run_transaction`. Reads see pre-write state of
//!   `reputation_snapshot` + `reputation_event`; snapshot refresh is the 15-min
//!   job's job, per OQ-024 semantics.
//! - **56f** (founder timestamp — OQ-022): founder-event `expires_at.gt(now())`
//!   where `now()` inside a tx = `transaction_timestamp()`, which semantically
//!   equals case-close ± query time.
//! - **56g** (no race): vote-tally flips case to Decided under the same tx, so
//!   concurrent endorsement writes against the same case's target cannot race.

use crate::governance::{
  actor_pseudonym_helper,
  config::{self, ConfigCache, Scope},
  governance_log,
};
use diesel::{
  ExpressionMethods,
  OptionalExtension,
  QueryDsl,
  dsl::{exists, now, select},
  insert_into,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema::newtypes::{CommunityId, ModerationCaseId};
use lemmy_db_schema::source::governance::reputation_event::ReputationEventInsertForm;
use lemmy_db_schema_file::PersonId;
use lemmy_db_schema_file::enums::{CaseSeverity, ReputationDimension, SanctionAction};
use lemmy_db_schema_file::schema::{reputation_event, reputation_snapshot, surety};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;

/// Severity bucket for sponsor-liability delta lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LiabilitySeverity {
  Minor,
  Moderate,
  Severe,
}

impl LiabilitySeverity {
  fn config_key(self) -> &'static str {
    match self {
      Self::Minor => "deltas.sponsor_liability_minor",
      Self::Moderate => "deltas.sponsor_liability_moderate",
      Self::Severe => "deltas.sponsor_liability_severe",
    }
  }

  fn as_str(self) -> &'static str {
    match self {
      Self::Minor => "minor",
      Self::Moderate => "moderate",
      Self::Severe => "severe",
    }
  }
}

/// Per-sponsor liability delta computed by `compute_sponsor_liability`.
///
/// Field names mirror the v0 closure-locals so `fire_sponsor_liability` can
/// re-emit byte-identical `governance_log` payloads without recomputing.
/// `PartialEq` (not `Eq`) because `f64` does not implement `Eq`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SponsorDelta {
  pub(crate) sponsor_id: PersonId,
  pub(crate) pre_multiplier_delta: i64,
  pub(crate) multiplier: f64,
  pub(crate) post_multiplier_delta: i64,
  pub(crate) final_delta: i64,
  /// `Some(uncapped)` when the floor clamp fired; `None` otherwise.
  pub(crate) clamped_from: Option<i64>,
  pub(crate) is_founder: bool,
  pub(crate) current_endorsement_strength: i64,
}

/// Integer-safe multiply + banker's rounding (GOTCHA-56c).
///
/// `i64 → f64` is exact for `|x| ≤ 2^53`; every v0 sponsor-liability delta
/// (base cap `-200`, founder multiplier `2.0`) stays far below that envelope.
/// `round_ties_even` (stable Rust 1.77+) avoids the cumulative bias of
/// `as i64`'s trunc-toward-zero across many sponsors. The `f64 → i64`
/// cast is defended by the upstream call site clamping `final_delta` through
/// `i32::try_from` before the insert — NaN / ±Inf cannot sneak through the
/// multiplier (config is a plain `f64` seeded from the migration).
///
#[expect(
  clippy::as_conversions,
  reason = "i64→f64 safe for v0 magnitude; f64→i64 after round_ties_even — guarded by i32::try_from at insert"
)]
fn multiply_and_round(pre_multiplier_delta: i64, multiplier: f64) -> i64 {
  let product = (pre_multiplier_delta as f64) * multiplier;
  product.round_ties_even() as i64
}

/// Map a `SanctionAction` to its severity bucket. Exhaustive — no `_ =>`
/// wildcard, so a new variant forces a compile-time decision. See GOTCHA-56a.
fn severity_for_action(action: SanctionAction) -> LiabilitySeverity {
  match action {
    SanctionAction::Label
    | SanctionAction::VisibilityReduction
    | SanctionAction::Restoration => LiabilitySeverity::Minor,
    SanctionAction::TemporaryRestriction | SanctionAction::ContentRemoval => {
      LiabilitySeverity::Moderate
    }
    SanctionAction::CommunityExclusion
    | SanctionAction::InstanceSuspension
    | SanctionAction::FederationQuarantineRecommendation => LiabilitySeverity::Severe,
  }
}

/// Map a `CaseSeverity` to its liability severity bucket. Exhaustive — no
/// `_ =>` wildcard per ADR-013. Both `High` and `Critical` map to `Severe`
/// (the longest grace window, 168 h by default) so the harshest sanctions
/// always give sponsors the most time to revoke.
#[allow(dead_code)]
fn liability_severity_from_case_severity(severity: CaseSeverity) -> LiabilitySeverity {
  match severity {
    CaseSeverity::Low => LiabilitySeverity::Minor,
    CaseSeverity::Medium => LiabilitySeverity::Moderate,
    CaseSeverity::High => LiabilitySeverity::Severe,
    CaseSeverity::Critical => LiabilitySeverity::Severe,
  }
}

/// Read the grace-window duration for a case severity tier.
///
/// Reads `liability.grace_window_<bucket>_hours` at `Scope::Instance` per
/// DQ #178 (LOCKED — instance-only; no community cascade). Returns
/// `chrono::Duration::hours(value)`.
///
/// Pure read; idempotent. Severity snapshot semantics: the caller passes
/// `case_row.severity` (snapshotted at jury-assemble time per ADR-010 — not
/// re-derived from config at grace-window computation time).
#[allow(dead_code)]
pub(crate) async fn grace_window_for_severity(
  severity: CaseSeverity,
  cache: &mut ConfigCache,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<chrono::Duration> {
  let liability_sev = liability_severity_from_case_severity(severity);
  let key = match liability_sev {
    LiabilitySeverity::Minor => "liability.grace_window_minor_hours",
    LiabilitySeverity::Moderate => "liability.grace_window_moderate_hours",
    LiabilitySeverity::Severe => "liability.grace_window_severe_hours",
  };
  let hours: i64 =
    config::get_int(cache, &mut (&mut *conn).into(), Scope::Instance, key).await?;
  Ok(chrono::Duration::hours(hours))
}

/// Pure-read phase of sponsor-liability computation.
///
/// Queries `surety` for active sponsors, reads config keys (severity bucket
/// delta + floor + founder/regular multipliers), looks up
/// `reputation_snapshot.endorsement_strength` per sponsor for clamp math,
/// computes per-sponsor base + remainder + multiplier + clamp arithmetic.
///
/// Returns an empty `Vec` when `target_person_id` has no active sureties
/// (GOTCHA-56d — zero-sponsor early return; wrapper then returns `Ok(0)`).
///
/// **NO `reputation_event` INSERT, NO `governance_log::append`, NO UPDATE.**
pub(crate) async fn compute_sponsor_liability(
  conn: &mut AsyncPgConnection,
  target_person_id: PersonId,
  case_id: ModerationCaseId,
  community_id: Option<CommunityId>,
  action: SanctionAction,
  cache: &mut ConfigCache,
) -> LemmyResult<Vec<SponsorDelta>> {
  let _ = case_id; // not used by compute; preserved for signature symmetry with fire

  let severity = severity_for_action(action);
  let raw_delta: i64 = config::get_int(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    severity.config_key(),
  )
  .await?;

  let sponsor_ids: Vec<PersonId> = surety::table
    .filter(surety::sponsored_id.eq(target_person_id))
    .filter(surety::revoked_at.is_null())
    .select(surety::sponsor_id)
    .order_by(surety::sponsor_id.asc())
    .load::<PersonId>(conn)
    .await?;

  let sponsor_count = sponsor_ids.len();
  if sponsor_count == 0 {
    return Ok(vec![]);
  }

  let sponsor_count_i64 = i64::try_from(sponsor_count).map_err(|_e| {
    LemmyErrorType::Unknown(format!("sponsor count {sponsor_count} overflows i64"))
  })?;
  let per_sponsor_base: i64 = raw_delta / sponsor_count_i64;
  let remainder: i64 = raw_delta % sponsor_count_i64;
  let remainder_abs = remainder.unsigned_abs();
  let extra_unit: i64 = if raw_delta < 0 { -1 } else { 1 };

  let floor: i64 = config::get_int(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "liability.sponsor_liability_floor",
  )
  .await?;

  let founder_multiplier: f64 = config::get_float(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "liability.founder_multiplier",
  )
  .await?;
  let regular_multiplier: f64 = config::get_float(
    cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "liability.regular_multiplier",
  )
  .await?;

  let mut deltas = Vec::with_capacity(sponsor_count);

  for (i, sponsor_id) in sponsor_ids.iter().copied().enumerate() {
    let i_u64 = u64::try_from(i).map_err(|_e| {
      LemmyErrorType::Unknown(format!("sponsor index {i} overflows u64"))
    })?;
    let remainder_bump: i64 = if i_u64 < remainder_abs {
      extra_unit
    } else {
      0
    };
    let pre_multiplier_delta: i64 = per_sponsor_base + remainder_bump;

    // Founder check (OQ-022 — now() == transaction_timestamp() == case-close ± ms).
    // Restrict to reason="founder_seed" so non-founder expiring EndorsementStrength
    // writes (none today, but defence-in-depth against future drift) cannot grant
    // the founder multiplier. Mirrors count_active_founders at
    // crates/tools/seed_founders/src/main.rs:142.
    let is_founder: bool = select(exists(
      reputation_event::table
        .filter(reputation_event::person_id.eq(sponsor_id))
        .filter(reputation_event::dimension.eq(ReputationDimension::EndorsementStrength))
        .filter(reputation_event::reason.eq("founder_seed"))
        .filter(reputation_event::expires_at.is_not_null())
        .filter(reputation_event::expires_at.gt(now)),
    ))
    .get_result::<bool>(conn)
    .await?;

    let multiplier: f64 = if is_founder {
      founder_multiplier
    } else {
      regular_multiplier
    };

    let post_multiplier_delta = multiply_and_round(pre_multiplier_delta, multiplier);

    // Clamp-math snapshot lookup: prefer community-scoped row if present
    // (captures community-local endorsement context from Phase 5a
    // create_endorsement writes), fall back to instance-scoped (founder
    // seeds + prior sponsor-liabilities, both instance-scoped post
    // decision-queue #16). Returning 0 on full miss preserves the
    // "no baseline capital" semantic. Canonical pattern mirrors
    // read_reputation_summary at db_views/reputation/src/impls.rs:86-128
    // — two independent scope queries, no union, no cross-scope ordering
    // (the previous union-with-DESC let a newer instance row override the
    // community row; that is exactly the split-plane bug class we cleaned
    // up in decision-queue #16).
    let current_endorsement_strength: i64 = match community_id {
      Some(cid) => {
        let community_value = reputation_snapshot::table
          .filter(reputation_snapshot::person_id.eq(sponsor_id))
          .filter(reputation_snapshot::community_id.eq(cid))
          .select(reputation_snapshot::endorsement_strength)
          .first::<i32>(conn)
          .await
          .optional()?
          .map(i64::from);
        match community_value {
          Some(v) => v,
          None => reputation_snapshot::table
            .filter(reputation_snapshot::person_id.eq(sponsor_id))
            .filter(reputation_snapshot::community_id.is_null())
            .select(reputation_snapshot::endorsement_strength)
            .first::<i32>(conn)
            .await
            .optional()?
            .map(i64::from)
            .unwrap_or(0),
        }
      }
      None => reputation_snapshot::table
        .filter(reputation_snapshot::person_id.eq(sponsor_id))
        .filter(reputation_snapshot::community_id.is_null())
        .select(reputation_snapshot::endorsement_strength)
        .first::<i32>(conn)
        .await
        .optional()?
        .map(i64::from)
        .unwrap_or(0),
    };

    let mut final_delta: i64 = post_multiplier_delta;
    let mut clamped_from: Option<i64> = None;
    if current_endorsement_strength + final_delta < floor {
      clamped_from = Some(final_delta);
      final_delta = floor - current_endorsement_strength;
    }

    deltas.push(SponsorDelta {
      sponsor_id,
      pre_multiplier_delta,
      multiplier,
      post_multiplier_delta,
      final_delta,
      clamped_from,
      is_founder,
      current_endorsement_strength,
    });
  }

  Ok(deltas)
}

/// Write phase of sponsor-liability application.
///
/// Iterates a pre-computed `deltas` slice (from `compute_sponsor_liability`)
/// and for each entry: INSERTs a `reputation_event` row, appends a
/// `sponsor_liability_applied` governance-log entry, and optionally appends
/// `sponsor_liability_clamped` when the floor clamp fired.
///
/// Returns the number of sponsors processed (== `deltas.len()`).
///
/// # Transaction discipline (GOTCHA-56e)
///
/// `conn` is the outer `run_transaction` connection from `process_vote` or
/// from SL-c's per-case `run_transaction`. DO NOT call `run_transaction`
/// inside this function.
pub(crate) async fn fire_sponsor_liability(
  conn: &mut AsyncPgConnection,
  _target_person_id: PersonId,
  case_id: ModerationCaseId,
  _community_id: Option<CommunityId>,
  action: SanctionAction,
  deltas: &[SponsorDelta],
  _cache: &mut ConfigCache,
) -> LemmyResult<usize> {
  let severity = severity_for_action(action);

  for delta in deltas {
    let final_delta_i32: i32 = i32::try_from(delta.final_delta).map_err(|_e| {
      LemmyErrorType::Unknown(format!(
        "sponsor-liability delta {} overflows i32",
        delta.final_delta
      ))
    })?;

    let form = ReputationEventInsertForm {
      person_id: delta.sponsor_id,
      // Split-plane fix (decision-queue #16): write instance-scoped.
      // The clamp read above unions `community_id IS NULL OR = cid`
      // so it always sees instance-scoped rows (including founder seeds,
      // which are instance-scoped). Writing community-scoped here left
      // the liability on a plane load_live_events filters out of the
      // instance recompute, so founder +seed / −liability never composed.
      // `source_case_id` preserves the per-case audit linkage regardless
      // of plane.
      community_id: None,
      dimension: ReputationDimension::EndorsementStrength,
      delta: final_delta_i32,
      source_case_id: Some(case_id),
      source_report_id: None,
      reason: "sponsor_liability_applied".to_string(),
      expires_at: None,
    };
    insert_into(reputation_event::table)
      .values(&form)
      .execute(conn)
      .await?;

    let sponsor_pseudonym =
      actor_pseudonym_helper::get_or_create(&mut (&mut *conn).into(), delta.sponsor_id).await?;

    governance_log::append(
      &mut (&mut *conn).into(),
      governance_log::ENTRY_KIND_SPONSOR_LIABILITY_APPLIED,
      json!({
        "case_id": case_id.0,
        "sponsor_pseudonym": sponsor_pseudonym,
        "severity": severity.as_str(),
        "pre_multiplier_delta": delta.pre_multiplier_delta,
        "multiplier": delta.multiplier,
        "post_multiplier_delta": delta.post_multiplier_delta,
        "final_delta": delta.final_delta,
        "is_founder": delta.is_founder,
      }),
      Some(sponsor_pseudonym.clone()),
    )
    .await?;

    if let Some(uncapped) = delta.clamped_from {
      // Reconstruct floor from clamped delta math: final_delta = floor - current_endorsement_strength
      // therefore floor = final_delta + current_endorsement_strength.
      let floor = delta.final_delta + delta.current_endorsement_strength;
      governance_log::append(
        &mut (&mut *conn).into(),
        governance_log::ENTRY_KIND_SPONSOR_LIABILITY_CLAMPED,
        json!({
          "case_id": case_id.0,
          "sponsor_pseudonym": sponsor_pseudonym,
          "uncapped_delta": uncapped,
          "clamped_delta": delta.final_delta,
          "floor": floor,
          "current_endorsement_strength": delta.current_endorsement_strength,
        }),
        Some(sponsor_pseudonym),
      )
      .await?;
    }
  }

  Ok(deltas.len())
}

/// Apply sponsor-liability deltas to all active sponsors of `target_person_id`.
///
/// # Ordering (Watch 3)
///
/// For each sponsor, in `sponsor_id ASC` order:
///   raw_delta (from config, negative)
///   → /sponsor_count (trunc-toward-zero)
///   → first `|remainder|` sponsors receive one extra unit toward `raw_delta`'s sign
///   → ×multiplier (`founder_multiplier` if sponsor has unexpired founder event, else `regular_multiplier`)
///   → round half-to-even back to i64
///   → clamp against `liability.sponsor_liability_floor` and current endorsement_strength
///
/// # Returns
///
/// Count of sponsors processed (== number of `reputation_event` rows and
/// `sponsor_liability_applied` log entries written).
pub(crate) async fn apply_sponsor_liability(
  conn: &mut AsyncPgConnection,
  target_person_id: PersonId,
  case_id: ModerationCaseId,
  community_id: Option<CommunityId>,
  action: SanctionAction,
  cache: &mut ConfigCache,
) -> LemmyResult<usize> {
  let deltas =
    compute_sponsor_liability(conn, target_person_id, case_id, community_id, action, cache)
      .await?;
  fire_sponsor_liability(conn, target_person_id, case_id, community_id, action, &deltas, cache)
    .await
}
