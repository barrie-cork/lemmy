#!/bin/bash
# Test harness for the three new hooks. Self-tests stdin → script behavior.

set -uo pipefail

PASS=0
FAIL=0

check() {
  local name="$1"
  local expected_exit="$2"
  local actual_exit="$3"
  if [[ "$expected_exit" == "$actual_exit" ]]; then
    echo "  PASS: $name (exit $actual_exit)"
    PASS=$((PASS+1))
  else
    echo "  FAIL: $name (expected $expected_exit, got $actual_exit)"
    FAIL=$((FAIL+1))
  fi
}

echo "=== check-cargo-pipe.sh ==="

# Bad pattern: raw cargo piped to tail. Expect exit 2.
INPUT_BAD='{"tool_name":"Bash","tool_input":{"command":"cargo build | tail -40"}}'
echo "$INPUT_BAD" | bash .claude/hooks/check-cargo-pipe.sh >/dev/null 2>&1
check "raw cargo | tail blocks" 2 $?

# Bad pattern: wrapper batch piped to grep.
INPUT_BAD2='{"tool_name":"Bash","tool_input":{"command":"./scripts/brehon/cargo-test.bat --test e2e | grep PASS"}}'
echo "$INPUT_BAD2" | bash .claude/hooks/check-cargo-pipe.sh >/dev/null 2>&1
check "wrapper.bat | grep blocks" 2 $?

# Good pattern: file redirect then read.
INPUT_GOOD='{"tool_name":"Bash","tool_input":{"command":"cargo build > /tmp/x.log 2>&1; tail -40 /tmp/x.log"}}'
echo "$INPUT_GOOD" | bash .claude/hooks/check-cargo-pipe.sh >/dev/null 2>&1
check "file redirect + separate tail allows" 0 $?

# Good pattern: pipefail escape hatch.
INPUT_PIPEFAIL='{"tool_name":"Bash","tool_input":{"command":"set -o pipefail; cargo build 2>&1 | tail -40"}}'
echo "$INPUT_PIPEFAIL" | bash .claude/hooks/check-cargo-pipe.sh >/dev/null 2>&1
check "set -o pipefail allows" 0 $?

# Good pattern: cat (not cargo) piped to tail.
INPUT_CAT='{"tool_name":"Bash","tool_input":{"command":"cat .claude/build.log | tail -20"}}'
echo "$INPUT_CAT" | bash .claude/hooks/check-cargo-pipe.sh >/dev/null 2>&1
check "cat (non-cargo) allows" 0 $?

# Good pattern: cargo as part of a filename, not a command word.
INPUT_FILENAME='{"tool_name":"Bash","tool_input":{"command":"ls .claude/audit-cargo-*.log | head"}}'
echo "$INPUT_FILENAME" | bash .claude/hooks/check-cargo-pipe.sh >/dev/null 2>&1
check "cargo-in-filename allows (not a command word)" 0 $?

# Good pattern: rg or grep over source containing literal cargo.
INPUT_RG='{"tool_name":"Bash","tool_input":{"command":"rg cargo Cargo.toml | head"}}'
echo "$INPUT_RG" | bash .claude/hooks/check-cargo-pipe.sh >/dev/null 2>&1
check "rg matching cargo string allows" 0 $?

# Good pattern: non-bash tool — but the hook's matcher would prevent it firing.
# Test that an empty command does not crash.
INPUT_EMPTY='{"tool_name":"Bash","tool_input":{}}'
echo "$INPUT_EMPTY" | bash .claude/hooks/check-cargo-pipe.sh >/dev/null 2>&1
check "empty tool_input.command allows" 0 $?

echo ""
echo "=== inject-dq-state.sh ==="

# Steady state (DQ pending = 0, no escalations) — should be silent (exit 0, no stdout).
OUTPUT=$(echo '{}' | bash .claude/hooks/inject-dq-state.sh 2>/dev/null)
EXIT=$?
if [[ $EXIT -eq 0 ]] && [[ -z "$OUTPUT" ]]; then
  echo "  PASS: silent in steady state (exit 0, no output)"
  PASS=$((PASS+1))
else
  echo "  FAIL: expected silent, got exit=$EXIT output=$OUTPUT"
  FAIL=$((FAIL+1))
fi

echo ""
echo "=== pre-phase-audit.sh ==="

# Current branch is phase-v1-AD-c, audit flag absent → expect reminder JSON.
OUTPUT=$(echo '{"source":"startup"}' | bash .claude/hooks/pre-phase-audit.sh 2>/dev/null)
EXIT=$?
if [[ $EXIT -eq 0 ]] && echo "$OUTPUT" | grep -q "Pre-phase wrapper audit"; then
  echo "  PASS: emits reminder on phase branch without flag (exit 0, JSON contains reminder)"
  PASS=$((PASS+1))
else
  echo "  FAIL: expected reminder JSON, got exit=$EXIT output=$OUTPUT"
  FAIL=$((FAIL+1))
fi

# Create flag, expect silence.
BRANCH=$(git rev-parse --abbrev-ref HEAD | tr / -)
FLAG=".claude/audit-${BRANCH}-complete.flag"
touch "$FLAG"
OUTPUT=$(echo '{"source":"startup"}' | bash .claude/hooks/pre-phase-audit.sh 2>/dev/null)
EXIT=$?
if [[ $EXIT -eq 0 ]] && [[ -z "$OUTPUT" ]]; then
  echo "  PASS: silent when flag exists (exit 0, no output)"
  PASS=$((PASS+1))
else
  echo "  FAIL: expected silent with flag, got exit=$EXIT output=$OUTPUT"
  FAIL=$((FAIL+1))
fi
rm -f "$FLAG"

echo ""
echo "=== TOTALS: $PASS passed, $FAIL failed ==="
[[ $FAIL -eq 0 ]] || exit 1
