[role:bm-task] Option B smoke — verify model identifier in headless worker

## Scope

This is a verification-only smoke test for PMD #109 / Option B (the settings.json model-pin removal at commit 25044906d). The advisor needs to confirm that Junior workers branched from the patched governance-v0 actually run on Sonnet 4.6 instead of Opus 4.7.

Do NOT write any code. Do NOT open a PR. Do NOT modify any tracked file. Do exactly the following:

1. Read `.claude/settings.json` in this worktree. Confirm the file does NOT contain a `"model":` key. The expected top-level keys are: `effortLevel`, `skillListingBudgetFraction`, `hooks`. Report what you find.

2. Run `pwd` and `git log --oneline -1`. Report both.

3. State your own model identity. Look at any system message you received about your model, and report what it says — e.g. "I am Claude Sonnet 4.6" or "I am Claude Opus 4.7" or "model id reported: <X>".

4. Exit with no commits. The advisor will read the JSONL log of this task and grep the `"model":` field to verify the actual API model used.

## Required reading

- `.claude/settings.json` (read the worktree's local copy, not the user-scope one)

## Constraints

- Touch nothing. No edits, no writes, no commits.
- This task should complete in under 2 minutes.
- Do not invoke MCPs, do not run cargo, do not read other files.
- The success criterion is the worker completing without error and the log JSONL containing a `"model":"claude-sonnet-4-6"` (or `"claude-sonnet-4-6[1m]"`) field. Anything else (Opus, Haiku) means Option B did not propagate.
