# Brief: m3-core-e2e-pilot fix-impl #3 — LiveKit explicit --keys + MinIO recordings bucket

## §1 Role + dispatch line

`[role:impl-task] m3-core-e2e-pilot-fix-impl-e2e-harness-3 — see .claude/PRPs/briefs/m3-core-e2e-pilot-fix-impl-e2e-harness-3.md`

You are the **impl-task** subagent (Sonnet 4.6). This is a **fix-impl task** — two compose-config fixes uncovered by the advisor's e2e run #762-rerun (the tests RAN this time; both failures are env-config, not code/test-logic). After the edits, commit + push, write a `kind: "log"` DQ. **Do NOT run the e2e suite** (the advisor re-runs it — the run needs host-side stack bring-up + bucket-create timing).

## §2 Scope

**Branch:** `phase-m3-core-e2e-pilot` (tip `2bb790239` or newer).

**Two fixes. Two files.** No Cargo.toml changes. No Rust/test-logic changes. Both edits are in `services/bridge/` compose files (impl-owned; NOT in any never-touch list).

### Root cause (verified empirically by advisor, 2026-06-21)

The e2e run executed for real after the advisor fixed the daemon cargo PATH + chown'd the root-owned `services/bridge/target/`. Results:
- **stage_mode** → PASS (already recorded resolved).
- **recording** → 2/3 pass; `recording_lands_with_hash_on_chain` FAIL: MinIO `NoSuchBucket: recordings`.
- **room_provisioning** → `rtc_disabled_townhall_clean_posture` FAIL (bridge `createRoom` non-2xx) + 5 `todo!()` D2-deferred stubs (expected-fail).
- **emergency_mute** → marquee FAIL: `RoomClient::create_room` twirp `error decoding response body`.

LiveKit container log proved the auth root cause: `401 invalid token ... go-jose: error in cryptographic primitive`, and at startup `no keys provided, using placeholder keys {API Key: devkey, API Secret: secret}`. The bridge + tests sign JWTs with `devkey`/**`devsecret`**, but LiveKit's placeholder secret is **`secret`** — mismatch → every `create_room` 401s. This breaks BOTH emergency_mute AND room_provisioning's `rtc_disabled` test (bridge-a's own LiveKit call 401s).

Verified from `docker run --rm livekit/livekit-server:v1.7 --help`:
- `--dev` does NOT set API keys (only log-level/formatter/pprof). The `devkey`/`secret` pair is the *placeholder* fallback when no keys are provided.
- `--keys value` exists; env alias `LIVEKIT_KEYS`; **format is `key: secret`** (note the space).

The recording failure is a separate, second env gap: MinIO never creates the `recordings` bucket (only `.minio.sys` exists). MinIO does NOT auto-create buckets.

### Fix 1 — LiveKit explicit --keys so devsecret matches (docker-compose.yml)

File: `services/bridge/docker-compose.yml`, **line 57** (the livekit `command:` line). Current:

```yaml
    command: ["--dev", "--bind", "0.0.0.0"]   # --dev = devkey/devsecret; --bind 0.0.0.0 so host-native tests reach Twirp via Docker port-map (v1.7 --dev alone binds container-loopback only)
```

Replace with (adds explicit `--keys`, fixes the WRONG comment):

```yaml
    command: ["--dev", "--bind", "0.0.0.0", "--keys", "devkey: devsecret"]   # explicit --keys "key: secret" so LiveKit verifies the devsecret-signed JWTs the bridge + tests mint (v1.7 --dev does NOT set keys; its placeholder secret is "secret", not "devsecret"). --bind 0.0.0.0 for host port-map reachability.
```

**Note the exact secret string format:** `"devkey: devsecret"` — one array element, key and secret separated by `colon-space`, matching the `LIVEKIT_KEY=devkey`/`LIVEKIT_SECRET=devsecret` env on the `bridge` service (lines ~62-63) and the `LIVEKIT_API_SECRET: "${LIVEKIT_API_SECRET:-devsecret}"` defaults in docker-compose.e2e.yml.

### Fix 2 — MinIO recordings bucket auto-create (docker-compose.e2e.yml)

File: `services/bridge/docker-compose.e2e.yml`. MinIO (block at **line 94**) does not create buckets on startup. Add a one-shot `minio-init` sidecar using `minio/mc` that creates the `recordings` bucket after MinIO is up.

Insert a new service immediately AFTER the `minio:` service block (after its `networks:` lines end, before the `tuwunel-b:` block at line 112). Match the indentation of the other services (2-space service key, 4-space fields):

```yaml
  # ─── One-shot: create the recordings bucket MinIO does not auto-create ────
  minio-init:
    image: minio/mc:RELEASE.2024-11-05T11-29-45Z   # AGPL-3.0; pinned, mc CLI for bucket setup
    profiles: ["rtc"]
    depends_on:
      - minio
    entrypoint: >
      /bin/sh -c "
      until mc alias set local http://minio:9000 \"$${MINIO_ROOT_USER:-minioadmin}\" \"$${MINIO_ROOT_PASSWORD:-miniopassword}\"; do echo 'waiting for minio'; sleep 1; done &&
      mc mb --ignore-existing local/${S3_BUCKET:-recordings} &&
      echo 'minio-init: bucket ${S3_BUCKET:-recordings} ready'
      "
    networks:
      - bridge-net
```

**Critical compose-escaping note:** inside the `entrypoint`, MinIO root creds use `$${...}` (double-dollar) so docker-compose does NOT interpolate them at parse time — they expand inside the container shell from the env mc inherits. But `${S3_BUCKET:-recordings}` uses SINGLE dollar so compose substitutes the bucket name at parse time (it has no env in the mc container otherwise). This matches the bucket default the recording test uses (`S3_BUCKET` env, default `recordings` — `services/bridge/tests/recording.rs:33-34`).

Do NOT add a top-level volume for minio-init (it's stateless). Do NOT modify the existing `minio:` block.

## §3 Required reading

- `services/bridge/docker-compose.yml` lines 47-65 (the `bridge:` env block with `LIVEKIT_KEY`/`LIVEKIT_SECRET`, then the `livekit:` block ~line 57) — confirm the secret is `devsecret` so your `--keys` matches.
- `services/bridge/docker-compose.e2e.yml` lines 60-112 (the `bridge:` env S3 block at ~69-72, the `minio:` block at 94-110, where `tuwunel-b:` starts at 112) — to place minio-init correctly.
- `services/bridge/tests/recording.rs` lines 30-40 + 79-86 (confirms bucket name `recordings` + the put_object that needs it).
- `.claude/lessons/feedback_bridge_validates_on_linux_not_windows.md` — the bridge is Linux-only; you are editing compose only, no cargo, but be aware the worker runs on the daemon (Linux).
- `.claude/decision-queue.json` — read before the DQ-log write.

## §4 Procedure

1. Confirm branch: `git branch --show-current` = `phase-m3-core-e2e-pilot`.
2. Fix 1: edit docker-compose.yml line 57 — add `"--keys", "devkey: devsecret"` to the command array + replace the comment.
3. Fix 2: insert the `minio-init` service after the `minio:` block in docker-compose.e2e.yml.
4. Verify scope:
   ```bash
   git diff --stat
   ```
   Expect exactly 2 files: `docker-compose.yml`, `docker-compose.e2e.yml`.
5. Verify YAML still parses (the worker is on Linux with docker available):
   ```bash
   cd services/bridge && docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc config >/dev/null && echo COMPOSE_CONFIG_OK
   ```
   If `config` errors, fix the YAML indentation/escaping before committing. Do NOT `up` the stack.
6. Commit:
   ```bash
   git add services/bridge/docker-compose.yml services/bridge/docker-compose.e2e.yml
   git commit -m "fix(bridge/e2e): LiveKit explicit --keys devkey:devsecret + MinIO recordings bucket init"
   ```
   Commit body:
   ```
   Fix 1: docker-compose.yml — livekit command adds --keys "devkey: devsecret".
   v1.7 --dev does NOT set API keys; its placeholder secret is "secret", not the
   "devsecret" the bridge + tests sign with → every JWT 401'd (go-jose crypto
   primitive). Explicit --keys makes LiveKit verify the devsecret-signed tokens.
   Fixes emergency_mute marquee + room_provisioning rtc_disabled createRoom.

   Fix 2: docker-compose.e2e.yml — add minio-init (minio/mc) one-shot that creates
   the recordings bucket. MinIO does not auto-create buckets; recording test
   put_object got NoSuchBucket. mc mb --ignore-existing on the S3_BUCKET default.

   No Cargo.toml/Cargo.lock changes. No Rust/test-logic changes.

   LESSON: LiveKit v1.7 --dev does NOT provision API keys (only debug log-level);
   it falls back to placeholder key=devkey/secret=secret. To verify externally-signed
   JWTs you MUST pass --keys "key: secret" (env LIVEKIT_KEYS), format key-colon-space-secret.
   MinIO creates no buckets on startup; a minio/mc init sidecar (mc mb --ignore-existing)
   is the compose-native way to pre-create them for e2e.
   ```
7. Push: `git push origin phase-m3-core-e2e-pilot`.
8. Write `kind: "log"` DQ entry via `bash /srv/brehon-fork/scripts/brehon/dq-v3-new-entry.sh` then `bash /srv/brehon-fork/scripts/brehon/dq-v3-append-fragment.sh <fragment.json>`. Context: "Fix-impl #3: LiveKit explicit --keys devkey:devsecret (docker-compose.yml) + MinIO recordings-bucket minio-init sidecar (docker-compose.e2e.yml). Advisor re-runs e2e #763: bring up stack (now incl minio-init), re-run recording/room_provisioning/emergency_mute." Commit + push the DQ.

## §5 Constraints

- **2 files only**: docker-compose.yml, docker-compose.e2e.yml.
- **NO Cargo.toml/Cargo.lock changes. NO Rust changes. NO test-logic changes.**
- **NO e2e run, NO `docker compose up`** — advisor re-runs it (needs host-side stack lifecycle + aux-service juggling). `docker compose config` (parse-check only) is allowed and required per §4 step 5.
- **`--keys` value is exactly `"devkey: devsecret"`** (colon-space). Do NOT use `secret` (that's LiveKit's placeholder, the bug) and do NOT drop the space.
- **minio-init goes AFTER the `minio:` block, BEFORE `tuwunel-b:`** — match 2-space service-key indentation.
- **LESSON trailer** (in commit body above).

## §6 DoD

- `git diff --stat HEAD~1` = 2 files (docker-compose.yml, docker-compose.e2e.yml).
- `grep -- '--keys' services/bridge/docker-compose.yml` → 1 hit on the livekit command line.
- `grep -c 'devkey: devsecret' services/bridge/docker-compose.yml` → 1.
- `grep -c 'minio-init:' services/bridge/docker-compose.e2e.yml` → 1.
- `grep -c 'mc mb' services/bridge/docker-compose.e2e.yml` → 1.
- `cd services/bridge && docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc config >/dev/null && echo OK` → prints OK (YAML parses).
- DQ log entry committed + pushed.

## §7 HANDOVER

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-fix-impl-e2e-harness-3
  fix1_livekit_keys: <"--keys devkey: devsecret added" | "FAIL: <reason>">
  fix2_minio_init: <"minio-init mc sidecar added, mc mb recordings" | "FAIL: <reason>">
  compose_config_ok: <"docker compose config passed" | "FAIL: <reason>">
  files_changed: <list>
  dq_log_id: <id>
  notes: "<confirm only 2 files; confirm --keys value is colon-space devsecret>"
```
