# Plan: v1-AD-e — Server-rendered admin dashboard HTML pages (Dashboard + Audit, pure-templating over shipped v1-AD-d)

## Table of contents

| § | Heading |
|---|---|
| 1 | Summary |
| 2 | Source |
| 3 | Problem statement |
| 4 | Solution statement |
| 5 | Metadata + complexity |
| 6 | Relationship to other v1-AD sub-phases |
| 7 | Preflight guardrails + scope-cut DQ |
| 8 | Flow design |
| 9 | Mandatory reading |
| 10 | Patterns to mirror |
| 11 | Files to change |
| 12 | NOT building in v1-AD-e |
| 13 | Step-by-step tasks |
| 14 | Testing strategy |
| 15 | Validation commands (DoD) |
| 16 | Acceptance criteria |
| 16a | Stories |
| 17 | Completion checklist |
| 18 | Risks and mitigations |
| 19 | Notes |
| 20 | Confidence score |

---

## 1. Summary

v1-AD-e is the **page-layer sub-phase** of the v1 admin-dashboard keystone. It ships the first server-rendered HTML in the brehon-fork workspace: two browser-accessible admin pages that render data already produced by the **shipped** v1-AD-d endpoints — a **Dashboard page** (`GET /api/v4/governance/admin/dashboard/view`) rendering the `AdminDashboardResponse` aggregate, and an **Audit page** (`GET /api/v4/governance/admin/audit/view`) rendering recent config-change entries with a live tail via client-side `EventSource` against the existing `/admin/audit/stream` SSE endpoint. Both pages are instance-admin-gated (reusing `is_admin` + `LocalUserView`), feature-flagged on the already-seeded `governance.dashboard.html_pages_enabled` config key (default `true`), and contain **zero new DB read paths** — they call the existing data-gathering logic in `lemmy_api`. The headline acceptance condition: an authenticated instance admin loading `/admin/governance/dashboard/view` in a browser sees the live aggregate as HTML; a non-admin gets 403; with `html_pages_enabled = false` both routes return 404/disabled. The two PRD §6 pages requiring net-new read paths (Config-editor, Single-key-editor, Rule-set-manager) are **explicitly deferred** to a later v1-AD-f (see §7 scope-cut DQ).

## 2. Source

- `.claude/PRPs/prds/v1-admin-dashboard.prd.md` §6 (Pages — minimal server-rendered admin UI), §6.1 (page implementation notes: templates dir, `LocalUserView` auth, vanilla JS, `html_pages_enabled` opt-in), §6.2 (dashboard widget data sources), §7.1 (capability checks — `GET /admin/dashboard` = `is_admin`), §1.1 G6 (backend-first minimal pages), §3.1 (`governance.dashboard.*` namespace — both keys instance-scope, `html_pages_enabled` default `true`).
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — **OQ-V1-AD-01 re-resolved 2026-05-16** (v1-AD-e un-descoped from v1 on parallel-lane-capacity driver; engine choice askama-vs-maud **explicitly OPEN**, delegated to this plan §5 + a backend-lead DQ; premise of the 2026-04-20 deferral *not* overturned — sequencing-only supersede; PRD §6 page list is the scope superset; this plan §7 recommends a shipped-read-path-bounded subset with a DQ cut line). Changelog entry dated 2026-05-16. ADR-010 (React/SPA is v2 — **unchanged**, this plan ships server-rendered HTML only, no SPA/JS framework). ADR-011 (AGPLv3 — HTML templates are AGPL source like the rest; no relicence). ADR-004 (governance plane separation — pages live under the existing `/api/v4/governance/admin/*` tree, governance handler modules). ADR-015 (pseudonyms — pages render `AdminConfigAuditEntry.actor_pseudonym`, which is **already redacted** by the shipped projection; no new redaction surface). ADR-013 (EmergencyRemove exhaustive match — no new `CaseStatus` match introduced; pages render the pre-computed `ActiveCasesSummary.by_status` BTreeMap).
- `.claude/PRPs/plans/completed/v1-admin-dashboard-d.plan.md` §19.3 ("What v1-AD-e will inherit": import `AdminDashboardResponse`, call `admin_dashboard` logic, consume `/admin/audit/stream` via client-side `EventSource`, add zero new Rust DB paths) — this plan is the literal execution of that inheritance spec.
- `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` §5 (testing strategy — integration-only, e2e in `crates/server/tests/e2e.rs`), §4 cross-cutting (pseudonym/redaction — satisfied transitively, no new emission path).
- `.claude/lessons/feedback_library_add_after_shipping.md` — binds Task 1 (new workspace dep added in its own isolated commit, clippy-narrowed log captured immediately post-add to catch a lint cascade from the dependency-tree rebuild).
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — binds the e2e task: the e2e test fn + helpers must use a uniform `Result` shape; a v1-AD-* / v1-SL-* / v1-JM-* sibling fixtures module already exists in `crates/server/tests/e2e.rs` using `LemmyResult<()>` — mirror Case A verbatim.
- `.claude/lessons/feedback_async_pool_test_pattern.md` — binds the e2e task: `AsyncPgConnection::establish` + `DbPool::Conn` fixture pattern for the seed step.
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — binds the e2e task: `crates/server/tests/e2e.rs` is >8000 lines; the e2e edit is its own dedicated task, never bundled with handler logic, single append-only edit.
- `.claude/lessons/feedback_windows_e2e_requires_bat_wrapper.md` — binds §15: e2e runs via `scripts\brehon\cargo-test.bat --workspace --test e2e --features full`, never bare `cargo test`.
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` + `feedback_clippy_test_style.md` — bind §15.2 (clippy `--no-deps -- -D warnings`, rerun after any fix).
- Prior retro guardrails: `.claude/PRPs/reports/v1-AD-c-closeout.md` (CR triage four-bucket pattern), `.claude/PRPs/reports/v1-AD-b-plan-drift-notes.md` (`ConfigScope` gotcha — relevant only if the scope-cut DQ pulls Config-editor in; not in default scope).

## 3. Problem statement

The v1 admin-dashboard keystone shipped its full API surface (v1-AD-a substrate → v1-AD-b config HTTP → v1-AD-c rule-sets/snapshot → v1-AD-d dashboard aggregate + SSE, PRs #72/#76/#81/#87/#90) but **no browser-accessible operator UI**. Three gaps:

1. **No HTML dashboard.** `GET /api/v4/governance/admin/dashboard` returns `AdminDashboardResponse` as JSON. A pilot operator running governance day-to-day must `curl … | jq` to read active-case counts, jury-queue depth, federation status, and reputation buckets. PRD §6 specifies a server-rendered HTML page for exactly this. Tie: §13 Task 2 (dashboard handler) + §10.2 (response-tree → template mapping).

2. **No HTML audit tail.** `/admin/audit/stream` (SSE) and `/admin/config/audit` (paged JSON) exist, but the only way to watch config changes live is a hand-rolled SSE client or DB tailing. PRD §6.2 specifies the audit page with "last 20 + SSE push." Tie: §13 Task 3 (audit handler) + Task 4 (the page's vanilla `<script>` EventSource block).

3. **The feature flag is seeded but unreachable.** v1-AD-a seeded `governance.dashboard.html_pages_enabled` (default `true`, instance-scope, `ConfigKeyMetadata` at `config.rs:2317-2328`) explicitly to gate this sub-phase's pages. Today the key is read by nothing — flipping it has no effect because no HTML route consults it. Tie: §13 Task 2/Task 3 (both gate on this key via the existing config accessor) + §10.4 (config-read pattern).

Without v1-AD-e the keystone is API-complete but operator-incomplete; the deferral premise ("curl+jq is sufficient for pilot") still technically holds (this is not a correctness gap) but the parallel-lane capacity now exists to close the UX gap cheaply while the v1-AD design context is warm and unrebased (per OQ-V1-AD-01 re-resolution 2026-05-16).

## 4. Solution statement

Introduce the workspace's first HTML-rendering path as a **thin presentation layer** over shipped logic, with **no new DB queries, no new DTOs, no new capability gates, no new migrations, no new config keys**.

```
                 EXISTING (shipped v1-AD-d)                  NEW (v1-AD-e)
┌──────────────────────────────────────────┐   ┌───────────────────────────────────┐
│ admin_dashboard()                          │   │ admin_dashboard_html()             │
│   is_admin? → gather 6 sections →           │   │   is_admin? → html_pages_enabled?  │
│   Json<AdminDashboardResponse>             │◄──┤   → reuse gather logic →            │
│   (crates/api/api/src/governance/           │   │   render → HttpResponse text/html  │
│    admin_dashboard.rs)                      │   │   (admin_dashboard_html.rs, SAME   │
│                                             │   │    crate ⇒ pub(crate) reuse OK)    │
├──────────────────────────────────────────┤   ├───────────────────────────────────┤
│ admin_audit_stream() SSE                    │   │ admin_audit_html()                 │
│   event: <kind>\ndata: <json>\n\n          │◄──┤   is_admin? → html_pages_enabled?  │
│   (admin_audit_stream.rs)                   │   │   → render last-N + <script>       │
│                                             │   │     EventSource(/audit/stream)     │
└──────────────────────────────────────────┘   └───────────────────────────────────┘
       route: /api/v4/governance/admin/dashboard       route: …/admin/dashboard/view
       route: …/admin/audit/stream                      route: …/admin/audit/view
```

**Load-bearing architectural decision (from Explore):** `project_to_audit_entry` and the section-gathering fns (`count_active_cases`, `count_jury_queue`, `list_recent_config_changes`, `federation_summary`, `reputation_instance_scope`, `rule_sets_summary`) are `pub(crate)` private `async fn` inside the `lemmy_api` crate (`crates/api/api/src/governance/admin_dashboard.rs`). To reuse them without a refactor, the HTML handlers **must live in the same crate** — new sibling module `crates/api/api/src/governance/admin_dashboard_html.rs`. Task 2 extracts the existing `admin_dashboard` body's data-gathering into a `pub(crate) async fn gather_dashboard(...) -> LemmyResult<AdminDashboardResponse>` so both the JSON handler and the HTML handler call one source of truth (no logic duplication, no behaviour drift). This is a **structural refactor of one existing function**, not new business logic.

Engine: §5 recommends **maud** (single dep, zero new non-Rust files, native actix `Responder`, matches the codebase's inline-Rust idiom — the SSE handler already builds non-JSON responses inline) and files a DQ for backend-lead sign-off; askama is presented fairly as the alternative. The plan body below is **engine-parameterised** — Task 1 installs whichever engine the DQ resolves; Tasks 2–4's IMPLEMENT lines name the render call abstractly (`render_dashboard(&resp) -> String`) so the resolved engine slots in without re-planning. Reader can predict §11 from this section: one new handler module, one `mod.rs` line, one route block in `lib.rs`, one `Cargo.toml` dep line(s), template artifacts iff askama wins, one e2e test append.

## 5. Metadata + complexity

- **Phase:** `v1-AD-e`
- **Branch:** `phase-v1-AD-e` (cut by BM-task before Task 1, off `governance-v0`)
- **Target impl-task model:** `sonnet-4-6` (default)
- **Estimated tasks:** 7 (Task 0 pre-flight + 5 impl + retro)
- **Estimated cargo budget:** pre-Shape-G (Shape G suspended until 2026-06-01 per `project_shape_g_suspended_2026_05_16` — **validate-pending-laptop mode active**). Peak ≈ 6 GB for `cargo check --workspace --features full` on the laptop (canonical runner per `project_laptop_canonical_cargo_runner`). Single-lane; no cohort fan-out memory stacking.
- **Forbidden-window applicability:** standard (per advisor-orchestrator.md table — binds laptop cargo under validate-pending-laptop).
- **Complexity score:** **4 / 10** — see breakdown.

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 5 impl tasks (Tasks 1–5); none above the 5-task line |
| Migrations touched | +2 each | 0 | Zero new migrations (config key already seeded) |
| Crates touched | +1 each | 2 | `lemmy_api` (handlers + Cargo.toml) + `lemmy_server` (e2e test) |
| `crates/server/tests/e2e.rs` edits | +3 each | 1 | One dedicated e2e task (Task 5), single append |
| New ADR-affecting decisions | +2 each | 0 | OQ-V1-AD-01 already re-resolved 2026-05-16 *before* this plan; engine-choice DQ is a plan-level decision, NOT an ADR supersede (ADR-010/011/004/015/013 all unchanged) |
| Cargo budget peak above 6 GB | +1 per GB | 0 | Peak ≈ 6 GB, not above |
| **Total** | — | **4** | Threshold for split-DQ: `>8` (Sonnet). 4 ≤ 8 → **no split**. |

Score 4/10 reflects the pure-additive, pure-presentation nature: no schema, no new read paths, no ADR risk, one isolated dep add, one e2e append. The only non-trivial element is "first HTML in the workspace" (mitigated by §18 + the maud-recommendation reducing new infra to near-zero).

### 5.2 Engine recommendation (DQ pre-seed — backend-lead sign-off required)

Per OQ-V1-AD-01 re-resolution 2026-05-16, the engine choice is delegated here with a DQ. **Recommendation: `maud`.** Evidence (all verbatim from codebase Explore, 2026-05-16):

| Factor | maud | askama | Source (file:line) |
|---|---|---|---|
| New workspace deps | **1** (`maud`, `actix-web` feature) | 2 (`askama` + `askama_web`; old `askama_actix` is stale vs actix-web 4.13) | `Cargo.toml:184` (`actix-web = "4.13.0"`) |
| New non-Rust files | **0** (templates are `html!{}` macros inline in `.rs`) | `templates/*.html` dir (fresh — none exists) | Glob: no `templates/` anywhere under `crates/` |
| `build.rs` / `askama.toml` | **none** | none for modern askama (proc-macro codegen), but `templates/` dir + recompile coupling | only `diesel_utils` + `email` have `build.rs`; no `askama.toml` anywhere |
| actix Responder | **native** (`Markup: Responder` with `actix-web` feature) | manual `.content_type("text/html…").body(s)` (or `askama_web` shim) | `admin_audit_stream.rs:124-126,254-260` (manual non-JSON response precedent) |
| Codebase idiom fit | **high** — "everything is Rust source" (cf. SSE frames built inline) | medium — introduces an external-file template dir to an API-only codebase | §4 Explore: zero HTML/template precedent in workspace |
| First HTML in workspace | yes (either) | yes (either) | — |

maud minimises new supply-chain surface, adds zero new file-type infrastructure, and matches how the only existing non-JSON response (SSE) is constructed (inline, in Rust). askama's compile-time template checking is real but its advantage is diluted here: the templates are small (2 pages), the data is strongly typed already (`AdminDashboardResponse`), and maud's `html!{}` is itself compile-time-checked Rust. **DQ entry filed (see §19)** — if backend-lead picks askama, Task 1 installs `askama` + `askama_web` + creates `crates/api/api/templates/governance/` + the two `.html` templates, and Tasks 2–4 wire `Template::render()`; everything else in this plan is unchanged (engine-parameterised).

## 6. Relationship to other v1-AD sub-phases

- **Depends on (all SHIPPED):** v1-AD-a (`governance.dashboard.html_pages_enabled` config key + `ConfigKeyMetadata` + default const), v1-AD-b (`is_admin` admin-gate pattern, config accessor family), v1-AD-c (rule-set pointer feeding `rule_sets_summary`), **v1-AD-d (`admin_dashboard` aggregate + `AdminDashboardResponse` DTO tree + `admin_audit_stream` SSE + `project_to_audit_entry` — the exact substrate this plan renders)**.
- **Followed by (deferred):** **v1-AD-f** (the three remaining PRD §6 pages — Config-editor, Single-key-editor, Rule-set-manager — each needs net-new read-path Rust; out of scope here per §7 scope-cut DQ). v1.x/v2 React/SPA frontend pass per ADR-010 (orthogonal — superset, not a continuation of server-rendered pages).
- **Parallel-safe with:** v1-ship-1 (AGPL §13 source-disclosure: `GetSiteResponse.source_disclosure` + `GET /api/v4/source`). Zero file overlap — v1-ship-1 touches `api_common` site DTOs + a public route; v1-AD-e touches `crates/api/api/src/governance/` + `crates/api/routes/src/lib.rs` governance block + `crates/api/api/Cargo.toml`. Different lanes, different phase branches per `.claude/rules/multi-lane-worktree.md` (v1-AD-e gets its own `brehon-fork-ad-e` worktree after bm-cut).

## 7. Preflight guardrails + scope-cut DQ

Non-negotiable for this plan; the planner-side DoD smoke (§15) respects them:

- **R1:** every `i32 ↔ i64` comparison uses `i64::from(...)`, never `as` cast (per `feedback_clippy_test_style.md`). Relevant where the dashboard template renders `[i64; 5]` reputation buckets / `i64` counts — no casts in template arithmetic.
- **R5:** Task 0 enumerates ALL probes explicitly; do NOT inherit implicitly.
- **R6:** all clippy invocations use `--no-deps` uniformly.
- **R7:** `cargo test --no-run -p lemmy_server --test e2e` after any task touching a struct or re-export (Task 2's `gather_dashboard` extraction changes a function's call surface — R7 applies after Task 2).
- **R-html-1 (new, this plan):** the HTML handlers MUST be placed in `crates/api/api/src/governance/` (same crate as `admin_dashboard`) so the `pub(crate)` data-gathering fns and `project_to_audit_entry` are reachable. Placing them in `lemmy_routes` or a new crate is a hard refusal — it would force either `pub` API-surface widening (out of scope) or logic duplication (behaviour-drift risk). Per Explore Category 6 finding.
- **R-html-2 (new, this plan):** no JS framework, no bundler, no npm. The audit page's live tail is **vanilla `EventSource`** in an inline `<script>` block only (per PRD §6.1 "No JS frameworks; vanilla HTML + minimal `<script>`" and ADR-010 React-is-v2). A `package.json` / build-step addition is a hard refusal.
- **R-html-3 (new, this plan):** both pages gate on `governance.dashboard.html_pages_enabled` via the **existing** config accessor (the same read path v1-AD-b established). Do not add a new config key, do not hardcode the gate. When the key resolves `false`, return 404 (route-not-found semantics — the feature is off), not 403 (which means "you're not allowed").

### 7.1 Scope-cut DQ (advisor → user, pre-seeded resolved by planner recommendation)

PRD §6 lists **5 pages**; v1-AD-d shipped read paths for only **2** (Dashboard via `AdminDashboardResponse`, Audit via `project_to_audit_entry` + `/audit/stream`). The other 3 (Config-editor — needs `CONFIG_KEY_METADATA` form-render; Single-key-editor — needs per-key history read; Rule-set-manager — needs `rule_set_version` list read) each require **net-new read-path Rust**, contradicting the "pure templating, zero new Rust DB paths" framing from v1-AD-d §19.3 and the OQ-V1-AD-01 re-resolution's "shipped-read-path-bounded subset" instruction.

**Planner recommendation (DQ pre-seed — see §19 DQ #1):** v1-AD-e ships **Dashboard + Audit only**. Config-editor / Single-key-editor / Rule-set-manager defer to **v1-AD-f** (a follow-up sub-phase that pairs each page with its read-path task). Rationale: keeps v1-AD-e a true pure-presentation sub-phase (lowest risk for the workspace's first HTML), keeps the PR review surface small, and the 3 deferred pages are genuinely a different shape of work (read-path + template, not template-only). **User signs off the cut line via the DQ before plan approval.**

## 8. Flow design

### 8.1 Dashboard page request flow (Task 2)

```
Browser GET /api/v4/governance/admin/dashboard/view
  → actix middleware populates request extensions (LocalUserView)
  → admin_dashboard_html(context, local_user_view)
      → is_admin(&local_user_view)?              [403 NotAnAdmin if not admin]   (mirror admin_dashboard.rs:42)
      → html_pages_enabled = <config accessor>("governance.dashboard.html_pages_enabled")
          → if false → HttpResponse::NotFound() (feature off)                    (R-html-3)
      → resp: AdminDashboardResponse = gather_dashboard(conn, &mut cache, &mut pool).await?
          ↑ EXTRACTED from existing admin_dashboard() body (Task 2 refactor)
            count_active_cases / count_jury_queue / list_recent_config_changes /
            federation_summary / reputation_instance_scope / rule_sets_summary
      → html: String = render_dashboard(&resp)    [maud html!{} OR askama Template::render]
      → HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)   (mirror admin_audit_stream.rs:124-126 body idiom)
```

### 8.2 Audit page request flow (Task 3 + Task 4 client script)

```
Browser GET /api/v4/governance/admin/audit/view
  → admin_audit_html(context, local_user_view)
      → is_admin? → html_pages_enabled? (same gates as 8.1)
      → recent: Vec<AdminConfigAuditEntry> = list_recent_config_changes(conn).await?   (reuse, pub(crate))
      → html = render_audit(&recent)   — table of last-N rows + an inline <script> block:
            const es = new EventSource("/api/v4/governance/admin/audit/stream");
            es.addEventListener("admin_config_changed",      e => prependRow(JSON.parse(e.data)));
            es.addEventListener("admin_config_change_denied", e => prependRow(JSON.parse(e.data)));
        (frame format event:<kind>\ndata:<AdminConfigAuditEntry-json>\n\n — verbatim from admin_audit_stream.rs:245-248)
      → HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
```

### 8.3 Route registration (Task 6 wiring inside Task 2/3 commits)

New routes added adjacent to the existing JSON routes inside the **same** `scope("/governance").service(scope("/admin"))` block (`crates/api/routes/src/lib.rs`, insertion anchor between line 544 `/dashboard` JSON route and the `/config` sub-scope). Result:

```
/api/v4/governance/admin/dashboard          (existing JSON — v1-AD-d)
/api/v4/governance/admin/dashboard/view     (NEW HTML — Task 2)
/api/v4/governance/admin/audit/stream        (existing SSE — v1-AD-d)
/api/v4/governance/admin/audit/view          (NEW HTML — Task 3)
```

> **Route-naming note:** PRD §6 nominally writes the page URLs as `/admin/governance/dashboard`. The shipped API tree is firmly `/api/v4/governance/admin/...`. The plan adopts the `…/admin/dashboard/view` and `…/admin/audit/view` suffix form (a) to stay inside the existing scope (inheriting the established admin nesting + rate-limit wrap), (b) to avoid a second top-level scope, (c) so the JSON and HTML representations of the same resource share a path prefix (`/dashboard` vs `/dashboard/view`). This is a deviation from PRD §6's literal URL spelling but not its intent (a server-rendered admin page for the dashboard). Recorded in §19; if backend-lead wants the literal `/admin/governance/*` spelling, it becomes a separate top-level scope — flagged in DQ #1's options.

## 9. Mandatory reading

The impl-task subagent MUST Read these before its first edit:

- **Schema/type definitions:**
  - `crates/api/api_common/src/governance.rs:652-727` — `AdminDashboardResponse` + all 6 sub-structs (`ActiveCasesSummary`, `JuryQueueSummary`, `FederationSummary`, `RuleSetSummary`, `PerCommunityActiveRuleSet`) — every field the dashboard template renders.
  - `crates/api/api_common/src/governance.rs:395-401` + `:351-387` — `AdminReputationStatsResponse` + `ReputationBuckets`/`ThresholdsSnapshot`/`CapabilityCounts`/`FounderEventStats` (the `reputation` field tree).
  - `crates/api/api_common/src/governance.rs:555-573` — `AdminConfigAuditEntry` (the audit-row + `recent_config_changes` element type; note `signature: Option<Vec<u8>>` render as presence/hex, `new_value`/`previous_value` are `serde_json::Value`).
- **Existing patterns (MIRROR refs):**
  - `crates/api/api/src/governance/admin_dashboard.rs:38-64` — the handler whose body Task 2 extracts; the `is_admin → gather → Json` shape; imports at `:22-33`.
  - `crates/api/api/src/governance/admin_audit_stream.rs:108-126,245-248,254-260` — non-JSON `HttpResponse` body idiom (the HTML response mirror), SSE frame format (the client `<script>` contract), per-admin 409 plain-body precedent.
  - `crates/api/api/src/governance/admin_reputation_stats.rs:33-44` — clean admin-gated GET handler exemplar (import shape).
  - `crates/api/api/src/governance/config.rs:1177-1182,2317-2328` — the `governance.dashboard.html_pages_enabled` default const + `ConfigKeyMetadata` entry; **find the existing config accessor fn the v1-AD-b handlers use to read a bool instance key** (the `get_*_opt` / `ConfigCache` family referenced in `admin_dashboard.rs:46` `ConfigCache::new()`) and mirror that read for the gate.
  - `crates/api/routes/src/lib.rs:35-44,520-558` — governance handler imports + the full `scope("/governance")` tree; insertion anchor at `:544`.
- **Adjacent test fixtures:**
  - `crates/server/tests/e2e.rs` — locate the most recent `v1-AD-*` / `v1-SL-*` / `v1-JM-*` fixtures sibling module (admin-gated endpoint test with seeded admin LocalUser + JWT + reqwest call). Mirror its `LemmyResult<()>` outer + helper signatures **verbatim** (Case A per `feedback_lemmy_error_no_std_error.md`). Read the cited line range BEFORE authoring the test edit (canonical-schema-first).
- **Lessons (each gates a task):** `feedback_library_add_after_shipping.md` (Task 1), `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md` + `feedback_junior_worker_e2e_edit_hang.md` (Task 5), `feedback_clippy_rerun_after_fix.md` + `feedback_clippy_test_style.md` (§15.2 every task), `feedback_windows_e2e_requires_bat_wrapper.md` (§15.4).

## 10. Patterns to mirror

### 10.1 Non-JSON HttpResponse body (the HTML render → response)

**Mirror:** `crates/api/api/src/governance/admin_audit_stream.rs:123-126` (the 409 plain-body branch) and `:254-260` (the streaming branch). The HTML handlers use the **body** form, not streaming:

```rust
// SOURCE: admin_audit_stream.rs:123-126 (adapt content_type + body)
HttpResponse::Ok()
  .content_type("text/html; charset=utf-8")
  .body(html_string)   // String from render_dashboard(&resp) / render_audit(&recent)
```

For **maud**: `maud::html! { ... }` returns `Markup`; `Markup` implements actix `Responder` (with the `actix-web` feature) and serialises as `text/html; charset=utf-8` automatically — the handler may return `LemmyResult<Markup>` and skip the manual wrap, OR use `.body(markup.into_string())` with the explicit wrap above for uniformity with the SSE precedent. For **askama**: `template.render()? -> String`, then the explicit wrap above (or the `askama_web` Responder if that dep is taken).

### 10.2 Admin-gated GET handler preamble (auth + feature-gate)

**Mirror:** `crates/api/api/src/governance/admin_dashboard.rs:38-46`:

```rust
// SOURCE: admin_dashboard.rs:38-46 — copy the preamble shape
pub async fn admin_dashboard_html(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<HttpResponse> {        // HttpResponse (not Json) — HTML body
  is_admin(&local_user_view)?;          // 403 NotAnAdmin path delegated (mirror :42)
  // NEW: feature-gate (R-html-3) — read governance.dashboard.html_pages_enabled
  //   via the SAME config accessor v1-AD-b handlers use for instance bools.
  //   if !enabled → return Ok(HttpResponse::NotFound().finish());
  let mut cache = ConfigCache::new();   // mirror :44
  let mut pool = context.pool();
  let conn = &mut get_conn(&mut pool).await?;   // mirror :45-46
  // ... gather + render ...
}
```

### 10.3 Dashboard data-gathering extraction (Task 2 refactor — one source of truth)

**Mirror:** `crates/api/api/src/governance/admin_dashboard.rs:38-64` (the existing body). Extract lines `:48-63` (the six `let … = <fn>(…).await?;` + the `AdminDashboardResponse { … }` construction) into:

```rust
// NEW pub(crate) fn in admin_dashboard.rs — both handlers call this
pub(crate) async fn gather_dashboard(
  conn: &mut <conn type from :46>,
  cache: &mut ConfigCache,
  pool: &mut <pool type from :47>,
) -> LemmyResult<AdminDashboardResponse> {
  let active_cases = count_active_cases(conn).await?;
  let jury_queue = count_jury_queue(conn).await?;
  let recent_config_changes = list_recent_config_changes(conn).await?;
  let federation = federation_summary(conn).await?;
  let reputation = reputation_instance_scope(conn, cache, pool).await?;
  let rule_sets = rule_sets_summary(conn).await?;
  Ok(AdminDashboardResponse {
    active_cases, jury_queue, recent_config_changes,
    federation, reputation, rule_sets, calculated_at: Utc::now(),
  })
}
// admin_dashboard() body becomes: is_admin?; setup; Ok(Json(gather_dashboard(...).await?))
```

> **GOTCHA:** read the EXACT param types of `conn`/`pool` at `admin_dashboard.rs:45-47` and the exact signatures of the six gathered fns before extracting — the plan-time text above is shape-illustrative; the impl agent copies real types from the file. The extraction must be **behaviour-preserving** (R7 `cargo test --no-run` after Task 2 proves the JSON handler still compiles + the existing v1-AD-d e2e dashboard test still passes).

### 10.4 Feature-flag config read (the `html_pages_enabled` gate)

**Mirror:** the bool-instance-key read pattern v1-AD-b established. `admin_dashboard.rs:44` constructs `ConfigCache::new()`; the reputation gather at `:52` already threads `&mut cache`. Find the accessor that reads a single instance bool from config (search `crates/api/api/src/governance/config.rs` for the `get_*_opt` / `effective_*` family + how `admin_set_config`/`admin_get_config` read instance-scoped bools, and how `DEFAULT_GOVERNANCE_DASHBOARD_HTML_PAGES_ENABLED` at `config.rs:1177-1179` is the fallback). The gate is: read effective `governance.dashboard.html_pages_enabled` (scope: instance) → default `true` if no row → `false` means feature-off → 404.

> **GOTCHA:** the key is `ConfigScope::Instance` (verbatim `config.rs:2317-2328`). Do not pass a community scope. Default-true means the common case (no override row) serves pages — the e2e test must assert both the default-on path AND an explicit-off path (Task 5 seeds an `html_pages_enabled = false` row and asserts 404).

### 10.5 e2e admin-gated test fixture

**Mirror:** the most-recent `v1-AD-*`/`v1-SL-*`/`v1-JM-*` admin-endpoint test in `crates/server/tests/e2e.rs` (impl agent locates it via §9 reading). Verbatim mirror its: `async fn … -> LemmyResult<()>` outer, the seed-admin-LocalUser + JWT helper calls, the `reqwest`/in-process call to the endpoint, the status + body assertions. **Case A** per `feedback_lemmy_error_no_std_error.md` (sibling uses `LemmyResult<()>` — flip ALL helpers to `LemmyResult<T>`, no `.map_err` bridges).

## 11. Files to change

Grouped by crate. Each path verified to exist in the workspace (Explore, 2026-05-16) except the `creates:` new files.

**`lemmy_api` (`crates/api/api/`):**
- `crates/api/api/src/governance/admin_dashboard_html.rs` — **CREATE** — both HTML handlers (`admin_dashboard_html`, `admin_audit_html`) + the engine render fns (`render_dashboard`, `render_audit`) [iff maud: templates are inline `html!{}` here; iff askama: this holds the `#[derive(Template)]` structs] (Task 2, Task 3, Task 4).
- `crates/api/api/src/governance/admin_dashboard.rs` — **MODIFY** — extract `gather_dashboard` `pub(crate) fn`; `admin_dashboard()` body delegates to it (Task 2).
- `crates/api/api/src/governance/mod.rs` — **MODIFY** — add `pub(crate) mod admin_dashboard_html;` (alphabetical, near the existing `admin_dashboard` decl) (Task 2).
- `crates/api/api/Cargo.toml` — **MODIFY** — add the engine dep: iff maud → `maud = { version = "0.26", features = ["actix-web"] }` (inline single-consumer precedent — mirror `sitemap-rs`/`totp-rs` at lines 70-71); iff askama → `askama` + `askama_web` (or workspace-dep form). Exact version pinned at Task 1 after `cargo add` resolves against actix-web 4.13 (Task 1).
- `Cargo.lock` — **MODIFY** (generated) — dependency resolution from the Task 1 dep add (Task 1; committed with Task 1 per `feedback_commit_hygiene_lockfiles_and_task_labels`).

**Templates (iff askama wins DQ #2 — otherwise NOT created):**
- `crates/api/api/templates/governance/dashboard.html` — **CREATE (conditional)** — askama dashboard template (Task 2).
- `crates/api/api/templates/governance/audit.html` — **CREATE (conditional)** — askama audit template + inline `<script>` (Task 3, Task 4).

**`lemmy_routes` (`crates/api/routes/`):**
- `crates/api/routes/src/lib.rs` — **MODIFY** — import `admin_dashboard_html::{admin_dashboard_html, admin_audit_html}` (near `:35-44`); add 2 `.route(...)` lines inside the `scope("/admin")` block at the `:544` anchor (Task 2 adds the dashboard route in its commit; Task 3 adds the audit route in its commit).

**`lemmy_server` (`crates/server/`):**
- `crates/server/tests/e2e.rs` — **MODIFY** — append one fixtures module / test fn covering: (a) admin GET `/dashboard/view` → 200 + `text/html` + a known marker string from seeded data; (b) non-admin → 403; (c) `html_pages_enabled=false` → 404 (Task 5, single append-only edit — `feedback_junior_worker_e2e_edit_hang`).

### Struct-field add: enumerate all callsites

**N/A — this plan adds no struct fields.** `gather_dashboard` is a new function extracted from existing code; `AdminDashboardResponse` and all DTOs are consumed read-only and unchanged. No `rg`-enumeration needed (per template §11 skip condition: no public-struct field additions).

## 12. NOT building in v1-AD-e

- **Config-editor page** (`GET /admin/governance/config` schema-driven form) — deferred to **v1-AD-f**; reason: needs net-new `CONFIG_KEY_METADATA` form-render read path, not pure templating.
- **Single-key-editor page** (`/admin/governance/config/<key>`) — deferred to **v1-AD-f**; reason: needs per-key history read path + a dry-run POST-preview JS interaction (a new surface).
- **Rule-set-manager page** (`/admin/governance/rule-sets/<community_id>`) — deferred to **v1-AD-f**; reason: needs a `rule_set_version` list read path; not shipped by v1-AD-d.
- **Per-community dashboard** (`/admin/dashboard?community_id=X`) — deferred to v1.x; reason: v1-AD-d aggregate is instance-wide only (per v1-AD-d §19.9).
- **Any JS framework / SPA / npm / bundler** — v2 React pass per ADR-010 (unchanged); reason: PRD §6.1 + R-html-2 mandate vanilla HTML + minimal inline `<script>`.
- **Caching of the HTML render** (moka/TTL) — v1.x if pilots report slow loads; reason: v1-AD-d explicitly deferred dashboard caching; the HTML layer inherits that deferral.
- **Auth changes** — pages reuse `is_admin` + `LocalUserView` exactly; no step-up, no new capability (PRD §7.1 `GET /admin/dashboard` = `is_admin`).
- **New config keys** — `html_pages_enabled` is already seeded; no new key (R-html-3).
- **Modifying `admin_dashboard` / `admin_audit_stream` behaviour** — Task 2 is a behaviour-preserving extraction only; the JSON/SSE responses are byte-identical post-refactor (R7 proves it).

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task.** Pre-Shape-G (Shape G suspended) → each task's validation is the inline cargo DoD (§15), run on the laptop via validate-pending-laptop (impl-task pushes; advisor-laptop runs §15 commands; per `advisor-orchestrator.md` §5.2). No `[P]` markers — Tasks 2→3→4→5 are a strict dependency chain (3 needs 2's module; 4 edits 3's file; 5 tests 2+3); cohort parallelism yields nothing here. Task 0 is the barrier.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment ready for `v1-AD-e`; branch is `phase-v1-AD-e`; v1-AD-d deliverables intact on base; clippy baseline clean; engine DQ resolved.

**Probes (R5 — enumerate ALL explicitly):**

```bash
# Probe 0 — branch is phase-v1-AD-e
test "$(git branch --show-current)" = "phase-v1-AD-e" && echo "BRANCH OK" || { echo "WRONG BRANCH"; exit 1; }

# Probe 1 — v1-AD-d substrate intact on base: AdminDashboardResponse present
grep -q "pub struct AdminDashboardResponse" crates/api/api_common/src/governance.rs && echo "AD-d DTO OK" || { echo "AD-d DTO MISSING"; exit 1; }

# Probe 2 — admin_dashboard handler + gather targets present
grep -q "pub async fn admin_dashboard" crates/api/api/src/governance/admin_dashboard.rs && echo "AD-d HANDLER OK" || { echo "MISSING"; exit 1; }

# Probe 3 — config key seeded (v1-AD-a)
grep -q "governance.dashboard.html_pages_enabled" crates/api/api/src/governance/config.rs && echo "FLAG KEY OK" || { echo "FLAG KEY MISSING"; exit 1; }

# Probe 4 — route anchor present (lib.rs :544 region)
grep -q '\.route("/dashboard", get()\.to(admin_dashboard))' crates/api/routes/src/lib.rs && echo "ROUTE ANCHOR OK" || { echo "ROUTE ANCHOR MOVED — re-locate"; exit 1; }

# Probe 5 — engine DQ resolved (DQ #2 in §19 must be in resolved[] with an engine answer)
python -c "import json,io; d=json.load(io.open('.claude/decision-queue.json',encoding='utf-8')); ids=[e for e in d['resolved'] if 'v1-AD-e' in str(e.get('question','')) and 'engine' in str(e.get('question','')).lower()]; print('ENGINE DQ RESOLVED' if ids else 'ENGINE DQ UNRESOLVED'); exit(0 if ids else 1)"

# Probe 6 — scope-cut DQ resolved (DQ #1)
python -c "import json,io; d=json.load(io.open('.claude/decision-queue.json',encoding='utf-8')); ids=[e for e in d['resolved'] if 'v1-AD-e' in str(e.get('question','')) and ('scope' in str(e.get('question','')).lower() or 'page' in str(e.get('question','')).lower())]; print('SCOPE DQ RESOLVED' if ids else 'SCOPE DQ UNRESOLVED'); exit(0 if ids else 1)"

# Probe 7 — clippy baseline clean on lemmy_api (pre-existing state)
cmd //c "scripts\\brehon\\cargo-clippy.bat -p lemmy_api --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-AD-e-task0-clippy-baseline.log 2>&1"
echo "clippy baseline exit: $?"   # EXPECT 0

# Probe 8 (NEGATIVE — exit-code propagation sanity) — a guaranteed-fail grep must exit non-zero
grep -q "THIS_STRING_DOES_NOT_EXIST_ANYWHERE_v1ADe" crates/api/api/src/governance/mod.rs && echo "NEG FAIL (should not print)" || echo "NEG OK (propagation works)"

# Probe 9 — Docker daemon (e2e Task 5 needs testcontainers later; verify now)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 10 — concurrent-PR check (no open PR touches our files)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("crates/api/api/src/governance/admin_dashboard|crates/api/routes/src/lib.rs|crates/api/api/Cargo.toml")) | {number,title,headRefName}'
# EXPECT empty (v1-ship-1 must not touch these — verify lane isolation)
```

**EXPECT block:** Probes 0–7, 9 exit 0; Probe 8 prints `NEG OK`; Probe 10 empty output. **No commit at Task 0.**

### Task 1: Add HTML engine dependency (isolated commit)

**ACTION:** add the engine dep resolved by DQ #2 to `crates/api/api/Cargo.toml`; regenerate `Cargo.lock`; capture clippy-narrowed log immediately to catch a dependency-tree-rebuild lint cascade (per `feedback_library_add_after_shipping.md`).

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/Cargo.toml          # add engine dep (maud OR askama+askama_web per DQ #2)
  - Cargo.lock                          # generated resolution
```

**IMPLEMENT (file 1 of 2):** in `crates/api/api/Cargo.toml` `[dependencies]`, add — **iff DQ #2 = maud:** `maud = { version = "<pin>", features = ["actix-web"] }` (inline single-consumer form, mirror `sitemap-rs = "0.4.0"` / `totp-rs = {…}` at lines 70-71). **iff DQ #2 = askama:** `askama = "<pin>"` and `askama_web = { version = "<pin>", features = ["actix-web-4"] }` (or the current askama→actix integration crate name `cargo add` resolves). Pin = whatever `cargo add` selects compatible with `actix-web 4.13.0` — record the exact resolved version in the commit body.

**IMPLEMENT (file 2 of 2):** `Cargo.lock` regenerated by the `cargo add` / `cargo check`. Commit it with this task (lockfile co-commits with the dep change per `feedback_commit_hygiene_lockfiles_and_task_labels`).

**MIRROR:** `crates/api/api/Cargo.toml:70-71` (`sitemap-rs`/`totp-rs` inline single-consumer dep precedent).

**GOTCHA:** the old `askama_actix` crate is **stale** vs actix-web 4.13 (Explore finding) — if askama wins, use the current integration shim (`askama_web` with the actix-web-4 feature), NOT `askama_actix`. For maud, the `actix-web` feature gives `Markup: Responder` directly — no shim. After the dep add, the dependency tree rebuilds; an unrelated transitive lint can surface (per `feedback_library_add_after_shipping`) — the clippy run below is the detector, not optional.

**VALIDATE:**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-AD-e-task1-check.log 2>&1"
echo "exit: $?"; tail -20 .claude/PRPs/debug/v1-AD-e-task1-check.log    # EXPECT 0
cmd //c "scripts\\brehon\\cargo-clippy.bat -p lemmy_api --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-AD-e-task1-clippy.log 2>&1"
echo "clippy exit: $?"; tail -30 .claude/PRPs/debug/v1-AD-e-task1-clippy.log
# EXPECT clippy exit 0. If a transitive cascade appears, the log shows it — STOP, surface to advisor (do NOT #[allow]-spam).
```

### Task 2: Extract `gather_dashboard` + dashboard HTML handler + route

**ACTION:** behaviour-preservingly extract the dashboard data-gathering from `admin_dashboard()` into `pub(crate) async fn gather_dashboard`; create `admin_dashboard_html.rs` with `admin_dashboard_html` (auth + feature-gate + gather + render); register the module + the `/dashboard/view` route.

**FILES:**

```yaml
creates:
  - crates/api/api/src/governance/admin_dashboard_html.rs
modifies:
  - crates/api/api/src/governance/admin_dashboard.rs   # extract gather_dashboard; body delegates
  - crates/api/api/src/governance/mod.rs                # add pub(crate) mod admin_dashboard_html;
  - crates/api/routes/src/lib.rs                        # import + .route("/dashboard/view", get().to(admin_dashboard_html))
requires:
  - task: 1
    reason: render fn uses the engine dep added in Task 1 (won't compile without it)
```

**IMPLEMENT (file 1 of 4):** in `admin_dashboard.rs`, extract lines `:48-63` into `pub(crate) async fn gather_dashboard(conn, cache, pool) -> LemmyResult<AdminDashboardResponse>` (verbatim shape §10.3, real param types copied from `:45-47`); rewrite `admin_dashboard()` body to `is_admin?; <setup>; Ok(Json(gather_dashboard(conn, &mut cache, &mut pool).await?))`.

**IMPLEMENT (file 2 of 4):** create `admin_dashboard_html.rs` — `admin_dashboard_html(context, local_user_view) -> LemmyResult<HttpResponse>` per §10.2 preamble: `is_admin?` → read `governance.dashboard.html_pages_enabled` via the existing config accessor (§10.4) → if `false` `Ok(HttpResponse::NotFound().finish())` → else `let resp = gather_dashboard(...).await?` → `let html = render_dashboard(&resp)` → `Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html))`. Add `render_dashboard(&AdminDashboardResponse) -> String` — **iff maud:** `maud::html! { … }` rendering every field of the response tree (active_cases.by_status BTreeMap loop, jury_queue 3 counts, federation 3 counts, rule_sets per_community ≤100 loop, reputation 4 bucket arrays + 3 thresholds + capability/founder counts, calculated_at). **iff askama:** a `#[derive(Template)] #[template(path="governance/dashboard.html")] struct DashboardPage<'a>{ resp: &'a AdminDashboardResponse }` + `.render()?`.

**IMPLEMENT (file 3 of 4):** in `mod.rs`, add `pub(crate) mod admin_dashboard_html;` alphabetically adjacent to the `admin_dashboard` declaration.

**IMPLEMENT (file 4 of 4):** in `lib.rs`, add to the governance handler import block (`:35-44`) `admin_dashboard_html::{admin_dashboard_html, admin_audit_html}` (audit_html lands Task 3 — import both now is fine only if Task 3 lands before compile; to keep Task 2 independently-compiling, import ONLY `admin_dashboard_html::admin_dashboard_html` here and add `admin_audit_html` to the import in Task 3). Add `.route("/dashboard/view", get().to(admin_dashboard_html))` immediately after the existing `:544` `/dashboard` JSON route.

**MIRROR:** `admin_dashboard.rs:38-64` (handler+gather shape), `admin_audit_stream.rs:123-126` (HTML body idiom), `lib.rs:544` (route insertion).

**GOTCHA:** copy the EXACT `conn`/`pool`/`cache` types from `admin_dashboard.rs:44-47` — do not guess. The extraction must be byte-behaviour-preserving (R7 below proves the JSON handler + the existing v1-AD-d dashboard e2e test still pass). `ActiveCasesSummary.by_status` is a `BTreeMap<String,i64>` (stable order — safe template `for (k,v)`). `ReputationBuckets` fields are `[i64;5]` fixed arrays (R1 — no `as` casts rendering them). Feature-gate returns **404** not 403 (R-html-3).

**VALIDATE:**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-AD-e-task2-check.log 2>&1"
echo "exit: $?"; tail -20 .claude/PRPs/debug/v1-AD-e-task2-check.log   # EXPECT 0
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-AD-e-task2-clippy.log 2>&1"
echo "clippy exit: $?"; tail -20 .claude/PRPs/debug/v1-AD-e-task2-clippy.log   # EXPECT 0
# R7 — extraction is behaviour-preserving:
cmd //c "scripts\\brehon\\cargo-test.bat --no-run -p lemmy_server --test e2e > .claude/PRPs/debug/v1-AD-e-task2-testnorun.log 2>&1"
echo "test --no-run exit: $?"   # EXPECT 0
```

### Task 3: Audit HTML handler + route

**ACTION:** add `admin_audit_html` to `admin_dashboard_html.rs` (auth + feature-gate + reuse `list_recent_config_changes` + render table); register the `/audit/view` route + complete the lib.rs import.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/admin_dashboard_html.rs   # add admin_audit_html + render_audit
  - crates/api/routes/src/lib.rs                            # add admin_audit_html to import + .route("/audit/view", ...)
requires:
  - task: 2
    reason: admin_audit_html lives in the module created by Task 2; reuses render-helper conventions established there
```

**IMPLEMENT (file 1 of 2):** in `admin_dashboard_html.rs`, add `admin_audit_html(context, local_user_view) -> LemmyResult<HttpResponse>`: same `is_admin?` + `html_pages_enabled?`-404 preamble → `let recent: Vec<AdminConfigAuditEntry> = list_recent_config_changes(conn).await?` (reuse the `pub(crate)` fn from `admin_dashboard.rs`) → `let html = render_audit(&recent)` → `text/html` body. `render_audit(&[AdminConfigAuditEntry]) -> String` renders a table (id, created_at, entry_kind, scope, key, previous→new value via `serde_json::Value` `.to_string()`, actor_pseudonym, signature-present bool, denial_reason) **plus the inline `<script>` block from Task 4**.

**IMPLEMENT (file 2 of 2):** in `lib.rs`, extend the governance import to `admin_dashboard_html::{admin_dashboard_html, admin_audit_html}`; add `.route("/audit/view", get().to(admin_audit_html))` adjacent to the existing `scope("/audit").route("/stream", …)` service (so both `/audit/stream` and `/audit/view` are siblings).

**MIRROR:** `admin_dashboard_html.rs` (Task 2's handler shape — copy it), `admin_audit_stream.rs:245-248` (the `event:<kind>\ndata:<json>` contract the client script must match), `governance.rs:555-573` (`AdminConfigAuditEntry` field list).

**GOTCHA:** `signature: Option<Vec<u8>>` — render as `signed: true/false` or hex, never raw bytes into HTML. `new_value`/`previous_value` are `serde_json::Value` — `.to_string()` or match; `previous_value`/`previous_from`/`denial_reason` are `None` on pre-v1-AD-c rows (template must handle `Option`). Escape all string fields (maud auto-escapes; askama auto-escapes with the `.html` extension — confirm autoescape on, do NOT use `|safe`).

**VALIDATE:** same three-command block as Task 2 (`check` + `clippy --no-deps -D warnings` + `test --no-run -p lemmy_server --test e2e`), logs `v1-AD-e-task3-*`. EXPECT all exit 0.

### Task 4: Audit page live-tail `<script>` (vanilla EventSource)

**ACTION:** finalise the inline `<script>` block in `render_audit`'s output — vanilla `EventSource` against `/api/v4/governance/admin/audit/stream`, prepending rows on the two governance event kinds. No framework, no bundler (R-html-2).

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/admin_dashboard_html.rs   # complete the <script> in render_audit
requires:
  - task: 3
    reason: edits render_audit added in Task 3
```

**IMPLEMENT (file 1 of 1):** in `render_audit`, embed (maud `(PreEscaped("<script>…</script>"))` or askama raw-in-template) exactly:

```html
<script>
const es = new EventSource("/api/v4/governance/admin/audit/stream");
function prependRow(e){ const d=JSON.parse(e.data); /* build <tr> from d fields, insertBefore on tbody.firstChild */ }
es.addEventListener("admin_config_changed", prependRow);
es.addEventListener("admin_config_change_denied", prependRow);
es.onerror = () => { /* EventSource auto-reconnects per server 'retry:'; show a muted 'reconnecting' note */ };
</script>
```

The `data` field is a JSON `AdminConfigAuditEntry` (verbatim contract `admin_audit_stream.rs:245-248`). Field names in the JS must match the serde-serialised struct (snake_case per the DTO).

**MIRROR:** `admin_audit_stream.rs:189-252` (frame format the script consumes — `event:` names `admin_config_changed`/`admin_config_change_denied`, `retry: 10000` so the client need not implement backoff).

**GOTCHA:** `EventSource` only fires the default `message` handler unless `addEventListener(<event-name>)` is used — the server sends **named** events (`event: admin_config_changed`), so `es.onmessage` alone would receive nothing. Must use `addEventListener` for both kinds (this is the #1 SSE-client footgun). The per-admin SSE cap (server returns 409 on a second concurrent stream from the same admin) means opening the audit page twice in two tabs → second tab's `EventSource` errors; document this in a one-line HTML comment, do not try to work around it.

**VALIDATE:** `check` + `clippy --no-deps -D warnings` (no `test --no-run` needed — pure string-content change, no struct/re-export touch; R7 N/A). Logs `v1-AD-e-task4-*`. EXPECT exit 0.

### Task 5: e2e test (admin 200 / non-admin 403 / flag-off 404)

**ACTION:** append one fixtures module to `crates/server/tests/e2e.rs` asserting the three behaviours. Single append-only edit (`feedback_junior_worker_e2e_edit_hang` — file >8000 lines, never bundle with handler logic, this is its own task).

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # one new fixtures mod / test fn (append-only)
requires:
  - task: 2
    reason: tests /dashboard/view (Task 2 handler + route)
  - task: 3
    reason: tests /audit/view (Task 3 handler + route)
```

**IMPLEMENT (file 1 of 1):** locate the most-recent `v1-AD-*`/`v1-SL-*`/`v1-JM-*` admin-endpoint fixtures module (per §9 reading); mirror its shape **verbatim** (Case A — `LemmyResult<()>` outer, all helpers `LemmyResult<T>`, no `.map_err` bridges). Test fn(s):
- seed an instance admin LocalUser + obtain JWT (mirror sibling helper); seed minimal governance data so the dashboard has a non-trivial marker (e.g. one open `moderation_case`);
- GET `/api/v4/governance/admin/dashboard/view` with admin JWT → assert `200`, `content-type: text/html`, body contains a known marker (e.g. the seeded case's status string or a stable page heading);
- GET same with a non-admin user's JWT → assert `403`;
- seed `governance.dashboard.html_pages_enabled = false` (instance scope) via the existing config-write path → GET `/dashboard/view` with admin JWT → assert `404`; (optionally also `/audit/view` → `200` admin, `403` non-admin).

**MIRROR:** the sibling e2e fixtures module (impl agent cites the exact line range it mirrored in the commit body — canonical-schema-first per the file-class lesson injection rule). `feedback_async_pool_test_pattern.md` for the `AsyncPgConnection::establish` + `DbPool::Conn` seed step.

**GOTCHA:** the test must seed the admin flag in the same scope the handler reads (`ConfigScope::Instance`). Default is `true` so the 200/403 cases need NO flag row; only the 404 case writes `false`. Use the existing config-write helper/handler — do not raw-INSERT into `governance_config` (mirrors how sibling v1-AD tests set config). e2e runs via the bat wrapper (§15.4) — never bare `cargo test` (libpq.dll / `feedback_windows_e2e_requires_bat_wrapper`).

**VALIDATE:**

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-AD-e-task5-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-AD-e-task5-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-AD-e-task5-e2e.log"
tail -40 .claude/PRPs/debug/v1-AD-e-task5-e2e.log
# EXPECT: new test(s) pass; pre-existing e2e (incl. v1-AD-d dashboard test) still pass; trailer line E2E_EXIT_0
```

### Task 6: Retro

**Goal:** author retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`. One H2 per role (Advisor / Planning / Impl / BM) with three-signal scoring + lessons. Capture per-task complexity (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`). Promote any new lesson (likely candidates: "first-HTML-in-workspace engine-selection heuristic", "maud-vs-askama for actix-web 4.x decision record", "SSE-client `addEventListener` named-event footgun") to `.claude/lessons/feedback_*.md` in the same retro commit (per `feedback_one_system_memory_in_repo.md`). Write to `.claude/PRPs/reports/v1-AD-e-retro.md`.

---

## 14. Testing strategy

Per IMPLEMENTATION-PLAN-v0 §5 — integration-only, e2e in `crates/server/tests/e2e.rs`.

- **Unit (compile-time):** `cargo check --workspace --features full` (every task).
- **Lint:** `cargo clippy --workspace --features full --no-deps -- -D warnings` (every task; R6 uniform).
- **Test-target compile (R7):** `cargo test --no-run -p lemmy_server --test e2e` after Task 2 (extraction changes a fn surface) and Task 3 (new handlers referenced).
- **e2e execution:** `scripts\brehon\cargo-test.bat --workspace --test e2e --features full` — Task 5's new test(s) pass; **all pre-existing e2e pass**, including the shipped v1-AD-d dashboard test (regression guard on the Task 2 extraction).
- **Migration round-trip:** N/A (no migrations).
- **Manual smoke (advisor, post-merge, optional):** `curl -H "Authorization: Bearer <admin-jwt>" http://localhost:8536/api/v4/governance/admin/dashboard/view` → HTML; open in a browser; confirm the audit page's live tail updates when a config write is POSTed in another tab.

## 15. Validation commands (DoD)

> Planner-side: every command dry-run by the advisor against current HEAD before plan approval (per `feedback_plan_dod_dry_run_at_write.md`). All use the bat wrapper + `--features full`; clippy uses `--no-deps` uniformly (R6); no `-p <crate> --features full` combination (per `feedback_features_full_p_crate_incompatible` — workspace-scoped check/clippy, `-p` only for the `--no-run` test-target compile which does not pass `--features full`).

### 15.1 Static analysis (per task)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-AD-e-<task>-check.log 2>&1"
echo "exit: $?"   # EXPECT 0
```

### 15.2 Lint (per task — uniform R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-AD-e-<task>-clippy.log 2>&1"
echo "exit: $?"   # EXPECT 0   (rerun after ANY fix — feedback_clippy_rerun_after_fix)
```

### 15.3 Test-target compile (R7 — after Task 2 and Task 3)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --no-run -p lemmy_server --test e2e > .claude/PRPs/debug/v1-AD-e-<task>-testnorun.log 2>&1"
echo "exit: $?"   # EXPECT 0
```

### 15.4 e2e execution (Task 5; full-suite regression at phase close)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-AD-e-e2e-final.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-AD-e-e2e-final.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-AD-e-e2e-final.log"
tail -40 .claude/PRPs/debug/v1-AD-e-e2e-final.log
# EXPECT: new v1-AD-e tests pass; v1-AD-d dashboard test still passes; trailer E2E_EXIT_0
```

### 15.5 Cross-cutting verification

- [ ] No new `governance_log` emission introduced (pages are read-only — assert zero `governance_log::append` / INSERT added; grep diff).
- [ ] No new DTO, no struct-field add, no migration, no new config key (R-html-3).
- [ ] HTML handlers in `crates/api/api/src/governance/` only (R-html-1) — no `lemmy_routes`/new-crate handler logic.
- [ ] No `package.json` / npm / bundler / JS-framework import (R-html-2) — vanilla inline `<script>` only.
- [ ] Feature-off path returns **404**, not 403 (R-html-3) — asserted by Task 5.
- [ ] `is_admin` gate is the first statement in both handlers (mirror `admin_dashboard.rs:42`); 403 path delegated to `LemmyError` (no hand-rolled 403).
- [ ] Task 2 extraction behaviour-preserving — v1-AD-d dashboard e2e test still green (§15.4).
- [ ] All template string interpolation auto-escaped (maud default / askama `.html` autoescape) — no `|safe`, no raw `PreEscaped` except the controlled `<script>` literal in Task 4.
- [ ] R1: no `as` casts in template rendering of `i64`/`[i64;5]` values.
- [ ] R5: Task 0 enumerated all 11 probes. R6: all clippy `--no-deps`.
- [ ] ADRs unchanged: no edit to `99-decisions-and-open-questions.md` from this phase (OQ-V1-AD-01 re-resolution + changelog were applied BEFORE this plan, by the user, in the sibling repo — this plan only *cites* it).

### 15.6 DoD per workflow (Shape G) — N/A

Shape G is **suspended until 2026-06-01** (`project_shape_g_suspended_2026_05_16`). v1-AD-e runs **validate-pending-laptop**: impl-task pushes; advisor-laptop runs §15.1–15.4 sequentially per `advisor-orchestrator.md` §5.2; mutates the `kind: "validate-pending-laptop"` DQ entry. If v1-AD-e is still in flight after 2026-06-01 and Shape G is re-enabled (DQ #229), the §15 commands map 1:1 onto the workflow YAML — no plan change needed (the cargo command shapes are identical).

---

## 16. Acceptance criteria

- [ ] All 7 tasks completed in dependency order (0→1→2→3→4→5→retro).
- [ ] §15.1 (cargo check `--workspace --features full`) exit 0 after every task.
- [ ] §15.2 (cargo clippy `--no-deps -- -D warnings`) exit 0 after every task.
- [ ] §15.3 (cargo test `--no-run`) exit 0 after Task 2 and Task 3.
- [ ] §15.4 e2e — new v1-AD-e test(s) pass; **all** pre-existing e2e pass (v1-AD-d dashboard regression guard).
- [ ] §15.5 cross-cutting — all boxes ticked.
- [ ] §16a stories all `[done]`.
- [ ] No edits to files outside the §11 list.
- [ ] Retro committed (§13 Task 6).
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`.
- [ ] DQ #1 (scope cut) + DQ #2 (engine) both resolved before Task 1.

## 16a. Stories

### Story 1: An instance admin sees the governance dashboard as a web page

- **Composing tasks:** Task 1 (engine dep), Task 2 (gather extraction + dashboard handler + route).
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"` — the Task 5 dashboard sub-assertions (admin 200 + `text/html` + marker; non-admin 403; flag-off 404).
- **Expected output:** dashboard test cases pass; v1-AD-d dashboard JSON test still passes.
- **Brief-Scope outputs to verify (`/brehon-verify`):**
  - `crates/api/api/src/governance/admin_dashboard_html.rs` exists, non-empty, contains `pub async fn admin_dashboard_html`.
  - `crates/api/api/src/governance/admin_dashboard.rs` contains `pub(crate) async fn gather_dashboard`.
  - `crates/api/api/src/governance/mod.rs` contains `mod admin_dashboard_html`.
  - `crates/api/routes/src/lib.rs` contains `.route("/dashboard/view"`.

### Story 2: An instance admin watches config changes live in a web page

- **Composing tasks:** Task 3 (audit handler + route), Task 4 (EventSource script).
- **Checkpoint command:** same e2e binary — Task 5 audit sub-assertions (admin 200 `text/html` on `/audit/view`; non-admin 403).
- **Expected output:** audit page test cases pass.
- **Brief-Scope outputs to verify:**
  - `admin_dashboard_html.rs` contains `pub async fn admin_audit_html` and the literal `EventSource(` + `addEventListener("admin_config_changed"` + `addEventListener("admin_config_change_denied"`.
  - `crates/api/routes/src/lib.rs` contains `.route("/audit/view"`.

### Story 3: The pages can be disabled instance-wide

- **Composing tasks:** Task 2 + Task 3 (the `html_pages_enabled` 404 gate in both handlers), Task 5 (the flag-off assertion).
- **Checkpoint command:** same e2e binary — the `html_pages_enabled=false → 404` assertion.
- **Expected output:** flag-off returns 404 for both `/dashboard/view` and `/audit/view`; flag-default (no row) returns 200 for admin.
- **Brief-Scope outputs to verify:** `admin_dashboard_html.rs` reads `governance.dashboard.html_pages_enabled` and returns `NotFound` when false (grep the handler body for the key + a `NotFound` path).

> Verification mapping: `/brehon-verify` iterates these three stories, runs each checkpoint against the worktree branch, confirms each Brief-Scope output exists + matches its structural pattern. Phantom (task complete but output absent/empty) → catch-fire.

## 17. Completion checklist

- [ ] Task 0 audit complete (11 probes confirmed; DQ #1 + #2 resolved).
- [ ] Tasks 1–5 committed (one commit each; lockfile co-committed with Task 1).
- [ ] §15 validation green at every gate (validate-pending-laptop mutations all `result: pass`).
- [ ] §16a stories all `[done]`.
- [ ] Retro committed (`.claude/PRPs/reports/v1-AD-e-retro.md`).
- [ ] PR opened by BM session against `governance-v0` with `--repo barrie-cork/lemmy`.
- [ ] CodeRabbit review complete; findings triaged per `feedback_pr_review_triage_pattern.md` (four-bucket).
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-AD-e-verify.md` shows all 3 stories ✓.
- [ ] Post-merge phase branch retained for retro reads.

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| First HTML in workspace surfaces an unforeseen actix `Responder`/content-type interaction | MED | MED | maud recommendation (native `Responder`, smallest new surface); §10.1 mirrors the *proven* non-JSON `HttpResponse::Ok().content_type(...).body(...)` idiom from `admin_audit_stream.rs:124-126` — not a novel response path, just a new content-type. Task 1 isolates the dep add; Task 2's check/clippy catches binding issues before the handler is wired to a route. |
| New engine dep triggers a transitive lint cascade on the dependency-tree rebuild | MED | MED | Per `feedback_library_add_after_shipping`: Task 1 captures a clippy-narrowed log *immediately* after the dep add (before any handler code). Cascade visible there → STOP + surface (no `#[allow]`-spam). Single-consumer dep (only `lemmy_api`) bounds blast radius. |
| `gather_dashboard` extraction subtly changes `admin_dashboard()` JSON output | LOW | HIGH | Behaviour-preserving extraction (copy real types, no logic change); R7 `cargo test --no-run` after Task 2 + the **full v1-AD-d dashboard e2e test re-run** in §15.4 is the regression guard. If the existing dashboard test fails post-extraction, the extraction is wrong — STOP. |
| askama chosen but `askama_actix` (stale) used instead of the current shim | LOW | MED | §5.2 + Task 1 GOTCHA explicitly name `askama_web` (actix-web-4 feature) as the current integration; `cargo add` resolves the compatible version against actix-web 4.13; commit body records the exact pin. |
| SSE client uses `onmessage` and receives nothing (server sends named events) | MED | LOW | Task 4 GOTCHA + the literal script in §13 use `addEventListener("admin_config_changed"/"admin_config_change_denied")`; Story 2 `/brehon-verify` greps for both `addEventListener` literals (phantom-catch if the impl used `onmessage`). |
| Route-naming deviation (`/dashboard/view` vs PRD literal `/admin/governance/dashboard`) is wrong for the operator | LOW | LOW | §8.3 records the rationale; DQ #1 options include "literal `/admin/governance/*` as a separate top-level scope" so the user picks the URL shape at the scope-cut gate, before Task 1. |
| Unescaped audit-entry content (XSS via a crafted config `reason`/`key`) | LOW | HIGH | §15.5 box: maud auto-escapes; askama `.html` autoescape on; only the controlled Task-4 `<script>` literal is raw (`PreEscaped`), never user data. CR review will flag any `|safe`/raw on a data field. |
| Pre-existing e2e flake masks a real regression in the full-suite run | LOW | MED | §15.4 runs the **whole** e2e suite; a flake vs a real break is disambiguated by re-run (per standard e2e flake handling); the v1-AD-d dashboard test specifically is deterministic (seeded counts). |

## 19. Notes

**DQ pre-seeds (advisor files these to the user before plan approval; both must be `resolved[]` before Task 0 Probe 5/6 pass):**

- **DQ #1 — scope cut (page subset).** Question: "v1-AD-e ships Dashboard + Audit HTML pages only (the 2 with shipped read paths); Config-editor / Single-key-editor / Rule-set-manager defer to v1-AD-f (each needs net-new read-path Rust). Confirm the cut line." Options: (a) **Dashboard + Audit only — recommended** (true pure-presentation, smallest CR surface, the 3 deferred pages are read-path+template work); (b) all 5 pages now (larger, contradicts the pure-templating framing, adds read-path Rust this plan deliberately excludes); (c) Dashboard + Audit now AND adopt the literal PRD `/admin/governance/*` URL spelling as a separate top-level scope (URL-shape variant of (a)). `from: advisor`, `kind: blocker`, planner-recommended (a).
- **DQ #2 — HTML engine.** Question: "v1-AD-e is the first server-rendered HTML in the workspace. Pick the template engine (OQ-V1-AD-01 re-resolution 2026-05-16 delegated this here)." Options: (a) **maud — recommended** (1 dep, 0 new files, native actix `Responder`, matches the inline-Rust codebase idiom); (b) askama (2 deps incl. the `askama_web` actix-4 shim, fresh `templates/` dir, compile-time external-file template checking). Evidence table in §5.2. `from: advisor`, `kind: blocker`, planner-recommended (a). **Both options keep the React/SPA pass as v2 per ADR-010 — unchanged.**

**Governance basis:** OQ-V1-AD-01 was re-resolved 2026-05-16 (sequencing-only supersede; engine choice open; premise of the 2026-04-20 deferral *not* overturned). The re-resolution + changelog entry were authored by the advisor and **applied by the user to the sibling repo's `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`** before this plan was written. This plan does not edit that file (it lives outside the fork and is authoritative); it only cites the 2026-05-16 re-resolution as the live governance reference. No ADR is contradicted: ADR-010 (React/SPA = v2) unchanged, ADR-011 (AGPLv3) — HTML templates are AGPL source, ADR-004 (plane separation) — pages under the existing governance route tree, ADR-015 (pseudonyms) — rendered field is already-redacted by the shipped projection, ADR-013 (EmergencyRemove) — no new `CaseStatus` match.

**Why no `[P]` markers / serial:** Tasks 2→3→4→5 are a hard dependency chain (3's handler lives in 2's module; 4 edits 3's fn; 5 e2e-tests 2+3's routes). Cohort parallelism would yield nothing and risks worktree races for zero benefit. Task 1 (dep) strictly precedes Task 2 (`requires: task 1`). This is correctly a 6-step serial pipeline.

**Engine-parameterisation:** the plan body names the render call abstractly (`render_dashboard(&resp)`, `render_audit(&recent)`) so DQ #2's outcome slots in at Task 1 without re-planning. Only Task 1's dep line + Task 2/3's render-fn *internals* + the conditional `templates/` files differ by engine; the handler shape, routes, gates, tests, and DoD are engine-invariant.

**Alternatives considered + rejected:** (1) HTML handlers in `lemmy_routes` — rejected (R-html-1: `pub(crate)` data-gather fns unreachable; would force `pub` widening or logic duplication). (2) Add `actix-files` + serve a static SPA build — rejected (ADR-010 React=v2; R-html-2; introduces npm/bundler). (3) Duplicate the six gather queries in the HTML handler — rejected (behaviour-drift risk; the Task 2 extraction is the correct DRY fix). (4) Ship all 5 PRD §6 pages — rejected for v1-AD-e (3 need read-path Rust → not pure-presentation → defer to v1-AD-f via DQ #1).

## 20. Confidence score

- **Plan correctness:** 8/10 — substrate 100% shipped + verified at file:line; the only structural change (gather extraction) is behaviour-preserving with an explicit regression guard (v1-AD-d e2e re-run). −2: first HTML in the workspace is genuinely novel (no precedent to mirror beyond the SSE non-JSON-response idiom); the engine DQ resolving to askama would add fresh template-dir infra not yet exercised.
- **Cargo budget:** 9/10 — pure-additive, single dep, no migration; ≈6 GB peak well within the laptop's 64 GB; validate-pending-laptop serialises cargo (no cohort stacking).
- **Test coverage:** 8/10 — three behaviours (200/403/404) e2e-asserted + the v1-AD-d regression guard; −2: e2e cannot exercise the *browser* SSE live-tail (the `<script>` is asserted by structural grep in Story 2, not a headless-browser run — documented limitation, acceptable per IMPLEMENTATION-PLAN-v0 §5 "no frontend testing").
