# Conformance Audit — `admin_audit_stream.rs` — 2026-06-01

**Scope:** `file crates/api/api/src/governance/admin_audit_stream.rs`
**Trigger:** v1-quality-r3b brief-author prevention checkpoint (§3.1.1)
**Run by:** advisor session / governance-v0
**Hypothesis discipline:** every finding is a hypothesis until compiler-confirmed or sibling-diff-proven.

---

## Summary

| Axis | Title | Tier | Finding |
|------|-------|------|---------|
| 1 | Conn-type / tx-boundary | — | No `.run_transaction` callers. N/A. |
| 2 | Append reborrow shape | — | No `governance_log::append` callers. N/A. |
| 3 | Trait-bound completeness | — | No `#[async_trait]` generic functions. N/A. |
| 4 | Error idiom at trust boundary | — | No `.unwrap_or_default()` / `.unwrap_or(` patterns. N/A. |
| 5 | Conn acquisition idiom | **Tier 2** | `let mut pool` (not `let pool = &mut`). Justified; see below. |
| 6 | ADR-015 pseudonym handling | — | No `actor_pseudonym` references. N/A. |

**Tier-1 findings: 0.** No brief scope changes required.

---

## Axis 5 — Conn acquisition idiom (Tier 2, no action needed)

**Evidence string:**
`axis-5: admin_audit_stream.rs:219-220 new=let mut pool = context_for_stream.pool(); let Ok(mut conn) = get_conn(&mut pool).await else { continue }; expected=let pool = &mut context.pool(); let conn = &mut get_conn(pool).await?;`

**Pattern observed:**
```rust
// Line 219-220 (inside stream! macro body)
let mut pool = context_for_stream.pool();
let Ok(mut conn) = get_conn(&mut pool).await else { continue };
```

**Canonical sibling pattern (`admin_config.rs:172`, `submit_jury_vote.rs:131-132`, `admin_assign_jury.rs:88-89`):**
```rust
let pool = &mut context.pool();
let conn = &mut get_conn(pool).await?;
```

**Why Tier-2 (not Tier-1):** The divergence is structurally justified by the `stream!` macro context. Inside an `async_stream::stream!` body, the connection must be freshly acquired on each notification loop iteration — it cannot be held across `await` yield points (that would hold the `DbPool` borrow open across an unbounded stream wait). The `let mut pool` binding in `let-else` form (`.await else { continue }`) correctly handles the error by skipping the notification rather than terminating the stream. The `let pool = &mut ...` canonical form with `?`-propagation would terminate the entire SSE stream on a single transient pool-acquisition failure — wrong semantics for a long-lived SSE handler.

**Cross-check:** `admin_dashboard.rs:68`, `admin_dashboard_html.rs:73/247`, `admin_reputation_rollup.rs:30`, `admin_reputation_stats.rs:79`, `get_my_reputation.rs:37` all use `let mut pool = context.pool()` in non-transaction read paths. The pattern is established in the governance module for read-path handlers; `admin_audit_stream.rs` follows this established read-path pattern. The difference from the `let pool = &mut` form is purely one of mutability placement, not a weaker contract.

**Action:** Log to retro watch items. No brief scope change required.

---

## #167 Fix scope pre-check (contextual, not a conformance axis)

**Question from bootstrap §4 watchpoint 1:** does `context.database_url()` re-read `LEMMY_DATABASE_URL` from the live process environment?

**Finding:** No. `LemmyContext::database_url()` returns `&self.db_url` (context.rs:68), where `db_url` was captured at `LemmyContext::create()` time via `SETTINGS.get_database_url()` (context.rs:36). `Settings::get_database_url()` reads `LEMMY_DATABASE_URL` from the environment at that call site — which is startup time, not handler invocation time. Once `LemmyContext` is constructed, `context.database_url()` is a baked value that does NOT re-read the env var.

**Implication for #167:** The current code at `admin_audit_stream.rs:125` (`let db_url = context.database_url()`) is **already the correct architecture**. It does not re-read `LEMMY_DATABASE_URL` at handler invocation time. The `context.database_url()` method was added as the fix for issue #167 — it is already wired in. 

**What the plan must verify:** whether the e2e tests still require `LEMMY_DATABASE_URL` to be set during `admin_audit_stream` test execution (for the `bootstrap()` fixture's `LemmyContext::create()` call, not for the handler itself). The `bootstrap()` fixture at e2e.rs:846 uses `EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url)` — that guard's lifetime must extend through `LemmyContext::create()` but not necessarily through the handler's `tokio_postgres::connect(context.database_url(), ...)` call (because `context.database_url()` is already the baked value).

**Key planning question:** if `EnvVarGuard` drops before the test body calls `admin_audit_stream()`, will the handler's `tokio_postgres::connect(db_url, ...)` still work? Answer: yes — because `db_url` is captured in the `LemmyContext` struct at creation time. The guard only needs to be alive during `LemmyContext::create()`.

---

## What the plan must still deliver

Despite zero Tier-1 audit findings, the plan still needs to confirm:

1. **Verify the test does NOT drop the guard before `context` is fully constructed.** The `bootstrap()` fixture creates `LemmyContext` while the guard is alive (guard at line 846, `context` built in the same scope). ✓ Already correct structure.
2. **Confirm the e2e tests can drop the `LEMMY_DATABASE_URL` guard before `admin_audit_stream()` is called** — this is the architectural proof Issue #167 asks for. The plan's impl-task should add an assertion test that explicitly shows the guard is dropped before the handler runs.
3. **The DoD assertion** `grep -n "get_database_url\|LEMMY_DATABASE_URL" crates/api/api/src/governance/admin_audit_stream.rs` exits non-zero (i.e., neither string appears in the file) — this already holds since the file uses `context.database_url()`, not either of those patterns. The planner must verify this is already satisfied.

---

## Metrics update

See `.claude/PRPs/audit-metrics/admin_audit_stream.json` — updated by this run.
