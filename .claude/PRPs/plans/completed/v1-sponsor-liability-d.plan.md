# Plan: v1-sponsor-liability-d — `apply_sponsor_liability` split + `submit_jury_vote` mutation closing the SL producer side

> **Shape G plan** — SL-d ships under Shape G (Layer G2 push-and-exit).
> §15 references workflow YAMLs by path + expected `conclusion`, not
> inline cargo. Phase 1 (workspace check) runs on
> `cargo-validate-workspace.yml` against each `junior/*` worker branch;
> Phase 2 (e2e on `phase-v1-SL-d`) per advisor's local-vs-dispatch user
> gate (PR #105). See `.claude/PRPs/templates/plan.template.md` §15.6 +
> `.claude/PRPs/plans/v1-validate-agent.plan.md`.

## Table of contents

| § | Heading |
|---|---|
| 1 | Summary |
| 2 | Source |
| 3 | Problem statement |
| 4 | Solution statement |
| 5 | Metadata + complexity score |
| 6 | Relationship to other v1-SL sub-phases |
| 7 | Preflight guardrails inherited from prior phases |
| 8 | Flow design |
| 9 | Mandatory reading |
| 10 | Patterns to mirror |
| 11 | Files to change |
| 12 | NOT building in v1-SL-d |
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

v1-SL-d is the **producer-side rewrite** that closes the v1
sponsor-liability lane. Two halves shipped in one sub-phase:

1. **`apply_sponsor_liability` split** at
   `crates/api/api/src/governance/sponsor_liability.rs:142` into
   `compute_sponsor_liability(...) -> LemmyResult<Vec<SponsorDelta>>`
   (pure read; no DB writes; idempotent) +
   `fire_sponsor_liability(deltas, ...) -> LemmyResult<usize>` (DB
   writes only — `reputation_event` rows + `sponsor_liability_applied`
   / `sponsor_liability_clamped` log entries) + a thin wrapper
   `apply_sponsor_liability(...)` retaining the v0 signature `=
   compute + fire`. SL-c's scheduler call site at
   `sponsor_liability_grace.rs:510` continues to work without
   modification.
2. **`submit_jury_vote` mutation** at
   `crates/api/api/src/governance/submit_jury_vote.rs:458-493`
   replacing the v0 immediate-fire path with the
   `Decided → SponsorLiabilityPending` transition, grace-window
   computation, and deferred-write set per PRD §9.3 +
   jury-mechanics-v1 PRD §9.1 step 7. Cases with active sureties on
   the target person transition to `SponsorLiabilityPending` (with
   `grace_expires_at` snapshotted from
   `grace_window_for_severity(case.severity)`); cases without active
   sureties preserve v0 immediate-`Decided` semantics.

Plus a new helper `grace_window_for_severity(severity, cache, conn) ->
LemmyResult<chrono::Duration>` (pure read; instance-scoped per DQ
#178) sibling of `severity_for_action` at
`sponsor_liability.rs:112`, four e2e tests in a new
`mod v1_sl_d_fixtures` block at end of `crates/server/tests/e2e.rs`
(currently 12,819 lines), and one unit-test task covering compute
idempotency + grace-window severity mapping (per DQ #180 — unit tests
live in `sponsor_liability.rs::tests`, NOT `e2e.rs`).

**Headline acceptance condition.** Stories 1 + 2 + 3 are `[done]`
with their Phase 1 + Phase 2 workflows green: (Story 1) the
compute/fire split lands with the wrapper preserving v0 byte-identical
outputs and SL-c's call site continuing to work; (Story 2)
`submit_jury_vote` transitions liability-bearing+sponsored cases to
`SponsorLiabilityPending` with correct `grace_expires_at` snapshot and
deferred writes; (Story 3) `grace_window_for_severity` reads the
correct config key per severity tier.

**SL-d touches NO migration, NO new ENTRY_KIND_* const, NO
CaseStatus variant, NO `governance_config` seed key, NO HTTP endpoint,
NO DTO, NO route.** It is two file-level rewrites in `crates/api/api/`
+ four e2e tests + a unit-test task + retro. SL-d **closes the SL
producer side**; SL-e ships the lane-wide e2e suite end-to-end after.

---

## 2. Source

- `.claude/PRPs/briefs/sl-d-planning-1.md` @ `governance-v0`
  `f71840603` — the advisor brief (rewritten 2026-05-07; clarified
  2026-05-10 via DQ #176-#180; Hypothesis A/B retired). The brief is
  this plan's primary contract.
- `.claude/PRPs/prds/v1-sponsor-liability.prd.md` @ `governance-v0` —
  parent PRD. Specifically:
  - §1, §2 (vision/goals; SL-d is producer-side mutation only).
  - §3.1 + §3.4 (CaseStatus extensions — SL-d writes
    `SponsorLiabilityPending`; SL-c writes Fired/Escaped).
  - §4.1 (severity-proportional grace windows — load-bearing for
    `grace_window_for_severity`).
  - §4.3 (severity is snapshotted at Decided-transition time per
    ADR-010 won't-disadvantage rule — load-bearing).
  - §6 (scheduler — SL-c's; SL-d produces the cases SL-c consumes).
  - §8.1 (`liability_escape_reason` JSONB — SL-d does NOT write this
    column).
  - **§9.1 (`apply_sponsor_liability` split — SL-d's primary
    deliverable).**
  - **§9.3 (`submit_jury_vote` mutation — SL-d's other deliverable;
    pointer to `v1-jury-mechanics.prd.md` §9.1 step 7).**
  - §11.3 (v0 endpoint contracts preserved — DTO + response unchanged
    on `submit_jury_vote`; only internal lifecycle differs).
  - §11.4 (deferred-write semantics — `public_case_log` + juror
    `reputation_event` rows fire from SL-c's scheduler at fire/escape
    time, NOT at vote-tally time).
  - §15 row 4 (depends on Phases 1, 3 = SL-a + SL-c).
  - §17 (cross-cutting — ADR-013 enum-exhaustiveness on
    `submit_jury_vote.rs` new match sites).
  - §18 B6 (§9.3 reduced to pointer to jury-mechanics-v1 PRD §9.1).
- `.claude/PRPs/prds/v1-jury-mechanics.prd.md` §9.1 — the integrated
  9-step `submit_jury_vote` handler shape SL-d slots into. SL-d
  contributes the sponsor-liability branch (step 7).
- `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md` @
  `governance-v0` — most recent shipped sibling; canonical mirror for
  §6 relationship table shape, §13 anchor-Edit pattern,
  `mod v1_sl_d_fixtures` placement, §15 Shape G shape, §16a Stories
  grain.
- `.claude/PRPs/plans/v1-sponsor-liability-c.plan.md` (trunk c plan)
  — §6 + §10 inheritance shape; SL-c's call-site posture
  (apply_sponsor_liability unsplit at SL-c HEAD; SL-d's wrapper
  preserves the signature so SL-c continues to work).
- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — schema
  foundation SL-d reads.
- `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` — most recent
  shipped SL plan with full handler authorship; §13 anchor-Edit
  pattern + fixture-mod shape (`mod v1_sl_b_fixtures` at
  e2e.rs:10980) is the immediate predecessor pattern for
  `mod v1_sl_d_fixtures`.
- `.claude/PRPs/plans/v1-jury-mechanics-c.plan.md` — JM-c shipped the
  TODO marker at `submit_jury_vote.rs:458` as a hand-off pointer
  pointing directly at SL-d.
- `.claude/PRPs/templates/plan.template.md` — canonical 20-section
  schema; §15.6 Shape G DoD; §16a Stories mandatory; §13 FILES YAML
  per task; §5.1 complexity breakdown table.
- `.claude/agents/planning.md` — subagent contract; §13 per-task
  FILES YAML block discipline; §5 complexity score rule;
  canonical-schema-first gate.
- `.claude/rules/decision-queue.md` — DQ schema-v2 attribution
  (planner → `from: "planner"`, never `"advisor"`); Recipe 2
  planner-resolved pre-seed; `kind: "validate-pending"` Shape G
  routing.
- `.claude/rules/advisor-orchestrator.md` — Stage shape under Shape
  G; Phase 2 e2e local-vs-dispatch user gate (PR #105); §G4
  classifier case-shape rows 4a/4b/4c (per
  `feedback_lemmy_error_no_std_error.md` amendment 2026-05-09).
- `.claude/rules/branch-manager.md` — file-ownership; BM cuts
  `phase-v1-SL-d` after planner ships.
- `.claude/rules/phase-branch.md` — phase-branch +
  PR-into-`governance-v0` flow.
- `.claude/rules/governance-log-entry-kind-registry.md` — SL-a
  pre-landed `_SPONSOR_LIABILITY_PENDING` const at
  `governance_log.rs:199`; SL-d activates the emit at the
  `Decided → SponsorLiabilityPending` transition (per DQ #179
  resolution: emit-at-transition LOCKED). Registry rule's
  pre-landed-const exemption confirms this is the SL-d binding row.
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  — **ADR-005** (multi-dimensional reputation; preserved),
  **ADR-008** (append-only log; preserved), **ADR-010
  (won't-disadvantage rule — load-bearing for severity snapshot
  semantics)**, **ADR-013 (CaseStatus enum-exhaustiveness —
  load-bearing for SL-d's new `match case.status` sites in
  `submit_jury_vote.rs`)**, **ADR-014** (federation deferral —
  preserved), **ADR-015 (pseudonymisation — load-bearing for
  `_SPONSOR_LIABILITY_PENDING` payload's `target_pseudonym`)**,
  OQ-025 (sponsor-liability v1 resolution into this PRD).

### Lessons that bind §13 decisions (SL-d)

- **`feedback_lemmy_error_no_std_error.md`** — load-bearing. The 4
  e2e tests + 1 unit-test task author code in
  `crates/server/tests/e2e.rs` and
  `crates/api/api/src/governance/sponsor_liability.rs::tests`. The
  v1-SL-* fixture-mod canonical sibling at
  `crates/server/tests/e2e.rs:11001-11924` (`mod v1_sl_b_fixtures`)
  uses uniform `LemmyResult<T>` throughout (Case A per the lesson's
  amended case enumeration). **§13 Tasks 3-6 helpers + test fns ALL
  use `LemmyResult<()>` outer.** No `Box<dyn Error>` outer; no
  `.map_err` bridges. The 2026-05-09 v1-SL-c-2 3-cycle catch-fire
  (workflow runs `25582548670` / `25595869651` / `25603848858`)
  proved Case C (mixed shapes) is a hard refusal; SL-d follows
  Case A uniformly to avoid the same impedance mismatch. The §13
  stub for Tasks 3-6 prescribes `LemmyResult<()>` literally;
  per-task GOTCHA blocks cite the canonical sibling.
- **`feedback_junior_worker_e2e_edit_hang.md`** — load-bearing.
  e2e.rs is now 12,819 lines (DQ #180 confirmation; supersedes the
  brief's 10,500-10,700 estimate which predated SL-c-2 merge). 4
  e2e tests = 4 separate §13 tasks (Tasks 3, 4, 5, 6), each one
  anchor-Edit at file end. Bundle Edits hang Junior workers.
- **`feedback_complexity_score_pre_split.md`** — SL-d score
  computed mechanically in §5.1 below; tripped threshold; planner
  files DQ #186 self-resolved per Recipe 2 with proceed rationale
  (e2e factor dominates; SL-b/SL-c-2/JM-e proceed-as-one
  precedents).
- **`feedback_principles_not_rules.md`** — score is a signal, not
  a hard rule. SL-d's e2e tests are mechanically anchor-Edit-friendly
  per the trunk pattern; further splitting yields no meaningful
  reduction.
- **`feedback_advisor_watchpoint_specificity.md`** — every §4
  watchpoint cites a concrete file/handler/`schema.rs` line. Binds
  §4 entries (12 watchpoints below).
- **`feedback_explicit_file_arrays_on_tasks.md`** — every §13 task
  carries a FILES YAML block; cohort dispatch reads
  `union(creates, modifies)`.
- **`feedback_parallel_cohort_dispatch.md`** — Tasks 3-6 all
  `modifies: crates/server/tests/e2e.rs`; YAML overlap rule refuses
  cohort dispatch — they ship serially. Task 1 (sponsor_liability.rs
  split) and Task 2 (submit_jury_vote.rs mutation) are also serial
  because Task 2 depends on Task 1's `compute_sponsor_liability`
  symbol existing.
- **`feedback_pre_phase_dod_smoke_test.md`** +
  **`feedback_plan_dod_dry_run_at_write.md`** — advisor-side DoD
  smoke-test runs every §15 command literally before plan approval.
  Under Shape G the §15 entries name workflow YAML paths +
  `gh run list` queries — both verifiable mid-plan-approval.
- **`feedback_features_full_p_crate_incompatible.md`** — never
  `-p <crate>` + `--features full`. Workflow YAMLs comply
  (`cargo-validate-workspace.yml:88-95` uses `--workspace --features
  full`); §15.7 advisor-side smoke snippets respect the rule.
- **`feedback_features_full_workspace_only.md`** — `--features full`
  required to activate `DbEnum` + `ts-rs` derives. Encoded in
  workflow YAML.
- **`feedback_multi_write_handlers_need_transactions.md`** — SL-d's
  mutation runs **inside the existing**
  `submit_jury_vote::process_vote` outer `run_transaction` at
  `submit_jury_vote.rs:140`. SL-d does NOT add a new transaction.
  The threaded `&mut cache` (single-handler pattern) is preserved.
- **`feedback_insertform_default_propagation.md`** — citation-only:
  SL-d does not add new InsertForm fields. The mutation UPDATEs the
  existing `moderation_case` row's `status` + `grace_expires_at`
  columns via direct `update().set(...)` per the existing pattern at
  `submit_jury_vote.rs:491-497`.
- **`feedback_clippy_test_style.md`** — R1 every `i32 ↔ i64`
  comparison uses `i64::from(...)`. Bound where surfaces in §13
  Tasks 1-6.
- **`feedback_brehon_verify_pre_merge.md`** — §16a Stories grain
  enables `/brehon-verify` phantom check before `bm-merge`. SL-d's
  three stories (1, 2, 3) are checkpointed by Phase-1 workspace
  check + Phase-2 e2e on phase-branch tip.
- **`feedback_read_canonical_before_writing_spec.md`** — SL-d cites
  trunk SL-c plan + SL-c-2 sibling shipped plan (Tasks 1-5 and §13
  pattern) + SL-b's `mod v1_sl_b_fixtures` (canonical Case A error
  shape) + JM-e §13 fixture-mod conventions; canonical-schema gate
  satisfied.
- **`feedback_handover_trailer_cohort_propagation.md`** —
  non-binding under serial dispatch (no `[P]` cohorts in §13);
  per-task `HANDOVER:` commit trailer recommended where the next
  task benefits.
- **`feedback_retro_not_report.md`** + `feedback_four_role_retro_signals.md`
  + `feedback_retro_task_complexity_score.md` — retro shape (final
  task; per-task complexity table mandatory).
- **`feedback_schema_changing_spec_retrofit_question.md`** —
  citation-only (SL-d changes no spec/template shape).
- **`feedback_plan_stub_uniformity_with_canonical_sibling.md`** —
  binds §13 Tasks 3-6: stubs MUST use `LemmyResult<()>` outer
  uniformly with the canonical sibling at `e2e.rs:11001-11924`. The
  3-cycle SL-c-2 catch-fire (cycles same `(E0277, e2e.rs)`) was
  caused by stub-shape non-uniformity; SL-d's §13 stubs avoid that
  family.

### Related prior plans (canonical-shape mirror)

- `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md` (most recent
  shipped sibling under Shape G + spec-kit adoption).
- `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` (full handler
  authorship + canonical Case A error-shape fixture mod).
- `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` (Shape G +
  per-test anchor-Edit + §16a Stories pattern).
- `.claude/PRPs/plans/v1-jury-mechanics-c.plan.md` (the JM-c plan
  that shipped the TODO at `submit_jury_vote.rs:458`).

---

## 3. Problem statement

Post-SL-a-merge + post-SL-b-merge + post-SL-c-merge on
`governance-v0`:

- **The producer side of the SL lane is incomplete.** SL-c shipped
  the consumer (scheduler module + per-case fire-or-escape branches)
  but **no producer writes `SponsorLiabilityPending` cases yet**.
  SL-c's batch query (`grace_expires_at.le(Some(now))` filter) finds
  zero rows on a fresh `governance-v0` deployment because no handler
  ever set `case.status = SponsorLiabilityPending`. The lane's main
  loop is open.
- **The TODO marker at `submit_jury_vote.rs:458` is the
  advance-handoff.** JM-c shipped the marker explicitly so SL-d
  would know the exact mutation site. Quoting the in-source comment:
  > `TODO(v1-sponsor-liability-d): replace this v0 apply_sponsor_liability call with the [...]`
  This is the literal SL-d work site.
- **`apply_sponsor_liability` does both compute AND fire in one
  call.** Per PRD §9.1, the v1 architecture splits read-only
  computation from DB writes so the scheduler can fire deltas at
  grace-window expiry instead of immediately at vote-tally time.
  The split itself is a refactor (no behaviour change to the
  wrapper); the new signatures unblock SL-d's
  `compute_sponsor_liability`-only call from inside `process_vote`.
- **`grace_window_for_severity` does not yet exist.** SL-d's
  mutation needs a pure-read helper that maps `CaseSeverity →
  chrono::Duration` by reading
  `liability.grace_window_<minor|moderate|severe>_hours` from
  `governance_config` (instance scope, per DQ #178 LOCKED). The
  helper's signature is specified in PRD §9.4 + brief §2.1.c.
- **Watch 10 ADR-013 enum-exhaustiveness on the
  `submit_jury_vote.rs` mutation site.** SL-a shipped 3 new
  `CaseStatus` variants; SL-a Tasks 5+ updated existing match sites
  outside `submit_jury_vote.rs` to enumerate all 12 variants.
  SL-d's new branch logic in `submit_jury_vote.rs` introduces NEW
  match-site candidates — each must be exhaustive (no `_ =>` arms).
- **The existing `case_decided` log entry at
  `submit_jury_vote.rs:649-659` fires unconditionally on the v0
  path.** Per PRD §11.4 deferred-write semantics + brief §2.2 step
  8, SL-d preserves this entry (auditors see the decision event
  immediately) AND adds a `sponsor_liability_pending` log entry on
  the `Decided → SponsorLiabilityPending` transition (DQ #179
  LOCKED: emit-at-transition).
- **The deferred-write set is the most subtle invariant.** Per PRD
  §11.4 + brief §2.1.b step 3, on the Pending branch
  `public_case_log` entries and juror `reputation_event` rows are
  NOT written at vote-tally time — they fire from SL-c's scheduler
  when the case transitions to Fired/Escaped. The no-sponsor path
  (zero active sureties) preserves v0 immediate-`Decided` semantics
  + immediate juror reputation writes per PRD §11.3. Three e2e
  tests cover the three paths (Pending vs no-sponsor Decided vs
  NoAction).

The substrate is in place; SL-d supplies the producer-side rewrite.

---

## 4. Solution statement

Eight surgical changes for SL-d, organised as 7 impl tasks +
Task 0 pre-flight + retro.

### 4.1 Architecturally load-bearing decisions

- **Wrapper-preserves-signature is the SL-c-non-disruption invariant.**
  After Task 1 ships, `apply_sponsor_liability(conn, target_person_id,
  case_id, community_id, action: SanctionAction, cache: &mut
  ConfigCache) -> LemmyResult<usize>` retains the v0 signature
  byte-for-byte (verified by `git diff` against
  `governance-v0:crates/api/api/src/governance/sponsor_liability.rs:142`).
  SL-c's call site at `sponsor_liability_grace.rs:510` continues to
  work without modification. **Plan §13 does NOT update SL-c's call
  site** (per brief §2.3 — optional task; default lean is
  don't-update).
- **Compute is pure; fire is the only writer.** Per PRD §9.1:
  - `compute_sponsor_liability(conn, target_person_id, case_id,
    community_id, action, cache) -> LemmyResult<Vec<SponsorDelta>>`
    — pure read; queries `surety` for active sponsors, reads
    `governance_config` keys (severity bucket delta + floor +
    founder multiplier + regular multiplier), looks up
    `reputation_snapshot.endorsement_strength` per sponsor for clamp
    math, computes per-sponsor base + remainder + multiplier + clamp
    arithmetic, returns `Vec<SponsorDelta>`. **NO `reputation_event`
    INSERT, NO `governance_log::append`, NO UPDATE.**
  - `fire_sponsor_liability(conn, deltas: Vec<SponsorDelta>,
    case_id, target_person_id, action, cache) -> LemmyResult<usize>`
    — DB writes only. Iterates `deltas`; for each: INSERT
    `reputation_event` row, append `sponsor_liability_applied` log
    entry, append `sponsor_liability_clamped` if `clamp_engaged`.
    Returns `count == sponsor_count` (the v0 wrapper's return shape
    preserved).
  - `apply_sponsor_liability(conn, ..., cache) ->
    LemmyResult<usize>` — thin wrapper: `let deltas =
    compute_sponsor_liability(conn, ..., cache).await?;
    fire_sponsor_liability(conn, deltas, ..., cache).await`. Body is
    ~3 lines; signature byte-identical to v0.
- **`SponsorDelta` struct shape.** Defined `pub(crate)` in
  `sponsor_liability.rs` body alongside `LiabilitySeverity`:
  ```rust
  #[derive(Debug, Clone, PartialEq)]
  pub(crate) struct SponsorDelta {
    pub(crate) sponsor_id: PersonId,
    pub(crate) pre_multiplier_delta: i64,
    pub(crate) multiplier: f64,
    pub(crate) post_multiplier_delta: i64,
    pub(crate) final_delta: i64,
    pub(crate) clamped_from: Option<i64>,
    pub(crate) is_founder: bool,
    pub(crate) current_endorsement_strength: i64,
  }
  ```
  Field names mirror the v0 closure-local bindings at
  `sponsor_liability.rs:212-288` so `fire_sponsor_liability` can
  re-emit byte-identical `governance_log` payloads. The struct is
  internal to the crate; no `pub` visibility; not exposed via shim.
  PartialEq derive (not Eq — f64 doesn't implement Eq) supports
  Task 7 unit-test `assert_eq!`.
- **`grace_window_for_severity(severity, cache, conn) ->
  LemmyResult<Duration>`** lives in `sponsor_liability.rs` as a
  sibling of `severity_for_action` (per brief §2.1.c locking — NOT
  in `sponsor_liability_grace.rs`). Maps `CaseSeverity` →
  `liability.grace_window_<bucket>_hours` config key (Instance scope
  per DQ #178 LOCKED) → `chrono::Duration::hours(value)`. Pure
  read; idempotent. The mapping bucket follows the same
  severity-bucket table as `severity_for_action` (Minor / Moderate
  / Severe). v1 config namespace uses
  `liability.grace_window_minor_hours` / `..._moderate_hours` /
  `..._severe_hours`, all three SL-a-shipped at default 24/72/168
  per PRD §10.
- **`CaseSeverity` to `LiabilitySeverity` mapping.** The v0
  `severity_for_action(SanctionAction) -> LiabilitySeverity` at
  `sponsor_liability.rs:112` maps `SanctionAction →
  LiabilitySeverity`. SL-d's helper takes `CaseSeverity` (the column
  type at `moderation_case.severity`, defined at
  `crates/db_schema_file/src/enums.rs` — verify variants at task
  start). The helper defines a new internal mapping function
  `liability_severity_from_case_severity(CaseSeverity) ->
  LiabilitySeverity` (or folds into `grace_window_for_severity`'s
  body) exhaustively per ADR-013. **DQ #178 LOCKED** specifies
  severity comes from the case row, not the config — i.e.
  `case.severity` (a `CaseSeverity`) is the input. **Task 1
  GOTCHA** documents this and cites
  `crates/db_schema_file/src/enums.rs` `CaseSeverity` variants
  (read at task-start).
- **Inside-handler step ordering preserved.** SL-d's mutation
  modifies the existing `submit_jury_vote::process_vote`
  `run_transaction` body at `submit_jury_vote.rs:140`. Steps 1-6
  (load/validate/insert vote → tally → threshold check →
  winning-decision sanction insert) unchanged from JM-c shipped
  state. Steps 8.5 → 8.9 → 9-12 mutate per brief §2.2:
  - Step 8.5 (current call to `apply_sponsor_liability` at line
    471): replace with `compute_sponsor_liability(...)` returning
    `Vec<SponsorDelta>`. Then branch:
    - **`Vec<SponsorDelta>` non-empty (active sureties exist):**
      set transition flag → step 8.9 will write
      `case.status = SponsorLiabilityPending`, `grace_expires_at =
      now + grace_window_for_severity(case_row.severity)`. Defer
      juror reputation events + public_case_log to scheduler
      fire/escape time. Append `sponsor_liability_pending`
      governance_log entry with payload `{case_id, target_pseudonym,
      severity, grace_expires_at, sponsors_pseudonyms: [...]}` per
      registry assignment.
    - **`Vec<SponsorDelta>` empty (no active sureties):** preserve
      v0 immediate-`Decided` path. Step 8.9 writes
      `case.status = Decided`. Steps 9-12 fire normally
      (`case_decided` log, `public_case_log` insert, juror
      reputation events, reporter reputation event).
  - Step 8.9: existing UPDATE at line 491-498 already conditionally
    sets `status` — SL-d branches on the path determined at step
    8.5.
  - Step 9 (`case_decided` log at `submit_jury_vote.rs:649-659`):
    fires on BOTH paths (Pending and no-sponsor Decided). PRD §11.4
    auditor visibility — auditors see "case decided X" immediately,
    even when liability resolution defers to scheduler.
    `appeal_window_expires_at` UPDATE at lines 638-641 also fires
    on BOTH paths (per JM-c invariant; SL-d preserves).
  - Steps 10-12 (`public_case_log` insert + juror/reporter
    `reputation_event` rows): **only on the no-sponsor / NoAction /
    no-sanction paths (v0 path)**. On the Pending branch, these are
    the deferred writes that SL-c's scheduler emits at fire/escape
    time.
- **NoAction path skips liability machinery entirely.** Per brief
  §2.1.e Test #3: when `winning_decision == JuryDecision::NoAction`,
  `map_decision_to_sanction(...)` returns `None` at
  `submit_jury_vote.rs:423`, so the entire `if let Some((scope,
  action)) = ...` block (lines 423-481) is skipped.
  `compute_sponsor_liability` is NOT called. Case transitions to
  `Decided` (v0 semantics preserved). Test #3 asserts.
- **Deferred-write set boundary.** On the Pending branch:
  - **NOT written at vote-tally time:** `public_case_log` row,
    juror `reputation_event` rows, reporter `reputation_event` row,
    federation outbox, `case_closed` log.
  - **WRITTEN at vote-tally time:** vote insert, sanction insert,
    `case.status = SponsorLiabilityPending`, `case.grace_expires_at`,
    `case.appeal_window_expires_at` (per JM-c invariant — fires on
    BOTH paths), `case.winning_decision` (already set in v0 step
    8.9), `sanction_created` log, `case_decided` log,
    `sponsor_liability_pending` log.
  - **DEFERRED to scheduler fire/escape time** (SL-c's
    `sponsor_liability_grace.rs::fire_or_escape_case_inner`): on
    fire — `sponsor_liability_applied` log per sponsor +
    `sponsor_liability_fired` summary log + per-sponsor
    `reputation_event` rows + per-juror `reputation_event` rows +
    per-reporter `reputation_event` row + `public_case_log` row +
    federation outbox. On escape — `sponsor_liability_escaped` log
    + `liability_escape_reason` JSONB UPDATE on `moderation_case`.
  - **Per PRD §11.4 + DQ #179 LOCKED**: governance_log is NOT in
    the deferred set — `sponsor_liability_pending` fires immediately
    on transition.
- **Shape G — DoD references workflow YAMLs by path + expected
  `conclusion`, not inline cargo.** Per
  `.claude/PRPs/templates/plan.template.md` §15.6 + DQ #67. Every
  §13 task's DoD references `cargo-validate-workspace.yml` on the
  worker branch + workflow_run_id captured by impl-task subagent
  post-push. Phase 2 e2e fires on `phase-v1-SL-d` finalize-merge
  per advisor's local-vs-dispatch user gate (PR #105).
- **No migration round-trip workflow fires.** SL-d touches no
  `migrations/**` paths.

### 4.2 Watchpoints (specific files / handlers / `enums.rs` lines)

Per `feedback_advisor_watchpoint_specificity.md`. The 12
watchpoints below are the seed list per brief §4.1.

1. **TOCTOU on case-status mutation. (Task 2 binding —
   `submit_jury_vote.rs:140`.)** SL-d's mutation runs inside the
   existing `submit_jury_vote::process_vote` `run_transaction`. The
   case-row UPDATE happens inside the same tx as the
   sanction-insert + jury-tally — atomic. Verification step in
   Task 2: no separate-load-then-update; status comparison against
   the new branch condition (active sureties exist) happens inside
   the tx, on the `case_row` variable already loaded at line 198
   with FOR UPDATE per PR #98 cr-2.
2. **Severity snapshot semantics. (Task 1 + Task 2 binding —
   ADR-010 won't-disadvantage rule + PRD §4.3.)** The severity used
   to compute `grace_window_for_severity` is the severity recorded
   ON THE CASE ROW at decision-transition time
   (`case_row.severity`, read from `moderation_case.severity` per
   `crates/db_schema_file/src/schema.rs:783`) — NOT re-evaluated
   from config at fire-time. Task 2 specifies: read
   `case_row.severity` from the `case_row: ModerationCase` variable
   already loaded at `submit_jury_vote.rs:198`; pass that exact
   value to `grace_window_for_severity(case_row.severity, cache,
   conn)`. Re-deriving severity from config or from sanction action
   at fire-time is a regression. Test #1 asserts the exact
   `grace_expires_at` value matches the snapshot severity's
   config-key default (24/72/168 hours per PRD §10).
3. **No-sponsor path preserves v0 immediate-`Decided`. (Task 2 +
   Test #2 binding.)** The new branch logic checks
   `Vec<SponsorDelta>::is_empty()` from
   `compute_sponsor_liability`'s return; if empty, the v0 path runs
   (status = Decided, juror reputation events fire immediately, no
   `sponsor_liability_pending` log entry). Equivalent semantically
   to the v0 `apply_sponsor_liability` returning 0. Test #2 asserts.
4. **Wrapper signature preserved. (Task 1 binding.)** `pub(crate)
   async fn apply_sponsor_liability(conn, target_person_id, case_id,
   community_id, action, cache) -> LemmyResult<usize>` — same
   signature as v0 `sponsor_liability.rs:142`. SL-c's call site at
   `sponsor_liability_grace.rs:510` continues to work without
   modification. Task 1 includes a `git diff
   governance-v0..HEAD -- crates/api/api/src/governance/sponsor_liability.rs`
   spot check at task-end asserting the wrapper's signature line is
   byte-equal to v0; if it differs (e.g. argument reorder), STOP.
5. **Compute is pure. (Task 1 binding.)**
   `compute_sponsor_liability(...)` performs ZERO DB writes. Task 1
   includes a grep step:
   `rg -nE 'insert_into|update\(|append\(|persist'
   crates/api/api/src/governance/sponsor_liability.rs` — the only
   matches must be inside `fire_sponsor_liability`'s body OR inside
   the wrapper's call to `fire_sponsor_liability` — never inside
   `compute_sponsor_liability`'s body. The grep is mechanical; an
   accidental write inside `compute` is a ship-blocker.
6. **Fire is the only writer. (Task 1 binding.)** All
   `reputation_event` INSERT + `sponsor_liability_applied` /
   `sponsor_liability_clamped` log appends move to
   `fire_sponsor_liability`. Same grep step as #5. Verify by
   reading `fire_sponsor_liability`'s body at task-end: contains
   exactly `insert_into(reputation_event::table)` (1 site) +
   `governance_log::append(..., ENTRY_KIND_SPONSOR_LIABILITY_APPLIED, ...)`
   (1 site, in the per-sponsor for-loop) +
   `governance_log::append(..., ENTRY_KIND_SPONSOR_LIABILITY_CLAMPED, ...)`
   (1 site, conditional inside for-loop).
7. **Idempotent compute. (Task 7 binding — unit test.)** Calling
   `compute_sponsor_liability(...)` twice with identical inputs
   returns identical `Vec<SponsorDelta>`. Test #5 asserts. Property
   follows trivially from "no side effects in compute" but the test
   makes it observable. **Test lives in
   `sponsor_liability.rs::tests`** per DQ #180.
8. **`submit_jury_vote.rs` matches are exhaustive. (Task 2 binding
   — ADR-013 + R1 zero `_ =>` arms.)** Any new `match case.status`
   site SL-d adds must enumerate all 12 variants. Task 2 includes a
   grep step at task-end: `rg -nE 'match.*case\.status\b|match.*CaseStatus'
   crates/api/api/src/governance/submit_jury_vote.rs` — every match
   line's nearby block (next 14-20 lines) must NOT contain `_ =>`.
   The 12 `CaseStatus` variants are at
   `crates/db_schema_file/src/enums.rs:393-432` (verify at
   task-start).
9. **`sponsor_liability_pending` log entry on transition. (Task 2
   binding — DQ #179 LOCKED emit-at-transition.)** Use existing
   const `governance_log::ENTRY_KIND_SPONSOR_LIABILITY_PENDING`
   (re-exported via api shim at
   `crates/api/api/src/governance/governance_log.rs:76`; canonical
   declaration at
   `crates/db_schema/src/source/governance/governance_log.rs:199`).
   Payload shape per registry assignment + ADR-015:
   ```json
   {
     "case_id": <i32>,
     "target_pseudonym": "<UUID-string>",
     "severity": "<minor|moderate|severe>",
     "grace_expires_at": "<DateTime<Utc> ISO 8601>",
     "sponsors_pseudonyms": ["<UUID-string>", ...]
   }
   ```
   `target_pseudonym` resolved via
   `actor_pseudonym_helper::get_or_create(&mut conn.into(),
   target_id).await?`; `sponsors_pseudonyms` resolved per-sponsor
   via the same helper, looped. Pseudonyms are UUID strings; never
   raw `*_id` values.
10. **Pseudonym discipline. (Task 2 binding — ADR-015 + Watch 10.)**
    Every governance_log payload field naming a person uses
    `*_pseudonym`, never raw `*_id`. The `sponsor_liability_pending`
    payload contains `target_pseudonym` (string) +
    `sponsors_pseudonyms` (array of strings) + `severity` (string)
    + `grace_expires_at` (timestamp) + `case_id` (integer; case_id
    is OK to expose raw — not a person identifier). Task 2 +
    Test #1 defensive `assert_ne!(json["target_pseudonym"].as_str().unwrap(),
    &format!("{}", target_id.0))`.
11. **No new ENTRY_KIND_*** — all consts shipped in SL-a. (Task 2
    binding.)** Plan §13 must NOT add to the registry. Task 2 uses
    existing `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` const. No new
    const declaration. Verify at task-end:
    `git diff governance-v0..HEAD --
    crates/db_schema/src/source/governance/governance_log.rs`
    returns empty; same for
    `crates/api/api/src/governance/governance_log.rs`.
12. **e2e Edit-per-task discipline. (Tasks 3-6 binding —
    load-bearing for SL-d's task-list shape.)** Per
    `feedback_junior_worker_e2e_edit_hang.md`: e2e.rs is now
    **12,819 lines** (DQ #180 LOCKED; supersedes brief's
    10,500-10,700 estimate). 4 e2e tests = 4 individual §13 tasks
    (Tasks 3, 4, 5, 6), each one anchor-pattern Edit at file end
    inside a NEW `mod v1_sl_d_fixtures` block. Do NOT bundle. Unit
    tests 5+6 (Task 7) live in `sponsor_liability.rs::tests`, not
    e2e.rs.

### 4.3 Rejected alternatives

- **Bundle the 4 e2e tests into 1-2 §13 tasks.** Rejected per
  `feedback_junior_worker_e2e_edit_hang.md` — e2e.rs is 12,819
  lines; multi-test bulk Edits hang Junior workers. SL-c-2 shipped
  5 tests as 5 tasks for the same reason.
- **Split the SL-d sub-phase further (e.g. SL-d-1 split +
  grace-helper, SL-d-2 mutation + tests).** Per
  `feedback_complexity_score_pre_split.md` §5.2 below: planner
  files DQ #186 self-resolved with proceed rationale (e2e factor
  dominates +12 of total; further splitting yields no meaningful
  score reduction; SL-b/SL-c-2/JM-e proceed-as-one precedents).
  Advisor may overturn at plan-approval time.
- **Mark Tasks 1+2 as `[P]`.** Rejected: Task 2 imports
  `compute_sponsor_liability` from Task 1's edits; cohort dispatch
  would race the import not-yet-existing. They ship serially.
- **Mark Tasks 3-6 as `[P]`.** Rejected: YAML overlap rule refuses
  cohort because all 4 tasks `modifies: crates/server/tests/e2e.rs`.
  Each task anchor-Edits at the prior task's commit tip; cohort
  dispatch would race.
- **Author the SL-c call-site update from `apply_sponsor_liability`
  to `fire_sponsor_liability` directly.** Rejected per brief §2.3
  — optional task; default lean is don't-update
  (wrapper-preserving path is canonically correct; SL-c's code
  stays untouched). Plan §13 does NOT include this task.
- **Add a `sponsor_notifications` step
  (`notify_sponsor_of_pending_liability`).** Rejected per DQ #176
  LOCKED 2026-05-10 — sponsor notifications are OUT OF SCOPE for
  SL-d. PRD §9.3 lists exactly 3 SL-d contributions (transition,
  grace-window, deferred write set); notifications are not among
  them. OQ-V1-SL-03 unresolved (v3 polish).
- **Add a `computed_deltas` snapshot column on `moderation_case`.**
  Rejected per DQ #177 LOCKED 2026-05-10 —
  `fire_sponsor_liability` calls `compute_sponsor_liability`
  internally at SL-c scheduler fire-time; the wrapper = `compute +
  fire` inline. No persistence of deltas; recompute-at-fire is the
  canonical model. No new migration.
- **Re-derive severity from config or sanction at fire-time.**
  Rejected per ADR-010 won't-disadvantage rule + Watchpoint #2 —
  severity is snapshotted on the case row; SL-c scheduler reads
  `case.severity` for any future severity-dependent computation.
  SL-d's mutation reads `case.severity` at vote-tally time
  (already snapshotted from JM-a; SL-d does not re-write).
- **Use `_ =>` wildcard arms in the new `submit_jury_vote.rs`
  branch logic.** Rejected per ADR-013 + R1 + Watchpoint #8.
- **Test the restoration-completed branch.** Rejected per
  `feedback_build_what_tests_exercise.md` — never test a producer
  that doesn't yet emit. Restoration is restorative-mechanics-v1
  PRD's deliverable.
- **Use `Result<(), Box<dyn Error>>` outer in any e2e test fn.**
  Rejected per `feedback_lemmy_error_no_std_error.md` Case A
  canonical sibling at `crates/server/tests/e2e.rs:11001-11924`
  (`mod v1_sl_b_fixtures`). All §13 Tasks 3-6 use uniform
  `LemmyResult<()>` outer per the lesson's Case A.

---

## 5. Metadata

- **Phase:** `v1-SL-d`
- **Branch:** `phase-v1-SL-d` (cut by BM-task before Task 1, AFTER
  this plan merges to `governance-v0` and the user approves the
  plan)
- **Target impl-task model:** `sonnet-4-6` (default, per
  `feedback_brehon_subagent_model_effort_assignments.md` baseline).
- **Estimated tasks:** 9 (Task 0 pre-flight + Tasks 1-2 producer
  code + Tasks 3-6 e2e tests + Task 7 unit tests + Task 8 retro)
- **Estimated cargo budget:** 0 GB peak local (Shape G — cargo
  runs on GH-hosted runners; Phase 2 e2e on laptop ~26 min
  single-threaded if user picks local at gate 4)
- **Forbidden-window applicability:** non-binding for impl-task
  throughput under Shape G; standard for any local diagnostic cargo
  run (rare). Phase 2 e2e on laptop respects forbidden windows per
  user gate.
- **Complexity score:** **16/10** — see breakdown below.
  Threshold-tripping; planner DQ #186 filed and self-resolved per
  Recipe 2 with proceed rationale.

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Computed mechanically
from §13 task list:

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | **2** | 7 impl tasks (Tasks 1-7; Task 0 + retro excluded). `max(0, 7-5) = 2` |
| Migrations touched | +2 each | **0** | SL-d ships zero migrations |
| Crates touched | +1 each | **2** | `crates/api/api` (sponsor_liability.rs, submit_jury_vote.rs) + `crates/server` (tests/e2e.rs). Task 7 (unit tests in `sponsor_liability.rs::tests`) shares `crates/api/api` |
| `crates/lemmy_server/tests/e2e/*.rs` edits | +3 each | **12** | Tasks 3, 4, 5, 6 each modify `crates/server/tests/e2e.rs`. 4 × +3 = 12 |
| New ADR-affecting decisions | +2 each | **0** | All ADR decisions made in PRD / DQ #176-#180; SL-d implements, does not supersede |
| Cargo budget peak above 6 GB | +1 per GB | **0** | Shape G — cargo runs off-box |
| **Total** | — | **16** | Threshold for split-DQ (Sonnet target): `>8`. **Tripped.** |

### 5.2 Split-or-proceed DQ

Per `feedback_complexity_score_pre_split.md` + `planning.md` §5
"Decision-queue — pre-seed forward-looking OQs": planner files
**DQ #186** (`from: "planner"`, `kind: "blocker"`, `answered_by:
"planner"` self-resolved with rationale per Recipe 2) BEFORE
committing the plan. Question: "Complexity score 16 exceeds 8 —
split `v1-sponsor-liability-d` further (e.g. `v1-SL-d-1` split +
grace-helper [Tasks 1-2] + `v1-SL-d-2` mutation + tests
[Tasks 3-7]), or proceed?". Options: split / proceed.

**Planner observation (binding lean — proceed):** the dominant
factor is the 4 e2e edits (+12 of the +16 total). Splitting SL-d
further yields:
- **SL-d-1** (Tasks 1-2): 2 impl tasks, 0 migrations, 1 crate, 0
  e2e edits. Score: `max(0, 2-5) + 0 + 1 + 0 + 0 + 0 = 1`. **Below
  threshold.**
- **SL-d-2** (Tasks 3-7): 5 impl tasks, 0 migrations, 2 crates, 4
  e2e edits. Score: `max(0, 5-5) + 0 + 2 + 12 + 0 + 0 = 14`.
  **Still above threshold by +6.**

Further splitting SL-d-2 only meaningfully reduces score by
splitting the 4 e2e tasks across multiple sub-phases — which
fragments the behavioural test suite and adds 2x the
cohort/finalize/retro overhead for no architectural reduction. The
4 e2e tests are mechanically anchor-Edit-friendly (single fn at
file end inside `mod v1_sl_d_fixtures`); the SL-c-2 precedent
shipped 5 e2e tests at score 17 with proceed-as-one + zero
operational regret.

Per `feedback_principles_not_rules.md`: the score is a signal, not
a hard rule. Precedent for "score above threshold but proceed":
- SL-b: shipped at score 38 with proceed-as-one (DQ #143).
- SL-c-2: shipped at score 17 with proceed-as-one (DQ #151).
- JM-e: shipped at score 15 with proceed-as-one.
- SL-a: shipped at score 13 with proceed-as-one.
- Trunk SL-c: shipped at score 21 with proceed-as-one DQ #148
  (overridden to split by user at DQ #150, but the proceed-as-one
  logic still held — the user split was for PR-scope preference,
  not complexity reduction).

SL-d at 16 sits between SL-c-2 (17) and JM-e (15) — within the
established proceed-as-one envelope. Plan ships under the
**proceed** assumption. DQ #186 filed with `answered_by: "planner"`
self-resolved per Recipe 2 citing the rationale above; the advisor
may overturn at plan-approval time if user prefers a further split
— but the e2e-factor analysis suggests further splitting yields no
meaningful reduction.

### 5.3 Per-task complexity (Sonnet ceiling: ≤4 files, ≤2 crates per task)

Walking each §13 task against the Sonnet ceiling (per
`planning.md` §5b — non-binding for Sonnet target but tracked):

| Task | files | crates | within Sonnet ceiling? |
|---|---|---|---|
| Task 0 | 0 | 0 | yes (pre-flight, no edits) |
| Task 1 (split + grace-helper) | 1 (sponsor_liability.rs) | 1 (api/api) | yes |
| Task 2 (mutation) | 1 (submit_jury_vote.rs) | 1 (api/api) | yes |
| Task 3 (e2e #1 — Pending transition) | 1 (e2e.rs) | 1 (server) | yes |
| Task 4 (e2e #2 — no-sponsor) | 1 (e2e.rs) | 1 (server) | yes |
| Task 5 (e2e #3 — NoAction) | 1 (e2e.rs) | 1 (server) | yes |
| Task 6 (e2e #4 — wrapper composition) | 1 (e2e.rs) | 1 (server) | yes |
| Task 7 (unit tests 5+6) | 1 (sponsor_liability.rs) | 1 (api/api) | yes |
| Task 8 (retro) | 1 (retro.md) + 1 (registry-flip) | 0 | yes |

All tasks within ceiling. No task split needed at the per-task
level.

---

## 6. Relationship to other v1-SL sub-phases

| Sub-phase | Status | What it ships | SL-d dependency |
|---|---|---|---|
| v1-SL-a | MERGED (PR #111, governance-v0 @ `790f6101d`) | Schema + 13 seeded keys (incl. `liability.grace_window_<minor\|moderate\|severe>_hours` defaults 24/72/168) + 5 entry-kind consts (incl. `_SPONSOR_LIABILITY_PENDING` at line 199) + 3 CaseStatus variants (incl. `SponsorLiabilityPending`) + Issue #24 partial index + backfill | SL-d reads `governance_config` keys + writes `case.status = SponsorLiabilityPending` + writes `case.grace_expires_at` (column from SL-a) + emits `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` (const from SL-a) |
| v1-SL-b | MERGED (PR #119) | `revoke_endorsement` handler + DTO + route + 9-10 e2e tests. SL-b emits `_SPONSOR_LIABILITY_ESCAPED` for the revocation branch | SL-d does NOT touch SL-b's handler. SL-d's e2e tests pre-seed cases via the new producer code-path (NOT direct DB-write — SL-d's tests EXERCISE the producer); SL-b's tests pre-seed via direct DB-write (different test scope). SL-b's `mod v1_sl_b_fixtures` at e2e.rs:10980 is the canonical Case A error-shape sibling that SL-d's `mod v1_sl_d_fixtures` mirrors |
| v1-SL-c (c-1 + c-2) | MERGED (governance-v0; recent retro `5b80186e7 docs(retro): v1-SL-c-2 retro`) | Scheduler module `sponsor_liability_grace.rs` + clokwerk wiring + atomic guard pair + 5 `grace_check_*` e2e tests | **SL-c's scheduler calls `apply_sponsor_liability` (unsplit) at `sponsor_liability_grace.rs:510`**. SL-d's split makes `apply_sponsor_liability` a thin wrapper preserving the v0 signature — SL-c's call site continues to work without modification (Watchpoint #4). Plan §13 does NOT update SL-c's call site (per brief §2.3). `mod v1_sl_c_fixtures` at e2e.rs:11927 is the immediate predecessor fixture mod |
| **v1-SL-d (THIS PLAN)** | NOT YET CUT | `apply_sponsor_liability` split (compute + fire + wrapper) + `submit_jury_vote` mutation (Decided → SponsorLiabilityPending transition + grace-window snapshot + deferred-write set) + `grace_window_for_severity` helper + 4 e2e tests + 1 unit-test task | — |
| v1-SL-e | PENDING (depends on SL-d merge) | Lane-wide e2e suite (full revocation-during-window-escapes flow exercising SL-c scheduler + SL-d transition end-to-end) | SL-d's e2e tests exercise the producer in isolation (Pending transition + no-sponsor path + NoAction path + wrapper composition); SL-e's tests exercise the full lane (end-to-end through SL-d transition + SL-c scheduler fire/escape) |
| restorative-mechanics-v1 | PENDING (separate PRD; not yet drafted) | `restoration_complete` endpoint + restoration-escape branch (defendant-initiated; admin-attested) | SL-d does NOT wire restoration; SL-c stubbed the EscapeStatus enum; restoration producer comes from this future PRD. SL-d emits zero entries naming restoration |
| v1-jury-mechanics-c | MERGED (governance-v0) | `submit_jury_vote` 9-step rewrite + JM-c TODO marker at `submit_jury_vote.rs:458` pointing at SL-d | **SL-d discharges the JM-c TODO marker**. Task 0 Probe 8 verifies the marker is still present at task-start; Task 2 IMPLEMENT removes the marker block |

**Cross-PRD sequencing (per PRD §15 + §17.1):**

- SL-d parallel-safe with rep-tuning-r3/r4/r5 (different files +
  different concerns).
- SL-d parallel-safe with admin-dashboard-v1.
- SL-d unblocks SL-e (which exercises the full lane end-to-end).
- SL-d closes out the SL producer side; SL-e closes out the full SL
  lane; restorative-mechanics-v1 closes out the restoration-escape
  branch independently.

---

## 7. Preflight guardrails inherited from prior phases

- **DQ #44 — Docker daemon preflight.** Enumerated in §13 Task 0
  (R5 — Probe 0). SL-d's e2e suite uses testcontainers-rs; Docker
  Desktop / dockerd MUST be running before any test invocation
  (Phase 2 e2e on laptop) or local diagnostic cargo. Probe 0
  mandatory.
- **DQ #176 — Sponsor notifications OUT (LOCKED 2026-05-10).**
  SL-d does NOT ship `notify_sponsor_of_pending_liability`. Plan
  §12 enumerates this as out-of-scope.
- **DQ #177 — Compute deltas persistence (LOCKED recompute-at-fire,
  2026-05-10).** `fire_sponsor_liability` calls
  `compute_sponsor_liability` internally at SL-c scheduler
  fire-time. No new migration; no `computed_deltas` column. Plan
  §11 enumerates zero migrations.
- **DQ #178 — `grace_window_for_severity` scope (LOCKED instance,
  2026-05-10).** Helper reads
  `liability.grace_window_<bucket>_hours` at `Scope::Instance`,
  NOT community cascade. Bound in Watchpoint #2 + Task 1 IMPLEMENT.
- **DQ #179 — `sponsor_liability_pending` log entry (LOCKED
  emit-at-transition, 2026-05-10).** Bound in Watchpoint #9 + Task
  2 IMPLEMENT.
- **DQ #180 — Plan §13 task split (LOCKED 4 e2e + 1 unit,
  2026-05-10).** Tasks 3-6 = 4 e2e tests in `mod v1_sl_d_fixtures`
  in `e2e.rs`; Task 7 = 1 combined unit-impl task for tests 5+6 in
  `sponsor_liability.rs::tests`. e2e.rs is **12,819 lines**
  post-SL-c-2 (verified at plan-write time).
- **R1 — `i64::from(...)` on `i32 ↔ i64`.** Bound in §4 watchpoint
  surface and §13 Tasks 3-6 GOTCHAs (where cross-type comparison
  surfaces, e.g. `case.threshold_score` (i64) vs counts (usize)).
- **R5 — pre-phase harness audit enumerates ALL probes.** Bound in
  §13 Task 0. Probes 0..18 listed (SL-d verifies SL-a + SL-b + SL-c
  + JM-c shipped state on top of standard wrapper probes).
- **R6 — uniform `--no-deps -- -D warnings` clippy.** Encoded in
  `cargo-validate-workspace.yml:92`.
- **R7 — `cargo test --no-run -p lemmy_server --test e2e` on tasks
  touching a struct or re-export.** Encoded in
  `cargo-validate-workspace.yml:95`. Bound in Tasks 1, 2, 3-6, 7
  (Task 1 introduces `SponsorDelta` + `compute_sponsor_liability` +
  `fire_sponsor_liability`; Task 2 imports
  `compute_sponsor_liability`; Tasks 3-6 add e2e tests; Task 7 adds
  unit tests).
- **JM-c retro lessons — `feedback_clippy_rerun_after_fix.md`.**
  Bound in §13 Tasks 1-7 GOTCHA — if test edits unmask an
  `unused-imports` lint, the workspace-check workflow's clippy step
  catches.
- **JM-d retro §5 — `feedback_laptop_default_for_validate_pending.md`.**
  Phase 2 e2e local-default per advisor-orchestrator user gate (PR
  #105, 2026-04-28). **Binds SL-d** because SL-d has Phase 2 e2e
  gate.
- **SL-a retro lessons — pseudonym discipline (ADR-015) + Watch
  10.** Bound in §4 watchpoints #9 + #10 + §13 Task 2
  (sponsor_liability_pending payload).
- **SL-c-2 retro lessons (carry-forward) —
  `feedback_lemmy_error_no_std_error.md` Case A canonical sibling
  + `feedback_plan_stub_uniformity_with_canonical_sibling.md`.**
  Bound in §13 Tasks 3-6 stub-shape uniformity (LemmyResult<()>
  uniform throughout the new fixture mod, mirroring v1-SL-b
  `mod v1_sl_b_fixtures`).
- **SL-b retro lessons (carry-forward) — fixture-mod naming
  convention.** SL-d's `mod v1_sl_d_fixtures` follows the
  `mod v1_<phase>_fixtures` pattern.

---

## 8. Flow design

### 8.1 Before state (post-SL-c-2-merge on `governance-v0`)

The sponsor-liability lifecycle today (post-SL-c-2):

- `submit_jury_vote::process_vote` step 8.5 calls **v0 unsplit**
  `apply_sponsor_liability(...)` immediately at vote-tally time
  (`submit_jury_vote.rs:471`). Per-sponsor `reputation_event` rows
  + `sponsor_liability_applied` log entries fire at vote-tally
  time.
- Step 8.9 sets `case.status = Decided` unconditionally on the v0
  path (`submit_jury_vote.rs:493`).
- **No case ever reaches `SponsorLiabilityPending`.** SL-c's
  scheduler batch query finds zero rows on a fresh `governance-v0`
  deployment. The SL lane's main loop is open.
- The TODO marker at `submit_jury_vote.rs:458` is still present
  (JM-c shipped it; subsequent JM phases preserved).
- `sponsor_liability_grace.rs::fire_or_escape_case_inner` at
  `sponsor_liability_grace.rs:510` calls **v0 unsplit**
  `apply_sponsor_liability(...)` (SL-c's posture).
- Registry markers `_SPONSOR_LIABILITY_PENDING (pending; SL-d call
  site)` remain in
  `.claude/rules/governance-log-entry-kind-registry.md`.

### 8.2 After state (post-SL-d-merge)

```
[POST /api/v4/governance/jury/vote]
  process_vote (inside outer run_transaction):
    steps 1-7: load + validate + insert vote → tally → threshold check
              → winning-decision sanction insert (unchanged from JM-c)
    step 8.5: NEW — sponsor-liability evaluation
      if winning_decision != NoAction AND target_person_id is Some:
        let deltas = compute_sponsor_liability(conn, target_id, case_id,
                       community_id, action, &mut cache).await?;
        if deltas.is_empty():    [no-sponsor branch]
          path_kind = Decided           // v0 immediate semantics
        else:                    [pending branch]
          path_kind = Pending
          grace_dur = grace_window_for_severity(case_row.severity, &mut cache, conn).await?;
          grace_expires_at = now + grace_dur;
          target_pseudonym = actor_pseudonym_helper::get_or_create(...).await?;
          sponsors_pseudonyms = [actor_pseudonym_helper::get_or_create(s.sponsor_id) for s in deltas];
      else:
        path_kind = Decided           // NoAction or no-target case
    step 8.9: case UPDATE
      match path_kind:
        Decided => set status=Decided, decided_at=now, winning_decision=...
        Pending => set status=SponsorLiabilityPending, decided_at=now,
                   winning_decision=..., grace_expires_at=Some(...)
    step 9: case_decided log (BOTH paths) + appeal_window UPDATE (BOTH paths)
    step 9b NEW (Pending path only): sponsor_liability_pending log
      governance_log::append(ENTRY_KIND_SPONSOR_LIABILITY_PENDING,
        json!{case_id, target_pseudonym, severity, grace_expires_at,
              sponsors_pseudonyms}, Some(target_pseudonym));
    steps 10-12: public_case_log INSERT + juror reputation events +
                 reporter reputation event
                 [Decided path only — DEFERRED on Pending path to scheduler]
    step 12.5: federation outbox publish (Decided path only)

[scheduler tick (SL-c, unchanged)]
  run_grace_check_batch picks up Pending cases past grace_expires_at
  per case → evaluate_escape_conditions → fire_or_escape_case
    Fire branch:
      sponsor_count = apply_sponsor_liability(conn, ..., cache).await?;
        [thin wrapper: compute_sponsor_liability + fire_sponsor_liability]
      UPDATE case.status = SponsorLiabilityFired
      governance_log::append(_FIRED, ...);
    Escape branch:
      UPDATE case.status = SponsorLiabilityEscaped + liability_escape_reason
      governance_log::append(_ESCAPED, ...);
```

### 8.3 Endpoint changes

NONE. SL-d is a server-internal handler mutation + producer-side
refactor + tests + retro. No new HTTP endpoint, no DTO changes, no
route registration. `submit_jury_vote.rs` request DTO + response
DTO unchanged (PRD §11.3). Plan §13 must NOT include
`crates/api/api_common/src/governance.rs` edits or
`crates/api/routes/src/lib.rs` edits.

---

## 9. Mandatory reading

### 9.1 Brehon design docs (P0)

- `.claude/PRPs/prds/v1-sponsor-liability.prd.md` §9.1 (split —
  full), §9.3 (mutation — full), §9.4 (grace-window helper module),
  §6 (scheduler context), §4.1 (severity-proportional grace
  windows), §4.3 (severity snapshot semantics), §11.3 + §11.4
  (backwards compat), §15 (phase ordering — confirms SL-d depends
  on SL-a + SL-c).
- `.claude/PRPs/prds/v1-jury-mechanics.prd.md` §9.1 (full — the
  9-step integrated `submit_jury_vote` shape SL-d's mutation slots
  into).
- `.claude/PRPs/briefs/sl-d-planning-1.md` — the advisor brief.
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  ADR-005, ADR-008, ADR-010, ADR-013, ADR-014, ADR-015.

### 9.2 Codebase reads (P0 — mirror these patterns)

- `crates/api/api/src/governance/sponsor_liability.rs:1-358` (full
  file, 358 lines). Every helper, every comment, every clamp-math
  branch. **The MIRROR ref for the split** — Task 1 splits this
  body in place, preserves every closure-local, every config-cache
  call, every doc comment. Specifically:
  - Lines 142-358: `apply_sponsor_liability`'s body — split-line
    candidates marked.
  - Lines 112-124: `severity_for_action` — the sibling helper
    SL-d's `grace_window_for_severity` lives next to.
  - Lines 1-50: module doc-comment incl. Watch 10 PII discipline.
- `crates/api/api/src/governance/submit_jury_vote.rs` (965 lines).
  Specifically:
  - Line 458: `TODO(v1-sponsor-liability-d):` marker.
  - Lines 161-180: `process_vote` outer signature + ConfigCache
    declaration.
  - Lines 198-200: `case_row: ModerationCase` load with FOR UPDATE
    (PR #98 cr-2). SL-d's mutation reads `case_row.severity` here.
  - Lines 423-481: `if let Some((scope, action)) = map_decision_to_sanction(...)`
    block. Step 8.5 sponsor-liability branch lives here.
  - Lines 470-480: the v0 `apply_sponsor_liability` call SL-d
    replaces with `compute_sponsor_liability`.
  - Lines 491-498: case UPDATE (status/decided_at/winning_decision)
    that SL-d branches on path_kind.
  - Lines 638-659: appeal_window UPDATE + `case_decided` log; both
    fire on BOTH paths.
- `crates/api/api/src/governance/sponsor_liability_grace.rs:510`
  (SL-c's call site). Confirm: `apply_sponsor_liability` invocation
  takes `(conn, target_id, case_id, re_loaded.community_id, action,
  &mut per_case_cache)` — preserved by Task 1 wrapper.
- `crates/api/api/src/governance/governance_log.rs:1-120` (api
  shim). Confirm `pub use ENTRY_KIND_SPONSOR_LIABILITY_PENDING` at
  line 76.
- `crates/db_schema/src/source/governance/governance_log.rs:199` —
  `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` const declaration.
- `crates/api/api/src/governance/actor_pseudonym_helper.rs` —
  `get_or_create` source; SL-d uses for `target_pseudonym` +
  `sponsors_pseudonyms`.
- `crates/api/api/src/governance/config.rs:925-945` — grace-window
  default consts (`DEFAULT_LIABILITY_GRACE_WINDOW_*` /
  `DEFAULT_LIABILITY_GRACE_WINDOW_MAXIMUM_HOURS`).
- `crates/db_schema_file/src/enums.rs` lines covering 12
  `CaseStatus` variants (393-432); `CaseSeverity` variants (verify
  line range at task-start); `JuryDecision` variants (around 105 in
  submit_jury_vote.rs's ALL_JURY_DECISIONS const); `SanctionAction`
  variants.
- `crates/db_schema_file/src/schema.rs:772-801` — `moderation_case`
  table columns (esp. `severity`, `status`, `decided_at`,
  `target_person_id`, `community_id`, `winning_decision`,
  `appeal_window_expires_at`, `grace_expires_at`,
  `liability_escape_reason`).
- `crates/db_schema/src/source/governance/moderation_case.rs` —
  `ModerationCase` struct + `ModerationCaseInsertForm` (incl. SL-a
  fields).
- `crates/db_schema/src/source/governance/sanction.rs` —
  `Sanction` struct + `SanctionInsertForm`.
- `crates/db_schema/src/source/governance/surety.rs` — `Surety`
  struct + `SuretyInsertForm`.
- **Canonical Case A error-shape sibling:**
  `crates/server/tests/e2e.rs:11001-11924` (`mod v1_sl_b_fixtures`).
  Read in full to internalise the uniform `LemmyResult<T>` outer
  pattern. Tasks 3-6 mirror this shape.
- `crates/server/tests/e2e.rs:11927-...` (`mod v1_sl_c_fixtures`)
  — immediate predecessor fixture-mod; helper signatures + test fn
  signatures use `LemmyResult<()>` per the post-replan canonical
  shape.
- `crates/server/tests/e2e.rs:907` —
  `governance_log_hash_chain_holds` test pattern; SL-d Test #1
  follows the same `governance_log::table.filter(...)` query
  pattern for `_SPONSOR_LIABILITY_PENDING` row count + payload
  assertions.

### 9.3 Rules (P0 — auto-loaded)

- `.claude/rules/decision-queue.md` — DQ schema-v2; attribution
  integrity; mid-task push; planner Recipe 2 self-resolution.
- `.claude/rules/branch-manager.md` — file-ownership boundaries;
  BM cuts `phase-v1-SL-d` after planner ships.
- `.claude/rules/phase-branch.md` — phase-branch +
  PR-into-`governance-v0` flow.
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy`
  mandatory.
- `.claude/rules/governance-log-entry-kind-registry.md` —
  pre-landed-const exemption (`_SPONSOR_LIABILITY_PENDING` at line
  199, SL-a-shipped, SL-d call site discharges the (pending)
  marker at SL-d retro flip).
- `.claude/rules/cargo-output-capture.md` +
  `.claude/rules/no-cargo-output-paste.md` — cargo invocation
  discipline.
- `.claude/rules/pre-phase-harness-audit.md` — Task 0 audit shape.
- `.claude/rules/advisor-orchestrator.md` §G4 classifier (4a/4b/4c
  rows) — SL-d's e2e tasks mirror Case A canonical sibling shape;
  Case C (mixed shapes) is a hard refusal.

### 9.4 Lessons (P0 — bound to §13 decisions)

(See §2 "Lessons that bind §13 decisions" — full enumeration. Most
load-bearing: `feedback_lemmy_error_no_std_error.md` Case A,
`feedback_junior_worker_e2e_edit_hang.md`,
`feedback_complexity_score_pre_split.md`,
`feedback_plan_stub_uniformity_with_canonical_sibling.md`.)

### 9.5 External documentation

- chrono `Duration::hours(i64)` API — used by
  `grace_window_for_severity`. Citation-only.
- diesel `RunQueryDsl` + `AsyncPgConnection` — existing usage.
- testcontainers-rs Postgres lifecycle — existing usage across
  e2e.rs.

---

## 10. Patterns to mirror

Per `feedback_advisor_watchpoint_specificity.md` + DQ #147 (paired
canonical mirrors).

### 10.1 v0 `apply_sponsor_liability` body — the MIRROR for the split

**SOURCE:** `crates/api/api/src/governance/sponsor_liability.rs:142-358`.

The v0 body's structure (read at task start to internalise):

```rust
pub(crate) async fn apply_sponsor_liability(
  conn: &mut AsyncPgConnection,
  target_person_id: PersonId,
  case_id: ModerationCaseId,
  community_id: Option<CommunityId>,
  action: SanctionAction,
  cache: &mut ConfigCache,
) -> LemmyResult<usize> {
  // === COMPUTE PHASE BEGINS ===
  // (1) severity = severity_for_action(action);                  [pure]
  // (2) raw_delta = config::get_int(severity.config_key());       [read]
  // (3) sponsor_ids = SELECT FROM surety WHERE active            [read]
  // (4) early return Ok(0) if sponsor_count == 0                 [pure]
  // (5) per-sponsor: per_sponsor_base, remainder, founder_check  [reads + arithmetic]
  // (6) per-sponsor: clamp lookup against reputation_snapshot    [read]
  // (7) per-sponsor: final_delta + clamped_from + i32 conversion [pure]
  // === COMPUTE PHASE ENDS ===
  // === FIRE PHASE BEGINS (per-sponsor inside the for-loop) ===
  // (8) reputation_event::insert                                 [WRITE]
  // (9) governance_log::append(_APPLIED)                          [WRITE]
  // (10) optional governance_log::append(_CLAMPED)                [WRITE]
  // === FIRE PHASE ENDS ===
  Ok(sponsor_count)
}
```

**The split-line is between (7) and (8).** Compute returns
`Vec<SponsorDelta>` populated from (1)-(7); fire iterates the
deltas and runs (8)-(10) per delta.

**GOTCHA (preserve closure-locals as struct fields):** the v0
closure-local bindings at lines 212-288 (`pre_multiplier_delta`,
`remainder_bump`, `multiplier`, `post_multiplier_delta`,
`current_endorsement_strength`, `final_delta`, `clamped_from`,
`is_founder`) become `SponsorDelta` struct fields. The fire phase
re-builds the v0 governance_log payload from the struct, not from
recomputed closure locals. **Byte-identical payload preservation
is the wrapper's correctness contract.**

**GOTCHA (cache argument plumbing):** `compute_sponsor_liability`
and `fire_sponsor_liability` BOTH take `cache: &mut ConfigCache`
explicitly — `fire_sponsor_liability` does NOT need the cache for
any config read (the writes don't read config) but the wrapper
preserves the v0 signature, so both inner fns take the cache. The
wrapper passes the same `&mut cache` through to both. Per
`feedback_multi_write_handlers_need_transactions.md` single-handler
pattern.

### 10.2 Pure-fn / write-fn split — the canonical pattern

**SOURCE:** `crates/api/api/src/governance/sponsor_liability_grace.rs:90-540`
(SL-c's `evaluate_escape_conditions` is pure-read;
`fire_or_escape_case` is the writer). Same compute-vs-fire split,
just at a different granularity.

**GOTCHA (purity grep):** after Task 1 ships,
`rg -nE 'insert_into|update\(|append\(|persist'
crates/api/api/src/governance/sponsor_liability.rs` returns ONLY
matches inside `fire_sponsor_liability`'s body or inside the
wrapper's call to `fire_sponsor_liability`. NEVER inside
`compute_sponsor_liability`.

### 10.3 Inside-handler-transaction mutation — the MIRROR for the mutation

**SOURCE:** `crates/api/api/src/governance/submit_jury_vote.rs:140-674`
(`process_vote`).

Specifically:

- `submit_jury_vote.rs:140-146` — outer `run_transaction` call.
- `submit_jury_vote.rs:161-180` — `process_vote` declaration +
  cache.
- `submit_jury_vote.rs:198-200` — case_row FOR UPDATE load.
- `submit_jury_vote.rs:423-481` — sanction insert + step 8.5
  sponsor-liability branch.
- `submit_jury_vote.rs:483-498` — case status flip (step 8.9 — the
  line SL-d mutates).
- `submit_jury_vote.rs:500-526` — public_case_log + log entry
  (steps 10-11; deferred on Pending path).
- `submit_jury_vote.rs:528-607` — juror + reporter reputation
  events (steps 11-12; deferred on Pending path).
- `submit_jury_vote.rs:609-659` — appeal_window + case_decided log
  (BOTH paths).

**GOTCHA (single transaction):** SL-d's mutation modifies the
existing `process_vote` body. Does NOT add a new
`run_transaction`. Does NOT add a new ConfigCache (uses the
existing `&mut cache` at line 170). Per
`feedback_multi_write_handlers_need_transactions.md`.

**GOTCHA (Pending-branch step ordering):** the case UPDATE happens
once (single UPDATE statement), not twice. Pre-SL-d, lines 491-498
set status + decided_at + winning_decision. SL-d's branch augments
to: when path_kind is Pending, the same UPDATE additionally sets
`grace_expires_at = Some(grace_expires_at)` and writes
`status = SponsorLiabilityPending` instead of `Decided`. One
UPDATE, two outcome shapes.

### 10.4 governance_log payload schemas (Task 2 binding)

**SOURCE:** `.claude/rules/governance-log-entry-kind-registry.md`
v1-SL-a entry-kinds section + ADR-015.

`sponsor_liability_pending` payload (SL-d's emit at the
`Decided → SponsorLiabilityPending` transition; const at
`crates/db_schema/src/source/governance/governance_log.rs:199`):

```json
{
  "case_id": <i32>,
  "target_pseudonym": "<UUID-string>",
  "severity": "<minor|moderate|severe>",
  "grace_expires_at": "<DateTime<Utc> ISO 8601>",
  "sponsors_pseudonyms": ["<UUID-string>", "<UUID-string>", ...]
}
```

**GOTCHA (ADR-015 — pseudonym discipline):** `target_pseudonym`
is the `actor_pseudonym_helper::get_or_create(target_id)` UUID;
raw `target_id.0` MUST NOT appear. `sponsors_pseudonyms` is an
array of the per-sponsor pseudonyms (from the
`Vec<SponsorDelta>::sponsor_id` list, each looked up via
`actor_pseudonym_helper`). Test #1 includes defensive
`assert_ne!(json["target_pseudonym"].as_str().unwrap(),
&format!("{}", target_id.0))`.

**GOTCHA (severity string serialisation):** the `severity` field
is the lowercase string from `LiabilitySeverity::as_str()` at
`sponsor_liability.rs:82-89` — `"minor" | "moderate" | "severe"`.
Map from `case_row.severity: CaseSeverity` via Task 1's mapping
helper. Pure transformation.

### 10.5 Test fixture mod shape (Tasks 3-6 binding)

**SOURCE:** `crates/server/tests/e2e.rs:11001-11924` (`mod
v1_sl_b_fixtures` — the canonical Case A error-shape sibling).
Post-SL-c-2 the immediate predecessor is
`crates/server/tests/e2e.rs:11927-...` (`mod v1_sl_c_fixtures`).

```rust
mod v1_sl_d_fixtures {
  use super::*;
  use chrono::Utc;
  use diesel::{ExpressionMethods, QueryDsl, insert_into};
  use diesel_async::{AsyncPgConnection, RunQueryDsl};
  use lemmy_db_schema::{
    newtypes::{ModerationCaseId, PersonId, SuretyId},
    source::governance::{
      moderation_case::ModerationCaseInsertForm,
      sanction::SanctionInsertForm,
      surety::SuretyInsertForm,
    },
  };
  use lemmy_db_schema_file::{
    enums::{
      CaseSeverity, CaseStatus, CaseTargetType, JuryDecision,
      SanctionAction, SanctionScope,
    },
    schema::{governance_log, moderation_case, reputation_event, sanction, surety},
  };
  use lemmy_utils::error::LemmyResult;
  use serde_json::Value;

  // Helpers (signatures from canonical sibling at e2e.rs:11001-11050):

  async fn seed_target_with_sureties(
    conn: &mut AsyncPgConnection,
    sponsor_count: usize,
  ) -> LemmyResult<(PersonId, Vec<PersonId>)> {
    // Returns (target_person_id, vec![sponsor1, sponsor2, ...])
    // Sets up persons + endorsements + sureties.
    // Body shape mirrors mod v1_sl_b_fixtures helpers verbatim.
    todo!()
  }

  // ... 4 #[tokio::test] async fn ... per Tasks 3-6, all returning LemmyResult<()> ...
}
```

**GOTCHA (R4 — test fn names):** lowercase snake_case.

**GOTCHA (e2e fixture-mod placement):** the new mod opens AFTER
the last existing fixture mod (`mod v1_sl_c_fixtures` ends at
e2e.rs's final `}` line near 12,819). Task 3's IMPLEMENT at
task-start runs:

```bash
grep -n '^mod v1_' crates/server/tests/e2e.rs | tail -1
# EXPECT: mod v1_sl_c_fixtures
```

— and anchors the new `mod v1_sl_d_fixtures` AFTER its closing `}`.

**GOTCHA (Case A canonical sibling):** every helper signature ends
`-> LemmyResult<T>`; every test fn signature ends
`-> LemmyResult<()>`. Every `?` is bare. NO `Box<dyn Error>` outer;
NO `.map_err` bridges. Per
`feedback_lemmy_error_no_std_error.md` Case A. The canonical
sibling at `crates/server/tests/e2e.rs:11001-11924` (`mod
v1_sl_b_fixtures`) demonstrates 7 helpers + 9-10 test fns in this
shape — read in full at task-start.

### 10.6 `governance_log` row count assertion pattern

**SOURCE:** `crates/server/tests/e2e.rs:907`
(`governance_log_hash_chain_holds`).

```rust
let count: i64 = governance_log::table
  .filter(governance_log::entry_kind.eq("sponsor_liability_pending"))
  .count()
  .get_result(conn)
  .await?;
assert_eq!(count, 1);
```

**GOTCHA (R1 — i64 count comparison):** `.count()` returns `i64`;
compare against literal (no `i64::from` needed for literal; needed
when comparing against `usize::len()` of a Vec).

### 10.7 Unit test pattern for `sponsor_liability.rs::tests` (Task 7 binding)

**SOURCE:** read existing `#[cfg(test)] mod tests { ... }` blocks
under `crates/api/api/src/governance/*.rs` at task start. Skeleton:

```rust
#[cfg(test)]
mod tests {
  use super::*;
  use lemmy_utils::error::LemmyResult;

  #[tokio::test]
  async fn compute_sponsor_liability_idempotent() -> LemmyResult<()> {
    // 1. testcontainers-rs Postgres (or lighter setup if available);
    //    seed target + 2 sponsors + active sureties.
    // 2. let deltas1 = compute_sponsor_liability(...).await?;
    // 3. let deltas2 = compute_sponsor_liability(...).await?;
    // 4. assert_eq!(deltas1, deltas2);
    // 5. assert no reputation_event rows written.
    // 6. assert no governance_log rows written.
    Ok(())
  }

  #[tokio::test]
  async fn grace_window_for_severity_reads_correct_config_key() -> LemmyResult<()> {
    // 1. testcontainers-rs Postgres; SL-a-seeded config keys present.
    // 2. for each severity tier (Minor / Moderate / Severe):
    //      let dur = grace_window_for_severity(severity, &mut cache, conn).await?;
    //      assert_eq!(dur, Duration::hours(<expected>));  // 24/72/168
    Ok(())
  }
}
```

**GOTCHA:** `SponsorDelta` must derive `PartialEq + Debug` for
`assert_eq!` to work. Task 1 IMPLEMENT specifies these derives.

**GOTCHA (testcontainers in unit tests):** if `crates/api/api`
doesn't already pull in testcontainers, the unit test framework
must use a different setup pattern. Read existing patterns in
`crates/api/api/src/governance/*.rs` at Task 7 start; if no
in-crate unit-test pattern exists, file a planner DQ asking
whether to relocate the test to `crates/server/tests/e2e.rs`
instead. Default lean: keep in `sponsor_liability.rs::tests` per
DQ #180; if infrastructure friction, escalate.

---

## 11. Files to change

### `crates/api/api` crate

- `crates/api/api/src/governance/sponsor_liability.rs` (358 lines):
  - SPLIT `apply_sponsor_liability` body into `compute_sponsor_liability`
    + `fire_sponsor_liability` + thin wrapper `apply_sponsor_liability`
    preserving v0 signature. Plus new
    `pub(crate) struct SponsorDelta { ... }` after `LiabilitySeverity`
    decl.
  - ADD `pub(crate) async fn grace_window_for_severity(severity:
    CaseSeverity, cache: &mut ConfigCache, conn: &mut
    AsyncPgConnection) -> LemmyResult<chrono::Duration>` sibling of
    `severity_for_action`.
  - Plus internal helper `liability_severity_from_case_severity`.
  - **Task 1**.
  - Task 7 ADDS `#[cfg(test)] mod tests { ... }` at end-of-file with
    2 unit tests. **Task 7**.

- `crates/api/api/src/governance/submit_jury_vote.rs` (965 lines):
  - REPLACE the v0 `apply_sponsor_liability(...)` call at line 471
    with `compute_sponsor_liability(...)` returning
    `Vec<SponsorDelta>`.
  - ADD branch logic (path_kind = Pending vs Decided) at step 8.5.
  - MODIFY case UPDATE at lines 491-498 to branch on path_kind
    (status = SponsorLiabilityPending vs Decided; grace_expires_at
    set on Pending path).
  - ADD `sponsor_liability_pending` log emit on Pending path (per DQ
    #179 LOCKED).
  - GUARD steps 10-12 (public_case_log + juror reputation +
    reporter reputation) on path_kind = Decided.
  - GUARD step 12.5 (federation outbox) on path_kind = Decided.
  - PRESERVE step 9 (`appeal_window_expires_at` UPDATE +
    `case_decided` log on BOTH paths).
  - Remove the JM-c TODO comment at line 458.
  - **Task 2**.

### `crates/server` crate

- `crates/server/tests/e2e.rs` (currently 12,819 lines): append new
  `mod v1_sl_d_fixtures` AFTER `mod v1_sl_c_fixtures` (closes near
  line 12,819 — verify at task-start). Within the mod, ship 4
  tests via 4 anchor-Edit tasks (Tasks 3-6). Task 3 ships the mod
  shell + helpers + test #1; Tasks 4-6 anchor-insert subsequent
  tests inside the same mod. **Tasks 3-6**.

### Meta files (rules + reports)

- `.claude/rules/governance-log-entry-kind-registry.md` — at retro
  time (Task 8), flip `(pending) → (active)` marker on the SL-d
  fire-site row: `_SPONSOR_LIABILITY_PENDING`. **Task 8** (retro).
- `.claude/PRPs/reports/v1-SL-d-retro.md` — CREATE retro per
  `feedback_retro_not_report.md` +
  `feedback_four_role_retro_signals.md` +
  `feedback_retro_task_complexity_score.md`. **Task 8**.

### Files explicitly NOT touched

- `crates/api/api/src/governance/sponsor_liability_grace.rs` —
  SL-c shipped; SL-d's wrapper preserves the call site at line
  510; SL-d does NOT modify (per brief §2.3 default lean).
- `crates/api/api/src/governance/mod.rs` — pub mod declarations
  unchanged.
- `crates/routes/src/utils/scheduled_tasks.rs` — SL-c shipped the
  scheduler block; SL-d does NOT modify.
- `crates/api/api_crud/src/governance/revoke_endorsement.rs` —
  SL-b shipped; SL-d does NOT modify.
- `crates/api/api/src/governance/governance_log.rs` (api shim) —
  no re-export changes (SL-a-shipped consts already re-exported).
- `crates/db_schema/src/source/governance/governance_log.rs` — no
  const additions (SL-a-shipped).
- `crates/db_schema/src/source/governance/{moderation_case,sanction,surety,endorsement}.rs`
  — no struct changes.
- `crates/db_schema_file/src/{enums,schema}.rs` — no schema
  changes.
- `migrations/**` — zero migrations.
- `crates/api/api/src/governance/config.rs` — no const additions.
- `crates/db_views/governance_case/src/impls.rs` — view-crate
  unchanged.
- `crates/api/api_common/src/governance.rs` — no DTO additions.
- `crates/api/routes/src/lib.rs` — no route registration changes.
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
  `.coderabbit.yaml` — no dep / build-config / review-config
  changes.
- `.github/workflows/**` — no workflow YAML changes.

---

## 12. NOT building in v1-SL-d

- **Notification UX (`notify_sponsor_of_pending_liability`)** — out
  per DQ #176 LOCKED. PRD §13 OQ-V1-SL-03 (v3 polish).
- **`computed_deltas` snapshot column on `moderation_case`** — out
  per DQ #177 LOCKED. Recompute-at-fire is canonical.
- **Community-cascaded `grace_window_for_severity`** — out per DQ
  #178 LOCKED. Instance-only matches SL-c scheduler.
- **5th and 6th e2e tests in `e2e.rs`** — DQ #180 LOCKED: tests 5
  (compute idempotency) and 6 (grace-window mapping) are unit
  tests in `sponsor_liability.rs::tests`, NOT e2e tests.
- **`apply_sponsor_liability` call-site update in SL-c
  scheduler** — out per brief §2.3 default lean.
- **`restoration_complete` endpoint + restoration-escape branch**
  — restorative-mechanics-v1 PRD's deliverable.
- **`Decided → SponsorLiabilityFired` transition logic** — SL-c
  (already shipped at `sponsor_liability_grace.rs:519`).
- **`Decided → SponsorLiabilityEscaped` transition logic at
  vote-tally time** — SL-b (revocation-time escape) + SL-c
  (scheduler-tick escape); SL-d does NOT add a third escape branch.
- **e2e behavioural tests for the full lane** — SL-e's deliverable.
- **Cross-instance federation of grace-window events** — out per
  ADR-014 + PRD §2 OUT.
- **Step-up auth for admin-driven scheduler runs** — out per PRD
  §12.3 (v2 reservation).
- **New ENTRY_KIND_*** consts** — all needed shipped in SL-a.
- **New CaseStatus variants** — SL-a shipped 3.
- **New `governance_config` seeds** — SL-a shipped 13.
- **Schema migrations** — none.
- **Backfill of v0 cases** — SL-a Task 1 already backfilled.
- **Multi-sponsor `all_revocation` / `majority_revocation` rules at
  vote-tally time** — out per PRD §13 OQ-V1-SL-01.
- **Sponsor-of-sponsor liability chain depth (multi-hop)** — out
  per PRD §13 OQ-V1-SL-02 (v2 candidate).

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task**. SL-d ships 7
impl tasks + Task 0 pre-flight + retro = 9 tasks total. No `[P]`
cohorts (Task 2 imports Task 1's symbols; Tasks 3-6 all
`modifies: crates/server/tests/e2e.rs` so YAML overlap rule
refuses cohort dispatch — they ship serially. Task 7 modifies
`sponsor_liability.rs` which Task 1 wrote, so Task 7 depends on
Task 1's commit being on the phase branch). Task 0 is always
non-`[P]`.

> **Cohort dispatch (advisor-side):** No `[P]` cohorts in SL-d.
> All tasks dispatch serially per `advisor-orchestrator.md`.
> Per-task `HANDOVER:` commit trailer recommended where the next
> task benefits.

> **Shape G (Layer G2 push-and-exit):** §13 task bodies do NOT
> inline cargo invocations. Each task ends with a push to the
> worker branch; the impl-task subagent writes a
> `kind: "validate-pending"` DQ entry referencing
> `cargo-validate-workspace.yml` per
> `.claude/rules/decision-queue.md` schema-v2.

### Task 0: Pre-flight harness audit + branch verification + SL-a/SL-b/SL-c/JM-c state confirmation

**Goal:** verify environment + branch (`phase-v1-SL-d`) + SL-a +
SL-b + SL-c + JM-c shipped state all merged on `governance-v0`.

**FILES:**

```yaml
creates: []
modifies: []
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate ALL probes):**

```bash
# Probe 0 — Docker daemon (e2e harness uses testcontainers-rs)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING — start Docker Desktop / dockerd before continuing"; exit 1; }

# Probe -1 — submodule init (Linux/Junior worktree quirk)
git submodule status > /tmp/sl-d-task0-submodule.log 2>&1
if grep -q '^-' /tmp/sl-d-task0-submodule.log; then
  git submodule update --init --recursive > /tmp/sl-d-task0-submodule-init.log 2>&1
  echo "submodule init exit: $?"
fi

# Probe 1 — branch verification
git branch --show-current
# EXPECT: phase-v1-SL-d (BM-task cuts before Task 1, AFTER plan ships to governance-v0)

# Probe 2 — SL-a state confirmation: 3 CaseStatus variants present
rg -n 'SponsorLiabilityPending|SponsorLiabilityFired|SponsorLiabilityEscaped' crates/db_schema_file/src/enums.rs | head
# EXPECT: 3+ matches near lines 416/421/427

# Probe 3 — SL-a state confirmation: schema columns present
rg -n 'grace_expires_at -> Nullable<Timestamptz>|liability_escape_reason -> Nullable<Jsonb>' crates/db_schema_file/src/schema.rs | head
# EXPECT: lines 799 + 800

# Probe 4 — SL-a state confirmation: ENTRY_KIND consts declared
rg -n 'ENTRY_KIND_SPONSOR_LIABILITY_FIRED|ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED|ENTRY_KIND_SPONSOR_LIABILITY_PENDING' crates/db_schema/src/source/governance/governance_log.rs | head
# EXPECT: 3 matches at lines 197/198/199

# Probe 5 — SL-a state confirmation: shim re-exports
rg -n 'ENTRY_KIND_SPONSOR_LIABILITY_FIRED|ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED|ENTRY_KIND_SPONSOR_LIABILITY_PENDING' crates/api/api/src/governance/governance_log.rs | head
# EXPECT: 3 matches in pub use block

# Probe 6 — SL-a state confirmation: config keys + defaults seeded
rg -n 'DEFAULT_LIABILITY_GRACE_WINDOW_MINOR_HOURS|DEFAULT_LIABILITY_GRACE_WINDOW_MODERATE_HOURS|DEFAULT_LIABILITY_GRACE_WINDOW_SEVERE_HOURS|DEFAULT_LIABILITY_GRACE_WINDOW_MAXIMUM_HOURS' crates/api/api/src/governance/config.rs | head
# EXPECT: 4+ const declarations near lines 925-945

# Probe 7 — SL-c module landed + scheduler call site signature stable
rg -n 'pub mod sponsor_liability_grace;' crates/api/api/src/governance/mod.rs
# EXPECT: 1 line
test -f crates/api/api/src/governance/sponsor_liability_grace.rs && echo "SL-C MODULE PRESENT" || { echo "SL-C MODULE NOT FOUND — SL-d cannot proceed"; exit 1; }
rg -n 'sponsor_liability::apply_sponsor_liability' crates/api/api/src/governance/sponsor_liability_grace.rs
# EXPECT: 1 match near line 510 (SL-c's call site SL-d wrapper preserves)

# Probe 8 — JM-c TODO marker present at submit_jury_vote.rs:458
rg -n 'TODO\(v1-sponsor-liability-d\)' crates/api/api/src/governance/submit_jury_vote.rs
# EXPECT: 1 line near line 458 (the SL-d work site marker; if missing, file DQ before Task 2)

# Probe 9 — v0 apply_sponsor_liability signature confirmation
rg -n 'pub\(crate\) async fn apply_sponsor_liability' crates/api/api/src/governance/sponsor_liability.rs
# EXPECT: 1 match at line 142; signature preserved by Task 1 wrapper

# Probe 10 — SL-c call-site present (post-c-1-c-2 merge)
rg -n 'apply_sponsor_liability\(' crates/api/api/src/governance/sponsor_liability_grace.rs
# EXPECT: 1 line near 510 (Fire branch invocation; SL-d preserves)

# Probe 11 — SL-b merge confirmation: revoke_endorsement handler shipped
test -f crates/api/api_crud/src/governance/revoke_endorsement.rs && echo "SL-B SHIPPED" || echo "SL-B NOT YET MERGED — surface to advisor"

# Probe 12 — last fixture mod identification (Task 3 anchor)
rg -n '^mod v1_' crates/server/tests/e2e.rs | tail -1
# EXPECT: mod v1_sl_c_fixtures (post-SL-c-2-merge)

# Probe 13 — e2e.rs line count (sanity check — should be near 12,819 per plan-write time)
wc -l crates/server/tests/e2e.rs
# EXPECT: ~12,819 lines (DQ #180 LOCKED at plan-write time)

# Probe 14 — sanction multiplicity invariant (1:1 with case)
rg -n 'insert_into\(sanction::table\)' crates/api/api crates/api/api_crud
# EXPECT: 1 match at submit_jury_vote.rs:435 (single insert per case — 1:1 multiplicity)

# Probe 15 — concurrent-PR check (no other PR touches SL-d-owned files)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("sponsor_liability\\.rs|submit_jury_vote\\.rs|tests/e2e\\.rs|sponsor_liability_grace\\.rs")) | {number, title, headRefName}'
# EXPECT: empty output (no concurrent PRs touching SL-d-owned files)

# Probe 16 — registry marker still pending for _SPONSOR_LIABILITY_PENDING
rg -n '\(pending\)' .claude/rules/governance-log-entry-kind-registry.md | rg SPONSOR_LIABILITY_PENDING
# EXPECT: 1 line (the SL-d call-site marker; Task 8 retro flips to (active))

# Probe 17 — Shape G workflow YAMLs accessible
ls .github/workflows/cargo-validate-workspace.yml .github/workflows/cargo-test-e2e.yml
# EXPECT: both files exist

# Probe 18 — DQ pending status (advisory)
python3 -c "import json; d=json.load(open('.claude/decision-queue.json')); print('pending:', [(e['id'], e.get('kind'), e.get('from')) for e in d.get('pending',[])])"
```

**EXPECT:** Probes 0..18 exit 0 (or, for Probe 0/1/7/8, exit 1
with explicit STOP).

**No commit at Task 0** — verification only.

### Task 1: `apply_sponsor_liability` compute/fire split + `grace_window_for_severity` helper

**ACTION:** in `crates/api/api/src/governance/sponsor_liability.rs`,
split `apply_sponsor_liability` into `compute_sponsor_liability` +
`fire_sponsor_liability` + thin wrapper preserving v0 signature.
Define `pub(crate) struct SponsorDelta` (with field set per §10.1).
Add `pub(crate) async fn grace_window_for_severity(severity:
CaseSeverity, cache: &mut ConfigCache, conn: &mut
AsyncPgConnection) -> LemmyResult<chrono::Duration>` sibling of
`severity_for_action`. Add internal helper
`liability_severity_from_case_severity(CaseSeverity) ->
LiabilitySeverity` (or fold into `grace_window_for_severity`'s
body).

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/sponsor_liability.rs   # split + helpers + struct
```

**IMPLEMENT (file 1 of 1):** in
`crates/api/api/src/governance/sponsor_liability.rs`:

1. After `LiabilitySeverity` decl (around line 90), ADD
   `pub(crate) struct SponsorDelta { ... }` per §10.1 field shape.
   Derives: `Debug, Clone, PartialEq` (PartialEq required for Task
   7 assert_eq; not Eq because f64 doesn't impl Eq).
2. ADD `fn liability_severity_from_case_severity(severity:
   CaseSeverity) -> LiabilitySeverity` per §4.1 mapping. Exhaustive
   match per ADR-013 (no `_ =>` arms). Variant set to confirm at
   task-start by reading `crates/db_schema_file/src/enums.rs`
   `CaseSeverity` decl. **GOTCHA:** if `CaseSeverity` variants
   don't cleanly map to {Minor, Moderate, Severe}, file a planner
   DQ `kind: "blocker"` from `from: "impl"` before authoring.
3. REPLACE `apply_sponsor_liability`'s body at line 142+ with three
   pub(crate) async fns: `compute_sponsor_liability` (returns
   `Vec<SponsorDelta>`, no writes), `fire_sponsor_liability` (takes
   the deltas, performs writes), `apply_sponsor_liability` (thin
   wrapper = `compute + fire`, signature byte-identical to v0).
4. ADD `pub(crate) async fn grace_window_for_severity(severity:
   CaseSeverity, cache: &mut ConfigCache, conn: &mut
   AsyncPgConnection) -> LemmyResult<chrono::Duration>` after
   `severity_for_action` at line 124. Body reads
   `liability.grace_window_<bucket>_hours` at `Scope::Instance` per
   DQ #178; returns `chrono::Duration::hours(value)`.

**MIRROR:** §10.1 (v0 body structure → split-line at the (7)/(8)
boundary); §10.2 (pure-fn / write-fn split pattern from
`sponsor_liability_grace.rs:90-540`).

**GOTCHA (preserve byte-identical governance_log payload):** v0's
log payload at lines 322-337 references closure-locals
(`pre_multiplier_delta`, `multiplier`, `post_multiplier_delta`,
`final_delta`, `is_founder`). After the split,
`fire_sponsor_liability` reconstructs these from `SponsorDelta`
fields. Read v0's log payload line-by-line; ensure every field
maps to a `SponsorDelta` field.

**GOTCHA (PartialEq derive on SponsorDelta with f64 multiplier):**
f64 does not implement Eq, only PartialEq. Use `#[derive(Debug,
Clone, PartialEq)]` on SponsorDelta.

**GOTCHA (cache argument plumbing):** both
`compute_sponsor_liability` and `fire_sponsor_liability` take
`cache: &mut ConfigCache` even though only compute uses it. Wrapper
passes through. Per §10.1 GOTCHA.

**GOTCHA (Watchpoint #5 — purity grep):** at task-end run:
```bash
rg -nE 'insert_into|update\(|append\(|persist' crates/api/api/src/governance/sponsor_liability.rs
```
Zero matches inside `compute_sponsor_liability`'s body.

**GOTCHA (Watchpoint #4 — wrapper signature byte-identical):** at
task-end:
```bash
git diff governance-v0..HEAD -- crates/api/api/src/governance/sponsor_liability.rs > /tmp/sl-d-task1-diff.log 2>&1
rg 'pub\(crate\) async fn apply_sponsor_liability' /tmp/sl-d-task1-diff.log
```
The diff for the `apply_sponsor_liability` line must be a context
line (no leading `+` or `-`). If the line shows changes, STOP.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `refactor(v1-SL-d): split apply_sponsor_liability into compute + fire + wrapper, add grace_window_for_severity (task 1)`

### Task 2: `submit_jury_vote` mutation — Decided → SponsorLiabilityPending transition + grace-window snapshot + deferred writes + ADR-013 sweep

**ACTION:** in
`crates/api/api/src/governance/submit_jury_vote.rs`, replace the
v0 `apply_sponsor_liability` call at line 471 with
`compute_sponsor_liability`. Add path_kind branch logic at step
8.5/8.9. Modify case UPDATE to branch on path_kind. Emit
`sponsor_liability_pending` log on Pending path. Guard steps 10-12
+ 12.5 on path_kind = Decided. Preserve step 9 on BOTH paths.
Remove the JM-c TODO marker.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/submit_jury_vote.rs   # mutation + ADR-013 sweep
```

**IMPLEMENT (file 1 of 1):** in
`crates/api/api/src/governance/submit_jury_vote.rs`:

1. UPDATE imports at lines 42-48 to add
   `ENTRY_KIND_SPONSOR_LIABILITY_PENDING`.
2. REPLACE the v0 step 8.5 block at lines 470-480: invoke
   `compute_sponsor_liability` instead of v0
   `apply_sponsor_liability`; branch on `Vec::is_empty()` to set
   `path_kind` (Pending / Decided) + collect `grace_expires_at` +
   `deltas_for_pending`. Define `enum SLDPathKind { Pending,
   Decided }` (derive PartialEq).
3. MODIFY case UPDATE at lines 491-498 to branch on path_kind:
   `status` = SponsorLiabilityPending vs Decided; `grace_expires_at`
   = Some(t) on Pending else None.
4. ADD step 9b on Pending path: emit
   `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` governance_log entry per
   §10.4 payload schema. Position: AFTER case UPDATE, BEFORE step
   10. Build payload with `target_pseudonym` +
   `sponsors_pseudonyms` (per-sponsor lookups via
   `actor_pseudonym_helper::get_or_create`) + `severity` (from
   `liability_severity_from_case_severity` mapping). NEVER raw
   `*_id.0` values per ADR-015.
5. GUARD steps 10-12 (lines 500-607) on `path_kind ==
   SLDPathKind::Decided`. On Pending these are deferred to SL-c
   scheduler fire/escape time per PRD §11.4.
6. GUARD step 12.5 (federation outbox at lines 661-667) on
   `path_kind == SLDPathKind::Decided`.
7. PRESERVE step 9 (`appeal_window_expires_at` UPDATE +
   `case_decided` log) — these fire on BOTH paths. NO CHANGE.
8. REMOVE the JM-c TODO comment block at lines 456-469.
9. ADR-013 sweep — at task-end:
   ```bash
   rg -nE 'match.*case\.status\b|match.*CaseStatus|match.*case_row\.status\b|match.*case_row\.severity\b|match.*CaseSeverity' crates/api/api/src/governance/submit_jury_vote.rs
   ```
   For each match, read 14-20 lines of context; assert no `_ =>`
   arms in any new match site SL-d introduces.

**MIRROR:** §10.3 (existing `process_vote` outer-transaction shape
— SL-d preserves the single transaction); §10.4
(`sponsor_liability_pending` payload schema).

**GOTCHA (single transaction):** SL-d's mutation lives inside the
existing `run_transaction` at line 140. Does NOT add a new
`run_transaction`. Does NOT add a new ConfigCache (uses the
existing `&mut cache` at line 170). Per
`feedback_multi_write_handlers_need_transactions.md`.

**GOTCHA (Watchpoint #2 — severity snapshot):**
`grace_window_for_severity` takes `case_row.severity` (from
`moderation_case.severity` column, loaded at line 198). Re-deriving
severity from config or sanction at fire-time is a regression.

**GOTCHA (Watchpoint #11 — no new ENTRY_KIND_*):** Task 2 imports
existing `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` from the api shim.
NO new const declaration in `governance_log.rs` (canonical) OR the
shim. At task-end:
```bash
git diff governance-v0..HEAD -- crates/db_schema/src/source/governance/governance_log.rs crates/api/api/src/governance/governance_log.rs
```
Both diffs must be empty.

**GOTCHA (R7 — test target compile):** the workspace-check
workflow runs `cargo test --no-run -p lemmy_server --test e2e`. If
any existing e2e test imports the v0 `apply_sponsor_liability`
symbol (none expected), Task 1's wrapper preserves compile.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `feat(v1-SL-d): submit_jury_vote Decided→SponsorLiabilityPending transition + grace-window snapshot + deferred writes (task 2)`

### Task 3: e2e test #1 — Decided → SponsorLiabilityPending transition (open `mod v1_sl_d_fixtures` shell + helpers + Test #1)

**ACTION:** in `crates/server/tests/e2e.rs`, append a NEW
`mod v1_sl_d_fixtures` block AFTER `mod v1_sl_c_fixtures`. Inside
the new mod, add the use block + helpers + the first test fn
`submit_jury_vote_transitions_to_pending_for_liability_bearing_sponsored_case`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # open mod v1_sl_d_fixtures + helpers + test #1
```

**IMPLEMENT (file 1 of 1):** anchor-Edit at file end. Anchor
identification at task-start:
```bash
grep -n '^mod v1_' crates/server/tests/e2e.rs | tail -1
# EXPECT: mod v1_sl_c_fixtures
```
The new mod opens AFTER the closing `}` of `mod v1_sl_c_fixtures`
(near line 12,819).

**Content shape** (mirror `mod v1_sl_b_fixtures` Case A shape at
e2e.rs:11001-11924; helper signatures `LemmyResult<T>`, test fn
signatures `LemmyResult<()>`, bare `?` propagation, NO `Box<dyn
Error>` outer):

- Open `mod v1_sl_d_fixtures { use super::*; ... }` block.
- Add helper `seed_target_with_sureties(conn, sponsor_count)` per
  §10.5 skeleton, body mirrors `mod v1_sl_b_fixtures` helpers.
- Add first test fn
  `#[tokio::test] async fn submit_jury_vote_transitions_to_pending_for_liability_bearing_sponsored_case() -> LemmyResult<()>`:

  **Setup:** bootstrap LemmyContext with Postgres testcontainer.
  Seed sponsee + 2 sponsors with active sureties. Seed
  moderation_case in InReview status with target_person_id =
  sponsee, severity = CaseSeverity::Severe (or canonical mapping).
  Seat a jury panel (mirror existing JM-c/SL-b test setup).
  Pre-seed N-1 jury votes for SuspendCommunityMember (a
  liability-bearing sanction).

  **Drive:** invoke submit_jury_vote handler with the Nth (final)
  vote.

  **Assert:**
  - `response.case_decided == true`.
  - `response.decision == Some(JuryDecision::SuspendCommunityMember)`.
  - `moderation_case.status == CaseStatus::SponsorLiabilityPending`
    (NOT Decided).
  - `moderation_case.grace_expires_at` is `Some(t)` where
    `|t - (now + Duration::hours(<expected_for_severity>))| < 5
    seconds`. (Severe → 168h per PRD §10.)
  - 0 reputation_event rows for sponsors (compute is pure).
  - 0 reputation_event rows for jurors (deferred to scheduler).
  - 0 reputation_event rows for reporter (deferred to scheduler).
  - 0 public_case_log rows (deferred to scheduler).
  - 1 governance_log row entry_kind == "case_decided" (auditor
    visibility, PRD §11.4).
  - 1 governance_log row entry_kind == "sponsor_liability_pending"
    (DQ #179 LOCKED).
  - sponsor_liability_pending payload contains: case_id matches;
    target_pseudonym is a String; target_pseudonym does NOT equal
    `format!("{}", sponsee.0)` (defensive ADR-015 — Watchpoint
    #10); severity == "severe" (or matching string for chosen
    severity); grace_expires_at present; sponsors_pseudonyms is
    array of length 2; each entry is a String.
  - 1 governance_log row entry_kind == "sanction_created".
  - `moderation_case.appeal_window_expires_at` is `Some(t)` (PRD
    §11 + JM-c invariant — fires on BOTH paths).

**MIRROR:** §10.5 (fixture mod shape, Case A canonical sibling at
e2e.rs:11001-11924); §10.4 (sponsor_liability_pending payload
schema); §10.6 (governance_log row count assertion pattern).

**GOTCHA (R4):** test fn name lowercase snake_case.

**GOTCHA (Case A canonical sibling — `feedback_lemmy_error_no_std_error.md`):**
read `crates/server/tests/e2e.rs:11001-11924` (`mod
v1_sl_b_fixtures`) in full at task-start. Mirror the
`LemmyResult<T>` outer pattern verbatim. Helper signatures: `->
LemmyResult<T>`. Test fn signature: `-> LemmyResult<()>`. Bare `?`
propagation. NO `Box<dyn Error>` outer. NO `.map_err` bridges. The
3-cycle SL-c-2 catch-fire (cycles same `(E0277, e2e.rs)`) was
caused by stub-shape non-uniformity; SL-d avoids by following Case
A throughout per
`feedback_plan_stub_uniformity_with_canonical_sibling.md`.

**GOTCHA (anchor-Edit discipline per `feedback_junior_worker_e2e_edit_hang.md`):**
this Task 3 ships the `mod v1_sl_d_fixtures` shell + helpers + 1
test in ONE Edit at file end. Tasks 4-6 anchor-insert AFTER this
task's test body, INSIDE the same mod. Do NOT bulk-edit multiple
tests in one Edit. e2e.rs is **12,819 lines** post-SL-c-2.

**GOTCHA (Watchpoint #2 — grace_expires_at value assertion):** the
expected value depends on `case.severity` at seed time. If seeded
as `CaseSeverity::Severe`, expect `grace_expires_at - now ≈ 168
hours`. The exact severity mapping is defined by Task 1's
`liability_severity_from_case_severity`; Test #1 uses the same
mapping the handler uses. Assertion shape with 5s tolerance.

**GOTCHA (governance_log queries in tests):** use diesel
`governance_log::table.filter(...)` per existing JM-c
`governance_log_hash_chain_holds` test pattern at
`crates/server/tests/e2e.rs:907`.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-d): e2e test #1 — Decided → SponsorLiabilityPending transition + mod v1_sl_d_fixtures shell (task 3)`

### Task 4: e2e test #2 — No-sponsor path preserves v0 immediate-`Decided`

**ACTION:** anchor-insert
`submit_jury_vote_preserves_v0_decided_for_no_sponsor_target` test
fn inside `mod v1_sl_d_fixtures`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test #2 inside mod v1_sl_d_fixtures
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 3's test fn,
inside the same mod block.

`#[tokio::test] async fn submit_jury_vote_preserves_v0_decided_for_no_sponsor_target() -> LemmyResult<()>`:

**Setup:** bootstrap LemmyContext with Postgres testcontainer.
Seed target person with ZERO active sureties (no sponsors at all,
OR all pre-revoked before vote-tally — either case yields
`Vec<SponsorDelta>::is_empty() == true` at
`compute_sponsor_liability`). Seed moderation_case + jury panel as
in Test #1 with target_person_id = no-sponsor-target,
severity = CaseSeverity::Moderate. Pre-seed N-1 votes for
ContentRemoval (liability-bearing sanction).

**Drive:** submit_jury_vote handler with the Nth (final) vote.

**Assert:**
- `response.case_decided == true`.
- `response.decision == Some(JuryDecision::RemoveContent)`.
- `moderation_case.status == CaseStatus::Decided` (v0 path; NOT
  Pending).
- `moderation_case.grace_expires_at` IS NULL (no Pending
  transition; v0 branch sets None on the new UPDATE).
- 0 governance_log rows entry_kind == "sponsor_liability_pending"
  (no Pending transition).
- 1 governance_log row entry_kind == "case_decided" (auditor
  visibility).
- 1 governance_log row entry_kind == "sanction_created".
- 1 public_case_log row (v0 immediate write).
- juror reputation_event rows fire immediately (count ==
  panel_size).
- reporter reputation_event row fires immediately (count == 1 if
  case has creator_id).
- `moderation_case.appeal_window_expires_at` is `Some(t)`.

**MIRROR:** §10.5 (fixture mod, Case A canonical sibling); brief
§2.1.e Test #2; Watchpoint #3.

**GOTCHA (Watchpoint #3 — v0 path semantics):** the no-sponsor
path runs the v0 immediate-Decided code path.
`compute_sponsor_liability` returns `vec![]`; the handler's branch
logic sets `path_kind = Decided`; steps 10-12 fire on Decided path.
Test #2 is the strong assertion that the v0 semantics are
preserved.

**GOTCHA (zero-surety seed):** simplest pattern is to skip
`seed_target_with_sureties` entirely — create the target Person
but no surety rows. Alternative: seed N sureties then UPDATE all
`revoked_at = now() - 1h` before vote-tally
(`compute_sponsor_liability` filters `revoked_at IS NULL` per
`sponsor_liability.rs:161`).

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-d): e2e test #2 — no-sponsor path preserves v0 immediate-Decided (task 4)`

### Task 5: e2e test #3 — NoAction path skips liability machinery entirely

**ACTION:** anchor-insert
`submit_jury_vote_no_action_skips_liability_machinery` test fn
inside `mod v1_sl_d_fixtures`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test #3 inside mod v1_sl_d_fixtures
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 4's test fn.

`#[tokio::test] async fn submit_jury_vote_no_action_skips_liability_machinery() -> LemmyResult<()>`:

**Setup:** bootstrap LemmyContext with Postgres testcontainer.
Seed sponsee + 2 sponsors with active sureties (mirror Test #1).
Seed moderation_case + jury panel. Pre-seed N-1 votes for
JuryDecision::NoAction.

**Drive:** submit_jury_vote handler with the Nth (final) NoAction
vote.

**Assert:**
- `response.case_decided == true`.
- `response.decision == Some(JuryDecision::NoAction)`.
- `moderation_case.status == CaseStatus::Decided` (NoAction = no
  liability; `map_decision_to_sanction` returns None at
  submit_jury_vote.rs:423; the entire `if let Some((scope, action))
  = ...` block at lines 423-481 is skipped, so
  `compute_sponsor_liability` is never called).
- `moderation_case.grace_expires_at` IS NULL (no Pending transition).
- 0 governance_log rows entry_kind == "sponsor_liability_pending".
- 0 governance_log rows entry_kind == "sanction_created" (NoAction
  writes no sanction row per the table at submit_jury_vote.rs:26-35).
- 0 reputation_event rows for sponsors.
- 1 governance_log row entry_kind == "case_decided" (still emitted
  on NoAction path per JM-c shipped behaviour).
- juror + reporter reputation_event rows fire (NoAction is the v0
  path; not deferred).

**MIRROR:** §10.5 (fixture mod, Case A canonical sibling); brief
§2.1.e Test #3.

**GOTCHA (NoAction short-circuit):** at
`submit_jury_vote.rs:423`, `if let Some((scope, action)) =
map_decision_to_sanction(winning_decision)` is the gate. NoAction
→ `map_decision_to_sanction` returns None → the block is skipped.
`compute_sponsor_liability` never runs. SL-d's mutation does NOT
change this gate. Test #3 asserts.

**GOTCHA (case_decided log on NoAction):** the `case_decided` log
at `submit_jury_vote.rs:649-659` fires unconditionally
(path-agnostic). NoAction path still emits it.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-d): e2e test #3 — NoAction path skips liability machinery (task 5)`

### Task 6: e2e test #4 — Wrapper preserves v0 outputs (compute + fire composition is byte-identical to v0 wrapper)

**ACTION:** anchor-insert
`apply_sponsor_liability_wrapper_preserves_v0_outputs` test fn
inside `mod v1_sl_d_fixtures`. This is the LAST test in `mod
v1_sl_d_fixtures`; the closing `}` of the mod follows.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test #4 inside mod v1_sl_d_fixtures (closes the mod)
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 5's test fn.

`#[tokio::test] async fn apply_sponsor_liability_wrapper_preserves_v0_outputs() -> LemmyResult<()>`:

Tests the wrapper invariant: `apply_sponsor_liability(...)` (the
thin wrapper after Task 1's split) produces the same
reputation_event rows + sponsor_liability_applied / _CLAMPED log
entries as the v0 single-pass body would have produced for the
same inputs.

**Setup:** bootstrap LemmyContext with Postgres testcontainer.
Seed sponsee + 2 sponsors with active sureties. Seed
moderation_case in CaseStatus::SponsorLiabilityPending with
grace_expires_at = now() - 1 minute (expired). Seed sanction.

**Drive:** invoke `run_grace_check_batch(&context).await` — fires
the wrapper internally at `sponsor_liability_grace.rs:510`.

**Assert:**
- `case.status == CaseStatus::SponsorLiabilityFired` (SL-c set this
  after wrapper returned successfully).
- 2 reputation_event rows for sponsor_id ASC order; each row's
  dimension = EndorsementStrength; reason =
  "sponsor_liability_applied"; source_case_id = Some(case_id);
  delta = expected per-sponsor value from config (compute moderate
  severity bucket, founder-multiplier-none, no clamp engaged).
- 2 governance_log rows entry_kind == "sponsor_liability_applied";
  each payload has sponsor_pseudonym (string), severity =
  "moderate", pre_multiplier_delta, post_multiplier_delta,
  final_delta, multiplier fields per v0 payload at
  sponsor_liability.rs:322-337.
- 0 governance_log rows entry_kind == "sponsor_liability_clamped"
  (no clamp engaged in this test's setup).
- 1 governance_log row entry_kind == "sponsor_liability_fired"
  (SL-c summary).
- All 2 _APPLIED entries have payload fields BYTE-IDENTICAL to
  what a single-pass v0 body would have produced (verified by
  re-computing expected fields from seed fixtures and comparing).

**Strong assertion:** the wrapper's compute + fire composition
produces the same observable side effects as the v0 single-pass
body. Detected regression: any drift between compute's struct and
fire's payload-rebuild surfaces here.

**MIRROR:** §10.4 (governance_log payload schemas — the
sponsor_liability_applied / _clamped shapes); v0 body at
`sponsor_liability.rs:322-353` for expected payload field set.

**GOTCHA (wrapper visibility):** `apply_sponsor_liability` is
`pub(crate)`. Tests in `crates/server/tests/e2e.rs` are in a
different crate; they cannot call `pub(crate)` items directly.
**Default lean: drive via `run_grace_check_batch`** —
`lemmy_api::governance::sponsor_liability_grace::run_grace_check_batch`
is `pub` and SL-c-2 tests at e2e.rs:11932 already use this
pattern; precedent established. The scheduler's Fire branch at
`sponsor_liability_grace.rs:510` exercises the wrapper.

**GOTCHA (config-driven expected delta computation):** the test
re-computes the expected delta from config seeds (raw_delta /
sponsor_count + remainder + multiplier + clamp). Helper
`compute_expected_delta(severity, sponsor_count, founder_count,
...)` may be useful — define inline or in the fixture mod's
helpers.

**GOTCHA (closing the mod):** this test is the LAST inside `mod
v1_sl_d_fixtures`. The closing `}` of the mod follows immediately
after this test's `Ok(())` line. Future SL-e fixture mods open
AFTER `mod v1_sl_d_fixtures` closes.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-d): e2e test #4 — wrapper preserves v0 outputs (compose-equivalence) (task 6)`

### Task 7: Unit tests #5 + #6 — compute idempotency + grace-window severity mapping (`sponsor_liability.rs::tests`)

**ACTION:** in
`crates/api/api/src/governance/sponsor_liability.rs`, add
`#[cfg(test)] mod tests { ... }` block at end-of-file with 2 unit
tests:

1. `compute_sponsor_liability_idempotent` — calls
   `compute_sponsor_liability(...)` twice with identical inputs;
   asserts returned `Vec<SponsorDelta>` is byte-equal both times;
   asserts no DB rows written by either call.
2. `grace_window_for_severity_reads_correct_config_key_per_tier` —
   for each severity tier (Minor / Moderate / Severe), invoke
   `grace_window_for_severity` and assert the duration matches the
   SL-a-seeded default (24h / 72h / 168h).

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/sponsor_liability.rs   # append #[cfg(test)] mod tests block
```

**IMPLEMENT (file 1 of 1):** append at end-of-file (after the
closing `}` of `apply_sponsor_liability`):

- `#[cfg(test)] mod tests { use super::*; ... }` block.
- Two `#[tokio::test] async fn ... -> LemmyResult<()>` tests per
  §10.7 skeleton.
- Both use Postgres testcontainer setup (mirror sibling pattern at
  task-start).
- Both use `LemmyResult<()>` outer per Case A discipline.

**MIRROR:** §10.7 (unit test pattern); §10.4 (config-key
namespaces).

**GOTCHA (Watchpoint #7 — idempotent compute):** the assertion
`assert_eq!(deltas1, deltas2)` requires `SponsorDelta: PartialEq`
(derived in Task 1).

**GOTCHA (CaseSeverity variant set):** the variants used in the
test loop must match `crates/db_schema_file/src/enums.rs`
`CaseSeverity` declaration. Read at task-start.

**GOTCHA (testcontainers in `crates/api/api/`):** if `crates/api/api`
doesn't already pull in testcontainers, the unit tests will fail
to compile. Read existing `#[cfg(test)] mod tests` blocks under
`crates/api/api/src/governance/` at task-start. If no in-crate
test infrastructure, **file planner DQ** asking whether to
relocate to `crates/server/tests/e2e.rs`. Default lean (DQ #180
LOCKED): keep here.

**GOTCHA (R7 — test target compile):** the workspace-check
workflow runs `cargo test --no-run -p lemmy_server --test e2e`.
Task 7's unit tests live in
`crates/api/api/src/governance/sponsor_liability.rs`, not
`crates/server/tests/e2e.rs`. The workflow's per-crate check at
`cargo check --workspace --features full` validates compile; the
`-p lemmy_server --test e2e` step does NOT compile
`crates/api/api/src/governance/*::tests`. Phase 2 e2e runs the e2e
suite, not crates' own unit tests.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-d): unit tests for compute_sponsor_liability idempotency + grace_window_for_severity mapping (task 7)`

### Task 8: Retro

**Goal:** author retro per `feedback_retro_not_report.md` +
`feedback_four_role_retro_signals.md` +
`feedback_retro_task_complexity_score.md`. Flip
`(pending) → (active)` marker on registry §"v1-SL-a entry kinds"
for `_SPONSOR_LIABILITY_PENDING` (the SL-d call site row).

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-SL-d-retro.md
modifies:
  - .claude/rules/governance-log-entry-kind-registry.md
```

**IMPLEMENT (file 1 of 2):** in
`.claude/PRPs/reports/v1-SL-d-retro.md`, write the retro per the
canonical 4-role format (Advisor / Planning / Impl / BM):

- §1 Summary: SL-d shipped. Stories 1 + 2 + 3 ✓. Closes the SL
  producer side; SL-e ships the lane-wide e2e next.
- §2 Per-role signals (4 H2 sections; "what worked" + "what
  surprised us" + "what should change next"). Specifically:
  cross-PR carry-forward signals from SL-c-2 → SL-d (e.g. did the
  Case A canonical sibling discipline avoid the 3-cycle catch-fire?
  did the Probe 8 JM-c TODO marker check fire?).
- §3 Carry-forward — list of items SL-e/restorative-mechanics-v1
  should know.
- §4 Per-task complexity-score table (mandatory per
  `feedback_retro_task_complexity_score.md`):
  `| task | files-changed | commits | runtime-min | max-log-silence-min |`
  for each of Tasks 0..8.
- §5 Lessons promotion — any new `feedback_*.md` candidates
  (especially: cross-PR registry-flip discipline,
  wrapper-preserves-signature pattern as a refactor-only
  invariant, deferred-write semantics testing pattern).
- §6 Acceptance — confirm all checkboxes from §17.

**IMPLEMENT (file 2 of 2):** in
`.claude/rules/governance-log-entry-kind-registry.md`, locate the
"v1-SL-a entry kinds" section. Edit the "Emitting handler" column
for the `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` row: change "v1-SL-d
... (pending)" to "(active)" — flipped post-SL-d ship.

Total const count stays unchanged.

**Cross-cutting verification (Task 8 retro time — invariants):**
`rg` checks per §15.4.

**MIRROR:** `.claude/PRPs/reports/v1-SL-c-2-retro.md` (sibling);
`.claude/PRPs/reports/v1-SL-b-retro.md`;
`.claude/PRPs/reports/v1-JM-e-retro.md` for section structure.

**GOTCHA (retro before PR):** retro is written BEFORE `gh pr
create` per `feedback_retro_not_report.md`.

**GOTCHA:** §4 per-task complexity-score table is mandatory.

**Push and exit (Shape G — retro is meta-work; no `crates/**`
change → `cargo-validate-workspace` does not trigger).**

**COMMIT MESSAGE:** `docs(v1-SL-d): phase retrospective + flip ENTRY_KIND_SPONSOR_LIABILITY_PENDING registry marker (task 8)`

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** `cargo check --workspace --features
  full` via `cargo-validate-workspace.yml:88`.
- **Lint:** `cargo clippy --workspace --features full --no-deps
  -- -D warnings` via `cargo-validate-workspace.yml:92` (R6).
- **Test target compile (R7):** `cargo test --no-run -p
  lemmy_server --test e2e` via `cargo-validate-workspace.yml:95`.
- **Migration round-trip:** N/A — SL-d ships zero migrations.
- **e2e execution (Phase 2):** the 4 new e2e tests run via Phase 2
  e2e user gate: (a) local on laptop or (b) GH dispatch via
  `cargo-test-e2e.yml`. **This is the SL-d acceptance gate.**
- **Unit tests (Task 7):** the 2 unit tests in
  `sponsor_liability.rs::tests` run via `cargo test --workspace
  --features full`.

### 14.1 Pre-existing tests preserved

All SL-c-2 / SL-b / SL-a / JM-e / JM-c / JM-b / JM-a tests
preserved verbatim. Pre-existing
`submit_jury_vote_concurrent_votes_decide_exactly_once` regression
test (PR #98 cr-2) preserved — SL-d's mutation does NOT alter the
FOR UPDATE / lock-ordering invariant the test asserts.

### 14.2 Edge cases covered

- **Pending transition:** liability-bearing sanction + active
  sureties → status flips to SponsorLiabilityPending;
  `grace_expires_at` snapshotted from `case.severity` mapping;
  `sponsor_liability_pending` log emitted; deferred writes (Test
  #1).
- **No-sponsor path:** liability-bearing sanction + zero active
  sureties → status flips to Decided (v0 path); juror + reporter
  reputation events fire immediately (Test #2).
- **NoAction path:** `winning_decision == NoAction` →
  `map_decision_to_sanction` returns None; entire sponsor-liability
  block skipped (Test #3).
- **Wrapper composition:** `apply_sponsor_liability` =
  `compute + fire` produces byte-identical reputation_event rows +
  governance_log entries to the v0 single-pass body (Test #4 — via
  SL-c scheduler driving the wrapper).
- **Compute idempotency:** calling `compute_sponsor_liability`
  twice with identical inputs returns identical
  `Vec<SponsorDelta>`; no DB writes by either call (Test #5 unit).
- **Grace-window mapping:** each `CaseSeverity` tier maps to the
  correct config key + duration default (Test #6 unit).

### 14.3 Edge cases NOT covered (out of v1-SL-d scope)

- Concurrent SL-d + SL-c-scheduler on same case — defended by FOR
  UPDATE; no e2e test for the race itself.
- Restoration-completed escape branch — restorative-mechanics-v1
  PRD's deliverable.
- Lane-wide flow (Decided → Pending → Fired/Escaped) — SL-e's.
- Federation outbound on Pending transition — out per ADR-014.
- Modlog reader semantics during Pending state — covered by PRD
  §11.4 documentation; no client-side test.

---

## 15. Validation commands (DoD)

> **Shape G plan — DoD is per-workflow, not inline cargo.** Per
> `.claude/PRPs/templates/plan.template.md` §15.6. SL-d ships zero
> migrations, so `cargo-validate-migration.yml` does not fire.

### 15.1 Per-task workspace check (Shape G)

For every §13 impl task that modifies `crates/**` (Tasks 1, 2, 3,
4, 5, 6, 7):

- **DoD entry:** `cargo-validate-workspace.yml` on
  `junior/<task-slug>` SHA `<sha>` → `conclusion: "success"`
- **Validation command:** `gh run list --repo barrie-cork/lemmy
  --branch <branch> --workflow cargo-validate-workspace --limit 1
  --json conclusion,databaseId --jq '.[0]'`
- **EXPECT:** `{"conclusion": "success", "databaseId": <id>}`

The workflow runs `cargo check --workspace --features full`,
`cargo clippy --workspace --features full --no-deps -- -D
warnings`, and `cargo test --no-run -p lemmy_server --test e2e`
per `.github/workflows/cargo-validate-workspace.yml:88-95`. R6 +
R7 are encoded.

### 15.2 Migration round-trip — N/A

SL-d ships zero migrations; `cargo-validate-migration.yml` path
filter `migrations/**` excludes SL-d commits. No DoD entry.

### 15.3 Phase 2 e2e (post-finalize-merge of last impl task)

After Task 7's worker branch finalize-merges into `phase-v1-SL-d`,
the advisor surfaces the **Phase 2 e2e local-vs-dispatch user
gate** per `advisor-orchestrator.md` (PR #105):

- **(a) local:** `cmd //c "scripts\\brehon\\cargo-test.bat
  --workspace --test e2e --features full > <log> 2>&1 && echo
  E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` on
  laptop in `run_in_background`; ~26 min wall-clock; zero billed.
  Per `feedback_windows_e2e_requires_bat_wrapper.md`.
- **(b) dispatch:** `gh workflow run cargo-test-e2e.yml --repo
  barrie-cork/lemmy --ref phase-v1-SL-d`; ci-watcher polls; ~26
  min billed.

Plan-side DoD: e2e exit code 0 with all 4 SL-d e2e tests passing
+ all pre-existing tests still passing; failure path → §G4
classifier on log slice.

### 15.4 Cross-cutting verification (Task 8 retro time)

- [ ] `rg -c '^pub const ENTRY_KIND_'
  crates/db_schema/src/source/governance/governance_log.rs`
  returns the same total as post-SL-c-2 (unchanged).
- [ ] `rg '^\s+ENTRY_KIND_'
  crates/api/api/src/governance/governance_log.rs` line count
  unchanged (no new shim re-exports).
- [ ] `rg -n '\(pending\)'
  .claude/rules/governance-log-entry-kind-registry.md | rg
  SPONSOR_LIABILITY_PENDING` returns empty (marker flipped).
- [ ] `rg -n 'TODO\(v1-sponsor-liability-d\)'
  crates/api/api/src/governance/submit_jury_vote.rs` returns 0
  (marker discharged).
- [ ] `rg -n 'pub\(crate\) async fn compute_sponsor_liability|pub\(crate\) async fn fire_sponsor_liability|pub\(crate\) async fn apply_sponsor_liability|pub\(crate\) async fn grace_window_for_severity'
  crates/api/api/src/governance/sponsor_liability.rs` returns 4
  lines.
- [ ] `rg -n 'mod v1_sl_d_fixtures' crates/server/tests/e2e.rs`
  returns 1 line.
- [ ] R1: every `i32 ↔ i64` comparison in Tasks 1-7 uses
  `i64::from(...)` if cross-type comparison arises.
- [ ] R6: clippy invocations in `cargo-validate-workspace.yml`
  use `--no-deps -- -D warnings`.
- [ ] No edits to files outside §11 list.
- [ ] Every SL-d §16a story (1, 2, 3) is `[done]`.
- [ ] No new ENTRY_KIND_*** consts under
  `crates/db_schema/src/source/governance/governance_log.rs`.
- [ ] No new migrations: `git diff
  governance-v0..phase-v1-SL-d -- migrations/` returns empty.
- [ ] No SL-c module modifications: `git diff
  governance-v0..phase-v1-SL-d --
  crates/api/api/src/governance/sponsor_liability_grace.rs`
  returns empty.
- [ ] No scheduled_tasks.rs modifications: `git diff
  governance-v0..phase-v1-SL-d -- crates/routes/` returns empty.

### 15.5 ADR / OQ compliance verification

- [ ] **ADR-005** honoured — fire branch's per-sponsor
  `reputation_event` rows preserved by wrapper (Test #4).
- [ ] **ADR-008** honoured — every emit goes through
  `governance_log::append`; no direct INSERT.
- [ ] **ADR-010** honoured — won't-disadvantage rule preserved;
  severity is snapshotted on the case row (Watchpoint #2 + Test
  #1).
- [ ] **ADR-013** honoured — every new `match` site SL-d introduces
  in `submit_jury_vote.rs` is exhaustive (Watchpoint #8 grep step
  at Task 2 task-end). No `_ =>` arms in new sites.
- [ ] **ADR-014** honoured — no federation outbound on
  `sponsor_liability_pending` (Watchpoint via Task 2 step 6 guard).
- [ ] **ADR-015** honoured — `sponsor_liability_pending` log
  payload carries `target_pseudonym` + `sponsors_pseudonyms`
  strings; no raw `*_id.0` values (Test #1 defensive assertion).
- [ ] **PRD §9.1** honoured — `apply_sponsor_liability` split into
  compute + fire + wrapper; wrapper signature byte-identical to v0
  (Watchpoint #4 + Task 1 git diff at task-end).
- [ ] **PRD §9.3** honoured — 3 SL-d contributions implemented:
  Pending transition (Test #1); grace-window snapshot (Test #1
  grace_expires_at assertion); deferred write set (Test #1
  zero-row assertions).
- [ ] **PRD §11.3** honoured — `submit_jury_vote` DTO + response
  unchanged (Tests #1-#4 invoke through the same DTO).
- [ ] **PRD §11.4** honoured — auditor visibility preserved;
  `case_decided` log fires on BOTH paths (Tests #1 + #2 assert).
- [ ] **PRD §2 OUT** honoured — no notification UX (DQ #176), no
  computed-deltas snapshot column (DQ #177), no community-cascaded
  grace_window (DQ #178), no restoration-complete endpoint, no
  cross-instance federation, no step-up auth.
- [ ] **PRD §12.3 v2 reservation** honoured — no step-up auth on
  the producer.
- [ ] **DQ #176** honoured — `notify_sponsor_of_pending_liability`
  does NOT appear in plan §13 or impl code.
- [ ] **DQ #177** honoured — no `computed_deltas` column; no new
  migration.
- [ ] **DQ #178** honoured — `grace_window_for_severity` reads
  `Scope::Instance` only (Watchpoint via Task 1 IMPLEMENT).
- [ ] **DQ #179** honoured — `sponsor_liability_pending` log
  emitted at the transition (Watchpoint #9 + Task 2 IMPLEMENT).
- [ ] **DQ #180** honoured — Tasks 3-6 are 4 e2e tests; Task 7 is
  the unit-test task (in `sponsor_liability.rs::tests`, not
  `e2e.rs`).

### 15.6 DoD per workflow (canonical Shape G shape)

**Phase 1 (workspace check, per task):**

- Workflow: `.github/workflows/cargo-validate-workspace.yml`
- Branch (per task): `junior/<task-slug>`
- Expected `conclusion`: `"success"`

**Phase 1b (migration round-trip):** N/A — no migrations in SL-d.

**Phase 2 (e2e):**

- Workflow: `.github/workflows/cargo-test-e2e.yml` (only on user
  dispatch per PR #105)
- OR local: per advisor-orchestrator §5.2
  validate-pending-laptop-e2e — Windows bat wrapper with the
  `--workspace --test e2e --features full` invocation
- Branch: `phase-v1-SL-d`
- Expected: all tests pass

### 15.7 Manual validation snippets (advisor-side smoke only — non-binding)

> Per `feedback_features_full_p_crate_incompatible.md`: never `-p
> <crate>` + `--features full`; use `--workspace --features full`.

> Per `feedback_e2e_filter_assumes_naming.md`: SL-d e2e tests all
> live in `mod v1_sl_d_fixtures` — confirm via grep before any
> filtered run.

> Per `cargo-output-capture.md`: capture cargo output to a file;
> never pipe through tail/head/grep.

```bash
ls .github/workflows/cargo-validate-workspace.yml
ls .github/workflows/cargo-test-e2e.yml

gh run list --repo barrie-cork/lemmy \
  --branch phase-v1-SL-d \
  --workflow cargo-validate-workspace \
  --limit 1 --json conclusion,databaseId

cargo check --workspace --features full > /tmp/sl-d-check.log 2>&1
status=$?
tail -20 /tmp/sl-d-check.log
echo "exit: $status"

rg '#\[tokio::test\]\s*async fn .*' crates/server/tests/e2e.rs > /tmp/sl-d-tests.log
wc -l /tmp/sl-d-tests.log
# EXPECT: pre-merge baseline + 4 (SL-d Tests #1-#4)

rg -n 'pub\(crate\) async fn compute_sponsor_liability' crates/api/api/src/governance/sponsor_liability.rs
# EXPECT: 1 match (Task 1 split landed)

rg -n 'pub\(crate\) async fn grace_window_for_severity' crates/api/api/src/governance/sponsor_liability.rs
# EXPECT: 1 match (Task 1 helper landed)
```

These are advisor-side only; not §16 acceptance criteria.

---

## 16. Acceptance criteria

- [ ] All 9 tasks (Task 0..7 + Task 8 retro) committed.
- [ ] §15.1 (`cargo-validate-workspace.yml`) `conclusion:
  "success"` after every impl task push (Tasks 1-7).
- [ ] §15.2 (migration round-trip) — N/A (zero migrations).
- [ ] §15.3 (Phase 2 e2e — local or dispatch) all tests pass; the
  4 new `v1_sl_d_fixtures::*` tests green; the 2 unit tests in
  `sponsor_liability.rs::tests` green.
- [ ] §15.4 (cross-cutting verification — 14 boxes) all ticked.
- [ ] §15.5 (ADR / OQ compliance — 16 boxes) all ticked.
- [ ] §16a Stories 1 + 2 + 3 — all `[done]`.
- [ ] No edits to files outside §11 list (notably: zero SL-c
  module modifications, zero migrations).
- [ ] Retro committed per Task 8.
- [ ] PR opens against `governance-v0` (NOT `main`) with
  `--repo barrie-cork/lemmy`.
- [ ] `/brehon-verify` report at
  `.claude/PRPs/reports/v1-SL-d-verify.md` shows Stories 1-3 ✓.
- [ ] DQ #186 (split-or-proceed) self-resolved by planner with
  proceed rationale at plan-write time (Recipe 2 self-resolved
  per `decision-queue.md`).
- [ ] Registry marker flipped (`_SPONSOR_LIABILITY_PENDING` row's
  Emitting handler "(pending)" → "(active)" at Task 8 retro).
- [ ] JM-c TODO marker at `submit_jury_vote.rs:458` discharged.

---

## 16a. Stories (independently-testable behaviour units)

Per `.claude/PRPs/templates/plan.template.md` §16a +
`feedback_story_grain_checkpoint.md` +
`feedback_brehon_verify_pre_merge.md`. **SL-d ships THREE stories.**

### Story 1: Compute/fire split lands; wrapper preserves v0 signature; idempotent compute

- **Composing tasks:** Task 1, Task 7 (the unit-test pair lives
  inside `sponsor_liability.rs::tests` and validates Story 1's
  invariants behaviourally).
- **Checkpoint workflow (Phase 1):**
  `cargo-validate-workspace.yml` on Task 1's worker branch SHA →
  `conclusion: "success"`. Task 7's worker branch SHA →
  `conclusion: "success"`.
- **Checkpoint workflow (Phase 2):** Phase 2 e2e on phase-branch
  tip; `compute_sponsor_liability_idempotent` and
  `grace_window_for_severity_reads_correct_config_key_per_tier` in
  `sponsor_liability.rs::tests` pass.
- **Expected output (local Phase 2 — unit tests):** `2 passed; 0
  failed` for the 2 unit tests.
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/governance/sponsor_liability.rs` contains
    `pub(crate) async fn compute_sponsor_liability(`.
  - `crates/api/api/src/governance/sponsor_liability.rs` contains
    `pub(crate) async fn fire_sponsor_liability(`.
  - `crates/api/api/src/governance/sponsor_liability.rs` contains
    `pub(crate) async fn apply_sponsor_liability(` (wrapper,
    signature byte-identical to v0).
  - `crates/api/api/src/governance/sponsor_liability.rs` contains
    `pub(crate) async fn grace_window_for_severity(`.
  - `crates/api/api/src/governance/sponsor_liability.rs` contains
    `pub(crate) struct SponsorDelta {`.
  - `crates/api/api/src/governance/sponsor_liability.rs` contains
    `#[cfg(test)] mod tests {` with 2 `#[tokio::test]` test fns.
  - `crates/api/api/src/governance/sponsor_liability_grace.rs`
    diff vs governance-v0 is empty (SL-c call site preserved).

### Story 2: `submit_jury_vote` mutation transitions liability-bearing+sponsored cases to `SponsorLiabilityPending` with correct `grace_expires_at` snapshot; preserves `Decided` on no-sponsor + NoAction paths

- **Composing tasks:** Task 2, Task 3, Task 4, Task 5
- **Checkpoint workflow (Phase 1):**
  `cargo-validate-workspace.yml` on Task 2's, Task 3's, Task 4's,
  Task 5's worker branch SHAs → `conclusion: "success"`.
- **Checkpoint workflow (Phase 2):** Phase 2 e2e for the 3
  mutation tests (#1, #2, #3) → all pass.
- **Expected output (local Phase 2):** `3 passed; 0 failed`.
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/governance/submit_jury_vote.rs` contains
    `sponsor_liability::compute_sponsor_liability(` (Task 2's
    replacement of v0 call).
  - `crates/api/api/src/governance/submit_jury_vote.rs` contains
    `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` reference in the
    `governance_log::append` call site (Task 2's emit).
  - `crates/api/api/src/governance/submit_jury_vote.rs` does NOT
    contain `TODO(v1-sponsor-liability-d)` (Task 2 removed).
  - `crates/server/tests/e2e.rs` contains `mod v1_sl_d_fixtures`
    block.
  - `crates/server/tests/e2e.rs` contains `async fn
    submit_jury_vote_transitions_to_pending_for_liability_bearing_sponsored_case(`.
  - `crates/server/tests/e2e.rs` contains `async fn
    submit_jury_vote_preserves_v0_decided_for_no_sponsor_target(`.
  - `crates/server/tests/e2e.rs` contains `async fn
    submit_jury_vote_no_action_skips_liability_machinery(`.
  - Test #1 body asserts `case.status ==
    CaseStatus::SponsorLiabilityPending` AND
    `case.grace_expires_at` matches `now + Duration::hours(<expected>)`
    within 5s tolerance AND `count(sponsor_liability_pending) == 1`
    AND defensive ADR-015 `target_pseudonym` assertion.
  - Test #2 body asserts `case.status == CaseStatus::Decided` AND
    `count(sponsor_liability_pending) == 0` AND juror reputation
    events fire.
  - Test #3 body asserts `case.status == CaseStatus::Decided` AND
    `count(sponsor_liability_pending) == 0` AND
    `count(sanction_created) == 0` (NoAction writes no sanction).

### Story 3: `grace_window_for_severity` reads correct config key per severity tier

- **Composing tasks:** Task 1 (helper definition), Task 7 (unit
  test validation), Task 6 (e2e wrapper-composition test as
  defence-in-depth).
- **Checkpoint workflow (Phase 1):**
  `cargo-validate-workspace.yml` on Task 1's worker branch SHA →
  `conclusion: "success"`. Task 6's + Task 7's worker branch SHAs
  → `conclusion: "success"`.
- **Checkpoint workflow (Phase 2):** Phase 2 e2e for Test #4
  (wrapper composition);
  `grace_window_for_severity_reads_correct_config_key_per_tier`
  unit test in `sponsor_liability.rs::tests`.
- **Expected output (local Phase 2):** `1 passed; 0 failed` for
  e2e Test #4; `1 passed; 0 failed` for the unit test.
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/governance/sponsor_liability.rs` contains
    `pub(crate) async fn grace_window_for_severity(severity:
    CaseSeverity, cache: &mut ConfigCache, conn: &mut
    AsyncPgConnection) -> LemmyResult<chrono::Duration>`.
  - The function body references each of
    `liability.grace_window_minor_hours`,
    `liability.grace_window_moderate_hours`,
    `liability.grace_window_severe_hours`.
  - The function body uses `Scope::Instance` (per DQ #178 LOCKED).
  - `crates/server/tests/e2e.rs` contains `async fn
    apply_sponsor_liability_wrapper_preserves_v0_outputs(`.
  - The unit test
    `grace_window_for_severity_reads_correct_config_key_per_tier`
    iterates 3 severity tiers and asserts duration values 24h /
    72h / 168h.

> **Verification mapping:** `/brehon-verify` iterates this section,
> runs each Story's checkpoint workflow on the worktree branch,
> and confirms each Brief-Scope output exists + matches its
> structural pattern.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0..18 confirmed; Probes 7
  (SL-c module) + 8 (JM-c TODO marker) are critical).
- [ ] Tasks 1..7 committed.
- [ ] Task 8 retro committed.
- [ ] §15 validation green at every gate (Phase 1 workspace check
  ×7, Phase 2 e2e ×1).
- [ ] §16a Stories 1, 2, 3 all `[done]`.
- [ ] PR opened by BM session against `governance-v0`.
- [ ] CodeRabbit review complete with findings triaged per
  `feedback_pr_review_triage_pattern.md`.
- [ ] `/brehon-verify` report at
  `.claude/PRPs/reports/v1-SL-d-verify.md` shows Stories 1-3 ✓.
- [ ] Post-merge phase branch retained for retro reads.
- [ ] DQ #186 (split-or-proceed) self-resolved by planner with
  proceed rationale at plan-write time.
- [ ] Registry marker flipped (`_SPONSOR_LIABILITY_PENDING`).
- [ ] JM-c TODO marker at `submit_jury_vote.rs:458` discharged.

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Complexity score 16 prompts advisor to mandate further split | LOW | LOW | §5.2 — planner self-resolved DQ #186 with proceed rationale; further splitting yields no meaningful score reduction (e2e factor dominates +12 of total). SL-c-2 (17), JM-e (15), SL-b (38) precedents proceed-as-one |
| Junior worker-hang on e2e.rs Edits (file is 12,819 lines, growing) | MED | HIGH | Per `feedback_junior_worker_e2e_edit_hang.md`: each test is its own §13 task; per-task anchor-Edit appends inside `mod v1_sl_d_fixtures`; no bulk Edit. SL-c-2 shipped 5 e2e tests as 5 tasks cleanly at this file size |
| 3-cycle catch-fire on stub-shape non-uniformity (per SL-c-2 cycles 1-3) | MED | HIGH | §13 Tasks 3-6 stubs use `LemmyResult<()>` outer uniformly with v1-SL-b `mod v1_sl_b_fixtures` Case A canonical sibling at e2e.rs:11001-11924. `feedback_lemmy_error_no_std_error.md` Case A + `feedback_plan_stub_uniformity_with_canonical_sibling.md` cited in §9.4 + Tasks 3-6 GOTCHAs. §G4 row 4c (Case C — mixed shapes) is hard refusal; will not auto-fix |
| Wrapper signature drift between v0 + Task 1 split | LOW | HIGH | Watchpoint #4 — Task 1 task-end `git diff` on the wrapper line; if not byte-identical, STOP. SL-c's call site at `sponsor_liability_grace.rs:510` would break on signature drift |
| Compute purity violation (write leaks into compute body) | LOW | HIGH | Watchpoint #5 — Task 1 task-end grep for `insert_into\|update\|append\|persist` inside `compute_sponsor_liability`'s body; zero matches required. Test #4 (wrapper composition) catches behaviourally |
| ADR-013 enum-exhaustiveness violation in new `submit_jury_vote.rs` match site | LOW | HIGH | Watchpoint #8 — Task 2 task-end grep for `match.*case.status\|match.*CaseStatus` returns no `_ =>` arms in any new site SL-d introduces |
| Pseudonym-discipline violation (raw `*_id.0` leaks into payload) | LOW | HIGH | Watchpoint #10 — Test #1 defensive `assert_ne!(json["target_pseudonym"].as_str().unwrap(), &format!("{}", target_id.0))`. ADR-015 enforced |
| Severity snapshot regression (re-derive from config at fire-time instead of `case.severity`) | LOW | HIGH | Watchpoint #2 — Task 2 IMPLEMENT specifies `case_row.severity` (not re-derive). Test #1 asserts grace_expires_at matches expected value |
| Deferred-write set leaks (public_case_log or juror reputation events fire on Pending path) | LOW | MED | Test #1 asserts `count == 0` for these on Pending path. Task 2 step 5 guards the steps 10-12 block on path_kind == Decided |
| `case_decided` log accidentally guarded on path_kind == Decided (regression — auditor visibility lost) | LOW | HIGH | Task 2 IMPLEMENT step 7 explicitly preserves step 9 on BOTH paths. Test #1 asserts count("case_decided") == 1 on Pending path |
| `appeal_window_expires_at` UPDATE accidentally guarded on Decided (regression — JM-c invariant broken) | LOW | MED | Task 2 IMPLEMENT step 7 explicitly preserves step 9 on BOTH paths. Tests #1 + #2 assert `appeal_window_expires_at` is Some(t) on both paths |
| `grace_expires_at` value drift (wrong severity bucket → wrong duration) | LOW | MED | Test #1 asserts `grace_expires_at - now` within 5s of expected. The 3 severity tiers (24h/72h/168h) are SL-a-seeded |
| `compute_sponsor_liability` non-idempotent (e.g. caches mutate inputs) | LOW | MED | Test #5 (unit) asserts `assert_eq!(deltas1, deltas2)` over two consecutive calls. Watchpoint #7 |
| Concurrent SL-d producer + SL-c scheduler race on same case | LOW | LOW | FOR UPDATE at `submit_jury_vote.rs:198` (PR #98 cr-2) + SL-c per-case FOR UPDATE; single-threaded test harness can't exercise. Structural correctness inherited |
| `sponsor_liability_grace.rs` edits inadvertently land (SL-c module breach) | LOW | HIGH | Brief §4.3 hard refuses; §11 explicit NOT-touched list; §15.4 cross-cutting verification step (`git diff` empty). Junior subagent file-ownership boundaries enforce |
| Task 7 unit test infrastructure friction (`crates/api/api/` lacks testcontainers) | MED | LOW | Task 7 GOTCHA: read existing `#[cfg(test)]` blocks at task-start; if no in-crate pattern, file planner DQ asking whether to relocate to `crates/server/tests/e2e.rs`. Default lean (DQ #180): keep here. Worst case relocate; behaviour preserved |
| Wrapper-composition test (#4) cannot call `pub(crate)` `apply_sponsor_liability` directly | MED | LOW | Task 6 GOTCHA: drive via SL-c's `run_grace_check_batch`. SL-c-2 tests at e2e.rs:11932 already use this pattern; precedent established |
| GH-Actions minutes budget exceeded by 7 Phase-1 + 1 Phase-2 e2e | LOW | LOW | Phase 2 e2e local-default per JM-d retro §5; user picks dispatch only on audit-trail need. Total ~80 min monthly burn well within Free 3000 min |
| PMD #126 — DQ ID collision from concurrent SL-d / rep-tuning-r3 work | LOW | LOW | next-id calc spans archives + live; advisor monitors at finalize-merge. SL-d plan-write computed next_id = 186 (rt-r1-planning-1 clarify on advisor tip 72ad06e72 reserved 181-185) against live `decision-queue.json` at plan-write time (DQ #185 was the most recent advisor entry on advisor tip 72ad06e72 from rt-r1-planning-1 clarify) |
| JM-c TODO marker missing at task-time (intervening JM phase removed) | LOW | LOW | Task 0 Probe 8 catches; if marker absent, surface to advisor via DQ pending |
| Plan-time e2e.rs line count (12,819) mismatches impl-time (e.g. SL-e merged before SL-d cuts) | LOW | LOW | Task 0 Probe 13 detects (`wc -l`). If mismatch, fixture-mod placement still anchors AFTER the last `mod v1_*` — adapt anchor at Task 3 task-start |
| ts-rs derive regression on response DTO | LOW | LOW | SL-d does NOT modify `submit_jury_vote` DTO or response (PRD §11.3); existing ts-rs derives unchanged |
| Cross-PR carry-forward miss between SL-c-2 retro and SL-d planning (e.g. SL-c-2 retro flagged a SL-d expectation that SL-d doesn't honour) | LOW | LOW | SL-d Task 0 reads SL-c-2 retro at `.claude/PRPs/reports/v1-SL-c-2-retro.md`; specifically §3 carry-forward + §5 lessons. SL-d's Task 8 retro records cross-PR carry-forward findings |

---

## 19. Notes

### 19.1 Planner DQs filed (SL-d)

- **DQ #186** (`from: "planner"`, `kind: "blocker"`, `answered_by:
  "planner"` self-resolved per Recipe 2) — SL-d complexity-score
  split decision per §5.2. Question: "Complexity score 16 exceeds
  8 — split `v1-sponsor-liability-d` further (e.g. `v1-SL-d-1`
  split + grace-helper [Tasks 1-2] + `v1-SL-d-2` mutation + tests
  [Tasks 3-7]), or proceed?". Options: split / proceed. Planner
  self-resolved with **proceed** rationale: e2e factor dominates
  (+12 of +16 wrapped subtotal); SL-d-2 (mutation + tests) would
  still score 14 above threshold (further splitting yields no
  meaningful reduction); SL-b at 38 + SL-c-2 at 17 + JM-e at 15 +
  SL-a at 13 all shipped proceed-as-one without operational
  regret; per `feedback_principles_not_rules.md`, the e2e suite is
  anchor-Edit-friendly and the canonical Case A sibling discipline
  (post-SL-c-2 amendment) is established.

### 19.2 Self-resolved planner findings (LESSON candidates — SL-d)

- **The SL-d split-and-mutate posture is the canonical "wrapper-
  preserving refactor" pattern.** The split (compute + fire +
  wrapper) is observably non-disruptive to the SL-c call site
  because the wrapper preserves the v0 signature byte-for-byte.
  Test #4 (wrapper-composition) is the behavioural validation of
  this invariant. **Promote to PMD lesson candidate**:
  "wrapper-preserving refactor pattern — when splitting a public
  function into compute + write halves, retain the original
  signature as a thin wrapper to avoid disrupting downstream call
  sites; behaviourally verify with a compose-equivalence test".

- **The SL-d plan inherits the canonical-sibling stub-shape
  uniformity discipline from SL-c-2.** The 3-cycle SL-c-2
  catch-fire (cycles 1-3 same `(E0277, e2e.rs)`) was caused by
  stub-shape non-uniformity; SL-d's §13 Tasks 3-6 stubs explicitly
  prescribe `LemmyResult<()>` outer per
  `feedback_lemmy_error_no_std_error.md` Case A canonical sibling
  at e2e.rs:11001-11924, AND the GOTCHA blocks cite the lesson +
  the sibling line range explicitly. This is the first sub-phase
  to ship with the post-amendment lesson + the
  `feedback_plan_stub_uniformity_with_canonical_sibling.md`
  follow-on embedded in §13. **Recommend retro promotion**:
  confirm at SL-d retro time whether the discipline successfully
  prevented the 3-cycle pattern from reproducing.

- **Cross-PR carry-forward via SL-c retro → SL-d Task 0 (Probe 7
  module-presence + Probe 8 JM-c TODO marker) is the load-bearing
  hand-off discipline.** SL-d's Task 0 explicitly verifies SL-c
  module landed AND JM-c TODO marker present. Without these
  probes, SL-d risks being branched off a stale tip. Per
  `feedback_handover_trailer_cohort_propagation.md` (cross-cohort
  handover), the cross-PR analogue is the Probe-7 module-presence
  check + the SL-c retro §3 carry-forward + the SL-d Task 0 audit
  reading SL-c's retro. **Promote to PMD lesson candidate**:
  "cross-PR producer/consumer hand-off requires explicit Probe-N
  module-presence + Probe-N+1 hand-off-marker checks".

- **The deferred-write semantics test pattern is reusable for
  SL-e.** SL-d's Test #1 asserts `count(public_case_log) == 0` on
  Pending path; SL-e's lane-wide test will assert that AFTER the
  scheduler tick, `count(public_case_log) == 1`. The pattern of
  "negative assertion at vote-tally time + positive assertion at
  scheduler-tick time" is reusable. **Promote to PMD lesson
  candidate (post-SL-e ship)**: "deferred-write semantics testing
  — pair a negative assertion at the producer with a positive
  assertion at the consumer to validate the deferral".

### 19.3 Restoration-escape stub-and-future-wire breadcrumb

(Inherited from trunk SL-c plan §19.3 — SL-d does NOT add new
tests for restoration; restorative-mechanics-v1 PRD will add the
producer endpoint + consumer test.)

### 19.4 Pre-existing pending DQ entries

- **None at planning time.** `decision-queue.json` shows 0 pending
  entries on `governance-v0`. Recently resolved: DQ #176-#180
  (SL-d clarify pass via /brehon-clarify on 2026-05-10).
- **DQ #186 filed by this plan** (split-or-proceed;
  planner-resolved with proceed rationale).

### 19.5 Out-of-scope follow-ups (Task 8 retro candidates)

- **Lane-wide e2e suite** (full Decided → Pending → Fired/Escaped
  through SL-d transition + SL-c scheduler) — SL-e.
- **`restoration_complete` endpoint + restoration-escape branch
  detection** — restorative-mechanics-v1 PRD.
- **Sponsor notification UX** — PRD §13 OQ-V1-SL-03 (v3 polish).
- **Step-up auth for admin-driven sponsor-liability mutation** —
  v2.
- **Cross-instance federation of sponsor_liability_pending** — v2
  per ADR-014.
- **Multi-hop sponsor-of-sponsor liability** — PRD §13 OQ-V1-SL-02
  (v2 candidate).
- **Multi-sponsor escape rule (`all_revocation` /
  `majority_revocation`) at vote-tally time** — out per PRD §13
  OQ-V1-SL-01.

### 19.6 Confidence bands (SL-d)

- **High (9/10):** Split shape — exact mirror of v0 body's
  compute/fire boundary at `sponsor_liability.rs:142-358`. The
  wrapper-preserving discipline is canonical.
- **High (9/10):** test fixture mod shape — exact mirror of
  `mod v1_sl_b_fixtures` (Case A canonical sibling). Per-task
  anchor-Edit pattern well-rehearsed across SL-b + SL-c-2 + JM-e.
- **High (9/10):** `submit_jury_vote` mutation site — JM-c TODO
  marker explicitly points at the work site; brief §2.2 spells out
  the inside-handler step ordering verbatim.
- **High (8/10):** Test #1 Pending-transition assertion shape —
  every assertion grounded in PRD §9.3 step + §11.4 deferred-write
  spec.
- **High (8/10):** Test #2 no-sponsor + Test #3 NoAction —
  defendable v0 path semantics; map_decision_to_sanction short-
  circuit at line 423 + Vec::is_empty() check.
- **Moderate (7/10):** Test #4 wrapper-composition — visibility
  constraint requires driving via SL-c scheduler (Task 6 GOTCHA
  documents). SL-c-2 precedent established.
- **High (8/10):** Task 7 unit tests — straightforward
  idempotency + config-key-mapping; only risk is testcontainers
  infrastructure in `crates/api/api/` (Task 7 GOTCHA documents
  fallback to e2e.rs relocation).
- **Moderate (7/10):** §5 complexity score 16 trips threshold;
  planner self-resolved DQ #186 with proceed rationale citing
  e2e-factor analysis + SL-c-2/SL-b/JM-e proceed-as-one
  precedents.
- **High (9/10):** stub-shape uniformity (`feedback_lemmy_error_no_std_error.md`
  Case A + `feedback_plan_stub_uniformity_with_canonical_sibling.md`)
  encoded in §13 Tasks 3-6 GOTCHA blocks; cite canonical sibling
  line range explicitly. First plan to ship under the post-SL-c-2
  amendment.

### 19.7 Why no clarify DQ at impl time (SL-d)

Brief §4.2 enumerates the boundary-of-judgment cases that should
trigger an impl-side DQ. None apply to this SL-d plan as written:

- DQ #176 (notifications), #177 (persistence mechanism), #178
  (scope), #179 (log-emit position), #180 (task split) all LOCKED
  2026-05-10 — referenced in plan §7 + §11 + §13.
- v0 `apply_sponsor_liability` signature at
  `sponsor_liability.rs:142` is intact (Probe 9).
- Sanction multiplicity is 1:1 (Probe 14).
- ENTRY_KIND consts declared in SL-a (Probe 4).
- Shim re-exports present (Probe 5).
- SL-c module landed + scheduler call site signature stable
  (Probe 7).
- JM-c TODO marker present at line 458 (Probe 8).

If any baseline assumption changes between plan-write and
impl-time (specifically: SL-c module signatures drift, JM-c TODO
marker disappears, e2e.rs line count shifts dramatically), the
impl-task subagent files a DQ pending entry per
`feedback_principles_not_rules.md` + `decision-queue.md` Recipe 1.

### 19.8 Forward-only retrofit scope

Per `feedback_schema_changing_spec_retrofit_question.md` — SL-d
does NOT change the shape of any existing artifact class (no new
template section, no new schema marker, no new YAML field). All
§13 task contracts are routine impl-task contracts. No retrofit
question applies.

### 19.9 Cross-PR carry-forward discipline (SL-c → SL-d)

SL-d's Task 0 Probes 7 + 8 are the cross-PR module-presence +
TODO-marker checks. SL-d's Task 8 retro §2 (per-role signals)
records cross-PR carry-forward findings: did the SL-c-2 retro
accurately predict SL-d's needs? did Probe 7 catch any SL-c ship
gaps? did Probe 8 catch JM-c marker drift? did the SL-c-2 → SL-d
retro chain produce a clean hand-off?

This is the SECOND in-fork instance of cross-PR carry-forward
applied to a structurally-split sub-phase (after SL-c-1 → SL-c-2).
Promote findings to PMD per §19.2 recommendations.

---

## 20. Confidence score

- **Plan correctness:** 9/10 — patterns mirror v0
  `apply_sponsor_liability` body (split-line at compute/fire
  boundary), JM-c handler shape (`process_vote` outer-transaction
  + step ordering), v1-SL-b fixture mod (Case A canonical
  sibling). §13 task bodies anchor to literal line numbers +
  verbatim brief scope.
- **Cargo budget:** 10/10 — Shape G (cargo runs off-box; budget
  non-binding; forbidden-window non-binding for impl-task).
- **Test coverage:** 8/10 — 4 e2e tests + 2 unit tests covering
  Pending transition + no-sponsor path + NoAction path + wrapper
  composition + compute idempotency + grace-window mapping.
  Multi-sponsor escape rules at vote-tally time are not exercised
  (out per §12 — SL-c handles at scheduler-tick time).
  Restoration-escape branch is restorative-mechanics-v1 PRD's
  deliverable.
- **Story-grain decomposition:** 9/10 — SL-d ships 3 stories
  cleanly mapping to PRD §9.1 (Story 1: split + wrapper invariant)
  + §9.3 (Story 2: mutation + 3 path tests) + §9.4 (Story 3:
  grace-window helper).
- **Plan-shape conformance:** 9/10 — 20-section schema followed
  literally; §15.6 Shape G shape adopted; §16a mandatory present;
  §13 per-task FILES YAML block on every task; §5.1 complexity
  breakdown table mechanical; canonical-Case-A sibling referenced
  explicitly in §13 Tasks 3-6 GOTCHA blocks.

---

_Plan author: planning subagent (Junior worktree
`/srv/brehon-fork/.junior/worktrees/job-181` on
`junior/role-planning-v1-sponsor-liability-d-plan-...-181`,
2026-05-10 — clarify pass DQ #176-#180 resolved 2026-05-10 per
`f71840603 chore(advisor): clarify sl-d-planning-1`). Plan
committed on the worktree branch; Junior daemon's finalize step
pushes to `governance-v0`. BM-task cuts `phase-v1-SL-d` from
`governance-v0` after plan ships and user approves. SL-d score 16
above threshold; planner DQ #186 self-resolved with proceed
rationale (e2e factor dominates +12 of +16 total; further
splitting yields no meaningful reduction; SL-c-2 / SL-b / JM-e
proceed-as-one precedents). Confidence 9/10. Plan ships under
proceed-within-SL-d assumption (DQ #186)._
