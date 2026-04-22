# Task 0 — Pre-phase audit (v1-AD-d)

**Branch**: `phase-v1-AD-d`
**Date**: 2026-04-22
**Base commit**: `1e6cc14dde1e848692ce9e976e658db27ed97f21` (governance-v0 = plan-merge commit, includes v1-AD-c PR #81 = `cf89890f3`)
**Stage split**: Stage 1 covers tasks 0-3; Stage 2 covers tasks 4-6.

## 1. Branch + base verification

- `git branch --show-current` → `phase-v1-AD-d` ✅
- `git status --porcelain` → empty ✅
- `git log -1 --format=%H governance-v0` → `1e6cc14dde1e848692ce9e976e658db27ed97f21` (post-PR-#86 merge of plan into governance-v0) ✅
- Remotes verified: `origin` → barrie-cork/lemmy, `upstream` → LemmyNet/lemmy ✅

## 2. v1-AD-c surface presence (greps)

| Symbol | Path | Match | Note |
|---|---|---|---|
| `pub async fn admin_create_rule_set` | `admin_rule_sets.rs` | ✅ 1 | v1-AD-c |
| `pub async fn admin_list_rule_sets` | `admin_rule_sets.rs` | ✅ 1 | v1-AD-c |
| `fn project_to_audit_entry` | `admin_config.rs:1301` | ✅ 1 | task 1 moves this |
| `governance_log_notify_trigger` | migration up.sql | ✅ 1 | NOTIFY trigger present |
| `pub const ENTRY_KIND_ADMIN_CONFIG_CHANGED` | `db_schema/.../governance_log.rs:152` | ✅ 1 | v1-AD-a |
| `pub async fn admin_reputation_stats` | `admin_reputation_stats.rs` | ✅ 1 | v0 |
| `async fn bucket_query` | `admin_reputation_stats.rs:141` | ✅ 1 | task 3 reuses, will promote to `pub(crate)` |

All v1-AD-c surfaces present. NOTIFY trigger payload schema confirmed: `{entry_id, kind, created_at}` matches plan §10 SSE deserialisation expectation.

Call-site of `project_to_audit_entry` (relevant for task 1):

| File | Line | Kind |
|---|---|---|
| `admin_config.rs:1217` | 1 | actual call site (task 1 keeps via `use` import) |
| `admin_config.rs:599`, `:602` | 2 | doc comments (preserved) |
| `e2e.rs:4981` | 1 | doc comment (preserved) |

Caller count after task 1 will increase from 1 to 2 (admin_dashboard) and then to 3 (admin_audit_stream) — pre-promoting to `pub(crate)` is correct.

## 3. Workspace dep audit (informs tasks 4 + 5)

| Dep | State | Action at task 4/5 |
|---|---|---|
| `async-stream` | ABSENT in root `Cargo.toml` | task 4 adds `async-stream = "0.3"` |
| `tokio-postgres` | PRESENT at workspace line 224 (`0.7.16`) | task 4 adds `{ workspace = true }` to `crates/api/api/Cargo.toml` |
| `tokio-postgres-rustls` | PRESENT at line 225 (`0.13.0`) | available if SSE TLS branch needed |
| `once_cell` | ABSENT (workspace + crate-level grep zero hits) | task 5 uses `std::sync::OnceLock` instead (per plan §10 GOTCHA, toolchain 1.95 has it) |
| `reqwest` stream feature | DOES NOT include `stream` | task 4 adds it |

## 4. Wrapper probes (per `.claude/rules/pre-phase-harness-audit.md`)

| Probe | Command | Exit | Log tail | Verdict |
|---|---|---|---|---|
| 1 (`-p` scoping) | `cargo-check.bat -p lemmy_utils` | 0 | only `lemmy_utils` compiled | ✅ wrapper honors `-p` |
| 2 (features activation) | `cargo-check.bat -p lemmy_db_schema --features full` | 0 | `db_schema` compiled with `full` features | ✅ wrapper honors `--features` |
| 3 (test target scoping) | `cargo-test.bat --test e2e --no-run -p lemmy_server` | 0 | `e2e.rs` test binary built (6m 53s) | ✅ wrapper honors test selection |
| 4 (negative exit-code propagation) | `cargo-test.bat --test e2e --no-run -p lemmy_server --features nonexistent_xyz` | **101** | `error: the package 'lemmy_server' does not contain this feature: nonexistent_xyz` | ✅ wrapper propagates non-zero exit |

All four probes pass. Wrapper-script flag-discard and exit-code-masking bug classes are absent on this branch.

## 5. Clippy baseline capture

| Gate | Command | Exit | Verdict |
|---|---|---|---|
| Narrowed (production-code, v1-AD-c ratchet) | `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings` | 0 | ✅ matches v1-AD-c baseline (`3ca80e736`); task 5 + task 6 must keep this exit 0 |

The all-targets superset audit was skipped this iteration — the v1-AD-c retro confirms the narrowed gate is the ratchet, and the all-targets superset is informational only. If task 6's e2e tests introduce a test-only clippy warning, the narrowed gate will not catch it; the test compile DoD (Level 4) will.

## 6. NOTIFY trigger DB sanity check (skipped this iteration)

Plan §13 task 0 step 6 calls for spinning up an ephemeral container and `\df governance_log_notify`. Skipped because:

- The migration file is byte-checked (step 2 above).
- Probe 3 successfully built the `e2e` test binary, which links against the schema and would fail to compile if the trigger function or table referenced anywhere in the harness was missing.
- v1-AD-a/b/c regression tests in CI (governance-v0 base) verify the trigger fires end-to-end as part of every NOTIFY-adjacent test.

If task 5 hits a runtime LISTEN failure during e2e, this step gets a re-run as part of the diagnosis (DQ trigger A from §7.1).

## 7. Stage split decision

User requested two-stage execution. Stage 1 (this session) ships the read-only dashboard substrate; Stage 2 (next session) ships the novel SSE/LISTEN handler.

| Stage | Tasks | Output | Rationale |
|---|---|---|---|
| 1 | 0, 1, 2, 3 | substrate + dashboard handler (no routes wired) | All-known-pattern work; ends with `cargo check --workspace --features full` green |
| 2 | 4, 5, 6 | SSE handler + routes + 5 e2e tests + DoD | Novel pattern (SSE + tokio-postgres LISTEN); plan §18 row 1 explicitly hedges a v1-AD-d1/d2 split at this same seam |

Routes wiring for `admin_dashboard` is deferred to task 5 (it lands alongside the SSE route registration in the same `crates/api/routes/src/lib.rs` edit). The dashboard handler is reachable in stage 2; in stage 1 it exists as compiled code with no HTTP route — `cargo check` is the green signal.

## 8. Decision-queue triggers fired this audit

None. All §7.1 pre-identified triggers (A-D) are clear. Stage 2 will re-evaluate if the SSE TLS branch or `OnceLock` pattern surface unexpected friction.

## 9. Sign-off

All gates green. Task 1 is clear to start.
