# brehon-conformance-audit — impl Task 8 brief (Cohort 2)

## 1. Role + dispatch line

`[role:impl-task] brehon-conformance-audit Task 8 — clippy.toml + probe — see .claude/PRPs/briefs/brehon-conformance-audit-impl-8.md`

## 2. Scope

Implement plan §13 **Task 8** (lines 1238-1297) of `.claude/PRPs/plans/brehon-conformance-audit.plan.md` (revised at `7bb6c51bb`; phase tip `e3b62793f`). One commit, one file created:

```yaml
creates:
  - clippy.toml
modifies: []
requires: []
```

Repo-root `clippy.toml` with seed `disallowed-methods` entries per §10.8 verbatim (`core::option::Option::unwrap_or_default` + `core::result::Result::unwrap_or_default` with reason strings). Then run `cargo clippy --workspace --no-deps --features full -- -D warnings` probe BEFORE commit. Non-zero → STOP and file `kind: "blocker"` DQ. Zero → commit clippy.toml.

**Do NOT** author:
- SKILL.md, axis sub-files, find-sibling.sh, audit-metrics.schema.json, METRICS.md (Tasks 1, 2, 3, 4, 5 peers/spine).
- compute-metrics.sh (Task 6, Cohort 2.5).
- **Per-module `#![deny(clippy::disallowed_methods)]` attributes** — those are Task 9 (Cohort 4, SERIAL).
- `.mcp.json.example` (Task 10).
- Any test or Rust source file.

**Branch context:**
- `base_branch` = `phase-brehon-conformance-audit` (at `e3b62793f`).
- Worker forks to its own `junior/...` worktree branch.
- ONE commit on worker branch; daemon finalize-merges.

## 3. Required reading

### 3.0 Plan + ADR sources (P0)

1. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §10.8 (clippy.toml content — verbatim), §13 Task 8 (lines 1238-1297).
2. `.claude/PRPs/briefs/brehon-conformance-audit-planning-1.md` §0.1 PRECON-4 — Track B deny is per-module, NOT workspace. clippy.toml entries are global but level escalation is per-module (Task 9).

### 3.1 MIRROR — Lemmy workspace clippy config

3. `crates/utils/.clippy.toml` (or wherever the workspace's clippy.toml lives) — read for syntax precedent; this Task 8 lands a NEW repo-root `clippy.toml` distinct from any crate-local one. (If a repo-root `clippy.toml` already exists, surface to advisor as blocker — plan §13 Task 8 assumes none exists.)
4. `Cargo.toml` `[workspace.lints]` section — already governs workspace clippy; the Task 8 clippy.toml ADDS disallowed-methods, does NOT replace existing lint config.

### 3.2 Lessons (mandatory per §2.4 file-class table)

5. `feedback_clippy_test_style.md` — workspace clippy discipline; `-D warnings` enforcement.
6. `feedback_clippy_rerun_after_fix.md` — probe-then-fix-then-rerun pattern (plan §13 Task 8 GOTCHA explicit).
7. `feedback_fix_impl_pre_push_cargo_check.md` — pre-push discipline; the probe IS the §15 sanity for Task 8.
8. `feedback_lemmy_error_no_std_error.md` — `disallowed-methods` reason strings cite this lesson (source-of-truth for the recipe).

### 3.3 Rules (auto-loaded)

9. `.claude/rules/decision-queue.md` — `kind: "blocker"` shape if probe surfaces violations.
10. `.claude/rules/phase-branch.md` — worker branch push.

## 4. Constraints

1. **Mid-task push discipline** — any DQ pushed immediately to worker branch.
2. **Attribution integrity** — `from: "impl"`; never `answered_by: "advisor"`/`"user"`.
3. **`disallowed-methods` content EXACTLY per §10.8** — two entries (`Option::unwrap_or_default` + `Result::unwrap_or_default`) with the verbatim reason strings citing `feedback_lemmy_error_no_std_error.md` and `project_phase6_convention_divergence_class.md` axis #4.
4. **Probe BEFORE commit** — `bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings` runs on the working tree with clippy.toml present.
5. **Non-zero probe → STOP** — file `kind: "blocker"` DQ enumerating violations; do NOT commit clippy.toml. Advisor inserts a remediation task BEFORE Task 9. Plan §13 Task 8 + Watchpoint #4 explicit.
6. **Zero probe → commit clippy.toml** — single commit.
7. **NO per-module `#![deny()]` attributes** in this task. Those are Task 9. The probe with zero-exit confirms baseline clippy clean BUT not that disallowed-methods entries fire (they don't until Task 9's modules opt in).
8. **NO cargo-check** in this task — Task 8's §15 sanity IS the clippy probe (plan §13 Task 8 GOTCHA + §15.2 alignment).
9. **Commit subject template** — `feat(clippy): add repo-root clippy.toml with disallowed-methods seed (task 8)`.
10. **Single commit** per plan §13 norm.
11. **No `--no-verify`** — never skip hooks.
12. **TOML syntax must parse** — `cargo` rejects invalid TOML; the probe exit code catches this.

## 5. Validation gate (DoD per plan §15)

### 5.1 Probe (worker runs locally before push — REPLACES standard §15.1 cargo-check)

```bash
bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings > .claude/PRPs/debug/brehon-conformance-audit-task8-probe.log 2>&1
echo "exit: $?"
tail -30 .claude/PRPs/debug/brehon-conformance-audit-task8-probe.log
# EXPECT: exit 0 (zero — clippy.toml syntax parses, no existing federation code violates baseline)
# Non-zero → STOP, file kind: "blocker" DQ, do NOT commit clippy.toml.
```

### 5.2 TOML syntax (basic structural check)

```bash
python3 -c "
content = open('clippy.toml').read()
assert 'disallowed-methods' in content
assert 'core::option::Option::unwrap_or_default' in content
assert 'core::result::Result::unwrap_or_default' in content
assert 'feedback_lemmy_error_no_std_error.md' in content
print('OK')
"
# EXPECT: OK
```

### 5.3 §15 validate-pending-laptop DQ (raise after worker push)

```json
{
  "id": "<next>",
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO>",
  "question": "validate phase-brehon-conformance-audit task 8 worker branch",
  "branch": "<worker-branch>",
  "phase_task": 8,
  "commands": [
    "bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings > .claude/PRPs/debug/brehon-conformance-audit-task8-probe.log 2>&1; echo exit: $?"
  ],
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

DQ entry committed + pushed BEFORE worker exits.

**On probe-fail path**: in addition to the validate-pending-laptop DQ above, the worker raises a `kind: "blocker"` DQ enumerating each violation (file:line + method-path). Advisor inserts a remediation task BEFORE Task 9 dispatch.

## 6. KNOWN harness limitations

1. **CC v2.1.119 sensitive-file gate** — `clippy.toml` at repo root is NOT under `.claude/`; the gate should NOT fire. If it does, `/tmp + mv` is the fallback (plan §19.5 LESSON).
2. **Daemon finalize-merge no-push** — advisor handles SSH-push.
3. **Clippy probe runs cargo** — falls under Watchpoint #3 (no cargo in skill body) but the probe is OUTSIDE the skill body; it's a one-time pre-commit gate.

## 7. Next steps after this task

- Daemon finalize-merges Task 8 commit onto phase branch.
- Advisor polling + validate-pending-laptop mutation.
- After ALL Cohort 2 tasks (2, 3, 4, 5, 8, 10) pass, dispatch Cohort 2.5 (Task 6).
- After Cohort 2 + 2.5 finalize-merge, Cohort 4 (Task 9) adds `#![deny()]` per-module attributes.

## 8. Commit subject template (verbatim)

```
feat(clippy): add repo-root clippy.toml with disallowed-methods seed (task 8)
```

Body cites plan path + SHA + probe exit code (must be 0).

## 9. DoD for this brief (advisor-side gate — completed at brief commit)

- [x] Brief committed on `phase-brehon-conformance-audit` BEFORE Junior task dispatch.
- [x] Cites plan §13 Task 8 line range (1238-1297).
- [x] §4 enumerates 12 constraints.
- [x] §5 names validate-pending-laptop DQ shape + probe-fail blocker.
