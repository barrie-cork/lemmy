# Pi Extensions for Lemmy/Brehon Rust Development

**Status:** local Pi setup updated 2026-05-04.

This repo is a Rust/Cargo workspace for a governance-enabled Lemmy fork. Pi extensions should support navigation and validation without replacing the Brehon plan/task/BM/user-gate workflow.

## Current installed packages

Global Pi settings at `~/.pi/agent/settings.json` include:

```json
{
  "packages": [
    "npm:pi-rtk-optimizer",
    {
      "source": "git:github.com/tmustier/pi-extensions",
      "extensions": [
        "files-widget/index.ts",
        "code-actions/index.ts"
      ],
      "skills": [],
      "prompts": [],
      "themes": []
    }
  ]
}
```

The `tmustier/pi-extensions` package is intentionally filtered so only `/readfiles` and `/code` are loaded.

Helper binaries installed with Homebrew:

```bash
brew install rtk bat git-delta glow
```

## Package use cases

### `pi-rtk-optimizer`

Use for output compaction and safe RTK command rewriting during Rust/Lemmy work.

Good fits:

- Compact `cargo check`, `cargo test`, `cargo clippy`, and build logs.
- Summarise noisy test output.
- Keep large `git diff`, `rg`, and linter output manageable.
- Rewrite supported shell commands through `rtk rewrite` when beneficial.

Validation:

```bash
rtk --version
rtk rewrite 'git status'
```

Inside Pi:

```text
/rtk verify
/rtk show
/rtk stats
```

### `/readfiles` from `files-widget/index.ts`

Use for interactive repo browsing and targeted context selection.

Good fits:

- Browse `crates/` before touching API, DB, federation, or utility code.
- Inspect Diesel migrations and schema changes.
- Browse `.claude/PRPs/` plans/briefs when working under the Brehon process.
- Read governance docs under `docs/brehon-law-inspired-network/`.
- View diffs inline and send specific ranges to the agent.

Common commands:

```text
/readfiles
/readfiles crates/
/readfiles crates/db_schema/
/readfiles docs/brehon-law-inspired-network/
```

### `/code` from `code-actions/index.ts`

Use for extracting snippets from recent assistant messages.

Good fits:

- Copy a generated Rust snippet safely.
- Extract SQL migration snippets.
- Reuse `Cargo.toml` dependency blocks.
- Insert small generated blocks without manual copy/paste mistakes.

Common command:

```text
/code
```

## Project-local Pi extension scripts

Pi v0.72 renamed project `hooks/` to extensions and warns when `.pi/hooks/` exists. This repo therefore keeps migrated Claude-era executable helper scripts under:

```text
.pi/hook-scripts/
```

`lemmy-hooks.ts` invokes those scripts directly:

- `pre-phase-audit.sh` — startup reminder/context.
- `worktree-guard.sh` — write/bash guard.
- `observation-capture.sh` — shadow telemetry.
- `retro-check.sh` — warn-only shutdown retro nudge.

Do not recreate `.pi/hooks/`.

## Deliberately not installed for active Brehon implementation

These may be useful in other repos, but should stay disabled here unless explicitly approved:

- `context-workflow` — overlaps with the existing plan → impl → BM → validation → approval workflow.
- `pi-goal` — duplicates the Junior multi-agent system and can conflict with branch/task discipline.
- `pi-ralph-wiggum` — long autonomous loops risk bypassing mandatory plan and user-gate constraints.

## Reload after config changes

Inside Pi, run:

```text
/reload
```

Then confirm command completion includes:

```text
/readfiles
/code
/rtk
```
