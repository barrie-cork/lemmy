---
name: /brehon-verify pre-merge advisor gate (spec-kit pattern adoption)
description: Before queueing bm-merge, the advisor runs /brehon-verify <phase> to cross-check plan §16a stories against the worktree branch. Distinct from CR triage — verify catches phantom completions (file/symbol absent) and pre-merge regressions; CR catches code-quality findings. Both gates stay; neither replaces the other.
type: feedback
---

The advisor's `/brehon-verify <phase>` runs after every `[P]`-cohort + barrier task in §13 has shipped commits, and before `bm-merge` is queued. It cross-checks plan §16a stories against the actual worktree branch state.

**Why:** CR triage catches regressions post-PR. Nothing today catches the case where a task's commit lands on the phase branch but the expected output isn't actually there — e.g. the impl agent committed everything except one of the three planned files, or the file landed but the planned symbol is absent (renamed, deferred, or simply forgotten). Spec-kit's `/speckit.verify` is the primitive; the Brehon adaptation runs pre-merge (cheaper than post-merge revert) and reuses `.claude/PRPs/reports/` + the catch-fire procedure for phantoms.

**How to apply:**

- **Run only after every §13 task has a commit on `origin/phase-<phase>`.** Refuse if any task is missing its commit (impl run incomplete).
- **Refuse if a Junior task is currently running on the phase** (race risk).
- **Refuse if the plan has no §16a stories block** (legacy plan — manual reconciliation only; file a DQ asking for retrofit).
- **For each story:**
  - Output presence check: `git show origin/phase-<phase>:<file>` for each Brief-Scope output. Absent or empty → ✗ phantom.
  - Structural-pattern check: `rg -q '<pattern>'` on the file content. Pattern not matched → ✗ phantom.
  - Checkpoint command: run the bash literal block (typically a cargo test). Capture exit code separately (per `feedback_pipes_mask_exit_codes.md`). Non-zero → ✗ regression.
- **Forbidden-window discipline applies** — verify consumes the same EliteDesk resources as impl-tasks. Defer the whole pass; partial verify is worse than deferred verify.
- **Report at `.claude/PRPs/reports/<phase>-verify.md`** with one row per story. Commit subject: `docs(advisor): brehon-verify <phase> — <outcome summary>`.

**Outcome routing:**

- All ✓ → advance to merge-confirm user gate.
- Any phantom → catch-fire (per advisor-orchestrator.md). Surface to user with phantom story names + failing structural patterns. Queue a fix-in-PR impl-task targeting the missing output. Do not queue bm-merge.
- Any regression → catch-fire as "regression suspected; CR triage missed it". File DQ pending entry citing story + checkpoint output.
- Any [malformed] → file DQ asking planner to retrofit §16a. Verify pass is incomplete until corrective commit.

**Verify ≠ CR triage.** Both gates stay:
- Verify catches **phantom completions** (advisor-side reconciliation between brief Scope + plan §13 IMPLEMENT lists vs. worktree branch state).
- CR triage handles **regressions and quality findings** (CodeRabbit-side review post-PR).

**Refusals:**

- Never modify `<file>` on the phase branch to "fix" a phantom (advisor never authors content). Queue a fix-in-PR impl-task instead.
- Never write the report to `.claude/PRPs/debug/` (that's Junior subagent capture). Reports go to `.claude/PRPs/reports/`.
- Never run cargo-class checkpoints during a forbidden window (defer the entire pass).

**Re-running:** after fix-in-PR commits land, re-run `/brehon-verify` to confirm phantoms cleared. The report is committed at each run (not amended) — audit trail matters.

**Symptom to recognise in retrospect:** a CR comment along the lines of "this task said it added X but I don't see X anywhere" landing on a PR that was already approved by the advisor — that's a phantom verify would have caught. If you see one, audit the parent §16a entry — either it was omitted (legacy plan) or its Brief-Scope output descriptor was under-specified.

**Generalizes to:** any pre-PR review gate where "task complete" is signaled by commit + tests pass, but the commit might omit one of several planned outputs. The verify pattern (presence + structural pattern + checkpoint) is the cheapest reliable phantom-detector that doesn't require running the full test suite.
