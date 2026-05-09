# [role:impl-task] sl-c-2-fix-impl-3 — Case A type-shape restore for Task 2 (fix-impl)

## 1. Role + dispatch

`[role:impl-task]` — Fix Case A type-shape mismatch in `mod v1_sl_c_fixtures` introduced
by Task 2. Task 2 wrote `Result<(), Box<dyn Error>>` / `Result<T, Box<dyn Error>>` helpers
— mismatching Task 1's `LemmyResult<()>` / `LemmyResult<T>` shape. This is Case C per the
lesson; §G4 row 4b mandates flipping all signatures to Case A.

Plan: `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md` §13 Task 2 (fix).
Phase branch: `phase-v1-SL-c-2` (tip `3e1911509`).
Base this fix on the **Task 2 worker branch** (which has the correct test body, just wrong types):
`junior/role-impl-task-sl-c-2-impl-2-e2e-test-2-escape-path-see-claude-prps-briefs-sl-c-2-impl-2-md-164`

## 2. Scope

### 2.1 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md` §G4 classifier table)

> | Failure signature | Auto-fix | Source lesson |
> | **4b** `error[E0277]: ?` couldn't convert `LemmyError`/`LemmyResult<T>` to `Box<dyn Error>` AND a v1-SL-* / v1-JM-* sibling fixtures module exists in the same file using `LemmyResult<()>` outer | flip the test fn signature to `LemmyResult<()>` AND flip ALL helper signatures in this module's fixtures mod to `LemmyResult<T>`. No `.map_err` bridges. Mirror the canonical sibling shape verbatim (Case A per lesson). | `feedback_lemmy_error_no_std_error.md` Case A |

### 2.2 What Task 2 wrote (wrong)

In `mod v1_sl_c_fixtures` on the worker branch:
- `count_log_entries(...)` → `Result<i64, Box<dyn Error>>` ← **WRONG**
- `read_log_payload(...)` → `Result<Option<Value>, Box<dyn Error>>` ← **WRONG**
- `grace_check_escapes_case_when_sponsor_revoked_after_decided_at()` → `Result<(), Box<dyn Error>>` ← **WRONG**
- `use lemmy_utils::error::LemmyResult` import is **MISSING** from mod imports

### 2.3 Required changes (≤4 edits, all in `crates/server/tests/e2e.rs`)

**Edit 1 — Restore `LemmyResult` import in mod imports block**

Find the mod's `use` block. Add `use lemmy_utils::error::LemmyResult;` after the existing
`use lemmy_db_schema_file::{...};` block and before `use serde_json::Value;`.

The phase branch (Task 1) has this at line 11952. The worker branch dropped it.

**Edit 2 — Fix `count_log_entries` return type**

```rust
// WRONG (Task 2 wrote):
async fn count_log_entries(conn: &mut AsyncPgConnection, kind: &str) -> Result<i64, Box<dyn Error>> {

// CORRECT (Case A — matches Task 1 helpers):
async fn count_log_entries(conn: &mut AsyncPgConnection, kind: &str) -> LemmyResult<i64> {
```

No body changes needed — bare `?` propagates cleanly in both cases with Diesel.

**Edit 3 — Fix `read_log_payload` return type**

```rust
// WRONG (Task 2 likely wrote):
async fn read_log_payload(conn: &mut AsyncPgConnection, kind: &str) -> Result<Option<Value>, Box<dyn Error>> {

// CORRECT:
async fn read_log_payload(conn: &mut AsyncPgConnection, kind: &str) -> LemmyResult<Option<Value>> {
```

Verify the actual wrong signature by reading the file first before editing.

**Edit 4 — Fix test fn return type**

```rust
// WRONG:
async fn grace_check_escapes_case_when_sponsor_revoked_after_decided_at() -> Result<(), Box<dyn Error>> {

// CORRECT:
async fn grace_check_escapes_case_when_sponsor_revoked_after_decided_at() -> LemmyResult<()> {
```

No body changes needed — the test fn body uses only bare `?` (confirmed by advisor review).

### 2.4 What NOT to change

- Do NOT change `count_log_entries` or `read_log_payload` helper BODIES — only the return
  type annotation on the fn signature line.
- Do NOT add `.map_err` bridges anywhere — Case A uses none.
- Do NOT change Task 1's test fn (`grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries`) — it's already correct Case A.
- Do NOT change the imports added by Task 2 for `EndorsementInsertForm`, `endorsement` schema,
  etc. — those are correct additions.

### 2.5 Commit message

```
fix(v1-SL-c-2): restore Case A LemmyResult<T> shape in mod v1_sl_c_fixtures (task 2 fix-impl-3)
```

### 2.6 Shape G post-push

After committing and pushing:

1. Get workflow run id:
   ```bash
   gh run list --repo barrie-cork/lemmy --branch <your-branch> \
     --workflow cargo-validate-workspace.yml --limit 1 --json databaseId
   ```
2. Next safe DQ id = **170** (max across governance-v0 live file + archives is 169).
3. Write `kind: "validate-pending"`, `from: "impl"`, `id: 170` to `.claude/decision-queue.json`.
4. Commit + push immediately:
   ```
   git add .claude/decision-queue.json
   git commit -m "chore(decision-queue): impl raised DQ #170 — sl-c-2 fix-impl-3 workspace-check validate-pending"
   git push origin HEAD
   ```

## 3. Required reading

1. **`feedback_lemmy_error_no_std_error.md`** — Case A recipe. This fix applies Case A.
   Task 1 is Case A. Task 2 must also be Case A (same mod).
2. **`feedback_junior_worker_e2e_edit_hang.md`** — e2e.rs is large; use Grep to locate
   each fn signature before editing. Do NOT use a single large Edit.
3. **`feedback_clippy_test_style.md`** — no `unwrap()`/`expect()` in test helpers.
4. Read the **actual current state** of the worker branch file before editing — verify each
   wrong signature matches what's described in §2.3 before applying the fix. Use Grep for
   `count_log_entries` and `read_log_payload` to find exact lines.

## 4. Constraints

- **Base branch:** `junior/role-impl-task-sl-c-2-impl-2-e2e-test-2-escape-path-see-claude-prps-briefs-sl-c-2-impl-2-md-164`
  — so this fix builds on Task 2's correct body, not on the phase branch (which doesn't have
  Task 2's test body yet).
- **File ownership:** edit only `crates/server/tests/e2e.rs` and `.claude/decision-queue.json`.
- **Attribution:** DQ entry `from: "impl"`, `answered_by: null`.
- **DQ mid-task push:** commit + push DQ immediately after writing it.
- **≤4 file edits** — this is a narrow type-fix, not a rewrite.
- **Handover trailer:** include `HANDOVER:` YAML in commit body.

## 5. Prior context

Task 2 (Junior #164) wrote the correct escape-path test body but used the wrong return
types throughout `mod v1_sl_c_fixtures`:
- All three fn signatures use `Result<T, Box<dyn Error>>` instead of `LemmyResult<T>`
- `use lemmy_utils::error::LemmyResult` was dropped from imports

Task 1's helpers (`count_log_entries`, `read_log_payload`, `seed_pending_case`,
`seed_active_surety`) all use `LemmyResult<T>` on the phase branch. This fix restores
uniformity so the whole mod is Case A.

Phase branch tip: `3e1911509`. Task 2 worker branch:
`junior/role-impl-task-sl-c-2-impl-2-...-164` (tip `f46dc935b`).
