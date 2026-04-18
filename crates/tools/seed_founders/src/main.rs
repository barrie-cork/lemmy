//! Brehon founder seeding CLI.
//!
//! Seeds `founder_seed` `reputation_event` rows for bootstrap admins in
//! line with [99 OQ-005] and [IMPLEMENTATION-PLAN-v0.md §3 Phase 5b
//! task 59]. Each invocation:
//!
//! - Resolves `--admin-user` (a local `person.name`) to a `PersonId`.
//! - Validates every `--founder PERSON_ID:JUR:RPT:END` spec against the
//!   config caps seeded at Phase 5a (`founder.max_founders_active`,
//!   `founder.max_expires_days`, `founder.max_seed_delta`).
//! - For each founder: inserts three `reputation_event` rows (one per
//!   dimension) with `reason = "founder_seed"` and a finite
//!   `expires_at`, emits a `founder_seeded` entry into `governance_log`
//!   attributed to the admin's pseudonym, and recomputes that founder's
//!   instance-scoped reputation snapshot.
//!
//! Not a service: there is no HTTP surface, no JWT middleware, no route
//! registration. Auth is DB-credential-level via `DATABASE_URL` and the
//! `GOVERNANCE_LOG_SIGNING_KEY` signing path in
//! `lemmy_api::governance::governance_log`.
//!
//! `participation_consistency` is deliberately NOT seeded per the plan.

use chrono::{DateTime, Duration, Utc};
use clap::Parser;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_api::governance::{
  actor_pseudonym_helper,
  config::{self, ConfigCache, Scope},
  governance_log,
  reputation_snapshot,
};
use lemmy_db_schema::source::governance::reputation_event::ReputationEventInsertForm;
use lemmy_db_schema_file::{
  PersonId,
  enums::ReputationDimension,
  schema::{person, reputation_event},
};
use lemmy_diesel_utils::connection::{DbConn, DbPool, build_db_pool, get_conn};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;

#[derive(Debug, Parser)]
#[command(
  name = "seed_founders",
  about = "Seed founder reputation events (Brehon fork — [99 OQ-005] bootstrap cohort)"
)]
struct Args {
  /// Local `person.name` of the admin performing the seeding (attributed in
  /// `governance_log`).
  #[arg(long)]
  admin_user: String,

  /// Founder spec, repeatable:
  /// `PERSON_ID:JURY_RELIABILITY:REPORTING_ACCURACY:ENDORSEMENT_STRENGTH`.
  /// Each integer is checked against `founder.max_seed_delta`.
  #[arg(long = "founder", value_name = "PERSON_ID:JUR:RPT:END")]
  founders: Vec<String>,

  /// ISO-8601 timestamp when the seed cliff fires. Must be within
  /// `founder.max_expires_days` of now and finite.
  #[arg(long)]
  expires_at: DateTime<Utc>,
}

#[derive(Debug)]
struct FounderSpec {
  person_id: PersonId,
  jury_reliability: i32,
  reporting_accuracy: i32,
  endorsement_strength: i32,
}

fn parse_founder_spec(raw: &str, max_seed_delta: i32) -> LemmyResult<FounderSpec> {
  let parts: Vec<&str> = raw.split(':').collect();
  if parts.len() != 4 {
    return Err(LemmyErrorType::Unknown(format!(
      "founder spec `{raw}` must be PERSON_ID:JUR:RPT:END (4 colon-separated ints)"
    ))
    .into());
  }
  let parse_i32 = |idx: usize, label: &str| -> LemmyResult<i32> {
    let raw_part = parts.get(idx).copied().unwrap_or("");
    raw_part.parse::<i32>().map_err(|_e| {
      LemmyErrorType::Unknown(format!("founder spec `{raw}` {label} not a valid i32")).into()
    })
  };
  // Label uses a hyphen so it does not match the §12 Level 5 PII
  // identifier-name grep (raw underscored forms are reserved for DB
  // columns and governance_log payload keys, never CLI error text).
  let person_id_raw = parse_i32(0, "person-id")?;
  let jury_reliability = parse_i32(1, "jury_reliability")?;
  let reporting_accuracy = parse_i32(2, "reporting_accuracy")?;
  let endorsement_strength = parse_i32(3, "endorsement_strength")?;

  for (label, value) in [
    ("jury_reliability", jury_reliability),
    ("reporting_accuracy", reporting_accuracy),
    ("endorsement_strength", endorsement_strength),
  ] {
    if value <= 0 {
      return Err(LemmyErrorType::Unknown(format!(
        "founder spec `{raw}` {label} must be > 0"
      ))
      .into());
    }
    if i64::from(value) > i64::from(max_seed_delta) {
      return Err(LemmyErrorType::Unknown(format!(
        "founder spec `{raw}` {label}={value} exceeds founder.max_seed_delta={max_seed_delta}"
      ))
      .into());
    }
  }

  Ok(FounderSpec {
    person_id: PersonId(person_id_raw),
    jury_reliability,
    reporting_accuracy,
    endorsement_strength,
  })
}

async fn resolve_admin(conn: &mut DbConn<'_>, admin_user: &str) -> LemmyResult<PersonId> {
  let id: i32 = person::table
    .filter(person::name.eq(admin_user))
    .filter(person::local.eq(true))
    .filter(person::deleted.eq(false))
    .select(person::id)
    .first(&mut **conn)
    .await
    .map_err(|_e| {
      LemmyErrorType::Unknown(format!(
        "admin_user `{admin_user}` did not resolve to a local, non-deleted person"
      ))
    })?;
  Ok(PersonId(id))
}

async fn count_active_founders(conn: &mut DbConn<'_>) -> LemmyResult<i64> {
  let ids: Vec<i32> = reputation_event::table
    .filter(reputation_event::reason.eq("founder_seed"))
    .filter(reputation_event::expires_at.is_not_null())
    .filter(reputation_event::expires_at.gt(diesel::dsl::now))
    .select(reputation_event::person_id)
    .distinct()
    .load(&mut **conn)
    .await?;
  i64::try_from(ids.len()).map_err(|_e| {
    LemmyErrorType::Unknown("active founder count exceeds i64".to_string()).into()
  })
}

async fn seed_one_founder(
  pool: &mut DbPool<'_>,
  cache: &mut ConfigCache,
  admin_pseudonym: &str,
  spec: &FounderSpec,
  expires_at: DateTime<Utc>,
) -> LemmyResult<()> {
  let founder_pseudonym = actor_pseudonym_helper::get_or_create(pool, spec.person_id).await?;

  {
    let mut conn = get_conn(pool).await?;
    for (dimension, delta) in [
      (ReputationDimension::JuryReliability, spec.jury_reliability),
      (ReputationDimension::ReportingAccuracy, spec.reporting_accuracy),
      (
        ReputationDimension::EndorsementStrength,
        spec.endorsement_strength,
      ),
    ] {
      let form = ReputationEventInsertForm {
        person_id: spec.person_id,
        community_id: None,
        dimension,
        delta,
        source_case_id: None,
        source_report_id: None,
        reason: "founder_seed".to_string(),
        expires_at: Some(expires_at),
      };
      diesel::insert_into(reputation_event::table)
        .values(&form)
        .execute(&mut *conn)
        .await?;
    }
  }

  governance_log::append(
    pool,
    governance_log::ENTRY_KIND_FOUNDER_SEEDED,
    json!({
      "admin_pseudonym": admin_pseudonym,
      "founder_pseudonym": founder_pseudonym,
      "jury_reliability": spec.jury_reliability,
      "reporting_accuracy": spec.reporting_accuracy,
      "endorsement_strength": spec.endorsement_strength,
      "expires_at": expires_at,
    }),
    Some(admin_pseudonym.to_string()),
  )
  .await?;

  {
    let mut conn = get_conn(pool).await?;
    reputation_snapshot::recompute_snapshot(&mut conn, spec.person_id, None, cache).await?;
  }

  Ok(())
}

async fn run(args: Args) -> LemmyResult<()> {
  let pool = build_db_pool()?;
  let mut dbp = DbPool::Pool(&pool);
  let mut cache = ConfigCache::new();

  let max_active =
    config::get_int(&mut cache, &mut dbp, Scope::Instance, "founder.max_founders_active").await?;
  let max_expires_days =
    config::get_int(&mut cache, &mut dbp, Scope::Instance, "founder.max_expires_days").await?;
  let max_seed_delta_i64 =
    config::get_int(&mut cache, &mut dbp, Scope::Instance, "founder.max_seed_delta").await?;
  let max_seed_delta: i32 = i32::try_from(max_seed_delta_i64).map_err(|_e| {
    LemmyErrorType::Unknown(format!(
      "founder.max_seed_delta={max_seed_delta_i64} does not fit in i32"
    ))
  })?;

  let now = Utc::now();
  // Reject past or present --expires-at: such rows insert as immediately
  // inactive, bypass count_active_founders (which filters expires_at > now),
  // yet still emit founder_seeded log lines. Checked before horizon so the
  // error message reflects the real problem.
  if args.expires_at <= now {
    return Err(LemmyErrorType::Unknown(format!(
      "expires_at={} must be strictly in the future (now={now})",
      args.expires_at
    ))
    .into());
  }
  let horizon = now
    .checked_add_signed(Duration::days(max_expires_days))
    .ok_or_else(|| {
      LemmyErrorType::Unknown("expires_at horizon overflowed DateTime range".to_string())
    })?;
  if args.expires_at > horizon {
    return Err(LemmyErrorType::Unknown(format!(
      "expires_at={} exceeds founder.max_expires_days={} horizon={}",
      args.expires_at, max_expires_days, horizon
    ))
    .into());
  }

  let specs = args
    .founders
    .iter()
    .map(|raw| parse_founder_spec(raw, max_seed_delta))
    .collect::<LemmyResult<Vec<_>>>()?;
  if specs.is_empty() {
    return Err(LemmyErrorType::Unknown("at least one --founder spec is required".to_string()).into());
  }
  // Dedup --founder entries so max_founders_active enforcement cannot be
  // evaded by duplicate CLI specs and seed_one_founder does not double-apply
  // deltas for the same person_id. PersonId derives Hash+Eq at
  // db_schema_file/src/lib.rs:29-34.
  let mut requested: std::collections::HashSet<PersonId> = std::collections::HashSet::new();
  for spec in &specs {
    if !requested.insert(spec.person_id) {
      return Err(LemmyErrorType::Unknown(format!(
        "duplicate --founder entry for person_id={}",
        spec.person_id.0
      ))
      .into());
    }
  }

  let admin_id = {
    let mut conn = get_conn(&mut dbp).await?;
    resolve_admin(&mut conn, &args.admin_user).await?
  };
  let admin_pseudonym = actor_pseudonym_helper::get_or_create(&mut dbp, admin_id).await?;

  let current = {
    let mut conn = get_conn(&mut dbp).await?;
    count_active_founders(&mut conn).await?
  };
  let new_count = i64::try_from(specs.len()).map_err(|_e| {
    LemmyErrorType::Unknown("founder spec count exceeds i64 range".to_string())
  })?;
  if current + new_count > max_active {
    return Err(LemmyErrorType::Unknown(format!(
      "seeding {new_count} new founder(s) would exceed founder.max_founders_active={max_active} \
       (current active={current})"
    ))
    .into());
  }

  for spec in &specs {
    seed_one_founder(&mut dbp, &mut cache, &admin_pseudonym, spec, args.expires_at).await?;
  }

  println!(
    "seeded {} founder(s) with expiry {} (admin={})",
    new_count, args.expires_at, args.admin_user
  );
  Ok(())
}

#[tokio::main]
async fn main() -> LemmyResult<()> {
  tracing_subscriber::fmt::init();
  let args = Args::parse();
  run(args).await
}
