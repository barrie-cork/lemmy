# Plan: v1-quality-r3b — Capture DB URL at LemmyContext::create (Issue #167)

## 1. Summary

This sub-phase resolves GitHub Issue #167 by capturing the Postgres database
URL **once** when `LemmyContext` is constructed (storing it as a `db_url: String`
struct field), exposing it through a `database_url(&self) -> &str` accessor, and
switching the `admin_audit_stream` SSE handler to read the captured value instead
of re-reading `LEMMY_DATABASE_URL` from the live process environment on every
LISTEN connection. The headline acceptance condition: the three
`admin_audit_stream_*` e2e tests pass **without** their per-test
`EnvVarGuard::set("LEMMY_DATABASE_URL", …)` band-aid lines, and
`get_database_url` no longer appears anywhere in `admin_audit_stream.rs`.

## 2. Source

- GitHub Issue **#167** — "admin_audit_stream re-reads LEMMY_DATABASE_URL from
  live env; capture at context-create time instead" (the issue this sub-phase closes).
- Resolved DQ **`a3d0e9941441-039`** (`kind: clarify`, `answered_by: advisor`) @ `37d79dff5`
  — fix shape = **Option A** (capture-at-create, SETTINGS-internal); accessor named
  `database_url(&self) -> &str`; **zero callsite changes** (no new `create()` params).
- Resolved DQ **`052f0d5c016d-001`** (`kind: blocker`, `answered_by: advisor`) @ `37d79dff5`
  — confirms brief §2/§4 prose was stale (it described an Option-B `db_url()` accessor +
  callsite enumeration); Option A with `database_url()` is authoritative; the
  `admin_audit_stream.rs:125` call switches to `context.database_url()`.
- `.claude/lessons/feedback_envvarguard_fixture_lifetime_footgun.md` — binds §4 / §10.2:
  the guard drops at `bootstrap()` return, *before* the handler reads the live env; this
  is the root cause #167 fixes. Capture-at-create eliminates the footgun.
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — binds §10.3: no new error
  conversion is introduced (the handler keeps its existing `LemmyResult` / `.into()`
  shape); cited so the impl agent does not add a `Box<dyn Error>` return.
- `.claude/lessons/feedback_rust_visibility_cross_crate.md` — binds §10.1: the accessor
  MUST be `pub` because it is called from `lemmy_api` (a different crate from
  `lemmy_api_utils` where `LemmyContext` lives).
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — binds §10.3 / §13 Task 1:
  the e2e edit cites verbatim `old_string`/`new_string` anchors at lines 8255–8256,
  8280–8281, 8352–8353 to de-risk editing the ~17k-line `e2e.rs`.
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — binds §10.4:
  this fix introduces **no new DB writes**; the transaction-gate is closed (noted so the
  impl agent does not wrap the LISTEN connection in a transaction).

## 3. Problem statement

`crates/api/api/src/governance/admin_audit_stream.rs:125` currently does:

```rust
let db_url = context.settings().get_database_url();
let (pg_client, pg_conn) = tokio_postgres::connect(&db_url, NoTls).await…
```

`Settings::get_database_url()` (`crates/utils/src/settings/mod.rs:49`) reads
`LEMMY_DATABASE_URL` from the **live process environment at call time**. In the
e2e harness, the testcontainer URL is set by an `EnvVarGuard` inside
`admin_config_fixtures::bootstrap()` (`crates/server/tests/e2e.rs:6128`), but that
guard **drops when `bootstrap()` returns** — before the handler runs. The three
`admin_audit_stream_*` tests papered over this with their own per-test
`EnvVarGuard::set(…)` lines (the v1-quality-r2 fix-impl-1 band-aids at
`e2e.rs:8256/8281/8353`). The band-aid is fragile: any handler that re-reads the
env after the fixture guard drops, in any test that forgets the band-aid, silently
connects to the wrong (or no) database. → addressed by §13 Task 1, §10.1–§10.3.

## 4. Solution statement

Capture the URL once, at construction, and read the captured copy thereafter.

```
BEFORE (live env re-read at handler time):

  bootstrap() ── EnvVarGuard sets LEMMY_DATABASE_URL ──┐
       │                                               │ (guard scope)
       └── LemmyContext::create(…)                     │
       └── returns; guard DROPS ─────────────────── env UNSET ✗
                                                        │
  admin_audit_stream(context)                           │
       └── context.settings().get_database_url()  ← reads live env (now unset) ✗
              │  test body re-adds EnvVarGuard band-aid to compensate
              └── tokio_postgres::connect(&db_url)

AFTER (capture at create, read captured copy):

  bootstrap() ── EnvVarGuard sets LEMMY_DATABASE_URL ──┐
       │                                               │ (guard scope)
       └── LemmyContext::create(…)                     │
              └── db_url = SETTINGS.get_database_url()  ← captured HERE while guard live ✓
       └── returns; guard DROPS ── env unset, but db_url FIELD retains value ✓

  admin_audit_stream(context)
       └── context.database_url()                 ← reads captured field ✓
              └── tokio_postgres::connect(db_url)   (note: db_url is &str — drop the &)
```

`LemmyContext` gains a `db_url: String` field, populated inside `create()` by an
internal `SETTINGS.get_database_url()` call (no new parameter). A `pub fn
database_url(&self) -> &str` accessor exposes it. The handler reads
`context.database_url()`. The three test-body band-aid guards are removed; the
fixture-internal guard at `e2e.rs:6128` stays (it is what makes the capture see the
testcontainer URL). From §4 alone the reader can predict §11: `context.rs`,
`admin_audit_stream.rs`, `e2e.rs`.

## 4a. Watchpoints

| Watchpoint | File:location | What to watch for |
|---|---|---|
| `LemmyContext` struct gains `db_url` field | `crates/api/api_utils/src/context.rs:12-21` | Field present and correctly typed `db_url: String`; `create()` captures from `SETTINGS.get_database_url()` before struct literal |
| `database_url()` accessor visible cross-crate | `crates/api/api_utils/src/context.rs` (new method) | Accessor is `pub fn database_url(&self) -> &str`; NOT `pub(crate)` — called from `lemmy_api` |
| Handler switches to captured URL | `crates/api/api/src/governance/admin_audit_stream.rs:125-126` | `get_database_url()` replaced by `context.database_url()`; `&db_url` → `db_url` (no `&`) at line 126 |
| Fixture-internal guard preserved | `crates/server/tests/e2e.rs:6128` | `EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url)` line at :6128 **must not be removed** — it is what makes the capture see the testcontainer URL |
| Only 3 band-aids removed in e2e | `crates/server/tests/e2e.rs:8256/8281/8353` | Exactly those 3 lines deleted; no other `EnvVarGuard` lines touched |

---

## 5. Metadata

- **Phase:** `v1-quality-r3b`
- **Branch:** `phase-v1-quality-r3b` (cut by BM-task before Task 1)
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 3 (Task 0 pre-flight + Task 1 impl + Task 2 retro)
- **Estimated cargo budget:** ~6 GB peak (the gate-4 `--workspace --test e2e` run; the
  per-task `cargo check`/`clippy` are well under)
- **Forbidden-window applicability:** standard (per advisor-orchestrator.md table).
  Shape G is **SUSPENDED until 2026-06-01**, so validation runs laptop-side via the
  `validate-pending-laptop` handler (inline cargo §15, Windows bat-wrapper form).
- **Complexity score:** `6/10` — see breakdown below

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | Only Task 1 is an impl task (Task 0 + retro excluded); 1 ≤ 5 |
| Migrations touched | +2 each | 0 | Handler-only; no schema change |
| Crates touched | +1 each | 3 → +3 | `lemmy_api_utils`, `lemmy_api`, `lemmy_server` (test) |
| `crates/server/tests/e2e.rs` edits | +3 each | 1 → +3 | 3 guard removals + 3 `db_url`→`_db_url` renames, all in one file |
| New ADR-affecting decisions | +2 each | 0 | No ADR superseded |
| Cargo budget peak above 6 GB | +1 per GB | 0 | Peak ≈ 6 GB, not above |
| **Total** | — | **6** | Threshold for split-DQ: `>8` (Sonnet). 6 ≤ 8 → **proceed-as-one** |

Score 6 is below the Sonnet split threshold of 8 → no split-DQ filed. The three
crates are touched by a single atomic change (the field/accessor, its sole call
site, and the test that exercises it) that must land together to stay compile-green;
splitting would create a red intermediate state. The e2e-edit risk is mitigated
in-task via verbatim anchors (§10.3) per `feedback_fix_impl_pre_locate_e2e_anchors.md`,
which is the brief-specified design for this single-impl-task scope.

### 5.2 Per-task complexity ceiling

Target is Sonnet, so the hard per-task split rule (non-Sonnet only) does not apply.
Task 1 touches 3 files across 3 crates — at the Sonnet norm ceiling (≤4 files / ≤2
crates is the soft norm; the 3-crate spread here is intrinsic to one atomic change
and is accepted per §5.1). No task is split.

## 6. Relationship to other v1-quality sub-phases

- **Follows** v1-quality-r2 (which introduced the `EnvVarGuard` band-aids in
  `fix-impl-1`). This sub-phase supersedes those band-aids with the architectural fix.
- **Independent of** Issue #158 (`emit_reputation_event` helper extraction) — explicitly
  out of scope (§12); premature-DRY gate (only 2 callers).
- **No downstream dependency** — nothing else is blocked on #167.

## 7. Preflight guardrails inherited from prior phases

- **R5:** Task 0 enumerates ALL probes explicitly; do NOT inherit implicitly (per JM-b
  retro-events Event 4).
- **R6:** all clippy invocations use `--no-deps --features full` uniformly.
- **R-env:** any change touching env-var acquisition paths runs gate-4 full e2e
  (`--workspace --test e2e --features full`), per
  `feedback_gate4_full_e2e_env_refactor_class.md`. This fix changes *when*
  `get_database_url()` is read, so gate-4 is mandatory (§15.4).
- **R-e2e-anchor:** every `e2e.rs` edit cites verbatim `old_string`/`new_string`
  anchors with surrounding context lines (per `feedback_fix_impl_pre_locate_e2e_anchors.md`),
  because the file is ~17k lines and blind edits hang the worker.
- **R-visibility:** cross-crate accessors are `pub` (per `feedback_rust_visibility_cross_crate.md`).

## 8. Flow design

```
LemmyContext::create(pool, client, pictrs, secret, rate_limit)      [Task 1, context.rs:24-38]
   │
   ├── db_url = SETTINGS.get_database_url()   ← NEW: capture while env is live
   └── LemmyContext { …, db_url }             ← NEW field

LemmyContext::database_url(&self) -> &str { &self.db_url }          [Task 1, context.rs new accessor]
   ▲
   │ called by
admin_audit_stream(context, …)                                     [Task 1, admin_audit_stream.rs:125-126]
   ├── let db_url = context.database_url();   ← was context.settings().get_database_url()
   └── tokio_postgres::connect(db_url, NoTls) ← drop the & (db_url is now &str, not String)

admin_config_fixtures::bootstrap()                                 [Task 1, e2e.rs:6128 UNCHANGED]
   └── _g_db_url = EnvVarGuard::set(LEMMY_DATABASE_URL, &db_url)    ← stays: makes capture see testcontainer URL
   └── LemmyContext::create(…)                                     ← capture happens here, guard live

admin_audit_stream_forbidden_for_non_admin / _enforces_per_admin_cap / _emits_frame_on_config_change
   └── let (_container, context, _db_url) = bootstrap().await?     [Task 1, e2e.rs:8255/8280/8352 rename]
   └── (REMOVED) let _g_db_url = EnvVarGuard::set(…)               [Task 1, e2e.rs:8256/8281/8353 delete]
```

## 9. Mandatory reading

- **Schema/type definitions** — `crates/api/api_utils/src/context.rs:12-59` (the
  `LemmyContext` struct, `create()`, and the existing accessor pattern such as
  `secret(&self) -> &Secret` at 54-56).
- **Settings accessor** — `crates/utils/src/settings/mod.rs:49` (`pub fn
  get_database_url(&self) -> String`, infallible; this is what `create()` calls
  internally). Note `get_database_url_with_options` at :91 is NOT used.
- **Call site to change** — `crates/api/api/src/governance/admin_audit_stream.rs:125-132`
  (the `get_database_url()` read + `tokio_postgres::connect`).
- **Fixture (do NOT change)** — `crates/server/tests/e2e.rs:6107-6168`
  (`admin_config_fixtures::bootstrap()`; the guard at :6128 must stay).
- **Test bodies to edit** — `crates/server/tests/e2e.rs:8251-8262`, `8276-8283`,
  `8344-8355` (the three `admin_audit_stream_*` tests).
- **Lessons** — `feedback_envvarguard_fixture_lifetime_footgun.md`,
  `feedback_fix_impl_pre_locate_e2e_anchors.md`, `feedback_rust_visibility_cross_crate.md`,
  `feedback_lemmy_error_no_std_error.md`, `feedback_multi_write_handlers_need_transactions.md`.

## 10. Patterns to mirror

### 10.1 Struct field + accessor on LemmyContext

**Mirror:** `crates/api/api_utils/src/context.rs:12-21` (struct fields) + `:54-56`
(the `secret` accessor shape) + `:24-38` (`create()` body).

```rust
// In the struct (after rate_limit_cell at :20):
pub struct LemmyContext {
  pool: ActualDbPool,
  client: Arc<ClientWithMiddleware>,
  pictrs_client: Arc<ClientWithMiddleware>,
  secret: Arc<Secret>,
  rate_limit_cell: RateLimit,
  /// Database URL captured at construction time from `SETTINGS.get_database_url()`.
  /// Read this (via `database_url()`) instead of re-reading `LEMMY_DATABASE_URL`
  /// from the live process environment — the env var may be unset by the time a
  /// handler runs (the test EnvVarGuard drops at fixture return). See issue #167
  /// and feedback_envvarguard_fixture_lifetime_footgun.md.
  db_url: String,
}

// In create() body — capture before constructing the struct:
pub fn create(
  pool: ActualDbPool,
  client: ClientWithMiddleware,
  pictrs_client: ClientWithMiddleware,
  secret: Secret,
  rate_limit_cell: RateLimit,
) -> LemmyContext {
  let db_url = SETTINGS.get_database_url();   // NEW — SETTINGS already imported at :5-8
  LemmyContext {
    pool,
    client: Arc::new(client),
    pictrs_client: Arc::new(pictrs_client),
    secret: Arc::new(secret),
    rate_limit_cell,
    db_url,                                    // NEW
  }
}

// New accessor — mirror the `secret(&self) -> &Secret` shape; MUST be pub
// (called from lemmy_api, a different crate — feedback_rust_visibility_cross_crate.md):
pub fn database_url(&self) -> &str {
  &self.db_url
}
```

**GOTCHA:** `SETTINGS` is already imported (`crates/api/api_utils/src/context.rs:5-8`);
do NOT add an import. `create()` is **not** `async` and `get_database_url()` is
infallible (`-> String`, no `?`), so the body stays synchronous and panic-free.

### 10.2 Handler: read captured URL, drop the borrow

**Mirror:** `crates/api/api/src/governance/admin_audit_stream.rs:125-126`.

```rust
// BEFORE:
let db_url = context.settings().get_database_url();        // db_url: String
let (pg_client, pg_conn) = match tokio_postgres::connect(&db_url, NoTls).await {

// AFTER:
let db_url = context.database_url();                       // db_url: &str
let (pg_client, pg_conn) = match tokio_postgres::connect(db_url, NoTls).await {
//                                                        ^ NOTE: no & — see GOTCHA
```

**GOTCHA (the one that bites):** the accessor returns `&str`, not `String`. The old
code passed `&db_url` (i.e. `&String`, which coerces to `&str`). The new `db_url` is
already `&str`, so `&db_url` would be `&&str` — and `tokio_postgres::connect<T>` where
`T: TryInto<Config>` is **not** implemented for `&&str`. **Drop the `&`** at line 126:
pass `db_url` directly. Forgetting this is an `E0277` "trait bound not satisfied".

### 10.3 e2e: remove band-aid guards, rename now-unused binding

**Mirror (verbatim anchors — three byte-identical sites):** `e2e.rs:8255-8256`,
`8280-8281`, `8352-8353`. Per `feedback_fix_impl_pre_locate_e2e_anchors.md`, each edit
uses these exact `old_string` → `new_string` pairs (the surrounding test-fn context at
8251 / 8276 / 8344 disambiguates the otherwise-identical two-line blocks):

```rust
// old_string (all three sites — identical two lines):
  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

// new_string (all three sites — guard line deleted, binding renamed to silence unused):
  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
```

**GOTCHA:** after deleting the `_g_db_url` line, the third tuple element `db_url` is
**unused** (verified: it was referenced only by the deleted guard line in each test).
Leaving it as `db_url` triggers an `unused_variable` warning → fails
`clippy -- -D warnings`. Rename it to `_db_url` (underscore prefix). Do NOT delete the
binding position — `bootstrap()` returns a 3-tuple `(container, context, String)`; you
must still destructure all three. Do NOT touch the fixture-internal guard at `e2e.rs:6128`
— it stays (it is what makes the capture-at-create see the testcontainer URL).

### 10.4 No new DB writes (transaction gate closed)

**Mirror:** none — this is a *negative* pattern. Per
`feedback_multi_write_handlers_need_transactions.md`, multi-write handlers need a
transaction; this fix adds **zero** DB writes (it only changes *where* the connection
string comes from). Do NOT wrap the existing `tokio_postgres::connect` / LISTEN logic in
a new transaction — the handler's write surface is unchanged.

## 11. Files to change

**`lemmy_api_utils`** (`crates/api/api_utils/`):
- `crates/api/api_utils/src/context.rs` — add `db_url: String` field; capture
  `SETTINGS.get_database_url()` inside `create()`; add `pub fn database_url(&self) -> &str`
  accessor (Task 1).

**`lemmy_api`** (`crates/api/api/`):
- `crates/api/api/src/governance/admin_audit_stream.rs` — switch line 125 to
  `context.database_url()`; drop the `&` at line 126 (Task 1).

**`lemmy_server`** (`crates/server/`, test only):
- `crates/server/tests/e2e.rs` — delete the 3 `_g_db_url = EnvVarGuard::set(…)` band-aid
  lines (8256/8281/8353); rename the now-unused `db_url` binding to `_db_url` at the 3
  destructure sites (8255/8280/8352). Fixture `bootstrap()` at :6107-6168 is **unchanged** (Task 1).

### Struct-field add: enumerate all callsites (mandatory)

Per `feedback_planner_enumerate_struct_callsites_for_addfield.md`: `LemmyContext` gains a
field. Under **Option A** the field is populated **internally** by `create()` (no new
parameter), so every existing `LemmyContext::create(…)` call site is **source-compatible
and needs no change**. The struct is never built via a `LemmyContext { … }` literal
outside `create()` itself (the single literal is at `context.rs:31-37`, which Task 1
edits). Enumeration of all 16 `LemmyContext::create` call sites (pre-populated by the
advisor in brief §3a) confirms this — none pass `db_url`, none break:

```
crates/api/api_utils/src/context.rs:79   ← the factory's own test helper; create() body edited here
crates/server/src/lib.rs:210             ← main server startup; no change (no new param)
crates/server/tests/e2e.rs: 861, 2602, 3366, 4148, 4472, 4796, 4925, 5050, 5096,
                            5673, 5869, 6159, 16816, 17464  ← 14 sites; no change
```

**Caller crates (compile-only-after-Task-1): none** — because zero new parameters are
added, all callers compile unchanged. The only literal-construction site is inside
`create()` (edited in Task 1).

## 12. NOT building in v1-quality-r3b

- **Issue #158 (`emit_reputation_event` helper extraction)** — deferred; reason:
  premature-DRY gate (only 2 callers). Explicitly excluded per brief §2.
- **`BREHON_DISABLE_*` env-guard sweep in e2e.rs** — deferred to a separate sweep;
  reason: out of scope for #167 (different env vars, different tests).
- **Migrations / new endpoints** — none; reason: this is a handler-internal refactor only.
- **Changing `get_database_url()` itself or `get_database_url_with_options`** — deferred;
  reason: #167 changes *when/where* the URL is read, not *how* Settings computes it.

---

## 13. Step-by-step tasks

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify the environment is ready for `v1-quality-r3b`; confirm branch is
`phase-v1-quality-r3b`; confirm `context.rs`, `admin_audit_stream.rs`, and the three
e2e anchors are intact on the base; confirm a clean clippy baseline.

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate ALL probes):**

```bash
# Probe 0 — Docker daemon (e2e uses testcontainers)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — wrapper sanity: cargo-check honors -p (only lemmy_api_utils compiles)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_utils > .claude/PRPs/debug/v1-quality-r3b-audit-p.log 2>&1"
echo "exit: $?"; tail -20 .claude/PRPs/debug/v1-quality-r3b-audit-p.log

# Probe 2 — feature flag activation
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/PRPs/debug/v1-quality-r3b-audit-features.log 2>&1"
echo "exit: $?"; tail -20 .claude/PRPs/debug/v1-quality-r3b-audit-features.log

# Probe 3 — cargo-test wrapper honors target selection (e2e compiles, no run)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-quality-r3b-audit-test.log 2>&1"
echo "exit: $?"; tail -20 .claude/PRPs/debug/v1-quality-r3b-audit-test.log

# Probe 4 (negative) — wrapper propagates non-zero exit on bogus feature
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/v1-quality-r3b-audit-neg.log 2>&1"
echo "cargo-check.bat exit on bogus feature: $?"   # EXPECT non-zero (101)

# Probe 5 — anchors intact: the 3 band-aid guards still present on base
grep -n 'EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url)' crates/server/tests/e2e.rs
# EXPECT: lines 8256, 8281, 8353 (plus the fixture-internal one at 6128 — do NOT remove that)

# Probe 6 — handler call site intact on base
grep -n "context.settings().get_database_url()" crates/api/api/src/governance/admin_audit_stream.rs
# EXPECT: line 125

# Probe 7 — clippy baseline clean on the three target crates (scoped, --no-deps)
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r3b-audit-clippy.log 2>&1"
echo "exit: $?"; tail -40 .claude/PRPs/debug/v1-quality-r3b-audit-clippy.log

# Probe 8 — concurrent-PR check (no open PR touches our 3 files)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("context\\.rs|admin_audit_stream\\.rs|tests/e2e\\.rs")) | {number, title, headRefName}'
# EXPECT: empty output
```

**EXPECT block:**
- Probes 0,1,2,3,7 exit 0; Probe 4 exits NON-ZERO (101).
- Probe 5 prints lines 8256/8281/8353 (+ 6128). Probe 6 prints line 125. Probe 8 empty.

**No commit at Task 0** — verification only.

### Task 1: Capture DB URL at create; switch handler; drop test band-aids

**ACTION:** Add a `db_url: String` field + `database_url()` accessor to `LemmyContext`,
populate it inside `create()` from `SETTINGS.get_database_url()`, switch the
`admin_audit_stream` handler to read it, and remove the three e2e band-aid guards.

**FILES (machine-parseable, used by `/brehon-verify` + cohort dispatch):**

```yaml
creates: []
modifies:
  - crates/api/api_utils/src/context.rs           # add db_url field + accessor + capture in create()
  - crates/api/api/src/governance/admin_audit_stream.rs   # read context.database_url(); drop & at connect
  - crates/server/tests/e2e.rs                    # delete 3 guard lines; rename db_url->_db_url at 3 sites
```

(No `[P]` marker — single impl task; no cohort peers. No `requires:` — self-contained.)

**IMPLEMENT (file 1 of 3):** in `crates/api/api_utils/src/context.rs`, add the
`db_url: String` field to the struct (after `rate_limit_cell` at :20, with the verbatim
doc-comment from §10.1), add the `let db_url = SETTINGS.get_database_url();` capture inside
`create()` before the struct literal, add `db_url` to the struct literal, and add the
`pub fn database_url(&self) -> &str { &self.db_url }` accessor (mirror the `secret`
accessor at :54-56). Use the verbatim code from §10.1.

**MIRROR:** `crates/api/api_utils/src/context.rs:12-21` (struct), `:24-38` (`create()`),
`:54-56` (accessor shape).

**IMPLEMENT (file 2 of 3):** in `crates/api/api/src/governance/admin_audit_stream.rs`,
change line 125 from `context.settings().get_database_url()` to `context.database_url()`,
and at line 126 change `tokio_postgres::connect(&db_url, NoTls)` to
`tokio_postgres::connect(db_url, NoTls)` (drop the `&`). Use §10.2 verbatim.

**MIRROR:** `crates/api/api/src/governance/admin_audit_stream.rs:125-132`.

**IMPLEMENT (file 3 of 3):** in `crates/server/tests/e2e.rs`, at each of the three
`admin_audit_stream_*` tests, delete the `let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);`
line and rename the destructured `db_url` → `_db_url`. Use the verbatim
`old_string`/`new_string` pairs from §10.3 (sites at 8255-8256, 8280-8281, 8352-8353).
Do **not** touch the fixture-internal guard at :6128.

**MIRROR:** `crates/server/tests/e2e.rs:8255-8256`, `8280-8281`, `8352-8353`.

**GOTCHA:** (1) drop the `&` at `admin_audit_stream.rs:126` — `database_url()` returns
`&str`, so `&db_url` is `&&str` and fails `E0277` (§10.2). (2) rename the unused e2e
binding to `_db_url` or clippy `-D warnings` fails on `unused_variable` (§10.3). (3) do
not add a `SETTINGS` import — already present at `context.rs:5-8`.

**VALIDATE (story-checkpoint feeds §16a):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-quality-r3b-task1-check.log 2>&1"
echo "exit: $?"; tail -20 .claude/PRPs/debug/v1-quality-r3b-task1-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r3b-task1-clippy.log 2>&1"
echo "exit: $?"; tail -40 .claude/PRPs/debug/v1-quality-r3b-task1-clippy.log
# EXPECT: exit 0

# Grep gate (DoD §15.5): get_database_url gone from the handler
grep -c "get_database_url" crates/api/api/src/governance/admin_audit_stream.rs
# EXPECT: 0

cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-quality-r3b-task1-e2e.log 2>&1"
echo "exit: $?"; tail -40 .claude/PRPs/debug/v1-quality-r3b-task1-e2e.log
# EXPECT: exit 0; the 3 admin_audit_stream_* tests pass with no test-body env guard
```

**Commit:** `fix(governance): capture DB URL at LemmyContext::create (issue #167)`

### Task 2: Retro

**Goal:** author the retro per `feedback_retro_not_report.md` and
`feedback_four_role_retro_signals.md`. One H2 per role (Advisor / Planning / Impl / BM)
with signals + lessons. Promote any new lesson to `.claude/lessons/feedback_*.md` in the
same retro commit (per `feedback_one_system_memory_in_repo.md`). Include the per-task
complexity scorecard (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`) per
`feedback_retro_task_complexity_score.md`.

---

## 14. Testing strategy

- **Unit (compile-time):** `cargo check --workspace --features full` — confirms the field,
  capture, accessor, handler switch, and e2e edits all compile.
- **Lint:** `cargo clippy --workspace --features full --no-deps -- -D warnings` — confirms
  no `unused_variable` from the renamed e2e binding and no `E0277` from the borrow.
- **Test target compile:** covered by the e2e build leg below (the change touches the
  `LemmyContext` struct re-exported across crates).
- **e2e execution (gate-4, R-env):** `cargo test --workspace --test e2e --features full` —
  the three `admin_audit_stream_*` tests must pass **without** their test-body
  `EnvVarGuard` lines, proving capture-at-create works.
- **Migration round-trip:** N/A (no migrations).
- **Lesson gates:** `feedback_lemmy_error_no_std_error.md` (no new `Box<dyn Error>`),
  `feedback_fix_impl_pre_locate_e2e_anchors.md` (verbatim anchors used).

---

## 15. Validation commands (DoD)

> Shape G is SUSPENDED until 2026-06-01 → validation runs laptop-side via the
> `validate-pending-laptop` handler. Commands are the Windows bat-wrapper form (the laptop
> is the runner); the advisor dry-runs every one against current HEAD before plan approval.

### 15.1 Static analysis (Task 1)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-quality-r3b-task1-check.log 2>&1"
echo "exit: $?"   # EXPECT: exit 0
```

### 15.2 Lint (Task 1 — uniform R6, --no-deps)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r3b-task1-clippy.log 2>&1"
echo "exit: $?"   # EXPECT: exit 0
```

### 15.3 Test target compile (Task 1 — struct touched, re-exported across crates)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --no-run --features full > .claude/PRPs/debug/v1-quality-r3b-task1-testcompile.log 2>&1"
echo "exit: $?"   # EXPECT: exit 0
```

### 15.4 e2e test execution (gate-4 full — Task 1, R-env mandatory)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-quality-r3b-task1-e2e.log 2>&1"
echo "exit: $?"   # EXPECT: exit 0
# the 3 admin_audit_stream_* tests pass with no test-body LEMMY_DATABASE_URL guard
```

### 15.5 Cross-cutting verification

- [ ] `grep -c "get_database_url" crates/api/api/src/governance/admin_audit_stream.rs` returns `0`
- [ ] `grep -c "context.database_url()" crates/api/api/src/governance/admin_audit_stream.rs` returns `1`
- [ ] `grep -c 'EnvVarGuard::set("LEMMY_DATABASE_URL"' crates/server/tests/e2e.rs` returns `14`
      (17 total − 3 band-aids removed = 14; the fixture-internal guard at :6128 plus 13 unrelated test guards remain)
- [ ] `crates/api/api_utils/src/context.rs` contains `pub fn database_url(&self) -> &str`
- [ ] R5: Task 0 enumerated all probes (0–8)
- [ ] R6: all clippy invocations use `--no-deps --features full` uniformly
- [ ] No edits outside the §11 three-file list

---

## 16. Acceptance criteria

- [ ] All 3 tasks completed in dependency order
- [ ] §15.1 (cargo check) exit 0 after Task 1
- [ ] §15.2 (cargo clippy `--no-deps -- -D warnings`) exit 0 after Task 1
- [ ] §15.3 (cargo test --no-run) exit 0 after Task 1
- [ ] §15.4 (gate-4 full e2e) exit 0 — the 3 `admin_audit_stream_*` tests pass without band-aids
- [ ] §15.5 (cross-cutting verification) — all boxes ticked
- [ ] §16a Story 1 `[done]`
- [ ] No edits to files outside §11 list
- [ ] Retro committed per §13 Task 2
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: admin_audit_stream connects using the captured DB URL, no live-env dependency

- **Composing tasks:** Task 1 (single impl task — field/accessor + handler + e2e edits land together)
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full admin_audit_stream"`
- **Expected output:** `3 passed; 0 failed` for the `admin_audit_stream_*` tests (forbidden_for_non_admin, enforces_per_admin_cap, emits_frame_on_config_change)
- **Brief-Scope outputs to verify** (used by `/brehon-verify`):
  - `crates/api/api_utils/src/context.rs` contains `db_url: String` field AND `pub fn database_url(&self) -> &str`
  - `crates/api/api/src/governance/admin_audit_stream.rs` calls `context.database_url()` and no longer contains `get_database_url`
  - `crates/server/tests/e2e.rs` contains exactly ONE `EnvVarGuard::set("LEMMY_DATABASE_URL"…` (the fixture-internal one at :6128); the 3 test-body band-aids are removed

> **Verification mapping:** `/brehon-verify` iterates this section, runs the checkpoint
> against the worktree branch, and confirms each Brief-Scope output exists + matches its
> structural pattern. A phantom (Task 1 reports complete but the accessor is absent or the
> band-aids remain) triggers the catch-fire procedure in advisor-orchestrator.md.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (probes 0–8 confirmed)
- [ ] Task 1 committed (`fix(governance): capture DB URL at LemmyContext::create (issue #167)`)
- [ ] §15 validation green at every gate
- [ ] §16a Story 1 `[done]`
- [ ] Retro committed (Task 2)
- [ ] PR opened by BM session against `governance-v0`
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-quality-r3b-verify.md` shows Story 1 ✓
- [ ] Post-merge phase branch retained for retro reads

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Forgetting the `&`-drop at `admin_audit_stream.rs:126` (`E0277` on `&&str`) | MED | LOW | §10.2 GOTCHA spells out the exact edit; §15.1 catches it |
| Unused `db_url` binding after guard removal → clippy `-D warnings` fail | MED | LOW | §10.3 GOTCHA mandates rename to `_db_url`; §15.2 catches it |
| Editing the wrong/extra `EnvVarGuard` line (removing fixture-internal :6128) | LOW | HIGH | §10.3 + §15.5 assert exactly 14 guards remain (17 − 3 band-aids; :6128 is among them); removing :6128 drops count to 13 and makes all e2e fail loudly |
| Blind edit hangs the worker on the ~17k-line `e2e.rs` | LOW | MED | §10.3 verbatim anchors with disambiguating fn-context per `feedback_fix_impl_pre_locate_e2e_anchors.md` |
| Capture sees stale env if a future caller constructs context before setting `LEMMY_DATABASE_URL` | LOW | MED | Production sets the env before startup (`lib.rs:210`); fixture sets it before `create()` (:6128) — both correct today; noted in §19 |

---

## 19. Notes

- **Single atomic impl task (not split):** the change spans 3 crates but is one logical
  unit — the field/accessor (`lemmy_api_utils`), its sole call site (`lemmy_api`), and the
  test that exercises it (`lemmy_server`) must land together to keep every task compile-green.
  The e2e-edit-isolation norm (template §5.2) is non-Sonnet-only; for this Sonnet target the
  bundle is permitted, and the e2e hang-risk is mitigated in-task via §10.3 verbatim anchors —
  the brief-specified design (brief §2 "single impl-task").
- **Brief §2/§4 prose was stale** (it described an Option-B `db_url()` accessor + callsite
  enumeration). Resolved DQs `a3d0e9941441-039` + `052f0d5c016d-001` make **Option A** with the
  `database_url()` accessor authoritative; this plan follows the DQs, not the stale prose.
- **Why the fixture-internal guard at :6128 stays:** it is what makes `create()`'s internal
  `SETTINGS.get_database_url()` capture the testcontainer URL. Removing it would break the
  capture. Only the three *test-body* band-aids (8256/8281/8353) are redundant after the fix.
- **No DQ pre-seeds filed** — Option A is fully resolved; no open question remained at
  plan-authoring time.

---

## 20. Confidence score

- **Plan correctness:** 9/10 — both fix-shape DQs resolved; all call sites, anchors, and the
  two compile GOTCHAs verified against live code at HEAD `37d79dff5`.
- **Cargo budget:** 8/10 — peak is the gate-4 full e2e (~6 GB); per-task check/clippy are small.
- **Test coverage:** 9/10 — the three existing `admin_audit_stream_*` e2e tests directly prove
  the fix (they pass only because the captured URL survives the fixture guard drop).
