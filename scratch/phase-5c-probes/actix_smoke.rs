// Phase 5c task 68 probe — verify that `lemmy_api_routes::config` binds
// at the signature DQ #19 claims: `pub fn config(cfg: &mut ServiceConfig,
// rate_limit: &RateLimit)`. Compile-time probe only; no runtime assertion.
//
// Build: cargo check -p brehon_probe_actix_smoke --features full
//
// If this file fails to compile, DQ #19's answer is stale — task 68's
// test wiring needs a different import path or signature. The plan's
// §11.8 68c snippet would then need updating.
//
// Also statically counts the governance scope's routes by grep on
// lib.rs (done in the companion shell probe). If the count drops below
// 9 (the 8 existing + new routes 5c adds), the route-registration
// inventory is broken.

use actix_web::web::ServiceConfig;
use lemmy_utils::rate_limit::RateLimit;

#[allow(dead_code)]
fn binds_at_expected_signature(cfg: &mut ServiceConfig, rate_limit: &RateLimit) {
    lemmy_api_routes::config(cfg, rate_limit);
}
