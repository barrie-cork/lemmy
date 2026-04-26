# Plan: v1-JM-d — Appeals (bounded window + reporter-rights + auto re-jury + appeal-window-expiry job)

## Table of contents

1. Summary
2. Source
3. Problem statement
4. Solution statement (architectural decisions + watchpoints)
5. Metadata
6. Relationship to other v1-JM sub-phases
7. Preflight guardrails inherited from prior phases
8. Flow design
9. Mandatory reading
10. Patterns to mirror
11. Files to change
12. NOT building in v1-JM-d
13. Step-by-step tasks
14. Testing strategy
15. Validation commands (DoD)
16. Acceptance criteria
17. Completion checklist
18. Risks and mitigations
19. Notes
20. Sub-phase stub (v1-JM-e)

---

## 1. Summary

v1-JM-d is the Appeals sub-phase of the jury-mechanics PRD. It rewrites
`request_appeal.rs` for a **bounded** appeal window (replacing v0's
`closed_at`-based check with `appeal_window_expires_at`), extends caller
eligibility so the **original reporter** can appeal a `NoAction` /
`AdvisoryLabel` outcome (PRD §6.4 / OQ-V1-JM-06), and ships an
**auto-rejury** path that seats a larger appeal panel with the original
jurors excluded (PRD §6.1, §6.2, §6.3, §6.6). For the
`appeal.auto_select_on_appeal_acceptance = false` configuration, JM-d
adds a new admin-only handler `admin_trigger_appeal_rejury`. Finally, an
**hourly background job** (`run_appeal_window_expiry_batch`) sweeps
`Decided` cases whose `appeal_window_expires_at < now()`, flips them to
`Closed`, and emits `appeal_window_expired` per PRD §9.5.

JM-c's submit_jury_vote rewrite at PR #98 stabilised the
`appeal_window_expires_at` **writer**; JM-d is the first **reader**, the
first **mutator** at expiry, and the only writer of
`JuryAssignmentRole::Appeal` rows. Five ENTRY_KIND consts pre-landed by
JM-a (`appeal_panel_assembled`, `appeal_decided`, `appeal_rejected`,
`appeal_window_expired` — plus the v0 `appeal_requested`) all gain their
first live emitting call sites in this sub-phase. No new ENTRY_KIND
consts.

The bg-job pattern mirrors
`lemmy_api::governance::reputation_snapshot::run_snapshot_batch` (15-min
reputation tick wired in `crates/routes/src/utils/scheduled_tasks.rs:177-233`)
— **not** the "expired sanction cleanup" pattern PRD §9.5 names, which
does not exist in the codebase (DQ #51 raised by planner; see §19 / §4).

---

## 2. Source

- **Parent PRD**: [.claude/PRPs/prds/v1-jury-mechanics.prd.md](.claude/PRPs/prds/v1-jury-mechanics.prd.md) §17 row 4
- **PRD sections (specification)**: §6 (Appeals — first-class), §6.1 panel sizing, §6.2 original-jurors-excluded hard rule, §6.3 threshold-tier bump, §6.4 reporter-eligibility, §6.5 bounded window, §6.6 auto re-jury, §6.7 appeal status state machine, §9.3 `request_appeal` rewrite, §9.4 `admin_trigger_appeal_rejury`, §9.5 background job
- **PRD sections (defaults / cross-cutting)**: §10 default values matrix (knobs already seeded by JM-a — verified at `migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql:57-61`), §17.1 cross-PRD sequencing, §17.4 preflight guardrails (DQs #42/43/44/46 — all RESOLVED at command-template level)
- **Predecessor plan**: [.claude/PRPs/plans/v1-jury-mechanics-c.plan.md](.claude/PRPs/plans/v1-jury-mechanics-c.plan.md) §10.5 (appeal_window write), §10.8 (concurrency-test pattern + JM-c retro §3.2 amendment 1: lock-acquisition-order verification), §20 v1-JM-d stub
- **Predecessor retro**: [.claude/PRPs/reports/v1-JM-c-retro.md](.claude/PRPs/reports/v1-JM-c-retro.md) §3.3 handoff notes for JM-d (`appeal_window_expires_at` populated; `closed_at` no longer written by submit_jury_vote; deadlock branch lifecycle terminus)
- **Relevant ADRs**: ADR-008 (governance_log append-only), ADR-010 (no retroactive invalidation; appeal-window-days remains LIVE per PRD §9.1 cross-ref + PRD §17.4 carve-out for `appeal.window_days`), ADR-013 (CaseStatus exhaustive matches mandatory; preserved by JM-d), ADR-015 (GDPR pseudonymisation; appeal-panel pseudonyms via `actor_pseudonym_helper::get_or_create`)
- **Decision-queue inputs**:
  - DQ #50 (resolved, impl-self-resolved) — JM-c Test 6 deterministic Postgres deadlock under FK-SHARE-then-FOR-UPDATE; **applies directly to JM-d's bg-job design** (see §10.6 + §13 Task 5 + §18 Risk row 3).
  - DQ #48 (resolved, impl-self-resolved) — JM-a InsertForm drift precedent; informs JM-d's drift-fix task wording for `JuryAssignmentInsertForm.role` extension.
  - DQ #47 (planner-pending; non-blocking; OQ-V1-JM-07 v1.5 candidate) — does NOT block JM-d; non-emergency cases inherit JM-a DEFAULT 'Minor'.
- **DQ pre-seeds raised by this plan** (planner-attributed): DQ #51 (PRD §9.5 MIRROR-ref drift — `expired sanction cleanup` doesn't exist; recommendation: mirror `run_snapshot_batch` + `scheduled_tasks.rs:177-233`), DQ #52 (winning-decision storage choice — recommendation A: add `moderation_case.winning_decision JuryDecision NULL` + 1-line submit_jury_vote write, OR B: re-tally jury_vote rows in request_appeal). See §4 + §19.

---

## 3. Problem statement

After JM-c PR #98 merged, `appeal_window_expires_at` is populated on
every Decided case but is **read by no one**. Three concrete gaps remain
on the appeal lifecycle that the PRD §17 row 4 deliverable is
responsible for closing:

- `request_appeal.rs:107-114`
  (`crates/api/api_crud/src/governance/request_appeal.rs`) still gates
  appeals on `case.closed_at`. After JM-c, `closed_at` is NULL for
  every newly-Decided case until the (not-yet-existing)
  appeal-window-expiry job sets it. Result: the v0 appeal handler now
  rejects EVERY new appeal request as out-of-window, because
  `case.closed_at.map(|c| c > Utc::now()).unwrap_or(false)` returns
  `false` on a NULL `closed_at`. **This is a regression on
  `governance-v0` shipped by PR #98 that JM-d is the fix for.** The
  bounded-window check must switch to `appeal_window_expires_at`.

- `request_appeal.rs:117-120` only allows `case.target_person_id` to
  appeal. Per PRD §6.4 + OQ-V1-JM-06 lean (a), the **original reporter**
  has a legitimate grievance when `winning_decision IN (NoAction,
  AdvisoryLabel)` — but JM-d cannot make that decision without knowing
  what the winning decision was. PRD §9.3 recommends a new
  `moderation_case.winning_decision JuryDecision NULL` column written
  by submit_jury_vote at decision time.

- No background process reaps expired appeal windows. A `Decided` case
  whose appeal window has elapsed should transition to `Closed` so
  later admin queries / federation-state machinery can rely on the
  status invariant. PRD §9.5 specifies a periodic job mirroring an
  existing pattern; the named MIRROR ref does not exist (DQ #51).

Additionally, the PRD calls for an **auto-rejury** flow on appeal
acceptance (§6.6): when `appeal.auto_select_on_appeal_acceptance = true`,
the same transaction that creates the Appeal row also seats a larger
panel (1.5× rounded up, min original+2 — capped by `panel_size` bounds)
of jurors who are NOT among the original panel (PRD §6.2 — hard rule, no
override). For the false-config path, a new admin handler
`admin_trigger_appeal_rejury` does the same work post-hoc.

---

## 4. Solution statement

### 4.1 Architecturally load-bearing decisions locked in this sub-phase

1. **Bounded-window check uses `appeal_window_expires_at`** (PRD §6.5).
   `request_appeal.rs:112` swaps from
   `case.closed_at.map(|c| c > Utc::now()).unwrap_or(false)` to
   `case.appeal_window_expires_at.map(|c| c > Utc::now()).unwrap_or(false)`.
   The error variant on out-of-window stays `LemmyErrorType::NotFound`
   (Phase 5c v0 behaviour — see PRD §17.4 DQ #43 — flattens to HTTP 404
   regardless; impl session may swap to a more specific variant if the
   acceptance criteria require, but plan §16 does not gate on the code
   number).

2. **Reporter-rights eligibility reads `moderation_case.winning_decision`**
   (PRD §6.4 + §9.3 recommendation; planner DQ #52 lean A). JM-d adds:
   - New nullable column `moderation_case.winning_decision jury_decision`
     (existing enum from JM-a era; DEFAULT NULL) via Task 1 migration.
     `ModerationCase` + `ModerationCaseInsertForm` extended with
     `pub winning_decision: Option<JuryDecision>` (Task 2).
   - One-line write in `submit_jury_vote.rs` post-decision UPDATE:
     `moderation_case::winning_decision.eq(Some(winning_decision))`
     (Task 3). This is the only JM-d edit to submit_jury_vote.rs — at a
     different statement than the SL-d step-7 graft anchor
     (`apply_sponsor_liability` call), so the merge surface is disjoint
     per §17.1 "JM-d/e/SL-d each land on their own phase branch" +
     parallel-tracks framing.
   - Reporter eligibility check in `request_appeal.rs`:
     ```text
     if caller_id == case.target_person_id { OK }
     else if Some(caller_id) == case.creator_id
             && matches!(case.winning_decision, Some(JuryDecision::NoAction
                                              | JuryDecision::AdvisoryLabel)) { OK }
     else { Err(LemmyErrorType::NotFound) }
     ```

3. **`requester_role` on Appeal row** (PRD §6.4). New Postgres enum
   `appeal_requester_role` with two variants
   `Defendant | OriginalReporter`, mirrored as a `diesel-derive-enum`
   Rust enum `AppealRequesterRole` in
   `crates/db_schema_file/src/enums.rs`. `appeal.requester_role` column
   added NOT NULL DEFAULT 'Defendant' (covers the v0 backfill — every
   pre-JM-d Appeal row was filed by the target). `Appeal` +
   `AppealInsertForm` extended (Task 2).

4. **Auto-rejury path uses `select_eligible_jurors` with explicit
   exclusion** (PRD §6.2 + §6.6). New helper `select_appeal_panel(conn,
   &case, &mut cache) -> LemmyResult<AppealPanelSelection>` in
   `admin_assign_jury.rs` (alongside the existing
   `select_eligible_jurors` helper) that:
   - Loads original jurors via `jury_assignment::table.filter(case_id =
     ...).filter(role.eq(Original)).select(person_id)` (one query).
   - Computes
     `appeal_panel_size = max(ceil(panel_size_snapshot * appeal.panel_size_multiplier),
       panel_size_snapshot + appeal.panel_size_floor_increment)` per PRD
     §6.1, capped by `[3, 11]` (PRD §10 panel-size bounds).
   - Computes the appeal threshold-fraction by walking the cascade for
     the next-tier severity per PRD §6.3
     (`appeal.threshold_tier_bump` int 0/1/2): `tier_index(severity) +
     bump`, clamped to Severe; then
     `jury.threshold_fraction.<bumped_severity>` cascade lookup. The
     appeal panel's `threshold_count_snapshot` is stored alongside
     `panel_size_snapshot` for the appeal-panel rows (write target on
     the new `appeal` row columns added in Task 1's migration — NOT on
     `moderation_case` per ADR-010 immutability of the original
     verdict's parameters).
   - Calls
     `select_eligible_jurors(conn, &case, appeal_panel_size,
       Some(&original_juror_ids), &mut cache)`. Reuses the entire
     R1/R2/R3 cascade including `jury_constraint_violation_log` writes
     for the appeal pool — no second algorithm.

5. **Appeal-panel rows go to `jury_assignment` with role = Appeal**
   (PRD §8.2). `JuryAssignmentInsertForm` is extended in Task 2 with
   `pub role: Option<JuryAssignmentRole>` (drift-fix mirroring DQ #48
   pattern). DEFAULT-driven semantics: `None` ⇒ Postgres DEFAULT
   `'Original'` applies for the 6 v0/JM-b/JM-c writer sites; the new
   `select_appeal_panel` writer writes `Some(JuryAssignmentRole::Appeal)`.
   R3 sweep enumerates the 6 existing call sites that need
   `..Default::default()` (or explicit `role: None,`) added per
   `feedback_insertform_default_propagation.md`. See §10.5 + §13 Task 2.

6. **Background job lives in a new module mirroring
   `reputation_snapshot::run_snapshot_batch`** — not the PRD §9.5
   "expired sanction cleanup" pattern (which does not exist; planner DQ
   #51). New file
   `crates/api/api/src/governance/appeal_window_expiry.rs` exporting
   `pub async fn run_appeal_window_expiry_batch(context: &LemmyContext) -> LemmyResult<AppealWindowExpiryOutcome>`.
   Registered in `crates/routes/src/utils/scheduled_tasks.rs:setup` as
   a 1-hour tick (PRD §17 row 4 specifies hourly) using the same
   atomic-bool / `RunningGuard` / `BREHON_DISABLE_*_JOB` env override
   pattern as the reputation-snapshot tick. The job UPDATE uses `FOR
   UPDATE SKIP LOCKED` to avoid the FK-SHARE-then-EXCLUSIVE deadlock
   class JM-c retro §3.2 amendment 1 named (DQ #50). See §10.6.

7. **No new `ENTRY_KIND_*` consts.** All five appeal kinds were
   pre-landed by JM-a in the dual-file pattern (canonical const in
   `crates/db_schema/src/source/governance/governance_log.rs:181-184`
   + shim re-export in
   `crates/api/api/src/governance/governance_log.rs:42-46`). JM-d's
   contribution to
   `.claude/rules/governance-log-entry-kind-registry.md` is a single
   "v1-JM-d entry kinds" subsection that flips the `(pending)` markers
   on rows in the existing v1-JM-a table to live emitting-handler
   pointers (§10.7 + §13 Task 7). Per the registry's pre-landed-const
   exemption rule, those rows already name JM-d as the downstream
   sub-phase.

### 4.2 Watchpoints (specific files / tables / lines per `feedback_advisor_watchpoint_specificity.md`)

- **`crates/api/api_crud/src/governance/request_appeal.rs:107-115`** —
  the bounded-window check JM-d rewrites. The pre-JM-d behaviour reads
  `case.closed_at`; JM-c's `closed_at` removal at submit_jury_vote
  step 8 means **every newly-Decided post-JM-c case currently fails this
  check**. Watch: confirm Task 3 swaps to
  `case.appeal_window_expires_at` and the e2e tests assert both the
  within-window happy path and the past-expiry rejection.
- **`crates/api/api_crud/src/governance/request_appeal.rs:117-120`** —
  the caller-eligibility check
  (`case.target_person_id != Some(caller_id)`). Watch: Task 3 must
  extend to also accept `caller_id == case.creator_id` ∧
  `winning_decision IN (NoAction, AdvisoryLabel)` per PRD §6.4. The
  reporter-spoofing guard (§12.4) is satisfied by construction —
  `creator_id` is NULL for orphaned cases, so reporter-eligibility is
  impossible without a real reporter row.
- **`crates/api/api/src/governance/submit_jury_vote.rs`** —
  `winning_decision` write. JM-d adds **one** field assignment at the
  post-decision UPDATE (the no-sponsor `Decided`-flip update inside the
  JM-c rewrite). Search anchor:
  `moderation_case::status.eq(CaseStatus::Decided)` — append
  `.eq(Some(winning_decision))` clause for `winning_decision`. Verify
  the line lives BEFORE the SL-d step-7 graft anchor (the
  `TODO(v1-sponsor-liability-d)` comment) — disjoint surface.
- **`crates/db_schema/src/source/governance/moderation_case.rs:21-77`**
  (`ModerationCase`) and **lines 79-110**
  (`ModerationCaseInsertForm`) — both extended with
  `pub winning_decision: Option<JuryDecision>` in Task 2. Doc comment
  cites JM-d §9.3 + cross-references PRD §9.3 recommendation.
- **`crates/db_schema/src/source/governance/appeal.rs:17-25`**
  (`Appeal`) and **lines 27-35** (`AppealInsertForm`) — extended with
  `pub requester_role: AppealRequesterRole` (NOT NULL DEFAULT
  'Defendant') and `pub panel_size_snapshot: Option<i32>` +
  `pub threshold_count_snapshot: Option<i32>`. InsertForm fields are
  `Option<...>` so existing v0 callers compile (DEFAULT covers — there's
  only one caller today, in `request_appeal.rs:122-128`, which Task 3
  rewrites to set `Some(...)`).
- **`crates/db_schema/src/source/governance/jury_assignment.rs:44-56`**
  (`JuryAssignmentInsertForm`) — extended with
  `pub role: Option<JuryAssignmentRole>`. R3 sweep enumerates 6 caller
  sites: `crates/api/api/src/governance/admin_assign_jury.rs:234`,
  `crates/api/api/src/governance/admin_emergency_remove.rs:265`,
  `crates/api/api/src/governance/decline_jury_assignment.rs:177`,
  `crates/server/tests/e2e.rs:917`, `crates/server/tests/e2e.rs:3176`,
  `crates/server/tests/e2e.rs:3830`. Each gains `role: None,` (or
  `..Default::default()` per the form's `derive(Default)`). Task 2
  commit body lists the per-site touch.
- **`crates/db_schema_file/src/enums.rs`** — append a new
  `AppealRequesterRole` enum after the existing `JuryAssignmentRole`
  definition (line ~722). Mirror the `DbEnum` +
  `DbValueStyle = "verbatim"` shape used by JM-a's enums (lines
  704-722).
- **`crates/db_schema_file/src/schema.rs`** — regen via
  `diesel print-schema` after Task 1 migration runs.
  `appeal::requester_role -> AppealRequesterRole`,
  `appeal::panel_size_snapshot -> Nullable<Int4>`,
  `appeal::threshold_count_snapshot -> Nullable<Int4>`,
  `moderation_case::winning_decision -> Nullable<JuryDecision>`.
  Schema-regen drift between `crates/db_schema/src/source/...` structs
  and `schema.rs` will trip the `check_for_backend(diesel::pg::Pg)`
  derive — verify by `cargo check -p lemmy_db_schema --features full`
  after Task 2.
- **`crates/api/api/src/governance/admin_assign_jury.rs:456`**
  (`select_eligible_jurors` signature) — JM-d's new
  `select_appeal_panel` calls this with
  `exclude_person_ids = Some(&original_juror_ids)`. The
  `Option<&[PersonId]>` slot already exists; no signature change needed.
- **`crates/api/api/src/governance/admin_assign_jury.rs:232-262`** (the
  `JuryAssignmentInsertForm` writer + `jury_assigned` per-juror
  governance_log emission + extended `panel_assembled` log) —
  `select_appeal_panel` mirrors this shape but writes
  `role = Some(JuryAssignmentRole::Appeal)` and emits
  `appeal_panel_assembled` (not `panel_assembled`). Watch: pseudonym
  discipline (`actor_pseudonym_helper::get_or_create` per juror) must be
  preserved to satisfy ADR-015.
- **`crates/api/api/src/governance/governance_log.rs:42-46`** —
  pre-landed shim re-exports for `ENTRY_KIND_APPEAL_DECIDED`,
  `_APPEAL_PANEL_ASSEMBLED`, `_APPEAL_REJECTED`, `_APPEAL_REQUESTED`,
  `_APPEAL_WINDOW_EXPIRED`. JM-d activates a subset of these (no
  `ENTRY_KIND_APPEAL_DECIDED` writes in JM-d — appeal-panel vote tally
  is a JM-e capstone, NOT JM-d; verify call-site count per Task 7).
- **`crates/routes/src/utils/scheduled_tasks.rs:177-233`** — MIRROR ref
  for the appeal-window-expiry tick registration. Pattern:
  `let context_clone = context.reset_request_count(); scheduler.every(CTimeUnits::hour(1)).run(move || { ... });`.
  The atomic-bool guard + `BREHON_DISABLE_APPEAL_WINDOW_JOB` env var
  override mirrors lines 71-79 + 185-198.
- **`crates/api/api/src/governance/reputation_snapshot.rs:361-407`** —
  MIRROR ref for the new `run_appeal_window_expiry_batch` outcome-struct
  + `info!(target: "governance::...")` tracing pattern +
  `LemmyResult<...>` return.
- **`migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql`**
  — MIRROR for ADR-exception trail comment block + idempotent backfill
  pattern (`WHERE column IS NULL` guard so down/up cycles don't clobber
  existing writes). JM-d's Task 1 `up.sql` reuses this comment-block
  shape.
- **`.claude/rules/governance-log-entry-kind-registry.md` v1-JM-a
  section** — JM-d Task 7 appends a "v1-JM-d entry kinds (0 new)"
  subsection that flips `(pending)` markers in the existing v1-JM-a
  table for the kinds JM-d activates (`appeal_panel_assembled`,
  `appeal_window_expired` confirmed; verify `appeal_rejected` and
  `appeal_decided` at impl — leave `(pending)` if not emitted in JM-d).
- **`migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql:65`**
  — `jury_assignment.role NOT NULL DEFAULT 'Original'`. JM-d's
  `select_appeal_panel` writes explicit
  `Some(JuryAssignmentRole::Appeal)`; the DEFAULT covers all
  v0/JM-b/JM-c writer sites that omit `role` from the InsertForm
  literal (post-JM-d-extend) — confirm at Task 2 R3 sweep.
- **JM-c retro §3.2 amendment 1 (DQ #50 lock-acquisition order)** —
  `request_appeal`'s auto-rejury path INSERTs jury_assignment rows (FK
  SHARE on parent moderation_case) and does NOT need to FOR UPDATE the
  case afterward (no later step on the same row), so the
  FK-SHARE-then-EXCLUSIVE deadlock class JM-c hit is **not present**
  here. The appeal-window-expiry job, however, batches
  `UPDATE moderation_case SET status=Closed WHERE ...` and **must** use
  `FOR UPDATE SKIP LOCKED` (not bare FOR UPDATE) so a concurrent
  in-flight `request_appeal` on a row at the cusp of its window doesn't
  deadlock against the cron tick. Task 5 codifies this.
- **`scripts/brehon/cargo-check.sh` is +x; `cargo-clippy.sh` and
  `cargo-test.sh` are NOT +x in git** (`100644` mode in `git ls-tree
  HEAD scripts/brehon/`). Plan §15 invokes them via
  `bash scripts/brehon/...` (portable). Task 0 audit step verifies — if
  the impl prefers chmod, that's an out-of-band
  `chore(scripts): mark Linux wrappers executable` commit on a separate
  branch (NOT inside JM-d phase branch, per file-ownership boundaries —
  wrappers are infra).
- **Submodule `crates/email/translations` is uninitialized in this
  fresh worktree** (`git submodule status` returns `-a3f9...` prefix).
  `cargo check --workspace` fails on `lemmy_email` build script without
  `git submodule update --init --recursive` first. Task 0 audit step.
  Cite `feedback_worktree_submodules_not_auto_init.md`.
- **`rustup component list --installed`** on the EliteDesk Junior
  daemon does NOT include `clippy-x86_64-unknown-linux-gnu`. Task 0
  audit step verifies + STOPs if missing. Fix is
  `rustup component add clippy` outside the worktree (advisor / BM
  territory).

### 4.3 Rejected alternatives

- **B (re-tally jury_vote in `request_appeal`)** — duplicates the
  threshold-tally logic from JM-c submit_jury_vote.rs:319-348, doubles
  the maintenance surface, and forces the appeal handler to walk the
  per-decision tally on every reporter-eligibility check (8 enum
  variants × N rows). Not chosen. Filed as DQ #52 lean (B) for advisor
  visibility; recommendation is (A) — single column write at
  submit_jury_vote post-decision UPDATE.
- **Bundling all migration enums into one migration** — JM-a precedent
  (split `add_jury_mechanics_enums` from `add_jury_mechanics_columns`)
  makes ALTER-COLUMN-with-enum work in one migration after the enum
  exists. JM-d follows the same pattern: separate migration for the new
  enum (`appeal_requester_role`) before the migration that adds the
  column. Two migration directories.
- **Adding `role` to JuryAssignmentInsertForm with no Default override**
  — would require named `role: None,` at every existing call site (6),
  forcing a wider edit footprint and breaking the JM-a/JM-b precedent
  of `Default`-driven optional-field propagation. Chose
  `Option<JuryAssignmentRole>` + `derive(Default)` so existing call
  sites can adopt `..Default::default()` minimally; the explicit
  `role: None,` form is also acceptable per
  `feedback_insertform_default_propagation.md`.
- **Auto-rejury writes appeal-panel snapshot to
  `moderation_case.panel_size_snapshot`** — would clobber the
  original-panel snapshot. ADR-010's no-retroactive rule treats the
  case-level snapshot as immutable; the appeal panel's snapshot belongs
  on the `appeal` row (new columns `appeal.panel_size_snapshot` +
  `appeal.threshold_count_snapshot` added in Task 1's migration
  alongside `requester_role`). See §10.4.

---

## 5. Metadata

| Field | Value |
|---|---|
| Type | SCHEMA + HANDLER + BACKGROUND_JOB (mixed; PRD §17 row 4 description) |
| Complexity | MED — clear file boundaries; no new algorithms (reuses `select_eligible_jurors`); 1 new bg-job pattern; 1 new admin handler; 1 minimal cross-file edit (submit_jury_vote `winning_decision` write) |
| Crates Affected | `lemmy_db_schema_file`, `lemmy_db_schema`, `lemmy_api_common`, `lemmy_api`, `lemmy_api_crud`, `lemmy_routes`, `lemmy_server` (e2e tests + scheduler tick) |
| v0 Step | n/a (v1 sub-phase, builds on the v0 11-endpoint MVP — does not add a new MVP endpoint; extends `POST /api/v4/governance/appeal` and adds `POST /api/v4/governance/admin/trigger-appeal-rejury` per PRD §17 row 4) |
| Dependencies | v1-JM-c PR #98 merged on `governance-v0` (verified at session start; `git log --oneline governance-v0` shows the JM-c task commits as ancestors) |
| Estimated Tasks | 8 (Task 0 audit + Tasks 1-7 deliverable) |

---

## 6. Relationship to other v1-JM sub-phases

- **JM-a (schema + enums + backfill)** — lands `severity_tier`,
  `status_tier`, `panel_size_snapshot`, `quorum_snapshot`,
  `threshold_count_snapshot`, `appeal_window_expires_at` columns.
  Pre-lands the 5 `ENTRY_KIND_APPEAL_*` consts. JM-d activates a subset
  of those consts and adds two NEW columns
  (`moderation_case.winning_decision`, `appeal.requester_role` — plus
  `appeal.panel_size_snapshot` and `appeal.threshold_count_snapshot`).
- **JM-b (admin_assign_jury cascade + diversity constraints)** — lands
  `select_eligible_jurors` with the R1/R2/R3 cascade. JM-d's
  `select_appeal_panel` is a thin wrapper that calls
  `select_eligible_jurors` with `exclude_person_ids = original_jurors`.
  No new constraint algorithm; full reuse of the cascade including
  `jury_constraint_violation_log` writes.
- **JM-c (submit_jury_vote 9-step)** — writes
  `appeal_window_expires_at` on every Decided case (the value JM-d
  reads). JM-c's removal of `closed_at` writes is **the regression
  JM-d must fix** in `request_appeal.rs:107-115`. JM-d adds **one
  line** to JM-c's submit_jury_vote at the post-decision UPDATE: the
  `winning_decision: Some(winning_decision)` field assignment. Disjoint
  from SL-d's step-7 graft anchor.
- **JM-e (capstone test + cross-sub-phase integration)** — extends
  JM-c's `v0_case_completes_under_v0_rules_after_v1_config_flip` with
  appeal-side cross-sub-phase assertions; depends on JM-d's helpers and
  column additions.

Concurrent sub-phases (per PRD §17.1):

- **SL-d** can run in parallel on its own phase branch — different file
  region (step 7 graft anchor vs JM-d's post-decision UPDATE
  `winning_decision` write).
- **rep-tuning-r3** can run in parallel — emitter point at step 7,
  again different region from JM-d's write.

---

## 7. Preflight guardrails inherited from prior phases

Per PRD §17.4: DQs #42 / #43 / #44 / #46 are RESOLVED at command-template
level (2026-04-23). JM-d does NOT re-codify the checks; the task wording
relies on `/prp-core:prp-implement` Phase 1.4 (task-resume safety) and
§4.1.1 (HTTP status code audit) and §4.2.0 (Docker daemon preflight)
firing automatically. Cite the DQ ids only:

- DQ #42 — task-resume safety (Phase 1.4 LOAD detect-already-completed)
- DQ #43 — HTTP status code audit (3+ non-400 codes in JM-d:
  `LemmyErrorType::NotFound` for out-of-window appeal,
  out-of-eligibility caller, missing case; preserved verbatim from v0 —
  no new HTTP-status-code changes)
- DQ #44 — Docker daemon preflight (`docker ps` in
  `pre-phase-harness-audit.md` Probe 0 + `/prp-core:prp-implement
  §4.2.0` per e2e invocation)
- DQ #46 — carry-forward issue policy (post-merge follow-up GH issues
  filed in §3.1 of JM-d's retro)

Per JM-c retro §3.2 amendments now codified into this plan:

- §3.2 amendment 1 (lock-acquisition order pre-validation) — Task 5
  (background job) explicitly uses `FOR UPDATE SKIP LOCKED`; Task 6
  (e2e) includes a concurrency-safety probe.
- §3.2 amendment 2 (snapshot-value cross-references) — Task 6 e2e
  tests reference `admin_assign_jury_severity_tier_*` tests for the
  snapshot values they assert on (panel sizes, thresholds), not
  restated inline.
- §3.2 amendment 3 (variable-removal downstream sweep) — Task 3
  rewrite of `request_appeal.rs` removes the `case.closed_at`
  reference; the `closed_at` column is preserved (used by the bg job at
  Task 5 and read by `admin_close_case`), so this amendment is
  satisfied by not removing the column itself. Task 3 does NOT delete
  unused variables — it replaces a single `if` predicate.
- §3.2 amendment 4 (mandatory-read InsertForm sweep) — Task 2 R3 sweep
  enumerates 6 `JuryAssignmentInsertForm` sites + ALSO enumerates
  `ModerationCaseInsertForm` (3 prod + N test sites) and
  `AppealInsertForm` (1 prod site) for parallel coverage of the new
  optional-field additions per
  `feedback_insertform_default_propagation.md`. See §13 Task 2.
- §3.2 amendment 5 (skill-trigger hint parity) — §13 task wording uses
  the principle-style triggers per `feedback_principles_not_rules.md`
  (Task 2 names `/edit-mechanical` for the multi-site struct-field
  propagation, Task 6 names `/test-write` for the e2e-test scaffolding,
  Tasks 0/3/4/5/6/7 name `/cargo-validate` for the gating cargo runs).
  Skill mentions are advisory; impl session may run inline at its
  judgment.
- §3.2 amendment 6 (`git grep` symbol anchors over absolute line
  numbers) — §10 MIRROR refs use both file:line AND a grep symbol
  anchor where the line drift is likely to bite (e.g., the
  `apply_sponsor_liability` call site in submit_jury_vote.rs).

---

## 8. Flow design

### 8.1 Before state (post-JM-c PR #98 merge)

```
+-----------------------------------------------------------------------------+
|              BEFORE — appeal lifecycle on governance-v0                     |
+-----------------------------------------------------------------------------+
|                                                                             |
|   submit_jury_vote -> step 9 writes case.appeal_window_expires_at = now+7d  |
|                       case.closed_at = NULL                                 |
|                                                                             |
|   POST /api/v4/governance/appeal                                            |
|     |                                                                       |
|     v                                                                       |
|   request_appeal                                                            |
|     | - within_window = case.closed_at.map(|c| c > now()).unwrap_or(false)  |
|     | - closed_at IS NULL -> within_window = false                          |
|     | - returns LemmyErrorType::NotFound  <-- REGRESSION (always rejects)   |
|     v                                                                       |
|   no Decided -> Closed transition; cases pile up indefinitely               |
|   no auto-rejury; no admin-trigger handler                                  |
|   only target_person_id can appeal (no reporter-rights)                     |
|                                                                             |
|   PAIN:                                                                     |
|     - 100% of post-JM-c appeal requests return 404                          |
|     - Decided cases never transition to Closed -> status invariants drift   |
|     - Reporters can't appeal NoAction outcomes (PRD §6.4 unmet)             |
|                                                                             |
+-----------------------------------------------------------------------------+
```

### 8.2 After state (v1-JM-d)

```
+-----------------------------------------------------------------------------+
|                AFTER — bounded, reporter-aware, auto-rejuring               |
+-----------------------------------------------------------------------------+
|                                                                             |
|   submit_jury_vote -> also writes case.winning_decision (NEW)               |
|                       still writes appeal_window_expires_at                 |
|                                                                             |
|   POST /api/v4/governance/appeal                                            |
|     |                                                                       |
|     v                                                                       |
|   request_appeal                                                            |
|     | - within_window = case.appeal_window_expires_at > now()  <-- NEW     |
|     | - eligibility:                                                       |
|     |     defendant always | reporter iff winning_decision IN              |
|     |     {NoAction, AdvisoryLabel}                                        |
|     | - INSERT appeal { requester_role: <Defendant|OriginalReporter> }     |
|     | - case.status: Decided -> Appealed                                   |
|     | - if appeal.auto_select_on_appeal_acceptance: select_appeal_panel    |
|     |     - exclude original jurors via role = Original lookup             |
|     |     - panel_size = max(ceil(orig*multiplier), orig+floor_increment)  |
|     |     - threshold cascade with tier-bump                               |
|     |     - INSERT N rows in jury_assignment with role = Appeal            |
|     |     - emit appeal_panel_assembled                                    |
|     | - else: case sits in Appealed waiting for admin_trigger              |
|     v                                                                       |
|   POST /api/v4/governance/admin/trigger-appeal-rejury  (NEW)                |
|     | - admin-only                                                         |
|     | - status must be Appealed AND no role=Appeal rows exist yet          |
|     | - calls same select_appeal_panel helper                              |
|     | - emits appeal_panel_assembled (admin pseudonym as actor)            |
|     v                                                                       |
|   scheduler tick (1 hr): run_appeal_window_expiry_batch  (NEW)              |
|     | - SELECT id FROM moderation_case                                     |
|     |   WHERE status = Decided                                             |
|     |   AND appeal_window_expires_at < now()                               |
|     |   FOR UPDATE SKIP LOCKED                                             |
|     | - UPDATE -> status = Closed, closed_at = now()                       |
|     | - emit appeal_window_expired per row                                 |
|                                                                             |
|   VALUE_ADD:                                                                |
|     - regression fixed: appeals work again                                  |
|     - reporter-rights honoured for NoAction / AdvisoryLabel outcomes        |
|     - auto-rejury: defendant gets a larger panel, no original jurors        |
|     - status invariants restored: Decided => eventually Closed              |
|                                                                             |
+-----------------------------------------------------------------------------+
```

### 8.3 Endpoint changes

| Endpoint | Before | After |
|---|---|---|
| `POST /api/v4/governance/appeal` | Always 404 (post-JM-c regression) | Bounded by `appeal_window_expires_at`; defendant + (reporter on NoAction/AdvisoryLabel); auto-rejury when configured |
| `POST /api/v4/governance/admin/trigger-appeal-rejury` | (does not exist) | NEW: admin-only; for `appeal.auto_select_on_appeal_acceptance = false` config |
| Background job (no HTTP surface) | (does not exist) | NEW: hourly `run_appeal_window_expiry_batch` flips Decided -> Closed past window |

`crates/api/routes/src/lib.rs:535-553` (the `/admin` scope) gains one
new route (`POST /trigger-appeal-rejury`). No DTO changes to
`RequestAppeal` or `RequestAppealResponse` (the request body is
unchanged; v1 reporter-rights is a server-side eligibility extension,
not a DTO surface change).

---

## 9. Mandatory reading

### 9.1 Brehon design docs (P0)

| File | Lines | Why |
|---|---|---|
| `.claude/PRPs/prds/v1-jury-mechanics.prd.md` | §6 + §9.3-9.5 + §10 + §17 row 4 + §17.1 + §17.4 | Authoritative requirements for JM-d |
| `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` | ADR-008, ADR-010, ADR-013, ADR-015 | Hard constraints (governance log append-only; no retroactive invalidation; CaseStatus exhaustive matches; pseudonymisation) |
| `docs/brehon-law-inspired-network/01-vision-and-principles.md` | §5.7 (appeal "larger jury") | Justifies appeal-panel sizing semantics |
| `docs/brehon-law-inspired-network/04-data-model-and-api.md` | §17 row "appeal" | Data-model contract for the appeal table |

### 9.2 Codebase reads (P0 — mirror these patterns)

| File | Lines / Symbol | Why |
|---|---|---|
| `crates/api/api_crud/src/governance/request_appeal.rs` | full file (155 lines) | The handler being rewritten. The `process_appeal` shape (pseudonym -> tx -> 7-step body) is preserved; only the `within_window` predicate (line 112) and the eligibility check (line 118) and the new auto-rejury block change. |
| `crates/api/api/src/governance/admin_assign_jury.rs` | `process_assignment` (lines 101-290), `select_eligible_jurors` (456+), `compute_status_tier`, `ceil_count` | MIRROR for the auto-rejury seating + the `select_appeal_panel` helper. Preserve the per-juror pseudonym + `jury_assigned`-style emission shape; replace `panel_assembled` with `appeal_panel_assembled` for the appeal path. |
| `crates/api/api/src/governance/submit_jury_vote.rs` | post-decision UPDATE (search anchor: `moderation_case::status.eq(CaseStatus::Decided)`) | The single line JM-d edits to add `winning_decision: Some(winning_decision)` to the SET clause. |
| `crates/api/api/src/governance/reputation_snapshot.rs` | `run_snapshot_batch` (lines 361-407), `check_snapshot_staleness` (lines 423+) | MIRROR for `run_appeal_window_expiry_batch` outcome-struct + `info!` tracing pattern. |
| `crates/routes/src/utils/scheduled_tasks.rs` | tick registration (lines 81-233 — full setup function) | MIRROR for the hourly tick wiring + atomic-bool guard + `BREHON_DISABLE_*_JOB` env override. |
| `crates/db_schema/src/source/governance/moderation_case.rs` | full file | Pattern for adding `winning_decision: Option<JuryDecision>` to ModerationCase + InsertForm. |
| `crates/db_schema/src/source/governance/appeal.rs` | full file | Pattern for adding `requester_role: AppealRequesterRole` + appeal-panel snapshot columns to Appeal + InsertForm. |
| `crates/db_schema/src/source/governance/jury_assignment.rs` | full file | Pattern for adding `role: Option<JuryAssignmentRole>` to JuryAssignmentInsertForm (mirrors JM-b drift-fix DQ #48 for `selected_under_constraints`). |
| `crates/db_schema_file/src/enums.rs` | `JuryAssignmentRole` (lines 704-722), `JuryDecision` (lines 488-509), `AppealStatus` (lines 559-576), `JuryConstraintRelaxationReason` (lines 724-767) | Pattern for the new `AppealRequesterRole` enum. |
| `migrations/2026-04-23-000000-0000_add_jury_mechanics_enums/up.sql` | full file | MIRROR for new-enum migration (CREATE TYPE). |
| `migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql` | full file (120 lines) | MIRROR for ALTER TABLE + idempotent backfill + ADR-exception trail comment block. |
| `migrations/2026-04-23-000050-0000_add_jury_constraint_relaxation_reason_enum/up.sql` | full file | MIRROR for split-enum-then-column-add migration pair. |
| `crates/server/tests/e2e.rs` | `v1_jm_b_fixtures` (lines 6944-7178), golden-path test (line 1061+), severity-tier panel tests (line 7184+) | Test fixture mod the JM-d e2e tests reuse. `seed_jury_eligible_snapshots` is mandatory before every `admin_assign_jury` (R2 from JM-b retro §3.2 amendment 2). |
| `crates/api/api/src/governance/admin_close_case.rs` | full file | Closest existing admin handler for `admin_trigger_appeal_rejury` shape (auth check + status guard + governance_log emission). |

### 9.3 Test patterns (P1 — fixture sources)

| File | Lines / Symbol | Why |
|---|---|---|
| `crates/server/tests/e2e.rs` | `v1_jm_b_fixtures::bootstrap` (line 6978+) | Bootstrap returns `(_container, context, db_url)` — JM-d e2e tests reuse verbatim. |
| `crates/server/tests/e2e.rs` | `seed_user`, `seed_community`, `seed_jurors`, `seed_case`, `seed_jury_eligible_snapshots` | Already-public fixture functions. JM-d adds NO new fixtures unless an appeal-specific seeding helper is genuinely needed (e.g., `seed_appealed_case` that pre-walks through admin_assign_jury -> submit_jury_vote -> request_appeal). |
| `crates/server/tests/e2e.rs` | `LocalUserView::read_person(&mut context.pool(), juror_id).await?` pattern (line 1228, 1346, 1378, etc — ~10 callsites) | The canonical lookup pattern. JM-d does NOT add a `lookup_local_user_view` helper (per JM-c retro §2.5 — verify no existing API already covers this). |

### 9.4 Rules (P0 — auto-loaded but worth re-reading at session start)

- `.claude/rules/cargo-output-capture.md` — capture-then-tail discipline; mandatory for every cargo invocation
- `.claude/rules/no-cargo-output-paste.md` — keep cargo log tails out of conversation context
- `.claude/rules/decision-queue.md` — DQ attribution rules (impl-task may NOT write `answered_by: "advisor"`; mid-task DQ writes from this Junior worktree must commit + push immediately)
- `.claude/rules/governance-log-entry-kind-registry.md` — Acceptance invariants count check at end of phase (33 + flips of (pending) markers in the v1-JM-a row, no new const additions)
- `.claude/rules/branch-manager.md` — file ownership boundaries (impl session NEVER touches `.claude/PRPs/plans/**`, BM NEVER touches `crates/**`)
- `.claude/rules/phase-branch.md` — every commit on `phase-v1-JM-d`; PR base `governance-v0`
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory on every `gh pr` command
- `.claude/rules/pre-phase-harness-audit.md` — Probes 0-4 + DoD smoke + clippy baseline
- `.claude/rules/pm-plugin-hooks-stable.md` — no PM-adjacent code touched (JM-d is governance-handler + scheduled-task only)

### 9.5 External documentation

- `chrono::Duration` (used for `appeal.window_days * INTERVAL '1 day'` arithmetic) — `i64` arg per docs.rs/chrono — already verified in JM-c §10.5; reuse the same `Duration::days(window_days_i64)` shape.
- `clokwerk::AsyncScheduler::every(CTimeUnits::hour(1))` — JM-d's tick uses `CTimeUnits::hour(1)` per PRD §17 row 4 (hourly). Already imported via `lemmy_routes::utils::scheduled_tasks` use-block at line 4 (`clokwerk::{AsyncScheduler, TimeUnits as CTimeUnits}`).
- `diesel::dsl::FOR UPDATE SKIP LOCKED` — Diesel exposes this via `for_update().skip_locked()` on a query builder. Pattern used in `process_ranks_in_batches` at `scheduled_tasks.rs:298` (raw `sql_query` with `FOR UPDATE SKIP LOCKED` clause). Reuse via the typed query-builder in JM-d's batch.

---

## 10. Patterns to mirror

### 10.1 Migration: split enum-create + ALTER-with-column pair

**SOURCE:** `migrations/2026-04-23-000050-0000_add_jury_constraint_relaxation_reason_enum/up.sql` + `migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql`.

**JM-d Task 1 — TWO migrations, in chronological order:**

```sql
-- migrations/<ts>-000000-0000_add_appeal_requester_role_enum/up.sql
-- v1-JM-d Task 1 part A — new Postgres enum for appeal requester-role
-- discriminator. Mirrors the case_status_tier / severity_tier /
-- jury_assignment_role pattern from JM-a 2026-04-23-000000.
CREATE TYPE appeal_requester_role AS ENUM (
    'Defendant',
    'OriginalReporter'
);
```

```sql
-- migrations/<ts>-000100-0000_add_appeals_v1_columns/up.sql
-- v1-JM-d Task 1 part B — ALTER appeal + moderation_case for v1 appeals.
-- ============================================================
-- ADR exception trail (protected governance tables: moderation_case, appeal)
-- ============================================================
-- Additive ALTER (ADD COLUMN only). ADR-008 / ADR-015 satisfied: no DROP,
-- no UPDATE-in-place; new columns carry typed enum / decision-enum (no
-- free-text PII surface).
--
-- Controlling ADR: ADR-010 (staged releases) authorises the schema
-- extension; reversibility via companion down.sql holds.
-- ============================================================

-- Appeal table — requester_role discriminator + appeal-panel snapshot
-- columns (PRD §6.1, §6.4, §6.6 — appeal-panel rows live in
-- jury_assignment with role=Appeal but their snapshot parameters land
-- here so moderation_case.panel_size_snapshot stays immutable per
-- ADR-010).
ALTER TABLE appeal ADD COLUMN requester_role appeal_requester_role
    NOT NULL DEFAULT 'Defendant';
ALTER TABLE appeal ADD COLUMN panel_size_snapshot INTEGER;
ALTER TABLE appeal ADD COLUMN threshold_count_snapshot INTEGER;

-- moderation_case — winning_decision recorded at submit_jury_vote
-- decision time so request_appeal's reporter-rights check (PRD §6.4)
-- doesn't re-tally jury_vote rows.
ALTER TABLE moderation_case ADD COLUMN winning_decision jury_decision;

-- No backfill: pre-JM-d Decided cases have no recorded winning_decision
-- and reporter-rights eligibility is naturally false (Some(_)
-- match-arm); existing pre-JM-d Appeal rows pre-date the bounded-window
-- regression and were filed by defendants -> DEFAULT 'Defendant' is
-- correct.
```

**`down.sql` reversal**: drop columns then drop enum (reverse order).
Idempotent shape required (`DROP COLUMN IF EXISTS`); `DROP TYPE IF
EXISTS appeal_requester_role` after the column drop releases the
dependency.

**GOTCHA:** the timestamp prefixes must come AFTER `2026-04-23-000200-0000`
(JM-a's last migration). Use `date -u +'%Y-%m-%d-%H%M%S-0000'` at impl
time and verify the prefix is unique against `ls migrations/`. Do NOT
hard-code `2026-04-26` — the impl session may run on a different day.

### 10.2 ModerationCase + InsertForm extension

**SOURCE:** `crates/db_schema/src/source/governance/moderation_case.rs:99-110` (the JM-a v1 additions block).

**JM-d additions to the read model (line 76+ inside `pub struct ModerationCase`):**

```rust
  /// v1-JM-d §9.3: snapshot of the winning JuryDecision recorded by
  /// submit_jury_vote at decision time. Used by request_appeal's
  /// reporter-rights check (PRD §6.4) to determine whether the
  /// original reporter has a legitimate grievance (only on NoAction or
  /// AdvisoryLabel outcomes). NULL until JM-d is in place; pre-v1
  /// Decided cases stay NULL and reporter-rights is naturally false.
  pub winning_decision: Option<JuryDecision>,
```

**JM-d additions to the InsertForm (line 109+ inside `pub struct ModerationCaseInsertForm`):**

```rust
  /// v1-JM-d §9.3 InsertForm extension. Default `None`; submit_jury_vote
  /// writes via update().set(...) rather than re-inserting, so the
  /// InsertForm path is only exercised by case-open writers
  /// (create_report, admin_emergency_remove) which don't yet know the
  /// decision.
  pub winning_decision: Option<JuryDecision>,
```

Mirror the JM-a additions block's doc-comment style verbatim.

### 10.3 Appeal + InsertForm + AppealRequesterRole enum

**SOURCE:** `crates/db_schema_file/src/enums.rs:704-722` (`JuryAssignmentRole` — closest precedent in shape).

**JM-d additions to `crates/db_schema_file/src/enums.rs` (after `JuryAssignmentRole`, line ~722):**

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::AppealRequesterRole"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// v1-JM-d §6.4 / §9.3: discriminates the requester of an appeal so
/// the eligibility check (defendant always; reporter only on NoAction
/// / AdvisoryLabel) can be reproduced from a stored row without
/// re-walking case ownership. Backfill DEFAULT 'Defendant' covers
/// pre-v1 rows (pre-JM-d, only the target_person_id could appeal).
pub enum AppealRequesterRole {
  #[default]
  Defendant,
  OriginalReporter,
}
```

**JM-d additions to `Appeal` struct (line 25+ in appeal.rs):**

```rust
  /// v1-JM-d §6.4: who filed the appeal. Defendant always; original
  /// reporter only when winning_decision was NoAction / AdvisoryLabel.
  pub requester_role: AppealRequesterRole,
  /// v1-JM-d §6.1: snapshot of the appeal panel's size at seating
  /// time. NULL until select_appeal_panel runs (auto on accept, or
  /// admin-triggered).
  pub panel_size_snapshot: Option<i32>,
  /// v1-JM-d §6.3: threshold-count for the appeal panel (cascade
  /// resolution of jury.threshold_fraction at the bumped severity per
  /// appeal.threshold_tier_bump). NULL until select_appeal_panel runs.
  pub threshold_count_snapshot: Option<i32>,
```

**JM-d additions to `AppealInsertForm` (line 35+ in appeal.rs):**

```rust
  /// v1-JM-d §6.4: the requester_role discriminator. `Option<_>`
  /// because the DB DEFAULT 'Defendant' covers v0 callers if any
  /// remain post-rewrite — but request_appeal Task 3 always sets
  /// `Some(...)` based on the eligibility branch taken.
  pub requester_role: Option<AppealRequesterRole>,
  pub panel_size_snapshot: Option<i32>,
  pub threshold_count_snapshot: Option<i32>,
```

### 10.4 Appeal-panel snapshot location (per ADR-010)

The appeal panel's `panel_size_snapshot` + `threshold_count_snapshot`
land on the **`appeal` row**, NOT on `moderation_case`. Rationale:

- ADR-010 says the case-level snapshot freezes at jury-seating time.
  The original panel's parameters are immutable for the life of the
  case. If `moderation_case.panel_size_snapshot` were rewritten by
  `select_appeal_panel`, the v0-compat regression test
  (`v0_case_completes_under_v0_rules_after_v1_config_flip`) would
  fail — its assertion is that the snapshot stays at v0 defaults
  (`Some(5), Some(3), Some(3)`) even mid-flight.
- The `appeal` table is the natural home for appeal-specific snapshot
  parameters; future `appeal_decided` JM-e capstone will read these to
  compute the appeal-tally threshold check.

**Verification:** the
`v0_case_completes_under_v0_rules_after_v1_config_flip` test (JM-c
e2e) will continue to pass after JM-d because JM-d does NOT mutate
`moderation_case.panel_size_snapshot / quorum_snapshot /
threshold_count_snapshot` on the appeal-rejury path.

### 10.5 JuryAssignmentInsertForm role-field extension (mirrors DQ #48)

**SOURCE:** `crates/db_schema/src/source/governance/jury_assignment.rs:44-56` (existing JM-a/JM-b drift-fix block for `selected_under_constraints`).

**JM-d addition (append after `selected_under_constraints` field in `JuryAssignmentInsertForm`):**

```rust
  /// v1-JM-d §6.6 + §8.2: appeal-panel writer (`select_appeal_panel`)
  /// sets `Some(JuryAssignmentRole::Appeal)`; v0/v1-JM-b/JM-c writers
  /// (admin_assign_jury, admin_emergency_remove,
  /// decline_jury_assignment replacement, the e2e fixture sites at
  /// e2e.rs:917 / :3176 / :3830) keep `None` and rely on the DB
  /// DEFAULT 'Original'. Optional per the same `derive(Default)`
  /// pattern used for selected_under_constraints (DQ #48).
  pub role: Option<JuryAssignmentRole>,
```

**R3 sweep (Task 2 pre-impl checkpoint):**

```bash
# Enumerate every JuryAssignmentInsertForm caller site:
rg -n 'JuryAssignmentInsertForm \{' crates/ > /tmp/jm-d-insertform-sites.log
cat /tmp/jm-d-insertform-sites.log
# Expected: 6 sites
#   crates/api/api/src/governance/admin_assign_jury.rs:234
#   crates/api/api/src/governance/admin_emergency_remove.rs:265
#   crates/api/api/src/governance/decline_jury_assignment.rs:177
#   crates/server/tests/e2e.rs:917
#   crates/server/tests/e2e.rs:3176
#   crates/server/tests/e2e.rs:3830
```

For each enumerated site, the impl session adds **either** `role: None,`
**or** `..Default::default()` to the struct literal. Pick one approach
and apply uniformly per `feedback_insertform_default_propagation.md`
option (b). Recommended: `..Default::default()` because the form derives
`Default` and this future-proofs against further field additions (JM-c
retro §3.2 amendment 4 generalisation).

### 10.6 Appeal-window-expiry batch job (mirror reputation_snapshot tick)

**SOURCE:** `crates/routes/src/utils/scheduled_tasks.rs:71-79` (atomic-bool guard) + `:177-233` (15-min reputation tick) + `crates/api/api/src/governance/reputation_snapshot.rs:361-407` (batch fn shape).

**JM-d Task 5 — new module `crates/api/api/src/governance/appeal_window_expiry.rs`:**

```rust
//! Appeal-window expiry background job per PRD §9.5.
//!
//! Runs hourly: finds Decided cases whose appeal_window_expires_at is
//! in the past and flips them to Closed, emitting `appeal_window_expired`
//! per row. Uses FOR UPDATE SKIP LOCKED to coexist with concurrent
//! request_appeal handlers without deadlocks (per JM-c retro §3.2
//! amendment 1 / DQ #50 lock-acquisition order).

use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, dsl::update};
use diesel_async::RunQueryDsl;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::governance::moderation_case::ModerationCase;
use lemmy_db_schema_file::{enums::CaseStatus, schema::moderation_case};
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::LemmyResult;
use serde_json::json;
use tracing::info;

use crate::governance::{governance_log, governance_log::ENTRY_KIND_APPEAL_WINDOW_EXPIRED};

#[derive(Debug, Default)]
pub struct AppealWindowExpiryOutcome {
  pub cases_processed: usize,
}

/// Hourly tick — sweeps Decided cases past their
/// appeal_window_expires_at, flips each to Closed, emits
/// appeal_window_expired. SKIP LOCKED keeps concurrent request_appeal
/// handlers safe (request_appeal acquires its row lock implicitly via
/// the case load + update sequence; SKIP LOCKED here means a row
/// currently being mutated by request_appeal is left for the next
/// tick — preferable to a deadlock).
pub async fn run_appeal_window_expiry_batch(
  context: &LemmyContext,
) -> LemmyResult<AppealWindowExpiryOutcome> {
  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  let mut outcome = AppealWindowExpiryOutcome::default();
  let now = Utc::now();

  // Single-statement select with FOR UPDATE SKIP LOCKED so concurrent
  // mutations on the same row defer to the next tick. Per the JM-c
  // retro §3.2 amendment 1: lock-acquisition order matters, and a
  // batch UPDATE is the only writer here, so taking the lock as the
  // first action on each row is safe.
  let candidates: Vec<ModerationCase> = moderation_case::table
    .filter(moderation_case::status.eq(CaseStatus::Decided))
    .filter(moderation_case::appeal_window_expires_at.lt(now))
    .for_update()
    .skip_locked()
    .load(conn)
    .await?;

  for case in &candidates {
    update(moderation_case::table.filter(moderation_case::id.eq(case.id)))
      .set((
        moderation_case::status.eq(CaseStatus::Closed),
        moderation_case::closed_at.eq(Some(now)),
      ))
      .execute(conn)
      .await?;

    governance_log::append(
      &mut (&mut *conn).into(),
      ENTRY_KIND_APPEAL_WINDOW_EXPIRED,
      json!({
        "case_id": case.id.0,
        "decided_at": case.decided_at,
        "window_expired_at": case.appeal_window_expires_at,
      }),
      // No actor pseudonym — system-issued by the scheduler.
      None,
    )
    .await?;

    outcome.cases_processed += 1;
  }

  if !candidates.is_empty() {
    info!(
      "governance: appeal-window expiry tick — cases_processed={}",
      outcome.cases_processed
    );
  }
  Ok(outcome)
}
```

**Scheduler tick registration in `crates/routes/src/utils/scheduled_tasks.rs:setup`:**

```rust
// After the reputation_snapshot tick (line 233):

// Brehon governance: appeal-window expiry. Hourly tick — finds
// Decided cases whose appeal_window_expires_at is past and flips
// them to Closed.
//
// Concurrency guard mirrors REPUTATION_SNAPSHOT_RUNNING. Disable
// for e2e tests via BREHON_DISABLE_APPEAL_WINDOW_JOB=1 so test
// fixtures don't race the cron.
let context_appeal_expiry = context.reset_request_count();
scheduler.every(CTimeUnits::hour(1)).run(move || {
  let context = context_appeal_expiry.reset_request_count();
  async move {
    if std::env::var("BREHON_DISABLE_APPEAL_WINDOW_JOB").as_deref() == Ok("1") {
      return;
    }
    if APPEAL_WINDOW_EXPIRY_RUNNING
      .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
      .is_err()
    {
      warn!("appeal_window_expiry: previous batch still running, skipping this tick");
      return;
    }
    let _guard = AppealWindowExpiryRunningGuard;
    lemmy_api::governance::appeal_window_expiry::run_appeal_window_expiry_batch(&context)
      .await
      .inspect_err(|e| warn!("Failed to run appeal_window_expiry batch: {e}"))
      .ok();
  }
});
```

**Module-scope additions (lines 71-79 area):**

```rust
static APPEAL_WINDOW_EXPIRY_RUNNING: AtomicBool = AtomicBool::new(false);

struct AppealWindowExpiryRunningGuard;

impl Drop for AppealWindowExpiryRunningGuard {
  fn drop(&mut self) {
    APPEAL_WINDOW_EXPIRY_RUNNING.store(false, Ordering::Release);
  }
}
```

**GOTCHA:** the `crates/server/src/governance.rs` stub at lines 22-28
logs which Brehon governance jobs are wired. Append a one-line `info!`
for the appeal-window-expiry tick alongside the existing snapshot
line — keeps the declarative-stub-as-doc property the file's doc-comment
promises.

### 10.7 Governance-log registry update

**SOURCE:** `.claude/rules/governance-log-entry-kind-registry.md`
v1-JM-a section.

**JM-d Task 7 (registry update before merge):**

In the existing v1-JM-a table (lines for `_APPEAL_PANEL_ASSEMBLED`,
`_APPEAL_REJECTED`, `_APPEAL_WINDOW_EXPIRED` — and `_APPEAL_DECIDED`
**only if** JM-d's appeal-vote-tally path emits it; verify at impl
time — JM-d may NOT emit `_APPEAL_DECIDED` if the appeal-vote-tally is
JM-e territory), change the "Emitting handler" column from `(pending)`
markers to live file paths:

| const | Old | New |
|---|---|---|
| `ENTRY_KIND_APPEAL_PANEL_ASSEMBLED` | `(pending)` | `crates/api/api_crud/src/governance/request_appeal.rs::process_appeal` (auto-rejury branch) + `crates/api/api/src/governance/admin_trigger_appeal_rejury.rs` |
| `ENTRY_KIND_APPEAL_REJECTED` | `(pending)` | (verify call site at impl — likely still pending if JM-d does not emit it; leave `(pending)` for a future sub-phase) |
| `ENTRY_KIND_APPEAL_WINDOW_EXPIRED` | `(pending)` | `crates/api/api/src/governance/appeal_window_expiry.rs::run_appeal_window_expiry_batch` |
| `ENTRY_KIND_APPEAL_DECIDED` | `(pending)` | (verify; likely still `(pending)` — appeal-vote-tally is JM-e) |

Append a "v1-JM-d entry kinds (0 new)" subsection below the v1-JM-c
subsection, naming the kinds JM-d activates (or fewer if the
verify-at-impl dropdowns above pare it back). Acceptance invariants
already match: 33 v1-JM-c-end + 0 new = 33 (no count change).

**GOTCHA:** the `(pending)` cell text format must mirror the existing
shape verbatim. Run the registry's count check at the end of Task 7:

```bash
rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
# Expected: 33 (unchanged)
rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs \
  | awk -F: '/ENTRY_KIND_/ {print}' \
  | grep -oE '"[a-z_]+"' | sort | uniq -d
# Expected: empty
```

---

## 11. Files to change

| File | Action | Justification |
|---|---|---|
| `migrations/<ts>-000000-0000_add_appeal_requester_role_enum/up.sql` + `down.sql` | CREATE | New Postgres enum per §10.1 |
| `migrations/<ts>-000100-0000_add_appeals_v1_columns/up.sql` + `down.sql` | CREATE | ALTER appeal + moderation_case per §10.1 |
| `crates/db_schema_file/src/enums.rs` | UPDATE | Add `AppealRequesterRole` enum after `JuryAssignmentRole` per §10.3 |
| `crates/db_schema_file/src/schema.rs` | UPDATE | Regen via `diesel print-schema` after migrations apply |
| `crates/db_schema/src/source/governance/moderation_case.rs` | UPDATE | Add `winning_decision: Option<JuryDecision>` to `ModerationCase` + `ModerationCaseInsertForm` per §10.2 |
| `crates/db_schema/src/source/governance/appeal.rs` | UPDATE | Add `requester_role`, `panel_size_snapshot`, `threshold_count_snapshot` to `Appeal` + InsertForm per §10.3 |
| `crates/db_schema/src/source/governance/jury_assignment.rs` | UPDATE | Add `role: Option<JuryAssignmentRole>` to `JuryAssignmentInsertForm` per §10.5 |
| `crates/api/api/src/governance/admin_assign_jury.rs` | UPDATE | Add `role: None,` (or `..Default::default()`) at line 234 InsertForm literal; add new `pub(crate) async fn select_appeal_panel(...)` helper alongside `select_eligible_jurors` |
| `crates/api/api/src/governance/admin_emergency_remove.rs` | UPDATE | Add `role: None,` at line 265 InsertForm literal (R3 sweep amendment) |
| `crates/api/api/src/governance/decline_jury_assignment.rs` | UPDATE | Add `role: None,` at line 177 InsertForm literal (R3 sweep amendment) |
| `crates/api/api/src/governance/submit_jury_vote.rs` | UPDATE | Add `moderation_case::winning_decision.eq(Some(winning_decision))` to the post-decision UPDATE (one line) per §4.1 decision 2 |
| `crates/api/api_crud/src/governance/request_appeal.rs` | UPDATE | Bounded-window check; reporter-rights eligibility; auto-rejury seeding (auto-config branch); `requester_role` write on AppealInsertForm; `appeal_panel_assembled` emission |
| `crates/api/api/src/governance/admin_trigger_appeal_rejury.rs` | CREATE | New handler per PRD §9.4 |
| `crates/api/api/src/governance/mod.rs` | UPDATE | `pub mod admin_trigger_appeal_rejury;` (alphabetical insertion at line ~22) |
| `crates/api/api/src/governance/appeal_window_expiry.rs` | CREATE | New module hosting `run_appeal_window_expiry_batch` per §10.6 |
| `crates/api/api/src/governance/governance_log.rs` | (no change) | All five APPEAL_* re-exports already present at lines 42-46 (JM-a pre-landed) |
| `crates/api/api_common/src/governance.rs` | UPDATE | Add `AdminTriggerAppealRejury { case_id }` request DTO + `AdminTriggerAppealRejuryResponse { case_id, appeal_id, panel_person_ids }` response DTO; preserve existing `RequestAppeal` / `RequestAppealResponse` shapes |
| `crates/api/routes/src/lib.rs` | UPDATE | Add `admin_trigger_appeal_rejury::admin_trigger_appeal_rejury` import + `.route("/trigger-appeal-rejury", post().to(admin_trigger_appeal_rejury))` under the `/admin` scope at line ~537 |
| `crates/routes/src/utils/scheduled_tasks.rs` | UPDATE | Add `APPEAL_WINDOW_EXPIRY_RUNNING` static + `AppealWindowExpiryRunningGuard` + new hourly tick block per §10.6 |
| `crates/server/src/governance.rs` | UPDATE | Append one `info!` line documenting the new appeal-window-expiry tick (preserves the file's "single place for governance-job logging" doc-comment property) |
| `crates/server/tests/e2e.rs` | UPDATE | Add 7 new e2e tests under a new `mod v1_jm_d_fixtures { ... }` block (mirrors `v1_jm_b_fixtures`); update R3 sweep sites at lines 917 / 3176 / 3830 with `role: None,` (or `..Default::default()`) |
| `.claude/rules/governance-log-entry-kind-registry.md` | UPDATE | Flip `(pending)` markers on the APPEAL_* rows JM-d activates per §10.7 |
| `.claude/PRPs/reports/v1-JM-d-retro.md` | CREATE | End-of-phase retro per `feedback_retro_not_report.md` |

**Files explicitly NOT touched:**

- `crates/api/api/src/governance/admin_assign_jury.rs` step-1-through-10 of `process_assignment` (the original-jury seating path is JM-b territory; JM-d only adds the `select_appeal_panel` helper)
- `crates/api/api/src/governance/sponsor_liability.rs` — SL-d territory
- `crates/api/api/src/governance/reputation_snapshot.rs` — rep-tuning-r* territory
- The four unrelated v1 PRDs' reserved registry sections — only JM-d's section flips
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` — no new dependencies (clokwerk + diesel-async + diesel + chrono all already in the workspace)
- `.coderabbit.yaml` — review config unchanged (governance-table additive ALTER stays in scope per JM-a precedent)
- `crates/db_schema/src/source/governance/jury_assignment.rs` `JuryAssignment` read-model — already carries the `role` field per JM-a; only the InsertForm changes

---

## 12. NOT building in v1-JM-d

- **Appeal-vote tally / `appeal_decided` emission** — the appeal panel
  votes are submitted via the existing `submit_jury_vote` handler. The
  threshold-tier-bumped tally + `_APPEAL_DECIDED` emission belongs to
  v1-JM-e (capstone). JM-d seats the appeal panel; JM-e closes the loop.
  Confirm at impl: if a JM-d test fails because appeal-panel votes aren't
  tallied yet, that's expected — assert on the panel seating, not on
  the appeal verdict.
- **`admin_reject_appeal` handler** — pre-landed `_APPEAL_REJECTED`
  const stays `(pending)`. Out of JM-d scope; v1-JM-e or v1.5.
- **Step-up auth for severity-tier mid-case changes (§12.3)** — JM-e.
- **Reporter-rights extension to `Warning` / `Cooldown` outcomes** —
  PRD §6.4 limits reporter eligibility to `NoAction | AdvisoryLabel`.
  Future PRD revision (v1.5) may broaden; JM-d implements the PRD
  literally.
- **Cross-instance jury federation** — v2 (PRD §17 row 5).
- **Pre-existing `closed_at` semantics on legacy paths** —
  `admin_close_case` and the JM-a backfill wrote `closed_at`; JM-d does
  not change those writers. The bg job adds `closed_at` writes for the
  natural-expiry path; legacy `closed_at` reads (e.g.
  `report_to_modlog_golden_path` asserts) continue to work because the
  bg job sets `closed_at = now()` on transition.
- **OQ-V1-JM-07 / DQ #47 case-open severity inference for non-emergency
  paths** — v1.5 candidate, planner-pending. JM-d does NOT add any
  case-open severity_tier writers.
- **Concurrent appeal cap (PRD §7.2 `jury.max_concurrent_assignments_per_juror_total`
  cross-community)** — already implemented by `select_eligible_jurors`
  per JM-b; JM-d's `select_appeal_panel` inherits it transitively. No
  new cap logic.

---

## 13. Step-by-step tasks

Execute in order. One commit per task. Each task has MIRROR refs, exact
file paths, and validation commands. **All cargo invocations capture to
`.claude/PRPs/debug/v1-JM-d-task<N>-<probe>.log` per
`.claude/rules/cargo-output-capture.md`. Wrappers invoked as
`bash scripts/brehon/cargo-<verb>.sh` (portable — works regardless of
+x bit per §4.2).** Skill-trigger hints follow
`feedback_principles_not_rules.md`: mention the skill when its
discipline is the gating concern; impl session is free to inline at its
judgment.

### Task 0: Pre-flight harness audit + branch verification + JM-c state confirmation

**Goal:** verify environment is ready for JM-d impl on the Junior
worktree; confirm branch is `phase-v1-JM-d`; confirm JM-c's
`appeal_window_expires_at` writer is in place (`winning_decision` writer
will be added by JM-d Task 3).

**Probes** (per `.claude/rules/pre-phase-harness-audit.md`):

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe -1 — submodule init (Linux/Junior worktree quirk)
git submodule status > /tmp/jm-d-task0-submodule.log 2>&1
if grep -q '^-' /tmp/jm-d-task0-submodule.log; then
  git submodule update --init --recursive > /tmp/jm-d-task0-submodule-init.log 2>&1
  echo "submodule init exit: $?"
fi

# Probe -2 — clippy installed
rustup component list --installed > /tmp/jm-d-task0-rustup.log 2>&1
grep -q '^clippy' /tmp/jm-d-task0-rustup.log || {
  echo "CLIPPY NOT INSTALLED — run rustup component add clippy outside this worktree (advisor/BM territory)"; exit 1;
}

# Probe 1 — wrapper -p crate honour
bash scripts/brehon/cargo-check.sh -p lemmy_db_schema > /tmp/jm-d-task0-check-p.log 2>&1
echo "exit: $?"; tail -5 /tmp/jm-d-task0-check-p.log
# Expected: exit 0; "Checking lemmy_db_schema"; no other crates in tail

# Probe 2 — workspace + features full
bash scripts/brehon/cargo-check.sh --workspace --features full > /tmp/jm-d-task0-check-workspace.log 2>&1
echo "exit: $?"; tail -10 /tmp/jm-d-task0-check-workspace.log
# Expected: exit 0; clean baseline

# Probe 3 — cargo-test --no-run
bash scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server > /tmp/jm-d-task0-test-no-run.log 2>&1
echo "exit: $?"; tail -10 /tmp/jm-d-task0-test-no-run.log
# Expected: exit 0; e2e test target builds

# Probe 4 — wrappers fail loud on bogus features
bash scripts/brehon/cargo-check.sh -p lemmy_server --features nonexistent_xyz > /tmp/jm-d-task0-check-negative.log 2>&1
echo "negative-probe exit: $?"; tail -3 /tmp/jm-d-task0-check-negative.log
# Expected: non-zero exit; "does not contain this feature: nonexistent_xyz"

# Probe 5 — DoD smoke for §15 commands (each command run literally)
bash scripts/brehon/cargo-check.sh --workspace --features full > /tmp/jm-d-task0-dod-check.log 2>&1
echo "DoD check exit: $?"; tail -5 /tmp/jm-d-task0-dod-check.log
# Expected: exit 0

bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > /tmp/jm-d-task0-dod-clippy.log 2>&1
echo "DoD clippy exit: $?"; tail -10 /tmp/jm-d-task0-dod-clippy.log
# Expected: exit 0 (clean baseline)

# Probe 6 — branch verify
git branch --show-current
# Expected: phase-v1-JM-d  (NOT governance-v0)

# Probe 7 — JM-c state confirmation
git log --oneline governance-v0..HEAD | head -5
# Expected: empty or just the JM-d-cut commit

# Probe 8 — appeal_window_expires_at populated check (read-only)
grep -n 'appeal_window_expires_at' crates/api/api/src/governance/submit_jury_vote.rs | head
# Expected: at least one match showing JM-c's appeal-window write

# Probe 9 — DQ pull
cat .claude/decision-queue.json | python3 -c "import json,sys; d=json.load(sys.stdin); print('pending:', [e['id'] for e in d.get('pending',[])])"
# Expected: pending may include #47 (planner; non-blocking) and any JM-d planner pre-seeds (#51, #52)
```

**Goal:** every probe exit-0 (or, for Probe 4, non-zero by design). If
any probe fails, STOP and surface to advisor; do not start Task 1.

**No commit at Task 0** — audit logs are local
(`.claude/PRPs/debug/` gitignored; the `/tmp/` paths used above stay
out of git entirely).

**SKILL HINT:** `/cargo-validate` for the gating cargo runs at probes
1/2/3/5 if the impl prefers the skill's capture-then-tail wrapper.
Inline form (`> log 2>&1; echo "exit: $?"; tail -N`) is equivalent.

### Task 1: CREATE migrations — `appeal_requester_role` enum + `appeals_v1_columns`

**Goal:** schema-first delivery. Two migration directories, in
chronological order; both apply via
`cargo run -p lemmy_diesel_utils --features full` (per
`feedback_lemmy_migration_runner.md` — raw `diesel migration run` is
forbid-triggered on this workspace).

**ACTION:**
1. `mkdir -p migrations/<ts>-000000-0000_add_appeal_requester_role_enum/`
   (timestamp resolved at impl-time via
   `date -u +'%Y-%m-%d-%H%M%S-0000'`, ensuring it's >
   `2026-04-23-000200-0000`).
2. Write `up.sql` per §10.1; write `down.sql` (drop type if exists).
3. `mkdir -p migrations/<ts>-000100-0000_add_appeals_v1_columns/`.
4. Write `up.sql` per §10.1; write `down.sql` (drop columns then drop
   type).
5. Apply both via the migration runner (round-trip down/up to confirm
   `down.sql` reverses cleanly).

**MIRROR:** `migrations/2026-04-23-000050-0000_add_jury_constraint_relaxation_reason_enum/up.sql` + `migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql`.

**GOTCHA:** the timestamp prefix MUST come AFTER `2026-04-23-000200-0000`.
Sort `ls migrations/` to confirm position. Future-dating the prefix to
beyond JM-c's last is safe; back-dating is NOT.

**GOTCHA:** the `appeal` table's
`requester_role NOT NULL DEFAULT 'Defendant'` backfills via the
metadata-only attmissingval path (Postgres 11+); no table rewrite, fast
on any size of `appeal` table. The `moderation_case.winning_decision`
column is NULLABLE — no backfill at migration time.

**GOTCHA:** `down.sql` must `DROP COLUMN IF EXISTS` then `DROP TYPE IF
EXISTS` (reverse-order); if down runs after a partial up, `IF EXISTS`
keeps it idempotent. Round-trip via the runner:
`Options::default().run()` then `.revert().limit(2)` then `.run()`
again. The `phase1_migrations_round_trip` e2e test will exercise this
on the first e2e run after Task 1.

**VALIDATE (per `cargo-output-capture.md`):**

```bash
# Apply migrations
cargo run -p lemmy_diesel_utils --features full > /tmp/jm-d-task1-migrate.log 2>&1
echo "exit: $?"; tail -10 /tmp/jm-d-task1-migrate.log
# Expected: exit 0; both new migration dirs applied

# Round-trip
cargo run -p lemmy_diesel_utils --features full -- revert --limit=2 > /tmp/jm-d-task1-revert.log 2>&1
echo "revert exit: $?"
cargo run -p lemmy_diesel_utils --features full > /tmp/jm-d-task1-reapply.log 2>&1
echo "reapply exit: $?"

# Schema sanity (read-only)
PGPASSWORD=password psql -h localhost -U lemmy -d lemmy -c '\d appeal' > /tmp/jm-d-task1-psql-appeal.log 2>&1
PGPASSWORD=password psql -h localhost -U lemmy -d lemmy -c '\d moderation_case' > /tmp/jm-d-task1-psql-moderation_case.log 2>&1
# Expected: requester_role, panel_size_snapshot, threshold_count_snapshot
# present on appeal; winning_decision present on moderation_case
```

**COMMIT MESSAGE:** `feat(v1-JM-d): add appeals v1 schema columns + AppealRequesterRole enum (task 1)`

### Task 2: UPDATE Diesel models + `AppealRequesterRole` enum + InsertForm extensions

**Goal:** schema models match the new schema; R3 sweep enumerates and
fixes 6+ existing call sites that need optional-field defaults.

**ACTION:**
1. `crates/db_schema_file/src/enums.rs` — add `AppealRequesterRole`
   enum after line 722 per §10.3.
2. `cargo run -p lemmy_diesel_utils --features full -- print-schema > crates/db_schema_file/src/schema.rs.new`
   then merge into `schema.rs` (or run the project's preferred regen
   command). Confirm
   `appeal::requester_role -> AppealRequesterRole`,
   `appeal::panel_size_snapshot -> Nullable<Int4>`,
   `appeal::threshold_count_snapshot -> Nullable<Int4>`,
   `moderation_case::winning_decision -> Nullable<JuryDecision>`.
3. `crates/db_schema/src/source/governance/moderation_case.rs` — add
   `winning_decision: Option<JuryDecision>` to `ModerationCase` +
   `ModerationCaseInsertForm` per §10.2.
4. `crates/db_schema/src/source/governance/appeal.rs` — add three new
   fields to `Appeal` + `AppealInsertForm` per §10.3.
5. `crates/db_schema/src/source/governance/jury_assignment.rs` — add
   `role: Option<JuryAssignmentRole>` to `JuryAssignmentInsertForm` per
   §10.5.
6. R3 sweep: enumerate `JuryAssignmentInsertForm {` literals via
   `rg -n 'JuryAssignmentInsertForm \{' crates/`. For each (6 expected
   sites; see §10.5), apply minimal edit:
   - In `crates/api/api/src/governance/admin_assign_jury.rs:234`,
     `crates/api/api/src/governance/admin_emergency_remove.rs:265`,
     `crates/api/api/src/governance/decline_jury_assignment.rs:177`,
     and `crates/server/tests/e2e.rs:917 / :3176 / :3830`: append
     `..Default::default()` to the struct literal (mirrors §10.5
     recommended approach + `feedback_insertform_default_propagation.md`
     option (b)).
7. R3 sweep for ModerationCaseInsertForm + AppealInsertForm: enumerate
   via `rg -n 'ModerationCaseInsertForm \{' crates/` and
   `rg -n 'AppealInsertForm \{' crates/`. For
   ModerationCaseInsertForm sites without `..Default::default()`, add
   it; for the AppealInsertForm site at `request_appeal.rs:122-128`
   (will be rewritten in Task 3), no edit needed — Task 3 sets
   `requester_role` explicitly.

**MIRROR:** `crates/db_schema/src/source/governance/jury_assignment.rs:48-55`
(the JM-a-drift-fix doc-comment block) for the `role` field doc-comment
shape.

**GOTCHA:** if `cargo check -p lemmy_db_schema --features full` fails
after the schema regen with a `check_for_backend(diesel::pg::Pg)`
error, the regen mismatched the `Queryable` derive's expected types —
re-run the regen command and confirm the
`winning_decision -> Nullable<JuryDecision>` mapping is exact.

**GOTCHA:** `..Default::default()` on a struct that derives Default
auto-fills any future field additions — but the struct must have ALL
its fields covered by Default. `JuryAssignmentInsertForm` derives
Default (line 41 of jury_assignment.rs) and all fields are
`Option<_>` / default-impl primitives, so this is safe.

**SKILL HINT:** `/edit-mechanical` is the right shape for the R3 sweep
(repeat-pattern struct-field propagation across N call sites — exactly
the discipline the skill enforces with rg-enumerate-first). The impl
session may inline if more comfortable.

**VALIDATE:**

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > /tmp/jm-d-task2-check.log 2>&1
echo "exit: $?"; tail -10 /tmp/jm-d-task2-check.log
# Expected: exit 0

bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > /tmp/jm-d-task2-clippy.log 2>&1
echo "exit: $?"; tail -10 /tmp/jm-d-task2-clippy.log
# Expected: exit 0

bash scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server > /tmp/jm-d-task2-test-no-run.log 2>&1
echo "exit: $?"
# Expected: exit 0 (struct-extension test target compile per R7)
```

**COMMIT MESSAGE:** `feat(v1-JM-d): extend Diesel models for appeals v1 + R3 sweep on InsertForm callers (task 2)`

### Task 3: REWRITE `request_appeal.rs` (bounded window + reporter-rights + auto-rejury) + add `winning_decision` write to `submit_jury_vote.rs`

**Goal:** fix the post-JM-c regression; add reporter-rights eligibility;
seat the appeal panel when configured.

**ACTION:**

**Part A — `crates/api/api/src/governance/submit_jury_vote.rs`:**

1. Find the post-decision UPDATE block (search anchor:
   `moderation_case::status.eq(CaseStatus::Decided)`).
2. Append `moderation_case::winning_decision.eq(Some(winning_decision))`
   to the SET tuple. Preserve the surrounding two-UPDATE shape (JM-c
   §10.5 status/decided_at separate from appeal_window_expires_at). The
   `winning_decision` write goes with the status/decided_at update,
   NOT the appeal-window update.

**Part B — `crates/api/api_crud/src/governance/request_appeal.rs`:**

1. Replace the `within_window` line (line 112) with
   `case.appeal_window_expires_at.map(|c| c > Utc::now()).unwrap_or(false)`.
2. Add the reporter-eligibility branch:
   - if `case.target_person_id == Some(caller_id)` ->
     `requester_role = AppealRequesterRole::Defendant`
   - else if `case.creator_id == Some(caller_id) && matches!(case.winning_decision, Some(JuryDecision::NoAction | JuryDecision::AdvisoryLabel))` ->
     `requester_role = AppealRequesterRole::OriginalReporter`
   - else -> `Err(LemmyErrorType::NotFound.into())`
3. Set `AppealInsertForm.requester_role: Some(requester_role)` at the
   insert call.
4. After the case-status flip to `Appealed`, branch on
   `appeal.auto_select_on_appeal_acceptance` config:
   - true -> call new `select_appeal_panel` helper (introduced in this
     task's Part C); insert N rows in `jury_assignment` with
     `role = Some(JuryAssignmentRole::Appeal)` and
     `selected_under_constraints =
     Some(constraint_record.to_json())`; emit one
     `appeal_panel_assembled` governance_log entry (NOT one per juror —
     the audit pattern from `admin_assign_jury.rs:264-284` is one
     extended emission per panel, not one per row); write the
     appeal-panel snapshot (`panel_size_snapshot`,
     `threshold_count_snapshot`) onto the new Appeal row via UPDATE.
   - false -> no panel seating; case sits in Appealed waiting for
     `admin_trigger_appeal_rejury`.

**Part C — new helper in `crates/api/api/src/governance/admin_assign_jury.rs`:**

Add a new `pub(crate) async fn select_appeal_panel(...)` alongside
`select_eligible_jurors` (lines 456+ area):

```rust
/// v1-JM-d §6.6 + §6.1: assemble the appeal panel for a Decided case.
/// Excludes original jurors (PRD §6.2 hard rule); panel size derived
/// per PRD §6.1 multiplier+floor + bounded by [3,11]; threshold
/// resolved at the bumped severity per PRD §6.3.
///
/// Returns the panel ids + the constraint-record from the underlying
/// select_eligible_jurors call so the caller can write
/// selected_under_constraints to each appeal-panel row.
pub(crate) async fn select_appeal_panel(
  conn: &mut diesel_async::AsyncPgConnection,
  case: &ModerationCase,
  cache: &mut ConfigCache,
) -> LemmyResult<AppealPanelSelection> {
  // 1. Load original jurors (role=Original).
  let original_juror_ids: Vec<PersonId> = jury_assignment::table
    .filter(jury_assignment::case_id.eq(case.id))
    .filter(jury_assignment::role.eq(JuryAssignmentRole::Original))
    .select(jury_assignment::person_id)
    .load(conn)
    .await?;

  // 2. Compute appeal panel size (PRD §6.1).
  let original_panel_size = case.panel_size_snapshot.ok_or_else(|| /* ... */)?;
  let multiplier = config::get_float(cache, ..., "appeal.panel_size_multiplier").await?;
  let floor_increment = config::get_int(cache, ..., "appeal.panel_size_floor_increment").await?;
  let appeal_panel_size = compute_appeal_panel_size(original_panel_size, multiplier, floor_increment); // saturating, clamped [3,11]

  // 3. Threshold cascade at the bumped severity (PRD §6.3).
  let bump = config::get_int(cache, ..., "appeal.threshold_tier_bump").await?;
  let bumped_severity = bump_severity(case.severity_tier, bump); // clamps to Severe
  let threshold_fraction = config::get_float_cascade(
    cache, ..., "jury.threshold_fraction", &[severity_tier_slug(bumped_severity)]
  ).await?;
  let appeal_threshold_count = ceil_count(f64::from(appeal_panel_size) * threshold_fraction, "appeal_threshold_count")?;

  // 4. Reuse select_eligible_jurors with exclusion list.
  let (eligible, record) = select_eligible_jurors(
    conn, case, i64::from(appeal_panel_size), Some(&original_juror_ids), cache
  ).await?;

  Ok(AppealPanelSelection {
    person_ids: eligible,
    panel_size_snapshot: appeal_panel_size,
    threshold_count_snapshot: appeal_threshold_count,
    constraint_record: record,
  })
}

pub(crate) struct AppealPanelSelection {
  pub person_ids: Vec<PersonId>,
  pub panel_size_snapshot: i32,
  pub threshold_count_snapshot: i32,
  pub constraint_record: ConstraintRecord,
}
```

**MIRROR:** `select_eligible_jurors` signature + return-tuple shape;
`compute_status_tier` helper-fn placement; `ceil_count` for f64-to-i32
narrowing; `severity_tier_slug` for cascade-key construction.

**GOTCHA:** `case.panel_size_snapshot.ok_or_else(...)` — pre-v1 cases
have `Some(5)` from the JM-a backfill; cases that never reached
JurySelection (Open / ThresholdMet / EmergencyRemove) have NULL. The
status-guard at `request_appeal.rs:93-105` rejects all of those except
Decided — so a Decided case always has a populated
`panel_size_snapshot` by construction; `ok_or_else` returns
`LemmyErrorType::Unknown` as the defensive guard.

**GOTCHA:** the auto-rejury branch must run inside the same
`run_transaction` block as the appeal-row insert. If panel seating
fails (e.g. small pool), the appeal row inserts ARE rolled back per
the existing `process_appeal` transaction wrapping (line 65). This is
desired — failed panel selection means the appeal cannot proceed; both
operations succeed atomically or both fail.

**GOTCHA:** the LIVE `appeal.window_days` config read happens at
submit_jury_vote step 9 (per JM-c §10.5); JM-d does NOT re-read it.
The `appeal_window_expires_at` value in the case row is the source of
truth for the bounded-window check.

**GOTCHA (DQ #50 / JM-c retro §3.2 amendment 1 — lock-acquisition
order):** request_appeal does NOT take FOR UPDATE on the case row
before the appeal-row insert. The flow is:
- read case (line 86)
- INSERT appeal (FK SHARE on parent moderation_case)
- UPDATE moderation_case status (no FOR UPDATE — the case is loaded
  inside the `run_transaction`, the UPDATE acquires the row lock when
  executed)
This sequence is **not** the JM-c FK-SHARE-then-EXCLUSIVE class because
the appeal-row INSERT and the case UPDATE are sequential in one tx; no
concurrent second tx is racing for an EXCLUSIVE upgrade on the same
row. Concurrent appeal requests on the same case would deadlock under
`tokio::join!` — but that's a pathological client pattern (one user
filing two appeals simultaneously) and the existing v0 idempotency
guard at the appeal-status check would reject the second request once
the first commits. No new test for concurrent appeals; the existing
case-already-Appealed reject path is the protection.

**SKILL HINT:** `/cargo-validate` for the per-task gate;
`/edit-mechanical` is NOT applicable here (judgment-heavy edits in two
files).

**VALIDATE:**

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > /tmp/jm-d-task3-check.log 2>&1
echo "exit: $?"; tail -10 /tmp/jm-d-task3-check.log

bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > /tmp/jm-d-task3-clippy.log 2>&1
echo "exit: $?"; tail -10 /tmp/jm-d-task3-clippy.log

bash scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server > /tmp/jm-d-task3-test-no-run.log 2>&1
echo "exit: $?"
# Expected: exit 0
```

**COMMIT MESSAGE:** `feat(v1-JM-d): bounded-window appeal + reporter-rights + auto-rejury + winning_decision write (task 3)`

### Task 4: CREATE `admin_trigger_appeal_rejury` handler + DTO + route

**Goal:** new admin-only endpoint for the
`appeal.auto_select_on_appeal_acceptance = false` config path.

**ACTION:**

1. `crates/api/api_common/src/governance.rs` — add DTO:

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Admin-triggered appeal-rejury request — used when
/// `appeal.auto_select_on_appeal_acceptance = false`.
pub struct AdminTriggerAppealRejury {
  pub case_id: ModerationCaseId,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from admin-trigger-appeal-rejury.
pub struct AdminTriggerAppealRejuryResponse {
  pub case_id: ModerationCaseId,
  pub appeal_id: AppealId,
  pub panel_person_ids: Vec<PersonId>,
}
```

2. `crates/api/api/src/governance/admin_trigger_appeal_rejury.rs` —
   new file:
   - `is_admin` check on the caller (mirror `admin_close_case.rs`).
   - Load case; status MUST be `Appealed`, exhaustive match per
     ADR-013.
   - Verify no existing `jury_assignment` rows with `role = Appeal`
     exist for this case (idempotency).
   - Load the existing Appeal row for the case (the one filed by
     `request_appeal` while auto_select was off).
   - Call `admin_assign_jury::select_appeal_panel(conn, &case, &mut cache)`.
   - Insert N `JuryAssignmentInsertForm` rows with
     `role = Some(JuryAssignmentRole::Appeal)` + per-juror
     `selected_under_constraints` JSONB.
   - UPDATE the Appeal row to set `panel_size_snapshot` +
     `threshold_count_snapshot`.
   - Emit one `appeal_panel_assembled` governance_log entry attributed
     to admin pseudonym (mirror admin_assign_jury.rs:264-284 shape).
   - Wrap in `run_transaction` for atomicity.

3. `crates/api/api/src/governance/mod.rs` —
   `pub mod admin_trigger_appeal_rejury;` inserted alphabetically at
   line ~22.

4. `crates/api/routes/src/lib.rs` — import + route registration:
   - Import: `admin_trigger_appeal_rejury::admin_trigger_appeal_rejury,`
     at line ~37 area (alphabetical inside the existing
     `lemmy_api::governance::{...}` use block).
   - Route:
     `.route("/trigger-appeal-rejury", post().to(admin_trigger_appeal_rejury))`
     under the `/admin` scope (line ~538 area), after `/close-case`.

**MIRROR:** `admin_close_case.rs` (full file) for the admin-handler
shape. Reuse `select_appeal_panel` + the panel-seating block from Task
3 Part B (factor into a private fn in `request_appeal.rs` or
`admin_assign_jury.rs` if the duplication is meaningful — at impl
discretion).

**GOTCHA:** the appeal-panel-seating logic exists in TWO call sites
post-Task 4: (a) `request_appeal.rs` auto-rejury branch, (b)
`admin_trigger_appeal_rejury.rs`. Avoid duplication by factoring the
"insert N jury_assignment rows + emit appeal_panel_assembled + UPDATE
appeal snapshot" block into a private helper alongside
`select_appeal_panel`. Suggested signature:
`pub(crate) async fn seat_appeal_panel(conn, &case, &appeal, AppealPanelSelection, actor_pseudonym) -> LemmyResult<()>`.

**SKILL HINT:** `/cargo-validate` for the per-task gate.

**VALIDATE:**

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > /tmp/jm-d-task4-check.log 2>&1
echo "exit: $?"; tail -10 /tmp/jm-d-task4-check.log

bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > /tmp/jm-d-task4-clippy.log 2>&1
echo "exit: $?"; tail -10 /tmp/jm-d-task4-clippy.log

bash scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server > /tmp/jm-d-task4-test-no-run.log 2>&1
echo "exit: $?"
```

**COMMIT MESSAGE:** `feat(v1-JM-d): admin_trigger_appeal_rejury handler + route + DTO (task 4)`

### Task 5: CREATE `appeal_window_expiry` background job + scheduler tick

**Goal:** the hourly tick that flips Decided -> Closed for past-window
cases.

**ACTION:**

1. `crates/api/api/src/governance/appeal_window_expiry.rs` — new
   module per §10.6 (full body listed there).
2. `crates/api/api/src/governance/mod.rs` —
   `pub mod appeal_window_expiry;` alphabetical.
3. `crates/routes/src/utils/scheduled_tasks.rs`:
   - At module scope (lines 71-79 area): add
     `APPEAL_WINDOW_EXPIRY_RUNNING: AtomicBool` static +
     `AppealWindowExpiryRunningGuard` struct + `Drop` impl.
   - In `setup` after the reputation-snapshot tick (line 233):
     register the new hourly tick block per §10.6.
4. `crates/server/src/governance.rs` — append a one-line `info!` to
   the existing logged-jobs registry message documenting the new tick.

**MIRROR:** `crates/routes/src/utils/scheduled_tasks.rs:71-79`
(atomic-bool + guard) + `:177-233` (15-min tick block).
`crates/api/api/src/governance/reputation_snapshot.rs:361-407` (batch
fn shape with outcome struct + tracing).

**GOTCHA:** the tick is hourly (`CTimeUnits::hour(1)`) per PRD §17 row
4. The reputation tick is `CTimeUnits::minutes(15)`. Don't copy the
15-min cadence by accident.

**GOTCHA (DQ #50 / JM-c retro §3.2 amendment 1):** the SELECT uses
`for_update().skip_locked()`. SKIP LOCKED means a concurrent
`request_appeal` mid-tx on the same row gets passed over this tick;
the next tick (1 hour later) catches it. Acceptable: appeal windows
are bounded in days, not minutes, so a 1-hour delay on the close
transition is invisible to the user.

**GOTCHA:** the FOR UPDATE in `request_appeal.rs` is implicit in the
status-flip UPDATE statement (Diesel acquires the row lock on update);
this tick's `for_update().skip_locked()` is on the SELECT before the
batch UPDATE. The two cannot deadlock because:
- request_appeal: SELECT case -> INSERT appeal -> UPDATE moderation_case (acquires X lock here)
- bg job: SELECT FOR UPDATE SKIP LOCKED moderation_case -> UPDATE moderation_case
If the bg job's SELECT runs while request_appeal holds the row, SKIP
LOCKED makes it skip; if the bg job's SELECT runs first and
request_appeal arrives after, request_appeal blocks at its UPDATE
(proper FIFO). No deadlock by construction.

**GOTCHA:** the e2e tests must set `BREHON_DISABLE_APPEAL_WINDOW_JOB=1`
when bootstrapping `LemmyContext` so the cron tick doesn't race them.
The existing `BREHON_DISABLE_SNAPSHOT_JOB` env-var pattern (set once
at bootstrap-time) covers this — Task 6 e2e tests should use the same
discipline.

**GOTCHA:** the `actor_pseudonym` field of the governance_log entry is
`None` (system-issued by scheduler) — the `governance_log::append`
signature accepts `Option<String>`. Non-pseudonymised system events
are valid per ADR-015 (the pseudonymisation rule applies to USER
identity, not system identity).

**SKILL HINT:** `/cargo-validate` for the per-task gate.

**VALIDATE:**

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > /tmp/jm-d-task5-check.log 2>&1
echo "exit: $?"; tail -10 /tmp/jm-d-task5-check.log

bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > /tmp/jm-d-task5-clippy.log 2>&1
echo "exit: $?"; tail -10 /tmp/jm-d-task5-clippy.log

bash scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server > /tmp/jm-d-task5-test-no-run.log 2>&1
echo "exit: $?"
```

**COMMIT MESSAGE:** `feat(v1-JM-d): appeal-window-expiry background job + hourly scheduler tick (task 5)`

### Task 6: ADD e2e tests under `mod v1_jm_d_fixtures`

**Goal:** behavioral regression coverage for JM-d's six new behaviours.

**ACTION:**

Add a new `mod v1_jm_d_fixtures` block in `crates/server/tests/e2e.rs`
after `mod v1_jm_b_fixtures`. Reuse `v1_jm_b_fixtures::bootstrap` /
`seed_user` / `seed_jurors` / `seed_case` /
`seed_jury_eligible_snapshots` verbatim. The new fixtures (if needed)
are:

- `seed_appealed_case` — walks through admin_assign_jury ->
  submit_jury_vote with N votes for `JuryDecision::NoAction` (so
  winning_decision is NoAction) -> returns the decided case_id ready
  for an appeal.

Tests (7, all using R2 fixture-seeding discipline per JM-b retro §3.2
amendment 2 + JM-c §10.7):

| # | Test | Asserts |
|---|---|---|
| 1 | `request_appeal_within_window_succeeds_after_jm_c` | After JM-c-style decided case, `request_appeal` succeeds (the regression fix); appeal row inserted with `requester_role = Defendant`; case status flips Decided -> Appealed. |
| 2 | `request_appeal_past_window_returns_not_found` | Manually update `appeal_window_expires_at = now() - 1 day`; `request_appeal` returns `NotFound`. |
| 3 | `reporter_can_appeal_no_action_outcome` | Decided case with `winning_decision = NoAction`, caller is `case.creator_id`; appeal succeeds; `requester_role = OriginalReporter`. |
| 4 | `reporter_cannot_appeal_remove_content_outcome` | Decided case with `winning_decision = RemoveContent`; caller is `case.creator_id`; appeal returns `NotFound` (reporter eligibility denied). |
| 5 | `auto_rejury_seats_appeal_panel_excluding_originals` | `appeal.auto_select_on_appeal_acceptance = true` (default); request_appeal seats N rows in `jury_assignment` with `role = Appeal`; assert no person_id from the original panel appears in the appeal panel; assert appeal-panel size matches PRD §6.1 math (cross-reference `admin_assign_jury_severity_tier_*` for original-panel size; appeal-panel = max(ceil(orig × multiplier), orig + floor_increment) clamped to [3,11]). |
| 6 | `admin_trigger_appeal_rejury_seats_panel_when_auto_disabled` | Set `appeal.auto_select_on_appeal_acceptance = false`; request_appeal flips case to Appealed but seats no panel; admin calls `admin_trigger_appeal_rejury`; appeal-panel rows are inserted; `appeal_panel_assembled` log emitted with admin pseudonym. |
| 7 | `appeal_window_expiry_job_flips_decided_to_closed` | Set `BREHON_DISABLE_APPEAL_WINDOW_JOB=1` for the test; manually update `appeal_window_expires_at = now() - 1h`; call `appeal_window_expiry::run_appeal_window_expiry_batch(&ctx)` directly; assert case.status = Closed, closed_at populated, `appeal_window_expired` log entry emitted. |

**MIRROR:** existing JM-c tests (e2e.rs lines ~7400-7800 area) for the
direct-handler-invocation pattern. Use
`LocalUserView::read_person(&mut context.pool(), juror_id).await?`
(NOT a new helper — see JM-c retro §2.5).

**GOTCHA (per JM-c retro §3.2 amendment 2):** for any test that
asserts on the appeal-panel snapshot values (panel_size_snapshot,
threshold_count_snapshot on the Appeal row), reference the JM-b panel-
size assertion test
(`admin_assign_jury_severity_tier_regular_severe_panel_7_jurors`) via
inline comment rather than restating the math. The PRD §6.1 appeal-panel
math depends on `original_panel_size` which is set by JM-b's snapshot
logic; cross-reference avoids drift.

**GOTCHA (per JM-c retro §3.2 amendment 1):** Test 7 (bg-job
concurrency) should NOT use `tokio::join!` to race two requests against
the bg job. The job uses SKIP LOCKED, so a concurrent request_appeal
would just defer to the next tick. The test's contract is: synchronous
call -> case flipped. If a future "concurrent request_appeal during
job tick" test is wanted, file a follow-up GH issue; do not add it to
JM-d to avoid the FK-SHARE deadlock class (which is the same JM-c hit).

**GOTCHA:** R2 fixture seeding — every test that exercises
`admin_assign_jury -> submit_jury_vote` MUST call
`v1_jm_b_fixtures::seed_jury_eligible_snapshots(conn, &juror_ids)`
BEFORE `admin_assign_jury`. Without this, the small-pool fallback
fires and the snapshot fields take non-steady-state values.

**SKILL HINT:** `/test-write` is the right shape for these e2e tests —
the skill enforces the `LemmyResult<()>` + pseudonymisation +
no-`unwrap` discipline across new fixtures. Inline form is also
acceptable; reference the existing JM-c tests for the boilerplate.

**VALIDATE:**

```bash
bash scripts/brehon/cargo-test.sh --test e2e -p lemmy_server v1_jm_d > /tmp/jm-d-task6-tests.log 2>&1
echo "exit: $?"; tail -30 /tmp/jm-d-task6-tests.log
# Expected: exit 0; "7 passed; 0 failed"
```

**COMMIT MESSAGE:** `test(v1-JM-d): e2e coverage for bounded window + reporter-rights + auto-rejury + bg-job (task 6)`

### Task 7: Full-workspace validation pass + governance-log registry update + retro

**Goal:** full-suite green; registry flips committed; retro authored
per `feedback_retro_not_report.md`.

**ACTION:**

1. Update `.claude/rules/governance-log-entry-kind-registry.md` — flip
   `(pending)` markers per §10.7. Append a "v1-JM-d entry kinds (0
   new)" subsection. Run the count-check + duplicate-check at the
   bottom; both must hold.

2. Author `.claude/PRPs/reports/v1-JM-d-retro.md` per JM-c retro
   template (§§ TL;DR, "what worked", "what surprised", "what to carry
   forward", "what didn't need fixing", "quantified outcomes vs
   confidence", "tool-use self-assessment", "CR finding quality",
   "suggested action items for advisor", "for future v1-JM-e").

3. Run full validation:

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > /tmp/jm-d-task7-check.log 2>&1
echo "exit: $?"; tail -10 /tmp/jm-d-task7-check.log

bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > /tmp/jm-d-task7-clippy.log 2>&1
echo "exit: $?"; tail -10 /tmp/jm-d-task7-clippy.log

bash scripts/brehon/cargo-test.sh --test e2e -p lemmy_server > /tmp/jm-d-task7-e2e.log 2>&1
echo "exit: $?"; tail -60 /tmp/jm-d-task7-e2e.log
# Expected: exit 0; "67 passed; 0 failed; 4 ignored"
# (60 pre-JM-d + 7 new JM-d tests = 67 passed; ignored count unchanged from JM-c)
```

4. Cross-cutting verification (per template §15.5):
   - Every new governance_log emission goes via
     `governance_log::append` (NOT direct INSERT). Grep:
     `rg 'governance_log::append' crates/api/api/src/governance/{request_appeal,admin_trigger_appeal_rejury,appeal_window_expiry}.rs`
     (where applicable).
   - Pseudonymisation: every per-juror `appeal_panel_assembled` payload
     uses `actor_pseudonym_helper::get_or_create` (or pseudonyms from
     the constraint_record, NOT raw `person_id`).
   - `CaseStatus` exhaustive match preserved at request_appeal status
     guard.
   - `JuryDecision` exhaustive match in the reporter-eligibility check
     (`matches!(case.winning_decision, Some(NoAction | AdvisoryLabel))`
     is exhaustive over `Option<JuryDecision>` because the negative
     branch falls into the `else` `Err`).
   - Hash-chain test still passes (the `case_decided` payload field
     additions don't affect chain integrity — JM-c established this
     property, JM-d preserves it).

**MIRROR:** `.claude/PRPs/reports/v1-JM-c-retro.md` for the retro
section structure; `.claude/PRPs/reports/v1-JM-b-retro.md` for the
"what worked / what surprised" rhythm.

**GOTCHA:** the e2e suite count goes from 60 (JM-c-end) to 67
(JM-d-end). If the count is 66 or 68, audit the diff — Test 6 may have
been mis-named or a JM-d test may have been merged into another.

**GOTCHA:** the registry's count-check invariant must equal
`19 + 4 + 2 + 1 + 6 + 1 = 33` (unchanged — no new consts added by
JM-d). If `rg -c '^pub const ENTRY_KIND_'` returns 34, JM-d added a
const it shouldn't have.

**SKILL HINT:** `/cargo-validate` for the gating cargo runs.

**VALIDATE:** see step 3 above.

**COMMIT MESSAGE:** `chore(v1-JM-d): governance-log registry flips + Task 7 retro + final validation (task 7)`

---

## 14. Testing strategy

Per `IMPLEMENTATION-PLAN-v0.md §5`: integration-only for v0/v1, no
unit tests until something breaks twice. All tests live in
`crates/server/tests/e2e.rs`.

### 14.1 Tests to add (7 new tests under `mod v1_jm_d_fixtures`)

See §13 Task 6 table above for the full list with assertions.

### 14.2 Edge cases covered

- Appeal-window past expiry -> reject (not bounded-NULL hopeful match)
- Reporter on RemoveContent / Cooldown / Warning / Suspend / Federation -> reject
- Defendant always succeeds (within window)
- Auto-rejury with `auto_select = true` (default config)
- Admin-triggered rejury with `auto_select = false`
- Bg-job idempotency (running twice does not re-flip already-Closed cases — verified by the `WHERE status = Decided` filter)
- Bg-job pseudonymisation absence (system event with no actor pseudonym is correct per ADR-015)

### 14.3 Edge cases NOT covered (out of JM-d scope)

- Concurrent `request_appeal` + bg job on same row -> JM-c-class
  deadlock surface; SKIP LOCKED in the bg job means the bg job defers,
  no test needed beyond the implicit "bg job runs once per hour, race
  window is sub-second" pragma. JM-c's deadlock test is `#[ignore]`'d;
  JM-d does NOT add a parallel-request appeal test.
- Appeal panel votes / `appeal_decided` emission — JM-e capstone.
- Reporter-rights for `Warning` / `Cooldown` outcomes — out of PRD
  §6.4 literal specification (NoAction + AdvisoryLabel only).
- Cross-instance jury — v2.

### 14.4 Pre-existing tests preserved

- `report_to_modlog_golden_path` — preserved (does not touch appeal
  path).
- `sanction_notice_round_trip` — preserved (the JM-c fixture-fix
  landed in PR #98).
- All 5 JM-c tests — preserved (auto-loaded via the e2e suite).
- All JM-b severity-tier tests — preserved.

---

## 15. Validation commands (DoD)

Per `.claude/rules/cargo-output-capture.md` + per JM-b/c retro R6:
**all clippy invocations use
`--workspace --features full --no-deps -- -D warnings`.** Wrappers
invoked via `bash scripts/brehon/...` (portable,
+x-bit-independent; see §4.2 watchpoints).

### 15.1 Static analysis (per task)

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-JM-d-<task>-check.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### 15.2 Lint (per task — uniform R6)

```bash
bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-d-<task>-clippy.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### 15.3 Test target compile (R7 — per task that touches a struct OR a re-export)

```bash
bash scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-JM-d-<task>-test-no-run.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### 15.4 e2e test execution (Tasks 6 + 7)

```bash
# Run JM-d tests by name pattern (Task 6)
bash scripts/brehon/cargo-test.sh --test e2e -p lemmy_server v1_jm_d > .claude/PRPs/debug/v1-JM-d-task6-tests.log 2>&1
echo "exit: $?"; tail -30 .claude/PRPs/debug/v1-JM-d-task6-tests.log
# EXPECT: exit 0; "7 passed; 0 failed"

# Run full e2e suite (Task 7)
bash scripts/brehon/cargo-test.sh --test e2e -p lemmy_server > .claude/PRPs/debug/v1-JM-d-task7-e2e.log 2>&1
echo "exit: $?"; tail -60 .claude/PRPs/debug/v1-JM-d-task7-e2e.log
# EXPECT: exit 0; "67 passed; 0 failed; 4 ignored"
```

### 15.5 Migration round-trip (Task 1 only)

```bash
cargo run -p lemmy_diesel_utils --features full -- revert --limit=2 > .claude/PRPs/debug/v1-JM-d-task1-revert.log 2>&1
echo "revert exit: $?"
cargo run -p lemmy_diesel_utils --features full > .claude/PRPs/debug/v1-JM-d-task1-reapply.log 2>&1
echo "reapply exit: $?"
# EXPECT: both exit 0
```

### 15.6 Cross-cutting verification (Task 7)

- [ ] Every new governance_log emission calls `governance_log::append(...)` (not direct INSERT)
- [ ] Every `appeal_panel_assembled` / `appeal_window_expired` payload uses pseudonyms or system-issued (None) per ADR-015
- [ ] `CaseStatus` exhaustive match preserved at request_appeal:93-105 + admin_trigger_appeal_rejury status guard
- [ ] No raw `person_id` or username written to governance_log payloads
- [ ] R1: every `i32 <-> i64` comparison uses `i64::from(...)`, never `as` cast (per JM-b/JM-c retros R1)
- [ ] R2: every JM-d test calls `seed_jury_eligible_snapshots` before `admin_assign_jury`
- [ ] R3: every InsertForm caller-site enumerated + minimal touch (3 prod + 3 e2e for JuryAssignmentInsertForm; per-form inventory documented in Task 2 commit body)
- [ ] R4: every JM-d test name lowercase snake_case (verified at Task 6)
- [ ] R5: Task 0 enumerated all probes (Docker + submodule + clippy-installed + 4 wrapper + clippy baseline + DoD smoke + branch verify + DQ pull = 10 probes)
- [ ] R6: all clippy invocations use `--no-deps` uniformly (verified by `rg 'cargo-clippy.sh' .claude/PRPs/plans/v1-jury-mechanics-d.plan.md` — every match must include `--no-deps`)
- [ ] R7: every task that touches a struct OR a re-export OR an InsertForm runs `bash scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server` (Tasks 2, 3, 4, 5)
- [ ] Registry count-check: `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` returns 33 (unchanged)
- [ ] Registry duplicate-check: `rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs | awk -F: '/ENTRY_KIND_/ {print}' | grep -oE '"[a-z_]+"' | sort | uniq -d` returns empty

### 15.7 Manual validation (optional — for impl session sanity check)

After Task 5 + 7, manually verify the bg job works:

```sql
-- Insert a fake Decided case with past appeal window
PGPASSWORD=password psql -h localhost -U lemmy -d lemmy <<SQL
INSERT INTO moderation_case (
  community_id, creator_id, target_type, target_person_id,
  reason_code, severity, status, threshold_score,
  decided_at, appeal_window_expires_at,
  panel_size_snapshot, quorum_snapshot, threshold_count_snapshot
)
VALUES (
  NULL, NULL, 'Person', (SELECT id FROM person LIMIT 1),
  'manual_test', 'Low', 'Decided', 1,
  now() - INTERVAL '2 days', now() - INTERVAL '1 day',
  5, 3, 3
) RETURNING id;
SQL
# Note the returned id

# Trigger the batch directly (via a small bin or a one-off cargo test)
# OR wait an hour for the scheduler tick

# Verify status flipped
PGPASSWORD=password psql -h localhost -U lemmy -d lemmy -c \
  "SELECT id, status, closed_at FROM moderation_case WHERE id = <returned_id>"
# EXPECT: status = 'Closed', closed_at IS NOT NULL
```

---

## 16. Acceptance criteria

- [ ] All 8 tasks (Task 0 audit + Tasks 1-7 deliverable) completed in dependency order
- [ ] §15.1 (cargo check) exit 0 after every task
- [ ] §15.2 (cargo clippy with `--no-deps -- -D warnings`) exit 0 after every task
- [ ] §15.3 (cargo test --no-run) exit 0 after every task that touches a struct or re-export (Tasks 2, 3, 4, 5)
- [ ] §15.4 (e2e tests) — 7 new JM-d tests pass; all pre-existing tests still pass; 4 ignored carriers preserved
- [ ] §15.5 (migration round-trip) — `revert --limit=2` then re-apply both exit 0
- [ ] §15.6 (cross-cutting verification) — all 13 boxes ticked
- [ ] No contradictions with the 15 ADRs (ADR-008 + ADR-010 + ADR-013 + ADR-015 explicitly verified)
- [ ] Registry count remains 33; no new ENTRY_KIND consts; (pending) markers flipped per §10.7
- [ ] No edits to files outside §11 list
- [ ] Retro file written + validation logs captured in `.claude/PRPs/debug/`
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`
- [ ] DQ #51 (planner) and DQ #52 (planner) resolved by advisor before PR merge (or at PR-merge boundary), confirming the MIRROR-ref drift documented + the winning_decision storage approach approved

---

## 17. Completion checklist

- [ ] Task 0 audit complete (probes 0-9 confirmed)
- [ ] Task 1 (migrations) committed
- [ ] Task 2 (Diesel models + InsertForm extensions + R3 sweep) committed
- [ ] Task 3 (request_appeal rewrite + submit_jury_vote winning_decision write + select_appeal_panel helper) committed
- [ ] Task 4 (admin_trigger_appeal_rejury handler + DTO + route) committed
- [ ] Task 5 (appeal_window_expiry job + scheduler tick + governance.rs log) committed
- [ ] Task 6 (7 e2e tests) committed
- [ ] Task 7 (registry flips + retro + final validation) committed
- [ ] PR opened by BM session against `governance-v0`
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`
- [ ] Post-merge JM-d branch retained for retro reads (per JM-b/c precedent)

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `winning_decision` write at submit_jury_vote.rs collides with SL-d's step-7 graft when SL-d merges later | LOW | LOW | The `winning_decision` field write goes inside the existing post-decision UPDATE block (status/decided_at), not at the SL-d graft anchor (apply_sponsor_liability call). Disjoint statements; merge resolves cleanly. Watchpoint in §4.2. |
| `JuryAssignmentInsertForm.role` field addition breaks existing struct literals | LOW | LOW | R3 sweep enumerates 6 sites at Task 2 start; `..Default::default()` at each per `feedback_insertform_default_propagation.md` option (b). Cargo check at Task 2 catches any missed site. |
| Auto-rejury panel sizing math (PRD §6.1: 1.5× rounded up, min original+2, capped [3,11]) produces unexpected values for some panel sizes | LOW | LOW | The math is fully deterministic; Task 6 Test 5 asserts the exact computed value for the test fixture's panel size, and the assertion comment cross-references PRD §6.1 + §10. |
| Bg job + concurrent `request_appeal` deadlock (DQ #50 class) | LOW | MED | SKIP LOCKED on the bg job's SELECT — request_appeal mid-tx on a row gets passed over; next tick catches it. Watchpoint in §4.2; design rationale in §10.6. |
| The `closed_at` write the bg job adds breaks pre-existing `report_to_modlog_golden_path` assertions | LOW | LOW | The golden-path test asserts on the post-Decided closed_at being populated — JM-c removed that write, but the test was already updated in JM-c. The bg job restores `closed_at` writes for the natural-expiry path; the golden-path test runs before any bg-tick fires (e2e disables the tick via env var), so closed_at stays NULL during the test -> test does NOT depend on bg-job state. Verified via Task 0 grep. |
| `appeal.panel_size_multiplier × original_panel_size` is a float; ceiling-narrowing trips clippy::as_conversions | LOW | LOW | Reuse `admin_assign_jury::ceil_count` per §10.4 — already has `#[expect(clippy::as_conversions, ...)]` justified by the bound on inputs. JM-d does NOT add new `as` casts. |
| Migration timestamp prefix collides with another v1 sub-phase migration landing in parallel | LOW | LOW | Task 1 GOTCHA names the timestamp resolution at impl time; `ls migrations/` sort confirms position. Junior worktrees serialize per per-PRD branch, so the only concurrent-migration risk is rep-tuning-r1 — different table, different prefix range. |
| Submodule init or clippy install fails at Task 0 -> cannot start | LOW | HIGH | Task 0 probes -1 / -2 STOP if missing; advisor takes the chmod / install action outside the worktree, then Junior re-runs Task 0. Adds a round-trip but caught early. |
| Mid-phase context-window exhaustion (post-Phase-1 ~360k token zone) | LOW | MED | JM-d is 8 tasks (vs JM-c's 8); explicit handover protocol if mid-phase context > 180k; Task 0 enumerates probes to avoid late-phase audit cascades. Plan §10 snippet discipline keeps re-reads down. |
| `.claude/rules/governance-log-entry-kind-registry.md` count-check fails post-task-7 because JM-d accidentally added a new const | LOW | LOW | Task 7 acceptance gate runs the count-check; the impl reverts the unintended const if it appears. PRD §17 row 4 explicitly says "no new ENTRY_KIND consts". |
| Reporter-rights eligibility produces wrong allow on `Warning` / `Cooldown` outcomes | LOW | MED | The `matches!(case.winning_decision, Some(NoAction | AdvisoryLabel))` is exact and exhaustive over the 8 JuryDecision variants; tests 3 + 4 both happy-path and negative-path the eligibility check. |
| `appeal.panel_size_floor_increment = 2` + small original panel = clamp to [3,11] cap | LOW | LOW | Bounds are already enforced upstream by `admin_assign_jury_severity_tier_*` tests; JM-d's `compute_appeal_panel_size` clamps via `.clamp(3, 11)`. |
| LSP / rust-analyzer-lsp unavailable on Junior daemon -> impl agent struggles to verify select_eligible_jurors signature drift | LOW | LOW | Plan §9.2 lists the file:line for the signature; impl falls back to `Read` + `Grep` (per `feedback_advisor_watchpoint_specificity.md`). |

---

## 19. Notes

- **PRD §9.5 MIRROR-ref drift (DQ #51).** PRD §9.5 names "the existing
  `expired sanction cleanup` job pattern in
  `crates/server/src/governance.rs`" as the MIRROR ref for the new
  appeal-window-expiry job. That file is a 28-line declarative stub
  with NO sanction-cleanup code; the actual Brehon governance-tick
  precedent is
  `lemmy_api::governance::reputation_snapshot::run_snapshot_batch`
  registered at `crates/routes/src/utils/scheduled_tasks.rs:177-233`.
  This plan uses that as the MIRROR. **DQ #51 (planner-attributed)
  raised** for advisor confirmation — non-blocking; if the advisor
  prefers a different MIRROR (e.g. Lemmy's `update_banned_when_expired`
  at scheduled_tasks.rs:610), Task 5 can be re-scoped.

- **`winning_decision` storage choice (DQ #52).** PRD §9.3 leans toward
  "add a `moderation_case.winning_decision` column" but lists the
  alternative ("re-tally jury_vote rows in request_appeal") as also
  valid. This plan chose the column approach (cleaner, faster,
  one-line submit_jury_vote write). **DQ #52 (planner-attributed)
  raised** for advisor confirmation. If the advisor prefers re-tally,
  Task 1 drops the `moderation_case.winning_decision` ALTER, Task 2
  drops the `ModerationCase.winning_decision` field, Task 3 drops the
  submit_jury_vote write and adds re-tally logic in `request_appeal.rs`
  reporter-eligibility branch. Both options ship the same observable
  behaviour from the API surface.

- **Appeal-panel snapshot location.** PRD §6.6 specifies the appeal
  panel size + threshold can be configured via `appeal.*` knobs. The
  plan stores the appeal-panel snapshot on the `appeal` row (not
  `moderation_case`) to satisfy ADR-010's snapshot immutability for the
  original verdict's parameters. See §10.4 + §13 Task 1.

- **`closed_at` semantics post-JM-d.** Pre-JM-d post-JM-c-merge:
  `closed_at` was always NULL on Decided cases. Post-JM-d-merge:
  `closed_at` is set by the bg job when transitioning Decided -> Closed
  (or by `admin_close_case` on manual close — pre-existing path).
  Pre-v1 cases retain JM-a-backfill `closed_at`. JM-d does NOT
  retroactively backfill `closed_at` for newly-Decided post-JM-c cases
  that have not yet expired.

- **The reporter-rights extension is the only PRD §6.4 implementation
  in JM-d.** Spoofing protection (§12.4) is satisfied by construction —
  `case.creator_id` is NULL for orphaned cases (deleted reporter) ->
  reporter-rights branch evaluates `Some(caller_id) == None` -> false
  -> falls through to `Err(NotFound)`. No additional spoofing-protection
  code needed.

- **No federation publishes from JM-d.** `appeal_panel_assembled`,
  `appeal_window_expired`, etc. are local-only governance_log entries
  (per ADR-014 — federation of governance signals is fork-only AP types
  and is currently scoped to sanction-related events, not
  appeal-related). If a future federation phase adds appeal AP types,
  JM-d's emissions are upstream candidates.

- **Confidence score: 8/10.** Per JM-c retro §5: amendments 1+2+3+4
  apply to JM-d and lift confidence; the 2-point discount is for: (1)
  the cross-file edit to submit_jury_vote.rs (single-line, but a
  cross-file collision risk with SL-d that has not yet shipped), (2)
  the new bg-job pattern (first JM-side job; the reputation_snapshot
  precedent is well-trodden but the `for_update().skip_locked()` shape
  is novel-to-Brehon for batched UPDATE). The plan §10 snippet
  discipline + per-task validation gates + R3 sweep enumeration are all
  proven. Test count + test-fixture inheritance from JM-b's
  `seed_jury_eligible_snapshots` give high coverage at low marginal
  cost.

- **Task 0 audit logs and Task 7 validation logs land in
  `.claude/PRPs/debug/`** — the directory is gitignored
  (`.claude/.gitignore` line 2). Logs stay local; no commit touches
  them. The Task 7 retro summarises validation outcomes.

- **HARNESS NOTE FOR IMPL SESSION** (planning-side observation, 2026-04-26):
  the Junior worktree's Claude Code harness blocks Write/Edit on
  `.claude/PRPs/plans/**` and `.claude/decision-queue.json` due to a
  built-in sensitive-file matcher; the planning subagent landed this
  plan via `cp /tmp/<file> .claude/PRPs/plans/...` (Bash bypasses the
  matcher). Impl session should be aware that Edit/Write to these
  paths may also be blocked; fall back to Bash `cat <<'EOF' > target`
  or `cp /tmp/staging .claude/...` if needed. The advisor / BM session
  can add an explicit allow rule
  (`"permissions": {"allow": ["Edit(./.claude/**)", "Write(./.claude/**)"]}`)
  in `.claude/settings.local.json` to remove this friction; that's
  out-of-band infra work, NOT part of JM-d's scope.

---

## 20. Sub-phase stub (v1-JM-e)

Per the v1-AD / v1-JM-a/b/c precedent: JM-e is its own plan file, owns
its own PR -> `governance-v0`, earns its own CodeRabbit review, and gets
written only after JM-d merges.

### v1-JM-e — Capstone test + cross-sub-phase integration assertions + appeal-vote tally

- **Appeal-vote tally** in `submit_jury_vote.rs`: detect `role = Appeal`
  jury_assignment + Appeal-row presence; compute the bumped-tier
  threshold tally; emit `appeal_decided` (the one ENTRY_KIND const
  still pending after JM-d). Set `appeal.decided_at`.
- **Cross-sub-phase integration tests**: full lifecycle from report ->
  assign (JM-b) -> vote (JM-c) -> appeal (JM-d) -> re-jury (JM-d) ->
  appeal-decision (JM-e) -> close (JM-d bg job).
- **Audit log invariant test**: every case's `governance_log` sequence
  matches the expected state-transition shape.
- **Mid-flight config-churn regression test** — extends JM-c's test 5
  with appeal-window churn during the appeal phase.
- **Step-up auth stub for severity-tier changes mid-case** (PRD §12.3) —
  v1 ships `step_up_token: Option<String>` DTO slot.
- **Constraint-relaxation admin-visibility check** (PRD §12.2).
- ~4-5 tasks, closes the JM PRD.

---

_Plan author: planning subagent (Junior `jm-d-planning-1`,
2026-04-26). Plan committed on `junior/jm-d-planning-1` (Junior
worktree branch); finalize step pushes to that branch and the advisor
session merges via the standard sub-phase flow into `governance-v0`
where JM-d's BM-task then cuts `phase-v1-JM-d`. Confidence 8/10. Two
planner DQs raised (#51 PRD MIRROR-ref drift; #52 winning_decision
storage choice) — non-blocking for plan approval, advisor confirms
before impl Task 1._
