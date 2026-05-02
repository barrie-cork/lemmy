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
PSQL="psql -v ON_ERROR_STOP=1 -U ${POSTGRES_USER} -d ${POSTGRES_DB}"

echo "brehon-fixtures: acquiring advisory lock"
${PSQL} -c "SELECT pg_advisory_lock(0);"

echo "brehon-fixtures: applying migrations from /tmp/migrations"
# Apply migrations in lexicographic order. Diesel migration directory
# layout: each migration is a directory containing up.sql + down.sql.
# We only apply up.sql here.
for migration_dir in $(ls -1 /tmp/migrations | sort); do
    if [ -f "/tmp/migrations/${migration_dir}/up.sql" ]; then
        echo "  applying ${migration_dir}/up.sql"
        ${PSQL} -f "/tmp/migrations/${migration_dir}/up.sql"
        # Record in __diesel_schema_migrations so MigrationHarness.run_pending_migrations
        # is a no-op on this image. The schema_migrations table itself is
        # created by the first Diesel migration.
        ${PSQL} -c "INSERT INTO __diesel_schema_migrations (version) VALUES ('${migration_dir%%_*}') ON CONFLICT DO NOTHING;" || true
    fi
done

echo "brehon-fixtures: rebuilding replaceable schema"
${PSQL} -c "DROP SCHEMA IF EXISTS r CASCADE; CREATE SCHEMA r;"
${PSQL} -f /tmp/replaceable_schema/utils.sql
${PSQL} -f /tmp/replaceable_schema/triggers.sql

echo "brehon-fixtures: releasing advisory lock"
${PSQL} -c "SELECT pg_advisory_unlock(0);"

echo "brehon-fixtures: done"
