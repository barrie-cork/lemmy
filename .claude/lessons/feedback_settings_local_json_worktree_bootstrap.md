---
name: settings.local.json must be copied into headless-agent worktrees
description: When cutting a fresh git worktree for a headless Agent, copy .claude/settings.local.json (gitignored) alongside .env. Without it, Edit/Write to .claude/ paths auto-deny silently because there is no allow-list and no human to prompt.
type: feedback
originSessionId: f3663548-c6e4-49d0-84b9-1077376a717b
---
When cutting a fresh git worktree to host a headless `Agent` session, copy **two** files from the primary worktree:

```bash
cp /c/Users/barri/Developer/brehon-fork/.env "$NEW_WORKTREE/.env"
cp /c/Users/barri/Developer/brehon-fork/.claude/settings.local.json "$NEW_WORKTREE/.claude/settings.local.json"
```

**Why:** Both files are gitignored — they live only in the primary worktree by default. `.env` is well-known (vcvars + libpq paths). `settings.local.json` carries the **permissions allow-list** that has accumulated over months of sessions.

**Failure shape (what you see if you skip it):**

A headless `Agent` session boots in the new worktree, gets through code edits and validation cleanly, then **silently halts at the commit/push/PR step**. Edits to `.claude/` paths (decision-queue.json, runlog files, retros under `.claude/PRPs/reports/`) auto-deny without prompts because:

1. Default behaviour without an allow-list is to prompt the user to approve.
2. In headless `Agent` mode there is no user to prompt.
3. The result is a silent deny — agent sees its `Edit`/`Write` call return error but no actionable message, and `Bash` heredoc to the same paths fails the same way for the same reason.

**Real cause: absent allow-list, not a deny rule.** Agents tend to mis-attribute the symptom to a deny rule and look for one in `settings.json`. There usually isn't one. The fix is to provide the allow-list.

**Reference incidents (2026-04-19):**

- **polish-3 (PR #65)** — agent made all 7 file edits correctly in `brehon-fork-polish-3-docs-sweep`, then halted at `git commit`. Diagnosed symptom as "denied wholesale", attributed to a deny rule that didn't exist. Recovered by committing from the primary worktree via `git -C`. ~20 min cost.
- **polish-2 (PR #67)** — same gap, same worktree class. Agent worked around it by writing `.claude/PRPs/reports/polish-2-retro.md` via Python `pathlib.Path.write_text` and the brief sweep via `Bash + re.subn`. Shipped end-to-end but with cosmetic friction.

**Belt-and-braces:** if the agent is *interactive* (user available to approve prompts), `settings.local.json` is optional but still recommended — eliminates the prompts.

**Encoded in:** `.claude/rules/multi-session-worktree-safety.md` "Bootstrap checklist when cutting a worktree for a headless agent" section.
