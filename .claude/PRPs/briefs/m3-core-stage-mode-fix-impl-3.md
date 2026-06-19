# Brief: m3-core-stage-mode fix-impl-3 (Task 4 tokio time/test-util feature gate)

## §1 Role + dispatch line

`[role:impl-task] m3-core-stage-mode-fix-impl3-tokio-time-feature-gate — see .claude/PRPs/briefs/m3-core-stage-mode-fix-impl-3.md`

## §2 Scope

**Apply the advisor's resolution of blocker DQ `6302ef3b3382-001`** (raised by Task 4 #713). Task 4's `stage.rs` is committed (`53b8c5612`) and correct — it uses `tokio::time::sleep` in `run_grace` (runtime) + `#[tokio::test(start_paused = true)]` / `tokio::time::advance` in the two grace tests (test-only). The bridge's `tokio` dep currently has only `features = ["rt-multi-thread", "macros"]` — both `time` and `test-util` are missing, so the crate does not compile. This is the ONLY blocker; the code is done.

**ADVISOR DECISION (2026-06-19): option-a — split runtime vs test feature.**
- `time` is a genuine RUNTIME dependency (`run_grace` calls `tokio::time::sleep` in the production cancel-future path) → goes in `[dependencies]`.
- `test-util` (source of `pause`/`advance`/`start_paused`) has ZERO production callsites — only the two virtual-time tests use it → goes in `[dev-dependencies]`.
- option-b (both in `[dependencies]`) was rejected: shipping `test-util` into the prod binary alters runtime timer semantics and is a reviewer red-flag. The clean split is idiomatic.

This is a feature-toggle on an EXISTING dep (`tokio` is already a bridge dep) — NOT a new crate, so the plan's "no new dependency" watchpoint is satisfied. Enabling `test-util` fulfils plan §10.7's mandated `start_paused`/`advance` virtual-time test pattern.

**Produces (exactly 1 file edit, ONE commit):**
1. `services/bridge/Cargo.toml` — two changes:
   - **`[dependencies]` tokio line (currently line ~14):** add `"time"` to the features array:
     ```toml
     # before:
     tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
     # after:
     tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }
     ```
   - **Add a `[dev-dependencies]` section** (none exists yet — add it after the `[dependencies]` block, before any other section). Enable `test-util` on tokio there:
     ```toml
     [dev-dependencies]
     tokio = { version = "1", features = ["test-util"] }
     ```
     Cargo unions feature sets across `[dependencies]` + `[dev-dependencies]` for the test build, so the test target gets `rt-multi-thread + macros + time + test-util`; the prod binary gets only the `[dependencies]` set (no `test-util`).

**Do NOT:**
- Add `test-util` to the `[dependencies]` tokio entry (that is the rejected option-b — `test-util` must be test-only).
- Add any NEW crate — only toggle features on the existing `tokio` dep.
- Touch `services/bridge/src/stage.rs` or any other source file — the code is already correct and committed; this is a Cargo.toml-only fix.
- Touch `Cargo.lock` by hand — cargo regenerates it on the next build (the laptop validate step). If `Cargo.lock` changes as a side-effect of a local cargo run, do NOT run cargo yourself; leave it to the laptop validate.
- Touch any `crates/**`, any migration, any other manifest.

**Also resolve DQ `6302ef3b3382-001`:** move it from `pending[]` to `resolved[]` with `answer` = "option-a per advisor 2026-06-19: `time` added to [dependencies], `test-util` added to a new [dev-dependencies] section; test-only, prod binary unaffected.", `answered_by: "advisor"`, `resolved_at: <now ISO8601>`. (The advisor authored this decision in the brief; you are applying it.) Use the Read+Edit JSON approach or `python` to mutate the entry; commit the DQ change in the SAME commit as the Cargo.toml edit.

**Branch:** forks from the **Task 4 #713 worker branch** `junior/role-impl-task-m3-core-stage-mode-task4-30s-grace-boundary-see-claude-prps-briefs-m3-core-stage-mode-impl-4-md-713` (tip `9d035ade5`) — so the fix lands on Task 4's lineage with the committed `stage.rs`.

## §3 Required reading

- `services/bridge/Cargo.toml:1-40` — the `[dependencies]` block + the `tokio` line at ~14; confirm NO `[dev-dependencies]` section exists yet.
- `services/bridge/src/stage.rs:163-180` — the `run_grace` async fn using `tokio::time::sleep` (confirms `time` is a runtime need); `:392-445` the two `#[tokio::test(start_paused = true)]` tests (confirm `test-util` is test-only).
- **Lessons (mandatory, §2.4 file-class — `services/bridge/**` + Cargo.toml):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo runs **Linux ONLY** via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Docker `rust:1.95`).
  - `feedback_linux_compile_proof_is_a_gate.md` — write a `validate-pending-laptop-linux` DQ; the laptop runs `cargo-linux.sh`; `bm-pr` gates on `result:pass`.

## §4 Constraints

- **ONE commit, ≤2 files** (Cargo.toml + decision-queue.json) — `fix(rtc): enable tokio time + test-util features for stage grace timer (fix-impl 3)`.
- **`time` in `[dependencies]`, `test-util` in `[dev-dependencies]`** — the split is the advisor decision; do NOT collapse to one section.
- **No new crate, no source edit** — Cargo.toml feature toggle only + the DQ resolution.
- **Resolve DQ `6302ef3b3382-001`** in the same commit (move pending→resolved, `answered_by: "advisor"`).
- **Bridge compiles on Linux ONLY.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml stage"]`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself (laptop advisor validates). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.

## §3a Handover from prior cohort

Task 4 (#713) committed `stage.rs` @ `53b8c5612` — the `run_grace` async grace-timer (separate async method, sync `promote_next`/`on_grace_expired` unchanged, so the 6 Task-3 tests are intact) + 2 new `#[tokio::test(start_paused = true)]` grace tests (`grace_no_activate_auto_revokes_and_promotes_next` asserting `RevokePublish(W1)` THEN `GrantPublish(W2)`, + a contrast test where activation cancels the grace). DoD-grep PASSED (R7: zero `thread::sleep`, uses `tokio::time::sleep` in `tokio::select!`). The crate does not compile only because tokio's `time` + `test-util` features are off. This fix-impl toggles them per the advisor's option-a decision; nothing else changes.
