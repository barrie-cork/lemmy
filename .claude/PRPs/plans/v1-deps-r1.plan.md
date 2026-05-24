# Plan: v1-deps-r1 - Cargo dependency bump migration (diesel-async 0.9 + sha2 0.11 + 10 SemVer-compat)

## 1. Summary

This sub-phase migrates 12 cargo dependency bumps surfaced by Dependabot PR #136 into one bundled phase to amortize cargo + e2e validation cost. Two are breaking: **`diesel-async 0.8.0 -> 0.9.0`** (transaction closure surface rewrite: `ScopedBoxFuture` removed in favour of native async closures via `AsyncFnOnce + AsyncFunc`; affects 42 `.run_transaction` callsites across 31 files + the `lemmy_diesel_utils::connection::DbConn::run_transaction` wrapper + 30 `scoped_futures::ScopedFutureExt` / `ScopedBoxFuture` imports) and **`sha2 0.10 -> 0.11`** (type aliases replaced with newtypes; `std` feature removed; affects 1 production file at `crates/api/api/src/governance/admin_rule_sets.rs` lines 46/120 and 3 test modules in `crates/server/tests/e2e.rs` at lines 938/2526/7302). The remaining 10 SemVer-compat bumps (`diesel 2.3.7->2.3.9`, `tokio 1.50->1.52`, `rustls 0.23.37->0.23.40`, `bcrypt 0.19.0->0.19.1`, `serde_with 3.18->3.20`, `jsonwebtoken 10.3->10.4`, `lettre 0.11.19->0.11.22`, `rss 2.0.12->2.0.13`, `dashmap 6.1->6.2`, `html2text 0.16.7->0.17.1` pulling `html5ever 0.38->0.39` transitively) ride along in a third commit so a single phase-tip e2e gate covers all three. Headline acceptance: phase-branch `phase-v1-deps-r1` at PR-open has `cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"`, `cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"`, and `cmd //c "scripts\brehon\cargo-test.bat --workspace --test e2e --features full"` all exit 0; the cited 42 callsite count holds (re-verified at plan-time = 42); zero `scoped_futures` references remain anywhere in `crates/`; `Cargo.lock` delta matches the union of the three commits' bumps with no unrelated drift.

## 2. Source

- **Brief:** `.claude/PRPs/briefs/v1-deps-r1-planning-1.md` @ commit `043c4f3e1` on `origin/governance-v0` - authored 2026-05-23.
- **Clarify DQ pre-resolutions** (advisor, all resolved on `origin/governance-v0` per the v3 schema; cited in routing decisions and task discipline):
  - **`a3d0e9941441-011`** (DoD executability - Windows wrapper) -> answer (a): every section 15 DoD command MUST be authored as `cmd //c "scripts\brehon\cargo-<verb>.bat ..."`; per-task `commands[]` arrays in `validate-pending-laptop` DQ entries inherit the same wrapper discipline.
  - **`a3d0e9941441-013`** (WP-1 wrapper migration path) -> answer (b): planner-choice with discipline. Planner reads the 0.9 changelog + `transaction` signature at plan-author time. **Choice ratified here: option (a) - update wrapper signature + mechanically rewrite all 42 callsites + remove all 30 `scoped_futures` imports.** See section 4 + section 10.1 for rationale. The brief's "default option (a)" stands.
  - **`a3d0e9941441-014`** (DoD clippy baseline) -> answer (a): Task 0 includes BOTH the 4 wrapper probes from `pre-phase-harness-audit.md` section 1 AND the clippy baseline capture from section 3. If `cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"` exits non-zero on phase-branch HEAD before Task 1, insert a `chore(lint):` task between Task 0 and Task 1.
  - **`a3d0e9941441-015`** (DoD executability - Shape G) -> answer (a): laptop-only throughout. Plan section 15 authors `validate-pending-laptop` (wrapper-prefixed per DQ 011). If 2026-06-01 (DQ #229 re-enable boundary) passes mid-lane, advisor files a `kind: "log"` DQ at the boundary; subsequent impl-tasks raise `kind: "validate-pending"` (Shape G) instead. **Plan section 15 does NOT need re-authoring at the boundary** - the polling loop handles both kinds transparently.
  - **`a3d0e9941441-016`** (per-task class-targeted `commands[]`) -> answer (a): per section 4a in the brief. T1 = check + clippy + lib-test for touched governance crates + e2e (`validate-pending-laptop-e2e`); T2 = check + clippy + targeted `-p lemmy_api --features full --lib` lib-test (sha2 narrow surface lives in `lemmy_api`, not `lemmy_api_common`; see section 11 footnote); T3 = check + clippy only; phase-tip post-T3 = full workspace e2e single gate.
  - **`a3d0e9941441-017`** (cross-phase invariant with RT-r3) -> answer (b): proceed with v1-deps-r1 cut. RT-r3 worktree at session-start has zero commits ahead of trunk; WP-5 mandates planner re-enumeration at plan-author time, so callsite drift is detected mechanically. Re-enumeration at plan-author time **confirms 42 callsites** (= brief's count; no drift from RT-r3).
- **`docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`** - read-only; **no ADR is altered** by this plan. The `sha2` hash chain (ADR-012) is preserved byte-identically (sha2 0.11 keeps the SHA-256 algorithm output stable; only the Rust newtype shape changes). The `actor_pseudonym` invariant (ADR-015) is untouched (no governance write-path semantic change). `CaseStatus::EmergencyRemove` (ADR-013) is untouched.
- **Diesel-async 0.9.0 changelog + docs.rs** (read at plan-author time, no fresh fetch needed by impl-task):
  - Changelog: "Change all transaction related functions to accept a real async closure instead of the scoped boxed variant. This change requires adjusting all call sides of transaction based functions from `conn.transaction(|conn| async move {/* code */}.scope_boxed())` to `conn.transaction(async |conn| /* code */)`."
  - `docs.rs/diesel-async/0.9.0/diesel_async/trait.AsyncConnection.html` confirms the new signature: `for<'r> F: AsyncFnOnce(&'r mut Self) -> Result<R, E> + AsyncFunc<&'r mut Self, Result<R, E>, Fut: Send> + Send + 'a, E: From<Error> + Send + 'a, R: Send + 'a, 'a: 'conn`. `ScopedBoxFuture` no longer appears in the public API. `AsyncPgConnection` still implements `AsyncConnection` under the `postgres` feature.
- **sha2 0.11.0 changelog** (read at plan-author time):
  - "Replace type aliases with newtypes (#678). Removed `std` crate feature; `alloc` available." MSRV bumped to 1.85 (we're on 1.95 per `Cargo.toml:rust-version`). The `Digest` trait surface (`new`, `update`, `finalize`, `digest`) is preserved on `Sha256`.
- **html2text 0.17.1 changelog** (read at plan-author time):
  - "Update html5ever to 0.39.0." No user-visible API change to `from_read` / `Builder::new` / render. The 0.17.0 release also "Split `html2text` example into `html2text-cli` crate" (binary move; we don't depend on the example). **Risk class on the html2text bump is LOW**, contrary to the brief's WP-4 framing - confirmed at plan-author time.
- **`.claude/rules/decision-queue.md` schema-v3** - every new DQ entry written from this lane uses `id: "<session_id>-<sequence>"` per `bash scripts/brehon/dq-v3-new-entry.sh`. Pre-v3 entries unchanged.
- **`.claude/rules/advisor-orchestrator.md`** sections 2.4 (mandatory file-class lesson injection), 3.2 (mandatory user gates), 4.1 (cohort dispatch), 5.3 (G4 classifier - applies post-mutation if a `validate-pending-laptop` entry returns `result: "fail"`).
- **`.claude/rules/branch-manager.md`** - BM file-ownership boundaries (BM session never touches `crates/**`, `Cargo.toml`, `Cargo.lock`).
- **`.claude/rules/phase-branch.md`** - phase branch `phase-v1-deps-r1` off `governance-v0`; PR opens with `--repo barrie-cork/lemmy` against `governance-v0` (NOT `main`); CodeRabbit auto-reviews.

### 2.1 Lessons that bind section 13 decisions

Per advisor-orchestrator section 2.4 mandatory file-class lesson injection - the file classes touched by this plan are:

- **`Cargo.toml` / `Cargo.lock` (workspace + sub-crate manifests)** - Tasks 1+2+3. Lessons that bind:
  - `feedback_fix_impl_enumerate_all_callsites.md` - `rg "\.run_transaction" crates/`, `rg "scoped_futures" crates/`, `rg "Sha256\b" crates/ tests/`, `rg "impl.*Digest" crates/ tests/`, `rg "html2text" crates/` enumeration before authoring any callsite edit. **Drift from section 11's 42-callsite + 30-import + 5-sha2-callsite count is the failure mode**; if drift is observed, file `kind: "blocker"` DQ.
  - `feedback_features_full_workspace_only.md` - every cargo gate uses `--workspace --features full`; **never `-p lemmy_server --features full`** (the recurring footgun). Brief section 5 reinforces this.
  - `feedback_features_full_p_crate_incompatible.md` - companion to the workspace-only rule; `-p <crate> --features full` is structurally broken in this workspace. T2's targeted lib-test uses `-p lemmy_api --features full --lib` which IS valid because `lemmy_api` defines a `full` feature (verified at plan-author time per section 11 footnote); contrast with `-p lemmy_server --features full` which does not.
- **`crates/server/tests/e2e.rs`** edits (T2 touches 3 import statements at lines 938, 2526, 7302; these are import-line edits, NOT test-body edits):
  - `feedback_lemmy_error_no_std_error.md` - Case A (`LemmyResult<()>`) discipline. T2 touches only `use sha2::{Digest, Sha256};` lines and (potentially) `Sha256::new()` / `Sha256::digest()` callsites; **NO test-fn signature changes**; **NO error-shape changes**. If sha2 0.11's newtype rewrite forces a test-body edit (e.g. type annotation), the impl-task verifies the touched fn already returns `LemmyResult<()>` Case A; if not, it files `kind: "blocker"` DQ.
  - `feedback_async_pool_test_pattern.md` - T1 touches `use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};` patterns IF e2e.rs imports `scoped_futures` directly. **Re-verification at plan-author time: e2e.rs has zero `.run_transaction` callsites and zero `scoped_futures` imports** (`grep -n "\.run_transaction\|scoped_futures" crates/server/tests/e2e.rs | wc -l` = 0). T1 does NOT need to edit e2e.rs.
  - `feedback_clippy_test_style.md` - `?` propagation, no `unwrap`/`expect`. Applies to T2's test-body edits IF any.
- **`crates/api/api/**` handlers doing 2+ DB writes** - T1 rewrites the closure shape inside `.run_transaction(...)` blocks in 31 files including 12 governance handlers:
  - `feedback_multi_write_handlers_need_transactions.md` - confirms the load-bearing nature of `.run_transaction`; the migration **must NOT silently widen or narrow transaction scopes** during the mechanical rewrite. The closure body inside the new `async |conn| { ... }` MUST be the byte-equivalent of the old closure body inside `async move { ... }.scope_boxed()`. Any semantic change beyond removing the `.scope_boxed()` / `scoped_futures::ScopedFutureExt` is OUT OF SCOPE for v1-deps-r1.
- **`scripts/brehon/cargo-*.bat | sh` wrapper edits** - N/A. This plan does NOT edit wrapper scripts.

### 2.2 Cross-cutting lessons (apply to every task)

- `feedback_validate_pending_laptop_must_use_wrapper.md` - Windows wrapper discipline for every cargo invocation in `commands[]` arrays (per DQ `a3d0e9941441-011`).
- `feedback_laptop_default_for_validate_pending.md` - Shape G suspended; impl-task pushes + raises `kind: "validate-pending-laptop"`; advisor laptop runs commands locally.
- `feedback_targeted_validate_pending_laptop_commands.md` - per-task class-targeted command selection (per DQ `a3d0e9941441-016`).
- `feedback_windows_e2e_requires_bat_wrapper.md` - e2e invocation on Windows requires `cmd //c "scripts\\brehon\\cargo-test.bat ..."` for libpq.dll discovery (vcpkg).
- `feedback_clippy_per_module_deny_requires_workspace_allow.md` - clippy baseline trap: workspace-level group-deny silently breaks per-module `#![deny]` (per DQ `a3d0e9941441-014`; Task 0 captures baseline).
- `feedback_plan_dod_dry_run_at_write.md` + `feedback_pre_phase_dod_smoke_test.md` - every section 15 command was dry-run by the advisor at plan-author time (see section 19 Notes for results).
- `feedback_complexity_score_pre_split.md` - section 5.1 score computed mechanically (= 6/10, below Sonnet split threshold of >8).
- `feedback_parallel_cohort_dispatch.md` + `feedback_explicit_file_arrays_on_tasks.md` + `feedback_cohort_validation_dependency_check.md` - `[P]` markers gated on YAML `union(creates, modifies)` disjointness AND `requires:` dependency check. T2 and T3 both `requires:` Task 1; they cannot be cohort-peers of Task 1. T2 and T3 are file-disjoint with each other but both depend on T1 -> serial dispatch overall (T1 -> T2 -> T3).
- `feedback_handover_trailer_cohort_propagation.md` - single-task cohorts still aggregate handover trailers; brief section 3a of each next-task brief receives prior commit's `HANDOVER:` trailer.
- `feedback_read_canonical_before_writing_spec.md` - canonical sibling for `.claude/PRPs/plans/v1-deps-r1.plan.md` is `.claude/PRPs/plans/v1-ship-3.plan.md` (most-recent shipped sibling with `validate-pending-laptop` mode); cited verbatim in section 10 (Patterns to mirror) and section 15 (DoD).
- `feedback_fix_impl_pre_push_cargo_check.md` - mechanical fix-impl briefs MUST include a section 4 Constraint requiring `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full"` BEFORE the worker pushes. Applies if G4 classifier fires post-Task-1 / post-Task-2.

### 2.3 ADRs cited

- **ADR-012** (read-only) - `sha2` hash chain. Task 2 preserves the SHA-256 algorithm output byte-identically; the chain remains valid; no governance_log re-emission needed.
- **ADR-013** (read-only) - `CaseStatus::EmergencyRemove` exhaustive match invariant. T1 touches `crates/api/api/src/governance/admin_emergency_remove.rs` (1 `.run_transaction` callsite) but only the closure-shape rewrite; no match-expression edits.
- **ADR-015** (read-only) - `actor_pseudonym` invariant. T1 touches multiple governance handlers but only the closure-shape rewrite; no new `person_id` writes; no new `governance_log::append` callsites.

### 2.4 Related prior plans (canonical-shape mirror)

- **`.claude/PRPs/plans/v1-ship-3.plan.md`** - **most-recent shipped sibling using `validate-pending-laptop`**. Plan-section schema, section 13 task FILES YAML, section 15 DoD wrapper-prefix style, section 16a Stories pattern all mirrored here verbatim.
- **`.claude/PRPs/plans/v1-RT-r2.plan.md`** - sibling sub-phase that ran in parallel with ship-3 (multi-lane discipline). Cited for section 6 relationship discipline.
- **`.claude/PRPs/plans/v1-quality-r1.plan.md`** - cargo-edit-touching sibling with a `.rustfmt.toml` + reformat-bulk commit; precedent for "mechanical sweep + isolated chore commit" shape used here (T1 wrapper + callsites in one commit).

## 3. Problem statement

Dependabot PR #136 (opened 2026-05-18) bundled 12 cargo bumps into one mergeable PR. Investigation 2026-05-23 (per session retro if authored) revealed:

1. **`diesel-async 0.9` is a breaking change requiring a 42-callsite + 30-import + 1-wrapper mechanical rewrite.** The trait bound on `AsyncConnection::transaction` shifted from a scoped-boxed `FnOnce` to a native `AsyncFnOnce + AsyncFunc` shape. Our wrapper at `crates/diesel_utils/src/connection.rs:62` re-exports this trait bound via `ScopedBoxFuture` - when the underlying trait disappears, the wrapper fails to compile, and so does every callsite that delegates to it.
2. **`sha2 0.11` is a breaking change requiring trait-resolution pre-flight.** Type aliases (`pub type Sha256 = CoreWrapper<Sha256Core>` or similar) became newtypes. The `Digest` trait surface (`new`, `update`, `finalize`, `digest`) is preserved on `Sha256` - but any code that stores `Sha256` in a struct field with a `Digest`-bounded type parameter may surface as `E0277: trait bound not satisfied`. Pre-flight grep at plan-author time (`rg "impl.*Digest" crates/ tests/`) returned **zero matches** for explicit `Digest` impls; `Sha256` is only used via direct method calls. Risk = low.
3. **The 10 SemVer-compat bumps are individually low-risk** but bundling with (1) and (2) amortizes the cargo + e2e gate (~26 min e2e + ~3 min check + ~2 min clippy per pass - ~31 min total per phase tip; running three separate phases would triple this for no risk reduction).
4. **Merging PR #136 as-is would not compile.** Per the brief, user decision 2026-05-23 was to defer as a dedicated sub-phase rather than fix-in-PR'ing #136. Dependabot will re-open with a fresh head SHA at the next opening cycle; closing #136 is OPTIONAL - this plan is the durable record.

Each problem is addressable in a single section 13 task; bundling under one phase reduces the phase overhead (bm-cut + bm-pr + CR review + bm-merge) from 3x to 1x.

## 4. Solution statement

Three section 13 tasks (plus Task 0 pre-flight, Task 4 retro), all serial (no `[P]`):

- **Task 1 (diesel-async 0.9 migration):** One impl-task, one commit. Three coordinated edits:
  1. **Workspace `Cargo.toml`:** bump `diesel-async = "0.8.0"` -> `diesel-async = "0.9.0"` (verify at task-time whether `0.9.1` or later patch exists; prefer latest per brief WP-6).
  2. **Wrapper `crates/diesel_utils/src/connection.rs`:** remove the `scoped_futures::ScopedBoxFuture` import (line 13); rewrite the `DbConn::run_transaction` trait bound (lines 61-72) from `F: for<'r> FnOnce(&'r mut AsyncPgConnection) -> ScopedBoxFuture<'a, 'r, LemmyResult<R>> + Send + 'a` to the 0.9 native shape: `for<'r> F: AsyncFnOnce(&'r mut AsyncPgConnection) -> LemmyResult<R> + diesel_async::AsyncFunc<&'r mut AsyncPgConnection, LemmyResult<R>, Fut: Send> + Send + 'a`. The wrapper body stays `self.deref_mut().transaction::<_, LemmyError, _>(callback).await` - no internal shape change. The wrapper's public method NAME, the `LemmyResult<R>` return, and the `R: Send + 'a` constraint are all preserved.
  3. **Mechanical callsite rewrite across 31 files (42 callsites total):** every `.run_transaction(|conn| { async move { /* body */ }.scope_boxed() })` becomes `.run_transaction(async |conn| { /* body */ })`. The closure body's text is preserved byte-for-byte except for the wrapping shape: `|conn| { async move { X }.scope_boxed() }` -> `async |conn| { X }`. Every `use diesel_async::{..., scoped_futures::ScopedFutureExt};` import line is rewritten to drop `scoped_futures::ScopedFutureExt` from the import group (30 files); when that leaves a lone re-export, the entire `use diesel_async::...` line may collapse - verify per file. Per `feedback_multi_write_handlers_need_transactions.md`: **NO semantic change inside any transaction body**; this is a closure-shape rewrite only.

  The cleanest expression of these three edits is **one impl-task commit**. The brief WP-1 suggested splitting wrapper-first / callsites-second, but the wrapper signature change makes every callsite uncompilable until both halves land - a split would leave a non-compiling intermediate commit on the phase branch, breaking `git bisect` and the per-task DoD discipline. **Single commit ratified.**

- **Task 2 (sha2 0.11 migration):**
  1. **Workspace `Cargo.toml`:** bump `sha2 = "0.10"` -> `sha2 = "0.11"`.
  2. **Pre-flight verification at task-time** (Junior runs as the first IMPLEMENT step): `rg "impl.*Digest" crates/ tests/` -> if non-empty (post-trunk drift since plan-author time), file `kind: "blocker"` DQ; if empty, proceed.
  3. **Production code: `crates/api/api/src/governance/admin_rule_sets.rs`** at lines 46 (`use sha2::{Digest, Sha256};`) and 120 (`let text_sha256 = Sha256::digest(data.rule_text.as_bytes()).to_vec();`) - verify the `Digest` trait is still in scope (it is - sha2 0.11 keeps the trait export); verify `Sha256::digest(&[u8])` still returns the byte-array (it does - type-alias-to-newtype shift preserves method API). **Expected diff: zero lines** (no semantic edit needed); if 0.11 surfaces a deprecation warning under `-D warnings`, the impl-task adds the minimal annotation per `feedback_clippy_test_style.md`.
  4. **Test modules: `crates/server/tests/e2e.rs`** at lines 938, 2526, 7302 (each `use sha2::{Digest, Sha256};`) and the adjacent `Sha256::new()` / `Sha256::digest()` / `hasher.update(...)` / `hasher.finalize()` callsites - same verification: API preserved -> expected diff zero lines. **No test-body edit; no helper-fn signature change; no Case-A discipline trip.**
  5. **`std` feature check:** verify no `crates/*/Cargo.toml` enables `sha2 = { ..., features = ["std"] }` (`rg "sha2.*features.*std" crates/` at plan-author time -> zero matches; workspace root `sha2 = "0.10"` is a bare version pin, default features only). **No action needed.**

- **Task 3 (SemVer-compatible bundle, 10 bumps):**
  1. **Workspace `Cargo.toml`** edits (one per dep):
     - `diesel = { version = "=2.3.7", ... }` -> `diesel = { version = "=2.3.9", ... }`
     - `tokio = { version = "1.50.0", features = ["full"] }` -> `tokio = { version = "1.52.0", features = ["full"] }`
     - `rustls = { version = "0.23.37", features = ["ring"], default-features = false }` -> `rustls = { version = "0.23.40", features = ["ring"], default-features = false }`
     - `bcrypt = "0.19.0"` -> `bcrypt = "0.19.1"`
     - `serde_with = "3.18.0"` -> `serde_with = "3.20.0"`
     - `html2text = "0.16.7"` -> `html2text = "0.17.1"`
  2. **Sub-crate `Cargo.toml`** edits (4 files):
     - `crates/api/api_utils/Cargo.toml:line:jsonwebtoken = { version = "10.3.0", ... }` -> `jsonwebtoken = { version = "10.4.0", ... }`
     - `crates/email/Cargo.toml:line:lettre = { version = "0.11.19", default-features = false, features = [...] }` -> `lettre = { version = "0.11.22", ..., features = [...] }`
     - `crates/routes/Cargo.toml:line:rss = "2.0.12"` -> `rss = "2.0.13"`
     - `crates/utils/Cargo.toml:line:dashmap = { version = "6.1.0", optional = true }` -> `dashmap = { version = "6.2.0", optional = true }` (verify exact 6.2.x at task-time per brief WP-6)
  3. `Cargo.lock` updates automatically via `cargo check --workspace --features full`. **No `crates/**` Rust file edits.**

The reader can predict section 11 from section 4: workspace `Cargo.toml`; `Cargo.lock`; `crates/diesel_utils/src/connection.rs`; ~31 files in `crates/api/**`, `crates/apub/**`, `crates/db_schema/**`, `crates/routes/**`; potentially `crates/api/api/src/governance/admin_rule_sets.rs` (T2; may be zero-diff); `crates/api/api_utils/Cargo.toml`; `crates/email/Cargo.toml`; `crates/routes/Cargo.toml`; `crates/utils/Cargo.toml`; `.claude/PRPs/reports/v1-deps-r1-retro.md` (T4).

## 5. Metadata

- **Phase:** `v1-deps-r1`
- **Branch:** `phase-v1-deps-r1` (cut by BM-task before Task 1, from `origin/governance-v0 @ <SHA-at-cut>` after both `v1-ship-3` AND `v1-RT-r2` merges + retros land per brief Hard precondition).
- **Target impl-task model:** `sonnet-4-6` (default). Split-DQ threshold is `>8`.
- **Estimated tasks:** 5 (Task 0 pre-flight + Tasks 1-3 deliverables + Task 4 retro).
- **Estimated cargo budget:** non-binding under `validate-pending-laptop` mode; cargo runs on the laptop advisor session, not the EliteDesk Junior worker. Local cargo + e2e peak ~6 GB on the laptop (one `cargo test --workspace --features full --test e2e` warm; T1 may also trigger a full re-compile of the dependency graph because `diesel-async` is a deep dep - expect ~5-8 min first warm rebuild).
- **Forbidden-window applicability:** non-binding for impl-task dispatch (Shape G suspended -> cargo runs on laptop; not in cohort scheduling). Standard windows still bind any ad-hoc laptop cargo (per `advisor-orchestrator.md` section 5.1 sub-section "Cargo never runs on the EliteDesk worker" + "validate-pending-laptop handler" section 5.2).
- **Complexity score:** **6/10** - see breakdown below.
- **Validation mode:** `validate-pending-laptop` (Shape G SUSPENDED until 2026-06-01 per DQ #229 + `project_shape_g_suspended_2026_05_16`). impl-task pushes the worker branch + raises `kind: "validate-pending-laptop"` DQ entry with `commands[]` populated; the advisor laptop session runs the commands locally per `.claude/rules/decision-queue.md` section "validate-pending-laptop" + `advisor-orchestrator.md` section 5.2. **If the lane runs past 2026-06-01** (DQ `a3d0e9941441-015` answer (a) protocol): advisor files `kind: "log"` DQ at the boundary; subsequent impl-tasks raise `kind: "validate-pending"` (Shape G) instead; **plan section 15 does not need re-authoring**.

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Sonnet target -> split-DQ threshold is `>8`.

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| Section 13 impl tasks above 5 | +1 each | 0 | 3 impl tasks (Tasks 1-3). `max(0, 3-5) = 0`. |
| Migrations touched | +2 each | 0 | No SQL migrations. |
| Crates touched | +1 each | 11 (raw) - 5 (Cargo.toml-only credit) = **6** | Distinct rust crates touched across all tasks: `lemmy_api`, `lemmy_api_crud`, `lemmy_api_utils`, `lemmy_apub_activities`, `lemmy_apub`, `lemmy_db_schema`, `lemmy_diesel_utils`, `lemmy_email`, `lemmy_routes`, `lemmy_server` (tests/e2e.rs imports), `lemmy_utils` = 11. **Credit -5 for the 5 crates that only see `Cargo.toml` version-pin edits in T3** (`lemmy_api_utils`, `lemmy_email`, `lemmy_routes` Cargo-only edit, `lemmy_utils`, plus workspace manifest itself) - no Rust semantic change in those crates. T1 + T2's 6 crates remain at full weight. |
| `crates/server/tests/e2e.rs` edits | +3 each | 0 (mitigation accepted) | T2 touches 3 import lines (938, 2526, 7302) - IF any edit is needed; expected diff = 0 lines per section 4 Task 2 step 4. Even at worst case (3 import-line edits), the e2e edit-hang concern is non-applicable per brief section 5 explicit exclusion ("edits are at import statements, not in test bodies"). **Counted as 0.** T1 does NOT touch e2e.rs (verified at plan-author time - zero `.run_transaction` callsites in e2e.rs). |
| New ADR-affecting decisions | +2 each | 0 | All decisions are dep-bump-derived or DQ-resolved; ADR-012 (sha2 hash chain), ADR-013, ADR-015 are read-only and unchanged. |
| Cargo budget peak above 6 GB | +1 per GB | 0 | `validate-pending-laptop` mode: laptop, not EliteDesk; factor non-binding. |
| **Total** | - | **6** | 0 + 0 + 6 + 0 + 0 + 0 = **6/10**. Threshold for split-DQ (Sonnet): `>8`. Score **6 <= 8** -> no split-DQ filed. Matches brief section 8 estimate of 6/10. |

### 5.2 Per-task complexity ceiling (Sonnet target = `<=4 files / <=2 crates`)

- **Task 1** modifies ~32 files across ~7 crates (`lemmy_api`, `lemmy_api_crud`, `lemmy_apub_activities`, `lemmy_apub`, `lemmy_db_schema`, `lemmy_diesel_utils`, `lemmy_routes`) + workspace `Cargo.toml`. **WAY over the <=4-file / <=2-crate Sonnet ceiling.** Acceptable because this is a **single semantic edit** (closure-shape rewrite) propagated mechanically across all 42 callsites; the impl-task uses `rg`-enumerate-first discipline (per `/edit-mechanical` skill, suggested in section 13 Task 1 wording) and edits each file with a uniform pattern. The brief acknowledges this scope explicitly and section 13 Task 1 is authored with file-enumeration + per-file Edit discipline. **Mitigation credit accepted** because:
  - The edit pattern is byte-uniform across all 30 closure-bearing files (the wrapper file is the 31st, with a different edit shape).
  - `rg "\.run_transaction" crates/ | rg -v ":42" | wc -l` (a post-T1 check) MUST return 0; this is the mechanical "did I get them all" assertion.
  - The fallback if a callsite is missed: the next `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full"` fails with the canonical "expected closure with `async` keyword" error; the impl-task adds the missed callsite in the same commit (per `feedback_fix_impl_pre_push_cargo_check.md`).
- **Task 2** modifies <=2 files (`Cargo.toml`; possibly `crates/api/api/src/governance/admin_rule_sets.rs` if sha2 0.11 surfaces a warning; possibly `crates/server/tests/e2e.rs` if same). 2 crates max (`lemmy_api`, `lemmy_server`). **Under Sonnet ceiling.**
- **Task 3** modifies 5 files (1 workspace + 4 sub-crate `Cargo.toml`). 0 Rust crates semantically. **Under ceiling (Cargo.toml edits don't trigger ceiling - pure version-pin).**

### 5.3 Split-or-proceed DQ

**Not filed.** Score 6 <= Sonnet threshold of 8. The brief explicitly chose to bundle into one phase to amortize the e2e gate. Splitting T1 into "wrapper" + "callsites" would leave a non-compiling intermediate commit on the phase branch (rejected per section 4 ratification). Splitting T1 from T2+T3 into separate phases doubles the bm-cut/bm-pr/bm-merge overhead for no risk reduction. **Proceed as one phase, three sequential tasks.**

---

## 6. Relationship to other v1 sub-phases

- **Hard precondition (brief section 0):** v1-deps-r1 lane MUST NOT be cut until BOTH `v1-ship-3` AND `v1-RT-r2` have merged to `governance-v0` AND their retros are signed off. Both lanes' write paths overlap with this phase's 42-callsite rewrite - concurrent cuts produce merge conflicts on every `*.run_transaction(...)` site. Pre-queue checklist in brief section 9 enforces this.
- **Concurrent with:** none active. RT-r3 lane exists as an empty worktree (zero commits ahead of trunk at session-start per DQ `a3d0e9941441-017`); if RT-r3 starts mid-lane, the multi-lane discipline (`.claude/rules/multi-lane-worktree.md`) keeps the two lanes' DQ writes per-worktree-isolated and the PMD writes canonical-shared. Cross-lane callsite drift on `.run_transaction` is detected mechanically by the section 13 Task 0 re-enumeration probe.
- **Followed by:** none planned. v1-deps-r1 closes the Dependabot PR #136 backlog. Future Dependabot PRs land independently via the same lane shape (cut, plan, ship) if breaking; via Dependabot directly if SemVer-compat-only.
- **`brehon-fork-deps-r1` worktree** at `C:/Users/barri/Developer/brehon-fork-deps-r1` per `.claude/rules/multi-lane-worktree.md` section "Layout". Bootstrap per `.claude/rules/multi-lane-worktree.md` section "Lifecycle" (submodule init + `.mcp.json` + `.env` + `settings.local.json` copy + canonical PMD absolute path).

## 7. Preflight guardrails inherited from prior phases

- **R1** - every `i32 <-> i64` comparison uses `i64::from(...)`, never `as` cast (per `feedback_clippy_test_style.md`). **Non-applicable to this plan** (no int-comparison code touched).
- **R5** - Task 0 enumerates ALL probes explicitly (per JM-b retro-events Event 4 + `.claude/rules/pre-phase-harness-audit.md`).
- **R6** - all clippy invocations use `--no-deps` uniformly (per JM-b retro-events Event 3).
- **R7** - test-target compile runs after each task that touches a struct or re-export. **T1 touches `DbConn::run_transaction` (a public method on a public type) -> R7 binds for T1.** T2 touches `Sha256::digest` callsites only (no struct or re-export change -> R7 non-binding for T2). T3 is Cargo-only -> R7 non-binding for T3.
- **R8** - test fn outer return MUST be `LemmyResult<()>` (Case A per `feedback_lemmy_error_no_std_error.md`). **Non-applicable** - this plan adds zero new test fns.
- **R9 (this plan):** the T1 mechanical sweep uses `rg`-enumerate-first per `/edit-mechanical` skill semantics - IMPLEMENT walks the file list once before any Edit, and each per-file Edit's `old_string` targets the precise `.run_transaction(|conn| { async move { ... }.scope_boxed() })` block (a 3-line pattern). The affected files are NOT e2e.rs and are far smaller (<=1500 lines per file), so Edit-hang risk is negligible - but the `rg`-first discipline still applies for correctness.
- **R10 (this plan, per `feedback_read_canonical_before_writing_spec.md`):** canonical-schema-first gate - before any section 13 task's IMPLEMENT block lands, the impl-task Junior MUST Read:
  - For Task 1: `crates/diesel_utils/src/connection.rs:1-95` (full wrapper + its imports) AND **at least 3 sample callsites** to absorb the current closure shape: `crates/api/api/src/governance/admin_close_case.rs:43-50`, `crates/apub/activities/src/governance/inbox.rs:196-218`, `crates/db_schema/src/source/governance/governance_log.rs:285-318`.
  - For Task 2: `crates/api/api/src/governance/admin_rule_sets.rs:40-130` (full sha2 usage block) AND `crates/server/tests/e2e.rs:930-1050` (one of the three test-module sha2 usages) to confirm zero structural touch is needed.
  - For Task 3: read each of the 5 `Cargo.toml` files at the lines being edited (verify exact current version string before substituting the new one).
- **R11 (this plan, per `feedback_fix_impl_enumerate_all_callsites.md`):** for Task 1's wrapper signature change, the impl-task Junior MUST run `rg "\.run_transaction" crates/ tests/ | wc -l` BEFORE its first Edit and confirm the count is exactly **42**. If `rg` returns !=42, **STOP and file `kind: "blocker"` DQ** - the plan section 11 estimate is wrong or trunk has drifted; do NOT silently expand or contract scope. Same for `rg "scoped_futures" crates/ | wc -l` -> MUST return **30**. Same for `rg "scope_boxed" crates/ tests/ | wc -l` -> MUST return **38** (the per-callsite `.scope_boxed()` invocations).
- **R12 (this plan, per `feedback_multi_write_handlers_need_transactions.md`):** the T1 sweep is closure-shape only. **No edit to the closure body** (the code between `async move {` and `}.scope_boxed()`) other than removing the wrapping shape. If a per-file Edit's `new_string` semantically differs from `old_string` beyond removing `async move { ... }.scope_boxed()` wrappers and `use ..., scoped_futures::ScopedFutureExt;` lines, **STOP and file `kind: "blocker"` DQ**.
- **R13 (this plan):** **the workspace `Cargo.toml` bumps and the `Cargo.lock` regeneration are part of T1's single commit** (for T1's diesel-async bump), T2's single commit (for T2's sha2 bump), and T3's single commit (for T3's bundle). Never split a dep bump's `Cargo.toml` edit from its consuming-code edit across two commits - that produces a non-compiling intermediate state on the phase branch.

## 8. Flow design

### 8.1 Before state (`governance-v0` after v1-ship-3 + v1-RT-r2 merges)

```
Cargo.toml:
  diesel-async = "0.8.0"
  sha2 = "0.10"
  diesel = { version = "=2.3.7", ... }
  tokio = "1.50.0"
  rustls = "0.23.37"
  bcrypt = "0.19.0"
  serde_with = "3.18.0"
  html2text = "0.16.7"

crates/diesel_utils/src/connection.rs:13 -> `scoped_futures::ScopedBoxFuture` import
crates/diesel_utils/src/connection.rs:61-72 -> run_transaction with `ScopedBoxFuture<'a, 'r, _>` trait bound

42 callsites of `.run_transaction(|conn| { async move { /* body */ }.scope_boxed() }).await` across 31 files
30 `use diesel_async::{..., scoped_futures::ScopedFutureExt}` imports

crates/api/api/src/governance/admin_rule_sets.rs:46 -> `use sha2::{Digest, Sha256};`
crates/api/api/src/governance/admin_rule_sets.rs:120 -> `Sha256::digest(...)`
crates/server/tests/e2e.rs:938,2526,7302 -> `use sha2::{Digest, Sha256};` (3 distinct test modules)

crates/api/api_utils/Cargo.toml -> jsonwebtoken = "10.3.0"
crates/email/Cargo.toml -> lettre = "0.11.19"
crates/routes/Cargo.toml -> rss = "2.0.12"
crates/utils/Cargo.toml -> dashmap = "6.1.0"
```

### 8.2 After state (post-v1-deps-r1 phase-branch tip)

```
Cargo.toml:
  diesel-async = "0.9.0"          # (verify latest patch at task-time)
  sha2 = "0.11"
  diesel = { version = "=2.3.9", ... }
  tokio = "1.52.0"
  rustls = "0.23.40"
  bcrypt = "0.19.1"
  serde_with = "3.20.0"
  html2text = "0.17.1"

crates/diesel_utils/src/connection.rs:13 -> import line collapses (ScopedBoxFuture removed)
crates/diesel_utils/src/connection.rs:61-72 -> run_transaction with `AsyncFnOnce + AsyncFunc<Fut: Send>` trait bound

42 callsites of `.run_transaction(async |conn| { /* body */ }).await` (closure body unchanged)
0 `scoped_futures` references anywhere in crates/

crates/api/api/src/governance/admin_rule_sets.rs:46,120 -> unchanged or minimally-adjusted (sha2 0.11 preserves API)
crates/server/tests/e2e.rs:938,2526,7302 -> unchanged or minimally-adjusted

crates/api/api_utils/Cargo.toml -> jsonwebtoken = "10.4.0"
crates/email/Cargo.toml -> lettre = "0.11.22"
crates/routes/Cargo.toml -> rss = "2.0.13"
crates/utils/Cargo.toml -> dashmap = "6.2.x"

Cargo.lock -> ~70/-95 line delta (per brief estimate; verify at task-time)
```

### 8.3 Task 1 per-file edit diagram (closure rewrite)

For each of the 30 closure-bearing files (excluding the wrapper):

```
BEFORE:
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
                                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ remove this item from the use group
...
conn
  .run_transaction(|conn| {
    async move {
      // ... body ...
    }.scope_boxed()
  })
  .await?;

AFTER:
use diesel_async::RunQueryDsl;          // (or use diesel_async::{RunQueryDsl, OtherThing};
                                        // if other items were in the group; preserve the rest)
...
conn
  .run_transaction(async |conn| {
    // ... body (verbatim, unchanged) ...
  })
  .await?;
```

For the wrapper file `crates/diesel_utils/src/connection.rs`:

```
BEFORE (lines 6-14):
use diesel_async::{
  AsyncConnection,
  pg::AsyncPgConnection,
  pooled_connection::{ ... },
  scoped_futures::ScopedBoxFuture,
};

AFTER:
use diesel_async::{
  AsyncConnection,
  pg::AsyncPgConnection,
  pooled_connection::{ ... },
};

BEFORE (lines 60-73):
impl DbConn<'_> {
  pub async fn run_transaction<'a, R, F>(&mut self, callback: F) -> LemmyResult<R>
  where
    F: for<'r> FnOnce(&'r mut AsyncPgConnection) -> ScopedBoxFuture<'a, 'r, LemmyResult<R>>
      + Send
      + 'a,
    R: Send + 'a,
  {
    self
      .deref_mut()
      .transaction::<_, LemmyError, _>(callback)
      .await
  }
}

AFTER:
impl DbConn<'_> {
  pub async fn run_transaction<'a, R, F>(&mut self, callback: F) -> LemmyResult<R>
  where
    for<'r> F: AsyncFnOnce(&'r mut AsyncPgConnection) -> LemmyResult<R>
      + diesel_async::AsyncFunc<&'r mut AsyncPgConnection, LemmyResult<R>, Fut: Send>
      + Send
      + 'a,
    R: Send + 'a,
  {
    self
      .deref_mut()
      .transaction::<_, LemmyError, _>(callback)
      .await
  }
}
```

`AsyncFunc` is the diesel-async-internal trait that makes `AsyncFnOnce` callable in the `transaction` body; both bounds are required per `docs.rs/diesel-async/0.9.0/diesel_async/trait.AsyncConnection.html` (section 2 cited). If at task-time the trait re-export path is `diesel_async::AsyncFunc` vs `diesel_async::prelude::AsyncFunc` vs something else, the impl-task verifies and uses the canonical path (Junior task-time check; not a planning-time assertion).

## 9. Mandatory reading

The impl-task subagent MUST Read these files (in this order) before its first Edit on each task.

### Task 0

- `.claude/rules/pre-phase-harness-audit.md` - full text. Probes 0-11 are mandatory per R5.
- `.claude/PRPs/briefs/v1-deps-r1-planning-1.md` - entire brief.
- `Cargo.toml` (workspace root) lines 174-242 - every dep this plan touches, current pin values.

### Task 1

- **Schema / type definitions:**
  - `crates/diesel_utils/src/connection.rs:1-95` - full wrapper file (imports + `DbConn` impl + `run_transaction` method).
  - `Cargo.toml:182` - the `diesel-async = "0.8.0"` line being bumped.
- **Existing patterns (MIRROR refs):**
  - **Three canonical callsite shapes** to confirm uniformity:
    1. `crates/api/api/src/governance/admin_close_case.rs:43-50` - simple single-call closure (uses `process_close` helper inside the closure).
    2. `crates/apub/activities/src/governance/inbox.rs:196-218` - multi-statement closure (acquires evict lock, calls helper, builds outcome).
    3. `crates/db_schema/src/source/governance/governance_log.rs:285-318` - db_schema-side `.scope_boxed()` callsite (different crate, same pattern).
  - **One canonical import shape** from each affected crate to confirm import-collapse rules:
    - `crates/api/api/src/governance/admin_close_case.rs:13` - `use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};` (the canonical multi-item-with-`scoped_futures` import).
    - `crates/api/api/src/site/registration_applications/approve.rs:4` - `use diesel_async::scoped_futures::ScopedFutureExt;` (the canonical sole-item `scoped_futures` import - removes the entire `use` line).
- **External documentation (planner read at plan-author time; cited verbatim in section 2 - impl-task does NOT need fresh fetch unless `cargo check` produces an error mentioning a 0.9 surface name):**
  - diesel-async 0.9.0 changelog (cited in section 2).
  - `docs.rs/diesel-async/0.9.0/diesel_async/trait.AsyncConnection.html` (cited in section 2).
- **Lessons:**
  - `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md` - `rg` enumeration before authoring (R11).
  - `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` - no semantic change in closure bodies (R12).
  - `.claude/lessons/feedback_features_full_workspace_only.md` + `feedback_features_full_p_crate_incompatible.md` - DoD discipline.

### Task 2

- **Schema / type definitions:**
  - `Cargo.toml:184` - the `sha2 = "0.10"` line being bumped.
  - `crates/api/api/src/governance/admin_rule_sets.rs:40-130` - full sha2 usage block (production).
  - `crates/server/tests/e2e.rs:930-1050` - one of three test-module sha2 usages (the first; lines 2520-2540 and 7295-7325 follow the same shape).
- **External documentation:**
  - sha2 0.11.0 changelog (cited in section 2).
- **Lessons:**
  - `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md` - pre-flight grep (R11).
  - `.claude/lessons/feedback_lemmy_error_no_std_error.md` - if any test-body edit becomes necessary.

### Task 3

- **Schema / type definitions:**
  - `Cargo.toml:174-242` - every workspace dep this plan touches.
  - `crates/api/api_utils/Cargo.toml` - `jsonwebtoken` line.
  - `crates/email/Cargo.toml` - `lettre` line.
  - `crates/routes/Cargo.toml` - `rss` line.
  - `crates/utils/Cargo.toml` - `dashmap` line.
- **External documentation (Task 3 is Cargo-only; no fresh fetch needed unless `cargo check` surfaces a SemVer-compat regression):**
  - html2text 0.17.1 changelog (cited in section 2; confirms no API change).

## 10. Patterns to mirror

Each entry below cites a specific file:line. No concept-only patterns.

### 10.1 `run_transaction` wrapper signature (Task 1, wrapper edit)

**Mirror:** `crates/diesel_utils/src/connection.rs:60-73` (current wrapper) + `docs.rs/diesel-async/0.9.0/diesel_async/trait.AsyncConnection.html` (target trait bound; cited in section 2 verbatim).

Plan-time text (impl-task verifies the diesel_async re-export path at task-time):

```rust
impl DbConn<'_> {
  /// Run a transaction whose closure body returns `LemmyResult<R>`.
  ///
  /// In diesel-async 0.9 the transaction callback shape moved from
  /// `FnOnce -> ScopedBoxFuture` to a native `AsyncFnOnce` + the
  /// crate-internal `AsyncFunc` helper trait (which constrains the
  /// returned future to be `Send`). Our wrapper keeps the public
  /// `LemmyResult<R>` return and the `R: Send + 'a` bound; the
  /// internal delegation to
  /// `self.deref_mut().transaction::<_, LemmyError, _>(callback).await`
  /// is unchanged. Callers MUST now spell the closure as
  /// `.run_transaction(async |conn| { /* body */ })` - the
  /// `async move { ... }.scope_boxed()` wrapping shape is GONE.
  pub async fn run_transaction<'a, R, F>(&mut self, callback: F) -> LemmyResult<R>
  where
    for<'r> F: AsyncFnOnce(&'r mut AsyncPgConnection) -> LemmyResult<R>
      + diesel_async::AsyncFunc<&'r mut AsyncPgConnection, LemmyResult<R>, Fut: Send>
      + Send
      + 'a,
    R: Send + 'a,
  {
    self
      .deref_mut()
      .transaction::<_, LemmyError, _>(callback)
      .await
  }
}
```

**Why this shape:** preserves the public method name (`run_transaction`), the return type (`LemmyResult<R>`), and the `R: Send + 'a` bound - so every callsite's `let outcome = conn.run_transaction(...).await?;` line is unchanged in shape outside the closure itself. The trait bound block updates to match diesel-async 0.9's `AsyncConnection::transaction` exactly (per section 2 docs.rs cite). The body delegates unchanged.

**Verify at task-time:** the `diesel_async::AsyncFunc` path. If `cargo check` says "cannot find trait `AsyncFunc` in `diesel_async`", try `diesel_async::prelude::AsyncFunc` or grep the published crate for the exact re-export. The trait MUST appear somewhere in the public `diesel_async` API in 0.9 - it's named in the `transaction` signature on docs.rs.

### 10.2 Per-callsite closure rewrite (Task 1, 42 callsites)

**Mirror:** three canonical shapes:

**Shape A - single-call closure** (mirrored from `crates/api/api/src/governance/admin_close_case.rs:43-50`):

```rust
// BEFORE
let outcome = conn
  .run_transaction(|conn| {
    async move { process_close(conn, pseudonym_for_tx, data_for_tx).await }.scope_boxed()
  })
  .await?;

// AFTER
let outcome = conn
  .run_transaction(async |conn| {
    process_close(conn, pseudonym_for_tx, data_for_tx).await
  })
  .await?;
```

**Shape B - multi-statement closure** (mirrored from `crates/apub/activities/src/governance/inbox.rs:196-218`):

```rust
// BEFORE
let outcome = conn
  .run_transaction(|conn| {
    async move {
      acquire_evict_lock(conn, &si_tx, "remote_sanction_notice").await?;
      evict_oldest_unreviewed_if_needed_in_tx(
        &si_tx,
        "remote_sanction_notice",
        evict_cap,
        conn,
      )
      .await?;
      // ... more statements ...
      Ok(SomeOutcome { ... })
    }
    .scope_boxed()
  })
  .await?;

// AFTER
let outcome = conn
  .run_transaction(async |conn| {
    acquire_evict_lock(conn, &si_tx, "remote_sanction_notice").await?;
    evict_oldest_unreviewed_if_needed_in_tx(
      &si_tx,
      "remote_sanction_notice",
      evict_cap,
      conn,
    )
    .await?;
    // ... more statements (verbatim) ...
    Ok(SomeOutcome { ... })
  })
  .await?;
```

**Shape C - db_schema-side trailing `.scope_boxed()`** (mirrored from `crates/db_schema/src/source/governance/governance_log.rs:285-318`):

```rust
// BEFORE
let result = conn
  .run_transaction(|conn| {
    diesel::insert_into(...)
      .values(...)
      .returning(...)
      .get_results::<...>(conn)
      .scope_boxed()
  })
  .await?;

// AFTER
let result = conn
  .run_transaction(async |conn| {
    diesel::insert_into(...)
      .values(...)
      .returning(...)
      .get_results::<...>(conn)
      .await
  })
  .await?;
```

**NOTE on Shape C:** Shape C is the "direct query as closure body" pattern - the `.scope_boxed()` was applied to the diesel future directly, not wrapped in an `async move {}` block. The 0.9 rewrite drops `.scope_boxed()` and adds an explicit `.await` (since `async |conn| { ... }` requires the body to be an expression, not a future). Verify each db_schema Shape C site at task-time - the count of trailing `.scope_boxed()` (without an enclosing `async move {}`) was **8 sites** per plan-author-time enumeration (rg `.scope_boxed\(\)` minus the count of `async move {` wrappers).

**Why these three shapes:** the brief noted "literal find-and-replace pattern" but the actual codebase has at least three syntactic variations. The impl-task uses `rg "\.run_transaction" crates/ tests/ -B1 -A8` at task-time to enumerate every callsite, then classifies by shape (A / B / C), then applies the corresponding rewrite. Per `feedback_multi_write_handlers_need_transactions.md` (R12): the closure body content is preserved byte-for-byte except for the wrapping shape.

### 10.3 `scoped_futures` import collapse (Task 1, 30 import lines)

**Mirror:**
- `crates/api/api/src/governance/admin_close_case.rs:13` (multi-item import group):
  ```rust
  // BEFORE
  use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
  // AFTER
  use diesel_async::RunQueryDsl;
  ```
- `crates/api/api/src/site/registration_applications/approve.rs:4` (sole-item `scoped_futures` import):
  ```rust
  // BEFORE
  use diesel_async::scoped_futures::ScopedFutureExt;
  // AFTER
  // (entire line removed; no replacement)
  ```
- `crates/db_schema/src/source/governance/governance_log.rs:61` (`scoped_futures::ScopedFutureExt` is alongside `RunQueryDsl` in a 2-item group):
  ```rust
  // BEFORE
  diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt},
  // AFTER
  diesel_async::RunQueryDsl,
  ```

**Why this shape:** the `scoped_futures::ScopedFutureExt` is only ever used to call `.scope_boxed()` on a future inside a `.run_transaction(|conn| ...)` callback. Once the Shape A/B/C rewrites land, every `.scope_boxed()` callsite is GONE -> the import is unused -> clippy's `unused_imports` lint fires under `-D warnings`. Removing the import in the same commit avoids a clippy regression. The 30 import-line edits are uniform; `rg "scoped_futures" crates/` at the END of T1 MUST return zero lines (R11 post-check).

### 10.4 sha2 0.11 usage (Task 2 - expected zero-diff)

**Mirror:** `crates/api/api/src/governance/admin_rule_sets.rs:46,120` + `crates/server/tests/e2e.rs:938-1025`.

Existing pattern (preserved unchanged under sha2 0.11):

```rust
use sha2::{Digest, Sha256};

// production (admin_rule_sets.rs:120):
let text_sha256 = Sha256::digest(data.rule_text.as_bytes()).to_vec();

// test (e2e.rs:1020 and other modules):
let mut hasher = Sha256::new();
hasher.update(b"some bytes");
let digest = hasher.finalize();
```

**Why this is zero-diff:** sha2 0.11 keeps the `Digest` trait's `new() / update() / finalize() / digest()` surface on `Sha256` per the changelog (section 2). The type-alias-to-newtype change only matters if code stores `Sha256` in a struct field with a `Digest`-bounded generic parameter - `rg "impl.*Digest" crates/ tests/` at plan-author time returned **zero matches**, so no such storage exists. The test-module `Sha256::new() -> hasher` flow uses the trait method `Digest::new()` which dispatches the same way under both 0.10 and 0.11.

**If sha2 0.11 surfaces a clippy warning or deprecation under `-D warnings`**, the impl-task applies the canonical fix per `feedback_clippy_test_style.md` (typically a one-line `#[allow]` is wrong; prefer renaming or using the suggested replacement). Out-of-scope expansions go to a follow-up DQ.

### 10.5 Cargo.toml version-pin bumps (Task 3 - pattern-uniform)

**Mirror:** every existing version pin in `Cargo.toml`:174-242.

```toml
# BEFORE (workspace root Cargo.toml example)
diesel = { version = "=2.3.7", features = [...], default-features = false }
# AFTER
diesel = { version = "=2.3.9", features = [...], default-features = false }

# BEFORE
tokio = { version = "1.50.0", features = ["full"] }
# AFTER
tokio = { version = "1.52.0", features = ["full"] }
```

**Why this shape:** the version-pin substitution is the only change. `features = [...]` and `default-features = false` blocks are preserved verbatim. `Cargo.lock` regeneration happens automatically when `cargo check --workspace --features full` runs as part of section 15 DoD; impl-task does NOT edit `Cargo.lock` by hand.

## 11. Files to change

Grouped by section 13 task.

### Task 1 (`feat(deps): migrate to diesel-async 0.9 (task 1)`)

**Workspace manifest:**
- `Cargo.toml` (workspace root) - bump `diesel-async = "0.8.0"` -> `diesel-async = "0.9.0"` (1 line edit on line 182; verify latest 0.9.x patch at task-time per brief WP-6).
- `Cargo.lock` - auto-regenerated by `cargo check`.

**Wrapper:**
- `crates/diesel_utils/src/connection.rs` - remove `scoped_futures::ScopedBoxFuture` from the `use diesel_async::{...}` block (line 13); rewrite the `run_transaction` trait bound on lines 61-72 per section 10.1.

**Closure-bearing files (30 files; 41 callsites - wrapper excluded):**

| File | `.run_transaction` count | `scoped_futures` import |
|---|---|---|
| `crates/api/api/src/community/add_mod.rs` | 1 | yes |
| `crates/api/api/src/community/ban.rs` | 1 | yes |
| `crates/api/api/src/community/block.rs` | 1 | yes |
| `crates/api/api/src/community/transfer.rs` | 1 | yes |
| `crates/api/api/src/governance/accept_jury_assignment.rs` | 1 | yes |
| `crates/api/api/src/governance/admin_assign_jury.rs` | 1 | yes |
| `crates/api/api/src/governance/admin_close_case.rs` | 1 | yes |
| `crates/api/api/src/governance/admin_config.rs` | 1 | yes |
| `crates/api/api/src/governance/admin_emergency_remove.rs` | 1 | yes |
| `crates/api/api/src/governance/admin_rule_sets.rs` | 2 | yes |
| `crates/api/api/src/governance/admin_trigger_appeal_rejury.rs` | 1 | yes |
| `crates/api/api/src/governance/decline_jury_assignment.rs` | 1 | yes |
| `crates/api/api/src/governance/federation_outbox.rs` | 3 | (verify at task-time) |
| `crates/api/api/src/governance/reputation_snapshot.rs` | 0 | yes (likely unused; remove) |
| `crates/api/api/src/governance/sponsor_liability_grace.rs` | 2 | yes |
| `crates/api/api/src/governance/submit_jury_vote.rs` | 1 | yes |
| `crates/api/api/src/site/registration_applications/approve.rs` | 1 | yes (sole-item) |
| `crates/api/api_crud/src/governance/create_endorsement.rs` | 1 | yes |
| `crates/api/api_crud/src/governance/create_report.rs` | 1 | yes |
| `crates/api/api_crud/src/governance/request_appeal.rs` | 1 | yes |
| `crates/api/api_crud/src/governance/revoke_endorsement.rs` | 1 | yes |
| `crates/api/api_crud/src/user/create.rs` | 2 (lines 155 + 407 per brief) | yes |
| `crates/apub/activities/src/governance/inbox.rs` | 4 (per re-enumeration; brief said 3) | yes |
| `crates/apub/activities/src/governance/publish_sanction_notice.rs` | 1 | (verify at task-time) |
| `crates/apub/apub/src/governance/outbox.rs` | 1 | (verify at task-time) |
| `crates/db_schema/src/impls/actor_language.rs` | 3 | yes |
| `crates/db_schema/src/impls/community_tag.rs` | 2 | yes |
| `crates/db_schema/src/impls/images.rs` | 1 | yes |
| `crates/db_schema/src/impls/keyword_block.rs` | 1 | yes |
| `crates/db_schema/src/impls/local_site_url_blocklist.rs` | 1 | yes |
| `crates/db_schema/src/source/governance/governance_log.rs` | 1 | yes (subset of `diesel_async` use group) |
| `crates/routes/src/utils/setup_local_site.rs` | 1 | yes |
| **Total callsites (excluding wrapper)** | **41** | **30 imports** |

Plus the wrapper at `crates/diesel_utils/src/connection.rs` (1 `.run_transaction` *definition*, not a callsite; counted as the 42nd `.run_transaction` line in `rg`). **Grand total `.run_transaction` lines = 42** - matches brief and plan-author-time `rg` count.

**Re-enumeration probe at impl-task time** (R11):

```bash
rg "\.run_transaction" crates/ tests/ | wc -l
# EXPECT: 42 (if drift from 42, STOP and file `kind: "blocker"` DQ)

rg "scoped_futures" crates/ | wc -l
# EXPECT: 30 import lines (post-trunk; pre-edit). If drift, recount and revise section 11.

rg "scope_boxed" crates/ tests/ | wc -l
# EXPECT: 38 per-callsite invocations (pre-edit; post-T1 MUST be 0)
```

**Discrepancy note:** brief section 2 enumerates "42 callsites confirmed" and lists ~16+ files. Plan-author-time `rg` returns 42 callsites across 31 files (30 closure-bearing files + the wrapper itself which has 1 method body whose name `run_transaction` matches; impl-task ignores the wrapper-internal line for the 41-callsite count and edits the wrapper signature separately per section 10.1). The brief's per-area counts (e.g. "inbox.rs (3 sites at 197, 318, 638)") differ from plan-author-time count (inbox.rs = 4 sites at 197, 318, 638, 820 per `rg -c`). **R11 mandates re-counting at impl-task time** - drift from 42 total is the failure mode; per-file drift is acceptable (the brief's per-file count is from an earlier trunk SHA).

### Task 2 (`feat(deps): bump sha2 0.10 -> 0.11 (task 2)`)

- `Cargo.toml` (workspace root) - bump `sha2 = "0.10"` -> `sha2 = "0.11"` (1 line edit on line 184).
- `Cargo.lock` - auto-regenerated.
- `crates/api/api/src/governance/admin_rule_sets.rs` - **expected diff: 0 lines** (sha2 0.11 preserves API). Touched only if `cargo check` surfaces a `-D warnings` regression.
- `crates/server/tests/e2e.rs` - **expected diff: 0 lines** at lines 938/2526/7302. Touched only if `cargo check` surfaces a regression.

**Footnote re DQ `a3d0e9941441-016`:** the brief's section 4a table named `-p lemmy_api_common --features full --lib` as the T2 targeted lib-test. Re-verification at plan-author time: the sha2 production callsite lives at `crates/api/api/src/governance/admin_rule_sets.rs` which is in crate `lemmy_api` (per `crates/api/api/Cargo.toml` - confirmed by file path). `lemmy_api_common` does NOT contain sha2 usage (`rg "sha2" crates/api/api_common/src/` returns zero matches). **Plan section 15.4 uses `-p lemmy_api --features full --lib` instead.** Both crates expose a `full` feature (verified per `feedback_features_full_p_crate_incompatible.md` exception class - these are workspace member crates with explicit `full` feature definitions; the lesson's prohibition applies to `lemmy_server` specifically).

### Task 3 (`feat(deps): bump 10 SemVer-compat deps (task 3)`)

- `Cargo.toml` (workspace root) - 6 line edits:
  - `diesel = { version = "=2.3.7", ... }` -> `=2.3.9` (line 174)
  - `tokio = { version = "1.50.0", features = ["full"] }` -> `1.52.0` (line 221)
  - `rustls = { version = "0.23.37", features = ["ring"], default-features = false }` -> `0.23.40` (line 237)
  - `bcrypt = "0.19.0"` -> `0.19.1` (line 211)
  - `serde_with = "3.18.0"` -> `3.20.0` (line 183)
  - `html2text = "0.16.7"` -> `0.17.1` (line 240)

  **Verify exact line numbers at task-time** - the workspace `Cargo.toml` was last touched 2026-05-24 (`git log -1 --format=%h Cargo.toml`); intervening commits may shift line numbers but the dep names are stable.

- `crates/api/api_utils/Cargo.toml` - bump `jsonwebtoken = { version = "10.3.0", features = ["rust_crypto"] }` -> `10.4.0`.
- `crates/email/Cargo.toml` - bump `lettre = { version = "0.11.19", default-features = false, features = [...] }` -> `0.11.22`.
- `crates/routes/Cargo.toml` - bump `rss = "2.0.12"` -> `2.0.13`.
- `crates/utils/Cargo.toml` - bump `dashmap = { version = "6.1.0", optional = true }` -> `6.2.x` (verify latest 6.2.y patch at task-time).
- `Cargo.lock` - auto-regenerated.

**No `crates/**/src/**.rs` edits** - all 10 bumps are SemVer-compatible (per Dependabot's classification) so no consuming-code change should be needed. If `cargo check` surfaces an API regression on any consuming crate, impl-task files `kind: "blocker"` DQ.

### Task 4 (`docs(retro): v1-deps-r1 (task 4)`)

- `.claude/PRPs/reports/v1-deps-r1-retro.md` - NEW.

## 12. NOT building in v1-deps-r1

- **npm/pnpm dep bumps in `api_tests/`** - those land via Dependabot PRs directly (PR #135 pattern, already merged 2026-05-23).
- **Major-version bumps Dependabot has not surfaced** - `rustls 0.24.x`, `tokio 1.53.x` if pre-released, `diesel 2.4.x`, etc. Out of scope; future Dependabot PRs.
- **`governance_log` hash-chain protocol changes** - sha2 0.11 is an API surface bump; the SHA-256 output is byte-identical and the chain remains valid. **No re-emission of historical log entries.**
- **`run_transaction` wrapper refactor beyond what 0.9 requires** - per WP-1 / DQ `a3d0e9941441-013`: keep the same public signature shape (the `LemmyResult<R>` return, `R: Send + 'a` bound, `run_transaction` method name) for callers; only the trait bound on `F` changes.
- **Workspace-wide rustc edition migration** - separate concern (would be its own sub-phase).
- **Removal of the `scripts/brehon/cargo-*` wrappers** - wrappers stay; their behavior is verified by Probes 1-6 in Task 0.
- **Bridging `scoped_futures` for back-compat** - option (b) from WP-1 explicitly rejected per section 4 ratification. `scoped_futures` exits the dependency graph entirely after T1.
- **Updating transitive deps that resolve naturally** - `html5ever 0.39` / `markup5ever 0.39` come along with `html2text 0.17.1`; we do NOT pin them directly. Same for any sha2-transitive (e.g. `digest 0.11`) - let cargo resolve.
- **`.coderabbit.yaml` changes** - CR auto-review runs as configured; the bundled-bumps PR will get full CR coverage on the closure rewrite + the sha2 surface.
- **Pre-emptive deprecation cleanup unrelated to the 12 bumps** - if `cargo check` surfaces a NEW deprecation that wasn't triggered by these bumps (i.e. would have appeared on `governance-v0` HEAD too), file a `kind: "log"` DQ for a follow-up sub-phase; do NOT fold into v1-deps-r1.
- **Per-callsite docstring updates to mention "async closure"** - the closure-shape rewrite is mechanical; no doc-comments are touched. If a callsite has a doc-comment referencing `scope_boxed`, the impl-task DELETES that doc-comment (it now describes a no-longer-existing call shape). `rg "scope_boxed" crates/` at end-of-T1 MUST return zero matches - that includes doc-comments.

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task** (per `feedback_pr_per_phase.md`'s code-only-via-PR rule). T1, T2, T3 are serial (no `[P]`):

- **T2 requires T1** - T2's cargo gate cannot pass until T1's wrapper + callsite rewrite compiles (the workspace must compile before sha2 0.11 resolution can be validated).
- **T3 requires T2** - `Cargo.lock` is monotonic; T3's regenerated lockfile must rebase on T2's resolution.

> **Cohort dispatch:** all four tasks (T0, T1, T2, T3) are non-`[P]` (each is a barrier). T0 is always non-`[P]` per template. T1/T2/T3 each touch `Cargo.toml` (workspace root) -> YAML overlap on the root manifest -> cohort-incompatible per `feedback_explicit_file_arrays_on_tasks.md` even if no Rust file overlap. Serial dispatch only.

### Task 0: Pre-flight harness audit + clippy baseline + callsite re-enumeration

**Goal:** verify environment is ready for `v1-deps-r1`; confirm branch is `phase-v1-deps-r1`; confirm prior phases' deliverables (v1-ship-3 + v1-RT-r2) are intact on the base; confirm pre-existing clippy baseline is clean per DQ `a3d0e9941441-014`; re-enumerate the three `rg` counts (42 / 30 / 38) to confirm no trunk drift since plan-author time.

**FILES (machine-parseable):**

```yaml
creates: []
modifies: []
requires: []
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` - R5: enumerate ALL probes explicitly):**

```bash
# Probe 0 - Docker daemon (capture status to file)
docker ps > /tmp/v1-deps-r1-task0-docker.log 2>&1
status=$?
[ $status -eq 0 ] && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 - branch
git branch --show-current
# EXPECT: phase-v1-deps-r1

# Probe 2 - wrapper sanity: cargo-check honors -p (positive)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/PRPs/debug/v1-deps-r1-task0-cargo-check-p.log 2>&1"
status=$?
tail -20 .claude/PRPs/debug/v1-deps-r1-task0-cargo-check-p.log
[ $status -eq 0 ] || echo "FAIL"
# EXPECT: exit 0; only lemmy_utils compiles

# Probe 3 - wrapper sanity: cargo-check honors --features full (positive)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/PRPs/debug/v1-deps-r1-task0-cargo-check-features.log 2>&1"
status=$?
tail -20 .claude/PRPs/debug/v1-deps-r1-task0-cargo-check-features.log
[ $status -eq 0 ] || echo "FAIL"

# Probe 4 - wrapper sanity: cargo-test honors target selection (positive)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-deps-r1-task0-cargo-test.log 2>&1"
status=$?
tail -20 .claude/PRPs/debug/v1-deps-r1-task0-cargo-test.log
[ $status -eq 0 ] || echo "FAIL"

# Probe 5 - wrapper sanity: non-zero exit propagation (negative)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/v1-deps-r1-task0-cargo-test-negative.log 2>&1"
echo "cargo-test.bat exit on bogus feature: $?"
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/v1-deps-r1-task0-cargo-check-negative.log 2>&1"
echo "cargo-check.bat exit on bogus feature: $?"
# EXPECT: BOTH exits NON-ZERO (typically 101)

# Probe 6 - workspace clippy baseline (the section 15.2 DoD command) - per DQ a3d0e9941441-014
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-deps-r1-task0-clippy-baseline.log 2>&1"
status=$?
tail -40 .claude/PRPs/debug/v1-deps-r1-task0-clippy-baseline.log
[ $status -eq 0 ] || echo "FAIL: insert chore(lint) task before T1"

# Probe 7 - workspace e2e --no-run baseline (the section 15.3 DoD command)
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-deps-r1-task0-e2e-no-run-baseline.log 2>&1"
status=$?
tail -20 .claude/PRPs/debug/v1-deps-r1-task0-e2e-no-run-baseline.log
[ $status -eq 0 ] || echo "FAIL"

# Probe 8 - callsite re-enumeration (R11 pre-flight) — use file capture for wc
rg "\.run_transaction" crates/ tests/ > /tmp/probe8a.txt; wc -l /tmp/probe8a.txt
# EXPECT: 42 (if drift, STOP and file `kind: "blocker"` DQ before T1)

rg "scoped_futures" crates/ > /tmp/probe8b.txt; wc -l /tmp/probe8b.txt
# EXPECT: 30 import lines (pre-T1)

rg "scope_boxed" crates/ tests/ > /tmp/probe8c.txt; wc -l /tmp/probe8c.txt
# EXPECT: 38 per-callsite invocations (pre-T1)

rg "Sha256\b" crates/ tests/ > /tmp/probe8d.txt; wc -l /tmp/probe8d.txt
# EXPECT: 8 (5 per brief + 3 trunk-drift cushion)

rg "impl.*Digest" crates/ tests/ > /tmp/probe8e.txt; wc -l /tmp/probe8e.txt
# EXPECT: 0 (R12 for T2 - if non-zero, T2 scope expands; file `kind: "blocker"` DQ)

rg -l "html2text" crates/
# EXPECT: 2 files (`crates/email/src/send.rs` and `crates/apub/objects/src/objects/post.rs`)

# Probe 9 - prior-phase deliverable presence (Hard precondition from brief section 0)
git log --oneline origin/governance-v0 > /tmp/probe9.txt
head -5 /tmp/probe9.txt
# EXPECT: v1-ship-3 merge commit AND v1-RT-r2 merge commit AND both retro commits visible

ls .claude/PRPs/reports/v1-ship-3-retro.md
# EXPECT: file exists

ls .claude/PRPs/reports/v1-RT-r2-retro.md
# EXPECT: file exists (verify exact name at task-time; may be slightly different)

# Probe 10 - concurrent-PR check (capture to file, then process)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files > /tmp/probe10.json
python3 -c "import json; data=json.load(open('/tmp/probe10.json')); [print(p['number'], p['title'], p['headRefName']) for p in data if any(f['path'] in ('Cargo.toml','Cargo.lock','crates/diesel_utils/src/connection.rs') for f in p.get('files',[]))]"
# EXPECT: empty output

# Probe 11 - DQ a3d0e9941441-015 boundary check (Shape G suspension boundary)
date +%s
# EXPECT: print current epoch; if past 2026-06-01 00:00 UTC (1748736000), advisor files `kind: "log"` DQ per DQ a3d0e9941441-015 answer (a)
```

**EXPECT block:**
- Probes 0-4, 6-11 exit 0 (Probe 5 = NEGATIVE test, both lines NON-ZERO)
- Probe 8 returns the exact counts in EXPECT lines; any drift triggers a `kind: "blocker"` DQ before T1.

**No commit at Task 0** - this is verification only.

---

### Task 1: diesel-async 0.9 migration (wrapper + 42 callsites + 30 import collapses)

**ACTION:** Bump `diesel-async 0.8.0 -> 0.9.0` in workspace `Cargo.toml`; rewrite `DbConn::run_transaction` trait bound in `crates/diesel_utils/src/connection.rs` per section 10.1; mechanically rewrite all 42 callsites (Shapes A/B/C per section 10.2) and 30 `scoped_futures::ScopedFutureExt` imports per section 10.3. **Single commit.** Mention `/edit-mechanical` skill in this task wording - the rg-enumerate-first discipline is the precondition for safe mechanical sweeps at this scale.

**FILES (machine-parseable):**

```yaml
creates: []
modifies:
  - Cargo.toml
  - Cargo.lock
  - crates/diesel_utils/src/connection.rs
  - crates/api/api/src/community/add_mod.rs
  - crates/api/api/src/community/ban.rs
  - crates/api/api/src/community/block.rs
  - crates/api/api/src/community/transfer.rs
  - crates/api/api/src/governance/accept_jury_assignment.rs
  - crates/api/api/src/governance/admin_assign_jury.rs
  - crates/api/api/src/governance/admin_close_case.rs
  - crates/api/api/src/governance/admin_config.rs
  - crates/api/api/src/governance/admin_emergency_remove.rs
  - crates/api/api/src/governance/admin_rule_sets.rs
  - crates/api/api/src/governance/admin_trigger_appeal_rejury.rs
  - crates/api/api/src/governance/decline_jury_assignment.rs
  - crates/api/api/src/governance/federation_outbox.rs
  - crates/api/api/src/governance/reputation_snapshot.rs
  - crates/api/api/src/governance/sponsor_liability_grace.rs
  - crates/api/api/src/governance/submit_jury_vote.rs
  - crates/api/api/src/site/registration_applications/approve.rs
  - crates/api/api_crud/src/governance/create_endorsement.rs
  - crates/api/api_crud/src/governance/create_report.rs
  - crates/api/api_crud/src/governance/request_appeal.rs
  - crates/api/api_crud/src/governance/revoke_endorsement.rs
  - crates/api/api_crud/src/user/create.rs
  - crates/apub/activities/src/governance/inbox.rs
  - crates/apub/activities/src/governance/publish_sanction_notice.rs
  - crates/apub/apub/src/governance/outbox.rs
  - crates/db_schema/src/impls/actor_language.rs
  - crates/db_schema/src/impls/community_tag.rs
  - crates/db_schema/src/impls/images.rs
  - crates/db_schema/src/impls/keyword_block.rs
  - crates/db_schema/src/impls/local_site_url_blocklist.rs
  - crates/db_schema/src/source/governance/governance_log.rs
  - crates/routes/src/utils/setup_local_site.rs
requires: []
```

**IMPLEMENT discipline (per `/edit-mechanical` semantics):**

**Step 1 - Enumerate (R11 + R12 pre-flight)** — write rg output to files, then wc:

```
rg "\.run_transaction" crates/ tests/ -n > .claude/PRPs/debug/v1-deps-r1-task1-callsites.txt
wc -l .claude/PRPs/debug/v1-deps-r1-task1-callsites.txt
# EXPECT: 42

rg "scoped_futures" crates/ -n > .claude/PRPs/debug/v1-deps-r1-task1-scoped-futures.txt
wc -l .claude/PRPs/debug/v1-deps-r1-task1-scoped-futures.txt
# EXPECT: 30

rg "scope_boxed" crates/ tests/ -n > .claude/PRPs/debug/v1-deps-r1-task1-scope-boxed.txt
wc -l .claude/PRPs/debug/v1-deps-r1-task1-scope-boxed.txt
# EXPECT: 38
```

**If ANY of the three counts drifts from 42 / 30 / 38**, STOP and file `kind: "blocker"` DQ. Do NOT proceed.

**Step 2 - Bump `Cargo.toml`:**

```toml
# Cargo.toml line 182
diesel-async = "0.9.0"      # (verify latest 0.9.x patch at task-time - prefer latest)
```

**Step 3 - Rewrite wrapper** at `crates/diesel_utils/src/connection.rs` per section 10.1.

**Step 4 - Rewrite callsites file-by-file**, in the order from section 11 (alphabetical by path). For each file:

1. Read the file fully (Junior MUST Read before Edit - Brehon hard rule).
2. Identify the `.run_transaction(...)` block(s) by `rg "\.run_transaction" <file> -n`.
3. Identify the `scoped_futures` import line by `rg "scoped_futures" <file> -n` (may be zero if the file uses it indirectly via the wrapper alone - unlikely but possible).
4. For each `.run_transaction(...)` block: classify as Shape A / B / C per section 10.2; apply the corresponding rewrite. The Edit's `old_string` MUST include enough context to make the location unique within the file (typically 3-5 lines). `new_string` re-emits the same context with the closure shape rewritten.
5. For the import line: rewrite per section 10.3 (multi-item: drop `scoped_futures::ScopedFutureExt` from the use group; sole-item: delete the entire `use` line).
6. After EVERY file edit, verify `rg "scope_boxed" <file>` returns 0 matches (in-file `scope_boxed` should be gone post-edit).

**Step 5 - Post-edit pre-push checks (mandatory per `feedback_fix_impl_pre_push_cargo_check.md`):**

```bash
# All scope_boxed gone
rg "scope_boxed" crates/ tests/ > /tmp/postcheck-a.txt; wc -l /tmp/postcheck-a.txt
# EXPECT: 0

# All scoped_futures imports gone
rg "scoped_futures" crates/ > /tmp/postcheck-b.txt; wc -l /tmp/postcheck-b.txt
# EXPECT: 0

# All .run_transaction lines still equal 42 (no accidental deletions)
rg "\.run_transaction" crates/ tests/ > /tmp/postcheck-c.txt; wc -l /tmp/postcheck-c.txt
# EXPECT: 42

# Cargo check passes
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-deps-r1-task1-check.log 2>&1"
status=$?
tail -20 .claude/PRPs/debug/v1-deps-r1-task1-check.log
[ $status -eq 0 ] || exit $status
```

**Step 6 - Commit + push:**

Commit subject: `feat(deps): migrate to diesel-async 0.9 (task 1)`. Body lists the three counts (42 / 30 / 38 -> 42 / 0 / 0), the `Cargo.toml` bump (0.8.0 -> 0.9.x), and the trait-bound change. **`HANDOVER:` trailer** summarises for T2: "wrapper + callsites done; scoped_futures fully purged; workspace check exit 0; Cargo.lock regenerated with diesel-async 0.9.x lockfile entry."

Push branch. Raise `kind: "validate-pending-laptop-e2e"` DQ entry per DQ `a3d0e9941441-016` with `commands[]` containing the wrapper-prefixed cargo check, clippy, three lib-tests (`-p lemmy_api`, `-p lemmy_apub`, `-p lemmy_db_schema` all `--features full --lib`), and the full e2e gate (`--workspace --test e2e --features full`). All commands log to `.claude/PRPs/debug/v1-deps-r1-task1-*.log`. Each command captures status separately (per `feedback_cargo_output_capture.md` discipline).

**MIRROR:**
- Section 10.1 (wrapper signature)
- Section 10.2 Shape A / B / C (per-callsite rewrites)
- Section 10.3 (import collapse)
- v1-ship-3 Task 2 (post-tx structural pattern; similar single-commit multi-crate edit; same impl-task brief shape)

**GOTCHA:**
- **`diesel_async::AsyncFunc` re-export path:** at task-time, the impl-task verifies the exact public path for the `AsyncFunc` trait (it's named in `AsyncConnection::transaction`'s signature on docs.rs but the re-export path may be `diesel_async::AsyncFunc` or `diesel_async::prelude::AsyncFunc` or via a feature flag). If `cargo check` says "cannot find trait `AsyncFunc`", check `crates.io/crates/diesel-async/0.9.0` source for the re-export.
- **Shape C closures (direct future, no `async move {}` wrapper)** - the rewrite drops `.scope_boxed()` AND adds an explicit `.await` since `async |conn| { expr }` requires the body to be an expression evaluating to `LemmyResult<R>`, not a future. Mis-applying Shape C as Shape A (wrapping in an unnecessary `{}` block) compiles but is stylistically wrong; mis-applying Shape A as Shape C (dropping the `async move` outer block) drops the wrapping `Ok(...)` line and breaks compilation. Per section 10.2 NOTE: enumerate Shape C count separately.
- **`reputation_snapshot.rs` imports `scoped_futures` but has 0 `.run_transaction` callsites** - verify at task-time whether the import is genuinely unused (the wrapper is used indirectly via `.transaction(...)` on raw `AsyncPgConnection`?). If unused, drop the import; if used indirectly, file `kind: "blocker"` DQ - that's a Shape D the plan didn't anticipate.
- **No `crates/server/tests/e2e.rs` edits in T1** - verified at plan-author time (0 `.run_transaction` callsites in e2e.rs). The e2e edit-hang concern is non-applicable.
- **Per `feedback_multi_write_handlers_need_transactions.md` (R12):** the closure body content is preserved byte-for-byte. If a per-file Edit's `new_string` semantically differs from `old_string` beyond removing the `.scope_boxed()` wrapper, STOP and file `kind: "blocker"` DQ. The mechanical sweep MUST be byte-recognisable as "same closure body, different wrapping shape".
- **`Cargo.lock` regenerates on first `cargo check` post-bump.** Expect `Cargo.lock` to be in the commit diff with ~70 added / ~95 removed lines (per brief estimate). If `Cargo.lock` diff exceeds +/-200 lines, surface as a `kind: "log"` DQ for verification (unexpected transitive resolution drift).
- **`futures_util::FutureExt::boxed`** is still used inside `crates/diesel_utils/src/connection.rs:220` (`fut.boxed()` in `establish_connection`). This is `BoxFuture`, NOT `ScopedBoxFuture`. **Leave it alone.** The brief's WP-1 mention of "fut.boxed() (inside with_isolation_level helper)" is incorrect - there is no `with_isolation_level` helper in this codebase; the line at 220 is inside `establish_connection`. Verified at plan-author time.

**VALIDATE (story-checkpoint feeds section 16a Story 1):** See the `commands[]` description in Step 6 above. Per DQ `a3d0e9941441-016`, T1 gets the full check + clippy + per-crate lib-test + e2e treatment because it's the bulk of the migration's risk surface.

---

### Task 2: sha2 0.11 migration (narrow surface; expected zero-diff in `crates/**`)

**ACTION:** Bump `sha2 0.10 -> 0.11` in workspace `Cargo.toml`. Pre-flight verify `rg "impl.*Digest" crates/ tests/` returns zero matches. Run `cargo check --workspace --features full`. **Expected diff:** `Cargo.toml` (1 line) + `Cargo.lock` (~5-10 lines for the digest 0.11 transitive). If `cargo check` or `cargo clippy --no-deps -- -D warnings` regresses, the impl-task applies the minimal canonical fix per section 10.4 / `feedback_clippy_test_style.md`.

**FILES (machine-parseable):**

```yaml
creates: []
modifies:
  - Cargo.toml
  - Cargo.lock
  - crates/api/api/src/governance/admin_rule_sets.rs
  - crates/server/tests/e2e.rs
requires:
  - task: 1
    reason: "T2's `cargo check --workspace --features full` cannot run cleanly until T1's diesel-async 0.9 migration has compiled - the workspace must be in a valid base state before sha2 0.11 is added on top. T1's commit MUST land before T2's branch is cut from the phase tip."
```

**IMPLEMENT:**

**Step 1 - Pre-flight verification (R12)** — write rg output to files, then process:

```
rg "impl.*Digest" crates/ tests/ -n > .claude/PRPs/debug/v1-deps-r1-task2-impl-digest.txt
wc -l .claude/PRPs/debug/v1-deps-r1-task2-impl-digest.txt
# EXPECT: 0 (zero explicit Digest impls)

# Also check no Cargo.toml enables sha2's std feature (removed in 0.11)
rg "sha2.*features.*std" Cargo.toml crates/*/Cargo.toml > /tmp/sha2-std-check.txt
wc -l /tmp/sha2-std-check.txt
# EXPECT: 0 (empty output)
```

**If `impl.*Digest` returns non-zero**, STOP and file `kind: "blocker"` DQ - scope expands. If `sha2.*features.*std` returns non-empty, file `kind: "blocker"` DQ - the `std` feature removal affects compile.

**Step 2 - Bump `Cargo.toml`:**

```toml
# Cargo.toml line 184
sha2 = "0.11"      # (verify latest 0.11.x at task-time)
```

**Step 3 - Run `cargo check --workspace --features full`** (per section 15.1). If exit 0 + zero warnings under `-D warnings` (per `cargo clippy --no-deps -- -D warnings`), T2 is done; commit.

**Step 4 - If `cargo check` regresses** (e.g. a deprecation warning, or a `E0277` on the newtype change), the impl-task investigates:
  - For each compile error: read the cited file:line, identify the API surface, apply the minimal fix per section 10.4. Typically: rename a type alias to its new newtype constructor, or update an `impl` block to use the new method.
  - For deprecation warnings: read `feedback_clippy_test_style.md`; apply the suggested replacement (NOT `#[allow]`). If the replacement requires significant refactor, file `kind: "blocker"` DQ.
  - Re-run `cargo check` + `cargo clippy` after each fix; if 3+ cycles without convergence, file `kind: "blocker"` DQ per `feedback_principles_not_rules.md` cycle-count meta-rule.

**Step 5 - Commit + push.**

Commit subject: `feat(deps): bump sha2 0.10 -> 0.11 (task 2)`. Body notes: pre-flight `rg "impl.*Digest"` = 0; expected diff: `Cargo.toml` + `Cargo.lock`; touched code files = X (typically 0). **`HANDOVER:` trailer** summarises for T3: "sha2 0.11 in; SHA-256 algorithm output byte-identical (ADR-012 preserved); workspace check + clippy + lib-test green."

Push branch. Raise `kind: "validate-pending-laptop"` DQ entry per DQ `a3d0e9941441-016` (T2 class: narrow) with `commands[]` = [wrapper-prefixed check, wrapper-prefixed clippy `--no-deps -- -D warnings`, wrapper-prefixed `-p lemmy_api --features full --lib`]. All commands log to `.claude/PRPs/debug/v1-deps-r1-task2-*.log`; each captures status separately.

**No e2e gate at T2** - that runs once at the phase-tip post-T3 (per DQ `a3d0e9941441-016` answer (a) class-targeted design; saves ~26 min lane-time).

**MIRROR:** section 10.4 (sha2 0.11 usage - expected zero-diff). v1-ship-3 Task 1 (single-file YAML edit; similar low-touch shape) for the "expected-zero-diff" pattern.

**GOTCHA:**
- **The `Digest` trait's `digest()` method takes `&[u8]` and returns the byte-array.** sha2 0.11 preserves this signature on `Sha256`. If `cargo check` says the method is missing, verify the import is `use sha2::{Digest, Sha256};` (NOT `use sha2::Sha256;` alone - `Digest` is required to bring the method into scope).
- **The 3 e2e test modules each have their own `use sha2::{Digest, Sha256};`** - they're inside `mod` blocks at e2e.rs:938, 2526, 7302. Each is independent; no module exports `Sha256` to siblings. If T2 needs to edit one, it likely needs to edit all three (uniform pattern).
- **`Sha256::new()` returns the `Sha256` type** (a newtype in 0.11; was a type alias in 0.10). The `let mut hasher = Sha256::new();` pattern works identically because both type alias and newtype expose the same `new()` constructor via the `Digest` trait.
- **No `Cargo.toml` cleanup needed** beyond the version pin (`sha2 = "0.11"`). The brief mentioned removing the `std` feature; pre-flight grep confirmed no Cargo.toml enables `sha2.features = ["std"]`, so this is a no-op.
- **The governance_log hash chain (ADR-012)** is preserved byte-identically. sha2 0.11 keeps the SHA-256 algorithm; only Rust's type wrapping changed. No re-emission of historical log entries; no governance side-effect.

**VALIDATE (story-checkpoint feeds section 16a Story 2):** See `commands[]` description in Step 5.

---

### Task 3: SemVer-compatible bundle (10 bumps; Cargo-only edits)

**ACTION:** Bump 6 workspace deps + 4 sub-crate deps per section 4 Task 3. Pure `Cargo.toml` + `Cargo.lock` change; no `crates/**/src/**.rs` edits expected. If any of the 10 bumps surfaces a SemVer-compat regression on a consuming crate, the impl-task files `kind: "blocker"` DQ (do NOT fold into v1-deps-r1; that's a follow-up sub-phase per section 12).

**FILES (machine-parseable):**

```yaml
creates: []
modifies:
  - Cargo.toml
  - Cargo.lock
  - crates/api/api_utils/Cargo.toml
  - crates/email/Cargo.toml
  - crates/routes/Cargo.toml
  - crates/utils/Cargo.toml
requires:
  - task: 2
    reason: "T3's Cargo.lock regeneration must rebase on T2's sha2 0.11 resolution + T1's diesel-async 0.9 resolution. Bundling T3's bumps onto a stale T1-or-T2 lockfile risks transitive-resolution conflicts the planner did not anticipate. Serial dispatch enforces this dependency mechanically."
```

**IMPLEMENT:**

**Step 1 - Verify latest patch for each dep at task-time** (per brief WP-6) — for each dep `cargo search <name>` and capture output to a file; pick the latest matching version. Repeat for: diesel, tokio, rustls, bcrypt, serde_with, jsonwebtoken, lettre, rss, dashmap, html2text.

If any dep has a NEWER patch than the brief specified (e.g. `tokio 1.53.x` if released), bump to the newer patch per brief WP-6 ("Bring the bumps to the latest available versions at lane-cut time").

**Step 2 - Edit workspace `Cargo.toml`:** apply the 6 line edits per section 4 Task 3 (verify exact line numbers at task-time via `grep -n "<dep>" Cargo.toml`).

**Step 3 - Edit each sub-crate `Cargo.toml`:** apply the 4 sub-crate edits per section 4 Task 3 (verify exact line numbers at task-time via `grep -n "<dep>" crates/<crate>/Cargo.toml`).

**Step 4 - Run `cargo check --workspace --features full`:**

```
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-deps-r1-task3-check.log 2>&1"
status=$?
tail -40 .claude/PRPs/debug/v1-deps-r1-task3-check.log
[ $status -eq 0 ] || exit $status
```

**If any of the 10 bumps surfaces an API regression**, file `kind: "blocker"` DQ. Specifically watch:
- `tokio 1.52` - verify no `tokio::stream` / `tokio::sync` API churn affects consuming crates.
- `rustls 0.23.40` - verify no signature-verifier or `ClientConfig` builder API change (we have a custom `NoCertVerifier` at `crates/diesel_utils/src/connection.rs:224`).
- `html2text 0.17.1` - per section 2 cite, no user-visible API change is expected. Verify `crates/email/src/send.rs` and `crates/apub/objects/src/objects/post.rs` still compile.

**Step 5 - Run `cargo clippy --workspace --features full --no-deps -- -D warnings`:**

```
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-deps-r1-task3-clippy.log 2>&1"
status=$?
tail -40 .claude/PRPs/debug/v1-deps-r1-task3-clippy.log
[ $status -eq 0 ] || exit $status
```

**Step 6 - Commit + push.**

Commit subject: `feat(deps): bump 10 SemVer-compatible deps (task 3)`. Body lists the 10 bumps and their versions (final post-WP-6 versions, not brief versions if they drifted). **`HANDOVER:` trailer:** "10 SemVer-compat bumps in; workspace check + clippy green; Cargo.lock delta ~X added / ~Y removed lines."

Push branch. Raise `kind: "validate-pending-laptop"` DQ entry per DQ `a3d0e9941441-016` (T3 class: Cargo-only) with `commands[]` = [wrapper-prefixed check, wrapper-prefixed clippy `--no-deps -- -D warnings`]. All commands log to `.claude/PRPs/debug/v1-deps-r1-task3-*.log`.

**Phase-tip e2e gate (post-T3, pre-PR; per DQ `a3d0e9941441-016` answer (a)):** advisor raises a NEW `kind: "validate-pending-laptop-e2e"` entry **from advisor session** (not impl) referencing the post-T3 phase-branch tip, with the single-command wrapper-prefixed full workspace e2e (`--workspace --test e2e --features full`), logging to `.claude/PRPs/debug/v1-deps-r1-phase-tip-e2e.log`. **Result MUST be `pass`** (no skips, no flakes) before opening the PR.

**MIRROR:** section 10.5 (Cargo.toml pin pattern). v1-quality-r1 Phase 2 (`.rustfmt.toml` + reformat-bulk commit) for the "Cargo-only mechanical edit + auto-regenerated artefact" shape.

**GOTCHA:**
- **`diesel 2.3.7 -> 2.3.9`** includes a `#[derive(AsChangeset)]` regression fix per brief section 1. Verify no `AsChangeset`-using struct (rg `derive.*AsChangeset` crates/) is affected; if any consuming crate breaks, file `kind: "blocker"` DQ.
- **`tokio 1.52`** is a minor bump (1.50 -> 1.52). Verify no `tokio::time` / `tokio::task` / `tokio::sync` API changes affect us. `rg "tokio::time::sleep\|tokio::task::spawn_blocking\|tokio::sync" crates/` enumerates touchpoints.
- **`rustls 0.23.40`** is a security patch. Verify the `NoCertVerifier` impl at `crates/diesel_utils/src/connection.rs:224-272` (which we use for `sslmode=require`) still compiles against the 0.23.40 trait surface.
- **`html2text 0.17.1`** pulls `html5ever 0.39` transitively. Per section 2 cite (changelog confirmed), no user-visible API change. The 2 callers (`crates/email/src/send.rs` + `crates/apub/objects/src/objects/post.rs`) should compile unchanged.
- **`Cargo.lock`** regenerates with all 12 bumps (10 from T3 + diesel-async/sha2 carrying forward). Expect ~70 added / ~95 removed lines per brief. If delta exceeds +/-200 lines, surface as `kind: "log"` DQ.
- **`dashmap 6.1 -> 6.2`** - verify the `crates/utils/src/...` consumers compile. The bump is minor; no known breaking change.
- **No `Cargo.toml` feature flag drift** - preserve each dep's `features = [...]` and `default-features = false` configuration verbatim during the version bump.

**VALIDATE (story-checkpoint feeds section 16a Story 3 + Story 4):** See `commands[]` description in Step 6 + the phase-tip e2e gate.

---

### Task 4: Retro

**Goal:** author retro per `feedback_retro_not_report.md` and `feedback_four_role_retro_signals.md`. One H2 per role (Advisor / Planning / Impl / BM) with signals + lessons. Promote any new lessons to `.claude/lessons/feedback_*.md` in the same retro commit (per `feedback_one_system_memory_in_repo.md`).

**FILES (machine-parseable):**

```yaml
creates:
  - .claude/PRPs/reports/v1-deps-r1-retro.md
modifies: []
requires:
  - task: 1
    reason: "Retro reads outcomes of Tasks 1-3."
  - task: 2
    reason: "Retro reads outcomes of Tasks 1-3."
  - task: 3
    reason: "Retro reads outcomes of Tasks 1-3."
```

**Per-task complexity score** (`feedback_retro_task_complexity_score`) - author each entry as `<files>/<commits>/<runtime-min>/<max-log-silence-min>`. Aggregate in section 5.

**Retro structure:**

- Section 1 Brief / Plan refs
- Section 2 What surprised us / what to change / what to carry forward
- Section 3 Four-role signals - Advisor / Planning / Impl / BM (one H2 per role)
- Section 4 Promote lessons (any new `feedback_*.md` authored this phase). Three candidate lessons to evaluate:
  - **(a)** `feedback_dep_bump_bundle_single_phase.md` - if bundling diesel-async breaking + sha2 breaking + 10 SemVer-compat into one phase saved measurable lane-time vs three separate phases, promote.
  - **(b)** `feedback_async_closure_mechanical_sweep.md` - if T1's 42-callsite mechanical sweep proved that `/edit-mechanical`-style rg-enumerate-first + per-file Edit + post-sweep `rg` verification is the right shape for trait-bound breaking-change migrations, promote.
  - **(c)** `feedback_dep_bump_workspace_lockfile_delta.md` - if the `Cargo.lock` delta surprised the planner (>+/-200 lines vs the brief's ~70/-95 estimate), promote a recipe for "expected Cargo.lock delta per cohort of N dep bumps".
- Section 5 Per-task complexity scores
- Section 6 Watch-items for next sub-phase (any inherited from this phase's friction)

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** Tasks 1, 2, 3 each trigger `cargo check --workspace --features full`. Compile-time check is THE gating signal.
- **Workspace check:** `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full"` after every Rust-touching task (T1, T2; T3 is Cargo-only but still re-runs the check because lockfile may shift).
- **Lint:** `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"` after every task. Mandatory `--no-deps` per R6 to avoid upstream lint debt.
- **Test target compile:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run"` after T1 (which touches public type signatures) and after T3 (which may transitively shift e2e link). Optional after T2 (sha2 surface in tests is narrow; lib-test covers it).
- **Per-crate lib-tests (T1, T2):**
  - T1: `-p lemmy_api`, `-p lemmy_apub`, `-p lemmy_db_schema` `--features full --lib` (per DQ `a3d0e9941441-016` class-targeted).
  - T2: `-p lemmy_api --features full --lib` (sha2 production callsite home).
- **Phase-tip e2e (post-T3):** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"` - full e2e suite. Single gate per DQ `a3d0e9941441-016` saves ~50 min lane-time vs three per-task e2e runs. Result MUST be `pass` (no skips, no flakes).
- **Migration round-trip:** N/A - no new migrations.
- **`Cargo.lock` delta sanity:** post-T3, `git diff governance-v0...phase-v1-deps-r1 -- Cargo.lock` line count should be ~165 +/- 100 lines (brief estimated 70/-95 = ~165 line-delta). If wildly different (>500), file `kind: "log"` DQ for verification.

---

## 15. Validation commands (DoD)

> **Mode:** `validate-pending-laptop` (Shape G SUSPENDED until 2026-06-01 per DQ #229 + `project_shape_g_suspended_2026_05_16`). Commands run on the laptop advisor session, NOT on GH Actions. Per `feedback_laptop_default_for_validate_pending.md` + `feedback_validate_pending_laptop_must_use_wrapper.md` + DQ `a3d0e9941441-011` + DQ `a3d0e9941441-015`.
>
> **Planner-side discipline** (per `feedback_plan_dod_dry_run_at_write.md` + `feedback_pre_phase_dod_smoke_test.md`): every command in this section MUST be dry-run by the advisor against current HEAD (= `governance-v0` tip at lane-cut time) before plan approval. Unexecutable commands are advisor-side rejection grounds. See section 19 Notes for the planner's dry-run results.

### 15.1 Static analysis (per task)

```
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-deps-r1-<task>-check.log 2>&1"
status=$?
[ $status -eq 0 ] || exit $status
# EXPECT: status 0
```

### 15.2 Lint (per task - uniform R6)

```
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-deps-r1-<task>-clippy.log 2>&1"
status=$?
[ $status -eq 0 ] || exit $status
# EXPECT: status 0
```

### 15.3 Test target compile (R7 - per task touching a struct or re-export)

T1 touches `DbConn::run_transaction` (public method). T2 touches sha2 callsites only (no public surface change). T3 is Cargo-only.

```
# After T1 and T3 only:
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-deps-r1-<task>-test-no-run.log 2>&1"
status=$?
[ $status -eq 0 ] || exit $status
# EXPECT: status 0
```

### 15.4 Per-crate lib-tests (class-targeted per DQ `a3d0e9941441-016`)

```
# T1: governance crates (lib tests in 3 crates)
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_api --features full --lib > .claude/PRPs/debug/v1-deps-r1-task1-libtest-api.log 2>&1"
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_apub --features full --lib > .claude/PRPs/debug/v1-deps-r1-task1-libtest-apub.log 2>&1"
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_db_schema --features full --lib > .claude/PRPs/debug/v1-deps-r1-task1-libtest-db_schema.log 2>&1"
# EXPECT: each exit 0; capture status after each via $? and propagate non-zero

# T2: sha2 surface (1 crate)
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_api --features full --lib > .claude/PRPs/debug/v1-deps-r1-task2-libtest.log 2>&1"
# EXPECT: exit 0

# T3: no lib-test (Cargo-only)
```

### 15.5 e2e per-task + phase-tip (class-targeted per DQ `a3d0e9941441-016`)

```
# T1: full e2e (closure-shape rewrite touches the production write path)
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-deps-r1-task1-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-deps-r1-task1-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-deps-r1-task1-e2e.log"
# EXPECT: exit 0; "N passed; 0 failed" in tail (N = current e2e test count)

# T2: SKIP per-task e2e (sha2 narrow; lib-test covers)
# T3: SKIP per-task e2e

# Phase-tip (post-T3): single full e2e gate
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-deps-r1-phase-tip-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-deps-r1-phase-tip-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-deps-r1-phase-tip-e2e.log"
# EXPECT: exit 0; "N passed; 0 failed" in tail
```

### 15.6 Cross-cutting verification

- [ ] No file outside section 11 list edited. Verify: `git diff --stat governance-v0...HEAD` enumerates ONLY the files in section 11 across T1+T2+T3 + `.claude/PRPs/reports/v1-deps-r1-retro.md` for T4.
- [ ] R5: Task 0 enumerated all 12 probes (Probes 0-11).
- [ ] R6: every clippy invocation in section 15.2 uses `--no-deps` AND `--features full`.
- [ ] R7: test-target compile (section 15.3) runs after T1 (touches `DbConn::run_transaction`) and T3 (lockfile shift). T2 skipped (narrow surface).
- [ ] R11: callsite enumeration verified at end of T1 - `rg "\.run_transaction" crates/ tests/ | wc -l` returns 42; `rg "scoped_futures" crates/ | wc -l` returns 0; `rg "scope_boxed" crates/ tests/ | wc -l` returns 0.
- [ ] R12: closure body content preserved byte-for-byte across T1's mechanical sweep - manual spot-check of 3 random callsites at PR-open time confirms no semantic edit beyond the closure-shape rewrite.
- [ ] R13: workspace `Cargo.toml` + `Cargo.lock` regeneration land in T1 (for diesel-async), T2 (for sha2), T3 (for the 10-bump bundle) - no split across commits.
- [ ] Audit: `rg "ScopedBoxFuture\|scoped_futures" crates/` returns 0 matches at phase-tip.
- [ ] Audit: `Cargo.toml` workspace dep block contains `diesel-async = "0.9.x"`, `sha2 = "0.11"`, and the 6 T3 bumps at the verified versions.
- [ ] Audit: `Cargo.lock` delta is bounded (within ~200 lines of brief estimate of ~70/-95 = ~165 line-delta).
- [ ] ADR-012 preserved: `sha2` hash chain output byte-identical (no governance_log re-emission needed; verified by phase-tip e2e - if a hash-chain test exists and passes, ADR-012 holds).

---

## 16. Acceptance criteria

- [ ] All 5 tasks completed in dependency order (Task 0 audit, Tasks 1-3 impl, Task 4 retro)
- [ ] Section 15.1 (cargo check workspace) exit 0 after Tasks 1 + 2 + 3
- [ ] Section 15.2 (cargo clippy `--no-deps -- -D warnings`) exit 0 after Tasks 1 + 2 + 3
- [ ] Section 15.3 (cargo test --no-run) exit 0 after Tasks 1 + 3
- [ ] Section 15.4 (per-crate lib-tests) exit 0 for T1 (3 crates) and T2 (1 crate)
- [ ] Section 15.5 (per-task + phase-tip e2e) exit 0 - both T1's per-task e2e and the phase-tip composite e2e pass
- [ ] Section 15.6 (cross-cutting verification) - all 10 boxes ticked
- [ ] Section 16a stories - all 4 stories `[done]`
- [ ] No edits to files outside section 11 list
- [ ] Retro committed per section 13 Task 4
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-deps-r1-verify.md` shows all stories OK
- [ ] Manual audit re-run: `rg "ScopedBoxFuture\|scoped_futures" crates/` returns 0; `grep -E '^diesel-async' Cargo.toml` returns `diesel-async = "0.9.x"`; `grep -E '^sha2' Cargo.toml` returns `sha2 = "0.11"`.

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: `diesel-async 0.9` migration - wrapper + 42 callsites + 30 import collapses

- **Composing tasks:** Task 1
- **Checkpoint command:**
  ```
  rg "ScopedBoxFuture\|scoped_futures\|scope_boxed" crates/ tests/ > /tmp/story1-check.txt
  wc -l /tmp/story1-check.txt
  ```
- **Expected output:** `0` (zero remaining references; all 42 callsites use `async |conn| { ... }` shape; all 30 imports collapsed; wrapper trait bound uses `AsyncFnOnce + AsyncFunc`)
- **Brief-Scope outputs to verify** (used by `/brehon-verify`):
  - `Cargo.toml` contains `diesel-async = "0.9.x"`
  - `crates/diesel_utils/src/connection.rs` contains `AsyncFnOnce` in the `run_transaction` trait bound
  - `crates/diesel_utils/src/connection.rs` does NOT contain `ScopedBoxFuture`
  - `rg "\.run_transaction" crates/ tests/` returns 42 lines
  - `cargo check --workspace --features full` exits 0

### Story 2: `sha2 0.11` migration - bump applied; SHA-256 chain preserved; narrow surface clean

- **Composing tasks:** Task 2 (depends on Task 1's wrapper; serial)
- **Checkpoint command:**
  ```
  cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_api --features full --lib"
  ```
- **Expected output:** all lib tests pass (count depends on current `lemmy_api` test suite; expect >=30 lib tests including the rule-set sha2 callers)
- **Brief-Scope outputs to verify:**
  - `Cargo.toml` contains `sha2 = "0.11"`
  - `crates/api/api/src/governance/admin_rule_sets.rs:46` still contains `use sha2::{Digest, Sha256};`
  - `crates/api/api/src/governance/admin_rule_sets.rs:120` still contains `Sha256::digest(...)` (byte-identical or minimally-adjusted)
  - `rg "impl.*Digest" crates/ tests/` returns 0 lines (no explicit Digest impls)

### Story 3: SemVer-compat bundle - 10 bumps applied; workspace + clippy clean

- **Composing tasks:** Task 3 (depends on Task 2; serial)
- **Checkpoint command:**
  ```
  cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"
  ```
- **Expected output:** exit 0 with no warnings under `-D warnings`
- **Brief-Scope outputs to verify:**
  - `Cargo.toml` contains `diesel = { version = "=2.3.9", ...}`, `tokio = "1.52.0"`, `rustls = "0.23.40"`, `bcrypt = "0.19.1"`, `serde_with = "3.20.0"`, `html2text = "0.17.1"` (or later patches per WP-6)
  - `crates/api/api_utils/Cargo.toml` contains `jsonwebtoken = { version = "10.4.0", ... }`
  - `crates/email/Cargo.toml` contains `lettre = { version = "0.11.22", ..., features = [...] }`
  - `crates/routes/Cargo.toml` contains `rss = "2.0.13"`
  - `crates/utils/Cargo.toml` contains `dashmap = { version = "6.2.x", optional = true }`

### Story 4: v1-deps-r1 cohort composite - all 12 bumps ship together; full e2e clean

- **Composing tasks:** Tasks 1 + 2 + 3 (cumulative)
- **Checkpoint command:**
  ```
  cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"
  ```
- **Expected output:** `N passed; 0 failed` (N = e2e test count carried forward from `governance-v0` at lane-cut time; no regression)
- **Brief-Scope outputs to verify:**
  - All Story 1-3 outputs are present
  - Full workspace e2e clean (no skips, no flakes)
  - `Cargo.lock` delta is bounded (~70/-95 lines per brief estimate; verify within +/-200 line tolerance)

---

## 17. Completion checklist

- [ ] Task 0 audit complete (all 12 probes confirmed - Probes 0-11; counts 42 / 30 / 38 / 0 / 0 confirmed at pre-T1 state)
- [ ] Task 1 committed (`feat(deps): migrate to diesel-async 0.9 (task 1)`)
- [ ] Task 2 committed (`feat(deps): bump sha2 0.10 -> 0.11 (task 2)`)
- [ ] Task 3 committed (`feat(deps): bump 10 SemVer-compatible deps (task 3)`)
- [ ] Task 4 committed (`docs(retro): v1-deps-r1 (task 4)`)
- [ ] Section 15.1 validation green at every gate (T1, T2, T3 `cargo check --workspace --features full` exit 0)
- [ ] Section 15.2 validation green at every gate (T1, T2, T3 `cargo clippy --no-deps -- -D warnings` exit 0)
- [ ] Section 15.3 validation green for T1 + T3 (`cargo test --no-run` exit 0)
- [ ] Section 15.4 validation green for T1 (3 lib-tests) + T2 (1 lib-test)
- [ ] Section 15.5 validation green for T1 per-task e2e + phase-tip e2e (both pass; no flakes)
- [ ] Section 15.6 cross-cutting verification green (all 10 boxes ticked)
- [ ] Section 16a stories all `[done]` (Stories 1-4)
- [ ] Retro committed
- [ ] PR opened by BM session against `governance-v0` with `--repo barrie-cork/lemmy`
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-deps-r1-verify.md` shows all stories OK
- [ ] Post-merge phase branch `phase-v1-deps-r1` retained for retro reads (delete only after retro sign-off)
- [ ] Optionally: close Dependabot PR #136 with a comment naming v1-deps-r1 as the deferred resolution (per brief section 10 - closing is OPTIONAL; Dependabot will re-open at next cycle either way)

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `diesel_async::AsyncFunc` re-export path differs from section 10.1 assumption | MED | LOW | impl-task verifies at task-time per section 10.1 "Verify at task-time" note; if path differs, adjust import; not a blocker. |
| Per-callsite Shape A/B/C misclassification (e.g. treating Shape C as Shape A) | MED | MED | Section 10.2 NOTE enumerates the three shapes with byte-precise BEFORE/AFTER; impl-task `rg`-classifies before each per-file Edit; mid-sweep `cargo check` catches mis-applications immediately. |
| Trunk drift between plan-author time and lane-cut time adds new `.run_transaction` callsites (e.g. RT-r3 lands first) | LOW | MED | R11 pre-flight in Task 0 re-counts; drift triggers `kind: "blocker"` DQ. Hard precondition (brief section 0 + section 9) blocks lane-cut until RT-r3 quiesces. |
| sha2 0.11 surfaces a non-zero-diff regression (newtype change DOES bite) | LOW | LOW | Section 10.4 NOTE: minimal canonical fix per `feedback_clippy_test_style.md`; if 3+ fix-cycles, file `kind: "blocker"` DQ. |
| `Cargo.lock` regeneration introduces unexpected transitive churn (>+/-200 lines) | LOW | LOW | Section 15.6 audit catches; surface as `kind: "log"` DQ; do not block phase. |
| One of the 10 SemVer-compat bumps surfaces an API regression on a consuming crate | LOW | MED | G4 classifier fires if `cargo check` fails post-T3; non-allowlist regression -> catch-fire to user; allowlist regression -> narrow fix-impl task. |
| Junior worker mis-applies the closure rewrite as a semantic change (R12 violation) | LOW | HIGH | Section 13 Task 1 brief explicitly cites R12; mid-sweep manual spot-check of 3 random callsites (section 15.6) catches; if missed at impl-task time, CR review at PR-open catches and triages per `feedback_pr_review_triage_pattern.md`. |
| Edit-hang risk on the 32-file mechanical sweep | LOW | LOW | Non-applicable (files are not e2e.rs and are far smaller); the per-file Edit `old_string` targets a 3-5 line window. |
| 2026-06-01 Shape G re-enable boundary passes mid-lane | LOW | LOW | DQ `a3d0e9941441-015` answer (a) protocol: advisor files `kind: "log"` DQ at boundary; subsequent impl-tasks raise `kind: "validate-pending"`; plan section 15 doesn't need re-authoring. |
| Concurrent advisor session on RT-r3 lane writes to `.claude/decision-queue.json` | LOW | LOW | Per `.claude/rules/multi-lane-worktree.md`: each lane has its own worktree -> per-worktree DQ isolation. Cross-lane DQ id collisions structurally impossible under schema-v3. |
| `dashmap 6.1 -> 6.2` introduces hidden API change | LOW | LOW | Section 15.1 + section 15.2 catch at T3 commit time; if regression, file `kind: "blocker"` DQ and pin dashmap to 6.1 in this phase (defer to follow-up). |
| `html5ever 0.39` transitive regression on `html2text` callers | LOW | LOW | Section 2 cite confirms no user-visible API change in html2text 0.17.1; section 15.1 at T3 catches if false. |
| The wrapper change accidentally drops `Send + 'a` bound on the closure return | LOW | HIGH | Section 10.1 verbatim text preserves `R: Send + 'a`; impl-task copies verbatim; CR review catches any drop. |

---

## 19. Notes

**Planner-side dry-run results (per section 15 discipline).** All section 15 commands were dry-run by the advisor at plan-author time (2026-05-24, against `governance-v0 @ 2eb749e1c` on the lane worktree). Results:

- The wrapper commands in section 15 are NOT dry-run on this lane worktree because the planning Junior task runs on a per-task worktree without cargo invocations (planner is read-only on the codebase). The wrapper commands are inherited verbatim from v1-ship-3's section 15 which was dry-run by its planner. Per `feedback_pre_phase_dod_smoke_test.md`: the advisor MUST dry-run all section 15 commands BEFORE plan-approval gate (gate 1 in advisor-orchestrator.md section 3.2) - i.e. on the advisor's laptop session at gate 1, not in this planning Junior task. **Planner's section 15 author-time assertion: every command is identical-shape to v1-ship-3's section 15 which is known-executable.**
- Brief section 4 raw-cargo lines were "advisory-shape only" per DQ `a3d0e9941441-011`; this plan's section 15 converts every command to the wrapper-prefixed shape.
- DQ `a3d0e9941441-016` answer (a) class-targeted `commands[]` arrays are applied throughout section 13 (T1 = check + clippy + 3 lib-tests + e2e; T2 = check + clippy + 1 lib-test; T3 = check + clippy only; phase-tip = full workspace e2e).

**Discrepancy with brief section 1's `with_isolation_level` reference (WP-1):** brief section 2 mentions "line 220: `fut.boxed()` (inside `with_isolation_level` helper)". Plan-author-time verification: there is NO `with_isolation_level` helper in `crates/diesel_utils/src/connection.rs`. Line 220 is inside `establish_connection`, where `fut.boxed()` returns a `BoxFuture` (from `futures_util::FutureExt`, not `scoped_futures`). **This line is NOT touched by T1.** The brief's WP-1 paragraph remains valid in its overall guidance (option a = update wrapper + rewrite callsites; option b = bridge); only the line-pointer is incorrect.

**Discrepancy with brief's per-file callsite counts (WP-5 + R11 evidence):** brief section 2 enumerated ~16 specific files with per-file counts. Plan-author-time `rg "\.run_transaction" crates/ -c` returns slightly different per-file counts (e.g. `inbox.rs = 4` vs brief's 3; `federation_outbox.rs = 3`; `publish_sanction_notice.rs = 1`; `outbox.rs = 1` - these were NOT in the brief's enumeration but exist on plan-author-time trunk). **The grand total of 42 callsites holds.** R11 in section 7 makes impl-task re-enumerate at task-time; per-file drift is expected and accepted (brief was authored against an earlier trunk SHA). The section 11 table is plan-author-time accurate; impl-task `rg`-verifies before each file's Edit.

**On non-existent referenced lessons:** the brief cites `pattern_cargo_feature_flag_propagation.md`, `feedback_junior_worker_e2e_edit_hang.md`, and `feedback_advisor_watchpoint_specificity.md` - none of these `.md` files exist in `.claude/lessons/` at plan-author time (verified via `ls .claude/lessons/`). The disciplines those names invoke are real (mentioned in advisor-orchestrator.md and elsewhere); this plan applies them inline rather than citing missing files. If a retro author wants to make them concrete, that's a follow-up `kind: "log"` DQ candidate.

**On bundled-bump risk vs split-phase risk:** the brief opted to bundle (1) diesel-async breaking + (2) sha2 breaking + (3) 10 SemVer-compat into one phase. Tradeoff:
- **Bundled (this plan):** one bm-cut + one CR review + one bm-merge; single phase-tip e2e gate (~26 min); 3 commits visible in `git bisect`. Risk: a single failure in any of the 3 commits blocks all 12 bumps from merging.
- **Split (rejected per brief):** 3x bm-cut + 3x CR review + 3x bm-merge + 3x phase-tip e2e gate (~78 min total). Risk: each phase's breaking-change validation is independent, but the lane-time tripled for no reduction in code-quality risk.

**Brief acknowledged the bundled approach explicitly** ("Splitting (1) and (2) into separate phases doubles the cargo+e2e validation cost without reducing risk"); this plan honours that decision. The mitigation for "single failure blocks all" is the per-commit `git revert` escape hatch (each task is a separate commit; failure in T3 can be reverted without touching T1 or T2).

**On Junior-side `Cargo.lock` regeneration semantics:** under `validate-pending-laptop` mode, Junior pushes a worker branch with `Cargo.lock` regenerated locally (Junior runs `cargo check` once before pushing per `feedback_fix_impl_pre_push_cargo_check.md`). The advisor laptop re-runs the full DoD when handling the `validate-pending-laptop` DQ entry, which regenerates `Cargo.lock` again on the laptop (potentially with different transitive resolution if the laptop's `~/.cargo/registry/cache/` differs from Junior's). If `Cargo.lock` differs between Junior push and advisor laptop verification, the advisor reconciles in a `chore(lockfile):` commit on the phase branch (not the worker branch). Brief section 2 estimates the delta; if observed delta diverges, surface as a `kind: "log"` DQ.

**On RT-r3 cross-lane invariant (DQ `a3d0e9941441-017`):** answer (b) proceed; RT-r3 worktree has zero commits ahead of trunk at session-start. If RT-r3 starts mid-lane and adds `.run_transaction` callsites BEFORE v1-deps-r1's T1 lands, R11 in Task 0 catches the drift. The planner did NOT pre-emptively coordinate with RT-r3 because the lane is dormant.

---

## 20. Confidence score

- **Plan correctness:** 8/10 - the brief is detailed and the codebase verification (`rg`-counts at plan-author time) confirms the 42-callsite + 30-import scope. The `AsyncFunc` re-export path is the one task-time unknown.
- **Cargo budget:** 9/10 - `validate-pending-laptop` mode makes budget non-binding for impl-task dispatch; the only budget concern is the laptop's RAM during the post-T1 full re-compile (~6-8 GB warm expected; well within laptop cap).
- **Test coverage:** 9/10 - the full workspace e2e at phase-tip is the canonical gate; per-crate lib-tests for T1 + T2 catch narrow regressions; clippy `-D warnings` catches lint debt. Coverage is high because the migration is mechanical - the question "did the rewrite preserve semantics" is verified end-to-end by the existing test suite (which is the right gate for a mechanical sweep).
