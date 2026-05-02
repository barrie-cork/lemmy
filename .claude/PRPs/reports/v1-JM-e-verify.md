# /brehon-verify — v1-JM-e

**Phase:** v1-JM-e
**Branch:** `phase-v1-JM-e` @ `4915a655b` (post-Task-6 retro + registry flip)
**PR:** *not yet opened* — `phase-v1-JM-e → governance-v0` pending bm-pr
**Run:** 2026-05-02, after retro authored + registry marker flipped
**Spec:** `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` §16a Stories block
**Outcome:** All four §16a stories ✓; no phantoms; safe to advance to bm-pr stage.

---

## Summary

| # | Story | Status | Evidence |
|---|---|---|---|
| 1 | Appeal-vote tally fires `appeal_decided` and closes the case | **✓** | `submit_jury_vote.rs:672` `async fn process_appeal_vote(`; emits `ENTRY_KIND_APPEAL_DECIDED` (3×); `AppealStatus::Decided` present; `JuryAssignmentRole::Appeal` arm (2×); e2e `mod v1_jm_e_fixtures` block + `seed_appealed_case_with_panel` (2× — original + via-report variant); test `appeal_panel_decides_no_action_overrides_to_advisory_label_chain` (1×). Capstone passed locally @ Task 2 commit `263a08f00`; same code path passed in Task 4 capstone regression `c83aaec05` (66/0/3 in 28:44). |
| 2 | governance_log sequence matches PRD §6.7 state machine | **✓** | `governance_log_sequence_matches_prd_state_machine` test fn present; ordered prefix in test body at e2e.rs:9603 cites `report_created → threshold_met → jury_assigned → panel_assembled → ...`. Test passed locally @ Task 3 commit `53e1b9ef8`; passed in Task 4 regression at log line 109. |
| 3 | appeal.window_days config churn binds at decision time | **✓** | `config_churn_appeal_window_days_does_not_invalidate_decided_cases` test fn present; test body uses `Duration::days(7)` (default-window assertion) and `Duration::days(30)` (post-flip assertion). Test passed locally @ Task 4 commit `c83aaec05` in 44.09s; passed in full regression at log line 101. |
| 4 | §12 security cluster — admin-visibility + spoofing-protection | **✓** | `constraint_relaxation_visible_to_community_admin_orphan_case_blocks_spoofing` test fn present; queries `jury_constraint_violation_log` (22 occurrences in test body region); asserts `is_err()` on `request_appeal` from non-defendant; `step_up_token: Option<String>` field present on `AdminTriggerAppealRejury` in `crates/api/api_common/src/governance.rs`. Test passed in Task 4 regression at log line 104 (byte-identical to Task 5 commit `7726f8cf4`). |

**Phantoms:** none. Every Brief-Scope output the plan §16a names is present + matches its structural pattern.

**Cross-cutting verification (plan §15.3):**
- `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` returns **33** ✓ (unchanged; no new const introduced this sub-phase)
- `rg -n '\(pending\)' .claude/rules/governance-log-entry-kind-registry.md | rg APPEAL_DECIDED` returns only the prose-explanation row (line 192), not the table-row entry. Table row at line 140 now reads `v1-JM-e crates/api/api/src/governance/submit_jury_vote.rs::process_appeal_vote (active)` ✓
- `process_appeal_vote` defined at `submit_jury_vote.rs:672`; emits `ENTRY_KIND_APPEAL_DECIDED` at `:872` ✓

**E2E regression evidence:**
- Task 3 full e2e: 64 passed; 0 failed; 3 ignored; 29:46 wall-clock @ commit `53e1b9ef8`
- Task 4 full e2e: 66 passed; 0 failed; 3 ignored; 28:44 wall-clock @ commit `c83aaec05` (also covers Task 5 byte-identical)
- 3 ignored tests are pre-existing GH-issue-tracked deflakes (#42, #43, #45) — not JM-e regressions

**DQ state at verify time:**
- Pending: 0
- Resolved: 108 (incl. #110 + #111 e2e validates passed)
- No carry-forward critical findings

---

## Outcome

**All §16a stories ✓; no phantoms; cross-cutting registry invariants hold; e2e regressions clean.**

Safe to advance to:
1. **Retro sign-off user gate** (retro at `.claude/PRPs/reports/v1-JM-e-retro.md` confidence 0.85)
2. **bm-cut chore(lint)** if `cargo clippy --workspace --features full --no-deps -- -D warnings` flags anything (gh workflow `cargo-validate-workspace.yml` will catch)
3. **bm-pr** to open `phase-v1-JM-e → governance-v0`
4. **bm-poll-cr** + **bm-triage** four-bucket
5. **bm-merge** confirm (final-tip Phase-2 e2e gate per `feedback_e2e_local_or_dispatch_user_choice`)
6. **/brehon-phase-transition**

No phantoms means no `fix-in-pr` work latent at this point — the PR cycle's only remaining surface is CR review + any clippy debt the workspace-check catches.
