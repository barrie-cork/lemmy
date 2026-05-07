# Plan: v1-sponsor-liability-c-2 — 5 `grace_check_*` e2e tests against the merged c-1 module + retro that flips registry markers

> **Shape G plan** — SL-c-2 ships under Shape G (Layer G2 push-and-exit). §15
> references workflow YAMLs by path + expected `conclusion`, not inline cargo.
> Cargo runs on GitHub-hosted runners (workspace-check on `junior/*`) and on
> the laptop / `workflow_dispatch` GH runner (e2e on `phase-v1-SL-c-2`
> post-finalize-merge — Phase 2 e2e gate is c-2's deliverable; c-1 had no
> Phase 2 e2e). See `.claude/PRPs/templates/plan.template.md` §15.6 +
> `.claude/PRPs/plans/v1-validate-agent.plan.md`.

> **Split context (c-2 only).** This plan is the second of two — `v1-SL-c-1`
> (sibling — module + scheduler wiring; ships first) and `v1-SL-c-2` (this
> plan — 5 e2e tests + retro; ships second, depends on c-1 merge). The
> split was directed by user (DQ #150) overriding the trunk plan's planner
> proceed-as-one lean (DQ #148). The trunk plan at
> `.claude/PRPs/plans/v1-sponsor-liability-c.plan.md` (committed `7af7dfa93`)
> stays in place as the historical record. c-1 and c-2 inherit
> §1 (Goal) / §2 (Anti-goals — minus the proceed-as-one rationale, which is
> superseded) / §3 (Problem statement) / §4 (Solution statement —
> watchpoints carried verbatim, with #12 binding c-2 specifically) /
> §6 (Relationship table — c-2 adds c-1 as upstream MERGED dep) /
> §7-§12 / §17-§20 from the trunk, with sub-phase-specific slices in
> §5/§11/§13/§14/§15.4/§16/§16a.

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
| 12 | NOT building in v1-SL-c-2 |
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

v1-SL-c-2 ships **5 integration tests** that behaviourally validate
the v1-SL-c-1 module shipped to `governance-v0`. The tests cover PRD
§6.1 (scheduler tick), §6.2 (per-case batch semantics), and §6.3
(failure mode + observability) — exercising the fire branch
(`SponsorLiabilityPending → SponsorLiabilityFired`), the escape branch
(`SponsorLiabilityPending → SponsorLiabilityEscaped`), the no-op case
(future grace_expires_at), the per-case isolation invariant (one bad
case must not block batch), and the `job.grace_check_batch_size` config
cap.

Tests pre-seed `SponsorLiabilityPending` cases via direct DB-write
(`ModerationCaseInsertForm`), pre-seed `surety` rows for active
sponsors, and where applicable mutate `surety.revoked_at` to exercise
the escape branch. Each test sets `BREHON_DISABLE_GRACE_CHECK_JOB=1` at
bootstrap so the c-1 cron tick doesn't race manual
`run_grace_check_batch` invocations. Tests live in a NEW
`mod v1_sl_c_fixtures` block in `crates/server/tests/e2e.rs`, anchored
after the last existing fixture mod.

c-2 also ships the c-1 + c-2 retros' registry-marker flips: the
`_SPONSOR_LIABILITY_FIRED` row's `(pending)` marker → `(active)`, and
the `_SPONSOR_LIABILITY_ESCAPED` row's SL-c portion `(pending)` →
`(active)`. Per `feedback_build_what_tests_exercise.md` (PMD #14):
`(active)` requires test exercising, not just code presence — c-1
shipped the code; c-2 ships the tests that exercise it; c-2 retro
flips the markers.

**c-2 ships ZERO new module / scheduler / migration code.** The c-1
module + clokwerk wiring shipped via `phase-v1-SL-c-1` PR (merged to
`governance-v0` before c-2 is cut). c-2 imports
`lemmy_api::governance::sponsor_liability_grace::run_grace_check_batch`
directly and invokes it from each test.

**Compute/fire posture (advisor-locked, masthead of `sl-c-planning-1.md`).**
SL-c calls **unsplit v0** `apply_sponsor_liability(conn, target_person_id,
case_id, community_id, action, &mut cache)` from the scheduler's fire
branch. The PRD §9.1 compute/fire split is **NOT** SL-c's deliverable —
it remains SL-d's. c-2's tests assert that the fire branch invokes the
v0 helper and that `sponsor_count` matches the active surety count.
(Posture statement carried verbatim from trunk plan §1.)

**Headline acceptance condition (c-2 only).** Stories 2 + 3 are `[done]`
with their checkpoint workflows green: (Story 2) the fire branch
transitions `SponsorLiabilityPending → SponsorLiabilityFired`, emits
per-sponsor `sponsor_liability_applied` entries (from v0
`apply_sponsor_liability`) plus the SL-c summary `sponsor_liability_fired`
entry, and writes `reputation_event` rows; (Story 3) the escape branch
transitions to `SponsorLiabilityEscaped`, sets `liability_escape_reason`
JSONB matching PRD §8.1 schema (`version: 1`, `actor_pseudonym` per
ADR-015), emits `sponsor_liability_escaped`, and writes NO
`reputation_event` rows; the per-case isolation invariant holds; the
batch_size config caps iteration; the future-grace case is correctly
no-op'd. (Story 1 already shipped + verified in c-1.)

This sub-phase touches NO schema, NO migration, NO ENTRY_KIND const, NO
CaseStatus variant, NO `governance_config` seed, NO HTTP endpoint, NO
DTO, NO route, NO new module, NO new scheduler block. It is **5 e2e
tests + retro that flips registry markers**.

---

## 2. Source

- `.claude/PRPs/briefs/sl-c-planning-1.md` @ `governance-v0` `cb0a9a4ea`
  — the original advisor brief (post-`/brehon-clarify`; DQ #144-#147
  resolved 2026-05-07).
- `.claude/PRPs/briefs/sl-c-split-planning-1.md` @ `governance-v0`
  (this plan's split brief; DQ #150 records the user override
  authorising the split into c-1 + c-2).
- `.claude/PRPs/plans/v1-sponsor-liability-c.plan.md` @ `7af7dfa93`
  — **trunk plan; this c-2 plan is the Tasks-3-7+retro slice**.
  §1, §3, §4, §6, §7-§12, §17-§20 inherited verbatim from trunk with
  c-2 / c-1 annotations.
- `.claude/PRPs/plans/v1-sponsor-liability-c-1.plan.md` (sibling) —
  ships first; c-2 depends on c-1 having merged to `governance-v0`.
  c-2 §6 records c-1 as MERGED upstream dependency.
- `.claude/PRPs/prds/v1-sponsor-liability.prd.md` @ `governance-v0` —
  parent PRD; **§6 IS the SL-c spec** (§6.1 scheduler tick, §6.2
  per-case batch semantics, §6.3 failure mode + observability, §6.4
  configuration); §1 (Brehon athgabál framing); §2 (IN/OUT — confirms
  SL-c is scheduler module + wiring + observability + tests); §3.1 +
  §3.4 (CaseStatus extensions — confirms SL-a shipped 3 variants;
  c-2 tests exercise `SponsorLiabilityPending → SponsorLiabilityFired`
  AND `→ SponsorLiabilityEscaped`); §7 (restoration interaction —
  restoration-completed escape branch is stub-only; c-2 does NOT
  test it); **§8.1 (`liability_escape_reason` JSONB schema —
  load-bearing for c-2 Test #2 defensive assertion)**; §9.1
  (`apply_sponsor_liability` split — SL-d's deliverable; c-1 calls
  UNSPLIT v0); §9.4 (helper module signatures — c-1 implements; c-2
  invokes); §9.5 (scheduler wiring — c-1 implements); §10 (Defaults
  Matrix — c-2 Test #5 overrides `job.grace_check_batch_size`); §11
  (backwards compat — §11.2 mid-flight cases; c-2's pre-seed
  semantically equivalent); §12 (security — §12.4 threat model);
  §15 (implementation phases — confirms SL-c is row 3); §17
  (cross-cutting — ADR-013 enum-exhaustiveness; c-2 adds zero
  variants); §18 (B4 key-rename table for `liability.*`).
- `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` — most recent
  shipped SL plan; §6 table shape (canonical mirror for c-2 §6); §13
  task ordering; §15 Shape G DoD shape; §10 §13 §16a conventions;
  §18 risks shape; **most importantly**, SL-b's e2e tests in
  `mod v1_sl_b_fixtures` are the immediate predecessor fixture-mod
  pattern c-2 follows.
- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — predecessor
  ships the schema + seeds + consts + variants c-2 reads. c-2 §11
  file inventory does not duplicate SL-a entries.
- `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` — most recent
  post-Shape-G + post-spec-kit-adoption plan; §15 per-workflow DoD,
  §16a Stories grain, §13 FILES YAML blocks, §5.1 complexity breakdown
  table, §13 anchor-Edit-per-test discipline.
- `.claude/PRPs/templates/plan.template.md` — canonical 20-section
  schema; §15.6 Shape G DoD; §16a Stories mandatory.
- `.claude/agents/planning.md` — subagent contract; §13 per-task FILES
  YAML block discipline; §5 complexity score rule; canonical-schema-
  first gate.
- `.claude/rules/decision-queue.md` — schema-v2 attribution (planner →
  `from: "planner"`, never `"advisor"`); Recipe 1 (raise blocker) +
  Recipe 2 (planner-resolved pre-seed); `kind: "validate-pending"`
  Shape G routing.
- `.claude/rules/advisor-orchestrator.md` — Stage shape "Each impl-task
  complete (under Shape G)" + Phase 2 e2e local-vs-dispatch user gate
  (PR #105). **For c-2: Phase 2 e2e IS triggered after Task 5's
  finalize-merge** (5 new tests must execute against the phase-branch
  tip).
- `.claude/rules/governance-log-entry-kind-registry.md` —
  `_SPONSOR_LIABILITY_FIRED` (line 172) and `_SPONSOR_LIABILITY_ESCAPED`
  (line 173) markers stay `(pending)` after c-1 retro per
  `feedback_build_what_tests_exercise.md`. **c-2 retro flips both** —
  `_FIRED` row entirely (e2e tests exercise the fire branch);
  `_ESCAPED` row's SL-c portion (e2e tests exercise the
  scheduler-branch escape).
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  — **ADR-005** (multi-dimensional reputation; c-2 Test #1 asserts
  per-sponsor `reputation_event` rows count == sponsor count),
  **ADR-008** (append-only signed log; c-2 Test #1 asserts every
  governance_log row matches `governance_log::append`'s shape),
  **ADR-010** (won't-disadvantage rule; v1 mid-flight backfill —
  c-2's pre-seed via `ModerationCaseInsertForm` is semantically
  equivalent), **ADR-013** (CaseStatus enum-exhaustiveness — no `_ =>`
  arms in c-2 matches; tests assert against named variants), **ADR-014**
  (federation deferral — c-2 tests assert no federation outbound),
  **ADR-015 (pseudonymisation — load-bearing for c-2 Test #2;
  defensive assertion that `actor_pseudonym` is a String, NOT raw
  `caller_id.0`)**, OQ-V1-SL-05 (`liability_escape_reason` schema
  versioning — c-2 Test #2 asserts `version: 1`).

### Lessons that bind §13 decisions (c-2)

- **`feedback_junior_worker_e2e_edit_hang.md`** —
  **load-bearing for c-2**. 5 e2e tests = 5 individual §13 tasks,
  each one anchor-pattern Edit at file end. e2e.rs is currently 10976
  lines on `governance-v0`; will be ~10500-10600 once SL-b merges
  (SL-b adds 9-10 tests at ~600 lines combined plus a fixture mod).
  Bundle Edits hang Junior workers.
- **`feedback_e2e_filter_assumes_naming.md`** — §15.7 manual
  validation snippets, if included, run full e2e suite without
  `--test e2e <filter>`, OR confirm via grep first. c-2 tests all
  start with `grace_check_` — convention enforced.
- **`feedback_build_what_tests_exercise.md`** (PMD #14) —
  **load-bearing for c-2 retro**. The `(pending) → (active)` registry
  marker flips happen NOW (in c-2 retro), AFTER e2e tests
  behaviourally exercise the fire-sites. c-1 shipped the code without
  flipping; c-2 ships the tests + the flip together.
- `feedback_multi_write_handlers_need_transactions.md` —
  citation-only for c-2 (no transaction code in c-2; c-1 ships the
  per-case tx). Tests assert outcome, not tx structure.
- `feedback_advisor_watchpoint_specificity.md` (cited by trunk) —
  every §4 watchpoint cites a concrete file/handler/`schema.rs`/
  `enums.rs` line. Binds §4 entries.
- **`feedback_complexity_score_pre_split.md`** — c-2 score 17
  (above threshold 8). Per the lesson:
  **planner files a `pending` DQ pre-commit asking split-or-proceed**
  with `from: "planner"`, `kind: "blocker"`, `answered_by: "planner"`
  self-resolved per Recipe 2. Recommended self-resolve: proceed (the
  e2e suite is anchor-Edit-friendly per
  `feedback_principles_not_rules.md`; further splitting fragments
  the suite). See §5.2.
- **`feedback_principles_not_rules.md`** — score is a signal, not
  a hard rule. c-2's 17 is dominated by the +15 e2e factor (5 ×
  +3); the 5 tests are mechanically anchor-Edit-friendly per the
  trunk plan §10.8 fixture-mod shape. SL-b shipped 9-10 e2e tests
  proceed-as-one without operational regret.
- `feedback_explicit_file_arrays_on_tasks.md` — every §13 task carries
  a FILES YAML block; cohort dispatch reads `union(creates, modifies)`.
- `feedback_parallel_cohort_dispatch.md` — Tasks 1-5 all
  `modifies: crates/server/tests/e2e.rs`; YAML overlap rule refuses
  cohort dispatch; tasks ship serially (no `[P]` markers in §13).
- `feedback_pre_phase_dod_smoke_test.md` +
  `feedback_plan_dod_dry_run_at_write.md` — advisor-side DoD smoke-test
  runs every §15 command literally before plan approval. Under Shape
  G the §15 entries name workflow YAML paths + `gh run list` queries —
  both verifiable mid-plan-approval.
- `feedback_features_full_p_crate_incompatible.md` — never `-p <crate>`
  + `--features full`. Workflow YAMLs already comply
  (`cargo-validate-workspace.yml:88-95`); §15.7 manual snippets respect.
- `feedback_features_full_workspace_only.md` — `--features full`
  required to activate `DbEnum` + `ts-rs` derives. Encoded in workflow
  YAML.
- `feedback_insertform_default_propagation.md` — citation-only: c-2
  does not add new InsertForm fields. Tests pre-seed
  `SponsorLiabilityPending` cases via existing
  `ModerationCaseInsertForm` (verified at
  `crates/db_schema/src/source/governance/moderation_case.rs:99-137`,
  carries `grace_expires_at` + `liability_escape_reason` from SL-a
  Task 4).
- `feedback_clippy_test_style.md` — R1 every `i32 ↔ i64` comparison
  uses `i64::from(...)`. c-2 tests primarily compare counts (`usize
  == usize`); R1 not a major surface but assertions on
  `sponsor_count` (i64 from json) vs `usize` may surface.
- `feedback_brehon_verify_pre_merge.md` — §16a Stories grain enables
  `/brehon-verify` phantom check before `bm-merge`. c-2's two
  stories (2 + 3) are checkpointed by Phase-1 workspace check on
  Task 5 + Phase-2 e2e on phase-branch tip.
- `feedback_read_canonical_before_writing_spec.md` — c-2 cites
  trunk plan + sibling shipped plans (SL-b §13 anchor-Edit pattern,
  JM-e §13 fixture-mod pattern); canonical-schema gate satisfied.
- `feedback_pr_per_phase.md` — one commit per task; §13 ordering
  matters.
- `feedback_handover_trailer_cohort_propagation.md` — non-binding
  under serial dispatch (no `[P]` cohorts in §13); per-task
  `HANDOVER:` commit trailer still recommended.
- `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`
  + `feedback_retro_task_complexity_score.md` — retro shape (final
  task; c-2 retro flips registry markers).
- `feedback_schema_changing_spec_retrofit_question.md` — citation-only
  (c-2 changes no spec/template shape).

### Related prior plans (canonical-shape mirror)

- `.claude/PRPs/plans/v1-sponsor-liability-c.plan.md` (trunk) —
  parent plan; c-2 inherits §1/§3/§4/§6/§7-§12/§17-§20 with c-1/c-2
  annotations.
- `.claude/PRPs/plans/v1-sponsor-liability-c-1.plan.md` — sibling;
  c-2 depends on c-1 having merged to `governance-v0`.
- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — schema
  foundation c-2 reads.
- `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` — most recent
  shipped SL plan; §13 anchor-Edit-per-test pattern + fixture-mod
  shape for `mod v1_sl_b_fixtures` is c-2's immediate predecessor.
- `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` — Shape-G
  + anchor-Edit-per-test reference.

---

## 3. Problem statement

(Inherited from trunk plan §3 with c-2 emphasis: post-c-1-merge, the
module + scheduler block exist on `governance-v0` but no test
exercises them. c-2 closes that gap.)

Post-SL-a-merge + post-c-1-merge on `governance-v0` (sequence: SL-a
@ `790f6101d` → SL-b PR #119 merged → c-1 PR merged):

- **The c-1 module + scheduler block ship as code but are not yet
  test-exercised.** The five public async fns
  (`run_grace_check_batch`, `evaluate_escape_conditions`,
  `fire_or_escape_case`, `check_grace_staleness`) compile + clippy
  clean (per c-1's workspace-check workflow) but no test invokes
  them. This is the design — c-1's split from c-2 was directed by
  user (DQ #150) precisely so c-2 can author tests against a
  merged module rather than against an unmerged worktree branch.
- **Registry markers `_SPONSOR_LIABILITY_FIRED` (line 172) and
  `_SPONSOR_LIABILITY_ESCAPED` (line 173, SL-c portion) remain
  `(pending)` after c-1 retro.** Per
  `feedback_build_what_tests_exercise.md` (PMD #14): `(active)`
  marker requires test exercising, not just code presence. c-2's
  retro flips them after the e2e tests behaviourally validate the
  fire-sites end-to-end.
- **The fire path's ADR-005 contract is not behaviourally asserted.**
  PRD §9.4 specifies that `apply_sponsor_liability` writes one
  `reputation_event` per sponsor + one `sponsor_liability_applied`
  log entry per sponsor; c-1's code calls the v0 helper but no
  c-1 test asserts the row counts. c-2 Test #1 is the
  authoritative behavioural assertion.
- **The escape path's ADR-015 pseudonym contract is not
  behaviourally asserted.** PRD §8.1 specifies that
  `liability_escape_reason` JSONB must carry `actor_pseudonym`
  (string), NOT raw `caller_id`; c-1's code constructs the JSON
  correctly via `actor_pseudonym_helper::get_or_create` but no
  c-1 test asserts the JSON shape. c-2 Test #2 is the
  authoritative defensive assertion (catches future regressions
  that might inadvertently write a raw id).
- **The per-case isolation invariant (PRD §6.3 + watchpoint #8) is
  not behaviourally asserted.** c-1's code wraps each case in
  `.inspect_err(|e| warn!(...)).ok()` so one bad case can't block
  the batch; c-2 Test #4 is the strong assertion catching this
  regression. Without c-2 Test #4, a future refactor could
  inadvertently propagate per-case errors to the outer batch and
  no test would catch it.
- **The batch_size config knob (PRD §6.4) is not behaviourally
  asserted.** c-1's code reads `job.grace_check_batch_size` from
  config and applies `.limit(batch_size_i64)` to the batch query;
  c-2 Test #5 asserts that overriding the config knob caps
  iteration as expected.
- **The future-grace no-op case is not behaviourally asserted.**
  c-1's batch query filters `.le(Some(now))` on
  `grace_expires_at`; c-2 Test #3 asserts that a case with
  future grace_expires_at is correctly skipped (no spurious side
  effects across `reputation_event`, `governance_log`, or
  `moderation_case` mutations).

The substrate is in place; c-2 supplies the 5 e2e tests + retro that
flips registry markers.

---

## 4. Solution statement

(Inherited from trunk plan §4 with c-2 / c-1 task-split annotations.
The architectural decisions are the same; c-1 shipped them in code,
c-2 ships behavioural validation.)

Five surgical changes for c-2, organised as 5 impl tasks (Tests #1
through #5) + Task 0 pre-flight + retro.

### 4.1 Architecturally load-bearing decisions (c-2 only — test side)

(Decisions that bind c-2's test code specifically. Architectural
decisions binding the c-1 module are inherited from c-1 §4.1; not
re-litigated here.)

- **The 5 e2e tests live in a NEW `mod v1_sl_c_fixtures` block in
  `crates/server/tests/e2e.rs`, after `mod v1_sl_b_fixtures`.** SL-b
  is in flight at trunk plan-write time; once it merges, e2e.rs
  grows to ~10500-10600 lines and `mod v1_sl_b_fixtures` becomes the
  last fixture mod. c-2 Task 1 (Test #1) anchors at the end of `mod
  v1_sl_b_fixtures` and opens `mod v1_sl_c_fixtures`. Subsequent
  c-2 tests (Tasks 2-5) anchor-Edit AFTER the prior task's commit
  tip, INSIDE the same mod, per `feedback_junior_worker_e2e_edit_hang.md`.
  Tasks 1-5 all `modifies: crates/server/tests/e2e.rs`; YAML
  overlap rule refuses cohort dispatch — they ship serially.
- **If SL-b has not merged at c-2 plan-implement time, Task 1's
  anchor falls back to end of `mod v1_jm_e_fixtures`** (line 9786 +
  ~1190 lines = ends near 10976 on `governance-v0` HEAD). The
  impl-task brief should grep at task-start-time:
  `grep -n '^mod v1_' crates/server/tests/e2e.rs | tail -1` to find
  the last fixture mod and anchor there. (Inherited from trunk plan
  §4.1.)
- **Tests pre-seed `SponsorLiabilityPending` cases via direct
  DB-write (`ModerationCaseInsertForm`)**, NOT via SL-d's mutation
  handler (which doesn't exist yet). The pre-seed sets
  `status: SponsorLiabilityPending`, `grace_expires_at: now() ± delta`,
  `target_person_id: Some(sponsee_id)`, `community_id: None or
  Some(...)`, plus the SL-a-mandatory fields per the existing
  `ModerationCaseInsertForm` shape. Tests also INSERT a `sanction`
  row per case (for fire-branch tests; escape-branch tests can omit
  if the escape happens before sanction lookup, but for consistency
  + future proofing all tests insert one). And tests INSERT `surety`
  rows for active sponsors.
- **`BREHON_DISABLE_GRACE_CHECK_JOB=1` env var override.** Tests set
  this at bootstrap; the c-1 scheduler's tick block returns early.
  Tests drive `run_grace_check_batch` directly to control timing.
- **Test imports (use block) for the new mod.** Tests import
  `lemmy_api::governance::sponsor_liability_grace::{run_grace_check_batch,
  GraceCheckBatchOutcome}` directly from the c-1-merged module; if
  the import path resolves, c-1 is correctly merged. If the import
  fails, Task 0 Probe 7 (c-1 module presence check) catches.
- **Shape G — DoD references workflow YAMLs by path + expected
  `conclusion`, not inline cargo.** Per
  `.claude/PRPs/templates/plan.template.md` §15.6 + DQ #67 resolution.
  Every §13 task's DoD references `cargo-validate-workspace.yml` on
  the worker branch + workflow_run_id captured by impl-task subagent
  post-push. Phase 2 e2e (the 5-test execution gate) fires on
  `phase-v1-SL-c-2` finalize-merge per advisor's local-vs-dispatch
  user gate (PR #105).
- **No migration round-trip workflow fires.** c-2 touches no
  `migrations/**` paths.

### 4.2 Watchpoints (specific files / handlers / schema lines)

(Inherited verbatim from trunk plan §4.2, with annotations on which
watchpoints bind c-2 specifically. c-1 ships the code; c-2 ships the
defensive assertions in tests.)

1. **Per-case `FOR UPDATE`. (c-1 binding — already shipped.)** c-2
   tests do NOT need to assert the FOR UPDATE lock directly (that's
   a sub-tx implementation detail); tests assert the OUTCOME — case
   correctly transitions even if a hypothetical concurrent SL-b
   revocation were running. Single-threaded test harness can't
   exercise true concurrency; structural correctness via the per-case
   tx body shape is the assertion. **c-2 Tests #1-5 implicitly rely
   on this watchpoint** (the per-case tx body shape is what makes
   the outcomes reproducible).
2. **Re-check status inside transaction. (c-1 binding — already
   shipped.)** Same as #1: tests assert outcome, not transaction
   step ordering.
3. **Atomic concurrency guard pattern. (c-1 binding — already
   shipped.)** c-2 tests rely on the c-1 guard pair existing —
   without it, the cron tick races the test's manual
   `run_grace_check_batch` invocation. Probe 8 confirms presence.
4. **`liability_escape_reason` JSON schema locked. (c-2 binding —
   Task 2 (Test #2) defensive ADR-015 assertion.)** Schema:
   `{"version": 1, "reason": "sponsor_revoked", "actor_pseudonym":
   "...", "endorsement_id": <i64>}`. **c-2 Test #2** asserts:
   - `json["version"] == 1`
   - `json["reason"] == "sponsor_revoked"`
   - `json["actor_pseudonym"]` is a String (per ADR-015)
   - `json["actor_pseudonym"]` does NOT equal `format!("{}",
     sponsor_id.0)` (defensive — catches a regression that writes
     the raw id)
   - `json["endorsement_id"] >= 0` (best-effort lookup may yield 0
     if no endorsement row was seeded)
5. **Two log entries per fire path, one per escape path. (c-2
   binding — Test #1 + Test #2 governance_log row count
   assertions.)** Fire path emits BOTH per-sponsor
   `sponsor_liability_applied` entries (from v0
   `apply_sponsor_liability` at `sponsor_liability.rs:322-337` per
   sponsor) AND the SL-c summary `sponsor_liability_fired` entry.
   Escape path emits ONLY the `sponsor_liability_escaped` entry.
   - **c-2 Test #1** asserts: `count(*) WHERE entry_kind =
     'sponsor_liability_applied'` matches active sponsor count
     (2); `count(*) WHERE entry_kind = 'sponsor_liability_fired'`
     == 1.
   - **c-2 Test #2** asserts: `count(*) WHERE entry_kind =
     'sponsor_liability_applied'` == 0; `count(*) WHERE
     entry_kind = 'sponsor_liability_fired'` == 0;
     `count(*) WHERE entry_kind = 'sponsor_liability_escaped'`
     == 1.
6. **Sanction-action lookup correctness. (c-1 binding — already
   shipped.)** c-2 tests pre-seed sanction rows; no defensive
   assertion needed beyond ensuring the test setup is correct.
7. **No new `moderation_case.status` mutations OUTSIDE the batch
   loop. (c-1 binding — already shipped.)** c-2 tests assert that
   only cases passing through the batch loop have their status
   mutated; the orphan-case `ModerationCaseInsertForm` pattern
   (used by JM-d cancellation tests with `creator_id=NULL +
   winning_decision=NoAction`) is the cautionary anchor — c-2's
   tests pre-seed `SponsorLiabilityPending` cases via
   `ModerationCaseInsertForm` direct-write but never mutate
   orphan cases.
8. **Per-case isolation — outer batch never returns `Err`.
   (c-2 binding — Test #4 strong assertion.)** **c-2 Test #4** is
   the authoritative assertion: case_b (malformed; zero sanction
   rows) ordered FIRST in the batch query, case_a (well-formed)
   second. Without the per-case error-catch in the outer for-loop
   (c-1 implementation), case_b's empty-sanction warn would have
   stopped iteration; case_a's fire would never have happened.
   Test asserts: case_a fired AND case_b skipped silently AND outer
   batch returned `Ok(...)`.
9. **`BREHON_DISABLE_GRACE_CHECK_JOB` env var precedes the atomic
   guard. (c-1 binding — already shipped.)** c-2 tests rely on the
   c-1 implementation's order. Probe 9 (env-var literal in
   scheduled_tasks.rs) confirms.
10. **No new ENTRY_KIND_*** consts. (c-2 binding — neither Task 1
    nor any subsequent task adds to the registry.)** All needed
    consts shipped in SL-a. **c-2 retro flips `(pending) →
    (active)` markers** for `_SPONSOR_LIABILITY_FIRED` (entirely)
    and `_SPONSOR_LIABILITY_ESCAPED` (SL-c portion).
11. **No migration in c-2. (c-2 binding.)** Plan §13 produces zero
    migration files. `git diff governance-v0..phase-v1-SL-c-2 --
    migrations/` at plan-approval time must show no output.
12. **e2e Edit-per-task discipline. (c-2 binding — load-bearing
    for c-2.)** 5 c-2 tests = 5 individual §13 tasks (Tasks 1-5),
    each one anchor-pattern Edit at file end. Do NOT bundle. Per
    `feedback_junior_worker_e2e_edit_hang.md` — e2e.rs is now
    10976 lines on `governance-v0`; ~10500-10600 once SL-b
    merges. Bundle Edits hang Junior workers. **This watchpoint
    is the single most important one for c-2's task-list shape**:
    c-2 ships 5 tests as 5 tasks specifically because of this
    constraint.
13. **R1 `i64::from(...)` discipline. (c-2 binding — Tests where
    cross-type comparison surfaces.)** c-2 surfaces:
    - `outcome.fired` (usize) vs literal `1` (i32 → usize coerced
      by literal form) — direct comparison, no bridge needed.
    - `sponsor_count` from JSON (i64) vs `usize` from
      `outcome.fired` — bridge via `i64::from(outcome.fired) == json["sponsor_count"].as_i64().unwrap()`
      where applicable.
14. **Restoration-escape stub-only (DQ #145). (c-2 binding —
    Test #2 does NOT exercise restoration; restorative-mechanics-v1
    PRD owns that test.)** c-2 ships zero tests for the
    restoration-completed branch; per
    `feedback_build_what_tests_exercise.md`, never test a producer
    that doesn't yet emit. The c-1 stub stays untouched in c-2.

### 4.3 Rejected alternatives (c-2 only)

- **Bundle the 5 e2e tests into 1-2 §13 tasks.** Rejected per
  `feedback_junior_worker_e2e_edit_hang.md` — e2e.rs is 10976+ lines;
  multi-test bulk Edits hang Junior workers. SL-b shipped 9-10
  tests as 9-10 tasks for the same reason.
- **Mark Tasks 1-5 as `[P]`.** Rejected: YAML overlap rule refuses
  cohort because all 5 tasks `modifies: crates/server/tests/e2e.rs`.
  Each task anchor-Edits at the prior task's commit tip; cohort
  dispatch would race.
- **Author the 5 e2e tests as part of c-1 (proceed-as-one).**
  Rejected per DQ #150 — user authorised the split at plan-approval
  gate.
- **Write the e2e tests against an unmerged c-1 worktree branch.**
  Rejected — c-2's tests need a stable module; the split exists
  precisely so c-2 can write tests against a merged c-1.
- **Pre-implement a test for the restoration-completed escape
  branch.** Rejected per DQ #145 advisor lean +
  `feedback_build_what_tests_exercise.md` (PMD #14). The
  `ENTRY_KIND_RESTORATION_COMPLETED` const has zero producers;
  testing detection of a non-existent emit is dead-code.
- **Flip registry markers in c-1 retro instead of c-2 retro.**
  Rejected per `feedback_build_what_tests_exercise.md` (PMD #14):
  `(active)` requires test exercising, not just code presence.
  c-1 ships code only; c-2 ships tests; c-2 retro flips.
- **Skip the `(pending) → (active)` flip entirely (leave markers
  pending forever).** Rejected — the registry rule's purpose is
  to track which consts have live emitters tested end-to-end. After
  c-2 ships, both `_FIRED` and `_ESCAPED` SL-c portion DO have
  live emitters tested end-to-end; the flip records that.

---

## 5. Metadata

- **Phase:** `v1-SL-c-2`
- **Branch:** `phase-v1-SL-c-2` (cut by BM-task before Task 1, AFTER
  c-1 PR merges to `governance-v0`)
- **Estimated tasks:** 7 (Task 0 pre-flight + Tasks 1-5 impl (5 e2e
  tests) + Task 6 retro)
- **Estimated cargo budget:** 0 GB peak local (Shape G — cargo runs
  on GH-hosted runners; Phase 2 e2e on laptop ~26 min single-threaded)
- **Forbidden-window applicability:** non-binding for impl-task
  throughput under Shape G; standard for any local diagnostic cargo
  run (rare). Phase 2 e2e on laptop respects forbidden windows per
  user gate.
- **Complexity score:** **17/10** — see breakdown below.
  Threshold-tripping; planner DQ filed (proceed-as-one rationale
  pre-seeded per Recipe 2 self-resolved).

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Computed mechanically
from §13 task list:

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | **0** | 5 impl tasks (Tasks 1-5; Task 0 + retro excluded). `max(0, 5-5) = 0` |
| Migrations touched | +2 each | **0** | c-2 ships zero migrations |
| Crates touched | +1 each | **1** | `lemmy_server` (tests/e2e.rs, Tasks 1-5). c-2 does NOT touch `crates/api/api/` or `crates/routes/` (those shipped in c-1) |
| `crates/lemmy_server/tests/e2e/*.rs` edits | +3 each | **15** | Tasks 1, 2, 3, 4, 5 each modify `crates/server/tests/e2e.rs`. 5 × +3 = 15 |
| New ADR-affecting decisions | +2 each | **0** | All ADR decisions made in trunk plan / DQ #144-#147; no new ADR-affecting decisions in c-2 |
| Cargo budget peak above 6 GB | +1 per GB | **0** | Shape G — cargo runs off-box |
| Wrapped subtotal | — | **16** |  |
| Adjustment | — | **+1** | Fold-in: c-1 dependency check at Task 0 Probe 7 (verifying c-1's module landed on `governance-v0`); requires explicit cross-PR carry-forward discipline at Task 0 + retro time |
| **Total** | — | **17** | Threshold for split-DQ: `>8`. **Tripped.** |

### 5.2 Split-or-proceed DQ

Per `feedback_complexity_score_pre_split.md` + `planning.md` §5
"Decision-queue — pre-seed forward-looking OQs": planner files
**DQ #151** (`from: "planner"`, `kind: "blocker"`, `answered_by:
"planner"` self-resolved with rationale per Recipe 2) BEFORE
committing the plan. Question: "Complexity score 17 exceeds 8 — split
`v1-sponsor-liability-c-2` further (e.g. `v1-SL-c-2-A` Tasks 1-2 +
`v1-SL-c-2-B` Tasks 3-5), or proceed?". Options: split-further /
proceed.

**Planner observation (binding lean — proceed):** the dominant
factor is the 5 e2e edits (+15 of the +16 wrapped subtotal).
Splitting c-2 further yields:
- **c-2-A** (Tasks 1-2): 2 impl tasks, 0 migrations, 1 crate, 2 e2e
  edits (5 + 0 + 1 + 6 = 12). Still above threshold.
- **c-2-B** (Tasks 3-5): 3 impl tasks, 0 migrations, 1 crate, 3 e2e
  edits (5 + 0 + 1 + 9 = 15). Still above threshold.

Further splitting does NOT bring the score below 8 because the
e2e-edit factor is +3 per test and there are 5 tests. The only way
to get c-2's score below 8 is to split such that each sub-phase
has ≤2 e2e edits — which means 3 phases (c-2-A: 2 tests, c-2-B: 2
tests, c-2-C: 1 test) and each phase still scores ≥6+3+3 = 12+ if
the dependency-check adjustment is preserved per phase. The split
does NOT meaningfully reduce the per-phase score.

Per `feedback_principles_not_rules.md`: the score is a signal, not
a hard rule. The 5 e2e tests are mechanically anchor-Edit-friendly:
each test is a single fn at file end inside `mod v1_sl_c_fixtures`;
the existing `bootstrap()` helper at `e2e.rs:767` provides users +
community; the trunk plan §10.8 fixture-mod shape is well-rehearsed
across SL-b's 9-10 tests + JM-e's tests + JM-d's tests. Per-task
wall-clock under Sonnet 4.6 likely 6-8 min per test (Edit +
workspace-check workflow). 5 tests × 7 min average = ~35 min
impl-task time; plus per-task validate-pending workflows.

Precedent for "score above threshold but proceed":
- SL-b: shipped at score 38 with proceed-as-one (DQ #143 — 9-10 e2e
  tests as 9-10 tasks; no operational regret).
- JM-e: shipped at score 15 with proceed-as-one.
- SL-a: shipped at score 13 with proceed-as-one.
- Trunk SL-c (overridden by user to split into c-1 + c-2): shipped
  at score 21 with proceed-as-one DQ #148 (overridden — but
  the proceed-as-one logic still holds for c-2 internally,
  because the e2e factor dominates).

c-2 at 17 is comparable to JM-e's 15 and below SL-b's 38 — within
the established proceed-as-one envelope for test-heavy phases.

**Per `feedback_principles_not_rules.md`:** "Mechanical scheduler
+tests work with strong precedents — `reputation_snapshot.rs::run_snapshot_batch`
+ `appeal_window_expiry.rs::run_appeal_window_expiry_batch` ship in
production since Phase 5a/5c with no operational regret. Each test
is anchor-Edit-friendly..." (carried verbatim from trunk plan §5.2.)

This plan ships under the **proceed** assumption. DQ #151 is filed
with `answered_by: "planner"` (self-resolved per Recipe 2) citing
the rationale above; the advisor may overturn at plan-approval time
if the user prefers a further split — but the e2e-factor analysis
suggests further splitting yields no meaningful score reduction.

DQ #150 (parent split decision: c-1 + c-2) is the binding precedent
for this split level. Per `feedback_principles_not_rules.md`,
further splitting requires the same kind of operational mandate
that DQ #150 provided (user override based on PR-scope preference);
absent such mandate, proceed-as-one within c-2 is the recommended
self-resolve.

---

## 6. Relationship to other v1-SL sub-phases

| Sub-phase | Status | What it ships | c-2 dependency |
|---|---|---|---|
| v1-SL-a | MERGED (PR #111, governance-v0 @ `790f6101d`) | Schema + 13 seeded keys + 5 entry-kind consts (incl. `_SPONSOR_LIABILITY_FIRED`, `_SPONSOR_LIABILITY_ESCAPED`, `_RESTORATION_COMPLETED`) + 3 CaseStatus variants + Issue #24 partial index + backfill | c-2 reads `moderation_case.{grace_expires_at, liability_escape_reason, severity, decided_at, target_person_id, community_id, status}`, `surety.{revoked_at, sponsor_id, sponsored_id}`, `sanction.{action, case_id}`, `governance_config.{job.grace_check_*, liability.grace_window_maximum_hours}` via test pre-seeds; tests assert on `_SPONSOR_LIABILITY_FIRED` (Test #1) and `_SPONSOR_LIABILITY_ESCAPED` (Test #2) emit row counts |
| v1-SL-b | MERGED expected (PR #119 on `phase-v1-SL-b`; c-2 plan-write awaits SL-b merge) | `revoke_endorsement` handler + DTO + route + 9-10 e2e tests | SL-b's `mod v1_sl_b_fixtures` is c-2's immediate predecessor fixture mod; c-2's `mod v1_sl_c_fixtures` opens AFTER it. SL-b's revocation-branch escape fires the SAME `_SPONSOR_LIABILITY_ESCAPED` const c-2's Test #2 asserts on (test scope: c-2 tests pre-seed `SponsorLiabilityPending` cases via direct DB-write, NOT via SL-b's revocation handler — cleaner isolation; SL-b's tests already cover the revocation-branch escape path) |
| v1-SL-c-1 | **MERGED** (PR #<TBD>, governance-v0 @ `<sha-TBD>`) | `sponsor_liability_grace.rs` module + scheduler wiring + atomic guard pair | **c-2 reads `pub async fn run_grace_check_batch`, `evaluate_escape_conditions`, `fire_or_escape_case`, `check_grace_staleness` from c-1; c-2 tests pre-seed `SponsorLiabilityPending` cases via direct DB-write and invoke `run_grace_check_batch` directly (per trunk plan §13 Tasks 3-7).** c-1 ship → c-2 cut from governance-v0 after c-1 PR merges |
| **v1-SL-c-2 (THIS PLAN)** | NOT YET CUT | 5 e2e tests (`grace_check_*`) against the merged c-1 module + retro that flips `(pending) → (active)` registry markers | — |
| v1-SL-d | PENDING (depends on c-1 + c-2 merges) | `submit_jury_vote` mutation + `apply_sponsor_liability` compute/fire split + `Decided → SponsorLiabilityPending` transition + `_SPONSOR_LIABILITY_PENDING` fire-site | SL-d transitions cases to `SponsorLiabilityPending` (the state c-2's tests pre-seed via direct DB-write). c-2 does NOT author SL-d's mutation. SL-d's compute/fire split refactor either keeps `apply_sponsor_liability` as a thin wrapper (c-2's test assertions remain correct) OR replaces with `compute_sponsor_liability` + `fire_sponsor_liability` (c-2's tests still pass because they assert on outcome via row counts, not on the helper's signature). Either is forward-compatible per advisor-locked posture |
| v1-SL-e | PENDING (depends on c-1 + c-2 + SL-d) | Lane-wide e2e suite (full grace-window flow exercising SL-d transition + c-1 scheduler) | c-2's tests pre-seed pending cases directly; SL-e's tests exercise the full lane (end-to-end through SL-d's transition). c-2 + SL-e are complementary (c-2 = scheduler-mechanic isolation; SL-e = full-lane integration) |
| restorative-mechanics-v1 | PENDING (separate PRD; not yet drafted) | `restoration_complete` endpoint + restoration-escape branch (defendant-initiated; admin-attested) | c-2 tests do NOT exercise the restoration-completed branch (per DQ #145 + `feedback_build_what_tests_exercise.md` — never test a producer that doesn't yet emit). When restorative-mechanics-v1 ships the producer, that PRD's plan adds the read-side query AND a corresponding e2e test inside its own fixture mod |

**Cross-PRD sequencing (per PRD §15 + §17.1):**

- c-2 parallel-safe with rep-tuning-r3/r4/r5 (different files +
  different concerns).
- c-2 parallel-safe with admin-dashboard-v1.
- c-2 unblocks SL-d (which writes the cases c-2's tests pre-seed)
  and SL-e (which exercises the full lane end-to-end).
- c-2 + c-1 together close out trunk plan v1-SL-c (SL-c row in PRD
  §15 marked complete after c-2 ships).

---

## 7. Preflight guardrails inherited from prior phases

(Inherited from trunk plan §7. c-2 specifically applies: DQ #44 +
DQ #144-#147 + DQ #150 + R1 + R4 + R5 + R6 + R7 + JM-d retro lessons
+ SL-a retro lessons + SL-b retro lessons. R2 + R3 are non-binding
for c-2 (no jury-assembly tests, no DTO).)

- **DQ #44 — Docker daemon preflight.** Enumerated in §13 Task 0
  (R5 — Probe 0). c-2's e2e suite uses testcontainers-rs; Docker
  Desktop / dockerd MUST be running before any test invocation.
  Probe 0 is mandatory.
- **DQ #144 — Two-tier ConfigCache lifecycle.** Resolved (advisor)
  in trunk; bound in c-1's code. **c-2 binding:** c-2 tests do NOT
  re-derive the ConfigCache pattern; they invoke `run_grace_check_batch`
  which uses c-1's ConfigCache implementation.
- **DQ #145 — Restoration-escape branch stub-only.** Resolved
  (advisor) in trunk; bound in c-1's code. **c-2 binding:** c-2
  tests do NOT exercise the restoration-completed branch.
- **DQ #146 — Staleness formula.** Resolved (advisor); bound in
  c-1's code. **c-2 binding:** c-2 tests do NOT explicitly assert
  staleness (pure observability — no e2e test for the
  `tracing::error!` emit path; structural assertion via c-1's
  function body shape covers it). c-2 tests focus on
  `run_grace_check_batch` outcome semantics.
- **DQ #147 — Both paired canonical batch-runner mirrors cited.**
  Resolved (advisor); bound in c-1's code. Citation-only for c-2.
- **DQ #150 — Split into c-1 + c-2 (binding).** Resolved (advisor
  on user authority): user override at plan-approval gate
  2026-05-07. c-2 ships e2e tests against the merged c-1 module.
- **DQ #151 (filed by this plan) — c-2 split-or-proceed.**
  Self-resolved (planner Recipe 2) with proceed rationale citing
  `feedback_principles_not_rules.md` + DQ #150 split-level
  precedent + e2e-factor analysis.
- **R1 — `i64::from(...)` on `i32 ↔ i64`.** Bound in §4 watchpoint
  #13 + §13 Tasks 1-5 GOTCHAs (where cross-type comparison
  surfaces, e.g. `sponsor_count` from JSON vs `usize` from
  `outcome.fired`).
- **R2 — `seed_jury_eligible_snapshots` BEFORE `admin_assign_jury` in
  tests.** Non-binding for c-2 (no jury-assembly tests).
- **R3 — struct-extension grep sweep.** Non-binding for c-2 (no
  DTO extensions).
- **R4 — lowercase snake_case test names.** Bound in §13 Tasks 1-5
  (all 5 tests use lowercase snake_case names per the trunk plan
  §13 Tasks 3-7).
- **R5 — pre-phase harness audit enumerates ALL probes.** Bound in
  §13 Task 0. Probes 0..15 listed (probe count higher than c-1
  because c-2 verifies c-1's merged state on top of SL-a + SL-b).
- **R6 — uniform `--no-deps -- -D warnings` clippy.** Encoded in
  `cargo-validate-workspace.yml:92`.
- **R7 — `cargo test --no-run -p lemmy_server --test e2e` on tasks
  touching a struct or re-export.** Encoded in
  `cargo-validate-workspace.yml:95`. Bound in Tasks 1-5 (e2e file
  edits).
- **JM-d retro §3.5 — `feedback_clippy_rerun_after_fix.md`.** Bound
  in §13 Tasks 1-5 GOTCHA — if test edits unmask an `unused-imports`
  lint, the workspace-check workflow's clippy step catches.
- **JM-d retro §5 — `feedback_laptop_default_for_validate_pending.md`.**
  Phase 2 e2e local-default per advisor-orchestrator user gate (PR
  #105, 2026-04-28). **Binds c-2** because c-2 has Phase 2 e2e gate.
- **SL-a retro §lessons — pseudonym discipline (ADR-015).** Bound in
  §4 watchpoint #4 + §13 Task 2 (Test #2 defensive ADR-015
  assertion).
- **SL-a retro §lessons — registry rule pre-landed-const exemption.**
  c-2 retro (Task 6) flips `(pending) → (active)` markers for the
  c-1-shipped fire-sites once c-2's e2e tests behaviourally exercise
  them.
- **SL-b retro lessons (expected).** Anything emerging from SL-b's
  retro inherits forward into c-2 (e.g. e2e fixture-mod naming
  convention).

---

## 8. Flow design

(Inherited from trunk plan §8. The architectural "before / after"
state diagram is identical because c-1 + c-2 together produce the
trunk's full deliverable. c-2 ships behavioural validation of c-1's
shipped flow.)

### 8.1 Before state (post-c-1-merge expected)

The grace-window lifecycle today (post-SL-a-merge + post-SL-b-merge
+ post-c-1-merge):

- The `sponsor_liability_grace.rs` module exists with 4 public
  async fns; clokwerk tick block fires every 5 min on boot;
  atomic concurrency guard pair `SPONSOR_LIABILITY_GRACE_RUNNING`
  + `GraceCheckRunningGuard` declared. **All shipped by c-1.**
- **No test exercises the c-1 module** end-to-end. The
  workspace-check workflow validates compile + clippy + test-no-run
  on c-1's worker branch, but no integration test invokes
  `run_grace_check_batch`.
- Registry markers `_SPONSOR_LIABILITY_FIRED (pending)` and
  `_SPONSOR_LIABILITY_ESCAPED (pending; SL-c portion)` remain
  unflipped per `feedback_build_what_tests_exercise.md` (PMD #14):
  c-1 shipped code without flipping; c-2 retro flips after e2e
  tests pass.

### 8.2 After state (post-c-2-merge — code + tests both shipped, registry markers flipped)

A new clokwerk tick (default 5 min) iterates pending-grace cases
**and is now exercised by 5 e2e tests** that pre-seed cases via
direct DB-write and invoke `run_grace_check_batch` directly.

(See trunk plan §8.2 for the full detailed flow diagram — the
architectural shape is identical; c-2 ships the tests that
behaviourally validate c-1's shipped flow.)

```
[Phase 2 e2e: tests/e2e.rs::v1_sl_c_fixtures]
  for each test:
    bootstrap() with BREHON_DISABLE_GRACE_CHECK_JOB=1
    seed users + community
    seed sponsors + active sureties
    seed SponsorLiabilityPending cases (via ModerationCaseInsertForm direct write)
    [optionally] mutate surety.revoked_at to exercise escape branch
    invoke run_grace_check_batch(&context).await
    assert outcomes (fired / escaped / skipped counts; case status; row counts)
```

### 8.3 Endpoint changes

NONE. c-2 is e2e tests + retro. Plan §13 must NOT include
`crates/api/api_common/src/governance.rs` edits or
`crates/api/routes/src/lib.rs` edits.

---

## 9. Mandatory reading

(Inherited from trunk plan §9; subset binding to c-2 — sections
9.1, 9.2 (subset focused on test-fixture patterns), 9.3, 9.4, 9.5
in full because c-2 implements 5 e2e tests against c-1's merged
module.)

### 9.1 Brehon design docs (P0)

- `.claude/PRPs/prds/v1-sponsor-liability.prd.md` §6 (full), §6.4,
  §6.3, §6.2, §6.1, §11.2, §12.4, §15 row 3, §17, §18, §3.1, §3.4,
  §7 (restoration interaction — c-2 does NOT test), §8.1
  (`liability_escape_reason` schema — c-2 Test #2 defensive
  assertion), §10 (Defaults Matrix — c-2 Test #5 overrides
  `job.grace_check_batch_size`)
- `.claude/PRPs/briefs/sl-c-planning-1.md` (trunk brief)
- `.claude/PRPs/briefs/sl-c-split-planning-1.md` (this plan's
  brief)
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  ADR-005, ADR-008, ADR-013, ADR-014, ADR-015, OQ-V1-SL-05

### 9.2 Codebase reads (P0 — mirror these patterns)

(c-2-binding subset — focus on test fixture patterns + module
imports.)

- `crates/api/api/src/governance/sponsor_liability_grace.rs` (c-1
  shipped) — full file. c-2 imports
  `run_grace_check_batch`, `GraceCheckBatchOutcome`,
  `EscapeStatus`, `evaluate_escape_conditions` from this module.
- `crates/api/api/src/governance/sponsor_liability.rs:1-358` —
  v0 `apply_sponsor_liability` semantics; c-2 Test #1 asserts on
  the per-sponsor `sponsor_liability_applied` log entries this
  helper emits.
- `crates/api/api/src/governance/governance_log.rs:1-81` (api shim
  re-exports) — c-2 tests reference
  `ENTRY_KIND_SPONSOR_LIABILITY_FIRED`,
  `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED`,
  `ENTRY_KIND_SPONSOR_LIABILITY_APPLIED` for governance_log row
  count assertions.
- `crates/db_schema/src/source/governance/governance_log.rs:196-199`
  — ENTRY_KIND_* const declarations.
- `crates/api/api/src/governance/actor_pseudonym_helper.rs` —
  `get_or_create` source; c-2 Test #2 asserts the resulting
  pseudonym shape.
- `crates/api/api/src/governance/config.rs:925-945` — grace-window
  + grace-check default consts.
- `crates/db_schema_file/src/enums.rs:411-432` — 3 SponsorLiability*
  CaseStatus variants + doc-comments.
- `crates/db_schema_file/src/schema.rs:772-801` — `moderation_case`
  table columns (incl. SL-a additions at 799-800).
- `crates/db_schema_file/src/schema.rs:1266-1278` — `sanction` table
  columns.
- `crates/db_schema_file/src/schema.rs:1347+` — `surety` table
  columns.
- `crates/db_schema/src/source/governance/moderation_case.rs:13-137`
  — `ModerationCase` struct + `ModerationCaseInsertForm` (incl.
  SL-a fields). c-2 tests pre-seed via this form.
- `crates/db_schema/src/source/governance/sanction.rs` — `Sanction`
  struct + `SanctionInsertForm`. c-2 tests pre-seed.
- `crates/db_schema/src/source/governance/surety.rs` — `Surety`
  struct + `SuretyInsertForm`. c-2 tests pre-seed; Test #2 mutates
  `revoked_at`.
- `crates/db_schema_file/src/schema.rs` `governance_config` table
  — c-2 Test #5 inserts/updates the `job.grace_check_batch_size`
  config row.
- `crates/server/tests/e2e.rs:88-...` — `mod governance_fixtures`
  (esp. `bootstrap()` at line 767 — multi-user fixture).
- `crates/server/tests/e2e.rs:9786-...` — `mod v1_jm_e_fixtures`
  (most recent fixture-mod pattern; c-2's `v1_sl_c_fixtures`
  follows).
- (Post-SL-b-merge): `mod v1_sl_b_fixtures` — closest fixture
  mirror (sponsor-liability-domain seeders).
- `crates/server/tests/e2e.rs:907` —
  `governance_log_hash_chain_holds` test pattern; c-2 tests follow
  the same `governance_log::table.filter(...)` query pattern for
  row count assertions.

### 9.3 Rules (P0 — auto-loaded)

- `.claude/rules/decision-queue.md` — DQ schema-v2; attribution
  integrity; mid-task push; planner Recipe 2 self-resolution.
- `.claude/rules/branch-manager.md` — file-ownership boundaries; BM
  cuts `phase-v1-SL-c-2` AFTER c-1 merge.
- `.claude/rules/phase-branch.md` — phase-branch + PR-into-`governance-v0`
  flow.
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy`
  mandatory.
- `.claude/rules/governance-log-entry-kind-registry.md` —
  pre-landed-const exemption + retro-time marker flip discipline.
  **For c-2: c-2 retro flips `(pending) → (active)` markers** per
  `feedback_build_what_tests_exercise.md` (e2e tests now exercise
  the fire-sites end-to-end).
- `.claude/rules/cargo-output-capture.md` +
  `.claude/rules/no-cargo-output-paste.md` — cargo invocation
  discipline (relevant for §15.7 manual snippets).
- `.claude/rules/pre-phase-harness-audit.md` — Task 0 audit shape.

### 9.4 Lessons (P0 — bound to §13 decisions)

(See §2 "Lessons that bind §13 decisions" — full enumeration. Most
load-bearing for c-2: `feedback_junior_worker_e2e_edit_hang.md`,
`feedback_complexity_score_pre_split.md`, `feedback_principles_not_rules.md`,
`feedback_build_what_tests_exercise.md`,
`feedback_e2e_filter_assumes_naming.md`.)

### 9.5 External documentation

- diesel `for_update()` semantics (citation-only for c-2 — c-1
  ships the FOR UPDATE call; c-2 tests assert outcome).
- chrono `Duration::hours(i64)` API (citation-only for c-2 —
  c-1 ships the chrono call; c-2 tests use `Duration::minutes(N)`
  for grace_offset seeding).
- testcontainers-rs Postgres lifecycle (existing usage across
  e2e.rs; c-2 tests inherit).

---

## 10. Patterns to mirror

(Inherited verbatim from trunk plan §10 §10.8 (test fixture mod
shape) + the relevant subsections referenced by c-2's tests. §10.1
through §10.5 + §10.7 are c-1 deliverables — citation-only for c-2.
§10.6 governance_log payload schemas binds c-2 Tests #1 + #2 row
count assertions.)

Per `feedback_advisor_watchpoint_specificity.md` + DQ #147 (paired
canonical batch-runner mirrors).

### 10.1-10.5, 10.7 — c-1 deliverables, citation-only for c-2

(See c-1 plan §10 + trunk plan §10 for full detail. c-2 tests
invoke the c-1 implementations; tests do not re-derive the
patterns.)

### 10.6 governance_log payload schemas (c-2 Test #1 + #2 binding)

**SOURCE:** PRD §8.1 + DQ #144/#145/#146 resolutions.

`liability_escape_reason` JSONB (written to `moderation_case` row
on escape branch — c-2 Test #2 defensive assertion):

```json
{
  "version": 1,
  "reason": "sponsor_revoked",
  "actor_pseudonym": "<revoking-sponsor-pseudonym>",
  "endorsement_id": <endorsement.id.0 (i64)>
}
```

`sponsor_liability_escaped` log entry payload (single entry per
escape — c-2 Test #2):

```json
{
  "case_id": <case.id.0>,
  "escaped_at": "<DateTime<Utc> ISO 8601>",
  "reason": "sponsor_revoked",
  "actor_pseudonym": "<revoking-sponsor-pseudonym>",
  "endorsement_id": <endorsement.id.0>
}
```

`sponsor_liability_fired` log entry payload (single SUMMARY entry
on fire — c-2 Test #1):

```json
{
  "case_id": <case.id.0>,
  "fired_at": "<DateTime<Utc> ISO 8601>",
  "target_pseudonym": "<sponsee-pseudonym>",
  "sponsor_count": <usize as i64>
}
```

**GOTCHA (ADR-015 — pseudonym discipline):** `actor_pseudonym` and
`target_pseudonym` are REQUIRED string fields. c-2 Test #2
defensively asserts `assert_ne!(json["actor_pseudonym"].as_str().unwrap(),
&format!("{}", sponsor_id.0))`.

**GOTCHA (layering invariant — fire branch):** c-1's code emits
ONE `sponsor_liability_applied` entry PER SPONSOR (from v0 helper)
+ ONE `sponsor_liability_fired` summary. c-2 Test #1 asserts BOTH:
the per-sponsor entries (count == active sponsor count) AND the
single summary.

### 10.8 Test fixture mod shape (c-2 binding)

**SOURCE:** `crates/server/tests/e2e.rs:9786-...` (`mod
v1_jm_e_fixtures`); post-SL-b-merge the canonical mirror is
`mod v1_sl_b_fixtures`.

```rust
mod v1_sl_c_fixtures {
  use super::*;
  use chrono::{DateTime, Duration, Utc};
  use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, insert_into, update};
  use diesel_async::RunQueryDsl as AsyncRunQueryDsl;
  use lemmy_api::governance::sponsor_liability_grace::{
    GraceCheckBatchOutcome, run_grace_check_batch,
  };
  use lemmy_db_schema::{
    newtypes::{ModerationCaseId, PersonId, SanctionId, SuretyId},
    source::governance::{
      moderation_case::ModerationCaseInsertForm,
      sanction::SanctionInsertForm,
      surety::SuretyInsertForm,
    },
  };
  use lemmy_db_schema_file::{
    enums::{
      CaseSeverity, CaseStatus, CaseStatusTier, CaseTargetType,
      SanctionAction, SanctionScope, SeverityTier,
    },
    schema::{governance_log, moderation_case, sanction, surety},
  };

  async fn seed_pending_case(
    conn: &mut AsyncPgConnection,
    sponsee: PersonId,
    grace_offset: Duration,
    sanction_action: Option<SanctionAction>,
  ) -> Result<ModerationCaseId, Box<dyn Error>> {
    // ... (full body in §13 Task 1)
  }

  async fn seed_active_surety(
    conn: &mut AsyncPgConnection,
    sponsor: PersonId,
    sponsee: PersonId,
  ) -> Result<SuretyId, Box<dyn Error>> {
    // ... (full body in §13 Task 1)
  }

  // ... 5 #[tokio::test] async fn ... per Tasks 1-5 ...
}
```

**GOTCHA (R4 — test fn names):** lowercase snake_case.

**GOTCHA (e2e fixture-mod placement):** the new mod opens AFTER the
last existing fixture mod (post-SL-b-merge: `mod v1_sl_b_fixtures`;
fallback if SL-b unmerged: `mod v1_jm_e_fixtures` at line 9786 +
~1190 lines = ends near 10976).

**GOTCHA (BREHON_DISABLE_GRACE_CHECK_JOB):** every c-2 test sets
this env var at bootstrap — otherwise the c-1 cron tick races the
test's manual `run_grace_check_batch` invocation. Per
`scheduled_tasks.rs:198`'s pattern.

---

## 11. Files to change

### `lemmy_server` crate

- `crates/server/tests/e2e.rs` — append new `mod v1_sl_c_fixtures`
  AFTER the last existing fixture mod (post-SL-b-merge: after `mod
  v1_sl_b_fixtures`; fallback: after `mod v1_jm_e_fixtures` at line
  10975 on `governance-v0`). Within the mod, ship 5 tests via 5
  anchor-Edit tasks (Tasks 1-5). Task 1 ships the mod shell +
  helpers + test #1; Tasks 2-5 anchor-insert subsequent tests
  inside the same mod. **Tasks 1-5**.

### Meta files (rules + reports)

- `.claude/rules/governance-log-entry-kind-registry.md` — flip
  `(pending)` → `(active)` markers on the SL-c fire-sites:
  `_SPONSOR_LIABILITY_FIRED` row entirely;
  `_SPONSOR_LIABILITY_ESCAPED` row's SL-c portion (the SL-b portion
  was flipped by SL-b retro). **Task 6 (retro)**.
- `.claude/PRPs/reports/v1-SL-c-2-retro.md` — CREATE retro per
  `feedback_retro_not_report.md` +
  `feedback_four_role_retro_signals.md` +
  `feedback_retro_task_complexity_score.md`. **Task 6**.

### Files explicitly NOT touched (c-2 only)

- `crates/api/api/src/governance/sponsor_liability_grace.rs` — c-1
  shipped; c-2 imports + invokes from tests but does NOT modify.
- `crates/api/api/src/governance/mod.rs` — c-1 shipped the `pub mod
  sponsor_liability_grace;` line; c-2 does NOT modify.
- `crates/routes/src/utils/scheduled_tasks.rs` — c-1 shipped the
  scheduler block + atomic guard pair; c-2 does NOT modify.
- `crates/api/api/src/governance/sponsor_liability.rs` — v0 helper
  stays intact through c-2; SL-d is the compute/fire split.
- `crates/api/api/src/governance/submit_jury_vote.rs` — SL-d adds
  the `Decided → SponsorLiabilityPending` transition.
- `crates/api/api_crud/src/governance/revoke_endorsement.rs` — SL-b
  ships.
- `crates/api/api/src/governance/governance_log.rs` (api shim) — no
  re-export changes.
- `crates/db_schema/src/source/governance/governance_log.rs` — no
  const additions.
- `crates/db_schema/src/source/governance/{moderation_case,sanction,surety,endorsement}.rs`
  — no struct changes.
- `crates/db_schema_file/src/{enums,schema}.rs` — no schema changes.
- `migrations/**` — zero migrations.
- `crates/api/api/src/governance/config.rs` — no const additions.
- `crates/db_views/governance_case/src/impls.rs` — view-crate stays
  unchanged.
- `crates/api/api/src/governance/{request_appeal,admin_close_case,admin_assign_jury,reputation_snapshot,appeal_window_expiry,...}.rs`
  — already ADR-013-extended in SL-a Task 5; c-2 does not touch
  any of these handlers.
- `crates/api/api_common/src/governance.rs` — no DTO additions
  (c-2 is test-only).
- `crates/api/routes/src/lib.rs` — no route registration.
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
  `.coderabbit.yaml` — no dep / build-config / review-config changes.
- `.github/workflows/**` — no workflow YAML changes.

---

## 12. NOT building in v1-SL-c-2

(Inherited from trunk plan §12; c-2 specifically additionally
excludes the c-1 module + scheduler code (which shipped in c-1).)

- **`sponsor_liability_grace.rs` module + scheduler block + atomic
  guard pair** — c-1's deliverable; shipped before c-2 cuts.
- **`apply_sponsor_liability` compute/fire split** — SL-d's. c-2
  tests assert on the v0 helper's output (per-sponsor
  `sponsor_liability_applied` log entries + reputation_event rows).
- **`submit_jury_vote` mutation: `Decided → SponsorLiabilityPending`
  transition** — SL-d's. c-2's tests pre-seed
  `SponsorLiabilityPending` cases via direct DB-write.
- **`revoke_endorsement` handler** — SL-b's; c-2 expects SL-b
  merged before c-2 plan-implement; c-2 tests do NOT invoke SL-b's
  handler (cleaner isolation).
- **`restoration/complete` endpoint + restoration-escape branch
  detection** — restorative-mechanics-v1 PRD's. c-2 tests do NOT
  exercise the restoration-completed branch.
- **e2e behavioural tests for the full lane** (Decided → Pending →
  Fired/Escaped through SL-d transition + c-1 scheduler) — SL-e's.
  c-2's tests pre-seed `SponsorLiabilityPending` directly.
- **Sponsor notification UX** — out per PRD §13 OQ-V1-SL-03.
- **Step-up auth for admin-driven scheduler runs** — out per PRD
  §12.3 (v2 reservation).
- **Cross-instance federation of grace-window events** — out per
  PRD §2 OUT + ADR-014.
- **New ENTRY_KIND_*** consts** — all needed shipped in SL-a. c-2
  tests assert against existing consts.
- **New CaseStatus variants** — SL-a shipped 3.
- **New `governance_config` seeds** — SL-a shipped 13. c-2 Test #5
  UPSERTs `job.grace_check_batch_size` (existing seed key); does
  NOT add new seed keys.
- **Schema migrations** — none.
- **Backfill of v0 cases** — SL-a Task 1 already backfilled.
- **Scheduler-tick-interval e2e test** — non-binding for c-2 (covered
  by existing snapshot/appeal-window scheduler-tick tests'
  precedent; c-2 relies on `BREHON_DISABLE_GRACE_CHECK_JOB=1` +
  direct `run_grace_check_batch` invocation).
- **Staleness threshold semantic e2e test** — non-binding for c-2
  (pure tracing emit; structural assertion via c-1 code body shape).
- **Multi-sponsor `all_revocation` / `majority_revocation` rules
  at scheduler-tick time** — covered by SL-b's tests at revocation
  time; c-2's scheduler-branch escape uses `surety.revoked_at >=
  decided_at` check (PRD-aligned simplification — see trunk plan
  §12 for full rationale).

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task** (per
`feedback_pr_per_phase.md`). c-2 ships 5 impl tasks (5 e2e tests)
+ Task 0 pre-flight + retro. No `[P]` cohorts (Tasks 1-5 all
`modifies: crates/server/tests/e2e.rs`; YAML overlap rule refuses
cohort dispatch — they ship serially). Task 0 is always non-`[P]`.

> **Cohort dispatch (advisor-side):** No `[P]` cohorts in c-2. All
> tasks dispatch serially per `advisor-orchestrator.md`. Single-task
> cohort each; cohort handover aggregation per
> `feedback_handover_trailer_cohort_propagation.md` still applies as
> a per-task `HANDOVER:` commit trailer when the next task benefits.

> **Shape G (Layer G2 push-and-exit):** §13 task bodies do NOT inline
> cargo invocations. Each task ends with a push to the worker branch;
> the impl-task subagent writes a `kind: "validate-pending"` DQ entry
> referencing `cargo-validate-workspace.yml` per
> `.claude/rules/decision-queue.md` schema-v2.

### Task 0: Pre-flight harness audit + branch verification + SL-a/SL-b/c-1 state confirmation

**Goal:** verify environment + branch (`phase-v1-SL-c-2`) +
SL-a + SL-b + **c-1 module** all merged on `governance-v0`.

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
git submodule status > /tmp/sl-c-2-task0-submodule.log 2>&1
if grep -q '^-' /tmp/sl-c-2-task0-submodule.log; then
  git submodule update --init --recursive > /tmp/sl-c-2-task0-submodule-init.log 2>&1
  echo "submodule init exit: $?"
fi

# Probe 1 — branch verification
git branch --show-current
# EXPECT: phase-v1-SL-c-2 (BM-task cuts before Task 1, AFTER c-1 merges)

# Probe 2 — SL-a state confirmation: 3 CaseStatus variants present
rg -n 'SponsorLiabilityPending|SponsorLiabilityFired|SponsorLiabilityEscaped' crates/db_schema_file/src/enums.rs | head
# EXPECT: 3+ matches (variant declarations near lines 416/421/427)

# Probe 3 — SL-a state confirmation: schema columns present
rg -n 'grace_expires_at -> Nullable<Timestamptz>|liability_escape_reason -> Nullable<Jsonb>' crates/db_schema_file/src/schema.rs | head
# EXPECT: lines 799 + 800

# Probe 4 — SL-a state confirmation: ENTRY_KIND consts declared
rg -n 'ENTRY_KIND_SPONSOR_LIABILITY_FIRED|ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED|ENTRY_KIND_RESTORATION_COMPLETED' crates/db_schema/src/source/governance/governance_log.rs | head
# EXPECT: 3 matches at lines 196 (RESTORATION_COMPLETED), 197 (ESCAPED), 198 (FIRED)

# Probe 5 — SL-a state confirmation: shim re-exports
rg -n 'ENTRY_KIND_SPONSOR_LIABILITY_FIRED|ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED|ENTRY_KIND_RESTORATION_COMPLETED' crates/api/api/src/governance/governance_log.rs | head
# EXPECT: 3 matches in pub use block (lines 68, 74, 75)

# Probe 6 — SL-a state confirmation: config keys + defaults seeded
rg -n 'DEFAULT_JOB_GRACE_CHECK_INTERVAL_MINUTES|DEFAULT_JOB_GRACE_CHECK_BATCH_SIZE|DEFAULT_JOB_GRACE_CHECK_STALENESS_ALERT_MULTIPLIER|DEFAULT_LIABILITY_GRACE_WINDOW_MAXIMUM_HOURS' crates/api/api/src/governance/config.rs | head
# EXPECT: 4 const declarations near lines 929 + 943-945

# Probe 7 — c-1 module landed on governance-v0 (CRITICAL — c-2 depends on c-1)
rg -n 'pub mod sponsor_liability_grace;' crates/api/api/src/governance/mod.rs
# EXPECT: 1 line (the pub mod declaration shipped by c-1 Task 1)
test -f crates/api/api/src/governance/sponsor_liability_grace.rs && echo "C1 MODULE FILE PRESENT" || { echo "C1 MODULE NOT FOUND — c-2 cannot proceed; check c-1 PR merge status"; exit 1; }

# Probe 7b — c-1 module exports the 4 expected pub fns
rg -n 'pub async fn run_grace_check_batch|pub async fn evaluate_escape_conditions|pub async fn fire_or_escape_case|pub async fn check_grace_staleness' crates/api/api/src/governance/sponsor_liability_grace.rs
# EXPECT: 4 lines

# Probe 8 — c-1 atomic concurrency guard pair landed in scheduled_tasks.rs
rg -n 'SPONSOR_LIABILITY_GRACE_RUNNING|GraceCheckRunningGuard' crates/routes/src/utils/scheduled_tasks.rs | head
# EXPECT: at least 3 matches (static decl + impl Drop + compare_exchange)

# Probe 9 — c-1 BREHON_DISABLE_GRACE_CHECK_JOB env-var override landed
rg -n '"BREHON_DISABLE_GRACE_CHECK_JOB"' crates/routes/src/utils/scheduled_tasks.rs
# EXPECT: 1 match

# Probe 10 — SL-b merge confirmation: revoke_endorsement handler shipped
test -f crates/api/api_crud/src/governance/revoke_endorsement.rs && echo "SL-b SHIPPED" || echo "SL-b NOT YET MERGED — Task 1 anchors fall back to mod v1_jm_e_fixtures end"

# Probe 11 — last fixture mod identification (Task 1 anchor)
rg -n '^mod v1_' crates/server/tests/e2e.rs | tail -1
# EXPECT: mod v1_sl_b_fixtures (post-SL-b-merge) OR mod v1_jm_e_fixtures (fallback)

# Probe 12 — v0 apply_sponsor_liability signature confirmation
rg -n 'pub\(crate\) async fn apply_sponsor_liability' crates/api/api/src/governance/sponsor_liability.rs
# EXPECT: 1 match at line ~142; signature: (conn, target_person_id, case_id, community_id, action: SanctionAction, cache: &mut ConfigCache) -> LemmyResult<usize>

# Probe 13 — sanction multiplicity invariant (1:1 with case)
rg -n 'insert_into\(sanction::table\)' crates/api/api crates/api/api_crud
# EXPECT: 1 match at submit_jury_vote.rs:435 (single insert per case — 1:1 multiplicity)

# Probe 14 — PM-plugin-hooks-stable check (per .claude/rules/pm-plugin-hooks-stable.md)
for h in \
  local_private_message_before_create \
  local_private_message_after_create \
  local_private_message_before_update \
  local_private_message_after_update \
  federated_private_message_before_receive \
  federated_private_message_after_receive; do
  rg -q "\"$h\"" crates/ || { echo "missing hook literal: $h"; exit 1; }
done

# Probe 15 — concurrent-PR check
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("tests/e2e\\.rs|sponsor_liability_grace\\.rs|scheduled_tasks\\.rs|governance/mod\\.rs")) | {number, title, headRefName}'
# EXPECT: empty output (no concurrent PRs touching c-2-owned files)

# Probe 16 — registry markers still pending (c-2 retro flips them)
rg -n '\(pending\)' .claude/rules/governance-log-entry-kind-registry.md | rg -E 'SPONSOR_LIABILITY_FIRED|SPONSOR_LIABILITY_ESCAPED'
# EXPECT: at least 2 matches (FIRED row + SL-c portion of ESCAPED row)

# Probe 17 — Shape G workflow YAMLs accessible (yamllint may not be installed; soft-fail)
yamllint .github/workflows/cargo-validate-workspace.yml \
         .github/workflows/cargo-test-e2e.yml > /tmp/sl-c-2-task0-yamllint.log 2>&1 || \
         echo "yamllint not installed or warnings — non-blocking; advisor verifies workflow shape pre-merge"

# Probe 18 — DQ pending status (advisory)
python3 -c "import json; d=json.load(open('.claude/decision-queue.json')); print('pending:', [(e['id'], e.get('kind'), e.get('from')) for e in d.get('pending',[])])"
```

**EXPECT:** Probes 0..16 exit 0 (or, for Probe 0/1/7, exit 1 with
explicit STOP). Probes 17-18 are informational.

**No commit at Task 0** — verification only.

### Task 1: e2e test #1 — Fire path (expired pending case, no escape conditions) + open `mod v1_sl_c_fixtures` shell + helpers

**ACTION:** in `crates/server/tests/e2e.rs`, append a NEW `mod
v1_sl_c_fixtures` block AFTER the last existing fixture mod. Inside
the new mod, add the use block + 2 helper fns
(`seed_pending_case`, `seed_active_surety`) per §10.8 + the first
test fn `grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # open mod v1_sl_c_fixtures + helpers + test #1
```

**IMPLEMENT (file 1 of 1):** anchor-Edit at file end. The anchor is
the closing `}` of the last existing fixture mod.

**Anchor identification at task-start:**
```bash
grep -n '^mod v1_' crates/server/tests/e2e.rs | tail -1
```
- If output is `mod v1_sl_b_fixtures` (post-SL-b-merge expected):
  the new mod opens AFTER the closing `}` of that mod.
- Fallback: if SL-b is unmerged at task-time (Probe 10 reported
  fallback), the new mod opens AFTER the closing `}` of `mod
  v1_jm_e_fixtures` (line 9786 + ~1190 lines of fixture body).

**Content to append** (carried verbatim from trunk plan §13 Task 3):

```rust
mod v1_sl_c_fixtures {
  use super::*;
  use chrono::{DateTime, Duration, Utc};
  use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, insert_into, update};
  use diesel_async::RunQueryDsl as AsyncRunQueryDsl;
  use lemmy_api::governance::sponsor_liability_grace::{
    GraceCheckBatchOutcome, run_grace_check_batch,
  };
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
      CaseSeverity, CaseStatus, CaseStatusTier, CaseTargetType,
      SanctionAction, SanctionScope, SeverityTier,
    },
    schema::{governance_log, moderation_case, sanction, surety},
  };

  /// Seed a `SponsorLiabilityPending` case for `sponsee` with a
  /// chosen grace_offset (relative to now; negative for expired,
  /// positive for not-yet-expired). Optionally inserts a sanction
  /// row with the given `SanctionAction`.
  async fn seed_pending_case(
    conn: &mut AsyncPgConnection,
    sponsee: PersonId,
    grace_offset: Duration,
    sanction_action: Option<SanctionAction>,
  ) -> Result<ModerationCaseId, Box<dyn Error>> {
    let now = Utc::now();
    let decided_at = now - Duration::hours(24);
    let grace_expires_at = now + grace_offset;
    let case_id = insert_into(moderation_case::table)
      .values(ModerationCaseInsertForm {
        target_type: CaseTargetType::Person,
        target_person_id: Some(sponsee),
        reason_code: "v1_sl_c_test".to_string(),
        severity: CaseSeverity::Medium,
        status: CaseStatus::SponsorLiabilityPending,
        threshold_score: 100,
        grace_expires_at: Some(grace_expires_at),
        ..Default::default()
      })
      .returning(moderation_case::id)
      .get_result::<ModerationCaseId>(conn)
      .await?;
    update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
      .set(moderation_case::decided_at.eq(Some(decided_at)))
      .execute(conn)
      .await?;
    if let Some(action) = sanction_action {
      insert_into(sanction::table)
        .values(SanctionInsertForm {
          case_id,
          scope: SanctionScope::Community,
          action,
          target_person_id: Some(sponsee),
          ends_at: None,
          active: Some(true),
          ..Default::default()
        })
        .execute(conn)
        .await?;
    }
    Ok(case_id)
  }

  /// Seed an active (non-revoked) surety from sponsor → sponsee.
  async fn seed_active_surety(
    conn: &mut AsyncPgConnection,
    sponsor: PersonId,
    sponsee: PersonId,
  ) -> Result<SuretyId, Box<dyn Error>> {
    insert_into(surety::table)
      .values(SuretyInsertForm {
        sponsor_id: sponsor,
        sponsored_id: sponsee,
        community_id: None,
      })
      .returning(surety::id)
      .get_result::<SuretyId>(conn)
      .await
      .map_err(|e| e.into())
  }

  #[tokio::test]
  async fn grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries() -> Result<(), Box<dyn Error>> {
    // Per Test #1 (PRD §6.1 + §6.2 step 5 fire branch).
    //
    // Setup: BREHON_DISABLE_GRACE_CHECK_JOB=1 (via bootstrap).
    //   1 sponsee + 2 sponsors. Active sureties (no revocations).
    //   ModerationCase status=SponsorLiabilityPending,
    //     grace_expires_at = now() - 1 minute (expired).
    //   sanction row with SanctionAction::ContentRemoval (Moderate severity).
    //
    // Drive: run_grace_check_batch(&context).await.
    //
    // Assert:
    //   - outcome.fired == 1, outcome.escaped == 0, outcome.skipped == 0,
    //     outcome.cases_processed == 1.
    //   - case.status == SponsorLiabilityFired.
    //   - 2 reputation_event rows for the sponsors (sponsor_count == 2).
    //   - 1 governance_log row "sponsor_liability_fired" (SL-c summary).
    //   - 2 governance_log rows "sponsor_liability_applied" (one per sponsor;
    //     v0 helper output).
    //   - 0 governance_log rows "sponsor_liability_escaped".
    //   - "sponsor_liability_fired" payload contains target_pseudonym (str),
    //     sponsor_count = 2, case_id matching.
    //   - case.liability_escape_reason IS NULL (fire branch doesn't write).

    Ok(())
  }
}
```

**MIRROR:** §10.6 (fire-branch governance_log payload), §10.8
(fixture mod shape). Adjacent test mod:
`crates/server/tests/e2e.rs:9786-...` (`mod v1_jm_e_fixtures`); or
post-SL-b: `mod v1_sl_b_fixtures` (the immediate predecessor).

**GOTCHA (R4):** test fn name lowercase snake_case.

**GOTCHA (BREHON_DISABLE_GRACE_CHECK_JOB):** test bootstrap sets
this env var before LemmyContext construction. Mirror SL-b test
infrastructure if present; fallback to inline `std::env::set_var`
in test body before bootstrap.

**GOTCHA (Watchpoint #5 — both kinds of log entries):** the fire
branch emits BOTH per-sponsor `sponsor_liability_applied` entries
AND the SL-c summary `sponsor_liability_fired` entry. Test asserts
both:
- `count(*) WHERE entry_kind = 'sponsor_liability_applied'` matches
  active sponsor count (2).
- `count(*) WHERE entry_kind = 'sponsor_liability_fired'` == 1.

**GOTCHA (governance_log queries in tests):** use diesel
`governance_log::table.filter(...)` per existing JM-c
`governance_log_hash_chain_holds` test pattern at
`crates/server/tests/e2e.rs:907`.

**GOTCHA (Watchpoint #1 — FOR UPDATE invariant):** test does NOT
need to assert the FOR UPDATE lock directly (that's a sub-tx
implementation detail); it asserts the OUTCOME — case correctly
transitions even if a hypothetical concurrent SL-b revocation were
running. This test is single-threaded; the FOR UPDATE assertion is
implicit via the structural correctness of the per-case tx body.

**GOTCHA (anchor-Edit discipline per `feedback_junior_worker_e2e_edit_hang.md`):**
this Task 1 ships the `mod v1_sl_c_fixtures` shell + 2 helpers + 1
test in ONE Edit at file end. Tasks 2-5 anchor-insert AFTER this
task's test body, INSIDE the same mod. Do NOT bulk-edit multiple
tests in one Edit.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-c-2): e2e test #1 — fire path expired pending case + mod v1_sl_c_fixtures shell (task 1)`

### Task 2: e2e test #2 — Escape path (sponsor revoked between decided_at and now)

**ACTION:** anchor-insert
`grace_check_escapes_case_when_sponsor_revoked_after_decided_at`
test fn inside `mod v1_sl_c_fixtures`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test #2 inside mod v1_sl_c_fixtures
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 1's test fn,
inside the same mod block (the closing `}` of the mod still wraps
this test). Carried verbatim from trunk plan §13 Task 4:

```rust
  #[tokio::test]
  async fn grace_check_escapes_case_when_sponsor_revoked_after_decided_at() -> Result<(), Box<dyn Error>> {
    // Per Test #2 (PRD §6.2 step 4 escape branch — "any sponsor
    // revoked since decided_at").
    //
    // Setup: BREHON_DISABLE_GRACE_CHECK_JOB=1.
    //   1 sponsee + 1 sponsor. Active surety initially.
    //   ModerationCase status=SponsorLiabilityPending,
    //     grace_expires_at = now() - 1 minute (expired),
    //     decided_at = now() - 24h.
    //   sanction row.
    //   THEN: UPDATE surety SET revoked_at = now() - 1h
    //     (revoked AFTER decided_at, BEFORE now()).
    //
    // Drive: run_grace_check_batch(&context).await.
    //
    // Assert:
    //   - outcome.escaped == 1, outcome.fired == 0.
    //   - case.status == SponsorLiabilityEscaped.
    //   - case.liability_escape_reason IS Some(json) where:
    //     * json["version"] == 1
    //     * json["reason"] == "sponsor_revoked"
    //     * json["actor_pseudonym"] is a String (NOT raw caller_id)
    //     * json["actor_pseudonym"] does NOT equal format!("{}", sponsor_id.0)
    //       (defensive ADR-015)
    //     * json["endorsement_id"] >= 0 (best-effort lookup; may be 0
    //       if no endorsement row was seeded — this test seeds one)
    //   - 0 reputation_event rows for the sponsor (escape branch
    //     does NOT call apply_sponsor_liability).
    //   - 1 governance_log row "sponsor_liability_escaped".
    //   - 0 governance_log rows "sponsor_liability_fired".
    //   - 0 governance_log rows "sponsor_liability_applied".
    //   - "sponsor_liability_escaped" payload matches the JSONB shape.

    Ok(())
  }
```

**MIRROR:** §10.6 escape JSONB schema; §4 watchpoint #4.

**GOTCHA (Watchpoint #4 — ADR-015 actor_pseudonym):** `actor_pseudonym`
field is a STRING (the pseudonym), NOT a number. Defensive
assertion: `assert_ne!(json["actor_pseudonym"].as_str().unwrap(),
&format!("{}", sponsor_id.0))`.

**GOTCHA (Watchpoint #5 — only escape entry, no fire-side entries):**
the escape branch does NOT call `apply_sponsor_liability`, so the
per-sponsor `sponsor_liability_applied` entries (and any
`sponsor_liability_clamped`) MUST be absent. Test asserts the count
== 0.

**GOTCHA (endorsement seeding for ref_id):** for the test's
`endorsement_id` field assertion, the test seeds an `endorsement`
row from sponsor → sponsee BEFORE seeding the surety, so
`evaluate_escape_conditions`'s endorsement-id lookup finds a value.

**GOTCHA (revoked_at semantic):** `surety.revoked_at = now() - 1h`
is AFTER `decided_at = now() - 24h` AND BEFORE `now()` — both bounds
required by `evaluate_escape_conditions`'s filter.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-c-2): e2e test #2 — escape branch sponsor revoked since decided_at (task 2)`

### Task 3: e2e test #3 — No-op (future grace_expires_at)

**ACTION:** anchor-insert
`grace_check_no_op_when_grace_expires_at_in_future` test fn inside
`mod v1_sl_c_fixtures`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test #3 inside mod v1_sl_c_fixtures
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 2's test fn.
Carried verbatim from trunk plan §13 Task 5:

```rust
  #[tokio::test]
  async fn grace_check_no_op_when_grace_expires_at_in_future() -> Result<(), Box<dyn Error>> {
    // Per Test #3 (PRD §6.1 batch-query filter — only cases past
    // grace_expires_at).
    //
    // Setup: BREHON_DISABLE_GRACE_CHECK_JOB=1.
    //   1 sponsee + 1 sponsor. Active surety.
    //   ModerationCase status=SponsorLiabilityPending,
    //     grace_expires_at = now() + 2 hours (NOT YET EXPIRED),
    //     decided_at = now() - 24h.
    //   sanction row.
    //
    // Drive: run_grace_check_batch(&context).await.
    //
    // Assert:
    //   - outcome.cases_processed == 0 (case not selected by batch query).
    //   - outcome.fired == 0, outcome.escaped == 0.
    //   - case.status STILL == SponsorLiabilityPending (unchanged).
    //   - case.liability_escape_reason IS STILL NULL.
    //   - 0 reputation_event rows for the sponsor.
    //   - 0 governance_log rows "sponsor_liability_fired".
    //   - 0 governance_log rows "sponsor_liability_escaped".
    //   - 0 governance_log rows "sponsor_liability_applied".

    Ok(())
  }
```

**MIRROR:** PRD §6.1 batch-query filter (`grace_expires_at.le(Some(now))`).

**GOTCHA (boundary semantic):** the batch query uses `.le(Some(now))`
(less-than-or-equal). A case with `grace_expires_at == now()` exactly
would fire; +2 hours is comfortably outside the window.

**GOTCHA (no spurious side effects):** the no-op assertion is the
strong signal — verifying that an unselected case has zero
side-effects across reputation_event, governance_log, and
moderation_case row mutation.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-c-2): e2e test #3 — no-op for future grace_expires_at (task 3)`

### Task 4: e2e test #4 — Per-case isolation (one bad case doesn't block batch)

**ACTION:** anchor-insert
`grace_check_per_case_isolation_skips_bad_case_processes_good_case`
test fn inside `mod v1_sl_c_fixtures`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test #4 inside mod v1_sl_c_fixtures
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 3's test fn.
Carried verbatim from trunk plan §13 Task 6:

```rust
  #[tokio::test]
  async fn grace_check_per_case_isolation_skips_bad_case_processes_good_case() -> Result<(), Box<dyn Error>> {
    // Per Test #4 (PRD §6.3 + §4 watchpoint #8 — per-case isolation
    // invariant).
    //
    // Setup: BREHON_DISABLE_GRACE_CHECK_JOB=1.
    //   2 sponsees: sponsee_a (well-formed) and sponsee_b (malformed).
    //   1 sponsor for sponsee_a. Active surety for sponsee_a.
    //   Case A: status=SponsorLiabilityPending, grace_expires_at expired,
    //     sanction inserted (well-formed).
    //   Case B: status=SponsorLiabilityPending, grace_expires_at expired,
    //     ZERO sanction rows (malformed — empty-sanction edge case).
    //     Force this by passing sanction_action: None to seed_pending_case.
    //
    // Drive: run_grace_check_batch(&context).await.
    //
    // Assert:
    //   - outcome.cases_processed == 2.
    //   - outcome.fired == 1 (case A).
    //   - outcome.skipped == 1 (case B — sanction lookup returned None).
    //   - outcome.escaped == 0.
    //   - case_a.status == SponsorLiabilityFired.
    //   - case_b.status == SponsorLiabilityPending (unchanged — silently skipped).
    //   - case_b.liability_escape_reason IS STILL NULL.
    //   - 1 reputation_event row for sponsor (case A's sponsor).
    //   - 1 governance_log row "sponsor_liability_fired" (case A only).
    //   - 1 governance_log row "sponsor_liability_applied" (case A's sponsor).
    //   - 0 governance_log rows for case B's id.
    //
    // Strong assertion: outer batch returned Ok(...) (test bootstraps
    //   with case_b ordered FIRST in the batch query, since
    //   grace_expires_at ASC sort is the iteration order. Seed case_b
    //   with grace_expires_at slightly earlier than case_a's, so case_b
    //   is processed first; case_b's tracing::error! + skip MUST NOT
    //   block case_a's later iteration).

    Ok(())
  }
```

**MIRROR:** §10.7 empty-sanction handling.

**GOTCHA (Watchpoint #8):** the strong assertion is "outer batch
returned Ok(...) AND iteration continued past case_b". Without the
per-case error-catch in the outer for-loop, case_b's empty-sanction
warn would have stopped the iteration. The test asserts case_a's
fire happened — proves continuation.

**GOTCHA (ordering — case_b first):** the batch query sorts by
`grace_expires_at ASC`, so seed case_b's expired_at slightly EARLIER
than case_a's (e.g. `now() - 2 minutes` vs `now() - 1 minute`).

**GOTCHA (silent skip semantic):** case_b stays Pending — admin must
intervene. Future ticks will also skip case_b. This is the design
semantic.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-c-2): e2e test #4 — per-case isolation skips bad case processes good case (task 4)`

### Task 5: e2e test #5 — Batch size respects config

**ACTION:** anchor-insert
`grace_check_batch_size_config_caps_iteration` test fn inside
`mod v1_sl_c_fixtures`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test #5 inside mod v1_sl_c_fixtures (closes the mod)
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 4's test fn.
This is the LAST test in `mod v1_sl_c_fixtures`; the closing `}` of
the mod follows this test. Carried verbatim from trunk plan §13
Task 7:

```rust
  #[tokio::test]
  async fn grace_check_batch_size_config_caps_iteration() -> Result<(), Box<dyn Error>> {
    // Per Test #5 (PRD §6.4 + §4.1 batch_size config).
    //
    // Setup: BREHON_DISABLE_GRACE_CHECK_JOB=1.
    //   UPSERT governance_config row "job.grace_check_batch_size" = 2
    //     (instance scope). Default is 100; we override to 2.
    //   5 sponsees + 5 sponsors (1 surety each). All 5 cases:
    //     status=SponsorLiabilityPending, grace_expires_at expired,
    //     sanction inserted.
    //
    // Drive (first invocation): run_grace_check_batch(&context).await.
    //
    // Assert (first invocation):
    //   - outcome.cases_processed == 2 (batch_size cap honoured).
    //   - outcome.fired == 2.
    //   - 2 cases transitioned to SponsorLiabilityFired.
    //   - 3 cases STILL == SponsorLiabilityPending.
    //
    // Drive (second invocation): run_grace_check_batch(&context).await.
    //
    // Assert (second invocation):
    //   - outcome.cases_processed == 2 (next 2 picked up).
    //   - outcome.fired == 2.
    //   - 4 cases now SponsorLiabilityFired total.
    //   - 1 case STILL == SponsorLiabilityPending.
    //
    // Drive (third invocation): run_grace_check_batch(&context).await.
    //
    // Assert (third invocation):
    //   - outcome.cases_processed == 1 (last remaining).
    //   - outcome.fired == 1.
    //   - all 5 cases now SponsorLiabilityFired.

    Ok(())
  }
```

**MIRROR:** PRD §6.4 batch_size knob.

**GOTCHA (governance_config UPSERT):** the test must insert/update
the `job.grace_check_batch_size` config row at the `Instance` scope.
Use the existing `governance_config` table directly via diesel:
```rust
insert_into(governance_config::table)
  .values((
    governance_config::scope.eq("instance"),
    governance_config::key.eq("job.grace_check_batch_size"),
    governance_config::value.eq("2"),
  ))
  .on_conflict((governance_config::scope, governance_config::key))
  .do_update()
  .set(governance_config::value.eq("2"))
  .execute(conn)
  .await?;
```

**GOTCHA (sort-order determinism):** the batch query's `ORDER BY
grace_expires_at ASC` means the 5 cases must have DISTINCT
`grace_expires_at` values for predictable ordering. Seed each with
`grace_expires_at = now() - Duration::minutes(N)` for N in 5..0.

**GOTCHA (config caching):** the outer ConfigCache in
`run_grace_check_batch` is fresh per call; the second/third
invocations see the updated config row.

**GOTCHA (closing the mod):** this test is the LAST inside `mod
v1_sl_c_fixtures`. The closing `}` of the mod follows immediately
after this test's `Ok(())` line. Future SL-d/SL-e fixture mods open
AFTER `mod v1_sl_c_fixtures` closes.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-c-2): e2e test #5 — batch_size config caps iteration (task 5)`

### Task 6: Retro

**Goal:** author retro per `feedback_retro_not_report.md` +
`feedback_four_role_retro_signals.md` +
`feedback_retro_task_complexity_score.md`. **Flip
`(pending) → (active)` markers** on registry §"v1-SL-a entry kinds"
for SL-c's fire-sites: `_SPONSOR_LIABILITY_FIRED` row entirely;
`_SPONSOR_LIABILITY_ESCAPED` row's SL-c portion (the SL-b portion
was flipped by SL-b retro). c-2's e2e tests have now behaviourally
exercised both fire-sites end-to-end per
`feedback_build_what_tests_exercise.md`.

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-SL-c-2-retro.md
modifies:
  - .claude/rules/governance-log-entry-kind-registry.md
```

**IMPLEMENT (file 1 of 2):** in
`.claude/PRPs/reports/v1-SL-c-2-retro.md`, write the retro per the
canonical 4-role format (Advisor / Planning / Impl / BM). Include:

- §1 Summary: c-2 shipped (5 e2e tests + retro). Story status —
  Stories 2 + 3 ✓. Combined with c-1's Story 1 ✓, the trunk SL-c
  deliverable is fully shipped.
- §2 Per-role signals (4 H2 sections; each lists "what worked" +
  "what surprised us" + "what should change next"). Cross-PR
  carry-forward signals from c-1 → c-2 (e.g. did the c-1 module
  imports resolve cleanly at c-2 Task 1 anchor-Edit time? did
  Probe 7 catch any c-1 ship gaps?).
- §3 Carry-forward — list of items SL-d/SL-e/restorative-mechanics-v1
  should know.
- §4 Per-task complexity-score table (mandatory per
  `feedback_retro_task_complexity_score.md`):
  `| task | files-changed | commits | runtime-min | max-log-silence-min |`
  for each of Tasks 0..6.
- §5 Lessons promotion — any new `feedback_*.md` candidates
  (especially: c-1 / c-2 split-mode lessons; the post-c-1-merge
  registry-flip discipline as a generalisable PMD entry; cross-PR
  carry-forward via Probe 7 module-presence checks).
- §6 Acceptance — confirm all checkboxes from §17.

**IMPLEMENT (file 2 of 2):** in
`.claude/rules/governance-log-entry-kind-registry.md`, locate the
"v1-SL-a entry kinds" section. Edit the "Emitting handler" column
for two rows:

- `ENTRY_KIND_SPONSOR_LIABILITY_FIRED` (row at line 172): change
  "v1-SL-c
  `crates/api/api/src/governance/sponsor_liability_grace.rs::run_grace_check_batch`
  fire branch (pending)" to "(active)" — flipped 2026-MM-DD post-c-2.
- `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` (row at line 173): the SL-c
  portion of "v1-SL-b ... AND v1-SL-c
  `sponsor_liability_grace.rs::evaluate_escape_conditions` (pending)"
  → "(active)". (The SL-b portion was already flipped by SL-b retro.)

Total const count stays unchanged.

**Cross-cutting verification (Task 6 retro time — invariants):**

- `rg -c '^pub const ENTRY_KIND_'
  crates/db_schema/src/source/governance/governance_log.rs` returns
  the same total as post-SL-b state (unchanged).
- `rg '^\s+ENTRY_KIND_'
  crates/api/api/src/governance/governance_log.rs | wc -l` returns
  the same total (unchanged shim re-exports).
- `rg -n '\(pending\)'
  .claude/rules/governance-log-entry-kind-registry.md | rg
  SPONSOR_LIABILITY_FIRED` returns empty (marker fully flipped).
- `rg -n '\(pending\)'
  .claude/rules/governance-log-entry-kind-registry.md | rg
  SPONSOR_LIABILITY_ESCAPED` returns empty (BOTH portions flipped:
  SL-b's by SL-b retro, SL-c's by this retro).
- `rg -n 'mod v1_sl_c_fixtures' crates/server/tests/e2e.rs` returns
  1 line.
- `rg -c '#\[tokio::test\]\s*async fn grace_check_'
  crates/server/tests/e2e.rs` returns 5 (SL-c test count).
- `rg -n 'pub async fn run_grace_check_batch|pub async fn evaluate_escape_conditions|pub async fn fire_or_escape_case|pub async fn check_grace_staleness'
  crates/api/api/src/governance/sponsor_liability_grace.rs` returns
  4 lines (c-1 module unchanged by c-2).
- `rg -n 'SPONSOR_LIABILITY_GRACE_RUNNING'
  crates/routes/src/utils/scheduled_tasks.rs` returns at least 3
  lines (c-1 atomic guard unchanged by c-2).
- `cargo build` workspace exit 0 via `cargo-validate-workspace.yml`
  on the phase-branch tip.
- `/brehon-verify` reports Stories 2 + 3 `[done]`.
- `git diff governance-v0..phase-v1-SL-c-2 -- migrations/` returns
  empty (zero migrations).
- `git diff governance-v0..phase-v1-SL-c-2 -- crates/api/`
  returns empty (zero c-1-module modifications).
- `git diff governance-v0..phase-v1-SL-c-2 -- crates/routes/`
  returns empty (zero scheduled_tasks.rs modifications).

**MIRROR:** `.claude/PRPs/reports/v1-SL-c-1-retro.md` (sibling;
expected to exist post-c-1 ship); `.claude/PRPs/reports/v1-SL-b-retro.md`;
`.claude/PRPs/reports/v1-JM-e-retro.md` for section structure.

**GOTCHA (retro before PR):** retro is written BEFORE `gh pr
create` per `feedback_retro_not_report.md`.

**GOTCHA:** §4 per-task complexity-score table is mandatory.

**GOTCHA (registry rule — both portions of `_ESCAPED`):** the
registry shows `_ESCAPED` with TWO emitting handlers (SL-b
revoke_endorsement AND SL-c sponsor_liability_grace). c-2 retro
flips ONLY the SL-c portion. SL-b retro should already have flipped
the SL-b portion. If SL-b retro missed it (e.g. ran before SL-b PR
#119 merged), the c-2 retro flips BOTH portions and a §5 retro
note flags the SL-b retro miss.

**GOTCHA (cross-PR registry-flip discipline):** this is the c-2
deferral cited in c-1 retro §3 carry-forward (per
`feedback_build_what_tests_exercise.md`). c-2 retro discharges
that deferral.

**Push and exit (Shape G — retro is meta-work; no `crates/**`
change → `cargo-validate-workspace` does not trigger).**

**COMMIT MESSAGE:** `docs(v1-SL-c-2): phase retrospective + flip ENTRY_KIND_* registry markers (task 6)`

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** `cargo check --workspace --features full`
  via `cargo-validate-workspace.yml:88`.
- **Lint:** `cargo clippy --workspace --features full --no-deps --
  -D warnings` via `cargo-validate-workspace.yml:92` (R6).
- **Test target compile (R7):** `cargo test --no-run -p lemmy_server
  --test e2e` via `cargo-validate-workspace.yml:95`. Triggers on
  Tasks 1-5 (e2e file edits).
- **Migration round-trip:** N/A — c-2 ships zero migrations.
- **e2e execution (Phase 2):** the 5 new tests run via Phase 2 e2e
  user gate: (a) local on laptop or (b) GH dispatch via
  `cargo-test-e2e.yml`. **This is the c-2 acceptance gate.**

Pre-merge advisor-side verification: Stories 2 + 3 checkpoints are
Phase 2 e2e exit 0 on the post-finalize-merge phase-branch tip;
`/brehon-verify` Brief-Scope outputs check.

### 14.1 Pre-existing tests preserved

- All c-1-shipped infrastructure (no tests, but module code) — c-2
  imports + invokes; preserves the c-1 module unchanged
- All SL-b-shipped tests (`mod v1_sl_b_fixtures::*`, 9-10 tests
  post-PR #119) — preserved verbatim
- All SL-a-shipped tests — preserved verbatim
- All JM-e-shipped tests (`mod v1_jm_e_fixtures::*`) — preserved
- All JM-c/JM-b-shipped tests — preserved
- `governance_log_hash_chain_holds` at e2e.rs:907 — preserved
- All endorsement-creation tests (Phase 5b) — preserved
- All snapshot-job tests (`reputation_snapshot::run_snapshot_batch`
  e2e exercises) — preserved
- All appeal-window-expiry-job tests
  (`appeal_window_expiry::run_appeal_window_expiry_batch` e2e
  exercises) — preserved

### 14.2 Edge cases covered

- Fire path: expired pending case, active sureties → status flips,
  per-sponsor reputation_event rows + per-sponsor `applied` entries
  + summary `fired` entry (Test #1)
- Escape path: sponsor revocation between decided_at and now →
  status flips, `liability_escape_reason` JSONB written, no
  reputation_event rows (Test #2)
- Boundary: future grace_expires_at → no-op, no side effects
  (Test #3)
- Per-case isolation: empty-sanction case + well-formed case both in
  batch → bad case skipped silently, good case fires, outer batch
  returns Ok (Test #4)
- Config-driven batch size: 5 cases, batch_size=2 → 3 ticks needed;
  each tick respects cap (Test #5)

### 14.3 Edge cases NOT covered (out of v1-SL-c-2 scope)

- Restoration-completed escape branch — stub-only per DQ #145; will
  be tested by restorative-mechanics-v1 PRD's plan when the producer
  endpoint ships.
- Multi-sponsor `all_revocation` / `majority_revocation` rules at
  scheduler-tick time — SL-b's `revoke_endorsement` handles the
  multi-sponsor escape at revocation time; the case wouldn't reach
  c-1's scheduler unless still Pending.
- Concurrent SL-b + c-1 on same case (race) — defended by
  watchpoint #1 (FOR UPDATE) + watchpoint #2 (status re-check); no
  e2e test for the race itself.
- Scheduler tick interval respect (`every(5).run(...)`) — tested by
  the existing snapshot/appeal-window job tests' scheduler-tick
  pattern.
- Staleness threshold semantic — covered by no e2e test in c-2
  (pure tracing emit; structural assertion via the function body's
  shape).
- Federation outbound on `sponsor_liability_fired` /
  `sponsor_liability_escaped` — out per ADR-014 (v0 deferral).
- v0 mid-flight backfill behaviour — SL-a Task 1 backfill
  populates the cases c-1 picks up; SL-a's tests verify backfill;
  c-2's tests pre-seed via `ModerationCaseInsertForm` (synthetic
  equivalent).

---

## 15. Validation commands (DoD)

> **Shape G plan — DoD is per-workflow, not inline cargo.** Per
> `.claude/PRPs/templates/plan.template.md` §15.6. c-2 ships zero
> migrations, so `cargo-validate-migration.yml` does not fire.

### 15.1 Per-task workspace check (Shape G)

For every §13 impl task that modifies `crates/**` (Tasks 1, 2, 3, 4,
5):

- **DoD entry:** `cargo-validate-workspace.yml` on `junior/<task-slug>`
  SHA `<sha>` → `conclusion: "success"`
- **Validation command:** `gh run list --repo barrie-cork/lemmy
  --branch <branch> --workflow cargo-validate-workspace --limit 1
  --json conclusion,databaseId --jq '.[0]'`
- **EXPECT:** `{"conclusion": "success", "databaseId": <id>}`

The workflow runs `cargo check --workspace --features full`,
`cargo clippy --workspace --features full --no-deps -- -D warnings`,
and `cargo test --no-run -p lemmy_server --test e2e` per
`.github/workflows/cargo-validate-workspace.yml:88-95`. R6 + R7 are
encoded.

### 15.2 Migration round-trip — N/A

c-2 ships zero migrations; `cargo-validate-migration.yml` path
filter `migrations/**` excludes c-2 commits. No DoD entry.

### 15.3 Phase 2 e2e (post-finalize-merge of last impl task)

After Task 5's worker branch finalize-merges into `phase-v1-SL-c-2`,
the advisor surfaces the **Phase 2 e2e local-vs-dispatch user gate**
per `advisor-orchestrator.md`:

- **(a) local:** `cargo test -p lemmy_server --test e2e --features
  full -- --test-threads=1` on laptop in `run_in_background`; ~26 min
  wall-clock; zero billed.
- **(b) dispatch:** `gh workflow run cargo-test-e2e.yml --repo
  barrie-cork/lemmy --ref phase-v1-SL-c-2`; ci-watcher polls; ~26 min
  billed.

Plan-side DoD: e2e exit code 0 with all 5 c-2 tests passing;
failure path → §G4 classifier on log slice.

### 15.4 Cross-cutting verification (Task 6 retro time)

- [ ] `rg -c '^pub const ENTRY_KIND_'
  crates/db_schema/src/source/governance/governance_log.rs` returns
  the same total as post-SL-b state (unchanged).
- [ ] `rg '^\s+ENTRY_KIND_'
  crates/api/api/src/governance/governance_log.rs | wc -l` returns
  the same total (unchanged shim re-exports).
- [ ] `rg -n '\(pending\)'
  .claude/rules/governance-log-entry-kind-registry.md | rg
  SPONSOR_LIABILITY_FIRED` returns empty.
- [ ] `rg -n '\(pending\)'
  .claude/rules/governance-log-entry-kind-registry.md | rg
  SPONSOR_LIABILITY_ESCAPED` returns empty.
- [ ] `rg -n 'pub mod sponsor_liability_grace;'
  crates/api/api/src/governance/mod.rs` returns 1 line (c-1
  unchanged by c-2).
- [ ] `rg -n 'SPONSOR_LIABILITY_GRACE_RUNNING'
  crates/routes/src/utils/scheduled_tasks.rs` returns at least 3
  lines (c-1 unchanged).
- [ ] `rg -n '"BREHON_DISABLE_GRACE_CHECK_JOB"'
  crates/routes/src/utils/scheduled_tasks.rs` returns 1 line (c-1
  unchanged).
- [ ] `rg -n 'pub async fn run_grace_check_batch|pub async fn evaluate_escape_conditions|pub async fn fire_or_escape_case|pub async fn check_grace_staleness'
  crates/api/api/src/governance/sponsor_liability_grace.rs` returns
  4 lines (c-1 unchanged).
- [ ] `rg -n 'mod v1_sl_c_fixtures'
  crates/server/tests/e2e.rs` returns 1 line.
- [ ] `rg -c '#\[tokio::test\]\s*async fn grace_check_'
  crates/server/tests/e2e.rs` returns 5.
- [ ] R1: every `i32 ↔ i64` comparison in Tasks 1-5 uses
  `i64::from(...)` if cross-type comparison arises.
- [ ] R6: clippy invocations in `cargo-validate-workspace.yml` use
  `--no-deps -- -D warnings`.
- [ ] No edits to files outside §11 list.
- [ ] Every c-2 §16a story (2, 3) is `[done]`.
- [ ] No new ENTRY_KIND_*** consts under
  `crates/db_schema/src/source/governance/governance_log.rs`.
- [ ] No new migrations: `git diff governance-v0..phase-v1-SL-c-2
  -- migrations/` returns empty.
- [ ] No c-1 module modifications: `git diff
  governance-v0..phase-v1-SL-c-2 -- crates/api/` returns empty.
- [ ] No scheduled_tasks.rs modifications: `git diff
  governance-v0..phase-v1-SL-c-2 -- crates/routes/` returns empty.

### 15.5 ADR / OQ compliance verification

- [ ] **ADR-005** honoured — fire branch's
  `apply_sponsor_liability` writes per-dimension reputation_event
  rows. (Behaviourally validated by Test #1.)
- [ ] **ADR-008** honoured — every emit goes through
  `governance_log::append`; no direct INSERT to `governance_log`
  table.
- [ ] **ADR-010** honoured — won't-disadvantage rule preserved;
  c-2's pre-seed pattern is semantically equivalent to SL-a's
  mid-flight backfill.
- [ ] **ADR-013** honoured — exhaustive `EscapeStatus` match in
  c-1's `fire_or_escape_case_inner` enumerates `Escape{...}` AND
  `Fire`; no `_ =>` arm. c-2 tests assert against named variants.
- [ ] **ADR-014** honoured — no federation outbound on
  `sponsor_liability_fired` or `sponsor_liability_escaped`. (c-2
  tests do NOT exercise federation.)
- [ ] **ADR-015** honoured —
  `liability_escape_reason` JSONB carries `actor_pseudonym` (string),
  NOT raw `caller_id`. (Behaviourally validated by Test #2 defensive
  assertion.)
- [ ] **OQ-V1-SL-05** honoured — `liability_escape_reason` JSON
  carries `version: 1` from day one. (Validated by Test #2.)
- [ ] **PRD §6** honoured — scheduler tick + per-case isolation +
  staleness check all behaviourally validated by c-2 tests + c-1
  shipped code.
- [ ] **PRD §2 OUT** honoured — no `apply_sponsor_liability`
  compute/fire split, no `submit_jury_vote` mutation, no
  `restoration_complete` endpoint, no notification UX, no
  cross-instance federation.
- [ ] **PRD §12.3 v2 reservation** honoured — no step-up auth on
  scheduler.

### 15.6 DoD per workflow (canonical Shape G shape)

**Phase 1 (workspace check):**

- Workflow: `.github/workflows/cargo-validate-workspace.yml`
- Branch (per task): `junior/<task-slug>`
- Expected `conclusion`: `"success"`

**Phase 1b (migration round-trip):** N/A — no migrations in c-2.

**Phase 2 (e2e):**

- Workflow: `.github/workflows/cargo-test-e2e.yml` (only on user
  dispatch per PR #105)
- OR local: `cargo test -p lemmy_server --test e2e --features full
  -- --test-threads=1` on laptop
- Branch: `phase-v1-SL-c-2`
- Expected: all tests pass

### 15.7 Manual validation snippets (advisor-side smoke only — non-binding)

> Per `feedback_features_full_p_crate_incompatible.md`: never `-p
> <crate>` + `--features full`; use `--workspace --features full`.

> Per `feedback_e2e_filter_assumes_naming.md`: c-2 tests all start
> with `grace_check_` — confirm via grep before filter.

```bash
ls .github/workflows/cargo-validate-workspace.yml
ls .github/workflows/cargo-test-e2e.yml

gh run list --repo barrie-cork/lemmy \
  --branch phase-v1-SL-c-2 \
  --workflow cargo-validate-workspace \
  --limit 1 --json conclusion,databaseId

cargo check --workspace --features full > /tmp/sl-c-2-check.log 2>&1
status=$?
tail -20 /tmp/sl-c-2-check.log
echo "exit: $status"

rg '#\[tokio::test\]\s*async fn grace_check_' crates/server/tests/e2e.rs | wc -l
# EXPECT: 5

rg -n 'pub async fn run_grace_check_batch' crates/api/api/src/governance/sponsor_liability_grace.rs
# EXPECT: 1 match (c-1 module unchanged by c-2)
```

These are advisor-side only; not §16 acceptance criteria.

---

## 16. Acceptance criteria

- [ ] All 7 tasks (Task 0..5 + Task 6 retro) committed.
- [ ] §15.1 (`cargo-validate-workspace.yml`) `conclusion: "success"`
  after every impl task push (Tasks 1-5).
- [ ] §15.2 (migration round-trip) — N/A (zero migrations).
- [ ] §15.3 (Phase 2 e2e — local or dispatch) all tests pass; the 5
  new `v1_sl_c_fixtures::grace_check_*` tests green.
- [ ] §15.4 (cross-cutting verification — 16 boxes) all ticked.
- [ ] §15.5 (ADR / OQ compliance — 10 boxes) all ticked.
- [ ] §16a Stories 2 + 3 — both `[done]`.
- [ ] No edits to files outside §11 list (notably: zero c-1 module
  modifications).
- [ ] Retro committed per Task 6.
- [ ] PR opens against `governance-v0` (NOT `main`) with
  `--repo barrie-cork/lemmy`.
- [ ] `/brehon-verify` report at
  `.claude/PRPs/reports/v1-SL-c-2-verify.md` shows Stories 2 + 3 ✓.
- [ ] DQ #150 (parent split) cited in plan §2; DQ #151 (this plan's
  split-or-proceed) self-resolved by planner with proceed
  rationale at plan-write time (Recipe 2 self-resolved per
  `decision-queue.md`).
- [ ] Registry markers flipped (`_SPONSOR_LIABILITY_FIRED` row
  entirely; `_SPONSOR_LIABILITY_ESCAPED` row's SL-c portion).

---

## 16a. Stories (independently-testable behaviour units)

Per `.claude/PRPs/templates/plan.template.md` §16a +
`feedback_story_grain_checkpoint.md` +
`feedback_brehon_verify_pre_merge.md`. **c-2 ships TWO stories**
(Stories 2 + 3 from the trunk plan; Story 1 shipped in c-1).

### Story 1 — N/A for c-2 (shipped in c-1)

c-1 deliverable: Module + scheduler wiring + atomic guard pair
compile clean. c-1's verify report at
`.claude/PRPs/reports/v1-SL-c-1-verify.md` confirms Story 1
`[done]`.

### Story 2: Fire branch transitions case to `SponsorLiabilityFired` with both per-sponsor and summary log entries + writes `reputation_event` rows

- **Composing tasks:** Task 1
- **Checkpoint workflow (Phase 1):** `cargo-validate-workspace.yml`
  on Task 1's worker branch SHA → `conclusion: "success"`.
- **Checkpoint workflow (Phase 2):** Phase 2 e2e for
  `grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries`
  → pass.
- **Expected output (local Phase 2):** `1 passed; 0 failed`.
- **Brief-Scope outputs to verify:**
  - `crates/server/tests/e2e.rs` contains `mod v1_sl_c_fixtures`
    block.
  - `crates/server/tests/e2e.rs` contains
    `async fn grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries(`.
  - `crates/server/tests/e2e.rs` contains
    `async fn seed_pending_case(` helper inside the new mod.
  - `crates/server/tests/e2e.rs` contains
    `async fn seed_active_surety(` helper inside the new mod.
  - Test body asserts BOTH `entry_kind = "sponsor_liability_fired"`
    (count == 1) AND `entry_kind = "sponsor_liability_applied"`
    (count == sponsor_count == 2) (verifiable via `rg`).
  - Test body asserts `case.status == CaseStatus::SponsorLiabilityFired`.

### Story 3: Escape branch + per-case isolation + batch-size config + future-grace no-op behave correctly

- **Composing tasks:** Tasks 2, 3, 4, 5
- **Checkpoint workflow (Phase 1):** `cargo-validate-workspace.yml`
  on Task 5's worker branch SHA → `conclusion: "success"`.
- **Checkpoint workflow (Phase 2):** Phase 2 e2e for the 4 tests →
  all pass.
- **Expected output (local Phase 2):** `4 passed; 0 failed`.
- **Brief-Scope outputs to verify:**
  - `crates/server/tests/e2e.rs` contains
    `async fn grace_check_escapes_case_when_sponsor_revoked_after_decided_at(`.
  - `crates/server/tests/e2e.rs` contains
    `async fn grace_check_no_op_when_grace_expires_at_in_future(`.
  - `crates/server/tests/e2e.rs` contains
    `async fn grace_check_per_case_isolation_skips_bad_case_processes_good_case(`.
  - `crates/server/tests/e2e.rs` contains
    `async fn grace_check_batch_size_config_caps_iteration(`.
  - Task 2 test body asserts `case.status ==
    CaseStatus::SponsorLiabilityEscaped` AND
    `case.liability_escape_reason["version"] == 1` AND
    `case.liability_escape_reason["reason"] == "sponsor_revoked"`
    AND `case.liability_escape_reason["actor_pseudonym"].is_string()`
    AND defensive ADR-015 assertion (actor_pseudonym does NOT equal
    `format!("{}", sponsor_id.0)`).
  - Task 3 test body asserts `outcome.cases_processed == 0` AND
    case status STILL Pending.
  - Task 4 test body asserts `outcome.fired == 1 && outcome.skipped
    == 1` AND case_a fired AND case_b still Pending (per-case
    isolation).
  - Task 5 test body asserts three iterations process 2 + 2 + 1
    cases respectively (batch_size cap).

> **Verification mapping:** `/brehon-verify` iterates this section,
> runs each Story's checkpoint workflow on the worktree branch, and
> confirms each Brief-Scope output exists + matches its structural
> pattern.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0..18 confirmed; Probe 7 c-1
  module presence is critical).
- [ ] Tasks 1..5 committed.
- [ ] Task 6 retro committed.
- [ ] §15 validation green at every gate (Phase 1 workspace check
  ×5, Phase 2 e2e ×1).
- [ ] §16a Stories 2 + 3 both `[done]`.
- [ ] PR opened by BM session against `governance-v0`.
- [ ] CodeRabbit review complete with findings triaged per
  `feedback_pr_review_triage_pattern.md`.
- [ ] `/brehon-verify` report at
  `.claude/PRPs/reports/v1-SL-c-2-verify.md` shows Stories 2 + 3 ✓.
- [ ] Post-merge phase branch retained for retro reads.
- [ ] DQ #151 (split-or-proceed) self-resolved by planner with
  proceed rationale at plan-write time.
- [ ] Registry markers flipped (`_SPONSOR_LIABILITY_FIRED` row
  entirely; `_SPONSOR_LIABILITY_ESCAPED` row's SL-c portion).

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Complexity score 17 prompts advisor to mandate further split | LOW | LOW | §5.2 — planner self-resolved DQ #151 with proceed rationale; further splitting yields no meaningful score reduction (e2e factor dominates). DQ #150 split-level is precedent; the user's split preference was for c-1 vs c-2, not for further fragmentation |
| Junior worker-hang on e2e.rs Edits (file is 10976+ lines, growing post-SL-b) | MED | HIGH | Per `feedback_junior_worker_e2e_edit_hang.md`: each test is its own §13 task; per-task anchor-Edit appends inside `mod v1_sl_c_fixtures`; no bulk Edit. SL-b shipped 9-10 e2e tests as 9-10 tasks cleanly at 10976-line file size |
| c-1 not yet merged at c-2 cut time | LOW | HIGH | BM-task does NOT cut `phase-v1-SL-c-2` until `c-1` PR merges to `governance-v0`. Probe 7 in Task 0 catches if BM-task mistakenly cuts early |
| c-1 module signature drift between c-1 ship and c-2 plan-implement | LOW | MED | Probe 7b verifies the 4 expected pub fns exist; if drift, Task 0 STOP and impl-task files DQ before Task 1 |
| SL-b not yet merged at c-2 plan-implement time | LOW | LOW | Probe 10 detects (`test -f revoke_endorsement.rs`); Task 1's anchor falls back to `mod v1_jm_e_fixtures` end if SL-b unmerged |
| Sanction multiplicity drift (1:1 → many-to-one in future SL-d/SL-e refactor) | LOW | LOW | Citation-only (c-1's lookup is defensive); c-2 tests pre-seed sanction rows so multiplicity is controlled per test |
| `liability_escape_reason` JSONB schema regression (raw `caller_id` instead of `actor_pseudonym`) | LOW | HIGH | Test #2 defensive ADR-015 assertion catches |
| TOCTOU on case status (concurrent SL-b revocation between batch query and per-case tx) | LOW | HIGH | c-1 ships the FOR UPDATE; c-2 tests assert outcome (no e2e test for the race itself — single-threaded test harness) |
| Per-case error propagation breaks isolation (one bad case blocks batch) | LOW | HIGH | Test #4 strong assertion catches |
| `BREHON_DISABLE_GRACE_CHECK_JOB` env var leaks atomic-bool slot if order reversed | LOW | MED | c-1 binding (already shipped); c-2 tests rely on the env-var override working correctly |
| Restoration-stub branch accidentally fires (despite DQ #145 advisor lock) | LOW | HIGH | c-1 binding; c-2 ships no test that exercises restoration (per `feedback_build_what_tests_exercise.md`) |
| Staleness threshold formula error (h vs s confusion) | LOW | LOW | c-1 binding; c-2 ships no staleness e2e test |
| ts-rs derive regression (export breaks downstream client codegen) | LOW | LOW | c-2 adds zero ts-rs derives |
| GH-Actions minutes budget exceeded by 5 Phase-1 + 1 Phase-2 e2e | LOW | LOW | Phase 2 e2e local-default per JM-d retro §5; user picks dispatch only on audit-trail need. Total ~50 min monthly burn well within Free 3000 min |
| PMD #126 — DQ ID collision from concurrent c-2 / rep-tuning-r3 work | LOW | LOW | next-id calc spans archives + live; advisor monitors at finalize-merge. c-2 plan-write computed next_id = 151 against live `decision-queue.json` at plan-write time (DQ #150 was the most recent advisor entry) |
| Junior worker pre-pushes break finalize-merge | LOW | LOW | Junior daemon's finalize step is post-Shape-G correct (per JM-d retro §1.1) |
| Test pre-seed `SponsorLiabilityPending` cases via direct DB-write inadvertently break SL-b's tests | LOW | LOW | Tests insert NEW rows via `ModerationCaseInsertForm`; no UPDATE on existing rows. Each test bootstraps a fresh Postgres container per testcontainers-rs |
| ADR-013 enum-exhaustiveness lint trips on a new match site c-2 accidentally introduces | LOW | LOW | c-2 tests assert on named variants (`CaseStatus::SponsorLiabilityFired`, etc); no new `match` sites introduced by c-2 |
| `governance_config` UPSERT in Test #5 interacts with config cache staleness across batch invocations | LOW | LOW | The outer ConfigCache in `run_grace_check_batch` is fresh per call (`ConfigCache::new()` at function entry). Second/third invocations see updated config |
| clokwerk schedule-pinning means batch_size config flip mid-run doesn't take effect until next tick | LOW | LOW | The batch_size is read PER-TICK (inside the closure body via `config::get_int`), not at scheduler setup. Tested via Test #5 (config flipped before invocation) |
| **Cross-PR carry-forward miss between c-1 retro and c-2 retro** (e.g. c-1 retro flagged a c-2 expectation that c-2 doesn't honour) | MED | LOW | c-2 Task 0 reads c-1 retro at `.claude/PRPs/reports/v1-SL-c-1-retro.md`; specifically §3 carry-forward + §5 lessons. c-2's Task 6 retro §2 cross-PR carry-forward subsection records anything that drifted |
| **Phase 2 e2e regression on existing tests** (c-2's edits affect e2e.rs imports / fixture-mod placement) | LOW | MED | c-2's anchor-Edit is at file end inside a NEW mod; existing fixture mods are untouched. Phase 1 workspace check's `cargo test --no-run` step validates compile of all existing tests against c-2's new symbols |

---

## 19. Notes

### 19.1 Planner DQs filed (c-2)

- **DQ #151** (`from: "planner"`, `kind: "blocker"`, `answered_by:
  "planner"` self-resolved per Recipe 2) — c-2 complexity-score
  split decision per §5.2. Question: "Complexity score 17 exceeds
  8 — split `v1-sponsor-liability-c-2` further (e.g. c-2-A Tasks
  1-2 + c-2-B Tasks 3-5), or proceed?". Options: split-further /
  proceed. Planner self-resolved with **proceed** rationale: e2e
  factor dominates (+15 of +16 wrapped subtotal); further splitting
  does NOT bring per-phase score below 8 because the +3-per-test
  weight applies to each sub-phase; SL-b at 38 + JM-e at 15 + SL-a
  at 13 + trunk-SL-c at 21 all shipped proceed-as-one without
  operational regret; per `feedback_principles_not_rules.md`, the
  e2e suite is anchor-Edit-friendly and the trunk plan §10.8
  fixture-mod shape is well-rehearsed. Advisor may overturn at
  plan-approval time if user prefers further split — but the
  e2e-factor analysis suggests further splitting yields no
  meaningful score reduction.

### 19.2 Self-resolved planner findings (LESSON candidates — c-2)

- **The c-2 split inherits the trunk plan's e2e-factor dominance
  pattern exactly.** Trunk plan §19.2 raised "5-test discipline
  inflates §5 e2e factor count proportionally to test count".
  c-2 confirms: c-2's 17 (1 crate + 5 e2e + adj 1 = 5 test count
  cleanly maps to 17). The complexity-score gate becomes a
  near-certain "trip" for any test-heavy phase. Per
  `feedback_principles_not_rules.md`, accept that the gate's purpose
  is to surface "split-or-proceed" as an explicit decision —
  test-heavy phases routinely answer proceed. Trunk-SL-c §19.2 +
  SL-b §19.2 + c-2 §19.2 all confirm; **promote to PMD lesson
  candidate**: "complexity-score gate routinely trips for
  test-heavy phases; proceed-as-one is the standard answer when
  e2e factor dominates".
- **Cross-PR carry-forward via Probe 7 (module-presence check) is
  the load-bearing c-2 / c-1 hand-off discipline.** Without
  Probe 7, c-2 risks being branched off a stale tip where c-1's
  module hasn't merged. Per
  `feedback_handover_trailer_cohort_propagation.md` (cross-cohort
  handover), the cross-PR analogue is the Probe-7 module-presence
  check + the c-1 retro §3 carry-forward + the c-2 Task 0 audit
  reading c-1's retro. **Promote to PMD lesson candidate**:
  "cross-PR cohort dispatch (split sub-phases) requires explicit
  Probe-7 module-presence check + retro §3 carry-forward
  discipline".
- **Registry-marker flip deferral (`feedback_build_what_tests_exercise.md`)
  works cleanly across split phases.** c-1 shipped code without
  flipping; c-2 ships tests + flip together. The deferral is
  explicit (c-1 retro §3 names it; c-2 Task 6 discharges it). This
  is the first in-fork instance of the deferral applied across a
  split sub-phase boundary.

### 19.3 Restoration-escape stub-and-future-wire breadcrumb

(Inherited verbatim from trunk plan §19.3 — c-1 ships the stub; c-2
ships zero tests for the stubbed branch.)

### 19.4 Pre-existing pending DQ entries

- **None at planning time.** `decision-queue.json` shows 0 pending
  entries on governance-v0 @ `abcf778e1`. Recently resolved:
  DQ #144-#147 (SL-c clarify pass) + DQ #148 (proceed-as-one,
  superseded by DQ #150) + DQ #149 (baseline_sponsor_count) +
  DQ #150 (split override).
- **DQ #151 filed by this plan** (split-or-proceed; planner-resolved
  with proceed rationale).

### 19.5 Out-of-scope follow-ups (Task 6 retro candidates)

- **`apply_sponsor_liability` compute/fire split** — SL-d.
- **`submit_jury_vote` mutation: `Decided →
  SponsorLiabilityPending` transition** — SL-d.
- **`restoration_complete` endpoint + restoration-escape branch
  detection** — restorative-mechanics-v1 PRD.
- **Lane-wide e2e suite** (full Decided → Pending → Fired/Escaped
  through SL-d transition + c-1 scheduler) — SL-e.
- **Sponsor notification UX** — PRD §13 OQ-V1-SL-03.
- **Step-up auth for admin-driven scheduler runs** — v2.
- **Cross-instance federation of grace-window events** — v2 per
  ADR-014.
- **Admin dashboard surface for staleness alerts** — admin-dashboard-v1.

### 19.6 Confidence bands (c-2)

- **High (9/10):** test fixture mod shape — exact mirror of
  `mod v1_sl_b_fixtures` (post-SL-b) or `mod v1_jm_e_fixtures`
  (fallback). The trunk plan §10.8 verbatim helper bodies.
- **High (9/10):** test-per-task discipline (5 tests = 5 tasks)
  per `feedback_junior_worker_e2e_edit_hang.md`. SL-b precedent
  9-10 tests as 9-10 tasks cleanly.
- **High (8/10):** Test #1 fire-branch assertion shape — exact
  mirror of trunk plan §13 Task 3 verbatim.
- **High (8/10):** Test #2 escape-branch ADR-015 defensive
  assertion — explicit `assert_ne!` against raw id.
- **High (8/10):** Test #4 per-case isolation strong assertion —
  case_b ordered first, case_a second, asserts case_a fired
  despite case_b's empty-sanction warn.
- **High (8/10):** Test #5 batch_size config — three invocations
  asserting iteration cap.
- **Moderate (7/10):** §5 complexity score 17 trips threshold;
  planner self-resolved DQ #151 with proceed rationale citing
  e2e-factor analysis + DQ #150 split-level precedent.
- **High (8/10):** registry-marker flip discipline at c-2 retro
  per `feedback_build_what_tests_exercise.md` — first in-fork
  instance of the deferral applied across a split sub-phase
  boundary.

### 19.7 Why no clarify DQ at impl time (c-2)

(Inherited from trunk plan §19.7 + adapted for c-2.)

Brief §4.2 enumerates the boundary-of-judgment cases that should
trigger an impl-side DQ. None apply to this c-2 plan as written:

- v0 `apply_sponsor_liability` signature at
  `sponsor_liability.rs:142` is intact.
- Sanction multiplicity is 1:1.
- ENTRY_KIND consts declared in SL-a.
- Shim re-exports present.
- **c-1 module pub fns landed and verified by Probe 7 + 7b at
  Task 0.**
- Canonical mirrors intact.
- DQ #144-#147 + #150 resolved.

If any of these baseline assumptions changes between plan-write and
impl-time (specifically: c-1 module signatures drift between c-1
ship and c-2 cut), the impl-task subagent files a DQ pending entry.

### 19.8 Forward-only retrofit scope

Per `feedback_schema_changing_spec_retrofit_question.md` — c-2 does
NOT change the shape of any existing artifact class (no new template
section, no new schema marker, no new YAML field). All §13 task
contracts are routine impl-task contracts. No retrofit question
applies.

### 19.9 Cross-PR carry-forward discipline (c-1 → c-2)

The c-2 plan's Task 0 Probe 7 is the module-presence check. c-2
Task 6 retro §2 (per-role signals) records cross-PR carry-forward
findings: did c-1 retro accurately predict c-2's needs? did Probe 7
catch any c-1 ship gaps? did the c-1 → c-2 retro chain produce a
clean hand-off?

This is the first in-fork instance of cross-PR carry-forward applied
to a structurally-split sub-phase. Promote findings to PMD per §19.2
recommendations.

---

## 20. Confidence score

- **Plan correctness:** 9/10 — patterns mirror
  trunk plan §10.8 (fixture mod shape) verbatim; §13 task bodies
  carry verbatim from trunk plan Tasks 3-7; the 5 tests are
  mechanical with strong precedents in SL-b, JM-e, JM-d.
- **Cargo budget:** 10/10 — Shape G (cargo runs off-box; budget
  non-binding; forbidden-window non-binding for impl-task).
- **Test coverage:** 8/10 — 5 behaviourally distinct tests covering
  fire path + escape path + no-op + per-case isolation + batch_size
  config. Multi-sponsor escape rules at scheduler-tick time are not
  exercised (out per §12 — SL-b's revocation handler covers them at
  revocation time). Restoration-escape branch is stub-only.
- **Story-grain decomposition:** 9/10 — c-2 ships exactly Stories
  2 + 3; Story 1 deferred to c-1; clean grain matches trunk plan.
- **Plan-shape conformance:** 9/10 — 20-section schema followed
  literally; §15.6 Shape G shape adopted; §16a mandatory present;
  §13 per-task FILES YAML block on every task; §5.1 complexity
  breakdown table mechanical; c-2-specific slices in §5/§11/§13/§14/
  §15.4/§16/§16a clearly demarcated from inherited sections.

---

_Plan author: planning subagent (laptop sibling worktree
`/Users/barrie/Developer/lemmy-advisor-sl-c` on `governance-v0` @
`abcf778e1`, 2026-05-07 — c-1 / c-2 split per DQ #150 user override
at plan-approval gate). Plan committed locally on `governance-v0`
together with sibling `v1-sponsor-liability-c-1.plan.md`; advisor
publishes via standard sub-phase flow. BM-task cuts
`phase-v1-SL-c-2` (only after c-1 PR merges). c-2 score 17 above
threshold; planner DQ #151 self-resolved with proceed rationale
(e2e factor dominates; further splitting yields no meaningful
reduction; SL-b/SL-a/JM-e/trunk-SL-c proceed-as-one precedents).
Confidence 9/10. Plan ships under proceed-within-c-2 assumption
(DQ #151)._
