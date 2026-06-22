# Handover — m3-core-e2e-pilot AS-register (2026-06-21, session boundary)

**Phase:** m3-core-e2e-pilot (M3-core Phase 6/6). Mode B (canonical `brehon-fork` on `governance-v0`; phase work via daemon).
**Stage:** all cycle-4 fixes LANDED + verified; **bundled e2e gate is the only remaining step** before retro → bm-pr → merge.
**Author:** advisor (governance-v0).

## ⚠️ READ FIRST — I may have reaped your in-progress stack

At session wrap I ran a "clean end-state" check, saw the e2e stack up, and tore it
down + restarted `web-archive-frontend`. The stack was **"Up About a minute"** — i.e.
a session (likely yours) had just `docker compose up`'d it to run the bundled gate. It
was only ~1 min in (no test had run yet), so the only cost is re-running the `up`.
**If your stack disappeared, this is why — just bring it back up.** Current daemon
end-state: e2e stack DOWN, `docker-portainer-1` + `web-archive-frontend`/`-api`/`-opensearch` UP.
LESSON for me: lead the end-state check with `docker ps Status` (age) BEFORE deciding to
tear anything down — a freshly-up stack means another session is driving.

## RESUME BLOCK (read first)

- **Phase branch tip (origin == daemon):** `4df0ed991 feat(e2e): merge as-reg-overlay impl-task …` on `origin/phase-m3-core-e2e-pilot`.
- **gov-v0 tip:** `63e5db35e` (briefs + plan committed).
- **All cycle-4 fixes are IN and verified — nothing left to implement.** Next = ONE bundled e2e gate stack cycle (gate-4 = LOCAL, daemon, ~30 min).

## What landed this session (the cycle-4 fixes)

The handover's "2 big root causes" resolved into 3 small fixes, all verified:

1. **recording** — env-only, proven GREEN 3/3 this session. No code. Run env (see below).
2. **emergency_mute** (#764, commit `d1af134af`) — 2-line accepted-error-set extension
   (`unavailable` + `no response from servers`). Verified: LiveKit v1.7 psrpc routes
   `UpdateParticipant` to the node owning the participant session; a never-connected
   publisher (no Element Call client in the base stack) → 3s psrpc timeout → 503
   `unavailable`. This IS zero-holder-by-absence (= accepted `not found`). Proven from
   LiveKit twirp.go/psrpc logs.
3. **room_provisioning** (#766 base `58b23918f` + #767 overlay `8f8b65221`) — per the
   approved plan `m3-core-e2e-pilot-as-register.plan.md`. Root cause: bridge Matrix
   `createRoom` → `401 M_UNKNOWN_TOKEN` because `matrix-conduit:v0.6.0` NEVER registers
   the appservice (mounted `registration.yaml` + `CONDUIT_AS_TOKEN` are inert; Conduit
   registers AS only via its admin room). Fix = swap both compose files' Matrix
   homeservers (base `tuwunel`, overlay `tuwunel`+`tuwunel-b`) from Conduit v0.6.0 to the
   real **Tuwunel** image (`@sha256:1319f9fd…`, already proven live in
   `docker-compose.pilot.yml`), which auto-loads `registration.yaml` from
   `TUWUNEL_APPSERVICE_DIR`. Planner verified createRoom → 200 vs Tuwunel live (§10.3).
   Both edits R-NOSRC clean (no `src/`/`crates/`), BUG-15 distinct domains preserved.

## The bundled e2e gate (next concrete action)

ONE stack cycle on the daemon (`ssh homeserver`, native cargo; PATH + chown durable),
mutating 4 `validate-pending-laptop-e2e` DQs (all `result: None`, on the phase branch):

| DQ id | task | what to run | expect |
|---|---|---|---|
| `be3e6842603f-001` | as-register-task1 | base createRoom :8448 (plan §15.1) | HTTP 200 + room_id |
| `f433ffd8a4a2-001` | as-register-task2 | createRoom :8448 **and** :8449 (plan §15.2) | both HTTP 200 |
| `13951b0fc56a-001` | emergency_mute-fix | `--test emergency_mute -- --ignored` | pass (psrpc `unavailable` now accepted) |
| `c2e678abdbb2-002` | recording (task5) | `--test recording -- --ignored` | 3/3 pass (re-confirm) |

PLUS the acceptance test `rtc_disabled_townhall_clean_posture` (plan §15.3) against the
Tuwunel governance stack — this is the one flow NOT yet seen live in-stack (planner proved
createRoom standalone; the full bridge→Tuwunel→createRoom→`bridge_room` row write needs the
real run).

### Stack-up procedure

1. Free ports: `docker stop web-archive-frontend docker-portainer-1` (:8080, :9000).
2. Up the rtc stack with the NEW Tuwunel images:
   `cd /srv/brehon-fork/services/bridge && docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc up -d`
   (bridge-b may fail :8081 bind — D2-deferred, doesn't block in-instance tests; a
   `127.0.0.1:8081` host listener owns it.)
3. **R-REGTOKEN**: both tuwunel services now boot with `TUWUNEL_REGISTRATION_TOKEN` — if a
   tuwunel container EXITS at boot, that token is missing (it isn't — verified in the edit).
4. For the **recording** test, the bridge + test env must agree on the pilot:
   `export BREHON_ROOM_EVENT_URL=http://100.81.145.58:1236/api/v4/governance/room-event`
   `export BRIDGE_CALLBACK_SECRET=brehon-bridge-callback-secret-pilot-01`  (then
   `docker compose … up -d --force-recreate --no-deps bridge` so the container picks it up)
   `export LIVEKIT_ADMIN_URL=http://localhost:7880` ; `unset LIVEKIT_URL`
   `export BRIDGE_DB_PATH=/srv/brehon-fork/services/bridge/.e2e-data/bridge-a/bridge-a.db`
5. Run each target: `cargo test --manifest-path services/bridge/Cargo.toml --test <T> -- --ignored > /tmp/m3-e2e-<T>.log 2>&1; echo EXIT=$?` (NO pipe). Targets: `recording`, `emergency_mute`, `room_provisioning`.
6. For the createRoom curl checks, see plan §15.1/§15.2 (they prove AS registration independently of the rust tests).
7. Mutate each DQ (`answered_by: advisor-laptop`, `result: pass|fail`, `resolved_at`); pass→resolved, fail→stays pending for §G4.
8. Teardown + restore: `docker compose … down ; docker start docker-portainer-1 web-archive-frontend`. End-state: stack DOWN, portainer + web-archive UP.

## After all gates pass

Task 7 (D2 pilot runbook, NON-impl) → AS-register retro (plan §13 Task 3) → phase retro
→ `/brehon-verify m3-core-e2e-pilot` → bm-pr → gate-3 CR triage → gate-5 merge-confirm →
bm-merge → gate-6 retro sign-off → `/brehon-phase-transition`.

## Retro carry-forwards (harvest at phase retro)

- bridge e2e run env must override 3 vars to reuse pilot :1236 + localhost LiveKit.
- LiveKit v1.7 psrpc-timeout `unavailable` = not-connected (not a bug).
- matrix-conduit v0.6.0 needs admin-room AS registration; mounted registration.yaml is
  inert → switch to real Tuwunel (3rd never-booted-placeholder defect, after sled + CONDUIT_PORT).
- **USER-REQUESTED:** lightweight task-status triage agent (`list_tasks` no-filter dumps
  65K chars; want a Haiku one-screen running/recent/next probe; `brehon-state-status`
  overlaps — decide extend-vs-new). Full note in `workflow_state_m3_core_e2e_pilot.md`.
- end-state check must lead with container age (`docker ps` Status) before teardown — this
  session reaped a freshly-up stack (see top of file).

## Resume command

`Resume m3-core-e2e-pilot. All cycle-4 fixes LANDED + verified (phase tip 4df0ed991): recording (env), emergency_mute #764, as-register #766/#767 (Conduit→Tuwunel). Read .claude/PRPs/handovers/m3-core-e2e-pilot-as-register-handover-2026-06-21.md FIRST — note: a prior session may have reaped your e2e stack (just re-up it). Next = ONE bundled e2e gate stack cycle, mutate 4 DQs (be3e6842603f-001, f433ffd8a4a2-001, 13951b0fc56a-001, c2e678abdbb2-002) + run rtc_disabled_townhall acceptance. gate-4=LOCAL. End-state: stack DOWN, portainer + web-archive UP.`
