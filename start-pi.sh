#!/bin/bash
# start-pi.sh — launch pi-coding in this repo. ./start-pi.sh
# Forces cwd to repo root and adds project scripts to PATH if present.

cd "$(dirname "$0")"

SCRIPTS_DIR="$(pwd)/.claude/scripts"
if [ -d "$SCRIPTS_DIR" ]; then
  export PATH="$SCRIPTS_DIR:$PATH"
fi

# Wire local Brehon PMD if present so .pi/hook-scripts/retro-check.sh can
# consult it. Without this export retro-check falls back through:
#   $PROJECT_MEMORY_DB → main repo's .project-memory/memory.db → local
#   .project-memory/memory.db
# and on this Mac all three are missing or symlinked to an unreachable
# target on the EliteDesk, so retro-check fails open. See PI_AUDIT_REPORT.md
# Finding 2 (medium / PORTABILITY).
PMD_DB="$(pwd)/.claude/memory/memory.db"
if [ -f "$PMD_DB" ]; then
  export PROJECT_MEMORY_DB="$PMD_DB"
fi

exec pi "$@"
