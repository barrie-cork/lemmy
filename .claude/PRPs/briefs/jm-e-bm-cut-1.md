---
role: bm-task
verb: bm-cut
args: v1-JM-e
phase: v1-JM-e
created: 2026-04-30
---

# Brief — `bm-cut v1-JM-e`

## 1. Role + dispatch line

`[role:bm-task] bm-cut v1-JM-e — see .claude/PRPs/briefs/jm-e-bm-cut-1.md`

You are the **bm-task** subagent. Execute the `bm-cut` verb with argument `v1-JM-e`.
This will cut a new local-only phase branch `phase-v1-JM-e` off `governance-v0`.

## 2. Scope

**Produce:**
- New local branch `phase-v1-JM-e` cut from current `origin/governance-v0` HEAD (`eacb6ed1e` at brief-write).
- Create `.claude/runlog/v1-JM-e-runlog.md` and append the bm-cut Phase 4 entry.
- A "Branch cut" 4-section summary returned per `.claude/commands/bm/bm-cut.md` Phase 5.

**Do NOT:**
- Push the new branch (per Phase 3 — pushing an empty branch creates a useless remote ref).
- Author any plan content, impl code, or `crates/**` changes.
- Open a PR — that's `bm-pr` after impl tasks land.
- Touch `governance-v0` directly (hard refusal #3).

**Commit only:**
- `.claude/runlog/v1-JM-e-runlog.md` on `phase-v1-JM-e`.

## 3. Required reading

1. `.claude/commands/bm/bm-cut.md` — operational script (Phase 0–5). Follow step-by-step.
2. `.claude/rules/branch-manager.md` — operating rules, hard refusals, autonomy bounds.
3. `.claude/rules/phase-branch.md` — phase-branch flow.
4. `.claude/agents/bm-task.md` — your contract (no-AskUserQuestion, DQ-blocked stop pattern).
5. **Lesson scan** (Glob `.claude/lessons/` and Read any with filename keywords matching `bm-cut`/`phase-branch`/`settings_json`):
   - `feedback_pr_per_phase.md`
   - `feedback_branch_manager_pm_split.md`
   - Any `feedback_settings_json_*` or `feedback_phase_*` if present.

## 4. Constraints

- **Plan file precondition (Phase 2):** verify `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` exists on `governance-v0` before cutting. Confirmed present at `eacb6ed1e` per advisor pre-flight 2026-04-30.
- **Trunk-clean precondition (Phase 1):** the script does its own `git fetch origin` + `git status --short`. Honour STOP conditions; if trunk is dirty or ahead of remote, file a `pending` DQ entry from `from: "bm"` per `.claude/agents/bm-task.md` confirmation protocol — do **not** ask interactively (you're in `-p` mode).
- **No-AskUserQuestion:** any "Yes asks first" branch in the script becomes a DQ-stop. Write a `pending` DQ from `bm`, commit + push it, return `blocked-on-DQ-#<id>`.
- **Linux discipline:** EliteDesk worker. Use `bash`/`git`/`gh`. No `.bat` wrappers.
- **Pipes mask exit codes** (`feedback_pipes_mask_exit_codes.md`): never pipe `git`/`gh` through `tail`/`grep` when needing exit-status. Capture to file or check `$?` directly.
- **Commit message format:** runlog commit uses `chore(bm): cut phase-v1-JM-e off governance-v0`. Lands on `phase-v1-JM-e`, never on `governance-v0`.
- **Settings.json verification (mandatory per advisor-context §5):** after the branch is cut, verify `.claude/settings.json` model field. Run `git show phase-v1-JM-e:.claude/settings.json | head -3` and confirm the model identifier reads `opus[1m]`. If drift (e.g. `claude-opus-4-7`), STOP and file a `pending` DQ from `from: "bm"`, kind: `"blocker"`, citing the JM-d Task 2 settings.json drift incident (cost ~3 hours). Do not proceed with phase work until advisor resolves.
- **Serena MCP smoke (mandatory per advisor-context §3 carry-forward):** after the branch is cut, verify `.mcp.json` does NOT contain a `serena` key on `phase-v1-JM-e`. Run `git show phase-v1-JM-e:.mcp.json | grep -c '"serena"'` and confirm output is `0`. If serena re-appeared, STOP and file a DQ — serena's auto-cargo-check causes OOM cascades on EliteDesk parallel workers (per `feedback_serena_auto_cargo_check.md`).
- **Lesson trailer (optional):** if execution surfaces a new lesson, add a single `LESSON:` line at the end of the runlog commit body per `feedback_junior_pmd_write_convention.md`. Skip for routine progress.

## 5. Expected output

A 4-section summary per the bm-cut script Phase 5:

```
## /bm-cut complete

**Branch cut:** phase-v1-JM-e
**Off:** governance-v0 @ <short-sha>
**Plan file on trunk:** .claude/PRPs/plans/v1-jury-mechanics-e.plan.md
**Pushed?:** No (deferred to first commit + /bm-push)
**Runlog:** .claude/runlog/v1-JM-e-runlog.md created
**Settings.json model:** opus[1m] (verified)
**Serena MCP:** absent (verified)

### Hand-off to impl session

Impl can now run `/prp-implement` against the plan above. BM picks up
on `/bm-status` once commits land.
```

The advisor's polling loop reads this and queues the first `impl-task` for plan Task 0 / Task 1.
