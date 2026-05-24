[role:bm-task] Role-signal hook smoke — verify Junior bm-task emits role-signal row to canonical PMD

## 1. Scope

Verification-only smoke test for the role-signal hook fix in commit
`feaa75db9` (fix(role-signal): use [role:X] tag not branch as Junior gate).
The advisor needs to confirm that:

(a) the patched Stop hook fires for a Junior bm-task (no longer gated on
    `^junior/` branch detection against the daemon's main checkout), and
(b) a `role-signal` sibling row appears in the canonical laptop PMD
    alongside this task's sentinel write (either direct via the CLI if
    reachable, or queued to JSONL on EliteDesk for laptop-side drain).

Do NOT write any code. Do NOT open a PR. Do NOT modify any tracked file.
Do exactly the following:

1. Call `mcp__project-memory__memory_write` with these exact arguments:
   - `memory_type: "deploy-note"`
   - `title: "role-signal-hook-smoke-2026-05-24 sentinel"`
   - `content: "Sentinel write from bm-task to verify role-signal hook fix feaa75db9. Branch: <report your current git branch here via git rev-parse --abbrev-ref HEAD>. Worktree pwd: <report pwd>."`
   - `tags: "smoke,role-signal-fix,task-id-<your-task-id>"`
   - `repo_name: "brehon-fork"`
   Report the row id returned.

2. Run `pwd` and `git rev-parse --abbrev-ref HEAD` and report both.

3. Exit with no commits. The Stop hook will fire automatically after
   this task completes; the advisor will check PMD for both:
   - your sentinel row (memory_type=deploy-note)
   - a role-signal row (tags contain "role:bm-task" + "kind:utilisation")
   And/or rsync the EliteDesk JSONL queue file if the CLI fallback path
   was taken.

## 2. Required reading

(none — sentinel write is the entire task)

## 3. Constraints

- Touch nothing on disk. No edits, no writes to files, no commits, no
  pushes. The ONLY PMD write you do is the one in §1 step 1.
- This task should complete in under 60 seconds.
- Do not invoke any MCP server other than `project-memory` for the §1
  step 1 write.
- Do not read any other files.
- Do not write a Task retro at the end (Stop hook retro-check.sh will
  not fire on bm-task on governance-v0 main checkout — and that's fine
  for this smoke; we ONLY care that the role-signal hook fires).
- The success criterion is BOTH rows appearing in canonical PMD (or the
  sentinel landing AND the JSONL queue file appearing on EliteDesk for
  drain).
