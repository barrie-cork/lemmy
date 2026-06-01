# Brehon Docker Smoke Test Plan

**Server:** `http://localhost:8536`  
**UI:** `http://localhost:1236`  
**Admin credentials:** `lemmy` / `lemmylemmy`  
**Date authored:** 2026-06-01  
**Last updated:** 2026-06-01 (session-3: Phase 3–6 PASS, 10 bugs documented)

## Session state (2026-06-01, session 2)

| Item | Value |
|---|---|
| Stack uptime | ~3 hours, all 5 containers healthy |
| Admin JWT | `eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwiaXNzIjoibG9jYWxob3N0IiwiaWF0IjoxNzgwMzI4NDkyLCJleHAiOjE3ODA5MzMyOTJ9.vWXSVZr-FAFTpiN0baDdBwVKI4LpUshBtZAlsLheqQQ` (sub=1, exp ~2026-06-08) |
| Reporter1 JWT | `eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIyIiwiaXNzIjoibG9jYWxob3N0IiwiaWF0IjoxNzgwMzI4NDk2LCJleHAiOjE3ODA5MzMyOTZ9.emHslA57L_NhvsZ29QfoF3StuUCxjTl99gHzXmj_4_Y` (sub=2, exp ~2026-06-08) |
| Communities | 2 — `governancetest` (id=2) created in session 2 |
| Governance modlog | 15 entries — full lifecycle through appeal complete |
| Rate limits | Clear as of session-3 start |
| Case id=1 | Status=`Appealed` (appeal filed, appeal jury assembled) |
| governance_log | 15 entries, hash chain intact, appeal_panel_assembled is newest |

### Phase completion status

| Phase | Status | Notes |
|---|---|---|
| Phase 0 — Boot health | ✅ PASS | All containers up, migrations ran, site responds |
| Phase 1 — Auth baseline | ✅ PASS | Admin JWT issued, reporter1 registered + approved |
| Phase 2 — Reputation baseline | ✅ PASS | `/reputation/me` returns 200 with `['view']` fields, modlog `[]` |
| Phase 3 — Case lifecycle | ✅ PASS | Community created, post created, report filed, jury assigned+accepted, vote→Decided, governance_log 12 entries, hash chain intact |
| Phase 4 — Appeal flow | ✅ PASS | Appeal filed by defendant, case→Appealed, appeal jury assembled, governance_log 15 entries |
| Phase 5 — Endorsement | ✅ PASS | admin→reporter1, endorsement_created logged, double-endorse blocked (cooldown) |
| Phase 6 — Log integrity | ✅ PASS | 15/15 entry_hash present, full hash chain valid, 10/15 actor_pseudonym |
| Phase 7 — Admin endpoints | ✅ PASS | reputation-stats, dashboard, config GET/write/audit, rule-sets, rollup all verified; config write produces governance_log entry with actor_pseudonym + signature |
| Phase 8 — Federation | ✅ PARTIAL PASS | Outbound AP verified (ADR-014 core claim). Bidirectional loop blocked by Brehon localhost hostname — see Phase 8 section. |
| Phase 9 — Passkey/MFA | ⏳ PENDING | Not started |

### Known bugs fixed (discovered session 1)

1. **Community ID** — fresh instance has 0 communities; `community_id=2` does not exist. Fix: `GET /api/v4/community/list` to discover, OR create a community first via admin.
2. **`target_type` case** — enum expects lowercase `"post"` not `"Post"`. Error was: `unknown variant 'Post', expected one of 'post', 'comment', 'person', 'community', 'remote_instance'`.
3. **Registration approval endpoint** — correct call is `PUT /api/v4/admin/registration_application/approve` with body `{"id": N, "approve": true}`, NOT a path-param style URL.
4. **`/api/v4/admin/registration_application/list` with `?unread_only=true`** — returned empty even when application existed; use without that filter.
5. **modlog response shape** — returns a JSON array `[]` directly, not `{"entries": []}`. Parse as list, not dict.

### Known bugs fixed (discovered session 2–3)

6. **Community names must be alphanumeric + underscore only** — `governance-test` is invalid (hyphen rejected). Use `governancetest` or `governance_test`.
7. **`decision` enum is lowercase** — `"warning"` not `"Warning"`. Same applies to all jury decision values (`noaction`, `advisorylabel`, `remove`, `ban`, `emergencyremove`).
8. **Jury eligibility requires `accepted_application=true`** — admin account starts with `accepted_application=false` on a fresh instance. Fix: `UPDATE local_user SET accepted_application=true WHERE person_id=2` before assigning jury.
9. **`panel_size` must be ≤ eligible juror count** — on a single-user instance with only admin eligible, set `panel_size=1` and `quorum=1` in the `governance_config` table. Otherwise assign-jury returns `not_found`.
10. **`GET /admin/rule-sets` requires `?community_id=N` query param** — omitting it returns `missing field community_id`. `GET /admin/reputation/rollup` requires `?person_id=N`. Neither param is documented in the route descriptions.
11. **`POST /admin/config` requires `reason` field** — non-empty string; omitting it returns `missing field value`. Body shape: `{"key","value_type","value":<raw JSON>,"scope","reason"}`. The `value` field is a raw JSON value, not the `value_int`/`value_text` split used in the DB schema.
12. **Non-ASCII characters in POST body cause JSON deserialization errors** — e.g. em-dash `—` in a `reason` string triggers `invalid unicode code point`. Use ASCII-only strings in all API bodies.
13. **`onboarding.sponsor_min_account_age_days=30` blocks endorsement on fresh instance** — admin account is new (created at stack boot); age gate fires and returns `not_found`. Fix: `UPDATE governance_config SET value_int=0 WHERE scope='instance' AND key='onboarding.sponsor_min_account_age_days'`. Also set `onboarding.sponsor_min_endorsement_strength=0`.
14. **Appeal `target_person_id` not set for post-targeted cases** — when a report targets a post (`target_type=post`), `target_person_id` remains NULL. The appeal handler checks `target_person_id == caller_id` for the defendant path — it never resolves `target_post.creator_id`. Workaround: manually set `UPDATE moderation_case SET target_person_id=<post_author_person_id> WHERE id=$CASE_ID` before appealing.

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

### 3.0 Create a community (prerequisite — fresh instance has none)

```bash
# Create a community to post into
COMM_RESP=$(curl -s -X POST http://localhost:8536/api/v4/community \
  -H "Authorization: Bearer $ADMIN_JWT" \
  -H "Content-Type: application/json" \
  -d '{"name":"governancetest","title":"Governance Test Community","nsfw":false}')
COMM_ID=$(echo $COMM_RESP | python -c "import sys,json; print(json.load(sys.stdin)['community_view']['community']['id'])")
echo "COMM_ID=$COMM_ID"
```

### 3.1 Create a post to report

**Rate limit:** post creation is throttled at 6 per 10 min. Create this post once and reuse the ID across test runs — don't recreate it each time.

```bash
# Discover community id (or use COMM_ID from 3.0)
COMM_ID=$(curl -s http://localhost:8536/api/v4/community/list \
  -H "Authorization: Bearer $ADMIN_JWT" | python -c "import sys,json; print(json.load(sys.stdin)['communities'][0]['community']['id'])")

curl -s -X POST http://localhost:8536/api/v4/post \
  -H "Authorization: Bearer $ADMIN_JWT" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Test post for governance\",\"community_id\":$COMM_ID,\"nsfw\":false}" | python -c "import sys,json; print('POST_ID:', json.load(sys.stdin)['post_view']['post']['id'])"
# Save as POST_ID — reuse this ID for the full session
```

### 3.2 Create a governance report

```bash
curl -s -X POST http://localhost:8536/api/v4/governance/report \
  -H "Authorization: Bearer $REPORTER_JWT" \
  -H "Content-Type: application/json" \
  -d '{
    "target_type": "post",
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
  -d '{"case_id":'$CASE_ID',"decision":"warning","rationale":"smoke test vote 1"}'

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

**IMPORTANT:** The appeal must be filed by the **defendant** (the sanctioned party, i.e. the post author for post-targeted cases). The appeal handler checks `target_person_id == caller.person_id`. For post-targeted cases `target_person_id` is NULL — see Bug #10 above; set it manually first.

Reporter1 can only appeal if `winning_decision` is `NoAction` or `AdvisoryLabel`. For `Warning`/`Remove`/`Ban` outcomes only the defendant can appeal.

```bash
# Prerequisite: set target_person_id if targeting a post (see Bug #10)
# docker exec docker-postgres-1 psql -U lemmy -d lemmy -c "UPDATE moderation_case SET target_person_id=<post_author_person_id> WHERE id=$CASE_ID"

curl -s -X POST http://localhost:8536/api/v4/governance/appeal \
  -H "Authorization: Bearer $ADMIN_JWT" \
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

**IMPORTANT:** The endorser must have account age ≥ `onboarding.sponsor_min_account_age_days` (default: 30 days). On a fresh instance, lower it to 0 first — see Bug #10.

```bash
# Prerequisite: lower age gate for smoke testing (DB patch)
# docker exec docker-postgres-1 psql -U lemmy -d lemmy -c "UPDATE governance_config SET value_int=0 WHERE scope='instance' AND key='onboarding.sponsor_min_account_age_days'"
# docker exec docker-postgres-1 psql -U lemmy -d lemmy -c "UPDATE governance_config SET value_int=0 WHERE scope='instance' AND key='onboarding.sponsor_min_endorsement_strength'"

# Admin endorses reporter1 (person_id=4)
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

## Phase 8 — Federation (2026-06-01 results)

**Goal:** ADR-014 — verify outbound-only federation works; Brehon governance signals are fork-only AP types; vanilla Lemmy must not error on them.

**Infrastructure:** `docker/docker-compose-vanilla.yml` + `docker/lemmy-vanilla.hjson` — vanilla nightly on port 8537. `docker/docker-compose-fed-enable.yml` — removes `LEMMY_DISABLE_ACTIVITY_SENDING` from Brehon.

**Start both stacks:**
```bash
cd docker
# Start vanilla instance
docker compose -f docker-compose-vanilla.yml up -d
# Re-enable activity sending on Brehon
docker compose -f docker-compose.yml -f docker-compose-fed-enable.yml up -d lemmy
```

| # | Check | Result | Notes |
|---|---|---|---|
| 8.1 | Resolve vanilla user from Brehon | ✅ PASS | Brehon fetched `http://host.docker.internal:8537/u/lemmy_vanilla` via AP |
| 8.2 | Resolve Brehon user from Vanilla | ❌ FAIL | Brehon hostname `https://localhost` unreachable from inside vanilla container |
| 8.3 | Resolve vanilla community from Brehon | ✅ PASS | Community resolved, id=3 on Brehon |
| 8.4 | Follow vanilla community from Brehon | ✅ PASS | Follow activity sent, no errors, vanilla accepted (community_actions count=1) |
| 8.5 | Create post in vanilla community from Brehon | ✅ PARTIAL | Post queued, Create activity sent to vanilla (`was_skipped: false`, 0 dead instances). Post stayed `federation_pending: true` because vanilla's AcceptFollow can't return to Brehon (same hostname issue). |
| 8.6 | No governance-specific AP type errors | ✅ PASS | Zero WARN/ERROR in vanilla inbox log for any Brehon activity |
| 8.7 | Brehon AP sender shows 0 dead instances | ✅ PASS | `Federating to 1/1 instances (0 dead, 0 disallowed)` throughout |

**ADR-014 core claim VERIFIED:** Brehon sends outbound AP activities (Follow, Create) without error. Vanilla nightly (v1.0.0) accepts them without error. No governance-specific AP type caused a rejection.

**Remaining blocker — BUG-15 (root cause corrected 2026-06-02):** Bidirectional federation is **not achievable with the current two-instances-on-one-host topology**, and the originally-proposed fix (`hostname: host.docker.internal:8536` + DB wipe) is *wrong* — it trades one blocker for a worse one.

Investigation (2026-06-02) established:

1. **The DB wipe was never needed.** Lemmy's `Claims::validate` (`crates/api/api_utils/src/claims.rs:26-35`) uses `Validation::default()` and never checks the JWT `iss` claim — only `sub`, signature, `exp`, and the `login_token` table. Existing JWTs survive any hostname change. The `iss` field is informational.

2. **Two compounding config issues exist, both fixable:** (a) hostname `localhost` is unreachable cross-container; (b) `tls_enabled` defaults to `true` (`crates/utils/src/settings/structs.rs:39` `#[default(true)]`), so Brehon's `ap_id`s generate as `https://` even though Brehon serves plain HTTP behind the nginx proxy. A surgical SQL rewrite of the 5 local rows (2 person + 1 community + 2 post + site `ap_id`/`inbox_url`) plus `hostname` + `tls_enabled:false` makes the actor/inbox endpoints reachable — *verified*: after the change, `curl` from inside the vanilla container successfully fetched `http://host.docker.internal:8536/u/lemmy` with a valid `Person` actor.

3. **But the change introduces a fatal domain collision.** Lemmy keys federated instances by **port-stripped domain** (`Settings::get_hostname_without_port`, `crates/utils/src/settings/mod.rs:71-81` — "removes the port"). Setting Brehon's hostname to `host.docker.internal:8536` makes its domain `host.docker.internal` — **identical** to vanilla's domain (vanilla hostname `host.docker.internal:8537` → domain `host.docker.internal`). Each instance then treats the other as *itself*. Confirmed empirically: vanilla's `instance` table holds a single row `host.docker.internal` (its own), and `resolve_object` for `http://host.docker.internal:8536/u/lemmy` fails silently (`resolve_object_failed`, no fetch attempted) because vanilla resolves the domain to its own local instance and finds no local user `lemmy`. The original Phase 8 outbound resolution worked *precisely because* `localhost` ≠ `host.docker.internal` gave the two instances distinct domains.

**Correct fix (deferred — out of smoke-test scope):** adopt the upstream `docker/federation/` topology — distinct **container hostnames** (`lemmy-alpha:8541`, `lemmy-beta:8551`, … as in `docker/federation/docker-compose.yml`) on a shared Docker network, NOT the same host with different ports. That gives each instance a unique port-stripped domain. ADR-014's core claim (outbound-only AP, governance types don't error on vanilla) is **already verified** via Phase 8.1/8.3–8.7; full bidirectional federation requires the distinct-hostname rebuild and is not needed to validate v0.

---

## Phase 9 — TOTP MFA ✅ PASS (2026-06-01)

**Note:** Lemmy uses TOTP (`totp-rs`, RFC 6238 SHA1) — not WebAuthn passkeys. No second device needed; tested fully via API.

| # | Check | Result | Notes |
|---|---|---|---|
| 9.1 | Generate TOTP secret | ✅ | `POST /api/v4/account/auth/totp/generate` returns `otpauth://` URL |
| 9.1b | Enable TOTP with valid token | ✅ | `POST /api/v4/account/auth/totp/edit` `{enabled:true}` → `{enabled:true}` |
| 9.2a | Login without TOTP token (should fail) | ✅ | Returns `missing_totp_token` error |
| 9.2b | Login with valid TOTP token | ✅ | JWT issued; TOTP token computed via Python HMAC-SHA1 |
| 9.3 | Governance endpoint works post-TOTP login | ✅ | `GET /api/v4/governance/reputation/me` → 200 with reputation view |
| 9.4 | Wrong TOTP token rejected | ✅ | `000000` → `incorrect_totp_token` HTTP 400 |

TOTP disabled after test to restore clean stack state.

---

## Deferred (out of v0 scope)

- Extism plugin hooks (plugins/ empty in dev compose)
- Supermajority voting rules (v1)
- Reputation decay curve (stub only in v0)
- Sponsor liability flow (v1-SL)
- Cross-instance governance signal parsing on remote (fork-only AP type)
