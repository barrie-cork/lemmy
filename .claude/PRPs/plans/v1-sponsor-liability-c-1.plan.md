# Plan: v1-sponsor-liability-c-1 — `sponsor_liability_grace` scheduler module + clokwerk wiring (no e2e)

> **Shape G plan** — SL-c-1 ships under Shape G (Layer G2 push-and-exit). §15
> references workflow YAMLs by path + expected `conclusion`, not inline cargo.
> Cargo runs on GitHub-hosted runners (workspace-check on `junior/*`). **Phase 2
> e2e is NOT triggered for c-1** because c-1 ships zero new e2e tests; the e2e
> behavioural validation lives in c-2.

> **Split context (c-1 only).** This plan is the first of two — `v1-SL-c-1`
> (this plan) and `v1-SL-c-2` (sibling). The split was directed by user
> (DQ #150) overriding the trunk plan's planner proceed-as-one lean (DQ #148).
> The trunk plan at `.claude/PRPs/plans/v1-sponsor-liability-c.plan.md`
> (committed `7af7dfa93`) stays in place as the historical record. c-1 and
> c-2 inherit §1 (Goal) / §2 (Anti-goals — minus the proceed-as-one rationale,
> which is superseded) / §3 (Problem statement) / §4 (Solution statement —
> watchpoints carried verbatim, with #12 annotated "applies to c-2 only") /
> §6 (Relationship table — c-2 adds c-1 as upstream MERGED dep) / §7-§12 /
> §17-§20 from the trunk, with sub-phase-specific slices in §5/§11/§13/§14/
> §15.4/§16/§16a.

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
| 12 | NOT building in v1-SL-c-1 |
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

v1-SL-c-1 ships the v0-deferred `sponsor_liability_grace` scheduler module
per PRD §6 + §9.4 + §9.5 + §15 row 3 — **the module + wiring half of the
SL-c deliverable**. It creates a new server-internal module at
`crates/api/api/src/governance/sponsor_liability_grace.rs` exporting four
public async functions (`run_grace_check_batch`, `evaluate_escape_conditions`,
`fire_or_escape_case`, `check_grace_staleness`); wires a clokwerk tick block
in `crates/routes/src/utils/scheduled_tasks.rs::setup` (sibling of the
15-minute `reputation_snapshot` block at lines 189-245 + the hourly
`appeal_window_expiry` block at lines 254-274); and adds a third atomic
concurrency guard pair (`SPONSOR_LIABILITY_GRACE_RUNNING` +
`GraceCheckRunningGuard`, sibling of `REPUTATION_SNAPSHOT_RUNNING` and
`APPEAL_WINDOW_EXPIRY_RUNNING`).

**c-1 ships ZERO new e2e tests.** The 5 grace_check_* e2e tests
ship in v1-SL-c-2 against the c-1-merged module via direct
`run_grace_check_batch` invocation. Behavioural validation of the fire +
escape branches lives in c-2. c-1's behavioural correctness is asserted
by Story 1's structural Brief-Scope outputs (function signatures, atomic
guard pair, scheduler block + env-var override) and by the workspace-check
workflow's compile + clippy gates.

The scheduler reads `SponsorLiabilityPending` cases past their
`grace_expires_at`, opens one `run_transaction` per case (per-case
isolation), evaluates escape conditions (sponsor revoked since
`decided_at`?), and either fires (calls **unsplit v0**
`apply_sponsor_liability` from `sponsor_liability.rs:142`, then UPDATEs
`status = SponsorLiabilityFired`, emits `sponsor_liability_fired` summary
log) or escapes (UPDATEs `status = SponsorLiabilityEscaped`, sets
`liability_escape_reason` JSONB, emits `sponsor_liability_escaped` log).
SL-c-1 does NOT add new schema, migrations, ENTRY_KIND consts, CaseStatus
variants, or `governance_config` seeds — all required substrate shipped in
SL-a; the v0 fire helper and the `_FIRED`/`_ESCAPED` consts are intact.

**Why now (c-1 only).** SL-a shipped 2026-05-04 (PR #111, `governance-v0` @
`790f6101d`) laying schema + 13 config seeds + 5 ENTRY_KIND consts + 3
CaseStatus variants. SL-b ships the revocation-branch escape fire-site
(in flight; PR #119). c-1 ships the scheduler module + wiring; c-2
behaviourally validates via 5 e2e tests. Splitting c-1 from c-2 keeps c-1's
PR scope tight (2 impl tasks + retro) and lets the e2e-heavy c-2 work
against a merged c-1 module rather than against an unmerged module on a
worktree branch.

**Compute/fire posture (advisor-locked, masthead of `sl-c-planning-1.md`).**
SL-c calls **unsplit v0** `apply_sponsor_liability(conn, target_person_id,
case_id, community_id, action, &mut cache)` from the scheduler's fire
branch. The PRD §9.1 compute/fire split is **NOT** SL-c's deliverable —
it remains SL-d's. SL-c's call site is forward-compatible: when SL-d
ships and `apply_sponsor_liability` becomes a thin wrapper (or is
replaced by `compute_sponsor_liability` + `fire_sponsor_liability`),
SL-c-1's call site updates as a no-op-behaviour change. (Carried verbatim
from trunk plan §1.)

**Headline acceptance condition (c-1 only).** Story 1 is `[done]` — the
new module + scheduler block + atomic guard pair compile clean (workspace
check workflow `conclusion: success`) and the structural Brief-Scope
outputs (function signatures, enum variants, struct fields, scheduler-
block elements) are present per `/brehon-verify`. Stories 2 + 3
(behavioural fire/escape/no-op/isolation/batch-size) ship in c-2.

This sub-phase touches NO schema, NO migration, NO ENTRY_KIND const, NO
CaseStatus variant, NO `governance_config` seed, NO HTTP endpoint, NO
DTO, NO route, NO e2e test. It is **server-internal scheduler module +
clokwerk block + retro**.

---

## 2. Source

- `.claude/PRPs/briefs/sl-c-planning-1.md` @ `governance-v0` `cb0a9a4ea`
  — the original advisor brief (post-`/brehon-clarify`; DQ #144-#147
  resolved 2026-05-07).
- `.claude/PRPs/briefs/sl-c-split-planning-1.md` @ `governance-v0`
  (this plan's split brief; DQ #150 records the user override authorising
  the split into c-1 + c-2).
- `.claude/PRPs/plans/v1-sponsor-liability-c.plan.md` @ `7af7dfa93`
  — **trunk plan; this c-1 plan is the Tasks-0+1+2+retro slice**.
  §1, §3, §4, §6, §7-§12, §17-§20 inherited verbatim from trunk with
  c-1 / c-2 annotations.
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
- `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` — most recent
  shipped SL plan; §6 table shape (canonical mirror for c-1 §6); §13
  task ordering; §15 Shape G DoD shape.
- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — predecessor
  ships the schema + seeds + consts + variants c-1 reads. c-1 §11
  file inventory does not duplicate SL-a entries.
- `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` — most recent
  post-Shape-G plan; §15 per-workflow DoD shape; §13 FILES YAML blocks.
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
  (PR #105). For c-1: Phase 2 e2e is **not triggered** (no new e2e
  tests).
- `.claude/rules/governance-log-entry-kind-registry.md` — confirms
  `_SPONSOR_LIABILITY_FIRED` (line 172, "v1-SL-a const; v1-SL-c call
  site (pending)") and `_SPONSOR_LIABILITY_ESCAPED` (line 173, "v1-SL-a
  const; v1-SL-b + v1-SL-c call sites (pending)") shipped in SL-a.
  c-1 is the file-author for both fire-sites — code lands at c-1
  merge time, but the registry markers stay `(pending)` until c-2
  retro flips them after the e2e tests behaviourally validate the
  fire-sites work end-to-end. Per `feedback_build_what_tests_exercise.md`
  (PMD #14): the `(active)` marker requires test exercising, not just
  code presence.
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  — **ADR-005** (multi-dimensional reputation; SL-c-1's fire branch
  writes per-dimension events via v0 `apply_sponsor_liability`),
  **ADR-008** (append-only signed log; every SL-c-1 emit goes through
  `governance_log::append`), **ADR-010** (won't-disadvantage rule;
  v1 mid-flight backfill SL-c first sees), **ADR-013** (CaseStatus
  enum-exhaustiveness — no `_ =>` arms in c-1 matches), **ADR-014**
  (federation deferral — SL-c emits log events on local instance only;
  no outbox), **ADR-015 (pseudonymisation — load-bearing for SL-c-1;
  every payload field naming a person uses pseudonym, not raw id)**,
  OQ-V1-SL-05 (`liability_escape_reason` schema versioning — `version:
  1` from day one).

### Lessons that bind §13 decisions (c-1)

- `feedback_multi_write_handlers_need_transactions.md` —
  **load-bearing for c-1**. Each per-case body wraps re-load + status
  re-check + sanction lookup + escape-condition evaluation + UPDATE +
  log entry inside one `run_transaction`. The outer
  `run_grace_check_batch` does NOT open a transaction — it iterates
  cases, each opening its own per-case tx. Per-case errors are caught
  + logged via `warn!`; iteration continues.
- `feedback_advisor_watchpoint_specificity.md` (cited by trunk) —
  every §4 watchpoint cites a concrete file/handler/`schema.rs`/
  `enums.rs` line. Binds §4 entries.
- `feedback_complexity_score_pre_split.md` — c-1 score 3 (below
  threshold; no split-or-proceed DQ).
- `feedback_explicit_file_arrays_on_tasks.md` — every §13 task carries
  a FILES YAML block; cohort dispatch reads `union(creates, modifies)`.
- `feedback_parallel_cohort_dispatch.md` — Tasks 1 + 2 are NOT
  cohort-compatible because Task 2 imports symbols Task 1 creates → serial.
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
- `feedback_clippy_test_style.md` — R1 every `i32 ↔ i64` comparison
  uses `i64::from(...)`. Bound in §4 watchpoint #13.
- **`feedback_junior_worker_e2e_edit_hang.md`** — citation-only for
  c-1 (c-1 has zero e2e edits; the lesson binds c-2 instead).
- `feedback_brehon_verify_pre_merge.md` — §16a Story 1 grain enables
  `/brehon-verify` phantom check before `bm-merge`.
- `feedback_principles_not_rules.md` — c-1 score 3 is well below
  threshold; no proceed-as-one rationale needed.
- `feedback_read_canonical_before_writing_spec.md` — c-1 cites
  trunk plan + sibling shipped plans; canonical-schema gate satisfied.
- `feedback_pr_per_phase.md` — one commit per task; §13 ordering
  matters.
- `feedback_handover_trailer_cohort_propagation.md` — non-binding
  under serial dispatch (no `[P]` cohorts in §13); per-task
  `HANDOVER:` commit trailer still recommended.
- `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`
  + `feedback_retro_task_complexity_score.md` — retro shape (final
  task; c-1 retro acknowledges c-2 dependency for behavioural
  validation).
- **`feedback_build_what_tests_exercise.md`** (PMD #14) —
  **load-bearing for c-1**. The restoration-escape detection branch
  is stub-only per DQ #145; `evaluate_escape_conditions`'s restoration
  check returns `Fire` until restorative-mechanics-v1 ships the
  producer. Plan §19 documents the stub-and-future-wire path.

### Related prior plans (canonical-shape mirror)

- `.claude/PRPs/plans/v1-sponsor-liability-c.plan.md` (trunk) —
  parent plan; c-1 inherits §1/§3/§4/§6/§7-§12/§17-§20 with
  c-1/c-2 annotations.
- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — schema
  foundation c-1 reads. c-1 §11 does not duplicate.
- `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` — most recent
  SL plan; §6 + §13 + §15 shapes mirrored.
- `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` — Shape-G reference.

---

## 3. Problem statement

(Inherited verbatim from trunk plan §3 with c-1 emphasis on what c-1
specifically delivers vs c-2.)

Post-SL-a-merge on `governance-v0` @ `790f6101d` (and post-SL-b-merge
expected before c-1 is cut):

- **The `sponsor_liability_grace.rs` module does not exist.** No
  scheduler module reads `SponsorLiabilityPending` cases past their
  `grace_expires_at` and transitions them. The 13 `liability.*` +
  `job.grace_check_*` config seeds shipped in SL-a (verified at
  `crates/api/api/src/governance/config.rs:925-930` + `:943-945` +
  the seed-row entries at `:1019-1056` + `:1373` etc.) have no reader.
  **(c-1 ships this module.)**
- **The `SponsorLiabilityFired` terminal state has no fire site.**
  Per the registry at `.claude/rules/governance-log-entry-kind-registry.md:172`,
  `_FIRED`'s emitting handler is "v1-SL-c
  `crates/api/api/src/governance/sponsor_liability_grace.rs::run_grace_check_batch`
  fire branch (pending)". The const `ENTRY_KIND_SPONSOR_LIABILITY_FIRED`
  exists at `crates/db_schema/src/source/governance/governance_log.rs:198`
  with shim re-export at `crates/api/api/src/governance/governance_log.rs:75`,
  but no emitter. The `(pending)` marker is unflipped. **(c-1 ships
  the fire site code; c-2 retro flips the registry marker after e2e
  tests exercise it.)**
- **The scheduler-branch fire-site for `_ESCAPED` is uncovered.** Per
  the registry at line 173, `_ESCAPED`'s handler is "v1-SL-b
  `revoke_endorsement.rs` (pending) AND v1-SL-c `sponsor_liability_grace.rs::evaluate_escape_conditions`
  (pending)". SL-b ships the revocation-branch fire-site; c-1 ships the
  scheduler-branch fire-site. **(c-1 ships the scheduler-branch
  escape code; c-2 retro flips the SL-c portion of the registry
  marker after e2e tests exercise it.)**
- **Cases in `SponsorLiabilityPending` are stuck.** Per
  `crates/db_schema_file/src/enums.rs:411-416` doc-comments, cases
  reach `SponsorLiabilityPending` via SL-d's `submit_jury_vote`
  rewrite OR via SL-a's mid-flight backfill (PRD §11.2). Neither
  population transitions to a terminal state without c-1's scheduler
  module + clokwerk wiring picking them up.
- **The `BREHON_DISABLE_GRACE_CHECK_JOB` test override is unhooked.**
  The pattern is precedented at `crates/routes/src/utils/scheduled_tasks.rs:197`
  (snapshot job's `BREHON_DISABLE_SNAPSHOT_JOB`) and `:258`
  (appeal-window-expiry's `BREHON_DISABLE_APPEAL_WINDOW_JOB`); c-1
  wires it. **c-2's e2e tests rely on the override.**
- **The third sibling concurrency-guard pair is missing.**
  `scheduled_tasks.rs:71-79` declares `REPUTATION_SNAPSHOT_RUNNING` +
  `RunningGuard`; lines 83-91 declare `APPEAL_WINDOW_EXPIRY_RUNNING` +
  `AppealWindowExpiryRunningGuard`. c-1 adds the third pair
  (`SPONSOR_LIABILITY_GRACE_RUNNING` + `GraceCheckRunningGuard`) so
  the new tick block can guard against overlap if a batch exceeds the
  5-minute interval.
- **The grace-window staleness observability surface is missing.** Per
  PRD §6.3, cases stuck >2× max grace window must emit `tracing::error!`
  for ops visibility. The pattern is precedented at
  `crates/api/api/src/governance/reputation_snapshot.rs:423`
  (`check_snapshot_staleness`); c-1 adds a sibling
  `check_grace_staleness` inside the new module.

The substrate is in place; c-1 supplies the new module + scheduler
block edits. **5 integration tests live in c-2 (NOT in c-1).**

---

## 4. Solution statement

(Inherited from trunk plan §4 with c-1 / c-2 task-split annotations.
The architectural decisions are the same; c-1 ships them in code, c-2
behaviourally validates.)

Three surgical changes for c-1, organised as 2 impl tasks + Task 0
pre-flight + retro.

### 4.1 Architecturally load-bearing decisions locked in this sub-phase

(All decisions inherited verbatim from trunk plan §4.1; reproduced in
full so c-1 is self-contained.)

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
  branch wired by restorative-mechanics-v1.
- **Two-tier `ConfigCache` lifetime per DQ #144.** Outer
  `ConfigCache::new()` declared at the top of `run_grace_check_batch`
  for batch-level reads (`job.grace_check_batch_size`,
  `liability.grace_window_maximum_hours`,
  `job.grace_check_staleness_alert_multiplier`). Per-case
  `ConfigCache::new()` declared at the top of `fire_or_escape_case`
  (inside the per-case scope) and threaded into
  `apply_sponsor_liability(... &mut cache)`. Mirrors
  `reputation_snapshot.rs:363/389` verbatim.
- **Inside-transaction step ordering (PRD §6.2 — load-bearing).**
  `fire_or_escape_case` body, inside one `run_transaction` closure
  per case:
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
     and return `Ok(())` (skip case silently).
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
         — single summary entry on TOP of v0's per-sponsor entries.
- **Outer `run_grace_check_batch` swallows per-case errors via
  `.inspect_err(|e| warn!(...)).ok()` pattern (per PRD §6.3 + per-case
  isolation invariant).** The for-loop body wraps the
  `fire_or_escape_case` call; on `Err`, log the case_id and continue.
  Outer function returns `Ok(...)` regardless of per-case outcomes.
- **Scheduler tick wiring at `scheduled_tasks.rs:setup` — third
  sibling block.** Pattern matches the snapshot block at lines 189-245
  and the appeal-window-expiry block at lines 254-274. Concrete shape
  per trunk plan §4.1 (verbatim — see trunk for the full code block).
- **`check_grace_staleness` formula per DQ #146.** `threshold_hours =
  liability.grace_window_maximum_hours × job.grace_check_staleness_alert_multiplier`
  computed inside the function from its `i64` + `f64` parameters.
  Per-case test: `now - decided_at > threshold_hours`. Defaults give
  720 × 2.0 = 1440h ≈ 60 days, matching PRD §6.3 "(>60 days)".
- **Sanction-action lookup query.** Verified 1:1 multiplicity at
  `crates/api/api/src/governance/submit_jury_vote.rs:435` (single
  `insert_into(sanction::table)` per case). c-1 queries
  `sanction::table.filter(case_id.eq(case_id)).order_by(id.asc()).limit(1)`.
- **Shape G — DoD references workflow YAMLs by path + expected
  `conclusion`, not inline cargo.** Every §13 task's DoD references
  `cargo-validate-workspace.yml` on the worker branch + workflow_run_id
  captured by impl-task subagent post-push.
- **No migration round-trip workflow fires.** c-1 touches no
  `migrations/**` paths. **No e2e workflow fires either** — Phase 2
  e2e is c-2's gate.

### 4.2 Watchpoints (specific files / handlers / schema lines)

(Inherited verbatim from trunk plan §4.2, with annotations on which
watchpoints apply to c-1 vs c-2.)

1. **Per-case `FOR UPDATE`. (c-1 binding — Task 1.)** Plan §13 Task 1 cites:
   `fire_or_escape_case` step 1 — re-load `moderation_case` row with
   `for_update()` inside the per-case transaction. Defends against
   scheduler-vs-handler race per
   `feedback_multi_write_handlers_need_transactions.md` and Phase 5a
   Watch 9 pattern. Without `FOR UPDATE`, SL-b's `revoke_endorsement`
   could mutate the case between batch query and per-case tx, causing
   a double-state-write.
2. **Re-check status inside transaction. (c-1 binding — Task 1.)**
   Plan §13 Task 1 specifies: after `FOR UPDATE`, re-read `case.status`;
   if no longer `SponsorLiabilityPending`, return `Ok(())` from the
   per-case body without UPDATE. Second half of the race defence.
3. **Atomic concurrency guard pattern. (c-1 binding — Task 2.)**
   Plan §13 Task 2 cites canonical mirror at `scheduled_tasks.rs:71-79`
   (`REPUTATION_SNAPSHOT_RUNNING` + `RunningGuard`) and 83-91
   (`APPEAL_WINDOW_EXPIRY_RUNNING` + `AppealWindowExpiryRunningGuard`),
   producing a sibling `SPONSOR_LIABILITY_GRACE_RUNNING` +
   `GraceCheckRunningGuard` pair. **Do NOT** reuse an existing static
   — different concurrency domains; sharing would deadlock-couple.
4. **`liability_escape_reason` JSON schema locked. (c-1 binding —
   Task 1, code; c-2 binding — Test #2 defensive assertion.)** Schema:
   `{"version": 1, "reason": "sponsor_revoked", "actor_pseudonym":
   "...", "endorsement_id": <i64>}`. `actor_pseudonym` mandatory per
   ADR-015; `version: 1` mandatory per OQ-V1-SL-05. Plan §13 Task 1
   cites `actor_pseudonym_helper::get_or_create` (sourced from
   `crates/api/api/src/governance/actor_pseudonym_helper.rs`) for
   pseudonym derivation. Raw `caller_id` / raw `from_person_id` in
   reason JSON is a GDPR-013 violation, catch-fire.
5. **Two log entries per fire path, one per escape path. (c-1 binding
   — Task 1, code; c-2 binding — Test #1 governance_log row count
   assertions.)** Fire path emits BOTH the per-sponsor
   `sponsor_liability_applied` entries (from v0
   `apply_sponsor_liability` at `sponsor_liability.rs:322-337` per
   sponsor) AND the SL-c summary `sponsor_liability_fired` entry
   (single, after the v0 helper returns). Escape path emits ONLY the
   `sponsor_liability_escaped` entry.
6. **Sanction-action lookup correctness. (c-1 binding — Task 1.)**
   Plan §13 Task 1 cites: query
   `sanction::table.filter(case_id.eq(case_id)).order_by(id.asc()).limit(1).select((action, scope)).first(conn).optional()`.
   If `None`, log `tracing::error!` and skip case silently.
7. **No new `moderation_case.status` mutations OUTSIDE the batch
   loop. (c-1 binding — Task 1.)** c-1 writes `status` ONLY inside
   `fire_or_escape_case`. MUST NOT mutate `Decided` cases, `Open`
   cases, or any other state.
8. **Per-case isolation — outer batch never returns `Err`. (c-1
   binding — Task 1, code; c-2 binding — Test #4 strong assertion.)**
   Plan §13 Task 1 specifies: per-case errors are caught via
   `.inspect_err(|e| warn!(...)).ok()`; iteration continues;
   `run_grace_check_batch` returns `Ok(...)` even if N-1 cases fail.
9. **`BREHON_DISABLE_GRACE_CHECK_JOB` env var precedes the atomic
   guard. (c-1 binding — Task 2.)** Plan §13 Task 2 specifies: env-var
   check is FIRST in the closure body, BEFORE `compare_exchange`.
   Mirror `BREHON_DISABLE_SNAPSHOT_JOB` at `scheduled_tasks.rs:197-199`
   and `BREHON_DISABLE_APPEAL_WINDOW_JOB` at `scheduled_tasks.rs:258-260`.
10. **No new ENTRY_KIND_*** consts. (c-1 binding — neither Task 1 nor
    Task 2 add to the registry.)** All needed consts shipped in SL-a:
    `ENTRY_KIND_SPONSOR_LIABILITY_FIRED` at line 198,
    `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` at 197,
    `ENTRY_KIND_RESTORATION_COMPLETED` at 196.
11. **No migration in c-1. (c-1 binding.)** Plan §13 produces zero
    migration files. `git diff governance-v0..phase-v1-SL-c-1 --
    migrations/` at plan-approval time must show no output.
12. **e2e Edit-per-task discipline. (applies to c-2 only — c-1 has
    zero e2e edits.)** 5 SL-c tests = 5 individual c-2 §13 tasks,
    each one anchor-pattern Edit at file end. Do NOT bundle. Per
    `feedback_junior_worker_e2e_edit_hang.md`. **c-1's §13 has no
    e2e tasks** — this watchpoint is non-binding for c-1; it's
    inherited as documentation only and binds c-2.
13. **R1 `i64::from(...)` discipline. (c-1 binding — Tasks 1 + 2.)**
    Per `feedback_clippy_test_style.md`, every `i32 ↔ i64` comparison
    uses `i64::from(...)`. c-1 surfaces:
    - `liability.grace_window_maximum_hours` (i64 via `get_int`)
      multiplied by `f64` multiplier → `f64` arithmetic; round to
      `i64` via `round()`. No `i32` intermediate.
    - `chrono::Duration::hours(threshold_hours)` requires `i64`.
    - `job.grace_check_batch_size` (i64 via `get_int`) → `usize`
      via `usize::try_from(...)` (mirrors
      `reputation_snapshot.rs:367-371`).
14. **Restoration-escape stub-only (DQ #145). (c-1 binding — Task 1.)**
    Plan §13 Task 1 specifies: `evaluate_escape_conditions` defines
    the `EscapeStatus::Escape{reason: "restoration_completed", ...}`
    enum value as a documented future-wire branch but the function
    body NEVER constructs it. The restoration-completed read returns
    `Fire` unconditionally. Per
    `feedback_build_what_tests_exercise.md` (PMD #14).

### 4.3 Rejected alternatives (c-1 only)

- **Bundle Task 1 (module) + Task 2 (scheduler) into one task.**
  Rejected per `feedback_pr_per_phase.md` one-commit-per-task
  discipline + per-task FILES YAML block + per-task workspace-check
  workflow run. Two logically distinct concerns get two commits.
- **Mark Task 1 + Task 2 as `[P]`.** Rejected: Task 2 (scheduler
  block in `scheduled_tasks.rs`) imports the symbols Task 1 creates
  in `sponsor_liability_grace.rs` (specifically
  `lemmy_api::governance::sponsor_liability_grace::run_grace_check_batch`
  and `::check_grace_staleness`). If Tasks 1+2 dispatch
  simultaneously, Task 2's worker branch is cut from `phase-v1-SL-c-1`
  before Task 1's commit lands; Task 2's compile fails with
  "unresolved import sponsor_liability_grace::*". Serial dispatch is
  correct.
- **Pre-implement the restoration-escape detection branch** (per
  PRD §6.2 step 3 second bullet). Rejected per DQ #145 advisor lean
  + `feedback_build_what_tests_exercise.md`. Stub only.
- **Add a `restoration_escapes_liability` config-key read in
  `evaluate_escape_conditions`.** Rejected per the same logic.
- **Use a per-batch `run_transaction` (single tx wrapping all
  cases).** Rejected per PRD §6.3 + per-case isolation invariant.
- **Use FOR UPDATE SKIP LOCKED in the batch query.** Rejected — the
  per-case tx pattern handles concurrency differently.
- **Define `EscapeStatus` as a struct with `Option<String> reason`
  field.** Rejected — enum variant is more expressive and
  exhaustive matching is cleaner.
- **Ship c-1 + c-2 as one plan.** Rejected per DQ #150 — user
  authorised the split at plan-approval gate.
- **Write the e2e tests against an unmerged c-1 worktree branch.**
  Rejected — c-2's tests need a stable module; the split exists
  precisely so c-2 can write tests against a merged c-1.

---

## 5. Metadata

- **Phase:** `v1-SL-c-1`
- **Branch:** `phase-v1-SL-c-1` (cut by BM-task before Task 1, AFTER
  SL-b PR #119 merges)
- **Estimated tasks:** 4 (Task 0 pre-flight + Tasks 1-2 impl + Task 3
  retro)
- **Estimated cargo budget:** 0 GB peak local (Shape G — cargo runs
  on GH-hosted runners)
- **Forbidden-window applicability:** non-binding for impl-task
  throughput under Shape G; standard for any local diagnostic cargo
  run (rare)
- **Complexity score:** **3/10** — see breakdown below.
  Below threshold; no split-or-proceed DQ.

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Computed mechanically
from §13 task list:

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | **0** | 2 impl tasks (Tasks 1, 2; Task 0 + retro excluded). `max(0, 2-5) = 0` |
| Migrations touched | +2 each | **0** | c-1 ships zero migrations |
| Crates touched | +1 each | **2** | `lemmy_api` (sponsor_liability_grace.rs + governance/mod.rs, Task 1), `lemmy_routes` (utils/scheduled_tasks.rs, Task 2). c-1 does NOT touch `crates/server/tests/e2e.rs` |
| `crates/lemmy_server/tests/e2e/*.rs` edits | +3 each | **0** | c-1 has zero e2e edits — those live in c-2 |
| New ADR-affecting decisions | +2 each | **0** | All ADR decisions made in trunk plan / DQ #144-#147 |
| Cargo budget peak above 6 GB | +1 per GB | **0** | Shape G — cargo runs off-box |
| Wrapped subtotal | — | **2** |  |
| Adjustment | — | **+1** | Fold-in: §4.2 watchpoint #14 (restoration stub-and-future-wire breadcrumb in §19); requires drift-stub discipline + §19 documentation overhead |
| **Total** | — | **3** | Threshold for split-DQ: `>8`. **Not tripped.** |

### 5.2 Split-or-proceed DQ — N/A

c-1 score 3 is well below threshold. No split-or-proceed DQ filed.

The split that produced this plan was directed by user override
(DQ #150) over the trunk plan's planner proceed-as-one lean (DQ #148).
That decision is recorded in DQ #150 + cited in §2 Source. Per
`feedback_principles_not_rules.md`: complexity score is a signal, not
a hard rule; user judgment overrode planner judgment at the
plan-approval gate.

---

## 6. Relationship to other v1-SL sub-phases

| Sub-phase | Status | What it ships | c-1 dependency |
|---|---|---|---|
| v1-SL-a | MERGED (PR #111, governance-v0 @ `790f6101d`) | Schema + 13 seeded keys + 5 entry-kind consts (incl. `_SPONSOR_LIABILITY_FIRED`, `_SPONSOR_LIABILITY_ESCAPED`, `_RESTORATION_COMPLETED`) + 3 CaseStatus variants + Issue #24 partial index + backfill | c-1 reads `moderation_case.{grace_expires_at, liability_escape_reason, severity, decided_at, target_person_id, community_id, status}`, `surety.{revoked_at, sponsor_id, sponsored_id}`, `sanction.{action, case_id}`, `governance_config.{job.grace_check_*, liability.grace_window_maximum_hours}`; fires `_SPONSOR_LIABILITY_FIRED` always (fire branch summary), `_SPONSOR_LIABILITY_ESCAPED` on scheduler-branch escape; reads `_RESTORATION_COMPLETED` for stubbed restoration-escape branch (never fires in c-1) |
| v1-SL-b | IN FLIGHT (PR #119 on `phase-v1-SL-b`; c-1 plan-write awaits SL-b merge) | `revoke_endorsement` handler + DTO + route + 9-10 e2e tests | SL-b's revocation-branch escape fires the SAME `_SPONSOR_LIABILITY_ESCAPED` const c-1's scheduler-branch escape fires. c-1 does NOT author SL-b's handler |
| **v1-SL-c-1 (THIS PLAN)** | NOT YET CUT | Scheduler module `sponsor_liability_grace.rs` (4 fns) + clokwerk tick wiring + atomic concurrency guard | — |
| v1-SL-c-2 | NOT YET CUT (depends on c-1 merge) | 5 e2e tests (`grace_check_*`) against the merged c-1 module + retro that flips `(pending) → (active)` registry markers | c-2 reads `pub async fn run_grace_check_batch`, `evaluate_escape_conditions`, `fire_or_escape_case`, `check_grace_staleness` from c-1. c-2 tests pre-seed `SponsorLiabilityPending` cases via direct DB-write and invoke `run_grace_check_batch` directly (per trunk plan §13 Tasks 3-7). c-1 ship → c-2 cut from governance-v0 after c-1 PR merges |
| v1-SL-d | PENDING (depends on c-1 + c-2 merges) | `submit_jury_vote` mutation + `apply_sponsor_liability` compute/fire split + `Decided → SponsorLiabilityPending` transition + `_SPONSOR_LIABILITY_PENDING` fire-site | SL-d transitions cases to `SponsorLiabilityPending` (the state c-2's tests pre-seed via direct DB-write). c-1 does NOT author SL-d's mutation |
| v1-SL-e | PENDING (depends on c-1 + c-2 + SL-d) | Lane-wide e2e suite (full grace-window flow exercising SL-d transition + c-1 scheduler) | c-2's tests pre-seed pending cases directly; SL-e's tests exercise the full lane (end-to-end through SL-d's transition) |
| restorative-mechanics-v1 | PENDING (separate PRD; not yet drafted) | `restoration_complete` endpoint + restoration-escape branch (defendant-initiated; admin-attested) | c-1 stubs the `EscapeStatus::Escape{reason: "restoration_completed", ...}` enum value but `evaluate_escape_conditions` never returns it. When restorative-mechanics-v1 ships the producer, that PRD's plan adds the read-side query AND a corresponding e2e test |

**Cross-PRD sequencing (per PRD §15 + §17.1):**

- c-1 parallel-safe with rep-tuning-r3/r4/r5 (different files +
  different concerns).
- c-1 parallel-safe with admin-dashboard-v1 (admin-config-write
  surface; c-1 READS the config keys, doesn't write).
- c-1 unblocks c-2 (which writes the e2e tests against c-1's module).
- c-2 unblocks SL-d (which writes the cases c-1's scheduler consumes)
  and SL-e (which exercises the full lane end-to-end).

---

## 7. Preflight guardrails inherited from prior phases

(Inherited from trunk plan §7. c-1 specifically applies: DQ #44 +
DQ #144-#147 + R1 + R5 + R6 + R7 + JM-d retro lessons + SL-a retro
lessons. R2 + R3 + R4 are non-binding for c-1 (no jury-assembly
tests, no DTO, no test names — those bind c-2).)

- **DQ #44 — Docker daemon preflight.** Enumerated in §13 Task 0
  (R5 — Probe 0). c-1's e2e is empty so testcontainers Docker
  isn't strictly needed at impl-time, but the workspace-check workflow
  doesn't use Docker either. Probe 0 still runs as a hygiene gate.
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
  returns `Fire` unconditionally. Bound in §13 Task 1 + §19 Notes.
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
- **DQ #150 — Split into c-1 + c-2 (binding).** Resolved (advisor
  on user authority): user override at plan-approval gate
  2026-05-07. c-1 ships module + scheduler wiring; c-2 ships e2e
  tests. The trunk plan stays in place as historical record.
- **R1 — `i64::from(...)` on `i32 ↔ i64`.** Bound in §4 watchpoint
  #13 + §13 Task 1 GOTCHAs.
- **R2 — `seed_jury_eligible_snapshots` BEFORE `admin_assign_jury` in
  tests.** Non-binding for c-1 (no tests in c-1).
- **R3 — struct-extension grep sweep.** Non-binding for c-1 (no
  DTO extensions; no new public types other than enum + outcome
  struct that have zero pre-existing literal-construction sites).
- **R4 — lowercase snake_case test names.** Non-binding for c-1 (no
  tests; binds c-2).
- **R5 — pre-phase harness audit enumerates ALL probes.** Bound in
  §13 Task 0.
- **R6 — uniform `--no-deps -- -D warnings` clippy.** Encoded in
  `cargo-validate-workspace.yml:92`.
- **R7 — `cargo test --no-run -p lemmy_server --test e2e` on tasks
  touching a struct or re-export.** Encoded in
  `cargo-validate-workspace.yml:95`. Bound in Task 1 (new pub fns +
  enum + outcome struct).
- **JM-d retro §3.5 — `feedback_clippy_rerun_after_fix.md`.** Bound
  in §13 Task 1 + Task 2 GOTCHA.
- **JM-d retro §5 — `feedback_laptop_default_for_validate_pending.md`.**
  Phase 2 e2e local-default per advisor-orchestrator user gate (PR
  #105, 2026-04-28). **For c-1: Phase 2 e2e is not triggered**
  (no new e2e tests).
- **SL-a retro §lessons — pseudonym discipline (ADR-015).** Bound in
  §4 watchpoint #4 + §13 Task 1 (escape branch payload schema).
- **SL-a retro §lessons — registry rule pre-landed-const exemption.**
  c-1 is the file-author for `_SPONSOR_LIABILITY_FIRED` and the
  scheduler-branch portion of `_SPONSOR_LIABILITY_ESCAPED`. Per
  registry rule + `feedback_build_what_tests_exercise.md`,
  `(pending)` markers stay until the const has a tested emitter.
  c-1 ships the code; c-2 retro flips the markers after e2e tests
  pass. c-1 retro acknowledges this hand-off explicitly.
- **SL-b retro lessons (expected).** Anything emerging from SL-b's
  retro (e.g. e2e fixture-mod naming convention) inherits forward
  into c-1 → c-2.

---

## 8. Flow design

(Inherited from trunk plan §8 — the architectural "before / after"
state diagram is identical because c-1 + c-2 together produce the
trunk's full deliverable. c-1 ships the scheduler-tick code and the
clokwerk wiring; c-2 ships the behavioural validation.)

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
  SL-b emits the revocation-branch; c-1 will emit the
  scheduler-branch.

### 8.2 After state (post-c-1-merge — code shipped, behavioural validation pending in c-2)

A new clokwerk tick (default 5 min) iterates pending-grace cases
**but is not yet exercised by tests** — the integration tests live
in c-2. The code is structurally correct (compile + clippy clean per
the workspace-check workflow), and the runtime invocation pattern
mirrors the two existing scheduler precedents.

(See trunk plan §8.2 for the full detailed flow diagram — the
architectural shape is identical; c-1 ships the code that produces
this flow.)

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

### 8.3 Endpoint changes

NONE. c-1 is server-internal scheduler module + clokwerk wiring.
Plan §13 must NOT include `crates/api/api_common/src/governance.rs`
edits or `crates/api/routes/src/lib.rs` edits.

---

## 9. Mandatory reading

(Inherited from trunk plan §9; subset binding to c-1 — sections
9.1, 9.2, 9.3, 9.4, 9.5 in full because c-1 implements the module
and the scheduler block. The fixture-mod section under §9.2 [`mod
v1_jm_e_fixtures`, `mod v1_sl_b_fixtures`] is non-binding for c-1
since c-1 ships no test code; it remains as documentation for c-2.)

### 9.1 Brehon design docs (P0)

- `.claude/PRPs/prds/v1-sponsor-liability.prd.md` §6 (full), §9.4,
  §9.5, §6.4, §6.3, §11.2, §12.4, §15 row 3, §17, §18, §3.1, §3.4,
  §7 (restoration interaction), §8.1 (`liability_escape_reason`
  schema)
- `.claude/PRPs/briefs/sl-c-planning-1.md` (trunk brief)
- `.claude/PRPs/briefs/sl-c-split-planning-1.md` (this plan's brief)
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  ADR-005, ADR-008, ADR-010, ADR-013, ADR-014, ADR-015,
  OQ-V1-SL-05, OQ-V1-SL-01

### 9.2 Codebase reads (P0 — mirror these patterns)

- `crates/api/api/src/governance/sponsor_liability.rs:1-358` —
  v0 `apply_sponsor_liability` signature + semantics; c-1 calls
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
  c-1's per-case).
- `crates/routes/src/utils/scheduled_tasks.rs:65-91` — three sibling
  guard patterns (`REPUTATION_SNAPSHOT_RUNNING`,
  `APPEAL_WINDOW_EXPIRY_RUNNING`, plus c-1's new
  `SPONSOR_LIABILITY_GRACE_RUNNING`).
- `crates/routes/src/utils/scheduled_tasks.rs:189-245` — snapshot
  scheduler block (canonical clokwerk + env-var disable + atomic
  guard + staleness pass).
- `crates/routes/src/utils/scheduled_tasks.rs:254-274` —
  appeal-window-expiry scheduler block (second sibling).
- `crates/api/api/src/governance/governance_log.rs:1-81` (api shim
  re-exports).
- `crates/db_schema/src/source/governance/governance_log.rs:196-199`
  — ENTRY_KIND_* const declarations c-1 uses.
- `crates/api/api/src/governance/actor_pseudonym_helper.rs` (full
  file — small) — `get_or_create` source for c-1's
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
  reason-validation pattern (citation-only; c-1 has no reason input).
- `crates/api/api/src/governance/submit_jury_vote.rs:435` — sanction
  insert site (1:1 multiplicity verification).

### 9.3 Rules (P0 — auto-loaded)

- `.claude/rules/decision-queue.md` — DQ schema-v2; attribution
  integrity; mid-task push; planner Recipe 2 self-resolution.
- `.claude/rules/branch-manager.md` — file-ownership boundaries; BM
  cuts `phase-v1-SL-c-1` AFTER SL-b merge.
- `.claude/rules/phase-branch.md` — phase-branch + PR-into-`governance-v0`
  flow.
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy`
  mandatory.
- `.claude/rules/governance-log-entry-kind-registry.md` —
  pre-landed-const exemption + retro-time marker flip discipline.
  **For c-1: registry markers stay `(pending)`** (no tests yet);
  c-2 retro flips them.
- `.claude/rules/cargo-output-capture.md` +
  `.claude/rules/no-cargo-output-paste.md` — cargo invocation
  discipline (relevant for §15.7 manual snippets).
- `.claude/rules/pre-phase-harness-audit.md` — Task 0 audit shape.

### 9.4 Lessons (P0 — bound to §13 decisions)

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

(Inherited verbatim from trunk plan §10 sub-sections binding to
c-1's deliverable: §10.1 outer batch runner, §10.2 per-case
transaction body, §10.3 atomic concurrency guard pair, §10.4
staleness check, §10.5 scheduler tick wiring, §10.6 governance_log
payload schemas, §10.7 sanction action lookup pattern. §10.8 test
fixture mod shape is non-binding for c-1 — that pattern is
referenced by c-2.)

Per `feedback_advisor_watchpoint_specificity.md` + DQ #147 (paired
canonical batch-runner mirrors).

### 10.1 Outer batch runner (canonical mirror — paired)

**SOURCE A:** `crates/api/api/src/governance/reputation_snapshot.rs:361-407`
(per-chunk pattern; `run_snapshot_batch`).

**SOURCE B:** `crates/api/api/src/governance/appeal_window_expiry.rs:34-87`
(per-row pattern; `run_appeal_window_expiry_batch`).

c-1's `run_grace_check_batch` is closer to SOURCE B (per-row → per-case
isolation) but adopts SOURCE A's two-tier ConfigCache pattern (per DQ
#144).

(See trunk plan §10.1 for the full code block — the mirror shape is
identical; reproduced by reference rather than by copy to keep this
plan's §10 lean.)

### 10.2 Per-case transaction body (canonical mirror)

**SOURCE:** `crates/api/api/src/governance/appeal_window_expiry.rs:55-78`
(per-row body pattern; c-1's per-case body is a richer variant).

(See trunk plan §10.2 for the full code block.)

### 10.3 Atomic concurrency guard pair (canonical mirror)

**SOURCE:** `crates/routes/src/utils/scheduled_tasks.rs:71-79` +
`:83-91` (two existing sibling pairs). c-1 adds a third.

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

### 10.4 Staleness check (canonical mirror)

**SOURCE:** `crates/api/api/src/governance/reputation_snapshot.rs:423-459`
(`check_snapshot_staleness`). c-1's `check_grace_staleness` mirrors
but the threshold formula differs per DQ #146.

(See trunk plan §10.4 for the full code block.)

### 10.5 Scheduler tick wiring (canonical mirror)

**SOURCE:** `crates/routes/src/utils/scheduled_tasks.rs:189-245`
(snapshot block) + `:254-274` (appeal-window-expiry block). c-1's
block is structurally identical with sponsor-liability-grace
nomenclature.

(See trunk plan §4.1 for the verbatim code block.)

### 10.6 governance_log payload schemas (fire + escape branches)

**SOURCE:** PRD §8.1 + DQ #144/#145/#146 resolutions.

(See trunk plan §10.6 for the full JSON schemas. ADR-015 pseudonym
discipline + ADR-013 enum-exhaustive match on `EscapeStatus` are the
load-bearing invariants.)

### 10.7 Sanction action lookup pattern

**SOURCE:** `crates/db_schema_file/src/schema.rs:1266-1278` + verified
1:1 multiplicity at `submit_jury_vote.rs:435`.

(See trunk plan §10.7 for the full query.)

### 10.8 Test fixture mod shape — N/A for c-1

c-1 ships no test code; this trunk-plan section binds c-2 instead.

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

### Meta files (reports)

- `.claude/PRPs/reports/v1-SL-c-1-retro.md` — CREATE retro per
  `feedback_retro_not_report.md` +
  `feedback_four_role_retro_signals.md` +
  `feedback_retro_task_complexity_score.md`. **Task 3**.

### Files explicitly NOT touched (c-1 only)

- `crates/server/tests/e2e.rs` — c-2's; c-1 has zero e2e test
  edits.
- `.claude/rules/governance-log-entry-kind-registry.md` — registry
  marker flips happen at c-2 retro time after e2e tests exercise
  the fire-sites; c-1's code lands but the markers stay `(pending)`
  per `feedback_build_what_tests_exercise.md`.
- `crates/api/api/src/governance/sponsor_liability.rs` — v0 helper
  stays intact through c-1; SL-d is the compute/fire split.
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
  — already ADR-013-extended in SL-a Task 5; c-1 does not touch
  any of these handlers.
- `crates/api/api_common/src/governance.rs` — no DTO additions
  (c-1 is server-internal).
- `crates/api/routes/src/lib.rs` — no route registration (c-1 is
  server-internal).
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
  `.coderabbit.yaml` — no dep / build-config / review-config changes.
- `.github/workflows/**` — no workflow YAML changes.

---

## 12. NOT building in v1-SL-c-1

(Inherited from trunk plan §12; c-1 specifically additionally excludes
the e2e tests + registry marker flips, which live in c-2.)

- **5 e2e tests (`grace_check_*`)** — c-2's deliverable. c-1's
  workspace-check workflow only confirms compile + clippy + test-no-run;
  behavioural validation lives in c-2.
- **Registry marker flips** (`_SPONSOR_LIABILITY_FIRED` (pending) →
  (active); `_SPONSOR_LIABILITY_ESCAPED` SL-c portion (pending) →
  (active)) — c-2 retro deliverable. Per
  `feedback_build_what_tests_exercise.md` (PMD #14): `(active)` marker
  requires test exercising, not just code presence. c-1 ships the
  code; c-2 ships the tests; c-2 retro flips.
- **`apply_sponsor_liability` compute/fire split** — SL-d's. c-1
  calls UNSPLIT v0 helper per advisor-locked posture (masthead of
  brief).
- **`submit_jury_vote` mutation: `Decided → SponsorLiabilityPending`
  transition** — SL-d's.
- **`revoke_endorsement` handler** — SL-b's; in flight on PR #119
  at c-1 plan-write time.
- **`restoration/complete` endpoint + restoration-escape branch
  detection** — restorative-mechanics-v1 PRD's. c-1 stubs the
  `EscapeStatus::Escape{reason: "restoration_completed", ...}` enum
  value but `evaluate_escape_conditions` never returns it in v1-SL-c
  (per DQ #145).
- **e2e behavioural tests for the full lane** (Decided → Pending →
  Fired/Escaped through SL-d transition + c-1 scheduler) — SL-e's.
- **Sponsor notification UX** — out per PRD §13 OQ-V1-SL-03.
- **Step-up auth for admin-driven scheduler runs** — out per PRD
  §12.3 (v2 reservation).
- **Cross-instance federation of grace-window events** — out per
  PRD §2 OUT + ADR-014.
- **New ENTRY_KIND_*** consts** — all needed shipped in SL-a.
- **New CaseStatus variants** — SL-a shipped 3.
- **New `governance_config` seeds** — SL-a shipped 13.
- **Schema migrations** — none.
- **Backfill of v0 cases** — SL-a Task 1 already backfilled.
- **`liability.multi_sponsor_escape_rule` reading in scheduler
  context.** SL-b reads this for revocation-branch escape;
  c-1's scheduler-branch escape uses a simpler `surety.revoked_at >=
  decided_at` check.

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task** (per
`feedback_pr_per_phase.md`). c-1 ships 2 impl tasks + Task 0
pre-flight + retro. No `[P]` cohorts (Tasks 1 + 2 serial because
Task 2 imports Task 1's symbols). Task 0 is always non-`[P]`.

> **Cohort dispatch (advisor-side):** No `[P]` cohorts in c-1. All
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

**Goal:** verify environment + branch (`phase-v1-SL-c-1`) +
SL-a + SL-b schema/handlers/seeds intact.

**FILES:**

```yaml
creates: []
modifies: []
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate ALL probes):**

```bash
# Probe 0 — Docker daemon (workspace-check workflow doesn't use Docker; hygiene gate only)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || echo "DOCKER NOT RUNNING (non-blocking for c-1; binds c-2)"

# Probe -1 — submodule init (Linux/Junior worktree quirk)
git submodule status > /tmp/sl-c-1-task0-submodule.log 2>&1
if grep -q '^-' /tmp/sl-c-1-task0-submodule.log; then
  git submodule update --init --recursive > /tmp/sl-c-1-task0-submodule-init.log 2>&1
  echo "submodule init exit: $?"
fi

# Probe 1 — branch verification
git branch --show-current
# EXPECT: phase-v1-SL-c-1 (BM-task cuts before Task 1)

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
test -f crates/api/api_crud/src/governance/revoke_endorsement.rs && echo "SL-b SHIPPED" || echo "SL-b NOT YET MERGED — c-1 can still ship its module + scheduler wiring; c-2 anchor-Edit fallback applies"

# Probe 8 — v0 apply_sponsor_liability signature confirmation
rg -n 'pub\(crate\) async fn apply_sponsor_liability' crates/api/api/src/governance/sponsor_liability.rs
# EXPECT: 1 match at line ~142; signature: (conn, target_person_id, case_id, community_id, action: SanctionAction, cache: &mut ConfigCache) -> LemmyResult<usize>

# Probe 9 — sanction multiplicity invariant (1:1 with case)
rg -n 'insert_into\(sanction::table\)' crates/api/api crates/api/api_crud
# EXPECT: 1 match at submit_jury_vote.rs:435 (single insert per case — 1:1 multiplicity)

# Probe 10 — REPUTATION_SNAPSHOT_RUNNING + APPEAL_WINDOW_EXPIRY_RUNNING patterns confirmed
rg -n 'REPUTATION_SNAPSHOT_RUNNING|APPEAL_WINDOW_EXPIRY_RUNNING' crates/routes/src/utils/scheduled_tasks.rs | head
# EXPECT: at least 4 matches (declarations + Drop impls + compare_exchange usages)

# Probe 11 — PM-plugin-hooks-stable check (per .claude/rules/pm-plugin-hooks-stable.md)
for h in \
  local_private_message_before_create \
  local_private_message_after_create \
  local_private_message_before_update \
  local_private_message_after_update \
  federated_private_message_before_receive \
  federated_private_message_after_receive; do
  rg -q "\"$h\"" crates/ || { echo "missing hook literal: $h"; exit 1; }
done

# Probe 12 — concurrent-PR check
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("sponsor_liability_grace\\.rs|scheduled_tasks\\.rs|governance/mod\\.rs")) | {number, title, headRefName}'
# EXPECT: empty output (no concurrent PRs touching c-1-owned files)

# Probe 13 — Shape G workflow YAMLs accessible (yamllint may not be installed; soft-fail)
yamllint .github/workflows/cargo-validate-workspace.yml > /tmp/sl-c-1-task0-yamllint.log 2>&1 || \
         echo "yamllint not installed or warnings — non-blocking; advisor verifies workflow shape pre-merge"

# Probe 14 — DQ pending status (advisory)
python3 -c "import json; d=json.load(open('.claude/decision-queue.json')); print('pending:', [(e['id'], e.get('kind'), e.get('from')) for e in d.get('pending',[])])"
```

**EXPECT:** Probes 1..12 exit 0. Probes 0, 13, 14 are informational.

**No commit at Task 0** — verification only.

### Task 1: Create `sponsor_liability_grace.rs` module + module wiring

**ACTION:** in `crates/api/api/src/governance/`, create new file
`sponsor_liability_grace.rs` with the module doc-comment + 4 public
async fns (`run_grace_check_batch`, `evaluate_escape_conditions`,
`fire_or_escape_case`, `check_grace_staleness`) + `EscapeStatus`
public enum + `GraceCheckBatchOutcome` public struct +
`PerCaseOutcome` private enum + `fire_or_escape_case_inner` private
fn per §10.1, §10.2, §10.4 (all carried verbatim from trunk plan §10).
In `crates/api/api/src/governance/mod.rs`, add `pub mod
sponsor_liability_grace;` alphabetically AFTER `pub mod
sponsor_liability;` (verified at `mod.rs:38`).

**FILES:**

```yaml
creates:
  - crates/api/api/src/governance/sponsor_liability_grace.rs
modifies:
  - crates/api/api/src/governance/mod.rs   # add pub mod sponsor_liability_grace; alphabetically
```

**IMPLEMENT (file 1 of 2):** in
`crates/api/api/src/governance/sponsor_liability_grace.rs`, write
the full module body per trunk plan §10.1 (`run_grace_check_batch`)
+ §10.2 (`fire_or_escape_case` + `fire_or_escape_case_inner`) + §10.4
(`check_grace_staleness`) + the `evaluate_escape_conditions` body.

(Module doc-comment shape, use block, public types, full async fn
bodies — all carried verbatim from trunk plan §13 Task 1's IMPLEMENT
block. Do not duplicate here; the trunk is the canonical source. The
plan-implement subagent reads the trunk's task 1 body and writes
identical Rust. The split is purely a §13-task-list partition; the
code emitted is the same as if proceed-as-one had been chosen.)

**IMPLEMENT (file 2 of 2):** in
`crates/api/api/src/governance/mod.rs`, after line 38 (`pub mod
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

**GOTCHA (ADR-013 exhaustive match on EscapeStatus):** the `match
escape` arms `Escape{...}` AND `Fire` only. NO `_ =>` arm.

**GOTCHA (DQ #144 — two-tier ConfigCache):** outer cache at top of
`run_grace_check_batch`; per-case cache at top of
`fire_or_escape_case_inner`, threaded into
`apply_sponsor_liability(... &mut per_case_cache)`.

**GOTCHA (DQ #145 — restoration-escape stub):** the
`evaluate_escape_conditions` function body has the comment block
explaining the future branch + the `let _ = case_id;` hint to
suppress unused-var lints. The function literally returns `Fire` if
no surety revocation matches.

**GOTCHA (clippy rerun per `feedback_clippy_rerun_after_fix.md`):**
new module + new pub fns may trip `unused-imports` lint on
intermediate iterations. The cargo-validate-workspace workflow's
clippy step catches.

**GOTCHA (R7 — struct/fn additions affect re-exports):** the new
`pub` fns + types are visible to downstream crates via the
`crates/api/api/src/governance/mod.rs` `pub mod` declaration. The
workspace-check workflow's `cargo test --no-run -p lemmy_server
--test e2e` step (line 95) catches any cross-crate compile
regression.

**GOTCHA (`Selectable` derive on ModerationCase):** the struct at
`crates/db_schema/src/source/governance/moderation_case.rs:13-94`
already derives `Identifiable, Queryable, Selectable`. c-1's batch
query uses `ModerationCase::as_select()` — works.

**GOTCHA (cargo-features parity):** the new module + types live
behind no `#[cfg(feature = ...)]` gate.

**GOTCHA (split-mode — no e2e validation in c-1):** the
workspace-check workflow runs `cargo test --no-run -p lemmy_server
--test e2e` per line 95, which only confirms cross-crate compile of
e2e.rs. **It does NOT execute the e2e tests** — c-2 ships the new
tests. Nothing in c-1 calls `run_grace_check_batch` from a test
context. The function is still tested for compilation correctness.

**Push and exit (Shape G):** push to origin/`junior/<task-slug>`;
impl-task subagent writes `kind: "validate-pending"` DQ entry
capturing `workflow_run_id` of `cargo-validate-workspace.yml`.

**COMMIT MESSAGE:** `feat(v1-SL-c-1): create sponsor_liability_grace module + 4 pub fns + module wiring (task 1)`

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
`crates/routes/src/utils/scheduled_tasks.rs` — full code blocks
carried verbatim from trunk plan §13 Task 2 IMPLEMENT block (module-
scope guard pair + scheduler tick block).

(See trunk plan §13 Task 2 IMPLEMENT for the verbatim code. The
plan-implement subagent reads the trunk's body and writes identical
Rust. Split mode does not change the emitted code.)

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
`APPEAL_WINDOW_EXPIRY_RUNNING`.

**GOTCHA (Watchpoint #9 — env-var FIRST):** the
`std::env::var("BREHON_DISABLE_GRACE_CHECK_JOB")` check is the FIRST
statement in the `async move` body, BEFORE `compare_exchange`.

**GOTCHA (clokwerk pinning):** the `get_int` call for
`grace_interval_minutes` runs ONCE at `setup()` invocation. The
`scheduler.every(...)` consumes a `u32` and pins the schedule.

**GOTCHA (`u32::try_from(i64)`):** clokwerk's `minutes(...)` takes
`u32`. `unwrap_or(5)` on the conversion is the same defensive
default as `unwrap_or(5)` on `get_int` failure.

**GOTCHA (Watchpoint #5 — staleness reads INSIDE closure):** the
`max_grace_hours` + `multiplier` reads happen INSIDE the closure
body (per-tick), not at `setup()` time.

**GOTCHA (`Selectable` cross-feature):** the
`lemmy_api::governance::sponsor_liability_grace` import path resolves
correctly because Task 1 wired the module via `pub mod
sponsor_liability_grace;` in `governance/mod.rs`. If Task 1's
`pub mod` line is missing, this Task 2 push fails with "unresolved
module" — the validate-pending workflow catches.

**GOTCHA (R7 propagation):** new module + scheduler block are
visible in the `lemmy_routes` crate. `cargo test --no-run` step in
the workspace-check workflow validates cross-crate compile.

**GOTCHA (clippy rerun per `feedback_clippy_rerun_after_fix.md`):**
new clokwerk block may trip `clippy::redundant-closure-call` or
similar; let workflow run identify; re-run on next push if needed.

**GOTCHA (no e2e fires in c-1):** the workspace-check workflow's
`cargo test --no-run` step validates compile of `e2e.rs`'s existing
tests against the new module + scheduler import; it does NOT
execute c-2's tests because they don't exist yet on this branch.

**Push and exit (Shape G):** push to origin/`junior/<task-slug>`;
impl-task subagent writes `kind: "validate-pending"` DQ entry.

**COMMIT MESSAGE:** `feat(v1-SL-c-1): wire sponsor_liability_grace scheduler block + atomic concurrency guard (task 2)`

### Task 3: Retro

**Goal:** author retro per `feedback_retro_not_report.md` +
`feedback_four_role_retro_signals.md` +
`feedback_retro_task_complexity_score.md`. **Do NOT flip registry
markers** — those flip in c-2 retro after e2e tests exercise the
fire-sites (per `feedback_build_what_tests_exercise.md`).

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-SL-c-1-retro.md
modifies: []
```

**IMPLEMENT (file 1 of 1):** in
`.claude/PRPs/reports/v1-SL-c-1-retro.md`, write the retro per the
canonical 4-role format (Advisor / Planning / Impl / BM). Include:

- §1 Summary: c-1 shipped (module + scheduler wiring + retro). Story
  status — Story 1 ✓. Stories 2 + 3 are c-2 deliverables.
- §2 Per-role signals (4 H2 sections; each lists "what worked" +
  "what surprised us" + "what should change next").
- §3 Carry-forward — explicit hand-off to c-2:
  - Module signatures locked at Task 1 commit SHA `<sha>`.
  - `cargo test --no-run -p lemmy_server --test e2e` confirmed
    cross-crate compile clean — c-2's anchor-Edit-per-test pattern
    is safe.
  - Pseudonym discipline (ADR-015) implemented in the escape
    branch — c-2's Test #2 defensive assertion is the strong
    signal.
  - Per-case isolation pattern (`.inspect_err`) implemented — c-2's
    Test #4 is the strong signal.
  - Restoration-stub (DQ #145) implemented — c-2 should NOT add a
    test for it; restorative-mechanics-v1 owns that.
  - Registry markers (`_FIRED`, `_ESCAPED` SL-c portion) STAY
    `(pending)` after c-1 retro. **c-2 retro flips them** after
    e2e tests pass.
- §4 Per-task complexity-score table (mandatory per
  `feedback_retro_task_complexity_score.md`):
  `| task | files-changed | commits | runtime-min | max-log-silence-min |`
  for each of Tasks 0..3.
- §5 Lessons promotion — any new `feedback_*.md` candidates from
  the c-1 ship.
- §6 Acceptance — confirm all checkboxes from §17.

**Cross-cutting verification (Task 3 retro time — invariants):**

- `rg -c '^pub const ENTRY_KIND_'
  crates/db_schema/src/source/governance/governance_log.rs` returns
  the same total as post-SL-b state (unchanged).
- `rg '^\s+ENTRY_KIND_'
  crates/api/api/src/governance/governance_log.rs | wc -l` returns
  the same total (unchanged shim re-exports).
- `rg -n 'pub mod sponsor_liability_grace;'
  crates/api/api/src/governance/mod.rs` returns 1 line.
- `rg -n 'pub async fn run_grace_check_batch|pub async fn evaluate_escape_conditions|pub async fn fire_or_escape_case|pub async fn check_grace_staleness'
  crates/api/api/src/governance/sponsor_liability_grace.rs` returns
  4 lines.
- `rg -n 'SPONSOR_LIABILITY_GRACE_RUNNING'
  crates/routes/src/utils/scheduled_tasks.rs` returns at least 3
  lines (declaration + Drop + compare_exchange).
- `cargo build` workspace exit 0 via `cargo-validate-workspace.yml`
  on the phase-branch tip.
- `/brehon-verify` reports Story 1 `[done]`.
- `git diff governance-v0..phase-v1-SL-c-1 -- migrations/` returns
  empty (zero migrations).
- `git diff governance-v0..phase-v1-SL-c-1 -- crates/server/tests/`
  returns empty (zero e2e edits — confirms c-1 / c-2 split discipline).

**MIRROR:** `.claude/PRPs/reports/v1-SL-b-retro.md` (predecessor;
expected to exist post-SL-b ship); `.claude/PRPs/reports/v1-SL-a-retro.md`;
`.claude/PRPs/reports/v1-JM-e-retro.md` for section structure.

**GOTCHA (retro before PR):** retro is written BEFORE `gh pr
create` per `feedback_retro_not_report.md`.

**GOTCHA:** §4 per-task complexity-score table is mandatory.

**GOTCHA (no registry flip in c-1):** the registry-marker flip is
explicitly deferred to c-2 retro per
`feedback_build_what_tests_exercise.md`. c-1 retro §3 carry-forward
makes this hand-off explicit.

**Push and exit (Shape G — retro is meta-work; no `crates/**`
change → `cargo-validate-workspace` does not trigger).**

**COMMIT MESSAGE:** `docs(v1-SL-c-1): phase retrospective — module + scheduler wiring shipped (task 3)`

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
  re-exports + use-block paths). c-1 has zero e2e edits, so the
  test compile re-validates the existing e2e.rs against the new
  module + scheduler symbols.
- **Migration round-trip:** N/A — c-1 ships zero migrations.
- **e2e execution (Phase 2):** **N/A for c-1** — c-1 ships zero new
  e2e tests. Phase 2 e2e gate is c-2's deliverable. Existing e2e
  tests on `governance-v0` continue to pass (the workspace-check
  workflow's `cargo test --no-run` step confirms compile; behavioural
  re-execution of existing tests is not required for c-1's merge —
  the existing tests passed when SL-b merged and the new c-1 module
  doesn't touch their fixtures).

Pre-merge advisor-side verification: Story 1 checkpoint is
workspace-check workflow `conclusion: "success"` on Task 2's push
(module + scheduler wiring landed); `/brehon-verify` Brief-Scope
outputs check.

### 14.1 Pre-existing tests preserved

- All SL-b-shipped tests (`mod v1_sl_b_fixtures::*`, 9-10 tests
  post-PR #119) — preserved verbatim
- All SL-a-shipped tests — preserved verbatim
- All JM-e-shipped tests (`mod v1_jm_e_fixtures::*`) — preserved
- All JM-c/JM-b-shipped tests — preserved
- `governance_log_hash_chain_holds` at e2e.rs:907 — preserved
- All endorsement-creation tests (Phase 5b) — preserved
- All snapshot-job tests — preserved
- All appeal-window-expiry-job tests — preserved

### 14.2 Edge cases covered (c-1)

- **Compile-time correctness** of the new module + scheduler block
  via the workspace-check workflow.
- **Clippy correctness** of the new module + scheduler block via
  `--no-deps -- -D warnings`.
- **Cross-crate import correctness** via `cargo test --no-run -p
  lemmy_server --test e2e` (Tasks 1, 2 only — confirms downstream
  crates can resolve the new symbols).

c-1's e2e ship-list is empty by design.

### 14.3 Edge cases covered (deferred to c-2)

- Fire path behavioural validation — c-2 Test #1.
- Escape path behavioural validation — c-2 Test #2.
- No-op (future grace_expires_at) — c-2 Test #3.
- Per-case isolation — c-2 Test #4.
- Batch-size config — c-2 Test #5.

---

## 15. Validation commands (DoD)

> **Shape G plan — DoD is per-workflow, not inline cargo.** Per
> `.claude/PRPs/templates/plan.template.md` §15.6. c-1 ships zero
> migrations, so `cargo-validate-migration.yml` does not fire.
> **c-1 also ships zero new e2e tests, so `cargo-test-e2e.yml` does
> not fire automatically** — Phase 2 e2e is c-2's gate.

### 15.1 Per-task workspace check (Shape G)

For every §13 impl task that modifies `crates/**` (Tasks 1, 2):

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

c-1 ships zero migrations.

### 15.3 Phase 2 e2e — N/A for c-1

c-1 ships zero new e2e tests; the `grace_check_*` test suite lives
in c-2. Phase 2 e2e gate (local-vs-dispatch user gate per
`advisor-orchestrator.md`) is c-2's deliverable. **Advisor's
plan-approval gate for c-1 does NOT run a Phase 2 e2e check.**

### 15.4 Cross-cutting verification (Task 3 retro time)

- [ ] `rg -c '^pub const ENTRY_KIND_'
  crates/db_schema/src/source/governance/governance_log.rs` returns
  the same total as post-SL-b state (unchanged).
- [ ] `rg '^\s+ENTRY_KIND_'
  crates/api/api/src/governance/governance_log.rs | wc -l` returns
  the same total (unchanged shim re-exports).
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
- [ ] R1: every `i32 ↔ i64` comparison in Tasks 1 + 2 uses
  `i64::from(...)` if cross-type comparison arises.
- [ ] R6: clippy invocations in `cargo-validate-workspace.yml` use
  `--no-deps -- -D warnings`.
- [ ] No edits to files outside §11 list.
- [ ] Story 1 (c-1's only story) is `[done]`.
- [ ] No new ENTRY_KIND_*** consts under
  `crates/db_schema/src/source/governance/governance_log.rs`.
- [ ] No new migrations: `git diff governance-v0..phase-v1-SL-c-1
  -- migrations/` returns empty.
- [ ] No e2e edits: `git diff governance-v0..phase-v1-SL-c-1 --
  crates/server/tests/` returns empty (c-1 / c-2 split discipline).
- [ ] Registry markers UNCHANGED — `_SPONSOR_LIABILITY_FIRED` row
  + `_SPONSOR_LIABILITY_ESCAPED` SL-c portion BOTH still
  `(pending)`. (c-2 retro flips after e2e validation.)

> **Note (vs trunk):** the trunk plan's §15.4 includes a
> `rg -c '#\[tokio::test\]\s*async fn grace_check_'` assertion
> expecting count = 5. **c-1 §15.4 omits this command** — c-1's
> grace_check_ test count is 0 by design; the assertion is
> meaningless until c-2 ships. c-2's §15.4 keeps the assertion
> with expected count 5.

### 15.5 ADR / OQ compliance verification

- [ ] **ADR-005** honoured — fire branch's
  `apply_sponsor_liability` writes per-dimension reputation_event
  rows. (Code shipped in c-1; behavioural validation in c-2 Test #1.)
- [ ] **ADR-008** honoured — every emit goes through
  `governance_log::append`; no direct INSERT to `governance_log`
  table.
- [ ] **ADR-010** honoured — won't-disadvantage rule preserved;
  c-1 reads SL-a's mid-flight backfill cases without retroactive
  invalidation.
- [ ] **ADR-013** honoured — exhaustive `EscapeStatus` match in
  `fire_or_escape_case_inner` enumerates `Escape{...}` AND `Fire`;
  no `_ =>` arm.
- [ ] **ADR-014** honoured — no federation outbound on
  `sponsor_liability_fired` or `sponsor_liability_escaped`.
- [ ] **ADR-015** honoured —
  `liability_escape_reason` JSONB carries `actor_pseudonym` (string),
  NOT raw `caller_id`. (Code shipped; defensive Test #2 in c-2.)
- [ ] **OQ-V1-SL-05** honoured — `liability_escape_reason` JSON
  carries `version: 1` from day one.
- [ ] **PRD §6** code shipped (scheduler tick + per-case isolation +
  staleness check). Behavioural validation in c-2.
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

**Phase 1b (migration round-trip):** N/A — no migrations in c-1.

**Phase 2 (e2e):** N/A for c-1 — no new e2e tests; c-2's gate.

### 15.7 Manual validation snippets (advisor-side smoke only — non-binding)

> Per `feedback_features_full_p_crate_incompatible.md`: never `-p
> <crate>` + `--features full`; use `--workspace --features full`.

```bash
ls .github/workflows/cargo-validate-workspace.yml

gh run list --repo barrie-cork/lemmy \
  --branch phase-v1-SL-c-1 \
  --workflow cargo-validate-workspace \
  --limit 1 --json conclusion,databaseId

cargo check --workspace --features full > /tmp/sl-c-1-check.log 2>&1
status=$?
tail -20 /tmp/sl-c-1-check.log
echo "exit: $status"

rg -n 'pub async fn run_grace_check_batch' crates/api/api/src/governance/sponsor_liability_grace.rs
# EXPECT: 1 match

rg -n 'pub mod sponsor_liability_grace;' crates/api/api/src/governance/mod.rs
# EXPECT: 1 match
```

These are advisor-side only; not §16 acceptance criteria.

---

## 16. Acceptance criteria

- [ ] All 4 tasks (Task 0, Tasks 1-2, Task 3 retro) committed.
- [ ] §15.1 (`cargo-validate-workspace.yml`) `conclusion: "success"`
  after every impl task push (Tasks 1-2).
- [ ] §15.2 (migration round-trip) — N/A (zero migrations).
- [ ] §15.3 (Phase 2 e2e) — N/A for c-1 (no new e2e tests).
- [ ] §15.4 (cross-cutting verification — 14 boxes) all ticked.
- [ ] §15.5 (ADR / OQ compliance — 10 boxes) all ticked.
- [ ] §16a Story 1 `[done]`.
- [ ] No edits to files outside §11 list.
- [ ] Retro committed per Task 3.
- [ ] PR opens against `governance-v0` (NOT `main`) with
  `--repo barrie-cork/lemmy`.
- [ ] `/brehon-verify` report at
  `.claude/PRPs/reports/v1-SL-c-1-verify.md` shows Story 1 ✓.
- [ ] DQ #150 (split override) cited in plan §2 + §5.2; resolved
  in `decision-queue.json` resolved array.
- [ ] Registry markers UNCHANGED (still `(pending)` for
  `_SPONSOR_LIABILITY_FIRED` + `_SPONSOR_LIABILITY_ESCAPED` SL-c
  portion); c-2 retro flips them.

---

## 16a. Stories (independently-testable behaviour units)

Per `.claude/PRPs/templates/plan.template.md` §16a +
`feedback_story_grain_checkpoint.md` +
`feedback_brehon_verify_pre_merge.md`. **c-1 ships ONE story** (the
module + scheduler wiring as a single behaviourally-distinct unit
verifiable structurally). Stories 2 + 3 from the trunk plan ship
in c-2.

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

> **Verification mapping:** `/brehon-verify` iterates Story 1's
> Brief-Scope outputs and confirms each exists + matches its
> structural pattern on the c-1 worktree branch. Story 2 + 3 are
> verified by `/brehon-verify` against the c-2 worktree branch
> after c-2 ships.

### Stories 2 + 3 — N/A for c-1

c-2 deliverables:

- **Story 2** (Fire branch transitions case to `SponsorLiabilityFired`
  with both per-sponsor and summary log entries + writes
  `reputation_event` rows) — c-2 Task 1 (Test #1).
- **Story 3** (Escape branch + per-case isolation + batch-size config
  + future-grace no-op behave correctly) — c-2 Tasks 2, 3, 4, 5
  (Tests #2 through #5).

c-1's verify report mentions Stories 2 + 3 as deferred to c-2.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0..14 confirmed).
- [ ] Tasks 1..2 committed.
- [ ] Task 3 retro committed.
- [ ] §15 validation green at every gate (Phase 1 workspace check
  ×2; Phase 2 e2e N/A for c-1).
- [ ] §16a Story 1 `[done]`.
- [ ] PR opened by BM session against `governance-v0`.
- [ ] CodeRabbit review complete with findings triaged per
  `feedback_pr_review_triage_pattern.md`.
- [ ] `/brehon-verify` report at
  `.claude/PRPs/reports/v1-SL-c-1-verify.md` shows Story 1 ✓.
- [ ] Post-merge phase branch retained for c-2 to read at retro
  time (cross-PR carry-forward).
- [ ] Registry markers UNCHANGED (still `(pending)` for
  `_SPONSOR_LIABILITY_FIRED` + `_SPONSOR_LIABILITY_ESCAPED` SL-c
  portion); c-2 retro flips them.

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Task 2 (scheduler block) dispatches before Task 1 (module) finalize-merges; compile fails on `unresolved import sponsor_liability_grace::*` | LOW | MED | Tasks 1 + 2 dispatch serially (no `[P]` cohort); per `advisor-orchestrator.md` cohort-dispatch rule, advisor waits for prior task finalize-merge before next dispatch |
| SL-b not yet merged at c-1 plan-implement time | LOW | LOW | c-1 does not depend on SL-b's e2e tests or fixture mods; only on SL-b's revoke_endorsement handler being absent from Task 1's escape-branch code path. Probe 7 detects |
| Sanction multiplicity drift (1:1 → many-to-one in future SL-d/SL-e refactor) | LOW | LOW | Watchpoint #6 — `ORDER BY id ASC LIMIT 1` is defensive |
| `liability_escape_reason` JSONB schema regression (raw `caller_id` instead of `actor_pseudonym`) | LOW | HIGH | §4 watchpoint #4 in code; **defensive ADR-015 assertion lives in c-2 Test #2** — c-1 ships the code; c-2 catches regressions. c-1's structural Brief-Scope outputs assert `actor_pseudonym_helper::get_or_create` is referenced |
| TOCTOU on case status (concurrent SL-b revocation between batch query and per-case tx) | LOW | HIGH | §4 watchpoint #1 — FOR UPDATE lock + status re-check inside per-case tx; structural correctness via the per-case tx body shape (no e2e test for the race itself in either c-1 or c-2 — single-threaded test harness) |
| Per-case error propagation breaks isolation (one bad case blocks batch) | LOW | HIGH | §4 watchpoint #8 in code; **Test #4 strong assertion lives in c-2** |
| `BREHON_DISABLE_GRACE_CHECK_JOB` env var leaks atomic-bool slot if order reversed | LOW | MED | §4 watchpoint #9 + Task 2 GOTCHA — env-var check FIRST in closure body, BEFORE compare_exchange |
| Restoration-stub branch accidentally fires (despite DQ #145 advisor lock) | LOW | HIGH | §4 watchpoint #14 — `evaluate_escape_conditions` literally returns `Fire` if no surety revocation matches; restoration-completed read is a comment block + `let _ = case_id;` placeholder. Function body asserted by Story 1 Brief-Scope structural pattern |
| Staleness threshold formula error (h vs s confusion) | LOW | LOW | DQ #146 + §10.4 — formula `threshold_hours = max_grace_hours × multiplier`; `chrono::Duration::hours(threshold_hours_i)`. Defaults give 1440h ≈ 60d matching PRD §6.3 |
| ts-rs derive regression (export breaks downstream client codegen) | LOW | LOW | c-1 adds zero ts-rs derives. Workspace-check workflow `cargo test --no-run` step catches any cross-crate regression |
| GH-Actions minutes budget exceeded by 2 Phase-1 workspace checks | LOW | LOW | c-1's two workspace-check runs ≈ 8-10 min combined; well within Free 3000 min |
| Junior worker pre-pushes break finalize-merge | LOW | LOW | Junior daemon's finalize step is post-Shape-G correct (per JM-d retro §1.1) |
| ADR-013 enum-exhaustiveness lint trips on a new match site c-1 accidentally introduces | LOW | LOW | §4 watchpoint #7 — c-1 uses Diesel `.filter(status.eq(SponsorLiabilityPending))`, not `match case.status`. New `match escape: EscapeStatus` is enumerated exhaustively |
| `apply_sponsor_liability` v0 helper signature drift between brief-write and impl-time | LOW | LOW | Probe 8 verifies signature at task 0; if drift, Task 0 STOP and impl-task files DQ before Task 1 |
| **c-1 ships unexercised code** (compile-clean but never invoked at runtime by any test until c-2 ships) | MED | LOW | This is the design. c-1 is split from c-2 precisely so c-2 can write tests against a merged c-1. The window between c-1 PR-merge and c-2 PR-merge is the unexercised period; Junior daemon does not ship code during this window without c-2 in flight. Risk monitored as a one-time c-1 → c-2 hand-off correctness gate |
| **c-2 not authored before c-1 ships** (orphaned code on `governance-v0`) | LOW | LOW | c-2 plan is authored in the same commit as c-1 plan (both files at canonical paths in `.claude/PRPs/plans/`). c-2 plan's existence pre-conditions c-1's PR opening |

---

## 19. Notes

### 19.1 Planner DQs filed (c-1)

- **None at planning time.** c-1 score 3 below threshold (no
  split-or-proceed DQ). DQ #150 records the user override directing
  the split into c-1 + c-2; that DQ was advisor-resolved before this
  plan was authored.

### 19.2 Self-resolved planner findings (LESSON candidates — c-1)

- **The c-1 / c-2 split was directed by user override at the
  plan-approval gate, not by planner judgment.** This is the first
  in-fork example of an at-write-time split per
  `feedback_complexity_score_pre_split.md` precedent — DQ #150's
  resolution drives the structural shape.
- **The split's mechanical execution is straightforward at the
  Task-2/3 boundary** of the trunk plan because the trunk's task
  list naturally divides between "module + wiring" (Tasks 1-2) and
  "e2e tests" (Tasks 3-7). The split would have been harder if the
  Tasks 3-7 e2e tests were interleaved with module-level code edits
  in the trunk's task list — they aren't, so the split is clean.
- **The split's main cost is documentation overhead** (c-1 retro
  must explicitly carry-forward the Tasks 3-7 e2e expectations to
  c-2; c-2 plan must consume c-1 as a merged dependency). The
  benefit is tighter PR scope (c-1 is 2 impl tasks; c-2 is 5 impl
  tasks; together they're trunk's 7 impl tasks but each closes
  separately).
- **Registry-marker flip discipline correctly defers to c-2 retro.**
  Per `feedback_build_what_tests_exercise.md` (PMD #14): `(active)`
  marker requires test exercising, not just code presence. c-1 ships
  the code but no tests; the markers stay `(pending)` after c-1
  retro and flip after c-2 ship. This is a clean application of
  the lesson and the first in-fork instance of a split where the
  registry-flip timing matters.

### 19.3 Restoration-escape stub-and-future-wire breadcrumb

(Inherited verbatim from trunk plan §19.3 — unchanged for c-1.)

Per DQ #145 advisor lean + `feedback_build_what_tests_exercise.md`
(PMD #14) — never pre-implement detection for a producer that
doesn't yet emit. c-1 stubs the
`EscapeStatus::Escape{reason: "restoration_completed", ...}`
branch in two places:

1. The `EscapeStatus` enum at the top of
   `sponsor_liability_grace.rs` defines the `Escape{reason:
   String, actor_pseudonym: String, ref_id: i64}` variant generally
   — the `reason` field's type is `String`, not an enum, so any
   future reason string fits. The doc-comment on the enum
   enumerates `"sponsor_revoked"` (active in c-1) and
   `"restoration_completed"` (stub).
2. The `evaluate_escape_conditions` body has a comment block
   describing the future restoration-completed branch, with a
   `let _ = case_id;` line to suppress unused-var lints. The
   function returns `Fire` if no surety revocation matches.

When **restorative-mechanics-v1 PRD** is drafted and ships:

- That PRD's plan adds a query to `evaluate_escape_conditions`
  immediately after the surety-revocation check.
- That PRD's plan adds a corresponding e2e test in its own
  fixture mod.
- That PRD's plan flips the `(pending)` marker on
  `ENTRY_KIND_RESTORATION_COMPLETED` registry row.

The c-1 stub keeps the enum extensible (no breaking change when
restoration ships) while preventing dead-code drift.

### 19.4 Pre-existing pending DQ entries

- **None at planning time.** `decision-queue.json` shows 0 pending
  entries on governance-v0 @ `abcf778e1`. Recently resolved:
  DQ #144-#147 (SL-c clarify pass) + DQ #148 (proceed-as-one,
  superseded by DQ #150) + DQ #149 (baseline_sponsor_count) +
  DQ #150 (split override).

### 19.5 Out-of-scope follow-ups (Task 3 retro candidates)

- **5 e2e tests + registry marker flips** — c-2 deliverable.
- **`apply_sponsor_liability` compute/fire split** — SL-d.
- **`submit_jury_vote` mutation: `Decided →
  SponsorLiabilityPending` transition** — SL-d.
- **`restoration_complete` endpoint + restoration-escape branch
  detection** — restorative-mechanics-v1 PRD.
- **Lane-wide e2e suite** — SL-e.
- **Sponsor notification UX** — PRD §13 OQ-V1-SL-03.
- **Step-up auth for admin-driven scheduler runs** — v2.
- **Cross-instance federation of grace-window events** — v2 per
  ADR-014.
- **Admin dashboard surface for staleness alerts** — admin-dashboard-v1.

### 19.6 Confidence bands (c-1)

- **High (9/10):** module structure — exact paired mirror of
  `reputation_snapshot.rs::run_snapshot_batch` and
  `appeal_window_expiry.rs::run_appeal_window_expiry_batch`.
- **High (9/10):** scheduler tick wiring — third sibling of two
  shipped patterns.
- **High (9/10):** ConfigCache two-tier pattern per DQ #144.
- **High (9/10):** §5 complexity score 3 (well below threshold).
- **High (8/10):** ADR-015 pseudonym discipline in code (defensive
  test in c-2; code review at PR time + Story 1 structural
  Brief-Scope output `actor_pseudonym_helper::get_or_create`
  reference).
- **Moderate (7/10):** sanction multiplicity assumption (1:1)
  verified at plan-write time; defensive `ORDER BY id ASC LIMIT 1`
  guards.
- **High (8/10):** registry-marker deferral discipline
  (`feedback_build_what_tests_exercise.md`) — clean application;
  c-1 retro §3 carry-forward is the explicit hand-off.
- **High (8/10):** split mechanics — task list naturally divides at
  Task-2/3 boundary; no interleaved tasks.

### 19.7 Why no clarify DQ at impl time (c-1)

(Inherited from trunk plan §19.7 — unchanged for c-1.)

Brief §4.2 enumerates the boundary-of-judgment cases that should
trigger an impl-side DQ. None apply to this c-1 plan as written:

- v0 `apply_sponsor_liability` signature at
  `sponsor_liability.rs:142` is intact.
- Sanction multiplicity is 1:1.
- ENTRY_KIND consts declared in SL-a.
- Shim re-exports present.
- Canonical mirrors intact.
- DQ #144-#147 + #150 resolved.

If any of these baseline assumptions changes between plan-write and
impl-time, the impl-task subagent files a DQ pending entry.

### 19.8 Forward-only retrofit scope

Per `feedback_schema_changing_spec_retrofit_question.md` — c-1 does
NOT change the shape of any existing artifact class (no new template
section, no new schema marker, no new YAML field). All §13 task
contracts are routine impl-task contracts. No retrofit question
applies.

### 19.9 Registry-marker flip deferral (c-1 → c-2 hand-off)

Per `feedback_build_what_tests_exercise.md` (PMD #14): the registry
markers `_SPONSOR_LIABILITY_FIRED (pending)` and
`_SPONSOR_LIABILITY_ESCAPED (pending)` (SL-c portion) MUST stay
`(pending)` after c-1 ships and retro. The flip happens at c-2
retro AFTER the e2e tests behaviourally validate the fire-sites.
This is the c-1 → c-2 explicit hand-off — c-1 retro §3
carry-forward names the registry markers and the deferral.

---

## 20. Confidence score

- **Plan correctness:** 9/10 — patterns mirror
  `reputation_snapshot.rs::run_snapshot_batch` +
  `appeal_window_expiry.rs::run_appeal_window_expiry_batch` directly;
  §10 + §13 + §16a all match the SL-b/JM-e Shape-G shape; the
  module + scheduler block are mechanical with strong precedents.
- **Cargo budget:** 10/10 — Shape G (cargo runs off-box).
- **Test coverage:** N/A — c-1 ships zero new tests by design;
  behavioural validation is c-2's deliverable. Workspace-check
  workflow validates compile + clippy + test-no-run; that's the
  c-1 acceptance gate.
- **Story-grain decomposition:** 9/10 — c-1 ships exactly Story 1
  (module + scheduler wiring); Stories 2 + 3 explicitly deferred
  to c-2 with a clear hand-off via §6 + §17 + §19.9.
- **Plan-shape conformance:** 9/10 — 20-section schema followed
  literally; §15.6 Shape G shape adopted; §16a mandatory present;
  §13 per-task FILES YAML block on every task; §5.1 complexity
  breakdown table mechanical; c-1-specific slices in §5/§11/§13/§14/
  §15.4/§16/§16a clearly demarcated from inherited sections.

---

_Plan author: planning subagent (laptop sibling worktree
`/Users/barrie/Developer/lemmy-advisor-sl-c` on `governance-v0` @
`abcf778e1`, 2026-05-07 — c-1 / c-2 split per DQ #150 user override
at plan-approval gate). Plan committed locally on `governance-v0`
together with sibling `v1-sponsor-liability-c-2.plan.md`; advisor
publishes via standard sub-phase flow. BM-task cuts
`phase-v1-SL-c-1` (only after SL-b PR #119 merges). c-1 score 3 is
below threshold; no planner DQs filed. Confidence 9/10. Plan ships
under split-c-1-c-2 assumption (DQ #150)._
