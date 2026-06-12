# Snapshots the user-scope Claude Code skill bodies into a TRACKED mirror so
# they get a git diff trail, CR-reviewability, and a rollback point.
#
# Why: user-scope skills live at ~/.claude/commands/*.md and ~/.claude/skills/*/
# (outside any git repo). The executable half of features like /auto-phase
# --unattended lives there, so without a snapshot it has NO version history,
# no review, and no rollback. This mirrors those files into
# .claude/user-skills-snapshot/ under brehon-fork, where `git diff` shows what
# changed. Per session-retro-2026-06-12-unattended-gate-allowlist.md
# What-to-change #2 + feedback_narrow_the_ask_before_building_mechanism.md.
#
# This is a SNAPSHOT, not a sync-back: the canonical source of truth stays
# ~/.claude/ (that's what Claude Code loads). The mirror is a read-only record.
# To restore a skill from the mirror, copy it back manually + review the diff.
#
# Modes:
#   (default)  snapshot - mirror ~/.claude skills into the tracked dir
#   -Check     drift-only - exit 1 if the mirror would change, mutate nothing
#
# Implementation note: uses robocopy /MIR (Windows-native mirror) rather than a
# hand-rolled hash-compare-and-prune loop. robocopy is purpose-built for this,
# propagates a documented exit-code bitmask (<8 = success), and removed a class
# of silent-no-op bugs the hand-rolled version hit (2026-06-12 debug session).
#
# Idempotent. Run from anywhere; resolves the repo root from this script's path.

param(
    [switch]$Check
)

$ErrorActionPreference = "Stop"

# Repo root = two levels up from this script (scripts/brehon/ -> repo root).
# .Path forces a string - Resolve-Path returns a PathInfo that misbehaves in
# path math and as a robocopy argument.
$repoRoot   = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$userClaude = Join-Path $env:USERPROFILE ".claude"
$mirror     = Join-Path $repoRoot ".claude\user-skills-snapshot"

if (-not (Test-Path $userClaude)) {
    Write-Host "FAIL: $userClaude does not exist - nothing to snapshot."
    exit 1
}

# What to mirror: the user-scope command bodies + the multi-file user skills.
# (Project-scope .claude/commands + .claude/skills are ALREADY tracked in-repo;
#  this script only covers the USER scope at ~/.claude that git can't see.)
# robocopy filters: commands = *.md only; skills = everything (multi-file skills
# carry SKILL.md + scripts + templates).
$jobs = @(
    @{ Rel = "commands"; Src = (Join-Path $userClaude "commands"); Files = @("*.md") },
    @{ Rel = "skills";   Src = (Join-Path $userClaude "skills");   Files = @() }
)

# robocopy exit codes are a bitmask: 0=no change, 1=copied, 2=extra removed,
# 3=copied+removed, 8+ = error. So "<8" means success. /L = list-only (dry run).
$anyChange = $false
$hadError  = $false

foreach ($job in $jobs) {
    if (-not (Test-Path $job.Src)) {
        Write-Host "skip: $($job.Src) not present"
        continue
    }

    $dest = Join-Path $mirror $job.Rel

    # Build robocopy args. /MIR mirrors (copies new/changed, prunes deleted).
    # /NJH /NJS /NDL /NP quiet the banner/summary/dir-list/progress noise.
    # /R:1 /W:1 keep retries minimal (these are local files).
    $rcArgs = @($job.Src, $dest) + $job.Files + @("/MIR", "/R:1", "/W:1", "/NJH", "/NJS", "/NDL", "/NP")
    if ($Check) { $rcArgs += "/L" }   # list-only: report what WOULD change

    $output = & robocopy.exe @rcArgs
    $rc = $LASTEXITCODE

    if ($rc -ge 8) {
        $hadError = $true
        Write-Host "ERROR: robocopy failed for $($job.Rel) (exit $rc):"
        $output | ForEach-Object { Write-Host "  $_" }
        continue
    }

    if ($rc -ge 1) {
        $anyChange = $true
        $verb = if ($Check) { "would change" } else { "synced" }
        Write-Host "$($job.Rel): $verb (robocopy code $rc)"
        # Show the changed file lines (robocopy prints them when files move).
        $output | Where-Object { $_ -match '\S' } | ForEach-Object { Write-Host "    $_" }
    } else {
        Write-Host "$($job.Rel): up to date"
    }
}

if ($hadError) {
    Write-Host "FAIL: one or more robocopy jobs errored - see above."
    exit 2
}

if ($Check) {
    if ($anyChange) {
        Write-Host ""
        Write-Host "DRIFT: user-skills snapshot is stale. Run scripts/brehon/sync-user-skills.ps1 (no -Check) to refresh, then commit."
        exit 1
    }
    Write-Host "OK: user-skills snapshot is up to date with ~/.claude."
    exit 0
}

if ($anyChange) {
    Write-Host ""
    Write-Host "Snapshot updated under .claude/user-skills-snapshot/."
    Write-Host "Next: review with 'git -C `"$repoRoot`" diff --stat .claude/user-skills-snapshot/' then commit on governance-v0."
} else {
    Write-Host "No changes - user-skills snapshot already current."
}
exit 0
