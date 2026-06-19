# Brief: m3-core-stage-mode impl-4 (Task 4 — 30s grace auto-revoke + next-promote boundary)

## §1 Role + dispatch line

`[role:impl-task] m3-core-stage-mode-task4-30s-grace-boundary — see .claude/PRPs/briefs/m3-core-stage-mode-impl-4.md`

## §2 Scope

**Task 4 of plan `.claude/PRPs/plans/m3-core-stage-mode.plan.md` (lines 526–558).** Wire a **30s grace timer** into `stage.rs`: a promoted-but-not-activated speaker is **auto-revoked at 30s** and the next queued watcher is promoted. This is the **load-bearing boundary** (plan R7) — the test MUST fire the 30s boundary via `tokio::time` virtual-time, NOT a happy-path assert.

`requires: task 3` (satisfied — Task 3 `Stage` machine + `GrantSink` landed on `phase-m3-core-stage-mode` tip `806f0d3b6`). Task 4 EXTENDS the Task-3 `Stage`.

**Produces (exactly 1 file modified, ONE commit):**
1. **MODIFY** `services/bridge/src/stage.rs`:
   - `const GRACE_SECS: u64 = 30;`
   - Wire the grace timer per §10.7: a `tokio::time::sleep(Duration::from_secs(GRACE_SECS))` inside a `tokio::select!` raced against an activation signal; on grace-elapsed-without-activation → `on_grace_expired(p, sink)` → `RevokePublish(p)` + `promote_next` (the next FIFO head). The `on_grace_expired` signature was declared in Task 3 (§10.6) — wire its body + the timer now.
   - **Unit test (deterministic, the §16a Story 2 DoD):** `#[tokio::test(start_paused = true)]` `grace_no_activate_auto_revokes_and_promotes_next` — promote W1 (W2 queued), `tokio::time::advance(Duration::from_secs(30)).await` WITHOUT activating W1, assert the test `GrantSink` recorded `RevokePublish(W1)` **THEN** `GrantPublish(W2)`.
   - **Contrast test:** promote W1, `on_activate(W1)` **before** 30s, advance 30s, assert **NO** revoke fired (activation cancels the grace).

**SEAM NOTE (read carefully — likely the one judgment call):** Task 3's `promote_next` is **synchronous** (`fn promote_next(&mut self, sink, conn) -> Result<()>`). The grace timer is **async** (`tokio::select!` / `tokio::time::sleep`). Do NOT force `async` onto the existing sync `promote_next` signature if that breaks the Task-3 callers/tests. The clean approaches (pick whichever fits the Task-3 shape with least churn):
   - (a) keep `promote_next` sync; add a **separate async** method (e.g. `run_grace(&mut self, p, sink)` or a `promote_next_with_grace`) that does the `select!` and calls the sync `promote_next` for the actual promotion; OR
   - (b) the grace `select!` lives in an async helper the test drives directly, with `promote_next` unchanged.
   The 6 existing Task-3 tests MUST still pass. If you cannot wire the timer without changing a Task-3 public signature that has callers, raise a `kind: "blocker"` DQ rather than guessing.

**Do NOT:**
- Use `std::thread::sleep` or any wall-clock sleep — the timer MUST be `tokio::time` (cancellable + virtual-time-controllable). A wall-clock sleep makes the test wait 30 real seconds and fails R7.
- Write a test that only asserts a successful promote — the DoD is the 30s-no-activate boundary firing `RevokePublish` THEN `GrantPublish(next)`.
- Build the real LiveKit `GrantSink` adapter, the `room_event_client` (Task 5), or chair dual-sourcing (Task 6).
- Touch any `crates/**`, any other `services/bridge/**` file, any migration, `Cargo.toml`/`Cargo.lock` (tokio is already a bridge dep).

**Branch:** forks from `phase-m3-core-stage-mode` (current tip `806f0d3b6` — has Tasks 1+2+3).

## §3 Required reading

- `.claude/PRPs/plans/m3-core-stage-mode.plan.md` — Task 4 (526–558), **§10.7** (the `#[tokio::test(start_paused = true)]` + `tokio::time::advance(30s)` virtual-time pattern — the canonical test shape to mirror), §8 flow line 109-110 (`promote → 30s grace → on no-activate auto-revoke + promote next`).
- `services/bridge/src/stage.rs` (the WHOLE file, on phase tip `806f0d3b6`) — the Task-3 `Stage`/`SeatState`/`GrantCmd`/`GrantSink`/`promote_next`/`on_activate`/`on_grace_expired` (declared) + the 6 existing tests (mirror their `GrantSink`-recorder test idiom for the new grace tests).
- **Lessons (mandatory):**
  - `feedback_build_what_tests_exercise.md` — the grace test must EXERCISE the real 30s-no-activate flow (advance virtual clock, assert the `RevokePublish`→`GrantPublish` order), NOT a happy-path promote.
  - **Bridge file-class (§2.4 — `services/bridge/**`):**
    - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo runs **Linux ONLY** via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml`.
    - `feedback_linux_compile_proof_is_a_gate.md` — write a `validate-pending-laptop-linux` DQ; laptop runs `cargo-linux.sh`; `bm-pr` gates on `result:pass`.
    - `feedback_clippy_test_style.md` — no `unwrap`/`expect`/`unwrap_or_default` (Task 3 just got bitten by `unwrap_or_default` — use `?` + `.context()`); tests return `Result<()>` / use `anyhow`.

## §4 Constraints

- **ONE commit, 1 file** — `feat(rtc): 30s grace auto-revoke + next-promote boundary (task 4)`.
- **`tokio::time` ONLY (R7 load-bearing):** the grace timer is `tokio::time::sleep` in a `tokio::select!` against an activation signal. NEVER wall-clock. The test uses `#[tokio::test(start_paused = true)]` + `tokio::time::advance(Duration::from_secs(30)).await`.
- **The boundary test is the DoD** — `grace_no_activate_auto_revokes_and_promotes_next` asserts `RevokePublish(W1)` THEN `GrantPublish(W2)`; the contrast test asserts activation-before-30s cancels the grace (NO revoke). A successful-promote-only test does NOT satisfy §16a Story 2.
- **Don't break Task-3's 6 tests** — keep `promote_next`/`on_activate` working for their existing callers. Reconcile the sync/async seam per the SEAM NOTE; blocker-DQ if it forces a Task-3 signature break.
- **No `unwrap_or_default`/`unwrap`/`expect`** (clippy `-D warnings` — Task 3 fix-impl-2 was exactly this; don't repeat). Propagate with `?` + `.context()`.
- **ADR-015 (pseudonyms-only):** the grace timer operates on pseudonym strings (the FIFO entries) — never `person_id`/username.
- **NO new dependency** — `tokio` (with `time` feature) is already a bridge dep. If `tokio`'s `time`/`macros`/`test-util` feature is missing for `start_paused`, that is a `Cargo.toml` feature-flag question — raise a `kind: "blocker"` DQ (do NOT add a new crate; a feature toggle on an existing dep may be needed and is a scope call).
- **Bridge compiles on Linux ONLY.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml stage"]`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP**. Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.
- End the commit body with a `LESSON:` trailer if you find anything durable.

## §3a Handover from prior cohort

Task 3 (#711/#712) landed `stage.rs` on phase @ `806f0d3b6` — the full chair-seat + persisted FIFO + mic-pass machine + 6 tests (incl. the marquee `fifo_mic_pass_in_sequence`). `promote_next` leaves the promoted participant at `SeatState::Promoted` awaiting activation **with NO timer** — that's deliberate; Task 4 adds the 30s timer. `on_grace_expired(p, sink)` was declared in Task 3 per §10.6 but its body/timer-wiring is YOUR job. **Watch the sync/async seam** (Task-3 `promote_next` is sync; the grace timer is async — see §2 SEAM NOTE). Task 3's fix-impl-2 was a clippy `unwrap_or_default` bust — do NOT reintroduce any disallowed-method.
