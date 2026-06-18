# Brief: m3-core-infra fix-impl-5a (Task 5 deploy-smoke corrections)

## §1 Role + dispatch line

`[role:impl-task] m3-core-infra-fix-impl-5a-livekit-cmd-and-elementcall-tag — see .claude/PRPs/briefs/m3-core-infra-fix-impl-5a.md`

## §2 Scope

Two corrections to `services/bridge/docker-compose.yml`, both surfaced by the advisor's
Task 5 deploy-smoke (the gate that proves the compose stack actually boots — it caught two
placeholder values the plan §10.8 stanza had not pull-verified):

1. **Element Call image tag** — `ghcr.io/element-hq/element-call:0.6.0` does NOT exist
   (`manifest unknown` on pull). The real tag is `v0.6.0` (v-prefix). Verified via
   `docker manifest inspect ghcr.io/element-hq/element-call:v0.6.0` (exists) + a clean pull +
   HTTP 200 on the container's port.
2. **LiveKit command** — `command: ["--config", "/etc/livekit.yaml"]` makes the container
   exit immediately (`open /etc/livekit.yaml: no such file or directory`) because no config
   file is mounted. Change to `command: ["--dev"]` — LiveKit's built-in dev mode boots
   standalone with no config file and uses the `devkey`/`devsecret` placeholder credentials,
   which match the `LIVEKIT_KEY: "devkey"` / `LIVEKIT_SECRET: "devsecret"` env the lk-jwt
   stanza already sets. Verified: `docker run … livekit/livekit-server:v1.8 --dev` boots
   cleanly ("starting LiveKit server", port 7880 bound).

**Produces (exactly 1 file edit, one commit):**
- `services/bridge/docker-compose.yml` — 2 line changes (livekit `command:` + element-call `image:`).

**Do NOT touch:** any other file. Do NOT edit `crates/`, `AGPL-NOTICE.md`, `migrations/`,
or any other compose file. The other two RTC images (livekit:v1.8, lk-jwt-service:0.3.0)
are correct — do NOT change them.

**Branch:** forks from `phase-m3-core-infra` (current tip `7fc555b0a`).

## §3 Required reading

- `services/bridge/docker-compose.yml` lines ~53-70 — the `livekit:` and `element-call:`
  stanzas. Read the WHOLE RTC block first to confirm line numbers (the file may have shifted).
- `.claude/PRPs/plans/m3-core-infra.plan.md` §10.8 — the original stanza (note: the `--config`
  flag and `0.6.0` tag there were placeholders; the plan itself says "pin exact tags at impl").

## §4 Constraints

### The two edits (exact)

**Edit 1 — LiveKit command** (line ~56):
```yaml
    command: ["--config", "/etc/livekit.yaml"]
```
→
```yaml
    command: ["--dev"]   # standalone dev mode; uses devkey/devsecret (matches lk-jwt env). No mounted config in v0.
```

**Edit 2 — Element Call image tag** (line ~69):
```yaml
    image: ghcr.io/element-hq/element-call:0.6.0     # AGPL-3.0; pin at impl
```
→
```yaml
    image: ghcr.io/element-hq/element-call:v0.6.0    # AGPL-3.0; pinned (v-prefix) — verified via docker manifest inspect 2026-06-18
```

That is the entire change. Do NOT touch the `profiles: ["rtc"]` lines, the ports, the
networks, the env block, or the AGPL-NOTICE row.

### Anchor uniqueness gate (pre-edit)

1. `grep -c '/etc/livekit.yaml' services/bridge/docker-compose.yml` → must be `1`.
2. `grep -c 'element-call:0.6.0' services/bridge/docker-compose.yml` → must be `1`.

If either > 1, STOP and raise a DQ blocker.

### DoD (verify before committing — read-only greps, no docker)

1. `grep -c '"--dev"' services/bridge/docker-compose.yml` → returns `1`.
2. `grep -c 'element-call:v0.6.0' services/bridge/docker-compose.yml` → returns `1`.
3. `grep -c 'element-call:0.6.0' services/bridge/docker-compose.yml` → returns `0` (old tag gone).
4. `grep -c '/etc/livekit.yaml' services/bridge/docker-compose.yml` → returns `0` (old command gone).

### Commit + stop

One commit: `fix(rtc): correct livekit --dev command + element-call v0.6.0 tag (deploy-smoke fixes, task 5)`.
Push to the worker branch. Then **stop**. This is a non-cargo task — do NOT run docker, cargo,
or write a validate-pending DQ. The advisor re-runs the deploy-smoke (compose boot + liveness)
inline after finalize-merge. End the commit body with a `LESSON:` trailer.
