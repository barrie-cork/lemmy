# Register/unregister the PMD pheromone monitor as an hourly Windows Scheduled Task
# for a 24h observation window after the read-pheromone rollout.
#
#   Register:   powershell -File pmd-monitor-schedule.ps1
#   Unregister: powershell -File pmd-monitor-schedule.ps1 -Remove
#
# The task fires the snapshot script hourly. It auto-expires after 24h+a bit via
# an end boundary, so it does NOT need manual cleanup (but -Remove is provided).
# Runs in the user context (no admin) — Task Scheduler per-user tasks don't need
# elevation, unlike the NSSM service restart earlier.

param([switch]$Remove)

$TaskName = "PMD-Pheromone-Monitor-24h"
$Script   = "C:/Users/barri/Developer/brehon-fork/.claude/tools/pmd-pheromone-monitor.ps1"

if ($Remove) {
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
    Write-Output "Unregistered $TaskName"
    return
}

$action = New-ScheduledTaskAction -Execute "powershell.exe" `
    -Argument "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File `"$Script`""

# Hourly, starting 1 hour from now, repeating for 24 hours then stopping.
$start = (Get-Date).AddMinutes(60)
$trigger = New-ScheduledTaskTrigger -Once -At $start `
    -RepetitionInterval (New-TimeSpan -Hours 1) `
    -RepetitionDuration (New-TimeSpan -Hours 24)

$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries `
    -StartWhenAvailable -ExecutionTimeLimit (New-TimeSpan -Minutes 10)

Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger `
    -Settings $settings -Description "Hourly read-pheromone signal snapshot (both PMDs) for 24h post-rollout 2026-06-07" `
    -Force | Out-Null

Write-Output "Registered $TaskName"
Write-Output "  first run : $($start.ToString('yyyy-MM-dd HH:mm'))"
Write-Output "  cadence   : hourly for 24h"
Write-Output "  log       : .claude/runlog/pmd-pheromone-monitor.jsonl"
Get-ScheduledTask -TaskName $TaskName | Select-Object TaskName, State | Format-Table -AutoSize
