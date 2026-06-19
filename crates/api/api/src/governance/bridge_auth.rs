use actix_web::HttpRequest;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

/// Verifies the `Authorization: Bearer <secret>` header for bridge-to-binary callbacks.
/// Service-principal auth — NOT is_admin, NOT JWT. Called as first line of T4a and T4b handlers.
pub fn verify_bridge_secret(req: &HttpRequest) -> LemmyResult<()> {
  // Hard error on unset secret: a missing BRIDGE_CALLBACK_SECRET is a config fault,
  // not a silently-everything-rejected state (this is a service-principal auth gate).
  let expected = std::env::var("BRIDGE_CALLBACK_SECRET").map_err(|_e| LemmyErrorType::NotLoggedIn)?;
  let provided = req
    .headers()
    .get("Authorization")
    .and_then(|v| v.to_str().ok())
    .and_then(|s| s.strip_prefix("Bearer "))
    .unwrap_or("");
  // Plain == is acceptable for v0 (constant-time compare adds a dep for low-value gain)
  if provided.is_empty() || provided != expected {
    return Err(LemmyErrorType::NotLoggedIn.into());
  }
  Ok(())
}
