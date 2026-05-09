---
name: Windows e2e requires cargo-test.bat wrapper and --workspace flag
description: On Windows, bare `cargo test` for the e2e suite fails STATUS_DLL_NOT_FOUND because libpq.dll is only on the Windows DLL search path when cargo-test.bat runs. Bash PATH export has no effect on the Windows PE DLL loader. Also --workspace required — lemmy_server has no 'full' feature.
type: feedback
originSessionId: 74b3a25c-bcd1-4b3d-8421-b78b6c6ef678
---
Use the bat wrapper for all e2e runs on Windows:

```bash
# ✅ Correct
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-<phase>-<sha>.log 2>&1 && echo E2E_EXIT_0 >> .claude/runlog/... || echo E2E_EXIT_NONZERO >> .claude/runlog/..."
```

With `run_in_background: true` in the Bash tool. Never:

```bash
# ❌ WRONG — libpq.dll not on Windows DLL search path
cargo test --workspace --test e2e --features full -- --test-threads=1

# ❌ WRONG — bash PATH export does NOT affect Windows DLL loader
export PATH="C:/...vcpkg/bin:$PATH"
cargo test ...

# ❌ WRONG — lemmy_server has no 'full' feature
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_server --features full ..."
```

**Why the wrapper is mandatory:** `libpq.dll` lives at
`vcpkg\installed\x64-windows\bin\`. The Windows PE DLL loader uses
Windows `%PATH%`, set by the bat wrapper's `set PATH=%VCPKG_ROOT%\...\bin;%PATH%`.
MSYS2/Git Bash's `$PATH` is a POSIX construct — it does NOT propagate
to the Windows DLL search path. `export PATH=...` in bash is a no-op
for Windows EXE DLL resolution.

**Why `--workspace` not `-p lemmy_server`:** `lemmy_server` does not
declare the `full` feature. `-p lemmy_server --features full` always
errors: `error: the package 'lemmy_server' does not contain this
feature: full`. `--workspace --features full` propagates `full` to the
crates that do declare it (lemmy_db_schema, lemmy_utils, etc.).

**Symptom:** exit code `0xc0000135` (`STATUS_DLL_NOT_FOUND`), log ends
with `E2E_EXIT_127`. Compile succeeds; the test EXE crashes on load.

**Incident:** v1-SL-c-2 Phase 2 e2e, 2026-05-09. Three failed attempts
(r1: bash cargo; r2: bash + PATH export; r3: bat wrapper -p lemmy_server)
before r4 (bat wrapper --workspace) ran correctly. ~35 min wasted.
Full RCA: `.claude/PRPs/reports/rca-phase2-e2e-invocation-failure-2026-05-09.md`.

**Related lessons:** `feedback_cmd_c_redirect_exit_code_capture.md`,
`feedback_wrapper_script_flag_silence.md`.
