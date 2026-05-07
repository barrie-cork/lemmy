# Plan: v1-sponsor-liability-c — `sponsor_liability_grace` scheduler module + clokwerk wiring + 5 integration tests

> **Shape G plan** — SL-c ships under Shape G (Layer G2 push-and-exit). §15
> references workflow YAMLs by path + expected `conclusion`, not inline cargo.
> Cargo runs on GitHub-hosted runners (workspace-check on `junior/*`) and on
> the laptop / `workflow_dispatch` GH runner (e2e on `phase-v1-SL-c`
> post-finalize-merge). See `.claude/PRPs/templates/plan.template.md` §15.6
> + `.claude/PRPs/plans/v1-validate-agent.plan.md`.

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
| 12 | NOT building in v1-SL-c |
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

v1-SL-c ships the v0-deferred `sponsor_liability_grace` scheduler module per
PRD §6 + §9.4 + §9.5 + §15 row 3. It creates a new server-internal module at
`crates/api/api/src/governance/sponsor_liability_grace.rs` exporting four
public async functions (`run_grace_check_batch`, `evaluate_escape_conditions`,
`fire_or_escape_case`, `check_grace_staleness`); wires a clokwerk tick block
in `crates/routes/src/utils/scheduled_tasks.rs::setup` (sibling of the
15-minute `reputation_snapshot` block at lines 189-245 + the hourly
`appeal_window_expiry` block at lines 254-274); adds a third atomic
concurrency guard pair (`SPONSOR_LIABILITY_GRACE_RUNNING` +
`GraceCheckRunningGuard`, sibling of `REPUTATION_SNAPSHOT_RUNNING` and
`APPEAL_WINDOW_EXPIRY_RUNNING`); and ships **5 integration tests** that
behaviourally cover PRD §6.1 (scheduler tick), §6.2 (per-case batch
semantics), and §6.3 (failure mode + observability). The scheduler reads
`SponsorLiabilityPending` cases past their `grace_expires_at`, opens one
`run_transaction` per case (per-case isolation), evaluates escape conditions
(sponsor revoked since `decided_at`?), and either fires (calls **unsplit v0**
`apply_sponsor_liability` from `sponsor_liability.rs:142`, then UPDATEs
`status = SponsorLiabilityFired`, emits `sponsor_liability_fired` summary
log) or escapes (UPDATEs `status = SponsorLiabilityEscaped`, sets
`liability_escape_reason` JSONB, emits `sponsor_liability_escaped` log).
SL-c does NOT add new schema, migrations, ENTRY_KIND consts, CaseStatus
variants, or `governance_config` seeds — all required substrate shipped in
SL-a; the v0 fire helper and the `_FIRED`/`_ESCAPED` consts are intact.

**Why now.** SL-a shipped 2026-05-04 (PR #111, `governance-v0` @ `790f6101d`)
laying schema + 13 config seeds + 5 ENTRY_KIND consts + 3 CaseStatus
variants. SL-b is in flight on `phase-v1-SL-b` (PR #119), shipping the
revocation handler that produces the `sponsor_liability_escaped` revocation-
branch fire-site. Until SL-c lands, cases in `SponsorLiabilityPending`
remain stuck — the grace window can never expire, sponsors never absorb
liability, and the `_FIRED` registry marker stays `(pending)`. SL-c
unblocks SL-d (whose `submit_jury_vote → SponsorLiabilityPending`
transition writes the cases SL-c then consumes).

**Compute/fire posture (advisor-locked, masthead of `sl-c-planning-1.md`).**
SL-c calls **unsplit v0** `apply_sponsor_liability(conn, target_person_id,
case_id, community_id, action, &mut cache)` from the scheduler's fire
branch. The PRD §9.1 compute/fire split is **NOT** SL-c's deliverable —
it remains SL-d's. SL-c's call site is forward-compatible: when SL-d
ships and `apply_sponsor_liability` becomes a thin wrapper (or is
replaced by `compute_sponsor_liability` + `fire_sponsor_liability`),
SL-c's call site updates as a no-op-behaviour change.

**Headline acceptance condition.** All three §16a stories are `[done]`
with their checkpoint workflows green: (1) the new module + scheduler
block + atomic guard pair compile clean and the scheduler tick fires on
boot; (2) the fire branch transitions `SponsorLiabilityPending →
SponsorLiabilityFired`, emits per-sponsor `sponsor_liability_applied`
entries (from v0 `apply_sponsor_liability`) plus the SL-c summary
`sponsor_liability_fired` entry, and writes `reputation_event` rows;
(3) the escape branch transitions `SponsorLiabilityPending →
SponsorLiabilityEscaped`, sets `liability_escape_reason` JSONB matching
PRD §8.1 schema (`version: 1`, `actor_pseudonym` per ADR-015), emits
`sponsor_liability_escaped`, and writes NO `reputation_event` rows; the
per-case isolation invariant holds (one bad case doesn't block batch);
the staleness check emits `tracing::error!` when cases are stuck >2×
grace window.

This sub-phase touches NO schema, NO migration, NO ENTRY_KIND const, NO
CaseStatus variant, NO `governance_config` seed, NO HTTP endpoint, NO
DTO, NO route. It is **server-internal scheduler module + clokwerk block
+ 5 e2e tests + retro**.

---

## 2. Source

- `.claude/PRPs/briefs/sl-c-planning-1.md` @ `governance-v0` `cb0a9a4ea`
  — the advisor brief (post-`/brehon-clarify`; DQ #144-#147 resolved
  2026-05-07).
- `.claude/PRPs/prds/v1-sponsor-liability.prd.md` @ `governance-v0` —
  parent PRD; **§6 IS the SL-c spec** (§6.1 scheduler tick, §6.2
  per-case batch semantics, §6.3 failure mode + observability, §6.4
  configuration); §1 (Brehon athgabál framing); §2 (IN/OUT — confirms
  SL-c is scheduler module + wiring + observability + tests); §3.1 +
  §3.4 (CaseStatus extensions — confirms SL-a shipped 3 variants;
  SL-c reads `SponsorLiabilityPending`, writes `SponsorLiabilityFired`
  AND `SponsorLiabilityEscaped`); §7 (restoration interaction —
  restorative-mechanics-v1 PRD owns the producer; SL-c stubs the
  branch); **§8.1 (`liability_escape_reason` JSONB schema —
  load-bearing; `version: 1` per OQ-V1-SL-05)**; §9.1 (`apply_sponsor_liability`
  split — SL-d's deliverable; SL-c calls UNSPLIT v0); §9.4 (helper module
  signatures — SL-c implements); §9.5 (scheduler wiring — SL-c
  implements); §10 (Defaults Matrix — confirms `job.grace_check_*`
  defaults all SL-a-seeded); §11 (backwards compat — §11.2 mid-flight
  cases); §12 (security — §12.4 threat model); §15 (implementation
  phases — confirms SL-c is row 3); §17 (cross-cutting — ADR-013
  enum-exhaustiveness; SL-c adds zero variants); §18 (B4 key-rename
  table for `liability.*`).
- `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` — most recent SL
  plan; §6 table shape (canonical mirror for SL-c §6); §16a Stories
  shape; §13 task ordering; §15 Shape G DoD shape; §10 §13 §16a
  conventions; §18 risks shape.
- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — predecessor
  ships the schema + seeds + consts + variants SL-c reads. SL-c §11
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
  (PR #105).
- `.claude/rules/governance-log-entry-kind-registry.md` — confirms
  `_SPONSOR_LIABILITY_FIRED` (line 172, "v1-SL-a const; v1-SL-c call
  site (pending)") and `_SPONSOR_LIABILITY_ESCAPED` (line 173, "v1-SL-a
  const; v1-SL-b + v1-SL-c call sites (pending)") shipped in SL-a;
  SL-c is the fire-site for `_FIRED` AND a fire-site for `_ESCAPED`'s
  scheduler-branch portion. Registry total stays at the post-SL-a
  count.
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  — **ADR-005** (multi-dimensional reputation; SL-c's fire branch
  writes per-dimension events via v0 `apply_sponsor_liability`),
  **ADR-008** (append-only signed log; every SL-c emit goes through
  `governance_log::append`), **ADR-010** (won't-disadvantage rule;
  v1 mid-flight backfill SL-c first sees), **ADR-013** (CaseStatus
  enum-exhaustiveness — no `_ =>` arms in SL-c matches), **ADR-014**
  (federation deferral — SL-c emits log events on local instance only;
  no outbox), **ADR-015 (pseudonymisation — load-bearing for SL-c;
  every payload field naming a person uses pseudonym, not raw id)**,
  OQ-V1-SL-05 (`liability_escape_reason` schema versioning — `version:
  1` from day one).

### Lessons that bind §13 decisions

- `feedback_multi_write_handlers_need_transactions.md` —
  **load-bearing for SL-c**. Each per-case body wraps re-load + status
  re-check + sanction lookup + escape-condition evaluation + UPDATE +
  log entry inside one `run_transaction`. The outer
  `run_grace_check_batch` does NOT open a transaction — it iterates
  cases, each opening its own per-case tx. Per-case errors are caught
  + logged via `warn!`; iteration continues.
- `feedback_advisor_watchpoint_specificity.md` (cited by brief) —
  every §4 watchpoint cites a concrete file/handler/`schema.rs`/
  `enums.rs` line. Binds §4 entries.
- `feedback_complexity_score_pre_split.md` — score 21 → planner files
  proceed-as-one DQ #148 BEFORE commit (mirroring SL-b DQ #143).
  Binds §5.2.
- `feedback_explicit_file_arrays_on_tasks.md` — every §13 task carries
  a FILES YAML block; cohort dispatch reads `union(creates, modifies)`.
- `feedback_parallel_cohort_dispatch.md` — Tasks 3-7 all
  `modifies: crates/server/tests/e2e.rs`; YAML overlap rule refuses
  cohort dispatch; tasks ship serially (no `[P]` markers in §13). Task
  1 (new module) and Task 2 (scheduler wiring) edit different files
  but Task 2 imports the symbol Task 1 creates → serial.
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
- `feedback_insertform_default_propagation.md` — citation-only: SL-c
  does not add new InsertForm fields. Tests pre-seed
  `SponsorLiabilityPending` cases via existing
  `ModerationCaseInsertForm` (verified at
  `crates/db_schema/src/source/governance/moderation_case.rs:99-137`,
  carries `grace_expires_at` + `liability_escape_reason` from SL-a Task
  4).
- `feedback_clippy_test_style.md` — R1 every `i32 ↔ i64` comparison
  uses `i64::from(...)`. Bound in §4 watchpoint #14
  (`config::get_int` returns `i64`; chrono `Duration::seconds(i64)`;
  bridge if `i32` intermediates appear).
- **`feedback_junior_worker_e2e_edit_hang.md`** —
  **load-bearing for SL-c**. 5 e2e tests = 5 individual §13 tasks,
  each one anchor-pattern Edit at file end. e2e.rs is currently 10976
  lines on `governance-v0`; will be ~10500-10600 once SL-b merges
  (SL-b adds 9 tests at ~600 lines combined plus a fixture mod). Bundle
  Edits hang Junior workers.
- **`feedback_e2e_filter_assumes_naming.md`** — §15.7 manual
  validation snippets, if included, run full e2e suite without
  `--test e2e <filter>`, OR confirm via grep first.
- `feedback_brehon_verify_pre_merge.md` — §16a Stories grain enables
  `/brehon-verify` phantom check before `bm-merge`.
- `feedback_principles_not_rules.md` — score 21 is a signal; planner
  observation in §5.2 leans proceed-as-one with rationale (each test
  is anchor-Edit-friendly; per-task wall-clock ~6-8 min under Sonnet
  4.6; SL-a + SL-b + JM-e proceed-as-one precedents).
- `feedback_read_canonical_before_writing_spec.md` — citation-only
  (SL-c adds no new spec/template/rule); plan §10 cites
  `reputation_snapshot.rs::run_snapshot_batch` AND
  `appeal_window_expiry.rs::run_appeal_window_expiry_batch` as paired
  canonical mirrors per DQ #147.
- `feedback_schema_changing_spec_retrofit_question.md` — citation-only
  (SL-c changes no spec/template shape).
- `feedback_pr_per_phase.md` — one commit per task; §13 ordering
  matters.
- `feedback_handover_trailer_cohort_propagation.md` — non-binding
  under serial dispatch (no `[P]` cohorts in §13); per-task
  `HANDOVER:` commit trailer still recommended as mechanical
  discipline.
- `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`
  + `feedback_retro_task_complexity_score.md` — retro shape (final
  task).
- **`feedback_build_what_tests_exercise.md`** (PMD #14) —
  **load-bearing for SL-c**. The restoration-escape detection branch
  is stub-only per DQ #145; `evaluate_escape_conditions`'s restoration
  check returns `Fire` until restorative-mechanics-v1 ships the
  producer. Plan §19 documents the stub-and-future-wire path.

### Related prior plans (canonical-shape mirror)

- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — schema
  foundation SL-c reads. SL-c §11 does not duplicate.
- `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` — most recent SL
  plan; §6 + §16a + §13 + §15 shapes mirrored.
- `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` — Shape-G
  + anchor-Edit-per-test reference.
- `.claude/PRPs/plans/phase-5b-sponsor-liability-and-founder-bootstrap.plan.md`
  — context-only; v0 `apply_sponsor_liability` foundation. SL-c reads
  but does NOT touch.
- `.claude/PRPs/briefs/sl-a-planning-1.md` + `sl-b-planning-1.md` +
  `sl-c-planning-1.md` — exemplar planning briefs for shape and
  constraint language. SL-c brief (this plan's source) is the most
  recent.

---

## 3. Problem statement

Post-SL-a-merge on `governance-v0` @ `790f6101d` (and post-SL-b-merge
expected before SL-c is cut):

- **The `sponsor_liability_grace.rs` module does not exist.** No
  scheduler module reads `SponsorLiabilityPending` cases past their
  `grace_expires_at` and transitions them. The 13 `liability.*` +
  `job.grace_check_*` config seeds shipped in SL-a (verified at
  `crates/api/api/src/governance/config.rs:925-930` + `:943-945` +
  the seed-row entries at `:1019-1056` + `:1373` etc.) have no reader.
- **The `SponsorLiabilityFired` terminal state has no fire site.**
  Per the registry at `.claude/rules/governance-log-entry-kind-registry.md:172`,
  `_FIRED`'s emitting handler is "v1-SL-c
  `crates/api/api/src/governance/sponsor_liability_grace.rs::run_grace_check_batch`
  fire branch (pending)". The const `ENTRY_KIND_SPONSOR_LIABILITY_FIRED`
  exists at `crates/db_schema/src/source/governance/governance_log.rs:198`
  with shim re-export at `crates/api/api/src/governance/governance_log.rs:75`,
  but no emitter. The `(pending)` marker is unflipped.
- **The scheduler-branch fire-site for `_ESCAPED` is uncovered.** Per
  the registry at line 173, `_ESCAPED`'s handler is "v1-SL-b
  `revoke_endorsement.rs` (pending) AND v1-SL-c `sponsor_liability_grace.rs::evaluate_escape_conditions`
  (pending)". SL-b ships the revocation-branch fire-site; SL-c ships
  the scheduler-branch fire-site. Both must land before the registry
  marker fully flips to `(active)`.
- **Cases in `SponsorLiabilityPending` are stuck.** Per
  `crates/db_schema_file/src/enums.rs:411-416` doc-comments, cases
  reach `SponsorLiabilityPending` via SL-d's `submit_jury_vote`
  rewrite OR via SL-a's mid-flight backfill (PRD §11.2). Neither
  population transitions to a terminal state without SL-c's scheduler
  picking them up.
- **The `BREHON_DISABLE_GRACE_CHECK_JOB` test override is unhooked.**
  The pattern is precedented at
  `crates/routes/src/utils/scheduled_tasks.rs:197` (snapshot job's
  `BREHON_DISABLE_SNAPSHOT_JOB`) and `:258`
  (appeal-window-expiry's `BREHON_DISABLE_APPEAL_WINDOW_JOB`); SL-c's
  e2e tests rely on the third sibling override.
- **The third sibling concurrency-guard pair is missing.**
  `scheduled_tasks.rs:71-79` declares `REPUTATION_SNAPSHOT_RUNNING` +
  `RunningGuard`; lines 83-91 declare `APPEAL_WINDOW_EXPIRY_RUNNING` +
  `AppealWindowExpiryRunningGuard`. SL-c adds the third pair
  (`SPONSOR_LIABILITY_GRACE_RUNNING` + `GraceCheckRunningGuard`) so
  the new tick block can guard against overlap if a batch exceeds the
  5-minute interval.
- **The grace-window staleness observability surface is missing.** Per
  PRD §6.3, cases stuck >2× max grace window must emit `tracing::error!`
  for ops visibility. The pattern is precedented at
  `crates/api/api/src/governance/reputation_snapshot.rs:423`
  (`check_snapshot_staleness`); SL-c adds a sibling
  `check_grace_staleness` inside the new module.

The substrate is in place; only the new module + scheduler block edits
+ 5 integration tests are missing.

---

## 4. Solution statement

Seven surgical changes, organised as 7 impl tasks + Task 0 pre-flight +
retro.

### 4.1 Architecturally load-bearing decisions locked in this sub-phase

- **The scheduler module is a sibling of `reputation_snapshot.rs` and
  `appeal_window_expiry.rs`, not a rewrite or extension of either.**
  `crates/api/api/src/governance/sponsor_liability_grace.rs` is
  created; mirrors `appeal_window_expiry.rs:34-87`'s per-row pattern
  (closer fit than `reputation_snapshot.rs:361-407`'s per-chunk
  pattern, since SL-c is one tx per case, not one tx per chunk). Module
  wired via `crates/api/api/src/governance/mod.rs` `pub mod
  sponsor_liability_grace;` (alphabetically AFTER
  `pub mod sponsor_liability;` per the existing precedent in that
  file's alphabetical block).
- **Four public async functions per PRD §9.4.** Module exports:
  - `pub async fn run_grace_check_batch(context: &LemmyContext) ->
    LemmyResult<GraceCheckBatchOutcome>` — scheduler entry. Outer
    function — does NOT open a transaction; iterates cases, each
    opening its own.
  - `pub async fn evaluate_escape_conditions(conn: &mut AsyncPgConnection,
    case_id: ModerationCaseId, target_person_id: PersonId,
    community_id: Option<CommunityId>, decided_at: DateTime<Utc>,
    cache: &mut ConfigCache) -> LemmyResult<EscapeStatus>` — pure
    read; returns `EscapeStatus::Escape{reason, actor_pseudonym,
    ref_id}` or `EscapeStatus::Fire`.
  - `pub async fn fire_or_escape_case(conn: &mut AsyncPgConnection,
    case_row: ModerationCase, status: EscapeStatus, cache: &mut
    ConfigCache) -> LemmyResult<()>` — per-case transaction body.
  - `pub async fn check_grace_staleness(conn: &mut AsyncPgConnection,
    max_grace_hours: i64, multiplier: f64, now: DateTime<Utc>) ->
    LemmyResult<()>` — pure observability; emits `tracing::error!`
    if any pending case has `decided_at < now() - threshold_hours`.
- **`EscapeStatus` enum — three variants (one stub-only).** Per DQ
  #145:
  ```rust
  enum EscapeStatus {
      Fire,
      Escape { reason: String, actor_pseudonym: String, ref_id: i64 },
  }
  ```
  Inside `evaluate_escape_conditions`, the construction of an
  `Escape{reason: "restoration_completed", ...}` value is **not
  wired** in v1-SL-c — the restoration-completed read returns `Fire`
  unconditionally. The reason-string convention `"sponsor_revoked"`
  fires; `"restoration_completed"` is documented in §19 as the future
  branch wired by restorative-mechanics-v1. This is the
  "drift-stub the construction site so future planners find the
  breadcrumb" pattern per `feedback_build_what_tests_exercise.md`.
- **Two-tier `ConfigCache` lifetime per DQ #144.** Outer
  `ConfigCache::new()` declared at the top of `run_grace_check_batch`
  for batch-level reads (`job.grace_check_batch_size`,
  `liability.grace_window_maximum_hours`,
  `job.grace_check_staleness_alert_multiplier`). Per-case
  `ConfigCache::new()` declared at the top of `fire_or_escape_case`
  (inside the per-case scope) and threaded into
  `apply_sponsor_liability(... &mut cache)`. Mirrors
  `reputation_snapshot.rs:363/389` verbatim — the per-chunk cache
  there is the analogue of SL-c's per-case cache. Sharing one cache
  across `run_transaction` closures is fiddly with `&mut`; isolation
  by per-case cache provides clean per-case-tx semantics.
- **Inside-transaction step ordering (PRD §6.2 — load-bearing for
  §13 task split).** `fire_or_escape_case` body, inside one
  `run_transaction` closure per case:
  1. **Re-load case row with `FOR UPDATE`** — `moderation_case::table`
     filter `id.eq(case_row.id)` + `for_update()` + `first(conn)`.
     Per `feedback_multi_write_handlers_need_transactions.md` and
     Phase 5a Watch 9 pattern. Defends against scheduler-vs-handler
     race (e.g. SL-b's `revoke_endorsement` mutated this case
     mid-batch).
  2. **Re-check status** — if `re_loaded.status !=
     CaseStatus::SponsorLiabilityPending`, return `Ok(())` from the
     per-case body without UPDATE. Defence against SL-b mutating to
     `SponsorLiabilityEscaped` between batch query and per-case tx.
  3. **Lookup sanction action** — `sanction::table` filter
     `case_id.eq(case_id)` + `order_by(id.asc())` +
     `select((action, scope))` + `first::<(SanctionAction,
     SanctionScope)>(conn).optional()`. If `None`, `tracing::error!`
     and return `Ok(())` (skip case silently — empty-sanction is a v0
     invariant violation but doesn't crash batch).
  4. **Branch on `EscapeStatus`:**
     - `Escape{reason, actor_pseudonym, ref_id}`:
       - `update(moderation_case::table.filter(id.eq(case_id)))
         .set((status.eq(SponsorLiabilityEscaped),
         liability_escape_reason.eq(Some(json!({"version": 1, "reason":
         reason, "actor_pseudonym": actor_pseudonym, "endorsement_id":
         ref_id})))))`
       - `governance_log::append(conn,
         ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED, json!({"case_id":
         case_id.0, "escaped_at": now, "reason": reason,
         "actor_pseudonym": actor_pseudonym, "endorsement_id":
         ref_id}), Some(actor_pseudonym.clone())).await?`
     - `Fire`:
       - `let sponsor_count =
         apply_sponsor_liability(conn, target_person_id, case_id,
         community_id, action, &mut cache).await?` — invokes the v0
         helper unsplit per advisor-locked posture. Writes per-sponsor
         `reputation_event` rows + `sponsor_liability_applied` log
         entries + `sponsor_liability_clamped` log entries when
         applicable.
       - `update(moderation_case::table.filter(id.eq(case_id)))
         .set(status.eq(SponsorLiabilityFired))`
       - `governance_log::append(conn,
         ENTRY_KIND_SPONSOR_LIABILITY_FIRED, json!({"target_pseudonym":
         target_pseudonym, "sponsor_count": sponsor_count, "case_id":
         case_id.0, "fired_at": now}), Some(target_pseudonym)).await?`
         — single summary entry on TOP of v0's per-sponsor entries
         (Test #1 asserts both kinds are present).
- **Outer `run_grace_check_batch` swallows per-case errors via
  `.inspect_err(|e| warn!(...)).ok()` pattern (per PRD §6.3 + per-case
  isolation invariant).** The for-loop body wraps the
  `fire_or_escape_case` call; on `Err`, log the case_id and continue.
  Outer function returns `Ok(...)` regardless of per-case outcomes.
- **Scheduler tick wiring at `scheduled_tasks.rs:setup` — third
  sibling block.** Pattern matches the snapshot block at lines 189-245
  and the appeal-window-expiry block at lines 254-274. Concrete
  shape:
  - **Module-scope statics** (lines 65-91 area): add at line 92 (after
    `AppealWindowExpiryRunningGuard`):
    ```rust
    static SPONSOR_LIABILITY_GRACE_RUNNING: AtomicBool = AtomicBool::new(false);

    struct GraceCheckRunningGuard;

    impl Drop for GraceCheckRunningGuard {
      fn drop(&mut self) {
        SPONSOR_LIABILITY_GRACE_RUNNING.store(false, Ordering::Release);
      }
    }
    ```
  - **Inside `setup()` (after the appeal-window-expiry block at line
    274):** add the new tick block per PRD §6.1:
    ```rust
    // Brehon governance: sponsor-liability grace-check tick. Interval
    // read from job.grace_check_interval_minutes at scheduler setup
    // (default 5; restart-required tunability — clokwerk pins
    // at registration). Find SponsorLiabilityPending cases past their
    // grace_expires_at and transition to Fired or Escaped.
    // Concurrency guard mirrors REPUTATION_SNAPSHOT_RUNNING and
    // APPEAL_WINDOW_EXPIRY_RUNNING. Disabled for e2e tests via
    // BREHON_DISABLE_GRACE_CHECK_JOB=1.
    let context_grace = context.reset_request_count();
    let grace_pool = &mut context.pool();
    let grace_interval_minutes_i64 = lemmy_api::governance::config::get_int(
      &mut lemmy_api::governance::config::ConfigCache::new(),
      grace_pool,
      lemmy_api::governance::config::Scope::Instance,
      "job.grace_check_interval_minutes",
    )
    .await
    .unwrap_or(5);
    let grace_interval_minutes: u32 = u32::try_from(grace_interval_minutes_i64)
      .unwrap_or(5);
    scheduler.every(CTimeUnits::minutes(grace_interval_minutes)).run(move || {
      let context = context_grace.reset_request_count();
      async move {
        if std::env::var("BREHON_DISABLE_GRACE_CHECK_JOB").as_deref() == Ok("1") {
          return;
        }
        if SPONSOR_LIABILITY_GRACE_RUNNING
          .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
          .is_err()
        {
          warn!("sponsor_liability_grace: previous batch still running, skipping this tick");
          return;
        }
        let _guard = GraceCheckRunningGuard;
        lemmy_api::governance::sponsor_liability_grace::run_grace_check_batch(&context)
          .await
          .inspect_err(|e| warn!("Failed to run grace_check batch: {e}"))
          .ok();

        // Staleness pass after the batch (mirrors snapshot pattern at
        // scheduled_tasks.rs:213-244).
        let staleness_pool = &mut context.pool();
        let mut staleness_cache =
          lemmy_api::governance::config::ConfigCache::new();
        let max_grace_hours = lemmy_api::governance::config::get_int(
          &mut staleness_cache,
          staleness_pool,
          lemmy_api::governance::config::Scope::Instance,
          "liability.grace_window_maximum_hours",
        )
        .await
        .unwrap_or(720);
        let multiplier = lemmy_api::governance::config::get_float(
          &mut staleness_cache,
          staleness_pool,
          lemmy_api::governance::config::Scope::Instance,
          "job.grace_check_staleness_alert_multiplier",
        )
        .await
        .unwrap_or(2.0);
        match get_conn(staleness_pool).await {
          Ok(mut conn) => {
            if let Err(e) =
              lemmy_api::governance::sponsor_liability_grace::check_grace_staleness(
                &mut conn,
                max_grace_hours,
                multiplier,
                Utc::now(),
              )
              .await
            {
              warn!("grace staleness check failed: {e}");
            }
          }
          Err(e) => warn!("grace staleness check: get_conn failed: {e}"),
        }
      }
    });
    ```
  Order in `setup()`: the env-var check is FIRST in the closure body
  (before `compare_exchange`); reversing leaks atomic-bool slots when
  tests set the env var.
- **`check_grace_staleness` formula per DQ #146.** `threshold_hours =
  liability.grace_window_maximum_hours × job.grace_check_staleness_alert_multiplier`
  computed inside the function from its `i64` + `f64` parameters.
  Per-case test: `now - decided_at > threshold_hours`. Defaults give
  720 × 2.0 = 1440h ≈ 60 days, matching PRD §6.3 "(>60 days)". Measure
  from `decided_at` (NOT `grace_expires_at`) — the semantic is "this
  case has been pending so long it's escaped notice"; `decided_at` is
  the canonical case-age anchor. Pure observability — no DB writes,
  no governance_log entries. Returns `Ok(())` even if many cases are
  stuck — emits `tracing::error!` per stuck case.
- **Sanction-action lookup query.** Verified 1:1 multiplicity at
  `crates/api/api/src/governance/submit_jury_vote.rs:435` (single
  `insert_into(sanction::table)` per case). SL-c queries
  `sanction::table.filter(case_id.eq(case_id)).order_by(id.asc()).limit(1)`
  to defend against future drift; if zero rows, `tracing::error!`
  + skip case (don't crash batch). No planner-DQ on multiplicity —
  brief §4.2 said file DQ if multiplicity is many-to-one; verified
  1:1, no DQ filed.
- **The 5 e2e tests live in a NEW `mod v1_sl_c_fixtures` block in
  `crates/server/tests/e2e.rs`, after `mod v1_sl_b_fixtures`.** SL-b
  is in flight; once it merges, e2e.rs grows to ~10500-10600 lines
  and `mod v1_sl_b_fixtures` becomes the last fixture mod. SL-c's
  Task 3 anchors at the end of `mod v1_sl_b_fixtures` and opens
  `mod v1_sl_c_fixtures`. Subsequent SL-c tests (Tasks 4-7)
  anchor-Edit AFTER the prior task's test inside the same mod, per
  `feedback_junior_worker_e2e_edit_hang.md`. Tasks 3-7 all
  `modifies: crates/server/tests/e2e.rs`; YAML overlap rule refuses
  cohort dispatch — they ship serially. **If SL-b has not merged at
  SL-c plan-implement time, Task 3's anchor falls back to end of
  `mod v1_jm_e_fixtures` (line 10975 on `governance-v0` HEAD).** The
  impl-task brief should grep at task-start-time:
  `grep -n '^mod v1_' crates/server/tests/e2e.rs | tail -1` to find
  the last fixture mod and anchor there. Per-task GOTCHA noted in
  §13 Task 3.
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
  this at bootstrap; the scheduler's tick block returns early. Tests
  drive `run_grace_check_batch` directly to control timing.
- **Shape G — DoD references workflow YAMLs by path + expected
  `conclusion`, not inline cargo.** Per
  `.claude/PRPs/templates/plan.template.md` §15.6 + DQ #67 resolution.
  Every §13 task's DoD references `cargo-validate-workspace.yml` on
  the worker branch + workflow_run_id captured by impl-task subagent
  post-push.
- **No migration round-trip workflow fires.** SL-c touches no
  `migrations/**` paths; `cargo-validate-migration.yml`'s path filter
  excludes it. Only `cargo-validate-workspace.yml` (per task) +
  `cargo-test-e2e.yml` (per Phase 2 e2e dispatch on `phase-v1-SL-c`).

### 4.2 Watchpoints (specific files / handlers / schema lines)

Per `feedback_advisor_watchpoint_specificity.md` (cited by brief §4.1)
— every entry cites a concrete file/line/handler. The 12 watchpoints
seeded by brief §4.1, plus additions surfaced at plan-write time:

1. **Per-case `FOR UPDATE`.** Plan §13 Task 1 cites:
   `fire_or_escape_case` step 1 — re-load `moderation_case` row with
   `for_update()` inside the per-case transaction. Defends against
   scheduler-vs-handler race per
   `feedback_multi_write_handlers_need_transactions.md` and Phase 5a
   Watch 9 pattern. Without `FOR UPDATE`, SL-b's `revoke_endorsement`
   could mutate the case between batch query and per-case tx, causing
   a double-state-write.
2. **Re-check status inside transaction.** Plan §13 Task 1 specifies:
   after `FOR UPDATE`, re-read `case.status`; if no longer
   `SponsorLiabilityPending`, return `Ok(())` from the per-case body
   without UPDATE. Second half of the race defence.
3. **Atomic concurrency guard pattern.** Plan §13 Task 2 cites
   canonical mirror at `scheduled_tasks.rs:71-79`
   (`REPUTATION_SNAPSHOT_RUNNING` + `RunningGuard`) and 83-91
   (`APPEAL_WINDOW_EXPIRY_RUNNING` + `AppealWindowExpiryRunningGuard`),
   producing a sibling `SPONSOR_LIABILITY_GRACE_RUNNING` +
   `GraceCheckRunningGuard` pair. **Do NOT** reuse an existing static
   — different concurrency domains; sharing would deadlock-couple.
4. **`liability_escape_reason` JSON schema locked.** Schema:
   `{"version": 1, "reason": "sponsor_revoked", "actor_pseudonym":
   "...", "endorsement_id": <i64>}`. `actor_pseudonym` mandatory per
   ADR-015; `version: 1` mandatory per OQ-V1-SL-05. Plan §13 Task 1
   cites `actor_pseudonym_helper::get_or_create` (sourced from
   `crates/api/api/src/governance/actor_pseudonym_helper.rs`) for
   pseudonym derivation. Raw `caller_id` / raw `from_person_id` in
   reason JSON is a GDPR-013 violation, catch-fire. Test #2 (Task 4)
   asserts the JSON shape defensively.
5. **Two log entries per fire path, one per escape path.** Fire path
   emits BOTH the per-sponsor `sponsor_liability_applied` entries
   (from v0 `apply_sponsor_liability` at `sponsor_liability.rs:322-337`
   per sponsor) AND the SL-c summary `sponsor_liability_fired` entry
   (single, after the v0 helper returns). Escape path emits ONLY the
   `sponsor_liability_escaped` entry (no `apply_sponsor_liability`
   invoked). Plan §13 Task 1 specifies which entries fire on which
   branch; Test #1 (Task 3) asserts governance_log row counts per
   branch.
6. **Sanction-action lookup correctness.** Plan §13 Task 1 cites:
   query `sanction::table.filter(case_id.eq(case_id)).order_by(id.asc()).limit(1).select((action, scope)).first(conn).optional()`.
   If `None`, log `tracing::error!` and skip case silently (don't
   UPDATE status; don't crash batch). 1:1 multiplicity verified at
   `submit_jury_vote.rs:435` (single insert per case).
7. **No new `moderation_case.status` mutations OUTSIDE the batch
   loop.** SL-c writes `status` ONLY inside `fire_or_escape_case`.
   MUST NOT mutate `Decided` cases, `Open` cases, or any other state.
   The orphan-case `ModerationCaseInsertForm` pattern at e2e.rs (used
   by JM-d cancellation tests with `creator_id=NULL +
   winning_decision=NoAction`) is the cautionary anchor — SL-c's
   tests pre-seed `SponsorLiabilityPending` cases via
   `ModerationCaseInsertForm` direct-write but never mutate orphan
   cases.
8. **Per-case isolation — outer batch never returns `Err`.** Plan
   §13 Task 1 specifies: per-case errors are caught via
   `.inspect_err(|e| warn!(...)).ok()`; iteration continues;
   `run_grace_check_batch` returns `Ok(...)` even if N-1 cases fail.
   Per PRD §6.3. Without this, one bad case (e.g. malformed sanction
   row) blocks the entire batch indefinitely. Test #4 (Task 6) is
   the strong assertion catching this regression.
9. **`BREHON_DISABLE_GRACE_CHECK_JOB` env var precedes the atomic
   guard.** Plan §13 Task 2 specifies: env-var check is FIRST in the
   closure body, BEFORE `compare_exchange`. Mirror
   `BREHON_DISABLE_SNAPSHOT_JOB` at `scheduled_tasks.rs:197-199` and
   `BREHON_DISABLE_APPEAL_WINDOW_JOB` at `scheduled_tasks.rs:258-260`.
   Reversing means tests that set the env var still consume an
   atomic-bool slot, leaking guards.
10. **No new ENTRY_KIND_*** consts.** All needed consts shipped in
    SL-a: `ENTRY_KIND_SPONSOR_LIABILITY_FIRED` at
    `crates/db_schema/src/source/governance/governance_log.rs:198`,
    `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` at line 197,
    `ENTRY_KIND_RESTORATION_COMPLETED` at line 196 (read for the
    stubbed branch, never written by SL-c). Shim re-exports at
    `crates/api/api/src/governance/governance_log.rs:74` (`_ESCAPED`),
    `:75` (`_FIRED`), `:68` (`_RESTORATION_COMPLETED`). Plan §13 must
    NOT add to the registry.
11. **No migration in SL-c.** Plan §13 produces zero migration files.
    `git diff governance-v0..phase-v1-SL-c -- migrations/` at
    plan-approval time must show no output.
    `cargo-validate-migration.yml` path filter excludes SL-c tasks.
12. **e2e Edit-per-task discipline.** 5 SL-c tests = 5 individual
    §13 tasks (Tasks 3-7), each one anchor-pattern Edit at file end.
    Do NOT bundle. Per `feedback_junior_worker_e2e_edit_hang.md` —
    e2e.rs is now 10976 lines on `governance-v0`; ~10500-10600 once
    SL-b merges. Bundle Edits hang Junior workers.
13. **R1 `i64::from(...)` discipline.** Per
    `feedback_clippy_test_style.md`, every `i32 ↔ i64` comparison
    uses `i64::from(...)`. SL-c surfaces:
    - `liability.grace_window_maximum_hours` (i64 via `get_int`)
      multiplied by `f64` multiplier → `f64` arithmetic; round to
      `i64` via `round_ties_even`. No `i32` intermediate.
    - `chrono::Duration::hours(threshold_hours)` requires `i64`.
    - `job.grace_check_batch_size` (i64 via `get_int`) → `usize`
      via `usize::try_from(...)` (mirrors
      `reputation_snapshot.rs:367-371`).
14. **Restoration-escape stub-only (DQ #145).** Plan §13 Task 1
    specifies: `evaluate_escape_conditions` defines the
    `EscapeStatus::Escape{reason: "restoration_completed", ...}`
    enum value as a documented future-wire branch but the function
    body NEVER constructs it. The restoration-completed read returns
    `Fire` unconditionally. Per
    `feedback_build_what_tests_exercise.md` (PMD #14) — never
    pre-implement detection for a producer that doesn't yet emit.
    Cited evidence: `ENTRY_KIND_RESTORATION_COMPLETED` exists at
    `governance_log.rs:196` (SL-a-shipped) but has zero emit-sites
    in the codebase today (verified at plan-write time:
    `rg -n 'ENTRY_KIND_RESTORATION_COMPLETED' crates/api/` returns
    only the const declaration + shim re-export; no `governance_log::append`
    call site). Plan §19 documents the future-wire path explicitly.

### 4.3 Rejected alternatives

- **Bundle the 5 e2e tests into 1-2 §13 tasks.** Rejected per
  `feedback_junior_worker_e2e_edit_hang.md` — e2e.rs is 10976+ lines;
  multi-test bulk Edits hang Junior workers. SL-b shipped 9 tests as
  9 tasks for the same reason.
- **Mark Tasks 3-7 as `[P]`.** Rejected: YAML overlap rule refuses
  cohort because all 5 tasks `modifies: crates/server/tests/e2e.rs`.
  Each task anchor-Edits at the prior task's commit tip; cohort
  dispatch would race.
- **Mark Task 1 + Task 2 as `[P]`.** Rejected: Task 2 (scheduler
  block in `scheduled_tasks.rs`) imports the symbols Task 1 creates
  in `sponsor_liability_grace.rs` (specifically
  `lemmy_api::governance::sponsor_liability_grace::run_grace_check_batch`
  and `::check_grace_staleness`). If Tasks 1+2 dispatch
  simultaneously, Task 2's worker branch is cut from `phase-v1-SL-c`
  before Task 1's commit lands; Task 2's compile fails with
  "unresolved import sponsor_liability_grace::*". Serial dispatch is
  correct.
- **Consolidate Task 1 (module) + Task 2 (scheduler) into one
  task.** Rejected per `feedback_pr_per_phase.md` one-commit-per-task
  discipline + per-task FILES YAML block + per-task workspace-check
  workflow run. Two logically distinct concerns get two commits.
- **Pre-implement the restoration-escape detection branch** (per
  PRD §6.2 step 3 second bullet). Rejected per DQ #145 advisor lean
  + `feedback_build_what_tests_exercise.md`. The
  `ENTRY_KIND_RESTORATION_COMPLETED` const has zero producers today;
  pre-implementing the read-side detection is dead-code that drifts
  from the eventually-shipped producer's payload schema. Stub only.
- **Add a `restoration_escapes_liability` config-key read in
  `evaluate_escape_conditions`.** Rejected per the same logic — the
  knob exists in PRD §10 but has no producer to gate. SL-c's
  `evaluate_escape_conditions` checks ONLY `surety.revoked_at >=
  decided_at`; the restoration-completed branch is stub.
- **Use a per-batch `run_transaction` (single tx wrapping all
  cases).** Rejected per PRD §6.3 + per-case isolation invariant +
  watchpoint #8. One bad case must not block the batch; batch-level
  tx would either commit-all-or-rollback-all, violating the
  "next tick retries" semantic.
- **Use FOR UPDATE SKIP LOCKED in the batch query** (mirror
  `appeal_window_expiry.rs:50-51`'s pattern). Rejected — the SKIP
  LOCKED pattern is for batch-level UPDATE under one tx, where
  concurrent row mutators must be deferred to the next tick. SL-c's
  per-case tx pattern handles this differently: each per-case tx
  acquires its OWN `FOR UPDATE` lock at step 1 of the inner body,
  AFTER the outer batch query (no lock) completes. The outer batch
  query intentionally doesn't lock — it's a snapshot of work to do;
  the per-case tx re-validates before mutating.
- **Read `BREHON_DISABLE_GRACE_CHECK_JOB` AFTER the
  `compare_exchange`.** Rejected — per watchpoint #9, env-var check
  is FIRST in the closure body. Reversing leaks atomic-bool slots
  when tests set the env var.
- **Define `EscapeStatus` as a struct with `Option<String> reason`
  field.** Rejected — enum variant is more expressive and exhaustive
  matching at fire-or-escape-case is cleaner. ADR-013-style
  enum-exhaustiveness applies (no `_ =>` arm in match on
  `EscapeStatus`); adding a new variant would force a compile-time
  decision.

---

## 5. Metadata

- **Phase:** `v1-SL-c`
- **Branch:** `phase-v1-SL-c` (cut by BM-task before Task 1, AFTER
  SL-b PR #119 merges)
- **Estimated tasks:** 9 (Task 0 pre-flight + Tasks 1-7 impl + Task 8
  retro)
- **Estimated cargo budget:** 0 GB peak local (Shape G — cargo runs
  on GH-hosted runners)
- **Forbidden-window applicability:** non-binding for impl-task
  throughput under Shape G; standard for any local diagnostic cargo
  run (rare)
- **Complexity score:** **21/10** — see breakdown below.
  Threshold-tripping; planner DQ #148 filed (proceed-as-one rationale
  pre-seeded).

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Computed mechanically
from §13 task list:

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | **2** | 7 impl tasks (Tasks 1-7; Task 0 + retro excluded). `max(0, 7-5) = 2` |
| Migrations touched | +2 each | **0** | SL-c ships zero migrations (per §4.1 + §11) |
| Crates touched | +1 each | **3** | `lemmy_api` (sponsor_liability_grace.rs + governance/mod.rs, Task 1), `lemmy_routes` (utils/scheduled_tasks.rs, Task 2), `lemmy_server` (tests/e2e.rs, Tasks 3-7) |
| `crates/lemmy_server/tests/e2e/*.rs` edits | +3 each | **15** | Tasks 3, 4, 5, 6, 7 each modify `crates/server/tests/e2e.rs`. 5 × +3 = 15 |
| New ADR-affecting decisions | +2 each | **0** | PRD §6 + §9.4 + §9.5 already settled; DQ #144-#147 resolved at clarify time; SL-c is implementation-only |
| Cargo budget peak above 6 GB | +1 per GB | **0** | Shape G — cargo runs off-box |
| Wrapped subtotal | — | **20** |  |
| Adjustment | — | **+1** | Fold-in: §4.2 watchpoint #14 (restoration stub-and-future-wire breadcrumb in §19); requires drift-stub discipline + §19 documentation overhead |
| **Total** | — | **21** | Threshold for split-DQ: `>8`. **Tripped.** |

### 5.2 Split-or-proceed DQ

Per `feedback_complexity_score_pre_split.md` + `planning.md` §5
"Decision-queue — pre-seed forward-looking OQs": planner files
**DQ #148** (`from: "planner"`, `kind: "blocker"`, `answered_by:
"planner"` self-resolved with rationale per Recipe 2) BEFORE
committing the plan. Question: "Complexity score 21 exceeds 8 — split
`v1-sponsor-liability-c` into `v1-SL-c-1` (Tasks 1-2: module +
scheduler wiring, score ~7) + `v1-SL-c-2` (Tasks 3-7 + retro: 5 e2e
tests, score ~16), or proceed as one plan?". Options: split / proceed.

**Planner observation (binding lean — proceed-as-one):** the dominant
factor is the 5 e2e edits (+15 of the +21). Splitting into c1
(module + scheduler) and c2 (5 tests) yields scores of `0 + 0 + 2 +
0 = 2` (c1: 2 impl tasks, 0 migrations, 2 crates, 0 e2e edits) and
`0 + 0 + 1 + 15 = 16` (c2: 5 impl tasks, 0 migrations, 1 crate, 5
e2e edits). The c2 score remains above 8; the split does NOT
meaningfully reduce the second sub-phase. Splitting also fragments
a logically atomic deliverable (PRD §15 row 3 names the deliverable
as one phase: "Scheduler + grace-check helper"). Splitting also
introduces a spurious phase-branch boundary at task-2-to-task-3
transition; tests cannot run until the module exists, so they would
need to be written against an unmerged module (or the c2 phase
branched from c1's phase tip, which is more topologically complex
than warranted).

Per `feedback_principles_not_rules.md`: the score is a signal, not
a hard rule. Mechanical scheduler+tests work with strong precedents
— `reputation_snapshot.rs::run_snapshot_batch` +
`appeal_window_expiry.rs::run_appeal_window_expiry_batch` ship in
production since Phase 5a/5c with no operational regret. Each test
is anchor-Edit-friendly (single fn at file end; existing
`bootstrap()` helper at `e2e.rs:767` provides users + community).
Per-task wall-clock under Sonnet 4.6 likely 6-8 min per test (Edit
+ workspace-check workflow). 5 tests × 7 min average + 2 module
tasks × 12 min = ~59 min impl-task time; plus per-task
validate-pending workflows.

Precedent: SL-a shipped at score 13 with proceed-as-one and no
operational regret per the SL-a retro. SL-b shipped at score 38 with
proceed-as-one (DQ #143). JM-e shipped at score 15 with
proceed-as-one. SL-c at 21 is below SL-b's 38 and within the
established proceed-as-one envelope.

This plan ships under the **proceed-as-one** assumption. DQ #148 is
filed with `answered_by: "planner"` (self-resolved per Recipe 2)
citing the rationale above; the advisor may overturn at plan-approval
time if the user prefers split.

---

## 6. Relationship to other v1-SL sub-phases

| Sub-phase | Status | What it ships | SL-c dependency |
|---|---|---|---|
| v1-SL-a | MERGED (PR #111, governance-v0 @ `790f6101d`) | Schema + 13 seeded keys + 5 entry-kind consts (incl. `_SPONSOR_LIABILITY_FIRED`, `_SPONSOR_LIABILITY_ESCAPED`, `_RESTORATION_COMPLETED`) + 3 CaseStatus variants + Issue #24 partial index + backfill | SL-c reads `moderation_case.{grace_expires_at, liability_escape_reason, severity, decided_at, target_person_id, community_id, status}`, `surety.{revoked_at, sponsor_id, sponsored_id}`, `sanction.{action, case_id}`, `governance_config.{job.grace_check_*, liability.grace_window_maximum_hours}`; fires `_SPONSOR_LIABILITY_FIRED` always (fire branch summary), `_SPONSOR_LIABILITY_ESCAPED` on scheduler-branch escape; reads `_RESTORATION_COMPLETED` for stubbed restoration-escape branch (never fires in SL-c) |
| v1-SL-b | IN FLIGHT (PR #119 on `phase-v1-SL-b`; SL-c plan-write awaits SL-b merge) | `revoke_endorsement` handler + DTO + route + 9 e2e tests | SL-b's revocation-branch escape fires the SAME `_SPONSOR_LIABILITY_ESCAPED` const SL-c's scheduler-branch escape fires (per registry §"v1-SL-a entry kinds"). SL-c's tests pre-seed `SponsorLiabilityPending` cases via direct DB-write, NOT via SL-b's revocation handler (cleaner isolation; SL-b's tests already cover the revocation-branch escape path). SL-c does NOT author SL-b's handler |
| **v1-SL-c (THIS PLAN)** | NOT YET CUT | Scheduler module `sponsor_liability_grace.rs` (4 fns) + clokwerk tick wiring + atomic concurrency guard + 5 e2e tests | — |
| v1-SL-d | PENDING (depends on SL-c) | `submit_jury_vote` mutation + `apply_sponsor_liability` compute/fire split + `Decided → SponsorLiabilityPending` transition + `_SPONSOR_LIABILITY_PENDING` fire-site | SL-d transitions cases to `SponsorLiabilityPending` (the state SL-c's tests pre-seed via direct DB-write). SL-c does NOT author SL-d's mutation. SL-d's compute/fire split refactor either keeps `apply_sponsor_liability` as a thin wrapper (SL-c call site remains correct) OR replaces with `compute_sponsor_liability` + `fire_sponsor_liability` (SL-c call site updates as no-op-behaviour change). Either is forward-compatible per advisor-locked posture |
| v1-SL-e | PENDING (depends on SL-c + SL-d) | Lane-wide e2e suite (full grace-window flow exercising SL-d transition + SL-c scheduler) | SL-c's tests pre-seed pending cases directly; SL-e's tests exercise the full lane (end-to-end through SL-d's transition) |
| restorative-mechanics-v1 | PENDING (separate PRD; not yet drafted) | `restoration_complete` endpoint + restoration-escape branch (defendant-initiated; admin-attested) | SL-c stubs the `EscapeStatus::Escape{reason: "restoration_completed", ...}` enum value but `evaluate_escape_conditions` never returns it. When restorative-mechanics-v1 ships the producer (the endpoint that emits `_RESTORATION_COMPLETED`), that PRD's plan adds the read-side query to `evaluate_escape_conditions` AND a corresponding e2e test |

**Cross-PRD sequencing (per PRD §15 + §17.1):**

- SL-c parallel-safe with rep-tuning-r3/r4/r5 (different files +
  different concerns).
- SL-c parallel-safe with admin-dashboard-v1 (admin-config-write
  surface; SL-c READS the config keys, doesn't write).
- SL-c unblocks SL-d (which writes the cases SL-c consumes) and
  SL-e (which exercises the full lane end-to-end).

---

## 7. Preflight guardrails inherited from prior phases

Per SL-a + SL-b retros + R1-R7 inherited via JM-* + AD-a + SL-a +
(SL-b expected) retros + DQs #144-#147 resolved at clarify time:

- **DQ #44 — Docker daemon preflight.** Enumerated in §13 Task 0
  (R5 — Probe 0).
- **DQ #144 — Two-tier ConfigCache lifecycle.** Resolved (advisor):
  outer `ConfigCache::new()` for batch-level reads; per-case
  `ConfigCache::new()` declared at top of `fire_or_escape_case` and
  threaded into `apply_sponsor_liability(... &mut cache)`. Mirrors
  `reputation_snapshot.rs:363/389` verbatim. Bound in §13 Task 1 +
  §10.1.
- **DQ #145 — Restoration-escape branch stub-only.** Resolved
  (advisor): `EscapeStatus::Escape{reason: "restoration_completed",
  ...}` enum value defined for ADR-013-style exhaustiveness
  completeness; `evaluate_escape_conditions`'s restoration check
  returns `Fire` unconditionally. Producer-side wiring deferred to
  restorative-mechanics-v1 PRD. Test #7 dropped from §13.
  Bound in §13 Task 1 + §19 Notes.
- **DQ #146 — Staleness formula.** Resolved (advisor): `threshold_hours
  = liability.grace_window_maximum_hours × job.grace_check_staleness_alert_multiplier`;
  per-case test `now - decided_at > threshold_hours`; defaults 720 ×
  2.0 = 1440h ≈ 60d. Bound in §13 Task 1 (`check_grace_staleness`
  body) + §10.4.
- **DQ #147 — Both paired canonical batch-runner mirrors cited.**
  Resolved (advisor): `reputation_snapshot.rs::run_snapshot_batch` +
  `check_snapshot_staleness` AND
  `appeal_window_expiry.rs::run_appeal_window_expiry_batch` are both
  cited in §10. Bound in §10.1, §10.2, §10.3.
- **R1 — `i64::from(...)` on `i32 ↔ i64`.** Bound in §4 watchpoint
  #13 + §13 Task 1 GOTCHAs (chrono `Duration::hours(i64)`,
  `usize::try_from(i64)` for batch_size).
- **R2 — `seed_jury_eligible_snapshots` BEFORE `admin_assign_jury` in
  tests.** Non-binding for SL-c (no jury-assembly tests; tests
  pre-seed `SponsorLiabilityPending` cases directly via
  `ModerationCaseInsertForm`).
- **R3 — struct-extension grep sweep.** Non-binding for SL-c (no
  DTO extensions; no new public types other than enum + outcome
  struct that have zero pre-existing literal-construction sites).
- **R4 — lowercase snake_case test names.** Bound in §13 Tasks 3-7.
- **R5 — pre-phase harness audit enumerates ALL probes.** Bound in
  §13 Task 0. Probes 0..14 listed (probe count higher than SL-b
  because SL-c verifies SL-b's merged state on top of SL-a's).
- **R6 — uniform `--no-deps -- -D warnings` clippy.** Encoded in
  `cargo-validate-workspace.yml:92`.
- **R7 — `cargo test --no-run -p lemmy_server --test e2e` on tasks
  touching a struct or re-export.** Encoded in
  `cargo-validate-workspace.yml:95`. Bound in Task 1 (new pub fns +
  enum + outcome struct).
- **JM-d retro §3.5 — `feedback_clippy_rerun_after_fix.md`.** Bound
  in §13 Task 1 + Task 2 GOTCHA — if module + scheduler refactor
  unmasks an `unused-mut` or `unused-imports` lint, re-run clippy
  locally before push (under Shape G this lives in the workflow).
- **JM-d retro §5 — `feedback_laptop_default_for_validate_pending.md`.**
  Phase 2 e2e local-default per advisor-orchestrator user gate (PR
  #105, 2026-04-28).
- **SL-a retro §lessons — pseudonym discipline (ADR-015).** Bound in
  §4 watchpoint #4 + §13 Task 1 (escape branch payload schema).
- **SL-a retro §lessons — registry rule pre-landed-const exemption.**
  SL-c is the fire-site for `_SPONSOR_LIABILITY_FIRED` and the
  scheduler-branch fire-site for `_SPONSOR_LIABILITY_ESCAPED`. Per
  registry rule, `(pending)` markers stay until the const has a live
  emitter; SL-c retro flips relevant markers (see §13 Task 8 retro).
- **SL-b retro lessons (expected).** Anything emerging from SL-b's
  retro (e.g. e2e fixture-mod naming convention, post-merge
  governance-log row count assertions) inherits forward into SL-c.
  SL-c plan-implement reads SL-b retro at task 0 if available.

---

## 8. Flow design

### 8.1 Before state (post-SL-b-merge expected)

The grace-window lifecycle today (post-SL-a-merge; SL-b in flight):

- SL-a Task 1 backfill creates `SponsorLiabilityPending` cases with
  `grace_expires_at = decided_at + 24h` for v0 mid-flight cases
  (PRD §11.2).
- Future SL-d's `submit_jury_vote` rewrite will create
  `SponsorLiabilityPending` cases at `Decided` time with
  severity-proportional grace windows.
- SL-b's `revoke_endorsement` handler (in flight) transitions
  `SponsorLiabilityPending → SponsorLiabilityEscaped` on revocation
  during grace.
- **No process transitions `SponsorLiabilityPending` to
  `SponsorLiabilityFired`.** Cases past their grace window remain
  pending indefinitely.
- **No process emits `sponsor_liability_fired`.** The const exists
  at `governance_log.rs:198`; no fire-site.
- **No process emits the scheduler-branch `sponsor_liability_escaped`.**
  SL-b emits the revocation-branch; SL-c will emit the
  scheduler-branch.

```
                    [SL-d future]                [SL-b in flight]
                         |                              |
                         v                              v
   Decided ─────► SponsorLiabilityPending ─────► SponsorLiabilityEscaped
                         |                              ^
                         |                              |
                         |                          (revocation)
                         |
                         |     [SL-c THIS PLAN]
                         |          |
                         |          v
                         └────► SponsorLiabilityFired (grace expired)
                                or
                                SponsorLiabilityEscaped (sponsor revoked)
```

### 8.2 After state (post-SL-c-merge)

A new clokwerk tick (default 5 min) iterates pending-grace cases:

```
[Every 5 minutes, in scheduled_tasks.rs:setup]
  scheduler.every(CTimeUnits::minutes(5)).run(...) {
    if BREHON_DISABLE_GRACE_CHECK_JOB == "1": skip
    if SPONSOR_LIABILITY_GRACE_RUNNING already: skip
    let _guard = GraceCheckRunningGuard;

    sponsor_liability_grace::run_grace_check_batch(&context).await;
    sponsor_liability_grace::check_grace_staleness(&conn, ...).await;
  }
```

The batch:

```
[run_grace_check_batch]
  read job.grace_check_batch_size (outer ConfigCache);
  query SponsorLiabilityPending cases past grace_expires_at, LIMIT batch_size;

  for each case in batch:
    [per-case run_transaction]
      let mut per_case_cache = ConfigCache::new();
      step 1: re-load case row FOR UPDATE
      step 2: re-check status → if not Pending, return Ok(())
      step 3: SELECT (action, scope) FROM sanction WHERE case_id = $id LIMIT 1
              → if None, tracing::error! + return Ok(())
      step 4: let escape_status = evaluate_escape_conditions(...)
              [returns Escape{reason: "sponsor_revoked", ...} or Fire]
      step 5: fire_or_escape_case(&mut per_case_cache, ...):
        if Escape:
          UPDATE moderation_case SET status=Escaped, liability_escape_reason=json!(...)
          governance_log::append("sponsor_liability_escaped", ...)
        if Fire:
          let count = apply_sponsor_liability(... &mut per_case_cache).await?
            [v0 helper writes per-sponsor reputation_event + sponsor_liability_applied
             entries + sponsor_liability_clamped if applicable]
          UPDATE moderation_case SET status=Fired
          governance_log::append("sponsor_liability_fired", {target_pseudonym,
                                  sponsor_count, case_id, fired_at}, ...)
    [end per-case tx]
    on per-case Err: warn!(); continue (per-case isolation)

  return Ok(GraceCheckBatchOutcome { cases_processed, fired, escaped })
```

The staleness check:

```
[check_grace_staleness]
  threshold_hours = max_grace_hours × multiplier (=720 × 2.0 = 1440)
  query MAX(SponsorLiabilityPending cases).decided_at
  if any case has now - decided_at > threshold_hours:
    tracing::error!(target: "governance::integrity", ...)
  return Ok(())
```

### 8.3 Endpoint changes

NONE. SL-c is server-internal scheduler module + clokwerk wiring.
Plan §13 must NOT include `crates/api/api_common/src/governance.rs`
edits or `crates/api/routes/src/lib.rs` edits.

---

## 9. Mandatory reading

### 9.1 Brehon design docs (P0)

- `.claude/PRPs/prds/v1-sponsor-liability.prd.md` §6 (full), §9.4,
  §9.5, §6.4, §6.3, §11.2, §12.4, §15 row 3, §17, §18, §3.1, §3.4,
  §7 (restoration interaction), §8.1 (`liability_escape_reason`
  schema)
- `.claude/PRPs/briefs/sl-c-planning-1.md` (this plan's source)
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  ADR-005, ADR-008, ADR-010, ADR-013, ADR-014, ADR-015,
  OQ-V1-SL-05, OQ-V1-SL-01

### 9.2 Codebase reads (P0 — mirror these patterns)

- `crates/api/api/src/governance/sponsor_liability.rs:1-358` —
  v0 `apply_sponsor_liability` signature + semantics; SL-c calls
  unsplit (per advisor-locked posture).
- `crates/api/api/src/governance/reputation_snapshot.rs` (paired
  canonical mirror per DQ #147):
  - `:361-407` — `run_snapshot_batch` body (outer batch-runner shape;
    config-key + outer ConfigCache).
  - `:423-459` — `check_snapshot_staleness` body (sibling staleness
    pattern).
  - `:363/389` — two-tier ConfigCache pattern (DQ #144 mirror).
- `crates/api/api/src/governance/appeal_window_expiry.rs:1-87`
  (paired canonical mirror per DQ #147) — `run_appeal_window_expiry_batch`
  body (per-row pattern, closer fit than snapshot's per-chunk for
  SL-c's per-case).
- `crates/routes/src/utils/scheduled_tasks.rs:65-91` — three sibling
  guard patterns (`REPUTATION_SNAPSHOT_RUNNING`,
  `APPEAL_WINDOW_EXPIRY_RUNNING`, plus SL-c's new
  `SPONSOR_LIABILITY_GRACE_RUNNING`).
- `crates/routes/src/utils/scheduled_tasks.rs:189-245` — snapshot
  scheduler block (canonical clokwerk + env-var disable + atomic
  guard + staleness pass).
- `crates/routes/src/utils/scheduled_tasks.rs:254-274` —
  appeal-window-expiry scheduler block (second sibling).
- `crates/api/api/src/governance/governance_log.rs:1-81` (api shim
  re-exports).
- `crates/db_schema/src/source/governance/governance_log.rs:196-199`
  — ENTRY_KIND_* const declarations SL-c uses.
- `crates/api/api/src/governance/actor_pseudonym_helper.rs` (full
  file — small) — `get_or_create` source for SL-c's
  `actor_pseudonym` derivation in escape JSONB + log payloads.
- `crates/api/api/src/governance/config.rs:925-945` — grace-window
  + grace-check default consts.
- `crates/api/api/src/governance/config.rs` — `ConfigCache::new()`,
  `get_int`, `get_float`, `Scope` API.
- `crates/db_schema_file/src/enums.rs:411-432` — 3 SponsorLiability*
  CaseStatus variants + doc-comments.
- `crates/db_schema_file/src/schema.rs:772-801` — `moderation_case`
  table columns (incl. SL-a additions at 799-800).
- `crates/db_schema_file/src/schema.rs:1266-1278` — `sanction` table
  columns.
- `crates/db_schema_file/src/schema.rs:1347+` — `surety` table
  columns.
- `crates/db_schema/src/source/governance/moderation_case.rs:13-137`
  — `ModerationCase` struct + `ModerationCaseInsertForm` (incl. SL-a
  fields at 135-136).
- `crates/db_schema/src/source/governance/sanction.rs` — `Sanction`
  struct + multiplicity (1:1 with case verified at
  `submit_jury_vote.rs:435`).
- `crates/db_schema/src/source/governance/surety.rs` — `Surety`
  struct + filter shape for `revoked_at IS NOT NULL AND revoked_at
  >= decided_at`.
- `crates/api/api/src/governance/mod.rs` — alphabetical placement
  for `pub mod sponsor_liability_grace;` (between
  `sponsor_liability` and `submit_jury_vote`).
- `crates/api/api/src/governance/admin_close_case.rs:30-32` —
  reason-validation pattern (citation-only; SL-c has no reason input).
- `crates/server/tests/e2e.rs:88-...` — `mod governance_fixtures`
  (esp. `bootstrap()` at line 767 — multi-user fixture).
- `crates/server/tests/e2e.rs:9786-...` — `mod v1_jm_e_fixtures`
  (most recent fixture-mod pattern; SL-c's `v1_sl_c_fixtures` follows).
- (Post-SL-b-merge): `mod v1_sl_b_fixtures` — closest fixture mirror
  (sponsor-liability-domain seeders).
- `crates/api/api/src/governance/submit_jury_vote.rs:435` — sanction
  insert site (1:1 multiplicity verification).

### 9.3 Rules (P0 — auto-loaded)

- `.claude/rules/decision-queue.md` — DQ schema-v2; attribution
  integrity; mid-task push; planner Recipe 2 self-resolution.
- `.claude/rules/branch-manager.md` — file-ownership boundaries; BM
  cuts `phase-v1-SL-c` AFTER SL-b merge.
- `.claude/rules/phase-branch.md` — phase-branch + PR-into-`governance-v0`
  flow.
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy`
  mandatory.
- `.claude/rules/governance-log-entry-kind-registry.md` —
  pre-landed-const exemption + retro-time marker flip discipline.
- `.claude/rules/cargo-output-capture.md` +
  `.claude/rules/no-cargo-output-paste.md` — cargo invocation
  discipline (relevant for §15.7 manual snippets).
- `.claude/rules/pre-phase-harness-audit.md` — Task 0 audit shape.

### 9.4 Lessons (P0 — bound to §13 decisions per §2 lesson list)

(See §2 "Lessons that bind §13 decisions" — full enumeration.)

### 9.5 External documentation

- diesel `for_update()` semantics (already used at
  `appeal_window_expiry.rs:50`).
- chrono `Duration::hours(i64)` API (existing usage in
  reputation_snapshot).
- clokwerk `every(...).run(...)` API (existing patterns at
  scheduled_tasks.rs).

---

## 10. Patterns to mirror

Per `feedback_advisor_watchpoint_specificity.md` + DQ #147 (paired
canonical batch-runner mirrors).

### 10.1 Outer batch runner (canonical mirror — paired)

**SOURCE A:** `crates/api/api/src/governance/reputation_snapshot.rs:361-407`
(per-chunk pattern; `run_snapshot_batch`).

**SOURCE B:** `crates/api/api/src/governance/appeal_window_expiry.rs:34-87`
(per-row pattern; `run_appeal_window_expiry_batch`).

SL-c's `run_grace_check_batch` is closer to SOURCE B (per-row → per-case
isolation) but adopts SOURCE A's two-tier ConfigCache pattern (per DQ
#144).

```rust
pub async fn run_grace_check_batch(
  context: &LemmyContext,
) -> LemmyResult<GraceCheckBatchOutcome> {
  let pool = &mut context.pool();
  let mut batch_cache = ConfigCache::new();

  let batch_size = config::get_int(
    &mut batch_cache,
    pool,
    Scope::Instance,
    "job.grace_check_batch_size",
  )
  .await?;
  let batch_size_i64: i64 = batch_size;

  let conn = &mut get_conn(pool).await?;
  let now: DateTime<Utc> = Utc::now();

  // Outer batch query — NO transaction here. Snapshot of cases past their
  // grace_expires_at; per-case tx revalidates with FOR UPDATE.
  let candidates: Vec<ModerationCase> = moderation_case::table
    .filter(moderation_case::status.eq(CaseStatus::SponsorLiabilityPending))
    .filter(moderation_case::grace_expires_at.le(Some(now)))
    .order_by(moderation_case::grace_expires_at.asc())
    .limit(batch_size_i64)
    .select(ModerationCase::as_select())
    .load(conn)
    .await?;

  let mut outcome = GraceCheckBatchOutcome {
    cases_processed: 0,
    fired: 0,
    escaped: 0,
    skipped: 0,
  };

  if candidates.is_empty() {
    info!("governance: grace-check tick — no pending cases past grace_expires_at");
    return Ok(outcome);
  }

  for case in candidates {
    let case_id = case.id;
    // Per-case run_transaction — outer batch never returns Err on per-case
    // failures (per PRD §6.3 + watchpoint #8).
    let case_outcome = conn
      .run_transaction(|conn| {
        async move {
          fire_or_escape_case_inner(conn, &case, now).await
        }
        .scope_boxed()
      })
      .await;
    match case_outcome {
      Ok(PerCaseOutcome::Fired) => outcome.fired += 1,
      Ok(PerCaseOutcome::Escaped) => outcome.escaped += 1,
      Ok(PerCaseOutcome::Skipped) => outcome.skipped += 1,
      Err(e) => {
        warn!("grace-check: case_id={case_id_int} per-case tx failed: {e}",
          case_id_int = case_id.0);
        outcome.skipped += 1;
      }
    }
    outcome.cases_processed += 1;
  }

  info!(
    "governance: grace-check tick — cases_processed={}, fired={}, escaped={}, skipped={}",
    outcome.cases_processed, outcome.fired, outcome.escaped, outcome.skipped
  );
  Ok(outcome)
}
```

**GOTCHA (R1):** `batch_size` is `i64` (`get_int` returns i64); diesel
`limit(...)` takes `i64` directly — no `as` cast.

**GOTCHA (per-case error catch):** the `match case_outcome { Err(e) =>
warn!(...) }` pattern is the strong assertion of watchpoint #8 — outer
function returns `Ok(outcome)` regardless. Per-case failures don't block
batch.

### 10.2 Per-case transaction body (canonical mirror)

**SOURCE:** `crates/api/api/src/governance/appeal_window_expiry.rs:55-78`
(per-row body pattern; SL-c's per-case body is a richer variant).

```rust
async fn fire_or_escape_case_inner(
  conn: &mut AsyncPgConnection,
  case: &ModerationCase,
  now: DateTime<Utc>,
) -> LemmyResult<PerCaseOutcome> {
  let mut per_case_cache = ConfigCache::new();
  let case_id = case.id;

  // Step 1: re-load with FOR UPDATE.
  let re_loaded: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .for_update()
    .select(ModerationCase::as_select())
    .first(conn)
    .await?;

  // Step 2: re-check status (defence against scheduler-vs-handler race).
  if re_loaded.status != CaseStatus::SponsorLiabilityPending {
    return Ok(PerCaseOutcome::Skipped);
  }

  let target_person_id = match re_loaded.target_person_id {
    Some(pid) => pid,
    None => {
      tracing::error!(
        target: "governance::integrity",
        case_id = re_loaded.id.0,
        "SponsorLiabilityPending case has NULL target_person_id"
      );
      return Ok(PerCaseOutcome::Skipped);
    }
  };
  let decided_at = match re_loaded.decided_at {
    Some(d) => d,
    None => {
      tracing::error!(
        target: "governance::integrity",
        case_id = re_loaded.id.0,
        "SponsorLiabilityPending case has NULL decided_at"
      );
      return Ok(PerCaseOutcome::Skipped);
    }
  };

  // Step 3: lookup sanction action.
  let sanction_row: Option<(SanctionAction, SanctionScope)> = sanction::table
    .filter(sanction::case_id.eq(case_id))
    .order_by(sanction::id.asc())
    .limit(1)
    .select((sanction::action, sanction::scope))
    .first::<(SanctionAction, SanctionScope)>(conn)
    .await
    .optional()?;
  let (action, _scope) = match sanction_row {
    Some(s) => s,
    None => {
      tracing::error!(
        target: "governance::integrity",
        case_id = re_loaded.id.0,
        "SponsorLiabilityPending case has zero sanction rows — v0 invariant violation; skipping"
      );
      return Ok(PerCaseOutcome::Skipped);
    }
  };

  // Step 4: evaluate escape conditions.
  let escape = evaluate_escape_conditions(
    conn,
    case_id,
    target_person_id,
    re_loaded.community_id,
    decided_at,
    &mut per_case_cache,
  )
  .await?;

  // Step 5: branch.
  match escape {
    EscapeStatus::Escape { reason, actor_pseudonym, ref_id } => {
      let escape_reason_json = json!({
        "version": 1,
        "reason": reason,
        "actor_pseudonym": actor_pseudonym,
        "endorsement_id": ref_id,
      });
      diesel::update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
        .set((
          moderation_case::status.eq(CaseStatus::SponsorLiabilityEscaped),
          moderation_case::liability_escape_reason.eq(Some(escape_reason_json)),
        ))
        .execute(conn)
        .await?;
      governance_log::append(
        &mut (&mut *conn).into(),
        ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED,
        json!({
          "case_id": case_id.0,
          "escaped_at": now,
          "reason": reason,
          "actor_pseudonym": actor_pseudonym,
          "endorsement_id": ref_id,
        }),
        Some(actor_pseudonym),
      )
      .await?;
      Ok(PerCaseOutcome::Escaped)
    }
    EscapeStatus::Fire => {
      let sponsor_count = sponsor_liability::apply_sponsor_liability(
        conn,
        target_person_id,
        case_id,
        re_loaded.community_id,
        action,
        &mut per_case_cache,
      )
      .await?;
      diesel::update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
        .set(moderation_case::status.eq(CaseStatus::SponsorLiabilityFired))
        .execute(conn)
        .await?;
      let target_pseudonym = actor_pseudonym_helper::get_or_create(
        &mut (&mut *conn).into(),
        target_person_id,
      )
      .await?;
      governance_log::append(
        &mut (&mut *conn).into(),
        ENTRY_KIND_SPONSOR_LIABILITY_FIRED,
        json!({
          "case_id": case_id.0,
          "fired_at": now,
          "target_pseudonym": target_pseudonym,
          "sponsor_count": sponsor_count,
        }),
        Some(target_pseudonym.clone()),
      )
      .await?;
      Ok(PerCaseOutcome::Fired)
    }
  }
}
```

**GOTCHA (FOR UPDATE):** the lock at step 1 + status re-check at step 2
together defend against SL-b's revocation handler racing the scheduler.

**GOTCHA (NULL target_person_id / decided_at):** v0 invariant violation
but SL-c handles gracefully — `tracing::error!` + skip. Don't crash batch.

**GOTCHA (ADR-013 — exhaustive match on EscapeStatus):** the `match
escape` enumerates `Escape{...}` AND `Fire`. No `_ =>` arm. Adding a
future variant forces a compile-time decision.

**GOTCHA (per-case ConfigCache passed to apply_sponsor_liability):** the
v0 helper at `sponsor_liability.rs:148` accepts `&mut ConfigCache`. SL-c
threads `per_case_cache` (declared at top of this fn) into the helper —
this matches DQ #144's per-case-tier semantic.

**GOTCHA (governance_log::append — Option<String> actor_pseudonym):**
the `append` shim accepts `Option<String>` for the actor; on Fire we
pass `Some(target_pseudonym)` (the case's target as the affected
person); on Escape we pass `Some(actor_pseudonym)` (the revoking sponsor
or restoration author).

### 10.3 Atomic concurrency guard pair (canonical mirror)

**SOURCE:** `crates/routes/src/utils/scheduled_tasks.rs:71-79` +
`:83-91` (two existing sibling pairs). SL-c adds a third.

```rust
// Line 92+ (after AppealWindowExpiryRunningGuard's impl Drop).
static SPONSOR_LIABILITY_GRACE_RUNNING: AtomicBool = AtomicBool::new(false);

struct GraceCheckRunningGuard;

impl Drop for GraceCheckRunningGuard {
  fn drop(&mut self) {
    SPONSOR_LIABILITY_GRACE_RUNNING.store(false, Ordering::Release);
  }
}
```

**GOTCHA (no shared static):** must be a NEW pair, not reuse of
`REPUTATION_SNAPSHOT_RUNNING` or `APPEAL_WINDOW_EXPIRY_RUNNING`.
Different concurrency domains; sharing would deadlock-couple
unrelated schedulers.

**GOTCHA (Drop on panic):** RAII `Drop` impl ensures the flag clears
on panic (not just normal return). The `let _guard =
GraceCheckRunningGuard;` line in the closure body keeps the guard
alive until the closure ends.

### 10.4 Staleness check (canonical mirror)

**SOURCE:** `crates/api/api/src/governance/reputation_snapshot.rs:423-459`
(`check_snapshot_staleness`). SL-c's `check_grace_staleness` mirrors but
the threshold formula differs per DQ #146.

```rust
pub async fn check_grace_staleness(
  conn: &mut AsyncPgConnection,
  max_grace_hours: i64,
  multiplier: f64,
  now: DateTime<Utc>,
) -> LemmyResult<()> {
  use diesel::dsl::min;

  // threshold_hours = max_grace_hours * multiplier (DQ #146).
  // Defaults: 720 * 2.0 = 1440h ≈ 60d (matches PRD §6.3 "(>60 days)").
  let threshold_hours_f = (max_grace_hours as f64) * multiplier;
  let threshold_hours_i = threshold_hours_f.round() as i64;
  let threshold = now - chrono::Duration::hours(threshold_hours_i);

  // Per-case test: now - decided_at > threshold_hours
  // (i.e. decided_at < threshold).
  let stuck_min_decided: Option<DateTime<Utc>> = moderation_case::table
    .filter(moderation_case::status.eq(CaseStatus::SponsorLiabilityPending))
    .filter(moderation_case::decided_at.lt(Some(threshold)))
    .select(min(moderation_case::decided_at))
    .first(conn)
    .await?;

  if let Some(min_decided) = stuck_min_decided {
    tracing::error!(
      target: "governance::integrity",
      decided_at = ?min_decided,
      threshold = ?threshold,
      max_grace_hours,
      multiplier,
      "sponsor_liability_grace staleness detected: at least one case has decided_at older than threshold (PRD §6.3)"
    );
  }

  Ok(())
}
```

**GOTCHA (DQ #146 — formula source):** `threshold_hours =
max_grace_hours × multiplier`. Both knobs read at scheduler tick time
(by the caller in `scheduled_tasks.rs`), passed in as parameters.
Pure observability — the function never DB-writes.

**GOTCHA (R1 — type bridge):** `max_grace_hours` is i64; multiplied by
f64 → f64; rounded to i64 via `round()` (no `round_ties_even` needed —
1440.0 is already an integer-valued f64). `chrono::Duration::hours`
takes `i64`.

**GOTCHA (anchor at decided_at):** measure case age from `decided_at`,
NOT `grace_expires_at`. The semantic is "this case has been pending so
long it's escaped notice"; `decided_at` is the canonical anchor.
`grace_expires_at` is a derived value (decided_at + severity-tier-grace);
using it would bias the threshold by the grace-tier amount.

### 10.5 Scheduler tick wiring (canonical mirror)

**SOURCE:** `crates/routes/src/utils/scheduled_tasks.rs:189-245`
(snapshot block) + `:254-274` (appeal-window-expiry block). SL-c's
block is structurally identical with sponsor-liability-grace
nomenclature.

(See §4.1's verbatim code block — single source of truth; not
duplicated here.)

**GOTCHA (env-var FIRST in closure):** per watchpoint #9, the
`std::env::var("BREHON_DISABLE_GRACE_CHECK_JOB")` check is the first
statement in the `async move` body, BEFORE `compare_exchange`.
Reversing leaks atomic-bool slots when tests set the env var.

**GOTCHA (interval read at setup-time):** clokwerk pins schedules at
registration. The `let grace_interval_minutes = ... unwrap_or(5);`
line runs once at `setup()` invocation. Config changes via
`POST /admin/config` need a server restart to take effect (per PRD
§6.4 + restart-required tunability semantic).

**GOTCHA (`u32::try_from` for clokwerk):** clokwerk's `minutes(...)`
takes `u32`. The config returns `i64`; bridge with
`u32::try_from(...).unwrap_or(5)` — defensive default if the config
value somehow exceeds u32 range (it shouldn't; PRD §10 caps at 60).

### 10.6 governance_log payload schemas (fire + escape branches)

**SOURCE:** PRD §8.1 + DQ #144/#145/#146 resolutions.

`liability_escape_reason` JSONB (written to `moderation_case` row on
escape branch):

```json
{
  "version": 1,
  "reason": "sponsor_revoked",
  "actor_pseudonym": "<revoking-sponsor-pseudonym>",
  "endorsement_id": <endorsement.id.0 (i64)>
}
```

(For the stubbed restoration branch — never fires in v1-SL-c —
the schema would be `{"version": 1, "reason": "restoration_completed",
"actor_pseudonym": "<defendant-or-admin-pseudonym>", "restoration_id":
<i64>}`.)

`sponsor_liability_escaped` log entry payload (single entry per
escape):

```json
{
  "case_id": <case.id.0>,
  "escaped_at": "<DateTime<Utc> ISO 8601>",
  "reason": "sponsor_revoked",
  "actor_pseudonym": "<revoking-sponsor-pseudonym>",
  "endorsement_id": <endorsement.id.0>
}
```

`sponsor_liability_fired` log entry payload (single SUMMARY entry on
fire — **on top of v0's per-sponsor `sponsor_liability_applied`
entries**, not a replacement):

```json
{
  "case_id": <case.id.0>,
  "fired_at": "<DateTime<Utc> ISO 8601>",
  "target_pseudonym": "<sponsee-pseudonym>",
  "sponsor_count": <usize as i64>
}
```

**GOTCHA (ADR-015 — pseudonym discipline):** `actor_pseudonym` and
`target_pseudonym` are REQUIRED string fields. NEVER write
`caller_id.0` / `from_person_id.0` / any raw `PersonId`. The
pseudonym is sourced via
`actor_pseudonym_helper::get_or_create(&mut conn.into(), person_id)`
inside the per-case tx.

**GOTCHA (layering invariant — fire branch):** the v0
`apply_sponsor_liability` at `sponsor_liability.rs:322-337` writes
ONE `sponsor_liability_applied` entry PER SPONSOR (and one
`sponsor_liability_clamped` per clamped sponsor). SL-c's summary
`sponsor_liability_fired` is a SEPARATE entry, fired AFTER
`apply_sponsor_liability` returns. Test #1 (Task 3) asserts BOTH:
the per-sponsor entries (count == active sponsor count) AND the
single summary.

### 10.7 Sanction action lookup pattern

**SOURCE:** `crates/db_schema_file/src/schema.rs:1266-1278` + verified
1:1 multiplicity at `submit_jury_vote.rs:435`.

```rust
let sanction_row: Option<(SanctionAction, SanctionScope)> = sanction::table
  .filter(sanction::case_id.eq(case_id))
  .order_by(sanction::id.asc())
  .limit(1)
  .select((sanction::action, sanction::scope))
  .first::<(SanctionAction, SanctionScope)>(conn)
  .await
  .optional()?;
```

**GOTCHA (multiplicity — verified 1:1):** at plan-write time
`submit_jury_vote.rs:435` performs exactly one `insert_into(sanction::table)`
per case (per JM-d code). The `ORDER BY id ASC LIMIT 1` is defensive
against future drift; if SL-d or later phases introduce many-to-one
sanctions, SL-c's logic still picks the canonical first.

**GOTCHA (empty-row handling):** `.optional()?` yields `Option<...>`;
`None` is logged via `tracing::error!` and skipped silently. The case
stays `SponsorLiabilityPending` for the next tick (which will also
skip; an admin must intervene to either insert a sanction or
hand-flip the status).

### 10.8 Test fixture mod shape

**SOURCE:** `crates/server/tests/e2e.rs:9786-...` (`mod
v1_jm_e_fixtures`); post-SL-b-merge the canonical mirror is
`mod v1_sl_b_fixtures`.

```rust
mod v1_sl_c_fixtures {
  use super::*;
  use chrono::{DateTime, Duration, Utc};
  use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, insert_into, update};
  use diesel_async::RunQueryDsl as AsyncRunQueryDsl;
  use lemmy_api::governance::sponsor_liability_grace::run_grace_check_batch;
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
    // Set decided_at to derive case-age (for staleness tests).
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

  // ... 5 #[tokio::test] async fn ... per Tasks 3-7 ...
}
```

**GOTCHA (R4 — test fn names):** lowercase snake_case.

**GOTCHA (e2e fixture-mod placement):** the new mod opens AFTER the
last existing fixture mod (post-SL-b-merge: `mod v1_sl_b_fixtures`;
fallback if SL-b unmerged: `mod v1_jm_e_fixtures` at line 9786 +
~1190 lines = ends near 10976). Subsequent sub-phases (SL-d, SL-e)
extend their own fixture mods after SL-c's.

**GOTCHA (BREHON_DISABLE_GRACE_CHECK_JOB):** every SL-c test sets
this env var at bootstrap — otherwise the cron tick races the test's
manual `run_grace_check_batch` invocation. Per
`scheduled_tasks.rs:198`'s pattern. The bootstrap sets it via
`std::env::set_var` BEFORE `LemmyContext` construction.

---

## 11. Files to change

### `lemmy_api` crate

- `crates/api/api/src/governance/sponsor_liability_grace.rs` — NEW
  file. Module doc-comment + 4 public async fns + `EscapeStatus` enum
  + `GraceCheckBatchOutcome` struct + `PerCaseOutcome` enum (private)
  + `fire_or_escape_case_inner` private async fn. **Task 1**.
- `crates/api/api/src/governance/mod.rs` — add `pub mod
  sponsor_liability_grace;` alphabetically AFTER `pub mod
  sponsor_liability;`. **Task 1**.

### `lemmy_routes` crate

- `crates/routes/src/utils/scheduled_tasks.rs` — add
  `SPONSOR_LIABILITY_GRACE_RUNNING` static + `GraceCheckRunningGuard`
  struct + `impl Drop` (after line 91); add new
  `scheduler.every(...).run(...)` block inside `setup()` (after the
  appeal-window-expiry block at line 274). **Task 2**.

### `lemmy_server` crate

- `crates/server/tests/e2e.rs` — append new `mod v1_sl_c_fixtures`
  AFTER the last existing fixture mod (post-SL-b-merge: after `mod
  v1_sl_b_fixtures`; fallback: after `mod v1_jm_e_fixtures` at line
  10975 on `governance-v0`). Within the mod, ship 5 tests via 5
  anchor-Edit tasks (Tasks 3-7). Task 3 ships the mod shell + helpers
  + test #1; Tasks 4-7 anchor-insert subsequent tests inside the same
  mod. **Tasks 3-7**.

### Meta files (rules + reports)

- `.claude/rules/governance-log-entry-kind-registry.md` — flip
  `(pending)` → `(active)` markers on the SL-c fire-sites:
  `_SPONSOR_LIABILITY_FIRED` row entirely;
  `_SPONSOR_LIABILITY_ESCAPED` row's SL-c portion (the SL-b portion
  was flipped by SL-b retro). **Task 8 (retro)**.
- `.claude/PRPs/reports/v1-SL-c-retro.md` — CREATE retro per
  `feedback_retro_not_report.md` +
  `feedback_four_role_retro_signals.md` +
  `feedback_retro_task_complexity_score.md`. **Task 8**.

### Files explicitly NOT touched

- `crates/api/api/src/governance/sponsor_liability.rs` — v0 helper
  stays intact through SL-c; SL-d is the compute/fire split.
- `crates/api/api/src/governance/submit_jury_vote.rs` — SL-d adds
  the `Decided → SponsorLiabilityPending` transition; SL-c does NOT
  touch.
- `crates/api/api_crud/src/governance/revoke_endorsement.rs` — SL-b
  ships; SL-c does NOT touch.
- `crates/api/api/src/governance/governance_log.rs` (api shim) — no
  re-export changes; SL-a Task 7 already exported the SL-c-needed
  consts.
- `crates/db_schema/src/source/governance/governance_log.rs` — no
  const additions; SL-a Task 7 declared.
- `crates/db_schema/src/source/governance/{moderation_case,sanction,surety,endorsement}.rs`
  — no struct changes; SL-a Task 4 added the SL-needed fields.
- `crates/db_schema_file/src/{enums,schema}.rs` — no schema changes.
- `migrations/**` — zero migrations.
- `crates/api/api/src/governance/config.rs` — no const additions; the
  SL-c-required keys shipped in SL-a Task 6.
- `crates/db_views/governance_case/src/impls.rs` — view-crate stays
  unchanged.
- `crates/api/api/src/governance/{request_appeal,admin_close_case,admin_assign_jury,reputation_snapshot,appeal_window_expiry,...}.rs`
  — already ADR-013-extended in SL-a Task 5; SL-c does not touch
  any of these handlers.
- `crates/api/api_common/src/governance.rs` — no DTO additions
  (SL-c is server-internal).
- `crates/api/routes/src/lib.rs` — no route registration (SL-c is
  server-internal).
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
  `.coderabbit.yaml` — no dep / build-config / review-config changes.
- `.github/workflows/**` — no workflow YAML changes.

---

## 12. NOT building in v1-SL-c

- **`apply_sponsor_liability` compute/fire split** — SL-d's. SL-c
  calls UNSPLIT v0 helper per advisor-locked posture (masthead of
  brief).
- **`submit_jury_vote` mutation: `Decided → SponsorLiabilityPending`
  transition** — SL-d's. SL-c's tests pre-seed
  `SponsorLiabilityPending` cases via direct DB-write.
- **`revoke_endorsement` handler** — SL-b's; in flight on PR #119
  at SL-c plan-write time.
- **`restoration/complete` endpoint + restoration-escape branch
  detection** — restorative-mechanics-v1 PRD's. SL-c stubs the
  `EscapeStatus::Escape{reason: "restoration_completed", ...}` enum
  value but `evaluate_escape_conditions` never returns it in v1-SL-c
  (per DQ #145).
- **e2e behavioural tests for the full lane** (Decided → Pending →
  Fired/Escaped through SL-d transition + SL-c scheduler) — SL-e's.
  SL-c's tests pre-seed `SponsorLiabilityPending` directly.
- **Sponsor notification UX** — out per PRD §13 OQ-V1-SL-03.
- **Step-up auth for admin-driven scheduler runs** — out per PRD
  §12.3 (v2 reservation).
- **Cross-instance federation of grace-window events** — out per
  PRD §2 OUT + ADR-014.
- **New ENTRY_KIND_*** consts** — all needed shipped in SL-a. SL-c
  fires existing consts.
- **New CaseStatus variants** — SL-a shipped 3; SL-c uses; does not
  add.
- **New `governance_config` seeds** — SL-a shipped 13; SL-c reads
  `job.grace_check_*` + `liability.grace_window_maximum_hours`;
  does not seed.
- **Schema migrations** — none; SL-a is the foundation.
- **Backfill of v0 cases** — SL-a Task 1 already backfilled per ADR-010
  won't-disadvantage rule. SL-c's first batch run picks up these
  pre-existing pending cases on a freshly-deployed v1 instance.
- **`liability.multi_sponsor_escape_rule` reading in scheduler
  context.** SL-b reads this for revocation-branch escape
  determination; SL-c's scheduler-branch escape uses a simpler
  `surety.revoked_at >= decided_at` check. Multi-sponsor escape rule
  resolution lives in `revoke_endorsement` (SL-b), not in the
  scheduler — the scheduler's escape branch fires when ANY
  surety has been revoked since decided_at, irrespective of
  community escape rule. This is the cleanest semantic for the
  scheduler: at grace-expiry time, if at least one sponsor revoked,
  the chain's already-severed by SL-b's handler at revocation time;
  the scheduler check here is a defensive double-fire guard.
  Rationale: SL-b's `revoke_endorsement` already transitioned the
  case to `Escaped` if escape conditions matched the community's
  rule; the case wouldn't reach SL-c's scheduler unless still
  `Pending`. The scheduler's escape check is therefore a fallback
  for edge cases (e.g. revocation racing the scheduler tick).
- **Multi-sponsor escape-rule unit tests** — covered by SL-b's
  test #8 (any_revocation default exercise). SL-c's tests focus on
  scheduler-tick mechanics, not escape-rule resolution.

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task** (per
`feedback_pr_per_phase.md`). Each task header carries a `[P]` marker
iff its **FILES** YAML `union(creates, modifies)` shares no path with
any other `[P]`-marked task in the same cohort. Per the YAML overlap
rule (Tasks 3-7 all `modifies: crates/server/tests/e2e.rs`), the e2e
tasks are NOT cohort-compatible; they ship serially. Tasks 1 and 2
are also serial because Task 2 imports the symbols Task 1 creates.
Task 0 (pre-flight harness audit) is **always** non-`[P]`.

> **Cohort dispatch (advisor-side):** No `[P]` cohorts in SL-c. All
> tasks dispatch serially per `advisor-orchestrator.md`. Single-task
> cohort each; cohort handover aggregation per
> `feedback_handover_trailer_cohort_propagation.md` still applies as
> a per-task `HANDOVER:` commit trailer when the next task benefits.

> **Shape G (Layer G2 push-and-exit):** §13 task bodies do NOT inline
> cargo invocations. Each task ends with a push to the worker branch;
> the impl-task subagent writes a `kind: "validate-pending"` DQ entry
> referencing `cargo-validate-workspace.yml` per
> `.claude/rules/decision-queue.md` schema-v2.

### Task 0: Pre-flight harness audit + branch verification + SL-a/SL-b state confirmation

**Goal:** verify environment + branch (`phase-v1-SL-c`) +
SL-a + SL-b schema/handlers/seeds intact.

**FILES:**

```yaml
creates: []
modifies: []
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate ALL probes):**

```bash
# Probe 0 — Docker daemon (e2e harness uses testcontainers-rs)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe -1 — submodule init (Linux/Junior worktree quirk)
git submodule status > /tmp/sl-c-task0-submodule.log 2>&1
if grep -q '^-' /tmp/sl-c-task0-submodule.log; then
  git submodule update --init --recursive > /tmp/sl-c-task0-submodule-init.log 2>&1
  echo "submodule init exit: $?"
fi

# Probe 1 — branch verification
git branch --show-current
# EXPECT: phase-v1-SL-c (BM-task cuts before Task 1)

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

# Probe 7 — SL-b merge confirmation: revoke_endorsement handler shipped
test -f crates/api/api_crud/src/governance/revoke_endorsement.rs && echo "SL-b SHIPPED" || echo "SL-b NOT YET MERGED — Task 3 anchors fall back to mod v1_jm_e_fixtures end"
# EXPECT: SL-b SHIPPED (assuming SL-b PR #119 merged before SL-c cut)

# Probe 8 — last fixture mod identification (Task 3 anchor)
rg -n '^mod v1_' crates/server/tests/e2e.rs | tail -1
# EXPECT: mod v1_sl_b_fixtures (post-SL-b-merge) OR mod v1_jm_e_fixtures (fallback)

# Probe 9 — v0 apply_sponsor_liability signature confirmation
rg -n 'pub\(crate\) async fn apply_sponsor_liability' crates/api/api/src/governance/sponsor_liability.rs
# EXPECT: 1 match at line ~142; signature: (conn, target_person_id, case_id, community_id, action: SanctionAction, cache: &mut ConfigCache) -> LemmyResult<usize>

# Probe 10 — sanction multiplicity invariant (1:1 with case)
rg -n 'insert_into\(sanction::table\)' crates/api/api crates/api/api_crud
# EXPECT: 1 match at submit_jury_vote.rs:435 (single insert per case — 1:1 multiplicity)

# Probe 11 — REPUTATION_SNAPSHOT_RUNNING + APPEAL_WINDOW_EXPIRY_RUNNING patterns confirmed
rg -n 'REPUTATION_SNAPSHOT_RUNNING|APPEAL_WINDOW_EXPIRY_RUNNING' crates/routes/src/utils/scheduled_tasks.rs | head
# EXPECT: at least 4 matches (declarations + Drop impls + compare_exchange usages)

# Probe 12 — PM-plugin-hooks-stable check (per .claude/rules/pm-plugin-hooks-stable.md)
for h in \
  local_private_message_before_create \
  local_private_message_after_create \
  local_private_message_before_update \
  local_private_message_after_update \
  federated_private_message_before_receive \
  federated_private_message_after_receive; do
  rg -q "\"$h\"" crates/ || { echo "missing hook literal: $h"; exit 1; }
done

# Probe 13 — concurrent-PR check
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("sponsor_liability_grace\\.rs|scheduled_tasks\\.rs|governance/mod\\.rs|tests/e2e\\.rs")) | {number, title, headRefName}'
# EXPECT: empty output (no concurrent PRs touching SL-c-owned files)

# Probe 14 — Shape G workflow YAMLs accessible (yamllint may not be installed; soft-fail)
yamllint .github/workflows/cargo-validate-workspace.yml \
         .github/workflows/cargo-test-e2e.yml > /tmp/sl-c-task0-yamllint.log 2>&1 || \
         echo "yamllint not installed or warnings — non-blocking; advisor verifies workflow shape pre-merge"

# Probe 15 — DQ pending status (advisory)
python3 -c "import json; d=json.load(open('.claude/decision-queue.json')); print('pending:', [(e['id'], e.get('kind'), e.get('from')) for e in d.get('pending',[])])"
```

**EXPECT:** Probes 0..13 exit 0 (or, for Probe 0/1, exit 1 with
explicit STOP). Probes 14-15 are informational.

**No commit at Task 0** — verification only.

### Task 1: Create `sponsor_liability_grace.rs` module + module wiring

**ACTION:** in `crates/api/api/src/governance/`, create new file
`sponsor_liability_grace.rs` with the module doc-comment + 4 public
async fns (`run_grace_check_batch`, `evaluate_escape_conditions`,
`fire_or_escape_case`, `check_grace_staleness`) + `EscapeStatus`
public enum + `GraceCheckBatchOutcome` public struct +
`PerCaseOutcome` private enum + `fire_or_escape_case_inner` private
fn per §10.1, §10.2, §10.4. In
`crates/api/api/src/governance/mod.rs`, add `pub mod
sponsor_liability_grace;` alphabetically AFTER `pub mod
sponsor_liability;` (verified at `mod.rs:35`).

**FILES:**

```yaml
creates:
  - crates/api/api/src/governance/sponsor_liability_grace.rs
modifies:
  - crates/api/api/src/governance/mod.rs   # add pub mod sponsor_liability_grace; alphabetically
```

**IMPLEMENT (file 1 of 2):** in
`crates/api/api/src/governance/sponsor_liability_grace.rs`, write
the full module body per §10.1 (`run_grace_check_batch`) + §10.2
(`fire_or_escape_case` + `fire_or_escape_case_inner`) + §10.4
(`check_grace_staleness`) + the `evaluate_escape_conditions` body.

Module doc-comment shape (mirror
`appeal_window_expiry.rs:1-7` + `reputation_snapshot.rs:1-...`):

```rust
//! Sponsor-liability grace-window scheduler module.
//!
//! Per PRD §6 + §9.4. Runs every `job.grace_check_interval_minutes`
//! (default 5): finds `SponsorLiabilityPending` cases past their
//! `grace_expires_at` and transitions each to `SponsorLiabilityFired`
//! (apply_sponsor_liability fires) or `SponsorLiabilityEscaped`
//! (sponsor revoked since decided_at). Per-case `run_transaction`
//! isolation per PRD §6.2 + Watch 9 / `feedback_multi_write_handlers_need_transactions.md`.
//!
//! ## Watch 10 — PII discipline (ADR-015)
//!
//! Every governance-log payload field naming a person uses
//! `*_pseudonym` (string sourced via
//! `actor_pseudonym_helper::get_or_create`), NEVER raw `PersonId`.
//! `liability_escape_reason` JSONB written to `moderation_case`
//! follows the same rule.
//!
//! ## Compute/fire posture (advisor-locked, v1-SL-c masthead)
//!
//! This module calls **unsplit v0** `apply_sponsor_liability` from
//! the fire branch (per
//! `crates/api/api/src/governance/sponsor_liability.rs:142`). The
//! PRD §9.1 compute/fire split is SL-d's deliverable. SL-c's call
//! site is forward-compatible: when SL-d ships and the helper
//! either remains as a thin wrapper or is replaced by
//! `compute_sponsor_liability` + `fire_sponsor_liability`, the
//! call site updates as a no-op-behaviour change.
//!
//! ## Restoration-escape branch — STUB-ONLY in v1-SL-c (DQ #145)
//!
//! `EscapeStatus::Escape{reason: "restoration_completed", ...}` is
//! defined as a documented future-wire branch but
//! `evaluate_escape_conditions` NEVER constructs it. The
//! `restoration_completed` log entry has zero producers in v1-SL-c
//! (verified by registry rule). When restorative-mechanics-v1 PRD
//! ships the producer endpoint, that PRD's plan adds the read-side
//! query to `evaluate_escape_conditions` AND a corresponding e2e
//! test.
```

Use block:

```rust
use chrono::{DateTime, Utc};
use diesel::{
  ExpressionMethods,
  OptionalExtension,
  QueryDsl,
  SelectableHelper,
  dsl::min,
};
use diesel_async::{
  AsyncPgConnection, RunQueryDsl,
  scoped_futures::ScopedFutureExt,
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{
  newtypes::{CommunityId, ModerationCaseId, PersonId},
  source::governance::moderation_case::ModerationCase,
};
use lemmy_db_schema_file::enums::{CaseStatus, SanctionAction, SanctionScope};
use lemmy_db_schema_file::schema::{moderation_case, sanction, surety};
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;
use tracing::{info, warn};

use crate::governance::{
  actor_pseudonym_helper,
  config::{self, ConfigCache, Scope},
  governance_log::{
    self, ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED, ENTRY_KIND_SPONSOR_LIABILITY_FIRED,
  },
  sponsor_liability,
};
```

Then the public types:

```rust
/// Outcome of a single `run_grace_check_batch` invocation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GraceCheckBatchOutcome {
  pub cases_processed: usize,
  pub fired: usize,
  pub escaped: usize,
  pub skipped: usize,
}

/// Outcome of one case's escape-condition evaluation.
///
/// `Fire` (default) means apply_sponsor_liability runs; `Escape`
/// means the case escapes liability (no sponsor reputation_event
/// rows; `liability_escape_reason` JSONB recorded). The
/// `reason` string discriminates branches: `"sponsor_revoked"`
/// (SL-c scheduler), or `"restoration_completed"` (future
/// restorative-mechanics-v1 PRD; stub-only in v1-SL-c).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscapeStatus {
  /// Escape branch fired. `reason` is one of the documented values.
  Escape {
    reason: String,
    actor_pseudonym: String,
    /// `endorsement.id.0` for `sponsor_revoked`;
    /// future `restoration.id.0` for `restoration_completed`.
    ref_id: i64,
  },
  /// Default — fire branch (apply_sponsor_liability).
  Fire,
}

/// Private outcome enum used by `fire_or_escape_case_inner` to
/// communicate per-case result back to `run_grace_check_batch`'s
/// counter increments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PerCaseOutcome {
  Fired,
  Escaped,
  Skipped,
}
```

Then the four public async fns + private helper, per §10.1, §10.2,
§10.4 verbatim plus an `evaluate_escape_conditions` body:

```rust
pub async fn evaluate_escape_conditions(
  conn: &mut AsyncPgConnection,
  case_id: ModerationCaseId,
  target_person_id: PersonId,
  _community_id: Option<CommunityId>,
  decided_at: DateTime<Utc>,
  _cache: &mut ConfigCache,
) -> LemmyResult<EscapeStatus> {
  // Branch 1: any active surety for target_person_id with
  // revoked_at >= decided_at? → Escape{reason: "sponsor_revoked"}.
  //
  // We join surety to endorsement to derive the actor pseudonym
  // (the revoking sponsor's PersonId, then via actor_pseudonym_helper).
  // surety.revoked_at doesn't carry the actor; use surety.sponsor_id
  // as the actor source.
  let revoked_surety: Option<(PersonId, DateTime<Utc>)> = surety::table
    .filter(surety::sponsored_id.eq(target_person_id))
    .filter(surety::revoked_at.is_not_null())
    .filter(surety::revoked_at.ge(Some(decided_at)))
    .order_by(surety::revoked_at.asc())
    .limit(1)
    .select((surety::sponsor_id, surety::revoked_at.assume_not_null()))
    .first::<(PersonId, DateTime<Utc>)>(conn)
    .await
    .optional()?;

  if let Some((revoker_id, _revoked_at)) = revoked_surety {
    let actor_pseudonym = actor_pseudonym_helper::get_or_create(
      &mut (&mut *conn).into(),
      revoker_id,
    )
    .await?;
    // ref_id: best-effort endorsement.id lookup. If multiple
    // endorsements exist (sponsor → sponsee at multiple
    // community scopes), pick the first by id ascending. If none
    // (defensive — a surety implies an endorsement existed),
    // ref_id stays 0; downstream consumers handle gracefully.
    use lemmy_db_schema_file::schema::endorsement;
    let endorsement_id: Option<i32> = endorsement::table
      .filter(endorsement::from_person_id.eq(revoker_id))
      .filter(endorsement::to_person_id.eq(target_person_id))
      .order_by(endorsement::id.asc())
      .limit(1)
      .select(endorsement::id)
      .first::<i32>(conn)
      .await
      .optional()?;
    let ref_id = endorsement_id.map(i64::from).unwrap_or(0);
    return Ok(EscapeStatus::Escape {
      reason: "sponsor_revoked".to_string(),
      actor_pseudonym,
      ref_id,
    });
  }

  // Branch 2 (STUB-ONLY in v1-SL-c per DQ #145):
  // Restoration completed during the grace window?
  // The producer endpoint `restoration/complete` does not exist
  // in v1; ENTRY_KIND_RESTORATION_COMPLETED has zero emit-sites.
  // When restorative-mechanics-v1 ships the producer, this branch
  // queries governance_log for a `restoration_completed` entry
  // for target_person_id between decided_at and now, returning
  // EscapeStatus::Escape { reason: "restoration_completed", ... }.
  //
  // Until then, fall through to Fire.
  let _ = case_id;  // suppress unused-var on stub branch

  Ok(EscapeStatus::Fire)
}

pub async fn fire_or_escape_case(
  conn: &mut AsyncPgConnection,
  case: ModerationCase,
  status: EscapeStatus,
  _cache: &mut ConfigCache,
) -> LemmyResult<()> {
  // Public entry; thin wrapper that runs the inner body inside a
  // run_transaction. Used by callers OTHER than run_grace_check_batch
  // (e.g. tests, future admin-driven manual runs). The batch loop
  // uses fire_or_escape_case_inner directly inside its per-case
  // run_transaction closure.
  let now: DateTime<Utc> = Utc::now();
  conn
    .run_transaction(|conn| {
      async move {
        // Apply the caller-evaluated status without re-evaluating;
        // this is for caller-driven invocations.
        match status {
          EscapeStatus::Escape { reason, actor_pseudonym, ref_id } => {
            // (same body as fire_or_escape_case_inner's escape arm —
            // refactored to share via a helper if needed; inline
            // mirror for clarity)
            let escape_reason_json = json!({
              "version": 1,
              "reason": reason,
              "actor_pseudonym": actor_pseudonym,
              "endorsement_id": ref_id,
            });
            diesel::update(moderation_case::table.filter(moderation_case::id.eq(case.id)))
              .set((
                moderation_case::status.eq(CaseStatus::SponsorLiabilityEscaped),
                moderation_case::liability_escape_reason.eq(Some(escape_reason_json)),
              ))
              .execute(conn)
              .await?;
            governance_log::append(
              &mut (&mut *conn).into(),
              ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED,
              json!({
                "case_id": case.id.0,
                "escaped_at": now,
                "reason": reason,
                "actor_pseudonym": actor_pseudonym,
                "endorsement_id": ref_id,
              }),
              Some(actor_pseudonym),
            )
            .await?;
            Ok(())
          }
          EscapeStatus::Fire => {
            // Fire branch (caller-driven) — assumes the caller has
            // already done sanction-action lookup. For safety,
            // re-derive here.
            // (Implementation note: in v1-SL-c this public entry is
            // not exercised by tests — they call run_grace_check_batch
            // directly. Keeping the body for future callers; the
            // happy-path is run_grace_check_batch → fire_or_escape_case_inner.)
            Err(LemmyErrorType::Unknown(
              "fire_or_escape_case public entry: Fire branch requires sanction-action context; call run_grace_check_batch instead".to_string()
            ).into())
          }
        }
      }
      .scope_boxed()
    })
    .await
}
```

(Full body for `run_grace_check_batch`, `fire_or_escape_case_inner`,
`check_grace_staleness` per §10.1, §10.2, §10.4 verbatim. The
`fire_or_escape_case` public entry is a thin wrapper; the
batch-driven path uses the private `_inner` directly inside its own
run_transaction closure for cleaner per-case isolation.)

**IMPLEMENT (file 2 of 2):** in
`crates/api/api/src/governance/mod.rs`, after line 35 (`pub mod
sponsor_liability;`), insert:

```rust
pub mod sponsor_liability_grace;
```

The line lands between `sponsor_liability` and `submit_jury_vote`
(alphabetical).

**MIRROR:** §10.1, §10.2, §10.3 (atomic guard — Task 2 scope), §10.4,
§10.6, §10.7. Canonical full-file mirrors:
`crates/api/api/src/governance/reputation_snapshot.rs:361-459`,
`crates/api/api/src/governance/appeal_window_expiry.rs:1-87`,
`crates/api/api/src/governance/sponsor_liability.rs:142-358`.

**GOTCHA (R1 — i64 typing):** `batch_size: i64` (config `get_int`
returns i64); diesel `limit(...)` accepts i64 directly. `chrono::Duration::hours`
takes i64. `max_grace_hours * multiplier` arithmetic is `i64 → f64
→ i64` via `round()`.

**GOTCHA (per-case run_transaction inside outer for-loop):** the
outer `run_grace_check_batch` does NOT open a transaction. Each
iteration of the for-loop opens its own via `conn.run_transaction`.
The closure body is the per-case body; on `Err`, the tx rolls back
(case stays Pending) and the warn-log fires; outer continues.

**GOTCHA (ADR-013 exhaustive match on EscapeStatus):** the `match
escape` arms `Escape{...}` AND `Fire` only. NO `_ =>` arm. Adding a
future variant (e.g. an actual `restoration_completed` arm if
restorative-mechanics-v1 lands and adds it) forces a compile-time
decision.

**GOTCHA (DQ #144 — two-tier ConfigCache):** outer cache at top of
`run_grace_check_batch` (used for batch-size + staleness multiplier
reads at scheduler-tick time). Per-case cache at top of
`fire_or_escape_case_inner`, threaded into
`apply_sponsor_liability(... &mut per_case_cache)`. Do NOT share one
cache across distinct `run_transaction` closures.

**GOTCHA (DQ #145 — restoration-escape stub):** the
`evaluate_escape_conditions` function body has the comment block
explaining the future branch + the `let _ = case_id;` hint to
suppress unused-var lints. The function literally returns `Fire` if
no surety revocation matches. NEVER construct
`EscapeStatus::Escape{reason: "restoration_completed", ...}` in
v1-SL-c.

**GOTCHA (clippy rerun per `feedback_clippy_rerun_after_fix.md`):**
new module + new pub fns may trip `unused-imports` lint on
intermediate iterations. The cargo-validate-workspace workflow's
clippy step catches; if dispatch refactor unmasks lint, re-run
clippy locally before push (or let workflow re-run on the next
push).

**GOTCHA (R7 — struct/fn additions affect re-exports):** the new
`pub` fns + types are visible to downstream crates via the
`crates/api/api/src/governance/mod.rs` `pub mod` declaration. The
workspace-check workflow's `cargo test --no-run -p lemmy_server
--test e2e` step (line 95) catches any cross-crate compile
regression.

**GOTCHA (`Selectable` derive on ModerationCase):** the struct at
`crates/db_schema/src/source/governance/moderation_case.rs:13-94`
already derives `Identifiable, Queryable, Selectable` per
`#[cfg_attr(feature = "full", derive(...))]` line 15. SL-c's batch
query uses `ModerationCase::as_select()` — works.

**GOTCHA (cargo-features parity):** the new module + types live
behind no `#[cfg(feature = ...)]` gate. They compile in both
`--features full` and feature-less builds. The `crate::governance`
imports are stable across feature flags.

**Push and exit (Shape G):** push to origin/`junior/<task-slug>`;
impl-task subagent writes `kind: "validate-pending"` DQ entry
capturing `workflow_run_id` of `cargo-validate-workspace.yml`.

**COMMIT MESSAGE:** `feat(v1-SL-c): create sponsor_liability_grace module + 4 pub fns + module wiring (task 1)`

### Task 2: Wire scheduler tick block + atomic concurrency guard pair in `scheduled_tasks.rs`

**ACTION:** in `crates/routes/src/utils/scheduled_tasks.rs`, add
the third sibling concurrency-guard pair
(`SPONSOR_LIABILITY_GRACE_RUNNING` static + `GraceCheckRunningGuard`
struct + `impl Drop`) after line 91 (after
`AppealWindowExpiryRunningGuard`'s impl Drop), AND add a new
`scheduler.every(...).run(...)` block inside `setup()` after the
appeal-window-expiry block at line 274, mirroring the snapshot block
at lines 189-245.

**FILES:**

```yaml
creates: []
modifies:
  - crates/routes/src/utils/scheduled_tasks.rs   # add 3rd guard pair + new clokwerk tick block calling run_grace_check_batch + check_grace_staleness
```

**IMPLEMENT (file 1 of 1):** in
`crates/routes/src/utils/scheduled_tasks.rs`:

1. **Module-scope concurrency guard pair** — anchor at line 91 (end
   of `AppealWindowExpiryRunningGuard`'s `impl Drop` block). Insert
   immediately after line 91:

   ```rust

   // Concurrency guard for the 5-minute Brehon sponsor-liability
   // grace-check tick. Mirrors REPUTATION_SNAPSHOT_RUNNING +
   // RunningGuard (lines 71-79) and APPEAL_WINDOW_EXPIRY_RUNNING +
   // AppealWindowExpiryRunningGuard (lines 83-91).
   static SPONSOR_LIABILITY_GRACE_RUNNING: AtomicBool = AtomicBool::new(false);

   struct GraceCheckRunningGuard;

   impl Drop for GraceCheckRunningGuard {
     fn drop(&mut self) {
       SPONSOR_LIABILITY_GRACE_RUNNING.store(false, Ordering::Release);
     }
   }
   ```

2. **Scheduler tick block** — anchor at the end of the
   appeal-window-expiry block. Use `grep -n
   '"BREHON_DISABLE_APPEAL_WINDOW_JOB"' crates/routes/src/utils/scheduled_tasks.rs`
   to confirm the appeal-window block's location at task-time
   (current: line 258); the new block lands after the matched
   block's closing `});`. Verified at plan-write time: appeal-window
   block ends at line 274 (the final `});` of that
   `scheduler.every(CTimeUnits::hour(1)).run(...)`).

   Insert immediately after that `});`:

   ```rust

   // Brehon governance v1: sponsor-liability grace-check tick.
   // Interval is read from `job.grace_check_interval_minutes` at
   // scheduler setup (default 5). Find SponsorLiabilityPending cases
   // past their grace_expires_at and transition them to Fired or
   // Escaped per PRD §6.1 + §6.2.
   //
   // Restart-required tunability: clokwerk schedules pin at
   // registration. Flipping `job.grace_check_interval_minutes` via
   // POST /admin/config takes effect at next server restart.
   // Mirrors v0 reputation-snapshot precedent (15-min interval
   // hardcoded at scheduler setup).
   //
   // Disabled in tests via BREHON_DISABLE_GRACE_CHECK_JOB=1
   // (mirrors BREHON_DISABLE_SNAPSHOT_JOB pattern at line 197).
   //
   // Concurrency guard mirrors REPUTATION_SNAPSHOT_RUNNING /
   // APPEAL_WINDOW_EXPIRY_RUNNING. After the batch tick, a sibling
   // staleness pass emits tracing::error! per stuck case (PRD §6.3).
   let context_grace = context.reset_request_count();
   let grace_pool = &mut context.pool();
   let grace_interval_minutes_i64: i64 = lemmy_api::governance::config::get_int(
     &mut lemmy_api::governance::config::ConfigCache::new(),
     grace_pool,
     lemmy_api::governance::config::Scope::Instance,
     "job.grace_check_interval_minutes",
   )
   .await
   .unwrap_or(5);
   let grace_interval_minutes: u32 =
     u32::try_from(grace_interval_minutes_i64).unwrap_or(5);
   scheduler.every(CTimeUnits::minutes(grace_interval_minutes)).run(move || {
     let context = context_grace.reset_request_count();
     async move {
       // Watchpoint #9: env-var check FIRST in closure body. Reversing
       // means tests that set BREHON_DISABLE_GRACE_CHECK_JOB still
       // consume an atomic-bool slot, leaking guards.
       if std::env::var("BREHON_DISABLE_GRACE_CHECK_JOB").as_deref() == Ok("1") {
         return;
       }
       if SPONSOR_LIABILITY_GRACE_RUNNING
         .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
         .is_err()
       {
         warn!("sponsor_liability_grace: previous batch still running, skipping this tick");
         return;
       }
       let _guard = GraceCheckRunningGuard;
       lemmy_api::governance::sponsor_liability_grace::run_grace_check_batch(&context)
         .await
         .inspect_err(|e| warn!("Failed to run grace_check batch: {e}"))
         .ok();

       // Staleness pass after the batch (per PRD §6.3 + DQ #146).
       // Mirrors snapshot pattern at scheduled_tasks.rs:213-244.
       let staleness_pool = &mut context.pool();
       let mut staleness_cache =
         lemmy_api::governance::config::ConfigCache::new();
       let max_grace_hours = lemmy_api::governance::config::get_int(
         &mut staleness_cache,
         staleness_pool,
         lemmy_api::governance::config::Scope::Instance,
         "liability.grace_window_maximum_hours",
       )
       .await
       .unwrap_or(720);
       let multiplier = lemmy_api::governance::config::get_float(
         &mut staleness_cache,
         staleness_pool,
         lemmy_api::governance::config::Scope::Instance,
         "job.grace_check_staleness_alert_multiplier",
       )
       .await
       .unwrap_or(2.0);
       match get_conn(staleness_pool).await {
         Ok(mut conn) => {
           if let Err(e) =
             lemmy_api::governance::sponsor_liability_grace::check_grace_staleness(
               &mut conn,
               max_grace_hours,
               multiplier,
               Utc::now(),
             )
             .await
           {
             warn!("grace staleness check failed: {e}");
           }
         }
         Err(e) => warn!("grace staleness check: get_conn failed: {e}"),
       }
     }
   });
   ```

**IMPORTS:** verify `AtomicBool`, `Ordering`, `Utc`, `CTimeUnits`,
`get_conn`, `warn` are already in scope (confirmed at
`scheduled_tasks.rs:3-62`). No new imports required.

**MIRROR:** §10.3 (guard pattern), §10.5 (clokwerk block).
`crates/routes/src/utils/scheduled_tasks.rs:71-79` (snapshot guard
pair), `:83-91` (appeal-window-expiry guard pair), `:189-245`
(snapshot scheduler block), `:254-274` (appeal-window-expiry
scheduler block).

**GOTCHA (Watchpoint #3 — distinct guard pair):** must add a NEW
static + struct, NOT reuse `REPUTATION_SNAPSHOT_RUNNING` or
`APPEAL_WINDOW_EXPIRY_RUNNING`. Different concurrency domains;
sharing would deadlock-couple unrelated schedulers.

**GOTCHA (Watchpoint #9 — env-var FIRST):** the
`std::env::var("BREHON_DISABLE_GRACE_CHECK_JOB")` check is the FIRST
statement in the `async move` body, BEFORE `compare_exchange`.

**GOTCHA (clokwerk pinning):** the `get_int` call for
`grace_interval_minutes` runs ONCE at `setup()` invocation. The
`scheduler.every(...)` consumes a `u32` and pins the schedule.
Config changes via `POST /admin/config` need a server restart to
take effect (per PRD §6.4).

**GOTCHA (`u32::try_from(i64)`):** clokwerk's `minutes(...)` takes
`u32`. `unwrap_or(5)` on the conversion is the same defensive
default as `unwrap_or(5)` on `get_int` failure — both fall back to
the PRD default.

**GOTCHA (Watchpoint #5 — staleness reads INSIDE closure):** the
`max_grace_hours` + `multiplier` reads happen INSIDE the closure
body (per-tick), not at `setup()` time. Both knobs are per-tick
mutable per PRD §6.4 ("Yes — read per-staleness-check"). The
interval-minutes is the only knob that's restart-required.

**GOTCHA (`Selectable` cross-feature):** the `lemmy_api::governance::sponsor_liability_grace`
import path resolves correctly because Task 1 wired the module via
`pub mod sponsor_liability_grace;` in `governance/mod.rs`. If Task 1's
`pub mod` line is missing, this Task 2 push fails with "unresolved
module" — the validate-pending workflow catches.

**GOTCHA (R7 propagation):** new module + scheduler block are
visible in the `lemmy_routes` crate. `cargo test --no-run` step in
the workspace-check workflow validates cross-crate compile.

**GOTCHA (clippy rerun per `feedback_clippy_rerun_after_fix.md`):**
new clokwerk block may trip `clippy::redundant-closure-call` or
similar; let workflow run identify; re-run on next push if needed.

**Push and exit (Shape G):** push to origin/`junior/<task-slug>`;
impl-task subagent writes `kind: "validate-pending"` DQ entry.

**COMMIT MESSAGE:** `feat(v1-SL-c): wire sponsor_liability_grace scheduler block + atomic concurrency guard (task 2)`

### Task 3: e2e test #1 — Fire path (expired pending case, no escape conditions) + open `mod v1_sl_c_fixtures` shell + helpers

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
- Fallback: if SL-b is unmerged at task-time (Probe 7 failed), the
  new mod opens AFTER the closing `}` of `mod v1_jm_e_fixtures`
  (line 9786 + ~1190 lines of fixture body).

**Content to append:**

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

**MIRROR:** §10.1, §10.2, §10.6 (fire-branch governance_log payload),
§10.8 (fixture mod shape). Adjacent test mod:
`crates/server/tests/e2e.rs:9786-...` (`mod v1_jm_e_fixtures`); or
post-SL-b: `mod v1_sl_b_fixtures` (the immediate predecessor).

**GOTCHA (R4):** test fn name lowercase snake_case.

**GOTCHA (BREHON_DISABLE_GRACE_CHECK_JOB):** test bootstrap sets
this env var before LemmyContext construction. Mirror SL-b test
infrastructure if present; fallback to inline `std::env::set_var`
in test body before bootstrap.

**GOTCHA (Watchpoint #5 — both kinds of log entries):** the fire
branch emits BOTH the per-sponsor `sponsor_liability_applied`
entries (from v0 `apply_sponsor_liability` at
`sponsor_liability.rs:322-337` per sponsor) AND the SL-c summary
`sponsor_liability_fired` entry. Test asserts both:
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
this Task 3 ships the `mod v1_sl_c_fixtures` shell + 2 helpers + 1
test in ONE Edit at file end. Tasks 4-7 anchor-insert AFTER this
task's test body, INSIDE the same mod. Do NOT bulk-edit multiple
tests in one Edit.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-c): e2e test #1 — fire path expired pending case + mod v1_sl_c_fixtures shell (task 3)`

### Task 4: e2e test #2 — Escape path (sponsor revoked between decided_at and now)

**ACTION:** anchor-insert
`grace_check_escapes_case_when_sponsor_revoked_after_decided_at`
test fn inside `mod v1_sl_c_fixtures`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test #2 inside mod v1_sl_c_fixtures
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 3's test fn,
inside the same mod block (the closing `}` of the mod still wraps
this test).

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

**MIRROR:** §10.2 escape-branch arm; §10.6 escape JSONB schema;
§4 watchpoint #4.

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
Without this, `ref_id` falls back to 0 (defensive default in
implementation).

**GOTCHA (revoked_at semantic):** `surety.revoked_at = now() - 1h`
is AFTER `decided_at = now() - 24h` AND BEFORE `now()` — both bounds
required by `evaluate_escape_conditions`'s filter
(`revoked_at.is_not_null() AND revoked_at >= decided_at`).

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-c): e2e test #2 — escape branch sponsor revoked since decided_at (task 4)`

### Task 5: e2e test #3 — No-op (future grace_expires_at)

**ACTION:** anchor-insert
`grace_check_no_op_when_grace_expires_at_in_future` test fn inside
`mod v1_sl_c_fixtures`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test #3 inside mod v1_sl_c_fixtures
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 4's test fn:

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

**MIRROR:** §10.1 batch-query filter (`grace_expires_at.le(Some(now))`);
PRD §6.1.

**GOTCHA (boundary semantic):** the batch query uses `.le(Some(now))`
(less-than-or-equal). A case with `grace_expires_at == now()` exactly
would fire; +2 hours is comfortably outside the window.

**GOTCHA (no spurious side effects):** the no-op assertion is the
strong signal — verifying that an unselected case has zero
side-effects across reputation_event, governance_log, and
moderation_case row mutation.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-c): e2e test #3 — no-op for future grace_expires_at (task 5)`

### Task 6: e2e test #4 — Per-case isolation (one bad case doesn't block batch)

**ACTION:** anchor-insert
`grace_check_per_case_isolation_skips_bad_case_processes_good_case`
test fn inside `mod v1_sl_c_fixtures`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test #4 inside mod v1_sl_c_fixtures
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 5's test fn:

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
    //     ZERO sanction rows (malformed — empty-sanction edge case
    //     per §10.7). Force this by passing sanction_action: None to
    //     seed_pending_case.
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

**MIRROR:** §10.1 outer batch error-catch pattern (`match
case_outcome { Err(e) => warn!(...) }`); §10.7 empty-sanction
handling.

**GOTCHA (Watchpoint #8):** the strong assertion is "outer batch
returned Ok(...) AND iteration continued past case_b". Without the
per-case error-catch in the outer for-loop, case_b's empty-sanction
warn would have stopped the iteration. The test asserts case_a's
fire happened — proves continuation.

**GOTCHA (ordering — case_b first):** the batch query sorts by
`grace_expires_at ASC`, so seed case_b's expired_at slightly EARLIER
than case_a's (e.g. `now() - 2 minutes` vs `now() - 1 minute`). This
ensures case_b is processed FIRST; if case_b's failure stopped the
loop, case_a would never fire — that's the regression this test
catches.

**GOTCHA (silent skip semantic):** case_b stays Pending — admin must
intervene (insert a sanction or hand-flip status). Future ticks will
also skip case_b. This is the design semantic per §10.7.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-c): e2e test #4 — per-case isolation skips bad case processes good case (task 6)`

### Task 7: e2e test #5 — Batch size respects config

**ACTION:** anchor-insert
`grace_check_batch_size_config_caps_iteration` test fn inside
`mod v1_sl_c_fixtures`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test #5 inside mod v1_sl_c_fixtures (closes the mod)
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 6's test fn.
This is the LAST test in `mod v1_sl_c_fixtures`; the closing `}` of
the mod follows this test.

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

**MIRROR:** §10.1 batch-query `limit(batch_size_i64)` filter;
PRD §6.4 batch_size knob.

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
(Confirm exact `governance_config` schema at task-time —
`crates/db_schema_file/src/schema.rs` has the table; the on-conflict
keys match the existing seed pattern.)

**GOTCHA (sort-order determinism):** the batch query's `ORDER BY
grace_expires_at ASC` means the 5 cases must have DISTINCT
`grace_expires_at` values for predictable ordering. Seed each with
`grace_expires_at = now() - Duration::minutes(N)` for N in 5..0
(so case 0 expires first, etc.).

**GOTCHA (config caching):** the outer ConfigCache in
`run_grace_check_batch` is fresh per call; the second/third
invocations see the updated config row (no cache staleness across
calls).

**GOTCHA (closing the mod):** this test is the LAST inside `mod
v1_sl_c_fixtures`. The closing `}` of the mod follows immediately
after this test's `Ok(())` line. Future SL-d/SL-e fixture mods open
AFTER `mod v1_sl_c_fixtures` closes.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-c): e2e test #5 — batch_size config caps iteration (task 7)`

### Task 8: Retro

**Goal:** author retro per `feedback_retro_not_report.md` +
`feedback_four_role_retro_signals.md` +
`feedback_retro_task_complexity_score.md`. Flip
`(pending) → (active)` markers on registry §"v1-SL-a entry kinds"
for SL-c's fire-sites: `_SPONSOR_LIABILITY_FIRED` row entirely;
`_SPONSOR_LIABILITY_ESCAPED` row's SL-c portion (the SL-b portion
was flipped by SL-b retro).

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-SL-c-retro.md
modifies:
  - .claude/rules/governance-log-entry-kind-registry.md
```

**IMPLEMENT (file 1 of 2):** in
`.claude/PRPs/reports/v1-SL-c-retro.md`, write the retro per the
canonical 4-role format (Advisor / Planning / Impl / BM). Include:

- §1 Summary: SL-c shipped (module + scheduler wiring + 5 e2e tests
  + retro). Story status — Story 1 ✓ / Story 2 ✓ / Story 3 ✓.
- §2 Per-role signals (4 H2 sections; each lists "what worked" +
  "what surprised us" + "what should change next").
- §3 Carry-forward — list of items SL-d/SL-e/restorative-mechanics-v1
  should know.
- §4 Per-task complexity-score table (mandatory per
  `feedback_retro_task_complexity_score.md`):
  `| task | files-changed | commits | runtime-min | max-log-silence-min |`
  for each of Tasks 0..8.
- §5 Lessons promotion — any new `feedback_*.md` candidates
  (especially: e2e-fixture-mod naming convention if SL-c's
  `mod v1_sl_c_fixtures` shape diverged from SL-b's; staleness
  threshold formula edge cases; restoration-stub-and-future-wire
  pattern as a generalisable PMD entry).
- §6 Acceptance — confirm all checkboxes from §17.

**IMPLEMENT (file 2 of 2):** in
`.claude/rules/governance-log-entry-kind-registry.md`, locate the
"v1-SL-a entry kinds" section. Edit the "Emitting handler" column
for two rows:

- `ENTRY_KIND_SPONSOR_LIABILITY_FIRED` (row at line 172): change
  "v1-SL-c
  `crates/api/api/src/governance/sponsor_liability_grace.rs::run_grace_check_batch`
  fire branch (pending)" to "(active)" — flipped 2026-MM-DD post-SL-c.
- `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` (row at line 173): the SL-c
  portion of "v1-SL-b ... AND v1-SL-c
  `sponsor_liability_grace.rs::evaluate_escape_conditions` (pending)"
  → "(active)". (The SL-b portion was already flipped by SL-b retro.)

Total const count stays unchanged.

**Cross-cutting verification (Task 8 retro time — invariants):**

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
  4 lines.
- `rg -n 'SPONSOR_LIABILITY_GRACE_RUNNING'
  crates/routes/src/utils/scheduled_tasks.rs` returns at least 3
  lines (declaration + Drop + compare_exchange).
- `cargo build` workspace exit 0 via `cargo-validate-workspace.yml`
  on the phase-branch tip.
- `/brehon-verify` reports all three §16a stories `[done]`.
- `git diff governance-v0..phase-v1-SL-c -- migrations/` returns
  empty (zero migrations).

**MIRROR:** `.claude/PRPs/reports/v1-SL-b-retro.md` (predecessor;
expected to exist post-SL-b ship); `.claude/PRPs/reports/v1-SL-a-retro.md`;
`.claude/PRPs/reports/v1-JM-e-retro.md` for section structure.

**GOTCHA (retro before PR):** retro is written BEFORE `gh pr
create` per `feedback_retro_not_report.md`.

**GOTCHA:** §4 per-task complexity-score table is mandatory.

**GOTCHA (registry rule — both portions of `_ESCAPED`):** the
registry shows `_ESCAPED` with TWO emitting handlers (SL-b
revoke_endorsement AND SL-c sponsor_liability_grace). SL-c retro
flips ONLY the SL-c portion. SL-b retro should already have flipped
the SL-b portion. If SL-b retro missed it (e.g. ran before SL-b PR
#119 merged), the SL-c retro flips BOTH portions and a §5 retro
note flags the SL-b retro miss.

**Push and exit (Shape G — retro is meta-work; no `crates/**`
change → `cargo-validate-workspace` does not trigger).**

**COMMIT MESSAGE:** `docs(v1-SL-c): phase retrospective + flip ENTRY_KIND_* registry markers (task 8)`

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** `cargo check --workspace --features full`
  via `cargo-validate-workspace.yml:88`.
- **Lint:** `cargo clippy --workspace --features full --no-deps --
  -D warnings` via `cargo-validate-workspace.yml:92` (R6).
- **Test target compile (R7):** `cargo test --no-run -p lemmy_server
  --test e2e` via `cargo-validate-workspace.yml:95`. Triggers on
  Tasks 1, 2 (new module + scheduler block — affect cross-crate
  re-exports + use-block paths) and Tasks 3-7 (e2e file edits).
- **Migration round-trip:** N/A — SL-c ships zero migrations.
- **e2e execution (Phase 2):** the 5 new tests run via Phase 2 e2e
  user gate: (a) local on laptop or (b) GH dispatch via
  `cargo-test-e2e.yml`.

Pre-merge advisor-side verification: Story 1 checkpoint is
workspace-check workflow `conclusion: "success"` on Task 2's push
(module + scheduler wiring landed); Stories 2-3 checkpoints are
Phase 2 e2e exit 0 on the post-finalize-merge phase-branch tip;
`/brehon-verify` Brief-Scope outputs check.

### 14.1 Pre-existing tests preserved

- All SL-b-shipped tests (`mod v1_sl_b_fixtures::*`, 9 tests post-PR
  #119) — preserved verbatim
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

### 14.3 Edge cases NOT covered (out of v1-SL-c scope)

- Restoration-completed escape branch — stub-only per DQ #145; will
  be tested by restorative-mechanics-v1 PRD's plan when the producer
  endpoint ships.
- Multi-sponsor `all_revocation` / `majority_revocation` rules at
  scheduler-tick time — SL-b's `revoke_endorsement` handles the
  multi-sponsor escape at revocation time; the case wouldn't reach
  SL-c's scheduler unless still Pending (already-escaped cases skip
  in step 2 of `fire_or_escape_case_inner`).
- Concurrent SL-b + SL-c on same case (race) — defended by
  watchpoint #1 (FOR UPDATE) + watchpoint #2 (status re-check); no
  e2e test for the race itself (single-threaded test harness can't
  exercise true concurrency; structural correctness via the per-case
  tx body shape is the assertion).
- Scheduler tick interval respect (`every(5).run(...)`) — tested by
  the existing snapshot/appeal-window job tests' scheduler-tick
  pattern; SL-c's tests rely on `BREHON_DISABLE_GRACE_CHECK_JOB=1`
  + direct `run_grace_check_batch` invocation.
- Staleness threshold semantic — covered by no e2e test in SL-c
  (pure tracing emit; structural assertion via the function body's
  shape; integration with admin dashboard for surfacing is
  admin-dashboard-v1's concern).
- Federation outbound on `sponsor_liability_fired` /
  `sponsor_liability_escaped` — out per ADR-014 (v0 deferral).
- v0 mid-flight backfill behaviour — SL-a Task 1 backfill
  populates the cases SL-c picks up; SL-a's tests verify backfill;
  SL-c's tests pre-seed via `ModerationCaseInsertForm` (synthetic
  equivalent).

---

## 15. Validation commands (DoD)

> **Shape G plan — DoD is per-workflow, not inline cargo.** Per
> `.claude/PRPs/templates/plan.template.md` §15.6. SL-c ships zero
> migrations, so `cargo-validate-migration.yml` does not fire.

### 15.1 Per-task workspace check (Shape G)

For every §13 impl task that modifies `crates/**` (Tasks 1, 2, 3, 4,
5, 6, 7):

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

SL-c ships zero migrations; `cargo-validate-migration.yml` path
filter `migrations/**` excludes SL-c commits. No DoD entry.

### 15.3 Phase 2 e2e (post-finalize-merge of last impl task)

After Task 7's worker branch finalize-merges into `phase-v1-SL-c`,
the advisor surfaces the **Phase 2 e2e local-vs-dispatch user gate**
per `advisor-orchestrator.md`:

- **(a) local:** `cargo test -p lemmy_server --test e2e --features
  full -- --test-threads=1` on laptop in `run_in_background`; ~26 min
  wall-clock; zero billed.
- **(b) dispatch:** `gh workflow run cargo-test-e2e.yml --repo
  barrie-cork/lemmy --ref phase-v1-SL-c`; ci-watcher polls; ~26 min
  billed.

Plan-side DoD: e2e exit code 0 with all 5 SL-c tests passing;
failure path → §G4 classifier on log slice.

### 15.4 Cross-cutting verification (Task 8 retro time)

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
  crates/api/api/src/governance/mod.rs` returns 1 line.
- [ ] `rg -n 'SPONSOR_LIABILITY_GRACE_RUNNING'
  crates/routes/src/utils/scheduled_tasks.rs` returns at least 3
  lines (static decl + Drop impl + compare_exchange).
- [ ] `rg -n '"BREHON_DISABLE_GRACE_CHECK_JOB"'
  crates/routes/src/utils/scheduled_tasks.rs` returns 1 line.
- [ ] `rg -n 'pub async fn run_grace_check_batch|pub async fn evaluate_escape_conditions|pub async fn fire_or_escape_case|pub async fn check_grace_staleness'
  crates/api/api/src/governance/sponsor_liability_grace.rs` returns
  4 lines.
- [ ] `rg -n 'mod v1_sl_c_fixtures'
  crates/server/tests/e2e.rs` returns 1 line.
- [ ] `rg -c '#\[tokio::test\]\s*async fn grace_check_'
  crates/server/tests/e2e.rs` returns 5.
- [ ] R1: every `i32 ↔ i64` comparison in Tasks 1 + 2 uses
  `i64::from(...)` if cross-type comparison arises.
- [ ] R6: clippy invocations in `cargo-validate-workspace.yml` use
  `--no-deps -- -D warnings`.
- [ ] No edits to files outside §11 list.
- [ ] Every SL-c §16a story is `[done]`.
- [ ] No new ENTRY_KIND_*** consts under
  `crates/db_schema/src/source/governance/governance_log.rs`.
- [ ] No new migrations: `git diff governance-v0..phase-v1-SL-c --
  migrations/` returns empty.

### 15.5 ADR / OQ compliance verification

- [ ] **ADR-005** honoured — fire branch's
  `apply_sponsor_liability` writes per-dimension reputation_event
  rows.
- [ ] **ADR-008** honoured — every emit goes through
  `governance_log::append`; no direct INSERT to `governance_log`
  table.
- [ ] **ADR-010** honoured — won't-disadvantage rule preserved;
  SL-c reads SL-a's mid-flight backfill cases without retroactive
  invalidation.
- [ ] **ADR-013** honoured — exhaustive `EscapeStatus` match in
  `fire_or_escape_case_inner` enumerates `Escape{...}` AND `Fire`;
  no `_ =>` arm. Existing exhaustive `CaseStatus` matches across
  ADR-013 sweep preserved.
- [ ] **ADR-014** honoured — no federation outbound on
  `sponsor_liability_fired` or `sponsor_liability_escaped`.
- [ ] **ADR-015** honoured —
  `liability_escape_reason` JSONB carries `actor_pseudonym` (string),
  NOT raw `caller_id`. Both governance_log payloads carry
  `actor_pseudonym` / `target_pseudonym`.
- [ ] **OQ-V1-SL-05** honoured — `liability_escape_reason` JSON
  carries `version: 1` from day one.
- [ ] **PRD §6** honoured — scheduler tick + per-case isolation +
  staleness check all shipped per spec.
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

**Phase 1b (migration round-trip):** N/A — no migrations in SL-c.

**Phase 2 (e2e):**

- Workflow: `.github/workflows/cargo-test-e2e.yml` (only on user
  dispatch per PR #105)
- OR local: `cargo test -p lemmy_server --test e2e --features full
  -- --test-threads=1` on laptop
- Branch: `phase-v1-SL-c`
- Expected: all tests pass

### 15.7 Manual validation snippets (advisor-side smoke only — non-binding)

> Per `feedback_features_full_p_crate_incompatible.md`: never `-p
> <crate>` + `--features full`; use `--workspace --features full`.

> Per `feedback_e2e_filter_assumes_naming.md`: SL-c tests all start
> with `grace_check_` — confirm via grep before filter.

```bash
ls .github/workflows/cargo-validate-workspace.yml
ls .github/workflows/cargo-test-e2e.yml

gh run list --repo barrie-cork/lemmy \
  --branch phase-v1-SL-c \
  --workflow cargo-validate-workspace \
  --limit 1 --json conclusion,databaseId

cargo check --workspace --features full > /tmp/sl-c-check.log 2>&1
status=$?
tail -20 /tmp/sl-c-check.log
echo "exit: $status"

rg '#\[tokio::test\]\s*async fn grace_check_' crates/server/tests/e2e.rs | wc -l
# EXPECT: 5

rg -n 'pub async fn run_grace_check_batch' crates/api/api/src/governance/sponsor_liability_grace.rs
# EXPECT: 1 match
```

These are advisor-side only; not §16 acceptance criteria. Per DQ
#67 resolution.

---

## 16. Acceptance criteria

- [ ] All 9 tasks (Task 0..7 + Task 8 retro) committed.
- [ ] §15.1 (`cargo-validate-workspace.yml`) `conclusion: "success"`
  after every impl task push (Tasks 1-7).
- [ ] §15.2 (migration round-trip) — N/A (zero migrations).
- [ ] §15.3 (Phase 2 e2e — local or dispatch) all tests pass; the 5
  new `v1_sl_c_fixtures::grace_check_*` tests green.
- [ ] §15.4 (cross-cutting verification — 13 boxes) all ticked.
- [ ] §15.5 (ADR / OQ compliance — 10 boxes) all ticked.
- [ ] §16a stories — all three `[done]`.
- [ ] No edits to files outside §11 list.
- [ ] Retro committed per Task 8.
- [ ] PR opens against `governance-v0` (NOT `main`) with
  `--repo barrie-cork/lemmy`.
- [ ] `/brehon-verify` report at
  `.claude/PRPs/reports/v1-SL-c-verify.md` shows all stories ✓.
- [ ] DQ #148 (split-or-proceed) resolved by planner with
  proceed-as-one rationale at plan-write time (Recipe 2 self-resolved
  per `decision-queue.md`).
- [ ] Registry markers flipped (`_SPONSOR_LIABILITY_FIRED` row
  entirely; `_SPONSOR_LIABILITY_ESCAPED` row's SL-c portion).

---

## 16a. Stories (independently-testable behaviour units)

Per `.claude/PRPs/templates/plan.template.md` §16a +
`feedback_story_grain_checkpoint.md` +
`feedback_brehon_verify_pre_merge.md`. Three stories — one per
behaviourally distinct unit.

### Story 1: Module + scheduler wiring + atomic guard pair compile clean and the scheduler block fires on boot

- **Composing tasks:** Tasks 1, 2
- **Checkpoint workflow (Phase 1):** `cargo-validate-workspace.yml`
  on Task 2's worker branch SHA → `conclusion: "success"`
- **Expected output (workspace-check):** all three jobs pass (check,
  clippy --no-deps -- -D warnings, test --no-run)
- **Brief-Scope outputs to verify (used by `/brehon-verify`):**
  - `crates/api/api/src/governance/sponsor_liability_grace.rs`
    exists; contains `pub async fn run_grace_check_batch(`
    declaration; contains `pub async fn evaluate_escape_conditions(`;
    contains `pub async fn fire_or_escape_case(`; contains
    `pub async fn check_grace_staleness(`.
  - `crates/api/api/src/governance/sponsor_liability_grace.rs`
    contains `pub enum EscapeStatus` declaration with `Escape {`
    and `Fire` arms.
  - `crates/api/api/src/governance/sponsor_liability_grace.rs`
    contains `pub struct GraceCheckBatchOutcome` with `cases_processed`,
    `fired`, `escaped`, `skipped` fields.
  - `crates/api/api/src/governance/sponsor_liability_grace.rs`
    references `ENTRY_KIND_SPONSOR_LIABILITY_FIRED` AND
    `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` AND
    `actor_pseudonym_helper::get_or_create` AND
    `sponsor_liability::apply_sponsor_liability` AND
    `governance_log::append`.
  - `crates/api/api/src/governance/sponsor_liability_grace.rs`
    contains `for_update()` (per watchpoint #1).
  - `crates/api/api/src/governance/mod.rs` contains
    `pub mod sponsor_liability_grace;` line.
  - `crates/routes/src/utils/scheduled_tasks.rs` contains
    `static SPONSOR_LIABILITY_GRACE_RUNNING: AtomicBool` declaration.
  - `crates/routes/src/utils/scheduled_tasks.rs` contains
    `struct GraceCheckRunningGuard;` declaration.
  - `crates/routes/src/utils/scheduled_tasks.rs` contains
    `impl Drop for GraceCheckRunningGuard` block.
  - `crates/routes/src/utils/scheduled_tasks.rs` contains
    `"BREHON_DISABLE_GRACE_CHECK_JOB"` literal.
  - `crates/routes/src/utils/scheduled_tasks.rs` contains
    `lemmy_api::governance::sponsor_liability_grace::run_grace_check_batch`
    call.
  - `crates/routes/src/utils/scheduled_tasks.rs` contains
    `lemmy_api::governance::sponsor_liability_grace::check_grace_staleness`
    call.

### Story 2: Fire branch transitions case to `SponsorLiabilityFired` with both per-sponsor and summary log entries + writes `reputation_event` rows

- **Composing tasks:** Task 3
- **Checkpoint workflow (Phase 1):** `cargo-validate-workspace.yml`
  on Task 3's worker branch SHA → `conclusion: "success"`.
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

- **Composing tasks:** Tasks 4, 5, 6, 7
- **Checkpoint workflow (Phase 1):** `cargo-validate-workspace.yml`
  on Task 7's worker branch SHA → `conclusion: "success"`.
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
  - Task 4 test body asserts `case.status ==
    CaseStatus::SponsorLiabilityEscaped` AND
    `case.liability_escape_reason["version"] == 1` AND
    `case.liability_escape_reason["reason"] == "sponsor_revoked"`
    AND `case.liability_escape_reason["actor_pseudonym"].is_string()`
    AND defensive ADR-015 assertion (actor_pseudonym does NOT equal
    `format!("{}", sponsor_id.0)`).
  - Task 5 test body asserts `outcome.cases_processed == 0` AND
    case status STILL Pending.
  - Task 6 test body asserts `outcome.fired == 1 && outcome.skipped
    == 1` AND case_a fired AND case_b still Pending (per-case
    isolation).
  - Task 7 test body asserts three iterations process 2 + 2 + 1
    cases respectively (batch_size cap).

> **Verification mapping:** `/brehon-verify` iterates this section,
> runs each Story's checkpoint workflow on the worktree branch, and
> confirms each Brief-Scope output exists + matches its structural
> pattern.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0..15 confirmed).
- [ ] Tasks 1..7 committed.
- [ ] Task 8 retro committed.
- [ ] §15 validation green at every gate (Phase 1 workspace check
  ×7, Phase 2 e2e ×1).
- [ ] §16a stories all `[done]`.
- [ ] PR opened by BM session against `governance-v0`.
- [ ] CodeRabbit review complete with findings triaged per
  `feedback_pr_review_triage_pattern.md`.
- [ ] `/brehon-verify` report at
  `.claude/PRPs/reports/v1-SL-c-verify.md` shows all stories ✓.
- [ ] Post-merge phase branch retained for retro reads.
- [ ] DQ #148 (split-or-proceed) self-resolved by planner with
  proceed-as-one rationale at plan-write time.
- [ ] Registry markers flipped (`_SPONSOR_LIABILITY_FIRED` row
  entirely; `_SPONSOR_LIABILITY_ESCAPED` row's SL-c portion).

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Complexity score 21 prompts advisor to mandate split | LOW | LOW | §5.2 — planner self-resolved DQ #148 with proceed-as-one rationale; SL-b shipped at score 38 (heavier) and JM-e at 15 with no operational regret. Advisor may overturn at plan-approval if user prefers split |
| Junior worker-hang on e2e.rs Edits (file is 10976+ lines, growing post-SL-b) | MED | HIGH | Per `feedback_junior_worker_e2e_edit_hang.md`: each test is its own §13 task; per-task anchor-Edit appends inside `mod v1_sl_c_fixtures`; no bulk Edit. SL-b shipped 9 e2e tests as 9 tasks cleanly at 10976-line file size |
| Task 2 (scheduler block) dispatches before Task 1 (module) finalize-merges; compile fails on `unresolved import sponsor_liability_grace::*` | LOW | MED | Tasks 1-7 dispatch serially (no `[P]` cohort); per `advisor-orchestrator.md` cohort-dispatch rule, advisor waits for prior task finalize-merge before next dispatch |
| SL-b not yet merged at SL-c plan-implement time | LOW | LOW | Task 0 Probe 7 detects (`test -f revoke_endorsement.rs`); Task 3's anchor falls back to `mod v1_jm_e_fixtures` end if SL-b unmerged. Plan ships under "SL-b merge before SL-c cut" assumption per brief observation 6 |
| Sanction multiplicity drift (1:1 → many-to-one in future SL-d/SL-e refactor) | LOW | LOW | Watchpoint #6 — `ORDER BY id ASC LIMIT 1` is defensive; future refactor would need to update SL-c's lookup. SL-d retro should flag if multiplicity changes |
| `liability_escape_reason` JSONB schema regression (raw `caller_id` instead of `actor_pseudonym`) | LOW | HIGH | §4 watchpoint #4 + Test #2 (Task 4) defensive ADR-015 assertion catches |
| TOCTOU on case status (concurrent SL-b revocation between batch query and per-case tx) | LOW | HIGH | §4 watchpoint #1 — FOR UPDATE lock + status re-check inside per-case tx; structural correctness via the per-case tx body shape (no e2e test for the race itself) |
| Per-case error propagation breaks isolation (one bad case blocks batch) | LOW | HIGH | §4 watchpoint #8 + Test #4 (Task 6) is the strong assertion catching this regression |
| `BREHON_DISABLE_GRACE_CHECK_JOB` env var leaks atomic-bool slot if order reversed | LOW | MED | §4 watchpoint #9 + Task 2 GOTCHA — env-var check FIRST in closure body, BEFORE compare_exchange |
| Restoration-stub branch accidentally fires (despite DQ #145 advisor lock) | LOW | HIGH | §4 watchpoint #14 — `evaluate_escape_conditions` literally returns `Fire` if no surety revocation matches; restoration-completed read is a comment block + `let _ = case_id;` placeholder. Function body asserted by Story 1 Brief-Scope structural pattern |
| Staleness threshold formula error (h vs s confusion) | LOW | LOW | DQ #146 + §10.4 — formula `threshold_hours = max_grace_hours × multiplier`; `chrono::Duration::hours(threshold_hours_i)`. Defaults give 1440h ≈ 60d matching PRD §6.3 |
| ts-rs derive regression (export breaks downstream client codegen) | LOW | LOW | SL-c adds zero ts-rs derives (no new DTO; new `EscapeStatus` enum + `GraceCheckBatchOutcome` struct are server-internal — no `#[cfg_attr(feature = "ts-rs", ...)]` annotations). Workspace-check workflow `cargo test --no-run` step catches any cross-crate regression |
| GH-Actions minutes budget exceeded by 7 Phase-1 + 1 Phase-2 e2e | LOW | LOW | Phase 2 e2e local-default per JM-d retro §5; user picks dispatch only on audit-trail need. Total ~50 min monthly burn well within Free 3000 min |
| PMD #126 — DQ ID collision from concurrent SL-c / rep-tuning-r3 work | LOW | LOW | next-id calc spans archives + live; advisor monitors at finalize-merge. SL-c plan-write computed next_id = 148 against live `decision-queue.json` at plan-write time |
| Junior worker pre-pushes break finalize-merge | LOW | LOW | Junior daemon's finalize step is post-Shape-G correct (per JM-d retro §1.1) |
| Test pre-seed `SponsorLiabilityPending` cases via direct DB-write inadvertently break SL-b's tests | LOW | LOW | Tests insert NEW rows via `ModerationCaseInsertForm`; no UPDATE on existing rows. Each test bootstraps a fresh Postgres container per testcontainers-rs |
| ADR-013 enum-exhaustiveness lint trips on a new match site SL-c accidentally introduces | LOW | LOW | §4 watchpoint #7 — SL-c uses Diesel `.filter(status.eq(SponsorLiabilityPending))`, not `match case.status`. New `match escape: EscapeStatus` is enumerated exhaustively (Fire + Escape) — no `_ =>` arm |
| `apply_sponsor_liability` v0 helper signature drift between brief-write and impl-time | LOW | LOW | Probe 9 verifies signature at task 0; if drift, Task 0 STOP and impl-task files DQ before Task 1. Verified at plan-write: `pub(crate) async fn apply_sponsor_liability(conn, target_person_id, case_id, community_id, action, &mut ConfigCache) -> LemmyResult<usize>` |
| `governance_config` UPSERT in Test #5 (Task 7) interacts with config cache staleness across batch invocations | LOW | LOW | The outer ConfigCache in `run_grace_check_batch` is fresh per call (`ConfigCache::new()` at function entry). Second/third invocations see updated config |
| clokwerk schedule-pinning means batch_size config flip mid-run doesn't take effect until next tick | LOW | LOW | The batch_size is read PER-TICK (inside the closure body via `config::get_int`), not at scheduler setup. Only the interval is restart-pinned. Tested via Test #5 (config flipped before invocation) |

---

## 19. Notes

### 19.1 Planner DQs filed

- **DQ #148** (`from: "planner"`, `kind: "blocker"`, `answered_by:
  "planner"` self-resolved per Recipe 2) — complexity-score split
  decision per §5.2. Question: "Complexity score 21 exceeds 8 — split
  `v1-sponsor-liability-c` into `v1-SL-c-1` (Tasks 1-2: module +
  scheduler wiring, score ~7) + `v1-SL-c-2` (Tasks 3-7 + retro: 5
  e2e tests, score ~16), or proceed as one plan?". Options: split /
  proceed. Planner self-resolved with **proceed-as-one** rationale:
  dominant factor is e2e edits (+15 of +21); split's c2 still scores
  16 (above 8); split fragments a logically atomic deliverable (PRD
  §15 row 3); SL-b at 38 + JM-e at 15 + SL-a at 13 all shipped
  proceed-as-one without operational regret. Advisor may overturn
  at plan-approval time if user prefers split.

### 19.2 Self-resolved planner findings (LESSON candidates)

- **The brief's pre-estimate (5-7) significantly underestimated the
  e2e factor.** The actual mechanical score per
  `feedback_complexity_score_pre_split.md` factor table is 21 — the
  +3-per-e2e-edit weight applied to 5 tests dominates the +15
  contribution. The brief's "pre-estimate range 5-7" appears to have
  underweighted the e2e factor. Aligns with SL-b §19.2 self-finding
  ("brief pre-estimate optimistic"). Recommendation: brief-template
  pre-estimate convention should be dropped in favour of "planner
  computes from §5.1 breakdown table" per DQ #138 resolution. SL-c
  is the second consecutive plan to file this finding; promote to
  PMD lesson candidate.
- **The 5-test discipline (per `feedback_junior_worker_e2e_edit_hang.md`)
  inflates §5 e2e factor count proportionally to test count.**
  Future SL-d/SL-e plans with similar test-heavy lanes will
  routinely score in the 20-50 range. The complexity-score gate
  becomes a near-certain "trip" for any test-heavy phase.
  Recommendation: per `feedback_principles_not_rules.md`, accept
  that the gate's purpose is to surface "split-or-proceed" as an
  explicit decision — test-heavy phases routinely answer proceed.
  SL-b §19.2 raised the same observation; SL-c confirms.
- **No new ENTRY_KIND consts or schema = clean SL-c shape.** Like
  SL-b, SL-c is purely module + wiring + tests. Score composition
  is dominated by impl-task multiplicity (7 tasks + e2e factor)
  rather than cross-cutting structural changes.

### 19.3 Restoration-escape stub-and-future-wire breadcrumb

Per DQ #145 advisor lean + `feedback_build_what_tests_exercise.md`
(PMD #14) — never pre-implement detection for a producer that
doesn't yet emit. SL-c stubs the
`EscapeStatus::Escape{reason: "restoration_completed", ...}`
branch in two places:

1. The `EscapeStatus` enum at the top of
   `sponsor_liability_grace.rs` defines the `Escape{reason:
   String, actor_pseudonym: String, ref_id: i64}` variant generally
   — the `reason` field's type is `String`, not an enum, so any
   future reason string fits. The doc-comment on the enum
   enumerates `"sponsor_revoked"` (active in SL-c) and
   `"restoration_completed"` (stub).
2. The `evaluate_escape_conditions` body has a comment block
   describing the future restoration-completed branch, with a
   `let _ = case_id;` line to suppress unused-var lints. The
   function returns `Fire` if no surety revocation matches.

When **restorative-mechanics-v1 PRD** is drafted and ships:

- That PRD's plan adds a query to `evaluate_escape_conditions`
  immediately after the surety-revocation check — query
  `governance_log` for any `restoration_completed` entry for
  `target_person_id` between `decided_at` and `now`. If found,
  return
  `EscapeStatus::Escape{reason: "restoration_completed", actor_pseudonym:
  <admin or defendant pseudonym>, ref_id: <restoration.id.0 (i64)>}`.
- That PRD's plan adds a corresponding e2e test
  (`grace_check_escapes_case_when_restoration_completed_during_window`)
  inside its own fixture mod (e.g. `mod v1_restoration_v1_fixtures`)
  in `crates/server/tests/e2e.rs`.
- That PRD's plan flips the `(pending)` marker on
  `ENTRY_KIND_RESTORATION_COMPLETED` registry row.

The SL-c stub keeps the enum extensible (no breaking change when
restoration ships) while preventing dead-code drift.

### 19.4 Pre-existing pending DQ entries

- **None at planning time.** `decision-queue.json` shows 0 pending
  entries on governance-v0 @ `cb0a9a4ea`. Recently resolved: DQ
  #144-#147 (SL-c clarify pass).

### 19.5 Out-of-scope follow-ups (Task 8 retro candidates)

- **`apply_sponsor_liability` compute/fire split** — SL-d.
- **`submit_jury_vote` mutation: `Decided →
  SponsorLiabilityPending` transition** — SL-d.
- **`restoration_complete` endpoint + restoration-escape branch
  detection** — restorative-mechanics-v1 PRD.
- **Lane-wide e2e suite** (full Decided → Pending → Fired/Escaped
  through SL-d transition + SL-c scheduler) — SL-e.
- **Sponsor notification UX** — PRD §13 OQ-V1-SL-03; gated on
  notification surface generalised by jury-mechanics-v1 OQ-005.
- **Step-up auth for admin-driven scheduler runs** — v2.
- **Cross-instance federation of grace-window events** — v2 per
  ADR-014.
- **Admin dashboard surface for staleness alerts** — gated on
  admin-dashboard-v1 PRD's read-side surface.

### 19.6 Confidence bands

- **High (9/10):** module structure — exact paired mirror of
  `reputation_snapshot.rs::run_snapshot_batch` (per-chunk → adapted
  per-case) and `appeal_window_expiry.rs::run_appeal_window_expiry_batch`
  (per-row pattern).
- **High (9/10):** scheduler tick wiring — third sibling of two
  shipped patterns; structural template proven across two
  production schedulers since Phase 5a.
- **High (9/10):** ConfigCache two-tier pattern per DQ #144 — exact
  mirror of `reputation_snapshot.rs:363/389`.
- **High (8/10):** 5 e2e tests — each is anchor-Edit-friendly; sanction
  insert + surety insert + ModerationCaseInsertForm pre-seed is
  mechanical.
- **Moderate (7/10):** §5 complexity score 21 will trip
  split-or-proceed DQ #148; planner self-resolved with proceed-as-one
  rationale citing SL-b/SL-a/JM-e precedents.
- **Moderate (6/10):** restoration-stub-and-future-wire breadcrumb —
  novel pattern; planner judgment on stub shape (enum variant
  defined, function body never constructs) follows
  `feedback_build_what_tests_exercise.md`.
- **High (8/10):** ADR-015 pseudonym discipline — defensive Test #2
  assertion catches regression; canonical
  `actor_pseudonym_helper::get_or_create` source.
- **Moderate (7/10):** sanction multiplicity assumption (1:1)
  verified at plan-write time but defensive `ORDER BY id ASC LIMIT 1`
  guards future drift.

### 19.7 Why no clarify DQ at impl time

Brief §4.2 enumerates the boundary-of-judgment cases that should
trigger an impl-side DQ. None apply to this plan as written:

- v0 `apply_sponsor_liability` signature at
  `sponsor_liability.rs:142` is intact (verified plan-write time:
  `pub(crate) async fn apply_sponsor_liability(conn, target_person_id,
  case_id, community_id, action: SanctionAction, cache: &mut
  ConfigCache) -> LemmyResult<usize>`).
- Sanction multiplicity is 1:1 (verified at
  `submit_jury_vote.rs:435` — single `insert_into(sanction::table)`
  per case in v0/JM-c/JM-d code). No planner-DQ on multiplicity.
- `_SPONSOR_LIABILITY_FIRED` (line 198) and
  `_SPONSOR_LIABILITY_ESCAPED` (line 197) and
  `_RESTORATION_COMPLETED` (line 196) ENTRY_KIND consts declared in
  SL-a.
- Shim re-exports at `crates/api/api/src/governance/governance_log.rs`
  lines 68 + 74 + 75 are present.
- Canonical mirrors `reputation_snapshot::run_snapshot_batch`,
  `appeal_window_expiry::run_appeal_window_expiry_batch`, and the
  `scheduled_tasks.rs:setup` shape are intact.
- DQ #144-#147 resolved at clarify-time; embedded in brief as
  authoritative.

If any of these baseline assumptions changes between plan-write and
impl-time, the impl-task subagent files a DQ pending entry.

### 19.8 Forward-only retrofit scope

Per `feedback_schema_changing_spec_retrofit_question.md` — SL-c does
NOT change the shape of any existing artifact class (no new template
section, no new schema marker, no new YAML field). All §13 task
contracts are routine impl-task contracts; no advisor-side schema
addition. No retrofit question applies.

---

## 20. Confidence score

- **Plan correctness:** 9/10 — patterns mirror
  `reputation_snapshot.rs::run_snapshot_batch` +
  `appeal_window_expiry.rs::run_appeal_window_expiry_batch` directly;
  §10 + §13 + §16a all match the SL-b/JM-e Shape-G shape; the
  module + scheduler block + 5 tests are mechanical with strong
  precedents.
- **Cargo budget:** 10/10 — Shape G (cargo runs off-box; budget
  non-binding; forbidden-window non-binding).
- **Test coverage:** 8/10 — 5 behaviourally distinct tests covering
  fire path + escape path + no-op + per-case isolation + batch_size
  config. Multi-sponsor escape rules at scheduler-tick time are not
  exercised (out per §12 — SL-b's revocation handler covers them at
  revocation time; SL-c's scheduler picks up cases that SL-b didn't
  already escape). Restoration-escape branch is stub-only.
- **Story-grain decomposition:** 9/10 — every task maps to exactly
  one story; all three stories have concrete checkpoint workflows
  + grep-verifiable Brief-Scope outputs. Story 2 is single-task
  (Task 3 is the fire-path test in isolation); Stories 1 + 3 are
  multi-task.
- **Plan-shape conformance:** 9/10 — 20-section schema followed
  literally; §15.6 Shape G shape adopted; §16a mandatory present;
  §13 per-task FILES YAML block on every task; §5.1 complexity
  breakdown table mechanical.

---

_Plan author: planning subagent (laptop fallback CWD
`/Users/barrie/Developer/lemmy-advisor-sl-c`, sibling worktree on
`governance-v0` @ `cb0a9a4ea`, 2026-05-07 — Junior daemon broken on
EliteDesk; Junior task cancelled; falling back to local Opus 4.7 (1M)
authoring per advisor instruction). Plan committed locally on
`governance-v0`; advisor publishes via standard sub-phase flow into
`governance-v0` after plan-approval gate (DoD smoke + watchpoint
specificity + complexity proceed-as-one confirmation) where SL-c's
BM-task then cuts `phase-v1-SL-c` (only after SL-b PR #119 merges).
One planner DQ self-resolved at commit time: DQ #148
(split-or-proceed; planner-resolved with proceed-as-one rationale).
Confidence 8.5/10. Plan ships under proceed-as-one assumption._

LESSON: a third sibling scheduler module (after `reputation_snapshot.rs` +
`appeal_window_expiry.rs`) reinforces the value of paired canonical
mirrors per DQ #147 — citing BOTH a per-chunk pattern AND a per-row
pattern lets the planner pick the closer-fit shape per case
(SL-c is per-case, closer to per-row). Future scheduler additions
(restoration-completed v1 if ever, etc) should continue this
pair-citation discipline.

LESSON: the brief-template pre-estimate convention (e.g. SL-c
brief's "5-7" range) repeatedly underweights the e2e-edit factor +3
multiplier. SL-b §19.2 raised this; SL-c §19.2 confirms. Two
consecutive datapoints — promote to PMD lesson candidate per
`feedback_junior_pmd_write_convention.md`. Recommendation: brief
template should drop the pre-estimate field entirely (or relabel as
"planner computes from §5.1 breakdown — pre-estimate is not
authoritative") per DQ #138 resolution, since the formula gives a
predictable mechanical answer the planner is required to compute.

