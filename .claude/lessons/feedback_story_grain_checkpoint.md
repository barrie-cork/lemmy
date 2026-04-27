---
name: Story-grain checkpoints in plan §16a (spec-kit pattern adoption)
description: Insert a §16a Stories block in plan files between §16 Acceptance criteria and §17 Completion checklist. A story is the smallest unit producing an end-to-end testable behaviour, with its own checkpoint command. The advisor's `/brehon-verify` runs each story checkpoint and confirms Brief-Scope outputs exist on the worktree branch. Catches phantom completions earlier than CR triage.
type: feedback
---

Brehon plans carry §16 acceptance criteria that roll up at the phase level. A multi-task phase can pass §16 in aggregate while individual tasks shipped commits whose expected outputs are absent (renamed, deferred, or simply forgotten). CR triage catches regressions post-PR; nothing today catches **phantom completions** (task marked done but expected output not actually present) advisor-side, pre-merge.

§16a Stories block adds a story-grain layer between phase acceptance and per-task validation. Each story is the smallest unit that produces an end-to-end testable behaviour, with its own checkpoint command and Brief-Scope outputs. The advisor's `/brehon-verify` iterates §16a, runs each checkpoint against the worktree branch, and confirms outputs match structural patterns.

**Why:** `feedback_advisor_watchpoint_specificity.md` already requires watchpoints cite specific file:line. `/brehon-verify` is the **mechanical reconciliation** of those watchpoints + brief Scope + plan §13 IMPLEMENT lists against worktree branch state. Spec-kit's `/speckit.verify` is the same primitive. The Brehon adaptation runs pre-merge (cheaper than post-merge revert) and uses `git show <branch>:<file>` per the existing BM-task discipline.

**How to apply (planner side — see `.claude/agents/planning.md`):**

For each story:

- **Composing tasks:** list of §13 task numbers (must be a contiguous run, or a `[P]` cohort).
- **Checkpoint command:** the bash literal block — typically the e2e probe nearest the behaviour. The advisor runs this verbatim against the worktree branch via `/brehon-verify`.
- **Expected output:** the literal output line confirming success (e.g. `1 passed; 0 failed`).
- **Brief-Scope outputs to verify:** bulleted list of `<file>` + structural-pattern descriptors. Examples:
  - "contains `<symbol>` declaration" → `rg -q 'pub (const|fn|struct|trait|enum) <symbol>'`
  - "re-exports `<symbol>`" → `rg -q 'pub use .*<symbol>'`
  - "test `<test_fn>` exists in `<test_file>`" → `rg -q 'fn <test_fn>\b' <test_file>`
  - "migration `<id>__<name>` runs forward+backward" → check `up.sql` + `down.sql` exist + non-empty + (optionally) round-trip script

A small phase (1-3 tasks) ships a **single story** whose checkpoint is the phase-as-a-whole — back-compatible with current plans. Phases with 4+ tasks should ship 2-3 stories.

**Refusal:** if §16a Brief-Scope outputs name a file or symbol that the composing tasks' IMPLEMENT lists don't produce, the story is mis-mapped — the impl-task agent surfaces this as a DQ pending entry, the planner retrofits.

**How to apply (advisor side — see `.claude/commands/brehon-verify.md`):**

After every cohort + barrier task in §13 has shipped commits and before queueing `bm-merge`:

1. Run `/brehon-verify <phase>`.
2. For each story, the command:
   - Confirms Brief-Scope outputs exist on `phase-<phase>` (`git show origin/phase-<phase>:<file>`).
   - Runs the structural-pattern check via `rg -q '<pattern>'` against the file content.
   - Runs the checkpoint command against the worktree branch.
3. Each story is classified ✓ / ✗-phantom / ✗-regression / [malformed].
4. Report at `.claude/PRPs/reports/<phase>-verify.md` is committed.
5. Any phantom or regression triggers catch-fire (per advisor-orchestrator.md).
6. All ✓ → merge-confirm gate.

**Refusals:**

- Never modify a phantom file from the advisor session (advisor never authors content). Queue a fix-in-PR impl-task instead.
- Never run `/brehon-verify` while a Junior impl-task is running on the same phase (race).
- Never silently skip a `[malformed]` story — file a DQ asking the planner to retrofit.

**Symptom to recognise in retrospect:** a CR comment along the lines of "this task said it added X but I don't see X anywhere" — that's a phantom that `/brehon-verify` would have caught pre-merge. If you see one, the parent §16a was either omitted (legacy plan) or under-specified (Brief-Scope output descriptor too vague to mechanically verify). Backfill the lesson into the planner's contract.

**Generalizes to:** any artifact-producing workflow where "task complete" is signaled by commit + tests pass, but the commit might omit one of several planned outputs. The structural-pattern check (file present + pattern matched) is the cheapest reliable phantom-detector that doesn't require running the full test suite.
