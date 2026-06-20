---
phase: m3-core-e2e-pilot
scope: Cohort A landed; Tasks 3+5 -linux gates pending (mid-phase handover)
lane_mode: B
authored: 2026-06-20
authored_by: advisor (canonical brehon-fork / governance-v0 session)
last_handover_path: .claude/PRPs/handovers/m3-core-e2e-pilot-cohortA-handover-2026-06-20.md
purpose: Resume the m3-core-e2e-pilot advisor session. Cohort A (Tasks 3+5) is landed but their Linux compile gates have not yet been run. Read this first.
---

# ⏩ RESUME — read first

**You are the advisor for Brehon m3-core-e2e-pilot** (M3 town halls, **Phase 6 of 6 — FINAL**). CWD `C:/Users/barri/Developer/brehon-fork`, branch `governance-v0` (Mode B — drive from canonical via Junior dispatch). No homeserver advisor session.

## State at handoff (literal)

- **Phase branch `phase-m3-core-e2e-pilot` tip: `69f050e2a`** (origin == daemon-local, SYNC). gov-v0 = `e4bea4b67` (unrelated CI commit `38e0c1845` + my Cohort-A briefs commit; phase work never touched gov-v0).
- **governance-v0** has all session meta-commits (briefs, DQ). No phase code on gov-v0 (correct).
- **DQ pending (6 on phase branch):** Task-3 `-linux` + Task-3 `-e2e`, Task-5 `-linux` (`c2e678abdbb2-001`) + Task-5 `-e2e` (`c2e678abdbb2-002`), + 2 older inherited entries.

## What's DONE (Tasks 0,1,2,3,5)

- **Task 0** (advisor-inline, no commit) — freeze baselines confirmed (ENTRY_KIND 72, etc); bridge baseline + 4 integration scaffolds compile clean on Linux.
- **Task 1** (#748 + fix #749 rocksdb + fix #750 CONDUIT_PORT) — e2e harness GREEN. `docker-compose.e2e.yml` (MinIO + tuwunel-b/bridge-b distinct-domain matrix-b.localhost — BUG-15) + `e2e-harness-smoke.sh` (reach-the-containers, MinIO readiness poll) + `registration-b.yaml`. **reach-smoke = E2E_HARNESS_REACH_OK** (DQ `2c511098c5cc-001` PASS). Two latent base-compose defects fixed: Conduit needs `rocksdb` not `sled`; Conduit listens on 6167 → set `CONDUIT_PORT: "8448"`.
- **Task 2** (#751, landed `5abf83328`) — recording live LiveSink (native async-fn-in-trait + generic `maybe_record<S>`, NO async-trait) + cr-2 (403 empty pseudonym) + cr-3 (`bridge_room::participants_for_recording`) + live Egress POST + rust-s3 put_object (config-sourced media_url). **Linux compile gate PASS** (DQ `20110030b2fd-001`). Advisor removed a stray `dq-frag-task2.json` the worker left.
- **Task 3** (#752, landed merge `d8c76625f`) — `tests/stage_mode.rs` live (+168): 4-user mic-pass + 30s grace + chair override/transfer; asserts `governance_log` chair rows with pseudonym fields.
- **Task 5** (#753, landed `69f050e2a`) — `tests/recording.rs` live (+377): recording-lands-on-chain + clean-posture-off + participant-floor.

## ⚠ COHORT-MERGE HAZARD (what happened with Task 5 — read before any future cohort)

Task 5's worker (#753) forked from `0248ac9ab` (BEFORE Task 3 merged). Its worker-branch diff vs the Task-3 phase tip showed `stage_mode.rs -168` — a **full merge/cherry-pick would have REVERTED Task 3's work**. Fix applied: a **PATH-SCOPED take** — `git checkout <task5-worker-branch> -- services/bridge/tests/recording.rs` + appended only Task-5's 2 DQ entries, committed `69f050e2a`. Both files now intact (stage_mode +168 AND recording +377 both present).

**Rule for the remaining cohort merges:** when a cohort member's worker forked from a base older than the current phase tip, do NOT branch-merge its worker branch. Take ONLY its owned file(s) path-scoped + its DQ entries. Verify with `git diff --stat <phase-tip> <worker-branch>` — if it shows a sibling's file changing, that's a stale-base revert risk.

## NEXT CONCRETE ACTION

1. **Run Tasks 3+5 `-linux` compile gates** (gate-4 decided = LOCAL; bridge is Linux-only):
   ```bash
   git worktree add C:/Users/barri/Developer/brehon-fork-validate-cohortA origin/phase-m3-core-e2e-pilot
   cd C:/Users/barri/Developer/brehon-fork-validate-cohortA
   git submodule update --init --recursive        # lemmy_email build.rs needs translations
   cp C:/Users/barri/Developer/brehon-fork/.env .env
   scripts/brehon/cargo-linux.sh check  --manifest-path services/bridge/Cargo.toml
   scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings
   scripts/brehon/cargo-linux.sh test   --manifest-path services/bridge/Cargo.toml --test stage_mode --no-run
   scripts/brehon/cargo-linux.sh test   --manifest-path services/bridge/Cargo.toml --test recording  --no-run
   ```
   Mutate Task-3's `-linux` DQ + Task-5's `-linux` DQ (`c2e678abdbb2-001`) each `result: pass|fail` (`answered_by: advisor-laptop`) on the daemon's phase branch (check daemon current branch first — it's been on `phase-m3-core-e2e-pilot`). **LEAVE both `-e2e` DQs pending** (the live `-e2e` batch runs LAST). Remove the worktree after.
   - On `-linux` fail → §G4 (allowlist→fix-impl; else catch-fire; cycle-count meta-rule: 3rd same `(error_class, file)` = hard catch-fire).
2. **On both `-linux` pass → dispatch Cohort B + Task 4** (cross-lane cap = 2):
   - **Task 6** (room_provisioning) — mirror `m3-core-e2e-pilot-impl-3.md`/`impl-5.md` shape. Criteria 146/142 = **TWO NEW fns** (`rtc_disabled_townhall_clean_posture` + `anonymous_townhall_identity_never_reaches_livekit`) APPENDED after `messaging_disabled_prevents_provisioning` — NOT `todo!()` anchors (these are new fns, not scaffold conversions). R7 negative invariant for 146 (FAIL-if-gate-forced-on); cite `livekit_jwt::mint_pseudonym_claims` for 142. Plan §13 Task 6 (~522-543), §16a Story 6.
   - **Task 4 SOLO** (marquee `emergency_mute.rs`) — cross-instance `<500ms` measured at the **PUBLISHER CLIENT** (R-PUBCLIENT catch-fire if server-side). Per **DQ `3004b6625b83-001` = option-B** (user-ratified): automated IN-INSTANCE publisher-client `<500ms` (add livekit-client dev-dep) + the TRUE cross-instance `<500ms` routed to the D2 pilot per the PRD fallback. **Runs ALONE** (the dev-dep regenerates Cargo.lock → contention). Plan §13 Task 4 (~473-495), §16a Story 2 (the marquee).
3. **After Tasks 3/4/5/6 all land + `-linux`-green → run the live `-e2e` batch** on the laptop stack (gate-4=LOCAL): bring the harness up (`bash scripts/brehon/e2e-harness-smoke.sh` for warm-up, then `cargo-linux.sh test --test <name> -- --ignored` per target). Tear down `docker compose ... down -v` after. Mutate each `-e2e` DQ pass/fail. The marquee may fire the PRD fallback (cross-instance to pilot) — that's DQ-documented, not a fail.
4. **Then Task 7** (D2 pilot runbook — NON-impl, no cargo) + **Task 8** (retro) → `/brehon-verify` → bm-pr → CR/triage → bm-merge → `/brehon-phase-transition`.

## Phase-specific tripwires (catch-fire)

- **emergency-mute measured SERVER-SIDE** instead of publisher-client → catch-fire (false-green, R-PUBCLIENT, plan §16a Story 2 mechanical check).
- **New `ENTRY_KIND_ROOM_*` const or any `crates/db_schema/migrations/**`** → catch-fire (registry FROZEN at 72).
- **A §13 task implementing the pilot as a cargo impl-task** → catch-fire (Task 7 = operational, NON-impl, DoD = retro-recorded outcome).
- **Bridge cargo in Windows form** → DoD-issue (Linux-only via cargo-linux.sh; `ruma-common` E0119).
- **R-CHAIN:** recording `content_sha256` must ride `append_room_event` (the chain row + signature), never a MinIO-metadata-only check.

## Operational reminders (Mode B)

- **Daemon finalize-merge consistently merges locally but does NOT push** — after every Junior task, check `ssh homeserver "cd /srv/brehon-fork && git rev-parse phase-m3-core-e2e-pilot"` vs origin; push if ahead. Sometimes it doesn't merge at all (Task 5) → merge the worker branch manually, path-scoped per the hazard note above.
- **Daemon current branch:** check `ssh homeserver "cd /srv/brehon-fork && git branch --show-current"` BEFORE any Mode-B brief sync (it's been on the phase branch, not gov-v0; a `git worktree add phase-...` will FAIL if so — commit the brief in-place or use a uniquely-named temp branch).
- Workers may leave stray `dq-frag-*.json` under `.claude/PRPs/debug/` — the briefs now say "delete after appending"; verify + remove any new ones at finalize. (Old inherited frags from prior phases are separate debt.)
- DQ attribution: advisor commits `^(chore|docs)\((advisor|decision-queue)\)`. `--repo barrie-cork/lemmy` on all gh pr; PRs base `governance-v0` never `main`.
- NO CARGO ON ELITEDESK — workers write validate-pending-laptop[-linux][-e2e] DQs + STOP; the laptop runs ALL cargo/e2e.
- The 6 user gates: plan-approval (DONE) / ADR-DQ / CR-triage / **gate-4 e2e local-vs-dispatch (DONE = LOCAL)** / merge-confirm / retro-sign-off.

## Source-of-truth files

- `.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` — the plan (§13 tasks, §15 DoD, §16a stories, §7 R-guardrails).
- `.claude/PRPs/briefs/m3-core-e2e-pilot-impl-{3,5}.md` — Cohort A briefs (mirror for Task 6); `impl-2.md` for the src pattern.
- `workflow_state_m3_core_e2e_pilot.md` (PMD System-1) — running scratchpad (every decision + finalize anomaly recorded).
- `.claude/PRPs/handovers/m3-core-e2e-pilot-bootstrap.md` — the stable phase advice.
- DQ `3004b6625b83-001` (resolved option-B) — the marquee measurement contract for Task 4.

## Lessons to harvest at retro (Task 8)

1. `matrix-conduit:v0.6.0` needs `CONDUIT_DATABASE_BACKEND=rocksdb` (sled not compiled in) — and Conduit's default listen port is 6167, so set `CONDUIT_PORT` to match compose mappings. A reach-smoke that BOOTS services (not just pulls images) surfaces never-booted-placeholder bugs.
2. Cohort members forking from the same base: when one lands first, the other's worker branch is stale w.r.t. the first's file → do a PATH-SCOPED take of only its owned file(s), never a branch-merge.
3. Native async-fn-in-trait + generic `<S>` cleanly replaces a sync trait with zero new deps (RecordingSink).
4. Workers leave stray dq-frag temp artifacts — gitignore `.claude/PRPs/debug/dq-frag-*.json` or enforce "delete after append" in briefs.
5. Daemon finalize-merge: merges-local-no-push (every task) AND sometimes doesn't merge at all (Task 5) — advisor post-task look-order check is load-bearing.
