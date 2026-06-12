---
name: feedback_windows_git_show_colon_path
description: git show origin/branch:path fails on Windows due to colon ambiguity in the path separator; always route through SSH or a temp-worktree fetch instead.
type: feedback
---

`git show origin/<branch>:<path>` fails on Windows with path errors because the colon in `<branch>:<path>` is ambiguous — the shell or git's Windows layer parses it as a drive letter separator. This surfaces as "fatal: ambiguous argument" or "not a valid object name".

**Why:** Windows git resolves `origin/phase-m2-late-2:.claude/PRPs/plans/m2-late-2.plan.md` with the colon being parsed as a Windows path separator, not the git `<treeish>:<path>` separator. The error is environment-specific and doesn't reproduce on Linux/Mac, so it's easy to miss in lesson authoring.

**How to apply:**

1. **Remote branch content on Windows:** Route through `ssh homeserver "git show origin/<branch>:<path>"` — homeserver runs Linux git which parses the colon correctly.
2. **If SSH is unavailable:** Use a throwaway worktree: `git worktree add ../brehon-fork-tmp origin/<branch>` then `Read` the file directly. Remove with `git worktree remove ../brehon-fork-tmp` after.
3. **Local branches:** `git show <local-branch>:<path>` on Windows uses the local branch ref which avoids the `origin/` colon — test first, but often works for local refs.

**Pattern:** never `git show origin/<branch>:...` on Windows. The SSH route via `homeserver` (Linux) is always reliable.

**Recurrence:** 2× distinct sessions (m2-late-2 plan-read + m2-late-2 AB setup); same workaround both times. Third recurrence → add to `.claude/rules/advisor-orchestrator.md` §"DQ on a non-trunk branch" as a hard callout (currently references `git show <branch-with-slashes>:<path>` but doesn't explicitly warn about the Windows colon trap).
