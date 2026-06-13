#!/usr/bin/env bash
# Brehon pilot-seed scenario: ensure N fresh, community-SCOPED, strictly
# jury-eligible accounts exist (juror<prefix>1..N).
#
# Why: the pilot's original juror1-5 are NOT strictly eligible (they seat via
# the small-pool relaxed fallback). Strict eligibility needs a reputation_snapshot
# row scoped to the case's community_id (the IS-NOT-DISTINCT-FROM scope trap,
# lesson §3). Panels also CONSUME their pool and the appeal selector EXCLUDES the
# original panel (lesson §4), so seed generously.
#
# Usage:   bash seed-jurors.sh [COUNT] [PREFIX]
#   COUNT  number of eligible accounts to ensure (default 5)
#   PREFIX account name prefix (default "juror"); makes <prefix>1..<prefix>N
# Output:  PERSON_IDS=<space-separated ids>
#          (idempotent — existing accounts are reused, re-flagged eligible)

set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"; . "$DIR/lib.sh"

COUNT="${1:-5}"
PREFIX="${2:-juror}"

ids=$(seed_eligible_jurors "$PREFIX" "$COUNT")
echo "PERSON_IDS=$ids"
echo "COMMUNITY_ID=$COMMUNITY_ID"
echo "SEEDED count=$COUNT prefix=$PREFIX (community-scoped jury_eligible=true)"
