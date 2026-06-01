---
name: Lessons need YAML frontmatter or sync silently skips them (invisible to PMD recall)
description: Feedback rule — scripts/sync-lessons-to-pmd.sh imports a .claude/lessons/{feedback,reference}_*.md file only if it has a leading ---\n…\n---\n frontmatter block with a non-empty name: field; a bare # Title + body lesson is silently skipped (errors: N) on every sync and is forever invisible to memory_search_hybrid recall with zero signal; enforced author-time by the lesson-frontmatter-reminder.sh PostToolUse hook + backstopped by scripts/brehon/lesson-frontmatter-lint.sh (weekly-review Step 1c)
type: feedback
---
# Lessons need YAML frontmatter or the PMD sync silently skips them

**Context:** `.claude/lessons/feedback_*.md` and `reference_*.md` files are the durable lesson corpus. They reach `memory_search_hybrid` recall only via `scripts/sync-lessons-to-pmd.sh`, which parses each file's YAML frontmatter and inserts a `memory_type: "pattern"` row. The sync's parser is the contract — and it is strict.

## The defect class

A lesson authored in bare `# Title` + body style (no frontmatter) is **silently skipped** by the sync. The only signal is the `errors: N` count in the sync's final line — no per-file error unless you pass `--verbose`, and the file stays on disk looking perfectly fine. It is therefore **invisible to `memory_search_hybrid` recall forever**, even though it's committed, tracked, and visible in the editor.

On 2026-05-29, **8 lessons** were found in exactly this state — unindexed for weeks. Every session's semantic recall had been missing them. They had been authored in an H1+TL;DR style that the sync regex rejects:

```
imported: 0   skipped: 157   errors: 8   ← the 8 broken files, on every sync
```

After prepending frontmatter:

```
imported: 8   skipped: 157   errors: 0
```

## What the sync actually requires (the contract)

From `scripts/sync-lessons-to-pmd.sh::parse_lesson`, two checks, in order:

1. **Frontmatter block present.** The file must match the regex
   `^---\s*\n(.*?)\n---\s*\n(.*)$` (DOTALL) — i.e. line 1 is `---`, there's a closing `---`, body follows. No leading blank lines, no `# Title` before the `---`.
2. **Non-empty `name:`.** The frontmatter block must contain a `name:` field with a non-empty value (it becomes the PMD memory title + the idempotency key). A missing/empty `name` is a separate `errors+=1` path.

The parser reads **flat top-level keys** (`fm.get("name")`, `fm.get("description")`, `fm.get("type")`) — so the canonical corpus form is flat:

```yaml
---
name: <short title — becomes the PMD memory title>
description: <one-line summary for recall>
type: feedback
---
```

A **nested** `metadata:\n  type: feedback` (the auto-memory MEMORY.md convention) leaves `type` empty in the sync (tags become `lesson,` — harmless but inconsistent with the 165-file corpus). Use the flat `type: feedback` for lesson files. `name` + `description` are the load-bearing fields; `type` only affects the tag suffix.

## The structural fix (shipped 2026-05-29)

Two-layer defence, mirroring the rule-narrative-bloat hook from the same session:

1. **Author-time PostToolUse hook** — `.claude/hooks/lesson-frontmatter-reminder.sh` (matcher `Edit|Write`, advisory, exit 0 always). Fires the moment a `feedback_*.md`/`reference_*.md` lesson is written/edited into the no-frontmatter (or empty-`name`) state, with a copy-pasteable frontmatter template. Catches the common case at the cheapest moment — when it's a one-line fix, not weeks later at audit time.
2. **Backstop sweep** — `scripts/brehon/lesson-frontmatter-lint.sh` scans the whole dir; exit 2 if any file would be skipped. Invoked by **weekly-review Step 1c** for files that slip in via direct git operations (a `git mv`, a merge, a manual editor write) which fire no PostToolUse hook.

Both validate against a **verbatim copy** of the sync's regex + `name:` extraction, so "passes the lint" ≡ "imports cleanly." If the sync's parser ever changes, update the lint helper's Python block + the hook in lockstep (the helper is the single source of truth; the hook calls it).

## How to apply

- **Authoring a new lesson:** start the file with the flat frontmatter block above. The hook will nudge you if you forget.
- **After authoring/syncing:** confirm `errors: 0` from `scripts/sync-lessons-to-pmd.sh`, then run the backfill (`dist/scripts/backfill.js`) so the new row is embedded, not just FTS5-indexed (per [[feedback_pmd_backfill_after_write]]).
- **Auditing the corpus:** `bash scripts/brehon/lesson-frontmatter-lint.sh` — a clean run prints "all N lesson file(s) have valid frontmatter."

## Why this matters

The lesson corpus is the substrate the whole Recursive Learning System recalls from. A lesson that can't be recalled is worse than no lesson — it consumes authoring effort and creates false confidence that the knowledge is "captured" when no `memory_search_hybrid` will ever surface it. The silent-skip is the dangerous part: there is no error, no broken build, no failed test — just a slow, invisible erosion of recall coverage. The enforcement converts a paper convention into a loud author-time signal. Related: [[feedback_lesson_must_pair_with_structural_fix_when_fixable]] (this lesson ships its own fix), [[feedback_rule_narrative_to_refs_at_author_time]] (the sibling author-time hook from the same session).
