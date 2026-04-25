---
name: test-write
description: Author Brehon e2e integration tests with the canonical pool/conn/LemmyResult fixture.
user-invocable: true
---

# `/test-write` — author Brehon e2e integration tests

Write integration tests under `crates/server/tests/e2e.rs` (or task-specific test files when the plan creates them) following the patterns Brehon's e2e harness mandates. Skill body holds the procedure; helper files hold the canonical fixtures.

## When to invoke

- A plan task explicitly requires test cases (e.g. JM-b Tasks 7-8: `compute_status_tier` + 3-phase select coverage)
- A new handler endpoint needs e2e coverage before PR open
- A bug fix needs a regression test

Skip when: writing a unit test inside a `#[cfg(test)] mod tests` block — those follow standard Rust idiom and don't need this skill's harness scaffolding.

## Inputs

The user/plan provides:

- **Subject under test** — file path + handler/function name (e.g., `crates/api/api/src/governance/admin_assign_jury.rs::handler`)
- **Test cases** — list of behaviours to cover. Each case is one of:
  - Golden path (input → expected state)
  - Error case (input → expected error variant)
  - Idempotency (re-run guard)
  - Constraint violation / cascade (R1/R2/R3 relaxation, etc.)
- **Fixture requirements** — what the test setup needs (e.g., `N jurors with specific roles`, `case in JurySelection state`, `governance_log entry kind X already present`)

If any input is missing, STOP and ask. Do NOT guess what the test should assert.

## Procedure

1. **Read the helper templates.** Use Read on these (they're not auto-loaded — load on first invocation only):
   - `.claude/skills/test-write/e2e-harness-pattern.md` — canonical async pool + DbPool::Conn + LemmyResult shape
   - `.claude/skills/test-write/rate-limit-debug.md` — the set_config incantation for bypassing rate limits in test bodies

2. **Read the subject.** Open the handler file the test targets. Note:
   - Handler signature (Json input type, return type)
   - Auth requirements (`local_user_view` extracts, capability flags)
   - Side effects (DB writes, governance_log emissions, plugin hooks fired)

3. **Read 2-3 sibling tests.** `Grep` for existing tests against the same crate. Match their style:
   - Naming: `async fn <handler>_<case_short_name>()` typical
   - Assertion shape (`assert_eq!`, `.unwrap()` is forbidden — use `?` per `feedback_clippy_test_style.md`)
   - Cleanup: most tests don't clean up explicitly; they rely on transactional rollback via the test pool

4. **Author the test cases.** For each case:
   - Build fixtures via the helper inserts in `e2e-harness-pattern.md` (don't reinvent — reuse)
   - Construct the handler input (Json with the form type)
   - Call the handler
   - Assert post-state via direct DB queries (NOT via API round-trip — too slow, masks bugs)
   - Use `LemmyResult<()>` signature, never `unwrap`/`expect` (`feedback_clippy_test_style.md`)

5. **Verify the test compiles.** Pair-invoke `/cargo-validate test --no-run -p lemmy_server --test e2e`. If it doesn't compile, fix and re-validate. Do NOT report the test as done until `--no-run` exits 0.

6. **Do NOT run the tests.** That's `/cargo-validate test --test e2e -p lemmy_server` (or the cargo-runner background subagent for full sweeps). This skill writes; the validate skill runs.

## Test-pattern invariants (load-bearing — do not deviate)

- **Signature:** `async fn <name>() -> LemmyResult<()>` always
- **Pool:** `let context = LemmyContext::init_for_tests().await?;` — never construct manually
- **DB conn:** `let mut conn = context.pool().get().await?;` (or `&mut DbPool::Conn` borrow when calling helpers per `feedback_async_pool_test_pattern.md`)
- **No `.unwrap()` / `.expect()`:** use `?` always (`feedback_clippy_test_style.md`)
- **Multi-write tests:** wrap in `conn.run_transaction()` per `feedback_multi_write_handlers_need_transactions.md` if the subject under test is multi-write
- **Rate-limit debug:** if the test issues N requests against a rate-limited endpoint, set `RATE_LIMIT_DEBUG_CONFIG_POST_BUCKET` per the helper (`feedback_rate_limit_debug_config_post_bucket.md`)

## Brehon-specific assertions

- **Governance log entries:** assert via `governance_log::find_by_kind_and_target_id` (or equivalent) — not via raw SQL string match. Hash-chain integrity check is implicit.
- **Pseudonymisation:** any test asserting on `actor_pseudonym` must use the helper that resolves `Person → ActorPseudonym` (ADR-015). Never assert raw `person_id` in governance-log payloads.
- **Plugin hooks:** if the handler under test sits on a PM-path or governance-path hook (per `pm-plugin-hooks-stable.md`), the test should NOT mock the hook — let the real dispatcher fire (it's a no-op when no plugin registered).

## Reporting format

```
test file: crates/server/tests/e2e.rs (modified)
cases written: N
  - <handler>_golden_path
  - <handler>_<error_case>
  - ...

compile check: /cargo-validate test --no-run -p lemmy_server --test e2e
exit: 0

next step: invoke /cargo-validate to run the tests, OR delegate full e2e sweep to cargo-runner subagent.
```

## What this skill never does

- Run the tests (use `/cargo-validate` or background subagent)
- Mock the database (per `feedback_multi_write_handlers_need_transactions.md` and project convention — tests use a real test pool)
- Use `unwrap`/`expect`/`#[allow(...)]` (per `feedback_clippy_test_style.md`)
- Assert on `Person.id` / `Person.name` in governance contexts (use pseudonyms — ADR-015)
- Edit handler code to make tests easier (out of scope; tests adapt to the contract, not vice versa)
- Skip the compile-check before reporting done

## Related rules (auto-loaded)

- `.claude/rules/pm-plugin-hooks-stable.md` — PM-path hook stability for tests touching PM crates
- ADR-015 — pseudonymisation invariants

## Helper files (load on demand)

- `e2e-harness-pattern.md` — async pool + DbPool::Conn + LemmyResult fixture template
- `rate-limit-debug.md` — set_config recipe for rate-limit bypass in tests

## Memory references

- `feedback_clippy_test_style.md` — LemmyResult<()> + ? operator
- `feedback_async_pool_test_pattern.md` — DbPool::Conn shape
- `feedback_multi_write_handlers_need_transactions.md` — transaction wrapping
- `feedback_rate_limit_debug_config_post_bucket.md` — rate-limit set_config
- `feedback_features_full_workspace_only.md` — `--features full` only via `--workspace`
