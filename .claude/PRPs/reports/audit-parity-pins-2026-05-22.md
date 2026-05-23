# Audit: test-only parity-pin / drift-guard patterns — 2026-05-22

## Summary
- **Total parity-pins found:** 2 primary patterns + 1 meta-audit class
- **In-sync:** 0 (one has drifted significantly)
- **Drifting (test would fail):** 1 critical
- **Unverifiable (no test exists):** 1 (the EXPECTED_SEED_COUNT composite consts)
- **Audit scope:** crates/*/src/**, crates/*/tests/**, crates/server/tests/e2e.rs, test_errors_used.rs
- **Workspace:** Full governance, db_schema, utils modules

## Findings

### 1. CRITICAL: \REQUIRES_RE_JURY_KEYS\ drift

**File:** \crates/api/api/src/governance/case_open_snapshot.rs:71-79\  
**Test:** \snapshot_keyset_matches_requires_re_jury_metadata()\ at line 84

**Current const (7 keys):**
- jury.panel_size
- jury.quorum
- jury.severity_thresholds.minor
- jury.severity_thresholds.moderate
- jury.severity_thresholds.severe
- jury.diversity_constraints_enabled
- jury.appeal_panel_size_increase

**Source-of-truth:** \CONFIG_KEY_METADATA\ in \crates/api/api/src/governance/config.rs:1677\ (139 entries total)

**Drift status:** **DRIFTING** — Test compares against metadata entries where \equires_re_jury == true\. There are **27 keys** in the metadata marked \equires_re_jury: true\, but the const only lists **7**. The test will FAIL when run.

**Missing 20 keys:** jury.constraints.* (5 keys), jury.panel_size.*.{minor,moderate,severe} (9 keys), jury.quorum_fraction.* (3 keys), jury.threshold_fraction.* (3 keys)

**Root cause:** The runtime helper \uild_applied_config_snapshot()\ returns only 7 hardcoded keys in its JSON object. The parity const was not updated when the metadata added 20 new \equires_re_jury: true\ keys in later phases.

---

### 2. MEDIUM: \EXPECTED_SEED_COUNT_V1_*\ family (composite validation)

**File:** \crates/api/api/src/governance/config.rs:1586-1642\  
**Test:** \seeded_keys_count_matches_const_count()\ at line 3368

**Constants:** EXPECTED_SEED_COUNT=34, V1_AD=27, V1_JM=27, V1_SL=13, V1_RT_NETNEW=26, V1_RT_LOGICAL=29, V1_FED_IN=11

**Source-of-truth:** \SEEDED_KEYS_WITH_CONSTS\ (same file) — tuple array of (key_name, const_name, value_type)

**Drift status:** **UNVERIFIABLE** — The test validates total count (sum must equal SEEDED_KEYS_WITH_CONSTS.len()) but does NOT verify individual phase-level counts correspond to actual keys seeded per phase. If a new v1-RT key is added without incrementing EXPECTED_SEED_COUNT_V1_RT_NETNEW, the total would pass but phase-level audit trail would be silent.

---

### 3. LOWER: \	est_errors_used.rs\ meta-audit

**File:** \crates/utils/tests/test_errors_used.rs:5-44\  
**Pattern:** Dynamic enum iteration via strum::IntoEnumIterator, not a manual parity-pin.

**Drift status:** **IN-SYNC** — Test dynamically enumerates the source-of-truth, so no manual parity-pins exist here.

---

## High-risk patterns deserving a new parity test

1. **\uild_applied_config_snapshot()\ vs. REQUIRES_RE_JURY_KEYS** — The helper hardcodes 7 keys in its runtime JSON. If a new \equires_re_jury\ key is added to metadata, the snapshot helper MUST be updated in tandem. Missing integration test: exercise the snapshot helper and verify its JSON keys match REQUIRES_RE_JURY_KEYS.

2. **Phase-level seed counts vs. actual seeded keys** — No test verifies that a new key seeded in Phase 2 increments the Phase 2 count. Missing test: per-phase breakdown that counts SEEDED_KEYS_WITH_CONSTS entries and compares against the corresponding EXPECTED_* const.

---

## Recommended DQ entries

1. **High:** Fix REQUIRES_RE_JURY_KEYS (case_open_snapshot.rs:71-79). Update const to match all 27 keys from CONFIG_KEY_METADATA where requires_re_jury==true, AND update build_applied_config_snapshot() to return all 27 keys or document why only 7 are returned.

2. **Medium:** Add integration test in e2e.rs that calls build_applied_config_snapshot() and validates returned JSON contains exactly the keys from filtered metadata.

3. **Medium:** Add phase-level breakdown test in config.rs that groups SEEDED_KEYS_WITH_CONSTS by phase and asserts each group's count matches its EXPECTED_* const.

4. **Low:** Document intent of EXPECTED_SEED_COUNT consts in docstrings (audit vs. release notes vs. coverage tracking).

---

## Audit Methodology

- **Workspace:** C:/Users/barri/Developer/brehon-fork-audit-2026-05-22 (detached HEAD at governance-v0 tip)
- **Search patterns:** const [A-Z_]+_(KEYS|LIST|NAMES|FIELDS|VARIANTS|COLUMNS):, test assertions comparing hardcoded arrays
- **Files examined:** case_open_snapshot.rs, config.rs (3479 lines, 139 metadata entries), admin_emergency_remove.rs, actor_pseudonym_helper.rs, test_errors_used.rs
- **Total constants:** 8 (1 string slice, 7 numeric EXPECTED_* counts)
- **Test functions covering parity:** 4

**Report generated:** 2026-05-22
