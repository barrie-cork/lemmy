---
phase: v1-SL-e
role: impl-task
task: 3
brief_n: 3
authored: 2026-05-12
---

# [role:impl-task] v1-SL-e task 3 — e2e test #3 backfill-of-mid-flight (v0→v1 deploy) + close mod v1_sl_e_fixtures — see .claude/PRPs/briefs/sl-e-impl-3.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-e task 3 — e2e test #3 backfill-of-mid-flight (v0→v1 deploy) + close mod v1_sl_e_fixtures`

## §2 Scope

Anchor-insert `backfill_of_mid_flight_v0_to_v1_deploy` test fn inside `mod v1_sl_e_fixtures` in `crates/server/tests/e2e.rs`, BEFORE the closing `}` of the mod. This is the **LAST** test in the mod; the closing `}` of the mod follows immediately after Test #3's closing `}`.

**Single file, one anchor-Edit.** Per `feedback_junior_worker_e2e_edit_hang.md`: e2e.rs is **13,693+ lines** — anchor-Edit only, never full-file Read/Edit.

Full IMPLEMENT spec in plan §13 Task 3 (lines 2006-2195).

## §3 Required reading

**Mandatory file-class lessons:**

1. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Case A canonical** throughout. `LemmyResult<()>` test fn, `LemmyResult<T>` helpers. NO `Box<dyn Error>`. NO `.map_err`. Mirror Task 1's `revocation_during_window_escapes_full_lane` and Task 2's `window_expiry_fires_full_lane`.
2. `.claude/lessons/feedback_async_pool_test_pattern.md` — e2e fixture context pattern.
3. `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **MANDATORY anchor-based Edit, never full-file Read.** One Edit call only. Use the closing `}` of `mod v1_sl_e_fixtures` (added by Task 1, untouched by Task 2 which inserted BEFORE it) as anchor `old_string` — find the unique closing `}` after `window_expiry_fires_full_lane`'s `Ok(())`.
4. `.claude/lessons/feedback_postgres_jsonb_canonicalization.md` — for `sponsor_pseudonym` payload assertions.

**Plan sections:**
5. `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md` §13 Task 3 — full IMPLEMENT spec
6. `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md` §10 — MIRROR refs (§10.6 programmatic backfill UPDATE; §10.4 scheduler tick; §10.5 force-rewind grace_expires_at; §10.7 governance_log assertions)
7. `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md` §4.2 — watchpoints (esp. #6 verbatim backfill SQL, #7 no edits outside e2e.rs)
8. `docs/brehon-law-inspired-network/04-data-model-and-api.md` — PRD §8.4 backfill SQL (copy verbatim into `diesel::sql_query`)
9. `.claude/rules/decision-queue.md` — Recipe 1 if blocker found

## §3a Handover from prior task

- **Task 2 (job-242):** commit `842593ab6` (test #2 window-expiry-fires) on phase-v1-SL-e. Phase 1 workspace check DQ #196 → pass. Phase 2 e2e (local) DQ #197 → **pass** (85 passed, 0 failed, 1795.39s).
- **phase-v1-SL-e tip:** `0a9b2cb31`
- **e2e.rs current mod structure:** `mod v1_sl_e_fixtures` contains, in order:
  - `seed_target_with_sureties_and_endorsements` helper
  - `count_log_entries` / `read_log_payload` helpers
  - `drive_jury_to_quorum` helper
  - `revocation_during_window_escapes_full_lane` test fn (Task 1)
  - `window_expiry_fires_full_lane` test fn (Task 2)
  - Closing `}` of the mod (still in place — Task 2 anchor-inserted before it)
- **Key decisions from prior tasks:**
  - `sql_query`, `update`, `SanctionInsertForm`, `AsyncConnection`, `moderation_case` DSL imports are now in-scope (Tasks 1 + 2 added them). Task 3 should mostly reuse; only add anything genuinely new (e.g. `diesel::sql_query` for the verbatim backfill UPDATE if not yet present).
  - `decided_at` is NOT in `ModerationCaseInsertForm` per SL-c-2 (e2e.rs:11982-12008) — use UPDATE post-insert (mirror that pattern).
  - **Single sponsor** for Test #3 minimal seed (multi-sponsor coverage already in Tests #1 + #2).
- **Case A canonical** confirmed across mod v1_sl_e_fixtures.

## §4 Constraints

- **Touch only:** `crates/server/tests/e2e.rs` + `.claude/decision-queue.json` (if blocker raised).
- **One anchor-Edit** before the closing `}` of `mod v1_sl_e_fixtures`. Do NOT Read the full file. Do NOT use `replace_all`.
- **Case A canonical**: `LemmyResult<()>` test fn outer. No `Box<dyn Error>`, no `.map_err`.
- **Env var safety**: `unsafe { std::env::set_var(...) }` (Rust 1.85+). Restore via `prev_disable` pattern per SL-c-2 at e2e.rs:12046-12049 + 12141-12146.
- **Verbatim backfill SQL per watchpoint #6**: copy PRD §8.4 SQL UPDATE verbatim into `diesel::sql_query(...)`. Comment immediately before the query cites PRD §8.4 explicitly. Do NOT paraphrase.
- **Rewind is a test artifact**: the IMPLEMENT comment immediately before the rewind UPDATE explicitly notes "test artifact — compresses §8.4's 24h grace into milliseconds for test purposes; production behaviour is 24h grace per PRD §8.4".
- **Anchor uniqueness**: Use enough context around the closing `}` of `mod v1_sl_e_fixtures` to make `old_string` unique — include the `Ok(())` + closing `}` of `window_expiry_fires_full_lane` + the mod's closing `}`. Task 2's test fn is currently the last item before the mod's closing `}`.
- **No edits outside e2e.rs (watchpoint #7)**: at task-end, run `git diff governance-v0..HEAD --stat`. EXPECT: ONE file: `crates/server/tests/e2e.rs | +<lines>`. If any other file appears (especially under `crates/`, `migrations/`, `.github/workflows/`, `docs/`), STOP and raise blocker.
- **Commit message:** `test(v1-SL-e): e2e test #3 — backfill-of-mid-flight (v0→v1 deploy) + close mod v1_sl_e_fixtures (task 3)`
- **Shape G:** after committing, push worker branch; write `kind: "validate-pending"` DQ entry with `workflow_run_id` from `gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --json databaseId`. Commit + push DQ entry.
- **DQ atomic raise:** `git add .claude/decision-queue.json && git commit && git push origin <branch>` THEN end task.
- **Encoding:** Python `json.dump(..., ensure_ascii=False, indent=2)` for DQ writes.
- **next_id:** compute across `.claude/decision-queue.json` AND `.claude/decision-queue-archive-*.json`. Current max on phase-v1-SL-e is 197 — next id is 198.
- **Attribution:** `from: "impl"`, never `from: "advisor"`.
