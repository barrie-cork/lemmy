# Root Cause Analysis — Issue #8

**Issue**: `scripts/brehon/cargo-test.bat` returns exit code 0 when cargo fails before compilation (e.g. `the package 'lemmy_server' does not contain this feature`), causing false-green validation.
**Root Cause**: Each of the four exit paths in `cargo-test.bat` ends with `goto :eof` (lines 90, 95, and implicit fallthrough on lines 98-99). `goto :eof` resets the script's exit errorlevel to 0, discarding whatever errorlevel `cargo.exe` returned on the line before. Batch scripts must use `exit /b !errorlevel!` (with delayed expansion) to propagate cargo's exit code back to the parent `cmd /c` caller.
**Severity**: **Critical** — this invalidates every ralph-loop DoD gate, the Phase 5c task-0 audit, and the broader "cargo green = task done" contract that `/prp-ralph` depends on for its stop hook.
**Confidence**: **High** — reproduced directly with a deliberately-bogus feature flag; wrapper reported exit 0 while cargo emitted `error: the package 'lemmy_server' does not contain this feature: nonexistent_xyz`.

---

## Evidence Chain

**WHY 1**: Why does `cargo-test.bat` report exit 0 when cargo fails with "does not contain this feature"?

↓ **BECAUSE**: The four exit points in the script all end with `goto :eof`, and `goto :eof` sets the script's own exit errorlevel to 0 — it does **not** preserve the errorlevel set by the preceding `cargo.exe` invocation.

**Evidence** — `scripts/brehon/cargo-test.bat:89-99`:

```bat
echo BREHON_TEST_THREADS_GUARD: appending --test-threads=1 after existing `--`
"%USERPROFILE%\.cargo\bin\cargo.exe" test %* --test-threads=1
goto :eof                                                 <-- line 90, clobbers errorlevel

:append_with_sep
echo BREHON_TEST_THREADS_GUARD: appending `-- --test-threads=1` for e2e race safety
"%USERPROFILE%\.cargo\bin\cargo.exe" test %* -- --test-threads=1
goto :eof                                                 <-- line 95, clobbers errorlevel

:run_plain
"%USERPROFILE%\.cargo\bin\cargo.exe" test %*
goto :eof                                                 <-- line 99, clobbers errorlevel
```

**WHY 2**: Why does `goto :eof` reset errorlevel to 0 instead of preserving cargo's exit code?

↓ **BECAUSE**: `goto` is itself a batch command. When the batch parser dispatches a `goto` it updates the internal errorlevel register to reflect `goto`'s own success (0), overwriting whatever the previous command set. This is a well-documented cmd.exe quirk — only `exit /b <N>` and `exit <N>` transfer a specific value; `goto :eof` is implemented as "jump to end-of-file and return", which runs the implicit end-of-script cleanup that resets errorlevel.

**Evidence** — direct reproduction in a bash shell (which invokes `cmd /c` the same way the ralph loop and `cargo-output-capture.md`-style wrappers do):

```text
$ cmd //c "cmd /c exit /b 42 & goto :eof"
$ echo $?
1

$ cmd //c "setlocal enabledelayedexpansion & cmd /c exit /b 42 & exit /b !errorlevel!"
$ echo $?
42
```

The first form — identical in shape to lines 89-90 of `cargo-test.bat` — returns 1. The second form, using `exit /b !errorlevel!` with delayed expansion, correctly returns 42. This is the exact class of bug in the wrapper.

**WHY 3**: Why does the user see this bug now, when Move 7 (commit `e7cad24fd`) already shipped?

↓ **BECAUSE**: The Move-7 commit `e7cad24fd` **introduced** the `goto`-heavy flow — the pre-Move-7 version of `cargo-test.bat` (commit `e370523c7`) had a single final `cargo.exe test %*` line with no `goto`, so cargo's errorlevel was the script's errorlevel by default. The new guard's four exit points all terminate with `goto :eof`, which converts the previously-working exit-code propagation into silent clobbering on every invocation that enters the guard branches.

**Evidence** — `git log --oneline -- scripts/brehon/cargo-test.bat`:

```text
e7cad24fd chore(scripts): Phase 5c risk-reduction Move 7 — --test-threads=1 guard
e370523c7 chore(scripts): fill PQ_LIB_DIR for Windows libpq (vcpkg x64-windows)
dabe55a23 chore(scripts): add cargo-test.bat for Windows + libpq via vcpkg
```

And `git show e7cad24fd`:

```text
chore(scripts): Phase 5c risk-reduction Move 7 — --test-threads=1 guard

Guard uses goto-based flow (four exit points: run_plain, append_with_sep,
and two append-with-existing-`--` variants) instead of nested && / || so
batch parser semantics stay predictable across cmd.exe versions.
```

The commit message explicitly calls out the goto-based flow as a deliberate choice to avoid `&&`/`||` parsing surprises. But the author did not also add `exit /b !errorlevel!` at each exit, so the predictability came at the cost of silent exit-code masking.

**WHY 4**: Why did the phase-5b regression "false-succeed on the first invocation"?

↓ **BECAUSE**: The first invocation was run with `--features full` against a crate (`lemmy_server`) that does not expose a `full` feature to this code path, so cargo failed pre-compilation with an error analogous to the reproduction above. The wrapper's `goto :eof` masked cargo's non-zero exit, so the surrounding script / human / ralph iteration read "exit code 0" and interpreted it as a clean test pass. The second invocation (without `--features full`) actually exercised the guard's plain path and cargo genuinely passed — so the symptom looked like an intermittent flake rather than a wrapper bug.

↓ **ROOT CAUSE**: The four cargo-invocation lines in `scripts/brehon/cargo-test.bat` (lines 89, 94, and 98) are each followed by `goto :eof`, with no intervening `exit /b !errorlevel!`. The script does not use `setlocal enabledelayedexpansion`, so even if a fix used `%errorlevel%` it would be parse-time-captured (which is always 0 for the compiled-in value before cargo runs).

**Evidence** — `scripts/brehon/cargo-test.bat:89-99` (reproduced above) plus confirmed reproduction:

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features nonexistent_xyz > .claude\\audit-repro.log 2>&1"
echo "wrapper exit code via cmd /c: $?"
# Output:
#   wrapper exit code via cmd /c: 0
# Log tail:
#   error: the package 'lemmy_server' does not contain this feature: nonexistent_xyz
```

Cargo exited with an error, the wrapper reported 0. The ralph loop (or any `claude -p` caller relying on `$?`) would false-green on this state.

---

## Git History

- **Introduced**: `e7cad24fd` — "chore(scripts): Phase 5c risk-reduction Move 7 — --test-threads=1 guard" — 2026-04-18
- **Author**: Barrie (via Opus 4.7 in Phase 5c risk-reduction moves)
- **Recent changes**: Yes — **this is the introducing commit**. It landed two commits before the current `HEAD` (`9ecd87165` docs-plan-slices).
- **Type**: **Regression introduced by the Move-7 guard**. The pre-Move-7 wrapper (single `cargo.exe test %*` line at end of script) propagated exit codes correctly by default. The guard's goto-based flow broke that.

This is not a long-standing bug. The Phase 5c risk-reduction strategy (see memory `feedback_risk_reduction_8_moves_pattern.md`) shipped eight moves to harden Phase 5c before branch cut — Move 7 was supposed to prevent e2e races via `--test-threads=1` auto-append. It succeeded at that goal but introduced a worse bug: every e2e invocation's exit code is now masked.

---

## Fix Specification

### What Needs to Change

Each of the four exit points in `scripts/brehon/cargo-test.bat` must capture cargo's errorlevel and propagate it via `exit /b <N>`. Because the final labelled block (`:run_plain`) reaches end-of-file without an explicit exit, it also needs a trailing `exit /b !errorlevel!` — and the script as a whole needs `setlocal enabledelayedexpansion` at the top so `!errorlevel!` resolves at runtime, not parse-time.

There's an additional subtlety: `exit /b !errorlevel!` between a cargo call and `goto :eof` works, but replacing `goto :eof` with `exit /b !errorlevel!` directly is simpler and more obviously correct. The `goto :eof` pattern was only load-bearing because the original author assumed fallthrough to end-of-file was equivalent to exit — it isn't, when there are labels below the current line. With four exit points, four explicit `exit /b` calls are the clearest shape.

### Implementation Guidance

```bat
REM Current (problematic) — cargo-test.bat:89-99
echo BREHON_TEST_THREADS_GUARD: appending --test-threads=1 after existing `--`
"%USERPROFILE%\.cargo\bin\cargo.exe" test %* --test-threads=1
goto :eof                                                 <-- clobbers errorlevel to 0

:append_with_sep
echo BREHON_TEST_THREADS_GUARD: appending `-- --test-threads=1` for e2e race safety
"%USERPROFILE%\.cargo\bin\cargo.exe" test %* -- --test-threads=1
goto :eof                                                 <-- clobbers errorlevel to 0

:run_plain
"%USERPROFILE%\.cargo\bin\cargo.exe" test %*
goto :eof                                                 <-- clobbers errorlevel to 0
```

```bat
REM Required (fixed) — cargo-test.bat
REM   1. Add `setlocal enabledelayedexpansion` near the top (before any labelled block)
REM   2. Replace each `goto :eof` after a cargo call with `exit /b !errorlevel!`
REM   3. Keep the internal errorlevel-check on vcvars64.bat as-is (that one already
REM      uses `exit /b 1` on failure).

@echo off
setlocal enabledelayedexpansion
REM ... existing preamble unchanged ...

REM (lines 89-99 become:)
echo BREHON_TEST_THREADS_GUARD: appending --test-threads=1 after existing `--`
"%USERPROFILE%\.cargo\bin\cargo.exe" test %* --test-threads=1
exit /b !errorlevel!

:append_with_sep
echo BREHON_TEST_THREADS_GUARD: appending `-- --test-threads=1` for e2e race safety
"%USERPROFILE%\.cargo\bin\cargo.exe" test %* -- --test-threads=1
exit /b !errorlevel!

:run_plain
"%USERPROFILE%\.cargo\bin\cargo.exe" test %*
exit /b !errorlevel!
```

Also consider applying the same hardening to `cargo-check.bat` as a **preventative** measure. It currently has a single terminal `cargo.exe check %*` line with no `goto` after it, so by pure luck the errorlevel propagates today — but any future edit that adds a `goto :eof` for symmetry would reintroduce the bug. A prophylactic `setlocal enabledelayedexpansion` + trailing `exit /b !errorlevel!` makes the propagation explicit rather than incidental.

### Files to Modify

- `scripts/brehon/cargo-test.bat` — add `setlocal enabledelayedexpansion` after `@echo off`; replace three `goto :eof` calls (lines 90, 95, 99) with `exit /b !errorlevel!`. Primary fix.
- `scripts/brehon/cargo-check.bat` — add `setlocal enabledelayedexpansion` after `@echo off`; replace the terminal cargo invocation with `"%USERPROFILE%\.cargo\bin\cargo.exe" check %%* & exit /b !errorlevel!` (or an equivalent two-line form). Preventative hardening — the same bug class exists-in-waiting here.

### Verification

1. **Negative-feature reproduction** — invoke the wrapper with a bogus feature and assert non-zero:
   ```bash
   cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features nonexistent_xyz > .claude\\verify-neg.log 2>&1"
   echo "exit: $?"   # must print non-zero (cargo returns 101 for this error)
   tail -5 .claude/verify-neg.log   # must show "error: the package ... does not contain this feature"
   ```
2. **Negative-target reproduction** — invoke with a bogus `--test` target name:
   ```bash
   cmd //c "scripts\\brehon\\cargo-test.bat --test nonexistent_target -p lemmy_server > .claude\\verify-target.log 2>&1"
   echo "exit: $?"   # must print non-zero
   ```
3. **Positive baseline** — invoke with a known-good `--no-run` build and assert exit 0:
   ```bash
   cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude\\verify-pos.log 2>&1"
   echo "exit: $?"   # must print 0
   ```
4. **Guard still appends `--test-threads=1`** — invoke with `--test e2e` (no `--no-run`) and verify the log contains `BREHON_TEST_THREADS_GUARD: appending`:
   ```bash
   cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > .claude\\verify-guard.log 2>&1"
   grep BREHON_TEST_THREADS_GUARD .claude/verify-guard.log
   ```
5. **Cargo-check preventative check** — apply the same bogus-feature probe to `cargo-check.bat` before and after the fix to confirm the preventative hardening works:
   ```bash
   cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude\\verify-check.log 2>&1"
   echo "exit: $?"   # must print non-zero
   ```

### Related rules / memories to consult when fixing

- `.claude/rules/cargo-output-capture.md` — the companion rule about exit-code propagation through pipes. This bug is the *script-internal* analogue: the wrapper itself loses the exit code before the pipe even sees it.
- `.claude/rules/pre-phase-harness-audit.md` §1 — the pre-phase wrapper-behavior probes. **These probes would have caught this bug** if they also tested the negative case. Consider amending the audit to include a bogus-feature probe.
- Memory `feedback_wrapper_script_flag_silence.md` — the Phase 2a wrapper-bug class (silent flag discard). This issue is a different flavour of the same trust failure: "wrapper did the thing" vs "wrapper reported success".
- Memory `feedback_cargo_invocations.md` — never pipe cargo through tail. Same spirit: don't trust a wrapped exit code.

---

## Severity rationale

This is the worst kind of bug — a **trust failure in the validation layer** — because it poisons every downstream signal:

1. **Ralph-loop stop hook** reads cargo's exit code to decide whether a task's DoD is green. False-green means the loop writes `<promise>COMPLETE</promise>` on broken builds and advances to the next task with a poisoned state file.
2. **Pre-phase harness audit** (Task 0) runs wrapper probes to validate the environment before task 1. A false-green audit means the phase starts with the audit's explicit job — catching wrapper bugs — defeated.
3. **Phase 5b retro analysis** — the user explicitly noted that the 5b regression "false-succeeded on the first invocation because of this". Any retro conclusions drawn from cargo exit codes during that period need re-verification.
4. **Every `cargo-output-capture.md` pattern** assumes cargo's exit code is faithfully propagated. This wrapper breaks that assumption silently.

The fix is mechanical (~6 lines of batch edits) but the **implications for prior signals** need retrospective re-validation — specifically: any cargo-test.bat invocation logged after `e7cad24fd` (2026-04-18) that reported exit 0 should be re-run with a patched wrapper if the "success" was load-bearing for a task-DoD decision.
