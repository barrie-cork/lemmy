# v1-JM-d retro — Jury Mechanics sub-phase D (Diesel models + R3 InsertForm sweep)

**Sub-phase:** v1-JM-d (Diesel models for appeals v1, R3 sweep on InsertForm callers, Tasks 1-7 progressively)
**Branch:** Direct-to-trunk (governance-v0); no `phase-v1-JM-d` PR cycle. Tasks 1-3 shipped via Junior worker branches with finalize-merge into trunk (pre-Shape-G); Tasks 4-5 shipped direct-to-trunk from Mac session (out-of-band shortcut, see §1.7).
**Base:** governance-v0 @ pre-Task-1 tip (`bm-cut` ran 2026-04-27 task #8)
**Plan:** `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` (confidence 6/10 at write; reality settled to ~5/10 — see §6)
**Dates:** 2026-04-27 → 2026-04-30 (4 days, ~14 commits ahead of pre-cut tip)
**Impl model:** mixed — Tasks 1-3 four-role Junior orchestration (advisor → planning → impl → BM); Tasks 4-5 foreground from Mac session direct-to-trunk; advisor session re-engaged at retro time. **First sub-phase to exercise the four-role model end-to-end** — see §2 framing.

---

## TL;DR for the next advisor

**Plan was overscoped; four-role model exercised but with three serious infrastructure incidents (settings.json drift causing 200k-context overflow, EliteDesk OOM cascade from serena+cargo, SQLite WAL collision causing zombie DB row); GH workspace-check found a sibling clippy finding that local-clippy missed because the original fix unmasked it.** All Tasks 1-5 shipped functionally; Tasks 6-7 (e2e fixtures + registry flips + retro) collapsed into this retro because the JM-d work substantively closed when Task 5's e2e passed. Five infrastructure lessons promoted to homeserver PMD (mirrored to `.claude/lessons/` discipline pending — see §5).

The four-role model worked but stressed every joint. Specifically:

1. **Settings.json model pin drift** caused a 200k-context overflow on Task 2 (resumed as task #13 after `claude-opus-4-7` was corrected to `opus[1m]`). Lesson: ensure `.claude/settings.json` model field on phase branches matches the 1M-context family identifier; verify via `head -3` in worker pre-flight.

2. **Mass-cancel SQLite WAL collision** orphaned job #55 (sweep-c1-bm-tooling) when 8 parallel sweep jobs were cancelled within 40 seconds. The cancel-handler's status-flip write failed under WAL contention with no retry, leaving `jobs.status='running'` indefinitely. Cleared via direct DB write 2026-04-30 18:10 UTC. Lesson `feedback_junior_cancel_db_lock_zombie.md` captures both the diagnostic and the fix-SQL.

3. **Serena MCP auto-spawned cargo check on every parallel worker.** When the sweep cohort dispatched 6 workers concurrently, each worker's serena MCP auto-instantiated rust-analyzer and ran `cargo check --workspace`. Six concurrent cargo workspaces on the EliteDesk (10 GB cgroup cap, 16 GB box) hit OOM cascade with load=26.41, swap full, 12/15 GB RAM consumed. Root cause: serena's auto-LSP design + parallel cohort dispatch + cargo's per-target lock contention. Mitigated by removing serena from `/srv/brehon-fork/.mcp.json` (backup at `.mcp.json.bak.2026-04-30`); rust-analyzer-lsp@claude-plugins-official user-scope plugin retains the LSP tool. Smoke task #64 confirmed `LSP_AVAILABLE=yes` post-fix.

4. **Mac task-hopper bundling missed a sibling clippy finding.** Commit `953a8360b` ("drop dead store + stale comment") removed the dead-store on `admin_assign_jury.rs:727` but left `mut` stale on line 653. Local clippy at fix-time still reported clean *because* line 727 was the only finding *before* the fix; the fix itself unmasked the sibling `unused-mut`. Caught later by GH workspace-check on `25181904020`. Lesson promoted: re-run clippy after applying any clippy fix.

5. **GH workspace-check is ~3x slower than laptop clippy** (21+ min queue+run vs ~4 min warm laptop) even with `actions/cache@v4` configured for `target/`. Decisive evidence that laptop-default for routine validate-pending conserves GH minutes for the big PR runs. Lesson promoted: default to `kind: validate-pending-laptop` for advisor-driven supersede / fix-validation rounds.

The mandatory user gates held — plan approval, judgment-heavy DQ, merge confirms (where applicable), retro sign-off. No gates were skipped for speed.

---

## 1. What worked — keep doing

### 1.1 Four-role orchestration shipped Tasks 1-3 cleanly under Shape G

Task 3 was the cleanest — `[role:impl-task]` + ci-watcher Phase 1 + ci-watcher Phase 2 cycle ran exactly as designed in v1-validate-agent. Both ci-watchers mutated their `validate-pending` entries correctly, advisor's polling loop picked up the result transitions within one tick. The push-and-exit impl-task contract held; no advisor manual finalize-merge was needed.

**Keep**: the v1-validate-agent infrastructure design works under real load. Shape G's "cargo runs on GH Actions" promise delivers when the workflow YAMLs are correctly configured and cache keys are stable.

### 1.2 validate-pending-laptop pattern proved out for Task 5 e2e

Task 5's e2e ran on the laptop in `run_in_background` per the Phase 2 e2e user gate (PR #105 — user-picks-local-vs-dispatch). Result: 62 passed / 0 failed / 3 ignored, 1741.48s wall-clock (~29 min). Zero billed GH minutes. DQ #97 (`kind: "validate-pending-laptop-e2e"`) mutated cleanly with `result: "pass"` and full evidence. `cmd.exe /c scripts\brehon\cargo-test.bat` wrapper handled the libpq.dll PATH correctly for Windows-native test execution.

**Keep**: when e2e is needed and the user picks "local," the validate-pending-laptop handler in `advisor-orchestrator.md` works as documented. The `.bat` wrapper compatibility note for pre-2026-04-29 worker branches (per `feedback_libpq_path_dll`) was also load-bearing.

### 1.3 Direct-to-trunk shortcut worked when advisor session was unreachable (Tasks 4-5)

Mac session driving Tasks 4-5 directly into trunk (no phase branch / PR) per the authorisation in `project_brehon_jm_d_task4_5_direct_to_trunk.md` produced 6 trunk commits in a few hours including CR-fix bundles. The gh issues #84/#85/#88 were closed inline via task-hopper-style bundles. No Mac↔laptop coordination races (the `feedback_active_parallel_session_fetch_first.md` lesson held: every Mac commit was visible to laptop within a `git fetch` round-trip).

**Keep**: the direct-to-trunk shortcut is appropriate for low-risk hygiene/CR-fix work when one session is already authoring at speed and the advisor session is sleeping. The lesson `feedback_disclose_shared_state_edits.md` (surface conflict-safety reasoning before editing) was honored by Mac's commit messages naming the trail commits explicitly.

### 1.4 Decision-queue mid-task push worked under Shape G

Tasks 3 + 4 + 5 each pushed `validate-pending` DQ entries from `from: "impl"` mid-task, advisor polling loop picked them up on next tick, ci-watcher (or laptop handler) mutated them in place. The schema-v2 entries (with `kind`, `workflow_run_id`, `branch`, `phase_task`, `result`, `log_slice`, `failed_jobs` fields) round-tripped correctly through the JSON schema. No drift between writers (impl) and readers (advisor + ci-watcher).

**Keep**: Schema-v2 validate-pending entries are mature. The §G4 classifier ran on Task 4 + Task 5 fails (allowlist-match → narrow fix-impl-task brief queued; one catch-fire avoided). Worth keeping the kind:validate-pending family as the load-bearing async coordination primitive.

### 1.5 Pre-flight precheck skill caught one failure mode early

The `/precheck` skill (5-probe pre-queue check, per `pattern_junior_pre_queue_discipline`) ran before each impl-task dispatch, surfaced one stale-stash + one CWD-drift before queueing. Cost: <2s. Catch: would-be-dirty-worktree contamination prevented twice.

**Keep**: `/precheck` is canonical pre-queue discipline. The 6-gate checklist (git, lessons, mirror, memory, cron, PMD) is the right shape; consider promoting to a Junior pre-flight hook if user-scope work permits.

### 1.6 User gates held throughout

Every named user gate per `.claude/rules/advisor-orchestrator.md` fired correctly:
- Plan approval: surfaced after planning task #7 + DoD smoke test + watchpoint-specificity gate.
- Phase 2 e2e local-vs-dispatch user gate: surfaced for Task 4 (chose dispatch) and Task 5 (chose local). Both choices produced clean evidence.
- Judgment-heavy DQ entries: zero user-relays needed during JM-d (all `pending` entries were schema-v2 validate-pending mechanical mutations; the planner-side advisor-resolved DQs from earlier sub-phases stayed self-resolved).
- CR triage: N/A (no formal phase PR; see §2.4).
- Merge confirm: N/A (direct-to-trunk; finalize-merge by Junior daemon for Tasks 1-3, direct push for Tasks 4-5).
- Retro sign-off: this retro itself.

**Keep**: gate discipline is load-bearing; the slow-OK + reliable autonomy goals (per `feedback_brehon_autonomy_goals`) require *not* skipping any gate even when the work appears mechanical.

### 1.7 Direct-to-trunk authorisation was useful and well-documented

The Mac-direct-to-trunk shortcut for Tasks 4-5 worked because:
- The authorisation block in `project_brehon_jm_d_task4_5_direct_to_trunk.md` was explicit ("when advisor session unreachable").
- Mac commits referenced the trail explicitly (`bb00edd55` = "trail #71 + #47 closure — bundled into 953a8360b").
- The advisor session re-engaged on resume by reading the handoff block, not by trying to retroactively reconstruct Mac's intent.

**Keep**: when one session needs to operate without the orchestrator, document the authorisation + scope + rollback path *before* the work starts. The shortcut is fine when bounded and traceable; gets messy when freeform.

---

## 2. Per-role signals (four-role exercised end-to-end)

### 2.1 Advisor signals — first real four-role exercise; three infrastructure incidents

This was the first sub-phase to fully exercise the persistent advisor session as orchestrator:

**Worked:**
- Polling loop discipline held (~10-min cadence on `list_tasks`; full state load only on transitions).
- DQ triage decision tree fired correctly on each new `pending` entry (advisor-answer / catch-fire / user-relay paths all exercised at least once).
- Watchpoint-specificity gate held — every plan §4 watchpoint cited a specific table/line.
- DoD smoke-test gate held on plan approval.
- Two-phase Shape G validation (workspace-check on `junior/*`, e2e on `phase-v1-*`) ran for Tasks 3-5 as designed.

**Stressed:**
- **Settings.json model drift incident (Task 2 / 2026-04-27).** `phase-v1-JM-d/.claude/settings.json` pinned `claude-opus-4-7` instead of `opus[1m]`. Worker hit 200k-context overflow at peak token usage 205,096 (126,469 cache_read + 78,627 cache_creation), api_retry events ramped up. Recovered: cancelled task #12, fixed settings.json drift, applied rescued patch as `082319c`, requeued as #13. Cost: ~3 hours of advisor babysitting + 1 wasted Junior session.
- **Mass-cancel zombie incident (Sweep cohort / 2026-04-30).** Advisor cancelled 8 sweep jobs (#55-#63) after the EliteDesk OOM cascade. Job #55's status-flip write failed under SQLite WAL contention (8 simultaneous cancels in <40s). Row stuck `running`; daemon's `Active jobs: 1` falsely reported active work for 2+ hours. Cleared via direct DB UPDATE 18:10 UTC. The advisor's polling loop never detected this — `daemon_status` reads the same stale row. Lesson `feedback_junior_cancel_db_lock_zombie.md` captures the diagnostic SQL + fix-SQL + upstream-fix candidate.
- **Serena+cargo OOM cascade (Sweep dispatch / 2026-04-30).** Advisor dispatched 6 sweep workers in parallel; each worker's serena MCP auto-spawned cargo check; cumulative load + memory exceeded EliteDesk capacity (load=26.41, swap full, 12/15 GB used). Catch-fire to user; serena removed from `.mcp.json`; rust-analyzer-lsp plugin retained for LSP tool availability. The forbidden-window check held (sweep was outside 02:55-04:15 UTC NAS-backup window) but the *spatial* contention from parallel workers wasn't anticipated. Lesson `feedback_serena_auto_cargo_check.md` already promoted.

**Carry-forward for v1-JM-e retro:**
- Watch settings.json model field on every new phase branch immediately after `bm-cut` — verify via `head -3 .claude/settings.json` matches `opus[1m]` (or whatever 1M-context identifier the four-role tiering patch expects). Add to `/precheck` if not already there.
- Cohort dispatch with `[P]` markers needs a serial-cargo gate per `feedback_serena_auto_cargo_check.md`: if removed-serena environment, no contention; if serena's still present anywhere, fall back to `max_concurrency: 1` until serena's gone everywhere.
- Daemon's startup reconciliation does NOT scan stale-running rows whose worker PID is dead. Restart-as-recovery is a false friend for zombie rows. Direct DB write is the only path until upstream patches the cancel-handler retry.

### 2.2 Planning signals — overscoped; one phantom DQ; complexity score signal worked

**Worked:**
- Plan §5 complexity-score awareness fired on Tasks 4-5 (cargo-class, score >8) — pre-queue lesson check pulled in `feedback_junior_worker_e2e_edit_hang.md` and the libpq.dll Windows-PATH lesson. Both were load-bearing during impl + validate cycles.
- Plan §13 task list mostly held; 5 of 7 tasks shipped per plan; Tasks 6-7 collapsed into retro (see §3.1).

**Stressed:**
- **Overscope.** Plan was authored as a 7-task sub-phase (Diesel models + R3 sweep + e2e fixtures + registry flips + retro). Reality: the Diesel models + R3 sweep work substantively closed the sub-phase by Task 5; Tasks 6-7 were cleanup + retro, not material new work. The plan should have been a 5-task plan with retro as the only post-Task-5 deliverable.
- **DQ #68 orphan inert.** A planner-side DQ from earlier sub-phase work referenced `migrate-roundtrip.sh` which is still a stub from v1-validate-agent Task 1. JM-d Task 2 was supposed to replace it; the replacement did not happen because Task 2's cargo-incompatible flow shifted to Shape G mid-flight. Carry-forward for the next migration-authoring sub-phase: the stub at `scripts/brehon/migrate-roundtrip.sh:1-3` still exits non-zero on real migration detection.

**Carry-forward:** plans for sub-phases that primarily refactor existing patterns should target ≤5 tasks; the e2e-fixture + registry-flip work that JM-d planned as Tasks 6-7 is better as a follow-up sub-phase or explicit retro deliverable, not a §13 task.

### 2.3 Impl signals — Tasks 1-3 clean four-role; Tasks 4-5 direct-to-trunk shortcut

**Tasks 1-3 (four-role Junior):**
- All three shipped via `[role:impl-task]` Junior workers with finalize-merge. No mid-task DQ trapping issues.
- Task 2 hit the settings.json drift incident (resolved); Task 3 ran cleanly through Phase 1 + Phase 2 ci-watchers.
- Worktree-per-job isolation held — no parallel-agent shared-worktree race conditions.

**Tasks 4-5 (Mac direct-to-trunk):**
- Authorised shortcut per `project_brehon_jm_d_task4_5_direct_to_trunk.md`; no formal phase-branch / PR cycle.
- Mac session bundled CR-fix work for issues #84/#85/#88 into the same commit-stream — efficient, but mixed task-per-commit history (the canonical retro pattern).
- Task 5's e2e ran on laptop per Phase 2 user gate; full evidence captured in DQ #97.
- One sibling clippy finding (line 653 unused-mut) escaped Mac's local clippy because the original fix at line 727 unmasked it. Caught later by GH workspace-check.

**Carry-forward:** when a Mac/laptop direct-to-trunk shortcut is authorised, the "task-per-commit" retro convention should be relaxed in advance — bundled CR-fix commits are fine, but the retro should explicitly note when one trunk commit closes multiple tasks.

### 2.4 BM signals — no formal PR cycle for JM-d

JM-d shipped without a phase PR. Tasks 1-3 finalize-merged into governance-v0 directly via Junior; Tasks 4-5 went via Mac direct push. No CodeRabbit review cycle. No CR triage. No bm-merge confirm.

**Implication for retros:** the "BM signals" section of the retro corpus assumes a per-phase PR cycle. JM-d's direct-to-trunk path means BM-equivalent work is empty. Future sub-phases that take this shortcut should explicitly note "no BM cycle" in the retro plan section rather than authoring an empty BM signals section.

**Carry-forward for v1-JM-e:** if v1-JM-e returns to the phase-PR pattern (which is the default per `feedback_pr_per_phase.md`), the BM signals section becomes load-bearing again. Watch CR triage at v1-JM-e merge time for any new findings the JM-d direct-to-trunk path masked.

---

## 3. What didn't work — fix or watch

### 3.1 Tasks 6-7 collapsed into retro (plan overscope)

Plan §13 had Task 6 (e2e fixtures under `mod v1_jm_d_fixtures`) and Task 7 (registry flips + retro). Reality: the e2e fixtures got built into Task 5's e2e suite directly; the registry flips became hygiene commits the Mac session bundled into Task 4-5 trail commits. Neither shipped as a standalone task.

**Fix:** for the next sub-phase, plan §13 task count should be calibrated to "what's a meaningful unit of work that produces its own §16a story" — not "what fits the convention." If a task's work is hygiene that naturally rides on other tasks' commits, fold it into the parent task's §16a story rather than spinning a separate task.

### 3.2 Settings.json model drift wasted ~3 hours (Task 2 incident)

Already covered in §2.1. The fix is mechanical — verify `.claude/settings.json` model field after `bm-cut` and after any settings.json mirror commit.

**Fix:** add to `/precheck` skill's git probe — if `head -3 .claude/settings.json` shows a model identifier that doesn't match `opus[1m]` (or whatever tier the task expects), refuse and surface to user.

### 3.3 SQLite WAL contention zombie row (Task #55 / 2026-04-30)

Already covered in TL;DR + §2.1. The cancel-handler in `/opt/junior-src/src/daemon/executor.ts` swallows `database is locked` exceptions without retry. Root cause: parallel cancel of 8 jobs in <40s flooded the WAL with simultaneous writes; one lost the lock race; that one's row stuck `running` indefinitely.

**Fix:** upstream patch candidate — wrap the cancel-handler's `jobs.status='cancelled'` write in retry-with-backoff (5x, 100ms-1s) for SQLITE_BUSY. Currently fails fast and silently. Until patched, work around by staging cancels (cancel one, wait for journal "Job cancelled by user" line, then cancel next) or by direct DB write to clear zombies after-the-fact.

**Watch at v1-JM-e:** if a similar mass-cancel happens, expect at least one zombie. Diagnostic SQL is in `feedback_junior_cancel_db_lock_zombie.md`.

### 3.4 Serena+cargo OOM cascade (Sweep dispatch / 2026-04-30)

Already covered in TL;DR + §2.1. Six parallel workers with serena MCP each auto-spawned rust-analyzer + cargo check; load=26.41, swap full, 12/15 GB used. Catch-fire fired correctly; serena removed; rust-analyzer-lsp plugin retained for LSP tool.

**Fix:** serena removed from `/srv/brehon-fork/.mcp.json` (already shipped). Lesson `feedback_serena_auto_cargo_check.md` documents the failure mode.

**Watch at v1-JM-e:** verify serena does NOT come back via any settings mirror or rule patch. Smoke task #64 (post-removal) confirmed LSP tool still available; smoke a similar task at v1-JM-e bm-cut to re-verify.

### 3.5 Clippy fix unmasked sibling finding (admin_assign_jury / 2026-04-30)

Already covered in TL;DR. Mac session committed `953a8360b` (line 727 dead-store removed); local clippy clean at the time; GH workspace-check on `25181904020` then caught `unused-mut` on line 653 (the binding the dead-store had been writing to). Fix shipped on `junior/jm-d-task-5-workspace-recheck` as `3f7dfec22` ("drop unused mut on current_geo_enabled in admin_assign_jury").

**Fix:** lesson `feedback_clippy_rerun_after_fix.md` promoted to homeserver PMD. Re-run clippy after applying any `unused-*` fix before pushing.

**Watch at v1-JM-e:** any task-hopper bundle that includes `unused-*` clippy fixes should run a final `cargo clippy --workspace --features full --no-deps -- -D warnings` after all fixes are applied and before commit. Cost is ~4 min warm on laptop; the catch is one entire GH workspace-check round-trip avoided.

### 3.6 GH workspace-check ~3x slower than laptop clippy on cache-hit (validation incident / 2026-04-30)

GH run `25181904020` took 21+ min queue+run for what laptop clippy completed in ~4 min warm. Cache config (`actions/cache@v4` for `~/.cargo/registry`, `~/.cargo/git`, `target/`) was correct; key (`cargo-validate-workspace-${{ runner.os }}-${{ hashFiles('Cargo.lock') }}`) should have hit since `Cargo.lock` hadn't changed since the last green run. The slowness is GH-runner-side queue + cold-start overhead on this workspace size.

**Fix:** lesson `feedback_laptop_default_for_validate_pending.md` promoted to homeserver PMD. Default to `kind: validate-pending-laptop` for routine advisor-driven supersede / fix-validation rounds; reserve `kind: validate-pending` (GH-side) for impl-task pushes where the workflow run is the canonical evidence trail for a per-phase PR.

**Watch at v1-JM-e:** if any validate-pending round-trip exceeds 15 min on GH, reconsider whether the run was actually load-bearing or whether laptop-side would have sufficed.

---

## 4. Per-task complexity score table

Per `feedback_retro_task_complexity_score.md`. Format: `<files-touched>/<commits>/<runtime-min>/<max-log-silence-min>`.

| Task | Subject | Files | Commits | Runtime (min) | Max log silence | Notes |
|---|---|---|---|---|---|---|
| 1 | Diesel migrations + InsertForm append | ~8 | 1 | ~24 | ~3 | Cleanest task; four-role baseline |
| 2 | Diesel models extension + R3 sweep | ~12 | 2 | ~165 | ~12 | Settings.json incident; resumed as #13 after recovery |
| 3 | submit_jury_vote lock-ordering + idempotency | ~6 | 4 | ~96 | ~6 | Phase 1 + Phase 2 ci-watchers + 2 fix-3a/3c rounds |
| 4 | admin_dashboard active_cases + e2e | ~10 | 4 | ~145 | ~8 | Phase 1 fail → fix-4a-v2 → fix-4a-v3; Phase 2 e2e PASSED on laptop |
| 5 | Appeal-window-expiry background job + scheduler tick | ~8 | 3 | ~95 | n/a (laptop-driven) | E2e PASSED on laptop (DQ #97); workspace-fail at #90 superseded via this retro's recheck branch |
| Retro | This file + 2 promoted lessons | 3 | 1 | ~60 | low | Including `feedback_clippy_rerun_after_fix.md` + `feedback_laptop_default_for_validate_pending.md` |

**Aggregate:** ~47 distinct files, 15 commits, ~585 min total wall-clock across 4 calendar days. Median task ~96 min. Outlier: Task 2 (~165 min) — settings.json incident dominated. Even excluding the recovery, Task 2 was ~70 min of impl (within Sonnet 60-min watchdog envelope but borderline).

**Carry-forward:** Tasks averaging >120 min wall-clock are at watchdog risk under Sonnet impl. Plan complexity score `>8` correctly flagged Tasks 4-5 as cargo-class; for v1-JM-e, score the cargo-class tasks honestly and consider splitting any single task that projects >100 min wall-clock.

---

## 5. Lessons promoted to homeserver PMD

Five infrastructure lessons emerged this sub-phase. All saved to `homeserver/.claude/projects/.../memory/` (the laptop-side PMD, indexed by `MEMORY.md`). Mirror-to-`.claude/lessons/` discipline is pending for next mirror commit (per `pattern_lesson_lifecycle_chain`).

1. **`feedback_clippy_rerun_after_fix.md`** — high-applicability. After any `unused-*` clippy fix, re-run clippy on the affected crate (and ideally the workspace) before pushing. The original lint was downstream of dead code; removing the dead code can promote sibling bindings to also-stale state. Cost: 30s-4m warm. Catch: one entire GH workspace-check round-trip avoided per slip. Source: incident on `admin_assign_jury.rs:653` post-`953a8360b`.

2. **`feedback_laptop_default_for_validate_pending.md`** — load-bearing. GH workspace-check is ~3x slower than laptop clippy even on cache-hit (21+ min vs ~4 min). Default to `kind: validate-pending-laptop` for advisor-driven supersede / fix-validation. Reserve `kind: validate-pending` (GH-side) for impl-task pushes where the workflow run is canonical PR evidence. Conserves GH minutes for big PR runs. Source: GH run `25181904020` cycle vs laptop clippy on the same diff.

3. **`feedback_junior_cancel_db_lock_zombie.md`** — diagnostic + fix. Mass-cancel SQLite WAL collision causes zombie `jobs.status='running'` rows. Provides diagnostic SQL + fix-SQL + upstream-patch candidate (retry-with-backoff on SQLITE_BUSY in cancel-handler).

4. **`feedback_serena_auto_cargo_check.md`** — already-saved. Serena MCP auto-spawns rust-analyzer + cargo check per parallel worker; remove from `.mcp.json` before parallel batches; rust-analyzer-lsp@claude-plugins-official plugin retains LSP tool.

5. **`feedback_active_parallel_session_fetch_first.md`** — already-saved. Mac+laptop both committing live; always `git fetch && git log @{u}` before any reset.

---

## 6. Confidence score

**5/10.**

Lower than v1-validate-agent's 8/10 because three serious infrastructure incidents (settings.json drift, mass-cancel zombie, OOM cascade) all surfaced during this sub-phase, each costing significant advisor time + at least one wasted Junior session. Discounted from a higher score because:

- Tasks 6-7 collapsed into retro rather than shipping as designed §16a stories — plan overscope is the planner's signal, not a retro praise point.
- Settings.json drift was preventable but not prevented (lesson promoted now).
- Mass-cancel zombie required direct DB write to clear; daemon has no reconciliation path.
- Serena OOM cascade was preventable in retrospect — should have been caught at the cohort-dispatch design review, not at task #55 catch-fire.
- Mac task-hopper bundling missed a sibling clippy finding; cost was one wasted GH workflow run + one diagnostic round.

Praise points that prevent a lower score:

- Functional acceptance held — all material JM-d work shipped (Diesel models + R3 sweep + appeal-window-expiry job + admin_dashboard + e2e regressions all pass).
- Four-role model exercised end-to-end on Tasks 1-3; ci-watcher mutated DQ entries correctly; Phase 1 + Phase 2 validation worked.
- Direct-to-trunk shortcut for Tasks 4-5 produced honest commit history with explicit trail commits.
- Five infrastructure lessons promoted; v1-JM-e and onwards inherits the avoidance discipline.
- User gates held — every named gate fired; no skips for speed.

The headline "v1-JM-d closes Jury Mechanics ahead of v1-jury-mechanics-e" is **achieved functionally but with significant infrastructure debt repaid in lessons**. v1-JM-e will be the test of whether those lessons stick or whether new failure modes surface.

---

_Retro author: persistent advisor session (homeserver, `C:\Users\barri\Developer\homeserver`, 2026-04-30 19:55 UTC). Resumed from `project_brehon_phase_v1_jm_d_notes.md` handoff @ trunk `ead792254`. Five infrastructure lessons promoted to homeserver PMD; mirror to brehon-fork `.claude/lessons/` discipline pending for next mirror commit. v1-JM-e brief authored + clarified; planning task awaiting user approval gate per `.claude/rules/advisor-orchestrator.md`. Workspace-check supersede on DQ #90 superseded by this retro's evidence trail (run 25181904020 surfaced clippy-rerun lesson; downstream `3f7dfec22` clippy fix on `junior/jm-d-task-5-workspace-recheck` is the substantive supersede artifact); explicit DQ #90 supersede mutation deferred to user authorisation per the "retro is the supersede evidence" decision 2026-04-30 19:50 UTC._
