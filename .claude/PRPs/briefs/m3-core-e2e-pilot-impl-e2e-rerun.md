# Brief: m3-core-e2e-pilot e2e re-run — recording + room_provisioning + emergency_mute

## §1 Role + dispatch line

`[role:impl-task] m3-core-e2e-pilot-e2e-rerun — see .claude/PRPs/briefs/m3-core-e2e-pilot-impl-e2e-rerun.md`

You are the **impl-task** subagent (Sonnet 4.6). This is a **pure e2e run task** — NO code edits, NO Cargo.toml changes. You bring up the compose stack, set `BRIDGE_DB_PATH` from the live Docker volume, run 3 test targets natively on Linux, record results in a DQ log entry. Then teardown and STOP.

**Context:** Junior #759 ran e2e on Linux and found:
- stage_mode: PASS ✓ (confirmed; no re-test needed)
- recording=101, room_provisioning=101, emergency_mute=101 — failed due to:
  - LiveKit v1.8 h2c requirement vs livekit-api 0.4.24 HTTP/1.1 → **FIXED** (v1.7 pinned in docker-compose.yml, commit `31895b60e`)
  - BRIDGE_DB_PATH defaulting to `/data/bridge-a.db` (container-internal) when tests read the DB directly → **MUST SET** from Docker volume mountpoint
  - rust-s3 virtual-hosted URL style → **FIXED** (`Bucket::new_with_path_style` 2×, commit `31895b60e`)

This re-run targets the 3 failing targets with all fixes applied.

## §2 Scope

**Branch:** `phase-m3-core-e2e-pilot` (tip `09abfaf5a` or newer — confirm at task start).

**What you produce:**
1. Bring up e2e compose stack on EliteDesk.
2. Compute `BRIDGE_DB_PATH` from live Docker volume.
3. Run 3 cargo test targets natively: `recording`, `room_provisioning`, `emergency_mute`.
4. Write ONE `kind: "log"` DQ entry with per-target results.
5. Teardown compose stack.

**Do NOT:**
- Edit any source file.
- Write `validate-pending-laptop-*` DQ entries.
- Leave the compose stack running.

## §3 Required reading

- `services/bridge/docker-compose.yml` + `services/bridge/docker-compose.e2e.yml`
- `scripts/brehon/e2e-harness-smoke.sh`
- `.claude/decision-queue.json` (use `bash scripts/brehon/dq-v3-new-entry.sh` for id)

## §4 Procedure

### Step 1 — Confirm branch + Docker

```bash
cd /srv/brehon-fork
git branch --show-current   # must be phase-m3-core-e2e-pilot
git log -1 --oneline        # confirm tip is 09abfaf5a or newer
docker info --format '{{.OSType}}'  # must print: linux
```

Confirm the LiveKit fix is present:
```bash
grep 'v1.7' services/bridge/docker-compose.yml
```
Must print the livekit image line with `v1.7`. If missing, STOP and write a `kind: "blocker"` DQ.

### Step 2 — Bring up the e2e stack

```bash
cd /srv/brehon-fork/services/bridge
docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc up -d --build
```

Wait for all containers healthy (~2 min warm, ~10-15 min cold Docker build).

Run smoke:
```bash
cd /srv/brehon-fork
bash scripts/brehon/e2e-harness-smoke.sh > /tmp/m3-e2e-rerun-smoke.log 2>&1
SMOKE_EXIT=$?
tail -10 /tmp/m3-e2e-rerun-smoke.log
echo "SMOKE_EXIT=$SMOKE_EXIT"
```

If SMOKE_EXIT != 0: teardown (Step 5), write DQ log with smoke failure, STOP.

### Step 3 — Compute BRIDGE_DB_PATH

After the stack is up, compute the host-accessible path to the bridge-a SQLite DB:

```bash
BRIDGE_VOLUME_MOUNT=$(docker volume inspect bridge_bridge_a_data --format '{{.Mountpoint}}')
export BRIDGE_DB_PATH="${BRIDGE_VOLUME_MOUNT}/bridge-a.db"
echo "BRIDGE_DB_PATH=$BRIDGE_DB_PATH"
```

Verify the file exists:
```bash
ls -la "$BRIDGE_DB_PATH"
```

If the file does not exist yet, the bridge-a container may not have initialized it. Wait 30s and retry. If still absent after 60s: teardown, write DQ log noting the path, STOP.

**Important:** `BRIDGE_DB_PATH` MUST be set in the shell environment for the cargo test invocations in Step 4. Keep the same shell session for Steps 3 and 4.

### Step 4 — Run 3 test targets natively

Run each target in order. Capture to file (do NOT pipe through tail/grep — exit code must be from cargo, not the pipe):

#### recording

```bash
cd /srv/brehon-fork
cargo test --manifest-path services/bridge/Cargo.toml --test recording -- --ignored > /tmp/m3-e2e-rerun-recording.log 2>&1
RECORDING_EXIT=$?
echo "RECORDING_EXIT=$RECORDING_EXIT"
tail -30 /tmp/m3-e2e-rerun-recording.log
```

#### room_provisioning

```bash
cargo test --manifest-path services/bridge/Cargo.toml --test room_provisioning -- --ignored > /tmp/m3-e2e-rerun-room_provisioning.log 2>&1
ROOM_EXIT=$?
echo "ROOM_EXIT=$ROOM_EXIT"
tail -40 /tmp/m3-e2e-rerun-room_provisioning.log
```

**NOTE for room_provisioning:** 5 of 7 tests are `todo!("implement against live docker-compose stack")` stubs — they WILL FAIL with `not yet implemented`. That is expected (D2 pilot deferred work). The 2 non-stub tests are:
- `rtc_disabled_townhall_clean_posture` — reads BRIDGE_DB_PATH SQLite
- `anonymous_townhall_identity_never_reaches_livekit` — makes LiveKit API call

Note which tests fail with `not yet implemented` vs real failures.

#### emergency_mute

```bash
cargo test --manifest-path services/bridge/Cargo.toml --test emergency_mute -- --ignored > /tmp/m3-e2e-rerun-emergency_mute.log 2>&1
MUTE_EXIT=$?
echo "MUTE_EXIT=$MUTE_EXIT"
tail -40 /tmp/m3-e2e-rerun-emergency_mute.log
```

**NOTE for emergency_mute:** the test is `mute_all_drops_all_publishers_cross_instance_under_500ms`. If it PASSES, note the elapsed time from the output. If it FAILS, capture the exact error line.

### Step 5 — Teardown

```bash
cd /srv/brehon-fork/services/bridge
docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc down -v
```

Run teardown even if tests failed.

### Step 6 — Write DQ log entry

```bash
cd /srv/brehon-fork
bash scripts/brehon/dq-v3-new-entry.sh
```

Write fragment at `/tmp/m3-e2e-rerun-dq-frag.json`:

```json
{
  "id": "<generated-id>",
  "from": "impl",
  "kind": "log",
  "timestamp": "<ISO8601-UTC>",
  "question": "m3-core-e2e-pilot e2e re-run (v1.7 + path-style fixes) — recording/room_provisioning/emergency_mute results",
  "options": ["all-pass", "partial-pass", "all-fail"],
  "context": "Re-ran 3 targets after LiveKit v1.7 pin + rust-s3 path-style fix (commit 31895b60e). BRIDGE_DB_PATH=<computed path>. smoke=<0|N> recording=<0|N> room_provisioning=<0|N — note: 5 todo! stubs counted as expected-fail; 2 real tests: rtc_disabled + anonymous_identity> emergency_mute=<0|N — if pass, note elapsed ms>. Key finding: <one line>.",
  "answer": "results documented; advisor to update -e2e DQ entries (1c37b605fd1a-002, c2e678abdbb2-002, 44243654b24d-002)",
  "answered_by": "impl-self-resolved",
  "resolved_at": "<ISO8601-UTC>",
  "approved_by": null,
  "approved_at": null
}
```

Append to resolved[]:
```bash
bash /srv/brehon-fork/scripts/brehon/dq-v3-append-fragment.sh /tmp/m3-e2e-rerun-dq-frag.json
rm /tmp/m3-e2e-rerun-dq-frag.json
```

Commit + push:
```bash
git add .claude/decision-queue.json
git commit -m "chore(decision-queue): impl logged m3 e2e re-run results — <5-word summary of outcome>"
git push origin phase-m3-core-e2e-pilot
```

Commit body:
```
BRIDGE_DB_PATH set from docker volume inspect (host-accessible path).
LiveKit v1.7 fix applied (commit 31895b60e).
rust-s3 path-style fix applied (commit 31895b60e).

LESSON: BRIDGE_DB_PATH must be computed from `docker volume inspect bridge_bridge_a_data --format '{{.Mountpoint}}'` after stack is up; the default /data/bridge-a.db is container-internal only.
```

## §5 Constraints

- **NO source edits** — operational run only.
- **NO daemon cargo (Brehon workspace)** — bridge cargo on Linux host is permitted (separate workspace from `crates/`).
- **NO `cargo-linux.sh`** — run cargo NATIVELY. cargo-linux.sh runs inside an isolated container and can't reach compose services.
- **BRIDGE_DB_PATH must be exported** before running recording and room_provisioning (they read SQLite directly).
- **Teardown always** — even on failure.
- **Same shell session** for Steps 3 and 4 (env var must be inherited by cargo invocations).

## §6 DoD

- Smoke: exit 0.
- recording: exit logged (0 = pass; non-zero = note exact error from tail).
- room_provisioning: 2 non-stub tests result logged; 5 todo! stubs documented as expected-fail.
- emergency_mute: exit logged; if pass, elapsed time noted.
- DQ log entry written, committed, pushed.
- Compose stack torn down.

## §7 HANDOVER

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-e2e-rerun
  bridge_db_path: <computed path>
  smoke: <0|N>
  recording: <0|N>
  room_provisioning_rtc_disabled: <PASS|FAIL — the real test>
  room_provisioning_anonymous_identity: <PASS|FAIL — the real test>
  room_provisioning_stubs: <5 expected todo! failures>
  emergency_mute: <0|N — if 0, note elapsed ms>
  dq_log_id: <id>
  key_finding: "<one sentence>"
  notes: "<confirm no source edits; teardown ran>"
```
