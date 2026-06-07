# PMD read-pheromone 24h monitor — one hourly snapshot of BOTH PMDs.
#
# Snapshots the laptop PMD (local sqlite3) and the EliteDesk PMD (over ssh),
# appends one JSON object per machine per run to a JSONL log. Designed to be
# invoked hourly by Windows Task Scheduler for ~24h after the read-pheromone
# rollout (2026-06-07), so we have data to tune LAMBDA / frequency-damping /
# the RRF nudge weight, and to catch regressions (FTS-storm, NULL read_count).
#
# Usage (manual):   powershell -File pmd-pheromone-monitor.ps1
# Usage (scheduler): registered via pmd-monitor-schedule.ps1 (sibling script)
#
# Output: .claude/runlog/pmd-pheromone-monitor.jsonl  (one line per machine per run)
# Analyze with: pmd-pheromone-analyze.ps1

$ErrorActionPreference = "Stop"
$LaptopDb = "C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db"
$ElitedeskDb = "/srv/project-memory/homeserver.db"
$OutDir = "C:/Users/barri/Developer/brehon-fork/.claude/runlog"
$OutFile = Join-Path $OutDir "pmd-pheromone-monitor.jsonl"
if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Force $OutDir | Out-Null }

# One SQL block that emits a single pipe-delimited metrics row. Written as a
# SINGLE LINE (no newlines) so it survives passing through `ssh homeserver`.
#
# Anomaly columns are the regression tripwires:
#  - bad_rc  = NULL or negative read_count (formula safety).
#  - fts_storm = the TRUE FTS-storm signature: a row read today whose updated_at
#    ALSO moved today but was created EARLIER. Deposit must never bump updated_at,
#    so a pre-existing row that's read-today must keep its old updated_at. We
#    exclude created_at>=today to avoid false positives from rows simply WRITTEN
#    today (their created_at==updated_at==today legitimately). >0 here = real bug.
$MetricsSql = "SELECT (SELECT COUNT(*) FROM memories) AS rows_total, (SELECT COUNT(*) FROM memory_vectors WHERE model='nomic-embed-text') AS vectors, (SELECT COUNT(*) FROM memories WHERE read_count>0) AS warm_rows, (SELECT COALESCE(SUM(read_count),0) FROM memories) AS read_sum, (SELECT COALESCE(MAX(read_count),0) FROM memories) AS read_max, (SELECT COUNT(*) FROM memories WHERE last_read_at >= date('now')) AS read_today, (SELECT COUNT(*) FROM memories WHERE read_count IS NULL OR read_count<0) AS bad_rc, (SELECT COUNT(*) FROM memories WHERE last_read_at >= date('now') AND updated_at >= date('now') AND created_at < date('now')) AS fts_storm"

# Top-5 most-read memories (id:count) — compact, for trend-spotting which
# memories the pheromone is actually elevating.
$TopSql = "SELECT id || ':' || read_count FROM memories WHERE read_count>0 ORDER BY read_count DESC, last_read_at DESC LIMIT 5;"

function Get-Snapshot {
    param([string]$Machine, [string]$MetricsRaw, [string]$TopRaw)
    $f = $MetricsRaw.Trim() -split '\|'
    $top = if ($TopRaw.Trim()) { ($TopRaw.Trim() -split "`n") | ForEach-Object { $_.Trim() } } else { @() }
    [pscustomobject]@{
        ts                 = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
        machine            = $Machine
        rows_total         = [int]$f[0]
        vectors            = [int]$f[1]
        warm_rows          = [int]$f[2]
        read_sum           = [int]$f[3]
        read_max           = [int]$f[4]
        read_today         = [int]$f[5]
        bad_rc             = [int]$f[6]
        fts_storm          = [int]$f[7]
        top5               = $top
    }
}

$lines = @()

# --- Laptop (local sqlite3) ---
try {
    $m = & sqlite3 $LaptopDb $MetricsSql
    $t = & sqlite3 $LaptopDb $TopSql
    $snap = Get-Snapshot -Machine "laptop" -MetricsRaw $m -TopRaw ($t -join "`n")
    $lines += ($snap | ConvertTo-Json -Compress -Depth 4)
} catch {
    $lines += (@{ ts=(Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ"); machine="laptop"; error="$($_.Exception.Message)" } | ConvertTo-Json -Compress)
}

# --- EliteDesk (over ssh) ---
# Windows OpenSSH strips shell quotes before the remote bash sees them, so any SQL
# with parens/quotes breaks. Quoting-proof workaround: base64-encode the SQL here,
# decode it remotely, and feed it to sqlite3 via stdin. No shell metacharacters
# traverse the ssh argv boundary.
function Invoke-RemoteSql {
    param([string]$Sql)
    $b64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($Sql))
    # remote: decode b64 -> sqlite3 reads SQL from stdin against the DB
    return (& ssh homeserver "echo $b64 | base64 -d | sqlite3 $ElitedeskDb")
}
try {
    $m = Invoke-RemoteSql -Sql $MetricsSql
    $t = Invoke-RemoteSql -Sql $TopSql
    $snap = Get-Snapshot -Machine "elitedesk" -MetricsRaw $m -TopRaw ($t -join "`n")
    $lines += ($snap | ConvertTo-Json -Compress -Depth 4)
} catch {
    $lines += (@{ ts=(Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ"); machine="elitedesk"; error="$($_.Exception.Message)" } | ConvertTo-Json -Compress)
}

Add-Content -Path $OutFile -Value $lines -Encoding utf8
Write-Output "snapshot appended ($($lines.Count) lines) -> $OutFile"
$lines | ForEach-Object { Write-Output $_ }
