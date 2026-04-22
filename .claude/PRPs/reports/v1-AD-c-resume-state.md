# v1-AD-c implementation — resume state

**Saved**: 2026-04-21 (session close)
**Branch**: `phase-v1-AD-c`
**HEAD**: `a15bf3d04` (task 5 commit)
**Base**: `f03ed1cba`
**Plan**: `.claude/PRPs/plans/v1-admin-dashboard-c.plan.md`

Read this file first in the next session, then the runlog at
`.claude/PRPs/v1-AD-c-runlog/01-v1-AD-c-progress.md` for full history
and `02-task0-narrative.md` for archived long-form rationale.

## Tasks committed this session

| Task | Commit | Subject |
|---|---|---|
| tX | `0dd3840e1` | docs(plan): fix task 2/5b/7 VALIDATE to use -p lemmy_api for --features full propagation |
| t2 | `746ebc194` | feat(api-common): rule-set DTOs + AdminConfigAuditEntry.previous_from (task 2) |
| t3 | `4706715d6` | feat(admin-rule-sets): admin_create_rule_set + admin_list_rule_sets + ENTRY_KIND_RULE_SET_VERSION_CREATED (task 3) |
| t4 | `d623bcff5` | refactor(admin-config): thread pre-tx provenance into audit payload (task 4, closes #77) |
| t5 | `a15bf3d04` | feat(case-open): pin applied_config_snapshot + rule_set_version_id (task 5) |

Pre-session completed: t0 (plan + audit), t1 (#78 fix, `c8c29c973`).

## Phase progress

- [x] t0 — plan + pre-phase audit
- [x] t1 — Scope::parse_wire non-positive fix (closes #78)
- [x] t2 — rule-set DTOs + previous_from DTO field
- [x] t3 — admin_create_rule_set + admin_list_rule_sets + const
- [x] t4 — admin_config payload-shape evolution (closes #77)
- [x] t5 — case-open snapshot pin
- [ ] **t6 — NEXT**: auto protocol (no pause-per-commit)
- [ ] t7 — auto protocol (no pause-per-commit)
- [ ] t8 — pause protocol (e2e tests)

## Protocol for next session

Per runlog header: `pause(1,2,3,4,5,8) auto(0,6,7)`.

**Tasks 6 and 7 are auto** — execute back-to-back without waiting for
advisor review-go between commits. Task 8 returns to pause-per-commit
for the 8 e2e tests.

## Out-of-scope local-tree state (NOT touched this session)

These were pre-existing at session start and intentionally not staged:

- `M .claude/hooks/README.md`
- `M .claude/settings.json`
- `?? .claude/channels/`
- `?? .claude/hooks/check-cargo-pipe.sh`
- `?? .claude/hooks/inject-dq-state.sh`
- `?? .claude/hooks/pre-phase-audit.sh`
- `?? .claude/hooks/test-hooks.sh`
- `?? .claude/routines/`
- `?? .claude/audit-clippy-distribution.txt`
- `?? .claude/PRPs/reports/v1-AD-c-resume-state.md` (this file)
- `?? .claude/PRPs/v1-AD-c-runlog/` (runlog + narrative archive)

Next session: check with user whether any of these should be staged,
or leave untouched until phase-close.

## Retros logged this session (from advisor)

- **retro7** (t3): `CreateRuleSetTxArgs` struct + pattern-destructure is
  the preferred fix for clippy `too_many_arguments` — reuse in t6/t7 if
  the threshold hits again. Do NOT use `#[allow(clippy::too_many_arguments)]`.
- **retro8** (t4): pre-existing `lemmy_api_crud` reqwest_middleware
  compile break in the OAuth path blocks runtime lib-test execution
  under `-p lemmy_api --features full`. Accept compile-time validation
  (`cargo check -p lemmy_api --features full`) as sufficient per v1-AD-b
  task 9 precedent. Tracking issue deferred to phase-close.
- **task-notification exit summaries lie**: confirmed a second time at
  t5 — the background `cargo-test.bat` notification reported exit 0 but
  the log tail showed the same api_crud compile failure (exit 101).
  Always verify via log tail per `feedback_task_notification_exit_summary_unreliable.md`.

## Heads-up for task 6

1. Read plan §13 task 6 in full BEFORE editing.
2. Task 6 is `auto` protocol — commit without pausing for review-go.
3. If clippy `too_many_arguments` fires, apply retro7 pattern.
4. If runtime lib-test fails with `lemmy_api_crud` reqwest error, apply
   retro8 (accept compile-time validation, don't fight the pre-existing
   break).

## Validation-log pointers (all green at session close)

- `.claude/build-task2-api.log` — t2 code compile
- `.claude/build-task3a.log`, `.claude/build-task3b.log` — t3 db_schema + api compile
- `.claude/build-task3-clippy.log` — t3 workspace clippy
- `.claude/build-task4.log` — t4 api compile
- `.claude/test-task4-e2e.log` — t4 e2e test-compile (exit 0, 11m 12s)
- `.claude/build-task5b.log` — t5 api compile
- `.claude/build-task5-clippy.log` — t5 workspace clippy

## Key files written this session

- `crates/api/api/src/governance/admin_rule_sets.rs` (new, 370L — t3)
- `crates/api/api/src/governance/case_open_snapshot.rs` (new, 98L — t5)
- `crates/api/api/src/governance/admin_config.rs` (modified — t4)
- `crates/api/api_crud/src/governance/create_report.rs` (modified — t5)
- `crates/db_schema/src/source/governance/governance_log.rs` (+1 const — t3)
- `crates/api/api/src/governance/governance_log.rs` (+1 re-export — t3)
- `crates/api/api/src/governance/mod.rs` (+2 modules — t3 + t5)
- `crates/api/api/Cargo.toml` (+sha2, hex — t3)
- `.claude/rules/governance-log-entry-kind-registry.md` (v1-AD-c section — t3)
- `.claude/PRPs/plans/v1-admin-dashboard-c.plan.md` (3 VALIDATE lines — tX)

## End of state
