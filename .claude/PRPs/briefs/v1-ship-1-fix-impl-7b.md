# v1-ship-1 — fix-impl-7b brief (one-token deref: SessionMiddleware::new arg Data<LemmyContext> → LemmyContext)

## 1. Role + dispatch line

`[role:impl-task]` v1-ship-1 fix-impl-7b — in `agpl_source_disclosure_surface_returns_notice` (single anchor-Edit into `crates/server/tests/e2e.rs`): change the **one argument** `SessionMiddleware::new(context.clone())` → `SessionMiddleware::new((**context).clone())` at the line fix-impl-7 added, so the test compiles (`SessionMiddleware::new` takes `LemmyContext`; the agpl test's `context` is `Data<LemmyContext>`). Nothing else changes.

Dispatch string (verbatim):

```
[role:impl-task] v1-ship-1 fix-impl-7b — see .claude/PRPs/briefs/v1-ship-1-fix-impl-7b.md
```

## 2. Scope

### 2.1 Why this fix exists (one-token follow-on correction of fix-impl-7; same authorization)

fix-impl-7 (#303, commit `4041afda1`, finalize-merged into `phase-v1-ship-1`) added `.wrap(SessionMiddleware::new(context.clone()))` to the agpl test App, mirroring the passing sibling `all_mvp_endpoints_return_non_404` (e2e.rs:3860) byte-for-byte. §5.2 Phase-1 then ran on the merged tip `f9214f2f8`:

- cmd1 `cargo-check --workspace --features full` = **PASS** (1m48s)
- cmd2 `cargo-clippy --workspace --features full --no-deps -- -D warnings` = **PASS** (3m01s, no -D warnings)
- cmd3 `cargo-test --no-run -p lemmy_server --test e2e` = **FAIL**:

```
error[E0308]: mismatched types
  --> crates\server\tests\e2e.rs:14911:36
   |
14911 |       .wrap(SessionMiddleware::new(context.clone()))
   |             ---------------------- ^^^^^^^^^^^^^^^ expected `LemmyContext`, found `Data<LemmyContext>`
   |             |
   |             arguments to this function are incorrect
   = note: expected struct `LemmyContext`
              found struct `actix_web::web::Data<LemmyContext>`
note: associated function defined here
  --> crates\routes\src\middleware\session.rs:22:10
   | 22 |   pub fn new(context: LemmyContext) -> Self {
```

**Root cause (a brief defect in fix-impl-7, NOT a Junior bug):** `SessionMiddleware::new` takes a **bare `LemmyContext`** by value (`crates/routes/src/middleware/session.rs:22`). In the agpl test, `context` is `Data<LemmyContext>` — it comes from `governance_fixtures::bootstrap()` which returns `Data<LemmyContext>` (e2e.rs:803). The fix-impl-7 brief mandated a byte-for-byte mirror of the sibling at e2e.rs:3860 — but **that sibling's `context` is a bare `LemmyContext`** (it does `let context = LemmyContext::create(...)` directly at e2e.rs:3849, not via `bootstrap()`). The two tests obtain `context` differently, so the byte-for-byte mirror is a type error here. cmd1+cmd2 PASS, so the rest of fix-impl-7 (the `use`, the `.wrap` placement, Part A, Part B) is correct — **only the argument type needs a deref.**

**The fix is the proven in-file idiom.** `Data<T>` (actix `web::Data`) is an `Arc<T>` that derefs to `T`; `LemmyContext` is `Clone`. The established convention in this exact file for converting a `bootstrap()`-derived `Data<LemmyContext>` to a by-value `LemmyContext` is **`(**context).clone()`** — used at **16 existing sites** in e2e.rs (lines 2363, 3096, 9008, 9164, 9379, 9502, 9682, 9827, 10340, 10578, 10783, 13055, 13295, 13494, 14042, 14382), all `.app_data((**context).clone())` in `bootstrap()`-style tests. Mirror that idiom for the `SessionMiddleware::new` argument.

### 2.2 The fix — ONE anchor-Edit into `crates/server/tests/e2e.rs`, ONE commit, ONE token

Edit ONLY the existing `agpl_source_disclosure_surface_returns_notice` fn. The current line (added by fix-impl-7, ~e2e.rs:14911) is:

```rust
      .wrap(SessionMiddleware::new(context.clone()))
```

Change it to (add `**` deref, mirroring the 16-site in-file idiom):

```rust
      .wrap(SessionMiddleware::new((**context).clone()))
```

That is the **entire change**: `context.clone()` → `(**context).clone()` inside the `SessionMiddleware::new(...)` call on that one line. One `Edit` with a unique `old_string` of exactly that line (or the minimal surrounding context to make it unique — the `.app_data(Data::new(context.clone()))` line directly above also contains `context.clone()`, so the `old_string` MUST include enough context to target ONLY the `.wrap(SessionMiddleware::new(context.clone()))` line, e.g. include the `.wrap(` prefix; do NOT accidentally edit the `.app_data(...)` line).

**Do NOT touch anything else:**

- The `.app_data(Data::new(context.clone()))` line directly above (e2e.rs:~14910) is **correct as-is** (it compiled in cmd1/cmd2 — `Data::new` takes the inner value; leave it byte-identical). Only the `SessionMiddleware::new` argument on the `.wrap(...)` line changes.
- fix-impl-7's `use lemmy_routes::middleware::session::SessionMiddleware;` (e2e.rs:~14877) stays unchanged.
- fix-impl-6 Part A (body-on-failure asserts for `/api/v4/site` + `/api/v4/source`, ~e2e.rs:14919-14953) stays byte-identical.
- fix-impl-6 Part B (complete `SiteInsertForm { ap_id: Some(...), last_refreshed_at: Some(...), inbox_url: Some(...), private_key: Some(...), public_key: Some(...), ..SiteInsertForm::new("agpl test site".to_string(), instance.id) }` + `generate_actor_keypair`, ~e2e.rs:14888-14896) stays byte-identical.
- The sibling `all_mvp_endpoints_return_non_404`'s `.wrap(SessionMiddleware::new(context.clone()))` at e2e.rs:3860 stays UNCHANGED — its bare `context.clone()` is correct because *its* `context` is a bare `LemmyContext`. Do NOT edit that test. Do NOT "fix" it to match — it is already correct.

**Boundaries:**

- **Commit ONLY** `crates/server/tests/e2e.rs`. No other file. (`creates: []`, `modifies: [crates/server/tests/e2e.rs]`.)
- **Do NOT** edit `crates/api/**`, `crates/db_schema/**`, `crates/db_views/**`, `crates/routes/**` (incl `session.rs` — `SessionMiddleware::new`'s signature is CORRECT; the test must adapt to it, not the reverse), `crates/server/src/**`, or any production code. If you believe a production change is needed, STOP and raise a `kind: "blocker"` DQ (`from: "impl"`).
- **Do NOT** add a new test, touch any other test (including the sibling `all_mvp_endpoints_return_non_404` you must NOT edit), touch `governance_fixtures::bootstrap`, change any assertion logic, or modify fix-impl-6 Part A / Part B / fix-impl-7's `use`.
- The diff MUST be exactly **one line changed**: `context.clone()` → `(**context).clone()` inside `.wrap(SessionMiddleware::new(...))`. Net `git diff` of the commit = 1 line modified in e2e.rs (1 deletion + 1 insertion of the same line with `**` added). If your diff is more than this one line in e2e.rs → STOP, you have over-reached; re-read this section.
- The test fn outer return stays **`lemmy_utils::error::LemmyResult<()>`** (Case A — unchanged). The `**` deref introduces no `?`, no error bridge.

## 3. Required reading (read these FIRST, in order)

1. `crates/server/tests/e2e.rs:14906-14913` — the `App` builder block in the agpl fn. Lines: `.app_data(Data::new(context.clone()))` (~14910, leave unchanged), `.wrap(SessionMiddleware::new(context.clone()))` (~14911, THIS is the one line you change). Confirm the exact line numbers (they may shift slightly post-merge) and pick a unique `old_string`.
2. `crates/routes/src/middleware/session.rs:16-25` — `SessionMiddleware` struct + `pub fn new(context: LemmyContext) -> Self`. Confirms the param is a bare `LemmyContext` by value. This signature is CORRECT and is NOT to be changed.
3. `crates/server/tests/e2e.rs:803` (inside `governance_fixtures::bootstrap`) — confirms `bootstrap()` returns `Data<LemmyContext>`, so the agpl test's `context` is `Data<LemmyContext>`.
4. `crates/server/tests/e2e.rs:2363` (and any of 3096/9008/9164/13055/14042/14382) — the **canonical in-file idiom** `.app_data((**context).clone())`: `(**context).clone()` converts a `bootstrap()`-derived `Data<LemmyContext>` to a by-value `LemmyContext`. This is exactly the conversion to apply to the `SessionMiddleware::new` argument. Mirror it.
5. `crates/server/tests/e2e.rs:3849-3862` — the sibling `all_mvp_endpoints_return_non_404`: note its `context` is `let context = LemmyContext::create(...)` (bare `LemmyContext`, line ~3849), which is WHY its `.wrap(SessionMiddleware::new(context.clone()))` at 3860 is correct without a deref. Context only — do NOT edit this test.
6. `.claude/PRPs/plans/v1-ship-1-r1.plan.md` §10.6 — Case A error-shape discipline (this fix stays Case A; the deref introduces no `?`).
7. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **§2.4 MANDATORY (e2e.rs edit).** Case A; the agpl test outer is already `lemmy_utils::error::LemmyResult<()>` — keep it; this fix adds no fallible line.
8. `.claude/lessons/feedback_async_pool_test_pattern.md` — **§2.4 MANDATORY (e2e.rs edit).** Context: the existing `&mut context.pool()` block is unchanged; the deref only affects the `SessionMiddleware::new` argument.
9. `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **§2.4 MANDATORY (e2e.rs edit; plan §13 Task 4 GOTCHA 3).** This is the SMALLEST possible edit (one token added to one line). ONE Edit only. Never a full-file rewrite.
10. `.claude/lessons/feedback_read_canonical_before_writing_spec.md` — the canonical in-file idiom is `(**context).clone()` (16 sites, e.g. e2e.rs:2363); mirror it exactly, do not invent an alternate deref form (`*context`, `context.get_ref().clone()`, `context.as_ref().clone()` may also compile but the in-file convention is `(**context).clone()` — use that for consistency).

## 4. Constraints (enforce — hard refusals)

1. **Exactly ONE line changed in e2e.rs, ONE Edit.** Per `feedback_junior_worker_e2e_edit_hang.md` + plan §13 Task 4 GOTCHA 3. The change is `context.clone()` → `(**context).clone()` inside `.wrap(SessionMiddleware::new(...))` on the line fix-impl-7 added. The `old_string` MUST be unique and MUST target the `.wrap(SessionMiddleware::new(...))` line, NOT the `.app_data(Data::new(context.clone()))` line above it (which also contains `context.clone()` and must stay unchanged). If you cannot make a unique single-line `old_string`, include the `.wrap(` token in it. >1 line changed in e2e.rs = over-reach → STOP, re-read §2.2.
2. **Mirror the proven in-file idiom `(**context).clone()`** (16 sites, canonical at e2e.rs:2363). Do NOT invent a different deref (`*context`, `.get_ref()`, `.as_ref()`); use `(**context).clone()` for consistency with the file's convention.
3. **fix-impl-6 Part A + Part B + fix-impl-7's `use` + the `.app_data(...)` line + the sibling test (e2e.rs:3860) are ALL preserved byte-identical.** Verify after your edit: `git diff` of the commit shows exactly 1 line modified in e2e.rs (the `.wrap(SessionMiddleware::new(...))` line gaining `**`), nothing else.
4. **Do NOT change `session.rs` or any production code.** `SessionMiddleware::new(context: LemmyContext)` is correct; the test adapts to it.
5. **Case A only.** Outer `lemmy_utils::error::LemmyResult<()>` unchanged. The `**` deref adds no `?`, no error bridge.
6. **Validation = Shape G SUSPENDED → validate-pending-laptop.** Per `.claude/rules/advisor-orchestrator.md` §5.2 + DQ #229 (Shape G suspended repo-wide until 2026-06-01). After commit + push to your worker branch, write a `kind: "validate-pending-laptop"` DQ entry (`from: "impl"`, `answered_by: null`), `phase_task: 4`, `branch: <your worker branch>`, `commands:` the §15.1–15.3 workspace commands verbatim:
   - `bash scripts/brehon/cargo-check.sh --workspace --features full`
   - `bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings`
   - `bash scripts/brehon/cargo-test.sh --no-run -p lemmy_server --test e2e`
   (the e2e *run* — §15.4 — is a SEPARATE advisor-driven Phase-2 step; do NOT attempt it.) Compute `next_id` across `.claude/decision-queue.json` pending+resolved + every `.claude/decision-queue-archive-*.json` (max+1). Current max id is **248** (DQ #248 = the fix-impl-7 Phase-1 E0308 fail), so your validate-pending-laptop entry is **249** — verify by computing max+1 before writing; if drift, recompute (do NOT hardcode if the assert fails). Commit the DQ entry + push to your worker branch immediately (mid-task visibility — `.claude/rules/decision-queue.md`).
7. **Commit message (verbatim):** `test(e2e): deref Data<LemmyContext> for SessionMiddleware::new arg in agpl test ((**context).clone(), mirrors 16-site in-file idiom) (fix-impl-7b)`. In the commit body, state: (a) the cause (fix-impl-7's byte-for-byte mirror of a sibling whose context is a bare LemmyContext, while agpl's context is Data<LemmyContext> from bootstrap()), (b) the one-token fix mirrors the canonical `.app_data((**context).clone())` idiom (e2e.rs:2363 +15 others), (c) that fix-impl-6 Part A/B + fix-impl-7's use + the .app_data line + the sibling test are all preserved byte-identical.
8. **DQ attribution:** `from: "impl"` only. NEVER `answered_by: "advisor"` / `"user"`. NEVER `kind: "clarify"` / `"validate-result"` / `"validate-failed"`. Per `.claude/rules/decision-queue.md` hard refusals.
9. **Mandatory post-task retro** before exit (`.claude/rules/post-task-retro.md`): `memory_write_eval`, `source_ref` = your exact worker branch name (the Stop hook on `junior/*` branches requires `source_ref` = branch + a fresh `Task retro:` row in the last 30 min). Do NOT forge `created_at`, do NOT use raw SQL, do NOT modify the hook — those bypass attempts are tracked.

## 5. §2.4 mandatory-lesson firing record (advisor audit)

Authored under `.claude/rules/advisor-orchestrator.md` §2.4. File list = `crates/server/tests/e2e.rs` (1 fn, one-token deref on one line). Table matches fired:

- `crates/server/tests/e2e.rs` (any edit) → `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md` → §3 items 7, 8. Case A: the agpl fn outer is already `lemmy_utils::error::LemmyResult<()>`; the `**` deref introduces no `?` — Case A trivially preserved (no error-shape decision).
- e2e-edit-hang discipline plan-bound (plan §13 Task 4 GOTCHA 3) → `feedback_junior_worker_e2e_edit_hang.md` → §3 item 9 + §4.1. This is the smallest possible edit (one token, one line, one Edit).
- `feedback_read_canonical_before_writing_spec.md` → §3 item 10 (the canonical in-file idiom is `(**context).clone()` at 16 sites incl e2e.rs:2363; mirror byte-for-byte, no invented deref form).
- §G4 class: this is a **NON-allowlist** fail (E0308 compile error — not E0432/deprecated/clippy-doc/LemmyError-class). The §G4 verbatim-row blockquote gate does NOT apply (allowlist-only). Cycle-count: this is the **1st `(E0308, e2e.rs)`** occurrence (the prior 3x-same-surface was the now-overturned `/api/v4/site` 500 RUNTIME class — a different class). This is a hand-authored brief for a one-token follow-on correction of fix-impl-7, authorized under the **existing user §G4 override DQ #247** (user chose "fix-impl-7b under DQ#247 override" via AskUserQuestion 2026-05-18 — the deref completes the same authorized "wire SessionMiddleware" change; no fresh override needed). A subsequent failure on a NEW surface = surface + WAIT user (do not auto-author); a repeat `(E0308, e2e.rs)` after this (cycle 2/3) classifies normally, cycle 3 = §G4 hard-refusal re-plan.
- §2.3 PMD presearch: lane PMD DB is the per-worktree DB (lesson corpus indexed in canonical DB only — known lane-DB-isolation issue, Task #6; non-blocking — §2.4 mechanical injection is the load-bearing path; lessons read from disk at `.claude/lessons/`). Canonical-schema-first satisfied: the fix mirrors the proven 16-site in-file idiom `(**context).clone()` (canonical at e2e.rs:2363) — explicitly cited, byte-for-byte.
