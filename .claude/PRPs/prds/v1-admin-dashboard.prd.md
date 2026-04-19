# v1 Admin Dashboard — Sub-PRD

**Audience:** Backend lead + design (no designer assigned), pilot operator
**Status:** Draft (v1 design phase, 2026-04-19)
**Theme:** Community sovereignty via configurable governance — every knob has an instance default and a per-community override
**Owner:** TBD (backend lead) — keystone v1 deliverable
**Predecessor v0 surfaces:** `governance_config` table (Phase 5a task 50), `admin-config-write.sh` operator script (Phase 5c task 70), three v0 admin endpoints (`/admin/assign-jury`, `/admin/close-case`, `/admin/reputation-stats`)
**Resolves / consumes:** OQ-018 (admin HTTP config-write — primary), OQ-002, OQ-005 (partial — operator UX only), OQ-019, OQ-020, OQ-025, OQ-026 (config schema decision)
**Out of scope:** OQ-009 juror anonymity UX (depends on OQ-005), OQ-010 production signing keys (v2), OQ-011 first-target community (orthogonal), full React frontend (v2/v3 per ADR-010)

---

## 1. Vision & Goals

> **The admin dashboard exists so a community can change how it governs itself without re-deploying the binary.**

Per [01 §3 Core principle](../../../docs/brehon-law-inspired-network/01-vision-and-principles.md), justice is enforced by social trust and mutual obligation. That principle lives in policy parameters (jury size, sponsor liability multipliers, founder seed caps, gate strategies) — and policy must be **discoverable, versioned, auditable, and scoped**. v0's "edit `governance_config` over psql" is operationally sufficient for one solo operator on one instance; the moment a second pilot community asks "can our jury be 7 instead of 5?", the answer must be a JSON write, not a code change.

### 1.1 Goals

| # | Goal | How this PRD achieves it |
|---|---|---|
| G1 | Every governance knob has a default + an override path | §3 schema + §5 defaults table |
| G2 | Every config change is auditable and signed | §4 single-key write writes one `governance_log` entry per change via existing `governance_log::append` |
| G3 | Per-community sovereignty without breaking instance-wide invariants | §3 merge precedence: community → instance → const |
| G4 | Operators can preview impact before committing | §4 `dry_run` query parameter on every write |
| G5 | In-flight juries are not retroactively invalidated by config edits | §3 `requires_re_jury` flag + §4 `apply_at` semantics |
| G6 | Backend-first, minimal pages — Lemmy-UI plugin / React frontend deferred | §6 server-rendered admin pages (askama or maud); v1.5/v2 React pass |

### 1.2 Non-goals (v1)

- Per-user preference editing (Lemmy-native settings cover this)
- Moderation case management UI (separate v1 PRD: `v1-jury-mechanics.prd.md`)
- Federation peer trust state UI (separate v1 PRD: `v1-federation-inbound.prd.md` if extracted)
- Full React/SPA dashboard (v1.5/v2 frontend pass)
- Rule-text editor with WYSIWYG (v1 ships JSON + plaintext upload only — see §3.6 `rule_set.*`)
- Step-up auth implementation (v2 per ADR-010 / `06 §2.1`); this PRD reserves the slot via `requires_step_up` config metadata
- Multi-tenant federations on one instance (deferred — see §9)

---

## 2. Scope

| In scope | Out of scope |
|---|---|
| Instance-config + per-community-config read/write HTTP API | Per-user preferences (Lemmy-native) |
| Founder/sponsor settings, jury parameters, reputation tuning, federation policy, threshold knobs | Moderation case adjudication UI |
| Rule-set version management (immutable versions, append history) | Federation operator UX (peer add/remove flows) |
| Audit log surface for config changes | Step-up auth implementation (v2) |
| Minimal server-rendered admin pages (Rust templating) | Production OPA/OpenFGA migration (v2) |
| Capability-gated endpoints (instance_admin, community_admin) | Rich React frontend (v1.5/v2) |
| Dry-run preview, `apply_at` immediate vs. next-jury-cycle semantics | Bulk import/export (open question §9) |

---

## 3. Configuration Schema (JSON-shaped)

### 3.1 Hierarchical namespaces (v1 inventory)

The v0 seed list (34 keys, `crates/api/api/src/governance/config.rs:463 EXPECTED_SEED_COUNT`) is the starting inventory. v1 adds keys for rule-set versioning, federation policy, appeal windows, sponsor liability grace windows (OQ-025), participation_consistency sources (OQ-019), per-dimension decay/bounds (reputation-tuning-v1), and federation-peer trust controls (federation-inbound-v1). Final v1 inventory — authoritative across all five v1 sub-PRDs — is **~138 keys across 15 namespaces**. Per-PRD contribution breakdown:

| Sub-PRD | Net key additions | Primary namespaces touched |
|---|---|---|
| admin-dashboard-v1 (this PRD) | ~24 | `jury.*`, `liability.*`, `report.*`, `onboarding.*`, `founder.*`, `participation.*`, `federation.*`, `rule_set.*` |
| jury-mechanics-v1 | ~18 | `jury.*` (incl. 9-cell `panel_size.<status>.<severity>` matrix per B2 cascade), `appeal.*` (5 keys — jury-mechanics owns per B1) |
| reputation-tuning-v1 | 28 | `decay.*` (8 per-dim/per-dir), `bounds.*` (8 floor/ceiling), `deltas.*` (5), `participation.*` (3 context knobs), `job.*` (3 cadence + rollup), `feature.*` (1 v1-decay flag) |
| sponsor-liability-v1 | 10 | `liability.*` (flat per B4 — grace_window_\*_hours x6, restoration/multi-sponsor x3, revoke rate-limit x1) |
| federation-inbound-v1 | ~8 | `federation.*` (peer-trust knobs, advisory-only toggle, TTL, quarantine-severity floor) |
| **TOTAL** | **~88 new v1 keys** (+ 34 v0 keys ≈ 122–138 depending on subtle overlaps) | across 15 namespaces |

Final 15-namespace registry (this section authoritative; individual-PRD matrices are the sources of truth for their own rows):

| Namespace | v0 keys | v1 additions | v1 owner-PRDs | Purpose |
|---|---|---|---|---|
| `thresholds.*` | 3 | 0 | admin-dashboard (no changes) | Capability cutoffs (jury_reliability, reporting_accuracy, endorsement_strength) |
| `jury.*` | 5 | 4 (admin-dashboard: severity-thresholds, diversity, deadline window, + jury-mechanics cascade matrix keys per B2) | admin-dashboard, jury-mechanics | Jury size, quorum, gating, fallback. Bare `jury.panel_size = 7` kept as final fallback per B2 cascade `jury.panel_size.<status>.<severity>` → `jury.panel_size.<severity>` → `jury.panel_size` → const. |
| `deltas.*` | 9 | +5 (reputation-tuning: participation_weekly_active, participation_dormant, participation_juror_aligned, evidence_cited, evidence_bad_faith) | admin-dashboard, reputation-tuning | Per-event reputation deltas (policy knobs) |
| `liability.*` | 3 | +10 (flat per B4: grace_window_{minor,moderate,severe,minimum,maximum,alert_threshold}_hours x6, restoration_escapes_liability, restoration_severity_reduction_steps, multi_sponsor_escape_rule, revoke_rate_limit_per_day — all sponsor-liability-owned) | admin-dashboard, sponsor-liability | Sponsor liability tuning — flat namespace per v0 precedent |
| `report.*` | 5 | 1 (`weighted_report_clamp_ceiling_review_v1` per OQ-006 footnote) | admin-dashboard | Threshold formula |
| `decay.*` | 1 | +7 (reputation-tuning: 8 per-dim × per-dir half-lives, of which 1 mirrors v0 default so net +7 new keys) | admin-dashboard, reputation-tuning | Per-dimension + per-direction reputation decay tuning |
| `bounds.*` | 0 | 8 (reputation-tuning: 4 dims × floor+ceiling snapshot clamps) | reputation-tuning | Soft bounds on snapshot dimension sums — distinct from `thresholds.*` (capability cutoffs) |
| `onboarding.*` | 3 | 4 (admin-dashboard: `sponsor_gate_strategy` enum widened per OQ-020; `sponsor_min_endorsement_strength` for `'reputation'` strategy; `sponsor_allowlist_table_name`; `provisional_membership_cooldown_days`) | admin-dashboard, reputation-tuning (gate strategies) | Onboarding policy |
| `founder.*` | 3 | 1 (`founder_seal_visible_in_profile` bool per OQ-017) | admin-dashboard | Founder seeding |
| `job.*` | 2 | +3 (reputation-tuning: participation_interval_days, rollup_interval_days, rollup_equal_weights per B3 namespace collapse) | admin-dashboard, reputation-tuning | Background job cadence + batch-config. `job.*` is the single authoritative cadence namespace — no parallel `cron.*` namespace per B3. |
| `participation.*` | 0 | +5 (3 reputation-tuning context knobs: activity_threshold_comments, lookback_days, dormancy_window_days; + 2 admin-dashboard OQ-019 knobs. `weekly_active_delta` and `dormant_delta` moved to `deltas.*` per B3; `attestation_enabled` remains here.) | admin-dashboard, reputation-tuning | Participation-context knobs (policy-adjacent; the reputation-delta values live under `deltas.*`) |
| `feature.*` | 0 | 1 (`feature.reputation_v1_decay_enabled` per B3) | reputation-tuning | Feature-flag namespace. First-class per B3 — future deferred-enforcement toggles extend this. |
| `appeal.*` | 0 | 5 (owned by jury-mechanics-v1 §10: `window_days`, `panel_size_multiplier`, `panel_size_floor_increment`, `threshold_tier_bump`, `auto_select_on_appeal_acceptance`. `original_jurors_excluded` is hard code rule, not a config key — per B1) | jury-mechanics | Appeal mechanics |
| `federation.*` | 0 | 5 (`inbound_advisory_only`, `peer_attestation_ttl_days`, `signature_required`, `quarantine_recommendation_severity_floor`, `outbound_publish_enabled`; federation-inbound-v1 may add further peer-trust knobs per its §8.0/§10) | admin-dashboard, federation-inbound | Federation policy. Peer-trust knobs live here; see `v1-federation-inbound.prd.md` §10. |
| `rule_set.*` | 0 | 4 (`active_version_id`, `auto_carry_in_flight_cases`, `text_max_bytes`, `version_propagation_delay_hours`) | admin-dashboard | Rule-set version pointer |

**Namespace-collapse rationale (B3):** v0 establishes `liability.*` and `job.*` as flat namespaces. A dual `cron.*` + `job.*` pair for the same concept (background scheduler cadence) would be a permanent operator tax. Participation-event tuning keys aren't cron knobs semantically — they are reputation deltas conditioned on cron-batch context, so they belong under `deltas.*` (policy) or `participation.*` (context). `bounds.*` is genuinely new (post-clamp snapshot ceilings — distinct from `thresholds.*` capability cutoffs). `feature.*` is prophylactic for growth (every future deferred-enforcement toggle will need one; starting the namespace now avoids later retrofits).

### 3.2 Per-key metadata (config-key registry)

Each key has compile-time metadata that drives schema-driven UI, validation, and the Watch-1 parity contract (already enforced for v0 keys via `SEEDED_KEYS_WITH_CONSTS`):

```rust
pub struct ConfigKeyMetadata {
  pub key: &'static str,                  // "jury.panel_size"
  pub value_type: ValueType,              // Int | Float | Bool | Text | Enum(&'static [&'static str])
  pub default: ConstDefault,              // mirrors DEFAULT_* const
  pub valid_range: Option<NumericRange>,  // for Int/Float
  pub valid_enum: Option<&'static [&'static str]>, // for Enum
  pub scope: ConfigScope,                 // Instance | Community | Both
  pub requires_re_jury: bool,             // §3.4 — does change affect in-flight juries?
  pub requires_step_up: bool,             // §7 — reserved for v2 step-up auth
  pub apply_at_default: ApplyAt,          // Immediate | NextJuryCycle | NextSnapshotJob
  pub description: &'static str,          // shown in /admin/governance/config form
  pub doc_anchor: &'static str,           // "01#5.6" → links to design doc section
}
```

**v1 convention:** every new key added to `SEEDED_KEYS_WITH_CONSTS` MUST also appear in a parallel `CONFIG_KEY_METADATA` table. A new `parity::every_seeded_key_has_metadata` test enforces it (matches the existing `every_seeded_key_has_const_fallback` test pattern at `crates/api/api/src/governance/config.rs:484`).

**Storage commitment (per NOT4 2026-04-19):** `CONFIG_KEY_METADATA` is a **compile-time `&'static [ConfigKeyMetadata]` array** (and NOT a runtime `config_key_metadata` table). Every new key requires a PR-against-Rust-code edit — no DB migration adds metadata rows. This is deliberate: it matches the existing Watch-1 parity contract (`SEEDED_KEYS_WITH_CONSTS` is already compile-time), enforces review of new knob semantics at PR time rather than runtime-seed time, and removes a class of drift ("DB has key with no const, or const with no metadata"). The trade-off is that a new key cannot be hot-shipped without a code rollout — acceptable because governance knobs are rare additions and the parity test would reject a DB-only addition anyway. The `§6.1` `/admin/governance/config` page reads from the compile-time array (`&CONFIG_KEY_METADATA[..]`), not from a DB query.

### 3.3 Merge precedence (already implemented in v0 reader)

Per `crates/api/api/src/governance/config.rs:269 fetch_value`:

```
1. governance_config row WHERE scope = 'community:<id>' AND key = '<key>'   (most specific)
2. governance_config row WHERE scope = 'instance' AND key = '<key>'
3. Rust DEFAULT_* const                                                      (fallback)
```

v1 dashboard surfaces this provenance in `GET /admin/governance/config` responses — every value carries an `effective_from: "community:42" | "instance" | "default"` field so operators see *why* a value is what it is.

### 3.4 `requires_re_jury` flag (per ADR-010 invariant)

ADR-010 commits that a config edit must not retroactively invalidate in-flight juries. The flag distinguishes:

- **`requires_re_jury = false`** (most keys): change is read at next handler invocation. Examples: `report.case_threshold_micros`, `deltas.juror_aligned`, `onboarding.sponsor_min_account_age_days`.
- **`requires_re_jury = true`** (panel composition keys): change is read only at `Open | ThresholdMet` → `JurySelection` transition. In-flight cases (`JurySelection | InReview`) keep the panel size + thresholds they were seated under, snapshotted onto `moderation_case.applied_config_snapshot` (new column added in v1 migration). Examples: `jury.panel_size`, `jury.severity_thresholds`, `jury.diversity_constraints`.

`apply_at` semantics on the write endpoint (§4) make this explicit: `apply_at = "next_jury_cycle"` for `requires_re_jury` keys is the safe default.

### 3.5 Status-aware rule extension (OQ-026 resolution)

OQ-026 asked: how do we add status-conditional rules (`jury.panel_size.founder = 7` vs `jury.panel_size.regular = 5`) without a schema migration?

**v1 decision: dotted-namespace + reader-side cascade (OQ-026 option (a)).**

- Status-conditional keys use the form `<namespace>.<param>.<status_tier>` where `status_tier ∈ { regular, founder }` (and is extensible to more tiers in v2).
- Reader cascade extends naturally: when handler asks for `jury.panel_size` for a founder-targeted case, the reader tries `jury.panel_size.founder` first, then falls back to `jury.panel_size`.
- No schema migration. No `status_tier` column. Composable with the existing `community:<id>` cascade.
- v1 ships `liability.founder_multiplier` / `liability.regular_multiplier` (already in v0) as the proof-of-pattern; new status-conditional keys land additively.
- v2+ migrates composable rules to OPA only if pattern (a) becomes unwieldy (>3 status-conditional variants per rule).

**Rejected alternatives:** option (b) multiplier keys is too implicit for non-multiplicative rules; option (c) schema column is migration cost we don't need yet.

**Reader cascade (per B2 2026-04-19):** for a status-conditional key like `jury.panel_size`, the reader attempts in order: `jury.panel_size.<status>.<severity>` → `jury.panel_size.<severity>` → `jury.panel_size` (the bare key) → Rust const fallback. Bare keys like `jury.panel_size = 7` remain valid as the v1 root default and preserve v0 read sites (e.g. `admin_assign_jury.rs:120`) unchanged during v1 rollout. See jury-mechanics-v1 §3.4 and §4.3 for the cascade helper and the 9-cell matrix.

### 3.6 Rule-set versioning (OQ-002 resolution)

Per OQ-002, community rule sets need versioning so a case decided under rule-set v3 survives a later v4 publish.

**Schema (new v1 migration `add_rule_set_versions`):**

```sql
CREATE TABLE rule_set_version (
  id           SERIAL PRIMARY KEY,
  community_id INTEGER NOT NULL REFERENCES community(id) ON DELETE CASCADE,
  version      INTEGER NOT NULL,                       -- monotonic per community
  parent_id    INTEGER REFERENCES rule_set_version(id), -- previous version, NULL for v1
  text_sha256  BYTEA NOT NULL,                          -- content hash
  rule_text    TEXT NOT NULL,                           -- full plaintext (markdown)
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by   INTEGER REFERENCES person(id) ON DELETE RESTRICT,
  UNIQUE (community_id, version)
);

-- Pin a case to the rule-set version active at decision time.
ALTER TABLE moderation_case ADD COLUMN rule_set_version_id INTEGER REFERENCES rule_set_version(id);
```

- `rule_set.active_version_id` config key (per-community scope) points to the live version.
- New version creation is **append-only** (no UPDATE on rule_set_version rows ever). The pattern matches `governance_config`'s append-history shape.
- `submit_jury_vote` snapshots `case.rule_set_version_id = current_active_version` at decision time; the public modlog shows the rule text *as it was* when the case was decided.
- Activating a new version is a config write (`POST /api/v4/governance/admin/config { key: "rule_set.active_version_id", scope: "community:42", value: 7 }`), which produces the standard governance_log audit entry.
- v1.x: no in-band rule-text editor; ops upload the rule text via a separate `POST /admin/governance/rule-sets` endpoint (§4) that writes both the rule_set_version row AND the config-pointer flip in one transaction.

---

## 4. HTTP API Surface

All routes mount under `/api/v4/governance/admin/` — already a registered scope (`crates/api/routes/src/lib.rs:531-536`). v1 adds five new routes alongside the v0 admin trio.

### 4.1 Endpoint table

| Method | Path | Purpose | Capability | Step-up (v2) |
|---|---|---|---|---|
| GET | `/api/v4/governance/admin/config` | Read full effective config + provenance | `instance_admin` (full) or `community_admin(id)` (community-scoped subset) | No |
| GET | `/api/v4/governance/admin/config?key=<dotted_key>&community_id=<id>` | Read single key (resolved via cascade) | same as above | No |
| POST | `/api/v4/governance/admin/config` | Single-key write (with `dry_run` + `apply_at` + `community_id`) | `instance_admin` for instance keys; `community_admin(community_id)` for community-scoped keys | Yes for `requires_step_up` keys |
| GET | `/api/v4/governance/admin/config/audit` | Paginated config-change log (filters by key, scope, actor, date range) | `instance_admin` (instance audit) or `community_admin(id)` (community audit) | No |
| GET | `/api/v4/governance/admin/rule-sets` | List rule-set versions for a community | `community_admin(id)` or `instance_admin` | No |
| POST | `/api/v4/governance/admin/rule-sets` | Create new rule-set version (immutable; references parent) | `community_admin(id)` for that community | Yes |
| GET | `/api/v4/governance/admin/dashboard` | Aggregate dashboard data (active cases, jury queue depth, federation status, recent config changes) | `instance_admin` | No |
| GET | `/api/v4/governance/admin/audit/stream` | SSE stream of governance_log entries (live) | `instance_admin` | No |

### 4.2 `POST /admin/config` request shape

```jsonc
{
  "key": "jury.panel_size",          // dotted key from §3.1
  "value_type": "int",                // matches §3.2 metadata
  "value": 7,                         // typed by value_type (i64 | f64 | bool | string | enum-string)
  "scope": "instance",                // "instance" | "community:<id>"
  "apply_at": "next_jury_cycle",      // "immediate" (default) | "next_jury_cycle" | "next_snapshot_job"
  "dry_run": false,                   // if true, no write happens; response includes preview
  "reason": "Pilot retro: bumping to v1-target panel size"
}
```

**Response (success):**

```jsonc
{
  "applied": true,
  "config_id": 12345,                 // governance_config.id of new row
  "governance_log_id": 67890,         // governance_log.id of audit entry
  "preview": {                        // populated even on success — the diff that was applied
    "previous": { "value": 5, "effective_from": "instance" },
    "new":      { "value": 7, "effective_from": "instance" },
    "downstream_impact": {
      "in_flight_cases_grandfathered": 3,    // cases with status ∈ {JurySelection, InReview} whose panel stays at 5
      "users_losing_jury_eligible": null,    // null if not relevant; integer for threshold-related keys
      "estimated_first_effect_at": "next jury seating after now()"
    }
  },
  "applied_at": "2026-04-19T15:32:01Z"
}
```

**Response (`dry_run = true`):** identical shape, `applied: false`, `config_id: null`, `governance_log_id: null`. The `preview.downstream_impact` block is computed by the same Rust function regardless of dry-run, so what you see is what you get.

### 4.3 `dry_run` impact computation

Per OQ-018's lean ("the whole point of config is that admins iterate on values, and dry-run lets them see downstream effects"). Computed for these key categories:

| Key category | Computed impact |
|---|---|
| `thresholds.*` | `users_losing_jury_eligible`, `users_gaining_jury_eligible` (count diff against current `reputation_snapshot`) |
| `jury.panel_size` | `in_flight_cases_grandfathered` (count of cases in `JurySelection|InReview`) |
| `jury.max_concurrent_assignments` | `currently_at_or_above_new_cap` (count of users) |
| `liability.sponsor_liability_floor` | `sponsors_at_floor_today` (count) |
| `report.case_threshold_micros` | `cases_that_would_have_opened_in_last_30d` (using current report data, recomputed against new threshold) |
| `decay.*` | (no impact preview — decay is gradual; just acknowledged) |
| All others | `estimated_first_effect_at` description string |

Impact computation is **a single read-only Diesel query per category**, run inside the same `run_transaction` as the write (with a `SAVEPOINT` rollback when `dry_run = true`).

### 4.4 `apply_at` semantics

| Value | Meaning | Used for |
|---|---|---|
| `immediate` (default) | New config row visible to next handler read; cached values invalidated within request | Most keys |
| `next_jury_cycle` | Row inserted with `valid_from = now() + jury_cycle_window`; alternative: row inserted immediately but flagged so jury-selection logic skips it for 24h | Keys with `requires_re_jury = true` |
| `next_snapshot_job` | Row inserted; `reputation_snapshot` job refreshes against new value at next scheduled run | Decay/threshold keys that require snapshot recomputation to take effect |

**Implementation note:** `next_jury_cycle` is the simpler-than-it-sounds case. `governance_config_current` view already returns the most-recent row by `valid_from DESC`. We extend it to `WHERE valid_from <= now()`. A "scheduled" config edit becomes an INSERT with `valid_from = now() + duration`; the view auto-promotes it at the right time. No background job needed.

### 4.5 Config audit endpoint shape

`GET /admin/config/audit?key=<key>&scope=<scope>&actor=<pseudonym>&since=<iso8601>&page=1&limit=50`

Response: paginated list of `governance_log` rows where `entry_kind = 'admin_config_changed'`, with payload destructured into `key`, `previous_value`, `new_value`, `actor_pseudonym`, `reason`, `dry_run` (if dry-run was logged as advisory — see §4.6), `created_at`. Source query is on the existing `governance_log` table, which already indexes on `entry_kind` (`migrations/2026-04-15-100500-0000_add_governance_log/up.sql:15`).

### 4.6 Dry-run logging policy

Open question §9 candidate, **PRD decision: dry-runs are NOT logged to `governance_log`**. Rationale: governance_log is for events that *changed state*; logging every dry-run preview pollutes the audit trail and burns hash-chain entries. If a v1.5 user wants "I want to know who keeps previewing this key", we add a separate `dry_run_preview_log` table. Documented as a non-decision-deferred-to-future.

### 4.7 Routes wiring

New file: `crates/api/api/src/governance/admin_config.rs` (handler module). Registered alongside existing `/admin` scope:

```rust
// crates/api/routes/src/lib.rs (additions)
.service(
  scope("/admin")
    .route("/assign-jury", post().to(admin_assign_jury))
    .route("/close-case", post().to(admin_close_case))
    .route("/reputation-stats", get().to(admin_reputation_stats))
    // v1 additions:
    .service(
      scope("/config")
        .route("", get().to(admin_get_config))
        .route("", post().to(admin_set_config))
        .route("/audit", get().to(admin_get_config_audit)),
    )
    .service(
      scope("/rule-sets")
        .route("", get().to(admin_list_rule_sets))
        .route("", post().to(admin_create_rule_set)),
    )
    .route("/dashboard", get().to(admin_dashboard))
    .service(
      scope("/audit")
        .route("/stream", get().to(admin_audit_stream)),  // SSE endpoint per IMPLEMENTATION-PLAN-v0.md §6.1
    ),
),
```

---

## 5. Default Values

For every config key, this section proposes the v1 default and tags it (a) high-confidence, (b) needs-pilot-data, (c) decide-later.

**Key:** **(a)** = ship as-is; **(b)** = ship default but flag for pilot-retro review; **(c)** = decide-later via decision-queue or new OQ.

### 5.1 v0 keys — defaults already shipped (32 of 34 high-confidence)

The v0 seed values from `migrations/2026-04-18-000000-0000_add_governance_config/up.sql` are reaffirmed for v1. Each line cites its design-doc source.

| Key | Default | Tag | Rationale |
|---|---|---|---|
| `thresholds.jury_reliability` | 50 | (b) | [01 §5.4] capability gating; review after first community runs ≥10 juries |
| `thresholds.reporting_accuracy` | 50 | (b) | Same as above |
| `thresholds.endorsement_strength` | 25 | (b) | Same as above |
| `jury.panel_size` | **5** v0 → **7** v1 | (a) | [01 §5.6] v1 target; ADR-007 simplification ends. Per B2 (2026-04-19): this remains a valid cascade-fallback key — reader precedence is `jury.panel_size.<status>.<severity>` → `jury.panel_size.<severity>` → `jury.panel_size` → Rust const. See jury-mechanics-v1 §3.4 for the dotted-namespace matrix. |
| `jury.quorum` | **3** v0 → **4** v1 | (a) | Per [01 §5.6]: quorum scales with panel; majority of 7 = 4 |
| `jury.age_requirement_days` | 60 | (a) | [01 §5.4] |
| `jury.max_concurrent_assignments` | 3 | (a) | OQ-004 resolved |
| `jury.fallback_on_small_pool` | true | (a) | Bootstrap-friendly; admin can flip false later |
| `deltas.juror_aligned` | 10 | (b) | Pilot-data tuning |
| `deltas.juror_outlier` | -5 | (b) | Pilot-data tuning |
| `deltas.reporter_upheld` | 10 | (b) | Pilot-data tuning |
| `deltas.reporter_dismissed` | -5 | (b) | Pilot-data tuning |
| `deltas.endorsement_created_sponsor` | 5 | (a) | OQ-013 resolved |
| `deltas.endorsement_created_sponsee` | 5 | (a) | OQ-013 resolved |
| `deltas.sponsor_liability_minor` | -10 | (b) | [01 §5.2] -1% mapped to integer; pilot |
| `deltas.sponsor_liability_moderate` | -50 | (b) | -5% mapped; pilot |
| `deltas.sponsor_liability_severe` | -200 | (b) | -10–20% mapped; pilot |
| `liability.founder_multiplier` | 2.0 | (a) | [01 §5.2 footnote] honour-price principle |
| `liability.regular_multiplier` | 1.0 | (a) | Identity multiplier baseline |
| `liability.sponsor_liability_floor` | 0 | (a) | OQ-024 resolved |
| `report.base_weight` | 1.0 | (a) | OQ-006 resolved |
| `report.clamp_min` | 0.1 | (a) | OQ-006 resolved |
| `report.clamp_max` | 2.0 | (b) | OQ-006 ceiling flagged for pilot retro |
| `report.recency_half_life_hours` | 168.0 | (a) | OQ-006 resolved (1 week) |
| `report.case_threshold_micros` | 3_000_000 | (b) | OQ-006 resolved; pilot may tighten |
| `decay.positive_half_life_days` | 90 | (a) | [01 §5.3] −10% per 90 days |
| `onboarding.default_membership_state` | "member" | (a) | OQ-016 resolved |
| `onboarding.sponsor_gate_strategy` | "age" | (a) | OQ-014 resolved |
| `onboarding.sponsor_min_account_age_days` | 30 | (a) | OQ-014 resolved |
| `founder.max_founders_active` | 20 | (b) | Pilot data driven |
| `founder.max_expires_days` | 365 | (b) | Pilot data driven |
| `founder.max_seed_delta` | 200 | (b) | Pilot data driven |
| `job.snapshot_interval_seconds` | 900 | (a) | Operational; tune as load reveals |
| `job.snapshot_batch_chunk_size` | 500 | (a) | Operational |

### 5.2 v1 new keys — defaults

| Key | Default | Tag | Rationale |
|---|---|---|---|
| `jury.severity_thresholds.minor` | "majority" (enum) | (a) | [01 §5.6] |
| `jury.severity_thresholds.moderate` | "60%" | (a) | [01 §5.6] |
| `jury.severity_thresholds.severe` | "75%" | (a) | [01 §5.6] |
| `jury.diversity_constraints_enabled` | true | (b) | [01 §5.6]; pilot retro reviews enforcement vs. small-community feasibility |
| `jury.appeal_panel_size_increase` | 2 | (a) | Appeal panel = 7 + 2 = 9 (per [01 §5.7] "larger jury") |
| `jury.deadline_window_hours` | 72 | (b) | Resolves [04 §4.2] `deadline_at` Phase 2a stub; pilot tunes |
| `liability.grace_window_minor_hours` | 24 | (a) | OQ-025 lean; sponsor-liability-v1 §4.2 owned |
| `liability.grace_window_moderate_hours` | 72 | (a) | OQ-025 lean; sponsor-liability-v1 §4.2 owned |
| `liability.grace_window_severe_hours` | 168 | (a) | OQ-025 lean; sponsor-liability-v1 §4.2 owned |
| `liability.grace_window_minimum_hours` | 1 | (a) | sponsor-liability-v1 §4.2 instance-wide floor (community cannot go below) |
| `liability.grace_window_maximum_hours` | 720 | (a) | sponsor-liability-v1 §4.2 instance-wide ceiling (30-day cap) |
| `liability.grace_window_alert_threshold_hours` | 24 | (a) | sponsor-liability-v1 §4.2 audit alert when community sets any per-tier below this |
| `liability.restoration_escapes_liability` | true | (a) | sponsor-liability-v1 §7.3 — restoration-completion default-escapes |
| `liability.restoration_severity_reduction_steps` | 0 | (b) | sponsor-liability-v1 §7.3 — 0 = full escape; non-zero shifts severity tier N steps down |
| `liability.multi_sponsor_escape_rule` | "any_revocation" (enum) | (b) | sponsor-liability-v1 §13.1 / OQ-V1-SL-01 — `any_revocation` \| `all_revocation` \| `majority_revocation` |
| `liability.revoke_rate_limit_per_day` | 5 | (b) | sponsor-liability-v1 §12.1 per-user per-rolling-24h cap on revocations |
| `decay.negative_half_life_days` | 180 | (a) | [01 §5.3] "negative slower" |
| `decay.endorsement_strength_half_life_days` | 90 | (a) | Default per-dimension half-life inherits positive |
| `decay.jury_reliability_half_life_days` | 90 | (a) | Same |
| `onboarding.sponsor_min_endorsement_strength` | 25 | (b) | OQ-020 — read only when `sponsor_gate_strategy = 'reputation'` |
| `onboarding.sponsor_allowlist_table_name` | "sponsor_allowlist" | (a) | OQ-020 explicit; v1 adds the table additively |
| `onboarding.provisional_membership_cooldown_days` | 14 | (b) | OQ-016 v1 enforcement; pilot |
| `founder.founder_seal_visible_in_profile` | true | (a) | OQ-017 lean; admin can flip false |
| `participation.weekly_active_delta` | 1 | (b) | OQ-019 option (a) |
| `participation.dormant_threshold_days` | 30 | (a) | OQ-019 |
| `participation.dormant_delta` | -2 | (b) | OQ-019 |
| `participation.attestation_enabled` | false | (c) | OQ-019 option (c) deferred to v2 unless pilot asks |
| *(appeal.* namespace owned by jury-mechanics-v1 §10 per B1 resolution 2026-04-19)* | — | — | See v1-jury-mechanics.prd.md §10 for `appeal.window_days`, `appeal.panel_size_multiplier`, `appeal.panel_size_floor_increment`, `appeal.threshold_tier_bump`, `appeal.auto_select_on_appeal_acceptance`. Note: `appeal.original_jurors_excluded` is a hard code rule per [01 §5.7], not a config key. |
| `federation.inbound_advisory_only` | true | (a) | ADR-006 — locked; admin can flip in v2 only |
| `federation.peer_attestation_ttl_days` | 30 | (b) | Pilot |
| `federation.signature_required` | true | (a) | Security baseline |
| `federation.quarantine_recommendation_severity_floor` | "moderate" (enum) | (b) | Pilot |
| `federation.outbound_publish_enabled` | true | (a) | Default-on; admin can disable for emergency |
| `rule_set.active_version_id` | NULL → set per-community at first rule_set creation | (a) | NULL-safe; reader returns "no rule set defined" |
| `rule_set.auto_carry_in_flight_cases` | true | (a) | Existing cases keep their snapshotted version; new cases use new |
| `rule_set.text_max_bytes` | 65536 | (a) | 64 KiB Markdown ceiling |
| `rule_set.version_propagation_delay_hours` | 24 | (b) | OQ-018 lean ("only instance-admin + 24h delay" — generalised to rule-set activation) |

**Counts:** 34 v0 keys (32 (a)/(b), 2 (c) — none in v0), ~35 v1-new keys enumerated in this PRD (admin-dashboard-owned + sponsor-liability-owned rows), of which **1 is decide-later (c)**: `participation.attestation_enabled`. Federation peer-trust-state knobs live in `v1-federation-inbound.prd.md` §10 and are not enumerated here. The authoritative v1 cross-PRD total of ~138 keys across 15 namespaces is documented in §3.1; §5.2 lists only the admin-dashboard + sponsor-liability rows, with jury-mechanics-v1, reputation-tuning-v1 and federation-inbound-v1 rows in their own PRDs.

### 5.3 Decide-later (c) defaults — escalation

For each (c) key, open a new OQ in `99-decisions-and-open-questions.md`:

| (c) Key | Proposed OQ | Reason |
|---|---|---|
| `participation.attestation_enabled` | OQ-V1-AD-04 — admin attestation table for participation | Needs UX design; pilot may not want it |

*(Federation peer-trust-state knobs previously listed here are owned by `v1-federation-inbound.prd.md` §10 and its own OQ-FED-IN-\* register. N1 2026-04-19: removed phantom "4 reserved" entry that inflated the (c) count without corresponding table rows.)*

---

## 6. Pages (minimal server-rendered admin UI)

Per CLAUDE.md "Frontend UI — v0 is backend + API only" and §1.1 G6, v1 ships **server-rendered HTML pages from Rust** — askama or maud (recommend askama: more mature, works with actix-web's `Responder` via the askama-actix-web crate).

| Page | URL | Purpose |
|---|---|---|
| Dashboard | `GET /admin/governance/dashboard` | Active cases, jury queue depth, recent config changes, federation status |
| Config editor | `GET /admin/governance/config` | Schema-driven form (one row per key in `CONFIG_KEY_METADATA`); JSON or HTML mode |
| Single-key editor | `GET /admin/governance/config/<key>?scope=<scope>` | Detail view: current value, history, preview button |
| Audit log | `GET /admin/governance/audit` | Searchable log; same data as `/api/v4/governance/admin/config/audit` |
| Rule-set manager | `GET /admin/governance/rule-sets/<community_id>` | List versions, view diffs, upload new |

### 6.1 Page implementation notes

- Templates in `crates/api/api/templates/governance/*.html.askama`
- Auth: re-use Lemmy's existing `LocalUserView` extractor; `is_admin()` for instance pages, `CommunityModeratorView::check_is_community_moderator` for community pages
- No JS frameworks; vanilla HTML + minimal `<script>` for the dry-run preview button (POST to API, render diff inline)
- Pages are **opt-in** — instance can disable HTML pages and run API-only via `governance.dashboard.html_pages_enabled = true|false` (new key, default `true`)

### 6.2 Dashboard widget data sources

| Widget | Source query | Refresh |
|---|---|---|
| Active cases | `SELECT COUNT(*) ... WHERE status NOT IN (Closed)` on `moderation_case` | Per page-load |
| Jury queue depth | `SELECT COUNT(*) ... WHERE status IN (Selected, Accepted)` on `jury_assignment` | Per page-load |
| Recent config changes | Last 20 from `/admin/config/audit` | Per page-load + SSE push from `/admin/audit/stream` |
| Federation status | Per-peer attestation count from `federation_attestation` (Phase 6 table) | Per page-load |
| Reputation health | Re-uses `admin_reputation_stats` handler output (Phase 5c task 62) | Per page-load |

---

## 7. Security

### 7.1 Capability checks

| Endpoint | Required capability |
|---|---|
| `POST /admin/config` with `scope = "instance"` | `is_admin(local_user_view)` |
| `POST /admin/config` with `scope = "community:<id>"` | `CommunityModeratorView::check_is_community_moderator(pool, community_id, person_id)` AND key.scope ∈ {Community, Both} |
| `POST /admin/config` with `requires_step_up = true` | (v1) instance_admin only; (v2) instance_admin + step-up token |
| `POST /admin/rule-sets` | `community_admin(community_id)` for `scope = community`; `is_admin` for instance-level rule sets |
| `GET /admin/dashboard` | `is_admin(local_user_view)` |
| `GET /admin/audit/stream` | `is_admin(local_user_view)` |

**Negative test invariant:** any `POST /admin/config` where (a) `scope = "instance"` but caller is not `is_admin`, OR (b) `scope = "community:<id>"` but caller is not a moderator of `<id>`, OR (c) key.scope is `Instance` but the request scope is `Community`, MUST return `403` and emit a `governance_log` entry of `entry_kind = "admin_config_change_denied"` with the attempt details.

### 7.2 Step-up auth (v2 reserved slot)

This PRD does not implement step-up auth (v2 milestone per ADR-010 and `06 §2.1`). It reserves the surface:

- `requires_step_up: bool` field in `ConfigKeyMetadata` (§3.2)
- `requires_step_up = true` for: `jury.panel_size`, `jury.quorum`, `jury.severity_thresholds.*`, `liability.*`, `rule_set.active_version_id`, `federation.inbound_advisory_only` (the keys that match `06 §2.1` step-up list: closing a case, changing quorum/voting thresholds, federation trust state change, rule-set edit)
- v1 handler returns 403 with body `{ "error": "step_up_required", "message": "v2: step-up auth not yet implemented; attempt logged" }` only if a future config flag `governance.dashboard.step_up_enforced = true` is set. v1 default is `false` (advisory: log the attempt but allow).

### 7.3 Rate limiting

- Admin endpoints inherit the existing `/governance` scope's `rate_limit.post()` middleware (`crates/api/routes/src/lib.rs:516`)
- `dry_run = true` requests are NOT rate-limited differently from real writes — the impact computation is the expensive part, and we want to discourage bulk previewing
- SSE stream `/admin/audit/stream` has its own throttle: max 1 connection per admin user

### 7.4 Governance log entry on every config write

Every successful `POST /admin/config` (with `dry_run = false`) inserts a `governance_log` entry of `entry_kind = "admin_config_changed"` via the existing signed/hash-chained `governance_log::append` helper (`crates/api/api/src/governance/governance_log.rs:87`). Payload schema:

```jsonc
{
  "scope": "instance",
  "key": "jury.panel_size",
  "value_type": "int",
  "previous_value": 5,
  "new_value": 7,
  "apply_at": "next_jury_cycle",
  "reason": "Pilot retro: bumping to v1-target panel size",
  "config_id": 12345,
  "downstream_impact": { /* same shape as §4.2 preview */ }
}
```

This matches the existing `admin-config-write.sh` script's transactional pattern (`scripts/brehon/admin-config-write.sh:140-160`) and reuses the same `entry_kind` string for backwards-compat with v0 audit consumers.

### 7.5 Rule-set version-change delay

Per OQ-018 lean ("only instance-admin + 24h delay" for rule-set version changes):

- `rule_set.active_version_id` defaults to `apply_at = "next_jury_cycle"` in metadata
- Additionally, the handler enforces a minimum 24h delay configured via `rule_set.version_propagation_delay_hours` (default 24)
- `dry_run` shows the activation timestamp prominently

### 7.6 Authorization invariants

- A community admin cannot edit instance keys (enforced server-side via `key.scope` check)
- A community admin can edit community-scoped overrides for keys with `scope ∈ { Community, Both }` only for their own community
- Instance admin can edit anything; community-scoped writes by instance admin are allowed (operator override)
- All denied attempts are logged to governance_log with `entry_kind = "admin_config_change_denied"` (new entry_kind)

---

## 8. Migration & Rollout Strategy

### 8.1 Storage layer

`governance_config` is already shipped in v0 (Phase 5a task 50, migration `2026-04-18-000000-0000_add_governance_config`). The append-history shape supports v1 unchanged — no schema migration needed for the table itself.

### 8.2 New v1 migrations

| Migration | Tables/columns | When |
|---|---|---|
| `add_rule_set_versions` | `rule_set_version` table; `moderation_case.rule_set_version_id` column | First v1 milestone |
| `add_sponsor_allowlist` | `sponsor_allowlist` table (per OQ-020 `'allowlist'` strategy) | v1.x once strategy is needed |
| `add_case_applied_config_snapshot` | `moderation_case.applied_config_snapshot` JSONB column | First v1 milestone (supports `requires_re_jury` grandfathering) |
| `seed_v1_config_keys` | INSERT 27 new v1 keys into `governance_config` (excludes `rule_set.active_version_id` per v1-AD-a §4.1 absence-of-row decision) | First v1 milestone |

All four migrations are idempotent; each `up.sql` uses `ON CONFLICT DO NOTHING` for INSERTs and `IF NOT EXISTS` for CREATE statements (matching the v0 patterns).

### 8.3 Initial seed (27 new keys; `rule_set.active_version_id` deliberately un-seeded)

Single `INSERT … ON CONFLICT (scope, key, valid_from) DO NOTHING` per the v0 pattern (`migrations/2026-04-18-000000-0000_add_governance_config/up.sql:78`). Defaults from §5.2.

### 8.4 Deprecation path for `admin-config-write.sh` — gated on OQ-018

**Per NOT5 2026-04-19, script deprecation is GATED on the OQ-018 HTTP config-write endpoint actually shipping**, not on a v1.1 milestone alone. Sponsor-liability-v1 introduces 10+ new keys that currently must be set via `admin-config-write.sh` until OQ-018's endpoint lands. Deprecating the script on a v1.1 date-milestone without confirming OQ-018 has shipped would strand pilot operators.

- **v1.0** ships HTTP API (`POST /api/v4/governance/admin/config`) + `admin-config-write.sh` side-by-side. README marks the script deprecated in favour of the HTTP API; the script gains a `--legacy-mode-confirm` flag to require explicit opt-in.
- **Deprecation condition (NOT a date — a capability test):** script retirement is gated on all three of the following being true simultaneously:
  1. OQ-018 `POST /api/v4/governance/admin/config` endpoint is live on a shipped release (not just merged to `main`).
  2. Pilot operator confirms the HTTP API covers every key class the script writes (strings, ints, floats, bools, enums, text blobs).
  3. `governance_log` entries written by both paths are byte-identical (verified via round-trip test: write via HTTP, write via script, diff the two log rows — signatures will differ but payload + `entry_kind` must match).
- **v1.1 is the target but contingent.** If OQ-018's endpoint ships in v1.0 and conditions 2+3 are met by v1.0 pilot review, v1.1 removes the script. If OQ-018 slips to v1.1 or v1.2, script removal slips with it. The script is harmless to keep shipping; the ops cost is one deprecation warning banner on startup, not runtime load.
- **governance_log audit-trail continuity.** Both code paths emit `governance_config_changed` entries via the same `governance_log::append` function, so there is no audit-trail discontinuity whether operators use the script or the HTTP API during the side-by-side window.

### 8.5 Backwards compatibility

- Hardcoded `DEFAULT_*` consts in `crates/api/api/src/governance/config.rs` remain as the **third-tier fallback** if a `governance_config` row is missing — already documented and tested (`every_seeded_key_has_const_fallback`)
- v1 adds new `DEFAULT_*` consts for the 27 new keys; the parity test becomes parametric — `SEEDED_KEYS_WITH_CONSTS.len() == EXPECTED_SEED_COUNT + EXPECTED_SEED_COUNT_V1_AD` (34 + 27 = 61 post-v1-AD-a merge; further sub-PRDs contribute their own `EXPECTED_SEED_COUNT_V1_*` consts per v1-AD-a §19)
- v0 callers reading config via `get_int`/`get_float`/`get_bool`/`get_text` keep working unchanged
- v0's `admin_assign_jury` already reads `jury.panel_size` and `jury.fallback_on_small_pool` from config — no code change for the panel-size bump from 5 to 7 beyond the seed value

### 8.6 Rollout sequencing

1. Land migration `add_rule_set_versions` + `add_case_applied_config_snapshot` + `seed_v1_config_keys` (one PR)
2. Land HTTP API endpoints (`admin_config.rs`, `admin_rule_sets.rs`, `admin_dashboard.rs`, `admin_audit_stream.rs`) — one PR per endpoint module to keep diffs reviewable
3. Land server-rendered HTML pages (askama templates) — one PR
4. Update `INSTALL.md` (v1 self-host packaging deliverable per `05 §7.1`) to reference the dashboard
5. Pilot operator confirms HTTP API works (all three §8.4 deprecation-condition gates satisfied); `admin-config-write.sh` flagged deprecated
6. v1.x retires the script — contingent on OQ-018 endpoint having actually shipped, not a date milestone (per NOT5 2026-04-19)

---

## 9. Open Questions for v1 Design Phase

| ID | Question | Lean | Owner |
|---|---|---|---|
| OQ-V1-AD-01 (new) | Bulk config import/export format (JSON Lines for diff-friendly Git? YAML for human-readable? Postgres dump?) | JSON Lines, one row per `governance_config` row, with provenance comment per stanza | TBD |
| OQ-V1-AD-02 (new) | Multi-tenant case: does one Brehon instance host multiple distinct community federations? | Defer to v2; v1 assumes single-tenant per instance per ADR-001 | TBD |
| OQ-V1-AD-03 (new) | Dashboard for federation peer trust state — separate v1 PRD (`v1-federation-inbound.prd.md`) or merged here? | Separate PRD; this PRD's `federation.*` namespace covers the *config*, the federation PRD covers the operator UX for peer add/remove/quarantine | TBD |
| OQ-V1-AD-04 (new) | Admin attestation for participation_consistency (OQ-019 option (c)) — UX shape? | Defer; ship config flag (`participation.attestation_enabled = false`) and revisit at pilot retro | TBD |
| OQ-V1-AD-05 (new) | Should `dry_run` previews be logged to a separate `dry_run_preview_log` table for forensic value? | No, defer; if pilot wants it, add then | TBD |

---

## 10. Cross-References

### 10.1 v1 PRDs that consume this dashboard

| PRD | What it consumes |
|---|---|
| `v1-jury-mechanics.prd.md` (sibling) | All `jury.*` keys including new `jury.severity_thresholds.*`, `jury.diversity_constraints_enabled`, `jury.appeal_panel_size_increase`, `jury.deadline_window_hours` |
| `v1-reputation-tuning.prd.md` (sibling) | `decay.*`, `deltas.*`, `participation.*` namespaces; reuses `admin_reputation_stats` panel |
| `v1-sponsor-liability.prd.md` (sibling) | `liability.grace_window_*` keys (OQ-025); uses dashboard's `dry_run` to preview sponsor liability changes |
| `v1-federation-inbound.prd.md` (sibling) | `federation.*` namespace; this PRD's dashboard surfaces *what* — federation PRD surfaces *how* peers are added and managed. **Depends on Phase 6 merge** — see `v1-federation-inbound.prd.md` §8.0 for hard-gate conditions (per B5 resolution 2026-04-19). |
| `v1-rule-set-versioning.prd.md` (sibling, optional) | Rule-set creation flow; this PRD owns the API, the rule-set PRD owns the editor UX (could be merged into this PRD if a single owner ships both) |

### 10.2 v1 carry-forward issues this PRD resolves or shapes

| Issue | Status |
|---|---|
| #16 OQ-018 admin config-write HTTP endpoint | **This PRD is the design.** |
| #15 formalise appeal window with bounded duration | **Resolved** by `appeal.window_days = 7` (owned by jury-mechanics-v1 §6.5 + §10 per B1 2026-04-19; this PRD surfaces the knob via the config-write API but not the default) |
| #13 per-community permission filter on `list_cases` | Out of scope (separate PRD), but the `community_admin(id)` capability check pattern this PRD establishes is reused |
| #12 add `assignee` filter to `list_cases` DTO | Orthogonal |
| #14 re-jury path for Appealed cases | Out of scope (jury-mechanics PRD); affected by `jury.appeal_panel_size_increase` from this PRD |
| #11 original-reporter appeals | Orthogonal |
| #19 community-scope `count_active_sanctions` | Orthogonal but depends on per-community config awareness this PRD codifies |
| #20 `staleness_check` overflow | Orthogonal |
| #22 `list_capability_changed_entries_since` limit clamp | Orthogonal |
| #29 config-flip assertion determinism | Test-only; this PRD's `apply_at = next_jury_cycle` semantics may simplify this test |

### 10.3 Design-doc anchors

- [99 OQ-018](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) — primary
- [99 OQ-002, OQ-005, OQ-019, OQ-020, OQ-025, OQ-026](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)
- [99 ADR-010 staged releases](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md), [ADR-013 EmergencyRemove](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md), [ADR-015 pseudonyms](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)
- [01 §3 Core principle](../../../docs/brehon-law-inspired-network/01-vision-and-principles.md), [§5 Baseline policy parameters](../../../docs/brehon-law-inspired-network/01-vision-and-principles.md)
- [04 §3 governance_config / governance_log models](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md), [§7 Routes](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md)
- [05 §7.1 v1 deliverables](../../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md)
- [06 §2.1 step-up auth](../../../docs/brehon-law-inspired-network/06-security-and-threat-model.md), [§2.2 governance plane separation](../../../docs/brehon-law-inspired-network/06-security-and-threat-model.md)
- [V2/messaging.md §3.3 admin config panel pattern](../../../docs/brehon-law-inspired-network/V2/messaging.md) — informs the per-community-with-instance-default schema sketch this PRD adopts

### 10.4 Code anchors (v0 surfaces this PRD extends)

- `crates/api/api/src/governance/config.rs` — v0 reader, `Scope` enum, `ConfigCache`, `SEEDED_KEYS_WITH_CONSTS`, parity tests
- `crates/db_schema/src/source/governance/governance_config.rs` — Diesel model + `GovernanceConfigInsertForm`
- `crates/db_schema/src/source/governance/governance_log.rs` — append-only log model
- `crates/api/api/src/governance/governance_log.rs` — `append()` helper + entry_kind consts
- `crates/api/api/src/governance/admin_close_case.rs`, `admin_assign_jury.rs`, `admin_reputation_stats.rs` — v0 admin handler patterns to follow
- `migrations/2026-04-18-000000-0000_add_governance_config/up.sql` — v0 schema + 34-key seed
- `migrations/2026-04-20-000000-0000_add_governance_log_notify/up.sql` — `pg_notify('governance_events', …)` trigger that powers the SSE audit stream
- `scripts/brehon/admin-config-write.sh` — operator-tooling pattern this PRD replaces

---

## 11. Resolutions applied (2026-04-19)

v1-planning-session cross-PRD coherence audit applied the following resolutions. Traceable via `.claude/PRPs/v1-planning-queue.json`.

| ID | Resolution | Effect in this PRD |
|---|---|---|
| **B1** | jury-mechanics owns `appeal.*` namespace fully | §5.2: 3 `appeal.*` rows deleted + cross-ref line to jury-mechanics-v1 §10. §3.1 `appeal.*` row updated (5 v1 additions, jury-mechanics-owned). §10.2 issue #15 row notes resolution ownership shift to jury-mechanics. |
| **B2** | Dotted `jury.panel_size.<status>.<severity>` matrix WITH single-key fallback per OQ-026 cascade | §5.1 `jury.panel_size = 7` row kept as v1 root default with cascade-precedence note. §3.5 cascade sentence added (cascade order + v0 compatibility). jury-mechanics-v1 §3.4 owns the 9-cell matrix + cascade helper directive. |
| **B3** | Collapse `cron.*` → `job.*`/`participation.*`/`deltas.*`; keep `bounds.*` + `feature.*` as new 1st-class namespaces | **Done.** §3.1 prose rewritten to 138-key / 15-namespace authoritative registry with per-PRD contribution sub-table; new rows for `bounds.*` (8 keys, reputation-tuning), `feature.*` (1 key, reputation-tuning); `participation.*` / `deltas.*` / `job.*` / `onboarding.*` rows extended to reflect reputation-tuning renames. Namespace-collapse rationale footnote added. |
| **B4** | Sponsor-liability `sponsor.*` → flat `liability.*` | **Done.** §3.1 `liability.*` row updated to +10; §5.2 gained 7 new sponsor-liability-owned rows (grace_window minimum/maximum/alert_threshold, restoration_escapes_liability, restoration_severity_reduction_steps, multi_sponsor_escape_rule, revoke_rate_limit_per_day). Cross-reference line added to each new row citing sponsor-liability-v1 sections. |
| **B5** | federation-inbound hard-gated on Phase 6 merge | **Done.** §10.1 federation-inbound consumer row gains inline "**Depends on Phase 6 merge** — see `v1-federation-inbound.prd.md` §8.0 for hard-gate conditions" sentence. Full §8.0 preconditions list owned by federation-inbound-v1 (its §16 Resolutions documents the new §8.0 / §8.2 preamble / §15.2 prescriptive rewrite). |
| **N1** (collateral of B4) | Drop phantom 4 reserved federation-peer keys from §5.2 (c)-count | **Done.** §5.2 count line rewritten to drop phantom 4 (now reports 1 decide-later: `participation.attestation_enabled`). §5.3 escalation table dropped the `(4 reserved federation-peer keys)` row with a footnote pointing federation-peer knobs to `v1-federation-inbound.prd.md` §10. |
| **NOT1** | OQ prefixed-namespace convention (collision resolution) | **Done.** admin-dashboard's five `OQ-027..031` renumbered to `OQ-V1-AD-01..05` via mechanical replace-all across §5.3 table, §9 Open Questions table, §11 Sources list, and the (c)-key escalation footer. No substantive text changed; only the identifier prefix shifts to match reputation-tuning / sponsor-liability / federation-inbound's pre-existing `OQ-V1-*` convention. Jury-mechanics OQ-027..032 renamed separately to OQ-V1-JM-01..06 (see `v1-jury-mechanics.prd.md` §15 Resolutions). |
| **NOT4** | §3.2 `CONFIG_KEY_METADATA` storage: commit to compile-time | **Done.** §3.2 convention paragraph followed by a new explicit storage-commitment paragraph: `CONFIG_KEY_METADATA` is a `&'static [ConfigKeyMetadata]` compile-time array (NOT a runtime DB table). Every new key requires a PR-against-Rust-code edit. Matches the Watch-1 parity contract with `SEEDED_KEYS_WITH_CONSTS` (also compile-time). §6.1 reference already reads "one row per key in `CONFIG_KEY_METADATA`" which is consistent with the compile-time commitment. |
| **NOT5** | §8.4 `admin-config-write.sh` deprecation gated on OQ-018 | **Done.** §8.4 rewritten: deprecation is gated on three conditions simultaneously (OQ-018 endpoint shipped, pilot operator confirms all key classes covered, `governance_log` entries byte-identical), not on a v1.1 date milestone. v1.1 is the target but contingent on OQ-018 actually shipping. §8.6 rollout sequencing bullet updated to reflect the capability-gate semantics. |
