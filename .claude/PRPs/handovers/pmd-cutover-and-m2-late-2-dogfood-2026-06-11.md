# Handover — PMD cutover + Junior hook bug + m2-late-2/test dogfood (REV 2, 2026-06-11 evening)

**Date:** 2026-06-11 (rev 2, ~20:30 local). **Author session:** advisor on laptop, CWD `C:/Users/barri/Developer/brehon-fork`, branch `governance-v0`.
**Read this with zero conversation context.** Supersedes the morning rev of this file. Three threads below; one is newly-discovered (Thread C, a Junior daemon bug now filed as a GH issue).

---

## ⚠️ Session-start gotcha that WILL recur

This canonical checkout is **shared by 2-3 concurrent CC sessions** (seen 2026-06-11: transcripts `df821efb`, `6db85786` writing live). The foreign-WIP guard (`feedback_canonical_checkout_foreign_wip_means_stop.md`) fires every time. At this rev's wrap, the tree carried **3 foreign deletions** (`.claude/PRPs/v1-AD-c-runlog/*`) from a concurrent session — NOT mine, left untouched. Before ANY meta-work: `git status --short` + check `ls -t ~/.claude/projects/C--Users-barri-Developer-brehon-fork/*.jsonl | head` for live transcripts. Commit ONLY your own files via atomic burst (stage-specific → commit -F → show --stat verify).

---

## Thread C — Junior `create_hook` is BROKEN (NEW; GH issue filed) — DO NOT retry blindly

**Status:** root-caused, filed, parked. The Telegram completion hook (✅/❌ on task done/failed) **cannot be created** right now. It is **non-gating** — poll `list_tasks` for completion; never stall on it (`feedback_daemon_telegram_completion_hook.md`).

**GH issue:** https://github.com/barrie-cork/junior-mcp/issues/1
**PMD:** issue-note id 941 (`memory_search_hybrid "junior hook add claude -p MCP init hang"`).

**Root cause (isolated, falsified 4 wrong hypotheses first):** `create_hook` → daemon `junior hook add` → `extractHook()` spawns bare `claude -p ... --model haiku` (NO explicit `--mcp-config`). That bare launch **hangs intermittently (exit 124)** inside any project dir because `claude`'s default MCP auto-init stalls. Proof: `claude -p` in `/tmp` (no `.mcp.json`) → exit 0; in any `/srv/<project>` → exit 124; in `/srv/brehon-fork --strict-mcp-config --mcp-config "{}"` → exit 0. Workers are unaffected (daemon gives them explicit `--mcp-config`). The `"Junior MCP not configured"` warning is **cosmetic** (it's a `detectMcp` check for an `mcpServers.junior` key) — NOT the cause; a hook WAS created via this same path on 2026-05-30.
**Fix is in the junior-mcp binary** (issue suggests `--strict-mcp-config --mcp-config "{}"` for extraction, or a timeout). Nothing to fix on our side. Do NOT re-edit `.mcp.json` or trust state chasing this.

**Config residue left in place (harmless):** daemon `~/.claude.json` `/srv/brehon-fork` set `hasTrustDialogAccepted: true` (was false; six siblings were true). Backup `~/.claude.json.bak-trust-20260611-191824`. Did NOT fix the hang; kept as hygiene only.

---

## Thread A — Finish PMD homeserver cutover (round-trip INCONCLUSIVE; daemon NOT yet retired)

### Verified this session
- homeserver PMD live store baseline: **max_id 941, max_created 2026-06-11 18:48:55, total 820**.
- EliteDesk `/srv/brehon-fork/.mcp.json` project-memory → `http://100.81.145.58:11435/mcp` (type http, Bearer). ✅ repoint confirmed; `PMD_HTTP_TOKEN` in `.env`.
- homeserver `project-memory-http.service` active (PID 1166528).
- Workers DO get `project-memory` + `Ref` MCP (daemon passes `--mcp-config <repo>/.mcp.json`; job-649 worktree confirmed both servers).

### ⚠️ Round-trip NOT cleanly proven
Dispatched job-649 (`[role:smoke]` PMD round-trip). Worker **ran successfully** (result success, "✅ PASS", model opus-4-8, 17 min). BUT:
- It branched off **`phase-m2-late-1`**, NOT my `smoke/pmd-roundtrip-20260611` throwaway — **the daemon ignored the `base_branch` override** (daemon checkout was on phase-m2-late-1). Watch this for any future base_branch override.
- **homeserver PMD max_id stayed at 941** — no new INSERT. The worker's PMD activity ("boost 200 on 649 round-trip") was a read/pheromone UPDATE, not a `memory_write_eval` INSERT that incremented max_id. So we did NOT prove a worker write lands on homeserver.

### THE REMAINING STEP — retire laptop daemon (still gated, now on a CLEAN round-trip)
Laptop NSSM service **`pmd-http-mcp`** (was: node PID 5532 / nssm 4744) still serves the stale laptop DB. **Do NOT retire until a worker write is confirmed on homeserver.** Next session:
1. Dispatch ONE trivial Junior task whose ONLY job is `memory_write_eval` (or a plain `memory_write`), then verify `ssh homeserver 'sqlite3 /srv/brehon-fork/.project-memory/memory.db "SELECT MAX(id),MAX(created_at) FROM memories"'` advances **past 941**. (job-649's success without an INSERT means a read-only smoke is insufficient — make the next one write-and-verify.)
2. Only then: `Get-NetTCPConnection -LocalPort 11435 -State Established` (expect loopback only), `Stop-Service pmd-http-mcp; Set-Service pmd-http-mcp -StartupType Disabled`, verify Stopped/Disabled.
3. Do NOT delete laptop DB (rollback backup). Rollback copies: laptop `.mcp.json.bak-homeserver-20260611`, EliteDesk `.mcp.json.bak-cutover-20260611`.

---

## Thread B — Dogfood `/auto-phase` (user wants `/auto-phase test` next session)

**User's latest direction (2026-06-11 evening):** "begin `/auto-phase test` in the next new session." This supersedes "run m2-late-2 for real" as the dogfood vehicle — `test` is a sandbox sub-phase to exercise the new context-management wiring without committing real m2-late-2 scope. (If `test` is not a real roadmap sub-phase and `/auto-phase test` errors, fall back to m2-late-2 per the original plan — confirm with user.)

### What the dogfood is exercising
The `/auto-phase` + `/compact-phase` context-management wiring applied 2026-06-11 (commit `cd7f9dff0`, Phase D+E validated): schema-v3 auto-state ledger, digest ring, spill guard (>16000-char tool results spill to `.claude/auto-state/<phase>.spill/`), auto-handover refresh, digest-first resume.

### Resume path (fresh session)
1. Foreign-WIP guard (above) — `git status`, check live transcripts.
2. `git fetch origin governance-v0` — verify trunk. Local was `23798109e` (pushed to origin this session). **NOTE: a concurrent session left 3 `.claude/PRPs/v1-AD-c-runlog/*` deletions uncommitted — confirm committed/clean or still foreign before meta-work.**
3. Re-sync daemon-local trunk before any bm-cut: `ssh homeserver 'cd /srv/brehon-fork && git fetch origin governance-v0 && git update-ref refs/heads/governance-v0 origin/governance-v0'` (use `update-ref`, NOT `reset --hard` — `feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md`). **Daemon checkout was on `phase-m2-late-1` this session — check `git -C /srv/brehon-fork branch --show-current` and that no stray task is mid-flight.**
4. `/auto-phase test` → Phase 0 prereqs → bm-cut → planning → `/brehon-clarify` → planning Junior → …
5. Dogfood observation checklist (record in a report): first stage transition creates `stage_digests` entry; >16000-char tool result → spill file; auto-handover refreshes `.claude/PRPs/handovers/test-auto-<date>.md`; compact/resume uses `stage_digests[-1]` / `[-3:]`.

### Gates that WILL fire (user-gated, don't skip)
Plan approval, CR triage, Phase-2 e2e local-vs-dispatch, merge confirm, retro sign-off (advisor-orchestrator §3.2). NO-CARGO-ON-ELITEDESK applies. (For a `test` sandbox sub-phase, several gates may be vacuous — but the wiring should still fire them.)

---

## Thread D — Daemon git-state cleanup BEFORE `/auto-phase test` bm-cut (investigated; partly fixed)

The EliteDesk daemon checkout `/srv/brehon-fork` was found in a messy-but-recoverable state. **Investigated, no work lost, partly reconciled:**

- **HEAD parked on `phase-m2-late-1`** (tip `e53592bb0`), NOT `governance-v0`. This is why job-649 branched off `phase-m2-late-1` instead of the `base_branch` override I passed (daemon honors override inconsistently when checkout is on a non-default branch). **`e53592bb0` is a pure merge commit — both parents (`003a57f4d`, `6044bc78b`) are already in `origin/governance-v0`; PR #192 merged the real content at `be8134d0b`. Zero unique work; safe to abandon.**
- **Daemon trunk ref FIXED this session:** `git update-ref refs/heads/governance-v0 origin/governance-v0` → daemon `governance-v0` now `23798109e` (was stale `bd5270225`). (Used `update-ref`, not `reset --hard`, per `feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md`.) **NOTE: origin advanced again after this to `137fc1aad`** (runlog cleanup) — re-run the update-ref next session.
- **`git checkout governance-v0` ABORTED** — the daemon worktree has **stale Pi-comparator WIP**: 5 modified `scripts/brehon/comparator-*.sh` + untracked `.claude/PRPs/comparator/runs/planning-00{2..8}/`, mtimes **June 7-8** (NOT live — abandoned artifacts from the planning-001..008 comparator runs). Did NOT force checkout/stash (it's the comparator owner's uncommitted work). HEAD stays on phase-m2-late-1 until that WIP is committed/cleaned.
- **`governance-v0-local` (`39e48ac65`)** — harmless 3rd trunk ref, ancestor of origin (old SL-c-1 bm-pr brief commit). Leave it. Daemon has **59 local branches** (ab-cell/ab-test/planning-* cruft) — cleanup candidate, non-urgent.

**Next-session bm-cut prerequisite (do BEFORE `/auto-phase test`):**
1. `ssh homeserver 'cd /srv/brehon-fork && git fetch origin governance-v0 && git update-ref refs/heads/governance-v0 origin/governance-v0'` (re-sync to `137fc1aad`+).
2. Decide what to do with the stale comparator WIP (ask user — commit it under a comparator-artifacts commit, or `git stash` it on the daemon). Only then can the daemon HEAD return to `governance-v0`.
3. If `/auto-phase test` bm-cut branches from `refs/heads/governance-v0` (the ref, not HEAD), the parked HEAD is tolerable — but a clean daemon-on-governance-v0 is safer. Verify `git -C /srv/brehon-fork branch --show-current` = governance-v0 before dispatch.

---

## Repo state at this handover
- Laptop branch `governance-v0`, HEAD **`137fc1aad`** (pushed to origin; laptop=origin in sync). Commits this session: lesson `23798109e` (pushed for concurrent session), handover rev 2 `2bd989568`, runlog cleanup `137fc1aad`.
- The 3 `.claude/PRPs/v1-AD-c-runlog/` deletions were **user-made cleanup** (v1-AD-c shipped PR #81, runlog archived) — committed `137fc1aad`. (Earlier rev of this handover wrongly called them foreign — corrected.)
- Out-of-repo work this session (no commit): `.mcp.json` repoint (done prior session), daemon `~/.claude.json` trust toggle (Thread C residue), GH issue barrie-cork/junior-mcp#1, PMD issue-note id 941, daemon `update-ref` trunk sync (Thread D).
- **Outstanding:** Telegram hook BROKEN (Thread C, filed #1, non-gating); PMD cutover round-trip UNPROVEN — laptop `pmd-http-mcp` NOT retired (Thread A); daemon stale comparator WIP + parked HEAD (Thread D); `/auto-phase test` not started (Thread B — user's next-session goal).
