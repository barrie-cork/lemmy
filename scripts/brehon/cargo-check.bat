@echo off
setlocal enabledelayedexpansion
REM Brehon dev utility: run `cargo check --workspace` with the Visual Studio
REM 2022 Build Tools linker on PATH. Lemmy's build needs `link.exe`, which only
REM appears on PATH after vcvars64.bat is sourced in the current shell.
REM
REM Usage (from any cmd.exe or from bash via `cmd /c`):
REM    scripts\brehon\cargo-check.bat
REM
REM If the VS Build Tools path below is wrong on another machine, update it
REM before running.
REM ---- libpq discovery (parity with cargo-test.bat) -------------------------
REM cargo check runs pq-sys build.rs which caches PQ_LIB_DIR. Without this,
REM a subsequent cargo-test.bat finds a stale cache entry recording
REM PQ_LIB_DIR=Err(NotPresent) and fails at link time (LNK1181).
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
echo ---
cd /d "%~dp0..\.."
"%USERPROFILE%\.cargo\bin\cargo.exe" check %*
exit /b !errorlevel!
