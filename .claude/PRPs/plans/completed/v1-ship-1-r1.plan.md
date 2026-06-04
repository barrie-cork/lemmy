# Plan: v1-ship-1-r1 — AGPL §13 source-disclosure surface (MIRROR refs refreshed)

> **RE-PLAN.** Design carried forward verbatim from the parked `v1-ship-1.plan.md` (2026-05-14). Every `file:line` MIRROR ref, §11 caller enumeration, §10 verbatim block, and §15 DoD reference is re-derived against `governance-v0` HEAD = `9504c806d` (worker-branch view; one commit behind `origin/governance-v0`, which has only the brief's own clarify commit `cb3134b24` on top). The parked plan stays as audit trail; this file is the dispatchable plan.
>
> **What changed since the parked plan:** the 5-PR refactor tier merged (#128–#132), most notably PR-1 #132's LemmyResult-unification rewrite of `crates/server/tests/e2e.rs`. Every cited e2e anchor drifted; the canonical e2e error-shape is now uniformly Case A (per `feedback_lemmy_error_no_std_error.md`). Two pre-resolved clarifies (DQ #226 + DQ #227) and one verified dependency-graph fact (`lemmy_api_crud` does NOT depend on `lemmy_api_routes`) shape the structural details below. The user-visible deliverable — `source_disclosure` block on `GetSiteResponse` + `GET /api/v4/source` returning the AGPL notice + one named e2e — is unchanged.

## 1. Summary

Extends `GetSiteResponse` with a `source_disclosure` block (license SPDX, repo URL, build-time-injected fork commit SHA, relative `/api/v4/source` disclosure URL) and adds a new public endpoint `GET /api/v4/source` returning the verbatim `AGPL-NOTICE.md` body as JSON `{ notice, license }`. First external user hitting `/api/v4/site` receives the disclosure pointer on the standard handshake; following the URL resolves to the notice text. One e2e test asserts both surfaces in-process via `actix_web::test::init_service`. Closes the first of two structural ship-gates in `v0-endpoint-coverage-2026-05-14.md` (AGPL §13 user-visible compliance). No ADR contradiction; ADR-011 is the load-bearing constraint and this is its user-visible surfacing.

## 2. Source

- `.claude/PRPs/prds/v1-ship-readiness.prd.md` §7.1 @ `governance-v0` (PRD authored 2026-05-14; v1-ship-1 is its first sub-phase).
- `.claude/PRPs/plans/v1-ship-1.plan.md` @ `aea538166` (parked plan — design is locked here; MIRROR refs were stale, hence this re-plan).
- `.claude/PRPs/briefs/v1-ship-1-r1-planning-1.md` @ `ba94510ee` + clarify commit `cb3134b24` (DQ #226 + DQ #227 resolutions are binding inputs).
- `.claude/PRPs/reports/v0-endpoint-coverage-2026-05-14.md` §8 Tier-1 gap #1 (substrate evidence — AGPL §13 not user-visible).
- **ADR-011** — `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` §ADR-011 (AGPLv3 inherited + source-disclosure required; load-bearing constraint).
- `AGPL-NOTICE.md` (repo root) — verbatim notice body, 39 lines / ~3,595 bytes; `include_str!` target.
- **Canonical sibling plan (schema-first per `feedback_read_canonical_before_writing_spec.md`):** `.claude/PRPs/plans/v1-ship-1.plan.md` (same 20-section schema; this re-plan mirrors its shape).
- Lessons binding decisions:
  - `feedback_lemmy_error_no_std_error.md` — Case A discipline for the new e2e test (post-LemmyResult-unification canonical).
  - `feedback_planner_enumerate_struct_callsites_for_addfield.md` — §11 R9 enumeration (Source struct-field-add → grep every constructor + destructure).
  - `feedback_features_full_p_crate_incompatible.md` + `feedback_features_full_workspace_only.md` — §15 DoD shape (`--workspace --features full` only).
  - `feedback_clippy_doc_lazy_continuation_in_doc_comments.md` — doc-comment shape on new DTO fields.
  - `feedback_complexity_score_pre_split.md` — §5.1 breakdown table.
  - `feedback_plan_dod_dry_run_at_write.md` + `feedback_pre_phase_dod_smoke_test.md` — §15 commands dry-run at advisor approval gate.
  - `feedback_plan_baseline_self_reference.md` — MIRROR refs cite symbol + line so the plan self-heals against minor pre-dispatch drift.
  - `feedback_test_target_compile_validation.md` — R7 (test-target compile after every struct/re-export touch).
  - `feedback_read_canonical_before_writing_spec.md` — schema-first gate (canonical sibling cited above).
  - `feedback_principles_not_rules.md` — scope discipline (do not "improve" the design; only refresh evidence).
- Rules cited by name (no standalone lesson file in worktree; encoded in advisor rules):
  - `.claude/rules/advisor-orchestrator.md` §3.5 (watchpoint specificity gate — every §4 watchpoint cites file:line).
  - `.claude/rules/advisor-orchestrator.md` §4.1 + §4.3 (cohort dispatch + `requires:` discipline).
  - `.claude/rules/advisor-orchestrator.md` §5.2 (validate-pending laptop / Shape G two-phase validation).
  - `.claude/rules/decision-queue.md` (DQ schema, attribution, `kind: "validate-pending"` mutation).

## 3. Problem statement

ADR-011 requires every release to honour AGPL §13 (Remote Network Interaction): any user interacting with the running fork across a network must be offered the Corresponding Source. The fork ships `LICENSE` (AGPL-3.0 text) and `AGPL-NOTICE.md` (source-disclosure narrative) at the repo root, but **no HTTP endpoint returns either to a connecting client**. From the moment the fork accepts a first external HTTP request, the §13 clause is engaged and the user-facing disclosure surface is absent. This is the load-bearing structural ship-gate for "compliantly accepts first external user."

Tied to §13 Task 1 (DTOs), Task 2 (field add + handler population + build.rs env injection), Task 3 (`get_source` handler + route + module wiring), and Task 4 (e2e).

## 4. Solution statement

Two surfaces, one for discovery and one for the body, both public, no auth, no per-route rate-limit override (inherit the `/api/v4` scope's `rate_limit.message()`):

1. **First-touch handshake (discovery):** the existing `GetSiteResponse` payload — which every Lemmy/Brehon client deserializes on connection at `GET /api/v4/site` — gains a `source_disclosure: SourceDisclosure` field. The `SourceDisclosure` block carries SPDX license (`"AGPL-3.0"`), the canonical fork repo URL (`"https://github.com/barrie-cork/lemmy"`), the build-time-injected fork commit SHA, and a relative URL (`"/api/v4/source"`) the client follows for the full notice.
2. **Disclosure body (resolution):** a new endpoint `GET /api/v4/source` returns `GetSourceResponse { notice: String, license: String }` where `notice` is the verbatim `AGPL-NOTICE.md` body sourced via `include_str!` at compile time. Zero DB read, zero auth check, no input shape.

The fork-commit SHA is read at compile time via `env!("BREHON_FORK_COMMIT")`. The env var is populated by a `build.rs` in **`crates/api/api_crud/`** (the leaf crate that consumes the const inside `read_site`). The build script shells `git rev-parse HEAD` (or honours `BREHON_FORK_COMMIT` when set by CI/Docker), falling back to the literal string `"unknown"` non-fatally when `.git/` and the env var are both absent (e.g. source-tarball builds).

```
client → GET /api/v4/site
       ← { ..., source_disclosure: { license, repo_url, fork_commit, disclosure_url: "/api/v4/source" }}

client → GET /api/v4/source
       ← { notice: <AGPL-NOTICE.md verbatim>, license: "AGPL-3.0" }
```

The `read_site` cache (`static CACHE: CacheLock<GetSiteResponse> = LazyLock::new(build_cache);` at `crates/api/api_crud/src/site/read.rs:25`) holds the entire `GetSiteResponse`. Every field of `SourceDisclosure` is a compile-time constant, so the cache value is correct for the instance lifetime — adding the field does not break the cache invariant. No new crate, no new migration, no new ADR.

> **Build.rs location — evidence-driven re-derivation.** The parked plan's primary path (`build.rs` in `crates/api/routes/`, re-export `pub const BREHON_FORK_COMMIT` via `lemmy_api_routes`, consumed by `lemmy_api_crud::site::read`) cannot work: verification of `crates/api/api_crud/Cargo.toml` `[dependencies]` (read at re-plan time) shows `lemmy_api_crud` does NOT depend on `lemmy_api_routes` (the reverse holds — `lemmy_api_routes` depends on `lemmy_api_crud`). Adding the dep would be circular. The parked plan documented this exact fallback in its Task 3 GOTCHA 1; we promote it from "fallback if dep graph blocks" to the only viable path. **This is not a design change** (env! + build.rs + const, all preserved); only the build.rs location and the const's scope are refreshed against verified evidence. See §10.4 for the verbatim build.rs body.

## 5. Metadata

- **Phase:** `v1-ship-1` (the `-r1` suffix marks the re-plan; phase branch stays `phase-v1-ship-1`).
- **Branch:** `phase-v1-ship-1` (cut by BM-task before Task 1).
- **Target impl-task model:** `sonnet-4-6`.
- **Estimated tasks:** 6 (Task 0 pre-flight + Tasks 1-4 impl + Task 5 retro).
- **Estimated cargo budget:** N/A under Shape G (validation off-box on GitHub Actions per Shape-G discipline, `.github/workflows/cargo-validate-workspace.yml` + `.github/workflows/cargo-test-e2e.yml`).
- **Forbidden-window applicability:** non-binding under Shape G for impl-task dispatch; binding for any ad-hoc local cargo runs (DoD smoke test, hand-debug).
- **Complexity score:** **8/10** — at-but-not-above the Sonnet split threshold (`>8`). No split-DQ.

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Target model is Sonnet 4.6 → split-DQ threshold is `> 8`.

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 4 impl tasks (Tasks 1-4); below threshold |
| Migrations touched | +2 each | 0 | No schema changes |
| Crates touched | +1 each | 5 | `lemmy_db_views_site` (Task 1 + Task 2), `lemmy_api_crud` (Task 2 + new build.rs), `lemmy_api` (Task 3 — new `site/source.rs`), `lemmy_api_routes` (Task 3 — route registration), `lemmy_server` test target (Task 4 — `crates/server/tests/e2e.rs`) |
| `crates/server/tests/e2e.rs` edits | +3 each | 3 | One §13 task with `crates/server/tests/e2e.rs` in `modifies:` (Task 4) |
| New ADR-affecting decisions | +2 each | 0 | ADR-011 surfaced, not amended |
| Cargo budget peak above 6 GB | +1 per GB | 0 | Shape G — validation off-box |
| **Total** | — | **8** | At Sonnet threshold (`>8` → split); **not above**, so no split-DQ |

The Sonnet target's "proceed-as-one with prior-Sonnet-phase precedent" override would apply at `score = 9+`; we are at `8`, so the standard rule (no split needed) governs. Re-derived against the parked plan's `3/10`: the parked plan undercounted crates (omitted `lemmy_api` and `lemmy_server` test target) and the e2e factor (the parked plan said "not weight-scoring as a 'edit' factor since it's append-only" — but the canonical rule is about edit count, not edit shape; a §13 task that modifies `e2e.rs` counts regardless of whether the edit is a clean append). The re-derived 8 is the honest count.

### 5.2 Per-task complexity ceiling (Sonnet target)

Sonnet ceiling: `≤ 4` files per task / `≤ 2` crates per task / e2e edits in their own task. Walk:

- Task 0 (pre-flight): 0 files / 0 crates. ✓
- Task 1 (DTOs): 1 file / 1 crate. ✓
- Task 2 (field + handler + build.rs): 3 files / 2 crates. ✓
- Task 3 (handler + route + mod wiring): 3 files / 2 crates. ✓
- Task 4 (e2e — dedicated): 1 file / 1 crate (lemmy_server test target). ✓
- Task 5 (retro): 1 file / 0 production crates. ✓

No split-candidate detected.

## 6. Relationship to other v1-ship sub-phases

- **Independent of v1-ship-2** (e2e backfill on 4 untested endpoints, per PRD §7.2). Both can run in parallel sub-phase lanes; only file overlap is `crates/server/tests/e2e.rs` (additive appends each lane).
- **Precedes v1-ship-3** in retro-discipline rhythm only (not in code). v1-ship-3 touches `docker-compose.yml`, `create_report` handler, and a different e2e test; no shared files with this plan.
- **No dependency on RT-r2..RT-r6 / JM-f / restorative-mechanics-v1.** Those are parallel feature lanes per PRD §1.3.

## 7. Preflight guardrails inherited from prior phases

- **R1** (per `feedback_clippy_test_style.md` — referenced by name in the corpus): every `i32 ↔ i64` comparison uses `i64::from(...)`, never `as` cast. *Light here* — this plan introduces no integer comparisons.
- **R5** (per JM-b retro Event 4 + `.claude/rules/pre-phase-harness-audit.md`): Task 0 enumerates ALL probes explicitly; no implicit inheritance.
- **R6** (per JM-b retro Event 3): all clippy invocations use `--no-deps` uniformly. The workspace-check workflow line 92 honours this.
- **R7** (per `feedback_test_target_compile_validation.md`): tasks that touch a struct or re-export run `cargo test --no-run -p lemmy_server --test e2e`. The workspace-validate workflow runs this step on every push.
- **R8** (per `feedback_lemmy_error_no_std_error.md` Case A): test fn outer return uses `lemmy_utils::error::LemmyResult<()>`; helpers + Lemmy-native calls + serde_json calls all bridge via bare `?` (LemmyError already has `From<serde_json::Error>` etc). NO `.map_err(|e| anyhow::anyhow!(...))?` bridge required for `governance_fixtures::bootstrap()` (returns `LemmyResult<...>`).
- **R9** (per `feedback_planner_enumerate_struct_callsites_for_addfield.md`): `GetSiteResponse` field-add task enumerates every site (constructor + destructure) in §11; back-compat path is verified at plan-write time. Result of `rg "GetSiteResponse" crates/ --include="*.rs"` is captured in §11 below.
- **R10** (per `feedback_pre_phase_dod_smoke_test.md` + `feedback_plan_dod_dry_run_at_write.md`): every §15 command shape verified against current `.github/workflows/*.yml` at plan-write time.

## 8. Flow design

### Before

```
+---------+                 +-------------------+
| client  |--GET /site----->| get_site handler  |
+---------+                 |  read_site()      |
                            | + LazyLock CACHE  |
                            +-------------------+
                                    |
                                    v
                            +-------------------+
                            | GetSiteResponse   |  ← no source_disclosure
                            | { site_view, ... }|
                            +-------------------+
```

### After

```
+---------+                 +-------------------+
| client  |--GET /site----->| get_site handler  |
+---------+                 |  read_site()      |
     |                      | + LazyLock CACHE  |
     |                      +-------------------+
     |                              |
     |                              v
     |                      +-------------------+
     |                      | GetSiteResponse   |
     |                      | { site_view, ..., |
     |                      |   source_disclosure: SourceDisclosure {
     |                      |     license, repo_url, fork_commit, disclosure_url
     |                      |   }}
     |                      +-------------------+
     |                            ^
     |                            |  env!("BREHON_FORK_COMMIT")  ← build.rs in api_crud/
     |
     |--GET /source-------->+-------------------+
                            | get_source handler|
                            | (no DB, no auth)  |
                            +-------------------+
                                    |
                                    v
                            +-------------------+
                            | GetSourceResponse |
                            | { notice, license }
                            | notice = include_str!(../../../../../AGPL-NOTICE.md)
                            +-------------------+
```

Boxes ↔ §13 tasks:

- `SourceDisclosure` + `GetSource` + `GetSourceResponse` DTOs in `crates/db_views/site/src/api.rs` → **Task 1**
- `source_disclosure` field on `GetSiteResponse` + populated in `read_site` + `crates/api/api_crud/build.rs` env injection → **Task 2** (atomic — field add without handler population would break compilation; build.rs lives with the consumer per §4 evidence)
- `get_source` handler module (`crates/api/api/src/site/source.rs`) + module re-export (`crates/api/api/src/site/mod.rs`) + route registration (`crates/api/routes/src/lib.rs`) → **Task 3** (atomic — handler + route + import in one task because route depends on handler symbol and import depends on the new mod)
- e2e test → **Task 4**

## 9. Mandatory reading

The `impl-task` subagent reads these files before its first edit on each task. Every line range was confirmed by `Read` at plan-write time (HEAD `9504c806d`).

### Schema/type definitions

- `crates/db_views/site/src/api.rs:1-47` — file-level imports (`serde::{Deserialize, Serialize}`, `serde_with::skip_serializing_none`, ts-rs feature attributes via `cfg_attr`).
- `crates/db_views/site/src/api.rs:332-355` — `GetSiteResponse` full definition (struct keyword at line 337; lead `#[skip_serializing_none]` at line 332; verbatim plan §10.1). **Current shape:** `#[skip_serializing_none]` + `#[derive(Debug, Serialize, Deserialize, Clone)]` + `#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]` + `#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]`. **Last field is `captcha_enabled: bool` at line 354.**
- `crates/api/api/src/site/federated_instances.rs:1-15` — minimal sibling handler shape (verbatim plan §10.3). 15 lines total at HEAD.
- `crates/api/api_crud/src/site/read.rs:1-71` — full file (only 71 lines). Includes `get_site` at line 20, `static CACHE: CacheLock<GetSiteResponse> = LazyLock::new(build_cache);` at line 25, `read_site` at line 41, the single `Ok(GetSiteResponse { ... })` constructor at line 57-70 (closing `})` at line 70-71).
- `crates/api/api_utils/src/context.rs:12-59` — `LemmyContext` definition; `settings()` accessor (carry-forward range from parked plan; **UNVERIFIED at re-plan time** — re-confirm before first edit if the impl-task needs to call `settings()`).

### Existing patterns (MIRROR refs in §13)

- `crates/api/routes/src/lib.rs:216-230` — `pub fn config(cfg: &mut ServiceConfig, rate_limit: &RateLimit)` open + `scope("/api/v4")` at line 218 + `.wrap(rate_limit.message())` at 219 + `.service(scope("/site")...)` at 221-230 (the site scope's closing `)` is at line 230).
- `crates/api/routes/src/lib.rs:288` — `.route("/federated_instances", get().to(get_federated_instances))` — the canonical "sibling route at `/api/v4` scope level, NOT inside `/site`" precedent. Task 3 mirrors this shape for `/source`.
- `crates/api/routes/src/lib.rs:107-124` — the existing `use lemmy_api::{ ... site::{ ... federated_instances::get_federated_instances, ...}}` import block; Task 3 adds `source::get_source` to this import group.
- `crates/api/routes/src/lib.rs:1` — `use actix_web::{guard, web::*};` (everything in `web::*` is in scope — `get`, `post`, `scope`, `delete`, `put`, `resource`, `ServiceConfig` etc).
- `crates/db_views/site/src/api.rs:337-355` — DTO derive shape (`#[skip_serializing_none]` + `#[derive(Debug, Serialize, Deserialize, Clone)]` + ts-rs cfg-attrs). Task 1's new DTOs mirror this shape (minus `#[skip_serializing_none]` for `SourceDisclosure`/`GetSourceResponse` since neither has `Option` fields).
- `crates/server/tests/e2e.rs:118` — `mod governance_fixtures {` declaration (parked plan said line 88; drift confirmed).
- `crates/server/tests/e2e.rs:801` — `pub async fn bootstrap() -> LemmyResult<(testcontainers::ContainerAsync<testcontainers::GenericImage>, Data<LemmyContext>, String)>` (the canonical bootstrap; DQ #226 RESOLVED: this is the chosen bootstrap for the new e2e test).
- `crates/server/tests/e2e.rs:5556` — `mod admin_config_fixtures {` (the WRONG sibling per DQ #226). Its `bootstrap()` lives at line 5578 and depends on `super::governance_fixtures::start_postgres` + `super::governance_fixtures::apply_all_schema` (line 5590-5598) — NOT independently functional. Task 4 must call **`governance_fixtures::bootstrap()`** at line 801, NOT `admin_config_fixtures::bootstrap()` at line 5578.
- `crates/server/tests/e2e.rs:2164` — `async fn report_to_modlog_golden_path() -> lemmy_utils::error::LemmyResult<()>` (Case A outer return; parked plan cited line 2077 — drift confirmed).
- `crates/server/tests/e2e.rs:3693` — `async fn all_mvp_endpoints_return_non_404() -> lemmy_utils::error::LemmyResult<()>` (Case A outer return; in-process HTTP via `actix_web::test::init_service` / `test::TestRequest::get().uri(path).to_request()` / `test::call_service(&app, req).await`; parked plan cited 3700-3880 — close, but refresh to `3693`).
- `crates/server/tests/e2e.rs:3770-3775` — verbatim `App::new().app_data(Data::new(context.clone())).wrap(SessionMiddleware::new(context.clone())).configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit))` (the canonical in-process app builder for an HTTP-level test; the new test mirrors this minus SessionMiddleware since the AGPL endpoints don't need a session).
- `crates/server/tests/e2e.rs:11001-11924` (carry-forward citation per `feedback_lemmy_error_no_std_error.md` line 24) — the v1-SL-b `mod v1_sl_b_fixtures` canonical reference for Case A throughout (every helper `LemmyResult<T>`, every test fn `LemmyResult<()>`, every `?` bare). **Note:** this lesson cites this exact range; verifying the range still bounds the v1-SL-b mod is left to the impl-task (cheap `grep -n "^mod v1_sl_b_fixtures"` at task start).

### Adjacent test fixtures

- `crates/server/tests/e2e.rs:801-855` — `governance_fixtures::bootstrap()` body + return tuple shape `(testcontainers::ContainerAsync<testcontainers::GenericImage>, Data<LemmyContext>, String)`. The destructure pattern `let (_container, context, _db_url) = governance_fixtures::bootstrap().await?;` is used by every governance test (lines 8071, 8132, 8199, ..., dozens). The new test mirrors this exactly.
- `crates/server/tests/e2e.rs:826` — `let rate_limit = RateLimit::with_debug_config();` (used 10+ times in sibling tests; the new test reuses verbatim).

### Lessons binding tasks

- `feedback_lemmy_error_no_std_error.md` — Task 4 (e2e error-shape Case A; canonical-sibling per §3 line 24 of the lesson).
- `feedback_features_full_p_crate_incompatible.md` — §15 commands use `--workspace --features full`, never `-p <crate> --features full`.
- `feedback_features_full_workspace_only.md` — same rule, doubled.
- `feedback_clippy_doc_lazy_continuation_in_doc_comments.md` — doc-comments on `SourceDisclosure` / `GetSource` / `GetSourceResponse` fields must not lazy-continue (single-line `///` or blank `///` between paragraphs).
- `feedback_planner_enumerate_struct_callsites_for_addfield.md` — Task 2 (R9).
- `feedback_test_target_compile_validation.md` — Tasks 1, 2, 3, 4 all touch a struct or re-export → workflow runs `cargo test --no-run -p lemmy_server --test e2e` on every push (R7).
- `feedback_principles_not_rules.md` — design is locked; only evidence is refreshed (per brief §0).

## 10. Patterns to mirror

Per `.claude/rules/advisor-orchestrator.md` §3.5 watchpoint specificity gate: every pattern cites a specific file:line; line numbers were confirmed by `Read` at plan-write time. Where the plan needs a stable reference, prefer a grep-able symbol + the line so the plan self-heals (per `feedback_plan_baseline_self_reference.md`).

### 10.1 GetSiteResponse derive + field shape

**Mirror:** `crates/db_views/site/src/api.rs:332-355` (struct keyword at line 337; closing `}` at line 355). Symbol to grep if drifted: `pub struct GetSiteResponse {`.

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// An expanded response for a site.
pub struct GetSiteResponse {
  pub site_view: SiteView,
  pub admins: Vec<PersonView>,
  pub version: String,
  pub all_languages: Vec<Language>,
  pub discussion_languages: Vec<LanguageId>,
  /// If the site has any taglines, a random one is included here for displaying
  pub tagline: Option<Tagline>,
  /// A list of external auth methods your site supports.
  pub oauth_providers: Vec<PublicOAuthProvider>,
  pub admin_oauth_providers: Vec<AdminOAuthProvider>,
  pub blocked_urls: Vec<LocalSiteUrlBlocklist>,
  pub active_plugins: Vec<PluginMetadata>,
  /// The number of seconds between the last application published, and approved / denied time.
  ///
  /// Useful for estimating when your application will be approved.
  pub last_application_duration_seconds: Option<i64>,
  pub captcha_enabled: bool,
}
```

**New `SourceDisclosure` struct** (Task 1, appended after `GetSiteResponse` at line 355). Match the parent's derive set minus `#[skip_serializing_none]` (no `Option` fields):

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// AGPL §13 source-disclosure block.
///
/// Returned as part of every `GetSiteResponse` so a first-touch client
/// receives the SPDX license, the canonical repository URL, the running
/// fork commit, and the relative URL of the disclosure body.
pub struct SourceDisclosure {
  /// SPDX identifier, always "AGPL-3.0".
  pub license: String,
  /// Canonical fork repository URL.
  pub repo_url: String,
  /// HEAD commit SHA of the running fork build; "unknown" if unavailable at build time.
  pub fork_commit: String,
  /// Relative URL the client follows for the full notice body.
  pub disclosure_url: String,
}
```

**New `GetSource` + `GetSourceResponse` structs** (Task 1, same file, appended after `SourceDisclosure`):

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Request for `GET /api/v4/source`. Unit struct — no input fields.
pub struct GetSource {}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response for `GET /api/v4/source`.
///
/// Returns the verbatim AGPL-NOTICE.md body bundled with the SPDX license identifier.
pub struct GetSourceResponse {
  pub notice: String,
  pub license: String,
}
```

**Field add to `GetSiteResponse`** (Task 2, appended after `captcha_enabled: bool` at line 354 — inside the struct body, before the closing `}` at line 355):

```rust
  // ... existing fields ...
  pub captcha_enabled: bool,
  /// AGPL §13 source-disclosure surface; see [`SourceDisclosure`].
  pub source_disclosure: SourceDisclosure,
```

### 10.2 read_site population

**Mirror:** `crates/api/api_crud/src/site/read.rs:1-71` (full file — only 71 lines). Symbols to grep if drifted: `async fn read_site`, `Ok(GetSiteResponse {`. The current constructor at line 57:

```rust
async fn read_site(context: &LemmyContext) -> LemmyResult<GetSiteResponse> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let admins = PersonView::list_admins(None, site_view.instance.id, &mut context.pool()).await?;
  let all_languages = Language::read_all(&mut context.pool()).await?;
  let discussion_languages = SiteLanguage::read_local_raw(&mut context.pool()).await?;
  let blocked_urls = LocalSiteUrlBlocklist::get_all(&mut context.pool()).await?;
  let tagline = Tagline::get_random(&mut context.pool()).await.ok();
  let admin_oauth_providers = AdminOAuthProvider::get_all(&mut context.pool()).await?;
  let oauth_providers =
    AdminOAuthProvider::convert_providers_to_public(admin_oauth_providers.clone());
  let last_application_duration_seconds =
    RegistrationApplication::last_updated(&mut context.pool())
      .await
      .ok()
      .and_then(|u| u.updated_published_duration());

  Ok(GetSiteResponse {
    site_view,
    admins,
    version: VERSION.to_string(),
    all_languages,
    discussion_languages,
    blocked_urls,
    tagline,
    oauth_providers,
    admin_oauth_providers,
    active_plugins: plugin_metadata(),
    last_application_duration_seconds,
    captcha_enabled: is_captcha_plugin_loaded(),
  })
}
```

Task 2 extends this with three additions:

**Addition 1** — at the top of the file, update line 16's import + declare the three constants:

Replace line 16's existing:

```rust
use lemmy_db_views_site::{SiteView, api::GetSiteResponse};
```

with:

```rust
use lemmy_db_views_site::{SiteView, api::{GetSiteResponse, SourceDisclosure}};
```

Then insert (between current line 18 and current line 20):

```rust
const BREHON_FORK_COMMIT: &str = env!("BREHON_FORK_COMMIT");
const BREHON_REPO_URL: &str = "https://github.com/barrie-cork/lemmy";
const SOURCE_DISCLOSURE_URL: &str = "/api/v4/source";
```

**Addition 2** — inside the `Ok(GetSiteResponse { ... })` constructor (current closing brace at line 70), append after the `captcha_enabled: is_captcha_plugin_loaded(),` line:

```rust
    captcha_enabled: is_captcha_plugin_loaded(),
    source_disclosure: SourceDisclosure {
      license: "AGPL-3.0".to_string(),
      repo_url: BREHON_REPO_URL.to_string(),
      fork_commit: BREHON_FORK_COMMIT.to_string(),
      disclosure_url: SOURCE_DISCLOSURE_URL.to_string(),
    },
  })
```

**Addition 3** — the `build.rs` (see §10.4) lives in `crates/api/api_crud/` so `env!("BREHON_FORK_COMMIT")` resolves at compile time for this crate.

> **Cache safety:** all four `SourceDisclosure` values are compile-time constants. The `LazyLock<CacheLock<GetSiteResponse>>` at `read.rs:25` holds the new shape correctly for the instance lifetime. Do NOT add any runtime read (e.g. re-reading the env var per-request) — that defeats the cache and the const value is the contract.

### 10.3 Minimal sibling handler

**Mirror:** `crates/api/api/src/site/federated_instances.rs:1-15` (full file — only 15 lines):

```rust
use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_site::{FederatedInstanceView, api::GetFederatedInstances};
use lemmy_diesel_utils::pagination::PagedResponse;
use lemmy_utils::error::LemmyResult;

pub async fn get_federated_instances(
  Query(data): Query<GetFederatedInstances>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<PagedResponse<FederatedInstanceView>>> {
  let federated_instances = FederatedInstanceView::list(&mut context.pool(), data).await?;

  // Return the jwt
  Ok(Json(federated_instances))
}
```

Task 3 creates `crates/api/api/src/site/source.rs` mirroring this shape — no `Query` (no input), no `context` (no DB), so the parameter list is empty:

```rust
use actix_web::web::Json;
use lemmy_db_views_site::api::GetSourceResponse;
use lemmy_utils::error::LemmyResult;

const AGPL_NOTICE: &str = include_str!("../../../../../AGPL-NOTICE.md");

/// `GET /api/v4/source` — returns the verbatim AGPL-NOTICE.md body.
///
/// Public endpoint; no auth required. Read-only; no DB access.
/// Surfaces the AGPL §13 source-disclosure requirement to any connecting client.
pub async fn get_source() -> LemmyResult<Json<GetSourceResponse>> {
  Ok(Json(GetSourceResponse {
    notice: AGPL_NOTICE.to_string(),
    license: "AGPL-3.0".to_string(),
  }))
}
```

**`include_str!` path arithmetic** — verified by counting at plan-write time:

`source.rs` is at `crates/api/api/src/site/source.rs`. The `include_str!` macro resolves paths relative to the file containing the call.

- `..` from `source.rs` → `crates/api/api/src/`
- `..` → `crates/api/api/`
- `..` → `crates/api/`
- `..` → `crates/`
- `..` → repo root
- Append `/AGPL-NOTICE.md`

Total: 5 `..` segments → `"../../../../../AGPL-NOTICE.md"`. If `AGPL-NOTICE.md` is missing, `include_str!` fails the build loudly with `error: couldn't read AGPL-NOTICE.md` — acceptable failure mode per the parked plan and PRD §7.1 risk table.

### 10.4 Build.rs in crates/api/api_crud/ (env injection)

No in-workspace precedent reads `BREHON_FORK_COMMIT`, but two existing `build.rs` files demonstrate the build-script idiom (`crates/diesel_utils/build.rs` + `crates/email/build.rs`). The new file:

```rust
// crates/api/api_crud/build.rs (NEW — Task 2)
fn main() {
  let commit = std::env::var("BREHON_FORK_COMMIT")
    .or_else(|_| {
      std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .ok_or(std::env::VarError::NotPresent)
    })
    .unwrap_or_else(|_| "unknown".to_string());
  println!("cargo:rustc-env=BREHON_FORK_COMMIT={commit}");
  println!("cargo:rerun-if-env-changed=BREHON_FORK_COMMIT");
  println!("cargo:rerun-if-changed=../../../.git/HEAD");
}
```

**Note on `.git/HEAD` path:** the `rerun-if-changed` directive resolves relative to the manifest dir (`crates/api/api_crud/`). To reach repo root's `.git/HEAD`: up 3 levels (`..` × 3 → from `api_crud/` → `api/` → `crates/` → repo root) + `.git/HEAD` → `"../../../.git/HEAD"`. The parked plan's same path was correct because its build.rs lived at the same depth in `crates/api/routes/`; carried forward.

**Crate dependency check (verified at plan-write):** `crates/api/api_crud/Cargo.toml` already has a `[build-dependencies]` block (lines 66-68) with unused serde + serde_json (harmless). The `[package]` section has no `build = "..."` line, so cargo auto-detects `build.rs` at the crate root — **no Cargo.toml edit needed**.

**Why this crate, not `crates/api/routes/`:** `lemmy_api_crud` does NOT depend on `lemmy_api_routes` (verified by reading `crates/api/api_crud/Cargo.toml` `[dependencies]` at plan-write — `lemmy_api_routes` is absent). Putting build.rs + const in `lemmy_api_routes` would force `lemmy_api_crud → lemmy_api_routes`, creating a circular dep (since `lemmy_api_routes → lemmy_api_crud` is the existing direction). The parked plan documented this fallback path explicitly (its Task 3 GOTCHA 1); evidence at re-plan time promotes it from fallback to primary.

### 10.5 Site route registration shape

**Mirror:** `crates/api/routes/src/lib.rs:107-124` (use-block) + `216-230` (config fn open + site scope) + `288` (sibling-route precedent). Symbols to grep if drifted: `pub fn config(cfg: &mut ServiceConfig`, `scope("/site")`, `"/federated_instances"`.

The current site scope at lines 222-230:

```rust
      .service(
        scope("/site")
          .route("", get().to(get_site))
          .route("", post().to(create_site))
          .route("", put().to(edit_site))
          .route("/icon", post().to(upload_site_icon))
          .route("/icon", delete().to(delete_site_icon))
          .route("/banner", post().to(upload_site_banner))
          .route("/banner", delete().to(delete_site_banner)),
      )
```

The federated_instances precedent at line 288:

```rust
      .route("/federated_instances", get().to(get_federated_instances))
```

Task 3 adds the new route as a sibling at the `/api/v4` scope level (NOT inside `/site`), mirroring the federated_instances precedent. Place it immediately after the `/site` scope closing `)` at line 230, before the `.route("/modlog", ...)` at line 231:

```rust
        scope("/site")
          // ... existing site routes ...
          .route("/banner", delete().to(delete_site_banner)),
      )
      // AGPL §13 source disclosure — public, no auth, no DB
      .route("/source", get().to(get_source))
      .route("/modlog", get().to(get_mod_log))
```

The route registers to **`/api/v4/source`** (because the parent `scope("/api/v4")` opens at line 218 and this `.route(...)` is at the scope's top level, not nested in `/site`). Verify by counting parentheses against §10.5 mirror lines — if the `/source` route were inside the `/site` scope (i.e. before the `),` at line 230), it would resolve to `/api/v4/site/source`, contradicting PRD §7.1 ship-criteria.

**Use-block addition** at lines 107-124 (the existing `lemmy_api::site::{ ... }` group). Add `source::get_source,` alphabetically after `registration_applications` (before the closing `}` of the `site::` group):

```rust
  site::{
    admin_allow_instance::admin_allow_instance,
    admin_block_instance::admin_block_instance,
    admin_list_users::admin_list_users,
    federated_instances::get_federated_instances,
    list_all_media::list_all_media,
    mod_log::get_mod_log,
    purge::{ ... },
    registration_applications::{ ... },
    source::get_source,   // ← NEW
  },
```

### 10.6 e2e test outer shape (Case A discipline, post-LemmyResult-unification)

**Mirror canonical Case A:** `crates/server/tests/e2e.rs:2164` (`report_to_modlog_golden_path`) — outer return `lemmy_utils::error::LemmyResult<()>`, bare `?` on every Lemmy-native call:

```rust
#[tokio::test(flavor = "multi_thread")]
async fn report_to_modlog_golden_path() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::{Data, Json, Query};
  // ... imports (inlined inside the test fn per the e2e.rs convention) ...
  // ... body uses bare `?` throughout ...
}
```

The new test (Task 4) mirrors this signature verbatim. Per `feedback_lemmy_error_no_std_error.md` lines 12-24, Case A is the canonical post-unification shape and `crates/server/tests/e2e.rs:11001-11924` (the v1-SL-b fixtures mod) is the canonical reference: every helper `LemmyResult<T>`, every test fn `LemmyResult<()>`, every `?` bare. Mirror Case A verbatim. **Do not** use `Result<(), Box<dyn Error>>` (Case B) nor mix shapes (Case C — `feedback_lemmy_error_no_std_error.md` §"Case C" is a hard refusal).

**Bootstrap call:** `governance_fixtures::bootstrap()` at `crates/server/tests/e2e.rs:801` returns `LemmyResult<(testcontainers::ContainerAsync<...>, Data<LemmyContext>, String)>`. The destructure pattern in sibling tests (e.g. line 8071) is `let (_container, context, _db_url) = governance_fixtures::bootstrap().await?;` — bare `?`. **DQ #226 RESOLVED**: this is the chosen bootstrap; the sibling `admin_config_fixtures::bootstrap()` at line 5578 is the WRONG choice (bumps rate-limit buckets, depends on `super::governance_fixtures`, intended for admin tests).

### 10.7 In-process HTTP via actix_web::test

**Mirror:** `crates/server/tests/e2e.rs:3693-3815` (`all_mvp_endpoints_return_non_404`), specifically the app-builder block at lines 3770-3775:

```rust
let app = test::init_service(
  App::new()
    .app_data(Data::new(context.clone()))
    .wrap(SessionMiddleware::new(context.clone()))
    .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit))
).await;
```

And the per-endpoint probe loop pattern at lines 3799-3815:

```rust
let req = test::TestRequest::get().uri(path).to_request();
let resp = test::call_service(&app, req).await;
let status = resp.status().as_u16();
// ... assertions ...
let body_bytes = test::read_body(resp).await;
let body: SomeResponse = serde_json::from_slice(&body_bytes)?;   // bare `?` into LemmyResult — From<serde_json::Error> for LemmyError holds (see e2e.rs:4413 sibling)
```

The new test (Task 4) mirrors this minus `SessionMiddleware` (the AGPL endpoints need no session) and uses two distinct `test::TestRequest::get().uri(...)` calls — one for `/api/v4/site`, one for `/api/v4/source`. Per `crates/server/tests/e2e.rs:826` the canonical rate-limit fixture is `let rate_limit = RateLimit::with_debug_config();` (reused verbatim — no per-test bucket overrides needed since the new test fires 2 GETs, well under any bucket).

## 11. Files to change

Grouped by crate. Each path was confirmed to exist at HEAD `9504c806d` (or marked **(new)**). Each `[P]` cohort decision derives from the FILES YAML below — `intersect(union(creates, modifies)_taskA, union(creates, modifies)_taskB) == ∅`.

### `lemmy_db_views_site` (crate: `crates/db_views/site/`)

- `crates/db_views/site/src/api.rs` — Task 1 appends `SourceDisclosure` + `GetSource` + `GetSourceResponse` structs after `GetSiteResponse` at line 355. Task 2 adds `source_disclosure: SourceDisclosure` field to `GetSiteResponse` (before its closing `}` at line 355).

### `lemmy_api_crud` (crate: `crates/api/api_crud/`)

- `crates/api/api_crud/src/site/read.rs` — Task 2 imports `SourceDisclosure`, declares three private consts, populates `source_disclosure` in the constructor at line 57-70.
- `crates/api/api_crud/build.rs` **(new)** — Task 2 creates the build script that injects `BREHON_FORK_COMMIT` (§10.4 body).

### `lemmy_api` (crate: `crates/api/api/`)

- `crates/api/api/src/site/source.rs` **(new)** — Task 3 creates the `get_source` handler (§10.3 body).
- `crates/api/api/src/site/mod.rs` — Task 3 appends `pub mod source;` to the module declarations (current content: 8 lines at HEAD; new line is line 9).

### `lemmy_api_routes` (crate: `crates/api/routes/`)

- `crates/api/routes/src/lib.rs` — Task 3 adds `source::get_source,` to the `lemmy_api::site::{...}` use-block at lines 107-124, and adds `.route("/source", get().to(get_source))` after the `/site` scope closes at line 230.

### `crates/server/tests/` (test target; `lemmy_server` crate)

- `crates/server/tests/e2e.rs` — Task 4 appends one new test `agpl_source_disclosure_surface_returns_notice` at the end of the file (current line count 14,775; the append lands at ~14,775+).

### §11 R9 — Caller enumeration for `GetSiteResponse` field add (mandatory)

Per `feedback_planner_enumerate_struct_callsites_for_addfield.md`. Result of `rg "GetSiteResponse" crates/ --include="*.rs"` (executed at plan-write; full output captured below; **noise-filtered to constructor + destructure sites only**):

| Site kind | Path:line | Compiles after Task 2? | Notes |
|---|---|---|---|
| Definition | `crates/db_views/site/src/api.rs:337` | n/a | Task 2 edits in place to add the field. |
| **Constructor** (only one) | `crates/api/api_crud/src/site/read.rs:57` | Yes — Task 2 adds the field. | The sole `Ok(GetSiteResponse { ... })` literal. |
| **Destructure (rest-pattern)** | `crates/api/routes_v3/src/handlers.rs:249-258` | **Yes — non-breaking** | Uses `let GetSiteResponse { site_view, admins, version, all_languages, discussion_languages, tagline, blocked_urls, .. } = ...;` with `..` at line 257. The rest-pattern means new fields don't break this site. **DQ #227 RESOLVED**: confirmed non-breaking; no code change needed. |
| Use/import only | `crates/api/api_common/src/site.rs:13`, `crates/api/api_crud/src/site/read.rs:16,23,25,41`, `crates/api/routes_v3/src/handlers.rs:96,138,248` | Yes — references the type only. | No literal construction or exhaustive destructure. |

**Total constructor sites:** **1** (the parked plan's claim of one constructor at `read.rs:64-71` holds in spirit; the actual current line is 57). **Total exhaustive destructures:** **0** (the `routes_v3/handlers.rs` destructure uses `..` rest-pattern — additive-safe). **Task 2's `modifies:` array (`crates/db_views/site/src/api.rs` + `crates/api/api_crud/src/site/read.rs`) covers every site that needs editing.** No additional crate enters the dep / FILES YAML graph.

Verification check the impl-task runs after Task 2's commit:

```bash
rg "GetSiteResponse \{" crates/ --include="*.rs"
# EXPECT: exactly 3 lines:
#   crates/db_views/site/src/api.rs:337:pub struct GetSiteResponse {     ← definition
#   crates/api/routes_v3/src/handlers.rs:249:  let GetSiteResponse {     ← rest-pattern destructure
#   crates/api/api_crud/src/site/read.rs:NN:  Ok(GetSiteResponse {       ← the one constructor (line drifts after edit)
```

## 12. NOT building in v1-ship-1

Out-of-scope items with rationale. Each pairs a "tempting addition" with a deferral target.

- **WebAuthn / passkey MFA** — deferred to `v2-security-hardening.prd.md`; ADR-010 v2 staging.
- **Source-tarball or signed-binary release artifact, SBOM, signed binaries** — deferred to `v2-release-pipeline.prd.md`; PRD §3 moves this out explicitly.
- **Public anchoring of the disclosure URL (Sigstore Rekor / BTC `OP_RETURN`)** — deferred to `v3-verifiability.prd.md`.
- **`AGPL-NOTICE.md` content update / translation / accessibility audit** — out of scope; this plan ships the *surface*, not the *content*. Content edits go through a separate `docs(brehon-fork)` commit, independent of this plan.
- **Per-endpoint OpenAPI registration** — the workspace does not use OpenAPI/utoipa (confirmed by parked plan's Explore agent; carried forward); zero codegen surface to update.
- **Rate-limit override on `/api/v4/source`** — inherits the global `rate_limit.message()` from the `/api/v4` scope (line 219). The notice payload is ~3.5 KB; the default limit is appropriate. No per-route override.
- **v1-ship-2** (e2e backfill on 4 untested endpoints) — separate sub-phase; PRD §7.2.
- **v1-ship-3** (Postgres pin + `POST /report` reshape + 2-sponsors e2e) — separate sub-phase; PRD §7.3.
- **Changing the `read_site` cache shape (e.g. per-request fork commit re-read)** — out of scope; the field's value is build-time-stable by design, the cache is correct.
- **Changing `GetSiteResponse`'s derive shape to add `PartialEq, Eq, Hash`** — out of scope; the parent struct does not have these derives, and `SourceDisclosure` does not need them for any planned use.

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task** (per `feedback_pr_per_phase.md`'s code-only-via-PR rule). Each task header carries a `[P]` marker iff its `union(creates, modifies)` shares no path with any other `[P]`-marked task in the same cohort. Cohort plan:

- Task 0: solo (pre-flight barrier — always non-`[P]`).
- Task 1: solo (DTOs barrier — every downstream task `requires:` Task 1's types).
- Cohort A: Tasks 2 + 3 (both `[P]`; file-disjoint between each other; both `requires:` Task 1).
- Task 4: solo (e2e — `requires:` Tasks 2 + 3; barrier for retro).
- Task 5: solo (retro — always non-`[P]`).

**Shape G (Layer G2 push-and-exit) plan.** Cargo invocations belong to `.github/workflows/cargo-validate-workspace.yml` (push-triggered on `junior/*`) and `.github/workflows/cargo-test-e2e.yml` (advisor-dispatched on phase branch). impl-task subagents push and exit; ci-watcher mutates the `kind: "validate-pending"` DQ entry per `.claude/rules/decision-queue.md` §"Two-phase validation under Shape G" + `.claude/rules/advisor-orchestrator.md` §5.2.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment ready for `v1-ship-1`; branch is `phase-v1-ship-1`; prior phase deliverables intact on base; AGPL-NOTICE.md present; workflow YAMLs present; the dual-bootstrap topology unchanged.

**FILES:**

```yaml
creates: []
modifies: []
requires: []
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` + R5 — enumerate ALL probes explicitly):**

```bash
# Probe 0 — Docker daemon running (needed only for any ad-hoc local e2e run; Shape G runs off-box)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — branch sanity
test "$(git branch --show-current)" = "phase-v1-ship-1" || { echo "WRONG BRANCH"; exit 1; }

# Probe 2 — working tree clean
test -z "$(git status --porcelain)" || { echo "WORKING TREE DIRTY"; exit 1; }

# Probe 3 — phase branch descends from governance-v0 (advisor-side merge-base sanity)
git merge-base --is-ancestor governance-v0 HEAD && echo "BASE OK" || { echo "BASE DRIFT"; exit 1; }

# Probe 4 — workflows present
test -f .github/workflows/cargo-validate-workspace.yml && \
  test -f .github/workflows/cargo-test-e2e.yml && \
  echo "WORKFLOWS OK" || { echo "WORKFLOWS MISSING"; exit 1; }

# Probe 5 — AGPL-NOTICE.md exists at repo root with substantive content
test -f AGPL-NOTICE.md && [ "$(wc -c < AGPL-NOTICE.md)" -gt 1000 ] && \
  echo "AGPL-NOTICE OK ($(wc -c < AGPL-NOTICE.md) bytes)" || { echo "AGPL-NOTICE MISSING OR EMPTY"; exit 1; }

# Probe 6 — GetSiteResponse struct still present (Task 1+2 anchor sanity)
grep -nE "^pub struct GetSiteResponse \{$" crates/db_views/site/src/api.rs | head -1
# EXPECT: one line, "337:pub struct GetSiteResponse {" or nearby (drift ±20 lines acceptable; symbol presence is the contract)

# Probe 7 — read_site constructor still has the captcha_enabled trailing field (Task 2 anchor sanity)
grep -nE "captcha_enabled: is_captcha_plugin_loaded\(\)," crates/api/api_crud/src/site/read.rs | head -1
# EXPECT: one line, near line 69 (drift ±10 lines acceptable; symbol presence is the contract)

# Probe 8 — exactly two pub async fn bootstrap() exist in e2e.rs (DQ #226 sanity)
grep -cE "^  pub async fn bootstrap\(\) -> LemmyResult<\(" crates/server/tests/e2e.rs
# EXPECT: 2 (one in governance_fixtures, one in admin_config_fixtures)

# Probe 9 — governance_fixtures::bootstrap is at e2e.rs:801 ± drift (the chosen bootstrap)
grep -nE "^  pub async fn bootstrap\(\) -> LemmyResult<\(" crates/server/tests/e2e.rs | head -1
# EXPECT: a line number near 801 (drift within ±50 lines is acceptable; symbol presence is the contract)

# Probe 10 — federated_instances sibling-route precedent still on lib.rs (Task 3 mirror sanity)
grep -nE '\.route\("/federated_instances", get\(\)\.to\(get_federated_instances\)\)' crates/api/routes/src/lib.rs | head -1
# EXPECT: one line near 288

# Probe 11 — concurrent-PR check (no other PR touches §11 files)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("crates/db_views/site/src/api\\.rs|crates/api/api_crud/src/site/read\\.rs|crates/api/api_crud/build\\.rs|crates/api/api/src/site/mod\\.rs|crates/api/api/src/site/source\\.rs|crates/api/routes/src/lib\\.rs|crates/server/tests/e2e\\.rs")) | {number, title, headRefName}'
# EXPECT: empty output. Non-empty → STOP and reconcile (file ownership conflict).
```

**EXPECT block:**

- Probes 0-11 all pass (exit 0 or expected output as commented).
- No NEGATIVE probe in Task 0 of this plan: Shape G means we don't validate the local wrapper here (cargo runs off-box on GitHub Actions). Exit-code propagation under Shape G lives in the workflow YAML (`run: cargo …` will surface non-zero exits via GitHub's step status).

**No commit at Task 0** — verification only.

### Task 1: Add `SourceDisclosure`, `GetSource`, `GetSourceResponse` DTOs

**ACTION:** in `crates/db_views/site/src/api.rs`, append three new public structs (`SourceDisclosure`, `GetSource`, `GetSourceResponse`) after `GetSiteResponse` at line 355. Do NOT yet add the `source_disclosure` field to `GetSiteResponse` (that's Task 2 — keeps field-add atomic with handler population).

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_views/site/src/api.rs   # append 3 new DTO structs after GetSiteResponse (after current line 355)
requires: []
```

**IMPLEMENT (file 1 of 1):** in `crates/db_views/site/src/api.rs` (append after the closing `}` of `GetSiteResponse` at current line 355). Use the verbatim derives + doc-comments from §10.1:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// AGPL §13 source-disclosure block.
///
/// Returned as part of every `GetSiteResponse` so a first-touch client
/// receives the SPDX license, the canonical repository URL, the running
/// fork commit, and the relative URL of the disclosure body.
pub struct SourceDisclosure {
  /// SPDX identifier, always "AGPL-3.0".
  pub license: String,
  /// Canonical fork repository URL.
  pub repo_url: String,
  /// HEAD commit SHA of the running fork build; "unknown" if unavailable at build time.
  pub fork_commit: String,
  /// Relative URL the client follows for the full notice body.
  pub disclosure_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Request for `GET /api/v4/source`. Unit struct — no input fields.
pub struct GetSource {}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response for `GET /api/v4/source`.
///
/// Returns the verbatim AGPL-NOTICE.md body bundled with the SPDX license identifier.
pub struct GetSourceResponse {
  pub notice: String,
  pub license: String,
}
```

**MIRROR:** `crates/db_views/site/src/api.rs:337-355` for the derive shape (already-present `GetSiteResponse` struct). Note: the parent uses `#[skip_serializing_none]` because it has `Option` fields; `SourceDisclosure` and `GetSourceResponse` have no `Option` fields so omit that attribute.

**GOTCHA:** doc-comments use single-line `///` only or have a blank `///` between paragraphs (per `feedback_clippy_doc_lazy_continuation_in_doc_comments.md`). The clippy lint `clippy::doc_lazy_continuation` fires on `/// line1\n///   line2` patterns. The §10.1 verbatim shape already complies.

**VALIDATE (story-checkpoint feeds §16a Story 1):**

```bash
git add crates/db_views/site/src/api.rs
git commit -m "feat(db_views_site): add SourceDisclosure, GetSource, GetSourceResponse DTOs (task 1)"
git push origin junior/v1-ship-1-task-1
# Capture workflow_run_id from:
#   gh run list --repo barrie-cork/lemmy --branch junior/v1-ship-1-task-1 --workflow cargo-validate-workspace --limit 1 --json databaseId
# Write kind: "validate-pending" DQ entry per Shape G discipline; advisor queues ci-watcher.
```

**Expected workflow conclusion:** `.github/workflows/cargo-validate-workspace.yml` on `junior/v1-ship-1-task-1` → `conclusion: "success"` (cargo check + clippy + test --no-run all green on the new types).

### Task 2 [P]: Add `source_disclosure` field to `GetSiteResponse` + populate in `read_site` + `build.rs`

**ACTION:** atomically (one commit) add `source_disclosure: SourceDisclosure` to `GetSiteResponse`, populate it in the `read_site` constructor at `crates/api/api_crud/src/site/read.rs:57`, and create `crates/api/api_crud/build.rs` to inject `BREHON_FORK_COMMIT`. Field add without handler population breaks the workspace compile; without build.rs the new `env!()` call fails to compile.

**FILES:**

```yaml
creates:
  - crates/api/api_crud/build.rs                  # inject BREHON_FORK_COMMIT via git rev-parse HEAD fallback chain
modifies:
  - crates/db_views/site/src/api.rs               # add `source_disclosure: SourceDisclosure` field to GetSiteResponse
  - crates/api/api_crud/src/site/read.rs          # import SourceDisclosure, declare three consts, populate source_disclosure
requires:
  - task: 1
    reason: Task 2 references the `SourceDisclosure` type added in Task 1
```

**IMPLEMENT (file 1 of 3):** in `crates/db_views/site/src/api.rs`, inside `GetSiteResponse` (struct keyword at line 337, closing `}` at line 355), append after the `captcha_enabled: bool` field (current line 354):

```rust
  pub captcha_enabled: bool,
  /// AGPL §13 source-disclosure surface; see [`SourceDisclosure`].
  pub source_disclosure: SourceDisclosure,
```

**IMPLEMENT (file 2 of 3):** in `crates/api/api_crud/src/site/read.rs`, update the imports + declare constants + extend the constructor:

1. **Update line 16's import** from:

   ```rust
   use lemmy_db_views_site::{SiteView, api::GetSiteResponse};
   ```

   to:

   ```rust
   use lemmy_db_views_site::{SiteView, api::{GetSiteResponse, SourceDisclosure}};
   ```

2. **Add three private constants** between line 18 (`use std::sync::LazyLock;`) and line 20 (`pub async fn get_site`):

   ```rust
   const BREHON_FORK_COMMIT: &str = env!("BREHON_FORK_COMMIT");
   const BREHON_REPO_URL: &str = "https://github.com/barrie-cork/lemmy";
   const SOURCE_DISCLOSURE_URL: &str = "/api/v4/source";
   ```

3. **Extend the `Ok(GetSiteResponse { ... })` constructor** (currently lines 57-70). Append after `captcha_enabled: is_captcha_plugin_loaded(),` at current line 69:

   ```rust
       captcha_enabled: is_captcha_plugin_loaded(),
       source_disclosure: SourceDisclosure {
         license: "AGPL-3.0".to_string(),
         repo_url: BREHON_REPO_URL.to_string(),
         fork_commit: BREHON_FORK_COMMIT.to_string(),
         disclosure_url: SOURCE_DISCLOSURE_URL.to_string(),
       },
     })
   ```

**IMPLEMENT (file 3 of 3):** create `crates/api/api_crud/build.rs` with the verbatim body from §10.4:

```rust
fn main() {
  let commit = std::env::var("BREHON_FORK_COMMIT")
    .or_else(|_| {
      std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .ok_or(std::env::VarError::NotPresent)
    })
    .unwrap_or_else(|_| "unknown".to_string());
  println!("cargo:rustc-env=BREHON_FORK_COMMIT={commit}");
  println!("cargo:rerun-if-env-changed=BREHON_FORK_COMMIT");
  println!("cargo:rerun-if-changed=../../../.git/HEAD");
}
```

**MIRROR:** §10.1 (DTO derive shape), §10.2 (read_site constructor), §10.4 (build.rs body).

**GOTCHA 1:** `Cargo.toml` already has a `[build-dependencies]` section (lines 66-68 of `crates/api/api_crud/Cargo.toml`) with unused serde/serde_json; no edit needed. Cargo auto-detects `build.rs` at the crate root because `[package]` has no `build = "..."` override.

**GOTCHA 2:** `lemmy_api_crud` does NOT depend on `lemmy_api_routes` (verified at plan-write; see §10.4). Do NOT attempt to re-export `BREHON_FORK_COMMIT` through `lemmy_api_routes` — that would create a circular dependency. The const lives privately in `read.rs`; no re-export, no extra workspace surface.

**GOTCHA 3:** R9 — only `read_site` constructs `GetSiteResponse`. After commit, the §11 verification command `rg "GetSiteResponse \{" crates/ --include="*.rs"` must show exactly three lines: the struct def, the rest-pattern destructure in `routes_v3/handlers.rs:249`, and the one constructor in `read.rs` (line drifts after edit). Zero new constructors.

**GOTCHA 4:** the `routes_v3/handlers.rs:249-258` destructure uses `..` rest-pattern (line 257) — additive-safe. DQ #227 RESOLVED: no edit to `routes_v3/handlers.rs` needed.

**GOTCHA 5:** the `build.rs`'s `git rev-parse HEAD` shells out at compile time. In environments where `.git/` is absent (e.g. source tarballs) AND the `BREHON_FORK_COMMIT` env var is unset, the fallback returns the literal string `"unknown"` — non-fatal. CI/Docker should set the env var explicitly to bypass the git call. This matches the parked plan's risk profile (§7.1 risk table); no change.

**GOTCHA 6:** `rerun-if-changed=../../../.git/HEAD` — the path is relative to the manifest dir (`crates/api/api_crud/`). Three `..` segments reach repo root. Verified by counting; matches the parked plan's directive (the path is the same number of segments whether build.rs lives in `crates/api/routes/` or `crates/api/api_crud/` — both crates sit at the same depth).

**VALIDATE (story-checkpoint feeds §16a Story 1):**

```bash
git add crates/db_views/site/src/api.rs crates/api/api_crud/src/site/read.rs crates/api/api_crud/build.rs
git commit -m "feat(api_crud): wire source_disclosure into GetSiteResponse + build.rs (task 2)"
git push origin junior/v1-ship-1-task-2
```

**Expected workflow conclusion:** `.github/workflows/cargo-validate-workspace.yml` on `junior/v1-ship-1-task-2` → `conclusion: "success"`. Specifically asserts: (a) the new field type-checks against `SourceDisclosure`, (b) the build script emits `cargo:rustc-env=BREHON_FORK_COMMIT=<sha>` and `env!()` resolves, (c) the routes_v3 rest-pattern destructure still compiles (non-breaking field add), (d) no orphan constructor breakage.

### Task 3 [P]: Add `get_source` handler + route registration + module re-export

**ACTION:** create `crates/api/api/src/site/source.rs` with the `get_source` handler (§10.3 body); add `pub mod source;` to `crates/api/api/src/site/mod.rs`; register `.route("/source", get().to(get_source))` immediately after the `/site` scope in `crates/api/routes/src/lib.rs` AND add `source::get_source,` to the `lemmy_api::site::{...}` use-block.

**FILES:**

```yaml
creates:
  - crates/api/api/src/site/source.rs
modifies:
  - crates/api/api/src/site/mod.rs                # add `pub mod source;` (append, becomes line 9)
  - crates/api/routes/src/lib.rs                  # add `source::get_source,` to use-block; add `.route("/source", ...)` at scope-level
requires:
  - task: 1
    reason: handler returns Json<GetSourceResponse>; the type must exist on the phase branch before this compile
```

**IMPLEMENT (file 1 of 3):** create `crates/api/api/src/site/source.rs` with the verbatim body from §10.3:

```rust
use actix_web::web::Json;
use lemmy_db_views_site::api::GetSourceResponse;
use lemmy_utils::error::LemmyResult;

const AGPL_NOTICE: &str = include_str!("../../../../../AGPL-NOTICE.md");

/// `GET /api/v4/source` — returns the verbatim AGPL-NOTICE.md body.
///
/// Public endpoint; no auth required. Read-only; no DB access.
/// Surfaces the AGPL §13 source-disclosure requirement to any connecting client.
pub async fn get_source() -> LemmyResult<Json<GetSourceResponse>> {
  Ok(Json(GetSourceResponse {
    notice: AGPL_NOTICE.to_string(),
    license: "AGPL-3.0".to_string(),
  }))
}
```

**IMPLEMENT (file 2 of 3):** in `crates/api/api/src/site/mod.rs` (8 lines at HEAD; the new line is line 9), append:

```rust
pub mod source;
```

**IMPLEMENT (file 3 of 3):** in `crates/api/routes/src/lib.rs`:

1. **Update the `lemmy_api::site::{...}` use-block** at lines 107-124. Add `source::get_source,` as a new line in the `site::` group, alphabetically after `registration_applications::{...},`. Resulting shape:

   ```rust
     site::{
       admin_allow_instance::admin_allow_instance,
       admin_block_instance::admin_block_instance,
       admin_list_users::admin_list_users,
       federated_instances::get_federated_instances,
       list_all_media::list_all_media,
       mod_log::get_mod_log,
       purge::{ ... },
       registration_applications::{ ... },
       source::get_source,
     },
   ```

2. **Add `.route("/source", get().to(get_source))`** immediately after the `/site` scope's closing `,` at line 230, and immediately before the existing `.route("/modlog", get().to(get_mod_log))` at line 231. Resulting shape (showing context around line 230):

   ```rust
           scope("/site")
             .route("", get().to(get_site))
             .route("", post().to(create_site))
             .route("", put().to(edit_site))
             .route("/icon", post().to(upload_site_icon))
             .route("/icon", delete().to(delete_site_icon))
             .route("/banner", post().to(upload_site_banner))
             .route("/banner", delete().to(delete_site_banner)),
         )
         // AGPL §13 source disclosure — public, no auth, no DB
         .route("/source", get().to(get_source))
         .route("/modlog", get().to(get_mod_log))
   ```

**MIRROR:** §10.3 (federated_instances minimal handler shape), §10.5 (site scope + sibling-route precedent).

**GOTCHA 1:** `include_str!` is **relative to the source file containing the macro call** — from `crates/api/api/src/site/source.rs`, 5 `..` segments reach repo root: `..` → `src/`, `..` → `api/` (inner crate), `..` → `api/` (parent dir), `..` → `crates/`, `..` → repo root. Then `/AGPL-NOTICE.md`. The exact macro call is `include_str!("../../../../../AGPL-NOTICE.md")`. Re-count if the impl-task questions the depth.

**GOTCHA 2:** `include_str!` is compile-time. If `AGPL-NOTICE.md` is missing or unreadable at build time, the build fails loudly with `error: couldn't read AGPL-NOTICE.md` — the desired behaviour per PRD §7.1 risk table and Task 0 Probe 5 (which asserts presence at phase start).

**GOTCHA 3:** the route is at the `/api/v4` scope level, NOT inside `/site`. Placing it inside `/site` resolves to `/api/v4/site/source` and contradicts PRD §7.1 ship-criteria. Read the line context around line 230 carefully — the placement is between `)` (end of `/site` scope's `.service(...)`) and `.route("/modlog", ...)`.

**GOTCHA 4:** the handler takes **no parameters**. No `context: Data<LemmyContext>`. No `Query<...>`. Adding either is correct-but-wasteful (no DB read, no input). Keep it minimal.

**GOTCHA 5:** `use actix_web::{guard, web::*};` at line 1 of `lib.rs` brings `get` and `scope` into scope. No new `use` line needed beyond the `source::get_source,` entry in the `lemmy_api::site::{...}` import block.

**VALIDATE (story-checkpoint feeds §16a Story 2):**

```bash
git add crates/api/api/src/site/source.rs crates/api/api/src/site/mod.rs crates/api/routes/src/lib.rs
git commit -m "feat(api): add get_source handler + /api/v4/source route (task 3)"
git push origin junior/v1-ship-1-task-3
```

**Expected workflow conclusion:** `.github/workflows/cargo-validate-workspace.yml` on `junior/v1-ship-1-task-3` → `conclusion: "success"`. Specifically asserts: (a) `include_str!` resolves the AGPL-NOTICE.md path, (b) the handler compiles against `GetSourceResponse`, (c) the route registration imports `get_source` correctly. After daemon finalize-merge of Task 3, the phase-branch tip does not yet carry Task 4's e2e; the next workflow run on the phase tip will run the existing test suite minus the new test.

### Task 4: Add e2e test `agpl_source_disclosure_surface_returns_notice`

**ACTION:** append a single new test to `crates/server/tests/e2e.rs` exercising both `/api/v4/site` (asserting `source_disclosure` field shape) and `/api/v4/source` (asserting notice body present). One single Edit at file end — large e2e.rs edits hang Junior workers (DQ #117 + multiple retros); keep this edit one anchor-Edit append, well under 100 lines.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append agpl_source_disclosure_surface_returns_notice test (~80 lines, single anchor-Edit at file end)
requires:
  - task: 2
    reason: GetSiteResponse must carry source_disclosure for the assertion to deserialize
  - task: 3
    reason: /api/v4/source route must be registered for the second GET to return 200
```

**IMPLEMENT (file 1 of 1):** append after the last existing test in `crates/server/tests/e2e.rs`. Use Case A discipline (per §10.6) and the in-process HTTP pattern (per §10.7). Inline imports inside the test fn (matches the e2e.rs sibling convention):

```rust
#[tokio::test(flavor = "multi_thread")]
async fn agpl_source_disclosure_surface_returns_notice() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::{App, test, web::Data};
  use lemmy_db_views_site::api::{GetSiteResponse, GetSourceResponse};
  use lemmy_utils::rate_limit::RateLimit;

  let (_container, context, _db_url) = governance_fixtures::bootstrap().await?;

  let rate_limit = RateLimit::with_debug_config();
  let app = test::init_service(
    App::new()
      .app_data(Data::new(context.clone()))
      .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit)),
  )
  .await;

  // --- 1. GET /api/v4/site returns source_disclosure block. ---
  let site_req = test::TestRequest::get().uri("/api/v4/site").to_request();
  let site_resp = test::call_service(&app, site_req).await;
  assert_eq!(site_resp.status().as_u16(), 200, "/api/v4/site must return 200");

  let site_body_bytes = test::read_body(site_resp).await;
  let site_body: GetSiteResponse = serde_json::from_slice(&site_body_bytes)?;

  assert_eq!(
    site_body.source_disclosure.license, "AGPL-3.0",
    "source_disclosure.license must be 'AGPL-3.0' per ADR-011"
  );
  assert_eq!(
    site_body.source_disclosure.disclosure_url, "/api/v4/source",
    "source_disclosure.disclosure_url must point to /api/v4/source"
  );
  assert!(
    !site_body.source_disclosure.repo_url.is_empty(),
    "source_disclosure.repo_url must be non-empty"
  );
  assert!(
    !site_body.source_disclosure.fork_commit.is_empty(),
    "source_disclosure.fork_commit must be non-empty (build.rs default 'unknown' is acceptable)"
  );

  // --- 2. GET /api/v4/source returns the AGPL notice body. ---
  let source_req = test::TestRequest::get().uri("/api/v4/source").to_request();
  let source_resp = test::call_service(&app, source_req).await;
  assert_eq!(source_resp.status().as_u16(), 200, "/api/v4/source must return 200");

  let source_body_bytes = test::read_body(source_resp).await;
  let source_body: GetSourceResponse = serde_json::from_slice(&source_body_bytes)?;

  assert_eq!(source_body.license, "AGPL-3.0");
  assert!(
    source_body.notice.contains("GNU Affero General Public License"),
    "AGPL-NOTICE.md body must contain the canonical license name"
  );
  assert!(
    source_body.notice.len() > 100,
    "notice body must be substantive (got {} bytes)",
    source_body.notice.len()
  );

  Ok(())
}
```

**MIRROR:** §10.6 (`report_to_modlog_golden_path` outer shape at `crates/server/tests/e2e.rs:2164` — Case A `LemmyResult<()>`; `governance_fixtures::bootstrap()` at line 801 — chosen bootstrap per DQ #226), §10.7 (`all_mvp_endpoints_return_non_404` in-process HTTP at lines 3693-3815).

**GOTCHA 1 (R8 — Case A discipline):** outer return is `lemmy_utils::error::LemmyResult<()>` (Case A per `feedback_lemmy_error_no_std_error.md`). `governance_fixtures::bootstrap()` returns `LemmyResult<...>` so bare `?` propagates `LemmyError → LemmyError` directly. `serde_json::from_slice` returns `Result<T, serde_json::Error>` and LemmyError already has `From<serde_json::Error>` (sibling tests use bare `?` on it — e.g. `e2e.rs:4413`). Do NOT use `Result<(), Box<dyn Error>>` outer (Case B) — that would mix shapes against the existing Case A sibling and trigger §G4 row 4c hard refusal.

**GOTCHA 2 — dual-bootstrap resolution (DQ #226):** call `governance_fixtures::bootstrap()` at `crates/server/tests/e2e.rs:801`, NOT `admin_config_fixtures::bootstrap()` at line 5578. The sibling `admin_config_fixtures::bootstrap()` bumps rate-limit buckets internally (for admin multi-write tests), depends on `super::governance_fixtures::start_postgres` + `apply_all_schema` (lines 5590-5598), and is intended for admin config tests — not a generic HTTP-surface test. The advisor pre-resolved this; do NOT file a blocker about which to pick.

**GOTCHA 3 — single-edit append discipline:** e2e.rs is 14,775 lines at HEAD. Edits into e2e.rs that span large ranges or use multiple Edit operations hang Junior workers (DQ #117 + many retros — the discipline name "junior worker e2e edit hang" is the canonical handle in the corpus). This task is a single anchor-Edit at file end — append after the last `Ok(())` of the last existing test. ~80 lines total, well under the hang threshold. **One Edit call, not two; not a Write-replace.**

**GOTCHA 4 — test name uniqueness:** the test name `agpl_source_disclosure_surface_returns_notice` MUST be unique in `e2e.rs`. Pre-edit verification:

```bash
grep -cE "async fn agpl_source_disclosure_surface_returns_notice" crates/server/tests/e2e.rs
# EXPECT: 0 (the test does not yet exist)
```

If non-zero, the refactor tier may have added a colliding test name; surface as a `kind: "blocker"` DQ — the planner did not anticipate the collision.

**GOTCHA 5:** `RateLimit::with_debug_config()` exists at `lemmy_utils::rate_limit::RateLimit` (sibling test at `crates/server/tests/e2e.rs:826` and 10+ other call sites). The new test does NOT bump buckets (we only make 2 GETs; the 6/300s Post bucket is irrelevant). No `enum_map!` block needed.

**VALIDATE (story-checkpoint feeds §16a Story 3):**

```bash
git add crates/server/tests/e2e.rs
git commit -m "test(e2e): assert AGPL §13 disclosure surface via /api/v4/site + /api/v4/source (task 4)"
git push origin junior/v1-ship-1-task-4
```

**Expected workflow conclusion (Phase 1):** `.github/workflows/cargo-validate-workspace.yml` on `junior/v1-ship-1-task-4` → `conclusion: "success"` (workspace check + clippy + test --no-run for the new test).

**Expected workflow conclusion (Phase 2):** after the daemon finalize-merges `junior/v1-ship-1-task-4` into `phase-v1-ship-1`, the advisor raises a `kind: "validate-pending"` DQ for the new phase-branch tip and either (a) runs `.github/workflows/cargo-test-e2e.yml` via `gh workflow run` (user-gate 4 — dispatch mode) or (b) runs e2e locally per `.claude/rules/advisor-orchestrator.md` §5.2 validate-pending-laptop handler (user-gate 4 — local mode). Either path expects the new test `agpl_source_disclosure_surface_returns_notice` in the output as `passed`, and the pre-existing test count + pass count unchanged.

### Task 5: Retro

**Goal:** author retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`. One H2 per role (Advisor / Planning / Impl / BM) with signals + lessons. Promote any new lessons to `.claude/lessons/feedback_*.md` in the same retro commit (per `feedback_one_system_memory_in_repo.md`).

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-ship-1-retro.md
modifies: []
requires:
  - task: 4
    reason: retro reflects on the full impl arc + the e2e green signal
```

> If the retro proposes new durable lessons, the same commit adds `.claude/lessons/feedback_*.md` files + indexes them in `.claude/lessons/MEMORY.md`; expand `creates:` and `modifies:` accordingly. Default shape above is the minimum.

**Three required H2 sections** (per `feedback_retro_not_report.md`):

```markdown
# v1-ship-1 retro

## What surprised us
- (per-role signals; one bullet per role at minimum — Advisor / Planning / Impl / BM)

## What to change
- (concrete deltas: rule edits, lesson promotions, brief-template tweaks)

## What to carry forward
- (patterns + decisions that worked; cite prior-phase recurrence if applicable)
```

**Suggested signals to harvest (planner pre-seeds):**

- **Did the re-plan discipline (parked plan + brief + refresh) save time vs a fresh plan?** If yes → lesson "parked-plan refresh recipe" worth promoting.
- **Did the build.rs location switch (api_crud over routes) cause any downstream surprise?** If the impl-task hit a circular-dep build error, that's evidence the §10.4 rationale carried (and the parked plan's primary path was indeed broken).
- **Did the single anchor-Edit on e2e.rs land cleanly under the hang threshold?** If yes → confirm the discipline; if no → escalate.
- **Did DQ #226 + DQ #227 resolutions pre-empt round-trips?** Should help the planning brief template add a "pre-clarify ambiguous DTO callers" step.
- **(NEW from this re-plan task)** Was the `.claude/PRPs/plans/` sensitive-file gate also blocking the planning Junior task? If yes → the daemon needs a permissions fix; the worker should run with permissions that allow writes to `.claude/PRPs/plans/**`.

**No commit at retro draft** — the BM subagent commits the retro after user sign-off (per `.claude/rules/advisor-orchestrator.md` user gate 6).

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** `cargo check --workspace --features full` — via `.github/workflows/cargo-validate-workspace.yml` after every push (Shape G).
- **Lint:** `cargo clippy --workspace --features full --no-deps -- -D warnings` — same workflow.
- **Test target compile:** `cargo test --no-run -p lemmy_server --test e2e` — same workflow (per the workflow YAML line 95).
- **e2e execution:** `cargo test --workspace --features full --test e2e -- --test-threads=1` — via `.github/workflows/cargo-test-e2e.yml` on `phase-v1-ship-1` tip after daemon finalize-merge of Task 4 (workflow_dispatch only; the advisor invokes per user gate 4).
- **Migration round-trip:** N/A — no schema changes.

User-gate 4 (Phase 2 e2e — local vs dispatch) fires when `phase-v1-ship-1` tip reaches the e2e validation point. Options per `.claude/rules/advisor-orchestrator.md` §3.2:

- **(a) Local** — laptop runs `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true`. ~26 min. Zero billed minutes.
- **(b) Dispatch** — `gh workflow run cargo-test-e2e.yml --repo barrie-cork/lemmy --ref phase-v1-ship-1`. ~26 min. Billed.

Either path: success = workflow conclusion `success` AND the new test name `agpl_source_disclosure_surface_returns_notice` appears in the output as `passed`.

---

## 15. Validation commands (DoD)

> Shape G plan (per `feedback_features_full_workspace_only.md` + `feedback_features_full_p_crate_incompatible.md` + plan.template.md §15.6). Commands here are the canonical shape; impl-task subagents push and exit; ci-watcher mutates DQ entries from workflow runs.

### 15.1 Static analysis (per task — workspace check)

**Workflow:** `.github/workflows/cargo-validate-workspace.yml` on `junior/v1-ship-1-task-N`.

Workflow internal step (line 88-89):

```yaml
- name: cargo check workspace + features full
  run: cargo check --workspace --features full
```

**EXPECT:** workflow conclusion `success`.

Verbatim shape verified against `.github/workflows/cargo-validate-workspace.yml` lines 18-28 at plan-write — the trigger paths include `crates/**` (matches every §11 path) and `Cargo.toml`/`Cargo.lock` (matches the existing `[build-dependencies]` block in `crates/api/api_crud/Cargo.toml`; no edit), so every Task 1-4 push triggers the workflow.

### 15.2 Lint (per task — uniform R6)

**Workflow:** same as 15.1.

Workflow internal step (line 91-92):

```yaml
- name: cargo clippy workspace + features full
  run: cargo clippy --workspace --features full --no-deps -- -D warnings
```

**EXPECT:** workflow conclusion `success`. Zero new warnings introduced; `clippy::doc_lazy_continuation` does not fire on the new `///` doc-comments per §10.1.

### 15.3 Test target compile (R7 — Tasks 1, 2, 3, 4 all touch a struct or re-export)

**Workflow:** same as 15.1.

Workflow internal step (line 94-95):

```yaml
- name: cargo test compile (no-run)
  run: cargo test --no-run -p lemmy_server --test e2e
```

**EXPECT:** workflow conclusion `success`.

> **R7 trigger reasoning:** Task 1 adds new public types in `lemmy_db_views_site`; Task 2 adds a public field; Task 3 adds a new public handler symbol; Task 4 adds a test that imports + asserts both. The workspace-validate workflow runs this step on every push, so every Task 1-4 push satisfies R7 automatically.

### 15.4 e2e test execution (Task 4 — post-finalize-merge of `phase-v1-ship-1`)

**Workflow:** `.github/workflows/cargo-test-e2e.yml` on `phase-v1-ship-1` (post-finalize-merge of Task 4). Trigger: `workflow_dispatch` only (per workflow line 28-34); advisor invokes via `gh workflow run` per user gate 4.

Workflow internal steps (lines 84-90):

```yaml
- name: Compile e2e test target
  run: cargo test --workspace --features full --test e2e --no-run

- name: Run e2e (single-threaded; testcontainers per test)
  ...
  run: cargo test --workspace --features full --test e2e -- --test-threads=1
```

**EXPECT:** workflow conclusion `success`. Test name `agpl_source_disclosure_surface_returns_notice` appears in the output as `passed`. The full e2e suite runs (workflow uses `--test-threads=1`); the new test is fixture-independent so flake risk is bounded.

> **Local-mode alternative (per `.claude/rules/advisor-orchestrator.md` §3.2 gate 4 / `feedback_windows_e2e_requires_bat_wrapper.md` + §5.2 validate-pending-laptop handler):**
>
> ```
> cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-v1-ship-1-<sha>.log 2>&1 && echo E2E_EXIT_0 >> .claude/runlog/e2e-v1-ship-1-<sha>.log || echo E2E_EXIT_NONZERO >> .claude/runlog/e2e-v1-ship-1-<sha>.log"
> ```
>
> with `run_in_background: true`. Same expected output. Never bare `cargo test` on Windows (libpq.dll requires the bat wrapper); never `-p lemmy_server --features full` (lemmy_server has no `full` feature).

### 15.5 Cross-cutting verification

Bulleted checklist the planner asserts hold at end-of-phase:

- [ ] No file outside §11 list edited (verify via `git diff --stat governance-v0...HEAD` enumerating only the §11 paths plus the retro file).
- [ ] **R1** (i64::from): no `as` casts for int comparisons (verify: `rg "as i64|as i32" crates/api/api_crud/src/site/read.rs crates/db_views/site/src/api.rs crates/api/api/src/site/source.rs` returns empty — none of the §10 verbatim blocks introduce a cast).
- [ ] **R5** (Task 0 enumerated all 12 probes; Probe 5 confirms AGPL-NOTICE.md presence + size; Probe 8 confirms two bootstraps exist).
- [ ] **R6** (clippy `--no-deps -- -D warnings` uniform; workflow step 91-92 confirms).
- [ ] **R7** (test-target compile after each struct/re-export touch; workflow step 94-95 runs it on every push).
- [ ] **R8** (test fn outer return is `lemmy_utils::error::LemmyResult<()>`; verify in `crates/server/tests/e2e.rs` for `async fn agpl_source_disclosure_surface_returns_notice`).
- [ ] **R9** (`rg "GetSiteResponse \{" crates/ --include="*.rs"` returns exactly 3 lines: definition, rest-pattern destructure, one constructor — see §11 verification command).
- [ ] Doc-comments on `SourceDisclosure`, `GetSource`, `GetSourceResponse` pass `clippy::doc_lazy_continuation`.
- [ ] `AGPL-NOTICE.md` is byte-equal to the build-time `include_str!` target (verify: `wc -c AGPL-NOTICE.md` matches whatever the build returns; if `AGPL-NOTICE.md` is edited mid-phase, that's a separate `docs(brehon-fork)` commit on `governance-v0`, not in this plan).
- [ ] `BREHON_FORK_COMMIT` env injection: the workspace-check workflow runs `cargo check --workspace --features full`, which compiles `lemmy_api_crud` → invokes `build.rs` → emits `cargo:rustc-env=BREHON_FORK_COMMIT=<sha>`. The const compiles in `read.rs`. Acceptance: workflow `success` on Task 2.
- [ ] **Dual-bootstrap resolution (DQ #226):** verify the new test's call site uses `governance_fixtures::bootstrap()` (not `admin_config_fixtures::bootstrap()`) — `grep -nE "governance_fixtures::bootstrap\(\)\.await" crates/server/tests/e2e.rs` includes the new test's line.

### 15.6 DoD per workflow (Shape G)

- **DoD entry**: `cargo-validate-workspace.yml` on `junior/v1-ship-1-task-N` SHA `<sha>` → `conclusion: "success"`. Required for each Task 1-4 push.
- **DoD entry**: `cargo-test-e2e.yml` on `phase-v1-ship-1` SHA `<post-finalize-merge-sha>` → `conclusion: "success"`. Required after Task 4 finalize-merge.
- **Validation command (advisor-side, post-push):**

  ```bash
  gh run list --repo barrie-cork/lemmy --branch <branch> --workflow <workflow-file> --limit 1 --json conclusion,databaseId --jq '.[0]'
  ```

  **EXPECT:** `{"conclusion": "success", "databaseId": <id>}`.

- **DQ entry (impl-task writes Phase 1; advisor writes Phase 2 per `.claude/rules/decision-queue.md` §"Two-phase validation under Shape G"):** `kind: "validate-pending"`, `from: "impl"` (Phase 1) or `from: "advisor"` (Phase 2), `workflow_run_id: <int>`, `branch: "<branch>"`, `phase_task: <N>`. ci-watcher mutates by matching `workflow_run_id`.

---

## 16. Acceptance criteria

- [ ] All 6 tasks completed in dependency order (Task 0 audit, Tasks 1-4 impl, Task 5 retro).
- [ ] §15.1 (cargo check workspace) → `success` after every push (Tasks 1-4).
- [ ] §15.2 (cargo clippy `--no-deps -- -D warnings`) → `success` after every push.
- [ ] §15.3 (cargo test --no-run -p lemmy_server --test e2e) → `success` after every push.
- [ ] §15.4 (cargo-test-e2e.yml on phase tip) → `success`; `agpl_source_disclosure_surface_returns_notice` passes; pre-existing test count + pass count unchanged.
- [ ] §15.5 (cross-cutting verification) — all 11 boxes ticked.
- [ ] §16a stories — all 3 stories `[done]`.
- [ ] No edits to files outside §11 list (verified by `git diff --stat governance-v0...HEAD`).
- [ ] Retro committed per Task 5.
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`.
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`.
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-ship-1-verify.md` shows all stories ✓.
- [ ] **Manual smoke (advisor-side, post-merge, optional):** `curl http://localhost:8536/api/v4/site | jq .source_disclosure` returns the four-field block; `curl http://localhost:8536/api/v4/source | jq .notice | head` returns notice text. Verifiable by an external observer with no project context (PRD §7.1 ship-criteria).

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: First-touch handshake exposes source-disclosure pointer

- **Composing tasks:** Task 1 (DTOs), Task 2 (field + handler populate + build.rs).
- **Checkpoint command:**

  ```bash
  cargo test --workspace --features full --test e2e agpl_source_disclosure_surface_returns_notice -- --nocapture
  ```

  (Run inside the e2e workflow; locally via the `.bat` wrapper per §15.4 local-mode block.)
- **Expected output:** `1 passed; 0 failed`; the assertions `source_disclosure.license == "AGPL-3.0"` and `source_disclosure.disclosure_url == "/api/v4/source"` both pass.
- **Brief-Scope outputs to verify** (used by `/brehon-verify`):
  - `crates/db_views/site/src/api.rs` contains `pub struct SourceDisclosure`.
  - `crates/db_views/site/src/api.rs` contains `pub source_disclosure: SourceDisclosure` inside `GetSiteResponse`.
  - `crates/api/api_crud/src/site/read.rs` constructor literal contains `source_disclosure: SourceDisclosure {`.
  - `crates/api/api_crud/build.rs` exists and contains the literal `"cargo:rustc-env=BREHON_FORK_COMMIT"`.
  - `crates/api/api_crud/src/site/read.rs` contains `const BREHON_FORK_COMMIT: &str = env!("BREHON_FORK_COMMIT");`.

### Story 2: Disclosure URL resolves to the AGPL notice body

- **Composing tasks:** Task 3 (handler + route + mod wiring).
- **Checkpoint command:** same as Story 1 (one test covers both surfaces; grep the test output for the second `assert!` chain on `source_body.notice`).
- **Expected output:** `1 passed; 0 failed`; `source_body.notice.contains("GNU Affero General Public License")` passes.
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/site/source.rs` exists and contains `pub async fn get_source`.
  - `crates/api/api/src/site/source.rs` contains `include_str!("../../../../../AGPL-NOTICE.md")`.
  - `crates/api/api/src/site/mod.rs` contains `pub mod source;`.
  - `crates/api/routes/src/lib.rs` contains `.route("/source", get().to(get_source))`.
  - `crates/api/routes/src/lib.rs` contains `source::get_source,` inside the `lemmy_api::site::{...}` use-block.

### Story 3: Named e2e test covers both surfaces with Case A discipline

- **Composing tasks:** Task 4.
- **Checkpoint command:**

  ```bash
  grep -nE "^async fn agpl_source_disclosure_surface_returns_notice\(\) -> lemmy_utils::error::LemmyResult<\(\)>" crates/server/tests/e2e.rs
  ```

- **Expected output:** one match; the line number is whatever the append landed at (~14,775+).
- **Brief-Scope outputs to verify:**
  - `crates/server/tests/e2e.rs` contains `async fn agpl_source_disclosure_surface_returns_notice() -> lemmy_utils::error::LemmyResult<()>`.
  - Test body contains BOTH `test::TestRequest::get().uri("/api/v4/site")` AND `test::TestRequest::get().uri("/api/v4/source")` calls.
  - Test uses `governance_fixtures::bootstrap()` (NOT `admin_config_fixtures::bootstrap()`).
  - Test uses Case A discipline (outer `LemmyResult<()>`, bare `?` propagation, no `.map_err(|e| anyhow::anyhow!(...))?` bridges).

> **Verification mapping:** the advisor's `/brehon-verify` step iterates this section, runs each Story's Checkpoint against the worktree branch, and confirms each Brief-Scope output exists + matches its structural pattern. Phantoms trigger the catch-fire procedure in `.claude/rules/advisor-orchestrator.md`.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (all 12 probes confirmed).
- [ ] Tasks 1-4 committed (one commit per task per `feedback_pr_per_phase.md`).
- [ ] §15 validation green at every gate (4 workspace-check workflow runs + 1 e2e workflow run, all `success`).
- [ ] §16a stories all `[done]` (3 stories).
- [ ] Retro committed per Task 5.
- [ ] PR opened by BM session against `governance-v0` with `--repo barrie-cork/lemmy`.
- [ ] CodeRabbit review complete; findings triaged.
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-ship-1-verify.md` shows all 3 stories ✓.
- [ ] Post-merge `phase-v1-ship-1` branch retained for retro reads.

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Upstream rebase between plan-write and impl shifts `GetSiteResponse` line number from 337 | MED | LOW | MIRROR refs cite the symbol (`pub struct GetSiteResponse {`) + line — plan self-heals; impl-task re-greps the symbol if line drifts. Per `feedback_plan_baseline_self_reference.md`. |
| `BREHON_FORK_COMMIT` env empty in CI without `.git/` (e.g. sparse checkout, source tarball) | LOW | LOW | `build.rs` fallback chain returns `"unknown"` non-fatally; AGPL §13 still complies (the notice URL is the load-bearing surface, not the SHA) |
| `include_str!` path miscount (5 vs 4 vs 6 `..` segments) from `crates/api/api/src/site/source.rs` | LOW | LOW | Task 0 Probe 5 confirms `AGPL-NOTICE.md` presence at repo root; if path is wrong, build fails loudly (compile-time error names the path). Impl-task re-counts segments before commit; §10.3 GOTCHA 1 walks the count. |
| `ts-rs` codegen drift on new fields (`GetSiteResponse.source_disclosure`) | LOW | LOW | `ts(optional_fields, export)` derive is consistent with sibling fields; field add is additive in TS; no required-field collision |
| `read_site` cache holds stale `source_disclosure` on hot reload | NEGLIGIBLE | NEGLIGIBLE | Values are compile-time constants; instance-lifetime cache is correct |
| Lemmy upstream rebase changes `GetSiteResponse` shape | MED | LOW | Field addition is additive; rebase conflicts resolve by re-adding the field at the new line number; the `read_site` constructor patch is mechanical |
| e2e flake from `governance_fixtures::bootstrap()` testcontainers cold-start (~30s) | LOW | LOW | Existing test pattern survives; the new test uses the same `bootstrap()` helper as every other governance test; CI workflow uses `--test-threads=1` to serialize containers |
| `clippy::doc_lazy_continuation` fires on multi-line doc-comments | MED | LOW | All `///` doc-comments in plan §10.1 are single-line OR have blank `///` between paragraphs; impl-task copies verbatim |
| AGPL-NOTICE.md path moves (e.g. `docs/AGPL-NOTICE.md`) post-impl | LOW | LOW | `include_str!` is compile-time; any path drift breaks the build immediately. Probe 5 in Task 0 confirms current location. |
| The 5-segment `../../../../../AGPL-NOTICE.md` is fragile to file moves | LOW | LOW | Acceptable trade-off; the alternative (read at runtime via `Settings`) defeats the goal of compile-time inclusion (zero runtime path-resolution risk). |
| Cohort A (Tasks 2 + 3) collides on the daemon when both finalize-merge into `phase-v1-ship-1` | LOW | LOW | Tasks 2 + 3 are file-disjoint (verified by §11 + the §13 FILES YAML overlap check); the daemon merge serialization handles each branch's tip independently. Per `feedback_ci_watcher_serial_per_task_pair.md`, ci-watcher dispatch is **one per logical task** (Tasks 2 + 3 = 2 ci-watchers in sequence per advisor cohort). |
| A second `pub async fn bootstrap()` exists post-rebase (3rd sibling appears) | LOW | LOW | Probe 8 in Task 0 asserts exactly 2 bootstraps. If a 3rd appears, Probe 8 fails; advisor surfaces the discrepancy + re-files DQ. |
| Misclassification: impl-task picks `admin_config_fixtures::bootstrap()` instead of `governance_fixtures::bootstrap()` | LOW | HIGH | §10.6 GOTCHA 2 + §15.5 R-check both call this out explicitly; Story 3 Brief-Scope output verifies the chosen bootstrap; DQ #226 RESOLUTION is binding. |

---

## 19. Notes

- **Re-plan rationale.** The design (AGPL §13 surface = field + endpoint) is unchanged from the parked plan. Every `file:line` MIRROR ref was re-derived against HEAD; the build.rs location promotion (api_crud over routes) follows from verified `[dependencies]` and was already documented as the parked plan's fallback path. Per the brief's principle: "the entire value of this re-plan is *citation freshness + the dual-bootstrap resolution*."
- **AGPL §13 substance vs surface.** This plan ships the *surface* (HTTP endpoint, DTO field). The *substance* (the `AGPL-NOTICE.md` text) is already authored; this plan does not edit it. If the legal text needs revision, that's a separate `docs(brehon-fork)` commit — independent of this plan and not part of v1-ship-1 acceptance.
- **Why `disclosure_url` is relative, not absolute.** The fork instance doesn't know its own public hostname at build time; the relative URL lets clients resolve against the server they're already talking to. Same reasoning as other Lemmy URL fields.
- **Why no auth on `/api/v4/source`.** ADR-011 requires source disclosure to all users interacting over the network. Auth-gating would defeat the §13 surface. Read-only; payload is ~3.5 KB; default rate-limit (`rate_limit.message()`) is appropriate.
- **Why include the SHA in the disclosure.** AGPL §13 binds the *running version* of the modified code. The SHA identifies the build precisely so an external auditor can fetch the exact source. `"unknown"` fallback is acceptable for source-tarball builds where `.git/` is absent — the auditor still has the repo URL.
- **Decision-queue pre-seed (planner → advisor):** none. DQ #226 + DQ #227 (advisor-side, already RESOLVED) are this plan's binding inputs. No new clarify entries from the planner. The single forward-looking concern (Task 0 Probe 8 catching a possible 3rd bootstrap appearance post-rebase) is encoded as a probe + risk row, not as a DQ.
- **Lesson trailer candidates (for retro Task 5 harvest, not yet promoted):**
  - "Parked plan + brief + refresh = re-plan recipe" — the time saved vs a fresh plan is worth a memorable name.
  - "Verify the dep graph BEFORE accepting any 're-export the const' wiring" — the parked plan's primary build.rs location was broken because the dep graph hadn't been verified; the fallback path was correct on the merits.
  - "Pre-clarify ambiguous DTO callers before planning" — DQ #227's RESOLUTION pattern (enumerate constructors AND destructures, classify rest-pattern destructures as non-breaking explicitly) saves a planner round-trip.
  - "`.claude/PRPs/plans/` is sensitive-file-gated for Junior workers" — the daemon spawn args / permission settings must allow planning workers to write to that path.

---

## 20. Confidence score

- **Plan correctness:** 9/10 — every file:line citation comes from a direct Read against HEAD `9504c806d` at plan-write time; mechanical Lemmy patterns; ADR-011 is the load-bearing constraint and this is its surfacing. One residual minor: the §10.5 line numbers for the `lemmy_api::site::{...}` use-block (107-124) were located via grep, not Read of the full range — impl-task confirms before edit.
- **Cargo budget:** 10/10 — Shape G; validation off-box; zero EliteDesk impact.
- **Test coverage:** 9/10 — one new e2e exercises both surfaces with Case A discipline; the missing surface (e.g. unauthenticated regression on `/api/v4/site` separately) is not in scope since the existing all-mvp test at `e2e.rs:3693` already covers non-404 for the site endpoint. PRD §7.1 ship-criteria fully exercised by the new test.

---

*Planned: 2026-05-16 (RE-PLAN against `governance-v0` HEAD `9504c806d`; design verbatim from parked `v1-ship-1.plan.md` @ `aea538166`; MIRROR refs refreshed; build.rs location promoted from fallback to primary on verified dep-graph evidence).*
*Status: DRAFT — pending advisor §3.4 DoD smoke test, §3.5 watchpoint specificity gate, and user gate 1 (plan approval).*
