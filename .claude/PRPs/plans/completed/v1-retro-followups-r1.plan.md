# v1-retro-followups-r1 — session-retro proposal followups

**Source:** `.claude/PRPs/reports/session-retro-2026-05-22-v1-dq-schema-r1-cohort-2-ship.md` §"What to change" + §"Promotion candidates"
**Trunk base:** `c858aa7ab` (or later — verify at start)
**Working branch:** `governance-v0` (direct-commit per `.claude/rules/phase-branch.md` — meta-only, no Rust, no CR-value)
**Ship discipline:** advisor-direct authorship ELIGIBLE per Option B precedent (changes are mechanical, plan §11 enumerates exact edits, scope is `.claude/` meta-only)
**Complexity score:** 0/10 — three trivial doc additions, all under 30 lines of total diff
**Estimated wall-clock:** ~15 min if advisor-direct; ~30 min if Junior-dispatched (Cohort 1 [P], 3 tasks file-disjoint)

## 1. Summary

Three concrete findings from the v1-dq-schema-r1 cohort-2 session retro:

1. **Pre-commit hygiene in canonical checkout** — extend `feedback_multi_lane_worktree_discipline.md` with a sub-section requiring `git status --short` before every `git add` in a checkout that may be shared with another session. Recurrence is 1× this session + ≥2× prior (mixed-scope retro commit `c858aa7ab` picked up 3 pre-staged files from a concurrent advisor session). Pattern documented but not previously codified as pre-commit discipline.

2. **Empirical pre-merge validation in impl-task contract** — add a one-line entry to `.claude/agents/impl-task.md` task-0 checklist requiring state-dependent logic changes (sort keys, dedupe rules, format converters) be run against live target state before commit. The plan's example may have an empty-corpus blind spot. Validated this session — Task 3 of v1-dq-schema-r1 found a hidden `TypeError` the plan's empty-corpus example would not have surfaced.

3. **Pre-compact handover file discipline** — extend `.claude/rules/advisor-orchestrator.md` "Polling loop" or "Catch-fire procedures" with a discipline note that ANY session likely to span `/compact` or session-end should author a pre-compact handover file at `.claude/PRPs/handovers/<phase>-<scope>-<date>.md` BEFORE the compact. Validated this session — the handover file enabled first-pass resume in 3 tool calls.

## 2. Source

Session retro: `.claude/PRPs/reports/session-retro-2026-05-22-v1-dq-schema-r1-cohort-2-ship.md`
PMD evals: 464 (per-task retro v1-dq-schema-r1), 465 (session retro)

## 3. Problem statement

The three findings are independent discipline gaps that each cost wall-clock minutes when they fire:

| # | Gap | Cost when fires | Recurrence |
|---|---|---|---|
| 1 | `git add` without `git status` check in shared checkout | ~5 min to investigate mixed-scope commit + decide whether to revert (force-push is forbidden) | 1× this session + ≥2× prior |
| 2 | Plan-example-only validation misses live-corpus traps | Hidden `TypeError`/silent-degradation discovered late or in production (Task 3 caught pre-merge — would have been a broken resolver) | 1× this session + multiple prior per `pattern_test_against_reality_not_syntax` |
| 3 | No pre-`/compact` handover → context-recovery overhead post-compact | ~10-30 min of conversation re-reading + state reconstruction per resume | 1× this session as the *positive* case (handover existed); recurrence as a negative case across prior `/compact` sessions |

None ship Rust, none touch migrations, none touch tests. All three are documentary discipline reinforcements in `.claude/` artifacts that already exist.

## 4. Solution statement

Three additive doc edits in three files. No deletions. No renames. No new files. Each edit is self-contained (no cross-file references between the three edits).

After this sub-phase ships:
- The multi-lane discipline lesson explicitly names the `git status` pre-commit check.
- The impl-task agent contract task-0 checklist names the live-state validation step.
- The advisor-orchestrator polling-loop section names the pre-compact handover discipline.

## 5. Metadata

```yaml
phase_slug: v1-retro-followups-r1
phase_branch: governance-v0    # direct-commit; no phase branch cut
cohort_pattern: optional        # all three tasks [P] file-disjoint if dispatched to Junior
complexity_score: 0
shape: pre-Shape-G              # no cargo, no tests, no e2e — pure docs
parallelism: 3-way [P] possible (Task 1, 2, 3 all file-disjoint)
estimated_wallclock_min:
  advisor_direct: 15
  junior_dispatch: 30
```

### 5.1 Complexity factor breakdown

- **Files touched:** 3 (one per task)
- **Crate touches:** 0 (no Rust)
- **Migration count:** 0
- **e2e edits:** 0
- **Cargo invocations:** 0 (plan §15 DoD is grep-only)
- **Cross-crate impact:** 0
- **Risk class:** doc-only

### 5.2 Per-task complexity ceiling

N/A — no Sonnet impl-tasks targeted; advisor-direct preferred.

## 6. Relationship to other v1-* sub-phases

- **v1-dq-schema-r1 (shipped `c858aa7ab` 2026-05-22)** — this sub-phase's session retro produced these three findings. Direct successor.
- **v1-federation-inbound-c (in flight, concurrent session)** — orthogonal; this plan touches no files on the fed-in-c critical path.
- **DQ #338 daemon-bug (pending, concurrent session handling)** — orthogonal; this plan does not touch daemon code.

## 7. Preflight guardrails inherited from prior phases

- **Multi-lane worktree discipline** (per `.claude/rules/multi-lane-worktree.md`) — this sub-phase's commits land on `governance-v0` (canonical checkout meta-edits per Hard Refusal #2 carve-out). No phase branch cut.
- **Direct-commit policy** (per `.claude/rules/phase-branch.md`) — commits to `governance-v0` directly because diff is `.claude/`-only and CR review would be net-noise.
- **Attribution integrity** (per `.claude/rules/decision-queue.md`) — every commit subject matches `^(chore|docs)\((advisor|decision-queue)\)`; `answered_by: "advisor"` permitted only for an advisor session committing inline.
- **Concurrent-session awareness** — per Hard Refusal #6 of `.claude/rules/multi-lane-worktree.md`, this plan's commits use the atomic read-mutate-commit protocol (`git fetch` → read → re-compute → mutate → verify → `git add` → commit → push as single sequence). Mandatory because the canonical checkout may be shared.

## 8. Flow design

```
Task 0 (pre-flight) → Task 1 [P] → Task 2 [P] → Task 3 [P] → Task 4 (retro)
                       ↓             ↓             ↓
                   (file-disjoint; safe to parallel-dispatch OR advisor-direct serial)
```

Three parallel tasks, no inter-task dependencies, no shared files. Cohort dispatch eligible but optional.

## 9. Mandatory reading

Before starting Task 1, 2, or 3, the implementer (advisor or Junior) reads:

1. The session retro that produced these findings: `.claude/PRPs/reports/session-retro-2026-05-22-v1-dq-schema-r1-cohort-2-ship.md`
2. The current state of the target file for the task being implemented:
   - Task 1 → `.claude/lessons/feedback_multi_lane_worktree_discipline.md`
   - Task 2 → `.claude/agents/impl-task.md`
   - Task 3 → `.claude/rules/advisor-orchestrator.md`
3. **Task-1-specific:** `.claude/rules/multi-lane-worktree.md` (the canonical file Task 1's edit cross-references)
4. **Task-2-specific:** `.claude/lessons/pattern_test_against_reality_not_syntax.md` (the already-promoted pattern Task 2's edit reinforces)
5. **Task-3-specific:** any prior `.claude/PRPs/handovers/*.md` to confirm the location convention (Task 3's edit names this path)

## 10. Patterns to mirror

### 10.1 Lesson-extension shape (Task 1)

When extending an existing `.claude/lessons/feedback_*.md` file with a new sub-section, the mirror is the recent extension of `feedback_cohort_dq_id_collision.md` by v1-dq-schema-r1 Task 2 (commit `1803b8546`): a new H2 `## Status (post-<sub-phase>, <date>)` § appended at the top of the file, with a dated lead-in and a citation to the source retro. Mirror that shape: H2 `## Status (post-v1-retro-followups-r1, 2026-05-22)` + dated paragraph + citation to this retro.

### 10.2 Agent-contract-extension shape (Task 2)

When adding to an existing `.claude/agents/*.md` task-0 checklist, the mirror is the recent v1-dq-schema-r1 Task 2 additions of `## DQ schema-v3 (post-v1-dq-schema-r1)` sections to all four agent files (commit `1803b8546`). Mirror that shape: new H2 `## Live-state validation (post-v1-retro-followups-r1)` § with one-paragraph discipline note + the one-line checklist item to add.

### 10.3 Rule-section-extension shape (Task 3)

When adding to `.claude/rules/advisor-orchestrator.md`, the mirror is the recent extension at the §1 "Polling loop" bullet list (the SessionStart canonical-PMD guard bullet + the SessionStart multi-lane check bullet, both added 2026-05-22). Mirror that shape: a new bullet in §1 "Polling loop" with `**Pre-compact handover discipline (post-v1-retro-followups-r1):**` lead-in + one-paragraph discipline note.

## 11. Files to change

```yaml
modifies:
  - .claude/lessons/feedback_multi_lane_worktree_discipline.md     # Task 1
  - .claude/agents/impl-task.md                                    # Task 2
  - .claude/rules/advisor-orchestrator.md                          # Task 3
creates:
  - .claude/PRPs/reports/v1-retro-followups-r1-retro.md           # Task 4 (retro file itself)
```

No other files touched. No deletions. No renames.

## 12. NOT building in v1-retro-followups-r1

- No new lesson file (the change to Task 1's target IS the new sub-section — no separate `feedback_*.md` is authored).
- No PreToolUse / PostToolUse hook to mechanically enforce `git status` before `git add` (deferred — see retro Decisions to revisit; that's a heavier mechanism).
- No changes to `.claude/agents/{planning,bm-task,ci-watcher}.md` — Task 2 only touches `impl-task.md` because that's the role whose plan-example-only validation triggered the finding.
- No `/auto-phase` state machine changes (the auto-phase skill is orthogonal to these three findings).
- No DQ schema changes (v3 just shipped; this sub-phase is operationally orthogonal).
- No Junior dispatch required (advisor-direct ELIGIBLE per Option B precedent; the implementer chooses).

## 13. Step-by-step tasks

### Task 0: Pre-flight (inline — advisor-direct or Junior task-0)

**ACTION:** verify clean working tree on `governance-v0`, fetch + ff origin, confirm DQ #338 still pending under concurrent session (not consumed elsewhere), confirm no concurrent session has staged files in the canonical checkout's index.

**FILES (machine-parseable):**
```yaml
creates: []
modifies: []
requires: []
```

**Commands:**
```bash
cd /c/Users/barri/Developer/brehon-fork
git fetch origin governance-v0
git log governance-v0..origin/governance-v0 --oneline | head -5    # expect empty (already current) or fast-forwardable
git status --short                                                  # MUST be empty before any commit (per Hard Refusal #6 multi-lane atomic protocol)
git ls-files --error-unmatch .claude/decision-queue.json && python -c "import io, json; d=json.load(io.open('.claude/decision-queue.json', encoding='utf-8')); print('pending:', [e['id'] for e in d.get('pending',[])])"
# EXPECT: pending: [338]  (or whatever the concurrent session has reduced it to)
git worktree list   # surface other active worktrees if any
```

**GOTCHA:** if `git status --short` shows pre-staged files, STOP — they belong to another session. Surface to user; do NOT proceed to Task 1 until index is clean OR the implementer is the owning session.

### Task 1 [P]: Extend `feedback_multi_lane_worktree_discipline.md` with pre-commit hygiene sub-section

**ACTION:** append a new H2 sub-section `## Pre-commit hygiene in canonical checkout (post-v1-retro-followups-r1, 2026-05-22)` to `.claude/lessons/feedback_multi_lane_worktree_discipline.md`. Documents the `git status --short` → `git diff --cached` pre-commit check discipline.

**FILES (machine-parseable):**
```yaml
modifies:
  - .claude/lessons/feedback_multi_lane_worktree_discipline.md
creates: []
requires:
  - task: 0
    reason: "Working tree must be clean before any commit per Hard Refusal #6 multi-lane atomic protocol."
```

**Discipline:** [P] (file-disjoint from Task 2 and Task 3)

**IMPLEMENT (file 1 of 1):** in `.claude/lessons/feedback_multi_lane_worktree_discipline.md`, append a new H2 § at end of file (or before the final "See also" / references § if one exists). Content shape:

```markdown
## Pre-commit hygiene in canonical checkout (post-v1-retro-followups-r1, 2026-05-22)

When the canonical `brehon-fork` checkout is shared with a concurrent advisor session (the working-tree-and-index pair are shared in a non-worktree checkout), the index can contain files staged by the OTHER session that the current session is unaware of. A `git add <single-file>` does NOT clear other staged files — it adds to whatever is already staged. The next `git commit` then folds those orphan-staged files into the commit, producing a mixed-scope commit whose subject does not advertise its full contents.

**Why this matters:** `git push` to `governance-v0` is irreversible without force-push (forbidden by `no-destructive-defaults.md`). A mixed-scope commit cannot be cleanly rewritten after push; the commit history reader can only know what's inside by reading the diff, not the subject. This violates commit-hygiene patterns documented in `feedback_commit_hygiene_lockfiles_and_task_labels.md`.

**How to apply (mandatory before every `git add` in the canonical checkout):**

1. `git status --short` — confirm the unstaged AND staged sets match the planned commit. If unexpected files appear in either column, STOP and investigate (likely concurrent-session work-in-progress).
2. If `git status` is clean except for the file(s) the current session authored: proceed with `git add <files>` + `git commit`.
3. If `git status` shows pre-staged files from another session: `git diff --cached` to confirm content; surface to user via the runlog or AskUserQuestion; do NOT silently fold them into an unrelated commit.
4. Post-`git add`, post-`git commit`: `git show --stat HEAD` to verify the commit-history reader will see exactly what the commit body advertised.

**Detection:** a commit whose subject names ONE artifact (e.g. "task 4 — four-role retro") but whose `--stat` shows >1 unrelated file is a process miss. Retro flags it. Future PostToolUse hook on `git commit` could scan staged-vs-subject for divergence but is heavier than the manual check.

**See also:**
- `feedback_commit_hygiene_lockfiles_and_task_labels.md` — commit-subject-matches-content discipline
- `.claude/rules/multi-lane-worktree.md` §"Hard refusals" #6 — atomic read-mutate-commit protocol
```

**GOTCHA:** do NOT duplicate content from the parent file's existing §"Hard refusals" #6 — the new sub-section EXTENDS Hard Refusal #6 with the pre-`git add` step; the parent #6 already covers the atomic-commit protocol. Cross-reference, don't duplicate.

**VALIDATE:**
```yaml
commands:
  - "test -f .claude/lessons/feedback_multi_lane_worktree_discipline.md"
  - "grep -c '^## Pre-commit hygiene in canonical checkout (post-v1-retro-followups-r1' .claude/lessons/feedback_multi_lane_worktree_discipline.md"
  - "grep -c 'git status --short' .claude/lessons/feedback_multi_lane_worktree_discipline.md"
```

**EXPECT:** file exists; grep counts ≥1 each.

### Task 2 [P]: Add live-state validation note to `.claude/agents/impl-task.md`

**ACTION:** append a new H2 § `## Live-state validation (post-v1-retro-followups-r1, 2026-05-22)` to `.claude/agents/impl-task.md`. Documents the discipline of running new state-dependent logic against live target state before commit, citing the v1-dq-schema-r1 Task 3 TypeError-catch as the recurrence evidence.

**FILES (machine-parseable):**
```yaml
modifies:
  - .claude/agents/impl-task.md
creates: []
requires:
  - task: 0
    reason: "Working tree must be clean."
```

**Discipline:** [P] (file-disjoint from Task 1 and Task 3)

**IMPLEMENT (file 1 of 1):** in `.claude/agents/impl-task.md`, append a new H2 § at the appropriate place (after the existing task-0 checklist sections, before any "See also" section). Content shape:

```markdown
## Live-state validation (post-v1-retro-followups-r1, 2026-05-22)

For state-dependent logic changes — sort keys, dedupe rules, format converters, any function whose behavior depends on the shape of input data — run the new logic against the live target state BEFORE commit, NOT just against the plan's example.

**Why:** the plan's example may have an empty-corpus or single-shape blind spot. v1-dq-schema-r1 Task 3 surfaced a hidden `TypeError` in `resolve-dq-canonical.sh`'s sort key when run against the live 134-entry DQ (133 int + 1 str ids); the plan's example had only int ids and would have passed silently. Without the live-state check, the resolver would have crashed on every post-migration invocation.

**How to apply (add to per-task validation checklist):**

For any task whose IMPLEMENT files include logic touching state shape:
1. Identify the live target state (path, ref).
2. Construct a minimal Python (or shell) repro of the new logic against that state.
3. Run before `git commit`. If `TypeError` / `KeyError` / silent-degradation symptoms appear, the plan example missed a corpus shape — file `kind: blocker` DQ for advisor to extend the plan §13 with the missing case OR fix in same commit if scope permits.

**Related pattern:** `pattern_test_against_reality_not_syntax.md` (promoted; this § is one application of the broader pattern).
```

**GOTCHA:** the impl-task contract has multiple H2 sections; insert this new § ALPHABETICALLY adjacent to existing `## Live-*` sections if any, else at end before "See also". Do NOT relocate existing sections.

**VALIDATE:**
```yaml
commands:
  - "test -f .claude/agents/impl-task.md"
  - "grep -c '^## Live-state validation (post-v1-retro-followups-r1' .claude/agents/impl-task.md"
  - "grep -c 'pattern_test_against_reality_not_syntax' .claude/agents/impl-task.md"
```

**EXPECT:** file exists; grep counts ≥1 each.

### Task 3 [P]: Add pre-compact handover discipline bullet to `advisor-orchestrator.md`

**ACTION:** add a new bullet to `.claude/rules/advisor-orchestrator.md` §1 "Polling loop" documenting the pre-compact handover file discipline. Cite the v1-dq-schema-r1 Cohort 2 resume as the validating evidence.

**FILES (machine-parseable):**
```yaml
modifies:
  - .claude/rules/advisor-orchestrator.md
creates: []
requires:
  - task: 0
    reason: "Working tree must be clean."
```

**Discipline:** [P] (file-disjoint from Task 1 and Task 2)

**IMPLEMENT (file 1 of 1):** in `.claude/rules/advisor-orchestrator.md`, insert a new bullet at the end of the §1 "Polling loop" bullet list (the existing bullets cover session-start CWD check, polling cadence, status-transition routing, etc). Content shape:

```markdown
- **Pre-compact handover discipline (post-v1-retro-followups-r1, 2026-05-22):** any advisor session likely to span `/compact`, session-end, or context truncation MUST author a self-contained handover file BEFORE the boundary, at `.claude/PRPs/handovers/<phase>-<scope>-<date>.md`. The file's body MUST be readable by a future session with zero conversation context — it states the current sub-phase, the stage in the state machine, the last commit on the relevant branch, the next concrete action, and any cross-session dependencies (DQ pending entries by id, concurrent session activity). Validated by the v1-dq-schema-r1 Cohort 2 handover (`.claude/PRPs/handovers/v1-dq-schema-r1-cohort-2-handover-2026-05-22.md`) — enabled first-pass resume in 3 tool calls. Commit + push the handover file BEFORE invoking `/compact`; the handover lives on `governance-v0` even if the work is on a phase branch, so the canonical checkout always sees the latest.
```

**GOTCHA:** the §1 "Polling loop" bullet list has a specific order (CWD check → cadence → status routing → DQ reading patterns → SessionStart hooks). The new bullet goes at the END of that list (after the SessionStart-multi-lane-check bullet added 2026-05-22). Do NOT relocate existing bullets.

**VALIDATE:**
```yaml
commands:
  - "test -f .claude/rules/advisor-orchestrator.md"
  - "grep -c 'Pre-compact handover discipline (post-v1-retro-followups-r1' .claude/rules/advisor-orchestrator.md"
  - "grep -c '\\.claude/PRPs/handovers/' .claude/rules/advisor-orchestrator.md"
```

**EXPECT:** file exists; grep counts ≥1 each.

### Task 4: Retro

**ACTION:** author the four-role retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`. One commit on `governance-v0`.

**FILES (machine-parseable):**
```yaml
creates:
  - .claude/PRPs/reports/v1-retro-followups-r1-retro.md
modifies: []
requires:
  - task: 1
  - task: 2
  - task: 3
```

**Discipline:** Non-[P]. Last task in plan.

**IMPLEMENT (file 1 of 1):** in `.claude/PRPs/reports/v1-retro-followups-r1-retro.md`, author the retro. Required structure: H1 title + four H2 §sections (What surprised us / What to change / What to carry forward / Per-role signals) + four H3 sub-§sections under Per-role signals (Advisor / Planning / Impl / BM — BM stated N/A since direct-commit, no PR).

**GOTCHA:** No BM activity; explicitly state "### BM\n\nN/A this phase (no PR / no bm-cut / no bm-pr / no bm-merge; direct-commit on governance-v0)."

**VALIDATE:**
```yaml
commands:
  - "test -f .claude/PRPs/reports/v1-retro-followups-r1-retro.md"
  - "grep -c '^# Retro: v1-retro-followups-r1' .claude/PRPs/reports/v1-retro-followups-r1-retro.md"
  - "grep -c '^## What surprised us' .claude/PRPs/reports/v1-retro-followups-r1-retro.md"
  - "grep -c '^## What to change' .claude/PRPs/reports/v1-retro-followups-r1-retro.md"
  - "grep -c '^## What to carry forward' .claude/PRPs/reports/v1-retro-followups-r1-retro.md"
  - "grep -c '^### Advisor' .claude/PRPs/reports/v1-retro-followups-r1-retro.md"
  - "grep -c '^### Planning' .claude/PRPs/reports/v1-retro-followups-r1-retro.md"
  - "grep -c '^### Impl' .claude/PRPs/reports/v1-retro-followups-r1-retro.md"
  - "grep -c '^### BM' .claude/PRPs/reports/v1-retro-followups-r1-retro.md"
```

**EXPECT:** all greps ≥ 1.

## 14. Testing strategy

This sub-phase touches NO Rust. Layered testing:
- Grep checks (§15 below) per task.
- No compile, no e2e, no migration round-trip.

## 15. Validation commands (DoD)

### 15.1 Static analysis (per task)

Run after each task lands:

```bash
# Task 1
grep -c '^## Pre-commit hygiene in canonical checkout (post-v1-retro-followups-r1' .claude/lessons/feedback_multi_lane_worktree_discipline.md
grep -c 'git status --short' .claude/lessons/feedback_multi_lane_worktree_discipline.md

# Task 2
grep -c '^## Live-state validation (post-v1-retro-followups-r1' .claude/agents/impl-task.md
grep -c 'pattern_test_against_reality_not_syntax' .claude/agents/impl-task.md

# Task 3
grep -c 'Pre-compact handover discipline (post-v1-retro-followups-r1' .claude/rules/advisor-orchestrator.md
grep -c '\.claude/PRPs/handovers/' .claude/rules/advisor-orchestrator.md

# Task 4
grep -c '^# Retro: v1-retro-followups-r1' .claude/PRPs/reports/v1-retro-followups-r1-retro.md
```

EXPECT: every grep ≥ 1.

### 15.2 Cross-cutting verification

After all 4 tasks land:

```bash
git log governance-v0 --oneline | head -5
# EXPECT: top 4 commits are chore(advisor): v1-retro-followups-r1 task 1..4
git diff <pre-sub-phase-sha>..governance-v0 --stat
# EXPECT: 4 files changed; modifications under .claude/lessons/, .claude/agents/, .claude/rules/, .claude/PRPs/reports/
```

## 16. Acceptance criteria

- [ ] Task 1 lands new H2 § in `feedback_multi_lane_worktree_discipline.md`; grep validates.
- [ ] Task 2 lands new H2 § in `.claude/agents/impl-task.md`; grep validates.
- [ ] Task 3 lands new bullet in `.claude/rules/advisor-orchestrator.md` §1; grep validates.
- [ ] Task 4 lands retro file; all 9 structural grep checks pass.
- [ ] Origin `governance-v0` advanced by exactly 4 commits (one per task) OR 3 commits if Tasks 1+2+3 dispatched as a single cohort by Junior (one finalize-merge commit per [P] task) — depends on ship discipline chosen.
- [ ] No phase branch cut. No PR.
- [ ] Plan §16a Stories 1-4 all `[done]`.

## 16a. Stories (independently-testable behaviour units)

### Story 1: Multi-lane lesson documents pre-commit hygiene

- **Composing tasks:** Task 1.
- **Checkpoint command:**
```bash
grep -c '^## Pre-commit hygiene in canonical checkout' .claude/lessons/feedback_multi_lane_worktree_discipline.md
grep -c 'git status --short' .claude/lessons/feedback_multi_lane_worktree_discipline.md
grep -c 'git diff --cached' .claude/lessons/feedback_multi_lane_worktree_discipline.md
```
- **Expected:** each ≥1.

### Story 2: impl-task contract requires live-state validation

- **Composing tasks:** Task 2.
- **Checkpoint command:**
```bash
grep -c '^## Live-state validation' .claude/agents/impl-task.md
grep -c 'pattern_test_against_reality_not_syntax' .claude/agents/impl-task.md
grep -c 'TypeError' .claude/agents/impl-task.md
```
- **Expected:** each ≥1.

### Story 3: Polling loop documents pre-compact handover discipline

- **Composing tasks:** Task 3.
- **Checkpoint command:**
```bash
grep -c 'Pre-compact handover discipline' .claude/rules/advisor-orchestrator.md
grep -c '\.claude/PRPs/handovers/' .claude/rules/advisor-orchestrator.md
grep -c 'Cohort 2 handover' .claude/rules/advisor-orchestrator.md
```
- **Expected:** each ≥1.

### Story 4: Retro file present and structured

- **Composing tasks:** Task 4.
- **Checkpoint command:** §13 Task 4 VALIDATE block.
- **Expected:** all 9 greps ≥1.

## 17. Completion checklist

- [ ] Task 0 pre-flight clean (working tree empty, origin in sync, DQ #338 still pending under concurrent session ownership).
- [ ] Task 1 committed.
- [ ] Task 2 committed.
- [ ] Task 3 committed.
- [ ] Task 4 retro committed.
- [ ] §15 validation green at every gate.
- [ ] §16a stories 1-4 all `[done]`.
- [ ] GitHub issue #142 closure status: already ready (closed when v1-dq-schema-r1 shipped; this sub-phase is orthogonal followup).

## 18. Risks and mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| Concurrent session edits same target file between Task 0 and Task N | low (concurrent session is on fed-in-c critical path; this sub-phase's targets are not on that path) | Atomic read-mutate-commit per multi-lane Hard Refusal #6. |
| `git status` shows pre-staged files at Task 0 | medium (this is exactly the recurrence pattern Task 1 documents) | STOP at Task 0; surface to user; do NOT proceed. This is the dogfood test of the lesson Task 1 ships. |
| Daemon-bug DQ #338 fires during Junior dispatch | medium (if Junior-dispatched) | Choose advisor-direct ship discipline (Option B precedent) — eliminates daemon path entirely. |
| Plan template heaviness fights the small diff | low (this plan deliberately uses a compact section set, not the full 41-section template) | Plan is calibrated to the scope; no full-template fields like §10.1 multi-pattern mirrors are populated. |

## 19. Notes

- This plan is the structured form of the three "What to change" findings in `.claude/PRPs/reports/session-retro-2026-05-22-v1-dq-schema-r1-cohort-2-ship.md`. Implementation in a future session per user direction 2026-05-22.
- Advisor-direct ship is RECOMMENDED (Option B precedent from v1-dq-schema-r1 Task 3): the three edits are mechanical, the plan §13 enumerates exact content, and Junior dispatch adds daemon-bug DQ #338 risk for no parallelism gain (the three tasks are too small to benefit from [P]).
- If Junior-dispatched, the three [P] tasks file-disjoint; cohort dispatch is structurally safe per `.claude/rules/advisor-orchestrator.md` §4 cohort dispatch sequence.
- No DQ entries authored by this plan. No clarify pass needed — the source retro IS the clarification.
- No PR. No bm-cut. No bm-pr. No bm-merge. Direct-commit per `phase-branch.md`.
