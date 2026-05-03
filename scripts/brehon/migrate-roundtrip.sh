#!/usr/bin/env bash
# migrate-roundtrip.sh — placeholder for migration round-trip validation
#
# Invoked by .github/workflows/cargo-validate-migration.yml on pushes
# to phase-v1-* / junior/* branches that touch migrations/**.
#
# v1-validate-agent (this commit) ships the workflow YAML + this stub.
# The first sub-phase that authors a real migration after Shape G ships
# (likely v1-JM-e or v1-JM-d Task 2 once it's queued via DQ #61) MUST
# replace this body with real round-trip logic:
#
#   1. Diff migrations/ vs governance-v0 to identify the new migration id.
#   2. Spin up a throwaway Postgres (testcontainers-style or apt-installed).
#   3. Apply the new migration via cargo run -p lemmy_diesel_utils
#      --features full -- run.
#   4. Run a representative SELECT against the changed schema to confirm
#      the migration applied cleanly.
#   5. Apply the down migration via cargo run -p lemmy_diesel_utils
#      --features full -- redo (or equivalent).
#   6. Re-apply forward; confirm idempotent.
#
# Until that lands, this stub exits 0 on push events that don't actually
# touch migrations/ (the workflow's `paths:` filter prevents most invocations);
# if it IS triggered by a real migrations/ change, it exits non-zero so the
# author of that migration is forced to replace the stub.
#
# See .claude/PRPs/plans/v1-validate-agent.plan.md §13 Task 1 GOTCHA + §19
# (Notes) for the planning-side gap that produced this stub.

set -euo pipefail

# Verify the diff base ref is fetched. CI uses fetch-depth: 0 so this
# should always pass, but fail loud if it doesn't — a missing ref would
# silently exit 0 and defeat the guard (the original cr-1 finding on
# PR #104).
if ! git rev-parse --verify origin/governance-v0 >/dev/null 2>&1; then
    echo "ERROR: migrate-roundtrip.sh requires origin/governance-v0 to be" >&2
    echo "  fetched. The CI checkout step must use fetch-depth: 0 (or" >&2
    echo "  explicitly fetch governance-v0). Aborting." >&2
    exit 2
fi

# Identify any new migration directories vs governance-v0.
NEW_MIGRATIONS=$(git diff --name-only --diff-filter=A origin/governance-v0...HEAD -- 'migrations/*/up.sql' | xargs -I{} dirname {} 2>/dev/null | sort -u)
if [ -z "$NEW_MIGRATIONS" ]; then
    echo "migrate-roundtrip.sh: no new migrations vs governance-v0; exit 0."
    exit 0
fi

echo "migrate-roundtrip.sh: detected new migrations:"
echo "$NEW_MIGRATIONS"

# Spin up an ephemeral Postgres container for the forward pass.
# Note: lemmy_diesel_utils binary reads DATABASE_URL from LEMMY_DATABASE_URL env var
# and accepts no CLI sub-commands. Using env-var invocation + clean-container replay
# per plan §13 GOTCHA (binary surface: no run/revert sub-commands).
PG_PORT=$(shuf -i 30000-39999 -n 1)
PG_CONTAINER=$(docker run -d --rm \
    -e POSTGRES_PASSWORD=ci-roundtrip-throwaway \
    -e POSTGRES_DB=lemmy_roundtrip \
    -p ${PG_PORT}:5432 \
    pgautoupgrade/pgautoupgrade:18-alpine)
trap 'docker stop $PG_CONTAINER >/dev/null 2>&1 || true' EXIT

# Wait for Postgres ready.
for i in $(seq 1 30); do
    if docker exec "$PG_CONTAINER" pg_isready -U postgres >/dev/null 2>&1; then break; fi
    sleep 1
done

# 1. Apply ALL migrations forward (including the new ones).
LEMMY_DATABASE_URL="postgres://postgres:ci-roundtrip-throwaway@localhost:${PG_PORT}/lemmy_roundtrip" \
    cargo run -p lemmy_diesel_utils --features full
echo "forward exit: $?"

# 2. Spin up a fresh container (replaces revert+re-run; binary has no revert sub-command).
PG_PORT2=$(shuf -i 40000-49999 -n 1)
PG_CONTAINER2=$(docker run -d --rm \
    -e POSTGRES_PASSWORD=ci-roundtrip-throwaway \
    -e POSTGRES_DB=lemmy_roundtrip \
    -p ${PG_PORT2}:5432 \
    pgautoupgrade/pgautoupgrade:18-alpine)
trap 'docker stop $PG_CONTAINER2 >/dev/null 2>&1 || true; docker stop $PG_CONTAINER >/dev/null 2>&1 || true' EXIT

for i in $(seq 1 30); do
    if docker exec "$PG_CONTAINER2" pg_isready -U postgres >/dev/null 2>&1; then break; fi
    sleep 1
done

# 3. Re-apply forward on fresh container (idempotency check: same migrations, clean DB).
LEMMY_DATABASE_URL="postgres://postgres:ci-roundtrip-throwaway@localhost:${PG_PORT2}/lemmy_roundtrip" \
    cargo run -p lemmy_diesel_utils --features full
echo "re-forward exit: $?"

echo "migrate-roundtrip.sh: round-trip complete for $(echo "$NEW_MIGRATIONS" | wc -l) new migration(s)."
exit 0
