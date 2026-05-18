---
phase: v1-ship-1
role: impl-task
task: 6
brief_n: 6
plan: .claude/PRPs/plans/v1-ship-1-r2.plan.md
created: 2026-05-18
related_dq: 261
---

# [role:impl-task] v1-ship-1-r2 Task 6 e2e rebuild — see .claude/PRPs/briefs/v1-ship-1-impl-6.md

> **Clarify provenance:** the parent planning brief was clarified before
> planning task #309. **DQ #261** (`from: "planner"`, `kind: "log"`,
> RESOLVED — read it) records the type-precision mechanism correction
> baked into this plan: the original brief's Option(b) literal recipe
> (`to_request_data()` as the `.app_data()` source) is **mechanically
> incompatible** (returns `activitypub_federation::config::Data<T>` ≠
> the `actix_web::web::Data<T>` `get_site` extracts → reproduces HTTP
> 500). The plan adopts the production line-for-line idiom
> (`federation_config.deref().clone()`, `lib.rs:364`). User APPROVED
> this at Gate 1, 2026-05-18.

## 0. Pre-flight (subagent runs this before reading anything else)

### 0.1 Forbidden-window check

The `impl-task` agent already runs the forbidden-window check from
`.claude/agents/impl-task.md` "Task-0 pre-flight". This brief inherits
that — do not duplicate the bash. No `forbidden-window-override` on the
dispatch line (advisor confirmed the queue time is inside a safe
window).

### 0.2 Shape G suspended — §5.2-laptop validation shape (authority)

**Shape G (GitHub-Actions cargo validation) is SUSPENDED repo-wide
until 2026-06-01 per DQ #229** (Actions minutes exhausted). This is the
binding authority for the validation shape in §5 below. **Do NOT** write
a `kind: "validate-pending"` entry and do NOT capture a
`workflow_run_id`. After you push your worker branch, write a
`kind: "validate-pending-laptop"` entry (per `.claude/agents/impl-task.md`
"Pre-Shape-G plans" + `.claude/rules/decision-queue.md` "validate-pending-
laptop handler") naming the three §5 commands verbatim in `commands[]`.
The advisor-laptop session runs them and mutates the entry. The Phase-2
e2e is advisor-driven (advisor raises a separate
`kind: "validate-pending-laptop-e2e"` entry after your worker branch
finalize-merges into `phase-v1-ship-1`) — NOT your responsibility.

### 0.3 Submodule init (MANDATORY — carried from Task 0 #312 advisory signal)

**Before any cargo invocation in §5, run:**

```bash
git submodule update --init crates/email/translations
```

Rationale: Task 0 (#312) Probe 14 failed on first run with exit 101
because the `crates/email/translations` git submodule
(lemmy-translations) is **uninitialized in fresh Junior worktrees** —
`lemmy_email`'s `build.rs` does `read_dir("translations/backend/")`
which fails `No such file or directory`, breaking the whole workspace
compile. The Task 0 worker self-recovered with the command above; this
is the known `feedback_worktree_submodules_not_auto_init.md` /
`feedback_phase_lane_worktree_bootstrap_checklist.md` class. **Run the
submodule-init command BEFORE §5 cmd1** or all three §5 cargo commands
fail with the same error and you burn a full validation cycle. This is
NOT optional and NOT a blocker — it is a one-line worktree-bootstrap
fixup the daemon's worktree creation does not perform.

## 1. Role + dispatch line

`[role:impl-task] v1-ship-1-r2 Task 6 e2e rebuild — see .claude/PRPs/briefs/v1-ship-1-impl-6.md`

You are the **impl-task** subagent (Sonnet 4.6 per your frontmatter).
Execute plan Task 6 from `.claude/PRPs/plans/v1-ship-1-r2.plan.md` §13
(Task 6, lines ~579-803). This is the **single live deliverable** of
the r2 re-plan: rebuild the App-construction inside the failed e2e test
`agpl_source_disclosure_surface_returns_notice` on the verified-correct
canonical `lib.rs:364/379-382` idiom. Tasks 1-4 are MERGED
carry-forward (do NOT re-touch them).

## 2. Scope

**Produce** (exactly one commit):

- `crates/server/tests/e2e.rs` — a **single anchored Edit** that
  replaces the **body** of `agpl_source_disclosure_surface_returns_notice`
  (the lines BETWEEN the fn signature's opening `{` and the fn's
  closing `}`) with the verified-correct body in §2.1 below. The fn
  **signature line** (`async fn agpl_source_disclosure_surface_returns_notice()
  -> lemmy_utils::error::LemmyResult<()> {`) and the fn's **closing
  `}`** are NOT changed.

**Do NOT** in this task:

- Touch `crates/api/api_crud/**`, `crates/api/api/src/site/source.rs`,
  `crates/api/routes/src/lib.rs`, `crates/db_views/site/src/api.rs`,
  `crates/api/api_crud/build.rs` — Tasks 1-4 deliverables, MERGED on
  the phase branch. Verified present by Task 0 Probes 5/6/7.
- Edit `AGPL-NOTICE.md` (out of scope per r1.plan.md §12, still
  binding).
- Add a second e2e test (out of scope per PRD §7.1 — the single named
  test is the ship gate).
- Change the fn signature or the test name (GOTCHA 5).

**Commit message** (exactly, per plan §13 Task 6 COMMIT):
`test(e2e): rebuild agpl_source_disclosure_surface_returns_notice App on canonical lib.rs:364 idiom (task 6)`

The commit **body MUST quote** `config.rs:264-270` (`Deref<Target=T>`)
+ `lib.rs:364` + `lib.rs:379` as the source citations for the chosen
mechanism (per R11 + DQ #261 + `feedback_read_canonical_before_writing_spec.md`).
Include a `LESSON:` trailer + a `HANDOVER:` YAML trailer per
`feedback_handover_trailer_cohort_propagation.md` /
`feedback_junior_pmd_write_convention.md`.

### 2.1 The EXACT replacement body (verbatim from plan §13 Task 6 IMPLEMENT)

This is the **contract**. The plan's §13 Task 6 fenced `rust` block
(plan lines ~602-733) is the source-of-truth; reproduced here verbatim.
Your Edit's `new_string` is exactly this body (Case A discipline;
preserves fix-impl-6 Part A + Part B; mirrors §10.5/§10.6/§10.7
verbatim):

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

### 2.2 Single-anchored-Edit procedure (GOTCHA 1 — `feedback_junior_worker_e2e_edit_hang.md`)

`e2e.rs` is **14,981 lines** at the phase tip. Multiple Edits or a
full-file Write **hang Junior workers** (DQ #117 + many retros). You
MUST:

1. `grep -nE "^async fn agpl_source_disclosure_surface_returns_notice" crates/server/tests/e2e.rs`
   — confirm the signature line (advisor verified `:14865` at phase
   tip; accept drift, use the grep result).
2. Locate the fn's closing `}` — the brace that closes this fn (at
   phase tip `:14981`; it is the last `}` before EOF since this is the
   final test in the file).
3. **ONE Edit call.** `old_string` = the **entire current body between
   (but not including) the signature's opening `{` and the closing
   `}`**. To make the anchor unambiguous, include the signature line
   AND the closing `}` line in BOTH `old_string` and `new_string`
   (i.e. the unit of replacement is the whole fn region:
   `async fn agpl_..._notice() -> ...LemmyResult<()> {` … `}`).
4. `new_string` = the signature line + the §2.1 body + the closing
   `}` (so the signature and brace are byte-identical, only the body
   changes).
5. **NOT** two Edits; **NOT** a Write of the whole file; **NOT** a
   sequence of small line-Edits. If the Edit tool rejects
   (`old_string not unique`), enlarge the anchor further (more
   surrounding context) — never fall back to line-by-line.

## 3. Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entry DQ #261** (the
   planner type-precision correction — `from: "planner"`,
   `kind: "log"`). This is the single most load-bearing context: it
   explains why `to_request_data()` / `init_test_context()` are
   mechanically wrong and why `federation_config.deref().clone()` is
   correct.
2. **Plan §13 Task 6** (lines ~579-803) — the canonical step list +
   the verbatim IMPLEMENT body + all 6 GOTCHAs + the VALIDATE blocks.
   The plan's fenced `rust` block is the contract; §2.1 of this brief
   reproduces it verbatim.
3. **Plan §10.5, §10.6, §10.7** (lines ~318-421) — the
   `lib.rs:228-241` / `lib.rs:364` / `lib.rs:379-382` mirror
   derivations + the BINDING "why this works / why the failed line
   was wrong" rationale you must quote in the commit body.
4. **Plan §10.8, §10.9, §10.10** (lines ~422-434) — Case A error
   shape (settled, preserve) + fix-impl-6 Part A (body-on-failure
   asserts, preserve verbatim) + Part B (full seed scaffold, preserve
   verbatim).
5. **`.claude/lessons/feedback_lemmy_error_no_std_error.md`** —
   **Case A** discipline (mandatory file-class injection: e2e.rs
   edit). Outer `LemmyResult<()>`, bare `?` throughout, no
   `Box<dyn Error>`, no `.map_err(|e| anyhow::anyhow!(...))?`
   bridges. A v1-SL-b `mod v1_sl_b_fixtures` sibling at
   `e2e.rs:11131-11924` is the canonical Case A — the §2.1 body
   already mirrors it; do not deviate.
6. **`.claude/lessons/feedback_async_pool_test_pattern.md`** —
   (mandatory file-class injection: e2e.rs edit). `context.pool()`
   borrow pattern for the seed scaffold (already correct in §2.1
   Part B — preserve).
7. **`.claude/lessons/feedback_junior_worker_e2e_edit_hang.md`** —
   the single-anchored-Edit discipline (GOTCHA 1 invokes it
   explicitly even for one Edit, because the file is ~15k lines).
8. **`.claude/lessons/feedback_read_canonical_before_writing_spec.md`**
   — why the chosen mechanism mirrors the canonical `lib.rs:364`
   composition rather than inventing a variant.
9. **`.claude/rules/decision-queue.md`** — Recipe 1 (blocker DQ) +
   "validate-pending-laptop handler" (the §5 entry shape) +
   "Mid-task visibility" (commit+push the DQ immediately).

## 3a. Handover from prior cohort

(none — Task 6 is a solo non-`[P]` task. Its `requires:` are Tasks
1/2/3/4, all MERGED on `phase-v1-ship-1` and confirmed present by
Task 0 Probes 5/6/7/8. There is no in-flight prior cohort whose
`HANDOVER:` trailer would feed this task.)

## 4. Constraints

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-ship-1`. Finalize
  merges your worktree branch back; do NOT push to `phase-v1-ship-1`
  directly.
- **One commit.** If clippy/check fails on the first attempt, amend
  or fixup — do NOT split the commit. (The plan §13 Task 6 is a
  single-task; one `test(e2e):` commit only.)
- Mid-task DQ visibility: if you raise a `pending` blocker entry,
  **commit + push immediately** to your worktree branch:
  ```bash
  git add .claude/decision-queue.json
  git commit -m "chore(decision-queue): impl raised DQ #<id> — <slug>"
  git push origin <worktree-branch>
  ```
  Compute `next_id` across `.claude/decision-queue.json` +
  `.claude/decision-queue-archive-*.json` (DQ #50 collision lesson).
- No `answered_by: "advisor"` / `"user"` from this subagent.
  Self-resolve only as `"impl-self-resolved"`. `from: "impl"`.
  NEVER `kind: "clarify"` (advisor-only).

### GOTCHA 2 — type-precision (R11 / DQ #261) — VERIFY BEFORE COMMIT

Before commit, **grep + visually confirm**:

- `.app_data(Data::new(inner_context.clone()))` uses **`inner_context`**
  (a `LemmyContext` from `federation_config.deref().clone()`), **NOT
  `context`** (the bootstrap's `Data<LemmyContext>`). Wrong
  substitution → `Data<Data<LemmyContext>>` → HTTP 500 (the
  proven-failed shape across 5 fix-impl cycles). This is the ONE
  TOKEN that was wrong in fix-impl-8.
- `.wrap(SessionMiddleware::new(inner_context.clone()))` uses
  `inner_context`, NOT `(**context).clone()` (fix-impl-7b's variant).
  Both compile; mirror-fidelity dictates `inner_context` (lib.rs:382
  mirror uses the `context` var that is `LemmyContext` per the
  let-binding at lib.rs:364).
- `.wrap(FederationMiddleware::new(federation_config.clone()))` uses
  `federation_config` (the binding from `.build().await?`), NOT a
  re-built second config.

Run: `git diff crates/server/tests/e2e.rs | grep -cE '^\+\s+\.app_data\(Data::new\(inner_context\.clone\(\)\)\)'`
→ MUST be `1` (GOTCHA 6). Count 0 or ≥2 = malformed Edit; STOP, do
NOT commit, re-do the Edit.

### GOTCHA 3 — `use std::ops::Deref;` MUST be in the fn body

The §2.1 body's inline import block already contains
`use std::ops::Deref;`. WITHOUT it, `.deref()` on
`FederationConfig<LemmyContext>` fails to resolve. Do NOT remove it.
(Equivalent alternative that compiles without the `use`:
`(*federation_config).clone()` — but §2.1 uses the explicit
`.deref().clone()` to byte-mirror `lib.rs:364`; keep it as written.)

### GOTCHA 4 — bootstrap selection (DQ #226)

§2.1 calls `governance_fixtures::bootstrap()` (the chosen one,
`e2e.rs:801`). It does NOT call `admin_config_fixtures::bootstrap()`
(`:5665`, the REJECTED sibling). Preserve exactly — do not swap.

### GOTCHA 5 — fn-name uniqueness (verify pre-Edit)

```bash
grep -cE "async fn agpl_source_disclosure_surface_returns_notice" crates/server/tests/e2e.rs
# EXPECT: 1
```
0 or >1 → file `kind: "blocker"` DQ (`from: "impl"`); the file
structure changed beyond plan anticipation. Advisor verified this
returns `1` at phase tip — re-verify on your worktree.

### Plan-cited line numbers may have drifted

Advisor drift-checked at brief-authoring time: signature `:14865`,
closing `}` `:14981`, total 14,981 lines, fn-count 1, bootstrap first
hit `:801` — **zero drift** at phase tip `f92d2c3cc`. Still
`grep -n` before the Edit; if drifted, follow grep output, not the
numbers.

### Lesson trailer

Per §2 the commit body includes a `LESSON:` line. Suggested (the
generalisable insight from the 5-attempt failure): the
actix-`Data`-double-wrap-with-bootstrap class — registering
`Data::new(<already-a-Data>)` produces `Data<Data<T>>`, a distinct
TypeId the extractor never finds; failure is dynamic at extractor
time (compiles fine), so 5 cycles each added middleware instead of
challenging the one wrong token. One discrete `LESSON:` line, cite
`e2e.rs` + `lib.rs:364/379`.

## 4.1 CANONICAL CASE OVERRIDE

**Case A (settled — preserve, do NOT re-decide).** Per plan §10.8 +
`feedback_lemmy_error_no_std_error.md`: the fn signature is
`async fn agpl_source_disclosure_surface_returns_notice() ->
lemmy_utils::error::LemmyResult<()>` (outer `LemmyResult<()>`); the
§2.1 body uses **bare `?` throughout**, no `Box<dyn Error>`, no
`.map_err(|e| anyhow::anyhow!(...))?` bridges. The canonical Case A
sibling is `e2e.rs:11131-11924` (`mod v1_sl_b_fixtures`). The §2.1
body already mirrors it — there is NO Case A/B/C decision to make
here; it is pre-settled. Do not introduce error bridges.

## 5. Validation gates (§5.2-laptop Phase 1 — per §0.2 / DQ #229)

**Shape G suspended until 2026-06-01 (DQ #229).** After the Edit +
commit + push of your worker branch, write ONE
`kind: "validate-pending-laptop"` DQ entry (`from: "impl"`,
`answered_by: null`) with `commands[]` = the **three commands below
verbatim** (including the `cmd //c` wrapper, scope flags, and
`--features full` where shown), `branch` = your worker branch,
`phase_task` = 6, and the nullable fields (`result`, `log_slice`,
`failed_commands`) = null. Commit + push the DQ entry per
"Mid-task visibility". Do NOT run cargo yourself — the advisor-laptop
session runs these and mutates the entry.

**FIRST run the §0.3 submodule-init** (`git submodule update --init
crates/email/translations`) — these three commands all fail without
it.

```bash
# §15.1 — workspace check
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-ship-1-r2-task6-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-1-r2-task6-check.log
# EXPECT: exit 0

# §15.2 — clippy
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-ship-1-r2-task6-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-1-r2-task6-clippy.log
# EXPECT: exit 0

# §15.3 — test target compile (per DQ #259: -p lemmy_server --test e2e, NO --features full)
cmd //c "scripts\\brehon\\cargo-test.bat --no-run -p lemmy_server --test e2e > .claude/PRPs/debug/v1-ship-1-r2-task6-test-no-run.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-1-r2-task6-test-no-run.log
# EXPECT: exit 0
```

These `.claude/PRPs/debug/*.log` paths are gitignored debug class —
writing them is expected, not a sensitive-file violation. The
**Phase-2 e2e** (`--workspace --test e2e --features full`,
`agpl_source_disclosure_surface_returns_notice` PASS) is
**advisor-driven post-finalize-merge** — NOT your responsibility; do
not run it, do not raise its entry.

If §15.1/§15.2/§15.3 would fail (you can't know — advisor runs them),
that is handled by the advisor's §G4 classifier, not by you. Do NOT
`#[allow]`-spam or patch around a hypothetical failure. Your job ends
at: Edit done + GOTCHA 2/5/6 verified locally (grep only, no cargo) +
commit + push + validate-pending-laptop DQ raised.

## 6. Expected output (return to advisor)

```
## Task 6 complete — v1-ship-1-r2 e2e App-construction rebuild

**Commit:** <sha> on <worktree-branch>
**Files changed:** crates/server/tests/e2e.rs (single anchored Edit —
  body of agpl_source_disclosure_surface_returns_notice replaced;
  signature + closing brace byte-identical)
**GOTCHA verification (grep-only, pre-commit):**
  - GOTCHA 2/6: `.app_data(Data::new(inner_context.clone()))` count = 1 ✓
  - GOTCHA 3: `use std::ops::Deref;` present in fn body ✓
  - GOTCHA 4: governance_fixtures::bootstrap() present; admin_config_fixtures::bootstrap absent ✓
  - GOTCHA 5: fn-name count = 1 ✓
  - inner_context (not context) used in app_data + SessionMiddleware ✓
  - inner_context derived via federation_config.deref().clone() ✓
**Submodule init (§0.3):** ran `git submodule update --init crates/email/translations` ✓
**DQ raised:** #<id> kind:validate-pending-laptop (3 §15 cmds, phase_task=6)
**Validation:** NOT run by impl (Shape G suspended; advisor-laptop runs §15.1/2/3)
**Next:** advisor-laptop runs §5.2 Phase-1 trio; on PASS + finalize-merge,
  advisor raises Phase-2 e2e validate-pending-laptop-e2e for the post-merge tip.
```

Plus any DQ #N references if you raised a blocker mid-task.

## 7. Why this brief differs from the plan

**No design deviations — clean execution of plan §13 Task 6.** Three
brief-side amplifications, all codifying (not changing) plan intent:

1. **§0.3 submodule-init step prepended.** Not in the plan's Task 6
   VALIDATE block, but mandated by the Task 0 (#312) advisory signal:
   fresh Junior worktrees have `crates/email/translations`
   uninitialized → all three §5 cargo commands fail without
   `git submodule update --init crates/email/translations`. This is a
   worktree-bootstrap fixup (`feedback_worktree_submodules_not_auto_init.md`),
   not a plan change — the plan assumes a working compile environment;
   this step makes that assumption hold on a fresh daemon worktree.
2. **§0.2 Shape-G-suspended authority made explicit.** The plan's
   §13 Task 6 VALIDATE says "§5.2-laptop Phase 1 (per §0.2 / Shape G
   suspended per DQ #229)" — referencing a brief-side §0.2. This
   brief defines that §0.2 (the DQ #229 suspension is the authority
   for writing `validate-pending-laptop`, not `validate-pending`).
3. **GOTCHA 2/5/6 verification made a pre-commit hard gate in §4.**
   The plan lists them as GOTCHAs; this brief makes the grep checks
   an explicit STOP-before-commit gate so the one-token error
   (`context` vs `inner_context`) that defeated 5 prior cycles cannot
   recur silently.

The §2.1 body is reproduced **verbatim** from the plan's §13 Task 6
IMPLEMENT fenced block — byte-for-byte, no paraphrase (per the
anti-paraphrase discipline; the plan's canonical recipe is the
contract).
