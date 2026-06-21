# Brief — m3-core-e2e-pilot AS-register impl Task 1: base docker-compose.yml Conduit→Tuwunel

**Role + dispatch:** `[role:impl-task] as-reg-base — see .claude/PRPs/briefs/m3-core-e2e-pilot-impl-as-register-1.md`

Base branch: `phase-m3-core-e2e-pilot`. Compose-only, ONE file.

## Scope

Execute **Task 1** of `.claude/PRPs/plans/m3-core-e2e-pilot-as-register.plan.md`:
convert the base `tuwunel:` service in `services/bridge/docker-compose.yml` from
`matrixconduit/matrix-conduit:v0.6.0` to the real Tuwunel image so the appservice
auto-loads from `appservice_dir` and `createRoom` returns 2xx.

EDIT EXACTLY ONE FILE: `services/bridge/docker-compose.yml` (the `tuwunel:` service).
Do NOT touch `docker-compose.e2e.yml` (that's Task 2), `services/bridge/src/`,
`registration.yaml` content, or any `crates/` path.

Follow the plan's §13 Task 1 IMPLEMENT block verbatim:
- `image:` → `ghcr.io/matrix-construct/tuwunel@sha256:1319f9fd40a92a46251e86bd7865facf24d1c974b30731f1bd3059eb446e318f`
- replace the `environment:` block with the `TUWUNEL_*` block from plan §10.2
  (server_name `localhost`, federation `false`, `TUWUNEL_PORT: "8448"`,
  `TUWUNEL_REGISTRATION_TOKEN: "brehon-e2e-reg-token"`,
  `TUWUNEL_APPSERVICE_DIR: "/etc/tuwunel/appservices"`); DROP
  `CONDUIT_AS_TOKEN`/`CONDUIT_HS_TOKEN`/`CONDUIT_DATABASE_BACKEND`.
- `volumes:` → `tuwunel_data:/var/lib/tuwunel` +
  `./registration.yaml:/etc/tuwunel/appservices/registration.yaml:ro`
- keep `ports: ["8448:8448"]` + `bridge-net` membership unchanged.
- if a top-level `volumes:` key declared `tuwunel_data` (or the conduit volume name),
  rename/keep it so the new mount resolves.

## Required reading (Read BEFORE first edit)

- `.claude/PRPs/plans/m3-core-e2e-pilot-as-register.plan.md` §10.2 (env mapping verbatim),
  §13 Task 1 (IMPLEMENT + VALIDATE), §7 (R-REGTOKEN, R-PORT, R-DOMAIN, R-PIN guardrails).
- `services/bridge/docker-compose.pilot.yml:12-29` — the PROVEN Tuwunel service shape to mirror.
- `services/bridge/docker-compose.yml` (the file you edit — read the whole `tuwunel:` block + any top-level `volumes:`).
- `services/bridge/registration.yaml` (read-only — confirms the AS id/tokens the appservice_dir loads).
- `.claude/lessons/feedback_lemmy_federation_domain_collision_one_host.md` (BUG-15: base server_name stays `localhost`).

## Constraints

- **R-NOSRC:** if the fix appears to need a `services/bridge/src/` edit, STOP and write a
  `kind: blocker` DQ — the provisioning code is correct (proven: 200 vs Tuwunel, 401 vs
  Conduit). The defect is homeserver-side only.
- **R-REGTOKEN:** Tuwunel SHUTS DOWN at boot if `allow_registration=true` without
  `TUWUNEL_REGISTRATION_TOKEN`. The token line is mandatory (verified live, plan §10.3).
- **R-PORT:** `TUWUNEL_PORT: "8448"` is mandatory — Tuwunel defaults to 8008 and the bridge's
  `http://tuwunel:8448` would silently fail.
- **R-PIN:** image pinned by the exact `@sha256:1319...` digest — no floating tag.
- **Gate-4 = LOCAL.** After the edit: do NOT run docker/cargo yourself. Write a
  `validate-pending-laptop-e2e` DQ entry (use `bash scripts/brehon/dq-v3-append-fragment.sh`)
  with `phase_task: "as-register-task1"`, `branch: "phase-m3-core-e2e-pilot"`,
  `e2e_filter: null`, and `commands` = the plan §15.1 harness check (docker up base tuwunel +
  curl createRoom → expect HTTP 200). Commit + push, then **STOP**. The advisor runs the gate.
  Per `feedback_validate_pending_laptop_write_then_stop.md`.
- Commit subject: `chore(e2e): base docker-compose tuwunel Conduit→Tuwunel for AS auto-load (as-register task 1)`.
- Commit body ends with: `LESSON: matrix-conduit:v0.6.0 never registers a mounted appservice (admin-room only); switching to the pilot's real Tuwunel image makes appservice_dir auto-load registration.yaml at boot → createRoom 200.`
- Push to `phase-m3-core-e2e-pilot`.
