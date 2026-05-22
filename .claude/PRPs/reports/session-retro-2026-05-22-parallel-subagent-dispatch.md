# Session retro — 2026-05-22 — parallel-subagent-dispatch

**Harness:** claude-code
**Session window:** 2026-05-22T01:00Z → 2026-05-22T01:25Z (~25 min — short segment, post-prior-retro)
**Branch at start:** `6467ae69d` (`governance-v0`)
**Branch at end:** `528cb76ee` (`governance-v0`)
**Files touched:** 4 (own commit)
**Commits:** 1 (auto: 0, explicit: 1 — `528cb76ee`)
**Prior retro on this session:** `session-retro-2026-05-22-dq338-rca-revision-option-b-ship.md`. This is a **short follow-up retro** for the sub-agent-dispatch segment that came after.

## TL;DR

After the prior retro identified three carry-forward items (rule promotion / lesson authoring / hooks audit), I dispatched three `general-purpose` sub-agents in parallel and bundled their deliverables in one commit. The dispatch worked: all three returned usable deliverables on first invocation, parallel execution was ~5-7 min wall-clock, and the bundle shipped clean. **One new failure mode:** sub-agent A's working-tree edit was *clobbered by a concurrent session's commit* before I verified the result — distinct from the prior session's index-staging-race; this is a **working-tree-mutation race**. Re-applied inline. Highest-leverage carry-forward: parallel sub-agent dispatch for independent deliverables is now repeatable; verify-after-subagent-completes is mandatory belt-and-braces.

---

## What surprised us

### Advisor

- **The clobber pattern is broader than the prior collision lesson captured.** I shipped `feedback_cross_session_commit_attribution_collision.md` in this session's first half — it covers two sessions racing on the shared `.git/` *index* (one's `git add` sweeps another's staged files into its commit subject). But this segment hit a different shape: sub-agent A wrote an edit to `advisor-orchestrator.md` working-tree at line 389; before I committed, a concurrent session's commit advanced `governance-v0` past my fetch-window, and its tree state did NOT include sub-agent A's edit. The unstaged edit was effectively reverted by `git pull` / fast-forward state alignment. The prior lesson's `git status` mitigation between add and commit doesn't catch this — my `git status` was fine, but the *working-tree change itself* was gone before I staged it. New race surface: **unstaged working-tree edits + concurrent push to same branch → silent revert when local diverges from remote and fast-forward state-aligns**.
- **`Agent` tool parallelism actually parallelises.** I sent three `Agent` invocations in one assistant message and they ran concurrently — total wall-clock ~7 min for all three, not 21 min serial. Sub-agent A returned first (~75s), then C (~342s, the longest — it did the most reading), then B (~124s). The single-message dispatch is the load-bearing part; I'd previously assumed dispatching one at a time was a discipline thing, but the harness actually does parallelise when given multiple Agent blocks in one message.

### Planning

(N/A — this segment did no planning.)

### Impl

- **Sub-agent A's "edit landed" claim was true at sub-agent exit, false at parent-session verify.** Sub-agent A's report stated "Edit landed at line 389, file modified but not staged"; ~30s later I ran `grep -n "Falsifiable-hypothesis gate"` and got zero matches. Both statements were true at their respective moments — the concurrent session intervened between them. **Sub-agent reports describe sub-agent state at exit, NOT current parent-session state.** Trivial in isolation, easy to internalise. Worth noting because it inverts the usual subagent reliability pattern (subagent claims something is done → assume it's done).

### BM

(N/A — no BM verbs.)

---

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Promote `feedback_cross_session_commit_attribution_collision.md` to cover BOTH races** (index-staging AND working-tree-mutation). Add a "Working-tree-mutation race" sub-section: when an unstaged edit lives in the working tree and a concurrent session pushes to the same branch, the next `git pull` / fast-forward (implicit or explicit) silently reverts the unstaged edit. Mitigation: stage immediately after the edit lands (`git add` BEFORE any other tool call); OR re-grep to verify the edit still exists immediately before staging. | Catches the next clobber-class incident. | minor (edit existing lesson) | 1× this segment + 1× prior session segment = 2 incidents on shared `.git/` writes. Recurrence threshold MET; promote now. |
| 2 | **Add a "verify-after-subagent-completes" check to any future sub-agent dispatch on tracked files.** Specifically: after the sub-agent returns and claims an edit landed, run `grep` for a distinctive string from its reported diff before considering the edit committed-pending. Cost: one grep call per sub-agent. Saves: catch-fire incident class where sub-agent's claim is stale by parent verify time. | Prevents trusting stale sub-agent reports on tracked-file edits. | minor (one grep per dispatch) | 1× this segment — single-occurrence noise; defer promotion until 2nd. |
| 3 | **Dispatch parallel sub-agents in a single Agent-tool-block message routinely.** This worked first-try, ~3× wall-clock improvement over serial. Codify the pattern in advisor-orchestrator.md §"Subagent delegation": "for N independent deliverables, dispatch all N in one message with multiple Agent blocks; total wall-clock is max(per-agent runtime), not sum." | Speeds future retro-followup / multi-deliverable sessions. | minor (one-paragraph rule edit) | 1× this segment (first deliberate use; the pattern existed in tool docs but wasn't internalised). Defer promotion; verify it generalises across 2+ session types first. |

## What to carry forward

- **Parallel sub-agent dispatch via one message with multiple `Agent` blocks** for independent retro-followups. Demonstrated 3 sub-agents in ~7 min wall-clock; single-invocation reliability. Repeat pattern next time a retro identifies ≥2 independent deliverables.
- **Verify sub-agent-claimed file edits with `grep` before committing.** Caught the working-tree-clobber this segment; would have shipped a missing rule edit otherwise.
- **In-repo lesson copies are the citation target, not the user-memory originals.** Sub-agent A's rule citation pointed at `.claude/lessons/feedback_falsifiable_hypothesis_before_structural_fix.md` which initially didn't exist on disk (only in `C:/Users/barri/.claude/projects/.../memory/`). The fix was to author an in-repo copy alongside the user-memory copy. Pattern: every rule citation `.claude/lessons/<file>` MUST be a real path under the in-repo lessons dir; user-memory pointers are for cross-session memory injection, not rule-side citations.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `Agent` sub-agent A (rule promotion) | ~10 | 5 | medium | First-try usable diff; ~75s runtime. Wasted 5 min when I re-discovered the edit had been clobbered + re-applied inline (vs trusting the sub-agent's report). |
| `Agent` sub-agent B (lesson authoring) | ~15 | 0 | none | 89-line lesson on first invocation; ~124s runtime. Matched canonical lesson shape (frontmatter + sections); cross-refs accurate. |
| `Agent` sub-agent C (hooks audit) | ~25 | 0 | low | 201-line audit report on first invocation; ~342s runtime. The longest of the three (read 14 hook bodies + timed several). Returned actionable tier-1 recommendations + latency outlier table. |
| Parallel dispatch (one message, three `Agent` blocks) | ~14 | 0 | medium | Three sub-agents ran concurrently; wall-clock ~7 min vs ~12-15 min serial. Bigger surprise than expected — I'd assumed dispatch order mattered. |
| Verify-by-grep (sub-agent A clobber catch) | ~10 | 0 | high | Three lines of bash caught the clobber. Without it: would have committed a 4-file bundle missing one of the deliverables; would have shipped under a misleading commit subject claiming the rule edit landed. |
| `git add` specific files + `git status` verify (per prior lesson) | ~5 | 0 | none | Clean attribution; only my 4 files staged. Pattern is now reflex. |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Sub-agent dispatch + bundle commit | 4 | 1 | ~25 | n/a (interactive, not Junior) |

No flags against thresholds (>55min runtime, >40min log silence, >8 files). Below the watchdog envelope by design — this was a deliberate-short session segment.

## Decisions to revisit

- **Whether `feedback_cross_session_commit_attribution_collision.md` should split into TWO lessons** (index-staging race vs working-tree-mutation race) or stay one lesson covering both. The races have distinct mitigations (verify-staged-files-match-intent vs verify-edit-still-exists). One lesson with two sub-sections is the minimal-overhead path; splitting is cleaner per-mitigation but more files to maintain. Lean toward single-lesson-with-two-subsections.
- **Whether to add a PreToolUse hook for the working-tree-mutation race.** Pattern: any `Edit` on a tracked file followed by ≥30s of other tool calls without `git add` is a candidate for clobber. Hook could WARN. Defer until 2nd incident — recurrence is currently 1.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1: extend `feedback_cross_session_commit_attribution_collision.md` to cover working-tree-mutation race. Recurrence ≥ 2 across this session's segments (index-staging in first half + working-tree-mutation here).
- [ ] Change #3: codify "parallel sub-agent dispatch in one message" in advisor-orchestrator.md §"Subagent delegation". Recurrence not yet ≥ 2; defer until verified across 2+ session types (next applicable: retro-followups in another sub-phase).
- [ ] Change #2 (verify-after-subagent): single occurrence; record but don't promote.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
