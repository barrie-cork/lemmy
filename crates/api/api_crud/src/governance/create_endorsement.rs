//! `POST /api/v4/governance/endorsement` — create a peer-to-peer
//! endorsement of another user, optionally scoped to a community.
//!
//! Config-driven gate strategy per [99 OQ-014]:
//! `onboarding.sponsor_gate_strategy` ∈ {`"age"`, `"open"`, `"closed"`}
//! (fallback `"age"` for any unknown value; see [`SponsorGateStrategy`]).
//!
//! v0 behaviour:
//!
//! - `"closed"` rejects with `NotFound` (pilot-phase kill-switch).
//! - `"open"` bypasses the age gate but still applies the caller's
//!   active-endorsement cap and the cooldown window.
//! - `"age"` rejects if the caller's account age is below
//!   `onboarding.sponsor_min_account_age_days`.
//!
//! Every write (endorsement row, optional surety row, two reputation
//! events, two snapshot recomputes, one governance-log entry) runs inside
//! one `run_transaction` closure so a partial failure leaves no
//! half-assembled state. See GOTCHA-55e: `FOR UPDATE` on the existing
//! snapshot row serialises concurrent recomputes for the same
//! `(person_id, community_id)` pair.
//!
//! Per GOTCHA-55d the "max 5 active endorsements" check counts
//! `revoked_at IS NULL`, while the 48h-cooldown check counts all rows
//! (revoked or not) because the burst of intent happens on
//! `created_at`.
//!
//! Per GOTCHA-55h the two emitted reputation events have
//! `expires_at = None` — these are permanent organic events, not founder
//! seeds. Per GOTCHA-55b `reputation_snapshot.can_sponsor` is NOT read
//! here; v0 gates strictly on the `sponsor_gate_strategy` config
//! (OQ-014). The `lint-no-can-sponsor-read.sh` guard enforces that at
//! phase close.

use actix_web::web::{Data, Json};
use chrono::{DateTime, Duration, Utc};
use diesel::{
  ExpressionMethods,
  QueryDsl,
  dsl::count_star,
  insert_into,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_api::governance::{
  actor_pseudonym_helper,
  config::{self, ConfigCache, Scope},
  governance_log,
  reputation_snapshot,
};
use lemmy_api_common::governance::{CreateEndorsement, CreateEndorsementResponse};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_schema::source::{
  governance::{
    endorsement::{Endorsement, EndorsementInsertForm},
    reputation_event::ReputationEventInsertForm,
    surety::SuretyInsertForm,
  },
  person::Person,
};
use lemmy_db_schema_file::{
  PersonId,
  enums::ReputationDimension,
  schema::{endorsement, person, reputation_event, surety},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::{connection::get_conn, traits::Crud};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;
use tracing::warn;

/// v0 caller-side cap on concurrent active endorsements.
const MAX_ACTIVE_ENDORSEMENTS: i64 = 5;

/// v0 per-caller cooldown window between endorsements.
const ENDORSEMENT_COOLDOWN_HOURS: i64 = 48;

/// Cap on concurrent active sureties per sponsee. A new endorsement
/// conditionally inserts a surety row only if the sponsee has fewer
/// than this many already.
const MAX_ACTIVE_SURETIES_PER_SPONSEE: i64 = 2;

/// Parsed value of the `onboarding.sponsor_gate_strategy` config key.
///
/// GOTCHA-55a: `Unknown(String)` is an exhaustive-but-final arm rather
/// than `_ =>`; workspace clippy denies `_ =>` under
/// `feedback_clippy_test_style`. [`parse`](Self::parse) returns
/// `Unknown(s)` for anything outside the three v0 variants; the match
/// arm logs and falls through to `'age'` semantics inline.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SponsorGateStrategy {
  Age,
  Open,
  Closed,
  Unknown(String),
}

impl SponsorGateStrategy {
  fn parse(s: &str) -> Self {
    match s {
      "age" => Self::Age,
      "open" => Self::Open,
      "closed" => Self::Closed,
      other => Self::Unknown(other.to_string()),
    }
  }

  /// Canonical label for governance-log attribution.
  fn label(&self) -> &str {
    match self {
      Self::Age => "age",
      Self::Open => "open",
      Self::Closed => "closed",
      Self::Unknown(s) => s.as_str(),
    }
  }
}

pub async fn create_endorsement(
  Json(data): Json<CreateEndorsement>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CreateEndorsementResponse>> {
  check_local_user_valid(&local_user_view)?;

  let sponsor_id = local_user_view.person.id;
  let sponsor_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), sponsor_id).await?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let data_for_tx = data;
  let pseudonym_for_tx = sponsor_pseudonym.clone();

  let outcome = conn
    .run_transaction(|conn| {
      async move {
        process_endorsement(conn, sponsor_id, pseudonym_for_tx, data_for_tx).await
      }
      .scope_boxed()
    })
    .await?;

  Ok(Json(outcome))
}

/// Body of the `run_transaction` closure. Named helper so the outer
/// future stays under the workspace `large_futures` lint threshold
/// (mirror of `admin_assign_jury::process_assignment`).
async fn process_endorsement(
  conn: &mut AsyncPgConnection,
  sponsor_id: PersonId,
  sponsor_pseudonym: String,
  data: CreateEndorsement,
) -> LemmyResult<CreateEndorsementResponse> {
  let mut config = ConfigCache::new();

  // Step 1 — read the gate strategy from config.
  let strategy_str = config::get_text(
    &mut config,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "onboarding.sponsor_gate_strategy",
  )
  .await?;
  let strategy = SponsorGateStrategy::parse(&strategy_str);

  // Step 2 — dispatch on strategy. Unknown falls through to age semantics
  // per GOTCHA-55a (no `_ =>` catchall allowed).
  match &strategy {
    SponsorGateStrategy::Closed => {
      warn!("endorsement attempt under 'closed' gate from {sponsor_pseudonym}");
      return Err(LemmyErrorType::NotFound.into());
    }
    SponsorGateStrategy::Open => { /* bypass age gate */ }
    SponsorGateStrategy::Age => {
      enforce_age_gate(conn, &mut config, sponsor_id).await?;
    }
    SponsorGateStrategy::Unknown(s) => {
      warn!("unknown sponsor_gate_strategy '{s}' — falling back to 'age'");
      enforce_age_gate(conn, &mut config, sponsor_id).await?;
    }
  }

  // Step 3 — reject self-endorsement; confirm target exists.
  if data.person_id == sponsor_id {
    return Err(LemmyErrorType::NotFound.into());
  }
  Person::read(&mut (&mut *conn).into(), data.person_id).await?;

  // Step 4 — caller-side cap on active endorsements.
  let active_count: i64 = endorsement::table
    .filter(endorsement::from_person_id.eq(sponsor_id))
    .filter(endorsement::revoked_at.is_null())
    .select(count_star())
    .get_result(conn)
    .await?;
  if active_count >= MAX_ACTIVE_ENDORSEMENTS {
    return Err(LemmyErrorType::NotFound.into());
  }

  // Step 5 — 48h cooldown on caller. Counts revoked rows too
  // (GOTCHA-55d — the burst of intent is what's rate-limited, not the
  // live state).
  let cutoff: DateTime<Utc> = Utc::now() - Duration::hours(ENDORSEMENT_COOLDOWN_HOURS);
  let recent_count: i64 = endorsement::table
    .filter(endorsement::from_person_id.eq(sponsor_id))
    .filter(endorsement::created_at.gt(cutoff))
    .select(count_star())
    .get_result(conn)
    .await?;
  if recent_count > 0 {
    return Err(LemmyErrorType::NotFound.into());
  }

  // Step 6 — insert endorsement row.
  let form = EndorsementInsertForm {
    from_person_id: sponsor_id,
    to_person_id: data.person_id,
    community_id: data.community_id,
  };
  let inserted: Endorsement = insert_into(endorsement::table)
    .values(&form)
    .get_result(conn)
    .await?;

  // Step 7 — conditional surety row. Only inserted if the sponsee has
  // fewer than MAX_ACTIVE_SURETIES_PER_SPONSEE active sureties.
  let active_sureties: i64 = surety::table
    .filter(surety::sponsored_id.eq(data.person_id))
    .filter(surety::revoked_at.is_null())
    .select(count_star())
    .get_result(conn)
    .await?;
  let surety_created = if active_sureties < MAX_ACTIVE_SURETIES_PER_SPONSEE {
    let sf = SuretyInsertForm {
      sponsor_id,
      sponsored_id: data.person_id,
      community_id: data.community_id,
    };
    insert_into(surety::table)
      .values(&sf)
      .execute(conn)
      .await?;
    true
  } else {
    false
  };

  // Step 8 — emit two reputation events with expires_at=None per
  // GOTCHA-55h (permanent organic events, not founder seeds).
  let sponsor_delta = config::get_int(
    &mut config,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "deltas.endorsement_created_sponsor",
  )
  .await?;
  let sponsee_delta = config::get_int(
    &mut config,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "deltas.endorsement_created_sponsee",
  )
  .await?;
  let sponsor_delta_i32 = i32::try_from(sponsor_delta).map_err(|_e| {
    LemmyErrorType::Unknown(format!(
      "deltas.endorsement_created_sponsor out of range for i32: {sponsor_delta}"
    ))
  })?;
  let sponsee_delta_i32 = i32::try_from(sponsee_delta).map_err(|_e| {
    LemmyErrorType::Unknown(format!(
      "deltas.endorsement_created_sponsee out of range for i32: {sponsee_delta}"
    ))
  })?;

  insert_into(reputation_event::table)
    .values(ReputationEventInsertForm {
      person_id: sponsor_id,
      community_id: data.community_id,
      dimension: ReputationDimension::EndorsementStrength,
      delta: sponsor_delta_i32,
      source_case_id: None,
      source_report_id: None,
      reason: "endorsement_created_sponsor".to_string(),
      expires_at: None,
    })
    .execute(conn)
    .await?;
  insert_into(reputation_event::table)
    .values(ReputationEventInsertForm {
      person_id: data.person_id,
      community_id: data.community_id,
      dimension: ReputationDimension::ParticipationConsistency,
      delta: sponsee_delta_i32,
      source_case_id: None,
      source_report_id: None,
      reason: "endorsement_created_sponsee".to_string(),
      expires_at: None,
    })
    .execute(conn)
    .await?;

  // Step 9 — recompute both snapshots. FOR UPDATE inside
  // recompute_snapshot serialises against concurrent endorsements
  // targeting the same sponsee (GOTCHA-55e / Watch 9).
  reputation_snapshot::recompute_snapshot(conn, sponsor_id, data.community_id, &mut config).await?;
  reputation_snapshot::recompute_snapshot(conn, data.person_id, data.community_id, &mut config)
    .await?;

  // Step 10 — emit the governance-log entry. `append` uses its own
  // in-tx connection via `&mut conn.into()`; the redaction layer scrubs
  // the payload per ADR-015.
  let target_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut (&mut *conn).into(), data.person_id).await?;
  governance_log::append(
    &mut (&mut *conn).into(),
    governance_log::ENTRY_KIND_ENDORSEMENT_CREATED,
    json!({
      "sponsor_pseudonym": sponsor_pseudonym,
      "target_pseudonym":  target_pseudonym,
      "community_id":      data.community_id.map(|c| c.0),
      "gate_strategy":     strategy.label(),
      "surety_created":    surety_created,
    }),
    Some(sponsor_pseudonym),
  )
  .await?;

  Ok(CreateEndorsementResponse {
    endorsement_id: inserted.id,
    surety_created,
  })
}

/// Enforce the `"age"` gate (also the fallback for `Unknown`).
/// Fetches `onboarding.sponsor_min_account_age_days` and rejects if the
/// caller's account is younger.
async fn enforce_age_gate(
  conn: &mut AsyncPgConnection,
  config: &mut ConfigCache,
  sponsor_id: PersonId,
) -> LemmyResult<()> {
  let min_age = config::get_int(
    config,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "onboarding.sponsor_min_account_age_days",
  )
  .await?;
  let sponsor_published: DateTime<Utc> = person::table
    .filter(person::id.eq(sponsor_id))
    .select(person::published_at)
    .first(conn)
    .await?;
  let age_days = (Utc::now() - sponsor_published).num_days();
  if age_days < min_age {
    return Err(LemmyErrorType::NotFound.into());
  }
  Ok(())
}
