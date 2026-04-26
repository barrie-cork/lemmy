---
name: Wrapper scripts can silently discard flags
description: Wrapper scripts that forward cargo/npm/etc. commands can silently discard flags (especially target selection like `-p crate-name`) without emitting any error. Re-validate wrapper behavior against intent at the start of any phase of work, before trusting its output.
type: feedback
originSessionId: 50cf5c6f-6b81-45b0-b3b4-45ac46b352d8
---
Wrapper scripts — `.bat`, `.sh`, shell functions, or Makefile targets that forward commands to `cargo`, `npm`, `pnpm`, `docker`, etc. — can silently discard flags and still exit 0. The most common failure mode is target-selection flags (`-p <crate>`, `--package <name>`, `--workspace` vs `--package`, `--features <flag>`) being hardcoded in the wrapper and ignoring the user's version.

**Symptom:** the wrapper exits 0, the captured log looks reasonable, but the scope of what actually ran doesn't match intent. A per-crate check false-greens because the wrapper is running `--workspace` instead.

**Why:** Hit during Brehon Phase 2a checkpoint-2 (2026-04-15). The fork's `scripts/brehon/cargo-check.bat` hardcoded `cargo check --workspace` and silently discarded any `-p <crate>` arguments passed to it. Phase 0 and Phase 1 never noticed because workspace-check is a superset of per-crate-check for cross-feature type errors. Phase 2a task 14 hit it because the new governance view crate had code behind `#[cfg(feature = "full")]` that workspace-check with default features didn't activate. The bug had survived all prior validation and was only caught by an advisor-initiated per-crate audit.

**How to apply:**

- **Run three probes at the start of any phase** that uses wrapper scripts for validation:
  1. Per-crate honor: `wrapper -p <small-crate>` → confirm log shows only that crate compiling
  2. Feature flag honor: `wrapper -p <crate> --features <flag>` → confirm log shows `--features <flag>` in the cargo invocation
  3. Target selection honor: `wrapper --test <test_name> -p <crate>` → confirm only that test target compiles
- **If any probe fails, stop and fix the wrapper** before starting the phase. Pre-phase wrapper fix is cheap; mid-phase wrapper fix costs a checkpoint cascade.
- **Capture the probe output to files** so the audit is auditable: `wrapper ... > .claude/audit-*.log 2>&1`. Never rely on "it looked right when I ran it."
- **Document the wrapper's actual behavior at the top of the file** — what it forwards, what it hardcodes, what it discards. If the wrapper is doing anything non-obvious, say so in comments.

**Generalizes to:** any tooling layer where you've added a convenience wrapper over a CLI and the wrapper is not itself unit-tested. Applies to make, just, npm-scripts, shell aliases, CI job wrappers, editor integration scripts. The common pattern is "I wrote this wrapper once to handle an edge case and haven't looked at it since."

**Symptom to recognise in retrospect:** you run a validation command through a wrapper, it exits 0, you trust it, and then a different validation path (direct cargo invocation, CI, another machine) surfaces a failure that should have been caught. The gap between "wrapper-based validation said green" and "reality is red" is the wrapper eating flags.

**Mitigation pattern:** the pre-phase harness audit documented in the Brehon fork's `.claude/rules/pre-phase-harness-audit.md` — run specific probes against the wrapper at phase start, before any task runs. Generalizes to: run your wrapper probes in CI, or as the first step of any multi-step validation run.
