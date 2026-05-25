# Planning Brief — v1-deps-r1: Cargo Dependency Bump Migration

**Phase:** v1-deps-r1
**Branch:** phase-v1-deps-r1 (cut from governance-v0 — exact SHA at lane-cut time)
**Authored:** 2026-05-23
**Authored by:** advisor (canonical brehon-fork session)
**PRD:** (none — pure dependency bump phase; no PRD needed)
**Plan target:** `.claude/PRPs/plans/v1-deps-r1.plan.md`

**Hard precondition:** Do NOT cut this phase until BOTH `v1-ship-3` AND `v1-RT-r2` have merged to `governance-v0` and their retros are signed off. The cargo workspace surface this phase rewrites overlaps with both active lanes' write paths. Concurrent cuts will produce merge conflicts on every `*.run_transaction(...)` callsite (42 known files).

---

## 1. What this phase delivers

A clean migration of 12 cargo dependency bumps surfaced by Dependabot PR #136 (deferred 2026-05-23 because two of them are breaking and one of them rewrites the load-bearing governance write path). The phase consolidates three change classes into a single bundled phase to amortize cargo + e2e validation cost:

1. **`diesel-async` 0.8.0 → 0.9.0 — BREAKING.** Upstream changelog: "Change all transaction related functions to accept a real async closure instead of the scoped boxed variant. This change requires adjusting all call sides of transaction based functions from `conn.transaction(|conn| async move {/* code */}.scoped_boxed())` to `conn.transaction(async |conn| /* code */)`." Affects: the wrapper at `crates/diesel_utils/src/connection.rs` (imports `scoped_futures::ScopedBoxFuture` which is removed in 0.9; the `run_transaction` method signature uses `ScopedBoxFuture`) AND every callsite of `.run_transaction(|conn| { async move { ... }.boxed() })` across governance handlers. **42 callsites confirmed** as of trunk SHA `28f0dc0ff`.

2. **`sha2` 0.10 → 0.11 — BREAKING.** Upstream changelog: "Update to digest v0.11. Replace type aliases with newtypes (#678). Removed std feature." Affects: `crates/api/api/src/governance/admin_rule_sets.rs:46,120` (Sha256::digest for rule_text computation, ADR-012 hash-chain consumer) and 3 e2e test modules in `crates/server/tests/e2e.rs` using `Sha256::new() / update() / finalize()`. The `Digest` trait surface (`new`, `update`, `finalize`, `digest`) is preserved; the newtype change MAY surface as a trait-resolution error if any code stores `Sha256` in a struct with a `Digest`-bounded type param. No such pattern observed in the fork at brief-author time, but a planner pre-flight grep is mandatory (see WP-2).

3. **Ten SemVer-compatible bumps (low risk).** Bundled with the breaking pair to amortize the e2e gate: `diesel 2.3.7 → 2.3.9` (includes `#[derive(AsChangeset)]` regression fix), `tokio 1.50 → 1.52`, `rustls 0.23.39 → 0.23.40` (security), `bcrypt 0.19.0 → 0.19.1`, `serde_with 3.18 → 3.20`, `jsonwebtoken 10.3 → 10.4`, `lettre 0.11.21 → 0.11.22`, `rss 2.0.12 → 2.0.13`, `dashmap 6.1 → 6.2`, `html2text 0.16.7 → 0.17.1` (pulls `html5ever 0.38 → 0.39` + `markup5ever 0.38 → 0.39` transitively).

All three classes ship in one PR. Splitting (1) and (2) into separate phases doubles the cargo+e2e validation cost without reducing risk — the cohort §13 plan can sequence them as Task 1 (diesel-async), Task 2 (sha2), Task 3 (SemVer-compat bundle), each with its own commit so any individual bump can be reverted via `git revert` if a regression surfaces post-merge.

---

## 2. Key file anchors (verify these before authoring the plan)

| File | Anchor | Purpose |
|---|---|---|
| `Cargo.toml` (workspace root) | `diesel = { version = "=2.3.7", ... }` line | Task 1+3: workspace-level version pins |
| `Cargo.toml` (workspace root) | `diesel-async = "0.8.0"` | Task 1: bump to 0.9.0 |
| `Cargo.toml` (workspace root) | `sha2 = "0.10"` | Task 2: bump to 0.11 |
| `Cargo.lock` | (auto-updated; expect ~70/-95 line delta per Dependabot PR #136) | Lockfile reflects all bumps |
| `crates/diesel_utils/src/connection.rs:11` | `use diesel_async::{ ... scoped_futures::ScopedBoxFuture, ... }` | Task 1: import must be removed (path no longer exists in 0.9) |
| `crates/diesel_utils/src/connection.rs:61-72` | `run_transaction` wrapper method | Task 1: signature must change to accept an async closure; internal `.transaction::<_, LemmyError, _>(callback)` call must use the 0.9 closure-style API |
| `crates/diesel_utils/src/connection.rs:220` | `fut.boxed()` (inside `with_isolation_level` helper) | Task 1: may need rewrite depending on how the new transaction API handles isolation-level wrappers |
| `crates/api/api/src/governance/admin_rule_sets.rs:46,120` | `use sha2::{Digest, Sha256}` + `Sha256::digest(...)` | Task 2: verify newtype change does not break this usage |
| `crates/api/api_utils/Cargo.toml` | `jsonwebtoken = { version = "10.3.0", features = ["rust_crypto"] }` | Task 3: bump to 10.4.0 |
| `crates/email/Cargo.toml` | `lettre = { version = "0.11.19", ... }` | Task 3: bump to 0.11.22 |
| `crates/routes/Cargo.toml` | `rss = "2.0.12"` | Task 3: bump to 2.0.13 |
| `crates/utils/Cargo.toml` | `dashmap = { version = "6.1.0", optional = true }` | Task 3: bump to 6.2.1 |
| `crates/server/tests/e2e.rs:938,2526,7297` | `use sha2::{Digest, Sha256};` (3 distinct test modules) | Task 2: verify api surface usage still compiles |

42 callsite enumeration for Task 1 (from `grep -rn "\.run_transaction" crates/`):
- `crates/api/api/src/community/{add_mod,ban,block,transfer}.rs` (4 sites)
- `crates/api/api/src/governance/{accept_jury_assignment, admin_assign_jury, admin_close_case, admin_config, admin_emergency_remove, admin_rule_sets (×2), admin_trigger_appeal_rejury, decline_jury_assignment, sponsor_liability_grace (×2), submit_jury_vote}.rs` (12 sites)
- `crates/api/api/src/site/registration_applications/approve.rs` (1 site)
- `crates/api/api_crud/src/governance/{create_endorsement, create_report, request_appeal, revoke_endorsement}.rs` (4 sites)
- `crates/api/api_crud/src/user/create.rs` (2 sites at line 155 + 407)
- `crates/apub/activities/src/governance/inbox.rs` (3 sites at 197, 318, 638)
- Plus ~16 more across `crates/db_schema`, `crates/routes`, e2e tests — re-enumerate via `rg "\.run_transaction" crates/` at plan-author time (the count above is from 2026-05-23 trunk @ `28f0dc0ff`; v1-ship-3 + v1-RT-r2 may add callsites).

**Planner MUST re-enumerate at plan-author time** — per `feedback_fix_impl_enumerate_all_callsites.md`. Treat any drift from 42 as a signal to recount, not to trust the brief.

---

## 3. Watchpoints for the planner

**WP-1 (Task 1 — diesel-async closure migration is mechanical but pervasive):** The migration is a literal find-and-replace pattern:
- Before: `.run_transaction(|conn| { async move { /* body */ }.boxed() }).await`
- After: `.run_transaction(async |conn| { /* body */ }).await` (or equivalent — verify against `diesel-async 0.9` docs which signature shape the wrapper should accept)

Two ways to absorb the change:
- **(a) Update the wrapper signature** at `connection.rs:61` to accept an async closure, then mechanically rewrite all 42 callsites. Largest diff.
- **(b) Keep the wrapper signature, build an internal bridge** that converts our existing `FnOnce -> ScopedBoxFuture` shape into an async closure for the 0.9 `transaction` method. Smaller diff at callsites but more compiler-magic in the wrapper.

Planner picks the path. Default: **option (a)** — explicit closure rewrite is what every callsite reader will see, and the `scoped_futures` crate is going away upstream; bridging keeps a dead pattern alive. Trade-off: bigger diff. Validate via `cargo check --workspace --features full` after EACH commit in Task 1 (split into wrapper-first commit + callsites-second commit if practical).

**WP-2 (Task 2 — sha2 newtype impact on trait bounds):** Before touching `Cargo.toml`, run `rg -E "Sha256\b" crates/ tests/` and `rg "impl.*Digest" crates/ tests/`. If any struct stores `Sha256` in a field with a `Digest`-bounded generic parameter, the newtype change may surface as `error[E0277]: trait bound not satisfied`. Expected zero hits in the fork at brief-author time, but **the planner pre-flight grep is mandatory** — drift is the failure mode. If hits surface, file a `kind: "blocker"` DQ before authoring Task 2.

**WP-3 (Task 2 — `std` feature removal):** `sha2 0.11` removes the `std` crate feature in favor of `alloc`. If our `Cargo.toml` enables `sha2 = { version = "0.11", features = ["std"] }` or anything similar, this fails at resolve time. Check the workspace root + every consuming crate's `Cargo.toml` (`crates/server/Cargo.toml` has `sha2 = { workspace = true }` — workspace-level features apply). Plan default: do not enable `std`; rely on default features (the `Digest` trait surface we use does not require `std`).

**WP-4 (Task 3 — html2text + html5ever transitively):** `html2text 0.17` pulls `html5ever 0.39` + `markup5ever 0.39`. Both are major-version bumps on transitive crates. Lemmy's RSS/markdown rendering uses `html2text` via a small surface — `rg -l "html2text" crates/` — verify and check release notes for that crate's API stability across 0.16 → 0.17 before merging Task 3.

**WP-5 (post-lane drift):** This brief references trunk SHA `28f0dc0ff` (2026-05-23). When this lane is cut, trunk will be ahead by at least: ship-3 merge SHA + ship-3 retro SHA + RT-r2 merge SHA + RT-r2 retro SHA + any RT-r3 work that may have started in parallel. Planner MUST re-run the `rg "\.run_transaction" crates/` enumeration against trunk-at-cut-time and update Task 1's callsite count in the plan §13 task table.

**WP-6 (Dependabot drift — Dependabot will likely re-open PR #136 with a different head SHA):** If Dependabot has bumped any of the 12 packages further between 2026-05-23 and lane-cut time (e.g. `diesel 2.3.9 → 2.3.10`, `tokio 1.52.3 → 1.53.x`), bring the bumps to the latest available versions at lane-cut time, not the versions in this brief. The breaking pair (diesel-async 0.9, sha2 0.11) are the floor; later patch versions are preferred.

---

## 4. DoD gates (cargo + e2e)

Per `feedback_plan_dod_dry_run_at_write.md`. **All DoD commands MUST be wrapper-prefixed (`cmd //c "scripts\brehon\cargo-<verb>.bat ..."`) per DQ `a3d0e9941441-011` clarify (Windows + libpq.dll + vcvars discipline).** The raw-cargo lines below are advisory-shape only; the planner converts every command to wrapper form when authoring plan §15 + per-task §13 DoDs. Per-task `commands[]` arrays for `validate-pending-laptop` DQ entries are class-targeted per DQ `a3d0e9941441-016` (see §4a below).

**Task 1 (diesel-async 0.9 migration):**
```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"          # must exit 0
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"    # must exit 0
```
**Task 1 e2e gate** (validate-pending-laptop-e2e, per `feedback_laptop_default_for_validate_pending.md` + `feedback_validate_pending_laptop_must_use_wrapper.md` + `feedback_windows_e2e_requires_bat_wrapper.md`):
```
cmd //c "scripts\brehon\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-deps-r1-task1-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-deps-r1-task1-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-deps-r1-task1-e2e.log"
```
(run_in_background: true; ~26 min on the laptop)

**Task 2 (sha2 0.11 migration):**
```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"
cmd //c "scripts\brehon\cargo-test.bat -p lemmy_api_common --features full --lib"  # targeted lib-test (sha2 surface is narrow)
```
Full e2e gate runs only after Task 3 lands (single phase-tip gate at PR-open per DQ `a3d0e9941441-016`).

**Task 3 (SemVer-compatible bundle):**
```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"
```

**Phase-tip e2e gate (post-Task 3, pre-PR):** full workspace e2e per the validate-pending-laptop pattern. Per DQ `a3d0e9941441-015` clarify, Shape G stays SUSPENDED through 2026-06-01 — if the lane runs past June 1 (DQ #229 re-enable), advisor files a `kind: "log"` DQ at the boundary and subsequent impl-tasks raise `kind: "validate-pending"` (Shape G) instead of `validate-pending-laptop`; plan §15 does NOT need re-authoring (polling loop handles both kinds transparently). Result MUST be `pass` for every test (no skips, no flakes) before opening the PR.

### 4a. Per-task class-targeted `commands[]` (per DQ `a3d0e9941441-016`)

| Task | Class | `commands[]` shape (each wrapper-prefixed per DQ 011) |
|---|---|---|
| T1 | Production code + tests (42-callsite mechanical rewrite — full coverage) | check + clippy + lib-test for touched governance crates + e2e (raise as `validate-pending-laptop-e2e` kind) |
| T2 | Production code narrow + 3 test modules (sha2 surface narrow) | check + clippy + targeted lib-test `-p lemmy_api_common --features full --lib` |
| T3 | Cargo.toml + Cargo.lock edits only (no behavioral change) | check + clippy |
| Phase-tip (post-T3, pre-PR) | Full workspace e2e single gate | `cmd //c "scripts\brehon\cargo-test.bat --workspace --test e2e --features full"` |

This replaces three per-task e2e runs with one phase-tip e2e gate (saves ~50 min lane-time).

---

## 5. Lesson injections (mandatory, per advisor-orchestrator §2.4)

**Inject for Task 1 (diesel-async 0.9 migration touching 42 files):**
- `feedback_fix_impl_enumerate_all_callsites.md` — `rg "\.run_transaction" crates/` before authoring; planner MUST cite the actual count vs the brief's 42-callsite count. Drift is the failure mode (per WP-5).
- `feedback_multi_write_handlers_need_transactions.md` — confirms the load-bearing nature of the `.run_transaction` pattern; do not silently widen or narrow transaction scopes during the mechanical rewrite.
- `feedback_async_pool_test_pattern.md` — for the e2e test file `use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl}` imports; the 0.9 API surface for `AsyncConnection::establish` may differ — verify and update imports.
- `feedback_lemmy_error_no_std_error.md` — Case A discipline for any e2e test fn touched by the migration.

**Inject for Task 2 (sha2 0.11 migration):**
- `feedback_fix_impl_enumerate_all_callsites.md` — `rg "Sha256\b" crates/ tests/` enumeration.

**Inject for all tasks:**
- `feedback_clippy_test_style.md` — `#![deny(unwrap, expect)]` applies; use `?` throughout.
- `feedback_features_full_workspace_only.md` — every cargo gate above uses `--workspace --features full`; never `-p lemmy_server --features full` (the recurring footgun).
- `feedback_features_full_p_crate_incompatible.md` — companion to the workspace-only rule; `-p <crate> --features full` is structurally broken in this workspace.
- `pattern_cargo_feature_flag_propagation.md` — covers feature unification across the 6 sub-crate Cargo.toml edits (api_utils, email, routes, utils, etc).
- `feedback_validate_pending_laptop_must_use_wrapper.md` — Windows wrapper discipline for every cargo invocation in `commands[]` arrays (per DQ `a3d0e9941441-011` clarify).
- `feedback_targeted_validate_pending_laptop_commands.md` — per-task class-targeted command selection (per DQ `a3d0e9941441-016` clarify; see §4a).
- `feedback_windows_e2e_requires_bat_wrapper.md` — e2e invocation on Windows requires `cmd //c "scripts\brehon\cargo-test.bat ..."` for libpq.dll discovery.
- `feedback_clippy_per_module_deny_requires_workspace_allow.md` — clippy baseline trap: workspace-level group-deny silently breaks per-module `#![deny]` (per DQ `a3d0e9941441-014` clarify; Task 0 captures baseline before Task 1).

**Do NOT inject** `feedback_junior_worker_e2e_edit_hang.md` (only relevant for large mid-file edits; this phase's e2e file edits are at import statements, not in test bodies).

---

## 6. Scope boundaries (stop-and-ask tripwires)

- **Stop if** any `diesel-async 0.9` API surface other than `transaction` has changed (check `AsyncConnection::establish`, `AsyncPgConnection` field names, `pooled_connection` exports) — file `kind: "blocker"` DQ before authoring any Task 1 callsite rewrites.
- **Stop if** the `sha2 0.11` newtype change surfaces in `rg "impl.*Digest" crates/ tests/` (pre-flight check) — re-scope Task 2 before authoring.
- **Stop if** `html5ever 0.39` introduces a breaking API change to `html2text` callers (`rg "html2text" crates/`) — Task 3 either downgrades `html2text` to the latest 0.16.x or defers.
- **Stop if** trunk-at-cut-time has acquired a new dependency that conflicts with any bump (e.g. a new crate pinned to `diesel-async 0.8.x`) — DQ + ask user.
- **Stop if** the cargo gate runs locally on Windows fail for reasons other than the bump itself (libpq env, vcvars, etc) — fix the wrapper, not the dependency.

---

## 7. Not in scope for v1-deps-r1

- npm/pnpm dependency bumps in `api_tests/` — those land via Dependabot PRs directly (PR #135 pattern, already merged 2026-05-23).
- Major-version bumps that Dependabot has not surfaced yet (e.g. `rustls 0.24.x`, `tokio 1.53.x` if pre-released).
- Any change to the `governance_log` hash-chain protocol — sha2 0.11 is an in-place API surface bump; the hash output is byte-identical SHA-256, the chain remains valid.
- Any refactor of the `run_transaction` wrapper beyond what diesel-async 0.9 requires — keep the same public signature shape for callers if possible (per WP-1 option a/b decision).
- Workspace-wide rustc edition migration — separate concern.
- Removal of the `scripts/brehon/cargo-*` wrappers — wrappers stay; their behavior is verified by Probes 1-4 in `.claude/rules/pre-phase-harness-audit.md`.

---

## 8. Plan structure guidance

The plan should have 3 §13 tasks (or 4 if WP-1 splits Task 1 into wrapper + callsites). **Task 0 MUST include both the four wrapper-probes from `pre-phase-harness-audit.md` §1 AND the clippy baseline capture from §3** (per DQ `a3d0e9941441-014` clarify) — non-zero baseline → planner adds a `chore(lint):` pre-task before Task 1 to clear pre-existing debt:

| Task # | Deliverable | Files | `[P]`? |
|---|---|---|---|
| T0 | Pre-phase harness audit + clippy baseline capture + re-enumerate callsites | `.claude/PRPs/debug/v1-deps-r1-task0-*.log` (diagnostic) | No |
| T1 | diesel-async 0.9 migration: wrapper + 42 callsites (planner-choice WP-1 a/b per DQ `a3d0e9941441-013`; default option a) | `crates/diesel_utils/src/connection.rs`, ~42 governance handler files, e2e.rs imports | No — single-file invariants violated by parallel dispatch |
| T2 | sha2 0.11 migration | `Cargo.toml`, `admin_rule_sets.rs`, e2e.rs (3 test modules) | No — must follow T1 (`cargo check` validates T1 first) |
| T3 | SemVer-compatible bundle (10 bumps) | `Cargo.toml`, `Cargo.lock`, 4 sub-crate Cargo.toml files | No — must follow T2 (`Cargo.lock` ordering) |

`[P]` is **NOT applicable** to any of T1/T2/T3:
- T1 alone touches 42 files in a single semantic change; splitting risks partial-state compile errors mid-cohort.
- T2 depends on T1's wrapper landing cleanly (the workspace must compile before sha2's underlying `digest 0.11` resolution can be validated).
- T3 depends on T2's resolution (Cargo.lock is monotonic; later commits must rebase onto earlier ones).

Plan complexity: **6/10**. Estimated wall-clock: 2-4 days (T1 is the bulk; T2 + T3 are <1 day combined). One ci-watcher cycle per task at minimum; T1 may need fix-impl recovery if any callsite was missed in the enumeration.

---

## 9. Pre-queue checklist (advisor to run before dispatching planning Junior)

- [ ] **Hard precondition check:** `gh pr view <ship-3-pr> --json mergedAt --jq .mergedAt` and same for RT-r2 — both MUST return non-null. If either is null, do NOT cut this lane.
- [ ] **Retro sign-off check:** `ls .claude/PRPs/reports/v1-ship-3-retro.md v1-RT-r2-retro.md` (or equivalent canonical retro names) — both must exist on trunk.
- [ ] `/brehon-clarify` run on this brief — WP-1 (option a vs b for the wrapper migration path) is the candidate clarify-DQ; WP-2 (sha2 newtype pre-flight grep) is verified by the planner's pre-author check, not a clarify question.
- [ ] `git -C C:/Users/barri/Developer/brehon-fork show governance-v0:.claude/PRPs/briefs/v1-deps-r1-planning-1.md` — must succeed (brief committed before dispatch).
- [ ] `rg "\.run_transaction" crates/ | wc -l` — re-count callsites; if drift from 42 is >5, surface to user before plan dispatch.
- [ ] `memory_search_hybrid("diesel-async transaction closure migration", limit: 5)` — check for any lessons authored between 2026-05-23 and lane-cut time on the same topic.

---

## 10. Background: why this is deferred

PR #136 (Dependabot, opened 2026-05-18) bundled all 12 bumps into a single mergeable PR. Investigation on 2026-05-23 (per session retro `session-retro-2026-05-23-tidy-up-and-dependabot-investigation.md` if authored) revealed:

- diesel-async 0.9 is a breaking change requiring 42-file mechanical rewrite of the governance write path.
- sha2 0.11 is a breaking change requiring trait-resolution pre-flight verification.
- Merging PR #136 as-is would not compile.

User decision (2026-05-23): defer the migration as a dedicated sub-phase rather than fix-in-PR'ing #136 (which would force the migration to happen in a context where the Dependabot branch lifecycle competes with active lanes). Closing PR #136 with a comment naming the deferral is OPTIONAL — Dependabot will re-open with a fresh head SHA at the next opening cycle, and this brief is the durable record of the migration plan.

Roadmap entry: see `.claude/PRPs/v1-roadmap.json` `deps` lane → `v1-deps-r1` sub-phase (status=unstarted, with `depends_on: ["v1-ship-3", "v1-RT-r2"]` as the precondition gate).
