@echo off
REM Brehon dev utility: run `cargo clippy` with the Visual Studio 2022 Build
REM Tools linker on PATH. Lemmy's build needs `link.exe`, which only appears on
REM PATH after vcvars64.bat is sourced in the current shell.
REM
REM Usage (from any cmd.exe or from bash via `cmd /c`):
REM    scripts\brehon\cargo-clippy.bat -p <crate> --features full --no-deps -- -D warnings
REM
REM If the VS Build Tools path below is wrong on another machine, update it
REM before running.
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul
if errorlevel 1 (
    echo VCVARS_FAILED: vcvars64.bat returned errorlevel %errorlevel%
    exit /b 1
)
echo VCVARS_OK
where link
echo ---
cd /d "%~dp0..\.."
"%USERPROFILE%\.cargo\bin\cargo.exe" clippy %*
