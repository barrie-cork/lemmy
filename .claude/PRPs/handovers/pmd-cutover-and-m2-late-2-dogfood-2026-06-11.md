# Handover — PMD homeserver cutover (finish) + m2-late-2 dogfood (start)

**Date:** 2026-06-11. **Author session:** advisor on P50 (laptop), CWD `C:/Users/barri/Developer/brehon-fork`, branch `governance-v0`.
**Read this with zero conversation context.** Two independent remaining threads below.

---

## Thread A — Finish the PMD homeserver cutover (1 step left)

### Background
The canonical Brehon PMD is being migrated from the **laptop** to **homeserver**. The IP `100.104.171.26` is the **LAPTOP** (`desktop-jtgr71s`); homeserver is `100.81.145.58`. A stale `pmd-invariants.md` value (now fixed, commit `6deae5665`) caused `.mcp.json` to point at the laptop daemon last session, splitting 4 rows.

### Done this session (all verified)
1. **Migrated 4 divergent rows (ids 938–941)** laptop → homeserver via guarded idempotent INSERTs. homeserver store is now the **superset: 820 memories, max_id 941, 0 unembedded** (backfill timer embedded them).
2. **Laptop `.mcp.json`** project-memory → `http://100.81.145.58:11435/mcp` (homeserver), Bearer auth preserved. End-to-end MCP handshake verified. Takes effect on NEXT session restart.
3. **EliteDesk `/srv/brehon-fork/.mcp.json`** (Junior workers) project-memory → `http://100.81.145.58:11435/mcp`. Backup at `/srv/brehon-fork/.mcp.json.bak-cutover-20260611`. Verified homeserver reachable from homeserver via both Tailscale IP and localhost.
4. **Docs fixed:** `pmd-invariants.md` #1 (committed `6deae5665`), `project_pmd_homeserver_http_topology.md` memory, MEMORY.md index line — all now show `100.81.145.58` + the IP-trap warning.

### THE ONE REMAINING STEP — retire the laptop daemon
The laptop runs an NSSM service **`pmd-http-mcp`** (StartType Automatic, node PID 5532 / nssm PID 4744) serving the now-stale laptop DB `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db`. It must be **stopped + disabled** so no stray write splits the store again. **Deferred to a fresh session** because (a) stopping an auto-start service is the riskiest step, (b) it should be gated on a real Junior round-trip proving workers reach homeserver.

**Safe order to finish (fresh session):**
1. Confirm no client is mid-write: `Get-NetTCPConnection -LocalPort 11435 -State Established` (expect only loopback/self).
2. Dispatch ONE trivial Junior task (or wait for the m2-late-2 bm-cut below) and confirm its `memory_write_eval` retro lands on **homeserver** (`ssh homeserver 'sqlite3 /srv/brehon-fork/.project-memory/memory.db "SELECT MAX(id), MAX(created_at) FROM memories"'` advances). This proves the EliteDesk repoint works before retiring the laptop daemon.
3. Then retire: `Stop-Service pmd-http-mcp; Set-Service pmd-http-mcp -StartupType Disabled` (PowerShell, may need elevation). Verify: `Get-Service pmd-http-mcp` → Stopped/Disabled.
4. The laptop DB is now a frozen backup — do NOT delete (rollback safety). The laptop `.mcp.json.bak-homeserver-20260611` and EliteDesk `.mcp.json.bak-cutover-20260611` are rollback copies.

**Verification the cutover is fully complete:** both `.mcp.json` files → `100.81.145.58`; laptop `pmd-http-mcp` Stopped+Disabled; a fresh `memory_write` round-trips to homeserver only.

### Watch
- `pmd-invariants.md` still has older sub-sections + many handover/brief files citing `100.104.171.26` as historical record — NOT swept this session (they're point-in-time). Only the live rule #1 was corrected. Don't mass-rewrite history; fix on-touch.

---

## Thread B — m2-late-2 dogfood (NOT yet started)

### Why
User wants to **dogfood the /auto-phase context-management wiring** (schema-v3 ledger, digest ring, spill guard, auto-handover, digest-first resume — applied to `~/.claude/commands/{auto-phase,compact-phase}.md` last session, commit `cd7f9dff0`, Phase D + Phase E validated). User chose to dogfood by running **m2-late-2 for real** (the genuine next sub-phase) rather than a sandbox.

### Pre-flight done this session
- Trunk pushed: origin `governance-v0` = `bd5270225` (was 4 behind). **NOTE: HEAD has since advanced** — concurrent session committed `f2092a06d` then this session committed `6deae5665`. Re-fetch + re-verify trunk before bm-cut.
- Daemon-local trunk synced once (`be8134d0b` → `bd5270225`); **re-sync before bm-cut** since HEAD moved again: `ssh homeserver 'cd /srv/brehon-fork && git fetch origin governance-v0 && git update-ref refs/heads/governance-v0 origin/governance-v0'`.
- No `phase-m2-late-2` branch exists yet (clean to cut).
- **m2-late-2 has NO plan authored yet.** Scope (per workflow_state_m2_late_2 + bootstrap): bridge power-level enforcement + CR-A atomicity fix + pilot verification.

### Resume path (fresh session)
1. `git -C C:/Users/barri/Developer/brehon-fork worktree list` + `git status` (canonical-checkout foreign-WIP guard — there WAS a concurrent session this session, 2 lesson files left dirty: `feedback_canonical_checkout_foreign_wip_means_stop.md`, `feedback_falsifiable_hypothesis_before_structural_fix.md` — confirm they're committed/clean or still foreign before meta-work).
2. Read `.claude/PRPs/handovers/m2-late-2-bootstrap.md` (full handoff letter) + `workflow_state_m2_late_2.md`.
3. `/auto-phase m2-late-2` → Phase 0 prereqs → bm-cut → planning brief → `/brehon-clarify` → queue planning Junior → … This exercises the new wiring live (Phase 1 standing rules fire on each stage transition; the auto-state ledger at `.claude/auto-state/m2-late-2.json` gets created fresh with schema_version 3).
4. **Dogfood observation checklist** (Phase F of the auto-phase validation plan — record in a report): on first stage transition, `stage_digests` gets an entry; on any >16000-char tool result, a spill file appears under `.claude/auto-state/m2-late-2.spill/`; auto-handover refreshes `.claude/PRPs/handovers/m2-late-2-auto-<date>.md` + ledger pointers; on any compact/resume, `/compact` uses `stage_digests[-1]` and Phase 0.5 Step E uses `stage_digests[-3:]`.

### Gates that WILL fire (don't skip — user-gated)
Plan approval, CR triage, Phase-2 e2e local-vs-dispatch, merge confirm, retro sign-off (per advisor-orchestrator §3.2). NO-CARGO-ON-ELITEDESK hard rule applies (workers write `validate-pending-laptop` + stop; laptop runs cargo/e2e).

---

## Repo state at handover
- Branch `governance-v0`, HEAD `6deae5665` (my PMD-topology doc fix).
- Working tree: 2 foreign lesson files modified (concurrent session's WIP — NOT mine, left untouched).
- My committed work this session: `cd7f9dff0` (auto-phase wiring, prior session) + `6deae5665` (PMD topology doc fix). PMD-migration was DB-side (homeserver), no repo commit needed.
- `.mcp.json` (laptop, gitignored) repointed to homeserver — effective next restart.
