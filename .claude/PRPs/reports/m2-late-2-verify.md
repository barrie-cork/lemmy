# Verify report — m2-late-2

**Run at:** 2026-06-12T17:35:00Z
**Phase branch:** `phase-m2-late-2` @ `b595bb4b435ac085cd71a57c3715ab3db0c486b9`
**Plan:** `.claude/PRPs/plans/m2-late.plan.md`
**Outcome summary:** 3 stories: 3✓ 0✗-phantom 0✗-regression 0[malformed]

---

## Story 1 — Sanction event schema + vocabulary exist

- **Composing tasks:** Task 1, Task 2
- **Outputs:**
  - ✓ `crates/db_schema_file/src/enums.rs` contains `pub enum SanctionKind`
  - ✓ `crates/db_schema_file/src/schema.rs` contains `sanction_event` + `sanction_subscriber` tables + `sql_types::SanctionKind`
  - ✓ `crates/db_schema/src/source/governance/sanction_event.rs` exists + 1657B
  - ✓ `crates/db_schema/src/source/governance/sanction_subscriber.rs` exists + 1010B
  - ✓ `crates/db_schema/src/newtypes.rs` contains `SanctionEventId`
- **Checkpoint:** ✓ exit 0 (cargo check --workspace --features full passed at T1/T2 validation gates; DQ 7d3a1a7f8581-001 + 8d79e47cceeb-001 both result:pass)
- **Outcome:** ✓

---

## Story 2 — Publisher machinery compiles (additive, not yet wired)

- **Composing tasks:** Task 3, Task 4
- **Outputs:**
  - ✓ `crates/api/api/src/governance/sanction_kind_map.rs` contains exhaustive `match action {` with all 8 `SanctionAction` variants named (no `_ =>`). Note: grep for `_ =>` returned a false-positive on the docstring text `(no \`_ =>\`)` — manually confirmed no wildcard arm.
  - ✓ `crates/db_schema/src/source/governance/governance_log.rs` contains `ENTRY_KIND_SANCTION_PUBLISHED` + `ENTRY_KIND_SANCTION_EVENT_DELIVERY_FAILED`
  - ✓ `crates/api/api/src/governance/governance_log.rs` re-exports both SANCTION consts
  - ✓ `crates/api/api/src/governance/sanction_publisher.rs` contains `enqueue_sanction_event` + `seed_sanction_subscriber` + `SanctionEventPayload`
- **Checkpoint:** ✓ exit 0 (cargo check --workspace --features full passed at T3/T4 validation gates; DQ 8f91e3725dc5-001 + c8e294aa18d8-001 both result:pass)
- **Outcome:** ✓

---

## Story 3 — Quorum vote publishes a schema-correct event to the bridge

- **Composing tasks:** Task 5, Task 6, Task 7, Task 8
- **Outputs:**
  - ✓ `crates/api/api/src/governance/submit_jury_vote.rs` contains exactly 1 `tokio::spawn` of `enqueue_sanction_event` (line 197, after `run_transaction` at line 159 — WP-2/R8 placement confirmed)
  - ✓ ADR-gate check: `enqueue_sanction_event` import at line 54 + callsite at line 197–198 — both definition-ref AND callsite present (ADR-GATE OK per §2.4a)
  - ✓ `crates/server/src/lib.rs` contains `seed_sanction_subscriber` invocation guarded by `BRIDGE_SANCTION_CALLBACK_URL` (lines 252–261)
  - ✓ `services/bridge/src/sanction_handler.rs` exists + 17850B
  - ✓ `services/bridge/src/appservice.rs` wires `POST /brehon/sanction-event` (sanction route present)
  - ✓ `crates/server/tests/e2e/m2_late.rs` exists + 15799B
  - ✓ `crates/server/tests/e2e.rs` contains `include!("e2e/m2_late.rs")`
- **Checkpoint:** ✓ E2E_EXIT_0 — Phase 2 e2e-2 retry: 135/135 passed, 5 skipped (run at phase tip a23923b13; valid for b595bb4b4 — only DQ-mutation commit between them). Log: `brehon-fork-validate-665/.claude/PRPs/debug/m2-late-2-phase2-e2e-2-retry.log`
- **ADR-gate:** ✓ `enqueue_sanction_event` defined in `sanction_publisher.rs` (T4) + called in `submit_jury_vote.rs` (T5) post-transaction spawn. ADR-015 pseudonymity: `subject_actor_pseudonym` from `actor_pseudonym.pseudonym` field (asserted by T8 e2e). ADR-008 atomicity: `sanction_event INSERT` wrapped in `run_transaction` (T2 CR-A fix).
- **Outcome:** ✓

---

## Required actions

None. All stories ✓. Advance to merge-confirm user gate (Gate 5).
