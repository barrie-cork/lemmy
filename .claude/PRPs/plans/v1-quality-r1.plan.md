# Codebase quality plan — three defects + three latent-defect audits

## Resume context (2026-05-22)

**Branch:** `phase-v1-quality-r1` (worktree: `C:/Users/barri/Developer/brehon-fork-quality-r1`)
- Cut at `77aa29aea` (= `chore(bm): bm-cut phase-v1-federation-inbound-d @ 26c6badf2` on `governance-v0` ancestor)
- ~3 commits behind current `origin/governance-v0` (`e9caf8933`); intervening commits are skill/retro/runlog edits that do NOT touch Phase 1/2/3 files. Optional pre-PR rebase but not required to resume.
- **NOT pushed to origin yet.** First push happens at end of Phase 1+2 (or all three phases) per discretion.

**Plan-baseline trunk:** `7e6c4202f` (audit ran here; trunk has since advanced to `e9caf8933` with no scope-overlapping changes).

**In-flight uncommitted edits in worktree:**
- `.rustfmt.toml` — Phase 2 nightly-only options removed; `cargo fmt --all` reformat not yet run.
- `crates/api/api/src/governance/case_open_snapshot.rs` — Phase 1 snapshot helper updated to 27 keys with `get_int`/`get_float`/`get_bool`/`get_text` dispatch; `REQUIRES_RE_JURY_KEYS` const updated; parity test not yet run.

**Phase status:**
| Phase | Status |
|---|---|
| 1 — jury-keys snapshot drift | DRAFTED (uncommitted) — needs parity test + workspace cargo-check + commit |
| 2 — .rustfmt.toml stable/nightly | DRAFTED (uncommitted, partial) — needs `cargo fmt --all` reformat + verify + commit |
| 3 — LazyLock workspace-wide (19 tests, 11 files) | NOT STARTED |
| 4 — three latent-defect audits | DONE (reports in audit-2026-05-22 worktree); findings folded into Phase 5 DQ list |
| 5 — DQ entries + PR | NOT STARTED |

**Plan deviations from baseline (resolved correctly during impl):**
- Count is 27 (not 26 as the plan baseline narrative said). Plan explicitly said "reconcile against `CONFIG_KEY_METADATA` filter at impl time — the count is authoritative there"; impl did that correctly.
- `config::get_float` reader (config.rs:260) is used for the 6 fraction keys. Plan baseline omitted this reader from the dispatch list; impl picked it up from the source-of-truth correctly. ADR-010 compliance intact (Float is the only sensible reader for `quorum_fraction.*` / `threshold_fraction.*`).

## Context

A read-only validation sweep of the Brehon-fork governance-v0 trunk (`7e6c4202f`, audit worktree `C:/Users/barri/Developer/brehon-fork-audit-2026-05-22`) ran fmt-check, clippy, test no-e2e, and test e2e. e2e passed clean (103/0/5). The other three checks surfaced four issues, three of which are real defects on trunk:

| Defect | Severity | Class |
|---|---|---|
| `REQUIRES_RE_JURY_KEYS` parity-pin drift in `case_open_snapshot.rs` | **High** — ADR-010 violation; new cases miss 20 jury-config keys in their pinned snapshot | Brehon governance runtime bug |
| `cargo fmt --check` 1028 diffs across 50+ files | **Medium** — environmental, not real drift; nightly-only rustfmt options silently no-op on stable | Build/dev-loop friction |
| Clippy 8 errors in `admin_config.rs` (3) + `reputation_snapshot.rs` (5) | **Low** — test-style hygiene (`unwrap`/`expect`/`changes[N]`); blocks CI clippy gate | Test-style breach |
| `LazyLock` panic cascade on lib tests (Settings init from wrong CWD) | **Medium** — masks real first-failing test signal; 19 vulnerable tests across 11 files (Audit 4.3) | Test infra |

The plan addresses the **High** and **Medium** runtime/infra defects via three implementation phases, defers Clippy via a DQ entry per user decision, and runs three follow-up audits to catch the same defect classes wherever else they may be hiding.

Per the user decision matrix:
- Fmt → remove nightly-only options from `.rustfmt.toml`, reformat workspace once.
- Jury-keys → fix snapshot helper + test guard only (WRITE-side; v1-AD-c scope). v1-JM-b read-side flip is a separate DQ entry.
- Clippy → DQ entry only; no fix in this plan.
- Broader audit → run all three latent-defect sweeps (CONFIG_KEY_METADATA consumers, parity-pin patterns, Settings::SETTINGS callers).
- LazyLock → workspace-wide scope per user decision 2026-05-22 (was lemmy_api-only in baseline; Audit 4.3 expanded to 19 tests across 11 files).

## Authority + scope

- **Branch:** `phase-v1-quality-r1` off `governance-v0` per `.claude/rules/phase-branch.md`.
- **Owner mode:** advisor session orchestrates; the actual work goes through Junior planning + impl tasks per the four-role model. This plan is the source of truth a planning Junior would consume.
- **Out of scope:** the v1-JM-b jury-reader flip (separate DQ + sub-phase), the clippy fixes (DQ entry only), any v1-federation-inbound-d work currently in flight.

## Phase 1 — fix jury-keys snapshot drift (WRITE-side only)

**Goal:** make `build_applied_config_snapshot` write all 27 `requires_re_jury: true` keys into `moderation_case.applied_config_snapshot`, so new cases pin the full v1-JM-a-era config per ADR-010. Update the test-only drift-guard const to match.

> **Authoritative count is 27** per Audit 4.1 (`CONFIG_KEY_METADATA` filtered for `requires_re_jury: true`). Original baseline used "26"; the discrepancy resolved when impl read the source-of-truth at `crates/api/api/src/governance/config.rs`. The plan-baseline narrative ("19 new keys") is preserved for context but the canonical total is 27 (7 pre-existing + 20 newly-added).

**Files modified (1):**
- `crates/api/api/src/governance/case_open_snapshot.rs`

**Changes:**
1. In `build_applied_config_snapshot`:
   - Add 20 new reader calls (`config::get_int` / `get_float` / `get_bool` / `get_text`) for the 20 missing keys, grouped by `ValueType` in the source.
   - Extend the `json!({...})` block to map each key to its `.await?`-resolved value.
   - **Dispatch on `ValueType`** — readers exist at `crates/api/api/src/governance/config.rs`: `get_int`, `get_float` (line 260), `get_bool`, `get_text`. Per Audit 4.1 + 4.2: `Int` keys → `get_int` → `i64`; `Float` keys (the 6 `quorum_fraction.*` / `threshold_fraction.*`) → `get_float` → `f64`; `Bool` → `get_bool`; `Enum` → `get_text` → raw string. Reuse `crate::governance::config::{CONFIG_KEY_METADATA, get_int, get_float, get_bool, get_text, ConfigCache, Scope}`.
2. In `REQUIRES_RE_JURY_KEYS` const:
   - Replace the 7-entry literal with a list containing all 27 keys (enumerate; const-fn extraction not viable in stable Rust).
3. Re-run the parity test `snapshot_keyset_matches_requires_re_jury_metadata` — must pass.

**Dependency files read-only:**
- `crates/api/api/src/governance/config.rs` — `CONFIG_KEY_METADATA` is the source-of-truth (line ~1677). Readers `get_int/get_float/get_bool/get_text` already exist.
- `crates/api/api_crud/src/governance/create_report.rs` (around line 247) — caller; do NOT modify; verify the scope/pool flow stays compatible.

**Verification:**
- `bash scripts/brehon/cargo-test.bat -p lemmy_api --lib governance::case_open_snapshot::parity` — must show `1 passed`.
- `bash scripts/brehon/cargo-check.sh --workspace --features full` — must exit 0.
- Spot-check the new JSON snapshot shape against the 27 keys (eyeball the `json!` block grouping; impl groups by ValueType for readability).

**Out of scope (DQ entry instead — see Phase 5):** the v1-JM-b read-side flip in `admin_assign_jury.rs:480–550`, `submit_jury_vote.rs`, `sponsor_liability.rs`. Until that ships, in-flight juries on cases opened *before* this fix will continue to read live config for the missing 20 keys (the bug remains for backfill cases). New cases opened *after* this fix pin correctly. The DQ entry tracks the residual ADR-010 gap.

### Phase 1 status — RESUMED 2026-05-22 (in-flight on `phase-v1-quality-r1`)

A pre-pause draft is already present in the worktree (uncommitted):
- `build_applied_config_snapshot` updated to read all 27 keys, grouped by ValueType (Int / Float / Bool / Enum). `get_float` correctly used for the 6 fraction keys.
- Docstring header updated 7 → 27.
- `REQUIRES_RE_JURY_KEYS` const updated to mirror the helper's emitted key set.
- Parity test source unchanged (it filters `CONFIG_KEY_METADATA` dynamically — auto-adapts).

**Remaining Phase 1 steps to resume:**
1. Run `bash scripts/brehon/cargo-test.bat -p lemmy_api --lib governance::case_open_snapshot::parity` → must show `1 passed` (validates const ↔ helper ↔ metadata three-way).
2. Run `bash scripts/brehon/cargo-check.sh --workspace --features full` → must exit 0 (no consumer broke from the wider snapshot).
3. Stage + commit: `feat(case-open-snapshot): pin all 27 requires_re_jury keys (ADR-010 compliance)`.

## Phase 2 — resolve `.rustfmt.toml` stable/nightly divergence

**Goal:** make `cargo fmt --check` pass on stable Rust 1.95 (the pinned toolchain). Per user decision: remove nightly-only options + reformat workspace once.

**Files modified (1 config + ~50 reformatted):**
- `.rustfmt.toml` — remove nightly-only options
- 50+ files across `crates/api/api/`, `crates/api/api_common/`, `crates/db_schema/`, `crates/apub/`, `crates/db_views/`, `crates/server/tests/e2e.rs`, `crates/tools/seed_founders/` — auto-reformatted

**Changes:**
1. `.rustfmt.toml` — remove these 5 nightly-only lines:
   ```toml
   imports_layout = "HorizontalVertical"
   imports_granularity = "Crate"
   group_imports = "One"
   wrap_comments = true
   comment_width = 100
   ```
   Keep: `tab_spaces = 2`, `edition = "2024"`.
2. Run `cargo fmt --all` (stable) once → reformats all 50+ files in one auto-generated commit. Commit subject: `style(fmt): reformat workspace under stable rustfmt (drop nightly-only options)`.
3. Verify `cargo fmt --all -- --check` exits 0.
4. **Optional follow-up (NOT in this plan):** add a `cargo fmt --check` step to `.github/workflows/cargo-validate-workspace.yml` so future fmt drift is caught in CI. Surface this as a DQ entry in Phase 5 if you want it tracked but not bundled with this plan.

**Cost:** ~5 min auto-reformat + ~30 s review of diff sample (every diff will be import-collapse and indent-tweak; no semantics change). The fmt-reformat commit will be large — flag in PR body that it's mechanical.

**Out of scope:** any documentation update on contributor fmt expectations; pre-commit hook addition.

### Phase 2 status — RESUMED 2026-05-22 (in-flight on `phase-v1-quality-r1`)

A pre-pause draft is already present in the worktree (uncommitted):
- `.rustfmt.toml` has exactly the 5 nightly-only lines removed; `tab_spaces = 2` and `edition = "2024"` retained.

**Remaining Phase 2 steps to resume:**
1. Commit the `.rustfmt.toml` change FIRST as its own commit: `chore(fmt): remove nightly-only rustfmt options (stable-compat)`. Keeps the reformat-bulk commit isolated.
2. Run `cargo fmt --all` from the worktree root to reformat the workspace.
3. Verify `cargo fmt --all -- --check` exits 0.
4. Commit the reformatted files: `style(fmt): reformat workspace under stable rustfmt`. Expect ~50 files touched; review a sample diff to confirm import-collapse + indent-tweak only (no semantics change).

## Phase 3 — fix Settings `LazyLock` poisoning across lib tests (workspace-wide)

**Goal:** stop the `Settings::SETTINGS` `LazyLock` from poisoning when ANY lib test is run from a crate-local CWD (where `config/config.hjson` doesn't resolve). Per Audit 4.3 (`.claude/PRPs/reports/audit-settings-lazy-lock-callers-2026-05-22.md`), 19 vulnerable tests across 11 files in `crates/api/api/` (7) and `crates/apub/` (12) hit the same poisoning path — the original audit-log only surfaced the `lemmy_api` cascade because `-p lemmy_api --lib` was the only crate-local lib invocation that ran. Workspace-wide scope per user decision 2026-05-22.

**Files modified (~12):**
- 1 helper site — likely `crates/utils/src/lib.rs` or a `#[cfg(test)] mod test_init { ... }` in a shared utils submodule, OR a small new `crates/utils/src/test_init.rs` exposed under `#[cfg(test)]`. The exact location to be confirmed during planning Junior's read of `crates/utils/src/lib.rs` + `crates/api/api/src/lib.rs` (look for existing test-init scaffolding to extend rather than create new).
- 11 test files — add a one-line helper call at the top of each `#[cfg(test)]` block (or in each test fn that calls `LemmyContext::init_test_context()`):
  - `crates/api/api/src/federation/user_settings_backup.rs` (3 tests at lines 322, 393, 430)
  - `crates/api/api/src/federation/resolve_object.rs` (1 test at line 132)
  - `crates/api/api/src/site/mod_log.rs` (2 tests at lines 98, 400)
  - `crates/api/api/src/site/registration_applications/tests.rs` (1 test at line 128)
  - `crates/apub/objects/src/objects/comment.rs` (2 tests)
  - `crates/apub/objects/src/objects/community.rs` (1 test)
  - `crates/apub/objects/src/objects/instance.rs` (1 test)
  - `crates/apub/objects/src/objects/person.rs` (2 tests)
  - `crates/apub/objects/src/objects/post.rs` (2 tests)
  - `crates/apub/objects/src/objects/private_message.rs` (2 tests)
  - `crates/apub/objects/src/utils/markdown_links.rs` (2 tests)
  - `crates/apub/apub/src/collections/community_moderators.rs` (1 test)
  - `crates/apub/apub/src/http/community.rs` (4 tests)

**Path A (chosen) — shared `ensure_default_settings()` helper:**

1. Add a helper function with this shape (location to be confirmed at impl time):
   ```rust
   #[cfg(any(test, feature = "full"))]
   pub fn ensure_default_settings() {
     use std::sync::Once;
     static INIT: Once = Once::new();
     INIT.call_once(|| {
       // SAFETY: env var mutation before any thread reads SETTINGS LazyLock.
       unsafe { std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1"); }
     });
   }
   ```
   `Once` is preferred over `LazyLock<()>` because it explicitly captures "run exactly once" intent without owning state, and matches the `set_var` semantic (no value to return).

2. Add one call `lemmy_utils::ensure_default_settings();` (or whatever the chosen helper path is) at the top of each vulnerable test fn — OR call it once per `#[cfg(test)] mod tests` block via a `Once`-guarded helper. Pattern is identical to `crates/server/tests/e2e.rs:807, 2538` proven working.

3. If `lemmy_utils` is not in the `[dev-dependencies]` of `crates/apub/objects/Cargo.toml` or `crates/apub/apub/Cargo.toml`, add it. Likely already there — verify at impl time.

**Path B (NOT chosen) — modify `LemmyContext::init_test_context()`:**

Per Audit 4.3 "Long-term fix" recommendation. Touches a shared helper used across many test entry points; lower per-test boilerplate but higher blast radius. **Deferred to a follow-up DQ entry** so the v1-quality-r1 PR stays scope-limited and reviewable.

**Verification:**
- `bash scripts/brehon/cargo-test.bat -p lemmy_api --lib` → all tests pass (was the original failure trail).
- `bash scripts/brehon/cargo-test.bat -p lemmy_apub_objects --lib` → all tests pass (was previously masked by `cargo test --workspace --lib --no-run` not executing).
- `bash scripts/brehon/cargo-test.bat -p lemmy_apub --lib` → all tests pass (same masking).
- `bash scripts/brehon/cargo-test.bat --workspace --lib` from workspace root → all tests pass.
- `bash scripts/brehon/cargo-check.sh --workspace --features full` → exit 0 (no clippy/check regressions from the helper addition).

**Out of scope:** any change to `LazyLock` poison-on-init semantics; that's by Rust language design. Path B (modify `init_test_context()`) deferred to DQ.

## Phase 4 — latent-defect audit (read-only, three sweeps) — DONE

**Goal:** find recurrences of the same three defect classes elsewhere in the codebase before they manifest at runtime. Read-only investigations; each produced an audit report under `.claude/PRPs/reports/` in the audit worktree.

### Audit 4.1 — `CONFIG_KEY_METADATA` consumer drift — DONE

**Output:** `C:/Users/barri/Developer/brehon-fork-audit-2026-05-22/.claude/PRPs/reports/audit-config-key-metadata-consumers-2026-05-22.md`

**Findings:** 1 drifting consumer (`REQUIRES_RE_JURY_KEYS` — already in Phase 1 scope) + 2 safe filter-based consumers + 2 safe by-name lookups + 6 enum constants locked to single metadata entries. **No new defects** outside Phase 1 scope. Authoritative count = 27.

### Audit 4.2 — Test-only "parity-pin" / "drift-guard" patterns — DONE

**Output:** `C:/Users/barri/Developer/brehon-fork-audit-2026-05-22/.claude/PRPs/reports/audit-parity-pins-2026-05-22.md`

**Findings:** `REQUIRES_RE_JURY_KEYS` is the only true drift (Phase 1 covers). `EXPECTED_SEED_COUNT_V1_*` family validates totals but lacks per-phase breakdown — hygiene, not defect. **No new defects.** EXPECTED_SEED breakdown test deferred via Phase 5 DQ.

### Audit 4.3 — `Settings::SETTINGS` access from non-e2e test contexts — DONE

**Output:** `C:/Users/barri/Developer/brehon-fork-audit-2026-05-22/.claude/PRPs/reports/audit-settings-lazy-lock-callers-2026-05-22.md`

**Findings:** 19 vulnerable tests across 11 files (much broader than the 8 cascading failures originally observed). Caused Phase 3 scope expansion from `lemmy_api`-only to workspace-wide. **Three additional follow-up DQ entries** filed in Phase 5 (init_test_context auto-set, other LazyLock/OnceLock scan, CI workflow env var).

## Phase 5 — file DQ entries + bundle into one PR

After Phases 1–4 complete:

1. **DQ entry for clippy** (deferred per user decision):
   - `kind: "log"`, `from: "advisor"`, `answered_by: "advisor"`, scope `crates/api/api/src/governance/admin_config.rs` + `reputation_snapshot.rs`.
   - Content: 8 errors enumerated (5 test fns affected); fix recipe per `feedback_clippy_test_style.md` (migrate to `LemmyResult<()>` + `?`); estimated effort: 1 task, ~30 min impl.
2. **DQ entry for v1-JM-b read-side flip** (Phase 1 boundary):
   - `kind: "log"`, `from: "advisor"`, `answered_by: "advisor"`.
   - Content: snapshot helper now pins all 27 keys; readers in `admin_assign_jury.rs` / `submit_jury_vote.rs` / `sponsor_liability.rs` still read live config for the 20 v1-JM-a keys; ADR-010 violation persists for in-flight cases opened before this plan ships AND for new cases until v1-JM-b flips the readers.
3. **DQ entry for fmt CI gate** (Phase 2 boundary, optional):
   - `kind: "log"`, `from: "advisor"`, `answered_by: "advisor"`.
   - Content: `.github/workflows/cargo-validate-workspace.yml` does not currently run `cargo fmt --check`; recommend adding to prevent future drift recurrence.
4. **Phase 4 audit DQ entries** (from completed audits):
   a. **`EXPECTED_SEED_COUNT_V1_*` per-phase breakdown test** (`kind: "log"`, scope `crates/api/api/src/governance/config.rs`). Per Audit 4.2: total-count test catches the most likely defect class but doesn't catch a "key moved between phases without const adjustment" footgun. Hygiene, not defect. Deferred per user decision 2026-05-22.
   b. **`init_test_context()` auto-set env var** (`kind: "log"`, scope `crates/api/api_common/src/context.rs` or wherever `init_test_context` lives). Per Audit 4.3 "Long-term fix": modify the helper to call `ensure_default_settings()` itself, removing the per-test boilerplate added in Phase 3. Smaller follow-up; Phase 3 is the immediate fix that doesn't risk init_test_context blast radius.
   c. **Other `LazyLock`/`OnceLock` statics for poisoning risk** (`kind: "log"`, scope workspace). Per Audit 4.3 last recommendation: scan `crates/apub/objects/src/utils/functions.rs:CACHE` + any other `LazyLock`/`OnceLock`. Single follow-up audit task.
   d. **CI workflow `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS=1` for lib-test runs** (`kind: "log"`, scope `.github/workflows/`). Per Audit 4.3: GitHub CI today only runs e2e (single binary), but if lib tests are ever added to CI, this should be set at the workflow level too. Hygiene.
5. **PR** opens from `phase-v1-quality-r1` → `governance-v0` per `.claude/rules/phase-branch.md`. PR body lists each phase + the DQ entries above as "tracked follow-ups". Include `--repo barrie-cork/lemmy` (mandatory per `.claude/rules/gh-pr-fork-target.md`).

## Verification — end-to-end

After all phases complete, on `phase-v1-quality-r1` branch:

1. `bash scripts/brehon/cargo-check.sh --workspace --features full` → exit 0 (no regressions).
2. `cargo fmt --all -- --check` → exit 0 (Phase 2 fixed).
3. `bash scripts/brehon/cargo-test.bat --workspace --lib` from workspace root → all tests pass (Phase 3 fixed cascade workspace-wide; Phase 1 fixed parity test). Plus per-crate spot-checks: `bash scripts/brehon/cargo-test.bat -p lemmy_api --lib` and `bash scripts/brehon/cargo-test.bat -p lemmy_apub_objects --lib` and `bash scripts/brehon/cargo-test.bat -p lemmy_apub --lib` — each must pass from its crate-local CWD. Specifically verify `governance::case_open_snapshot::parity::snapshot_keyset_matches_requires_re_jury_metadata` is green.
4. `bash scripts/brehon/cargo-test.bat --workspace --test e2e --features full` → 103+ pass / 0 fail (was already green; confirm no regression).
5. `cargo clippy --workspace --features full --all-targets -- -D warnings` → still exits 101 (clippy fix is deferred per Phase 5 DQ); the 8 errors must still be the SAME 8 (no new clippy regressions from Phases 1–3).
6. Spot-read the new snapshot helper output: pick an arbitrary case-open seed from `crates/server/tests/e2e.rs` v1-AD-c fixtures, confirm the JSONB has 27 keys (read the test logs or write a one-off println).
7. Phase 4 audit reports exist at the cited paths and are non-empty.

## Out of scope — explicitly NOT in this plan

- Clippy fixes for `admin_config.rs` + `reputation_snapshot.rs` (DQ entry only).
- v1-JM-b jury-reader flip to snapshot consumer (DQ entry only).
- CI workflow update to add `cargo fmt --check` (optional DQ).
- Any change to `crates/utils/src/settings/mod.rs` init logic (Path B in Phase 3 explicitly rejected).
- Any work on the v1-federation-inbound-d planning currently in flight.
- Any fix derived from the Phase 4 audits — those produce reports + DQ entries only; the actual fixes are a future plan once user reviews the audit findings.

## Audit reports (cross-worktree refs)

These reports live in the read-only audit worktree `C:/Users/barri/Developer/brehon-fork-audit-2026-05-22/` and are NOT staged into this branch. They are referenced for context only; if you need to read them from this worktree, either:
- read them directly at the cross-worktree paths below, OR
- copy them into `.claude/PRPs/reports/` on this branch in a separate `chore(reports):` commit if you want the trail bundled with the PR.

Paths:
- `.claude/PRPs/reports/audit-config-key-metadata-consumers-2026-05-22.md`
- `.claude/PRPs/reports/audit-parity-pins-2026-05-22.md`
- `.claude/PRPs/reports/audit-settings-lazy-lock-callers-2026-05-22.md`

Also: `.claude/test-e2e-audit-2026-05-22.log`, `test-noe2e-audit-2026-05-22.log`, `clippy-audit-2026-05-22.log`, `fmt-check-audit-2026-05-22.log` are the original validation logs that surfaced the four defects.
