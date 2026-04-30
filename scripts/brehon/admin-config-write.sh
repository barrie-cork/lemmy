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
#   value_type        Value type, one of: int, bool, text, float
#   value             The value to set (integer, float, boolean lowercase, or quoted string)
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

# ---------------------------------------------------------------------------
# Validate value per type and build the SQL column assignment.
# VALUE_COL  — the governance_config column name to set (e.g. value_int)
# VALUE_SQL  — the SQL expression for the INSERT VALUES clause.
#              For numeric/bool types this is the validated literal; for text
#              it uses a psql variable reference :'value' so psql handles
#              quoting and escaping.
# VALUE_PAYLOAD_SQL — the jsonb_build_object value expression for governance_log.
# ---------------------------------------------------------------------------
case "$VTYPE" in
  int)
    if ! [[ "$VALUE" =~ ^-?[0-9]+$ ]]; then
      echo "error: value_type 'int' requires an integer value (got: $VALUE)" >&2
      exit 2
    fi
    VALUE_COL="value_int"
    VALUE_SQL="$VALUE"
    VALUE_PAYLOAD_SQL="$VALUE"
    ;;
  float)
    if ! [[ "$VALUE" =~ ^-?[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?$ ]]; then
      echo "error: value_type 'float' requires a numeric value (got: $VALUE)" >&2
      exit 2
    fi
    VALUE_COL="value_float"
    VALUE_SQL="$VALUE"
    VALUE_PAYLOAD_SQL="$VALUE"
    ;;
  bool)
    if ! [[ "$VALUE" =~ ^(true|false)$ ]]; then
      echo "error: value_type 'bool' requires 'true' or 'false' (got: $VALUE)" >&2
      exit 2
    fi
    VALUE_COL="value_bool"
    VALUE_SQL="$VALUE"
    VALUE_PAYLOAD_SQL="$VALUE"
    ;;
  text)
    VALUE_COL="value_text"
    VALUE_SQL=":'value'"
    VALUE_PAYLOAD_SQL=":'value'"
    ;;
  *)
    echo "error: value_type must be one of: int, bool, text, float (got: $VTYPE)" >&2
    exit 2
    ;;
esac

# ---------------------------------------------------------------------------
# Look up the person_id for the acting admin.
# Uses psql -t (tuples-only) -c to return a single integer, trimming whitespace.
# Fails fast if the pseudonym is not found so we never write a NULL updated_by.
# ---------------------------------------------------------------------------
UPDATED_BY=$(psql -t "$DATABASE_URL" \
  -v admin_pseudonym="$ADMIN_PSEUDONYM" \
  -c "SELECT person_id FROM actor_pseudonym WHERE pseudonym = :'admin_pseudonym'" \
  | tr -d '[:space:]')

if [ -z "$UPDATED_BY" ]; then
  echo "error: admin_pseudonym '$ADMIN_PSEUDONYM' not found in actor_pseudonym table" >&2
  exit 1
fi

if ! [[ "$UPDATED_BY" =~ ^[0-9]+$ ]]; then
  echo "error: unexpected person_id value '$UPDATED_BY' returned from actor_pseudonym lookup" >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Execute the transactional INSERT pair.
# All user-supplied string values (scope, key, admin_pseudonym, reason, value
# for text type) are passed via psql -v variables and referenced as :'varname'
# in SQL — psql applies single-quote escaping so no injection is possible.
# Numeric/bool values were validated by regex above and are safe to interpolate
# as literals.
#
# Closes #84 (option A): the payload now includes `previous_value` and
# `previous_from` at the tail, mirroring the Rust handler shape from commit
# d623bcff5 (`crates/api/api/src/governance/admin_config.rs`
# `build_admin_config_changed_payload`). The pre-INSERT SELECT below reads
# the row that THIS write will supersede — same scope, same key, latest
# valid_from <= now(). When no prior row exists, both fields are JSON null,
# which matches the handler's behaviour for a fresh key.
# Field order is load-bearing per the handler:
#   scope, key, value_type, value, reason, previous_value, previous_from
# project_to_audit_entry hydrates previous_value/previous_from from the
# tail; missing-tail rows (pre-deprecation shell writes) degrade to None.
# ---------------------------------------------------------------------------
psql "$DATABASE_URL" \
  -v scope="$SCOPE" \
  -v key="$KEY" \
  -v vtype="$VTYPE" \
  -v value="$VALUE" \
  -v admin_pseudonym="$ADMIN_PSEUDONYM" \
  -v reason="$REASON" \
  <<SQL
BEGIN;

-- Capture the pre-INSERT row (if any) into a transactional CTE-driven
-- temp value. We can't use a CTE across separate statements, so use a
-- DO block with PERFORM-style read into temp records via psql's gset
-- isn't viable inside a single SQL stream — instead, project the previous
-- row at INSERT time using a sub-SELECT in jsonb_build_object below.

INSERT INTO governance_config (scope, key, value_type, ${VALUE_COL}, valid_from, updated_by)
VALUES (:'scope', :'key', :'vtype', ${VALUE_SQL}, now(), ${UPDATED_BY});

INSERT INTO governance_log (entry_kind, payload, actor_pseudonym)
VALUES (
  'admin_config_changed',
  jsonb_build_object(
    'scope',          :'scope',
    'key',            :'key',
    'value_type',     :'vtype',
    'value',          ${VALUE_PAYLOAD_SQL},
    'reason',         :'reason',
    -- previous_value: the JSON value from the most recent governance_config
    -- row for (scope, key) STRICTLY BEFORE the INSERT above. The INSERT in
    -- this same tx has valid_from = now(); a sub-SELECT with valid_from <
    -- now() excludes the row we just wrote and finds the prior row (or none).
    -- Returns JSON null when no prior row exists (fresh key).
    'previous_value', (
      SELECT CASE prev.value_type
               WHEN 'int'   THEN to_jsonb(prev.value_int)
               WHEN 'float' THEN to_jsonb(prev.value_float)
               WHEN 'bool'  THEN to_jsonb(prev.value_bool)
               WHEN 'text'  THEN to_jsonb(prev.value_text)
             END
      FROM governance_config prev
      WHERE prev.scope = :'scope'
        AND prev.key   = :'key'
        AND prev.valid_from < now()
      ORDER BY prev.valid_from DESC
      LIMIT 1
    ),
    -- previous_from: ISO-8601 timestamp of the prior row's valid_from
    -- (the handler's `previous.effective_from`). JSON null when no prior row.
    'previous_from', (
      SELECT to_char(prev.valid_from AT TIME ZONE 'UTC',
                     'YYYY-MM-DD"T"HH24:MI:SS.US"Z"')
      FROM governance_config prev
      WHERE prev.scope = :'scope'
        AND prev.key   = :'key'
        AND prev.valid_from < now()
      ORDER BY prev.valid_from DESC
      LIMIT 1
    )
  ),
  :'admin_pseudonym'
);

COMMIT;
SQL
