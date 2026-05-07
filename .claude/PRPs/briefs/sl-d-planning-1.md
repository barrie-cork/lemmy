# SL-d planning brief

> **Rewrite 2026-05-07** — supersedes the 2026-05-03 draft. The original carried a Hypothesis A/B scope-resolution DQ (PRD-aligned narrow vs homeserver-advisor-context broad). The user has since locked the **PRD-aligned narrow scope** for the SL lane (confirmed by SL-a/SL-b/SL-c all shipping per PRD §15 row-by-row assignments); the broader homeserver-advisor-context interpretation has been retired. This rewrite drops Hypothesis B entirely.

**Written**: 2026-05-07 by advisor session (this Mac, brehon-fork CWD `/Users/barrie/Developer/lemmy-advisor-sl-c` — sibling worktree on `governance-v0`) for execution by either Junior on EliteDesk OR a local `planning` subagent invocation depending on EliteDesk daemon state at dispatch time.
**Subagent target**: `planning` (Opus 4.7, color purple — see `.claude/agents/planning.md`).
**Worktree**: Junior cuts `junior/sl-d-planning-1` from `governance-v0` per concurrency-1 default; OR if executed locally, the `planning` subagent operates in a sibling git worktree on `governance-v0`. The plan file commits and pushes to `governance-v0` at finalize.
**Authority anchor**: `v1-sponsor-liability.prd.md` §9.1 (`apply_sponsor_liability` split — **SL-d's primary deliverable**) + §9.3 (`submit_jury_vote` mutation — what SL-d contributes into the JM-PRD-§9.1 integrated 9-step handler) + §15 row 4 (`v1-SL-d — submit_jury_vote mutation + apply_sponsor_liability split`). PRD §15 row 4 explicitly **depends on Phases 1, 3 (SL-a + SL-c)**.

**Scope (locked, no Hypothesis question)**: SL-d is the v0→v1 producer-side rewrite. Two deliverable halves:
1. **`apply_sponsor_liability` split** into `compute_sponsor_liability` (pure read) + `fire_sponsor_liability` (DB writes) + thin wrapper retaining the v0 signature for backwards-compat.
2. **`submit_jury_vote` mutation** at `crates/api/api/src/governance/submit_jury_vote.rs:458-493` — replace the v0 immediate-fire path with the `Decided → SponsorLiabilityPending` transition + grace-window computation + deferred-write set.

The TODO marker at `submit_jury_vote.rs:458` (`TODO(v1-sponsor-liability-d): replace this v0 apply_sponsor_liability call with the [...]`) is the literal SL-d work site — JM-c shipped that marker as a hand-off pointer.

---

## 1. Role + dispatch line

`[role:planning] v1-sponsor-liability-d plan — apply_sponsor_liability split + submit_jury_vote mutation`

The actual create-task description is a single line under 100 chars per `.claude/rules/advisor-orchestrator.md` Junior task description template:

```
[role:planning] v1-sponsor-liability-d plan — see .claude/PRPs/briefs/sl-d-planning-1.md
```

Everything else lives in this brief.

---

## 2. Scope — what to produce

Produce **one plan file** at `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md` for sub-phase **v1-SL-d**. The plan covers PRD §9.1 (split) + §9.3 (mutation) + the §13 cross-cutting ADR-013 exhaustiveness updates SL-d's new match arms force.

### 2.1 Five concrete deliverables (per PRD §9.1 + §9.3)

The plan's §13 task list MUST cover all five:

a. **Split `apply_sponsor_liability`** at `crates/api/api/src/governance/sponsor_liability.rs:142` into three functions:
  - **`compute_sponsor_liability(conn, target_person_id, case_id, community_id, action: SanctionAction, cache: &mut ConfigCache) -> LemmyResult<Vec<SponsorDelta>>`** — pure read. Computes severity, raw_delta, sponsor enumeration, per-sponsor base + remainder distribution, founder multiplier, zero-floor clamp. Returns the deltas. **No DB writes; no governance_log appends; idempotent.**
  - **`fire_sponsor_liability(conn, deltas: Vec<SponsorDelta>, case_id, target_person_id, action, cache) -> LemmyResult<usize>`** — DB writes. For each delta: insert `reputation_event` row, append `sponsor_liability_applied` log entry, append `sponsor_liability_clamped` log entry where the zero-floor clamp engaged. Returns count of sponsors processed.
  - **`apply_sponsor_liability(...)` as thin wrapper** — preserves the v0 signature (`pub(crate) async fn apply_sponsor_liability(conn, target_person_id, case_id, community_id, action, cache) -> LemmyResult<usize>`). Body: `let deltas = compute_sponsor_liability(...).await?; fire_sponsor_liability(deltas, ...).await`. Per PRD §9.1 — "Existing `apply_sponsor_liability` becomes a thin wrapper retained for **internal call sites that DO want immediate fire** (none in v1, but preserved for v2 admin-override paths)." **Importantly: SL-c's scheduler calls this wrapper directly (per SL-c brief masthead lock); the wrapper preserves SL-c's call-site behaviour with zero changes to SL-c's code.**

  Define `pub(crate) struct SponsorDelta { pub sponsor_id: PersonId, pub raw_delta: i64, pub clamped_delta: i64, pub clamp_engaged: bool, pub founder_multiplier_applied: bool }` (or similar — planner's call on exact field names; the brief's shape is illustrative).

b. **Mutate `submit_jury_vote.rs`** at the v0 immediate-fire site (`submit_jury_vote.rs:458-493`, verified at brief-write time). PRD §9.3 specifies what SL-d contributes into the JM-PRD-§9.1 integrated 9-step handler:
  1. **`SponsorLiabilityPending` status transition.** When sanction has liability implications AND target has active sponsors, replace `moderation_case::status.eq(CaseStatus::Decided)` (line 493) with `moderation_case::status.eq(CaseStatus::SponsorLiabilityPending)`.
  2. **Grace-window computation.** `grace_expires_at = now() + grace_window_for_severity(severity)`, snapshotted onto `moderation_case.grace_expires_at` at the Pending transition. Severity is the snapshot severity (PRD §4.3) read from the case row, not re-evaluated from config at fire-time.
  3. **Deferred write set.** On the Pending branch, `public_case_log` entries and juror `reputation_event` rows are NOT written at vote-tally time — they fire from SL-c's scheduler when the case transitions to Fired/Escaped (per PRD §11.4 documented behavioural change). The no-sponsor path (when `target_person_id` has zero active sureties) preserves v0 immediate-`Decided` semantics + immediate juror reputation writes per PRD §11.3.
  4. **Compute-only inline call.** Replace the v0 `sponsor_liability::apply_sponsor_liability(...)` call at line 471 with `sponsor_liability::compute_sponsor_liability(...)` — compute deltas and snapshot them onto `moderation_case` (or pass them forward to the scheduler via a deferred-write column; PRD §9.3 leaves this implementation-detail open). **Planner DQ if PRD §9.3 doesn't lock it precisely** — see §4.2.

c. **Helper: `grace_window_for_severity(severity: CaseSeverity, cache: &mut ConfigCache, conn) -> LemmyResult<chrono::Duration>`** — pure read; reads the appropriate `liability.grace_window_<minor|moderate|severe>_hours` config key per severity tier (per PRD §4.1 mapping); returns the duration. Lives in `sponsor_liability.rs` (sibling of `severity_for_action` at line 112). NOT in `sponsor_liability_grace.rs` (SL-c's module) — SL-d's mutation is in the api crate; the helper belongs alongside its peers.

d. **Cross-cutting ADR-013 exhaustiveness updates** at the `submit_jury_vote.rs` boundary. SL-a shipped the 3 new `CaseStatus` variants and updated SL-a's-shipped match sites (admin_dashboard.rs, get_case.rs, etc.). SL-d adds NEW match sites in `submit_jury_vote.rs` itself (e.g. the new branch logic comparing decision-bearing-sanction against active-sponsors); each new `match case.status { ... }` site SL-d introduces must enumerate ALL 12 variants explicitly per ADR-013 (no `_ =>` arms). Plan §13 task that authors the mutation must include a grep step: `grep -n "match.*CaseStatus\|case_status::" crates/api/api/src/governance/submit_jury_vote.rs` after edits to confirm zero wildcard arms. **SL-d does NOT update match sites OUTSIDE `submit_jury_vote.rs`** — those landed in SL-a (per SL-a plan §13 ADR-013 sweep tasks).

e. **Integration tests** for the producer side. The plan §13 must enumerate per-test-function tasks (one Edit per task, anchor-pattern based at file end) per `feedback_junior_worker_e2e_edit_hang.md` — `e2e.rs` is now ~10,500-10,700 lines (post-SL-a + post-SL-b + post-SL-c). Required test branches:

  1. **Decided → SponsorLiabilityPending transition** — synthetic case with target having 1+ active sureties, jury votes a liability-bearing sanction (e.g. CommunityExclusion). Assert: case transitions to `SponsorLiabilityPending` (NOT `Decided`); `grace_expires_at = decided_at + grace_window_severe_hours`; no `reputation_event` rows for sponsors yet (deferred); `case_decided` log entry IS emitted (auditor visibility per PRD §11.4).
  2. **No-sponsor path preserves v0 immediate-`Decided`** — target has zero active sureties (or all revoked pre-vote). Jury votes a liability-bearing sanction. Assert: case transitions to `Decided` (v0 path); no `SponsorLiabilityPending`; juror reputation events fire immediately; `apply_sponsor_liability` returns 0 (no sponsors, no writes).
  3. **NoAction path skips liability machinery entirely** — jury votes `JuryDecision::NoAction`. Assert: case still transitions to `Decided` (no liability evaluation); no `SponsorLiabilityPending`; no calls to `compute_sponsor_liability`.
  4. **Wrapper preserves v0 outputs** — test that `apply_sponsor_liability(...)` (the thin wrapper) produces byte-identical `reputation_event` rows + log entries to v0 output. Run pre-split (impossible at SL-d HEAD; conceptual: assert the `compute → fire` composition matches the historical single-pass output by golden-file comparison or row-count + content equality on a known fixture). Test #4 may be unit-only; planner judgment.
  5. **Compute is idempotent** — call `compute_sponsor_liability(...)` twice with the same inputs; assert returned `Vec<SponsorDelta>` is byte-equal both times; assert no DB rows written by either call. (Pure-function property test; lives in unit tests under `sponsor_liability.rs::tests` or a new `tests/` integration test.)
  6. **Grace-window severity mapping** — for each severity tier (Minor/Moderate/Severe), verify `grace_window_for_severity` reads the correct config key and returns the right duration. (Unit test or e2e with config seeding — planner judgment.)

  **Test count: 4-6** (4 e2e + 0-2 unit). Plan §13 may bundle compute-idempotency + grace-window-mapping into one §13 task if both are unit tests. Each e2e test = one §13 task with anchor-Edit at file end.

### 2.2 Inside-handler step ordering (PRD §9.3 + JM PRD §9.1 — load-bearing)

`submit_jury_vote::process_vote` already runs inside one outer `run_transaction` (per existing line 140 + `feedback_multi_write_handlers_need_transactions.md`). SL-d's mutation modifies the existing transaction body; does NOT add a new transaction.

Inside the existing run_transaction, SL-d's contributions slot into the integrated 9-step handler shape (per JM PRD §9.1):

(steps 1-6 unchanged from JM-c shipped state — load/validate/insert vote → tally → threshold check → winning-decision sanction insert)

7. **NEW (SL-d)**: After the sanction is inserted, evaluate sponsor-liability branch:
   - **If sanction has liability implications** (e.g. action ∈ {CommunityExclusion, InstanceSuspension, FederationQuarantineRecommendation, ContentRemoval, TemporaryRestriction, VisibilityReduction, Label, Restoration} per `severity_for_action`): query active sureties for `target_person_id`.
     - **If active sureties exist**: invoke `compute_sponsor_liability(...)` to compute deltas (no writes). Set `case.status = SponsorLiabilityPending`, `case.grace_expires_at = decided_at + grace_window_for_severity(severity)`. Snapshot computed deltas onto the case row OR persist them in a deferred-write table for SL-c's scheduler to consume — **planner DQ if PRD §9.3 doesn't specify the persistence mechanism**.
     - **If zero active sureties**: invoke v0 `apply_sponsor_liability(...)` (the thin wrapper) — returns 0 since no sponsors; no-op. Set `case.status = Decided`. v0 path preserved.
   - **If sanction has no liability implications** (no-sponsor sanctions like AdminWarning, NoAction): set `case.status = Decided`. v0 path preserved; no liability evaluation.
8. **NEW (SL-d)**: write `case_decided` governance_log entry (always, per PRD §11.4 deferred-write semantics — auditors see the decision event before the liability-resolution event lands later via SL-c's scheduler).
9. (existing JM-c step 9 — appeal-window bound — unchanged)

### 2.3 Scope boundary — what is NOT in SL-d

Per PRD §15 phase table:

- **SL-a (shipped):** Schema + 13 seeded keys + 5 entry-kind consts + 3 CaseStatus variants + Issue #24 partial index + backfill. SL-d READS these; does not add to them.
- **SL-b (shipped or in flight):** `revoke_endorsement` handler + DTO + route + 9 e2e tests. SL-d does NOT touch SL-b's handler. SL-b emits `sponsor_liability_escaped` for the revocation branch; SL-d's tests pre-seed cases with `SponsorLiabilityPending` via the new producer code-path (NOT direct DB-write — SL-d's tests EXERCISE the producer).
- **SL-c (shipped or being planned):** Scheduler module `sponsor_liability_grace.rs` + clokwerk wiring + 5-6 e2e tests. SL-c calls `apply_sponsor_liability` (unsplit) at SL-c HEAD. **SL-d's split makes `apply_sponsor_liability` a thin wrapper** — SL-c's call site continues to work without modification (signature preserved). Plan §13 may optionally include a task that updates SL-c's scheduler call site to call `fire_sponsor_liability` directly (post-compute) for clarity, but this is **optional** and the wrapper-preserving path is canonically correct.
- **SL-e (next, after SL-d):** Lane-wide e2e suite (full revocation-during-window-escapes flow exercising SL-c scheduler + SL-d transition end-to-end). SL-d's tests exercise the producer in isolation; SL-e's tests exercise the full lane.
- **restorative-mechanics-v1 (separate PRD, not yet drafted):** `restoration_completed` endpoint. SL-d does NOT wire restoration; SL-c stubbed the EscapeStatus enum; restoration producer comes from this future PRD.

**Hard out-of-scope for SL-d** (per PRD §2 OUT + §15):

- Cross-instance sponsor-liability federation (deferred to v2 per ADR-014).
- Step-up auth enforcement (PRD §12.3 — v1 reservation only).
- Restoration completion endpoint (PRD §7.4 — owned by restorative-mechanics-v1 PRD).
- Notification UX richness beyond Lemmy notifications (PRD §13 OQ-V1-SL-03 — v3 polish).
- The dashboard write surface for grace-window keys (PRD §14 — owned by admin-dashboard-v1 PRD; AD lane shipped).
- Any new migration. SL-a shipped the schema; SL-d adds zero migrations. Plan §13 must NOT include a migration task.
- Any new ENTRY_KIND_* const. All 5 SL-v1 consts shipped in SL-a (`endorsement_revoked` 195, `restoration_completed` 196, `sponsor_liability_escaped` 197, `sponsor_liability_fired` 198, `sponsor_liability_pending` 199 — verified via SL-c brief). Plan §13 must NOT add to the registry.
- New CaseStatus variants. SL-a shipped 3 (`SponsorLiabilityPending/Fired/Escaped`). SL-d uses them; does not add.
- Any `governance_config` seed. The 13 SL-a seeds cover SL-d's needs (read-only access to `liability.grace_window_<tier>_hours`).
- New HTTP endpoints, new DTOs, new routes. SL-d is a server-internal handler mutation; same call surface as v0.

### 2.4 Plan file deliverable shape

- **One file:** `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md`.
- **Follow `.claude/PRPs/templates/plan.template.md` 20-section schema literally.** §16a Stories is mandatory.
- **Shape G applies — SL-d is the fourth plan under Shape G** (after JM-e + SL-a + SL-b + SL-c). §15 DoD MUST use the per-workflow shape (workflow path + phase-branch SHA + `conclusion: "success"`). Inline cargo invocations are forbidden in §15.
- **§5 complexity score with breakdown table** per `feedback_complexity_score_pre_split.md`. Pre-estimate range **5-8** depending on how the planner counts: split + mutation + grace-helper = 3-4 §13 tasks (factor +0); plus 4 e2e tests = 4 §13 tasks (factor +12 if e2e edits weight 3; or +0 if anchor-Edit-per-task is light); plus 1-2 crates touched (factor +1 to +2). The mutation is well-bounded (one file, ~30 lines of edits at known line-ranges). If the planner's final tally exceeds 8, file split-or-proceed `kind: "blocker"` DQ before finalising §13. Possible split: SL-d1 (split + grace-helper) / SL-d2 (mutation + tests).
- **§16a Stories** — every story names (a) composing §13 tasks, (b) a checkpoint command in the Shape-G workflow shape, (c) Brief-Scope outputs to verify (file:line + symbol). Likely 3 stories: (1) "compute/fire split lands; wrapper preserves v0 signature; idempotent compute", (2) "submit_jury_vote producer mutation transitions to SponsorLiabilityPending on liability-bearing+sponsored cases; preserves Decided on no-sponsor and NoAction paths", (3) "grace_window_for_severity helper reads correct config key per severity tier".
- **§4 watchpoints** — every entry cites a specific file, table, or `schema.rs` line. The 12-watchpoint seed list below.
- **Explicit scope-boundary section in §6** ("Relationship to other v1-SL sub-phases") — mirror SL-c plan §6 shape.

**Commit only the plan file.** Do not author Rust code, do not open PRs, do not touch any file under `crates/`, `migrations/`, or `tests/`. Plan-file commit pushes to `governance-v0` after DoD smoke + watchpoint-specificity gates pass and the user approves.

---

## 3. Required reading (in order, before drafting any plan section)

1. `.claude/agents/planning.md` — your subagent contract.
2. `.claude/PRPs/templates/plan.template.md` — canonical Brehon plan template (20 sections incl. §15.6 Shape-G DoD and §16a Stories).
3. `.claude/PRPs/prds/v1-sponsor-liability.prd.md` — the parent PRD. **Read in full the first time.** Specifically:
   - §1 + §2 (vision/goals; scope IN/OUT — confirm SL-d is producer-side mutation only).
   - §3.1 + §3.4 (CaseStatus extensions — SL-d writes `SponsorLiabilityPending` only; the Fired/Escaped writes belong to SL-c).
   - §4.1 (severity-proportional grace windows — load-bearing for `grace_window_for_severity` helper).
   - §4.3 (severity is snapshotted at Decided-transition time per ADR-010 won't-disadvantage rule — load-bearing for the SL-d mutation).
   - §6 (scheduler — SL-c's; SL-d's mutation produces the cases SL-c consumes).
   - §8.1 (`liability_escape_reason` JSONB schema — SL-d does NOT write this column; SL-b/SL-c do).
   - **§9.1 (apply_sponsor_liability split — SL-d's primary deliverable; READ IN FULL).**
   - **§9.3 (submit_jury_vote mutation — SL-d's other deliverable; READ IN FULL).**
   - §11 (backwards compat — incl. §11.3 "v0 endpoint contracts preserved" — `submit_jury_vote` DTO + response unchanged; only internal lifecycle differs).
   - §15 (implementation phases — confirms SL-d is row 4; depends on Phases 1, 3 = SL-a + SL-c).
   - §17 (cross-cutting impact — confirms ADR-013 invariant).
   - §18 (resolutions applied — B6 row: §9.3 reduced from full pseudocode to pointer to v1-jury-mechanics.prd.md §9.1 — load-bearing).
4. **`.claude/PRPs/prds/v1-jury-mechanics.prd.md` §9.1** — the integrated 9-step `submit_jury_vote` handler shape SL-d's mutation slots into. **Read this section in full.** SL-d contributes the sponsor-liability branch (step 7) into JM PRD's pseudocode.
5. **`crates/api/api/src/governance/sponsor_liability.rs`** — **read the entire file (358 lines).** SL-d splits this. Specifically:
   - Line 142: `pub(crate) async fn apply_sponsor_liability(conn, target_person_id, case_id, community_id, action: SanctionAction, cache: &mut ConfigCache) -> LemmyResult<usize>`. The wrapper SL-d preserves; the body SL-d splits into compute + fire.
   - Line 112: `fn severity_for_action(action: SanctionAction) -> LiabilitySeverity` — pure function used inside `apply_sponsor_liability` AND by SL-d's new `grace_window_for_severity`.
   - Lines 1-50: module doc-comment incl. Watch 10 PII discipline. SL-d's split inherits this.
   - Body of `apply_sponsor_liability`: understand exactly which lines compute (severity, raw_delta, sponsor enumeration, per-sponsor base + remainder, founder multiplier, clamp) vs which write (`reputation_event` insert, `sponsor_liability_applied` log, `sponsor_liability_clamped` log). The split-line is the boundary between read-only computation and DB write.
6. **`crates/api/api/src/governance/submit_jury_vote.rs`** — **read the entire file (965 lines).** SL-d mutates it. Specifically:
   - **Line 458: `// TODO(v1-sponsor-liability-d): replace this v0 apply_sponsor_liability call with the [...]` — the literal SL-d work site.** JM-c shipped this marker as a hand-off.
   - Line 471: the v0 `apply_sponsor_liability` call site (inside run_transaction).
   - Line 493: `moderation_case::status.eq(CaseStatus::Decided)` — the line SL-d mutates to set `SponsorLiabilityPending` on liability-bearing+sponsored cases.
   - Line 105: `ALL_JURY_DECISIONS` const — verify shape; SL-d uses it but doesn't modify.
   - Lines 140-160: the outer `run_transaction` invocation.
   - Lines 161-280: `process_vote` body — the existing 9-step handler's first 6 steps (load/validate/insert vote → tally → threshold check → winning-decision sanction insert).
   - Lines 351-360 + 793: the iteration sites (`for candidate in ALL_JURY_DECISIONS`).
7. **`.claude/PRPs/plans/v1-sponsor-liability-c.plan.md`** — read **§6 (Relationship to other v1-SL sub-phases) and §11 (Files to change)**. Confirm SL-c calls `apply_sponsor_liability` (unsplit) — SL-d's wrapper preserves SL-c's call site.
8. **`crates/api/api/src/governance/sponsor_liability_grace.rs`** (SL-c's module, will exist when SL-d plans) — read the call site that invokes `apply_sponsor_liability(...)` from `fire_or_escape_case`. Confirm: SL-d's wrapper preserves this call site without changes. Optional plan §13 task: update this call site to call `fire_sponsor_liability` directly (post-compute) for clarity. NOT mandatory.
9. `crates/api/api/src/governance/governance_log.rs` — confirm:
   - Line 195: `ENTRY_KIND_ENDORSEMENT_REVOKED`.
   - Line 197: `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED`.
   - Line 198: `ENTRY_KIND_SPONSOR_LIABILITY_FIRED`.
   - Line 199: `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` — SL-d's mutation MAY emit this entry on the Decided→Pending transition; planner DQ if PRD §9.3 ambiguous on whether the transition itself logs. Default lean: yes, emit `sponsor_liability_pending` log entry on the transition for auditor parity with `sponsor_liability_fired`/`sponsor_liability_escaped`.
10. `crates/api/api/src/governance/config.rs` — find and read:
   - The default-string consts for `liability.grace_window_minor_hours`, `liability.grace_window_moderate_hours`, `liability.grace_window_severe_hours` (SL-a Task 5 seeded; verify const names + defaults 24/72/168).
   - The `ConfigCache` API + `Scope::Community` / `Scope::Instance` cascade pattern. SL-d's `grace_window_for_severity` reads instance-scoped (planner DQ if community cascade needed; default lean: instance-only, matching scheduler).
11. `crates/db_schema_file/src/enums.rs:411-432` — confirm:
   - `CaseStatus::SponsorLiabilityPending` variant (SL-d writes this).
   - The doc-comment at this variant: "Set by SL-d (`submit_jury_vote` rewrite); pre-v1 backfill in v1-SL-a sets it for v0 mid-flight cases." — this is the authoritative assignment of who-writes-what.
12. `crates/db_schema_file/src/schema.rs` — verify:
   - `moderation_case.grace_expires_at -> Nullable<Timestamptz>` (line ~799; SL-a-shipped).
   - `moderation_case.severity -> CaseSeverity` (line ~783).
   - `moderation_case.decided_at -> Nullable<Timestamptz>`.
   - `moderation_case.status -> CaseStatus`.
   - The `sanction` table — confirms `action -> SanctionAction`, `case_id`.
13. `crates/db_schema/src/source/moderation_case.rs` — verify the `ModerationCaseUpdateForm` accepts `status`, `grace_expires_at`. SL-d UPDATEs both via Diesel.
14. `crates/server/tests/e2e.rs` — locate (~10,500-10,700 lines post-SL-c). Read the first 100 lines (test-setup helpers); the most-recent SL-c tests (after SL-c merges; if SL-c still in flight at SL-d planning time, find SL-b tests instead); the ALL_JURY_DECISIONS-related test patterns; existing endorsement/surety/sanction test helpers.
15. **Glob `.claude/lessons/feedback_*.md` and Read every file whose name keyword matches:** `multi_write_handlers`, `transactions`, `for_update`, `idempotency`, `recompute_snapshot`, `pseudonym`, `pii`, `gdpr`, `redaction`, `scrub`, `actor_pseudonym`, `e2e_filter`, `e2e_edit_hang`, `junior_worker_e2e`, `complexity_score`, `pre_queue`, `watchpoint`, `dod`, `validate`, `shape_g`, `validate_pending`, `governance_log`, `entry_kind`, `seed`, `cargo`, `features-full`, `features_full_p_crate`, `wrapper-script`, `clippy`, `test-style`, `pre_phase_dod`, `dry_run`, `wrapper_silence`, `principles_not_rules`, `read_canonical`, `parallel_cohort`, `cohort_yaml`, `four_role`, `retro_not_report`, `insertform`, `propagation`, `branch_switch`, `commit_aggressively`, `daemon_finalize`, `pre_phase_harness_audit`, `multi_sponsor`, `escape_rule`, `restoration`, `version_tagged_json`, `severity`, `snapshot`, `wont_disadvantage`, `transaction_atomicity`, `pure_function`, `compute_fire_split`. That is the lessons-corpus discipline per `planning.md` step 3.
16. **Glob `.claude/lessons/reference_*.md` and Read every file whose name keyword matches:** `governance_log_entry_kind_registry`, `phase_branch`, `branch_manager`, `worktree`.
17. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — read **ADR-005** (multi-dimensional reputation), **ADR-008** (append-only log), **ADR-010** (won't-disadvantage rule — load-bearing for severity snapshot semantics), **ADR-013** (CaseStatus enum-exhaustiveness — load-bearing for SL-d's new match arms), **ADR-014** (federation deferral), **ADR-015** (pseudonymisation). OQ-025 (sponsor-liability v1 — resolution into this PRD).
18. `.claude/PRPs/plans/v1-jury-mechanics-c.plan.md` (or whichever JM phase shipped the line-458 TODO marker) — read §13 to see the JM-c handoff intent.
19. `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` and `v1-sponsor-liability-b.plan.md` — read §10 (file inventory) + §13 (task shape) + §15 DoD.
20. `.claude/PRPs/briefs/sl-a-planning-1.md`, `sl-b-planning-1.md`, `sl-c-planning-1.md` — exemplar planning briefs.
21. `.github/workflows/cargo-validate-workspace.yml` + `cargo-validate-features-full.yml` + `cargo-test-e2e.yml` — Shape-G workflows. SL-d does NOT add or edit workflows.
22. **DQ #67 resolved (per user 2026-04-27)** — workflow YAML DoD dry-run discipline. Honoured by SL-d §15.
23. **DQ #144-#147 resolved (advisor 2026-05-07 via /brehon-clarify on sl-c-planning-1)** — these clarify entries are SL-c-specific but two of them ripple through to SL-d:
    - **DQ #144 (ConfigCache lifetime)**: SL-d's `grace_window_for_severity` and the inline `compute_sponsor_liability` call in `submit_jury_vote::process_vote` thread the existing `process_vote`'s `&mut cache` (not a new cache) — `process_vote` runs inside one outer transaction; one cache for the entire transaction is the right shape. Different pattern from SL-c's batch-loop two-tier; SL-d is single-transaction-handler-shape.
    - **DQ #147 (paired canonical mirrors)**: SL-d's plan §10 cites `crates/api/api/src/governance/sponsor_liability.rs` (v0 code as MIRROR ref for the split) AND `crates/api/api_crud/src/governance/create_endorsement.rs` (the canonical mirror for inside-transaction multi-write handlers; same as SL-b's mirror).

---

## 4. Constraints (hard rules — violating any of these is a process breach)

### 4.1 Plan-content discipline

- **Shape G DoD shape (forward-only).** §15 MUST use the per-workflow shape per `.claude/PRPs/templates/plan.template.md` §15.6. Inline cargo invocations forbidden in §15.
- **§16a Stories mandatory.** Likely 3 stories (see §2.4).
- **§4 watchpoints cite specific files / handlers / `enums.rs` lines**, never abstract concepts. **The 12 watchpoints below are the seed list SL-d's plan §4 MUST include**, each with its citation gate:
   1. **TOCTOU on case-status mutation.** SL-d's mutation runs inside the existing `submit_jury_vote::process_vote` `run_transaction` (line 140). The case-row UPDATE happens inside the same tx as the sanction-insert + jury-tally — atomic. Plan §13 task that authors the mutation MUST verify: no separate-load-then-update; status comparison against `Decided` happens inside the tx.
   2. **Severity snapshot semantics.** Per ADR-010 won't-disadvantage rule + PRD §4.3: the severity used to compute `grace_window_for_severity` is the severity recorded ON THE CASE ROW at decision-transition time — NOT re-evaluated from config at fire-time. Plan §13 must specify: read `case.severity` from the moderation_case row that was just inserted/updated; pass that exact value to `grace_window_for_severity(severity, ...)`. Re-deriving severity from config at fire-time is a regression.
   3. **No-sponsor path preserves v0 immediate-`Decided`.** The new branch logic must check `surety` table for active sureties on `target_person_id`; if zero, the v0 path runs (status = Decided, juror reputation events fire immediately, `apply_sponsor_liability` returns 0 with no writes). Plan §13 task that authors the branch must include test #2 (no-sponsor path) as proof.
   4. **Wrapper signature preserved.** `pub(crate) async fn apply_sponsor_liability(conn, target_person_id, case_id, community_id, action, cache) -> LemmyResult<usize>` — same signature as v0 line 142. SL-c's call site continues to work without modification. Plan §13 must include a `git diff` step asserting the wrapper's signature line is byte-equal to v0.
   5. **Compute is pure.** `compute_sponsor_liability(...)` performs ZERO DB writes. No `reputation_event::insert`, no `governance_log::append`, no UPDATE. Plan §13 task must include a grep step: `grep -nE 'insert|update|append|persist' inside compute_sponsor_liability function body` returns zero matches.
   6. **Fire is the only writer.** All `reputation_event` INSERT + `sponsor_liability_applied`/`sponsor_liability_clamped` log appends move to `fire_sponsor_liability`. Plan §13 task must verify: `compute_sponsor_liability` returns deltas; `fire_sponsor_liability` writes them; no other code path writes liability events.
   7. **Idempotent compute** — calling `compute_sponsor_liability(...)` twice with identical inputs returns identical `Vec<SponsorDelta>`. Test #5 asserts. Property follows trivially from "no side effects in compute" but the test makes it observable.
   8. **`submit_jury_vote.rs` matches are exhaustive.** Per ADR-013 + `feedback_clippy_test_style.md` (R1 zero `_ =>` arms). Any new `match case.status { ... }` site SL-d adds must enumerate all 12 variants. Plan §13 task that authors the mutation must include grep step: `grep -nE 'match.*case\.status\b|match.*CaseStatus' crates/api/api/src/governance/submit_jury_vote.rs` post-edit returns zero `_ =>` lines.
   9. **`sponsor_liability_pending` log entry on transition.** PRD §9.3 deferred-write semantics: write `sponsor_liability_pending` log on the Decided→Pending transition for auditor parity with the eventual Fired/Escaped entries from SL-c's scheduler. Use `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` (governance_log.rs:199, SL-a-shipped). If PRD §9.3 doesn't explicitly say so, planner DQ.
   10. **Pseudonym discipline.** ADR-015. Every governance_log payload field naming a person uses `*_pseudonym`, never raw `*_id`. The `sponsor_liability_pending` payload includes `target_pseudonym` + `case_id` + `severity` (no raw `target_person_id`).
   11. **No new ENTRY_KIND_*** — all consts shipped in SL-a. Plan §13 must NOT add to the registry. Use existing `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` for the transition log.
   12. **e2e Edit-per-task discipline** per `feedback_junior_worker_e2e_edit_hang.md`. Each new e2e test = its own §13 task with anchor-pattern Edit at file end. e2e.rs is now ~10,500-10,700+ lines.

- **§5 complexity score breakdown table mandatory** per `feedback_complexity_score_pre_split.md`. Pre-estimate 5-8.
- **e2e edit discipline** per `feedback_junior_worker_e2e_edit_hang.md` + `feedback_e2e_filter_assumes_naming.md`.
- **Cargo feature/flag propagation** per `pattern_cargo_feature_flag_propagation.md` + `feedback_features_full_p_crate_incompatible.md`.
- **Build only what tests exercise** per PMD #14. SL-d ships the producer; SL-c already ships the consumer. Don't pre-implement consumer-side updates beyond the optional-call-site-update task.
- **Multi-write handlers transactionality** per `feedback_multi_write_handlers_need_transactions.md` — SL-d's mutation runs inside the existing `submit_jury_vote::process_vote` outer transaction. The ConfigCache used is the existing `&mut cache` threaded through `process_vote` (per DQ #144 single-handler pattern).
- **R-rule inheritance from JM-a/b/c/d/e + AD-a + SL-a/b/c retros** — every R1-R7 from prior retros applies.
- **Wrapper-script flag silence** per `feedback_wrapper_script_flag_silence.md`.
- **Pseudonym discipline** per ADR-015 + Watch 10.
- **Read-canonical-mirrors-first.** Plan §10 must cite `crates/api/api/src/governance/sponsor_liability.rs` (v0 code as the MIRROR ref for the split — same file SL-d edits, but the v0 shape is what SL-d preserves the wrapper of) AND `crates/api/api/src/governance/submit_jury_vote.rs:140-280` (the existing `process_vote` outer-transaction body — the canonical mirror for transaction lifecycle in `submit_jury_vote.rs`).

### 4.2 Decision-queue discipline (per `.claude/rules/decision-queue.md`)

- **Attribution integrity.** `from: "planner"` or `null`. NEVER `"advisor"`, `"user"`, `"impl-self-resolved"`. Subjects on planner DQ commits MUST start with `chore(decision-queue): planner raised DQ #<id> — <slug>`.
- **Mid-task DQ commits push immediately**, not at finalize. Push to `junior/sl-d-planning-1` (or whatever working branch the local agent uses).
- **Boundary-of-judgment — when to STOP and queue rather than guess:**
  - **Compute deltas persistence mechanism** — PRD §9.3 step 3 says "deferred write set" but doesn't lock how `compute_sponsor_liability`'s output gets passed to SL-c's scheduler. Options: (a) snapshot `Vec<SponsorDelta>` JSON on `moderation_case.computed_deltas` (new column? — but no migration in SL-d per §2.3), (b) recompute in `fire_sponsor_liability` from the case row + sanction + sureties at fire-time (idempotent so always correct), (c) some other persistence path. Default lean: **(b) recompute at fire-time** — `fire_sponsor_liability` calls `compute_sponsor_liability` internally and uses the result. This keeps SL-d migration-free and matches the wrapper composition `apply = compute + fire`. Surface as planner DQ if PRD §9.3 reads otherwise.
  - **`sponsor_liability_pending` log entry — emit at transition?** Watchpoint #9. Default lean: yes; surface DQ if PRD ambiguous.
  - **SL-c call-site update — optional or mandatory?** §2.3 says optional (wrapper preserves signature). If the planner judges the call should be updated for clarity, surface as DQ — advisor decides.
  - **Sanction-action multiplicity** — same question SL-c had. Plan §13 task that reads sanction action must specify SQL: `SELECT action, scope FROM sanction WHERE case_id = $case_id ORDER BY id ASC LIMIT 1`. If multiple sanctions per case, planner DQ.
  - **`grace_window_for_severity` location** — §2.1.c says `sponsor_liability.rs` (sibling of `severity_for_action`); if planner judges a different module fits better, file DQ.
  - **DO NOT file a scope-hypothesis DQ.** SL-d's scope is unambiguous from PRD §9.1 + §9.3 + §15 row 4; this brief is explicit. The retired Hypothesis A/B framing is gone.

### 4.3 File ownership

- **Touch only:**
  - `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md` (CREATE).
  - `.claude/decision-queue.json` (APPEND new pending or planner-resolved entries).
- **Do NOT edit anything under** `crates/`, `migrations/`, `tests/`, `docs/`, `.claude/PRPs/prds/`, `.claude/PRPs/reports/`, `.claude/agents/`, `.claude/rules/`, `.claude/commands/`, `.github/workflows/`, or any other plan file in `.claude/PRPs/plans/`.
- **One commit at finalize:** `feat(plan): v1-SL-d sub-phase plan` (matching SL-a/SL-b/SL-c/JM-* planner-task commit subjects).

### 4.4 Schema-first discipline

- **Read `crates/db_schema_file/src/schema.rs` BEFORE designing §13.** Confirm at planning time:
  - `moderation_case.grace_expires_at TIMESTAMPTZ NULL`, `liability_escape_reason JSONB NULL`, `decided_at TIMESTAMPTZ NULL`, `severity case_severity NOT NULL`, `status case_status NOT NULL`, `target_person_id INT4 NULL`, `community_id INT4 NULL`.
  - `case_status` enum 12 variants.
  - `sanction` table baseline + `surety` baseline + `endorsement` baseline.
- If any baseline assumption fails, file a `kind: "blocker"` DQ.

### 4.5 Cross-cutting from PMD-promoted patterns

- **Pattern: `multi_write_handlers_need_transactions`** — applies. SL-d's mutation lives inside the existing `process_vote` transaction.
- **Pattern: `verify_before_trusting_shell_output`** — when the planner runs git-grep to enumerate match sites, verify via direct file read.
- **Pattern: `cargo_feature_flag_propagation`** — `--workspace --features full` only.
- **Pattern: dual-file ENTRY_KIND edit** (per v1-AD-a §10.8 + v1-JM-a §15) — does NOT apply to SL-d (consts shipped in SL-a; SL-d reads via shim re-export).
- **Pattern: `read_canonical`** — v0 `apply_sponsor_liability` is the MIRROR ref for the split; v0 `process_vote` body is the MIRROR for the transaction lifecycle.

### 4.6 Attribution integrity reminder

The only valid `answered_by` labels for a Junior planning subagent are `"planner"` or `null`. Never `"advisor"`, `"user"`, `"impl-self-resolved"`.

---

**Lean / advisor-side tip (not a constraint):** SL-d is the **producer-side rewrite** that closes the SL lane's main loop. SL-a shipped the schema; SL-b the revocation handler; SL-c the consumer (scheduler). SL-d ships what produces the cases SL-c consumes — the `Decided → SponsorLiabilityPending` transition. The loop closes when SL-e ships (lane-wide e2e suite proving the full producer→consumer path).

A second observation: the JM-c TODO at `submit_jury_vote.rs:458` is the literal advance-handoff. JM-c shipped the marker explicitly so SL-d would know the exact mutation site. Plan §13 must include a `git log -p submit_jury_vote.rs | grep -B2 'TODO(v1-sponsor-liability-d)'` step to verify the marker is still present at planning HEAD; if it's gone (e.g. some intervening JM phase removed it), re-establish the line by reading `case_decided` log emit or `case_status::eq` at the v0 path.

A third observation: the wrapper-preserves-signature property is what makes SL-d non-disruptive to SL-c. SL-c's scheduler calls `apply_sponsor_liability(...)` with the v0 signature. After SL-d's split, that call still works — wrapper composition is transparent. Plan §13 may include an OPTIONAL task that updates SL-c's call site from `apply_sponsor_liability` to `fire_sponsor_liability(deltas, ...)` for clarity (after first calling `compute_sponsor_liability` in SL-c's flow). But this is purely cosmetic — the wrapper-preserving path is canonically correct and avoids touching SL-c's code at all. Plan should default to "don't update SL-c" and only include the update if the planner judges the post-split call site reads better.

A fourth observation: the deferred-write semantics (PRD §11.4) are the most subtle invariant. Auditors see `case_decided` immediately at vote-tally; then later (minutes-to-days) see `sponsor_liability_pending` → eventually `sponsor_liability_fired` or `_escaped` from SL-c's scheduler. Modlog readers polling for the case may see "no entry" between the Pending transition and the eventual Fired/Escaped. Plan §11 (Files to change) and §16a Stories should include checkpoints that verify this auditor-visibility pattern (e.g. story 2's checkpoint: "case present in `governance_log` with `case_decided` AND `sponsor_liability_pending` entries; absent from `public_case_log` until scheduler fires").

A fifth observation: SL-d's primary risk is **interaction with SL-c**. SL-c's scheduler picks up the cases SL-d's mutation creates. If SL-d's `grace_expires_at` calculation is off (wrong severity, wrong config key), SL-c's scheduler fires too early or too late. Test #1 (Decided→Pending transition) MUST assert the exact `grace_expires_at` value matches the snapshot severity's config-key default — this is the integration point between the two phases. Without this assertion, a regression in `grace_window_for_severity` would only surface at SL-e e2e time.

A sixth observation: complexity pre-estimate is 5-8. The dominant contributors are (a) the handler mutation (one file, ~30 lines of Edit), (b) the sponsor_liability.rs split (one file, restructure into 3 functions), (c) 4 e2e tests. Total §13 tasks: 6-8 (Task 0 + split + grace-helper + mutation + 4 e2e + retro). Cohort eligible for `[P]`: tests are anchor-Edit-per-task and could run parallel after split + mutation merges. Planner judges based on FILES YAML overlap.

---

_Brief author: advisor session (this Mac CWD `/Users/barrie/Developer/lemmy-advisor-sl-c`, sibling worktree on `governance-v0`, 2026-05-07). This rewrite supersedes the 2026-05-03 draft (which carried Hypothesis A/B scope-resolution baggage, now retired). Per the advisor-orchestrator clarify gate, advisor runs `/brehon-clarify .claude/PRPs/briefs/sl-d-planning-1.md` after this brief commits and pushes; planning task only queues after every clarify-DQ entry is resolved._
