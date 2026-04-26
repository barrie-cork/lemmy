---
name: RateLimit::with_debug_config() Post bucket is 6/300s — trips route-shape tests
description: Actix integration tests iterating multiple POST routes through /governance scope (wrapped with rate_limit.post()) hit 429 after the 6th call; use set_config to raise limits for route-shape tests
type: feedback
originSessionId: bae44467-63e7-4c21-ad41-057cefebb80e
---
`RateLimit::with_debug_config()` (used as the permissive test default) sets:

```
Post: max_requests=6, interval=300s
```

Any integration test that hits >6 POST endpoints within the same test in under 5 minutes (e.g. the task 68 `all_mvp_endpoints_return_non_404` sweep which exercises 14 routes) will trip the bucket and get 429 Too Many Requests starting on the 7th call.

**The fix** — after constructing the RateLimit, before building the actix App, bump every bucket:

```rust
use enum_map::enum_map;
use lemmy_utils::rate_limit::{ActionType, BucketConfig};

let rate_limit = RateLimit::with_debug_config();
rate_limit.set_config(enum_map! {
  ActionType::Message => BucketConfig { max_requests: 10_000, interval: 60 },
  ActionType::Post    => BucketConfig { max_requests: 10_000, interval: 60 },
  ActionType::Register => BucketConfig { max_requests: 10_000, interval: 60 },
  ActionType::Image   => BucketConfig { max_requests: 10_000, interval: 60 },
  ActionType::Comment => BucketConfig { max_requests: 10_000, interval: 60 },
  ActionType::Search  => BucketConfig { max_requests: 10_000, interval: 60 },
  ActionType::ImportUserSettings => BucketConfig { max_requests: 10_000, interval: 60 },
});
```

Requires `enum-map = { workspace = true }` in `[dev-dependencies]` of the test crate.

**How to apply:**
- Only needed for actix integration tests that exercise rate-limited routes at volume
- Direct-handler-call tests (bypass actix middleware) don't need this
- The scope that wraps `rate_limit.post()` includes ALL `/governance` routes (POST and GET), so the bucket applies to the whole route family, not just POSTs
- The rate-limit behaviour IS production code; don't assume 200s for permissive test mode — `with_debug_config` is *slightly* more permissive than prod, not "test mode with limits off"

Discovered 2026-04-19 when task 68's 14-endpoint non-404 sweep returned 429 on the 7th call.
