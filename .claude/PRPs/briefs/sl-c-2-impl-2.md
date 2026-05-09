# [role:impl-task] sl-c-2-impl-2 — e2e test #2 escape path (task 2)

## 1. Role + dispatch

`[role:impl-task]` — e2e test #2: `grace_check_escapes_case_when_sponsor_revoked_after_decided_at`
inside `mod v1_sl_c_fixtures` in `crates/server/tests/e2e.rs`.

Plan: `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md` §13 Task 2.
Phase branch: `phase-v1-SL-c-2` (tip `3e1911509`).

## 2. Scope

Anchor-insert one test fn inside `mod v1_sl_c_fixtures` (the mod Task 1
already created). The mod's closing `}` still wraps this test.

**Single file edit:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # insert test #2 after Task 1's test fn, inside mod v1_sl_c_fixtures
```

### 2.1 CANONICAL CASE OVERRIDE — read before writing any code

**The plan §13 Task 2 IMPLEMENT block prescribes `Result<(), Box<dyn Error>>` as the
return type. This is WRONG for Task 2 and must NOT be followed.**

`mod v1_sl_c_fixtures` was established by Task 1 using Case A (uniform `LemmyResult<()>`
throughout — zero `Box<dyn Error>`, zero `.map_err`). Task 2 appends into the **same mod**.
The canonical-schema-first gate requires: **mirror the existing mod's case verbatim**.

**Mandatory Case A substitution:**
- Test fn signature: `async fn grace_check_escapes_case_when_sponsor_revoked_after_decided_at() -> LemmyResult<()>`
- Every helper in this fn: `-> LemmyResult<T>` if extracted (prefer inline — no new helpers needed for this test)
- All `?` propagation: bare, no `.map_err` bridges
- Import: `use lemmy_utils::error::LemmyResult;` (already present from Task 1)

Evidence: Task 1's test fn at the anchor site uses `LemmyResult<()>`. The v1-SL-b sibling
module at `e2e.rs:11001-11924` is the canonical reference — same shape.

This override is required by `feedback_lemmy_error_no_std_error.md` §"How to apply" rule 1
and `advisor-orchestrator.md §2.4` canonical-schema-first gate. If any ambiguity arises
between the plan stub and this override, **this brief wins**.

### 2.2 Implementation

After reading Task 1's test fn (anchor site) to confirm the mod structure, insert the
following immediately after Task 1's `Ok(())` + closing `}` of that fn, still inside
`mod v1_sl_c_fixtures`:

```rust
  #[tokio::test]
  async fn grace_check_escapes_case_when_sponsor_revoked_after_decided_at() -> LemmyResult<()> {
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

**Key gotchas (from plan §13 Task 2):**

1. **Watchpoint #4 — ADR-015 actor_pseudonym:** `actor_pseudonym` field is a STRING
   (the pseudonym), NOT a number. Assert:
   `assert_ne!(json["actor_pseudonym"].as_str().unwrap(), &format!("{}", sponsor_id.0))`.

2. **Watchpoint #5 — only escape entry, no fire-side entries:** escape branch does NOT
   call `apply_sponsor_liability`. Assert count of `sponsor_liability_applied` rows == 0.

3. **endorsement seeding for ref_id:** seed an `endorsement` row from sponsor → sponsee
   BEFORE seeding the surety so `evaluate_escape_conditions`'s endorsement-id lookup
   finds a value.

4. **revoked_at semantic:** `surety.revoked_at = now() - 1h` is AFTER
   `decided_at = now() - 24h` AND BEFORE `now()` — both bounds required by the filter.

**MIRROR refs:** plan §10.6 escape JSONB schema; §4 watchpoint #4 and #5.

### 2.3 Commit message

```
test(v1-SL-c-2): e2e test #2 — escape path sponsor revoked after decided_at (task 2)
```

### 2.4 Shape G post-push

After committing and pushing to the Junior worktree branch:

1. Capture the workflow run id:
   ```bash
   gh run list --repo barrie-cork/lemmy --branch <your-branch> \
     --workflow cargo-validate-workspace.yml --limit 1 --json databaseId
   ```
2. Compute next DQ id by checking BOTH `.claude/decision-queue.json`
   AND `.claude/decision-queue-archive-*.json` (if any) for `max(all ids) + 1`.
   Current max on governance-v0 = **168**. Next safe id = **169**.
3. Write `kind: "validate-pending"`, `from: "impl"`, `id: 169` to
   `.claude/decision-queue.json` on your worktree branch.
4. Commit and push immediately:
   ```
   git add .claude/decision-queue.json
   git commit -m "chore(decision-queue): impl raised DQ #169 — sl-c-2 task 2 workspace-check validate-pending"
   git push origin HEAD
   ```

**Never** compute next_id from only your local worktree's DQ view — other branches
may have added entries. Fetch `governance-v0` and check `max(all ids)` across both
live file and archives before writing.

## 3. Required reading

Before writing any code:

1. **`feedback_lemmy_error_no_std_error.md`** — Case A vs B vs C enumeration. Task 2
   MUST use Case A (same mod as Task 1). Plan stub says `Box<dyn Error>` — ignore it,
   use `LemmyResult<()>` per §2.1 above.
2. **`feedback_async_pool_test_pattern.md`** — AsyncPgConnection + DbPool pool pattern
   for e2e test setup.
3. **`feedback_junior_worker_e2e_edit_hang.md`** — e2e.rs is 11000+ lines; DO NOT
   use a single Edit that re-writes large sections. Use anchored single-insertion Edit
   targeting the exact line after Task 1's test fn's closing `}`. If the file is too
   large to locate the anchor via Read, use Grep to find the anchor string first.
4. **`feedback_clippy_test_style.md`** — no `unwrap()`/`expect()` in test bodies;
   use `?` propagation throughout (works cleanly with Case A `LemmyResult<()>`).
5. Read **`crates/server/tests/e2e.rs`** at the `mod v1_sl_c_fixtures` anchor (Grep
   for `mod v1_sl_c_fixtures` first, then Read ±50 lines around it). Confirm Task 1's
   test fn signature is `LemmyResult<()>` before writing Task 2. This is the
   canonical-schema-first gate check.
6. Read **`crates/server/tests/e2e.rs`** §10.6 MIRROR refs: the escape-path JSONB
   shape (search for `liability_escape_reason` and `sponsor_liability_escaped` in the
   handler code to confirm field names before asserting them in the test).

## 4. Constraints

- **File ownership:** edit only `crates/server/tests/e2e.rs` and
  `.claude/decision-queue.json`. No other files.
- **Attribution:** DQ entry `from: "impl"`, `answered_by: null`. Never `answered_by: "advisor"`.
- **DQ mid-task push:** commit + push DQ entry immediately after writing it (per §2.4).
- **No new helpers:** Task 2's test body should be self-contained (inline setup). If a
  helper is genuinely needed, it must also use `LemmyResult<T>` (Case A).
- **Anchor discipline:** insert AFTER Task 1's fn closing `}`, BEFORE the mod's closing
  `}`. Do not disturb Task 1's fn body.
- **Handover trailer:** include a `HANDOVER:` YAML block in the commit message body with
  `filesCreated`, `filesModified`, `keyDecisions`, and `notes`.

## 5. Prior cohort handover

Task 1 replan established `mod v1_sl_c_fixtures` in `crates/server/tests/e2e.rs` using
Case A (`LemmyResult<()>` throughout). Merged to `phase-v1-SL-c-2` tip `3e1911509`.
Phase 1 workspace-check DQ #167 `result=pass` (workflow 25605783542). Phase 2 e2e
DQ #168 `result=pass` (76 passed, 0 failed, E2E_EXIT_0 via cargo-test.bat --workspace).

No HANDOVER YAML trailer found on the replan worker branch commit (degraded handover —
not a catch-fire). Key facts from auto-state:
- Task 1 added `mod v1_sl_c_fixtures` with test fn
  `grace_check_fires_when_grace_expires_surety_active -> LemmyResult<()>`
- The mod closing `}` is the anchor for Task 2's insertion
- `crates/server/tests/e2e.rs` is approximately 11200 lines post-Task-1
