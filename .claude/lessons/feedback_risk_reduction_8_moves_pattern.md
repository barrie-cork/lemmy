---
name: Risk-reduction 8-moves pattern for phase ralph loops
description: Before the ralph loop cuts a phase branch, apply 8 low-cost guardrails to collapse unknown-unknowns into knowns — Phase 5c reference implementation
type: feedback
originSessionId: 7ded97b8-c346-4fa3-b03f-6690b2b11bd2
---
When starting a large multi-task phase on this fork, do NOT cut the phase branch and start `/prp-ralph-slice` without running the 8 risk-reduction moves first. Phase 5c (2026-04-18) is the reference — ~3 hours of preparation across 8 moves reduced a ~5-day phase confidence score from 8/10 to 9/10 and preempted the classes of bug that previously surfaced via mid-phase `println`-driven debug sessions.

**Why:** Phase 5b's split-plane bug (decision-queue #16) cost half a ralph iteration because an integration invariant was only visible via a diagnostic print. Phase 2a's checkpoint-2 cascade cost 30 min of advisor time because a wrapper bug false-greened task-14 validation. Both classes are catchable before task 1 runs. The 8-moves pattern generalises the preemption.

**How to apply:**

1. **Move 1 — Resolve critical-path decision-queue entries BEFORE task 0.** Identify questions in the plan's §17 intake whose wrong answer would break compile-time correctness of a later task. For each, ANSWER by reading the relevant file or docs.rs page now (typically 15 min total). Demote the rest to "impl-self-resolve at task 0" — those are informational, not gating.

2. **Move 2 — Audit the authoritative DoD for unallocated lines.** Read IMPLEMENTATION-PLAN-v0.md §3 "Phase N definition of done" and confirm every bullet maps to a plan §11 task. If a DoD line has no task slot, add a decision-queue intake entry (with recommendation) and decide placement before task 0.

3. **Move 3 — Reorder any task that bundles a file edit + the test that validates the edit, so the test lands FIRST on a throwaway local commit.** If the test edit can fail in the wrong way (e.g. over-asserting, under-asserting), you must observe it failing in the right way BEFORE the production edit lands. After the edit, re-observe, then land the fix, then squash. Three observations replace one hope.

4. **Move 4 — Fill per-handler test gaps proactively.** Count handlers added in the phase; subtract any that have a direct happy-path e2e. If >2 handlers have only route-level / smoke coverage, extend the smoke test with per-handler fixture + response-body assertions in the same commit. Cheaper than a PR-review round-trip finding the DTO shape bug later.

5. **Move 5 — Extract shared helpers at first-use time, not "maybe later".** If task N writes a helper that task N+K "may reuse", extract it to a sibling module NOW. Leaving duplicated code until task N+K cleanup is how a CodeRabbit "duplicate logic" finding lands in a PR.

6. **Move 6 — Pin every unpinned external API via a scratch probe at task 0.** SQL syntax on the actual Postgres image, crate API shape at the actual version, route-wiring fn names. Each probe ≤ 30 lines in `scratch/phase-N-probes/`, committed as `chore(scratch): phase-N external-API probes` before task 1. If any probe fails, surface via decision queue before burning context.

7. **Move 7 — Land any known-needed test-harness guard on governance-v0 BEFORE cutting the phase branch.** Phase 5b carry-forward #3 (the `--test-threads=1` wrapper guard) is the archetype: it was identified at 5b close, it belongs before 5c, and landing it as `chore(scripts):` on governance-v0 keeps the phase branch focused on feature work.

8. **Move 8 — Reserve a slice-split boundary without committing to it.** Pre-identify the natural halfway point in the task list. If the first half finishes in ≤ 60% of planned iterations, continue straight through. If it takes ≥ 80%, stop at the boundary, archive, rest the context window, start the second slice fresh. Don't announce the split in the plan — it's a release valve, not a commitment.

**How to apply:** When a user asks "what would reduce the risk of implementing this plan?", run two parallel Explore agents first (one on prior-phase evidence, one on plan's structural risk concentrations), then synthesize the 8 moves against the evidence. Write the strategy to the plan file WITHOUT rewriting the plan itself — the plan stays authoritative, the strategy adds guardrails around it. Total cost: ~2-3 hours vs. a likely ~1 day of avoided rework.

**Concrete Phase 5c artefacts:**
- Strategy document: `C:\Users\barri\.claude\plans\what-would-be-a-proud-balloon.md` (the 8 moves in detail).
- Plan-file edits: `.claude/PRPs/plans/phase-5c-remaining-endpoints-and-observability.plan.md` §2, §5 LOC cap table, §11.0 task 0 step 8, §11.3 task 63d, §11.4 task 64 flip-verification sequence + jury_common.rs, §11.8 task 68c Phase B, §14 risk register (+ 7 rows), §17 intake.
- Decision-queue: entries #19 (actix config fn), #20 (tokio-postgres poll-based), #21 (staleness alert) — all `answered_by: impl-self-resolved` before task 0.
- Wrapper edit: `scripts/brehon/cargo-test.bat` `--test-threads=1` guard via goto-based flow.
