# Plan: v1-federation-inbound-c — reader-side append-history fix on `get_inbound_config_int` + mirror

## 1. Summary

This sub-phase ships a **two-line reader-side correctness fix** on the federation-inbound `governance_config` read path, plus a single e2e regression test that exercises the post-fix override-after-baseline contract. The deliverable is a code-only change (zero migration, zero handler-contract change, zero new ADR-affecting decision) that closes a latent staleness bug in `wrap_governance_inbound`'s size-cap and rate-cap reads. Acceptance: after Tasks 1+2 land, an admin-edit INSERT-with-newer-`valid_from` on any `federation.inbound.*` config key is read by the inbox wrapper on the very next request; the Task 3 e2e test asserts this end-to-end via `per_peer_rate_per_hour` (override 100->2, third activity 429s instead of falling through to the seeded cap).

## 2. Source

- `.claude/PRPs/briefs/v1-federation-inbound-c-planning-1.md` (this plan's authority anchor, scope (a) only) @ `4fc1d7722`
- `.claude/PRPs/handovers/v1-federation-inbound-c-bootstrap.md` (2026-05-20 base context) @ `253720491`
- `.claude/PRPs/handovers/advisor-v1-federation-inbound-c-2026-05-21-scope-a-only.md` (2026-05-21 scope delta) @ `253720491`
- `.claude/PRPs/plans/v1-federation-inbound-b.plan.md` §15 (validate-pending-laptop DoD shape, same-lane exemplar) @ `governance-v0`
- `.claude/PRPs/reports/v1-federation-inbound-b-retro.md` (carry-forward signals)
- `crates/db_schema/src/source/governance/governance_config.rs:34-40` (append-only contract — model file doc-comment)
- `migrations/2026-04-18-000000-0000_add_governance_config/up.sql:38-52` (append-history schema invariant + `governance_config_current` view definition)
- `crates/api/api/src/governance/config.rs:740-769` (canonical reader pattern; inline comment at 741-746 cites the regression history)
- DQ #324 (advisor-resolved 2026-05-21): NEW e2e tests use INSERT-with-newer-`valid_from`; existing tests using UPDATE stay unchanged
- DQ #325 (advisor-resolved 2026-05-21): Task 3 updates the workaround comment at `e2e.rs:15605-15610` (option `update-comment-in-task-3`)

Lessons that bind decisions (cited where they fire):

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` Case A (e2e error shape — uniform `LemmyResult<()>`)
- `.claude/lessons/feedback_async_pool_test_pattern.md` (e2e connection acquisition)
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` (conditional — only if the e2e test's seed inserts must be atomic)
- `.claude/lessons/feedback_features_full_workspace_only.md` + `.claude/lessons/feedback_features_full_p_crate_incompatible.md` (cargo-clippy `--features full` discipline)
- `.claude/lessons/feedback_clippy_test_style.md` (R1 — `i64::from(...)` not `as i64`)
- `.claude/lessons/feedback_complexity_score_pre_split.md` (§5 score computation)
- `.claude/lessons/feedback_advisor_watchpoint_specificity.md` (§4 watchpoint citations)
- `.claude/lessons/feedback_cohort_dq_id_collision.md` Option 3 (advisor pre-reserves DQ ids for Tasks 1+2 cohort)
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` (post-bm-cut worktree hygiene)
- `.claude/lessons/feedback_retro_not_report.md` + `.claude/lessons/feedback_four_role_retro_signals.md` + `.claude/lessons/feedback_retro_task_complexity_score.md` (retro shape)
- `.claude/lessons/feedback_windows_e2e_requires_bat_wrapper.md` (Phase-2 e2e invocation discipline)
- `.claude/lessons/feedback_pre_phase_dod_smoke_test.md` + `.claude/lessons/feedback_plan_dod_dry_run_at_write.md` (§15 dry-run discipline)
- `.claude/lessons/feedback_wrapper_script_flag_silence.md` (Task 0 Probes 1-4 rationale)
- `.claude/lessons/feedback_brehon_verify_pre_merge.md` (§16a stories drive `/brehon-verify`)

ADRs: none modified. The fix is mechanical and preserves the existing v0 simplifications (ADR-001..ADR-015 unchanged); the append-only design is captured in the migration up.sql comment at line 6-17 and the model file doc-comment cited above (not a separate ADR).

## 3. Problem statement

The `governance_config` table is append-only by design: admin edits INSERT a new `(scope, key, valid_from)` row rather than UPDATE the existing one, so the full audit trail is preserved. The migration also defines a `governance_config_current` SQL view that surfaces the latest row per `(scope, key)` via `DISTINCT-ON (...) ORDER BY valid_from DESC`.

**Two reader sites query the BASE TABLE without `.order_by(governance_config::valid_from.desc())` before `.first(...)`**, so Postgres returns rows in arbitrary order once history accumulates. Both sites read size caps + rate caps that gate inbound federation enforcement — a stale read means the gate fires (or doesn't) against a value the admin already overrode.

| Site | Function / scope | Defect at | Reads keys |
|---|---|---|---|
| `crates/apub/activities/src/governance/inbox.rs:421-438` | `get_inbound_config_int` (private helper used by `wrap_governance_inbound` for size/rate-cap reads) | `.first::<Option<i64>>(conn)` at line 427 — no preceding `.order_by(...)` between `.select(governance_config::value_int)` (line 426) and the `.first(...)` call | `payload_size_cap_<kind>`, `federation.inbound.per_peer_storage_cap`, `federation.inbound.per_peer_rate_per_hour` |
| `crates/apub/activities/src/governance/publish_trust_attestation.rs:142-152` | inline `let actor_cap: i64 = { ... }` block (mirror of the helper — comment at line 135-138 justifies the duplication on circular-dep grounds) | `.first::<Option<i64>>(conn)` at line 146 — no preceding `.order_by(...)` between `.select(governance_config::value_int)` (line 145) and the `.first(...)` call | `federation.inbound.per_actor_attestation_rate_per_hour` |

A third reader at `crates/api/api/src/governance/config.rs:740-769` is the **canonical pattern**: it reads the same base table with `.order_by(governance_config::valid_from.desc())` at line 757, and the inline comment at lines 741-746 cites the regression history that gave rise to the discipline ("Regression history: commit 8e3bba1 dropped the governance_config_current view; this code path needs to do the latest-wins ordering itself").

A fourth reader at `crates/api/api/src/governance/admin_dashboard.rs:294-303` already does the ordering correctly (verified by `rg "governance_config::table" crates/`); no fix needed there.

The defect class is not new code; it is a reader-side correctness bug carried over from fed-in-b. The fix is mechanical and ties to one §10 pattern (Pattern 10.1 below) and the two §13 reader-fix tasks.

## 4. Solution statement

Add a single Diesel `.order_by(governance_config::valid_from.desc())` chain to each of the two defective reader sites, inserted **between** the existing `.select(governance_config::value_int)` chain and the `.first::<Option<i64>>(conn)` call. Everything else stays unchanged: filters, error mapping, null handling, the helper-duplication comment at `publish_trust_attestation.rs:135-138`.

Add one e2e regression test inside the existing `mod v1_federation_inbound_b_fixtures` (`crates/server/tests/e2e.rs:15510-15773`) that:

1. Bootstraps an allowlisted peer.
2. INSERTs a NEWER-`valid_from` row overriding the seeded `federation.inbound.per_peer_rate_per_hour` cap from 100 -> 2.
3. Sends 2 activities (succeed under the 2-cap).
4. Sends a 3rd activity -> expects 429.

The 429 only fires if the reader picked the override row (not the seed). Without Tasks 1+2 the test would FAIL because the reader could return the seed cap (100) and let the 3rd activity through. With Tasks 1+2 the test passes deterministically.

Also update the workaround comment at `e2e.rs:15605-15610` per DQ #325 — replace the "raw INSERT returns an arbitrary row" claim (true pre-fix, obsolete post-fix) with a one-line note explaining the post-fix landscape; leave the existing UPDATE-based test body unchanged (it still works post-fix for historical-continuity reasons per DQ #324).

No new module, no new helper, no new file. Two file edits + one test addition.

## 5. Metadata

- **Phase:** `v1-federation-inbound-c`
- **Branch:** `phase-v1-federation-inbound-c` (cut by `bm-task` after User Gate 1)
- **Target impl-task model:** `sonnet-4-6` (Junior daemon default)
- **Estimated tasks:** 5 (Task 0 pre-flight + Tasks 1-3 work + Task 4 retro)
- **Estimated cargo budget:** N/A (validate-pending-laptop DoD — cargo runs on laptop, NOT EliteDesk; per Shape-G-suspended PMD `project_shape_g_suspended_2026_05_16`)
- **Forbidden-window applicability:** standard (per advisor-orchestrator.md table; non-binding for impl-task throughput since cargo runs on laptop, but binding for advisor-side §15 dry-run runs and Phase-2 e2e)
- **Complexity score:** **2/10** — see breakdown below

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Target = `sonnet-4-6` -> split-DQ threshold is `> 8`.

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 3 impl tasks (Task 1-3); below threshold |
| Migrations touched | +2 each | 0 | Zero migrations (PRECON-5) |
| Crates touched | +1 each | 2 | `crates/apub/activities/` + `crates/server/tests/` |
| `crates/server/tests/e2e.rs` edits | +3 each | 1 | One Task 3 edit (Append into `mod v1_federation_inbound_b_fixtures`; <=200-line insertion budget) |
| New ADR-affecting decisions | +2 each | 0 | None |
| Cargo budget peak above 6 GB | +1 per GB | 0 | Validate-pending-laptop runs on laptop (Shape G suspended); budget non-binding |
| **Total** | — | **2/10** | Threshold `> 8` not crossed -> no split-DQ |

### 5.2 Per-task complexity ceiling (non-Sonnet target only)

Not applicable — target is `sonnet-4-6`. Sonnet ceilings (`<= 4` files / `<= 2` crates) are well within each task's footprint:

- Task 1: `modifies: [inbox.rs]` — 1 file / 1 crate.
- Task 2: `modifies: [publish_trust_attestation.rs]` — 1 file / 1 crate.
- Task 3: `modifies: [e2e.rs]` — 1 file / 1 crate.

## 6. Relationship to other v1-federation-inbound-* sub-phases

- **Depends on:** `v1-federation-inbound-b` (merged at PR #139, governance-v0 commit `16ede83b6` Merge of `phase-v1-federation-inbound-b`). The defect lives in fed-in-b code (`wrap_governance_inbound` + `publish_trust_attestation`) and the e2e regression test extends fed-in-b's `mod v1_federation_inbound_b_fixtures` sibling module.
- **Followed by:** likely `v1-federation-inbound-d` (deferred (b) Copilot DoS-hardening family per brief §0.1.1) — re-evaluated at this sub-phase's retro. If DoS-hardening is shelved, the next federation lane is open.
- **Subsumed work (no overlap):** `brehon-conformance-audit` lane subsumes (c) Phase-6 convention-divergence audit per brief §0.1.1; in-flight planning brief at `.claude/PRPs/briefs/brehon-conformance-audit-planning-1.md` committed at `57ce4c322` on `governance-v0` (referenced via MEMORY.md "Active workflow state").

## 7. Preflight guardrails inherited from prior phases

- **R1:** every `i32 / i64` comparison uses `i64::from(...)`, never `as` cast (per `feedback_clippy_test_style.md`). The new e2e test must use `i64::from(...)` if comparing `u32` activity counts against an `i64` cap.
- **R5:** Task 0 enumerates ALL probes explicitly (Probes 0-4 per `.claude/rules/pre-phase-harness-audit.md`); do NOT inherit implicitly.
- **R6:** all clippy invocations use `--no-deps -- -D warnings` uniformly (per `feedback_clippy_test_style.md` + JM-b retro Event 3).
- **R7:** Task 3 (struct or re-export touchpoint? — no; but `cargo test --no-run` still runs as a sanity check per Task 3's VALIDATE block to confirm e2e harness re-links).
- **R9:** Tasks 1+2 are `[P]` only because their `modifies` arrays share zero paths (`inbox.rs` vs `publish_trust_attestation.rs`); the YAML overlap check in `.claude/rules/advisor-orchestrator.md` §4.1 step 4 must confirm this before cohort dispatch.
- **R-laptop-cargo:** all cargo invocations run on the laptop (validate-pending-laptop DoD per `.claude/rules/advisor-orchestrator.md` §5.2); never on the EliteDesk daemon (Shape G suspended per PMD `project_shape_g_suspended_2026_05_16`, DQ #229 pending re-enable 2026-06-01).
- **R-cohort-DQ-pre-reserve:** advisor reserves Tasks 1+2 `validate-pending-laptop` DQ stubs in `pending[]` BEFORE the cohort dispatches (per `feedback_cohort_dq_id_collision.md` Option 3). Each impl brief names its assigned DQ id explicitly in §4 Constraints.
- **R-windows-e2e:** Phase-2 e2e MUST use the bat wrapper invocation per `feedback_windows_e2e_requires_bat_wrapper.md`. Never bare `cargo test` (libpq.dll path); never `-p lemmy_server --features full` (no `full` feature on `lemmy_server`).

## 8. Flow design

Before Tasks 1+2 (defect): `wrap_governance_inbound` calls `get_inbound_config_int(..., "federation.inbound.per_peer_rate_per_hour")` which runs `SELECT value_int FROM governance_config WHERE scope='instance' AND key='...' LIMIT 1` with no `ORDER BY` clause — Postgres returns an arbitrary row, so the per-peer rate gate fires against a STALE cap (e.g. the seed cap=100 instead of the admin override cap=2). The inner handler then permits (or rejects) traffic on stale policy.

After Tasks 1+2 (fix): the helper runs the same SELECT plus `ORDER BY valid_from DESC` before `LIMIT 1`. The per-peer rate gate sees the LATEST cap (admin override cap=2). The inner handler fires on the latest policy. The same shape applies to Task 2's `publish_trust_attestation` inline `actor_cap` block — the SELECT chain inside the `let actor_cap: i64 = { ... }` block grows one `.order_by(...)` line; everything else (filter chain, error mapping, null-handling, the `subject_url` extraction outside the block) stays identical.

Task 3's e2e test exercises the Task-1 path (the `inbox.rs:421-438` reader is the one called by `wrap_governance_inbound`'s Gate 4 per-peer rate check). It does not exercise Task 2's `publish_trust_attestation` reader directly — Task 2's correctness is enforced by clippy + the canonical-mirror discipline + the per-pattern §10 citation. Adding a second e2e test for Task 2 would broaden scope beyond PRECON-4 ("ONE test, append-history-aware") and is deferred to retro consideration if a regression class emerges later.

## 9. Mandatory reading

The impl-task subagent MUST `Read` each of these before its first edit. Cite each in plan §10.

### Schema / type definitions

- **`crates/db_schema/src/source/governance/governance_config.rs:1-54`** — model file. Note lines 16-20 + 34-40 doc-comments confirming append-only design and the `governance_config_current` view as the canonical "latest row per key" surface.
- **`migrations/2026-04-18-000000-0000_add_governance_config/up.sql:6-17` + `:38-52`** — design intent + the `(scope, key, valid_from)` unique index that makes append-history possible + the `governance_config_current` view definition.

### Existing patterns (the MIRROR refs §13 tasks point at)

- **`crates/api/api/src/governance/config.rs:730-770`** — the canonical reader pattern. **Read the inline comment at lines 741-746 verbatim**; it cites the regression history with commit `8e3bba1`.
- **`crates/api/api/src/governance/admin_dashboard.rs:290-303`** — a second already-correct reader (uses `.order_by(governance_config::valid_from.desc())` at line 302). Confirms the discipline is in active use elsewhere; reference only — no edit.
- **`crates/apub/activities/src/governance/inbox.rs:415-503`** — the surrounding wrapper context (`v1-federation-inbound-b Task 4 — wrapper, trait, rate-limit, label handler`); shows how `get_inbound_config_int` is called from `wrap_governance_inbound` and from each `publish_*` size-cap pre-check.

### Adjacent test fixtures (so impl doesn't re-invent seed helpers)

- **`crates/server/tests/e2e.rs:15510-15773`** — the existing `mod v1_federation_inbound_b_fixtures` module (264 lines). **Read ONLY this range** via `Read offset:15510 limit:264` — the file is 15773 lines; never full-file Read. Identify which sibling test to mirror in Task 3 (see §10.2 — `per_peer_rate_limit_returns_429` at lines 15600-15628 is the closest shape).
- **`crates/server/tests/e2e.rs:15605-15610`** — the workaround comment Task 3 step (b) updates per DQ #325.

### Lessons (each gates a §13 task; cite in §3 of each impl-task brief at advisor-side dispatch)

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A discipline (sibling module uses `LemmyResult<()>` per the read above).
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish + DbPool::Conn` for connection acquisition.
- `.claude/lessons/feedback_clippy_test_style.md` — R1 (`i64::from(...)` not `as i64`) + R6 (`--no-deps -- -D warnings`).
- `.claude/lessons/feedback_features_full_workspace_only.md` — clippy must use `--workspace --features full` to see the governance code under the `#[cfg(feature = "full")]` gate.
- `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — never `-p lemmy_server --features full`.
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` (conditional) — only fires if Task 3's seed inserts must be atomic; the simple shape (one INSERT for the override row, baseline already seeded by migration) is single-statement and does not require `conn.run_transaction(...)`.
- `.claude/lessons/feedback_windows_e2e_requires_bat_wrapper.md` — Phase-2 e2e invocation discipline (laptop-bg).
- `.claude/lessons/feedback_cohort_dq_id_collision.md` — Option 3 advisor pre-reserves cohort DQ ids.

## 10. Patterns to mirror

### 10.1 Canonical reader pattern — `.order_by(valid_from.desc())` between `.select(...)` and `.first(...)`

**Mirror:** `crates/api/api/src/governance/config.rs:740-769`

Verbatim shape (with inline comment that cites the regression history — the impl-task brief at advisor-side dispatch quotes this block):

```rust
async fn fetch_value_at_scope(
  pool: &mut DbPool<'_>,
  scope_str: Cow<'static, str>,
  key: &str,
) -> LemmyResult<Option<CachedValue>> {
  let conn = &mut get_conn(pool).await?;

  // governance_config is append-only with multiple rows per (scope, key)
  // keyed by valid_from. ORDER BY valid_from DESC + LIMIT 1 (.first) is
  // load-bearing — without it Postgres returns arbitrary order and reads
  // can return stale seeded rows instead of admin_set_config writes.
  // Regression history: commit 8e3bba1 dropped the governance_config_current
  // view; this code path needs to do the latest-wins ordering itself.
  let row: Option<ConfigRow> = governance_config::table
    .filter(governance_config::scope.eq(scope_str.into_owned()))
    .filter(governance_config::key.eq(key))
    .select((
      governance_config::value_type,
      governance_config::value_int,
      governance_config::value_float,
      governance_config::value_bool,
      governance_config::value_text,
    ))
    .order_by(governance_config::valid_from.desc())   // THE LOAD-BEARING LINE
    .first::<ConfigRow>(conn)
    .await
    .optional()?;
  // ...
}
```

**Apply (Task 1 — `crates/apub/activities/src/governance/inbox.rs:421-438`):**

Insert `.order_by(governance_config::valid_from.desc())` on a new line between line 426 (`.select(governance_config::value_int)`) and line 427 (`.first::<Option<i64>>(conn)`), preserving 4-space indentation and the trailing dot of `.select(...)`. Post-edit, the chain reads:

```rust
let val: Option<i64> = governance_config::table
  .filter(governance_config::scope.eq("instance"))
  .filter(governance_config::key.eq(config_key))
  .select(governance_config::value_int)
  .order_by(governance_config::valid_from.desc())   // NEW
  .first::<Option<i64>>(conn)
  .await
  .map_err(|_e| {
    LemmyErrorType::Unknown(format!("governance_config.{config_key} not seeded"))
  })?;
```

Do **NOT** touch the function's doc-comment at lines 418-420 (it justifies the helper's existence on circular-dep grounds and stays accurate post-fix). Do **NOT** edit the function signature, the filter chain, the error mapping, or the null-handling.

**Apply (Task 2 — `crates/apub/activities/src/governance/publish_trust_attestation.rs:142-152`):**

Insert `.order_by(governance_config::valid_from.desc())` on a new line between line 145 (`.select(governance_config::value_int)`) and line 146 (`.first::<Option<i64>>(conn)`), preserving 8-space indentation (one level deeper than Task 1's site because this is inside an inline block) and the trailing dot of `.select(...)`. Preserve every other line in the block unchanged (the helper-duplication comment at 135-138, the filter chain at 143-144, the error mapping at 148-150, the null-handling at 151).

**GOTCHAs (apply to both sites):**

- Do **NOT** introduce `governance_config_current` view usage. The view is **not registered as a Diesel table** in `crates/db_schema_file/src/schema.rs` (verified by `grep -c governance_config_current crates/db_schema_file/src/schema.rs` -> 0). Diesel-typed reads must go to the base table + `.order_by(...)`. Using the view requires raw SQL (out of scope per PRECON-2; the view IS used elsewhere via raw SQL at `admin_config.rs:1003-1073` — that is a different reader class and not part of this sub-phase).
- Do **NOT** extract a shared helper across `inbox.rs` and `publish_trust_attestation.rs` (PRECON-3). The duplication is intentional — `publish_trust_attestation.rs:135-138` doc-comments the circular-dep avoidance (`lemmy_api -> lemmy_apub -> lemmy_apub_activities`). Extracting would either require `pub(crate)` expansion of `inbox.rs` internals or a new shared sub-module; both grow scope beyond "reader-side append-history fix".
- Do **NOT** edit the comment at `publish_trust_attestation.rs:135-138` — it stays verbatim (still accurate post-fix; the duplication still exists; the rationale still holds).
- The error-mapping `.map_err(|_e| LemmyErrorType::Unknown(...))?` and the `val.ok_or_else(...)?` null-handling stay unchanged at both sites.

### 10.2 E2e regression sibling — `per_peer_rate_limit_returns_429`

**Mirror:** `crates/server/tests/e2e.rs:15600-15628` (the sibling test inside `mod v1_federation_inbound_b_fixtures` whose shape most closely matches the override-after-baseline pattern).

Shape:

```rust
#[tokio::test(flavor = "multi_thread")]
async fn per_peer_rate_limit_returns_429() -> LemmyResult<()> {
  let (_container, fed_cfg, db_url, _peer_id) =
    bootstrap_with_peer("rate-test.test", Some(FederationPeerTrust::Allowlisted)).await?;
  let context = fed_cfg.to_request_data();
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  // ... seed override ...
  for i in 0..2 {
    let activity = build_unique_sanction_notice_activity("rate-test.test", i)?;
    ActivityTrait::receive(activity, &context).await?;
  }
  let activity3 = build_unique_sanction_notice_activity("rate-test.test", 2)?;
  let result = ActivityTrait::receive(activity3, &context).await;
  assert!(result.is_err());
  let err = result.err().unwrap();
  assert!(matches!(err.error_type, LemmyErrorType::FederationPeerRateLimitExceeded));
  assert_eq!(err.status_code(), StatusCode::TOO_MANY_REQUESTS);
  Ok(())
}
```

**Apply (Task 3 — new test function inside `mod v1_federation_inbound_b_fixtures`):**

- Test fn signature: `async fn <name>() -> LemmyResult<()>` (Case A per `feedback_lemmy_error_no_std_error.md`).
- `bootstrap_with_peer("override-test.test", Some(FederationPeerTrust::Allowlisted)).await?` (reuse the existing helper at `e2e.rs:15541-15575`).
- `AsyncPgConnection::establish(&db_url).await?` for the seed-override connection (canonical-sibling-mirror; per `feedback_async_pool_test_pattern.md`).
- INSERT a NEWER-`valid_from` `governance_config` row overriding the seeded `federation.inbound.per_peer_rate_per_hour` (seed `value_int = 100`; override to `value_int = 2`). Use the typed Diesel insert via `governance_config::table` + `governance_config::scope/key/value_type/value_int/valid_from` columns (NOT `diesel::sql_query("INSERT ...")` — typed inserts catch column-name drift at compile time).
  - The override row's `valid_from` is set via `governance_config::valid_from.eq(diesel::dsl::now)` (parity with the existing sibling-bootstrap pattern at `e2e.rs:15554-15561` which uses `instance::published_at.eq(diesel::dsl::now)`). `now()` is strictly later than the seed's pinned literal `2026-04-18T00:00:00Z`, so the override is unambiguously the latest row.
- Send 2 unique activities — both must succeed (`?` propagation; both calls return `Ok(())`).
- Send the 3rd unique activity — `assert!(result.is_err())`; `matches!(err.error_type, LemmyErrorType::FederationPeerRateLimitExceeded)`; `err.status_code() == StatusCode::TOO_MANY_REQUESTS`.
- Optional but recommended: assert `federation_inbox_dropped_log` contains exactly 1 row with `drop_reason = "rate_limit_peer"` for `source_instance = "override-test.test"` (parity with the `blocklisted_peer_returns_403` assertion shape at `e2e.rs:15589-15595`).

**Step (b): update the workaround comment at `e2e.rs:15605-15610`** (per DQ #325, option `update-comment-in-task-3`):

Replace the existing 6-line comment block with a single one-line note:

```rust
// governance_config is append-history: UPDATE mutates the existing seed row;
// the post-v1-federation-inbound-c reader (.order_by(valid_from.desc())) makes
// INSERT-with-newer-valid_from also safe. This test uses UPDATE for historical
// continuity; new override tests use INSERT (see <new-test-fn-name>).
```

The existing `diesel::sql_query("UPDATE governance_config SET value_int = 2 ...")` body (lines 15611-15616) stays UNCHANGED — UPDATE still works post-fix (mutates the existing seed row in place; reader returns the only row).

**GOTCHAs:**

- **Test fn name:** the planner does **NOT** pick the exact identifier. The impl-task brief (authored post-plan-approval) commits the name at canonical-schema-first time after the impl agent reads the sibling module and picks a name matching the `<scenario>_<assertion>` convention. Recommended candidates: `appended_config_override_takes_effect_returns_429`, `latest_value_from_wins_over_seeded_baseline`, `appended_per_peer_rate_override_enforces_lower_cap`. The impl brief picks one and updates the workaround-comment reference in step (b) to cite it.
- **Edit budget:** <=200 lines insertion within the sibling module per the brief's invocation of the e2e-edit-hang discipline. Never full-file Edit on `e2e.rs` (15773 lines). The new test is inserted **inside `mod v1_federation_inbound_b_fixtures` between two existing test fns** (recommended anchor: between `per_peer_rate_limit_returns_429` ending at line 15628 and `replayed_activity_returns_409` starting at line 15630). The Edit operation should target a ~30-50 line block (the new test fn body) plus the ~3-line comment update at lines 15605-15610.
- **Error shape (Case A per `feedback_lemmy_error_no_std_error.md`):** test fn returns `LemmyResult<()>`. Every `?` is bare. NO `.map_err(|e| format!("{e}").into())?` bridges. NO `Box<dyn Error>` anywhere. Mirror the sibling verbatim.
- **No transaction wrapper needed:** the seed override is one INSERT; the activity sends are sequential and each commits its own state via the existing `ActivityTrait::receive` path. `conn.run_transaction(...)` is unnecessary unless the planner discovers at sibling-read time that the test setup requires atomic baseline+override (which is not the case here — baseline is migration-seeded; override is a single INSERT).
- **Hour-bucket consideration:** `wrap_governance_inbound` uses `current_hour_bucket()` (`chrono::Utc::now().timestamp() / 3600`) for the in-memory rate-limit map. The test sends 3 activities in fast succession; they will all land in the same hour bucket, which is the assumption the sibling test `per_peer_rate_limit_returns_429` already relies on. The new test inherits the same assumption — no special handling.

## 11. Files to change

Grouped by crate. Each entry: path + one-line purpose + which §13 task(s) write it. The planner-side `cargo metadata` check (per `.claude/agents/planning.md`) verifies each path exists in the workspace before plan commit; manual `ls` confirms each path is present at `governance-v0` HEAD `48ae7249c`.

**`crates/apub/activities/`** (federation activity handlers — governance subdir):

- `crates/apub/activities/src/governance/inbox.rs` — add `.order_by(governance_config::valid_from.desc())` chain in `get_inbound_config_int` between lines 426 and 427 (Task 1).
- `crates/apub/activities/src/governance/publish_trust_attestation.rs` — add same `.order_by(...)` chain in the inline `let actor_cap: i64 = { ... }` block between lines 145 and 146; preserve the helper-duplication comment at 135-138 (Task 2).

**`crates/server/tests/`** (e2e harness):

- `crates/server/tests/e2e.rs` — append one new `#[tokio::test(flavor = "multi_thread")]` test fn inside the existing `mod v1_federation_inbound_b_fixtures` (between lines 15628 and 15630 recommended); update the workaround comment at lines 15605-15610 per DQ #325; insertion budget <=200 lines (Task 3).

**No struct-field add:** none of Tasks 1-3 adds a new field to a public struct. The "Struct-field add: enumerate all callsites" discipline (per `feedback_planner_enumerate_struct_callsites_for_addfield.md`) does not fire for this plan; no `rg "<Type>" crates/ tests/` enumeration required.

**No migrations** (PRECON-5). The e2e test's INSERT of an override row is **test-fixture only** (lives inside the test fn body, not under `migrations/**`).

## 12. NOT building in v1-federation-inbound-c

Per brief §0.1.1 + §0.2. Each entry pairs a "tempting addition" with a deferral pointer; the planner refuses to in-scope any of these mid-impl.

1. **DoS-hardening on `inbox.rs:473` rate-limit map (LRU bounded)** — deferred to v1-federation-inbound-d (or later). Reason: judgment-heavy mitigation choice (in-process LRU vs Postgres-backed counter vs hash-keyed map) warrants its own plan + User Gate 1. Trigger: this sub-phase's retro evaluates whether to in-scope.
2. **DoS-hardening on `publish_trust_attestation.rs:165` raw-string HashMap key (hash mitigation)** — deferred to v1-federation-inbound-d. Reason: same as #1.
3. **TOCTOU eviction fix on `inbox.rs:698`** — deferred to v1-federation-inbound-d. Reason: concurrency design (single-atomic-SQL vs transactional row-lock) warrants its own plan §4 watchpoint + User Gate 1 review.
4. **Phase-6 convention-divergence audit of fed-in-b inbox.rs additions** — SUBSUMED by `brehon-conformance-audit` lane (planning brief at `57ce4c322` on `governance-v0`). Reason: the SKILL + Clippy gates being built there address the same defect class structurally. Trigger: this sub-phase's retro reviews any residual audit gaps after conformance-audit ships.
5. **Helper extraction across `inbox.rs` + `publish_trust_attestation.rs`** — circular-dep constraint stands (per PRECON-3). Reason: extracting requires either `pub(crate)` expansion of `inbox.rs` internals or a new shared sub-module; both grow scope. Trigger: separate refactor sub-phase IF the duplication grows beyond two sites.
6. **Diesel-table registration of `governance_config_current` view in `schema.rs`** — out of scope. Reason: Diesel-typed reads use base table + `.order_by(...)`; registering the view as a typed table is a separate substrate-design decision. Trigger: separate schema-cleanup sub-phase IF more than two readers need the view-shaped query.
7. **Any new migration under `migrations/**` or `crates/db_schema/migrations/**`** — scope violation per PRECON-5. Catch-fire if proposed.
8. **Any Shape G workflow change** — Shape G is SUSPENDED per DQ #229 + PMD `project_shape_g_suspended_2026_05_16`. Reason: the `validate-pending-laptop` DoD applies (per PRECON-6); reactivating Shape G is a cross-cutting workflow change owned by a future re-enable phase. Trigger: 2026-06-01 re-enable check; if Shape G is back, future sub-phases shift back to `kind: "validate-pending"` (workflow_run_id).
9. **A second e2e test asserting Task 2's `publish_trust_attestation` reader** — deferred to retro consideration. Reason: PRECON-4 caps Task 3 at "ONE test, append-history-aware"; Task 2's correctness is enforced by canonical-mirror discipline (clippy + Pattern §10.1 verbatim) plus the workspace cargo check post-edit. Trigger: if a regression class emerges on the trust-attestation path post-merge, add a dedicated test in a future sub-phase.

---

## 13. Step-by-step tasks

Execute in dependency order. One commit per task (per `feedback_pr_per_phase.md` + `.claude/rules/branch-manager.md` "Junior finalize-merge: one commit per task"). Each task header carries a `[P]` marker iff its `union(creates, modifies)` shares no path with any other `[P]`-marked task in the same cohort. Task 0 is always non-`[P]` (barrier).

> **Cohort dispatch:** Tasks 1+2 form one `[P]` cohort (disjoint `modifies` arrays). Task 3 is non-`[P]` (barrier; `requires: [1, 2]`). Task 4 retro is non-`[P]`. Cohort dispatch validation per `.claude/rules/advisor-orchestrator.md` §4.1: pairwise YAML disjointness OK; `requires:` dependency check OK (Task 3's `requires: [1, 2]` blocks until both merged onto `phase-v1-federation-inbound-c`).
>
> **Validate-pending-laptop DoD** (per PRECON-6 + Shape-G suspended): each impl-task pushes its worker branch and writes `kind: "validate-pending-laptop"` to `decision-queue.json` naming §15 commands verbatim. Advisor reads on next poll, runs commands locally on the canonical laptop checkout, mutates the entry (`answered_by: "advisor-laptop"`). Cargo never runs on the EliteDesk Junior daemon (per `.claude/rules/advisor-orchestrator.md` §5.2 "Cargo never runs on the EliteDesk worker").

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment is ready for `v1-federation-inbound-c`; confirm branch is `phase-v1-federation-inbound-c`; confirm fed-in-b deliverables are intact on the phase-branch base; confirm pre-existing clippy baseline is clean against the §15 invocations.

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate ALL probes explicitly).** Run on the **canonical laptop checkout** (`C:/Users/barri/Developer/brehon-fork-fed-in-c` worktree, post-bm-cut) with the laptop wrapper invocations:

```bash
# Probe 0 - Docker daemon (required for Phase-2 e2e)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 - per-crate check honors -p
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/PRPs/debug/v1-federation-inbound-c-task0-probe1.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-c-task0-probe1.log

# Probe 2 - feature flag activation
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/PRPs/debug/v1-federation-inbound-c-task0-probe2.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-c-task0-probe2.log

# Probe 3 - cargo-test wrapper honors target selection
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-federation-inbound-c-task0-probe3.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-c-task0-probe3.log

# Probe 4 - exit-code propagation on bogus feature (negative probe)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/v1-federation-inbound-c-task0-probe4a.log 2>&1"
echo "cargo-test.bat exit on bogus feature: $?"
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/v1-federation-inbound-c-task0-probe4b.log 2>&1"
echo "cargo-check.bat exit on bogus feature: $?"

# Probe 5 - branch verification
git branch --show-current
# EXPECT: phase-v1-federation-inbound-c
git log governance-v0..HEAD --oneline | wc -l
# EXPECT: 0 (phase branch just cut; no commits ahead of trunk yet)

# Probe 6 - fed-in-b deliverables intact on the base
grep -c "^pub const ENTRY_KIND_" crates/db_schema/src/source/governance/governance_log.rs
# EXPECT: 55
grep -n "mod v1_federation_inbound_b_fixtures" crates/server/tests/e2e.rs
# EXPECT: line 15510
grep -n "fn get_inbound_config_int" crates/apub/activities/src/governance/inbox.rs
# EXPECT: line 421
grep -n "let actor_cap: i64 =" crates/apub/activities/src/governance/publish_trust_attestation.rs
# EXPECT: line 139

# Probe 7 - canonical reader pattern is still at the documented line
grep -n "order_by(governance_config::valid_from.desc())" crates/api/api/src/governance/config.rs
# EXPECT: line 757

# Probe 8 - clippy baseline against the §15.2 invocation
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-c-task0-clippy-baseline.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-federation-inbound-c-task0-clippy-baseline.log
```

**EXPECT block (overall):**

- Probes 0..3, 5..8 exit 0.
- Probe 4 exits NON-ZERO (negative test confirms exit-code propagation per `feedback_wrapper_script_flag_silence.md`).
- Probe 6 outputs: `55` + a line `15510:mod v1_federation_inbound_b_fixtures {` + a line near 421 for `get_inbound_config_int` + a line near 139 for `let actor_cap: i64 =`.

**No commit at Task 0** — this is verification only. If any probe fails, file a `kind: "blocker"` DQ (`from: "advisor"`, since Task 0 is run advisor-side as part of plan approval per §15.5 below + `feedback_pre_phase_dod_smoke_test.md`) and STOP. Do NOT dispatch Tasks 1+2 with a red probe.

### Task 1 [P]: Add `.order_by(valid_from.desc())` in `get_inbound_config_int`

**ACTION:** mechanically insert one `.order_by(governance_config::valid_from.desc())` line in `crates/apub/activities/src/governance/inbox.rs` between the existing `.select(governance_config::value_int)` line and the `.first::<Option<i64>>(conn)` line of `get_inbound_config_int`. No other line in the function changes.

**FILES (machine-parseable, used by `/brehon-verify` + cohort dispatch):**

```yaml
creates: []
modifies:
  - crates/apub/activities/src/governance/inbox.rs
requires: []
```

**IMPLEMENT (file 1 of 1):** in `crates/apub/activities/src/governance/inbox.rs`, locate the function `get_inbound_config_int` (currently at line 421). Insert a new line containing `.order_by(governance_config::valid_from.desc())` between the existing `.select(governance_config::value_int)` (currently line 426) and the existing `.first::<Option<i64>>(conn)` (currently line 427), preserving 4-space indentation and the trailing dot of `.select(...)`. Do **NOT** touch the function's doc-comment at lines 418-420; do **NOT** edit the function signature, the filter chain, the error mapping, or the null-handling.

**MIRROR:** `crates/api/api/src/governance/config.rs:740-769` — the canonical reader pattern. The inline comment at lines 741-746 is the verbatim regression-history citation; the impl-task brief at advisor-side dispatch quotes that comment block.

**GOTCHA:** Do NOT introduce `governance_config_current` view usage (the view is not registered in `crates/db_schema_file/src/schema.rs`; Diesel-typed reads must hit the base table + `.order_by(...)`).

**VALIDATE (story-checkpoint feeds §16a Story 1):**

Worker pre-push (Junior `impl-task` worktree):

```bash
git diff --stat
# EXPECT: 1 file changed, 1 insertion(+), 0 deletions(-)
git add crates/apub/activities/src/governance/inbox.rs
git commit -m "feat(fed-in-c): add .order_by(valid_from.desc()) in get_inbound_config_int (task 1)"
git push origin <worker-branch>
# Worker then writes kind: "validate-pending-laptop" DQ entry with §15 commands verbatim and assigned DQ id <id-A> (advisor-reserved per PRECON-7).
```

Post-push (advisor-laptop, on the canonical laptop checkout):

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-c-task1-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-c-task1-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-c-task1-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-c-task1-clippy.log
# EXPECT: exit 0
```

Advisor mutates the `validate-pending-laptop` DQ entry per `.claude/rules/advisor-orchestrator.md` §5.2 (`answered_by: "advisor-laptop"`, `result: "pass"`, move to `resolved[]`) on success.

### Task 2 [P]: Add `.order_by(valid_from.desc())` in `publish_trust_attestation`'s inline `actor_cap` block

**ACTION:** mechanically insert one `.order_by(governance_config::valid_from.desc())` line in `crates/apub/activities/src/governance/publish_trust_attestation.rs` between the existing `.select(governance_config::value_int)` line and the `.first::<Option<i64>>(conn)` line of the inline `let actor_cap: i64 = { ... }` block. No other line changes.

**FILES:**

```yaml
creates: []
modifies:
  - crates/apub/activities/src/governance/publish_trust_attestation.rs
requires: []
```

**IMPLEMENT (file 1 of 1):** in `crates/apub/activities/src/governance/publish_trust_attestation.rs`, locate the inline block `let actor_cap: i64 = { ... }` (currently starting at line 139). Insert a new line containing `.order_by(governance_config::valid_from.desc())` between the existing `.select(governance_config::value_int)` (currently line 145) and the existing `.first::<Option<i64>>(conn)` (currently line 146), preserving 8-space indentation (one level deeper than Task 1's site because this is inside a nested block) and the trailing dot of `.select(...)`.

Do **NOT** edit the helper-duplication comment at lines 135-138 (it remains accurate post-fix). Do **NOT** edit the `subject_url` extraction at lines 123-133 (outside the `actor_cap` block; unrelated). Do **NOT** touch the rate-counter increment at lines 154-166 (Gate 5; unrelated). Do **NOT** extract a shared helper (PRECON-3).

**MIRROR:** `crates/api/api/src/governance/config.rs:740-769` (same canonical pattern as Task 1) + cross-link `inbox.rs:421-438` (the post-Task-1 fix site; Tasks 1+2 mirror each other intra-cohort).

**GOTCHA:** Same as Task 1 — no view usage, no helper extraction, no edit to the comment. **Indentation differs** from Task 1 (8 spaces vs 4 spaces) because the chain is inside an inline block.

**VALIDATE (story-checkpoint feeds §16a Story 1):**

Pre-push + post-push pattern identical to Task 1, with `task2` substituted for `task1` in the log paths and `<id-B>` substituted for `<id-A>` in the DQ entry. Worker commit subject: `feat(fed-in-c): add .order_by(valid_from.desc()) in publish_trust_attestation actor_cap (task 2)`.

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-c-task2-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-c-task2-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-c-task2-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-c-task2-clippy.log
# EXPECT: exit 0
```

### Task 3: E2e regression test in `mod v1_federation_inbound_b_fixtures` + workaround-comment update

**ACTION:** add one `#[tokio::test(flavor = "multi_thread")]` to `crates/server/tests/e2e.rs` inside the existing `mod v1_federation_inbound_b_fixtures` exercising the post-fix append-history override behaviour via `federation.inbound.per_peer_rate_per_hour`. Update the workaround comment at lines 15605-15610 per DQ #325.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
requires:
  - task: 1
    reason: "Task 3's assertion (3rd activity returns 429 under override cap=2) deterministically holds only when get_inbound_config_int reads the override row not the seed. Task 1 is the fix site."
  - task: 2
    reason: "Task 3 does not exercise publish_trust_attestation directly, but the e2e test target compiles the whole workspace including Task 2's edits; cohort barrier is on both fixes being merged before Task 3 dispatches."
```

**IMPLEMENT (file 1 of 1):**

**Step (a)** — Add a new test fn inside `mod v1_federation_inbound_b_fixtures`. Recommended anchor: between the closing brace of `per_peer_rate_limit_returns_429` (currently line 15628) and the `#[tokio::test(flavor = "multi_thread")]` attribute of `replayed_activity_returns_409` (currently line 15630). Test fn body skeleton (the impl agent picks the exact name + adjusts whitespace to match sibling conventions):

```rust
  #[tokio::test(flavor = "multi_thread")]
  async fn <name>() -> LemmyResult<()> {
    let (_container, fed_cfg, db_url, _peer_id) =
      bootstrap_with_peer("override-test.test", Some(FederationPeerTrust::Allowlisted)).await?;
    let context = fed_cfg.to_request_data();
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    // governance_config is append-only — INSERT a newer-valid_from row to
    // override the seeded federation.inbound.per_peer_rate_per_hour (seed
    // value_int = 100 per migration 2026-04-18-000000-0000_add_governance_config).
    // Post-v1-federation-inbound-c, get_inbound_config_int reads the latest row;
    // the override cap=2 means the 3rd activity should 429.
    diesel::insert_into(governance_config::table)
      .values((
        governance_config::scope.eq("instance"),
        governance_config::key.eq("federation.inbound.per_peer_rate_per_hour"),
        governance_config::value_type.eq("int"),
        governance_config::value_int.eq(Some(2_i64)),
        governance_config::valid_from.eq(diesel::dsl::now),
      ))
      .execute(&mut conn)
      .await?;
    for i in 0..2 {
      let activity = build_unique_sanction_notice_activity("override-test.test", i)?;
      ActivityTrait::receive(activity, &context).await?;
    }
    let activity3 = build_unique_sanction_notice_activity("override-test.test", 2)?;
    let result = ActivityTrait::receive(activity3, &context).await;
    assert!(result.is_err(), "3rd activity must 429 against override cap=2");
    let err = result.err().unwrap();
    assert!(matches!(err.error_type, LemmyErrorType::FederationPeerRateLimitExceeded));
    assert_eq!(err.status_code(), StatusCode::TOO_MANY_REQUESTS);
    // Optional but recommended: assert the drop log row landed.
    let drop_rows: i64 = federation_inbox_dropped_log::table
      .filter(federation_inbox_dropped_log::source_instance.eq("override-test.test"))
      .filter(federation_inbox_dropped_log::drop_reason.eq("rate_limit_peer"))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(drop_rows, 1, "exactly one rate_limit_peer drop expected");
    Ok(())
  }
```

**Step (b)** — Replace the existing workaround comment at lines 15605-15610 of `crates/server/tests/e2e.rs` with a one-line post-fix-landscape note (per DQ #325, option `update-comment-in-task-3`). Pre-fix comment (to remove):

```rust
    // governance_config is append-history with UNIQUE on (scope, key, valid_from)
    // - NOT on (scope, key). The migration 2026-05-17 already seeded this key
    // with value_int=100; raw INSERT would create a second row and the reader
    // (get_inbound_config_int) returns an arbitrary one. UPDATE mutates the
    // existing seed row in place. See migration 2026-04-18 comment "Do NOT use
    // (scope, key) as the conflict target" for the schema invariant.
```

Post-fix comment (to insert in its place; the impl agent updates `<new-test-fn-name>` to the chosen name from step (a)):

```rust
    // governance_config is append-history: UPDATE mutates the existing seed row;
    // the post-v1-federation-inbound-c reader (.order_by(valid_from.desc())) makes
    // INSERT-with-newer-valid_from also safe. This test uses UPDATE for historical
    // continuity; new override tests use INSERT (see <new-test-fn-name>).
```

The existing `diesel::sql_query("UPDATE governance_config SET value_int = 2 ...")` body (currently lines 15611-15616 pre-comment-edit) stays UNCHANGED — UPDATE still works post-fix.

**MIRROR:** `crates/server/tests/e2e.rs:15600-15628` (`per_peer_rate_limit_returns_429`) — the canonical sibling shape inside `mod v1_federation_inbound_b_fixtures` whose error-shape (Case A `LemmyResult<()>`), connection acquisition (`AsyncPgConnection::establish`), activity-send loop, and assertion grammar this test mirrors. The impl-task brief at advisor-side dispatch quotes that test verbatim as the §3 Required reading anchor and the §4 Constraints mirror-target.

**GOTCHA:**

- **Edit budget <=200 lines insertion** within the sibling module. Never full-file Edit on `e2e.rs` (15773 lines). The two Edit operations (step (a) new test fn ~30 lines, step (b) comment update ~6 lines net) sit well below budget.
- **Error shape Case A** per `feedback_lemmy_error_no_std_error.md` — `LemmyResult<()>` outer, bare `?` propagation, NO `Box<dyn Error>` bridges, NO `.map_err(|e| format!("{e}").into())?` annotation closures. Mirror sibling verbatim.
- **Connection acquisition** via `AsyncPgConnection::establish(&db_url).await?` per `feedback_async_pool_test_pattern.md` (sibling already imports `diesel_async::AsyncPgConnection` at line 15517; no new use statement needed).
- **No `conn.run_transaction(...)` wrapper** — the override INSERT is a single statement; activity sends are sequential and each commits its own state. `feedback_multi_write_handlers_need_transactions.md` does not fire here.
- **Test fn name discipline:** the planner does not pick the identifier (canonical-schema-first gate is implemented at impl-task brief time). Recommended candidates: `appended_config_override_takes_effect_returns_429`, `latest_value_from_wins_over_seeded_baseline`, `per_peer_rate_override_via_append_history_returns_429`. Whatever name the impl agent picks at sibling-read time updates the workaround-comment reference in step (b) accordingly.
- **R1 (clippy_test_style):** no `i32 as i64` casts. The `2_i64` literal and `0..2` u32 loop counter both fit cleanly without casts; if the impl agent introduces an `i64::from(...)` for any count comparison, that's the canonical form.
- **R6:** clippy invocation at validate uses `--workspace --features full --no-deps -- -D warnings` (per §15.2).
- **Hour-bucket assumption** — the test sends 3 activities synchronously; they will all fall into the same `current_hour_bucket()`. The sibling test `per_peer_rate_limit_returns_429` already relies on this; the new test inherits.

**VALIDATE (story-checkpoint feeds §16a Story 2):**

Pre-push + post-push pattern identical to Tasks 1+2 with `task3` substituted. Worker commit subject: `feat(fed-in-c): add e2e regression for append-history override + update workaround comment (task 3)`.

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-c-task3-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-c-task3-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-c-task3-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-c-task3-clippy.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-federation-inbound-c-task3-test-norun.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-c-task3-test-norun.log
# EXPECT: exit 0 (e2e harness re-links after the test fn insertion)
```

Phase-2 e2e (user gate 4 — local vs dispatch per `feedback_e2e_local_or_dispatch_user_choice.md`; never auto-pick). Default-recommended LOCAL after all 3 tasks finalize-merged onto phase branch:

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-v1-federation-inbound-c-<sha>.log 2>&1 && echo E2E_EXIT_0 >> .claude/runlog/e2e-v1-federation-inbound-c-<sha>.log || echo E2E_EXIT_NONZERO >> .claude/runlog/e2e-v1-federation-inbound-c-<sha>.log"
# run_in_background: true - ~26 min
# EXPECT (tail of log): E2E_EXIT_0; all v1_federation_inbound_b_fixtures tests pass (5 pre-existing + 1 new = 6 passed); pre-existing fed-in-a tests still pass.
```

### Task 4: Retro

**ACTION:** author `.claude/PRPs/reports/v1-federation-inbound-c-retro.md` per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`.

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-federation-inbound-c-retro.md
modifies: []
requires:
  - task: 3
    reason: "Retro signals require Task 3's Phase-2 e2e completion + any CR-fix-in-PR cycles having landed before authorship."
```

**IMPLEMENT:** four H2 sections (Advisor / Planning / Impl / BM) with signals + per-task complexity scores (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`) + lessons promoted this phase + carry-forward items for the next sub-phase. Specific lookback items:

- **Advisor signal:** did the cohort DQ pre-reservation (PRECON-7 Option 3) avoid the id-collision class? Did the retro-bypass JSONL `.claude/governance-log/retro-bypass.jsonl` show monotonic decrease?
- **Planning signal:** did the dogfood at §0 / §5 verifications hold against final HEAD? Was Pattern §10.1 cited correctly by both impl-task briefs?
- **Impl signal:** per-task complexity (`1/1/<runtime>/<silence>` expected per Task 1 + Task 2; `1/1/<longer>/<silence>` expected for Task 3 due to e2e edit + Phase-2 e2e run).
- **BM signal:** did `bm-cut` + `bm-pr` + `bm-merge` flow cleanly? Any CR triage cycles?
- **Carry-forward:** evaluate whether (b) Copilot DoS-hardening family promotes to v1-federation-inbound-d (decision-point per brief §0.1.1 + §0.2 item #1); evaluate whether any residual Phase-6 audit gaps remain post-`brehon-conformance-audit` (brief §0.2 item #4).

**VALIDATE:** retro committed; no failing checkpoints; the lessons-promoted block (if any) lands in the same retro commit (per `feedback_one_system_memory_in_repo.md` — promote inline at retro time).

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** `cargo check --workspace --features full` after every task (§15.1).
- **Lint:** `cargo clippy --workspace --features full --no-deps -- -D warnings` after every task (§15.2).
- **Test target compile:** `cargo test --workspace --features full --test e2e --no-run` after Task 3 (§15.3).
- **e2e execution:** `cargo test --workspace --test e2e --features full` (full run, all tests) — Phase-2 e2e post-finalize-merge (§15.4).
- **Migration round-trip:** N/A (zero migrations per PRECON-5).

**Behavioural assertion (post-fix):** Task 3's new test, when run against an artificial pre-fix codebase (no `.order_by(...)` chain in `get_inbound_config_int`), would fail non-deterministically — Postgres returns either the seed row (cap=100, all 3 activities succeed, `result.is_err()` is false, test fails) or the override row (cap=2, 3rd activity 429s, test passes) depending on planner output. Post-fix, the result is deterministic.

## 15. Validation commands (DoD)

> **Planner-side discipline (per `feedback_plan_dod_dry_run_at_write.md` + `feedback_pre_phase_dod_smoke_test.md`):** every command below MUST be dry-run by the advisor against current HEAD before plan approval (User Gate 1). Unexecutable commands are advisor-side rejection grounds.
>
> **DoD shape:** `validate-pending-laptop` (per PRECON-6 — Shape G SUSPENDED until 2026-06-01, DQ #229 pending re-enable). Each impl-task raises `kind: "validate-pending-laptop"` post-push naming these §15 commands verbatim with `--workspace --features full`. Advisor mutates with `answered_by: "advisor-laptop"`. Cargo runs on the laptop, never on the EliteDesk daemon.

### 15.1 Per-task workspace check (Tasks 1, 2, 3)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-c-task<N>-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-c-task<N>-check.log
```

**EXPECT:** exit 0.

### 15.2 Per-task clippy (Tasks 1, 2, 3 — uniform R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-c-task<N>-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-c-task<N>-clippy.log
```

**EXPECT:** exit 0.

### 15.3 Test target compile (R7 — Task 3 only)

Tasks 1+2 do not touch a struct or re-export; R7 does not fire. Task 3 inserts a new test fn into `e2e.rs` — re-link must succeed:

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-federation-inbound-c-task3-test-norun.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-c-task3-test-norun.log
```

**EXPECT:** exit 0.

### 15.4 Phase-2 e2e (post-finalize-merge — user-gate-4)

**(a) Local laptop bg** (default-recommended; ~26 min, zero billed; per `feedback_windows_e2e_requires_bat_wrapper.md`):

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-v1-federation-inbound-c-<sha>.log 2>&1 && echo E2E_EXIT_0 >> .claude/runlog/e2e-v1-federation-inbound-c-<sha>.log || echo E2E_EXIT_NONZERO >> .claude/runlog/e2e-v1-federation-inbound-c-<sha>.log"
# run_in_background: true
```

**(b) GH dispatch** (escape hatch only; ~26 min billed; per User Gate 4 option (b)):

```bash
gh workflow run cargo-test-e2e.yml --repo barrie-cork/lemmy --ref phase-v1-federation-inbound-c
```

**EXPECT:** all tests pass — 5 pre-existing `v1_federation_inbound_b_fixtures` tests + 1 NEW (the Task 3 override-after-baseline test) + pre-existing fed-in-a tests (no regression). Tail of the log: `E2E_EXIT_0`.

### 15.5 Shape G section — DORMANT until 2026-06-01

**NOT applicable.** Per PRECON-6 + DQ #229. If Shape G re-enables before this sub-phase ships, the planner re-files a `chore(decision-queue)` advisor entry switching the §15 shape to Shape G workflow references; no plan re-author needed (forward-only retrofit per `feedback_schema_changing_spec_retrofit_question.md`).

### 15.6 Cross-cutting verification

The planner asserts each box holds at end-of-phase:

- [ ] `grep -n "order_by(governance_config::valid_from.desc())" crates/apub/activities/src/governance/inbox.rs` returns exactly 1 match (inside `get_inbound_config_int`).
- [ ] `grep -n "order_by(governance_config::valid_from.desc())" crates/apub/activities/src/governance/publish_trust_attestation.rs` returns exactly 1 match (inside the `actor_cap` block).
- [ ] `grep -n "fn get_inbound_config_int" crates/apub/activities/src/governance/inbox.rs` still returns the helper at line 421 (signature unchanged; no symbol drift).
- [ ] `grep -n "Mirrors the .get_inbound_config_int. helper in inbox.rs" crates/apub/activities/src/governance/publish_trust_attestation.rs` returns the comment at line 135 (PRECON-3 honoured; comment untouched).
- [ ] `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` returns **55** (no kind additions/deletions; this sub-phase does not touch the registry).
- [ ] `rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l` returns **55** (shim parity unchanged).
- [ ] `grep -n "mod v1_federation_inbound_b_fixtures" crates/server/tests/e2e.rs` returns the existing match (~line 15510; sibling module retained).
- [ ] The new test fn appears between `per_peer_rate_limit_returns_429` and `replayed_activity_returns_409` inside `mod v1_federation_inbound_b_fixtures` (or another valid anchor inside the module — the impl agent's pick).
- [ ] The workaround comment at the location of the previous `lines 15605-15610` workaround block now reads the post-fix landscape note (per DQ #325).
- [ ] No edit to `crates/db_schema/src/source/governance/governance_config.rs` (model file untouched).
- [ ] No edit to `migrations/**` or `crates/db_schema/migrations/**` (PRECON-5).
- [ ] No new file under `crates/apub/activities/src/governance/` (PRECON-3; helper not extracted).
- [ ] R1: no `i32 as i64` casts in new code (Task 3 uses `i64` literal `2_i64`; `0..2` u32 counter does not need a cast for the activity loop).
- [ ] R6: every §15.2 invocation uses `--no-deps -- -D warnings`.
- [ ] R7: Task 3 ran `cargo test --no-run` (§15.3).
- [ ] PRECON-1 (scope (a) only) honoured: no DoS-hardening edits, no convention-audit edits, no helper extraction.
- [ ] PRECON-2 (canonical-mirror) honoured: both fix sites cite `config.rs:740-769`.
- [ ] PRECON-3 (no helper extraction) honoured.
- [ ] PRECON-4 (one e2e test, append-history-aware) honoured.
- [ ] PRECON-5 (zero migrations) honoured.
- [ ] PRECON-6 (validate-pending-laptop DoD) honoured.
- [ ] PRECON-7 (cohort DQ pre-reservation) honoured: advisor reserved `<id-A>` and `<id-B>` before Tasks 1+2 dispatch; each impl brief named its assigned id in §4 Constraints.
- [ ] PRECON-8 (review-point sequencing) honoured: brief approved -> planning dispatched -> plan approved -> bm-cut -> cohort -> Task 3 -> bm-pr -> bm-merge.
- [ ] DQ #324 honoured (NEW tests use INSERT-with-newer-`valid_from`; the new Task 3 test does so).
- [ ] DQ #325 honoured (workaround comment updated per option `update-comment-in-task-3`).

### 15.7 ADR / OQ compliance

- [ ] **ADR-006** (advisory-only inbound persistence): unchanged. The fix is reader-side; no row-shape change; `local_case_id` semantics untouched.
- [ ] **ADR-013** (illegal content / `CaseStatus::EmergencyRemove`): not in code path; unchanged.
- [ ] **ADR-014** (governance signals are fork-only AP types): unchanged. Wrapper still fires only on governance AP types; vanilla Lemmy unaffected.
- [ ] **ADR-015** (pseudonymisation): unchanged. No new TEXT column; no raw-id surface.
- [ ] **Append-only contract** (`crates/db_schema/src/source/governance/governance_config.rs:34-40` + `migrations/2026-04-18-...up.sql:34-40`): preserved. Reader now correctly surfaces the latest row per `(scope, key)` — the design contract the model file already documents.

---

## 16. Acceptance criteria

Roll-up of §15 + §16a story checkpoints. The planner asserts each box is ticked at end-of-phase. The advisor's `/brehon-verify` cross-checks each box against the worktree branch before queueing `bm-merge`.

- [ ] All 5 tasks (Task 0 + 4 work tasks) completed in dependency order.
- [ ] §15.1 (cargo check) exit 0 after Tasks 1, 2, 3.
- [ ] §15.2 (cargo clippy `--no-deps -- -D warnings`) exit 0 after Tasks 1, 2, 3.
- [ ] §15.3 (cargo test --no-run) exit 0 after Task 3.
- [ ] §15.4 (e2e tests) — 1 new test passes; 5 pre-existing `v1_federation_inbound_b_fixtures` tests still pass; pre-existing fed-in-a tests still pass.
- [ ] §15.5 N/A (Shape G dormant).
- [ ] §15.6 cross-cutting verification — all boxes ticked.
- [ ] §15.7 ADR/OQ compliance — all boxes ticked.
- [ ] §16a stories — all stories `[done]`.
- [ ] No edits to files outside §11 list.
- [ ] Retro committed per §13 Task 4.
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`.

---

## 16a. Stories (independently-testable behaviour units)

> Per `feedback_brehon_verify_pre_merge.md`. Each story names its composing §13 tasks + a checkpoint command and Brief-Scope outputs `/brehon-verify` confirms.

### Story 1: Reader-side append-history fix lands at both call sites

- **User-facing behaviour:** an admin-edit INSERT-with-newer-`valid_from` on `federation.inbound.*` config keys takes effect on the next inbound activity (per-peer rate, per-actor attestation rate, per-type payload size cap).
- **Composing tasks:** Task 1, Task 2 (both `[P]`; share zero IMPLEMENT files).
- **Checkpoint command (laptop):** `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"` against Task 2's worker-branch (or the post-finalize-merge phase-branch tip — whichever the advisor checks at story-completion time). EXPECT exit 0.
- **Brief-Scope outputs to verify:**
  - `crates/apub/activities/src/governance/inbox.rs` contains exactly one `.order_by(governance_config::valid_from.desc())` line inside `get_inbound_config_int` (between `.select(governance_config::value_int)` and `.first::<Option<i64>>(conn)`).
  - `crates/apub/activities/src/governance/publish_trust_attestation.rs` contains exactly one `.order_by(governance_config::valid_from.desc())` line inside the `actor_cap` inline block.
  - The helper-duplication comment at `publish_trust_attestation.rs:135-138` is unchanged.
  - The function signature of `get_inbound_config_int` (line 421) is unchanged.

### Story 2: E2e regression test asserts the post-fix override behaviour end-to-end

- **User-facing behaviour:** a federation-inbound activity exceeding the admin-override per-peer rate cap returns 429 (and writes a `federation_inbox_dropped_log` row + `governance_log` entry kind `federation_inbound_dropped_rate_limit_peer`), even when the original seed cap would have admitted the activity.
- **Composing tasks:** Task 3 (barrier; `requires: [1, 2]`).
- **Checkpoint command (laptop, Phase-2 e2e):** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1"` with `run_in_background: true`. EXPECT `E2E_EXIT_0` in the log tail.
- **Brief-Scope outputs to verify:**
  - `crates/server/tests/e2e.rs` contains one new `#[tokio::test(flavor = "multi_thread")]` test fn inside `mod v1_federation_inbound_b_fixtures` exercising the override-after-baseline pattern.
  - The new test fn returns `LemmyResult<()>` (Case A per `feedback_lemmy_error_no_std_error.md`).
  - The new test fn uses `AsyncPgConnection::establish` + `diesel::insert_into(governance_config::table).values(...)` with `valid_from.eq(diesel::dsl::now)`.
  - The new test asserts `LemmyErrorType::FederationPeerRateLimitExceeded` and `StatusCode::TOO_MANY_REQUESTS` for the 3rd activity.
  - The workaround comment near the previous `lines 15605-15610` block has been replaced with the post-fix landscape note (DQ #325).
  - The existing `per_peer_rate_limit_returns_429` test body (lines 15600-15628 pre-edit) is unchanged.

### Story 3: Retro captures four-role signals + complexity scores + carry-forward

- **User-facing behaviour:** `bm-merge` runs only after retro authorship; user gate 6 (retro sign-off) gates `/brehon-phase-transition`.
- **Composing tasks:** Task 4 (non-`[P]`; `requires: [3]`).
- **Checkpoint command:** `test -f .claude/PRPs/reports/v1-federation-inbound-c-retro.md && grep -c '^## ' .claude/PRPs/reports/v1-federation-inbound-c-retro.md` returns >= 4 (one H2 per role).
- **Brief-Scope outputs to verify:**
  - The retro file exists at the canonical path.
  - Four H2 sections present: Advisor / Planning / Impl / BM.
  - Per-task complexity score block present (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`) per `feedback_retro_task_complexity_score.md`.
  - Carry-forward block names the v1-federation-inbound-d decision-point (b family in/out) + any residual conformance-audit gap evaluation.
  - Lessons promoted this phase (if any) land in the same retro commit body, paths under `.claude/lessons/feedback_*.md`.

> **Verification mapping:** `/brehon-verify` iterates this section per `.claude/commands/brehon-verify.md`, runs each Story's Checkpoint against the worktree branch, and confirms each Brief-Scope output exists + matches its structural pattern. Phantoms (task complete but output absent or empty) trigger the catch-fire procedure in advisor-orchestrator.md.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (all 9 probe families confirmed).
- [ ] Tasks 1, 2, 3 committed in dependency order (Tasks 1+2 cohort dispatched simultaneously; Task 3 dispatched after both Tasks 1+2 finalize-merged onto `phase-v1-federation-inbound-c`).
- [ ] Task 4 retro committed.
- [ ] §15 validation green at every gate.
- [ ] §16a stories all `[done]`.
- [ ] Retro committed.
- [ ] PR opened by BM session against `governance-v0` with `--repo barrie-cork/lemmy`.
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`.
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-federation-inbound-c-verify.md` shows all stories OK.
- [ ] DQ #324 / DQ #325 honoured.
- [ ] PRECON-1 / PRECON-2 / PRECON-3 / PRECON-4 / PRECON-5 / PRECON-6 / PRECON-7 / PRECON-8 honoured.
- [ ] Post-merge phase branch retained for retro reads.

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Impl agent paraphrases the canonical reader pattern (e.g. uses `value_text` or a different filter chain) | LOW | MEDIUM | §10.1 quotes the verbatim canonical chain; both impl-task briefs cite `config.rs:740-769` as MIRROR with line range; mechanical one-line insertion; pre-push `cargo-check` catches drift. |
| Impl agent extracts a shared helper across the two files (PRECON-3 violation) | LOW | HIGH | PRECON-3 in plan §0/§12; impl brief §4 Constraints forbids extraction; canonical-schema-first gate (read sibling first) reinforces. |
| Impl agent introduces `governance_config_current` view via raw SQL (PRECON-2 violation) | LOW | MEDIUM | PRECON-2 in plan §0/§10.1 GOTCHAs; brief §4 Constraints forbids view usage; Task-0 Probe verifies the view is not Diesel-registered. |
| Impl agent does full-file Edit on `e2e.rs` and worker hangs | LOW | HIGH | Brief §4 Constraints + plan §10.2 GOTCHA cap insertion at <=200 lines; sibling-module insertion is bracketed between two existing test fns at known line numbers. |
| Case-shape drift on Task 3 (Case B or Case C per `feedback_lemmy_error_no_std_error.md`) | LOW | HIGH | §10.2 mirrors the sibling verbatim (Case A); §4 Constraints forbids `.map_err(\|e\| ... .into())?` bridges; the §G4 4c row catches Case C drift and forces re-plan rather than auto-fix. |
| Cohort DQ id collision on Tasks 1+2 dispatch | MEDIUM (without mitigation) | LOW | PRECON-7 Option 3: advisor pre-reserves `<id-A>` + `<id-B>` in `pending[]` BEFORE cohort dispatch; each impl brief names its assigned id. Per `feedback_cohort_dq_id_collision.md`. |
| Phase-2 e2e test exhibits flake due to in-memory rate-limit map state from prior test | LOW | MEDIUM | The new test uses a fresh peer domain (`"override-test.test"`) — distinct from `"rate-test.test"` used by the sibling — so the `(peer_domain, hour_bucket)` map keys don't collide. testcontainers spin up fresh Postgres + the rate-limit OnceLock is per-process. If flake emerges, that's a watchpoint for the next sub-phase. |
| Advisor-side §3.4 DoD smoke fails at plan-approval time (e.g. Probe 8 baseline clippy regresses against current HEAD) | LOW | HIGH | Task 0 probes already capture the baseline; if Probe 8 fails before Task 1 dispatch, advisor files DQ pending and requests planner narrowing OR a pre-phase `chore(lint)` commit before Task 1 — per `feedback_pre_phase_dod_smoke_test.md`. |
| Shape G re-enables mid-sub-phase (2026-06-01 boundary crossed) | LOW | LOW | PRECON-6 carries a forward reminder. If crossed, advisor switches the in-flight `kind` from `validate-pending-laptop` to `validate-pending` at next impl-task dispatch; mechanical, no plan re-author. |
| Junior worker forks from stale base (pre-bm-cut tip) and misses fed-in-b's `mod v1_federation_inbound_b_fixtures` | LOW | HIGH | bm-cut creates `phase-v1-federation-inbound-c` off `governance-v0` post-PR-#139-merge (`16ede83b6`); Probe 6 at Task 0 verifies the sibling module exists on the cut branch; per `feedback_junior_292_stale_base_recover_recipe.md` for recovery if drift detected post-dispatch. |

---

## 19. Notes

- **Scope (a) only** — this plan deliberately excludes the (b) Copilot DoS-hardening family and (c) Phase-6 convention audit per advisor handover 2026-05-21. The brief's §0.1.1 + §0.2 enumerate the deferral pointers; this plan's §12 mirrors them. The user reviews this scope at User Gate 1 along with §15 DoD smoke + §4 watchpoint specificity gate + §5 dogfood gate per `.claude/rules/advisor-orchestrator.md` §3.

- **Cohort DQ pre-reservation (PRECON-7)** — between plan approval and Tasks 1+2 dispatch, the advisor allocates two new pending `validate-pending-laptop` DQ stubs (`<id-A>` for Task 1, `<id-B>` for Task 2) per `feedback_cohort_dq_id_collision.md` Option 3. Each stub's `answer` field reads `"reserved for v1-federation-inbound-c cohort 1 task <N>"`. Each impl-task brief authored at advisor-side dispatch names its assigned id in §4 Constraints. After both workers push, the advisor reads each entry's mutating fields (workflow_run_id N/A under validate-pending-laptop; the entry instead carries `commands[]` per the laptop DoD shape) and runs §15 commands on the canonical laptop checkout, then mutates the entry to `result: "pass"` / `result: "fail"` accordingly.

- **PRECON enumeration (8 items)** — recorded in §6 + §12 of this plan and consumed by §15.6 cross-cutting verification. The brief §0.1 already authoritatively states them; the plan reproduces them in actionable form (per-PRECON checkbox under §15.6).

- **Open questions filed as DQ pre-seeds** — none. The brief's §0.1 closes the 8 PRECONs in advance; no `kind: "blocker"` DQ entries are pre-seeded from this plan. The planning subagent does NOT raise additional clarifies (per `.claude/rules/advisor-orchestrator.md` §3.3 — clarify is advisor-only at pre-planning, and the two clarifies #324 + #325 already resolved cover the brief's e2e-test design questions).

- **Lessons promoted this phase (anticipated)** — none required by current plan. If retro signals (Task 4) surface a new pattern not captured in existing lessons, the retro commit body promotes it under `.claude/lessons/feedback_*.md` per `feedback_one_system_memory_in_repo.md`.

- **Alternative approaches considered (rejected at brief-author time)** —
  1. **Helper extraction across the two files** (rejected per PRECON-3; circular-dep avoidance + scope-creep).
  2. **Switch to raw SQL via `governance_config_current` view** (rejected per PRECON-2; view not Diesel-registered, and the Diesel-typed chain is the canonical pattern elsewhere).
  3. **Add `cargo test` execution at every task** (rejected; `cargo test --no-run` at Task 3 is sufficient — e2e execution is the Phase-2 gate per User Gate 4; running ~26 min after every task is wasteful).
  4. **Author two e2e tests (one per fix site)** (rejected per PRECON-4; one test on the per-peer rate path exercises the canonical reader behaviour; the trust-attestation reader is covered by cargo check + clippy + canonical-mirror discipline + Pattern §10.1 verbatim).

---

## 20. Confidence score

- **Plan correctness:** 9/10 — defect verified at brief author time (§5 dogfood), canonical pattern verified at brief author time (`config.rs:741-757`), sibling module verified at brief author time (`e2e.rs:15510-15773`), schema invariant verified at brief author time (`grep -c "governance_config_current" crates/db_schema_file/src/schema.rs` = 0). The fix is mechanical and one-line per site.
- **Cargo budget:** 10/10 — N/A (validate-pending-laptop runs on laptop; budget non-binding).
- **Test coverage:** 8/10 — one e2e test asserts the override-after-baseline behaviour end-to-end via the per-peer-rate path. Task 2's `publish_trust_attestation` reader correctness is enforced by clippy + canonical-mirror discipline rather than a dedicated e2e (PRECON-4 cap). If a regression class emerges on the trust-attestation path post-merge, a follow-up sub-phase adds a dedicated test; flagged in §12 item #9.
