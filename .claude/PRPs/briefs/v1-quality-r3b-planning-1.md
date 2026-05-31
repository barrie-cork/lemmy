[role:planning] v1-quality-r3b — Issue #167 LemmyContext::create DB URL capture, admin_audit_stream fix

---

## §1. Role + dispatch

**Role:** `[role:planning]`  
**Summary:** Author `.claude/PRPs/plans/v1-quality-r3b.plan.md` covering Issue #167: capture the Postgres database URL once at `LemmyContext::create` time (in `context.rs`) and use it in `admin_audit_stream.rs` instead of re-reading the live env var on each LISTEN connection attempt.

---

## §2. Scope

**Produce:** `.claude/PRPs/plans/v1-quality-r3b.plan.md` — a complete Brehon sub-phase plan following `.claude/PRPs/templates/plan.template.md`.

**Explicit boundaries:**

- **In scope (Issue #167, Option A):**
  - Add a `db_url: String` field to the `LemmyContext` struct in `crates/api/api_utils/src/context.rs`
  - Update `LemmyContext::create(...)` to accept and store this value
  - Update all `LemmyContext::create(...)` call sites (enumerate them — use `grep -rn "LemmyContext::create" crates/`)
  - Replace `context.settings().get_database_url()` at `admin_audit_stream.rs:125` with `context.db_url()`  (or equivalent accessor)
  - Add `pub fn db_url(&self) -> &str` accessor to `LemmyContext`
  - Update the `admin_audit_stream` e2e test to verify the handler works without `LEMMY_DATABASE_URL` being explicitly set in the test body
  - Single impl-task (T1) — no `[P]` (two files, serial)

- **Out of scope:**
  - Issue #158 (`emit_reputation_event` helper extraction) — only 2 callers; premature-DRY gate. **Do NOT include.**
  - `BREHON_DISABLE_*` sites in e2e.rs — separate sweep
  - Any migration under `crates/db_schema/migrations/` — this phase is handler-only
  - Any new API endpoint — existing handler refactor only

**Commit only:** `.claude/PRPs/plans/v1-quality-r3b.plan.md`  
**Do NOT author:** implementation code, migration files, test fixtures, other plans.

---

## §3. Required reading

The advisor has pre-populated key research results below to save context. Read the listed files; skip the greps (results are given).

### §3a. Pre-populated research (do NOT re-run these greps)

**Option A shape (from resolved DQ `052f0d5c016d-001` + `a3d0e9941441-039`):**
- `LemmyContext::create(...)` calls `SETTINGS.get_database_url()` INTERNALLY and stores as `db_url: String` field on the struct.
- Zero new parameters added to `create(...)`.
- New accessor: `pub fn database_url(&self) -> &str { &self.db_url }` (named `database_url`, not `db_url`).
- `admin_audit_stream.rs:125` changes from `context.settings().get_database_url()` to `context.database_url()`.
- `Settings::get_database_url()` reads `LEMMY_DATABASE_URL` from the live env at call time (confirmed). Capturing at `create()` time (before guards drop) fixes the test footgun.

**`LemmyContext::create` call sites (full enumeration — 16 sites, 2 non-test + 14 in e2e.rs):**
```
crates/api/api_utils/src/context.rs:79      ← inside create() itself (self-call factory)
crates/server/src/lib.rs:210                ← main server startup
crates/server/tests/e2e.rs:861
crates/server/tests/e2e.rs:2602
crates/server/tests/e2e.rs:3366
crates/server/tests/e2e.rs:4148
crates/server/tests/e2e.rs:4472
crates/server/tests/e2e.rs:4796
crates/server/tests/e2e.rs:4925
crates/server/tests/e2e.rs:5050  (context_a)
crates/server/tests/e2e.rs:5096  (context_b)
crates/server/tests/e2e.rs:5673
crates/server/tests/e2e.rs:5869
crates/server/tests/e2e.rs:6159
crates/server/tests/e2e.rs:16816
crates/server/tests/e2e.rs:17464
```
Note: `context.rs:79` is the factory function itself — it reads `SETTINGS.get_database_url()` internally and passes nothing extra. The other 15 sites pass arguments to `create()`; since we add ZERO new params (Option A), they need NO change. Only the factory body changes.

**`admin_audit_stream` e2e test anchors (pre-located):**
- `admin_audit_stream_forbidden_for_non_admin` starts at line **8251**; `EnvVarGuard` band-aid at line **8256**
- `admin_audit_stream_enforces_per_admin_cap` starts at line **8276**; `EnvVarGuard` band-aid at line **8281**
- `admin_audit_stream_emits_frame_on_config_change` starts at line **8344**; `EnvVarGuard` band-aid at line **8353**

T1 e2e edit: remove the 3 `let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);` lines (lines 8256, 8281, 8353). These were the v1-quality-r2-fix-impl-1 band-aids, superseded by the architectural fix.

**Lesson injections (file-class, already evaluated — include in plan §14):**
- `feedback_lemmy_error_no_std_error.md` — any new code uses `LemmyResult<()>` with `?`
- `feedback_fix_impl_pre_locate_e2e_anchors.md` — impl-task brief must cite lines 8256/8281/8353 verbatim
- `feedback_rust_visibility_cross_crate.md` — accessor must be `pub` (called from different crate)
- `feedback_envvarguard_fixture_lifetime_footgun.md` — plan §10 must explain why capture-at-create eliminates the footgun
- `feedback_multi_write_handlers_need_transactions.md` — no new DB writes in this fix; note this in §10

### §3b. Required reads (small, load-bearing)

1. **Plan template:** `.claude/PRPs/templates/plan.template.md` — follow every section; §5 complexity scoring, §13 task format, §15 DoD commands, §16a story map.

2. **Current `context.rs`:** `crates/api/api_utils/src/context.rs` — read the full file to understand the struct definition, existing fields, `create()` signature, and where to add `db_url: String` and the accessor.

3. **Current `admin_audit_stream.rs` lines 115–135:** `crates/api/api/src/governance/admin_audit_stream.rs` — read just these 20 lines to see the exact `get_database_url()` call at line 125 you will replace.

---

## §4. Constraints

- **Follow the plan template exactly.** Every section from §1 through §20 must be present. §5 complexity score must use the documented formula. §13 tasks must use the `Task N: <title>` + `IMPLEMENT:` / `VALIDATE:` format with `[P]` markers as appropriate.
- **Enumerate all `LemmyContext::create` call sites in §10** before writing §13. Do not guess — run the grep and list every file:line. A missed call site will produce a compile error at impl-time; the planner is responsible for finding them all.
- **§10 must include the e2e test update** as part of T1's scope: remove the 3 `EnvVarGuard::set(LEMMY_DATABASE_URL, ...)` band-aid lines from the `admin_audit_stream` test bodies (they were the v1-quality-r2 fix-impl-1 workaround, now superseded by the architectural fix).
- **§10 must specify the accessor name** — either `pub fn db_url(&self) -> &str` or `pub fn db_url(&self) -> &String` — and note whether it should be `pub` (yes: called from `admin_audit_stream.rs` in a different crate).
- **§15 DoD must include a grep check** that confirms `get_database_url` no longer appears in `admin_audit_stream.rs` after the fix: `grep -c "get_database_url" crates/api/api/src/governance/admin_audit_stream.rs` must exit 0 or return 0.
- **§15 DoD must include gate-4 full e2e** (`cargo test --workspace --test e2e --features full`) — required by `feedback_gate4_full_e2e_env_refactor_class.md` for any change touching env-var acquisition paths, even a fix.
- **§5 complexity pre-scoring:** expected score is low (1 struct-field add + 2 file edits + N call-site fixes + 1 e2e test update). If score comes out > 8, note it and recommend proceed-as-one (no further split warranted for a single-impl-task scope).
- **DQ mid-task discipline:** if a blocker arises during planning, write a `kind: "blocker"` DQ entry, commit + push, and stop. Do not guess.
- **Commit only the plan file.** Subject: `docs(plan): v1-quality-r3b — Issue #167 LemmyContext DB URL capture`
