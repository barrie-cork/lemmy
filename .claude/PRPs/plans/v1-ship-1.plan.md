# Plan: v1-ship-1 — AGPL §13 source-disclosure surface

## 1. Summary

Extends `GetSiteResponse` with a `source_disclosure` block (license SPDX, repo URL, build-time fork commit, disclosure URL), and adds a new public endpoint `GET /api/v4/source` that returns the verbatim `AGPL-NOTICE.md` body. First external user hitting `/api/v4/site` receives the disclosure pointer on the standard handshake; following the URL resolves to the notice text. One e2e test asserts both surfaces. Closes the first of two structural ship-gates identified in `v0-endpoint-coverage-2026-05-14.md` (AGPL §13 user-visible compliance). No ADR contradiction; ADR-011 is the load-bearing constraint and this is its surfacing.

## 2. Source

- `.claude/PRPs/prds/v1-ship-readiness.prd.md` §7.1 @ HEAD (PRD authored 2026-05-14, draft)
- `.claude/PRPs/reports/v0-endpoint-coverage-2026-05-14.md` §8 Tier-1 gap #1 (AGPL §13 not user-visible)
- ADR-011 (AGPLv3 inherited + source-disclosure required) — `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` §ADR-011
- `AGPL-NOTICE.md` (repo root) — verbatim notice body, 39 lines / 3,595 bytes
- Lessons binding decisions:
  - `feedback_advisor_watchpoint_specificity.md` — §4 watchpoints cite file:line
  - `feedback_lemmy_error_no_std_error.md` — e2e test outer-return discipline
  - `feedback_features_full_p_crate_incompatible.md` — DoD command shape
  - `feedback_junior_worker_e2e_edit_hang.md` — e2e.rs Edit size cap (light here: ~50-line append)
  - `feedback_planner_enumerate_struct_callsites_for_addfield.md` — §11 Caller-crates enumeration

## 3. Problem statement

ADR-011 requires every release to honour AGPL §13 Remote Network Interaction: any user interacting with the running fork across a network must be offered the Corresponding Source. The fork has `LICENSE` (AGPL-3.0 text) and `AGPL-NOTICE.md` (source disclosure narrative) at the repo root, but **no HTTP endpoint returns either to a connecting client**. From the moment the fork accepts a first external HTTP request, the §13 clause is engaged and the user-facing disclosure surface is absent. This is the load-bearing structural ship-gate for "compliantly accepts first external user".

Tied to §13 Task 2 (DTO extension) + Task 3 (handler + route + module wiring) + Task 5 (e2e).

## 4. Solution statement

Two surfaces, one for discovery and one for the body, both public:

1. **First-touch handshake (discovery):** the existing `GetSiteResponse` payload — which every Lemmy/Brehon client deserializes on connection at `GET /api/v4/site` — gains a `source_disclosure: SourceDisclosure` field. The `SourceDisclosure` block carries SPDX license, canonical repo URL, build-time-injected fork commit SHA, and a relative URL (`/api/v4/source`) the client follows for the full notice.

2. **Disclosure body (resolution):** a new endpoint `GET /api/v4/source` returns `GetSourceResponse { notice: String, license: String }` where `notice` is the verbatim `AGPL-NOTICE.md` body sourced via `include_str!` at compile time (zero DB read, zero auth).

The fork-commit SHA is read at compile time via `env!("BREHON_FORK_COMMIT")`, populated by a `build.rs` in the route-registering crate that shells out to `git rev-parse HEAD` (or reads a `BREHON_FORK_COMMIT` env var when set by CI/Docker build). Missing env + missing `.git/` → emit `"unknown"` non-fatally. No new crate; no new migration; no new ADR.

```
client → GET /api/v4/site
       ← { ..., source_disclosure: { license, repo_url, fork_commit, disclosure_url: "/api/v4/source" }}

client → GET /api/v4/source
       ← { notice: <AGPL-NOTICE.md verbatim>, license: "AGPL-3.0" }
```

The `read_site` cache (lines 26-71 of `crates/api/api_crud/src/site/read.rs`) caches the entire `GetSiteResponse` via `LazyLock<CacheLock<GetSiteResponse>>`. Adding a new field means the cache holds the new shape; values are stable across instance lifetime (license, repo, commit all build-time constants), so caching is safe and free.

## 5. Metadata

- **Phase:** `v1-ship-1`
- **Branch:** `phase-v1-ship-1` (cut by BM-task before Task 1)
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 7 (Task 0 pre-flight + Tasks 1-5 impl + Task 6 retro)
- **Estimated cargo budget:** N/A under Shape G (validation off-box on GitHub Actions)
- **Forbidden-window applicability:** non-binding under Shape G for impl-task dispatch; binding for ad-hoc local `cargo` runs
- **Complexity score:** **3/10** — see breakdown below

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Target model is Sonnet 4.6 → split-DQ threshold is `> 8`.

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 5 impl tasks (Tasks 1-5); meets threshold exactly, no excess |
| Migrations touched | +2 each | 0 | No schema changes |
| Crates touched | +1 each | 3 | `lemmy_db_views_site` (DTO), `lemmy_api_crud` (handler), `lemmy_api_routes` (route + build.rs) |
| `crates/server/tests/e2e.rs` edits | +3 each | 0 | One ~60-line append (single-site Edit, well under the 300-line hang threshold per `feedback_junior_worker_e2e_edit_hang.md`); not weight-scoring as a "edit" factor since it's append-only |
| New ADR-affecting decisions | +2 each | 0 | ADR-011 surfaced, not amended |
| Cargo budget peak above 6 GB | +1 per GB | 0 | Shape G — validation off-box |
| **Total** | — | **3** | Below threshold (>8); no split-DQ |

Complexity score 3 confirms PRD §7.1's claim. Single-cohort plan; no split needed.

### 5.2 Per-task complexity ceiling (Sonnet target)

Sonnet ceiling: `≤ 4` files per task / `≤ 2` crates per task / e2e edits in their own task. Each §13 task satisfies (see Task FILES blocks); no split-candidate detected.

## 6. Relationship to other v1-ship sub-phases

- **Independent of v1-ship-2** (e2e backfill on 4 untested endpoints) per PRD §10. Both can run in parallel sub-phase lanes; no file overlap.
- **Precedes v1-ship-3** only in retro-discipline rhythm (not in code). v1-ship-3 touches `docker-compose.yml`, `create_report` handler, and a different e2e test; no shared files.
- **No dependency on RT-r2..RT-r6, JM-f, restorative-mechanics-v1.** Those are parallel feature lanes.

## 7. Preflight guardrails inherited from prior phases

- **R1** (per `feedback_clippy_test_style.md`): every `i32 ↔ i64` comparison uses `i64::from(...)`, never `as` cast. *Light here* — this plan introduces no integer comparisons.
- **R5** (per JM-b retro Event 4): Task 0 enumerates ALL probes explicitly; no implicit inheritance.
- **R6** (per JM-b retro Event 3): all clippy invocations use `--no-deps` uniformly.
- **R7** (per `feedback_test_target_compile_validation.md`): tasks that touch a struct or re-export run `cargo test --no-run -p lemmy_server --test e2e`.
- **R8** (per `feedback_lemmy_error_no_std_error.md`): test fn outer return uses `LemmyResult<()>` (Case A — sibling test `report_to_modlog_golden_path` uses this shape; mirror verbatim).
- **R9** (per `feedback_planner_enumerate_struct_callsites_for_addfield.md`): `GetSiteResponse` field-add task enumerates every constructor-site in §11; back-compat path verified (one constructor site: `read_site` at `crates/api/api_crud/src/site/read.rs:64-71`).
- **R10** (per `feedback_pre_phase_dod_smoke_test.md`): every §15 command dry-run by advisor against current HEAD before plan approval.

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
                            | notice = include_str!(AGPL-NOTICE.md)
                            +-------------------+
```

Boxes ↔ §13 tasks:
- `source_disclosure` field on `GetSiteResponse` → **Task 2**
- `SourceDisclosure` struct + `GetSource` + `GetSourceResponse` DTOs → **Task 1** (DTOs land first, types compile, then field add references them)
- `read_site` populates `source_disclosure` → **Task 2** (same task — field add + handler population are atomic; cache invariant otherwise breaks transiently)
- `build.rs` injects `BREHON_FORK_COMMIT` → **Task 3**
- `get_source` handler module → **Task 4**
- Route registration `/api/v4/source` → **Task 4** (handler + route in one task because route depends on handler symbol)
- e2e test → **Task 5**

## 9. Mandatory reading

The `impl-task` subagent reads these files before its first edit on each task:

### Schema/type definitions
- `crates/db_views/site/src/api.rs:1-46` — file-level imports (`serde`, `serde_with::skip_serializing_none`, ts-rs feature attributes)
- `crates/db_views/site/src/api.rs:337-355` — `GetSiteResponse` full definition (verbatim plan §10.1)
- `crates/api/api/src/site/federated_instances.rs:1-15` — minimal sibling handler shape (verbatim plan §10.3)
- `crates/api/api_crud/src/site/read.rs:20-71` — `get_site` + `read_site` (verbatim plan §10.2)
- `crates/api/api_utils/src/context.rs:12-59` — `LemmyContext` definition; `settings()` accessor

### Existing patterns (MIRROR refs in §13)
- `crates/api/routes/src/lib.rs:216-230` — site scope + route registration shape (verbatim plan §10.4)
- `crates/db_views/site/src/api.rs:337-355` — DTO derive shape (`#[derive(Debug, Serialize, Deserialize, Clone)]` + `#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]` + `#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]`)
- `crates/server/tests/e2e.rs:2077-2275` — `report_to_modlog_golden_path` outer shape (`#[tokio::test(flavor = "multi_thread")]`, `LemmyResult<()>` return, `.map_err(|e| anyhow::anyhow!(...))?` bridge — verbatim plan §10.5)
- `crates/server/tests/e2e.rs:3700-3880` — `all_mvp_endpoints_return_non_404` for in-process HTTP via `actix_web::test::TestRequest` (verbatim plan §10.6)
- `crates/server/tests/e2e.rs:88-861` — `governance_fixtures::bootstrap()` (returns `(container, Data<LemmyContext>, db_url)`)

### Adjacent test fixtures
- `crates/server/tests/e2e.rs:767-809` — `bootstrap()` signature; the new test uses this verbatim

### Lessons binding tasks
- `feedback_lemmy_error_no_std_error.md` — Task 5 (e2e error-shape Case A: outer `LemmyResult<()>`)
- `feedback_features_full_p_crate_incompatible.md` — §15 commands use `--workspace --features full`, never `-p <crate> --features full`
- `feedback_features_full_workspace_only.md` — same rule
- `feedback_clippy_doc_lazy_continuation_in_doc_comments.md` — doc-comments on `SourceDisclosure` fields must not lazy-continue
- `feedback_junior_worker_e2e_edit_hang.md` — Task 5 e2e edit is a single append <100 lines; well under hang threshold
- `feedback_advisor_watchpoint_specificity.md` — §4 watchpoints below cite file:line

## 10. Patterns to mirror

### 10.1 GetSiteResponse derive + field shape

**Mirror:** `crates/db_views/site/src/api.rs:337-355`

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

**New `SourceDisclosure` struct** (same file, sibling to `GetSiteResponse`) MUST use this exact derive shape — `#[skip_serializing_none]` optional (no `Option` fields, none required), but derive set + ts-rs cfg-attrs are mandatory for codegen parity:

```rust
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// AGPL §13 source-disclosure block. Surfaces the SPDX license, the
/// canonical repository URL, the running fork commit, and the relative
/// URL of the disclosure body. Returned as part of every `GetSiteResponse`
/// so a first-touch client deserializes it on connect.
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

Field add to `GetSiteResponse` (Task 2) — appended after `captcha_enabled`:

```rust
  // ... existing fields ...
  pub captcha_enabled: bool,
  /// AGPL §13 source-disclosure surface; see [`SourceDisclosure`].
  pub source_disclosure: SourceDisclosure,
```

### 10.2 read_site population

**Mirror:** `crates/api/api_crud/src/site/read.rs:20-71`

```rust
pub async fn get_site(
  local_user_view: Option<LocalUserView>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GetSiteResponse>> {
  // This data is independent from the user account so we can cache it across requests
  static CACHE: CacheLock<GetSiteResponse> = LazyLock::new(build_cache);
  let mut site_response = Box::pin(CACHE.try_get_with((), read_site(&context)))
    .await
    .map_err(|e| anyhow::anyhow!("Failed to construct site response: {e}"))?;

  // filter oauth_providers for public access
  if !local_user_view
    .map(|l| l.local_user.admin)
    .unwrap_or_default()
  {
    site_response.admin_oauth_providers = vec![];
  }

  Ok(Json(site_response))
}

async fn read_site(context: &LemmyContext) -> LemmyResult<GetSiteResponse> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let admins = PersonView::list_admins(None, site_view.instance.id, &mut context.pool()).await?;
  // ... existing fields ...

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

Task 2 extends `read_site` to populate `source_disclosure` from compile-time constants. Pattern:

```rust
const BREHON_FORK_COMMIT: &str = env!("BREHON_FORK_COMMIT");
const BREHON_REPO_URL: &str = "https://github.com/barrie-cork/lemmy";
const SOURCE_DISCLOSURE_URL: &str = "/api/v4/source";

// In read_site(), append to the constructor:
  Ok(GetSiteResponse {
    // ... existing fields ...
    captcha_enabled: is_captcha_plugin_loaded(),
    source_disclosure: SourceDisclosure {
      license: "AGPL-3.0".to_string(),
      repo_url: BREHON_REPO_URL.to_string(),
      fork_commit: BREHON_FORK_COMMIT.to_string(),
      disclosure_url: SOURCE_DISCLOSURE_URL.to_string(),
    },
  })
```

### 10.3 Minimal sibling handler

**Mirror:** `crates/api/api/src/site/federated_instances.rs:1-15`

```rust
pub async fn get_federated_instances(
  Query(data): Query<GetFederatedInstances>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<PagedResponse<FederatedInstanceView>>> {
  let federated_instances = FederatedInstanceView::list(&mut context.pool(), data).await?;

  // Return the jwt
  Ok(Json(federated_instances))
}
```

Task 4 mirrors this shape for `get_source` — no DB read (drop the `context` param entirely), no `Query` (no input):

```rust
// crates/api/api/src/site/source.rs
use actix_web::web::Json;
use lemmy_db_views_site::api::GetSourceResponse;
use lemmy_utils::error::LemmyResult;

const AGPL_NOTICE: &str = include_str!("../../../../../AGPL-NOTICE.md");

pub async fn get_source() -> LemmyResult<Json<GetSourceResponse>> {
  Ok(Json(GetSourceResponse {
    notice: AGPL_NOTICE.to_string(),
    license: "AGPL-3.0".to_string(),
  }))
}
```

### 10.4 Site route registration shape

**Mirror:** `crates/api/routes/src/lib.rs:216-230`

```rust
pub fn config(cfg: &mut ServiceConfig, rate_limit: &RateLimit) {
  cfg.service(
    scope("/api/v4")
      .wrap(rate_limit.message())
      // Site
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
      // ... other scopes ...
```

Task 4 adds the new route immediately after the `/site` scope closes — as a sibling to the scope, NOT inside it (matches the precedent `/federated_instances` at line 288):

```rust
      )
      // Site (existing scope above)
      // AGPL §13 source disclosure
      .route("/source", get().to(get_source))
      // ... other scopes continue ...
```

### 10.5 e2e test outer shape (Case A discipline)

**Mirror:** `crates/server/tests/e2e.rs:2077-2090` and `crates/server/tests/e2e.rs:767-809`

```rust
#[tokio::test(flavor = "multi_thread")]
async fn report_to_modlog_golden_path() -> lemmy_utils::error::LemmyResult<()> {
  // Fixture bootstrap — start container, build context
  let (_container, context, _db_url) = governance_fixtures::bootstrap().await?;

  // ... seeded fixtures + handler invocations + assertions ...
}
```

`bootstrap()` signature (verbatim from `crates/server/tests/e2e.rs:767-779`):

```rust
pub async fn bootstrap() -> LemmyResult<(
  testcontainers::ContainerAsync<testcontainers::GenericImage>,
  Data<LemmyContext>,
  String,  // db_url
)>
```

Case A discipline (per `feedback_lemmy_error_no_std_error.md`): test fn returns `LemmyResult<()>`; helpers in `governance_fixtures` returning `Result<T, Box<dyn Error>>` bridge via `.map_err(|e| anyhow::anyhow!("ctx: {e}"))?`.

### 10.6 In-process HTTP via actix_web::test

**Mirror:** `crates/server/tests/e2e.rs:3700-3880` (`all_mvp_endpoints_return_non_404`)

```rust
let app = test::init_service(
  App::new()
    .app_data(context.clone())
    .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit)),
)
.await;

let req = test::TestRequest::get()
  .uri("/api/v4/site")
  .to_request();
let resp = test::call_service(&app, req).await;
let status = resp.status().as_u16();
assert_eq!(status, 200);

let body_bytes = test::read_body(resp).await;
let body: GetSiteResponse = serde_json::from_slice(&body_bytes)?;
```

Task 5 mirrors this verbatim for both `/api/v4/site` and `/api/v4/source` assertions.

### 10.7 build.rs env-injection (no in-workspace precedent; standard pattern)

No existing `crates/*/build.rs` reads `BREHON_FORK_COMMIT`. The standard pattern Rust-side:

```rust
// crates/api/routes/build.rs (NEW — Task 3)
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
  println!("cargo:rerun-if-changed=.git/HEAD");
}
```

`build.rs` lives in `crates/api/routes/` because that crate is the natural integration boundary for the const — the routes lib has no own build.rs, and adding one there avoids polluting `lemmy_db_views_site` (which is a leaf data crate). The `BREHON_FORK_COMMIT` env is re-exported via `pub const BREHON_FORK_COMMIT: &str = env!("BREHON_FORK_COMMIT");` in `crates/api/routes/src/lib.rs` so `lemmy_api_crud::site::read::read_site` reads it through `lemmy_api_routes::BREHON_FORK_COMMIT`.

**Alternative location considered + rejected:** `crates/api/api_crud/build.rs` would put the env closer to the consumer (`read_site`), but `api_crud` is a transitive dep of `api_routes`, so the path is the same. Pick `routes` because the disclosure surface is a routes concern semantically.

## 11. Files to change

Grouped by crate. The planner-side `cargo metadata` check verifies each path exists; new files marked **(new)**.

### `lemmy_db_views_site` (crate: `crates/db_views/site/`)

- `crates/db_views/site/src/api.rs` — add `SourceDisclosure` struct + `GetSiteResponse.source_disclosure` field + `GetSource` (unit struct, for symmetry) + `GetSourceResponse` struct (Task 1, Task 2)

### `lemmy_api_crud` (crate: `crates/api/api_crud/`)

- `crates/api/api_crud/src/site/read.rs` — populate `source_disclosure` in `read_site` (Task 2)

### `lemmy_api` (crate: `crates/api/api/`)

- `crates/api/api/src/site/source.rs` **(new)** — `get_source` handler returning `Json<GetSourceResponse>` with `include_str!("../../../../../AGPL-NOTICE.md")` (Task 4)
- `crates/api/api/src/site/mod.rs` — add `pub mod source;` (Task 4)

### `lemmy_api_routes` (crate: `crates/api/routes/`)

- `crates/api/routes/build.rs` **(new)** — inject `BREHON_FORK_COMMIT` env var via `git rev-parse HEAD` fallback (Task 3)
- `crates/api/routes/src/lib.rs` — register `.route("/source", get().to(get_source))` after the `/site` scope; add `pub const BREHON_FORK_COMMIT: &str = env!("BREHON_FORK_COMMIT");` (Task 3, Task 4)

### `crates/server/tests/` (tests, not a crate proper)

- `crates/server/tests/e2e.rs` — append one new test `agpl_source_disclosure_surface_returns_notice` (Task 5)

### Caller crates (compiles-only-after-Task-2)

Per `feedback_planner_enumerate_struct_callsites_for_addfield.md`. Enumeration of every `GetSiteResponse { ... }` constructor site via `rg "GetSiteResponse \{" crates/`:

| Constructor site | File:line | Crate | After Task 2 needs… |
|---|---|---|---|
| `read_site` constructor | `crates/api/api_crud/src/site/read.rs:64-71` | `lemmy_api_crud` | Add `source_disclosure: SourceDisclosure { ... }` to the literal (covered by Task 2) |

**One constructor site exists** — the only place `GetSiteResponse` is constructed is `read_site`. No other code in the workspace assembles this struct from literal fields (it's deserialized elsewhere, not constructed). Task 2's modify list (`crates/api/api_crud/src/site/read.rs`) covers all callers. The cohort dispatch logic (per `.claude/rules/advisor-orchestrator.md`) needs no extra crate in `modifies:`.

Verification: re-run `rg "GetSiteResponse \{" crates/` after Task 2 commit; expect zero new errors.

### `e2e.rs` constructor-site check (not applicable here)

`GetSiteResponse` is not constructed in `e2e.rs` (only deserialized). No callsite ripple.

## 12. NOT building in v1-ship-1

- **WebAuthn / passkey MFA** — deferred to `v2-security-hardening.prd.md`; ADR-010 v2.
- **Source-tarball or signed-binary release artifact** — deferred to `v2-release-pipeline.prd.md`; PRD §3 explicitly moves this out.
- **Public anchoring of the disclosure URL (Sigstore Rekor / BTC `OP_RETURN`)** — deferred to `v3-verifiability.prd.md`.
- **`AGPL-NOTICE.md` content update / translation / accessibility audit** — out of scope; this plan ships the *surface*, not the *content*. Content edits go through a separate `docs(brehon-fork)` commit independent of v1-ship-1.
- **Per-endpoint OpenAPI registration** — the workspace does not use OpenAPI/utoipa (confirmed by Explore agent); zero codegen surface to update. If OpenAPI adoption happens later, the new route gets registered then.
- **Rate-limit override on `/api/v4/source`** — inherits the global `rate_limit.message()` from the `/api/v4` scope; no per-route override. The notice payload is ~3.5 KB; the default limit is appropriate.
- **`v1-ship-2` (e2e backfill on 4 untested endpoints)** — separate sub-phase; no shared files except `e2e.rs` (additive append).
- **`v1-ship-3` (Postgres pin + `POST /report` reshape + 2-sponsors e2e)** — separate sub-phase.

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task.** Cohort dispatch: Tasks 1 + 3 are `[P]` (DTO + build.rs file-disjoint, both bottom of dep graph); Task 2 depends on Task 1 (field references `SourceDisclosure`); Task 4 depends on Task 3 (handler reads `BREHON_FORK_COMMIT`); Task 5 depends on Tasks 2 + 4 (e2e exercises both surfaces).

**Shape G (Layer G2 push-and-exit) plan.** Cargo invocations belong to `.github/workflows/cargo-validate-workspace.yml` and `.github/workflows/cargo-test-e2e.yml`. impl-task subagents push and exit; ci-watcher mutates the `validate-pending` DQ entry per `.claude/rules/decision-queue.md` "Two-phase validation under Shape G".

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment ready for `v1-ship-1`; branch is `phase-v1-ship-1`; prior phase deliverables intact on base; clippy baseline clean.

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate ALL probes explicitly):**

```bash
# Probe 0 — Docker daemon running (e2e harness needs Postgres container)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — branch sanity
test "$(git branch --show-current)" = "phase-v1-ship-1" || { echo "WRONG BRANCH"; exit 1; }

# Probe 2 — working tree clean
test -z "$(git status --porcelain)" || { echo "WORKING TREE DIRTY"; exit 1; }

# Probe 3 — base sha is at expected governance-v0 HEAD
git merge-base --is-ancestor governance-v0 HEAD && echo "BASE OK" || { echo "BASE DRIFT"; exit 1; }

# Probe 4 — wrapper script honors --workspace --features full (cargo-check.sh exit-code propagation)
bash scripts/brehon/cargo-check.sh --workspace --features full > /tmp/audit-cargo-check.log 2>&1
echo "exit: $?"
tail -20 /tmp/audit-cargo-check.log
# EXPECT: exit 0; baseline clean before any edits

# Probe 5 — wrapper script fails loud on bogus feature (negative test, R5 exit-code propagation)
bash scripts/brehon/cargo-check.sh --workspace --features nonexistent_xyz > /tmp/audit-cargo-check-negative.log 2>&1
echo "exit: $?"
# EXPECT: exit 101 (cargo's error code); MUST NOT be 0

# Probe 6 — clippy baseline clean under --no-deps -- -D warnings
bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > /tmp/audit-clippy.log 2>&1
echo "exit: $?"
tail -20 /tmp/audit-clippy.log
# EXPECT: exit 0; pre-existing clippy debt would block Task 2's clippy DoD

# Probe 7 — concurrent-PR check (no other PR touches §11 files)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("crates/db_views/site/src/api\\.rs|crates/api/api_crud/src/site/read\\.rs|crates/api/routes/src/lib\\.rs|crates/api/api/src/site/mod\\.rs|crates/server/tests/e2e\\.rs")) | {number, title, headRefName}'
# EXPECT: empty output; non-empty → STOP and reconcile (file ownership conflict)

# Probe 8 — AGPL-NOTICE.md exists at repo root (Task 4 include_str! target)
test -f AGPL-NOTICE.md && wc -l AGPL-NOTICE.md || { echo "AGPL-NOTICE.md MISSING"; exit 1; }
# EXPECT: file exists, line count > 0
```

**EXPECT block:**
- Probes 0, 1, 2, 3, 4, 6, 7, 8 exit 0
- Probe 5 exits **non-zero** (typically 101 — confirms exit-code propagation; per `.claude/rules/pre-phase-harness-audit.md` §1.4)

**No commit at Task 0** — verification only.

### Task 1 [P]: Add `SourceDisclosure`, `GetSource`, `GetSourceResponse` DTOs

**ACTION:** in `crates/db_views/site/src/api.rs`, append three new public structs (`SourceDisclosure`, `GetSource`, `GetSourceResponse`) after `GetSiteResponse`. Do NOT yet add the field to `GetSiteResponse` itself (that's Task 2 — keeps the field-add atomic with handler population).

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_views/site/src/api.rs   # append 3 new DTO structs after GetSiteResponse
requires: []
```

**IMPLEMENT (file 1 of 1):** in `crates/db_views/site/src/api.rs` (append after the `GetSiteResponse` definition at line 355). Use the verbatim derives + doc-comments from §10.1:

```rust
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// AGPL §13 source-disclosure block.
pub struct SourceDisclosure {
  pub license: String,
  pub repo_url: String,
  pub fork_commit: String,
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
/// Response for `GET /api/v4/source`. Returns the verbatim AGPL-NOTICE.md
/// body bundled with the SPDX license identifier.
pub struct GetSourceResponse {
  pub notice: String,
  pub license: String,
}
```

**MIRROR:** `crates/db_views/site/src/api.rs:337-355` for the derive shape (already-present `GetSiteResponse` struct).

**GOTCHA:** doc-comments use single-line `///` only; multi-line continuation must indent with a blank line between paragraphs (per `feedback_clippy_doc_lazy_continuation_in_doc_comments.md`). The clippy lint `clippy::doc_lazy_continuation` fires on `/// line1\n///   line2` patterns.

**VALIDATE (story-checkpoint feeds §16a Story 1):**

```bash
# Shape G — push and exit; ci-watcher mutates DQ entry on workflow completion
git add crates/db_views/site/src/api.rs
git commit -m "feat(db_views_site): add SourceDisclosure, GetSource, GetSourceResponse DTOs (task 1)"
git push origin junior/v1-ship-1-task-1
# Capture workflow_run_id; write kind: "validate-pending" DQ entry per Shape G discipline
```

**Expected workflow conclusion:** `cargo-validate-workspace.yml` on `junior/v1-ship-1-task-1` SHA → `conclusion: "success"`.

### Task 2: Add `source_disclosure` field to `GetSiteResponse` + populate in `read_site`

**ACTION:** in `crates/db_views/site/src/api.rs`, add `source_disclosure: SourceDisclosure` field to `GetSiteResponse` after `captcha_enabled`. In `crates/api/api_crud/src/site/read.rs`, populate the field in the `read_site` constructor. Atomic single commit — field add without handler population would break compilation.

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_views/site/src/api.rs              # add source_disclosure field to GetSiteResponse
  - crates/api/api_crud/src/site/read.rs          # populate source_disclosure in read_site
requires:
  - task: 1                                       # SourceDisclosure type must exist
    reason: GetSiteResponse.source_disclosure references SourceDisclosure
  - task: 3                                       # BREHON_FORK_COMMIT must be in scope
    reason: read_site reads BREHON_FORK_COMMIT from lemmy_api_routes
```

**IMPLEMENT (file 1 of 2):** in `crates/db_views/site/src/api.rs`, modify `GetSiteResponse` struct (verbatim shape from §10.1):

```rust
pub struct GetSiteResponse {
  // ... existing fields unchanged ...
  pub captcha_enabled: bool,
  /// AGPL §13 source-disclosure surface; see [`SourceDisclosure`].
  pub source_disclosure: SourceDisclosure,
}
```

**IMPLEMENT (file 2 of 2):** in `crates/api/api_crud/src/site/read.rs` (currently lines 41-71 hold `read_site`). Add at top of file:

```rust
use lemmy_api_routes::BREHON_FORK_COMMIT;
use lemmy_db_views_site::api::SourceDisclosure;

const BREHON_REPO_URL: &str = "https://github.com/barrie-cork/lemmy";
const SOURCE_DISCLOSURE_URL: &str = "/api/v4/source";
```

Modify the `Ok(GetSiteResponse { ... })` constructor at lines 64-71 — append after `captcha_enabled: is_captcha_plugin_loaded(),`:

```rust
    source_disclosure: SourceDisclosure {
      license: "AGPL-3.0".to_string(),
      repo_url: BREHON_REPO_URL.to_string(),
      fork_commit: BREHON_FORK_COMMIT.to_string(),
      disclosure_url: SOURCE_DISCLOSURE_URL.to_string(),
    },
```

**MIRROR:** `crates/api/api_crud/src/site/read.rs:41-71` for the `read_site` constructor shape (verbatim §10.2).

**GOTCHA 1:** `read_site` is wrapped in `LazyLock<CacheLock<GetSiteResponse>>` (lines 26-27 — see `static CACHE: CacheLock<GetSiteResponse> = LazyLock::new(build_cache);`). The cache holds the entire response. The new field is built from compile-time constants → cache value is correct for the instance lifetime. **Do not add any runtime read** (e.g. re-reading the env var per-request) — defeats the cache, and the const value is the contract.

**GOTCHA 2:** if `lemmy_api_routes` is NOT already a dep of `lemmy_api_crud`, the `use lemmy_api_routes::BREHON_FORK_COMMIT;` import will fail. Verify with `cat crates/api/api_crud/Cargo.toml | grep lemmy_api_routes`. If missing: either (a) make the const live in a more central crate (e.g. `lemmy_utils`) or (b) declare it inline in `read.rs` via `const BREHON_FORK_COMMIT: &str = env!("BREHON_FORK_COMMIT");` and arrange `crates/api/api_crud/build.rs` to do the env injection instead. The plan picks option (b) at impl-time if the dep is missing — see Task 3 fallback note. **Pre-impl verification step:** the impl-task subagent runs `cat crates/api/api_crud/Cargo.toml | grep lemmy_api_routes` as the first action of Task 2; if absent, fallback applies.

**GOTCHA 3:** R9 — only `read_site` constructs `GetSiteResponse`. After commit, `rg "GetSiteResponse \{" crates/` must show **only** `crates/api/api_crud/src/site/read.rs:NN` (with the new field present). Zero other constructors.

**VALIDATE (story-checkpoint feeds §16a Story 1):**

```bash
git add crates/db_views/site/src/api.rs crates/api/api_crud/src/site/read.rs
git commit -m "feat(api_crud): wire source_disclosure into GetSiteResponse (task 2)"
git push origin junior/v1-ship-1-task-2
```

**Expected workflow conclusion:** `cargo-validate-workspace.yml` → `success`. Specifically asserts: (a) `lemmy_db_views_site` compiles with the new field, (b) `lemmy_api_crud::site::read` populates it, (c) no orphan constructor breakage.

### Task 3 [P]: Add `build.rs` to `crates/api/routes/` for `BREHON_FORK_COMMIT` env injection

**ACTION:** create `crates/api/routes/build.rs` to inject `BREHON_FORK_COMMIT` via `git rev-parse HEAD` fallback chain. Re-export the constant from `crates/api/routes/src/lib.rs`.

**FILES:**

```yaml
creates:
  - crates/api/routes/build.rs
modifies:
  - crates/api/routes/src/lib.rs   # add `pub const BREHON_FORK_COMMIT: &str = env!("BREHON_FORK_COMMIT");`
  - crates/api/routes/Cargo.toml   # add `build = "build.rs"` to [package] section (if needed)
requires: []
```

**IMPLEMENT (file 1 of 3):** create `crates/api/routes/build.rs`:

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
  println!("cargo:rerun-if-changed=.git/HEAD");
}
```

**IMPLEMENT (file 2 of 3):** in `crates/api/routes/src/lib.rs`, add near the top (after the existing imports):

```rust
/// HEAD commit SHA of the running fork build; "unknown" if unavailable at build time.
///
/// Surfaced via `GetSiteResponse.source_disclosure.fork_commit` for AGPL §13 disclosure.
pub const BREHON_FORK_COMMIT: &str = env!("BREHON_FORK_COMMIT");
```

**IMPLEMENT (file 3 of 3):** in `crates/api/routes/Cargo.toml`, verify the `[package]` section names the build script. Cargo defaults to `build.rs` if a file at that path exists — no edit needed unless the section sets `build = "..."` to a different path. Pre-impl verification: `grep -A2 '^\[package\]' crates/api/routes/Cargo.toml | grep -E '^build'`. If a `build = ...` line exists pointing elsewhere, raise a `kind: "blocker"` DQ. **Most likely no edit needed.**

**MIRROR:** §10.7 (no in-workspace precedent for env injection; pattern is standard Rust build-script idiom).

**GOTCHA 1:** if the deps graph blocks `lemmy_api_crud → lemmy_api_routes` (Task 2 GOTCHA 2 fallback), Task 3 ALSO adds an inline `const BREHON_FORK_COMMIT: &str = env!("BREHON_FORK_COMMIT");` to `crates/api/api_crud/src/site/read.rs` directly, AND adds `crates/api/api_crud/build.rs` mirroring the routes build.rs. The fallback path is mechanical; the impl-task subagent checks the dep graph at task start and picks the path. Both paths converge on the same final shape (`BREHON_FORK_COMMIT` is a `&'static str` const reachable from `read_site`).

**GOTCHA 2:** the `build.rs` runs on the build host, not the runtime host. `BREHON_FORK_COMMIT` is therefore the build-time commit, not the running-process commit. This is correct for AGPL §13 — the disclosure binds the binary's provenance, not its runtime identity.

**GOTCHA 3:** `git rev-parse HEAD` will fail loudly in environments where `.git/` is absent (e.g. published source tarballs that don't ship the git directory). The fallback chain correctly returns `"unknown"` in that case. CI environments that set `BREHON_FORK_COMMIT` env explicitly bypass the git call.

**VALIDATE (story-checkpoint feeds §16a Story 2):**

```bash
git add crates/api/routes/build.rs crates/api/routes/src/lib.rs crates/api/routes/Cargo.toml
git commit -m "feat(routes): inject BREHON_FORK_COMMIT via build.rs (task 3)"
git push origin junior/v1-ship-1-task-3
```

**Expected workflow conclusion:** `cargo-validate-workspace.yml` → `success`. Build emits `cargo:rustc-env=BREHON_FORK_COMMIT=<sha>`; the const compiles.

### Task 4: Add `get_source` handler + route registration

**ACTION:** create `crates/api/api/src/site/source.rs` with the `get_source` handler. Add `pub mod source;` to `crates/api/api/src/site/mod.rs`. Register `.route("/source", get().to(get_source))` in `crates/api/routes/src/lib.rs` after the `/site` scope.

**FILES:**

```yaml
creates:
  - crates/api/api/src/site/source.rs
modifies:
  - crates/api/api/src/site/mod.rs        # add `pub mod source;`
  - crates/api/routes/src/lib.rs          # register the new route
requires:
  - task: 1
    reason: handler returns Json<GetSourceResponse>; the type must exist
```

**IMPLEMENT (file 1 of 3):** create `crates/api/api/src/site/source.rs`:

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

**IMPLEMENT (file 2 of 3):** in `crates/api/api/src/site/mod.rs`, append `pub mod source;` alongside the existing `pub mod federated_instances;` etc. declarations.

**IMPLEMENT (file 3 of 3):** in `crates/api/routes/src/lib.rs`, locate the `/site` scope at line 222-230 (per §10.4). Immediately after the closing `)` of `.service(scope("/site"))`, add:

```rust
        // AGPL §13 source disclosure — public, no auth, no DB
        .route("/source", get().to(get_source))
```

Also add the import at the top of the file alongside other `use lemmy_api::site::...` imports:

```rust
use lemmy_api::site::source::get_source;
```

**MIRROR:** §10.3 (`federated_instances.rs` minimal handler) for handler shape; §10.4 (site scope) for route registration.

**GOTCHA 1:** the `include_str!` path is **relative to the source file**, not the crate root. From `crates/api/api/src/site/source.rs`, the path `../../../../../AGPL-NOTICE.md` resolves: up `site/` → `src/` → `api/` (the inner crate) → `api/` (the parent dir under `crates/api/`) → `crates/` → repo root. Verify by counting: `src/site/source.rs` → up 2 → crate root (`crates/api/api/`) → up 3 more → repo root. Total: 5 `..` segments. Confirmed by Explore agent for sibling handler paths.

**GOTCHA 2:** `include_str!` is a compile-time macro. If `AGPL-NOTICE.md` is missing or unreadable at build time, the build fails loudly with `error: couldn't read AGPL-NOTICE.md`. This is the desired behaviour (per PRD §7.1 risk table: "file move breaks the build loudly").

**GOTCHA 3:** the route is NOT inside the `/site` scope — it's at the `/api/v4` scope level (verify against §10.4). Putting it inside `/site` would resolve to `/api/v4/site/source`, which the PRD §7.1 ship-criteria contradicts (the path is `/api/v4/source`). Read the line context carefully before placing the `.route(...)`.

**GOTCHA 4:** the handler takes **no parameters**. No `context: Data<LemmyContext>`. No `Query<...>`. Adding either is correct-but-wasteful (no DB read, no input). Keep it minimal.

**GOTCHA 5:** if `lemmy_api_routes` does not already import `get` (the actix-web method shortcut for `web::get()`), confirm the existing `use actix_web::web::{delete, get, post, put, scope, ServiceConfig};` line at the top of `lib.rs` includes it. The §10.4 snippet confirms `get()` is in scope.

**VALIDATE (story-checkpoint feeds §16a Story 2):**

```bash
git add crates/api/api/src/site/source.rs crates/api/api/src/site/mod.rs crates/api/routes/src/lib.rs
git commit -m "feat(api): add get_source handler + /api/v4/source route (task 4)"
git push origin junior/v1-ship-1-task-4
```

**Expected workflow conclusion:** `cargo-validate-workspace.yml` → `success`. After workspace check passes, the daemon finalize-merges this branch into `phase-v1-ship-1`; the phase-branch push triggers `cargo-test-e2e.yml` (Phase 2 e2e), but Task 5 hasn't landed yet → e2e runs the existing 11 tests + zero new (acceptable; baseline).

### Task 5: Add e2e test `agpl_source_disclosure_surface_returns_notice`

**ACTION:** append a new test to `crates/server/tests/e2e.rs` exercising both `/api/v4/site` (asserting `source_disclosure` field shape) and `/api/v4/source` (asserting notice body present).

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append agpl_source_disclosure_surface_returns_notice test (~80 lines)
requires:
  - task: 2
    reason: GetSiteResponse must carry source_disclosure for the assertion to deserialize
  - task: 4
    reason: /api/v4/source route must be registered for the second GET to return 200
```

**IMPLEMENT (file 1 of 1):** append after the last existing test in `crates/server/tests/e2e.rs`. Use the Case A discipline (per §10.5) and the in-process HTTP pattern (per §10.6):

```rust
#[tokio::test(flavor = "multi_thread")]
async fn agpl_source_disclosure_surface_returns_notice() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::{App, test};
  use lemmy_api_utils::rate_limit::RateLimit;
  use lemmy_db_views_site::api::{GetSiteResponse, GetSourceResponse};

  let (_container, context, _db_url) = governance_fixtures::bootstrap().await?;

  let rate_limit = RateLimit::with_debug_config();
  let app = test::init_service(
    App::new()
      .app_data(context.clone())
      .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit)),
  )
  .await;

  // --- 1. GET /api/v4/site returns source_disclosure block. ---
  let site_req = test::TestRequest::get().uri("/api/v4/site").to_request();
  let site_resp = test::call_service(&app, site_req).await;
  assert_eq!(site_resp.status().as_u16(), 200, "/api/v4/site must return 200");

  let site_body_bytes = test::read_body(site_resp).await;
  let site_body: GetSiteResponse = serde_json::from_slice(&site_body_bytes)
    .map_err(|e| anyhow::anyhow!("deserialize GetSiteResponse: {e}"))?;

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
  let source_body: GetSourceResponse = serde_json::from_slice(&source_body_bytes)
    .map_err(|e| anyhow::anyhow!("deserialize GetSourceResponse: {e}"))?;

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

**MIRROR:** §10.5 (`report_to_modlog_golden_path` outer shape + `bootstrap()` usage); §10.6 (in-process HTTP via `actix_web::test::TestRequest` + `test::call_service`).

**GOTCHA 1:** R8 — outer return is `LemmyResult<()>` (Case A per `feedback_lemmy_error_no_std_error.md`). `bootstrap()` and `serde_json::from_slice` errors bridge via `.map_err(|e| anyhow::anyhow!("ctx: {e}"))?`. Do NOT use `Box<dyn Error>` outer — sibling tests use `LemmyResult<()>` and the canonical case enumeration in `lemmy_error_no_std_error.md` lists this as Case A.

**GOTCHA 2:** the new test imports must use the `use` block **inside the test fn** (matching the in-file convention — e2e.rs sibling tests inline imports per-test to reduce top-of-file churn). Do NOT add to the file-level `use` block.

**GOTCHA 3:** `RateLimit::with_debug_config()` already exists (per `crates/server/tests/e2e.rs` confirmed by Explore agent — used in `report_to_modlog_golden_path` line 2160 region). No new fixture helper required.

**GOTCHA 4:** test name MUST be `agpl_source_disclosure_surface_returns_notice` (verbatim per PRD §7.1 ship-criteria and §16a Story 3). The verify pass (`/brehon-verify`) greps for this literal name in `e2e.rs`.

**GOTCHA 5:** per `feedback_junior_worker_e2e_edit_hang.md`, the append is ~70 lines — well under the 300-line hang threshold. Single-edit-per-commit is the discipline; the impl-task subagent uses a single `Edit` operation appending to the file's end.

**VALIDATE (story-checkpoint feeds §16a Story 3):**

```bash
git add crates/server/tests/e2e.rs
git commit -m "test(e2e): assert AGPL §13 disclosure surface via /api/v4/site + /api/v4/source (task 5)"
git push origin junior/v1-ship-1-task-5
```

**Expected workflow conclusion:** `cargo-validate-workspace.yml` → `success` (compile of the new test). After daemon finalize-merge, `cargo-test-e2e.yml` runs on `phase-v1-ship-1` tip → expects the new test in the run output + 0 failures.

### Task 6: Retro

**Goal:** author retro per `feedback_retro_not_report.md` and `feedback_four_role_retro_signals.md`. One H2 per role (Advisor / Planning / Impl / BM) with signals + lessons. Promote any new lessons to `.claude/lessons/feedback_*.md` in the same retro commit (per `feedback_one_system_memory_in_repo.md`).

**FILES:**

```yaml
creates:
  - .claude/PRPs/retros/v1-ship-1-retro.md
  - .claude/lessons/feedback_*.md    # zero or more new lessons; promoted from retro signals
modifies:
  - .claude/lessons/MEMORY.md         # if a new lesson is authored, index it here
requires:
  - task: 5
    reason: retro reflects on the full impl arc + the e2e green signal
```

**Three required H2 sections** (per `feedback_retro_not_report.md`):

```markdown
# v1-ship-1 retro

## What surprised us
- (per-role signals; one bullet per role at minimum)

## What to change
- (concrete deltas: rule edits, lesson promotions, brief-template tweaks)

## What to carry forward
- (patterns + decisions that worked; cite prior-phase recurrence if applicable)
```

**No commit at retro draft** — the BM subagent commits the retro after user sign-off (per `.claude/rules/advisor-orchestrator.md` user gate 6).

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** `cargo check --workspace --features full` — via `cargo-validate-workspace.yml` after every push (Shape G).
- **Lint:** `cargo clippy --workspace --features full --no-deps -- -D warnings` — same workflow.
- **Test target compile:** `cargo test --no-run --workspace --features full --test e2e` — same workflow.
- **e2e execution:** `cargo test --workspace --features full --test e2e agpl_source_disclosure_surface_returns_notice` — via `cargo-test-e2e.yml` on `phase-v1-ship-1` tip after daemon finalize-merge.
- **Migration round-trip:** N/A — no schema changes.

User-gate 4 (Phase 2 e2e — local vs dispatch) fires when `phase-v1-ship-1` tip reaches `cargo-test-e2e.yml` triggering condition. Default per `feedback_default_local_testing.md`: laptop-local `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1"`. Per `feedback_laptop_default_for_validate_pending.md` + `feedback_windows_e2e_requires_bat_wrapper.md`.

---

## 15. Validation commands (DoD)

> Shape G plan. Commands here are the canonical shape; impl-task subagents push and exit; ci-watcher mutates DQ entries from workflow runs.

### 15.1 Static analysis (per task — workspace check)

Workflow: `.github/workflows/cargo-validate-workspace.yml` on `junior/v1-ship-1-task-N`.
Internal command:
```bash
cargo check --workspace --features full
```
EXPECT: workflow conclusion `success`.

### 15.2 Lint (per task — uniform R6)

Same workflow runs:
```bash
cargo clippy --workspace --features full --no-deps -- -D warnings
```
EXPECT: workflow conclusion `success`. Zero new warnings introduced.

### 15.3 Test target compile (R7 — Tasks 1, 2, 4, 5 touch structs or re-exports)

Same workflow runs:
```bash
cargo test --workspace --features full --test e2e --no-run
```
EXPECT: workflow conclusion `success`.

### 15.4 e2e test execution (Task 5)

Workflow: `.github/workflows/cargo-test-e2e.yml` on `phase-v1-ship-1` (post-finalize-merge of Task 5).
Internal command:
```bash
cargo test --workspace --features full --test e2e agpl_source_disclosure_surface_returns_notice
```
EXPECT: workflow conclusion `success`; test name appears in output; 1 passed; 0 failed. The full e2e suite ALSO runs (no `-- --test-threads=1` constraint inherent; flake-tolerant since the new test fixture is independent).

### 15.5 Cross-cutting verification

- [ ] No file outside §11 list edited (verify via `git diff --stat governance-v0...HEAD` enumerating only the §11 paths)
- [ ] R1: no `as` casts introduced for int comparisons (verify: `rg "as i64|as i32" crates/api/api_crud/src/site/read.rs crates/db_views/site/src/api.rs` returns empty)
- [ ] R5: Task 0 enumerated all probes
- [ ] R6: all clippy invocations use `--no-deps` uniformly (`cargo-validate-workspace.yml` confirms)
- [ ] R7: test-target compile runs after each struct-touching task (workflow runs it on every push)
- [ ] R8: test fn outer return is `LemmyResult<()>` (verify in `crates/server/tests/e2e.rs` for the new test)
- [ ] R9: `rg "GetSiteResponse \{" crates/` returns only `crates/api/api_crud/src/site/read.rs` and the new field is present
- [ ] Doc-comments on `SourceDisclosure`, `GetSource`, `GetSourceResponse` pass `clippy::doc_lazy_continuation`
- [ ] `AGPL-NOTICE.md` byte-equal to the build-time `include_str!` target (verify: `wc -c AGPL-NOTICE.md` matches expected ~3595)
- [ ] `BREHON_FORK_COMMIT` env injection: `cargo build --workspace --features full && strings target/debug/lemmy_server | grep BREHON_FORK_COMMIT` returns a non-`unknown` SHA on dev machines with a `.git/` directory (acceptance: any non-empty string at runtime; `"unknown"` acceptable for source-tarball builds per Task 3 GOTCHA 3)

### 15.6 DoD per workflow (Shape G)

- **DoD entry**: `cargo-validate-workspace.yml` on `junior/v1-ship-1-task-N` SHA `<sha>` → `conclusion: "success"`. Required for every Task 1, 2, 3, 4, 5 push.
- **DoD entry**: `cargo-test-e2e.yml` on `phase-v1-ship-1` SHA `<post-finalize-merge-sha>` → `conclusion: "success"`. Required after Task 5 finalize-merge.
- **Validation command**: `gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --json conclusion,databaseId --jq '.[0]'`
- **EXPECT**: `{"conclusion": "success", "databaseId": <id>}`

---

## 16. Acceptance criteria

- [ ] All 6 tasks completed in dependency order (Task 0 audit, Tasks 1-5 impl, Task 6 retro)
- [ ] §15.1 (cargo check workspace) exit 0 after every task (workflow `success`)
- [ ] §15.2 (cargo clippy `--no-deps -- -D warnings`) exit 0 after every task
- [ ] §15.3 (cargo test --no-run) exit 0 after Tasks 1, 2, 4, 5
- [ ] §15.4 (e2e tests) — `agpl_source_disclosure_surface_returns_notice` passes; pre-existing tests still pass
- [ ] §15.5 (cross-cutting verification) — all 10 boxes ticked
- [ ] §16a stories — all 3 stories `[done]`
- [ ] No edits to files outside §11 list
- [ ] Retro committed per §13 Task 6
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-ship-1-verify.md` shows all stories ✓
- [ ] Manual smoke: `curl http://localhost:8536/api/v4/site | jq .source_disclosure` returns the four-field block; `curl http://localhost:8536/api/v4/source | jq .notice | head` returns notice text. Verifiable by an external observer with no project context (PRD §7.1 ship-criteria).

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: First-touch handshake exposes source-disclosure pointer

- **Composing tasks:** Task 1 (DTOs), Task 2 (field + handler populate), Task 3 (build.rs env injection)
- **Checkpoint command:** `bash scripts/brehon/cargo-test.sh --workspace --features full --test e2e agpl_source_disclosure_surface_returns_notice -- --nocapture | grep "source_disclosure.license"` (the test asserts `source_disclosure.license == "AGPL-3.0"` and `source_disclosure.disclosure_url == "/api/v4/source"`)
- **Expected output:** `1 passed; 0 failed`; asserts pass
- **Brief-Scope outputs to verify** (used by `/brehon-verify`):
  - `crates/db_views/site/src/api.rs` contains `pub struct SourceDisclosure`
  - `crates/db_views/site/src/api.rs` contains `pub source_disclosure: SourceDisclosure` inside `GetSiteResponse`
  - `crates/api/api_crud/src/site/read.rs` constructor literal contains `source_disclosure: SourceDisclosure {`
  - `crates/api/routes/build.rs` exists + contains `cargo:rustc-env=BREHON_FORK_COMMIT`
  - `crates/api/routes/src/lib.rs` contains `pub const BREHON_FORK_COMMIT: &str = env!("BREHON_FORK_COMMIT");`

### Story 2: Disclosure URL resolves to the AGPL notice body

- **Composing tasks:** Task 4 (handler + route + module wiring)
- **Checkpoint command:** `bash scripts/brehon/cargo-test.sh --workspace --features full --test e2e agpl_source_disclosure_surface_returns_notice -- --nocapture | grep "GNU Affero"` (the test asserts `notice.contains("GNU Affero General Public License")`)
- **Expected output:** `1 passed; 0 failed`; assert passes
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/site/source.rs` exists + contains `pub async fn get_source`
  - `crates/api/api/src/site/source.rs` contains `include_str!("../../../../../AGPL-NOTICE.md")`
  - `crates/api/api/src/site/mod.rs` contains `pub mod source;`
  - `crates/api/routes/src/lib.rs` contains `.route("/source", get().to(get_source))`
  - `crates/api/routes/src/lib.rs` contains `use lemmy_api::site::source::get_source;`

### Story 3: Named e2e test covers both surfaces with Case A discipline

- **Composing tasks:** Task 5
- **Checkpoint command:** `rg -n "async fn agpl_source_disclosure_surface_returns_notice" crates/server/tests/e2e.rs`
- **Expected output:** one match, returning `LemmyResult<()>`
- **Brief-Scope outputs to verify:**
  - `crates/server/tests/e2e.rs` contains `async fn agpl_source_disclosure_surface_returns_notice() -> lemmy_utils::error::LemmyResult<()>`
  - Test body contains BOTH `test::TestRequest::get().uri("/api/v4/site")` AND `test::TestRequest::get().uri("/api/v4/source")` calls
  - Test uses `governance_fixtures::bootstrap()` (no fresh fixture invented)

---

## 17. Completion checklist

- [ ] Task 0 audit complete (all 9 probes confirmed; Probe 5 returns non-zero)
- [ ] Task 1..5 committed (one commit per task per `feedback_pr_per_phase.md`)
- [ ] §15 validation green at every gate (5 workflow runs `success`)
- [ ] §16a stories all `[done]` (3 stories)
- [ ] Retro committed per Task 6
- [ ] PR opened by BM session against `governance-v0`
- [ ] CodeRabbit review complete; findings triaged
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-ship-1-verify.md` shows all 3 stories ✓
- [ ] Post-merge `phase-v1-ship-1` branch retained for retro reads

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `lemmy_api_routes` not in `lemmy_api_crud`'s dep graph → `use lemmy_api_routes::BREHON_FORK_COMMIT` fails | MED | LOW | Fallback path in Task 3 GOTCHA 1: inline the const in `read.rs` + add `crates/api/api_crud/build.rs`. Impl-task verifies dep graph at task start (`cat Cargo.toml | grep lemmy_api_routes`). Same end-state; mechanical path-switch. |
| `BREHON_FORK_COMMIT` env empty in CI without `.git/` (e.g. sparse checkout) | LOW | LOW | Fallback chain in `build.rs` returns `"unknown"` non-fatally; AGPL §13 still complies (the notice URL is the load-bearing surface, not the SHA) |
| `include_str!` path miscount (5 vs 4 `..` segments) | MED | LOW | Probe 8 in Task 0 confirms `AGPL-NOTICE.md` presence; if path is wrong, build fails loudly (compile-time error names the path). Impl-task re-counts segments before commit. |
| `ts-rs` codegen drift on new fields (`GetSiteResponse.source_disclosure`) | LOW | LOW | `ts(optional_fields, export)` derive is consistent with sibling fields; field-add is additive in TS; no required-field collision |
| `read_site` cache holds stale `source_disclosure` on hot reload | NEGLIGIBLE | NEGLIGIBLE | Values are compile-time constants; instance lifetime cache is correct |
| Lemmy upstream rebase changes `GetSiteResponse` shape | MED | LOW | Field addition is additive; rebase conflicts resolve by re-adding the field at the new line number; the `read_site` constructor patch is mechanical |
| e2e flake from `bootstrap()` testcontainers cold-start (~30s) | LOW | LOW | Existing test pattern survives; the new test uses the same `bootstrap()` helper as `report_to_modlog_golden_path` |
| `clippy::doc_lazy_continuation` fires on multi-line doc-comments | MED | LOW | All `///` doc-comments in plan §10.1 are single-line OR have blank `///` between paragraphs; impl-task copies verbatim |
| AGPL-NOTICE.md path moves (e.g. `docs/AGPL-NOTICE.md`) post-impl | LOW | LOW | `include_str!` is compile-time; any path drift breaks the build immediately. Probe 8 in Task 0 confirms current location. |
| The 5-segment `../../../../../AGPL-NOTICE.md` is fragile to file moves | LOW | LOW | Acceptable trade-off; the alternative (read at runtime via Settings) defeats the goal of compile-time inclusion (zero runtime path-resolution risk) |

---

## 19. Notes

- **AGPL §13 substance vs surface.** This plan ships the *surface* (HTTP endpoint, DTO field). The *substance* (the actual `AGPL-NOTICE.md` text) is already authored; this plan does not edit it. If the legal text needs revision, that's a separate `docs(brehon-fork)` commit — independent of this plan and not part of v1-ship-1 acceptance.
- **Why `disclosure_url` is relative, not absolute.** The fork instance doesn't know its own public hostname at build time; the relative URL lets clients resolve against the server they're already talking to. The same reasoning applies to other Lemmy URL fields.
- **Why no auth on `/api/v4/source`.** ADR-011 requires source disclosure to all users interacting over the network. Auth-gating would defeat the §13 surface. The endpoint is read-only, payload is ~3.5 KB, default rate-limit (`rate_limit.message()`) is appropriate.
- **Why include the SHA in the disclosure.** AGPL §13 binds the *running version* of the modified code. The SHA identifies the build precisely so an external auditor can fetch the exact source. `"unknown"` fallback is acceptable for source-tarball builds where `.git/` is absent — the auditor still has the repo URL.
- **Decision-queue pre-seed (planner → advisor):** none. The plan resolves the disclosure-surface shape via PRD §11 (BOTH site field AND dedicated endpoint, decided at PRD authorship). No open clarify entries.

---

## 20. Confidence score

- **Plan correctness:** 9/10 — every file:line citation comes from a verified Explore agent read against current HEAD; mechanical Lemmy patterns; ADR-011 is the load-bearing constraint and this is its surfacing. One open question: `lemmy_api_routes` ∈ `lemmy_api_crud` dep graph — fallback path documented; impl-task verifies at task start.
- **Cargo budget:** 10/10 — Shape G; validation off-box; zero EliteDesk impact.
- **Test coverage:** 9/10 — one new e2e exercises both surfaces; missing surface (e.g. unauthenticated regression on `/api/v4/site` separately) is not in scope since the existing all-mvp test at line 3700 already covers non-404. PRD §7.1 ship-criteria fully exercised by the new test.

---

*Planned: 2026-05-14*
*Status: DRAFT — pending advisor §3.4 DoD smoke test, §3.5 watchpoint specificity gate, and user gate 1 (plan approval).*
