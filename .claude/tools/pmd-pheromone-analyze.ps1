# Analyze the PMD pheromone monitor JSONL - summarize the 24h observation window.
# Produces the data to decide whether to tune LAMBDA / frequency-damp / RRF nudge,
# or to confirm the rollout is healthy. ASCII-only, concatenation not -f, to avoid
# Windows-PowerShell encoding/format-string parse traps.
#
#   powershell -File pmd-pheromone-analyze.ps1

$ErrorActionPreference = "Stop"
$LogFile = "C:/Users/barri/Developer/brehon-fork/.claude/runlog/pmd-pheromone-monitor.jsonl"
$LaptopDb = "C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db"

if (-not (Test-Path $LogFile)) { Write-Output "No monitor log yet at $LogFile"; return }

$snaps = Get-Content $LogFile | ForEach-Object { try { $_ | ConvertFrom-Json } catch {} } | Where-Object { $_ -and $_.machine }

foreach ($machine in @("laptop", "elitedesk")) {
    $rows = @($snaps | Where-Object { $_.machine -eq $machine -and -not $_.error } | Sort-Object ts)
    if ($rows.Count -eq 0) { Write-Output ("`n=== " + $machine + " : no snapshots ==="); continue }
    $first = $rows[0]; $last = $rows[-1]
    $pct = [math]::Round(100 * $last.vectors / [math]::Max(1, $last.rows_total), 1)
    $depositDelta = $last.read_sum - $first.read_sum
    $hrs = ([datetime]$last.ts - [datetime]$first.ts).TotalHours
    $rate = if ($hrs -gt 0) { [math]::Round($depositDelta / $hrs, 1) } else { "n/a" }
    $maxStorm = ($rows | Measure-Object -Property fts_storm -Maximum).Maximum
    $maxBad   = ($rows | Measure-Object -Property bad_rc -Maximum).Maximum
    $stormFlag = if ($maxStorm -gt 0) { "BUG: FTS-STORM x" + $maxStorm + " (deposit bumped updated_at)" } else { "0 clean" }
    $badFlag   = if ($maxBad -gt 0) { "BUG: bad read_count x" + $maxBad } else { "0 clean" }

    Write-Output ("`n=== " + $machine + " - " + $rows.Count + " snapshots, " + $first.ts + " -> " + $last.ts + " ===")
    Write-Output ("  rows         : " + $first.rows_total + " -> " + $last.rows_total)
    Write-Output ("  vectors      : " + $first.vectors + " -> " + $last.vectors + "  (" + $pct + " pct)")
    Write-Output ("  warm rows    : " + $first.warm_rows + " -> " + $last.warm_rows + "  (+" + ($last.warm_rows - $first.warm_rows) + ")")
    Write-Output ("  read_sum     : " + $first.read_sum + " -> " + $last.read_sum + "  (+" + $depositDelta + " deposits)")
    Write-Output ("  read_max     : " + $first.read_max + " -> " + $last.read_max)
    Write-Output ("  deposit rate : " + $rate + " per hour")
    Write-Output ("  ANOMALY fts_storm (max): " + $stormFlag)
    Write-Output ("  ANOMALY bad_rc    (max): " + $badFlag)
    Write-Output ("  top-read (latest): " + ($last.top5 -join ', '))
}

# --- Live decay-efficacy probe (laptop) -------------------------------------
# NOTE: this shows the RAW frequency*recency signal (ln(1+rc)*exp(-lambda*age))
# for trend legibility - it can exceed 1.0. The live ranking divides by
# FREQ_NORM=4.6 and caps at 1.0, THEN multiplies by the 0.0083 RRF weight, so the
# actual ranking influence is much smaller. Read this probe for SHAPE (which rows
# lead, how fast age_days grows), not absolute magnitude.
Write-Output "`n=== Live boost distribution (laptop, top 10 by current RAW boost) ==="
$boostSql = "SELECT id, importance, read_count, round(ln(1+read_count) * exp(-0.0231 * (julianday('now') - julianday(COALESCE(last_read_at, created_at)))), 4) AS boost, round(julianday('now') - julianday(COALESCE(last_read_at, created_at)), 2) AS age_days FROM memories WHERE read_count > 0 ORDER BY boost DESC LIMIT 10;"
& sqlite3 -header -column $LaptopDb $boostSql

Write-Output "`n=== Tuning notes ==="
Write-Output "  LAMBDA=0.0231 (30-day half-life). If after 24h the hottest memories age_days"
Write-Output "  is tiny and boosts cluster near ln(1+rc)/4.6, decay was not exercised yet -"
Write-Output "  judge LAMBDA over a longer window or via an aged fixture. If one memory boost"
Write-Output "  dwarfs the rest, consider a frequency cap. RRF nudge = 0.5/60 = 0.0083 (hybrid)."
