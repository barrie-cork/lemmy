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
# A file passes this lint (`OK`) IFF it would import cleanly. If the sync
# script's parser changes, update the Python block below in lockstep (it is a
# deliberate copy of the script's regex + name-extraction so "passes" ≡ "imports").
#
# Additionally flags (advisory, NON-fatal) the nested-`metadata.type` drift
# class: a lesson whose `type:` lives only under a nested `metadata:` key and not
# at the top level. The sync STILL imports these (its flat partition-parser
# strips leading indent, and the metadata.type fallback reads them explicitly),
# so they are NOT stranded — but they diverge from the top-level convention the
# rest of the corpus uses. This class is taught by the System-1 memory-write
# instruction block and is wrong when copied to a .claude/lessons/ file.
#
# Modes:
#   lesson-frontmatter-lint.sh                 # sweep: scan all lesson files.
#                                              # Prints one BROKEN line per
#                                              # would-be-skipped file + one
#                                              # NESTED line per nested-metadata
#                                              # file (advisory).
#                                              # Exit 0 = no BROKEN; exit 2 = >=1
#                                              # BROKEN (NESTED alone never exits 2).
#   lesson-frontmatter-lint.sh --one <file>    # single-file: print `OK`,
#                                              # `MISSING: <reason>`, or
#                                              # `WARN-NESTED: <reason>` to stdout.
#                                              # Exit 0 always (advisory; the
#                                              # caller — the hook — decides).
#   lesson-frontmatter-lint.sh --verbose       # sweep + also print OK lines.
#   lesson-frontmatter-lint.sh -h|--help
#
# Exit codes (sweep mode):
#   0  — every synced lesson file imports (no BROKEN; NESTED advisories allowed)
#   1  — lessons dir missing, or python3 not on PATH
#   2  — at least one lesson file would be SILENTLY SKIPPED by the sync (BROKEN
#         frontmatter). Nested-metadata.type files alone do NOT trigger exit 2 —
#         they import and are advisory-only.

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

# check_one <file> — echoes "OK", "MISSING: <reason>", or "WARN-NESTED: <reason>".
# Always rc 0.
#   OK          — top-level frontmatter present (imports cleanly, canonical shape).
#   MISSING:    — no frontmatter block or empty/absent name (the sync SILENTLY
#                 SKIPS this — the hard-broken class; sweep counts it + exits 2).
#   WARN-NESTED — name present but `type` lives ONLY under a nested `metadata:`
#                 key, not at the top level. The sync DOES still import this
#                 (its flat partition-parser strips leading indent, and the
#                 §1.4 fallback reads metadata.type explicitly), so it is NOT
#                 invisible to recall — but it diverges from the top-level
#                 convention used by the rest of the corpus and is fragile.
#                 Advisory only: sweep prints it but does NOT count it as broken
#                 / does NOT exit 2; the reminder hook surfaces it as a heads-up.
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
# Indent-aware type-shape check. The flat parser above CANNOT distinguish a
# top-level `type:` from a nested `  type:` (k.strip() drops the indent), so we
# re-scan the raw frontmatter by column:
#   top_type    — a `type:` at column 0 (the canonical, top-level shape).
#   nested_type — a `type:` indented under a `metadata:` block (the drift class
#                 taught by the System-1 memory-write instruction block, wrong
#                 when copied to a .claude/lessons/ file).
lines = fm_raw.splitlines()
top_type = any(re.match(r'^type:\s*\S', ln) for ln in lines)
has_metadata = any(re.match(r'^metadata:\s*$', ln) for ln in lines)
nested_type = has_metadata and any(re.match(r'^\s+type:\s*\S', ln) for ln in lines)
if not top_type and nested_type:
    print("WARN-NESTED: 'type:' is nested under 'metadata:' but absent at the top level; flatten to a top-level 'type:' (the convention the rest of the lesson corpus uses)")
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
nested=0
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
    WARN-NESTED:*)
      # Advisory only — the file STILL imports (flat-parser indent-strip +
      # the §1.4 metadata.type fallback), so it is NOT counted as broken and
      # does NOT trigger exit 2. Surface it so it gets flattened to the
      # top-level convention.
      echo "NESTED: $base — ${result#WARN-NESTED: }"
      nested=$((nested + 1))
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
  echo "Fix: add a top-level frontmatter block (name/description/type) to the top of each, then re-run scripts/sync-lessons-to-pmd.sh."
  [ "$nested" -gt 0 ] && echo "Plus $nested file(s) with nested 'metadata.type' (NESTED lines above) — these import but diverge from the top-level convention; flatten them too."
  exit 2
fi
if [ "$nested" -gt 0 ]; then
  echo "all $checked lesson file(s) import cleanly, but $nested use nested 'metadata.type' (NESTED lines above)."
  echo "These are not stranded (the sync reads them via the flat parser + metadata.type fallback) but diverge from the top-level 'type:' convention — flatten when convenient. Non-fatal."
  exit 0
fi
echo "all $checked lesson file(s) have valid top-level frontmatter — sync would import cleanly"
exit 0
