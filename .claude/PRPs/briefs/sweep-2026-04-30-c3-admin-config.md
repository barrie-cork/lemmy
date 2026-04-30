# [role:impl-task] sweep-2026-04-30 C3 — admin-config-write.sh previous_value option A (issue #84)

## 1. Dispatch line

`[role:impl-task] sweep-c3-admin-config — see .claude/PRPs/briefs/sweep-2026-04-30-c3-admin-config.md`

## 2. Scope

Update the v0 shell wrapper at `scripts/brehon/admin-config-write.sh` to emit `previous_value` and `previous_from` in its `admin_config_changed` payload, matching the Rust handler's payload shape (commit `d623bcff5`). PRD §8.4 amended condition 3 (commit `5c04e6ec2`) accepted **option A** (the wrapper pre-fetches the current row before INSERT) as the deprecation path.

**What "option A" means:**

1. Wrapper takes a config key + new value.
2. **Before** the INSERT, wrapper queries the current row for that key from `admin_config` (or whatever the source-of-truth table is — verify via the Rust handler's source).
3. Constructs the `admin_config_changed` payload with both `previous_value` (the old value) and `previous_from` (the prior `from` field, e.g. who set it).
4. Pre-INSERT order is critical: read first, then INSERT, so the read sees the row state before this write.

**Shell-parity test must be updated** to assert byte-identical payload between wrapper and Rust handler. The test lives in `crates/server/tests/e2e.rs` per the issue body — but the test is a Rust integration test that calls the shell wrapper as a subprocess and compares emitted payloads. **Do NOT run the test from this worker** (no cargo on EliteDesk); just edit the assertion and let GH Actions or laptop validate.

**Out of scope:**
- Do NOT modify the Rust handler at `crates/api/api/src/governance/admin_config_write.rs` or wherever `d623bcff5` placed it. Handler is the reference shape.
- Do NOT change the `admin_config_changed` entry-kind constant.
- Do NOT modify other scripts under `scripts/brehon/*` outside of the wrapper.

**Boundaries:**
- Edit `scripts/brehon/admin-config-write.sh` (wrapper, ~shell).
- Edit the relevant `crates/server/tests/e2e.rs` test assertion ONLY for the shell-parity test for `admin_config_changed`. No other e2e test should be touched. Locate via `grep -n "admin_config_changed\|admin-config-write.sh" crates/server/tests/e2e.rs`.
- Single commit subject: `chore(admin-config): wrapper emits previous_value + previous_from pre-INSERT (closes #84)`.

## 3. Required reading

- **`scripts/brehon/admin-config-write.sh`** (the wrapper to update)
- **`crates/api/api/src/governance/`** — find the handler emitted by `d623bcff5`. Run `git show d623bcff5 --stat` to identify the file. Read the handler's payload-construction block to mirror its shape exactly.
- **`crates/server/tests/e2e.rs`** — the shell-parity test for `admin_config_changed`. Open the file at the matching test name only. **Do NOT load the whole 8969-line file via Edit.** Use `Read` with `offset` + `limit` for the relevant 30-50 lines.
- **`.claude/lessons/feedback_junior_worker_e2e_edit_hang.md`** — critical lesson: large e2e.rs Edits hang. Use surgical Edit with strong unique markers from a localized Read.
- **GitHub issue #84 body**: `gh issue view 84 --repo barrie-cork/lemmy --json body --jq .body`
- **PRD §8.4** — `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` or `docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md` §8.4 (search for "admin_config_changed").
- **`.claude/rules/governance-log-entry-kind-registry.md`** for `ENTRY_KIND_ADMIN_CONFIG_CHANGED`.

## 4. Constraints

**HARD FORBIDS:**
- `cargo *` on the worker. No `cargo check`, no `cargo test`, no clippy. The shell-parity test will be validated off-box via GH Actions or laptop (your job is to write the assertion, NOT to run it).
- Loading `crates/server/tests/e2e.rs` whole via Read or Edit. Read the file with `offset` + `limit` only at the matching test region. Edit with a strong unique anchor that won't collide elsewhere.
- Modifying any Rust handler file. Wrapper-only diff (shell + a one-line test assertion change).

**Required behaviour:**
- Single commit subject: `chore(admin-config): wrapper emits previous_value + previous_from pre-INSERT (closes #84)`.
- Trailer: `Closes: barrie-cork/lemmy#84`.
- Push branch and exit.
- After push, raise a `kind: "validate-pending"` DQ entry with the GH Actions run id from `gh run list --repo barrie-cork/lemmy --branch <your-worker-branch> --workflow cargo-validate-workspace --limit 1 --json databaseId,headBranch`. The shell-parity Rust test must compile clean; the advisor's polling loop dispatches a ci-watcher to confirm.
- DO NOT open PR — Junior daemon finalize-merges after validate-pending resolves green.

**File-locality:** `scripts/brehon/admin-config-write.sh` + a single function-region edit in `crates/server/tests/e2e.rs`. No overlap with C1, C2, C4.

**Edge-case to flag in DQ:** if the handler's payload shape includes a `previous_value: null` for new keys (no prior row), the wrapper must emit the same. Document handling explicitly in the wrapper. If unsure, raise `kind: "clarify"` DQ entry.

**Mid-task DQ push:** any payload-shape ambiguity → `kind: "clarify"`, `from: "impl"` in `.claude/decision-queue.json`, push, continue.
