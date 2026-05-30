//! Admin endpoints for the sponsor-allowlist table (v1-RT-r4).
//!
//! Two handlers — `add` and `remove` — mirror `admin_set_config`
//! (`:383-558`) step ordering verbatim:
//!
//! 1. `is_admin` check — denial path is OUTSIDE `run_transaction`.
//! 2. Pseudonym acquisition (admin + target person) — outside tx.
//! 3. Write path inside `run_transaction`: db write then
//!    `governance_log::append` (append opens its OWN internal tx /
//!    SAVEPOINT per `governance_log.rs:287` — do NOT wrap in a second
//!    explicit tx).
//!
//! ADR-015: log payloads carry pseudonyms only, never raw `person_id`.
//! ADR-008: governance_log written via `append`, never direct INSERT.

use crate::governance::{
  actor_pseudonym_helper,
  governance_log::{self, ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED, ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED},
};
use actix_web::web::{Data, Json};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_api_common::governance::{
  AddSponsorAllowlist, AddSponsorAllowlistResponse, RemoveSponsorAllowlist,
  RemoveSponsorAllowlistResponse,
};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::source::governance::sponsor_allowlist::{
  SponsorAllowlist, SponsorAllowlistInsertForm, sponsor_allowlist_delete, sponsor_allowlist_insert,
};
use lemmy_db_schema_file::schema::sponsor_allowlist;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;

/// `POST /api/v4/governance/admin/sponsor-allowlist/add`
///
/// Inserts a new row into `sponsor_allowlist` and emits
/// `sponsor_allowlist_added` to the governance log.
pub async fn add(
  Json(data): Json<AddSponsorAllowlist>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AddSponsorAllowlistResponse>> {
  // Step 2: admin check — denial path is outside run_transaction.
  is_admin(&local_user_view)?;

  let admin_id = local_user_view.person.id;
  let pool = &mut context.pool();

  // Step 4: admin pseudonym (ADR-015 — never raw admin_id in log payload).
  let admin_pseudonym = actor_pseudonym_helper::get_or_create(pool, admin_id).await?;

  // Step 5: pseudonym for the person being allowlisted.
  let person_pseudonym = actor_pseudonym_helper::get_or_create(pool, data.person_id).await?;

  // Pre-clone for move into transaction closure.
  let admin_pseudonym_tx = admin_pseudonym.clone();
  let person_pseudonym_tx = person_pseudonym.clone();
  let person_id = data.person_id;
  let community_id = data.community_id;
  let note = data.note.clone();

  // Step 6: write inside run_transaction.
  let conn = &mut get_conn(pool).await?;
  let allowlist_id = conn
    .run_transaction(async move |conn| {
      let form = SponsorAllowlistInsertForm {
        community_id,
        person_id,
        added_by_admin_id: admin_id,
        note: note.clone(),
      };
      let row = sponsor_allowlist_insert(&form, conn).await?;

      let payload = json!({
        "allowlist_id":               row.id.0,
        "community_id":               community_id.map(|c| c.0),
        "person_pseudonym":           person_pseudonym_tx,
        "added_by_admin_pseudonym":   admin_pseudonym_tx,
        "note":                       note,
        "added_at":                   row.created_at,
      });

      governance_log::append(
        &mut conn.into(),
        ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED,
        payload,
        Some(admin_pseudonym.clone()),
      )
      .await?;

      Ok(row.id)
    })
    .await?;

  Ok(Json(AddSponsorAllowlistResponse { allowlist_id }))
}

/// `POST /api/v4/governance/admin/sponsor-allowlist/remove`
///
/// Deletes an existing `sponsor_allowlist` row (identified by
/// `person_id + community_id`) and emits `sponsor_allowlist_removed`
/// to the governance log.  The row must be resolved first to obtain
/// `allowlist_id` for the log payload (per plan §4 GOTCHA).
pub async fn remove(
  Json(data): Json<RemoveSponsorAllowlist>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<RemoveSponsorAllowlistResponse>> {
  // Step 2: admin check — denial path is outside run_transaction.
  is_admin(&local_user_view)?;

  let admin_id = local_user_view.person.id;
  let pool = &mut context.pool();

  // Step 4: admin pseudonym.
  let admin_pseudonym = actor_pseudonym_helper::get_or_create(pool, admin_id).await?;

  // Step 5: pseudonym for the person being removed from the allowlist.
  let person_pseudonym = actor_pseudonym_helper::get_or_create(pool, data.person_id).await?;

  // Pre-clone for move into transaction closure.
  let admin_pseudonym_tx = admin_pseudonym.clone();
  let person_pseudonym_tx = person_pseudonym.clone();
  let person_id = data.person_id;
  let community_id = data.community_id;

  // Step 6: write inside run_transaction.
  let conn = &mut get_conn(pool).await?;
  conn
    .run_transaction(async move |conn| {
      // Resolve the row first — we need allowlist_id for the log payload.
      let row: SponsorAllowlist = match community_id {
        Some(cid) => {
          sponsor_allowlist::table
            .filter(sponsor_allowlist::person_id.eq(person_id))
            .filter(sponsor_allowlist::community_id.eq(cid))
            .select(SponsorAllowlist::as_select())
            .first(conn)
            .await
            .optional()?
        }
        None => {
          sponsor_allowlist::table
            .filter(sponsor_allowlist::person_id.eq(person_id))
            .filter(sponsor_allowlist::community_id.is_null())
            .select(SponsorAllowlist::as_select())
            .first(conn)
            .await
            .optional()?
        }
      }
      .ok_or(LemmyErrorType::NotFound)?;

      // Check deleted count — guards against a concurrent remove race where the
      // row disappears between the SELECT and DELETE inside the same tx.
      let deleted = sponsor_allowlist_delete(row.id, conn).await?;
      if deleted == 0 {
        return Err(LemmyErrorType::NotFound.into());
      }

      let payload = json!({
        "allowlist_id":                   row.id.0,
        "community_id":                   community_id.map(|c| c.0),
        "person_pseudonym":               person_pseudonym_tx,
        "removed_by_admin_pseudonym":     admin_pseudonym_tx,
        "removed_at":                     chrono::Utc::now(),
      });

      governance_log::append(
        &mut conn.into(),
        ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED,
        payload,
        Some(admin_pseudonym.clone()),
      )
      .await?;

      Ok(())
    })
    .await?;

  Ok(Json(RemoveSponsorAllowlistResponse { success: true }))
}
