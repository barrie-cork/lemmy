#!/usr/bin/env bash
# Phase 5a task 51 sibling — enforce [99 OQ-014]'s v0 silence on
# reputation_snapshot.can_sponsor.
#
# Task 50 added the column; task 53 (reputation_snapshot calculator) writes
# it on every recompute. But NO v0 handler reads it — `create_endorsement`
# (task 55) uses the age-only gate per OQ-014. v1 flips
# `config.sponsorship.require_reputation_gate` to activate the reputation
# gate; at that point `create_endorsement` becomes the first authorised
# reader and this script's exclusion list expands.
#
# Authorised sites (exclusion list):
#   - crates/db_schema_file/                             (column type)
#   - crates/db_schema/src/source/governance/reputation_snapshot.rs  (struct + InsertForm)
#   - crates/api/api/src/governance/reputation_snapshot.rs            (task 53 writer)
#   - crates/db_views/reputation/src/                                 (doc-comments in view crate)
#   - crates/api/api_crud/src/governance/create_endorsement.rs        (task 55 — doc-comment asserts `can_sponsor` is NOT read per OQ-014 / GOTCHA-55b)
#   - crates/api/api/src/governance/admin_reputation_stats.rs        (task 62 — COUNT(*) FILTER observability, NOT gate read)
#   - crates/api/api_common/src/governance.rs                        (task 62 — `can_sponsor_count: i64` DTO field)
#   - crates/server/tests/e2e.rs                                     (task 63d — INSERT test fixture for staleness-alert path)
#   - migrations/                                                     (up.sql)
#
# Fails with a non-zero exit if any other file in crates/ references
# `can_sponsor`.

set -euo pipefail

FORBIDDEN=$(grep -rn --include='*.rs' 'can_sponsor' crates/ \
  | grep -v '^crates/db_schema_file/' \
  | grep -v '^crates/db_schema/src/source/governance/reputation_snapshot.rs' \
  | grep -v '^crates/api/api/src/governance/reputation_snapshot.rs' \
  | grep -v '^crates/db_views/reputation/src/' \
  | grep -v '^crates/api/api_crud/src/governance/create_endorsement.rs' \
  | grep -v '^crates/api/api/src/governance/admin_reputation_stats.rs' \
  | grep -v '^crates/api/api_common/src/governance.rs' \
  | grep -v '^crates/server/tests/e2e.rs' \
  || true)

if [ -n "$FORBIDDEN" ]; then
  echo "ERROR: unauthorised can_sponsor read(s) detected outside the approved sites."
  echo ""
  echo "Per [99 OQ-014], no v0 handler may read reputation_snapshot.can_sponsor."
  echo "create_endorsement (task 55) uses the age-only gate. If this is a v1"
  echo "activation, add the new site to this script's exclusion list."
  echo ""
  echo "$FORBIDDEN"
  exit 1
fi

echo "can_sponsor guard: pass"
