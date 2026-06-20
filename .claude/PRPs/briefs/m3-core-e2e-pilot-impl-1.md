# Brief: m3-core-e2e-pilot Task 1 — e2e acceptance harness (MinIO + 2nd federated instance override + reach-smoke)

## §1 Role + dispatch line

`[role:impl-task] m3-core-e2e-pilot-task1-e2e-harness-override-compose-reach-smoke — see .claude/PRPs/briefs/m3-core-e2e-pilot-impl-1.md`

You are the **impl-task** subagent (Sonnet 4.6). This task is **infra config + a smoke script — NO Rust, NO cargo, NO dependency**. It is the load-bearing risk-isolation task: the net-new e2e acceptance harness that every later acceptance test (Tasks 3–6) hard-`requires:`.

## §2 Scope

**Plan task: `.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` §13 Task 1** (read it — full ACTION/IMPLEMENT/MIRROR/GOTCHA/VALIDATE there). Also §10.1 (override compose), §10.2 (reach-smoke).

**Produces (2 files created, possibly +2 for instance-B config, ONE commit):**

1. **`services/bridge/docker-compose.e2e.yml`** — a NEW **override** compose (layered on the base via `docker compose -f docker-compose.yml -f docker-compose.e2e.yml --profile rtc up`, mirroring `docker/docker-compose-fed-enable.yml`). It adds the two things the base lacks (confirmed absent at Task 0):
   - **`minio:`** — AGPL-3.0 generic-S3 recording store (D4/D5). `S3_ENDPOINT` is wired to the bridge config via env (e.g. `http://minio:9000`); the override MAY name `minio:9000` (it is config wiring, NOT bridge source — R-S3ENDPOINT only forbids the literal in `services/bridge/src/`). MinIO root creds come from `.env`, never hardcoded.
   - **`tuwunel-b:` + `bridge-b:`** — a SECOND federated instance with a **DISTINCT `MATRIX_SERVER_NAME`/`CONDUIT_SERVER_NAME` domain** (`matrix-b.localhost` vs base instance-A `matrix-a.localhost`), NOT just a distinct port (R-DOMAIN/BUG-15 — see §4). `bridge-b` binds to `tuwunel-b`; federation enabled on BOTH homeservers (`ALLOW_FEDERATION: true` + reachable `server_name` resolution via `extra_hosts`/network alias).
   - The base `tuwunel`/`bridge` represent instance-A; the override sets instance-A's domain to `matrix-a.localhost` and establishes the federation link to instance-B.

2. **`scripts/brehon/e2e-harness-smoke.sh`** — a NEW reach-the-containers smoke (per §10.2). `set -euo pipefail`; brings the stack up (`docker compose -f services/bridge/docker-compose.yml -f services/bridge/docker-compose.e2e.yml --profile rtc up -d`); asserts: (1) MinIO health (`curl -fsS http://localhost:9000/minio/health/live` → 200); (2) LiveKit reachable (`curl -fsS http://localhost:7880`); (3) instance-A bridge liveness; (4) instance-B tuwunel reachable + federates with instance-A on the DISTINCT domain (`curl -fsS http://localhost:<tuwunelB>/_matrix/federation/v1/version` → 200 with `server_name matrix-b.localhost`); prints **`E2E_HARNESS_REACH_OK`** on full success.

3. **IF the base `registration.yaml`/`tuwunel-pilot.toml` do NOT parameterise the domain:** create `services/bridge/registration-b.yaml` + a `tuwunel-b` config snippet, cloned-with-distinct-domain. **First CHECK** whether the base files parameterise the domain (via env) before creating B-variants — prefer env parameterisation (fewer files).

**Do NOT (scope boundaries):**
- Do NOT add any Rust, any `Cargo.toml`/`Cargo.lock` change, any dependency. This task is infra only.
- Do NOT modify the base `services/bridge/docker-compose.yml` (the override is layered, base stays unmutated).
- Do NOT hardcode the S3 endpoint in any `services/bridge/src/**` file (you touch no Rust this task; just don't).
- Do NOT bring the stack up yourself / run the smoke yourself — the LAPTOP advisor runs the reach-smoke (this is a heavy two-instance + MinIO + LiveKit stack; no daemon Docker for it). You author the files + write the validate DQ + STOP.
- Do NOT touch any `crates/**`, migration, const, the registry, or any `.claude/PRPs/plans/` file.

**Branch:** forks from `phase-m3-core-e2e-pilot` (current tip `91257a52b`).

## §3 Required reading

- **`.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` §13 Task 1** (~lines 382–401) — authoritative ACTION/IMPLEMENT/MIRROR/GOTCHA/VALIDATE. Also §10.1 (~165–196, the override compose skeleton) + §10.2 (~198–216, the reach-smoke skeleton).
- **`docker/docker-compose-fed-enable.yml`** — the override-LAYERING precedent (re-creates only changed/added services; do NOT re-declare the whole base).
- **`services/bridge/docker-compose.yml`** — the base: `tuwunel:` (~:19), `livekit:` (~:53, profile `rtc`), `lk-jwt-service:` (~:59), `element-call:` (~:68), `bridge-net:` (~:91). Your override adds to this network.
- **`services/bridge/docker-compose.pilot.yml:12-89`** — the real-Tuwunel + bridge single-instance shape to MIRROR for `tuwunel-b`/`bridge-b` (`MATRIX_SERVER_NAME`, `extra_hosts`, env wiring).
- **`services/bridge/registration.yaml` + `services/bridge/tuwunel-pilot.toml`** — the AS/homeserver config to clone-with-distinct-domain for instance-B IF they don't parameterise the domain.
- **Lessons (binding — non-mechanical, plan-cited):**
  - `feedback_lemmy_federation_domain_collision_one_host.md` (BUG-15) — the 2nd instance needs a DISTINCT hostname/domain, not just a distinct port. THE load-bearing constraint of this task.
  - `feedback_bridge_validates_on_linux_not_windows.md` — bridge runs Linux; you author config, you run NO cargo.
  - `feedback_validate_pending_laptop_write_then_stop.md` — write the validate DQ and STOP; the laptop is the runner.

## §4 Constraints

- **R-DOMAIN / BUG-15 (load-bearing — CATCH-FIRE class if violated):** instance-A and instance-B MUST have DISTINCT domains (`matrix-a.localhost` / `matrix-b.localhost`), NOT a shared host with two ports. **Why it can't be deferred:** Lemmy/Matrix key federated identity by **port-stripped domain** — two instances on the same host:port-stripped name are indistinguishable, so the marquee `mute_all_drops_all_publishers_cross_instance_under_500ms` test (Task 4) cannot tell the two instances apart and the cross-instance path is never actually exercised (silent false-pass). **DoD:** `grep -E 'matrix-a\.localhost|matrix-b\.localhost' services/bridge/docker-compose.e2e.yml` shows BOTH distinct domains; the compose names the federation link.
- **R-S3ENDPOINT (config-not-source):** the override MAY name `minio:9000` (it is the deployment wiring). The constraint forbids a hardcoded `minio.`-shaped literal in `services/bridge/src/**` — which you do not touch this task. Just ensure the bridge gets the endpoint via env (`S3_ENDPOINT`), not baked in.
- **Reach-smoke is the risk-isolation proof:** the smoke MUST prove the harness REACHES every container (MinIO health + LiveKit + bridge-A + tuwunel-B federation) BEFORE any acceptance test depends on it. A failing reach-smoke STOPS the phase (acceptance failures would be unattributable — bootstrap stop-and-ask). The smoke prints `E2E_HARNESS_REACH_OK` only on full success; `set -euo pipefail` so any probe failure aborts.
- **Creds from `.env`, never hardcoded:** MinIO root user/password + any instance-B secrets are read from env / `.env`, never literal in the committed YAML.
- **NO daemon Docker / NO cargo.** Write a `validate-pending-laptop-e2e` DQ with:
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
  Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT bring the stack up or run the smoke yourself (the laptop advisor runs it; this is the heavy two-instance stack). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **ONE commit** — `feat(rtc): e2e acceptance harness — MinIO + 2nd federated instance override + reach-smoke (task 1)`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.
- **Handover:** deliver the HANDOVER block inline in your task output (do NOT write a `.claude/PRPs/handovers/` file — sensitive-file guard).
- End the commit body with a `LESSON:` trailer if you find anything durable (e.g. a tuwunel federation-config gotcha).

## §3a Handover from prior tasks

- Task 0 (advisor-inline, no commit) — freeze baselines confirmed (ENTRY_KIND 72, 2 `bail!` stubs, 2 cr-2/cr-3 scaffold points); bridge baseline + all 4 integration scaffolds compile clean on Linux (`emergency_mute`/`recording`/`room_provisioning`/`stage_mode`); MinIO + 2nd instance confirmed GENUINELY ABSENT in the base compose (Task 1 is net-new); override-layering precedent + instance shape present.
- Phase branch `phase-m3-core-e2e-pilot` cut from `governance-v0` @ `e0141107c`; current tip `91257a52b` (bm-cut runlog commit).

## §5 HANDOVER (worker fills at task end — return inline in task output)

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-task1
  filesCreated: [services/bridge/docker-compose.e2e.yml, scripts/brehon/e2e-harness-smoke.sh]   # +registration-b.yaml/tuwunel-b config IF needed
  filesModified: []
  keyDecisions:
    - "override compose adds minio: + tuwunel-b:/bridge-b: (distinct domain matrix-b.localhost vs matrix-a.localhost — BUG-15)"
    - "S3_ENDPOINT wired via env (config wiring, not bridge source); MinIO creds from .env"
    - "reach-smoke asserts MinIO health + LiveKit + bridge-A + tuwunel-B federation; prints E2E_HARNESS_REACH_OK"
    - "instance-B config: <env-parameterised | needed registration-b.yaml + tuwunel-b snippet>"
  validate_dq: <composite-id of the new validate-pending-laptop-e2e DQ (commands = the reach-smoke)>
  notes: "<confirm two distinct domains in the compose; confirm no Rust/Cargo touched; confirm creds from env not literal>"
```
