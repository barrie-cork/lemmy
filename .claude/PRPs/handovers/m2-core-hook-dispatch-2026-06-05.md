# M2-core-hook — dispatch handover (2026-06-05)

**Readable with zero conversation context.** Advisor session driving the in-binary M2-core slice.

## RESUME block

- **Current sub-phase:** `m2-core-hook` (M2-core in-binary slice). Plan: `.claude/PRPs/plans/m2-core-transition-hook.plan.md` (APPROVED by user 2026-06-05).
- **State-machine stage:** Tasks 1+2 DONE + MERGED. **Task 3 BOTH ARMS RUNNING: #611 Sonnet (canonical, base phase-m2-core-hook) + #612 MiniMax (base ab-test/m2-t3-minimax, never merges).** Task 2 AB: #609 Sonnet done (2m58s, pass); #610 MiniMax failed 4s (model 404). Task 1 AB: identical diffs, both pass.
- **Lane mode:** Mode B (drive from canonical `brehon-fork` / `governance-v0`; base_branch=`phase-m2-core-hook`; impl briefs author on trunk + SSH-merge into phase branch).
- **Last trunk commit:** `9d89e5ca4` (Task-3 brief) on `governance-v0`; phase branch tip `af6cf8c8c` (brief synced; DQ 262f6fd132fe-001 resolved pass @ f08b2d9a3).
- **VERIFIED_AT:** `af6cf8c8c` (phase-m2-core-hook @ origin; ab-test/m2-t3-minimax base created off same tip).

## Plan task → dispatch map

Plan has Tasks 0–9. Pre-Shape-G: every impl-task writes a `validate-pending-laptop` DQ with the cargo command verbatim and STOPS — laptop advisor runs cargo (NO CARGO ON ELITEDESK hard rule).

| Plan Task | Class | MiniMax-qualifying? | Files | Notes |
|---|---|---|---|---|
| 0 | chore (rg re-enumerate) | n/a | none (read-only) | Gate: re-verify 11 transition sites vs HEAD before any wiring. No cargo. |
| 1 | impl | ✅ | `api_common/governance.rs` | DTO `BridgeNotifyPayload` tagged enum. MIRROR site/api.rs:762-770. |
| 2 | impl | ✅ | `bridge_notify.rs` | refactor PM path onto union. requires:1 |
| 3 | impl | ✅ | `bridge_notify.rs` | add `governance_case_after_transition`. requires:2 (same file as 2 → SERIAL) |
| 4 | impl | ❌ (3 files) | db_schema log + api shim + registry doc | 10 ENTRY_KIND_ROOM_* consts. |
| 5 | impl | ✅ | `api/governance/governance_log.rs` | `append_room_event` wrapper. requires:4 |
| 6 | impl | ❌ (5 files) | 5 handler files | wire hook at 8 handler sites. requires:3 |
| 7 | impl | ✅ | `appeal_window_expiry.rs`+`sponsor_liability_grace.rs` | wire hook at 3 cron sites. requires:3 |
| 8 | impl | ❌ (e2e) | `e2e/governance.rs` | 4 e2e tests. requires:5,6,7 |
| 9 | chore | n/a | DQ raise | clippy gate + doc-drift follow-up DQ. |

**Dependency reality:** 1→2→3 chain (2,3 both edit bridge_notify.rs → strictly serial). 4→5. 3→{6,7}. {5,6,7}→8.

## MiniMax trial (FIRES this phase, n=8 ≥ 5)

- User decision 2026-06-05: **Run trial, accept key re-exposure** (key un-rotated, echoed by list_tasks envOverrides).
- Qualifying tasks: **1, 2, 3, 5, 7**.
- Per runbook §2.3: Sonnet (control) arm = canonical, merges to `phase-m2-core-hook`. MiniMax (M2.7) arm = comparison-only, on throwaway `ab-test/m2-t<N>-minimax` branch off the same base, NEVER merges.
- MiniMax dispatch: `env_overrides` with MiniMax endpoint + `ANTHROPIC_MODEL=MiniMax-M2.7` (see `scripts/brehon/queue-minimax-task.sh` for the canonical env shape).
- Results → append a `## m2-core-hook` section to `.claude/PRPs/reports/minimax-m27-trial-results.md`.
- **Concurrency cap: 2 total running tasks** (shared `.git/index.lock`). A Sonnet+MiniMax pair = 2 → at cap. Do NOT add a 3rd concurrent.

## Cross-session deps

- DQ pending: none at dispatch time.
- Telegram completion hook (ID 3): **active but BuildMessage:ModuleNotFound** — won't fire ✅/❌. Non-blocking (notification carrier only); rely on polling loop. Homeserver-ops fix deferred.
- Docker Desktop: **was stopped** at plan-approval; must be running before Task 8 (e2e) + final regression.
- deps-r4: 13 aarch64-only wasmtime alerts re-opened by extism revert; not a blocker.

## Next concrete action

Poll #611 + #612 → on both done: read validate-pending-laptop DQ from #611 (phase_task=3), run `./scripts/brehon/cargo-check.sh -p lemmy_api_utils --features full` on laptop (apply bridge_notify.rs + governance.rs from phase tip to working tree), mutate DQ pass→resolved on phase branch, push daemon merge of #611 to origin, record AB delta vs #612, then author + dispatch Tasks 4 and 6+7 per dependency order.

**⚠️ POINT-IN-TIME — re-verify before acting:** #611/#612 may be done by next session. Check `show_task 611` + `show_task 612` first.

**Task 4 base:** governance-v0 (3 files, NOT MiniMax). **Tasks 6+7 base:** phase-m2-core-hook AFTER Task 3 merges (requires:3).
