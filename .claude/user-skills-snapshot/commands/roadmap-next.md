---
description: Recommend + cut the next unstarted sub-phase from v1-roadmap.json, bootstrap the lane worktree (including .mcp.json wiring), and flip the roadmap entry to in_flight.
argument-hint: (none — reads roadmap automatically)
---

# /roadmap-next — cut next roadmap sub-phase

Runs in the **canonical checkout** (`C:/Users/barri/Developer/brehon-fork`) on `governance-v0`.

---

## Phase 0 — Hard refusals (check before any state-changing call)

**Step 0 (MANDATORY, before any routing decision):** Read
`.claude/refs/auto-roadmap.md` into context. That file holds the
canonical hard refusals (#1-#10) + ownership boundaries + state-routing
invariants 1-8 that this skill enforces. It does NOT auto-load at session
start (relocated 2026-05-22 from `.claude/rules/` to `.claude/refs/` to
free Memory-files budget). This Read is the first action of every
`/roadmap-next` invocation.

1. **Wrong CWD.** Must run from `C:/Users/barri/Developer/brehon-fork`. Check:
   ```bash
   pwd
   git branch --show-current
   ```
   If CWD is any `brehon-fork-<lane>` worktree → STOP: "skill 1 must run in canonical CWD; current CWD is `<X>`".

2. **Wrong branch.** `git branch --show-current` must be `governance-v0`. Any other branch → STOP.

3. **Dirty working tree.** `git status --short` non-empty → STOP, surface file list, do NOT auto-stash.

4. **Roadmap missing or malformed.** Glob `.claude/PRPs/v1-roadmap.json`. Missing or JSON parse error → STOP, surface path + error.

5. **No eligible sub-phase.** `what_remains.high_priority_unstarted` is empty AND all lanes are `done | skipped` → STOP: "no eligible sub-phase to recommend; check roadmap or close in-flight lanes first".

6. **Target worktree already exists.** `git worktree list` shows `../brehon-fork-<lane-suffix>` already present → STOP, surface existing path. User must `git worktree remove` it first.

---

## Phase 1 — Read roadmap + recommend

1. Read `.claude/PRPs/v1-roadmap.json`.
2. Read `what_remains.high_priority_unstarted` and `implementation_steering.next_logical_sub_phase`.
3. Find the first `unstarted` sub-phase in the priority list.
4. Display recommendation via `AskUserQuestion`:

   ```
   Next eligible sub-phase: <lane> — <sub-phase-id>
   Scope: <scope field from roadmap>
   PRD: <prd field from lane entry>
   Rationale: <rationale from what_remains>
   ```

   Options:
   - **Confirm** — proceed with the recommendation
   - **Pick different sub-phase** (via "Other" — user types the sub-phase id, e.g. `v1-ship-2`)
   - **Defer** — exit without cutting

   On **Defer**: exit cleanly; do NOT mutate any state.

---

## Phase 2 — Derive lane suffix + branch name

From the confirmed sub-phase id (e.g. `v1-RT-r2`, `v1-ship-2`):

| Sub-phase id | Lane suffix | Branch name | Worktree path |
|---|---|---|---|
| `v1-RT-r2` | `rt-r2` | `phase-v1-RT-r2` | `C:/Users/barri/Developer/brehon-fork-rt-r2` |
| `v1-ship-2` | `ship-2` | `phase-v1-ship-2` | `C:/Users/barri/Developer/brehon-fork-ship-2` |
| `v1-RT-r3` | `rt-r3` | `phase-v1-RT-r3` | `C:/Users/barri/Developer/brehon-fork-rt-r3` |

Derivation rule: strip `v1-` prefix, lowercase, hyphens preserved. If sub-phase id has mixed case (e.g. `v1-RT-r2` → `rt-r2`).

---

## Phase 3 — Dispatch bm-cut Junior

Dispatch the `branch-manager` subagent to cut the phase branch:

```
Agent(subagent_type="branch-manager", model="haiku", prompt=
  "Run bm-cut for <branch-name>. Follow .claude/commands/bm/bm-cut.md phases 0-5.
   Cut off governance-v0, push to origin with -u.
   Note: plan file will be missing (no plan written yet for this sub-phase) — override the plan-file
   check and proceed. Write a bm-cut runlog entry. Return a one-paragraph summary including
   the branch name and push status.")
```

Wait for bm-cut to return. On non-zero exit → STOP, surface BM output verbatim.

Verify push landed: `git ls-remote origin refs/heads/<branch-name>` must return non-empty.

---

## Phase 4 — Create lane worktree

From the canonical checkout:

```bash
git fetch origin <branch-name>
git worktree add C:/Users/barri/Developer/brehon-fork-<lane-suffix> <branch-name>
```

Verify the worktree directory exists before proceeding.

---

## Phase 5 — Bootstrap .mcp.json (MCP wiring)

This is the step the old checklist described as "cp .mcp.json.example → .mcp.json" but the example
carries placeholder `/path/to/` strings. The correct action is to write a complete, working `.mcp.json`
directly.

Write `C:/Users/barri/Developer/brehon-fork-<lane-suffix>/.mcp.json` with this exact content
(substitute `<lane-suffix>` in `PROJECT_ROOT`):

```json
{
  "mcpServers": {
    "project-memory": {
      "command": "node",
      "args": ["C:\\Users\\barri\\Developer\\MCPs\\project-memory-mcp\\dist\\index.js"],
      "env": {
        "PROJECT_MEMORY_DB": "C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db",
        "PROJECT_ROOT": "C:\\Users\\barri\\Developer\\brehon-fork-<lane-suffix>",
        "PROJECT_NAME": "brehon-fork",
        "OLLAMA_URL": "http://homeserver:11434"
      }
    },
    "junior-brehon": {
      "command": "node",
      "args": ["C:\\Users\\barri\\Developer\\MCPs\\junior-mcp\\dist\\index.js"],
      "env": {
        "JUNIOR_REPO": "brehon-fork",
        "JUNIOR_SSH_HOST": "homeserver"
      }
    },
    "ref-context": {
      "type": "http",
      "url": "https://api.ref.tools/mcp",
      "headers": {
        "x-ref-api-key": "ref-b45392fba76a0206ad0a"
      }
    },
    "tavily": {
      "type": "stdio",
      "command": "cmd",
      "args": ["/c", "npx", "-y", "tavily-mcp@latest"],
      "env": {
        "TAVILY_API_KEY": "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"
      }
    }
  }
}
```

**Verify canonical PMD path** (mandatory per `.claude/rules/pmd-invariants.md` invariant #1):
```bash
python -c "
import io, json
c = json.load(io.open('C:/Users/barri/Developer/brehon-fork-<lane-suffix>/.mcp.json', encoding='utf-8'))
db = c['mcpServers']['project-memory']['env']['PROJECT_MEMORY_DB']
assert db == 'C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db', f'FAIL: PMD path wrong: {db}'
print('OK: canonical PMD path verified')
"
```

Expected: `OK: canonical PMD path verified`. Any other output → fix `.mcp.json` before proceeding.

---

## Phase 6 — Bootstrap .claude/settings.local.json (hook wiring)

Write `C:/Users/barri/Developer/brehon-fork-<lane-suffix>/.claude/settings.local.json`.

**If the file does not exist**, create it with:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": ".*",
        "hooks": [
          {"type": "command", "command": "bash .claude/hooks/pmd-canonical-guard.sh"},
          {"type": "command", "command": "bash .claude/hooks/session-start-multi-lane-check.sh"}
        ]
      }
    ]
  }
}
```

**If the file already exists**, merge the `SessionStart` hooks array into the existing file —
do NOT overwrite other sections (`permissions`, `env`, etc). The two hooks must be present in order
(pmd-canonical-guard first, multi-lane-check second).

---

## Phase 7 — Programmatic verification (steps 8 + 10 from bootstrap checklist)

Run both checks from the **lane worktree CWD** (`C:/Users/barri/Developer/brehon-fork-<lane-suffix>`):

**Check A — pmd-canonical-guard.sh wiring:**
```bash
python -c "
import io, json
s = json.load(io.open('.claude/settings.local.json', encoding='utf-8'))
ss = s.get('hooks', {}).get('SessionStart', [])
wired = any(
    'pmd-canonical-guard.sh' in h.get('command', '')
    for entry in ss for h in entry.get('hooks', [])
)
assert wired, 'FAIL: pmd-canonical-guard.sh SessionStart wiring missing'
print('OK: pmd-canonical-guard.sh wired at SessionStart')
"
```

**Check B — session-start-multi-lane-check.sh wiring:**
```bash
python -c "
import io, json
s = json.load(io.open('.claude/settings.local.json', encoding='utf-8'))
ss = s.get('hooks', {}).get('SessionStart', [])
wired = any(
    'session-start-multi-lane-check.sh' in h.get('command', '')
    for entry in ss for h in entry.get('hooks', [])
)
assert wired, 'FAIL: session-start-multi-lane-check.sh SessionStart wiring missing'
print('OK: session-start-multi-lane-check.sh wired at SessionStart')
"
```

Both must output `OK: ...`. Any `FAIL` → fix the settings file before proceeding.

**Check C — PreToolUse hook present (bootstrap checklist step 11):**
```bash
python -c "
import io, json
s = json.load(io.open('.claude/settings.json', encoding='utf-8'))
pre = s.get('hooks', {}).get('PreToolUse', [])
wired = any(
    'refuse-ssh-reset-hard-shared-checkout.sh' in h.get('command', '')
    for entry in pre for h in entry.get('hooks', [])
)
assert wired, 'FAIL: refuse-ssh-reset-hard-shared-checkout.sh PreToolUse wiring missing — re-pull governance-v0'
print('OK: refuse-ssh-reset-hard-shared-checkout.sh wired at PreToolUse')
"
```

Run from either the lane worktree or canonical (`.claude/settings.json` is tracked and shared).

---

## Phase 8 — Update v1-roadmap.json

Atomic protocol (per `.claude/rules/multi-lane-worktree.md` Hard refusal #6):

1. `git fetch origin governance-v0` immediately before the write.
2. Read `.claude/PRPs/v1-roadmap.json` fresh.
3. Mutate: set the sub-phase's `status` from `"unstarted"` → `"in_flight"`, add:
   ```json
   "worktree": "C:/Users/barri/Developer/brehon-fork-<lane-suffix>",
   "in_flight_since": "<today ISO date>"
   ```
   Also bump top-level `"last_updated_at": "<today ISO date>"`.
4. Verify JSON (`python -c "import io, json; json.load(io.open('.claude/PRPs/v1-roadmap.json', encoding='utf-8')); print('OK')"`)
5. `git add .claude/PRPs/v1-roadmap.json` → `git commit -m "chore(advisor): roadmap-next flip <sub-phase> in_flight + cut lane"` → `git push origin governance-v0` — **as a single uninterrupted sequence**.
6. After push, verify the entry survived: `git log -1 --stat`.

On non-fast-forward push (another session committed between step 1 and step 5): re-fetch, re-read, re-mutate, re-commit, re-push. Up to 3 attempts. On 3rd failure → STOP, surface to user. Never `--force`.

---

## Phase 9 — Append to runlog

Append to `.claude/runlog/bm-runlog.md` (create if absent):

```markdown
## advisor: roadmap-next cut <sub-phase> — <ISO timestamp>
- **branch:** <branch-name>
- **worktree:** C:/Users/barri/Developer/brehon-fork-<lane-suffix>
- **bootstrap:** .mcp.json ✓  pmd-canonical-guard ✓  multi-lane-check ✓
- **roadmap:** flipped <sub-phase> unstarted → in_flight
```

Commit this to `governance-v0`: `git add .claude/runlog/bm-runlog.md && git commit -m "docs(advisor): roadmap-next runlog for <sub-phase>" && git push origin governance-v0`.

---

## Phase 10 — Handoff output

Print one screen:

```
=== roadmap-next: <sub-phase> cut ===

Branch:    <branch-name> (pushed to origin)
Worktree:  C:/Users/barri/Developer/brehon-fork-<lane-suffix>

Bootstrap:
  .mcp.json             ✓  (project-memory, junior-brehon, ref-context, tavily)
  pmd-canonical-guard   ✓  (SessionStart hook wired)
  multi-lane-check      ✓  (SessionStart hook wired)
  PreToolUse hook       ✓  (refuse-ssh-reset-hard-shared-checkout.sh)

Roadmap:   <sub-phase> → in_flight

Next steps:
  1. Open Claude Code in  C:/Users/barri/Developer/brehon-fork-<lane-suffix>
  2. Run /auto-roadmap       (skill 2 — drives the sub-phase via /auto-phase)
```

---

## Pre-commit dogfood

This skill was walked against the `v1-ship-2` case on 2026-05-22:

- Phase 0: canonical checkout confirmed, `governance-v0` branch, clean tree.
- Phase 1: roadmap read; `what_remains.high_priority_unstarted[0]` → `v1-ship-2`. AskUserQuestion surfaced with scope from roadmap.
- Phase 2: suffix derived as `ship-2`; branch `phase-v1-ship-2`.
- Phase 3: bm-cut dispatch pattern verified against `.claude/commands/bm/bm-cut.md`.
- Phase 4: `git worktree add` command verified against `multi-lane-worktree.md` §Lifecycle.
- Phase 5: `.mcp.json` content verified against `brehon-fork-fed-in-d/.mcp.json` (the only lane with a correctly configured MCP at time of authorship). `PROJECT_MEMORY_DB` canonical path confirmed. **Root cause of `ship-2` MCP failure**: `.mcp.json` was copied from `.mcp.json.example` which has `/path/to/` placeholders — this skill writes the real paths instead. `rust-analyzer-mcp` excluded (binary not installed).
- Phase 6-7: settings.local.json wiring and verification scripts taken verbatim from `feedback_phase_lane_worktree_bootstrap_checklist.md` steps 6-11.
- Phase 8: atomic roadmap-update protocol from `multi-lane-worktree.md` Hard refusal #6.

**What worked:** Phase 5 MCP-content approach (write real paths, skip example copy) is the fix for the class of MCP failures seen in `ship-2`. **What to watch:** plan-file absence in bm-cut (Phase 3 override note) — checklist step 5 originally expected the plan to exist; `/roadmap-next` cuts before planning, so the override is intentional.

---

## See also

- `.claude/refs/auto-roadmap.md` — companion rule with hard refusals + state-routing invariants
- `~/.claude/commands/auto-roadmap.md` — skill 2 body (runs in lane worktree, drives sub-phase via `/auto-phase`)
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` — the bootstrap checklist this skill operationalises (steps 5-11)
- `.claude/rules/multi-lane-worktree.md` — worktree discipline + hard refusals
- `.claude/rules/pmd-invariants.md` — PMD canonical-path invariant (invariant #1) enforced in Phase 5
- `.claude/commands/bm/bm-cut.md` — BM cut script invoked in Phase 3
- `.claude/PRPs/v1-roadmap.json` — the roadmap file Phase 1 reads + Phase 8 mutates
- `.claude/PRPs/specs/auto-roadmap-skill-pair.md` — feasibility spec this skill implements
