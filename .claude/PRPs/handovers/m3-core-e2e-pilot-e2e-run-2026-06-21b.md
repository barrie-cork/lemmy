# Handover — m3-core-e2e-pilot e2e run (2026-06-21b, cycle-3 catch-fire)

**Phase:** m3-core-e2e-pilot (M3-core Phase 6/6 — FINAL). Mode B (canonical `brehon-fork` on `governance-v0`; phase work via daemon).
**Stage:** fix-impl #3 SHIPPED; e2e run reached **cycle-3 catch-fire** — auto-fix loop STOPPED per user decision 2026-06-21. Phase stays blocked at e2e.
**Author:** advisor (governance-v0).

> Supersedes `m3-core-e2e-pilot-e2e-run-2026-06-21.md`. That handover's STEP-0 cleanup
> + :8080 decision are DONE; its "fire-and-forget :3000" assumption was DISPROVEN this
> session (see "Empirical findings"). Read THIS file first on resume.

## RESUME BLOCK (read first)

- **Phase branch tip (origin):** `f5a4c0e47 chore(decision-queue): record e2e re-run #763 …` on `origin/phase-m3-core-e2e-pilot`.
- **fix commit:** `cb3e67e90 fix(bridge/e2e): LiveKit explicit --keys devkey:devsecret + MinIO recordings bucket init` — on origin. DoD green (2 compose files).
- **Aux services: BOTH RESTORED.** e2e stack DOWN (0 containers), `docker-portainer-1` running, `web-archive-frontend` running on :8080. **No live cleanup obligations.**
- **Next concrete action:** this is a PLANNED piece of work, not another reactive fix. Stand up the Brehon governance server on :3000 for e2e + investigate LiveKit `update_participant unavailable`. Do NOT queue another fix-impl without a plan — cycle-3 catch-fire is in effect.

## What this session did

1. Resumed e2e-pilot. Discharged the prior handover's STEP-0 (Portainer was down → restarted; e2e stack was up → reused then torn down) + resolved the :8080 conflict (stop `web-archive-frontend`, bridge-a binds :8080, restore after).
2. **Fixed two daemon environment blockers that masked the real test results** (the prior "EXIT=101" runs never actually ran the tests):
   - cargo not on non-interactive SSH PATH → `export PATH=$HOME/.cargo/bin:$PATH`.
   - `services/bridge/target/` was **root-owned** (Docker-built, Jun 13) → `sudo chown -R barrie:barrie services/bridge/target`. **This is durable — won't recur unless a Docker build re-roots it.**
3. First real run surfaced LiveKit-401 (auth) + MinIO NoSuchBucket. Authored + dispatched **fix-impl #3** (Junior #763): LiveKit explicit `--keys "devkey: devsecret"` + MinIO `minio-init` (mc) bucket sidecar. Both verified working.
4. Re-ran → **cycle-3 catch-fire**: new root causes surfaced (below). User chose STOP.

## Empirical findings (CORRECTS the prior handover)

- **stage_mode** → ✅ PASS (`four_mic_pass_then_grace_boundary_emits_chair_entries`). DQ `79d716d1f587-002` resolved.
- **LiveKit `--dev` does NOT set API keys** — verified via `docker run --rm livekit/livekit-server:v1.7 --help`: `--dev` only sets log-level/formatter/pprof. With no `--keys`, LiveKit logs "no keys provided, using placeholder keys {devkey, secret}" — secret is **`secret`**, not `devsecret`. The bridge+tests sign with `devsecret`. Fixed via `--keys "devkey: devsecret"` (env alias `LIVEKIT_KEYS`; format `key: secret` colon-space).
- **MinIO creates no buckets on startup** — fixed via `minio/mc` one-shot (`mc mb --ignore-existing local/recordings`). Verified `recordings` bucket created.
- **:3000 is NOT fire-and-forget** (the prior handover's assumption was WRONG). `recording_lands_with_hash_on_chain` does `POST room-event to Brehon` at `http://localhost:3000/api/v4/governance/room-event` and `?`-propagates the error → hard fail when :3000 is down. `rtc_disabled_townhall_clean_posture` asserts a townhall bridge_room row that the :3000 room-event flow creates.

## Remaining failures (the cycle-4 layer — DO NOT auto-fix)

| Target | Real test | Failure | Root cause |
|---|---|---|---|
| recording | `recording_lands_with_hash_on_chain` | `POST room-event to Brehon: localhost:3000 connection refused` | **Brehon server :3000 not running** |
| room_provisioning | `rtc_disabled_townhall_clean_posture` | no townhall bridge_room row (case 77001) | bridge `create_community_room` depends on **:3000** room-event flow |
| emergency_mute | `mute_all_drops_all_publishers_cross_instance_under_500ms` | `emergency_mute.rs:204`: `update_participant → twirp unavailable: no response from servers` | **LiveKit `update_participant` unavailable** (got past auth; revoke call fails). NOTE: the assert already accepts "participant not found" as zero-holder-by-absence; `unavailable` may belong in the same accepted set since no real WebRTC clients connect in the base stack — OR a LiveKit routing/config issue. Decide deliberately. |

Passing now (was failing pre-fix-#3): `room_provisioning::anonymous_townhall_identity_never_reaches_livekit`. Plus the 5 `room_provisioning` `todo!()` stubs remain D2-deferred (expected-fail).

## Two open questions for the next (planned) session

1. **Stand up Brehon governance server on :3000 for e2e.** This is the Lemmy/Brehon backend (`lemmy_server` with governance routes), not a bridge container — a meaningfully larger setup than a compose line. The pilot server runs on homeserver :1236 (different stack: `brehon-*` containers); :3000 is the bridge's hardcoded default `BREHON_ROOM_EVENT_URL`. Options: (a) point the bridge at the running pilot :1236 governance API (env override `BREHON_ROOM_EVENT_URL`); (b) spin a dedicated lemmy_server for e2e on :3000; (c) re-scope these tests.
2. **LiveKit `update_participant unavailable`.** Investigate whether it's a LiveKit single-node routing config issue, or whether the test's accepted-error set should include `unavailable` (same zero-holder-by-absence logic as "not found", since the base stack has no Element Call clients).

## 3 `-e2e` DQ entries (all `result: fail`, stay pending on phase branch for triage)

`c2e678abdbb2-002` recording · `44243654b24d-002` room_provisioning · `1c37b605fd1a-002` emergency_mute. (`79d716d1f587-002` stage_mode = resolved/pass.) ID→test mapping CORRECTED this session (prior handover had it scrambled) — confirm by reading each entry's `question` field.

## Re-run procedure (for when :3000 is sorted)

On daemon (`ssh homeserver`), native cargo (PATH + chown fixes are durable):
1. Free :8080: `docker stop web-archive-frontend`. Up stack: `cd /srv/brehon-fork/services/bridge && docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc up -d`. (minio-init now creates the bucket; bridge-b may fail :8081 bind — host 127.0.0.1:8081 listener — but bridge-b is D2-deferred cross-instance only, doesn't block in-instance tests.)
2. Stand up / wire :3000 governance API (the open question above).
3. `export PATH=$HOME/.cargo/bin:$PATH && export BRIDGE_DB_PATH=/srv/brehon-fork/services/bridge/.e2e-data/bridge-a/bridge-a.db`
4. Per target: `cargo test --manifest-path services/bridge/Cargo.toml --test <T> -- --ignored > /tmp/m3-e2e-<n>-<T>.log 2>&1; echo EXIT=$?` (NO pipe). Targets: `recording`, `room_provisioning`, `emergency_mute`.
5. Teardown stack + `docker start docker-portainer-1 web-archive-frontend`. End-state: stack DOWN, portainer + web-archive up.

## After all real tests pass

Task 7 (D2 pilot runbook, NON-impl) → retro → `/brehon-verify m3-core-e2e-pilot` → bm-pr → gate-3 CR triage → gate-5 merge-confirm → bm-merge → gate-6 retro sign-off → `/brehon-phase-transition`.

## Resume command

`Resume m3-core-e2e-pilot. fix-impl #3 SHIPPED (cb3e67e90, phase tip f5a4c0e47). e2e at cycle-3 catch-fire — auto-fix STOPPED. Read .claude/PRPs/handovers/m3-core-e2e-pilot-e2e-run-2026-06-21b.md FIRST. NO live cleanup obligations (stack down, portainer + web-archive up). Next = PLANNED work: stand up Brehon :3000 governance API for e2e (or point bridge at pilot :1236 via BREHON_ROOM_EVENT_URL) + investigate LiveKit update_participant 'unavailable'. Do NOT queue another reactive fix-impl.`
