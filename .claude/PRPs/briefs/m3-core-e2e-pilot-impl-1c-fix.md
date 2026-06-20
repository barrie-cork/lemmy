# Brief: m3-core-e2e-pilot Task 1c (fix-impl) — Conduit CONDUIT_PORT=8448 (listens on 6167 by default)

## §1 Role + dispatch line

`[role:impl-task] m3-core-e2e-pilot-task1c-fix-conduit-port-8448 — see .claude/PRPs/briefs/m3-core-e2e-pilot-impl-1c-fix.md`

You are the **impl-task** subagent (Sonnet 4.6). NARROW fix-impl for the second Task-1 reach-smoke defect. NO Rust, NO cargo, NO dependency — compose env only.

## §2 Scope

**Why:** the Task-1b rocksdb fix WORKED (both Conduit homeservers now boot healthy, no crash). But the reach-smoke probe 4 still fails (`curl(52) Empty reply` on :8449). Root cause (advisor-verified by direct probe 2026-06-20): **Conduit listens on container port `6167` by default** (`CONDUIT_PORT` is unset in both composes), but the port mappings target container **`8448`** (`8448:8448` instance-A, `8449:8448` instance-B) — nothing listens on 8448. The advisor confirmed `tuwunel-b:6167/_matrix/key/v2/server` returns `server_name: "matrix-b.localhost"` correctly (BUG-15 distinct-domain WORKS); the only problem is the listen port. Also `TUWUNEL_URL: http://tuwunel:8448` / `tuwunel-b:8448` point the bridges at the dead port. **Fix: set `CONDUIT_PORT: "8448"`** so Conduit listens on 8448 — then ALL existing 8448 mappings + `TUWUNEL_URL` become valid with zero other changes.

**Produces (2 files modified, ONE commit):**

1. **`services/bridge/docker-compose.yml`** — add `CONDUIT_PORT: "8448"` to the `tuwunel:` service `environment:` block (alongside the existing `CONDUIT_SERVER_NAME` etc, ~line 24-37). This is the BASE compose — advisor-authorised carve-out (the base is broken; same as Task-1b).

2. **`services/bridge/docker-compose.e2e.yml`** — add `CONDUIT_PORT: "8448"` to the `tuwunel-b:` service `environment:` block (~line 114-121). The override's `tuwunel:` section (~line 32) does NOT need it — it inherits the base via compose env-merge. **VERIFY before editing:** the override `tuwunel:` block should NOT get its own CONDUIT_PORT (would be redundant; base covers it). Only `tuwunel-b:` needs the explicit add here.

**Do NOT:**
- Change any port MAPPING (`8448:8448`, `8449:8448` stay — they're correct once Conduit listens on 8448).
- Change `TUWUNEL_URL` (`:8448` is correct once Conduit listens there).
- Change the rocksdb backend (Task-1b fixed it; leave it), the distinct-domain config, federation wiring, MinIO/LiveKit/bridge defs, registration-b.yaml, or the smoke script (the readiness poll from Task-1b is fine).
- Add Rust, Cargo, dependency, or touch any `crates/**`/migration/const/registry/plan.
- Change the image tag (matrix-conduit:v0.6.0 is correct).

**Branch:** forks from `phase-m3-core-e2e-pilot` (current tip `521230c2d`).

## §3 Required reading

- **`services/bridge/docker-compose.yml:19-44`** — base `tuwunel:` env block (add CONDUIT_PORT here; note rocksdb already set by Task-1b).
- **`services/bridge/docker-compose.e2e.yml:112-131`** — `tuwunel-b:` env block (add CONDUIT_PORT here; note `tuwunel:` at ~:32 inherits base).
- **Lessons:** `feedback_validate_pending_laptop_write_then_stop.md` (write the DQ + STOP; laptop re-runs smoke). `feedback_bridge_validates_on_linux_not_windows.md` (config only; no cargo/Docker by you).

## §4 Constraints

- **`8448` is the verified-correct CONDUIT_PORT value** — it aligns with the existing mappings (`8448:8448`, `8449:8448`) + `TUWUNEL_URL` (`:8448`). The advisor confirmed Conduit serves the key endpoint correctly once it listens on the mapped port. Do not pick a different port or change the mappings instead.
- **Exactly 2 env additions** — base `tuwunel` + override `tuwunel-b`. If you find the override `tuwunel:` block ALSO needs it (i.e. env-merge doesn't apply), STOP and raise a `kind: "blocker"` DQ (advisor's merge assumption wrong). Otherwise 2 is correct.
- **Leave Task-1b's work intact** — rocksdb backend + the smoke readiness poll stay. You ONLY add CONDUIT_PORT.
- **NO daemon Docker / NO cargo.** Write a `validate-pending-laptop-e2e` DQ (`bash scripts/brehon/dq-v3-new-entry.sh` for id, `dq-v3-append-fragment.sh <frag>.json --pending` to append):
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
  Commit + push the DQ on the worker branch, then **STOP** — the laptop advisor re-runs the smoke.
- **ONE commit** — `fix(rtc): set CONDUIT_PORT=8448 (Conduit defaults to 6167) so port mappings + TUWUNEL_URL resolve (task 1c)`. End the body with a `LESSON:` trailer: Conduit's default listen port is 6167; compose mappings/TUWUNEL_URL must either set CONDUIT_PORT to match or map to 6167.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.
- **Handover:** inline in task output (no handover file — sensitive-file guard).

## §3a Handover from prior tasks

- Task 1 #748 — authored the harness (override compose + smoke + registration-b). Correct except inherited config bugs.
- Task 1b #749 `4f42fcd71` — fixed CONDUIT_DATABASE_BACKEND sled→rocksdb (matrix-conduit:v0.6.0 has no sled) + smoke 45s→MinIO readiness poll. Homeservers now boot healthy.
- Advisor re-ran smoke: probes 1-3 PASS; probe 4 fails because Conduit listens on 6167 not 8448. Verified tuwunel-b:6167 serves the key endpoint with server_name=matrix-b.localhost. DQ `53441afbc986-001` marked fail with this diagnosis.
- Phase tip `521230c2d`.

## §5 HANDOVER (worker fills at task end — return inline)

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-task1c-fix
  filesModified: [services/bridge/docker-compose.yml, services/bridge/docker-compose.e2e.yml]
  keyDecisions:
    - "CONDUIT_PORT: \"8448\" added to base tuwunel env + override tuwunel-b env (2 services; override tuwunel-a inherits base)"
    - "port mappings + TUWUNEL_URL UNCHANGED (correct once Conduit listens on 8448); rocksdb + smoke readiness poll UNTOUCHED"
  validate_dq: <composite-id of the new validate-pending-laptop-e2e DQ>
  notes: "<confirm exactly 2 CONDUIT_PORT additions; confirm no mapping/TUWUNEL_URL/rocksdb/image change>"
```
