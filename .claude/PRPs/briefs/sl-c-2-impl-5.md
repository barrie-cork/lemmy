# [role:impl-task] sl-c-2-impl-5 — e2e test #5 batch size config (task 5)

## 1. Role + dispatch

`[role:impl-task]` — e2e test #5: `grace_check_batch_size_config_caps_iteration`
inside `mod v1_sl_c_fixtures` in `crates/server/tests/e2e.rs`.

Plan: `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md` §13 Task 5.
Phase branch: `phase-v1-SL-c-2`.
Base: phase branch tip (after Tasks 1+2+fix+3+4 have been merged).

## 2. Scope

Anchor-insert one test fn inside `mod v1_sl_c_fixtures` (after Task 4's test fn).
**This is the LAST test in `mod v1_sl_c_fixtures`** — the mod's closing `}` follows
immediately after this test's `Ok(())` line.

**Single file edit:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test #5 inside mod v1_sl_c_fixtures (closes the mod)
```

### 2.1 CANONICAL CASE OVERRIDE — read before writing any code

**The plan §13 Task 5 IMPLEMENT block prescribes `Result<(), Box<dyn Error>>` as the
return type. This is WRONG and must NOT be followed.**

`mod v1_sl_c_fixtures` was established by Task 1 using Case A (uniform `LemmyResult<()>`
throughout — zero `Box<dyn Error>`, zero `.map_err`). Task 5 appends into the **same
mod**. The canonical-schema-first gate requires: **mirror the existing mod's case verbatim**.

**Mandatory Case A substitution:**
- Test fn signature: `async fn grace_check_batch_size_config_caps_iteration() -> LemmyResult<()>`
- All `?` propagation: bare, no `.map_err` bridges
- Import: `use lemmy_utils::error::LemmyResult;` (already present from Task 1)

**This brief wins over the plan stub.**

### 2.2 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md` §G4 classifier table)

> | Failure signature | Auto-fix | Source lesson |
> | **4b** `error[E0277]: ?` couldn't convert `LemmyError`/`LemmyResult<T>` to `Box<dyn Error>` AND a v1-SL-* / v1-JM-* sibling fixtures module exists in the same file using `LemmyResult<()>` outer | flip the test fn signature to `LemmyResult<()>` AND flip ALL helper signatures in this module's fixtures mod to `LemmyResult<T>`. No `.map_err` bridges. Mirror the canonical sibling shape verbatim (Case A per lesson). | `feedback_lemmy_error_no_std_error.md` Case A |

### 2.3 Implementation

Grep for `grace_check_per_case_isolation_skips_bad_case` to locate Task 4's fn.
Insert the following immediately after that fn's closing `}`, still inside
`mod v1_sl_c_fixtures`. The mod's closing `}` follows immediately.

```rust
  #[tokio::test]
  async fn grace_check_batch_size_config_caps_iteration() -> LemmyResult<()> {
    // Per Test #5 (PRD §6.4 + §4.1 batch_size config).
    //
    // Setup: BREHON_DISABLE_GRACE_CHECK_JOB=1.
    //   UPSERT governance_config row "job.grace_check_batch_size" = 2
    //     (instance scope). Default is 100; we override to 2.
    //   5 sponsees + 5 sponsors (1 surety each). All 5 cases:
    //     status=SponsorLiabilityPending, grace_expires_at expired,
    //     sanction inserted.
    //
    // Drive (first invocation): run_grace_check_batch(&context).await.
    //
    // Assert (first invocation):
    //   - outcome.cases_processed == 2 (batch_size cap honoured).
    //   - outcome.fired == 2.
    //   - 2 cases transitioned to SponsorLiabilityFired.
    //   - 3 cases STILL == SponsorLiabilityPending.
    //
    // Drive (second invocation): run_grace_check_batch(&context).await.
    //
    // Assert (second invocation):
    //   - outcome.cases_processed == 2 (next 2 picked up).
    //   - outcome.fired == 2.
    //   - 4 cases now SponsorLiabilityFired total.
    //   - 1 case STILL == SponsorLiabilityPending.
    //
    // Drive (third invocation): run_grace_check_batch(&context).await.
    //
    // Assert (third invocation):
    //   - outcome.cases_processed == 1 (last remaining).
    //   - outcome.fired == 1.
    //   - all 5 cases now SponsorLiabilityFired.

    Ok(())
  }
```

**Key gotchas (from plan §13 Task 5):**

1. **governance_config UPSERT:** insert/update the `job.grace_check_batch_size` config
   row at `Instance` scope using diesel:
   ```rust
   insert_into(governance_config::table)
     .values((
       governance_config::scope.eq("instance"),
       governance_config::key.eq("job.grace_check_batch_size"),
       governance_config::value.eq("2"),
     ))
     .on_conflict((governance_config::scope, governance_config::key))
     .do_update()
     .set(governance_config::value.eq("2"))
     .execute(conn)
     .await?;
   ```

2. **Sort-order determinism:** batch query's `ORDER BY grace_expires_at ASC` means
   the 5 cases must have DISTINCT `grace_expires_at` values. Seed each with
   `grace_expires_at = now() - Duration::minutes(N)` for N in 5..=1 (distinct minutes).

3. **Config caching:** `ConfigCache` in `run_grace_check_batch` is fresh per call;
   second/third invocations see the updated config row correctly.

4. **Closing the mod:** this test is the LAST inside `mod v1_sl_c_fixtures`. The
   closing `}` of the mod follows immediately after `Ok(())`. Future SL-d/SL-e
   fixture mods open AFTER `mod v1_sl_c_fixtures` closes.

**MIRROR:** plan §6.4 batch_size knob.

### 2.4 Commit message

```
test(v1-SL-c-2): e2e test #5 — batch_size config caps iteration (task 5)
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
   git commit -m "chore(decision-queue): impl raised DQ #<id> — sl-c-2 task 5 workspace-check validate-pending"
   git push origin HEAD
   ```

## 3. Required reading

1. **`feedback_lemmy_error_no_std_error.md`** — Case A recipe. Plan stub says
   `Box<dyn Error>` — ignore it, use `LemmyResult<()>` per §2.1.
2. **`feedback_junior_worker_e2e_edit_hang.md`** — e2e.rs is 12000+ lines; use
   Grep to locate anchor fns before editing.
3. **`feedback_clippy_test_style.md`** — no `unwrap()`/`expect()` in test bodies.
4. **`feedback_async_pool_test_pattern.md`** — AsyncPgConnection + DbPool pattern.
5. Grep for `grace_check_per_case_isolation_skips_bad_case` before writing —
   confirm it exists and uses `LemmyResult<()>`. Canonical-schema-first gate.

## 4. Constraints

- **File ownership:** edit only `crates/server/tests/e2e.rs` and `.claude/decision-queue.json`.
- **Attribution:** DQ entry `from: "impl"`, `answered_by: null`.
- **DQ mid-task push:** commit + push DQ immediately after writing it.
- **Anchor discipline:** insert AFTER Task 4's fn closing `}`, before the mod's
  closing `}`. The mod closes after this test — do not add another `}` after `Ok(())`.
- **Handover trailer:** include `HANDOVER:` YAML in commit body.
