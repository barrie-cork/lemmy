---
name: PMD is cross-lane — pin every worktree MCP to the canonical DB
description: Multi-lane worktrees each ran project-memory MCP with a relative PROJECT_MEMORY_DB, stranding retros in lane-local DBs while the Stop hook reads the canonical DB → ~27+ false retro-check blocks across one phase. Pin every .mcp.json to the absolute canonical PMD path.
type: feedback
---

# PMD is cross-lane shared — pin every worktree's MCP to the canonical DB

The project-memory DB (PMD) is **global knowledge** (lessons, retros,
patterns) and must live in **one canonical store** that every lane's MCP
writes to and the Stop hook reads from. This is the exact opposite of
the decision-queue, which `.claude/rules/multi-lane-worktree.md`
correctly isolates **per-lane**. Conflating the two — treating the PMD
as per-worktree like the DQ — is the failure.

Canonical PMD: **`C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db`**
(the canonical checkout's `.project-memory/`).

## The rule

Every worktree's `.mcp.json` (gitignored — it holds API keys, correctly
untracked) MUST set the `project-memory` server's `PROJECT_MEMORY_DB` to
the **absolute canonical path** above. NEVER:

- a relative `.project-memory/memory.db` — the MCP resolves it against
  the per-worktree `PROJECT_ROOT`, so each lane writes to its own
  lane-local DB.
- a `brehon-fork-<lane>/.project-memory/...` absolute path — same
  stranding, just explicit.

Only `PROJECT_ROOT` varies per worktree. `PROJECT_MEMORY_DB` is
invariant across all lanes. The tracked `.mcp.json.example` encodes
this with a `_comment_pmd_cross_lane` guard key — bootstrap new lanes
from the template and the canonical path carries over verbatim.

## Why (the incident)

**v1-ship-1, 2026-05-18.** This worktree (`brehon-fork-ship-1`) ran the
project-memory MCP with `PROJECT_MEMORY_DB=.project-memory/memory.db`
(relative) + `PROJECT_ROOT=brehon-fork-ship-1`, so every
`memory_write_eval` wrote to `brehon-fork-ship-1/.project-memory/memory.db`.

The Stop hook `.claude/hooks/retro-check.sh` resolves the PMD via
`git rev-parse --git-common-dir`, which from **any** worktree points at
the **canonical** `brehon-fork/.git` → the hook always reads
`brehon-fork/.project-memory/memory.db`.

Net effect: every v1-ship-1 session wrote a **genuine, well-formed,
branch-correct retro** that the hook **could never see**. ~27+ false
Stop-hook blocks accumulated across the phase. All **21** v1-ship-1
retros (the entire phase's retro history) were stranded in the
lane-local DB, invisible to the canonical-DB-reading hook. Every
session re-explained the gap and held position (no bypass) — correct
handling, but a large compounding token tax that displaced reasoning
headroom.

The federation-inbound-a lane did NOT false-block — it had no
per-worktree `.mcp.json`, so its MCP resolved to the canonical DB by
default and its retros landed where the hook looks. Convergence by
accident for one lane; the fix makes it convergence **by design** for
all.

## Hard refusal: never "fix" this by editing the hook

The hook's `git-common-dir` resolution is **correct** — it
intentionally lands on the canonical shared DB so retros written from
any lane are visible. The defect class is **always MCP-side**: a
relative or per-lane `PROJECT_MEMORY_DB`. Editing
`retro-check.sh` to chase a stranded-retro symptom (a) is explicitly
forbidden by the hook message, (b) is tracked as a bypass attempt, and
(c) fixes the wrong layer. The fix is always: correct the offending
lane's `.mcp.json` to the absolute canonical path.

Also never: forge `created_at`, raw-SQL-insert the retro, or write a
padding duplicate retro to satisfy the counter — all are bypasses /
anti-patterns (`feedback_retro_not_report`). The substantive deliverable
(a genuine retro) existing is what matters; the hook's blindness to it
is the environmental defect to fix at the config layer.

## Diagnosis recipe (when retro-check false-blocks despite a written retro)

```bash
# 1. Where does THIS worktree's MCP write? (.mcp.json is gitignored — read it)
python -c "import json,io;print(json.load(io.open('.mcp.json',encoding='utf-8'))['mcpServers']['project-memory']['env']['PROJECT_MEMORY_DB'])"

# 2. Where does the HOOK read? (git-common-dir → canonical repo's .project-memory)
GC=$(git rev-parse --git-common-dir); echo "$(dirname "$GC")/.project-memory/memory.db"

# 3. If (1) != (2): the retro is stranded. Confirm — is the just-written eval
#    in the hook's DB?
sqlite3 "<hook-DB-from-step-2>" "SELECT COUNT(*) FROM memories WHERE memory_type='qa-result' AND title LIKE 'Task retro:%' AND created_at >= datetime('now','-60 minutes')"
#    0 here + a real recent eval elsewhere = this exact defect.

# 4. Fix: pin .mcp.json PROJECT_MEMORY_DB to the absolute canonical path.
#    Takes effect on NEXT MCP restart (the cached handle doesn't repoint live).
# 5. Reconcile stranded rows via memory_write_eval AFTER the MCP restarts
#    onto canonical (NEVER raw SQL — that's a forbidden bypass).
```

## Sequencing constraint

`.mcp.json` edits take effect only on **MCP server restart** — the
running MCP cached its DB handle at startup. So:

1. Edit `.mcp.json` → absolute canonical path (persists on disk; file is
   gitignored so it's not committed, but the working copy survives).
2. Restart the MCP (next session, or explicit `/mcp` reconnect).
3. THEN reconcile any stranded historical retros via `memory_write_eval`
   (now landing in canonical). Doing step 3 before step 2 just re-writes
   into the stranded lane-local DB.

## See also

- `.claude/rules/multi-lane-worktree.md` §"PMD is cross-lane shared,
  NOT per-lane isolated" — the canonical rule.
- `.mcp.json.example` `_comment_pmd_cross_lane` guard key — the template
  invariant new lanes inherit.
- `feedback_multi_lane_worktree_discipline.md` — the per-lane DQ
  isolation this is the deliberate inverse of.
- `feedback_pmd_two_memory_systems_distinction.md` — `.md` auto-memory
  vs DB-backed PMD (the DB is the one this lesson pins).
- `feedback_retro_not_report.md` — why padding-duplicate retros are not
  an acceptable workaround for the false-block.
