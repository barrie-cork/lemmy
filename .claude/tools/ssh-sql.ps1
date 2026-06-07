# ssh-sql.ps1 — run SQL (or any command string) on a remote host over ssh
# WITHOUT the Windows-OpenSSH quote-stripping class biting you.
#
# THE PROBLEM this kills:
#   Windows OpenSSH (the `ssh` that ships with Win10/11) strips one layer of
#   shell quotes from the command string BEFORE the remote bash ever sees it.
#   So `ssh homeserver "sqlite3 db.db \"SELECT COUNT(*) FROM t WHERE x>0;\""`
#   arrives at the remote as a mangled, half-unquoted string, and any SQL with
#   parens, quotes, `||`, `*`, or `>` breaks (silently, or with a cryptic
#   sqlite/bash error). This bit the pheromone-PMD monitor TWICE in one session
#   (2026-06-07) and recurs across prior memory (see the lesson cross-refs).
#
# THE FIX:
#   base64-encode the SQL/command on THIS side, ship the (alphanumeric-only)
#   base64 blob across the ssh argv boundary where no shell metacharacter can
#   be corrupted, and `base64 -d` it remotely before feeding to the target.
#   No shell metacharacter ever traverses the ssh boundary.
#
# USAGE
#   # Run SQL against a remote sqlite DB:
#   $rows = & .claude/tools/ssh-sql.ps1 -RemoteHost homeserver `
#             -Database /srv/project-memory/homeserver.db `
#             -Sql "SELECT COUNT(*) FROM memories WHERE read_count>0;"
#
#   # Run an arbitrary remote command (paren/quote-safe) without a DB:
#   $out = & .claude/tools/ssh-sql.ps1 -RemoteHost homeserver `
#             -Command "grep -rn 'reset --hard' /opt/junior-src/src/ | wc -l"
#
# Returns the remote stdout (string array, one element per output line), exactly
# as the underlying `ssh` call would. Throws on ssh failure (ErrorActionPreference).
#
# WHY a tool, not inline: the base64-over-ssh idiom is easy to forget and easy
# to get subtly wrong (forgetting the decode, double-quoting the blob). Centralise
# it once. Cross-ref: .claude/lessons/feedback_spec_deploy_facts_are_hypotheses.md
# (the session that motivated extraction) + the Invoke-RemoteSql original in
# .claude/tools/pmd-pheromone-monitor.ps1 (now a candidate to call this helper).
#
# GIT-BASH CALLER GOTCHA: if you invoke this via `powershell -File ssh-sql.ps1 ...`
# FROM Git-Bash/MSYS, a remote path arg like -Database /srv/x.db gets MSYS
# path-translated to C:/Program... before powershell sees it (you'll get
# "unable to open database C:/Program"). Prefix the invocation with
# MSYS_NO_PATHCONV=1, or call it from a real PowerShell session / Task Scheduler
# (the sibling pmd-*.ps1 tools all run from Task Scheduler, where this never bites).

[CmdletBinding(DefaultParameterSetName = "Sql")]
param(
    # Remote ssh target (an entry in ~/.ssh/config, e.g. "homeserver", or user@host).
    [Parameter(Mandatory = $true)]
    [string]$RemoteHost,

    # SQL to run against -Database via sqlite3 (the common case).
    [Parameter(Mandatory = $true, ParameterSetName = "Sql")]
    [string]$Sql,

    # Path to the sqlite DB ON THE REMOTE host. Required with -Sql.
    # NOT named -Db: that collides with CmdletBinding's auto -Debug alias.
    [Parameter(Mandatory = $true, ParameterSetName = "Sql")]
    [Alias("DbPath")]
    [string]$Database,

    # Arbitrary remote command string (paren/quote-safe). Mutually exclusive with -Sql/-Db.
    [Parameter(Mandatory = $true, ParameterSetName = "Command")]
    [string]$Command
)

$ErrorActionPreference = "Stop"

# The payload that runs remotely. For the SQL case we pipe decoded SQL into
# sqlite3 via stdin so the SQL itself never appears as a shell argument either.
if ($PSCmdlet.ParameterSetName -eq "Sql") {
    $payload = $Sql
    $b64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($payload))
    # Remote: decode the SQL blob -> feed to sqlite3 on stdin against $Database.
    # The DB path is a plain path (no metacharacters in practice); the risky
    # part (the SQL) travels as base64. If your DB path ever contains spaces,
    # quote it remotely: ... | sqlite3 "$Database"
    return (& ssh $RemoteHost "echo $b64 | base64 -d | sqlite3 $Database")
}
else {
    $payload = $Command
    $b64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($payload))
    # Remote: decode the command blob -> run it through bash. The command text
    # travels as base64 so its parens/quotes/redirects survive the ssh boundary.
    return (& ssh $RemoteHost "echo $b64 | base64 -d | bash")
}
