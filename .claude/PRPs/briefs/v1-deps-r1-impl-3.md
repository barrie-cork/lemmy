# impl-task Brief — v1-deps-r1 Task 3: 9 SemVer-compatible dep bumps

**Role:** [role:impl-task]
**Phase:** v1-deps-r1
**Task:** 3 (5 workspace `Cargo.toml` bumps + 4 sub-crate `Cargo.toml` bumps; Cargo-only; single commit)
**Branch:** phase-v1-deps-r1
**Authored:** 2026-05-25 (pre-authored in parallel with T2 dispatch)

---

## 0. SCOPE ADJUSTMENT vs plan §4 Task 3 (read this first)

Plan §4 + §11 + §13 list T3 as **10 bumps** (6 workspace + 4 sub-crate). **Re-verification at brief-author time shows the workspace `Cargo.toml` already has `diesel = "=2.3.9"` at line 174 AND `diesel-async = "0.9.0"` at line 182** (T1 commit `7bd047f2d` co-bumped diesel 2.3.7→2.3.9 because the OrderDsl bound tightening required a `.select(language::all_columns)` reorder fix at `crates/db_schema/src/impls/language.rs:23-24` that depends on diesel ≥2.3.8's revised trait coherence). The diesel bump is therefore **NOT in T3's scope** — it landed in T1.

**T3 final scope: 9 bumps (5 workspace + 4 sub-crate).** All other plan §13 T3 IMPLEMENT steps remain unchanged.

---

## 1. Role + dispatch line

[role:impl-task] v1-deps-r1 task 3 — 9 SemVer-compat dep bumps — see .claude/PRPs/briefs/v1-deps-r1-impl-3.md

---

## 2. Scope

Per plan §13 Task 3 + plan §4 Task 3 (adjusted per §0 above). Pure `Cargo.toml` + `Cargo.lock` change; **no `crates/**/src/**.rs` edits expected**. All 9 bumps are SemVer-compatible (Dependabot classification) so no consuming-code change should be needed. **Single commit.**

**The 9 bumps at HEAD (verified by advisor 2026-05-25 pre-brief):**

| File | Line | Current → Target |
|---|---|---|
| `Cargo.toml` (workspace) | 184 | `serde_with = "3.18.0"` → `"3.20.0"` |
| `Cargo.toml` (workspace) | 210 | `bcrypt = "0.19.0"` → `"0.19.1"` |
| `Cargo.toml` (workspace) | 221 | `tokio = { version = "1.50.0", features = ["full"] }` → `"1.52.0"` |
| `Cargo.toml` (workspace) | 237 | `rustls = { version = "0.23.37", features = ["ring"], default-features = false }` → `"0.23.40"` |
| `Cargo.toml` (workspace) | 246 | `html2text = "0.16.7"` → `"0.17.1"` |
| `crates/api/api_utils/Cargo.toml` | 77 | `jsonwebtoken = { version = "10.3.0", features = ["rust_crypto"] }` → `"10.4.0"` |
| `crates/email/Cargo.toml` | 33 | `lettre = { version = "0.11.19", default-features = false, features = [...] }` → `"0.11.22"` |
| `crates/routes/Cargo.toml` | 59 | `rss = "2.0.12"` → `"2.0.13"` |
| `crates/utils/Cargo.toml` | 87 | `dashmap = { version = "6.1.0", optional = true }` → `"6.2.x"` (verify latest 6.2.y at task-time per WP-6) |

Line numbers are at advisor-brief-author time; T1's commit may have shifted some (cargo doesn't reformat the file, so shifts will be ≤±5 lines). **Verify each line via `grep -n "<dep>" <Cargo.toml>` at task-time** before substituting.

**Out of scope:**
- **No diesel bump** (already at 2.3.9 from T1).
- **No diesel-async bump** (already at 0.9.0 from T1).
- **No sha2 bump** (T2 ships before T3; expected at 0.11 from T2).
- No `crates/**/src/**.rs` edits. If a SemVer-compat bump surfaces an API regression on a consuming crate, **STOP and file `kind: "blocker"` DQ** — per plan §12, that becomes a follow-up sub-phase, not a fold-in.
- No `npm/pnpm` dep bumps in `api_tests/` — those land via Dependabot PRs directly.
- No transitive dep pins (`html5ever 0.39` / `markup5ever 0.39` come along with `html2text 0.17.1`; we do NOT pin them directly).
- No `.coderabbit.yaml` changes.
- No feature-flag drift — preserve each dep's `features = [...]` and `default-features = false` configuration verbatim during the version bump.

---

## 3. Required reading

### 3.1 Plan sections (cite by line range)

- `.claude/PRPs/plans/v1-deps-r1.plan.md` §2 (html2text 0.17.1 changelog cite — confirms no user-visible API change)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §4 Task 3 (full scope, lines 102–115)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §8 (before/after state)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §9 Task 3 (mandatory reading)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §10.5 (Cargo.toml pin pattern MIRROR)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §13 Task 3 (full IMPLEMENT discipline — authoritative; this brief carries the §0 scope adjustment)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §14 (testing strategy — T3 single-task gate is check + clippy, NOT e2e; phase-tip e2e is advisor-raised post-T3)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §15 (DoD commands, wrapper-prefixed)
- `.claude/PRPs/plans/v1-deps-r1.plan.md` §16a Story 3 (story checkpoint)

### 3.2 Mandatory lessons (per advisor-orchestrator.md §2.4 file-class injection)

- `feedback_features_full_workspace_only.md` — every cargo command uses `--workspace --features full`
- `feedback_features_full_p_crate_incompatible.md` — companion
- `feedback_pq_sys_wrapper_env_propagation.md` — wrapper-prefix discipline
- `feedback_wrapper_script_flag_silence.md` — flags after `--` reach cargo, before reach the wrapper
- `feedback_pipes_mask_exit_codes.md` — write rg to file then `wc -l`
- `feedback_fix_impl_enumerate_all_callsites.md` — verify each line number at task-time
- `feedback_fix_impl_pre_push_cargo_check.md` — Step 5 cargo check MANDATORY before push
- `feedback_verify_files_with_read.md` — Read before Edit
- `feedback_verify_all_bumped_packages_systematically.md` — explicit per-dep verification list

### 3.3 Handover from prior task (Task 2)

**(Populated by advisor before T3 queue, AFTER T2 validate-pending-laptop-e2e DQ → PASS. Template placeholder follows; advisor replaces with verbatim T2 HANDOVER trailer block extracted from `git log -1 --format=%B <T2-sha>`.)**

```yaml
prior_task:
  task: 2
  commit: <T2-sha — populate after T2 ship>
  type: sha2 0.10 → 0.11 migration (workspace bump only; 9 Sha256 callsites preserved)
  outcome: <populate from T2 validate-pending-laptop-e2e DQ result>
  validation_pass:
    cargo_workspace_check: <0 expected>
    cargo_workspace_clippy: <0 expected>
    lemmy_api_lib: <35/35 expected>
    lemmy_db_schema_lib: <37/37 expected>
    workspace_e2e: <109/109 expected>
    lemmy_apub_lib: <2/7 expected — pre-existing failures per DQ a22859c2ae07-003>
  source_diff_lines: <0 expected, per plan §10.4 risk = low>
  workspace_state_at_T2_tip:
    diesel: "=2.3.9"          # carried from T1
    diesel-async: "0.9.0"     # carried from T1
    sha2: "0.11"              # set by T2
    scoped_futures_refs: 0    # from T1
    scope_boxed_refs: 2       # both intentional doc-comment literals in connection.rs
    run_transaction_callsites: 38  # from T1
  notes: |
    sha2 0.11 in. T3 (10 SemVer-compat bumps, reduced to 9 per brief §0
    scope adjustment — diesel already in from T1) sees a workspace where
    diesel-async / diesel / sha2 are all at their target versions. T3
    bumps only touch Cargo.toml + Cargo.lock; no Rust source diff expected.
```

If T2 shipped with the apub libtest count CHANGED from 2/7 (e.g. dropped to 1/7), surface in §3.3 notes — sha2 0.11 is highly unlikely to interact with federation http handlers but the change would be diagnostic for T3 awareness.

---

## 4. Constraints

### 4.1 Hard rules (refuse if violated)

- **No commits to other tasks' files.** Authoritative file list: 5 `Cargo.toml` files + `Cargo.lock` (auto-regen). NO `crates/**/src/**.rs` edits. If cargo surfaces an API regression at a consuming crate, file `kind: "blocker"` DQ per §4.6 (do NOT improvise; do NOT widen scope to fix-in-task).
- **Single commit.** All 9 bumps + `Cargo.lock` in ONE commit. Subject: `feat(deps): bump 9 SemVer-compatible deps (task 3)`.
- **Wrapper-prefix every cargo command on Windows laptop side.** The worker is on Linux EliteDesk so use the `.sh` form: `bash scripts/brehon/cargo-check.sh ...`. Advisor laptop running §4.4 §15 commands uses `cmd //c "scripts\\brehon\\cargo-*.bat ..."`.
- **Verify each line number BEFORE substituting.** Use `grep -n "<dep-name>" <file>` to confirm the line. T1's commit shifted some workspace `Cargo.toml` lines (cargo doesn't reformat but column-1 entries may have moved by ±5 lines).
- **Verify each target version at task-time** (per brief WP-6). For each dep, `cargo search <name>` and pick the latest matching SemVer-compat patch. If a NEWER patch than this brief specifies has been released (e.g. `tokio 1.53.x`), bump to the newer patch — capture the chosen version in the HANDOVER trailer.
- **Preserve feature lists + `default-features = false` blocks verbatim.** Only the version pin substitutes.
- **No `crates/**/src/**.rs` edits** as a hard refusal (single shipped exception: if a `#[allow(deprecated)]` line is the entire fix per `feedback_clippy_test_style.md` AND it's a one-line addition AND it's covered by the clippy/check log — STILL file `kind: "blocker"` DQ first; advisor decides whether to authorise in-scope).
- **Step 5 pre-push cargo check is MANDATORY** per `feedback_fix_impl_pre_push_cargo_check.md`. Non-zero exit → `kind: "blocker"` DQ. NEVER `#[allow]`-spam.
- **Read before Edit** (Brehon hard rule).
- **NO subagent batching** for T3. Scope is 5 files, well under Sonnet ceiling — serial worker execution. The T1 §4.7 carve-out does NOT extend to T3.

### 4.2 DQ discipline

- Use `bash scripts/brehon/dq-v3-new-entry.sh` to generate composite id.
- Use `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> [--pending]` to append.
- Commit + push immediately after raising any `kind: "blocker"` DQ.
- **NEVER write `answered_by: "advisor"`** (Hard refusal #1).
- **NEVER write `kind: "clarify"`** (advisor-only kind).

### 4.3 File-write discipline (worker is on Linux EliteDesk)

- Write rg / grep / cargo output to files first, then process (per `feedback_pipes_mask_exit_codes.md`).
- Use worktree-internal paths: `.claude/PRPs/debug/v1-deps-r1-task3-*.log`.
- Capture cargo logs via `> .log 2>&1` redirect; check `$?` immediately after.

### 4.4 Post-commit DQ: raise `kind: "validate-pending-laptop"`

Per plan §13 T3 + plan DQ `a3d0e9941441-016`. After commit + push, raise ONE `kind: "validate-pending-laptop"` DQ (NOT `-e2e` — T3 class is Cargo-only; e2e runs at phase-tip via separate advisor-raised entry per §4.4b) with `commands[]`:

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-deps-r1-task3-validate-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-deps-r1-task3-validate-clippy.log 2>&1"'
```

`branch: phase-v1-deps-r1`, `phase_task: 3`. Advisor laptop session runs these locally (Shape G SUSPENDED until 2026-06-01).

### 4.4b Phase-tip e2e gate (post-T3, advisor-raised)

Per plan §13 T3 final paragraph + DQ `a3d0e9941441-016` answer (a). **Do NOT raise this entry from the worker.** After T3's `kind: "validate-pending-laptop"` resolves PASS, the **advisor session** raises a NEW `kind: "validate-pending-laptop-e2e"` entry from `from: "advisor"` (subject must match `^chore\(advisor\)`) referencing the post-T3 phase-branch tip, with the single command:

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-deps-r1-phase-tip-e2e.log 2>&1"'
```

Result MUST be `pass` (109/109; no skips, no flakes) before opening the PR via bm-pr.

The advisor-laptop env recipe is identical to T1/T2 (see §4.4 in v1-deps-r1-impl-2.md): Docker `pgautoupgrade:18.4-alpine` on port 5433 with canonical lemmy/password creds + `LEMMY_DATABASE_URL` + `LEMMY_CONFIG_LOCATION`. The e2e command itself self-provisions testcontainers — env is irrelevant for e2e but harmless to have exported. **NEVER export `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS=1`** (footgun — loads hostname="unset" from defaults.hjson, breaks 3 test_crud tests).

### 4.5 HANDOVER trailer

Commit body MUST end with:

```yaml
HANDOVER:
  task: 3
  files_modified: 6  # 5 Cargo.toml files + Cargo.lock
  bumps_applied: 9   # serde_with + bcrypt + tokio + rustls + html2text (workspace) + jsonwebtoken + lettre + rss + dashmap (sub-crate)
  bumps_skipped_vs_plan:
    - dep: diesel
      reason: "Already at =2.3.9 from T1 commit 7bd047f2d (OrderDsl trait coherence fix forced co-bump)"
  versions_chosen:
    serde_with: "<final 3.20.x or later>"
    bcrypt: "<final 0.19.x>"
    tokio: "<final 1.52.x or later>"
    rustls: "<final 0.23.40 or later>"
    html2text: "<final 0.17.1 or later>"
    jsonwebtoken: "<final 10.4.x>"
    lettre: "<final 0.11.22 or later>"
    rss: "<final 2.0.13>"
    dashmap: "<final 6.2.x>"
  cargo_check_status: 0
  cargo_clippy_status: 0
  cargo_lock_delta_lines: <actual diff line count>
  source_diff_lines: 0   # if non-zero (i.e. a consuming crate needed an in-scope fix), document the file + reason
  notes: |
    9 SemVer-compat bumps in (vs plan's 10 — diesel already bumped in T1
    per §0). Workspace check + clippy green. Cargo.lock delta within
    expected envelope. Phase-tip e2e gate raised by advisor as a NEW
    validate-pending-laptop-e2e entry (NOT in this trailer's scope).
    Post-e2e-pass, BM bm-pr opens PR phase-v1-deps-r1 → governance-v0.
```

If `Cargo.lock` diff exceeds ±200 lines, raise `kind: "log"` DQ noting the lock churn — surprising churn on a SemVer-compat bundle would be diagnostic (e.g. unrelated transitive resolution drift that the plan didn't anticipate).

### 4.6 Escape hatch: SemVer-compat regression on a consuming crate

Per plan §12 "NOT building in v1-deps-r1": if any of the 9 bumps surfaces an API regression at a consuming crate (e.g. `tokio 1.52` removes a deprecated API a Lemmy file uses), STOP and raise `kind: "blocker"` DQ with:
- Failing dep + version
- Cargo error log slice (last ~50 lines)
- Consuming crate + file:line cited by cargo

**Do NOT fold the fix into T3.** Per plan §12: that becomes a follow-up sub-phase. The advisor decides whether to (a) defer the offending bump (downgrade T3 to N-1 bumps), (b) split the regression fix into a fix-impl-task, or (c) revert and re-plan.

**Specific watchpoints (from plan §13 T3 GOTCHA):**

- **`tokio 1.52`** is a minor bump (1.50 → 1.52). Verify no `tokio::time::sleep` / `tokio::task::spawn_blocking` / `tokio::sync::*` API removal affects us. `rg "tokio::time\|tokio::task\|tokio::sync" crates/ > .claude/PRPs/debug/v1-deps-r1-task3-tokio-touchpoints.txt; wc -l ...` enumerates touchpoints.
- **`rustls 0.23.40`** is a security patch. Verify the custom `NoCertVerifier` impl at `crates/diesel_utils/src/connection.rs:224-272` (used for `sslmode=require`) still compiles against the 0.23.40 trait surface — if `ServerCertVerifier` or related trait surface shifted, the impl may need adjustment (out-of-scope: file `kind: "blocker"`).
- **`html2text 0.17.1`** pulls `html5ever 0.39` transitively. Per plan §2 cite, no user-visible API change. The 2 callers (`crates/email/src/send.rs` + `crates/apub/objects/src/objects/post.rs`) should compile unchanged — verify with the cargo check exit code, not by reading the files.
- **`dashmap 6.1 → 6.2`** — verify `crates/utils/src/...` consumers compile. The bump is minor; no known breaking change.
- **`lettre 0.11.19 → 0.11.22`** — preserve `default-features = false, features = [...]` block verbatim. Email-send code at `crates/email/src/send.rs` should compile unchanged.
- **`jsonwebtoken 10.3 → 10.4`** — preserve `features = ["rust_crypto"]` block verbatim. JWT-handling code at `crates/api/api_utils/src/...` should compile unchanged.

If a watchpoint fires, raise `kind: "blocker"` DQ per §4.2.

---

## 5. Validation gate (single-task)

T3's per-task validation gate IS the §15.1 + §15.2 DoD subset listed in §4.4 (check + clippy). E2E is **NOT in T3's worker-raised gate** — the advisor raises a separate `kind: "validate-pending-laptop-e2e"` entry post-T3-pass for the full workspace e2e gate at the phase tip (§4.4b).

Expected outcomes (T3 worker-raised gate):
- `cargo check --workspace --features full`: exit 0
- `cargo clippy --workspace --features full --no-deps -- -D warnings`: exit 0

Expected outcomes (phase-tip e2e gate, advisor-raised):
- `cargo test --workspace --test e2e --features full`: 109/109 pass

If the workspace check / clippy passes but phase-tip e2e regresses (e.g. a `tokio 1.52`-related async runtime behavioural shift surfaces only in the federation test path), surface to user for triage — `kind: "log"` DQ with the failing test list + the diagnostic that the in-task gate passed.

---

## 6. Post-task

- Commit + push to `phase-v1-deps-r1` worker branch (daemon finalize-merges to `phase-v1-deps-r1`).
- Raise the §4.4 `validate-pending-laptop` DQ. Commit + push immediately.
- DO NOT proceed to the phase-tip e2e — that's advisor-raised per §4.4b.
- DO NOT proceed to bm-pr — that's also advisor-orchestrated, gated on the phase-tip e2e PASS.
- Junior task complete when DQ is raised + pushed; advisor takes over for validation cycle + e2e gate + bm-pr.
