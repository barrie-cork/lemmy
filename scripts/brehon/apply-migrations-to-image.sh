#!/bin/sh
# Pre-baked Postgres fixtures: apply Brehon migrations + replaceable
# schema during Docker image initdb phase. Companion to
# `Dockerfile.test-pg`.
#
# Runs as part of the postgres container's `/docker-entrypoint-initdb.d/`
# pipeline — fires once when the data directory is empty (i.e. on
# image build). Subsequent `docker run` of the resulting image starts
# Postgres with the schema already in place.
#
# Why this script vs running diesel from inside Postgres:
#   - Cargo-driven `lemmy_diesel_utils` is a Rust binary; we don't
#     want to ship the Rust toolchain into the test image.
#   - The forbid_diesel_cli trigger (in migration 2025-08-01-000017)
#     refuses inserts into __diesel_schema_migrations unless
#     pg_advisory_lock(0) is held. We replicate Diesel's ordering
#     here: lock, run migrations, unlock.
#
# Per `crates/server/tests/e2e.rs::governance_fixtures::apply_all_schema`
# (lines 81-94 of e2e.rs), the canonical apply sequence is:
#   1. SELECT pg_advisory_lock(0);
#   2. Run all migrations in `migrations/` in order
#   3. DROP SCHEMA IF EXISTS r CASCADE; CREATE SCHEMA r;
#   4. \i replaceable_schema/utils.sql
#   5. \i replaceable_schema/triggers.sql
#
# This script exits non-zero on any migration failure, which fails the
# Docker image build — exactly what we want.

set -eu

POSTGRES_USER="${POSTGRES_USER:-lemmy}"
POSTGRES_DB="${POSTGRES_DB:-lemmy}"
# cr-12: pg_advisory_lock(0) is session-scoped, so the lock plus every
# migration/INSERT/replaceable-schema apply MUST run inside ONE psql
# session — not separate ${PSQL} -c invocations, which open new
# sessions and silently drop the lock between calls. We assemble a
# single SQL script via stdin (here-doc) with ON_ERROR_STOP=1 so any
# migration failure aborts the whole apply (and thus fails the Docker
# image build). copilot-2 fix: removed `|| true` swallowing on the
# __diesel_schema_migrations INSERT; ON CONFLICT DO NOTHING is the
# explicit idempotency mechanism, and any other error must surface.
PSQL="psql -v ON_ERROR_STOP=1 -U ${POSTGRES_USER} -d ${POSTGRES_DB}"

echo "brehon-fixtures: assembling single-session SQL stream"
SQL_SCRIPT="$(mktemp /tmp/brehon-fixtures-XXXXXX.sql)"
trap 'rm -f "$SQL_SCRIPT"' EXIT

# 1. Acquire the lock — session-scoped, so it spans the entire stream.
echo "SELECT pg_advisory_lock(0);" > "$SQL_SCRIPT"

# 2. Apply every migration's up.sql in lexicographic order, immediately
#    followed by the __diesel_schema_migrations INSERT. The schema
#    table is created by the first Diesel migration, so the INSERT
#    after migration #1 finds the table present.
for migration_dir in $(ls -1 /tmp/migrations | sort); do
    if [ -f "/tmp/migrations/${migration_dir}/up.sql" ]; then
        echo "  scheduling ${migration_dir}/up.sql"
        echo "\\echo applying ${migration_dir}/up.sql" >> "$SQL_SCRIPT"
        echo "\\i /tmp/migrations/${migration_dir}/up.sql" >> "$SQL_SCRIPT"
        echo "INSERT INTO __diesel_schema_migrations (version) VALUES ('${migration_dir%%_*}') ON CONFLICT DO NOTHING;" >> "$SQL_SCRIPT"
    fi
done

# 3. Replaceable schema (rebuild from scratch; r.* is recreated on
#    every cargo run too, so resetting here matches runtime semantics).
echo "\\echo rebuilding replaceable schema" >> "$SQL_SCRIPT"
echo "DROP SCHEMA IF EXISTS r CASCADE; CREATE SCHEMA r;" >> "$SQL_SCRIPT"
echo "\\i /tmp/replaceable_schema/utils.sql" >> "$SQL_SCRIPT"
echo "\\i /tmp/replaceable_schema/triggers.sql" >> "$SQL_SCRIPT"

# 4. Release the lock at script end (psql disconnect would also drop
#    it, but make the unlock explicit for symmetry with the lock).
echo "SELECT pg_advisory_unlock(0);" >> "$SQL_SCRIPT"

echo "brehon-fixtures: executing single-session apply"
${PSQL} -f "$SQL_SCRIPT"

echo "brehon-fixtures: done"
