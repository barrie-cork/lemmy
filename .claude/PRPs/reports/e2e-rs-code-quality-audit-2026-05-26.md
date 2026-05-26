# `crates/server/tests/e2e.rs` — Code-quality audit (governance-v0)

- Target SHA context: `6ebd31599` (2026-05-23)
- File size verified: **17,099 lines / 629 KB** (matches prompt structural map)
- Scan mode: static, read-only, no clippy invocation
- Fetch: `gh api repos/barrie-cork/lemmy/contents/...` (raw URL returned 404 without token; token-URL succeeded)

## Coverage of the file

| Line range | Analysed |
|---|---|
| 1–17099 | **Yes** — full file scanned line-by-line for every axis 1–10 |
| Bodies of every `#[tokio::test]` and helper fn | Yes (signature + return type + content within ≤30-line windows around finding sites) |
| Bodies of large round-trip migration tests (1191–1915) | Yes (sigs + `sql_query(format!)` audit + ignored-test attribute) |
| Macros (`enum_map!`, `serde_json::json!`) | Sampled — confirmed false-positive class for arg-count heuristic |
| `crates/server/Cargo.toml` + workspace `[workspace.lints.clippy]` | Cross-checked to confirm `unwrap_used=deny`, `expect_used=deny`, `allow_attributes=deny` apply to test files |
| **Not analysed** | Macro-expanded code, sibling modules outside this file, runtime behaviour, panics observed at test time |

## Summary table — counts by axis × severity

| Axis | CRITICAL | MAJOR | MEDIUM | LOW | NIT | Total |
|---|---|---|---|---|---|---|
| 1 — Sibling-module shape consistency | 0 | 0 | 0 | 1 | 0 | 1 |
| 2 — Clippy compliance forecast | 0 | 59 | 0 | 0 | 0 | 59 |
| 3 — Test isolation / idempotency | 0 | 7 | 0 | 1 | 0 | 8 |
| 4 — SQL injection / unsafe construction | 0 | 0 | 0 | 2 | 0 | 2 |
| 5 — HTTP assertion completeness | 0 | 1 | 0 | 1 | 0 | 2 |
| 6 — Magic numbers / threshold hardcoding | 0 | 2 | 0 | 0 | 0 | 2 |
| 7 — Cross-module dependency leakage | 0 | 3 | 0 | 0 | 0 | 3 |
| 8 — Dead / disabled tests | 0 | 0 | 0 | 0 | 0 | 0 |
| 9 — Performance smells | 0 | 0 | 0 | 1 | 0 | 1 |
| 10 — Sub-phase coverage gaps | 0 | 4 | 0 | 8 | 0 | 12 |
| **Totals** | **0** | **76** | **0** | **14** | **0** | **90** |

## Top 5 fixes to land first

1. **Replace 9 raw `.unwrap()` calls in test bodies with `?` propagation or `expect(reason=…)`** — lines 14867, 14885, 15118, 15219, 15384, 16265, 16305, 16345, 16382. Each is JSON-payload `.as_str().unwrap()` or `Result::err().unwrap()` inside a `LemmyResult<()>` test fn — convert to `.context("…")?` / `ok_or_else(...)?` chains. **Mechanical (~10 min)**, eliminates the entire unwrap_used violation surface.
2. **Convert env-mutation tests at 10845/11074/12799/12930/13118/13214/13373 from inline restore to a `Drop` guard** — restore today runs only on the success path; any `?` between set_var and the restore leaks `BREHON_DISABLE_*_JOB=1` to subsequent tests under `--test-threads=1`. Introduce one `EnvVarGuard` struct with a `Drop` impl returning to `prev_*` state. **Substantive (~1–2 h)** — fixture work + 7 call-site rewrites.
3. **Remove `super::v1_jm_b_fixtures::*` reach-overs from `v1_sl_d_fixtures` (4 sites) and `v1_sl_e_fixtures` (2 sites)** — lines 13664, 13903, 14097, 14275, 14632, 14956. Either move `seed_jury_eligible_snapshots` into `governance_fixtures` (the canonical cross-module home) or duplicate the seeder per module. Cross-module reach-over violates the "governance_fixtures calls are allowed" rule and couples SL-d/e to JM-b's helper surface. **Substantive (~1 h)** — one helper migration + 6 call-site edits.
4. **Reword the 50+ `.expect("…")` calls in test bodies to use `expect_used`-allowed `?` propagation** — see Axis 2 finding list (lines 2153–15378). Bulk pattern: `Option::expect(msg)` → `.ok_or_else(|| LemmyErrorType::Unknown(msg.into()))?`. **Substantive (~3 h)** — high-volume mechanical edit, but each site reviewed for context.
5. **Add status-code assertions to the v4 endpoint sweep at 4146–4209** — current shape asserts membership in an allowlist `&[200, 400, 401]` but does not assert any specific endpoint returns 200 on a valid payload (only non-404). Add at least one happy-path probe per route family (report/endorsement/appeal/jury/admin) with a fully-formed payload + `assert_eq!(status, 200)`. **Substantive (~2 h)** — payload construction per route.

---

## Axis 1 — Sibling-module shape consistency

All 13 confirmed modules conform to the canonical shape: test fns return `LemmyResult<()>` (Case A), helpers return `LemmyResult<T>`, no `Box<dyn Error>` outside the `pg_template` block (governance_fixtures lines 288–735 — deliberate FFI boundary code with explicit closure-typed `.map_err`).

| Module | Start line | Outer test result | Helper result | Tokio flavor | Idiom drift notes |
|---|---|---|---|---|---|
| `governance_fixtures` | 115 | n/a (helpers only) | `LemmyResult<T>` / `Result<…, Box<dyn Error>>` for pg_template internals | n/a | `pg_template` submodule uses `Box<dyn Error>` deliberately (process-exec / docker boundary). Documented at L288–735. Consistent. |
| `admin_config_fixtures` | 6100 | tests use `lemmy_utils::error::LemmyResult<()>` | `LemmyResult<T>` | mostly `multi_thread` | Consistent. |
| `v1_jm_b_fixtures` | 8535 | `LemmyResult<()>` | `LemmyResult<T>` | `multi_thread` | Consistent. `seed_case` async with `unused_async` `#[expect]` at 8555 (sync Diesel body). |
| `v1_jm_e_fixtures` | 10454 | `LemmyResult<()>` | `LemmyResult<T>` | `multi_thread` | Consistent; **but reaches into `super::v1_jm_b_fixtures::*` (see Axis 7)** |
| `v1_sl_b_fixtures` | 11658 | `LemmyResult<()>` | `LemmyResult<T>` | bare `#[tokio::test]` (no flavor) at L11833, 11970, 12047, 12138, 12209, 12272, 12405, 12511, 12600 | Consistent error-shape. Bare flavor is allowed if tests don't need federation. |
| `v1_sl_c_fixtures` | 12689 | `LemmyResult<()>` | `LemmyResult<T>` | bare `#[tokio::test]` (12794, 12904, 13095, 13203, 13336) | Consistent. |
| `v1_sl_d_fixtures` | 13567 | `LemmyResult<()>` | `LemmyResult<T>` | bare `#[tokio::test]` (13623, 13865, 14058, 14252) | Consistent error-shape. Cross-module reach (Axis 7). |
| `v1_sl_e_fixtures` | 14426 | `LemmyResult<()>` | `LemmyResult<T>` | bare `#[tokio::test]` (14595, 14920, 15136) | Consistent error-shape. Cross-module reach (Axis 7). |
| `v1_federation_inbound_a_fixtures` | 15393 | `LemmyResult<()>` | `LemmyResult<T>` | `multi_thread` (15431, 15446) | Consistent. |
| `v1_ship_2_fixtures` | 15456 | `LemmyResult<()>` | `LemmyResult<T>` | `multi_thread` | Consistent. |
| `v1_federation_inbound_b_fixtures` | 16186 | `LemmyResult<()>` | `LemmyResult<T>` | `multi_thread` | Consistent. Uses `.unwrap()` on JSON `as_str()` (Axis 2). |
| `v1_federation_inbound_e_fixtures` | 16518 | `LemmyResult<()>` | `LemmyResult<T>` | `multi_thread` | Consistent. |
| `v1_ship_3_fixtures` | 16699 | `LemmyResult<()>` | `LemmyResult<T>` | `multi_thread` | Consistent. Has nested `#[expect(clippy::too_many_arguments)]` (16903) mirroring the canonical pattern from `sponsor_liability_with_founder_multiplier` (3516). |

### Findings (Axis 1)

- [LOW] line:11658-15392 — `v1_sl_*` modules (SL-b/c/d/e) default to bare `#[tokio::test]` while `v1_jm_*`, `v1_ship_*`, and `v1_federation_inbound_*` default to `#[tokio::test(flavor = "multi_thread")]`. SL tests do not exercise federation context (verified — no `FederationConfig::builder()` inside their bodies), so bare flavor is legal under the constraint, but the inconsistency means a later SL test that adds federation will silently inherit the wrong flavor — recommend module-internal convention header or `#[tokio::test(flavor = "multi_thread")]` everywhere.

---

## Axis 2 — Clippy compliance forecast

Workspace `[workspace.lints.clippy]` (verified by reading `Cargo.toml` on `governance-v0`): `unwrap_used = "deny"`, `expect_used = "deny"`, `allow_attributes = "deny"`. The lints inherit into `crates/server` (which sets `[lints]` workspace inheritance — the audit spot-checked `crates/server/Cargo.toml`). **Every `.unwrap()` and `.expect(…)` in this file will trigger a clippy denial unless covered by `#[expect(clippy::unwrap_used, reason=…)]`**. None of the existing `#[expect]` attributes in this file cover `unwrap_used` or `expect_used` (audited every `#[expect]` at lines 1930, 3275, 3516, 8555, 9748, 9937, 10620, 16903 — all cover `dead_code`, `too_many_arguments`, `too_many_lines`, `unused_async`, or `type_complexity`).

### `#[allow(...)]` survey

- Zero `#[allow(...)]` attributes in the file (verified by `grep '#\[allow('`). Compliant with `allow_attributes = "deny"`.

### `#[expect(...)]` survey

8 sites, all with `reason = "…"`. All reasons present and non-empty:

| Line | Lint | Reason text |
|---|---|---|
| 1930 | `dead_code` | "struct fields accessed via Diesel QueryableByName reflection" |
| 3275 | `clippy::too_many_lines` | "3-branch e2e per plan §11.5" |
| 3516 | `clippy::too_many_arguments` | "integration test helper orchestrates a full sanction round; all parameters are required" |
| 8555 | `clippy::unused_async` | "callers .await this; body uses sync Diesel but signature must be async for call-site consistency" |
| 9748 | `clippy::type_complexity` | "Diesel tuple query; local variable type annotation required for inference" |
| 9937 | `clippy::type_complexity` | "Diesel tuple query; local variable type annotation required for inference" |
| 10620 | `clippy::too_many_arguments` | "integration test fixture requires all caller-side inputs; no natural grouping" |
| 16903 | `clippy::too_many_arguments` | "integration test helper orchestrates a full sanction round; all parameters are required" |

### Findings (Axis 2)

`.unwrap()` (test body, MAJOR — direct `unwrap_used` violation):

- [MAJOR] line:14867 — `endo_payload["revoker_pseudonym"].as_str().unwrap()` in `revocation_during_window_escapes_full_lane` (test body) — replace with `.as_str().ok_or_else(|| LemmyErrorType::Unknown("expected string".into()))?`.
- [MAJOR] line:14885 — `escaped_payload["reason"].as_str().unwrap()` — same fix.
- [MAJOR] line:15118 — `fired_payload["target_pseudonym"].as_str().unwrap()` in `window_expiry_fires_full_lane` — same fix.
- [MAJOR] line:15219 — `pre_decided_at.unwrap() > Utc::now() - Duration::hours(24)` in `backfill_of_mid_flight_v0_to_v1_deploy` — replace with `.ok_or_else(...)?`.
- [MAJOR] line:15384 — `applied_payload["sponsor_pseudonym"].as_str().unwrap()` — same fix.
- [MAJOR] line:16265 — `result.err().unwrap()` in `blocklisted_peer_returns_403` — refactor as `match result { Ok(_) => panic!("expected error"), Err(e) => assert_eq!(e.status_code(), …) }` or use `.expect_err("...")` (still deniable) → preferred: assert via pattern match.
- [MAJOR] line:16305 — `result.err().unwrap()` in `per_peer_rate_limit_returns_429` — same fix.
- [MAJOR] line:16345 — `result.err().unwrap()` in `appended_config_override_takes_effect_returns_429` — same fix.
- [MAJOR] line:16382 — `result.err().unwrap()` in `replayed_activity_returns_409` — same fix.

`.expect(...)` (test body / helper, MAJOR — direct `expect_used` violation, 50 sites). First 20 listed:

- [MAJOR] line:1138 — `entry.expect("readable migrations dir entry")` in migrations-list helper (governance_fixtures) — replace with `?` + `LemmyErrorType` wrap.
- [MAJOR] line:1141 — `expect("entry file_type")` — same fix.
- [MAJOR] line:1536, 1867 — `u64::try_from(...).expect("len fits u64")` × 2 — known-safe coercion; replace with `let limit = u64::try_from(MIGRATIONS_TO_REVERT_PHASE_1.len()).map_err(|_| LemmyErrorType::Unknown("len fits u64".into()))?;`
- [MAJOR] line:2153, 2154, 2158, 2168 — `.expect("Row A (Decided) should be present")` etc. inside backfill assertion block — convert to `ok_or_else(...)?` chain.
- [MAJOR] line:2666 — `create_resp.case_id.expect("case_id present")` — Option unwrap on response; same fix.
- [MAJOR] line:2875, 2877, 2917 — golden-path date/rationale unwraps — same fix.
- [MAJOR] line:3198 — `i64::try_from(...).expect("count fits i64")` — same fix.
- [MAJOR] line:4716 — `Regex::new(pat).expect("valid regex")` inside `.map(|(label, pat)| ...)` — replace regex compilation with `lazy_static`/`OnceLock` + LemmyResult propagation, or `Regex::new(pat).map_err(|_| LemmyErrorType::Unknown("valid regex".into()))?`.
- [MAJOR] line:6044 — `.expect("GH #33: replacement must be selected (6th eligible is available)")` — Option assertion in test body; convert.
- [MAJOR] line:7743 — `resp.case_id.expect("case_id present on successful open")`.
- [MAJOR] line:7761, 7764 — `applied_config_snapshot` chain unwraps.
- [MAJOR] line:9585, 9740, 10929 — `final_resp = last_resp.expect("at least one vote cast")` × 3 — pattern repeats; canonical fix is `.ok_or_else(|| LemmyErrorType::Unknown("at least one vote cast".into()))?`.
- [MAJOR] line:9957, 9958, 10088, 10089, 10257, 10258, 11387, 11388 — `decided_at`/`appeal_window_expires_at` `Option::expect` × 8 — same canonical fix.
- _Additional 25 similar `.expect(...)` findings omitted; cite line range 10717–15378 — same MAJOR severity; same canonical fix (Option/Result `expect` → `ok_or_else(...)?` propagation)._

`unreachable!()` / `panic!(...)` in test bodies (LOW — controlled exits but `expect_used`-equivalent surface):

- [MAJOR] line:1136 — `.unwrap_or_else(|e| panic!("cannot read migrations dir {migrations_dir}: {e}"))` — helper-level; replace with `?` and `LemmyErrorType::Unknown`.
- [MAJOR] line:4219 — `_ => unreachable!()` inside method-string match in `all_mvp_endpoints_return_non_404` — bounded to the local `match *method` over `&["GET", "POST"]` so logically safe, but a `_ => unreachable!("method must be GET or POST")` with reason text would be stricter and survives any future test-table edit.
- [MAJOR] line:7361, 7364, 7378, 7385, 8336 — explicit `panic!("...")` in assertion arms — these are intentional test-failure paths; replace with `assert!(false, "...")` is no better, but a `return Err(LemmyErrorType::Unknown("...".into()).into())` propagation is consistent with `LemmyResult<()>` and avoids unwinding aesthetics. Lower priority — these don't fail clippy lints today.

Boolean comparisons (`== true` / `== false`): **none found** (compliant).

Functions with >7 arguments (true positives only — `enum_map!`/`json!`/SQL-literal blocks excluded):

- [INFO] line:3520 — `run_sanction_scenario` (9 args). Covered by `#[expect(clippy::too_many_arguments, …)]` at 3516.
- [INFO] line:16907 — `run_sanction_scenario` (duplicate in `v1_ship_3_fixtures`, 9 args). Covered by `#[expect(...)]` at 16903.
- [INFO] line:10624 — `seed_appealed_case_with_panel_via_report` (8 args, covered by `#[expect(...)]` at 10620).

Clone chains: 226 total `.clone()` occurrences scanned. None match `.clone().clone()` or `.to_string().clone()` anti-patterns. Pairs like `admin_audit_stream(context.clone(), admin_view.clone())` (5 sites) are conventional Actix `Data<>` clones — not a finding.

---

## Axis 3 — Test isolation / idempotency

### Findings

- [MAJOR] line:10844-11030 — `set_var("BREHON_DISABLE_APPEAL_WINDOW_JOB", "1")` at 10845 paired with inline restore at 11026 (no `Drop` guard). If any `?` between these two points propagates, the env var leaks to subsequent tests in the same process. Recommend `EnvVarGuard` RAII struct (constructor sets var, `Drop` restores via captured `prev_*: Option<OsString>`).
- [MAJOR] line:11074-11225 — same pattern (`BREHON_DISABLE_APPEAL_WINDOW_JOB`) in second test; same fix.
- [MAJOR] line:12799-12898 — same pattern (`BREHON_DISABLE_GRACE_CHECK_JOB`).
- [MAJOR] line:12930-13089 — same.
- [MAJOR] line:13118-13197 — same.
- [MAJOR] line:13214-13330 — same.
- [MAJOR] line:13373-13560 — same. The 7 `BREHON_DISABLE_*_JOB` test sites share one defect class; one `EnvVarGuard` covers all.
- [LOW] line:14578-14589 — `set_var` inside a closure setter pattern that already takes a "value: &str" — already encapsulated in `set()` function (14575), but still lacks `Drop`-based restore.

Idempotency under ISO-week: **non-applicable**. `grep -nE "iso_week"` returned zero hits in the file. No idempotency test calls `chrono::Utc::now().iso_week()`. Constraint's Monday-00:00 flake is moot here. (Constraint preserved for future ISO-week-derived tests.)

Sleeps > 100 ms (test isolation risk): one site only — line:8328 — `tokio::time::sleep(Duration::from_millis(50))` inside a 20-iteration polling loop (max wall ~1 s). Documented in comments at 8326–8327 as the intended SseGuard::Drop yield window. No 100 ms+ sleep elsewhere.

Sleeps ≥ 1 s in tests with no clear reason: none in test code. The two `Duration::from_secs(...)` hits at 559 (PG_DUMP_TIMEOUT_SECS) and 640 (PG_RESTORE_TIMEOUT_SECS) are command-timeout values in helpers (governance_fixtures pg_template), legitimately bounded by docker exec semantics.

Ignored tests missing GH reference: **none**. All 5 `#[ignore = …]` attributes at 1190, 1506, 1842 (GH #43), 3273 (GH #45), 4399 (GH #42) include the expected GH issue. Matches prompt's "Currently expected ignores" list exactly.

Exact row counts on growing tables (sample of 28 `assert_eq!(rows.len(), N)` / `count == N`): all observed assertions are scoped to either (a) seed-and-count loops where the seeder controls N, or (b) DDL/schema-existence counts where N is `0`/`1` and the table's growth is irrelevant. No assertion observed against an unbounded "currently has exactly K rows" on a shared table.

---

## Axis 4 — SQL injection / unsafe query construction

`sql_query(format!(…))` occurs 28 times (1244–8135). Every interpolation observed uses **compile-time literals** (table names from `&["..."]` arrays, column names, index names, pg_type names) or **typed numeric IDs** (`community_a.id.0` at 8135 — `i32`). No interpolation observed uses untrusted input — there is no test input source other than const data.

### Findings

- [LOW] line:1244, 1562 — `sql_query(format!("SELECT count(*) AS n FROM {table}"))` — `{table}` is from a literal `&["actor_pseudonym", "governance_log", …]` array; safe but breaks the `.bind::<Text,_>` pattern used in surrounding integration code. Consider `format_ident!`-style table-name validation via a typed enum (cosmetic only — not a security risk).
- [LOW] line:8135 — `format!("INSERT INTO governance_config (scope, …) VALUES ('community:{}', …)", community_a.id.0)` — `id.0` is `i32`; safe by type. No fix required, flagged for completeness.

No `format!` SQL with `String` interpolation, no shell-passthrough, no `execute(format!(…))` with user-controlled data.

---

## Axis 5 — HTTP assertion completeness

### Scope

Inspected: `/api/v4/...` posting/getting at lines 4053–4396 (`all_mvp_endpoints_return_non_404`), 6232–8351 (admin-config), 15840–15937 (`agpl_source_disclosure_surface_returns_notice`), 15995/16035/16060/16113/16165 (admin HTML pages), 16258–16417 (fed-in-b error twins).

### Findings

- [MAJOR] line:4146-4209 — `all_mvp_endpoints_return_non_404` asserts only that response status is in an allowlist `&[200, 400, 401]` (with malformed payload `{}`). **No probe asserts an endpoint actually returns 200 with a valid payload** — a route that erroneously always 401s would pass. Add at least one happy-path probe per route family (report/endorsement/appeal/jury/admin) with valid body + `assert_eq!(status, StatusCode::OK)` and structural body assertions.
- [LOW] line:15995-16208 — admin HTML happy-path / forbidden / 404 triples are complete. Body assertions present (16022, 16038, etc. assert `<html>` markers). No JSON-body or content-length assertions present, which is appropriate for HTML endpoints but could surface as `content-type: text/html` assertion (one-line add).

Error-path twins coverage: admin HTML endpoints (15995/16035/16060/16113/16165) are paired with `flag_off_returns_404` (16060/16113), `forbidden_for_non_admin` (16035/16165), and happy paths (15995/16113). Twin coverage is acceptable.

`agpl_source_disclosure_surface_returns_notice` (15840) covers happy path for `/api/v4/site` source_disclosure block (15920) and `/api/v4/source` license body (15937); no error twin defined, which is expected because the disclosure endpoint has no auth gate.

Fed-in-b error tests (16258, 16283, 16315, 16363, 16391, 16417) cover 403/429/429/409/200/200 cases — both error and happy paths present; minor `.err().unwrap()` issue noted in Axis 2.

---

## Axis 6 — Magic numbers / threshold hardcoding

### Findings

- [MAJOR] line:10670, 10704, 10708 — comments cite `DEFAULT_REPORT_CASE_THRESHOLD_MICROS = 3_000_000` and `1_000_000` (per-report weight) as numeric literals. Test asserts `4_000_000 > 3_000_000` (10708) using inline numbers. Suspected config key: `case_threshold_micros` (instance default). Replace with `governance_fixtures::DEFAULT_REPORT_CASE_THRESHOLD_MICROS` constant import or `get_int_opt(... "case_threshold_micros")` cascade read — prevents drift when default is tuned.
- [MAJOR] line:9027, 9101, 9122 — comments + assertions reference `DEFAULT_JURY_PANEL_SIZE_FOUNDER_SEVERE = 9` and `DEFAULT_JURY_PANEL_SIZE = 5` as numeric literals in the panel-size cascade test. Same pattern: import the const or read via cascade helper. Suspected config keys: `jury_panel_size`, `jury_panel_size.founder.severe`.

(Magic numbers like `2_642`'s `V0_THRESHOLD = 3` (line 2642 comment) appear documented inline — not a finding because they reflect v0-fixed PRD specifications that won't change.)

---

## Axis 7 — Cross-module dependency leakage

Constraint: tests inside `mod v1_X_fixtures` calling helpers from another `v1_Y_fixtures` module is forbidden; `governance_fixtures::*` is allowed.

### Findings

- [MAJOR] line:10523, 10526, 10665, 10666 — `v1_jm_e_fixtures` reaches into `super::v1_jm_b_fixtures::seed_jury_eligible_snapshots` and `super::v1_jm_b_fixtures::seed_case`. Documented in comments at 10482 and 10602 ("reuses `v1_jm_b_fixtures::seed_case`"), so the leakage is deliberate but still violates the rule. Recommend: hoist `seed_jury_eligible_snapshots` and `seed_case` into `governance_fixtures` as canonical shared helpers, OR duplicate the small seeders inside `v1_jm_e_fixtures`.
- [MAJOR] line:13664, 13903, 14097, 14275 — `v1_sl_d_fixtures` reaches into `super::v1_jm_b_fixtures::seed_jury_eligible_snapshots`. SL-d is sponsor-liability — no reason to depend on jury-management fixtures. Recommend hoisting the seeder to `governance_fixtures`.
- [MAJOR] line:14632, 14956 — `v1_sl_e_fixtures` does the same. Same fix.

(Note: same-module `v1_jm_b_fixtures::*` calls outside the mod, e.g. at top-level test bodies between 11658–17099, are NOT cross-module leakage if they originate from a free-floating test fn at file scope. Inspected — the only inter-module `super::v1_jm_b_fixtures` references are the 10 lines listed above, all from `v1_jm_e`, `v1_sl_d`, `v1_sl_e`.)

---

## Axis 8 — Dead / disabled tests

- All 5 `#[ignore = …]` attributes carry GH-issue references (1190 #43, 1506 #43, 1842 #43, 3273 #45, 4399 #42). Match prompt's expected ignores **exactly**.
- No `#[ignore]` without GH reference.
- No commented-out module-level fn blocks found.
- No `Ok(())`-only stub tests found.
- No `#[ignore]` attributes outside the expected 5.

### Findings

(None — Axis 8 clean.)

---

## Axis 9 — Performance smells

- Heavyweight context boot (`governance_fixtures::bootstrap()` — full Postgres testcontainers + migrations + JWT/site setup) used for every test including simple DB-only assertions. Documented as a deliberate v0 design (the file's single-binary-test convention runs ~1 container per test). Not a defect; trade-off.

### Findings

- [LOW] line:8327-8336 — 20-iteration retry loop with `from_millis(50)` sleep (max 1 s) before `panic!`. Per-test cost ~0.05–1 s depending on Drop scheduling. Acceptable but consider `tokio::task::yield_now()` first iteration before any sleep.

No >100-row seeding loops found (largest is `for i in 0..20` at 8327; next is `0..13` for `seed_jurors(... 13)` at 10856). No loops issue per-row INSERT statements where bulk INSERT would dominate.

No tests-level sleeps ≥ 1 s. Container-level timeouts (PG_DUMP_TIMEOUT_SECS / PG_RESTORE_TIMEOUT_SECS at 559, 640) are bounded helper timeouts — legitimate.

---

## Axis 10 — Sub-phase coverage gaps

Confirmed modules per prompt structural map: governance_fixtures (115), admin_config_fixtures (6100), v1_jm_b (8535), v1_jm_e (10454), v1_sl_b (11658), v1_sl_c (12689), v1_sl_d (13567), v1_sl_e (14426), v1_federation_inbound_a (15393), v1_ship_2 (15456), v1_federation_inbound_b (16186), v1_federation_inbound_e (16518), v1_ship_3 (16699). Free-floating tests (without a named module) include the 1916–5630 v1-JM-a/v1-SL-a/v1-AD region.

### Classification of missing module names

| Sub-phase | Classification | Evidence |
|---|---|---|
| **v1-JM-a** | Embedded | Free-floating tests at 1916 (`v1_jm_a_backfill_populates_v0_snapshot`), 3179 (`v1_jm_a_seed_migration_is_idempotent`); inline JM-a structures (BackfilledRow at 1930). |
| **v1-JM-c** | Embedded | Inline references at 2877, 2881 (golden-path window assertion); JM-c step 9 logic is exercised by `report_to_modlog_golden_path` (2484). |
| **v1-JM-d** | Embedded | Migration list at 1986 cites "2 JM-d Task 1 migrations"; forward-revert-reapply tests at 1191/1507/1843 cover JM-d schema. |
| **v1-SL-a** | Embedded | Free-floating `sponsor_liability_with_founder_multiplier` at 3274 (ignored, GH #45); helper `run_sanction_scenario` at 3520. |
| **v1-SL-c-1** | [LOW] Gap | No explicit reference (`grep` finds only SL-c at 12689). SL-c-1 sub-task either covered by SL-c or absent. |
| **v1-AD-c** | Embedded | `admin_config_fixtures` at 6100 covers AD-b/AD-c per comments at 6207 ("v1-AD-c task 8 helper"). |
| **v1-AD-d** | Embedded | Reference at 6211 ("v1-AD-d will re-use this helper"). |
| **v1-AD-e** | Embedded | Admin-config tests at 6232–8351 (16 tests) span AD-e dashboard/audit. |
| **v1-RT-r1** | Embedded | Migration list at 1089 ("v1-RT-r1 (4 migrations, bump 14 → 18)"); schema probes at 1317–1330 cover RT-r1 columns. |
| **v1-RT-r2** | [MAJOR] Gap | No reference (`grep` returns 0). May not yet exist on `governance-v0` or may be in flight — flag to advisor. |
| **v1-federation-inbound-c** | [MAJOR] Gap | No reference found; fed-in-a, b, e present but c missing. |
| **v1-federation-inbound-d** | [MAJOR] Gap | No reference found. |
| **v1-ship-1** | [MAJOR] Gap | No reference (`grep "v1_ship_1\|v1-ship-1\|ship-1"` returns 0). ship-2 and ship-3 present. |
| **v1-rls-r1** | [LOW] Gap | Likely non-test scope (RLS = recursive learning system — meta-infra, not test e2e). Acceptable absence. |
| **v1-dq-schema-r1** | [LOW] Gap | DQ schema is advisor-side; not in-scope for e2e tests. Acceptable absence. |
| **v1-retro-followups-r1** | [LOW] Gap | Retro infrastructure; not in-scope for e2e tests. Acceptable absence. |

### Findings (Axis 10)

- [MAJOR] coverage gap — v1-RT-r2 has no test surface in `e2e.rs`. If RT-r2 has shipped on `governance-v0`, missing test module is a regression risk.
- [MAJOR] coverage gap — v1-federation-inbound-c missing despite fed-in-a/b/e present.
- [MAJOR] coverage gap — v1-federation-inbound-d missing.
- [MAJOR] coverage gap — v1-ship-1 missing despite ship-2/ship-3 present; first ship sub-phase has no fixture module.
- [LOW] — v1-SL-c-1 not distinguishable from SL-c; either rename for clarity or document the consolidation.
- [LOW] × 3 — v1-rls-r1, v1-dq-schema-r1, v1-retro-followups-r1 are accepted-absent (meta-infra, not e2e scope) — recorded for completeness.

---

## Method notes

- All line numbers verified against the fetched copy of `crates/server/tests/e2e.rs` at branch `governance-v0` via `gh api` token-URL fallback. Raw URL `https://raw.githubusercontent.com/barrie-cork/lemmy/governance-v0/crates/server/tests/e2e.rs` returned HTTP 404 (likely private fork — token URL succeeded).
- File length verified: `wc -l` → 17099 lines. Matches prompt header.
- The mod-decl scan found exactly the 13 modules the prompt cataloged; no undocumented modules.
- Workspace clippy lint config verified by reading `Cargo.toml` at `[workspace.lints.clippy]` on `governance-v0` — `unwrap_used = "deny"`, `expect_used = "deny"`, `allow_attributes = "deny"` confirmed.
- Axis 2 finding count (59) = 9 unwraps + 50 in-test/helper expects + 0 panics-as-violation. The 50 expect count is a tally of `grep -c "\.expect("`; first 20 cited explicitly, remainder consolidated as a single bullet per the section length guideline.
