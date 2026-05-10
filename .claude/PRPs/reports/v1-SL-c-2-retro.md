# Retro: v1-SL-c-2 — 5 grace_check e2e tests

**Date:** 2026-05-10
**Sub-phase:** v1-SL-c-2
**Plan:** `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md`
**Shipped:** `governance-v0` tip `adc25da49` (cherry-pick merge)

---

## §1 Outcome

**Shipped.** 5 e2e test stubs in `mod v1_sl_c_fixtures` (`crates/server/tests/e2e.rs`):

| Task | Test fn | Commit | Impl runtime |
|---|---|---|---|
| 1 (+ replan) | `grace_check_fires_expired_case` | `d05db9001` + `74a78ca0e` | ~8h (inc. 3-cycle saga + replan) |
| 2 (+ fix) | `grace_check_escapes_case_when_sponsor_revoked_after_decided_at` | `ad37db02e` + `1550d296c` | ~3h |
| 3 | `grace_check_no_op_when_grace_expires_at_in_future` | `c70585c4b` | ~9 min |
| 4 | `grace_check_per_case_isolation_skips_bad_case_processes_good_case` | `bd8dc4a56` | ~8 min |
| 5 | `grace_check_batch_size_config_caps_iteration` | `03b05c019` | ~9 min |

**Phase-1 workspace-checks:** DQ #172, #173, #174 — all `result=pass`.
**Phase-2 e2e:** DQ #175 — 76 passed, 0 failed, 3 ignored (1541s, local laptop).
**PR:** #122 opened, CR reviewed, closed due to DQ conflict → cherry-pick merge to governance-v0.

---

## §2 Per-role signals

### Advisor

**Hits:**
- Brief override (`§2.1 CANONICAL CASE OVERRIDE`) successfully prevented Tasks 2-5 from repeating the Case B error that plagued Task 1.
- L18 workaround (advisor direct-mutate on stale ci-watcher base) handled correctly every cycle — 4 occurrences total.
- Phase-2 e2e local laptop run used correctly (user-selected, bat wrapper, `--workspace --features full`).
- Cherry-pick merge resolved the PR DIRTY conflict cleanly — all 6 test commits landed.

**Misses:**
- **DQ branch isolation:** Phase branch accumulated DQ commits that conflicted with governance-v0. Root cause: ci-watcher finalize-merges write DQ mutations to the phase branch. Future phases should route all DQ writes to governance-v0 only — or the bm-cut should configure the phase branch to exclude `.claude/decision-queue.json` from finalize-merge commits.
- **bm-merge worktree findings isolation:** BM task #180 read a stale `pr-122-findings.yaml` (pre-triage) from its worktree — the advisor's direct SSH edit wasn't propagated to the worktree. The correct fix is to update the findings file before queueing bm-merge, not mid-task.
- **PR DIRTY state discovered late** (at merge time). Earlier check of `mergeStateStatus` after opening PR would have surfaced it before waiting for CR review.

### Planning (impl-task subagent)

**Hits:**
- Tasks 3, 4, 5 dispatched and completed in 8-9 min each — clean execution once Case A shape established.
- All workers correctly read the brief override and used `LemmyResult<()>`.

**Misses:**
- Task 1 required 3 cycles + a replan before landing. Root cause: plan §13 stub prescribed `Box<dyn Error>` and the brief's §G4 verbatim recipe was paraphrased in the first two fix-impl briefs.
- Task 2 required a fix-impl to restore Case A after cherry-pick onto replan tip introduced a single regression.
- DQ id collision pattern recurred on Tasks 3 and 4 (workers computed next id from local view missing advisor-side entries).

### BM

**Hits:**
- bm-pr, bm-poll-cr completed cleanly.
- bm-triage brief correctly specified buckets — worker didn't follow it (kept `fix-in-pr`), but advisor caught and applied directly.

**Misses:**
- bm-triage #179 ignored the brief's bucket assignments and kept `recommendation: request-changes`. Brief was clear; worker mis-executed.
- bm-merge #180 correctly refused on stale findings YAML — but the stale-findings root cause was the advisor applying triage via direct SSH edit rather than via a committed file visible to the worktree.

### ci-watcher

**Hits:**
- ci-watcher #176 (DQ #174) successfully pushed to origin — no L18.
- All 3 Phase-1 workspace-checks resolved `result=pass` correctly.

**Misses:**
- L18 (stale EliteDesk governance-v0) recurred 3 times (ci-watchers for DQ #172, #173, Phase-2 e2e). Same root cause as prior phases — EliteDesk local `governance-v0` not fetched before branching. Not yet fixed at infra level.

---

## §3 Lessons (durable — promote to PMD)

1. **DQ commits on phase branch cause PR DIRTY conflicts.** ci-watcher finalize-merges write DQ mutations to the phase branch, which diverges from governance-v0's DQ. Mitigation: after Phase-2 e2e passes, do a quick `mergeStateStatus` check before opening the PR — surface early and rebase/cherry-pick before CR review wastes time.

2. **bm-triage findings update must be committed before bm-merge queued.** Direct SSH edits to `pr-NNN-findings.yaml` are invisible to BM worktrees. Always edit via a committed update (or the advisor applies the bucket changes to the file and commits before queueing bm-merge).

3. **Brief verbatim §G4 recipe + canonical-schema-first gate eliminated repeat cycles.** Tasks 3-5 (all using the pre-authored override brief) ran in <10 min each. Tasks 1-2 (brief vs plan stub conflict) cost ~11h total. The ratio validates the brief override discipline.

4. **L18 (stale EliteDesk governance-v0) is a systemic infra issue.** 4 occurrences across c-2. Fix: add `git fetch origin governance-v0 && git pull --ff-only origin governance-v0` to the ci-watcher agent's task-0 pre-flight before branching. Surface in next advisor-orchestrator revision.

---

## §4 Per-task complexity scores

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---|---|---|---|
| 1 (inc. replan + 2 fix cycles) | 1 | 3 | ~480 | ~30 |
| 2 (inc. fix) | 1 | 2 | ~60 | ~10 |
| 3 | 1 | 1 | ~9 | ~3 |
| 4 | 1 | 1 | ~8 | ~3 |
| 5 | 1 | 1 | ~9 | ~3 |

---

## §5 Watch-items for next phase

- [ ] **L18 fix:** add `git fetch + pull --ff-only governance-v0` to ci-watcher agent pre-flight.
- [ ] **PR DIRTY early-check:** add `mergeStateStatus` check after bm-pr, surface before queueing bm-poll-cr.
- [ ] **bm-triage committed findings:** establish convention that triage updates are committed to a tracked location (or findings file committed on governance-v0) before bm-merge brief is authored.
- [ ] **DQ id collision:** workers should compute next id including `decision-queue-archive-*.json` files — already in `dq-recipes.md` but workers still miss it.
- [ ] **PENDING collapse to single-session** (deferred from SL-b) — surface now that SL-c is shipped.
