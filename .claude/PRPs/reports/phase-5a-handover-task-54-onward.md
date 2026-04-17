# Phase 5a handover — tasks 54–56 for fresh impl session

**Purpose.** Compact handover for a new session to complete Phase 5a without re-reading the 1865-line plan cold. Tasks 0, 50, 51, 52, 53 are done on `phase-5a`. Tasks 54, 55, 56 remain.

---

## §1 Current git state

- **Branch:** `phase-5a`
- **Last commit:** `8549f0e7d` `feat(governance): task 53 — reputation snapshot calculator with expires_at filter, decay half-life guard, capability_changed log emit, FOR UPDATE concurrency guard`
- **Commits ahead of `governance-v0`:** 8 (will be 9 after this handover commit)
- **Working tree:** clean after task 53 (verify via `git status --short`)
- **Stashed work:** none

---

## §2 What's done, what's left

### Done
- Task 0 — pre-phase audit (3 wrapper probes + 3 DoD dry-runs) + plan-drift fix + pagination carry-patch
- Task 50 — `governance_config` table + reader (`crates/api/api/src/governance/config.rs`) + 34 seed rows + structural parity tests + DB round-trip (`config_parity_round_trip` in e2e) + `reputation_snapshot.can_sponsor` column + `threshold_score` micros rescale + `ReputationSnapshot*` struct updates
- Task 51 — `person.membership_state` column + `MembershipState` enum (lowercase DB tokens via `DbValueStyle = "snake_case"`) + `parse_membership_state` helper in `config.rs` + `register()` handler patches at both call sites (pre-tx config read per GOTCHA-51b) + federated-upsert default + two grep-guard scripts
- Task 52 — `crates/db_views/reputation` crate with `ReputationSummaryView` + `EndorsementSummaryView` + three tuple-load queries, no `Selectable` derive per view-crate-selectable-template.md
- `chore(carry-patches)` — Person-literal fan-out (3 upstream test fixtures patched with `TODO(brehon-fork): Person::membership_state field added in Phase 5a task 51 — upstream this to LemmyNet/lemmy — PR #___` markers) + 4 lint-guard exclusion amendments
- Task 53 — `reputation_snapshot.rs` (~700 lines including 4 unit tests) with `recompute_snapshot`, `run_snapshot_batch` (chunked per `job.snapshot_batch_chunk_size`), `detect_capability_changes`; `ENTRY_KIND_*` const block (15 entries) prepended to `governance_log.rs`; `mod.rs` wiring

### Plan §15 Acceptance Criteria — green rows

- [x] Pre-5a carry-patch committed on phase-5a
- [x] Task 50 seed contains `job.snapshot_batch_chunk_size = 500`; `DEFAULT_JOB_SNAPSHOT_BATCH_CHUNK_SIZE` const exists; `SEEDED_KEYS_WITH_CONSTS` has 34 entries
- [x] Task 50 parity test round-trips through `ConfigCache::get_<type>()` for every seeded key
- [x] Task 53 `run_snapshot_batch` reads `job.snapshot_batch_chunk_size` at tick-start and chunks `ORDER BY person_id ASC`
- [x] Task 53 GOTCHA-53i rationale present (naive FOR UPDATE) — see `reputation_snapshot.rs` module docstring
- [x] Watch 2 grep marker: `expires_at IS NULL OR expires_at > now()` in `reputation_snapshot.rs`
- [x] Watch 8 grep marker: `if event.expires_at.is_none()` in decay branch
- [x] Watch 9 grep marker: `FOR UPDATE` (via `.for_update()`) on old_snapshot SELECT
- [x] `cargo check --features full --workspace`: exit 0 at task 53 HEAD
- [x] `cargo clippy --features full --workspace --no-deps -- -D warnings`: exit 0 at task 53 HEAD
- [x] `cargo test -p lemmy_server --test e2e`: 9 passed (incl. `config_parity_round_trip`, `phase1_migrations_round_trip`)

### Plan §15 — pending rows

- [ ] Task 54 — `run_snapshot_batch` registered via clokwerk in `scheduled_tasks.rs` + `BREHON_DISABLE_BACKGROUND_JOBS=1` override
- [ ] Task 55 — `POST /api/v4/governance/endorsement` routed; `create_endorsement.rs` handler; `CreateEndorsementResponse` DTO
- [ ] Task 56 — full §14 Levels 0–5 green + `report_to_modlog_golden_path` passes + both lint guards pass + completion report at `.claude/PRPs/reports/phase-5a-complete-report.md` + PR opened with `--repo barrie-cork/lemmy --base governance-v0 --head phase-5a`

---

## §3 Plan-to-implementation deviations landed so far

1. **Carry-patch outcome: `#[expect]` → no-attribute (not `#[allow]`).** Plan §12.0 step 4 said swap `#[expect(clippy::multiple_bound_locations)]` → `#[allow(...)]` at `crates/diesel_utils/src/pagination.rs:220`. Workspace denies `clippy::allow-attributes`, so both `#[expect]` AND `#[allow]` fail. Resolution: removed the attribute entirely (permitted by plan §16 row 1). Scope also grew: same pattern at `crates/db_views/vote/src/impls.rs:135` — patched identically. TODO(brehon-fork) markers on both sites.

2. **`ReputationSnapshot`/`ReputationSnapshotInsertForm` `can_sponsor` field landed in task 50's commit** (not task 53). Per GOTCHA-53f, keeping migration + model atomic is preferred to a task-boundary blur. Advisor confirmed at branch-cut time.

3. **Plan-drift: `--features full` dropped from every `-p lemmy_server` invocation.** `lemmy_server/Cargo.toml` does not declare a `full` feature; cargo rejects. 9 sites fixed in commit `8a5ec091d`. Decision-queue #14 logs the rationale.

4. **Task 53 three-files-one-commit.** `governance_log.rs` const block (task 53 spec step 4 via GOTCHA-53e) + `mod.rs` wiring + `reputation_snapshot.rs` body landed as single commit `8549f0e7d` per "one commit per task" rule — they are trivially-dependent sub-files, not separable work units.

5. **Task 51 Person-literal fan-out landed as separate `chore(carry-patches)` commit** (`2d5032500` between `89879740a` task 52 and `8549f0e7d` task 53). `Person` struct gained required `membership_state: MembershipState` field (per plan §12.2 step 4); 3 upstream test fixtures broke: `crates/db_schema/src/impls/person.rs:467`, `crates/db_views/registration_applications/src/impls.rs:295,373`. Each patched with `membership_state: MembershipState::Member` + TODO(brehon-fork) marker. Lint-guard exclusions amended to cover those paths + `crates/server/tests/e2e.rs` (doc comment mention of migration name) + `crates/db_views/reputation/src/` (doc-comment `can_sponsor` mentions).

6. **`PHASE_1_MIGRATION_COUNT` bumped 6→8** (in task 50 from 6→7 for `add_governance_config`; in task 51 from 7→8 for `add_person_membership_state`). `phase1_migrations_round_trip` test in e2e reverts the contiguous-governance migration stack LIFO; must track with each new migration.

7. **`upsert_snapshot` in task 53 uses branchful SELECT-then-INSERT-or-UPDATE, not `ON CONFLICT DO UPDATE`.** The partial unique index on `reputation_snapshot(person_id) WHERE community_id IS NULL` (task 50) doesn't satisfy Diesel's DSL `on_conflict` target-column check cleanly. The existing `FOR UPDATE` from `read_existing_snapshot_for_update` makes the branchful path race-free per Watch 9. Documented in `reputation_snapshot.rs` comments near the upsert.

8. **Decision-queue entry #13 advisor-answered (admin-config-write.sh wrapper ships as 5c sibling docs, not 5c impl task).** Decision-queue entry #14 impl-self-resolved (plan-drift `--features full` fix). Both land in `resolved` array.

9. **`DbValueStyle = "snake_case"` on `MembershipState` enum** (not `"verbatim"` as other governance enums use). DB stores lowercase tokens `member|provisional|suspended` so `onboarding.default_membership_state` config-text round-trips cleanly. Plan §12.2 SQL sketch line 948 mandated lowercase DB tokens; the enum derive style had to follow.

---

## §4 Decision-queue state

Read `.claude/decision-queue.json` at session start — the following summarises:

### Resolved in this session
- **#13** advisor-answered — `admin-config-write.sh` wrapper: ship as 5c sibling docs/scripts artefact, NOT a 5c impl-plan task.
- **#14** impl-self-resolved — drop `--features full` from `-p lemmy_server` invocations across the plan (9 sites fixed in commit `8a5ec091d`).

### Still pending (do NOT resolve during 5a)
- **#11** — OQ-004 juror cap = 3 (5b task 57 scope).
- **#12** — sponsor-liability severity units `minor=-10|moderate=-50|severe=-200` (5b task 56 scope).
- **#7** impl-self-resolved but kept in `pending` section from earlier sessions; ignore.
- **#8, #9, #10** — all from Phase 4b sessions, impl-self-resolved but stayed in `pending`. Ignore.
- **#4, #5, #6** — Phase 4b advisor answers that stayed in `pending` after being answered. Ignore.

### New entries in this session
None opened by tasks 50–53 beyond #13 and #14 (both resolved). Task 54/55 may surface new ones — append with `status: "advisor-needed"` and continue per `.claude/rules/decision-queue.md`.

---

## §5 Task 54 specification (verbatim from plan §12.5)

**Goal.** Wire `run_snapshot_batch` into Lemmy's existing clokwerk scheduler so snapshots are fresh at jury-assignment time for 5b. Keep `crates/server/src/governance.rs` declarative.

**Steps.**

1. **`crates/routes/src/utils/scheduled_tasks.rs` — add a new scheduler block.** Where: just after the daily scheduler (line 145 region, before the `loop { ... }` at line 148).
   ```rust
   // Brehon governance: reputation snapshot recalculation. Every 15
   // minutes, find users with new events since the last tick (or with
   // founder seeds that just expired) and recompute their snapshot row.
   // Controlled by `job.snapshot_interval_seconds` config at v1; the
   // clokwerk schedule-at-registration time means a config change needs
   // a server restart in v0 (acceptable limitation).
   let context_gov_snapshot = context.reset_request_count();
   scheduler.every(CTimeUnits::minutes(15)).run(move || {
     let context = context_gov_snapshot.reset_request_count();
     async move {
       // Test override: e2e tests set this env var to call run_snapshot_batch
       // directly without the scheduler racing them. S4 from design review.
       if std::env::var("BREHON_DISABLE_BACKGROUND_JOBS").as_deref() == Ok("1") {
         return;
       }
       lemmy_api::governance::reputation_snapshot::run_snapshot_batch(&context)
         .await
         .inspect_err(|e| warn!("Failed to run snapshot batch: {e}"))
         .ok();
     }
   });
   ```

2. **`crates/routes/Cargo.toml` — verify `lemmy_api` is a workspace dep.** It should already be (routes imports from it for route registration). If not, add.

3. **`crates/server/src/governance.rs` — update the stub message.** The file stays ≤30 lines. Change the `info!` message from "not yet scheduled" to "registered via scheduled_tasks (15-minute tick)".
   ```rust
   pub fn schedule_governance_jobs(_context: &LemmyContext) {
     info!(
       "governance: snapshot recalculation job registered via \
        lemmy_api::governance::reputation_snapshot::run_snapshot_batch \
        (15-minute tick in scheduled_tasks::setup; BREHON_DISABLE_BACKGROUND_JOBS=1 disables for e2e)",
     );
   }
   ```

**GOTCHAs (verbatim).**

- **GOTCHA-54a. Advisor narrative drift.** IMPLEMENTATION-PLAN-v0.md §3 Phase 5a task 54 says `tokio::spawn` directly from `crates/server/src/governance.rs`. That conflicts with Lemmy's single-scheduler convention in `crates/routes/src/utils/scheduled_tasks.rs` AND with 03 §11's "server stays declarative" rule. **Resolution**: implement via the clokwerk pattern. Document the deviation in the task 54 commit message and in the phase-5a completion report.
- **GOTCHA-54b. Watch 6.** The `.inspect_err().ok()` pattern is the Lemmy-native way to log-and-continue inside a clokwerk closure (see `scheduled_tasks.rs:74`). The scheduler's outer `loop { scheduler.run_pending().await; tokio::time::sleep(...).await; }` at line 148 ensures ticks continue regardless of closure outcome. Never introduce an inner `loop {}` inside the closure — clokwerk's `run()` wants a single-shot async block.
- **GOTCHA-54c. BREHON_DISABLE_BACKGROUND_JOBS=1** is an env-var check inside the closure, not at registration time. The e2e test must set it BEFORE spawning the server's scheduler (i.e. before `setup()` is called). In practice, `tests/e2e.rs` spawns its own `scheduled_tasks::setup` task OR bypasses it entirely (the existing e2e tests do not spin up the HTTP server; they drive the DB directly). Verify at task 54 implementation time whether any existing test setup calls `setup()` — if not, the env-var is defensive for future e2e tests that do.
- **GOTCHA-54d.** The 15-minute interval is hardcoded in the scheduler registration, but the config key `job.snapshot_interval_seconds = 900` (seeded in task 50) exists for v1. Document in the registration comment that flipping the config to a new value requires a server restart; live-re-schedule is v1.
- **GOTCHA-54e.** `context.reset_request_count()` is the Lemmy-native pattern for passing a `Data<LemmyContext>` into a cron closure without carrying over the request-count state — see `scheduled_tasks.rs:108` (daily block). Do the same here.
- **GOTCHA-54f.** (Chunked batch semantics — already implemented in task 53's `run_snapshot_batch`. Task 54 only registers the scheduler tick; the chunking lives inside `run_snapshot_batch`.)
- **GOTCHA-54g. Observability.** Log the chunk count + dirty-pair total + expired-founder count at each `run_snapshot_batch` invocation using structured `tracing::info!` fields. (Task 53 already emits `pairs_total`, `chunks`, `chunk_size`, `expired_founders` — verify at task 54 close.)

**Validation (task 54).**
```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_routes --features full > .claude/build-task54-routes.log 2>&1"; status=$?; tail -15 .claude/build-task54-routes.log; echo "exit: $status"
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server > .claude/build-task54-server.log 2>&1"; status=$?; tail -15 .claude/build-task54-server.log; echo "exit: $status"
cmd //c "scripts\\brehon\\cargo-check.bat --features full --workspace > .claude/build-task54-ws.log 2>&1"; status=$?; tail -15 .claude/build-task54-ws.log; echo "exit: $status"
```

**Commit.** `feat(governance): task 54 — register snapshot recalc job via scheduled_tasks (15-minute clokwerk tick, BREHON_DISABLE_BACKGROUND_JOBS=1 override)`

**Expected shape:** one commit, ~50–70 lines added (scheduler block + server.rs stub message update + possibly 1-3 lines in `crates/routes/Cargo.toml` if `lemmy_api` dep is missing).

---

## §6 Task 55 specification (verbatim from plan §12.6)

**Goal.** The only new HTTP endpoint in 5a. Config-driven `'age' | 'open' | 'closed'` dispatch; deltas + conditional surety insert; recompute both parties; governance log.

**Steps (shape).**

1. **`crates/api/api_common/src/governance.rs`** — insert `CreateEndorsementResponse` after `CreateEndorsement` at line 193:
   ```rust
   #[skip_serializing_none]
   #[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
   #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
   #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
   pub struct CreateEndorsementResponse {
     pub endorsement_id: EndorsementId,
     pub surety_created: bool,
   }
   ```

2. **`crates/api/api_crud/src/governance/create_endorsement.rs`** — new file. Handler outer wrapper mirrors `admin_assign_jury.rs:42-66` (§8.4). Inner `process_endorsement`:
   - **Step 1.** Read `config::get_text(..., "onboarding.sponsor_gate_strategy")`. Parse via `SponsorGateStrategy::parse(&s)`.
   - **Step 2.** Match strategy — `Closed` returns `LemmyErrorType::NotFound.into()`; `Open` bypasses; `Age` reads `onboarding.sponsor_min_account_age_days`, checks `person.published_at`, rejects if age < min; `Unknown(s)` warns and falls through to `Age` logic inline (no `_ =>` per GOTCHA-55a).
   - **Step 3.** Reject if `data.person_id == sponsor_id` (self-endorse). `Person::read(data.person_id)` errors if target missing.
   - **Step 4.** Max-5 active endorsements from caller: `endorsement WHERE from_person_id = sponsor AND revoked_at IS NULL`.
   - **Step 5.** 48h cooldown: `endorsement WHERE from_person_id = sponsor AND created_at > now() - 48h` (counts revoked rows too — GOTCHA-55d).
   - **Step 6.** Insert endorsement row via `EndorsementInsertForm { from_person_id, to_person_id, community_id }`.
   - **Step 7.** Conditional surety: if `surety WHERE sponsored_id = target AND revoked_at IS NULL` count < 2, insert `SuretyInsertForm { sponsor_id, sponsored_id, community_id }` and set `surety_created = true`.
   - **Step 8.** Two `reputation_event` inserts: `+config.deltas.endorsement_created_sponsor` on `EndorsementStrength` for caller, `+config.deltas.endorsement_created_sponsee` on `ParticipationConsistency` for target. Both with `expires_at: None` (GOTCHA-55h).
   - **Step 9.** Call `reputation_snapshot::recompute_snapshot(conn, sponsor_id, data.community_id, &mut config)` then same for `data.person_id`. Both inside the same `run_transaction` (GOTCHA-55e — FOR UPDATE serialises concurrent inserts).
   - **Step 10.** `governance_log::append` with `ENTRY_KIND_ENDORSEMENT_CREATED`, payload `{ sponsor_pseudonym, target_pseudonym, community_id, gate_strategy, surety_created }`, `actor_pseudonym = Some(sponsor_pseudonym.clone())`.

3. **`crates/api/api_crud/src/governance/mod.rs`** — `pub mod create_endorsement; pub use create_endorsement::create_endorsement;`.

4. **`crates/api/routes/src/lib.rs` line 500 area** — insert `.route("/endorsement", post().to(create_endorsement))` inside the `/governance` scope after `.route("/report", ...)`.

**GOTCHAs (verbatim).**

- **GOTCHA-55a.** The `SponsorGateStrategy` enum uses `Unknown(String)` as an exhaustive-but-final arm rather than `_ =>` — the latter is forbidden by `feedback_clippy_test_style`. `parse()` returns `Unknown(s)` for anything not `"age" | "open" | "closed"`; the match arm logs and falls through to `'age'` behaviour.
- **GOTCHA-55b.** `can_sponsor` is NOT checked by any gate strategy in v0 per OQ-014. The `reputation_snapshot.can_sponsor` column added in task 50 is populated by task 53 but not read here. The `lint-no-can-sponsor-read.sh` script (task 51) verifies this at phase-close.
- **GOTCHA-55c.** `Endorsement` columns are `from_person_id`/`to_person_id`; `surety` columns are `sponsor_id`/`sponsored_id`. Do NOT confuse them. Verify schema at `crates/db_schema_file/src/schema.rs:370` (endorsement) and `:1234` (surety).
- **GOTCHA-55d.** The max-5 check counts `revoked_at IS NULL`; the 48h-cooldown check counts rows regardless of `revoked_at` (a revoked endorsement still counts against the cooldown because the burst of intent happened). Document in the doc comment.
- **GOTCHA-55e.** `recompute_snapshot` inside the tx is safe per Watch 9 (`FOR UPDATE` + branchful upsert). Concurrent endorsements targeting the same sponsee by different sponsors will both call `recompute_snapshot(sponsee_id)` — the FOR UPDATE serialises them, each sees the other's delta before computing. Interleaved `capability_changed` entries in governance_log are acceptable.
- **GOTCHA-55f.** The error type returned by the guards is `LemmyErrorType::NotFound` for v0. A dedicated `LemmyErrorType::EndorsementRejected` is a v1 carry-patch (upstream-held enum); stick with NotFound to keep 5a scope small.
- **GOTCHA-55g.** The `run_transaction` wrapper constructs `context.pool()` outside the tx and passes `conn: &mut AsyncPgConnection` inside. The ConfigCache is per-request (`ConfigCache::new()` at handler entry) and threaded into the tx closure via `mut`. All `config::get_*` calls flow through this cache.
- **GOTCHA-55h.** `reputation_event.expires_at` on the two new events MUST be `None` (permanent organic events — not founder seeds). Default is `Some` per the InsertForm shape, so be explicit: `expires_at: None`.

**Validation (task 55).**
```bash
cmd //c "scripts\\brehon\\cargo-check.bat --features full --workspace > .claude/build-task55-ws.log 2>&1"; status=$?; tail -15 .claude/build-task55-ws.log; echo "exit: $status"
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_routes --features full > .claude/build-task55-routes.log 2>&1"; status=$?; tail -15 .claude/build-task55-routes.log; echo "exit: $status"
```
Note: `-p lemmy_api_crud --features full` may false-red on `user/create.rs` OAuth code per `feedback_api_crud_oauth_feature_quirk.md`. Fall back to `--workspace` if so.

**Commit.** `feat(governance): task 55 — create_endorsement handler with config-driven gate-strategy dispatch (age|open|closed) per OQ-014`

**Expected shape:** one commit, ~200–250 lines (new handler file ~180 lines + DTO addition ~10 lines + mod.rs 2 lines + route registration 1 line + Cargo.lock delta if any).

---

## §7 Task 56 specification (verbatim from plan §12.7)

**Goal.** Prove regression invariants and open the PR.

**Steps.**

1. **Run every §14 validation command** (Levels 0–5). Capture to files; tail only.
2. **Run `report_to_modlog_golden_path`** — single most load-bearing regression guard.
   ```bash
   cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --test e2e -- report_to_modlog_golden_path > .claude/build-task56-golden.log 2>&1"; status=$?; tail -30 .claude/build-task56-golden.log; echo "exit: $status"
   ```
3. **Run both lint guards.**
   ```bash
   bash scripts/brehon/lint-no-membership-read.sh; echo "membership exit: $?"
   bash scripts/brehon/lint-no-can-sponsor-read.sh; echo "can_sponsor exit: $?"
   ```
4. **Write the completion report** at `.claude/PRPs/reports/phase-5a-complete-report.md`. Structure: (a) delivered vs plan; (b) commit list + SHAs; (c) deviations from plan (all §3 entries above PLUS any new task 54 and task 55 deviations); (d) decision-queue entries opened/closed (#13 #14 resolved; #11 #12 untouched); (e) carry-forward into 5b/5c; (f) retro nomination (short form per advisor rule 12).
5. **Open PR.**
   ```bash
   git push -u origin phase-5a
   gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-5a \
     --title "Phase 5a — governance_config, membership_state, reputation infra, create_endorsement" \
     --body "$(cat <<'EOF'
   ## Summary
   - governance_config table + reader (Rust const parity)
   - person.membership_state deferred-enforcement column + grep-guard CI
   - crates/db_views/reputation view crate
   - reputation_snapshot calculator + 15-min background job
   - create_endorsement handler (config-driven age|open|closed gate)

   ## Completion report
   `.claude/PRPs/reports/phase-5a-complete-report.md`

   ## Plan reference
   `.claude/PRPs/plans/phase-5a-config-and-reputation-infrastructure.plan.md`

   ## Regression guard
   - Phase 4 `report_to_modlog_golden_path` passes
   - 9 existing e2e tests pass
   - Zero clippy warnings under `--features full --workspace --no-deps -- -D warnings`
   EOF
   )"
   ```

The `--repo barrie-cork/lemmy` flag is mandatory per `.claude/rules/gh-pr-fork-target.md` — `gh` defaults to upstream `LemmyNet/lemmy` on forks.

**Commit.** `docs(report): Phase 5a complete — governance_config + reputation infra + create_endorsement`

---

## §8 Operational constraints the next session MUST follow

- **Rule 4** (cargo-output-capture.md / no-cargo-output-paste.md) — every cargo invocation → file → `$?` → tail ≤20 lines. Never pipe through tail/head/grep; never paste full logs into conversation.
- **Rule 5** — one commit per task. Task 53's three-files-one-commit pattern is canonical: sub-files that share a single spec row stay atomic.
- **Rule 19** (phase-branch.md / gh-pr-fork-target.md) — PR via `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-5a` at task 56. No direct commits to `governance-v0`.
- **Rule 20** — grep every referenced symbol (config key, function, enum variant, file path) against on-disk state before committing.
- **Seven watchpoints in 5a scope:** config seed/fallback parity (Watch 1, done); `expires_at` cliff filter (Watch 2, done); background-job failure mode (Watch 6 — relevant to task 54); `membership_state` silent-read prevention (Watch 7, grep-guard enforces); decay-vs-cliff double discount (Watch 8, done); snapshot-recompute concurrent race (Watch 9, done via FOR UPDATE + advisory xact lock); admin governance-weight modlog attribution (Watch 11, `admin-config-write.sh` deferred per DQ#13).
- **Decision-queue discipline** (.claude/rules/decision-queue.md) — append `advisor-needed` entries rather than blocking the loop; self-resolve only when plan intent is unambiguous.
- **feedback_background_task_notification_lies** — the `<task-notification>` `exit code` summary can lie. For DoD-critical commands, prefer foreground OR cross-check the log tail for `error:` / `Finished` / `test result:` markers.
- **feedback_clippy_test_style** — no `#[allow(...)]` in tests (workspace denies `clippy::allow-attributes`); tests use `-> Result<(), Box<dyn Error>>` with `?`; no `_ =>` wildcards in exhaustive matches where `feedback_clippy_test_style` applies.
- **feedback_lemmy_error_no_std_error** — `LemmyError` does not implement `std::error::Error`. Tests bridge with `.map_err(|e| -> Box<dyn Error> { format!("{e}").into() })`.
- **feedback_api_crud_oauth_feature_quirk** — `-p lemmy_api_crud` false-reds on `user/create.rs` OAuth reqwest code unless `--features full` propagates. Use `--workspace --features full` as fallback.

---

## §9 Paths the next session MUST read (and ones it MUST NOT)

### Must-read (in this order)
1. **This handover file** (`.claude/PRPs/reports/phase-5a-handover-task-54-onward.md`)
2. **Plan file narrow reads only:**
   - §12.5 (task 54) — already reproduced verbatim in §5 above
   - §12.6 (task 55) — already reproduced in §6 above
   - §12.7 (task 56) — already reproduced in §7 above
   - §14 Levels 0–5 (DoD validation commands) — `Read(offset=1604, limit=120)`
   - §15 Acceptance Criteria — `Read(offset=1730, limit=35)`
3. **`C:\Users\barri\Developer\homeserver\.claude\advisor-context-phase-5.md` §4 + §5 only** — watchpoints and operational rules; skip §1–§3 + §6–§8 (history, not forward-looking).
4. **`.claude/decision-queue.json`** — verify #11/#12 still pending, #13/#14 resolved; check for any new entries.
5. **`git log --oneline origin/governance-v0..HEAD`** — confirm 9 commits (8 feat/chore + this handover).
6. **Existing handler pattern for task 55 outer wrapper:** `crates/api/api/src/governance/admin_assign_jury.rs:42-66` (§8.4 reference).
7. **Existing create_report.rs for task 55 handler file shape:** `crates/api/api_crud/src/governance/create_report.rs`.
8. **Existing clokwerk block for task 54 registration pattern:** `crates/routes/src/utils/scheduled_tasks.rs:67-82` (10-minute tick precedent).

### Must NOT read
- The full plan file end-to-end (tasks 0–53 are done; task bodies are reproduced above).
- `C:\Users\barri\Developer\homeserver\.claude\memory\` (advisor-managed; out of scope for impl).
- `PHASE-5-DESIGN-REVIEW.md` / `PHASE-5-HISTORICAL-FIDELITY.md` (findings baked into plan already).
- Tasks 0–53 commit diffs beyond the one-line `git log` summary.
- Full `reputation_snapshot.rs` (task 53 done; trust tests + clippy). Only grep it if task 55 needs a specific `recompute_snapshot` signature detail.

---

## §10 Session-split signal

**Token count at handover-draft time:** approximately 470–490k of the 1M context window. Well above the 200k effective-reasoning threshold — this session has been compacting iteratively via monitor-driven background validation, but we're past the clean-reasoning zone for high-density new code (task 54 is low-density mechanical, but task 55 is medium-density handler logic and warrants a fresh session).

**A fresh session starts with this handover as the first read.** Reasoning degrades past 200k; this handover exists to keep the next session well inside that bound for tasks 54–56. Start with `Read` on this file, then §5/§6/§7 of this file give the verbatim task specs without the planner narrative. Do NOT re-run `/prp-implement` against the plan — that would start from task 0 semantics. Treat tasks 54/55/56 as the next three commits on `phase-5a`.
