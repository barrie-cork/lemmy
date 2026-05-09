# [role:impl-task] sl-c-2-impl-3 — e2e test #3 no-op (future grace_expires_at) (task 3)

## 1. Role + dispatch

`[role:impl-task]` — e2e test #3: `grace_check_no_op_when_grace_expires_at_in_future`
inside `mod v1_sl_c_fixtures` in `crates/server/tests/e2e.rs`.

Plan: `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md` §13 Task 3.
Phase branch: `phase-v1-SL-c-2`.
Base: phase branch tip (after Tasks 1+2+fix have been merged).

## 2. Scope

Anchor-insert one test fn inside `mod v1_sl_c_fixtures` (after Task 2's fix test fn).
The mod's closing `}` still wraps this test.

**Single file edit:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test #3 inside mod v1_sl_c_fixtures
```

### 2.1 CANONICAL CASE OVERRIDE — read before writing any code

**The plan §13 Task 3 IMPLEMENT block prescribes `Result<(), Box<dyn Error>>` as the
return type. This is WRONG and must NOT be followed.**

`mod v1_sl_c_fixtures` was established by Task 1 using Case A (uniform `LemmyResult<()>`
throughout — zero `Box<dyn Error>`, zero `.map_err`). Tasks 2, 3, 4, 5 all append
into the **same mod**. The canonical-schema-first gate requires: **mirror the existing
mod's case verbatim**.

**Mandatory Case A substitution:**
- Test fn signature: `async fn grace_check_no_op_when_grace_expires_at_in_future() -> LemmyResult<()>`
- All `?` propagation: bare, no `.map_err` bridges
- Import: `use lemmy_utils::error::LemmyResult;` (already present from Task 1)

Evidence: Task 1's test fn uses `LemmyResult<()>`. The v1-SL-b sibling module at
`e2e.rs:11001-11924` is the canonical reference — same shape.

This override is required by `feedback_lemmy_error_no_std_error.md` §"How to apply"
rule 1 and `advisor-orchestrator.md §2.4` canonical-schema-first gate. **This brief
wins over the plan stub.**

### 2.2 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md` §G4 classifier table)

> | Failure signature | Auto-fix | Source lesson |
> | **4b** `error[E0277]: ?` couldn't convert `LemmyError`/`LemmyResult<T>` to `Box<dyn Error>` AND a v1-SL-* / v1-JM-* sibling fixtures module exists in the same file using `LemmyResult<()>` outer | flip the test fn signature to `LemmyResult<()>` AND flip ALL helper signatures in this module's fixtures mod to `LemmyResult<T>`. No `.map_err` bridges. Mirror the canonical sibling shape verbatim (Case A per lesson). | `feedback_lemmy_error_no_std_error.md` Case A |

### 2.3 Implementation

Grep for `grace_check_fires_expired_case` to locate the mod anchor. Then find the
closing `}` of the LAST test fn currently inside `mod v1_sl_c_fixtures` (Task 2's
fn or its fix). Insert the following immediately after that fn's closing `}`:

```rust
  #[tokio::test]
  async fn grace_check_no_op_when_grace_expires_at_in_future() -> LemmyResult<()> {
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

**Key gotchas (from plan §13 Task 3):**

1. **Boundary semantic:** batch query uses `.le(Some(now))` — a case with
   `grace_expires_at == now()` exactly would fire; `+2 hours` is comfortably outside.

2. **No spurious side effects:** the no-op assertion is the strong signal —
   zero side-effects across `reputation_event`, `governance_log`, and
   `moderation_case` row mutation.

### 2.4 Commit message

```
test(v1-SL-c-2): e2e test #3 — no-op for future grace_expires_at (task 3)
```

### 2.5 Shape G post-push

After committing and pushing:

1. Get workflow run id:
   ```bash
   gh run list --repo barrie-cork/lemmy --branch <your-branch> \
     --workflow cargo-validate-workspace.yml --limit 1 --json databaseId
   ```
2. Next safe DQ id = **172** (max across governance-v0 live file + archives
   is 171 after Phase-2 e2e DQ #171 resolved). Verify by reading
   `.claude/decision-queue.json` and archives — use `max(all ids) + 1`.
3. Write `kind: "validate-pending"`, `from: "impl"`, `id: 172` to
   `.claude/decision-queue.json`.
4. Commit + push immediately:
   ```
   git add .claude/decision-queue.json
   git commit -m "chore(decision-queue): impl raised DQ #171 — sl-c-2 task 3 workspace-check validate-pending"
   git push origin HEAD
   ```

## 3. Required reading

1. **`feedback_lemmy_error_no_std_error.md`** — Case A recipe. This task is Case A.
   Plan stub says `Box<dyn Error>` — ignore it, use `LemmyResult<()>` per §2.1.
2. **`feedback_junior_worker_e2e_edit_hang.md`** — e2e.rs is 12000+ lines; use
   Grep to locate the anchor fn before editing. Do NOT use a single large Edit.
3. **`feedback_clippy_test_style.md`** — no `unwrap()`/`expect()` in test bodies;
   use `?` propagation (works cleanly with Case A `LemmyResult<()>`).
4. **`feedback_async_pool_test_pattern.md`** — AsyncPgConnection + DbPool pattern
   for e2e test setup.
5. Read **`crates/server/tests/e2e.rs`** at the `mod v1_sl_c_fixtures` closing
   region (Grep for `grace_check_escapes_case_when_sponsor_revoked` — Task 2's fn
   name — then read ±20 lines). Confirm the last fn in the mod uses `LemmyResult<()>`
   before writing Task 3. Canonical-schema-first gate check.

## 4. Constraints

- **File ownership:** edit only `crates/server/tests/e2e.rs` and `.claude/decision-queue.json`.
- **Attribution:** DQ entry `from: "impl"`, `answered_by: null`.
- **DQ mid-task push:** commit + push DQ immediately after writing it.
- **No new helpers:** Task 3's test body is self-contained (just `Ok(())`
  stub — no setup needed for a no-op test).
- **Anchor discipline:** insert AFTER the last existing test fn's closing `}`,
  BEFORE the mod's closing `}`. Do not disturb prior fn bodies.
- **Handover trailer:** include `HANDOVER:` YAML in commit body.

## 5. Prior task handover

```yaml
prior_tasks:
  - task: 1
    commit: 3e1911509
    filesCreated: []
    filesModified:
      - crates/server/tests/e2e.rs
    keyDecisions:
      - Task 1 replan established Case A (LemmyResult<T> uniform) for all v1_sl_c_fixtures helpers
    notes: Fire path test + mod shell established; Case A shape canonical for the mod
  - task: 2
    commit: d4578b415
    filesCreated: []
    filesModified:
      - crates/server/tests/e2e.rs
    keyDecisions:
      - Used Result<(), Box<dyn Error>> initially; fix-impl-3 restored Case A
      - Added EndorsementInsertForm import; seeded endorsement before surety
    notes: Escape path test; surety.revoked_at after decided_at triggers EscapeStatus::Escape
  - task: 2-fix
    commit: df1d419ef
    filesCreated: []
    filesModified:
      - crates/server/tests/e2e.rs
    keyDecisions:
      - Cherry-pick onto replan tip; only 1 sig remained Box<dyn Error>; restored to LemmyResult<()>
    notes: All v1_sl_c_fixtures signatures now uniformly Case A; phase-v1-SL-c-2 tip df1d419ef
```

Phase-2 e2e (DQ #171, run 25611232282) passed on tip df1d419ef. Task 3 appends the
no-op test after Task 2's fix fn. Confirm last fn in mod uses `LemmyResult<()>`
(canonical-schema-first gate) before writing.
