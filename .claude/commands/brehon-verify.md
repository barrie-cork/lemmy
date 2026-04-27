---
description: Advisor-side post-impl, pre-merge spec-conformance check. Iterates plan §16a stories, confirms expected outputs exist on worktree branch, runs story checkpoints. Catches phantom completions before bm-merge.
argument-hint: <phase-slug, e.g. v1-JM-e>
---

<objective>
The `/brehon-verify` command runs after every `[P]`-cohort + barrier task in §13 has shipped commits, and before `bm-merge` is queued. It cross-checks the plan §16a stories against the actual worktree branch state, catching phantom completions (task marked done but expected output absent or empty) earlier than CR triage.

This is a spec-kit-derived pattern (see `feedback_brehon_verify_pre_merge.md`). It runs **only in the advisor session**.

**Hard precondition (per advisor-orchestrator.md "Stage-shape orchestration"):** every plan §13 task must have a commit on `phase-<phase>` (or its successor PR-targeted branch). The command refuses if any §13 task has no commit yet.
</objective>

<usage>
**`$ARGUMENTS`** = `<phase-slug>`. Examples:

- `/brehon-verify v1-JM-e`
- `/brehon-verify v1-JM-d` (re-verify after triage fixes)

The command derives the phase branch name (`phase-<phase>`) and the plan path (`.claude/PRPs/plans/v1-jury-mechanics-<letter>.plan.md` or similar — match by phase-slug pattern).

If `$ARGUMENTS` is empty: refuse with "specify a phase-slug".
</usage>

<workflow>

### Step 1: Locate inputs

For the phase-slug provided:

1. Plan file: search `.claude/PRPs/plans/*.plan.md` for one whose §5 Metadata `Phase:` matches the slug. If multiple match (rare), use the most recent mtime.
2. Phase branch: `phase-<phase>` (e.g. `phase-v1-JM-e`). Confirm it exists: `git fetch origin && git rev-parse --verify origin/phase-<phase>`.
3. Brief paths: `Glob .claude/PRPs/briefs/<phase>-impl-*.md` (impl-task briefs only — clarify is pre-planning, retro is reflective).

If any input is missing, refuse with the missing-input message and the path attempted.

### Step 2: Pre-flight refusals

Refuse to verify if:

- **Any §13 task has no commit on `origin/phase-<phase>`.** Check by `git log origin/phase-<phase> --oneline | grep -E '\(task <N>\)'`. If a task is missing its commit, the impl run is incomplete — do not phantom-check yet.
- **A Junior task is currently running on the phase.** Per `mcp__junior-brehon__list_tasks`. Wait or cancel before verify.
- **Plan has no §16a stories block.** This is a plan written before the spec-kit-pattern adoption — file a DQ pending entry asking for retrofit and skip verify (manual reconciliation only).
- **The phase branch has uncommitted state on the impl daemon's worktree.** Junior's finalize push should have flushed it; if it hasn't, surface as catch-fire per advisor-orchestrator.md catch-fire procedures.

### Step 3: Read plan §16a stories

Parse the plan file. Find `## 16a. Stories` and extract each `### Story N: <title>` block. For each story, capture:

- **Composing tasks:** integer list of §13 task numbers
- **Checkpoint command:** the bash literal block under "Checkpoint command:"
- **Expected output:** the literal under "Expected output:"
- **Brief-Scope outputs to verify:** bulleted list of `<file>` + structural-pattern descriptors

If any field is missing for a story, treat as `[malformed]` — surface to user as a DQ pending entry asking the planner to retrofit (do not skip silently — story-grain decomposition is load-bearing).

### Step 4: Per-story verification

For each story:

#### Step 4a: Output presence + structural-pattern check

For each Brief-Scope output:

```bash
# Confirm file exists on phase branch
git show origin/phase-<phase>:<file> > /dev/null 2>&1 || echo "PHANTOM: <file> absent"

# Confirm non-empty
[ "$(git show origin/phase-<phase>:<file> | wc -c)" -gt 0 ] || echo "PHANTOM: <file> empty"

# Confirm structural pattern (parsed from "X contains Y", "Y exists in X", "test Z exists in <test_file>", etc)
git show origin/phase-<phase>:<file> | rg -q '<pattern>' || echo "PHANTOM: <pattern> not found in <file>"
```

The structural pattern is the descriptor next to each Brief-Scope output bullet. Examples:
- "contains `<symbol>` declaration" → `rg -q 'pub (const|fn|struct|trait|enum) <symbol>'`
- "re-exports `<symbol>`" → `rg -q 'pub use .*<symbol>'`
- "test `<test_fn>` exists in `<test_file>`" → `rg -q 'fn <test_fn>\b' <test_file>` (parsed from descriptor)
- "migration `<id>__<name>` runs forward+backward" → check `migrations/<id>__<name>/up.sql` and `down.sql` both exist + non-empty + (optionally) run `bash scripts/brehon/migrate-roundtrip.sh <id>__<name>` if available

If a structural pattern can't be parsed mechanically, the planner left an under-specified descriptor — file a DQ pending entry asking for revision and treat the story as `[malformed]`.

#### Step 4b: Checkpoint command execution

Run the story's checkpoint command literally — the same way Junior would:

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server <test_fn_name> > .claude/PRPs/debug/<phase>-verify-story-<N>.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/<phase>-verify-story-<N>.log
```

Capture exit code separately (per `feedback_pipes_mask_exit_codes.md`). Any non-zero exit = ✗ for the story.

**Forbidden-window check:** if the checkpoint command is cargo-class and current UTC is in a forbidden window, defer verify to the next safe minute and log the deferral. Verify is not impl-task — it doesn't carry an `impl-task` task-0 pre-flight — but it consumes the same EliteDesk resources, so the same window discipline applies.

#### Step 4c: Story outcome

For each story, classify as:

- **✓** all outputs present + structural patterns matched + checkpoint exit 0
- **✗ regression** outputs present + structural patterns matched + **checkpoint exit non-zero** (suspected regression CR triage missed)
- **✗ phantom** any output absent, empty, or structural-pattern-failing
- **[malformed]** plan §16a entry under-specified — needs planner retrofit

### Step 5: Write verify report

Path: `.claude/PRPs/reports/<phase>-verify.md`. Format:

```markdown
# Verify report — <phase>

**Run at:** <ISO 8601 UTC>
**Phase branch:** `phase-<phase>` @ `<sha>`
**Plan:** `<plan-path>` @ `<sha>`
**Outcome summary:** N stories: <X>✓ <Y>✗-phantom <Z>✗-regression <W>[malformed]

---

## Story 1 — <title>

- **Composing tasks:** <list>
- **Outputs:** ✓ <output-1> | ✓ <output-2> | ✗ <output-3 — pattern not found>
- **Checkpoint:** ✓ exit 0 (`<test_fn>` 1 passed; 0 failed)
- **Outcome:** ✗ phantom (output-3 missing)

## Story 2 — …

…

---

## Required actions

(Filled in only if outcome is not all-✓:)

- **Phantoms:** Story <N> requires <action — typically queue a fix-in-PR impl-task targeting the missing output>
- **Regressions:** Story <M> checkpoint failed at <log-path>; surface to user via catch-fire per advisor-orchestrator.md
- **Malformed:** Story <K> §16a entry under-specified; queue a planner retrofit task
```

### Step 6: Commit + outcome routing

#### All stories ✓

Commit the report:

```
docs(advisor): brehon-verify <phase> — all stories ✓
```

Then advance the orchestrator stage to merge-confirm user gate.

#### Any phantom or regression

Commit the report:

```
docs(advisor): brehon-verify <phase> — <X> phantom, <Y> regression
```

Surface to user via catch-fire per `.claude/rules/advisor-orchestrator.md` "Catch-fire procedures". Do not queue bm-merge.

#### Any malformed

Commit the report:

```
docs(advisor): brehon-verify <phase> — <X> malformed §16a entries
```

File a DQ pending entry asking the planner to retrofit. The verify pass is incomplete until the planner ships a corrective commit.

### Step 7: Final report (return to user)

```
## Verify pass complete — <phase>

**Stories:** <total>
**✓:** <count>
**✗ phantom:** <count> (paths: <list>)
**✗ regression:** <count> (logs: <list>)
**[malformed]:** <count> (story names: <list>)
**Report:** `.claude/PRPs/reports/<phase>-verify.md` @ `<sha>`

**Merge-confirm gate:** CLEAR / BLOCKED-on-<reason>
```

</workflow>

<hard-refusals>

1. **Never queue `bm-merge`** while `/brehon-verify` shows any phantom or regression. The verify gate is mandatory before bm-merge confirm. Per advisor-orchestrator.md.

2. **Never modify `<file>` on the phase branch** to "fix" a phantom. The advisor never authors content. If a phantom needs fixing, file a DQ pending entry and queue a fix-in-PR impl-task.

3. **Never run `/brehon-verify` in parallel with a Junior impl-task on the same phase.** The phase branch is moving — verify would race. Refuse and tell the advisor to wait.

4. **Never silently skip a `[malformed]` story.** Every §16a entry that can't be mechanically parsed is a planner-side miss; surfacing it via DQ is the corrective signal.

5. **Never write the verify report to `.claude/PRPs/debug/`** (that directory is for Junior subagent capture). Reports go to `.claude/PRPs/reports/`.

6. **Never run cargo-class checkpoints during a forbidden window.** Defer the entire verify pass — partial verify is worse than deferred verify (a partial-pass report is misleading).

</hard-refusals>

<rationale>

### Why this command exists

The Brehon plan §16 acceptance criteria roll up at the phase level. CR triage catches regressions post-PR. Neither catches the case where a task's commit lands on the phase branch but the **expected output isn't actually there** — e.g. the impl agent committed everything except one of the three planned files, or the file landed but the planned symbol is absent (renamed, deferred, or simply forgotten).

`feedback_advisor_watchpoint_specificity.md` already requires watchpoints cite specific file:line. `/brehon-verify` is the **mechanical reconciliation** of those watchpoints + brief Scope + plan §13 IMPLEMENT lists against worktree branch state, run **before** CR has a chance to be the only line of defense.

Spec-kit calls this `/speckit.verify`. The Brehon adaptation:

- Reuses `git show <branch>:<file>` per the existing BM-task discipline (per `.claude/rules/branch-manager.md`-derived "reports from review-status tasks live on the task branch").
- Reuses `.claude/PRPs/reports/` for the output (existing convention).
- Reuses the catch-fire procedure for phantoms (advisor-orchestrator.md).
- Plugs into the existing four-bucket triage shape if `/brehon-verify` finds gaps that warrant it.

### Why post-impl, pre-merge

Earlier (post each task) would race with the impl daemon and produce flapping reports. Later (post-merge) means the bad commit is on `governance-v0`, requiring a revert. Pre-merge after all `[story:done]` checkpoints is the cheapest reliable point.

### When to skip /brehon-verify entirely

- **Plans without §16a stories block.** Pre-rule legacy plans get manual reconciliation; the rule applies forward-only (per memory-injection's forward-only consistency principle).
- **Single-story phases (1-3 tasks).** The phase-as-a-whole is the story; CR triage catches regressions; verify report is one row.
- **Retro-only commits.** No impl, no §13 tasks, nothing to verify.

### Re-running /brehon-verify

After fix-in-PR commits land on the phase branch, re-run `/brehon-verify` to confirm phantoms cleared. The report is committed at each run (not amended) — the audit trail matters.

</rationale>
