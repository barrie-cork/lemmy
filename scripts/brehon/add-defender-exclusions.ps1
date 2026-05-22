# Add Windows Defender exclusions for Brehon Rust development.
# Per .claude/PRPs/reports/laptop-optimization-2026-05-22.md finding #3.
#
# MUST be run from an elevated PowerShell (Run as Administrator).
# Get-MpPreference can read; Add-MpPreference cannot write without admin.
#
# Idempotent: Add-MpPreference silently ignores duplicates.
#
# To run:
#   1. Press Win+X, choose "Windows PowerShell (Admin)" or
#      "Terminal (Admin)"
#   2. cd C:\Users\barri\Developer\brehon-fork
#   3. powershell -ExecutionPolicy Bypass -File scripts\brehon\add-defender-exclusions.ps1
#
# To rollback (also from elevated PS):
#   Remove-MpPreference -ExclusionPath "<path>"
#   Remove-MpPreference -ExclusionProcess "<exe>"

$ErrorActionPreference = "Stop"

# Confirm we're elevated; abort with a clear message if not.
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]"Administrator")
if (-not $isAdmin) {
    Write-Error "This script must be run from an elevated PowerShell (Run as Administrator). Aborting."
    exit 1
}

Write-Output "=== Adding Defender path exclusions ==="

# Per-worktree target/ dirs. Wildcard covers all current + future
# brehon-fork-<lane> worktrees, so new sub-phases don't need a fresh
# exclusion. Per .claude/rules/multi-lane-worktree.md the worktree
# naming is stable.
$paths = @(
    "C:\Users\barri\Developer\brehon-fork\target",
    "C:\Users\barri\Developer\brehon-fork-*\target",
    "C:\Users\barri\.cargo",
    "C:\Users\barri\.rustup",
    "C:\Users\barri\Developer\MCPs"
)

foreach ($p in $paths) {
    Add-MpPreference -ExclusionPath $p
    Write-Output "  + $p"
}

Write-Output ""
Write-Output "=== Adding Defender process exclusions ==="

# Process exclusions are faster than path-based suppression because
# Defender skips the file *and* anything the process emits during its
# lifetime — covers the entire rustc -> linker -> exe chain.
$processes = @(
    "rustc.exe",
    "cargo.exe",
    "rust-lld.exe",
    "link.exe",
    "cargo-clippy.exe"
)

foreach ($e in $processes) {
    Add-MpPreference -ExclusionProcess $e
    Write-Output "  + $e"
}

Write-Output ""
Write-Output "=== Verifying ==="

$pref = Get-MpPreference
Write-Output ""
Write-Output "ExclusionPath ($($pref.ExclusionPath.Count)):"
$pref.ExclusionPath | ForEach-Object { Write-Output "  $_" }

Write-Output ""
Write-Output "ExclusionProcess ($($pref.ExclusionProcess.Count)):"
$pref.ExclusionProcess | ForEach-Object { Write-Output "  $_" }

Write-Output ""
Write-Output "Done. Defender will skip these on the next file/process event."
Write-Output "No reboot required."
