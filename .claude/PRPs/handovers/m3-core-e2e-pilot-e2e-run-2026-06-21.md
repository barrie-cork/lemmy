# Handover — m3-core-e2e-pilot e2e run (2026-06-21, session wrap)

**Phase:** m3-core-e2e-pilot (M3-core Phase 6/6 — FINAL). Mode B (canonical `brehon-fork` on `governance-v0`; phase work via daemon).
**Stage:** fix-impl #2 SHIPPED; e2e run BLOCKED on host-port conflicts (awaiting user decision).

## RESUME BLOCK (read first)

- **Phase branch tip (origin):** `e4f649812 chore(merge): finalize m3-core-e2e-pilot fix-impl-e2e-harness-2 (job-762)` — VERIFIED on `origin/phase-m3-core-e2e-pilot` at 2026-06-21 ~13:27.
- **Fix commit:** `f5cbd2ca9 fix(bridge/e2e): rust-s3 builder API + LiveKit --bind 0.0.0.0 + DB host bind-mount` — on origin. DoD all GREEN.
- **Next concrete action:** decide the host :8080 conflict (see OPEN DECISION below), then run the 3 e2e targets advisor-side, mutate 4 `-e2e` DQ entries, then Task 7.

## ⚠️ TWO LIVE CLEANUP OBLIGATIONS ON THE DAEMON (homeserver)

1. **Portainer is STOPPED.** `docker-portainer-1` = `Exited` (we stopped it to free host :9000 for MinIO, with user approval). **MUST restart when e2e done:** `ssh homeserver "docker start docker-portainer-1"`. It's portainer-ce:2.39.0 from `/srv/docker` compose, restart=unless-stopped, persistent `/data` volume — safe to restart, config intact.
2. **e2e compose stack is UP** on the daemon (bridge-a, minio, livekit, tuwunel, tuwunel-b, element-call, lk-jwt all `Up`; bridge-b `Created`/not-up). **MUST teardown when done:** `ssh homeserver "cd /srv/brehon-fork/services/bridge && docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc down"`.

## What shipped this session

- **Junior #762** (fix-impl #2) — DONE/succeeded 13:24. Three fixes, 4 files, commit `f5cbd2ca9`:
  - recording.rs: `Bucket::new(...).map(|b| b.with_path_style())` builder chain (×2) — `new_with_path_style` does NOT exist in rust-s3 0.34 (cycle-1 brief guessed wrong method name).
  - docker-compose.yml: livekit `command: ["--dev","--bind","0.0.0.0"]` — v1.7 `--dev` binds container-loopback only.
  - docker-compose.e2e.yml: bridge_a_data/bridge_b_data named volumes → host bind-mounts `./.e2e-data/bridge-{a,b}` (named-volume mountpoints root-owned; host-native test couldn't read DB).
  - `.e2e-data/` added to services/bridge/.gitignore.
- DQ log `8aca794fb044-007` (impl-self-resolved) on phase branch.
- Daemon finalize-merge `e4f649812` pushed to origin by advisor (daemon was ahead — broken-pipe network blip earlier delayed the daemon's own push).
- Host prep DONE on daemon: `services/bridge/.e2e-data/bridge-{a,b}` created + chmod 0777.

## OPEN DECISION (was mid-AskUserQuestion when session wrapped)

**Host :8080 conflict.** bridge-a needs host :8080 (tests default `BRIDGE_URL=localhost:8080`), but **`web-archive-frontend`** (web-archive-web image, `/srv/web-archive` compose, restart=unless-stopped, up 58 min, the web-archive web UI) owns :8080. Bridge-a is currently `Up` but published NO host ports (`ports=map[]`) because its compose `up` aborted mid-network-setup — it needs a clean `up` recreate to bind :8080.

Recommended (mirrors the user's Portainer decision): `docker stop web-archive-frontend` → recreate stack with `up` so bridge-a binds :8080 → run targets → teardown → restart web-archive-frontend AND portainer. Alternatives: remap bridge-a `8080:8080`→`8090:8080` (compose edit, worker cycle) + run with `BRIDGE_URL=localhost:8090`; or user frees :8080 manually. **User interrupted before answering — re-surface this on resume.**

Note: bridge-b :8081 is now FREE (earlier 8081/Caddy conflict cleared); bridge-b may even come up on recreate.

## E2E run procedure (once :8080 resolved)

On daemon (`ssh homeserver`), all native cargo (bridge = separate workspace, Linux-native cargo authorized):
1. Ensure stack up (recreate via `up` after freeing :8080). Verify: bridge-a :8080 = HTTP 200 (NOT web-archive — confirm `docker ps --filter publish=8080` shows `bridge-bridge-1`), livekit :7880 = 200, minio :9000 = 200.
2. `export BRIDGE_DB_PATH=/srv/brehon-fork/services/bridge/.e2e-data/bridge-a/bridge-a.db`
3. `cd /srv/brehon-fork`; for each target run `cargo test --manifest-path services/bridge/Cargo.toml --test <T> -- --ignored > /tmp/m3-e2e-762-<T>.log 2>&1; echo EXIT=$?` (NO pipe-to-tail — cargo-output-capture rule). Targets: `recording`, `room_provisioning`, `emergency_mute`. (stage_mode already PASS — DQ `79d716d1f587-002`.)
4. Teardown + restart aux (the two obligations above).

## Test facts established this session (CORRECTS the prior latent audit)

- **Brehon server on :3000 is NOT running.** Both recording.rs (`BREHON_ROOM_EVENT_URL` default `localhost:3000/api/v4/governance/room-event`) and emergency_mute.rs declare it, BUT neither asserts on a :3000 response in its own body — bridge's Brehon callbacks are fire-and-forget. So :3000 is a "note if missing" precondition, likely NOT a hard blocker. VERIFY empirically — do not assume.
- `recording_lands_with_hash_on_chain`: uses bridge :8080 GET (non-participant 403 / participant 200 + media_url) + SQLite (`bridge_db_path`) + MinIO. No direct :3000 assert. Needs bind-mount fix (applied ✓).
- `emergency_mute` marquee `mute_all_drops_all_publishers_cross_instance_under_500ms`: step 1 POSTs `{bridge_url}/brehon/room-event` (bridge :8080) and ASSERTS `is_success()` (town_hall provisioning must succeed via bridge-a). Then mints LiveKit tokens + fires mute-all via `RoomClient::update_participant` (:7880), measured at publisher-client handle (R-PUBCLIENT). pub-a (instance-A) = in-instance <500ms path; pub-b (instance-B) = cross-instance, D2-deferred. Disconnected publishers → "not found" = zero-holder-by-absence (expected in base stack, no Element Call clients).
- room_provisioning: 5 of 7 are `todo!()` stubs (D2-deferred, expected-fail). 2 real: `rtc_disabled_townhall_clean_posture` (SQLite read), `anonymous_townhall_identity_never_reaches_livekit` (local JWT + LiveKit liveness).

## 4 `-e2e` DQ entries to mutate per results (on phase branch)

`79d716d1f587-002` stage_mode→pass (already confirmed) · `1c37b605fd1a-002` recording · `c2e678abdbb2-002` room_provisioning · `44243654b24d-002` emergency_mute.

## CATCH-FIRE guard (cycle-count meta-rule)

Cycles so far on e2e harness were DIFFERENT error classes each (cycle 1: version+DNS+wrong-method-name; cycle 2: builder-API + LiveKit-bind + DB-perms) = progress, not thrash. **If any REAL (non-todo!) test still fails after this run → that's cycle 3 on the e2e harness → CATCH-FIRE per `.claude/rules/advisor-orchestrator.md` §5.3. Surface to user, do NOT auto-fix.**

## After all real tests pass

Task 7 (D2 pilot runbook — NON-impl, no cargo, DoD = retro-recorded) → retro → `/brehon-verify m3-core-e2e-pilot` → bm-pr → gate-3 CR triage → gate-5 merge-confirm → bm-merge → gate-6 retro sign-off → `/brehon-phase-transition`.

## Resume command

`/auto-phase M3 — resume m3-core-e2e-pilot e2e run. #762 fix-impl #2 SHIPPED (f5cbd2ca9 on origin/phase-m3-core-e2e-pilot @ e4f649812). Read .claude/PRPs/handovers/m3-core-e2e-pilot-e2e-run-2026-06-21.md FIRST. Resolve the OPEN :8080 conflict (web-archive-frontend owns it; mirror the Portainer decision), then run e2e advisor-side per the handover procedure. RESTART docker-portainer-1 (stopped) + teardown e2e stack when done.`
