# Handover — Role-customized Claude Code harness (T1 shipped, T1b + T2-T8 remaining)

**Author:** advisor session, governance-v0 canonical checkout, 2026-05-24
**For:** the next advisor session resuming this work (after compact / restart / `/clear` / lane switch)
**Goal:** ship the substrate so each Brehon role (advisor / planning / impl-task / bm-task / ci-watcher) can have its own startup context + tools, with RLS-driven evolution. **Minimal start, no strips yet — substrate first, optimisation later.**

## 1. Resume in three reads

1. **The plan** — `C:\Users\barri\.claude\plans\i-am-interesting-assessing-ethereal-shore.md`. The full 5-task plan, approved in plan mode by the user. **The plan IS the contract for this whole effort** — do not deviate without surfacing first. The first 50 lines explain the user's strip-aggression choice (minimal) and the signal-substrate framing.
2. **This handover** (you're reading it) — what's done, what's next, what was learned along the way.
3. **The MCP commit** — `barrie-cork/project-memory-mcp` @ `67b0019`. `git -C C:/Users/barri/Developer/MCPs/project-memory-mcp show --stat 67b0019` shows the three files. The commit body is the canonical reference for the HTTP transport design.

If you read only one paragraph: **The HTTP MCP transport capability is shipped and proven cross-machine (laptop ↔ EliteDesk over Tailscale). Both `.mcp.json` files are deliberately NOT yet repointed — that's the user-gated "go-live" step T1b. T2 onward (the role-substrate, signal hooks, consumers) is independent of the go-live and can start in parallel.**

## 2. What's done (T1a)

### 2.1 MCP server has an HTTP transport

`C:/Users/barri/Developer/MCPs/project-memory-mcp/src/index.ts` rewritten to branch on `PMD_TRANSPORT` env var:

- `stdio` (default, unchanged) — preserves every existing MCP client config.
- `http` — `StreamableHTTPServerTransport` from `@modelcontextprotocol/sdk/server/streamableHttp.js`, wrapped in `node:http.createServer`. Stateful sessions per `jsonResponseStreamableHttp.js` SDK example: per-request transport map keyed by `mcp-session-id`, new `McpServer` instance per session, `transport.onclose` drops the entry from the map.

**Why stateful and not stateless:** I tried stateless first (one transport, shared across all requests). The SDK rejected `tools/list` follow-ups with 500 because the streamable transport tracks per-session message routing internally. The canonical pattern in the SDK's own examples is stateful, and it matches our actual concurrency model (each Claude Code client process holds one MCP session for its lifetime — Junior worktrees naturally get their own sessions).

**Auth model:** `PMD_HTTP_TOKEN` env var → required as `Authorization: Bearer <token>` header for non-loopback requests. Loopback (127.0.0.1, ::1, ::ffff:127.0.0.1) requests skip the bearer check — so the laptop advisor against `http://localhost:11435/mcp` works tokenless while EliteDesk Junior across Tailscale must present the token.

**Why not mTLS or per-client tokens:** YAGNI for v0 — single shared secret protecting one home-network service. If we onboard a third device or want per-client revocation later, that's a follow-up. The current code path makes it trivial — `req.headers["authorization"]` is the only check.

### 2.2 Launcher scripts

- `MCPs/project-memory-mcp/scripts/start-pmd-http-server.ps1` — Windows. Reads token from `%USERPROFILE%\.config\project-memory-mcp\token`, sets all required env vars, runs `node dist/index.js`.
- `MCPs/project-memory-mcp/scripts/start-pmd-http-server.sh` — Linux/macOS. Same shape, `$HOME/.config/project-memory-mcp/token`.

Both check the token is ≥32 chars; both refuse to start if `dist/index.js` is missing (forcing `npm run build` first).

### 2.3 End-to-end cross-machine round-trip proven

- **Loopback laptop:** `initialize` → `tools/list` → `memory_write` (sentinel id 516) → `memory_search` round-tripped successfully via HTTP.
- **Cross-machine:** from EliteDesk over Tailscale to `http://100.104.171.26:11435/mcp` with bearer header → `initialize` (32ms RTT) → `memory_write` returned id **517** → row visible from laptop side via `better-sqlite3` direct read.

The sentinel rows (516, 517, both tagged `t1-smoke,sentinel`) have been marked `expires_at = 2026-05-25 00:00:00` and will auto-prune at next `memory_prune` run.

### 2.4 No firewall rule needed (surprising — investigate before rollout)

The first cross-machine `curl` from EliteDesk (server not yet up) timed out — classic "Windows Defender drops inbound" signature. But once the server was listening, the second cross-machine `curl` got 200 OK in 32ms. **So node.exe inbound on 11435 is permitted with no explicit firewall rule.**

Most likely causes:
- The Tailscale virtual interface might be classified as "Private" or "Domain" network in Windows, which has more permissive defaults than "Public".
- A prior dev-tool installation (node.js installer, VS Code, etc.) may have auto-created a broad allow rule for node.exe.

**Action for T1b:** before go-live, run `Get-NetFirewallRule | Where-Object { $_.DisplayName -like '*node*' }` from an admin shell to confirm what rule is actually permitting it. If it's a broad node.exe rule, we may want to tighten to a port-specific rule scoped to the Tailscale interface only (the script `New-NetFirewallRule` invocation in section 3.1 needs admin and was deferred).

### 2.5 Commit

```
commit 67b0019 (barrie-cork/project-memory-mcp main, pushed)
Date:   2026-05-24
Subject: feat(transport): add HTTP transport via StreamableHTTPServerTransport
Files:  src/index.ts (+147 -10)
        scripts/start-pmd-http-server.ps1 (+49)
        scripts/start-pmd-http-server.sh (+57)
```

stdio path is byte-for-byte preserved; backward compatible.

## 3. What's NOT done (intentionally)

### 3.1 T1b — Go-live (user-gated, do NOT auto-execute)

The capability is there. The switch from stdio to HTTP is **a deliberate user-driven step** because flipping it without a 24/7 server provisioned would instantly break:
- Laptop advisor session boot (MCP-init failure → no `mcp__project-memory__*` tools)
- All Junior daemon dispatches (every Junior task fails MCP-init → can't write retros → Stop hook false-blocks)

**Procedure when user is ready:**

1. **Provision the server lifecycle.** Three options, user picks one:
   - **NSSM service** (recommended for "set and forget"): `nssm install pmd-http-mcp "C:\Program Files\nodejs\node.exe" "C:\Users\barri\Developer\MCPs\project-memory-mcp\dist\index.js"`, set env vars via `nssm set pmd-http-mcp AppEnvironmentExtra PMD_TRANSPORT=http PMD_HTTP_PORT=11435 ...`, `nssm start pmd-http-mcp`. Auto-restarts on crash, survives logoff.
   - **Windows Task Scheduler** with "at logon" trigger running the `.ps1` launcher. Loses server during logoff.
   - **Manual terminal** running `pwsh scripts/start-pmd-http-server.ps1` in a kept-open window. Simplest; loses server when terminal closes or laptop sleeps.

2. **Confirm/tighten firewall.** Per §2.4 — verify what rule is permitting inbound 11435 from Tailscale. If broad, narrow it:
   ```powershell
   # MUST run as admin
   New-NetFirewallRule -DisplayName "project-memory-mcp HTTP (Tailscale)" `
     -Direction Inbound -Protocol TCP -LocalPort 11435 -Action Allow `
     -InterfaceAlias "Tailscale" -Profile Any
   ```

3. **Edit laptop `C:/Users/barri/Developer/brehon-fork/.mcp.json`** — replace the project-memory entry:
   ```json
   "project-memory": {
     "type": "http",
     "url": "http://localhost:11435/mcp"
   }
   ```
   No token needed (loopback bypass). Restart any open Claude Code session.

4. **SSH to EliteDesk and edit `/srv/brehon-fork/.mcp.json`** — replace project-memory entry + add missing servers (the EliteDesk copy is missing junior-brehon, ref-context, tavily — found during recon). Token to use is at `C:\Users\barri\.config\project-memory-mcp\token` on the laptop (current value below in §5, redacted form). Schema for the new entry:
   ```json
   "project-memory": {
     "type": "http",
     "url": "http://100.104.171.26:11435/mcp",
     "headers": {
       "Authorization": "Bearer <copy-token-here>"
     }
   }
   ```

5. **Restart Junior daemon** so it re-reads `.mcp.json` per worktree: `ssh homeserver 'sudo systemctl restart junior@brehon-fork.service'`. Queue a throwaway `[role:bm-task]` task that writes a sentinel memory; from laptop confirm it appears via `memory_search` for the sentinel string.

6. **Pin the canonical-PMD invariant for EliteDesk.** Once the EliteDesk `.mcp.json` is HTTP-pointed, `PROJECT_MEMORY_DB` and `OLLAMA_URL` env vars in the daemon-side `.mcp.json` are MOOT — those resolve server-side on the laptop now. But the `pmd-canonical-guard.sh` SessionStart hook only checks for the literal absolute Windows path; it needs a Linux-side sibling that asserts the `project-memory` server is `type: http` and `url` matches the laptop's Tailscale IP. **Author this as part of T1b** — a new `.claude/hooks/pmd-canonical-guard-linux.sh` that mirrors the Windows version's intent for the EliteDesk case. Wire it via the Junior `settings.local.json` template (whatever EliteDesk bootstrap does for Junior worktrees — needs investigation; sub-task §1 of T1b).

### 3.2 Why I didn't repoint laptop's `.mcp.json` now

Same logic as not repointing EliteDesk's: the moment I edit it, any new Claude Code session boot tries HTTP. If the user closes the terminal running the launcher, the next session boot fails. **A persistent-server lifecycle decision is the user's call, not the agent's.**

## 4. What's next (T2–T8)

T2 onward is fully independent of the T1b go-live (the role substrate, signal hooks, and consumer commands can all be authored against the existing stdio MCP and reuse the same code paths after go-live). **Recommend starting T2 immediately while T1b waits on user provisioning.**

### T2 — `.claude/roles/<role>/` substrate (4 dirs × 5 files = 20 new tracked files)

Per plan §Task 2. Direct on `governance-v0` (meta-edit per `phase-branch.md`).

For each of `planning`, `impl-task`, `bm-task`, `ci-watcher`:

```
.claude/roles/<role>/
├── manifest.yaml          # version metadata, role description, signal-emission settings
├── rules.allowlist        # one rule filename per line; BASELINE = ALL 24 .claude/rules/*.md filenames
├── mcp.json               # BASELINE = copy of current .mcp.json
├── system-prompt.md       # BASELINE = empty
└── README.md              # role purpose, when to edit, link to RLS feedback loop
```

**Baseline must be deliberately identical to current behaviour.** This task creates the durable git-tracked home for FUTURE edits, not strips. Each future strip is a tracked commit whose subtree-SHA (`git rev-parse HEAD:.claude/roles/<role>`) becomes the natural `config_version` for signal attribution.

**Watchpoints:**
- `rules.allowlist` should be the output of `ls .claude/rules/*.md | xargs basename -s .md` at authoring time, one per line. Order doesn't matter; the daemon will sort when assembling later.
- `mcp.json` baseline copy is just a `cp` from the canonical `.mcp.json` — but the canonical is gitignored, so this means writing the same content (without secrets) into the role-specific file. **Strip secrets** (`PMD_HTTP_TOKEN`, `x-ref-api-key`, any Anthropic key) from the role-specific copy; the daemon must read those from env at dispatch time.
- README.md should explicitly say "edits here change Junior dispatch starting next session; the daemon does NOT honour this manifest yet in iteration 1 — T5 lands that". Otherwise a future reader will think it's wired up.

Verify: `ls .claude/roles/{planning,impl-task,bm-task,ci-watcher}/` shows the four dirs populated; `git log -1 .claude/roles/impl-task/` resolves the new commit's SHA.

### T3a — `write-role-signal.js` CLI in project-memory-mcp

Per plan §Task 3 hook-side concern. Hooks are bash; bash can't natively call MCP tools. Need a standalone Node CLI that:

- Mirrors the `dist/scripts/backfill.js` pattern (already in the repo at `MCPs/project-memory-mcp/src/scripts/backfill.ts`).
- Reads `PROJECT_MEMORY_DB`, `PROJECT_ROOT` from env.
- Takes args: `--role <X> --kind <startup-cost|utilisation|outcome> --task-id <id> --branch <X> --config-version <sha> --content '<json or text>'`.
- Opens DB with `better-sqlite3`, inserts directly into `memories` table with `memory_type='summary'` (90d default expiry — see schema in `MCPs/project-memory-mcp/src/db.ts` line ~10 for the CHECK constraint enum).
- Tags = `role:<X>,kind:<...>,task_id:<id>,branch:<X>,config_version:<sha>`.
- Exits 0 on success, non-zero with stderr message on failure.

This is the laptop-side write tool. **After T1b go-live**, hooks running on EliteDesk would need to either (a) reach the laptop's HTTP MCP, which means an HTTP-flavoured `write-role-signal-http.sh` using curl, or (b) drop to a JSONL queue file (per the concurrency-fallback design in the plan) for the laptop to drain later. **Author both for T3b.**

Add to `MCPs/project-memory-mcp/package.json` `"bin"` so `npx write-role-signal ...` works (optional polish).

Build via `npm run build`. Commit on `barrie-cork/project-memory-mcp:main` direct (no PR flow — solo).

### T3b — Three signal-emission hooks (with JSONL-queue fallback)

Per plan §Task 3. Three new shell scripts:

- `.claude/hooks/role-signal-startup.sh` (SessionStart) → emits `kind:startup-cost`. Inventories `.claude/rules/*.md`, `.mcp.json` server names, `.claude/skills/*/SKILL.md`; estimates startup-token cost via `wc -c | awk '{print int($1/4)}'`. Calls `write-role-signal` (or its HTTP/queue sibling).
- `.claude/hooks/role-signal-utilisation.sh` (Stop, BEFORE retro) → emits `kind:utilisation`. Reads the session transcript JSONL at `~/.claude/projects/<encoded-cwd>/<session-uuid>.jsonl` (path on stdin from Stop hook). Greps `tool_name == "Read"` for `.claude/rules/*.md` patterns; greps `tool_name` prefix `mcp__` for MCP usage; emits aggregated row.
- `.claude/hooks/role-signal-outcome.sh` (Stop, AFTER retro) → emits `kind:outcome`. Diffs `.claude/decision-queue.json` between `git merge-base <branch> governance-v0` and `HEAD` for DQ writes; counts blockers. Reads the just-written `Task retro:%` row from PMD for exit_status. Emits structured fields.

**JSONL-queue fallback** (concurrency-safe per the plan): if the HTTP MCP is unreachable (`curl -m 2 -sS -o /dev/null -w "%{http_code}" http://<laptop>:11435/mcp/health || echo "down"` returning non-2xx), each hook appends a single JSONL row to `/srv/brehon-fork/.claude/role-signal-queue.jsonl` (EliteDesk) or `C:/Users/barri/Developer/brehon-fork/.claude/role-signal-queue.jsonl` (laptop). A weekly-review step (T4b) drains the queue into PMD when the server is back. **Queue is gitignored.**

**Hook contract:** all three hooks MUST exit 0 even on failure (signals are advisory; don't block the session). Failures go to stderr and the queue file.

### T3c — Wire hooks into Junior `settings.local.json` template

The EliteDesk daemon spawns Junior workers with their own `.claude/settings.local.json`. That file is generated by `/srv/brehon-fork/scripts/junior-spawn.sh` (or wherever the daemon's worktree-init lives — **needs investigation**, was flagged as a gap in the original recon pass). Add SessionStart + Stop hook entries pointing at the three new scripts.

**DO NOT wire these into the laptop's `settings.local.json`** — the laptop advisor session is the *consumer* of role signals, not the *subject*. (Unless the user wants advisor-self-tracking too — surface as a question if T3c authoring suggests it'd be cheap.)

### T4a — `/check-role-health <role>` slash command

User-scope at `~/.claude/commands/check-role-health.md`. Reads last 30d of `role-signal` rows for `<role>` (via PMD), computes:
- median loaded-rules count vs median used-rules count → the strip surface.
- MCPs loaded but never invoked → strip candidates.
- failure-rate trend over time.
- DQ-blocker-rate trend.

Returns a one-screen markdown report naming specific rule files as strip candidates. **Advisory only** — does not edit `.claude/roles/<role>/`.

### T4b — Extend weekly-review skill with role-health step

Add a new step in `.claude/skills/weekly-review/SKILL.md` (verify path) that runs `/check-role-health` for each role and includes the summaries in the weekly digest. Also **drains the JSONL signal queue** (per T3b concurrency-fallback design) before the role-health step runs — otherwise the queue accumulates indefinitely.

### T5 — End-to-end verification

Five verification steps from plan §Verification. Most can't run until ~20+ Junior dispatches have accumulated under T3-instrumented dispatch, so this is the long-tail "ship and watch" step. The structural verification (substrate exists, hooks fire on one dispatch, signal triad lands in PMD) can be done immediately after T3c.

## 5. Operational facts (verified during T1a)

| Fact | Value |
|---|---|
| Laptop Tailscale IPv4 | `100.104.171.26` |
| MCP HTTP port | `11435` |
| Token file (laptop) | `C:\Users\barri\.config\project-memory-mcp\token` (48 hex chars, generated 2026-05-24) |
| Canonical PMD path (laptop, absolute) | `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` |
| EliteDesk daemon's `.mcp.json` path | `/srv/brehon-fork/.mcp.json` (gitignored — manually maintained, last touched Apr 30 17:50) |
| EliteDesk MCP build path | `/home/barrie/MCPs/project-memory-mcp/dist/index.js` (NOT a git checkout, scp/sync'd Apr 11) |
| Ollama endpoint (laptop env) | `http://homeserver:11434` (Tailscale-routed; nomic-embed-text 768-dim) |
| MCP SDK version | `@modelcontextprotocol/sdk` 1.12.0 — ships `StreamableHTTPServerTransport` |
| Round-trip latency EliteDesk→laptop HTTP | 32ms (single test, in 32ms includes initialize handshake) |

**Token is NOT in this file or any committed file** — read it from `C:\Users\barri\.config\project-memory-mcp\token` at go-live time. Treat as a shared secret (same posture as `PMD_HTTP_TOKEN` mentions in launcher script comments).

## 6. Reconciled design decisions (do NOT re-litigate without user input)

| Decision | Choice | Source |
|---|---|---|
| Strip aggression | Minimal — substrate first, no strips in iteration 1 | User answer, Q2 pre-plan |
| Self-improve home | `.claude/roles/<role>/` tracked in repo | User answer, Q2 pre-plan |
| Signal store | PMD via `memory_write` (`memory_type: "summary"`) | User answer, Q2 pre-plan |
| Config version | Git SHA of `.claude/roles/<role>/` subtree | User answer, Q2 pre-plan |
| Sub-phase shape | Direct meta-edit on `governance-v0`, no plan file, no PR | User answer; per `phase-branch.md` "Direct on `governance-v0`" |
| Step 1 vs planning Junior | Advisor implements directly, skip planning Junior | User answer post-plan |
| Cross-machine transport | HTTP MCP (not SSHFS) | User answer, pre-implementation Q |
| HTTP scope | Both machines on HTTP (not "laptop stdio + EliteDesk HTTP") | User answer, pre-implementation Q |
| HTTP session mode | Stateful per-session (per SDK canonical example) | Empirical — stateless 500'd on follow-up `tools/list` |
| Auth | Bearer token via `PMD_HTTP_TOKEN`, loopback bypass | Author decision; documented in commit body |
| Repoint timing | Capability ships now (T1a); repoints are user-gated (T1b) | Author decision after weighing "server-lifecycle dependency on stdio→HTTP swap" risk |

## 7. Invariants you MUST preserve

These are not heuristics. From the auto-loaded rules + PMD invariants:

- **Canonical PMD path** stays `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db`. The HTTP transport doesn't change the storage path — it just routes writes from EliteDesk through HTTP into the same canonical file. `pmd-invariants.md` invariant #1 still rules. Never edit `retro-check.sh` to "fix" a stranded-retro symptom — the fix is always MCP-client-side (`.mcp.json`).
- **`answered_by: "advisor"`** in `.claude/decision-queue.json` is ONLY for the persistent advisor session. Hooks/CLIs writing role-signals must use the `memory_write` MCP tool, not DQ entries — role-signals are `memory_type: "summary"` rows, NOT DQ blockers.
- **`phase-branch.md` Direct vs PR flow:** `.claude/roles/`, `.claude/hooks/`, `.claude/commands/`, `.claude/skills/`, `MCPs/project-memory-mcp/scripts/` are all "Direct on `governance-v0`" or "direct on MCP repo main". The MCP server's `src/` is also direct (no PR flow visible in MCP repo history). The `crates/`, `migrations/`, `tests/` PR flow doesn't apply to this work at all.
- **Junior subagent contracts** (`.claude/agents/<name>.md`): the `tools:` allowlist on each subagent IS an MCP-tool allowlist — but inheriting at the parent level. The `.mcp.json` swap to HTTP doesn't change which tools subagents can call; only how those tool calls reach the server.
- **Hard refusal: never write `kind: "clarify"` in DQ from non-advisor.** Doesn't apply here (we're not writing DQ from Junior), just don't introduce a regression while authoring hooks.
- **Junior daemon's worker is forbidden from editing `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `.claude/PRPs/prds/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`** per `branch-manager.md`. None of T2-T8 touches these files; if T3b's hooks need to read git diffs of those paths, that's read-only.
- **Pre-compact handover discipline** (this file IS the handover). When the next session also approaches a `/compact` boundary, author its own handover BEFORE running compact, committed to `governance-v0` so the canonical checkout always sees the latest.

## 8. Open questions for the next session to consider (NOT escalations)

These don't need to be answered before T2 starts — surface them at gate moments:

1. **NSSM vs Task Scheduler vs manual terminal for the persistent HTTP server.** T1b user choice. The launcher script supports all three; only the lifecycle differs. Recommend NSSM for "set and forget"; manual terminal is fine for early iteration when restarts are easy.
2. **Tighten the firewall rule.** Per §2.4 — what's currently permitting inbound 11435 from Tailscale? If it's a broad node.exe allow, narrow it before go-live.
3. **Should the advisor session self-emit role-signals?** T3c only wires hooks into Junior's settings. The case for advisor-self-tracking: the strip surface for an `advisor` "role" is potentially larger than any Junior because the laptop carries the full 3000-line rule corpus. The case against: advisor is the *consumer*, not the subject; one of its jobs is to read role-signals, so it always touches the role-signal-related rules even when no other work would.
4. **Hook B (utilisation) — transcript path stability.** The plan assumes hook B can read `~/.claude/projects/<encoded-cwd>/<session-uuid>.jsonl`. This path is Claude Code-internal. If the format changes between CC versions, hook B breaks silently. **Defence:** wrap the parsing in `set +e` + a defensive `jq -r '.tool_name? // empty' 2>/dev/null`; if zero tool calls extracted, emit an empty utilisation row (still a signal — "couldn't parse").
5. **Signal-rate explosion.** Plan estimates 3 rows/dispatch × ~100/wk = 1200/mo. Weekly-review T4b should prune rows older than 90d (matches the `memory_type: "summary"` default expiry — so PMD auto-prunes them). Verify the `memory_prune` MCP tool already handles this; if not, T4b drives it explicitly.

## 9. Anti-patterns observed and avoided during T1a

For future-you to also avoid:

- **Don't write your own task-list polling.** The harness already notifies when background tasks complete. I started one server in background, got the notification on exit, did NOT sleep-poll.
- **Don't trust SDK feature claims without reading the canonical example.** I authored stateless HTTP mode first based on the type docs; it 500'd on `tools/list`. The SDK's `examples/server/jsonResponseStreamableHttp.js` shows the canonical stateful pattern. Always check examples before committing to a design.
- **Don't enable services that can break callers.** I had every opportunity to repoint laptop `.mcp.json` immediately after the smoke test passed. I didn't, because the server-lifecycle ownership is the user's call. Authoring infrastructure that depends on a persistent service the user hasn't provisioned is the same defect class as a lesson without its structural fix (per `feedback_lesson_must_pair_with_structural_fix_when_fixable.md`).
- **Killing background node processes affects every parallel CC session.** When I killed the smoke test process, four OTHER node.exe PIDs (other CC sessions' stdio MCPs) died too. They reconnect automatically on next tool call, but a session-in-flight that was mid-MCP-call may have seen a transient error. **Defence:** when killing background MCPs, narrow the filter to PIDs we know we started (background-task-id correlated PID, or unique CommandLine string).
- **Em-dashes in `memory_write` content** get mangled to `?` in PowerShell-passed JSON. The row was correct on disk; the terminal display was wrong. Use ASCII hyphens in titles authored programmatically, or quote-escape via heredoc.

## 10. Files modified this session

**MCP repo (`barrie-cork/project-memory-mcp`):**
- `src/index.ts` — rewritten (commit 67b0019)
- `scripts/start-pmd-http-server.ps1` — new (commit 67b0019)
- `scripts/start-pmd-http-server.sh` — new (commit 67b0019)

**Brehon repo (`barrie-cork/lemmy:governance-v0`):**
- `.claude/PRPs/handovers/role-customization-2026-05-24.md` — this file

**Laptop filesystem (not in any git):**
- `C:\Users\barri\.config\project-memory-mcp\token` — new (48-char hex secret)

**Sentinel PMD rows (will auto-prune 2026-05-25):**
- Memory id 516 (laptop loopback smoke)
- Memory id 517 (EliteDesk-over-Tailscale cross-machine smoke)

## 11. Resume from here

```
TaskList                  # see #2-#9, all pending
# Read .claude/PRPs/handovers/role-customization-2026-05-24.md (this file)
# Read C:\Users\barri\.claude\plans\i-am-interesting-assessing-ethereal-shore.md
# Start T2 unless user wants to drive T1b first
```

Default starting move: T2. T1b is user-gated and independent of T2 progress.
