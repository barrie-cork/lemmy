---
paths:
  - "crates/**"
  - ".github/workflows/**"
  - "scripts/brehon/**"
---

# Pre-phase harness audit

Before the first task of any new Brehon phase, run this audit. It's
mandatory — not a suggestion, not a checkpoint-conditional step. Skipping
this in Phase 2a caused a four-layer cascade at checkpoint-2 that cost
~30 minutes of advisor time and 3 meta-commits to unwind. The audit is
10-15 minutes at phase start and catches the class of bug that's
invisible to `cargo check --workspace` and survives prior-phase
validation.

This rule loads at session start (along with the rest of `.claude/rules/`). Every ralph loop reads it at its first iteration.

## Wrapper invocation

The audit runs on the laptop (Windows) via the bat-wrapper invocation
`cmd //c "scripts\\brehon\\cargo-<verb>.bat <args>"`. Linux/macOS
equivalents exist at `./scripts/brehon/cargo-<verb>.sh <args>` for
portability but Brehon's primary runner is Windows-laptop.

The wrapper must:

- Accept `$@` / `%*` and pass it through to cargo (no flag-silent
  hardcoding of `--workspace`, `-p <crate>`, etc — see
  `feedback_wrapper_script_flag_silence.md`).
- Propagate cargo's exit code (`exit /b !errorlevel!` on Windows).
  The negative probe (Probe 4) catches exit-code masking.
- Set up libpq discovery before invoking cargo (vcpkg on Windows
  with the PATH/lib variables; apt-installed `libpq-dev` on Linux
  discovered automatically by `pkg-config`).

## What to audit

### 0. Docker daemon preflight

Probe 0 runs before any other probe. The e2e harness (`cargo test --test
e2e`) uses testcontainers-rs → Postgres in a container; if the Docker
daemon is down, the test failure prints as
`start_postgres: failed to create a container: Error in the hyper legacy
client: client error (Connect)` — which reads like "Postgres container
crashed" when the actual problem is "Docker Desktop is stopped." Brehon
runs primarily on Windows per CLAUDE.md, and Docker Desktop has a habit
of stopping on sleep/resume, so mid-session stops are realistic too.

```bash
# Probe 0 — Docker daemon running
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || {
  echo "DOCKER NOT RUNNING — start Docker Desktop / dockerd before continuing"
  exit 1
}
```

Expected: `DOCKER OK`. If not, STOP — start Docker Desktop and re-run
Probe 0 before any cargo invocation (cargo itself doesn't need Docker,
but every e2e probe and every task-level test invocation does).

Note: the `/prp-core:prp-implement` command template also runs this
same probe at §4.2.0 before every `cargo test --test e2e` invocation.
Running it here too catches the common case (Docker was stopped before
the phase started) without waiting for the first e2e run to fail; the
per-command repeat catches the mid-session stop case.

Per DQ #44 (v1-AD-d retro §2.3) lean (c) — probe in both places, costs
~50ms per run.

### 1. Wrapper script behavior vs intent

The scripts under `scripts/brehon/` are the only supported way to run
cargo on this project on Windows (libpq + vcvars requirements). They must
behave as documented. Two failure modes are equally dangerous because
both false-green validation:

- **Silent flag discard** — wrapper runs cargo against the wrong scope or
  without the requested feature, but still reports what cargo did. Caught
  by the positive probes (1, 2, 3) below.
- **Exit-code masking** — wrapper runs cargo correctly but loses cargo's
  exit code before returning to the caller (e.g. `goto :eof` in a batch
  script clobbers errorlevel, swallowing real failures). Caught by the
  negative probe (4) below. This bug class shipped once already on
  2026-04-18 in `e7cad24fd` — see
  `.claude/PRPs/debug/rca-issue-8-cargo-test-exit-code-masking.md`.

Run all four probes and confirm the captured logs and exit codes match
intent:

```bash
# Probe 1 — per-crate check honors -p
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/audit-cargo-check-p.log 2>&1"
tail -20 .claude/audit-cargo-check-p.log
# Expected: only lemmy_utils compiles. If you see "Checking lemmy_db_schema"
# or other crates, the wrapper is discarding -p and running --workspace.
# STOP and fix the wrapper before starting the phase.

# Probe 2 — feature flag activation
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/audit-cargo-check-features.log 2>&1"
tail -20 .claude/audit-cargo-check-features.log
# Expected: compiles with features enabled. If the log doesn't show
# --features full in the cargo invocation, the wrapper is dropping the
# flag. STOP.

# Probe 3 — cargo-test wrapper honors target selection
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/audit-cargo-test.log 2>&1"
tail -20 .claude/audit-cargo-test.log
# Expected: only e2e test target compiles. If the wrapper builds
# additional test binaries or the whole workspace, it's discarding flags.

# Probe 4 — wrappers fail loud on cargo errors (exit-code propagation)
#
# The previous three probes only confirm the wrapper ran cargo correctly.
# They do NOT confirm the wrapper will surface cargo's failure to the
# caller — a broken wrapper that always returns 0 passes probes 1-3 and
# still poisons every downstream DoD gate. Probe 4 feeds cargo a known-
# bad flag and asserts the wrapper propagates the non-zero exit code.
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-test-negative.log 2>&1"
echo "cargo-test.bat exit on bogus feature: $?"
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-check-negative.log 2>&1"
echo "cargo-check.bat exit on bogus feature: $?"
# Expected: BOTH lines print a non-zero exit code (typically 101, cargo's
# error exit). If either prints 0, the wrapper is masking cargo's exit
# code. STOP and fix the wrapper before starting the phase. Tail of each
# log should contain: "error: the package 'lemmy_server' does not contain
# this feature: nonexistent_xyz". If the tail shows the error but the exit
# is still 0, you're looking at the exit-code-masking bug class directly.
```

If any probe fails, **stop the phase** and fix the wrapper in a pre-phase
commit on the current branch. Do not start task 1 with a broken wrapper.
The checkpoint-2 audit in Phase 2a found the flag-discard bug under task
14 — it would have cost one audit instead of a four-layer unwind had the
probe run before task 1. The exit-code-masking bug shipped under Move 7
in Phase 5c risk-reduction and went undetected until the first Slice A
regression false-succeeded — it would have cost one audit had probe 4
existed then.

### 2. DoD smoke test — every validation command in the plan

Open the plan file at `.claude/PRPs/plans/phase-*.plan.md` and find every
validation command named in a task DoD (§9 in the current plan
template). For each one, run it against the current branch HEAD (which
should be the prior-phase HEAD + plan commit) and capture exit code.

```bash
# For every command `X` listed in a task DoD:
cmd //c "X > .claude/audit-dod-<taskN>.log 2>&1"
echo "exit: $?"
```

Expected exit code:
- **Validation commands that check the crate under development must
  currently fail** (crate doesn't exist yet). Note these — they're the
  expected-red set that task N will turn green.
- **Validation commands that check unchanged crates must currently
  pass.** If any unchanged-crate check is currently red, the plan's DoD
  is either citing pre-existing upstream lint debt, using an unactivated
  feature flag, or using a command flag the wrapper silently drops.
  Document as drift and adjust plan wording before task 1.

Three specific DoD footguns to check proactively:

- **`cargo clippy -p <crate> -- -D warnings` without `--features full`.**
  If the governance code is behind a `#[cfg(feature = "full")]` gate, the
  clippy command won't see it and will either pass (because it compiled
  nothing) or fail on unrelated code. Always add `--features full` to any
  clippy command that touches governance crates.

- **`cargo clippy ... -- -D warnings` without `--no-deps`.** Without
  `--no-deps`, clippy lints the upstream `lemmy_*` deps too, and any
  pre-existing upstream lint debt (e.g. `lemmy_diesel_utils::pagination`
  lint hits) will mask your own code under a deny-warnings gate. Use
  `--no-deps` unless you genuinely want to lint transitive deps.

- **`cargo test --test e2e` without `-p lemmy_server`.** The e2e harness
  lives in `crates/server/tests/e2e.rs`. Without `-p lemmy_server`, cargo
  may build the whole workspace's test targets, wasting time and
  potentially invoking libpq-dependent test binaries that don't need to
  run.

### 3. Clippy baseline capture

Run the exact clippy command the plan specifies against the prior-phase
HEAD, captured to file:

```bash
cmd //c "scripts\\brehon\\cargo-check.bat ... --some-clippy-command > .claude/audit-clippy-baseline.log 2>&1"
echo "exit: $?"
tail -40 .claude/audit-clippy-baseline.log
```

If the baseline is non-zero, the plan's clippy DoD is unexecutable before
the phase even starts. Two paths:

- **Fix in a pre-phase commit.** If the lint debt is in code you own
  (governance crates from earlier phases), fix it first and commit as
  `chore(lint): clear <lint name> debt pre-phase N`.
- **Narrow the plan's clippy DoD.** Change the DoD to use `--no-deps`,
  `-p <new-crate-only>`, or `--features <specific>` so it doesn't inherit
  the pre-existing debt. Commit as `docs(plan): narrow clippy DoD to
  avoid pre-existing upstream debt` with a reference to which debt is
  being excluded.

Do not start task 1 with a clippy baseline that the plan cannot green.

## What the audit is NOT

- Not a replacement for checkpoint gates — it's a pre-flight, not a
  phase-close check.
- Not a guarantee the plan is correct — it only validates that the
  plan's stated commands run against the starting state. Query shapes,
  type errors, and lint surprises inside task bodies are still caught by
  the ralph loop's per-task validation.
- Not something the advisor runs for the impl agent. The impl agent runs
  the audit as step 0 of the loop, alongside task 0's branch
  verification. The advisor verifies the audit output as part of
  approving the loop start.

## When to skip

Never skip for a new phase. Skip is allowed only when:

- Re-running a partially-failed loop after a mid-phase reset
- Running a bugfix mini-phase (hotfix on governance-v0 outside the
  normal phase cadence)

If you're tempted to skip for any other reason, you're about to pay the
checkpoint-2 cost.

## References

- Phase 2a retrospective §3 (the four-layer checkpoint-2 cascade) —
  advisor memory `project_brehon_phase_2a_complete.md`
- `feedback_wrapper_script_flag_silence.md` — the wrapper bug class
- `feedback_pre_phase_dod_smoke_test.md` — the DoD smoke test pattern
- Plan file's Task 0 section — runs alongside this audit
