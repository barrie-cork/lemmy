# Advisor handover — 2026-05-03 — sl-a-cohort-a-merged-ci-watchers-running

**Written:** 2026-05-03T11:08Z
**Author:** advisor session (`C:\Users\barri\Developer\brehon-fork`)
**Branch:** `governance-v0` @ `eb7f183a9` (`chore(advisor): briefs sl-a-ci-watcher-1..6 — cohort A 6 workflow runs (v1-SL-a)`)
**Purpose:** Self-contained brief for next-session resume after cohort A 5-way DQ #118 collision was recovered + 6 ci-watchers dispatched.

## TL;DR

- **This session shipped:** Cohort A (Tasks 1+2+3+6+7) dispatched 5-way [P]; hit 4-way DQ #118 read-modify-write collision (T2/T3/T6/T7); T1 self-corrected; daemon raced ahead during scheduled wakeup gap and finalize-merged all 5 with renumbered DQ ids referenced in merge commit subjects (T3=#118/T2=#119/T7=#120/T6=#121/T1mig=#122/T1ws=#123); advisor accepted daemon's mapping (Option AA) + added analytical commit; force-pushed origin to align; cherry-picked DQ #124 (kind:log retro lesson) + 6 ci-watcher briefs onto phase-v1-SL-a; queued #89-#94 ci-watchers in parallel.
- **Pending:** 6 ci-watchers running (poll wf 25275388975/25275426712/25275487098/25275653498/25275688423/25275688427); 6 DQ `kind: validate-pending` entries on phase-v1-SL-a tip awaiting mutation; phase-v1-SL-a tip `094cf58e2` on origin.
- **Blocked:** nothing; ci-watchers are autonomous and Junior-managed.
- **Next session must:** poll #89-#94 transitions; on all-resolved, surface DQ pass/fail summary to user + ask whether to proceed with cohort B (Tasks 4+5). Per Option B (DQ #117): cohort A workspace-check fails are PLANNER-INTENTIONAL; advisor §G4 classifier holds without auto-fix. Only T1 migration-check (#122) could legitimately pass standalone.
- **External gate:** Phase 2 e2e (workflow-dispatch-only) is post-cohort-B finalize; user picks local-vs-dispatch then.

## Cold-resume sequence

1. Read `CLAUDE.md` + `.claude/rules/*.md` (auto-loaded in `-p` mode; manually in interactive: `advisor-orchestrator.md` is the orchestrator law, `branch-manager.md`, `decision-queue.md`, `phase-branch.md`).
2. Read this file in full.
3. Read `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` §13 lines 1590-1730 (Tasks 4+5 — cohort B) to be ready for next dispatch wave.
4. Read `.claude/decision-queue.json` on `phase-v1-SL-a` tip (cross-branch — not local DQ): `git show origin/phase-v1-SL-a:.claude/decision-queue.json`. Confirm 6 pending entries OR mutation outcomes if ci-watchers landed.
5. Read `.claude/runlog/v1-SL-a-runlog.md` tail for collision-incident context + `.claude/runlog/bm-runlog.md` last 5 entries.
6. Verify state:
   ```bash
   git fetch origin
   git rev-parse HEAD                               # expect eb7f183a9
   git status --short                               # expect M retro-check.sh + 2 untracked (NOT mine)
   git rev-parse origin/governance-v0               # expect eb7f183a9 (synced)
   git rev-parse origin/phase-v1-SL-a               # expect 094cf58e2 (or further if ci-watchers mutated)
   gh pr list --repo barrie-cork/lemmy --state open # expect #108 + #110 (#109 closed/superseded)
   ssh homeserver 'cd /srv/brehon-fork && git branch --show-current'  # expect phase-v1-SL-a
   ```
7. Check Junior task state: `mcp__junior-brehon__list_tasks status=running` and `status=done` for ids 89-94. If all done, read each ci-watcher's branch DQ via `git show origin/junior/role-ci-watcher-...:.claude/decision-queue.json` to capture each entry's mutated `result`/`log_slice`/`failed_jobs`.
8. Append cold-resume event to runlog:
   ```
   2026-05-0?T??:??Z | advisor | meta | cold-resume | handover=advisor-2026-05-03-sl-a-cohort-a-merged-ci-watchers-running.md drift=<none|<detail>>
   ```

## State at handover

### Git

- **Laptop branch:** `governance-v0` @ `eb7f183a9` (synced with origin).
- **Phase branch on origin:** `phase-v1-SL-a` @ `094cf58e2` (cohort A merged + recovery commit + 6 ci-watcher briefs cherry-picked).
- **EliteDesk branch:** `phase-v1-SL-a` @ `094cf58e2` (in sync).
- **Working tree (laptop):**
  - `M .claude/hooks/retro-check.sh` — **NOT mine.** User-authored edit (widens retro-check window from 15→60 min on non-junior branches). Survived stash-pop after recovery.
  - `?? .claude/PRPs/plans/tier-3-pg-dump-restore-template.plan.md` — untracked, not mine.
  - `?? .claude/PRPs/reviews/pr-107-redflag-ack.md` — untracked, not mine.
- **Unpushed:** none on `governance-v0` or `phase-v1-SL-a`.

### Worktrees

```
C:/Users/barri/Developer/brehon-fork         eb7f183a9 [governance-v0]
C:/Users/barri/Developer/brehon-fork-tooling 7b785d46a [tooling-local-validation]
```

`tooling-local-validation` is user's PR #110 source (advanced from prior handover's `3a233c077`); not relevant to SL-a impl flow.

### Open PRs

| # | Title | Branch | Base | mergeState | review |
|---|---|---|---|---|---|
| 110 | feat(local-tooling): tier 1 rust-lld + tier 2 nextest + tier 3 pg_dump template | `tooling-local-validation` | `governance-v0` | UNKNOWN | (none) |
| 108 | fix: clippy::map_err_ignore on accept_jury_assignment (JM-e carry-forward) | `chore/clippy-map-err-ignore-jm-e` | `governance-v0` | CLEAN | (none) |

#109 (rust-lld, prior handover) was superseded by #110 (3-tier consolidation). Neither on SL-a critical path. Don't touch.

### Decision queue

- **Pending on `governance-v0` (laptop branch):** 0
- **Pending on `phase-v1-SL-a` (the live cohort A entries):** 6
  - #118 `validate-pending` phase_task=3 wf=25275388975 (T3 workspace-check)
  - #119 `validate-pending` phase_task=2 wf=25275426712 (T2 workspace-check)
  - #120 `validate-pending` phase_task=7 wf=25275487098 (T7 workspace-check)
  - #121 `validate-pending` phase_task=6 wf=25275653498 (T6 workspace-check)
  - #122 `validate-pending` phase_task=1 wf=25275688423 (T1 migration-check)
  - #123 `validate-pending` phase_task=1 wf=25275688427 (T1 workspace-check)
- **Resolved this session:**
  - #117 (advisor `kind: log`) — cohort-fail-expected rule mismatch (Option B chosen for in-flight SL-a)
  - #124 (advisor `kind: log`) — cohort-id-collision lesson + recommendation (A) for retro: pre-allocate id ranges in [P] briefs
- **Total resolved across both branches:** 115 (governance-v0 view) + 6 pending on phase branch.

### Junior tasks (this session — sequential timeline)

| # | Role | Status | Notes |
|---|---|---|---|
| 84 | impl-task | done | T1 migration; self-corrected #121+#122 (pre-recovery); now mapped to #123(ws) + #122(mig) |
| 85 | impl-task | done | T2 enums.rs; #118 collision; now #119 |
| 86 | impl-task | done | T3 schema.rs; #118 collision; now #118 |
| 87 | impl-task | done | T6 config.rs; #118 collision; now #121 |
| 88 | impl-task | done | T7 governance_log; #118 collision; now #120 |
| 89 | ci-watcher | running | Polls wf=25275388975 → DQ #118 (T3) |
| 90 | ci-watcher | running | Polls wf=25275426712 → DQ #119 (T2) |
| 91 | ci-watcher | running | Polls wf=25275487098 → DQ #120 (T7) |
| 92 | ci-watcher | running | Polls wf=25275653498 → DQ #121 (T6) |
| 93 | ci-watcher | running | Polls wf=25275688423 → DQ #122 (T1 mig) |
| 94 | ci-watcher | running | Polls wf=25275688427 → DQ #123 (T1 ws) |

Ci-watchers were queued at ~10:00Z. Workflows fired ~09:25-09:42Z. ci-watchers should resolve within 1-2 min each (workflow has terminated; long-poll returns immediately).

### Plan / PRD / ADR in-flight

- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — committed (governance-v0 + phase-v1-SL-a both have it). Cohort A done; cohort B (Tasks 4+5) is the next §13 wave.
- No PRD or ADR edits in flight.
- `.claude/PRPs/plans/tier-3-pg-dump-restore-template.plan.md` — untracked, user's work, not mine.

### Relays

- **Advisor → impl awaiting impl response:** none (no relay files written this session — DQ #117 + #124 + briefs were the coordination surface).
- **Impl → advisor awaiting advisor answer:** none.

## What's pending (ordered by priority)

### 1. Wait for 6 ci-watchers (#89-#94) to resolve, then triage

**Context:** Each ci-watcher polls one cohort A workflow run and mutates the paired `validate-pending` DQ entry in place per option 2 (single-entry mutation). The phase-v1-SL-a `decision-queue.json` will have 6 entries either passed (`pending[]` → `resolved[]`) or failed (`result: "fail"`, stays in `pending[]`).

**Per Option B (DQ #117 — user-confirmed):**

- T2/T3/T6/T7 workspace-check fails ARE expected (no exhaustive-match handlers, no ModerationCase struct extension code yet — Tasks 4+5 cohort B fixes both). Advisor §G4 classifier MUST NOT auto-queue fix-impl-tasks for these.
- T1 migration-check (#122) and T1 workspace-check (#123) might pass standalone (T1 is SQL + migration only). If T1 passes, that's a good signal; if it fails, advisor §G4 classifier should classify per the standard allowlist.

**Polling check:**

```bash
# Check task state
mcp__junior-brehon__list_tasks status=running       # expect [] when all 6 done
mcp__junior-brehon__list_tasks status=done          # expect 89-94 in there

# Check DQ state on phase-v1-SL-a
git fetch origin --quiet
git show origin/phase-v1-SL-a:.claude/decision-queue.json | python3 -c "
import json, sys
d = json.load(sys.stdin)
print('pending:', len(d['pending']))
for e in d['pending']:
    print(f'  #{e[\"id\"]} result={e.get(\"result\")} phase_task={e[\"phase_task\"]} wf={e[\"workflow_run_id\"]}')
print('resolved tail:')
for e in d['resolved'][:5]:
    print(f'  #{e[\"id\"]} result={e.get(\"result\")} answered_by={e.get(\"answered_by\")}')
"
```

**Surface to user when all 6 transitioned:** pass/fail summary table + recommendation. If pattern matches Option B expectation (T2/T3/T6/T7 fail with non-exhaustive-match/missing-field, T1 passes), recommend proceeding to cohort B without §G4 auto-fix.

### 2. Author + queue Cohort B (Tasks 4+5, 2-way [P])

**Context:** Plan §13 line 1240 declares cohort B = Tasks 4+5 (2-way [P]). YAML disjointness must be checked first (cohort dispatch sequence step 4):

- Task 4 modifies `crates/db_schema/src/source/governance/moderation_case.rs` (`ModerationCase` struct + InsertForm). Lines 1590-1646.
- Task 5 modifies 6 ADR-013 enum-exhaustiveness match sites across `crates/api/api/src/governance/...` and similar. Lines 1647-1730.

These should be disjoint (Task 4 = `db_schema`, Task 5 = api match sites). Run the FILES YAML overlap check before queueing.

**Pre-allocate DQ id range** per DQ #124 retro recommendation (forward guard): cohort B = 2 tasks × 1 validate-pending each = 2 DQ entries. Pre-allocate ids 125 + 126; brief constraints carry "Use DQ id 125 (Task 4)" and "Use DQ id 126 (Task 5)" at brief-write time. **This is the new pattern** — first cohort to use it.

**Brief authoring:**

- `.claude/PRPs/briefs/sl-a-impl-4.md` (Task 4)
- `.claude/PRPs/briefs/sl-a-impl-5.md` (Task 5)

Mirror sl-a-impl-1.md template. §3a "Handover from prior cohort" aggregates the 5 cohort A `HANDOVER:` YAML trailers from each merged source-code commit (per `feedback_handover_trailer_cohort_propagation.md`). Note: cohort A merge commit subjects already include the daemon-renumbered DQ ids; don't re-extract them — read the source-code commits' bodies for the `HANDOVER:` YAML trailers.

**Dispatch:**

1. Commit both briefs on `governance-v0` in one push.
2. Cherry-pick onto `phase-v1-SL-a`.
3. Push `phase-v1-SL-a`.
4. Run `/precheck` to confirm EliteDesk readiness.
5. Queue both tasks in parallel.

**Cohort B is the green-gate per plan §14 Story 1.** When Task 5's push triggers workspace-check, the resulting `conclusion: "success"` (assuming Tasks 4+5 land their respective fixes) closes the cohort A workspace-check failures retroactively (the failures were dependency on cohort B). That ci-watcher run is the first cohort A barrier release.

### 3. Track 4 retro-harvest items for SL-a retro

(prior 3 from previous handover + 1 new from this session)

**(a) bm-cut Junior over-pushed phase-v1-SL-a** (from prior handover — daemon-script-deviation; investigate at retro).
**(b) Settings.json model-field check is stale** (from prior handover — bm-cut briefs no longer have a real field to check).
**(c) Brief-on-phase-branch cherry-pick step undocumented** (from prior handover — should be documented in `advisor-orchestrator.md` "Stage shape" section).
**(d) NEW — Cohort A 5-way DQ #118 collision** — see DQ #124 + `.claude/runlog/v1-SL-a-runlog.md`. Recovery via Option AA (accepted daemon's chronological mapping). Forward guard recommendation = pre-allocate id ranges in cohort briefs.

### 4. Address open carry-forward PRs (still advisor-gate unclear)

PR #110 (3-tier tooling) and #108 (clippy carry-forward) — neither on SL-a critical path. Defer until SL-a ships. **Note PR #109 was superseded by #110 mid-session** — drift since prior handover.

## What NOT to touch

Per `.claude/rules/branch-manager.md` and `.claude/rules/handover.md`:

- `crates/**`, `migrations/**`, `tests/**` — impl-owned. Cohort B impl-tasks are how these get touched.
- Active plan file `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — committed and approved; do not edit unless a planner DQ raises a defect.
- `.claude/hooks/retro-check.sh` — modified in working tree by user (not advisor). Leave as-is; survives stash/pop.
- `.claude/PRPs/plans/tier-3-pg-dump-restore-template.plan.md` — untracked, user's work, not mine.
- `.claude/PRPs/reviews/pr-107-redflag-ack.md` — user's in-progress review, not mine.
- PRs #108 + #110 — carry-forward / user's tooling, not SL-a.
- The 5 worker branches still on origin (`junior/role-impl-task-v1-sl-a-task-{1,2,3,6,7}-...-{84,85,86,87,88}`). They share `15b8fc300` ancestor; preserved as audit trail. Can be deleted post-cohort-A close, NOT before.

## Key new rules / patterns established this session

These belong in the SL-a retro for promotion to lessons + rules:

1. **Cohort dispatch DQ id pre-allocation (DQ #124).** When dispatching N parallel `[P]` tasks that will write `validate-pending` entries, advisor pre-allocates the next N free ids and assigns each to a specific task in the brief constraints. Prevents read-modify-write race. Cohort B (Tasks 4+5) is the first to use the new pattern.

2. **Cohort-fail-expected pattern (DQ #117).** When a plan's §14 Story 1 names a NON-COHORT-A push as the green-gate (e.g. "Task 5's push" rather than each cohort A task's push), the advisor MUST tolerate `result: "fail"` mutations on cohort A `validate-pending` entries without §G4-auto-fix-classification. Plan template should declare this pattern explicitly per-cohort.

3. **Recovery from ID collision = accept daemon's chronological mapping.** If a multi-worker DQ id collision happens, the daemon's finalize-merge-time chronological renumbering (visible in merge commit subjects) produces a valid mapping that preserves all workflow_run_ids. Advisor adds an analytical commit on top + retro lesson; force-push acceptable as it's lossless on source code.

## Notes on the four-role mechanical model (this session's reinforcement)

(prior session's 4 notes still apply — not repeating here. New observation:)

- **Junior daemon's finalize-merge stage handles DQ id collisions transparently** by renumbering during merge-commit construction. The daemon's merge subject names the assigned id. This is robust but loses the worker-side audit trail (worker's commit body still references the pre-merge id). For SL-a this is acceptable; long-term the pre-allocation pattern (DQ #124) eliminates the renumbering altogether.

## Bootstrap prompt (paste into next session)

```
I'm resuming advisor work on the Brehon governance fork. Previous session
wrote a handover at `.claude/PRPs/handovers/advisor-2026-05-03-sl-a-cohort-a-merged-ci-watchers-running.md`.

First steps:
1. Read CLAUDE.md + .claude/rules/*.md (auto-loaded in -p mode).
2. Read `.claude/PRPs/handovers/advisor-2026-05-03-sl-a-cohort-a-merged-ci-watchers-running.md` in full.
3. Execute its "Cold-resume sequence" in order.
4. Report the TL;DR summary plus any state drift you detected back to me.

Do not take any state-changing actions until I confirm.
```

## Closing state assertions (verify on resume)

- `git branch --show-current` → `governance-v0`
- `git rev-parse HEAD` → `eb7f183a9378e3dd384e8d2e7360da77feba3d32`
- `git rev-parse origin/governance-v0` → `eb7f183a9378e3dd384e8d2e7360da77feba3d32` (synced)
- `git rev-parse origin/phase-v1-SL-a` → `094cf58e202377e385e0e1731545aaed383a88b2` OR newer (if ci-watchers mutated)
- `git status --short` → `M .claude/hooks/retro-check.sh` + 2 untracked
- `.claude/decision-queue.json` (governance-v0) pending count → 0
- `git show origin/phase-v1-SL-a:.claude/decision-queue.json` pending count → 6 OR fewer (as ci-watchers mutate to pass)
- Junior tasks #84-#88 all `done`; #89-#94 `running` or `done`
- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` exists on `governance-v0` (2541 lines)
- Active handover file exists: `test -f .claude/PRPs/handovers/advisor-2026-05-03-sl-a-cohort-a-merged-ci-watchers-running.md`
