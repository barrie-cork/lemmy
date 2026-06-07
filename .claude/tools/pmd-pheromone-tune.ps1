# PMD pheromone weight-TUNING analysis — consumes the read_events telemetry table.
# This is the script that turns per-event data into weight recommendations. Run it
# after a meaningful observation window (days, not the first hour) on whichever DB
# accumulated real search traffic (normally the laptop Brehon PMD).
#
#   powershell -File pmd-pheromone-tune.ps1 [-Db <path>]
#
# It answers the three tuning questions, each tied to a specific weight:
#   PHEROMONE_WEIGHT -> how often / how far does pheromone actually reorder results?
#   FREQ_NORM        -> is the read_count distribution wide enough that damping matters?
#   LAMBDA           -> do observed RE-READ intervals match the 30-day half-life?

param([string]$Db = "C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db")

$ErrorActionPreference = "Stop"
if (-not (Test-Path $Db)) { Write-Output "DB not found: $Db"; return }

$n = & sqlite3 $Db "SELECT COUNT(*) FROM read_events;"
Write-Output "=== read_events telemetry: $n events in $Db ==="
if ([int]$n -eq 0) { Write-Output "No events yet. Run some searches, then re-run after a few days of real traffic."; return }

# --- PHEROMONE_WEIGHT: reorder frequency + magnitude (hybrid only) -----------
Write-Output "`n--- PHEROMONE_WEIGHT signal (hybrid events only) ---"
Write-Output "How often does pheromone change the ranking, and by how much?"
& sqlite3 -header -column $Db @"
SELECT
  COUNT(*) AS hybrid_events,
  SUM(CASE WHEN rank_with != rank_without THEN 1 ELSE 0 END) AS reordered,
  ROUND(100.0 * SUM(CASE WHEN rank_with != rank_without THEN 1 ELSE 0 END) / COUNT(*), 1) AS reorder_pct,
  ROUND(AVG(ABS(rank_with - rank_without)), 2) AS avg_abs_rank_shift,
  MAX(ABS(rank_with - rank_without)) AS max_rank_shift
FROM read_events WHERE tool='hybrid' AND rank_without IS NOT NULL;
"@
Write-Output "Interpretation: reorder_pct near 0 => weight too weak (pheromone never matters);"
Write-Output "very high with large shifts => weight may be overriding relevance (raise RRF dominance)."

# --- FREQ_NORM: read_count distribution at read time -------------------------
Write-Output "`n--- FREQ_NORM signal (read_count distribution at read time) ---"
& sqlite3 -header -column $Db @"
SELECT
  ROUND(AVG(read_count_at),2) AS mean_rc,
  MAX(read_count_at) AS max_rc,
  (SELECT read_count_at FROM read_events ORDER BY read_count_at LIMIT 1 OFFSET (SELECT COUNT(*)/2 FROM read_events)) AS median_rc
FROM read_events;
"@
Write-Output "FREQ_NORM=4.6 assumes a hot memory reaches read_count~100. If max_rc stays low"
Write-Output "(say <20), the frequency term never saturates and damping is barely exercised —"
Write-Output "consider lowering FREQ_NORM so real read-counts produce a fuller [0,1] range."

# --- LAMBDA: observed re-read intervals vs the 30-day half-life --------------
Write-Output "`n--- LAMBDA signal (re-read intervals: gap between consecutive reads of same memory) ---"
& sqlite3 -header -column $Db @"
WITH ordered AS (
  SELECT memory_id, ts,
         LAG(ts) OVER (PARTITION BY memory_id ORDER BY ts) AS prev_ts
  FROM read_events
)
SELECT
  COUNT(*) AS reread_pairs,
  ROUND(AVG(julianday(ts) - julianday(prev_ts)), 2) AS mean_gap_days,
  ROUND(MAX(julianday(ts) - julianday(prev_ts)), 2) AS max_gap_days
FROM ordered WHERE prev_ts IS NOT NULL;
"@
Write-Output "LAMBDA=0.0231 => 30-day half-life. If mean_gap_days is, say, ~3, memories are"
Write-Output "re-read far faster than the half-life — boost barely decays between reads, so a"
Write-Output "SHORTER half-life (larger LAMBDA) would make recency more discriminating. Rule of"
Write-Output "thumb: set half-life ~ a few x the median re-read gap; LAMBDA = ln(2)/half_life_days."

Write-Output "`n=== Caveat (read before acting) ==="
Write-Output "These are USAGE-SHAPE signals, not an optimum. True optimization needs a RELEVANCE"
Write-Output "label (did the surfaced memory answer the need?). read_events has no such label yet."
Write-Output "Use this to spot gross mis-tuning (weight ineffective, freq never saturates, decay"
Write-Output "off by an order of magnitude), not to chase a precise optimum. A relevance signal"
Write-Output "(e.g. logging which surfaced memory was subsequently acted on) is the next step."
