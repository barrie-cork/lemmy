# Pre-phase harness audit

Before the first task of any new Brehon phase, run this audit. It's
mandatory — not a suggestion, not a checkpoint-conditional step. Skipping
this in Phase 2a caused a four-layer cascade at checkpoint-2 that cost
~30 minutes of advisor time and 3 meta-commits to unwind. The audit is
10-15 minutes at phase start and catches the class of bug that's
invisible to `cargo check --workspace` and survives prior-phase
validation.

This rule auto-loads in `-p` mode. Every ralph loop reads it at its
first iteration.

## What to audit

### 1. Wrapper script behavior vs intent

The scripts under `scripts/brehon/` are the only supported way to run
cargo on this project on Windows (libpq + vcvars requirements). They must
behave as documented. **Silent flag discard is the most dangerous failure
mode** because it false-greens validation without emitting any error.

Run these three probes and confirm the captured logs match intent:

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
```

If any probe fails, **stop the phase** and fix the wrapper in a pre-phase
commit on the current branch. Do not start task 1 with a broken wrapper.
The checkpoint-2 audit in Phase 2a found this bug under task 14 — it
would have cost one audit instead of a four-layer unwind had the probe
run before task 1.

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
