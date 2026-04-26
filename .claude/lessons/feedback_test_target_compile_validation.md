---
name: Test-target compile validation is mandatory for e2e-touching PRs
description: cargo check --workspace (even with --features full) does NOT compile test targets; use cargo test --no-run or --all-targets
type: feedback
originSessionId: b8299e6d-d78c-4d44-9296-5e7c3aa707be
---
`cargo check --workspace --features full` compiles **library + binary targets only** — it does NOT compile test targets under `crates/server/tests/` or any `#[cfg(test)]` code. A PR can pass `check` + `clippy` workspace gates and still fail on CI with a genuine compile error the moment tests try to build.

**Why:** `DoD gate: cargo test --test e2e -p lemmy_server --no-run` was missing from Phase 5c's DoD table. PR #10 (2026-04-19) passed `check` + `clippy` clean but CodeRabbit re-review found a missing import in `sponsor_liability_with_founder_multiplier` — the call-site existed but the inner `use` block was untouched. The error would have surfaced on first CI e2e run.

**How to apply:** any PR that touches `crates/server/tests/e2e.rs`, any test-macro-decorated `pub fn`, or adds handlers referenced from tests must run `cargo test --test e2e -p lemmy_server --no-run --features full` (note: `lemmy_server` crate doesn't have a `full` feature — drop that flag for the server-crate invocation specifically; the workspace/per-crate form `cargo test --workspace --features full --no-run` is the one in the Phase 5c DoD template).

The wrapper script on Windows (`scripts/brehon/cargo-test.bat`) ALSO has the exit-code-masking bug class — see `feedback_batch_goto_eof_clobbers_errorlevel.md`. Always `tail` the log and grep for `^error` even when the harness reports exit 0.

Add to phase DoD templates going forward: `cargo test --test e2e --no-run` as a separate mandatory gate alongside `check` and `clippy`.
