# Session retro — 2026-06-08 — Pi harness context gap

**Harness:** claude-code (advisor session)
**Session window:** 2026-06-07 ~22:00 UTC → 2026-06-08 ~07:30 UTC (~570 min, multi-turn)
**Branch at start:** `be8134d0b` (`governance-v0`)
**Branch at end:** `74daf8b0f` (`governance-v0`)
**Files touched:** 4 (comparator-judge.sh, prp-plan.md, pi-harness-constraints.md, memory update)
**Commits:** 2 explicit (`35ad84565` digest fix, `74daf8b0f` harness constraints)

## TL;DR

The session completed planning-007 (GPT-5.5 planning challenger) end-to-end: rate-limit recovery from prior runs, successful plan production (844 lines), MiniMax M3 judge run (twice — first with broken digest, second valid), and a root-cause investigation of the 8-point score gap. The core finding: **the gap was entirely a harness context gap, not a model capability gap**. GPT-5.5 read zero `.claude/lessons/` files in 51 turns; the Opus control session gets those auto-loaded. Two specific missing pieces cost 6/8 pts: validate-pending-laptop discipline (Dim 7, -4) and ADR-015 callsite requirement (Dim 3, -2). Fix: progressive disclosure via a mandatory `Read: .claude/rules/pi-harness-constraints.md` gate in the Pi planning prompt. Shipped as commit `74daf8b0f`.

---

## What surprised us

**Advisor:**
- The `digest_plan()` regex only matched numbered section headings (`## 1.`, `## 4.`, etc.) — entirely stripping the GPT-5.5 plan which uses `## Summary`, `## Solution Statement`. Two judge runs produced ghost-plan scores (24/7, then 24/7 again) before the trace analysis revealed the real cause. The digest worked perfectly once fixed; it just needed the unnumbered aliases.

- `git update-ref refs/heads/governance-v0 origin/governance-v0` updates the ref but NOT the working tree when the daemon is checked out on a different branch (`phase-m2-late-1`). The fix (direct `cp` to the daemon's working tree) was correct but the fetch→checkout assumption was wrong. This is a recurring failure mode on multi-lane setups.

- The `plans_produced: 0` counter in `meta.json` is a persistent runner bug — it never increments even when the plan is committed. The workaround (`git log ab-cell/<exp>-challenger --oneline`) is reliable but the counter is misleading. Two iterations were wasted assuming it was authoritative.

- GPT-5.5 produced a **structurally sound, MIRROR-ref-heavy 844-line plan** in 51 turns with 1 compaction. The write-first gate (commit `da97ed20d`) worked exactly as designed — prior runs without it produced 0 output plans as GPT-5.5 exhausted context reading files before writing anything.

**Planning (no planning task this session):** N/A

**Impl (no impl tasks this session):** N/A

**Comparator/judge role:**
- MiniMax M3 blind judge reasoning was thorough (pass 1: ~200 lines of structured analysis, pass 2: similar). Both passes independently picked the same winner despite being position-swapped. The `MIXED_ROUTING` label in the JSON was a display artifact — the underlying scoring was clean.

---

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Fix `plans_produced` counter** in `comparator-token-extract.sh` — detect commits on `ab-cell/<exp>-challenger` branch, not in the working tree | Eliminates misleading `plans_produced: 0` that caused 2× wasted investigation turns | minor | 2× this session |
| 2 | **Add daemon-branch-awareness to `run-comparator.sh`** — before pushing `governance-v0` fixes to daemon, check `ssh homeserver "cd /srv/brehon-fork && git branch --show-current"` and copy the file directly if not on `governance-v0` | Eliminates the `git update-ref` footgun where the working tree stays stale | minor | 1× this session, recurring pattern in multi-lane |
| 3 | **Add `HARNESS_CONSTRAINTS: loaded` verification to judge scoring** — if the challenger trace shows zero reads of `.claude/rules/pi-harness-constraints.md`, flag in `token-signals.json` as `harness_context_loaded: false` | Makes context-gap attribution mechanical; prevents future judge runs scoring a ghost-plan scenario silently | minor | 1× this session (2 ghost runs before diagnosis) |
| 4 | **Pin judge digest format check** in `comparator-judge.sh` — after `digest_plan()`, assert `len(control_digest) > 1000` and `len(challenger_digest) > 1000`; fail fast if either is empty/tiny | Catches stripped-digest before wasting 2× MiniMax API calls on ghost scores | minor | 1× this session (two wasted runs) |

---

## What to carry forward

- **Progressive disclosure works for Pi context injection.** The pattern — short prompt pointer → `Read: <constraints-file>` → `HARNESS_CONSTRAINTS: loaded` confirmation → continue — is the right architecture for giving Pi-hosted models operational context without bloating the always-loaded base prompt. GPT-5.5 natively uses tool calls; having it `Read` on demand is exactly how it should work. Apply this pattern for any future constraint category (e.g. four-role model context, DQ schema, phase-specific watchpoints).

- **Trace analysis is the reliable root-cause tool.** When a judge score looks wrong, reading the challenger's `trace.jsonl` for actual `tool_execution_start` events (files read, bash commands run) produces a definitive answer in ~5 minutes. Both "ghost plan" diagnoses this session came from trace analysis, not from reading the judge's reasoning. Keep `comparator-token-extract.sh`'s trace parsing as the primary diagnostic.

- **Write-first gate is load-bearing for large-context models.** GPT-5.5 with 272K context will exhaust it reading 50 files before writing if not forced to commit early. The gate (`commit skeleton after P0 docs, before codebase exploration`) solved the 0/4 → 2/2 plan-production rate. Carry this pattern to any future challenger model with a large context window.

- **Judge position-swapping works.** The MiniMax M3 judge independently agreed on the winner across both position swaps despite spending ~200 lines reasoning about it. The `MIXED_ROUTING` label is a display bug, not a reliability problem. Trust the per-dimension scores over the routing label.

- **Comparator evaluation is iterative, not one-shot.** This session fixed: (a) write-first gate for plan production, (b) digest regex for valid plan capture, (c) harness context injection for fair comparison. Each fix was validated by a new run. The right model for evaluator development is: run → diagnose via trace → fix one thing → re-run. Budget 3-4 iterations per new challenger setup.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Judge run (pass 1, broken digest) | 0 | 25 | high | digest stripped GPT-5.5 plan entirely; scored ghost plan |
| Judge run (pass 2, broken digest on daemon) | 0 | 20 | high | daemon not updated by `git update-ref` when on different branch |
| Trace analysis (`trace.jsonl` inspection) | 40 | 0 | medium | definitive diagnosis of both gaps in ~5 min; highly efficient |
| `comparator-judge.sh` digest fix (`35ad84565`) | 30 | 0 | low | straightforward regex extension once root cause was clear |
| Pi harness constraints injection (`74daf8b0f`) | est. 4+ hrs future savings | 0 | low | progressive disclosure pattern clean; judge can now be re-run on fair basis |
| MiniMax M3 judge (valid run) | 60 | 0 | low | both passes agreed; thorough 200-line reasoning per pass |
| planning-007 run (GPT-5.5, post write-first gate) | — | 0 | medium | 844-line plan in 51 turns; structurally strong; technically capable model |

## Complexity scores (heavy tasks only)

No impl tasks this session. The two commits were small targeted edits:

| Task | Files | Commits | Runtime (min) | Notes |
|---|---:|---:|---:|---|
| digest regex fix | 1 | 1 | 15 | targeted; no complexity risk |
| harness constraints + Pi prompt update | 2 | 1 | 25 | new file + prompt edit; straightforward |

---

## Decisions to revisit

- **planning-008 timing:** now that the harness context gap is fixed, planning-008 is the first fair comparison run. The judge can be re-run on planning-007 OR a fresh planning-008 can be launched. The question is whether to re-run on the same task (m2-late) to keep the comparison controlled, or move to a different planning task for fresh data. Recommend: one more run on the same task with the harness fix active to isolate the variable.

- **`MIXED_ROUTING` display bug:** the routing consensus field doesn't handle the case where pass1="ARM" (A wins) and pass2="ARM_B_BETTER" (B wins = same winner, different label). Fix should map both to `CONTROL_WINS` or `CHALLENGER_WINS` based on which arm was control in each pass. Minor, but misleading in the summary JSON.

- **n=1 data constraint:** planning-007 is a single data point. Per spec §8, n≥5 required before routing decision. The fix is applied; the priority is running planning-008/009/010 to build the n≥5 corpus before drawing conclusions.

---

## Promotion candidates

- [ ] `plans_produced counter bug`: promote to lesson `feedback_comparator_plans_produced_counter_unreliable.md` — warn future evaluators to check `git log ab-cell/` not `meta.json` counter
- [ ] `daemon working tree stale after git update-ref`: merge into existing `feedback_finalize_merge_where_to_look_first.md` or `multi-lane-worktree.md` — the "update-ref vs checkout on non-matching branch" failure mode belongs there
- [ ] `judge digest empty-plan guard`: add assertion to `comparator-judge.sh` directly (code change, not lesson) — see "What to change" #4

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
