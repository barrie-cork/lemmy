# Rate-limit debug config (set_config recipe)

When an e2e test issues N requests against a rate-limited endpoint, the default `RateLimit::with_debug_config()` may still rate-limit if the bucket isn't bumped. This file is the canonical recipe for raising every bucket high enough that test bodies don't trip rate limits.

Source-of-truth: `feedback_rate_limit_debug_config_post_bucket.md` + working callsites at `crates/server/tests/e2e.rs:2730-2770` (post-Phase 5 era).

## When to use this

- The test under construction issues 2+ requests against an endpoint that has a rate-limit bucket configured
- The handler under test calls `Local{User,Site,...}::increment_rate_limit_bucket` or the equivalent
- A test was previously passing but started failing with `RateLimitError` after a refactor (a bucket got narrower)

## When you do NOT need this

- Single-request tests
- Tests that read state without writing
- Tests that don't go through the actix middleware stack (calling handler functions directly)

## The recipe

```rust
use lemmy_utils::rate_limit::{ActionType, BucketConfig};

let rate_limit = RateLimit::with_debug_config();

// Bump every bucket so test bodies don't trip limits.
rate_limit.set_config(enum_map! {
  ActionType::Message => BucketConfig { capacity: 1_000, refill_rate: 1_000 },
  ActionType::Register => BucketConfig { capacity: 1_000, refill_rate: 1_000 },
  ActionType::Post => BucketConfig { capacity: 1_000, refill_rate: 1_000 },
  ActionType::Image => BucketConfig { capacity: 1_000, refill_rate: 1_000 },
  ActionType::Comment => BucketConfig { capacity: 1_000, refill_rate: 1_000 },
  ActionType::Search => BucketConfig { capacity: 1_000, refill_rate: 1_000 },
  ActionType::ImportUserSettings => BucketConfig { capacity: 1_000, refill_rate: 1_000 },
});

let context = Data::new(LemmyContext::create(
  pool,
  middleware_client.clone(),
  middleware_client,
  secret,
  rate_limit.clone(),  // clone so subsequent .configure() can use rate_limit too
));
```

## Why every bucket?

The compiler enforces `enum_map!` exhaustiveness over `ActionType`. Missing one variant fails to compile. Bumping all uniformly is also defensible: tests should not be implicitly aware of which buckets the handler triggers.

If a future Lemmy release adds a new `ActionType` variant, the test fails to compile until the new variant is added to the `set_config` map — that's the right failure mode (forces explicit decision rather than silent under-config).

## Common mistake — bumping `Post` only

Per `feedback_rate_limit_debug_config_post_bucket.md`: setting only `ActionType::Post` (because "the test creates posts") leaves `ActionType::Comment` at its default. The test then trips when reaching a comment-creation step that wasn't obviously rate-limited from the test's surface. Always bump all buckets.

## What NEVER goes in a test

- `RateLimit::with_debug_config()` followed by no `set_config` call when the test issues 2+ writes
- `tokio::time::sleep` to "wait for the bucket to refill" — flaky; bump capacity instead
- Per-request rate-limit bypass via `#[cfg(test)]` shims — the project standard is `set_config`, not conditional bypass
