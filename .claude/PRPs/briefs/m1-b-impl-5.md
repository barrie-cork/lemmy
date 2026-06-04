---
phase: m1-b
role: impl-task
n: 5
authored: 2026-06-04
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m1-b
task_number: 5
minimax_trial: eligible (Task 5 — control=Sonnet, trial=MiniMax-M2.7; both arms read THIS brief)
---

# [role:impl-task] m1-b Task 5 — identity-policy validator + route registration

## 1. Role + dispatch line

```
[role:impl-task] m1-b task-5 identity-policy validator + routes — see .claude/PRPs/briefs/m1-b-impl-5.md
```

> **A/B note (does not change your work):** this task is dispatched as two parallel arms off the SAME base — a control arm (Sonnet) and a trial arm (MiniMax-M2.7). Each arm gets its own worktree branch and runs this brief independently. As a worker you do exactly what this brief says; you are NOT aware of the other arm. Do not reference, wait on, or coordinate with any other task.

## 2. Scope

Add the ADR-015 identity-policy validator (`validate_identity_policy`) to the Task-4 handler and call it in `admin_set_messaging_config` BEFORE the insert; register the messaging-config routes in the routes crate; and fix a one-line `pub` visibility warning Task 4 left behind. **Two files modified, zero created.** **This is pre-Shape-G: write a `validate-pending-laptop` DQ entry, commit + push, then STOP. Do NOT run cargo yourself** (cargo runs on the laptop, never on the daemon).

**Produce (exactly 2 files modified, 0 created):**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/messaging_config.rs   # + validate_identity_policy + call in admin_set; + pub on GetMessagingConfigQuery (§4.4 fix)
  - crates/api/routes/src/lib.rs                         # register /governance/admin/messaging-config POST+GET
requires:
  - task: 4   # SATISFIED: messaging_config.rs (admin_set_messaging_config + admin_get_messaging_config) merged on phase-m1-b @ 4f18ec3c3 (validated pass @ 0f8c51b9e). The validator lives in THIS handler; the routes point at its fns.
```

**Do NOT:**
- Add a new migration, a new DB write, a new table, or any governance-log / hash-chain write. Task 5 is a **pure validator** (no DB) + route wiring + a 1-line visibility fix. There is NO mutation added by this task beyond what Task 4 already does.
- Change `admin_set_messaging_config`'s write path, transaction posture, or value_type derivation. You ONLY insert the `validate_identity_policy(&data)?;` call BEFORE the existing insert. Leave everything else Task 4 wrote intact.
- Add `governance_log::append`, `run_transaction`, dry-run, or any feature Task 4 correctly omitted.
- Invent a NEW `LemmyErrorType` variant or a novel error idiom for the validator. **Mirror the rejection idiom the file already uses** — see §4.1 (the file rejects bad input with `LemmyErrorType::Unknown(format!(...))`; the validator does the same).
- Add identity-policy enforcement for room types beyond `jury*` / `appeal*`, or wire it to anything M2 (B-actor, portable IDs, room provisioning). The validator is **vacuously satisfied in M1** (no jury/appeals rooms exist yet) — it exists as a STRUCTURAL PIN so M2 cannot introduce a non-pseudonymous default. Match §10.4 exactly; add nothing.
- Touch any file other than the two named. Do NOT edit the handler's value_type table, the DTOs (`api_common/src/governance.rs`), the Diesel model, the migration, or `mod.rs` (Task 4 already added `pub mod messaging_config;`).
- Run `cargo check` / `cargo clippy` / any cargo or docker command. Validation is the laptop's job (write-then-stop).
- Commit to `governance-v0` or any branch other than your task worktree branch.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

### 3a. Handover from prior cohort (Task 4)

```yaml
prior_cohort_task:
  - task: 4
    finalize_merge: 4f18ec3c3          # Sonnet (control) arm FF-promoted onto phase-m1-b; DQ-resolved @ 0f8c51b9e
    validated: pass                     # cargo check --workspace --features full (0 err) + e2e --no-run (0 err); 1 non-blocking warning (see §4.4)
    filesCreated:
      - crates/api/api/src/governance/messaging_config.rs   # admin_set_messaging_config + admin_get_messaging_config + local split_typed_value (minus Float arm)
    filesModified:
      - crates/api/api/src/governance/mod.rs                # pub mod messaging_config;  (DONE — do NOT touch)
    keyDecisions:
      - "admin_set_messaging_config(Json(data): Json<AdminSetMessagingConfig>, local_user_view: LocalUserView, context: Data<LemmyContext>) -> LemmyResult<Json<AdminSetMessagingConfigResponse>> — you add `validate_identity_policy(&data)?;` near the TOP of this fn, after the admin gate, BEFORE the value_type split + insert."
      - "admin_get_messaging_config -> Json<ConfigValueWithProvenance> (the CORRECT bare provenance shape — do NOT change it)"
      - "the file already rejects bad input via `LemmyErrorType::Unknown(format!(...))` (in split_typed_value) — the identity-policy validator MUST use the same error idiom (§4.1)"
      - "AdminSetMessagingConfig { scope: String, key: String, value: serde_json::Value } — the validator reads data.key (String), data.scope (String), data.value (serde_json::Value)"
      - "the handler left ONE non-blocking warning: `struct GetMessagingConfigQuery` is private but used in pub fn admin_get_messaging_config (private_interfaces). You fix it with a 1-char `pub` — §4.4."
```

### MIRROR refs — read these EXACT locations on your base branch (phase-m1-b @ 0f8c51b9e)

`crates/api/api/src/governance/messaging_config.rs` (the file you are editing — read it fully first):
- The full `admin_set_messaging_config` fn — find where the admin gate ends and the value-type split / insert begins. The `validate_identity_policy(&data)?;` call goes BETWEEN the admin gate and the split (so a policy-violating request is rejected before any value processing).
- The existing `split_typed_value` (or inline JSON-shape match) rejection lines — note the EXACT `LemmyErrorType::Unknown(format!(...))` idiom. Your validator's rejection MUST match it (same variant, same `format!` style with a descriptive ADR-015 message).
- `struct GetMessagingConfigQuery` (the GET query struct, ~line 135) — note it is declared without `pub`. §4.4: add `pub`.

`crates/api/api/src/governance/admin_config.rs` (denial-path mirror):
- **`:426`** — the policy-denial return shape. **NOTE:** that line returns `LemmyErrorType::NotAnAdmin.into()` because it is an *admin-capability* denial. **Your validator is NOT an admin denial** — it is a *value-policy* refusal (a valid admin still cannot set a non-pseudonymous identity_policy for a jury/appeal scope). So do NOT copy `NotAnAdmin`. Mirror the *structure* (a guarded `return Err(...)` before the main work) but use the **value-rejection idiom from your own file** (`LemmyErrorType::Unknown(format!(...))` — see §4.1), per the GOTCHA in plan §13 Task 5 ("match the exact LemmyErrorType variant the codebase already exposes for config-rejection").

`crates/api/routes/src/lib.rs` (route registration — the file you are editing):
- **`:478-512`** — the existing `scope("/admin")` block where `admin_set_config` (the admin_config GET/POST) is registered. **Mirror this scope-nesting + registration pattern exactly.** Under the existing `/admin` scope, add a nested `scope("/messaging-config")` with `.route("", post().to(admin_set_messaging_config)).route("", get().to(admin_get_messaging_config))` (§4.2).
- **`~:36`** — the import block where `admin_config`'s handler fns are brought in. Add the two imports for `admin_set_messaging_config` + `admin_get_messaging_config` near here, mirroring how the admin_config handlers are imported (same path style: `lemmy_api::governance::messaging_config::{...}` or whatever the sibling `admin_config` import uses — match it exactly).

### Plan sections
- `.claude/PRPs/plans/m1.plan.md` §13 Task 5 (ACTION / IMPLEMENT file 1+2 / MIRROR / GOTCHA / VALIDATE) — authoritative.
- `.claude/PRPs/plans/m1.plan.md` §10.4 (the `validate_identity_policy` reference implementation — your validator mirrors this, with the real error variant per §4.1).
- `.claude/PRPs/plans/m1.plan.md` §12 (NOT building in m1 — the validator is the ONLY ADR-015 surface; no enforcement beyond jury/appeal scopes, no M2 wiring).
- ADR-015 in `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — the identity-policy pin the validator structurally enforces (pseudonymous-by-default; jury/appeals may never be non-pseudonymous).

### Lessons (mandatory)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — error construction: `LemmyResult<T>` + `?`; build errors via `LemmyErrorType::<Variant>.into()`. The validator returns `LemmyResult<()>` and constructs its rejection via `LemmyErrorType::Unknown(format!(...)).into()` (matching the file's existing idiom).
- `.claude/lessons/feedback_clippy_test_style.md` — workspace clippy denies `unwrap`/`expect`/`allow_attributes`. The validator + the handler edit must use NO `.unwrap()` / `.expect()` — propagate with `?`. (Task 4's Sonnet arm already used `.ok_or_else(...)?`; keep that style.)
- `.claude/lessons/feedback_features_full_workspace_only.md` + `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — the laptop DoD runs `--features full --workspace` (never `-p <crate> --features full`). The routes crate + handler are gated `#[cfg(feature = "full")]` where they touch DB/route types — mirror the existing gating.
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write the DQ entry + push, then stop. Do NOT run cargo.

> Consult-only (informational, no action): the value_type derivation table, the single-write/no-transaction posture, and the DTO shapes are ALL Task 4's responsibility and are already correct on your base — do NOT re-derive or re-edit them. Task 5 is additive: one validator fn + one call site + two route lines + one `pub`.

## 4. Constraints

### 4.0 The validator is a STRUCTURAL PIN — vacuously satisfied in M1, must STILL exist + compile (the load-bearing constraint)

Plan §13 Task 5 GOTCHA is authoritative:

> The validator is vacuously satisfied in M1 (no jury/appeals rooms yet) — it must STILL exist and be tested (criterion #5), so M2 cannot introduce a non-pseudonymous default. Match the exact `LemmyErrorType` variant the codebase already exposes for config-rejection (read `admin_config.rs:426`).

In M1 there are no jury/appeals room scopes, so the validator's guard never fires at runtime today. That is intentional. It exists NOW so that when M2 introduces jury/appeal rooms, the code path that would let an operator set a non-pseudonymous identity_policy for those scopes is **already closed**. Do not "simplify" it away as dead code; do not gate it behind a feature flag; do not make it conditional on jury rooms existing. It is a permanent structural refusal.

### 4.1 validate_identity_policy — the validator (per §10.4, with the REAL error variant)

Add a module-private helper to `messaging_config.rs`:

```rust
// Rejects any attempt to set a jury/appeals room-type identity_policy to a non-pseudonymous value.
// Vacuously satisfied in M1 (jury/appeals rooms arrive in M2) — the gate exists NOW so M2 cannot
// introduce a non-pseudonymous default. ADR-015.
fn validate_identity_policy(data: &AdminSetMessagingConfig) -> LemmyResult<()> {
  if data.key == "identity_policy"
    && (data.scope.starts_with("jury") || data.scope.starts_with("appeal"))
    && data.value.as_str() != Some("pseudonymous")
  {
    return Err(LemmyErrorType::Unknown(format!(
      "identity_policy for scope `{}` must be `pseudonymous` (ADR-015 — jury/appeals rooms may never be non-pseudonymous)",
      data.scope
    )).into());
  }
  Ok(())
}
```

**Error-variant decision (do NOT deviate):** use `LemmyErrorType::Unknown(format!(...))` — this is the SAME variant `messaging_config.rs` already uses to reject bad value shapes (in the value_type split). Using the file's own rejection idiom keeps the handler internally consistent. Do NOT use `NotAnAdmin` (that is an admin-gate denial, semantically wrong here — a valid admin is still refused this specific value). Do NOT invent a new variant. If, after reading the file, you believe a more specific variant exists and is clearly more correct (e.g. a `CouldntUpdate` or a config-specific variant the file imports), you MAY use it — but ONLY if it is already imported/used in this file or `admin_config.rs`; otherwise default to `Unknown(format!(...))`. Do NOT add a new variant to `crates/utils/src/error.rs`.

> `data.scope` is a `String` (per the Task-3 DTO `AdminSetMessagingConfig { scope: String, ... }`), so `.starts_with(...)` works directly — no `.as_str()` needed for `starts_with`. `data.value` is `serde_json::Value`, so `.as_str()` returns `Option<&str>` (compare to `Some("pseudonymous")`).

### 4.2 Call the validator in admin_set_messaging_config

Insert exactly one line in `admin_set_messaging_config`, AFTER the admin-capability gate and BEFORE the value_type derivation / split / insert:

```rust
  validate_identity_policy(&data)?;   // §10.4 — ADR-015 pin
```

Placement rationale: a policy-violating request must be rejected before any value processing or DB read/write. Do NOT move it after the insert. Do NOT change any other line of `admin_set_messaging_config`.

### 4.3 Route registration in routes/src/lib.rs

Mirror `routes/src/lib.rs:478-512` (the `scope("/admin")` block that registers `admin_set_config`). Under the existing `/admin` scope, add:

```rust
.service(
  scope("/messaging-config")
    .route("", post().to(admin_set_messaging_config))
    .route("", get().to(admin_get_messaging_config)),
)
```

…or whatever the EXACT registration idiom the sibling uses (it may be `.route("/messaging-config", post().to(...))` flat rather than a nested scope — **read `:478-512` and match the sibling's exact pattern**, whether nested-scope or flat-route). The resulting routes must be `POST /api/v4/governance/admin/messaging-config` and `GET /api/v4/governance/admin/messaging-config`. Add the imports for the two handler fns near `:36`, mirroring the admin_config import style. The handler fns are at `lemmy_api::governance::messaging_config::{admin_set_messaging_config, admin_get_messaging_config}` (confirm the crate path by reading how `admin_set_config` is imported — match it exactly).

### 4.4 Fold-in fix: make GetMessagingConfigQuery pub (Task-4 leftover warning)

Task 4 (Sonnet, validated) left ONE non-blocking warning: `struct GetMessagingConfigQuery` (the GET query struct, ~line 135 in `messaging_config.rs`) is declared private but used in the signature of the `pub fn admin_get_messaging_config` → `warning: private_interfaces`. Fix it with a one-character change:

```rust
// before:
struct GetMessagingConfigQuery { ... }
// after:
pub struct GetMessagingConfigQuery { ... }
```

This is a clippy/rustc visibility lint, not a logic change. Do this in the SAME edit pass as the validator (Task 5 already owns this file). After your edit, `cargo check --workspace --features full` should emit **0 warnings** for this file (the laptop will verify).

> If the struct name differs slightly from `GetMessagingConfigQuery` (read the file — Task 4 named it; the warning cites the exact name at line ~135), apply `pub` to whatever struct the `private_interfaces` warning names. If you cannot find a private struct used in a `pub fn` signature in this file, the warning may already be absent (in which case skip §4.4 — do not invent a struct to make pub). Read the file first.

### 4.5 If something is genuinely ambiguous — raise a blocker, do not guess

If the route-registration idiom (nested-scope vs flat-route), the handler-fn import path, or the exact error variant is not derivable from the MIRROR refs + plan, raise a `kind: "blocker"` DQ entry (`from: "impl"`, `answered_by: null`), commit + push it immediately (mid-task visibility), and STOP that thread. Do NOT invent a novel shape. Generate the id via `bash scripts/brehon/dq-v3-new-entry.sh`. (Reminder: guessing past a flagged ambiguity is a known failure mode — when in doubt, the documented fallback is "use the sibling's exact pattern"; if the sibling is genuinely silent, raise the blocker.)

### 4.6 validate-pending-laptop DQ entry

After editing both files and committing, append a `validate-pending-laptop` DQ entry. Generate the id via `bash scripts/brehon/dq-v3-new-entry.sh` (or append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`). Fields:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<your worktree branch>",
  "phase_task": 5,
  "commands": [
    "./scripts/brehon/cargo-check.sh --workspace --features full",
    "./scripts/brehon/cargo-test.sh --no-run -p lemmy_server --test e2e"
  ],
  "question": "Workspace check + e2e test-target compile for identity-policy validator + route registration — laptop runs cargo.",
  "options": ["pass", "fail"],
  "context": "Task 5 added validate_identity_policy (ADR-015 pin) to messaging_config.rs + a call in admin_set_messaging_config before the insert, registered POST+GET /governance/admin/messaging-config in routes/src/lib.rs, and made GetMessagingConfigQuery pub (clears the Task-4 private_interfaces warning). Pure validator (no DB write) + route wiring + 1-char visibility fix. cargo check --workspace --features full required (feature=full route + Diesel gating); e2e --no-run confirms the routes + validator compile into the test target. Expect 0 warnings (the Task-4 private_interfaces warning is fixed here). Laptop only.",
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
- **Commit subject:** `feat(api): add identity-policy validator + register messaging-config routes (task 5)`
- Sequence:
  ```
  git add crates/api/api/src/governance/messaging_config.rs crates/api/routes/src/lib.rs .claude/decision-queue.json
  git commit -m "feat(api): add identity-policy validator + register messaging-config routes (task 5)"
  git push origin <your worktree branch>
  ```
  (Or two commits — code first, then `chore(decision-queue): impl raised validate-pending-laptop DQ — m1-b task 5`.)
- End the commit body with a `LESSON:` trailer if you hit a footgun, and the `HANDOVER:` YAML trailer below.
- Then **STOP**. Do not run cargo.

### 4.8 Attribution
- `from: "impl"` on the DQ entry; `answered_by: null` (the laptop advisor mutates it).
- NEVER `answered_by: "advisor"` and NEVER `approved_by` from this session.

### Mandatory lessons fired for this brief
- handler/validator error idiom → `feedback_lemmy_error_no_std_error.md` ✓ + `feedback_clippy_test_style.md` ✓ (no unwrap/expect; `?` propagation; `Unknown(format!(...))` rejection)
- `#[cfg(feature = "full")]` route + Diesel gating → `feedback_features_full_workspace_only.md` ✓ + `feedback_features_full_p_crate_incompatible.md` ✓ (DoD uses `--workspace`)
- validate-pending-laptop → `feedback_validate_pending_laptop_write_then_stop.md` ✓

> No `crates/server/tests/e2e.rs` edit in this task (the e2e tests are Task 7) → the e2e-anchor lessons do NOT fire. No new migration → the migration lessons do NOT fire. No multi-write → the transaction lesson does NOT fire (this task adds ZERO DB writes).

## HANDOVER (fill in your real values before final commit)

```yaml
HANDOVER:
  task: m1-b-task-5
  filesModified:
    - crates/api/api/src/governance/messaging_config.rs
    - crates/api/routes/src/lib.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "validate_identity_policy: module-private fn; rejects identity_policy != pseudonymous for jury*/appeal* scopes; LemmyErrorType::Unknown(format!(...)) idiom (matches the file's existing value-rejection style); vacuously satisfied in M1 (structural pin for M2 per ADR-015)"
    - "called as `validate_identity_policy(&data)?;` after the admin gate, BEFORE the value_type split/insert in admin_set_messaging_config"
    - "routes: POST + GET /governance/admin/messaging-config registered under the existing /admin scope in routes/src/lib.rs (mirrored :478-512 admin_set_config registration)"
    - "GetMessagingConfigQuery made pub — clears the Task-4 private_interfaces warning (now 0 warnings)"
  notes: "validation = cargo check --workspace --features full + e2e --no-run on laptop; worker wrote validate-pending-laptop DQ + stopped. This is the last Tree-B handler/route task; Task 6 (bridge_notify) + Task 7 (e2e tests) remain. The validator's runtime test is Task 7 criterion #5."
```
