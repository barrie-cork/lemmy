//! Multi-source participation cron — Sources 1 + 2 per PRD section 5.3.
//!
//! Runs every `job.participation_interval_days` (default 7): emits
//! `+1 participation_consistency` per active user per community
//! (Source 1) and `-2 participation_consistency` per dormant user
//! per community (Source 2). Per-community `run_transaction`
//! isolation per PRD section 5.3 + `feedback_multi_write_handlers_need_transactions.md`.
//!
//! ## Idempotency
//!
//! Both batches write `reputation_event` rows with `dedupe_key` set
//! to a per-(community, person, ISO week) string. The
//! `reputation_event_dedupe_key_partial_idx` partial unique index
//! (r1-shipped) catches repeat-tick inserts; Diesel's
//! `.on_conflict_do_nothing()` consumes the violation silently. A
//! second tick within the same ISO week is a no-op (zero new rows).
//!
//! ## Actor attribution (ADR-015 + DQ a3d0e9941441-018)
//!
//! `governance_log` entries from this module use `actor_pseudonym =
//! None` (system-attributed). Mirror v0 precedent at
//! `admin_assign_jury.rs:932` doc-comment.

use chrono::{Datelike, Duration, Utc};
use diesel::{
  QueryableByName,
  dsl::insert_into,
  sql_query,
  sql_types::{BigInt, Integer, Timestamptz},
};
use diesel_async::RunQueryDsl;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::governance::reputation_event::ReputationEventInsertForm,
};
use lemmy_db_schema_file::{
  PersonId,
  enums::{ReputationDimension, ReputationEventSourceType},
  schema::reputation_event,
};
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::LemmyResult;
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn};

use crate::governance::{
  config::{self, ConfigCache, Scope},
  governance_log::{self, ENTRY_KIND_PARTICIPATION_CRON_TICK},
};

#[derive(QueryableByName)]
struct ActivityRow {
  #[diesel(sql_type = Integer)]
  community_id: i32,
  #[diesel(sql_type = Integer)]
  creator_id: i32,
}

#[derive(QueryableByName)]
struct DormantPersonRow {
  #[diesel(sql_type = Integer)]
  community_id: i32,
  #[diesel(sql_type = Integer)]
  person_id: i32,
}

/// Per-tick counters returned for telemetry and test assertions.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ParticipationBatchOutcome {
  pub communities_processed: usize,
  pub events_emitted: usize,
  pub events_deduped: usize,
}

/// Source 1 — weekly activity cron. SELECTs per-(community, person)
/// comment counts in the lookback window; for each qualifying person,
/// emits `+1 participation_consistency` with dedupe-key idempotency.
pub async fn run_activity_batch(
  context: &LemmyContext,
) -> LemmyResult<ParticipationBatchOutcome> {
  let pool = &mut context.pool();
  let mut cache = ConfigCache::new();

  let raw_lookback_days = config::get_int(
    &mut cache,
    pool,
    Scope::Instance,
    "participation.lookback_days",
  )
  .await
  .unwrap_or(config::DEFAULT_PARTICIPATION_LOOKBACK_DAYS);
  let lookback_days = raw_lookback_days.max(1);
  if raw_lookback_days < 1 {
    warn!(
      "participation_activity_cron: invalid lookback_days={raw_lookback_days}; clamped to 1"
    );
  }

  let raw_activity_threshold = config::get_int(
    &mut cache,
    pool,
    Scope::Instance,
    "participation.activity_threshold_comments",
  )
  .await
  .unwrap_or(config::DEFAULT_PARTICIPATION_ACTIVITY_THRESHOLD_COMMENTS);
  let activity_threshold = raw_activity_threshold.max(1);
  if raw_activity_threshold < 1 {
    warn!(
      "participation_activity_cron: invalid activity_threshold_comments={raw_activity_threshold}; clamped to 1"
    );
  }

  let delta_active = i32::try_from(
    config::get_int(
      &mut cache,
      pool,
      Scope::Instance,
      "deltas.participation_weekly_active",
    )
    .await
    .unwrap_or(config::DEFAULT_DELTAS_PARTICIPATION_WEEKLY_ACTIVE),
  )
  .unwrap_or(1);

  let cutoff = Utc::now() - Duration::days(lookback_days);
  let iso = Utc::now().iso_week();
  let iso_week = format!("{}-W{:02}", iso.year(), iso.week());

  let conn = &mut get_conn(pool).await?;

  let rows: Vec<ActivityRow> = sql_query(
    "SELECT community_id, creator_id \
     FROM comment \
     WHERE published_at >= $1 \
       AND NOT deleted \
       AND NOT removed \
     GROUP BY community_id, creator_id \
     HAVING COUNT(*) >= $2",
  )
  .bind::<Timestamptz, _>(cutoff)
  .bind::<BigInt, _>(activity_threshold)
  .load(conn)
  .await?;

  let mut by_community: HashMap<CommunityId, Vec<PersonId>> = HashMap::new();
  for row in rows {
    by_community
      .entry(CommunityId(row.community_id))
      .or_default()
      .push(PersonId(row.creator_id));
  }

  let mut outcome = ParticipationBatchOutcome::default();

  for (community_id, person_ids) in by_community {
    let person_count = person_ids.len();
    let iso_week_tx = iso_week.clone();

    let tx_result = conn
      .run_transaction(async move |conn| {
        let mut emitted: usize = 0;
        let mut deduped: usize = 0;
        for person_id in &person_ids {
          let dk = format!(
            "activity_cron:{}:{}:{}",
            community_id.0, person_id.0, iso_week_tx
          );
          let form = ReputationEventInsertForm {
            person_id: *person_id,
            community_id: Some(community_id),
            dimension: ReputationDimension::ParticipationConsistency,
            delta: delta_active,
            source_case_id: None,
            source_report_id: None,
            reason: "participation_activity_cron".to_string(),
            expires_at: None,
            dedupe_key: Some(dk),
            source_event_type: Some(ReputationEventSourceType::ParticipationCron),
          };
          let affected = insert_into(reputation_event::table)
            .values(&form)
            .on_conflict_do_nothing()
            .execute(conn)
            .await?;
          if affected == 0 {
            deduped += 1;
          } else {
            emitted += 1;
          }
        }
        governance_log::append(
          &mut (&mut *conn).into(),
          ENTRY_KIND_PARTICIPATION_CRON_TICK,
          json!({
            "community_id": community_id.0,
            "iso_week": iso_week_tx,
            "active_user_count": person_count,
            "events_emitted": emitted,
            "events_deduped": deduped,
          }),
          None,
        )
        .await?;
        Ok((emitted, deduped))
      })
      .await;

    match tx_result {
      Ok((emitted, deduped)) => {
        outcome.communities_processed += 1;
        outcome.events_emitted += emitted;
        outcome.events_deduped += deduped;
      }
      Err(e) => {
        warn!(
          "participation_cron: activity batch tx failed for community_id={}: {e}",
          community_id.0
        );
      }
    }
  }

  if outcome.communities_processed > 0 {
    info!(
      "governance: participation-activity tick — communities={}, emitted={}, deduped={}",
      outcome.communities_processed, outcome.events_emitted, outcome.events_deduped
    );
  }
  Ok(outcome)
}

/// Source 2 — weekly dormancy cron. SELECTs (community, person) pairs
/// with a prior participation_consistency event but zero recent
/// comments; emits `-2 participation_consistency` with dedupe-key
/// idempotency.
pub async fn run_dormancy_batch(
  context: &LemmyContext,
) -> LemmyResult<ParticipationBatchOutcome> {
  let pool = &mut context.pool();
  let mut cache = ConfigCache::new();

  let raw_dormancy_window_days = config::get_int(
    &mut cache,
    pool,
    Scope::Instance,
    "participation.dormancy_window_days",
  )
  .await
  .unwrap_or(config::DEFAULT_PARTICIPATION_DORMANCY_WINDOW_DAYS);
  let dormancy_window_days = raw_dormancy_window_days.max(1);
  if raw_dormancy_window_days < 1 {
    warn!(
      "participation_dormancy_cron: invalid dormancy_window_days={raw_dormancy_window_days}; clamped to 1"
    );
  }

  let delta_dormant = i32::try_from(
    config::get_int(
      &mut cache,
      pool,
      Scope::Instance,
      "deltas.participation_dormant",
    )
    .await
    .unwrap_or(config::DEFAULT_DELTAS_PARTICIPATION_DORMANT),
  )
  .unwrap_or(-2);

  let cutoff = Utc::now() - Duration::days(dormancy_window_days);
  let iso = Utc::now().iso_week();
  let iso_week = format!("{}-W{:02}", iso.year(), iso.week());

  let conn = &mut get_conn(pool).await?;

  // LEFT ANTI JOIN pattern: users with a prior ParticipationConsistency
  // event in this community but no qualifying comment in the window.
  let rows: Vec<DormantPersonRow> = sql_query(
    "SELECT DISTINCT re.community_id::integer AS community_id, re.person_id \
     FROM reputation_event re \
     WHERE re.community_id IS NOT NULL \
       AND re.dimension = 'ParticipationConsistency' \
       AND NOT EXISTS ( \
         SELECT 1 FROM comment c \
         WHERE c.creator_id = re.person_id \
           AND c.community_id = re.community_id \
           AND c.published_at >= $1 \
           AND NOT c.deleted \
           AND NOT c.removed \
       )",
  )
  .bind::<Timestamptz, _>(cutoff)
  .load(conn)
  .await?;

  let mut by_community: HashMap<CommunityId, Vec<PersonId>> = HashMap::new();
  for row in rows {
    by_community
      .entry(CommunityId(row.community_id))
      .or_default()
      .push(PersonId(row.person_id));
  }

  let mut outcome = ParticipationBatchOutcome::default();

  for (community_id, person_ids) in by_community {
    let person_count = person_ids.len();
    let iso_week_tx = iso_week.clone();

    let tx_result = conn
      .run_transaction(async move |conn| {
        let mut emitted: usize = 0;
        let mut deduped: usize = 0;
        for person_id in &person_ids {
          let dk = format!(
            "dormancy_cron:{}:{}:{}",
            community_id.0, person_id.0, iso_week_tx
          );
          let form = ReputationEventInsertForm {
            person_id: *person_id,
            community_id: Some(community_id),
            dimension: ReputationDimension::ParticipationConsistency,
            delta: delta_dormant,
            source_case_id: None,
            source_report_id: None,
            reason: "participation_dormancy_cron".to_string(),
            expires_at: None,
            dedupe_key: Some(dk),
            source_event_type: Some(ReputationEventSourceType::DormancyCron),
          };
          let affected = insert_into(reputation_event::table)
            .values(&form)
            .on_conflict_do_nothing()
            .execute(conn)
            .await?;
          if affected == 0 {
            deduped += 1;
          } else {
            emitted += 1;
          }
        }
        governance_log::append(
          &mut (&mut *conn).into(),
          ENTRY_KIND_PARTICIPATION_CRON_TICK,
          json!({
            "community_id": community_id.0,
            "iso_week": iso_week_tx,
            "dormant_user_count": person_count,
            "events_emitted": emitted,
            "events_deduped": deduped,
          }),
          None,
        )
        .await?;
        Ok((emitted, deduped))
      })
      .await;

    match tx_result {
      Ok((emitted, deduped)) => {
        outcome.communities_processed += 1;
        outcome.events_emitted += emitted;
        outcome.events_deduped += deduped;
      }
      Err(e) => {
        warn!(
          "participation_cron: dormancy batch tx failed for community_id={}: {e}",
          community_id.0
        );
      }
    }
  }

  if outcome.communities_processed > 0 {
    info!(
      "governance: participation-dormancy tick — communities={}, emitted={}, deduped={}",
      outcome.communities_processed, outcome.events_emitted, outcome.events_deduped
    );
  }
  Ok(outcome)
}
