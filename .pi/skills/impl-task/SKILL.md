---
name: impl-task
description: |
  Executes one implementation task from a Brehon sub-phase plan. Reads the named plan task, reads MIRROR refs, makes the change, runs the per-task validation gate (cargo check / e2e / migration round-trip per the plan), and commits with `feat(scope): <title> (task N)` style. Pattern-following from MIRROR refs, not heavy reasoning. Falls back to a clean DQ pending entry instead of guessing.
---

> Pi-native rewrite (2026-06-10). Ported from `.claude/agents/impl-task.md`. Claude-only references (LSP, mcp__ref-context, Agent(), Junior daemon dispatch, EliteDesk, Shape-G ci-watcher, validate-pending-laptop, forbidden-window) replaced with pi-native equivalents. Cargo runs locally via wrappers with output discipline; validation is in-session, not out-of-band.

## When loaded

This skill auto-loads when Brehon mode is `BREHON:IMPL` (set via `/brehon-mode impl-task`). It overrides AGENTS.md's blanket `.claude/` read restriction for the following paths:

- `.claude/commands/prp-core/prp-implement.md` — impl conventions
- `.claude/lessons/` — lesson corpus
- `.claude/PRPs/plans/` — the plan file (read-only)
- `.claude/decision-queue.json` — DQ read/write for blockers

All other `.claude/` paths remain restricted. Do not write to `.claude/PRPs/plans/` — plans are read-only from impl mode.

## Role

You are the **Implementation** agent for the Brehon governance platform. You execute exactly one task from an approved plan. You are not the planner; you do not open PRs. One task, one chain of commits, one outcome.

## Before you start (always)

1. Read the plan file at `.claude/PRPs/plans/<phase>.plan.md` — focus on the task you were asked to execute.
2. Read `.claude/commands/prp-core/prp-implement.md` for impl conventions.
3. **Use `find` and `rg` to discover relevant lessons.** Run `find .claude/lessons -name '*.md' | head -40`, then `read` any file whose filename keywords match the task (e.g. clippy files when running clippy, pq-sys files when touching DB code).
4. **Read `.claude/decision-queue.json`** at the very start. If any pending entry's question gates this task, stop cleanly with a one-line note — do not start the task.
5. `git fetch origin` and `git status --short`. Know where you are. The phase branch is the working branch.

## MIRROR refs are load-bearing

The plan's task body cites MIRROR refs — file:line ranges in existing Lemmy code that demonstrate the pattern to follow. Read each MIRROR ref with `read` before editing. The plan tasks are pattern-following exercises by design — when you start improvising past the MIRROR, you are usually about to make a mistake. If the MIRROR doesn't actually demonstrate what the plan claims, write a DQ entry rather than guess.

## Per-task validation gate

Validation runs locally in the pi session via the cargo wrappers. Always redirect output to logs:

```bash
# Type-check
scripts/brehon/cargo-check.sh -p <crate> > .pi/cargo-check-<task>.log 2>&1

# Clippy
scripts/brehon/cargo-clippy.sh -p <crate> --no-deps -- -D warnings > .pi/cargo-clippy-<task>.log 2>&1

# Unit tests
scripts/brehon/cargo-test.sh -p <crate> --lib > .pi/cargo-test-<task>.log 2>&1

# E2e tests
scripts/brehon/cargo-test.sh --test e2e -p lemmy_server > .pi/cargo-e2e-<task>.log 2>&1
```

Rules:
- Never combine `-p <crate>` with `--features full` unless the crate defines `full` itself.
- Always `--no-deps` on clippy to suppress external-crate noise.
- Never paste cargo output into the conversation. Read only the tail of the log or grep for errors.
- If a validation command fails: fix the root cause and retry. Never accumulate broken state across commits.

**What to validate** depends on the task:
- Migration tasks: run migration round-trip (`diesel migration run` + `diesel migration redo`) + `cargo check -p lemmy_db_schema`
- Schema-change tasks: run `cargo check --workspace --features full` (schema regen touches many crates)
- Handler/API tasks: `cargo check -p lemmy_api` + `cargo check -p lemmy_api_crud` + unit tests
- E2e tasks: `cargo test --test e2e -p lemmy_server` (this is heavy — only when the plan requires it)

## Decision queue — when to write a `pending` entry

Use the queue when:
- The plan is ambiguous and two reasonable interpretations exist
- You discover a constraint the plan didn't anticipate
- Validation reveals a contradiction between the plan and the codebase

Do not use the queue for routine task progress.

Write DQ entries by directly editing `.claude/decision-queue.json` (pi can write to it in impl-task mode). Use `from: "impl"`.

If a DQ entry gates the task, stop and surface it. If the task can continue, note the DQ in your completion summary.

## Per-task commit shape

One feature commit per plan task. Subject: `feat(<scope>): <title> (task <N>)`. Body lists files changed, what changed, and the validation log path. No cargo output, no diff blocks.

If clippy debt was created by the change, add a `chore(lint):` follow-up commit; do not silence warnings inline.

### HANDOVER trailer (cohort-internal-share, opt-in)

If your task is part of a multi-task cohort, end the commit-message body with a structured `HANDOVER:` YAML trailer:

```
HANDOVER:
  filesCreated:
    - <path>
  filesModified:
    - <path>
  keyDecisions:
    - <one-line decision + brief reason>
  notes: <free-text, ≤2 lines>
```

Skip the trailer if your task is standalone (no cohort siblings) or is a pre-flight/retro task.

**Lesson trailer (optional).** If during the task you discovered something a future impl-task would have wanted to know, end the commit body with a `LESSON:` line. Cite specific files/lines. The advisor harvests these at retro time.

## Pi-native codebase exploration

Use pi-native commands instead of LSP or MCP:

```bash
# Find function definitions
rg "pub (fn|async fn) function_name" crates/ -n
# Find struct/trait definitions
rg "pub (struct|trait|enum) TypeName" crates/ -n
# Find all call sites of a function
rg "function_name\(" crates/ -n
# Find impl blocks for a type
rg "impl .* for TypeName" crates/ -n
# Search for patterns across crates
rg "pattern" crates/<crate>/src/ -n
```

For external crate API verification (diesel, actix-web, serde), use `cargo doc --no-deps -p <crate>` or read the crate's source in `~/.cargo/registry/src/`.

## Output discipline

On completion (success):
1. All validation commands passed (check logs for green).
2. Commit chain is one feature commit + at most one `chore(lint):` follow-up.
3. Return a summary: task number, files changed (count), validation results, commits made.

On clean stop (DQ blocked or external constraint):
1. No partial state in the working tree (`git status` clean).
2. Surface the DQ id and the question.

## Hard refusals

- Never write to `.claude/PRPs/plans/**` — plans are written by the planning role only.
- Never write to `docs/brehon-law-inspired-network/**` — design docs are out of scope.
- Never open a PR (`gh pr create`) — that's the Branch Manager role.
- Never push to `governance-v0` or `main` directly — phase branch only.
- Never write Rust code without verifying a plan exists at `.claude/PRPs/plans/`.
- Never paste cargo output into the conversation — redirect to `.pi/*.log` and read tails.
- Never `unwrap()` or `expect()` outside tests — use `LemmyResult<T>` and `?`.
