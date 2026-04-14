@echo off
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
REM     git clone https://github.com/microsoft/vcpkg %USERPROFILE%\vcpkg
REM     %USERPROFILE%\vcpkg\bootstrap-vcpkg.bat
REM     %USERPROFILE%\vcpkg\vcpkg.exe install libpq:x64-windows-static-md
REM
REM Then update PQ_LIB_DIR below to point at the installed lib dir, e.g.
REM     set PQ_LIB_DIR=%USERPROFILE%\vcpkg\installed\x64-windows-static-md\lib
REM
REM Usage (from any cmd.exe or from bash via `./scripts/brehon/cargo-test.bat`):
REM    scripts\brehon\cargo-test.bat --test e2e --no-run -p lemmy_server
REM    scripts\brehon\cargo-test.bat --test e2e -p lemmy_server
REM
REM All arguments after the script name are forwarded verbatim to `cargo test`.
REM
REM If the VS Build Tools path below is wrong on another machine, update it
REM before running.

REM ---- libpq.lib discovery (vcpkg-based) --------------------------------
REM Set PQ_LIB_DIR to the directory containing libpq.lib. The placeholder
REM below preserves any value already exported in the parent shell; override
REM it here (uncomment and edit the `set` line) once you have vcpkg +
REM libpq:x64-windows-static-md installed.
REM
REM     set PQ_LIB_DIR=%USERPROFILE%\vcpkg\installed\x64-windows-static-md\lib
set PQ_LIB_DIR=%PQ_LIB_DIR%
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
"%USERPROFILE%\.cargo\bin\cargo.exe" test %*
