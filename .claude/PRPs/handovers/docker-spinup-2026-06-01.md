# Handover — Docker spin-up session (2026-06-01)

**Author:** advisor session (canonical `brehon-fork` / `governance-v0`)
**Purpose:** Next session spins up the server locally and smoke-tests governance endpoints.

---

## ⏩ RESUME — read first

**You are in a new session to spin up and test the Brehon server locally via Docker.**

governance-v0 tip: `9a57f1a63` (chore(docker): add GOVERNANCE_LOG_SIGNING_KEY)

---

## Exact command to run

```bash
cd C:/Users/barri/Developer/brehon-fork/docker
docker compose up --build
```

- **UI:** http://localhost:1236
- **API:** http://localhost:8536
- **Admin login:** `lemmy` / `lemmylemmy`
- **First build:** ~15–30 min (full Rust workspace compiled in Linux container via cargo-chef)
- **Subsequent builds:** fast (deps layer cached)

## What was done this session to enable this

1. `GOVERNANCE_LOG_SIGNING_KEY` added to `docker/docker-compose.yml` (commit `9a57f1a63`) — required for governance log; lazy-loaded on first governance write.
2. `docker/plugins/` directory confirmed present (empty = fine).
3. Postgres: `pgautoupgrade:18.4-alpine` — PG18, fresh volume, migrations run automatically.

## What to watch for on first boot

1. **Migration output** — watch `lemmy` container logs for `Running migration` lines. Expect ~40+ migrations (Lemmy base + Brehon governance). Any `ERROR` here = DB issue.
2. **Setup endpoint** — on very first boot, Lemmy runs the setup (creates admin from `lemmy.hjson`). Should see `Site setup complete` in logs.
3. **Scheduler ticks** — BREHON_DISABLE_* NOT set → reputation snapshot + fed-replay-cleanup + rollup cron all fire on schedule. Expected log noise, not errors.
4. **UI** — navigate to http://localhost:1236, log in as `lemmy`/`lemmylemmy`, verify the site loads and governance endpoints are reachable.

## Governance endpoints to smoke-test manually

All 11 v0 endpoints per `docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md`:
- `POST /api/v1/brehon/report` — create a governance case
- `GET /api/v1/brehon/modlog` — list moderation log
- `GET /api/v1/brehon/reputation/me` — get my reputation
- `POST /api/v1/brehon/endorsement` — create endorsement
- `POST /api/v1/brehon/appeal` — request appeal
- ... (full list in the PRD)

## Concurrent activity

- **v1-quality-r3b** is being driven by the other lane (Issue #167 `admin_audit_stream` DB URL fix). It will NOT affect Docker spin-up — it's a test-isolation fix, not a server behaviour change. Proceed with Docker independently.

## If build fails

- **Rust compile error** — check `docker logs <lemmy-container>`. If it's a Brehon-specific crate, note the error and surface to user.
- **Migration error** — `docker exec -it <postgres-container> psql -U lemmy -d lemmy` to inspect DB state.
- **Port conflict** — compose exposes `1236` and `8536`. Change left-side port in `docker-compose.yml` if conflicts.

---

## Cross-session state

- governance-v0: `9a57f1a63` — clean, no pending DQ
- v1-quality-r3b: other lane, no action needed from this session
- roadmap: updated (all lanes current as of 2026-06-01)
