---
phase: m2-core-hook
role: impl-task
n: 1
authored: 2026-06-05
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m2-core-hook
task_number: 1
minimax_trial: eligible (Task 1 — control=Sonnet, trial=MiniMax-M2.7; both arms read THIS brief)
---

# [role:impl-task] m2-core-hook Task 1 — bridge-notify discriminated-union DTO

## 1. Role + dispatch line

```
[role:impl-task] m2-core-hook task-1 bridge payload DTO — see .claude/PRPs/briefs/m2-core-hook-impl-1.md
```

> **A/B note (does not change your work):** this task is dispatched as two parallel arms off the SAME base — a control arm (Sonnet) and a trial arm (MiniMax-M2.7). Each arm gets its own worktree branch and runs this brief independently. As a worker you do exactly what this brief says; you are NOT aware of the other arm. Do not reference, wait on, or coordinate with any other task.

## 2. Scope

Add the shared discriminated-union bridge-notify payload DTO to `api_common`. **One file, modify-only.** **This is pre-Shape-G: write a `validate-pending-laptop` DQ entry, commit + push, then STOP. Do NOT run cargo yourself** (cargo runs on the laptop, never on the daemon).

**Produce (exactly 1 file modified, 0 created):**

```yaml
creates: []
modifies:
  - crates/api/api_common/src/governance.rs   # + BridgeNotifyPayload (tagged enum) + CaseTransitionEvent + PrivateMessagePayload
requires: []   # Task 1 is the base of the chain; no prior cohort dep.
```

**Do NOT:**
- Add any handler, route, validator, or DB code. Task 1 is the DTO only. The producer that builds these structs is Task 2/3 (in `bridge_notify.rs`).
- Add `ts-rs` derives to the new types. The bridge consumer is an external Matrix daemon, NOT the TS client — these payloads are serde-only (this is a deliberate divergence from the `ts-rs`-gated config DTOs in the same file). See §4.0.
- Derive `Eq` on any struct that carries a field which is not `Eq`. (All fields here are integer / `Option<i32>` / `String` / `CaseStatus` — those ARE `Eq`-capable, so `Eq` is allowed here; but follow §4.1's exact derive stack — do not invent.)
- Touch any file other than `crates/api/api_common/src/governance.rs`.
- Run `cargo check` / `cargo clippy` / any cargo or docker command. Validation is the laptop's job (write-then-stop).
- Commit to `governance-v0` or any branch other than your task worktree branch.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

### MIRROR refs — read these EXACT line ranges on your base branch

**Tagged-enum pattern (the load-bearing MIRROR):** `crates/db_views/site/src/api.rs:762-770` — the `#[serde(tag = "type_", rename_all = "snake_case")]` internally-tagged enum (`PostOrCommentOrPrivateMessage`). Mirror its `#[serde(tag = ...)]` attribute exactly for `BridgeNotifyPayload`. (That sibling carries ts-rs cfg_attrs — you OMIT those per §4.0.)

**Existing PM payload shape to lift:** `crates/api/api_utils/src/bridge_notify.rs:28-38` — the current inline `struct Payload { private_message_id: i32, creator_id: i32, recipient_id: i32 }`. Your `PrivateMessagePayload` is this struct, moved into `api_common` and named. (Task 2 will refactor `bridge_notify.rs` to use it — you just define it here.)

**DTO conventions in the target file:** `crates/api/api_common/src/governance.rs` — read `:449` `AdminSetConfig` (the `#[skip_serializing_none]` + derive stack convention). Your new types follow the same *base* derive stack MINUS the ts-rs cfg_attrs (see §4.0). Note `CaseStatus`'s import path: grep the file for how an existing governance DTO imports an enum from `lemmy_db_schema_file::enums` (e.g. `CaseSeverity`, `JuryDecision`) — `CaseStatus` imports identically.

**The CaseStatus enum:** `crates/db_schema_file/src/enums.rs:382-429` — confirm `CaseStatus` derives `Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash` + `#[serde(rename_all = "snake_case")]`. It IS usable in `api_common` without the `full` feature (the serde derives are unconditional). 12 variants.

### Plan sections
- `.claude/PRPs/plans/m2-core-transition-hook.plan.md` §"Step-by-Step Tasks" Task 1 (ACTION / IMPLEMENT / MIRROR / GOTCHA / VALIDATE).
- `.claude/PRPs/plans/m2-core-transition-hook.plan.md` §"Patterns to Mirror" → `TAGGED_ENUM_PAYLOAD`.

### Lessons (mandatory + consult)
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write the DQ entry + push, then stop. (MANDATORY — validate-pending-laptop.)
- Consult-only: `feedback_advisor_cr_enum_drift.md` — keep the new tagged-enum's serde tag/rename conventions consistent with the `PostOrCommentOrPrivateMessage` sibling; do not invent a novel discriminator key (`type_` is the convention, not `type` or `kind`).

## 4. Constraints

### 4.0 SERDE-ONLY, NO ts-rs (the divergence from sibling config DTOs)

The config DTOs in this file (`AdminSetConfig` etc.) carry `#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]` because they cross to the TypeScript client. **The bridge-notify payload does NOT** — it is consumed by the external Matrix bridge over HTTP, never by the TS client. So:

- Derive stack for ALL three new types: `#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]` — NO ts-rs cfg_attrs, NO `#[cfg_attr(feature = "full", ...)]` (these are plain DTOs, no diesel).
- `BridgeNotifyPayload` additionally carries `#[serde(tag = "type_", rename_all = "snake_case")]`.

### 4.1 The three types (authoritative shapes — plan §13 Task 1)

```rust
/// Payload mirrored to the Matrix bridge for a private-message event.
/// (Lifted verbatim from bridge_notify.rs's prior inline struct; Task 2 refactors the producer.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrivateMessagePayload {
  pub private_message_id: i32,
  pub creator_id: i32,
  pub recipient_id: i32,
}

/// Payload mirrored to the Matrix bridge when a moderation case changes status.
/// Integer + enum fields only — no usernames/emails (the bridge owns pseudonym resolution).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseTransitionEvent {
  pub case_id: i32,
  pub old_status: Option<CaseStatus>,
  pub new_status: CaseStatus,
  pub community_id: Option<i32>,
  pub target_type: String,
}

/// Discriminated union of bridge-notify events. `type_` tag distinguishes
/// the variants on the wire (serde internally-tagged, snake_case).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type_", rename_all = "snake_case")]
pub enum BridgeNotifyPayload {
  PrivateMessage(PrivateMessagePayload),
  CaseTransition(CaseTransitionEvent),
}
```

**Verify before committing:**
- `CaseStatus` is imported at the top of `governance.rs` (add the `use lemmy_db_schema_file::enums::CaseStatus;` import if absent — mirror the path an existing enum import in this file uses; do NOT invent a new path).
- The `#[serde(tag = "type_", ...)]` attribute is on the ENUM, byte-matching the `PostOrCommentOrPrivateMessage` sibling's tag style (`type_` with the trailing underscore — it is the established convention to avoid the `type` reserved word).
- `Option<CaseStatus>` serialises cleanly (it does — `CaseStatus: Serialize`). On the wire `old_status: null` is valid (the bridge tolerates it).
- Place the three types together as a labelled block (a `// M2 bridge-notify payload (governance-triggered rooms)` comment) — group them, do not scatter.

### 4.2 validate-pending-laptop DQ entry

After writing the file and committing, append a `validate-pending-laptop` DQ entry. Generate the id via `bash scripts/brehon/dq-v3-new-entry.sh` (or append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`). Fields:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<your worktree branch>",
  "phase_task": 1,
  "commands": [
    "./scripts/brehon/cargo-check.sh --workspace --features full"
  ],
  "question": "Workspace check for the bridge-notify discriminated-union DTO — laptop runs cargo.",
  "options": ["pass", "fail"],
  "context": "Task 1 added BridgeNotifyPayload (serde tagged enum, type_ tag) + CaseTransitionEvent + PrivateMessagePayload to api_common/src/governance.rs. Serde-only (NO ts-rs — bridge consumer, not TS client). CaseStatus imported from lemmy_db_schema_file::enums. cargo check --workspace --features full required (governance enum reachability under full). Laptop only.",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

> One command: §"Validation Commands" Level 1 workspace check. (No e2e needed — Task 1 is a pure DTO with no test surface yet; the e2e suite is Task 8.)

### 4.3 Commit + push discipline
- **Commit subject:** `feat(api_common): add bridge-notify discriminated-union payload DTO (task 1)`
- Sequence:
  ```
  git add crates/api/api_common/src/governance.rs .claude/decision-queue.json
  git commit -m "feat(api_common): add bridge-notify discriminated-union payload DTO (task 1)"
  git push origin <your worktree branch>
  ```
  (Or two commits — code first, then `chore(decision-queue): impl raised validate-pending-laptop DQ — m2-core-hook task 1`.)
- End the commit body with a `LESSON:` trailer if you hit a footgun, and the `HANDOVER:` YAML trailer below.
- Then **STOP**. Do not run cargo.

### 4.4 Attribution
- `from: "impl"` on the DQ entry; `answered_by: null` (the laptop advisor mutates it).
- NEVER `answered_by: "advisor"` and NEVER `approved_by` from this session.

### Mandatory lessons fired for this brief
- validate-pending-laptop → `feedback_validate_pending_laptop_write_then_stop.md` ✓
- (No e2e / migration / handler-2-write / cfg-full-gate file-class match → no other mechanical lesson fires. §2.3 hybrid search confirmed no cross-cutting lesson needed for a pure tagged-enum DTO.)

## HANDOVER (fill in your real values before final commit)

```yaml
HANDOVER:
  task: m2-core-hook-task-1
  filesCreated: []
  filesModified:
    - crates/api/api_common/src/governance.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "BridgeNotifyPayload: serde internally-tagged enum (tag = type_, snake_case); variants PrivateMessage + CaseTransition"
    - "CaseTransitionEvent { case_id:i32, old_status:Option<CaseStatus>, new_status:CaseStatus, community_id:Option<i32>, target_type:String } — integer+enum only, no PII"
    - "PrivateMessagePayload lifted from bridge_notify.rs inline struct (Task 2 refactors producer)"
    - "serde-only, NO ts-rs (external Matrix bridge consumer, not TS client) — deliberate divergence from config DTOs in same file"
  notes: "validation = cargo check --workspace --features full on laptop; worker wrote validate-pending-laptop DQ + stopped. Task 2 imports BridgeNotifyPayload + PrivateMessagePayload to refactor bridge_notify.rs; Task 3 builds CaseTransitionEvent in the new governance_case_after_transition fn."
```
