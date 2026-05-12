---
phase: v1-SL-e
role: impl-task
task: 2
brief_n: 2
authored: 2026-05-12
---

# [role:impl-task] v1-SL-e task 2 — e2e test #2 window-expiry-fires + anchor-insert inside mod v1_sl_e_fixtures — see .claude/PRPs/briefs/sl-e-impl-2.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-e task 2 — e2e test #2 window-expiry-fires + anchor-insert inside mod v1_sl_e_fixtures`

## §2 Scope

Anchor-insert `window_expiry_fires_full_lane` test fn inside `mod v1_sl_e_fixtures` in `crates/server/tests/e2e.rs`, BEFORE the closing `}` of the mod. The mod was opened by Task 1 and closed with `}` — Tasks 2-3 insert before that closing brace.

**Single file, one anchor-Edit.** Per `feedback_junior_worker_e2e_edit_hang.md`: e2e.rs is **13,693+ lines** — anchor-Edit only, never full-file Read/Edit.

Full IMPLEMENT spec in plan §13 Task 2 (lines 1874-2004).

## §3 Required reading

**Mandatory file-class lessons:**

1. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Case A canonical** throughout. `LemmyResult<()>` test fn, `LemmyResult<T>` helpers. NO `Box<dyn Error>`. NO `.map_err`. Mirror `mod v1_sl_b_fixtures` at `crates/server/tests/e2e.rs:11001-11924` and Task 1's new mod `mod v1_sl_e_fixtures`.
2. `.claude/lessons/feedback_async_pool_test_pattern.md` — e2e fixture context pattern.
3. `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **MANDATORY anchor-based Edit, never full-file Read.** One Edit call only. Use the closing `}` of `mod v1_sl_e_fixtures` (added by Task 1) as anchor `old_string` — find the unique closing `}` after `revocation_during_window_escapes_full_lane`'s `Ok(())`.

**Plan sections:**
4. `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md` §13 Task 2 — full IMPLEMENT spec
5. `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md` §10 — MIRROR refs (§10.1 fixture mod shape; §10.3 jury-vote drive; §10.4 scheduler tick; §10.5 force-rewind grace_expires_at; §10.7 governance_log assertions)
6. `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md` §4.2 — watchpoints (esp. #2 time-handling, #3 env var, #4 real HTTP-call N/A for Test #2, #5 pseudonym discipline)
7. `.claude/rules/decision-queue.md` — Recipe 1 if blocker found

## §3a Handover from prior task

- **Task 1 (job-236):** commit `5b1898051` on phase-v1-SL-e. Phase 1 workspace check DQ #193 → pass. Phase 2 e2e (local) DQ #194 raised, result pending at brief-write time.
- **phase-v1-SL-e tip:** `d70f1b525` (advisor raised DQ #194)
- **e2e.rs last mod structure:** `mod v1_sl_e_fixtures` opened and closed by Task 1. Contains:
  - `seed_target_with_sureties_and_endorsements` helper
  - `count_log_entries` / `read_log_payload` helpers
  - `drive_jury_to_quorum` helper (takes `admin_view` param; no conn/instance_id params)
  - `revocation_during_window_escapes_full_lane` test fn
  - Closing `}` of the mod
- **Key decisions from Task 1:**
  - `sql_query`, `update`, `SanctionInsertForm` NOT yet imported — Task 2 needs `update` (for grace_expires_at rewind) and `moderation_case` table DSL; add to the use block inside the mod or at the top of the new test fn.
  - `AsyncConnection` import was added by Task 1 (was absent from SL-d Task 3).
  - Module currently closed: Tasks 2-3 anchor-insert BEFORE the closing `}`.
- **Case A canonical** confirmed for mod v1_sl_e_fixtures.

## §4 Constraints

- **Touch only:** `crates/server/tests/e2e.rs` + `.claude/decision-queue.json` (if blocker raised).
- **One anchor-Edit** before the closing `}` of `mod v1_sl_e_fixtures`. Do NOT Read the full file. Do NOT use `replace_all`.
- **Case A canonical**: `LemmyResult<()>` test fn outer. No `Box<dyn Error>`, no `.map_err`.
- **Env var safety**: `unsafe { std::env::set_var(...) }` required (Rust 1.85+). Restore via `prev_disable` pattern per SL-c-2 at e2e.rs:12046-12049 + 12141-12146.
- **Imports**: `update` + `moderation_case::table` DSL needed for grace_expires_at rewind. Add inside the test fn's use block or at mod-level if not already present.
- **Anchor uniqueness**: Use enough context around the closing `}` of `mod v1_sl_e_fixtures` to make `old_string` unique — include the `Ok(())` + closing `}` of `revocation_during_window_escapes_full_lane` + the mod's closing `}`.
- **Commit message:** `test(v1-SL-e): e2e test #2 — window-expiry-fires (full lane) (task 2)`
- **Shape G:** after committing, push worker branch; write `kind: "validate-pending"` DQ entry with `workflow_run_id` from `gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --json databaseId`. Commit + push DQ entry.
- **DQ atomic raise:** `git add .claude/decision-queue.json && git commit && git push origin <branch>` THEN end task.
- **Encoding:** Python `json.dump(..., ensure_ascii=False, indent=2)` for DQ writes.
- **next_id:** compute across `.claude/decision-queue.json` AND `.claude/decision-queue-archive-*.json`. Current max on phase-v1-SL-e is 194 — next id is 195.
- **Attribution:** `from: "impl"`, never `from: "advisor"`.
