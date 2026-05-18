# Plan: v1-ship-1-r2 — rebuild AGPL §13 e2e harness on the canonical FederationConfig idiom

> **RE-PLAN (second).** Design, surface, and §13 Tasks 1-4 are **carried forward verbatim** from the merged `v1-ship-1-r1.plan.md` — `source_disclosure` DTOs, `read_site` population, `crates/api/api_crud/build.rs` env injection, the `get_source` handler + route are on `phase-v1-ship-1` and PROVEN CORRECT (every §5.2 Phase-1 PASS — cargo check + clippy `-D warnings` + cargo-test `--no-run`). The single residual defect is the App-construction inside the one acceptance e2e test `agpl_source_disclosure_surface_returns_notice` at `crates/server/tests/e2e.rs:14865-14981` — 5 successive attempts (the original e2e task → fix-impl-5 → fix-impl-6 → fix-impl-7 → fix-impl-7b → fix-impl-8) all returned HTTP 500 *"Requested application data is not configured correctly."* on `/api/v4/site`. This plan rebuilds **only** that test's App-construction on the canonical real-server idiom (single source-of-truth `FederationConfig` → context-out-of-config via `federation_config.deref().clone()` per `lib.rs:364`), keeps fix-impl-6 Part A (body-on-failure asserts) + Part B (complete `SiteInsertForm` seed) verbatim, and changes nothing else.
>
> **What I verified at current tip (worker branch off `phase-v1-ship-1` @ `92c4cc190`).** Every `file:line` / API in this plan was confirmed by direct `Read`: `crates/api/api_utils/src/context.rs:1-101` (`init_test_federation_config` + `init_test_context`); `crates/server/src/lib.rs:225-390` (real-server federation composition); `crates/api/api_crud/src/site/read.rs:1-82` (current merged `get_site` + `read_site` shape with `source_disclosure` populated); `crates/server/tests/e2e.rs:118` (`mod governance_fixtures` + `use actix_web::web::Data;` at line 119), `:801` (`pub async fn bootstrap()` chosen), `:5665` (sibling `admin_config_fixtures::bootstrap` — REJECTED per DQ #226), `:14865-14981` (the failed test region — the single Edit target); `crates/diesel_utils/src/connection.rs:160-199` (`build_db_pool()` + `build_db_pool_for_tests()` — reads `LEMMY_DATABASE_URL`); `~/.cargo/registry/src/.../activitypub_federation-0.7.0-beta.11/src/{config.rs:143,335-398,264-270,actix_web/middleware.rs:1-77}` (`to_request_data` signature, `Data<T>` definition + `Deref<Target=T>`, `FederationMiddleware::call`).
>
> **The central finding the brief's investigation MISSED (filed as planner DQ #261 — `kind: "log"`, advisor-validate at plan approval).** Brief §0.1 Finding 3 + Option (b) directs the e2e to obtain its `Data<LemmyContext>` for actix `.app_data(...)` via `federation_config.to_request_data()`. **At current tip this is mechanically incompatible with the merged `get_site` signature** — see §10.7 for the source-cited derivation. The verified-correct mechanism that honors the brief's spirit (mirror lib.rs federation-config-derived single context source) is **`let inner_context: LemmyContext = federation_config.deref().clone();` then `.app_data(Data::new(inner_context.clone()))`** — the line-for-line mirror of production `lib.rs:364` + `lib.rs:379`. The plan adopts this mechanism; §10.7 + §18 + §19 cite the source evidence; the advisor validates at user gate 1.

## 1. Summary

Replaces the failing App-construction inside the single e2e `agpl_source_disclosure_surface_returns_notice` (at `crates/server/tests/e2e.rs:14865-14981`) with the byte-for-byte mirror of the real server's `create_http_server` composition (`crates/server/src/lib.rs:228-241/364/379-382`): build `FederationConfig` from the bootstrap context; derive the App's raw `LemmyContext` from `federation_config.deref().clone()`; wrap actix-Data + FederationMiddleware + IdempotencyMiddleware + SessionMiddleware in production order. Preserves fix-impl-6 Part A (body-on-failure asserts) + Part B (complete `SiteInsertForm` / sysacct / LocalSite / LocalSiteRateLimit seed) verbatim. Closes the AGPL §13 ship-gate: `phase-v1-ship-1` reaches Phase-2 e2e green and the fork can compliantly accept its first external user.

## 2. Source

- `.claude/PRPs/briefs/v1-ship-1-r2-planning-1.md` @ `92c4cc190` — the brief authorising this re-plan. Investigation findings §0.1, scope-fence §0.0, and clarify-pass resolutions §0.1.2 (DQ #257-#260) are binding inputs.
- `.claude/PRPs/plans/v1-ship-1.plan.md` (parked) + `.claude/PRPs/plans/v1-ship-1-r1.plan.md` (refreshed) — design + §13 Tasks 1-4 carry-forward; both retained as audit trail (per `.claude/rules/no-destructive-defaults.md`).
- `.claude/PRPs/prds/v1-ship-readiness.prd.md` §7.1 (AGPL §13 acceptance — `source_disclosure` block on `GetSiteResponse` + `GET /api/v4/source` body).
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — **ADR-011** (AGPLv3 inherited + source-disclosure required; load-bearing).
- **Canonical sibling plan (per `feedback_read_canonical_before_writing_spec.md`):** `v1-ship-1-r1.plan.md` (same 20-section schema; this re-plan mirrors its shape with one structural variant — Tasks 1-4 are carried as already-shipped context, not re-executed).
- **DQ resolutions binding this plan:**
  - DQ #257 (`build_db_pool_for_tests()` at `crates/diesel_utils/src/connection.rs:197` connects to `LEMMY_DATABASE_URL`; `governance_fixtures::bootstrap()` at `e2e.rs:814` sets that env BEFORE calling `build_db_pool_for_tests()` at `:822`).
  - DQ #258 (`FederationConfig::to_request_data(&self) -> Data<T>` at `config.rs:143` BORROWS `&self` and clones the config internally; `FederationConfig<T>: Deref<Target=T>` at `config.rs:264-270`).
  - DQ #259 (§15 Phase-1 cmd3 `cargo-test.bat --no-run -p lemmy_server --test e2e` PROVEN — DQ #254 + DQ #250; do NOT normalize to `--workspace --features full` per `feedback_features_full_p_crate_incompatible.md`).
  - DQ #260 (no cross-lane file-ownership conflict on `crates/server/tests/e2e.rs`; the §13 e2e task has sole ownership).
- **Planner-raised DQ:**
  - **DQ #261** (`kind: "log"`, `from: "planner"`) — type-precision: `to_request_data()` returns `activitypub_federation::config::Data<T>`, NOT `actix_web::web::Data<T>` extracted by `get_site`. The brief's Option (b) literal recipe is mechanically incompatible; the verified-correct mechanism (mirroring `lib.rs:364`) is `federation_config.deref().clone()`. See §10.7 + §19.
- **Lessons binding decisions** (every cited lesson actually shaped the plan body — none "for awareness only"):
  - `feedback_read_canonical_before_writing_spec.md` — mirror the existing canonical `lib.rs` composition, NOT a new hand-assembly.
  - `feedback_lemmy_error_no_std_error.md` — Case A discipline (`LemmyResult<()>` outer, bare `?` propagation); the failed test already uses Case A correctly.
  - `feedback_junior_worker_e2e_edit_hang.md` — `e2e.rs` is ~14,981 lines; single targeted anchored Edit replacing the failed fn region (lines 14865-14981), NEVER full-file Edit.
  - `feedback_async_pool_test_pattern.md` — pool/conn/LemmyResult fixture discipline (Case A confirms).
  - `feedback_advisor_watchpoint_specificity.md` — every §4 watchpoint cites a specific file:line at current tip.
  - `feedback_plan_dod_dry_run_at_write.md` + `feedback_pre_phase_dod_smoke_test.md` — every §15 command executable as-written.
  - `feedback_features_full_p_crate_incompatible.md` — §15 Phase-1 cmd3 uses `-p lemmy_server --test e2e` (NOT `--features full`).
  - `feedback_features_full_workspace_only.md` — §15 Phase-2 e2e uses `--workspace --features full`.
  - `feedback_windows_e2e_requires_bat_wrapper.md` — local e2e via `cmd //c scripts\brehon\cargo-test.bat ...`, NEVER bare `cargo test`.
  - `feedback_laptop_default_for_validate_pending.md` — §5.2-laptop shape (Shape G suspended per DQ #229 until 2026-06-01).
  - `feedback_verify_files_with_read.md` (pattern) — every `file:line` confirmed by `Read`, not just `grep`.
  - `feedback_plan_baseline_self_reference.md` — MIRROR refs cite symbol + line so the plan self-heals.
  - `feedback_complexity_score_pre_split.md` — §5.1 breakdown table.
  - `feedback_principles_not_rules.md` — the brief's Option (a)/(b) is a *prior*, not a forced answer; planner decides on source evidence (the DQ #261 mechanism-precision call).
  - `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` — Task 5 retro shape.

## 3. Problem statement

The AGPL §13 surface (DTOs + field-add + `build.rs` env injection + `get_source` handler + route) is MERGED on `phase-v1-ship-1` and proven type-correct (every Phase-1 cargo check / clippy / test --no-run green at tip `92c4cc190`). But the **single acceptance e2e test** (`agpl_source_disclosure_surface_returns_notice` at `e2e.rs:14865`) returns HTTP 500 *"Requested application data is not configured correctly. View/enable debug logs for more details."* on `GET /api/v4/site`. That test is the PRD §7.1 acceptance gate — without it green, `phase-v1-ship-1` cannot ship and the fork is out of AGPL §13 compliance the moment it accepts a first external user (ADR-011).

The 5 fix-impl attempts all hand-assembled the App with progressively more middleware wraps (fix-impl-7 = +SessionMiddleware; fix-impl-7b = +`(**context).clone()` deref for the SessionMiddleware arg; fix-impl-8 = +FederationMiddleware + IdempotencyMiddleware + SessionMiddleware mirroring `lib.rs:380-382`). All produced the byte-identical 500. The §G4 hard refusal fired after fix-impl-8 (2nd same-tuple `(actix-Data-500, e2e.rs)` post-allowlist-recipe-exhaustion); the user directed (DQ #256) advisor-investigation then re-plan.

Tied to §13 Task 6 (this plan's e2e rebuild) — Tasks 1-4 are merged.

## 4. Solution statement

Single principle: **the actix App's `Data<LemmyContext>` and the `FederationConfig`'s inner `LemmyContext` must be ONE object derived from a single source-of-truth `FederationConfig`** — the production line-for-line pattern at `crates/server/src/lib.rs:228-241/364/379-382`.

Concrete shape of the test (replaces the body at `e2e.rs:14865-14981`, single anchored Edit):

```
                  +---- governance_fixtures::bootstrap() at e2e.rs:801
                  |     - starts testcontainer Postgres
                  |     - sets LEMMY_DATABASE_URL env
                  |     - returns Data<LemmyContext> (context_data) tied to that pool
                  |
                  v
   AGPL seed --> [(**context_data).pool()] -- seed: Instance / Site (complete) /
                                              sysacct Person / LocalSite /
                                              LocalSiteRateLimit (fix-impl-6 Part B)
                  |
                  v
   FederationConfig::builder()
       .domain((**context_data).settings().hostname.clone())
       .app_data((**context_data).clone())   // derefs Data<LemmyContext> -> LemmyContext -> clone (Arc internal so SAME pool)
       .debug(true)
       .http_fetch_limit(0)
       .build().await?
                  |
                  v
   federation_config : FederationConfig<LemmyContext>
                  |
                  +--- federation_config.deref().clone()    --> inner_context : LemmyContext (mirrors lib.rs:364 EXACTLY)
                  |                                              SAME pool by Arc-clone semantics
                  v
   App::new()
     .app_data(Data::new(inner_context.clone()))                 // actix Data<LemmyContext>   (mirrors lib.rs:379)
     .wrap(FederationMiddleware::new(federation_config.clone()))  //                            (mirrors lib.rs:380)
     .wrap(IdempotencyMiddleware::new(idempotency_set.clone()))   //                            (mirrors lib.rs:381)
     .wrap(SessionMiddleware::new(inner_context.clone()))         //                            (mirrors lib.rs:382)
     .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit))
                  |
                  v
   test::init_service(...) -> app
                  |
                  +---- GET /api/v4/site   --> assert source_disclosure block
                  +---- GET /api/v4/source --> assert AGPL notice body
```

The lone single-line bug in fix-impl-8 was line 14920: `.app_data(Data::new(context.clone()))` where `context` is `Data<LemmyContext>` (from bootstrap) — `Data::new(Data<LemmyContext>)` produces `Data<Data<LemmyContext>>`, registered under that doubled type. The handler at `read.rs:26` extracts `Data<LemmyContext>` (single-level actix Data), can't find it in app_data, returns HTTP 500. Fix: `inner_context.clone()` is `LemmyContext` (raw, not double-wrapped), so `Data::new(inner_context.clone())` produces correctly-typed `Data<LemmyContext>`.

The reader should be able to predict §11 (only `crates/server/tests/e2e.rs` modified — the single failed fn region replaced) from §4 alone.

## 5. Metadata

- **Phase:** `v1-ship-1` (the `-r2` suffix marks the second re-plan; phase branch stays `phase-v1-ship-1`).
- **Branch:** `phase-v1-ship-1` (already cut; Tasks 1-4 merged; this plan's worker branches off the current tip).
- **Target impl-task model:** `sonnet-4-6`.
- **Estimated tasks:** **3** — Task 0 pre-flight + Task 6 (the e2e rebuild) + Task 7 retro. Tasks 1-5 are carried-forward / already-shipped (see §13 carry-forward note). Task numbering preserves continuity with the parked `v1-ship-1.plan.md` / `r1.plan.md` (Tasks 1-4 are the merged surface tasks; Task 5 was the e2e in r1, replaced by Task 6 here).
- **Estimated cargo budget:** ~6 GB peak local (cargo-check warm; testcontainer Postgres ~512 MB; e2e ~5 GB Cargo working set). Local-only — Shape G suspended per DQ #229.
- **Forbidden-window applicability:** standard (per `.claude/rules/advisor-orchestrator.md` §5.1 table) — binding because §5.2-laptop runs cargo locally.
- **Complexity score:** **2/10** — well below the Sonnet split threshold (`>8`). The residual surface is one e2e test on a verified-canonical idiom.

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Target model is Sonnet 4.6 → split-DQ threshold is `> 8`.

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 1 impl task (Task 6); below threshold |
| Migrations touched | +2 each | 0 | No schema changes |
| Crates touched | +1 each | 1 | Only `crates/server/tests/e2e.rs` (lemmy_server test target) modified by an impl task |
| `crates/server/tests/e2e.rs` edits | +3 each | 1 | One §13 impl task with `crates/server/tests/e2e.rs` in `modifies:` (Task 6) |
| New ADR-affecting decisions | +2 each | 0 | ADR-011 surfaced (already-merged surface); no amendment |
| Cargo budget peak above 6 GB | +1 per GB | 0 | At ~6 GB; not above |
| **Total** | — | **5** | Below the Sonnet `>8` threshold → no split-DQ |

Re-derivation note vs r1's 8/10: the r1 score counted all 4 surface impl tasks + 5 crates + 3× e2e weight. r2 inherits only the e2e factor (the single residual impl task) + 1 crate; the merged surface is NOT re-counted (the work is already on the phase branch). Honest re-count = 5.

### 5.2 Per-task complexity ceiling (Sonnet target)

Sonnet ceiling: `≤ 4` files per task / `≤ 2` crates per task / e2e edits in their own dedicated task. Walk:

- Task 0 (pre-flight): 0 files / 0 crates. ✓
- Task 6 (e2e — dedicated): 1 file / 1 crate (lemmy_server test target). ✓
- Task 7 (retro): 1 file / 0 production crates. ✓

No split-candidate.

## 6. Relationship to other v1-ship sub-phases

- **No dependency on v1-ship-2 / v1-ship-3** (PRD §7.2 / §7.3) — independent.
- **No dependency on RT-r1..RT-r6 / JM-f / SL-d / restorative-mechanics-v1.** Concurrent lane (per DQ #260 — file-ownership disjoint).
- **No dependency on v1-federation-inbound-a** — DQ #260 verified disjoint (federation-inbound-a touches `schema.rs` + `config.rs` federation.inbound.*, NOT `e2e.rs`).

## 7. Preflight guardrails inherited from prior phases

- **R1** (per `feedback_clippy_test_style.md`): every `i32 ↔ i64` comparison uses `i64::from(...)`, never `as` cast. *Light here* — this plan introduces no integer comparisons.
- **R5** (per JM-b retro Event 4 + `.claude/rules/pre-phase-harness-audit.md`): Task 0 enumerates ALL probes explicitly.
- **R6** (per JM-b retro Event 3): clippy invocations use `--no-deps` uniformly. §15.2 honors.
- **R7** (per `feedback_test_target_compile_validation.md`): tasks touching a struct or re-export run `cargo test --no-run -p lemmy_server --test e2e`. Task 6 touches the e2e target itself; §15 Phase-1 covers this.
- **R8** (per `feedback_lemmy_error_no_std_error.md` Case A): test fn outer return is `lemmy_utils::error::LemmyResult<()>`; helpers + Lemmy-native calls + serde_json calls bridge via bare `?`. The current failed fn at `e2e.rs:14865` already uses Case A correctly — the rebuild preserves it.
- **R9** (per `feedback_planner_enumerate_struct_callsites_for_addfield.md`): not applicable — no struct field added in this plan; the field-add was Task 2 (merged).
- **R10** (per `feedback_pre_phase_dod_smoke_test.md` + `feedback_plan_dod_dry_run_at_write.md`): every §15 command shape verified executable at plan-write time.
- **R11 (new, from this plan's DQ #261):** **for any e2e test HTTP-driving `actix_web::web::Data<LemmyContext>` extractors, the registered `.app_data(...)` MUST be a single-level `actix_web::web::Data<LemmyContext>` — never a `Data<Data<T>>` produced by `Data::new(<already-wrapped-Data>)`, and never `to_request_data()`'s return (which is `activitypub_federation::config::Data<T>`, a different type).** Mirror `lib.rs:364` + `lib.rs:379` literally — derive raw `LemmyContext` from `federation_config.deref().clone()`, then wrap once in actix `Data::new(...)`.

## 8. Flow design

### Before (failed shape — fix-impl-8, currently on phase branch tip at `e2e.rs:14918-14926`)

```
governance_fixtures::bootstrap()
   |
   v
context : actix_web::web::Data<LemmyContext>      <- Arc<LemmyContext>
                                |
              (**context).clone() ----+--- federation_config = FederationConfig::builder()
                                |    |        .app_data((**context).clone())   // LemmyContext
                                |    |        .build().await?
                                |    v
                                |    federation_config : FederationConfig<LemmyContext>
                                v
                                +--> Data::new(context.clone())    // <- BUG: context is already Data<LemmyContext>;
                                                                   //          Data::new() wraps once more ->
                                                                   //          Data<Data<LemmyContext>>
                                +--> App::new()
                                       .app_data(Data<Data<LemmyContext>>)        <- registered under WRONG type
                                       .wrap(FederationMiddleware::new(federation_config.clone()))
                                       .wrap(IdempotencyMiddleware::new(...))
                                       .wrap(SessionMiddleware::new((**context).clone()))
                                       .configure(... lemmy_api_routes::config ...)

GET /api/v4/site -> handler get_site(context: Data<LemmyContext>)
                       ^ tries to extract Data<LemmyContext>; NOT in app_data (only Data<Data<LemmyContext>> is)
                   -> actix returns HTTP 500 "Requested application data is not configured correctly"
```

### After (verified-correct shape — mirrors `lib.rs:228-241/364/379-382` LINE-FOR-LINE)

```
governance_fixtures::bootstrap()   // unchanged — testcontainer Postgres + sets LEMMY_DATABASE_URL
   |
   v
context : Data<LemmyContext>  (actix Data; from bootstrap line 827)
   |
   |                                                                    (fix-impl-6 Part B PRESERVED verbatim)
   +-- AGPL surface seed against (&mut (**context).pool()) :
   |     Instance::read_or_create("test.invalid")
   |     SiteInsertForm { ap_id, inbox_url, public_key, private_key, last_refreshed_at, .. }
   |     Site::create
   |     PersonInsertForm::test_form("agpl_sysacct")
   |     Person::create  (sysacct)
   |     LocalSite::create  (FK -> sysacct.id)
   |     LocalSiteRateLimit::create
   |
   v
federation_config = FederationConfig::builder()                          (mirrors lib.rs:228-241 SHAPE)
                      .domain((**context).settings().hostname.clone())
                      .app_data((**context).clone())                     // LemmyContext (NOT Data)
                      .debug(true)
                      .http_fetch_limit(0)
                      .build().await?
   |
   |    federation_config : FederationConfig<LemmyContext>
   v
inner_context = federation_config.deref().clone()                        (lib.rs:364 LINE-FOR-LINE MIRROR)
   |                                          ^^ Deref<Target=LemmyContext>; clone returns a fresh
   |                                             LemmyContext sharing the SAME ActualDbPool (Arc-inside)
   |
   v
idempotency_set = IdempotencySet::default()
   |
   v
App::new()
  .app_data(Data::new(inner_context.clone()))                            // actix Data<LemmyContext>      (lib.rs:379 MIRROR)
  .wrap(FederationMiddleware::new(federation_config.clone()))            // request-extension              (lib.rs:380 MIRROR)
  .wrap(IdempotencyMiddleware::new(idempotency_set.clone()))             //                                (lib.rs:381 MIRROR)
  .wrap(SessionMiddleware::new(inner_context.clone()))                   //                                (lib.rs:382 MIRROR)
  .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit))           // same as failed shape

GET /api/v4/site -> get_site(local_user_view: Option<LocalUserView>, context: Data<LemmyContext>)
                       ^ actix finds Data<LemmyContext> in app_data -> extraction OK
                   -> read_site(&context) -> SiteView::read_local(&mut context.pool()) -> returns seeded row
                   -> 200 OK; serialised GetSiteResponse carries source_disclosure block

GET /api/v4/source -> get_source() (no params; in-file include_str!)
                   -> 200 OK; { notice: <AGPL-NOTICE.md>, license: "AGPL-3.0" }
```

Boxes <-> §13 tasks:
- `federation_config` build + `inner_context` derive + App composition + 2 GETs + assertions -> **Task 6** (the only impl task in this plan).
- Tasks 1-4 (DTO + field + handler/route + build.rs) are **MERGED, carry forward** (no impl re-execution).

## 9. Mandatory reading

The `impl-task` subagent reads these before its first edit. Every line range was confirmed by `Read` at plan-write time (worker branch off `phase-v1-ship-1` @ `92c4cc190`).

### Schema/type definitions

- `crates/api/api_utils/src/context.rs:1-101` — full file. **Critical lines:**
  - `:2` — `use activitypub_federation::config::{Data, FederationConfig};` (the `Data` in `init_test_context()` return is `activitypub_federation::config::Data`, NOT actix Data — see DQ #261 + §10.7).
  - `:39-44` — `pool() -> DbPool<'_>` and `inner_pool() -> &ActualDbPool` accessors.
  - `:65-96` — `init_test_federation_config()` (the federation_config builder pattern to mirror in §10.7).
  - `:97-100` — `init_test_context()` (the canonical idiom that returns `activitypub_federation::config::Data<LemmyContext>` via `to_request_data()`; **NOT** directly used by the e2e because the handler extracts actix Data — see §10.7).
- `crates/server/src/lib.rs:228-241` — `federation_config` build (`app_data(context.clone())`, `debug(cfg!(debug_assertions))`, `http_fetch_limit(FEDERATION_HTTP_FETCH_LIMIT)`).
- `crates/server/src/lib.rs:247` — `let request_data = federation_config.to_request_data();` (used for outgoing activities task at `:249` + scheduled tasks at `:253`; **NOT** for the App's `app_data` — see §10.7).
- `crates/server/src/lib.rs:349-386` — `create_http_server` body. **Critical lines:**
  - `:363-364` — `HttpServer::new(move || { let context: LemmyContext = federation_config.deref().clone(); ... })` — the canonical pattern Task 6 mirrors.
  - `:368-386` — full App composition: `.app_data(Data::new(context.clone()))` at `:379`, `.wrap(FederationMiddleware::new(federation_config.clone()))` at `:380`, `.wrap(IdempotencyMiddleware::new(idempotency_set.clone()))` at `:381`, `.wrap(SessionMiddleware::new(context.clone()))` at `:382`.
  - `:56` — `use std::{ops::Deref, time::Duration};` — confirms `.deref()` needs the `Deref` trait import (Task 6's test imports must include `use std::ops::Deref;` inside the fn block OR use the `(*federation_config).clone()` syntactic form which auto-derefs).
- `crates/api/api_crud/src/site/read.rs:1-82` — full file (merged, on phase tip). **Critical lines:**
  - `:1` — **`use actix_web::web::{Data, Json};`** — `get_site`'s `Data` is actix Data, the load-bearing fact for DQ #261 + §10.7.
  - `:24-27` — `get_site` extractor chain: `pub async fn get_site(local_user_view: Option<LocalUserView>, context: Data<LemmyContext>) -> LemmyResult<Json<GetSiteResponse>>`.
  - `:74-79` — `source_disclosure: SourceDisclosure { license, repo_url, fork_commit, disclosure_url }` populated in the constructor (merged Task 2).
- `crates/api/api_crud/src/site/read.rs:20-22` — three const declarations (`BREHON_FORK_COMMIT` / `BREHON_REPO_URL` / `SOURCE_DISCLOSURE_URL`) — context for Task 6 assertions.
- `crates/api/api/src/site/source.rs` — entire file (merged Task 3); confirms `get_source()` returns `LemmyResult<Json<GetSourceResponse>>` with no params.
- `crates/api/routes/src/lib.rs:288` (approx — re-grep for `"/source", get().to(get_source)`) — merged route registration (Task 3).

### Existing patterns (MIRROR refs cited in §10 + §13 Task 6)

- `crates/server/tests/e2e.rs:118-119` — `mod governance_fixtures {` + `use actix_web::web::Data;` (the `Data` type inside the mod is actix Data; `bootstrap()` at `:801` returns `Data<LemmyContext>` = actix `Data<LemmyContext>`).
- `crates/server/tests/e2e.rs:801-836` — `governance_fixtures::bootstrap()` body. Return tuple: `(testcontainers::ContainerAsync<testcontainers::GenericImage>, Data<LemmyContext>, String)`. Sets `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS=1`, `GOVERNANCE_LOG_SIGNING_KEY=<seed>`, `LEMMY_DATABASE_URL=<container url>` (`:814`), then `let pool: ActualDbPool = build_db_pool_for_tests();` (`:822`) — the testcontainer + seed-pool reconciliation.
- `crates/server/tests/e2e.rs:5665` — `pub async fn bootstrap()` inside `mod admin_config_fixtures` (line `:5643`). **The WRONG bootstrap for AGPL — REJECTED per DQ #226.** Depends on `super::governance_fixtures::start_postgres` and bumps rate-limit buckets internally; not what an HTTP-surface test wants. Task 6 MUST call `governance_fixtures::bootstrap()` at line 801, NOT this sibling.
- `crates/server/tests/e2e.rs:14865-14981` — the **failed test region** (current fix-impl-8 body). Task 6 REPLACES this body with a single anchored Edit; the test fn name stays `agpl_source_disclosure_surface_returns_notice` verbatim (PRD §7.1 + `/brehon-verify` Story 3 greps this literal).
- `crates/server/tests/e2e.rs:14881-14907` — fix-impl-6 Part B (the complete `SiteInsertForm` + keypair + sysacct + LocalSite + LocalSiteRateLimit seed). **PRESERVE VERBATIM** — this part is correct (DQ #226 + the parked-plan investigation refuted the incomplete-site-row theory; Finding 4 in brief §0.1).
- `crates/server/tests/e2e.rs:14929-14937` + `:14958-14966` — fix-impl-6 Part A (body-on-failure asserts: `assert_eq!(status, 200, "... — body: {}", String::from_utf8_lossy(&body))`). **PRESERVE VERBATIM** — twice-decisive in surfacing the 500 + body.
- `crates/server/tests/e2e.rs:3857-3862` — `all_mvp_endpoints_return_non_404` App construction. **Not directly mirrored** because that test builds its own raw `LemmyContext` at `:3849` (no `governance_fixtures::bootstrap()` and no Data-wrapped context to deref); the AGPL test's source-of-truth must be `governance_fixtures::bootstrap()` (for the testcontainer + seed) — so the AGPL test mirrors `lib.rs:228-241/364/379-382`, NOT `all_mvp_endpoints`. **State this verbatim in the impl-task brief** to pre-empt any "use the sibling test's pattern" miss.
- `crates/server/tests/e2e.rs:11131-11924` (the v1-SL-b `mod v1_sl_b_fixtures`) — canonical Case A reference per `feedback_lemmy_error_no_std_error.md:24`; every helper `LemmyResult<T>`, every test fn `LemmyResult<()>`, every `?` bare. Confirm the v1-SL-b mod still lives at this range before commit.
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/activitypub_federation-0.7.0-beta.11/src/config.rs:143` — `pub fn to_request_data(&self) -> Data<T>` (borrows `&self`, clones the config internally; verified DQ #258 — no consume risk for chained `FederationMiddleware::new(federation_config.clone())`).
- `~/.cargo/registry/src/.../activitypub_federation-0.7.0-beta.11/src/config.rs:264-270` — `impl<T: Clone> Deref for FederationConfig<T> { type Target = T; fn deref(&self) -> &T { &self.app_data } }` — the load-bearing impl that makes `federation_config.deref().clone()` produce a `LemmyContext`.
- `~/.cargo/registry/src/.../activitypub_federation-0.7.0-beta.11/src/config.rs:335-398` — `pub struct Data<T: Clone> { config: FederationConfig<T>, request_counter: RequestCounter }` + its own `Deref<Target=T>`. **Different type from `actix_web::web::Data<T>`.**
- `~/.cargo/registry/src/.../activitypub_federation-0.7.0-beta.11/src/actix_web/middleware.rs:1-77` — `FederationMiddleware::call` inserts `FederationConfig<T>` into `req.extensions_mut()` (line 58); `impl<T: Clone + 'static> FromRequest for Data<T>` (line 64-76) extracts `activitypub_federation::config::Data<T>` from those extensions. Does NOT touch `actix_web::web::Data<T>`.

### Adjacent test fixtures

- `crates/diesel_utils/src/connection.rs:160-199` — `build_db_pool()` (`:160`) reads `SETTINGS.get_database_url_with_options()` which resolves via `LEMMY_DATABASE_URL`; `build_db_pool_for_tests()` (`:197`) is the test-public wrapper. Confirms DQ #257.

### Lessons binding tasks

- `feedback_lemmy_error_no_std_error.md` Case A — Task 6 outer return.
- `feedback_junior_worker_e2e_edit_hang.md` — Task 6 single anchored Edit (replace lines 14865-14981 in ONE Edit).
- `feedback_features_full_p_crate_incompatible.md` — §15 Phase-1 cmd3 uses `-p lemmy_server --test e2e` (no `--features full`).
- `feedback_features_full_workspace_only.md` — §15 Phase-2 e2e uses `--workspace --features full`.
- `feedback_windows_e2e_requires_bat_wrapper.md` — local invocation via `cmd //c scripts\brehon\cargo-test.bat ...`.
- `feedback_laptop_default_for_validate_pending.md` — §5.2-laptop shape (Shape G suspended).
- `feedback_read_canonical_before_writing_spec.md` — mirror `lib.rs:228-241/364/379-382` LINE-FOR-LINE.
- `feedback_async_pool_test_pattern.md` — pool/conn/LemmyResult fixture discipline.
- `feedback_test_target_compile_validation.md` — Phase-1 cmd3 runs after the Edit.
- `feedback_principles_not_rules.md` — the planner's mechanism-precision call (DQ #261).

## 10. Patterns to mirror

Per `.claude/rules/advisor-orchestrator.md` §3.5 watchpoint specificity gate: every pattern cites a specific file:line; line numbers confirmed by `Read` at plan-write time.

### 10.1 GetSiteResponse + SourceDisclosure (SETTLED — merged, no re-derive)

**Mirror:** `crates/db_views/site/src/api.rs:332-385` (struct keyword for `GetSiteResponse` at `:337`; `SourceDisclosure` struct + `source_disclosure` field added by merged Task 1+2). Settled on phase branch tip @ `92c4cc190`. **No edit in this plan.**

### 10.2 read_site population (SETTLED — merged, no re-derive)

**Mirror:** `crates/api/api_crud/src/site/read.rs:1-82` (merged Task 2). Settled. **No edit in this plan.**

### 10.3 get_source handler (SETTLED — merged, no re-derive)

**Mirror:** `crates/api/api/src/site/source.rs` (merged Task 3). Settled. **No edit in this plan.**

### 10.4 build.rs env injection (SETTLED — merged, no re-derive)

**Mirror:** `crates/api/api_crud/build.rs` (merged Task 2). Settled. **No edit in this plan.**

### 10.5 Real-server federation_config build (THE primary mirror for Task 6)

**Mirror:** `crates/server/src/lib.rs:228-241`. Symbols to grep if drifted: `let mut federation_config_builder = FederationConfig::builder();`, `.build().await?`.

```rust
let mut federation_config_builder = FederationConfig::builder();
federation_config_builder
  .domain(SETTINGS.hostname.clone())
  .app_data(context.clone())
  .client(client.clone())
  .http_fetch_limit(FEDERATION_HTTP_FETCH_LIMIT)
  .debug(cfg!(debug_assertions))
  .http_signature_compat(true)
  .url_verifier(Box::new(VerifyUrlData(context.inner_pool().clone())));
// (signed_fetch_actor branch omitted — irrelevant for the test)
let federation_config = federation_config_builder.build().await?;
```

Task 6's test mirrors a **simplified subset** appropriate for an in-process test:

```rust
let federation_config = activitypub_federation::config::FederationConfig::builder()
  .domain((**context).settings().hostname.clone())
  .app_data((**context).clone())   // (**context) derefs Data<LemmyContext> -> LemmyContext (via actix Data's Deref<Target=T>); clone gives LemmyContext
  .debug(true)                     // safe in tests
  .http_fetch_limit(0)             // tests never fetch
  .build()
  .await?;
```

**Omitted vs production builder (justified):** `.client(...)` is set by default from `init_test_federation_config()`-style construction at `context.rs:69-71` (the test's `context` already has its client baked in via bootstrap); `.http_signature_compat(true)` is irrelevant when `http_fetch_limit(0)` means zero outgoing fetches; `.url_verifier(...)` is irrelevant for the same reason; `signed_fetch_actor` is for federated-fetch signing (no federated fetch in this test).

### 10.6 Real-server context-out-of-config (THE central fix — mirrors lib.rs:364 line-for-line)

**Mirror:** `crates/server/src/lib.rs:364`. Symbols to grep if drifted: `let context: LemmyContext = federation_config.deref().clone();`.

```rust
let context: LemmyContext = federation_config.deref().clone();
```

Task 6's test (renames the local binding `inner_context` to avoid shadowing the bootstrap's outer `context: Data<LemmyContext>`):

```rust
use std::ops::Deref;  // bring trait into scope for `.deref()`
let inner_context: LemmyContext = federation_config.deref().clone();
```

**Why this works (BINDING — quote in commit body):**
1. `FederationConfig<T>: Deref<Target=T>` per `~/.cargo/registry/src/.../activitypub_federation-0.7.0-beta.11/src/config.rs:264-270`. So `federation_config.deref()` returns `&LemmyContext`.
2. `LemmyContext` is `Clone` (`crates/api/api_utils/src/context.rs:12`). So `.clone()` produces a fresh `LemmyContext`.
3. **The pool is shared:** `LemmyContext.pool: ActualDbPool`. `ActualDbPool` is a deadpool-diesel Pool which uses `Arc` internally. `.clone()` clones the Arc, NOT the pool. So `inner_context.pool()` and `(**outer_context).pool()` point at the SAME pool — and the AGPL seed (written via `outer_context`'s pool) is visible to handler reads (which use `inner_context`'s pool via `Data<LemmyContext>`).

**Equivalent syntactic form** (compiler accepts either; the explicit `.deref()` is preferred because it mirrors lib.rs:364 byte-for-byte and reads at the same grep target):

```rust
let inner_context: LemmyContext = (*federation_config).clone();   // (*X) auto-derefs via Deref; clones T = LemmyContext
```

### 10.7 In-process HTTP via actix_web::test (THE failed line + the corrected line)

**Mirror:** `crates/server/src/lib.rs:368-386` (production App composition). Task 6's test condenses to the load-bearing 5 lines:

```rust
let app = test::init_service(
  App::new()
    .app_data(Data::new(inner_context.clone()))                          // actix Data<LemmyContext>      (lib.rs:379 MIRROR)
    .wrap(FederationMiddleware::new(federation_config.clone()))          //                              (lib.rs:380 MIRROR)
    .wrap(IdempotencyMiddleware::new(idempotency_set.clone()))           //                              (lib.rs:381 MIRROR)
    .wrap(SessionMiddleware::new(inner_context.clone()))                 //                              (lib.rs:382 MIRROR)
    .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit)),
).await;
```

**Why the failed line was wrong (BINDING — quote in commit body + retro):**

The failed code at `e2e.rs:14920` was `.app_data(Data::new(context.clone()))` where:
- `context: Data<LemmyContext>` from `governance_fixtures::bootstrap()` at `:14881` (`bootstrap()` returns `actix_web::web::Data<LemmyContext>` per `e2e.rs:119` (`use actix_web::web::Data;`) + bootstrap line `:827` (`let context = Data::new(LemmyContext::create(...));`)).
- `context.clone()` is `Data<LemmyContext>` (Arc clone — same type).
- `Data::new(<arg>)` is `actix_web::web::Data::new`, signature `pub fn new(state: T) -> Data<T>`. Passing `Data<LemmyContext>` gives `Data<Data<LemmyContext>>`.
- `.app_data(<x>)` registers `<x>` keyed by `TypeId::of::<typeof(x)>()`. So the App stores under TypeId `Data<Data<LemmyContext>>`, NOT `Data<LemmyContext>`.
- `get_site` extractor `Data<LemmyContext>` (actix `FromRequest`) looks up `Data<LemmyContext>` in `app_data` — not found -> actix returns HTTP 500 *"Requested application data is not configured correctly. View/enable debug logs for more details."*

The **corrected line** registers `Data<LemmyContext>` (single-level) because `inner_context.clone()` is `LemmyContext` (NOT `Data<LemmyContext>`) and `Data::new(LemmyContext)` is `Data<LemmyContext>`. Extractor finds it. 200 OK.

**Why NOT `to_request_data()` (binding finding — see DQ #261):**

The brief §0.1 Option (b) literal recipe says:
> "...call `.to_request_data()` on it to obtain the `Data<LemmyContext>` to register as `.app_data(...)` — so the registered context derives from the SAME `bootstrap()` context the seed wrote into, via the canonical idiom..."

`FederationConfig::to_request_data(&self) -> Data<T>` per `activitypub_federation-0.7.0-beta.11/src/config.rs:143` returns **`activitypub_federation::config::Data<T>`** (per the `pub struct Data<T>` at `:335`). That is a DIFFERENT TYPE from `actix_web::web::Data<T>`.

- Passing `activitypub_federation::config::Data<LemmyContext>` to `.app_data(...)` registers it under TypeId `activitypub_federation::config::Data<LemmyContext>` (one TypeId).
- The handler `get_site` extracts `actix_web::web::Data<LemmyContext>` (read.rs:1 imports `use actix_web::web::{Data, Json};`; read.rs:26 uses `Data<LemmyContext>` from THAT import) — TypeId `actix_web::web::Data<LemmyContext>` (a SEPARATE TypeId).
- Mismatch -> same HTTP 500.

The brief's Option (b) literal recipe is **mechanically incompatible** with the merged `get_site` signature at current tip. The brief's *spirit* ("derive the actix `Data<LemmyContext>` from the same `FederationConfig` that drives `FederationMiddleware`, mirroring real-server's single source-of-truth composition") is preserved by the chosen `federation_config.deref().clone()` mechanism — which IS the production line-for-line at `lib.rs:364`. **This is not a design change vs the brief; it is the precise mechanism call inside Option (b) that the brief author got incorrect.** See §19 + DQ #261 for the full source-cited rationale; advisor validates at user gate 1.

**`init_test_context()` is NOT used directly** for the same reason — its return is `activitypub_federation::config::Data<LemmyContext>` (per `context.rs:97-100` returning `config.to_request_data()` with `Data` imported from `activitypub_federation::config` at line 2). Same type-mismatch.

**Why NOT just `.app_data(context.clone())` (dropping `Data::new()`)?** That would register `Data<LemmyContext>` directly (the bootstrap's `context` is already `Data<LemmyContext>`), and the handler would extract it correctly. This minimal-Edit fix would also work IN ISOLATION. **REJECTED** because:
1. The bootstrap's `Data<LemmyContext>` is built from a `LemmyContext` that was NOT routed through any `FederationConfig` — so a separate `FederationConfig` built later (with `app_data((**context).clone())`) has an `inner: LemmyContext` clone that is logically equivalent (same pool by Arc-clone) but distinct from the registered `Data`'s inner. Production `lib.rs:364+379` derives BOTH from one `federation_config.deref().clone()` to make the "single source of truth" invariant structural, not coincidental.
2. `feedback_read_canonical_before_writing_spec.md` — mirror the existing canonical composition, not a new variant. Production at `lib.rs:364-386` IS the canonical. The chosen mechanism IS that canonical.
3. Future-resilience — if `LemmyContext` ever gains a non-Arc field (e.g. a per-request token, or a settings override), the bootstrap-`Data` and federation-config-derived clones could legitimately diverge; the `lib.rs:364` pattern keeps the invariant safe.

### 10.8 e2e error-shape (Case A — settled; preserve)

**Mirror canonical Case A:** `crates/server/tests/e2e.rs:11131-11924` (the v1-SL-b `mod v1_sl_b_fixtures` per `feedback_lemmy_error_no_std_error.md:24`). Outer return `lemmy_utils::error::LemmyResult<()>`; bare `?` everywhere; `governance_fixtures::bootstrap().await?` propagates `LemmyError -> LemmyError` directly; `serde_json::from_slice(&bytes)?` propagates via `From<serde_json::Error> for LemmyError` already in scope.

The current failed fn at `e2e.rs:14865` already uses Case A correctly (`async fn agpl_source_disclosure_surface_returns_notice() -> lemmy_utils::error::LemmyResult<()>`). Task 6 preserves the signature verbatim — only the App-construction (lines `:14909-14926`) is replaced.

### 10.9 fix-impl-6 Part A (body-on-failure asserts — preserve verbatim)

**Mirror:** `e2e.rs:14928-14937` and `:14957-14966` (the two `let ... bytes = test::read_body(...).await; assert_eq!(status, 200, "... — body: {}", String::from_utf8_lossy(&bytes))` blocks). Task 6 preserves these verbatim. They are correct and twice-decisive in making failures legible.

### 10.10 fix-impl-6 Part B (complete SiteInsertForm + sysacct + LocalSite + LocalSiteRateLimit seed — preserve verbatim)

**Mirror:** `e2e.rs:14881-14907` (the full seed scaffold). Task 6 preserves this verbatim. It writes against `&mut (**context).pool()` (the bootstrap's pool); the corrected App-construction derives its `inner_context` via `federation_config.deref().clone()` which shares the SAME pool (Arc-clone semantics) — so handler reads see the seeded rows.

## 11. Files to change

Grouped by crate. Every path confirmed to exist at worker-branch tip @ `92c4cc190`.

### `crates/server/tests/` (test target; `lemmy_server` crate)

- `crates/server/tests/e2e.rs` — Task 6 **replaces the body of `agpl_source_disclosure_surface_returns_notice`** (currently at `:14865-14981`). Single anchored Edit. The fn signature line (`:14865`) stays; the closing `}` (`:14981`) stays; everything between is replaced.

**No other file in any crate is touched by this plan.** Tasks 1-4 (the surface) are merged on `phase-v1-ship-1` and CARRIED FORWARD as already-shipped context.

### Caller enumeration — N/A

Per `feedback_planner_enumerate_struct_callsites_for_addfield.md`: not triggered. No struct field added; the field-add was merged Task 2; current callsites are settled and `routes_v3/handlers.rs:249` uses a `..` rest-pattern destructure that's additive-safe.

## 12. NOT building in v1-ship-1-r2

- **Changes to the merged surface (Tasks 1-4 — DTOs / `read_site` / `get_source` / `build.rs`).** Already shipped on `phase-v1-ship-1` and PROVEN CORRECT. If the impl-task believes any merged surface needs change for the e2e to pass, STOP and file `kind: "blocker"` DQ — that is an advisor/user decision, not a Task 6 self-resolution.
- **A third hand-assembly variant** (e.g. "add one more middleware / change wrap order / try a different SessionMiddleware arg shape"). Hand-assembly is the proven-failed class. The mechanism is fixed at "mirror `lib.rs:228-241/364/379-382` line-for-line."
- **`init_test_context()` direct adoption** (Brief §0.1.1 Option (a)). Decided AGAINST per §10.7 + DQ #261 (the return type is federation Data, not actix Data; same incompatibility as `to_request_data()`).
- **A second e2e test** (e.g. covering anon-vs-authed for `/api/v4/site`). Out of scope per PRD §7.1 ship-criteria; the single named test is the ship gate.
- **`AGPL-NOTICE.md` content edits.** Out of scope per `v1-ship-1-r1.plan.md` §12 (still binding).
- **Re-running merged Tasks 1-4.** They are already on phase branch. The §13 carry-forward notes them as merged-context; Task 0's Probe 6/7 confirms presence.
- **WebAuthn / SBOM / Sigstore Rekor / per-route rate-limit overrides** — all PRD-deferred per `v1-ship-1-r1.plan.md` §12 (carried forward verbatim).

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per Task** (per `feedback_pr_per_phase.md`). §5.2-laptop DoD shape (Shape G suspended per DQ #229 until 2026-06-01 — see §15 + §0.2 of brief).

> **§13 Tasks 1, 2, 3, 4 — MERGED (carry-forward; do NOT re-execute).**
>
> - **Task 1** (DTOs: `SourceDisclosure`, `GetSource`, `GetSourceResponse`) — MERGED at `phase-v1-ship-1` tip ancestry (see `crates/db_views/site/src/api.rs:332+` on phase branch). No re-impl.
> - **Task 2** (`source_disclosure` field on `GetSiteResponse` + `read_site` population + `crates/api/api_crud/build.rs` env injection) — MERGED (see `crates/api/api_crud/src/site/read.rs:74-79` on phase branch). No re-impl.
> - **Task 3** (`get_source` handler + `/api/v4/source` route + module re-export) — MERGED (see `crates/api/api/src/site/source.rs` + `crates/api/routes/src/lib.rs` on phase branch). No re-impl.
> - **Task 4** (`agpl_source_disclosure_surface_returns_notice` e2e test fn shell — Phase-1 PASS shipped, Phase-2 e2e fails) — the **fn signature** at `e2e.rs:14865` is merged; the **fn body** at `:14866-14980` is the failed App-construction this re-plan REPLACES via Task 6. The fn-signature line is preserved verbatim by Task 6 (single anchored Edit replaces lines `:14866-14980` only, not the signature).
>
> Task numbering preserves continuity with the parked `v1-ship-1.plan.md` / `r1.plan.md`. **Cohort plan:**
>
> - Task 0: solo (pre-flight barrier — always non-`[P]`).
> - Task 6: solo (e2e — no `[P]`-cohort because it's the only impl task; barrier for retro).
> - Task 7: solo (retro — always non-`[P]`).

### Task 0: Pre-flight harness audit + branch verification + merged-surface presence check

**Goal:** verify environment ready; branch is `phase-v1-ship-1`; merged Tasks 1-4 deliverables present on phase branch; AGPL-NOTICE.md present; the failed test region still at the expected location; canonical mirror anchors at `lib.rs` still present.

**FILES:**

```yaml
creates: []
modifies: []
requires: []
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` + R5 — enumerate ALL probes explicitly):**

```bash
# Probe 0 — Docker daemon running (Shape G suspended; e2e runs locally)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — branch sanity
test "$(git branch --show-current)" = "phase-v1-ship-1" || { echo "WRONG BRANCH"; exit 1; }

# Probe 2 — working tree clean
test -z "$(git status --porcelain)" || { echo "WORKING TREE DIRTY"; exit 1; }

# Probe 3 — phase branch descends from governance-v0
git merge-base --is-ancestor governance-v0 HEAD && echo "BASE OK" || { echo "BASE DRIFT"; exit 1; }

# Probe 4 — AGPL-NOTICE.md present (Task 3's include_str! target)
test -f AGPL-NOTICE.md && [ "$(wc -c < AGPL-NOTICE.md)" -gt 1000 ] && \
  echo "AGPL-NOTICE OK ($(wc -c < AGPL-NOTICE.md) bytes)" || { echo "AGPL-NOTICE MISSING OR EMPTY"; exit 1; }

# Probe 5 — merged Task 1+2 surface present (source_disclosure field populated in read_site)
grep -nE "source_disclosure: SourceDisclosure \{" crates/api/api_crud/src/site/read.rs | head -1
# EXPECT: one line (currently at :74)

# Probe 6 — merged Task 2 build.rs present
test -f crates/api/api_crud/build.rs && \
  grep -q 'BREHON_FORK_COMMIT' crates/api/api_crud/build.rs && \
  echo "BUILD.RS OK" || { echo "BUILD.RS MISSING OR WRONG"; exit 1; }

# Probe 7 — merged Task 3 surface present (get_source handler + module)
test -f crates/api/api/src/site/source.rs || { echo "source.rs MISSING"; exit 1; }
grep -q 'pub async fn get_source' crates/api/api/src/site/source.rs || { echo "get_source MISSING"; exit 1; }
grep -q 'pub mod source;' crates/api/api/src/site/mod.rs || { echo "module re-export MISSING"; exit 1; }
grep -q '"/source", get().to(get_source)' crates/api/routes/src/lib.rs || { echo "/source route MISSING"; exit 1; }
echo "TASK 3 SURFACE OK"

# Probe 8 — failed test fn region present (Task 6's Edit target)
grep -nE "^async fn agpl_source_disclosure_surface_returns_notice\(\) -> lemmy_utils::error::LemmyResult<\(\)>" crates/server/tests/e2e.rs | head -1
# EXPECT: one line near :14865 (drift ±50 lines acceptable; symbol presence is the contract)

# Probe 9 — canonical mirror anchors at lib.rs still present
grep -nE 'let context: LemmyContext = federation_config\.deref\(\)\.clone\(\);' crates/server/src/lib.rs | head -1
# EXPECT: one line near :364 (the §10.6 byte-for-byte mirror anchor)
grep -nE '\.app_data\(Data::new\(context\.clone\(\)\)\)' crates/server/src/lib.rs | head -1
# EXPECT: one line near :379 (the §10.7 app_data mirror anchor)
grep -nE '\.wrap\(FederationMiddleware::new\(federation_config\.clone\(\)\)\)' crates/server/src/lib.rs | head -1
# EXPECT: one line near :380

# Probe 10 — context.rs canonical fixture anchors present
grep -nE 'pub async fn init_test_federation_config\(\) -> FederationConfig<LemmyContext>' crates/api/api_utils/src/context.rs | head -1
# EXPECT: one line near :65
grep -nE 'pub async fn init_test_context\(\) -> Data<LemmyContext>' crates/api/api_utils/src/context.rs | head -1
# EXPECT: one line near :97 (the activitypub_federation Data return — informational)

# Probe 11 — governance_fixtures::bootstrap chosen anchor present + admin_config_fixtures::bootstrap REJECTED anchor present (DQ #226 sanity)
grep -cE "^  pub async fn bootstrap\(\) -> LemmyResult<\(" crates/server/tests/e2e.rs
# EXPECT: 2 (governance_fixtures + admin_config_fixtures)
grep -nE "^  pub async fn bootstrap\(\) -> LemmyResult<\(" crates/server/tests/e2e.rs | head -1
# EXPECT: first hit near :801 (governance_fixtures::bootstrap — the chosen one)

# Probe 12 — activitypub_federation Cargo.lock version still 0.7.0-beta.11 (DQ #258 sanity)
grep -A1 '^name = "activitypub_federation"' Cargo.lock | grep -q 'version = "0.7.0-beta.11"' && \
  echo "AF VERSION OK" || { echo "AF VERSION DRIFTED — re-verify DQ #258 against new version"; exit 1; }

# Probe 13 — concurrent-PR check (no other open PR touches e2e.rs in the AGPL fn region)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path == "crates/server/tests/e2e.rs") | {number, title, headRefName}'
# EXPECT: empty output. Non-empty -> STOP and reconcile (file ownership conflict).

# Probe 14 — wrapper-script flag-silence (Windows positive probe)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server > .claude/audit-cargo-check-p.log 2>&1"
echo "Probe 14 exit: $?"
tail -5 .claude/audit-cargo-check-p.log
# EXPECT: exit 0 + log shows only lemmy_server compile.

# Probe 15 — wrapper-script exit-code propagation (Windows negative probe; per pre-phase-harness-audit.md §1)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-check-negative.log 2>&1"
echo "Probe 15 exit on bogus feature: $?"
# EXPECT: non-zero (typically 101). If 0 -> wrapper masks exit codes; STOP.
```

**EXPECT block:**

- Probes 0-13 all pass (exit 0 or expected output as commented).
- Probe 14 exits 0 (positive).
- Probe 15 exits non-zero (negative — exit-code propagation OK).

**No commit at Task 0** — verification only. If any probe fails, the impl-task halts with `FAIL: Probe <N> — <one-line>` and does NOT proceed to Task 6.

### Task 6: Rebuild the App-construction inside `agpl_source_disclosure_surface_returns_notice` on the canonical `lib.rs:228-241/364/379-382` idiom

**ACTION:** in `crates/server/tests/e2e.rs`, **single anchored Edit** that replaces the body of the existing test fn `agpl_source_disclosure_surface_returns_notice` (currently lines `:14866-14980` between the signature at `:14865` and the closing `}` at `:14981`) with the verified-correct App construction per §10.5/§10.6/§10.7. Preserve fix-impl-6 Part A (body-on-failure asserts) + Part B (complete `SiteInsertForm`/keypair/sysacct/LocalSite/LocalSiteRateLimit seed) **verbatim**. Preserve the fn signature `async fn agpl_source_disclosure_surface_returns_notice() -> lemmy_utils::error::LemmyResult<()>` verbatim.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # single anchored Edit replacing lines 14866-14980 (the failed App construction). Signature line :14865 + closing `}` :14981 untouched.
requires:
  - task: 1
    reason: GetSiteResponse must carry SourceDisclosure (merged on phase-v1-ship-1).
  - task: 2
    reason: source_disclosure field populated in read_site + BREHON_FORK_COMMIT injected (merged).
  - task: 3
    reason: /api/v4/source route registered + handler present (merged).
  - task: 4
    reason: fn signature `agpl_source_disclosure_surface_returns_notice() -> LemmyResult<()>` present on phase branch (merged Task 4 of r1.plan.md).
```

**IMPLEMENT (file 1 of 1):** in `crates/server/tests/e2e.rs`, single anchored Edit replacing lines `:14866-14980` with the verified-correct body. The **exact replacement body** (Case A discipline; preserves fix-impl-6 Part A + Part B; mirrors §10.5/§10.6/§10.7 verbatim):

```rust
  use actix_web::{App, test, web::Data};
  use lemmy_db_views_site::api::{GetSiteResponse, GetSourceResponse};
  use lemmy_utils::rate_limit::RateLimit;
  use lemmy_db_schema::source::{
    instance::Instance,
    local_site::{LocalSite, LocalSiteInsertForm},
    local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitInsertForm},
    person::{Person, PersonInsertForm},
    site::{Site, SiteInsertForm},
  };
  use lemmy_diesel_utils::traits::Crud;
  use lemmy_routes::middleware::session::SessionMiddleware;
  use lemmy_routes::middleware::idempotency::{IdempotencyMiddleware, IdempotencySet};
  use activitypub_federation::config::{FederationConfig, FederationMiddleware};
  use lemmy_api_utils::context::LemmyContext;
  use std::ops::Deref;

  // ------------------- 1. testcontainer + AGPL surface seed (fix-impl-6 Part B PRESERVED) -------------------
  let (_container, context, _db_url) = governance_fixtures::bootstrap().await?;

  // Seed instance + Site + LocalSite + LocalSiteRateLimit so `SiteView::read_local`
  // (called by `read_site` for GET /api/v4/site) returns a row instead of
  // LocalSiteNotSetup -> HTTP 500. Mirrors the canonical scaffold at e2e.rs:4751-4761
  // (governance_outbox_emits_remote_sanction_notice_on_local_sanction).
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  {
    let pool = &mut context.pool();
    let site_key_pair = activitypub_federation::http_signatures::generate_actor_keypair()?;
    let site_form = SiteInsertForm {
      ap_id: Some(url::Url::parse("https://test.invalid")?.into()),
      last_refreshed_at: Some(chrono::Utc::now()),
      inbox_url: Some(url::Url::parse("https://test.invalid/inbox")?.into()),
      private_key: Some(site_key_pair.private_key),
      public_key: Some(site_key_pair.public_key),
      ..SiteInsertForm::new("agpl test site".to_string(), instance.id)
    };
    let site = Site::create(pool, &site_form).await?;
    // System account: throwaway Person — LocalSite needs a non-null FK.
    let sysacct_form = PersonInsertForm::test_form(instance.id, "agpl_sysacct");
    let sysacct = Person::create(pool, &sysacct_form).await?;
    let local_site_form = LocalSiteInsertForm::new(site.id, sysacct.id);
    let local_site = LocalSite::create(pool, &local_site_form).await?;
    LocalSiteRateLimit::create(pool, &LocalSiteRateLimitInsertForm::new(local_site.id)).await?;
  }

  // ------------------- 2. federation_config + inner_context (mirrors lib.rs:228-241 + lib.rs:364 VERBATIM) -------------------
  // §10.5: build FederationConfig from the bootstrap context. `(**context).clone()`
  // derefs Data<LemmyContext> -> LemmyContext (via actix Data's Deref<Target=T>);
  // clone gives a fresh LemmyContext whose ActualDbPool is Arc-shared with the
  // bootstrap's pool — so the AGPL seed (written via context.pool() above) is
  // visible to handler reads (via the inner_context.pool() below).
  let federation_config = FederationConfig::builder()
    .domain((**context).settings().hostname.clone())
    .app_data((**context).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await?;

  // §10.6: lib.rs:364 line-for-line mirror.
  // `FederationConfig<T>: Deref<Target=T>` (config.rs:264-270). `.deref().clone()` gives a
  // LemmyContext sharing the SAME pool as `federation_config.app_data`'s inner clone.
  let inner_context: LemmyContext = federation_config.deref().clone();
  let idempotency_set = IdempotencySet::default();

  // ------------------- 3. App composition (mirrors lib.rs:379-382 VERBATIM) -------------------
  let rate_limit = RateLimit::with_debug_config();
  let app = test::init_service(
    App::new()
      .app_data(Data::new(inner_context.clone()))                                  // lib.rs:379 mirror — actix Data<LemmyContext>
      .wrap(FederationMiddleware::new(federation_config.clone()))                  // lib.rs:380 mirror
      .wrap(IdempotencyMiddleware::new(idempotency_set.clone()))                   // lib.rs:381 mirror
      .wrap(SessionMiddleware::new(inner_context.clone()))                         // lib.rs:382 mirror
      .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit)),
  )
  .await;

  // ------------------- 4. GET /api/v4/site — assert source_disclosure block (fix-impl-6 Part A PRESERVED) -------------------
  let site_req = test::TestRequest::get().uri("/api/v4/site").to_request();
  let site_resp = test::call_service(&app, site_req).await;
  let site_status = site_resp.status().as_u16();
  let site_body_bytes = test::read_body(site_resp).await;
  assert_eq!(
    site_status, 200,
    "/api/v4/site must return 200 — body: {}",
    String::from_utf8_lossy(&site_body_bytes)
  );
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

  // ------------------- 5. GET /api/v4/source — assert AGPL notice body (fix-impl-6 Part A PRESERVED) -------------------
  let source_req = test::TestRequest::get().uri("/api/v4/source").to_request();
  let source_resp = test::call_service(&app, source_req).await;
  let source_status = source_resp.status().as_u16();
  let source_body_bytes = test::read_body(source_resp).await;
  assert_eq!(
    source_status, 200,
    "/api/v4/source must return 200 — body: {}",
    String::from_utf8_lossy(&source_body_bytes)
  );
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
```

**MIRROR:** §10.5 (`lib.rs:228-241` federation_config build), §10.6 (`lib.rs:364` inner_context derive), §10.7 (`lib.rs:379-382` App composition), §10.8 (Case A error shape), §10.9 (fix-impl-6 Part A), §10.10 (fix-impl-6 Part B).

**GOTCHA 1 — single anchored Edit on e2e.rs (`feedback_junior_worker_e2e_edit_hang.md`):** `e2e.rs` is ~14,981 lines at HEAD. Multiple Edits or full-file Edit operations hang Junior workers (DQ #117 + many retros). The impl-task **MUST**:
- Locate the fn signature line via `grep -nE "^async fn agpl_source_disclosure_surface_returns_notice"` BEFORE the Edit.
- Locate the function's closing `}` line (the second-to-last `}` between the fn opening `{` and the next `#[tokio::test]` / EOF — at HEAD = `:14981`).
- Pass the Edit tool `old_string` = the **entire body between (but not including) the signature opening `{` and the closing `}`** — anchored on the signature's opening brace line AND the closing brace line so the replacement is unambiguous.
- Pass `new_string` = the verified-correct body above (~120 lines).
- **ONE Edit call.** Not two; not a Write of the whole file; not a sequence of small Edits.
- If the Edit tool rejects the operation (e.g. `old_string not unique`), enlarge the anchor by including the signature line + the trailing `}` line in both `old_string` and `new_string` (so the entire fn region is the unit-of-replacement). Do NOT try to Edit-by-line.

**GOTCHA 2 — verify the type-precision rule (R11 / DQ #261).** Before commit, the impl-task **MUST** verify (via `grep` + visual inspection):
- The line `.app_data(Data::new(inner_context.clone()))` uses `inner_context` (a `LemmyContext` from `federation_config.deref().clone()`), NOT `context` (the bootstrap's `Data<LemmyContext>`). Wrong substitution -> `Data<Data<LemmyContext>>` -> HTTP 500 (the proven-failed shape).
- The `.wrap(SessionMiddleware::new(inner_context.clone()))` uses `inner_context`, NOT `(**context).clone()` (fix-impl-7b's variant). Both compile; mirror-fidelity dictates `inner_context` (lib.rs:382 mirror uses the same `context` var that's `LemmyContext` per the let-binding at `:364`).
- The `.wrap(FederationMiddleware::new(federation_config.clone()))` uses `federation_config` (the binding from `.build().await?`), NOT a re-built second config.

Commit body MUST quote `config.rs:264-270` (`Deref<Target=T>`) + `lib.rs:364` + `lib.rs:379` as the source citations for the chosen mechanism, per R11 + DQ #261 + `feedback_read_canonical_before_writing_spec.md`.

**GOTCHA 3 — `use std::ops::Deref;` MUST be in the fn body.** Per `crates/server/src/lib.rs:56` the `Deref` trait is imported via `use std::{ops::Deref, time::Duration};` at file scope. The e2e test imports its dependencies INLINE inside the test fn body (per the e2e.rs convention — e.g. `:14866-14879` does this). Add `use std::ops::Deref;` to the inline import block. WITHOUT this `use`, the `.deref()` method call on `FederationConfig<LemmyContext>` will fail to resolve.
- **Equivalent alternative** (compiles without the `use`): `(*federation_config).clone()` (auto-deref). The plan picks the explicit `.deref().clone()` because it byte-mirrors `lib.rs:364`. Both compile to the same code.

**GOTCHA 4 — dual-bootstrap resolution (DQ #226):** call `governance_fixtures::bootstrap()` at `crates/server/tests/e2e.rs:801`, NOT `admin_config_fixtures::bootstrap()` at `:5665`. Already correct in the failed test code (`:14881` calls `governance_fixtures::bootstrap()`) — preserve.

**GOTCHA 5 — test fn name uniqueness:** the test name `agpl_source_disclosure_surface_returns_notice` MUST remain unique. Pre-Edit verification:

```bash
grep -cE "async fn agpl_source_disclosure_surface_returns_notice" crates/server/tests/e2e.rs
# EXPECT: 1 (the single existing fn at :14865; Task 6 replaces its BODY, not the fn name)
```

If 0 or >1, file `kind: "blocker"` DQ — the file structure changed in a way the plan didn't anticipate.

**GOTCHA 6 — order of Edit verification:** after the Edit, the impl-task runs `git diff crates/server/tests/e2e.rs | grep -cE '^\+\s+\.app_data\(Data::new\(inner_context\.clone\(\)\)\)'` and expects `1`. Counts of 0 or 2+ surface a malformed Edit before commit.

**VALIDATE — §5.2-laptop Phase 1 (per §0.2 / Shape G suspended per DQ #229):**

```bash
# All three commands run on the laptop canonical checkout via the bat wrapper;
# advisor mutates the validate-pending-laptop DQ entry per §15 below.

# cmd1: workspace check
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-ship-1-r2-task6-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-1-r2-task6-check.log
# EXPECT: exit 0

# cmd2: clippy
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-ship-1-r2-task6-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-1-r2-task6-clippy.log
# EXPECT: exit 0

# cmd3: test target compile (e2e — per DQ #259, NO --features full; -p lemmy_server is correct)
cmd //c "scripts\\brehon\\cargo-test.bat --no-run -p lemmy_server --test e2e > .claude/PRPs/debug/v1-ship-1-r2-task6-test-no-run.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-1-r2-task6-test-no-run.log
# EXPECT: exit 0
```

**VALIDATE — §5.2-laptop Phase 2 (advisor-side, after worker-branch finalize-merges into `phase-v1-ship-1`):**

```bash
# advisor raises kind: "validate-pending-laptop-e2e" DQ with this command;
# bg cargo run; advisor mutates DQ on E2E_EXIT marker.
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-v1-ship-1-r2-<sha>.log 2>&1 && echo E2E_EXIT_0 >> .claude/runlog/e2e-v1-ship-1-r2-<sha>.log || echo E2E_EXIT_NONZERO >> .claude/runlog/e2e-v1-ship-1-r2-<sha>.log"
# EXPECT: log tail contains "E2E_EXIT_0" + the line "test agpl_source_disclosure_surface_returns_notice ... ok"
# Pre-existing e2e test count + pass count unchanged.
```

**COMMIT (Junior finalize):** `test(e2e): rebuild agpl_source_disclosure_surface_returns_notice App on canonical lib.rs:364 idiom (task 6)` — body cites the source evidence per R11 + DQ #261 + GOTCHA 2 + `feedback_read_canonical_before_writing_spec.md`. Include `LESSON:` trailer + `HANDOVER:` YAML per `feedback_handover_trailer_cohort_propagation.md`.

### Task 7: Retro

**Goal:** author retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`. One H2 per role (Advisor / Planning / Impl / BM) with signals + lessons. Promote any new lessons to `.claude/lessons/feedback_*.md` in the same retro commit (per `feedback_one_system_memory_in_repo.md`).

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-ship-1-r2-retro.md
modifies: []
requires:
  - task: 6
    reason: retro reflects on the rebuild + Phase-2 e2e green signal
```

> If the retro proposes new durable lessons, the same commit adds `.claude/lessons/feedback_*.md` files + updates the lessons index per `feedback_one_system_memory_in_repo.md`; expand `creates:` and `modifies:` accordingly. Default shape above is the minimum.

**Three required H2 sections** (per `feedback_retro_not_report.md`):

```markdown
# v1-ship-1-r2 retro

## What surprised us
- (per-role signals; one bullet per role at minimum — Advisor / Planning / Impl / BM)

## What to change
- (concrete deltas: rule edits, lesson promotions, brief-template tweaks)

## What to carry forward
- (patterns + decisions that worked; cite prior-phase recurrence if applicable)
```

**Suggested signals to harvest (planner pre-seeds):**

- **Did the "brief Option recipe vs source evidence" reconciliation save a re-plan cycle?** If the impl shipped clean on Task 6's first §5.2-Phase-2 attempt -> confirm the planner-DQ #261 mechanism. Promote: a new lesson `feedback_actix_data_vs_federation_data_in_e2e.md` capturing the type-precision rule (R11 in this plan).
- **Did the single anchored Edit land in one tool call without worker-hang?** Confirm `feedback_junior_worker_e2e_edit_hang.md` discipline.
- **Did the §5.2-laptop Phase-2 cargo run finish under ~30 min?** Confirm `feedback_windows_e2e_requires_bat_wrapper.md` discipline; flag long runs in the §5 budget for r3 / future ship plans.
- **Did the user gate 1 reject the brief-vs-source mechanism call (preferring `init_test_context()` Option (a) or some other path)?** If yes -> the planner's confidence in the verified-correct mechanism was misplaced; capture as a lesson on "when to file `kind: 'blocker'` vs `kind: 'log'` for brief-mechanism deviations."
- **Did Probes 9-12 catch any drift in canonical mirrors (lib.rs:364 / read.rs:1 / context.rs:97 / Cargo.lock)?** If yes -> the §3 R10 discipline is paying off; promote.
- **5-attempt failure analysis worth surfacing:** the 5 fix-impl cycles all added MORE middleware; none challenged the fundamental `.app_data(Data::new(context.clone()))` line. Capture as a lesson on "type-precision in actix_web extractors when the source type is already a Data wrapper" (the actix Arc-wrapping invariant).

**No commit at retro draft** — the BM subagent commits the retro after user sign-off (per `.claude/rules/advisor-orchestrator.md` user gate 6).

---

## 14. Testing strategy

Layer-by-layer (Shape G suspended per DQ #229; ALL cargo invocations local-laptop via the bat wrapper):

- **Unit (compile-time):** `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full"` — runs after every Task 6 push (impl-task subagent triggers; advisor-laptop mutates DQ on exit).
- **Lint:** `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"` — same.
- **Test target compile:** `cmd //c "scripts\\brehon\\cargo-test.bat --no-run -p lemmy_server --test e2e"` — per DQ #259, NO `--features full`; `-p lemmy_server` is correct.
- **e2e execution:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"` — Phase-2, advisor-driven after daemon finalize-merge of Task 6 worker branch into `phase-v1-ship-1`. Expected: the named test + all pre-existing tests PASS.
- **Migration round-trip:** N/A — no schema changes.

User-gate 4 (Phase 2 e2e — local vs dispatch — per `.claude/rules/advisor-orchestrator.md` §3.2): defaults to LOCAL per `feedback_laptop_default_for_validate_pending.md` + DQ #229 suspension. If the user opts for dispatch (only valid IF Shape G is re-enabled per DQ #229's date), the advisor would `gh workflow run cargo-test-e2e.yml --repo barrie-cork/lemmy --ref phase-v1-ship-1`; until 2026-06-01, dispatch is NOT an option.

---

## 15. Validation commands (DoD)

> **Planner-side discipline (per `feedback_plan_dod_dry_run_at_write.md` + `feedback_pre_phase_dod_smoke_test.md`):** every command below dry-runs at advisor §3.4 DoD smoke test (user gate 1) before the impl-task is queued. Unexecutable commands (missing `--features full`, missing `--no-deps`, `-p <crate>` + `--features full` per `feedback_features_full_p_crate_incompatible.md`, wrapper-script flag silence per Task 0 Probe 14-15) are advisor-side rejection grounds.
>
> **§5.2-laptop shape (Shape G suspended per DQ #229 until 2026-06-01; cite §0.2 of the brief + DQ #229 as the authority for the shape choice).**

### 15.1 Static analysis (per task — workspace check)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-ship-1-r2-task6-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-1-r2-task6-check.log
# EXPECT: exit 0
```

### 15.2 Lint (per task — uniform R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-ship-1-r2-task6-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-1-r2-task6-clippy.log
# EXPECT: exit 0
```

### 15.3 Test target compile (R7 — Task 6 touches the test target)

```bash
# Per DQ #259: -p lemmy_server --test e2e (NO --features full; -p + --features full is incompatible per
# feedback_features_full_p_crate_incompatible.md; lemmy_server has no "full" feature).
cmd //c "scripts\\brehon\\cargo-test.bat --no-run -p lemmy_server --test e2e > .claude/PRPs/debug/v1-ship-1-r2-task6-test-no-run.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-1-r2-task6-test-no-run.log
# EXPECT: exit 0
```

### 15.4 e2e test execution (Task 6 — Phase 2, advisor-driven after finalize-merge into phase-v1-ship-1)

```bash
# Advisor raises kind: "validate-pending-laptop-e2e" DQ for the post-finalize-merge phase tip,
# then runs (bg) on the canonical laptop checkout:
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-v1-ship-1-r2-<sha>.log 2>&1 && echo E2E_EXIT_0 >> .claude/runlog/e2e-v1-ship-1-r2-<sha>.log || echo E2E_EXIT_NONZERO >> .claude/runlog/e2e-v1-ship-1-r2-<sha>.log"
# EXPECT: log tail "E2E_EXIT_0" + the line "test agpl_source_disclosure_surface_returns_notice ... ok"
# Pre-existing e2e test count + pass count unchanged.
```

Never bare `cargo test` on Windows (libpq.dll needs the bat wrapper's vcpkg PATH setup); never `-p lemmy_server --features full` (`lemmy_server` has no `full` feature; use `--workspace`) — per `feedback_windows_e2e_requires_bat_wrapper.md`.

### 15.5 Cross-cutting verification

Bulleted checklist the planner asserts hold at end-of-phase:

- [ ] **No edits to files outside §11 list** (single file `crates/server/tests/e2e.rs`). Verify: `git diff --stat phase-v1-ship-1...HEAD` lists ONLY `crates/server/tests/e2e.rs` + `.claude/PRPs/reports/v1-ship-1-r2-retro.md` (+ optional lesson promotions). No edits to `crates/api/api_crud/`, no edits to `crates/api/api/src/site/source.rs`, no edits to `crates/api/routes/src/lib.rs`, no edits to `crates/db_views/site/src/api.rs`, no edits to `crates/api/api_crud/build.rs`.
- [ ] **R1** (i64::from): no `as` casts in the new test body (none introduced per §10.7 verbatim).
- [ ] **R5** (Task 0 enumerated all 16 probes).
- [ ] **R6** (clippy `--no-deps -- -D warnings` uniform).
- [ ] **R7** (cargo test --no-run -p lemmy_server --test e2e after Task 6 commit).
- [ ] **R8** (Case A: fn outer `LemmyResult<()>`, bare `?` throughout — verify `rg "\.map_err\(\|e\| anyhow::anyhow!" crates/server/tests/e2e.rs` returns NO match inside the Task 6 fn region).
- [ ] **R11 — type-precision (NEW from DQ #261)** verify: `grep -nE '\.app_data\(Data::new\(inner_context\.clone\(\)\)\)' crates/server/tests/e2e.rs` returns exactly one match inside the new test fn. The grep result MUST contain the literal `inner_context.clone()` (NOT `context.clone()`).
- [ ] **Canonical mirror fidelity** verify: `grep -nE 'let inner_context: LemmyContext = federation_config\.deref\(\)\.clone\(\);' crates/server/tests/e2e.rs` returns exactly one match (the lib.rs:364 byte-mirror).
- [ ] **Use Deref trait** verify: `grep -nE 'use std::ops::Deref;' crates/server/tests/e2e.rs` returns at least one match inside the Task 6 fn body. (Equivalent: the impl-task chose the `(*federation_config).clone()` syntactic alternative per §10.6 / GOTCHA 3 — both are accepted; the cross-cutting check is "no method-resolution error on `.deref()`".)
- [ ] **fix-impl-6 Part A preserved**: both `assert_eq!(..., 200, "... — body: {}", String::from_utf8_lossy(&...))` patterns present (one for `/api/v4/site`, one for `/api/v4/source`).
- [ ] **fix-impl-6 Part B preserved**: `SiteInsertForm { ap_id: Some(...), ..., last_refreshed_at: Some(chrono::Utc::now()), inbox_url: Some(...), private_key: Some(...), public_key: Some(...), ..SiteInsertForm::new(...) }` present.
- [ ] **bootstrap selection (DQ #226)**: the test calls `governance_fixtures::bootstrap()` — verify `rg "governance_fixtures::bootstrap\(\)\.await\?" crates/server/tests/e2e.rs` includes the new test's region; `rg "admin_config_fixtures::bootstrap\(\)" crates/server/tests/e2e.rs` does NOT.
- [ ] **AGPL-NOTICE.md unchanged** — no edits this plan; `git diff --stat phase-v1-ship-1...HEAD AGPL-NOTICE.md` is empty.

### 15.6 §5.2-laptop DoD (binding — Shape G suspended per DQ #229)

- **Phase 1 (Task 6 worker push):** advisor raises `kind: "validate-pending-laptop"` DQ with the three commands above (15.1+15.2+15.3); advisor-laptop runs each sequentially; mutates DQ entry per `.claude/rules/decision-queue.md` "validate-pending-laptop handler".
- **Phase 2 (post finalize-merge into `phase-v1-ship-1`):** advisor raises `kind: "validate-pending-laptop-e2e"` DQ with the §15.4 command; advisor-laptop runs (bg); mutates on `E2E_EXIT_0` / `E2E_EXIT_NONZERO` marker. On success -> user gate 5 (merge confirm). On fail -> §G4 classifier (likely Case 4c hard-refusal if a 3rd same-tuple `(actix-Data-500, e2e.rs)` reproduces).
- **Cycle-count meta-rule (`advisor-orchestrator.md` §5.3):** if Task 6's first Phase-2 attempt fails AND the failure log slice contains the same `actix-Data-500 e2e.rs` tuple, the advisor's cycle-count for this (error_class, file) tuple becomes **3** (counting fix-impl-7 + fix-impl-7b + fix-impl-8 as cycles 1-2 + this attempt as cycle 3) -> HARD REFUSAL — surface to user as catch-fire; do NOT auto-queue another fix-impl. The plan's §10.7 + DQ #261 + R11 are the source-cited basis for why the chosen mechanism IS correct; a 3rd same-tuple fail would invalidate the planner's confidence and require user override.

---

## 16. Acceptance criteria

- [ ] All 3 §13 tasks completed in dependency order (Task 0 audit, Task 6 impl, Task 7 retro).
- [ ] §15.1 (cargo check workspace) exit 0 after Task 6 push.
- [ ] §15.2 (cargo clippy `--no-deps -- -D warnings`) exit 0 after Task 6 push.
- [ ] §15.3 (cargo test --no-run -p lemmy_server --test e2e) exit 0 after Task 6 push.
- [ ] §15.4 (e2e --workspace --features full) — `agpl_source_disclosure_surface_returns_notice` passes; pre-existing e2e test count + pass count unchanged.
- [ ] §15.5 cross-cutting verification — all 12 boxes ticked.
- [ ] §16a Story 3 `[done]`.
- [ ] No edits to files outside §11 list (verified by `git diff --stat phase-v1-ship-1...HEAD`).
- [ ] Retro committed per Task 7.
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy` (Tasks 1-4 + Task 6 + Task 7 all on the same `phase-v1-ship-1` branch).
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`.
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-ship-1-r2-verify.md` shows Story 3 ✓.
- [ ] **Manual smoke (advisor-side, post-merge, optional):** `curl http://localhost:8536/api/v4/site | jq .source_disclosure` returns the four-field block; `curl http://localhost:8536/api/v4/source | jq .notice | head` returns notice text. Verifiable by an external observer with no project context (PRD §7.1 ship-criteria).

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: First-touch handshake exposes source-disclosure pointer — SATISFIED (merged on phase-v1-ship-1)

- **Composing tasks:** Task 1 (DTOs — merged), Task 2 (field + handler populate + build.rs — merged).
- **Checkpoint command:** N/A — Phase-1 §5.2 commands are PASSING on phase tip (DQ #254 evidence). Type-correct; the field is populated.
- **Brief-Scope outputs to verify (`/brehon-verify`):**
  - `crates/db_views/site/src/api.rs` contains `pub struct SourceDisclosure`.
  - `crates/db_views/site/src/api.rs` contains `pub source_disclosure: SourceDisclosure` inside `GetSiteResponse`.
  - `crates/api/api_crud/src/site/read.rs` constructor literal contains `source_disclosure: SourceDisclosure {`.
  - `crates/api/api_crud/build.rs` exists and contains the literal `"cargo:rustc-env=BREHON_FORK_COMMIT"`.
  - `crates/api/api_crud/src/site/read.rs` contains `const BREHON_FORK_COMMIT: &str = env!("BREHON_FORK_COMMIT");`.

> **Story 1 status: merged @ `92c4cc190` (phase-v1-ship-1 tip). No re-impl. The e2e-side ASSERTION of this surface lives in Story 3.**

### Story 2: Disclosure URL resolves to the AGPL notice body — SATISFIED (merged on phase-v1-ship-1)

- **Composing tasks:** Task 3 (handler + route + mod wiring — merged).
- **Checkpoint command:** N/A — Phase-1 §5.2 commands are PASSING on phase tip.
- **Brief-Scope outputs to verify (`/brehon-verify`):**
  - `crates/api/api/src/site/source.rs` exists and contains `pub async fn get_source`.
  - `crates/api/api/src/site/source.rs` contains `include_str!("../../../../../AGPL-NOTICE.md")`.
  - `crates/api/api/src/site/mod.rs` contains `pub mod source;`.
  - `crates/api/routes/src/lib.rs` contains `.route("/source", get().to(get_source))`.
  - `crates/api/routes/src/lib.rs` contains `source::get_source,` inside the `lemmy_api::site::{...}` use-block.

> **Story 2 status: merged @ `92c4cc190` (phase-v1-ship-1 tip). No re-impl. The e2e-side ASSERTION of this surface lives in Story 3.**

### Story 3: Named e2e test covers both surfaces with Case A discipline AND the verified-correct App-construction — THE ONLY LIVE STORY

- **Composing tasks:** Task 6 (the e2e App-construction rebuild).
- **Checkpoint command:**

```bash
# After Task 6 commit + worker-branch finalize-merge into phase-v1-ship-1:
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full -- agpl_source_disclosure_surface_returns_notice --exact > .claude/runlog/story3-checkpoint-v1-ship-1-r2.log 2>&1"
echo "exit: $?"
tail -30 .claude/runlog/story3-checkpoint-v1-ship-1-r2.log
```

- **Expected output:** exit 0; log contains `test agpl_source_disclosure_surface_returns_notice ... ok` and `test result: ok. 1 passed; 0 failed`.

- **Brief-Scope outputs to verify (`/brehon-verify`):**
  - `crates/server/tests/e2e.rs` contains `async fn agpl_source_disclosure_surface_returns_notice() -> lemmy_utils::error::LemmyResult<()>` (signature preserved).
  - The new test body contains the literal `let federation_config = FederationConfig::builder()` (the §10.5 mirror).
  - The new test body contains the literal `let inner_context: LemmyContext = federation_config.deref().clone();` (the §10.6 lib.rs:364 byte-mirror).
  - The new test body contains the literal `.app_data(Data::new(inner_context.clone()))` (the §10.7 lib.rs:379 mirror — the FIX).
  - The new test body contains `.wrap(FederationMiddleware::new(federation_config.clone()))` (lib.rs:380 mirror).
  - The new test body contains `.wrap(IdempotencyMiddleware::new(idempotency_set.clone()))` (lib.rs:381 mirror).
  - The new test body contains `.wrap(SessionMiddleware::new(inner_context.clone()))` (lib.rs:382 mirror).
  - The new test body contains BOTH `test::TestRequest::get().uri("/api/v4/site")` AND `test::TestRequest::get().uri("/api/v4/source")` calls (the two surfaces asserted).
  - The new test body contains `governance_fixtures::bootstrap()` (DQ #226 — the chosen bootstrap).
  - The new test body does NOT contain `admin_config_fixtures::bootstrap` (DQ #226 — REJECTED sibling).
  - The new test body does NOT contain `.to_request_data()` used as the `.app_data(...)` source (DQ #261 — type-mismatch).
  - The new test body uses Case A discipline (outer `LemmyResult<()>`, no `Box<dyn Error>`, no `.map_err(|e| anyhow::anyhow!(...))?` bridges).

> **Verification mapping:** the advisor's `/brehon-verify` step iterates this section, runs the Story 3 Checkpoint against the worktree branch, and confirms each Brief-Scope output exists + matches its structural pattern. Phantom (test fn present but App-construction still on the old failed shape) triggers the catch-fire procedure in `.claude/rules/advisor-orchestrator.md`.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (all 16 probes confirmed).
- [ ] Task 6 committed (single anchored Edit; one commit; preserves Part A + Part B verbatim).
- [ ] Task 7 retro committed (per `feedback_retro_not_report.md` shape).
- [ ] §15 validation green (Phase-1 trio + Phase-2 e2e all PASS).
- [ ] §16a Story 3 `[done]`; Stories 1-2 SATISFIED (merged context).
- [ ] PR opened by BM session against `governance-v0` with `--repo barrie-cork/lemmy` (PR carries the full v1-ship-1 commit history: Tasks 1-4 + Task 6 + Task 7).
- [ ] CodeRabbit review complete; findings triaged.
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-ship-1-r2-verify.md` shows Story 3 ✓.
- [ ] Post-merge `phase-v1-ship-1` branch retained for retro reads.

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Type-precision fix doesn't resolve the HTTP 500 (3rd same-tuple `actix-Data-500 e2e.rs`) | LOW | HIGH (BLOCKS SHIP) | §10.7 + DQ #261 source-cite the exact mechanism; §15.6 cycle-count meta-rule forces catch-fire on a 3rd attempt rather than auto-queueing fix-impl-9; advisor user gate 5 (merge confirm) is a manual barrier |
| Impl-task picks `(*federation_config).clone()` syntactic form vs explicit `.deref().clone()` | LOW | LOW | Both compile to the same code; §10.6 GOTCHA 3 lists both as acceptable; only structural mirror-fidelity matters |
| Impl-task forgets `use std::ops::Deref;` inline import -> compile error | LOW | LOW | §10.6 GOTCHA 3 explicitly calls out the import; Phase-1 cmd1 (cargo check) catches it loudly |
| Impl-task accidentally drops fix-impl-6 Part A (body-on-failure asserts) | LOW | MED | §10.9 + §15.5 cross-cutting verify both check; the Edit's verbatim body includes both asserts |
| Impl-task accidentally drops fix-impl-6 Part B (complete `SiteInsertForm` seed) | LOW | MED | §10.10 + §15.5 cross-cutting verify both check; the Edit's verbatim body includes the full seed |
| e2e.rs line numbers drift (rebase / cohort A/B work landed) between plan-write and impl | MED | LOW | MIRROR refs cite symbol + line; impl-task re-greps before the Edit; Probe 8 confirms fn signature symbol present |
| Junior worker hangs on the e2e.rs Edit (~14,981 lines) | LOW | MED | GOTCHA 1 mandates single anchored Edit; `feedback_junior_worker_e2e_edit_hang.md` discipline; the replacement is ~120 lines, well under any hang threshold |
| Sibling test (e.g. another lane's PR) modifies `e2e.rs` post-plan-write -> Probe 13 fires | LOW | LOW | Probe 13 surfaces the conflict; impl-task STOPS with `kind: "blocker"` DQ for coordination |
| `governance_fixtures::bootstrap()` testcontainer cold-start ~30s on full e2e suite | LOW | LOW | Existing pattern; CI uses `--test-threads=1` via the wrapper; the new test reuses the same bootstrap as dozens of sibling governance tests |
| `clippy::doc_lazy_continuation` fires (new docs) | NIL | NIL | No new doc-comments in Task 6; the body is implementation, not type-doc |
| Brief Option (a) `init_test_context()` is the right path and I missed it | LOW | LOW | §10.7 documents why Option (a) is also rejected (same type-mismatch as `to_request_data()` since `init_test_context()` itself returns `activitypub_federation::config::Data<LemmyContext>`); user gate 1 (plan approval) re-validates |
| User gate 1 rejects the planner's DQ #261 mechanism-precision call | LOW | MED | The plan §10.7 + §19 fully source-cite the rationale; if user prefers another mechanism, the planner is wrong and the brief's spirit was correct in a way I didn't see — user-relay resolves |
| §5.2-laptop Phase-2 e2e exceeds ~30 min wall clock | LOW | LOW | Existing pattern; user-gate 4 is local (per `feedback_laptop_default_for_validate_pending.md`); cargo warm cache reduces re-runs |
| The `Cargo.lock` activitypub_federation version drifts (different to-be-released beta) between plan-write and impl | LOW | MED | Probe 12 confirms version; if drifted, the impl-task STOPS with `kind: "blocker"` DQ — re-verify DQ #258 against the new version's `config.rs:143` signature |
| Cross-lane file-ownership conflict on `e2e.rs` appears mid-impl | LOW | LOW | DQ #260 RESOLVED (no current conflict); Probe 13 re-checks at impl-time; the Edit is anchored to ONE fn so most cross-lane appends in other regions don't conflict |
| Upstream Lemmy rebase moves `lib.rs:364`'s pattern | LOW | MED | Probe 9 confirms the mirror anchor present at impl-time; if drifted, the impl-task re-derives the pattern from the current `crates/server/src/lib.rs` and surfaces in retro (no DQ unless the pattern itself changed semantically) |

---

## 19. Notes

- **Re-plan rationale.** The surface is settled and merged; the ONLY residual defect is one line of test App-construction. The brief's §0.1 investigation found 3 of 4 facts correctly but Finding 3 misidentified the `Data<T>` type the failed line registers vs the type the handler extracts. The plan corrects that single mechanism precision while preserving the brief's entire spirit (mirror real-server composition, keep `governance_fixtures::bootstrap()`, preserve fix-impl-6 Part A + Part B, change nothing else).

- **The DQ #261 mechanism-precision call — the load-bearing decision in this plan.** Brief §0.1 Finding 3 claims `to_request_data()` produces the `Data<LemmyContext>` that `get_site` extracts. Source at current tip refutes this:
  - `read.rs:1` — `use actix_web::web::{Data, Json};` -> `get_site`'s `Data<LemmyContext>` is **actix Data**.
  - `~/.cargo/registry/.../config.rs:143` — `pub fn to_request_data(&self) -> Data<T>` where `Data` at `:335` is `pub struct Data<T: Clone> { config: FederationConfig<T>, request_counter: RequestCounter }` -> **NOT** actix Data.
  - `lib.rs:247` (`let request_data = federation_config.to_request_data();`) is used at `:249` (`handle_outgoing_activities(request_data.clone())`) + `:253` (`scheduled_tasks::setup(request_data.clone())`) — those tasks consume `activitypub_federation::config::Data<T>`. The actix App at `:368-386` does NOT use `request_data`; it derives `let context: LemmyContext = federation_config.deref().clone();` at `:364` and wraps via `Data::new(context.clone())` at `:379`.
  - Therefore the canonical "single source of truth" mechanism is `federation_config.deref().clone()`, NOT `to_request_data()`. This is also what brief Option (b) preamble means by "context-out-of-config" (line in §0.1.1: "mirrors `lib.rs:241->247->364` exactly: build config from the context, then context-out-of-config") — the "context-out-of-config" step is `lib.rs:364`, which uses `.deref().clone()`. Brief's later sentence ("the fix is that the `.app_data` `Data<LemmyContext>` is now `federation_config.to_request_data()`") contradicts the same paragraph's preamble. The plan honors the preamble.

- **Why this isn't a `kind: "blocker"` DQ (and IS a `kind: "log"` DQ).** Per brief §4.2 the planner should file a blocker when "the investigation's Finding 1-4 appear contradicted by what you Read at current tip." Finding 3 IS contradicted on the precise mechanism, but the brief's SPIRIT ("mirror real-server single-source-of-truth FederationConfig composition") IS achievable with a verified-correct mechanism. Filing a blocker would just kick a same-direction decision up to the user; filing a log + writing the plan with a clear source-cited rationale lets user gate 1 review the deviation as a normal plan-approval gate. If user gate 1 rejects -> retro signal + re-plan (legitimate failure mode; cost ~1 cycle). If user gate 1 accepts -> ship-gate cleared with minimal advisor friction.

- **Why NOT just `.app_data(context.clone())` (drop `Data::new()`).** Technically would work as a one-token fix (the bootstrap's `context` is already `Data<LemmyContext>`); registers correctly under TypeId. **REJECTED** because:
  1. The bootstrap's `Data<LemmyContext>` was built from a `LemmyContext` that has NOT been routed through any `FederationConfig`. Production at `lib.rs:364+379` derives BOTH the actix `Data<LemmyContext>` and the `FederationMiddleware`'s inner `FederationConfig` from one common deref — making the "single source of truth" invariant structural rather than coincidental. `feedback_read_canonical_before_writing_spec.md` says mirror the canonical composition, not a divergent variant.
  2. The brief's spirit (and Option (b) preamble): "mirrors `lib.rs:241->247->364` exactly."

- **Why NOT brief Option (a) (`init_test_context()`).** Same type-mismatch as `to_request_data()`: `init_test_context()` calls `config.to_request_data()` (`context.rs:99`) which returns `activitypub_federation::config::Data<LemmyContext>` (per `:2` import). Same TypeId-mismatch with `get_site`'s extractor. Per DQ #257: Option (a) would also work for the **testcontainer + seed** angle if `LEMMY_DATABASE_URL` is set first (e.g. by calling `governance_fixtures::bootstrap()` for the testcontainer side and `init_test_context()` for the App side), but it doesn't fix the type-mismatch. Option (a) is unsalvageable for an HTTP-route test against the merged `get_site`.

- **The 5-attempt failure was a structural-thinking miss, not a mechanical one.** Every fix-impl attempt added MORE middleware (FederationMiddleware, IdempotencyMiddleware, SessionMiddleware) or changed the SessionMiddleware arg shape (`context` -> `(**context).clone()`). None challenged the fundamental line `.app_data(Data::new(context.clone()))` because the type-double-wrapping is invisible at the type-checker level (it compiles fine; failure is dynamic at extractor time). The fix is one token: `context` -> `inner_context`. R11 + a future lesson `feedback_actix_data_double_wrap_with_bootstrap.md` would capture this generalisably.

- **Decision-queue pre-seed (planner -> advisor):** DQ #261 (`kind: "log"`, `from: "planner"`, `answered_by: "planner-self-resolved"`). The planner's mechanism-precision finding is documented and surfaces at retro. Advisor validates at user gate 1.

- **Lesson trailer candidates (for Task 7 retro harvest):**
  - "actix `Data<T>` double-wrap when source is already a `Data<T>` — `.app_data(Data::new(<existing Data>))` silently produces `Data<Data<T>>`." Promote to `feedback_actix_data_double_wrap_with_bootstrap.md`.
  - "Brief Option recipes describe SPIRIT (mirror X), not LITERAL APIs — when the precise API named in the recipe is mechanically wrong at current tip, file `kind: 'log'` DQ + write plan with verified-correct mechanism." Promote to `feedback_brief_recipe_vs_source_mechanism.md`.
  - "When a test fails 5x with the same dynamic-extraction error, search FIRST for the type-level invariant being broken (e.g. `Data<Data<T>>`), not for missing middleware. Source: the v1-ship-1 5-attempt cycle." Promote to `feedback_5_attempt_extraction_error_check_type_invariant.md`.

---

## 20. Confidence score

- **Plan correctness:** 9/10 — every file:line citation confirmed by direct `Read` at worker-branch tip @ `92c4cc190`; the chosen mechanism (`federation_config.deref().clone()`) is the line-for-line mirror of production `lib.rs:364` AND it produces the correctly-typed `actix_web::web::Data<LemmyContext>` per actix's `app_data` semantics; the type-mismatch ruling-out of `to_request_data()` is source-cited at three levels (config.rs:143 signature; config.rs:335 struct def; read.rs:1 import). One residual: the plan rests on the planner's mechanism-precision call (DQ #261) — if user gate 1 rejects, the planner is wrong, cost is one re-plan cycle.
- **Cargo budget:** 8/10 — §5.2-laptop Phase-2 e2e ~25-30 min wall clock; cargo cache should be warm (Phase-1 trio just ran); EliteDesk impact zero (cargo on laptop). Forbidden-window check applies.
- **Test coverage:** 9/10 — Story 3 (the agpl e2e) covers BOTH surfaces with Case A discipline; fix-impl-6 Part A surfaces failure shape (body included on non-200); Part B's complete `SiteInsertForm` exercises the merged Task 2 field-add through the cache. The all-mvp-endpoints test at `e2e.rs:3693` separately covers non-404 for `/api/v4/site` — unchanged.

---

*Planned: 2026-05-18 (RE-PLAN-2 against `phase-v1-ship-1` HEAD `92c4cc190`; design carried verbatim from `v1-ship-1-r1.plan.md`; surface MERGED; the one residual defect — App-construction in the single acceptance e2e — rebuilt on the verified-canonical `lib.rs:228-241/364/379-382` line-for-line mirror with planner-DQ #261 mechanism precision).*
*Status: DRAFT — pending advisor §3.4 DoD smoke test, §3.5 watchpoint specificity gate, user gate 1 (plan approval).*
