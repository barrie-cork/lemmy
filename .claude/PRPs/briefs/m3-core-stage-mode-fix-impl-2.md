# Brief: m3-core-stage-mode fix-impl-2 (Task 3 clippy unwrap_or_default)

## §1 Role + dispatch line

`[role:impl-task] m3-core-stage-mode-fix-impl2-stage-load-error-propagate — see .claude/PRPs/briefs/m3-core-stage-mode-fix-impl-2.md`

## §2 Scope

**Fix the ONE clippy error that failed Task 3's `validate-pending-laptop-linux` validation** (DQ `e1256574bda6-001`, CLIPPY_EXIT_101). `cargo check` PASSES and all 6 `stage` tests PASS (incl. the marquee `fifo_mic_pass_in_sequence`); the ONLY blocker is a single disallowed-method clippy error.

**The failing lint (verbatim):**
```
error: use of a disallowed method `core::result::Result::unwrap_or_default`
  --> src/stage.rs:61:18
```

In `Stage::load`, a malformed persisted `queue_state` JSON is silently swallowed to an empty queue via `.unwrap_or_default()` — losing the raised-hand FIFO order on a corrupt row. **User decision (2026-06-19): propagate the parse error** so a corrupt `queue_state` surfaces instead of vanishing. This is the semantically-correct fix (a corrupt raised-hand queue must not silently reset).

**Produces (exactly 1 file edit, ONE commit):**
1. `services/bridge/src/stage.rs` — two changes:
   - **Import:** line 5 is `use anyhow::{anyhow, Result};` → add `Context`: `use anyhow::{anyhow, Context, Result};`.
   - **`Stage::load` (the `Some(json) =>` arm at ~line 58-61):** replace `.unwrap_or_default()` with `.context("corrupt bridge_room.queue_state")?`:
     ```rust
     // before:
     Some(json) => serde_json::from_str::<Vec<String>>(&json)
         .map(|v| v.into_iter().collect())
         .unwrap_or_default(),
     // after:
     Some(json) => serde_json::from_str::<Vec<String>>(&json)
         .map(|v| v.into_iter().collect())
         .context("corrupt bridge_room.queue_state")?,
     None => VecDeque::new(),
     ```
   `Stage::load` already returns `Result<Self>`, so `?` propagates cleanly. No signature change.

**Do NOT:**
- Change any other line of `stage.rs` — only the import + the one `.unwrap_or_default()` → `.context(...)?` swap.
- Touch any test (all 6 pass; the `Stage::load` test callers at lines 214/327/335 use well-formed data — the fix does NOT break them, verified by advisor).
- Touch any other file, any `crates/**`, any migration, `Cargo.toml`/`Cargo.lock`.
- Add any `#[allow]` — this is a real fix (error propagation), not a suppression.

**Branch:** forks from the **Task 3 worker branch** `junior/role-impl-task-m3-core-stage-mode-task3-stage-fifo-mic-pass-see-claude-prps-briefs-m3-core-stage-mode-impl-3-md-711` (tip `d26d87ba2`) — so the fix lands on Task 3's lineage.

## §3 Required reading

- `services/bridge/src/stage.rs:1-70` — the `use anyhow::{...}` import (line 5) + the `Stage::load` fn (`:56-66`) with the `.unwrap_or_default()` at `:61`.
- **Lessons (mandatory, §2.4 file-class — `services/bridge/**`):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo runs **Linux ONLY** via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Docker `rust:1.95`).
  - `feedback_linux_compile_proof_is_a_gate.md` — write a `validate-pending-laptop-linux` DQ; the laptop runs `cargo-linux.sh`; `bm-pr` gates on `result:pass`.
  - `feedback_clippy_test_style.md` — the disallowed-method list (`unwrap_or_default`, `unwrap`, `expect`) is enforced under `-D warnings`; propagate with `?` + `.context()`.

## §4 Constraints

- **ONE commit, 1 file** — `fix(rtc): propagate corrupt queue_state parse error in Stage::load (fix-impl 2)`.
- **Error-propagation, NOT suppression** — replace `.unwrap_or_default()` with `.context("corrupt bridge_room.queue_state")?`. Do NOT `#[allow(clippy::disallowed_methods)]` — the lint is correct; a corrupt persisted queue must surface.
- **Add `Context` to the existing anyhow import** — `use anyhow::{anyhow, Context, Result};` (line 5). Without it, `.context(...)` won't resolve.
- **No behavior change to the happy path** — well-formed `queue_state` still parses to the FIFO; only the malformed case now errors instead of resetting. The 6 existing tests must still pass (they use well-formed data).
- **Bridge compiles on Linux ONLY.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml stage"]`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself (laptop advisor validates). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.

## §3a Handover from prior cohort

Task 3 (#711) shipped `stage.rs` with the full chair-seat + FIFO + mic-pass state machine + 6 deterministic tests (all PASS incl. the marquee `fifo_mic_pass_in_sequence`). `cargo check` PASSED. The ONLY validation blocker is a single `unwrap_or_default` clippy error at `stage.rs:61` in `Stage::load` (corrupt-queue_state silent swallow). This fix-impl propagates that parse error per the user's decision; nothing else about Task 3 changes.
