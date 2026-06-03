# Session retro — 2026-06-01 — docker-smoke-test-phases-3-7

**Harness:** claude-code  
**Session window:** ~16:00 IST → ~19:00 IST (~180 min)  
**Branch at start:** `f7c75127a` (`governance-v0`)  
**Branch at end:** `0ba955280` (`governance-v0`)  
**Files touched:** 4 (smoke-test-plan.md, smoke-test-findings.md, project_docker_spinup_ready.md, MEMORY.md + handover)  
**Commits:** 7 explicit (all docs/chore; no code changes)

## TL;DR

Ran Phases 3–7 of the Brehon Docker smoke test end-to-end inline (no Junior dispatch). Completed the full governance case lifecycle (report → jury → vote → appeal → endorsement) and verified all admin endpoints. Found and documented 14 API bugs/discrepancies, most stemming from missing prerequisites on a fresh single-user instance. The biggest recurring friction pattern was the rate limiter (180/60s general bucket) — probing too eagerly after each step exhausted the budget repeatedly and turned a ~60-min test run into ~180 min. The most actionable change is a smoke-test script that respects rate limits by design: one burst of test calls, then a single coordinated wait, rather than per-step reactive waits.

---

## What surprised us

- **Rate limit exhausted 5× despite spacing 2–3s between calls.** The 180/60s bucket looks generous but admin endpoints count against it too. A burst of 4–5 admin GET calls after any phase transition was enough to trigger 429. Didn't expect the admin surface to share the same bucket as user-facing API calls.

- **`appeal_window_expires_at` is set but `target_person_id` is not for post-targeted cases.** The appeal handler has the window correctly set, but the defendant check uses `target_person_id == caller.person_id` with no fallback to resolve `post.creator_id`. The window is functional but the defendant path is unreachable. Silently wrong — nothing errors at case creation time.

- **`onboarding.sponsor_min_account_age_days = 30` is a fresh-instance footgun.** Every governance feature that touches the sponsorship/endorsement graph silently requires accounts to be 30 days old. On a fresh instance every account is 0 days old. Returns `not_found` — same opaque error as appeal and jury eligibility. This pattern (config gate returns `not_found` instead of a diagnostic error) appeared three times: jury eligibility, endorsement, and config write body mismatch.

- **`Explore` subagent was highly effective for handler-source lookups.** Each invocation (appeal handler, endorsement handler, admin route list, config write struct) returned the exact code path needed in one round-trip and stayed out of the main context window. Saved significant wall-clock time compared to manually grepping a codebase that isn't in the CWD.

- **Non-ASCII characters in POST bodies cause deserialization errors.** The em-dash `—` in a `reason` string triggered `invalid unicode code point`. Surprising because the API server is Rust/Axum which handles UTF-8 natively — likely a shell encoding issue (PowerShell heredoc mangling) rather than the server itself, but the symptom is at the JSON parse layer.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Add a smoke-test bootstrap SQL script** at `docker/scripts/smoke-test-setup.sql` that applies all 4 config patches (panel_size=1, quorum=1, sponsor_min_account_age=0, sponsor_min_endorsement=0) + `accepted_application=true` for admin in one idempotent run. | Eliminates the 4 separate DB patches currently scattered through the test plan. Fresh-instance smoke testing becomes one `docker exec ... < smoke-test-setup.sql` before Phase 3. | minor | 4 separate patches this session; same pattern will recur every fresh stack spin-up |
| 2 | **Batch Phase 7 admin endpoint probes into a single script** (`docker/scripts/smoke-test-phase7.sh`) that spaces calls with fixed 3s sleeps and does all 8 endpoints in one coordinated pass, exiting with non-zero on any 4xx/5xx that isn't `429`. | Turns a 5× rate-limit-retry loop into a single ~30s sequential pass. Currently each endpoint is a separate manual call with re-discovery of rate limit state. | minor | Rate limit exhausted 5× across session; Phase 7 alone triggered 3 of those |
| 3 | **Document the `not_found` opaque error pattern as BUG-2** in findings doc: jury eligibility, endorsement age gate, and config body mismatch all return `not_found` instead of a diagnostic code. File a tracked issue so a future fix pass gives these distinct 4xx codes (`400 Bad Request` with `reason` for config; `403 Forbidden` with `required_age_days` for sponsor gate). | Future testers won't spend 10-15 min re-reading handler source every time a `not_found` appears. The 3-occurrence pattern this session makes it a confirmed class, not a one-off. | minor (doc) / medium (code fix) | 3× this session: jury, endorsement, appeal |
| 4 | **Fix appeal handler to resolve `target_post_id.creator_id` as defendant** when `target_person_id IS NULL` in `crates/api/api_crud/src/governance/request_appeal.rs` ~line 135. | Unblocks appeal for the majority of governance cases (post-targeted reports are the common case). Currently requires manual DB patch. | medium | 1× this session — but BUG-1 blocks all post-targeted appeals in production |

## What to carry forward

- **`Explore` subagent for handler lookups is the right tool.** When hitting a `not_found` with unclear cause, spawning an Explore agent to read the handler source (with exact file, not function name) returns the NotFound condition table in one pass. Used 4× this session with zero wasted round-trips. Pattern: brief the agent on the error symptom, the case data, and what you've already ruled out.

- **Poll-until-clear pattern for rate limits beats sleep.** Using `until [ "$(curl ...)" = "ok" ]; do sleep 15; done` run in background gives a notification when clear and costs zero active attention. Previously used bare `sleep 90` which blocked the session. 

- **Verify DB state before blaming the API.** Three times this session, `not_found` looked like a code bug but was actually a config/data prerequisite not met. Reading the handler source first (via Explore) and cross-checking with a direct DB query resolved each in <5 min. Without those two verification steps the session would have spent 30+ min chasing phantom bugs.

- **Findings doc as a parallel artifact alongside the test plan.** The `docker-smoke-test-findings.md` captures API discrepancies in a form directly usable for doc writing — structured by bug/diff/config/behaviour rather than by phase. Written once per session close; far easier to turn into API documentation than the phase-by-phase test plan.

- **Rate limit accounting: admin endpoints cost from the same 180/60s budget.** Space admin endpoint probes the same as user-facing calls. Treat each GET to an admin endpoint as costing 1 request from the same bucket; plan a full 60s+ gap between any burst of >5 admin calls.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `Explore` subagent (×4: appeal handler, endorsement handler, admin route list, config write struct) | 40 | 0 | medium | Each returned exact code path in one pass; stayed out of main context. ~10 min saved per lookup vs manual grep across unfamiliar crate layout |
| Rate-limit reactive recovery (×5) | 0 | 45 | low | Knew rate limit was an issue from prior session; still hit it 5×. The fix is a batching script (see §"What to change" #2), not better reactive recovery |
| `until` poll loop (background) | 10 | 0 | none | Clean pattern; ran 2× without issues; notification-on-complete worked |
| Direct DB queries for state verification | 15 | 0 | none | Faster than waiting for API to recover from rate limit; authoritative. Used 6× across phases 3–7 |
| Per-step interactive calling | 0 | 30 | low | Each phase's calls done one-by-one manually. Batching into scripts from the start would have saved ~30 min |
| `git stash list` session-start check | 2 | 0 | none | Clean stash (empty). Mandatory per advisor-orchestrator rule; took 2s |

## Complexity scores (heavy tasks only)

This session had no Junior tasks and no multi-file code changes — all work was interactive test execution + documentation. No complexity scores applicable under the `<files>/<commits>/<runtime-min>/<max-log-silence-min>` metric (which targets impl-task subagent work).

**Session-level throughput:** 7 commits, 4 files, ~180 min wall-clock, covering 5 phases (3–7) and a findings reference doc. Roughly 36 min/phase average; Phase 7 inflated by 3× rate-limit waits (~45 min of the total were idle).

## Decisions to revisit

- **Phase 8 (federation) — second instance approach:** The handover proposes a second Docker Compose stack on port 8537. Alternatives to consider: (a) use the existing `docker/federation/` directory if it already has a second-instance config; (b) use a public Lemmy instance for the remote end to avoid running two stacks locally. Check `docker/federation/` contents before spinning up a second stack from scratch.

- **`not_found` opaque error for config-gated failures:** Three different handlers return `not_found` for what are really `403 Forbidden` or `400 Bad Request` situations. Worth a single pass to give each a proper error code before Phase 8/9 — these will recur in federation and passkey testing.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Smoke-test bootstrap SQL script** (`docker/scripts/smoke-test-setup.sql`): promote prerequisite DB patches from scattered test plan notes into a single idempotent script. Blocks fresh-instance testing without it. 4× recurrence this session.
- [ ] **`not_found` opaque error pattern lesson** (`.claude/lessons/feedback_governance_not_found_opaque_error.md`): three distinct governance handlers returning `not_found` for pre-condition failures. Pattern is confirmed; future developers and testers will hit this. 3× this session.
- [ ] **`docker/scripts/smoke-test-phase7.sh`** (or general `smoke-test-runner.sh`): batched endpoint probe script with built-in 3s spacing. 5× rate-limit exhaustion this session makes this a high-value addition. minor cost.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
