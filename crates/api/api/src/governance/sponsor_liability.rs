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
//! ## GOTCHAs (from plan §11.1)
//!
//! - **56a** (severity for `Restoration` → Minor): restorative sanctions imply
//!   the target violated norms, so the sponsor chain still bears softest-bucket
//!   accountability. v1 may refine `Restoration` into specific variants with
//!   their own severities.
//! - **56c** (integer-math drift): `round_ties_even` is banker's rounding on
//!   stable Rust 1.77+; avoids the `as i64` truncate-toward-zero bias across
//!   many sponsors.
//! - **56d** (zero-sponsor early return): returns `Ok(0)` without any writes so
//!   Phase 4's golden-path test (target with no sureties) remains unchanged.
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
use lemmy_db_schema_file::enums::{ReputationDimension, SanctionAction};
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
    return Ok(0);
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

    let final_delta_i32: i32 = i32::try_from(final_delta).map_err(|_e| {
      LemmyErrorType::Unknown(format!(
        "sponsor-liability delta {final_delta} overflows i32"
      ))
    })?;

    let form = ReputationEventInsertForm {
      person_id: sponsor_id,
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
      actor_pseudonym_helper::get_or_create(&mut (&mut *conn).into(), sponsor_id).await?;

    governance_log::append(
      &mut (&mut *conn).into(),
      governance_log::ENTRY_KIND_SPONSOR_LIABILITY_APPLIED,
      json!({
        "case_id": case_id.0,
        "sponsor_pseudonym": sponsor_pseudonym,
        "severity": severity.as_str(),
        "pre_multiplier_delta": pre_multiplier_delta,
        "multiplier": multiplier,
        "post_multiplier_delta": post_multiplier_delta,
        "final_delta": final_delta,
        "is_founder": is_founder,
      }),
      Some(sponsor_pseudonym.clone()),
    )
    .await?;

    if let Some(uncapped) = clamped_from {
      governance_log::append(
        &mut (&mut *conn).into(),
        governance_log::ENTRY_KIND_SPONSOR_LIABILITY_CLAMPED,
        json!({
          "case_id": case_id.0,
          "sponsor_pseudonym": sponsor_pseudonym,
          "uncapped_delta": uncapped,
          "clamped_delta": final_delta,
          "floor": floor,
          "current_endorsement_strength": current_endorsement_strength,
        }),
        Some(sponsor_pseudonym),
      )
      .await?;
    }
  }

  Ok(sponsor_count)
}
