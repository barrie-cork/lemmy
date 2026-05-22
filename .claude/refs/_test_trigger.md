# Sentinel probe — context-injection trigger test

> **Purpose:** verify whether `.claude/refs/` auto-loads into the harness
> context budget at session start. Per
> `.claude/PRPs/handovers/context-prune-option-b-2026-05-22.md` Step 1
> + `feedback_context_trim_verify_empirically.md` (PMD #147).

## What to do with this file

1. After commit + push, close the current Claude Code session.
2. Open a fresh session in `C:/Users/barri/Developer/brehon-fork`.
3. Run `/context` in the fresh session.
4. Look for the **sentinel string below** in the "Memory files" list,
   OR for this file's path under loaded resources.

## Sentinel string

`SENTINEL_REFS_PROBE_20260523_C7F4B91A`

## Pass / Fail

- **PASS** = sentinel string is NOT in the Memory files list AND this
  file's path is NOT loaded. `.claude/refs/` does not auto-load.
  → Proceed to handover Step 2 (move auto-phase.md + auto-roadmap.md
  into `.claude/refs/`).
- **FAIL** = sentinel string IS present OR the file path IS loaded.
  `.claude/refs/` auto-loads recursively. → STOP. Revert plan; re-scope
  Option B around in-rule compression.

## Cleanup

Whether PASS or FAIL, delete this file (`git rm .claude/refs/_test_trigger.md`)
+ commit + push **before** any further Option-B work.

## Authored

Advisor session on `governance-v0`, 2026-05-22, head ~`84a2698f3` +
this commit.
