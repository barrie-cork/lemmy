---
name: governance-not-found-opaque-error
description: Three distinct Brehon governance handlers return not_found for configuration pre-condition failures rather than 400/403. Pattern confirmed 3x in smoke test session 2026-06-01. Read handler source + check DB config before assuming a code bug.
metadata:
  type: feedback
---

Three distinct governance handlers return `{"error":"not_found"}` for situations that are really configuration pre-condition failures — not missing resources:

1. **`POST /governance/admin/assign-jury`** — returns `not_found` when eligible juror count < `panel_size`. The pool exists; the config is just miscalibrated for the current user count.
2. **`POST /governance/endorsement`** — returns `not_found` when the endorser's account age < `onboarding.sponsor_min_account_age_days` (default: 30 days). Fresh-instance accounts are 0 days old.
3. **`POST /governance/appeal`** — returns `not_found` for the defendant path when `moderation_case.target_person_id IS NULL` (post-targeted cases never set this column). The appeal window is open; the defendant is just unreachable.

**Why:** The handlers map all failure paths to `LemmyErrorType::NotFound` to avoid leaking information about internal state. This is privacy-correct but opaque for local development and smoke testing.

**How to apply:** When a governance endpoint returns `not_found` and the targeted resource clearly exists in the DB:

1. **Read the handler source** (via Explore agent: `Find the handler for POST /api/v4/governance/<endpoint> and list all NotFound return sites and their exact conditions`). Takes ~2 min; returns the full condition table.
2. **Check the config gate**: `SELECT scope, key, value_type, COALESCE(value_int::text, value_float::text, value_bool::text, value_text) as value FROM governance_config WHERE key LIKE '%<relevant_prefix>%';`
3. **Check the data gate**: for jury — `SELECT accepted_application FROM local_user WHERE person_id=<id>`; for appeal — `SELECT target_person_id FROM moderation_case WHERE id=<id>`.

Do NOT assume a code bug until both steps come up empty. All three instances above were config/data issues, not handler bugs.

**Fix status:**
- `assign-jury` + `endorsement` — **workable via config patch** (lower thresholds for testing). SQL in `docker/scripts/smoke-test-setup.sql`.
- `appeal` — **code bug** (BUG-1 in `docker-smoke-test-findings.md`): `request_appeal.rs` never resolves `target_post_id.creator_id` as the defendant. No config workaround; requires manual DB patch (`UPDATE moderation_case SET target_person_id=<author_id>`).

**Related:** [[feedback_pre_phase_dod_smoke_test]], `docker/scripts/smoke-test-setup.sql`
