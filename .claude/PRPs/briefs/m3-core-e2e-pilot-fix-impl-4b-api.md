# Brief: m3-core-e2e-pilot fix-impl Task-4b — correct livekit-api 0.4.24 API usage in `emergency_mute.rs`

## §1 Role + dispatch line

`[role:impl-task] m3-core-e2e-pilot-fix-task4b-livekit-api-surface — see .claude/PRPs/briefs/m3-core-e2e-pilot-fix-impl-4b-api.md`

You are the **impl-task** subagent (Sonnet 4.6). This is a **narrow fix-impl** (≤2 edits, ONE file: `services/bridge/tests/emergency_mute.rs`). NO dep change, NO logic redesign. Runs ALONE (cross-lane cap = 2; you are the only task).

## §2 Scope — two API-usage bugs the `=0.7.7` pin uncovered

**Context (advisor-diagnosed, user-ratified):** the prior fix pinned `livekit-protocol = "=0.7.7"` (committed `da5cf0f9d`), which fixed the dependency conflict — the `livekit-api 0.4.24` crate now compiles. But that uncovered **2 real API-usage bugs in the test code** that were previously masked by the dep failure. Both are because the original Task-4 author **assumed** the `livekit-api 0.4.24` API surface instead of reading it. Your job: correct them by **reading the ACTUAL `livekit-api 0.4.24` API first.**

**HARD RULE — READ THE REAL API BEFORE EDITING (do NOT guess; guessing caused this):**
The crate source is in your Docker build env after the first `cargo-linux.sh` invocation, OR fetch it. Before editing, run (capture to a file, do NOT pipe cargo):
```bash
# Make the crate source available + locate the two symbols:
scripts/brehon/cargo-linux.sh fetch --manifest-path services/bridge/Cargo.toml > .claude/fetch-4b.log 2>&1
# Then, inside the same Docker image, grep the registry source for:
#   - where ParticipantPermission ACTUALLY lives (livekit_api::? or livekit_protocol::?)
#   - the real signature of RoomClient::with_api_key (does it return RoomClient or Result<RoomClient>?)
```
Use `scripts/brehon/cargo-linux.sh` to run a grep inside the image against `/usr/local/cargo/registry/src/*/livekit-api-0.4.24/` and `.../livekit-protocol-0.7.7/`. Find the TRUTH; then edit. If you cannot locate the crate source in the image, raise a `kind: blocker` DQ — do NOT guess a second time.

**The 2 bugs (current state in `tests/emergency_mute.rs`):**

1. **`tests/emergency_mute.rs:25`** — `error[E0432]: unresolved import`:
   ```rust
   use livekit_api::proto::ParticipantPermission;   // WRONG — `proto` is not a module in livekit_api 0.4.24
   ```
   **Fix:** correct the import path to wherever `ParticipantPermission` ACTUALLY lives in the dependency graph. Strong candidate: `livekit_protocol::ParticipantPermission` (the proto types live in the `livekit-protocol` crate, which is now a direct dev-dep via the `=0.7.7` pin) — **but VERIFY by grep before using it.** If `ParticipantPermission` is not actually referenced anywhere in the test body after you inspect (i.e. the import was speculative/unused), DELETE the import line entirely rather than re-pointing it. Check: `grep -n ParticipantPermission services/bridge/tests/emergency_mute.rs` — if the only hit is the `use` line, remove it; if it's used in the body, fix the path.

2. **`tests/emergency_mute.rs:141`** — `error[E0599]: no method named map_err found for struct RoomClient`:
   ```rust
   let lk_client = RoomClient::with_api_key(
       &livekit_admin_url, &livekit_api_key, &livekit_api_secret,
   )
   .map_err(|e| anyhow::anyhow!("RoomClient::with_api_key: {e}"))?;   // WRONG — with_api_key returns RoomClient, not Result
   ```
   **Fix:** `with_api_key` returns `RoomClient` directly (infallible constructor). Drop the `.map_err(...)?`:
   ```rust
   let lk_client = RoomClient::with_api_key(
       &livekit_admin_url, &livekit_api_key, &livekit_api_secret,
   );
   ```
   **VERIFY** the real return type by reading the signature first — if 0.4.24's `with_api_key` actually DOES return a `Result` (compiler says otherwise, but confirm), keep error handling but use the correct shape. The compiler error is authoritative: "method not found in RoomClient" means the receiver is `RoomClient`, not `Result<RoomClient, _>`, so `.map_err` is wrong.

**Do NOT:**
- Touch any file other than `services/bridge/tests/emergency_mute.rs`.
- Change the measurement semantics — the R-PUBCLIENT publisher-client `<500ms` logic (`t0..elapsed` around `update_participant`) stays EXACTLY as-is. You are fixing the CLIENT-handle construction + an import, not the measurement.
- Touch Cargo.toml / Cargo.lock (the `=0.7.7` pin stays; do NOT re-pin or bump).
- Touch any other test, src, migration, const, registry, or plan.
- Guess the API a second time. READ IT.

**Branch:** forks from `phase-m3-core-e2e-pilot` (current tip ≥ `ba7d5bfcf`; confirm at task spawn).

## §3 Required reading

- **This brief** + the 2 compiler errors above (authoritative).
- **`services/bridge/tests/emergency_mute.rs`** lines 1-50 (imports) + 130-165 (RoomClient construction + the measurement loop you must NOT disturb).
- **The real `livekit-api 0.4.24` source** (grep in the Docker image — mandatory per the HARD RULE).
- **Lessons (mandatory):**
  - `feedback_lemmy_error_no_std_error.md` — error-shape discipline if you adjust a `?`/`map_err`.
  - `pattern_test_against_reality_not_syntax.md` + `feedback_verify_automated_reviewer_claims_against_compiler.md` — the compiler is the source of truth for the API shape; verify against it, don't assume.
  - `feedback_bridge_validates_on_linux_not_windows.md` + `feedback_validate_pending_laptop_write_then_stop.md` — bridge cargo is Linux-only; you run the grep/fetch in Docker but the VALIDATION (check/clippy/test --no-run) is the laptop's — write the DQ + STOP.

## §4 Constraints

- **READ-API-FIRST is load-bearing (CATCH-FIRE on re-guess):** this is fix-cycle 2 for Task-4. The first version of this test guessed the API and was wrong twice (E0432 + E0599). You MUST ground both fixes in the actual 0.4.24 source. **Why:** a 3rd same-class failure trips the cycle-count hard-refusal and burns the phase. DoD: cite the grep hit (file:line in the livekit-api/livekit-protocol source) for BOTH the corrected import path AND the `with_api_key` return type, in your HANDOVER notes.
- **Measurement untouched (R-PUBCLIENT):** `grep -n 'Instant::now\|t0.elapsed\|update_participant' services/bridge/tests/emergency_mute.rs` must return the SAME lines after your edit as before (you only touch the import on :25 and the constructor on :141). The `<500ms` publisher-client measurement is the marquee — do not disturb it.
- **ONE commit** — `fix(rtc): correct livekit-api 0.4.24 API usage in emergency_mute.rs (E0432 import + E0599 with_api_key) (task 4b)`. `LESSON:` trailer: `LESSON: livekit-api 0.4.24 — ParticipantPermission lives in <verified path>; RoomClient::with_api_key is infallible (returns RoomClient, not Result).`
- **NO daemon cargo for validation.** Write ONE DQ (`bash scripts/brehon/dq-v3-new-entry.sh` for the id, `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending`):
  ```
  kind: "validate-pending-laptop-linux"
  branch: "phase-m3-core-e2e-pilot"
  phase_task: 4
  commands: [
    "scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml",
    "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings",
    "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test emergency_mute --no-run"
  ]
  result: null
  log_slice: null
  failed_commands: null
  ```
  You MAY run `cargo-linux.sh fetch` + grep-in-Docker for the API READ (that is research, not validation, and does not OOM — it's a fetch + grep, no compile). Do NOT run the check/clippy/test validation yourself — that's the laptop's. Commit + push the DQ on the worker branch, then **STOP**.
- **Clean up:** delete `.claude/fetch-4b.log` + any DQ fragment file after appending (do NOT commit them).
- **DQ mid-task discipline:** any blocker (e.g. can't locate crate source) → `kind: "blocker"` DQ, commit + push, stop. Do NOT guess.
- **Handover:** inline (no handover file).

## §5 HANDOVER (worker fills at task end — return inline)

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-fix-task4b-livekit-api-surface
  filesModified: [services/bridge/tests/emergency_mute.rs]
  apiVerification:
    - "ParticipantPermission real path: <cite livekit-*/src/*.rs file:line you grepped>"
    - "RoomClient::with_api_key real return type: <cite signature file:line> (infallible RoomClient / Result<RoomClient>)"
  keyDecisions:
    - "Line 25 import: <re-pointed to X | deleted because unused>"
    - "Line 141: dropped .map_err(...)? because with_api_key returns RoomClient directly <OR kept Result handling because it actually returns Result>"
    - "R-PUBCLIENT measurement (t0..elapsed around update_participant) UNTOUCHED"
    - "Cargo.toml/lock UNTOUCHED (=0.7.7 pin preserved)"
  validate_dq_linux: <id>
  notes: "<confirm you READ the real API (cite grep hits) before editing; confirm only emergency_mute.rs touched; confirm measurement lines unchanged; confirm no Cargo.toml/lock edit; confirm fetch-4b.log + dq-frag deleted>"
```
