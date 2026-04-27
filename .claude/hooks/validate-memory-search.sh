#!/usr/bin/env bash
# PreToolUse hook: enforce FTS5 limits on the legacy memory_search tool.
# Policy: memory_search_hybrid is the default for multi-word queries (semantic +
# FTS5 via RRF). memory_search is the exact-token FTS5 fallback, still capped at
# 2 words. This hook only constrains the legacy tool; hybrid calls pass through.
# See rules/shared/pmd-search-strategy.md for the full policy.
#
# Event: PreToolUse
# Matcher: mcp__project-memory__memory_search|mcp__vault-memory__memory_search
# Timeout: 5000
# Returns JSON with permissionDecision: "deny" to block bad calls.
#
# NOTE on matcher: the matcher above is an unanchored substring that also
# matches memory_search_hybrid. The script body below explicitly exempts any
# tool name ending in _hybrid, which is correct regardless of whether CC treats
# the matcher as regex or glob. Do NOT rely on end-anchoring the matcher alone.

set -o pipefail

INPUT=$(cat)

TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null) || exit 0

# Exempt memory_search_hybrid (and any future *_hybrid variant): semantic +
# FTS5 RRF handles multi-word queries correctly, so the FTS5 word-count limit
# does not apply.
case "$TOOL_NAME" in
  *_hybrid) exit 0 ;;
esac

QUERY=$(echo "$INPUT" | jq -r '.tool_input.query // empty' 2>/dev/null) || exit 0
TAGS=$(echo "$INPUT" | jq -r '.tool_input.tags // empty' 2>/dev/null) || exit 0
MEMORY_TYPE=$(echo "$INPUT" | jq -r '.tool_input.memory_type // empty' 2>/dev/null) || exit 0

WORD_COUNT=$(echo "$QUERY" | wc -w | tr -d ' ')
REASONS=""

# Allow type-filtered searches with relaxed word limit
if [ -n "$MEMORY_TYPE" ] && [ "$WORD_COUNT" -le 3 ]; then
  exit 0
fi

# Check word count (max 2 — allows compound terms like "docker deploy")
if [ "$WORD_COUNT" -gt 2 ]; then
  REASONS="Query has $WORD_COUNT words ('$QUERY'). Use 1-2 keywords max for better FTS5 recall."
fi

# Check tags present (unless memory_type filter is used as alternative)
if [ -z "$TAGS" ] && [ -z "$MEMORY_TYPE" ]; then
  if [ -n "$REASONS" ]; then
    REASONS="$REASONS Also missing tags."
  else
    REASONS="Missing tags parameter. Use one tag (e.g. 'junior', 'infrastructure', 'deployment', 'configuration')."
  fi
fi

if [ -n "$REASONS" ]; then
  jq -n --arg reason "$REASONS" '{
    hookSpecificOutput: {
      hookEventName: "PreToolUse",
      permissionDecision: "deny",
      permissionDecisionReason: $reason
    }
  }'
  exit 0
fi

# All checks passed — allow the call
exit 0
