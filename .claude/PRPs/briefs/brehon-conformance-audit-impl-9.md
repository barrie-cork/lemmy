# brehon-conformance-audit — impl Task 9 brief (Cohort 2.8)

## 1. Role + dispatch line

`[role:impl-task] brehon-conformance-audit Task 9 — add #![deny(clippy::disallowed_methods)] to 3 federation mod.rs files — see .claude/PRPs/briefs/brehon-conformance-audit-impl-9.md`

## 2. Scope

Implement plan §13 **Task 9** of `.claude/PRPs/plans/brehon-conformance-audit.plan.md` (lines 1421-1483). ONE commit, THREE files modified — adds `#![deny(clippy::disallowed_methods)]` as a file-head **inner attribute** on three federation module roots, re-enabling `clippy::disallowed_methods` enforcement ONLY in those modules (per rustc lint-precedence rule 4: lower-syntax-tree attribute wins over workspace `[workspace.lints.clippy].disallowed_methods = "allow"`).

```yaml
creates: []
modifies:
  - crates/apub/activities/src/governance/mod.rs       # add #![deny(clippy::disallowed_methods)] at file head
  - crates/api/api/src/governance/mod.rs               # add #![deny(clippy::disallowed_methods)] at file head
  - crates/db_schema/src/source/governance/mod.rs      # add #![deny(clippy::disallowed_methods)] at file head
requires:
  - task: 8
    reason: "clippy.toml entries (Task 8 6720dc72a) + workspace-allow override (Task 8 6720dc72a) + TOML syntax fix (3082c35ff) must exist before the deny attributes deny against them. Reverse order produces a build-broken intermediate state (clippy errors on unknown disallowed_methods key)."
```

**Branch context:**
- `base_branch` = `phase-brehon-conformance-audit` (at `3082c35ff` — TOML fix already applied; clippy.toml + Cargo.toml workspace-allow already in place).
- Worker forks to its own `junior/...` worktree branch.
- ONE commit on worker branch; daemon finalize-merges.

### 2.1 Exact insertion specification per file

The three files have different head structures. The `#![deny(...)]` attribute is an **inner attribute** that MUST be a file-head attribute. Conventional placement is `#![deny(...)]` at the very first line (line 1), with one blank line separating it from `//!` doc comments or `pub mod`/`pub use` items below.

**File 1 of 3: `crates/apub/activities/src/governance/mod.rs`**

Current head (lines 1-10 read):
```
//! Governance Create-wrapper activities.
//!
//! This module hosts the `Activity` trait implementations for the three
//! Brehon governance Create wrappers:
//!
//! - `PublishSanctionNotice` — broadcast that a finalised case applied
//!   a sanction (Phase 6 task 73 type, task 74 outbound, task 75 inbound)
...
```

**Insertion**: Prepend at line 1 — file becomes:
```
#![deny(clippy::disallowed_methods)]

//! Governance Create-wrapper activities.
//!
//! This module hosts ...
```

**File 2 of 3: `crates/api/api/src/governance/mod.rs`**

Current head (lines 1-10 read):
```
//! Brehon governance HTTP handlers and cross-cutting helpers.
//!
//! Organisation matches `docs/brehon-law-inspired-network/03-architecture.md §7`:
...
```

**Insertion**: Prepend at line 1 — file becomes:
```
#![deny(clippy::disallowed_methods)]

//! Brehon governance HTTP handlers and cross-cutting helpers.
//!
//! Organisation matches ...
```

**File 3 of 3: `crates/db_schema/src/source/governance/mod.rs`**

Current head (lines 1-10 read):
```
pub mod actor_pseudonym;
pub mod appeal;
pub mod case_evidence;
...
```

**Insertion**: Prepend at line 1 — file becomes:
```
#![deny(clippy::disallowed_methods)]

pub mod actor_pseudonym;
pub mod appeal;
pub mod case_evidence;
...
```

In ALL THREE files: the deny attribute is on line 1, a blank line on line 2, original content begins on line 3.

**Do NOT** author:
- Any change OTHER than the file-head attribute insertion on these 3 files (no doc comment edits, no `pub mod` reorder, no whitespace cleanup).
- Any change to OTHER files under the three module roots (e.g. `crates/apub/activities/src/governance/sanctions.rs`, `crates/api/api/src/governance/handlers.rs`, `crates/db_schema/src/source/governance/governance_log.rs`).
- Any change to `clippy.toml` or `Cargo.toml` (Task 8's outputs stay as-is).
- Any new Rust source file or test.

## 3. Required reading

### 3.0 Plan + Task 8 context (P0)

1. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §13 Task 9 (lines 1421-1483) — full task text.
2. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §10.8 (lines 485-543) — REVISED 2026-05-21; mechanism rationale.
3. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §10.9 (read at impl time) — per-module deny attribute spec.
4. `clippy.toml` (at phase tip `3082c35ff`) — the 2 disallowed methods (`Option::unwrap_or_default`, `Result::unwrap_or_default`) that the deny attributes will enforce against.
5. `Cargo.toml` (at phase tip `3082c35ff`) — confirm `disallowed_methods = "allow"` is on the workspace `[workspace.lints.clippy]` block (line ~122).

### 3.1 Lessons (mandatory per file-class injection §2.4)

6. `feedback_lemmy_error_no_std_error.md` — error-shape canonical recipe; explains why `.unwrap_or_default()` is forbidden in federation code (Case A/B/C signature uniformity).
7. `feedback_fix_impl_pre_push_cargo_check.md` — pre-push cargo-check + clippy gate (Constraint #9 + #10).
8. `feedback_fix_impl_enumerate_all_callsites.md` — if a deny attribute surfaces an EXISTING violation in federation code, enumerate ALL callsites before patching.

### 3.2 Rules (auto-loaded)

9. `.claude/rules/decision-queue.md` — `kind: "validate-pending-laptop"` shape for §5.3 DQ raise.
10. `.claude/rules/phase-branch.md` — worker branch push discipline.

### 3.3 §G4 CANONICAL RECIPE (no allowlist row — this is a structural plan implementation, not §G4 fix)

This is a structural Task 9 implementation, NOT a fix-impl. No §G4 allowlist row applies. The §2.1 insertion specification above is the contract.

## 4. Constraints

1. **Mid-task push discipline** (per `.claude/rules/decision-queue.md` "Mid-task visibility") — any DQ pushed to worker branch immediately after raise.
2. **Attribution integrity** — `from: "impl"`; never write `answered_by: "advisor"`/`"user"`.
3. **Three-file diff** — `git status --short` after the change must show EXACTLY:
   - `M crates/apub/activities/src/governance/mod.rs`
   - `M crates/api/api/src/governance/mod.rs`
   - `M crates/db_schema/src/source/governance/mod.rs`
   Any other file in the diff is out of scope.
4. **NO file-content changes other than the 2-line prepend** — only `#![deny(clippy::disallowed_methods)]\n\n` at file head. Do NOT reorder existing content, do NOT delete content, do NOT add anything else.
5. **No `#[allow]`-spam** (per `feedback_fix_impl_pre_push_cargo_check.md`) — if a deny attribute surfaces a federation-code violation in governance modules, worker patches the violation in the SAME commit IF in-scope (federation-code in governance modules), OR files `kind: "blocker"` DQ if out-of-scope.
6. **No `--no-verify`** — never skip hooks (per `.claude/rules/no-destructive-defaults.md`).
7. **Pre-push workspace cargo-check** (per `feedback_fix_impl_pre_push_cargo_check.md`) — run `bash scripts/brehon/cargo-check.sh --workspace --features full` LOCALLY before pushing. Non-zero exit → file `kind: "blocker"` DQ; do not push.
8. **Pre-push narrow federation clippy** — run `bash scripts/brehon/cargo-clippy.sh -p lemmy_apub_activities -p lemmy_api -p lemmy_db_schema --features full --no-deps -- -D warnings` LOCALLY before pushing. Exit 0 expected — federation governance modules are already axis-4 clean per fed-in-b fix-impl-3 evidence (Finding 6.1 closed `8b04e69a6`). Non-zero exit → enumerate all callsites + decide patch-or-blocker per Constraint #5.
9. **Pre-push workspace clippy** — run `bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings` LOCALLY before pushing. Exit 0 expected — workspace allow override silences `disallowed_methods` everywhere except the 3 federation modules.
10. **Commit subject** — `feat(brehon-conformance-audit): add #![deny(clippy::disallowed_methods)] to 3 federation mod.rs files (Task 9)`.
11. **Commit body** — cite plan §13 Task 9, §10.9 spec, rustc lint-precedence rule 4, and reference Task 8 commits `6720dc72a` (clippy.toml + workspace-allow) + `3082c35ff` (TOML syntax fix).

## 5. Validation gate (DoD per plan §15)

### 5.1 Pre-push validation (worker runs locally before push)

```bash
# Workspace check stays green
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-task9-prepush-check.log 2>&1
echo "check exit: $?"
tail -5 .claude/PRPs/debug/brehon-conformance-audit-task9-prepush-check.log
# EXPECT: exit 0

# Narrow federation clippy (deny attributes now active on 3 federation crates)
bash scripts/brehon/cargo-clippy.sh -p lemmy_apub_activities -p lemmy_api -p lemmy_db_schema --features full --no-deps -- -D warnings > .claude/PRPs/debug/brehon-conformance-audit-task9-prepush-narrow-clippy.log 2>&1
echo "narrow clippy exit: $?"
tail -30 .claude/PRPs/debug/brehon-conformance-audit-task9-prepush-narrow-clippy.log
# EXPECT: exit 0 (federation governance code already axis-4 clean per fed-in-b fix-impl-3)

# Workspace clippy stays green (sanity — confirms workspace-allow override outside 3 federation modules)
bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings > .claude/PRPs/debug/brehon-conformance-audit-task9-prepush-workspace-clippy.log 2>&1
echo "workspace clippy exit: $?"
tail -10 .claude/PRPs/debug/brehon-conformance-audit-task9-prepush-workspace-clippy.log
# EXPECT: exit 0
```

### 5.2 Structural check (worker runs after change, before commit)

```bash
# Each mod.rs carries the deny attribute on line 1
for f in crates/apub/activities/src/governance/mod.rs crates/api/api/src/governance/mod.rs crates/db_schema/src/source/governance/mod.rs; do
  head -1 "$f"
done
# EXPECT: each line is "#![deny(clippy::disallowed_methods)]"

# Line 2 is blank, line 3 is original first content line
for f in crates/apub/activities/src/governance/mod.rs crates/api/api/src/governance/mod.rs crates/db_schema/src/source/governance/mod.rs; do
  sed -n '2p' "$f" | head -c 40
  echo "<EOL>"
done
# EXPECT: each line is empty (just <EOL>)

git status --short
# EXPECT: exactly 3 lines, all "M crates/<path>/governance/mod.rs"
```

### 5.3 §15 validate-pending-laptop DQ (raise after worker push)

After worker pushes the worker branch, raise a `kind: "validate-pending-laptop"` DQ entry naming THREE cargo commands (workspace check + narrow federation clippy + workspace clippy) verbatim.

DQ entry shape (see `.claude/rules/decision-queue.md` "kind: validate-pending-laptop"):

```json
{
  "id": <next-id>,
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<UTC ISO 8601>",
  "branch": "<worker-branch-name>",
  "phase_task": 9,
  "commands": [
    "bash scripts/brehon/cargo-check.sh --workspace --features full",
    "bash scripts/brehon/cargo-clippy.sh -p lemmy_apub_activities -p lemmy_api -p lemmy_db_schema --features full --no-deps -- -D warnings",
    "bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings"
  ],
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answered_by": null,
  "resolved_at": null
}
```

After raising, commit + push to worker branch:

```
git add .claude/decision-queue.json
git commit -m "chore(decision-queue): impl raised DQ #<id> — task 9 validate-pending-laptop"
git push origin <worker-branch>
```

## 6. Failure-class mapping

| Symptom | Action |
|---|---|
| `cargo check --workspace --features full` exits non-zero AFTER edit | `kind: "blocker"` DQ — inner-attribute insertion cannot break rustc unless it's malformed; verify file structure. Worker does NOT push. |
| Narrow federation clippy exit non-zero with `error: use of a disallowed method` in federation governance code (`crates/apub/activities/src/governance/`, `crates/api/api/src/governance/`, `crates/db_schema/src/source/governance/`) | Federation governance modules should already be axis-4 clean. Enumerate ALL callsites in the 3 module trees with `rg "unwrap_or_default" crates/apub/activities/src/governance/ crates/api/api/src/governance/ crates/db_schema/src/source/governance/`. Patch in SAME commit if in-scope (≤5 sites, federation governance code). >5 sites or out-of-scope → `kind: "blocker"` DQ. |
| Narrow federation clippy exit non-zero with `error: use of a disallowed method` in OTHER crate code (e.g. lemmy_utils, lemmy_db_schema OUTSIDE source/governance) | `kind: "blocker"` DQ — the workspace-allow override SHOULD silence these. If they surface, the deny attribute may be cascading via re-exports or module visibility. Attach log slice ≤200 lines. Worker does NOT push. |
| Workspace clippy exit non-zero with `error: use of a disallowed method` outside the 3 federation modules | `kind: "blocker"` DQ — workspace-allow override is broken or rustc lint-precedence rule 4 is being misapplied. This invalidates the whole §10.8 mechanism. Worker does NOT push. |
| Any non-3-mod.rs file in `git status --short` after `git add` | `kind: "blocker"` DQ — accidental contamination; worker does NOT commit. |
| Daemon finalize-merge conflict on `.claude/decision-queue.json` (concurrent advisor write race) | Standard finalize stash-pop conflict pattern; daemon resolves; advisor reconciles per `feedback_junior_finalize_merge_race_lossless_reconcile.md`. |

## 7. Out of scope

- Editing `clippy.toml` (Task 8's output; stays as-is).
- Editing `Cargo.toml` (Task 8's workspace-allow line; stays as-is).
- Editing any other `*.rs` file under the 3 module roots — only the 3 `mod.rs` files at the listed paths.
- Editing any test file under `tests/`.
- Editing any rule, lesson, or plan file.
- Adding any new lesson file (defer to Task 12 lesson-promotion phase or Task 13 retro).
- Cleaning up axis-4 violations elsewhere in the workspace — workspace-allow override silences those by design.
