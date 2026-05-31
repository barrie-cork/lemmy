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

Before writing the plan, read these in order:

1. **Plan template:** `.claude/PRPs/templates/plan.template.md` — follow every section; pay attention to §5 complexity scoring, §13 task format, §15 DoD commands, §16a story-to-task mapping.

2. **Prior plan (MIRROR reference for plan shape):** `.claude/PRPs/plans/v1-quality-r3.plan.md` — especially §3 (scope note), §5 (complexity score), §10 (implementation detail), §13 (task list format), §15 (DoD commands). Note: that plan explicitly defers Issue #167 to this plan via `a3d0e9941441-039`.

3. **Resolved clarify DQ for #167 shape:** read `.claude/decision-queue.json` — find the resolved entry `id: "a3d0e9941441-039"` (or in archive if not present) — it records the advisor's direction for Option A.

4. **Current `context.rs` (read the full file):**  
   `crates/api/api_utils/src/context.rs`  
   Note: `LemmyContext::create` currently takes 5 params (pool, client, pictrs_client, secret, rate_limit_cell) and returns `LemmyContext`. The struct has no `db_url` field yet. `context.settings()` returns `&'static Settings` from the `SETTINGS` global.

5. **Current `admin_audit_stream.rs` (read lines 100–160):**  
   `crates/api/api/src/governance/admin_audit_stream.rs`  
   Line 125: `let db_url = context.settings().get_database_url();` — this reads `LEMMY_DATABASE_URL` from the live process env. Line 126: `tokio_postgres::connect(&db_url, NoTls).await` — this is the LISTEN connection that fails after the env-var guard drops in tests.

6. **All `LemmyContext::create` call sites (enumerate before writing §10):**  
   Run: `grep -rn "LemmyContext::create" crates/ --include="*.rs"`  
   The plan §10 must list every call site and describe the required change at each (add the `db_url` argument). This is the `feedback_planner_enumerate_struct_callsites_for_addfield.md` discipline — every call site must be named before any task is dispatched.

7. **`Settings::get_database_url()` source (understand the env-read path):**  
   Run: `grep -rn "fn get_database_url" crates/ --include="*.rs"` then read the function body. Confirms whether it reads the env var at call time (expected: yes, from `SETTINGS` which is a `Lazy<Settings>` initialized at startup from env). The fix captures this value at `LemmyContext::create` time (before any guards are dropped) rather than re-reading at LISTEN-connect time.

8. **The `admin_audit_stream` e2e test block (pre-locate verbatim anchors):**  
   Run: `grep -n "admin_audit_stream\|fn forbidden_for_non_admin\|fn enforces_per_admin_cap\|fn emits_frame_on_config_change" crates/server/tests/e2e.rs | head -20`  
   The plan §10 must record the exact line numbers of the three test functions so the impl-task brief can pre-locate them. The e2e test update (remove the explicit `EnvVarGuard::set(LEMMY_DATABASE_URL, ...)` guards from the 3 test bodies — they were the band-aid fix in v1-quality-r2-fix-impl-1) is part of T1's scope.

9. **Mandatory lessons (file-class injection per `advisor-orchestrator.md` §2.4):**
   - `feedback_multi_write_handlers_need_transactions.md` — confirm: does the #167 fix introduce any new DB writes? (Expected: no — handler-only refactor. But the planner must confirm.)
   - `feedback_lemmy_error_no_std_error.md` — any new handler code must use `LemmyResult<()>` with `?`.
   - `feedback_fix_impl_pre_locate_e2e_anchors.md` — mandatory for any e2e.rs edit; the plan §13 T1 brief constraint must require pre-location of verbatim anchors.
   - `feedback_rust_visibility_cross_crate.md` — the new `db_url` accessor must be `pub` (visible from `lemmy_api` which imports `lemmy_api_utils`).
   - `feedback_envvarguard_fixture_lifetime_footgun.md` — the #167 fix is the proper architectural resolution of the footgun; the plan §10 must explain why capturing at `create()` eliminates the footgun.
   - `feedback_envvarguard_audit_window.md` — if any audit script in the DoD checks for `get_database_url` absence, use a file-content grep (not a windowed scan).

10. **Canonical sibling plan for format (read one more):** `.claude/PRPs/plans/v1-RT-r3.plan.md` — skim section headers to confirm §1–§20 shape matches template. The r3b plan must have the same structure.

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
