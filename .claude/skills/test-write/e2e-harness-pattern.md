# e2e harness pattern (canonical)

The fixture shape every Brehon e2e test under `crates/server/tests/e2e.rs` must follow. Copy and adapt; do not reinvent.

This pattern is grounded in the existing test at `crates/server/tests/e2e.rs:1100-1200` (post-Phase-5 era) and the lessons in `feedback_async_pool_test_pattern.md`, `feedback_clippy_test_style.md`, `feedback_multi_write_handlers_need_transactions.md`.

## Required imports

```rust
use lemmy_api_utils::{context::LemmyContext, ...};   // handler-specific bits
use lemmy_db_schema::source::{instance::Instance, person::Person, ...};
use lemmy_db_schema_file::PersonId;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::{
  connection::{ActualDbPool, DbPool, build_db_pool_for_tests},
  traits::Crud,
};
use lemmy_utils::{error::LemmyResult, rate_limit::RateLimit, settings::SETTINGS};
use actix_web::web::Data;
use reqwest_middleware::ClientBuilder;
```

## Test signature

```rust
#[tokio::test]
async fn <handler>_<case_name>() -> LemmyResult<()> {
  // body
  Ok(())
}
```

**Invariants (do not deviate):**

- Return `LemmyResult<()>`, never `()` or `Result<(), Box<dyn Error>>`
- Use `?` for every `Result` — NEVER `unwrap()`, `expect()`, or `#[allow(...)]` (per `feedback_clippy_test_style.md`; project lint config denies these)
- `#[tokio::test]` because the harness is async-pool-based

## Setup block — five steps in order

### Step 1: Set env BEFORE any Lemmy code

Deterministic ed25519 signing seed; required so governance-log entries hash identically across test runs.

```rust
const SIGNING_SEED_HEX: &str =
  "0000000000000000000000000000000000000000000000000000000000000001";
// SAFETY: tests run with --test-threads=1 (no concurrent env mutation).
unsafe {
  std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
  std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
}
```

### Step 2: Spin up Postgres + apply schema

```rust
let (_container, host_port) = governance_fixtures::start_postgres()
  .await
  .map_err(|e| anyhow::anyhow!("start_postgres: {e}"))?;
let db_url = governance_fixtures::db_url(host_port);
unsafe { std::env::set_var("LEMMY_DATABASE_URL", &db_url); }

{
  let mut sync_conn = PgConnection::establish(&db_url)?;
  governance_fixtures::apply_all_schema(&mut sync_conn)
    .map_err(|e| anyhow::anyhow!("apply_all_schema: {e}"))?;
}
```

The `_container` binding is critical — drop = container teardown. Keep it alive for the test's lifetime.

### Step 3: Build the async pool

```rust
let pool: ActualDbPool = build_db_pool_for_tests();
```

`build_db_pool_for_tests` reads `LEMMY_DATABASE_URL` from SETTINGS and runs `schema_setup::run` (idempotent against the schema applied in step 2; acquires `pg_advisory_lock(0)` to bypass the `forbid_diesel_cli` trigger).

### Step 4: Build LemmyContext

```rust
let client = client_builder(&SETTINGS).build()?;
let middleware_client = ClientBuilder::new(client).build();
let secret = Secret { id: 0, jwt_secret: String::new().into() };
let rate_limit = RateLimit::with_debug_config();
let context = Data::new(LemmyContext::create(
  pool,
  middleware_client.clone(),
  middleware_client,
  secret,
  rate_limit,
));
```

If your handler requires `activitypub_federation::config::Data<LemmyContext>` (e.g., post-Phase-6 `submit_jury_vote`), additionally build a federation context — see `e2e.rs:1176-1183`.

### Step 5: Seed fixtures

```rust
let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
```

Build persons, communities, posts as the test requires. Reuse the helper closures at the top of existing tests rather than inlining.

## Connection patterns

### `&mut context.pool()` (preferred for short calls)

```rust
let admin_view = LocalUserView::read_person(&mut context.pool(), admin_id).await?;
```

### `let mut pool = context.pool();` (when reused several times)

```rust
let mut pool = context.pool();
let admin_view = LocalUserView::read_person(&mut pool, admin_id).await?;
let community = Community::create(&mut pool, &community_form).await?;
```

### `DbPool::Conn` borrow (for helpers that take an explicit conn)

```rust
let mut conn = context.pool().get().await?;
helper_fn(&mut conn, ...).await?;
```

Prefer `&mut context.pool()` unless the helper signature mandates `DbPool::Conn`. See `feedback_async_pool_test_pattern.md` for the rationale.

## Multi-write tests need transactions

If the test asserts on the post-state of TWO OR MORE writes that must succeed atomically (and the handler under test wraps them in a transaction itself), the test should NOT need its own transaction wrapper. But if the test issues N independent writes as fixture setup AND asserts cross-row invariants, wrap them:

```rust
context.pool().get().await?.run_transaction(|conn| async move {
  // multi-write fixture setup
  Ok::<_, LemmyError>(())
}.scope_boxed()).await?;
```

Per `feedback_multi_write_handlers_need_transactions.md`: any code path with 2+ writes whose post-condition matters needs transactional discipline.

## Calling the handler under test

```rust
let payload = HandlerForm {
  field: value,
  ..Default::default()  // only if the form has #[derive(Default)]
};
let response = handler::handle(payload, context.clone()).await?;
```

If the form does NOT derive Default, build it explicitly with all fields populated. Per R5.1: a form being `#[derive(Default)]` does NOT mean explicit-field literal sites get propagated automatically — be explicit.

## Asserting post-state

**Direct DB queries, not API round-trips.** API round-trips are slow and can mask bugs (e.g., a write that succeeded but a read filter excluded).

```rust
let case = ModerationCase::read(&mut context.pool(), case_id).await?;
assert_eq!(case.status, CaseStatus::JurySelection);
assert_eq!(case.severity_tier, SeverityTier::Major);
```

For governance-log entries, use the helper:

```rust
let entries = governance_log::find_by_kind_and_target_id(
  &mut context.pool(),
  ENTRY_KIND_SEVERITY_TIER_FROZEN,
  case_id,
).await?;
assert_eq!(entries.len(), 1);
```

## Pseudonymisation (ADR-015)

Any assertion involving `actor_pseudonym` MUST use the helper that resolves `Person → ActorPseudonym`. NEVER assert on raw `person_id` in governance-log payloads — that violates ADR-015 and will fail review.

```rust
let pseudonym = actor_pseudonym_helper::resolve(&mut context.pool(), admin_id).await?;
assert_eq!(entry.actor_pseudonym, Some(pseudonym));
```

## What NEVER appears in a Brehon e2e test

- `unwrap()`, `expect()`, `#[allow(...)]` — use `?` always (lint config denies)
- Mocked DB connections — tests use a real pool against a real Postgres container
- `assert!(handler_result.is_ok())` — use `?` to propagate; the test signature is `LemmyResult<()>`
- Hardcoded `Person.id = 1` assertions — pseudonyms (ADR-015)
- `tokio::time::sleep` for synchronisation — use the test pool's transactional ordering

## When to invoke `/cargo-validate` from this skill

After the test is written, before reporting done:

```
/cargo-validate test --no-run -p lemmy_server --test e2e
```

Expect exit 0. If the test compiles, it's structurally sound; running it (`/cargo-validate test --test e2e -p lemmy_server` or the `cargo-runner` background subagent for full sweeps) is a separate step.
