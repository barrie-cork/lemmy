# Handover — Role-customized harness (T2, T3a, T3b+c shipped; T1b + T4 + T8 remaining)

**Author:** advisor session, governance-v0 canonical checkout, 2026-05-24 (session 2 — continuation of session 1's [role-customization-2026-05-24.md](role-customization-2026-05-24.md))
**For:** the next advisor session resuming this work (after fresh-session restart per user request 2026-05-24)
**Goal:** ship the per-role context-management substrate + RLS-driven evolution. Iteration 1 = lean shape: substrate (T2 ✓) + one CLI (T3a ✓) + one Stop hook (T3b+c ✓) + one consumer command (T4 pending).

## 1. Resume in three reads

1. **The plan** — `C:\Users\barri\.claude\plans\i-am-interesting-assessing-ethereal-shore.md`. The full 5-task plan, approved in plan mode by the user. **The plan IS the contract** — but note iteration 1 deviates from the plan's 3-hook Task 3 design (see §4 "Lean-shape pivot" below).
2. **Session 1 handover** — [role-customization-2026-05-24.md](role-customization-2026-05-24.md). T1a (HTTP MCP transport) + remaining-task framing. Still applies for T1b (go-live procedure) + T8 (eventual `--bare` daemon dispatch).
3. **This handover** — what session 2 shipped (T2 substrate, T3a CLI, T3b+c hook) and the lean-shape pivot's rationale (Stop hooks fire in parallel per current CC docs, so the planned 3-hook ordering was structurally impossible).

If you read only one paragraph: **The substrate (T2), the laptop-side CLI (T3a), and one consolidated Stop hook (T3b+T3c) are shipped and proven (smoke rows 521-524 in PMD, all sentinel-marked, expire 2026-05-25). Stop hooks fire in parallel per current Claude Code docs — the original 3-hook plan was redesigned to one hook that captures everything the consumer (T4 /check-role-health) needs: rules read + MCPs invoked + DQ blockers added. T4 (consumer command) is the next high-value piece. T1b (HTTP MCP go-live: NSSM install + firewall + .mcp.json repoints) is independent and user-driven.**

## 2. What session 2 shipped

### 2.1 T2 — Role substrate (commit `1bb4db189` on governance-v0)

`.claude/roles/{planning,impl-task,bm-task,ci-watcher}/` × 5 files each (manifest.yaml, rules.allowlist, mcp.json, system-prompt.md, README.md). 20 files total. Baseline = current full-fat behaviour (all 24 rules in allowlist; verbatim copy of `.mcp.json` with secrets → `${ENV_VAR}` placeholders).

**Subtree SHAs at T2 baseline** (these are the first `config_version` anchors every signal row will carry until the first strip):

| Role | Subtree SHA |
|---|---|
| planning | `381dee052dfae927a0d70aab932a7eddaa139377` |
| impl-task | `a3d5b613e8fde29230ec6df52dc079dc801033da` |
| bm-task | `a6a21c07070ed33c4cb9157e1754e8b32b38ae97` |
| ci-watcher | `9c5e6d4d6ed2a9d3d1c49cfbe5c91f9132927e08` |

### 2.2 T3a — write-role-signal CLI in project-memory-mcp (commit `5cc7071` on main, pushed)

`src/scripts/write-role-signal.ts` + `package.json` `bin` entry. Mirrors `backfill.ts` shape (env resolution, better-sqlite3 open). Args: `--role --kind --task-id --branch --config-version --content` (JSON-validated). Inserts one `memory_type='summary'` row with tags `role-signal,role:X,kind:Y,task_id:Z,branch:W,config_version:S`. Exit 0 success (row id to stdout), 1 bad args, 2 DB error.

Built and tested. Dist path: `C:/Users/barri/Developer/MCPs/project-memory-mcp/dist/scripts/write-role-signal.js`.

### 2.3 T3b + T3c — one Stop hook + wiring (commit `f6088a83d` on governance-v0)

`.claude/hooks/role-signal-utilisation.sh` (the hook) + entry added to `.claude/settings.json` Stop array + `.claude/role-signal-queue.jsonl` added to `.gitignore`.

The hook:
- Self-gates on `junior/*` branches (advisor sessions and chore branches exit 0 immediately).
- Parses `transcript_path`, `session_id`, `cwd` from stdin JSON per current code.claude.com/docs/en/hooks.md schema.
- Extracts role from `CLAUDE_PROMPT` env (matches `[role:planning|impl-task|bm-task|ci-watcher]`).
- Resolves `config_version` via `git rev-parse HEAD:.claude/roles/<role>` (with a fix for the Windows quirk where missing refs print to STDOUT not stderr → falls back to `pre-t2`).
- Greps the transcript JSONL for `tool_use` events where `name == "Read"` and `file_path` matches `.claude/rules/*.md` → `rules_read[]`.
- Greps the transcript for `tool_use` events where `name` starts with `mcp__` → `mcp_tools_invoked[]`.
- Computes `dq_blockers_added` by jq-diffing `.claude/decision-queue.json` blocker count on this branch vs `governance-v0` base.
- Emits one PMD row via `write-role-signal` CLI. On any failure (CLI missing, exit non-zero), falls back to `.claude/role-signal-queue.jsonl`.
- Exits 0 always (signals are advisory; this hook never blocks Junior task completion).

Smoke-tested with rows 521-524 (all marked sentinel, expires 2026-05-25). Verified:
- Junior-only gate works (`governance-v0` returns early, no PMD write).
- `config_version` resolves correctly for both real subtree SHAs (`a6a21c07...` for bm-task) and the `pre-t2` fallback (synthetic temp-repo test).
- CLI round-trip works; row visible via direct sqlite3 query.

## 3. What's NOT done (pending tasks)

### 3.1 T1b — HTTP MCP go-live (user-driven, recon complete)

The capability is shipped (T1a commit `67b0019` on project-memory-mcp). Session 1 left it user-gated. Session 2 did recon:

- **NSSM not installed** on laptop. User chose NSSM lifecycle in session 2 prompts. Pre-req: `winget install --id=NSSM.NSSM` OR manual download from https://nssm.cc.
- **No firewall rule currently visible** for node.exe / port 11435. The Tailscale interface is classified "Private" (`Get-NetConnectionProfile` shows `InterfaceAlias=Tailscale, NetworkCategory=Private`), which has more permissive defaults than "Public". User chose to "investigate before any change" — so no tightening planned for iteration 1 unless evidence emerges.
- **EliteDesk `/srv/brehon-fork/.mcp.json`** is sparser than expected: only `project-memory` (local DB path — exactly the partition T1b fixes) and `Ref` (stale key). Missing: `junior-brehon` (correct — daemon IS that), `tavily`. Will need additions during repoint.
- **`/usr/local/bin/junior` is a stripped Node.js single-file binary** — daemon's worktree spawn logic is embedded, NOT a shell script we can patch. T3c bypassed this entirely by using the **tracked** `.claude/settings.json` (Junior worktrees inherit it via normal git checkout — no daemon-side template patching needed). Same model applies to T1b: the `.mcp.json` change is git-propagated, not daemon-injected.

**Remaining T1b steps (each is a discrete user-gated action):**

1. **Install NSSM** — user runs `winget install --id=NSSM.NSSM` as admin.
2. **Author NSSM install script** (task #12, pending — no apply, draft only). Shape:
   ```powershell
   # As admin
   $token = Get-Content C:\Users\barri\.config\project-memory-mcp\token -Raw
   nssm install pmd-http-mcp "C:\Program Files\nodejs\node.exe" "C:\Users\barri\Developer\MCPs\project-memory-mcp\dist\index.js"
   nssm set pmd-http-mcp AppDirectory "C:\Users\barri\Developer\MCPs\project-memory-mcp"
   nssm set pmd-http-mcp AppEnvironmentExtra "PMD_TRANSPORT=http" "PMD_HTTP_PORT=11435" "PMD_HTTP_TOKEN=$token" "PROJECT_MEMORY_DB=C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db" "OLLAMA_URL=http://homeserver:11434"
   nssm set pmd-http-mcp AppStdout "C:\Users\barri\Developer\MCPs\project-memory-mcp\logs\stdout.log"
   nssm set pmd-http-mcp AppStderr "C:\Users\barri\Developer\MCPs\project-memory-mcp\logs\stderr.log"
   nssm start pmd-http-mcp
   ```
   Verify: `nssm status pmd-http-mcp` returns `SERVICE_RUNNING`; `curl http://localhost:11435/mcp -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"initialize","params":{...},"id":1}'` returns 200.
3. **Draft .mcp.json repoint patches** (task #11, pending). Two files to edit at user gate:
   - `C:/Users/barri/Developer/brehon-fork/.mcp.json` (laptop) — replace `project-memory` entry with `{ "type": "http", "url": "http://localhost:11435/mcp" }`. No token (loopback bypass).
   - `/srv/brehon-fork/.mcp.json` (EliteDesk, via ssh) — replace `project-memory` with `{ "type": "http", "url": "http://100.104.171.26:11435/mcp", "headers": { "Authorization": "Bearer <token>" } }`. Also add missing `junior-brehon`/`tavily` if you want Junior workers to have those tools (probably yes — current EliteDesk `.mcp.json` only has `project-memory` + stale `Ref`).
4. **Restart CC sessions** (laptop) and **restart Junior daemon** (`sudo systemctl restart junior@brehon-fork.service`).
5. **Verify with sentinel task**: queue a throwaway `[role:bm-task]` Junior task that writes `memory_write_eval` with a sentinel string; from laptop confirm via `memory_search "<sentinel>"`. Row should appear (proving cross-machine HTTP path works) AND should have the role-signal sibling row from the new hook.

### 3.2 T4a — `/check-role-health <role>` consumer (HIGH VALUE, next high-value piece)

User-scope at `~/.claude/commands/check-role-health.md`. Per the lean-shape pivot (§4 below), this command joins:
- **What's loaded** — read `.claude/roles/<role>/rules.allowlist` + `mcp.json` at report time, `wc -c` each rule file, sum.
- **What's used** — query PMD for last-30d `tags LIKE '%role-signal%' AND tags LIKE 'role:<role>%' AND tags LIKE 'kind:utilisation%'` rows; parse content JSON; aggregate rules_read frequencies + mcp_tools_invoked frequencies.
- **Outcome** — join utilisation rows to existing `Task retro:%` rows (via `task_id` / `branch` tags) for success/partial/failure + retro score.
- **DQ-blocker rate** — sum `dq_blockers_added` field across utilisation rows, group by `config_version`.

Output: one-screen markdown report naming specific rule files / MCP servers as strip candidates ("never read across 18 dispatches"). Advisory only — does NOT edit `.claude/roles/<role>/`.

Until T4 ships, the signal rows accumulate in PMD but no one reads them. This is the consumer the whole substrate exists for.

### 3.3 T4b — weekly-review skill extension

Lower priority than T4a. Add a step to `.claude/skills/weekly-review/SKILL.md` (verify path) that runs `/check-role-health` for each role and includes summaries in the weekly digest. Also **drains the JSONL signal queue** at `.claude/role-signal-queue.jsonl` into PMD before the role-health step runs.

### 3.4 T5 / T8 — Daemon honours rules.allowlist (deferred, out of iteration 1 scope per user direction)

The `--bare` dispatch (`claude -p --bare --settings ... --mcp-config ... --append-system-prompt ... --allowedTools ...`) wired into Junior daemon spawn. Requires modifying the daemon binary OR adding a wrapper. Plan from real data once `/check-role-health` shows stable strip candidates across ≥20 dispatches per role.

## 4. Lean-shape pivot (rationale; do NOT re-litigate without user input)

The original plan §Task 3 specified THREE hooks: `role-signal-startup.sh` (SessionStart, kind:startup-cost), `role-signal-utilisation.sh` (Stop BEFORE retro, kind:utilisation), `role-signal-outcome.sh` (Stop AFTER retro, kind:outcome).

This design was structurally impossible under current Claude Code:

1. **Stop hooks fire in parallel** per https://code.claude.com/docs/en/hooks.md ("All matching hooks run in parallel, and identical handlers are deduplicated automatically"). There is no ordering mechanism between Stop hooks. So `utilisation BEFORE retro-check` and `outcome AFTER retro-check` could not be guaranteed.

2. **Stop hooks have no additionalContext output**. They can only block (`decision: "block"`) or allow. So one hook cannot influence what another hook sees.

User (this session, post-confirmation) explicitly framed the goal as **per-role context management**, not measurement for its own sake. That clarification collapsed the design to:

- **Startup-cost is derivable from `.claude/roles/<role>/` at any time** — read manifest, sum rule-file bytes. No hook needed; `/check-role-health` reads the role manifest at report time.
- **Outcome is derivable from existing `Task retro:%` rows joined on task_id/branch** — `retro-check.sh` already enforces a retro per Junior task with that title pattern. No need for a separate outcome row; the consumer joins.
- **One hook captures everything the consumer needs**: rules_read (for "loaded but never read" strip candidates), mcp_tools_invoked (for "loaded but never invoked" strip candidates), dq_blockers_added (the per-task autonomy-friction signal user specifically requested as useful).

Outcome of pivot: 1 CLI + 1 hook + 1 consumer = the substrate, vs 1 CLI + 3 hooks + 1 consumer + 1 queue-fallback in the original plan. Same RLS feedback loop, smaller surface area, no ordering games.

## 5. Operational facts (carried from session 1 + updated this session)

| Fact | Value |
|---|---|
| Laptop Tailscale IPv4 | `100.104.171.26` |
| MCP HTTP port | `11435` |
| Token file (laptop) | `C:\Users\barri\.config\project-memory-mcp\token` (48 hex chars) |
| Canonical PMD path (laptop, absolute) | `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` |
| Tailscale interface profile | Private (more permissive firewall defaults than Public) |
| NSSM status | NOT installed; `winget install --id=NSSM.NSSM` (or manual from nssm.cc) before T1b §2.b can run |
| Firewall rule for :11435 / node.exe | none visible via `Get-NetFirewallRule` — Tailscale-Private likely permitting; user chose "investigate before tightening" — no rule change planned for iteration 1 |
| EliteDesk `/srv/brehon-fork/.mcp.json` (pre-T1b) | only `project-memory` (local DB!) + stale `Ref`; missing `junior-brehon` + `tavily` |
| EliteDesk Junior daemon | `/usr/local/bin/junior` (stripped Node.js single-file binary); spawn logic embedded, not script-patchable |
| `.claude/settings.json` (tracked) | already has retro-check.sh Stop hook; session 2 added role-signal-utilisation.sh sibling at lines 43-47 |

## 6. Reconciled design decisions (do NOT re-litigate without user input)

Additions from session 2 to the session 1 table:

| Decision | Choice | Source |
|---|---|---|
| Role mcp.json baseline | Verbatim copy of canonical `.mcp.json` minus secrets (`${ENV_VAR}`) | User answer, session 2 pre-T2 |
| Server lifecycle for HTTP MCP | NSSM service (auto-restart, survives logoff) | User answer, session 2 pre-T1b |
| Firewall rule | Investigate before any change | User answer, session 2 pre-T1b |
| T3 scope | Full T3a+b+c (later collapsed to 3a+lean-3b+c during pivot) | User answer, session 2 pre-T3 |
| Plan §Task 3 — 3 hooks vs 1 | Lean-shape pivot to 1 hook (parallelism + outcome derivable from retros + startup-cost derivable from manifest) | Author decision after WebFetch confirmed Stop hooks fire in parallel; user confirmed the goal is per-role context management |
| DQ-blocker count | YES, include as `dq_blockers_added` field in utilisation row content | User answer, session 2 post-pivot |

## 7. Invariants you MUST preserve

(From session 1, unchanged.)

- Canonical PMD path stays `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db`. Never edit `retro-check.sh` to "fix" stranded-retro symptoms.
- `answered_by: "advisor"` in DQ is ONLY for the persistent advisor session. role-signal rows use `memory_type='summary'`, not DQ entries.
- `phase-branch.md`: `.claude/roles/`, `.claude/hooks/`, `.claude/commands/`, `.claude/skills/`, `MCPs/project-memory-mcp/scripts/` are direct-on-trunk meta-work. `crates/`, `migrations/`, `tests/` are PR flow.
- Pre-compact handover discipline — this file IS the handover for session 2. Author your own before next session boundary.

## 8. Open questions for the next session

1. **NSSM is not installed.** User chose NSSM lifecycle but pre-req hasn't been satisfied. Surface at T1b kickoff: "do you want to `winget install --id=NSSM.NSSM` now, or defer T1b to a later session?"
2. **EliteDesk `.mcp.json` additions** — when repointing for T1b, should we add `tavily` server to Junior workers (they'd gain web-search capability)? Or keep narrow? Probably narrow per "minimal start" framing, but worth asking.
3. **T4a vs T1b — which next?** T4a (`/check-role-health` consumer) makes the signal corpus useful and can ship without T1b (laptop PMD already accessible). T1b makes Junior signals reach the canonical PMD (currently they'd land in lane-local PMDs and be invisible — the same defect class as v1-ship-1 stranding). Strong argument for T1b first: until Junior workers can write to canonical PMD, the hook authored in session 2 is half-functional. Strong argument for T4a first: the consumer is the value-delivery moment. **Recommended: T1b first** (one user-driven afternoon — install NSSM, paste two .mcp.json edits, queue smoke task), then T4a (a few hours of slash-command authoring against a populated PMD).
4. **Signal-rate explosion guard.** Plan estimates ~3 rows/dispatch × ~100/wk = 1200/mo. Lean-shape ships only 1 row/dispatch = ~400/mo. Still want T4b weekly-review prune of rows >90d. Verify `memory_prune` MCP tool handles this; if not, T4b drives it explicitly.

## 9. Anti-patterns observed and avoided during session 2

(Additions to session 1's list.)

- **Don't trust the agent's hooks-doc summary without verifying.** I dispatched the claude-code-guide agent for the latest hooks docs; it surfaced "Stop hooks run in parallel" as a load-bearing claim. I WebFetched the canonical docs page to verify before redesigning. Confirmed — but the agent had also confidently claimed `CLAUDE_SESSION_ID` env var works (it doesn't; session_id comes via stdin JSON only). Always verify load-bearing agent claims against primary sources.
- **Don't redesign at the agent's first suggestion of a problem.** The claude-code-guide agent recommended "consolidate into one Stop hook" immediately. I held the AskUserQuestion until the user clarified the GOAL (per-role context management) — the redesign that emerged then was leaner than the agent's first suggestion (which would have kept all three signal kinds; the goal-driven version collapsed two of them entirely).
- **Don't smoke-test only the happy path.** The first smoke test on `governance-v0` (advisor branch) returned exit 0 silently because of the junior/* gate — looked like the hook "worked" but actually proved nothing. The second smoke on a synthetic `junior/x` temp repo caught the `git rev-parse` Windows STDOUT quirk that the first test masked. Smoke against EVERY gate's both sides.
- **Windows `git rev-parse` quirk on missing refs.** Prints the literal ref to STDOUT (not stderr) with exit 128. `2>/dev/null || fallback` doesn't catch it. Need explicit exit-code check: `_x=$(git rev-parse ... 2>/dev/null) && OUT="$_x" || OUT=""`. Worth a lesson file at retro time.

## 10. Files modified this session

**MCP repo (`barrie-cork/project-memory-mcp`):**
- `src/scripts/write-role-signal.ts` — new (commit `5cc7071`)
- `package.json` — `bin` entry added (same commit)

**Brehon repo (`barrie-cork/lemmy:governance-v0`):**
- `.claude/roles/{planning,impl-task,bm-task,ci-watcher}/{manifest.yaml,rules.allowlist,mcp.json,system-prompt.md,README.md}` — new × 20 (commit `1bb4db189`)
- `.claude/hooks/role-signal-utilisation.sh` — new (commit `f6088a83d`)
- `.claude/settings.json` — added role-signal-utilisation.sh entry to Stop array (same commit)
- `.gitignore` — added `.claude/role-signal-queue.jsonl` (same commit)
- `.claude/PRPs/handovers/role-customization-2026-05-24-session2.md` — this file (will be its own commit)

**Sentinel PMD rows (will auto-prune 2026-05-25):**
- 521 (smoke-t3a-001), 522 (smoke-junior-001), 523 (smoke-junior-002), 524 (smoke-real-001)

## 11. Resume from here

```
TaskList                  # see #11, #12, #13, #14 pending
# Read .claude/PRPs/handovers/role-customization-2026-05-24-session2.md (this file)
# Read .claude/PRPs/handovers/role-customization-2026-05-24.md (session 1)
# Recommended next: T1b kickoff — ask user if NSSM install is ready, then queue T1b §2.b
# Alternative: T4a — author /check-role-health, useful immediately against laptop-side PMD
```
