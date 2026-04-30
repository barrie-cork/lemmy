# [role:bm-task] smoke-no-serena — verify worker init without serena MCP

## 1. Dispatch line

`[role:bm-task] smoke-no-serena — see .claude/PRPs/briefs/smoke-no-serena-2026-04-30.md`

## 2. Scope

Smoke test confirming Junior worker boots cleanly after `serena` was
removed from `/srv/brehon-fork/.mcp.json` (2026-04-30, post-cargo-cascade
catch-fire). The user wants rust-analyzer LSP via the
`rust-analyzer-lsp@claude-plugins-official` plugin (user-scope, already
enabled in `~/.claude/settings.json`) instead of serena's auto-cargo-check
path.

**This task does:**
1. Print `WORKER_BOOT_OK` and your model identifier.
2. Use `Read` to read `crates/api/api/src/governance/admin_assign_jury.rs`
   line 1-30 (a Rust file — exercises the LSP attach path).
3. Print whether `LSP` tool is available in your tool list.
4. Exit zero. No commits, no edits.

**This task does NOT:**
- Edit any file.
- Run cargo, clippy, build, or test.
- Make commits.
- Spawn rust-analyzer manually (we're testing whether the plugin auto-attaches).

## 3. Required reading

- `.mcp.json` at `/srv/brehon-fork/.mcp.json` (note: `serena` is removed; `project-memory` + `Ref` only)

## 4. Constraints

**Hard forbids:**
- `cargo *` of any kind.
- Editing any file.
- Running >5 minutes (it's a smoke test; if the worker can't print and exit in <2 min, something is wrong — exit non-zero with a one-line diagnosis).

**Required output:**
- Print `WORKER_BOOT_OK` on a line.
- Print `MODEL=<model-id>` (read from your env or system prompt).
- Print `LSP_AVAILABLE=<yes|no>` based on your tool inventory.
- Exit 0.

This brief is a one-shot smoke test post-`.mcp.json` change. Cancel
the task if it's still running after 2 minutes.
