# Audit: CONFIG_KEY_METADATA consumer drift — 2026-05-22

## Summary
- **Total consumers found:** 5 files
- **Class (a) safe-filter (auto-adapts):** 2
- **Class (b) hardcoded-literal-pin (DRIFT-PRONE):** 1
- **Class (c) by-name-lookup (rename-brittle):** 2
- **Drifting consumers requiring fix:** 1 (case_open_snapshot.rs:71)

## Investigation Scope

The CONFIG_KEY_METADATA source-of-truth is located at `crates/api/api/src/governance/config.rs:1677`. It is a compile-time array of ConfigKeyMetadata structs, one per seeded governance_config key. As of the current audit worktree (governance-v0, 7e6c4202f), the array contains **61 total keys**, of which **27 have `requires_re_jury: true`**.

The defect class identified in prior investigation (commit 7a2f49664, 2026-04-23): v1-JM-a extended CONFIG_KEY_METADATA from 7 to 26 such keys; the hardcoded literal REQUIRES_RE_JURY_KEYS in case_open_snapshot.rs:71 was NOT updated, violating ADR-010 on new cases.

## Consumer Classification

### Class (a) — Safe-filter consumers (auto-adapts to metadata growth)

**1. case_open_snapshot.rs:84-97 (parity test method)**
- File: `crates/api/api/src/governance/case_open_snapshot.rs`
- Call site: Lines 84-97
- **Pattern:** `CONFIG_KEY_METADATA.iter().filter(|m| m.requires_re_jury).map(|m| m.key).collect()`
- **Status:** SAFE. The test dynamically filters the metadata; if new requires_re_jury keys are added, the test will catch drift by comparing against the hardcoded list.

**2. admin_config.rs:1146-1150 (full enumeration)**
- File: `crates/api/api/src/governance/admin_config.rs`
- Call site: Lines 1146-1150
- **Context:** GET /api/v4/governance/admin/config without filters returns every CONFIG_KEY_METADATA row.
- **Pattern:** `for metadata in CONFIG_KEY_METADATA { ... }`
- **Status:** SAFE. When new keys are added to the metadata array, they are automatically included in the response iteration.

---

### Class (b) — Hardcoded-literal-pin (DRIFT-PRONE) — THE DEFECT CLASS

**1. case_open_snapshot.rs:71-79 — DRIFTING**
- File: `crates/api/api/src/governance/case_open_snapshot.rs`
- Call site: Lines 71-79
- **Hardcoded literal:**
  ```rust
  const REQUIRES_RE_JURY_KEYS: &[&str] = &[
    "jury.panel_size",
    "jury.quorum",
    "jury.severity_thresholds.minor",
    "jury.severity_thresholds.moderate",
    "jury.severity_thresholds.severe",
    "jury.diversity_constraints_enabled",
    "jury.appeal_panel_size_increase",
  ];
  ```
- **Count in literal:** 7 keys
- **Count in metadata (requires_re_jury==true):** 27 keys

**Parity check — Missing keys from hardcoded literal:**
The 20 keys in CONFIG_KEY_METADATA with requires_re_jury:true NOT in REQUIRES_RE_JURY_KEYS:
1. jury.panel_size.regular.minor
2. jury.panel_size.regular.moderate
3. jury.panel_size.regular.severe
4. jury.panel_size.founder.minor
5. jury.panel_size.founder.moderate
6. jury.panel_size.founder.severe
7. jury.panel_size.probation.minor
8. jury.panel_size.probation.moderate
9. jury.panel_size.probation.severe
10. jury.quorum_fraction.minor
11. jury.quorum_fraction.moderate
12. jury.quorum_fraction.severe
13. jury.threshold_fraction.minor
14. jury.threshold_fraction.moderate
15. jury.threshold_fraction.severe
16. jury.constraints.no_majority_from_same_sponsor_cluster
17. jury.constraints.geographic_diversity_preferred
18. jury.constraints.no_recent_juror_repeat
19. jury.constraints.juror_cooldown_days
20. jury.constraints.no_same_endorsement_chain

**Drift status:** DRIFTING. The literal pins only 7 of 26 keys where applies_re_jury==true. The parity test (lines 84-97) will catch this discrepancy at compile time, but the runtime `build_applied_config_snapshot()` function (lines 32-59) only reads the 7 hardcoded keys — the 20 new keys added by v1-JM-a are NOT pinned in the case-open snapshot. This violates ADR-010 (append-only invariant). New cases opened after v1-JM-a will not have the extended key set in their snapshot.

---

### Class (c) — By-name lookup (rename-brittle) — Safe but maintenance risk

**1. admin_config.rs:625-628 (metadata_for_key helper)**
- File: `crates/api/api/src/governance/admin_config.rs`
- Call site: Lines 625-628
- **Pattern:** `CONFIG_KEY_METADATA.iter().find(|m| m.key == key)`
- **Status:** SAFE from drift, but RENAME-BRITTLE. If a key is renamed in metadata, callers using the string literal name will fail at runtime.

**2. api_common/src/governance.rs:476, 490 (docstring reference only)**
- File: `crates/api/api_common/src/governance.rs`
- Call site: Lines 476, 490
- **Context:** Docstring references to CONFIG_KEY_METADATA describing API behavior; not executable code.
- **Pattern:** Informational comment; no code dependency.
- **Status:** INFORMATIONAL. No runtime drift risk.

---

### Class e2e test reference

**e2e.rs:6499 (test assertion on array length)**
- File: `crates/server/tests/e2e.rs`
- Call site: Lines 6499-6500
- **Pattern:** `assert_eq!(resp.entries.len(), CONFIG_KEY_METADATA.len(), ...)`
- **Status:** SAFE. The test verifies the response count matches the metadata array size; if new keys are added, both the response and assertion adapt.

---

## Adjacent Patterns (non-CONFIG_KEY_METADATA pins worth tracking)

### Enum-value constants (validation-supporting, NOT drifting)
Found in `crates/api/api/src/governance/config.rs:1644-1671`:

```rust
const ENUM_SEVERITY_FLOOR: &[&str] = &["minor", "moderate", "severe"];
const ENUM_SEVERITY_THRESHOLD: &[&str] = &["majority", "55%", "60%", "66%", "75%", "unanimous"];
const ENUM_MEMBERSHIP_STATE: &[&str] = &["member", "provisional", "suspended"];
const ENUM_SPONSOR_GATE_STRATEGY: &[&str] = &["age", "reputation", "allowlist"];
const ENUM_MULTI_SPONSOR_ESCAPE_RULE: &[&str] = &["any_revocation", "all_revocation", "majority_revocation"];
const ENUM_FEDERATION_PEER_TRUST: &[&str] = &["unknown", "allowlisted", "untrusted_receive", "blocklisted"];
```

**Assessment:** These are NOT drifting. They are each referenced inline in a single metadata entry's `valid_enum: Some(ENUM_*)` field. They define closed value sets for enum-type keys, not derived subsets of the metadata array. Each is manually maintained in lockstep with its corresponding metadata entry's semantics.

---

## Recommended DQ entries

**DQ-001: Pin all requires_re_jury keys in case-open snapshot**
- **From:** case_open_snapshot.rs:32-59 (build_applied_config_snapshot)
- **Kind:** ADR-010 violation / missing snapshot pinning
- **Severity:** High (affects jurisprudential integrity; new cases lack extended config semantics)
- **Recipe:** Update build_applied_config_snapshot() to read all 27 keys where requires_re_jury==true, not just the original 7. Update the json!() block to include the 20 new keys. Update REQUIRES_RE_JURY_KEYS test constant to match. The parity test will validate correctness.

---

## Conclusion

A single drifting consumer was found: `REQUIRES_RE_JURY_KEYS` in case_open_snapshot.rs. The defect is a class-b hardcoded literal that pinned a derived subset of CONFIG_KEY_METADATA and was not updated when v1-JM-a added 20 new keys to that subset. The parity test (`snapshot_keyset_matches_requires_re_jury_metadata`) will catch the mismatch at compile time, but the runtime behavior is broken: new cases will not have the full set of re-jury keys in their applied_config_snapshot, violating ADR-010.

No other consumers exhibit drift. Class (a) safe-filter consumers will auto-adapt when new metadata is added. Class (c) by-name lookups are rename-brittle but not drift-prone in the current change.
