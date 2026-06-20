# Brief: m3-core-e2e-pilot Task 1b (fix-impl) — Conduit DB backend sled→rocksdb + reach-smoke readiness poll

## §1 Role + dispatch line

`[role:impl-task] m3-core-e2e-pilot-task1b-fix-conduit-rocksdb-backend-and-smoke-readiness — see .claude/PRPs/briefs/m3-core-e2e-pilot-impl-1b-fix.md`

You are the **impl-task** subagent (Sonnet 4.6). This is a NARROW fix-impl for a Task-1 reach-smoke failure. NO Rust, NO cargo, NO dependency — config + shell only.

## §2 Scope

**Why:** the Task-1 reach-smoke FAILED. Root cause (advisor-verified by direct probe 2026-06-20): BOTH `tuwunel` (instance-A) AND `tuwunel-b` Exit(1) on init with `BadConfig("Database backend not found.")` because `CONDUIT_DATABASE_BACKEND: "sled"` is NOT compiled into the `matrixconduit/matrix-conduit:v0.6.0` image. The advisor booted that exact image with `CONDUIT_DATABASE_BACKEND=rocksdb` and it started clean (`Created new rocksdb database with version 13`). **The fix is `sled`→`rocksdb`.** The image is correct (Conduit is the documented upstream of Tuwunel; local-dev placeholder) — do NOT change the image tag.

**Produces (2 files modified, ONE commit):**

1. **`services/bridge/docker-compose.yml`** (line 26) — change `CONDUIT_DATABASE_BACKEND: "sled"` → `CONDUIT_DATABASE_BACKEND: "rocksdb"` in the `tuwunel:` service. (This is the BASE compose — the Task-1 brief said don't modify base, but the base is itself broken; this carve-out is advisor-authorised per gate decision 2026-06-20.)

2. **`services/bridge/docker-compose.e2e.yml`** (line 116) — change `CONDUIT_DATABASE_BACKEND: "sled"` → `CONDUIT_DATABASE_BACKEND: "rocksdb"` in the `tuwunel-b:` service. (The override's `tuwunel:` section at ~line 32 does NOT set the backend — it inherits the base via compose merge, so the base fix covers instance-A. ONLY tuwunel-b needs an explicit change here. VERIFY by grep before editing: `grep -n CONDUIT_DATABASE_BACKEND services/bridge/docker-compose.e2e.yml` should show exactly 1 occurrence at ~116.)

3. **`scripts/brehon/e2e-harness-smoke.sh`** — replace the fixed `sleep 45` init wait with a **readiness-poll loop** for MinIO (the secondary nit: MinIO returned 200 at ~5min but not at the 45s mark on a cold start). Keep it simple: poll `curl -fsS http://localhost:9000/minio/health/live` every 5s up to ~24 tries (2 min cap), break on first success; if the cap is hit, fail with a clear message. This makes the smoke robust to cold-start timing without a magic-number sleep. (ponytail: a bounded readiness poll, not a fixed sleep — the upgrade path if other services also race is to poll each, but MinIO is the slow one.)

**Do NOT:**
- Change any image tag (the image is correct; only the backend value is wrong).
- Add any Rust, Cargo, or dependency.
- Touch the distinct-domain config, the federation wiring, the port mappings, MinIO/LiveKit/bridge service defs, or `registration-b.yaml` — those are all correct (MinIO+LiveKit+bridge-a reached fine; only the homeservers crashed).
- Modify any `crates/**`, migration, const, registry, or plan file.

**Branch:** forks from `phase-m3-core-e2e-pilot` (current tip `e8b35d1aa`).

## §3 Required reading

- **`services/bridge/docker-compose.yml:23-44`** — the base `tuwunel:` service (the `sled` at :26 to fix; note the image-pin comment confirming Conduit-as-Tuwunel-upstream).
- **`services/bridge/docker-compose.e2e.yml:112-131`** — the `tuwunel-b:` service (the `sled` at ~:116 to fix; note `tuwunel:` at ~:32 does NOT set backend → inherits base).
- **`scripts/brehon/e2e-harness-smoke.sh`** — the smoke (the `sleep 45` to replace with a MinIO readiness poll; probes 1-4 stay otherwise unchanged).
- **Lessons:** `feedback_validate_pending_laptop_write_then_stop.md` (write the DQ + STOP; the laptop re-runs the smoke). `feedback_bridge_validates_on_linux_not_windows.md` (you author config; you run NO cargo, NO Docker).

## §4 Constraints

- **`rocksdb` is the verified-correct value** — the advisor empirically confirmed `matrix-conduit:v0.6.0` boots with `rocksdb` and crashes with `sled`. Do not second-guess to a third value.
- **Exactly 2 backend lines change** — base :26 + override tuwunel-b :116. Confirm the override's `tuwunel:` section does NOT carry its own backend (grep `CONDUIT_DATABASE_BACKEND` in the override = 1 hit). If there are MORE than 2 total across both files, STOP and raise a `kind: "blocker"` DQ (the advisor's scope assumption was wrong).
- **Smoke readiness poll stays bounded** — a finite retry cap (≤2 min) with a clear timeout failure, NOT an unbounded `while true`. `set -euo pipefail` must still hold.
- **NO daemon Docker / NO cargo.** Write a `validate-pending-laptop-e2e` DQ (use `bash scripts/brehon/dq-v3-new-entry.sh` for the id, `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append):
  ```
  kind: "validate-pending-laptop-e2e"
  branch: "phase-m3-core-e2e-pilot"
  phase_task: 1
  commands: ["bash scripts/brehon/e2e-harness-smoke.sh"]
  e2e_filter: null
  result: null
  log_slice: null
  failed_commands: null
  ```
  Commit + push the DQ on the worker branch, then **STOP** — do NOT bring the stack up or run the smoke (the laptop advisor re-runs it). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **ONE commit** — `fix(rtc): conduit DB backend sled→rocksdb (unsupported in matrix-conduit:v0.6.0) + smoke MinIO readiness poll (task 1b)`. End the body with a `LESSON:` trailer noting the matrix-conduit:v0.6.0 sled-not-compiled gotcha (durable: future Conduit-image work must use rocksdb).
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.
- **Handover:** deliver the HANDOVER block inline in your task output (no `.claude/PRPs/handovers/` file — sensitive-file guard).

## §3a Handover from prior task

- Task 1 #748 `f8cd5f3f8` (merged `eb6afd110`) — authored `docker-compose.e2e.yml` (MinIO + tuwunel-b/bridge-b, distinct domains) + `e2e-harness-smoke.sh` + `registration-b.yaml`. ALL correct EXCEPT the inherited `sled` backend (mirrored from the broken base). Reach-smoke probes 1-3 (MinIO/LiveKit/bridge-a) PASS; probe 4 (tuwunel-b) FAIL because both homeservers crashed on `sled`.
- DQ `ec2aa316b69b-001` marked `result=fail` (`e8b35d1aa`) with the full diagnosis.
- Advisor verified the fix: `matrix-conduit:v0.6.0` + `CONDUIT_DATABASE_BACKEND=rocksdb` boots clean.
- Phase branch tip `e8b35d1aa`.

## §5 HANDOVER (worker fills at task end — return inline in task output)

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-task1b-fix
  filesModified: [services/bridge/docker-compose.yml, services/bridge/docker-compose.e2e.yml, scripts/brehon/e2e-harness-smoke.sh]
  keyDecisions:
    - "CONDUIT_DATABASE_BACKEND sled->rocksdb in base :26 + override tuwunel-b :116 (grep count of backend lines: <N>, expect 2 total)"
    - "smoke: sleep 45 -> MinIO readiness poll (5s interval, ~2min cap)"
    - "image UNCHANGED (matrix-conduit:v0.6.0 correct); distinct-domain/federation/ports UNCHANGED"
  validate_dq: <composite-id of the new validate-pending-laptop-e2e DQ>
  notes: "<confirm exactly 2 backend lines changed; confirm readiness poll bounded; confirm no image/domain/port change>"
```
