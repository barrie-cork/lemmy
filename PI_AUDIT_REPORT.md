# Pi best-practices audit — Lemmy/Brehon

**Date:** 2026-05-06
**Pi version:** 0.73.0
**Repo:** `/Users/barrie/Developer/lemmy`
**Docs source:** `/opt/homebrew/lib/node_modules/@mariozechner/pi-coding-agent/docs`

## Summary

The Brehon pi setup is broadly healthy and current with pi 0.73.0:

- `.pi/settings.json` is valid and uses documented project-relative paths.
- `.pi/extensions/lemmy-hooks.ts` follows the canonical `claude-rules.ts` system-prompt injection pattern and type-checks cleanly.
- `.pi/extensions/tsconfig.json` is present, matching the known global-install type-checking quirk.
- 12 pi-native skill wrappers and 27 prompt templates are discoverable.
- Legacy Claude skills are loaded via `../.claude/skills`, which matches pi's documented cross-harness skill support.
- Compaction is enabled.

No high-severity doc drift was found. The main gap is memory-state ergonomics: pi session memory is available through pi sessions/compaction, but the local Brehon PMD/retro hook path is currently not portable on this Mac checkout unless `PROJECT_MEMORY_DB` is exported.

## Automated audit script output

The official skill scripts emitted 1 low-severity finding:

```json
{"severity":"low","category":"UNDERUSE","path":"/Users/barrie/Developer/lemmy/.pi/extensions/lemmy-hooks.ts","message":"Extension registers no commands, tools, or providers — only event handlers. If users would benefit from a manual trigger, pi.registerCommand() is available (see extensions.md 'pi.registerCommand')."}
```

## Findings

### 1. Extension is event-only; no manual Brehon status command

- **Severity:** low
- **Category:** UNDERUSE
- **Path:** `.pi/extensions/lemmy-hooks.ts`
- **Reference:** pi docs `extensions.md` document `pi.registerCommand()`, and examples include `model-status.ts`, `preset.ts`, and `todo.ts` for user-invoked commands/status UI.
- **Issue:** The extension injects context, blocks dangerous bash, runs copied hooks, and auto-commits edit/write results, but exposes no `/brehon-status`-style manual command.
- **Impact:** Not blocking. The current event-only design works. A command would make it easier to inspect branch, DQ pending count, PR state, Junior task status hints, active model, and memory DB path without asking the LLM to rediscover them.
- **Proposed fix:** Add a small `pi.registerCommand("brehon-status", ...)` that reports:
  - current branch + dirty state,
  - `.claude/decision-queue.json` pending/resolved counts,
  - PR for current phase branch if any,
  - whether `PROJECT_MEMORY_DB`/PMD path is reachable,
  - active model/thinking level.
- **Action:** pending user decision.

### 2. Local PMD memory path is not wired for pi sessions on this Mac checkout

- **Severity:** medium
- **Category:** PORTABILITY / OTHER
- **Path:** `start-pi.sh`, `.pi/hook-scripts/retro-check.sh`, `.project-memory`
- **Reference:** pi docs `settings.md`/`extensions.md` do not provide MCP memory; pi-native persistence is sessions/compaction/extension entries. The repo-local hook explicitly supports `PROJECT_MEMORY_DB`, but `start-pi.sh` does not set it.
- **Issue:** `.project-memory` is a symlink to `/srv/brehon-fork/.project-memory`, which is not reachable from this Mac checkout. The actual local SQLite memory DB exists at `.claude/memory/memory.db`, but `retro-check.sh` looks for `PROJECT_MEMORY_DB`, then worktree main repo `.project-memory/memory.db`, then `.project-memory/memory.db`. Therefore the retro check currently fails open locally instead of consulting `.claude/memory/memory.db`.
- **Impact:** Medium for the user's stated goal of harnessing memory configuration. Pi's own session memory/compaction works, but Brehon PMD-backed retro enforcement/lookup is not reliably connected in local pi sessions.
- **Proposed fix:** In `start-pi.sh`, export the local DB when present:

  ```bash
  PMD_DB="$(pwd)/.claude/memory/memory.db"
  if [ -f "$PMD_DB" ]; then
    export PROJECT_MEMORY_DB="$PMD_DB"
  fi
  ```

  Longer term, prefer a repo-local CLI shim/skill for PMD search/write, because pi has no MCP and `.claude/memory/` is currently untracked local state.
- **Action:** pending user decision.

### 3. Pi session storage is not project-local

- **Severity:** low
- **Category:** UNDERUSE
- **Path:** `.pi/settings.json`
- **Reference:** pi docs `settings.md` and `sessions.md` document `sessionDir`, `/resume`, `/tree`, `/fork`, `/clone`, `/compact`, and named sessions.
- **Issue:** `.pi/settings.json` does not set `sessionDir`, so sessions use the global default under `~/.pi/agent/sessions/` rather than `.pi/sessions`.
- **Impact:** Not blocking. Global sessions work. Project-local sessions would make Brehon session history easier to back up, inspect, and keep separated from other repos. `.gitignore` already ignores `.pi/sessions/`, so the repo is prepared for this.
- **Proposed fix:** Add to `.pi/settings.json`:

  ```json
  "sessionDir": ".pi/sessions"
  ```

  Then use `/name`, `/tree`, `/fork`, and `/compact` intentionally for advisor/BM/impl session boundaries.
- **Action:** pending user decision.

### 4. No pi-native PMD search/write tool or skill exists

- **Severity:** low
- **Category:** UNDERUSE
- **Path:** `.pi/skills/`, `.pi/extensions/`
- **Reference:** pi docs `extensions.md` explain custom tools and `registerTool()`; `skills.md` recommends CLI/script-backed skills. `PI_QUIRKS.md §11` notes pi has no MCP, so MCP replacements should be CLI shims or extensions.
- **Issue:** Brehon has PMD lessons and an SQLite DB, but pi currently only injects coordination state and runs retro shadow hooks. There is no `/skill:memory-search`, `/skill:memory-write-eval`, or `pmd_search` custom tool for local pi sessions.
- **Impact:** Not blocking for normal coding. It means pi sessions rely on loaded context + grep/read of `.claude/lessons` instead of querying the SQLite memory DB directly.
- **Proposed fix:** Add a small CLI shim (for example `.pi/hook-scripts/pmd_cli.py`) plus a pi skill `pmd-memory`, or register typed tools `pmd_search` / `pmd_write_eval` in `lemmy-hooks.ts`. Prefer a CLI shim first: it is harness-agnostic and easy to test.
- **Action:** pending user decision.

### 5. Good practice confirmed: AGENTS.md dual-harness isolation

- **Severity:** none
- **Category:** confirmation
- **Path:** `AGENTS.md`, `.pi/PROJECT_CONTEXT.md`, `.pi/settings.json`
- **Reference:** `PI_QUIRKS.md §17`; pi docs `usage.md`/context behavior; project migration notes.
- **Finding:** The repo uses `AGENTS.md` as the pi entry point and avoids loading `CLAUDE.md` by default. `.pi/PROJECT_CONTEXT.md` is injected via the extension. This is the right shape for a dual-harness Brehon repo.

### 6. Good practice confirmed: extension type-checking and prompt/skill discovery

- **Severity:** none
- **Category:** confirmation
- **Path:** `.pi/extensions/tsconfig.json`, `.pi/prompts/`, `.pi/skills/`, `.pi/settings.json`
- **Reference:** pi docs `skills.md`, `prompt-templates.md`, `extensions.md`; `PI_QUIRKS.md §1`, §2, §16.
- **Finding:** TypeScript check passed with `tsc --noEmit`; prompt templates are flat `.pi/prompts/*.md` files; skill commands are enabled; settings paths resolve correctly relative to `.pi/`.

## Suggested next actions

1. Decide whether to apply Finding 2 now. This is the only medium-severity finding and directly affects local PMD/retro memory wiring.
2. Consider adding `sessionDir` to `.pi/settings.json` if you want Brehon pi sessions stored under the repo-local `.pi/sessions/` directory.
3. Defer `brehon-status` and PMD custom tool/skill unless you want more pi-native orchestration ergonomics now.
