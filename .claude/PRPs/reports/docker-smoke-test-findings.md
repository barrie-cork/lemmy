# Brehon Docker Smoke Test — API Findings

**Date:** 2026-06-01  
**Sessions:** 1–3  
**Instance:** Fresh Docker stack, localhost:8536  
**Phases tested:** 0–6 (PASS), 7 in progress

This document captures all API discrepancies, bugs, and non-obvious behaviours discovered
during end-to-end smoke testing of the Brehon governance overlay on a fresh Docker instance.
Intended audience: future developer onboarding, API documentation, and integration test authors.

---

## Confirmed bugs (unresolved — need code fixes)

### BUG-1: Appeal handler does not resolve post author as defendant

**Endpoint:** `POST /api/v4/governance/appeal`  
**File:** `crates/api/api_crud/src/governance/request_appeal.rs` ~line 135  
**Symptom:** Returns `{"error":"not_found"}` when the post author tries to appeal a post-targeted case.  
**Root cause:** The handler checks `case.target_person_id == caller.person_id` for the defendant path.
For `target_type=post` cases, `target_person_id` is always NULL — the post's `creator_id` is never
resolved into `target_person_id`. The post author has no path to appeal.  
**Reporter-appeal restriction:** Reporter can only appeal if `winning_decision` ∈ {`NoAction`, `AdvisoryLabel`}.
For `Warning`/`Remove`/`Ban` decisions only the defendant can appeal — but defendant is unreachable
when `target_person_id=NULL`.  
**Workaround (smoke testing):** `UPDATE moderation_case SET target_person_id=<post_author_person_id> WHERE id=$CASE_ID`  
**Fix suggestion:** In `request_appeal`, when `target_person_id IS NULL` and `target_post_id IS NOT NULL`,
resolve the post's `creator_id` and use that as the defendant check.

---

## API behaviour differences from documentation/expectation

### DIFF-1: All enum values are lowercase

**Applies to:** `target_type`, `decision`, and likely all governance enums.  
**Examples:**
- `target_type`: `"post"` not `"Post"`, `"comment"` not `"Comment"`, `"person"` not `"Person"`
- `decision` (jury vote): `"warning"` not `"Warning"`, `"noaction"` not `"NoAction"`
- Inferred lowercase: `"remove"`, `"ban"`, `"emergencyremove"`, `"advisorylabel"`

**Error when wrong case:**
```json
{"error":"unknown variant 'Post', expected one of 'post', 'comment', 'person', 'community', 'remote_instance'"}
```

---

### DIFF-2: Public modlog returns summarised `public_case_log` rows, not raw governance_log

**Endpoint:** `GET /api/v4/governance/modlog`  
**Expected:** List of governance_log entries with `entry_kind`, `entry_hash`, `actor_pseudonym`  
**Actual:** List of `public_case_log` summary rows — one per case, shape:
```json
[{"case_id": 1, "summary": "Case #1 (Post): jury decided Warning. Reason: spam",
  "published_at": "2026-06-01T...", "appealed": true}]
```
**Implication:** Hash chain verification and GDPR pseudonym audit require direct DB access or a
separate admin endpoint (`/api/v4/governance/admin/log` — returns empty body, may not be implemented).

---

### DIFF-3: Double-endorse returns `not_found` not `conflict`

**Endpoint:** `POST /api/v4/governance/endorsement`  
**Actual block mechanism:** 48-hour cooldown (`liability.revoke_rate_limit_per_day`), not a true
idempotency check. Returns `{"error":"not_found"}` — opaque, doesn't tell the caller why.  
**Expected:** `{"error":"conflict"}` or `{"error":"already_endorsed"}` would be clearer.

---

### DIFF-4: `GET /api/v4/governance/modlog` returns bare array, not `{"entries": [...]}`

When returning multiple entries the response is a JSON array `[...]` not `{"entries": [...]}`.
When rate-limited it returns `{"error": "too_many_requests"}` — a dict.
Callers must handle both types.

---

## Configuration patches required for single-user smoke testing

These governance_config values are production-suitable but block smoke testing on a fresh
single-user instance. Apply before running the relevant phase:

| Config key | Default | Set to | Phase | Why |
|---|---|---|---|---|
| `jury.panel_size*` (all sub-keys) | 5 | 1 | Phase 3 | Only 1 eligible juror on fresh instance |
| `jury.quorum` | 3 | 1 | Phase 3 | Must be ≤ panel_size |
| `onboarding.sponsor_min_account_age_days` | 30 | 0 | Phase 5 | Admin account is brand new |
| `onboarding.sponsor_min_endorsement_strength` | 25 | 0 | Phase 5 | No endorsement history yet |

**SQL to apply all patches at once:**
```sql
UPDATE governance_config SET value_int=1
  WHERE scope='instance' AND key LIKE 'jury.panel_size%';
UPDATE governance_config SET value_int=1
  WHERE scope='instance' AND key='jury.quorum';
UPDATE governance_config SET value_int=0
  WHERE scope='instance' AND key='onboarding.sponsor_min_account_age_days';
UPDATE governance_config SET value_int=0
  WHERE scope='instance' AND key='onboarding.sponsor_min_endorsement_strength';
```

---

## Per-account DB patches required for single-user testing

```sql
-- Admin account: approved jury eligibility (defaults to false on fresh instance)
UPDATE local_user SET accepted_application=true WHERE person_id=2;

-- (Phase 4 only) Set target_person_id for appeal to work on post-targeted cases
-- Replace 2 with actual post author's person_id, 1 with actual case_id
UPDATE moderation_case SET target_person_id=2 WHERE id=1;
```

---

## Rate limiting behaviour

**General bucket:** 180 requests / 60 seconds. Easy to exhaust during a test session.
- No `Retry-After` header returned on 429
- Wait at least 75–90 seconds after exhaustion before retrying
- Space governance calls with ≥ 2s between each
- Error shape when rate-limited: `{"error":"too_many_requests"}` (a dict, not a list — see DIFF-4)

**Post creation bucket:** 6 / 600 seconds. Create test posts once and reuse IDs.

---

## Auth / registration non-obvious behaviour

- **Auth endpoint is v4-specific:** `POST /api/v4/account/auth/login` (NOT `/api/v4/user/login`)
- **Registration mode:** `RequireApplication` — new accounts need explicit admin approval before JWTs work for writes
- **Admin JWT sub:** The JWT `sub` field is `local_user.id`, NOT `person.id`
  - Admin: JWT sub=1 → local_user_id=1 → person_id=2
  - Reporter1: JWT sub=2 → local_user_id=2 → person_id=4
- **Community names:** alphanumeric + underscore only; hyphens are rejected with a 400

---

## Governance log integrity (verified via DB)

After completing phases 3–6 (case lifecycle + appeal + endorsement):

```
Total entries: 16 (IDs: 1, 3–16; id=2 is a gap — one system event skipped/never written)
Hash chain: valid on all entries (genesis on id=1; prev_hash = prior entry_hash throughout)
entry_hash: 16/16 non-null
actor_pseudonym: 10/16 non-null (6 system-generated entries have null — correct; no direct actor)
```

**System entries without pseudonym:** `jury_constraint_relaxed` ×2, `sanction_created`,
`public_log_published`, `case_decided`, `severity_tier_frozen` — all system-initiated events
where GDPR pseudonymisation doesn't apply (no human actor).

**Governance log entry kinds observed:**
`report_created`, `jury_constraint_relaxed`, `severity_tier_frozen`, `jury_assigned`,
`panel_assembled`, `jury_accepted`, `jury_vote_submitted`, `sanction_created`,
`public_log_published`, `vote_outcome_recorded`, `case_decided`, `appeal_requested`,
`jury_constraint_relaxed` (appeal), `appeal_panel_assembled`, `endorsement_created`

---

## Person / local_user ID mapping (this instance)

| local_user.id (JWT sub) | person.id | username | role |
|---|---|---|---|
| 1 | 2 | lemmy | admin |
| 2 | 4 | reporter1 | regular user |
| 3 | 3 | lemmy_Lj3hGhclCOLgwx | system/bot |

---

## Phases 7–9 status

| Phase | Status | Blocker |
|---|---|---|
| 7 — Admin endpoints | 🔄 In progress | Rate limit during initial probe |
| 8 — Federation | ⏳ | Needs second Lemmy instance |
| 9 — Passkey/MFA | ⏳ | Needs WebAuthn client setup |
