#!/usr/bin/env bash
# admin-config-write.sh — transactional wrapper for admin governance_config edits.
#
# Writes a new governance_config row AND an `admin_config_changed` governance_log
# entry in the same transaction. This preserves action-time attribution for the
# auditor: without it, admins editing governance_config via raw psql leave no
# attributable action-time entry, and downstream capability_changed emissions
# (Phase 5a task 53) have no single causal root to point at.
#
# Per decision-queue #13 (answered 2026-04-17): ship this as operator tooling
# alongside v0, NOT as a handler. A proper HTTP endpoint replaces it in v1 per
# OQ-018.
#
# Scoping: this is a sibling docs/scripts artefact, not the admin HTTP path.
# Use only from operator shells with DB access; never expose publicly.
#
# Usage:
#   ./scripts/brehon/admin-config-write.sh <scope> <key> <value_type> <value> <admin_pseudonym> [<reason>]
#
# Arguments:
#   scope             Config scope, one of: instance, community
#   key               Config key (e.g. jury.max_concurrent_assignments)
#   value_type        Value type, one of: int, bool, string
#   value             The value to set (integer, boolean lowercase, or quoted string)
#   admin_pseudonym   The acting admin's actor_pseudonym UUID (lookup via
#                     `SELECT pseudonym FROM actor_pseudonym WHERE person_id = <admin_id>`)
#   reason            Optional. Free-text note; recorded in payload.reason.
#
# Requires:
#   DATABASE_URL      Standard libpq URL (same format Lemmy server uses)
#   psql              postgres client in PATH
#
# Example:
#   DATABASE_URL=postgres://lemmy:pass@localhost/lemmy \
#     ./scripts/brehon/admin-config-write.sh instance jury.max_concurrent_assignments int 5 \
#       "00000000-0000-0000-0000-0000000000aa" "raising cap after founder batch seed"

set -euo pipefail

if [ "$#" -lt 5 ]; then
  echo "usage: $0 <scope> <key> <value_type> <value> <admin_pseudonym> [<reason>]" >&2
  exit 2
fi

SCOPE="$1"
KEY="$2"
VTYPE="$3"
VALUE="$4"
ADMIN_PSEUDONYM="$5"
REASON="${6:-manual admin config edit}"

if [ -z "${DATABASE_URL:-}" ]; then
  echo "error: DATABASE_URL environment variable is required" >&2
  exit 2
fi

case "$VTYPE" in
  int)
    VALUE_CLAUSE="value_int = $VALUE"
    VALUE_PAYLOAD="$VALUE"
    ;;
  bool)
    VALUE_CLAUSE="value_bool = $VALUE"
    VALUE_PAYLOAD="$VALUE"
    ;;
  string)
    VALUE_CLAUSE="value_string = \$\$${VALUE}\$\$"
    VALUE_PAYLOAD="\"$VALUE\""
    ;;
  *)
    echo "error: value_type must be one of: int, bool, string (got: $VTYPE)" >&2
    exit 2
    ;;
esac

# scrub single-quote characters out of reason before embedding in the literal
# (payload value is JSON-encoded by jsonb_build_object below; scope/key pass
# through scrub_json via governance_log's own trigger chain on insert).
REASON_ESCAPED=$(printf '%s' "$REASON" | sed "s/'/''/g")

psql "$DATABASE_URL" <<SQL
BEGIN;

INSERT INTO governance_config (scope, key, value_type, ${VALUE_CLAUSE%% = *}, valid_from)
VALUES ('${SCOPE}', '${KEY}', '${VTYPE}', ${VALUE_CLAUSE#* = }, now());

INSERT INTO governance_log (entry_kind, payload, actor_pseudonym)
VALUES (
  'admin_config_changed',
  jsonb_build_object(
    'scope', '${SCOPE}',
    'key', '${KEY}',
    'value_type', '${VTYPE}',
    'value', ${VALUE_PAYLOAD},
    'reason', '${REASON_ESCAPED}'
  ),
  '${ADMIN_PSEUDONYM}'
);

COMMIT;
SQL
