# Phase 1 rubric — Schema + Diesel foundation

Phase 1 is COMPLETE and shipped to `origin/governance-v0` as of 2026-04-15
(commit `94eba51a0`). This rubric applies to any PR that touches Phase 1
surfaces — migrations, `crates/db_schema/src/source/governance/**`, enums,
or the hash-chain trigger. New work in these paths is almost always a bugfix
or a rebase-conflict resolution.

Focus areas beyond the ADR rubric:

- **Migration ordering.** If a new migration is added, the enum-adding
  migration must come before the table-adding migration that references it.
  Flag migrations that would fail on a clean database because they reference
  an enum type that does not yet exist.

- **Hash-chain trigger integrity.** Any migration that edits the
  `add_governance_log` family must preserve (a) the `BEFORE INSERT` trigger
  computing `entry_hash = sha256(prev_hash || entry_kind || payload || created_at)`,
  and (b) the `GRANT` that denies `UPDATE` / `DELETE` on `governance_log` to
  the app role. Both layers per ADR-008. Flag migrations that remove either.

- **`actor_pseudonym` presence.** The `actor_pseudonym` table was created in
  Phase 1 even though the lookup logic is wired in Phase 5. Per ADR-015.
  Flag any attempt to drop it or make it conditional.

- **`CaseStatus::EmergencyRemove` presence.** The enum variant is required
  by ADR-013. Flag any diff that removes it, renames it, or makes its
  inclusion conditional.

- **Down migrations.** Every `up.sql` has a matching `down.sql` that fully
  reverses. Flag missing or partial down migrations.

- **Hash-chain round-trip test.** The `governance_log_hash_chain_holds` test
  under `tests/e2e.rs` is the acceptance gate for ADR-008. Flag any
  modification that weakens it (e.g. removing assertions, shortening the
  chain, switching to an in-memory Postgres mock).
