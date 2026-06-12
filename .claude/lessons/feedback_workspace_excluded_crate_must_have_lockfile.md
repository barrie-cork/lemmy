---
name: Workspace-excluded binary crate must have Cargo.lock committed
description: Any cargo workspace-excluded binary crate (e.g. services/bridge, R9 pattern) must have its Cargo.lock committed. Without a lock, every cold cargo check re-resolves the dep tree against the current crates.io state and can silently pick up a breaking transitive bump.
type: feedback
---

Any crate listed in the root `Cargo.toml` `exclude = [...]` array that is a **binary crate or application** (not a library) **must have its `Cargo.lock` committed**. Without it, every cold `cargo check`/`build`/`test` re-resolves the dep tree against the current crates.io index and can silently pick up a breaking transitive bump on any run.

## Why

Lock files exist to make builds reproducible. For workspace **members**, the single root `Cargo.lock` covers all member crates. For workspace-**excluded** crates, there is no such safety net — each crate is a separate cargo project with its own dep resolution. If the excluded crate's `Cargo.lock` is absent:

- Every cold Docker/CI run downloads the latest-compatible versions of all deps.
- A transitive dep that ships a minor bump (e.g. `time 0.3.47 → 0.3.48`) can introduce a breaking change (e.g. E0119 conflicting trait impl) that didn't exist the day before.
- The breakage is **silent** — no diff, no commit, nothing in git to flag — just a different crates.io resolution.

**Origin incident (2026-06-12, `services/bridge`):** `matrix-sdk = "0.18"` pulled `ruma-common 0.19.0` which specified `time ^0.3.47`. Without a lock, Docker resolved `time 0.3.48` (June 2026 bump). `ruma-common 0.19.0`'s `StringEnum` derive generated conflicting `From<HourBase>` impls with `time 0.3.48+`, producing 25 `E0119` errors. The fix was `time = "=0.3.47"` pin + committing the generated lock. The prior session's warmup had falsely appeared to succeed (background-task exit code lied); the wrong premise was baked into a lesson the same day it was wrong.

**Revisit condition:** `time = "=0.3.47"` is a durable workaround until `ruma-common` or `matrix-sdk` ships a dep chain that resolves the E0119 conflict. When `matrix-sdk` bumps its ruma dependency, re-run the Docker check without the pin; if green, remove it. Do NOT bump `matrix-sdk` or run `cargo update` in `services/bridge` without immediately verifying the Docker build.

## Rule

After authoring or scaffolding any workspace-excluded binary crate:

1. Run the first `cargo check` (or `cargo build`) — use `cargo-linux.sh --manifest-path <crate>/Cargo.toml` for Linux-validated crates.
2. Commit the generated `Cargo.lock` in the same commit or immediately after, on the phase branch (not a worktree-local stash).
3. Treat a missing lock as a **DoD miss** — any plan §15 bridge command that implicitly requires a clean build but has no committed lock is incomplete.

## Detection

```bash
git ls-tree HEAD services/bridge/Cargo.lock   # empty output = NOT tracked
ls services/bridge/Cargo.lock                 # file absent = not generated yet
```

Empty `git ls-tree` + exit 0 means **not tracked** (per `pattern_verify_before_trusting_shell_output`).

## How to apply

- **Planner:** if the phase's §13 tasks touch `services/bridge/Cargo.toml` (dep changes), include a `DoD: services/bridge/Cargo.lock committed` line in the task's DoD.
- **Advisor (§3.4 DoD smoke test):** before running any bridge cargo DoD command, verify `git ls-tree HEAD services/bridge/Cargo.lock` is non-empty. If empty, note as a DoD issue.
- **Impl-task:** after any `cargo check` on the bridge that generates or updates `Cargo.lock`, commit it before writing the `validate-pending-laptop` DQ entry.

## See also

- `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo routing; the origin incident
- `feedback_background_task_notification_lies.md` — why the prior session's warmup appeared to succeed
- `feedback_linux_compile_proof_is_a_gate.md` — bridge Linux validation as a gate
