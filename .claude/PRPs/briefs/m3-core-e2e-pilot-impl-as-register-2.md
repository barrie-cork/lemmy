# Brief — m3-core-e2e-pilot AS-register impl Task 2: docker-compose.e2e.yml override Conduit→Tuwunel

**Role + dispatch:** `[role:impl-task] as-reg-overlay — see .claude/PRPs/briefs/m3-core-e2e-pilot-impl-as-register-2.md`

Base branch: `phase-m3-core-e2e-pilot` (tip already has Task 1's base Tuwunel conversion). Compose-only, ONE file.

## Scope

Execute **Task 2** of `.claude/PRPs/plans/m3-core-e2e-pilot-as-register.plan.md`:
apply the same Conduit→Tuwunel conversion to the OVERLAY's `tuwunel:` override block
(domain `matrix-a.localhost`, federation ON) AND the `tuwunel-b:` service (domain
`matrix-b.localhost`, federation ON, `registration-b.yaml`) in
`services/bridge/docker-compose.e2e.yml`, preserving BUG-15 distinct domains.

EDIT EXACTLY ONE FILE: `services/bridge/docker-compose.e2e.yml`.
Do NOT touch `docker-compose.yml` (Task 1, already done), `services/bridge/src/`,
the registration YAMLs' content, or any `crates/` path.

Task 1 (base) already converted the base `tuwunel:` to Tuwunel (image
`@sha256:1319f9fd...`, `TUWUNEL_*` env, `appservice_dir`, `/var/lib/tuwunel`). The
overlay OVERRIDES that base, so it MUST be Tuwunel-consistent — a `CONDUIT_*` override
on a Tuwunel base would be inert.

Follow the plan's §13 Task 2 IMPLEMENT block verbatim:
- **`tuwunel:` override** — `TUWUNEL_SERVER_NAME: "matrix-a.localhost"`,
  `TUWUNEL_ALLOW_FEDERATION: "true"`, `TUWUNEL_PORT: "8448"`,
  `TUWUNEL_REGISTRATION_TOKEN: "brehon-e2e-reg-token"`,
  `TUWUNEL_APPSERVICE_DIR: "/etc/tuwunel/appservices"`; DROP the
  `CONDUIT_AS_TOKEN`/`CONDUIT_HS_TOKEN`/`CONDUIT_DATABASE_BACKEND` overrides; keep the
  `matrix-a.localhost` network alias. (Inherits image + registration mount + volume from
  the Task-1 base; if the override block must restate the registration mount, use
  `./registration.yaml:/etc/tuwunel/appservices/registration.yaml:ro`.)
- **`tuwunel-b:`** — `image:` → the Tuwunel `@sha256:1319f9fd...` digest;
  `TUWUNEL_SERVER_NAME: "matrix-b.localhost"`, `TUWUNEL_PORT: "8448"`,
  `TUWUNEL_ALLOW_FEDERATION: "true"`, `TUWUNEL_ALLOW_REGISTRATION: "true"`,
  `TUWUNEL_REGISTRATION_TOKEN: "brehon-e2e-reg-token"`, `TUWUNEL_MAX_REQUEST_SIZE`,
  `TUWUNEL_APPSERVICE_DIR: "/etc/tuwunel/appservices"`; DROP
  `CONDUIT_DATABASE_BACKEND`/`CONDUIT_AS_TOKEN`/`CONDUIT_HS_TOKEN`;
  `volumes:` → `tuwunel_b_data:/var/lib/tuwunel` +
  `./registration-b.yaml:/etc/tuwunel/appservices/registration.yaml:ro`; keep
  `ports: ["8449:8448"]` + the `matrix-b.localhost` alias.
- If a top-level `volumes:` key in this file declared the conduit-b volume, rename/add
  `tuwunel_b_data` so the new mount resolves.

## Required reading (Read BEFORE first edit)

- `.claude/PRPs/plans/m3-core-e2e-pilot-as-register.plan.md` §13 Task 2 (IMPLEMENT + VALIDATE),
  §10.2 (base env mapping to mirror), §7 (R-REGTOKEN/R-PORT/R-DOMAIN/R-PIN guardrails),
  §18 R-FED (federation-over-localhost is OUT of scope — only AS-registration per instance).
- `services/bridge/docker-compose.yml` — the Task-1-converted base `tuwunel:` (read it; the
  override must be consistent with this shape).
- `services/bridge/docker-compose.e2e.yml` — the file you edit; READ THE WHOLE FILE first
  (it is the override layer; find the `tuwunel:` override block + the `tuwunel-b:` block).
- `services/bridge/docker-compose.pilot.yml:12-29` — the proven Tuwunel service shape.
- `services/bridge/registration-b.yaml` (read-only — confirms bridge-b's AS id/domain).
- `.claude/lessons/feedback_lemmy_federation_domain_collision_one_host.md` (BUG-15: tuwunel
  stays `matrix-a.localhost`, tuwunel-b stays `matrix-b.localhost` — do NOT collapse).

## Constraints

- **R-NOSRC:** if the fix appears to need a `services/bridge/src/` edit, STOP + `kind: blocker` DQ.
- **R-REGTOKEN:** every Tuwunel service needs `TUWUNEL_REGISTRATION_TOKEN` or it shuts down at boot.
- **R-PORT:** `TUWUNEL_PORT: "8448"` on every converted service (bridge `http://tuwunel*:8448`).
- **R-DOMAIN (BUG-15):** preserve the 2 distinct server_names + network aliases.
- **R-PIN:** Tuwunel pinned by the exact `@sha256:1319...` digest.
- **Gate-4 = LOCAL.** After the edit: do NOT run docker/cargo yourself. Write a
  `validate-pending-laptop-e2e` DQ (via `bash scripts/brehon/dq-v3-append-fragment.sh`)
  with `phase_task: "as-register-task2"`, `branch: "phase-m3-core-e2e-pilot"`,
  `e2e_filter: null`, `commands` = the plan §15.2 harness check (docker up tuwunel + tuwunel-b
  via both compose files + curl createRoom on :8448 AND :8449 → both expect HTTP 200).
  Commit + push, then **STOP**. The advisor runs the gate.
  Per `feedback_validate_pending_laptop_write_then_stop.md`.
- Commit subject: `chore(e2e): overlay tuwunel + tuwunel-b Conduit→Tuwunel for AS auto-load (as-register task 2)`.
- Commit body ends with: `LESSON: the e2e override block must match the base image family — a CONDUIT_* env override on a Tuwunel base is inert; converted both federated instances to Tuwunel preserving BUG-15 distinct domains.`
- Push to `phase-m3-core-e2e-pilot`.
