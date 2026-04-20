# Pre-phase harness audit

Mandatory before the first task of any new phase. Never skip.

## 1. Wrapper script probes

```bash
# Probe 1 — per-crate check honors -p
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/audit-cargo-check-p.log 2>&1"
tail -20 .claude/audit-cargo-check-p.log
# Expected: only lemmy_utils compiles. STOP if other crates appear.

# Probe 2 — feature flag activation
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/audit-cargo-check-features.log 2>&1"
tail -20 .claude/audit-cargo-check-features.log
# Expected: --features full in cargo invocation. STOP if missing.

# Probe 3 — cargo-test honors target selection
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/audit-cargo-test.log 2>&1"
tail -20 .claude/audit-cargo-test.log
# Expected: only e2e test target compiles.

# Probe 4 — exit-code propagation (negative test)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-test-negative.log 2>&1"
echo "cargo-test.bat exit on bogus feature: $?"
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-check-negative.log 2>&1"
echo "cargo-check.bat exit on bogus feature: $?"
# Expected: BOTH non-zero. If either is 0, wrapper masks exit codes. STOP.
```

If any probe fails, fix the wrapper before starting task 1.

## 2. DoD smoke test

Run every validation command from the plan's task DoDs against current HEAD:

```bash
cmd //c "X > .claude/audit-dod-<taskN>.log 2>&1"
echo "exit: $?"
```

- Commands for the new crate should currently fail (expected-red).
- Commands for unchanged crates must currently pass.

DoD footguns:
- `cargo clippy -p <crate> -- -D warnings` without `--features full` skips `#[cfg(feature = "full")]` code.
- `cargo clippy` without `--no-deps` inherits upstream lint debt.
- `cargo test --test e2e` without `-p lemmy_server` builds the whole workspace.

## 3. Clippy baseline capture

Run the plan's clippy command captured to file. If non-zero, either fix in a pre-phase commit or narrow the DoD scope. Do not start task 1 with an unachievable clippy baseline.

## When to skip

Only when re-running a partially-failed loop or a hotfix mini-phase.
