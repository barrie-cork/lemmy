# v1-ship-1 — fix-impl-6 brief (complete-site seed + error-body asserts in the AGPL e2e test)

## 1. Role + dispatch line

`[role:impl-task]` v1-ship-1 fix-impl-6 — in `agpl_source_disclosure_surface_returns_notice` (single anchor-Edit into `crates/server/tests/e2e.rs`): (A) make the `/api/v4/site` and `/api/v4/source` status asserts print the response body on failure, and (B) replace the bare `SiteInsertForm::new(...)` with a **complete** site form mirroring the proven production path `crates/routes/src/utils/setup_local_site.rs` so `GET /api/v4/site` returns 200 instead of 500.

Dispatch string (verbatim):

```
[role:impl-task] v1-ship-1 fix-impl-6 — see .claude/PRPs/briefs/v1-ship-1-fix-impl-6.md
```

## 2. Scope

### 2.1 Why this fix exists (root cause — read first, this supersedes fix-impl-5's diagnosis)

Phase-2 e2e RUN on `phase-v1-ship-1` failed **twice** with the **identical** panic — fix-impl-5 (#300, the 4-row scaffold) did **not** fix it:

```
thread 'agpl_source_disclosure_surface_returns_notice' panicked at crates\server\tests\e2e.rs:14909:3:
assertion `left == right` failed: /api/v4/site must return 200
  left: 500
 right: 200
test result: FAILED. 89 passed; 1 failed; 5 ignored
```

Only this test failed; **no pre-existing test regressed** (the prior 89 still pass, ignored=5 unchanged).

**Root cause (advisor-pinned via 4 read-only investigation passes; this is a plan defect, NOT an impl/test-author bug):**

- fix-impl-5 added `Instance::read_or_create` + `Site::create(SiteInsertForm::new("agpl test site", instance.id))` + `Person sysacct` + `LocalSite::create` + `LocalSiteRateLimit::create`. That shape is the **canonical Lemmy db-test fixture** (`crates/db_schema/src/test_data.rs:18-43` `TestData::create` uses the exact same minimal form) — so the *shape* is correct and is NOT the defect.
- The defect: `SiteInsertForm::new` (`crates/db_schema/src/source/site.rs:51-78`) sets ONLY `name`+`instance_id`. The fields `ap_id`, `inbox_url`, `public_key`, `private_key`, `last_refreshed_at` are `#[new(default)] = None`. The `Site` struct (`site.rs:19-46`) types `ap_id: DbUrl`, `inbox_url: DbUrl`, `public_key: String`, `last_refreshed_at: DateTime<Utc>` as **non-Option**. The INSERT succeeds (DB column defaults), but the HTTP handler path — `GET /api/v4/site` → `read_site` (`crates/api/api_crud/src/site/read.rs:45-81`) → `SiteView::read_local` → `.select(Self::as_select()).first(conn).await.optional()?` (`crates/db_views/site/src/impls.rs:62-73`) — fails materialising that incomplete `site` row into the non-Option `Site` struct, the error propagates through `get_site`'s `.map_err(|e| anyhow::anyhow!("Failed to construct site response: {e}"))?` → **HTTP 500**.
- Why fix-impl-5's scaffold "passes" the db-layer tests but fails here: `TestData::create`-based tests call db functions directly; **`agpl_*` is the ONLY e2e test that HTTP-drives `/api/v4/site`** (via `test::init_service` + `test::call_service`). The plan §10.7 "mirror precedent" (`governance_outbox_emits_remote_sanction_notice_on_local_sanction`, e2e.rs:4744-4761) seeds those rows but only calls `SiteView::read_local` **internally** — it never HTTP-calls `/api/v4/site`. **A precedent that does not call the endpoint is not a valid precedent for that endpoint.** This is the true shape of the plan defect.
- The exact failing sub-call/column is still unconfirmed because **the test asserts only `status == 200` and discards the response body** — the literal `LemmyError` has been invisible across all analysis. Part A fixes that permanently; Part B fixes the cause.

The **proven** complete-site path is `crates/routes/src/utils/setup_local_site.rs:80-96` (Lemmy's real first-run setup). It builds `SiteInsertForm` with `ap_id`, `last_refreshed_at`, `inbox_url`, `private_key`, `public_key` **explicitly populated** via keypair/url helpers — exactly the non-Option fields the bare `::new` leaves `None`. **Mirror that.**

### 2.2 The fix — ONE anchor-Edit into `crates/server/tests/e2e.rs`, TWO coupled parts, ONE commit

Edit ONLY the existing `agpl_source_disclosure_surface_returns_notice` fn (e2e.rs:14864-14951). Do NOT add a new test, do NOT touch any other test, do NOT touch any production code, do NOT touch `governance_fixtures::bootstrap`.

#### Part A — body-on-failure asserts (permanent test-quality fix)

The fn currently does (e2e.rs:14906-14912):

```rust
  // --- 1. GET /api/v4/site returns source_disclosure block. ---
  let site_req = test::TestRequest::get().uri("/api/v4/site").to_request();
  let site_resp = test::call_service(&app, site_req).await;
  assert_eq!(site_resp.status().as_u16(), 200, "/api/v4/site must return 200");

  let site_body_bytes = test::read_body(site_resp).await;
  let site_body: GetSiteResponse = serde_json::from_slice(&site_body_bytes)?;
```

Problem: line 14909's `assert_eq!` panics **before** `test::read_body` runs (14911), so the real error body is discarded. Restructure so the body is read FIRST and folded into the assert message:

```rust
  // --- 1. GET /api/v4/site returns source_disclosure block. ---
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
```

Apply the **same restructure** to the `/api/v4/source` block (e2e.rs:14931-14937):

```rust
  // --- 2. GET /api/v4/source returns the AGPL notice body. ---
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
```

`test::read_body` + `serde_json::from_slice` are already used at e2e.rs:14911/14936 — this is the **in-file canonical**; do NOT add any new import. The four `source_disclosure` asserts (14914-14929) and the three `source_body` asserts (14939-14948) and the final `Ok(())` stay byte-identical — only the two status-assert blocks are restructured.

#### Part B — complete `SiteInsertForm` mirroring `setup_local_site`

The seed block is currently (e2e.rs:14885-14896):

```rust
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  {
    let pool = &mut context.pool();
    let site_form = SiteInsertForm::new("agpl test site".to_string(), instance.id);
    let site = Site::create(pool, &site_form).await?;
    // System account: throwaway Person — LocalSite needs a non-null FK.
    let sysacct_form = PersonInsertForm::test_form(instance.id, "agpl_sysacct");
    let sysacct = Person::create(pool, &sysacct_form).await?;
    let local_site_form = LocalSiteInsertForm::new(site.id, sysacct.id);
    let local_site = LocalSite::create(pool, &local_site_form).await?;
    LocalSiteRateLimit::create(pool, &LocalSiteRateLimitInsertForm::new(local_site.id)).await?;
  }
```

Replace **only** the `let site_form = SiteInsertForm::new(...)` line with a complete form built the way `setup_local_site.rs:80-96` builds it. The production code does exactly:

```rust
// crates/routes/src/utils/setup_local_site.rs:80-96 (proven reference — DO NOT copy verbatim, ADAPT per below)
let site_key_pair = generate_actor_keypair()?;
let site_ap_id = Url::parse(&settings.get_protocol_and_hostname())?;
let site_form = SiteInsertForm {
  ap_id: Some(site_ap_id.clone().into()),
  last_refreshed_at: Some(Utc::now()),
  inbox_url: Some(generate_inbox_url()?),
  private_key: Some(site_key_pair.private_key),
  public_key: Some(site_key_pair.public_key),
  ..SiteInsertForm::new(name, instance.id)
};
```

Adapt for the test context (the test has NO real `Settings`, so `settings.get_protocol_and_hostname()` is not available — use a parseable URL for the test instance domain `"test.invalid"`, which matches the `Instance::read_or_create(&mut context.pool(), "test.invalid")` on the line above):

```rust
    let site_key_pair = generate_actor_keypair()?;
    let site_ap_id = url::Url::parse("https://test.invalid")?;
    let site_form = SiteInsertForm {
      ap_id: Some(site_ap_id.clone().into()),
      last_refreshed_at: Some(chrono::Utc::now()),
      inbox_url: Some(url::Url::parse("https://test.invalid/inbox")?.into()),
      private_key: Some(site_key_pair.private_key),
      public_key: Some(site_key_pair.public_key),
      ..SiteInsertForm::new("agpl test site".to_string(), instance.id)
    };
```

Notes on the adaptation (read `setup_local_site.rs` in full FIRST to confirm exact types):

- `generate_actor_keypair` is `activitypub_federation::http_signatures::generate_actor_keypair` (setup_local_site.rs:1). Add the needed inner `use`(s) **inside the test fn** alongside the existing inner `use` block (the fn already has `use lemmy_db_schema::source::{...}; use lemmy_diesel_utils::traits::Crud;` from fix-impl-5 — add the keypair `use` next to those, NOT at top-of-file). Confirm the exact path by reading setup_local_site.rs:1.
- `SiteInsertForm.ap_id` / `.inbox_url` are `Option<DbUrl>`. The production code uses `site_ap_id.clone().into()` and `generate_inbox_url()?` (returns a `DbUrl`-convertible). For the test, `url::Url::parse(...)?.into()` converts `Url` → `DbUrl` (this is the same `.into()` the production path uses on `site_ap_id`). Verify `DbUrl: From<Url>` by reading how `setup_local_site.rs:89` does `site_ap_id.clone().into()` — mirror that exact conversion.
- `site_key_pair.private_key` / `.public_key`: `generate_actor_keypair()` returns a struct with `private_key`/`public_key`. `SiteInsertForm.private_key` is `Option<String>`, `.public_key` is `Option<String>` — the production code assigns `Some(site_key_pair.private_key)` / `Some(site_key_pair.public_key)` directly (so the keypair fields are `String`). Mirror verbatim.
- Keep `Instance::read_or_create(... "test.invalid")`, the sysacct `Person`, `LocalSite::create`, `LocalSiteRateLimit::create`, and the `{ let pool = &mut context.pool(); ... }` block structure **exactly as fix-impl-5 left them** — only the `site_form` construction changes. `Site::create` will still auto-populate `site_language` (it does this when `is_new_site`; with `ap_id: Some(...)` it now checks `Site::read_from_apub_id` — for a fresh test DB that returns `None` so `is_new_site` is still true; this is correct and matches production).

**Bounded fallback (explicit — only if the production helpers are not reachable from `crates/server/tests/`):** if `generate_actor_keypair` cannot be imported into the e2e test crate (dependency not available to `crates/server` test target) OR the `Url → DbUrl` `.into()` does not resolve, do NOT improvise broadly. Instead, populate the five fields with the **minimal valid literal values** that satisfy the non-Option `Site` columns: a parseable `ap_id`/`inbox_url` (`url::Url::parse("https://test.invalid")?.into()` etc.), a `last_refreshed_at: Some(chrono::Utc::now())`, and for the keys use whatever the smallest in-repo helper produces (search the test file + `crates/db_schema` for an existing test keypair/`public_key` helper before hand-fabricating; a non-empty placeholder `public_key`/`private_key` String is acceptable for a test row IF no helper exists). **Document in the commit body which path was taken** (production-helper-mirrored vs literal-fallback) and WHY. Prefer the production-helper path; the fallback is a deterministic floor, not a first choice.

**Boundaries:**

- **Commit ONLY** `crates/server/tests/e2e.rs`. No other file. (`creates: []`, `modifies: [crates/server/tests/e2e.rs]`.)
- **Do NOT** edit `crates/api/**`, `crates/db_schema/**`, `crates/db_views/**`, `crates/routes/**`, or any production code. Tasks 2/3 impl (read.rs / api.rs / build.rs) is verified correct — the defect is the e2e fixture + the blind assertion ONLY. If you believe a production change is needed, STOP and raise a `kind: "blocker"` DQ (`from: "impl"`) — do NOT edit production code.
- **Do NOT** add a new test, touch any other test, touch `governance_fixtures::bootstrap`, or add a new fixture helper.
- The test fn outer return stays **`lemmy_utils::error::LemmyResult<()>`** (Case A — unchanged). Every new `.await`/fallible line uses bare `?` (`generate_actor_keypair()?`, `url::Url::parse(...)?`, the existing `Instance::read_or_create(...).await?` etc.). NO `.map_err` bridges, NO Case B/C.

## 3. Required reading (read these FIRST, in order)

1. `crates/server/tests/e2e.rs:14864-14951` — the failing test as it stands now (post-fix-impl-5). This is the ONLY thing you edit.
2. `crates/routes/src/utils/setup_local_site.rs` (whole file, esp. lines 1-2, 80-96) — **the proven complete-site construction to mirror.** Note the exact `use` paths (`generate_actor_keypair`, `generate_inbox_url`, `Url`) and the `SiteInsertForm { ap_id: Some(...), last_refreshed_at: Some(...), inbox_url: Some(...), private_key: Some(...), public_key: Some(...), ..SiteInsertForm::new(name, instance.id) }` shape.
3. `crates/db_schema/src/source/site.rs:19-78` — `Site` struct (non-Option `ap_id`/`inbox_url`/`public_key`/`last_refreshed_at`) vs `SiteInsertForm` (all `#[new(default)]`). This IS the defect — confirm the field types so your form compiles.
4. `crates/db_schema/src/impls/site.rs:30-52` — `Site::create` (`is_new_site` checks `Site::read_from_apub_id` when `ap_id.is_some()`; auto `SiteLanguage::update`). Confirms a fresh-DB `Site::create` with `ap_id: Some(...)` still initialises languages.
5. `crates/db_views/site/src/impls.rs:56-78` — `SiteView::read_local`'s `.select(Self::as_select()).first(conn).await.optional()?` (the suspected materialisation-failure site on the incomplete row).
6. `crates/api/api_crud/src/site/read.rs:24-81` — `get_site`/`read_site`; the `.map_err(|e| anyhow::anyhow!("Failed to construct site response: {e}"))?` at line 32 IS the 500. Part A makes this `{e}` visible in the panic.
7. `crates/db_schema/src/test_data.rs:18-43` — `TestData::create` (proves the bare-`::new` minimal form is the canonical *db-layer* fixture, so the gap is HTTP-handler-path-only — context, not a thing to copy).
8. `.claude/PRPs/plans/v1-ship-1-r1.plan.md` §10.6 — Case A error-shape discipline (the fix stays Case A; bare `?`; no bridges).
9. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **§2.4 MANDATORY (e2e.rs edit).** Case A canonical; bare `?` throughout. The agpl test outer is already `lemmy_utils::error::LemmyResult<()>` — keep it.
10. `.claude/lessons/feedback_async_pool_test_pattern.md` — **§2.4 MANDATORY (e2e.rs edit).** `&mut context.pool()` / DbPool usage (the existing `{ let pool = &mut context.pool(); ... }` block is the pattern; keep it).
11. `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **§2.4 MANDATORY (e2e.rs edit; plan §13 Task 4 GOTCHA 3).** This must be ONE Edit, not multiple, even though it touches 3 regions of the one fn (see §4.1).
12. `.claude/lessons/feedback_read_canonical_before_writing_spec.md` — read the in-file `test::read_body` usage (e2e.rs:14911/14936) before restructuring; mirror it, do not invent.

## 4. Constraints (enforce — hard refusals)

1. **Single anchor-Edit into e2e.rs covering all three regions.** Per `feedback_junior_worker_e2e_edit_hang.md` + plan §13 Task 4 GOTCHA 3. The Edit touches THREE spots in the one fn (the `site_form` line ~14888; the `/api/v4/site` assert block ~14906-14912; the `/api/v4/source` assert block ~14931-14937). Use ONE `Edit` whose `old_string` spans from the seed block's `let site_form = SiteInsertForm::new(...)` line through the end of the `/api/v4/source` `serde_json::from_slice` line (a single contiguous region — read e2e.rs:14885-14948 and pick the smallest contiguous `old_string` that contains all three edit points and is unique), and whose `new_string` is that region with Part A (both restructured assert blocks) + Part B (complete `site_form`) applied. If you cannot make it one contiguous Edit because the regions are far apart with unrelated code between, you MAY use at most the minimum number of Edits required, but PREFER one. If you find yourself wanting >2 Edits into e2e.rs, STOP and re-read GOTCHA 3.
2. **Mirror the production path; do NOT invent field names or alternate constructors.** The `SiteInsertForm { ap_id, last_refreshed_at, inbox_url, private_key, public_key, ..SiteInsertForm::new(name, instance.id) }` shape is confirmed from `setup_local_site.rs:88-95`. If any field/type/import does not resolve at compile time, do NOT improvise beyond the §2.2 bounded fallback — and if even the fallback does not compile, raise a `kind: "blocker"` DQ (`from: "impl"`) citing the exact compile error and STOP.
3. **`Site::create` must be called with the COMPLETE form** (the existing `let site = Site::create(pool, &site_form).await?;` line stays; only `site_form`'s construction changes). Seeding stays BEFORE the `App::new()` / first `TestRequest` (it already is — the existing block placement is correct).
4. **Do NOT change any assertion's *logic*.** Part A only restructures so the body is read before the status assert and added to the panic message; the asserted condition (`status == 200`, the four `source_disclosure` field asserts, the three `source_body` asserts) stays semantically identical. Part B only changes how `site_form` is built. Nothing else in the fn changes.
5. **Case A only.** Outer `lemmy_utils::error::LemmyResult<()>` (unchanged). Bare `?` on every new fallible line. No error-bridge closures. Mixing shapes = §G4 row 4c hard refusal.
6. **Validation = Shape G SUSPENDED → validate-pending-laptop.** Per `.claude/rules/advisor-orchestrator.md` §5.2 + DQ #229 (Shape G suspended repo-wide until 2026-06-01). After commit + push to your worker branch, write a `kind: "validate-pending-laptop"` DQ entry (`from: "impl"`, `answered_by: null`), `phase_task: 4`, `branch: <your worker branch>`, `commands:` the §15.1–15.3 workspace commands verbatim:
   - `bash scripts/brehon/cargo-check.sh --workspace --features full`
   - `bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings`
   - `bash scripts/brehon/cargo-test.sh --no-run -p lemmy_server --test e2e`
   (the e2e *run* — §15.4, Docker/testcontainers — is a SEPARATE advisor-driven Phase-2 step; do NOT attempt the e2e run yourself.) Compute `next_id` across `.claude/decision-queue.json` pending+resolved + every `.claude/decision-queue-archive-*.json` (max+1) — current max is **244**, so your entry is **245** (verify before writing; if drift, recompute — do NOT hardcode if the assert fails). Commit the DQ entry + push to your worker branch immediately (mid-task visibility — `.claude/rules/decision-queue.md`).
7. **Commit message (verbatim):** `test(e2e): seed complete site via setup_local_site path so /api/v4/site returns 200 + surface error body in agpl disclosure test (fix-impl-6)`. In the commit body, state which Part-B path was taken (production-helper-mirrored vs literal-fallback) and why.
8. **DQ attribution:** `from: "impl"` only. NEVER `answered_by: "advisor"` / `"user"`. NEVER `kind: "clarify"` / `"validate-result"` / `"validate-failed"`. Per `.claude/rules/decision-queue.md` hard refusals.
9. **Mandatory post-task retro** before exit (`.claude/rules/post-task-retro.md`): `memory_write_eval`, `source_ref` = your exact worker branch name (the Stop hook on `junior/*` branches requires `source_ref` = branch + a fresh `Task retro:` row in the last 30 min). Do NOT forge `created_at`, do NOT use raw SQL, do NOT modify the hook — those bypass attempts are tracked.

## 5. §2.4 mandatory-lesson firing record (advisor audit)

Authored under `.claude/rules/advisor-orchestrator.md` §2.4. File list = `crates/server/tests/e2e.rs` (1 fn, Part A + Part B). Table matches fired:

- `crates/server/tests/e2e.rs` (any edit) → `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md` → §3 items 9, 10. Case A sibling-mirror: the agpl fn's outer is already `lemmy_utils::error::LemmyResult<()>` and the prior fix-impl-5 block + the canonical precedent e2e.rs:4744-4761 both use bare `?` — Case A confirmed by reading the sibling, no `.map_err` bridge.
- e2e-edit-hang discipline is plan-bound (plan §13 Task 4 GOTCHA 3) regardless of edit count → `feedback_junior_worker_e2e_edit_hang.md` → §3 item 11 + §4.1.
- `feedback_read_canonical_before_writing_spec.md` → §3 item 12 (the in-file `test::read_body` usage at 14911/14936 is the canonical to mirror for Part A; no new import).
- §G4 class: this is a **NON-ALLOWLIST** fail (runtime HTTP 500, not E0432/deprecated/clippy-doc/LemmyError-class). The §G4 verbatim-row blockquote gate does NOT apply (allowlist-only). This is a hand-authored deeper-fix brief on an explicit user §G4 override (the user chose "Deeper-fix" + "Mirror setup_local_site" after the 2nd same-surface fail); the cited reference is the compile-tested production path `crates/routes/src/utils/setup_local_site.rs:80-96`.
- §2.3 PMD hybrid presearch: lane PMD DB is the per-worktree DB (lesson corpus indexed in canonical DB only — known lane-DB-isolation issue). Non-blocking: §2.4 mechanical injection is the load-bearing path; lessons read from disk at `.claude/lessons/` regardless of index. Canonical-schema-first satisfied: Part A mirrors the in-file `test::read_body` pattern (e2e.rs:14911/14936); Part B mirrors the proven production `setup_local_site.rs:88-95` `SiteInsertForm` construction — both explicitly cited.
