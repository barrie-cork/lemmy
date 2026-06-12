---
phase: pilot-internal
plan: "(none — ops/testing track, no plan file)"
phase_branch: "(none — ops work lands on governance-v0 or chore/pilot-internal)"
lane_mode: B
worktree: C:/Users/barri/Developer/brehon-fork
authored: 2026-06-12
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the pilot-internal session. Home-network user testing prep — bring the Brehon Lemmy fork + Matrix bridge to a state where household testers can exercise governance flows.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon pilot-internal** (home-network user testing prep). The M2 milestone is complete: m2-core-hook + m2-rooms-a + m2-late + m2-late-2 all merged. The Lemmy fork is live on the homeserver at `http://100.81.145.58:1236`. The task now is to verify the pilot infrastructure and seed the environment so real users on the home network can exercise governance flows.

**VERIFIED_AT: governance-v0 @ `a1c283686`** (retro commit for m2-late-2, 2026-06-12)

## Session-start ritual

1. `pwd && git branch --show-current && git worktree list` — confirm on `governance-v0` in canonical checkout.
2. `git fetch origin && git rev-parse --short governance-v0` — should match `a1c283686`; log any newer commits.
3. Read `workflow_state_m2_late_2.md` (CLOSED record) once — especially the two carry-forwards.
4. Read `.claude/decision-queue.json` — should be empty.
5. Check pilot containers: `ssh homeserver "cd /srv/brehon-fork/docker && docker compose ps"`.

## Next concrete action

Run the pilot readiness checklist below (§"Pilot readiness checklist"). Start at Step 1 (containers). Surface the full gap list to the user before taking any infrastructure action.

**This is NOT an impl-task sub-phase.** No phase branch, no bm-cut, no CodeRabbit. Work is either:
- Manual ops (SQL, docker compose, environment config) — advisor runs directly
- A `chore/pilot-internal` branch for any tracked config changes

---

## §1. Context — what shipped in M2

| Sub-phase | PR | Merged | Key deliverable |
|---|---|---|---|
| m2-core-hook | #188 | 2026-06-06 | governance_case_after_transition hook → bridge notify |
| m2-rooms-a | #191 | 2026-06-06 | Jury/appeal/emergency room provisioning; 10 Room::* hash-chain kinds |
| m2-late | #192 | 2026-06-09 | B-publish sanction propagation (sanction_event schema + enqueue_sanction_event + bridge ingest endpoint) |
| m2-late-2 | #196 | 2026-06-12 | CR-A atomicity (ADR-008 restored); bridge power-level enforcement per SanctionKind; case_id payload |

The binary at `docker/` was last built from `d12aed221` (extism-fix, pre-M1). **The pilot binary is STALE relative to governance-v0 @ `a1c283686`.** A rebuild is required before testing governance flows.

---

## §2. Pilot readiness checklist (in order)

### Step 1 — Containers up

```bash
ssh homeserver "cd /srv/brehon-fork/docker && docker compose ps"
```

Expected: `lemmy`, `lemmy-ui` (healthy), `postgres` (healthy), `proxy`, `pictrs` — all up. If any are down: `docker compose up -d`.

### Step 2 — Rebuild the Lemmy binary from current governance-v0

The deployed binary is from `d12aed221` (pre-M1/M2 code). M2 added 4 sub-phases of governance code. **Must rebuild.**

```bash
ssh homeserver "cd /srv/brehon-fork && git fetch origin && git checkout governance-v0 && git pull"
# Then rebuild:
ssh homeserver "cd /srv/brehon-fork/docker && docker compose build lemmy && docker compose up -d lemmy"
```

Expected: build succeeds (Docker uses the `rust:1.95` image), containers restart with the new binary.

⚠️ **extism 1.21.0 / wasmtime 41.0.4** — the lockfile intentionally stays at this version (`watch_extism_wasmtime_42_adopt`). Build should succeed. If it doesn't, check `watch_extism_wasmtime_42_adopt` in MEMORY.md before attempting any dep bumps.

### Step 3 — Pilot T6 verification (from m2-late-2 plan §"Task 6")

This is the deferred operator gate from m2-late-2. Manual SQL + reachability probe:

```bash
# Connect to postgres:
ssh homeserver "docker exec -it brehon-fork-postgres-1 psql -U lemmy lemmy"

# Check sanction_subscriber:
SELECT count(*) FROM sanction_subscriber WHERE active = true;
-- EXPECT: 1 (seeded by seed_sanction_subscriber at startup)
-- STOP CONDITION: if 0, the B-publish path has no subscriber — sanction delivery is a no-op

# Check BRIDGE_SANCTION_CALLBACK_URL is set:
# (read from the running lemmy container's env)
ssh homeserver "docker exec brehon-fork-lemmy-1 env | grep BRIDGE"
-- EXPECT: BRIDGE_SANCTION_CALLBACK_URL=http://<bridge-host>:<port>/brehon/sanction-event
-- EXPECT: BRIDGE_CALLBACK_SECRET=<non-empty>
```

Reachability probe (deliberate bad Bearer → expect 401, not connection refused):

```bash
ssh homeserver "curl -s -o /dev/null -w '%{http_code}' -X POST http://localhost:<bridge-port>/brehon/sanction-event -H 'Authorization: Bearer WRONG' -H 'Content-Type: application/json' -d '{}'"
-- EXPECT: 401 (bridge is reachable and auth is wired)
-- FAIL: connection refused (bridge not running), 000 (DNS/network), or 200 (auth not enforced)
```

### Step 4 — Seed test accounts

Create at least 2 non-admin accounts for testing (admin=`lemmy`/`lemmylemmy` already exists):

- One account as a community moderator / Brehon juror candidate
- One account as a regular user (the "defendant" in a test case)

Via the UI at `http://100.81.145.58:1236` (Tailscale) or via the Lemmy API:

```bash
curl -X POST http://100.81.145.58:8536/api/v4/user/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"testmod","password":"testpass123","password_verify":"testpass123","show_nsfw":false}'
```

### Step 5 — Seed a community

Create at least one community where governance events can be triggered:

Via the UI (admin account → Create Community) or API:

```bash
curl -X POST http://100.81.145.58:8536/api/v4/community \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer <admin-jwt>' \
  -d '{"name":"test-governance","title":"Test Governance Community","posting_restricted_to_mods":false}'
```

### Step 6 — Home-network DNS/reachability check

Confirm non-Tailscale devices on the home network can reach the pilot. Options:
- Via homeserver's LAN IP (e.g. `192.168.x.x:1236`) if not using Tailscale
- Via Tailscale IP `100.81.145.58:1236` on devices enrolled in the Tailscale network

This is user-facing: the user knows which devices are involved. Advisor surfaces the IP, user confirms reachability.

### Step 7 — .wasm governance plugins (optional for initial testing)

**No `.wasm` governance plugins exist yet.** This means:
- The DB-layer governance logic (jury selection, vote tallying, sanction events, room provisioning) fires correctly
- The Extism plugin hooks (`docker/plugins/`) do NOT fire — those await a future plugins sub-phase
- Initial testing can proceed without plugins; governance flows work at the DB/API layer

Confirm `docker/plugins/` is empty (expected) and note this in the testing guidance to testers.

---

## §3. Carry-forwards from m2-late-2

- **Verify-before-write for Python DQ mutations** — double-substitution risk on DQ JSON edits via Python one-liners. Use `git show` to read the exact string before replacing.
- **Daemon ref-sync** — use `git branch -f governance-v0 origin/governance-v0` (not `git merge`) when daemon HEAD ≠ target branch.
- **AB trial (MiniMax T3/T4/T5 arms) outstanding** — post-ship serial arms were NOT run. `ab-test/*` branches were NOT cut. MiniMax key rotation pending (user action). Do NOT start AB work until user confirms timing.

---

## §4. Watchlist

| Risk | Severity | Mitigation |
|---|---|---|
| Binary stale from `d12aed221` (pre-M2) | HIGH | Step 2 rebuild mandatory before any governance testing |
| sanction_subscriber = 0 (T6 STOP condition) | MED | Step 3 SQL check; if 0, diagnose `seed_sanction_subscriber` startup logic |
| extism 1.21.0 / wasmtime 41.0.4 lockfile | LOW | Do NOT bump to 1.30.0 (unbuildable); `watch_extism_wasmtime_42_adopt` |
| Bridge port exposure (home network) | MED | Bridge `/brehon/sanction-event` should NOT be directly reachable from untrusted hosts; verify proxy config |
| No .wasm plugins | INFO | Expected; note in testing guidance; governance still works at DB layer |

---

## §5. Key operational paths

- **Pilot UI:** `http://100.81.145.58:1236` (Tailscale IPv4)
- **Lemmy API:** `http://100.81.145.58:8536`
- **Bring stack up (if down):** `ssh homeserver "cd /srv/brehon-fork/docker && docker compose up -d"`
- **View logs:** `ssh homeserver "cd /srv/brehon-fork/docker && docker compose logs --tail=50 lemmy"`
- **Postgres connect:** `ssh homeserver "docker exec -it brehon-fork-postgres-1 psql -U lemmy lemmy"`
- **Rebuild:** `ssh homeserver "cd /srv/brehon-fork/docker && docker compose build lemmy && docker compose up -d lemmy"`

---

## §6. Governance flow for testers (reference)

Once the checklist is complete, the minimal governance flow a tester can exercise:

1. **Create post** as regular-user account in the test community
2. **Flag the post** (report it) as a different account
3. **Admin opens a case**: Lemmy admin console → create ModerationCase on the post
4. **Admin assigns jurors**: jury selection via admin API
5. **Jurors vote** (3/5 quorum): via Lemmy API or admin console
6. **Sanction fires**: on quorum, `enqueue_sanction_event` publishes to the bridge; bridge applies power-level changes in the jury room
7. **Verify Matrix room provisioned**: the case's jury room should exist on the Matrix homeserver (Tuwunel) at `http://100.81.145.58:8448` (or configured port)

⚠️ **governance_case_after_transition hook** (m2-core-hook) fires on CaseStatus transitions. This triggers bridge room provisioning (m2-rooms-a). Verify the Matrix homeserver (Tuwunel) is running and the bridge has a valid `MATRIX_HOMESERVER_URL` before expecting rooms to appear.

---

## §7. Git state at handoff

- `governance-v0` HEAD: `a1c283686` (`docs(retro): m2-late-2 retro`)
- Phase branch: none (m2-late-2 deleted on merge)
- DQ pending: 0
- Open PRs: PR #176 (BUG-1 cherry-pick — possibly superseded), PR #174 (npm Dependabot — triage needed). Neither blocks pilot testing.

---

## §8. What NOT to do

- Do NOT bump `extism` to 1.30.0 (known unbuildable with wasmtime 43)
- Do NOT merge `ab-test/*` branches (AB trial pending; never merges)
- Do NOT start M3 PRD authoring until pilot testing has begun and the user confirms M2 is considered done
- Do NOT run cargo on the EliteDesk daemon (NO-CARGO-ON-ELITEDESK)
