# Handover — M1 planning dispatch (2026-06-03)

**Author:** advisor session, canonical checkout `C:\Users\barri\Developer\brehon-fork` on `governance-v0`.
**Reason for handover:** `junior-brehon` MCP server not connected this session → cannot dispatch the planning Junior. A session reload is required (re-reads `.mcp.json`). This handover lets the reloaded session resume with zero conversation context.

**VERIFIED_AT:** `9dd601513` (governance-v0 tip at handover time).

---

## RESUME — next concrete action

1. **Reload the CC session** (this connects `junior-brehon` + `project-memory` MCP from `.mcp.json`).
2. Confirm MCP is live: `mcp__junior-brehon__list_tasks` (status only) returns without error. If still absent, the MCP config or the node stdio command is broken — diagnose `.mcp.json` `junior-brehon` entry before proceeding.
3. **Session-start ritual** (per advisor-orchestrator §1): `pwd && git branch --show-current && git worktree list && git stash list`. Expect: canonical checkout, `governance-v0`, single worktree, no stashes.
4. **Pre-dispatch git pre-flight** (`/precheck`): the planning Junior forks `junior/m1-planning-1` from the **committed HEAD** of `governance-v0` in `/srv/brehon-fork`. The brief MUST be visible at that ref. The brief committed at `46597f523`, clarify edits at `9dd601513` — both on `origin/governance-v0`. Confirm the daemon's `/srv/brehon-fork` has fetched governance-v0 to ≥ `9dd601513` (`ssh homeserver "cd /srv/brehon-fork && git fetch origin governance-v0 && git log origin/governance-v0 -1 --oneline"`).
5. **Telegram completion hook check** (per advisor-orchestrator §1): `mcp__junior-brehon__list_hooks`. If hook ID 1 absent (daemon restart wipes hooks), recreate per `feedback_daemon_telegram_completion_hook.md` (✅/❌ on done/failed).
6. **Cross-lane cap check** (per advisor-orchestrator §4): `mcp__junior-brehon__list_tasks(status="running")` — if ≥2 running, defer dispatch. Expect 0 running.
7. **Dispatch the planning Junior:**
   ```
   mcp__junior-brehon__create_task with:
     description: "[role:planning] M1 plan — see .claude/PRPs/briefs/m1-planning-1.md"
     base_branch: "governance-v0"
   ```
   (Confirm the exact create_task arg names against the live MCP schema — likely `description` + a base/branch arg. The dispatch string is the single line above, under 100 chars.)
8. **After dispatch:** set a `ScheduleWakeup` at ~1200s (planning runs 30-90 min). Poll `list_tasks`; on `done`, read the plan output.
9. **On plan ship (`.claude/PRPs/plans/m1.plan.md` on governance-v0):** run the plan-approval pre-gate sequence:
   - **DoD smoke test** (§3.4): run EVERY §15 validation command literally against current HEAD; capture exit codes. NOTE: the bridge tree (`services/bridge/`) checks run inside that dir, NOT `cargo --workspace` — per clarify DQ `a3d0e9941441-045`.
   - **Watchpoint specificity gate** (§3.5): every §4 watchpoint must cite a specific file/table/line.
   - **MiniMax trial designation** (§3.5a): walk each `[role:impl-task]` task against the 5 criteria; record rows; report cumulative count.
   - **Linux-compile gate awareness**: M1 adds a migration + greenfield bridge → the diff WILL touch Cargo.toml/migrations → the diff-scoped Linux-compile gate (`feedback_linux_compile_proof_is_a_gate.md`) applies before bm-pr. Note for later.
10. **Surface the plan to the user → gate 1 (plan approval)** via AskUserQuestion. WAIT.
11. **Only after approval:** `/auto-phase M1` becomes valid — OR continue the manual pipeline (queue `bm-cut`). The user chose the standard pipeline (brief → clarify → planning Junior); after plan approval, `/auto-phase M1` will drive bm-cut → impl → … → merge → retro, auto-resuming from the auto-state file it creates.

---

## STATE — what's done (all on origin/governance-v0)

| Item | Commit | Notes |
|---|---|---|
| OQ-V2-08 resolved (a Brehon↔Brehon only) + M1-scope decision (chat-infra only, backplane→M2+) | `b36dcfa0f` | In `99-decisions-and-open-questions.md` 2026-06-03 entry + PRD OQ table |
| M1 sub-PRD authored | `5c96f30c5` | `.claude/PRPs/prds/m1-chat-infrastructure.prd.md` — OQ-V2-09 soft-pause RESOLVED §6 |
| M1 planning brief authored | `46597f523` | `.claude/PRPs/briefs/m1-planning-1.md` |
| `/brehon-clarify` complete (4 DQ, 0 pending) | `9dd601513` | DQ `a3d0e9941441-045..048`; brief §4.2 + §6 updated |

**DQ state:** pending = 0. Four resolved clarify entries `a3d0e9941441-045` (user — bridge validation = laptop runs all), `-046` (advisor — config typed columns), `-047` (advisor — planner pins matrix-sdk version), `-048` (advisor — no cross-phase overlap).

---

## KEY DECISIONS (so the planner brief is read in the right frame)

- **M1 = chat infrastructure ONLY.** NO ADR-016 backplane (no B-fetch / B-publish / B-actor). Bridge daemon + Tuwunel + 1:1 DM + rich media + admin config panel + clean-disable posture. Backplane is M2+.
- **OQ-V2-08 = Brehon↔Brehon only** (fork-only, ADR-014). No vanilla-Lemmy degraded-mode interop.
- **OQ-V2-09 = soft pause** (reversible relay-pause; homeserver/rooms/media/registration persist; only Brehon→Matrix relay halts). Full 4-sub-question spec in sub-PRD §6.
- **Bridge location:** `services/bridge/` in-repo, **workspace-EXCLUDED** (own Cargo.toml; Brehon binary gains zero Matrix deps).
- **Person AP field:** reuse legacy `matrix_user_id` (`person.rs:49`) UNCHANGED; no new actor-extension on the wire in M1.
- **Homeserver:** Tuwunel (Synapse fallback) per OQ-V2-10.
- **Bridge validation:** laptop runs ALL of it (cargo check inside `services/bridge/` + docker-compose integration tests on laptop) — clarify DQ `-045`.

---

## DEFERRED PMD WRITE (backfill on reload — MCP was down)

The 2026-06-03 session retro's System-2 `memory_write_eval` could NOT be written
(project-memory MCP client not connected; HTTP daemon at localhost:11435 IS up).
Durable System-1 capture is at `.claude/PRPs/reports/session-retro-2026-06-03-m1-planning-pipeline.md`.
**On reload, after MCP connects, backfill:**
```
memory_write_eval:
  memory_type: "qa-result"
  title: "Task retro: M1 planning pipeline (pre-/auto-phase) 2026-06-03"
  score: 0.62  (goal partial / tests none / clean yes)
  tags: include "brehon-fork", outcome "partial"
  body: SCORE 0.62 / CONFIDENCE 0.70 / Goal achieved: partial (/auto-phase M1 needs a
        plan first; pre-pipeline done to dispatch boundary; dispatch blocked on
        junior-brehon MCP) / Tests: none / Clean: yes /
        ROOT_CAUSE: ENVIRONMENTAL — MCP-client-not-loaded (reload fixes).
```
(Do NOT forge created_at or use raw SQL — the retro report stands as the record; the
backfill is the canonical System-2 entry once the client is live.)

## CROSS-SESSION DEPS / GOTCHAS

- **MCP not connected** is the ONLY blocker. EliteDesk daemon IS active (verified via SSH `systemctl is-active junior@brehon-fork` = active).
- **`/auto-phase M1` still won't drive impl until the plan ships + is approved.** The original invocation was correct nomenclature (M1 = the ADR-016 rename of V2a) but premature — there was no plan. Don't re-refuse it as "wrong name"; it's the right name, just needs the plan first.
- **No phase branch yet.** bm-cut runs only AFTER plan approval. Planning brief + plan file both live on `governance-v0` (planner finalize-merges the plan to trunk).
- **19 Dependabot vulnerabilities** warned on every push to origin (2 critical) — pre-existing, unrelated to M1; the Deps lane owns them.
- **SSH from the Bash tool** needs `-o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=accept-new` (sandbox blocks the default known_hosts). Daemon host: `homeserver` (Tailscale `100.81.145.58`).
