# Plan: v1-quality-r3 — EnvVarGuard C4 follow-on sweep + Issue #167 admin_audit_stream DB-URL-at-create fix

## 1. Summary

This sub-phase closes the C4 follow-on env-leak surface deferred from v1-quality-r2 (§19) and fixes Issue #167. Two deliverables: (1) bring the 23 raw `unsafe { std::env::set_var(...) }` calls for `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS` (12 sites) and `GOVERNANCE_LOG_SIGNING_KEY` (11 sites) in `crates/server/tests/e2e.rs` under the existing `EnvVarGuard` RAII discipline — wrapping test-body sites with scope-held guards and documenting the 2 fixture-`bootstrap()` sites as intentional process-scoped exceptions (option-b); (2) move the `admin_audit_stream` handler's database-URL read from request-time `SETTINGS.get_database_url()` (a lazy `LEMMY_DATABASE_URL` env re-read) to construction-time capture in `LemmyContext::create`, exposed via a `database_url()` accessor (Issue #167, Option A — zero callsite changes). Headline acceptance: `cargo check`/`cargo clippy --no-deps -- -D warnings` clean, the full e2e suite green (gate-4 mandatory for this env-var-management refactor class), and the §15 audit confirming zero unguarded test-body setter sites remain (the 2 bootstrap exceptions documented).

## 2. Source

- `.claude/PRPs/plans/v1-quality-r2.plan.md` §12 + §19 @ `b3af1f4d6` — the **direct predecessor**. §12 deferred this exact C4 follow-on ("~28 additional `env::set_var` sites set `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS` + `GOVERNANCE_LOG_SIGNING_KEY` without `EnvVarGuard`") to v1-quality-r3; §19 pre-seeded the planner-DQ. r2 Tasks 4/5 are the canonical MIRROR for the EnvVarGuard hoist + setter-wrap pattern this phase replicates.
- Canonical sibling plans (per `feedback_read_canonical_before_writing_spec.md`): `.claude/PRPs/plans/v1-quality-r2.plan.md` (EnvVarGuard sweep shape, §15 DoD, §16a stories, T5 multiline-robust Python audit) + `.claude/PRPs/plans/v1-RT-r3.plan.md` (introduced `EnvVarGuard`).
- `.claude/PRPs/briefs/v1-quality-r3-planning-1.md` — the authoring brief (scope, claimed line numbers, #167 fix shape, #158 exclusion).
- Resolved clarify DQs (`from: advisor`, `answered_by: advisor`): `a3d0e9941441-039` (#167 = **Option A** internal SETTINGS read, zero callsite changes), `a3d0e9941441-040` (EnvVarGuard **already at test-crate root** e2e.rs:111/116/127 — do NOT re-hoist), `a3d0e9941441-041` (add `feedback_validate_pending_laptop_must_use_wrapper.md` to required reading).
- Lessons that bind decisions:
  - `feedback_envvarguard_fixture_lifetime_footgun.md` — the guard-drop-at-`bootstrap()`-return footgun; drives the option-a (test-body wrap, held to fn end) vs option-b (bootstrap site stays raw + SAFETY) split. The original v1-quality-r2 incident was `admin_audit_stream`'s lazy `get_database_url()` re-read — i.e. Issue #167 itself.
  - `feedback_gate4_full_e2e_env_refactor_class.md` — gate-4 full e2e mandatory for env-var-management refactors (compile-only gates cannot catch RAII-lifetime / read-timing regressions).
  - `feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-located verbatim `old_string`/`new_string` anchors mandatory for every e2e.rs Edit (the e2e-edit-hang mitigation).
  - `feedback_planner_enumerate_struct_callsites_for_addfield.md` — drove the §11 `LemmyContext { ... }` struct-literal enumeration for the #167 field-add.
  - `feedback_validate_pending_laptop_must_use_wrapper.md` + `feedback_windows_e2e_requires_bat_wrapper.md` — every cargo gate invokes the `scripts/brehon/cargo-*.bat` wrapper.
- ADRs: none superseded (this is a test-hygiene + read-timing refactor; no ADR-affecting decision).

## 3. Problem statement

1. **C4 follow-on env-leak surface (deferred from r2 §12).** 23 raw `unsafe { std::env::set_var(...) }` calls in `crates/server/tests/e2e.rs` set two env vars without `EnvVarGuard`:
   - `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS` — **12 sites**: lines 833, 2568, 3347, 4114, 4459, 4788, 4923, 5047, 5675, 5876, 6146, 16823.
   - `GOVERNANCE_LOG_SIGNING_KEY` — **11 sites**: single-line 834, 2569, 3348, 5048, 6147, 16824; **multi-line** (env-var name on its own line, literal hex value) 4116, 4461, 4790, 5677, 5878.

   Same defect class as #159/#160 (closed in r2). The `boot_context()` site (e2e.rs:17474/17475) is **already** EnvVarGuard-wrapped (r2 Task 4) — do NOT re-wrap. Tied to **Task 1** (bootstrap exceptions) + **Task 2** (test-body wraps).

2. **`admin_audit_stream` request-time DB-URL re-read (Issue #167).** `crates/api/api/src/governance/admin_audit_stream.rs:125` calls `context.settings().get_database_url()`, which (`crates/utils/src/settings/mod.rs:49-55`) lazily re-reads `LEMMY_DATABASE_URL` from the process env **at request time**. This re-read is exactly the v1-quality-r2 footgun incident: if an `EnvVarGuard` for `LEMMY_DATABASE_URL` has dropped before the request runs, the handler reads the wrong URL. The fix captures the URL **once at `LemmyContext::create` time** (the same SETTINGS read the pool build already makes) and exposes it via `database_url()`. Tied to **Task 3**.

## 4. Solution statement

**EnvVarGuard sweep (Tasks 1–2).** The `EnvVarGuard` RAII struct already lives at the e2e.rs test-crate root (lines 111–141), visible to all sibling fixtures modules. Two distinct edit shapes, kept in **separate tasks** to prevent confusing the two (mis-applying one shape to the other re-introduces the exact footgun):

- **Option-b (Task 1, 2 sites):** the two fixture `bootstrap()` functions — `governance_fixtures::bootstrap` (e2e.rs:827, sites 833/834) and `admin_config_fixtures::bootstrap` (e2e.rs:6138, sites 6146/6147) — have many callers and return `(ContainerAsync, Data<LemmyContext>, String)` (no guard-threading). Wrapping their INIT/GOV sites with a guard that drops at `bootstrap()` return would unset the env vars before the test body runs (the `feedback_envvarguard_fixture_lifetime_footgun.md` footgun). Because both env vars are **constant-valued** (`"1"` and a fixed signing seed) and the test harness runs single-threaded (`--test-threads=1`), leaving them process-scoped is benign. These sites **stay raw `set_var`** but gain a `// SAFETY:` justification comment documenting the intentional process-scoping.
- **Option-a (Task 2, ~10 test-fn sites):** every other site sits in a `#[tokio::test]` body where a guard naturally outlives the operations that read the env var (the guard binds in the test fn and drops at fn exit). Replace each `unsafe { set_var(INIT); set_var(GOV); }` block with `let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1"); let _g_gov = EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", <seed>);`.

**Issue #167 fix (Task 3).** Add a `db_url: String` field to `LemmyContext`; capture `SETTINGS.get_database_url()` inside `LemmyContext::create` (the single struct-literal constructor at context.rs:31); expose `pub fn database_url(&self) -> &str`. Change `admin_audit_stream.rs:125` from `context.settings().get_database_url()` to `context.database_url()`. The read moves from request-time to construction-time — in the e2e fixtures, `LemmyContext::create` is always called while the `LEMMY_DATABASE_URL` `EnvVarGuard` (`_g_db_url`) is alive, so the captured URL is correct.

The reader should predict §11 from this: e2e.rs (Tasks 1, 2), context.rs + admin_audit_stream.rs (Task 3).

## 5. Metadata

- **Phase:** `v1-quality-r3`
- **Branch:** `phase-v1-quality-r3` — bm-cut from `governance-v0` HEAD at task-execution time, post-clarify (clarify DQs already resolved).
- **Target impl-task model:** `sonnet-4-6` (default)
- **Estimated tasks:** 5 (Task 0 pre-flight + 3 impl tasks T1/T2/T3 + 1 retro)
- **Estimated cargo budget:** N/A (validate-pending-laptop; cargo runs on laptop, ~6 GB peak for `cargo test --workspace --features full`; Shape G SUSPENDED through 2026-06-01 per `project_shape_g_suspended_2026_05_16.md`).
- **Forbidden-window applicability:** non-binding for impl-task dispatch under validate-pending-laptop (cargo runs on laptop, not EliteDesk).
- **Complexity score:** **9/10** — over the Sonnet `>8` threshold → split-DQ filed (see §5.1 + §19). Recommendation in the DQ: **proceed-as-one** (the +6 is the e2e-edit factor over-weighting bounded, well-anchored mechanical edits per `feedback_plan_complexity_e2e_edit_factor_overweights_doc_only_tasks.md`; the brief explicitly scoped a single plan; each task is independently serial-gated). Advisor decides at gate-1.

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md` + `plan.template.md` §5.1. Counts are mechanical from the §13 task YAML:

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | **0** | 3 impl tasks (T1/T2/T3); excludes Task 0 + retro; 3 − 5 < 0 → 0. |
| Migrations touched | +2 each | **0** | Zero `crates/db_schema/migrations/**` files (confirmed: glob returns none). |
| Crates touched | +1 each | **+3** | `lemmy_server` (e2e.rs), `lemmy_api_utils` (context.rs), `lemmy_api` (admin_audit_stream.rs). |
| `crates/server/tests/e2e.rs` edits | +3 each | **+6** | T1 + T2 each modify e2e.rs. 2 × +3 = +6. |
| New ADR-affecting decisions | +2 each | **0** | Supersedes no `99-...md` entry. |
| Cargo budget peak above 6 GB | +1 per GB | **0** | Validate-pending-laptop, ~6 GB peak (not above). |
| **Total** | — | **9** | `> 8` Sonnet threshold → split-DQ fired (see §19). |

**Split-DQ filed (pending, `from: planner`, `kind: blocker`).** The natural split boundary is e2e-sweep (T1+T2, the +6 e2e factor) vs #167 (T3, production code in 2 separate crates). Recommendation: **proceed-as-one** — the score is dominated by the flat-+3-per-e2e-task factor, which `feedback_plan_complexity_e2e_edit_factor_overweights_doc_only_tasks.md` (promoted in r2 §19) flags as over-weighting mechanical anchored edits; each task is single-file / serial-gated and well within Sonnet's demonstrated envelope (r2 T5 ran 13 anchored e2e edits cleanly). Advisor/user decides at gate-1; do not self-resolve.

### 5.2 Per-task complexity ceiling

Target model is Sonnet, so the Sonnet ceiling applies (per template §5.2):

- `count(union(creates, modifies)) ≤ 4` files per task
- `count(distinct crates/<X>/ prefixes in union(creates, modifies)) ≤ 2` crates per task
- `crates/server/tests/e2e.rs` bundling allowed (Sonnet only)

Verified at §13 task YAML walk-time: T1 = 1 file (e2e.rs), 1 crate (lemmy_server). T2 = 1 file (e2e.rs), 1 crate. T3 = 2 files (context.rs + admin_audit_stream.rs), 2 crates (lemmy_api_utils + lemmy_api). T4 = 1 file (retro report), 0 crates. All within ceiling.

## 6. Relationship to other v1-quality-* sub-phases

- **Predecessor (merged):** `v1-quality-r2` (PR #163, merged 2026-05-29; `b3af1f4d6` promotes its lessons). This phase executes the C4 follow-on r2 §12 explicitly deferred to r3.
- **Predecessor (merged):** `v1-quality-r1` (PR #145). No overlap.
- **Successor (possible):** `v1-quality-r4` — only if a 3rd `emit_reputation_event` consumer materialises (Issue #158, see §12) or a new env-leak class is filed.
- **Concurrent lanes:** confirm at bm-cut time via `git worktree list`; this lane's only file overlap risk is e2e.rs — Task 0 Probe N checks open PRs touching e2e.rs.

## 7. Preflight guardrails inherited from prior phases

- **R1** (per `feedback_clippy_test_style.md`): every i32 ↔ i64 comparison uses `i64::from(...)`, never `as` cast. No new arithmetic in this plan; applies only if a guard binding introduces one (it does not).
- **R5** (per JM-b retro): Task 0 enumerates ALL probes explicitly (Docker preflight + 4 wrapper probes per `.claude/rules/pre-phase-harness-audit.md` + clippy baseline capture + brief re-enumeration counts).
- **R6** (per JM-b retro): all `cargo clippy` invocations use `--no-deps --features full -- -D warnings`. Verified in §15.
- **R7** (per JM-b retro): per task that touches a struct or signature, `cargo test --no-run -p lemmy_server --test e2e --features full` runs as a per-task gate. T3's `LemmyContext` field-add qualifies → R7 applies to T3.
- **R8** (per `feedback_features_full_p_crate_incompatible.md`): never combine `-p <crate>` with `--features full` except where the crate defines a `full` feature.
- **R9** (per `feedback_validate_pending_laptop_must_use_wrapper.md` + `feedback_windows_e2e_requires_bat_wrapper.md`): every cargo gate invokes the `scripts/brehon/cargo-*.bat` (Windows) / `.sh` (Linux) wrapper.
- **R10** (per `cargo-output-capture.md` + `no-cargo-output-paste.md`): all cargo invocations redirect to `.claude/PRPs/debug/<phase>-<task>-<verb>.log` with `> log 2>&1`; exit-code preservation via `$?`; never piped through `tail`/`head`/`grep`.
- **R11** (per `feedback_fix_impl_pre_locate_e2e_anchors.md`): every impl-task brief whose `modifies:` array includes `crates/server/tests/e2e.rs` MUST paste verbatim `old_string`/`new_string` anchors for every Edit. T1/T2 briefs honour this.
- **R12** (per `feedback_gate4_full_e2e_env_refactor_class.md`): full e2e execution is MANDATORY for this phase (env-var-management refactor class) — a compile-only gate is insufficient. Runs once post-T3 at phase-tip.

## 8. Flow design

```
Task 0 (harness audit + re-enumeration probes — no commit)
  │
  ▼
Task 1 (option-b: SAFETY-comment the 2 bootstrap footgun sites — keep raw set_var)
  │  modifies crates/server/tests/e2e.rs (governance_fixtures::bootstrap @827, admin_config_fixtures::bootstrap @6138)
  ▼
Task 2 (option-a: wrap ~10 test-body INIT/GOV sites with EnvVarGuard, held to fn end)
  │  modifies crates/server/tests/e2e.rs (10 test fns; multi-line GOV sites 4116/4461/4790/5677/5878)
  ▼
Task 3 [P] (Issue #167: db_url field + create()-time capture + database_url() accessor; admin_audit_stream callsite)
  │  modifies crates/api/api_utils/src/context.rs + crates/api/api/src/governance/admin_audit_stream.rs
  │  ([P] = file-disjoint from T1/T2; in practice no parallel peer — T1/T2 are a serial e2e chain — so dispatched alone)
  ▼
[phase-tip] gate-4 full e2e (R12 mandatory) — validate-pending-laptop-e2e
  │
  ▼
Task 4 (Retro)
  │  creates .claude/PRPs/reports/v1-quality-r3-retro.md
  ▼
bm-pr / bm-poll-cr / bm-triage / bm-merge (BM session)
```

**RAII lifetime note (T2):** `EnvVarGuard` stores `(key, prev_value)` and restores on `Drop`. Every `let _g_init = EnvVarGuard::set(...);` binding MUST live in the same scope as (and outlive) the `LemmyContext::create` / first-SETTINGS-access call that reads the env var. The `_g_*` binding (NOT `_`) is load-bearing — `let _ = EnvVarGuard::set(...)` drops the guard immediately. In a test body the guard drops at fn exit, which is correct. The 2 bootstrap sites (T1) are NOT wrapped precisely because a guard there would drop at `bootstrap()` return, before the test body runs.

**#167 read-timing note (T3):** moving the DB-URL read to `create()` time is safe in the e2e fixtures because `LemmyContext::create` is always called *after* `let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);` and *while* that guard is alive (e.g. bootstrap @827: `_g_db_url` at line 839, `create` at line 854). Production `server/src/lib.rs:210` reads from the real env/config, unaffected.

## 9. Mandatory reading

For the impl-task subagent before its first Edit:

**Schema/type definitions:**
- `crates/server/tests/e2e.rs:111-141` — `EnvVarGuard` struct + `impl EnvVarGuard` + `impl Drop`. Already at test-crate root (clarify DQ 040). Do NOT re-hoist. Canonical RAII reference.
- `crates/api/api_utils/src/context.rs:13-53` — `LemmyContext` struct (fields), `create()` (the single struct-literal constructor at :31), and the `settings()`/accessor pattern T3 mirrors.
- `crates/utils/src/settings/mod.rs:49-55` — `get_database_url()`; the lazy `LEMMY_DATABASE_URL` env re-read #167 moves to create-time.

**Existing patterns (MIRROR refs):**
- `crates/server/tests/e2e.rs:17474-17475` — `guards.push(EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1"));` + GOV; the boot_context wrap (already done; do NOT touch). Canonical INIT/GOV guard shape.
- `crates/server/tests/e2e.rs:16821-16828` — existing test-body `// SAFETY:` comment + `unsafe { set_var(INIT); set_var(GOV); }` + `let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);` — the exact before-shape T2 replaces, with the canonical SAFETY text T1 reuses.
- `.claude/PRPs/plans/v1-quality-r2.plan.md` §10.1/§10.2 + Task 5 VALIDATE block — the setter-wrap pattern + multiline-robust Python audit.

**Adjacent fixtures:**
- `crates/server/tests/e2e.rs:143` (`mod governance_fixtures`) + `:6110` (`mod admin_config_fixtures`) — the 2 bootstrap-owning modules (T1).

**Lessons:** `feedback_envvarguard_fixture_lifetime_footgun.md` (gates T1/T2 split), `feedback_gate4_full_e2e_env_refactor_class.md` (gates the mandatory e2e gate), `feedback_fix_impl_pre_locate_e2e_anchors.md` (gates every e2e Edit), `feedback_planner_enumerate_struct_callsites_for_addfield.md` (gates T3 field-add), `feedback_validate_pending_laptop_must_use_wrapper.md` (gates §15 wrapper usage).

## 10. Patterns to mirror

Per `feedback_advisor_watchpoint_specificity.md`: each entry cites a specific file:line.

### 10.1 EnvVarGuard RAII (canonical: e2e.rs:111-141)

**Mirror:** `crates/server/tests/e2e.rs:111-141`

```rust
struct EnvVarGuard {
  key: &'static str,
  prev: Option<String>,
}
impl EnvVarGuard {
  fn set(key: &'static str, value: &str) -> Self {
    let prev = std::env::var(key).ok();
    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
    unsafe { std::env::set_var(key, value); }
    EnvVarGuard { key, prev }
  }
}
impl Drop for EnvVarGuard {
  fn drop(&mut self) {
    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
    unsafe {
      match &self.prev {
        Some(prev) => std::env::set_var(self.key, prev),
        None => std::env::remove_var(self.key),
      }
    }
  }
}
```

Already at test-crate root (above the first `mod governance_fixtures {` at line 143). Sibling modules reach it via `use super::EnvVarGuard;` or an existing `use super::*;`. **Do NOT re-hoist** (clarify DQ 040). The two `unsafe { std::env::set_var(...) }` calls inside this impl are the RAII machinery — the §15 audit MUST exclude lines 111–141.

### 10.2 EnvVarGuard test-body wrap (option-a — canonical: e2e.rs:17474-17475)

**Mirror:** `crates/server/tests/e2e.rs:17474-17475` (boot_context, already wrapped) + the before-shape at `:16821-16824`.

**Before (the raw block T2 replaces):**

```rust
    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
    unsafe {
      std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
      std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    }
```

**After:**

```rust
    let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    let _g_gov = EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
```

The `_g_init` / `_g_gov` bindings (NOT `_`) are load-bearing — they hold the guard to the end of the test fn so the env vars stay set while `LemmyContext::create` and the governance-log signer read them. For sites where `GOVERNANCE_LOG_SIGNING_KEY` is set with a **literal hex string** spanning multiple lines (4116, 4461, 4790, 5677, 5878), the `value` argument is the same literal — copy it verbatim into the `EnvVarGuard::set(...)` call. Site 4923 has **INIT only** (no paired GOV) — emit only the `_g_init` line there.

### 10.3 Footgun-zone SAFETY comment (option-b — canonical: e2e.rs:827 + 6138)

**Mirror:** the existing test-body SAFETY text at `crates/server/tests/e2e.rs:16821`.

The two `bootstrap()` sites keep their raw `unsafe { set_var(INIT); set_var(GOV); }` block but gain a justification comment immediately above the `unsafe {`:

```rust
    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
    // These two env vars are intentionally process-scoped (NOT EnvVarGuard-wrapped):
    // both are constant-valued ("1" / fixed signing seed) and bootstrap() has many
    // callers across this test module — wrapping here would drop the guard at
    // bootstrap() return, unsetting the var before the test body runs (see
    // feedback_envvarguard_fixture_lifetime_footgun.md). LEMMY_DATABASE_URL IS
    // guarded (per-call value) at the _g_db_url binding below.
    unsafe {
      std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
      std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    }
```

(`admin_config_fixtures::bootstrap` @6138 defines a local `const SIGNING_SEED_HEX` immediately above its `unsafe {`; keep that const.)

### 10.4 LemmyContext db_url field + accessor (#167 — canonical: context.rs:13-53)

**Mirror:** `crates/api/api_utils/src/context.rs:13-53`

```rust
// struct (add field):
pub struct LemmyContext {
  pool: ActualDbPool,
  client: Arc<ClientWithMiddleware>,
  pictrs_client: Arc<ClientWithMiddleware>,
  secret: Arc<Secret>,
  rate_limit_cell: RateLimit,
  db_url: String,                         // NEW (#167): captured once at create() time
}

// create() (single struct-literal constructor at :31 — add capture + field):
  pub fn create(
    pool: ActualDbPool,
    client: ClientWithMiddleware,
    pictrs_client: ClientWithMiddleware,
    secret: Secret,
    rate_limit_cell: RateLimit,
  ) -> LemmyContext {
    LemmyContext {
      pool,
      client: Arc::new(client),
      pictrs_client: Arc::new(pictrs_client),
      secret: Arc::new(secret),
      rate_limit_cell,
      db_url: SETTINGS.get_database_url(),   // NEW: same SETTINGS read the pool build makes
    }
  }

// accessor (add next to settings()):
  pub fn database_url(&self) -> &str {
    &self.db_url
  }
```

`SETTINGS` is already imported (context.rs:7). `create()` signature is **unchanged** (no new parameter) → zero callsite changes (Option A, clarify DQ 039). The single `LemmyContext { ... }` struct-literal is at context.rs:31 — no other construction sites exist in `crates/` (verified §11).

### 10.5 admin_audit_stream callsite (#167 — canonical: admin_audit_stream.rs:125)

**Mirror:** `crates/api/api/src/governance/admin_audit_stream.rs:125`

**Before:** `let db_url = context.settings().get_database_url();`
**After:** `let db_url = context.database_url();`

The downstream `tokio_postgres::connect(&db_url, NoTls)` takes `&str`; `context.database_url()` returns `&str`, so `&db_url` becomes `db_url` (or keep `let db_url = context.database_url();` and pass `db_url`). Confirm the binding type at impl time — the connect call must receive `&str`.

## 11. Files to change

Grouped by crate. Each path verified present in the workspace.

- `crates/server/tests/e2e.rs` (crate `lemmy_server`) — option-b SAFETY comments on 2 bootstrap sites (Task 1) + option-a EnvVarGuard wraps of ~10 test-body INIT/GOV sites (Task 2). NO new tests; all-refactor. Single file, two non-overlapping edit shapes split across T1/T2 for correctness separation.
- `crates/api/api_utils/src/context.rs` (crate `lemmy_api_utils`) — add `db_url: String` field + `SETTINGS.get_database_url()` capture in `create()` + `database_url()` accessor (Task 3).
- `crates/api/api/src/governance/admin_audit_stream.rs` (crate `lemmy_api`) — change line 125 callsite to `context.database_url()` (Task 3).
- `.claude/PRPs/reports/v1-quality-r3-retro.md` — retro (Task 4).

### Struct-field add: all callsites enumerated (mandatory)

Per `feedback_planner_enumerate_struct_callsites_for_addfield.md`. Task 3 adds `db_url: String` to the public `LemmyContext` struct. `grep -rn 'LemmyContext {' crates/` at plan-authoring time returns exactly **one** struct-literal construction site:

- `crates/api/api_utils/src/context.rs:31` — the literal inside `create()`. (The other grep hits are `pub struct LemmyContext {` :13, `impl LemmyContext {` :23, and the `-> LemmyContext {` return-type at :30 — none are construction sites.)

There is **no** `LemmyContext { ... }` literal anywhere else in `crates/` — `create()` is the sole constructor. The 16 `LemmyContext::create(...)` *call* sites (14 in e2e.rs, `server/src/lib.rs:210`, `context.rs:79`) are unaffected because the signature is unchanged (Option A). Therefore the field-add requires editing only `context.rs` — no caller crate compiles-after dependency.

## 12. NOT building in v1-quality-r3

- **Issue #158 — `emit_reputation_event` helper extraction.** EXCLUDED as premature DRY. `grep -rn 'emit_reputation_event' crates/` evidence: a file-private `async fn emit_reputation_event` in `crates/api/api/src/governance/submit_jury_vote.rs:1096` (called 4× **within the same file**: 588/620/664/716) and a **separate** file-private `async fn emit_reputation_event_local` in `crates/api/api/src/governance/admin_emergency_remove.rs:448` (called once at :418). That is **2 implementations across 2 files (with one internal caller cluster), not 3+ independent consumers** of a shared helper — extracting a shared crate-level helper now would be speculative abstraction. Defer until a genuine 3rd consumer materialises. (r2 §12 already filed the `kind: "log"` deferral DQ for #158; this plan does not re-file it.)
- **Re-hoisting / re-wrapping `boot_context()` (e2e.rs:17474/17475).** Already EnvVarGuard-wrapped in r2 Task 4. Out of scope; do NOT touch.
- **`LEMMY_DATABASE_URL` setter sweep.** Already closed in r2 (Issue #160). The 2 bootstrap sites' `_g_db_url` guards are pre-existing and stay.
- **New `create()` parameter for #167.** Rejected (Option B, clarify DQ 039) — adds an unused-parameter burden to 16 callsites for zero benefit.

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task.** Task 0 is non-`[P]` (barrier).

> **Shape G note:** Shape G is SUSPENDED through 2026-06-01. This plan runs under validate-pending-laptop: cargo gates execute on the laptop advisor session via the `.bat` wrappers, not on GH Actions. §13 task bodies carry inline cargo VALIDATE blocks.

### Task 0: Pre-flight harness audit + branch verification + re-enumeration

**Goal:** verify environment is ready for `v1-quality-r3`; confirm branch is `phase-v1-quality-r3`; confirm the EnvVarGuard struct + all 23 setter sites are present on the branch tip exactly as §3 enumerates; confirm clippy baseline clean.

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate ALL probes explicitly):**

```bash
# Probe 0 — Docker daemon (needed for the phase-tip e2e gate)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — wrapper sanity: cargo-check honors -p (Windows: cmd //c "scripts\\brehon\\cargo-check.bat ...")
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/PRPs/debug/v1-quality-r3-audit-check-p.log 2>&1"
tail -20 .claude/PRPs/debug/v1-quality-r3-audit-check-p.log   # EXPECT: only lemmy_utils compiles

# Probe 2 — feature flag activation
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/PRPs/debug/v1-quality-r3-audit-check-features.log 2>&1"
tail -20 .claude/PRPs/debug/v1-quality-r3-audit-check-features.log

# Probe 3 — cargo-test wrapper honors target selection
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features full > .claude/PRPs/debug/v1-quality-r3-audit-test.log 2>&1"
tail -20 .claude/PRPs/debug/v1-quality-r3-audit-test.log

# Probe 4 — wrappers fail loud on cargo errors (exit-code propagation; negative test)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/v1-quality-r3-audit-negative.log 2>&1"
echo "cargo-check.bat exit on bogus feature: $?"   # EXPECT: non-zero (typically 101)

# Probe 5 — EnvVarGuard struct present at test-crate root (do NOT re-hoist)
grep -n "^struct EnvVarGuard {" crates/server/tests/e2e.rs   # EXPECT: line 111
grep -n "^mod governance_fixtures {" crates/server/tests/e2e.rs   # EXPECT: line 143 (struct precedes it)

# Probe 6 — re-enumerate setter counts (must match §3)
echo "INIT raw set_var sites:"; grep -c 'std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS"' crates/server/tests/e2e.rs   # EXPECT: 13 (12 raw + 0... see note)
echo "GOV via set_var (any form):"; python3 -c "import re; print(len(re.findall(r'std::env::set_var\(\s*\"GOVERNANCE_LOG_SIGNING_KEY\"', open('crates/server/tests/e2e.rs').read())))"   # EXPECT: 11

# Probe 7 — #167 targets present
grep -n "get_database_url" crates/api/api/src/governance/admin_audit_stream.rs   # EXPECT: line ~125
grep -n "LemmyContext {" crates/api/api_utils/src/context.rs   # EXPECT: single literal at :31

# Probe 8 — clippy baseline on the e2e target (capture; must be clean before T1)
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r3-audit-clippy-baseline.log 2>&1"
echo "clippy baseline exit: $?"   # EXPECT: 0

# Probe N — concurrent-PR check (no other open PR touches e2e.rs)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | contains("crates/server/tests/e2e.rs")) | {number, title, headRefName}'
# EXPECT: empty; if non-empty, STOP and reconcile (e2e.rs ownership conflict)
```

> **Probe 6 INIT-count note:** a bare `grep -c` of the INIT string also matches the boot_context `EnvVarGuard::set("LEMMY_INITIALIZE...")` line and the audit/comment lines. The brief's claimed raw-site count is **12** (`std::env::set_var(...)` form only); the impl agent should confirm 12 raw `std::env::set_var("LEMMY_INITIALIZE...` lines + 1 `EnvVarGuard::set("LEMMY_INITIALIZE...` (boot_context). If counts drift, STOP and re-survey before T1.

**EXPECT block:**
- Probes 0–3, 5–8, N exit 0 / match expected
- Probe 4 exits NON-ZERO (negative test confirms exit-code propagation)

**No commit at Task 0** — verification only.

### Task 1: Option-b — SAFETY-comment the 2 bootstrap footgun sites

**ACTION:** add a `// SAFETY:` justification comment above the `unsafe { set_var(INIT); set_var(GOV); }` block in the two fixture `bootstrap()` functions; keep the raw `set_var` calls (intentional process-scoping per §10.3).

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # 2 SAFETY-comment additions in governance_fixtures::bootstrap (@827) + admin_config_fixtures::bootstrap (@6138)
requires: []
```

**IMPLEMENT (file 1 of 1):** in `crates/server/tests/e2e.rs`, two Edits:
1. At `governance_fixtures::bootstrap` (fn @827; sites 833/834): insert the §10.3 SAFETY-justification comment block immediately above the existing `unsafe {` (currently at ~line 832, which has only the bare comment or none). Keep the two `set_var` lines verbatim.
2. At `admin_config_fixtures::bootstrap` (fn @6138; sites 6146/6147): same insertion above its `unsafe {`. Keep the local `const SIGNING_SEED_HEX` and the two `set_var` lines verbatim.

**MIRROR:** §10.3 (the SAFETY-comment shape) + the canonical SAFETY text at e2e.rs:16821.

**GOTCHA:** do NOT wrap these two sites with `EnvVarGuard` — a guard here drops at `bootstrap()` return and unsets the var before the test body runs (the footgun). The comment IS the deliverable; the raw `set_var` stays. R11: paste verbatim `old_string` (the `unsafe {` block + 3 surrounding lines for uniqueness — the two bootstrap fns are near-identical, so include the enclosing fn-distinct line, e.g. `governance_fixtures` uses `start_postgres()` directly vs `admin_config_fixtures` uses `super::governance_fixtures::start_postgres()`).

**VALIDATE:**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-quality-r3-task1-check.log 2>&1"
echo "check exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r3-task1-check.log
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r3-task1-clippy.log 2>&1"
echo "clippy exit: $?"
# EXPECT: both exit 0
```

**Commit subject:** `refactor(e2e): document 2 bootstrap env-var process-scoping exceptions with SAFETY justification (task 1)`.

### Task 2: Option-a — wrap ~10 test-body INIT/GOV setter sites with EnvVarGuard

**ACTION:** in each `#[tokio::test]` body that raw-sets `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS` (+ paired `GOVERNANCE_LOG_SIGNING_KEY`), replace the `unsafe { set_var(...); set_var(...); }` block with `let _g_init = EnvVarGuard::set(...)` (+ `let _g_gov = EnvVarGuard::set(...)`), bound to live to fn end.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # ~10 test-fn EnvVarGuard wraps (sites: INIT 2568,3347,4114,4459,4788,4923,5047,5675,5876,16823; GOV 2569,3348,4116,4461,4790,5048,5677,5878,16824)
requires:
  - task: 1
    reason: "T1 lands the 2 bootstrap SAFETY exceptions on the same file; T2 wraps the remaining sites. Serial (same-file e2e.rs) — T2 must edit on top of T1's tree to avoid anchor drift / merge churn."
```

**IMPLEMENT (file 1 of 1):** ~10 Edits in `crates/server/tests/e2e.rs`, one per test fn:
- **Paired sites (INIT+GOV in the same `unsafe` block):** 2568/2569, 3347/3348, 4114/4116, 4459/4461, 4788/4790, 5047/5048, 5675/5677, 5876/5878, 16823/16824 → replace the whole `unsafe { ... }` block (and its `// SAFETY:` comment line) with two `let _g_init = ...; let _g_gov = ...;` lines per §10.2.
- **INIT-only site 4923** → replace with a single `let _g_init = ...;` line (no GOV pair).
- **Multi-line GOV value sites** (4116, 4461, 4790, 5677, 5878): the `GOVERNANCE_LOG_SIGNING_KEY` value is a literal hex string on its own line — copy it verbatim into `EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", <verbatim-literal>)`.
- **`use` visibility:** for any test fn inside a nested `mod` that lacks `use super::EnvVarGuard;` (and lacks `use super::*;`), add `use super::EnvVarGuard;` near the module's `use` block. Confirm per-module at impl time (Task 0 Probe 5 + a per-site read). Top-level test fns reference `EnvVarGuard` directly.

**MIRROR:** §10.2 (test-body wrap shape) + e2e.rs:17474-17475 (boot_context INIT/GOV guards) + T1's commit on the phase tip.

**GOTCHA:** bind with named `_g_init`/`_g_gov` (NOT `_`). `let _ = EnvVarGuard::set(...)` drops immediately and re-introduces the leak. R11: paste verbatim `old_string`/`new_string` for all ~10 Edits in the brief; the `unsafe` blocks are near-identical across fns, so include enough surrounding context (the fn name line or a distinctive adjacent statement) to make each `old_string` unique.

**VALIDATE (story-checkpoint feeds §16a Story 1):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-quality-r3-task2-check.log 2>&1"
echo "check exit: $?"
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r3-task2-clippy.log 2>&1"
echo "clippy exit: $?"

# Brief-Scope audit — every INIT/GOV raw setter is gone EXCEPT the 2 sanctioned bootstrap sites,
# each of which must have a SAFETY comment. Handles multi-line GOV via \s* after set_var(.
python3 -c "
import re
src = open('crates/server/tests/e2e.rs').read()
def line_of(pos): return src[:pos].count('\n') + 1
def in_envvarguard_impl(pos):
    above = src[max(0,pos-600):pos]
    return 'impl EnvVarGuard' in above[-600:] or 'impl Drop for EnvVarGuard' in above[-600:]
def has_safety(pos):
    above = src[max(0,pos-400):pos]
    return '// SAFETY:' in above
raw = []
for var in ('LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS','GOVERNANCE_LOG_SIGNING_KEY'):
    for m in re.finditer(r'std::env::set_var\(\s*\"'+re.escape(var)+r'\"', src):
        pos = m.start()
        if in_envvarguard_impl(pos):   # RAII machinery at lines 111-141 — skip
            continue
        raw.append((line_of(pos), var, has_safety(pos)))
unsafe_without_safety = [(ln,v) for (ln,v,s) in raw if not s]
if unsafe_without_safety:
    print('FAIL — raw setter site(s) without SAFETY justification (must be wrapped or commented):', unsafe_without_safety)
    raise SystemExit(1)
# After T1+T2: exactly the 2 bootstrap sites remain raw (each INIT+GOV) = 2 INIT + 2 GOV = 4, all SAFETY-commented.
n_init = len([1 for (_,v,_) in raw if v=='LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS'])
n_gov  = len([1 for (_,v,_) in raw if v=='GOVERNANCE_LOG_SIGNING_KEY'])
if (n_init, n_gov) != (2, 2):
    print(f'FAIL — expected exactly 2 INIT + 2 GOV sanctioned bootstrap raw sites, got {n_init} INIT + {n_gov} GOV:', raw)
    raise SystemExit(1)
print(f'OK — {n_init} INIT + {n_gov} GOV raw sites remain, all SAFETY-commented (the 2 documented bootstrap exceptions); all test-body sites wrapped.')
" > .claude/PRPs/debug/v1-quality-r3-task2-audit.log 2>&1
echo "audit exit: $?"
tail -5 .claude/PRPs/debug/v1-quality-r3-task2-audit.log
# EXPECT: all three exit 0; audit prints OK with 2 INIT + 2 GOV.
```

**Commit subject:** `refactor(e2e): wrap test-body LEMMY_INITIALIZE + GOVERNANCE_LOG setters with EnvVarGuard (task 2)`.

### Task 3 [P]: Issue #167 — capture DB URL at LemmyContext::create, expose via accessor

**ACTION:** add a `db_url: String` field to `LemmyContext`, populate it from `SETTINGS.get_database_url()` inside `create()`, expose `database_url(&self) -> &str`, and change `admin_audit_stream.rs:125` to read from `context.database_url()`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api_utils/src/context.rs                        # add db_url field + create()-time capture + database_url() accessor
  - crates/api/api/src/governance/admin_audit_stream.rs        # line 125: context.settings().get_database_url() -> context.database_url()
requires: []
```

> **`[P]` note:** Task 3's `union(creates, modifies)` (context.rs + admin_audit_stream.rs) shares **zero** paths with T1/T2 (e2e.rs), so it is file-disjoint and marked `[P]`. In practice there is no parallel cohort peer — T1→T2 form a serial e2e chain and the retro depends on all — so the advisor dispatches Task 3 in its own slot. The `[P]` documents disjointness, not an executable cohort.

**IMPLEMENT (file 1 of 2):** in `crates/api/api_utils/src/context.rs` — (a) add `db_url: String` to the `LemmyContext` struct (after `rate_limit_cell`); (b) add `db_url: SETTINGS.get_database_url(),` to the `LemmyContext { ... }` literal in `create()` (:31); (c) add `pub fn database_url(&self) -> &str { &self.db_url }` adjacent to `settings()`. Per §10.4. `SETTINGS` already imported (:7); signature unchanged.

**IMPLEMENT (file 2 of 2):** in `crates/api/api/src/governance/admin_audit_stream.rs:125`, change `let db_url = context.settings().get_database_url();` → `let db_url = context.database_url();`. Per §10.5. Confirm the downstream `tokio_postgres::connect(...)` receives `&str` (adjust `&db_url` vs `db_url` so the type matches — `database_url()` already returns `&str`).

**MIRROR:** §10.4 (context.rs field+accessor) + §10.5 (admin_audit_stream callsite).

**GOTCHA:** R7 applies — this changes the `LemmyContext` struct shape, so `cargo test --no-run -p lemmy_server --test e2e --features full` must pass (all 16 `create()` callsites still compile because the signature is unchanged — Option A). The read now happens at `create()` time; in the e2e fixtures this is always while `_g_db_url` is alive (see §8 read-timing note), so no behavioural regression — but this is exactly why the gate-4 full e2e (R12) is mandatory.

**VALIDATE (story-checkpoint feeds §16a Story 2):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-quality-r3-task3-check.log 2>&1"
echo "check exit: $?"
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r3-task3-clippy.log 2>&1"
echo "clippy exit: $?"
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features full > .claude/PRPs/debug/v1-quality-r3-task3-test-norun.log 2>&1"
echo "test-norun exit: $?"

# Brief-Scope output check — #167 fix shape present, lazy re-read removed from the handler
grep -n "pub fn database_url" crates/api/api_utils/src/context.rs            # EXPECT: 1 hit
grep -n "db_url: SETTINGS.get_database_url()" crates/api/api_utils/src/context.rs   # EXPECT: 1 hit
grep -n "context.database_url()" crates/api/api/src/governance/admin_audit_stream.rs   # EXPECT: 1 hit
grep -n "context.settings().get_database_url()" crates/api/api/src/governance/admin_audit_stream.rs   # EXPECT: 0 hits
# EXPECT: check/clippy/test-norun exit 0; first 3 greps 1 hit each; last grep 0 hits.
```

**Commit subject:** `fix(governance): read admin_audit_stream DB URL from LemmyContext, captured at create-time (closes #167, task 3)`.

### Task 4: Retro

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-quality-r3-retro.md
modifies: []
requires:
  - task: 3
    reason: "Retro authored after all impl tasks land + phase-tip e2e passes; signals collected from the live phase branch + PR feedback."
```

**Goal:** author retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`. One H2 per role (Advisor / Planning / Impl / BM) with signals + lessons. Promote any new lessons to `.claude/lessons/feedback_*.md` in the same retro commit.

**ACTION:** write `.claude/PRPs/reports/v1-quality-r3-retro.md`: header (phase, PR#, merge SHA, dates); §1 what surprised us (per-role); §2 what to change (per-role; high-confidence only); §3 what to carry forward; §4 per-task complexity score (`<files>/<commits>/<runtime-min>/<max-log-silence-min>` per task) per `feedback_retro_task_complexity_score.md`; §5 lessons promoted; §6 four-role retro signals table. Specifically capture: did the option-a/option-b split (T1 vs T2) prevent the footgun-confusion it was designed to prevent? did the score-9 split-DQ resolve proceed-as-one, and was that the right call (feeds the e2e-edit-factor-over-weight lesson)?

**MIRROR:** `.claude/PRPs/reports/v1-quality-r2-retro.md`.

**GOTCHA:** the retro is NOT a status report — focus on signals worth carrying forward.

**VALIDATE:**

```bash
test -f .claude/PRPs/reports/v1-quality-r3-retro.md && wc -l .claude/PRPs/reports/v1-quality-r3-retro.md
grep -cE "^## (Advisor|Planning|Impl|BM)" .claude/PRPs/reports/v1-quality-r3-retro.md
# EXPECT: file exists, ≥ 50 lines; 4 role headers.
```

**Commit subject:** `docs(retro): v1-quality-r3 phase retro - PR #<N> merged <sha>`.

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full"` — after T1, T2, T3.
- **Lint:** `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"` — after T1, T2, T3.
- **Test target compile (R7):** `cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features full"` — after T3 (struct shape change).
- **e2e execution (R12 — MANDATORY, phase-tip):** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"` — ONCE post-T3. ~26 min on laptop. Raised as `kind: "validate-pending-laptop-e2e"`. Required because this is an env-var-management refactor class (`feedback_gate4_full_e2e_env_refactor_class.md`) — compile-only gates cannot catch RAII-lifetime / read-timing regressions (T2 guard scoping + T3 read-timing).
- **Migration round-trip:** N/A (no migrations).

## 15. Validation commands (DoD)

> Planner-side discipline (per `feedback_plan_dod_dry_run_at_write.md`): every command dry-runned by the advisor against `phase-v1-quality-r3` tip before plan approval. Wrappers on Linux at `scripts/brehon/cargo-*.sh`; on the laptop the `.bat` siblings are canonical (R9, clarify DQ 041).

### 15.1 Static analysis (per task in {task1, task2, task3})

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-quality-r3-<task>-check.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.2 Lint (per task in {task1, task2, task3})

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r3-<task>-clippy.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.3 Test target compile (R7 — task3)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features full > .claude/PRPs/debug/v1-quality-r3-task3-test-norun.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.4 e2e execution (R12 mandatory — phase-tip)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-quality-r3-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-quality-r3-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-quality-r3-e2e.log"
```

Raised as `kind: "validate-pending-laptop-e2e"` from T3's worker; advisor runs on laptop. Required pass (same test count as `governance-v0` base, 0 fail) before PR open. **Mandatory — not skippable** (R12).

### 15.5 Cross-cutting verification

- [ ] R1: no new `i32 ↔ i64` comparisons.
- [ ] R5: Task 0 enumerates all probes (Probes 0–8, N).
- [ ] R6: all clippy invocations use `--no-deps --features full -- -D warnings`.
- [ ] R7: `cargo test --no-run` runs after T3 (struct shape change).
- [ ] R8: never `-p <crate>` with `--features full` (the per-crate Probe 1/2 use `lemmy_db_schema`/`lemmy_utils` which the wrapper handles; workspace gates use `--workspace --features full`).
- [ ] R9: every cargo gate invokes the `.bat`/`.sh` wrapper.
- [ ] R10: every cargo invocation redirects to a `.claude/PRPs/debug/` file; exit via `$?`.
- [ ] R11: every e2e.rs Edit in T1/T2 has pre-located verbatim anchors in the brief.
- [ ] R12: full e2e executed once post-T3 and passed.
- [ ] §15-audit (T2): exactly 2 INIT + 2 GOV raw setter sites remain (the 2 documented bootstrap exceptions), each SAFETY-commented; zero unguarded test-body sites.
- [ ] #167: `context.database_url()` accessor present; `admin_audit_stream.rs` no longer calls `context.settings().get_database_url()`.
- [ ] No edits to files outside §11 (live edits: e2e.rs + context.rs + admin_audit_stream.rs + retro).
- [ ] Issue **#167** has a closing PR reference at merge time; **#158** remains open (premature-DRY, §12).

### 15.6 DoD per workflow (Shape G)

NOT APPLICABLE. Shape G SUSPENDED through 2026-06-01. All cargo gates run via validate-pending-laptop on the advisor session.

---

## 16. Acceptance criteria

- [ ] All 3 impl tasks (T1, T2, T3) completed in dependency order (T1→T2→T3).
- [ ] §15.1 (cargo check) exit 0 after T1, T2, T3.
- [ ] §15.2 (cargo clippy `--no-deps -- -D warnings`) exit 0 after T1, T2, T3.
- [ ] §15.3 (cargo test --no-run) exit 0 after T3.
- [ ] §15.4 (full e2e, R12) exit 0 phase-tip — base test count, 0 fail.
- [ ] §15.5 cross-cutting verification — all boxes ticked.
- [ ] §16a stories — all stories `[done]`.
- [ ] No edits to files outside §11 list.
- [ ] Retro committed per §13 Task 4.
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`.

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: Every test-body env-var setter is EnvVarGuard-scoped; the 2 bootstrap exceptions are documented (C4 follow-on closed)

- **Composing tasks:** Task 1 (bootstrap SAFETY exceptions), Task 2 (test-body wraps). Serial — both modify e2e.rs.
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"` (the phase-tip gate-4 run; R12).
- **Expected output:** base-equivalent pass count, 0 failed.
- **Brief-Scope outputs to verify** (used by `/brehon-verify`):
  - `crates/server/tests/e2e.rs`: the §15.4 audit prints OK — exactly 2 INIT + 2 GOV raw `std::env::set_var` sites remain (the 2 bootstrap fns), each with a `// SAFETY:` comment; zero unguarded test-body sites.
  - `crates/server/tests/e2e.rs`: `governance_fixtures::bootstrap` (@~827) and `admin_config_fixtures::bootstrap` (@~6138) each carry the §10.3 process-scoping SAFETY justification.

### Story 2: admin_audit_stream reads the DB URL captured at LemmyContext::create, not a request-time env re-read (Issue #167 fixed)

- **Composing tasks:** Task 3 (barrier — not in a cohort with T1/T2).
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"` (the same phase-tip run also exercises the admin_audit_stream path).
- **Expected output:** admin-audit-stream e2e test(s) pass; full suite 0 failed.
- **Brief-Scope outputs to verify:**
  - `crates/api/api_utils/src/context.rs` contains `db_url: String` field, `db_url: SETTINGS.get_database_url()` in `create()`, and `pub fn database_url(&self) -> &str`.
  - `crates/api/api/src/governance/admin_audit_stream.rs` calls `context.database_url()` and contains zero `context.settings().get_database_url()` calls.

> **Verification mapping:** `/brehon-verify` iterates this section, runs each Story's checkpoint against the worktree branch, and confirms each Brief-Scope output exists + matches its structural pattern. Phantoms trigger the catch-fire procedure.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (all probes confirmed).
- [ ] Task 1..3 committed; retro (Task 4) committed.
- [ ] §15 validation green at every gate (incl. mandatory full e2e, R12).
- [ ] §16a stories all `[done]`.
- [ ] PR opened by BM session against `governance-v0` with `--repo barrie-cork/lemmy`.
- [ ] CodeRabbit review complete; findings triaged per `feedback_pr_review_triage_pattern.md`.
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-quality-r3-verify.md` shows all stories ✓.
- [ ] Post-merge phase branch retained for retro reads.

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| A test-body site is wrapped with a guard that drops before `LemmyContext::create`/SETTINGS-access reads the var | LOW | HIGH | §10.2 binds `_g_init`/`_g_gov` in the test fn body (drops at fn exit, after all reads). R12 full e2e catches any mis-scoped guard (read-timing regression). |
| A bootstrap site is mistakenly wrapped (T1/T2 confusion) → guard drops at `bootstrap()` return, unsetting var before test body | MED | HIGH | T1 and T2 are **separate tasks** with separate briefs; T1's GOTCHA + §10.3 explicitly forbid wrapping. The §15.4 audit asserts exactly 2 bootstrap raw sites remain (mis-wrapping one drops the count to <2 → audit FAIL). |
| Multi-line GOV setter sites (4116/4461/4790/5677/5878) missed by a line-based grep | MED | MED | §15.4 audit uses `\s*` after `set_var(` to match multi-line forms; Probe 6 uses the same regex. The line-based brief grep is explicitly NOT the audit. |
| #167 read-timing change regresses a non-fixture caller (`server/src/lib.rs:210`) | LOW | MED | Production caller reads from real env/config at startup (unchanged semantics); only the *handler* re-read is removed. R12 full e2e + R7 test-norun cover the e2e path. |
| e2e.rs Edit-hang on the ~17.6k-line file (worker stalls mid-edit) | MED | MED | R11: every Edit has pre-located verbatim anchors (the canonical hang mitigation); T2's ~10 edits are bounded (r2 T5 ran 13 anchored edits cleanly). |
| Complexity score 9 > 8 → unnecessary split churn | LOW | LOW | §5.1 split-DQ recommends proceed-as-one with rationale (e2e-edit-factor over-weight); advisor decides at gate-1. |

---

## 19. Notes

- **Split-DQ (pending, `from: planner`, `kind: blocker`):** complexity score **9** exceeds the Sonnet `>8` threshold. Filed asking the advisor to choose split (boundary: r3 = e2e sweep T1+T2; r3b = #167 T3) vs proceed-as-one. **Planner recommendation: proceed-as-one** — the +6 is the flat-`+3`-per-e2e-task factor that `feedback_plan_complexity_e2e_edit_factor_overweights_doc_only_tasks.md` (promoted in r2 §19) flags as over-weighting bounded, well-anchored mechanical edits; the brief explicitly scoped a single plan; each task is single-file (T1/T2) or 2-file/2-crate (T3) and serial-gated. Do NOT self-resolve — advisor/user decides at gate-1.
- **`kind: "log"` pre-seed (resolved, `answered_by: planner`):** the option-b decision for the 2 bootstrap sites (keep raw `set_var` + SAFETY rather than wrap) is recorded as a planner log so the impl agent and a future env-leak phase know it was deliberate, not an oversight. Rationale: constant-valued vars + many-caller `bootstrap()` + `--test-threads=1` → benign process-scoping; wrapping would re-introduce the fixture-lifetime footgun.
- **GOVERNANCE_LOG_SIGNING_KEY count clarification:** the brief's claimed 11 GOV sites are correct, but **5 are multi-line** `set_var(` calls (name on its own line, literal hex value): 4116, 4461, 4790, 5677, 5878. A naive single-line `grep "set_var" | grep -v EnvVarGuard` (as the brief's draft DoD suggested) MISSES these 5. This plan's §15.4 audit + Task 0 Probe 6 use a `\s*`-tolerant regex instead. Flagged so the impl brief uses the robust audit.
- **EnvVarGuard already at root:** confirmed via clarify DQ 040 + Probe 5 (struct @111, before first mod @143). No hoist task (unlike r2 Task 4).
- **#167 is the r2 footgun incident itself:** `feedback_envvarguard_fixture_lifetime_footgun.md` records that the original v1-quality-r2 trigger was `admin_audit_stream`'s lazy `get_database_url()` re-read. This phase finally fixes the root cause (capture-at-create) rather than just guarding the setters.

---

## 20. Confidence score

- **Plan correctness:** 9/10 — all 23 sites + line numbers re-verified on the branch tip; #167 fix shape confirmed by clarify DQ + single-struct-literal enumeration. The −1 is residual uncertainty on exact per-module `use super::EnvVarGuard;` needs (resolved per-site at impl time).
- **Cargo budget:** 9/10 — validate-pending-laptop, ~6 GB peak; no migration round-trips.
- **Test coverage:** 9/10 — R12 mandatory full e2e is the right gate for this refactor class; the §15.4 audit mechanically proves the sweep completeness incl. the multi-line sites.
