# M2-core-hook — dispatch handover (2026-06-05)

**Readable with zero conversation context.** Advisor session driving the in-binary M2-core slice.

## RESUME block

- **Current sub-phase:** `m2-core-hook` (M2-core in-binary slice). Plan: `.claude/PRPs/plans/m2-core-transition-hook.plan.md` (APPROVED by user 2026-06-05).
- **State-machine stage:** Task 1 DONE + MERGED (phase-m2-core-hook @ `0662ffc6e`, DQ `61d96bf8b2d6-001` resolved pass). **Task 2 BOTH ARMS RUNNING: #609 Sonnet (canonical, base phase-m2-core-hook) + #610 MiniMax (base ab-test/m2-t2-minimax, never merges).** Task 1 AB result: both arms produced byte-for-byte identical diffs; Sonnet 2m26s, MiniMax 2m20s.
- **Lane mode:** Mode B (drive from canonical `brehon-fork` / `governance-v0`; base_branch=`phase-m2-core-hook`; impl briefs author on trunk + SSH-merge into phase branch).
- **Last trunk commit:** `a308c891d` (Task-2 brief) on `governance-v0`; phase branch tip `0662ffc6e` (brief synced).
- **VERIFIED_AT:** `0662ffc6e` (phase-m2-core-hook @ origin; daemon-local synced; ab-test/m2-t2-minimax base created off same tip).

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

Poll #609 + #610 → on both `done`: read validate-pending-laptop DQ from #609, run `./scripts/brehon/cargo-check.sh -p lemmy_api_utils --features full` on laptop, mutate DQ (pass→resolved), merge #609 into phase-m2-core-hook, record AB delta vs #610, then author + dispatch Task 3 (governance_case_after_transition fn, same file, requires:2, MiniMax-qualifying).

**Task 3 base:** must be the phase-m2-core-hook tip AFTER Task 2 merges (Task 3 requires:2 — modifies same file `bridge_notify.rs`). Do NOT dispatch Task 3 until Task 2 Sonnet arm is merged.
