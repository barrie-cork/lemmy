---
phase: v1-SL-e
role: impl-task
task: 1
brief_n: 1
authored: 2026-05-12
---

# [role:impl-task] v1-SL-e task 1 — e2e Test #1 revocation-during-window-escapes + open mod v1_sl_e_fixtures — see .claude/PRPs/briefs/sl-e-impl-1.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-e task 1 — e2e test #1 revocation-during-window-escapes + open mod v1_sl_e_fixtures`

## §2 Scope

Append `mod v1_sl_e_fixtures` to `crates/server/tests/e2e.rs` AFTER `mod v1_sl_d_fixtures` (closes near line 13,693). Inside the new mod: use block + shared helpers + `revocation_during_window_escapes_full_lane` test fn.

**Single file, one anchor-Edit at file end.** Per `feedback_junior_worker_e2e_edit_hang.md`: e2e.rs is **13,693 lines** post-SL-d — anchor-Edit only, never full-file Read/Edit.

Full IMPLEMENT spec in plan §13 Task 1 (lines 1664-1860+).

## §3 Required reading

**Mandatory file-class lessons (advisor-orchestrator.md §2.4):**

1. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Case A canonical sibling** at `crates/server/tests/e2e.rs:11001-11924` (`mod v1_sl_b_fixtures`). Mirror `LemmyResult<T>` outer pattern verbatim. Test fn → `-> LemmyResult<()>`. Helpers → `-> LemmyResult<T>`. NO `Box<dyn Error>`. NO `.map_err` bridges. Read the sibling mod in full at task-start.
2. `.claude/lessons/feedback_async_pool_test_pattern.md` — e2e fixture context pattern.
3. `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **MANDATORY anchor-based Edit, never full-file Read.** One Edit call only. Use the closing `}` of `mod v1_sl_d_fixtures` as anchor `old_string` (with sufficient context to be unique).

**Plan sections:**
4. `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md` §13 Task 1 — full IMPLEMENT spec (content shape, shared helpers, Test #1 assertions, all GOTCHAs)
5. `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md` §10 — MIRROR refs (§10.1 fixture mod shape; §10.2 shared helpers; §10.3 revocation pattern; §10.4 scheduler tick; §10.7 governance_log assertions)
6. `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md` §4.2 — all 8 watchpoints (esp. #3 env var, #4 real HTTP-call, #5 pseudonym discipline)
7. `.claude/rules/decision-queue.md` — Recipe 1 if blocker found

**Shape G lesson:**
8. `.claude/lessons/feedback_insertform_default_propagation.md` — not directly applicable but read for context on None-padding pattern if ReputationEventInsertForm callsites arise

## §3a Handover from prior task

- **Task 0 (pre-flight, job-235):** All 19 probes passed. `mod v1_sl_d_fixtures` closes at e2e.rs line 13,693. Advisory: registry `(pending)` markers for SL consts are stale doc drift (not blockers). Advisory: `cargo-test.sh` missing execute bit (Shape G non-binding). No blocker DQ raised.
- **phase-v1-SL-e tip:** `0b7616deb` (bm-cut off governance-v0 @ b6bc3eaf5)
- **e2e.rs last mod line:** Task 0 Probe 12 confirmed: last `mod v1_*` is `mod v1_sl_d_fixtures` closing at line ~13,693.

## §4 Constraints

- **Touch only:** `crates/server/tests/e2e.rs` + `.claude/decision-queue.json` (if blocker raised).
- **One anchor-Edit** at file end. Do NOT Read the full 13,693-line file. Do NOT use `replace_all`.
- **Case A canonical**: `LemmyResult<()>` test fn outer, `LemmyResult<T>` helper outers. Per `mod v1_sl_b_fixtures` at e2e.rs:11001-11924. Never `Box<dyn Error>`.
- **Env var safety**: `unsafe { std::env::set_var(...) }` required (Rust 1.85+). Restore via `prev_disable` pattern per SL-c-2 at e2e.rs:12046-12049 + 12141-12146.
- **Commit message:** `test(v1-SL-e): e2e test #1 revocation-during-window-escapes + open mod v1_sl_e_fixtures + shared helpers (task 1)`
- **Shape G:** after committing, push worker branch; write `kind: "validate-pending"` DQ entry with `workflow_run_id` from `gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --json databaseId`. Commit + push DQ entry.
- **DQ atomic raise:** `git add .claude/decision-queue.json && git commit && git push origin <branch>` THEN end task. Per `feedback_dq_raise_before_ci_watcher_queue.md`.
- **Encoding:** Python `json.dump(..., ensure_ascii=False, indent=2)` for DQ writes.
- **next_id:** compute across `.claude/decision-queue.json` AND `.claude/decision-queue-archive-*.json`.
- **Attribution:** `from: "impl"`, never `from: "advisor"`.
- **No edits** to `crates/api/**`, `migrations/**`, `.github/workflows/**`, or any file outside `crates/server/tests/e2e.rs`.
