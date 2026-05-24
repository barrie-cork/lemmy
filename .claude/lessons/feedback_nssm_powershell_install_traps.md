---
name: NSSM-on-Windows via PowerShell driver — three quoting/redirect traps to pre-bake into the install helper
description: When installing a native exe as a Windows service via NSSM from a PowerShell driver, three traps fire in sequence — NSSM stderr-as-error, `$Args` automatic-variable shadowing, and spaces-in-Application install-line quoting. Pre-bake the helper around all three or expect 4+ failed attempts before SERVICE_RUNNING. Bonus footgun: NSSM services default to LocalSystem so `$env:USERPROFILE` resolves to `C:\WINDOWS\system32\config\systemprofile`, not the calling user.
type: feedback
---

## TL;DR

Wrapping a native exe as a Windows service via NSSM from a PowerShell driver script hits three traps in sequence. T1b NSSM install for pmd-http-mcp (2026-05-24) took **4 failed install attempts** to land at SERVICE_RUNNING; the traps compound because each fix unblocks the next trap in line. Build the driver script around all three from the start.

Bonus footgun: NSSM services run as LocalSystem by default — `$env:USERPROFILE` resolves to `C:\WINDOWS\system32\config\systemprofile`, NOT the calling user's profile.

## The three traps

### Trap 1 — NSSM writes ALL output (including normal status) to stderr

Under `$ErrorActionPreference = "Stop"` + native-exe-stderr handling, PowerShell 5.1 wraps each stderr line as an ErrorRecord (`NativeCommandError`) and aborts the script even when nssm exit was 0. Reading `$Error[0]` shows a confusing "command failed" message that doesn't match the actual exit code.

**FIX:** invoke nssm via `Start-Process -Wait -RedirectStandardError <file>` (streams go directly to files, not through PowerShell's pipeline) and check `$proc.ExitCode` explicitly. Do NOT use `2>&1` redirect — that merges stderr into the PowerShell pipeline where it triggers the same trap.

### Trap 2 — `$Args` is a PowerShell automatic variable inside functions

A `param([string[]]$Args)` declaration silently fails to bind — the automatic variable shadows the parameter, and `Start-Process -ArgumentList $Args` sees `$null`. No error; just silently passes nothing.

**FIX:** never name a function param `$Args`. Use `$NssmArgs`, `$CmdArgs`, etc. Same rule applies to other automatics: `$Input`, `$PSItem`, `$Matches`, `$Host`.

### Trap 3 — NSSM install with `Application` containing spaces

When you pass `"C:\Program Files\nodejs\node.exe"` as the install-line app path, PowerShell adds outer quotes via `-ArgumentList` and NSSM stores the QUOTED string in the service registry. CreateProcess then fails with "The system cannot find the file specified" because there's no executable at literal `"C:\Program`.

**FIX:** install with a placeholder app like `cmd.exe`, then overwrite via `nssm set <svc> Application <path>` and `nssm set <svc> AppParameters <args>` separately. Each `set` call handles quoting correctly because the value is parsed as a single arg, not a command-line.

### Bonus — LocalSystem service account + $env:USERPROFILE

NSSM services run as `LocalSystem` by default. `$env:USERPROFILE` inside the service process resolves to `C:\WINDOWS\system32\config\systemprofile`, NOT the calling user's profile. If the wrapped process needs to read a config/token from `%USERPROFILE%` (e.g. `~/.config/<app>/<key>`), the lookup will land in the wrong directory.

**FIX:** two options:
- Run the service as the user: `nssm set <svc> ObjectName .\<user> <password>`. Requires storing the password — non-trivial.
- Bake the value into `AppEnvironmentExtra` at install time: when the install script runs as the calling user via UAC, `$env:USERPROFILE` IS correct, so capture it into the service env block: `nssm set <svc> AppEnvironmentExtra "USERPROFILE=$env:USERPROFILE"`.

The second option is generally simpler unless the wrapped process actively walks `~/...` for many resources.

## Pre-bake the helper function (the cheap escape from this trap chain)

Future operational tasks installing OTHER NSSM services (e.g. ollama if ever local-served, junior daemon if ever Windows-side) will hit the same set. Bake the helper into the driver script up-front:

```powershell
function Invoke-Nssm {
  param(
    [Parameter(Mandatory=$true)][string[]]$NssmArgs   # NOT $Args
  )
  $stdout = [System.IO.Path]::GetTempFileName()
  $stderr = [System.IO.Path]::GetTempFileName()
  $proc = Start-Process -FilePath $nssm `
    -ArgumentList $NssmArgs `
    -Wait `
    -NoNewWindow `
    -RedirectStandardOutput $stdout `
    -RedirectStandardError $stderr `
    -PassThru
  if ($proc.ExitCode -ne 0) {
    Write-Host "nssm exit=$($proc.ExitCode)"
    Write-Host (Get-Content $stderr -Raw)
    Remove-Item $stdout, $stderr -ErrorAction SilentlyContinue
    throw "nssm $($NssmArgs -join ' ') failed with exit $($proc.ExitCode)"
  }
  Remove-Item $stdout, $stderr -ErrorAction SilentlyContinue
}

# Install with placeholder app to avoid Trap 3.
Invoke-Nssm install $svc cmd.exe

# Then set Application + AppParameters separately — Trap 3 avoided.
Invoke-Nssm set $svc Application 'C:\Program Files\nodejs\node.exe'
Invoke-Nssm set $svc AppParameters '"C:\path\to\script.js" --port 1234'

# Bake user-profile env (avoids LocalSystem profile mismatch).
Invoke-Nssm set $svc AppEnvironmentExtra "USERPROFILE=$env:USERPROFILE"

# Start.
Invoke-Nssm start $svc
```

## When to apply

Every new NSSM service install on Windows that wraps a native exe via a PowerShell driver. T1b shipped the pmd-http-mcp service; future Windows-side service installs (potential candidates: ollama, junior-daemon-side, vault, any node-served sidecar) will repeat the same trap chain unless the helper is pre-baked.

## Cross-references

- `.claude/lessons/pattern_cross_platform_divergences.md` — Windows ≠ Mac ≠ Linux (covers the cross-platform leg, including PowerShell idiom traps).
- PMD lesson 525 — canonical source of the four-attempt evidence trail; this file mirrors it for laptop session injection per `feedback_lesson_mirror_check.md`.
- `feedback_python_utf8_encoding_windows.md` — sibling Windows-specific footgun (Python stdout default codec).
- `feedback_batch_goto_eof_clobbers_errorlevel.md` — sibling Windows-shell trap (cmd batch).
- T1b session retro 2026-05-24 (`role-customization-2026-05-24-session3.md` §1b) — evidence trail.
