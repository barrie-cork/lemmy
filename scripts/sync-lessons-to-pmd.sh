#!/usr/bin/env bash
# sync-lessons-to-pmd.sh — import .claude/lessons/feedback_*.md into PMD
#
# Reads every feedback_*.md lesson file, parses YAML frontmatter
# (name, description, type), and inserts as memory_type='pattern' rows
# in .project-memory/memory.db.
#
# Idempotent: re-running skips lessons whose title (taken from the
# frontmatter `name` field) already exists in the DB. To force a
# re-import after editing a lesson, delete the row first:
#   sqlite3 .project-memory/memory.db \
#     "DELETE FROM memories WHERE title = 'Lesson title here';"
#
# Embeddings: this script writes to the `memories` table only. The
# MCP server's `memory_vectors` table is populated when memories are
# written via the MCP tool with Ollama reachable. As of 2026-05-09 the
# laptop's MCP server cannot reach the EliteDesk's Ollama, so writes
# from this script (and from MCP calls on the laptop) leave
# memory_vectors empty. memory_search_hybrid auto-falls-back to FTS5,
# so lessons remain searchable by keyword. Wire Ollama in a future
# session to backfill embeddings.
#
# Usage:
#   bash scripts/sync-lessons-to-pmd.sh         # import any new lessons
#   bash scripts/sync-lessons-to-pmd.sh --dry-run  # show what would be imported
#   bash scripts/sync-lessons-to-pmd.sh --verbose  # show every action
#
# Exit codes:
#   0  — success (some new lessons imported, or all already present)
#   1  — DB unreachable, lessons dir missing, or sqlite3 not on PATH
#   2  — frontmatter parse error on at least one file (other lessons still imported)

set -euo pipefail

# Force UTF-8 across all child Python invocations (Windows defaults to cp1252,
# which fails on non-ASCII characters in lesson bodies — e.g. the → arrow).
# Per feedback_python_utf8_encoding_windows.md.
export PYTHONIOENCODING=utf-8
export PYTHONUTF8=1

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DB="${REPO_ROOT}/.project-memory/memory.db"
LESSONS_DIR="${REPO_ROOT}/.claude/lessons"
DRY_RUN=0
VERBOSE=0

for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    --verbose) VERBOSE=1 ;;
    -h|--help)
      sed -n '1,30p' "${BASH_SOURCE[0]}"
      exit 0
      ;;
    *)
      echo "unknown arg: $arg" >&2
      exit 1
      ;;
  esac
done

command -v sqlite3 >/dev/null 2>&1 || {
  echo "ERROR: sqlite3 not on PATH; install sqlite3 first" >&2
  exit 1
}

[ -f "$DB" ] || {
  echo "ERROR: PMD not found at $DB" >&2
  exit 1
}

[ -d "$LESSONS_DIR" ] || {
  echo "ERROR: lessons dir not found at $LESSONS_DIR" >&2
  exit 1
}

# Parse frontmatter + body from a markdown file.
# Frontmatter is YAML between leading '---' delimiters.
# Echoes three null-separated fields: name|description|type|body
parse_lesson() {
  local file="$1"
  python3 - "$file" <<'PY'
import sys, re, json, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
path = sys.argv[1]
text = open(path, encoding='utf-8').read()
m = re.match(r'^---\s*\n(.*?)\n---\s*\n(.*)$', text, re.DOTALL)
if not m:
    print(json.dumps({"error": "no frontmatter"}))
    sys.exit(0)
fm_raw, body = m.group(1), m.group(2).strip()
fm = {}
for line in fm_raw.splitlines():
    if ':' in line:
        k, _, v = line.partition(':')
        fm[k.strip()] = v.strip().strip('"').strip("'")
out = {
    "name": fm.get("name", ""),
    "description": fm.get("description", ""),
    "type": fm.get("type", ""),
    "body": body,
}
print(json.dumps(out, ensure_ascii=False))
PY
}

# SQL-escape a string for inline use (single quotes doubled).
sql_escape() {
  printf "%s" "$1" | sed "s/'/''/g"
}

imported=0
skipped=0
errors=0

for lesson in "$LESSONS_DIR"/feedback_*.md "$LESSONS_DIR"/reference_*.md; do
  [ -e "$lesson" ] || continue
  parsed_json="$(parse_lesson "$lesson")"

  if echo "$parsed_json" | python3 -c "import sys,json; d=json.load(sys.stdin); sys.exit(1 if d.get('error') else 0)"; then
    name="$(echo "$parsed_json" | python3 -c "import sys,json; print(json.load(sys.stdin)['name'])")"
    desc="$(echo "$parsed_json" | python3 -c "import sys,json; print(json.load(sys.stdin)['description'])")"
    ltype="$(echo "$parsed_json" | python3 -c "import sys,json; print(json.load(sys.stdin)['type'])")"
    body="$(echo "$parsed_json" | python3 -c "import sys,json; print(json.load(sys.stdin)['body'])")"
  else
    echo "WARN: could not parse frontmatter in $lesson; skipping" >&2
    errors=$((errors + 1))
    continue
  fi

  if [ -z "$name" ]; then
    echo "WARN: lesson $lesson missing 'name' frontmatter; skipping" >&2
    errors=$((errors + 1))
    continue
  fi

  # Idempotency check: skip if a memory with this title already exists.
  exists="$(sqlite3 "$DB" "SELECT COUNT(*) FROM memories WHERE title = '$(sql_escape "$name")';")"
  if [ "$exists" -gt 0 ]; then
    [ "$VERBOSE" -eq 1 ] && echo "skip (exists): $name"
    skipped=$((skipped + 1))
    continue
  fi

  if [ "$DRY_RUN" -eq 1 ]; then
    echo "would import: $name (from $(basename "$lesson"))"
    imported=$((imported + 1))
    continue
  fi

  # Insert. Tags: 'lesson' + the frontmatter type (e.g. 'feedback', 'pattern').
  tags="lesson,${ltype}"
  filename="$(basename "$lesson")"

  # The body becomes content; description goes as a content prefix so
  # FTS5 picks up both signals naturally.
  full_content="**Description:** ${desc}

${body}"

  sqlite3 "$DB" <<SQL
INSERT INTO memories (
  repo_name, memory_type, title, content, tags, source_type, source_ref,
  branch, file_path, importance, confidence
) VALUES (
  'brehon-fork', 'pattern',
  '$(sql_escape "$name")',
  '$(sql_escape "$full_content")',
  '$(sql_escape "$tags")',
  'lesson-import',
  'sync-lessons-to-pmd.sh',
  '',
  '.claude/lessons/$(sql_escape "$filename")',
  3, 1.0
);
SQL

  imported=$((imported + 1))
  [ "$VERBOSE" -eq 1 ] && echo "imported: $name"
done

echo "---"
if [ "$DRY_RUN" -eq 1 ]; then
  echo "DRY RUN: would import $imported lessons; $skipped already in DB"
else
  echo "imported: $imported   skipped: $skipped   errors: $errors"
fi

[ "$errors" -gt 0 ] && exit 2
exit 0
