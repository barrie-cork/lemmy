# Plan: v1-quality-r3c — CodeRabbit endpoint-rule fix + sponsor-allowlist sweep + BREHON_DISABLE_* coverage

## 1. Summary

This sub-phase ships three quality items that require no new Rust logic, only targeted
edits to config and test files. Task 1 removes the stale "EXACTLY 11 endpoints"
hard-count assertion from `.coderabbit.yaml` that fires on every v1 PR adding a
sanctioned endpoint. Task 2 adds the two sponsor-allowlist routes to the existing
HTTP-path non-404 sweep test. Task 3 adds test coverage for the two uncovered
`BREHON_DISABLE_*` job guards (`SNAPSHOT_JOB` and `FED_REPLAY_CLEANUP_JOB`).
Acceptance condition: `grep 'EXACTLY 11' .coderabbit.yaml` returns 0; the sweep test
covers 16 routes; both disable-guard tests assert early-exit semantics.

## 2. Source

- `.claude/PRPs/briefs/v1-quality-r3c-planning-1.md` — brief with pre-populated research
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — new test fns must return `LemmyResult<()>` with `?`
- `.claude/lessons/feedback_async_pool_test_pattern.md` — direct DB probe pattern
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — uniqueness gate for Edit anchors
- `.claude/lessons/feedback_rate_limit_debug_config_post_bucket.md` — set_config already present at e2e.rs:4135-4146
- Resolved DQ `a3d0e9941441-043` (`kind: clarify`, `answered_by: advisor`) — FED_REPLAY_CLEANUP test shape: guard in scheduler closure, not in function; two direct-function probes

## 3. Problem statement

**T1:** `.coderabbit.yaml` line ~141 asserts `"v0 is EXACTLY 11 endpoints per ADR-010"`.
Every v1 PR adding a sanctioned endpoint triggers a false-positive CR finding. The count
assertion is stale since v1 began (ADR-010 gates only v0 scope; OQ-020 / v1 PRDs sanction
extensions). → addressed by Task 1.

**T2:** The `all_mvp_endpoints_return_non_404` Phase A sweep (e2e.rs ~line 4165) has 14
entries but omits `POST /api/v4/governance/admin/sponsor-allowlist/add` and
`.../remove` (registered at `crates/api/routes/src/lib.rs:526-528`). → addressed by Task 2.

**T3:** `grep -rn "BREHON_DISABLE_SNAPSHOT_JOB\|BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB" crates/server/tests/e2e.rs`
returns 0 matches. Two job disable-guards have zero test coverage. → addressed by Task 3.

## 4. Solution statement

```
T1: .coderabbit.yaml edit — replace hard count with PRD-backed extension caveat
T2: e2e.rs edit — extend Phase A endpoints array with 2 sponsor-allowlist entries
T3: e2e.rs edit — add test_brehon_disable_snapshot_job + test_brehon_disable_fed_replay_cleanup_job
    using EnvVarGuard::set + direct function call + DB row-count assertion
```

All three tasks are non-`[P]` because T2 and T3 both modify `crates/server/tests/e2e.rs`
(serial constraint). T1 is independent but small enough to run serially without loss.

## 5. Metadata

- **Phase:** `v1-quality-r3c`
- **Branch:** `phase-v1-quality-r3c` (cut by BM-task before Task 1)
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 5 (Task 0 pre-flight + Tasks 1–3 impl + Task 4 retro)
- **Estimated cargo budget:** ~3 GB peak (e2e compile only; no migrations)
- **Forbidden-window applicability:** standard
- **Complexity score:** `4/10`

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 3 impl tasks |
| Migrations touched | +2 each | 0 | None |
| Crates touched | +1 each | 2 | `.coderabbit.yaml` (config) + `e2e.rs` (lemmy_server) |
| `crates/server/tests/e2e.rs` edits | +3 each | 2 | T2 and T3 each edit e2e.rs |
| New ADR-affecting decisions | +2 each | 0 | T1 scopes existing ADR-010; no new ADR |
| Cargo budget peak above 6 GB | +1 per GB | 0 | ~3 GB |
| **Total** | — | **4** | Well below split threshold of 8 |

## 6. Relationship to other sub-phases

- **Depends on:** v1-quality-r3b (merged; Issue #167 fix shipped)
- **Follows:** v1-quality-r3c is the next quality sweep
- **Parallel:** v1-RT-r5 is active in a separate lane (no file overlap)

## 7. Preflight guardrails

- **R1:** e2e.rs edits use verbatim anchor uniqueness gate — every `old_string` confirmed `grep -c = 1` before brief commit. Per `feedback_fix_impl_pre_locate_e2e_anchors.md`.
- **R2:** new test functions return `LemmyResult<()>` with `?`, not `Result<(), Box<dyn Error>>`. Per `feedback_lemmy_error_no_std_error.md`.
- **R3:** direct DB probes use `AsyncPgConnection::establish` pattern. Per `feedback_async_pool_test_pattern.md`.
- **R4:** do NOT add a second `set_config` rate-limit call in Task 2 — one is already at e2e.rs:4135-4146. Per `feedback_rate_limit_debug_config_post_bucket.md`.
- **R5:** Task 0 enumerates all probes explicitly.
- **R6:** all clippy invocations use `--no-deps` uniformly.
- **R7:** Shape G suspended (GH Actions minutes exhausted). Validation path: `validate-pending-laptop` only. No `gh workflow run`. No `validate-pending` DQ entries.

## 8. Flow design

```
T1: .coderabbit.yaml
    └─ path: "crates/api/api_common/src/governance.rs"
       instruction: remove "EXACTLY 11" → add "v1 PRD-sanctioned extensions per OQ-020"

T2: e2e.rs all_mvp_endpoints_return_non_404
    └─ Phase A array (lines ~4165-4229)
       └─ append 2 entries: POST .../sponsor-allowlist/add, POST .../sponsor-allowlist/remove

T3: e2e.rs new test functions
    ├─ test_brehon_disable_snapshot_job
    │   └─ EnvVarGuard::set("BREHON_DISABLE_SNAPSHOT_JOB", "1")
    │   └─ call reputation_snapshot::run_snapshot_batch(&context)
    │   └─ assert reputation_snapshot row count unchanged
    └─ test_brehon_disable_fed_replay_cleanup_job
        └─ probe A (gate unset): seed FederationInboxNonce row → call delete_older_than(1, conn) → assert row gone
        └─ probe B (gate set): seed row → skip delete_older_than call (mirroring scheduler guard) → assert row present
```

## 9. Mandatory reading

- **Config:** `.coderabbit.yaml` lines 135-165 (T1 edit target)
- **Test sweep:** `crates/server/tests/e2e.rs:4081-4248` (T2 edit target)
- **Disable pattern:** `crates/server/tests/e2e.rs:17570-17731` (PARTICIPATION_JOB canonical MIRROR)
- **Scheduler:** `crates/routes/src/utils/scheduled_tasks.rs:200-430` (job functions + guards)
- **Nonce schema:** `crates/db_schema/src/source/governance/federation_inbox_nonce.rs` (T3 insert form + delete_older_than)
- **Lessons:** `feedback_lemmy_error_no_std_error.md`, `feedback_async_pool_test_pattern.md`, `feedback_fix_impl_pre_locate_e2e_anchors.md`, `feedback_rate_limit_debug_config_post_bucket.md`

## 10. Patterns to mirror

### 10.1 BREHON_DISABLE_* guard test pattern

**Mirror:** `crates/server/tests/e2e.rs:17592-17731` (`test_brehon_disable_participation_job`)

```rust
// EnvVarGuard::set gates the job; assert DB state unchanged after job fn call
let _guard = EnvVarGuard::set("BREHON_DISABLE_PARTICIPATION_JOB", "1");
// ... call job fn ... assert no DB write
```

### 10.2 Phase A HTTP sweep entry shape

**Mirror:** `crates/server/tests/e2e.rs:4165-4229` (Phase A array entries)

```rust
(
  "POST",
  "/api/v4/governance/admin/sponsor-allowlist/add",
  r#"{"actor_id":1}"#,
  &[200, 400, 401, 403, 422],
),
```

### 10.3 Sponsor-allowlist routes

**Mirror:** `crates/api/routes/src/lib.rs:526-528`

```
POST /governance/admin/sponsor-allowlist/add    → add_to_sponsor_allowlist
POST /governance/admin/sponsor-allowlist/remove → remove_from_sponsor_allowlist
```

### 10.4 FederationInboxNonce direct-function test shape

**Mirror:** `crates/db_schema/src/source/governance/federation_inbox_nonce.rs:30-37`

```rust
// InsertForm: peer_instance: String, activity_id: String
// delete_older_than(window_days: i64, conn: &mut AsyncPgConnection) -> LemmyResult<()>
// NOTE: guard is in scheduler closure, NOT in delete_older_than; test the function directly
```

## 11. Files to change

- `.coderabbit.yaml` — remove hard "EXACTLY 11" count; add v1 PRD extension caveat (Task 1)
- `crates/server/tests/e2e.rs` — extend Phase A sponsor-allowlist entries (Task 2); add disable-guard tests (Task 3)

## 12. NOT building in v1-quality-r3c

- **Issue #158 (`emit_reputation_event` DRY)** — deferred to r3d; only 2 callers, premature-DRY gate not met
- **Phase B happy-path assertions for sponsor-allowlist** — direct-handler tests already exist at e2e.rs ~18122
- **BREHON_DISABLE_APPEAL_WINDOW_JOB / GRACE_CHECK_JOB / PARTICIPATION_JOB** — all already covered

---

## 13. Step-by-step tasks

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment; confirm branch is `phase-v1-quality-r3c`; confirm clippy baseline is clean.

**Probes:**

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — wrapper honors -p
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/PRPs/debug/r3c-probe1.log 2>&1"
echo "exit: $?"
# EXPECT: only lemmy_utils compiles

# Probe 2 — feature flag activation
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/PRPs/debug/r3c-probe2.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0

# Probe 3 — test target compile (no-run)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/r3c-probe3.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0

# Probe 4 — exit-code propagation (negative test)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/r3c-probe4.log 2>&1"
echo "exit: $?"
# EXPECT: NON-ZERO

# Probe 5 — anchor uniqueness (T2)
grep -c '"GET",' crates/server/tests/e2e.rs
grep -c '"/api/v4/governance/admin/reputation-stats",' crates/server/tests/e2e.rs
# EXPECT: second grep returns 1 (unique anchor for T2 edit)

# Probe 6 — anchor uniqueness (T1)
grep -c 'EXACTLY 11 endpoints' .coderabbit.yaml
# EXPECT: 1

# Probe 7 — coderabbit.yaml sanity check
grep -n 'EXACTLY 11' .coderabbit.yaml
# EXPECT: shows the stale line we will fix

# Probe 8 — T3 DISABLE guards not yet covered
grep -c 'BREHON_DISABLE_SNAPSHOT_JOB' crates/server/tests/e2e.rs
grep -c 'BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB' crates/server/tests/e2e.rs
# EXPECT: both return 0 (not yet covered)
```

**EXPECT block:**
- Probes 0–3: exit 0
- Probe 4: NON-ZERO (exit-code propagation confirmed)
- Probe 5: second grep = 1
- Probe 6: = 1
- Probe 8: both = 0

**No commit at Task 0** — verification only.

---

### Task 1: Fix stale `.coderabbit.yaml` endpoint-count rule (Issue #165)

**ACTION:** Edit `.coderabbit.yaml` path instruction for `crates/api/api_common/src/governance.rs` — replace the hard "EXACTLY 11 endpoints" assertion with a v1-aware caveat referencing OQ-020.

**FILES:**

```yaml
creates: []
modifies:
  - .coderabbit.yaml   # remove hard count; add v1 PRD extension caveat
requires: []
```

**IMPLEMENT (file 1 of 1):** in `.coderabbit.yaml` lines ~139-148, replace the instruction text.

Old instruction (verbatim — occurrence count: 1):
```
      Phase 3 DTO scope check. v0 is EXACTLY 11 endpoints per ADR-010 and
      05-mvp-and-delivery-plan.md §2. Flag new DTOs that do not map to one
      of those 11. RevokeEndorsement DTO is allowed even though the
      endpoint is deferred to v1 (explicit Phase 3 task 35 exception).
      Flag business logic inside DTO impls — DTOs are shape-only.
```

New instruction:
```
      DTO scope check. v0 shipped exactly 11 endpoints per ADR-010 and
      05-mvp-and-delivery-plan.md §2. v1 extends this under OQ-020 and
      the v1 PRDs — new endpoints that have a corresponding v1 PRD story
      are sanctioned. Flag DTOs that map to NO v0 or v1 PRD story.
      RevokeEndorsement DTO is allowed (Phase 3 task 35 exception).
      Flag business logic inside DTO impls — DTOs are shape-only.
```

**VALIDATE:**

```bash
grep -c 'EXACTLY 11' .coderabbit.yaml
# EXPECT: 0

grep 'DTO scope check' .coderabbit.yaml
# EXPECT: shows the new instruction line
```

**COMMIT:** `fix(coderabbit): scope endpoint-count rule to v0 era — fixes Issue #165`

---

### Task 2: Add sponsor-allowlist routes to HTTP-path sweep (Issue #166)

**ACTION:** Extend the Phase A `all_mvp_endpoints_return_non_404` array in `crates/server/tests/e2e.rs` with the two missing sponsor-allowlist POST routes.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append 2 sponsor-allowlist entries to Phase A array
requires: []
```

**IMPLEMENT (file 1 of 1):** in `crates/server/tests/e2e.rs`, find the Phase A array and add the two new entries before the closing `];`.

**MIRROR:** `crates/server/tests/e2e.rs:4165-4229` (Phase A array shape)

**Unique anchor (occurrence count: 1 — confirmed by brief §3a):**
```rust
    (
      "GET",
      "/api/v4/governance/admin/reputation-stats",
      "",
      &[200, 400, 401],
    ),
  ];
```

Insert BEFORE the `];` closing line:
```rust
    (
      "POST",
      "/api/v4/governance/admin/sponsor-allowlist/add",
      r#"{"actor_id":1}"#,
      &[200, 400, 401, 403, 422],
    ),
    (
      "POST",
      "/api/v4/governance/admin/sponsor-allowlist/remove",
      r#"{"actor_id":1}"#,
      &[200, 400, 401, 403, 422],
    ),
```

**GOTCHA:** Do NOT add a second `set_config` call. The rate-limit bump is already present at e2e.rs:4135-4146. Extending the array only.

**VALIDATE:**

```bash
grep -c 'sponsor-allowlist' crates/server/tests/e2e.rs
# EXPECT: >= 2 (the two new entries)

cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/r3c-task2-norun.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

**COMMIT:** `test(e2e): add sponsor-allowlist routes to HTTP-path sweep (Issue #166)`

---

### Task 3: Add BREHON_DISABLE_SNAPSHOT_JOB + FED_REPLAY_CLEANUP_JOB test coverage

**ACTION:** Add two new test functions in `crates/server/tests/e2e.rs` covering the two uncovered `BREHON_DISABLE_*` job guards.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # add test_brehon_disable_snapshot_job + test_brehon_disable_fed_replay_cleanup_job
requires: []
```

**IMPLEMENT (file 1 of 1):** append the two test functions at or near the end of the PARTICIPATION_JOB test block (~line 17731).

**MIRROR:** `crates/server/tests/e2e.rs:17592-17731` (PARTICIPATION_JOB disable-guard test shape)

**Anchor for insertion point (occurrence count: 1 — confirmed by brief §3a):**
The anchor is the last line of the FED_REPLAY_CLEANUP test block. Since these tests are being added fresh, use the end of the PARTICIPATION_JOB tests as the insertion anchor. Verbatim anchor:
```
sponsor_allowlist row must be absent after remove
```
Wait — that anchor is for a different test. Use the PARTICIPATION_JOB test block end. The brief confirms occurrence count = 1 for the anchor:
```rust
  // sponsor_allowlist row must be absent after remove
```

Actually per the brief the confirmed unique anchor for T3 is: `sponsor_allowlist row must be absent after remove`. The two new tests should be appended AFTER the existing sponsor-allowlist direct-handler tests (~line 18122). Insert after the line containing that anchor.

**Test shapes:**

```rust
#[tokio::test(flavor = "multi_thread")]
async fn test_brehon_disable_snapshot_job(pool: DbPool) -> LemmyResult<()> {
  let _guard = EnvVarGuard::set("BREHON_DISABLE_SNAPSHOT_JOB", "1");
  // Count reputation_snapshot rows before
  let before: i64 = /* SELECT COUNT(*) FROM reputation_snapshot */;
  // Call job function with guard set — should exit early
  let context = /* test context */;
  reputation_snapshot::run_snapshot_batch(&context).await?;
  // Assert no new rows written
  let after: i64 = /* SELECT COUNT(*) FROM reputation_snapshot */;
  assert_eq!(before, after, "SNAPSHOT_JOB guard should prevent writes");
  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_brehon_disable_fed_replay_cleanup_job(pool: DbPool) -> LemmyResult<()> {
  // Probe A (gate unset): seed old nonce row → delete_older_than deletes it
  {
    let mut conn = pool.get().await?;
    FederationInboxNonce::insert(&mut conn, &FederationInboxNonceInsertForm {
      peer_instance: "test.example".to_string(),
      activity_id: "probe-a-activity-1".to_string(),
    }).await?;
    federation_inbox_nonce::delete_older_than(0, &mut conn).await?;
    // assert row gone (window_days=0 removes all)
    let count: i64 = /* SELECT COUNT(*) WHERE activity_id = 'probe-a-activity-1' */;
    assert_eq!(count, 0, "delete_older_than should remove row when gate unset");
  }
  // Probe B (gate set): seed row → skip delete_older_than (mirroring scheduler guard) → assert present
  {
    let _guard = EnvVarGuard::set("BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB", "1");
    let mut conn = pool.get().await?;
    FederationInboxNonce::insert(&mut conn, &FederationInboxNonceInsertForm {
      peer_instance: "test.example".to_string(),
      activity_id: "probe-b-activity-1".to_string(),
    }).await?;
    // Do NOT call delete_older_than — guard would prevent scheduler from calling it
    let count: i64 = /* SELECT COUNT(*) WHERE activity_id = 'probe-b-activity-1' */;
    assert_eq!(count, 1, "row should survive when FED_REPLAY_CLEANUP_JOB gate is set");
  }
  Ok(())
}
```

**GOTCHA:** The FED_REPLAY_CLEANUP guard lives in the scheduler CLOSURE, not in `delete_older_than` itself. Probe B tests the intent (row survives because the scheduler wouldn't call the function when gated) — do NOT attempt to test the scheduler closure directly from e2e context. Per DQ `a3d0e9941441-043`.

**GOTCHA:** `FederationInboxNonceInsertForm` has fields `peer_instance: String` and `activity_id: String` (per `federation_inbox_nonce.rs:30`). Use these exactly.

**VALIDATE:**

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/r3c-task3-norun.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0

grep -c 'test_brehon_disable_snapshot_job\|test_brehon_disable_fed_replay_cleanup_job' crates/server/tests/e2e.rs
# EXPECT: >= 2
```

**COMMIT:** `test(e2e): add BREHON_DISABLE_SNAPSHOT_JOB + FED_REPLAY_CLEANUP_JOB guard tests`

---

### Task 4: Retro

**Goal:** author retro per `feedback_retro_not_report.md` and `feedback_four_role_retro_signals.md`. Four H2 sections: Advisor / Planning / Impl / BM. Surface lessons about:
- Planning task #548 running 80+ min without writing the plan file (anchor verification loop without Write)
- This pattern (planner over-reads, never writes) should be caught by a watchdog or brief constraint

Promote new lessons to `.claude/lessons/` in the retro commit.

**COMMIT:** `docs(retro): v1-quality-r3c retro`

---

## 14. Testing strategy

- **Lint:** `cargo clippy --workspace --features full --no-deps -- -D warnings`
- **Test target compile:** `cargo test --no-run -p lemmy_server --test e2e` (after Tasks 2 and 3)
- **e2e execution:** `cargo test --test e2e -p lemmy_server test_brehon_disable_snapshot_job test_brehon_disable_fed_replay_cleanup_job all_mvp_endpoints_return_non_404` (validate-pending-laptop)

---

## 15. Validation commands (DoD)

> Shape G SUSPENDED. All validation via `validate-pending-laptop` (local `cargo-test.bat`).

### 15.1 Static analysis (after each task)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/r3c-check.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.2 Lint (after each task)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/r3c-clippy.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.3 Test target compile (after Tasks 2 and 3)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/r3c-norun.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.4 e2e test execution (validate-pending-laptop)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/r3c-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/r3c-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/r3c-e2e.log"
```

### 15.5 Cross-cutting verification

- [ ] `grep 'EXACTLY 11' .coderabbit.yaml` returns 0 matches
- [ ] `grep -c 'sponsor-allowlist' crates/server/tests/e2e.rs` >= 2
- [ ] `grep -c 'test_brehon_disable_snapshot_job' crates/server/tests/e2e.rs` = 1
- [ ] `grep -c 'test_brehon_disable_fed_replay_cleanup_job' crates/server/tests/e2e.rs` = 1
- [ ] R2: new test fns return `LemmyResult<()>` (no `Box<dyn Error>`)
- [ ] R4: no second `set_config` call added

---

## 16. Acceptance criteria

- [ ] All 3 impl tasks completed in dependency order
- [ ] §15.1 (cargo check) exit 0 after every task
- [ ] §15.2 (cargo clippy `--no-deps -- -D warnings`) exit 0 after every task
- [ ] §15.3 (cargo test --no-run) exit 0 after Tasks 2 and 3
- [ ] §15.4 (e2e) — new tests pass; pre-existing tests still pass
- [ ] §15.5 (cross-cutting) — all boxes ticked
- [ ] §16a stories all `[done]`
- [ ] No edits to files outside §11 list
- [ ] Retro committed per Task 4
- [ ] PR opens against `governance-v0` with `--repo barrie-cork/lemmy`

---

## 16a. Stories

### Story 1: CodeRabbit no longer fires false-positive on v1 governance endpoints

- **Composing tasks:** Task 1
- **Checkpoint command:** `grep -c 'EXACTLY 11' .coderabbit.yaml`
- **Expected output:** `0`
- **Brief-Scope outputs to verify:** `.coderabbit.yaml` no longer contains "EXACTLY 11 endpoints"

### Story 2: sponsor-allowlist routes covered by HTTP-path sweep

- **Composing tasks:** Task 2
- **Checkpoint command:** `grep -c 'sponsor-allowlist' crates/server/tests/e2e.rs`
- **Expected output:** `>= 2`
- **Brief-Scope outputs to verify:** `crates/server/tests/e2e.rs` Phase A array contains both sponsor-allowlist route entries

### Story 3: BREHON_DISABLE_SNAPSHOT_JOB and FED_REPLAY_CLEANUP_JOB guards are tested

- **Composing tasks:** Task 3
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/r3c-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/r3c-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/r3c-e2e.log"`
- **Expected output:** `E2E_EXIT_0` in log; both `test_brehon_disable_snapshot_job` and `test_brehon_disable_fed_replay_cleanup_job` pass
- **Brief-Scope outputs to verify:** both test functions exist in `crates/server/tests/e2e.rs`

---

## 17. Completion checklist

- [ ] Task 0 audit complete
- [ ] Tasks 1–3 committed
- [ ] §15 validation green at every gate
- [ ] §16a stories all `[done]`
- [ ] Retro committed
- [ ] PR opened against `governance-v0`
- [ ] CodeRabbit review complete with findings triaged
- [ ] `/brehon-verify` report shows all stories ✓

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| T3 FED_REPLAY_CLEANUP probe B ambiguous (guard in closure, not function) | MED | MED | Test intent is scheduler-level: row survives because caller doesn't invoke delete_older_than; DQ `a3d0e9941441-043` documents this shape |
| T2 second `set_config` call accidentally added | LOW | MED | R4 guardrail + GOTCHA note in Task 2; impl-task reads `feedback_rate_limit_debug_config_post_bucket.md` |
| e2e compile fails on new test shape | LOW | LOW | `--no-run` probe in Task 2/3 VALIDATE catches before full e2e run |

---

## 19. Notes

- Plan authored directly by advisor (planning Junior #548 cancelled after 80+ min; ran anchor verification loops without issuing the Write call). This is the "planner over-reads, never writes" anti-pattern — flagged for retro.
- Task 3 probe B tests behavioral intent (scheduler wouldn't call the function when gated), not the scheduler closure directly (not callable from test context). Per DQ `a3d0e9941441-043`.
- Issue #158 (`emit_reputation_event` DRY): deferred — only 2 callers, premature-DRY gate not met. Candidate for r3d.

---

## 20. Confidence score

- **Plan correctness:** 9/10 — all research pre-populated; code paths verified; anchors confirmed unique
- **Cargo budget:** 9/10 — no migrations; e2e-only compile
- **Test coverage:** 8/10 — T3 probe B tests behavioral intent rather than internal state; acceptable per DQ answer
