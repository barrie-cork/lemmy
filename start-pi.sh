#!/bin/bash
# start-pi.sh — launch pi-coding in this repo.
# Forces cwd to repo root and adds project scripts to PATH if present.

cd "$(dirname "$0")"

SCRIPTS_DIR="$(pwd)/.claude/scripts"
if [ -d "$SCRIPTS_DIR" ]; then
  export PATH="$SCRIPTS_DIR:$PATH"
fi

exec pi "$@"
