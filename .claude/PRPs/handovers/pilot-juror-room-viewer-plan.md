# Handover — Juror-room viewer (Element Web) + post-wipe Matrix state

**Date:** 2026-06-13
**Author:** advisor (governance-v0 session)
**Session goal:** Make Brehon juror (jury deliberation) Matrix rooms human-viewable
in a browser, after a Matrix `server_name` migration forced a fresh-start volume wipe.
**Status:** Tunnel + fresh Tuwunel CONFIRMED working. Viewer account + Element Web NOT
yet deployed. This doc is the resume brief for that remaining work.

---

## RESUME — next concrete actions (in order)

### 0. Verify nothing regressed (≤2 min)
```bash
# Tuwunel up under the new identity, serving over the public tunnel:
curl -s --max-time 10 https://matrix.agentgrey.app/_matrix/client/versions -o /dev/null -w 'HTTP %{http_code}\n'   # expect 200
ssh homeserver "docker ps --filter name=brehon-tuwunel --filter name=brehon-bridge --format '{{.Names}}\t{{.Status}}'"  # both Up
```

### 1. Create a viewer Matrix account (registration is currently OFF)
`allow_registration = false` in `services/bridge/tuwunel-pilot.toml` (correct for a
public Cloudflare-exposed server — do NOT leave it on). Two ways to make ONE account:

**Option A — temp-flip + restart (simplest, ~30s of open reg):**
```bash
ssh homeserver "cd /srv/brehon-fork/services/bridge && sed -i 's/allow_registration = false/allow_registration = true/' tuwunel-pilot.toml && docker compose -f docker-compose.pilot.yml restart tuwunel"
# wait ~8s, then register over the tunnel (ASCII-only password — see Lessons):
curl -s -X POST https://matrix.agentgrey.app/_matrix/client/v3/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"barry","password":"<pick-a-strong-ascii-pw>","auth":{"type":"m.login.dummy"}}'
# expect: {"user_id":"@barry:matrix.agentgrey.app", ...}
# THEN flip it back OFF and restart:
ssh homeserver "cd /srv/brehon-fork/services/bridge && sed -i 's/allow_registration = true/allow_registration = false/' tuwunel-pilot.toml && docker compose -f docker-compose.pilot.yml restart tuwunel"
```
NOTE: restarting tuwunel here is user-directed and acceptable; bridge is `--no-deps`-safe
to leave running. Do the register BETWEEN the two flips.

**Option B — Tuwunel admin (no open-reg window):** Tuwunel has a `!admin` room /
admin API for creating users, but it needs the admin account/token established at
first boot. Not yet set up on the fresh DB. Option A is faster for the pilot.

⚠ Do NOT paste the password into chat (transcript exposure). Type it server-side or
have the user set it. Per `feedback_no_raw_api_key_through_chat.md`.

### 2. Add Element Web to the bridge compose
Edit `services/bridge/docker-compose.pilot.yml` — add an `element-web` service.
Image `vectorim/element-web`. Mount a `config.json` that defaults the homeserver to
`https://matrix.agentgrey.app`. Expose a host port (1237 is free; 1236 = Lemmy UI).
Then deploy ONLY that service:
```bash
ssh homeserver "cd /srv/brehon-fork/services/bridge && docker compose -f docker-compose.pilot.yml up -d --no-deps element-web"
```
`--no-deps` so tuwunel/bridge are untouched.

Minimal `element-web` config.json content (write alongside the compose, bind-mount to
`/app/config.json`):
```json
{
  "default_server_config": {
    "m.homeserver": { "base_url": "https://matrix.agentgrey.app", "server_name": "matrix.agentgrey.app" }
  },
  "disable_custom_urls": false,
  "brand": "Brehon Pilot"
}
```
`disable_custom_urls:false` lets you override the HS URL manually if discovery misbehaves.

### 3. (Optional but recommended) add `.well-known/matrix/client`
Currently `https://matrix.agentgrey.app/.well-known/matrix/client` → M_NOT_FOUND, so
Element auto-discovery from a bare MXID fails (you must type the HS URL manually). To
make it seamless, serve this JSON at that path (via the Cloudflare side or a tiny
static route): `{"m.homeserver":{"base_url":"https://matrix.agentgrey.app"}}`.
Not a blocker — manual HS-URL entry works without it.

### 4. Verify in browser (Chrome-driven)
- Open `http://192.168.1.157:1237` (or via tunnel if Element is also tunnelled).
- Log in: homeserver `https://matrix.agentgrey.app`, user `barry`, the password from step 1.
- To see a room you first need one to EXIST — see "No rooms exist yet" below. Once a
  jury is seated, join `#jury-case-<N>:matrix.agentgrey.app`.
- Jury rooms are `join_rule:public` + `history_visibility:shared` → join by alias/ID,
  no invite, full back-history readable.

---

## State as of this session (what's TRUE now)

| Thing | State |
|---|---|
| Tuwunel `server_name` | `matrix.agentgrey.app` (was `localhost`) |
| Tuwunel DB | FRESH (wiped + re-init, RocksDB sequence=0) |
| `allow_registration` | **false** (public server; keep off) |
| `allow_federation` | false |
| Public reachability | `https://matrix.agentgrey.app/_matrix/client/versions` → **200, valid TLS** |
| Cloudflare Tunnel | `cloudflared` token-tunnel running on homeserver (config dashboard-side, not on disk) |
| `.well-known/matrix/client` | M_NOT_FOUND (auto-discovery gap; manual HS URL works) |
| Old `:localhost` rooms (27) + puppets | GONE (volume wipe; accepted disposable) |
| Bridge DB | FRESH (`bridge_bridge_pilot_data` also wiped) |
| Lemmy Postgres | UNTOUCHED — cases, posts, jurors, registrations intact |
| Element Web | NOT deployed |
| Viewer Matrix account | NOT created |

### No rooms exist yet
Jury rooms are provisioned by the bridge ONLY when a panel is seated on a live case.
After the wipe there are zero Matrix rooms. To get a viewable `#jury-case-N` room:
run the governance flow to `JurySelection` with a seated panel (seed scripts at
`scripts/brehon/pilot-seed/` — `seed-jurors.sh` for eligibility, then drive a case to
jury seating). The bridge then creates `#jury-case-N:matrix.agentgrey.app`.

---

## Key facts about juror rooms (don't re-derive)

- Juror rooms are **bridge AS-puppet Matrix rooms**. Members are `@_brehon_*` puppet
  users (no passwords, NOT login-able). You cannot "log in as a juror."
- The `juror1-5` / `seed_eligible_jurors` accounts are **Lemmy** accounts (Postgres
  jury-eligibility rows), NOT Matrix accounts.
- To VIEW a room you need a real Matrix account (step 1) + a Matrix client (step 2),
  joining the public room. This is an observer pattern, not impersonation.
- Room aliases: `#<type>-case-<N>` — jury/appeal/emergency/community/spinout/membership.
  AS namespace regex `#.*-case-.*` (in `services/bridge/registration.yaml`).

---

## Lessons surfaced this session (also in PMD eval 996)

- **Matrix `server_name` is identity-bound to its data volume.** Tuwunel refuses to
  reuse a DB created under a different server_name ("Database belongs to X; configured
  server name is Y. Cannot reuse."). Changing server_name REQUIRES a volume wipe —
  no in-place rename.
- **ASCII-only in shell-embedded JSON** for Matrix API bodies. An em-dash in a
  `deny_reason` broke Tuwunel's deserializer ("invalid unicode code point at line 1
  column 101"). Same class as `feedback_powershell_ascii_only_code_strings.md`.
- **Eruda (`LEMMY_UI_ERUDA`) overlay** is invisible to the a11y tree / read_page but
  captured in screenshots (fixed overlay/shadow context); it reserved ~678px at top.
  Disabled this session (commit `a05f6fac4`).
- Registration-applications API returns **`items[]`**, not `registration_applications[]`.

---

## Constraints carried (do NOT violate)

- Do NOT leave `allow_registration = true` on the public server — flip it back after
  creating the viewer account.
- Do NOT paste secrets/passwords/tunnel tokens into chat.
- `--no-deps` for scoped container recreates; the Matrix fresh-start restart was the
  one user-directed exception.
- Do NOT wipe Lemmy Postgres — the pilot governance data lives there.
- `BRIDGE_CALLBACK_SECRET=brehon-bridge-callback-secret-pilot-01` must match bridge + Lemmy.

---

## Files this plan touches
- `services/bridge/docker-compose.pilot.yml` — add `element-web` service (NOT done)
- `services/bridge/element-config.json` — new (NOT done)
- `services/bridge/tuwunel-pilot.toml` — temp `allow_registration` flips (step 1)

## See also
- `.claude/PRPs/handovers/pilot-internal-SHARED-STATE.md` — overall pilot state
- `.claude/PRPs/handovers/pilot-phase8-tester-guide.md` — tester-facing guide
- `scripts/brehon/pilot-seed/` — seed harness (jurors, cases, resilience)
- PMD eval `996` — full session retro
