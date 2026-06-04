# Brief: m1-a impl Task 13 — integration test + registration + docker-compose

## 1. Role + dispatch line

```
[role:impl-task] m1-a task-13 integration-test-registration-compose — see .claude/PRPs/briefs/m1-a-impl-13.md
```

Model: `claude-sonnet-4-6` | Effort: medium

## 2. Scope

**Task 13** from `m1.plan.md §13`: wire Tuwunel + bridge in docker-compose and add the docker-compose-gated round-trip integration test.

### CREATES:
- `services/bridge/registration.yaml` — AS registration file (as_token, hs_token, user/alias namespaces). Used by Tuwunel to load the bridge AS.
- `services/bridge/docker-compose.yml` — Tuwunel (pinned image) + bridge side-by-side; federation-disabled posture; `ip_source` NOT set (loopback AS, fixes issue-#465); non-host-network handling.
- `services/bridge/tests/dm_round_trip.rs` — docker-compose-gated integration test:
  - Marked `#[ignore]` on individual test fns (so bare `cargo test` skips without docker stack).
  - Brings up stack, sends text+image+voice Brehon→Matrix, asserts delivery < 3s (criterion #1).
  - Exercises enable→disable→enable soft-pause via HTTP to brehon_read_url mock (criterion #4).
  - Exercises admin-panel restart-persistence fixture (criterion #2).

### MODIFIES:
- `services/bridge/Cargo.toml` — add `[dev-dependencies]` block with `tokio-test` (or `tokio::test` macro dep) and `reqwest` (already in deps, no dup needed). Only add what the integration test ACTUALLY imports — no unused dev-deps.
- `services/bridge/src/appservice.rs` — add the HTTP route for `POST /admin/provision-room` (calls `provision::create_community_room`) to the router. Task 12 left provision.rs as a function module; Task 13 wires it into the axum router.

### Does NOT touch:
- `services/bridge/src/provision.rs` — function is already correct; only the router registration in `appservice.rs` changes.
- `services/bridge/src/soft_pause.rs`, `relay.rs`, `puppet.rs`, `config.rs` — read for context, do NOT modify.
- Anything under `crates/` — R8 hard boundary: Tree A is workspace-EXCLUDED.
- `services/bridge/src/main.rs` — already wired from Task 12; do NOT modify.

### What to PRODUCE:

**`registration.yaml`**:
```yaml
id: brehon-bridge
url: http://bridge:8080
as_token: <matches BRIDGE_AS_TOKEN env in docker-compose>
hs_token: <matches BRIDGE_HS_TOKEN env in docker-compose>
sender_localpart: brehon
namespaces:
  users:
    - exclusive: false
      regex: "@brehon_.*"
  aliases:
    - exclusive: false
      regex: "#brehon_.*"
  rooms: []
```
Use a fixed dev-only token pair (e.g. `as_token: "brehon-as-dev-token-01"`, `hs_token: "brehon-hs-dev-token-01"`) — these are for the local docker-compose stack only, NOT production. Comment them as dev-only.

**`docker-compose.yml`**:
- Service `tuwunel`: use image `ghcr.io/matrix-org/conduit:next` or `matrixconduit/matrix-conduit:latest` (conduit/tuwunel — pick a stable pinned tag, not `:latest`; check ghcr.io/element-hq/conduit or similar for the Tuwunel fork). **IMPORTANT:** verify the correct image name at authoring time against known Tuwunel/Conduit distributions. Federation-disabled posture: set `CONDUIT_ALLOW_FEDERATION=false` or equivalent env for the image used. `ip_source` NOT set (do not set `CONDUIT_IP_SOURCE` — leave absent so loopback AS works per issue-#465).
- Service `bridge`: build from `services/bridge/` Dockerfile (you will NOT create a Dockerfile in Task 13 — use a placeholder build context or note that a Dockerfile is Task 15 scope; alternatively use `image: rust:1.95` with a command override if simpler for the test harness). The integration test exercises the stack by calling the bridge directly, so the docker-compose just needs to network them.
- Shared docker network for bridge↔tuwunel communication.

**ALTERNATIVE for docker-compose bridge service:** If creating a proper Dockerfile is too involved for this task, the `dm_round_trip.rs` test can instead start the bridge binary directly via `Command::new("cargo").args(["run", ...])` in the test setup — using only Tuwunel in docker-compose. Whichever approach is simpler and passes the validate-pending-laptop test.

**`dm_round_trip.rs`**:
```rust
// Integration test: docker-compose-gated DM round-trip (Task 13).
// Run with: cd services/bridge && docker compose up -d && cargo test --test dm_round_trip -- --ignored && docker compose down
// Do NOT run with bare `cargo test` — tests are #[ignore]'d to require explicit invocation.

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn dm_text_round_trip() {
    // 1. Wait for Tuwunel to be ready (HTTP poll /_matrix/client/v3/versions)
    // 2. Send a text DM via the bridge relay endpoint
    // 3. Assert response arrives at matrix-sdk client within 3s
    todo!("implement when docker-compose stack is wired")
}

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn soft_pause_enable_disable_cycle() {
    // Criterion #4: enable → disable → enable cycle via brehon_read_url mock
    todo!("implement soft-pause cycle test")
}

#[tokio::test]
#[ignore = "requires docker-compose stack"]
async fn restart_persistence() {
    // Criterion #2: admin-panel restart-persistence
    todo!("implement restart fixture")
}
```
**M1 scope note:** The tests may be `todo!()` stubs for M1 — the key deliverable is the structural wiring (registration.yaml, docker-compose.yml, test file with correct structure + #[ignore] gates). The test execution validate-pending-laptop step runs `cargo test --test dm_round_trip -- --ignored` which will compile + skip all #[ignore] tests (exit 0 if compilation succeeds). Full test IMPLEMENTATION is M2 scope unless the plan §13 GOTCHA says otherwise — **read §13 Task 13 GOTCHA before deciding**; if the plan says implement them now, implement them.

**`appservice.rs` change** — add provision route to router:
```rust
// In the router() function, add:
.route("/admin/provision-room", post(handle_provision_room))

// New handler:
async fn handle_provision_room(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let room_alias = body.get("room_alias")
        .and_then(|v| v.as_str())
        .unwrap_or("brehon-default");
    match provision::create_community_room(&state.config, room_alias).await {
        Ok(room_id) => (axum::http::StatusCode::OK, Json(serde_json::json!({"room_id": room_id}))).into_response(),
        Err(e) => {
            tracing::error!(err = %e, "provision failed");
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response()
        }
    }
}
```
Import `use axum::response::IntoResponse;` and `use axum::Json;` and `use provision;` at the top of `appservice.rs`.

### VALIDATION (write-then-stop):
Write the `validate-pending-laptop` DQ entry with:
```json
{
  "kind": "validate-pending-laptop",
  "commands": [
    "cd services/bridge && docker compose up -d",
    "cd services/bridge && cargo test --test dm_round_trip -- --ignored",
    "cd services/bridge && docker compose down"
  ],
  "phase_task": 13
}
```
Then commit + push. Do NOT run the docker-compose commands yourself on the daemon.

## 3. Required reading (read BEFORE writing any code)

1. `.claude/PRPs/plans/m1.plan.md` §13 Task 13 (full IMPLEMENT + GOTCHA block) — primary spec. Pay attention to the GOTCHA about #[ignore] + whether tests should be todo! stubs or implemented.
2. `services/bridge/src/appservice.rs` on `phase-m1-a` — current `AppState`, router, and existing handlers to understand the axum pattern to follow for the provision route.
3. `services/bridge/src/provision.rs` on `phase-m1-a` — `create_community_room` signature.
4. `services/bridge/Cargo.toml` on `phase-m1-a` — existing deps; only add dev-deps that the test file ACTUALLY imports.
5. `services/bridge/src/relay.rs` + `services/bridge/src/config.rs` on `phase-m1-a` — read for context (docker-compose env var names, AS token env vars).
6. `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write DQ then STOP.
7. `.claude/lessons/feedback_read_canonical_before_writing_spec.md` — read sibling modules before authoring new ones.

### R8 reminder (non-negotiable):
- This crate uses `anyhow` errors, NOT `LemmyError` or `LemmyResult`.
- No Diesel, no `lemmy_*` imports, no `--features full`.
- Cargo runs on the laptop (validate-pending-laptop), NOT on the daemon.

## 4. Constraints

1. **R8 boundary (catch-fire):** NEVER import from `crates/`, `lemmy_*`, `LemmyResult`, `LemmyError`, or Diesel.
2. **Workspace exclusion:** `services/bridge/` is in `exclude` in root `Cargo.toml`. Do NOT add it to `members`. Do NOT run `cargo check --workspace`.
3. **validate-pending-laptop — write then stop:** Write the DQ entry with the three-command array above, commit + push, then stop. Do NOT run docker-compose or cargo on the daemon.
4. **#[ignore] all integration tests:** bare `cargo test` inside `services/bridge/` must NOT attempt to run dm_round_trip tests. Use `#[ignore = "requires docker-compose stack"]` on every test function.
5. **DQ mid-task push:** After writing the validate-pending-laptop DQ entry, immediately: `git add .claude/decision-queue.json && git commit -m "chore(decision-queue): impl raised DQ <id> — task-13 validate-pending-laptop" && git push origin <current-branch>`.
6. **Attribution:** NEVER write `answered_by: "advisor"`. Use `answered_by: "impl-self-resolved"` for self-resolved blockers only.
7. **base_branch is `phase-m1-a`** — branch from `phase-m1-a` at `0b026cf0c`. All commits land on your worker branch.
8. **No new dependencies beyond what the test file uses.** If the integration test stubs are all `todo!()`, no new dev-deps are needed. Add only what's actually imported.
9. **dev-only tokens in registration.yaml:** Comment clearly `# dev-only — NOT for production`. Use the same fixed tokens in both registration.yaml and docker-compose.yml env vars.
10. **Federation-disabled posture:** Tuwunel must start with federation disabled (`allow_federation: false` or equivalent). Do NOT expose federation ports.
