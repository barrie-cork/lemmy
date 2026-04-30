# Plan: v1-JM-e — Appeal-vote tally + integration capstone + step-up / spoofing / admin-visibility hardening

> **Shape G plan** — v1-JM-e is the **first** plan to ship under Shape G (Layer G2 push-and-exit). §15 references workflow YAMLs by path + expected `conclusion`, not inline cargo. Cargo runs on GitHub-hosted runners (workspace-check on `junior/*`) and on the laptop / `workflow_dispatch` GH runner (e2e on `phase-v1-JM-e` post-finalize-merge). See `.claude/PRPs/templates/plan.template.md` §15.6 + `.claude/PRPs/plans/v1-validate-agent.plan.md`.

## Table of contents

| § | Heading |
|---|---|
| 1 | Summary |
| 2 | Source |
| 3 | Problem statement |
| 4 | Solution statement |
| 5 | Metadata + complexity score |
| 6 | Relationship to other v1-JM sub-phases |
| 7 | Preflight guardrails inherited from prior phases |
| 8 | Flow design |
| 9 | Mandatory reading |
| 10 | Patterns to mirror |
| 11 | Files to change |
| 12 | NOT building in v1-JM-e |
| 13 | Step-by-step tasks |
| 14 | Testing strategy |
| 15 | Validation commands (DoD — Shape G) |
| 16 | Acceptance criteria |
| 16a | Stories (independently-testable behaviour units) |
| 17 | Completion checklist |
| 18 | Risks and mitigations |
| 19 | Notes |
| 20 | Confidence score |


---

## 1. Summary

v1-JM-e is the **JM-PRD capstone**. It activates the appeal-vote tally branch in `submit_jury_vote.rs` (the one handler edit that closes the appeal lifecycle JM-d opened), wires the v1 step-up-auth DTO slot per ADR-010, and ships four behavioural-coverage e2e tests that prove the v1 jury-mechanics PRD as a whole: a cross-sub-phase capstone (report → assign → vote → appeal → re-jury → appeal-decide → appeal-window-expire-close), an audit-log invariant test (every case's `governance_log` sequence matches the PRD §6.7 / §9 expected ordered set), a mid-flight config-churn regression extending JM-c's `v0_case_completes_under_v0_rules_after_v1_config_flip` with `appeal.window_days` churn, and a §12 security-hardening cluster (constraint-relaxation admin-visibility query + appeal-rights spoofing-protection negative test).

This sub-phase **flips `ENTRY_KIND_APPEAL_DECIDED` from `(pending)` to `(active)`** in `.claude/rules/governance-log-entry-kind-registry.md` (the const declared by JM-a; v1-JM-e is the fire site). It does **not** add a new `ENTRY_KIND_*` const; the registry count check `19 + 4 + 2 + 1 + 6 + 1 = 33` stays unchanged.

**Why now.** JM-d shipped the `request_appeal` rewrite + `admin_trigger_appeal_rejury` + `appeal_window_expiry` background job; the appeal lifecycle is end-to-end except for the appeal-panel vote-tally itself. Until the tally fires, `appeal.decided_at` stays NULL and the case sits in `Appealed` indefinitely (or is force-closed by the appeal-window-expiry job without a verdict). JM-e is the missing tally branch and the integration tests that prove the lifecycle holds.

**Headline acceptance condition.** All four §16a stories are `[done]` with their checkpoint workflows green: (1) appeal-vote tally fires `appeal_decided` + sets `appeal.decided_at` + transitions case Decided→Closed when an appeal panel meets the bumped-tier threshold; (2) the cross-sub-phase capstone test runs end-to-end with every state transition + every governance_log emission asserted in PRD-mandated order; (3) the config-churn regression confirms `appeal.window_days` is read at decision time but `appeal_window_expires_at` is bound on the case row (live-read vs snapshot semantics from PRD §9.1 cross-references); (4) the §12 security cluster confirms admin-visibility of relaxation rows + the orphaned-case appeal-rights spoofing-protection branch.

This sub-phase does not touch schema, does not add migrations, and does not add `ENTRY_KIND_*` consts. It is **handler + tests + 1 DTO field + 1 registry-marker flip**.

---

## 2. Source

- `.claude/PRPs/prds/v1-jury-mechanics.prd.md` — parent PRD: §6 (Appeals — first-class) §6.1-6.7, §9.1 (combined 9-step pseudocode — JM-e adds the appeal-tally branch parallel to JM-c's original-jury tally), §10 (knobs `appeal.threshold_tier_bump`, `appeal.window_days`), §11 (v0-compat — informs config-churn regression), §12 (Security §12.1-§12.4), §17 row 5 (canonical scope source), §17.1 (cross-PRD sequencing — JM-e parallel-safe with SL-d / rep-tuning-r3), §17.4 (preflight DQs #42/#43/#44/#46 inherited; cite-by-id, no inline compensation)
- `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` — predecessor plan: §10.6 (appeal-window-expiry batch shape), §13 Tasks 1-7 (what JM-d shipped), §20 (JM-e stub — six concrete deliverables). NOTE: JM-d plan §16a Stories block was **not present** (JM-d predates the §16a-mandatory mark — JM-c also predates it; per `feedback_schema_changing_spec_retrofit_question.md` the §16a addition is forward-only)
- `.claude/PRPs/plans/v1-jury-mechanics-c.plan.md` — JM-c plan: §9 step-9 `appeal_window_expires_at` write JM-e tally branch must respect, §13 Task 5 mid-flight config-churn regression test JM-e extends, §10.4 deadlock-branch pattern (JM-e mirrors for the appeal-panel deadlock case), §10.9 v0-compat regression pattern (JM-e extends with appeal-side churn)
- `.claude/PRPs/templates/plan.template.md` — canonical 20-section schema; §15.6 Shape G DoD shape; §16a Stories mandatory
- `.claude/agents/planning.md` — subagent contract; §13 per-task FILES YAML block discipline; §5 complexity score rule
- `.claude/PRPs/briefs/jm-e-planning-1.md` — brief authored by advisor session 2026-04-30 on `governance-v0` @ `5c9119688`
- `.claude/rules/decision-queue.md` — DQ schema-v2; `kind: "validate-pending"` writer/mutator contract under Shape G; planner attribution
- `.claude/rules/advisor-orchestrator.md` — Shape G orchestration; cohort dispatch + YAML overlap check; Phase 2 e2e user gate (local-vs-dispatch)
- `.claude/rules/governance-log-entry-kind-registry.md` — registry count check (33 consts) + `(pending)`→`(active)` flip discipline for `ENTRY_KIND_APPEAL_DECIDED`
- ADR-010, ADR-013, ADR-014, ADR-015 (`docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`)
- `.claude/lessons/feedback_complexity_score_pre_split.md` — complexity gate (this plan's score is 15 — see §5.1; planner DQ filed)
- `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md` — §13 per-task FILES YAML block discipline
- `.claude/lessons/feedback_parallel_cohort_dispatch.md` — `[P]` markers + YAML overlap rule
- `.claude/lessons/feedback_schema_changing_spec_retrofit_question.md` — Shape G + §16a are forward-only
- `.claude/lessons/feedback_story_grain_checkpoint.md` — §16a story-grain rule
- `.claude/lessons/feedback_brehon_verify_pre_merge.md` — `/brehon-verify` consumes §16a stories pre-merge
- `.claude/lessons/feedback_pre_phase_dod_smoke_test.md` + `feedback_plan_dod_dry_run_at_write.md` — DoD dry-run discipline (under Shape G this means `yamllint <workflow>` + workflow-presence check)
- `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — never `-p <crate>` + `--features full` (binds §15.7 manual-validation snippets)
- `.claude/lessons/feedback_handover_trailer_cohort_propagation.md` — per-task `HANDOVER:` commit trailer
- `.claude/lessons/feedback_principles_not_rules.md` — guidance > rigid rules
- `.claude/lessons/feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md` — retro shape
- `.claude/lessons/feedback_local_runtime_before_push_saves_ci_cycle.md` + `feedback_laptop_default_for_validate_pending.md` — laptop-default e2e advisable

---

## 3. Problem statement

Post-JM-d on `governance-v0` @ `5c9119688`:

- **The appeal lifecycle has no terminal vote-tally.** `request_appeal.rs` flips Decided→Appealed and seats the appeal panel; `admin_trigger_appeal_rejury.rs` covers the manual fallback; `appeal_window_expiry.rs` flips past-window Decided cases to Closed without a verdict. But there is **no handler that fires on an appeal-panel vote and computes the bumped-tier threshold**.
- **`ENTRY_KIND_APPEAL_DECIDED` is declared but `(pending)` in the registry.** The const exists at `crates/db_schema/src/source/governance/governance_log.rs:182` (declared by JM-a) and is re-exported via the api shim, but no call site fires it.
- **`appeal.decided_at` has no writer.** The column exists (`appeal.rs:24`) but stays NULL for every appeal until JM-e wires the writer.
- **`step_up_token: Option<String>` DTO slot is missing.** PRD §12.3 reserves the wire shape for v2; v1 ships the slot with **ignore-behaviour**.
- **The cross-sub-phase capstone test is missing.** JM-d Tasks 6-7 collapsed into the JM-d retro per the JM-d retro §3.1 — "plan overscoped".
- **The audit-log invariant test is missing.** JM-c's `governance_log_hash_chain_holds` (e2e.rs:277) asserts row-shape; the PRD §6.7 + §9 expected ordered set has no test.
- **The mid-flight config-churn regression for `appeal.window_days` is missing** (already-decided branch).
- **§12.2 admin-visibility surface is unread.** No test confirms a community-admin can read `jury_constraint_violation_log` rows.
- **§12.4 spoofing-protection is unverified.** No negative test asserts the orphaned-case branch.

The substrate is in place; only the closing handler edits + behavioural tests are missing.


---

## 4. Solution statement

Six surgical changes, organised as five impl tasks + retro:

### 4.1 Architecturally load-bearing decisions locked in this sub-phase

- **The appeal-vote tally is a NEW BRANCH in `submit_jury_vote.rs`, not a rewrite or a separate handler.** Per JM-d plan §20 + brief §2.1.a, branch detection happens at step 1 of the existing handler (after the assignment-Accepted check) by querying `jury_assignment::role` for the casting juror's row. If `role = Appeal`, follow the appeal-tally branch; else follow the original-jury tally branch (JM-c-shipped). The original-jury tally branch is **preserved verbatim**.
- **The appeal-tally branch reads snapshots from the `appeal` row, not the `moderation_case` row.** JM-d's `seat_appeal_panel` writes `appeal.panel_size_snapshot` + `appeal.threshold_count_snapshot` (per `admin_assign_jury.rs:1219-1226`). The appeal-vote tally **MUST NOT** read `case.threshold_count_snapshot` (that's the original-jury threshold). Instead: load the `Appeal` row by `case_id`, read `appeal.threshold_count_snapshot` and `appeal.panel_size_snapshot`. NULL on either is a defensive `LemmyErrorType::Unknown` (mirrors `submit_jury_vote.rs:264-268`).
- **The bumped-tier threshold computation is NOT redone in JM-e.** It was done at appeal-panel seating time by JM-d's `select_appeal_panel` (`admin_assign_jury.rs:1124-1144`) and snapshotted onto the `appeal` row. JM-e reads the snapshot.
- **On appeal-tally threshold-met, the handler:** (a) sets `appeal.decided_at = now()` + `appeal.status = AppealStatus::Decided`, (b) emits `governance_log` entry of kind `appeal_decided` with payload `{case_id, appeal_id, original_winning_decision, appeal_winning_decision, appeal_panel_size, appeal_threshold_count}`, (c) flips `moderation_case.status = CaseStatus::Closed` + sets `closed_at = now()`. Per PRD §6.7 the second `Decided` and the `Closed` transition are collapsed because the appeal verdict is terminal.
- **On appeal-panel deadlock:** mirror JM-c's `jury_deadlock` pattern — flip `moderation_case.status = CaseStatus::AdminReview`, emit a `jury_deadlock` entry with appeal-context fields, do **NOT** set `appeal.decided_at`.
- **On appeal-tally partial vote:** early-return with `case_decided: false`.
- **The `case_decided` log entry does NOT fire on appeal-tally threshold-met — `appeal_decided` is the appeal-side terminal log.** The original `case_decided` already fired at the original-jury decision.
- **No new sanction row is inserted.** Per PRD §6.7 v1 simplification — the appeal verdict is recorded as audit but does **NOT** override the original sanction. Sanction-override is a v2 candidate.
- **No federation outbound publish on appeal_decided.** Per ADR-014 + PRD §17 row 5 OUT.
- **`step_up_token: Option<String>` is added as a DTO field with v1 ignore-behaviour.** Per PRD §12.3 + ADR-010. **Decision (planner-resolved DQ #99 — see §19):** ship `step_up_token: Option<String>` on the `AdminTriggerAppealRejury` DTO (in `crates/api/api_common/src/governance.rs`) as the **single, judgment-defensible site** — admin-trigger-appeal-rejury is the closest existing handler to a "mid-case admin override" in v1. Behaviour: ignore the field. One-line struct-field addition with no handler-code change.
- **The community-admin visibility check (PRD §12.2) is a TEST-ONLY query, not a new handler or view-crate column.** Per PMD #14 / `feedback_build_what_tests_exercise`, the JM-e e2e test issues a direct Diesel query against `jury_constraint_violation_log`.
- **The §12.4 spoofing-protection check is a NEGATIVE test.** `case.creator_id == Some(caller_id)` cannot match a NULL `creator_id`; the negative test confirms `Err(LemmyErrorType::NotFound)`.
- **The four e2e tests live in a NEW `mod v1_jm_e_fixtures` block in `crates/server/tests/e2e.rs`, after `mod v1_jm_b_fixtures`.** Reuse `v1_jm_b_fixtures::{bootstrap, seed_user, seed_community, seed_jurors, seed_case, seed_jury_eligible_snapshots}` verbatim.
- **Each e2e test is its OWN §13 task.** Per PMD-side `feedback_junior_worker_e2e_edit_hang` (e2e.rs is >9000 lines), the four tests are split into four §13 tasks. They each Edit `crates/server/tests/e2e.rs` so the YAML overlap check refuses cohort dispatch; they ship serially.
- **Shape G — DoD references workflow YAMLs by path + expected `conclusion`, not inline cargo.** Per `.claude/PRPs/templates/plan.template.md` §15.6: every §13 task's DoD references `cargo-validate-workspace.yml` on the worker branch + workflow_run_id captured by the impl-task subagent post-push.
- **The `closed_at` semantic is preserved.** JM-c removed `closed_at` writes from `submit_jury_vote.rs`. JM-e's appeal-tally branch writes `closed_at = now()` because the appeal verdict is the *real* terminal — the case is Closed. The `appeal_window_expiry` job's filter `status = Decided AND appeal_window_expires_at < now()` does NOT match the appeal-tally output (status flips to `Closed` in the same UPDATE), so the two writers never collide.

### 4.2 Watchpoints (specific files / tables / `schema.rs` lines)

1. **`crates/api/api/src/governance/submit_jury_vote.rs:140-184`** — entry block + step-2 case-load. Branch detection MUST sit between the step-1 assignment-Accepted check (lines 151-161) and the step-3 vote insert (lines 187-197). Extend the existing assignment-check SELECT to additionally return the `role` column. **Watch:** do NOT add a second round-trip.
2. **`crates/api/api/src/governance/submit_jury_vote.rs:351-390`** — JM-c `jury_deadlock` branch shape. The appeal-panel deadlock branch MIRRORS this verbatim, reading `appeal.panel_size_snapshot` + `appeal.threshold_count_snapshot`.
3. **`crates/db_schema/src/source/governance/appeal.rs:11-36`** — `Appeal` row shape. NULL on `panel_size_snapshot` or `threshold_count_snapshot` is a defensive `LemmyErrorType::Unknown`. **Watch:** the `decided_at` field is `Option<DateTime<Utc>>`; `AppealInsertForm` does NOT include it. JM-e updates via UPDATE.
4. **`crates/db_schema/src/source/governance/governance_log.rs:170-184`** — entry-kind block. `ENTRY_KIND_APPEAL_DECIDED` at line 182. JM-e fires it; JM-e MUST NOT add a new const. Registry count `33` stays unchanged.
5. **`crates/db_schema_file/src/schema.rs:155-166`** — `appeal` table block. Confirm: `decided_at -> Nullable<Timestamptz>` (line 162), `panel_size_snapshot -> Nullable<Int4>` (line 164), `threshold_count_snapshot -> Nullable<Int4>` (line 165). All present.
6. **`crates/db_schema_file/src/schema.rs:524-534`** — `jury_assignment` table block. `role -> JuryAssignmentRole` (line 533).
7. **`crates/api/api/src/governance/admin_assign_jury.rs:1063-1071`** — `AppealPanelSelection` struct. JM-e reads `appeal` row directly; does NOT call `select_appeal_panel`. **Watch:** do NOT introduce coupling between submit_jury_vote and admin_assign_jury for the appeal-tally branch.
8. **`crates/api/api_common/src/governance.rs`** — `AdminTriggerAppealRejury` struct receives `step_up_token: Option<String>` field with `#[serde(default, skip_serializing_if = "Option::is_none")]`. **Watch:** field MUST be `Option<String>`, not a newtype.
9. **`crates/server/tests/e2e.rs:7129-7794`** — `mod v1_jm_b_fixtures` block. Reuse `bootstrap`, `seed_user`, `seed_community`, `seed_jurors`, `seed_jury_eligible_snapshots`, `seed_case`, `seed_founder_event` verbatim. Add `mod v1_jm_e_fixtures` AFTER (around line 7800; anchor: after `seed_founder_event`'s closing brace). **Watch:** e2e.rs is >9000 lines; per PMD-side `feedback_junior_worker_e2e_edit_hang`, multi-test bulk Edits hang the worker.
10. **`crates/server/tests/e2e.rs:8689-8884`** — JM-c `v0_case_completes_under_v0_rules_after_v1_config_flip`. JM-e Task 4 ships a sibling test rather than in-place patch.
11. **`crates/server/tests/e2e.rs:277-421`** — `governance_log_hash_chain_holds`. JM-e Task 3 audit-invariant is structurally distinct (sequence-shape vs row-shape). **Watch:** do NOT modify.
12. **`crates/api/api_crud/src/governance/request_appeal.rs:120-131`** — OriginalReporter eligibility branch. JM-e Task 5 spoofing-protection asserts `case.creator_id IS NULL` falls through to `Err(LemmyErrorType::NotFound)`. **Watch:** does NOT modify request_appeal.rs.

### 4.3 Rejected alternatives

- **Snapshot the bumped-tier threshold at appeal-vote time.** Rejected: ADR-010 + JM-d already snapshot at panel-seat time onto `appeal.threshold_count_snapshot`.
- **Add a `role` field to the `AdminTriggerAppealRejury` DTO instead of `step_up_token`.** Rejected: PRD §12.3 specifically names `step_up_token: Option<String>`.
- **Override the original sanction when the appeal verdict differs.** Rejected: PRD §6.7 v1 simplification.
- **Federation outbound publish on appeal_decided.** Rejected: ADR-014.
- **Add `appeal_decided` ENTRY_KIND const in JM-e.** Rejected: const exists at `governance_log.rs:182`.
- **Bundle the four e2e tests into one §13 task.** Rejected: per `feedback_junior_worker_e2e_edit_hang`.
- **Mark the four e2e tasks as `[P]`.** Rejected: YAML overlap rule refuses cohort because all four tasks `modifies: crates/server/tests/e2e.rs`.
- **Add a new SQL view or view-crate column for §12.2.** Rejected: PMD #14 — drift-stub when tests don't seed all reason_codes.
- **Rewrite historical `case_decided` log entry to include the appeal verdict.** Rejected: violates hash-chain invariant.
- **Add `step_up_token` slot on every governance handler DTO.** Rejected: per planner DQ #99 — single defensible site.


---

## 5. Metadata

- **Phase:** `v1-JM-e`
- **Branch:** `phase-v1-JM-e` (cut by BM-task before Task 1)
- **Estimated tasks:** 7 (Task 0 pre-flight + Tasks 1-5 impl + Task 6 retro)
- **Estimated cargo budget:** 0 GB peak local (Shape G: cargo runs on GH-hosted runners)
- **Forbidden-window applicability:** non-binding for impl-task throughput under Shape G; standard for any local diagnostic cargo run
- **Complexity score:** **15/10** — see breakdown below. Threshold-tripping; planner DQ #98 filed.

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Computed mechanically from §13 task list:

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | **0** | 5 impl tasks (Tasks 1-5; Task 0 + retro excluded). Count is exactly 5, so no points |
| Migrations touched | +2 each | **0** | No schema migrations |
| Crates touched | +1 each | **3** | `lemmy_api` (submit_jury_vote.rs), `lemmy_api_common` (governance.rs DTO), `lemmy_server` (tests/e2e.rs) |
| `crates/lemmy_server/tests/e2e/*.rs` edits | +3 each | **12** | Tasks 2/3/4/5 each modify `crates/server/tests/e2e.rs` (file-class match; canonical path in this repo is the single file `crates/server/tests/e2e.rs`). 4 × +3 = 12 |
| New ADR-affecting decisions | +2 each | **0** | Bumped-tier threshold settled at JM-d; step-up_token DTO scope is a planner DQ |
| Cargo budget peak above 6 GB | +1 per GB | **0** | Shape G — cargo runs off-box |
| **Total** | — | **15** | Threshold for split-DQ: `>8`. **Tripped.** |

### 5.2 Split-or-proceed DQ

Per `feedback_complexity_score_pre_split.md` + `planning.md`: planner files **DQ #98** (`from: "planner"`, `kind: "blocker"`, `answered_by: null`) BEFORE committing the plan, asking "Complexity score 15 exceeds 8 — split `v1-jury-mechanics-e` into `jm-e-1` (Tasks 1-2: appeal-vote tally + capstone test) + `jm-e-2` (Tasks 3-5: audit-log invariant + config-churn regression + §12 security cluster) + retro, or proceed as one plan?"

**Planner observation (non-binding lean):** the dominant factor is the four e2e edits (+12 of the +15). Splitting yields scores of `0+0+1+3 = 4` and `0+0+2+9 = 11`. The split does NOT meaningfully reduce the second sub-phase's score. Per `feedback_principles_not_rules.md`, the score is a signal, not a hard rule.

This plan ships under the **proceed-as-one** assumption pending DQ #98 resolution.

---

## 6. Relationship to other v1-JM sub-phases

| Sub-phase | Status | What it ships | JM-e dependency |
|---|---|---|---|
| v1-JM-a | MERGED (PR #92, governance-v0 @ `e1c22c759`) | Schema + 27 seeded keys + 6 entry-kind consts (incl. `_APPEAL_DECIDED`) + backfill | JM-e fires `_APPEAL_DECIDED`; reads `appeal_window_expires_at` schema column; reads `panel_size_snapshot` / `threshold_count_snapshot` on `appeal` |
| v1-JM-b | MERGED (PR #95, governance-v0 @ `4d2b93ed9`) | `admin_assign_jury` cascade + diversity + snapshot writes + `severity_tier_frozen` + `jury_constraint_violation_log` writer | JM-e Task 5 reads `jury_constraint_violation_log`; reuses `v1_jm_b_fixtures` for all e2e tests |
| v1-JM-c | MERGED (governance-v0 @ post-`4d2b93ed9`) | `submit_jury_vote` 9-step rewrite + `_JURY_DEADLOCK` const + 6 e2e tests | JM-e Task 1 adds appeal-tally branch to same handler; JM-e Task 4 extends JM-c's v0-compat regression as sibling test; JM-e Task 3 generalises JM-c's hash-chain-holds from row-shape to sequence-shape |
| v1-JM-d | MERGED (governance-v0 @ `5c9119688`) — direct-to-trunk Tasks 1-5; Tasks 6-7 collapsed into retro | `request_appeal` rewrite + `admin_trigger_appeal_rejury` + `appeal_window_expiry` background job + `select_appeal_panel` / `seat_appeal_panel` helpers + `appeal.decided_at` / `requester_role` / `panel_size_snapshot` / `threshold_count_snapshot` schema | JM-e reads `appeal.panel_size_snapshot` + `threshold_count_snapshot`; JM-e Task 5 exercises `request_appeal.rs:120-131`. JM-d plan §16a Stories was NOT present — JM-d predates the §16a-mandatory mark per `feedback_schema_changing_spec_retrofit_question.md`. JM-e §6 lists JM-d's §13 task completions instead: Tasks 1 (migrations), 2 (Diesel models + R3 sweep), 3 (request_appeal rewrite + winning_decision write), 4 (admin_trigger_appeal_rejury), 5 (appeal_window_expiry job) — all on `governance-v0`. Task 5 was the carry-forward concurrent dependency the brief flagged; advisor confirmed merged at brief-write time. |
| **v1-JM-e (THIS PLAN)** | NOT YET CUT | Appeal-vote tally branch + step-up DTO slot + 4 e2e tests + registry marker flip + retro | — |

**Cross-PRD sequencing (per JM PRD §17.1):**

- JM-e parallel-safe with SL-d — different lines in submit_jury_vote.rs (SL-d at step 7; JM-e at step 1 dispatch).
- JM-e parallel-safe with rep-tuning-r3/r4/r5.
- JM-e is the closing sub-phase of the JM PRD.

---

## 7. Preflight guardrails inherited from prior phases

Per JM PRD §17.4 (DQs #42, #43, #44, #46 all command-template-resolved 2026-04-23) + JM-c retro §3.2 amendments R1-R7 + JM-d retro §5 lessons:

- **DQ #42 — task-resume safety.** Inherited via `/prp-core:prp-implement` §1.4. No inline compensation.
- **DQ #43 — HTTP status code audit.** Inherited via `/prp-core:prp-implement` §4.1.1. JM-e Task 1 returns existing error variants; no new 4xx.
- **DQ #44 — Docker daemon preflight.** Enumerated in §13 Task 0 (R5 — Probe 0).
- **DQ #46 — v1/limitation GH issue capture.** Cited in §13 Task 6. Candidates: sanction-override (v2), step_up_token enforcement (v2), admin-dashboard query (admin-dashboard-v1).
- **R1 — `i64::from(...)` on `i32 ↔ i64`.** Bound in §13 Task 1.
- **R2 — `seed_jury_eligible_snapshots` BEFORE `admin_assign_jury` in tests.** Bound in §13 Tasks 2-5.
- **R3 — struct-extension grep sweep on `AdminTriggerAppealRejury` literal-construction sites.** Bound in §13 Task 1.
- **R4 — lowercase snake_case test names.** Bound in §13 Tasks 2-5.
- **R5 — pre-phase harness audit enumerates ALL probes.** Bound in §13 Task 0. Probes 0-9 listed.
- **R6 — uniform `--no-deps -- -D warnings` clippy.** Encoded in `cargo-validate-workspace.yml` line 92.
- **R7 — `cargo test --no-run -p lemmy_server --test e2e` on tasks touching a struct or re-export.** Encoded in `cargo-validate-workspace.yml` line 95. Bound in Task 1.
- **JM-d retro §3.5 — `feedback_clippy_rerun_after_fix.md`.** Bound in §13 Task 1 GOTCHA.
- **JM-d retro §5 — `feedback_laptop_default_for_validate_pending.md`.** Phase 2 e2e local-default.
- **JM-d retro §5 — `feedback_serena_auto_cargo_check.md`.** Serena MCP not in `.mcp.json` (verified at brief-write time); no remediation needed.

---

## 8. Flow design

### 8.1 Before state (post-JM-d-merge)

Appeal lifecycle today:

- `request_appeal` (JM-d) → flips Decided→Appealed, inserts Appeal row (decided_at NULL, snapshots NULL until seat).
- Auto-rejury OR `admin_trigger_appeal_rejury` (JM-d) → `seat_appeal_panel` inserts N jury_assignment rows with role=Appeal, emits appeal_panel_assembled, UPDATEs `appeal.panel_size_snapshot` + `threshold_count_snapshot`.
- Appeal jurors vote → calls `submit_jury_vote`.

**PROBLEM:** when an Appeal-role juror calls `submit_jury_vote` on an Appealed case, the idempotency guard at `submit_jury_vote.rs:242-255` fires on `case.status = Appealed` and returns `case_decided: true` with no actual decision. The vote is recorded in `jury_vote` (step 3) but the post-decision branch is bypassed. `appeal.decided_at` stays NULL forever. The case sits in Appealed indefinitely (the bg job's filter is `status = Decided`, not `Appealed`).

### 8.2 After state (v1-JM-e)

Revised `submit_jury_vote`:

- Step 1: validate assignment Accepted (extended SELECT returns role).
- Step 2: FOR UPDATE moderation_case row.
- Step 2.5 (NEW): if `jury_assignment.role == Appeal` → `process_appeal_vote(...)` (helper extracted to keep `process_vote` under `large_futures` lint threshold).
- Else → original-jury tally branch (JM-c-shipped, preserved verbatim).

`process_appeal_vote` body:

- Step 3: insert jury_vote.
- Step 4: flip jury_assignment.status → Submitted.
- Step 5: emit `jury_vote_submitted` log.
- Step 6 (NARROWER guard): only `Closed | EmergencyRemove | AdminReview` are terminal; `Appealed` is the EXPECTED status during appeal-vote.
- Step 7-appeal: load Appeal row; read `appeal.panel_size_snapshot` + `threshold_count_snapshot`. NULL → `LemmyErrorType::Unknown`.
- Step 7.5-appeal: per-decision tally on appeal-side jurors (JOIN `jury_vote` × `jury_assignment.role=Appeal`); first decision meeting threshold wins.
- Step 8-appeal: UPDATE `appeal SET decided_at=now, status=Decided`; UPDATE `moderation_case SET status=Closed, closed_at=now`; emit `appeal_decided` log.
- Return `SubmitJuryVoteResponse { vote_recorded: true, case_decided: true, decision: Some(appeal_winning_decision) }`.

Deadlock branch (mirror JM-c §10.4): UPDATE `moderation_case SET status=AdminReview`; emit `jury_deadlock` with `panel_kind: "appeal"`; do NOT set `appeal.decided_at`.

DTO change: `AdminTriggerAppealRejury` gains `pub step_up_token: Option<String>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`. v1 ignore-behaviour.

Registry change: `(pending)` → `(active)` on `ENTRY_KIND_APPEAL_DECIDED` row.

### 8.3 Endpoint changes

- `POST /api/v4/governance/jury/vote` — internal branch added based on `jury_assignment.role`. No DTO change.
- `POST /api/v4/governance/admin/trigger-appeal-rejury` — DTO gains optional `step_up_token: Option<String>` (v1 ignore). Wire-format-additive.
- No new endpoints.


---

## 9. Mandatory reading

### 9.1 Brehon design docs (P0)

- `.claude/PRPs/prds/v1-jury-mechanics.prd.md` §6, §9.1, §9.3-9.5, §10, §11, §12, §17 row 5, §17.1, §17.4
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` ADR-010, ADR-013, ADR-014, ADR-015
- `docs/brehon-law-inspired-network/04-data-model-and-api.md` §17

### 9.2 Codebase reads (P0 — mirror these patterns)

- `crates/api/api/src/governance/submit_jury_vote.rs` (entire file, 724 lines) — primary edit site.
- `crates/api/api_crud/src/governance/request_appeal.rs` (186 lines) — JM-d-shipped; Task 5 spoofing test exercises lines 120-131.
- `crates/api/api/src/governance/admin_trigger_appeal_rejury.rs` (119 lines) — JM-d-shipped.
- `crates/api/api/src/governance/admin_assign_jury.rs:1063-1229` — `AppealPanelSelection` + `select_appeal_panel` + `seat_appeal_panel`.
- `crates/api/api/src/governance/appeal_window_expiry.rs` (87 lines) — JM-d-shipped; Task 2 capstone drives via direct fn call.
- `crates/api/api_common/src/governance.rs` — DTO file. Locate `AdminTriggerAppealRejury`.
- `crates/db_schema/src/source/governance/appeal.rs` (53 lines) — `Appeal` row + `AppealInsertForm`.
- `crates/db_schema/src/source/governance/governance_log.rs:170-184` — entry-kind block.
- `crates/db_schema_file/src/schema.rs:155-166` (appeal), `:524-534` (jury_assignment), `:772-798` (moderation_case).

### 9.3 Test patterns (P1 — fixture sources)

- `crates/server/tests/e2e.rs:7129-7794` — `mod v1_jm_b_fixtures`. Reuse: `bootstrap`, `seed_user`, `seed_community`, `seed_jurors`, `seed_jury_eligible_snapshots`, `seed_case`, `seed_founder_event`.
- `crates/server/tests/e2e.rs:8064-9031` — JM-c shipped tests. Mirror direct-handler-invocation pattern + federation-config-builder bootstrap (lines 8120-8127).
- `crates/server/tests/e2e.rs:4299-4471` — JM-d-shipped `appeal_inside_window_succeeds_expired_rejects`. Mirror cases-A/B insert pattern for Task 5 spoofing test.
- `crates/server/tests/e2e.rs:277-421` — `governance_log_hash_chain_holds`. Reference (do NOT modify).

### 9.4 Rules (P0 — auto-loaded)

- `.claude/rules/governance-log-entry-kind-registry.md` — registry count (33) + `(pending)`→`(active)` discipline
- `.claude/rules/decision-queue.md` — schema-v2 `kind: validate-pending`
- `.claude/rules/advisor-orchestrator.md` — Shape G + Phase 2 e2e user gate
- `.claude/rules/branch-manager.md`, `.claude/rules/phase-branch.md`
- `.claude/rules/cargo-output-capture.md` + `.claude/rules/no-cargo-output-paste.md`
- `.claude/rules/pm-plugin-hooks-stable.md` (Task 0 Probe 7)

### 9.5 External documentation

- chrono: `Duration::days(i64)` and `DateTime<Utc>::now()` — already used in JM-c step 9 + JM-d `appeal_window_expiry`.
- diesel-async: `RunQueryDsl::execute` + `.set((...))` tuple form.
- serde_json: `json!(...)` for the `appeal_decided` payload.


---

## 10. Patterns to mirror

### 10.1 Branch-detection extension at step 1 of submit_jury_vote

**SOURCE:** `crates/api/api/src/governance/submit_jury_vote.rs:151-161` — assignment-Accepted check.

```rust
// Step 1: verify assignment is Accepted; capture role for branch dispatch.
let role: JuryAssignmentRole = jury_assignment::table
    .filter(jury_assignment::case_id.eq(data.case_id))
    .filter(jury_assignment::person_id.eq(juror_id))
    .filter(jury_assignment::status.eq(JuryAssignmentStatus::Accepted))
    .select(jury_assignment::role)
    .first::<JuryAssignmentRole>(conn)
    .await
    .map_err(|_e| LemmyErrorType::NotFound)?;
```

Then dispatch:

```rust
// Step 2.5 (NEW for v1-JM-e): branch on jury_assignment.role.
match role {
    JuryAssignmentRole::Appeal => {
        return process_appeal_vote(conn, juror_id, juror_pseudonym, data, &case_row, now).await;
    }
    JuryAssignmentRole::Original => {
        // Original-jury tally — JM-c-shipped, preserved verbatim below.
    }
}
```

`process_appeal_vote` is a new private fn alongside `process_vote` in the same file.

### 10.2 Appeal row load + snapshot reads

**SOURCE:** `crates/db_schema/src/source/governance/appeal.rs:11-36` + `submit_jury_vote.rs:264-300` (NULL defensive guard).

```rust
let appeal_row: Appeal = appeal::table
    .filter(appeal::case_id.eq(data.case_id))
    .select(Appeal::as_select())
    .first(conn)
    .await
    .map_err(|_e| LemmyErrorType::NotFound)?;

let appeal_panel_size: i32 = appeal_row.panel_size_snapshot.ok_or_else(|| {
    LemmyErrorType::Unknown(format!(
        "appeal {} has NULL panel_size_snapshot; seat_appeal_panel did not run",
        appeal_row.id.0
    ))
})?;
let appeal_threshold_count: i32 = appeal_row.threshold_count_snapshot.ok_or_else(|| {
    LemmyErrorType::Unknown(format!(
        "appeal {} has NULL threshold_count_snapshot; seat_appeal_panel did not run",
        appeal_row.id.0
    ))
})?;
let appeal_threshold_count_i64 = i64::from(appeal_threshold_count);  // R1
```

### 10.3 Appeal-side per-decision tally

**SOURCE:** `crates/api/api/src/governance/submit_jury_vote.rs:283-343` (JM-c per-decision tally) — JM-e mirrors with role JOIN.

```rust
let all_appeal_votes: Vec<(JuryDecision, Option<String>)> = jury_vote::table
    .inner_join(
        jury_assignment::table.on(
            jury_assignment::case_id.eq(jury_vote::case_id)
                .and(jury_assignment::person_id.eq(jury_vote::juror_id))
        )
    )
    .filter(jury_vote::case_id.eq(data.case_id))
    .filter(jury_assignment::role.eq(JuryAssignmentRole::Appeal))
    .select((jury_vote::decision, jury_vote::rationale))
    .load::<(JuryDecision, Option<String>)>(conn)
    .await?;
let mut tally: HashMap<JuryDecision, Vec<Option<String>>> = HashMap::new();
for (decision, rationale) in all_appeal_votes {
    tally.entry(decision).or_default().push(rationale);
}
let appeal_vote_count: i64 = i64::try_from(
    tally.values().map(Vec::len).sum::<usize>()
).map_err(|_e| LemmyErrorType::Unknown("appeal vote count overflows i64".to_string()))?;

// Stable enum-order iteration (mirrors submit_jury_vote.rs:317-343).
// INVARIANT: this list MUST cover every JuryDecision variant.
let mut appeal_winning_decision: Option<JuryDecision> = None;
for candidate in [
    JuryDecision::NoAction,
    JuryDecision::AdvisoryLabel,
    JuryDecision::Warning,
    JuryDecision::Cooldown,
    JuryDecision::RemoveContent,
    JuryDecision::SuspendLocalUser,
    JuryDecision::SuspendCommunityMember,
    JuryDecision::RecommendFederationAction,
] {
    let count = i64::try_from(tally.get(&candidate).map_or(0, Vec::len)).map_err(|_e| {
        LemmyErrorType::Unknown(format!(
            "appeal vote count for {candidate:?} on case {} overflows i64",
            data.case_id.0
        ))
    })?;
    if count >= appeal_threshold_count_i64 {
        appeal_winning_decision = Some(candidate);
        break;
    }
}
```

**GOTCHA:** the JOIN keeps original-jury votes out of the appeal tally; if we omitted the role filter, original-jury votes would pollute the tally.

**GOTCHA (R1):** every `i32 ↔ i64` uses `i64::from(...)`. NEVER bare `as` cast.

### 10.4 Appeal-deadlock branch

**SOURCE:** `crates/api/api/src/governance/submit_jury_vote.rs:351-390` — JM-c verbatim with appeal-context fields.

```rust
let appeal_winning_decision = match appeal_winning_decision {
    Some(decision) => decision,
    None => {
        if appeal_vote_count == i64::from(appeal_panel_size) {
            update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
                .set(moderation_case::status.eq(CaseStatus::AdminReview))
                .execute(conn)
                .await?;

            let tally_payload: serde_json::Map<String, Value> = tally
                .iter()
                .map(|(decision, votes)| (format!("{decision:?}"), Value::from(votes.len())))
                .collect();

            governance_log::append(
                &mut conn.into(),
                ENTRY_KIND_JURY_DEADLOCK,
                json!({
                    "case_id": data.case_id.0,
                    "appeal_id": appeal_row.id.0,
                    "panel_kind": "appeal",
                    "panel_size_snapshot": appeal_panel_size,
                    "threshold_count_snapshot": appeal_threshold_count,
                    "tally": tally_payload,
                }),
                Some(juror_pseudonym.clone()),
            )
            .await?;
        }
        return Ok(SubmitJuryVoteResponse {
            vote_recorded: true,
            case_decided: false,
            decision: None,
        });
    }
};
```

**GOTCHA:** `panel_kind: "appeal"` distinguishes from JM-c's original-jury deadlock entries (which lack the field).

### 10.5 Appeal-decide UPDATE batch + governance_log emission

**SOURCE:** `crates/api/api/src/governance/submit_jury_vote.rs:468-475` + `:626-636`.

```rust
update(appeal::table.filter(appeal::id.eq(appeal_row.id)))
    .set((
        appeal::decided_at.eq(Some(now)),
        appeal::status.eq(AppealStatus::Decided),
    ))
    .execute(conn)
    .await?;

update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
    .set((
        moderation_case::status.eq(CaseStatus::Closed),
        moderation_case::closed_at.eq(Some(now)),
    ))
    .execute(conn)
    .await?;

let original_winning_decision: Option<JuryDecision> = case_row.winning_decision;

governance_log::append(
    &mut conn.into(),
    ENTRY_KIND_APPEAL_DECIDED,
    json!({
        "case_id": data.case_id.0,
        "appeal_id": appeal_row.id.0,
        "original_winning_decision": original_winning_decision,
        "appeal_winning_decision": appeal_winning_decision,
        "appeal_panel_size": appeal_panel_size,
        "appeal_threshold_count": appeal_threshold_count,
    }),
    Some(juror_pseudonym.clone()),
)
.await?;
```

**GOTCHA:** reuse the same `now` from step-3 vote-insert (`submit_jury_vote.rs:200`).

**GOTCHA:** no public_case_log insert; no juror reputation_event writes; no federation outbound. v1 simplifications per §4.1.

**GOTCHA:** `actor_pseudonym = Some(juror_pseudonym)` — casting juror who triggered threshold. ADR-015.

### 10.6 step_up_token DTO field addition

**SOURCE:** `crates/api/api_common/src/governance.rs` — locate `AdminTriggerAppealRejury` struct.

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Admin-triggered appeal-rejury request — used when
/// `appeal.auto_select_on_appeal_acceptance = false`.
pub struct AdminTriggerAppealRejury {
    pub case_id: ModerationCaseId,
    /// v1-JM-e + PRD §12.3: reserved slot for v2 step-up auth on
    /// admin-mediated mid-case actions. v1 ignore-behaviour — the field
    /// is read from the wire, never validated, never rejected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_up_token: Option<String>,
}
```

**GOTCHA (R3):** if any literal-construction site (`AdminTriggerAppealRejury { case_id }`) exists in tests, propagate `..Default::default()` per `feedback_insertform_default_propagation.md`. Run `git grep -l 'AdminTriggerAppealRejury {'` after the field add.

### 10.7 Audit-log invariant test pattern

**SOURCE:** new in JM-e — no precedent for sequence-shape governance_log assertions.

```rust
let entries: Vec<String> = governance_log::table
    .filter(governance_log::payload.contains(json!({"case_id": case_id.0})))
    .order_by(governance_log::id.asc())
    .select(governance_log::entry_kind)
    .load::<String>(&mut conn)
    .await?;

let kinds_no_jury_voted: Vec<&str> = entries
    .iter()
    .filter(|k| k.as_str() != "jury_voted" && k.as_str() != "jury_vote_submitted")
    .map(|k| k.as_str())
    .collect();
let expected_prefix = vec![
    "report_created",
    "threshold_met",
    "jury_assigned",
    "panel_assembled",
    "case_decided",
    "appeal_requested",
    "appeal_panel_assembled",
    "appeal_decided",
];
assert_eq!(kinds_no_jury_voted, expected_prefix,
    "governance_log sequence must match PRD §6.7 state-machine prefix");
```

**GOTCHA:** `payload.contains(json!(...))` requires diesel-async JSONB containment surface; alternative: select all rows + filter client-side. Impl picks cleaner shape.

### 10.8 Config-churn regression test pattern

**SOURCE:** `crates/server/tests/e2e.rs:8689-8884` — JM-c v0-compat.

JM-e ships `config_churn_appeal_window_days_does_not_invalidate_decided_cases` as a NEW sibling test (Edit-with-anchor preference for shorter edits):

1. Seed two cases A + B with v0-default config; both Decided via JM-c happy path.
2. Flip `appeal.window_days = 30` via `admin_set_config`.
3. Seed case C with new live config + drive to Decided.
4. Assert: case A `appeal_window_expires_at - decided_at ≈ 7 days` (untouched by flip).
5. Assert: case B `appeal_window_expires_at - decided_at ≈ 7 days` (also untouched).
6. Assert: case C `appeal_window_expires_at - decided_at ≈ 30 days` (LIVE config read at decision time).

### 10.9 Spoofing-protection negative test pattern

**SOURCE:** `crates/api/api_crud/src/governance/request_appeal.rs:120-131`.

```rust
let orphan_case_form = ModerationCaseInsertForm {
    community_id: None,
    creator_id: None,  // ORPHANED
    target_type: CaseTargetType::Person,
    target_person_id: Some(target_person_id),
    target_post_id: None,
    target_comment_id: None,
    target_community_id: None,
    target_remote_url: None,
    reason_code: "orphan_appeal_spoof_test".to_string(),
    severity: CaseSeverity::Low,
    status: CaseStatus::Decided,
    threshold_score: 1,
    severity_tier: Some(SeverityTier::Minor),
    ..Default::default()
};
// Direct insert + UPDATE for appeal_window_expires_at + winning_decision +
// panel_size_snapshot to satisfy request_appeal's status guards.

let resp = request_appeal(
    Json(RequestAppeal { case_id: orphan_case.id, reason: "spoof".to_string() }),
    context.clone(),
    spoofer_view,
).await;
assert!(resp.is_err(),
    "PRD §12.4: orphaned-case (creator_id=NULL) appeal-rights cannot be spoofed");
```

### 10.10 Constraint-relaxation admin-visibility test pattern

**SOURCE:** `crates/db_schema_file/src/schema.rs:541-550` (`jury_constraint_violation_log`) + JM-b's writer.

```rust
// Seed a small-pool case (4 jurors; need 5 for Minor) so JM-b's R1 relaxation fires.

let admin_visible_relaxations: i64 = jury_constraint_violation_log::table
    .filter(jury_constraint_violation_log::case_id.eq_any(
        moderation_case::table
            .filter(moderation_case::community_id.eq(Some(community_id)))
            .select(moderation_case::id)
    ))
    .count()
    .get_result::<i64>(&mut conn)
    .await?;
assert!(admin_visible_relaxations > 0,
    "PRD §12.2: community-admin sees relaxation rows for cases in their community");
```

**GOTCHA (PMD #14):** drift-stub if seed does not exercise all `JuryConstraintRelaxationReason` variants.


---

## 11. Files to change

### `lemmy_api` crate

- `crates/api/api/src/governance/submit_jury_vote.rs` — appeal-tally branch detection at step 1 + new `process_appeal_vote` private fn. **Task 1**.

### `lemmy_api_common` crate

- `crates/api/api_common/src/governance.rs` — add `step_up_token: Option<String>` field on `AdminTriggerAppealRejury`. **Task 1**.

### `lemmy_server` crate

- `crates/server/tests/e2e.rs` — add `mod v1_jm_e_fixtures` block with the four new tests + the `seed_appealed_case_with_panel` fixture. **Tasks 2, 3, 4, 5** (one test per task; fixture introduced in Task 2 and reused by Tasks 3-5).

### Meta files

- `.claude/rules/governance-log-entry-kind-registry.md` — flip `(pending)`→`(active)` marker on `ENTRY_KIND_APPEAL_DECIDED` row. **Task 6 (retro)**.
- `.claude/PRPs/reports/v1-JM-e-retro.md` — CREATE retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`. **Task 6**.

### Files explicitly NOT touched

- `crates/api/api/src/governance/admin_assign_jury.rs` — JM-d-shipped helpers; JM-e reads only
- `crates/api/api_crud/src/governance/request_appeal.rs` — JM-d-shipped; Task 5 spoofing test confirms current behaviour
- `crates/api/api/src/governance/admin_trigger_appeal_rejury.rs` — JM-d-shipped handler body; v1 ignore-behaviour means no handler-code change for new DTO field
- `crates/api/api/src/governance/appeal_window_expiry.rs` — JM-d-shipped; Task 2 capstone drives via direct fn call
- `crates/db_schema/src/source/governance/{appeal,governance_log,moderation_case,jury_assignment}.rs` — schema models stable
- `crates/db_schema_file/src/schema.rs` — no migration
- `migrations/**` — no schema changes
- `crates/api/routes/src/lib.rs` — no new routes
- `crates/server/src/governance.rs` — no scheduler changes
- `.coderabbit.yaml`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` — no dep / build-config changes

---

## 12. NOT building in v1-JM-e

- **Sanction-override on appeal verdict** — v2 candidate. PRD §6.7 v1 simplification.
- **Juror reputation events on appeal-panel votes** — v2 candidate.
- **Public-case-log update on appeal verdict** — v2 candidate.
- **Federation outbound publish on `appeal_decided`** — out per ADR-014.
- **Appeal-of-appeal** — out per PRD §6.7.
- **Cross-instance jury federation** — v2 territory per PRD §17 row 5 OUT.
- **Composable diversity constraints / `no_same_endorsement_chain`** — v1.5/v2.
- **Severity-tier change ENFORCEMENT** mid-case — v2.
- **Constraint-relaxation dashboard handler** — admin-dashboard-v1 PRD owns.
- **Appeal-panel concurrency test** — v2 (FK-SHARE deadlock class).
- **Capstone test parameterised by SeverityTier** — out for v1.
- **`step_up_token` validation on every governance handler DTO** — out per planner DQ #99.
- **`closed_at` writer migration for pre-v1-JM-e cases** — out.
- **JM-c v0-literal `"jury_vote_submitted"` cleanup** — coordinated rewrite + backfill, out of JM-e scope.


---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task** (per `feedback_pr_per_phase.md`). Each task header carries a `[P]` marker iff its **IMPLEMENT** files share no path with any other `[P]`-marked task in the same cohort. Per the YAML overlap rule (Tasks 2-5 all `modifies: crates/server/tests/e2e.rs`), the e2e tasks are NOT cohort-compatible; they ship serially. Task 0 is non-`[P]` (barrier).

> **Shape G (Layer G2 push-and-exit):** §13 task bodies do NOT inline cargo invocations. Each task ends with a push to the worker branch; the impl-task subagent writes a `kind: "validate-pending"` DQ entry referencing `cargo-validate-workspace.yml` per `.claude/rules/decision-queue.md` schema-v2.

### Task 0: Pre-flight harness audit + branch verification + JM-d state confirmation

**Goal:** verify environment + branch (`phase-v1-JM-e`) + JM-d schema/handlers intact.

**FILES:**

```yaml
creates: []
modifies: []
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate all probes):**

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe -1 — submodule init (Linux/Junior worktree quirk)
git submodule status > /tmp/jm-e-task0-submodule.log 2>&1
if grep -q '^-' /tmp/jm-e-task0-submodule.log; then
  git submodule update --init --recursive > /tmp/jm-e-task0-submodule-init.log 2>&1
  echo "submodule init exit: $?"
fi

# Probe 1 — branch verification
git branch --show-current
# EXPECT: phase-v1-JM-e

# Probe 2 — JM-d state confirmation: appeal schema columns present
rg -n 'decided_at -> Nullable<Timestamptz>' crates/db_schema_file/src/schema.rs | head
rg -n 'panel_size_snapshot -> Nullable<Int4>' crates/db_schema_file/src/schema.rs | head

# Probe 3 — JM-d state confirmation: helpers present
rg -n 'pub async fn select_appeal_panel|pub async fn seat_appeal_panel' crates/api/api/src/governance/admin_assign_jury.rs | head

# Probe 4 — JM-d state confirmation: ENTRY_KIND_APPEAL_DECIDED const declared
rg -n 'pub const ENTRY_KIND_APPEAL_DECIDED' crates/db_schema/src/source/governance/governance_log.rs | head

# Probe 5 — JM-c state confirmation: process_vote shape intact
rg -n 'async fn process_vote|async fn process_appeal_vote' crates/api/api/src/governance/submit_jury_vote.rs | head
# EXPECT: process_vote present; process_appeal_vote NOT present (Task 1 introduces it)

# Probe 6 — DQ pending status (advisory)
python3 -c "import json; d=json.load(open('.claude/decision-queue.json')); print('pending:', [(e['id'], e.get('kind')) for e in d.get('pending',[])])"

# Probe 7 — PM-plugin-hooks-stable check
for h in \
  local_private_message_before_create \
  local_private_message_after_create \
  local_private_message_before_update \
  local_private_message_after_update \
  federated_private_message_before_receive \
  federated_private_message_after_receive; do
  rg -q "\"$h\"" crates/ || { echo "missing hook literal: $h"; exit 1; }
done

# Probe 8 — concurrent-PR check
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("submit_jury_vote\\.rs|tests/e2e\\.rs")) | {number, title, headRefName}'
# EXPECT: empty output

# Probe 9 — Shape G workflow YAMLs lint-clean
yamllint .github/workflows/cargo-validate-workspace.yml .github/workflows/cargo-test-e2e.yml > /tmp/jm-e-task0-yamllint.log 2>&1
echo "yamllint exit: $?"
```

**EXPECT:** Probes 0..9 exit 0 (or, for Probe 0/1, exit 1 with explicit STOP).

**No commit at Task 0** — verification only.

### Task 1: Appeal-vote tally branch in submit_jury_vote.rs + step_up_token DTO slot

**ACTION:** add the `process_appeal_vote` private fn alongside `process_vote` in `submit_jury_vote.rs`, dispatch from the role-branch in step 1; add `step_up_token: Option<String>` field on `AdminTriggerAppealRejury` DTO.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/submit_jury_vote.rs
  - crates/api/api_common/src/governance.rs
```

**IMPLEMENT (file 1 of 2):** in `crates/api/api/src/governance/submit_jury_vote.rs`,

1. Extend the step-1 assignment query (lines 151-161) to return `role` per §10.1. Replace `.count() > 0` with `.first()` selecting `jury_assignment::role`.
2. Insert step-2.5 role-branch dispatch after the step-2 case-load (line 184). If `role == JuryAssignmentRole::Appeal`, call `process_appeal_vote(...)` and return its result.
3. Add `process_appeal_vote` as a new private fn after `process_vote` (around line 651). Body: load `Appeal` row (§10.2), per-decision tally on appeal-side votes (§10.3), deadlock branch (§10.4), appeal-decide UPDATE batch (§10.5).
4. Imports: add `appeal::Appeal`, `appeal` schema table, `AppealStatus`, `JuryAssignmentRole`, `ENTRY_KIND_APPEAL_DECIDED`.

**IMPLEMENT (file 2 of 2):** in `crates/api/api_common/src/governance.rs`, add `step_up_token: Option<String>` field per §10.6 to `AdminTriggerAppealRejury` struct. Confirm `#[derive(... Default ...)]` is present.

**MIRROR:** §10.1, §10.2, §10.3, §10.4, §10.5, §10.6.

**GOTCHA (R1):** `i64::from(...)` on every `i32 ↔ i64` comparison.

**GOTCHA (R3):** after DTO field add, run `git grep -l 'AdminTriggerAppealRejury {'`. Propagate `..Default::default()` to any literal-construction site.

**GOTCHA (R7):** DTO struct change affects compile across re-exports; the workspace-check workflow's `cargo test --no-run` step picks up failures.

**GOTCHA (clippy rerun per `feedback_clippy_rerun_after_fix.md`):** if dispatch refactor unmasks an `unused-mut` or `unused-imports` lint, re-run clippy locally before push.

**GOTCHA (run_transaction inheritance):** the wrapper at submit_jury_vote.rs:118-125 wraps `process_vote`; when dispatched to `process_appeal_vote`, the inner fn inherits the same transaction.

**GOTCHA (idempotency guard for Appealed):** the existing JM-c idempotency guard (lines 242-255) lists `Appealed` as a terminal-status early-return. The role-branch dispatch MUST fire BEFORE the idempotency guard. Concrete impl: dispatch goes IMMEDIATELY after the case-load (line 184) and BEFORE the `matches!` guard. Inside `process_appeal_vote`, the guard is narrower: only `Closed | EmergencyRemove | AdminReview` are terminal; `Appealed` is the EXPECTED status.

**GOTCHA:** the v0 literal `"jury_vote_submitted"` at submit_jury_vote.rs:216 is preserved verbatim.

**Push and exit (Shape G):** push to origin/`junior/<task-slug>`; impl-task subagent writes `kind: "validate-pending"` DQ entry capturing `workflow_run_id` of `cargo-validate-workspace.yml`. No local cargo invocation.

**COMMIT MESSAGE:** `feat(v1-JM-e): appeal-vote tally branch in submit_jury_vote + step_up_token DTO slot (task 1)`

### Task 2: e2e capstone test — full lifecycle

**ACTION:** in `crates/server/tests/e2e.rs`, add `mod v1_jm_e_fixtures` AFTER `mod v1_jm_b_fixtures`. Add `seed_appealed_case_with_panel` fixture + capstone test.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
```

**IMPLEMENT (file 1 of 1):** in `crates/server/tests/e2e.rs`, Edit-with-anchor.

- **Anchor:** end of `mod v1_jm_b_fixtures` (~line 7794).
- **Content:** new `mod v1_jm_e_fixtures` containing:
  1. `pub async fn seed_appealed_case_with_panel(...)` — drives `seed_case → seed_jury_eligible_snapshots → admin_assign_jury → accept_jury_assignment(×N) → submit_jury_vote(×threshold) → request_appeal` and returns `(case_id, appeal_id, appeal_panel_person_ids)`.
  2. `appeal_panel_decides_no_action_overrides_to_advisory_label_chain` — capstone test:
     - `seed_appealed_case_with_panel` → original NoAction; appeal panel of 7 (Minor severity bumped to Moderate per `appeal.threshold_tier_bump = 1` default; `0.6 × 7 = 5` ceil = `appeal.threshold_count_snapshot = 5`)
     - 7 appeal jurors accept; 5 vote AdvisoryLabel
     - assert: `appeal.decided_at` populated, `governance_log[appeal_decided]` emitted with `original_winning_decision = NoAction` + `appeal_winning_decision = AdvisoryLabel`
     - assert: `case.status = Closed`, `case.closed_at` populated
     - assert: no NEW sanction row inserted
     - drive `appeal_window_expiry::run_appeal_window_expiry_batch(&ctx)` directly; assert no-op (already Closed)

**MIRROR:** §10.7; JM-c `submit_jury_vote_severe_panel_meets_threshold` at e2e.rs:8064 (handler-invocation pattern + federation-config-builder bootstrap at lines 8120-8127); JM-d `appeal_inside_window_succeeds_expired_rejects` at e2e.rs:4299; JM-c `submit_jury_vote_writes_appeal_window_default` at e2e.rs:8439.

**GOTCHA (R2):** `seed_jury_eligible_snapshots` MUST run BEFORE `admin_assign_jury`. Fixture wraps this.

**GOTCHA (R4):** test name lowercase snake_case.

**GOTCHA:** `request_appeal` requires the caller to be the defendant when original = NoAction; the test driver uses the target user as appeal requester.

**GOTCHA:** the fixture seeds 12 jurors (5 original + 7 appeal — appeal-panel exclusion list removes 5 originals).

**GOTCHA:** for the capstone to drive `appeal_window_expiry` directly, set `BREHON_DISABLE_APPEAL_WINDOW_JOB=1` env at bootstrap.

**Push and exit (Shape G).**

**COMMIT MESSAGE:** `test(v1-JM-e): cross-sub-phase capstone — full appeal lifecycle (task 2)`

### Task 3: e2e audit-log invariant test

**ACTION:** add audit-invariant test to `mod v1_jm_e_fixtures`. Edit-with-anchor at end of v1_jm_e_fixtures block.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
```

**IMPLEMENT:** `governance_log_sequence_matches_prd_state_machine` test fn per §10.7:

- reuse `seed_appealed_case_with_panel`
- drive a full lifecycle (similar to capstone but with original = AdvisoryLabel + appeal = RemoveContent)
- capture every `governance_log` row for `case_id` in chronological order; map to `entry_kind`
- filter out `jury_voted` and `jury_vote_submitted`
- assert filtered list equals `["report_created", "threshold_met", "jury_assigned", "panel_assembled", "case_decided", "appeal_requested", "appeal_panel_assembled", "appeal_decided"]`

**MIRROR:** §10.7; JM-c `governance_log_hash_chain_holds` at e2e.rs:277 (entry-kind/payload query shape).

**GOTCHA:** `report_created` and `threshold_met` entries depend on case being opened via `create_report`. If fixture seeds via `seed_case` directly, those entries will be missing. **Decision:** Task 3 fixture variant drives `create_report` → `threshold_met`; add `seed_case_via_report` as a sibling fixture in `v1_jm_e_fixtures` if needed.

**GOTCHA:** `payload.contains(json!(...))` requires diesel-async JSONB support; alternative client-side filter.

**Push and exit.**

**COMMIT MESSAGE:** `test(v1-JM-e): audit-log invariant — governance_log sequence per PRD §6.7 (task 3)`

### Task 4: e2e config-churn regression — appeal.window_days

**ACTION:** add config-churn regression. Edit-with-anchor at end of v1_jm_e_fixtures.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
```

**IMPLEMENT:** `config_churn_appeal_window_days_does_not_invalidate_decided_cases` test fn per §10.8.

**MIRROR:** §10.8; JM-c `v0_case_completes_under_v0_rules_after_v1_config_flip` at e2e.rs:8689 (lines 8859-8884 for `appeal.window_days` flip).

**GOTCHA:** sibling test, NOT in-place patch (per Edit-with-anchor preference).

**GOTCHA (R2):** `seed_jury_eligible_snapshots` BEFORE `admin_assign_jury` for each of three cases.

**Push and exit.**

**COMMIT MESSAGE:** `test(v1-JM-e): config-churn regression — appeal.window_days (task 4)`

### Task 5: e2e §12 security cluster — admin-visibility + spoofing-protection

**ACTION:** add bundled §12 security cluster test. Edit-with-anchor at end of v1_jm_e_fixtures.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
```

**IMPLEMENT:** `constraint_relaxation_visible_to_community_admin_orphan_case_blocks_spoofing` test fn bundles two assertions:

- **§12.2 admin-visibility:** seed a small-pool case (4 jurors, Minor severity needs 5 → R1 relaxation); admin_assign_jury writes `jury_constraint_violation_log` row. Issue Diesel query from §10.10; assert `len > 0`.
- **§12.4 spoofing-protection:** seed orphaned Decided case (`creator_id: None`); call `request_appeal` from non-defendant non-creator user; assert `Err(LemmyErrorType::NotFound)`.
- **R3 propagation:** if `AdminTriggerAppealRejury { case_id }` constructed, use `..Default::default()`.

**MIRROR:** §10.9 (spoofing); §10.10 (admin-visibility); JM-d `appeal_inside_window_succeeds_expired_rejects` at e2e.rs:4299 (orphaned-case `ModerationCaseInsertForm` direct-insert pattern).

**GOTCHA (R2):** small-pool case seeds only 4 jurors (triggers JM-b's R1 relaxation per JM-b retro §3.1).

**GOTCHA:** `eq_any(subquery)` Diesel surface — confirm at impl time; alternative `eq_any(Vec<i32>)`.

**Push and exit.**

**COMMIT MESSAGE:** `test(v1-JM-e): §12 security cluster — admin-visibility + spoofing-protection (task 5)`

### Task 6: Retro + governance-log registry marker flip

**ACTION:**

1. Flip `(pending)` → `(active)` marker on `ENTRY_KIND_APPEAL_DECIDED` row in `.claude/rules/governance-log-entry-kind-registry.md` v1-JM-a section. Update "Emitting handler" column from `v1-JM-d submit_jury_vote.rs appeal-panel vote-tally path (pending)` to `v1-JM-e submit_jury_vote.rs::process_appeal_vote (active)`. Confirm count check at bottom remains `33`.
2. Author `.claude/PRPs/reports/v1-JM-e-retro.md` per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`. Sections: TL;DR; §1 What worked; §2 Per-role signals (Advisor / Planning / Impl / BM); §3 What didn't work; §4 Per-task complexity score table; §5 Lessons promoted; §6 Confidence score; §7 Follow-up GH issue candidates (per DQ #46).
3. Cross-cutting verification:
   - `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` returns **33**.
   - `rg -n '\(pending\)' .claude/rules/governance-log-entry-kind-registry.md | rg APPEAL_DECIDED` returns empty.
   - `/brehon-verify` reports all §16a stories `[done]`.

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-JM-e-retro.md
modifies:
  - .claude/rules/governance-log-entry-kind-registry.md
```

**MIRROR:** `.claude/PRPs/reports/v1-JM-d-retro.md` for retro section structure.

**GOTCHA:** count check must remain `33`. If 34, JM-e accidentally added a const it shouldn't have.

**Push and exit.**

**COMMIT MESSAGE:** `chore(v1-JM-e): governance-log registry flip + Task 6 retro (task 6)`


---

## 14. Testing strategy

Per `IMPLEMENTATION-PLAN-v0.md §5`: integration-only for v0/v1. All tests live in `crates/server/tests/e2e.rs`.

### 14.1 Tests to add (4 new tests under `mod v1_jm_e_fixtures`)

| # | Test | Asserts |
|---|---|---|
| 1 | `appeal_panel_decides_no_action_overrides_to_advisory_label_chain` | Capstone — full lifecycle: original NoAction → appeal AdvisoryLabel → `appeal.decided_at` populated, `governance_log[appeal_decided]` emitted with both decisions, `case.status = Closed`, `case.closed_at` populated, no new sanction row, appeal_window_expiry no-op |
| 2 | `governance_log_sequence_matches_prd_state_machine` | Audit invariant — for full lifecycle, `governance_log` rows in chronological order match the PRD §6.7 + §9 expected `entry_kind` sequence (after filtering `jury_voted`/`jury_vote_submitted`) |
| 3 | `config_churn_appeal_window_days_does_not_invalidate_decided_cases` | Config-churn — case A decided pre-flip retains 7-day window; case C decided post-flip uses 30-day. Confirms LIVE-read at decision-time + binding-on-case-row per PRD §11 |
| 4 | `constraint_relaxation_visible_to_community_admin_orphan_case_blocks_spoofing` | §12 — admin Diesel query against `jury_constraint_violation_log` returns rows for cases in their community (§12.2) AND `request_appeal` on orphaned (creator_id=NULL) Decided case fails with NotFound (§12.4) |

### 14.2 Edge cases covered

- Appeal panel decides verdict different from original (capstone NoAction → AdvisoryLabel chain)
- Appeal-side per-decision tally (every JuryDecision variant in stable enum-order)
- Appeal-side deadlock (covered structurally by JM-c deadlock test which is invariant under role)
- governance_log sequence ordering
- LIVE config read at decision time + binding-on-case-row
- Community-admin Diesel query against `jury_constraint_violation_log`
- Orphaned-case spoofing-protection negative test

### 14.3 Edge cases NOT covered (out of v1-JM-e scope)

- Appeal-of-appeal (rejected per PRD §6.7)
- Concurrent appeal-panel votes (FK-SHARE deadlock class — v2)
- Appeal-panel deadlock with appeal-context payload field (`panel_kind: "appeal"`) — structurally same as JM-c's
- Severity-tier-parameterised capstone (Moderate / Severe variants) — out for v1
- Federation outbound on appeal_decided (out per ADR-014)
- step_up_token validation behaviour (v2 — v1 ignore-only)

### 14.4 Pre-existing tests preserved

- All JM-c tests (6 tests at e2e.rs:8064-9031) — preserved verbatim
- All JM-d-shipped tests (e.g. `appeal_inside_window_succeeds_expired_rejects` at e2e.rs:4299) — preserved
- `governance_log_hash_chain_holds` at e2e.rs:277 — preserved
- `report_to_modlog_golden_path`, `sanction_notice_round_trip` — preserved
- All JM-b severity-tier tests — preserved

---

## 15. Validation commands (DoD)

> **Shape G plan — DoD is per-workflow, not inline cargo.** Per `.claude/PRPs/templates/plan.template.md` §15.6 + the JM-e brief §2.3 mandate.

### 15.1 Per-task workspace check (Shape G)

For every §13 impl task (Tasks 1-5):

- **DoD entry:** `cargo-validate-workspace.yml` on `junior/<task-slug>` SHA `<sha>` → `conclusion: "success"`
- **Validation command:** `gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --json conclusion,databaseId --jq '.[0]'`
- **EXPECT:** `{"conclusion": "success", "databaseId": <id>}`

The workflow runs `cargo check --workspace --features full`, `cargo clippy --workspace --features full --no-deps -- -D warnings`, and `cargo test --no-run -p lemmy_server --test e2e` per `.github/workflows/cargo-validate-workspace.yml:90-95`. R6 + R7 are encoded.

### 15.2 Phase 2 e2e (per task post-finalize-merge)

After each impl task's worker branch finalize-merges into `phase-v1-JM-e`, the advisor surfaces the **Phase 2 e2e local-vs-dispatch user gate** per `advisor-orchestrator.md`:

- **(a) local:** `cargo test -p lemmy_server --test e2e --features full -- --test-threads=1` on laptop in `run_in_background`; ~26 min wall-clock; zero billed.
- **(b) dispatch:** `gh workflow run cargo-test-e2e.yml --repo barrie-cork/lemmy --ref phase-v1-JM-e`; ci-watcher polls; ~26 min billed.

Plan-side DoD: e2e exit code 0; failure path → §G4 classifier on log slice.

### 15.3 Cross-cutting verification (Task 6 retro time)

- [ ] `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` returns `33` (unchanged)
- [ ] `rg -n '\(pending\)' .claude/rules/governance-log-entry-kind-registry.md | rg APPEAL_DECIDED` returns empty (marker flipped)
- [ ] `rg -n 'TODO\(v1-sponsor-liability-d\)' crates/api/api/src/governance/submit_jury_vote.rs` still returns the JM-c TODO
- [ ] R1: every `i32 ↔ i64` comparison in appeal-tally branch uses `i64::from(...)`
- [ ] R6: clippy invocations in `cargo-validate-workspace.yml` use `--no-deps -- -D warnings`
- [ ] No edits to files outside §11 list
- [ ] Every JM-e §16a story is `[done]`
- [ ] No new const declarations under `crates/db_schema/src/source/governance/`
- [ ] `step_up_token` field is exclusively on `AdminTriggerAppealRejury`
- [ ] The four new e2e tests use R2 discipline

### 15.4 Migration round-trip — N/A

JM-e ships no migrations.

### 15.5 ADR / OQ compliance verification

- [ ] ADR-010 honored — appeal panel snapshots are read from `appeal` row, NOT recomputed from live config
- [ ] ADR-013 honored — exhaustive `CaseStatus` match preserved
- [ ] ADR-014 honored — no federation outbound on `appeal_decided`
- [ ] ADR-015 honored — `actor_pseudonym = Some(juror_pseudonym)` on `appeal_decided`
- [ ] PRD §17 row 5 OUT honored — no sanction-row mutation; no public_case_log update; no juror reputation_event; no federation outbound

### 15.6 DoD per workflow (canonical Shape G shape)

**Phase 1 (workspace check):**

- Workflow: `.github/workflows/cargo-validate-workspace.yml`
- Branch: `phase-v1-JM-e`
- Expected `conclusion`: `"success"`

**Phase 2 (e2e):**

- Workflow: `.github/workflows/cargo-test-e2e.yml` (only on user dispatch)
- OR local: `cargo test -p lemmy_server --test e2e --features full -- --test-threads=1` on laptop
- Branch: `phase-v1-JM-e`
- Expected: all tests pass

### 15.7 Manual validation snippets (advisor-side smoke only — non-binding)

> Per `feedback_features_full_p_crate_incompatible.md`: never `-p <crate>` + `--features full`; use `--workspace --features full`.

```bash
yamllint .github/workflows/cargo-validate-workspace.yml .github/workflows/cargo-test-e2e.yml
echo "yamllint exit: $?"

cargo check --workspace --features full
echo "exit: $?"
```

These are advisor-side only; do not count as plan §16 acceptance criteria. Per DQ #67 resolution.

---

## 16. Acceptance criteria

- [ ] All 6 tasks (Task 0..5 + Task 6 retro) committed
- [ ] §15.1 (`cargo-validate-workspace.yml`) `conclusion: "success"` after every impl task push
- [ ] §15.2 (Phase 2 e2e — local or dispatch) all tests pass
- [ ] §15.3 (cross-cutting verification — 9 boxes) all ticked
- [ ] §15.5 (ADR / OQ compliance — 5 boxes) all ticked
- [ ] §16a stories — all four `[done]`
- [ ] No edits to files outside §11 list
- [ ] Retro committed per Task 6
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-JM-e-verify.md` shows all stories ✓

---

## 16a. Stories (independently-testable behaviour units)

Per `.claude/PRPs/templates/plan.template.md` §16a + `feedback_story_grain_checkpoint.md` + `feedback_brehon_verify_pre_merge.md`. Four stories — one per behaviourally distinct unit.

### Story 1: Appeal-vote tally fires `appeal_decided` and closes the case

- **Composing tasks:** Task 1, Task 2
- **Checkpoint workflow:** `cargo-validate-workspace.yml` on Task 2's worker branch → `conclusion: "success"`; Phase 2 e2e for `appeal_panel_decides_no_action_overrides_to_advisory_label_chain` → exit 0
- **Expected output (local):** `1 passed; 0 failed`
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/governance/submit_jury_vote.rs` contains `async fn process_appeal_vote(`
  - `crates/api/api/src/governance/submit_jury_vote.rs` references `ENTRY_KIND_APPEAL_DECIDED` and `appeal::table` and `AppealStatus::Decided`
  - `crates/api/api/src/governance/submit_jury_vote.rs` contains `JuryAssignmentRole::Appeal` match arm
  - `crates/server/tests/e2e.rs` contains `mod v1_jm_e_fixtures` block with `pub async fn seed_appealed_case_with_panel(`
  - `crates/server/tests/e2e.rs` contains `async fn appeal_panel_decides_no_action_overrides_to_advisory_label_chain(`

### Story 2: governance_log sequence matches PRD §6.7 state machine

- **Composing tasks:** Task 3 (depends on Tasks 1+2 shipped)
- **Checkpoint workflow:** `cargo-validate-workspace.yml` → `success`; Phase 2 e2e for `governance_log_sequence_matches_prd_state_machine` → pass
- **Expected output (local):** `1 passed; 0 failed`
- **Brief-Scope outputs to verify:**
  - `crates/server/tests/e2e.rs` contains `async fn governance_log_sequence_matches_prd_state_machine(`
  - Test body asserts entry-kind ordered prefix `["report_created", "threshold_met", "jury_assigned", "panel_assembled", "case_decided", "appeal_requested", "appeal_panel_assembled", "appeal_decided"]` (verifiable via `rg` for the literal vec)

### Story 3: appeal.window_days config churn binds at decision time

- **Composing tasks:** Task 4
- **Checkpoint workflow:** `cargo-validate-workspace.yml` → `success`; Phase 2 e2e for `config_churn_appeal_window_days_does_not_invalidate_decided_cases` → pass
- **Expected output (local):** `1 passed; 0 failed`
- **Brief-Scope outputs to verify:**
  - `crates/server/tests/e2e.rs` contains `async fn config_churn_appeal_window_days_does_not_invalidate_decided_cases(`
  - Test body asserts at least one case with `window_diff.num_days() == 7` AND another with `num_days() == 30` (verifiable via `rg` for the integer literals)

### Story 4: §12 security cluster — admin-visibility + spoofing-protection

- **Composing tasks:** Task 5 (requires Task 1's `step_up_token` field for R3 propagation)
- **Checkpoint workflow:** `cargo-validate-workspace.yml` → `success`; Phase 2 e2e for `constraint_relaxation_visible_to_community_admin_orphan_case_blocks_spoofing` → pass
- **Expected output (local):** `1 passed; 0 failed`
- **Brief-Scope outputs to verify:**
  - `crates/server/tests/e2e.rs` contains `async fn constraint_relaxation_visible_to_community_admin_orphan_case_blocks_spoofing(`
  - Test body queries `jury_constraint_violation_log` (verifiable via `rg` for the table name)
  - Test body asserts `is_err()` on a `request_appeal` with non-defendant caller on orphaned case (verifiable via `rg` for `creator_id: None` + `is_err()` proximity)
  - `crates/api/api_common/src/governance.rs` contains `pub step_up_token: Option<String>,` on `AdminTriggerAppealRejury`

> **Verification mapping:** `/brehon-verify` iterates this section, runs each Story's Checkpoint, and confirms each Brief-Scope output exists + matches structurally.


---

## 17. Completion checklist

- [ ] Task 0 audit complete (all probes confirmed)
- [ ] Task 1..5 committed
- [ ] Task 6 retro committed
- [ ] §15 validation green at every gate (Phase 1 workspace + Phase 2 e2e per user-gate choice)
- [ ] §16a stories all `[done]`
- [ ] PR opened by BM session against `governance-v0`
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-JM-e-verify.md` shows all stories ✓
- [ ] Post-merge phase branch retained for retro reads
- [ ] Follow-up GH issues filed per Task 6 candidate list (sanction-override-on-appeal v2; step_up_token enforcement v2; admin-dashboard query)

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `process_vote` step-1 query refactor (count→first) breaks original-jury tests | LOW | HIGH | §10.1 + Task 1 GOTCHA; Phase 1 workspace-check before push; original-jury tests at e2e.rs:8064-9031 must still pass |
| Appeal-row `panel_size_snapshot` NULL on appeal-tally → process breach surfaces too late | MED | MED | §10.2 defensive `LemmyErrorType::Unknown` |
| JOIN query on `jury_vote × jury_assignment` admits original-jury votes into appeal tally | MED | HIGH | §10.3 GOTCHA + Story 2 audit-invariant catches sequence misalignment |
| e2e.rs Edit-with-anchor causes worker-hang on the >9000-line file | MED | MED | Each task adds ONE test fn at ONE anchor; cohort-dispatch degraded to serial by YAML overlap rule |
| Complexity score `>8` rejected by advisor (split-mandated) | HIGH | LOW | §5.2 — DQ #98 filed; if split, this plan becomes `jm-e-1.plan.md` (Tasks 1-2) and `jm-e-2.plan.md` (Tasks 3-5+retro) is sibling |
| `step_up_token` DTO field site (DQ #99) re-litigated by user | LOW | LOW | §4.1 names `AdminTriggerAppealRejury` as v1 site |
| GH-Actions minutes budget exceeded by 5 Phase-1 + 5 Phase-2 e2e | MED | MED | Phase 2 e2e local-default per JM-d retro §5; user picks dispatch only on audit-trail need |
| Audit-invariant test fixture diverges from PRD §6.7 sequence (`threshold_met` not emitted on direct `seed_case`) | MED | MED | Task 3 GOTCHA — `seed_case_via_report` sibling fixture covers report-created path |
| PMD #126 — DQ ID collision from concurrent SL-d / rep-tuning-r3 work | LOW | LOW | next-id calc spans archives + live; advisor monitors at finalize-merge |
| Junior worker pre-pushes break finalize-merge | LOW | LOW | Junior daemon's finalize step is post-Shape-G correct (per JM-d retro §1.1) |

---

## 19. Notes

### 19.1 Planner DQs filed

- **DQ #98** (`from: "planner"`, `kind: "blocker"`, `answered_by: null`) — complexity-score-split decision per §5.2. Question: "Complexity score 15 exceeds 8 — split `v1-jury-mechanics-e` into `jm-e-1` (Tasks 1-2) + `jm-e-2` (Tasks 3-5) + retro, or proceed as one plan?". Options: split / proceed.
- **DQ #99** (`from: "planner"`, `kind: "blocker"`, `answered_by: "planner"`) — pre-seeded planner-resolved entry per `decision-queue.md` Recipe 2. Question: "Which endpoint(s) should carry the v1 `step_up_token: Option<String>` DTO slot per PRD §12.3?". Options: (a) `AdminTriggerAppealRejury`, (b) every governance handler DTO, (c) defer to v2. Answer: "(a) — `AdminTriggerAppealRejury` is the closest v1 endpoint to a 'mid-case admin override'; v2 step-up-aware admin endpoints can graft onto the same DTO shape without a wire migration. Per advisor pre-decision review (brief §2.1.e). Pre-resolved as `answered_by: planner` because the brief notes 'both [single-task and separate-task placement] are fine'."

### 19.2 Self-resolved planner findings (LESSON candidates)

- **Lesson files referenced in the brief but not present in `.claude/lessons/`.** Verified: `feedback_junior_worker_e2e_edit_hang.md`, `feedback_advisor_watchpoint_specificity.md`, `feedback_build_what_tests_exercise.md`, and `pattern_cargo_feature_flag_propagation.md` are NOT present. The disciplines are present in PMD per the brief's framing; the plan honors the disciplines without depending on file paths. Recommendation for retro: advisor mirror-promotes those PMD-side lessons into `.claude/lessons/` for next sub-phase's planner.
- **JM-d plan §16a Stories was not present.** JM-d predates the §16a-mandatory mark per `feedback_schema_changing_spec_retrofit_question.md`.

### 19.3 Pre-existing pending DQ entries

- **DQ #87 + DQ #86** (both `kind: validate-pending`, `from: "impl"`) — pre-existing pending entries from issue-96 cluster work (JM-d era — not JM-e-related). Advisory-only; advisor's polling loop is responsible for mutating them.

### 19.4 Out-of-scope follow-ups (Task 6 retro candidates per DQ #46)

- Sanction-override on appeal verdict — v2 candidate
- `step_up_token` enforcement on `AdminTriggerAppealRejury` — v2 candidate
- Admin-dashboard view for `jury_constraint_violation_log` — admin-dashboard-v1 candidate
- v1-JM-c v0-literal `"jury_vote_submitted"` cleanup — coordinated rewrite + backfill
- Appeal-panel concurrency test — v2 candidate
- Appeal-panel jury_reliability deltas — v2 candidate
- Public-case-log update on appeal verdict — v2 candidate
- Appeal-panel deadlock with `panel_kind: "appeal"` payload field — implicitly covered; explicit appeal-deadlock test future-coverage candidate

### 19.5 Confidence bands

- High (8/10): appeal-tally branch is structurally a mirror of JM-c's tally with appeal-row snapshots
- Moderate (6/10): audit-invariant test sequence — depends on `seed_case_via_report` reproducing all upstream entry-kinds
- Low (5/10): the §5 complexity score will trip the split-DQ; plan ships under proceed-as-one
- High (9/10): no new schema, no migrations, no new const declarations

---

## 20. Confidence score

- **Plan correctness:** 8/10 — patterns mirror JM-c tally + JM-c §10.4 deadlock + JM-d appeal-row reads
- **Cargo budget:** 10/10 — Shape G (cargo runs off-box; budget non-binding)
- **Test coverage:** 7/10 — four behaviourally distinct tests; deadlock-on-appeal coverage relies on JM-c's role-invariant deadlock test
- **Story-grain decomposition:** 8/10 — every task maps to exactly one story
- **Plan-shape conformance:** 9/10 — 20-section schema followed literally; §15.6 Shape G shape adopted; §16a mandatory present; §13 per-task FILES YAML block on every task

---

_Plan author: planning subagent (Junior `jm-e-planning-1`, 2026-04-30). Plan committed on `junior/role-planning-v1-jury-mechanics-e-plan-see-claude-prps-briefs-jm-e-planning-1-md-65`; finalize step pushes to that branch; advisor session merges via the standard sub-phase flow into `governance-v0` where JM-e's BM-task then cuts `phase-v1-JM-e`. Two planner DQs raised at commit time: DQ #98 (split-or-proceed; advisor blocking) and DQ #99 (step_up_token DTO site; planner-resolved with citation). Confidence 8/10. Plan ships under proceed-as-one assumption pending DQ #98 resolution._

LESSON: e2e.rs at >9000 lines remains the worker-hang risk surface for any phase that adds 4+ new tests. The §13 partitioning rule "one test per task, one Edit-with-anchor per Edit" needs to be load-bearing in any future plan that touches the file class; consider mirror-promoting `feedback_junior_worker_e2e_edit_hang.md` from PMD into `.claude/lessons/` if not already done so the planner subagent at next sub-phase doesn't re-derive the constraint.
