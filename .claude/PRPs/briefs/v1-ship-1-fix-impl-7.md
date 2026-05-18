# v1-ship-1 — fix-impl-7 brief (wire SessionMiddleware into the AGPL e2e test App so /api/v4/site stops 500ing at actix extraction)

## 1. Role + dispatch line

`[role:impl-task]` v1-ship-1 fix-impl-7 — in `agpl_source_disclosure_surface_returns_notice` (single anchor-Edit into `crates/server/tests/e2e.rs`): add `.wrap(SessionMiddleware::new(context.clone()))` to the test `App` (mirroring the proven in-file sibling `all_mvp_endpoints_return_non_404` at e2e.rs:3857-3862) + its inner `use`, with a bounded escalation to also wrap `FederationMiddleware` ONLY if `SessionMiddleware` alone leaves `/api/v4/site` at 500. Keeps fix-impl-6 Part A + Part B unchanged.

Dispatch string (verbatim):

```
[role:impl-task] v1-ship-1 fix-impl-7 — see .claude/PRPs/briefs/v1-ship-1-fix-impl-7.md
```

## 2. Scope

### 2.1 Why this fix exists (root cause — advisor-pinned via read-only source investigation; supersedes fix-impl-6's Part-B diagnosis)

Phase-2 e2e RUN on `phase-v1-ship-1` (tip `c2a7ed971`, post-fix-impl-6) failed a **3rd consecutive time**, same surface, but fix-impl-6's **Part A (body-on-failure asserts) WORKED** and made the real error legible:

```
thread 'agpl_source_disclosure_surface_returns_notice' panicked at crates\server\tests\e2e.rs:14919:3:
assertion `left == right` failed: /api/v4/site must return 200 — body: Requested application data is not configured correctly. View/enable debug logs for more details.
  left: 500
 right: 200
test result: FAILED. 89 passed; 1 failed; 5 ignored
```

Only this test failed; **no pre-existing test regressed** (prior 89 pass, ignored=5 unchanged).

**Root cause (advisor-pinned; this OVERTURNS the incomplete-site-row hypothesis that drove fix-impl-5 + fix-impl-6 Part B):**

- `"Requested application data is not configured correctly. View/enable debug logs for more details."` is **actix-web's built-in `ErrorInternalServerError`** message, returned when a `web::Data<T>` an extractor/middleware needs is **NOT registered on the `App`**. It is **NOT** the `"Failed to construct site response: {e}"` string from `crates/api/api_crud/src/site/read.rs:32`. **The request fails at actix extraction/middleware time and never enters `read_site`.** So this is NOT a DB / seed-row / `Site`-deserialization / moka-cache / `SiteView::read_local` problem at all. The entire incomplete-site-row line of investigation (4 analysis passes + fix-impl-5 4-row scaffold + fix-impl-6 Part B complete `SiteInsertForm`) is overturned by the legible body.
- **Evidence (source-grounded):**
  - The agpl test builds its `App` (e2e.rs:14906-14912) as:
    ```rust
    let rate_limit = RateLimit::with_debug_config();
    let app = test::init_service(
      App::new()
        .app_data(Data::new(context.clone()))
        .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit)),
    ).await;
    ```
    It registers `Data<LemmyContext>` (present) but **wraps NO middleware**.
  - The **real server** App (`crates/server/src/lib.rs:364-390`) wraps a middleware stack the test omits:
    ```rust
    let context: LemmyContext = federation_config.deref().clone();
    let rate_limit = federation_config.rate_limit_cell().clone();
    App::new()
      ...
      .app_data(Data::new(context.clone()))
      .wrap(FederationMiddleware::new(federation_config.clone()))   // lib.rs:380
      .wrap(IdempotencyMiddleware::new(idempotency_set.clone()))    // lib.rs:381
      .wrap(SessionMiddleware::new(context.clone()))                // lib.rs:382
      ...
      .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit))  // lib.rs:390
    ```
  - The `/api/v4/site` route is registered by `lemmy_api_routes::config` (`crates/api/routes/src/lib.rs:224`, `get_site`) inside the `scope("/api/v4").wrap(rate_limit.message())` chain (lib.rs:219-224). `get_site` (`read.rs:24-27`) extracts `Option<LocalUserView>` + `Data<LemmyContext>`. `LocalUserView::from_request` (`crates/db_views/local_user/src/impls.rs:219-224`) reads `req.extensions()` (populated by `SessionMiddleware`), returning `IncorrectLogin` if absent — which `Option<>` swallows to `None`. The rate-limiter (`crates/utils/src/rate_limit/mod.rs:140-149`) extracts only `req.connection_info()`. So the only `web::Data<T>` gap that produces actix's "application data not configured correctly" is a `Data<T>` that the **missing middleware stack** would register/insert.
- **The proven in-file canonical pattern** is the sibling test `all_mvp_endpoints_return_non_404` (e2e.rs:3779-3862), which is in the **passing 89** set and uses the SAME `lemmy_api_routes::config(cfg, &rate_limit)`:
  ```rust
  use lemmy_routes::middleware::session::SessionMiddleware;   // e2e.rs:3809
  ...
  let app = test::init_service(
    App::new()
      .app_data(Data::new(context.clone()))
      .wrap(SessionMiddleware::new(context.clone()))           // e2e.rs:3860  ← THE missing wrap
      .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit))
  ).await;
  ```
  This sibling wraps **only `SessionMiddleware`** (no `FederationMiddleware`, no `IdempotencyMiddleware`) and its governance-route sweep passes. **Caveat (do NOT over-claim):** `all_mvp_endpoints_return_non_404`'s endpoint list is governance routes only — it does **not** itself HTTP-drive `/api/v4/site`. **The agpl test is the ONLY test in the entire e2e suite that HTTP-drives `/api/v4/site`** (grep-confirmed: the only `uri("/api/v4/site")` is e2e.rs:14915). So `SessionMiddleware` is the **proven-minimal** wrap for `lemmy_api_routes::config` routes, but whether `/api/v4/site` *specifically* also needs `FederationMiddleware` is **not** proven by any passing sibling — hence the bounded escalation in §2.2.

### 2.2 The fix — ONE anchor-Edit into `crates/server/tests/e2e.rs`, ONE commit

Edit ONLY the existing `agpl_source_disclosure_surface_returns_notice` fn (e2e.rs:14864-14951). Do NOT add a new test, do NOT touch any other test, do NOT touch any production code, do NOT touch `governance_fixtures::bootstrap`. **fix-impl-6's Part A (body-on-failure asserts at 14914-14953) and Part B (complete `SiteInsertForm` at 14888-14896) STAY EXACTLY AS THEY ARE — do NOT revert, simplify, or modify them.** This fix is purely additive: the missing middleware wrap.

#### Primary change — add `SessionMiddleware` to the test App (mirror the proven sibling e2e.rs:3857-3862)

**(a)** Add the inner `use` for `SessionMiddleware` alongside the fn's existing inner `use` block. The fn currently opens (e2e.rs:14866-14876) with:

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
```

Add **one line** to this inner `use` block (mirror e2e.rs:3809 verbatim — same path):

```rust
  use lemmy_routes::middleware::session::SessionMiddleware;
```

Place it adjacent to the other inner `use`s (e.g. immediately after `use lemmy_diesel_utils::traits::Crud;`). Do NOT add a top-of-file `use`. Confirm the exact path by reading e2e.rs:3809 — it must be byte-identical (`lemmy_routes::middleware::session::SessionMiddleware`).

**(b)** Add `.wrap(SessionMiddleware::new(context.clone()))` to the `App` builder. The fn currently builds (e2e.rs:14906-14912):

```rust
  let rate_limit = RateLimit::with_debug_config();
  let app = test::init_service(
    App::new()
      .app_data(Data::new(context.clone()))
      .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit)),
  )
  .await;
```

Restructure to insert the wrap **between `.app_data(...)` and `.configure(...)`**, mirroring the sibling at e2e.rs:3858-3862 exactly (the wrap order there is `.app_data` → `.wrap(SessionMiddleware::new(context.clone()))` → `.configure`):

```rust
  let rate_limit = RateLimit::with_debug_config();
  let app = test::init_service(
    App::new()
      .app_data(Data::new(context.clone()))
      .wrap(SessionMiddleware::new(context.clone()))
      .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit)),
  )
  .await;
```

`context` here is the `LemmyContext` handle returned by `governance_fixtures::bootstrap()` at e2e.rs:14878 (`let (_container, context, _db_url) = governance_fixtures::bootstrap().await?;`). The sibling at e2e.rs:3860 does `SessionMiddleware::new(context.clone())` with a `LemmyContext`-typed `context` — mirror that exact call (`.clone()` included; `context` is used again later in the agpl fn for `context.pool()` so the `.clone()` is required, matching the sibling).

#### Bounded escalation (explicit — ONLY if the primary change is insufficient)

After the primary change, the **advisor** runs the §5.2 Phase-2 e2e (NOT you — see §4 constraint 6: the e2e *run* is advisor-driven). If that e2e run shows the agpl test STILL returns 500 with the SAME `"Requested application data is not configured correctly"` body (i.e. `SessionMiddleware` alone was insufficient because `/api/v4/site`'s chain also needs the federation `Data`), the **advisor** will author a follow-up scoped instruction. **You do NOT pre-emptively add `FederationMiddleware` in this task** — the proven sibling shows `SessionMiddleware` is the minimal wrap for `lemmy_api_routes::config`, and adding unproven middleware speculatively risks new failure modes (a mis-constructed `FederationConfig` is itself a 500 source). Implement ONLY the primary change (SessionMiddleware). If you have strong compile-time evidence the primary change cannot work alone, raise a `kind: "blocker"` DQ (`from: "impl"`) with the evidence and STOP — do NOT improvise `FederationMiddleware` wiring.

> Advisor note (not Junior scope, recorded for the audit trail): if escalation is needed, the in-file `FederationConfig` construction precedent is e2e.rs:2361 / 4694 (`activitypub_federation::config::FederationConfig::builder()...`) and the real-server wrap is `crates/server/src/lib.rs:380` (`FederationMiddleware::new(federation_config.clone())`, `FederationMiddleware` from `activitypub_federation::config`). That would be a separate fix-impl-8 under a fresh explicit user §G4 override — NOT this task.

**Boundaries:**

- **Commit ONLY** `crates/server/tests/e2e.rs`. No other file. (`creates: []`, `modifies: [crates/server/tests/e2e.rs]`.)
- **Do NOT** edit `crates/api/**`, `crates/db_schema/**`, `crates/db_views/**`, `crates/routes/**`, `crates/server/src/**`, or any production code. The defect is the e2e test `App` wiring ONLY. If you believe a production change is needed, STOP and raise a `kind: "blocker"` DQ (`from: "impl"`) — do NOT edit production code.
- **Do NOT** add a new test, touch any other test (including the `all_mvp_endpoints_return_non_404` sibling you are mirroring — read it, do not edit it), touch `governance_fixtures::bootstrap`, or add a new fixture helper.
- **Do NOT** revert or modify fix-impl-6's Part A (the body-on-failure assert restructure at ~14914-14953) or Part B (the complete `SiteInsertForm { ap_id: Some(...), last_refreshed_at: Some(...), inbox_url: Some(...), private_key: Some(...), public_key: Some(...), ..SiteInsertForm::new("agpl test site".to_string(), instance.id) }` at ~14888-14896). They are correct and necessary; this fix is additive only (the middleware wrap + its `use`).
- The test fn outer return stays **`lemmy_utils::error::LemmyResult<()>`** (Case A — unchanged). The new lines (`use ...SessionMiddleware;`, `.wrap(SessionMiddleware::new(context.clone()))`) introduce no new `?` and no error bridge. No `.map_err`, no Case B/C.

## 3. Required reading (read these FIRST, in order)

1. `crates/server/tests/e2e.rs:14864-14951` — the failing test as it stands now (post-fix-impl-6, with Part A + Part B already applied). This is the ONLY thing you edit; you are adding the middleware wrap + its `use`, nothing else.
2. `crates/server/tests/e2e.rs:3779-3862` — the **proven in-file canonical sibling** `all_mvp_endpoints_return_non_404`. Lines **3809** (`use lemmy_routes::middleware::session::SessionMiddleware;`) and **3857-3862** (the `App::new().app_data(...).wrap(SessionMiddleware::new(context.clone())).configure(lemmy_api_routes::config(...))` shape) are EXACTLY what you mirror. This test is in the passing 89 set with the SAME `lemmy_api_routes::config`. Mirror byte-for-byte; do NOT invent a different import path or wrap order. Do NOT edit this test.
3. `crates/server/src/lib.rs:360-395` — the real-server `App` builder. Confirms the production middleware stack (`FederationMiddleware` lib.rs:380, `IdempotencyMiddleware` lib.rs:381, `SessionMiddleware` lib.rs:382) and that `lemmy_api_routes::config` is wired after it (lib.rs:390). Context for WHY the wrap is needed; you only add `SessionMiddleware` (per the proven sibling), not the whole stack.
4. `crates/db_views/local_user/src/impls.rs:215-225` — `LocalUserView::from_request` reads `req.extensions()` (set by `SessionMiddleware`); confirms the dependency chain. Context, not a thing to edit.
5. `crates/api/api_crud/src/site/read.rs:24-32` — `get_site`; line 32's `.map_err(... "Failed to construct site response: {e}")?` is what the body WOULD show if the request reached `read_site`. The legible body does NOT contain this string, proving the request never reaches `read_site` (extraction/middleware fails first). Context.
6. `.claude/PRPs/plans/v1-ship-1-r1.plan.md` §10.6 — Case A error-shape discipline (this fix stays Case A; the new lines introduce no `?`).
7. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **§2.4 MANDATORY (e2e.rs edit).** Case A canonical. The agpl test outer is already `lemmy_utils::error::LemmyResult<()>` — keep it; this fix adds no fallible line.
8. `.claude/lessons/feedback_async_pool_test_pattern.md` — **§2.4 MANDATORY (e2e.rs edit).** Context: the existing `&mut context.pool()` block is unchanged; `context.clone()` for the middleware mirrors the sibling at e2e.rs:3860.
9. `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **§2.4 MANDATORY (e2e.rs edit; plan §13 Task 4 GOTCHA 3).** This is a SMALL additive edit (one `use` line + one `.wrap(...)` line). Use the minimum Edits (ideally ONE; at most TWO — one for the `use` block, one for the `App` builder, if they are too far apart for a single contiguous unique `old_string`). Do NOT do a full-file rewrite.
10. `.claude/lessons/feedback_read_canonical_before_writing_spec.md` — read the canonical sibling (e2e.rs:3809, 3857-3862) BEFORE editing; mirror it exactly, do not invent.

## 4. Constraints (enforce — hard refusals)

1. **Minimum Edits into e2e.rs (ideally ONE, at most TWO).** Per `feedback_junior_worker_e2e_edit_hang.md` + plan §13 Task 4 GOTCHA 3. The two change points are: (i) add `use lemmy_routes::middleware::session::SessionMiddleware;` to the fn's inner `use` block (~e2e.rs:14866-14876), and (ii) add `.wrap(SessionMiddleware::new(context.clone()))` between `.app_data(...)` and `.configure(...)` in the `App` builder (~e2e.rs:14906-14912). If a single contiguous unique `old_string` can span both (read 14866-14912 and pick the smallest unique region), use ONE Edit. Otherwise use exactly TWO Edits (one per change point). NEVER >2 Edits into e2e.rs for this task; if tempted, STOP and re-read GOTCHA 3.
2. **Mirror the proven sibling byte-for-byte.** The `use` path is exactly `lemmy_routes::middleware::session::SessionMiddleware` (e2e.rs:3809). The wrap is exactly `.wrap(SessionMiddleware::new(context.clone()))` placed between `.app_data(Data::new(context.clone()))` and `.configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit))` (e2e.rs:3859-3861). Do NOT reorder, do NOT use a different constructor, do NOT add `FederationMiddleware`/`IdempotencyMiddleware` (not in the proven sibling — speculative middleware is out of scope per §2.2).
3. **fix-impl-6 Part A + Part B are PRESERVED unchanged.** Do NOT touch the body-on-failure assert blocks or the complete `SiteInsertForm`. Verify after your edit that they are still present and byte-identical (a `git diff` of your commit must show ONLY the added `use` line + the added `.wrap(...)` line — nothing else changed in the fn).
4. **Do NOT change any assertion logic, the seed block, or `governance_fixtures::bootstrap`.** This fix adds exactly two lines (one `use`, one `.wrap`). Nothing else in the fn or file changes.
5. **Case A only.** Outer `lemmy_utils::error::LemmyResult<()>` (unchanged). The added lines introduce no `?`, no error-bridge. No Case B/C.
6. **Validation = Shape G SUSPENDED → validate-pending-laptop.** Per `.claude/rules/advisor-orchestrator.md` §5.2 + DQ #229 (Shape G suspended repo-wide until 2026-06-01). After commit + push to your worker branch, write a `kind: "validate-pending-laptop"` DQ entry (`from: "impl"`, `answered_by: null`), `phase_task: 4`, `branch: <your worker branch>`, `commands:` the §15.1–15.3 workspace commands verbatim:
   - `bash scripts/brehon/cargo-check.sh --workspace --features full`
   - `bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings`
   - `bash scripts/brehon/cargo-test.sh --no-run -p lemmy_server --test e2e`
   (the e2e *run* — §15.4, Docker/testcontainers — is a SEPARATE advisor-driven Phase-2 step; do NOT attempt the e2e run yourself.) Compute `next_id` across `.claude/decision-queue.json` pending+resolved + every `.claude/decision-queue-archive-*.json` (max+1). Current max id is **247** (DQ #247 = the user §G4 override that authorised this fix), so your validate-pending-laptop entry is **248** — verify by computing max+1 before writing; if drift, recompute (do NOT hardcode if the assert fails). Commit the DQ entry + push to your worker branch immediately (mid-task visibility — `.claude/rules/decision-queue.md`).
7. **Commit message (verbatim):** `test(e2e): wrap SessionMiddleware on agpl test App so /api/v4/site reaches read_site (mirrors all_mvp sibling e2e.rs:3860) (fix-impl-7)`. In the commit body, state: (a) the root cause (actix "application data not configured" = missing SessionMiddleware wrap, NOT a seed/DB cause — fix-impl-6 Part A made this legible), (b) that fix-impl-6 Part A + Part B are preserved unchanged, (c) the exact two lines added.
8. **DQ attribution:** `from: "impl"` only. NEVER `answered_by: "advisor"` / `"user"`. NEVER `kind: "clarify"` / `"validate-result"` / `"validate-failed"`. Per `.claude/rules/decision-queue.md` hard refusals.
9. **Mandatory post-task retro** before exit (`.claude/rules/post-task-retro.md`): `memory_write_eval`, `source_ref` = your exact worker branch name (the Stop hook on `junior/*` branches requires `source_ref` = branch + a fresh `Task retro:` row in the last 30 min). Do NOT forge `created_at`, do NOT use raw SQL, do NOT modify the hook — those bypass attempts are tracked.

## 5. §2.4 mandatory-lesson firing record (advisor audit)

Authored under `.claude/rules/advisor-orchestrator.md` §2.4. File list = `crates/server/tests/e2e.rs` (1 fn, additive middleware wrap + its `use`). Table matches fired:

- `crates/server/tests/e2e.rs` (any edit) → `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md` → §3 items 7, 8. Case A: the agpl fn outer is already `lemmy_utils::error::LemmyResult<()>`; the added lines introduce no `?` so there is no error-shape decision — Case A trivially preserved. The mirrored sibling (e2e.rs:3779-3862) is also `lemmy_utils::error::LemmyResult<()>` with the same `context.clone()` pattern — canonical confirmed by reading the sibling.
- e2e-edit-hang discipline is plan-bound (plan §13 Task 4 GOTCHA 3) regardless of edit count → `feedback_junior_worker_e2e_edit_hang.md` → §3 item 9 + §4.1. This edit is 2 lines, smallest possible.
- `feedback_read_canonical_before_writing_spec.md` → §3 item 10 (the in-file canonical is the passing sibling `all_mvp_endpoints_return_non_404` at e2e.rs:3809 + 3857-3862; mirror byte-for-byte, no invention).
- §G4 class: this is a **NON-ALLOWLIST** fail (runtime HTTP 500 at actix extraction, not E0432/deprecated/clippy-doc/LemmyError-class). The §G4 verbatim-row blockquote gate does NOT apply (allowlist-only). This is a hand-authored brief on an **explicit user §G4 override** recorded at **DQ #247** (`answered_by: "user"`, 2026-05-18): after the 3rd same-surface fail the user chose "Advisor authors a tight fix-impl brief" over a planner re-plan, because the cause is unambiguous + fully source-grounded (legible body via fix-impl-6 Part A + a proven passing in-file sibling). The override is scoped to THIS fix-impl-7 only; a 4th same-surface fail = §G4 re-plan hard-refusal (no fix-impl-8 without a fresh explicit user override).
- §2.3 PMD hybrid presearch: lane PMD DB is the per-worktree DB (lesson corpus indexed in canonical DB only — known lane-DB-isolation issue, non-blocking; §2.4 mechanical injection is the load-bearing path; lessons read from disk at `.claude/lessons/`). Canonical-schema-first satisfied: the fix mirrors the proven in-file passing sibling `all_mvp_endpoints_return_non_404` (e2e.rs:3809 + 3857-3862) — explicitly cited, byte-for-byte.
