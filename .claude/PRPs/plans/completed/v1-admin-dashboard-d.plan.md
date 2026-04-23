# Plan: v1-AD-d — Dashboard aggregate + SSE audit stream

## Table of contents

| § | Heading | Line |
|---|---|---|
| 1 | Summary | 28 |
| 2 | Source | 40 |
| 3 | Problem statement | 64 |
| 4 | Solution statement | 88 |
| 5 | Metadata | 148 |
| 6 | Sub-phase position (v1-AD-a → b → c → d) | 166 |
| 7 | Open questions already resolved / reserved | 186 |
| 8 | Flow design | 210 |
| 9 | Mandatory reading | 302 |
| 10 | Patterns to mirror | 350 |
| 11 | Files to change | 660 |
| 12 | NOT building in v1-AD-d | 685 |
| 13 | Step-by-step tasks | 720 |
| 14 | Testing strategy | 1150 |
| 15 | Validation commands (DoD) | 1200 |
| 16 | Acceptance criteria | 1270 |
| 17 | Completion checklist | 1300 |
| 18 | Risks and mitigations | 1330 |
| 19 | Notes | 1380 |

---

## 1. Summary

v1-AD-d is the **read-only aggregate + live-stream sub-phase** of the v1 admin-dashboard keystone. It ships two new `GET` endpoints on top of the v1-AD-a/b/c substrate: `GET /api/v4/governance/admin/dashboard` returns a single-round-trip aggregate (active cases, jury queue depth, reputation health, federation status, rule-set summary, recent config changes) suitable for a future server-rendered page; `GET /api/v4/governance/admin/audit/stream` is a hand-rolled Server-Sent Events stream that wraps Postgres `LISTEN governance_events` and forwards each notification as a typed `AdminConfigAuditEntry` for the two admin-config entry kinds. No new migrations, no new entry kinds, no writes — both endpoints are pure consumers of the substrate already in `governance-v0`.

Scope is **deliberately bounded**: v1-AD-d writes zero bytes to Postgres. It does NOT ship askama HTML pages (OQ-V1-AD-01 deferred to v1-AD-e / v1.x), does NOT ship a `tower-sse` / `actix-web-lab` dependency (OQ-V1-AD-02 resolved: hand-roll with `async-stream`), and does NOT extend any DTO outside the new dashboard response shape. Four to five tasks depending on whether the `/reputation-stats` embed re-uses the existing handler or copies its helpers.

---

## 2. Source

- [../prds/v1-admin-dashboard.prd.md](../prds/v1-admin-dashboard.prd.md) §4.1 (endpoint table rows for `/admin/dashboard` and `/admin/audit/stream`), §4.7 (routes wiring block), §6.2 (dashboard widget data-source matrix), §7.1 (capability = `is_admin` for both endpoints), §7.3 (SSE has its own throttle: max 1 connection per admin user — v1-AD-d enforces at handler level per §4.3 load-bearing decision).
- [./v1-admin-dashboard-b.plan.md](v1-admin-dashboard-b.plan.md) §4.1 (load-bearing decisions carried over — `is_admin` + `ConfigCache::new()` per request), §10 (patterns to mirror for `admin_get_config_audit` — shape mirror for the `recent_config_changes` widget), §11 (files-to-change shape).
- [./v1-admin-dashboard-c.plan.md](v1-admin-dashboard-c.plan.md) §4.1 (rule-set CRUD surface now available — dashboard may read it), §10.1 (capability-gate pattern).
- [../reports/v1-AD-d-pre-plan-brief.md](../reports/v1-AD-d-pre-plan-brief.md) — scope, dependencies, branch topology, advisor notes on hand-rolled SSE.
- [../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) — ADR-008 (signed log — dashboard reads `governance_log` but never writes), ADR-010 (v1 staged releases — v1-AD-d is the final v1-AD-wave sub-phase), ADR-013 (`CaseStatus::EmergencyRemove` exhaustive match — task 2 filter uses `.ne_all(...)` so no match statement), ADR-015 (pseudonyms — dashboard surfaces the same `actor_pseudonym` the audit list already returns; no new pseudonym write paths).
- [../../../docs/brehon-law-inspired-network/04-data-model-and-api.md](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) §7 (route table — confirms `/admin/dashboard` and `/admin/audit/stream` paths are reserved v1 slots).
- `migrations/2026-04-20-000000-0000_add_governance_log_notify/up.sql` — the `governance_events` NOTIFY trigger that powers the SSE endpoint (already shipped with v1-AD-a / v1-AD-b substrate).
- `.claude/rules/governance-log-entry-kind-registry.md` — **no new entry kinds added** by v1-AD-d. §Acceptance invariants count stays at 26 (19 v0 + 4 Phase 6 + 2 v1-AD-a + 1 v1-AD-c).
- `.claude/rules/phase-branch.md` — working branch is `phase-v1-AD-d` per task 0.
- `.claude/rules/pre-phase-harness-audit.md` — task 0 runs the 4-probe audit.
- `.claude/rules/no-cargo-output-paste.md` + `.claude/rules/cargo-output-capture.md` — SSE build log in particular can be noisy; always redirect to file.
- `project_v1_AD_c_closed.md` (advisor memory) — prior-phase-close state, carry-forward issues #82–#85 (all v1-AD-c chores; none block v1-AD-d).

---

## 3. Problem statement

Three things must ship before the v1 admin-dashboard keystone is user-visible (not counting the v1-AD-e askama pages, which remain descoped):

1. **A single-request dashboard aggregate.** A future server-rendered `/admin/governance/dashboard` HTML page (v1-AD-e or v1.x) plus any API client (curl, Postman, the forthcoming React shell) needs one GET that returns every "what state is this instance in right now" number: how many cases are open, how many juries are pending, how the reputation distribution looks, what the last N config changes were, whether federation peer-trust state is healthy, which communities have rule-sets and at which version. Today each widget requires 5–6 separate GETs against unrelated endpoints (or raw SQL), and three of those endpoints (rule-sets per community, federation attestations, recent governance_log entries) don't have a summary projection at all — clients would have to list-and-count. The dashboard aggregator is the synthesis.

2. **Live observability without polling.** Operators want to see config-change events land as they happen (whether via a Discord webhook, a tail on an admin console, or the future HTML page's live-updating audit widget). Polling `/admin/config/audit` every N seconds is wasteful, misses sub-N-second events between polls, and adds load during quiet periods. The v1-AD-a/b substrate already publishes each `governance_log` INSERT on Postgres channel `governance_events` via the `governance_log_notify` trigger. What's missing is the Rust-side consumer: a handler that holds a dedicated Postgres LISTEN connection, serialises each notification as a Server-Sent Events frame (`event: admin_config_changed\ndata: <json>\n\n`), and keeps the HTTP response open until the client disconnects or the server drops the connection.

3. **A substrate for the v1-AD-e / v1.x page layer.** The askama HTML pages per PRD §6 are descoped-for-now (OQ-V1-AD-01) but will eventually call exactly this dashboard aggregate. Shipping the JSON endpoint first means when the page ships, it is a pure templating change — no new DB queries, no new capability gates, no new DTO churn.

Without #1, pilot operators have no single-fetch "is the system healthy?" API. Without #2, the only path to live audit tailing is `tail -f` against Postgres logs or a separate job that polls. Without #3, v1-AD-e will bundle page-templating and data-fetching in a single PR, doubling the review surface and the regression risk.

---

## 4. Solution statement

Ship **two new GET handlers** and **one new workspace dependency** (`async-stream` — `futures-util` is already in `Cargo.toml:215`, `tokio-postgres` is already in `Cargo.toml:224`, `actix-web 4.13.0` is already in `Cargo.toml:175`):

- **`GET /api/v4/governance/admin/dashboard`** (`admin_dashboard`) — capability-gates via `is_admin(&local_user_view)?`, opens ONE `get_conn(pool)` connection, runs five to six sequential reads on it, and returns `AdminDashboardResponse { active_cases, jury_queue, recent_config_changes, federation, reputation, rule_sets, calculated_at }`. Each widget's data source:

  | Widget | Source | Query shape |
  |---|---|---|
  | `active_cases: ActiveCasesSummary` | inline `sql_query` on `moderation_case` | `COUNT(*) FILTER (WHERE status IN (...))` one-round-trip per bucket OR `GROUP BY status` one-round-trip total |
  | `jury_queue: JuryQueueSummary` | inline `sql_query` on `jury_assignment` | `COUNT(*) FILTER (WHERE status IN ('Selected', 'Accepted'))` + `COUNT(*) FILTER (WHERE status = 'Submitted')` |
  | `recent_config_changes: Vec<AdminConfigAuditEntry>` (last 20) | `governance_log` filtered to `entry_kind IN ('admin_config_changed', 'admin_config_change_denied')` via Diesel `.order_by(created_at.desc()).limit(20)` | re-uses `project_to_audit_entry` from `admin_config.rs` |
  | `federation: FederationSummary` | inline `sql_query` on `federation_attestation` | `COUNT(*) FILTER (WHERE valid_until > now())` active, `COUNT(*) FILTER (WHERE valid_until <= now())` expired, `COUNT(*)` total |
  | `reputation: AdminReputationStatsResponse` | re-use of `admin_reputation_stats` handler's helpers (`bucket_query`, `capability_query`, `founder_query`) at instance scope | delegated |
  | `rule_sets: RuleSetSummary` | two inline queries on `rule_set_version` + `governance_config` | `COUNT(DISTINCT community_id)` total, `COUNT(*)` total rows, + `rule_set.active_version_id` value per community (bounded at first 100 communities) |

  No `dry_run`, no writes, no `governance_log::append` call. Latency budget: 6–8 DB round-trips (bucket-per-dimension is 4 queries itself via the reputation helpers; the rest is 5 more inline queries). Acceptable for a dashboard load; caching is out of scope for v1-AD-d.

- **`GET /api/v4/governance/admin/audit/stream`** (`admin_audit_stream`) — capability-gates via `is_admin`, acquires a **dedicated** `tokio_postgres::Client` (NOT a diesel-async pool connection — LISTEN holds the connection open for the full request lifetime), executes `LISTEN governance_events`, and returns an `HttpResponse::Ok().content_type("text/event-stream").streaming(sse_body)` where `sse_body` is a hand-rolled `async_stream::stream!` that:

  1. Emits one `event: retry\ndata: 10000\n\n` header frame (ask client to reconnect after 10s on drop).
  2. Pulls `AsyncMessage::Notification` events off the `tokio_postgres` connection in a loop.
  3. For each notification: parses the JSON payload (`{entry_id, kind, created_at}` per the trigger), filters to the two `admin_config_*` kinds (drops all others — Phase 6 federation kinds, v0 reputation-delta kinds, v1-AD-c rule-set kinds all get dropped on the wire), fetches the full `governance_log` row by `entry_id`, projects through `project_to_audit_entry`, and yields `Ok::<Bytes, actix_web::Error>(Bytes::from(format!("event: {kind}\ndata: {json}\n\n", ...)))`.
  4. On `Err(_)` from the connection (drop, timeout), the stream ends — the client reconnects per the retry header.

  A heartbeat timer (via `tokio::time::interval(Duration::from_secs(15))` selected alongside the notification receiver with `tokio::select!`) sends `: keepalive\n\n` (SSE comment line) every 15 seconds so intermediate proxies and Actix's own keep-alive logic don't time out the idle connection.

  **Backpressure / cap**: per PRD §7.3, SSE has a throttle of "max 1 connection per admin user". v1-AD-d implements this via an **in-memory `Arc<DashMap<PersonId, ()>>`** initialised at module level (or, if we prefer to keep the handler stateless, via a per-connection `Mutex` check-and-insert). On second connection from the same admin, return 409 Conflict and a plain-text message. When the stream drops, `Drop` on a guard struct removes the entry. Alternative rejected: a DB row to track the state (too heavy for the use case and introduces a write).

- **`Cargo.toml` addition**: `async-stream = "0.3"` added to the workspace dependencies table. No other new deps. The e2e test reads the SSE stream via the existing `reqwest` workspace dep + its `stream` feature — which must be added to the workspace `reqwest = { ..., features = [...] }` list since it's not currently enabled.

Capability checks follow the v1-AD-b pattern: `is_admin(&local_user_view)?` at handler entry; failure returns `LemmyErrorType::NotAnAdmin` (HTTP 403), which v1-AD-b already maps. Both endpoints are instance-admin-only per PRD §7.1 (no per-community access — dashboard + SSE surface instance-level aggregates).

Routes register alongside the existing `/admin/config` and `/admin/rule-sets` blocks at `crates/api/routes/src/lib.rs:533-549`. The `/admin/dashboard` route goes at the top-level of the `/admin` scope (sibling of `/assign-jury`, `/reputation-stats`). The `/admin/audit/stream` route goes in a new nested `scope("/audit")` inside `/admin`. Sub-phase branch is `phase-v1-AD-d` (cut from `governance-v0` at the plan-merge commit). PR target is `governance-v0` per `.claude/rules/phase-branch.md`.

### 4.1 Architecturally load-bearing decisions locked in this sub-phase

- **Dashboard is 100% read-only.** Zero INSERTs to any governance table. `governance_log::append` is not called. `is_admin` failure does NOT emit an `admin_config_change_denied` entry (that kind is for config-write denials; dashboard viewing is not a write attempt). Clippy `unused_import` check on `governance_log` in both new files is load-bearing — the import must genuinely not be needed.
- **SSE uses `async-stream` + raw `tokio-postgres`, NOT `actix-web-lab::sse::Sse`.** Per OQ-V1-AD-02 resolution: adding `actix-web-lab` for one endpoint is more weight than hand-rolling. `futures-util 0.3.32` (workspace line 215) provides the `Stream` trait; `async-stream 0.3` (new dep) provides the `stream!` macro; `tokio-postgres 0.7.16` (workspace line 224) provides `AsyncMessage::Notification`. The SSE frame format is three lines of code (`event: X\ndata: Y\n\n`).
- **LISTEN uses a dedicated `tokio_postgres::Client`, NOT a diesel-async pooled connection.** Pooled connections round-trip back to the pool after each `.await`; LISTEN must hold its channel subscription for the request lifetime. The handler opens a fresh `tokio_postgres::connect()` using the same `DATABASE_URL` as the pool (reads it via `context.settings().get_database_url()` or equivalent; see `crates/diesel_utils/src/connection.rs:201-227` for the full `establish_connection` fn — note: function signature at :201, TLS branch body begins at :211). The handler spawns the connection's driver task with `tokio::spawn(async move { conn.await })` and uses the resulting `Client` for LISTEN. **Connection cleanup** happens when the stream ends and the `Client` is dropped — the spawned driver task completes automatically.
- **Per-user SSE cap is in-memory + best-effort.** A `once_cell::sync::Lazy<Mutex<HashSet<PersonId>>>` or equivalent module-level guard structure tracks active SSE connections per admin. On handler entry, insert-check; on stream drop, remove. This is **not durable across server restarts** (an admin whose session survives a restart has to reconnect) and **not distributed** (a multi-node instance would allow N connections per admin where N = node count). Both are acceptable for v1 — solo-dev single-node, and "no SSE duplicate cleanup on restart" is trivial. Record as a v2 improvement.
- **Heartbeat every 15s.** Shorter than the default Actix keep-alive (75s) and nginx's default proxy read timeout (60s). Longer than a human notices. Written as `: keepalive\n\n` (colon-leading = SSE comment = client ignores but intermediate proxies see traffic).
- **Recent config changes widget reuses `project_to_audit_entry` verbatim.** The helper is currently private to `admin_config.rs` (line ~1301). Task 1 bumps it to `pub(crate)` or moves it to a new shared module — the latter is lower risk if v1 changes the projection shape. Plan recommends moving to a new `crates/api/api/src/governance/audit_projection.rs` module that both `admin_config.rs` and `admin_dashboard.rs` / `admin_audit_stream.rs` import.
- **Federation widget queries `federation_attestation` only.** `federation_outbox` / `sent_activity` per-peer pending counts are too much surface for v1-AD-d; if pilots need them, they land as v1.x additions. The widget returns active attestations, expired attestations, and total. Null result (zero rows) is normal for a solo pilot — return `{active: 0, expired: 0, total: 0}`, not an error.
- **Rule-set widget is bounded to 100 communities.** `COUNT(DISTINCT community_id)` is O(N) but the per-community `active_version_id` lookup is O(K) where K is the number of communities with rule-sets; cap K at 100 via `ORDER BY community_id LIMIT 100` in the detail query. A solo pilot instance has ≤5 communities with rule-sets in v1; 100 is an ample ceiling with no visible truncation.
- **No SSE persistence / replay.** A client that reconnects sees only events from its reconnection moment forward; it must call `/admin/config/audit` separately for catch-up. The retry-on-drop header (`event: retry\ndata: 10000`) is the client's primary mitigation.
- **No new migrations.** The `governance_log_notify` trigger shipped with the v1-AD-a/b/c substrate.
- **No new entry kinds.** v1-AD-d is a read-only consumer of `admin_config_changed` / `admin_config_change_denied` (both v1-AD-a consts). Registry invariant count stays at 26.
- **No step-up auth.** Read endpoints are outside the step-up surface (PRD §7.2 lists write operations only).
- **`EmergencyRemove` handled by filter, not match.** The active-cases filter uses `.ne_all(&[CaseStatus::Decided, CaseStatus::Closed, CaseStatus::EmergencyRemove])` (or raw SQL `status NOT IN ('Decided', 'Closed', 'EmergencyRemove')`). No `match CaseStatus { ... }` statement in the handler, so ADR-013 exhaustive-match gotcha doesn't apply — but a grep-check at task 6 confirms zero `match.*CaseStatus` additions.

---

## 5. Metadata

| Field | Value |
|---|---|
| Type | `HANDLER + DTO` (read-only; no migrations, no entry kinds, no write paths) |
| Complexity | MEDIUM (SSE + LISTEN/NOTIFY is novel for this workspace) |
| Crates affected | `lemmy_api_common` (DTOs), `lemmy_api` (2 new handlers + audit-projection module move), `lemmy_api_routes` (2 route registrations), `lemmy_server` (tests/e2e.rs), root `Cargo.toml` (add `async-stream` + `reqwest` `stream` feature) |
| v0 step | v1 post-MVP per ADR-010 — admin-dashboard keystone closes |
| Dependencies | v1-AD-a, v1-AD-b, v1-AD-c merged at `governance-v0` (tip `cf89890f3`). **Pre-flight check (task 0)**: governance-v0 HEAD must contain `project_to_audit_entry` at `crates/api/api/src/governance/admin_config.rs`, `governance_log_notify_trigger` in DB, `admin_list_rule_sets` in routes, `admin_reputation_stats` handler. |
| Estimated tasks | **6 tasks** (Task 0 pre-flight + Tasks 1–5 execution) |
| Sub-phase target branch | `phase-v1-AD-d` (cut from `governance-v0` at plan-merge commit per task 0) |
| PR target | `governance-v0` (per `.claude/rules/phase-branch.md`) |
| Blocks | v1-AD-e (askama pages — consumes `AdminDashboardResponse`); v1.x operator-dashboard-first React shell |
| Unblocks | itself — all v1-AD-a/b/c substrate is live on `governance-v0` |

---

## 6. Sub-phase position (v1-AD-a → b → c → d)

| Sub-phase | Status | What's landed / pending |
|---|---|---|
| v1-AD-a | ✅ merged (PR #72, commit `e61f78edf`) | 4 migrations (incl. `add_governance_log_notify` trigger), `ConfigKeyMetadata` registry (61 entries), 2 new `ENTRY_KIND_ADMIN_CONFIG_*` consts, Diesel models for `RuleSetVersion`, entry-kind registry rule |
| v1-AD-b | ✅ merged (PR #76, commit `f03ed1cba`) | 3 admin-config handlers, `get_*_opt` accessor family, dry-run impact queries, capability gates, audit list handler, 13 e2e tests |
| v1-AD-c | ✅ merged (PR #81, commit `cf89890f3`) | 2 rule-set handlers, case-open snapshot pin, #77 provenance-threading refactor, #78 typed-scope-parse, 1 new ENTRY_KIND const |
| **v1-AD-d** (this plan) | pending | 2 read-only handlers, 1 shared audit-projection module extraction, 1 new `async-stream` workspace dep, 5 e2e tests |
| v1-AD-e | DEFERRED (OQ-V1-AD-01) | Askama HTML pages — consumes v1-AD-d's dashboard DTO |

v1-AD-d **closes the admin-dashboard keystone API surface**. After merge the five `/admin/*` routes (`/assign-jury`, `/close-case`, `/reputation-stats`, `/config*`, `/rule-sets*`, plus new `/dashboard` + `/audit/stream`) cover the full PRD §4.1 endpoint table minus the askama pages.

---

## 7. Open questions already resolved / reserved

| OQ | Resolution | Impact on v1-AD-d |
|---|---|---|
| OQ-V1-AD-01 | Defer HTML pages to v1.x / v1-AD-e | v1-AD-d ships no askama, no template crate dep |
| OQ-V1-AD-02 | Hand-roll SSE via `async-stream` | **This plan implements it.** New workspace dep `async-stream = "0.3"`. No `actix-web-lab`. |
| OQ-V1-AD-03 | Dry-run impact pre-tx read-only query; no SAVEPOINT | N/A — v1-AD-d is read-only; no transactions. |
| OQ-V1-AD-04 | Admin attestation for participation_consistency | N/A — participation config is not dashboard-surfaced in v1-AD-d |
| OQ-V1-AD-05 | Dry-run preview log | N/A — no dry-runs in v1-AD-d |
| OQ-018 | Admin HTTP config-write endpoint | Shipped in v1-AD-b; v1-AD-d's dashboard reads the audit trail that v1-AD-b writes. |

**No new OQs opened by v1-AD-d.** If a blocking ambiguity emerges mid-implementation, it goes to `.claude/decision-queue.json` per `.claude/rules/decision-queue.md`.

### 7.1 Potential decision-queue triggers (pre-identified)

- **DQ trigger A**: if `tokio_postgres::connect` behaves differently under the `rustls` path (per `crates/diesel_utils/src/connection.rs:201-227`, TLS branch body at :211), flag the TLS-config reuse question. The pool's `establish_connection` already handles `sslmode=require`; the SSE dedicated connection must follow the same branch. If the helper isn't reusable, DQ-V1-AD-D-01.
- **DQ trigger B**: if the per-user SSE cap state (`Mutex<HashSet<PersonId>>`) needs to survive server restart — pilot feedback only. Not a blocker for v1-AD-d.
- **DQ trigger C**: if pre-phase harness audit probe 4 (negative exit-code propagation) returns exit 0, stop and fix the wrapper before task 1 per `.claude/rules/pre-phase-harness-audit.md`.
- **DQ trigger D**: if `project_to_audit_entry` is referenced by v1-AD-b or v1-AD-c tests and making it `pub(crate)` or moving it to a new module breaks a test import, adjust the refactor shape (keep it in `admin_config.rs`, export by path). DQ only if the move-vs-keep decision has >15 min scope impact.

---

## 8. Flow design

### Before state (governance-v0 HEAD `cf89890f3`, v1-AD-c merged via PR #81)

```text
╔═══════════════════════════════════════════════════════════════════════════════╗
║                              BEFORE STATE                                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║  HTTP API surface:                                                            ║
║    /admin/assign-jury         (POST)  — v0                                    ║
║    /admin/close-case          (POST)  — v0                                    ║
║    /admin/reputation-stats    (GET)   — v0                                    ║
║    /admin/config              (POST)  — v1-AD-b                               ║
║    /admin/config              (GET)   — v1-AD-b                               ║
║    /admin/config/audit        (GET)   — v1-AD-b                               ║
║    /admin/rule-sets           (POST)  — v1-AD-c                               ║
║    /admin/rule-sets           (GET)   — v1-AD-c                               ║
║    /admin/dashboard           DOES NOT EXIST                                  ║
║    /admin/audit/stream        DOES NOT EXIST                                  ║
║                                                                               ║
║  Substrate shipped and ready:                                                 ║
║    governance_log_notify_trigger   fires on each governance_log INSERT,       ║
║                                    pg_notify('governance_events', {...})      ║
║    project_to_audit_entry          private helper in admin_config.rs:1301     ║
║    admin_reputation_stats helpers  bucket_query/capability_query/founder_query║
║    moderation_case, jury_assignment, federation_attestation                   ║
║    rule_set_version, governance_config                                        ║
║                                                                               ║
║  Workspace deps:                                                              ║
║    async-stream                    ABSENT                                     ║
║    tokio-postgres                  PRESENT (0.7.16, workspace line 224)       ║
║    futures-util                    PRESENT (0.3.32, workspace line 215)       ║
║    actix-web                       PRESENT (4.13.0, workspace line 175)       ║
║    reqwest                         PRESENT (0.13.2) — no `stream` feature     ║
║                                                                               ║
║  PAIN:                                                                        ║
║    - operators have no single-fetch "is the instance healthy?" endpoint       ║
║    - no live audit tailing short of polling /admin/config/audit every Ns      ║
║    - v1-AD-e (pages) is blocked on a dashboard data source                    ║
║    - governance_log_notify_trigger has no consumer in Rust — wasted substrate ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### After state (end of v1-AD-d)

```text
╔═══════════════════════════════════════════════════════════════════════════════╗
║                               AFTER STATE                                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║  HTTP API surface:                                                            ║
║    /admin/dashboard           (GET)   — v1-AD-d admin_dashboard         NEW   ║
║      returns:                                                                 ║
║        active_cases        { open, threshold_met, jury_selection, in_review,  ║
║                              decided, appealed, closed, emergency_remove,     ║
║                              admin_review, total_active }                     ║
║        jury_queue          { pending_accept, accepted, submitted }            ║
║        recent_config_changes  Vec<AdminConfigAuditEntry>  (last 20)           ║
║        federation          { active, expired, total }                         ║
║        reputation          AdminReputationStatsResponse (instance scope)      ║
║        rule_sets           { communities_with_rule_sets, total_versions,      ║
║                              per_community: Vec<{community_id, active_version_id}> } ║
║        calculated_at       DateTime<Utc>                                      ║
║                                                                               ║
║    /admin/audit/stream      (GET)   — v1-AD-d admin_audit_stream       NEW    ║
║      Content-Type: text/event-stream                                          ║
║      Frames:                                                                  ║
║        event: retry     data: 10000                      (once, on connect)   ║
║        event: admin_config_changed     data: {AdminConfigAuditEntry}          ║
║        event: admin_config_change_denied  data: {AdminConfigAuditEntry}       ║
║        : keepalive                                   (every 15s while idle)   ║
║      Cap: 1 open stream per admin PersonId (in-memory guard)                  ║
║      Filter: only the 2 admin_config_* entry kinds; all others dropped        ║
║                                                                               ║
║  Workspace deps:                                                              ║
║    async-stream = "0.3"            ADDED                                      ║
║    reqwest                         features += [ "stream" ]                   ║
║                                                                               ║
║  Substrate reuse:                                                             ║
║    governance_log_notify_trigger   NOW consumed by admin_audit_stream         ║
║    project_to_audit_entry          PROMOTED to pub(crate) or moved to         ║
║                                    crates/api/api/src/governance/audit_projection.rs ║
║    admin_reputation_stats helpers  REUSED (bucket_query/capability_query/     ║
║                                    founder_query) at instance scope           ║
║                                                                               ║
║  VALUE ADDED:                                                                 ║
║    - pilot operators can curl one URL and see the whole governance state      ║
║    - live audit tailing with zero polling                                     ║
║    - v1-AD-e (pages) unblocked — pure templating change                       ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### Endpoint changes

| Endpoint | Before | After |
|---|---|---|
| `GET /api/v4/governance/admin/dashboard` | 404 (not registered) | 200 with `AdminDashboardResponse`; 403 if not `is_admin` |
| `GET /api/v4/governance/admin/audit/stream` | 404 (not registered) | 200 with `Content-Type: text/event-stream`; 403 if not `is_admin`; 409 if this admin already has an open stream |

### Request / response shape examples

**Dashboard (200):**
```jsonc
{
  "active_cases": {
    "by_status": {
      "Open": 3,
      "ThresholdMet": 1,
      "JurySelection": 2,
      "InReview": 4,
      "Decided": 47,
      "Appealed": 1,
      "Closed": 20,
      "EmergencyRemove": 1,
      "AdminReview": 0
    },
    "total_active": 10  // sum of everything except Decided + Closed + EmergencyRemove
  },
  "jury_queue": {
    "pending_accept": 6,   // Selected
    "accepted": 12,
    "submitted": 34
  },
  "recent_config_changes": [ /* up to 20 AdminConfigAuditEntry objects */ ],
  "federation": {
    "active": 3,
    "expired": 1,
    "total": 4
  },
  "reputation": { /* full AdminReputationStatsResponse at instance scope */ },
  "rule_sets": {
    "communities_with_rule_sets": 2,
    "total_versions": 5,
    "per_community": [
      { "community_id": 42, "active_version_id": 3 },
      { "community_id": 99, "active_version_id": 1 }
    ]
  },
  "calculated_at": "2026-04-22T19:03:04Z"
}
```

**SSE frame examples:**
```
event: retry
data: 10000

event: admin_config_changed
data: {"id":12345,"entry_kind":"admin_config_changed","scope":"instance","key":"jury.panel_size","value_type":"int","previous_value":5,"previous_from":"instance","new_value":7,"reason":"Pilot retro","actor_pseudonym":"a1b2c3d4","created_at":"2026-04-22T19:03:04Z","signature":[...],"denial_reason":null}

: keepalive

```

---

## 9. Mandatory reading (implementation agent MUST read before starting)

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `crates/api/api/src/governance/admin_config.rs` | 38–99 (imports), 1127–1153 (`admin_get_config`), 1170–1233 (`admin_get_config_audit`), 1301–1353 (`project_to_audit_entry`) | Direct shape mirror for dashboard handler + audit-entry projection the widget reuses |
| P0 | `crates/api/api/src/governance/admin_reputation_stats.rs` | 1–44 (imports), 76–136 (handler body), 141–245 (helper functions `bucket_query`, `capability_query`, `founder_query`) | Direct shape mirror for aggregate handler; helpers reused verbatim in dashboard reputation widget |
| P0 | `crates/api/api/src/governance/admin_rule_sets.rs` | 266–339 (`admin_list_rule_sets`) | Pattern: read + active_version lookup; shape mirror for rule-sets widget |
| P0 | `migrations/2026-04-20-000000-0000_add_governance_log_notify/up.sql` | all | NOTIFY payload schema — the 3 fields `{entry_id, kind, created_at}` define the SSE handler's notification deserialisation |
| P0 | `crates/diesel_utils/src/connection.rs` | 201–227 (`establish_connection`) | Reference for how the pool establishes `tokio_postgres::Client` + TLS handling — SSE handler mirrors this for its dedicated LISTEN connection |
| P0 | `crates/api/api_common/src/governance.rs` | 281–346 (AdminReputationStats DTOs), 467–517 (AdminGetConfigAudit + AdminConfigAuditEntry) | DTO shape mirror; new dashboard DTOs follow the same `#[skip_serializing_none]` + `#[cfg_attr(feature = "ts-rs", ...)]` pattern |
| P0 | `crates/db_schema_file/src/enums.rs` | 380–408 (CaseStatus), 467–486 (JuryAssignmentStatus) | Exhaustive variant lists; active-cases widget filters on `.ne_all(...)` against CaseStatus variants |
| P1 | `crates/api/routes/src/lib.rs` | 515–549 (governance admin scope) | Route registration; new routes land inside `scope("/admin")` at lines 534–549 |
| P1 | `crates/db_schema/src/source/governance/governance_log.rs` | 152–161 (entry-kind consts) | Confirms v1-AD-d doesn't need to add entry kinds — it only filters on the existing two |
| P1 | `crates/db_schema/src/source/governance/federation_attestation.rs` | 1–40 (struct definition) | Columns the federation widget queries |
| P1 | `Cargo.toml` (root) | 175 (actix-web), 214–224 (futures, tokio-postgres), 187–191 (reqwest) | Existing workspace deps; task 4 adds `async-stream` and the `stream` feature to reqwest |
| P1 | `.claude/rules/pre-phase-harness-audit.md` | all | Task 0 runs the 4-probe audit |
| P1 | `.claude/rules/phase-branch.md` | all | Working branch must be `phase-v1-AD-d` before any commit |
| P1 | `.claude/rules/governance-log-entry-kind-registry.md` | Acceptance invariants | Count MUST remain at 26 after v1-AD-d (zero new kinds) |

**External Documentation (needed only for SSE / tokio-postgres LISTEN — verify at impl time):**

| Source | Version | Section | Why |
|---|---|---|---|
| [async-stream](https://docs.rs/async-stream/0.3.5/async_stream/) | 0.3 | `stream!` macro | Hand-rolled SSE body generator |
| [tokio-postgres](https://docs.rs/tokio-postgres/0.7.16/tokio_postgres/) | 0.7.16 | `Client::batch_execute`, `AsyncMessage::Notification` | LISTEN subscription + notification polling |
| [actix-web 4.13 docs](https://docs.rs/actix-web/4.13.0/actix_web/) | 4.13.0 | `HttpResponse::streaming`, `web::Bytes` | Streaming response body |
| [HTML5 SSE spec](https://html.spec.whatwg.org/multipage/server-sent-events.html) | WHATWG | §9.2.4 event-stream format | Correct frame format: `event: X\ndata: Y\n\n` (two `\n` at end) |

---

## 10. Patterns to mirror

**CAPABILITY_GATE (both new handlers, entry line):**
```rust
// SOURCE: crates/api/api/src/governance/admin_reputation_stats.rs:81
// COPY THIS PATTERN:
pub async fn admin_reputation_stats(
  Query(data): Query<AdminReputationStats>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminReputationStatsResponse>> {
  is_admin(&local_user_view)?;
  // ... body
}
```

For dashboard:
```rust
pub async fn admin_dashboard(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminDashboardResponse>> {
  is_admin(&local_user_view)?;
  // ... body
}
```

For SSE (note: returns `HttpResponse`, NOT `LemmyResult<Json<...>>` — streaming body can't fit in a Json extractor):
```rust
pub async fn admin_audit_stream(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> actix_web::Result<HttpResponse> {
  is_admin(&local_user_view).map_err(actix_web::error::ErrorForbidden)?;
  // ... acquire dedicated pg conn, build stream, return streaming response
}
```

**AGGREGATE_HANDLER (sequential reads, shared ConfigCache):**
```rust
// SOURCE: crates/api/api/src/governance/admin_reputation_stats.rs:76-136
// COPY THIS PATTERN:
is_admin(&local_user_view)?;

let mut cache = ConfigCache::new();
let mut pool = context.pool();
let conn = &mut get_conn(&mut pool).await?;

let community_bind: Option<i32> = data.community_id.map(|c| c.0);

// Step 1 — config reads via `get_int`
let thresholds_current = ThresholdsSnapshot { /* 3 get_int calls */ };

// Step 2 — inline count queries
let buckets = ReputationBuckets {
  reporting_accuracy: bucket_query(conn, "reporting_accuracy", community_bind).await?,
  // ...
};

let capability_counts = capability_query(conn, community_bind).await?;
let founder_event_stats = founder_query(conn, community_bind).await?;

Ok(Json(AdminReputationStatsResponse { /* ... */ }))
```

For dashboard: ~8 sequential reads on one `conn`, each with its own `sql_query` or ORM call.

**INLINE_COUNT_QUERY (reuse `SingleCountRow` + `sql_query`):**
```rust
// SOURCE: crates/api/api/src/governance/admin_config.rs:252-261
// COPY THIS PATTERN:
let sql = "\
   SELECT COUNT(*)::bigint AS c \
   FROM moderation_case \
   WHERE status IN ('JurySelection', 'InReview') \
     AND community_id IS NOT DISTINCT FROM $1";

let row: SingleCountRow = sql_query(sql)
  .bind::<Nullable<Integer>, _>(community_bind)
  .get_result(conn)
  .await?;
```

Dashboard uses this for `active_cases.by_status` (one query with `GROUP BY status` — one round trip), `jury_queue` (one query with `FILTER (WHERE status = ...)`), `federation` (one query with `FILTER`), `rule_sets.communities_with_rule_sets` (one query with `COUNT(DISTINCT)`), and `rule_sets.total_versions` (one `COUNT(*)`).

**GOVERNANCE_LOG_LIST (recent config changes widget):**
```rust
// SOURCE: crates/api/api/src/governance/admin_config.rs:1187-1213
// COPY THIS PATTERN (stripped down — no pagination, no filters):
let rows: Vec<GovernanceLog> = governance_log_schema::table
  .filter(governance_log_schema::entry_kind.eq_any(vec![
    ENTRY_KIND_ADMIN_CONFIG_CHANGED,
    ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED,
  ]))
  .order_by((
    governance_log_schema::created_at.desc(),
    governance_log_schema::id.desc(),
  ))
  .limit(20)
  .select(GovernanceLog::as_select())
  .load::<GovernanceLog>(conn)
  .await?;

let entries: Vec<AdminConfigAuditEntry> = rows
  .into_iter()
  .map(project_to_audit_entry)
  .collect();
```

**AUDIT_PROJECTION (reused verbatim):**
```rust
// SOURCE: crates/api/api/src/governance/admin_config.rs:1301-1353
// FULL FUNCTION TO MOVE TO crates/api/api/src/governance/audit_projection.rs
// (promote from `fn` to `pub(crate) fn`)
pub(crate) fn project_to_audit_entry(row: GovernanceLog) -> AdminConfigAuditEntry {
  let payload = &row.payload;
  let scope = payload.get("scope").and_then(|v| v.as_str()).unwrap_or("").to_string();
  let key = payload.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
  let value_type = payload.get("value_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
  let new_value = payload.get("value").cloned().unwrap_or(Value::Null);
  let previous_value = payload.get("previous_value").cloned();
  let previous_from = payload.get("previous_from").and_then(|v| v.as_str()).map(str::to_owned);
  let reason = payload.get("reason").and_then(|v| v.as_str()).unwrap_or("").to_string();
  let denial_reason = if row.entry_kind == ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED {
    payload.get("denial_reason").and_then(|v| v.as_str()).map(str::to_owned)
  } else {
    None
  };

  AdminConfigAuditEntry {
    id: row.id.0,
    entry_kind: row.entry_kind,
    scope,
    key,
    value_type,
    previous_value,
    previous_from,
    new_value,
    reason,
    actor_pseudonym: row.actor_pseudonym,
    created_at: row.created_at,
    signature: row.signature,
    denial_reason,
  }
}
```

**SSE_HAND_ROLLED_STREAM (NEW pattern — no existing mirror):**
```rust
// NEW PATTERN (crates/api/api/src/governance/admin_audit_stream.rs)
// Based on: async-stream 0.3 docs + tokio-postgres 0.7 AsyncMessage
// References: futures_util::Stream (workspace line 215), actix-web HttpResponse::streaming
use actix_web::{HttpResponse, web::{Bytes, Data}};
use async_stream::stream;
use futures_util::StreamExt;
use serde_json::json;
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tokio::time::interval;
use tokio_postgres::AsyncMessage;

// Module-level per-admin SSE-cap guard. Arc<Mutex<HashSet<_>>> keeps v1 simple
// (single-node solo-dev instance). Future: upgrade to DashMap or shared cache.
static ACTIVE_SSE_ADMINS: once_cell::sync::Lazy<Arc<Mutex<std::collections::HashSet<PersonId>>>>
  = once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(std::collections::HashSet::new())));

struct SseGuard {
  admin_id: PersonId,
}

impl Drop for SseGuard {
  fn drop(&mut self) {
    // Fire-and-forget cleanup. Use try_lock to avoid panicking in Drop.
    let admin_id = self.admin_id;
    tokio::spawn(async move {
      let mut set = ACTIVE_SSE_ADMINS.lock().await;
      set.remove(&admin_id);
    });
  }
}

pub async fn admin_audit_stream(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> actix_web::Result<HttpResponse> {
  is_admin(&local_user_view).map_err(actix_web::error::ErrorForbidden)?;

  let admin_id = local_user_view.person.id;

  // Enforce per-admin cap
  {
    let mut set = ACTIVE_SSE_ADMINS.lock().await;
    if set.contains(&admin_id) {
      return Err(actix_web::error::ErrorConflict(
        "another SSE stream is already open for this admin",
      ));
    }
    set.insert(admin_id);
  }
  let guard = SseGuard { admin_id };

  // Open a DEDICATED tokio_postgres connection (NOT via the pool — pooled
  // conns return after each await, breaking LISTEN's semantics)
  let db_url = context.settings().get_database_url();
  let (client, mut connection) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

  // Intercept notifications before the driver loops them to /dev/null
  let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<tokio_postgres::Notification>();
  let driver = tokio::spawn(async move {
    loop {
      match futures_util::future::poll_fn(|cx| connection.poll_message(cx)).await {
        Some(Ok(AsyncMessage::Notification(n))) => {
          let _ = tx.send(n);
        }
        Some(Ok(_)) => continue,
        Some(Err(_)) | None => break,
      }
    }
  });

  client
    .batch_execute("LISTEN governance_events")
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

  // Capture a pool handle for id-based fetches inside the stream
  let pool_handle = context.pool().clone();

  let body = stream! {
    yield Ok::<Bytes, actix_web::Error>(Bytes::from("event: retry\ndata: 10000\n\n"));

    let mut heartbeat = interval(Duration::from_secs(15));
    heartbeat.tick().await;  // fire first tick immediately, then every 15s

    loop {
      tokio::select! {
        _ = heartbeat.tick() => {
          yield Ok(Bytes::from(": keepalive\n\n"));
        }
        maybe_note = rx.recv() => {
          let Some(note) = maybe_note else { break };
          if note.channel() != "governance_events" { continue }

          // Deserialise payload: {"entry_id": i64, "kind": String, "created_at": RFC3339}
          let Ok(parsed): Result<serde_json::Value, _> = serde_json::from_str(note.payload()) else { continue };
          let Some(kind) = parsed.get("kind").and_then(|v| v.as_str()) else { continue };

          // Filter: only the two admin-config kinds
          if kind != ENTRY_KIND_ADMIN_CONFIG_CHANGED && kind != ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED {
            continue;
          }

          let Some(entry_id) = parsed.get("entry_id").and_then(|v| v.as_i64()) else { continue };

          // Fetch the full row from the pool via a short-lived pooled conn
          let Ok(pool_conn) = pool_handle.clone().get().await else { continue };
          let mut conn = pool_conn;
          let row: Option<GovernanceLog> = governance_log_schema::table
            .filter(governance_log_schema::id.eq(GovernanceLogId(entry_id)))
            .select(GovernanceLog::as_select())
            .first(&mut *conn).await.optional().ok().flatten();

          let Some(row) = row else { continue };
          let entry = project_to_audit_entry(row);
          let Ok(json) = serde_json::to_string(&entry) else { continue };

          yield Ok(Bytes::from(format!("event: {kind}\ndata: {json}\n\n")));
        }
      }
    }

    // Drop guard when stream ends
    drop(guard);
    drop(driver);  // joins the driver task implicitly via Drop
  };

  Ok(HttpResponse::Ok()
    .content_type("text/event-stream")
    .insert_header(("Cache-Control", "no-cache, no-transform"))
    .insert_header(("X-Accel-Buffering", "no"))  // nginx hint
    .streaming(body))
}
```

**Notes on the SSE pattern (will be refined at impl time):**
- `tokio_postgres::NoTls` vs `MakeRustlsConnect` branch must mirror `crates/diesel_utils/src/connection.rs:203-210` if `DATABASE_URL` contains `sslmode=require`. Task 3 documents the TLS branch.
- `pool_handle.clone().get()` pattern comes from the existing handlers' `context.pool()` usage. Exact syntax is `LemmyContext::pool()` — verify at impl.
- `once_cell::sync::Lazy` may require a new workspace dep — check Cargo.toml first; if absent, use `std::sync::OnceLock<Mutex<...>>` (stable since 1.70, no new dep).
- The `rx.recv()` + `heartbeat.tick()` `tokio::select!` pattern ensures a client sees either a real event or a keepalive within 15s, preventing nginx/cloudflare idle-timeout drops.

**CONFIG_CACHE_USAGE (for reputation widget):**
```rust
// SOURCE: crates/api/api/src/governance/admin_reputation_stats.rs:82
// COPY THIS PATTERN:
let mut cache = ConfigCache::new();
// Pass `&mut cache` to every get_int/get_int_opt/get_bool_opt/get_text_opt call
```

**DTO_SHAPE (new dashboard DTOs in api_common):**
```rust
// SOURCE: crates/api/api_common/src/governance.rs (AdminReputationStatsResponse pattern)
// COPY THIS SHAPE:
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminDashboardResponse {
  pub active_cases: ActiveCasesSummary,
  pub jury_queue: JuryQueueSummary,
  pub recent_config_changes: Vec<AdminConfigAuditEntry>,
  pub federation: FederationSummary,
  pub reputation: AdminReputationStatsResponse,
  pub rule_sets: RuleSetSummary,
  pub calculated_at: DateTime<Utc>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ActiveCasesSummary {
  pub by_status: std::collections::BTreeMap<String, i64>,
  pub total_active: i64,  // open + threshold_met + jury_selection + in_review + appealed + admin_review
}

// JuryQueueSummary, FederationSummary, RuleSetSummary, PerCommunityActiveRuleSet — same shape
```

**ROUTE_REGISTRATION:**
```rust
// SOURCE: crates/api/routes/src/lib.rs:533-549
// EXTEND THIS BLOCK:
scope("/admin")
  .route("/assign-jury", post().to(admin_assign_jury))
  .route("/close-case", post().to(admin_close_case))
  .route("/reputation-stats", get().to(admin_reputation_stats))
  .route("/dashboard", get().to(admin_dashboard))    // v1-AD-d NEW
  .service(
    scope("/config")
      .route("", post().to(admin_set_config))
      .route("", get().to(admin_get_config))
      .route("/audit", get().to(admin_get_config_audit)),
  )
  .service(
    scope("/rule-sets")
      .route("", post().to(admin_create_rule_set))
      .route("", get().to(admin_list_rule_sets)),
  )
  .service(
    scope("/audit")
      .route("/stream", get().to(admin_audit_stream)),  // v1-AD-d NEW
  ),
```

**E2E_TEST_SHAPE (dashboard):**
```rust
// SOURCE: crates/server/tests/e2e.rs:4800-4841 (admin_get_config_full mirror)
// COPY THIS PATTERN:
#[tokio::test(flavor = "multi_thread")]
async fn admin_dashboard_returns_aggregate() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Data;
  use lemmy_api::governance::admin_dashboard::admin_dashboard;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_dash", true).await?;

  let resp = admin_dashboard(context.clone(), admin_view).await?.into_inner();

  // Every widget populates — zero-row state is valid (empty arrays, zero counts)
  assert!(resp.active_cases.by_status.contains_key("Open"));
  assert_eq!(resp.recent_config_changes.len(), 0, "no config-change events in fresh DB");
  // ... etc
  Ok(())
}
```

**E2E_TEST_SHAPE (SSE — uses reqwest with `stream` feature):**
```rust
// NEW PATTERN (crates/server/tests/e2e.rs)
// Uses reqwest::Client::get(...).send().bytes_stream()
#[tokio::test(flavor = "multi_thread")]
async fn admin_audit_stream_emits_config_change() -> lemmy_utils::error::LemmyResult<()> {
  use futures_util::StreamExt;
  use tokio::time::{Duration, timeout};

  let (server, context, admin_auth) = start_test_server_with_admin().await?;
  let client = reqwest::Client::new();

  let resp = client
    .get(format!("http://{}/api/v4/governance/admin/audit/stream", server.addr()))
    .header("Authorization", format!("Bearer {}", admin_auth))
    .send()
    .await?;

  assert_eq!(resp.status(), 200);
  assert_eq!(resp.headers().get("content-type").unwrap(), "text/event-stream");

  let mut stream = resp.bytes_stream();
  // Expect the retry frame within 1s
  let first = timeout(Duration::from_secs(1), stream.next()).await?.unwrap()?;
  let first_str = std::str::from_utf8(&first)?;
  assert!(first_str.contains("event: retry"));

  // Trigger a config change in parallel
  tokio::spawn(async move {
    admin_config_fixtures::write_config(&context, "test.key", "val").await;
  });

  // Within 5s, expect an event: admin_config_changed frame
  let deadline = Duration::from_secs(5);
  let found = timeout(deadline, async {
    while let Some(chunk) = stream.next().await {
      let chunk = chunk?;
      let s = std::str::from_utf8(&chunk)?;
      if s.contains("event: admin_config_changed") {
        return Ok::<bool, LemmyError>(true);
      }
    }
    Ok(false)
  }).await??;
  assert!(found, "SSE did not emit admin_config_changed within deadline");
  Ok(())
}
```

---

## 11. Files to change

| File | Action | Justification |
|---|---|---|
| `Cargo.toml` (root) | UPDATE | Add `async-stream = "0.3"` to `[workspace.dependencies]`; add `"stream"` to reqwest feature list at workspace line 187–191 |
| `crates/api/api/Cargo.toml` | UPDATE | Add `async-stream = { workspace = true }` + `tokio-postgres = { workspace = true }` to `[dependencies]` + `once_cell` if absent (check first) |
| `crates/server/Cargo.toml` (dev-dependencies) | UPDATE | Ensure `reqwest` with `stream` feature is available for the SSE e2e test |
| `crates/api/api_common/src/governance.rs` | UPDATE | Add DTOs: `AdminDashboardResponse`, `ActiveCasesSummary`, `JuryQueueSummary`, `FederationSummary`, `RuleSetSummary`, `PerCommunityActiveRuleSet` |
| `crates/api/api/src/governance/audit_projection.rs` | CREATE | New module hosting `project_to_audit_entry` (moved from `admin_config.rs`). `pub(crate)` visibility. Original site imports the new path. |
| `crates/api/api/src/governance/admin_config.rs` | UPDATE | Remove `project_to_audit_entry` function body; import from new `audit_projection` module. No behaviour change. |
| `crates/api/api/src/governance/admin_dashboard.rs` | CREATE | `admin_dashboard` handler + private helpers: `count_active_cases_by_status`, `count_jury_queue`, `list_recent_config_changes`, `federation_summary`, `rule_sets_summary` |
| `crates/api/api/src/governance/admin_audit_stream.rs` | CREATE | `admin_audit_stream` handler + the SSE guard struct + driver-task pattern + LISTEN wiring |
| `crates/api/api/src/governance/mod.rs` | UPDATE | `pub mod admin_dashboard; pub mod admin_audit_stream; pub(crate) mod audit_projection;` declarations |
| `crates/api/routes/src/lib.rs` | UPDATE | Add `/admin/dashboard` route and `/admin/audit/stream` nested scope inside `/admin` (lines 533–549) |
| `crates/server/tests/e2e.rs` | UPDATE | Add 5 e2e tests: dashboard happy path (admin), dashboard forbidden (non-admin), SSE happy path (retry frame + live event), SSE forbidden (non-admin), SSE per-admin-cap (409 on second connection from same admin) |

Total: **11 file edits** (5 CREATE, 6 UPDATE). Zero migration changes. Zero entry-kind additions.

---

## 12. NOT building in v1-AD-d

- **No askama / HTML pages.** v1-AD-e or v1.x; OQ-V1-AD-01 resolved.
- **No new migrations.** The `governance_log_notify` trigger and all source tables shipped in v1-AD-a/b/c.
- **No new entry kinds.** Registry count stays at 26.
- **No cross-community dashboard scope.** Dashboard is instance-wide only. Per-community views are a v1.x or v1-AD-e scope.
- **No auth-log widget.** Failed `is_admin` attempts on any endpoint are NOT surfaced in the dashboard — they don't emit governance_log entries today and doing so is a separate concern (audit-logging for non-config endpoints is a broader sub-PRD if pilots ask).
- **No `tower-sse` / `actix-web-lab` dep.** Hand-rolled via `async-stream` per OQ-V1-AD-02.
- **No SSE event replay / persistence.** Client reconnect fetches catch-up via `/admin/config/audit` separately. Retry header on connect is the only client-side hint.
- **No distributed SSE cap.** The per-admin guard is in-memory on a single node. Multi-node instance support is a v2 concern.
- **No custom rate limit on SSE.** The endpoint inherits `rate_limit.post()` from the parent `/governance` scope (`crates/api/routes/src/lib.rs:518`). If pilots report bot-abuse, a `.wrap(rate_limit.X())` override is a 3-line follow-up.
- **No caching of dashboard response.** 6–8 DB round-trips per GET is acceptable; if pilots report slowness, a `moka` or similar in-memory cache with 5s TTL is a v1.x follow-up.
- **No sub-second timing in the dashboard response.** `calculated_at` is good enough; a benchmark of the aggregate's wall-clock latency is a retro concern.
- **No websocket alternative.** SSE is sufficient for one-way server-push; websockets add bidirectional surface that the use case doesn't need.
- **No `governance_log` write on SSE connect / disconnect.** This is a read-only endpoint; connecting is not a governance event. (Future: v2 could log SSE access as an `admin_audit_stream_connected` kind if audit regulation requires it.)
- **No reputation widget at per-community scope.** The dashboard aggregator always calls the reputation helpers at `Scope::Instance`. Per-community reputation is still available via the existing `/admin/reputation-stats?community_id=X` endpoint.
- **No `federation_outbox` / `sent_activity` pending count.** Per §4.1 load-bearing decision; add if pilots ask.
- **No backfill of any data.** Pre-existing governance_log rows are included in `recent_config_changes` automatically because they're already in the table.
- **No EXPECTED_SEED_COUNT changes.** v1-AD-d ships zero seed rows; count stays at 61.

---

## 13. Step-by-step tasks

Execute in order. One commit per task (except task 0). Each task has a MIRROR reference, an exact file path, and a validation command. Task 0 is **mandatory** per `.claude/rules/pre-phase-harness-audit.md` and runs BEFORE any implementation.

### Task 0 — Pre-phase harness audit + branch check (MANDATORY, pre-code)

- **ACTION**:
  1. Confirm branch: `git branch --show-current` → must print `phase-v1-AD-d`. If currently on `plan/v1-AD-d`, the advisor or this session will cut `phase-v1-AD-d` from the plan-merge commit per `.claude/rules/phase-branch.md`. If the branch doesn't exist, STOP and file DQ.
  2. Confirm base commit: `git log -1 --format=%H governance-v0` should be `cf89890f3` (post-PR-#81 merge) or a later governance-v0 HEAD if the plan itself merges first. Note the exact base in the task 0 report.
  3. Confirm v1-AD-c surfaces exist (all these greps must have ≥1 match):
     - `rg '^pub async fn admin_create_rule_set' crates/api/api/src/governance/admin_rule_sets.rs` → 1 match
     - `rg '^pub async fn admin_list_rule_sets' crates/api/api/src/governance/admin_rule_sets.rs` → 1 match
     - `rg '^fn project_to_audit_entry' crates/api/api/src/governance/admin_config.rs` → 1 match
     - `rg 'governance_log_notify_trigger' migrations/2026-04-20-000000-0000_add_governance_log_notify/up.sql` → 1 match
     - `rg '^pub const ENTRY_KIND_ADMIN_CONFIG_CHANGED' crates/db_schema/src/source/governance/governance_log.rs` → 1 match
     - `rg '^pub async fn admin_reputation_stats' crates/api/api/src/governance/admin_reputation_stats.rs` → 1 match
     - `rg '^async fn bucket_query' crates/api/api/src/governance/admin_reputation_stats.rs` → 1 match (helper we'll reuse)
  4. Run all four probes from `.claude/rules/pre-phase-harness-audit.md`:
     - Probe 1 (`-p` crate scoping): `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api > .claude/audit-cargo-check-p.log 2>&1"` → tail expects only `lemmy_api` compilation
     - Probe 2 (features activation): `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/audit-cargo-check-features.log 2>&1"`
     - Probe 3 (test target scoping): `cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/audit-cargo-test.log 2>&1"`
     - Probe 4 (negative exit-code propagation): `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-check-negative.log 2>&1"; echo "exit: $?"` → expect **non-zero exit** (typically 101)
  5. Clippy baseline capture (for task 5 delta check):
     - Narrowed command (v1-AD-c ratchet — production-code only): `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/audit-clippy-narrowed.log 2>&1"; echo "exit: $?"` → **expect exit 0** (matches v1-AD-c baseline from commit 3ca80e736)
     - All-targets superset (informational only): `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --all-targets --features full --no-deps -- -D warnings > .claude/audit-clippy-baseline.log 2>&1"; echo "exit: $?"` → expect red; record error count as baseline N.
  6. Verify `governance_log_notify_trigger` actually exists in test DB (sanity check — probe 3 built the e2e binary but didn't run it):
     - Start a scratch container + migrate: `scripts/brehon/spin-up-ephemeral-db.sh` (or the existing e2e harness bootstrap script). In psql, run `\df governance_log_notify` and `\d+ governance_log_notify_trigger`. Both must exist. Cleanup container afterward.
- **VALIDATE**: all six points pass; if any probe or the narrowed clippy baseline fails, STOP, file DQ, do not start task 1.
- **COMMIT**: none. Task 0 is read-only probe discipline.
- **OUTPUT**: append `.claude/PRPs/reports/v1-AD-d-task0-audit.md` with probe results, exit codes, baseline counts, and the `\df governance_log_notify` confirmation.

### Task 1 — Extract `project_to_audit_entry` to a shared module

- **ACTION**:
  1. Create `crates/api/api/src/governance/audit_projection.rs` containing the `project_to_audit_entry` function verbatim from `admin_config.rs:1301-1353`. Make it `pub(crate) fn` (not `pub` — v1-AD-d callers are both in the same `lemmy_api` crate).
  2. Port the imports the function needs: `lemmy_api_common::governance::AdminConfigAuditEntry`, `lemmy_db_schema::source::governance::governance_log::GovernanceLog`, `crate::governance::governance_log::ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED`, `serde_json::Value`. Copy verbatim from the admin_config.rs import block.
  3. Remove the function body from `admin_config.rs` (delete lines 1301–1353).
  4. In `admin_config.rs`, add `use crate::governance::audit_projection::project_to_audit_entry;` near the top-of-file imports. Existing call sites (line ~1216) now resolve via the import.
  5. Register `pub(crate) mod audit_projection;` in `crates/api/api/src/governance/mod.rs` (alphabetical among existing `pub mod`).
- **MIRROR**: §10 "AUDIT_PROJECTION" snippet; the function body is unchanged.
- **GOTCHA**: a grep for `project_to_audit_entry` across the workspace before and after the move must return identical hit counts (just different file paths). If task 4 (below) adds a third caller, task 1 anticipates that by making the visibility `pub(crate)` on day one, not `fn` (private).
- **GOTCHA (test impact)**: any v1-AD-b / v1-AD-c e2e test that constructed an `AdminConfigAuditEntry` and called `project_to_audit_entry` directly (unlikely — tests go through HTTP) needs its import updated. Run `rg 'project_to_audit_entry' crates/server/tests/e2e.rs` before task 1 to catch this; expected hit count is 0 for tests (they invoke the handler, not the projection).
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task1.log 2>&1"; echo "exit: $?"
  tail -20 .claude/build-task1.log
  # All v1-AD-b audit-handler tests still pass (behaviour unchanged):
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/test-task1.log 2>&1"; echo "exit: $?"
  rg 'project_to_audit_entry' crates/api/ crates/server/tests/ | wc -l
  # Expected: same count as before move (caller count unchanged)
  ```
- **COMMIT MESSAGE**: `refactor(governance): extract project_to_audit_entry to shared audit_projection module (task 1)`

### Task 2 — Add dashboard DTOs to `api_common`

- **ACTION**: Edit `crates/api/api_common/src/governance.rs`:
  - Add `AdminDashboardResponse`, `ActiveCasesSummary`, `JuryQueueSummary`, `FederationSummary`, `RuleSetSummary`, `PerCommunityActiveRuleSet` structs near the bottom of the file (after the v1-AD-c rule-set DTOs).
  - Every struct gets the canonical v1-AD-b derive block: `#[skip_serializing_none]`, `#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]` (Eq is safe since all fields are `i64`/`i32`/`String`/`BTreeMap<String, i64>`/`Vec<_>`; the outer `AdminDashboardResponse` drops `Eq` because it embeds `AdminReputationStatsResponse` which is `Eq`, fine, and `DateTime<Utc>` which is `Eq`, fine — but `Vec<AdminConfigAuditEntry>` contains `serde_json::Value` which is NOT `Eq`, so `AdminDashboardResponse` gets `PartialEq` only, NOT `Eq`).
  - `#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]` + `#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]` on every struct.
- **MIRROR**: §10 DTO_SHAPE snippet; v1-AD-b DTOs at `crates/api/api_common/src/governance.rs:339-345` and v1-AD-c DTOs at the end of the file.
- **IMPLEMENT**:
  ```rust
  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  pub struct AdminDashboardResponse {
    pub active_cases: ActiveCasesSummary,
    pub jury_queue: JuryQueueSummary,
    pub recent_config_changes: Vec<AdminConfigAuditEntry>,
    pub federation: FederationSummary,
    pub reputation: AdminReputationStatsResponse,
    pub rule_sets: RuleSetSummary,
    pub calculated_at: DateTime<Utc>,
  }

  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  pub struct ActiveCasesSummary {
    /// Keys: PascalCase `CaseStatus` variants ("Open", "ThresholdMet", ...).
    /// BTreeMap (not HashMap) so serialisation order is stable for golden tests.
    pub by_status: std::collections::BTreeMap<String, i64>,
    /// Sum of counts for all variants EXCEPT Decided, Closed, EmergencyRemove.
    pub total_active: i64,
  }

  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  pub struct JuryQueueSummary {
    /// JuryAssignment.status = 'Selected' (juror notified, not yet responded).
    pub pending_accept: i64,
    /// JuryAssignment.status = 'Accepted'.
    pub accepted: i64,
    /// JuryAssignment.status = 'Submitted' (vote cast).
    pub submitted: i64,
  }

  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  pub struct FederationSummary {
    pub active: i64,
    pub expired: i64,
    pub total: i64,
  }

  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  pub struct RuleSetSummary {
    pub communities_with_rule_sets: i64,
    pub total_versions: i64,
    /// Bounded to 100 entries per §4.1 load-bearing decision.
    pub per_community: Vec<PerCommunityActiveRuleSet>,
  }

  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  pub struct PerCommunityActiveRuleSet {
    pub community_id: i32,
    /// `None` if the community has `rule_set_version` rows but no `rule_set.active_version_id`
    /// config row (allowed by design — v1-AD-c never seeds the key).
    pub active_version_id: Option<i32>,
  }
  ```
- **GOTCHA**: `AdminDashboardResponse` drops `Eq` because `AdminConfigAuditEntry.new_value` is `serde_json::Value` (no `Eq` impl) and `AdminConfigAuditEntry.previous_value` is `Option<serde_json::Value>`. Derive `PartialEq` only on the outer. Compiler catches this.
- **GOTCHA**: `PerCommunityActiveRuleSet.community_id` is `i32`, not `CommunityId` — the wire representation is simpler and matches what `rule_set_version.community_id.0` extracts. If a pilot wants the typed newtype, they can re-wrap client-side.
- **VALIDATE**: `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_common --features full > .claude/build-task2.log 2>&1"; echo "exit: $?"`
- **COMMIT MESSAGE**: `feat(api-common): AdminDashboardResponse + aggregate DTOs (task 2)`

### Task 3 — `admin_dashboard` handler

- **ACTION**:
  1. Create `crates/api/api/src/governance/admin_dashboard.rs` containing:
     - The `admin_dashboard` pub handler (takes `Data<LemmyContext>` + `LocalUserView`, returns `LemmyResult<Json<AdminDashboardResponse>>`)
     - Six private async helper fns:
       - `count_active_cases(conn: &mut AsyncPgConnection) -> LemmyResult<ActiveCasesSummary>`
       - `count_jury_queue(conn: &mut AsyncPgConnection) -> LemmyResult<JuryQueueSummary>`
       - `list_recent_config_changes(conn: &mut AsyncPgConnection) -> LemmyResult<Vec<AdminConfigAuditEntry>>`
       - `federation_summary(conn: &mut AsyncPgConnection) -> LemmyResult<FederationSummary>`
       - `rule_sets_summary(conn: &mut AsyncPgConnection, cache: &mut ConfigCache, pool: &mut DbPool<'_>) -> LemmyResult<RuleSetSummary>`
       - `reputation_instance_scope(conn: &mut AsyncPgConnection, cache: &mut ConfigCache) -> LemmyResult<AdminReputationStatsResponse>` — wraps the three `bucket_query`/`capability_query`/`founder_query` imports at `Scope::Instance`
  2. Register `pub mod admin_dashboard;` in `crates/api/api/src/governance/mod.rs`.
- **MIRROR**:
  - §10 CAPABILITY_GATE for the handler entry
  - §10 AGGREGATE_HANDLER for the body shape
  - §10 INLINE_COUNT_QUERY for per-widget count queries
  - §10 GOVERNANCE_LOG_LIST for `list_recent_config_changes`
  - `admin_reputation_stats.rs:76-136` for the overall handler shape
- **IMPLEMENT (handler skeleton)**:
  ```rust
  // crates/api/api/src/governance/admin_dashboard.rs
  use actix_web::web::{Data, Json};
  use chrono::Utc;
  use diesel::{
    ExpressionMethods, QueryDsl, QueryableByName, SelectableHelper,
    sql_query,
    sql_types::{BigInt, Integer, Nullable, Text},
  };
  use diesel_async::{AsyncPgConnection, RunQueryDsl};
  use lemmy_api_common::governance::{
    ActiveCasesSummary, AdminConfigAuditEntry, AdminDashboardResponse,
    AdminReputationStatsResponse, CapabilityCounts, FederationSummary,
    FounderEventStats, JuryQueueSummary, PerCommunityActiveRuleSet,
    ReputationBuckets, RuleSetSummary, ThresholdsSnapshot,
  };
  use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
  use lemmy_db_schema::source::governance::governance_log::GovernanceLog;
  use lemmy_db_schema_file::schema::{
    governance_log as governance_log_schema, rule_set_version,
  };
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_diesel_utils::connection::{DbPool, get_conn};
  use lemmy_utils::error::LemmyResult;
  use std::collections::BTreeMap;

  use crate::governance::{
    admin_reputation_stats::{bucket_query, capability_query, founder_query},
    audit_projection::project_to_audit_entry,
    config::{self, ConfigCache, Scope, get_int},
    governance_log::{ENTRY_KIND_ADMIN_CONFIG_CHANGED, ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED},
  };

  pub async fn admin_dashboard(
    context: Data<LemmyContext>,
    local_user_view: LocalUserView,
  ) -> LemmyResult<Json<AdminDashboardResponse>> {
    is_admin(&local_user_view)?;

    let mut cache = ConfigCache::new();
    let mut pool = context.pool();
    let conn = &mut get_conn(&mut pool).await?;

    let active_cases = count_active_cases(conn).await?;
    let jury_queue = count_jury_queue(conn).await?;
    let recent_config_changes = list_recent_config_changes(conn).await?;
    let federation = federation_summary(conn).await?;
    let reputation = reputation_instance_scope(conn, &mut cache).await?;
    let rule_sets = rule_sets_summary(conn, &mut cache, &mut pool).await?;

    Ok(Json(AdminDashboardResponse {
      active_cases,
      jury_queue,
      recent_config_changes,
      federation,
      reputation,
      rule_sets,
      calculated_at: Utc::now(),
    }))
  }

  #[derive(QueryableByName)]
  struct StatusCountRow {
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = BigInt)]
    c: i64,
  }

  async fn count_active_cases(conn: &mut AsyncPgConnection) -> LemmyResult<ActiveCasesSummary> {
    let sql = "\
       SELECT status::text AS status, COUNT(*)::bigint AS c \
       FROM moderation_case \
       GROUP BY status";
    let rows: Vec<StatusCountRow> = sql_query(sql).load(conn).await?;
    let by_status: BTreeMap<String, i64> = rows.into_iter().map(|r| (r.status, r.c)).collect();

    // total_active = sum of all except Decided, Closed, EmergencyRemove
    let total_active: i64 = by_status
      .iter()
      .filter(|(k, _)| !matches!(k.as_str(), "Decided" | "Closed" | "EmergencyRemove"))
      .map(|(_, v)| *v)
      .sum();

    Ok(ActiveCasesSummary { by_status, total_active })
  }

  #[derive(QueryableByName)]
  struct JuryCountRow {
    #[diesel(sql_type = BigInt)]
    pending_accept: i64,
    #[diesel(sql_type = BigInt)]
    accepted: i64,
    #[diesel(sql_type = BigInt)]
    submitted: i64,
  }

  async fn count_jury_queue(conn: &mut AsyncPgConnection) -> LemmyResult<JuryQueueSummary> {
    let sql = "\
       SELECT \
         COUNT(*) FILTER (WHERE status = 'Selected')::bigint  AS pending_accept, \
         COUNT(*) FILTER (WHERE status = 'Accepted')::bigint  AS accepted, \
         COUNT(*) FILTER (WHERE status = 'Submitted')::bigint AS submitted \
       FROM jury_assignment";
    let row: JuryCountRow = sql_query(sql).get_result(conn).await?;
    Ok(JuryQueueSummary {
      pending_accept: row.pending_accept,
      accepted: row.accepted,
      submitted: row.submitted,
    })
  }

  async fn list_recent_config_changes(
    conn: &mut AsyncPgConnection,
  ) -> LemmyResult<Vec<AdminConfigAuditEntry>> {
    let rows: Vec<GovernanceLog> = governance_log_schema::table
      .filter(governance_log_schema::entry_kind.eq_any(vec![
        ENTRY_KIND_ADMIN_CONFIG_CHANGED,
        ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED,
      ]))
      .order_by((
        governance_log_schema::created_at.desc(),
        governance_log_schema::id.desc(),
      ))
      .limit(20)
      .select(GovernanceLog::as_select())
      .load(conn)
      .await?;
    Ok(rows.into_iter().map(project_to_audit_entry).collect())
  }

  #[derive(QueryableByName)]
  struct FederationCountRow {
    #[diesel(sql_type = BigInt)]
    active: i64,
    #[diesel(sql_type = BigInt)]
    expired: i64,
    #[diesel(sql_type = BigInt)]
    total: i64,
  }

  async fn federation_summary(conn: &mut AsyncPgConnection) -> LemmyResult<FederationSummary> {
    let sql = "\
       SELECT \
         COUNT(*) FILTER (WHERE valid_until IS NULL OR valid_until > now())::bigint AS active, \
         COUNT(*) FILTER (WHERE valid_until IS NOT NULL AND valid_until <= now())::bigint AS expired, \
         COUNT(*)::bigint AS total \
       FROM federation_attestation";
    let row: FederationCountRow = sql_query(sql).get_result(conn).await?;
    Ok(FederationSummary {
      active: row.active,
      expired: row.expired,
      total: row.total,
    })
  }

  #[derive(QueryableByName)]
  struct RuleSetAggregateRow {
    #[diesel(sql_type = BigInt)]
    communities_with_rule_sets: i64,
    #[diesel(sql_type = BigInt)]
    total_versions: i64,
  }

  #[derive(QueryableByName)]
  struct CommunityIdRow {
    #[diesel(sql_type = Integer)]
    community_id: i32,
  }

  async fn rule_sets_summary(
    conn: &mut AsyncPgConnection,
    cache: &mut ConfigCache,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<RuleSetSummary> {
    let agg: RuleSetAggregateRow = sql_query(
      "SELECT \
         COUNT(DISTINCT community_id)::bigint AS communities_with_rule_sets, \
         COUNT(*)::bigint AS total_versions \
       FROM rule_set_version",
    )
    .get_result(conn)
    .await?;

    // Per-community list (bounded to 100)
    let community_ids: Vec<CommunityIdRow> = sql_query(
      "SELECT DISTINCT community_id \
       FROM rule_set_version \
       ORDER BY community_id \
       LIMIT 100",
    )
    .load(conn)
    .await?;

    let mut per_community = Vec::with_capacity(community_ids.len());
    for r in community_ids {
      let cid = lemmy_db_schema::newtypes::CommunityId(r.community_id);
      let active_version_id = config::get_int_opt(
        cache,
        pool,
        Scope::Community(cid),
        "rule_set.active_version_id",
      )
      .await?
      .and_then(|i| i32::try_from(i).ok());
      per_community.push(PerCommunityActiveRuleSet {
        community_id: r.community_id,
        active_version_id,
      });
    }

    Ok(RuleSetSummary {
      communities_with_rule_sets: agg.communities_with_rule_sets,
      total_versions: agg.total_versions,
      per_community,
    })
  }

  async fn reputation_instance_scope(
    conn: &mut AsyncPgConnection,
    cache: &mut ConfigCache,
  ) -> LemmyResult<AdminReputationStatsResponse> {
    let thresholds_current = ThresholdsSnapshot {
      jury_reliability: i32::try_from(
        get_int(cache, &mut (&mut *conn).into(), Scope::Instance, "thresholds.jury_reliability").await?,
      ).unwrap_or(0),
      reporting_accuracy: i32::try_from(
        get_int(cache, &mut (&mut *conn).into(), Scope::Instance, "thresholds.reporting_accuracy").await?,
      ).unwrap_or(0),
      endorsement_strength: i32::try_from(
        get_int(cache, &mut (&mut *conn).into(), Scope::Instance, "thresholds.endorsement_strength").await?,
      ).unwrap_or(0),
    };

    let buckets = ReputationBuckets {
      reporting_accuracy: bucket_query(conn, "reporting_accuracy", None).await?,
      jury_reliability: bucket_query(conn, "jury_reliability", None).await?,
      participation_consistency: bucket_query(conn, "participation_consistency", None).await?,
      endorsement_strength: bucket_query(conn, "endorsement_strength", None).await?,
    };
    let capability_counts = capability_query(conn, None).await?;
    let founder_event_stats = founder_query(conn, None).await?;

    Ok(AdminReputationStatsResponse {
      buckets,
      thresholds_current,
      capability_counts,
      founder_event_stats,
      calculated_at: Utc::now(),
    })
  }
  ```
- **GOTCHA (visibility)**: `bucket_query`, `capability_query`, `founder_query` are currently private in `admin_reputation_stats.rs`. Task 3 promotes each to `pub(crate)` at the same commit. If the impl agent finds they have different signatures than shown (e.g. different `community_bind` shape), adapt; the snippet above is based on the `admin_reputation_stats.rs:141-245` reading.
- **GOTCHA (get_int signature)**: the actual `get_int` signature takes `&mut DbPool<'_>`, not `&mut AsyncPgConnection`. The snippet uses the `&mut (&mut *conn).into()` trick — this is the same trick `admin_reputation_stats.rs:86-92` uses. Verify at impl time; if `DbPool` conversion fails, acquire a fresh conn or pass `pool` instead of `conn`.
- **GOTCHA (ConfigCache + DbPool)**: `config::get_int_opt(cache, pool, ...)` takes `&mut DbPool<'_>`, which means `rule_sets_summary` must receive `pool` (NOT derive it from conn). Handler body passes both.
- **GOTCHA (status enum string match)**: PostgreSQL returns status values with verbatim casing per the `DbValueStyle = "verbatim"` attribute on `CaseStatus` (`crates/db_schema_file/src/enums.rs:386`). The string match in `count_active_cases` uses PascalCase strings ("Decided", "Closed", "EmergencyRemove") — correct per this setting. Any drift in the enum values would break this filter; a grep invariant check at task 5 catches it: `rg '"Decided"|"Closed"|"EmergencyRemove"' crates/api/api/src/governance/admin_dashboard.rs` must return 1 hit each in the `matches!` line.
- **GOTCHA (ADR-013 compliance)**: the "total_active" excludes `EmergencyRemove`. This is an **intentional filter**, not an exhaustive match. ADR-013's exhaustive-match requirement applies to `match CaseStatus { ... }` statements, not string filters. A grep `rg 'match.*CaseStatus' crates/api/api/src/governance/admin_dashboard.rs` must return 0 hits.
- **GOTCHA (total_active semantics)**: the PRD says "active cases" informally. The plan pins it to "NOT IN (Decided, Closed, EmergencyRemove)" per PRD §6.2 note on "status NOT IN (Closed)" and per admin_config.rs:255's equivalent `WHERE status IN ('JurySelection', 'InReview')`. If a pilot disagrees about `Appealed` or `AdminReview` being "active", that's a product-tuning follow-up; the wire output exposes both `by_status` (all variants) AND `total_active` (the sum), so clients can re-compute differently.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task3.log 2>&1"; echo "exit: $?"
  tail -30 .claude/build-task3.log
  # Verify no match-on-CaseStatus leaked in:
  rg 'match.*CaseStatus' crates/api/api/src/governance/admin_dashboard.rs
  # Expected: no output (zero matches)
  ```
- **COMMIT MESSAGE**: `feat(admin-dashboard): admin_dashboard aggregate handler (task 3)`

### Task 4 — Add `async-stream` workspace dep + reqwest `stream` feature

- **ACTION**:
  1. Edit root `Cargo.toml`:
     - Add `async-stream = "0.3"` to `[workspace.dependencies]` alphabetically (near line 214 where `futures` lives).
     - Edit the `reqwest = { version = "0.13.2", default-features = false, features = [...] }` block at line 187–191: add `"stream"` to the features array.
  2. Edit `crates/api/api/Cargo.toml`:
     - Add `async-stream = { workspace = true }` under `[dependencies]` (alphabetical).
     - Add `tokio-postgres = { workspace = true }` under `[dependencies]` — it wasn't previously an `api` crate dep (existing code never touched raw tokio-postgres); v1-AD-d's SSE handler makes it a direct dep.
     - Check for `once_cell` under `[dependencies]`; if missing, add via workspace. If `once_cell` isn't already a workspace dep, use `std::sync::OnceLock<Mutex<HashSet<_>>>` in task 5 instead (stable since Rust 1.70; toolchain is 1.95 per CLAUDE.md, so OnceLock is fine).
  3. `cargo check` the workspace to make sure nothing broke.
- **MIRROR**: existing workspace `[dependencies]` table pattern (alphabetical, `version = "X.Y"` for simple deps or `{ version = "X", default-features = false, features = [...] }` for complex ones).
- **GOTCHA (feature propagation)**: adding `"stream"` to the workspace-level `reqwest` features turns it on for EVERY consumer of reqwest. That's fine — `stream` is a zero-cost feature flag (it just enables the `Response::bytes_stream()` method). Run `cargo check --workspace --features full` after edit.
- **GOTCHA (once_cell vs OnceLock)**: `crates/api/api/Cargo.toml` almost certainly does NOT already depend on `once_cell`. Before adding it, grep the workspace: `rg 'once_cell' Cargo.toml crates/*/Cargo.toml` — if zero hits, use `std::sync::OnceLock<tokio::sync::Mutex<HashSet<PersonId>>>` in task 5. `OnceLock::new()` is `const`, so a module-level `static X: OnceLock<...> = OnceLock::new();` works fine.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/build-task4.log 2>&1"; echo "exit: $?"
  tail -20 .claude/build-task4.log
  # Confirm async-stream resolved:
  rg 'async-stream' Cargo.lock
  # Expected: a line like `name = "async-stream"` with a version 0.3.x
  # Confirm reqwest stream feature resolved:
  rg 'reqwest v0.13' Cargo.lock -A 15 | rg 'stream'
  # Expected: shows the feature is enabled
  ```
- **COMMIT MESSAGE**: `chore(deps): add async-stream + enable reqwest stream feature for v1-AD-d SSE (task 4)`

### Task 5 — `admin_audit_stream` SSE handler + per-admin cap + route wiring

- **ACTION**:
  1. Create `crates/api/api/src/governance/admin_audit_stream.rs` per §10 SSE_HAND_ROLLED_STREAM. Adapt the skeleton:
     - If `once_cell` isn't a workspace dep (task 4 verified), replace `once_cell::sync::Lazy<Arc<Mutex<...>>>` with `std::sync::OnceLock<Arc<tokio::sync::Mutex<...>>>`.
     - Match TLS-config choice from `crates/diesel_utils/src/connection.rs:201-227` (full `establish_connection` fn; TLS branch body at :211) — if the database URL contains `sslmode=require`, use `tokio_postgres_rustls::MakeRustlsConnect::new(...)`; otherwise `tokio_postgres::NoTls`. Copy the `DangerousClientConfigBuilder` block verbatim if needed (solo-dev Docker-Compose Postgres doesn't require TLS, so `NoTls` is the common branch).
     - Derive the `DATABASE_URL` from `context.settings()`. Exact accessor: check `lemmy_utils::settings::Settings` — likely `context.settings().database.get_connection_url()` or `context.settings().get_database_url()`. Mirror whatever `establish_connection` uses.
  2. Register `pub mod admin_audit_stream;` in `crates/api/api/src/governance/mod.rs`.
  3. Edit `crates/api/routes/src/lib.rs` at lines 533–549 to add:
     - `.route("/dashboard", get().to(admin_dashboard))` (new sibling of `/reputation-stats`)
     - `.service(scope("/audit").route("/stream", get().to(admin_audit_stream)))` (new nested scope, parallel to `/config` and `/rule-sets`)
  4. Add imports to `crates/api/routes/src/lib.rs`: `use lemmy_api::governance::{admin_audit_stream::admin_audit_stream, admin_dashboard::admin_dashboard};` (match existing import style at the top of the file).
- **MIRROR**:
  - §10 SSE_HAND_ROLLED_STREAM snippet (skeleton)
  - §10 ROUTE_REGISTRATION snippet
  - `crates/diesel_utils/src/connection.rs:201-227` for the TLS branch
  - `crates/api/api/src/governance/admin_reputation_stats.rs` imports block for standard handler imports
- **GOTCHA (SSE cap counter leak)**: if the `SseGuard` `Drop` impl spawns a `tokio::spawn` to clean up, but the runtime is shutting down, the cleanup task may not run. Mitigation: also cleanup on Actix's `on_stop` hook — but for v1, accept the leak risk on graceful shutdown (the process restart clears the HashSet). Document in a comment on the guard struct.
- **GOTCHA (driver task leak)**: the `tokio::spawn` for the tokio-postgres driver must be cancelled when the stream ends; otherwise it leaks. Attach the `JoinHandle` to the `SseGuard` struct and `abort()` in `Drop`:
  ```rust
  struct SseGuard {
    admin_id: PersonId,
    driver: Option<tokio::task::JoinHandle<()>>,
  }
  impl Drop for SseGuard {
    fn drop(&mut self) {
      if let Some(h) = self.driver.take() { h.abort(); }
      let admin_id = self.admin_id;
      tokio::spawn(async move {
        if let Some(lock) = ACTIVE_SSE_ADMINS.get() {
          lock.lock().await.remove(&admin_id);
        }
      });
    }
  }
  ```
- **GOTCHA (async_stream + Drop)**: the `stream!` macro captures variables by move. The guard must be moved INTO the stream closure so its `Drop` fires when the stream ends. Don't call `drop(guard)` explicitly — let the move handle it. If the impl needs explicit ordering, use `let _ = std::mem::replace(&mut guard, ...)` at stream-end, but simplest is to just `let _guard = SseGuard { ... };` inside the stream macro before the loop.
- **GOTCHA (SSE frame format)**: each SSE frame ends with `\n\n` (two newlines). The `event:` line and `data:` line each end with `\n`. A frame like `event: foo\ndata: bar\n\n` is correct; `event: foo\ndata: bar\n` (one newline) is NOT a valid SSE frame per HTML5 §9.2.4. The test in task 6 asserts this explicitly.
- **GOTCHA (keepalive as SSE comment)**: the keepalive line starts with `:` (colon) and ends with `\n\n`. The client ignores comment lines per spec. Don't send a `data:` field for keepalive — it would be misinterpreted as a real event with empty data.
- **GOTCHA (connection cleanup order)**: on stream drop, the order is: (a) `async_stream` generator's local state drops → (b) guard drops → (c) driver task aborts → (d) tokio-postgres client drops → (e) underlying tcp socket closes. If (d) happens before (c), the driver task sees a connection error and exits naturally anyway — no leak. Don't over-engineer.
- **GOTCHA (rate limit + SSE)**: inheriting `rate_limit.post()` from the parent `/governance` scope means reconnecting clients hit the rate limit after ~10 connects/minute (whatever `.post()` configures). If pilots report this as a bug, override with `.wrap(rate_limit.search())` or similar on the `/admin/audit` subscope. v1-AD-d ships the inherited limit; retro addresses.
- **GOTCHA (logger spam on disconnect)**: every client disconnect will log the spawned driver task's "connection closed" message at info level. If this is noisy, wrap the `tokio::spawn(async move { if let Err(e) = connection.await { ... } })` pattern with a level-downgrade or a `.is_connection_reset()` filter. v1-AD-d accepts the noise; a retro issue tracks if it becomes a signal-to-noise problem.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-task5a.log 2>&1"; echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_routes --features full > .claude/build-task5b.log 2>&1"; echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/build-task5c.log 2>&1"; echo "exit: $?"
  tail -20 .claude/build-task5c.log
  # Verify registry invariant unchanged:
  rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
  # Expected: 26 (v1-AD-d adds zero entry kinds)
  # Verify clippy narrowed baseline unchanged:
  cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/clippy-task5.log 2>&1"; echo "exit: $?"
  ```
- **COMMIT MESSAGE**: `feat(admin-audit-stream): hand-rolled SSE endpoint + route wiring (task 5)`

### Task 6 — e2e tests + final polish

- **ACTION**:
  1. Add 5 e2e tests to `crates/server/tests/e2e.rs`:
     - `admin_dashboard_returns_aggregate_for_admin` — happy path, zero-row DB, assert every widget is present with zero/empty defaults, `calculated_at` within ±5s of wall-clock
     - `admin_dashboard_forbidden_for_non_admin` — non-admin user, assert 403 / `NotAnAdmin`
     - `admin_dashboard_aggregates_populated_data` — seed 3 moderation cases (varying statuses), 2 jury assignments, 1 federation attestation, 1 rule-set version, 2 governance_log admin_config_changed rows; assert every count matches
     - `admin_audit_stream_emits_config_change` — start a handler, connect SSE, trigger a config write via `admin_set_config`, assert SSE frame arrives within 5s with the right `event:` type and payload shape
     - `admin_audit_stream_enforces_per_admin_cap` — same admin connects twice in parallel; assert the second connection returns 409 Conflict
  2. For the streaming tests, use `reqwest::Client::get(...).send().await?.bytes_stream()` + `futures_util::StreamExt::next()` + `tokio::time::timeout`. Import pattern: `use futures_util::StreamExt; use tokio::time::{Duration, timeout};`.
  3. Add SSE fixture helpers in `admin_config_fixtures` (or create a new `admin_audit_fixtures` module):
     - `async fn bootstrap_http_server(context: &LemmyContext) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>)` — spins up an in-process actix server bound to a random port, returns addr + driver handle. If the existing e2e harness already does this (check `crates/server/tests/e2e.rs` first ~100 lines), mirror that pattern.
- **MIRROR**:
  - §10 E2E_TEST_SHAPE for the dashboard test
  - §10 E2E_TEST_SHAPE (SSE) for the stream test
  - `crates/server/tests/e2e.rs:4800-4841` (`admin_get_config_full`) as the admin-gated handler invocation pattern (direct call, not HTTP, for the dashboard tests; HTTP for the SSE tests)
- **GOTCHA (http server for SSE)**: the existing e2e tests likely call handlers directly via `admin_set_config(Json(...), context, admin_view).await` rather than spinning up an HTTP server. For SSE, we NEED an actual HTTP server because `reqwest::bytes_stream` can't work against a direct handler call — the stream semantics require a real TCP connection. If no `bootstrap_http_server` helper exists yet, task 6 writes one. A 30-line actix test server using `HttpServer::new(|| { ... }).workers(1).bind(("127.0.0.1", 0))?.run()` pattern suffices.
- **GOTCHA (test timing)**: the SSE test's "write a config → expect SSE event" race can fail on slow CI if the config write's governance_log row hasn't committed before the SSE handler's LISTEN is ready. Mitigation: the SSE test spawns the LISTEN client FIRST, waits for the retry frame, THEN issues the config write. The retry frame confirms LISTEN is armed.
- **GOTCHA (cleanup between tests)**: the per-admin SSE cap's `ACTIVE_SSE_ADMINS` static HashSet leaks across tests if multiple SSE tests run sequentially on the same admin PersonId. Mitigation: each SSE test uses a fresh admin user (unique username) so the HashSet keys don't collide. OR: add a test-only `clear_sse_cap_state()` helper behind `#[cfg(test)]`.
- **GOTCHA (async_stream yields inside tokio::select)**: if the test times out on the retry frame, diagnose by checking: (a) is the `stream!` generator actually reaching the first `yield`? Wrap it in a `tracing::debug!` after `yield`. (b) is actix buffering the first bytes? The `X-Accel-Buffering: no` header + `Cache-Control: no-cache` headers are load-bearing for nginx-style proxies; verify they're set in task 5.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/build-task6.log 2>&1"; echo "exit: $?"
  tail -20 .claude/build-task6.log
  # Run just the v1-AD-d tests (assume the harness has test-filter by name):
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e admin_dashboard -p lemmy_server > .claude/test-task6a.log 2>&1"; echo "exit: $?"
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e admin_audit_stream -p lemmy_server > .claude/test-task6b.log 2>&1"; echo "exit: $?"
  tail -30 .claude/test-task6a.log
  tail -30 .claude/test-task6b.log
  # Full regression check: all pre-existing e2e tests pass:
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > .claude/test-task6-full.log 2>&1"; echo "exit: $?"
  tail -40 .claude/test-task6-full.log
  # Final clippy narrowed gate:
  cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/clippy-task6.log 2>&1"; echo "exit: $?"
  ```
- **COMMIT MESSAGE**: `test(admin-dashboard): 5 e2e tests for dashboard + SSE audit stream (task 6)`

---

## 14. Testing strategy

Per [IMPLEMENTATION-PLAN-v0.md §5](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): **integration-only** for v1. Unit tests only for the `project_to_audit_entry` move (where a tiny test that constructs a mock `GovernanceLog` and asserts the projection is still useful — but it already exists and the move doesn't change behaviour, so a new test isn't required).

### Tests to add (5)

| Test Name | What It Validates |
|---|---|
| `admin_dashboard_returns_aggregate_for_admin` | Happy path — fresh DB, assert every widget is present with zero/empty-collection defaults. `calculated_at` is within 5s of `Utc::now()`. |
| `admin_dashboard_forbidden_for_non_admin` | Capability gate — non-admin user receives 403 / `LemmyErrorType::NotAnAdmin`. No `governance_log` entry is written (dashboard is read-only). |
| `admin_dashboard_aggregates_populated_data` | Data fidelity — seed cases across 4 statuses (Open, JurySelection, Decided, EmergencyRemove), 3 jury assignments across statuses, 1 federation_attestation (expired), 1 rule-set version with active_version_id set, 2 admin_config_changed log entries. Assert: `active_cases.by_status` has the 4 expected buckets with correct counts; `active_cases.total_active` equals the 2 non-terminal cases (Open + JurySelection); `jury_queue.pending_accept` + `.accepted` + `.submitted` sum to 3; `federation.expired = 1, .total = 1`; `rule_sets.communities_with_rule_sets = 1, .per_community[0].active_version_id = Some(1)`; `recent_config_changes.len() = 2`. |
| `admin_audit_stream_emits_config_change` | SSE happy path — start HTTP server, connect to `/admin/audit/stream` as admin, assert first frame is `event: retry`, then in parallel trigger `admin_set_config("test.key", "val")`, assert within 5s a frame matching `event: admin_config_changed` arrives with the correct JSON shape. |
| `admin_audit_stream_enforces_per_admin_cap` | SSE cap + guard cleanup — same admin connects to `/admin/audit/stream` twice in parallel; first gets 200 + event-stream; second gets 409 Conflict with a plain-text error body. **Load-bearing**: after the first client disconnects (drop reqwest stream), a third connection from the same admin succeeds within 2s — this asserts the `SseGuard::Drop` impl actually clears the `ACTIVE_SSE_ADMINS` HashSet entry. A regression here would leak state across client reconnects in production. |

### Edge cases covered

- [ ] Fresh DB returns zero counts, not errors
- [ ] `EmergencyRemove` cases are excluded from `total_active` but present in `by_status`
- [ ] Federation attestations with `valid_until IS NULL` count as active (not expired)
- [ ] A community with `rule_set_version` rows but no `rule_set.active_version_id` config row returns `per_community[X].active_version_id = None`
- [ ] Recent-config-changes widget contains only the 2 admin-config entry kinds (not the rule-set-created or Phase-6 federation entry kinds)
- [ ] **CI-gated**: SSE guard cleanup on client disconnect — the `admin_audit_stream_enforces_per_admin_cap` test's third-connection-succeeds assertion proves `SseGuard::Drop` cleared the HashSet entry. This is the load-bearing cleanup test; zombie-task detection (via `tokio_metrics` or similar) is not CI-gated, but the HashSet-state proxy is.
- [ ] 2nd SSE connection from same admin returns 409 (not 500 or other)
- [ ] Keepalive frames arrive every 15s when idle — **NOT CI-gated**, manual-verify only via §15 Level 7 manual run. A 16s CI timeout per test is expensive and flake-prone; the SSE spec compliance (frame format correctness) is covered by the `admin_audit_stream_emits_config_change` test instead.
- [ ] `project_to_audit_entry` move is behaviour-neutral — all 13 v1-AD-b e2e tests continue to pass unchanged (run the full e2e suite in task 6)

### NOT covered (deferred to pilot ops / v1.x)

- SSE reconnect after server restart (manual ops test)
- SSE frame-order determinism under multi-writer contention (ordering is `governance_log.created_at` + `id`, already enforced; fuzzing this is a v1.x concern)
- Dashboard performance at 10k cases / 1M log rows (pilot-driven — if slow, add caching or materialised views)
- Federation widget with a live federation peer (federation-inbound PRD owns the integration tests for that widget's data source)

---

## 15. Validation commands (DoD)

Use these exact commands — do NOT substitute `npm`/`pnpm`. This is a Rust project. Every command captured to file per `.claude/rules/cargo-output-capture.md`.

### Level 1: STATIC_ANALYSIS

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/dod-level1-check.log 2>&1"; echo "exit: $?"
tail -20 .claude/dod-level1-check.log
```

**EXPECT**: exit 0, zero errors.

### Level 2: CLIPPY (narrowed — production-code only, matches v1-AD-c ratchet)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/dod-level2-clippy.log 2>&1"; echo "exit: $?"
tail -30 .claude/dod-level2-clippy.log
```

**EXPECT**: exit 0, zero warnings. Matches the v1-AD-b / v1-AD-c baseline.

### Level 3: INTEGRATION_TESTS (v1-AD-d + full regression)

```bash
# New v1-AD-d tests only:
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server admin_dashboard > .claude/dod-level3-dashboard.log 2>&1"; echo "exit: $?"
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server admin_audit_stream > .claude/dod-level3-stream.log 2>&1"; echo "exit: $?"
# Full e2e regression:
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > .claude/dod-level3-all.log 2>&1"; echo "exit: $?"
tail -40 .claude/dod-level3-all.log
```

**EXPECT**: all 5 new tests pass; all pre-existing v0 / v1-AD-a / v1-AD-b / v1-AD-c tests pass unchanged.

### Level 4: TEST_TARGET_COMPILE (test-target regression — carry-forward from v1-AD-a retro per `feedback_test_target_compile_validation.md`)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/dod-level4-testbuild.log 2>&1"; echo "exit: $?"
tail -20 .claude/dod-level4-testbuild.log
```

**EXPECT**: exit 0. Test target compiles cleanly even before running.

### Level 5: REGISTRY_INVARIANT

```bash
# v1-AD-d adds zero entry kinds; count must be unchanged:
rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
# Expected: 26

rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs | awk -F: '/ENTRY_KIND_/ {print}' | grep -oE '"[a-z_]+"' | sort | uniq -d
# Expected: empty

rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l
# Expected: 26 (shim re-export parity)
```

### Level 6: CROSS-CUTTING_VERIFICATION (ADR compliance)

```bash
# ADR-013 — no match-on-CaseStatus introduced in new files:
rg 'match.*CaseStatus' crates/api/api/src/governance/admin_dashboard.rs crates/api/api/src/governance/admin_audit_stream.rs crates/api/api/src/governance/audit_projection.rs
# Expected: no output

# ADR-008 — dashboard is read-only; no `governance_log::append` calls:
rg 'governance_log::append|governance_log::append_with' crates/api/api/src/governance/admin_dashboard.rs crates/api/api/src/governance/admin_audit_stream.rs
# Expected: no output

# ADR-015 — no direct person_id or username written in dashboard:
rg 'person_id|username|\.email' crates/api/api/src/governance/admin_dashboard.rs crates/api/api/src/governance/admin_audit_stream.rs
# Expected: zero writes to log (only reads via `local_user_view.person.id` for cap-check are acceptable)
```

### Level 7: MANUAL_VALIDATION (1 command — SSE sanity + full dashboard pull, to run against a local ephemeral-db instance)

```bash
# 1. Start the dev server (assume harness script exists):
scripts/brehon/dev-server-up.sh &
sleep 3
# 2. Dashboard pull (admin JWT assumed in env var):
curl -s -H "Authorization: Bearer $ADMIN_JWT" http://localhost:8536/api/v4/governance/admin/dashboard | jq .
# EXPECT: JSON with every widget key present
# 3. SSE pull (background curl + trigger a config write):
curl -sN -H "Authorization: Bearer $ADMIN_JWT" -H "Accept: text/event-stream" \
  http://localhost:8536/api/v4/governance/admin/audit/stream > .claude/sse-manual.log &
SSE_PID=$!
sleep 1
curl -X POST -H "Content-Type: application/json" -H "Authorization: Bearer $ADMIN_JWT" \
  -d '{"key":"jury.panel_size","value_type":"int","value":9,"scope":"instance","reason":"v1-AD-d manual"}' \
  http://localhost:8536/api/v4/governance/admin/config
sleep 2
kill $SSE_PID
cat .claude/sse-manual.log
# EXPECT: sse-manual.log contains `event: retry`, then `event: admin_config_changed` with the payload
scripts/brehon/dev-server-down.sh
```

**EXPECT**: all dashboard keys populated; SSE log shows retry + admin_config_changed frames.

---

## 16. Acceptance criteria

- [ ] Both handlers registered and reachable under `/api/v4/governance/admin/dashboard` and `/api/v4/governance/admin/audit/stream`
- [ ] Level 1–5 validation commands pass with exit 0
- [ ] All 5 new e2e tests pass; all pre-existing v1-AD-b / v1-AD-c tests pass unchanged
- [ ] Dashboard response includes all six widget sections with deterministic JSON field order
- [ ] SSE handler sends the initial `event: retry` frame, followed by `admin_config_changed` / `admin_config_change_denied` frames filtered from the `governance_events` channel
- [ ] Per-admin SSE cap returns 409 on second concurrent connection from same PersonId
- [ ] Keepalive `: keepalive\n\n` frames appear every 15s while idle
- [ ] No contradictions with the 15 ADRs in [99](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (ADR-010 append-only: zero INSERTs; ADR-013 EmergencyRemove: handled by filter, not match; ADR-015 pseudonymity: no direct person identifiers written to any log from these endpoints)
- [ ] No new governance_log entry kinds (registry count stays at 26)
- [ ] `project_to_audit_entry` move is behaviour-neutral (all v1-AD-b audit tests pass)
- [ ] Cross-cutting verification (Level 6) passes: no match-on-CaseStatus, no log writes, no person_id leakage

---

## 17. Completion checklist

- [ ] Task 0 (pre-phase audit) completed with `.claude/PRPs/reports/v1-AD-d-task0-audit.md` committed
- [ ] Task 1 (audit_projection extraction) landed; all v1-AD-b tests still green
- [ ] Task 2 (DTOs added to api_common) landed; `cargo check -p lemmy_api_common` green
- [ ] Task 3 (admin_dashboard handler) landed; `cargo check -p lemmy_api` green
- [ ] Task 4 (workspace deps — async-stream + reqwest stream feature) landed; `cargo check --workspace` green
- [ ] Task 5 (admin_audit_stream handler + routes) landed; `cargo check --workspace` + clippy narrowed green
- [ ] Task 6 (5 e2e tests + polish) landed; full e2e suite green
- [ ] Level 1–5 DoD gates exit 0
- [ ] Level 6 cross-cutting greps return zero matches
- [ ] Level 7 manual validation observed on a local ephemeral-db run (dashboard + SSE)
- [ ] Registry invariant check: `ENTRY_KIND_*` count unchanged at 26
- [ ] Clippy narrowed baseline unchanged from task 0 capture
- [ ] PR description cites this plan and the v1-AD-d pre-plan brief
- [ ] CR triage — any CR finding categorised per `feedback_pr_review_triage_pattern.md` four buckets
- [ ] Phase-close report at `.claude/PRPs/reports/v1-AD-d-complete-report.md`
- [ ] Advisor memory updated: `project_v1_AD_d_closed.md` with carry-forward issues (if any)

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| SSE + LISTEN is novel for this workspace — no existing consumer | HIGH | MED | Plan has full skeleton in §10; hand-rolled pattern is well-documented in `async-stream` + `tokio-postgres` docs; task 5 `GOTCHA` section catches the top-5 footguns (frame format, Drop ordering, driver leak, heartbeat interaction, nginx buffering). **Phase-close hedge**: if task 5 hits a novel-pattern wall that task 0 probes didn't catch and total iteration time exceeds 4× the task 3 budget, the sub-phase splits at that point: v1-AD-d1 ships dashboard alone (tasks 1+2+3 + the dashboard subset of task 6's e2e tests), v1-AD-d2 ships SSE as a follow-up sub-phase (tasks 4+5 + the SSE subset of task 6). Dashboard alone is genuinely valuable even if SSE slips — operators get the single-fetch aggregate now, SSE ships when it ships. |
| `actix-web 4.13.0`'s `HttpResponse::streaming` interaction with `X-Accel-Buffering: no` header may not flush small SSE frames promptly behind a reverse proxy | MED | MED | Plan sets both `Cache-Control: no-cache, no-transform` and `X-Accel-Buffering: no` headers per task 5; SSE is explicit about the need for no-buffering; pilot ops run direct-to-actix (no nginx) in v1 per [07 operations-and-federation.md](../../../docs/brehon-law-inspired-network/07-operations-and-federation.md) |
| `tokio_postgres::connect` with `sslmode=require` may require copying the `rustls` block from `establish_connection` verbatim | MED | LOW | Task 5 GOTCHA explicitly mirrors `crates/diesel_utils/src/connection.rs:201-227` (TLS branch body at :211); solo-dev default URL is `sslmode=disable` so `NoTls` is the common branch |
| Per-admin SSE cap leaks state across tests or across process restarts | LOW | LOW | In-memory HashSet is cleared by process restart (documented); tests use unique admin usernames; v2 upgrade path is `DashMap` or Redis |
| `admin_reputation_stats` helpers' visibility change to `pub(crate)` breaks an unrelated v1-AD-b test | LOW | LOW | v1-AD-b never imports these helpers by name (they're internal to `admin_reputation_stats.rs`); visibility promotion only widens access |
| Rule-sets summary's per-community loop (up to 100 iterations × `get_int_opt`) is slow on a pilot with many communities | LOW | LOW | 100-entry cap per §4.1 load-bearing decision; `ConfigCache` memoises within the loop; v1.x adds a single JOIN query if pilots report slow dashboard loads |
| A dashboard widget's inline SQL uses a status value that doesn't match the `DbValueStyle = "verbatim"` casing | MED | MED | Task 3 GOTCHA asserts verbatim PascalCase; invariant grep on "Decided"/"Closed"/"EmergencyRemove" strings in the handler; e2e test `admin_dashboard_aggregates_populated_data` seeds data across all four statuses and asserts the counts |
| `project_to_audit_entry` module move breaks an unrelated internal test import | LOW | LOW | Task 1 grep invariant; if detected, update imports at task 1 rather than deferring |
| Workspace `async-stream` dep addition triggers a dependency-tree rebuild that introduces an unrelated lint cascade | LOW | MED | Task 4 captures clippy narrowed log immediately after dep add; if the cascade appears, revert and defer SSE to a follow-up phase (dashboard alone is still valuable) |
| CI runs only e2e tests (per `feedback_ci_runs_integration_tests_only.md`) — any lib-unit-test logic we add won't run on CI | LOW | LOW | v1-AD-d adds no lib unit tests; all validation goes through e2e; documented memory already reflects this |
| SSE test flakes in CI due to timing-sensitive "write → expect event" assertions | MED | MED | Task 6 GOTCHA mandates: retry-frame-wait before write trigger; `timeout(Duration::from_secs(5), ...)` gives 5s slack; if CI still flakes, bump to 10s or move to a mock-pg-notify approach in a follow-up |

---

## 19. Notes

### 19.1 Why two handlers in one sub-phase

`admin_dashboard` and `admin_audit_stream` share 60% of their surface: both are admin-gated GETs under `/admin`, both consume the governance_log audit trail, both depend on the `project_to_audit_entry` projection. Splitting them across v1-AD-d1 + v1-AD-d2 would duplicate the task 1 (projection extraction), task 2 (DTO block), task 4 (Cargo.toml deps), and task 6 (test fixture) work. Shipping together is cheaper. The pre-plan brief (advisor, 2026-04-22) estimates 5–7 tasks and sees no separation value.

### 19.2 Why hand-rolled SSE instead of `actix-web-lab`

Per OQ-V1-AD-02 resolution. `actix-web-lab` is a crate from the actix-web ecosystem that provides a `Sse<Stream>` response type with built-in keepalive. Pros: less code (~50 lines saved). Cons: (a) one more workspace dep for one endpoint, (b) `actix-web-lab`'s SSE implementation is opinionated (fixed keepalive interval, fixed frame format), (c) adds a transitive on `tokio-util` that we don't otherwise need. The hand-rolled version is ~150 lines and gives us full control over heartbeat timing, frame format, and the per-admin cap integration. Preference goes to the cheaper dep surface.

### 19.3 What v1-AD-e will inherit

When v1-AD-e (askama HTML pages) picks up, it will:
- Import `AdminDashboardResponse` from `lemmy_api_common::governance`
- Call the existing `admin_dashboard` handler via Actix's internal routing (or, more likely, call the handler function directly and render the result through askama)
- Use the existing `admin_audit_stream` endpoint from client-side JS in the HTML page's `<script>` block (vanilla `EventSource("/api/v4/governance/admin/audit/stream")`)
- Add zero new Rust code paths — pure templating + client-side JS

### 19.4 Carry-forward issues from v1-AD-c (#82–#85)

None of these block v1-AD-d. Per `project_v1_AD_c_closed.md`, all four are chores (lint cleanup, doc polish). They can be addressed in parallel PRs or bundled into a "v1-AD-wave cleanup" PR after v1-AD-d merges. If any of them touch `admin_config.rs` (the file task 1 edits), coordinate timing.

### 19.5 Confidence score

**8.0 / 10** for one-pass implementation success (advisor review 2026-04-22 revised from impl's 8.5 — half-point lower because SSE novelty is genuinely HIGH per §18 row 1).

- **+2**: All substrate shipped (trigger, DTOs mirror, v1-AD-b handler patterns, v1-AD-c refactor already done)
- **+2**: Plan has full code skeletons for every task including the novel SSE pattern
- **+1**: No new migrations, no new entry kinds, no ADR risk
- **+1**: Parallel work of the advisor-recommended v1-AD-c tasks prove the team can ship multi-handler sub-phases cleanly
- **+0.5**: Hand-rolled SSE is low-surface — ~150 lines total, well-contained
- **+0.5**: §18 row 1 now carries a sub-phase-split hedge (v1-AD-d1 + v1-AD-d2) protecting phase-close timing if SSE hits a wall
- **-1.5**: SSE + LISTEN is novel for this workspace; first-run integration may surface unforeseen actix/tokio-postgres interactions (drop ordering, rustls TLS branch, idle-timeout nuances)
- **-0.5**: `bucket_query` / `capability_query` / `founder_query` visibility promotion in `admin_reputation_stats.rs` is a second-order refactor; if the impl agent finds their signatures differ from the plan's reading, task 3 needs adaptation

### 19.6 Task sizing relative to prior sub-phases

| Sub-phase | Tasks (exec) | File edits | New DTOs | New handlers | New entry kinds | Clippy baseline change |
|---|---|---|---|---|---|---|
| v1-AD-a | 7 | ~14 | 6 | 0 | 2 | no |
| v1-AD-b | 8 | ~9 | 5 | 3 | 0 | no |
| v1-AD-c | 8 | 12 | 5 | 2 | 1 | no |
| **v1-AD-d** | **6** | **11** | **6** | **2** | **0** | **no** |

v1-AD-d is the **smallest sub-phase of the v1-AD wave** by task count, reflecting its pure-additive, pure-read-only nature. File-edit count is comparable to v1-AD-b/c. DTO count is higher because the dashboard response nests six sub-structs, but each is trivial (≤5 fields).

### 19.7 Blocking on task 4 (Cargo.toml workspace dep)

Task 4 lands a new workspace dep (`async-stream`). Per advisor memory `project_v1_AD_c_closed.md`, touching `Cargo.toml` is low-risk but CR-visible — the change touches every crate's transitive build. If CR flags it, the response is "single-endpoint, one-time dep add, justified by OQ-V1-AD-02 hand-roll resolution". No CR precedent exists for rejecting a workspace dep add when the consumer is a single file; do not over-justify in the PR description.

### 19.8 Commit message patterns (v1-AD-b / v1-AD-c ratchet)

- Task 0 — no commit
- Task 1 — `refactor(governance): ...`
- Task 2 — `feat(api-common): ...`
- Task 3 — `feat(admin-dashboard): ...`
- Task 4 — `chore(deps): ...`
- Task 5 — `feat(admin-audit-stream): ...`
- Task 6 — `test(admin-dashboard): ...`

Commit messages match the v1-AD-b / v1-AD-c convention: `<type>(<scope>): <imperative description>` (no task number in subject, body may reference "task N" if the task has a dedicated artifact).

### 19.9 Follow-up suggestions (NOT in v1-AD-d)

- `admin_dashboard` caching (moka / 5s TTL) — v1.x if pilots report slow responses
- SSE reconnect resumption via `Last-Event-ID` header — v1.x
- Federation widget expansion to include `sent_activity` pending counts — v1.x federation-inbound consumption
- Per-community dashboard (new endpoint `/admin/dashboard?community_id=X`) — v1-AD-e or v1.x
- Websocket alternative to SSE — out of scope; SSE sufficient
- Structured logging for SSE drop reasons — v1.x observability pass
