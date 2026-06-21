# Plan: m3-core-e2e-pilot-as-register — register the bridge Matrix AS so e2e createRoom returns 2xx

## 1. Summary

The m3-core-e2e-pilot acceptance harness boots a Matrix homeserver
(`matrixconduit/matrix-conduit:v0.6.0`) that **never registers the Brehon bridge
appservice**, so every `provision::create_community_room` call (the
`POST /_matrix/client/v3/createRoom` the room provisioner makes with the AS
master token) is rejected `401 M_UNKNOWN_TOKEN`. This is the 3rd
"never-booted-base-placeholder" defect of the phase (after sled→rocksdb and
`CONDUIT_PORT 6167→8448`): the compose mounts `registration.yaml` "for reference"
and sets `CONDUIT_AS_TOKEN`/`CONDUIT_HS_TOKEN`, but **Conduit v0.6.0 loads neither
the mounted file nor those env vars** — both are inert. This sub-phase is a
**harness/compose-only** fix: it switches the e2e Matrix homeservers from Conduit
v0.6.0 to the real **Tuwunel** image (already pinned + proven in
`docker-compose.pilot.yml`), which auto-loads `registration.yaml` from its
`appservice_dir` at startup. **Headline acceptance:** against the Tuwunel-based
stack, `POST createRoom` with the AS bearer token returns **`200 + room_id`**
(verified live at plan time — see §10.3), unblocking the one implemented
createRoom-gated acceptance test `rtc_disabled_townhall_clean_posture`. **No Rust
changes** — the bridge provisioning code is already correct (the same
`create_community_room` returns 200 against Tuwunel and 401 against Conduit).

## 2. Source

- `.claude/PRPs/briefs/m3-core-e2e-pilot-planning-as-register.md` (the authorising brief) @ `f9ece3b39`.
- `.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` (the parent phase plan) §4 (Task 6 intended the `room_provisioning.rs` tests live) + §1 (the override-compose harness with the 2nd federated instance) @ phase-branch tip.
- `services/bridge/docker-compose.pilot.yml` + `services/bridge/tuwunel-pilot.toml` (authored 2026-06-13, the LIVE pilot stack) — the **proven Tuwunel + `appservice_dir` reference config** this plan mirrors.
- `services/bridge/src/provision.rs:10-37` (`create_community_room` — the createRoom seam; **unchanged** by this plan) + `services/bridge/src/room_provisioner.rs:201-247` (`provision_townhall_stage_room` — the townhall flow `rtc_disabled_townhall_clean_posture` exercises).
- `services/bridge/tests/room_provisioning.rs` (phase-branch tip) — the 7 test fns; 2 implemented (`rtc_disabled_townhall_clean_posture`, `anonymous_townhall_identity_never_reaches_livekit`), 5 still `todo!()`.
- Lessons that bind decisions:
  - `feedback_lemmy_federation_domain_collision_one_host.md` (BUG-15) — the 2nd e2e instance MUST keep a DISTINCT domain (`matrix-a.localhost` vs `matrix-b.localhost`); the Conduit→Tuwunel switch must not collapse them to one server_name.
  - `feedback_bridge_validates_on_linux_not_windows.md` — any `services/bridge` cargo runs on Linux via `scripts/brehon/cargo-linux.sh`, never Windows-local. (This plan ships **no cargo** — compose-only — but the lesson governs the §15 DoD shape if a cargo check is ever added.)
  - `feedback_validate_pending_laptop_write_then_stop.md` + `feedback_plan_dod_dry_run_at_write.md` — gate-4 is LOCAL; the impl writes a `validate-pending-laptop-e2e` DQ and STOPs; the advisor is the runner.
  - `feedback_build_what_tests_exercise.md` — the fix is validated by the OBSERVABLE createRoom response (200 + room_id), not by config-shape inspection.
- ADRs (`docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`): **ADR-011** (AGPL — Tuwunel is the same licence posture as the pilot already ships; no new dependency class). No ADR is superseded by this plan.

## 3. Problem statement

`provision::create_community_room` (`provision.rs:10`) authenticates to the
homeserver as the appservice via `.bearer_auth(&config.as_token)`
(`AS_TOKEN=brehon-as-dev-token-01`). For Matrix to accept that token the
homeserver must have **registered the appservice**. The e2e harness homeservers
do not:

- **Base** `services/bridge/docker-compose.yml` `tuwunel:` — `matrixconduit/matrix-conduit:v0.6.0`, mounts `./registration.yaml:/etc/conduit/registration.yaml:ro` + sets `CONDUIT_AS_TOKEN`/`CONDUIT_HS_TOKEN`. **Both inert** — Conduit v0.6.0 only registers an appservice at runtime via its admin room. → `create_community_room` ⇒ `401 M_UNKNOWN_TOKEN` (reproduced live, §10.3).
- **Overlay** `services/bridge/docker-compose.e2e.yml` `tuwunel:` (override) + `tuwunel-b:` — same Conduit image, same inert mount (`registration.yaml` / `registration-b.yaml`). Same 401.

Consequence: the only implemented createRoom-gated acceptance test,
`rtc_disabled_townhall_clean_posture` (`room_provisioning.rs:74`), asserts a
`bridge_room` row with a non-NULL `matrix_room_id`; with createRoom 401 the
provisioner logs `create_community_room failed for townhall` and writes no row →
the test fails (`no townhall bridge_room row`). This is §13 Task 1 + Task 2.

> **Ground-truth note (surfaced as DQ `kind: log`):** the brief lists **6** failing
> createRoom tests. On the phase-branch tip only **`rtc_disabled_townhall_clean_posture`**
> is an implemented createRoom test; the other 5 named tests
> (`jury_room_provisions_in_time_with_jurors`, `emergency_remove_provisions_quickly`,
> `full_lifecycle_emits_10_chain_entries`, `restart_idempotency_no_duplicate_room_created`,
> `messaging_disabled_prevents_provisioning`) are still `todo!()` stubs.
> "Making them pass" requires authoring test bodies — out of this plan's
> harness-only scope; they are D2-deferred (§12). This plan unblocks the AS
> registration the harness needs; implementing the 5 stubs is a separate test-
> authoring task.

## 4. Solution statement

Replace the e2e Matrix homeserver image with the **real Tuwunel** the pilot
already runs, and let Tuwunel auto-load the existing `registration.yaml` /
`registration-b.yaml` from its `appservice_dir`. Tuwunel is **Conduit-compatible**
and honours `TUWUNEL_*` environment variables (verified live, §10.3) — so the
conversion is a near-mechanical, per-service edit with **no new config file**:

For each `tuwunel*` service:
1. **Image** → `ghcr.io/matrix-construct/tuwunel@sha256:1319f9fd40a92a46251e86bd7865facf24d1c974b30731f1bd3059eb446e318f` (the exact digest pinned in `docker-compose.pilot.yml`).
2. **Env** → rename `CONDUIT_*` to `TUWUNEL_*`; **drop** `CONDUIT_DATABASE_BACKEND` (Tuwunel is always RocksDB) and the inert `CONDUIT_AS_TOKEN`/`CONDUIT_HS_TOKEN` (the AS token now comes from the registration file); **add** `TUWUNEL_APPSERVICE_DIR: /etc/tuwunel/appservices` and `TUWUNEL_REGISTRATION_TOKEN` (Tuwunel refuses `allow_registration=true` without a token — §10.3 / §18 R-REGTOKEN); set `TUWUNEL_PORT: "8448"` so the bridge's `TUWUNEL_URL: http://tuwunel*:8448` is unchanged.
3. **Registration mount** → remount the same YAML into the appservice dir: `./registration.yaml:/etc/tuwunel/appservices/registration.yaml:ro` (and `registration-b.yaml` for tuwunel-b). The YAML files themselves are **unchanged**.
4. **DB volume target** → `/var/lib/matrix-conduit` → `/var/lib/tuwunel`.
5. **BUG-15 preserved** → `TUWUNEL_SERVER_NAME` stays `localhost` (base) / `matrix-a.localhost` (overlay tuwunel) / `matrix-b.localhost` (tuwunel-b); the network aliases are untouched.

A reader can predict §11 from this: exactly two files change —
`services/bridge/docker-compose.yml` (base `tuwunel`, Task 1) and
`services/bridge/docker-compose.e2e.yml` (`tuwunel` override + `tuwunel-b`,
Task 2). **No `services/bridge/src/` change, no migration, no registration-YAML
content change.**

## 5. Metadata

- **Phase:** `m3-core-e2e-pilot-as-register` (a harness fix-forward on the open `m3-core-e2e-pilot` phase)
- **Branch:** `phase-m3-core-e2e-pilot` (the existing phase branch — the e2e harness files live only there; impl commits land on it, not a new phase branch)
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 4 (Task 0 pre-flight + 2 impl + retro)
- **Estimated cargo budget:** **N/A — no cargo**. This is compose/config only; validation is a docker-boot + `curl createRoom` harness check (§15) plus the advisor's `validate-pending-laptop-e2e` run of `rtc_disabled_townhall_clean_posture`.
- **Forbidden-window applicability:** binding only for the **local** docker-stack bring-up (Task 1/2 harness check + the gate-4 e2e run); non-binding for daemon impl-task throughput (no daemon cargo).
- **Complexity score:** `1/10` — see breakdown. Well under threshold; proceed-as-one.

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 2 impl tasks (1–2) |
| Migrations touched | +2 each | 0 | none |
| Crates touched | +1 each | 0 | compose/config only — no `crates/`, no `services/bridge/src/` |
| `crates/lemmy_server/tests/e2e/*.rs` edits | +3 each | 0 | none |
| New ADR-affecting decisions | +2 each | 0 | none (Tuwunel already ships in the pilot; same AGPL posture) |
| Cargo budget peak above 6 GB | +1 per GB | 0 | no cargo |
| **Total** | — | **1** | nominal floor for a 2-task compose change. Threshold for split-DQ: `>8` (Sonnet). No split. |

### 5.2 Per-task complexity ceiling (Sonnet target)

Sonnet ceiling = ≤4 files / ≤2 crates per task. Task 1 modifies 1 file
(`docker-compose.yml`); Task 2 modifies 1 file (`docker-compose.e2e.yml`). Both
satisfy the ceiling with margin. No e2e `*.rs` in any `modifies:`.

## 6. Relationship to other m3-core sub-phases

Fix-forward on the open **`m3-core-e2e-pilot`** phase (M3 Phase 6 of 6). It
unblocks the harness that phase's acceptance e2e depends on. The federated
cross-instance emergency-mute test (the phase's marquee) has its **own** brief
(`m3-core-e2e-pilot-impl-emergency-mute-fix.md`) and owns full cross-instance
federation validation — this plan only guarantees AS-registration (createRoom
2xx) on each instance, not Tuwunel↔Tuwunel federation (§12, §18 R-FED).

## 7. Preflight guardrails inherited from prior phases

- **R-REGTOKEN:** Tuwunel **refuses to boot** with `allow_registration=true` and no `registration_token` (a safety guard Conduit lacks). Every Tuwunel service in this plan sets `TUWUNEL_REGISTRATION_TOKEN` (dev-only literal). Verified live (§10.3) — the first probe shut down on exactly this; the second, with a token, booted and returned createRoom 200.
- **R-PORT:** set `TUWUNEL_PORT: "8448"` on every converted service so the bridge env (`TUWUNEL_URL: http://tuwunel*:8448`) and the host port mappings (`8448:8448`, `8449:8448`) are unchanged. Tuwunel's default is 8008 — failing to set this would silently break the bridge→HS connection.
- **R-DOMAIN (BUG-15):** keep the three distinct server_names (`localhost`, `matrix-a.localhost`, `matrix-b.localhost`) and the existing network aliases. Per `feedback_lemmy_federation_domain_collision_one_host.md`.
- **R-NOSRC:** if the fix appears to need a `services/bridge/src/` edit, **STOP and DQ** — the provisioning code is correct (proven: identical code returns 200 vs Tuwunel, 401 vs Conduit). The defect is homeserver-side only. Per the brief's Constraints.
- **R-PIN:** pin the Tuwunel image by the exact `@sha256:` digest from `docker-compose.pilot.yml` (no floating tag), per the phase's image-pin discipline.

## 8. Flow design

```
BEFORE (Conduit v0.6.0 — AS never registered):
  room_provisioner::provision_townhall_stage_room
    └─> provision::create_community_room  (POST /_matrix/client/v3/createRoom, Bearer AS_TOKEN)
          └─> Conduit v0.6.0  ──(mounted registration.yaml + CONDUIT_AS_TOKEN are INERT)──>  401 M_UNKNOWN_TOKEN
                └─> "create_community_room failed for townhall"  →  no bridge_room row  →  TEST FAILS

AFTER (Tuwunel — AS auto-loaded from appservice_dir):
  room_provisioner::provision_townhall_stage_room        [UNCHANGED]
    └─> provision::create_community_room                  [UNCHANGED]
          └─> Tuwunel v1.7.1  ──(appservice_dir loads registration.yaml at startup)──>  200 + room_id
                └─> bridge_room::upsert(townhall, matrix_room_id, ...)  →  row present  →  rtc_disabled_townhall PASSES
```

Boxes that change: only the homeserver container (image + env + mount), defined in
`docker-compose.yml` (Task 1) and `docker-compose.e2e.yml` (Task 2). The bridge
call-graph is untouched.

## 9. Mandatory reading

Files the impl-task subagent MUST Read before its first edit:

- **The proven Tuwunel reference** — `services/bridge/docker-compose.pilot.yml:12-29` (the `tuwunel:` service: image digest, the `appservices/registration.yaml` mount, the volume) + `services/bridge/tuwunel-pilot.toml` (the equivalent config keys: `server_name`, `registration_token`, `appservice_dir`).
- **The two files to edit** — `services/bridge/docker-compose.yml:18-47` (base `tuwunel:`) + `services/bridge/docker-compose.e2e.yml` (the `tuwunel:` override block + the `tuwunel-b:` block — read the whole file; it is the override layer).
- **The registrations (read-only, do NOT edit)** — `services/bridge/registration.yaml` + `services/bridge/registration-b.yaml`.
- **The createRoom seam (read-only, do NOT edit)** — `services/bridge/src/provision.rs:10-37`.
- **The acceptance test** — `services/bridge/tests/room_provisioning.rs:74-210` (`rtc_disabled_townhall_clean_posture` — what the fix must make pass).
- **Lessons** — `feedback_lemmy_federation_domain_collision_one_host.md` (BUG-15, R-DOMAIN); `feedback_validate_pending_laptop_write_then_stop.md` (write the e2e DQ + STOP).

## 10. Patterns to mirror

### 10.1 Tuwunel service definition (the proven pilot shape)

**Mirror:** `services/bridge/docker-compose.pilot.yml:12-21`

```yaml
  tuwunel:
    image: ghcr.io/matrix-construct/tuwunel@sha256:1319f9fd40a92a46251e86bd7865facf24d1c974b30731f1bd3059eb446e318f
    # ... config via mounted TOML in the pilot; this plan uses TUWUNEL_* env instead (§10.2)
    volumes:
      - tuwunel_data:/var/lib/tuwunel
      - ./registration.yaml:/etc/tuwunel/appservices/registration.yaml:ro
    ports:
      - "8448:8448"
```

### 10.2 CONDUIT_* → TUWUNEL_* env mapping (base `tuwunel:`, Task 1)

**Mirror:** the env block at `services/bridge/docker-compose.yml:24-37`, rewritten:

```yaml
    environment:
      TUWUNEL_SERVER_NAME: "localhost"                 # was CONDUIT_SERVER_NAME (BUG-15: unchanged)
      TUWUNEL_DATABASE_PATH: "/var/lib/tuwunel"        # replaces CONDUIT_DATABASE_BACKEND (always rocksdb)
      TUWUNEL_PORT: "8448"                             # R-PORT: keep bridge TUWUNEL_URL valid
      TUWUNEL_ADDRESS: "0.0.0.0"
      TUWUNEL_ALLOW_FEDERATION: "false"                # base = governance-only, federation off
      TUWUNEL_ALLOW_REGISTRATION: "true"
      TUWUNEL_REGISTRATION_TOKEN: "brehon-e2e-reg-token"   # R-REGTOKEN (dev-only — NOT for production)
      TUWUNEL_MAX_REQUEST_SIZE: "20971520"
      TUWUNEL_APPSERVICE_DIR: "/etc/tuwunel/appservices"   # auto-loads registration.yaml at startup
      # CONDUIT_AS_TOKEN / CONDUIT_HS_TOKEN DROPPED — inert; AS token comes from registration.yaml
```

### 10.3 LIVE feasibility evidence (run at plan time, daemon docker 29.3.0)

Decisive probes establishing Option B (Tuwunel) and the defect:

```
# DEFECT (Conduit v0.6.0, registration.yaml mounted, CONDUIT_AS_TOKEN set):
POST /_matrix/client/v3/createRoom  (Bearer brehon-as-dev-token-01)
  → HTTP 401  {"errcode":"M_UNKNOWN_TOKEN","error":"Unknown access token."}

# FIX (Tuwunel via TUWUNEL_* env, TUWUNEL_APPSERVICE_DIR loads registration.yaml):
POST /_matrix/client/v3/createRoom  (Bearer brehon-as-dev-token-01)
  → HTTP 200  {"room_id":"!fmcpJL12hKXtnCBmkJ:localhost"}

# R-REGTOKEN guard (Tuwunel, allow_registration=true, NO registration_token):
  → container SHUTS DOWN: "allow_registration enabled without a token configured ... tuwunel will shut down"
```

Both images are already pulled on the daemon (`ghcr.io/matrix-construct/tuwunel:latest` = digest `1319...`; `matrixconduit/matrix-conduit:v0.6.0`) — no pull cost at impl time.

## 11. Files to change

- `services/bridge/docker-compose.yml` — base `tuwunel:` service: Conduit→Tuwunel (image, `TUWUNEL_*` env, `appservice_dir` mount, `/var/lib/tuwunel` volume). (Task 1)
- `services/bridge/docker-compose.e2e.yml` — `tuwunel:` override block (server_name `matrix-a.localhost`, federation on) + `tuwunel-b:` block (server_name `matrix-b.localhost`, federation on, `registration-b.yaml` mount): same Conduit→Tuwunel conversion. (Task 2)

No public struct fields are added/renamed → the struct-callsite-enumeration rule
(§template) does not apply. No `crates/` path, no migration, no
`services/bridge/src/` path, no registration-YAML content change.

## 12. NOT building in m3-core-e2e-pilot-as-register

- **Implementing the 5 `todo!()` room_provisioning stubs** (`jury_room_*`, `emergency_remove_*`, `full_lifecycle_*`, `restart_idempotency_*`, `messaging_disabled_*`) — deferred to D2 pilot / a separate test-authoring task; reason: writing live-assertion test bodies is out of this plan's harness-only scope (the brief's "6 failing tests" conflates the parent plan's intent with the committed reality — §3 note).
- **Full Tuwunel↔Tuwunel cross-instance federation validation** — deferred to `m3-core-e2e-pilot-impl-emergency-mute-fix` (the marquee federated-mute brief owns it); reason: this plan guarantees AS registration (createRoom 2xx) per instance, which is necessary-but-not-sufficient for federation; federation-over-`.localhost`/plaintext is its own risk (§18 R-FED).
- **Any `services/bridge/src/` edit** — the provisioning code is correct (R-NOSRC); reason: identical code returns 200 vs Tuwunel and 401 vs Conduit, so the defect is homeserver-side only.
- **The Option-A admin-room registration init-container** — rejected; reason: Option B (Tuwunel auto-load) is proven in the pilot + needs zero runtime admin-room interaction (no fragile admin-verb discovery, the very thing that breeds "Nth placeholder bug").

---

## 13. Step-by-step tasks

### Task 0: Pre-flight harness audit + branch verification

**Goal:** confirm the daemon docker stack is ready, the Tuwunel + Conduit images are present, the branch is `phase-m3-core-e2e-pilot`, and the defect+fix baseline holds.

**Probes:**

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — branch is the phase branch (harness files live ONLY here)
test "$(git branch --show-current)" = "phase-m3-core-e2e-pilot" && echo "BRANCH OK" || { echo "WRONG BRANCH"; exit 1; }

# Probe 2 — both images already pulled (no pull cost)
docker images --format '{{.Repository}}' | grep -q 'matrix-construct/tuwunel' && echo "TUWUNEL IMG OK" || echo "TUWUNEL IMG MISSING (pull ghcr.io/matrix-construct/tuwunel)"
docker images --format '{{.Repository}}:{{.Tag}}' | grep -q 'matrixconduit/matrix-conduit:v0.6.0' && echo "CONDUIT IMG OK" || echo "CONDUIT IMG MISSING"

# Probe 3 — the two target files + the two registration YAMLs exist
for f in services/bridge/docker-compose.yml services/bridge/docker-compose.e2e.yml \
         services/bridge/registration.yaml services/bridge/registration-b.yaml; do
  test -f "$f" && echo "FILE OK: $f" || { echo "MISSING: $f"; exit 1; }
done

# Probe 4 (negative) — confirm the defect: Conduit returns 401 (see §10.3 for the exact run)
#   Booting Conduit + curl createRoom Bearer brehon-as-dev-token-01 → EXPECT HTTP 401 M_UNKNOWN_TOKEN
```

**EXPECT block:**
- Probes 0–3 exit 0 / print OK.
- Probe 4 (negative) confirms `401` (defect baseline) — establishes the before-state.

**No commit at Task 0** — verification only.

### Task 1: Convert base `docker-compose.yml` `tuwunel:` to Tuwunel + appservice_dir

**ACTION:** rewrite the base `tuwunel:` service to the real Tuwunel image with `TUWUNEL_*` env + `appservice_dir` auto-load of `registration.yaml`, so createRoom returns 2xx on the governance-only stack.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/docker-compose.yml   # tuwunel: image + TUWUNEL_* env + appservice_dir mount + /var/lib/tuwunel volume
```

**IMPLEMENT (file 1 of 1):** in `services/bridge/docker-compose.yml`, in the `tuwunel:` service:
- `image:` → `ghcr.io/matrix-construct/tuwunel@sha256:1319f9fd40a92a46251e86bd7865facf24d1c974b30731f1bd3059eb446e318f`.
- replace the `environment:` block with the `TUWUNEL_*` block from §10.2 (server_name `localhost`, federation `false`, `TUWUNEL_PORT: "8448"`, `TUWUNEL_REGISTRATION_TOKEN`, `TUWUNEL_APPSERVICE_DIR`); drop `CONDUIT_AS_TOKEN`/`CONDUIT_HS_TOKEN`.
- `volumes:` → `tuwunel_data:/var/lib/tuwunel` + `./registration.yaml:/etc/tuwunel/appservices/registration.yaml:ro`.
- keep `ports: ["8448:8448"]` and the `bridge-net` network membership unchanged.

**MIRROR:** `services/bridge/docker-compose.pilot.yml:12-21` (proven Tuwunel service) + §10.2 (env mapping).

**GOTCHA:** R-REGTOKEN — omitting `TUWUNEL_REGISTRATION_TOKEN` makes Tuwunel shut down at boot (not a runtime error — the container exits). R-PORT — omitting `TUWUNEL_PORT: "8448"` makes Tuwunel listen on 8008 and the bridge's `http://tuwunel:8448` silently fails.

**VALIDATE (story-checkpoint feeds §16a):**

```bash
# Boot ONLY the base tuwunel and prove AS registration via createRoom.
docker compose -f services/bridge/docker-compose.yml up -d tuwunel
# wait for /_matrix/client/versions ready, then:
curl -s -o /tmp/cr.json -w "HTTP_STATUS=%{http_code}\n" \
  -X POST http://localhost:8448/_matrix/client/v3/createRoom \
  -H "Authorization: Bearer brehon-as-dev-token-01" -H "Content-Type: application/json" \
  -d '{"room_alias_name":"jury-case-99999","preset":"public_chat","name":"x"}'
cat /tmp/cr.json
docker compose -f services/bridge/docker-compose.yml down
# EXPECT: HTTP_STATUS=200 and a {"room_id":"!...:localhost"} body
```

### Task 2: Convert `docker-compose.e2e.yml` `tuwunel:` override + `tuwunel-b:` to Tuwunel

**ACTION:** apply the same Conduit→Tuwunel conversion to the overlay's `tuwunel:` override (domain `matrix-a.localhost`, federation on) and the `tuwunel-b:` service (domain `matrix-b.localhost`, federation on, `registration-b.yaml`), preserving BUG-15 distinct domains.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/docker-compose.e2e.yml   # tuwunel override + tuwunel-b: Conduit→Tuwunel, distinct domains preserved
requires:
  - task: 1
    reason: "the overlay tuwunel block OVERRIDES the base tuwunel service from Task 1; the override must be Tuwunel-consistent (TUWUNEL_* env, appservice_dir mount, /var/lib/tuwunel volume) — overriding a Tuwunel base with CONDUIT_* env would be inert."
```

**IMPLEMENT (file 1 of 1):** in `services/bridge/docker-compose.e2e.yml`:
- `tuwunel:` override — `TUWUNEL_SERVER_NAME: "matrix-a.localhost"`, `TUWUNEL_ALLOW_FEDERATION: "true"`, `TUWUNEL_PORT: "8448"`, `TUWUNEL_REGISTRATION_TOKEN`, `TUWUNEL_APPSERVICE_DIR`; drop the `CONDUIT_AS_TOKEN`/`CONDUIT_HS_TOKEN` override; keep the `matrix-a.localhost` network alias. (Inherits image + registration mount + volume from the Task-1 base; if the override block must restate the registration mount, use the `appservices/registration.yaml` path.)
- `tuwunel-b:` — `image:` → the Tuwunel digest; `TUWUNEL_SERVER_NAME: "matrix-b.localhost"`, `TUWUNEL_PORT: "8448"`, `TUWUNEL_ALLOW_FEDERATION: "true"`, `TUWUNEL_ALLOW_REGISTRATION: "true"`, `TUWUNEL_REGISTRATION_TOKEN`, `TUWUNEL_MAX_REQUEST_SIZE`, `TUWUNEL_APPSERVICE_DIR`; drop `CONDUIT_DATABASE_BACKEND`/`CONDUIT_AS_TOKEN`/`CONDUIT_HS_TOKEN`; `volumes:` → `tuwunel_b_data:/var/lib/tuwunel` + `./registration-b.yaml:/etc/tuwunel/appservices/registration.yaml:ro`; keep `ports: ["8449:8448"]` + the `matrix-b.localhost` alias.

**MIRROR:** §10.2 (base mapping) + `docker-compose.pilot.yml:12-21`. For tuwunel-b, the same shape with the `-b` domain/volume/registration.

**GOTCHA:** R-DOMAIN/BUG-15 — `tuwunel` stays `matrix-a.localhost`, `tuwunel-b` stays `matrix-b.localhost`; do NOT collapse to one server_name. R-FED — `TUWUNEL_ALLOW_FEDERATION: "true"` lets each instance register its AS; whether Tuwunel↔Tuwunel federation actually resolves over `.localhost` is a SEPARATE concern owned by the emergency-mute brief (§18 R-FED), not validated here.

**VALIDATE (story-checkpoint feeds §16a):**

```bash
# Bring up the overlay and prove AS registration on BOTH instances independently.
docker compose -f services/bridge/docker-compose.yml -f services/bridge/docker-compose.e2e.yml up -d tuwunel tuwunel-b
# instance-A (host 8448) and instance-B (host 8449):
for p in 8448 8449; do
  curl -s -o /dev/null -w "port ${p}: HTTP_STATUS=%{http_code}\n" \
    -X POST http://localhost:${p}/_matrix/client/v3/createRoom \
    -H "Authorization: Bearer brehon-as-dev-token-01" -H "Content-Type: application/json" \
    -d '{"room_alias_name":"jury-case-99998","preset":"public_chat","name":"x"}'
done
docker compose -f services/bridge/docker-compose.yml -f services/bridge/docker-compose.e2e.yml down
# EXPECT: both ports HTTP_STATUS=200
```

### Task 3: Retro

**Goal:** author retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`. One H2 per role (Advisor / Planning / Impl / BM) with signals + lessons. Capture the "Nth-placeholder-defect" pattern (base compose never exercised through a real AS room-create until e2e) and whether a "boot-the-base-image-once" reach-smoke should be a standing pre-phase guardrail for any new pinned service image. Promote any new lesson to `.claude/lessons/feedback_*.md` in the retro commit.

---

## 14. Testing strategy

- **Harness check (per impl task):** `docker compose ... up <tuwunel service>` + `curl createRoom` Bearer AS_TOKEN → **HTTP 200 + room_id**. This is the decisive, deterministic proof the AS is registered (it IS the defect's inverse).
- **Acceptance (advisor-run, gate-4 LOCAL):** `rtc_disabled_townhall_clean_posture` against the governance-only Tuwunel stack — asserts a `bridge_room` row with non-NULL `matrix_room_id` (the createRoom result) + NULL `chair_id` (R7 rtc-off invariant). Scoped via `validate-pending-laptop-e2e`.
- **No cargo / no clippy / no migration round-trip** — this plan changes only compose YAML; there is nothing to compile.
- **Regression guard:** `anonymous_townhall_identity_never_reaches_livekit` must still pass (it has no Matrix dependency — unaffected by the homeserver swap).

---

## 15. Validation commands (DoD)

> Compose-only plan: the DoD is the docker-boot + curl harness check, NOT cargo. No `--features full`, no clippy, no `cargo-linux.sh` (no `services/bridge` cargo touched).

### 15.1 Task 1 harness check (base stack)

```bash
docker compose -f services/bridge/docker-compose.yml up -d tuwunel
# (poll http://localhost:8448/_matrix/client/versions until ready)
curl -s -o /tmp/t1.json -w "HTTP_STATUS=%{http_code}\n" \
  -X POST http://localhost:8448/_matrix/client/v3/createRoom \
  -H "Authorization: Bearer brehon-as-dev-token-01" -H "Content-Type: application/json" \
  -d '{"room_alias_name":"jury-case-99999","preset":"public_chat","name":"x"}'
docker compose -f services/bridge/docker-compose.yml down
# EXPECT: HTTP_STATUS=200; /tmp/t1.json contains "room_id"
```

### 15.2 Task 2 harness check (overlay, both instances)

```bash
docker compose -f services/bridge/docker-compose.yml -f services/bridge/docker-compose.e2e.yml up -d tuwunel tuwunel-b
for p in 8448 8449; do
  curl -s -o /dev/null -w "port ${p}: HTTP_STATUS=%{http_code}\n" \
    -X POST http://localhost:${p}/_matrix/client/v3/createRoom \
    -H "Authorization: Bearer brehon-as-dev-token-01" -H "Content-Type: application/json" \
    -d '{"room_alias_name":"jury-case-99998","preset":"public_chat","name":"x"}'
done
docker compose -f services/bridge/docker-compose.yml -f services/bridge/docker-compose.e2e.yml down
# EXPECT: both ports HTTP_STATUS=200
```

### 15.3 Acceptance (advisor-run gate-4, `validate-pending-laptop-e2e`)

The impl-task writes a `kind: "validate-pending-laptop-e2e"` DQ entry whose
`commands` run `rtc_disabled_townhall_clean_posture` against the governance-only
Tuwunel stack (LIVEKIT vars UNSET in the runner per the test's R7 gate), then
**STOPs**. The advisor is the runner (per `feedback_validate_pending_laptop_write_then_stop.md`).
- **EXPECT:** `rtc_disabled_townhall_clean_posture` passes (a `bridge_room` row with non-NULL `matrix_room_id`, NULL `chair_id`).

### 15.4 Cross-cutting verification

- [ ] R-NOSRC: `git diff --name-only phase-m3-core-e2e-pilot..HEAD` lists ONLY `services/bridge/docker-compose.yml` + `services/bridge/docker-compose.e2e.yml` (+ the retro file). No `services/bridge/src/`, no `crates/`, no migration, no registration-YAML content change.
- [ ] R-DOMAIN: `grep server_name` shows `localhost` / `matrix-a.localhost` / `matrix-b.localhost` distinct (BUG-15).
- [ ] R-REGTOKEN: every Tuwunel service sets `TUWUNEL_REGISTRATION_TOKEN`.
- [ ] R-PORT: every converted service sets `TUWUNEL_PORT: "8448"`.
- [ ] R-PIN: image pinned by the `@sha256:1319...` digest.

---

## 16. Acceptance criteria

- [ ] Task 0 pre-flight confirmed (docker up, both images present, branch = `phase-m3-core-e2e-pilot`, 401 defect baseline reproduced).
- [ ] Task 1 + Task 2 committed (one commit each).
- [ ] §15.1 + §15.2 harness checks return **HTTP 200** on all three Tuwunel endpoints.
- [ ] §15.3 acceptance: `rtc_disabled_townhall_clean_posture` passes against the Tuwunel stack.
- [ ] §15.4 cross-cutting: all 5 boxes ticked (esp. R-NOSRC — no `src/` change).
- [ ] §16a story `[done]`.
- [ ] No edits outside §11 list.
- [ ] Retro committed (§13 Task 3).
- [ ] PR (if the phase ships via PR) opens against `governance-v0` with `--repo barrie-cork/lemmy` — but note this fix lands on the existing `phase-m3-core-e2e-pilot` branch; merge follows the parent phase's flow.

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: The bridge appservice is registered → createRoom returns 2xx → the rtc-off town-hall acceptance test passes

- **Composing tasks:** Task 1 (base stack — the stack `rtc_disabled_townhall_clean_posture` runs against) + Task 2 (overlay — same fix, both federated instances, foundation for the deferred/federated tests).
- **Checkpoint command:** §15.1 + §15.2 (createRoom → HTTP 200 on ports 8448 + 8449), then §15.3 (`rtc_disabled_townhall_clean_posture` passes).
- **Expected output:** `HTTP_STATUS=200` on all three Tuwunel endpoints; `rtc_disabled_townhall_clean_posture` → `1 passed`.
- **Brief-Scope outputs to verify** (used by `/brehon-verify`):
  - `services/bridge/docker-compose.yml` `tuwunel:` uses the Tuwunel `@sha256:1319...` image + `TUWUNEL_APPSERVICE_DIR` + the `appservices/registration.yaml` mount.
  - `services/bridge/docker-compose.e2e.yml` `tuwunel:`+`tuwunel-b:` use the Tuwunel image with distinct `TUWUNEL_SERVER_NAME` (`matrix-a.localhost` / `matrix-b.localhost`) + the `appservices/registration*.yaml` mounts.
  - No `services/bridge/src/` file appears in the phase diff (R-NOSRC).

> Small phase (2 impl tasks) → a single story whose checkpoint is the createRoom-2xx + acceptance-test-passes behaviour.

---

## 17. Completion checklist

- [ ] Task 0 audit complete.
- [ ] Task 1..2 committed.
- [ ] §15 harness checks green (HTTP 200 ×3).
- [ ] §15.3 acceptance green (`rtc_disabled_townhall_clean_posture`).
- [ ] §16a Story 1 `[done]`.
- [ ] Retro committed.
- [ ] `/brehon-verify` report shows Story 1 ✓.

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| **R-REGTOKEN:** Tuwunel shuts down at boot without `registration_token` | HIGH (default) | HIGH (container exits) | Set `TUWUNEL_REGISTRATION_TOKEN` on every Tuwunel service (§10.2). Verified live (§10.3). |
| **R-PORT:** Tuwunel listens on 8008 (default), bridge `http://tuwunel:8448` fails | MED | HIGH (silent HS-unreachable) | Set `TUWUNEL_PORT: "8448"` on every converted service; §15 harness check curls 8448/8449 explicitly. |
| **R-FED:** Tuwunel↔Tuwunel federation may not resolve over `.localhost`/plaintext | MED | LOW for THIS plan | Out of scope — the federated cross-instance test is the emergency-mute brief's; this plan validates AS-registration (createRoom 2xx) per instance only (§12). If the emergency-mute test later needs federation, that brief addresses well-known/SRV/TLS. |
| **4th-placeholder drift:** the Tuwunel image surfaces its own unexercised-config bug | LOW | MED | Tuwunel is NOT never-booted — `docker-compose.pilot.yml` runs it live since 2026-06-13; the §10.3 probe booted it clean. The one drift (R-REGTOKEN) is already caught + mitigated. |
| Overlay override semantics: restating vs inheriting the base `tuwunel` image/mount | LOW | LOW | Impl reads the whole `docker-compose.e2e.yml` first; if the override must restate the registration mount, use the `appservices/registration.yaml` path (§13 Task 2). §15.2 boots the merged stack to confirm. |

---

## 19. Notes

- **Why Option B over Option A/C (brief's three options):** Option B (switch image to real Tuwunel) is proven in `docker-compose.pilot.yml` + verified live (§10.3: createRoom 200), needs **zero** runtime admin-room interaction, and is near-mechanical (env renames + image + mount). Option A (Conduit admin-room `appservices register` init-container) is fragile — it requires admin-user provisioning, admin-room discovery, and a Conduit-version-specific admin verb, i.e. exactly the kind of unexercised runtime path that breeds the next placeholder bug. Option C (re-scope the tests to D2) is unnecessary for the one implemented test (`rtc_disabled_townhall_clean_posture` becomes green with the harness fix) — though it is effectively what already holds for the 5 `todo!()` stubs (§12).
- **DQ `kind: log` filed** at plan time recording the brief↔reality discrepancy: 5 of the 6 brief-named "failing" tests are `todo!()` stubs, not createRoom-401 failures (§3 note). This plan delivers the AS-registration the harness needs; implementing those 5 stubs is separate D2 test-authoring.
- **Branch nuance:** the plan FILE finalize-merges to `governance-v0` (planning-brief convention); the IMPL tasks commit on `phase-m3-core-e2e-pilot` (the only branch where `docker-compose.e2e.yml` / `registration-b.yaml` exist, and where the sled→rocksdb + port-8448 prior fixes already live).

---

## 20. Confidence score

- **Plan correctness:** 9/10 — the fix is verified end-to-end live (401 vs Conduit, 200 vs Tuwunel) with the exact env the plan specifies; the only residual is the overlay override-restate detail (§18, low impact).
- **Cargo budget:** 10/10 — no cargo.
- **Test coverage:** 8/10 — directly proves the one implemented createRoom-gated test; the 5 `todo!()` stubs + cross-instance federation are explicitly out of scope (§12) and validated elsewhere.
