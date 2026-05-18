# v1-ship-1 fix-impl-8 — mirror full real-server middleware stack in the agpl test App

## 1. Role + dispatch line

`[role:impl-task] v1-ship-1 fix-impl-8 — see .claude/PRPs/briefs/v1-ship-1-fix-impl-8.md`

Dispatched as a normal `[role:impl-task]` Junior task, `base_branch=phase-v1-ship-1`.

## 2. Scope

### 2.1 Why (root cause, source-quoted — DO NOT re-derive)

§5.2 Phase-2 e2e on the prior tip `8a2e26e37` FAILED (DQ #251 fail; suite
89 passed / 1 failed / 5 ignored; ONLY `agpl_source_disclosure_surface_returns_notice`
fails — NO regression). fix-impl-6 Part A's body-on-failure assert made
the cause **legible** at `crates/server/tests/e2e.rs:14921:3`:

```
/api/v4/site must return 200 — body: Requested application data is not
configured correctly. View/enable debug logs for more details.
  left: 500  right: 200
```

That body is actix-web's generic `Data::<T>::from_request` message for an
**UNREGISTERED `web::Data<T>`** in the handler-extractor chain.
`SessionMiddleware` (fix-impl-7) was **necessary but INSUFFICIENT**:

- The real server App (`crates/server/src/lib.rs:379-382`) wraps:
  ```rust
  .app_data(Data::new(context.clone()))            // :379
  .wrap(FederationMiddleware::new(federation_config.clone())) // :380
  .wrap(IdempotencyMiddleware::new(idempotency_set.clone()))  // :381
  .wrap(SessionMiddleware::new(context.clone()))              // :382
  ```
- The agpl test App (`e2e.rs:14908-14909`) wraps **only**
  `.app_data(Data::new(context.clone()))` + `.wrap(SessionMiddleware::new((**context).clone()))`.
- `FederationMiddleware::call`
  (`activitypub_federation-0.7.0-beta.10/src/actix_web/middleware.rs:57`)
  does `req.extensions_mut().insert(self.config.clone())`;
  `activitypub_federation`'s `impl FromRequest for Data<T>` (ibid) errors
  if the `FederationConfig<T>` extension is absent. With
  `FederationMiddleware` not wrapped, the handler-extractor chain's `Data`
  extraction fails → HTTP 500 on `/api/v4/site`.

**§G4 authorisation:** the user authorised this fix in-channel
(`AskUserQuestion`, 2026-05-18) — recorded at **DQ #252**
(`answered_by: user`). The §G4 override scope (DQ #247 + #249) is
**EXTENDED** to cover this full-real-server-middleware-stack mirror. This
is a diagnostic ADVANCE (the failure surface CHANGED: read_site-string
500 → actix-Data-string 500), NOT a 3rd same-tuple cycle repeat.

### 2.2 The change (exactly three coupled edits, one anchored region, one commit)

In `crates/server/tests/e2e.rs`, **inside the single
`agpl_source_disclosure_surface_returns_notice` test fn only** (the
`use` block + the seed block + the App builder are all in this one fn,
~e2e.rs:14860-14913):

**(a) Add two imports** alongside the existing
`use lemmy_routes::middleware::session::SessionMiddleware;` (currently
~e2e.rs:14877), mirroring `lib.rs:3` + `lib.rs:34`:

```rust
use activitypub_federation::config::FederationMiddleware;
use lemmy_routes::middleware::idempotency::{IdempotencyMiddleware, IdempotencySet};
```

**(b) Build a `FederationConfig`** AFTER the existing seed block (after
the `LocalSiteRateLimit::create(...)` line, ~e2e.rs:14904) and BEFORE
`let rate_limit = RateLimit::with_debug_config();` / `let app =
test::init_service(...)`. **Mirror the PROVEN in-file canonical idiom at
e2e.rs:2361-2368 byte-for-byte** (this idiom is used at e2e.rs:2361-2368
AND e2e.rs:3094-3102 — read 2361-2368 first and copy its exact shape):

```rust
let federation_config = activitypub_federation::config::FederationConfig::builder()
    .domain(context.settings().hostname.clone())
    .app_data((**context).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await?;
```

(`context` here is the `Data<LemmyContext>` returned by
`governance_fixtures::bootstrap()`; `(**context).clone()` is the proven
16-site in-file deref to a bare `LemmyContext`. The outer fn returns
`lemmy_utils::error::LemmyResult<()>` so the bare `?` is correct — DO NOT
add a `.map_err`; mirror the 2361-2368 form which uses bare `?`, not the
3094-3102 form which uses `.map_err(|e| anyhow::anyhow!("{e}"))?`.)

**(c) In the App builder**, insert the two middleware wraps **AHEAD of
the existing `SessionMiddleware` wrap**, mirroring the exact real-server
stack order at `lib.rs:380-382`:

```rust
let app = test::init_service(
    App::new()
        .app_data(Data::new(context.clone()))
        .wrap(FederationMiddleware::new(federation_config.clone()))
        .wrap(IdempotencyMiddleware::new(IdempotencySet::default()))
        .wrap(SessionMiddleware::new((**context).clone()))
        .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit)),
)
.await;
```

(The `.app_data(Data::new(context.clone()))` line and the
`.wrap(SessionMiddleware::new((**context).clone()))` line are UNCHANGED;
the only additions are the two `.wrap(...)` lines between them, in that
order.)

### 2.3 Hard boundaries

- **ONLY** `crates/server/tests/e2e.rs` is edited. NO production code, NO
  `crates/api`, NO `crates/db_schema`, NO `crates/routes`, NO
  `crates/server/src/`. If you feel pressure to touch any of those →
  STOP and raise `kind: "blocker"` DQ.
- **ONLY** the `agpl_source_disclosure_surface_returns_notice` test fn is
  touched. Do NOT edit the passing sibling
  `all_mvp_endpoints_return_non_404` (e2e.rs:~3849-3862) or the canonical
  `FederationConfig` builder sites at 2361-2368 / 3094-3102 — those are
  READ-ONLY reference, mirror them, do not modify them.
- Preserve byte-identical: fix-impl-6 Part A (the two body-on-failure
  asserts for `/api/v4/site` + `/api/v4/source`), fix-impl-6 Part B (the
  complete `SiteInsertForm` with `ap_id`/`inbox_url`/`public_key`/
  `private_key`/`last_refreshed_at` + `generate_actor_keypair` + the
  instance/sysacct/LocalSite/LocalSiteRateLimit seed), fix-impl-7's
  `SessionMiddleware` use + the `.app_data` line + the `(**context).clone()`
  deref.
- Net diff expected: ONE file (`e2e.rs`), roughly **+8 / -1** (2 import
  lines + ~7 `FederationConfig` builder lines + 2 `.wrap` lines; the `-1`
  is only if the App builder reflows — likely `+9/-0`). If your diff
  touches any other file or any other fn → it is wrong; STOP.

## 3. Required reading (read these first, in order, before any edit)

1. `crates/server/tests/e2e.rs:2325-2375` — the **canonical in-file
   `FederationConfig::builder()` idiom to mirror byte-for-byte** (the
   `.domain().app_data((**context).clone()).debug(true).http_fetch_limit(0).build().await?`
   form). This is the canonical-schema-first source for (b).
2. `crates/server/tests/e2e.rs:3088-3110` — second instance of the same
   idiom (confirms the pattern; note it uses `.map_err(|e|
   anyhow::anyhow!("{e}"))?` — use the **2361-2368 bare-`?` form**
   instead, since the agpl fn returns `LemmyResult<()>`).
3. `crates/server/src/lib.rs:375-385` — the real-server App middleware
   stack (the wrap ORDER to mirror in (c): Federation → Idempotency →
   Session; imports at lib.rs:3 + lib.rs:34 for (a)).
4. `crates/server/tests/e2e.rs:14860-14925` — the full agpl test fn (the
   single anchored edit region; confirm the `use` block, the seed block
   end, and the App builder before editing).
5. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Case A**:
   the agpl test fn outer is `lemmy_utils::error::LemmyResult<()>`; the
   `FederationConfig` build uses a bare `?` (mirror 2361-2368). Do NOT
   introduce `Box<dyn Error>` or a `.map_err` bridge.
6. `.claude/lessons/feedback_async_pool_test_pattern.md` — async pool /
   `Data<LemmyContext>` test-context discipline.
7. `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — a
   targeted **anchored** Edit into the single test fn is safe (fix-impl-5/6/7/7b
   all proved this); NEVER attempt a full-file e2e Edit. Make 2-3
   small anchored Edits, not one giant rewrite.
8. `.claude/lessons/feedback_read_canonical_before_writing_spec.md` —
   mirror the existing in-file `FederationConfig` usage (2361-2368) +
   the existing `IdempotencySet::default()` pattern (lib.rs:381); do NOT
   invent a new builder shape or import path.

## 4. Constraints

- **Canonical-schema-first (mandatory):** the `FederationConfig` builder
  in (b) MUST be a byte-for-byte mirror of e2e.rs:2361-2368 (bare `?`
  form). The `FederationMiddleware`/`IdempotencyMiddleware` wraps + their
  imports MUST mirror `lib.rs:3`, `lib.rs:34`, `lib.rs:380-381`. Verify
  the exact import paths by reading those lines — do NOT guess.
- **Single anchored edit region**, the `agpl_source_disclosure_surface_returns_notice`
  fn only. 2-3 small anchored Edits (imports / federation_config / App
  builder). NO full-file Edit (e2e edit-hang lesson).
- **Outer fn signature unchanged:** `async fn
  agpl_source_disclosure_surface_returns_notice() ->
  lemmy_utils::error::LemmyResult<()>`. Bare `?` on the
  `FederationConfig` build (Case A — mirror 2361-2368, NOT 3094-3102).
- **§5.2 Phase-1 pre-push gate (mandatory):** BEFORE pushing the worker
  branch, run `bash scripts/brehon/cargo-check.sh --workspace --features
  full` (or `cmd //c scripts\brehon\cargo-check.bat --workspace
  --features full` on the Windows worker). Non-zero exit → fix in the
  same commit if in-scope (e2e.rs only) OR raise `kind: "blocker"` DQ if
  out-of-scope. NEVER `#[allow]`-spam to bypass. (Local `cargo check` is
  ~30s warm; one ci-watcher/laptop cycle is ~5+ min — net positive.)
- **Commit message (exact):**
  `test(e2e): wrap FederationMiddleware + IdempotencyMiddleware in agpl test App to mirror real-server stack so /api/v4/site returns 200 (fix-impl-8)`
- **Next DQ id** (if you must raise a blocker): compute
  `max(id) + 1` across `.claude/decision-queue.json` pending+resolved
  **AND** all `.claude/decision-queue-archive-*.json`. Push the DQ
  commit to the worker branch immediately (mid-task visibility).
- **HANDOVER trailer:** append a `HANDOVER:` YAML trailer to the commit
  body (filesCreated/filesModified/keyDecisions/notes) per the impl-task
  template — note which canonical sibling line range you mirrored for
  the `FederationConfig` builder.
- This is the §G4-authorised fix per **DQ #252** (`answered_by: user`).
  Scope of the override now spans fix-impl-7, fix-impl-7b, AND this
  full-middleware-stack mirror. Do NOT expand scope beyond §2.2's three
  edits. If §2.2 is insufficient (a NEW failure surface), that is a
  fresh user decision — raise `kind: "blocker"` and STOP; do NOT
  improvise additional middleware or production changes.
