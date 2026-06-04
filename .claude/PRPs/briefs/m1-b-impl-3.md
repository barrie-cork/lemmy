---
phase: m1-b
role: impl-task
n: 3
authored: 2026-06-04
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m1-b
task_number: 3
minimax_trial: eligible (Task 3 — control=Sonnet, trial=MiniMax-M2.7; both arms read THIS brief)
---

# [role:impl-task] m1-b Task 3 — messaging-config DTOs

## 1. Role + dispatch line

```
[role:impl-task] m1-b task-3 messaging-config DTOs — see .claude/PRPs/briefs/m1-b-impl-3.md
```

> **A/B note (does not change your work):** this task is dispatched as two parallel arms off the SAME base — a control arm (Sonnet) and a trial arm (MiniMax-M2.7). Each arm gets its own worktree branch and runs this brief independently. As a worker you do exactly what this brief says; you are NOT aware of the other arm. Do not reference, wait on, or coordinate with any other task.

## 2. Scope

Add the request/response DTOs for the admin messaging-config endpoint to `api_common`. **One file, modify-only.** **This is pre-Shape-G: write a `validate-pending-laptop` DQ entry, commit + push, then STOP. Do NOT run cargo yourself** (cargo runs on the laptop, never on the daemon).

**Produce (exactly 1 file modified, 0 created):**

```yaml
creates: []
modifies:
  - crates/api/api_common/src/governance.rs   # + AdminSetMessagingConfig + AdminSetMessagingConfigResponse
requires:
  - task: 2   # SATISFIED: model+newtype merged on phase-m1-b @ 8499c6c9a (validated pass). Response reuses ConfigValueWithProvenance.
```

**Do NOT:**
- Copy the *full* `AdminSetConfigResponse` shape. The sibling response has `applied` / `config_id` / `governance_log_id` / `preview` / `applied_at` fields — those exist because the v0 config handler does a **governance_log append + dry-run**. M1 messaging-config does **neither** (single write, no hash-chain, no dry-run — plan §10.3 + §12). **Your response is the SIMPLER `{ previous, new }` shape.** This is the load-bearing divergence from the sibling — see §4.0.
- Add any handler, route, validator, or DB code. Task 3 is DTOs only. The handler is Task 4; routes + validator are Task 5.
- Add a `dry_run`, `apply_at`, `governance_log_id`, `config_id`, or `preview` field to either DTO.
- Touch any file other than `crates/api/api_common/src/governance.rs`.
- Run `cargo check` / `cargo clippy` / any cargo or docker command. Validation is the laptop's job (write-then-stop).
- Commit to `governance-v0` or any branch other than your task worktree branch.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

### 3a. Handover from prior cohort (Task 2, #575)

```yaml
prior_cohort_tasks:
  - task: 2
    commit: 4039b72ca
    finalize_merge: 8499c6c9a        # daemon-merged into phase-m1-b
    validated: pass                  # cargo check --workspace --features full, 0 errors 0 warnings (advisor-laptop 2026-06-04)
    filesCreated:
      - crates/db_schema/src/source/governance/governance_messaging_config.rs   # GovernanceMessagingConfig model + InsertForm
    filesModified:
      - crates/db_schema/src/newtypes.rs        # MessagingConfigId(pub i32) @ :305
      - crates/db_schema_file/src/schema.rs     # governance_messaging_config table macro + joinable + allow_tables
    keyDecisions:
      - "MessagingConfigId(pub i32) lives in db_schema::newtypes (NOT api_common)"
      - "model fields: id, scope, key, value_type, value_int:Option<i64>, value_bool:Option<bool>, value_text:Option<String>, valid_from, updated_by — NO value_float"
    notes: "Task 3 reuses the EXISTING ConfigValueWithProvenance type (governance.rs:498). MessagingConfigId is referenced only indirectly via the response — your DTOs do NOT need to import MessagingConfigId unless the response carries an id (it does NOT in the simple {previous,new} shape — see §4.0)."
```

### MIRROR refs — read these EXACT line ranges on your base branch (phase-m1-b @ 8499c6c9a)

`crates/api/api_common/src/governance.rs`:
- **`:449` `AdminSetConfig`** — the request DTO to mirror for FIELD-LEVEL shape (derive stack + `#[skip_serializing_none]` + `#[cfg_attr(feature = "ts-rs", ...)]` gating). Your request is a SUBSET of this (only `scope`, `key`, `value`).
- **`:470` `AdminSetConfigResponse`** — the sibling response. **Read it to see what NOT to copy.** It has `applied`/`config_id`/`governance_log_id`/`preview`/`applied_at`. Your response drops ALL of those and is just `{ previous, new }`.
- **`:498` `ConfigValueWithProvenance`** — REUSE this type verbatim (do not redefine it). Fields: `value: serde_json::Value`, `effective_from: String`. Your response's `previous` and `new` are both this type.

> The sibling's `ConfigChangePreview` (`:485-487`) is `{ previous, new, downstream_impact }`. Your `AdminSetMessagingConfigResponse` is the same minus `downstream_impact` and flattened (no nested `preview` wrapper) — i.e. `{ previous: ConfigValueWithProvenance, new: ConfigValueWithProvenance }` directly on the response.

### Plan sections
- `.claude/PRPs/plans/m1.plan.md` §13 Task 3 (ACTION / IMPLEMENT / MIRROR / GOTCHA / VALIDATE).
- `.claude/PRPs/plans/m1.plan.md` §10.3 (the single-write-no-governance-log handler design — confirms WHY the response is simple).
- `.claude/PRPs/plans/m1.plan.md` §12 (NOT building in m1 — confirms no dry-run, no hash-chain).

### Lessons (mandatory — fired by file-class table)
- `.claude/lessons/feedback_features_full_workspace_only.md` — **fired by the `#[cfg_attr(feature = "full", ...)]` gating (GOTCHA).** The DTOs are gated for `ts-rs` under `feature = "ts-rs"` exactly like the sibling. The laptop DoD runs `--features full --workspace` (never `-p <crate> --features full`).
- `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — the DoD uses `--workspace`, never `-p api_common --features full`.
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write the DQ entry + push, then stop.

> Consult-only: `feedback_advisor_cr_enum_drift.md` — keep new DTO field/enum shapes consistent with the sibling's serde conventions; do not invent a novel value-encoding. (No enum in this task; informational.)

## 4. Constraints

### 4.0 RESPONSE SHAPE (the load-bearing divergence from the sibling)

The plan §13 Task 3 IMPLEMENT line is authoritative:

> `AdminSetMessagingConfig { scope, key, value: serde_json::Value }` + `AdminSetMessagingConfigResponse { previous, new: ConfigValueWithProvenance }`

So:

**Request — `AdminSetMessagingConfig`:**
| Field | Type |
|---|---|
| `scope` | `String` |
| `key` | `String` |
| `value` | `serde_json::Value` |

(Three fields only. The sibling `AdminSetConfig` also has `value_type`, `apply_at`, `dry_run`, `reason` — **do NOT add those**. The handler derives `value_type` from the JSON value's shape via `split_typed_value`, per §10.3.)

**Response — `AdminSetMessagingConfigResponse`:**
| Field | Type |
|---|---|
| `previous` | `ConfigValueWithProvenance` |
| `new` | `ConfigValueWithProvenance` |

(Two fields only. Reuse the EXISTING `ConfigValueWithProvenance` — do not redefine it, do not wrap it in a `preview`, do not add `applied`/`config_id`/`governance_log_id`/`applied_at`/`downstream_impact`.)

### 4.1 The one file — `crates/api/api_common/src/governance.rs` (MODIFY)

Add the two structs adjacent to the `AdminSetConfig` family (after `AdminSetConfigResponse` / `ConfigValueWithProvenance`, near `:498`, keeping the messaging-config pair grouped). For BOTH structs, mirror the sibling's exact attribute stack:

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminSetMessagingConfig {
  pub scope: String,
  pub key: String,
  pub value: serde_json::Value,
}
```

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminSetMessagingConfigResponse {
  pub previous: ConfigValueWithProvenance,
  pub new: ConfigValueWithProvenance,
}
```

**Verify before committing:**
- The derive stack EXACTLY matches the sibling `AdminSetConfig` (`:449`) attribute-for-attribute. If the sibling uses `Eq` on `ConfigValueWithProvenance` but not on `AdminSetConfig`, follow the sibling per-struct — do not add `Eq` to a struct whose field (`serde_json::Value`) does not implement `Eq`. (`serde_json::Value` is NOT `Eq` — so `AdminSetMessagingConfig` and `AdminSetMessagingConfigResponse` must NOT derive `Eq`, matching `AdminSetConfig` which also omits it. The sibling `ConfigValueWithProvenance` that you REUSE does derive `Eq` — that's fine, you're not changing it.)
- `ConfigValueWithProvenance` is already imported/defined in this same file — no new `use` needed for it.
- `serde_json::Value` — confirm the file already references `serde_json::Value` (the sibling `AdminSetConfig.value` uses it, so the import path is already established). Use the same path the sibling uses (`serde_json::Value`, not a bare `Value`).
- Doc-comments: add a one-line `///` doc on each struct in the sibling's style (the sibling docs name the route — e.g. `/// Request payload for POST /api/v4/governance/admin/messaging-config`). Keep them accurate to the route Task 5 will register (`/governance/admin/messaging-config`).

### 4.2 validate-pending-laptop DQ entry

After writing the file and committing, append a `validate-pending-laptop` DQ entry. Generate the id via `bash scripts/brehon/dq-v3-new-entry.sh` (or append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`). Fields:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<your worktree branch>",
  "phase_task": 3,
  "commands": [
    "./scripts/brehon/cargo-check.sh --workspace --features full",
    "./scripts/brehon/cargo-test.sh --no-run -p lemmy_server --test e2e"
  ],
  "question": "Workspace check + e2e test-target compile for messaging-config DTOs — laptop runs cargo.",
  "options": ["pass", "fail"],
  "context": "Task 3 added AdminSetMessagingConfig + AdminSetMessagingConfigResponse to api_common/src/governance.rs. Response reuses ConfigValueWithProvenance (NOT the full AdminSetConfigResponse shape — no governance_log_id/preview/applied). ts-rs derive gating mirrors AdminSetConfig sibling. cargo check --workspace --features full required (ts-rs export gating); e2e --no-run confirms the DTO compiles into the test target (plan §15.3 lists Task 3). Laptop only.",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

> Two commands: §15.1 workspace check + §15.3 e2e test-target compile (the plan lists Task 3 in §15.3). The laptop advisor runs both serially.

### 4.3 Commit + push discipline
- **Commit subject:** `feat(api_common): add messaging-config admin DTOs (task 3)`
- Sequence:
  ```
  git add crates/api/api_common/src/governance.rs .claude/decision-queue.json
  git commit -m "feat(api_common): add messaging-config admin DTOs (task 3)"
  git push origin <your worktree branch>
  ```
  (Or two commits — code first, then `chore(decision-queue): impl raised validate-pending-laptop DQ — m1-b task 3`.)
- End the commit body with a `LESSON:` trailer if you hit a footgun, and the `HANDOVER:` YAML trailer below.
- Then **STOP**. Do not run cargo.

### 4.4 Attribution
- `from: "impl"` on the DQ entry; `answered_by: null` (the laptop advisor mutates it).
- NEVER `answered_by: "advisor"` and NEVER `approved_by` from this session.

### Mandatory lessons fired for this brief
- `#[cfg_attr(feature = "...")]` gating → `feedback_features_full_workspace_only.md` ✓ + `feedback_features_full_p_crate_incompatible.md` ✓ (DoD uses `--workspace`)
- validate-pending-laptop → `feedback_validate_pending_laptop_write_then_stop.md` ✓

## HANDOVER (fill in your real values before final commit)

```yaml
HANDOVER:
  task: m1-b-task-3
  filesCreated: []
  filesModified:
    - crates/api/api_common/src/governance.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "AdminSetMessagingConfig { scope, key, value: serde_json::Value } — 3 fields, no value_type/apply_at/dry_run/reason"
    - "AdminSetMessagingConfigResponse { previous, new: ConfigValueWithProvenance } — 2 fields, REUSES existing ConfigValueWithProvenance, NO governance_log_id/preview/applied"
    - "ts-rs derive gating mirrors AdminSetConfig sibling; no Eq (serde_json::Value not Eq)"
  notes: "validation = cargo check --workspace --features full + e2e --no-run on laptop; worker wrote validate-pending-laptop DQ + stopped. Task 4 (handler) consumes AdminSetMessagingConfig as its request signature; Task 5 registers the route."
```
