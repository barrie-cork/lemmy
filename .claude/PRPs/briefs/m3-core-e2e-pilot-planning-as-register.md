# Brief — m3-core-e2e-pilot planning: Matrix AS registration for room_provisioning e2e

**Role + dispatch:** `[role:planning] as-register — see .claude/PRPs/briefs/m3-core-e2e-pilot-planning-as-register.md`

Base branch: `governance-v0` (planning briefs commit on trunk; plan finalize-merges to trunk).

## Scope

Author a SMALL implementation plan at
`.claude/PRPs/plans/m3-core-e2e-pilot-as-register.plan.md` that makes the 6 failing
`services/bridge/tests/room_provisioning.rs` e2e tests pass by ensuring the Brehon
bridge Matrix appservice is actually REGISTERED with the Conduit homeserver(s) in the
e2e stack.

Do NOT author implementation code. Produce only the plan file (template:
`.claude/commands/prp-plan.md` / `.claude/PRPs/templates/plan.template.md`).

## The defect (verified on the daemon 2026-06-21 — this is GROUND TRUTH, not hypothesis)

- The bridge provisions Matrix rooms via `services/bridge/src/provision.rs:15`:
  `POST {tuwunel_url}/_matrix/client/v3/createRoom` with `.bearer_auth(config.as_token)`
  (`AS_TOKEN=brehon-as-dev-token-01`).
- Live reproduction against `tuwunel` (`matrixconduit/matrix-conduit:v0.6.0`) returns
  **`401 {"errcode":"M_UNKNOWN_TOKEN","error":"Unknown access token."}`**.
- ROOT CAUSE: the appservice is never registered with Conduit. The compose
  (`services/bridge/docker-compose.yml:41-44`) mounts
  `./registration.yaml:/etc/conduit/registration.yaml:ro` "for reference" and sets
  `CONDUIT_AS_TOKEN`/`CONDUIT_HS_TOKEN` env — but **matrix-conduit v0.6.0 does NOT
  auto-load registrations from that file or those env vars.** Conduit registers
  appservices ONLY at runtime via its admin room (`@conduit:matrix-a.localhost` →
  `appservices register` with the YAML). The mounted file + env are inert.
- This is the 3rd "never-booted-base-placeholder" defect this phase (after the sled
  backend + CONDUIT_PORT 6167-vs-8448 bugs) — the base compose was never exercised
  through a real AS room-create until this phase's e2e.
- Affects BOTH `tuwunel` (base, `registration.yaml`) AND `tuwunel-b` (override,
  `registration-b.yaml` — `docker-compose.e2e.yml:140`).

## Failing tests (6) + the 1 that already passes

FAIL (all need a registered AS so bridge createRoom returns 2xx):
`jury_room_provisions_in_time_with_jurors`, `emergency_remove_provisions_quickly`,
`full_lifecycle_emits_10_chain_entries`, `restart_idempotency_no_duplicate_room_created`,
`messaging_disabled_prevents_provisioning`, `rtc_disabled_townhall_clean_posture`.
PASS (no AS room-create dependency): `anonymous_townhall_identity_never_reaches_livekit`.
(5 sibling `todo!()` stubs remain D2-deferred — out of scope.)

## Required recon — RUN THESE ON THE DAEMON (`ssh homeserver`) before writing the plan

The plan MUST weigh three options. Do NOT pick blindly — establish feasibility first:

1. **Option A — admin-room registration at startup.** FEASIBILITY PROBE (mandatory,
   ~15 min): boot the stack's `tuwunel` alone, register an admin user
   (`CONDUIT_ALLOW_REGISTRATION=true` is already set), find/join the Conduit admin room
   `@conduit:matrix-a.localhost`, send the `appservices register` command with
   `registration.yaml`'s content, then retry the bridge createRoom and confirm it now
   returns 2xx. Record the EXACT command sequence that worked (or why it didn't) in the
   plan. If A works, the impl is an init container / one-shot service (mirror the
   `minio-init` sidecar pattern at `docker-compose.e2e.yml:90-120`) that registers BOTH
   tuwunel-a + tuwunel-b before the bridges start. NOTE: matrix-conduit v0.6.0's admin
   command syntax — verify it; some Conduit versions use a different admin verb.
2. **Option B — switch image to real `tuwunel`.** The real Tuwunel binary DOES auto-load
   `/etc/conduit/registration.yaml`. Recon: find the upstream `tuwunel` image tag, confirm
   it accepts the existing `CONDUIT_*` env (Tuwunel is Conduit-compatible) + the rocksdb
   backend + CONDUIT_PORT 8448 fixes already in the compose. Risk: a new never-booted
   image may surface its OWN config drift (4th placeholder bug) — weigh this.
3. **Option C — re-scope the 6 tests to D2-pilot.** Mark them
   `#[ignore = "requires registered Matrix AS — D2 pilot"]` like the 5 existing `todo!()`
   stubs; validate room provisioning in the real D2 pilot run. Lowest effort, ships the
   phase now; cost = these 6 acceptance assertions move to manual D2 verification.

The plan's §5 complexity + §4 watchpoints must reflect which option is chosen and WHY,
citing the daemon probe evidence.

## Required reading

- `services/bridge/docker-compose.yml` (base tuwunel + the `:41-44` registration mount)
  and `services/bridge/docker-compose.e2e.yml` (tuwunel-b at `:128-140`, the `minio-init`
  sidecar pattern at `:90-120` as the init-container exemplar for option A).
- `services/bridge/registration.yaml` + `registration-b.yaml` (the AS registrations).
- `services/bridge/src/provision.rs` (the createRoom call) +
  `services/bridge/src/room_provisioner.rs:201-310` (townhall provisioning flow + the
  `bridge_room::upsert` the tests assert on).
- `services/bridge/tests/room_provisioning.rs` (the 6 tests + their assertions).
- `.claude/lessons/feedback_lemmy_federation_domain_collision_one_host.md` (BUG-15 —
  tuwunel-a vs tuwunel-b MUST keep distinct server_names; any registration fix must not
  collapse them).
- The two prior placeholder-defect fixes this phase (workflow_state_m3_core_e2e_pilot.md
  Task-1b/1c notes: sled→rocksdb, CONDUIT_PORT 6167→8448) — same defect family, same
  reach-smoke discipline.

## Constraints

- v0/M3 scope: the fix is HARNESS/COMPOSE only (or a test re-scope). NO governance-logic
  Rust changes. If option A/B touches `services/bridge/src/`, STOP and DQ — the bridge
  provisioning code is correct (it's the homeserver that doesn't recognize the AS).
- Gate-4 = LOCAL (decided 2026-06-20): the e2e validation runs on the laptop/daemon stack,
  not GH Actions. The plan's DoD for the impl is a `validate-pending-laptop-e2e` DQ that
  the advisor runs.
- Cycle-3 catch-fire is in effect for the OLD auto-fix loop — this plan is the deliberate
  replanned path, exactly what the user authorized. The plan must NOT propose another
  reactive compose-line guess; it must pick from A/B/C on probe evidence.
- Any clarify-DQ the planner needs → `kind: blocker` from `from: planner` is wrong; planning
  briefs use the advisor's `/brehon-clarify` gate. If the brief is ambiguous, raise a
  `kind: "log"` note; do not block.
- Plan complexity should be SMALL (1-3 tasks). If option C is chosen, the plan is a single
  re-scope task.
