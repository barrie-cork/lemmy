# v1-ship-1 — fix-impl-5 brief (seed local_site in the AGPL e2e test)

## 1. Role + dispatch line

`[role:impl-task]` v1-ship-1 fix-impl-5 — seed `instance`/`Site`/`LocalSite`/`LocalSiteRateLimit` in `agpl_source_disclosure_surface_returns_notice` (single anchor-Edit into `crates/server/tests/e2e.rs`) so `GET /api/v4/site` returns 200 instead of 500.

Dispatch string (verbatim):

```
[role:impl-task] v1-ship-1 fix-impl-5 — see .claude/PRPs/briefs/v1-ship-1-fix-impl-5.md
```

## 2. Scope

### 2.1 Why this fix exists (root cause — read first)

Phase-2 e2e RUN on `phase-v1-ship-1` tip `dab15ec56` FAILED. The new test `agpl_source_disclosure_surface_returns_notice` (`crates/server/tests/e2e.rs:14864-14925`) panicked on its **first** assertion:

```
thread 'agpl_source_disclosure_surface_returns_notice' panicked at crates\server\tests\e2e.rs:14883:3:
assertion `left == right` failed: /api/v4/site must return 200
  left: 500
 right: 200
```

`test result: FAILED. 89 passed; 1 failed; 5 ignored` — **only this test failed; no pre-existing test regressed.**

Root cause (advisor-pinned, **plan defect — NOT an impl/test-author bug**): the test bootstraps with `governance_fixtures::bootstrap()` (`e2e.rs:801-836`) which does **schema-apply only — it seeds ZERO rows**. `GET /api/v4/site` → handler `read_site` (`crates/api/api_crud/src/site/read.rs`) → `SiteView::read_local(&mut context.pool())` which `INNER JOIN`s `site`, `local_site`, `instance`, `local_site_rate_limit` (`crates/db_views/site/src/impls.rs:57-78`). With an empty (schema-only) DB those joins return no row → `Err` → `get_site`'s `.map_err(|e| anyhow::anyhow!("Failed to construct site response"))?` → **HTTP 500**. No existing e2e test calls `/api/v4/site`, so the gap was invisible until runtime and survived Phase-1 (compile-only) validation. Task 2's `source_disclosure` code is correct and is **not** the cause — the 500 is the pre-existing `read_site` path failing on an unseeded DB.

### 2.2 The fix — seed the 4-row site scaffold (verbatim canonical precedent)

There is an **existing compile-tested precedent in the same file**: `governance_outbox_emits_remote_sanction_notice_on_local_sanction` at `e2e.rs:4744-4761` seeds exactly this scaffold (`Instance → Site → Person sysacct → LocalSite → LocalSiteRateLimit`) specifically so `SiteView::read_local` returns a row instead of `LocalSiteNotSetup`. **Mirror it verbatim.** Do NOT invent field names or a new helper.

**Produce:** a single anchor-Edit into `crates/server/tests/e2e.rs` that does TWO things to the existing `agpl_source_disclosure_surface_returns_notice` fn (do NOT add a new test, do NOT touch any other test):

**(A)** Add these imports inside the test fn, alongside the existing inner `use` lines (the fn currently has `use actix_web::{App, test, web::Data};`, `use lemmy_db_views_site::api::{GetSiteResponse, GetSourceResponse};`, `use lemmy_utils::rate_limit::RateLimit;` near its top — add this block next to them, do NOT add top-of-file `use`s):

```rust
  use lemmy_db_schema::source::{
    instance::Instance,
    local_site::{LocalSite, LocalSiteInsertForm},
    local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitInsertForm},
    person::{Person, PersonInsertForm},
    site::{Site, SiteInsertForm},
  };
  use lemmy_diesel_utils::traits::Crud;
```

**(B)** Insert this seeding block **immediately after** the line `let (_container, context, _db_url) = governance_fixtures::bootstrap().await?;` (currently `e2e.rs:14870`) and **before** the `let rate_limit = RateLimit::with_debug_config();` / `App::new()` / first `TestRequest`:

```rust
  // Seed instance + Site + LocalSite + LocalSiteRateLimit so
  // `SiteView::read_local` (called by `read_site` for GET /api/v4/site)
  // returns a row instead of LocalSiteNotSetup → HTTP 500.
  // Mirrors the canonical scaffold at e2e.rs:4751-4761
  // (governance_outbox_emits_remote_sanction_notice_on_local_sanction).
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

Everything after the existing first `TestRequest` (the `/api/v4/site` call, the `source_disclosure` asserts, the `/api/v4/source` call, the body asserts, `Ok(())`) stays **exactly as-is** — do NOT modify the assertions or the rest of the test body.

**Boundaries:**

- **Commit ONLY** `crates/server/tests/e2e.rs`. No other file. (`creates: []`, `modifies: [crates/server/tests/e2e.rs]`.)
- **Do NOT** add a new test, touch any other test, touch `governance_fixtures::bootstrap`, add a new fixture helper, change any production code, or alter the existing assertions in this test.
- The test fn outer return stays **`lemmy_utils::error::LemmyResult<()>`** (Case A — unchanged). Every new `.await` line uses bare `?` propagation (the precedent at e2e.rs:4751-4761 already does this; `Instance::read_or_create`, `Site::create`, `Person::create`, `LocalSite::create`, `LocalSiteRateLimit::create` all return `LemmyResult<_>` so bare `?` works — NO `.map_err` bridges, NO Case B/C).
- `Crud` trait import is required for `::create` (the precedent block relies on it being in scope — verify it is; if the fn already has `use lemmy_diesel_utils::traits::Crud;` or a glob that covers it, do not duplicate).

## 3. Required reading (read these FIRST, in order)

1. `crates/server/tests/e2e.rs:14864-14925` — the failing test as it stands now. This is what you are editing.
2. `crates/server/tests/e2e.rs:4598-4608` — the import block of the canonical precedent test (mirror the `lemmy_db_schema::source::{...}` shape).
3. `crates/server/tests/e2e.rs:4744-4761` — **the canonical seeding scaffold to mirror verbatim.** Read its comment (4746-4750): it exists for exactly this reason (`SiteView::read_local` → `LocalSiteNotSetup` without it).
4. `.claude/PRPs/plans/v1-ship-1-r1.plan.md` §10.6 (lines 527-542) — Case A error-shape discipline (the fix stays Case A; no bridges).
5. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **§2.4 MANDATORY (e2e.rs edit).** Case A is the canonical shape; bare `?` throughout; the precedent already conforms.
6. `.claude/lessons/feedback_async_pool_test_pattern.md` — **§2.4 MANDATORY (e2e.rs edit).** `&mut context.pool()` / DbPool usage in e2e (the precedent's `let pool = &mut context_a.pool();` block is the pattern).
7. `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **§2.4 MANDATORY (e2e.rs edit; also plan §13 Task 4 GOTCHA 3).** Single small anchor-Edit only. This edit is ~19 added lines into one fn — well under the hang threshold — but it MUST be ONE Edit, not multiple.

## 4. Constraints (enforce — hard refusals)

1. **Single anchor-Edit into e2e.rs.** Per `feedback_junior_worker_e2e_edit_hang.md` + plan §13 Task 4 GOTCHA 3. The Edit's `old_string` anchors on the unique current text spanning the inner `use` lines + the `bootstrap()` line + the line right after it (enough surrounding context to be unique — read e2e.rs:14864-14872 to choose the anchor); `new_string` = that text with (A) the import block added next to the existing inner `use`s and (B) the seeding block inserted right after the `bootstrap()` line. ONE Edit call. If you find yourself wanting a second Edit into e2e.rs, STOP and re-read GOTCHA 3.
2. **Mirror the precedent verbatim — do NOT invent.** The constructor signatures are confirmed: `Instance::read_or_create(pool, "test.invalid")`, `SiteInsertForm::new(name: String, instance_id)`, `PersonInsertForm::test_form(instance_id, name: &str)`, `LocalSiteInsertForm::new(site_id, system_account: PersonId)`, `LocalSiteRateLimitInsertForm::new(local_site_id)`. Do NOT add fields, do NOT guess alternate constructors. If any signature does not resolve at compile time, do NOT improvise — raise a `kind: "blocker"` DQ (`from: "impl"`) citing the exact compile error and STOP.
3. **Seeding goes BEFORE the first `/api/v4/site` `TestRequest`.** `SiteView::read_local` is `static CACHE`-memoized — the first `/api/v4/site` call populates the cache. Seeding must be committed to the DB before that call (it already is the first request in the test, so placing the block right after `bootstrap()` is correct and sufficient).
4. **Do NOT change the assertions or the rest of the test body.** Only ADD imports + the seeding block. The `/api/v4/site` 200 assert, the four `source_disclosure` asserts, the `/api/v4/source` 200 assert, and the notice-body asserts stay byte-identical.
5. **Case A only.** Outer `lemmy_utils::error::LemmyResult<()>` (unchanged). Bare `?` on every new line. No error-bridge closures. Mixing shapes = §G4 row 4c hard refusal.
6. **Validation = Shape G SUSPENDED → validate-pending-laptop.** Per `.claude/rules/advisor-orchestrator.md` §5.2 + DQ #229 (Shape G suspended repo-wide until 2026-06-01). After commit + push to your worker branch, write a `kind: "validate-pending-laptop"` DQ entry (`from: "impl"`, `answered_by: null`), `phase_task: 4`, `branch: <your worker branch>`, `commands:` the §15.1–15.3 workspace commands verbatim:
   - `bash scripts/brehon/cargo-check.sh --workspace --features full`
   - `bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings`
   - `bash scripts/brehon/cargo-test.sh --no-run -p lemmy_server --test e2e`
   (the e2e *run* — §15.4, Docker/testcontainers — is a SEPARATE advisor-driven Phase-2 step; do NOT attempt the e2e run yourself). Compute `next_id` across `.claude/decision-queue.json` pending+resolved + any `.claude/decision-queue-archive-*.json` (max+1) — current max is **242**, so your entry is **243** (verify before writing; if drift, recompute). Commit the DQ entry + push to your worker branch immediately (mid-task visibility — `.claude/rules/decision-queue.md`).
7. **Commit message (verbatim):** `test(e2e): seed local_site so /api/v4/site returns 200 in agpl disclosure test (fix-impl-5)`.
8. **DQ attribution:** `from: "impl"` only. NEVER `answered_by: "advisor"` / `"user"`. NEVER `kind: "clarify"` / `"validate-result"` / `"validate-failed"`. Per `.claude/rules/decision-queue.md` hard refusals.
9. **Mandatory post-task retro** before exit (`.claude/rules/post-task-retro.md`): `memory_write_eval`, `source_ref` = your exact worker branch name (the Stop hook on `junior/*` branches requires `source_ref` = branch + a fresh `Task retro:` row in the last 30 min).

## 5. §2.4 mandatory-lesson firing record (advisor audit)

Authored under `.claude/rules/advisor-orchestrator.md` §2.4. File list = `crates/server/tests/e2e.rs` (1 edit, ~19 added lines, single fn). Table matches fired:

- `crates/server/tests/e2e.rs` (any edit) → `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md` → §3 items 5, 6.
- e2e-edit-hang discipline is plan-bound (plan §13 Task 4 GOTCHA 3) regardless of the ≥2-edit threshold → `feedback_junior_worker_e2e_edit_hang.md` → §3 item 7.
- §G4 class: this is a **NON-ALLOWLIST** fail (runtime assertion / HTTP 500, not E0432/deprecated/clippy-doc/LemmyError-class), so the §G4 verbatim-row blockquote gate does NOT apply (allowlist-only). This is a hand-authored fix-impl brief with a cited compile-tested canonical precedent (`e2e.rs:4744-4761`), per the §G4 "Non-allowlist (catch-fire to user)" path → user chose the fix-impl resolution path.
- §2.3 PMD hybrid presearch: lane PMD DB is the per-worktree DB (lesson corpus indexed in canonical DB only — known lane-DB-isolation issue, same root as the Stop-hook gap). Non-blocking: §2.4 mechanical injection is the load-bearing path; lessons read from disk at `.claude/lessons/` regardless of index. Canonical-schema-first satisfied: the fix mirrors a verbatim compile-tested in-file precedent (e2e.rs:4751-4761), explicitly cited.
