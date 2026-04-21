# governance_log entry_kind registry

The `governance_log` table records every governance event as a row with an
`entry_kind TEXT` column. Each kind string must appear exactly once as a
`pub const ENTRY_KIND_*` in
`crates/db_schema/src/source/governance/governance_log.rs` (canonical
DEFINITION since Phase 6 DQ-6.6-inbound) AND be re-exported via
`pub use` from `crates/api/api/src/governance/governance_log.rs` (shim)
so call sites importing from either path compile.

Adding a new kind requires:

1. A new `pub const ENTRY_KIND_<NAME>: &str = "<snake_case>";` in
   `crates/db_schema/src/source/governance/governance_log.rs`.
2. A matching `pub use` line in the shim at
   `crates/api/api/src/governance/governance_log.rs` (alphabetical
   within the `pub use` block).
3. An entry in the matching PRD section of THIS file with: const name,
   `&str` value, source PRD (v0 = "shipped", v1+ = PRD name), emitting
   handler file path, one-line semantic description.
4. Collision check: the count of lowercase string literals in the
   canonical definition file (the `entry_kind` `&str` values) must equal
   the count of `pub const ENTRY_KIND_*` declarations there, and every
   literal must appear exactly once:

   ```bash
   rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
   # Count A: number of ENTRY_KIND_* consts (canonical definitions)

   rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs \
     | awk -F: '/ENTRY_KIND_/ {print}' \
     | grep -oE '"[a-z_]+"' | sort -u | wc -l
   # Count B: number of unique lowercase string literals on ENTRY_KIND lines

   # Invariant: A == B
   rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs \
     | awk -F: '/ENTRY_KIND_/ {print}' \
     | grep -oE '"[a-z_]+"' | sort | uniq -d
   # Expected: empty (no duplicate literals)
   ```

   NOTE: this invariant is **governance-log-domain only** — it does NOT
   reference `EXPECTED_SEED_COUNT` or `EXPECTED_SEED_COUNT_V1_*` (which
   are config-key-seed parity counters, a different domain).

This file is MANDATORY READING in `-p` mode. Every PRP plan that touches
governance writes reads it at first iteration.

## v0 entry kinds (19, shipped governance-v0)

| Rust const | `&str` value | Source | Emitting handler | Semantic |
|---|---|---|---|---|
| `ENTRY_KIND_REPORT_CREATED` | `report_created` | Phase 4 shipped | `crates/api/api_crud/src/governance/create_report.rs` | Report submitted, case opened or threshold-appended |
| `ENTRY_KIND_THRESHOLD_MET` | `threshold_met` | Phase 4 shipped | `create_report.rs` | Report accumulation crossed `report.case_threshold_micros` |
| `ENTRY_KIND_JURY_ASSIGNED` | `jury_assigned` | Phase 4 shipped | `crates/api/api/src/governance/admin_assign_jury.rs` | Admin assigned jury to a case |
| `ENTRY_KIND_PANEL_ASSEMBLED` | `panel_assembled` | Phase 4b shipped | `admin_assign_jury.rs` | Panel selection ran; jurors seated |
| `ENTRY_KIND_JURY_VOTED` | `jury_voted` | Phase 4b shipped | `crates/api/api/src/governance/submit_jury_vote.rs` | Individual juror submitted vote |
| `ENTRY_KIND_SANCTION_CREATED` | `sanction_created` | Phase 4b shipped | `submit_jury_vote.rs` | Quorum reached → sanction row inserted |
| `ENTRY_KIND_PUBLIC_LOG_PUBLISHED` | `public_log_published` | Phase 4b shipped | `submit_jury_vote.rs` | Redacted public case log entry created |
| `ENTRY_KIND_REPUTATION_DELTA` | `reputation_delta` | Phase 5a shipped | `crates/api/api/src/governance/reputation_snapshot.rs` + call sites | Reputation event applied |
| `ENTRY_KIND_CASE_DECIDED` | `case_decided` | Phase 4b shipped | `submit_jury_vote.rs` | Case transitioned to `Decided` |
| `ENTRY_KIND_CAPABILITY_CHANGED` | `capability_changed` | Phase 5a shipped | `reputation_snapshot.rs` | `can_sponsor` / `jury_eligible` / `trusted_reporter` flip |
| `ENTRY_KIND_SPONSOR_LIABILITY_APPLIED` | `sponsor_liability_applied` | Phase 5b shipped | `crates/api/api/src/governance/sponsor_liability.rs` | Liability delta written |
| `ENTRY_KIND_SPONSOR_LIABILITY_CLAMPED` | `sponsor_liability_clamped` | Phase 5b shipped | `sponsor_liability.rs` | Floor clamp fired (OQ-024) |
| `ENTRY_KIND_FOUNDER_SEEDED` | `founder_seeded` | Phase 5b shipped | `crates/server/src/bin/seed_founders.rs` + `reputation_snapshot.rs` | Founder reputation-event inserted |
| `ENTRY_KIND_ENDORSEMENT_CREATED` | `endorsement_created` | Phase 5b shipped | `crates/api/api_crud/src/governance/create_endorsement.rs` | Sponsor-endorsement written |
| `ENTRY_KIND_EMERGENCY_REMOVED` | `emergency_removed` | Phase 5c shipped | `crates/api/api/src/governance/admin_emergency_remove.rs` | ADR-013 admin-override removal |
| `ENTRY_KIND_JURY_ACCEPTED` | `jury_accepted` | Phase 5c shipped | `crates/api/api/src/governance/accept_jury_assignment.rs` | Juror accepted assignment |
| `ENTRY_KIND_JURY_DECLINED` | `jury_declined` | Phase 5c shipped | `crates/api/api/src/governance/decline_jury_assignment.rs` | Juror declined assignment |
| `ENTRY_KIND_JURY_REPLACEMENT_SELECTED` | `jury_replacement_selected` | Phase 5c shipped | `admin_assign_jury.rs` | Replacement juror seated after decline |
| `ENTRY_KIND_APPEAL_REQUESTED` | `appeal_requested` | Phase 5c shipped | `crates/api/api_crud/src/governance/request_appeal.rs` | Appeal request submitted |

## Phase 6 entry kinds (4, merged to `governance-v0` at PR #46 / `08065e1a1`)

Phase 6 ships outbound federation publish + inbound receive (advisory-only
per ADR-006). All four consts are DEFINED in
`crates/db_schema/src/source/governance/governance_log.rs` and re-exported
through the `api` shim.

| Rust const | `&str` value | Source | Emitting handler | Semantic |
|---|---|---|---|---|
| `ENTRY_KIND_FEDERATION_SANCTION_SENT` | `federation_sanction_sent` | Phase 6 shipped | `crates/api/api/src/governance/federation_outbox.rs` | Outbound sanction AP activity enqueued for delivery |
| `ENTRY_KIND_FEDERATION_SANCTION_RECEIVED` | `federation_sanction_received` | Phase 6 shipped | `crates/apub/activities/src/governance/inbox.rs::receive_remote_sanction_notice` | Inbound sanction AP received (advisory; `local_case_id = NULL`) |
| `ENTRY_KIND_FEDERATION_ATTESTATION_SENT` | `federation_attestation_sent` | Phase 6 shipped | `federation_outbox.rs` (builder at `crates/apub/activities/src/governance/publish_trust_attestation.rs`) | Outbound trust attestation enqueued |
| `ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED` | `federation_attestation_received` | Phase 6 shipped | `inbox.rs::receive_remote_trust_attestation` | Inbound attestation received (advisory) |

## v1-AD-a entry kinds (2, this sub-phase)

Landed alongside task 7's seed migration. v1-AD-a writes the const
declarations only; v1-AD-b wires the actual HTTP handler call site.
`admin_config_changed` is byte-identical to the string the v0 CLI
wrapper writes (`scripts/brehon/admin-config-write.sh:148`) so the
wrapper-vs-handler payloads remain interchangeable until v1-AD-b
deprecates the wrapper.

| Rust const | `&str` value | Source | Emitting handler | Semantic |
|---|---|---|---|---|
| `ENTRY_KIND_ADMIN_CONFIG_CHANGED` | `admin_config_changed` | v1-AD-a const; v0 `scripts/brehon/admin-config-write.sh` call site | v1-AD-b `crates/api/api/src/governance/admin_config.rs` (pending) + v0 shell wrapper | Config row edit succeeded; payload carries `{scope, key, value_type, value, reason}` |
| `ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED` | `admin_config_change_denied` | v1-AD-a const; v1-AD-b call site | v1-AD-b `admin_config.rs` capability-check reject path | Capability / scope check rejected attempted config write; payload mirrors attempted-change with `denial_reason` |

## v1-AD-c entry kinds (1, this sub-phase)

Landed alongside task 3's `admin_create_rule_set` handler. The const +
emitting call site land in the same commit. Payload carries the new
`rule_set_version.id`, the parent (nullable), the canonical
`text_sha256` (hex-encoded), and the `governance_config` row id flipping
`rule_set.active_version_id` for the community.

| Rust const | `&str` value | Source | Emitting handler | Semantic |
|---|---|---|---|---|
| `ENTRY_KIND_RULE_SET_VERSION_CREATED` | `rule_set_version_created` | v1-AD-c shipped | `crates/api/api/src/governance/admin_rule_sets.rs::admin_create_rule_set` | Rule-set version inserted + activated atomically; payload carries `{community_id, version, parent_id, text_sha256, rule_set_version_id, config_id, activated_at}` |

## v1 PRD reservation sections (populated when each PRD's plan writes)

Each future v1 sub-PRD OWNS a section below. Populated by that sub-PRD's
own plan file at its const-introducing task. Reserved slots avoid
re-ordering churn when a later PRD lands first.

### jury-mechanics-v1 (reserved — §8.5 of PRD enumerates 6 new kinds)

_To be populated by `v1-jury-mechanics.plan.md` at the const-landing
task:_ `jury_constraint_relaxed`, `appeal_panel_assembled`,
`appeal_decided`, `appeal_rejected`, `appeal_window_expired`,
`severity_tier_frozen`.

### sponsor-liability-v1 (reserved — §17 of PRD enumerates 5 new kinds)

_To be populated by `v1-sponsor-liability.plan.md`:_
`sponsor_liability_pending`, `sponsor_liability_fired`,
`sponsor_liability_escaped`, `endorsement_revoked`,
`restoration_completed`.

### reputation-tuning-v1 (reserved — §7 of PRD enumerates 7 new kinds)

_To be populated by `v1-reputation-tuning.plan.md`:_
`participation_cron_tick`, `vote_outcome_recorded`,
`evidence_quality_recorded`, `rollup_recomputed`, `decay_knob_changed`,
`sponsor_allowlist_added`, `sponsor_allowlist_removed`.

### federation-inbound-v1 (reserved — §§5.3/6.2/6.3/8.2 of PRD enumerate 11 new kinds)

_To be populated by `v1-federation-inbound.plan.md`:_
`federation_label_received`, `federation_inbound_blocked`,
`federation_inbound_dropped_oversize`,
`federation_inbound_dropped_schema`,
`federation_inbound_dropped_rate_limit`,
`federation_inbound_dropped_actor_rate_limit`,
`federation_inbound_persist_failed`,
`federation_inbound_cross_linked`, `federation_inbound_dismissed`,
`federation_peer_trust_changed`,
`federation_inbound_storage_cap_evicted`.

## Acceptance invariants (checked at every plan-review)

- [ ] `rg '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs | wc -l` returns the total count of all populated rows above (**26** at v1-AD-c end: 19 v0 + 4 Phase 6 + 2 v1-AD-a + 1 v1-AD-c).
- [ ] `rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs | awk -F: '/ENTRY_KIND_/ {print}' | grep -oE '"[a-z_]+"' | sort | uniq -d` returns no duplicate string literal values.
- [ ] `rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l` equals the `db_schema` define count — shim re-export parity is load-bearing for callers that import from the api path.
- [ ] Every populated row in this file has a Rust const (in `db_schema`) AND a `pub use` re-export (in the api shim) AND a call site (v1-AD-a's two consts pre-land their call sites: `_CHANGED` has the v0 shell wrapper today; `_CHANGE_DENIED` has no call site until v1-AD-b lands).
- [ ] Registry file matches the "Proposed deliverable" enumeration of GH issue #41 at land-time.

## Closes

GH #41 — this file replaces the issue's tracker role. Subsequent v1 PRDs
append their section via their own `/prp-plan` → `/prp-ralph` runs.
