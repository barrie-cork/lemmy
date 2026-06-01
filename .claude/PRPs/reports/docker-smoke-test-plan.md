# Brehon Docker Smoke Test Plan

**Server:** `http://localhost:8536`  
**UI:** `http://localhost:1236`  
**Admin credentials:** `lemmy` / `lemmylemmy`  
**Date authored:** 2026-06-01  
**Last updated:** 2026-06-01 (rate limit + registration constraints added from live DB)

All requests use `Authorization: Bearer <jwt>` from a login call unless marked public.

---

## Lemmy constraints (live values from this instance)

These are the actual limits running in the Docker stack — not defaults from docs.

| Bucket | Max requests | Window |
|---|---|---|
| `message` (general API) | 180 | 60 s |
| `post` (create post/comment) | 6 | 600 s (10 min) |
| `register` (new account) | 10 | 3600 s (1 hr) |
| `search` | 60 | 600 s |
| `comment` | 6 | 600 s |

**Registration mode: `RequireApplication`** — new accounts need admin approval before their JWT works for write operations. Workaround for testing: use admin account for all reporter actions, OR pre-approve the application via admin UI/API before running governance steps.

**Auth endpoint (v4):** `POST /api/v4/account/auth/login` — NOT `/api/v4/user/login` (that's v3, returns 404).

**Register endpoint (v4):** `POST /api/v4/account/auth/register`

**Community creation:** open (not admin-only) — any approved user can create communities.

### Testing strategy given these constraints

1. **Never run more than ~150 API calls in a 60s window** from one IP — the general bucket (180/60s) is the easiest to hit. Space governance calls with a 1–2s sleep between steps.
2. **Auth bucket is separate and stricter** — login/register calls share a different bucket. If you hit 429 on login, wait the full reset window before retrying (check `x-ratelimit-reset` header).
3. **Post creation is heavily throttled** (6 per 10 min) — create test posts at the start of a session and reuse their IDs. Don't recreate for every test run.
4. **Use admin JWT for all write operations** in single-user testing — avoids the RequireApplication approval dance. Register reporter1 once, approve via admin, then reuse.
5. **Save JWTs across phases** — store in shell variables or `/tmp/` file; re-login only if token expires (~1 week default).

---

## Phase 0 — Boot health

Confirm the stack came up correctly before touching any governance endpoint.

| # | Check | How | Pass condition |
|---|---|---|---|
| 0.1 | All containers running | `docker compose ps` | All 5 services `Up` |
| 0.2 | Migrations ran | `docker compose logs lemmy \| grep -i migration` | ~40+ `Running migration` lines, no `ERROR` |
| 0.3 | Site setup complete | `docker compose logs lemmy \| grep -i "setup"` | `Site setup complete` present |
| 0.4 | API responds | `curl -s http://localhost:8536/api/v4/site \| jq .site_view.site.name` | Returns `"brehon-dev"` (or whatever lemmy.hjson sets) |
| 0.5 | Governance log signing key loaded | `docker compose logs lemmy \| grep -i "signing"` | No `missing signing key` WARN or ERROR |
| 0.6 | Scheduler ticks | `docker compose logs lemmy \| grep -i "reputation\|rollup\|replay"` | Cron lines appear within 5 min (not errors) |

---

## Phase 1 — Auth baseline

Get a JWT for subsequent calls. Create a second user to act as reporter/juror.

**Note:** registration mode is `RequireApplication` — reporter1 needs admin approval before their JWT works for write operations. Admin-approve via `POST /api/v4/admin/registration_application/{id}/approve` or via the UI at http://localhost:1236.

```bash
# Login as admin (correct v4 path)
curl -s -X POST http://localhost:8536/api/v4/account/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username_or_email":"lemmy","password":"lemmylemmy"}' | jq .jwt

# Register a second user (reporter) — answer field required by RequireApplication mode
curl -s -X POST http://localhost:8536/api/v4/account/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"reporter1","password":"Password123!","password_verify":"Password123!","show_nsfw":false,"answer":"smoke test"}' | jq .jwt

# Admin: approve reporter1's application (get application id first)
curl -s http://localhost:8536/api/v4/admin/registration_application/list \
  -H "Authorization: Bearer $ADMIN_JWT" | jq '.registration_applications[0].registration_application.id'
# Then approve:
curl -s -X PUT "http://localhost:8536/api/v4/admin/registration_application/$APP_ID/approve" \
  -H "Authorization: Bearer $ADMIN_JWT" -H "Content-Type: application/json" -d '{}'
```

| # | Check | Pass condition |
|---|---|---|
| 1.1 | Admin JWT issued | Non-null string returned |
| 1.2 | reporter1 registered | 200 (JWT may be null until approved — that's expected) |
| 1.3 | reporter1 application approved | Admin approval API returns 200 |
| 1.4 | reporter1 JWT issued post-approval | Login returns non-null JWT |
| 1.5 | GET /api/v4/site with admin JWT | `my_user.local_user_view.person.name == "lemmy"` |

---

## Phase 2 — Reputation baseline

Confirm the reputation snapshot table is seeded before any governance action.

```bash
# GET /api/v4/governance/reputation/me  (as admin)
curl -s http://localhost:8536/api/v4/governance/reputation/me \
  -H "Authorization: Bearer $ADMIN_JWT" | jq .
```

| # | Check | Pass condition |
|---|---|---|
| 2.1 | Endpoint returns 200 | No 404 / 500 |
| 2.2 | Response has reputation fields | `score` and `flags` fields present |
| 2.3 | Modlog readable (public) | `GET /api/v4/governance/modlog` → 200, empty `entries` array |

---

## Phase 3 — Core case lifecycle (happy path)

This is the main integration flow. Run steps in order; each step depends on the previous.

### 3.1 Create a post to report

**Rate limit:** post creation is throttled at 6 per 10 min. Create this post once and reuse the ID across test runs — don't recreate it each time.

```bash
# Use the auto-created "main" community (id=2 on fresh instance)
# Use ADMIN_JWT here to avoid reporter1 approval dependency
curl -s -X POST http://localhost:8536/api/v4/post \
  -H "Authorization: Bearer $ADMIN_JWT" \
  -H "Content-Type: application/json" \
  -d '{"name":"Test post for governance","community_id":2,"nsfw":false}' | jq .post_view.post.id
# Save as POST_ID — reuse this ID for the full session
```

### 3.2 Create a governance report

```bash
curl -s -X POST http://localhost:8536/api/v4/governance/report \
  -H "Authorization: Bearer $REPORTER_JWT" \
  -H "Content-Type: application/json" \
  -d '{
    "target_type": "Post",
    "target_id": '$POST_ID',
    "reason_code": "spam",
    "description": "smoke test report"
  }' | jq .
# Save case_id from response
```

| # | Check | Pass condition |
|---|---|---|
| 3.2a | Returns 200 | No 4xx/5xx |
| 3.2b | Case created | `case.status == "Open"` |
| 3.2c | `report_created` in governance log | `GET /api/v4/governance/modlog` → entry with `entry_kind: "report_created"` |
| 3.2d | Governance log hash chain valid | `prev_hash` on first entry is zeros; `entry_hash` is non-null |

### 3.3 Retrieve the case

```bash
curl -s "http://localhost:8536/api/v4/governance/case?case_id=$CASE_ID" \
  -H "Authorization: Bearer $ADMIN_JWT" | jq .
```

| # | Check | Pass condition |
|---|---|---|
| 3.3a | 200 returned | - |
| 3.3b | Correct case | `case.id == $CASE_ID`, `case.target_type == "Post"` |

### 3.4 Admin: assign jury (backstop — no automatic threshold in fresh instance)

```bash
curl -s -X POST http://localhost:8536/api/v4/governance/admin/assign-jury \
  -H "Authorization: Bearer $ADMIN_JWT" \
  -H "Content-Type: application/json" \
  -d '{"case_id": '$CASE_ID'}' | jq .
```

| # | Check | Pass condition |
|---|---|---|
| 3.4a | 200 returned | - |
| 3.4b | Case status advanced | `GET case` → `status` ∈ `{JurySelection, InReview}` |
| 3.4c | `jury_assigned` log entry present | modlog shows `entry_kind: "jury_assigned"` |

### 3.5 Check jury queue

```bash
curl -s http://localhost:8536/api/v4/governance/jury/me \
  -H "Authorization: Bearer $ADMIN_JWT" | jq .
```

| # | Check | Pass condition |
|---|---|---|
| 3.5a | Returns 200 | - |
| 3.5b | Case appears in queue OR empty (only 1 user) | No 5xx |

### 3.6 Submit jury votes → reach decision

On a fresh single-user instance the admin is the only eligible juror. Submit votes from the admin account; quorum = 3, panel = 5.

```bash
# Vote 1 — Warning
curl -s -X POST http://localhost:8536/api/v4/governance/jury/vote \
  -H "Authorization: Bearer $ADMIN_JWT" \
  -H "Content-Type: application/json" \
  -d '{"case_id":'$CASE_ID',"decision":"Warning","rationale":"smoke test vote 1"}'

# (Repeat for votes 2 and 3 — quorum of 3 triggers decision)
```

| # | Check | Pass condition |
|---|---|---|
| 3.6a | Each vote returns 200 | - |
| 3.6b | After 3rd vote: case status = `Decided` | `GET case → status == "Decided"` |
| 3.6c | Sanction created | Response or case contains sanction record |
| 3.6d | `case_decided` log entry | modlog shows `entry_kind: "case_decided"` |
| 3.6e | `sanction_created` log entry | modlog shows `entry_kind: "sanction_created"` |
| 3.6f | `reputation_delta` log entry | modlog shows reporter/target reputation change |

### 3.7 List cases

```bash
curl -s "http://localhost:8536/api/v4/governance/cases?status=Decided" \
  -H "Authorization: Bearer $ADMIN_JWT" | jq '.cases | length'
```

| # | Check | Pass condition |
|---|---|---|
| 3.7a | Returns 200 | - |
| 3.7b | At least 1 decided case | count ≥ 1 |

---

## Phase 4 — Appeal flow

Continues from Phase 3 (case in `Decided` state).

```bash
curl -s -X POST http://localhost:8536/api/v4/governance/appeal \
  -H "Authorization: Bearer $REPORTER_JWT" \
  -H "Content-Type: application/json" \
  -d '{"case_id":'$CASE_ID',"reason":"I dispute this decision — smoke test appeal"}' | jq .
```

| # | Check | Pass condition |
|---|---|---|
| 4.1 | Returns 200 | - |
| 4.2 | Case status = `Appealed` | `GET case → status == "Appealed"` |
| 4.3 | `appeal_requested` log entry | modlog entry present |
| 4.4 | Admin can close appeal | `POST /admin/close-case` → 200, status → `Closed` |

---

## Phase 5 — Endorsement

Test the endorsement/reputation signal path independently of the case flow.

```bash
# Admin endorses reporter1
curl -s -X POST http://localhost:8536/api/v4/governance/endorsement \
  -H "Authorization: Bearer $ADMIN_JWT" \
  -H "Content-Type: application/json" \
  -d '{"person_id":'$REPORTER_PERSON_ID'}' | jq .
```

| # | Check | Pass condition |
|---|---|---|
| 5.1 | Returns 200 | - |
| 5.2 | `endorsement_created` log entry | modlog shows entry |
| 5.3 | reporter1 reputation/me shows endorsement effect | `score` non-zero or `flags` updated |
| 5.4 | Cannot double-endorse | Second identical POST → 4xx (idempotency guard) |

---

## Phase 6 — Governance log integrity

Verify the hash chain is self-consistent after all the above writes.

```bash
# Fetch all modlog entries
curl -s "http://localhost:8536/api/v4/governance/modlog?limit=50" \
  -H "Authorization: Bearer $ADMIN_JWT" | jq '.entries | length'
```

| # | Check | Pass condition |
|---|---|---|
| 6.1 | Entry count ≥ expected | Should have at minimum: report_created, jury_assigned, jury_voted ×3, case_decided, sanction_created, reputation_delta ×N, appeal_requested, endorsement_created |
| 6.2 | Each entry has non-null `entry_hash` | `jq '[.entries[].entry_hash] | all(. != null)'` → `true` |
| 6.3 | Each entry has `actor_pseudonym` (GDPR) | `jq '[.entries[].actor_pseudonym] | all(. != null)'` → `true` |
| 6.4 | No PII in payloads | Spot-check `entries[].payload` — no `email`, `password`, `ip_address` fields |
| 6.5 | Entries are append-only | Count before + after a read-only call = unchanged |

---

## Phase 7 — Admin endpoints

Quick coverage pass on the admin backstop surface.

| # | Endpoint | Check |
|---|---|---|
| 7.1 | `GET /admin/reputation-stats` | 200, non-empty stats object |
| 7.2 | `GET /admin/dashboard` | 200, counts present |
| 7.3 | `GET /admin/config` | 200, config object with governance params |
| 7.4 | `POST /admin/config` (no-op write same values) | 200, `config_audit` entry in log |
| 7.5 | `GET /admin/config/audit` | 200, audit trail entry from 7.4 |

---

## Phase 8 — Federation (needs remote instance)

**Prerequisite:** a second vanilla Lemmy instance reachable from this host.  
**Scope:** outbound-only (v0 ADR-014). Brehon governance signals are fork-only AP types — the remote won't parse them, but it must not error or block federation.

| # | Check | Pass condition |
|---|---|---|
| 8.1 | Federate a post from remote → visible locally | Standard Lemmy federation works |
| 8.2 | Governance case created on a federated post | `report` → case created, no federation error in logs |
| 8.3 | `LEMMY_DISABLE_ACTIVITY_SENDING=true` override | Outbound AP activities suppressed in dev compose — confirm no unexpected federation noise in logs |
| 8.4 | *(v1 scope)* Governance AP activity published on case_decided | Not tested in v0 — log entry exists, AP publish deferred |

---

## Phase 9 — Passkey MFA (needs second device or browser profile)

**Prerequisite:** a second machine/browser that can respond to a WebAuthn ceremony.

| # | Check | Pass condition |
|---|---|---|
| 9.1 | Register a passkey on admin account via UI | WebAuthn registration ceremony completes, key stored |
| 9.2 | Log out, log back in with passkey | WebAuthn assertion ceremony succeeds, JWT issued |
| 9.3 | Governance endpoint still works post-passkey login | `GET /reputation/me` → 200 |
| 9.4 | Wrong device / cancelled ceremony → rejected | 401 returned, no session created |

---

## Deferred (out of v0 scope)

- Extism plugin hooks (plugins/ empty in dev compose)
- Supermajority voting rules (v1)
- Reputation decay curve (stub only in v0)
- Sponsor liability flow (v1-SL)
- Cross-instance governance signal parsing on remote (fork-only AP type)
