# Verify report — v1-federation-inbound-a

**Run at:** 2026-05-18T22:30:00Z
**Phase branch:** `phase-v1-federation-inbound-a` @ `9be7f9135`
**Plan:** `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` @ `9be7f9135`
**Outcome summary:** 3 stories: 3✓ 0✗-phantom 0✗-regression 0[malformed]
**Verify mode:** advisor-inline (per `/brehon-verify` spec §9 + L15)

---

## Pre-flight

- §13 task commits 1–9 all present on `origin/phase-v1-federation-inbound-a`
  (tasks 1–5 via their validate-pending-laptop pass commits; tasks 6–9 via
  `feat(v1-federation-inbound-a): … (task N)` commits).
- Task 10 (retro) NOT present pre-merge — **expected by design**: bm-pr brief §2
  + plan §17 specify the phase retro is authored at USER-GATE-6 POST-merge
  (standard non-lane-closer flow). Not a phantom; task 10 is not in any §16a
  story's composing set.
- No Junior task running/queued on the phase (daemon DB confirmed empty).
- Plan has §16a Stories block (line 1829). V1+ plan (FILES YAML present).

---

## Story 1 — Schema migration round-trips cleanly + new tables/enums/columns/indexes/seeds exist

- **Composing tasks:** Tasks 1–5 (Cohort A).
- **Outputs:**
  - ✓ `migrations/2026-05-17-000000-0000_add_federation_inbound_v1/up.sql`:
    `CREATE TYPE federation_peer_trust_enum` (1), `CREATE TABLE` × 4
    (federation_peer, federation_inbox_dropped_log, federation_inbox_nonce,
    remote_moderation_label), `ALTER TABLE` × 2, `INSERT INTO governance_config`
    11 rows.
  - ✓ `…/down.sql`: `DELETE FROM governance_config` (1), `DROP TABLE` × 4,
    `DROP TYPE` × 2 — exact inverse of up.sql (fix-impl-5 + cr-21/copilot-3/4
    target; confirmed symmetric on phase branch).
  - ✓ 4 NEW `table!` blocks in `crates/db_schema_file/src/schema.rs`
    (federation_peer (instance_id), federation_inbox_dropped_log (id),
    federation_inbox_nonce (peer_instance, activity_id),
    remote_moderation_label (id)) — confirmed NEW via diff vs
    `origin/governance-v0`.
  - ✓ 2 NEW newtypes in `crates/db_schema/src/newtypes.rs`
    (`FederationInboxDroppedLogId(pub i64)`,
    `RemoteModerationLabelId(pub i32)`). [`FederationPeer` reuses
    `InstanceId` per planner DQ #234 — by design, not a missing newtype.]
  - ✓ Extended Phase-6 blocks contain new column lines (task 8 commit
    `998fef033`, exercised green by Phase-2 e2e).
- **Checkpoint:** ✓ Phase-2 e2e #3 GREEN (`91 passed; 0 failed; 5 ignored`)
  — `test_phase1_migrations_forward/_revert/_reapply` pass. DoD chain
  `cargo-check --workspace --features full` exit 0 (DQ #270).
- **Outcome:** ✓
- **Descriptor note (`[descriptor-note]`, non-blocking):** §16a Brief-Scope
  says "schema.rs has 4 new `table!` blocks". Lemmy 1.0-beta's schema lives
  in a **separate crate** at `crates/db_schema_file/src/schema.rs`, NOT
  `crates/db_schema/src/schema.rs`. The descriptor's bare "schema.rs"
  underspecified the crate path. Artifacts all present + e2e-green; this is
  a minor planner-descriptor imprecision, not a phantom. Logged for retro
  (planner should cite full crate-qualified paths for Lemmy-1.0 schema).

## Story 2 — 11 config keys + 9 ENTRY_KIND consts land with parity + registry invariants

- **Composing tasks:** Tasks 3, 4.
- **Outputs:**
  - ✓ `config.rs` contains `EXPECTED_SEED_COUNT_V1_FED_IN: usize = 11;`.
  - ✓ `ENUM_FEDERATION_PEER_TRUST` declared **with `"allowlisted"` in the
    slice** — cr-21 critical fix confirmed present on phase branch
    (df6eee91b).
  - ✓ 36 `DEFAULT_FEDERATION_INBOUND_*` references (≥ 11 declarations +
    parity-sum usages).
  - ✓ Exactly **9 NEW** `pub const ENTRY_KIND_FEDERATION_*` in
    `crates/db_schema/src/source/governance/governance_log.rs` (diff vs
    `origin/governance-v0` = 9 added lines).
- **Checkpoint:** ✓ Phase-2 e2e #3 parity tests pass (suite GREEN; parity
  138==138 implied by 0 failed). DoD `cargo-test --test e2e --no-run
  --features full` exit 0.
- **Outcome:** ✓

## Story 3 — Trust-state foundation helper compiles + behaves correctly under DB fixture

- **Composing tasks:** Tasks 6, 7, 8, 9.
- **Outputs:**
  - ✓ 4 model files present under
    `crates/db_schema/src/source/governance/` (federation_peer.rs,
    federation_inbox_dropped_log.rs, federation_inbox_nonce.rs,
    remote_moderation_label.rs).
  - ✓ `mod.rs` has 4 new `pub mod` lines (count = 4).
  - ✓ `e2e.rs` has `mod v1_federation_inbound_a_fixtures` (1) + both trust
    probes (`federation_peer_trust_lookup_returns_seeded_state`,
    `…returns_unknown_for_first_seen`).
  - ✓ `federation_peer.rs` declares Diesel via
    `#[cfg_attr(feature = "full", diesel(table_name = federation_peer))]`
    + Identifiable/Queryable/Selectable derives + Insert/Update forms
    (cr-22/cr-23/copilot-5/copilot-6 fixes confirmed: window_days guard,
    `Option<Option<String>>`, cfg-gated Duration, added_by_actor in
    conflict path — all on phase branch).
- **Checkpoint:** ✓ Phase-2 e2e #3 `v1_federation_inbound_a_fixtures`
  module → 2 passes (observed `federation_peer_trust_lookup_returns_seeded_state
  ... ok` + `…returns_unknown_for_first_seen ... ok` in the e2e log).
  DoD `cargo-clippy`-equivalent: `cargo-check --workspace` (non-full) +
  `--features full` both exit 0 (DQ #270; cfg-gate regression class clean).
- **Outcome:** ✓

---

## Required actions

None — all 3 stories ✓. Merge-confirm gate CLEAR.

One retro carry-forward (non-blocking, descriptor-only): §16a Story 1
Brief-Scope said "schema.rs"; Lemmy-1.0 schema is the separate crate
`crates/db_schema_file/src/schema.rs`. Planner should crate-qualify schema
paths in future federation/db plans. Added to phase retro carry-forward set.
