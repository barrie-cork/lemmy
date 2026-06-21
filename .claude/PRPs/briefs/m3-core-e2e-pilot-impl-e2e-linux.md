# Brief: m3-core-e2e-pilot Linux e2e live run — EliteDesk native (4 bridge test targets)

## §1 Role + dispatch line

`[role:impl-task] m3-core-e2e-pilot-e2e-linux-run — see .claude/PRPs/briefs/m3-core-e2e-pilot-impl-e2e-linux.md`

You are the **impl-task** subagent (Sonnet 4.6). This is an **operational e2e run task** — NO code edits, NO DQ entries for validation gates, NO cargo compilation work. You bring up the docker-compose e2e stack, run 4 test targets natively (on the Linux host with `--network host`), record results, and write one DQ entry summarising the outcome. Then STOP.

**Why Linux:** the Windows laptop's `cargo-linux.sh` runs tests inside a network-isolated Docker container where `localhost:PORT` can't reach the compose stack. On Linux, `--network host` is real host networking. The test binary on the Linux host (EliteDesk) sees `localhost:8080` = bridge-a, `localhost:7880` = LiveKit, etc.

## §2 Scope

**Branch:** `phase-m3-core-e2e-pilot` (current tip `08696191d` or newer — confirm at task start).

**What you produce:**
1. Bring up the e2e stack on the EliteDesk using docker-compose.
2. Run 4 cargo test targets natively via `cargo test --manifest-path services/bridge/Cargo.toml --test <target> -- --ignored`.
3. Capture exit codes and failure slices.
4. Write ONE `kind: "log"` DQ entry to `.claude/decision-queue.json` with the per-target results.
5. Commit + push the DQ update on the phase branch.

**What you do NOT do:**
- Do NOT edit any source file (`crates/`, `services/bridge/src/`, migrations, tests, plans).
- Do NOT write `validate-pending-laptop-*` DQ entries — those are the laptop's gate pattern. This task writes a `kind: "log"` summary only.
- Do NOT leave the compose stack running after the test run. Always teardown.

## §3 Required reading

- **`services/bridge/docker-compose.yml`** + **`services/bridge/docker-compose.e2e.yml`** — the override-layered stack definition.
- **`scripts/brehon/e2e-harness-smoke.sh`** — the smoke script (probes MinIO + LiveKit + bridge-a + tuwunel-b). Run this BEFORE the 4 test targets.
- **`.claude/decision-queue.json`** (current phase-branch state) — read before writing the DQ entry; use `bash scripts/brehon/dq-v3-new-entry.sh` for the id.

## §4 Procedure

### Step 1 — Confirm branch + Docker available

```bash
git branch --show-current  # must be phase-m3-core-e2e-pilot
git log -1 --oneline
docker info --format '{{.OSType}}'  # must print: linux
```

If not on the phase branch: `git checkout phase-m3-core-e2e-pilot`.
If Docker not running: raise a `kind: "blocker"` DQ and STOP.

### Step 2 — Bring up the e2e stack

```bash
cd services/bridge
docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc up -d --build
```

This may take 10-15 minutes on first run (cold cargo build inside the bridge image).
Once up, run the smoke script:

```bash
cd /srv/brehon-fork
bash scripts/brehon/e2e-harness-smoke.sh 2>&1 | tee /tmp/m3-e2e-linux-smoke.log
grep 'SMOKE_EXIT\|E2E_HARNESS_REACH_OK\|FAIL\|error' /tmp/m3-e2e-linux-smoke.log | tail -10
```

**If SMOKE_EXIT != 0:** teardown (Step 4), write DQ with smoke failure, STOP.

### Step 3 — Run the 4 test targets natively (NOT via cargo-linux.sh)

Run cargo directly on the host (Linux), NOT inside a Docker container:

```bash
cd /srv/brehon-fork
```

For each target in `[stage_mode, recording, room_provisioning, emergency_mute]`:

```bash
cargo test --manifest-path services/bridge/Cargo.toml --test <TARGET> -- --ignored > /tmp/m3-e2e-linux-<TARGET>.log 2>&1
echo "E2E_<TARGET>_EXIT=$?"
```

Capture each exit code. Read the tail of the log on failure:

```bash
tail -30 /tmp/m3-e2e-linux-<TARGET>.log
```

**IMPORTANT for `room_provisioning`:** 5 of the 7 tests are `todo!("implement against live docker-compose stack")` stubs — they will FAIL with `not yet implemented`. That is expected; they are D2 pilot stubs. The 2 non-stub tests are `rtc_disabled_townhall_clean_posture` and `anonymous_townhall_identity_never_reaches_livekit`. Note which tests failed with `not yet implemented` vs real failures.

**IMPORTANT for `emergency_mute`:** the test is `mute_all_drops_all_publishers_cross_instance_under_500ms`. If it PASSES, note the elapsed time printed in the output. If it FAILS, capture the exact error line (not just the exit code).

### Step 4 — Teardown

```bash
cd /srv/brehon-fork/services/bridge
docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc down -v
```

### Step 5 — Write DQ log entry

Use `bash /srv/brehon-fork/scripts/brehon/dq-v3-new-entry.sh` to generate the id.

Write a fragment file at `/tmp/m3-e2e-linux-dq-frag.json`:

```json
{
  "id": "<generated-id>",
  "from": "impl",
  "kind": "log",
  "timestamp": "<ISO8601-UTC>",
  "question": "m3-core-e2e-pilot Linux e2e live run — per-target results",
  "options": ["pass", "fail"],
  "context": "Ran 4 cargo test targets natively on EliteDesk (Linux host, --network host = real host networking). smoke=<0|N> stage_mode=<0|N> recording=<0|N> room_provisioning=<0|N> emergency_mute=<0|N>. Note: 5 room_provisioning tests are todo!() stubs (expected fail). Key finding: <one line>.",
  "answer": "results documented above; advisor to assess pass/fail and update -e2e DQ entries",
  "answered_by": "impl-self-resolved",
  "resolved_at": "<ISO8601-UTC>",
  "approved_by": null,
  "approved_at": null
}
```

Append it to resolved[]: `bash /srv/brehon-fork/scripts/brehon/dq-v3-append-fragment.sh /tmp/m3-e2e-linux-dq-frag.json`

Delete the fragment: `rm /tmp/m3-e2e-linux-dq-frag.json`

Commit + push on the phase branch:
```bash
git add .claude/decision-queue.json
git commit -m "chore(decision-queue): impl logged m3 Linux e2e run results — <summary in 5 words>"
git push origin phase-m3-core-e2e-pilot
```

## §5 Constraints

- **NO source edits** — this task is purely operational. Any file in `crates/`, `services/bridge/src/`, migrations, or plans is off-limits.
- **NO daemon cargo via cargo-linux.sh** — you are running cargo NATIVELY on the Linux host. The NO-CARGO-ON-ELITEDESK rule applies to Brehon workspace (`crates/`) only; the bridge (`services/bridge/`) is a separate Rust project that compiles fine on Linux natively.
- **NO validate-pending-laptop DQ** — write `kind: "log"` only (self-resolved summary). The advisor reads the log entry and updates the existing `-e2e` DQ entries (ids: 79d716d1f587-002, 1c37b605fd1a-002, c2e678abdbb2-002, 44243654b24d-002).
- **Teardown always** — even on smoke failure, run `docker compose down -v` before stopping.
- **LESSON: trailer** — commit body: `LESSON: bridge e2e tests need native Linux host (--network host = real host on Linux; WSL2 on Windows Docker Desktop does not expose Windows host localhost).`

## §6 HANDOVER (worker fills inline at task end)

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-e2e-linux-run
  smoke: <0|N>
  stage_mode: <0|N>
  recording: <0|N — note: todo!() stubs excluded>
  room_provisioning: <0|N — note: 5 stubs expected-fail; 2 real tests: rtc_disabled + anonymous_identity>
  emergency_mute: <0|N — note: marquee; if pass, cite elapsed ms>
  key_finding: "<one sentence>"
  dq_log_id: <id of the log entry written>
  notes: "<confirm no source edits; confirm teardown ran; confirm DQ committed+pushed>"
```
