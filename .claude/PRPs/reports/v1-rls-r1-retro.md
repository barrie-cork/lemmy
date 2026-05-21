# v1-rls-r1 retrospective

**Phase:** v1-rls-r1 (Recursive Learning System hardening Wave 1)
**Plan:** `.claude/PRPs/plans/v1-rls-r1.plan.md` (committed `f61e09792`)
**Tip:** `3efa40b23`
**Date range:** 2026-05-20 (plan author) → 2026-05-21 (Tasks 1-13 ship)
**Tasks shipped:** 12 of 13 (Task 13 = this retro)

## What surprised us

### Advisor

- **Cross-cohort DQ id collision was unavoidable under §4.1 cohort dispatch.** Five workers forked from `phase-v1-rls-r1 @ 58496b4da` simultaneously; each computed `next_id = max(all_ids) + 1` against its isolated worktree view; each picked `307`. The cohort-dispatch protocol's FILES YAML disjointness check catches file-write collisions but DQ ids are a **global cross-worker resource** that no pairwise check addresses. Recovery cost ~30 min (cherry-pick 5 feat commits + consolidated cargo-check + DQ renumber commit + final mutate). Documented in `feedback_cohort_dq_id_collision.md` (shipped this phase) with three mitigations; structural fix (advisor pre-reserves N ids pre-dispatch) deferred to a later sub-phase.

- **Junior worker "hang post-DQ-raise" became a 2-of-3 pattern.** Workers #380 (Task 6) and #381 (Task 8) both authored their file outputs, raised their DQ entries, committed, pushed to origin — then hung at ~0% CPU for 10+ minutes with no log output and no cargo invocation. Cancelling + advisor FF-merge + lane-side cargo became the recovery template. The hang is silent (`Sl` state, low CPU, no stderr) so it can only be caught by liveness probing (`feedback_junior_task_liveness_check.md`). By Task 9 the cost-benefit shifted decisively in favour of advisor-authoring the remaining meta-work tasks.

- **Cross-lane daemon-ref contamination recurred 3+ times.** The daemon's local `phase-v1-rls-r1` branch ref was repeatedly reset to `phase-brehon-conformance-audit` work because the daemon's single-checkout model swaps refs when concurrent Junior workers dispatch from different `base_branch` values. The most dangerous instance: worker #383 forked from contaminated daemon-local ref `f9eb94969` (conformance-audit work) before commit; advisor caught it via `git log --oneline` pre-dispatch and cancelled. This is the **§292 stale-base recipe** but inverted — daemon-local is the polluted source, not just stale. The mitigation (`git checkout --detach HEAD && git branch -f && git checkout`) works but must run on every dispatch when another lane is concurrent.

### Planning

- **Plan §13's pre-assignment of DQ ids per-task (after the Cohort A collision)** worked end-to-end for serial-chain tasks (Task 5 worker correctly used #312; Task 6 worker used #313; Task 8 worker correctly adapted to #315 after #314 was consumed by Task 6's PMD-fallback retro). The recipe IS sound for serial dispatch; only the cohort-simultaneous case breaks it.

- **§13 Task 7's function-after-exit placement was not caught by Task 7's own VALIDATE** — that check only verified `grep -c "emit_retro_bypass_log" retro-check.sh = 3`, which the bug-shipped file passes (function definition + call site = 2; comment block = 1). Task 10's dogfood was the test that caught it. **Validation specificity matters:** a grep-count probe is structural, not behavioural.

### Impl

- **Workers correctly emit `kind: "log"` PMD-fallback retros** when the daemon worktree PMD's FTS5 vtable is unavailable. Tasks 6 + 8 both wrote retro content into a `kind: "log"` DQ entry, `answered_by: "impl-self-resolved"`, with retro scores embedded (`SCORE: 0.85` / `0.80`). The fallback path is what `feedback_junior_pmd_write_convention.md` predicted, but it consumed extra DQ ids the brief hadn't pre-allocated (Task 6 retro consumed #314, breaking Task 8's brief pre-assignment).

### BM

- N/A — v1-rls-r1 had no Junior `bm-task` dispatches (impl-task-only phase since v1-rls-r1 ships zero Rust; no PR triage round-trips planned until phase merge).

## What to change

### Advisor

- **For meta-work-only phases (zero Rust, all `.claude/**` + `docs/**`), default to advisor-authoring after the first cohort.** The plan's `[P]` markers + Cohort A model only pay off when worker independence is high AND the daemon is stable AND PMD plumbing works on the worker side. v1-rls-r1 demonstrated none of those held under concurrent multi-lane stress. Decision point: at the **second worker hang in a row** OR **first cross-lane contamination detected**, advisor should surface to user and offer the advisor-author path. The user-gate at v1-rls-r1 §post-Task-8 was the inflection.

- **Pre-dispatch daemon-FF as a structural gate.** Every Junior dispatch should do `ssh homeserver "cd /srv/brehon-fork && git checkout --detach HEAD && git branch -f phase-v1-<phase> origin/phase-v1-<phase> && git checkout phase-v1-<phase>"` (or a more lane-safe variant) BEFORE `mcp__junior-brehon__create_task`. Otherwise the worker's `base_branch` resolves against daemon-local-and-potentially-stale.

- **Cancel-and-FF-merge as a documented recovery template.** When a worker hangs post-DQ-raise but has pushed its feat + DQ commits, the standard recovery is: `cancel_task` → `git fetch origin "+refs/heads/junior/*:refs/remotes/origin/junior/*"` → `git merge --ff-only origin/junior/<branch>` → run cargo-check → mutate DQ → push. Should be in `advisor-orchestrator.md` §5.2 as a sub-pattern.

### Planning

- **Behavioural VALIDATE probes for hook scripts.** Task 7's grep-count probe missed the function-after-exit unreachable placement. A behavioural probe (e.g. "synthetic 3-attempt fail-open trigger → JSONL file MUST exist post-trigger") would have caught it. Plan §13 Task 10 included exactly that probe — the dogfood IS the behavioural check. Plans should consider whether the **per-task VALIDATE** has parity with the **integration dogfood VALIDATE**, or whether some bug classes can only be caught at integration time.

- **Pre-reserve DQ ids for cohort dispatch.** Interim mitigation 3 in `feedback_cohort_dq_id_collision.md` (advisor pre-authors N reserved stubs in pending[] before dispatching the cohort, brief names assigned id explicitly) is the cheapest tactical fix. Add to `advisor-orchestrator.md` §4.1 as a `[P]`-cohort-prerequisite step in the next planning sub-phase.

### Impl

- **Push first, raise DQ, exit cleanly.** The hang pattern (worker authors → DQ raise → push → hang) means workers should explicitly NOT attempt cargo-check in their own context — push the work + the DQ entry, then exit and let advisor run cargo-check on the canonical lane. Task 9's brief already added this guidance after Task 6/8 demonstrated the pattern. Make it a default in `impl-task-brief.template.md`.

### BM

- N/A — no BM activity this phase.

## What to carry forward

### Advisor

- **Atomic raise-and-resolve commit when advisor authors the work.** When advisor authors a task directly (Tasks 9-12), the DQ entry is written as `result: "pass"` in the same commit as the feat. No separate raise→mutate cycle; one commit captures both. Reduces audit-trail noise and matches the canonical "DQ as audit trail of decisions, not workflow" intent.

- **Single consolidated cargo-check across a bundled commit.** When two tasks ship in one commit (Tasks 11 + 12) and both are no-Rust-impact, one cargo-check covers both. Annotate this in the DQ entry's `answer` field so retro reads can reconstruct.

- **The lane-dedicated worktree pattern works.** `brehon-fork-rls-r1` stayed clean throughout; the contamination was always daemon-side. Confirms `multi-lane-worktree.md` discipline.

### Planning

- **Pre-commit dogfood IS load-bearing.** Plan §13 Task 10 was the only mechanism that caught Task 7's broken function placement. The cost of the dogfood was ~10 min advisor work + the fix-in-same-cycle authoring; the savings were "the JSONL trail actually works in production". Without it, `retro_bypass.jsonl` would have been empty forever and §5.2 autonomy-readiness criterion would have been unmeasurable while we believed it was instrumented.

- **§4 watchpoints proved specific enough.** All 10 watchpoints (R1-R10) cited specific files + lessons; no concept-only "watch X" entries. The §3.5 gate that demanded this pre-merge was preventive.

### Impl

- **Lessons + structural fix in pairs is now the durable pattern.** Task 12 closed two PENDING statuses (the canonical-PMD guard PENDING + the retro-harvest weekly cadence gap) by shipping both the structural code + the lesson naming it shipped. Every future PENDING-status lesson should aim for this pairing per `feedback_lesson_must_pair_with_structural_fix_when_fixable.md`.

### BM

- N/A.

## Per-task complexity scores

Format: `<files-touched>/<commits>/<runtime-min>/<max-log-silence-min>`. Junior dispatches that hung are noted separately.

| Task | Files | Commits | Runtime min | Silence max | Notes |
|---|---|---|---|---|---|
| 1 | 2 (creates) | 2 | 14 (worker) | <2 | Junior #372; Cohort A; pre-push cargo failed env trap, advisor cleared |
| 2 | 1 (creates) | 2 | 11 (worker) | <2 | Junior #373; Cohort A |
| 3 | 1 (creates) | 2 | 23 (worker) | <2 | Junior #374; Cohort A; longest cohort runtime |
| 4 | 1 (modifies) | 2 | 9 (worker) | <2 | Junior #375; Cohort A |
| 5 | 1 (modifies) | 2 | 3 (worker) | <2 | Junior #379; serial chain |
| 6 | 1 (creates) | 2 | 13 (worker hung) | 10+ | Junior #380; hang pattern instance #1 |
| 7 | 1 (modifies) | 2 | 11 (worker) | <2 | Junior #376; Cohort A; **bug shipped — fixed in Task 10** |
| 8 | 2 (1+1) | 3 | 12 (worker hung) | 10+ | Junior #381; hang pattern instance #2 + delayed retro #314 consumed brief-pre-assigned id |
| 9 | 1 (creates) | 1 | 2 (advisor) | n/a | Advisor-authored; cargo-check 2m |
| 10 | 2 (1 report + 1 retro-check fix) | 1 | 25 (advisor) | n/a | Advisor-driven; caught + fixed Task 7 regression in-cycle |
| 11 | 1 (modifies) | 1 (bundled with 12) | 3 (advisor) | n/a | Advisor-authored; surgical edits |
| 12 | 2 (creates [P]) | 1 (bundled with 11) | 4 (advisor) | n/a | Advisor-authored |
| 13 | 1 (creates this retro) | 1 | ~25 (advisor) | n/a | This retro |

**Aggregate:** 17 files touched, 21 commits on phase branch, ~155 min Junior wall-clock (4× worker hangs cost ~30 min recovery), ~80 min advisor authoring + dogfood. Total phase wall: ~16 hours including plan author 2026-05-20 → tip 2026-05-21.

## Decisions to revisit

- **`.claude/harvest/` gitignored-vs-tracked.** Task 5 made these gitignored (runtime journal semantics). If the v1-rls-r2 plan adds analytics over harvest output, tracking the artifact (NOT the gitignore) becomes interesting.

- **`pmd-invariants.md` auto-load-by-presence vs `paths:` frontmatter scoping.** Task 2 went auto-load (no frontmatter). If we add `paths:` scoping to other rule files later, retrofit decision.

- **Track C kind-registry placement (`docs/` vs `.claude/refs/`).** Task 8 put it in `docs/brehon-law-inspired-network/` because it's design-document-adjacent. `.claude/refs/` would have been more harness-internal. The choice may revisit if non-product hook-emitted JSONL kinds grow significantly.

- **Advisor-authoring fallback.** This phase legitimised the pattern under daemon-contamination stress. v1-rls-r2 should explicitly include the criterion ("at second worker hang OR first cross-lane contamination, advisor offers advisor-author path") in `advisor-orchestrator.md` or a dedicated lesson.

## Lessons promoted in this retro commit

- `feedback_advisor_authoring_under_daemon_stress.md` (NEW — see below; documents the v1-rls-r1 inflection at post-Task-8)
- `feedback_cohort_dq_id_collision.md` (already shipped at `c2291ed48` mid-phase) — first use of the lesson + retro citation
- `feedback_worker_hang_post_dq_raise.md` (NEW — see below)
- `feedback_cross_lane_daemon_ref_contamination.md` (NEW — see below)

## Harness gap to flag for v1-rls-r2

- **`.claude/**` write-protection / Junior-worker harness gap** — recurred during v1-rls-r1 planning Junior #367 (plan written to worktree root + escalation memo because sensitive-file gate blocked `.claude/**` writes). v1-rls-r2 candidate: add `.claude/PRPs/plans/**` + `.claude/decision-queue.json` to Junior worker permission allowlist. Per DQ #306 (transcribed during planning).

- **Cross-lane daemon-ref contamination** — observed 3+ times. Structural fix unknown; tactical fix (pre-dispatch detach+branch-f+checkout) works but adds latency. v1-rls-r2 candidate: investigate whether daemon can dispatch from per-lane ref-namespaces (e.g. `refs/heads/lanes/<lane>/phase-v1-...`) to avoid the single-branch-ref shared state.

- **Junior worker hang post-DQ-raise** — 2-of-3 incidence rate this phase. v1-rls-r2 candidate: liveness watchdog tightened (currently 60-min cap; could be 10-min cap on `Sl`-state workers post-push-confirmed-on-origin).

## See also

- `.claude/PRPs/plans/v1-rls-r1.plan.md` — the plan
- `.claude/PRPs/reports/v1-rls-r1-dogfood-2026-05-21.md` — Task 10 report
- `feedback_cohort_dq_id_collision.md` — Cohort A recovery lesson
- `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md` — the canonical retro discipline this honours
- `docs/research/brehon-rls-pmd-review.md` §3–§6 — originating recommendations that drove v1-rls-r1's scope
