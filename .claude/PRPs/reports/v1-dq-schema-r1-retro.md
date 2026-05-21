# Retro: v1-dq-schema-r1

**Sub-phase:** DQ schema-v3 (composite ids + approval audit trail)
**Author:** advisor session (laptop, `governance-v0`)
**Shipped:** 2026-05-22
**Trunk tip at retro:** `972247da4` (Task 3 commit; Task 4 retro = THIS commit)
**Source:** GitHub issue #142 (carry-forward from PR #141 CR-1 triage)

## What surprised us

1. **The plan landed at complexity 0/10 yet two of its four task units required ~40 min of advisor-side recovery work due to daemon failure modes outside the plan's scope.** Plan §5.1 scored complexity based on the diff shape (meta-only, no Rust, no migrations, no cross-crate impact). It did not — and structurally could not — score against the *infrastructure substrate the plan rode on*. DQ #338 (daemon finalize hard-resets trunk to wrong phase branch) cost ~25 min recovery on Task #399 and ~15 min on Task #401. A "complexity 0" plan that nonetheless cost ~40 min of out-of-band remediation deserves attention: future complexity scoring should consider a separate axis for "daemon-substrate reliability at queue time" when phase-v1-* branches are concurrently active.

2. **The fix for "DQ id collisions" produced its OWN collision DURING ship.** Cohort 1's worker-401 raised `kind: "validate-pending-laptop"` with id `339`. Concurrently the fed-in-c Task 3 worker raised an unrelated `validate-pending-laptop` also with id `339` — the exact bug class v1-dq-schema-r1 exists to eliminate. Both workers used the abolished `next_id = max(all_ids) + 1` recipe against their local DQ views. Documented in the manual-finalize merge commit body (`9f7ba68bb`). Composite-id `<session_id>-<seq>` (Task 1 ships) makes this arithmetically impossible going forward; the bug couldn't have picked a more emphatic exit moment.

3. **Live empirical validation revealed a hidden TypeError.** Pre-merge of Task 3, the live DQ contained 133 int ids + 1 str id (`ae5ff4ec3e8c-001`). The OLD sort key `key=lambda e: e['id']` throws `TypeError: '<' not supported between instances of 'str' and 'int'` on mixed corpora. Without Task 3, *any* invocation of `resolve-dq-canonical.sh` after Task 1's migration would crash. Task 3 was not optional polish — it was load-bearing for v3 readiness. Worth recording for future "is this 2-line patch necessary" judgment calls.

4. **The plan-commit became unreachable post-finalize.** Planning Junior #399 produced `b1a6553b8` (the 955-line plan). Daemon finalize-merged it into local governance-v0 as merge `2ad835aa4`, then immediately hard-reset to `origin/phase-v1-federation-inbound-c`. The plan-commit survived only via daemon reflog. Recovery via cherry-pick → recovery-branch push → laptop cherry-pick produced `c02dc8617` (daemon) → `f746a00b9` (laptop). Three SHAs, byte-identical trees. **Origin was protected only because the reset happened before push** — a single line reordering in `executor.ts` would have force-pushed governance-v0 to a phase branch's tip. The recovery recipe is documented in `feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md`; the structural fix awaits DQ #338 resolution.

## What to change

1. **Surface complexity-score reliability axis at plan-author time.** When `phase-v1-*` branches are active in the daemon worktree list and a planning task is queued from governance-v0, the planner should append a "Daemon substrate risk: HIGH (DQ #338 active)" note to the plan's complexity score. Or the advisor should preemptively use advisor-direct authoring for trivial plans (Option B path) when this risk holds. Either way, the silent assumption "Junior dispatch is reliable" needs to be surfaced.

2. **Document the manual-finalize recovery recipe in advisor-orchestrator.md.** Currently the recipe lives in `feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md` (created by user 2026-05-21). Promote a "Daemon-bug fallback: advisor-laptop manual finalize-merge" sub-section under §5.4 DQ triage decision tree so it's load-bearing path, not just a lesson file. Cohort 1 used it twice; cohort 2 (Task 3) skipped Junior entirely via Option B; both patterns will recur until DQ #338 ships.

3. **Resolve DQ #338 (option-a structural fix) in a near-term sub-phase.** Every planning task dispatched while ANY `phase-v1-*` branch is active reproduces the trigger conditions. Probability of recurrence: high. Recurrence cost: ~25 min recovery + ~15 min manual-finalize per task. The fix path is documented in the lesson file's "Investigation pointers" section (`/opt/junior-src/src/daemon/executor.ts` finalize-merge logic). A single-task sub-phase to patch + restart the daemon would pay for itself within 2-3 planning dispatches.

4. **`bm-runlog.md` merge=union policy reminder.** v1-dq-schema-r1 had no PR / no bm-cut / no bm-pr / no bm-merge — direct-commit policy — so no runlog conflict opportunity. But the pattern from v1-ship-1-r2 (L14 runlog-on-trunk self-conflicts with bm-pr) is relevant if future fix-up sub-phases for schema-v3 need PRs. Not a change for this retro, but worth carrying forward (next bullet).

## What to carry forward

1. **Composite id `<session_id>-<seq>` works as designed.** Validated empirically: two CC sessions (this advisor + concurrent fed-in-c session) cannot collide because their 12-hex prefixes don't intersect. The pre-reservation workaround documented in `feedback_cohort_dq_id_collision.md` is now superseded for v3 native entries (Task 2 marked the lesson's Status accordingly). Future cohort dispatches should rely on `dq-v3-new-entry.sh` and skip any "pre-reserve N ids" gymnastics.

2. **`approved_by` / `approved_at` fields are advisor-exclusive.** Task 2 added hard refusals #8 + #9 across all four `.claude/agents/*.md` contracts. Future judgment-heavy DQ user-gates (ADR-affecting, scope-changing, visible-to-others impact) should populate `approved_by` per CR-1's original recommendation. The advisor populates these fields **after** the user-gate AskUserQuestion relay; Junior subagents MUST NEVER populate them. CR-1's audit gap on PR #141 (resolver != approver) is structurally closed.

3. **Direct-commit policy on `governance-v0` for `.claude/` + `scripts/` meta-only.** This sub-phase shipped 5 commits (plan + 3 tasks + retro) directly on governance-v0 — no phase branch, no PR, no CodeRabbit review. Per `.claude/rules/phase-branch.md`, this is correct: CR is great at Rust + SQL, mediocre at advisor-prompt prose. The litmus test "would CR's review be net-noise on this diff?" held. Carry forward as the default for future meta-only sub-phases.

4. **Manual semantic merge of `.claude/decision-queue.json` is feasible but brittle.** Cohort 1's `--no-ff` finalize required two semantic merge conflict resolutions (Task 1 + Task 2 worker raises against the migrated v3 base). Programmatic Python merges via temporary `_tmp_merge_dq*.py` scripts produced correct results both times. Pattern: (a) preserve v3 base structure (migrated schema_version + id_v1), (b) move worker entries to resolved[] if validate-pending-laptop result resolved during recovery window, (c) preserve any concurrent-session entries (DQ #326, #338) verbatim. Documented here for future advisor-manual finalize cycles.

5. **Empirical pre-merge validation against the live DQ corpus is cheap and high-signal.** Before merging Task 3, ran the new sort key against the live 134-entry DQ in a Python REPL — discovered the OLD sort would crash on mixed-id corpora. The same recipe (run the new logic against the live state pre-merge, not just the plan's example) caught the empty-corpus blind spot earlier in Task 2's draft. Carry forward to any plan that touches a tooling script with state-dependent semantics.

## Per-role signals

### Advisor

**Calibration honesty:** ran two manual finalize-merge cycles (~15 min each); both correct first-pass via Python-script semantic merge. Did NOT silently retry on the daemon — instead surfaced both incidents (DQ #338 created with kind=blocker; recovery recipe authored). Falsifiable-hypothesis check held: when DQ #338 RCA was wrong (early hypothesis: daemon `reset --hard` in finalize code), pivoted within ~30 min once the concurrent fed-in-c session's `877bd849c` provided real evidence the trigger was `phase-v1-*` worktree state, not the executor's own reset semantics.

**Decisions made via user-gate:** plan approval (1 gate), DQ #338 cherry-pick recovery (1 gate), Task 3 Option A vs Option B (1 gate). Three gates, three relays, all converged within one round of AskUserQuestion. No silent self-attribution under "advisor" label.

**Cost:** estimated ~3h total wall-clock from issue #142 RCA to ship (across 2 sessions including ~40 min daemon-bug recovery).

**Memory hygiene:** committed every state-mutation step to git before context-compacting. The pre-compact handover file (`.claude/PRPs/handovers/v1-dq-schema-r1-cohort-2-handover-2026-05-22.md`) survived the compact and Resume-After-Compact picked up Task 3 cleanly with no context recovery needed.

### Planning

**Plan quality:** Junior #399 produced a 955-line plan in ~30 min after clarify-pass. Plan §0 RCA was complete (4 confirmed collisions cited with shas). §0.1 PRECON-1..5 BINDING section structurally prevented scope creep. §13 task-shape was machine-parseable enough that Task 3 was implementable from the FILES YAML + IMPLEMENT block alone — no plan revision needed.

**Complexity score:** 0/10 — accurate for the diff. See "What surprised us" §1 for the substrate-reliability blind spot. Score-axis improvement candidate for future retro.

**Clarify pass:** DQ #330-#336 (7 entries) resolved at `02b5a99ed` in advisor-mode. All 7 were self-answerable from lessons + ADRs — no user-relay needed for the planning brief. Pre-plan-write disambiguation paid off: zero post-plan revision rounds during impl.

### Impl

| Task | Files | Commits | Runtime (min) | Max-log-silence (min) | Notes |
|---|---|---|---|---|---|
| Task 1 [P] (Junior #400) | 4 new (2 scripts + .gitignore + migration touches 134 entries) | 1 feature + 1 DQ raise | ~12 min worker | n/a — daemon finalize wedged; advisor-laptop manual finalize | Worker pushed clean; DQ raise included |
| Task 2 [P] (Junior #401) | 8 modifications across rules/refs/lessons/agents | 1 feature + 1 DQ raise | ~18 min worker | ~21 min stale-running while daemon-HEAD wedged | Required manual `--no-ff` merge of both worker branches; two `.claude/decision-queue.json` semantic conflicts (worker-400 vs cf93b7ba6; worker-401 #339 raise vs v3 base) |
| Task 3 (advisor-direct) | 1 (resolve-dq-canonical.sh: +11/-2 lines) | 1 commit | ~5 min wall | n/a — advisor session | Option B per user gate; FILES YAML matched plan §13 exactly |

**Aggregate:** 3 tasks, 13 files touched, ~35 min impl wall-clock, ~40 min recovery overhead from daemon-bug DQ #338 (Tasks 1 + 2 cohort finalize). Net ~75 min from cohort dispatch to ship.

**Lesson injection compliance:** all impl-task briefs (impl-1, impl-2) cited §3 Required reading per mandatory file-class lesson table. Task 3 (advisor-direct) inherited the full plan §13 context including embedded MIRROR refs.

**Empirical validation:** every DoD probe ran against the live state (not just the plan's example). Caught the str/int mixed-corpus TypeError that the plan's empty-corpus example would not have surfaced.

### BM

N/A this phase (no PR / no bm-cut / no bm-pr / no bm-merge; direct-commit on governance-v0 per `.claude/rules/phase-branch.md` "Direct on governance-v0" set). Sub-phase touched only `.claude/`, `scripts/brehon/`, and `.gitignore` — all members of the meta-only file set where CodeRabbit review is net-noise. The next BM activity will be at v1-federation-inbound-c (or whichever next code-touching phase ships).

## Lessons promoted this phase

(none authored in this retro commit)

Pre-existing lessons updated by Task 2 of this sub-phase:

- `.claude/lessons/feedback_cohort_dq_id_collision.md` — `## Status (post-v1-dq-schema-r1, 2026-05-21)` § appended noting the pre-reservation workaround is superseded for v3 native entries.

Pre-existing lessons authored by user during this sub-phase (concurrent session):

- `.claude/lessons/feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md` — created at `cf93b7ba6`. Companion to DQ #338 (still pending; awaits option-a structural fix).

## Open follow-up

- **DQ #338** remains pending. Recommendation logged in `cf93b7ba6` body. Next sub-phase planning should either (a) consume #338 as its scope (single-task patch + daemon restart), or (b) acknowledge the recurrence cost in its own substrate-risk section.
- **GitHub issue #142** can be closed when v1-dq-schema-r1 ships; this retro commit + the migration + Task 3 sort fix completes the requested scope.
