@echo off
setlocal enabledelayedexpansion
REM Brehon dev utility: run `cargo nextest run` with the same vcvars +
REM libpq env as cargo-test.bat. Per Perplexity research 2026-05-02:
REM nextest's process-per-test model resolves the LazyLock<Settings>
REM singleton problem without code changes. Concurrency caps come from
REM .config/nextest.toml (threads-required), not from this wrapper.
REM
REM Usage (from any cmd.exe or from bash via `cmd /c`):
REM    scripts\brehon\cargo-nextest.bat run --workspace --features full --test e2e
REM    scripts\brehon\cargo-nextest.bat run --workspace --features full --test e2e -E "test(postgres_container_boots)"
REM
REM Pre-flight (one-time per machine): cargo install cargo-nextest
REM Verify:                            cargo nextest --version
REM
REM Why this exists: nextest binaries are linked the same way as
REM `cargo test` binaries, so the libpq + vcvars setup that cargo-test.bat
REM provides is also required for nextest. This wrapper mirrors that
REM setup verbatim. The `--test-threads=1` injection that cargo-test.bat
REM does for e2e is intentionally OMITTED here — nextest handles
REM concurrency via .config/nextest.toml.

REM ---- libpq discovery (vcpkg-based, x64-windows dynamic triplet) --------
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
    exit /b 1
)
echo PQ_LIB_DIR=%PQ_LIB_DIR%
where cargo-nextest >nul 2>&1
if errorlevel 1 (
    echo CARGO_NEXTEST_NOT_INSTALLED: run `cargo install cargo-nextest` first.
    exit /b 1
)
echo ---
cd /d "%~dp0..\.."
"%USERPROFILE%\.cargo\bin\cargo.exe" nextest %*
exit /b !errorlevel!
