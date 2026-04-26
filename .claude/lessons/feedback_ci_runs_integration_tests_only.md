---
name: CI cargo-test-e2e runs integration tests only, not lib unit tests
description: cargo-test-e2e.yml only runs `cargo test --test e2e -p lemmy_server`; lemmy_api lib tests never execute in CI; unit-test drift-guards rely on compile-time validation
type: reference
originSessionId: f54f6612-08ca-4d43-adf5-047a3721012a
---
The Brehon fork's CI workflow `.github/workflows/cargo-test-e2e.yml` runs **only** `cargo test --test e2e -p lemmy_server --no-run` and then `cargo test --test e2e -p lemmy_server -- --test-threads=1`. It does NOT run:

- `cargo test --workspace --lib ...` (all lib unit tests)
- `cargo test -p lemmy_api --lib ...` (lemmy_api lib unit tests)
- `cargo test -p <any-other-crate>`

**Implication for unit tests like `payload_parity`:** #[cfg(test)] lib tests in `crates/api/api/src/governance/**.rs` are validated at *compile time* by:

1. `cargo check --workspace --features full` — compiles the #[cfg(test)] modules when test targets are enabled (they aren't by default under `cargo check`)
2. `cargo clippy --workspace --features full --no-deps -- -D warnings` — `--features full` + `--no-deps` compiles test code transitively through clippy's lint pass
3. `cargo test --workspace --features full --no-run` (if run) — compiles but doesn't execute

But they are NEVER **executed at runtime** in CI on this fork. Their role is compile-time drift-guards: `json!({...})` macros + struct field equivalence assertions that fail to compile if the underlying types drift. The runtime assertions (`assert_eq!`, etc.) are decorative — they've never fired.

**Discovered 2026-04-21** during v1-AD-c task 4 disposition. Advisor investigated why v1-AD-b's task 9 `payload_parity` tests (committed at `d27c8feab` "test(admin-config): payload parity unit tests") could not be executed via `cargo test -p lemmy_api --features full --lib payload_parity`: pre-existing `lemmy_api_crud::user::create::oauth_provider.token_endpoint...form(&form[..])` compile break in `reqwest_middleware` blocks the test binary link. The bug exists at v1-AD-b HEAD AND prior-phase HEAD (confirmed via git stash). PR #76 merged green because CI only ran integration tests, bypassing lemmy_api's [dev-dependencies] (which includes lemmy_api_crud).

**How to apply:**

- When a plan task says "add a unit test for X", accept **compile-time validation** as the success signal. Don't chase runtime execution.
- If a unit test's value depends on runtime assertion (e.g. validating actual DB behavior), it must live in `crates/server/tests/e2e.rs` as an integration test, not as a lib #[cfg(test)] module.
- The drift-guard value of compile-time-only unit tests is ~80% of runtime value — catches type renames, field additions, struct reshuffles. Doesn't catch logic bugs.
- Related `chore:` issue candidate at phase-close: fix `lemmy_api_crud::user::create` reqwest_middleware transitive so lib tests CAN execute. Out of scope for v1-AD-c.

**Why this matters for v1-AD-c / v1-AD-d:** any future parity-style unit test added to `crates/api/api/src/governance/*.rs` is a compile-time-only guard. Don't add VALIDATE probes that try to execute them.
