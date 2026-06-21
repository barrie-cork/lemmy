# Brief — m3-core-e2e-pilot impl Task 1 (defect 1): :3000 room-event stub sidecar

## §1 Role + dispatch line

`[role:impl-task] defect1-stub — see .claude/PRPs/briefs/m3-core-e2e-pilot-impl-defect1-room-event-stub.md`

You are the **impl-task** subagent (Sonnet 4.6). One compose-config edit. After it, write a `validate-pending-laptop-e2e` DQ, commit + push, **STOP**. Do NOT run docker or cargo — the advisor runs the e2e (needs host-side stack lifecycle).

## §2 Scope

**Branch:** `phase-m3-core-e2e-pilot` (fork tip `d69d41c1a` or newer).

**One file. One service added.** No Rust, no Cargo, no test-logic changes.

Add a `room-event-stub` container to `services/bridge/docker-compose.e2e.yml` so the recording test's on-chain POST to host `:3000` gets a 2xx. Plan: `.claude/PRPs/plans/m3-core-e2e-pilot-e2e-fixes.plan.md` §10.1 + Task 1.

### IMPLEMENT (file 1 of 1)

In `services/bridge/docker-compose.e2e.yml`, under `services:`, add the VERBATIM block from plan §10.1, adjacent to the `minio` block:

```yaml
  # ─── Brehon governance :3000 stub for recording on-chain POST (defect 1) ────
  # recording_lands_with_hash_on_chain (and the bridge's drain_emits) POST
  # room_recording_uploaded to BREHON_ROOM_EVENT_URL (host :3000). The base e2e
  # stack runs no Brehon binary; this stub returns HTTP 200 to ANY request so the
  # test's R-CHAIN status-success assertion (recording.rs:118-123) is satisfied.
  # It does NOT persist a governance_log row — chain persistence is the governance
  # suite's job (crates/server/tests/e2e/governance.rs), NOT this bridge integration
  # test (which only asserts the bridge POSTs a well-formed 2xx room-event).
  room-event-stub:
    image: caddy:2.8-alpine            # Apache-2.0; pinned. `caddy respond` = fixed-status responder.
    command: ["caddy", "respond", "--listen", ":3000", "--status", "200", "--body", "room-event-stub-ok"]
    ports:
      - "3000:3000"
    networks:
      - bridge-net
```

**No `profiles:` key** — the stub must be up whenever the bridge is up (the bridge's `BREHON_ROOM_EVENT_URL` targets :3000 unconditionally). The recording run uses `--profile rtc` (a superset), so a no-profile service is included.

**Fallback (note in commit body if used):** if `caddy:2.8-alpine` lacks the `respond` subcommand, substitute `traefik/whoami` (returns 200 to any method). Do NOT silently swap the image — verify `caddy respond` accepts these flags first (`docker run --rm caddy:2.8-alpine caddy respond --help` if docker is available to you; otherwise leave caddy and note "unverified — advisor confirms at up -d").

## §3 Required reading (phase-branch versions)

- `services/bridge/docker-compose.e2e.yml` — the `minio` / `minio-init` sidecar shape (§10.1 mirror) + the `bridge` env block showing `BREHON_ROOM_EVENT_URL: http://host.docker.internal:3000/...` (the bridge already targets host :3000).
- `services/bridge/tests/recording.rs:88-133` — confirm the R-CHAIN assertion is ONLY `chain_resp.status().is_success()` (lines 118-123); no body read, no persistence check. This is why a 200-only stub is e2e-honest for THIS test.
- `.claude/PRPs/plans/m3-core-e2e-pilot-e2e-fixes.plan.md` §10.1 + Task 1 + §18 (the caddy-flag risk row).
- `.claude/lessons/feedback_bridge_validates_on_linux_not_windows.md` — the worker runs on the Linux daemon; you edit compose only, no cargo.
- `.claude/decision-queue.json` — read before the DQ write.

## §4 Procedure

1. Confirm branch: `git branch --show-current` = `phase-m3-core-e2e-pilot`.
2. Add the `room-event-stub` service (§2 verbatim).
3. Verify scope: `git diff --stat` = exactly 1 file (`docker-compose.e2e.yml`).
4. Verify YAML parses: `cd services/bridge && docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc config >/dev/null && echo COMPOSE_CONFIG_OK`. Fix indentation/escaping before committing. Do NOT `up` the stack.
5. Commit: `feat(e2e): room-event :3000 stub for recording on-chain POST (defect 1)`. Body: name the fallback if used + `LESSON:` if any.
6. Push: `git push origin phase-m3-core-e2e-pilot`.
7. Write the `validate-pending-laptop-e2e` DQ entry via `bash /srv/brehon-fork/scripts/brehon/dq-v3-new-entry.sh` then `dq-v3-append-fragment.sh <fragment> --pending`. Commit + push the DQ. Then STOP.

DQ fragment shape:
```
kind: validate-pending-laptop-e2e
from: impl
commands: ["cargo test --manifest-path services/bridge/Cargo.toml --test recording -- --ignored"]
branch: phase-m3-core-e2e-pilot
phase_task: 1
result: null, log_slice: null, failed_commands: null
context: advisor brings up the FULL e2e stack first (cd services/bridge && docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc up -d) which now includes room-event-stub on :3000; export PATH + BRIDGE_DB_PATH per the handover re-run procedure.
```

## §5 Constraints

- **1 file only**: `services/bridge/docker-compose.e2e.yml`. NO Rust, NO Cargo, NO test edits.
- **NO docker up, NO cargo** — write-then-STOP (`feedback_validate_pending_laptop_write_then_stop.md`). `docker compose config` (parse-check) is allowed + required.
- **No `profiles:` key** on the stub.
- **No silent image swap** — caddy is the default; document any fallback.
- DQ mid-task push mandatory (`decision-queue.md` Mid-task visibility).

## §6 DoD

- `git diff --stat HEAD~1` = 1 file.
- `grep -c 'room-event-stub:' services/bridge/docker-compose.e2e.yml` → 1.
- `grep -c '3000:3000' services/bridge/docker-compose.e2e.yml` → ≥1.
- `docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc config >/dev/null && echo OK` → OK.
- validate-pending-laptop-e2e DQ committed + pushed.

## §7 HANDOVER

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-impl-defect1-room-event-stub
  stub_added: <"room-event-stub caddy :3000 added" | "FAIL: <reason>">
  image_used: <"caddy:2.8-alpine" | "traefik/whoami (fallback — caddy respond unavailable)">
  compose_config_ok: <"passed" | "FAIL: <reason>">
  files_changed: <list>
  dq_id: <id>
  notes: "<confirm 1 file; confirm no profiles key>"
```
