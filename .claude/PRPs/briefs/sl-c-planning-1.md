# SL-c planning brief

**Written**: 2026-05-07 by advisor session (laptop, brehon-fork CWD) for Junior dispatch on EliteDesk.
**Subagent target**: `planning` (Opus 4.7, color purple — see `.claude/agents/planning.md`).
**Worktree**: Junior cuts `junior/sl-c-planning-1` from `governance-v0` per concurrency-1 default; the planning subagent commits the plan file and pushes back to `governance-v0` at finalize.
**Authority anchor**: `v1-sponsor-liability.prd.md` §6 (Grace-Window Scheduler — full spec) + §9.4 (helper module signatures) + §9.5 (scheduler wiring) + §15 row 3 (`v1-SL-c — Scheduler + grace-check helper`). SL-a shipped 2026-05-04 (PR #111, governance-v0 @ `790f6101d`); SL-b is in flight on `phase-v1-SL-b` (PR #119) at brief-write time. SL-c is the **third** SL-lane sub-phase to plan from this CWD.

**Compute/fire posture (advisor-locked, 2026-05-07)**: SL-c calls the **unsplit v0 `apply_sponsor_liability`** helper from the scheduler's fire branch. The PRD §9.1 compute/fire split is **NOT** SL-c's deliverable — it remains SL-d's. This locks the simpler interpretation of PRD §6.2 step 5 (which reads "now refactored per §9.1 into a 'fire' function gated on this scheduler" — that "now refactored" describes the post-SL-d state, not SL-c's prerequisite). SL-c's scheduler call site is forward-compatible with SL-d's split: when SL-d lands and `apply_sponsor_liability` becomes a thin wrapper around `compute_sponsor_liability` + `fire_sponsor_liability`, SL-c's call site can either keep calling the wrapper or be updated to call `fire_sponsor_liability` directly. Either is a no-op-behaviour change.

---

## 1. Role + dispatch line

`[role:planning] v1-sponsor-liability-c plan — sponsor_liability_grace scheduler module + clokwerk wiring + e2e`

The actual `mcp__junior-brehon__create_task` description is a single line under 100 chars per `.claude/rules/advisor-orchestrator.md` Junior task description template:

```
[role:planning] v1-sponsor-liability-c plan — see .claude/PRPs/briefs/sl-c-planning-1.md
```

Everything else lives in this brief.

---

## 2. Scope — what to produce

Produce **one plan file** at `.claude/PRPs/plans/v1-sponsor-liability-c.plan.md` for sub-phase **v1-SL-c**, the grace-window scheduler module + clokwerk wiring + observability + integration tests. The plan covers PRD §6 in full, §9.4 (helper module signatures), §9.5 (scheduler wiring), §6.4 (configuration knobs SL-a already seeded), and §6.3 (failure mode + observability). SL-a shipped the schema, enum variants, ENTRY_KIND consts, and config seeds; SL-b ships the revocation handler in parallel; SL-c reads SL-a's columns and writes to `moderation_case.{status, liability_escape_reason}`, emits two log entries (`sponsor_liability_fired`, `sponsor_liability_escaped`), and calls v0 `apply_sponsor_liability` from the fire branch.

### 2.1 Five concrete deliverables (per PRD §6 + §9.4 + §9.5)

The plan's §13 task list MUST cover all five:

a. **Create new module `sponsor_liability_grace.rs`** at `crates/api/api/src/governance/sponsor_liability_grace.rs`. Module wiring: add `pub mod sponsor_liability_grace;` to `crates/api/api/src/governance/mod.rs` (alphabetical position after `sponsor_liability`). The module exports three public async functions per PRD §9.4:
  - `pub async fn run_grace_check_batch(context: &LemmyContext) -> LemmyResult<()>` — scheduler entry. Reads `job.grace_check_batch_size` (default 100); queries pending cases with `status = 'SponsorLiabilityPending' AND grace_expires_at <= now() ORDER BY grace_expires_at ASC LIMIT batch_size`; iterates, opening one `run_transaction` per case (per-case isolation per PRD §6.2).
  - `pub async fn evaluate_escape_conditions(conn: &mut AsyncPgConnection, case_id, target_person_id, community_id, decided_at, cache: &mut ConfigCache) -> LemmyResult<EscapeStatus>` — pure read; returns `EscapeStatus::Escape{reason, actor_pseudonym, ref_id}` or `EscapeStatus::Fire`.
  - `pub async fn fire_or_escape_case(conn: &mut AsyncPgConnection, case_row, status: EscapeStatus, cache: &mut ConfigCache) -> LemmyResult<()>` — per-case transaction body. On `Escape`: UPDATE case to `SponsorLiabilityEscaped`, set `liability_escape_reason` JSONB (PRD §8.1 schema, `version: 1`), emit `sponsor_liability_escaped` log entry. On `Fire`: invoke v0 `apply_sponsor_liability(...)`, then UPDATE case to `SponsorLiabilityFired`, emit `sponsor_liability_fired` log entry.

  Also exports a sibling staleness check per PRD §6.3:
  - `pub async fn check_grace_staleness(conn, max_grace_seconds: i64, now: DateTime<Utc>) -> LemmyResult<()>` — pure observability; mirrors `reputation_snapshot::check_snapshot_staleness` at `reputation_snapshot.rs:423`. Emits `tracing::error!` if any pending case has `decided_at < now() - 2× max grace window`. Returns `Ok(())` either way; never DB-writes; never log-entry-writes.

  Per `feedback_build_what_tests_exercise.md` — drift-stub `EscapeStatus` enum variants the SL-c tests don't exercise. Specifically, the **restoration-completed escape branch** (PRD §6.2 step 3 second bullet) IS in scope for SL-c's `EscapeStatus` enum but **NOT** wired to a fire-site in v1-SL-c — see §2.3 for restoration scope. Plan §13 must produce an explicitly-stubbed `EscapeStatus::Escape{reason: "restoration_completed", ...}` constructor that returns `Fire` until restorative-mechanics-v1 lands. Document the stub-and-future-wire path in §19 Notes.

b. **Wire the scheduler** in `crates/routes/src/utils/scheduled_tasks.rs::setup`. Add a new `scheduler.every(...)` block per PRD §6.1, **mirroring** the existing `reputation_snapshot` block (lines 189-233 verified at brief-write time on governance-v0 HEAD `2a2649d88`). Specific deliverables in this file:
  - **Module-scope `static SPONSOR_LIABILITY_GRACE_RUNNING: AtomicBool = AtomicBool::new(false);`** plus `struct GraceCheckRunningGuard;` + `impl Drop for GraceCheckRunningGuard { fn drop(&mut self) { SPONSOR_LIABILITY_GRACE_RUNNING.store(false, Ordering::Release); } }` — sibling of `REPUTATION_SNAPSHOT_RUNNING`/`RunningGuard` at lines 71-79. Single source of truth for non-overlap.
  - **Read `job.grace_check_interval_minutes`** at scheduler `setup()` invocation time (NOT per-tick — clokwerk schedules pin at registration). Per PRD §6.4 the read happens once via `lemmy_api::governance::config::get_int(...)`; default 5; documented restart-required tunability.
  - **`scheduler.every(CTimeUnits::minutes(grace_interval_minutes)).run(move || ...)`** block with the body shape from PRD §6.1: env-var disable (`BREHON_DISABLE_GRACE_CHECK_JOB == "1"`), atomic compare_exchange concurrency guard, `let _guard = GraceCheckRunningGuard;`, then `lemmy_api::governance::sponsor_liability_grace::run_grace_check_batch(&context).await.inspect_err(|e| warn!(...)).ok()`.
  - **Optional staleness pass after the batch.** Per PRD §6.3 + the reputation_snapshot precedent at `scheduled_tasks.rs:209-232`. Calls `check_grace_staleness` after `run_grace_check_batch` returns, emitting `tracing::error!` if any pending case is past 2× max grace window (default 60 days = 168h × 2 ≈ 336h, but PRD §6.3 says "2× max grace window" so ≤720h × 2 = 1440h is the absolute upper bound governed by the `liability.grace_window_maximum_hours` ceiling). Plan §13 task that wires the staleness pass MUST cite the multiplier source: PRD §6.3 says "2× max grace window"; the actual `2.0` numeric default lives at `job.grace_check_staleness_alert_multiplier` (SL-a-seeded, verified at `config.rs:2672`). The check is pure observability — no DB writes, no governance_log entries.

c. **Sanction-action lookup for severity computation.** v0 `apply_sponsor_liability(action: SanctionAction)` requires the `SanctionAction` enum value to compute severity via `severity_for_action(action)` (verified at `sponsor_liability.rs:112`). The scheduler's case row carries `severity` directly (`moderation_case.severity` exists per `schema.rs:783`), but the v0 helper signature takes `action`, not severity directly. The scheduler MUST query the `sanction` table for the case's sanction row(s) and pass the action to `apply_sponsor_liability`. Plan §13 task that authors `fire_or_escape_case` must specify:
  - **SQL query**: `SELECT action, scope FROM sanction WHERE case_id = $case_id ORDER BY id ASC LIMIT 1` (verify shape at planning time — confirm the case→sanction multiplicity).
  - **Multiplicity invariant**: If the planner finds at planning time that `sanction.case_id` is many-to-one with `moderation_case.id` (multiple sanctions per case), file a `kind: "blocker"` DQ asking advisor whether to (1) iterate sanctions and fire liability for each, (2) take the highest-severity sanction, or (3) take the first/canonical sanction. Default lean: **(3) first sanction by id ASC** with a planner DQ surfacing the choice for transparency. The invariant likely is 1:1 (Phase 5b code shape suggests one sanction per case at decision-time), but verification matters.
  - **Empty-sanction edge case**: If `sanction` query returns zero rows for a `SponsorLiabilityPending` case, that's a v0 invariant violation (cases in this state by definition had a sanction inserted at decision-time). Plan §13 task must specify: log `tracing::error!` and skip the case silently (don't UPDATE status; don't crash batch). Add a watchpoint (§4 below).

d. **Integration tests** for the scheduler. The plan §13 must enumerate per-test-function tasks (one Edit per task, anchor-pattern based at file end) per `feedback_junior_worker_e2e_edit_hang.md` — `e2e.rs` is now ~10,300+ lines; SL-b will add ~9 more tests by the time SL-c plans, putting it at ~10,500-10,600. SL-c tests require synthetic seeding via direct DB-write of `SponsorLiabilityPending` cases (since SL-d's `submit_jury_vote → SponsorLiabilityPending` transition is not yet shipped at SL-c plan-time). Required test branches per PRD §6.1 + §6.2 + §6.3:

  1. **Fire path — expired pending case, no escape conditions** — seed a `SponsorLiabilityPending` case with `grace_expires_at = now() - 1 minute`, sanction inserted, surety active (no revocations). Manually trigger `run_grace_check_batch`. Assert: case transitions to `SponsorLiabilityFired`, `reputation_event` rows for sponsors written (count matches active sponsor count), `sponsor_liability_fired` log entry emitted, `sponsor_liability_applied` log entries emitted (one per sponsor — these are v0's per-sponsor entries from `apply_sponsor_liability`).
  2. **Escape path — sponsor revoked between decided_at and now** — seed a `SponsorLiabilityPending` case + active surety, then UPDATE surety `revoked_at = now()`. Trigger batch. Assert: case transitions to `SponsorLiabilityEscaped`, NO `reputation_event` rows for sponsors, `sponsor_liability_escaped` log entry emitted, `liability_escape_reason` JSONB matches `{"version": 1, "reason": "sponsor_revoked", "actor_pseudonym": "...", "endorsement_id": ...}`.
  3. **No-op — future grace_expires_at** — seed a `SponsorLiabilityPending` case with `grace_expires_at = now() + 2 hours` (not yet expired). Trigger batch. Assert: case still in `SponsorLiabilityPending`, no log entries, no reputation_event rows.
  4. **Per-case isolation — one bad case doesn't block batch** — seed two `SponsorLiabilityPending` cases (case A: well-formed, ready-to-fire; case B: malformed — e.g. references a deleted target_person_id row, or has zero sanctions). Trigger batch. Assert: case A transitions to `SponsorLiabilityFired`, case B remains in `SponsorLiabilityPending` with a `tracing::error!` log line surfaced (verify via log-capture if available), batch returns `Ok(())` not `Err`.
  5. **Batch size respects config** — seed `job.grace_check_batch_size = 2` and 5 expired pending cases. Trigger batch. Assert: only 2 cases transitioned (others remain pending). Trigger a second time: 2 more transitioned (total 4 pending → fired/escaped, 1 remaining).
  6. **No double-fire on already-Fired case** — seed a `SponsorLiabilityFired` case (status not Pending). Trigger batch. Assert: no UPDATE on case, no new log entries, no new `reputation_event` rows. Equivalent test on `SponsorLiabilityEscaped` is redundant; cover only the Fired-already case unless the planner judges otherwise.
  7. **(Optional, may defer to SL-e)** Restoration-escape — synthetic insertion of a `restoration_completed` log entry between case's `decided_at` and `now()`, then trigger batch. **PRD §6.2 step 3 lists this as an escape condition; PRD §7 (Restoration Interaction) defers the restoration-completion endpoint to restorative-mechanics-v1 PRD.** Plan §13 must surface the SL-c choice as a planner DQ: (i) wire restoration-escape detection in SL-c (read governance_log for `restoration_completed` entries with `subject_pseudonym = target's pseudonym, ts > decided_at`) and ship test #7, OR (ii) stub the `EscapeStatus::Escape{reason: "restoration_completed", ...}` branch but never let it fire in v1-SL-c (returns `Fire` from `evaluate_escape_conditions`); document deferral to restorative-mechanics-v1 in §19 Notes. **Advisor lean: option (ii)** — restorative-mechanics-v1 PRD doesn't exist yet, so detection-without-emission is incomplete. SL-c ships the consumer interface; restorative-mechanics-v1 wires the producer + the SL-c side of the read.

  **Test count estimate: 5 e2e tests** (drop #7 per advisor lean). Plan §13 may bump to 6 if the planner adds a staleness-detection unit-or-integration test for `check_grace_staleness`. Either way each test is one §13 task with anchor-Edit at file end.

e. **Test-side scheduler-disable** — `BREHON_DISABLE_GRACE_CHECK_JOB=1` env var disables the scheduler-tick block in `setup()` (mirrors `BREHON_DISABLE_SNAPSHOT_JOB` from `scheduled_tasks.rs:198`). e2e tests set this env var before bringing up the test server, then call `run_grace_check_batch` directly to drive ticks deterministically. Plan §13 task that wires the env-var check is sibling to deliverable (b); plan §14 Testing strategy must note that all SL-c e2e tests rely on this env-var override.

### 2.2 Inside-batch step ordering (PRD §6.2 — load-bearing for plan §13 task split)

PRD §6.2 enumerates the per-case transactional body. Plan §13 must order these correctly; reversing steps 1 and 2 yields scheduler-vs-handler races (see §4 watchpoint #2):

For each case row from the batch query, inside its **own** `run_transaction` (per-case isolation):

1. **Re-load case row with `FOR UPDATE`** — defends against scheduler-vs-handler race (e.g. SL-b's `revoke_endorsement` mutated this case mid-batch). Per `feedback_multi_write_handlers_need_transactions.md` and Phase 5a Watch 9 pattern.
2. **Re-check status** — if status is no longer `SponsorLiabilityPending` (e.g. SL-b's revocation handler escaped it between batch-query and per-case-tx), skip silently. Return `Ok(())` from the per-case body.
3. **Lookup sanction action** — `SELECT action, scope FROM sanction WHERE case_id = $case_id ORDER BY id ASC LIMIT 1`. If empty, log `tracing::error!` and skip silently (per §2.1 (c) empty-sanction edge case).
4. **Evaluate escape conditions** — `evaluate_escape_conditions(...)` returns `EscapeStatus`:
   - **Escape: any sponsor revoked since `decided_at`?** Query `surety` for rows matching `target_person_id` with `revoked_at IS NOT NULL AND revoked_at >= decided_at`. If any match, return `Escape{reason: "sponsor_revoked", actor_pseudonym, endorsement_id}`. (The matching surety row's `revoked_by_pseudonym` would be more accurate, but `surety.revoked_at` doesn't carry the actor — derive from joining endorsement and using `endorsement.from_person_id` for the actor pseudonym.)
   - **Escape: restoration completed during window?** Per PRD §6.2 step 3 second bullet — DEFERRED in SL-c (§2.1 (d) test #7 advisor lean: option (ii)). Plan §13 stubs the branch but it never fires in v1-SL-c. The check returns `Fire` and the plan §19 Notes lists the restorative-mechanics-v1 wiring path.
   - Otherwise: `Fire`.
5. **Apply outcome** via `fire_or_escape_case`:
   - **Escape branch**: UPDATE `moderation_case` SET `status = 'SponsorLiabilityEscaped'`, `liability_escape_reason = json!({"version": 1, ...})`. Emit `sponsor_liability_escaped` log entry.
   - **Fire branch**: Call v0 `apply_sponsor_liability(conn, target_person_id, case_id, community_id, action, &mut cache).await?` — this writes the per-sponsor `reputation_event` rows + `sponsor_liability_applied` log entries (and the `sponsor_liability_clamped` entries when applicable). After it returns, UPDATE `moderation_case` SET `status = 'SponsorLiabilityFired'`. Emit `sponsor_liability_fired` log entry (single, summary-shaped: `{"target_pseudonym": ..., "sponsor_count": <usize from apply>, "case_id": ...}`).
6. **Per-case transaction commits or rolls back atomically.** Plan §13 must specify: outer `run_grace_check_batch` MUST NOT propagate inner per-case errors (return `Err`); per-case errors are logged via `warn!` and the iteration continues. Per PRD §6.3 "Per-case failure inside the per-case transaction is logged but doesn't block the batch — next tick retries."

### 2.3 Scope boundary — what is NOT in SL-c

Per PRD §15 phase table:

- **SL-a (shipped):** Schema + 13 seeded keys + 5 entry-kind consts + 3 CaseStatus variants + Issue #24 partial index + backfill. SL-c READS these; does not add to them.
- **SL-b (in flight on PR #119):** `revoke_endorsement` handler + DTO + route + 9 e2e tests. SL-b emits `sponsor_liability_escaped` for the revocation-branch escape; SL-c emits the SAME const for the scheduler-tick escape branch. SL-c does NOT author SL-b's handler.
- **SL-d (next, after SL-c):** `submit_jury_vote` mutation + `apply_sponsor_liability` compute/fire split + `Decided → SponsorLiabilityPending` transition + `sponsor_liability_pending` fire-site. SL-c calls **unsplit** v0 `apply_sponsor_liability` (advisor-locked posture per masthead). SL-c's tests pre-seed `SponsorLiabilityPending` cases via direct DB-write because SL-d's transition isn't shipped yet. SL-c does NOT author SL-d's split or mutation.
- **SL-e (after SL-d):** Lane-wide e2e suite (full revocation-during-window-escapes flow exercising SL-c scheduler + SL-d transition). SL-c's tests pre-seed pending cases directly; SL-e's tests exercise the full lane (end-to-end through SL-d's transition).
- **restorative-mechanics-v1 (separate PRD, not yet drafted):** `restoration_completed` endpoint + restorative escape branch (defendant-initiated; admin-attested). SL-c stubs the `EscapeStatus::Escape{reason: "restoration_completed"}` branch but never fires it in v1-SL-c. Plan §19 Notes documents the wiring path.

**Hard out-of-scope for SL-c** (per PRD §2 OUT + §15 + §6 lane assignment):

- The `apply_sponsor_liability` compute/fire split (SL-d).
- The `submit_jury_vote → SponsorLiabilityPending` transition (SL-d).
- The `revoke_endorsement` handler (SL-b — already in flight).
- The restoration completion endpoint (restorative-mechanics-v1 PRD; not yet drafted).
- Notification UX (out — PRD §13 OQ-V1-SL-03).
- Cross-instance federation of grace-window events (out — v2 per ADR-014).
- Step-up auth for admin-driven scheduler runs (out — v2 reservation per PRD §12.3).
- Any new migration. SL-a shipped the schema; SL-c adds zero migrations. Plan §13 must NOT include a migration task. (Watch §4 #11.)
- Any new ENTRY_KIND_* const. All needed consts shipped in SL-a (`sponsor_liability_escaped` 74 + `sponsor_liability_fired` 75 + `restoration_completed` 68 + the v0 `sponsor_liability_applied` + `sponsor_liability_clamped` per `governance_log.rs`). Plan §13 must NOT add to the registry. (Watch §4 #10.)
- New CaseStatus variants. SL-a shipped 3 (`SponsorLiabilityPending/Fired/Escaped` at `enums.rs:411-432`). SL-c uses them; does not add.
- Any `governance_config` seed beyond what SL-a shipped. The 13 SL-a seeds (10 `liability.*` + 3 `job.grace_check_*`) cover SL-c's needs. SL-c READS `job.grace_check_interval_minutes`, `job.grace_check_batch_size`, `job.grace_check_staleness_alert_multiplier`, `liability.grace_window_maximum_hours`. If the planner discovers a needed knob not in SL-a's seed list, file a `kind: "blocker"` DQ — do not silently add a seed.
- New HTTP endpoints, new DTOs, new routes. SL-c is a server-internal scheduler module + clokwerk wiring. Plan §13 must NOT include `crates/api/api_common/src/governance.rs` edits or `crates/api/routes/src/lib.rs` edits.
- The existing `sponsor_liability.rs` module is **read-only** for SL-c. Plan §13 must NOT edit `crates/api/api/src/governance/sponsor_liability.rs`. The v0 `apply_sponsor_liability` function is called from SL-c via its existing `pub(crate)` visibility (both files live in `crates/api/api/`, so cross-module call inside the same crate works).

### 2.4 Plan file deliverable shape

- **One file:** `.claude/PRPs/plans/v1-sponsor-liability-c.plan.md`.
- **Follow `.claude/PRPs/templates/plan.template.md` 20-section schema literally.** §16a Stories is mandatory (post-spec-kit-adoption).
- **Shape G applies — SL-c is post-Shape-G** (after JM-e + SL-a + SL-b). §15 DoD MUST use the per-workflow shape (workflow path + phase-branch SHA + `conclusion: "success"`). Inline cargo invocations are forbidden in §15.
- **§5 complexity score with breakdown table** per `feedback_complexity_score_pre_split.md`. Pre-estimate range 5-7. The dominant complexity contributors are: (1) the new module file with 4 public functions (~250 lines), (2) the scheduler wiring in `scheduled_tasks.rs` (atomic + guard + 30-line scheduler block), (3) 5-6 e2e tests (each anchor-Edit-friendly per `feedback_junior_worker_e2e_edit_hang.md`). Pre-estimate is **below** the >8 split-or-proceed threshold; planner-side DQ unlikely. If the planner's final tally exceeds 8 unexpectedly (e.g. discovers SL-d hooks needed; sees the sanction-action lookup blowing scope), file split-or-proceed `kind: "blocker"` DQ.
- **§16a Stories** — every story names (a) composing §13 tasks, (b) a checkpoint command in the Shape-G workflow shape, (c) Brief-Scope outputs to verify (file:line + symbol) per `feedback_advisor_watchpoint_specificity.md`. Concept-only watchpoints rejected. Likely 3 stories: (1) "Scheduler module landed + clokwerk wiring + env-var disable + atomic concurrency guard"; (2) "Fire and escape branches transition cases correctly + log entries + ADR-015 pseudonymisation"; (3) "Per-case isolation + staleness check + batch-size config respected".
- **§4 watchpoints** — every entry cites a specific file, table, or `schema.rs`/handler line. No abstract concepts. The 12 SL-c watchpoints (next sub-section) are the seed list.
- **Explicit scope-boundary section in §6** ("Relationship to other v1-SL sub-phases") listing what each adjacent SL phase ships (SL-a/b/d/e) and the cross-PRD relationship with restorative-mechanics-v1. Mirror SL-b plan §6 shape (verified at `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` §6 — read first as canonical exemplar).

**Commit only the plan file.** Do not author Rust code, do not open PRs, do not touch any file under `crates/`, `migrations/`, or `tests/`. Junior finalize pushes the plan-file commit to `junior/sl-c-planning-1`; advisor merges it into `governance-v0` after DoD smoke + watchpoint-specificity gates pass and the user approves.

---

## 3. Required reading (in order, before drafting any plan section)

1. `.claude/agents/planning.md` — your subagent contract; defines model, tools, "Before you start (always)" sequence, plan-content discipline cross-references.
2. `.claude/PRPs/templates/plan.template.md` — canonical Brehon plan template (20 sections incl. §15.6 Shape-G DoD and §16a Stories). Structure is load-bearing.
3. `.claude/PRPs/prds/v1-sponsor-liability.prd.md` — the parent PRD. **Read in full the first time.** Specifically:
   - §1 (vision/goals — Brehon `athgabál` framing; gives motivation).
   - §2 (scope IN/OUT — confirm SL-c is scheduler module + wiring + observability + tests).
   - §3.1 + §3.4 (CaseStatus extensions — confirm SL-a shipped the 3 variants; SL-c reads `SponsorLiabilityPending`, writes `SponsorLiabilityFired` AND `SponsorLiabilityEscaped`).
   - **§6 IS the SL-c spec — read §6.1 scheduler tick + §6.2 batch semantics + §6.3 failure mode/observability + §6.4 configuration in full.**
   - §7 (Restoration Interaction — read to understand why SL-c stubs but doesn't wire the restoration-escape branch in v1; restorative-mechanics-v1 PRD owns the producer).
   - §8.1 (`liability_escape_reason` JSONB schema — load-bearing for SL-c's escape branch; the `version: 1` field is mandatory per OQ-V1-SL-05).
   - §9.1 (`apply_sponsor_liability` split — **READ TO UNDERSTAND WHAT SL-c DOES NOT DO**; the split is SL-d's. SL-c calls unsplit `apply_sponsor_liability` per advisor-locked posture in masthead).
   - §9.4 (helper module signatures — SL-c implements these).
   - §9.5 (scheduler wiring — SL-c implements this).
   - §10 (Defaults Matrix — confirm `job.grace_check_interval_minutes` default 5, `job.grace_check_batch_size` default 100, `job.grace_check_staleness_alert_multiplier` default 2.0 — all already seeded by SL-a).
   - §11 (backwards compat — §11.2 mid-flight cases at v1 deploy time → SL-a backfilled them to `SponsorLiabilityPending` with 24h grace; SL-c's first-ever batch run on a freshly-deployed v1 instance will pick these up).
   - §12 (security — §12.4 threat-model rows incl. scheduler-vs-handler race, scheduler crash leaving cases stuck, defendant-sponsor collusion).
   - §15 (implementation phases — confirms SL-c is row 3; SL-d depends on SL-c).
   - §17 (cross-cutting impact — names ADR-013 enum-exhaustiveness; confirm no new variants in SL-c).
   - §18 (resolutions applied — B4 key-rename table for `liability.*` is authoritative).
4. `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` — read **§6 (Relationship to other v1-SL sub-phases) and §16a Stories**. The §6 table shape is the canonical mirror for SL-c's §6. The §16a Stories shape is the canonical mirror for SL-c's stories.
5. `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — read §10 (file inventory), §13 task list (for shape consistency — SL-c's §13 should pattern-match SL-a's tasks), §15 DoD (Shape G workflow shape SL-c copies).
6. **`crates/routes/src/utils/scheduled_tasks.rs`** — **read the entire file (~700 lines, verified 29614 bytes at brief-write time on governance-v0 HEAD `2a2649d88`).** This is the canonical mirror SL-c's scheduler wiring is shaped on. Specifically:
   - Lines 65-80: `REPUTATION_SNAPSHOT_RUNNING: AtomicBool` + `struct RunningGuard` + `impl Drop for RunningGuard`. SL-c MUST add a sibling `SPONSOR_LIABILITY_GRACE_RUNNING` + `GraceCheckRunningGuard` pair, NOT reuse the existing static (different concurrency domains).
   - Lines 82-93: `AppealWindowExpiryRunningGuard` — second sibling pattern (post-AD-c era). Demonstrates that adding a third guard pair is precedented.
   - Lines 94-115: `pub async fn setup(context: Data<LemmyContext>) -> LemmyResult<()>` signature + pre-block setup. SL-c's new clokwerk block goes inside this function.
   - Lines 100-120: appeal-window-related blocks (10-min, 1-hour, daily). Demonstrates multiple `scheduler.every(...)` blocks with different intervals all live inside one `setup`.
   - **Lines 185-233: the full reputation_snapshot scheduler block** — the canonical mirror for SL-c. Note the body shape: env-var disable check, atomic compare_exchange, guard, `run_snapshot_batch`, optional staleness pass after. SL-c's new block is structurally identical with `BREHON_DISABLE_GRACE_CHECK_JOB`/`SPONSOR_LIABILITY_GRACE_RUNNING`/`GraceCheckRunningGuard`/`run_grace_check_batch`/`check_grace_staleness`.
   - Lines 251-280: appeal-window-expiry block — second exemplar of the same pattern. Compare with reputation_snapshot to see the variation envelope.
7. **`crates/api/api/src/governance/sponsor_liability.rs`** — **read the entire file (358 lines).** SL-c's scheduler calls `apply_sponsor_liability` directly, so understanding its signature, semantics, and side-effects is mandatory:
   - Line 142: `pub(crate) async fn apply_sponsor_liability(conn, target_person_id, case_id, community_id, action: SanctionAction, cache: &mut ConfigCache) -> LemmyResult<usize>`. Returns count of sponsors processed (== `reputation_event` rows + `sponsor_liability_applied` log entries written).
   - Line 112: `fn severity_for_action(action: SanctionAction) -> LiabilitySeverity` — pure function used inside `apply_sponsor_liability`.
   - Lines 1-50: module doc-comment incl. Watch 10 PII discipline (governance_log payloads carry `*_pseudonym`, never raw `*_id`). SL-c inherits this discipline for its `sponsor_liability_fired` and `sponsor_liability_escaped` payloads.
   - The full body of `apply_sponsor_liability`: understand exactly what it writes (`reputation_event` rows + per-sponsor `sponsor_liability_applied` log entries + `sponsor_liability_clamped` log entries when zero-floor clamp applies). SL-c's fire branch logs `sponsor_liability_fired` AS A SUMMARY entry **on top of** these per-sponsor entries — it does NOT replace them.
8. `crates/api/api/src/governance/reputation_snapshot.rs` — read:
   - Module doc-comment (any patterns SL-c inherits).
   - `pub async fn run_snapshot_batch(context: &LemmyContext) -> LemmyResult<()>` signature + body shape — the canonical mirror for SL-c's `run_grace_check_batch` shape (batch-query + per-row iteration + per-row transaction).
   - Line 423: `pub async fn check_snapshot_staleness(conn, interval_s, now)` — the canonical mirror for SL-c's `check_grace_staleness`.
   - The `recompute_snapshot` calls and `ConfigCache` lifecycle pattern.
9. `crates/api/api/src/governance/governance_log.rs` — confirm:
   - Line 75 `ENTRY_KIND_SPONSOR_LIABILITY_FIRED: &str = "sponsor_liability_fired"`.
   - Line 74 `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED: &str = "sponsor_liability_escaped"`.
   - Line 68 `ENTRY_KIND_RESTORATION_COMPLETED: &str = "restoration_completed"` — SL-c reads this when (eventually) detecting restoration-escape; SL-c does not WRITE it (restorative-mechanics-v1 will).
   - The shim's `pub use` re-export block (verify SL-a wired re-exports — SL-b plan §3 confirms this; if the re-exports are missing, file a `kind: "blocker"` DQ).
   - The `governance_log::append` function signature + the `scrub_json` invocation site. SL-c's payloads flow through `scrub_json` per ADR-015.
10. `crates/api/api/src/governance/config.rs` — find and read:
   - Line 1031-1054: `DEFAULT_JOB_GRACE_CHECK_INTERVAL_MINUTES`, `DEFAULT_JOB_GRACE_CHECK_BATCH_SIZE`, `DEFAULT_JOB_GRACE_CHECK_STALENESS_ALERT_MULTIPLIER` const declarations.
   - Lines 2648-2672: the seed-row entries for the three `job.grace_check_*` keys (verify defaults: 5 / 100 / 2.0 respectively).
   - The `ConfigCache` API + `Scope::Instance` / `Scope::Community` cascade pattern — SL-c's scheduler reads instance-scoped keys only (no community cascade for `job.*`).
   - `liability.grace_window_maximum_hours` default (PRD §10) — SL-c's staleness check multiplies this by 2.0 (the staleness alert multiplier) to compute the "stuck case" threshold. Confirm the const name + default at planning time.
11. `crates/db_schema_file/src/enums.rs:405-440` — confirm:
   - `CaseStatus::SponsorLiabilityPending` variant (SL-c reads this).
   - `CaseStatus::SponsorLiabilityFired` variant (SL-c writes this — doc-comment at lines 423-426 verified at brief-write time: "Set by SL-c scheduler.").
   - `CaseStatus::SponsorLiabilityEscaped` variant (SL-c writes this — doc-comment at lines 428-432 verified: "Set by SL-b (revoke_endorsement) or SL-c (scheduler escape branch).").
   - **The doc-comments PIN the assignment of who-writes-what across SL-b/c/d.** This is the authoritative source — if a future planner thinks SL-c writes some other state, the doc-comment is the override.
12. `crates/db_schema_file/src/schema.rs` — verify:
   - Line 783: `severity -> CaseSeverity` on `moderation_case`. Confirms severity is queryable directly without joining sanction.
   - Lines 799-800: `grace_expires_at -> Nullable<Timestamptz>` and `liability_escape_reason -> Nullable<Jsonb>` on `moderation_case` (SL-a Task 3 added).
   - The `sanction` table — confirms `action -> SanctionAction` column. Schema entry verified at brief-write time (sanction id, case_id, scope, action, target_*).
   - `case_status -> CaseStatus` enum-typed column (verify the type-name).
13. `crates/db_schema/src/source/moderation_case.rs` (or wherever the Diesel struct lives — confirm path at planning time) — verify:
   - The `ModerationCase` struct has `grace_expires_at: Option<DateTime<Utc>>` AND `liability_escape_reason: Option<serde_json::Value>` AND `decided_at: Option<DateTime<Utc>>` AND `severity: CaseSeverity` AND `status: CaseStatus` AND `target_person_id: Option<PersonId>` AND `community_id: Option<CommunityId>`.
   - The `ModerationCaseUpdateForm` (or equivalent) — SL-c UPDATEs `status` and `liability_escape_reason`; confirm the form has them.
   - The `ModerationCaseInsertForm` — SL-c's tests pre-seed `SponsorLiabilityPending` cases via this form; confirm it accepts the relevant columns.
14. `crates/db_schema/src/source/sanction.rs` (confirm path) — verify the `Sanction` struct + filter shapes. SL-c queries `sanction` for the case's action.
15. `crates/db_schema/src/source/surety.rs` (confirm path) — verify the `Surety` struct + filter shape for `revoked_at IS NOT NULL AND revoked_at >= decided_at` query.
16. `crates/db_schema/src/source/governance/governance_log.rs` (confirm path) — verify the `GovernanceLog` struct + the query shape for "any `restoration_completed` log entry between `decided_at` and now for this `target_pseudonym`". SL-c's restoration-escape branch reads this query (stubbed in v1-SL-c per advisor lean §2.1 (d) test #7 option (ii)).
17. `crates/server/tests/e2e.rs` — locate the file (~10,300 lines pre-SL-b, ~10,500-10,600 post-SL-b). Read:
    - The first 100 lines (test-setup helpers — `setup_e2e_pool`, fixture builders).
    - The most-recent SL-b tests (find via `git log -p --follow crates/server/tests/e2e.rs | head -200` after SL-b merges; if SL-b is still in flight at SL-c plan-time, find SL-a tests instead). SL-c's tests anchor at file-end; verify the pattern.
    - The orphan-case `ModerationCaseInsertForm` pattern — SL-c's tests must NOT exercise that pattern; tests pre-seed `SponsorLiabilityPending` cases with full target_person_id + sanction.
    - Existing endorsement-test helpers (grep for `create_endorsement` test calls; SL-c tests reuse them for surety/endorsement seeding).
    - Existing scheduler-driven test patterns (grep for `BREHON_DISABLE_*`; the snapshot-job tests are the closest mirror — find them and pattern-match the test-driven batch-run shape).
18. **Glob `.claude/lessons/feedback_*.md` and Read every file whose name keyword matches:** `multi_write_handlers`, `transactions`, `for_update`, `scheduler`, `clokwerk`, `concurrency`, `atomic`, `running_guard`, `idempotency`, `recompute_snapshot`, `pseudonym`, `pii`, `gdpr`, `redaction`, `scrub`, `actor_pseudonym`, `e2e_filter`, `e2e_edit_hang`, `junior_worker_e2e`, `complexity_score`, `pre_queue`, `watchpoint`, `dod`, `validate`, `shape_g`, `validate_pending`, `governance_log`, `entry_kind`, `seed`, `parametric`, `dual_file`, `re_export`, `cargo`, `features-full`, `features_full_p_crate`, `wrapper-script`, `clippy`, `test-style`, `pre_phase_dod`, `dry_run`, `wrapper_silence`, `principles_not_rules`, `read_canonical`, `parallel_cohort`, `cohort_yaml`, `four_role`, `retro_not_report`, `insertform`, `propagation`, `micros`, `micros_scaled`, `branch_switch`, `commit_aggressively`, `daemon_finalize`, `pre_phase_harness_audit`, `staleness`, `observability`, `tracing_error`, `pre_phase_dod_smoke_test`, `multi_sponsor`, `escape_rule`, `restoration`, `version_tagged_json`. That is the lessons-corpus discipline per `planning.md` step 3.
19. **Glob `.claude/lessons/reference_*.md` and Read every file whose name keyword matches:** `governance_log_entry_kind_registry`, `phase_branch`, `branch_manager`, `worktree`.
20. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — read **ADR-005** (multi-dimensional reputation; SL-c's fire branch writes per-dimension events via `apply_sponsor_liability`), **ADR-008** (append-only signed log; every SL-c emit goes through `governance_log::append`), **ADR-010** (won't-disadvantage rule; the v1 mid-flight backfill SL-c will first see), **ADR-013** (CaseStatus enum-exhaustiveness — no `_ =>` arms in SL-c's matches), **ADR-014** (federation deferral — SL-c emits log events on local instance only; do NOT outbox-emit), **ADR-015** (pseudonymisation; **load-bearing for SL-c** — every `sponsor_liability_fired` and `sponsor_liability_escaped` payload field that names a person uses pseudonym, not raw id). OQ-V1-SL-05 (`liability_escape_reason` schema versioning — `version: 1` from day one).
21. `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` — the most recent post-Shape-G plan with anchor-Edit-per-test discipline. Read §15 per-workflow DoD shape, §16a Stories shape, §5 complexity score breakdown table, the `[P]` cohort markers + FILES YAML blocks in §13.
22. `.claude/PRPs/plans/phase-5b-sponsor-liability-and-founder-bootstrap.plan.md` — the v0 sponsor-liability foundation. Read §13 to see the v0 `apply_sponsor_liability` task shape; SL-c's scheduler call site reads from this plan's tasks 56-58 to understand `apply_sponsor_liability`'s contract.
23. `.claude/PRPs/briefs/sl-a-planning-1.md` and `.claude/PRPs/briefs/sl-b-planning-1.md` — exemplar planning briefs for shape and constraint language. **SL-b is the most-recent SL planning brief — its §4 watchpoints, §3 required-reading style, and §5 lean/observation block are the canonical mirror SL-c's brief is shaped on.**
24. `.github/workflows/cargo-validate-workspace.yml` + `.github/workflows/cargo-validate-features-full.yml` + `.github/workflows/cargo-test-e2e.yml` — the existing Shape-G workflows the plan §15 references. Verify they carry `--no-deps -- -D warnings` and `--features full` where applicable. SL-c does NOT add new workflows or edit existing ones; if any flag is missing, file a DQ.
25. **DQ #67 resolved (per user 2026-04-27)** — workflow YAML DoD dry-run discipline. Honoured by SL-c §15: do NOT prescribe `act` invocations; do NOT require pre-merge full dry-run.

---

## 4. Constraints (hard rules — violating any of these is a process breach)

### 4.1 Plan-content discipline

- **Shape G DoD shape (forward-only).** §15 MUST use the per-workflow shape per `.claude/PRPs/templates/plan.template.md` §15.6. Inline cargo invocations are forbidden in §15. Per `feedback_schema_changing_spec_retrofit_question.md`. SL-c's two workflows are `cargo-validate-workspace.yml` (workspace-check on `junior/*`) and `cargo-test-e2e.yml` (e2e on `phase-v1-SL-c` after finalize-merge — `workflow_dispatch`-only per PR #105).
- **§16a Stories mandatory** (NOT optional). Every story names composing §13 tasks, a Shape-G checkpoint command, Brief-Scope outputs to verify (file:line + symbol), and lists the §13 IMPLEMENT entries it covers. Per `.claude/PRPs/templates/plan.template.md` §16a + `.claude/commands/brehon-verify.md`. Likely 3 stories (see §2.4).
- **§4 watchpoints cite specific files / handlers / `enums.rs` lines**, never abstract concepts. Per `feedback_advisor_watchpoint_specificity.md`. **The 12 watchpoints below are the seed list SL-c's plan §4 MUST include**, each with its citation gate:
   1. **Per-case `FOR UPDATE`.** Plan §13 task that authors `fire_or_escape_case` cites: "step 1 — re-load `moderation_case` row with `FOR UPDATE` inside the per-case transaction; defends against scheduler-vs-handler race per `feedback_multi_write_handlers_need_transactions.md` and Phase 5a Watch 9 pattern." Without `FOR UPDATE`, SL-b's revoke_endorsement could mutate the case between the batch query and the per-case tx, causing a double-state-write.
   2. **Re-check status inside transaction.** Plan §13 must specify: after `FOR UPDATE`, re-read `case.status`; if no longer `SponsorLiabilityPending`, return `Ok(())` from the per-case body without UPDATE. This is the second half of the race defence: between batch query and per-case tx open, status may have flipped.
   3. **Atomic concurrency guard pattern.** Plan §13 task that wires `scheduled_tasks.rs` MUST cite the canonical mirror lines 71-79 (`REPUTATION_SNAPSHOT_RUNNING` + `RunningGuard`) and produce a sibling `SPONSOR_LIABILITY_GRACE_RUNNING` + `GraceCheckRunningGuard` pair, NOT reuse the existing static. Two schedulers sharing a guard is a different concurrency domain and would deadlock-couple them.
   4. **`liability_escape_reason` JSON schema locked.** Schema: `{"version": 1, "reason": "sponsor_revoked"|"restoration_completed", "actor_pseudonym": "...", "endorsement_id"|"restoration_id": ...}`. `actor_pseudonym` mandatory per ADR-015 (GDPR pseudonymisation); `version: 1` mandatory per OQ-V1-SL-05. Plan §13 must cite `actor_pseudonym_helper::get_or_create` as the source. Raw `caller_id` / raw `from_person_id` in reason JSON is a GDPR-013 violation, catch-fire.
   5. **Two log entries per fire path, one per escape path.** Fire path emits BOTH the per-sponsor `sponsor_liability_applied` entries (from v0 `apply_sponsor_liability`) AND the SL-c summary `sponsor_liability_fired` entry. Escape path emits ONLY the `sponsor_liability_escaped` entry (no `apply_sponsor_liability` invoked). Plan §13 must specify exactly which entries fire on which branch; plan §14 testing strategy must assert governance_log row counts per branch.
   6. **Sanction-action lookup correctness.** Plan §13 task that authors `fire_or_escape_case` cites: "query `SELECT action, scope FROM sanction WHERE case_id = $case_id ORDER BY id ASC LIMIT 1`; if zero rows, log `tracing::error!` and skip case silently (don't UPDATE status; don't crash batch)." Empty-sanction edge case is a v0 invariant violation but SL-c MUST handle it gracefully.
   7. **No new `moderation_case.status` mutations OUTSIDE the batch loop.** SL-c writes `status` ONLY inside `fire_or_escape_case`. MUST NOT mutate `Decided` cases, `Open` cases, or any other state. The orphan-case ModerationCaseInsertForm pattern at `e2e.rs` is the cautionary anchor — SL-c's tests pre-seed `SponsorLiabilityPending` cases via `ModerationCaseInsertForm` direct-write but never mutate orphan cases.
   8. **Per-case isolation — outer batch never returns `Err`.** Plan §13 task for `run_grace_check_batch` MUST specify: per-case errors are caught + logged via `warn!`; iteration continues; `run_grace_check_batch` returns `Ok(())` even if N-1 cases fail. Per PRD §6.3. Without this, one bad case (e.g. malformed sanction row) blocks the entire batch indefinitely.
   9. **`BREHON_DISABLE_GRACE_CHECK_JOB` env var precedes the atomic guard.** Plan §13 task for `scheduled_tasks.rs` MUST specify: env-var check is the FIRST thing in the closure body, before `compare_exchange`. Mirror `BREHON_DISABLE_SNAPSHOT_JOB` at `scheduled_tasks.rs:198`. Reversing means tests that set the env var still consume an atomic-bool slot, leaking guards.
   10. **No new ENTRY_KIND_*** — all consts already shipped in SL-a (`sponsor_liability_fired` 75, `sponsor_liability_escaped` 74). Plan §13 must NOT add to the registry. Use existing const names.
   11. **No migration in SL-c.** Plan §13 must produce zero migration files. `git diff governance-v0..phase-v1-SL-c -- migrations/` at plan-approval time must show no output.
   12. **e2e Edit-per-task discipline.** 5-6 SL-c tests = 5-6 individual §13 tasks, each one anchor-pattern Edit at file end. Do NOT bundle. Per `feedback_junior_worker_e2e_edit_hang.md` — `e2e.rs` is now ~10,300+ lines (post-SL-a) and ~10,500-10,600 (post-SL-b); bundle Edits hang Junior workers.
- **§5 complexity score breakdown table mandatory** per `feedback_complexity_score_pre_split.md`. Pre-estimate range 5-7 (well under split threshold). If the planner's final tally exceeds 8 unexpectedly, file split-or-proceed `kind: "blocker"` DQ before finalising §13.
- **e2e edit discipline** per `feedback_junior_worker_e2e_edit_hang.md`. ALSO per `feedback_e2e_filter_assumes_naming.md`: test-name-substring filtering against cargo test will silently match zero tests if SL-c's test names don't share a slug with any prior phase. Plan §15.7 (if it includes manual validation snippets) MUST run full e2e suite (no `--test e2e <filter>`), or confirm the filter via grep before recommending it.
- **Cargo feature/flag propagation** per `pattern_cargo_feature_flag_propagation.md`. Never combine `-p <crate>` with `--features full` (per `feedback_features_full_p_crate_incompatible.md`); use `--workspace --features full`. Even though §15 is Shape-G workflow-shape, the planner-side DoD smoke test (advisor side, pre-merge) and any §15.7 manual validation snippets respect this.
- **Build only what tests exercise** per PMD #14 / `feedback_build_what_tests_exercise.md`. SL-c is scheduler module + tests; do NOT pre-implement the restoration-escape detection (gated to restorative-mechanics-v1 PRD per §2.1 (d) test #7 advisor lean option (ii)). Drift-stub the `EscapeStatus::Escape{reason: "restoration_completed"}` branch but never let `evaluate_escape_conditions` return it in v1-SL-c.
- **Multi-write handlers transactionality** per `feedback_multi_write_handlers_need_transactions.md` — **load-bearing for SL-c**. Each per-case body runs inside one `run_transaction` closure. The outer `run_grace_check_batch` does NOT open a transaction (it iterates cases, each opening its own).
- **R-rule inheritance from JM-a/b/c/d/e + AD-a + SL-a + SL-b retros** — every R1-R7 from prior retros applies. R6 in particular (clippy `--no-deps -- -D warnings`) — under Shape G this lives in the workflow YAML. R5 (Task 0 enumerates ALL probes explicitly) is load-bearing for SL-c's pre-flight harness audit.
- **Wrapper-script flag silence** per `feedback_wrapper_script_flag_silence.md`. Under Shape G non-binding for §15 DoD; if §15.7 manual-validation snippets are included, they MUST cite Linux `.sh` wrappers and verify wrapper `$@` passthrough.
- **Pseudonym discipline** per ADR-015 + Watch 10 from `sponsor_liability.rs:11-15` doc-comment. Every governance-log payload field that names a person uses `*_pseudonym` (string), never raw `*_id`. The pseudonym is sourced via `actor_pseudonym_helper::get_or_create`. Plan §4 watchpoint #4 cites this discipline; impl-task must follow.
- **Read-canonical-mirrors-first.** Plan §10 must cite `crates/api/api/src/governance/reputation_snapshot.rs` (the `run_snapshot_batch` + `check_snapshot_staleness` pair) AND `crates/routes/src/utils/scheduled_tasks.rs:185-233` (the canonical clokwerk block) as canonical mirrors. Deviation from these patterns requires an explicit §10 rationale.

### 4.2 Decision-queue discipline (per `.claude/rules/decision-queue.md`)

- **Attribution integrity.** Any DQ entry seeded by the planning subagent uses `answered_by: "planner"` (forward-looking pre-resolved entries) or `answered_by: null` (genuinely needs advisor input). NEVER `"advisor"`, `"user"`, `"impl-self-resolved"`. Subjects on planner DQ commits MUST start with `chore(decision-queue): planner raised DQ #<id> — <slug>`.
- **Mid-task DQ commits push immediately**, not at finalize, per `decision-queue.md` §"Mid-task visibility (Junior worktrees)". Push to `junior/sl-c-planning-1` (this worktree's branch); advisor's polling loop fetches all branches.
- **Boundary-of-judgment — when to STOP and queue rather than guess:**
  - **Sanction multiplicity.** If at planning time the planner finds that `sanction.case_id` is many-to-one with `moderation_case.id` (multiple sanctions per case, contradicting the v0-shape assumption of 1:1), file a `kind: "blocker"` DQ surfacing the discovered multiplicity and asking advisor whether to (1) iterate, (2) take highest-severity, or (3) take first-by-id. Advisor lean: **(3) first-by-id with rationale** but this is a planner-DQ-eligible question.
  - **Restoration-escape detection scope.** Per §2.1 (d) test #7: advisor lean is option (ii) — stub the `EscapeStatus::Escape{reason: "restoration_completed"}` branch but never let it fire in v1-SL-c. If the planner judges that the stub-vs-wire distinction needs explicit user confirmation (e.g. wants to know if restorative-mechanics-v1 PRD is imminent enough to warrant wiring now), file a `kind: "blocker"` DQ.
  - If the v0 `apply_sponsor_liability` signature has been mutated between brief-write and planning time (e.g. SL-b refactored it for some reason — unlikely per scope rules but possible), planner DQ surfacing the diff.
  - If `governance_log::append` shim re-exports for `ENTRY_KIND_SPONSOR_LIABILITY_FIRED` / `_ESCAPED` are missing at planning time → planner DQ. SL-a Task 7 should have wired them; SL-b plan §3 step 8 confirms; if missing, that's a SL-a regression.
  - If at planning time the canonical mirror `reputation_snapshot::run_snapshot_batch` has been mutated (e.g. AD-related work shifted its shape) → planner DQ surfacing the diff.
  - If at planning time `scheduled_tasks.rs::setup` signature has changed (e.g. new context-threading needed for the third scheduler block) → planner DQ.
  - **DO NOT file a scope-hypothesis DQ.** SL-c's scope is unambiguous from PRD §6 + §15 row 3; this brief is explicit. If the planner finds itself wanting to bundle SL-d (compute/fire split) into SL-c, refuse the temptation. The advisor-locked posture in masthead ("SL-c calls UNSPLIT v0 `apply_sponsor_liability`") is final.
  - **DO NOT file a scope-creep DQ on the staleness check.** PRD §6.3 + the reputation_snapshot mirror at `scheduled_tasks.rs:209-232` lock the shape. If the planner thinks an extra observability surface is needed (e.g. a Prometheus metric, an admin-dashboard panel), file a DQ — do NOT silently extend.

### 4.3 File ownership

- **Touch only:**
  - `.claude/PRPs/plans/v1-sponsor-liability-c.plan.md` (CREATE).
  - `.claude/decision-queue.json` (APPEND new pending or planner-resolved entries).
- **Do NOT edit anything under** `crates/`, `migrations/`, `tests/`, `docs/`, `.claude/PRPs/prds/`, `.claude/PRPs/reports/`, `.claude/agents/`, `.claude/rules/`, `.claude/commands/`, `.github/workflows/`, or any other plan file in `.claude/PRPs/plans/`.
- **One commit at finalize:** `feat(plan): v1-SL-c sub-phase plan` (matching SL-a/SL-b/JM-a/JM-c/JM-d/JM-e + AD-a planner-task commit subjects).

### 4.4 Schema-first discipline

- **Read `crates/db_schema_file/src/schema.rs` BEFORE designing §13.** Confirm at planning time that:
  - `moderation_case` table has `grace_expires_at TIMESTAMPTZ NULL`, `liability_escape_reason JSONB NULL`, `decided_at TIMESTAMPTZ NULL`, `severity case_severity NOT NULL`, `status case_status NOT NULL`, `target_person_id INT4 NULL`, `community_id INT4 NULL` columns (SL-a Task 3 added the grace + escape columns; the rest are baseline).
  - `case_status` enum type lists 12 variants including the 3 SL-a variants (`SponsorLiabilityPending`, `SponsorLiabilityFired`, `SponsorLiabilityEscaped`).
  - `sanction` table has `case_id INT4 NOT NULL`, `action sanction_action NOT NULL`, `scope sanction_scope NOT NULL` columns (Phase 5b baseline).
  - `surety` table has `revoked_at TIMESTAMPTZ NULL` (Phase 5a baseline) + `surety_sponsored_id_active` partial index (SL-a Task 1 — Issue #24).
  - `governance_log` table baseline columns (Phase 5a) for the restoration-escape lookup query.
- **Read `crates/db_schema_file/src/enums.rs:411-432`** to confirm the Rust enum has 12 variants AND the doc-comments explicitly assign `SponsorLiabilityFired` to "SL-c scheduler" and `SponsorLiabilityEscaped` to "SL-b (revoke_endorsement) or SL-c (scheduler escape branch)". If any of these baseline assumptions fails, file a `kind: "blocker"` DQ before writing §13.

### 4.5 Cross-cutting from PMD-promoted patterns

- **Pattern: `multi_write_handlers_need_transactions`** — applies. Each per-case body runs in one `run_transaction` with `FOR UPDATE` re-load + status re-check.
- **Pattern: `verify_before_trusting_shell_output`** — when the planner runs git-grep to enumerate match sites or test setups, verify the count via direct file read; don't trust pipe-counts.
- **Pattern: `cargo_feature_flag_propagation`** — applies to any `--features full` invocation in §15.7 manual snippets. Never combine `-p <crate>` with `--features full`.
- **Pattern: dual-file ENTRY_KIND edit** (per v1-AD-a §10.8 + v1-JM-a §15) — does NOT apply to SL-c. All consts shipped in SL-a; SL-c reads them via the existing shim re-export.
- **Pattern: `read_canonical`** — the canonical-schema-first gate. SL-c's plan §10 cites `reputation_snapshot.rs::run_snapshot_batch` AND `scheduled_tasks.rs:185-233` as canonical mirrors; deviation requires DQ.

### 4.6 Attribution integrity reminder

The only valid `answered_by` labels for a Junior planning subagent are `"planner"` or `null`. Never `"advisor"`, `"user"`, `"impl-self-resolved"`. Any commit with `answered_by: "advisor"` from this subagent triggers the catch-fire procedure in `advisor-orchestrator.md`.

---

**Lean / advisor-side tip (not a constraint):** SL-c is a "single new module + one scheduler block + 5-6 tests" sub-phase — even cleaner than SL-b which has a public HTTP endpoint surface. SL-c is purely server-internal (no DTO, no route, no rate-limit). The two canonical mirrors are exhaustively reviewed: `reputation_snapshot::run_snapshot_batch` ships in production since Phase 5a; `scheduled_tasks.rs::setup` has had three CR cycles across Phase 5a-c. SL-c's module is structurally a third sibling next to `reputation_snapshot.rs` and `appeal_window_expiry.rs` (verify the latter exists at planning time — it should, given the `AppealWindowExpiryRunningGuard` static at `scheduled_tasks.rs:82-93`).

A second observation: the per-case isolation invariant (one bad case doesn't block batch) is the most subtle correctness property. The clean implementation is: `run_grace_check_batch` iterates cases via `for case in batch_query() { match fire_or_escape_case(case).await { Ok(()) => {}, Err(e) => warn!(...) } }`. The outer function never propagates per-case errors. Test #4 (per-case-isolation) is the strong assertion that catches "outer body forgot to swallow inner errors" bugs. Plan §13 task that implements `run_grace_check_batch` must be paired with test #4; both fail together if either is wrong.

A third observation: the env-var override (`BREHON_DISABLE_GRACE_CHECK_JOB=1`) is the only way SL-c's e2e tests can run deterministically. Without it, the cron tick races the test's manual `run_grace_check_batch` invocation. The pattern is verbatim from `BREHON_DISABLE_SNAPSHOT_JOB` (verified at `scheduled_tasks.rs:198`). Plan §14 testing strategy must enumerate which tests rely on this env var (likely all 5-6) and confirm it's set in `setup_e2e_pool` or equivalent. If it's not, file a planner DQ before writing §13.

A fourth observation: SL-c's fire branch emits BOTH the per-sponsor `sponsor_liability_applied` entries (from v0 `apply_sponsor_liability`) AND the SL-c summary `sponsor_liability_fired` entry. This is the layering invariant: v0 entries describe per-sponsor reputation impact; SL-c's summary entry describes the case-level lifecycle event. They co-exist; SL-c does not replace the v0 entries. Test #1 (fire path) MUST assert both kinds are present in governance_log — the per-sponsor count matches active sponsors, plus exactly one summary. Missing the summary is a regression of SL-c's lifecycle observability; missing the per-sponsor entries is a regression of v0's reputation accounting.

A fifth observation: SL-c unblocks SL-d (per PRD §15 row 4 "depends on Phases 1, 3 (SL-a + SL-c)"). SL-d's `submit_jury_vote` mutation creates `SponsorLiabilityPending` cases that SL-c's scheduler then picks up. The producer-consumer separation is clean: SL-c is the consumer-first phase, testable in isolation by manufacturing pending cases. SL-d wires the producer; SL-e tests the full lane end-to-end. SL-c's tests must NOT pre-implement SL-d's transition (that would couple the phases).

A sixth observation: the laptop is running pi/SL-b on `phase-v1-SL-b` at brief-write time. SL-c planning will run on EliteDesk Junior in a worktree branched from `governance-v0`, in parallel with SL-b's impl. Once SL-c's plan ships and SL-b merges to trunk, SL-c can be cut as `phase-v1-SL-c` from post-SL-b governance-v0. Total wall-clock parallel-saving: ~half an SL-b's-worth (SL-c plan-authoring runs concurrent with SL-b's CR cycle + merge-up).

---

_Brief author: advisor session (laptop CWD `/Users/barrie/Developer/lemmy-advisor-sl-c`, sibling worktree on `governance-v0` @ `2a2649d88`, 2026-05-07). Brief committed on `governance-v0` before Junior planning task is queued. Per the advisor-orchestrator clarify gate, advisor runs `/brehon-clarify .claude/PRPs/briefs/sl-c-planning-1.md` after this brief commits and pushes; planning task only queues after every clarify-DQ entry is resolved. Lane progression confirmed 2026-05-07: SL-c scope is sponsor_liability_grace scheduler module + clokwerk wiring + observability + 5-6 e2e tests per PRD §6 + §15 row 3 (NOT "compute/fire split" — that's SL-d). The advisor-locked compute/fire posture is recorded in this brief's masthead (option A: SL-c calls unsplit v0 `apply_sponsor_liability`)._
