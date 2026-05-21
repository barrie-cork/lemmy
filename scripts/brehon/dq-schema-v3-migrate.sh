#!/usr/bin/env bash
# dq-schema-v3-migrate.sh — additive idempotent migration from schema-v2 to
# schema-v3. Adds id_v1, approved_by, approved_at to every entry.
#
# Usage:
#   ./scripts/brehon/dq-schema-v3-migrate.sh [--dry-run] [--file <path>]
#
# Options:
#   --dry-run      Print a summary of changes without writing the file.
#   --file <path>  Path to the decision-queue.json to migrate (default:
#                  .claude/decision-queue.json relative to repo root).
#
# Schema-v3 additions (additive only; never removes or renames fields):
#   id_v1       integer alias for pre-v3 entries (copies value of id).
#   approved_by null on all pre-v3 entries.
#   approved_at null on all pre-v3 entries.
#   schema_version bumped 2 → 3.
#
# Idempotency: if schema_version == 3, prints "already v3 — no-op", exits 0.
#
# Migration runs on the live .claude/decision-queue.json ONLY; archive files
# are NOT touched (PRECON-3 binding per v1-dq-schema-r1 plan §7).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# UTF-8 for Python (Windows defaults to cp1252).
export PYTHONIOENCODING=utf-8
export PYTHONUTF8=1

DRY_RUN=0
DQ_FILE="${REPO_ROOT}/.claude/decision-queue.json"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --file)
      DQ_FILE="$2"
      shift 2
      ;;
    *)
      echo "usage: $0 [--dry-run] [--file <path>]" >&2
      exit 2
      ;;
  esac
done

python3 - "${DQ_FILE}" "${DRY_RUN}" <<'PY'
import io, json, sys

dq_file = sys.argv[1]
dry_run = sys.argv[2] == '1'

with io.open(dq_file, encoding='utf-8') as f:
    data = json.load(f)

if data.get('schema_version') == 3:
    print("already v3 — no-op")
    sys.exit(0)

changed = 0
for arr in ('pending', 'resolved'):
    for entry in data.get(arr, []):
        # PRECON-1: never rename or remove id on pre-v3 entries.
        # Add id_v1 as integer alias only for int ids (pre-v3 entries).
        if 'id_v1' not in entry and isinstance(entry.get('id'), int):
            entry['id_v1'] = entry['id']
            changed += 1
        # PRECON-2: add approved_by + approved_at as null.
        if 'approved_by' not in entry:
            entry['approved_by'] = None
            changed += 1
        if 'approved_at' not in entry:
            entry['approved_at'] = None
            changed += 1

data['schema_version'] = 3

if dry_run:
    print(f"dry-run: would add fields to {changed} field slots across pending+resolved entries")
    print(f"dry-run: schema_version 2 → 3")
    sys.exit(0)

with io.open(dq_file, 'w', encoding='utf-8') as f:
    json.dump(data, f, indent=2, ensure_ascii=False)
    f.write('\n')

print(f"migrated: {changed} field slots added; schema_version → 3")
PY
