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

**MEMORY.md is byte-limited, not line-limited (2026-05-29):** the `memory-prune` skill documented a
"200-line hard limit (lines past 200 silently truncated)". Observed reality: a 186-line file (under
200) at 25375 bytes was **truncated** with a SessionStart warning `MEMORY.md is 25.2KB (limit:
24.4KB) — index entries are too long. Only part of it was loaded.` The binding limit is **bytes
(~24400)**, not lines; at ~135 bytes/line the byte budget binds first (~180 lines of average-length
entries). A lines-only gate reported 14 lines of headroom while the file was actively truncating.
Fix shipped: `memory-prune` now measures both, gates on bytes, and treats the Verbose-entry class as
a primary byte-reclaim lever (shorten longest entries / move long stale entries to Historical —
removing short entries barely moves the byte total). Same root pattern as the rules-subdirectory
case: the documented limit (lines) did not match observed load behavior (bytes); measure the
constraint that actually fires.

**Reading the signal:** a rule auto-loading as a `<system-reminder>`
mid-session is the `paths:`-on-Read mechanism working correctly — not
noise. The natural read is "the harness loaded the right governance for
the action in progress." On 2026-05-22, editing
`.claude/PRPs/handovers/context-prune-option-b-2026-05-22.md` triggered
`.claude/rules/handover.md` to load as a system-reminder. That signal
confirmed the handover size was within the canonical 150-350 line range
and the structure aligned with exemplars listed in `handover.md`
§"Canonical exemplars" — without a separate manual cross-check. Future
sessions should treat such mid-session rule loads as confirmatory signals
about action correctness, not interruptions.
