---
name: Plan §5 complexity score with split-or-proceed DQ at threshold > 8
description: Planner computes a mechanical complexity score (factor table in plan.template.md §5.1) and writes it into §5 Metadata before commit. If score > 8, planner files a `pending` DQ entry to the advisor asking "split into <slug>-1 + <slug>-2, or proceed?" Catches over-bundled sub-phases pre-impl rather than at retro.
type: feedback
---

In `.claude/PRPs/templates/plan.template.md` §5 Metadata, every plan now carries a `Complexity score: <N>/10` line + a §5.1 factor breakdown table. The planner computes the score mechanically from a fixed factor list (impl-task count above 5, migrations touched, crates touched, e2e.rs edits, ADR-affecting decisions, cargo budget peak above 6 GB) and writes it before commit. If `score > 8`, the planner files a `pending` DQ entry to the advisor with `from: "planner"`, `kind: "blocker"`, asking "complexity N exceeds threshold — split into `<slug>-1` + `<slug>-2`, or proceed?" The planner does not ship the plan until the DQ resolves.

**Why:** Brehon's recent JM-d incidents — task #13/#14 worker-hangs on the 8945-line e2e.rs (per `feedback_junior_worker_e2e_edit_hang.md`), task #18 pre-flight serialisation on shared cargo target, task #10 OOM cascade per `project_elitedesk_hung_2026_04_27.md` — all map to "this sub-phase was too complex for one impl run." The four-role retros eventually catch it via `feedback_retro_task_complexity_score.md` per-task one-liner aggregation. mfs's `prp-breakdown-tasks.md` Step 8 demonstrated a stronger pattern: a calculator the planner runs *pre-impl* with an auto-split recommendation if the score exceeds threshold or task count exceeds 20. The Brehon adaptation re-tunes the factor weights for the Brehon execution envelope (e2e.rs hang risk, EliteDesk cargo cap, ADR decision-cost) and turns the auto-split into a planner-files-DQ escalation rather than a silent decomposition.

The threshold > 8 is calibrated against shipped phases:

- v1-AD-* phases: scores 3-5 (small, focused, no e2e). Threshold non-binding.
- v1-JM-a/b/c phases: scores 5-7 (moderate, single-crate, bounded e2e). Threshold non-binding.
- v1-JM-d phase: score retroactively ~9 if computed (8 impl tasks, 2 migrations, 3 crates, 2 e2e edits = 3+4+3+6 = 16; the calibration used different weights, but the qualitative signal lines up). Threshold would have triggered. The retro flagged "should have been split" as a deferred follow-up — exactly the symptom this rule prevents.

**How to apply (planner side — see `.claude/agents/planning.md` "§5 complexity score + split threshold"):**

- After writing §13 (so the impl-task count + e2e.rs edit count + migration count are stable), compute each row of the §5.1 factor table mechanically:

  | Factor | Weight | How to count |
  |---|---|---|
  | §13 impl tasks above 5 | +1 each | excludes Task 0 (pre-flight) and the retro task |
  | Migrations touched | +2 each | grep `migrations/<id>__<name>/up.sql` paths in §11 |
  | Crates touched | +1 each | `crates/<X>/` path-prefix unique count in §11 |
  | `crates/lemmy_server/tests/e2e/*.rs` edits | +3 each | per `feedback_junior_worker_e2e_edit_hang.md` (worker-hang risk on >8000 line files) |
  | New ADR-affecting decisions | +2 each | any decision that supersedes an entry in `docs/research/brehon-law-inspired-network/99-decisions-and-open-questions.md` |
  | Cargo budget peak above 6 GB | +1 per GB | pre-Shape-G plans only; Shape-G plans set this to 0 (cargo runs off-box) |

- Write the total + per-factor breakdown into §5.1. Sum is the score; show your work in the table.
- If `score > 8`: file a `pending` DQ entry with `from: "planner"`, `kind: "blocker"`, `question: "complexity <N> exceeds threshold — split into <slug>-1 + <slug>-2, or proceed?"`, and `options: ["split into <slug>-1 (Tasks 1-N) + <slug>-2 (Tasks N+1..end)", "proceed despite score (cite mitigation: <e.g. Shape G removes cargo budget>)"]`. Commit + push the DQ before pushing the plan. The advisor's polling loop reads the DQ on next tick.
- If `score ≤ 8`: just write the score. No DQ, no escalation.

**How to apply (advisor side — see `.claude/rules/advisor-orchestrator.md` "Pre-queue lesson check" §5.1 sub-section):**

- When the planning subagent ships a plan, the DoD smoke test runs as normal (per `feedback_pre_phase_dod_smoke_test.md`).
- Before queueing the first impl-task: read plan §5 complexity score + §5.1 top factor.
- If the planner filed a `pending` DQ with `from: "planner"`, `kind: "blocker"`, referencing this plan's score, **resolve it first** per the DQ triage decision tree:
  - **Advisor-mode:** if a comparable prior phase shipped with similar score + similar top factor without incident (e.g. score 9, top factor "5 ADR decisions, all resolved cleanly"), the advisor can answer "proceed" with citation. Subject `chore(advisor|decision-queue): answer DQ #<id>`.
  - **User-relay:** if the dominant factor is e2e.rs edits (≥2) or the score exceeds 10, surface to user with the breakdown — the split-or-proceed call is judgment-heavy (scope changes, retrofit cost). Record user's answer with `answered_by: "user"`.
- After resolution, queue impl per the cohort dispatch rules. Append the lesson cited by the dominant factor to each impl-task's brief §3 Required reading (e.g. `feedback_junior_worker_e2e_edit_hang.md` for an e2e-dominant plan; `feedback_resource_budget_pre_queue.md` for a cargo-budget-dominant pre-Shape-G plan).

**Refusals:**

- Never edit a shipped plan's §5 complexity score retroactively to "make it pass" the threshold — the score is a planner-side write at plan-write time. If a retroactive recomputation would cross the threshold, that's a retro signal, not a plan edit.
- Never bypass the planner-DQ when score > 8 — the gate exists precisely to make split-or-proceed an explicit decision rather than an implicit ship.
- Never auto-split a plan from advisor side — the DQ surfaces the question; the user or advisor (with citation) answers; the planner ships either the original or two split plans. The advisor never authors plan content (per `.claude/rules/advisor-orchestrator.md`).

**When to skip:**

- Plans explicitly limited to a single retro-only task or a doc-only edit (no §13 impl tasks). Set complexity score to "n/a — no impl content".
- BM-task and planning-task plans (only impl-task plans carry the score; the others have different envelopes per `feedback_retro_task_complexity_score.md`).

**Symptom to recognise in retrospect:** in a finished sub-phase's retro, look for the per-task `complexity: <files>/<commits>/<runtime-min>/<max-log-silence-min>` (per `feedback_retro_task_complexity_score.md`) lines. If the median per-task `runtime-min` is >50 or median `max-log-silence-min` is >30, the plan was over-bundled for the impl-task envelope. Cross-check against §5 score:

- If §5 said "score 5" and the actual median is 50/30, the calculator is under-counting — propose a factor weight bump in the next plan-template iteration.
- If §5 said "score 9" and the planner's DQ was answered "proceed" without mitigation, the proceed answer was wrong — record the miss in retro §3 carry-forward and bias the next sub-phase toward "split".
- If §5 said "score 9" and the DQ was answered "split", the gate worked as designed.

**Generalises to:** any plan-driven workflow where impl-execution has wall-clock or context-budget limits the planner doesn't see. The mfs decomposer's calculator (Step 8) and Brehon's adaptation are the same primitive — pre-impl signal that prevents the "this should have been split" retro line. Different orchestration models tune the factors differently; the structural rule (planner computes, writes into §5 Metadata, escalates above threshold) is portable.

**Related:**
- `feedback_retro_task_complexity_score.md` — the per-task post-hoc metric the score's calibration is derived from
- `feedback_junior_worker_e2e_edit_hang.md` — the +3-per-e2e-edit weight's empirical source
- `feedback_resource_budget_pre_queue.md` — the cargo-budget-peak factor's source (and the EliteDesk's `MemoryMax=10G` cap)
- `feedback_parallel_cohort_dispatch.md` — cohort budget check uses the same per-task ~6 GB cargo peak the score's cargo-budget factor uses
- `feedback_schema_changing_spec_retrofit_question.md` — why this additive ships forward-only
- `feedback_principles_not_rules.md` — the `> 8` threshold is calibrated, not absolute; planner judgment can override with citation
