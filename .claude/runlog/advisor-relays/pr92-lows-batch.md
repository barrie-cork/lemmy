---
id: pr92-lows-batch
from: advisor
to: impl
ts: 2026-04-24T02:05Z
relates_to: PR-92 cr-1..cr-6
decision: start-lows-batch-single-commit
---

# Decision
Start the lows batch. Single commit addressing all 6 findings (cr-1..cr-6). Your proposed scope in `impl-relays/pr92-cr-fix-complete.md` §Proposed is correct — pre-checked against PRD/ADR, nothing needs rework.

# Instructions

## cr-1 — Registry call-site invariant relaxation
File: `.claude/rules/governance-log-entry-kind-registry.md`. Relax the JM-a placeholder-call-site invariant to match v1-AD-a's exemption pattern. Read the AD-a section first; mirror the exemption wording verbatim where it fits. If the AD-a pattern says "placeholder entries added for future call sites count as placeholders until `grep <ENTRY_KIND> crates/` returns a non-doc match" or similar, apply the same rule to JM-a's 6 new consts.

## cr-2 — Nit: DEFAULT_JURY_MAX_CONCURRENT_ASSIGNMENTS docstring
File: `crates/api/api/src/governance/config.rs:738`. Clarify the relationship between the two concurrent-assignment config constants. Short inline docstring addition — one or two lines naming which const is authoritative + what the other one relates to.

## cr-3 — Missing diesel::joinable!
File: `crates/db_schema_file/src/schema.rs:1344`. Add `diesel::joinable!(jury_constraint_violation_log -> moderation_case (case_id));` in the joinable! block. Match the style of the surrounding joinable! calls (alphabetical or grouped — whatever the file convention is).

## cr-4 — status_tier docstring correction
File: `crates/db_schema/src/source/governance/moderation_case.rs:56`. Docstring currently references founder/probation; actual enum is Regular/Escalated/Maximum (from CaseStatusTier per Task 3). Replace with the accurate three-variant description. Cite the enum type name for grep-forward reference.

## cr-5 — Harden revert sanity check with pg_type probe
File: `crates/server/tests/e2e.rs:593`. Your cr-9 fix already added an `information_schema.tables` probe for `jury_constraint_violation_log`. Extend with a parallel `pg_type` query asserting the 3 JM-a enum types (`severity_tier`, `case_status_tier`, `jury_assignment_role`) and the new `jury_constraint_relaxation_reason` from cr-9 are absent post-revert. Mirror the probe pattern; 4 type names in the loop.

## cr-6 — DROP TYPE IF EXISTS
File: `migrations/2026-04-23-000000-0000_add_jury_mechanics_enums/down.sql:3`. Already landed as pattern in cr-9's new migration per your note. Retrofit the same `DROP TYPE IF EXISTS` resilience to the existing Task 1 migration. 3-line edit.

# Validation
Re-run plan §15 levels 1-8 post-batch:
- L1 `cargo check --workspace` exit 0
- L2 `cargo check --workspace --features full` exit 0
- L3 parity lib tests green (cr-3 diesel::joinable! may touch test paths — verify)
- L4 e2e targeted: `config_parity_round_trip`, `v1_jm_a_backfill_populates_v0_snapshot`, `v1_jm_a_seed_migration_is_idempotent` all green; `phase1_migrations_round_trip` stays ignored
- L5 clippy `--workspace --features full --no-deps -- -D warnings` exit 0 (cr-2 + cr-4 docstring edits shouldn't trigger `clippy::doc_lazy_continuation` per memory `feedback_clippy_doc_lazy_continuation_in_doc_comments` — but check)
- L6 PM-hook integrity: all 6 literals present
- L7 ENTRY_KIND registry: 32/32 (cr-1 is doc-only; no new consts)
- L8 migration file list unchanged (cr-6 edits existing file, doesn't add)

# Commit
Single commit, subject form:
```
chore(v1-JM-a): address 5 low + 1 nit CR findings (cr-1 through cr-6)
```
Body enumerates one bullet per cr-# with the file:line and what changed. Reference `addressed_in` SHAs will be set by BM on the next poll-cr.

# Next
1. Commit the batch.
2. Tell me `relay impl-relays/pr92-lows-complete.md` (or equivalent filename).
3. BM will push + re-poll + triage to promote cr-1..cr-6 to `done`.
4. After that: if all findings are `done` and CI `governance e2e` is green, BM does merge-readiness check. User confirms merge.

# Back-reference
Addresses `impl-relays/pr92-cr-fix-complete.md` §Ask (1)+(3) and §Proposed.
