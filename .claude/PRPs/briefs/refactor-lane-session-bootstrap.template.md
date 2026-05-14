---
purpose: Shared bootstrap template for the 6 refactor lane sessions
authored: 2026-05-14
related:
  - .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md (audit ground-truth)
  - .claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md (execution plan)
  - .claude/lessons/feedback_concurrent_advisor_session_collision.md (PMD #302; single-purpose session discipline)
---

# Refactor lane session bootstrap — shared template

This is the canonical template every refactor-lane session reads as its
first action. The 6 lane-specific bootstrap files
(`refactor-<area>-bootstrap.md`) reference this template + supply
lane-specific substitutions.

A fresh Claude Code session opens in the worktree-specific CWD
(`C:/Users/barri/Developer/brehon-fork-refactor-<area>`), is given the
path to its lane bootstrap file, reads it, then reads THIS template
file, then executes.

---

## Session identity

You are a fresh Claude Code session in a brehon-fork refactor-lane
worktree. Your single task is to dispatch ONE refactor PR from the
audit's fix-before-next-phase tier. When the PR is open and the
validate-pending DQ entry is raised, you go idle — no follow-up work
in this session.

You are NOT the canonical governance-v0 advisor session. You are
NOT the ci-watcher. You are a single-purpose dispatch session,
scoped to your lane.

---

## Pre-flight discipline (run before any state-changing action)

Three checks in order. If any fail, STOP and surface to user.

### Check 1 — Lane CWD verification

```bash
pwd
git worktree list
git branch --show-current
```

Expected:
- `pwd` matches `<lane-worktree-path>` from your lane bootstrap.
- `git worktree list` includes both the canonical `brehon-fork` AND
  your lane worktree.
- Current branch is `governance-v0` initially (bm-cut will move to
  the chore branch).

If `pwd` is the canonical `brehon-fork` checkout: STOP. You are in
the wrong session. Per `feedback_multi_lane_worktree_discipline.md`,
each session is one CWD, one lane. Close this session; open a fresh
one in the correct worktree.

### Check 2 — Concurrent-session awareness

Per `feedback_concurrent_advisor_session_collision.md` (PMD #302):
the canonical session is read-only during refactor execution. Other
refactor-lane sessions may also be active in parallel — verify they
are each in their own worktree, NOT in this one.

```bash
ls C:/Users/barri/Developer/brehon-fork-refactor-*/ -d 2>&1
```

Expected: each lane has its own worktree directory. You should NOT
see another session writing to your lane's CWD. If you do
(e.g. `git status --short` shows mid-edit changes you didn't make),
STOP and surface.

### Check 3 — Audit + execution plan exist on trunk

```bash
git fetch origin governance-v0
git show origin/governance-v0:.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md > /dev/null && echo "audit OK"
git show origin/governance-v0:.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md > /dev/null && echo "plan OK"
```

Both should print OK. If not, STOP — the parallel session's audit
push has not yet landed on origin; you cannot start without the
ground-truth audit reference.

---

## Execution sequence

Each lane runs the same 5 phases. Read your lane bootstrap for
phase-specific substitutions (brief paths, branch name, finding
references).

### Phase 1 — bm-cut

Read your lane's bm-cut brief at the path named in your lane
bootstrap. Execute it per `.claude/commands/bm/bm-cut.md`:

1. Parse argument (chore type).
2. Verify trunk state.
3. Skip plan-file check (chore branch).
4. Cut branch locally (`git checkout -b chore/refactor-<area> governance-v0`).
5. Create runlog at `.claude/runlog/chore-refactor-<area>.md`.
6. Output Phase 5 summary.

Do NOT push the chore branch yet. The impl-task subagent's first
commit is what triggers the initial push.

### Phase 2 — impl-task dispatch

Read your lane's impl-task brief at the path named in your lane
bootstrap. The brief contains:
- Verbatim audit finding (the contract you implement against)
- Canonical mirror reference for the pattern to follow
- Pre-edit verification commands
- Edit specification (file + lines)
- Post-edit verification commands
- Validation gates
- COMMIT MESSAGE specification
- Out-of-scope enumeration

Execute the brief as authored. **Do not extend scope**. If
during execution you find something the brief did not anticipate,
file a DQ blocker per `.claude/rules/decision-queue.md` Recipe 1
and STOP. Do not improvise.

### Phase 3 — pre-push validation (mandatory)

Per `feedback_fix_impl_pre_push_cargo_check.md`. Before `git push`:

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/refactor-<area>-precheck.log 2>&1
status=$?
tail -20 .claude/PRPs/debug/refactor-<area>-precheck.log
[ $status -eq 0 ] || exit $status
```

For lanes that touch tests or testable code (PR-1, PR-6), also run
the appropriate cargo-test invocation per your brief's §2 validation
section. Per `.claude/rules/cargo-output-capture.md` —
capture-then-tail; never pipe cargo through tail/head.

If any gate fails: STOP, file a DQ blocker citing the failure mode,
do not push.

### Phase 4 — Shape G push + DQ raise (atomic)

Per `feedback_dq_raise_before_ci_watcher_queue.md`. Atomic ordering
is load-bearing:

1. Commit the refactor change with the brief's specified COMMIT
   MESSAGE.
2. Push to origin: `git push -u origin chore/refactor-<area>`.
3. Capture workflow_run_id:
   ```bash
   gh run list --repo barrie-cork/lemmy --branch chore/refactor-<area> --limit 1 --json databaseId
   ```
4. Write `kind: "validate-pending"` DQ entry per
   `.claude/rules/decision-queue.md` Recipe 1. Use Python with
   `json.dump(..., ensure_ascii=False, indent=2)`. Compute `next_id`
   spanning both `.claude/decision-queue.json` AND
   `.claude/decision-queue-archive-*.json`.
5. Commit + push the DQ entry to `chore/refactor-<area>` (same
   branch — DQ writes go to the worker branch, not trunk).
6. The lane session does NOT dispatch a ci-watcher. The canonical
   advisor session monitors validate-pending entries on its next
   poll and dispatches a ci-watcher.

### Phase 5 — Surface + go idle

Output a 3-line summary to the conversation:
```
Lane: chore/refactor-<area>
Refactor commit: <sha>
Validate-pending DQ: #<id> for workflow <run_id>
```

Then this session goes idle. No follow-up work. Wait for user
direction. The canonical session will poll DQ + dispatch
ci-watcher + handle bm-pr when validate-pending mutates to pass.

---

## Hard refusals

- **NEVER write `answered_by: "advisor"` to DQ.** You are a
  lane-dispatch session, not the advisor. Use `from: "impl"`,
  `answered_by: null` per Recipe 1.
- **NEVER edit files outside your lane's scope.** Your brief
  enumerates the exact files. Edits to any other file are
  out-of-scope and require a DQ blocker.
- **NEVER `cd` to another worktree.** One session, one CWD, one
  lane.
- **NEVER dispatch a Junior task or ci-watcher.** Out of scope.
- **NEVER open a PR.** That's the canonical session's job after
  validate-pending passes.
- **NEVER touch `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md`,
  `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md`,
  or any other lane's briefs.** Those are read-only references for
  your session.

---

## When to file a DQ blocker (not a self-resolve)

Per `.claude/rules/decision-queue.md` schema-v2 + Recipe 1:

- The audit finding's "current shape" doesn't match what you actually
  see in the file (baseline drift).
- The canonical mirror reference (e.g. `create_endorsement.rs:130-145`)
  has changed shape since the audit.
- A pre-push gate fails with a non-obvious cause.
- The brief's edit specification would cause a wider blast radius
  than scoped (e.g. need to touch a second file).
- You find a related issue NOT covered by your brief (file as a
  follow-up DQ for the next audit cycle, not as a scope expansion).

DQ blocker shape:
```json
{
  "id": <next_id>,
  "from": "impl",
  "kind": "blocker",
  "timestamp": "<ISO 8601 UTC>",
  "question": "<one-sentence question naming the issue>",
  "options": ["<option-a-short>", "<option-b-short>"],
  "context": "<what you checked; what you found; why this gates the refactor>",
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

Commit + push the DQ entry to `chore/refactor-<area>`, then go idle.
The canonical session will surface to user.

---

## Concurrent-session collision recovery

Per `feedback_concurrent_advisor_session_collision.md` (PMD #302):
if you notice the staging area contains files you did not add:

1. STOP. Do not commit.
2. `git restore --staged <path>` for each unwanted file.
3. Verify `git status --short` shows only your intended files.
4. Surface the unexpected files to user before any commit.

This should never happen if every refactor session is in its own
worktree, but the recovery is here in case.

---

## Where the canonical advisor session is

`C:/Users/barri/Developer/brehon-fork` — governance-v0, read-only
during your refactor execution. Polls DQ for validate-pending
mutations. Surfaces refactor PR-merge candidates when all 6 lanes
ship green. Does NOT write to your chore branch.

---

## Where the audit ground-truth lives

`.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md` —
read your specific finding section (your lane bootstrap will name
it) BEFORE making any edits. The audit's §5.1 cross-cutting note
("internal-inconsistency is the most actionable pattern") is the
framing every refactor uses. Don't expand scope beyond the
finding's specific text.

---

## See also

- `.claude/agents/impl-task.md` — subagent contract (one task,
  one commit, one feature)
- `.claude/rules/branch-manager.md` — chore branch file-ownership
- `.claude/rules/decision-queue.md` — DQ schema-v2 + Recipe 1 +
  attribution rules
- `.claude/rules/multi-lane-worktree.md` — worktree discipline
- `.claude/rules/cargo-output-capture.md` + `.claude/rules/no-cargo-output-paste.md`
  — capture-then-tail pattern; never paste cargo logs into commits
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md`
  — execution context (which PR you're working, what's parallel)
