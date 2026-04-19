#!/bin/bash

# PRP Ralph Stop Hook
# Prevents session exit when a Ralph loop is active
# Feeds the PRP plan execution prompt back for the next iteration

set -euo pipefail

# Read hook input from stdin
HOOK_INPUT=$(cat)

# State file location
STATE_FILE=".claude/prp-ralph.state.md"

# Check if Ralph loop is active
if [[ ! -f "$STATE_FILE" ]]; then
  # No active loop - allow exit
  exit 0
fi

# Parse YAML frontmatter from state file
FRONTMATTER=$(sed -n '/^---$/,/^---$/{ /^---$/d; p; }' "$STATE_FILE")

# Extract values
ITERATION=$(echo "$FRONTMATTER" | grep '^iteration:' | sed 's/iteration: *//')
MAX_ITERATIONS=$(echo "$FRONTMATTER" | grep '^max_iterations:' | sed 's/max_iterations: *//')
PLAN_PATH=$(echo "$FRONTMATTER" | grep '^plan_path:' | sed 's/plan_path: *//' | sed 's/^"\(.*\)"$/\1/')

# Validate numeric fields
if [[ ! "$ITERATION" =~ ^[0-9]+$ ]]; then
  echo "⚠️  PRP Ralph: State file corrupted (invalid iteration)" >&2
  rm "$STATE_FILE"
  exit 0
fi

if [[ ! "$MAX_ITERATIONS" =~ ^[0-9]+$ ]]; then
  echo "⚠️  PRP Ralph: State file corrupted (invalid max_iterations)" >&2
  rm "$STATE_FILE"
  exit 0
fi

# Check if max iterations reached
if [[ $MAX_ITERATIONS -gt 0 ]] && [[ $ITERATION -ge $MAX_ITERATIONS ]]; then
  echo "🛑 PRP Ralph: Max iterations ($MAX_ITERATIONS) reached."
  echo "   Check .claude/prp-ralph.state.md for progress log."
  rm "$STATE_FILE"
  exit 0
fi

# Get transcript path from hook input
TRANSCRIPT_PATH=$(echo "$HOOK_INPUT" | python3 -c "import sys,json; print(json.load(sys.stdin).get('transcript_path',''))")

if [[ ! -f "$TRANSCRIPT_PATH" ]]; then
  echo "⚠️  PRP Ralph: Transcript not found" >&2
  rm "$STATE_FILE"
  exit 0
fi

# Check for completion promise in last assistant message
if grep -q '"role":"assistant"' "$TRANSCRIPT_PATH"; then
  LAST_OUTPUT=$(grep '"role":"assistant"' "$TRANSCRIPT_PATH" | tail -1 | python3 -c "
import sys, json
try:
    msg = json.load(sys.stdin)
    parts = msg.get('message',{}).get('content',[])
    print('\n'.join(p.get('text','') for p in parts if p.get('type')=='text'))
except: pass
" 2>/dev/null || echo "")

  # Check for completion promise
  if echo "$LAST_OUTPUT" | grep -q '<promise>COMPLETE</promise>'; then
    echo "✅ PRP Ralph: All validations passed! Loop complete."
    rm "$STATE_FILE"
    exit 0
  fi
fi

# Not complete - continue loop
NEXT_ITERATION=$((ITERATION + 1))

# Update iteration in state file
TEMP_FILE="${STATE_FILE}.tmp.$$"
sed "s/^iteration: .*/iteration: $NEXT_ITERATION/" "$STATE_FILE" > "$TEMP_FILE"
mv "$TEMP_FILE" "$STATE_FILE"

# Build the prompt to feed back
PROMPT="# PRP Ralph Loop - Iteration $NEXT_ITERATION

## Your Task

Continue executing the PRP plan until ALL validations pass.

**Plan file**: \`$PLAN_PATH\`
**State file**: \`.claude/prp-ralph.state.md\`

## Instructions

1. **Read the plan file** - understand all tasks and validation requirements
2. **Check your previous work** - review files, git status, test outputs
3. **Identify what's incomplete** - which tasks/validations are still failing?
4. **Fix and implement** - address failures, complete remaining tasks
5. **Run ALL validations** - type-check, lint, tests, build
6. **Update progress** - mark tasks complete, add learnings to state file

## Validation Requirements

Run these (or equivalent from your plan):
\`\`\`bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
cargo test --test e2e
\`\`\`

## Completion

When ALL validations pass:
1. Generate implementation report
2. Archive the plan
3. Output: \`<promise>COMPLETE</promise>\`

If validations are still failing:
- Fix the issues
- End your response normally
- The loop will continue

**Do NOT output the completion promise if ANY validation is failing.**"

SYSTEM_MSG="🔄 PRP Ralph iteration $NEXT_ITERATION of $MAX_ITERATIONS | Plan: $PLAN_PATH"

# Output JSON to block exit and feed prompt back
python3 -c "
import json, sys
print(json.dumps({
    'decision': 'block',
    'reason': sys.argv[1],
    'systemMessage': sys.argv[2]
}))
" "$PROMPT" "$SYSTEM_MSG"

exit 0
