# impl-task Brief — v1-deps-r1 Task 2: sha2 0.11 migration

**Role:** [role:impl-task]
**Phase:** v1-deps-r1
**Task:** 2 (workspace `sha2` bump + 9-callsite API-preservation verification; single commit)
**Branch:** phase-v1-deps-r1
**Authored:** 2026-05-25

---

## 1. Role + dispatch line

[role:impl-task] v1-deps-r1 task 2 sha2 0.11 migration — see .claude/PRPs/briefs/v1-deps-r1-impl-2.md

---

## 2. Scope

Per plan §13 Task 2 + plan §4 Task 2 + plan §10.4. Bump `sha2 = "0.10"` → `sha2 = "0.11"` in workspace `Cargo.toml:185`. Verify the 9 existing `Sha256` callsites (2 production + 7 test) compile clean under sha2 0.11 with zero source-diff. **Single commit.**

Expected outcome is **zero source-diff beyond the Cargo.toml + Cargo.lock bump**. sha2 0.11's `Digest` trait preserves `new() / update() / finalize() / digest()` on `Sha256`; the type-alias-to-newtype shift only breaks code that stores `Sha256` in a struct field with a `Digest`-bounded generic parameter — plan-author-time `rg "impl.*Digest" crates/ tests/` returned **zero matches**, so no such storage exists.

**The 9 callsites at HEAD (verified by advisor 2026-05-25 pre-brief):**

```
crates/api/api/src/governance/admin_rule_sets.rs:46   use sha2::{Digest, Sha256};
crates/api/api/src/governance/admin_rule_sets.rs:120  let text_sha256 = Sha256::digest(data.rule_text.as_bytes()).to_vec();
crates/server/tests/e2e.rs:938                        use sha2::{Digest, Sha256};
crates/server/tests/e2e.rs:1020                       let mut hasher = Sha256::new();
crates/server/tests/e2e.rs:2526                       use sha2::{Digest, Sha256};
crates/server/tests/e2e.rs:3074                       let mut hasher = Sha256::new();
crates/server/tests/e2e.rs:7302                       use sha2::{Digest, Sha256};
crates/server/tests/e2e.rs:7320                       let text_sha256_a = Sha256::digest(b"seeded A".as_slice()).to_vec();
crates/server/tests/e2e.rs:7321                       let text_sha256_b = Sha256::digest(b"would-be B".as_slice()).to_vec();
```

Total: **9 hits** (matches Task 0 Probe 8d count exactly; no drift since plan-author time + T1 ship).

`sha2` is workspace-pinned at `Cargo.toml:185` and re-exported as `{ workspace = true }` from `crates/server/Cargo.toml:75` + `crates/api/api/Cargo.toml:86`. **No sub-crate `Cargo.toml` edits needed** — the single workspace bump propagates.

**Out of scope:**
- No edits to T1's diesel-async wrapper (`crates/diesel_utils/src/connection.rs`) or any `.run_transaction` callsite.
- No T3 SemVer-compat bumps (diesel 2.3.7→2.3.9, tokio 1.50→1.52, etc — those are Task 3).
- No e2e test-body restructuring. If sha2 0.11 forces a test-body edit, raise `kind: "blocker"` DQ per §4.6 below — do NOT improvise.
- No new `Sha256` callsites; no new `sha2::*` imports anywhere.
- No `std` feature flip (workspace pin is bare-version `sha2 = "0.10"`; bare-version `sha2 = "0.11"` is the target; default features only, per plan §4 Task 2 step 5).

---

## 3. Required reading

### 3.1 Plan sections (cite by line range)

- `.claude/PRPs/plans/v1-deps-r1.plan.md` §2 (sha2 0.11.0 changelog cite — "Replace type aliases with newtypes (#678). Removed `std` crate feature; `alloc` available.")
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §4 Task 2 (full scope, lines 94–100)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §8 (before/after state for sha2 lines)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §9 Task 2 (mandatory reading)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §10.4 (sha2 0.11 usage MIRROR, lines 540–560 — "expected zero-diff" rationale)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §13 Task 2 (full IMPLEMENT discipline — authoritative)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §15 (DoD commands, wrapper-prefixed)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §16a Story 2 (story checkpoint)

### 3.2 Mandatory lessons (per advisor-orchestrator.md §2.4 file-class injection)

- `feedback_features_full_workspace_only.md` — every cargo command uses `--workspace --features full`
- `feedback_features_full_p_crate_incompatible.md` — companion
- `feedback_pq_sys_wrapper_env_propagation.md` — wrapper-prefix discipline
- `feedback_wrapper_script_flag_silence.md` — flags after `--` reach cargo, before reach the wrapper
- `feedback_pipes_mask_exit_codes.md` — write rg to file then `wc -l`
- `feedback_fix_impl_enumerate_all_callsites.md` — rg-enumerate-first
- `feedback_fix_impl_pre_push_cargo_check.md` — Step 5 cargo check MANDATORY before push
- `feedback_verify_files_with_read.md` — Read before Edit

### 3.3 Test-touched-file lessons (per advisor-orchestrator.md §2.4 row "crates/server/tests/e2e.rs (any edit, any size)")

- `feedback_lemmy_error_no_std_error.md` — Case A / B / C enum. T2 expects **zero test-body edits**; if a body edit becomes necessary (e.g. type annotation), the impl-task verifies the touched fn already returns `LemmyResult<()>` Case A; if not, raise `kind: "blocker"` DQ.
- `feedback_async_pool_test_pattern.md` — Non-applicable to T2's expected zero-diff outcome; cited for awareness only.

### 3.4 Handover from prior task (Task 1)

```yaml
prior_task:
  task: 1
  commit: 7bd047f2d
  type: diesel-async 0.8 → 0.9 migration (wrapper + 38 callsites + 30 imports + Shape D)
  outcome: shipped + validated (validate-pending-laptop-e2e DQ a22859c2ae07-002 → pass)
  validation_pass:
    cargo_workspace_check: 0
    cargo_workspace_clippy: 0
    lemmy_api_lib: 35/35 pass
    lemmy_db_schema_lib: 37/37 pass
    workspace_e2e: 109/109 pass
    lemmy_apub_lib: 2/7 pass — pre-existing failures (NOT T1-caused; falsified vs 2f0ab6a87)
  apub_libtest_note: |
    5 lemmy_apub lib tests fail pre-existing on phase-v1-deps-r1 AND on
    governance-v0 trunk. Falsification: same failure signature reproduces
    at commit 2f0ab6a87 (pre-T1 baseline). File
    crates/apub/apub/src/http/community.rs unchanged between governance-v0
    and phase-v1-deps-r1. Tracked as DQ a22859c2ae07-003 kind:log; NOT
    blocking T2 ship. Workspace e2e (109/109) exercises the same
    federation flows successfully through testcontainers + real http
    stack. Expect the same 2/7 pass on T2's libtest gate; do NOT
    investigate or attribute to T2.
  wrapper_pattern_T2_inherits:
    file: crates/diesel_utils/src/connection.rs
    pattern: LocalAsyncFunc mirror trait + blanket impl + AsyncFnOnce-bounded run_transaction
    note: |
      T2 does NOT touch this file. The wrapper is the substrate for any
      future sub-phase that adds new `.run_transaction` callsites; sha2
      0.11 has zero interaction with it.
  workspace_state:
    scoped_futures_refs: 0  # workspace-wide; rg "scoped_futures" crates/ → 0
    scope_boxed_refs: 2     # both intentional doc-comment literals in connection.rs
    run_transaction_callsites: 38
  notes: |
    diesel-async 0.9 wrapper + callsites complete. T2 (sha2 0.11) sees
    a clean workspace under cargo check + clippy + workspace e2e. 9
    Sha256 callsites per Task 0 Probe 8d unaffected by T1 (no
    structural overlap; sha2 and diesel-async are independent dep
    graphs).
```

---

## 4. Constraints

### 4.1 Hard rules (refuse if violated)

- **No commits to other tasks' files.** Authoritative file list: `Cargo.toml` + `Cargo.lock` (auto-regen). Optionally one of `crates/api/api/src/governance/admin_rule_sets.rs` OR `crates/server/tests/e2e.rs` IF AND ONLY IF cargo surfaces a clippy/deprecation warning under `-D warnings` (per §4.6 escape hatch).
- **Single commit.** `Cargo.toml` + `Cargo.lock` in ONE commit. Subject: `feat(deps): migrate to sha2 0.11 (task 2)`.
- **Wrapper-prefix every cargo command on Windows laptop side.** The worker is on Linux EliteDesk so use the `.sh` form: `bash scripts/brehon/cargo-check.sh ...`. Advisor laptop running §4.4 §15 commands uses `cmd //c "scripts\\brehon\\cargo-*.bat ..."`.
- **Step 1 callsite count MUST equal 9.** Enumerate via `rg "Sha256\b" crates/ tests/ > .claude/PRPs/debug/v1-deps-r1-task2-enumerate.log; wc -l .claude/PRPs/debug/v1-deps-r1-task2-enumerate.log`. Expected: 9. If drift (≠9), file `kind: "blocker"` DQ via `bash scripts/brehon/dq-v3-append-fragment.sh --pending` and STOP.
- **Pre-flight grep MUST return empty.** `rg "impl.*Digest" crates/ tests/ > .claude/PRPs/debug/v1-deps-r1-task2-impl-digest.log; wc -l .claude/PRPs/debug/v1-deps-r1-task2-impl-digest.log`. Expected: 0. If non-zero, file `kind: "blocker"` DQ — a struct field with a `Digest`-bounded type parameter has been introduced since plan-author time, which IS sensitive to the 0.11 type-alias-to-newtype shift; the impl-task does NOT improvise around this; the advisor decides.
- **`std` feature check.** `rg "sha2.*features.*std" crates/ Cargo.toml > .claude/PRPs/debug/v1-deps-r1-task2-std-feature.log; wc -l .claude/PRPs/debug/v1-deps-r1-task2-std-feature.log`. Expected: 0. If non-zero, raise `kind: "blocker"` DQ — sha2 0.11 removes the `std` feature; an explicit dependency on it surfaces as a build break and requires a `default-features = false, features = ["alloc"]` migration this brief does NOT cover.
- **Zero source-diff is the expected outcome.** If `cargo check --workspace --features full` exits clean after only the Cargo.toml bump, that IS the deliverable. Do NOT preemptively edit any of the 9 callsites "to be safe".
- **Step 5 pre-push cargo check is MANDATORY** per `feedback_fix_impl_pre_push_cargo_check.md`. Non-zero exit → patch in same commit if in-scope (§4.6 escape hatch), else `kind: "blocker"` DQ. NEVER `#[allow]`-spam.
- **Read before Edit** (Brehon hard rule).
- **NO subagent batching** for T2. Plan §5.2 ceiling is `<=4 files / <=2 crates`; T2 expected scope is 2 files (Cargo.toml + Cargo.lock). Worker context is fine for serial execution. The T1 §4.7 carve-out does NOT extend to T2.

### 4.2 DQ discipline

- Use `bash scripts/brehon/dq-v3-new-entry.sh` to generate composite id.
- Use `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> [--pending]` to append.
- Commit + push immediately after raising any `kind: "blocker"` DQ.
- **NEVER write `answered_by: "advisor"`** (Hard refusal #1).
- **NEVER write `kind: "clarify"`** (advisor-only kind).

### 4.3 File-write discipline (worker is on Linux EliteDesk)

- Write rg output to files first, then `wc -l` the files (per `feedback_pipes_mask_exit_codes.md`).
- Use worktree-internal paths: `.claude/PRPs/debug/v1-deps-r1-task2-*.log`.
- Capture cargo logs via `> .log 2>&1` redirect; check `$?` immediately after.

### 4.4 Post-commit DQ: raise `kind: "validate-pending-laptop-e2e"`

Per plan §13 T2 + plan DQ `a3d0e9941441-016`. After commit + push, raise ONE `kind: "validate-pending-laptop-e2e"` DQ with `commands[]` (IDENTICAL to T1's 6 §15 commands per the plan's per-task class-targeted answer (a)):

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-deps-r1-task2-validate-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full -- -D warnings > .claude/PRPs/debug/v1-deps-r1-task2-validate-clippy.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_api --features full --lib > .claude/PRPs/debug/v1-deps-r1-task2-validate-libtest-api.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_apub --features full --lib > .claude/PRPs/debug/v1-deps-r1-task2-validate-libtest-apub.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_db_schema --features full --lib > .claude/PRPs/debug/v1-deps-r1-task2-validate-libtest-dbschema.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-deps-r1-task2-validate-e2e.log 2>&1"'
```

`branch: phase-v1-deps-r1`, `phase_task: 2`. Advisor laptop session runs these locally (Shape G SUSPENDED until 2026-06-01).

**Advisor-laptop env recipe (reproducible from session-retro-2026-05-25-t1-validate-gate-cleared.md):**

```bash
# Spin ephemeral dev-PG once per advisor session if not already up:
docker run -d --name lemmy-deps-pg -p 5433:5432 \
  -e POSTGRES_USER=lemmy -e POSTGRES_PASSWORD=password -e POSTGRES_DB=lemmy \
  pgautoupgrade/pgautoupgrade:18.4-alpine

# Export per shell session:
export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5433/lemmy
export LEMMY_CONFIG_LOCATION="$PWD/config/config.hjson"
# NOTE: do NOT export LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS=1 — it loads
# hostname="unset" from defaults.hjson and breaks 3 lemmy_db_schema
# test_crud tests (codified as footgun in T1 retro §"What to change").

# e2e self-provisions testcontainers — env is irrelevant for the e2e
# command but harmless to have exported.
```

### 4.5 HANDOVER trailer

Commit body MUST end with:

```yaml
HANDOVER:
  task: 2
  files_modified: 2  # Cargo.toml + Cargo.lock
  callsite_counts:
    sha256_pre: 9
    sha256_post: 9   # preserved (zero source-diff expected)
  cargo_check_status: 0
  cargo_lock_delta_lines: <actual diff line count>
  source_diff_lines: 0  # if non-zero, populate with actual count + reason
  notes: |
    sha2 0.10 → 0.11 bump complete. Zero source-diff outcome: API
    preservation (Digest::new / update / finalize / digest on Sha256)
    confirmed clean under workspace cargo check. The 9 callsites at
    admin_rule_sets.rs (2) + e2e.rs (7) compiled clean as-is. No
    `impl.*Digest` blocks introduced since plan-author time
    (preflight rg → 0). No `sha2.*features.*std` deps surfaced
    (workspace pin is bare-version, default features only).
    T3 (10 SemVer-compat bumps) sees a workspace where sha2 is on
    0.11 and the LocalAsyncFunc wrapper from T1 is untouched.
```

If `Cargo.lock` diff exceeds ±200 lines, raise `kind: "log"` DQ noting the lock churn (sha2 has few dependents in this workspace; large lock delta would be surprising and worth recording for T3 awareness).

### 4.6 Escape hatch: warning-only source edits in same commit

If `cargo check --workspace --features full` exits clean but `cargo clippy --workspace --features full -- -D warnings` surfaces a sha2-0.11-related warning at one of the 9 callsites (e.g. a deprecation notice on `Sha256::new` recommending a replacement), the impl-task MAY:

1. Apply the canonical fix per `feedback_clippy_test_style.md` (typically the suggested replacement; never `#[allow]`).
2. Re-run `cargo check` + `cargo clippy` to confirm zero exit.
3. Include the edit in the same single commit.
4. Update §4.5 HANDOVER trailer's `source_diff_lines:` field with the actual count + a one-line reason ("clippy deprecation fix at admin_rule_sets.rs:120 per sha2 0.11 changelog").

If the warning suggests a non-mechanical fix (struct refactor, trait re-bound, generic parameter change), STOP and raise `kind: "blocker"` DQ — the advisor decides. Do NOT improvise structural changes.

**Trait-resolution failure path:** If `cargo check` itself fails with `E0277: trait bound not satisfied` mentioning `Digest` or `Sha256` (unexpected per plan §10.4 risk = low assessment), raise `kind: "blocker"` DQ immediately with the failing log slice. The 0.11 type-alias-to-newtype shift has surfaced an interaction the plan-author-time `rg "impl.*Digest"` probe missed. Do NOT improvise; the advisor decides whether to (a) extend T2 scope, (b) split the failing site into a fix-impl, or (c) defer the bump.

---

## 5. Validation gate (single-task)

This task's per-task validation gate IS the §15 DoD subset listed in §4.4. The advisor laptop runs them post-commit when it picks up the `kind: "validate-pending-laptop-e2e"` DQ. **Worker does NOT run e2e — only the §4.7 (sic, actually §4.1 Step 5 hard-rule) in-task `cargo check --workspace --features full`.** E2E + per-crate lib-tests are advisor-laptop only.

Expected validate-pending-laptop-e2e outcome (identical to T1):
- `cargo check --workspace --features full`: exit 0
- `cargo clippy --workspace --features full -- -D warnings`: exit 0
- `lemmy_api --lib`: 35/35 pass
- `lemmy_apub --lib`: 2/7 pass (pre-existing failures per §3.4 apub_libtest_note — NOT T2-attributable; advisor cites DQ a22859c2ae07-003 in result attribution)
- `lemmy_db_schema --lib`: 37/37 pass
- `workspace e2e`: 109/109 pass

If the apub libtest count CHANGES from 2/7 (e.g. drops to 1/7 or rises to 3/7), surface to advisor for falsification — sha2 0.11 is highly unlikely to interact with federation http handlers but the change in failure shape would be diagnostic.

---

## 6. Post-task

- Commit + push to `phase-v1-deps-r1` worker branch (daemon finalize-merges to `phase-v1-deps-r1`).
- Raise the §4.4 `validate-pending-laptop-e2e` DQ. Commit + push immediately.
- DO NOT proceed to T3 dispatch — advisor reviews validate-pending result first.
- Junior task complete when DQ is raised + pushed; advisor takes over for validation cycle.
