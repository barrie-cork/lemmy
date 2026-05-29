# v1-quality-r2a retro

> Phase tip at retro authorship: `47dcc756c` (feat(quality): merge v1-quality-r2a task 2 — C3 deferral DQ).
> Plan: `.claude/PRPs/plans/v1-quality-r2a.plan.md` (committed `8ee61efca` after r2 split per user gate-1 decision).
> PR: not yet opened — bm-pr → CR cycle is the next stage after this retro.
> Authored: 2026-05-29 by advisor session running in Mode B (mobile remote-control from canonical `brehon-fork` checkout).
> Phase wall-clock: 4 Junior tasks (#490 + #492 + #494 + the resolved DQ #3ef987b66db4-001 amendment cycle) over ~2 days advisor-clock; total active impl-task wall time ≤8 min (3 Junior runs, longest 3m 58s).
> §16a stories: r2a was a sub-split of the original r2 plan (user gate-1 chose "split into r2a (T1+T2) + r2b (T3+T4+T5)"), so the full r2 §16a story list does not apply; the r2a-scoped acceptance is "Task 1 + Task 2 ship on phase tip + r2a-retro authored".
> Phase 2 e2e: **not run for r2a** — plan §15.4 explicitly states r2a has no e2e gate (zero Rust changes; T1 ships bash scripts under `scripts/brehon/`, T2 ships a `kind: log` DQ entry).

---

## Advisor signals

The advisor operated this lane in **Mode B (mobile remote-control)** throughout: canonical `brehon-fork` checkout on `governance-v0`, every impl-task dispatched to Junior with `base_branch=phase-v1-quality-r2`, briefs authored on canonical + Mode B sync'd via daemon SSH-merge per `multi-lane-worktree.md` §"Brief location and trunk→phase sync". No lane worktree existed for this phase. This was correct: r2a touches zero Rust, no cargo gates run, no validate-pending-laptop entries — Mode B's only Mode-A advantage (local cargo via lane worktree) was not relevant.

Gates honoured: clarify (no clarify-DQ fired; brief shape was clean), plan approval (gate-1 with the r2-split decision and the §13 sweep-id amendment per DQ #3ef987b66db4-001), DoD smoke test (passed pre-dispatch), watchpoint specificity (passed — all watchpoints cite specific entry-ids or script paths), Phase 2 e2e local-vs-dispatch (n/a — no e2e gate), merge confirm (pending — next stage after retro sign-off).

One process pattern worth flagging: gate-1 produced a **plan-id mismatch blocker** (DQ #3ef987b66db4-001 — Task 0 worker's probe-6 entry-id enumeration found 3 entries the original r2 plan §13 Task 1 named at 4 ids). The advisor resolved this via "option-A floor" (per `feedback_falsifiable_hypothesis_before_structural_fix.md`): amend the plan §13 ids to match worker-discovered truth rather than re-investigate. Two-commit attribution-pattern compliance enforced (one `docs(advisor): amend` commit + one `chore(decision-queue): advisor answered` commit) so both subjects match `^(chore|docs)\((advisor|decision-queue)\)` per `decision-queue.md` Attribution integrity §Detection.

---

## What surprised us

**The §13 Task 1 plan-id mismatch was caught by Task 0 pre-flight, not by the advisor's DoD smoke test.** The smoke test ran `bash scripts/brehon/dq-lint-durations.sh` (didn't exist yet) which exited "command not found" — interpreted as "Task 1 will create this". The actual mismatch class (the original r2 plan had named 4 entry-ids; only 3 matched the live `decision-queue.json`) only surfaced when Junior #490 enumerated probe-6 and raised the blocker. Lesson: pre-flight harness audits that **enumerate** something the plan **names** catch this class earlier than the advisor's DoD smoke (which only checks command runnability, not data shape).

**The C3 `kind: log` DQ entry shipped with a benign by-product commit.** Junior #494 was scoped to "write a fragment, run the helper". The brief said the fragment file `.claude/PRPs/debug/v1-quality-r2a-c3-defer.json` was gitignored. It is NOT — only `.claude/PRPs/debug/*.log` is gitignored. The worker correctly noticed the file was tracked and committed it as `chore(advisor): add C3 deferral debug fragment for v1-quality-r2a task 2` (commit `758bce3a1`). The fragment is a 16-line throwaway with no downstream consumers, so it's harmless on the phase branch. Lesson for future briefs: **verify gitignore claims with `git check-ignore -v <path>` before stating them.** The fragment commit subject's `chore(advisor):` prefix was technically inaccurate too (the worker is `bm-task` per author, not advisor), but the commit is byte-immutable now; documenting it here.

**Two-commit attribution-pattern compliance enforced as a session ritual.** The DQ #3ef987b66db4-001 plan-id-mismatch resolution required two commits (plan amendment + DQ answer) so both subjects matched the advisor commit-subject pattern. This was authored as a single advisor turn (read DQ → resolve via option-A → amend plan §13 → answer DQ → commit each separately → push). Without the ritual, a single `chore(advisor): amend plan + answer DQ` subject would have been a process-breach detection trigger. Worth promoting as a lesson if recurrence ≥3.

---

## What to change

**Pre-flight harness audits should enumerate every entry-id the plan §13 names.** Task 0's brief said "Probe 6: enumerate the sweep-target entry-ids from §13 Task 1". This is the right shape. Future Quality-class plans (r2b et seq.) should structurally require a Task 0 probe per `§13 Task N names entry-ids` rule. Add to the planning template's §16a "pre-flight" story shape.

**Brief gitignore claims need a `git check-ignore -v` proof line.** The Q-r2a impl-2 brief said `.claude/PRPs/debug/v1-quality-r2a-c3-defer.json` was gitignored. It wasn't. The brief author can verify in one shell call. Future briefs that name a gitignored artifact should cite the `.gitignore` line OR run the proof at brief-author time. Add to the impl-task brief template §3 Required reading as a sub-step.

**Mode B Brief sync ergonomics are good but the trunk→phase merge subject is verbose.** `Merge governance-v0 into phase-v1-quality-r2 — pull r2a impl-2 brief for Task 2 dispatch` is fine prose but long-line and noisy in `git log --oneline`. The current shape is documented in `multi-lane-worktree.md` §"Brief location and trunk→phase sync". Not changing this phase; flagging as a quality-of-life polish for the next time the spec is touched.

---

## What to carry forward

**The dq-lint-durations.sh + precheck.sh harness is durable infrastructure.** Task 1 shipped 29 LOC of `dq-lint-durations.sh` (composite-id-aware, exits 1 on bad entries) + 8 LOC of `precheck.sh` wrapper. Both ship on `governance-v0` after this PR merges. The lint will fire on every `/precheck` call going forward and on every advisor session-start. The 3 swept entries (id=315 `08:53Z → 10:45:00Z`; id=1b8527b076d4-001 `19:11:20.705989Z → 19:30:00Z`; id=81719cf8ca8d-001 `07:08:50.884428Z → 19:10:00Z`) close Issue #157.

**The C3 deferral DQ entry (`dd6012873857-001`) is a durable carry-forward signal.** Task 2 shipped exactly one new `kind: log + from: planner + answered_by: planner` entry in `resolved[]`. The trigger condition for v1-quality-r3 re-entry: any sub-phase introduces a 3rd reputation-event emit path. Two existing emitters live at `crates/api/api/src/governance/admin_emergency_remove.rs:448` (`emit_reputation_event_local`) and `crates/api/api/src/governance/submit_jury_vote.rs:1096` (`emit_reputation_event`); bodies are byte-identical. Issue #158 stays OPEN with a BM-filed deferral comment at merge time so v1-quality-r3 has a re-entry point.

**The r2b carry-forward lane is pre-scoped.** Task 29 in the session tracker (Lane Q-r2b: schedule separately after r2a ships) is pending. r2b targets C1 (#156) + C4 (#159 + #160) covering T3+T4+T5 from the original r2 plan. Next session that's not already lane-busy should bm-cut `phase-v1-quality-r2b` from then-current trunk + author a narrow planning brief.

**Option-A floor for plan-vs-truth mismatches.** DQ #3ef987b66db4-001's resolution pattern (amend the plan to match worker truth rather than re-investigate the divergence) was the right call here because the mismatch was a stale plan citation, not a divergent design intent. Generalisable rule: when a worker's pre-flight discovers the plan §13 named N entries and the live system has N±k entries, the advisor amends §13 unless §10.x has structural prose contradicting the new count. Per `feedback_falsifiable_hypothesis_before_structural_fix.md`.

---

## Per-role signals

### Advisor
Operated cleanly in Mode B. Six gates honoured (clarify, plan approval, DoD smoke, watchpoint specificity, e2e n/a, merge pending). One judgment call surfaced to user (gate-1 split of r2 into r2a + r2b); user authorised. One blocker DQ resolved with two-commit attribution-pattern compliance (one of the more disciplined cycles this phase). No catch-fires. Lane-mode discipline correct throughout — never wrote phase-branch DQ entries from canonical, used SSH-merge for trunk→phase sync, daemon-side push verification on every Junior completion (caught the daemon-false-success class on both T1 and T2 finalize).

### Planning
The original v1-quality-r2 plan was authored by a prior session. The r2a narrow-scope plan was advisor-authored at split time. Plan §13 Task 1's sweep-id list was stale — three of the four named entries existed; the fourth had been moved to archive. The plan's §10.3 deferral fragment for Task 2 was verbatim-usable (the worker copy-pasted with one `<ISO8601>` substitution). The §15 validation commands required no per-task cargo gates beyond Task 0, which was correct for an r2a scope of "zero Rust".

### Impl
Two Junior dispatches (#492 + #494; plus #490 for Task 0). All three ran short (≤4 min wall-clock). Task 0 produced one DQ blocker (plan-id mismatch, resolved option-A). Task 1 produced exactly the expected deliverables (29 LOC lint script + 8 LOC precheck + 3 swept entries closing #157). Task 2 produced one new `kind: log` DQ entry (plus a benign debug fragment commit; see "What surprised us" above). All commits carried `HANDOVER:` trailers per the cohort propagation lesson. Worker self-test gates exit 0 on every run.

### BM
bm-cut, bm-push for Task 0 ran in the prior r2 lane (commit `d5f0114eb`). bm-pr → bm-poll-cr → bm-triage → bm-merge are the next-stage tasks AFTER this retro signs off. No BM verbs ran during r2a tasks themselves (r2a's Junior dispatches were impl-task only). The benign `chore(advisor):` mis-attribution on commit `758bce3a1` (technically a bm-task author) is the only attribution polish to track; not worth a follow-up commit.

---

## Complexity / effort

| Stage | Files changed | Commits | Runtime approx |
|---|---|---|---|
| Plan split (r2 → r2a narrow) | 1 (plan) | 1 (`8ee61efca`) | ~10 min advisor |
| Task 0 (pre-flight + DQ #3ef987b66db4-001) | 1 (DQ entry) | 2 (worker + advisor amend cycle) | ~5 min Junior + ~10 min advisor |
| Task 1 (lint + sweep) | 2 scripts + 1 DQ sweep | 1 worker + 1 daemon merge | 3m 58s Junior + ~3 min advisor verify |
| Task 2 (C3 deferral DQ) | 1 DQ + 1 fragment by-product | 2 worker + 1 daemon merge | ~3 min Junior + ~3 min advisor verify |
| Retro (this file) | 1 retro | 1 | ~15 min advisor |

Total impl-task wall: ≤8 min cumulative Junior runtime. Total advisor wall (across 2 days clock): ~60 min active work (split decision + brief authorship + verifies + retro). No e2e. No cargo gates beyond Task 0 static analysis.

Cohort sizes: every task was non-`[P]` (serial). No parallelism within r2a — appropriate for the scale.

---

## Lessons promoted this phase

- None promoted to `.claude/lessons/` this phase. The "gitignore claim verification" finding (Surprise #2) and the "two-commit attribution ritual" (Surprise #3) are both 1× recurrences — watch-for-third before promoting.

## Watch-items for next sub-phase (r2b or any quality-class follow-on)

1. **Pre-flight harness probe per plan-§13-named-entry-id list.** Catches the plan-id-mismatch class earlier than DoD smoke. Promote to planning template §16a if reproduced in r2b.
2. **Gitignore claims in briefs need a verification line.** Add to impl-task brief template if reproduced.
3. **Two-commit attribution ritual on plan-amendment + DQ-answer coupling.** Promote to advisor-orchestrator.md §5.4 if reproduced.
