# JM-e planning brief

**Written**: 2026-04-30 by advisor session (laptop) for Junior dispatch on EliteDesk.
**Subagent target**: `planning` (Opus 4.7, color purple — see `.claude/agents/planning.md`).
**Worktree**: Junior cuts `junior/jm-e-planning-1` from `governance-v0` per concurrency-1 default; the planning subagent commits the plan file and pushes back to `governance-v0` at finalize.
**Authority anchor**: PRD §17 row 5 (`v1-JM-e — Security hardening + integration test capstone`) is the canonical scope source. JM-d plan §20 (`Sub-phase stub (v1-JM-e)`) lists the six concrete deliverables.

---

## 1. Role + dispatch line

`[role:planning] v1-jury-mechanics-e plan — appeal-vote tally + integration capstone + step-up/spoofing/admin-visibility hardening`

The actual `mcp__junior-brehon__create_task` description is a single line under 100 chars per `.claude/rules/advisor-orchestrator.md` Junior task description template:

```
[role:planning] v1-jury-mechanics-e plan — see .claude/PRPs/briefs/jm-e-planning-1.md
```

Everything else lives in this brief.

---

## 2. Scope — what to produce

Produce **one plan file** at `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` for sub-phase **v1-JM-e**, the JM-PRD capstone. The plan covers PRD §17 row 5 in full: appeal-vote tally activation, the cross-sub-phase integration test capstone, the audit-log invariant test, the mid-flight config-churn regression extension, the step-up auth DTO stub, the appeal-rights spoofing-protection check, and the constraint-relaxation admin-visibility check.

### 2.1 Six concrete deliverables (per JM-d plan §20 + PRD §17 row 5)

The plan's §13 task list MUST cover all six:

a. **Appeal-vote tally** in `crates/api/api/src/governance/submit_jury_vote.rs`. Detect `jury_assignment.role = JuryAssignmentRole::Appeal` rows for the case, compute the **bumped-tier threshold** (per PRD §6.3, default `appeal.threshold_tier_bump = 1` from Minor→Moderate→Severe), tally only Appeal-role votes, on threshold-met:
- set `appeal.decided_at = now()`,
- emit `governance_log` entry of kind `appeal_decided` (the const at `crates/db_schema/src/source/governance/governance_log.rs:182` is already declared by JM-a; JM-e is the **fire** site),
- compose with the existing case state machine per PRD §6.7 (`Decided → Appealed → Decided (again, with appeal verdict) → Closed`).

b. **Cross-sub-phase integration test (the capstone)** — full lifecycle: report → JM-b assign jury (with constraint relaxation) → JM-c vote (Decided) → JM-d request_appeal (bounded window, reporter-rights branch) → JM-d auto-rejury → JM-e appeal-vote tally → JM-d appeal-window-expiry bg job closes the case. The test asserts every state transition and every governance_log emission in the expected order.

c. **Audit-log invariant test** — for every case in the capstone fixture, assert `governance_log` entries for that case_id form the expected ordered set per PRD §6.7 + §9 (no duplicates, no missing entries, no out-of-order rows). Generalises the JM-c `governance_log_hash_chain_holds` test from row-shape to sequence-shape.

d. **Mid-flight config-churn regression** — extends JM-c test 5 (`config_get_int_cascade_resolves_*` family / `config_parity_round_trip`) with appeal-window churn: a case decided at `appeal.window_days = 7`, then config flipped to `appeal.window_days = 1` mid-window, must continue to honour the snapshot/non-snapshot rule from PRD §9.1 step 9 (window_days is **read at decision-time, not snapshotted**, so cases decided after the flip use the new value but already-decided cases keep their bound `appeal_window_expires_at`).

e. **Step-up auth stub** (PRD §12.3) — v1 ships `step_up_token: Option<String>` DTO slot on the relevant severity-tier-mutation endpoint(s); behaviour is **ignore the field** (v1 baseline) but the wire shape exists so v2 can activate enforcement without a DTO migration. Per ADR-010.

f. **Constraint-relaxation admin-visibility check** (PRD §12.2) — adds the queryable surface (`jury_constraint_violation_log` rows) cross-checked from a community-admin role view. Plan §13 must specify the exact query pattern + the test that asserts a community-admin can read the relaxation rows for cases in their community while a non-admin cannot.

Plus the spoofing-protection verification (PRD §12.4 — verify `case.creator_id IS NOT NULL` path in `request_appeal` is exercised by a negative test for an orphaned case). This is mostly a **test-only** task because the §12.4 PRD note states "no spoofing surface" by construction; JM-e adds the missing **negative test** that proves the construction.

### 2.2 Scope boundary — what JM-d already shipped (off-limits for JM-e)

JM-d shipped on `governance-v0` (verified at brief-write time, advisor read of origin/governance-v0 HEAD `5c9119688`):

- **Schema columns JM-e depends on are ALL present**: `appeal.decided_at`, `appeal.requester_role`, `appeal.panel_size_snapshot`, `appeal.threshold_count_snapshot`, `jury_assignment.role`, `moderation_case.winning_decision`, `moderation_case.panel_size_snapshot`, `moderation_case.quorum_snapshot`, `moderation_case.threshold_count_snapshot`, `moderation_case.appeal_window_expires_at`. JM-e MUST NOT redeclare or migrate these.
- **Enums JM-e depends on**: `AppealRequesterRole`, `JuryAssignmentRole`, `SeverityTier`, `CaseStatusTier`. All declared in JM-a/JM-d.
- **Handlers JM-d shipped**: `request_appeal` (rewrite — bounded window + reporter-rights + auto-rejury), `admin_trigger_appeal_rejury`, the appeal-window-expiry background job + scheduler tick.
- **Submit_jury_vote shape JM-c shipped**: 9-step combined handler with step-9 `appeal_window_expires_at` write. JM-e's appeal-vote tally is a **new branch in submit_jury_vote** that fires when `jury_assignment.role = Appeal` is detected at step 2 — it does NOT rewrite the original-jury tally branch JM-c shipped.
- **ENTRY_KIND consts already declared**: `ENTRY_KIND_APPEAL_DECIDED` (`governance_log.rs:182`, declared by JM-a per PRD §8.5). JM-e fires it; JM-e MUST NOT add a new const for this.
- **Carry-forward concurrent dependency** (note for §13 Task 0 audit): JM-d Task 5 (appeal-window-expiry background job, commit `ee7d4b1c6`) is on the worker branch `junior/role-impl-task-v1-jm-d-task-5-...-54` and NOT yet finalize-merged into `governance-v0`. Schema-wise this does not block JM-e (Task 5 is functional code, not schema). The advisor anticipates Task 5 lands before JM-e impl-tasks dispatch; JM-e plan should reference Task 5's `closed_at` write semantics (PRD §6.7 / JM-d plan §10.6) but not duplicate the bg-job pattern.

**Hard out-of-scope for JM-e** (per PRD §2 + §17 row 5 risk-LOW posture):

- Cross-instance jury (parked, v2 — OQ-V1-JM-04).
- Composable constraints / `no_same_endorsement_chain` (v1.5/v2-candidate per PRD §2 OUT).
- Severity-tier change *enforcement* (PRD §12.3 says v1 *rejects* mid-case severity changes at the API layer; JM-e ships only the DTO slot for the eventual v2 step-up, not the v2 enforcement).
- Federation of `appeal_decided` AP types (per ADR-014 — fork-only AP types, sanction-related only; appeal events are local-only governance_log entries).
- The dashboard query implementation surface (admin-dashboard-v1 PRD owns the dashboard; JM-e only adds the `jury_constraint_violation_log` query the dashboard *can* call).

### 2.3 Plan file deliverable shape

- **One file:** `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md`.
- **Follow `.claude/PRPs/templates/plan.template.md` 20-section schema literally.** §16a Stories is mandatory (JM-e is post-spec-kit-adoption; not optional).
- **Shape G applies — JM-e is the FIRST plan under Shape G.** Per `.claude/PRPs/templates/plan.template.md` §15.6 note ("v1-JM-e onward"), §15 DoD MUST use the **per-workflow** shape:
  - Each task's DoD entry names a workflow path on the phase-branch SHA + the expected `conclusion: "success"`. Example: `cargo-validate-workspace.yml` on `phase-v1-JM-e` SHA `<sha>` → `conclusion: "success"`.
  - The validation command is `gh run list --repo barrie-cork/lemmy --branch phase-v1-JM-e --limit 1 --json conclusion,databaseId --jq '.[0]'` — NOT inline cargo invocations.
  - Inline cargo commands are forbidden in §15 (forward-only Shape G adoption per `feedback_schema_changing_spec_retrofit_question.md`).
- **§5 complexity score with breakdown table per `feedback_complexity_score_pre_split.md`**. The advisor pre-estimates `>8` because of (a) e2e edits (3 points each — capstone test + audit-log invariant test + config-churn regression all touch `crates/lemmy_server/tests/e2e.rs`, the >8000-line file flagged by `feedback_junior_worker_e2e_edit_hang.md`), (b) ~5 impl tasks above 5 (= 0 points by table — only counts above 5), (c) crates touched (~3-4: `lemmy_api`, `lemmy_api_common`, `lemmy_db_views_*`, `lemmy_server`), (d) ADR-affecting decisions (~1 — the bumped-tier threshold-rounding rule may touch ADR-010 if not already resolved), (e) cargo budget peak under Shape G = 0. Mechanical estimate: **9-12**, threshold-tripping. Planner MUST file a `pending` DQ entry from `from: "planner", kind: "blocker"` BEFORE shipping the plan, asking "complexity N exceeds threshold — split into `jm-e-1` (appeal-vote tally + capstone test) + `jm-e-2` (audit invariant + config-churn regression + step-up + admin-visibility), or proceed?"
- **§16a Stories** — every story names (a) composing §13 tasks, (b) a checkpoint command in the Shape-G workflow shape, (c) Brief-Scope outputs to verify (file:line + symbol) per `feedback_advisor_watchpoint_specificity.md`. Concept-only watchpoints rejected at advisor-side review.
- **§4 watchpoints** — every entry cites a specific table, file, or `schema.rs` line. No abstract concepts.
- **Explicit scope-boundary section** in §6 ("Relationship to other v1-JM sub-phases") listing the JM-d §16a Stories that are `[done]` (note: JM-d plan predates §16a so this list will be JM-d §13 tasks completed instead — the planner notes the schema-pattern divergence).

**Commit only the plan file.** Do not author Rust code, do not open PRs, do not touch any file under `crates/`, `migrations/`, or `tests/`. Junior finalize pushes the plan-file commit to `junior/jm-e-planning-1`; advisor merges it into `governance-v0` after DoD smoke + watchpoint-specificity gates pass and the user approves.

---

## 3. Required reading (in order, before drafting any plan section)

1. `.claude/agents/planning.md` — your subagent contract; defines model, tools, "Before you start (always)" sequence, plan-content discipline cross-references.
2. `.claude/PRPs/templates/plan.template.md` — canonical Brehon plan template (20 sections incl. §15.6 Shape-G DoD and §16a Stories). Structure is load-bearing.
3. `.claude/PRPs/prds/v1-jury-mechanics.prd.md` — the parent PRD. Read §6 (Appeals — first-class), §9.1 (submit_jury_vote 9-step pseudocode — JM-e adds the appeal-tally branch parallel to JM-c's original-jury tally), §10 (knobs — `appeal.threshold_tier_bump`, `appeal.window_days`, `appeal.panel_size_multiplier`, `appeal.panel_size_floor_increment`), §11 (backwards compat with v0 cases — informs the config-churn regression), §12 (Security — §12.1 capability check, §12.2 admin-visibility, §12.3 step-up stub, §12.4 spoofing protection), §17 row 5 (your scope), §17.1 (cross-PRD sequencing — JM-e parallel-safe with SL-d / rep-tuning-r3 finished), §17.4 (preflight DQ inheritance — cite ids, no inline compensation).
4. `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` — predecessor plan; §13 task list (what JM-d shipped), §15 DoD commands (the prior pre-Shape-G shape — JM-e diverges to Shape G), §16 acceptance criteria, §17 completion checklist, §18 risks (the `winning_decision` write at submit_jury_vote that JM-e tally branch sits parallel to), §19 notes (DQ #51 MIRROR drift, DQ #52 winning_decision storage — JM-d resolved both; JM-e inherits the column-storage approach), §20 (your stub).
5. `.claude/PRPs/plans/v1-jury-mechanics-c.plan.md` — JM-c plan; §9 step-9 `appeal_window_expires_at` write JM-e tally branch must respect, §13 Task 5 mid-flight config-churn regression test JM-e extends, §16a Stories shape **if present** (JM-c also predates §16a — note in §6 of JM-e plan). The JM-c capstone test `v0_case_completes_under_v0_rules_after_v1_config_flip` is the test JM-e extends with appeal-side cross-sub-phase assertions.
6. `crates/api/api/src/governance/submit_jury_vote.rs` — the primary file JM-e impl-tasks modify. Read entire file at planning time. Note the JM-c handler shape, the step-2 case-load + juror-validation block (where the role detection branch lives), the step-5 threshold check (where the bumped-tier threshold computation must compose), the step-9 appeal-window-expires-at write block (boundary — appeal-tally branch must NOT re-fire step 9 because the appeal panel decision does not re-bound an appeal window). Per `feedback_junior_worker_e2e_edit_hang.md`: file size matters; `submit_jury_vote.rs` is ~600 lines (Edit-safe); `crates/lemmy_server/tests/e2e.rs` is >8000 lines (Edit-DANGER).
7. `crates/db_schema_file/src/schema.rs` — confirm at planning time:
   - `appeal.decided_at -> Nullable<Timestamptz>` (line ~155-167 in the appeal table block).
   - `appeal.requester_role -> AppealRequesterRole` (same block).
   - `jury_assignment.role -> JuryAssignmentRole` (line ~518-535).
   - `moderation_case.winning_decision` (verify presence — JM-d Task 1 added).
   - `moderation_case.appeal_window_expires_at` (JM-a §8.1).
   If any of these is absent at planning time, raise a DQ entry `kind: "blocker"` immediately before writing §13 — JM-e cannot be planned until JM-d schema is on `governance-v0`.
8. `crates/db_schema/src/source/governance/governance_log.rs` — confirm `ENTRY_KIND_APPEAL_DECIDED: &str = "appeal_decided"` const at line ~182. JM-e fires it; JM-e MUST NOT add a const.
9. `crates/db_schema/src/source/governance/appeal.rs` — read for the `Appeal` row's update-form / insert-form shapes the appeal-tally branch must use to set `decided_at`.
10. `crates/lemmy_server/tests/e2e.rs` — locate the JM-c `v0_case_completes_under_v0_rules_after_v1_config_flip` test, the JM-c `config_get_int_cascade_resolves_*` family, the JM-d `governance_log_hash_chain_holds` test, the JM-d `submit_jury_vote_writes_appeal_window_default` test, and the JM-d `submit_jury_vote_writes_appeal_window_live_config` test — JM-e's three new e2e edits (capstone + audit invariant + config-churn extension) sit alongside these. **Read by Grep + targeted Read; do NOT load the whole file.**
11. `.claude/decision-queue.json` — read ALL pending entries (advisor confirmed 0 pending on `governance-v0` at brief-write time, but the planner re-reads at planning-time). Read ALL resolved entries with `from: "planner"` referencing v1-JM-c/d/e to inherit the prior planning context (DQ #51, #52, #50, #48, #45, etc.).
12. `.claude/runlog/v1-JM-d-runlog.md` — JM-d's append-only ledger; read for handoff context.
13. **Glob `.claude/lessons/feedback_*.md` and Read every file whose name keyword matches:** `appeal`, `tally`, `e2e`, `edit_hang`, `concurrency`, `lock`, `deadlock`, `InsertForm`, `cargo`, `features-full`, `features_full_p_crate`, `wrapper-script`, `clippy`, `test-style`, `complexity_score`, `pre_queue`, `watchpoint`, `dod`, `validate`, `shape_g`, `validate_pending`, `governance_log`, `audit`, `config_cascade`, `config_churn`, `principles_not_rules`, `retro_not_report`, `four_role`, `schema_changing_spec_retrofit`. That's the lessons-corpus discipline per `planning.md` step 3.
14. **Glob `.claude/lessons/reference_*.md` and Read every file whose name keyword matches:** `nutomic`, `governance_log_entry_kind_registry`, `phase_branch`, `branch_manager`, `worktree`.
15. `.claude/rules/governance-log-entry-kind-registry.md` — JM-e flips the `(pending)` marker on `appeal_decided` to `(active)`. Verify the registry's count-check pattern; JM-e §15 cross-cutting must include the count-check (33 consts → still 33; just the marker flips).
16. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — read ADR-010 (no retroactive invalidation; the config-churn regression test is its load-bearing audit) and any OQ-V1-JM-* with `JM-e` in resolution. Also OQ-V1-JM-06 (reporter-eligibility scope — JM-d shipped; JM-e regression-tests it).
17. `.claude/PRPs/briefs/jm-d-planning-1.md` — read as exemplar for brief structure and constraint language. JM-e brief mirrors that shape.

---

## 4. Constraints (hard rules — violating any of these is a process breach)

### 4.1 Plan-content discipline

- **Shape G DoD shape (forward-only).** §15 MUST use the per-workflow shape per `.claude/PRPs/templates/plan.template.md` §15.6: workflow path + phase-branch SHA + `conclusion: "success"`. Inline cargo invocations are forbidden in §15 of this plan. Per `feedback_schema_changing_spec_retrofit_question.md` — Shape G is forward-only; JM-e is the first plan to ship under it.
- **§16a Stories mandatory** (NOT optional). Every story names composing §13 tasks, a Shape-G checkpoint command (workflow + branch + conclusion), Brief-Scope outputs to verify (file:line + symbol), and lists the §13 IMPLEMENT entries it covers. Per `.claude/PRPs/templates/plan.template.md` §16a + `.claude/commands/brehon-verify.md`. Phantom-completion catch surface.
- **§4 watchpoints cite specific files / tables / `schema.rs` lines**, never abstract concepts. Per `feedback_advisor_watchpoint_specificity.md`.
- **§5 complexity score breakdown table mandatory** per `feedback_complexity_score_pre_split.md`. If `score > 8`, planner MUST file a `pending` DQ entry from `from: "planner", kind: "blocker"` asking split-or-proceed BEFORE shipping the plan. The advisor pre-estimate for JM-e is **9-12**, near-certainly threshold-tripping. Plan should expect to be split into two sub-plans (`jm-e-1` + `jm-e-2`) unless the user approves the proceed-as-one path on the DQ.
- **e2e edit discipline** per `feedback_junior_worker_e2e_edit_hang.md`. Tasks that Edit `crates/lemmy_server/tests/e2e.rs` (>8000 lines) MUST be split: one task per Edit, each task adds **one** test function via Edit-with-anchor (not a multi-test bulk Edit). The capstone test, the audit-invariant test, and the config-churn regression are three distinct Edits → three distinct §13 tasks (not one bundled "tests" task). Each such task gets a separate worker branch + separate validate-pending DQ.
- **Cargo feature/flag propagation** per `pattern_cargo_feature_flag_propagation.md` (PMD-promoted pattern, 3+ occurrences). Never combine `-p <crate>` with `--features full` in any DoD command (per `feedback_features_full_p_crate_incompatible.md`); use `--workspace --features full`. Even though §15 is Shape-G workflow-shape, the planner-side DoD smoke test (advisor side, pre-merge) and any §15.7 manual validation snippets respect this.
- **Build only what tests exercise** per PMD #14 / `feedback_build_what_tests_exercise.md` (PMD-promoted, 3+ occurrences). When the appeal-vote-tally branch lands a new view-crate field that JM-e tests don't seed (e.g. an `appeal_decided_at` projection on a `governance_case` view), drift-stub it as `None` rather than shipping the LEFT JOIN. Document the cleanup path in §19 Notes.
- **R-rule inheritance from JM-b/c/d retros** — every R1-R7 from JM-b/c retros applies to JM-e. R6 in particular (clippy `--no-deps -- -D warnings`) — under Shape G this lives in the workflow YAML, not in §15, so the planner verifies the workflow YAML (`.github/workflows/cargo-validate-workspace.yml` + `cargo-test-e2e.yml`) carries `--no-deps` before naming the workflow in §15.
- **Wrapper-script flag silence** per `feedback_wrapper_script_flag_silence.md`. Under Shape G this is non-binding (cargo runs in the workflow, not via wrappers), but if §15.7 manual-validation snippets are included, they MUST cite Linux .sh wrappers (the laptop and EliteDesk paths both use bash) and verify wrapper $@ passthrough.

### 4.2 Decision-queue discipline (per `.claude/rules/decision-queue.md`)

- **Attribution integrity.** Any DQ entry seeded by the planning subagent uses `answered_by: "planner"` (forward-looking pre-resolved entries) or `answered_by: null` (genuinely needs advisor input). NEVER `"advisor"`, `"user"`, `"impl-self-resolved"`. Subjects on planner DQ commits MUST start with `chore(decision-queue): planner raised DQ #<id> — <slug>`.
- **Mid-task DQ commits push immediately**, not at finalize, per `decision-queue.md` §"Mid-task visibility (Junior worktrees)". Push to `junior/jm-e-planning-1` (this worktree's branch); advisor's polling loop fetches all branches.
- **Boundary-of-judgment — when to STOP and queue rather than guess:**
  - If the bumped-tier threshold rule (PRD §6.3) is ambiguous for any panel-size / decision-tier combination JM-e must support → queue a planner DQ for the tie-break rule.
  - If `appeal.decided_at` should be set by the **submit_jury_vote appeal-tally branch** OR by a **separate `submit_appeal_vote` handler** → queue a planner DQ. The PRD §17 row 5 wording ("appeal-vote tally" in submit_jury_vote.rs per JM-d plan §20) suggests the same handler with a role-branch, but the planner verifies by reading the submit_jury_vote.rs entry-point and the existing role-detection patterns.
  - If the `step_up_token` DTO slot affects multiple endpoints and the v1 ignore-behaviour requires touching code in more than one file, the planner queues a DQ to enumerate sites and confirm per `feedback_insertform_default_propagation.md`.
  - If the `jury_constraint_violation_log` query for community-admin role visibility (PRD §12.2) requires a new view-crate or a new SQL view, the planner queues a DQ asking which approach (view-crate column drift-stub vs new SQL view vs handler-side query) — per PMD #14 `build only what tests exercise`.
  - If JM-d Task 5 (appeal-window-expiry bg job) is NOT yet on `governance-v0` at planning time AND JM-e plan §13 Task 1 needs to reference its `closed_at` write semantics, queue a planner DQ asking: "block JM-e Task 1 dispatch until Task 5 lands on governance-v0, or proceed with a pre-condition note in §6?" The advisor's pre-brief read confirmed Task 5 was on the worker branch only at brief-write time; the planner re-checks at planning time.

### 4.3 File ownership

- **Touch only:**
  - `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` (CREATE).
  - `.claude/decision-queue.json` (APPEND new pending or planner-resolved entries).
- **Do NOT edit anything under** `crates/`, `migrations/`, `tests/`, `docs/`, `.claude/PRPs/prds/`, `.claude/PRPs/reports/`, `.claude/agents/`, `.claude/rules/`, `.claude/commands/`, `.github/workflows/`, or any other plan file in `.claude/PRPs/plans/`. Even if the existing workflow YAMLs (`cargo-validate-workspace.yml`, `cargo-validate-features-full.yml`, `cargo-test-e2e.yml`) need modification for JM-e validation, the planner records the request as a §13 task or a planner DQ — does NOT edit YAML directly.
- **One commit at finalize:** `feat(plan): v1-JM-e sub-phase plan` (matching JM-c/JM-d planner-task commit subjects).

### 4.4 Schema-first discipline

- **Read `crates/db_schema_file/src/schema.rs` BEFORE designing §13.** Confirm every column JM-e depends on exists. Required columns (advisor verified at brief-write time on origin/governance-v0 HEAD `5c9119688`):
  - `appeal.decided_at`, `appeal.requester_role`, `appeal.panel_size_snapshot`, `appeal.threshold_count_snapshot`.
  - `jury_assignment.role`.
  - `moderation_case.winning_decision`, `moderation_case.appeal_window_expires_at`, `moderation_case.severity_tier`, `moderation_case.status_tier`, `moderation_case.panel_size_snapshot`, `moderation_case.quorum_snapshot`, `moderation_case.threshold_count_snapshot`.
- If ANY of these is absent at planning time, file a `kind: "blocker"` DQ before writing §13. JM-e cannot be planned until JM-d schema is on `governance-v0`.

### 4.5 Cross-cutting from PMD-promoted patterns

- **PMD #126** (`automation candidate: gov-v0 → phase-branch merge-up with DQ renumbering`) — if JM-e's phase branch diverges from `governance-v0` post-cut (e.g. parallel SL-d or rep-tuning work lands on `governance-v0` while JM-e is in-flight), DQ ID collisions are likely. Plan §18 Risks should include a row for this (Likelihood LOW, Impact LOW, Mitigation: gov-v0 IDs canonical, phase-side renumbered, citations updated by sed in conflict-resolution commit). The script at `homeserver:scripts/brehon/merge-up-phase.sh` is candidate-not-built; the planner does NOT depend on the script existing.
- **PMD #14** (`build only what tests exercise`) — drift-stub the `jury_constraint_violation_log` query if the dashboard-visibility test does not seed all relaxation reason_codes, per the §4.1 lesson `feedback_build_what_tests_exercise.md`.

### 4.6 Attribution integrity reminder

The only valid `answered_by` labels for a Junior planning subagent are `"planner"` or `null`. Never `"advisor"`, `"user"`, `"impl-self-resolved"`. Any commit with `answered_by: "advisor"` from this subagent triggers the catch-fire procedure in `advisor-orchestrator.md`.

---

**Lean / advisor-side tip (not a constraint):** JM-e is the JM-PRD capstone — the test-density payoff for getting the plan right is high, and the marginal cargo budget under Shape G is zero. The biggest plan-shape risk is the §5 complexity score; JM-e is dense enough that the planner should expect a split-or-proceed DQ to fire. If the user authorises proceed-as-one, the §13 task ordering still benefits from cohort-dispatch (`[P]` markers per `brehon-fork/.claude/rules/brehon-cohort-dispatch.md`) on the three e2e tasks (capstone test, audit-invariant test, config-churn regression) — they share no IMPLEMENT files (each Edits one new test function in `e2e.rs`) and can run as a 3-task `[P]` cohort under Shape G's parallel validate-pending semantics, cutting wall-clock from ~3× cargo-validate-workspace cycles to ~1×.

A second observation: the appeal-vote tally in `submit_jury_vote.rs` is a **branch addition**, not a rewrite. JM-c's 9-step shape stays intact for the original-jury path; JM-e adds a parallel branch detected at step 2 (`if jury_assignment.role == Appeal`). The cleanest §13 design likely puts this as Task 1 (BARRIER, not `[P]` — it's the one source-edit dependency) with the three e2e tasks as a `[P]` cohort following.

A third observation: `step_up_token: Option<String>` (PRD §12.3) is genuinely a one-line DTO addition with v1 ignore-behaviour — it does NOT need its own complexity-bearing §13 task. The planner can fold it into Task 1 (the submit_jury_vote tally task) or a separate small Task 2; both are fine.

---

_Brief author: advisor session (laptop CWD `C:\Users\barri\Developer\homeserver`, brehon-fork checkout `C:\Users\barri\Developer\brehon-fork` on `governance-v0` @ `5c9119688`, 2026-04-30). Brief committed on `governance-v0` before Junior planning task is queued. Per the advisor-orchestrator clarify gate, advisor runs `/brehon-clarify .claude/PRPs/briefs/jm-e-planning-1.md` after this brief commits and pushes; planning task only queues after every clarify-DQ entry is resolved._
