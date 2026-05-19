@echo off
setlocal enabledelayedexpansion
REM Brehon dev utility: run the lemmy_diesel_utils migration runner with the
REM Visual Studio 2022 Build Tools linker AND the vcpkg libpq.dll directory on
REM the *Windows* %PATH%. migrate-roundtrip.sh delegates here on Windows.
REM
REM WHY this wrapper exists (incident 2026-05-16, v1-federation-inbound-a
REM Task 1, DQ #241): migrate-roundtrip.sh called `cargo run -p
REM lemmy_diesel_utils --features full` bare. The binary built fine but
REM crashed at load with STATUS_DLL_NOT_FOUND (0xc0000135) because libpq.dll
REM lives at vcpkg\installed\x64-windows\bin and the Windows PE DLL loader
REM resolves it via Windows %PATH%, NOT bash $PATH. A bash `export PATH=...`
REM is a NO-OP for the Windows DLL loader (see
REM .claude/lessons/feedback_windows_e2e_requires_bat_wrapper.md). Same class
REM as the v1-SL-c-2 Phase 2 e2e incident; the fix is the same .bat pattern.
REM
REM Usage (from bash via cmd //c):
REM    LEMMY_DATABASE_URL=postgres://... cmd //c "scripts\brehon\migrate-roundtrip-cargo.bat"
REM
REM LEMMY_DATABASE_URL is read from the environment (a normal env var DOES
REM propagate to this child process; only the DLL search %PATH% does not,
REM which is exactly what this wrapper fixes).
REM
REM If the VS Build Tools / vcpkg paths below are wrong on another machine,
REM update them (parity with cargo-check.bat lines 16-22).
REM ---- libpq discovery (parity with cargo-check.bat) -----------------------
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
where libpq
echo ---
cd /d "%~dp0..\.."
"%USERPROFILE%\.cargo\bin\cargo.exe" run -p lemmy_diesel_utils --features full
exit /b !errorlevel!
