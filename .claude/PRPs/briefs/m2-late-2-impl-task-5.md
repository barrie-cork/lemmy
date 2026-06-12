---
phase: m2-late-2
role: impl-task
n: 5
authored: 2026-06-12
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m2-late-2
task_number: 5
parallel_group: serial (requires task 4)
---

# [role:impl-task] m2-late-2 Task 5 — Bridge handler tests

## 1. Role + dispatch line

```
[role:impl-task] m2-late-2 task-5 bridge handler tests — see .claude/PRPs/briefs/m2-late-2-impl-task-5.md
```

## 2. Scope

Add `#[cfg(test)] mod tests` to `sanction_handler.rs` covering dep-free paths (auth
fail, no-rooms early-return, `compute_power_override` unit cases) plus one
`#[ignore]` integration test for the live GET+PUT path.

### IMPLEMENT

| File | Change |
|---|---|
| `services/bridge/src/sanction_handler.rs` | Add `#[cfg(test)] mod tests` block with 4 test cases per §3 below. |

### Explicit out-of-scope

- Do NOT add any new dev-dependency (`wiremock`, `mockito`, `httpmock`, `tempfile`
  if not already present) — would change `services/bridge/Cargo.lock` → R-bridge-lock.
- Do NOT touch `bridge_room.rs`, `room_provisioner.rs`, or any other file.
- Do NOT change `handle_sanction_event` or helper bodies (Task 4 shipped those).
- Do NOT add `services/bridge` to `workspace.members` or use `--workspace`.
- Do NOT write a `validate-pending-laptop-linux` DQ — the test commands use
  `cargo test` (not `cargo check`) so only the basic laptop DQ is needed.

## 3. Test cases (verbatim from plan §13 Task 5)

**Read `services/bridge/src/sanction_handler.rs` (WHOLE FILE) and
`services/bridge/src/puppet.rs` (focus on `PuppetMap` constructor) BEFORE writing
any test code** — the test setup must match the actual types.

### Test 1 — bad bearer → 401, no Matrix call

Build a minimal `AppState` (temp `bridge_db_path`, `reqwest::Client::new()`, a
`PuppetMap` — check its constructor in `puppet.rs`). Call `handle_sanction_event`
with a wrong/missing `Authorization` header. Assert status 401.

Auth check returns before any lookup or Matrix call, so no network is contacted.

### Test 2 — no rooms for case → 200 `applied:false`, no Matrix call

Open a temp bridge DB with the schema but no rows for the test `case_id`; valid
Bearer token. Call `handle_sanction_event`. Assert status 200 and
`applied == false` and `reason` mentions "no rooms".

Per Pattern 10.4 ordering, `ensure_puppet` is never reached in this path —
this is the dep-free path load-bearing for the test.

**GOTCHA on temp DB:** if `PuppetMap` cannot be constructed without a live Matrix
client, restructure tests 1–2 so they exercise the auth-fail / no-rooms paths
without constructing `PuppetMap` (auth-fail returns before lookup; no-rooms
returns before `ensure_puppet`). Use `bridge_room::open` with a temp path; call
`bridge_room::init_schema` (or equivalent) to create the schema without rows.

For temp DB path: use `std::env::temp_dir().join(format!("bridge-test-{}.db",
std::process::id()))` and delete it after the test. Do NOT use `tempfile` crate
unless it is already a dev-dep in `services/bridge/Cargo.toml`.

### Test 3 — `compute_power_override` unit cases

Pure-function asserts. No I/O. Sample content:
```json
{"events_default": 0, "events": {"m.room.message": 0}}
```

Assert for each `sanction_kind`:
- `"ban"` / `"mute"` / `"prevent_post"` → level `-1`, reason `"power_level_reduced_below_post_threshold"`
- `"mute_voice"` with no voice threshold in content → level `events_default - 1 = -1`, reason `"voice_power_reduced_fallback_post_threshold"`
- `"hide_content"` → level `-1`, reason `"redaction_not_available_in_m2_late_2"`
- `"restrict_reach"` → level `-1`, reason `"restrict_reach_translated_to_power_level_reduction"`
- `"unknown_kind"` → level `-1`, reason `"unknown_sanction_kind_default_post_reduction"`

### Test 4 — live integration test (skipped in CI)

```rust
#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn test_power_levels_applied_live() {
    // One room provisioned for a case → POST sanction event →
    // assert GET+PUT fired and applied:true.
    // Mirror: services/bridge/tests/room_provisioning.rs convention.
}
```

**Mirror:** `services/bridge/tests/room_provisioning.rs:12-20` — `#[tokio::test]` +
`#[ignore]` convention in the bridge test suite.

## 4. Validation gate (validate-pending-laptop DQ — write then STOP)

After committing the change, write a `kind: "validate-pending-laptop"` DQ entry:

```json
{
  "commands": [
    "bash -c 'cd services/bridge && cargo test'",
    "bash -c 'cd services/bridge && cargo clippy --no-deps -- -D warnings'"
  ],
  "branch": "<current-worktree-branch>",
  "phase_task": 5,
  "e2e_filter": null
}
```

**IMPORTANT — advisor translates to Docker Linux:**
```
scripts/brehon/cargo-linux.sh test  --manifest-path services/bridge/Cargo.toml
scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings
```

Expected: tests 1–3 pass, test 4 skipped (`#[ignore]`). Clippy: same 18
pre-existing baseline errors (all pre-T5) — no new errors from the test module.

Generate the DQ id via `bash scripts/brehon/dq-v3-new-entry.sh`.
Append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`.

**R-bridge-lock check BEFORE committing:** verify `git diff services/bridge/Cargo.toml`
and `git diff services/bridge/Cargo.lock` are empty. If either changed, STOP and
file a `kind: "blocker"` DQ.

Commit the DQ entry + push the branch, then **STOP**.

## 5. Required reading

- `services/bridge/src/sanction_handler.rs` (WHOLE FILE) — current T4 implementation;
  all types used in the test must match what's actually there
- `services/bridge/src/puppet.rs` — `PuppetMap` struct + constructor (check if it
  requires a live client or can be constructed from static config)
- `services/bridge/src/bridge_room.rs` — `open()` and `init_schema` / schema-init
  function (to create temp DB for test 2)
- `services/bridge/src/appservice.rs:33-41` — `AppState` fields
- `services/bridge/src/config.rs` — `BridgeConfig` fields needed for `AppState`
- `services/bridge/tests/room_provisioning.rs:12-20` — `#[tokio::test]` + `#[ignore]`
  convention MIRROR for test 4
- `.claude/lessons/feedback_linux_compile_proof_is_a_gate.md` — bridge Cargo.lock
  change triggers the Linux gate; the recommended approach adds NO new dev-dependency
- `.claude/PRPs/plans/m2-late-2.plan.md` — §13 Task 5 spec, §7 R9/R-bridge-lock

## 6. Constraints

- **R9 (CRITICAL):** `services/bridge` stays in root `Cargo.toml` `exclude`. NEVER
  `--workspace` for bridge cargo commands.
- **NO new bridge dev-dependency** — the tests must use only deps already present in
  `services/bridge/Cargo.toml` (reqwest, serde_json, tokio, rusqlite, axum). If
  `tempfile` is NOT already a dev-dep, use `std::env::temp_dir()` instead.
- **NO cargo on EliteDesk (NO-CARGO-ON-ELITEDESK)** — write the
  validate-pending-laptop DQ and STOP.
- **`compute_power_override` is private** (`fn`, not `pub fn`) — tests must be in
  the same module (`mod tests` inside `sanction_handler.rs`) to access it via `super::`.
- **Per Pattern 10.4 ordering:** test 2 specifically exercises the no-rooms early
  return that fires BEFORE `ensure_puppet` — preserve this ordering in the test setup.
- **`#[ignore]` test 4 must NOT be removed** — it documents the live path even
  though it's not CI-runnable.
- **DQ v3 id** — generate via `bash scripts/brehon/dq-v3-new-entry.sh`.
- **Mid-task push discipline** — commit DQ entry + push immediately after writing it.
- **Commit prefix** — `feat(bridge): ` subject, with `LESSON:` trailer if any
  non-obvious finding arises.

### Mandatory lesson fires (file-class table check)

| File class | Match? | Lesson injected |
|---|---|---|
| `services/bridge/` edits | ✅ | R9 constraint (§6 above); `feedback_linux_compile_proof_is_a_gate.md` (§5) |
| Any handler doing 2+ DB writes | ❌ tests are read-only | — |
| `crates/server/tests/e2e.rs` edits | ❌ no e2e edits | — |
