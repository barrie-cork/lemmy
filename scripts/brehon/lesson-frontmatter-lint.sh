#!/usr/bin/env bash
# lesson-frontmatter-lint.sh — verify .claude/lessons/{feedback,reference}_*.md
# carry the YAML frontmatter that scripts/sync-lessons-to-pmd.sh requires to
# import them into the PMD.
#
# WHY: sync-lessons-to-pmd.sh SILENTLY SKIPS (counts as `errors: N`) any lesson
# file that lacks a leading `---\n…\n---\n` frontmatter block with a non-empty
# `name:` field. A skipped lesson is invisible to memory_search_hybrid recall —
# forever, with no signal beyond the sync's error count. On 2026-05-29, 8
# lessons were found in this state, unindexed for weeks. This lint is the
# backstop sweep (the PostToolUse hook lesson-frontmatter-reminder.sh catches
# the author-time case; this catches files that slipped in via direct git ops,
# which fire no hook). Per
# session-retro-2026-05-29-harness-context-budget-trim.md §3 #5 +
# feedback_lessons_need_frontmatter_for_pmd_sync.md.
#
# VALIDATION CONTRACT — mirrors sync-lessons-to-pmd.sh::parse_lesson EXACTLY:
#   1. file matches the sync regex `^---\s*\n(.*?)\n---\s*\n(.*)$` (DOTALL), AND
#   2. the frontmatter block contains a non-empty `name:` field.
# A file passes this lint IFF it would import cleanly. If the sync script's
# parser changes, update the Python block below in lockstep (it is a deliberate
# copy of the script's regex + name-extraction so "passes lint" ≡ "imports").
#
# Modes:
#   lesson-frontmatter-lint.sh                 # sweep: scan all lesson files.
#                                              # Prints one line per BROKEN file.
#                                              # Exit 0 = all clean; exit 2 = >=1 broken.
#   lesson-frontmatter-lint.sh --one <file>    # single-file: print `OK` or
#                                              # `MISSING: <reason>` to stdout.
#                                              # Exit 0 always (advisory; the
#                                              # caller — the hook — decides).
#   lesson-frontmatter-lint.sh --verbose       # sweep + also print OK lines.
#   lesson-frontmatter-lint.sh -h|--help
#
# Exit codes (sweep mode):
#   0  — every synced lesson file has valid frontmatter
#   1  — lessons dir missing, or python3 not on PATH
#   2  — at least one lesson file would be skipped by the sync (broken frontmatter)

set -o pipefail

export PYTHONIOENCODING=utf-8
export PYTHONUTF8=1

# This script lives at scripts/brehon/, so the repo root is two levels up.
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LESSONS_DIR="${REPO_ROOT}/.claude/lessons"

MODE="sweep"
ONE_FILE=""
VERBOSE=0

while [ $# -gt 0 ]; do
  case "$1" in
    --one) MODE="one"; shift; ONE_FILE="${1:?--one requires a path}" ;;
    --verbose) VERBOSE=1 ;;
    -h|--help) sed -n '1,40p' "${BASH_SOURCE[0]}"; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
  shift
done

command -v python3 >/dev/null 2>&1 || {
  echo "ERROR: python3 not on PATH" >&2
  exit 1
}

# check_one <file> — echoes "OK" or "MISSING: <reason>". Always rc 0.
# The regex + name extraction is a verbatim copy of sync-lessons-to-pmd.sh.
check_one() {
  local file="$1"
  python3 - "$file" <<'PY'
import sys, re, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
path = sys.argv[1]
try:
    text = open(path, encoding='utf-8').read()
except FileNotFoundError:
    print("MISSING: file not found"); sys.exit(0)
# EXACT mirror of sync-lessons-to-pmd.sh::parse_lesson regex.
m = re.match(r'^---\s*\n(.*?)\n---\s*\n(.*)$', text, re.DOTALL)
if not m:
    print("MISSING: has no YAML frontmatter block (must start with '---' and have a closing '---')")
    sys.exit(0)
fm_raw = m.group(1)
fm = {}
for line in fm_raw.splitlines():
    if ':' in line:
        k, _, v = line.partition(':')
        fm[k.strip()] = v.strip().strip('"').strip("'")
name = fm.get("name", "")
if not name:
    print("MISSING: frontmatter present but 'name:' field is empty or absent")
    sys.exit(0)
print("OK")
PY
}

if [ "$MODE" = "one" ]; then
  check_one "$ONE_FILE"
  exit 0
fi

# Sweep mode.
[ -d "$LESSONS_DIR" ] || {
  echo "ERROR: lessons dir not found at $LESSONS_DIR" >&2
  exit 1
}

broken=0
checked=0
for lesson in "$LESSONS_DIR"/feedback_*.md "$LESSONS_DIR"/reference_*.md; do
  [ -e "$lesson" ] || continue
  checked=$((checked + 1))
  result="$(check_one "$lesson")"
  base="$(basename "$lesson")"
  case "$result" in
    OK)
      [ "$VERBOSE" -eq 1 ] && echo "OK: $base"
      ;;
    *)
      echo "BROKEN: $base — ${result#MISSING: }"
      broken=$((broken + 1))
      ;;
  esac
done

echo "---"
if [ "$broken" -gt 0 ]; then
  echo "$broken of $checked lesson file(s) would be SILENTLY SKIPPED by sync-lessons-to-pmd.sh (invisible to memory_search_hybrid)."
  echo "Fix: add a frontmatter block (name/description/metadata.type) to the top of each, then re-run scripts/sync-lessons-to-pmd.sh."
  exit 2
fi
echo "all $checked lesson file(s) have valid frontmatter — sync would import cleanly"
exit 0
