---
role: bm-task
verb: bm-cut
args: v1-JM-d
phase: v1-JM-d
created: 2026-04-27
---

# Brief — `bm-cut v1-JM-d`

## 1. Role + dispatch line

`[role:bm-task] bm-cut v1-JM-d — see .claude/PRPs/briefs/jm-d-bm-cut-1.md`

You are the **bm-task** subagent. Execute the `bm-cut` verb with argument `v1-JM-d`.
This will cut a new local-only phase branch `phase-v1-JM-d` off `governance-v0`.

## 2. Scope

**Produce:**
- New local branch `phase-v1-JM-d` cut from current `origin/governance-v0` HEAD.
- Append a new section to `.claude/runlog/v1-JM-d-runlog.md` (create if missing) per the bm-cut Phase 4 template.
- A "Branch cut" 4-section summary returned per `.claude/commands/bm/bm-cut.md` Phase 5.

**Do NOT:**
- Push the new branch (per Phase 3 — pushing an empty branch creates a useless remote ref with no PR target; defer to first impl commit + `bm-push`).
- Author any plan content, impl code, or `crates/**` changes.
- Open a PR. That's `bm-pr` later, after impl tasks land commits.
- Touch `governance-v0` directly (hard refusal #3 in the bm-task agent rules).

**Commit only:**
- `.claude/runlog/v1-JM-d-runlog.md` (append-only). Commit on `phase-v1-JM-d` after the branch is cut.

## 3. Required reading

Read in this order before executing:

1. `.claude/commands/bm/bm-cut.md` — the operational script (Phase 0–5). Follow it step by step.
2. `.claude/rules/branch-manager.md` — operating rules, hard refusals, autonomy bounds.
3. `.claude/rules/phase-branch.md` — phase-branch flow.
4. `.claude/agents/bm-task.md` — your own contract (no-AskUserQuestion, DQ-blocked stop pattern, lesson-trailer convention).
5. **Lesson scan** (Glob `.claude/lessons/` and Read any with filename keywords matching `bm-cut`/`phase-branch`):
   - `feedback_pr_per_phase.md` (PR-flow per phase)
   - `feedback_branch_manager_pm_split.md` (BM ≠ planner)
   - Any other `feedback_coderabbit_*` or `feedback_phase_*` if present.

## 4. Constraints

- **Plan file precondition (Phase 2):** verify `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` exists on `governance-v0` before cutting. (Confirmed present at trunk HEAD as of 2026-04-27 — but re-verify per the script.)
- **Trunk-clean precondition (Phase 1):** the `bm-cut` script does its own `git fetch origin` + `git status --short` checks. Honor any STOP conditions; if trunk is dirty or ahead of remote, stop and file a DQ pending entry per `.claude/agents/bm-task.md` confirmation protocol — do **not** ask interactively (you're in `-p` mode).
- **No-AskUserQuestion:** all "Yes asks first" branches in the bm-cut script (e.g. "currently on a `phase-*` branch with unpushed commits") become DQ-stop instead. Write a `pending` DQ entry with `from: "bm"`, commit + push it, return `blocked-on-DQ-#<id>`.
- **Linux discipline:** you run on the EliteDesk. Use Linux tooling (`bash`/`git`/`gh`). No `.bat` wrappers.
- **Pipes mask exit codes** (`feedback_pipes_mask_exit_codes.md`): never pipe `git`/`gh` through `tail`/`grep` when you need to know if it succeeded. Capture to file or check `$?` directly.
- **Commit message format:** for the runlog commit, use `chore(bm): cut phase-v1-JM-d off governance-v0`. Per `feedback_pr_per_phase.md`, this commit lands directly on `phase-v1-JM-d` (the new phase branch), never on `governance-v0`.
- **Lesson trailer (optional):** if during execution you discover something a future `bm-cut` would have wanted to know (a script step that silently misbehaved, a missing precondition the script doesn't check, etc.), end the runlog commit body with a single `LESSON:` line per `feedback_junior_pmd_write_convention.md`. Skip the trailer for routine progress.

## 5. Expected output

A 4-section summary per the bm-cut script Phase 5:

```
## /bm-cut complete

**Branch cut:** phase-v1-JM-d
**Off:** governance-v0 @ <short-sha>
**Plan file on trunk:** .claude/PRPs/plans/v1-jury-mechanics-d.plan.md
**Pushed?:** No (deferred to first commit + /bm-push)
**Runlog:** .claude/runlog/v1-JM-d-runlog.md updated

### Hand-off to impl session

Impl can now run `/prp-implement` against the plan above. BM will pick
up the branch on `/bm-status` once commits land.
```

The advisor's polling loop will read this and queue the first `impl-task` for plan task 1.
