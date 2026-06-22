# Brief — m3-core-e2e-pilot impl Task 2 (defect 3): fast-fail emergency-mute revoke loop

## §1 Role + dispatch line

`[role:impl-task] defect3-fastfail — see .claude/PRPs/briefs/m3-core-e2e-pilot-impl-defect3-mute-fast-fail.md`

You are the **impl-task** subagent (Sonnet 4.6). One test-file edit. After it, write a `validate-pending-laptop-e2e` DQ, commit + push, **STOP**. Do NOT run cargo — the advisor runs the e2e against the full stack.

## §2 Scope

**Branch:** `phase-m3-core-e2e-pilot` (fork tip `d69d41c1a` or newer).

**One file.** Wrap each `update_participant` call in `tokio::time::timeout(200ms)` so a never-connected publisher (no psrpc handler → ~3s timeout) fails fast and mute-all stays under the 500ms perf gate. Plan: §10.6 + Task 2.

### Root cause (verified live by advisor 2026-06-21)

`mute_all_drops_all_publishers_cross_instance_under_500ms` FAILED `6007ms > 500ms`. Each never-connected publisher costs a ~3s psrpc `unavailable` timeout INSIDE the timed T0→T1 window. The #764 fix made the error CLASSIFICATION correct (timeout ⇒ zero-holder-by-absence) but did nothing about the 6s to reach it.

### IMPLEMENT (file 1 of 1)

In `services/bridge/tests/emergency_mute.rs`, in the revoke loop (~`:185-235`), replace the `let result = lk_client.update_participant(…).await;` call with the `tokio::time::timeout` wrap from plan §10.6:

```rust
    for &publisher in publishers {
        // Fast-fail: a never-connected publisher has no psrpc handler → ~3s timeout.
        // Cap each call so N×cap stays < 500ms (R-TIMEOUT-MATH: 2 × 200ms = 400ms).
        // Elapsed ⇒ zero-holder-by-absence (same semantics as the `unavailable` arm).
        let result = match tokio::time::timeout(
            Duration::from_millis(200),
            lk_client.update_participant(
                &lk_room_name,
                publisher,
                UpdateParticipantOptions { /* … unchanged … */ },
            ),
        ).await {
            Ok(inner) => inner,                       // inner: Result<ParticipantInfo, _> — existing arms handle it
            Err(_elapsed) => {
                // No response within budget = never-connected = zero-holder satisfied by absence.
                revoke_results.push((publisher, true));
                continue;
            }
        };
        match result { /* … existing Ok / Err arms UNCHANGED … */ }
    }
```

Keep the existing `match result { Ok(info) => …, Err(e) => … }` arms (incl. the `unavailable`/`not found` accept arm + the `assert!(is_not_found, …)`) intact — a connected publisher that responds fast still flows through them.

## §3 Required reading (phase-branch versions)

- `services/bridge/tests/emergency_mute.rs:12-20` (criterion 141 doc) + `:185-235` (the timed revoke loop, the existing `unavailable`/`not found` accept arm, the `t0`/`elapsed`/`< 500ms` assert). Read the EXACT current shape of the `let result = …update_participant…` line + its surrounding `match` before editing — pre-locate the verbatim anchor.
- `.claude/PRPs/plans/m3-core-e2e-pilot-e2e-fixes.plan.md` §10.6 + Task 2 + §18 (200ms-too-tight risk row, mitigated: base stack has no connected publishers).
- `.claude/lessons/feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo is Linux; you edit a test file, advisor runs it.
- `.claude/decision-queue.json` — read before the DQ write.

## §4 Procedure

1. Confirm branch = `phase-m3-core-e2e-pilot`.
2. Pre-locate the verbatim `let result = lk_client.update_participant(` anchor (`grep -n 'update_participant' services/bridge/tests/emergency_mute.rs` — confirm it's a SINGLE occurrence in the revoke loop; if >1, scope your Edit to the loop body).
3. Apply the §2 timeout wrap, keeping the inner `match result` arms unchanged.
4. Verify scope: `git diff --stat` = exactly 1 file.
5. Confirm `Duration` is imported (`:23`) — it is; `tokio::time::timeout` needs no new dependency (`tokio` is already a dev-dep). If `Duration` import is somehow absent, add `use std::time::Duration;`.
6. Commit: `fix(bridge/e2e): fast-fail update_participant 200ms timeout in mute-all (defect 3)`. Body: note R-TIMEOUT-MATH (2×200ms<500ms) + a `LESSON:` on the psrpc-timeout-inside-timed-window class.
7. Push.
8. Write the `validate-pending-laptop-e2e` DQ (via the helper scripts), commit + push. STOP.

DQ fragment shape:
```
kind: validate-pending-laptop-e2e
from: impl
commands: ["cargo test --manifest-path services/bridge/Cargo.toml --test emergency_mute -- --ignored"]
branch: phase-m3-core-e2e-pilot
phase_task: 2
result: null, log_slice: null, failed_commands: null
context: advisor runs against the FULL e2e stack (--profile rtc) per the handover re-run procedure.
```

## §5 Constraints

- **1 file only**: `services/bridge/tests/emergency_mute.rs`. NO src, NO Cargo, NO other test.
- **NO cargo run** — write-then-STOP.
- **R-TIMEOUT-MATH**: per-call timeout 200ms; `N_publishers × 200ms < 500ms` (N=2 → 400ms). Do NOT raise 200ms.
- Keep the existing Ok/Err `match result` arms intact — only wrap the call, add the `Err(_elapsed)` outer arm.
- DQ mid-task push mandatory.

## §6 DoD

- `git diff --stat HEAD~1` = 1 file.
- `grep -c 'tokio::time::timeout' services/bridge/tests/emergency_mute.rs` → 1.
- `grep -c 'Duration::from_millis(200)' services/bridge/tests/emergency_mute.rs` → 1.
- The existing `is_not_found` accept arm + `assert!` still present (`grep -c 'is_not_found' …` → unchanged ≥1).
- validate-pending-laptop-e2e DQ committed + pushed.

## §7 HANDOVER

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-impl-defect3-mute-fast-fail
  timeout_wrap: <"update_participant wrapped in 200ms timeout; Elapsed⇒revoked-by-absence" | "FAIL: <reason>">
  existing_arms_intact: <"Ok/Err match arms unchanged" | "FAIL: <reason>">
  anchor_occurrences: <"update_participant single in revoke loop" | "<n> — scoped to loop">
  files_changed: <list>
  dq_id: <id>
  notes: "<confirm 1 file; confirm 200ms not raised>"
```
