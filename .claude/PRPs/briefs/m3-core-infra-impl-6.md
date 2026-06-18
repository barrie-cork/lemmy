# Brief: m3-core-infra impl-6 (Task 6)

## §1 Role + dispatch line

`[role:impl-task] m3-core-infra-task6-rtc-clean-posture-e2e — see .claude/PRPs/briefs/m3-core-infra-impl-6.md`

## §2 Scope

**Task 6 of plan `.claude/PRPs/plans/m3-core-infra.plan.md`.** Add ONE e2e test to
`crates/server/tests/e2e/governance.rs` proving a governance flow runs unchanged when
`rtc_enabled` is absent/false, with zero RTC side-effects (§16a Story 1).

**Produces (exactly 1 file edit, one commit):**
- `crates/server/tests/e2e/governance.rs` — one small additive test function.

**Do NOT touch:** any file other than `crates/server/tests/e2e/governance.rs`. Do NOT edit
`crates/`, `services/bridge/`, `migrations/`, `Cargo.toml`. This is a single test append.

**Branch:** forks from `phase-m3-core-infra` (current tip `6b46b4507`).

## §3 Required reading

- `.claude/PRPs/plans/m3-core-infra.plan.md` §10.11 (Clean-posture e2e — the mirror target)
  and Task 6 (lines ~580-608).
- `crates/server/tests/e2e/governance.rs:~5354-5392` — the **canonical mirror**: the
  `m2_hook_suppressed_when_messaging_disabled` test (the `messaging_enabled`-absent → no-op
  governance transition). Mirror its error-shape, fixture bootstrap, and assertion structure
  VERBATIM. Read this sibling at the cited line range BEFORE authoring — it shows the exact
  `LemmyResult<()>` return shape, the `governance_fixtures::bootstrap()` call, and the
  `governance_case_after_transition` driver with `messaging_enabled` absent.
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — the `LemmyResult<()>` / `?`-operator
  error shape. Mirror the sibling's case (A or B) verbatim — do NOT invent a new error idiom.
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` +
  `DbPool::Conn` pattern for e2e DB access (the sibling uses it; copy its form).
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate the verbatim
  `old_string` anchor before editing this >5400-line file; confirm uniqueness with `grep -c`.

## §4 Constraints

### The test (per §10.11, mirror the `messaging_enabled`-absent sibling)

Mirror `m2_hook_suppressed_when_messaging_disabled` (governance.rs ~5354-5392). The new test
asserts the **`rtc_enabled` off-path is a TESTED signal, not an assumption** (plan GOTCHA R7):

1. Bootstrap fixtures: `let (_container, context, db_url) = governance_fixtures::bootstrap().await?;`
2. Drive a governance case transition with NO `rtc_enabled` row present (so `read_current`
   returns `None` → `false`), mirroring the sibling's `governance_case_after_transition` call.
3. Assert the transition COMPLETES (`LemmyResult` Ok — the governance path is unaffected by
   the absent RTC config).
4. Assert `get_bridge_messaging_status` reports `rtc_enabled = false` (the default-off read).
5. No LiveKit calls are made — the binary has no LiveKit client; assert by construction that
   the governance path compiles + passes with the RTC stack absent.

Name the test descriptively, e.g. `m3_rtc_disabled_clean_posture_governance_unaffected`.
Return type: `lemmy_utils::error::LemmyResult<()>` (mirror the sibling exactly).

### Import discipline (the E0432 trap that bit Task 4)

`Crud` is NOT exported from `lemmy_db_schema`. If you need `Person::create` or any CRUD trait
method, import it as a **standalone** `use lemmy_diesel_utils::traits::Crud;` line — NOT nested
inside a `lemmy_db_schema::{...}` block. The Task 4 e2e test failed E0432 by nesting it wrong;
do not repeat. (The mirror sibling may not need `Crud` at all — check what it imports first.)

### Anchor uniqueness gate (mandatory pre-edit)

Before the Edit, pre-locate the verbatim `old_string` anchor for the insertion point (the END
of the mirror sibling test, or the end of the file's governance test module). Run:
1. `grep -c '<your-chosen-anchor>' crates/server/tests/e2e/governance.rs` → must be `1`.
2. `grep -c 'm3_rtc_disabled_clean_posture' crates/server/tests/e2e/governance.rs` → must be `0`
   before your edit (no name collision).

If the anchor is not unique, choose a longer/more-specific anchor before editing.

### Validate

Write a `validate-pending-laptop-e2e` DQ entry with:
```json
{
  "kind": "validate-pending-laptop-e2e",
  "commands": [
    "cmd //c \"scripts\\\\brehon\\\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/m3-core-infra-task6-e2e.log 2>&1\""
  ],
  "branch": "phase-m3-core-infra",
  "phase_task": "6",
  "e2e_filter": "test(m3_rtc_disabled_clean_posture)"
}
```
(Use your actual test-fn-name substring in `e2e_filter`.) Commit + push, then **stop**. Do NOT
run cargo yourself — the laptop advisor runs the scoped e2e. End the commit body with a
`LESSON:` trailer if you found anything durable.
