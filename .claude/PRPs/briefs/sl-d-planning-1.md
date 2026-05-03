# SL-d planning brief

**Written**: 2026-05-03 by advisor session (laptop, brehon-fork CWD) for Junior dispatch on EliteDesk.
**Subagent target**: `planning` (Opus 4.7, color purple — see `.claude/agents/planning.md`).
**Worktree**: Junior cuts `junior/sl-d-planning-1` from `governance-v0` per concurrency-1 default; the planning subagent commits the plan file and pushes back to `governance-v0` at finalize.
**Authority anchor**: `v1-sponsor-liability.prd.md` §15 row 4 (`v1-SL-d — submit_jury_vote mutation + apply_sponsor_liability split`) is the canonical scope source. **An advisor-side handoff document at `homeserver:.claude/advisor-context-v1-SL-d.md` describes a broader scope (compute branch + cascade + 33→38 registry expansion + orphan-case guard) that may bundle work the PRD's §15 table assigns across SL-a/SL-b/SL-c/SL-d.** The planner MUST surface this divergence as a `kind: "blocker"` DQ before authoring §13 — see §4.2 below.

---

## 1. Role + dispatch line

`[role:planning] v1-sponsor-liability-d plan — submit_jury_vote mutation + apply_sponsor_liability split (compute/fire)`

The actual `mcp__junior-brehon__create_task` description is a single line under 100 chars per `.claude/rules/advisor-orchestrator.md` Junior task description template:

```
[role:planning] v1-sponsor-liability-d plan — see .claude/PRPs/briefs/sl-d-planning-1.md
```

Everything else lives in this brief.

---

## 2. Scope — what to produce

Produce **one plan file** at `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md` for sub-phase **v1-SL-d**. The plan covers PRD §15 row 4: `submit_jury_vote` mutation + `apply_sponsor_liability` compute/fire split.

**A scope-boundary question must be resolved before §13 is authored** — see §4.2 below. The planner cannot proceed past §6 of the plan template until the advisor answers the scope DQ. This brief sets out two scope hypotheses (PRD-aligned vs advisor-context-aligned) and the planner picks based on the advisor's answer.

### 2.1 Two scope hypotheses

The PRD and the advisor-context describe overlapping but distinct scope. The planner MUST surface this as a DQ; advisor decides which to use as the scope basis for §13.

**Hypothesis A — PRD §15 row 4 (narrow scope):** SL-d ships the `apply_sponsor_liability` split + the `Decided → SponsorLiabilityPending` transition in `submit_jury_vote.rs`. PRD §15 row 4 explicitly **depends on Phases 1, 3 (SL-a + SL-c)**. SL-d does NOT include:
- the schema migration (SL-a — three new `CaseStatus` variants + two new `moderation_case` columns + Issue #24 partial index + 12 governance_config seeds);
- the `revoke_endorsement` handler (SL-b);
- the scheduler / `sponsor_liability_grace.rs` module (SL-c);
- the e2e suite (SL-e).

Under Hypothesis A, the deliverables are: (a) split `crates/api/api/src/governance/sponsor_liability.rs::apply_sponsor_liability` into `compute_sponsor_liability` (sets state, no reputation event) + `fire_sponsor_liability` (gated on scheduler), (b) wire `Decided → SponsorLiabilityPending` transition into `submit_jury_vote.rs` step where v0 currently fires liability immediately, (c) update affected match sites for `SponsorLiabilityPending` (per ADR-013 exhaustiveness — but only at the `submit_jury_vote.rs` boundary, not the full §3.3 enumeration which is SL-a's responsibility), (d) sponsor-notification stub (§5.1.1 if applicable). Estimated 4-6 §13 tasks; complexity ~5-7.

**Hypothesis B — advisor-context homeserver-side (broad scope):** SL-d ships the "compute branch slice" of the Sponsor-Liability lane: cascade-handler edits, registry expansion 33→38 (5 new ENTRY_KIND_* consts), endorsement-revocation cascade with orphan-case guard, micros-scaled config seeds, v0-compat sponsor-liability-knob churn regression. This bundles work that the PRD's §15 table distributes across SL-a (schema + 5 new ENTRY_KIND consts + governance_config seeds), SL-b (revoke endorsement handler — the cascade-handler), SL-c (scheduler), SL-d (compute branch), and SL-e (regression test).

Under Hypothesis B, the deliverables match the homeserver advisor-context §1 paragraph: graft-point compute branch + registry 33 → 38 + cascade implementation with orphan-case guard + micros-scaled config seeds + v0-compat sponsor-liability-knob churn regression. Estimated 8-12+ §13 tasks; complexity 9-13 (split-or-proceed DQ near-certain).

### 2.2 Source-of-truth files for either hypothesis

The planner reads BOTH and surfaces the conflict; the planner does NOT silently pick one.

**For Hypothesis A (PRD-aligned):**

- `.claude/PRPs/prds/v1-sponsor-liability.prd.md` §1 (vision/goals); §3 (CaseStatus extensions — three new variants); §5 (`POST /endorsement/revoke` — but this is SL-b's, not SL-d's); §6 (Grace-Window Scheduler — SL-c's); §9.1 (`apply_sponsor_liability` split — **SL-d's**); §9.3 (`submit_jury_vote` mutation pointer — **SL-d's**); §11 (backwards compat); §15 row 4 (the canonical scope row); §17 cross-cutting impact (the 5 new ENTRY_KIND consts, but assigned to SL-a per PRD §15 row 1).
- `crates/api/api/src/governance/sponsor_liability.rs` — the file SL-d splits.
- `crates/api/api/src/governance/submit_jury_vote.rs:105` — `ALL_JURY_DECISIONS` const + iteration sites at lines 349 + 787.
- `governance-log-entry-kind-registry.md` rule's `### sponsor-liability-v1 (reserved — §17 of PRD enumerates 5 new kinds)` block — these consts land in SL-a per PRD §15 row 1, NOT SL-d.

**For Hypothesis B (advisor-context-aligned):**

- `homeserver:.claude/advisor-context-v1-SL-d.md` — the homeserver-side advisor-context that describes the broad scope. **Read at brief-write time by advisor; cited in §3 below for planner reference.**

### 2.3 Plan file deliverable shape

Regardless of hypothesis chosen:

- **One file:** `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md`.
- **Follow `.claude/PRPs/templates/plan.template.md` 20-section schema literally.** §16a Stories is mandatory (SL-d is post-spec-kit-adoption; not optional).
- **Shape G applies — SL-d is the second plan under Shape G** (after JM-e). §15 DoD MUST use the per-workflow shape (workflow path + phase-branch SHA + `conclusion: "success"`). Inline cargo invocations are forbidden in §15.
- **§5 complexity score with breakdown table** per `feedback_complexity_score_pre_split.md`. If `score > 8`, planner MUST file a `pending` DQ entry from `from: "planner", kind: "blocker"` BEFORE shipping the plan, asking "complexity N exceeds threshold — split into `sl-d-1` + `sl-d-2`, or proceed?" Hypothesis-A pre-estimate: 5-7 (no split-DQ needed). Hypothesis-B pre-estimate: 9-13 (split-DQ near-certain).
- **§16a Stories** — every story names (a) composing §13 tasks, (b) a checkpoint command in the Shape-G workflow shape, (c) Brief-Scope outputs to verify (file:line + symbol) per `feedback_advisor_watchpoint_specificity.md`. Concept-only watchpoints rejected at advisor-side review.
- **§4 watchpoints** — every entry cites a specific file, table, or `schema.rs` line. No abstract concepts.
- **Explicit scope-boundary section in §6** ("Relationship to other v1-SL sub-phases") listing what each adjacent SL phase ships (SL-a/b/c/e) and confirming SL-d does not duplicate them.

**Commit only the plan file.** Do not author Rust code, do not open PRs, do not touch any file under `crates/`, `migrations/`, or `tests/`. Junior finalize pushes the plan-file commit to `junior/sl-d-planning-1`; advisor merges it into `governance-v0` after DoD smoke + watchpoint-specificity gates pass and the user approves.

### 2.4 Scope boundary — what prior SL-* and JM-* sub-phases shipped (off-limits for SL-d)

At brief-write time (advisor read of `governance-v0` HEAD `48c4f379d`):

- **SL lane prior phases NOT YET SHIPPED.** As of 2026-05-03, no `phase-v1-SL-*` branch has merged into `governance-v0`. The PRD §15 table lists all 5 SL phases as `pending`. **This is a critical pre-condition for SL-d.** If SL-a (schema + enum migration + 5 new ENTRY_KIND consts + governance_config seeds) is NOT yet on `governance-v0` at planning time, SL-d cannot be planned — it would depend on schema columns and enum variants that don't exist. **The planner re-verifies this at planning time and files a `kind: "blocker"` DQ if SL-a is not on trunk.**
- **v0 sponsor-liability code IS shipped** (Phase 5b, see `project_brehon_governance_platform.md` Phase 5b summary): `crates/api/api/src/governance/sponsor_liability.rs::apply_sponsor_liability` exists and runs in the same `run_transaction` as the sanction insert. SL-d's job is to **split** this into compute + fire halves. The v0 code is the MIRROR ref for the split.
- **Existing ENTRY_KIND consts at SL-d planning time:** 33 total per `governance-log-entry-kind-registry.md`. Two of these are sponsor-liability v0 consts (`ENTRY_KIND_SPONSOR_LIABILITY_APPLIED` at line 122; `ENTRY_KIND_SPONSOR_LIABILITY_CLAMPED` at line 123 of `crates/db_schema/src/source/governance/governance_log.rs`). The 5 new SL-v1 consts (`SPONSOR_LIABILITY_PENDING`, `SPONSOR_LIABILITY_FIRED`, `SPONSOR_LIABILITY_ESCAPED`, `ENDORSEMENT_REVOKED`, `RESTORATION_COMPLETED`) are **reserved** in the registry rule but NOT yet declared. **Per PRD §15 row 1, these consts land in SL-a, not SL-d.** The advisor-context's claim that SL-d expands the registry 33 → 38 contradicts the PRD. The planner surfaces this as part of the scope-boundary DQ.
- **JM lane status:** v1-JM-e SHIPPED 2026-05-02 via PR #107 (governance-v0 merge HEAD `ea9e0d309`; tip `48c4f379d` after CI workflow disable commits). All JM PRD §17 row 5 deliverables complete; `ENTRY_KIND_APPEAL_DECIDED` flipped `(active)` (registry stays at 33 — JM-e flipped a marker, did not add a const).

**Hard out-of-scope for SL-d** under either hypothesis (per PRD §2 OUT + §15):

- Cross-instance sponsor-liability federation (deferred to v2 per ADR-014).
- Step-up auth enforcement (PRD §12.3 — v1 reserves slot only).
- Restoration completion endpoint (PRD §7.4 — owned by restorative-mechanics-v1 PRD).
- Notification UX richness beyond the existing Lemmy notification + dashboard listing (PRD §13 OQ-V1-SL-03 — v3 polish).
- The dashboard write surface for grace-window keys (PRD §14 — owned by admin-dashboard-v1 PRD).

### 2.5 Pre-existing line-drift in advisor-context (advisor flagged at brief-write time)

The `homeserver:.claude/advisor-context-v1-SL-d.md` §4 watchpoint #1 cites `submit_jury_vote.rs:782` for ALL_JURY_DECISIONS const. This is **drift** — at HEAD `48c4f379d`, `ALL_JURY_DECISIONS` is declared at line **105**; line 782 is mid-iteration body inside the appeal-vote tally branch (one of the iteration sites is at 787). The const itself is at 105 (declared) and iterated at 349 and 787. The planner uses the verified line numbers, NOT the advisor-context's cited line. Surfaced for transparency; clarify-DQ may formalise.

---

## 3. Required reading (in order, before drafting any plan section)

1. `.claude/agents/planning.md` — your subagent contract; defines model, tools, "Before you start (always)" sequence, plan-content discipline cross-references.
2. `.claude/PRPs/templates/plan.template.md` — canonical Brehon plan template (20 sections incl. §15.6 Shape-G DoD and §16a Stories). Structure is load-bearing.
3. `.claude/PRPs/prds/v1-sponsor-liability.prd.md` — the parent PRD. Read **in full** the first time. Specifically:
   - §1 (vision/goals — Brehon `athgabál` framing)
   - §2 (scope IN/OUT — note SL-d is the v0→v1 split, not the schema add)
   - §3 (CaseStatus extensions — three new variants; ADR-013 enumeration discipline)
   - §5 (POST /endorsement/revoke — SL-b's, but SL-d's compute split must compose with it)
   - §6 (Grace-window scheduler — SL-c's, but SL-d's fire half is what the scheduler invokes)
   - §7 (restoration interaction — escape semantics)
   - §8 (DB & migration changes — SL-a's, but cite for column refs)
   - §9.1 (`apply_sponsor_liability` split — **SL-d's primary deliverable** under Hypothesis A)
   - §9.3 (`submit_jury_vote` mutation pointer to JM PRD §9.1 — **SL-d's other deliverable**)
   - §11 (backwards compat incl. documented behavioural changes)
   - §15 (implementation phases — confirms SL-d row 4 scope)
   - §17 (cross-cutting impact — confirms 5 new ENTRY_KIND consts assigned to SL-a, not SL-d)
4. **Homeserver advisor-context** at `C:\Users\barri\Developer\homeserver\.claude\advisor-context-v1-SL-d.md` — the handoff letter from JM-e close. **Read for advisor texture; do NOT treat as authoritative if it conflicts with the PRD.** If it conflicts (and §2.1 above says it does), surface as a `kind: "blocker"` DQ before authoring §13.
5. `.claude/rules/governance-log-entry-kind-registry.md` — registry invariant (33 consts current; reserved section for sponsor-liability-v1 enumerates 5 new). The 5 reserved consts are explicitly assigned to SL-a in the PRD §15 table; SL-d does not add to the registry under Hypothesis A.
6. `crates/api/api/src/governance/sponsor_liability.rs` — the v0 module SL-d splits. Read entire file at planning time. The v0 `apply_sponsor_liability` shape is the MIRROR ref for the compute/fire split (PRD §9.1).
7. `crates/api/api/src/governance/submit_jury_vote.rs` — the file SL-d mutates. **Verify at planning time** that:
   - `ALL_JURY_DECISIONS` const is at line ~105 (per advisor-side brief-write-time check; line 782 from the advisor-context is drift).
   - The v0 `apply_sponsor_liability` call site (where the immediate fire happens) is the boundary SL-d converts into a `Decided → SponsorLiabilityPending` transition.
   - The Phase 5b `apply_sponsor_liability(...)` invocation pattern (which Watch / GOTCHA does the v0 code rely on?).
8. `crates/db_schema/src/source/governance/governance_log.rs` — confirm 33 consts present; locate v0 `ENTRY_KIND_SPONSOR_LIABILITY_APPLIED` (line 122) + `_CLAMPED` (line 123). Under Hypothesis A SL-d does NOT add to this file.
9. `crates/db_schema_file/src/enums.rs` — verify `CaseStatus` enum at lines ~393-408 has 9 v0 variants. Per PRD §3.1 SL-a adds 3 more (`SponsorLiabilityPending`, `SponsorLiabilityFired`, `SponsorLiabilityEscaped`). Under Hypothesis A SL-d does NOT add to this file; under Hypothesis B SL-d may need to.
10. `.claude/decision-queue.json` — read ALL pending entries (advisor confirms 0 pending on `governance-v0` at brief-write time, but the planner re-reads at planning time). Read ALL resolved entries with `from: "planner"` referencing v1-JM-d/e and any v1-SL-* entries to inherit the prior planning context (the latter likely none — SL lane has not yet shipped).
11. `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` — JM-e plan as exemplar for first-Shape-G-plan structure; specifically §15 per-workflow DoD shape, §16a Stories shape, §5 complexity score breakdown table, the `[P]` cohort markers + FILES YAML blocks in §13.
12. `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` — predecessor sub-phase plan from JM lane (SL lane has none yet); useful for cross-lane patterns. Read §10 patterns (handler MIRROR refs), §13 task shape, §15 DoD (pre-Shape-G — for contrast).
13. `.claude/PRPs/briefs/jm-e-planning-1.md` — exemplar planning brief (most recent post-spec-kit-adoption + Shape G).
14. **Glob `.claude/lessons/feedback_*.md` and Read every file whose name keyword matches:** `sponsor`, `liability`, `endorsement`, `cascade`, `orphan`, `transaction`, `multi_write`, `apply`, `compute`, `fire`, `notify`, `e2e`, `edit_hang`, `cargo`, `features-full`, `features_full_p_crate`, `wrapper-script`, `clippy`, `test-style`, `complexity_score`, `pre_queue`, `watchpoint`, `dod`, `validate`, `shape_g`, `validate_pending`, `governance_log`, `audit`, `config_cascade`, `config_churn`, `micros`, `micros_scaled`, `principles_not_rules`, `retro_not_report`, `four_role`, `schema_changing_spec_retrofit`, `parallel_cohort`, `cohort_yaml`, `read_canonical`. That is the lessons-corpus discipline per `planning.md` step 3.
15. **Glob `.claude/lessons/reference_*.md` and Read every file whose name keyword matches:** `nutomic`, `governance_log_entry_kind_registry`, `phase_branch`, `branch_manager`, `worktree`.
16. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — read **ADR-010** (no retroactive invalidation; relevant for the v0-compat regression), **ADR-013** (CaseStatus enum-exhaustiveness — every match site updates explicitly), **ADR-015** (pseudonymisation; every governance_log payload uses `actor_pseudonym_helper` + `scrub_json`). Also OQ-025 (sponsor-liability v1 — resolution into this PRD) and any OQ-V1-SL-* with SL-d in resolution.
17. `.github/workflows/cargo-validate-workspace.yml` + `.github/workflows/cargo-validate-features-full.yml` + `.github/workflows/cargo-test-e2e.yml` — the existing Shape-G workflows the plan §15 references. Verify they carry `--no-deps -- -D warnings` (R6 inheritance) and `--features full` where applicable; if any workflow is missing a required flag, raise a planner DQ requesting the workflow YAML edit as a §13 task. The planner does NOT edit YAMLs in this brief — see §4.3 file ownership.
18. **DQ #67 resolved (per user 2026-04-27)** — workflow YAML DoD dry-run discipline. Honoured by SL-d §15: do NOT prescribe `act` invocations; do NOT require pre-merge full dry-run. Plan §15 dry-run for workflow-referenced entries is satisfied by `yamllint <workflow>` exit 0 OR `gh workflow view --repo barrie-cork/lemmy --ref phase-v1-SL-d <workflow>` returning a parsed view. Real validation happens on first push to the phase branch (Shape G's premise).

---

## 4. Constraints (hard rules — violating any of these is a process breach)

### 4.1 Plan-content discipline

- **Shape G DoD shape (forward-only).** §15 MUST use the per-workflow shape per `.claude/PRPs/templates/plan.template.md` §15.6. Inline cargo invocations are forbidden in §15. Per `feedback_schema_changing_spec_retrofit_question.md`.
- **§16a Stories mandatory** (NOT optional). Every story names composing §13 tasks, a Shape-G checkpoint command, Brief-Scope outputs to verify (file:line + symbol), and lists the §13 IMPLEMENT entries it covers. Per `.claude/PRPs/templates/plan.template.md` §16a + `.claude/commands/brehon-verify.md`.
- **§4 watchpoints cite specific files / tables / `schema.rs` lines**, never abstract concepts. Per `feedback_advisor_watchpoint_specificity.md`.
- **§5 complexity score breakdown table mandatory** per `feedback_complexity_score_pre_split.md`. Pre-estimate depends on hypothesis — see §2.1.
- **e2e edit discipline** per `feedback_junior_worker_e2e_edit_hang.md`. Tasks that Edit `crates/lemmy_server/tests/e2e.rs` (>10,000 lines now per JM-e §3.4 commentary) MUST be split: one task per Edit, each task adds **one** test function via Edit-with-anchor (not a multi-test bulk Edit). Each such task gets a separate worker branch + separate validate-pending DQ.
- **Cargo feature/flag propagation** per `pattern_cargo_feature_flag_propagation.md` (PMD-promoted, 3+ occurrences). Never combine `-p <crate>` with `--features full` in any DoD command (per `feedback_features_full_p_crate_incompatible.md`); use `--workspace --features full`. Even though §15 is Shape-G workflow-shape, the planner-side DoD smoke test (advisor side, pre-merge) and any §15.7 manual validation snippets respect this.
- **Build only what tests exercise** per PMD #14 / `feedback_build_what_tests_exercise.md` (PMD-promoted, 3+ occurrences). When the compute/fire split lands a new view-crate field that SL-d tests don't seed, drift-stub it as `None` rather than shipping the LEFT JOIN. Document the cleanup path in §19 Notes.
- **R-rule inheritance from JM-b/c/d/e retros** — every R1-R7 from JM retros applies to SL-d. R6 in particular (clippy `--no-deps -- -D warnings`) — under Shape G this lives in the workflow YAML, not §15, so the planner verifies the workflow YAML carries `--no-deps` before naming the workflow in §15.
- **Wrapper-script flag silence** per `feedback_wrapper_script_flag_silence.md`. Under Shape G this is non-binding for §15 DoD (cargo runs in the workflow), but if §15.7 manual-validation snippets are included, they MUST cite Linux `.sh` wrappers and verify wrapper `$@` passthrough.
- **Multi-write handlers transactionality** per `feedback_multi_write_handlers_need_transactions.md`. The compute/fire split **must preserve atomicity**: the Decided→Pending transition + governance_log emission + any sibling writes belong inside a `run_transaction` closure. The PRD §5.3 + §6.2 use this pattern; SL-d adopts it.
- **Micros-scaled config math** per `feedback_brehon_config_micros_scaled.md` (promoted JM-e Task 3). If SL-d touches sponsor-liability config keys (e.g. `liability.threshold_*` if the compute branch reads any), every such key follows the micros pattern (× 1,000,000) with strict `>` comparisons. Under Hypothesis A, SL-d does NOT seed config rows (SL-a does); under Hypothesis B SL-d may seed and must respect the pattern.

### 4.2 Decision-queue discipline (per `.claude/rules/decision-queue.md`)

- **Attribution integrity.** Any DQ entry seeded by the planning subagent uses `answered_by: "planner"` (forward-looking pre-resolved entries) or `answered_by: null` (genuinely needs advisor input). NEVER `"advisor"`, `"user"`, `"impl-self-resolved"`. Subjects on planner DQ commits MUST start with `chore(decision-queue): planner raised DQ #<id> — <slug>`.
- **Mid-task DQ commits push immediately**, not at finalize, per `decision-queue.md` §"Mid-task visibility (Junior worktrees)". Push to `junior/sl-d-planning-1` (this worktree's branch); advisor's polling loop fetches all branches.
- **Boundary-of-judgment — mandatory DQs the planner MUST file:**

  **MANDATORY DQ #1 — scope-hypothesis resolution (BEFORE §13 authoring).** The planner MUST file this as the first action after reading the PRD and the homeserver advisor-context. Question shape: "SL-d scope: PRD §15 row 4 (narrow — `apply_sponsor_liability` split + `Decided → SponsorLiabilityPending` transition only) OR homeserver advisor-context (broad — bundles SL-a/b/c/d into one sub-phase)?" Options: `(A) PRD-aligned narrow scope` / `(B) advisor-context broad scope` / `(C) intermediate scope — planner proposes ` (with concrete sub-list cited from PRD §15 + advisor-context §1). The planner does NOT proceed past §6 of the plan template until advisor answers. `kind: "blocker"`, `from: "planner"`, `answered_by: null`.

  **MANDATORY DQ #2 — SL-a precondition check.** The planner re-reads `governance-v0` at planning time. If SL-a (schema migration + 3 new `CaseStatus` variants + 5 new ENTRY_KIND consts + 12 governance_config seeds + Issue #24 partial index) is NOT yet on `governance-v0`, file a `kind: "blocker"` DQ asking "block SL-d planning until SL-a lands on governance-v0, or proceed with a pre-condition note in §6?" Without SL-a's schema, SL-d's compute branch references columns / enum variants that don't exist. `from: "planner"`, `answered_by: null`.

  **Other DQs as evidence demands:**
  - If the PRD §9.1 compute/fire split shape is ambiguous (which side owns the governance_log emission? compute or fire?) → planner DQ.
  - If the v0 `apply_sponsor_liability` invocation pattern at the `submit_jury_vote.rs` call site requires touching code in more than the boundary-call file (e.g. propagation through `crates/api/api_common/src/governance.rs` DTOs) → planner DQ.
  - If the §16a story shape needs more than one independently-testable behaviour for SL-d (e.g. compute split + transition split = two stories) → planner DQ asking advisor to confirm story decomposition.
  - If the planner discovers any contradiction between PRD §11 (backwards compat) and the v0 code's actual fire-time semantics → planner DQ.

### 4.3 File ownership

- **Touch only:**
  - `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md` (CREATE).
  - `.claude/decision-queue.json` (APPEND new pending or planner-resolved entries).
- **Do NOT edit anything under** `crates/`, `migrations/`, `tests/`, `docs/`, `.claude/PRPs/prds/`, `.claude/PRPs/reports/`, `.claude/agents/`, `.claude/rules/`, `.claude/commands/`, `.github/workflows/`, or any other plan file in `.claude/PRPs/plans/`. Even if existing workflow YAMLs need modification for SL-d validation, the planner records the request as a §13 task or a planner DQ — does NOT edit YAML directly.
- **One commit at finalize:** `feat(plan): v1-SL-d sub-phase plan` (matching JM-c/JM-d/JM-e planner-task commit subjects).

### 4.4 Schema-first discipline

- **Read `crates/db_schema_file/src/schema.rs` BEFORE designing §13.** Confirm every column SL-d depends on exists (or does not exist, if SL-a is unshipped). Required columns:
  - `moderation_case.grace_expires_at` (PRD §8.1 — SL-a lands).
  - `moderation_case.liability_escape_reason JSONB` (PRD §8.1 — SL-a lands).
  - `CaseStatus::SponsorLiabilityPending` enum value (PRD §3.1 — SL-a lands).
  - `CaseStatus::SponsorLiabilityFired` enum value (SL-a).
  - `CaseStatus::SponsorLiabilityEscaped` enum value (SL-a).
- If ANY of these is absent at planning time AND the chosen scope hypothesis depends on them, file a `kind: "blocker"` DQ before writing §13.

### 4.5 Cross-cutting from PMD-promoted patterns

- **PMD #126** (`automation candidate: gov-v0 → phase-branch merge-up with DQ renumbering`) — if SL-d's phase branch diverges from `governance-v0` post-cut (e.g. parallel rep-tuning or admin-dashboard work lands), DQ ID collisions are likely. Plan §18 Risks should include a row for this (Likelihood LOW, Impact LOW, Mitigation: gov-v0 IDs canonical, phase-side renumbered).
- **PMD #14** (`build only what tests exercise`) — drift-stub any new view-crate field SL-d's tests don't seed.

### 4.6 Attribution integrity reminder

The only valid `answered_by` labels for a Junior planning subagent are `"planner"` or `null`. Never `"advisor"`, `"user"`, `"impl-self-resolved"`. Any commit with `answered_by: "advisor"` from this subagent triggers the catch-fire procedure in `advisor-orchestrator.md`.

---

**Lean / advisor-side tip (not a constraint):** SL-d is the FIRST sub-phase in the SL lane to plan from this CWD; there is no SL-c-complete file to lean on (the lane has not yet shipped). The PRD §15 phase table is the authoritative scope source. The homeserver-side advisor-context describes a different scope and the divergence is large enough that the planner must surface it as DQ #1, not silently pick. The advisor (this session) leans toward Hypothesis A (PRD-aligned narrow scope) because (a) the PRD is the document the user signed off on, (b) SL-a/b/c being separate sub-phases means the work is already planned-to-be-split — bundling them would re-litigate that decision, (c) SL-d-as-narrow keeps the cargo footprint small (4-6 tasks, ~5-7 complexity, no split-DQ). But the user's pause-before-running-the-skill behaviour and the explicit advisor-context scope suggest they may have re-thought scope post-PRD; that's the user's call, not the planner's.

A second observation: under Hypothesis A, SL-d's primary test surface is the e2e regression that proves the v0 immediate-fire path is correctly converted to the Decided→Pending transition. That's likely **one** new e2e test (compute_split_transitions_to_pending_instead_of_immediate_fire), not three. Hypothesis B's test surface is broader (cascade test + orphan-case test + v0-compat sponsor-liability-knob churn — three tests) which is what drives Hypothesis B's complexity higher.

A third observation: Hypothesis B's "33 → 38 registry expansion" claim contradicts the registry rule's reserved-section assignment of those 5 consts to SL-a (per PRD §15 row 1). If the user picks Hypothesis B at DQ #1, the planner must either (a) acknowledge SL-d is bundling SL-a's registry work (which means re-naming the sub-phase from SL-d to SL-a/b/c/d-bundle), or (b) explicitly refuse the registry expansion in SL-d on the grounds that it's already assigned. Either path needs user input — this is the kind of judgment call the advisor cannot make alone.

---

_Brief author: advisor session (laptop CWD `C:\Users\barri\Developer\brehon-fork` on `governance-v0` @ `48c4f379d`, 2026-05-03). Brief committed on `governance-v0` before Junior planning task is queued. Per the advisor-orchestrator clarify gate, advisor runs `/brehon-clarify .claude/PRPs/briefs/sl-d-planning-1.md` after this brief commits and pushes; planning task only queues after every clarify-DQ entry is resolved._
