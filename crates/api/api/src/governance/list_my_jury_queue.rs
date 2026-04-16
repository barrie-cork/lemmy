//! `GET /api/v4/governance/jury/me` — the caller's jury queue.
//!
//! Returns every `JuryQueueView` row where the caller is assigned as a
//! juror, newest-first by `jury_assignment.selected_at`. No query params
//! in v0 per [04 §6.2] — filtering by status or community lands in
//! Phase 5.

use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_views_jury_queue::{JuryQueueView, impls::list_jury_assignments_for_person};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn list_my_jury_queue(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<Vec<JuryQueueView>>> {
  check_local_user_valid(&local_user_view)?;
  let rows =
    list_jury_assignments_for_person(&mut context.pool(), local_user_view.person.id).await?;
  Ok(Json(rows))
}
