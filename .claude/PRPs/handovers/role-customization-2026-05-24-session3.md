# Handover — Role-customized harness (T1b shipped + verified; T4a + hook-fix pending)

**Author:** advisor session, governance-v0 canonical checkout, 2026-05-24 (session 3 — continuation of [session2](role-customization-2026-05-24-session2.md))
**For:** next advisor session resuming role-customization after fresh-session restart
**Goal:** ship the per-role context-management substrate + RLS-driven evolution.

## 1. Resume in three reads

1. [Session 2 handover](role-customization-2026-05-24-session2.md) — what session 2 shipped (T2, T3a, T3b+c), the lean-shape pivot rationale, and the original T1b-onward plan.
2. **This handover** — what session 3 shipped (T1b end-to-end), the two diagnostic findings on the role-signal Stop hook not firing for Junior bm-tasks, and the precise fixes needed.
3. PMD lesson 525 (`feedback_nssm_powershell_install_traps.md` once mirrored) — the 3 NSSM/PowerShell traps from this session's install iteration.

If you read only one paragraph: **T1b is shipped and verified end-to-end. NSSM service `pmd-http-mcp` runs on the laptop (port 11435, AUTO_START), both `.mcp.json` files repointed, Junior daemon restarted, cross-machine PMD write proven via sentinel `t1b-hook-100532-b85e5d` in row 529 (written by Junior bm-task #447 from EliteDesk via Tailscale to canonical laptop PMD). The role-signal-utilisation.sh Stop hook does NOT fire on Junior bm-task workers — two compounding root causes: (1) the hook's `junior/*` branch gate excludes bm-task workers because Stop-hook stdin `cwd` is the session's process cwd (`/srv/brehon-fork`, branch `governance-v0`) not the worker's worktree, and (2) the `write-role-signal` CLI is missing from EliteDesk's MCPs dir. T4a (`/check-role-health`) is still the next high-value piece, but the hook fix should land first so T4a has data to consume.**

## 2. What session 3 shipped

### 2.1 NSSM service for HTTP MCP transport (T1b §1-3)

`pmd-http-mcp` Windows service via NSSM 2.24-101 (winget-installed this session). Wraps `node C:/Users/barri/Developer/MCPs/project-memory-mcp/dist/index.js` directly with all env vars baked into `AppEnvironmentExtra` at install time. AUTO_START, 10 MB log rotation, idempotent installer.

Install script committed locally as `19a46c7` on `main` in `MCPs/project-memory-mcp`: `scripts/install-nssm-service.ps1` (169 lines). **NOT pushed** — local-only until you push or I do next session.

Verified end-to-end:
- HTTP probe from laptop (loopback, no auth): MCP `initialize` returns server identity `project-memory-mcp` v1.0.0.
- HTTP probe from EliteDesk (Tailscale `100.104.171.26:11435` + bearer): same response, 200 OK.

### 2.2 `.mcp.json` repoints (T1b §3)

- **Laptop** `C:/Users/barri/Developer/brehon-fork/.mcp.json`: `project-memory` server replaced with `{ "type": "http", "url": "http://localhost:11435/mcp" }`. No token (loopback bypass). All other entries unchanged.
- **EliteDesk** `/srv/brehon-fork/.mcp.json`: `project-memory` replaced with `{ "type": "http", "url": "http://100.104.171.26:11435/mcp", "headers": { "Authorization": "Bearer 4964f33f2de863cce274a6da1b69e1d1022e9d2b8ac7cf8e" } }`. `Ref` entry kept as-is (stale key but unused). `tavily` NOT added (minimal-start choice).
- Backup of pre-patch EliteDesk file: `/srv/brehon-fork/.mcp.json.bak-20260524-094339`.

### 2.3 Daemon restart + cross-machine smoke (T1b §4-5)

- `sudo systemctl restart junior@brehon-fork.service` — active since `08:44:31 UTC`.
- Smoke task #447 (sentinel `t1b-hook-100532-b85e5d`) succeeded. Row 529 in canonical PMD proves Junior bm-task worker on EliteDesk wrote via HTTP MCP transport to the laptop's canonical PMD.
- Prior smoke attempts: #445 (worker hallucinated PASS by doing config audit instead of `memory_write`); #446 (worker did the actual `memory_write`, sentinel landed but hook still didn't fire — surfaced the brief-imperative-clarity issue with Haiku).

### 2.4 Daemon checkout switched from phase-v1-RT-r2 to governance-v0

EliteDesk `/srv/brehon-fork` was parked on the closed `phase-v1-RT-r2` branch (`8e68d79cc`, predating `f6088a83d` the role-signal hook commit). v1-RT-r2 fully landed via PR #150 at 07:15:53Z 2026-05-24, so switching to trunk was safe. Now on `governance-v0` (`eea48f9a5`). This was part of the hook diagnosis but had to happen regardless.

## 3. The Junior-side role-signal hook fix (URGENT, next session)

### 3.1 Root cause #1 — branch gate excludes bm-task workers

Hook line 64: `if [[ ! "$CURRENT_BRANCH" =~ ^junior/ ]]; then exit 0; fi`. The `CURRENT_BRANCH` is `$(git rev-parse --abbrev-ref HEAD)` resolved against the Stop-hook process's cwd. Per Claude Code's Stop-hook stdin schema, the `cwd` field is the Claude SESSION's process cwd, not per-tool-call cwd.

For Junior bm-task workers, the session starts at `cwd: /srv/brehon-fork` (main checkout, currently on `governance-v0`). Even though the worker `cd`s into the per-task worktree (`/srv/brehon-fork/.junior/worktrees/job-N`) for individual Bash calls, the Stop hook still sees `governance-v0` as the branch — gate fires, exit 0, no signal emitted.

Evidence (task #447 daemon log `/srv/brehon-fork/.junior/logs/job-447-run-446.log`):
- Single `init` block with `cwd: /srv/brehon-fork`.
- Worker session `054cd040` read `/srv/brehon-fork/.junior/worktrees/job-447/.claude/hooks/role-signal-utilisation.sh` (hook script IS present in the worktree, settings.json wiring IS active).
- ZERO `Stop` hook events in log (`grep -oE '"hook_event":"[^"]*"' | sort | uniq -c` → `2 SessionStart` only).
- No `role-signal-queue.jsonl` fallback file written.

Manual reproduction: `ssh homeserver "cd /srv/brehon-fork && CLAUDE_PROMPT='[role:bm-task] test' bash .claude/hooks/role-signal-utilisation.sh <<< '{...}'"` exits 0 with no output — the junior gate fires because main checkout is on governance-v0.

### 3.2 Root cause #2 — write-role-signal CLI missing on EliteDesk

`/home/barrie/MCPs/project-memory-mcp/dist/scripts/` contains only `backfill.{d.ts,js}`. The `write-role-signal.js` from MCP commit `5cc7071` (laptop-side, on `main`) was never pulled+built on EliteDesk. Even if root cause #1 is fixed, the hook would fall through to JSONL queue (which would land at `/srv/brehon-fork/.claude/role-signal-queue.jsonl` per the hook code, currently absent).

### 3.3 Recommended fixes (in order)

1. **Build write-role-signal on EliteDesk**: `ssh homeserver "cd /home/barrie/MCPs/project-memory-mcp && git pull && npm install && npm run build && ls dist/scripts/write-role-signal.js"`. Verify the file appears.
2. **Patch the hook's gate logic** to not depend on cwd's git branch. Three options worth considering at next-session brief-author time:
   - **Option A (smallest change):** check `CLAUDE_PROMPT` env for `[role:` regex AND task-id pattern; skip the branch gate entirely. The advisor session also runs Stop hooks but the role-tag wouldn't be in its prompt.
   - **Option B (most accurate):** parse `transcript_path` stdin field, scan for the dispatch line's `[role:X]` and branch (the daemon includes the worker's intended branch in the worker's prompt context). Heavier but correct for all worker shapes.
   - **Option C (daemon-side fix):** modify the daemon binary to invoke `claude --print` with `cwd` set to the worktree, not the main checkout. Best long-term but requires daemon-code changes (which require coordinating with Junior daemon owner, not just a brehon-fork edit).
3. **Re-queue smoke task to verify**: another `[role:bm-task]` sentinel write; confirm BOTH the sentinel row appears AND a role-signal sibling row appears tagged with `role:bm-task,kind:utilisation,task_id:<N>`.

The fix is straightforward — pick Option A for iteration 1 unless something else surfaces.

## 4. What's still pending (carry-forward from session 2 + new from session 3)

### 4.1 T4a — `/check-role-health <role>` consumer (HIGH VALUE)

Unchanged from session 2 handover §3.2. Should land AFTER the hook fix above so it has data to consume. Once role-signal rows accumulate (target: ≥18 dispatches per role), T4a's "rules loaded but never read" / "MCPs loaded but never invoked" analyses become meaningful.

### 4.2 T4b — weekly-review skill extension

Unchanged. Lower priority than T4a.

### 4.3 T5 / T8 — Daemon honours rules.allowlist via `--bare`

Unchanged. Plan once T4a shows stable strip candidates.

### 4.4 Push MCP repo commits

Two unpushed commits on `main` in `barrie-cork/project-memory-mcp` need pushing if you want them durable:
- `5cc7071 feat(role-signal): add write-role-signal CLI` (session 2)
- `19a46c7 feat(nssm): add NSSM service installer for pmd-http-mcp` (session 3)
Push: `cd C:/Users/barri/Developer/MCPs/project-memory-mcp && git push origin main`.

### 4.5 Documentation drift (deferred from session 3 retro)

- **PMD lesson 525** (NSSM 3 traps) needs mirroring to `.claude/lessons/feedback_nssm_powershell_install_traps.md` per `feedback_lesson_mirror_check.md`.
- **PMD lesson 526** (infra drift: HTTP MCP transport on port 11435) is descriptive only — no required action, but useful for next-session context.
- **CLAUDE.md operational facts** could gain a one-line `pmd-http-mcp` NSSM service entry.

## 5. Operational facts (additions to session 2 §5)

| Fact | Value |
|---|---|
| NSSM | 2.24-101-g897c7ad, installed via winget; PATH-effective after shell restart, also at `C:\Users\barri\AppData\Local\Microsoft\WinGet\Packages\NSSM.NSSM_Microsoft.Winget.Source_8wekyb3d8bbwe\nssm-2.24-101-g897c7ad\win64\nssm.exe` |
| Service name | `pmd-http-mcp` (NSSM-managed, AUTO_START) |
| Service log paths | `C:\Users\barri\Developer\MCPs\project-memory-mcp\logs\{stdout,stderr}.log` (10 MB rotation) |
| Service install script | `C:/Users/barri/Developer/MCPs/project-memory-mcp/scripts/install-nssm-service.ps1` (idempotent; runs admin via Start-Process -Verb RunAs UAC) |
| Service manage commands | `nssm {status\|start\|stop\|restart} pmd-http-mcp`; `nssm remove pmd-http-mcp confirm` |
| Daemon main checkout branch | `governance-v0` (was `phase-v1-RT-r2` until this session's fix) — keep on trunk by default; phase branches are worktree-scoped per task |
| EliteDesk write-role-signal | MISSING — needs `cd /home/barrie/MCPs/project-memory-mcp && git pull && npm install && npm run build` |
| Worker's Stop-hook cwd | session's PROCESS cwd (`/srv/brehon-fork`), NOT worktree per-task cwd — root cause for junior/* gate misfire |

## 6. Files modified this session

**Brehon repo (`barrie-cork/lemmy:governance-v0`):**
- (none — `.mcp.json` is gitignored)
- This handover file (next commit)

**MCP repo (`barrie-cork/project-memory-mcp:main`):**
- `scripts/install-nssm-service.ps1` — new (commit `19a46c7`, NOT pushed)

**EliteDesk:**
- `/srv/brehon-fork/.mcp.json` — repointed to HTTP transport (gitignored)
- `/srv/brehon-fork/.mcp.json.bak-20260524-094339` — pre-patch backup (untracked)
- `/srv/brehon-fork` checkout switched from `phase-v1-RT-r2` → `governance-v0`
- `junior@brehon-fork.service` restarted

**Laptop:**
- `C:/Users/barri/Developer/brehon-fork/.mcp.json` — repointed to loopback HTTP (gitignored)
- NSSM service `pmd-http-mcp` installed and running

**PMD rows written this session (canonical laptop DB):**
- 525 — lesson: NSSM-on-Windows via PowerShell — three quoting/redirect traps
- 526 — infra-drift: project-memory MCP moved to HTTP transport on port 11435
- 527 — Task retro: T1b HTTP MCP go-live
- 528 — Smoke #446 sentinel (`t1b-smoke-20260524-095755-06eceb7c`)
- 529 — Smoke #447 sentinel (`t1b-hook-100532-b85e5d`)

## 7. Open question for next session

**Worker's `base_branch` parameter is ignored for bm-tasks?** I passed `base_branch=governance-v0` on all three smoke tasks (#445, #446, #447). For #446 the daemon's finalize-agent reported `Base branch: phase-v1-RT-r2` despite my parameter — because the daemon's main checkout was on that branch at the time. After I switched the checkout to `governance-v0`, task #447's finalize-agent also reported `governance-v0`. So the parameter MAY work but is OVERRIDDEN by the daemon's current checkout state. Not blocking — but worth knowing the daemon's main-checkout branch is load-bearing for worker context, not just the parameter. May warrant a dedicated session retro lesson.

## 8. Resume from here

```
TaskList                            # see what's pending
Read .claude/PRPs/handovers/role-customization-2026-05-24-session3.md (this file)
Read .claude/PRPs/handovers/role-customization-2026-05-24-session2.md (session 2)
# Recommended next: §3.3 — build write-role-signal on EliteDesk → patch hook gate → smoke re-verify
# Alternative: T4a /check-role-health authoring (works against laptop PMD even without hook firing)
```
