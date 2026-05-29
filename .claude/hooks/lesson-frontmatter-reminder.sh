#!/usr/bin/env bash
# PostToolUse hook: advisory reminder when a .claude/lessons/ file is saved
# WITHOUT the YAML frontmatter that scripts/sync-lessons-to-pmd.sh requires.
#
# Event: PostToolUse
# Matcher: Edit|Write
# Timeout: 5000
#
# WHY: sync-lessons-to-pmd.sh imports feedback_*.md / reference_*.md into the
# PMD only when the file has a `---\n…\n---\n` frontmatter block with a
# non-empty `name:` field. A lesson authored in bare `# Title` + body style is
# SILENTLY SKIPPED on every sync (`errors: N`) and is therefore invisible to
# `memory_search_hybrid` recall — forever, with zero signal. On 2026-05-29, 8
# lessons were found in this exact state, unindexed for weeks (every session's
# semantic recall was missing them). Per
# session-retro-2026-05-29-harness-context-budget-trim.md §3 #5 +
# feedback_lessons_need_frontmatter_for_pmd_sync.md.
#
# This hook is the author-time nudge: it fires the moment a lesson file is
# written or edited into the no-frontmatter state, so the gap is caught when
# it's a one-line fix, not at audit time weeks later.
#
# Advisory ONLY — emits a reminder, never blocks (exit 0 always). A
# work-in-progress lesson the author will finish later is legitimate; the hook
# only surfaces the question. The weekly-review Step 1c sweep is the backstop
# for files that slip in via direct git operations (which fire no PostToolUse).
#
# SCOPE: only .claude/lessons/{feedback,reference}_*.md — the files the sync
# script globs (line `for lesson in "$LESSONS_DIR"/feedback_*.md
# "$LESSONS_DIR"/reference_*.md`). Other .claude/lessons/ files (e.g. a README)
# are not synced and not flagged.
#
# VALIDATION CONTRACT: mirrors sync-lessons-to-pmd.sh exactly — a file passes
# this lint IFF it would import cleanly:
#   1. starts with a `---\n … \n---\n` block (script regex
#      `^---\s*\n(.*?)\n---\s*\n(.*)$`), AND
#   2. that block contains a non-empty `name:` field (the script's separate
#      "missing 'name'" error path).
# Keep this in lockstep with the script. If the script's parser changes, update
# the awk check below + lesson-frontmatter-lint.sh (the shared sweep helper).

set -o pipefail

INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null) || exit 0

case "$TOOL_NAME" in
  Edit|Write) : ;;
  *) exit 0 ;;
esac

FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null) || exit 0
[ -z "$FILE_PATH" ] && exit 0

# Normalise separators (Windows bash can see backslashes) and match only the
# synced lesson files: .claude/lessons/feedback_*.md or .claude/lessons/reference_*.md.
NORM=$(printf '%s' "$FILE_PATH" | tr '\\' '/')
case "$NORM" in
  *.claude/lessons/feedback_*.md|*.claude/lessons/reference_*.md) : ;;
  *) exit 0 ;;
esac

# From here on use $NORM (forward-slash form) — a raw backslash path fails
# `[ -f ]` and the python open() under git-bash. The file should exist
# post-Edit/Write; if it doesn't (race / deletion), skip.
[ -f "$NORM" ] || exit 0

# Run the shared validation (same contract the sync script enforces). The helper
# prints "OK" on a clean file or a "MISSING: …" reason on a broken one.
# The helper lives at scripts/brehon/ (not .claude/hooks/). This hook is at
# .claude/hooks/, so the repo root is two levels up from $0 — resolve from the
# hook's own location, NOT $CWD (a PostToolUse hook's CWD is not guaranteed).
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$HOOK_DIR/../.." && pwd)"
HELPER="$REPO_ROOT/scripts/brehon/lesson-frontmatter-lint.sh"
if [ ! -f "$HELPER" ]; then
  exit 0  # helper not present — don't fail the edit over a missing advisory
fi

REASON=$(bash "$HELPER" --one "$NORM" 2>/dev/null) || REASON=""
case "$REASON" in
  OK|"") exit 0 ;;
esac
# Strip the "MISSING: " prefix so the sentence reads naturally.
REASON_TEXT="${REASON#MISSING: }"

BASENAME=$(basename "$NORM")
echo "Heads-up: lesson ${BASENAME} — ${REASON_TEXT}. scripts/sync-lessons-to-pmd.sh will SILENTLY SKIP it (counts as 'errors: N'), so memory_search_hybrid can never recall it — this is the class that stranded 8 lessons for weeks (2026-05-29). Add a YAML frontmatter block at the very top (the sync reads flat top-level keys — name + description + type, matching the existing lesson corpus):
---
name: <short title — becomes the PMD memory title>
description: <one-line summary for recall>
type: feedback
---
(then keep the body below). If this is a WIP you'll finish before committing, ignore. Backstop: weekly-review Step 1c + scripts/brehon/lesson-frontmatter-lint.sh sweep the whole dir."
exit 0
