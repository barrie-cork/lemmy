# Verify report — v1-SL-b

**Run at:** 2026-05-06T16:35:39Z
**Phase branch:** `phase-v1-SL-b` @ `7f0e242257d28ad63b444a7b104552e983086ecd`
**Plan:** `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md`
**Outcome summary:** 3 stories: 3✓ 0✗-phantom 0✗-regression 0[malformed]

---

## Pre-flight

- **Phase branch exists:** ✓ `origin/phase-v1-SL-b`
- **Junior phase tasks running:** ✓ none observed (`junior task list` showed latest SL-b task #127 done)
- **§16a stories block:** ✓ present
- **§13 implementation commits:** ✓ present by shipped scope
  - Tasks 1, 2, 3 have individual task commits.
  - Tasks 4-12 shipped as the planned e2e batch commit `fff427580 test(v1-SL-b): e2e tests #1-9 — revoke_endorsement happy/edge paths (tasks 4-12)` plus follow-up fix commit `4934db25d fix(v1-SL-b): 3 bugs from Phase 2 e2e (handler + 2 test fixtures)`.

---

## Story 1 — DTO extension + handler module + route registration compile clean and the new route resolves

- **Composing tasks:** 1, 2, 3
- **Outputs:**
  - ✓ `crates/api/api_common/src/governance.rs` contains `pub struct RevokeEndorsement` with `pub reason: String`; derive line omits `Copy`.
  - ✓ `crates/api/api_common/src/governance.rs` contains `pub struct RevokeEndorsementResponse` with `endorsement_id`, `revoked_at`, and `liability_chain_severed_for_cases` fields.
  - ✓ `crates/api/api_crud/src/governance/revoke_endorsement.rs` exists and contains `pub async fn revoke_endorsement(` plus private `async fn process_revocation(`.
  - ✓ Handler references `ENTRY_KIND_ENDORSEMENT_REVOKED`, `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED`, `actor_pseudonym_helper::get_or_create`, `reputation_snapshot::recompute_snapshot`, and `governance_log::append`.
  - ✓ `crates/api/api_crud/src/governance/mod.rs` contains `pub mod revoke_endorsement;`.
  - ✓ `crates/api/routes/src/lib.rs` imports `revoke_endorsement::revoke_endorsement,` and registers `.route("/endorsement/revoke", post().to(revoke_endorsement))`.
- **Checkpoint:** ✓ `cargo-validate-workspace.yml` run `25342530143` completed `success` on Task 3 worker branch SHA `573cc4f85`.
- **Outcome:** ✓

## Story 2 — Six PRD §5/§12 contract guards

- **Composing tasks:** 4, 5, 6, 7, 8, 9
- **Outputs:**
  - ✓ `crates/server/tests/e2e.rs` contains `mod v1_sl_b_fixtures`.
  - ✓ Test exists: `revoke_endorsement_self_succeeds_updates_surety_and_recomputes_snapshots`.
  - ✓ Test exists: `revoke_endorsement_admin_succeeds_under_threshold`.
  - ✓ Test exists: `revoke_endorsement_re_revoke_returns_existing_revoked_at_no_log_no_severance`.
  - ✓ Test exists: `revoke_endorsement_non_sponsor_non_admin_rejects_with_not_found`.
  - ✓ Test exists: `revoke_endorsement_empty_reason_rejects`.
  - ✓ Test exists: `revoke_endorsement_rate_limit_enforces_unless_admin_bypasses`.
  - ✓ Test bodies assert `LemmyErrorType::NotFound`, `LemmyErrorType::Unknown`, and the current rate-limit variant `LemmyErrorType::TooManyRequests`.
  - ✓ Admin bypass test asserts `payload["rate_limit_bypassed"] == Value::Bool(true)` and under-threshold cases assert the field is absent.
- **Checkpoint:** ✓ `cargo-validate-workspace.yml` run `25346692356` completed `success` on Task 4-12 worker branch SHA `0b1588759`; Phase 2 e2e run `25352168003` completed `success` on `phase-v1-SL-b` SHA `e8101d661`.
- **Outcome:** ✓

## Story 3 — Grace-window severance branches drive `SponsorLiabilityPending → SponsorLiabilityEscaped`

- **Composing tasks:** 10, 11, 12
- **Outputs:**
  - ✓ Test exists: `revoke_endorsement_severs_grace_window_single_sponsor_case`.
  - ✓ Test exists: `revoke_endorsement_multi_sponsor_any_revocation_severs_chain`.
  - ✓ Test exists: `revoke_endorsement_no_pending_case_no_severance_only_revoked_log`.
  - ✓ Task 10 test asserts `CaseStatus::SponsorLiabilityEscaped`, JSON `version: 1`, JSON `reason: sponsor_revoked`, and string `actor_pseudonym`.
  - ✓ Task 11 test asserts only sponsor A's surety is revoked while sponsor B/C sureties remain untouched.
  - ✓ Task 12 test asserts `response.liability_chain_severed_for_cases.is_empty()` and exactly one `endorsement_revoked` log with zero `sponsor_liability_escaped` logs.
- **Checkpoint:** ✓ `cargo-validate-workspace.yml` run `25346692356` completed `success` on Task 4-12 worker branch SHA `0b1588759`; Phase 2 e2e run `25352168003` completed `success` on `phase-v1-SL-b` SHA `e8101d661`.
- **Outcome:** ✓

---

## Required actions

None. Merge-confirm gate is clear for `bm-pr`.
