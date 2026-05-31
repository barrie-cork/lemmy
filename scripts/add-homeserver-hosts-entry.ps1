# Adds the Tailscale IP -> homeserver mapping to the Windows hosts file.
# Must run elevated (admin). Idempotent: skips if the entry already exists.
#
# Why: the PMD HTTP server + tooling use OLLAMA_URL=http://homeserver:11434,
# but 'homeserver' is only an SSH-config alias (no DNS / hosts entry), so it
# does not resolve on Windows. Every memory_search_hybrid silently degrades
# to FTS5-only. This maps the name to the Tailscale IP so it resolves.

$ErrorActionPreference = "Stop"
$hostsPath = "$env:SystemRoot\System32\drivers\etc\hosts"
$entry = "100.81.145.58    homeserver"

$existing = Get-Content -Path $hostsPath -ErrorAction SilentlyContinue
if ($existing -match "homeserver") {
    Write-Host "hosts already contains a 'homeserver' entry — no change made:"
    $existing | Select-String "homeserver" | ForEach-Object { Write-Host "  $_" }
    exit 0
}

Add-Content -Path $hostsPath -Value "`r`n$entry" -Encoding ascii
Write-Host "Added to hosts: $entry"

# Verify resolution immediately
$resolved = (Resolve-DnsName homeserver -ErrorAction SilentlyContinue).IPAddress
if ($resolved) {
    Write-Host "homeserver now resolves to: $resolved"
} else {
    Write-Host "WARN: added entry but homeserver still not resolving — check the file manually."
}
