# Plan: v1-AD-a — Admin Dashboard migrations + ConfigKeyMetadata registry

## Table of contents

| § | Heading | Line |
|---|---|---|
| 1 | Summary | 30 |
| 2 | Source | 38 |
| 3 | Problem statement | 48 |
| 4 | Solution statement | 59 |
| 5 | Metadata | 77 |
| 6 | Sub-phase split (v1-AD-a → v1-AD-e) | 94 |
| 7 | Open questions (must land before dependent sub-phases) | 112 |
| 8 | Flow design | 128 |
| 9 | Mandatory reading | 189 |
| 10 | Patterns to mirror | 219 |
| 11 | Files to change | 382 |
| 12 | NOT building in v1-AD-a | 408 |
| 13 | Step-by-step tasks | 426 |
| 14 | Testing strategy | 1000 |
| 15 | Validation commands (DoD) | 1032 |
| 16 | Acceptance criteria | 1096 |
| 17 | Completion checklist | 1122 |
| 18 | Risks and mitigations | 1140 |
| 19 | Notes | 1157 |
| 20 | Sub-phase stubs (v1-AD-b/c/d/e TOC) | 1166 |

---

## 1. Summary

v1-AD-a is the **foundation sub-phase** of the v1 admin-dashboard keystone. It ships the four new v1 migrations (`add_rule_set_versions`, `add_sponsor_allowlist`, `add_case_applied_config_snapshot`, `seed_v1_config_keys`), extends the v0 `EXPECTED_SEED_COUNT = 34` parity contract with a parametric `EXPECTED_SEED_COUNT_V1_AD = 27` (27 admin-dashboard-owned keys — `rule_set.active_version_id` deliberately omitted per §4.1), adds the compile-time `CONFIG_KEY_METADATA` registry (per NOT4 2026-04-19: `&'static [ConfigKeyMetadata]`, not a DB table), adds two new `ENTRY_KIND_ADMIN_CONFIG_*` consts, and initialises `.claude/rules/governance-log-entry-kind-registry.md` from the v0 baseline (closes GH issue #41 as a side-effect). No HTTP routes land in v1-AD-a — endpoints ship in v1-AD-b.

This sub-phase's value alone: the database is now capable of answering "what's the current value of each configurable knob?" and "what rule-set version was this case decided under?" — before any new write path exists. v1-AD-b then builds the write path on top.

---

## 2. Source

- [../prds/v1-admin-dashboard.prd.md](../prds/v1-admin-dashboard.prd.md) §3 (schema), §5 (defaults), §8 (migrations). §4/§6/§7 are v1-AD-b/c/d scope.
- [../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) ADR-008 (signed log), ADR-010 (staged releases), ADR-012 (Extism + Lemmy 1.0-beta), ADR-015 (pseudonyms), OQ-002 (rule-set versioning), OQ-018 (admin config write — **design authored by this plan's parent PRD**), OQ-026 (status-aware rule cascade).
- [../../../docs/brehon-law-inspired-network/04-data-model-and-api.md](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) §3 (governance_config + governance_log models).
- Parent PRD Resolutions applied §11 (2026-04-19) — B1/B2/B3/B4/B5, N1, NOT1, NOT4, NOT5 are locked; do NOT re-litigate.
- [GH issue #41](https://github.com/barrie-cork/lemmy/issues/41) — entry-kind registry tracker, closed as a side-effect of this sub-phase per handover directive 2026-04-19.

---

## 3. Problem statement

Four things must exist on disk before a v1 operator can edit any config via HTTP:

1. A canonical compile-time metadata registry (`ConfigKeyMetadata` array) so the write endpoint can validate type, range, scope, and `requires_re_jury` on every key.
2. An `applied_config_snapshot` JSONB column on `moderation_case` so in-flight juries can be grandfathered against panel-size / quorum / threshold edits (ADR-010 invariant: no retroactive invalidation).
3. A versioned rule-set table (`rule_set_version`) so `submit_jury_vote` can pin a case to the rule text as it was when the case was decided (OQ-002).
4. The v1 config rows seeded into `governance_config` with parity against new Rust `DEFAULT_*` consts (Watch-1 parity contract — already enforced for v0 at `config.rs:469`). Count is **parametric** (`EXPECTED_SEED_COUNT_V1_AD`, currently 27) and reconciled at task 7's pre-commit gate against PRD §5.2 as source of truth.

v0 ships none of these. The v0 shell script `scripts/brehon/admin-config-write.sh:148` already writes `admin_config_changed` entries, but there is no Rust const for that string — v1-AD-b needs one before it can call `governance_log::append`.

## 4. Solution statement

Land four idempotent migrations under the existing `YYYY-MM-DD-HHMMSS-NNNN_name` convention (next free slot `2026-04-21-000000-0000_*`), extend `crates/api/api/src/governance/config.rs` with a `CONFIG_KEY_METADATA: &'static [ConfigKeyMetadata]` array **additive to** `SEEDED_KEYS_WITH_CONSTS` (not a replacement), and add two new `ENTRY_KIND_*` consts to `governance_log.rs`. Extend the Diesel schema via hand-edit of `crates/db_schema_file/src/schema.rs` (the fork's convention: `@generated automatically by Diesel CLI` header + hand-added view/table blocks for governance; see schema.rs:426-441 for precedent). Add the `ConfigKeyMetadata` struct + `ValueType`/`ConfigScope`/`ApplyAt` enums in `config.rs` alongside the existing `Scope` enum.

No HTTP handlers. No routes. No askama templates. No SSE.

Per-sub-phase merge discipline: v1-AD-a ships as its own PR `phase-v1-AD-a` → `governance-v0`, earns its own CodeRabbit review, then v1-AD-b plans on top of the merged HEAD.

### 4.1 Architecturally load-bearing decisions locked in this sub-phase

- **CONFIG_KEY_METADATA is compile-time, not a DB table.** (NOT4 2026-04-19, PRD §3.2). A new key requires a PR-against-Rust-code edit. The parity test `every_seeded_key_has_metadata` rejects DB-only additions.
- **EXPECTED_SEED_COUNT stays at 34 (v0 invariant).** A new `EXPECTED_SEED_COUNT_V1_AD: usize = 27` is added *beside* it; the parity test becomes `SEEDED_KEYS_WITH_CONSTS.len() == EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD` so each subsequent v1 sub-PRD adds its own parametric count (`EXPECTED_SEED_COUNT_V1_JM`, etc.) without churning this one. See directive #4 from advisor 2026-04-19. (27, not 28, because `rule_set.active_version_id` is not seeded — its absence is the "no active version" signal; see next bullet.)
- **`rule_set.active_version_id` is deliberately UN-seeded in v1-AD-a.** Per advisor edit #2 (2026-04-19): storing `-1` as a `None` sentinel in a Postgres INTEGER column is a footgun that leaks to every `config::get_int` read. Instead, the absence of a `governance_config` row for this key IS the signal for "no active version" — the v0 reader already returns the Rust const default when no row matches, and `config.rs` will NOT declare a `DEFAULT_RULE_SET_ACTIVE_VERSION_ID` const. v1-AD-c seeds the key via INSERT only when admin actually activates a rule set via `POST /api/v4/governance/admin/rule-sets`. v1-AD-b readers needing this key call a new `config::get_int_opt` accessor (added in v1-AD-b, not here) that returns `None` when both the DB row and the const default are absent. This is the append-only convention: you write the row when the fact exists, not before.
- **Rule-set text is NOT federated in v0/v1-AD.** No `crates/apub/` changes. Federation of rule-set versions is v2+ scope.
- **`moderation_case.applied_config_snapshot` is nullable.** Existing (pre-v1-AD) cases keep NULL; new cases populate. No backfill required.

---

## 5. Metadata

| Field | Value |
|---|---|
| Type | `SCHEMA + CROSS_CUTTING` |
| Complexity | MEDIUM |
| Crates affected | `db_schema_file` (schema.rs hand-edit), `db_schema` (model/InsertForm), `api` (config.rs + governance_log.rs), `migrations/` (4 new) |
| v0 step | v1 post-MVP per [ADR-010](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) — keystone sub-phase |
| Dependencies | governance-v0 HEAD `3bbf419da`; no Phase 6 dependency (verified in PRD §8.2 migration list) |
| Estimated tasks | **7** |
| Sub-phase target branch | `phase-v1-AD-a` branched from `governance-v0` |
| PR target | `governance-v0` (per `.claude/rules/phase-branch.md`) |
| Closes | GH issue #41 (entry-kind registry) as a side-effect |
| Opens | 3 new OQs (see §7) before v1-AD-b plan writes |

---

## 6. Sub-phase split (v1-AD-a → v1-AD-e)

The parent PRD's 8 HTTP routes + dashboard aggregate + SSE + askama pages exceeds the governance-v0 phase-splitting rule (≤10-12 tasks per ralph loop). Split matches Phase 5's three-sub-phase cadence:

| Sub-phase | Scope | Task count | Blocks |
|---|---|---|---|
| **v1-AD-a** (this plan) | 4 migrations + `ConfigKeyMetadata` registry + 2 new ENTRY_KIND consts + entry-kind registry doc | 7 | v1-AD-b, c, d |
| v1-AD-b (future) | `POST/GET /admin/config` + `GET /admin/config/audit` + capability checks + dry-run impact computation + per-key type validation | 8-9 | v1-AD-e |
| v1-AD-c (future) | `rule_set_version` routes + `moderation_case.rule_set_version_id` wire-up in `submit_jury_vote` | 4-5 | — |
| v1-AD-d (future) | `GET /admin/dashboard` aggregate + `GET /admin/audit/stream` SSE | 3-4 | depends on OQ-V1-AD-02 |
| v1-AD-e (future, optional) | Askama HTML pages (§6 of parent PRD) | deferred | depends on OQ-V1-AD-01 |

Each sub-phase = its own PR → `governance-v0` → CodeRabbit review → retro. Matches Phase 5's cadence.

v1-AD-b through v1-AD-e are **stubbed** in §20 (TOC only) — they are not planned in detail here. Planning each requires its own `/prp-plan` run against the then-current `governance-v0` HEAD (v1-AD-a merged).

---

## 7. Open questions (must land before dependent sub-phases)

Per advisor directive 2026-04-19 (#3), three OQs must be opened in `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` **before** v1-AD-b/c/d plans are written. They block architecture choices those sub-phases depend on. v1-AD-a itself is unblocked — it only produces schema + metadata + consts.

| OQ | Question | Lean | Blocks |
|---|---|---|---|
| OQ-V1-AD-01 (new) | askama vs maud vs defer HTML pages — workspace dep policy, build-time impact, supply-chain surface | **Defer to v1.x; ship v1-AD-d API-only.** Neither crate exists in `Cargo.lock` today. | v1-AD-e |
| OQ-V1-AD-02 (new) | actix-web-lab for SSE vs hand-rolled chunked-response (~80 LOC) | Upstream Lemmy does NOT use `actix-web-lab` (grep against `Cargo.lock` 2026-04-19 returned no matches). Lean: hand-rolled via `async-stream` (already transitive dep). | v1-AD-d |
| OQ-V1-AD-03 (new) | Dry-run impact computation inside vs outside `run_transaction` | `crates/diesel_utils/src/connection.rs:68-79` shows `run_transaction` calls `diesel-async`'s `.transaction()` — no SAVEPOINT primitive exposed. Lean: **compute impact BEFORE tx opens** (read-only query against current config + the proposed value). Cleaner architecturally. Reshape PRD §4.3 to match. | v1-AD-b |

**Task 7 of this plan opens all three OQs.** The opening is the smallest possible change to 99-decisions — no resolution, no design. Resolution happens before v1-AD-b/c/d plans commit.

OQ-V1-AD-04 (`participation.attestation_enabled` UX shape, already listed in parent PRD §9) is not re-opened here — it's an existing PRD-tracked item.

---

## 8. Flow design

### Before state (governance-v0 HEAD 3bbf419da)

```text
╔═══════════════════════════════════════════════════════════════════════════════╗
║                              BEFORE STATE                                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║   governance_config table: 34 v0 rows seeded                                  ║
║   SEEDED_KEYS_WITH_CONSTS = [34 tuples]                                       ║
║   EXPECTED_SEED_COUNT = 34                                                    ║
║   moderation_case: 16 columns — no applied_config_snapshot                    ║
║   rule_set_version: does NOT exist                                            ║
║   sponsor_allowlist: does NOT exist                                           ║
║   ENTRY_KIND_* consts: 19 (governance_log.rs:50-68)                           ║
║   'admin_config_changed' entry_kind: used as a string literal by              ║
║     scripts/brehon/admin-config-write.sh:148 (no Rust const)                  ║
║   CONFIG_KEY_METADATA registry: does NOT exist                                ║
║                                                                               ║
║   PAIN: no way for HTTP handler to validate config writes at type-level;      ║
║   no rule-set versioning; no in-flight-jury grandfathering substrate.         ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### After state (end of v1-AD-a)

```text
╔═══════════════════════════════════════════════════════════════════════════════╗
║                               AFTER STATE                                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║   governance_config table: 61 rows (34 v0 + 27 v1-AD)                         ║
║   SEEDED_KEYS_WITH_CONSTS = [61 tuples]                                       ║
║   EXPECTED_SEED_COUNT = 34  (unchanged)                                       ║
║   EXPECTED_SEED_COUNT_V1_AD = 27  (new — 27 because rule_set.active_version_id║
║     is deliberately un-seeded; absence-of-row is the "no active version" sig) ║
║   moderation_case: 17 columns (+applied_config_snapshot JSONB nullable)       ║
║                    18 columns (+rule_set_version_id INT4 nullable FK)         ║
║   rule_set_version: new table, 7 columns, append-only                         ║
║   sponsor_allowlist: new table, 4 columns (reserved; read-path lands in v1-AD-b)║
║   ENTRY_KIND_* consts: 21 (19 v0 + ENTRY_KIND_ADMIN_CONFIG_CHANGED,           ║
║                            ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED)             ║
║   CONFIG_KEY_METADATA: &'static [ConfigKeyMetadata] len=61 compile-time       ║
║   parity::every_seeded_key_has_metadata test passes                           ║
║   .claude/rules/governance-log-entry-kind-registry.md initialised             ║
║                                                                               ║
║   VALUE: v1-AD-b can now build POST /admin/config without schema work.        ║
║   VALUE: ADR-010 invariant has a substrate (applied_config_snapshot column).  ║
║   VALUE: OQ-002 resolved at schema layer.                                     ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### Data flow — no HTTP surface changes in v1-AD-a

This sub-phase ships no new endpoints. All existing endpoints unchanged. `cargo check --workspace --features full` is the primary behavioural signal. One parity test + the existing `config_parity_round_trip` prove the new seed rows are readable via typed accessors.

---

## 9. Mandatory reading

The implementation agent MUST read these files before starting, in this order:

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `.claude/PRPs/prds/v1-admin-dashboard.prd.md` | §3 (schema), §5 (defaults), §8 (migrations), §11 (resolutions) | PRD is authoritative; §11 locks B1-B6/N1/NOT1-5 decisions |
| P0 | `crates/api/api/src/governance/config.rs` | 1-501 | v0 reader + parity tests. `CONFIG_KEY_METADATA` lands here. `EXPECTED_SEED_COUNT` line 463. `SEEDED_KEYS_WITH_CONSTS` lines 423-458. |
| P0 | `crates/api/api/src/governance/governance_log.rs` | 50-68 | 19 v0 ENTRY_KIND consts to NOT collide with. `append()` signature line 87. |
| P0 | `migrations/2026-04-18-000000-0000_add_governance_config/up.sql` | 1-113 | CREATE TABLE + view + idempotent INSERT shape. `ON CONFLICT (scope, key, valid_from) DO NOTHING` at line 113. |
| P0 | `crates/db_schema/src/source/governance/governance_config.rs` | 1-53 | `GovernanceConfig` Queryable + `GovernanceConfigInsertForm` Insertable |
| P0 | `crates/db_schema_file/src/schema.rs` | 412-455 | `@generated` header + hand-edited `governance_config`, `governance_config_current` view, `governance_log` table! blocks; precedent for adding new tables/views |
| P1 | `migrations/2026-04-15-100100-0000_add_governance_core/up.sql` | 1-18 | `moderation_case` CREATE TABLE — FK shape for `ON DELETE SET NULL` / `ON DELETE CASCADE` |
| P1 | `scripts/brehon/admin-config-write.sh` | 140-160 | Exact `admin_config_changed` payload shape — v1-AD-b must match this byte-for-byte per NOT5 gate 3 |
| P1 | `.claude/rules/cargo-output-capture.md` | all | Mandatory when running any validation |
| P1 | `.claude/rules/no-cargo-output-paste.md` | all | Companion to above |
| P1 | `.claude/rules/view-crate-selectable-template.md` | all | Not directly applicable (no view crate added) but load-bearing reasoning for future sub-phases |
| P1 | `.claude/rules/phase-branch.md` | all | Branch flow: `phase-v1-AD-a` → PR → `governance-v0` |
| P2 | `crates/server/tests/e2e.rs` | 1-60, 1293-1366 | e2e harness entry, `config_parity_round_trip` test |

### External documentation

| Source | Version | Section | Why |
|---|---|---|---|
| [diesel 2.x docs](https://docs.rs/diesel/2.2) | 2.2 (check workspace `Cargo.toml`) | Jsonb, `#[derive(Insertable)]`, `ON CONFLICT` | JSONB column precedent in governance_log; `ON CONFLICT DO NOTHING` shape |

No new external dependencies added in v1-AD-a (askama, actix-web-lab, etc. are v1-AD-d/e scope, gated on OQ-V1-AD-01/02).

---

## 10. Patterns to mirror

Every snippet below is **copied verbatim** from the current workspace — not invented. Implementer should copy-paste and edit field names only.

### 10.1 Migration shape (idempotent UP)

```sql
-- SOURCE: migrations/2026-04-18-000000-0000_add_governance_config/up.sql:19-36
-- COPY THIS PATTERN for rule_set_version and sponsor_allowlist CREATE TABLE:

CREATE TABLE governance_config (
    id          SERIAL PRIMARY KEY,
    scope       TEXT NOT NULL,
    key         TEXT NOT NULL,
    value_type  TEXT NOT NULL,
    value_int   BIGINT,
    value_float DOUBLE PRECISION,
    value_bool  BOOLEAN,
    value_text  TEXT,
    valid_from  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by  INTEGER REFERENCES person (id) ON DELETE RESTRICT
);
```

### 10.2 FK to community

```sql
-- SOURCE: migrations/2026-04-15-100100-0000_add_governance_core/up.sql:3
-- COPY THIS PATTERN for rule_set_version.community_id:

community_id INTEGER NOT NULL REFERENCES community (id) ON DELETE CASCADE,
```

### 10.3 Idempotent seed INSERT

```sql
-- SOURCE: migrations/2026-04-18-000000-0000_add_governance_config/up.sql:78-113
-- COPY THIS PATTERN for seed_v1_config_keys:

INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text) VALUES
    ('instance', 'jury.severity_thresholds.minor',       'text', NULL, NULL, NULL, 'majority'),
    ('instance', 'jury.severity_thresholds.moderate',    'text', NULL, NULL, NULL, '60%'),
    -- ... 26 more rows from §5.2 ...
ON CONFLICT (scope, key, valid_from) DO NOTHING;
```

### 10.4 ALTER TABLE ADD COLUMN (fast path — non-volatile default)

```sql
-- SOURCE: migrations/2026-04-18-000100-0000_add_person_membership_state/up.sql
-- COPY THIS PATTERN for moderation_case.rule_set_version_id + applied_config_snapshot:
-- Nullable columns need NO default and NO backfill.

ALTER TABLE moderation_case ADD COLUMN applied_config_snapshot JSONB;
ALTER TABLE moderation_case ADD COLUMN rule_set_version_id INTEGER REFERENCES rule_set_version(id);
```

### 10.5 Schema.rs hand-edit pattern

```rust
// SOURCE: crates/db_schema_file/src/schema.rs:426-441
// COPY THIS PATTERN for rule_set_version + sponsor_allowlist + column additions:
// Header says "@generated automatically by Diesel CLI" but new tables/views
// are hand-added under the generated block. Governance_config_current view
// is hand-edited precedent.

diesel::table! {
    governance_config_current (id) {
        id -> Int4,
        scope -> Text,
        // ... columns ...
    }
}
```

### 10.6 Diesel model + InsertForm

```rust
// SOURCE: crates/db_schema/src/source/governance/governance_config.rs:1-53
// COPY THIS PATTERN for RuleSetVersion + SponsorAllowlist:

#[skip_serializing_none]
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = governance_config))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct GovernanceConfig {
  pub id: GovernanceConfigId,
  pub scope: String,
  // ... fields ...
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = governance_config))]
pub struct GovernanceConfigInsertForm {
  pub scope: String,
  // ... fields (no id, no valid_from) ...
}
```

### 10.7 Parity test extension

```rust
// SOURCE: crates/api/api/src/governance/config.rs:469-500
// EXTEND THIS PATTERN — do not replace:

#[cfg(test)]
mod parity {
  use super::*;

  #[test]
  fn seeded_keys_count_matches_const_count() {
    // Existing v0 test — do not remove or weaken.
    // v1-AD extension: also assert the new split count.
    assert_eq!(
      SEEDED_KEYS_WITH_CONSTS.len(),
      EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD,
      "SEEDED_KEYS_WITH_CONSTS length ({}) must equal EXPECTED_SEED_COUNT ({}) + \
       EXPECTED_SEED_COUNT_V1_AD ({}) — add/remove keys in both places. \
       v1-AD-a does NOT include rule_set.active_version_id (absence-of-row \
       is the no-active-version signal per §4.1).",
      SEEDED_KEYS_WITH_CONSTS.len(),
      EXPECTED_SEED_COUNT,
      EXPECTED_SEED_COUNT_V1_AD,
    );
  }

  #[test]
  fn every_seeded_key_has_metadata() {
    // NEW v1-AD test — every SEEDED_KEYS_WITH_CONSTS entry must have a
    // matching CONFIG_KEY_METADATA entry (Watch-2 parity contract, NOT4).
    let metadata_keys: HashSet<&str> =
      CONFIG_KEY_METADATA.iter().map(|m| m.key).collect();
    for (key, _const_name, _vtype) in SEEDED_KEYS_WITH_CONSTS {
      assert!(
        metadata_keys.contains(key),
        "seeded key `{key}` missing from CONFIG_KEY_METADATA — add a \
         ConfigKeyMetadata entry in config.rs"
      );
    }
  }
}
```

### 10.8 ENTRY_KIND const addition

```rust
// SOURCE: crates/api/api/src/governance/governance_log.rs:50-68
// APPEND after ENTRY_KIND_APPEAL_REQUESTED:

// v1-AD-a additions (v1 admin dashboard sub-phase A):
pub const ENTRY_KIND_ADMIN_CONFIG_CHANGED: &str = "admin_config_changed";
pub const ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED: &str = "admin_config_change_denied";
// The "admin_config_changed" string matches the existing shell-script emission
// at scripts/brehon/admin-config-write.sh:148 — DO NOT rename. v1-AD-b's HTTP
// path must write byte-identical payloads per NOT5 deprecation gate 3.
```

---

## 11. Files to change

| File | Action | Justification |
|---|---|---|
| `migrations/2026-04-21-000000-0000_add_rule_set_versions/up.sql` | CREATE | Rule-set versioning schema per PRD §3.6 + OQ-002 |
| `migrations/2026-04-21-000000-0000_add_rule_set_versions/down.sql` | CREATE | Drop in reverse dependency order |
| `migrations/2026-04-21-000100-0000_add_sponsor_allowlist/up.sql` | CREATE | Reserve sponsor_allowlist table per PRD §3.1 + OQ-020 |
| `migrations/2026-04-21-000100-0000_add_sponsor_allowlist/down.sql` | CREATE | Clean rollback |
| `migrations/2026-04-21-000200-0000_add_case_applied_config_snapshot/up.sql` | CREATE | `applied_config_snapshot` + `rule_set_version_id` columns on moderation_case |
| `migrations/2026-04-21-000200-0000_add_case_applied_config_snapshot/down.sql` | CREATE | `ALTER TABLE moderation_case DROP COLUMN …` |
| `migrations/2026-04-21-000300-0000_seed_v1_config_keys/up.sql` | CREATE | 27 new v1-AD config rows; idempotent (28-in-PRD-§5.2 minus `rule_set.active_version_id`) |
| `migrations/2026-04-21-000300-0000_seed_v1_config_keys/down.sql` | CREATE | DELETE scoped to exact key list |
| `crates/db_schema_file/src/schema.rs` | UPDATE | Hand-add `rule_set_version`, `sponsor_allowlist` tables + new columns on `moderation_case` |
| `crates/db_schema/src/source/governance/rule_set_version.rs` | CREATE | Queryable + InsertForm |
| `crates/db_schema/src/source/governance/sponsor_allowlist.rs` | CREATE | Queryable + InsertForm (read-path unused in v1-AD-a; lands as API consumer in v1-AD-b) |
| `crates/db_schema/src/source/governance/mod.rs` | UPDATE | Export new modules |
| `crates/db_schema/src/newtypes.rs` | UPDATE | Add `RuleSetVersionId`, `SponsorAllowlistId` newtypes |
| `crates/api/api/src/governance/config.rs` | UPDATE | Add `ConfigKeyMetadata` struct + `ValueType`/`ConfigScope`/`ApplyAt` enums + 27 new `DEFAULT_*` consts + 27 new `const_default_*` match arms + extend `SEEDED_KEYS_WITH_CONSTS` to 61 tuples + `CONFIG_KEY_METADATA: &'static [ConfigKeyMetadata]` (61 entries) + `EXPECTED_SEED_COUNT_V1_AD: usize = 27` + new `every_seeded_key_has_metadata` parity test. Counts are parametric — task 7 reconciliation gate authoritative. |
| `crates/api/api/src/governance/governance_log.rs` | UPDATE | Add `ENTRY_KIND_ADMIN_CONFIG_CHANGED` + `ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED` consts at line 68+ |
| `.claude/rules/governance-log-entry-kind-registry.md` | CREATE | Initialise registry per advisor directive #2; close GH #41 |
| `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` | UPDATE | Open OQ-V1-AD-01/02/03 (leans only; no resolution) |

**Total: 17 files changed (4 NEW migration dirs × 2 files = 8 migration files, 4 NEW Rust files, 5 UPDATED Rust files, 1 NEW rules file, 1 UPDATED design doc).**

---

## 12. NOT building in v1-AD-a

Explicitly out of scope for this sub-phase. If one of these creeps in, stop and add it as a decision-queue entry.

- No HTTP handlers. Not `POST /admin/config`, not `GET /admin/config`, not `GET /admin/dashboard`. Those are v1-AD-b/d.
- No routes wiring. `crates/api/routes/src/lib.rs` is untouched.
- No DTOs. `crates/api/api_common/src/governance.rs` is untouched.
- No askama, maud, actix-web-lab, or any new template/SSE crate. OQ-V1-AD-01/02 resolve before v1-AD-d/e.
- No `submit_jury_vote` edits. The `rule_set_version_id` wire-up (pinning a case to the active version at decision time) is v1-AD-c.
- No `admin_assign_jury` edits. The `applied_config_snapshot` populate-at-jury-seating is v1-AD-c (touches `submit_jury_vote` + `admin_assign_jury`).
- No `governance_log::append` signature change. Only two consts added.
- No `Scope` enum extension beyond v0. OQ-026 dotted-namespace cascade (`jury.panel_size.<status>.<severity>`) is read-side work that lands alongside jury-mechanics-v1 consumers.
- No federation / ActivityPub work. ADR-014 content-level federation unaffected.
- No config cache invalidation work. v0's per-request `ConfigCache` stays exactly as-is; cross-request invalidation is v1-AD-b scope.
- No AGPL notice updates. Not a release artefact.

---

## 13. Step-by-step tasks

Execute in order. One commit per task on branch `phase-v1-AD-a`. Each task has a MIRROR reference, exact file paths, and a validation command. Task 0 is a pre-flight gate.

### Task 0: PRE-FLIGHT — verify branch + wrapper sanity + v0 baseline

- **ACTION**: Confirm `phase-v1-AD-a` branched from `governance-v0` at `3bbf419da` per `.claude/rules/phase-branch.md`. Run the pre-phase harness audit per `.claude/rules/pre-phase-harness-audit.md` — all four probes against `governance-v0` HEAD, capturing logs under `.claude/PRPs/debug/phase-v1-AD-a-audit-*.log`.
- **GOTCHA**: If the current branch is `governance-v0`, STOP and write a `.claude/decision-queue.json` entry — advisor cuts the phase branch, not the impl agent.
- **GOTCHA**: All four probes must pass AND the negative probes must return non-zero exit codes. Failure = wrapper bug that will invalidate every downstream cargo signal. Fix wrapper in a pre-task commit before Task 1.
- **VALIDATE**:
  ```bash
  git branch --show-current            # → phase-v1-AD-a
  git log -1 --format=%H governance-v0 # → 3bbf419da
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api > .claude/PRPs/debug/phase-v1-AD-a-audit-probe1.log 2>&1"
  echo "probe1 exit: $?"  # expect 0
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features nonexistent_xyz > .claude/PRPs/debug/phase-v1-AD-a-audit-probe4.log 2>&1"
  echo "probe4 exit: $?"  # expect non-zero
  ```
- **EXPECT**: branch correct; exit codes as commented; no commits from this task.

### Task 1: CREATE `migrations/2026-04-21-000000-0000_add_rule_set_versions/up.sql` + down.sql

- **ACTION**: New migration directory. `up.sql` creates `rule_set_version` table per PRD §3.6. `down.sql` drops it.
- **IMPLEMENT**:
  ```sql
  -- up.sql
  CREATE TABLE rule_set_version (
    id            SERIAL PRIMARY KEY,
    community_id  INTEGER NOT NULL REFERENCES community(id) ON DELETE CASCADE,
    version       INTEGER NOT NULL,
    parent_id     INTEGER REFERENCES rule_set_version(id),
    text_sha256   BYTEA NOT NULL,
    rule_text     TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by    INTEGER REFERENCES person(id) ON DELETE RESTRICT,
    UNIQUE (community_id, version)
  );

  CREATE INDEX idx_rule_set_version_community_created
    ON rule_set_version (community_id, created_at DESC);
  ```
  ```sql
  -- down.sql
  DROP INDEX IF EXISTS idx_rule_set_version_community_created;
  DROP TABLE IF EXISTS rule_set_version;
  ```
- **MIRROR**: `migrations/2026-04-15-100100-0000_add_governance_core/up.sql:1-18` for FK + CREATE TABLE shape.
- **GOTCHA**: `parent_id` is **nullable** (v1 of a rule-set has no parent).
- **GOTCHA**: No `CASCADE` on `parent_id` — DELETE of an old version must not cascade-delete newer versions (append-only invariant).
- **GOTCHA**: `text_sha256` is `BYTEA` (32 bytes), not `TEXT`. Consumers will hex-encode when displaying.
- **VALIDATE**: `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema > .claude/PRPs/debug/task1-check.log 2>&1"` — must exit 0. `diesel` CLI is forbidden per `.claude/memory/feedback_lemmy_migration_runner.md`; migrations run via `schema_setup::run` inside `config_parity_round_trip` (task 6).

### Task 2: CREATE `migrations/2026-04-21-000100-0000_add_sponsor_allowlist/up.sql` + down.sql

- **ACTION**: Create empty-reserved `sponsor_allowlist` table per PRD §3.1 + OQ-020. Read-path lands in v1-AD-b; v1-AD-a only ships the table.
- **IMPLEMENT**:
  ```sql
  -- up.sql
  CREATE TABLE sponsor_allowlist (
    id            SERIAL PRIMARY KEY,
    community_id  INTEGER NOT NULL REFERENCES community(id) ON DELETE CASCADE,
    person_id     INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (community_id, person_id)
  );
  ```
  ```sql
  -- down.sql
  DROP TABLE IF EXISTS sponsor_allowlist;
  ```
- **MIRROR**: Same FK shape as task 1. No indexes beyond the UNIQUE constraint in v1-AD-a (read-path indexes land in v1-AD-b when the allowlist strategy is actually consumed).
- **GOTCHA**: The table is **reserved**; no Rust code reads or writes it in v1-AD-a. The Diesel model (task 5) exists for compile-time visibility only.
- **VALIDATE**: `cargo-check.bat -p lemmy_db_schema` — exit 0.

### Task 3: CREATE `migrations/2026-04-21-000200-0000_add_case_applied_config_snapshot/up.sql` + down.sql

- **ACTION**: Add two nullable columns to `moderation_case`.
- **IMPLEMENT**:
  ```sql
  -- up.sql
  ALTER TABLE moderation_case ADD COLUMN applied_config_snapshot JSONB;
  ALTER TABLE moderation_case ADD COLUMN rule_set_version_id INTEGER REFERENCES rule_set_version(id);

  CREATE INDEX idx_moderation_case_rule_set_version
    ON moderation_case (rule_set_version_id)
    WHERE rule_set_version_id IS NOT NULL;
  ```
  ```sql
  -- down.sql
  DROP INDEX IF EXISTS idx_moderation_case_rule_set_version;
  ALTER TABLE moderation_case DROP COLUMN IF EXISTS rule_set_version_id;
  ALTER TABLE moderation_case DROP COLUMN IF EXISTS applied_config_snapshot;
  ```
- **MIRROR**: `migrations/2026-04-18-000100-0000_add_person_membership_state/up.sql` — ALTER TABLE ADD COLUMN precedent.
- **GOTCHA**: Both columns nullable. Existing pre-v1 cases keep NULL. No backfill — populate-at-write in v1-AD-c.
- **GOTCHA**: Must run AFTER task 1 (rule_set_version table must exist before FK). Timestamp ordering (`-000200-0000` after `-000000-0000`) enforces this.
- **GOTCHA**: Partial index `WHERE rule_set_version_id IS NOT NULL` keeps existing-case writes cheap (no index bloat on the common NULL case).
- **VALIDATE**: `cargo-check.bat -p lemmy_db_schema` — exit 0. Task 4 schema.rs update depends on this migration applying in the e2e harness.

### Task 4: UPDATE `crates/db_schema_file/src/schema.rs` — add new tables + columns

- **ACTION**: Hand-edit the `@generated` schema file. Add `rule_set_version` and `sponsor_allowlist` `diesel::table!` blocks. Add the two new columns to `moderation_case`.
- **IMPLEMENT**:
  ```rust
  diesel::table! {
      rule_set_version (id) {
          id -> Int4,
          community_id -> Int4,
          version -> Int4,
          parent_id -> Nullable<Int4>,
          text_sha256 -> Bytea,
          rule_text -> Text,
          created_at -> Timestamptz,
          created_by -> Nullable<Int4>,
      }
  }

  diesel::table! {
      sponsor_allowlist (id) {
          id -> Int4,
          community_id -> Int4,
          person_id -> Int4,
          created_at -> Timestamptz,
      }
  }
  ```
  And extend the existing `moderation_case` block (currently schema.rs:724-742) to add:
  ```rust
          applied_config_snapshot -> Nullable<Jsonb>,
          rule_set_version_id -> Nullable<Int4>,
  ```
- **MIRROR**: `schema.rs:412-455` for `governance_config` + `governance_config_current` hand-added blocks; `schema.rs:724-742` for the existing `moderation_case` block to extend.
- **GOTCHA**: This is a **hand-edit** of an `@generated` file (see schema.rs:1 header). The fork's convention for governance additions is documented in the header comment at schema.rs:426-428 ("Diesel does not auto-detect views, so this `table!` block is hand-written"). New tables added by governance migrations follow the same precedent.
- **GOTCHA**: Add `joinable!(moderation_case -> rule_set_version (rule_set_version_id));` if the schema.rs uses `joinable!` macros for other FK relationships — grep first; if none exist in the governance range, skip.
- **VALIDATE**: `cargo-check.bat -p lemmy_db_schema_file` — exit 0. `cargo-check.bat --workspace` — exit 0.

### Task 5: CREATE `crates/db_schema/src/source/governance/rule_set_version.rs` + `sponsor_allowlist.rs`; UPDATE `mod.rs` + `newtypes.rs`

- **ACTION**: Diesel models for both new tables.
- **IMPLEMENT**: Follow `crates/db_schema/src/source/governance/governance_config.rs:1-53` exactly. Both structs: `RuleSetVersion`/`SponsorAllowlist` Queryable + `RuleSetVersionInsertForm`/`SponsorAllowlistInsertForm` Insertable. Add `RuleSetVersionId(i32)` + `SponsorAllowlistId(i32)` newtypes to `crates/db_schema/src/newtypes.rs` (grep `GovernanceConfigId` for the exact pattern — it's a `pub struct …Id(pub i32)` with derives).
- **IMPORTS**:
  ```rust
  use crate::newtypes::{CommunityId, RuleSetVersionId};
  use chrono::{DateTime, Utc};
  use lemmy_db_schema_file::{schema::rule_set_version, PersonId};
  use serde::{Deserialize, Serialize};
  use serde_with::skip_serializing_none;
  ```
- **EXPORTS** in `crates/db_schema/src/source/governance/mod.rs`: add `pub mod rule_set_version;` and `pub mod sponsor_allowlist;` alphabetically.
- **GOTCHA**: `rule_text` is `String` in Rust (maps to `TEXT`); `text_sha256` is `Vec<u8>` (maps to `BYTEA`). The `skip_serializing_none` + `ts-rs` derives fire on both structs. Do **not** derive `AsChangeset` — both tables are append-only (same reasoning as `GovernanceConfigInsertForm`).
- **GOTCHA**: `SponsorAllowlistInsertForm` is reserved for v1-AD-b. Ship the struct in v1-AD-a; no code calls it yet. A `#[allow(dead_code)]` is NOT needed if the struct is `pub` and exported — exported items are never dead.
- **VALIDATE**: `cargo-check.bat --workspace` — exit 0. `cargo-check.bat --workspace --features full` — exit 0 (forces the `full`-gated derives to compile).

### Task 6: UPDATE `crates/api/api/src/governance/config.rs` — add `ConfigKeyMetadata`, 27 new consts, 61-entry `CONFIG_KEY_METADATA`, new parity test

- **ACTION**: Largest single edit. Four sub-steps:
  1. Add `ConfigKeyMetadata` struct + `ValueType`/`ConfigScope`/`ApplyAt` enums near the existing `Scope` enum (line 58). These are compile-time types; no DB representation.
  2. Add 27 new `pub const DEFAULT_*` declarations alongside the existing 34 (after line 353).
  3. Add 27 new match arms to `const_default_int`/`const_default_float`/`const_default_bool`/`const_default_text` (lines 354-414).
  4. Extend `SEEDED_KEYS_WITH_CONSTS` from 34 to 61 tuples (lines 423-458).
  5. Add `pub const EXPECTED_SEED_COUNT_V1_AD: usize = 27;` near the existing `EXPECTED_SEED_COUNT` (line 463). Do NOT change `EXPECTED_SEED_COUNT` itself — advisor directive #4: each sub-PRD adds its own parametric count.
  6. Add `pub const CONFIG_KEY_METADATA: &'static [ConfigKeyMetadata] = &[...]` with 61 entries (one per seeded key).
  7. Update existing `seeded_keys_count_matches_const_count` test to assert against `EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD`.
  8. Add new `every_seeded_key_has_metadata` parity test.
- **IMPLEMENT**:
  ```rust
  // Add near line 58 (alongside Scope):

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum ValueType {
    Int,
    Float,
    Bool,
    Text,
    // Enum stored as text; variants in ConfigKeyMetadata::valid_enum.
    Enum,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum ConfigScope {
    Instance,
    Community,
    Both,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum ApplyAt {
    Immediate,
    NextJuryCycle,
    NextSnapshotJob,
  }

  #[derive(Debug, Clone, Copy)]
  pub struct NumericRange {
    pub min: f64,
    pub max: f64,
  }

  #[derive(Debug, Clone, Copy)]
  pub struct ConfigKeyMetadata {
    pub key: &'static str,
    pub value_type: ValueType,
    pub valid_range: Option<NumericRange>,
    pub valid_enum: Option<&'static [&'static str]>,
    pub scope: ConfigScope,
    pub requires_re_jury: bool,
    pub requires_step_up: bool,
    pub apply_at_default: ApplyAt,
    pub description: &'static str,
    pub doc_anchor: &'static str,
  }
  ```
  27 new keys to add to `SEEDED_KEYS_WITH_CONSTS` — these MUST match the rows seeded in task 7's migration exactly:
  - `jury.severity_thresholds.minor|moderate|severe` (3× text)
  - `jury.diversity_constraints_enabled` (bool)
  - `jury.appeal_panel_size_increase` (int)
  - `jury.deadline_window_hours` (int)
  - `liability.grace_window_minor_hours|moderate_hours|severe_hours|minimum_hours|maximum_hours|alert_threshold_hours` (6× int)
  - `liability.restoration_escapes_liability` (bool)
  - `liability.restoration_severity_reduction_steps` (int)
  - `liability.multi_sponsor_escape_rule` (text enum)
  - `liability.revoke_rate_limit_per_day` (int)
  - `decay.negative_half_life_days` (int)
  - `decay.endorsement_strength_half_life_days` (int)
  - `decay.jury_reliability_half_life_days` (int)
  - `onboarding.sponsor_min_endorsement_strength` (int)
  - `onboarding.sponsor_allowlist_table_name` (text)
  - `onboarding.provisional_membership_cooldown_days` (int)
  - `founder.founder_seal_visible_in_profile` (bool)
  - `participation.weekly_active_delta` (int)
  - `participation.dormant_threshold_days` (int)
  - `participation.dormant_delta` (int)
  - `participation.attestation_enabled` (bool)
  - `federation.inbound_advisory_only` (bool)
  - `federation.peer_attestation_ttl_days` (int)
  - `federation.signature_required` (bool)
  - `federation.quarantine_recommendation_severity_floor` (text enum)
  - `federation.outbound_publish_enabled` (bool)
  - `rule_set.auto_carry_in_flight_cases` (bool)
  - `rule_set.text_max_bytes` (int)
  - `rule_set.version_propagation_delay_hours` (int)

  **`rule_set.active_version_id` is NOT in this list.** Per §4.1 decision (advisor edit #2, 2026-04-19): no seed row, no const, no `CONFIG_KEY_METADATA` entry in v1-AD-a. v1-AD-c adds all three when implementing rule-set creation, because until a rule set exists the key is semantically not-yet-decided rather than "has a default". v1-AD-b's `config::get_int_opt` accessor returns `None` when both the DB row and the const default are absent — that `None` IS the "no active version" signal. DO NOT add `DEFAULT_RULE_SET_ACTIVE_VERSION_ID` or a sentinel value.

  **Count: 27 exact.** Task 7's migration must INSERT exactly these 27 keys (28-in-PRD-§5.2 minus `rule_set.active_version_id`).
- **MIRROR**: `config.rs:469-500` for parity test shape; `config.rs:319-352` for DEFAULT_* const style; `config.rs:354-414` for match-arm style.
- **GOTCHA**: `rule_set.active_version_id` — per §4.1 decision (advisor edit #2) this key is DELIBERATELY OMITTED from v1-AD-a. No const, no seed row, no `SEEDED_KEYS_WITH_CONSTS` entry, no `CONFIG_KEY_METADATA` entry, no `const_default_int` match arm. The v0 config reader already returns an error when no DB row matches AND no const fallback exists. v1-AD-b will introduce a `config::get_int_opt` accessor that treats the "no row + no const" state as `None` instead of an error. Until v1-AD-c seeds the key on actual rule-set activation, any read via the normal `get_int` path produces an error — that's the correct behaviour because no caller in v1-AD-a/b should be reading an unactivated rule-set version. If the implementer is tempted to ship a `-1` sentinel "just for completeness", STOP — write a DQ entry citing advisor edit #2.
- **GOTCHA**: `liability.multi_sponsor_escape_rule` is an enum text (`any_revocation | all_revocation | majority_revocation`). `valid_enum` field in its ConfigKeyMetadata entry pins the three strings.
- **GOTCHA**: `participation.attestation_enabled = false` is (c) decide-later per PRD §5.2. Ship the key, the const, the metadata, the seed row. No read-site code in v1-AD-a.
- **GOTCHA**: Do not delete or weaken the existing `every_seeded_key_has_const_fallback` test. Add the new `every_seeded_key_has_metadata` test **alongside** it.
- **GOTCHA**: `NumericRange` is a compile-time struct, not the workspace `f64` range lint trap — `valid_range` is `Option<NumericRange>`, not `RangeInclusive<f64>`. Two f64 fields + a Copy derive keeps the `&'static` storage commitment working.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/task6-check.log 2>&1"
  echo "check exit: $?"   # expect 0
  cmd //c "scripts\\brehon\\cargo-test.bat --workspace --lib parity > .claude/PRPs/debug/task6-parity.log 2>&1"
  echo "parity exit: $?"  # expect 0
  ```
  Do NOT use `-p lemmy_api --features full` per advisor memory `feedback_features_full_workspace_only.md`.

### Task 7: CREATE `migrations/2026-04-21-000300-0000_seed_v1_config_keys/up.sql` + down.sql; UPDATE `crates/api/api/src/governance/governance_log.rs` with 2 new ENTRY_KIND consts

- **ACTION**: Seed the 27 new rows idempotently. The INSERT rows MUST match task 6's `SEEDED_KEYS_WITH_CONSTS` entries byte-for-byte. Also land the two new ENTRY_KIND consts (combined task because both are small, non-conflicting, and the parity test + migration test share a validation step).
- **IMPLEMENT** (migration):

  **⚠ NON-AUTHORITATIVE SQL BLOCK — reconciliation at plan-review (see GOTCHA below).** The rows listed below are drawn from PRD §5.2's full v1 cross-PRD enumeration and include keys owned by sponsor-liability-v1 (10× `liability.*`), federation-inbound-v1 (5× `federation.*` partial), reputation-tuning-v1 (3× `decay.*` partial, 4× `participation.*` partial), and jury-mechanics-v1 (some `jury.*` cascade rows). v1-AD-a ships ONLY the admin-dashboard-owned subset (per PRD §3.1 contribution sub-table: ~24 keys minus `rule_set.active_version_id` = **expected 23-27 admin-dashboard rows** — reconciliation gate below narrows this).

  The authoritative count (`EXPECTED_SEED_COUNT_V1_AD`, currently estimated 27) is reconciled at task-7-pre-commit time against PRD §3.1's per-sub-PRD ownership split, NOT against the rows listed below. The implementer:
  1. Reads PRD §3.1 contribution sub-table to extract admin-dashboard-owned keys
  2. Writes task 6's `SEEDED_KEYS_WITH_CONSTS` extension to match
  3. Writes task 7's INSERT block to match task 6 byte-for-byte
  4. Runs the reconciliation gate (below) before committing
  5. Adjusts `EXPECTED_SEED_COUNT_V1_AD` to the actual count

  If the implementer is unclear which keys belong to admin-dashboard vs sponsor-liability vs reputation-tuning etc., they MUST write a DQ entry pointing to PRD §3.1 "per-PRD contribution sub-table" and wait for advisor clarification — do NOT self-resolve by picking keys arbitrarily from the SQL block below.

  ```sql
  -- up.sql — NON-AUTHORITATIVE, see reconciliation gate above.
  -- Full PRD §5.2 enumeration (35 rows); v1-AD-a ships admin-dashboard-
  -- owned subset only (~27). Sponsor-liability, federation-inbound,
  -- reputation-tuning, and jury-mechanics keys below land in their own
  -- sub-PRD plans, NOT in v1-AD-a.
  INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text) VALUES
    ('instance', 'jury.severity_thresholds.minor',              'text',  NULL,  NULL, NULL,  'majority'),
    ('instance', 'jury.severity_thresholds.moderate',           'text',  NULL,  NULL, NULL,  '60%'),
    ('instance', 'jury.severity_thresholds.severe',             'text',  NULL,  NULL, NULL,  '75%'),
    ('instance', 'jury.diversity_constraints_enabled',          'bool',  NULL,  NULL, true,  NULL),
    ('instance', 'jury.appeal_panel_size_increase',             'int',   2,     NULL, NULL,  NULL),
    ('instance', 'jury.deadline_window_hours',                  'int',   72,    NULL, NULL,  NULL),
    ('instance', 'liability.grace_window_minor_hours',          'int',   24,    NULL, NULL,  NULL),
    ('instance', 'liability.grace_window_moderate_hours',       'int',   72,    NULL, NULL,  NULL),
    ('instance', 'liability.grace_window_severe_hours',         'int',   168,   NULL, NULL,  NULL),
    ('instance', 'liability.grace_window_minimum_hours',        'int',   1,     NULL, NULL,  NULL),
    ('instance', 'liability.grace_window_maximum_hours',        'int',   720,   NULL, NULL,  NULL),
    ('instance', 'liability.grace_window_alert_threshold_hours','int',   24,    NULL, NULL,  NULL),
    ('instance', 'liability.restoration_escapes_liability',     'bool',  NULL,  NULL, true,  NULL),
    ('instance', 'liability.restoration_severity_reduction_steps','int', 0,     NULL, NULL,  NULL),
    ('instance', 'liability.multi_sponsor_escape_rule',         'text',  NULL,  NULL, NULL,  'any_revocation'),
    ('instance', 'liability.revoke_rate_limit_per_day',         'int',   5,     NULL, NULL,  NULL),
    ('instance', 'decay.negative_half_life_days',               'int',   180,   NULL, NULL,  NULL),
    ('instance', 'decay.endorsement_strength_half_life_days',   'int',   90,    NULL, NULL,  NULL),
    ('instance', 'decay.jury_reliability_half_life_days',       'int',   90,    NULL, NULL,  NULL),
    ('instance', 'onboarding.sponsor_min_endorsement_strength', 'int',   25,    NULL, NULL,  NULL),
    ('instance', 'onboarding.sponsor_allowlist_table_name',     'text',  NULL,  NULL, NULL,  'sponsor_allowlist'),
    ('instance', 'onboarding.provisional_membership_cooldown_days','int',14,    NULL, NULL,  NULL),
    ('instance', 'founder.founder_seal_visible_in_profile',     'bool',  NULL,  NULL, true,  NULL),
    ('instance', 'participation.weekly_active_delta',           'int',   1,     NULL, NULL,  NULL),
    ('instance', 'participation.dormant_threshold_days',        'int',   30,    NULL, NULL,  NULL),
    ('instance', 'participation.dormant_delta',                 'int',   -2,    NULL, NULL,  NULL),
    ('instance', 'participation.attestation_enabled',           'bool',  NULL,  NULL, false, NULL),
    ('instance', 'federation.inbound_advisory_only',            'bool',  NULL,  NULL, true,  NULL),
    ('instance', 'federation.peer_attestation_ttl_days',        'int',   30,    NULL, NULL,  NULL),
    ('instance', 'federation.signature_required',               'bool',  NULL,  NULL, true,  NULL),
    ('instance', 'federation.quarantine_recommendation_severity_floor','text',NULL,NULL,NULL,'moderate'),
    ('instance', 'federation.outbound_publish_enabled',         'bool',  NULL,  NULL, true,  NULL),
    -- rule_set.active_version_id deliberately NOT seeded — absence-of-row IS
    -- the "no active version" signal. See §4.1 and task 6 GOTCHA.
    ('instance', 'rule_set.auto_carry_in_flight_cases',         'bool',  NULL,  NULL, true,  NULL),
    ('instance', 'rule_set.text_max_bytes',                     'int',   65536, NULL, NULL,  NULL),
    ('instance', 'rule_set.version_propagation_delay_hours',    'int',   24,    NULL, NULL,  NULL)
  ON CONFLICT (scope, key, valid_from) DO NOTHING;
  ```
  **Count: 27 unique keys** (audit by namespace):
  - jury: severity_thresholds ×3 + diversity + panel_size_increase + deadline = **6**
  - liability: grace_window ×6 + restoration ×2 + multi_sponsor + revoke_rate = **10**
  - decay: negative + endorsement_strength + jury_reliability = **3**
  - onboarding: sponsor_min_endorsement_strength + sponsor_allowlist_table_name + provisional_cooldown = **3**
  - founder: founder_seal_visible_in_profile = **1**
  - participation: weekly_active + dormant_threshold + dormant_delta + attestation_enabled = **4**
  - federation: inbound_advisory + peer_ttl + signature_required + quarantine_floor + outbound_publish = **5** *wait, check below*
  - rule_set: auto_carry + text_max_bytes + version_propagation_delay = **3** (active_version_id omitted per §4.1)
  
  Namespace audit total: 6 + 10 + 3 + 3 + 1 + 4 + 5 + 3 = **35** — this mismatches the 27-key count in §4.1. The drift exists because PRD §5.2 enumerates ~35 rows collectively in-scope for admin-dashboard + sponsor-liability-v1 (the latter owns 10 of the liability keys per B4). **v1-AD-a implementer owns only the admin-dashboard-exclusive keys** (PRD §3.1 contribution sub-table: "~24" admin-dashboard additions minus `rule_set.active_version_id` = ~23). This is the exact drift flagged in advisor edit #1.
  
  **Advisor edit #1 reconciliation point (MANDATORY before committing this migration):**
  
  Before `git commit` of task 7's migration, run:
  ```bash
  # Count SEEDED_KEYS_WITH_CONSTS.len() from task 6's edit
  grep -cE '^\s*\("[a-z_.]+", "DEFAULT_' crates/api/api/src/governance/config.rs
  # Expected: EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD (parametric —
  # currently 34 + 27 = 61, reconcile below if drift)
  
  # Count the INSERT rows written to task 7's up.sql
  grep -cE "^\s*\('instance'," migrations/2026-04-21-000300-0000_seed_v1_config_keys/up.sql
  # MUST equal EXPECTED_SEED_COUNT_V1_AD exactly
  
  # Dry-run the parametric parity test
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server config_parity_round_trip > .claude/PRPs/debug/task7-precommit-parity.log 2>&1"
  echo "precommit parity exit: $?"
  ```
  
  If the parity test fails at this point, DO NOT commit. The three outcomes:
  - **(a) const count > insert count**: add missing INSERT rows until they match.
  - **(b) const count < insert count**: remove surplus INSERT rows until they match.
  - **(c) neither side matches PRD §5.2's authoritative enumeration**: open a DQ entry citing advisor edit #1 with the actual count and wait for reconciliation. Do not self-resolve the drift — PRD §5.2 and sponsor-liability-v1 §4.2 ownership is the source of truth, not the plan file.
  
  Rationale: this converts count drift from a "surprise at task 7 end" to "reconciliation point explicit at migration-write time." `EXPECTED_SEED_COUNT_V1_AD = 27` in §4.1 is the plan's current best count; the implementer is authoritative against on-disk reality.
  
  ```sql
  -- down.sql
  DELETE FROM governance_config WHERE scope = 'instance' AND key IN (
    'jury.severity_thresholds.minor',
    -- … all 27 keys in the same order as up.sql's INSERT block …
    'rule_set.version_propagation_delay_hours'
  );
  ```
- **IMPLEMENT** (governance_log.rs):
  ```rust
  // Appended after line 68 (after ENTRY_KIND_APPEAL_REQUESTED):
  pub const ENTRY_KIND_ADMIN_CONFIG_CHANGED: &str = "admin_config_changed";
  pub const ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED: &str = "admin_config_change_denied";
  ```
- **MIRROR**: `migrations/2026-04-18-000000-0000_add_governance_config/up.sql:78-113` for INSERT shape + `ON CONFLICT (scope, key, valid_from)` target. `governance_log.rs:50-68` for ENTRY_KIND const style.
- **GOTCHA**: `ON CONFLICT (scope, key, valid_from) DO NOTHING` — `valid_from` defaults to `now()`, so two runs of `schema_setup::run()` at different times would each insert a fresh row with a different `valid_from` — the conflict key means they DON'T duplicate only when `valid_from` matches exactly. The v0 migration handles this by running inside the same single-transaction schema application, where `now()` returns the same timestamp for all rows in the same statement. Same applies here; no change needed.
- **GOTCHA**: `admin_config_changed` const must equal the existing shell-script string literal byte-for-byte (`scripts/brehon/admin-config-write.sh:148`). v1-AD-b's NOT5 deprecation gate 3 asserts byte-identical governance_log rows from both paths.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/task7-check.log 2>&1"
  echo "check exit: $?"
  # Run config_parity_round_trip — exercises task 7's seed migration + task 6's consts + metadata registry
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server config_parity_round_trip > .claude/PRPs/debug/task7-e2e.log 2>&1"
  echo "e2e exit: $?"
  ```
  Expect both to exit 0. The `config_parity_round_trip` test at `crates/server/tests/e2e.rs:1308-1366` will iterate all `SEEDED_KEYS_WITH_CONSTS` entries (currently `EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD` = 34 + 27 = 61 post-reconciliation) and call the typed accessor — any missing seed row, missing const, or type mismatch fails here.

### Task 8: INITIALISE `.claude/rules/governance-log-entry-kind-registry.md` (closes #41)

- **ACTION**: Per advisor directive #2, close GH #41 as part of v1-AD-a rather than maintaining a separate issue-driven flow. Write a new auto-loaded rule file whose section structure **mirrors issue #41's "Proposed deliverable" enumeration** so the issue closes with a one-line "landed in v1-AD-a at `<commit>`" comment (advisor edit #3).
- **IMPLEMENT**: Template (4-column rows: const name, &str value, source/writer, emitting-handler file path + one-line semantic description — per issue #41 "Proposed deliverable" bullet 1):
  ```markdown
  # governance_log entry_kind registry

  The `governance_log` table records every governance event as a row with an
  `entry_kind TEXT` column. Each kind string must appear exactly once as a
  `pub const ENTRY_KIND_*` in `crates/api/api/src/governance/governance_log.rs`.

  Adding a new kind requires:
  1. A new `pub const ENTRY_KIND_<NAME>: &str = "<snake_case>";` in
     `crates/api/api/src/governance/governance_log.rs`.
  2. An entry in the matching PRD section of THIS file with: const name,
     &str value, source PRD (v0 = "shipped", v1+ = PRD name), emitting
     handler file path, one-line semantic description.
  3. Grep DoD in plan file: `rg "ENTRY_KIND_<NAME>" crates/api/api/src/governance/governance_log.rs`
     returns the const; `rg '"<snake_case>"' crates/` returns the const line
     + every call site.
  4. Collision check at plan-review: the count of lowercase string literals in
     governance_log.rs (the entry_kind &str values) must equal the count of
     `pub const ENTRY_KIND_*` declarations in the same file, and every literal
     must appear exactly once. Two greps, same file, same domain:
     ```bash
     rg -c '^pub const ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs
     # Count A: number of ENTRY_KIND_* consts

     rg -oE '"[a-z_]+"' crates/api/api/src/governance/governance_log.rs | sort -u | wc -l
     # Count B: number of unique lowercase string literals

     # Invariant: A == B, AND no duplicate literals
     rg -oE '"[a-z_]+"' crates/api/api/src/governance/governance_log.rs | sort | uniq -d
     # Expected: empty (no duplicates)
     ```
     NOTE: this invariant is **governance-log-domain only** — it does NOT
     reference `EXPECTED_SEED_COUNT` or `EXPECTED_SEED_COUNT_V1_*` (which are
     config-key-seed parity counters, a different domain).

  This file is MANDATORY READING in `-p` mode. Every PRP plan that touches
  governance writes reads it at first iteration. Mirrors
  `feedback_plan_dod_dry_run_at_write.md` at plan-review time.

  ## v0 entry kinds (19, shipped governance-v0 `3bbf419da`)

  | Rust const | &str value | Source | Emitting handler | Semantic |
  |---|---|---|---|---|
  | `ENTRY_KIND_REPORT_CREATED` | `report_created` | Phase 4 shipped | `crates/api/api/src/governance/create_report.rs` | Report submitted, case opened or threshold-appended |
  | `ENTRY_KIND_THRESHOLD_MET` | `threshold_met` | Phase 4 shipped | `create_report.rs` | Report accumulation crossed `report.case_threshold_micros` |
  | `ENTRY_KIND_JURY_ASSIGNED` | `jury_assigned` | Phase 4 shipped | `admin_assign_jury.rs` | Admin assigned jury to a case |
  | `ENTRY_KIND_PANEL_ASSEMBLED` | `panel_assembled` | Phase 4b shipped | `admin_assign_jury.rs` | Panel selection ran; jurors seated |
  | `ENTRY_KIND_JURY_VOTED` | `jury_voted` | Phase 4b shipped | `submit_jury_vote.rs` | Individual juror submitted vote |
  | `ENTRY_KIND_SANCTION_CREATED` | `sanction_created` | Phase 4b shipped | `submit_jury_vote.rs` | Quorum reached → sanction row inserted |
  | `ENTRY_KIND_PUBLIC_LOG_PUBLISHED` | `public_log_published` | Phase 4b shipped | `submit_jury_vote.rs` | Redacted public case log entry created |
  | `ENTRY_KIND_REPUTATION_DELTA` | `reputation_delta` | Phase 5a shipped | `reputation_snapshot.rs` + call sites | Reputation event applied |
  | `ENTRY_KIND_CASE_DECIDED` | `case_decided` | Phase 4b shipped | `submit_jury_vote.rs` | Case transitioned to `Decided` |
  | `ENTRY_KIND_CAPABILITY_CHANGED` | `capability_changed` | Phase 5a shipped | `reputation_snapshot.rs` | `can_sponsor`/`jury_eligible` flip |
  | `ENTRY_KIND_SPONSOR_LIABILITY_APPLIED` | `sponsor_liability_applied` | Phase 5b shipped | `sponsor_liability.rs` | Liability delta written |
  | `ENTRY_KIND_SPONSOR_LIABILITY_CLAMPED` | `sponsor_liability_clamped` | Phase 5b shipped | `sponsor_liability.rs` | Floor clamp fired (OQ-024) |
  | `ENTRY_KIND_FOUNDER_SEEDED` | `founder_seeded` | Phase 5b shipped | `seed_founders` CLI + `reputation_snapshot.rs` | Founder reputation-event inserted |
  | `ENTRY_KIND_ENDORSEMENT_CREATED` | `endorsement_created` | Phase 5b shipped | `create_endorsement.rs` | Sponsor-endorsement written |
  | `ENTRY_KIND_EMERGENCY_REMOVED` | `emergency_removed` | Phase 5c shipped | `admin_emergency_remove.rs` | ADR-013 admin-override removal |
  | `ENTRY_KIND_JURY_ACCEPTED` | `jury_accepted` | Phase 5c shipped | `accept_jury_assignment.rs` | Juror accepted assignment |
  | `ENTRY_KIND_JURY_DECLINED` | `jury_declined` | Phase 5c shipped | `decline_jury_assignment.rs` | Juror declined assignment |
  | `ENTRY_KIND_JURY_REPLACEMENT_SELECTED` | `jury_replacement_selected` | Phase 5c shipped | `admin_assign_jury.rs` | Replacement juror seated after decline |
  | `ENTRY_KIND_APPEAL_REQUESTED` | `appeal_requested` | Phase 5c shipped | `request_appeal.rs` | Appeal request submitted |

  ## Phase 6 pending additions (≥4, in-flight on `phase-6` branch)

  Phase 6 is in-flight per `advisor-context-phase-6.md` §4 Watch 5 and has
  not merged to `governance-v0` at v1-AD-a write time. The expected kinds
  are enumerated below for collision-avoidance only; this file is updated
  to include emitting-handler paths once Phase 6 merges and their file
  locations are authoritative.

  | Rust const (expected) | &str value | Source | Emitting handler | Semantic |
  |---|---|---|---|---|
  | `ENTRY_KIND_FEDERATION_SANCTION_SENT` | `federation_sanction_sent` | Phase 6 pending | `crates/apub/activities/` (TBD post-merge) | Outbound sanction AP activity published |
  | `ENTRY_KIND_FEDERATION_SANCTION_RECEIVED` | `federation_sanction_received` | Phase 6 pending | Inbox handler (TBD) | Inbound sanction AP received (advisory) |
  | `ENTRY_KIND_FEDERATION_ATTESTATION_SENT` | `federation_attestation_sent` | Phase 6 pending | `crates/apub/activities/` | Outbound attestation published |
  | `ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED` | `federation_attestation_received` | Phase 6 pending | Inbox handler | Inbound attestation received |

  Upon Phase 6 merge: v1-AD-a planner (or Phase-6-merge-close retro)
  refreshes this section with actual const names and handler paths.
  `federation-inbound-v1` extends this section (see its own entry below)
  with its own 11-kind set.

  ## v1-AD-a additions (2, this sub-phase)

  | Rust const | &str value | Source | Emitting handler | Semantic |
  |---|---|---|---|---|
  | `ENTRY_KIND_ADMIN_CONFIG_CHANGED` | `admin_config_changed` | v1-AD-a (this plan) — const; v1-AD-b call site | Emitted by: v1-AD-b `crates/api/api/src/governance/admin_config.rs` (pending); v0 `scripts/brehon/admin-config-write.sh:148` (byte-identical payload per NOT5 gate 3) | Config row edit succeeded; payload carries `{scope, key, value_type, value, reason}` |
  | `ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED` | `admin_config_change_denied` | v1-AD-a (this plan) — const; v1-AD-b call site | v1-AD-b `admin_config.rs` capability-check reject path | Capability/scope check rejected attempted config write; payload mirrors attempted-change with `denial_reason` |

  ## v1 PRD reservation sections (to be populated when each PRD's plans write)

  Each future v1 sub-PRD OWNS a section below. Populated by the sub-PRD's
  own plan file at task-N-equivalent. Reserved slot avoids re-ordering when
  a later PRD lands first.

  ### jury-mechanics-v1 (reserved — §8.5 of PRD enumerates 6 new kinds)
  _To be populated by `v1-jury-mechanics.plan.md` task n: `jury_constraint_relaxed`, `appeal_panel_assembled`, `appeal_decided`, `appeal_rejected`, `appeal_window_expired`, `severity_tier_frozen`._

  ### sponsor-liability-v1 (reserved — §17 of PRD enumerates 5 new kinds)
  _To be populated by `v1-sponsor-liability.plan.md`: `sponsor_liability_pending`, `sponsor_liability_fired`, `sponsor_liability_escaped`, `endorsement_revoked`, `restoration_completed`._

  ### reputation-tuning-v1 (reserved — §7 of PRD enumerates 7 new kinds)
  _To be populated by `v1-reputation-tuning.plan.md`: `participation_cron_tick`, `vote_outcome_recorded`, `evidence_quality_recorded`, `rollup_recomputed`, `decay_knob_changed`, `sponsor_allowlist_added`, `sponsor_allowlist_removed`._

  ### federation-inbound-v1 (reserved — §§5.3/6.2/6.3/8.2 of PRD enumerate 11 new kinds)
  _To be populated by `v1-federation-inbound.plan.md`: `federation_label_received`, `federation_inbound_blocked`, `federation_inbound_dropped_oversize`, `federation_inbound_dropped_schema`, `federation_inbound_dropped_rate_limit`, `federation_inbound_dropped_actor_rate_limit`, `federation_inbound_persist_failed`, `federation_inbound_cross_linked`, `federation_inbound_dismissed`, `federation_peer_trust_changed`, `federation_inbound_storage_cap_evicted`._

  ## Acceptance invariants (checked at every plan-review)

  - [ ] `rg "^pub const ENTRY_KIND_" crates/api/api/src/governance/governance_log.rs | wc -l` returns total count of all populated sections
  - [ ] `rg -n '"[a-z_]+"' crates/api/api/src/governance/governance_log.rs | sort | uniq -d` returns no duplicates
  - [ ] Every populated row in this file has a Rust const AND a call site (except Phase 6 pending rows, which only require a plan file)
  - [ ] Registry file matches the "Proposed deliverable" enumeration of GH issue #41 at land-time
  ```
- **MIRROR**: GH issue #41 body "Proposed deliverable" section verbatim (column count, section structure, acceptance invariants).
- **GOTCHA**: This replaces GH #41's tracker role. When this file lands (PR merge to `governance-v0`), close #41 with a comment: "Registry landed at `.claude/rules/governance-log-entry-kind-registry.md` in v1-AD-a. Subsequent v1 PRDs append their section via their own `/prp-plan`→`/prp-ralph` runs."
- **GOTCHA**: Phase 6's 4 kinds are listed with "(expected)" + "TBD post-merge" handler paths because Phase 6 is in-flight and may land with slightly different constant names. The plan does NOT block on Phase 6's exact naming — after Phase 6 merges to `governance-v0`, the first v1 sub-PRD to plan refreshes this section. Until then, the registry serves collision-detection for v1-AD-a's 2 kinds + future v1 plans' kinds against each other.
- **GOTCHA (advisor note 2 — impl-time Phase 6 merge-state check):** this plan was written at 2026-04-19 with Phase 6 in-flight. If Phase 6 merges to `governance-v0` between plan-write and this task's ralph iteration, the "(expected)" + "TBD post-merge" labels are already stale at impl time. Task 8 must re-check merge state at impl time, NOT rely on the plan's static claim. If Phase 6 has merged, read the actual federation `ENTRY_KIND_*` constants from `crates/api/api/src/governance/governance_log.rs` and refresh the Phase 6 section with real names + real handler paths + drop the "(expected)" labels. If Phase 6 has NOT merged, labels stay as-is. Converts a static plan claim into an impl-time-verified claim.
- **VALIDATE**:
  ```bash
  # Phase 6 merge-state check (advisor note 2) — determines whether to drop "(expected)" labels
  git log governance-v0 --grep="Phase 6\|phase-6\|phase 6" --oneline -1
  # If non-empty: Phase 6 has merged; task 8 must refresh the Phase 6 section
  # with actual ENTRY_KIND names from governance_log.rs and drop "(expected)".
  # If empty: Phase 6 still in-flight; labels stay.
  
  rg "^pub const ENTRY_KIND_" crates/api/api/src/governance/governance_log.rs | wc -l
  # Expected pre-Phase-6-merge: 21  (19 v0 + 2 v1-AD-a)
  # Expected post-Phase-6-merge: 21 + (Phase 6 kind count, likely 4) = 25
  # Acceptance invariant is parametric: registry file populated-section rows
  # must equal this grep's output exactly.
  
  rg -n '"[a-z_]+"' crates/api/api/src/governance/governance_log.rs | awk -F: '/ENTRY_KIND_/ {print}' | grep -oE '"[a-z_]+"' | sort | uniq -d
  # Expected: empty output (no duplicates)
  
  # Registry file structure match
  grep -c "^## " .claude/rules/governance-log-entry-kind-registry.md
  # Expected: >= 7  (v0, Phase 6 pending, v1-AD-a, 4 reserved sections, acceptance)
  ```

### Task 9: OPEN OQ-V1-AD-01/02/03 in `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`

- **ACTION**: Append three open questions after existing OQ-026 in `99-decisions-and-open-questions.md`. Each is a ≤150-word entry matching the existing OQ template shape (Opened / Owner / Question / Current lean / Blocks / Target). Leans follow advisor directive #3.
- **IMPLEMENT**: Three entries, each:

  ```markdown
  ### OQ-V1-AD-01 — Server-rendered HTML page framework for admin dashboard

  - **Opened:** 2026-04-21
  - **Owner:** TBD (backend)
  - **Question:** v1 admin-dashboard PRD §6 assumes askama (or maud) for
    server-rendered admin HTML pages. Neither crate is in the workspace
    `Cargo.lock` today — adoption would add a templating dep + compile-time
    step to `lemmy_api`. Alternative: ship v1-AD-d API-only and defer HTML
    pages to v1.x or v2 (to land alongside the React frontend pass).
  - **Current lean:** Defer to v1.x. v1-AD-d ships API-only; askama decision
    revisits after pilot operator feedback on whether curl+jq-only workflow
    is operationally sufficient.
  - **Blocks:** v1-AD-e (askama HTML pages sub-phase).
  - **Target:** Before v1-AD-e plan writes.
  ```

  ```markdown
  ### OQ-V1-AD-02 — SSE implementation: actix-web-lab vs hand-rolled

  - **Opened:** 2026-04-21
  - **Owner:** TBD (backend)
  - **Question:** PRD §4 enumerates `GET /api/v4/governance/admin/audit/stream`
    as an SSE endpoint. Upstream Lemmy does not use `actix-web-lab` (grep
    of `Cargo.lock` 2026-04-19 returns no matches). Two paths: (a) add
    `actix-web-lab` dep for its `Sse` helper; (b) hand-roll ~80 LOC of
    chunked-response plumbing using `async-stream` (already a transitive
    dep). The Postgres `LISTEN governance_events` channel already exists
    (migration `2026-04-20-000000-0000`) — the SSE endpoint's only job is
    bridging Postgres NOTIFY payloads onto HTTP.
  - **Current lean:** Hand-roll (option b). Adds no new supply-chain
    surface and keeps Lemmy's actix-web dep shape intact.
  - **Blocks:** v1-AD-d (dashboard + SSE sub-phase).
  - **Target:** Before v1-AD-d plan writes.
  ```

  ```markdown
  ### OQ-V1-AD-03 — Dry-run impact computation inside vs outside run_transaction

  - **Opened:** 2026-04-21
  - **Owner:** TBD (backend)
  - **Question:** PRD §4.3 proposes running dry-run impact queries inside
    the same `run_transaction` as the config write with a `SAVEPOINT`
    rollback when `dry_run = true`. Inspection of
    `crates/diesel_utils/src/connection.rs:68-79` shows `run_transaction`
    wraps `diesel-async`'s `.transaction()` — no SAVEPOINT primitive is
    exposed. Either (a) extend `run_transaction` with a `run_savepoint`
    helper, or (b) compute impact as a read-only query BEFORE the tx opens
    against the proposed value and the current config snapshot, with no
    rollback needed.
  - **Current lean:** Option (b). Architecturally cleaner; dry-run is
    read-only; no tx-state visibility required. Reshape PRD §4.3 to match.
  - **Blocks:** v1-AD-b (POST /admin/config sub-phase).
  - **Target:** Before v1-AD-b plan writes.
  ```

- **MIRROR**: `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md:431-438` for OQ-018 as the exact shape to follow.
- **GOTCHA**: OQs are append-only. Do not edit existing ADRs. Do not resolve these three in v1-AD-a — resolution is user/advisor responsibility before v1-AD-b/c/d plans write.
- **GOTCHA**: The changelog section at the bottom of `99-decisions-and-open-questions.md` (line 498+) needs an append entry:
  ```markdown
  **2026-04-21** — *99*
  OQ-V1-AD-01, OQ-V1-AD-02, OQ-V1-AD-03 opened. All three block v1-AD
  sub-phases (e/d/b respectively). Leans documented; resolution before
  dependent plans write.
  ```
- **VALIDATE**: Grep check: `grep -c "^### OQ-" docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` returns the previous count + 3. Markdown lint pass (if one is in CI) passes.

---

## 14. Testing strategy

v0's test substrate already exists:
- Unit tests: `crates/api/api/src/governance/config.rs::parity` (2 tests currently; becomes 3 with `every_seeded_key_has_metadata`)
- Integration: `crates/server/tests/e2e.rs::config_parity_round_trip` — exercises every seeded key through the typed accessor against a real Postgres + all migrations applied

v1-AD-a adds **no new test files**. It extends the two existing tests:

### Tests added in this sub-phase

| Test | Location | What it validates |
|---|---|---|
| `parity::every_seeded_key_has_metadata` (NEW) | `crates/api/api/src/governance/config.rs::parity` | Every `SEEDED_KEYS_WITH_CONSTS` entry has a matching `CONFIG_KEY_METADATA` row |
| `parity::seeded_keys_count_matches_const_count` (EXTENDED) | same file | Now asserts `SEEDED_KEYS_WITH_CONSTS.len() == EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD` |
| `config_parity_round_trip` (UNCHANGED BODY, BROADER COVERAGE) | `crates/server/tests/e2e.rs:1308-1366` | Runs one typed read per `SEEDED_KEYS_WITH_CONSTS` entry (currently 61 = EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD) against a real Postgres with the matching seeded rows |

### Edge cases covered by existing tests

- [x] Hash-chain integrity (unaffected — v1-AD-a doesn't write to governance_log)
- [x] `actor_pseudonym` (unaffected — v1-AD-a writes no rows)
- [x] `EmergencyRemove` branch (unaffected — `CaseStatus` enum untouched)
- [x] Redaction (unaffected — no payload writes)
- [x] Migration up+down round-trip via `config_parity_round_trip` run → fresh schema apply

### Edge cases added

- [x] `rule_set_version.parent_id` nullability — null-safe Queryable derive
- [x] `moderation_case.applied_config_snapshot` JSONB optionality — Nullable<Jsonb> round-trips via `config_parity_round_trip` path (schema apply)
- [x] Seed migration idempotency — running `schema_setup::run()` twice doesn't duplicate rows (already enforced by `ON CONFLICT DO NOTHING`; `config_parity_round_trip` runs once per test invocation, same transaction)

---

## 15. Validation commands (DoD)

**Every command below was dry-run-tested against governance-v0 HEAD 3bbf419da on 2026-04-19 per `.claude/rules/pre-phase-harness-audit.md` + advisor directive.** Expected exit codes annotated inline.

### Level 1: per-task `cargo check` (run after every task)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace > .claude/PRPs/debug/phase-v1-AD-a-level1-taskN.log 2>&1"
echo "exit: $?"
```
**EXPECT**: 0. Reviews per-task log via `tail -20 .claude/PRPs/debug/phase-v1-AD-a-level1-taskN.log` per `.claude/rules/no-cargo-output-paste.md`.

### Level 2: `--features full` (tasks 5, 6, 7 specifically)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/phase-v1-AD-a-level2-taskN.log 2>&1"
echo "exit: $?"
```
**EXPECT**: 0. Forces the `full`-gated Diesel derives on new Queryable/Insertable structs to compile. NEVER use `-p <crate> --features full` per advisor memory — the `lemmy_server` crate doesn't declare the `full` feature.

### Level 3: parity unit tests

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --lib parity > .claude/PRPs/debug/phase-v1-AD-a-parity.log 2>&1"
echo "exit: $?"
```
**EXPECT**: 0. Three tests pass: `seeded_keys_count_matches_const_count`, `every_seeded_key_has_const_fallback`, `every_seeded_key_has_metadata`.

### Level 4: e2e parity + migration round-trip

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server --no-run > .claude/PRPs/debug/phase-v1-AD-a-e2e-build.log 2>&1"
echo "build exit: $?"
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server config_parity_round_trip > .claude/PRPs/debug/phase-v1-AD-a-e2e-run.log 2>&1"
echo "run exit: $?"
```
**EXPECT**: 0 on both. The `config_parity_round_trip` test applies all migrations (including the 4 new ones), then walks `SEEDED_KEYS_WITH_CONSTS` (`EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD` = 34 + 27 = 61 entries post-reconciliation; exact count is parametric) calling the typed accessors. Any missing seed row, missing const, or type mismatch fails here.

### Level 5: clippy (per plan-drift CI)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full -- -D warnings --no-deps > .claude/PRPs/debug/phase-v1-AD-a-clippy.log 2>&1"
echo "exit: $?"
```
**EXPECT**: 0. Per memory `feedback_clippy_vs_check_wrapper.md` — must use `cargo-clippy.bat`, not `cargo-check.bat`. `--no-deps` avoids inherited lint debt per `.claude/rules/pre-phase-harness-audit.md` DoD-footgun #3.

### Level 6: PM-hook integrity (belt-and-braces)

```bash
for h in \
  local_private_message_before_create \
  local_private_message_after_create \
  local_private_message_before_update \
  local_private_message_after_update \
  federated_private_message_before_receive \
  federated_private_message_after_receive; do
  rg -q "\"$h\"" crates/ || { echo "missing hook literal: $h"; exit 1; }
done
echo "all PM hooks present"
```
**EXPECT**: prints "all PM hooks present". v1-AD-a doesn't touch PM code but the per-phase guard is mandatory per `.claude/rules/pm-plugin-hooks-stable.md`.

---

## 16. Acceptance criteria

- [ ] All 9 tasks complete and committed on `phase-v1-AD-a`
- [ ] Branch is exactly 9 commits ahead of `governance-v0` (one per task; task 0 produces no commit)
- [ ] Levels 1-5 exit 0; level 6 prints all-hooks-present
- [ ] `SEEDED_KEYS_WITH_CONSTS.len() == EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD` (parametric, not a hard-coded 62 — the implementer reconciled the count at task 7's pre-commit gate per advisor edit #1)
- [ ] `CONFIG_KEY_METADATA.len() == SEEDED_KEYS_WITH_CONSTS.len()` (every seeded key has metadata)
- [ ] `EXPECTED_SEED_COUNT == 34` (unchanged, v0 invariant preserved)
- [ ] `EXPECTED_SEED_COUNT_V1_AD` matches the actual number of `v1-AD-a`-added tuples in `SEEDED_KEYS_WITH_CONSTS` AND the actual number of `ON CONFLICT DO NOTHING` INSERT rows in task 7's `up.sql` — this is the advisor edit #1 reconciliation invariant
- [ ] `rule_set.active_version_id` is NOT seeded, NOT in `SEEDED_KEYS_WITH_CONSTS`, NOT in `CONFIG_KEY_METADATA`, NOT in `const_default_int` (per advisor edit #2 — absence-of-row IS the "no active version" signal)
- [ ] `ENTRY_KIND_ADMIN_CONFIG_CHANGED == "admin_config_changed"` — byte-identical to shell-script emission at `scripts/brehon/admin-config-write.sh:148`
- [ ] `ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED == "admin_config_change_denied"`
- [ ] `.claude/rules/governance-log-entry-kind-registry.md` exists and lists 21 populated kinds (19 v0 + 2 v1-AD) + Phase 6 pending section + 4 reserved sections for future v1 PRDs (7 sections + acceptance = ≥8 `## ` headings)
- [ ] Registry file section structure matches GH #41 "Proposed deliverable" enumeration byte-for-byte (advisor edit #3)
- [ ] GH #41 closeable with "landed at `.claude/rules/governance-log-entry-kind-registry.md` in v1-AD-a at `<commit>`" comment
- [ ] OQ-V1-AD-01/02/03 opened in 99-decisions-and-open-questions.md (leans documented, no resolution)
- [ ] Changelog entry appended for 2026-04-21
- [ ] No contradictions with ADRs 1-15 — especially ADR-010 (staged releases), ADR-013 (EmergencyRemove match exhaustiveness), ADR-015 (pseudonym write-path untouched)
- [ ] No ActivityPub changes (no `crates/apub/` touched)
- [ ] No new dependencies added to any `Cargo.toml`
- [ ] Retro written at `.claude/PRPs/reports/phase-v1-AD-a-retro.md` BEFORE PR merge (per `feedback_retro_not_report.md`)
- [ ] PR opened via `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-v1-AD-a` per `.claude/rules/gh-pr-fork-target.md`
- [ ] PR merged with `--merge` (not `--squash`) — preserves task-per-commit history for CodeRabbit review per advisor close-out directive

---

## 17. Completion checklist

- [ ] Task 0 PRE-FLIGHT — branch verified, wrapper probes pass
- [ ] Task 1 — `add_rule_set_versions` migration
- [ ] Task 2 — `add_sponsor_allowlist` migration
- [ ] Task 3 — `add_case_applied_config_snapshot` migration
- [ ] Task 4 — schema.rs hand-edit
- [ ] Task 5 — Diesel models + newtypes + mod.rs
- [ ] Task 6 — `ConfigKeyMetadata` registry + 27 consts + parity test extension
- [ ] Task 7 — seed migration + 2 ENTRY_KIND consts
- [ ] Task 8 — entry-kind registry file
- [ ] Task 9 — three new OQs opened
- [ ] Acceptance criteria (§16) pass
- [ ] PR #TBD open against `governance-v0` with CodeRabbit auto-review triggered
- [ ] GH issue #41 closed with a comment pointing to `.claude/rules/governance-log-entry-kind-registry.md`

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `ConfigKeyMetadata` struct churn as v1-AD-b/c/d add use sites | MED | LOW | Ship in v1-AD-a with a conservative field set (10 fields listed above). Adding a field later is `&'static` array churn but parity-test-covered. Avoid `Vec<…>` / heap types — kill the temptation to make it runtime. |
| Seed-row count drift — PRD §5.2 "~28" vs plan §4.1 "27" vs actual task-6 tuples vs task-7 INSERT rows | MED | MED | **Advisor edit #1 mitigation (task 7 pre-commit reconciliation gate):** before committing task 7's migration, the implementer runs the two grep-count invariants + the parametric `config_parity_round_trip` test against a fresh container. Three reconciliation outcomes documented (add rows / remove rows / DQ). This converts drift from "end-of-plan surprise at task 7 validation" into an explicit task-7 pre-commit step — task 5/6 drift becomes detectable before any migration SQL commits. Note: plan §4.1 states 27 as the current best count (28-in-PRD minus `rule_set.active_version_id`); the parametric test accepts any value the implementer reconciles to. |
| Schema.rs hand-edit conflict on next upstream rebase | MED | MED | The fork already maintains hand-added governance tables in schema.rs; convention is documented at schema.rs:426-428. Weekly rebase catches any upstream auto-regen that would overwrite — triggers `/prp-debug` per CLAUDE.md. |
| `rule_set.active_version_id` sentinel confusion (if impl forgets advisor edit #2 and adds a `-1` default) | LOW | MED | §4.1 and task 6 GOTCHA both explicitly forbid the sentinel. Parity-test symmetry (metadata ↔ seed ↔ const) makes the omission verifiable: the test passes only when the key is absent from all three compile-time tables. If an implementer adds it anyway "for completeness," clippy-dead-code will complain about the unused const; the DQ escape hatch is documented. |
| v1-AD-b reader crashes on `rule_set.active_version_id` read before v1-AD-c seeds it | LOW | MED | v1-AD-b plan (written later) must introduce `config::get_int_opt` that treats "no row + no const" as `None`, not an error. Documented as load-bearing constraint in v1-AD-a §4.1 and in the v1-AD-b stub at §20. Not a v1-AD-a code smell; flagged here so the v1-AD-b planner doesn't write a caller that uses the error-returning `get_int` for this key. |
| Parity test mis-counts under `--workspace --features full` because `full` enables a derive that requires different types | LOW | LOW | Task 6 GOTCHA includes `--features full` validation. If the derive fails, the whole workspace fails to compile — loud, not silent. |
| v1-AD-a ships before advisor resolves OQ-V1-AD-01/02/03 | LOW | LOW | v1-AD-a doesn't consume any of the three OQ resolutions. They block v1-AD-b/d/e only. v1-AD-a ships, OQs sit open until next sub-phase plans. |
| `admin_config_changed` payload-shape drift between shell script + future Rust handler breaks NOT5 gate 3 | LOW | MED | v1-AD-b is the owner of this gate, not v1-AD-a. v1-AD-a only adds the const. v1-AD-b plan will include a byte-level payload-equivalence test per NOT5 §8.4 condition 3. |
| Issue #41 coordination with jury-mechanics-v1 / reputation-tuning-v1 / etc. plans that land later | LOW | MED | The registry file lives under `.claude/rules/` which auto-loads in `-p` mode. Every subsequent v1 plan's impl agent reads it at iteration 1. Each sub-PRD appends its own section on `/prp-plan` run; collision detection is `rg "^pub const ENTRY_KIND_"` count + literal-uniqueness. |
| Wrapper script bug re-emerges (exit-code masking, flag discard) | LOW | HIGH | Task 0 runs the pre-phase harness audit's four probes including the negative exit-code-propagation probe. Failure blocks task 1. |

---

## 19. Notes

- v1-AD-a is **the lightest of the five sub-phases**. It lands schema + metadata + registry foundation; no handlers, no DTOs, no routes. This is deliberate — it de-risks the heavier sub-phases (v1-AD-b's 8-9 tasks + dry-run + audit endpoint) by giving them a stable substrate to land on.
- The `EXPECTED_SEED_COUNT_V1_AD` pattern (parametric per-sub-PRD count) means jury-mechanics-v1, reputation-tuning-v1, sponsor-liability-v1, and federation-inbound-v1 each add their own const when they plan. Only the aggregate `SEEDED_KEYS_WITH_CONSTS.len()` drifts; each sub-PRD's contribution stays pinned. This is advisor directive #4's "parametric" instruction materialised.
- If the task 7 INSERT row count drifts from the task 6 `SEEDED_KEYS_WITH_CONSTS` extension, `config_parity_round_trip` fails loudly at level 4. The parity loop is the single source of truth; PRD §5.2 and §11 counts are descriptive, not normative.
- Issue #41's entry-kind registry was originally a standalone deliverable scheduled for AFTER Phase 6 merge BEFORE first v1 PRD's `/prp-plan` run. Advisor directive 2026-04-19 reassigns it to v1-AD-a's task 8 — fewer moving parts, one less cross-branch coordination.

---

## 20. Sub-phase stubs (v1-AD-b/c/d/e TOC)

These are planning scaffolds only. Each requires its own `/prp-plan` run against the then-current `governance-v0` HEAD (v1-AD-a merged). Do NOT implement from these stubs.

### v1-AD-b — Write endpoint + audit + dry-run

**Deliverables:**
- `POST /api/v4/governance/admin/config` single-key write handler (`admin_set_config`)
- `GET /api/v4/governance/admin/config` full-read + single-key-read handler (`admin_get_config`)
- `GET /api/v4/governance/admin/config/audit` paginated audit handler (`admin_get_config_audit`)
- New `config::get_int_opt` / `get_float_opt` / `get_bool_opt` / `get_text_opt` accessors — return `Ok(None)` when both DB row and const default are absent, instead of `Err` or `Ok(Some(fallback))`. **Required for `rule_set.active_version_id` path** (absence-of-row signal per advisor edit #2) AND any future key that follows the same convention.
  
  **GOTCHA for v1-AD-b planner (advisor note 1):** when v1-AD-b writes `config::get_int_opt`, the no-row-no-const path MUST return `Ok(None)`, not `Ok(Some(0))`, not `Ok(Some(-1))`, not any other fallback. If the impl agent writes `get_int` (the erroring accessor) and relies on a const fallback, they re-introduce the sentinel advisor edit #2 removed. Required test: `parity::rule_set_active_version_absent_returns_none` must assert `None` pre-v1-AD-c seeding (harness applies all migrations, does NOT seed `rule_set.active_version_id`, calls `get_int_opt(Scope::Instance, "rule_set.active_version_id")`, asserts `Ok(None)`). v1-AD-b plan DoD must include this test.
- Capability checks (is_admin for instance scope, CommunityModeratorView for community scope)
- Dry-run impact computation **outside** tx (per OQ-V1-AD-03 lean)
- Per-key type validation via `CONFIG_KEY_METADATA` lookup
- Payload byte-identity verification against shell script (NOT5 gate 3)

**Blocked by:** v1-AD-a merged; OQ-V1-AD-03 resolved.

**Task count estimate:** 8-9 (1 extra for `get_*_opt` accessor family).

### v1-AD-c — Rule-set versioning routes + wire-up

**Deliverables:**
- `GET /api/v4/governance/admin/rule-sets` list handler
- `POST /api/v4/governance/admin/rule-sets` create handler (appends version)
- `submit_jury_vote` edit: pin `moderation_case.rule_set_version_id = current_active_version` at decision time
- `admin_assign_jury` edit: populate `moderation_case.applied_config_snapshot` at panel-assembly time
- `rule_set.active_version_id = -1 → None` reader conversion

**Blocked by:** v1-AD-a merged.

**Task count estimate:** 4-5.

### v1-AD-d — Dashboard aggregate + SSE audit stream

**Deliverables:**
- `GET /api/v4/governance/admin/dashboard` aggregate handler (active cases / jury queue / recent config changes / federation status / reputation health)
- `GET /api/v4/governance/admin/audit/stream` SSE handler bridging Postgres `LISTEN governance_events` to HTTP
- SSE implementation per OQ-V1-AD-02 resolution (lean: hand-rolled via `async-stream`)

**Blocked by:** v1-AD-a merged; OQ-V1-AD-02 resolved.

**Task count estimate:** 3-4.

### v1-AD-e — Askama HTML pages (DEFERRED)

**Status:** Likely deferred to v1.x / v2 per OQ-V1-AD-01 lean. If user opts to ship: dashboard template, config editor template, audit log template, rule-set manager template.

**Blocked by:** v1-AD-a, b, c, d merged; OQ-V1-AD-01 resolved as "ship, not defer".

**Task count estimate:** 5-7 (if shipped).

---

**END OF v1-AD-a PLAN**
