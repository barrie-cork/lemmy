---
name: Batch `goto :eof` clobbers errorlevel — always use `exit /b !errorlevel!`
description: In cmd.exe batch scripts, `goto :eof` after a cargo/external command resets errorlevel to 0; use `setlocal enabledelayedexpansion` + `exit /b !errorlevel!` at each exit point to propagate the real exit code
type: feedback
originSessionId: 175bd59e-f876-4745-8e4c-7e2f30e42bd2
---
In Windows batch scripts, `goto :eof` is not exit-code-neutral — it resets
errorlevel to 0. Any wrapper shaped like:

```bat
"%USERPROFILE%\.cargo\bin\cargo.exe" test %*
goto :eof
```

will return 0 to the caller even when cargo failed. The caller (`cmd /c`,
bash `$?`, the ralph loop's stop hook, a GH Actions step) sees exit 0 and
treats the invocation as green. Pre-compile failures like `does not
contain this feature`, `no test target named X`, or compilation errors
are all silently masked.

**Why:** the combination of `goto :eof` in a .bat is a trap regardless of
whether you have multiple exit paths — a single line still triggers it
through implicit end-of-file fallthrough if labels exist below the
current line.

**How to apply:** for any batch script that invokes an external command
whose exit code matters (cargo, npm, docker, pytest, etc.):

1. Add `setlocal enabledelayedexpansion` near the top of the script
   (must be before any labelled block or any `%var%` expansion that
   depends on runtime state).
2. After **every** external-command invocation, use
   `exit /b !errorlevel!` — NOT `goto :eof`, NOT `exit /b %errorlevel%`
   (parse-time captured, always 0 on the first line).
3. Do the same for the final invocation in the script, even if it
   currently propagates by accident through end-of-file. Any future
   symmetry edit will reintroduce the bug class.

**Existence proof:** scripts/brehon/cargo-test.bat shipped this bug on
2026-04-18 as part of Phase 5c risk-reduction Move 7 (commit e7cad24fd).
Four `goto :eof` exit points false-greened every pre-compile failure.
Fixed at bb254e733 — full 5 Whys at
`.claude/PRPs/debug/rca-issue-8-cargo-test-exit-code-masking.md`.

**How to catch this in future:** `.claude/rules/pre-phase-harness-audit.md`
§1 Probe 4 feeds the wrapper a bogus `--features` flag and asserts
non-zero exit. If exit is 0, the wrapper is masking. This probe is
mandatory at every phase start (shipped 61edf2665). A positive-only
wrapper audit (can-it-compile-the-right-thing) is insufficient —
a wrapper must also fail-loud.

**Related:** `cargo-output-capture.md` is the bash/pipe analogue — pipes
mask exit codes too. Both rules are about trust in the validation
layer. The trust failure is the same; the mechanism is different.
