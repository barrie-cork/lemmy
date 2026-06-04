---
phase: m1-a
role: impl-task
n: 9
plan_task: 9
authored: 2026-06-04
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m1-a
task_number: 9
minimax_trial: NOT eligible — Tree A is greenfield (no in-repo MIRROR sibling), fails §0.1 MIRROR-ref-heavy criterion
related_dq: (none at authorship — Task 8 validate passed)
---

# [role:impl-task] m1-a Task 9 — AS transaction server (axum + ruma-appservice-api)

## 1. Role + dispatch line

```
[role:impl-task] m1-a task-9 AS transaction server — see .claude/PRPs/briefs/m1-a-impl-9.md
```

You are the **impl-task** subagent (Sonnet 4.6 per your frontmatter). Execute plan task 9 from `.claude/PRPs/plans/m1.plan.md` §13 Task 9 (Tree A).

> **⚠️ R8 — THIS IS NOT THE LEMMY WORKSPACE.** `services/bridge/` is a **greenfield, workspace-EXCLUDED** crate with its own `Cargo.toml`, its own error type (`anyhow`), and EXTERNAL conventions (`ruma-appservice-api` / `axum`). **Do NOT apply Lemmy-workspace lessons** here — no `LemmyResult`, no `LemmyError`, no Diesel, no `--features full`, no `#[cfg(feature = "full")]`, no `crates/server/tests/e2e.rs`. Read `ruma-appservice-api` 0.16 endpoint docs (via Ref MCP) BEFORE writing — there is no in-repo sibling to mirror.

> **⚠️ Task 8 skeleton is your base.** `services/bridge/{Cargo.toml, src/main.rs, src/config.rs}` already exist on the branch you start from. Read them before writing anything. Task 9 adds `src/appservice.rs` and mounts its router in `src/main.rs`.

## 2. Scope

Implement the **application-service HTTP endpoints** Tuwunel pushes events to. This is the AS transaction server that listens for Matrix events from the homeserver.

**This is pre-Shape-G: write a `validate-pending-laptop` DQ entry, commit + push, then STOP. Do NOT run cargo or docker yourself.**

**Produce (exactly 1 file created, 1 file modified):**

```yaml
creates:
  - services/bridge/src/appservice.rs   # axum router for PUT /transactions, GET /users, GET /rooms; hs_token auth
modifies:
  - services/bridge/src/main.rs         # mount appservice router (replace TODO task 9 stub)
```

**Commit message** (exactly): `feat(bridge): AS transaction server — axum routes + hs_token auth (task 9)`

**Do NOT** in this task:
- Implement puppet logic (Task 10), relay (Task 11), provisioning/soft-pause (Task 12), or docker-compose (Task 13).
- Apply ANY Lemmy-workspace convention (`LemmyResult`, `LemmyError`, Diesel, `--features full`, the e2e harness) to `services/bridge/**`. R8 — catch-fire.
- Touch any file under `crates/`, `migrations/`, `docs/`, or `.claude/` other than `.claude/decision-queue.json`.
- Run `cargo check` / `cargo build` / `cargo clippy` / `docker` — write-then-stop (R9).
- Commit to `governance-v0` or any branch other than your task worktree branch.
- Write `approved_by: "advisor"` in any DQ entry.
- Change the `[dependencies]` in `services/bridge/Cargo.toml` — the deps from Task 8 already include `ruma-appservice-api = "0.16"` and `axum = "0.8"`. **No new dependencies needed for Task 9.** If a dep is genuinely missing (compile error), raise a `kind: "blocker"` DQ rather than silently adding deps.

## 3. Required reading (in order)

### 3a. Handover from Task 8

```yaml
prior_cohort_task: 8
filesCreated_by_task8:
  - services/bridge/Cargo.toml
  - services/bridge/src/main.rs
  - services/bridge/src/config.rs
filesModified_by_task8:
  - Cargo.toml (added exclude = ["services/bridge"])
keyDecisions_from_task8:
  - "crate name: brehon-bridge; error type: anyhow"
  - "deps: matrix-sdk 0.18, ruma-appservice-api 0.16, axum 0.8, reqwest 0.12, serde, serde_json, tracing, anyhow"
  - "BridgeConfig: { tuwunel_url, as_token, hs_token, bridge_port, brehon_read_url } from env vars"
  - "main.rs: axum Router::new() stub (TODO task 9) + tokio::time::interval soft-pause stub (TODO task 12)"
notes: "Read services/bridge/src/main.rs and services/bridge/src/config.rs before writing — they define BridgeConfig you'll use."
```

**Before writing anything:** `Read services/bridge/src/main.rs` and `services/bridge/src/config.rs` to see the existing stubs and `BridgeConfig` fields. Your code mounts onto what's already there.

### MIRROR refs — EXTERNAL conventions (R8 — no in-repo sibling)

Read via `mcp__ref-context__ref_search_documentation` / `ref_read_url`:
- **`ruma-appservice-api` 0.16** — the endpoint request/response types for:
  - `PUT /_matrix/app/v1/transactions/{txnId}` — `push_transactions` endpoint
  - `GET /_matrix/app/v1/users/{userId}` — `query_user_id` endpoint
  - `GET /_matrix/app/v1/rooms/{roomAlias}` — `query_room_alias` endpoint
  Confirm the exact type names and how to extract the `access_token` bearer for hs_token auth.
- **`axum` 0.8** — how to extract `TypedHeader` or `Query`/`Bearer` auth from request headers; how to mount a sub-router; `State` extractor with `Arc<AppState>`.
- **`docs/research/matrix-homeserver-selection-2026.md`** — the four "verify before committing" Tuwunel items (§Recommendation). You MUST address items 1 and 2 (see §4.3 below).

### Plan sections (authoritative)
- `.claude/PRPs/plans/m1.plan.md` §13 Task 9 (ACTION / FILES / IMPLEMENT / MIRROR / GOTCHA / VALIDATE) — **THE contract.** Read the full block.
- `.claude/PRPs/plans/m1.plan.md` §7 R8 + R9.

### Lessons (short list — most Lemmy lessons do NOT apply, R8)
- `.claude/lessons/feedback_read_canonical_before_writing_spec.md` — Tier-2: read 1–2 sibling instances first. For Tree A "sibling" = `ruma-appservice-api` endpoint examples. Cite the example you followed in the commit body or a code comment.
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write the DQ entry + push, then STOP. Do NOT run cargo.
- **Explicitly NON-applicable:** `feedback_lemmy_error_no_std_error.md`, `feedback_features_full_workspace_only.md`, `feedback_clippy_test_style.md`, `feedback_lemmy_migration_runner.md`, all Diesel/e2e lessons — **R8: NONE apply to `services/bridge/**`.**

## 4. Constraints

### 4.1 The three AS endpoints to implement

Implement an axum `Router` that handles:

```
PUT  /_matrix/app/v1/transactions/{txnId}
GET  /_matrix/app/v1/users/{userId}
GET  /_matrix/app/v1/rooms/{roomAlias}
```

Use `ruma-appservice-api` 0.16 request/response types where they exist. If ruma provides typed extractors, use them; if not (bare path params), handle manually with axum extractors.

**Response shapes (confirm exact types via Ref MCP):**
- `PUT /transactions/{txnId}` — responds `200 OK` with `{}` (empty JSON object) on success; logs the events in the body for now (relay to Brehon wired in Task 11).
- `GET /users/{userId}` — responds `200 OK` with `{}` (user is handled by this AS) or `404` (not known to this AS).
- `GET /rooms/{roomAlias}` — responds `200 OK` with `{}` or `404`.

Keep the handler bodies MINIMAL — log the incoming data, return the correct response shape. The real relay logic wires in Task 11.

### 4.2 hs_token auth (middleware or extractor)

Every request from Tuwunel carries `Authorization: Bearer <hs_token>`. Verify it against `BridgeConfig.hs_token` on every request to all three endpoints.

Implementation options (pick one, cite your choice):
- **axum middleware** (`tower::Service` or `axum::middleware::from_fn`) — cleanest for blanket auth.
- **per-handler extractor** — simpler, slightly repetitive.

Reject with `401 Unauthorized` (JSON `{"errcode":"M_FORBIDDEN","error":"Invalid token"}`) on mismatch.

### 4.3 Tuwunel verify items (MANDATORY — GOTCHA from plan §13 Task 9)

The plan and `docs/research/matrix-homeserver-selection-2026.md` name four Tuwunel integration verify items. Two are relevant at this task:

**Item 1 — Issue #219 `whoami` response-code interaction:**  
Tuwunel may probe `GET /_matrix/app/v1/` or a `whoami` endpoint during AS registration. Confirm (via the matrix research doc + `ruma-appservice-api` docs) whether your router needs to handle this path or if the three standard endpoints suffice for Tuwunel to accept the AS. Document your finding in the commit body (one line: "Issue #219: <verified — requires X / not required because Y>"). Do NOT block the task on this if the doc gives a clear answer; raise a `kind: "blocker"` DQ only if genuinely unresolvable from the docs.

**Item 2 — Issue #465 `ip_source` NOT set (loopback AS):**  
The bridge runs on loopback against Tuwunel. The docker-compose (Task 13) must NOT set `ip_source` so that loopback-sourced requests aren't misidentified. This is a **Task 13 constraint**, not a code change here. Document in the commit body: "Issue #465: ip_source verified as Task 13 docker-compose constraint — no code change needed in appservice.rs."

The other two verify items (federation-disabled workaround; never-switch-fork) are Task 13 constraints. Note them in your HANDOVER so the Task 13 brief author is reminded.

### 4.4 `src/appservice.rs` shape (scaffold to follow)

```rust
use axum::{Router, routing::{put, get}};
use std::sync::Arc;
use crate::config::BridgeConfig;

pub struct AppState {
    pub config: Arc<BridgeConfig>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/_matrix/app/v1/transactions/:txn_id", put(handle_transactions))
        .route("/_matrix/app/v1/users/:user_id", get(handle_query_user))
        .route("/_matrix/app/v1/rooms/:room_alias", get(handle_query_room))
        .with_state(state)
    // TODO(task 9 auth): add hs_token auth middleware/layer here
}

// Handlers: implement with hs_token verification + minimal response bodies.
// Log incoming txnId/events for now; real relay in Task 11.
```

Adjust the exact types/generics to what `ruma-appservice-api` 0.16 and `axum` 0.8 actually require (Ref MCP is authoritative). The scaffold above is a starting shape, not a compile-ready template.

### 4.5 `src/main.rs` modification

Replace the `// TODO(task 9): mount real AS transaction routes (appservice.rs).` stub with the actual router mount. Read `main.rs` first to see the exact stub text and `axum::serve` call structure. The change is: replace `Router::new()` with `appservice::router(Arc::new(AppState { config: Arc::new(config) }))` (or equivalent). Add `mod appservice;` to the module declarations.

Keep the `// TODO(task 12)` soft-pause stub unchanged — that's Task 12 scope.

### 4.6 If something is genuinely ambiguous — raise a blocker, do not guess

If a ruma-appservice-api 0.16 type can't be found via Ref MCP, or the hs_token auth shape is ambiguous between the two implementation options, raise a `kind: "blocker"` DQ, commit + push it immediately, and STOP that thread. Do NOT invent type names; do NOT add new deps to resolve ambiguity without a DQ.

### 4.7 validate-pending-laptop DQ entry

After creating/modifying files and committing, append a `validate-pending-laptop` DQ entry. Generate the id + append via `bash scripts/brehon/dq-v3-new-entry.sh` (for id) then `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Fields:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<your worktree branch>",
  "phase_task": 9,
  "commands": [
    "cd services/bridge && cargo check"
  ],
  "question": "Tree-A Task 9: AS transaction server compiles inside services/bridge — laptop runs cargo check.",
  "options": ["pass", "fail"],
  "context": "Task 9 added services/bridge/src/appservice.rs (axum router for PUT /transactions, GET /users, GET /rooms; hs_token auth) and updated src/main.rs to mount it. Deps unchanged from Task 8 (ruma-appservice-api 0.16 + axum 0.8 already in Cargo.toml). VALIDATE: cd services/bridge && cargo check must be GREEN (exit 0). NOT --workspace. NOT --features full.",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

### 4.8 Commit + push discipline
- **Commit subject:** `feat(bridge): AS transaction server — axum routes + hs_token auth (task 9)`
- Include in commit body: one-line for Issue #219 finding, one-line for Issue #465 finding, your MIRROR citation.
- Sequence:
  ```
  git add services/bridge/src/appservice.rs services/bridge/src/main.rs .claude/decision-queue.json
  git commit -m "feat(bridge): AS transaction server — axum routes + hs_token auth (task 9)"
  git push origin <your worktree branch>
  ```
- End the commit body with the `HANDOVER:` YAML trailer below, then **STOP**.

### 4.9 Attribution
- `from: "impl"` on the DQ entry; `answered_by: null`.
- NEVER `answered_by: "advisor"` and NEVER `approved_by` from this session.

## HANDOVER (fill in your real values before final commit)

```yaml
HANDOVER:
  task: m1-a-task-9
  filesCreated:
    - services/bridge/src/appservice.rs
  filesModified:
    - services/bridge/src/main.rs
  keyDecisions:
    - "ruma-appservice-api 0.16 types used: <list the endpoint types you found>"
    - "hs_token auth: <middleware | per-handler extractor>"
    - "Issue #219 whoami: <verified — requires X / not required because Y>"
    - "Issue #465 ip_source: Task 13 docker-compose constraint (no code change here)"
  taskConstraintsForTask13:
    - "Issue #465: ip_source must NOT be set in docker-compose (loopback AS)"
    - "federation-disabled workaround: allow_federation=true + forbidden_remote_server_names=[.*] in Tuwunel config"
    - "never-switch-fork: always use the pinned Tuwunel image (never switch homeserver)"
  notes: "validation = cd services/bridge && cargo check (NOT --workspace, NOT --features full). Worker wrote validate-pending-laptop DQ + stopped. Task 10 (puppet-on-first-contact) requires Task 9 AS client + token setup."
```
