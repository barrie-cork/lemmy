# [role:impl-task] sl-c-2-impl-4 — e2e test #4 per-case isolation (task 4)

## 1. Role + dispatch

`[role:impl-task]` — e2e test #4:
`grace_check_per_case_isolation_skips_bad_case_processes_good_case`
inside `mod v1_sl_c_fixtures` in `crates/server/tests/e2e.rs`.

Plan: `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md` §13 Task 4.
Phase branch: `phase-v1-SL-c-2`.
Base: phase branch tip (after Tasks 1+2+fix+3 have been merged).

## 2. Scope

Anchor-insert one test fn inside `mod v1_sl_c_fixtures` (after Task 3's test fn).
The mod's closing `}` still wraps this test.

**Single file edit:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test #4 inside mod v1_sl_c_fixtures
```

### 2.1 CANONICAL CASE OVERRIDE — read before writing any code

**The plan §13 Task 4 IMPLEMENT block prescribes `Result<(), Box<dyn Error>>` as the
return type. This is WRONG and must NOT be followed.**

`mod v1_sl_c_fixtures` was established by Task 1 using Case A (uniform `LemmyResult<()>`
throughout — zero `Box<dyn Error>`, zero `.map_err`). Task 4 appends into the **same
mod**. The canonical-schema-first gate requires: **mirror the existing mod's case verbatim**.

**Mandatory Case A substitution:**
- Test fn signature: `async fn grace_check_per_case_isolation_skips_bad_case_processes_good_case() -> LemmyResult<()>`
- All `?` propagation: bare, no `.map_err` bridges
- Import: `use lemmy_utils::error::LemmyResult;` (already present from Task 1)

**This brief wins over the plan stub.**

### 2.2 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md` §G4 classifier table)

> | Failure signature | Auto-fix | Source lesson |
> | **4b** `error[E0277]: ?` couldn't convert `LemmyError`/`LemmyResult<T>` to `Box<dyn Error>` AND a v1-SL-* / v1-JM-* sibling fixtures module exists in the same file using `LemmyResult<()>` outer | flip the test fn signature to `LemmyResult<()>` AND flip ALL helper signatures in this module's fixtures mod to `LemmyResult<T>`. No `.map_err` bridges. Mirror the canonical sibling shape verbatim (Case A per lesson). | `feedback_lemmy_error_no_std_error.md` Case A |

### 2.3 Implementation

Grep for `grace_check_no_op_when_grace_expires_at_in_future` to locate Task 3's fn.
Insert the following immediately after that fn's closing `}`, still inside
`mod v1_sl_c_fixtures`:

```rust
  #[tokio::test]
  async fn grace_check_per_case_isolation_skips_bad_case_processes_good_case() -> LemmyResult<()> {
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

**Key gotchas (from plan §13 Task 4):**

1. **Watchpoint #8:** the strong assertion is "outer batch returned Ok(...) AND
   iteration continued past case_b". Without per-case error-catch, case_b's
   empty-sanction warn would stop iteration. Test asserts case_a's fire happened —
   proves continuation.

2. **Ordering — case_b first:** batch query sorts `grace_expires_at ASC`, so seed
   case_b's `expired_at` slightly earlier than case_a's (e.g. `now() - 2 minutes`
   vs `now() - 1 minute`).

3. **Silent skip semantic:** case_b stays Pending — admin must intervene. Future ticks
   also skip case_b. This is the design semantic (§10.7 empty-sanction handling).

**MIRROR:** plan §10.7 empty-sanction handling.

### 2.4 Commit message

```
test(v1-SL-c-2): e2e test #4 — per-case isolation skips bad case processes good case (task 4)
```

### 2.5 Shape G post-push

After committing and pushing:

1. Get workflow run id:
   ```bash
   gh run list --repo barrie-cork/lemmy --branch <your-branch> \
     --workflow cargo-validate-workspace.yml --limit 1 --json databaseId
   ```
2. Compute next safe DQ id: `max(all ids across .claude/decision-queue.json +
   archives) + 1`. Do NOT hardcode — earlier tasks may have added entries.
3. Write `kind: "validate-pending"`, `from: "impl"` to `.claude/decision-queue.json`.
4. Commit + push immediately:
   ```
   git add .claude/decision-queue.json
   git commit -m "chore(decision-queue): impl raised DQ #<id> — sl-c-2 task 4 workspace-check validate-pending"
   git push origin HEAD
   ```

## 3. Required reading

1. **`feedback_lemmy_error_no_std_error.md`** — Case A recipe. Plan stub says
   `Box<dyn Error>` — ignore it, use `LemmyResult<()>` per §2.1.
2. **`feedback_junior_worker_e2e_edit_hang.md`** — e2e.rs is 12000+ lines; use
   Grep to locate the anchor fn before editing.
3. **`feedback_clippy_test_style.md`** — no `unwrap()`/`expect()` in test bodies.
4. **`feedback_async_pool_test_pattern.md`** — AsyncPgConnection + DbPool pattern.
5. Grep for `grace_check_no_op_when_grace_expires_at_in_future` before writing —
   confirm it exists and uses `LemmyResult<()>`. Canonical-schema-first gate.

## 4. Constraints

- **File ownership:** edit only `crates/server/tests/e2e.rs` and `.claude/decision-queue.json`.
- **Attribution:** DQ entry `from: "impl"`, `answered_by: null`.
- **DQ mid-task push:** commit + push DQ immediately after writing it.
- **Anchor discipline:** insert AFTER Task 3's fn closing `}`, BEFORE mod closing `}`.
- **Handover trailer:** include `HANDOVER:` YAML in commit body.
