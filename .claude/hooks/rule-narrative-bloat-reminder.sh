#!/usr/bin/env bash
# PostToolUse hook: advisory reminder to externalise narrative when an edit to an
# always-load .claude/rules/*.md file adds a large prose block.
#
# Event: PostToolUse
# Matcher: Edit|Write
# Timeout: 5000
#
# WHY: the always-load rules corpus is ~35% of the ~200K effective context budget
# (measured 2026-05-29). Rule files bloat one incident-narrative at a time. The
# canonical-schema-first gate (advisor-orchestrator.md §3.6) says: keep the terse
# RULE STATEMENT inline, author incident narratives / FP-FN taxonomies / locked-
# decision rationale / worked examples in .claude/refs/ with a one-line pointer.
# This hook is the mechanical nudge for that gate. Per
# .claude/lessons/feedback_rule_narrative_to_refs_at_author_time.md.
#
# Advisory ONLY — emits additionalContext, never blocks (exit 0 always). A large
# addition can be a legitimate new rule statement; the human/retro decides. The
# hook only surfaces the question so the author considers refs/ at author time
# rather than at audit time.
#
# SCOPE: only fires for .claude/rules/*.md (the always-load corpus). SCOPED rule
# files (with paths: frontmatter) and .claude/refs/ are intentionally NOT flagged
# — refs/ IS the externalisation target, and SCOPED files don't auto-load.

set -o pipefail

# Tunable: how many added lines counts as "large narrative". A rule statement is
# usually 1-3 lines; a narrative block is 8+. Default 8, override via env.
THRESHOLD="${RULE_NARRATIVE_BLOAT_THRESHOLD:-8}"

INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null) || exit 0

case "$TOOL_NAME" in
  Edit|Write) : ;;
  *) exit 0 ;;
esac

# Resolve the edited file path from the tool input (Edit + Write both carry file_path).
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null) || exit 0
[ -z "$FILE_PATH" ] && exit 0

# Normalise separators (Windows bash can see backslashes) and match the always-load
# rules corpus: any path containing `.claude/rules/` and ending `.md`. Matches both
# absolute (C:/.../.claude/rules/x.md) and repo-relative (.claude/rules/x.md) forms.
# Subdirs recurse (auto-load does too). .claude/refs/ is intentionally excluded.
NORM=$(printf '%s' "$FILE_PATH" | tr '\\' '/')
case "$NORM" in
  *.claude/rules/*.md) : ;;
  *) exit 0 ;;
esac

# Skip SCOPED rule files — they don't auto-load, so narrative there costs nothing
# at session start. SCOPED = has a `paths:` key in YAML frontmatter (first ~10 lines).
if head -10 "$FILE_PATH" 2>/dev/null | grep -qE '^paths:'; then
  exit 0
fi

# Measure the size of the addition.
# - Edit: count newlines in new_string minus old_string (net lines added).
# - Write: can't cheaply diff; use a coarser signal (only nudge on Write if the
#   whole file is large AND has few refs pointers — but Write of a rule file is
#   rare and usually a deliberate rewrite, so we skip Write to avoid false noise).
if [ "$TOOL_NAME" = "Write" ]; then
  exit 0
fi

NEW_LINES=$(echo "$INPUT" | jq -r '.tool_input.new_string // ""' 2>/dev/null | grep -c '' 2>/dev/null || echo 0)
OLD_LINES=$(echo "$INPUT" | jq -r '.tool_input.old_string // ""' 2>/dev/null | grep -c '' 2>/dev/null || echo 0)
ADDED=$((NEW_LINES - OLD_LINES))

[ "$ADDED" -lt "$THRESHOLD" ] && exit 0

# The addition is large. Does the new_string already carry a refs/ pointer
# (i.e. the author already externalised)? If so, no nudge.
if echo "$INPUT" | jq -r '.tool_input.new_string // ""' 2>/dev/null | grep -qE '\.claude/refs/|refs/[a-z]'; then
  exit 0
fi

BASENAME=$(basename "$FILE_PATH")
echo "Heads-up: this edit added ~${ADDED} lines to always-load rule ${BASENAME} with no refs/ pointer. The rules corpus is ~35% of the 200K budget — if these lines are incident narrative / FP-FN taxonomy / locked-decision rationale / a worked example (the WHY), author them in .claude/refs/${BASENAME%.md}-incidents.md with a one-line pointer and keep only the terse rule statement + heading inline (advisor-orchestrator.md §3.6 + feedback_rule_narrative_to_refs_at_author_time.md). If these ARE the rule statement, ignore this."
exit 0
