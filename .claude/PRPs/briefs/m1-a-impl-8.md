---
phase: m1-a
role: impl-task
n: 8
plan_task: 8
authored: 2026-06-04
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m1-a
task_number: 8
minimax_trial: NOT eligible — Tree A is greenfield (no in-repo MIRROR sibling), fails §0.1 MIRROR-ref-heavy criterion
related_dq: a192dbab1de8-001
---

# [role:impl-task] m1-a Task 8 — bridge crate skeleton + workspace exclusion

## 1. Role + dispatch line

```
[role:impl-task] m1-a task-8 bridge skeleton — see .claude/PRPs/briefs/m1-a-impl-8.md
```

You are the **impl-task** subagent (Sonnet 4.6 per your frontmatter). Execute plan task 8 from `.claude/PRPs/plans/m1.plan.md` §13 Task 8 (Tree A).

> **⚠️ R8 — THIS IS NOT THE LEMMY WORKSPACE.** `services/bridge/` is a **greenfield, workspace-EXCLUDED** crate with its own `Cargo.toml`, its own error type, and EXTERNAL conventions (`matrix-sdk` / `ruma-appservice-api` / `axum`). **Do NOT apply Lemmy-workspace lessons** here — no `LemmyResult`, no `LemmyError`, no Diesel, no `--features full`, no `#[cfg(feature = "full")]`, no `crates/server/tests/e2e.rs`. Read 1–2 external `matrix-sdk`/`ruma-appservice-api`/`axum` quickstart shapes (via Ref MCP) BEFORE writing — there is no in-repo sibling to mirror.

## 2. Scope

Scaffold the bridge crate and **exclude it from the Brehon workspace**. This is the foundational Tree-A task — every later task (9–13) builds on this crate skeleton + the `exclude` invariant.

**This is pre-Shape-G: write a `validate-pending-laptop` DQ entry, commit + push, then STOP. Do NOT run cargo or docker yourself** (cargo runs on the laptop, never on the daemon — R9).

**Produce (exactly 3 files created, 1 file modified):**

```yaml
creates:
  - services/bridge/Cargo.toml      # package + deps (matrix-sdk, ruma-appservice-api, axum, tokio, reqwest, serde, serde_json, tracing, crate-local error)
  - services/bridge/src/main.rs     # #[tokio::main]; load config; start AS server + soft-pause poller (STUBS — wired in Tasks 9/12)
  - services/bridge/src/config.rs   # BridgeConfig { tuwunel_url, as_token, hs_token, bridge_port, brehon_read_url } from env/file
modifies:
  - Cargo.toml                       # add exclude = ["services/bridge"] to [workspace]
```

**Commit message** (exactly): `feat(bridge): scaffold services/bridge crate + workspace exclude (task 8)`

**Do NOT** in this task:
- **Add `services/bridge` to the workspace `members` array.** It MUST go in `exclude` (story 6 invariant — `cargo build --workspace` must pull ZERO Matrix deps). Adding it to `members` is a **catch-fire** breach. (See §4.0 — the load-bearing constraint.)
- Implement the AS transaction server (Task 9), puppet logic (Task 10), relay (Task 11), provisioning/soft-pause logic (Task 12), or the integration test/registration/docker-compose (Task 13). `main.rs` wires only **stubs** for the AS server + soft-pause poller — enough to compile, with `// TODO(task 9)` / `// TODO(task 12)` markers.
- Apply ANY Lemmy-workspace convention (`LemmyResult`, `LemmyError`, Diesel, `--features full`, `#[cfg(feature="full")]`, the e2e harness) to `services/bridge/**`. R8 — catch-fire.
- Touch any file under `crates/`, `migrations/`, `docs/`, or `.claude/` other than `.claude/decision-queue.json` (your validate-pending entry).
- Run `cargo check` / `cargo build` / `cargo clippy` / `docker` / any build command. Validation is the laptop's job (write-then-stop, R9).
- Commit to `governance-v0` or any branch other than your task worktree branch.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

### 3a. Handover from prior cohort

```yaml
prior_cohort_task: (none — first Tree-A task; Task 8 is the crate-skeleton foundation)
```

> Tree B (Tasks 1–7) shipped separately as m1-b (PR #177, merged). Task 8 consumes NOTHING from Tree B at compile time — it only establishes the bridge crate. The Tree-B `bridge_notify` POST seam (the inbound HTTP trigger this bridge will eventually receive) is consumed in Task 11 (relay), not here.

### MIRROR refs — EXTERNAL conventions only (no in-repo sibling exists — R8)

There is **no in-repo crate to mirror**. Before writing, consult (via `mcp__ref-context__ref_search_documentation` / `ref_read_url`):
- **`matrix-sdk` 0.18** — crate-level docs + an appservice/quickstart example for the `Cargo.toml` dep shape + `#[tokio::main]` main.rs skeleton. (Used heavily in Tasks 9–11; here you only need the dep line + that it builds.)
- **`ruma-appservice-api` 0.16** — crate docs for the dep shape (the AS endpoint types land in Task 9; here just the dep).
- **`axum`** (current stable) — minimal `Router` + `#[tokio::main]` server skeleton (the real routes land in Task 9; here a stub `Router::new()` + bind is enough).
- **`tokio`** — `#[tokio::main]` with `rt-multi-thread` + `macros` features; a `tokio::time::interval` poller stub shape for the soft-pause loop (real logic in Task 12).

### Design docs (the bridge architecture + why Tuwunel)
- `docs/brehon-law-inspired-network/chat1.md` + `chat2.md` — the chat-plane design (bridge daemon shape, AS model, 1:1 DM flow, soft-pause posture).
- `docs/research/matrix-homeserver-selection-2026.md` — the Tuwunel homeserver choice + the four "verify before committing" integration items (relevant for Task 9, context here).
- `docs/brehon-law-inspired-network/expert-review-suite/02-platform-components-lemmy-and-matrix.md` — Lemmy↔Matrix platform component split.

### Plan sections (authoritative)
- `.claude/PRPs/plans/m1.plan.md` §13 Task 8 (ACTION / FILES / IMPLEMENT / MIRROR / GOTCHA / VALIDATE) — **THE contract.** Read the full block.
- `.claude/PRPs/plans/m1.plan.md` §7 R8 (Tree A is NOT the Lemmy harness) + R9 (validate-pending-laptop write-then-stop).
- `.claude/PRPs/plans/m1-a.plan.md` §7 + §12 + §16a story 6 (the workspace-exclusion invariant).

### Lessons (the SHORT list — most Lemmy lessons do NOT apply, R8)
- `.claude/lessons/feedback_read_canonical_before_writing_spec.md` — Tier-2: read 1–2 sibling instances first. For Tree A "sibling" = external `matrix-sdk`/`ruma`/`axum` examples (no in-repo sibling). Cite the example you followed in a code comment / commit body.
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write the DQ entry + push, then STOP. Do NOT run cargo.
- **Explicitly NON-applicable (listed so you don't reach for them):** `feedback_lemmy_error_no_std_error.md`, `feedback_features_full_workspace_only.md`, `feedback_features_full_p_crate_incompatible.md`, `feedback_clippy_test_style.md`, `feedback_lemmy_migration_runner.md`, all Diesel/e2e lessons — **R8: NONE of these apply to `services/bridge/**`.** The bridge has its own toolchain context.

## 4. Constraints

### 4.0 THE workspace-exclusion invariant (story 6 — load-bearing, catch-fire if violated)

Root `Cargo.toml` currently has `[workspace]` (line 34), `members = [...]` (line 35), `resolver = "3"` (line 82), and **NO `exclude` array**. You MUST add:

```toml
[workspace]
members = [ ... ]      # leave UNCHANGED — do NOT add services/bridge here
exclude = ["services/bridge"]
resolver = "3"
```

- The `exclude` array is **mandatory** — absence-from-`members` alone is NOT sufficient. Cargo will still try to resolve a path-dependency or a nested crate under the workspace root unless it's explicitly excluded. (Plan §13 Task 8 GOTCHA.)
- **Story 6 gate:** `cargo tree --workspace 2>/dev/null | grep -c -E 'matrix-sdk|ruma'` MUST return `0`. The laptop verifies this (your validate-pending entry includes it). If `exclude` is missing or wrong, this assertion fails.
- **NEVER add `services/bridge` to `members`** — that pulls matrix-sdk + ruma into the Lemmy workspace build, which is the exact thing M1 forbids (clean-disable invariant). This is a hard **catch-fire** breach (plan §0 + m1-a-bootstrap §7).

### 4.1 `services/bridge/Cargo.toml` — deps (per §13 Task 8 IMPLEMENT)

```toml
[package]
name = "brehon-bridge"   # or a sensible name; the crate is standalone
version = "0.1.0"
edition = "2021"          # match what matrix-sdk 0.18 / ruma 0.16 require; confirm via Ref MCP
publish = false

[dependencies]
matrix-sdk = "0.18"
ruma-appservice-api = "0.16"
axum = "<current stable>"        # confirm exact compatible minor via Ref MCP
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }   # add features the stubs need
reqwest = { version = "<compatible>", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
# crate-local error: anyhow OR a thiserror-based error enum — pick ONE, your call (R8 — own error type, NOT LemmyError)
anyhow = "1"      # OR: thiserror = "1" + a BridgeError enum
```

> **Version pinning:** confirm exact compatible minor versions via Ref MCP (matrix-sdk 0.18 constrains its ruma/axum/reqwest peer versions). If matrix-sdk 0.18's transitive constraints force a specific axum/reqwest minor, honor that — the goal is `cargo check` GREEN inside `services/bridge/`. **If two named deps have an irreconcilable version conflict** (e.g. matrix-sdk 0.18 needs ruma-X but ruma-appservice-api 0.16 needs ruma-Y), raise a `kind: "blocker"` DQ (§4.4) rather than guessing — do NOT downgrade matrix-sdk off 0.18 or switch homeservers (those are pinned decisions per the design docs).

### 4.2 `services/bridge/src/config.rs` — `BridgeConfig`

```rust
// BridgeConfig loaded from env (and/or a config file). Fields per §13 Task 8 IMPLEMENT:
//   tuwunel_url, as_token, hs_token, bridge_port, brehon_read_url
// Use serde for deserialization if loading from a file; std::env::var for env.
// Own error handling (anyhow::Result or your BridgeError) — NOT LemmyResult.
pub struct BridgeConfig {
    pub tuwunel_url: String,
    pub as_token: String,
    pub hs_token: String,
    pub bridge_port: u16,
    pub brehon_read_url: String,
}
// impl BridgeConfig { pub fn from_env() -> anyhow::Result<Self> { ... } }
```

### 4.3 `services/bridge/src/main.rs` — STUBS only

```rust
// #[tokio::main] async fn main() -> anyhow::Result<()> {
//   tracing_subscriber init (optional at this task);
//   let config = BridgeConfig::from_env()?;
//   // TODO(task 9): start the axum AS transaction server (appservice.rs)
//   // TODO(task 12): spawn the soft-pause poller (soft_pause.rs)
//   // For task 8: a minimal axum Router::new() bound to config.bridge_port that
//   //   compiles + would serve (no real routes yet), and/or a tokio::time::interval
//   //   stub loop — enough that `cargo check` is GREEN. Do NOT implement real logic.
//   Ok(())
// }
mod config;
```

The bar for Task 8 is: **the crate compiles (`cd services/bridge && cargo check` green) and the workspace pulls zero Matrix deps.** The AS server + poller are STUBS with TODO markers; real logic is Tasks 9 + 12.

### 4.4 If something is genuinely ambiguous — raise a blocker, do not guess

If a dep version conflict is irreconcilable, or the config-loading shape (env vs file) is unclear from the design docs, raise a `kind: "blocker"` DQ entry (`from: "impl"`, `answered_by: null`), commit + push it immediately (mid-task visibility), and STOP that thread. Generate the id via `bash scripts/brehon/dq-v3-new-entry.sh`. Do NOT downgrade matrix-sdk, switch the homeserver, or add `services/bridge` to `members` to "make it build" — those are pinned design decisions.

### 4.5 validate-pending-laptop DQ entry

After creating all files and committing, append a `validate-pending-laptop` DQ entry. Generate the id + append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending` (avoids the Windows backslash-path class). Fields:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<your worktree branch>",
  "phase_task": 8,
  "commands": [
    "cd services/bridge && cargo check",
    "cargo tree --workspace 2>/dev/null | grep -c -E 'matrix-sdk|ruma'"
  ],
  "question": "Tree-A Task 8: bridge crate compiles inside services/bridge AND workspace pulls zero Matrix deps — laptop runs cargo.",
  "options": ["pass", "fail"],
  "context": "Task 8 scaffolded services/bridge/{Cargo.toml,src/main.rs,src/config.rs} (matrix-sdk 0.18 + ruma-appservice-api 0.16 + axum + tokio; crate-local error; AS-server + soft-pause STUBS with TODO markers) and added exclude=[\"services/bridge\"] to root [workspace]. VALIDATE: (1) `cd services/bridge && cargo check` must be GREEN (exit 0). (2) `cargo tree --workspace | grep -c -E 'matrix-sdk|ruma'` must return 0 (story 6 — exclusion holds; workspace build pulls ZERO Matrix deps). NO --features full (Tree A has no such feature). NOT --workspace cargo check for the bridge — the bridge is workspace-EXCLUDED.",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

> **Note on command 2:** the `grep -c` returns the count as its stdout AND exits non-zero when count is 0 (grep convention: no match → exit 1). The laptop advisor interprets `0` as PASS for the exclusion assertion (zero Matrix deps = correct). The worker just writes the command; the advisor handles the exit-code interpretation. Do NOT wrap it in logic to "fix" the exit code.

### 4.6 Commit + push discipline
- **Commit subject:** `feat(bridge): scaffold services/bridge crate + workspace exclude (task 8)`
- Sequence:
  ```
  git add services/bridge/Cargo.toml services/bridge/src/main.rs services/bridge/src/config.rs Cargo.toml .claude/decision-queue.json
  git commit -m "feat(bridge): scaffold services/bridge crate + workspace exclude (task 8)"
  git push origin <your worktree branch>
  ```
  (Or two commits — code first, then `chore(decision-queue): impl raised validate-pending-laptop DQ — m1-a task 8`.)
- End the commit body with a `LESSON:` trailer if you hit a footgun, and the `HANDOVER:` YAML trailer below.
- Then **STOP**. Do not run cargo or docker.

> **Cargo.lock note:** scaffolding a new excluded crate may generate `services/bridge/Cargo.lock` (excluded crates get their own lockfile). Commit it if created. Do NOT modify the root `Cargo.lock` — the `exclude` ensures the workspace lockfile is untouched by the bridge's deps (if the root `Cargo.lock` changes, the `exclude` is wrong — that's a signal, raise a blocker).

### 4.7 Attribution
- `from: "impl"` on the DQ entry; `answered_by: null` (the laptop advisor mutates it).
- NEVER `answered_by: "advisor"` and NEVER `approved_by` from this session.

## HANDOVER (fill in your real values before final commit)

```yaml
HANDOVER:
  task: m1-a-task-8
  filesCreated:
    - services/bridge/Cargo.toml
    - services/bridge/src/main.rs
    - services/bridge/src/config.rs
  filesModified:
    - Cargo.toml
    - .claude/decision-queue.json
  keyDecisions:
    - "crate name: <name you chose>; error type: <anyhow | thiserror BridgeError>"
    - "dep versions: matrix-sdk <X>, ruma-appservice-api <X>, axum <X>, reqwest <X> (pinned per Ref MCP compatibility)"
    - "exclude = [\"services/bridge\"] added to root [workspace]; members UNCHANGED"
    - "BridgeConfig fields: tuwunel_url, as_token, hs_token, bridge_port, brehon_read_url (loaded from <env|file>)"
    - "main.rs: AS-server stub (TODO task 9) + soft-pause poller stub (TODO task 12); compiles, no real logic"
  notes: "validation = `cd services/bridge && cargo check` (NOT --workspace, NOT --features full) + `cargo tree --workspace | grep -c matrix-sdk|ruma` == 0 (story 6). Worker wrote validate-pending-laptop DQ + stopped. Task 9 (AS transaction server) requires this skeleton + BridgeConfig."
```
