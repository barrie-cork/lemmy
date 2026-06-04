---
phase: m1-b
role: impl-task
n: 4
authored: 2026-06-04
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m1-b
task_number: 4
minimax_trial: eligible (Task 4 — control=Sonnet, trial=MiniMax-M2.7; both arms read THIS brief)
---

# [role:impl-task] m1-b Task 4 — admin messaging-config handler (single write)

## 1. Role + dispatch line

```
[role:impl-task] m1-b task-4 messaging-config handler — see .claude/PRPs/briefs/m1-b-impl-4.md
```

> **A/B note (does not change your work):** this task is dispatched as two parallel arms off the SAME base — a control arm (Sonnet) and a trial arm (MiniMax-M2.7). Each arm gets its own worktree branch and runs this brief independently. As a worker you do exactly what this brief says; you are NOT aware of the other arm. Do not reference, wait on, or coordinate with any other task.

## 2. Scope

Create the messaging-config admin handler: `admin_set_messaging_config` (write) + `admin_get_messaging_config` (read), plus the `pub mod` declaration. **One file created, one file modified.** **This is pre-Shape-G: write a `validate-pending-laptop` DQ entry, commit + push, then STOP. Do NOT run cargo yourself** (cargo runs on the laptop, never on the daemon).

**Produce (exactly 1 file created, 1 file modified):**

```yaml
creates:
  - crates/api/api/src/governance/messaging_config.rs   # admin_set_messaging_config + admin_get_messaging_config + local split_typed_value helper
modifies:
  - crates/api/api/src/governance/mod.rs                # + pub mod messaging_config;
requires:
  - task: 2   # SATISFIED: GovernanceMessagingConfig model+create+read_current merged on phase-m1-b @ edd2d4b0f (validated pass). Handler inserts GovernanceMessagingConfigInsertForm + reads _current view.
  - task: 3   # SATISFIED: AdminSetMessagingConfig + AdminSetMessagingConfigResponse merged on phase-m1-b @ edd2d4b0f (validated pass). Handler signature consumes the request DTO + returns the response DTO.
```

**Do NOT:**
- Add `run_transaction` / `conn.run_transaction(...)` / `get_conn` + transaction wrapping. **This is a SINGLE write. A single Diesel write needs no transaction.** Adding one is dead ceremony — see §4.0 (the load-bearing constraint). The sibling `admin_config.rs` uses `run_transaction` ONLY because it does a second write (`governance_log::append`); M1 drops that append, so the transaction goes too.
- Add `governance_log::append(...)` / any hash-chain / governance-log write. That is an M2 concern, explicitly out of scope (plan §12, §10.3 GOTCHA).
- Add `validate_identity_policy`. **That is Task 5, not Task 4.** Do NOT add the ADR-015 validator or any call to it. Task 4 is auth → derive value_type → split → single insert → read-back.
- Register any route. Routes are Task 5 (`crates/api/routes/src/lib.rs`). Do NOT touch the routes crate.
- Mirror admin_config's `metadata_for_key` / `CONFIG_KEY_METADATA` / `value_type_label` / `validate_value_shape` path. **M1 has NO per-key metadata registry.** The `value_type` is derived from the JSON value's runtime shape — see §4.1.
- Add a `dry_run` / preview / impact-query branch. M1 has no dry-run (plan §12).
- Run `cargo check` / `cargo clippy` / any cargo or docker command. Validation is the laptop's job (write-then-stop).
- Commit to `governance-v0` or any branch other than your task worktree branch.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

### 3a. Handover from prior cohorts (Tasks 2 + 3)

```yaml
prior_cohort_tasks:
  - task: 2
    finalize_merge: 8499c6c9a         # daemon-merged into phase-m1-b; then advanced to edd2d4b0f by Task 3 promotion
    validated: pass
    filesCreated:
      - crates/db_schema/src/source/governance/governance_messaging_config.rs   # GovernanceMessagingConfig model + InsertForm + create() + read_current()
    keyDecisions:
      - "GovernanceMessagingConfig::create(pool, form: &GovernanceMessagingConfigInsertForm) -> the single-insert entry point YOU call"
      - "GovernanceMessagingConfig::read_current(...) reads the _current view — admin_get uses this"
      - "model fields: id, scope, key, value_type:String, value_int:Option<i64>, value_bool:Option<bool>, value_text:Option<String>, valid_from, updated_by — NO value_float"
      - "GovernanceMessagingConfigInsertForm: #[derive(Clone, Default)] + Insertable, NO AsChangeset (append-only). Fields: scope, key, value_type, value_int, value_bool, value_text, updated_by"
      - "MessagingConfigId(pub i32) lives in db_schema::newtypes"
  - task: 3
    finalize_merge: edd2d4b0f          # Sonnet (control) arm canonical on phase-m1-b
    validated: pass
    filesModified:
      - crates/api/api_common/src/governance.rs   # AdminSetMessagingConfig + AdminSetMessagingConfigResponse
    keyDecisions:
      - "AdminSetMessagingConfig { scope: String, key: String, value: serde_json::Value } — your request signature. NO value_type field; you derive it."
      - "AdminSetMessagingConfigResponse { previous: ConfigValueWithProvenance, new: ConfigValueWithProvenance } — your return shape. NO governance_log_id/preview/applied."
      - "ConfigValueWithProvenance { value: serde_json::Value, effective_from: String } — build previous + new from the _current rows"
```

### MIRROR refs — read these EXACT locations on your base branch (phase-m1-b @ edd2d4b0f)

`crates/api/api/src/governance/admin_config.rs` (the canonical sibling handler — **mirror structure, NOT the metadata/governance-log/transaction parts**):
- **`:383` `admin_set_config`** — the handler skeleton: `Json(data)` + `LocalUserView` + `Data<LemmyContext>` extractors, `LemmyResult<Json<...Response>>` return. Mirror the **extractor signature + return type shape**. Do NOT mirror its metadata lookup (steps 1/4/5), its `run_transaction` write branch (`:497`), or its `governance_log::append` (`:558`/`:809`).
- **`:682` `fn split_typed_value(vt: ValueType, value: &Value, key: &str)`** — **this is module-private (bare `fn`, confirmed by advisor 2026-06-04 — it is NOT reachable cross-module).** So you must **copy the int / bool / text arms into a local helper** in your file. **DROP the `ValueType::Float` arm** (M1's table has no `value_float` column; the InsertForm has no `value_float` field). Return only the 3 columns the InsertForm needs `(value_int, value_bool, value_text)`.
- **`:747` `async fn check_policy(...)`** — the admin-capability gate. **This is also module-private.** Mirror the *capability check it performs* (it reads admin status off the `LocalUserView` / reputation flags). If `check_policy` is not reachable from your module, perform the equivalent admin-capability assertion inline using the same `LocalUserView`-based check the codebase already uses for admin endpoints (read how `:747` does it and replicate the assertion, returning the same error variant on denial). Do NOT invent a new capability model.

`crates/api/api/src/governance/config.rs`:
- **`:69` `pub enum ValueType`** — variants `Int | Float | Bool | Text | Enum`. You need `Int | Bool | Text` for your local split helper (drop Float; Enum maps to the text arm in the sibling). You MAY import this enum (it's `pub`), or derive your column-split directly off the JSON shape without the enum — your choice, but the column mapping must match §4.1.

`crates/api/api/src/governance/admin_config.rs` (for the read-back / response build):
- **`ConfigValueWithProvenance` construction** — find where admin_config builds a `ConfigValueWithProvenance` from a config row (`value: serde_json::Value`, `effective_from: String`). Mirror that conversion to build your `previous` and `new`.

### Plan sections
- `.claude/PRPs/plans/m1.plan.md` §13 Task 4 (ACTION / IMPLEMENT / MIRROR / GOTCHA / VALIDATE) — authoritative.
- `.claude/PRPs/plans/m1.plan.md` §10.3 (single-write-no-governance-log handler design + the run_transaction GOTCHA box) — read the GOTCHA box twice.
- `.claude/PRPs/plans/m1.plan.md` §12 (NOT building in m1 — no hash-chain, no dry-run, no validator-in-Task-4).

### Lessons (mandatory + one inverted-application warning)
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — **READ THIS, THEN DO THE OPPOSITE for Task 4.** This lesson says "2+ writes → `run_transaction`". Task 4 has **exactly ONE write**, so the lesson does **NOT** apply — and actively adding a transaction here is the failure mode. The lesson's own threshold is "2+ writes"; a single insert is below it. The §10.3 GOTCHA spells out why: admin_config wraps in a transaction *only because* it appends a governance_log row (a 2nd write); M1 drops that, leaving one write. **If you find yourself reaching for `run_transaction`, STOP — you are over-applying this lesson.**
- `.claude/lessons/feedback_features_full_workspace_only.md` — the laptop DoD runs `--features full --workspace` (never `-p <crate> --features full`). Handler code is gated `#[cfg(feature = "full")]` where it touches Diesel — mirror how admin_config.rs gates DB-touching code.
- `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — DoD uses `--workspace`, never `-p lemmy_api --features full`.
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — error construction: use `LemmyResult<T>` + `?`; build errors via `LemmyErrorType::<Variant>.into()` (the sibling uses `LemmyErrorType::Unknown(...)` for bad-input cases — match the sibling's error idiom for value-shape rejections).
- `.claude/lessons/feedback_clippy_test_style.md` — workspace clippy denies `unwrap`/`expect`/`allow_attributes`; propagate with `?`, never `.unwrap()` in handler code.
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write the DQ entry + push, then stop.

> Consult-only (informational, no action unless relevant): `feedback_governance_type_state_handlers.md` — the GovernanceCase<S> type-state pattern. **Task 4 does NOT load a ModerationCase and does NOT match on CaseStatus**, so the type-state wrapper does not apply here. Listed only so you don't go looking for it.

## 4. Constraints

### 4.0 SINGLE WRITE — NO TRANSACTION, NO GOVERNANCE-LOG (the load-bearing constraint)

The plan §13 Task 4 IMPLEMENT + GOTCHA are authoritative:

> `check_policy` (admin capability) → `split_typed_value` → **SINGLE insert. NO `run_transaction`, NO `governance_log::append`.**
> GOTCHA: do NOT add `run_transaction` — a single write needs no transaction, and adding the `governance_log::append` would be an M2 hash-chain scope breach.

Your write path is **exactly one Diesel insert** (`GovernanceMessagingConfig::create`). That is the entire mutation. Do not wrap it in a transaction. Do not append to any governance log. The append-only history is achieved by the table design (new row per change, `_current` view picks the latest `valid_from`) — NOT by a hash-chain.

### 4.1 value_type derivation (the second load-bearing divergence)

The request DTO is `{ scope, key, value: serde_json::Value }` — **there is NO `value_type` field and NO metadata registry in M1.** Derive `value_type` from the JSON value's runtime shape:

| `data.value` shape | `value_type` string | column set on InsertForm |
|---|---|---|
| `value.is_boolean()` | `"bool"` | `value_bool = Some(value.as_bool()?)`, others `None` |
| `value.is_i64()` or `value.is_u64()` | `"int"` | `value_int = Some(...)`, others `None` |
| `value.is_string()` | `"text"` | `value_text = Some(value.as_str()?.to_string())`, others `None` |
| anything else (float / array / object / null) | — | **reject** with `LemmyErrorType::Unknown(format!("messaging-config value for key `{key}` must be bool, integer, or string"))` |

> M1's table CHECK constraint allows only `int | bool | text` (no `float`). A float / array / object value must be rejected at the handler, not passed to the insert (it would violate the CHECK). This matches the migration (Task 1) — there is no `value_float` column.

The local `split_typed_value` helper (copied from `admin_config.rs:682` minus the Float arm) takes the derived `value_type` (or you may fold the derivation + split into one match on the JSON shape — either is fine, as long as the column mapping matches the table above and unknown shapes are rejected).

### 4.2 admin_set_messaging_config — the write handler

Signature (mirror `admin_config.rs:383` extractor shape, your DTO types):

```rust
pub async fn admin_set_messaging_config(
  Json(data): Json<AdminSetMessagingConfig>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<AdminSetMessagingConfigResponse>> {
  // 1. Admin capability gate (mirror check_policy :747; inline if not reachable).
  // 2. Derive value_type from data.value shape + split into (int,bool,text) columns (§4.1). Reject unknown shape.
  // 3. Read the CURRENT value for (scope, key) BEFORE the write → build `previous: ConfigValueWithProvenance`.
  //    (If no prior row exists, `previous` is the empty/default ConfigValueWithProvenance — mirror how admin_config
  //     represents "no prior value"; if the sibling has no such case, use ConfigValueWithProvenance::default().)
  // 4. Build GovernanceMessagingConfigInsertForm { scope, key, value_type, value_int, value_bool, value_text,
  //    updated_by: Some(local_user_view.person.id) } and call GovernanceMessagingConfig::create(&mut context.pool(), &form).await?.
  //    SINGLE write. No transaction.
  // 5. Read the CURRENT value again AFTER the write (or construct `new` from the value just written) → build `new`.
  // 6. Ok(Json(AdminSetMessagingConfigResponse { previous, new }))
}
```

> Steps 3 + 5 are two READS around one WRITE. Reads are not writes — this does NOT make it a multi-write handler. Still no transaction (reads don't need one, and read→write→read with no atomicity requirement across them is correct for a config set).

### 4.3 admin_get_messaging_config — the read handler

A read-only handler that returns the current effective config. Mirror the admin_config GET handler's shape if one exists; otherwise:
- Admin capability gate (same as set).
- Read `_current` view via `GovernanceMessagingConfig::read_current(...)` (Task 2 provided this).
- Return the current value(s). Shape: follow the plan — if §13/§10 specifies a response type for GET, use it; if not specified, return the `_current` rows in the simplest reasonable shape (a `Vec` of current configs, or a single `ConfigValueWithProvenance` if the GET is scoped to one key). **If the GET response shape is ambiguous from the plan, raise a `kind: "blocker"` DQ (`from: "impl"`) rather than guessing** — per §4.5.

### 4.4 mod.rs

Add `pub mod messaging_config;` to `crates/api/api/src/governance/mod.rs`, alphabetically/positionally consistent with the existing `pub mod admin_config;` (or wherever the sibling modules are declared — match the existing ordering style).

### 4.5 If something is genuinely ambiguous — raise a blocker, do not guess

If the GET response shape, the `check_policy` reachability, or the "no prior value" representation is not derivable from the MIRROR refs + plan, raise a `kind: "blocker"` DQ entry (`from: "impl"`, `answered_by: null`), commit + push it immediately (mid-task visibility), and STOP that thread. Do NOT invent a novel shape. Generate the id via `bash scripts/brehon/dq-v3-new-entry.sh`.

### 4.6 validate-pending-laptop DQ entry

After writing both files and committing, append a `validate-pending-laptop` DQ entry. Generate the id via `bash scripts/brehon/dq-v3-new-entry.sh` (or append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`). Fields:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<your worktree branch>",
  "phase_task": 4,
  "commands": [
    "./scripts/brehon/cargo-check.sh --workspace --features full",
    "./scripts/brehon/cargo-test.sh --no-run -p lemmy_server --test e2e"
  ],
  "question": "Workspace check + e2e test-target compile for messaging-config handler — laptop runs cargo.",
  "options": ["pass", "fail"],
  "context": "Task 4 created crates/api/api/src/governance/messaging_config.rs (admin_set_messaging_config + admin_get_messaging_config + local split_typed_value minus float arm) and added pub mod messaging_config to mod.rs. SINGLE write, NO run_transaction, NO governance_log append (plan §10.3 GOTCHA). value_type derived from JSON shape (no metadata registry in M1). cargo check --workspace --features full required (feature=full Diesel gating); e2e --no-run confirms the handler compiles into the test target. Laptop only.",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

> Two commands: §15.1 workspace check + §15.3 e2e test-target compile. The laptop advisor runs both serially.

### 4.7 Commit + push discipline
- **Commit subject:** `feat(api): add messaging-config admin handler (task 4)`
- Sequence:
  ```
  git add crates/api/api/src/governance/messaging_config.rs crates/api/api/src/governance/mod.rs .claude/decision-queue.json
  git commit -m "feat(api): add messaging-config admin handler (task 4)"
  git push origin <your worktree branch>
  ```
  (Or two commits — code first, then `chore(decision-queue): impl raised validate-pending-laptop DQ — m1-b task 4`.)
- End the commit body with a `LESSON:` trailer if you hit a footgun, and the `HANDOVER:` YAML trailer below.
- Then **STOP**. Do not run cargo.

### 4.8 Attribution
- `from: "impl"` on the DQ entry; `answered_by: null` (the laptop advisor mutates it).
- NEVER `answered_by: "advisor"` and NEVER `approved_by` from this session.

### Mandatory lessons fired for this brief
- single-write handler → `feedback_multi_write_handlers_need_transactions.md` cited as **inverted-application warning** (do NOT add a transaction) ✓
- `#[cfg(feature = "full")]` Diesel gating → `feedback_features_full_workspace_only.md` ✓ + `feedback_features_full_p_crate_incompatible.md` ✓ (DoD uses `--workspace`)
- handler error idiom → `feedback_lemmy_error_no_std_error.md` ✓ + `feedback_clippy_test_style.md` ✓
- validate-pending-laptop → `feedback_validate_pending_laptop_write_then_stop.md` ✓

## HANDOVER (fill in your real values before final commit)

```yaml
HANDOVER:
  task: m1-b-task-4
  filesCreated:
    - crates/api/api/src/governance/messaging_config.rs
  filesModified:
    - crates/api/api/src/governance/mod.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "admin_set_messaging_config: SINGLE insert, NO run_transaction, NO governance_log append (§10.3 GOTCHA)"
    - "value_type derived from JSON value shape (bool/int/text), NOT from a metadata registry (M1 has none); float/array/object rejected"
    - "split_typed_value copied locally from admin_config.rs:682 minus the Float arm (no value_float column in M1)"
    - "validate_identity_policy NOT added — that is Task 5; no route registered — that is Task 5"
  notes: "validation = cargo check --workspace --features full + e2e --no-run on laptop; worker wrote validate-pending-laptop DQ + stopped. Task 5 adds validate_identity_policy into THIS file + registers the route in routes/src/lib.rs."
```
