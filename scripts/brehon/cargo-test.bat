@echo off
setlocal enabledelayedexpansion
REM Brehon dev utility: run `cargo test` with the Visual Studio 2022 Build
REM Tools linker AND the PostgreSQL client library (libpq.lib) on PATH.
REM This is the `cargo test` sibling of `cargo-check.bat`.
REM
REM Why this script exists: `cargo check` only type-checks, so it never
REM invokes the linker. `cargo test` does invoke the linker, which needs
REM two things on Windows:
REM
REM   1. `link.exe` + MSVC runtime libs — provided by `vcvars64.bat`
REM      from Visual Studio Build Tools.
REM   2. `libpq.lib` — the PostgreSQL C client library, pulled in by
REM      `diesel` (postgres feature) + `pq-sys`. NOT provided by VS.
REM      You must install it separately, typically via vcpkg.
REM
REM vcpkg install (one-time per machine):
REM
REM     cd C:\Users\barri\Developer
REM     git clone https://github.com/microsoft/vcpkg
REM     cd vcpkg
REM     .\bootstrap-vcpkg.bat
REM     .\vcpkg install libpq:x64-windows
REM
REM This installs the DYNAMIC triplet (x64-windows), which means libpq.dll
REM must be on PATH at test runtime as well as libpq.lib being discoverable
REM at link time. This script handles both.
REM
REM If you choose the static triplet instead (x64-windows-static-md), drop
REM the PATH-prepend below and change the triplet name in PQ_LIB_DIR.
REM
REM Usage (from any cmd.exe or from bash via `./scripts/brehon/cargo-test.bat`):
REM    scripts\brehon\cargo-test.bat --test e2e --no-run -p lemmy_server
REM    scripts\brehon\cargo-test.bat --test e2e -p lemmy_server
REM
REM All arguments after the script name are forwarded verbatim to `cargo test`.
REM
REM If the VS Build Tools path or vcpkg path below is wrong on another
REM machine, update them before running.

REM ---- libpq discovery (vcpkg-based, x64-windows dynamic triplet) --------
REM PQ_LIB_DIR    — directory containing libpq.lib (link time)
REM PQ_INCLUDE_DIR — directory containing libpq-fe.h (build.rs time)
REM PATH prepend  — directory containing libpq.dll (test runtime)
set VCPKG_ROOT=C:\Users\barri\Developer\vcpkg
set PQ_LIB_DIR=%VCPKG_ROOT%\installed\x64-windows\lib
set PQ_INCLUDE_DIR=%VCPKG_ROOT%\installed\x64-windows\include
set PATH=%VCPKG_ROOT%\installed\x64-windows\bin;%PATH%
REM -----------------------------------------------------------------------

call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul
if errorlevel 1 (
    echo VCVARS_FAILED: vcvars64.bat returned errorlevel %errorlevel%
    exit /b 1
)
echo VCVARS_OK
where link
if not defined PQ_LIB_DIR (
    echo PQ_LIB_DIR_UNSET: libpq.lib is required but PQ_LIB_DIR is empty.
    echo See the vcpkg install instructions in this script's header.
    exit /b 1
)
echo PQ_LIB_DIR=%PQ_LIB_DIR%
echo ---
cd /d "%~dp0..\.."

REM ---- BREHON_USE_NEXTEST=1 dispatch path --------------------------------
REM When BREHON_USE_NEXTEST=1 is set, redirect through cargo-nextest. Per
REM Perplexity research 2026-05-02 + .config/nextest.toml: nextest's
REM process-per-test isolates LazyLock<Settings>, so the --test-threads=1
REM guard below is unnecessary under nextest. Concurrency caps are in
REM .config/nextest.toml (threads-required).
REM
REM Note: nextest invocation form is `cargo nextest run <args>` — the
REM `run` subcommand is added here automatically.
if "%BREHON_USE_NEXTEST%"=="1" (
    where cargo-nextest >nul 2>&1
    if errorlevel 1 (
        echo CARGO_NEXTEST_NOT_INSTALLED: run `cargo install cargo-nextest` first.
        exit /b 1
    )
    echo BREHON_USE_NEXTEST: dispatching through cargo nextest run
    "%USERPROFILE%\.cargo\bin\cargo.exe" nextest run %*
    exit /b !errorlevel!
)

REM ---- --test-threads=1 enforcement for e2e runs --------------------------
REM Phase 5b carry-forward #3: the e2e suite races under parallelism because
REM SETTINGS is a LazyLock singleton that caches the first test's
REM LEMMY_DATABASE_URL. The full e2e suite must run with --test-threads=1 or
REM tests after the first probe the wrong testcontainer. This guard appends
REM the flag automatically when invoking `--test e2e` without `--no-run` and
REM without an explicit --test-threads override. Build-only invocations and
REM non-e2e tests are untouched.
set "BREHON_ARGS=%*"

echo %BREHON_ARGS% | findstr /C:"--test e2e" >nul
if errorlevel 1 goto :run_plain

echo %BREHON_ARGS% | findstr /C:"--no-run" >nul
if not errorlevel 1 goto :run_plain

echo %BREHON_ARGS% | findstr /C:"--test-threads" >nul
if not errorlevel 1 goto :run_plain

echo %BREHON_ARGS% | findstr /C:" -- " >nul
if errorlevel 1 goto :append_with_sep

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
