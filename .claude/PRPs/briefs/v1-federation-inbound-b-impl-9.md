---
phase: v1-federation-inbound-b
role: impl-task
task: 9
brief_n: 1
authored: 2026-05-20
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
plan_task: "§13 Task 9 BARRIER — Update Phase-6 fixture + add handler-e2e module"
parent_phase_tip: d2385ae63 (Tasks 4-8 ALL COMPLETE + §15-green; fix chain history fix-impl-4/5/6; DQ #285/286/287/288 all resolved pass; Cohort B + Task 8 closed)
cohort: "BARRIER (not [P]) — Task 9 stands alone post-Task-8. requires: tasks 1, 4, 5, 6, 7, 8 (all merged + §15-green per DQ #285/286/287/288 + Task-1/4 land in the Task-4 chain merged in fix-impl-4 finalize-merge cd9ae510b)."
related_dq: "229 (Shape G suspended), 285/286/287/288 (Cohort B + Task 8 §15 pass), 235 (no .claude/** writes by Junior — harness gap)"
---

# [role:impl-task] v1-federation-inbound-b Task 9 — Phase-6 fixture Allowlist + handler-e2e module (5 tests) — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-9.md

> **Provenance:** Tasks 4-8 complete + §15-green at d2385ae63 (Cohort B Tasks 5/6/7 + Task 8 BARRIER; fix chain fix-impl-4/5/6; DQ #285/286/287/288 all resolved pass). Task 9 is the FINAL impl-task before bm-pr — appends a handler-e2e module (5 test fns asserting wrap_governance_inbound's gate semantics + happy paths) to `crates/server/tests/e2e.rs` per §10.6 + §10.7 + §13 Task 9 IMPLEMENT bodies, and IN-PLACE-Edits the Phase-6 `sanction_notice_round_trip` test to seed an Allowlisted `federation_peer` (else the wrapper's peer-trust gate rejects the round-trip).

## §0 Pre-flight (subagent runs this FIRST, before reading anything else)

- Confirm CWD branch is `phase-v1-federation-inbound-b` (you are on a Junior worktree branched off it). If `git branch --show-current` is anything else → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — inside a forbidden window → exit non-zero `FORBIDDEN_WINDOW: <window>`. Shape G SUSPENDED.
- **Submodule init (MANDATORY before any in-worker cargo — per `feedback_worktree_submodules_not_auto_init`):** `git submodule update --init 2>&1` from worktree root.
- **`requires: tasks 1,4,5,6,7,8` precondition self-verify:** Task 9 exercises every prior task's deliverable. Confirm via grep:
  ```
  grep -q "pub(crate) trait GovernanceInboundActivity" crates/apub/activities/src/governance/inbox.rs &&
  grep -q "pub(crate) async fn wrap_governance_inbound" crates/apub/activities/src/governance/inbox.rs &&
  grep -q "wrap_governance_inbound" crates/apub/activities/src/governance/publish_sanction_notice.rs &&
  grep -q "wrap_governance_inbound" crates/apub/activities/src/governance/publish_trust_attestation.rs &&
  grep -q "wrap_governance_inbound" crates/apub/activities/src/governance/publish_label.rs &&
  grep -q "FED_REPLAY_CLEANUP_RUNNING" crates/routes/src/utils/scheduled_tasks.rs &&
  grep -q "FederationPeerBlocklisted\|FederationPeerRateLimitExceeded\|FederationActivityReplayed" crates/utils/src/error.rs &&
  echo "PRECON-9-OK"
  ```
  If any grep fails → STOP + `kind: "blocker"` (precondition Task not merged).
- **Re-verify e2e.rs line count** per `feedback_junior_worker_e2e_edit_hang.md`:
  ```
  wc -l crates/server/tests/e2e.rs
  ```
  Plan-cited line count was 15482 at HEAD `49a0d6b1d`; current may have drifted slightly. Record the actual line count BEFORE the first Edit — it sets the APPEND anchor for the second Edit.
- Confirm the TWO Edit anchors exist:
  ```
  grep -n "sanction_notice_round_trip\b" crates/server/tests/e2e.rs | head -3
  grep -n "ActivityTrait::verify(&activity" crates/server/tests/e2e.rs | head -3
  grep -n "^mod v1_federation_inbound_a_fixtures" crates/server/tests/e2e.rs
  ```
  Expected: `sanction_notice_round_trip` test ~L5050+; `ActivityTrait::verify(&activity` line ~L5243-5244 (the insertion-BEFORE anchor for Edit 1); `mod v1_federation_inbound_a_fixtures` ~L15093; its closing `}` ~L15151 (the insertion-AFTER anchor for Edit 2). If any anchor missing → STOP + `kind: "blocker"`.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b Task 9 BARRIER — Phase-6 fixture Allowlist + handler-e2e module (5 tests: blocked/rate/replay/happy/label)`

Actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-b Task 9 — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-9.md
```

## §2 Scope

**Produce** (one commit) — **single file, `crates/server/tests/e2e.rs`** (plan §13 Task 9 FILES YAML: `modifies: [crates/server/tests/e2e.rs]`, `creates: []`).

Per plan **§10.6 + §10.7 + §13 Task 9 IMPLEMENT verbatim** — TWO Edits per `feedback_junior_worker_e2e_edit_hang.md`:

### 2.1 Edit 1 — IN-PLACE Allowlist fixture in `sanction_notice_round_trip` (§10.6 verbatim)

INSERT the §10.6 verbatim 10-line `federation_peer` Allowlist block IMMEDIATELY BEFORE the `ActivityTrait::verify(&activity, …)` call (~L5243-5244) inside the `sanction_notice_round_trip` test (~L5050+). The §10.6 block:

```rust
  // v1-federation-inbound-b: Allowlist the test peer so the wrapper's
  // peer-trust gate (Task 4) admits the activity. PRD §5.4 designed
  // breakage — fixture-only, no `_unchecked` variant per PRD §5.4 +
  // §11.4.
  {
    use lemmy_db_schema::source::governance::federation_peer::FederationPeerInsertForm;
    use lemmy_db_schema_file::enums::FederationPeerTrust;
    use lemmy_db_schema_file::schema::{federation_peer, instance};
    let mut async_conn_b_fixture = AsyncPgConnection::establish(&url_b).await?;
    // Look up instance-a.test by domain (created by Phase 6 fixture).
    let peer_instance_id: i32 = instance::table
      .filter(instance::domain.eq("instance-a.test"))
      .select(instance::id)
      .first::<i32>(&mut async_conn_b_fixture)
      .await?;
    let form = FederationPeerInsertForm {
      instance_id: lemmy_db_schema::newtypes::InstanceId(peer_instance_id),
      trust_level: Some(FederationPeerTrust::Allowlisted),
      added_by_actor: None,
      notes: None,
    };
    diesel::insert_into(federation_peer::table)
      .values(&form)
      .execute(&mut async_conn_b_fixture)
      .await?;
  }
```

Anchor on the v0 Step 9 comment block (~L5237-5242) + the `ActivityTrait::verify(...)` line (~L5243). Insert BETWEEN the comment block and the verify call. Pre-existing imports (`AsyncPgConnection`, etc.) are scoped to the test fn — the new fixture block uses local `use` statements as shown.

### 2.2 Edit 2 — APPEND new `mod v1_federation_inbound_b_fixtures` (§10.7 + §13 Task 9 IMPLEMENT verbatim)

APPEND a new module at file end, AFTER the closing `}` of `mod v1_federation_inbound_a_fixtures` (~L15151). The new module contains 5 test fns + 4 helper fns + `use` statements per §10.7 schema + §13 Task 9 IMPLEMENT bodies.

**5 test fns** (each `#[tokio::test(flavor = "multi_thread")]`, each `async fn ... -> LemmyResult<()>`):

1. `blocklisted_peer_returns_403` — asserts wrapper rejects Blocklisted peer; `LemmyErrorType::FederationPeerBlocklisted`; `StatusCode::FORBIDDEN`; `federation_inbox_dropped_log` row with `drop_reason = "blocklisted"`.
2. `per_peer_rate_limit_returns_429` — seeds 2 activities + 3rd hits cap; `LemmyErrorType::FederationPeerRateLimitExceeded`; `StatusCode::TOO_MANY_REQUESTS`.
3. `replayed_activity_returns_409` — pre-seeds nonce row + replay; `LemmyErrorType::FederationActivityReplayed`; `StatusCode::CONFLICT`.
4. `allowlisted_happy_path_persists_advisory_row` — happy path; asserts `remote_sanction_notice` row inserted + ADR-006 `local_case_id IS NULL`.
5. `moderation_label_handler_persists_and_logs` — `PublishLabel` happy path; asserts `remote_moderation_label` row inserted + `governance_log` entry kind `federation_label_received` present.

**4 helper fns** (private, inside the module):

- `bootstrap_with_peer(domain: &str, trust: Option<FederationPeerTrust>) -> LemmyResult<(container, context, db_url, peer_instance_id)>` — bootstraps `governance_fixtures::bootstrap()`, seeds an `instance` row + optional `federation_peer` row. Returns the 4-tuple.
- `build_minimal_sanction_notice_activity(peer_domain: &str) -> LemmyResult<PublishSanctionNotice>` — thin shim, delegates to `build_unique_sanction_notice_activity(peer_domain, 0)`.
- `build_unique_sanction_notice_activity(peer_domain: &str, seq: u32) -> LemmyResult<PublishSanctionNotice>` — thin shim, delegates to `build_sanction_notice_with_id(peer_domain, &format!(...))`.
- `build_sanction_notice_with_id(peer_domain: &str, activity_id: &str) -> LemmyResult<PublishSanctionNotice>` — **REAL IMPL** (plan §13 leaves as `todo!("...")` for impl-task). Mirror Phase-6 `sanction_notice_round_trip`'s activity construction at `crates/server/tests/e2e.rs:5050-5230`. Read those lines verbatim and adapt: replace hard-coded `instance-a.test` with `peer_domain`; replace hard-coded activity ID with `activity_id`; preserve ALL other fields (actor stub, object, subject, ts, etc.).
- `build_minimal_publish_label_activity(peer_domain: &str) -> LemmyResult<PublishLabel>` — **REAL IMPL** (plan §13 leaves as `todo!("...")`). Mirror the structure of `build_sanction_notice_with_id` but for `PublishLabel` (different protocol struct, different first-class fields per `crates/apub/activities/src/protocol/governance/publish_label.rs`). Set actor on `peer_domain`. Minimal valid AP body.

### 2.3 §13 Task 9 IMPLEMENT verbatim (5-test bodies — copy exactly)

**Copy each test fn body byte-for-byte from plan §13 Task 9 IMPLEMENT (lines 2125-2305 in `.claude/PRPs/plans/v1-federation-inbound-b.plan.md`).** The 5 test bodies + the `bootstrap_with_peer` helper are fully specified there — paraphrase is a process miss. Verbatim sections:

- `use super::*;` + all `use` statements (§10.7 lines 1284-1306 + §13 lines 2126-2150 — merge into one `use` block at module top; the schema-table `use` set is the union of both).
- `bootstrap_with_peer` helper (§13 lines 2153-2174).
- Test (a) `blocklisted_peer_returns_403` (§13 lines 2177-2194).
- Test (b) `per_peer_rate_limit_returns_429` (§13 lines 2197-2215).
- Test (c) `replayed_activity_returns_409` (§13 lines 2218-2239).
- Test (d) `allowlisted_happy_path_persists_advisory_row` (§13 lines 2242-2259).
- Test (e) `moderation_label_handler_persists_and_logs` (§13 lines 2262-2283).

The 2 `todo!("implement during impl-task — ...")` helpers (`build_sanction_notice_with_id` + `build_minimal_publish_label_activity`) are **NOT verbatim** — they require real implementation per §2.2 above. Mirror Phase-6's `sanction_notice_round_trip` at `crates/server/tests/e2e.rs:5050-5230` for the structure.

### 2.4 What is NOT in scope (the fence)

- **NEVER touch any file except `crates/server/tests/e2e.rs`.** All preconditions are merged on the phase tip — Task 9 ONLY assembles the test module + the Phase-6 fixture insert.
- **NEVER touch the Phase-6 `sanction_notice_round_trip` test body** other than INSERTING the §10.6 block before the `verify` call (Edit 1). Do NOT refactor, reformat, or remove any existing line.
- **NEVER touch `mod v1_federation_inbound_a_fixtures` (~L15093-15151).** Append the new mod AFTER its closing `}`. The new mod is a sibling, not a child.
- **NEVER add `_unchecked` variants of any seed helper.** PRD §5.4 + §11.4: the Allowlist insert IS the fixture (not a code-shape bypass).
- **NEVER add `#[allow(...)]` or `#[expect(...)]` anywhere.** Per fix-impl-6 retro: workspace `allow_attributes = "deny"`; `#[expect(...)]` only for sanctioned suppressions, and Task 9 doesn't need any.
- **NEVER use `.into()` on `LemmyErrorType` inside an `.ok_or_else()` / `.map_err()` closure.** Per fix-impl-5 retro: that fires E0283. Use bare `?` propagation everywhere (Case A per `feedback_lemmy_error_no_std_error.md`).
- **NEVER declare a `const` mid-fn after let-statements.** Per fix-impl-6 retro: clippy::items-after-statements. Hoist any const to the top of its containing fn.

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #285/286/287/288 (Cohort B + Task 8 §15 pass — establishes the preconditions Tasks 5/6/7/8 merged), DQ #229 (Shape G suspended), DQ #250 (delete_older_than `pub async fn` in lib crate — relevant only as a Task-8 prior-art note, not a Task-9 concern).
2. **Plan §10.6** at `.claude/PRPs/plans/v1-federation-inbound-b.plan.md:1232-1270` — the verbatim Allowlist insert block (the §2.1 contract).
3. **Plan §10.7** at the same file `:1272-1320` — the handler-e2e module skeleton (the §2.2 module-shell contract).
4. **Plan §13 Task 9** at the same file `:2074-2331` — the per-test bodies + helper signatures (the §2.3 contract; verbatim source for the 5 tests + `bootstrap_with_peer`).
5. **`crates/server/tests/e2e.rs:15093-15151`** (READ-ONLY, for the canonical sibling mirror): `mod v1_federation_inbound_a_fixtures` — the fed-in-a Case-A pattern (uniform `LemmyResult<()>` outer; bare `?`; `governance_fixtures::bootstrap()` per test). Your new `mod v1_federation_inbound_b_fixtures` mirrors this shape exactly.
6. **`crates/server/tests/e2e.rs:5050-5230`** (READ-ONLY, for activity-construction mirror): Phase-6 `sanction_notice_round_trip` activity construction — the source you mirror for `build_sanction_notice_with_id` real impl.
7. **`crates/apub/activities/src/protocol/governance/publish_label.rs`** (READ-ONLY, for the PublishLabel struct shape) — to implement `build_minimal_publish_label_activity`.
8. **Lessons** (MANDATORY per `.claude/rules/advisor-orchestrator.md` §2.4 — Task 9 edits the 15k-line `e2e.rs` AND runs `cargo-test --test e2e --no-run`):
   - `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **Why:** ≥2 edits in e2e.rs in a single task; Task 9 has EXACTLY 2 edits (insert + append) which is the safe pattern. ONE in-place edit + ONE append. Do NOT split into more edits.
   - `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Why:** uniform `LemmyResult<()>` outer (Case A); bare `?` propagation; no `.map_err`; no `.into()`-on-LemmyErrorType (E0283 trap, fix-impl-5 retro).
   - `.claude/lessons/feedback_async_pool_test_pattern.md` — **Why:** the tests use `AsyncPgConnection::establish(&db_url).await?` + `DbPool<'_>` via the bootstrap helper. Mirror the fed-in-a fixtures pattern exactly.
   - `.claude/lessons/feedback_clippy_test_style.md` + `.claude/lessons/feedback_clippy_rerun_after_fix.md` — **Why:** clippy denies `unwrap` (use `.unwrap()` only on `Option`/`Result` where the value is impossible-to-fail per the test setup); prefer bare `?`. Re-run clippy after any fix.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — capture-then-tail for §5 commands).

## §3a Handover from prior tasks

> **Handover state: DEGRADED** (per `.claude/rules/advisor-orchestrator.md` §4.3 — no `HANDOVER:` YAML trailer on the Task 4-8 chain). Synthesized by the advisor from the phase tip's public-API deliverable. The §0 grep self-verifies all symbols exist on the phase tip.

```yaml
prior_tasks:
  - tasks: [1, 2, 3]  # Task-4 chain prerequisites (error variants, status-code mapper, governance_log entry kinds, federation_inbox_dropped_log table, federation_peer schema, federation_inbox_nonce schema + delete_older_than)
    notes: "These shipped pre-Cohort-B and are referenced via use statements; you don't author them. LemmyErrorType variants FederationPeerBlocklisted / FederationPayloadTooLarge / FederationSchemaStrictnessViolation / FederationPeerRateLimitExceeded / FederationActorRateLimitExceeded / FederationActivityReplayed all exist. governance_log entry kinds federation_label_received / federation_sanction_received / federation_attestation_received / federation_inbound_blocked / federation_inbound_dropped_oversize / federation_inbound_dropped_replayed / federation_inbound_dropped_rate / federation_inbound_dropped_schema all exist. delete_older_than: pub async fn (DQ #250)."
  - task: 4 (chain): a3757eabe → cdff6f09d → f01a1d44e → 4a60667c9 → fix-impl-4 = 3bd2cfa4e
    filesModified: [inbox.rs]
    keyDecisions: "wrap_governance_inbound + GovernanceInboundActivity trait + helpers; fix-impl-4 added <'a> HRTB. The handler-e2e tests call ActivityTrait::receive(activity, &context).await which delegates through Activity::receive → wrap_governance_inbound → receive_remote_*; both halves are merged."
  - tasks: 5, 6, 7
    commits: [92605e907, ec3b8643f+d32d2c3f2, fe9effd6b]
    filesModified: [publish_sanction_notice.rs, publish_trust_attestation.rs, publish_label.rs]
    keyDecisions: "All 3 publish_*.rs handlers wrapped + trait-impl'd; §15-green. Task 9's 5 tests exercise PublishSanctionNotice (tests a/b/c/d) + PublishLabel (test e). PublishTrustAttestation has no test in Task 9 scope per plan §13 (the per-actor rate gate is a separate concern; the 5 tests cover the wrapper's per-PEER semantics)."
  - task: 8: 36d7bcd03
    filesModified: [scheduled_tasks.rs]
    keyDecisions: "BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB=1 env var disables the cron in tests. Task 9 tests don't set it explicitly — the bootstrap helper / test harness can either set it via std::env::set_var BEFORE bootstrap, OR accept that the cron may tick once during the test run (it's idempotent and benign since the test populates federation_inbox_nonce rows which are within the 7-day window)."
notes: "Task 9 ONLY edits e2e.rs. All 8 preconditions are merged + §15-green on phase tip d2385ae63. The §0 PRECON-9 grep block self-verifies."
```

## §4 Constraints (hard rules)

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-federation-inbound-b` (tip `d2385ae63`, Tasks 4-8 complete + §15-green). Finalize merges your worktree branch back; do NOT push to `phase-v1-federation-inbound-b` directly.
- ONE commit (single file). If §2.4 pre-push validation fails on first attempt, amend/fixup — do NOT split.
- Mid-task DQ visibility: if you raise a `pending` entry mid-task, **commit + push immediately**.
- No `answered_by: "advisor"` or `"user"`. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235)

If you need to write the §5 `validate-pending-laptop` DQ entry and the sensitive-file gate blocks it: write `TASK9_VALIDATE_PENDING.json` + `TASK9_ESCALATION.md` at worktree root, commit both, STOP. Tasks 5/6/7/8 workers raised inline OK; harness-gap likely does NOT fire.

### Task-9 GOTCHAs (load-bearing — Case A + e2e edit discipline + ADR-006)

- **TWO Edits ONLY on e2e.rs** (per `feedback_junior_worker_e2e_edit_hang.md`): ONE in-place Edit (insert) + ONE append. Do NOT split into N small edits. If the first Edit fails because the anchor moved, re-grep with a wider context and retry — do NOT degrade into many small chunks.
- **Case A: uniform `LemmyResult<()>` outer.** Every test fn signature: `async fn name() -> LemmyResult<()>`. Every helper fn signature: `async fn name(...) -> LemmyResult<T>` OR `fn name(...) -> LemmyResult<T>` (the activity-builder helpers are sync). Bare `?` propagation throughout. NO `.map_err`. NO `.into()` on `LemmyErrorType` inside closures (E0283 trap per fix-impl-5).
- **ADR-006 `local_case_id IS NULL`** assertions in tests (d) + (e). The `remote_sanction_notice` / `remote_moderation_label` rows inserted via the receive handlers MUST have `local_case_id = None` per ADR-006 (governance is advisory, not authoritative; local_case_id is only set when an operator promotes the remote signal via the case API). Tests assert this explicitly: `assert!(advisory.local_case_id.is_none(), "ADR-006: local_case_id MUST be NULL");`
- **`build_sanction_notice_with_id` real impl** — mirror Phase-6 e2e.rs:5050-5230 EXACTLY for the activity-construction shape. The only fields that vary by call site: actor URL (uses `peer_domain`), activity `id` (uses `activity_id` argument). All other fields (object, subject, ts, etc.) match Phase-6's fixture.
- **`build_minimal_publish_label_activity` real impl** — read `crates/apub/activities/src/protocol/governance/publish_label.rs` for the struct shape, build a minimal valid AP body with actor on `peer_domain`.
- **`BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB`** — if the test harness's `governance_fixtures::bootstrap()` doesn't already set this env var, you do NOT need to set it explicitly. The cron is benign within the test's 7-day window; replay cleanup won't fire against the test's nonce rows. (If a test does fail mysteriously due to cron interference, surface a `kind: "blocker"` DQ — do not silently add `std::env::set_var(...)` calls without recording the rationale.)
- **Per-peer rate-limit test (test b)** — the plan inserts a `governance_config` row to set `federation.inbound.per_peer_rate_per_hour = 2`. After 2 successful + 1 failure (the 3rd), assertion: `LemmyErrorType::FederationPeerRateLimitExceeded`. The in-memory counter (`rate_per_peer_counts`) is process-local; bootstrap creates a fresh `LemmyContext` per test so the counter starts at 0 each time.
- **Replay test (test c)** — pre-seed `federation_inbox_nonce` directly via diesel insert. The wrapper's nonce-check rejects the second receive of the same activity_id with `FederationActivityReplayed`.
- **Schema imports** — §10.7 + §13 Task 9 use a UNION of schema-table imports. Use the merged set listed in §13 (the wider one: includes `federation_inbox_dropped_log` which §10.7 omits — §13 is authoritative).
- **`LemmyError` import** — §10.7 imports `LemmyErrorType, LemmyResult` only; §13 also imports `LemmyError` (used in test a: `let err: LemmyError = result.err().unwrap();`). Use the merged set.

### Plan-cited line numbers may have drifted

- Edit 1 anchor (~L5243-5244): grep for `ActivityTrait::verify(&activity` inside `sanction_notice_round_trip`. The `wc -l` at task-start is the authoritative file length; Edit 2's append point is END-OF-FILE.
- Edit 2 anchor (~L15151): grep for the closing `}` of `mod v1_federation_inbound_a_fixtures`. Re-verify with: `grep -n "^mod v1_federation_inbound_a_fixtures" -A 2 crates/server/tests/e2e.rs | head -5` + then locate its matching `}` by reading forward.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

**Shape-G suspended until 2026-06-01** (DQ #229). After committing + pushing your worktree branch, write a `kind: "validate-pending-laptop"` DQ entry with `from: "impl"`, `phase_task: 9`, `branch: <your-worktree-branch>`, and `commands[]` set **verbatim** to (per plan §13 Task 9 "Push and exit" — exactly 3 commands, includes `cargo-test --test e2e --no-run` for Task 9):

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task9-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task9-clippy.log 2>&1"
cmd //c "scripts\brehon\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/fed-in-b-task9-test-norun.log 2>&1"
```

Note the THIRD command — `cargo-test --no-run` (R7 checkpoint per plan §14) — confirms the test target compiles and links the e2e binary. Phase-2 e2e EXECUTION is post-merge user-gate-4 per `advisor-orchestrator.md` §3.2 — NOT this brief's scope.

If the sensitive-file gate fires, use the §4 harness-gap path with `TASK9_VALIDATE_PENDING.json`.

## §6 Expected output (return to advisor)

```
## Task 9 complete — v1-federation-inbound-b Phase-6 fixture Allowlist + handler-e2e module

**Commit:** <sha> on <worktree-branch>
**File changed:** crates/server/tests/e2e.rs (2 hunks: 1 IN-PLACE Edit at sanction_notice_round_trip ~L5243; 1 APPEND new mod at file end)
**PRECON-9 self-check:** GovernanceInboundActivity trait + wrap_governance_inbound + all 3 publish_*.rs callers + FED_REPLAY_CLEANUP_RUNNING in scheduled_tasks.rs + all 6 LemmyErrorType variants — all confirmed present at base tip d2385ae63.
**e2e.rs line count at start:** <N> (was 15482 at plan write; current <N>)
**§10.6 Allowlist insert:** verbatim, BEFORE ActivityTrait::verify line
**§10.7 + §13 Task 9 module:** 5 test fns (blocked/rate/replay/happy/label) + bootstrap_with_peer + 4 build_* helpers; build_sanction_notice_with_id + build_minimal_publish_label_activity REAL IMPL mirroring Phase-6 e2e.rs:5050-5230
**Case A discipline:** uniform LemmyResult<()> outer; bare ? throughout; 0 .map_err; 0 .into() on LemmyErrorType
**ADR-006 assertions:** present in tests (d) + (e)
**TWO Edits ONLY:** 1 in-place + 1 append (per feedback_junior_worker_e2e_edit_hang.md)
**No other-file edit:** confirmed
**validate-pending-laptop DQ:** #<id> raised (commands: cargo-check.bat + cargo-clippy.bat -D warnings + cargo-test.bat --test e2e --no-run — 3 cmds for R7)
**Next:** advisor laptop runs §15 (3 cmds), mutates DQ #<id>; on pass → Task 9 complete → §16a Story 5 closed → advance to user-gate-5 (merge confirm) → bm-pr → CR triage → bm-merge → retro.
```

## §7 Why this brief differs from the plan

It does not — Task 9's scope is exactly plan §10.6 + §10.7 + §13 Task 9 IMPLEMENT. This brief adds only: (a) §0 forbidden-window + PRECON-9 grep block (all 6 prior-task symbols) + anchor-grep + `wc -l` re-verify per `feedback_junior_worker_e2e_edit_hang.md`, (b) §2.3 verbatim reference to plan-cited line ranges (the 5 test bodies + bootstrap_with_peer are §13 lines 2125-2305 byte-for-byte; only the 2 `todo!()` helpers need real impl), (c) §4 harness-gap escalation path + the explicit GOTCHAs (Case A discipline; ADR-006 assertions; TWO Edits ONLY; no `.into()` per fix-impl-5; no items-after-statements per fix-impl-6; build_*_activity real impl mirroring Phase-6), (d) §5 explicit `validate-pending-laptop` shape with the 3-command set (Task 9 is the only impl-task in -b with `cargo-test --no-run` — R7 checkpoint), (e) explicit "plan line numbers may have drifted" GOTCHA. The Allowlist insert + module skeleton are §10.6/§10.7/§13 verbatim — do not deviate.

Mirrors the canonical sibling `.claude/PRPs/briefs/v1-federation-inbound-b-impl-8.md` schema per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate (same §0-§7 brief STRUCTURE; same single-file BARRIER discipline as Task 9 is also BARRIER; §13 IMPLEMENT verbatim reproduction).

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`). Brief committed on `governance-v0` first; then cherry-picked / re-committed onto `phase-v1-federation-inbound-b` so the worker — which branches from the PHASE branch tip `d2385ae63` (Tasks 4-8 complete + §15-green) — sees it. `/precheck` re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC before queue. Task 9 BARRIER (not [P]); cohort cap=1 still enforced under user cap-≤2 OOM choice. Handover from Tasks 1-8 is DEGRADED (no HANDOVER: trailer on the chain) — synthesized in §3a from the public-API deliverable on the phase tip. After Task 9 §15-green: pipeline advances to user-gate-5 (merge confirm) → bm-pr → CR triage → bm-merge → retro per advisor-orchestrator.md §3.2._
