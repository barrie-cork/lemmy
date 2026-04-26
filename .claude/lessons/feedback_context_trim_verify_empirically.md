---
name: Verify context-trim assumptions empirically before designing scope
description: Feedback rule — don't trust documentation claims about Claude Code internals; measure in a fresh session with /context
type: feedback
originSessionId: 54245922-a49f-4cd5-9d75-f35c34d1573c
---
When optimizing context cost (trimming CLAUDE.md, scoping rules via `paths:`, archiving retired rules), **verify every load-behavior claim empirically in a fresh session** before designing scope boundaries. Documentation — both Anthropic's and our own — has proven wrong at least once.

**Why:** Phase A (commit `eb8ab5fc1`) archived `task-hopper.md` into `.claude/rules/archived/` under the documented assumption that subdirectories wouldn't auto-load. They do. The archive trim delivered ~0 tokens of savings until the directory was moved out of `rules/` entirely during Phase B verification. A fresh-session /context measurement caught it; relying on the documented behavior would have shipped the "savings" invisibly broken.

**How to apply:** Before committing a context-tooling change, propose empirical tests a real user can run: set up an instrumented rule (e.g. `_test_trigger.md` scoped via `paths:`), then in a fresh session have the user Read/Grep/Edit the target and paste back the Memory files bucket from `/context`. The user's transcript is ground truth; the Claude-side context injection you see mid-session is not, because rules are snapshotted at session start. Test-drive the scope BEFORE designing around it, not after.

**Concrete patterns the test revealed (2026-04-23):**
- `.claude/rules/` recurses into subdirectories (archive outside the dir)
- `paths:` frontmatter is include-only, no `!pattern` negation
- `paths:` loads only on Read events (not Grep, Glob, or Edit)

See `reference_claude_code_rules_loading.md` for full measurements.
