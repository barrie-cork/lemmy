#!/usr/bin/env bash
# Phase 5a task 51 — enforce [99 OQ-016]'s v0 silence on person.membership_state.
#
# Per the plan, the column ships in v0 but NO v0 handler reads it. v1 flips
# `config.onboarding.enforce_membership_state` to activate the read path. Any
# accidental v0 read defeats the deferred-enforcement story.
#
# Authorised sites (exclusion list):
#   - crates/db_schema_file/       (schema.rs + enums.rs own the column type)
#   - crates/db_schema/src/source/person.rs (struct + InsertForm)
#   - crates/db_schema/src/lib.rs  (Person1/Person2AliasAllColumnsTuple — schema-layer tuple aliases that MUST list the column to match person::all_columns arity)
#   - crates/api/api/src/governance/config.rs (parse_membership_state helper, consumed only by the authorised writer below)
#   - crates/api/api_crud/src/user/create.rs (register handler writes the value)
#   - crates/apub/objects/src/objects/person.rs (federated-person upsert writes `None` so the SQL DEFAULT takes effect)
#   - migrations/                  (up.sql + down.sql for the column)
#   - .claude/                     (plan, rules, memory, decision queue all discuss it)
#
# Fails with a non-zero exit if any other file in crates/ references the column.

set -euo pipefail

FORBIDDEN=$(grep -rn --include='*.rs' 'membership_state' crates/ \
  | grep -v '^crates/db_schema_file/' \
  | grep -v '^crates/db_schema/src/source/person.rs' \
  | grep -v '^crates/db_schema/src/lib.rs' \
  | grep -v '^crates/api/api/src/governance/config.rs' \
  | grep -v '^crates/api/api_crud/src/user/create.rs' \
  | grep -v '^crates/apub/objects/src/objects/person.rs' \
  || true)

if [ -n "$FORBIDDEN" ]; then
  echo "ERROR: unauthorised membership_state read(s) detected outside the approved sites."
  echo ""
  echo "Per [99 OQ-016], no v0 handler may read person.membership_state."
  echo "If this is a v1 activation, add the new site to this script's exclusion list."
  echo ""
  echo "$FORBIDDEN"
  exit 1
fi

echo "membership_state guard: pass"
