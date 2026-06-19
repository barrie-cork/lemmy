# Brief: m3-core-emergency-mute impl-4 (Task 4)

## §1 Role + dispatch line

`[role:impl-task] m3-core-emergency-mute-task4-emergency-mute-e2e — see .claude/PRPs/briefs/m3-core-emergency-mute-impl-4.md`

## §2 Scope

**Task 4 of plan `.claude/PRPs/plans/m3-core-emergency-mute.plan.md` (the "Docker-gated cross-instance end-to-end test" task).** Add the docker-gated `#[ignore]` integration test that carries the real cross-instance <500ms-at-the-publisher-client signal + the `room_mute_all` chain-row assertion. `requires: 2, 3` — both shipped (phase tip is past `42d5ce9f8`; `mute_all_power_levels` from Task 2 and `Stage::mute_all` from Task 3 are both on the branch).

**Produces (exactly 1 NEW file, ONE commit):**
1. `services/bridge/tests/emergency_mute.rs` — one `#[tokio::test] #[ignore = "requires docker-compose stack"] async fn mute_all_drops_all_publishers_cross_instance_under_500ms() -> anyhow::Result<()>` with **step comments + a `todo!()` body** (pilot-grade; the live stack is Phase-6).

**This is a STUB test — the DoD is that it COMPILES (`--no-run`), NOT that it runs.** The deterministic gate (the Story 1–3 unit tests) already ships in Tasks 1–3 and runs under `cargo-linux.sh test` without Docker. This file is the Phase-6-pilot-grade cross-instance signal — its body is `todo!()` until the live two-instance docker stack exists.

**The step comments (in the `todo!()`-stubbed body) must document, in order:**
1. provision a federated town-hall stage room across **two bridge instances** (chair + N publishers);
2. record a per-publisher publish-active baseline **at the publisher client**;
3. fire mute-all — the `mute_all_power_levels` PUT (cross-instance Matrix authority, Task 2) **and** the local `Stage::mute_all` sweep (Task 3);
4. assert EVERY non-chair publisher's publish right is revoked **measured at the publisher client** within **500ms** (the zero-holder negative invariant, cross-instance);
5. query `governance_log`: assert a `room_mute_all` row with `actor_pseudonym` = the chair **PSEUDONYM** (never `person_id` / `@user:domain`) + `payload.federated == true`.

**Do NOT touch:** any `crates/**` file; any `services/bridge/src/**` file (Tasks 1–3 shipped — `stage.rs`, `room_event_client.rs`, `mute_handler.rs`, `sanction_handler.rs` are all done); any migration; `Cargo.toml` / `Cargo.lock` (NO new dependency — reuse what the sibling `stage_mode.rs` test already imports: `tokio`, `anyhow`); any other file under `services/bridge/tests/`. Do NOT register any `ENTRY_KIND_*` (the const is shipped Phase 2). Do NOT implement the live body — `todo!()` is the contract.

**Branch:** forks from `phase-m3-core-emergency-mute` (Cohort A + clippy-debt + Task 3 + the DQ resolution are all landed; tip is past `33bacdd64`).

## §3 Required reading

- `.claude/PRPs/plans/m3-core-emergency-mute.plan.md` — the Task 4 section (`### Task 4: Docker-gated cross-instance end-to-end test`): follow its IMPLEMENT / MIRROR / GOTCHA / VALIDATE exactly. The GOTCHA is load-bearing: the `#[ignore]` test does NOT run under `cargo-linux.sh test` — the DoD is that it **compiles** (`--no-run`); the measurement is at the **publisher client**, not the server.
- **MIRROR (copy the shape verbatim):**
  - `services/bridge/tests/stage_mode.rs:1-23` — the ignore-stub shape: file-header doc comment (how to run with docker-compose), one `#[tokio::test] #[ignore = "requires docker-compose stack"] async fn …() -> anyhow::Result<()>` with numbered step comments + a `todo!("implement against live docker-compose stack (Phase-6 pilot grade)")` body. **Mirror this file's exact structure — same header comment style, same `anyhow::Result<()>` return, same `#[ignore]` reason string.**
  - `services/bridge/src/sanction_handler.rs:474-482` — the `#[ignore]` live-test convention (the `// Mirror: …` comment style + `todo!()`).
- **Lessons (mandatory, §2.4 file-class injection — new test under `services/bridge/tests/**` + bridge code):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — all `services/bridge` cargo via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Docker `rust:1.95`); NEVER Windows-local (`ruma-common` E0119).
  - `feedback_linux_compile_proof_is_a_gate.md` — this task writes a `validate-pending-laptop-linux` DQ; `bm-pr` gates on `result:pass`.
  - `feedback_validate_pending_laptop_write_then_stop.md` — write the DQ and STOP; the laptop advisor runs all cargo/Docker.
  - `feedback_clippy_test_style.md` — the test uses `anyhow::Result<()>` / `?` (mirror `stage_mode.rs`), no `unwrap`/`expect` (but the body is `todo!()` so this is moot until Phase-6).

## §4 Constraints

- **ONE commit, 1 NEW file** — `feat(rtc): docker-gated cross-instance emergency-mute e2e stub (task 4)`.
- **STUB ONLY — `todo!()` body is the contract.** The DoD is that `emergency_mute.rs` **compiles** (`--no-run`), NOT that it runs. Do NOT implement the live cross-instance logic — that is Phase-6 pilot work. The step comments (1–5 above) ARE the deliverable alongside the compiling `#[ignore]` shell. Per the plan GOTCHA.
- **`#[ignore = "requires docker-compose stack"]`** is mandatory — without it, a bare `cargo test` would try to run the `todo!()` and panic. Mirror `stage_mode.rs:12`.
- **ADR-015 (pseudonyms, in the step comments — load-bearing even in a stub):** step 5's comment MUST say `actor_pseudonym` = chair **PSEUDONYM** (never `person_id` / `@user:domain`). A future implementer reads these comments as the contract — an identity-shaped comment here seeds an ADR-015 violation in Phase-6. **DoD: `rg -i 'person_id|@.*:.*domain|real.?name' services/bridge/tests/emergency_mute.rs` returns nothing identity-shaped.** Per §10.6.
- **The measurement point is the PUBLISHER CLIENT, not the server** (Success-Criteria line 141 + PRD line 198) — step 2 + step 4 comments MUST say "measured at the publisher client". This is the cross-instance-correctness pin: a server-side assertion would pass even if the federated PUT never propagated.
- **PRD fallback documented, NOT silently dropped:** the step comments note that if cross-instance <500ms proves infeasible at the Phase-6 pilot, the fallback is best-effort cross-instance SLA + in-instance <500ms guarantee (via the `Stage::mute_all` LiveKit sweep), and a DQ surfaces before declaring the criterion failed. Per the plan GOTCHA.
- **NO new dependency** — `Cargo.toml`/`Cargo.lock` unchanged. The mirror `stage_mode.rs` already pulls `tokio` + `anyhow` as dev-deps; reuse them. If you think you need a new one, STOP and raise a `kind: "blocker"` DQ.
- **Bridge compiles on Linux ONLY.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test emergency_mute --no-run"]`. **NOTE: this is an INTEGRATION test (`tests/emergency_mute.rs`), so the validate uses `--test emergency_mute --no-run` — NOT `--bins` (that form is for `src/` unit tests).** `--no-run` is load-bearing: the `#[ignore]` test compiles but must NOT execute (no live stack). Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself. Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push on the worker branch, stop.
- **Handover:** deliver the HANDOVER block inline in your task output (do NOT write a `.claude/PRPs/handovers/` file — blocked by the sensitive-file guard in the Junior worktree context).
- End the commit body with a `LESSON:` trailer if you find anything durable.

## §3a Handover from prior tasks (Tasks 1–3, all shipped)

- **Task 1 (#726, `governance_log.rs`):** binary `RoomEventPayload.federated: Option<bool>` — the field your step-5 governance_log assertion reads.
- **Task 2 (#727, `mute_handler.rs`):** `mute_all_power_levels(state, case_id)` (async, `#[allow(dead_code)]` until wired) + `compute_mute_all_override` (pure) + `sanction_handler::{get_power_levels, put_power_levels}` widened to `pub(crate)`. **This is the CROSS-INSTANCE Matrix power-level authority — step 3's PUT.**
- **Task 3 (#728, `stage.rs` + `room_event_client.rs`):** `Stage::mute_all(publishers, federated, sink)` — the LOCAL LiveKit revoke sweep + the single `room_mute_all` EmitIntent (`actor_pseudonym=chair`, `federated=Some`). **This is step 3's local sweep + the source of the `room_mute_all` chain row step 5 asserts.** The marquee unit test `mute_all_revokes_all_publishers` already proves the zero-holder invariant deterministically; this e2e test is the cross-instance, real-client version of the same invariant.
- **Validation env note:** unit-test filters need `--bins`; but THIS task's test is an integration test → use `--test emergency_mute --no-run`. (The `--bins` vs `--test` distinction bit Task 2/3 — your validate command list already reflects it.)

## §5 HANDOVER (worker fills at task end — return inline in task output)

```yaml
HANDOVER:
  task: m3-core-emergency-mute-task4
  filesModified: [services/bridge/tests/emergency_mute.rs]
  keyDecisions:
    - "emergency_mute.rs: one #[ignore] #[tokio::test] mute_all_drops_all_publishers_cross_instance_under_500ms; todo!() body; 5 numbered step comments (provision 2 instances / baseline at publisher client / fire power-level PUT + local sweep / assert all revoked <500ms at client / assert room_mute_all row chair-pseudonym + federated)"
    - "ADR-015 pseudonym pin in step-5 comment; measurement-at-publisher-client pin in step 2+4; PRD fallback documented"
    - "no new dependency; mirrors stage_mode.rs:1-23 shape"
  validate_dq: <composite-id of the validate-pending-laptop-linux DQ>
  notes: "<whether emergency_mute.rs compiled first-try under --no-run; any import the mirror stage_mode.rs needed that wasn't obvious>"
```
