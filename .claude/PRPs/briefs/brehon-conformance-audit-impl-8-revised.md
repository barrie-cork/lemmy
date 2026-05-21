# brehon-conformance-audit — impl Task 8 brief (REVISED per DQ #311 — Cohort 2.7)

## 1. Role + dispatch line

`[role:impl-task] brehon-conformance-audit Task 8 REVISED — single-commit clippy.toml + Cargo.toml workspace-allow — see .claude/PRPs/briefs/brehon-conformance-audit-impl-8-revised.md`

## 2. Scope

Implement plan §13 **Task 8** (revised) of `.claude/PRPs/plans/brehon-conformance-audit.plan.md` (lines 1325-1419; phase tip `a629dca41`). **ONE commit, TWO files** — `clippy.toml` CREATED + `Cargo.toml` MODIFIED (one line added). These are the two coordinated edits the corrected mechanism requires (§10.8 + rustc lint-precedence rule 4).

```yaml
creates:
  - clippy.toml
modifies:
  - Cargo.toml
requires:
  - task: 8a
    reason: "Task 8a reverted the prior broken clippy.toml; the corrected single-commit dispatch must land against a tree without it."
```

**Do NOT** author:
- Per-module `#![deny(clippy::disallowed_methods)]` attributes — those are Task 9 (Cohort 4, SERIAL).
- Any other `Cargo.toml` edit (e.g. dependency change, toolchain change, workspace member change) — ONLY the single `disallowed_methods = "allow"` line goes in.
- SKILL.md, axis sub-files, find-sibling.sh, audit-metrics.schema.json, METRICS.md, compute-metrics.sh (Tasks 1, 2, 3, 4, 5, 6).
- `.mcp.json.example` (Task 10).
- Any test or Rust source file.

**Branch context:**
- `base_branch` = `phase-brehon-conformance-audit` (at `a629dca41`).
- Worker forks to its own `junior/...` worktree branch.
- ONE commit on worker branch; daemon finalize-merges.

## 3. Required reading

### 3.0 Plan + revision source (P0)

1. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §10.8 (lines 485-543) — CORRECTED mechanism per DQ #311; rustc lint-precedence rule 4 ("lower-syntax-tree attributes take precedence").
2. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §13 Task 8 (lines 1325-1419) — task spec with verbatim content blocks.
3. `.claude/PRPs/briefs/brehon-conformance-audit-planning-2-revise.md` — DQ #311 plan-revision brief. Read §2.1 (defect statement: missing `disallowed_methods = "allow"`), §2.2 (corrected mechanism), §2.3 (Task 8 revised spec).

### 3.1 Cargo.toml structural reference (canonical MIRROR)

4. `Cargo.toml` root, lines 84-122 — the `[workspace.lints.clippy]` block. Line 121 `unchecked_time_subtraction = "deny"` is the LAST entry before blank line 122 + `[workspace.dependencies]` heading at line 123. Insertion goes AFTER line 121, BEFORE the blank line.
5. `Cargo.toml` line 100 — `style = { level = "deny", priority = -1 }`. This is the **load-bearing reason** the workspace-allow line is needed: clippy's `style` group contains `disallowed_methods`, so a bare `clippy.toml` activates the lint at workspace `deny` without the explicit `disallowed_methods = "allow"` override.

### 3.2 Lessons (mandatory per §2.4 file-class table)

6. `feedback_clippy_test_style.md` — workspace clippy discipline + `-D warnings` enforcement.
7. `feedback_clippy_rerun_after_fix.md` — probe-then-fix-then-rerun pattern.
8. `feedback_fix_impl_pre_push_cargo_check.md` — pre-push cargo-check discipline (mandatory before worker pushes branch).
9. `feedback_fix_impl_enumerate_all_callsites.md` — if narrow-probe surfaces violations, enumerate ALL callsites before patching.
10. `feedback_lemmy_error_no_std_error.md` — `disallowed-methods` reason strings cite this lesson.

### 3.3 Rules (auto-loaded)

11. `.claude/rules/decision-queue.md` — `kind: "validate-pending-laptop"` shape for §5.3 DQ raise.
12. `.claude/rules/phase-branch.md` — worker branch push discipline.
13. `.claude/rules/no-destructive-defaults.md` — no `--no-verify`, no force-push.

## 4. Constraints

1. **Mid-task push discipline** — any DQ pushed to worker branch immediately after raise.
2. **Attribution integrity** — `from: "impl"`; never write `answered_by: "advisor"`/`"user"`/`"planner"`/`"ci-watcher"` from this worker.
3. **`clippy.toml` content EXACTLY per §10.8 + Task 8 IMPLEMENT block** — two entries with verbatim path strings + verbatim reason strings citing `feedback_lemmy_error_no_std_error.md` and `project_phase6_convention_divergence_class.md` axis #4. NO additional entries.
4. **`Cargo.toml` edit is EXACTLY ONE LINE INSERTION** — `disallowed_methods = "allow"` (plus the trailing comment, multi-line OK). Inserted AFTER line 121 `unchecked_time_subtraction = "deny"`, BEFORE the blank line 122 and the `[workspace.dependencies]` heading at line 123. NO other `Cargo.toml` change.
5. **Hyphen vs underscore awareness** — lint NAMES in `[workspace.lints.*]` use underscores (`disallowed_methods`); `clippy.toml` KEYS use hyphens (`disallowed-methods`). Both forms are intentional Cargo/Clippy convention. Worker MUST NOT "fix" the difference.
6. **Workspace-lint-carve-out citation in commit body** — the general Brehon "do not touch `Cargo.toml`" guideline is narrowly about dependency-shape changes, toolchain changes, and build-config restructuring. A single workspace-lint-level override line is the explicit carve-out per §10.8 PRECONDITION-MATCH + DQ #311. Commit body cites this carve-out + DQ #311 + rustc lint-precedence rule 4.
7. **Single commit** containing BOTH files. NEVER split across two commits.
8. **No `--no-verify`** — never skip hooks.
9. **Pre-push cargo-check** (per `feedback_fix_impl_pre_push_cargo_check.md`) — run `bash scripts/brehon/cargo-check.sh --workspace --features full` LOCALLY before pushing the worker branch. Non-zero exit → patch in same commit (if in-scope) OR file `kind: "blocker"` DQ. Never `#[allow]`-spam to bypass.
10. **Pre-push narrow clippy probe** — run `bash scripts/brehon/cargo-clippy.sh -p lemmy_apub_activities -p lemmy_api -p lemmy_db_schema --features full --no-deps -- -D warnings` LOCALLY before pushing the worker branch. Exit 0 expected (federation crates clean — fix-impl-3 closed Finding 6.1; Task 9 hasn't activated per-module deny yet).
11. **Pre-push workspace clippy probe** — run `bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings` LOCALLY before pushing. Exit 0 expected — workspace-allow override silences `disallowed_methods` workspace-wide; non-federation pre-existing `unwrap_or_default` calls stay quiet.
12. **Hard refusal if any probe fails** — if any of Constraint 9/10/11 surfaces a violation, enumerate ALL callsites per `feedback_fix_impl_enumerate_all_callsites.md`, file `kind: "blocker"` DQ to advisor with the enumeration list, do NOT push the worker branch.
13. **Commit subject template (per plan §13 Task 8 revised)** — `feat(brehon-conformance-audit): add clippy.toml + Cargo.toml workspace-allow for disallowed_methods (Task 8 — DQ #311 corrected mechanism)`.
14. **Commit body** — cite §10.8 + DQ #311 + rustc lint-precedence rule 4 ("lower-syntax-tree attribute wins"). Reference Task 9 as the per-module deny step that pairs with this workspace-allow.

## 5. IMPLEMENT — verbatim content

### 5.1 CREATE `clippy.toml` at repo root

```toml
# clippy.toml — repo root
# Brehon federation trust-boundary disallowed methods. Workspace-default lint level is ALLOW
# (set in root Cargo.toml [workspace.lints.clippy]); per-module #![deny(clippy::disallowed_methods)]
# in the three federation module roots re-enables enforcement only there (rustc lint-precedence rule 4).
# Source: feedback_lemmy_error_no_std_error.md + project_phase6_convention_divergence_class.md axis #4.

disallowed-methods = [
  { path = "core::option::Option::unwrap_or_default",
    reason = "Use .ok_or_else(|| LemmyErrorType::*) for required federation fields. See feedback_lemmy_error_no_std_error.md and project_phase6_convention_divergence_class.md axis #4." },
  { path = "core::result::Result::unwrap_or_default",
    reason = "Use ? or .map_err with explicit error type. See feedback_lemmy_error_no_std_error.md." }
]
```

### 5.2 EDIT `Cargo.toml` `[workspace.lints.clippy]` block

Insert AFTER line 121 (`unchecked_time_subtraction = "deny"`), BEFORE the blank line at line 122 and the `[workspace.dependencies]` heading at line 123, exactly:

```toml
disallowed_methods = "allow"   # Federation-only enforcement: workspace-default ALLOW;
                               # per-module #![deny(clippy::disallowed_methods)] in three
                               # federation mod.rs files (Task 9) re-enables enforcement
                               # only there (rustc lint-precedence rule 4). See clippy.toml + DQ #311.
```

Resulting block snippet (lines 119-122 + new lines):

```toml
if_then_some_else_none = "deny"
allow_attributes = "deny"
unchecked_time_subtraction = "deny"
disallowed_methods = "allow"   # Federation-only enforcement: workspace-default ALLOW;
                               # per-module #![deny(clippy::disallowed_methods)] in three
                               # federation mod.rs files (Task 9) re-enables enforcement
                               # only there (rustc lint-precedence rule 4). See clippy.toml + DQ #311.

[workspace.dependencies]
```

## 6. Validation gate (DoD per plan §15)

### 6.1 Pre-push workspace cargo-check (mandatory)

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-task8-prepush-check.log 2>&1
echo "exit: $?"
tail -10 .claude/PRPs/debug/brehon-conformance-audit-task8-prepush-check.log
# EXPECT: exit 0
```

### 6.2 Pre-push narrow clippy probe (federation crates only)

```bash
bash scripts/brehon/cargo-clippy.sh -p lemmy_apub_activities -p lemmy_api -p lemmy_db_schema --features full --no-deps -- -D warnings > .claude/PRPs/debug/brehon-conformance-audit-task8-narrow-probe.log 2>&1
echo "exit: $?"
tail -30 .claude/PRPs/debug/brehon-conformance-audit-task8-narrow-probe.log
# EXPECT: exit 0 — workspace-allow silences workspace-wide; without Task 9's per-module deny, federation modules ALSO pass.
```

### 6.3 Pre-push workspace clippy probe (sanity)

```bash
bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings > .claude/PRPs/debug/brehon-conformance-audit-task8-workspace-probe.log 2>&1
echo "exit: $?"
tail -30 .claude/PRPs/debug/brehon-conformance-audit-task8-workspace-probe.log
# EXPECT: exit 0 — workspace-allow override silences disallowed_methods across the workspace
```

### 6.4 TOML + Cargo.toml structural assertions

```bash
python3 -c "
content = open('clippy.toml').read()
assert 'disallowed-methods' in content
assert 'core::option::Option::unwrap_or_default' in content
assert 'core::result::Result::unwrap_or_default' in content
assert 'feedback_lemmy_error_no_std_error.md' in content
print('clippy.toml OK')
"

python3 -c "
import re
content = open('Cargo.toml').read()
m = re.search(r'\[workspace\.lints\.clippy\](.*?)(?=^\[|\Z)', content, re.DOTALL | re.MULTILINE)
assert m, 'workspace.lints.clippy block not found'
block = m.group(1)
assert 'disallowed_methods = \"allow\"' in block, 'disallowed_methods = allow not in [workspace.lints.clippy]'
print('Cargo.toml OK')
"
```

### 6.5 §15 validate-pending-laptop DQ (raise after worker push)

After all three pre-push probes pass and worker pushes the worker branch, raise a `kind: "validate-pending-laptop"` DQ entry naming the three §15 commands verbatim. Advisor laptop re-runs and mutates the entry.

DQ entry shape:

```json
{
  "id": <next-id>,
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<UTC ISO 8601>",
  "branch": "<worker-branch-name>",
  "phase_task": "8",
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

Commit + push to worker branch:

```
git add .claude/decision-queue.json
git commit -m "chore(decision-queue): impl raised DQ #<id> — task 8 validate-pending-laptop (3 commands)"
git push origin <worker-branch>
```

## 7. Self-resolve discipline

Per `.claude/rules/decision-queue.md` Recipe 2 — log a `kind: "log"` entry directly to `resolved[]` if the worker observes something durable (e.g. "Cargo.toml line-number for insertion drifted by N lines; canonical reference at §10.8"). Do NOT raise as `kind: "blocker"` for things the worker can answer with in-scope evidence.

## 8. Failure-class mapping

| Symptom | Action |
|---|---|
| Pre-push cargo-check exits non-zero | `kind: "blocker"` DQ — `Cargo.toml` syntax broke compilation; advisor inspects edit; worker does NOT push. |
| Pre-push narrow clippy probe exits non-zero | Federation-code violation surfaced. Enumerate ALL callsites in federation crates via `rg -- "(Option\|Result)::unwrap_or_default" crates/{apub,api,db_schema}`. If ≤3 sites in ≤2 federation files, patch in same commit. Otherwise `kind: "blocker"` DQ with enumeration. |
| Pre-push workspace clippy probe exits non-zero | Workspace-allow override misapplied (typo, wrong place in block, or hyphen-vs-underscore mistake). Inspect `Cargo.toml` edit; if `disallowed_methods = "allow"` line is correct, file `kind: "blocker"` — unexpected interaction (possibly `clippy.toml` hyphenated key mismatch). |
| Toml parse error on `clippy.toml` | Re-read §5.1 verbatim and compare byte-by-byte. |
| Daemon finalize-merge conflict on `.claude/decision-queue.json` | Standard pattern per `feedback_junior_finalize_merge_race_lossless_reconcile.md`. |
| `[workspace.lints.clippy]` block heading missing on `Cargo.toml` (drift since planner read) | `kind: "blocker"` DQ — surface to advisor with file `head -130 Cargo.toml`. Never improvise; the plan §10.8 spec assumed lines 84-122. |

## 9. Out of scope

- Per-module `#![deny(clippy::disallowed_methods)]` (Task 9 — Cohort 4).
- Editing fix-impl-1 (`b00be611a`) or fix-impl-3 (`34f5cc567`) commits.
- Any change to `[workspace.dependencies]`, `[workspace.package]`, `[workspace.lints.rust]`, or any other `Cargo.toml` block.
- `rust-toolchain.toml`, `Cargo.lock`, `clippy.toml` content changes beyond §5.1 verbatim.
- Any `crates/**`, `migrations/**`, `tests/**` edit.
- Any `.claude/rules/*`, `.claude/skills/*`, `.claude/lessons/*` edit.
