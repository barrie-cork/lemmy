use actix_web::web::Json;
use lemmy_db_views_site::api::GetSourceResponse;
use lemmy_utils::error::LemmyResult;

const AGPL_NOTICE: &str = include_str!("../../../../../AGPL-NOTICE.md");

/// `GET /api/v4/source` — returns the verbatim AGPL-NOTICE.md body.
///
/// Public endpoint; no auth required. Read-only; no DB access.
/// Surfaces the AGPL §13 source-disclosure requirement to any connecting client.
pub fn get_source() -> LemmyResult<Json<GetSourceResponse>> {
  Ok(Json(GetSourceResponse {
    notice: AGPL_NOTICE.to_string(),
    license: "AGPL-3.0".to_string(),
  }))
}
