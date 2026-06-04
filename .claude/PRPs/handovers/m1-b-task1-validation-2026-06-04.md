---
phase: m1-b
scope: task1-migration-validation
authored: 2026-06-04T09:16:00Z
authored_by: advisor (canonical brehon-fork / governance-v0 session)
resume_command: /auto-phase M1
---

# Handover — m1-b Task 1 migration validation (resume point)

> Self-contained. Reading this + `.claude/auto-state/m1-b.json` is enough to resume with zero conversation context.

## RESUME BLOCK

- **Phase:** `m1-b` (M1 Tree B, plan Tasks 0–7). Branch `phase-m1-b`. Lane **Mode B** (canonical `brehon-fork` on `governance-v0` drives via Junior dispatch; impl-task briefs authored on trunk + trunk→phase synced).
- **Resume with:** `/auto-phase M1` (auto-resumes from `.claude/auto-state/m1-b.json`, `stage: impl-cohort-2-running`).
- **State-machine stage:** `impl-cohort-2-running` — cohort member = Task 1 (#574). Task is **done**; the cohort barrier is NOT passed because the **validate-pending-laptop round-trip has not run yet**.
- **VERIFIED_AT:** worker branch `3773cf323` (Task 1 commit), phase-m1-b tip `8929da9a0`, governance-v0 `a25635cd1`.

## What is DONE + VERIFIED (grep-confirmed this session)

1. **Task 0 #573** (pre-flight, all 7 probes PASS) — done.
2. **Task 1 #574** (governance_messaging_config migration) — **done** (finished 08:41:15 UTC). Worker branch:
   `junior/role-impl-task-m1-b-task-1-governance-messaging-config-migration-see-claude-prps-briefs-m1-b-impl-1-md-574` @ **`3773cf323`**, pushed to origin.
   - ✅ Both SQL files exist: `migrations/2026-06-03-000000-0000_add_governance_messaging_config/{up,down}.sql`
   - ✅ Commit subject correct: `feat(migration): add governance_messaging_config typed-column table (task 1)`
   - ✅ GOTCHA-50a: `CREATE UNIQUE INDEX governance_messaging_config_scope_key_valid_from_idx` on `(scope, key, valid_from)` — NOT `(scope, key)`.
   - ✅ Seed rows: literal `'2026-06-03T00:00:00Z'::timestamptz` (not `now()`), `ON CONFLICT (scope, key, valid_from) DO NOTHING`. Two rows: `messaging_enabled=false (bool)`, `identity_policy='pseudonymous' (text)`.
   - ✅ `validate-pending-laptop` DQ entry RAISED on the worker branch (`commands: ["./scripts/brehon/migrate-roundtrip.sh"]`, `from: "impl"`, `phase_task: 1`, `answered_by: null`).
   - ⚠️ **ONE UNVERIFIED ITEM:** `grep -ci value_float` on up.sql returned **1**. Context strongly suggests it is in an **explanatory comment only** (the seed INSERT column list has no float slot — 7 values matching int/bool/text schema). **Pre-validation check:** run `git show 3773cf323:migrations/2026-06-03-000000-0000_add_governance_messaging_config/up.sql | grep -i value_float | grep -v -- "--"` — if it returns nothing (exit 1), it's comment-only and fine. If it returns a line, the worker added a `value_float` column against the brief (M1 = int/bool/text only) → catch-fire / fix.

## NEXT ACTIONS (in order) — the validate-pending-laptop handler

1. **Confirm `value_float` is comment-only** (the ⚠ above). 10-second grep. If a real column → STOP, surface.
2. **Get the migration onto `phase-m1-b`.** The worker branch `3773cf323` may or may not be daemon-finalize-merged into `phase-m1-b` yet. Check: `git fetch origin phase-m1-b && git log 8929da9a0..origin/phase-m1-b --oneline`. If the Task 1 commit isn't there, either wait for daemon finalize OR Mode-B SSH-merge the worker branch into phase-m1-b (same temp-worktree pattern used for brief sync). The DQ entry also needs to be on phase-m1-b (or read it via `scripts/brehon/resolve-dq-canonical.sh m1-b`, which unions worker-branch DQs).
3. **Run `migrate-roundtrip.sh` LOCALLY** (this is the laptop validate-pending-laptop handler):
   - **Docker Desktop MUST be running** (script spins ephemeral `pgautoupgrade:18-alpine` containers).
   - From the laptop checkout on a branch that HAS the new migration (pull phase-m1-b first, or checkout the worker branch). The script auto-detects the new migration via `git diff --diff-filter=A origin/governance-v0...HEAD -- 'migrations/*/up.sql'` — **no positional arg**.
   - It runs `cargo run -p lemmy_diesel_utils --features full` (Windows: delegates to `scripts/brehon/migrate-roundtrip-cargo.bat`). ~2–4 min.
4. **Mutate the validate-pending-laptop DQ entry:** `result: "pass"|"fail"`, `answered_by: "advisor-laptop"`, `resolved_at`, (on fail: `log_slice` last 100 lines + `failed_commands`). On pass → move to `resolved[]`. **Commit subject must match `^(chore|docs)\((advisor|decision-queue)\)`.** Use the canonical-source approach if the entry lives on a worker branch.
5. **On pass → advance auto-state to Task 2** (impl-cohort-3): set `stage: "impl-cohort-3"`, author Task 2 brief. **Task 2 is non-`[P]`** — modifies 4 files (Diesel model `governance_messaging_config.rs` + its `mod.rs` + `newtypes.rs` + `schema.rs`). Tasks 3/4/5 `require:` Task 2's schema types. File-class lessons to inject at brief-author time: `feedback_newtype_locations_lemmy_db_schema_vs_file.md` (newtype under db_schema), and walk the §13 IMPLEMENT files against the advisor-orchestrator file-class table.

## After Task 2 lands: MiniMax trial (binding user override)

- User override (still binding): **run the MiniMax-M2.7-vs-Sonnet-4.6 impl-task A/B trial for eligible m1-b tasks even below the 5/5 cumulative threshold.** Eligible = Tasks **3, 4, 5** (3/5 qualifying). Arms fire off `phase-m1-b@T2` AFTER real T1+T2 land (T3/4/5 require T2's schema types). Deliverable = measured AB results in `.claude/PRPs/reports/minimax-m27-trial-results.md`. Runbook: `.claude/PRPs/briefs/minimax-m27-trial-1.md`.

## Cross-session deps / watch

- **No-cargo-on-daemon HARD RULE:** all cargo/Docker/e2e run on the LAPTOP only. Workers write validate-pending-laptop DQ + STOP.
- **Linux-compile gate:** M1 adds a migration → diff touches `migrations/**` → a `validate-pending-laptop-linux` DQ (`scripts/brehon/cargo-linux.sh check --workspace --features full`, Docker `rust:1.95`) must be `result:pass` before `bm-pr` (per `feedback_linux_compile_proof_is_a_gate.md`). Raise it after the migration + Task 2 model land.
- **DQ pending:** 1 (the Task-1 validate-pending-laptop on worker branch `3773cf323`). Read canonical via `resolve-dq-canonical.sh m1-b`.
- **Daemon:** active as of 09:15Z. No tasks running.
- **next user gate (not imminent):** Phase-2 e2e local-vs-dispatch (first e2e-bearing task), then CR triage after bm-pr, then merge, then retro.

## Session side-work (not phase-blocking, already shipped)

- `/memory-prune` run: MEMORY.md 24711→24151 bytes (under the 24400 budget), v1→M1 reframe. User-scope auto-memory (not git-tracked).
- `/memory-prune` SKILL improved + committed `a25635cd1` on governance-v0: added Step 2.5 milestone-transition sweep + transfer test; Step 4 apply-with-Edit-tool + grep-verify-each-edit + Historical-line-byte-cost discipline.
