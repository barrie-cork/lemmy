# brehon-conformance-audit — impl Task 3 brief (Cohort 2)

## 1. Role + dispatch line

`[role:impl-task] brehon-conformance-audit Task 3 — find-sibling.sh — see .claude/PRPs/briefs/brehon-conformance-audit-impl-3.md`

## 2. Scope

Implement plan §13 **Task 3** (lines 939-991) of `.claude/PRPs/plans/brehon-conformance-audit.plan.md` (revised at `7bb6c51bb`; phase tip `e3b62793f`). One commit, one file created:

```yaml
creates:
  - .claude/skills/brehon-conformance-audit/scripts/find-sibling.sh
modifies: []
requires:
  - task: 1
    reason: "find-sibling.sh is REFERENCED by SKILL.md invocation discipline section."
```

bash + python in-file/module/crate sibling locator per §10.4 template. Three strategies: in-file (Grep+Read), in-module (Glob+Grep), in-crate (rust-analyzer LSP if available; Grep+Read fallback). Output: stdout `(sibling_file, sibling_line, sibling_signature)` or `NO_SIBLING_FOUND`.

**Do NOT** author:
- SKILL.md skeleton (shipped Task 1).
- Axis sub-files (Task 2 cohort 2 peer).
- Metrics schema or METRICS.md (Tasks 4, 5).
- `compute-metrics.sh` (Task 6, Cohort 2.5).
- `clippy.toml` (Task 8).
- Per-module deny attributes (Task 9).
- `.mcp.json.example` (Task 10).
- Anything under `crates/`, `migrations/`, `tests/`, `docs/brehon-law-inspired-network/`.

**Branch context:**
- `base_branch` = `phase-brehon-conformance-audit` (at `e3b62793f`).
- Worker forks to its own `junior/...` worktree branch.
- ONE commit on worker branch; daemon finalize-merges into phase branch.

## 3. Required reading

### 3.0 Plan + Task 1 spine (P0)

1. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §10.4 (find-sibling.sh template — verbatim), §13 Task 3 (lines 939-991).
2. `.claude/skills/brehon-conformance-audit/SKILL.md` (Task 1 output) — confirm forward-reference path matches.

### 3.1 MIRROR — existing bash scripts in scripts/brehon/

3. `scripts/brehon/cargo-check.sh` — `set -euo pipefail` discipline; argument validation; pre-flight checks. Read all 50-100 lines.
4. `scripts/brehon/git-show-json.sh` — argument parsing pattern; error exit codes; tmp file discipline.

### 3.2 Lessons + patterns

5. `pattern_verify_before_trusting_shell_output.md` — exit codes lie; validate explicitly.
6. `feedback_principles_not_rules.md` — script docstring style.

### 3.3 Rules (auto-loaded)

7. `.claude/rules/decision-queue.md` — mid-task visibility; attribution integrity.
8. `.claude/rules/phase-branch.md` — worker branch push discipline.

## 4. Constraints

1. **Mid-task push discipline** — any DQ entry pushed immediately to worker branch.
2. **Attribution integrity** — `from: "impl"`; never `answered_by: "advisor"`/`"user"`.
3. **`set -euo pipefail`** at top of script. Per `pattern_verify_before_trusting_shell_output.md`.
4. **`NO_SIBLING_FOUND` exits 0, not 1** — empty result is VALID (plan §13 Task 3 GOTCHA + Watchpoint #1).
5. **Bad-args path exits 2** with `ERROR: ...` to stderr.
6. **LSP path is a TODO** for v1 — rust-analyzer-mcp invocation from bash is non-trivial (JSON-RPC over stdio). Document Grep+Read fallback as primary path; the v2 LSP plumbing is a leading-comment note. (Plan §13 Task 3 GOTCHA explicit.)
7. **No cargo invocation** in this script (Watchpoint #3).
8. **Output format EXACT** — `printf '%s:%d:%s\n' "$file" "$line" "$sig"` or `printf 'NO_SIBLING_FOUND\n'`. Compute-metrics.sh (Task 6) consumes this format.
9. **Commit subject template** — `feat(skill): add find-sibling.sh script for brehon-conformance-audit (task 3)`.
10. **Single commit** per plan §13 norm.
11. **No `--no-verify`** — never skip hooks.
12. **Argument validation enforced** — both `target_file` (must exist) and `target_fn` required; missing or empty → exit 2.

## 5. Validation gate (DoD per plan §15)

### 5.1 Pre-push smoke test (worker runs locally before push)

```bash
# Positive: known sibling pair (post-Task-2-merge inbox.rs has axis examples)
bash .claude/skills/brehon-conformance-audit/scripts/find-sibling.sh \
  crates/apub/activities/src/governance/inbox.rs receive_remote_moderation_label
echo "exit: $?"
# EXPECT: stdout one line; NOT "NO_SIBLING_FOUND"; exit 0

# Negative: nonexistent function
bash .claude/skills/brehon-conformance-audit/scripts/find-sibling.sh \
  crates/apub/activities/src/governance/inbox.rs this_function_does_not_exist
echo "exit: $?"
# EXPECT: stdout "NO_SIBLING_FOUND"; exit 0

# Bad args
bash .claude/skills/brehon-conformance-audit/scripts/find-sibling.sh /nonexistent.rs foo 2>&1
echo "exit: $?"
# EXPECT: ERROR on stderr; exit 2
```

### 5.2 Cargo-check (workspace clean)

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-task3-check.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/brehon-conformance-audit-task3-check.log
# EXPECT: exit 0 (no Rust changes; sanity)
```

### 5.3 §15 validate-pending-laptop DQ (raise after worker push)

```json
{
  "id": "<next>",
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO>",
  "question": "validate phase-brehon-conformance-audit task 3 worker branch",
  "branch": "<worker-branch>",
  "phase_task": 3,
  "commands": [
    "bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-task3-check.log 2>&1; echo exit: $?"
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

## 6. KNOWN harness limitations

1. **CC v2.1.119 sensitive-file gate** — `.claude/skills/**` is on allowlist; if Write blocked, fall back to `/tmp + mv` per plan §19.5 LESSON.
2. **Daemon finalize-merge no-push** — advisor handles SSH-push post-finalize.

## 7. Next steps after this task

- Daemon finalize-merges onto phase branch.
- Advisor polling + validate-pending-laptop mutation as for Task 2.
- After ALL Cohort 2 tasks pass, dispatch Cohort 2.5 (Task 6).

## 8. Commit subject template (verbatim)

```
feat(skill): add find-sibling.sh script for brehon-conformance-audit (task 3)
```

Body cites plan path + SHA + Grep+Read fallback being v1 primary path.

## 9. DoD for this brief (advisor-side gate — completed at brief commit)

- [x] Brief committed on `phase-brehon-conformance-audit` BEFORE Junior task dispatch.
- [x] Cites plan §13 Task 3 line range (939-991).
- [x] Lists MIRROR exemplars (`scripts/brehon/cargo-check.sh` + `git-show-json.sh`).
- [x] §4 enumerates 12 constraints.
- [x] §5 names validate-pending-laptop DQ shape verbatim.
