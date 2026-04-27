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

# If the working tree contains a real new migration vs governance-v0, fail loud.
# Otherwise this is a no-op (the workflow's path filter shouldn't trigger us
# without a migrations/ change, but defence in depth).
if git diff --name-only origin/governance-v0...HEAD -- migrations/ | grep -q '\.sql$'; then
    echo "ERROR: migrate-roundtrip.sh is a stub. A real migration was detected" >&2
    echo "  in the diff vs governance-v0 — replace this stub with real round-trip" >&2
    echo "  logic before merging. See script header for the v1-JM-e checklist." >&2
    exit 1
fi

echo "migrate-roundtrip.sh: no migrations/ change detected vs governance-v0; stub exiting 0."
exit 0
