# Brief: m3-core-infra impl-5 (Task 5)

## §1 Role + dispatch line

`[role:impl-task] m3-core-infra-task5-rtc-sidecars-agpl-rows — see .claude/PRPs/briefs/m3-core-infra-impl-5.md`

## §2 Scope

**Task 5 of plan `.claude/PRPs/plans/m3-core-infra.plan.md`.** Append the three RTC
Docker sidecars (LiveKit, lk-jwt-service, Element Call) to
`services/bridge/docker-compose.yml` under a `profiles: ["rtc"]` gate, and add the
matching AGPL-NOTICE rows.

**Produces (exactly 2 file edits, one commit):**
1. `services/bridge/docker-compose.yml` — three new sidecar stanzas after the existing
   `tuwunel` stanza, each carrying `profiles: ["rtc"]`, on `bridge-net`.
2. `AGPL-NOTICE.md` — a new `## Additional components — M3 RTC stack` section with three
   `### <component>` rows.

**Do NOT touch:**
- `docker/docker-compose.yml` (the Lemmy-only compose — leave untouched).
- Any file under `crates/`, `services/bridge/src/`, `migrations/`, `tests/`.
- `Cargo.toml` / `Cargo.lock`.

**This is a non-cargo task.** There is NO cargo validation and NO `validate-pending-laptop`
DQ for this task. The gate is a deploy-smoke (Docker compose boot) that the **advisor runs
inline** at the §16a Story 4 checkpoint — NOT you. You write the YAML + AGPL rows, commit,
push, and **stop**. Do NOT run `docker compose`, `cargo`, or any validation yourself.

**Branch:** forks from `phase-m3-core-infra` (current tip `71b1d61e2`).

## §3 Required reading

- `.claude/PRPs/plans/m3-core-infra.plan.md` §10.8 (Docker sidecar stanza, profile-gated —
  the exact YAML shape to mirror, lines ~253-280), §10.9 (AGPL-NOTICE row format, ~282-284),
  and Task 5 (lines ~542-578).
- `services/bridge/docker-compose.yml` — read the WHOLE file first. The new stanzas go
  AFTER the `tuwunel` stanza; mirror its `networks: [bridge-net]` membership and indentation
  exactly. Confirm the `bridge-net` network name and the existing port assignments (the
  bridge uses 8080 — lk-jwt host port must avoid it; the plan specifies host 8085).
- `services/bridge/docker-compose.pilot.yml` — the real-image stanza shape + `extra_hosts`
  pattern (reference only; do NOT edit it).
- `AGPL-NOTICE.md` lines ~28-43 — the existing `### services/bridge/` + `### Tuwunel` rows.
  Mirror the three-part "what it is / licence / §13 applicability" shape: Tuwunel is the
  Apache-2.0 pattern; the bridge row is the AGPL-3.0 pattern.

## §4 Constraints

### The two edits (per §10.8 + §10.9)

**File 1 — `services/bridge/docker-compose.yml`** — add three stanzas after the `tuwunel`
service, each with `profiles: ["rtc"]` and `networks: [bridge-net]`:

```yaml
  livekit:
    image: livekit/livekit-server:v1.8   # Apache-2.0; pin exact tag at impl
    profiles: ["rtc"]
    command: ["--config", "/etc/livekit.yaml"]
    ports: ["7880:7880", "7881:7881"]
    networks: [bridge-net]
  lk-jwt-service:
    image: ghcr.io/element-hq/lk-jwt-service:0.3.0   # Apache-2.0; pin at impl
    profiles: ["rtc"]
    environment:
      LIVEKIT_URL: "ws://livekit:7880"
      LIVEKIT_KEY: "devkey"        # dev-only; real key via .env at deploy
      LIVEKIT_SECRET: "devsecret"
    ports: ["8085:8080"]           # host 8085 — avoids the bridge's 8080
    networks: [bridge-net]
  element-call:
    image: ghcr.io/element-hq/element-call:0.6.0     # AGPL-3.0; pin at impl
    profiles: ["rtc"]
    ports: ["8086:8080"]
    networks: [bridge-net]
```

Match the indentation of the existing `tuwunel` stanza exactly (services are nested under
the top-level `services:` key). If the image tags listed above resolve to a non-existent
tag when you sanity-check the registry shape, leave the tag AS WRITTEN — exact-tag pinning
is confirmed by the advisor's deploy-smoke (image pulls), not by you.

**File 2 — `AGPL-NOTICE.md`** — add a new section `## Additional components — M3 RTC stack`
with three `### <component>` rows. Each row follows the existing three-part shape (what it
is / licence / §13 applicability):
- `### LiveKit Server` — SFU media server; **Apache-2.0**; §13 N/A (permissive, not AGPL).
- `### lk-jwt-service` — LiveKit JWT auth helper; **Apache-2.0**; §13 N/A.
- `### Element Call` — WebRTC group-call UI; **AGPL-3.0** (same licence as us — source-
  disclosure obligation honoured via this notice + the public fork); §13 applies.

### ADR-011 tripwire (load-bearing)

Skipping the AGPL-NOTICE rows is an **ADR-011 source-disclosure breach** (every release must
honour the source-disclosure notice for AGPL-licensed components). Element Call is AGPL-3.0 —
its row is mandatory, not optional. **DoD:** `grep -c "Element Call" AGPL-NOTICE.md` returns
≥1 AND the row states the AGPL-3.0 licence. If you find yourself tempted to add the compose
stanzas without the AGPL rows, STOP and raise a DQ blocker — that is the tripwire.

### DoD (verify before committing — read-only greps, no cargo)

1. `grep -c 'profiles: \["rtc"\]' services/bridge/docker-compose.yml` → returns `3`
   (one per RTC sidecar).
2. `grep -c "Element Call" AGPL-NOTICE.md` → returns ≥1.
3. `docker/docker-compose.yml` is unchanged (`git diff --name-only` shows only the two
   target files).

### Commit + stop

One commit: `feat(rtc): add RTC Docker sidecars (profile-gated) + AGPL-NOTICE rows (task 5)`.
Push to the worker branch. Then **stop** — do NOT run docker, cargo, or any validation.
Do NOT write a `validate-pending-laptop` DQ (this task has no cargo gate). End the commit
body with a `LESSON:` trailer if you found anything durable; otherwise just commit.
