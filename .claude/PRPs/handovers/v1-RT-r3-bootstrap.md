---
phase: v1-RT-r3
plan: .claude/PRPs/plans/v1-RT-r3.plan.md   # (not yet authored)
phase_branch: phase-v1-RT-r3                  # not yet created — cut at bm-cut
worktree: C:/Users/barri/Developer/brehon-fork-rt-r3   # created at bm-cut; until then use canonical brehon-fork
authored: 2026-05-24
authored_by: advisor (canonical brehon-fork / governance-v0 session)
updated: 2026-05-25 (post v1-deps-r1 ship — completing-phase pivoted from v1-RT-r2 to v1-deps-r1)
purpose: Bootstrap the v1-RT-r3 advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon v1-RT-r3.** The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-rt-r3` once `bm-cut` creates the lane worktree (per `.claude/rules/multi-lane-worktree.md`). There is no homeserver session.

**Completing phase pivoted (2026-05-25):** since this bootstrap was authored (2026-05-24), `v1-deps-r1` shipped between RT-r2 and RT-r3. The immediately-prior CLOSED record is now `workflow_state_v1_deps_r1.md` (NOT RT-r2). Read both — RT-r2 close record stays live (lane changed twice; two-ago delete skipped per skill).

- **RT-r2 closed:** PR #150 merged 2026-05-24T07:15:53Z, merge SHA `3b36b4e61c51cda1d16d360572a12199dd53819a`.
- **deps-r1 closed:** PR #153 merged 2026-05-25T11:47:52Z, merge SHA `ab1e79a70ed696f9dae413db61f1a2bf6077113f`. Retro: `.claude/PRPs/reports/session-retro-2026-05-25-v1-deps-r1.md`.
- **Worktree cleanup status (2026-05-25):** `brehon-fork-rt-r2` was cleaned at RT-r2 ship. `brehon-fork-deps-r1` was cleaned at deps-r1 transition (`git worktree prune` after the directory was emptied by stash-restore). Only active worktrees: canonical `brehon-fork` (governance-v0) + `brehon-fork-rt-r3` (`phase-v1-RT-r3` at tip `eaa4669ea`, pre-created from earlier session).
- **Lane mode for this RT-r3 session:** user is driving from canonical `brehon-fork` (mobile remote-control); all impl/bm phase-branch work dispatched as Junior tasks targeting the existing `brehon-fork-rt-r3` lane. Canonical session writes briefs to `governance-v0`; never `git checkout phase-v1-RT-r3` here (multi-lane-worktree.md §Hard refusals #1).

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane; verify rt-r2 worktree is separate (or already removed).
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `6aaaa5c23` (see §"Git state at handoff"); if drifted, `git log --oneline 6aaaa5c23..governance-v0` and update your mental model before acting.
3. Read `.claude/decision-queue.json` for any pending entries (0 at handoff — see §"Decision-queue snapshot").
4. `memory_search_hybrid(query: "participation consistency cron weekly dormancy vote outcome", limit: 5)` — load relevant lessons before authoring the planning brief.

## Next concrete action

Author `.claude/PRPs/briefs/v1-RT-r3-planning-1.md` (scope per PRD §5.3 "Multi-source participation_consistency events" + §5.4 "Vote-outcome + evidence-quality emitters") → `/brehon-clarify` → queue planning Junior (`[role:planning] v1-RT-r3 — see .claude/PRPs/briefs/v1-RT-r3-planning-1.md`).

---

## 1. v1-RT-r3 in one paragraph

v1-RT-r3 ships the multi-source `participation_consistency` event emitters. The PRD §11 phase 3 description: "Add weekly activity cron + dormancy cron in `scheduled_tasks.rs`; add vote-outcome + evidence-quality emitters in `submit_jury_vote.rs`; add `flag-bad-faith` admin endpoint." Source 1 (weekly activity cron): emits `+1 participation_consistency` per user per community with ≥1 non-deleted comment in the lookback window. Source 2 (dormancy cron): emits `−2 participation_consistency` per dormant user (zero comments in dormancy window). Source 3 (vote-outcome): emits `+1 participation_consistency` per juror who voted with majority at case-decided time. Source 4 (evidence-quality): emits `+1 reporting_accuracy` / `−1 reporting_accuracy` from the `flag-bad-faith` admin endpoint. r3 produces new `reputation_event` rows; r2's per-dimension calculator + bounds clamp absorb them transparently. DoD: four emitter paths tested, dedupe_key idempotent across same-ISO-week re-runs, e2e green with flag=false (v0 path unchanged).

## 2. Why v1-RT-r3 is harder/easier than v1-RT-r2

**Easier:** No new decay math — r3 only emits raw `reputation_event` rows; r2's calculator absorbs them. No feature-flag branching complexity (r3 events flow through the existing flag=true path naturally). The `dedupe_key` + `source_event_type` schema from r1 is the exact infrastructure r3's cron idempotency depends on.

**Not easier:** Significantly wider file footprint than r2 (r2 = 1 file in 1 crate; r3 = 3+ crates: `scheduled_tasks.rs` in `crates/routes`, `submit_jury_vote.rs` in `crates/api`, new `flag-bad-faith` handler, migration if new config keys land). Cron architecture requires per-community-tx atomicity. Four distinct event sources may produce 3–4 separate impl tasks. e2e test coverage for cron paths requires either a test-scoped cron trigger or direct fn-call through the scheduler.

## 3. Carry-forward from v1-deps-r1 + v1-RT-r2

**From v1-deps-r1 retro (2026-05-25, most recent — see `.claude/PRPs/reports/session-retro-2026-05-25-v1-deps-r1.md`):**

1. **Stash-restore DQ contamination on governance-v0 (WATCH — emerging pattern, not yet 2nd occurrence):** the `chore: restore stashed changes from earlier session` commit (`17b40e2a4` on deps-r1) reintroduced 2 resolved DQ entries into pending on `governance-v0`, causing bm-merge #461 to false-block. Before dispatching any bm-merge in RT-r3, verify `.claude/decision-queue.json` on `governance-v0` is clean (zero stale pending entries from prior stash/restore). Recovery if it happens: `chore(advisor): resolve bm-merge-gate-failure-NNN` to drain stale entries, then retry bm-merge. If 2nd occurrence in RT-r3, promote to a lesson.

2. **`git update-ref` blocked when daemon is checked out on governance-v0** — observed during deps-r1 post-finalize daemon sync. Fallback: `git merge --ff-only origin/governance-v0`. Carry to precheck Check 3b instruction.

3. **#292 stale-base recovery recipe applied (T2 deps-r1)** — cherry-pick from worker branch to lane is the working recovery (`feedback_junior_292_stale_base_recover_recipe.md`). If RT-r3 cron-emitter tasks dispatch via Junior on a stale daemon-local ref, the recipe is the same.

4. **Advisory miss (low signal):** `crates/diesel_utils/src/connection.rs:83` doc-comment referencing `scope_boxed()` was NOT deleted by the worker despite plan §12 specifying deletion. Non-blocking. If RT-r3 touches `diesel_utils` (unlikely — RT-r3 is cron + emitters), opportunistically clean up.

5. **bm-merge brief — findings YAML is gitignored** — bm-merge brief v2 explicitly noted this; carry to RT-r3's bm-merge brief template.

**From v1-RT-r2 retro (most critical §3 actions — still binding):**

**Action 1 (MANDATORY for all r3 impl-task briefs with e2e in DoD):**
Every impl-task brief whose DoD includes an e2e command MUST include this exact guard in §4 Constraints:

> e2e runs on **laptop only** — after cargo-check/clippy/unit-tests pass, write `kind: "validate-pending-laptop-e2e"` DQ entry (commands array = all 4 validate commands, branch, phase_task) and **stop**. Do NOT run e2e on the EliteDesk worker; the laptop advisor session runs it and mutates the DQ entry.

This guard was missing from the RT-r2 Task 2 brief; the worker ran e2e on the EliteDesk as an until-loop; the 65-min session ended while the loop was still polling. Impl was lossless (recovered via SSH diff), but cost ~30 min overhead and a manual commit from the laptop lane.

**Action 2 (worker impl recovery recipe — confirmed working):**
When a worker ends without committing: `git diff HEAD` in the worker worktree (`/srv/brehon-fork/.junior/worktrees/job-N/`) recovers the full uncommitted diff. Copy the target file to the lane, run all 4 validate commands locally, commit from the laptop lane. Lossless if the session ended mid-loop (not mid-edit).

**BM-pr brief visibility (recovered this phase):**
BM-pr briefs MUST be committed to `governance-v0` before BM task dispatch. The BM worker uses `base_branch=governance-v0`; a brief committed only to the phase branch is invisible to the worker. If the brief is on the wrong branch, the advisor opens the PR directly via `gh pr create --repo barrie-cork/lemmy` rather than re-queueing — valid recovery, but adds overhead. Carry-forward: commit BM-pr briefs to `governance-v0` at brief-authoring time.

**Advisor-side from RT-r2 retro:**
- **Daemon-merge reconcile overhead is predictable:** Task-1 finalize-merge → reconcile commit → DQ re-application = 2 extra commits. Budget in complexity scores for serial tasks.
- **e2e 37.8 min on the laptop (2267s)** — slightly longer than the ~26-min typical. Possibly due to High Performance power plan set 2026-05-22. Baseline may shift; allow 45 min in scheduling.

**From v1-ship-2 (closed same day as RT-r2 transition):**
- **DQ rebase conflict resolution:** take-HEAD for field updates on existing entries + union (take-BOTH) for new entries not present in HEAD. Bare "take HEAD" silently drops new array element additions.
- **Force-push after autonomous rebase** requires explicit user authorization. Auto-mode classifier correctly blocks it.

## 4. v1-RT-r3-specific watchlist

1. **`scheduled_tasks.rs` callsite count** — `crates/routes/src/utils/scheduled_tasks.rs:177` is the canonical scheduler location per PRD §5.2. Before authoring the planning brief, `rg "participation_cron\|participation_weekly\|dormancy_cron\|rollup_cron" crates/` to confirm no orphan cron stubs from r1 that r3 would double-register.
2. **`submit_jury_vote.rs` transaction boundary** — vote-outcome emitter (`+1 participation_consistency`) lands in the same `submit_jury_vote` transaction per PRD §5.4. Verify the handler already uses `conn.run_transaction()`; if not, the emitter addition is a `feedback_multi_write_handlers_need_transactions.md` scope trigger — surface to user.
3. **Dedupe key idempotency** — PRD §5.3 specifies `format!("activity_cron:{community_id}:{person_id}:{iso_week}")` and `format!("dormancy_cron:{community_id}:{person_id}:{iso_week}")`. The `dedupe_key` column has a unique constraint (from r1 schema). Re-runs in the same ISO week must produce zero new rows (constraint violation = idempotent success). Planner must include a dedupe-key idempotency test in §16a stories.
4. **New config keys** — r3 introduces 6 new governance_config keys (per PRD §5.3 table): `deltas.participation_weekly_active`, `participation.activity_threshold_comments`, `participation.lookback_days`, `deltas.participation_dormant`, `participation.dormancy_window_days`, `job.participation_interval_days`. These were NOT seeded in r1 (r1 seeded 26 keys, all in `decay.*`, `bounds.*`, `feature.*`, `job.snapshot_interval_seconds`). r3 will need a new migration to seed these 6 keys. Verify: `rg "participation_weekly_active\|participation_dormant\|lookback_days\|dormancy_window" crates/db_schema/src/source/governance/` — should return nothing if r1 didn't seed them.
5. **e2e.rs concurrent edit coordination** — `crates/server/tests/e2e.rs` may be touched by multiple active lanes. Check `git log origin/governance-v0 -- crates/server/tests/e2e.rs` for recent activity before authoring the r3 e2e brief. Mandatory lesson injections per §2.4: `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md`; ≥2 e2e edits → also `feedback_junior_worker_e2e_edit_hang.md`.
6. **`actor_pseudonym` system entry** — PRD §11 ADR-015 check: cron-batch entries use a synthetic `system` pseudonym (new addition per PRD §5.3; "reserved value, never collides with a real person's pseudonym"). Planner must specify how this synthetic pseudonym is registered in the `actor_pseudonym` table.

## 5. Operational rules

- **Polling cadence:** ~10 min `mcp__junior-brehon__list_tasks`.
- **Brief discipline:** briefs at `.claude/PRPs/briefs/v1-RT-r3-<role>-<n>.md`, committed to `governance-v0` BEFORE `create_task` (planning briefs, bm-cut briefs, bm-pr briefs). Impl-task briefs committed to `phase-v1-RT-r3` BEFORE impl task dispatch. Pre-queue: `/precheck` + `memory_search_hybrid` + §2.4 mandatory file-class lesson injection.
- **BM-pr briefs — MUST be on `governance-v0`** before BM task dispatch. Do NOT commit bm-pr briefs to phase branch and expect the BM worker to find them. See §3 carry-forward above.
- **Mandatory lesson injection (§2.4):** any edit to `crates/server/tests/e2e.rs` → `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md`; ≥2 e2e edits → also `feedback_junior_worker_e2e_edit_hang.md`; any handler doing 2+ DB writes → `feedback_multi_write_handlers_need_transactions.md`; any new migration → `feedback_lemmy_migration_runner.md`; any `#[cfg(feature = "full")]` gate → `feedback_features_full_workspace_only.md` + `feedback_features_full_p_crate_incompatible.md`.
- **e2e-on-laptop-only guard (MANDATORY in every impl-task brief with e2e in DoD):** see §3 Action 1 verbatim text. No exceptions.
- **Shape G SUSPENDED** until 2026-06-01 (DQ #229). validate-pending-laptop pathway active per `advisor-orchestrator.md §5.2`. Cargo runs on laptop.
- **Windows e2e invocation:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true`. Never bare `cargo test` on Windows.
- **Model tiering:** Planning→Opus, Impl→Sonnet, BM/ci-watcher→Haiku.
- **Clarify gate:** `/brehon-clarify` before every planning brief dispatch.
- **6 user gates:** plan approval (gate 1), judgment-heavy DQ (gate 2), CR triage (gate 3), Phase-2 e2e local vs dispatch (gate 4), merge confirm (gate 5), retro sign-off (gate 6). Never skip.
- **DQ attribution:** `chore|docs(advisor|decision-queue):` subject pattern for any advisor DQ write.
- **Multi-lane PMD:** canonical `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` — never relative path in `.mcp.json`.

## 6. What changed from v1-RT-r2's rule set

- **Domain shift:** single-file computation math → multi-source cron infrastructure. r3 touches `scheduled_tasks.rs` + `submit_jury_vote.rs` + a new admin handler + potentially a new migration. Cohort dispatch likely 2–3 tasks rather than 1–2. Serial constraints from `requires:` in FILES YAML will be heavier.
- **e2e-on-laptop-only guard now explicit** in every brief with e2e in DoD (was implicit + missed in RT-r2 Task 2).
- **New lesson active:** `feedback_laptop_default_for_validate_pending.md` augmented with brief-authoring constraint (e2e-on-laptop-only guard). Injected under any impl-task brief whose DoD includes e2e.
- **BM-pr brief placement rule now explicit:** commit to `governance-v0`, not phase branch (was implicitly required; RT-r2 bm-pr-1.md was committed to phase branch, worker couldn't find it, advisor opened PR directly — lesson promoted).
- **Parallel lanes:** if v1-RT-r4 or v1-ship-3 was active at RT-r2 close, check for active worktrees at session start and follow multi-lane CWD check discipline.

## 7. Catch-fire procedures

From `.claude/rules/advisor-orchestrator.md §5.5`:
- Junior writes `crates/**` without authorising brief → catch-fire.
- `answered_by: "advisor"` in commit with non-`chore|docs(advisor|decision-queue):` subject → catch-fire.
- bm-task opens PR into `main` instead of `governance-v0` → catch-fire.
- Phase branch has uncommitted state when Junior reports complete → catch-fire.
- Conformance-audit Tier-1 finding on governance Rust files → HARD REFUSAL, surface to user.
- Cycle-count ≥3 with same `(error_class, file_basename)` → HARD REFUSAL catch-fire.
- `validate-pending` mutated to fail/cancelled/timed_out + non-allowlist → catch-fire.
- **Phase-specific:** if planner proposes a new migration for `decay.*` or `bounds.*` config keys → catch-fire (those were r1 scope; a re-seed signals r1 schema drift, not r3 work).
- **Phase-specific:** if a worker ends without committing → recover via SSH diff per §3 Action 2; do not re-dispatch a new impl task before verifying the uncommitted work is recoverable.

## 8. Archive after v1-RT-r3

Run `/brehon-phase-transition v1-RT-r3 v1-RT-r4` (or `v1-RT-r5` if r4 completes first). This skill will: close `workflow_state_v1_RT_r3.md`, delete the two-ago record (`workflow_state_v1_RT_r2.md`), create the `v1-RT-r4` skeleton, write the next bootstrap file, update MEMORY.md, commit on governance-v0. This bootstrap file stays in `.claude/PRPs/handovers/` as its own archive.

---

## Git state at handoff (captured literally — do not paraphrase)

**Original handoff (2026-05-24):** governance-v0 HEAD `9f1584574` (post-RT-r2/ship-3 merges).

**Refreshed handoff (2026-05-25, post v1-deps-r1 ship):**

- governance-v0 HEAD: `6aaaa5c23` — `docs(retro): v1-deps-r1 — diesel-async 0.9 + sha2 0.11 + 8 SemVer-compat bumps; PR #153 merged ab1e79a70`
- v1-deps-r1 merge SHA: `ab1e79a70ed696f9dae413db61f1a2bf6077113f` (PR #153 merged 2026-05-25T11:47:52Z)
- v1-RT-r2 merge SHA (still relevant — calculator + bounds clamp absorb r3 events): `3b36b4e61c51cda1d16d360572a12199dd53819a` (PR #150 merged 2026-05-24T07:15:53Z)
- Phase branch: `phase-v1-RT-r3` already exists at `eaa4669ea` (lane worktree `brehon-fork-rt-r3` pre-created from earlier session); branch did NOT come from a bm-cut against this transition's trunk — verify it forks cleanly from `6aaaa5c23` OR run a forward-merge before first impl push.
- Recent governance-v0 commits:

  ```
  6aaaa5c23 docs(retro): v1-deps-r1 — diesel-async 0.9 + sha2 0.11 + 8 SemVer-compat bumps; PR #153 merged ab1e79a70
  ab1e79a70 Merge pull request #153 from barrie-cork/phase-v1-deps-r1
  222bb5671 chore(advisor): author v1-deps-r1 bm-merge-2 brief (retry — DQ now clean)
  87de1e37f chore(advisor): resolve bm-merge-gate-failure-001 — stale pending entries from restore-stash; lane DQ clean (0 pending)
  2346ed06f chore(decision-queue): merge bm-task v1-deps-r1-bm-merge
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at refreshed handoff 2026-05-25 — 0 pending entries; verified `cat .claude/decision-queue.json | jq '.pending|length'` = 0; resolved count: 187)
Note: DQ #229 (Shape G re-enable) is in resolved[], dated reminder for 2026-06-01 (~1 week out — verify before any GH-Actions-side cargo work).
Note: RT-r2 validate-pending entries (8aca794fb044-001, b246616aaf8f-001, b6b7e4a77e02-001) all in resolved[].
Note: ship-3 clarify DQs (ship3clarify01-001/002/003) all in resolved[].
Note: deps-r1 lifecycle DQs (a22859c2ae07-001..005, 2b05f9a6183b-001, plus advisor-mode clarify entries) all in resolved[].
Note: bm-merge-gate-failure-001 from deps-r1 cycle is in resolved[] (stash-restore DQ contamination — watch for recurrence in RT-r3).
```

## Stop-and-ask tripwires

- Stop and ask if: `rg "participation_weekly_active\|participation_dormant\|lookback_days\|dormancy_window" crates/db_schema/src/source/governance/` returns hits — would mean r1 or ship-3 already seeded these keys; r3's migration scope needs adjustment.
- Stop and ask if: `rg "participation_cron\|participation_weekly\|dormancy_cron" crates/routes/` returns hits — orphan stubs would conflict with r3's cron registration.
- Stop and ask if: the planner proposes more than 4 impl tasks — r3's file footprint is wider than r2's, but 5+ tasks signals the planner is over-splitting; check if tasks can be batched by emitter pair.
- Stop and ask if: `submit_jury_vote.rs` does NOT already use `conn.run_transaction()` — the vote-outcome emitter addition requires a transaction boundary; if the handler is currently non-transactional, the scope of change is larger than the brief anticipates.
- Stop and ask if: the Phase-2 e2e log shows a failure in any pre-existing `v1_*_fixtures` test — regression suspected; do not auto-queue a fix-impl before surfacing to user.
- Stop and ask if: `git log governance-v0 ^phase-v1-RT-r3 --oneline` is non-empty (i.e. trunk is ahead of the pre-created rt-r3 branch tip `eaa4669ea`) — RT-r3 branch was created BEFORE deps-r1 shipped; forward-merge `6aaaa5c23` into `phase-v1-RT-r3` before any impl push, OR delete + recut from `governance-v0` if the lane has zero work. Per `feedback_daemon_local_trunk_stale_multi_lane.md`.
- Stop and ask if: `cat .claude/decision-queue.json | jq '.pending|length'` returns non-zero before dispatching bm-merge — stash-restore DQ contamination pattern (deps-r1 emerged). Drain stale pending via `chore(advisor): resolve bm-merge-gate-failure-NNN` before retry.
