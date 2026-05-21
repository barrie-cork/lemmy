# brehon-conformance-audit — impl Task 8a brief (Cohort 2.6, NEW per DQ #311 revision)

## 1. Role + dispatch line

`[role:impl-task] brehon-conformance-audit Task 8a — revert prior broken clippy.toml — see .claude/PRPs/briefs/brehon-conformance-audit-impl-8a.md`

## 2. Scope

Implement plan §13 **Task 8a** (lines 1281-1323) of `.claude/PRPs/plans/brehon-conformance-audit.plan.md` (revised at `a55f2b01f` per DQ #311; phase tip `a55f2b01f`). One commit, one file removed.

```yaml
creates: []
modifies:
  - clippy.toml          # removal (git rm); FILES YAML schema has no formal `deletes:` key
requires:
  - task: 0
    reason: "Pre-flight only; Task 8a is independent of Tasks 1-7 + 10 (file-set disjoint) but precedes Task 8 in cohort order."
```

Revert the prior **broken** `clippy.toml` commit (`c3aaba47f`) from the phase-branch tip. Single-file `git rm` commit. Worker does NOT re-create the file; Task 8 (next, single-commit corrected mechanism) re-lands `clippy.toml` PLUS adds the `Cargo.toml [workspace.lints.clippy]` workspace-allow override.

**Do NOT** author:
- Any `Cargo.toml` edit (Task 8's job).
- Any per-module `#![deny()]` attribute (Task 9's job).
- Any Rust source file or test.
- A new `clippy.toml` with different content — the file is removed entirely; Task 8 re-creates from scratch.

**Branch context:**
- `base_branch` = `phase-brehon-conformance-audit` (at `a55f2b01f`).
- Worker forks to its own `junior/...` worktree branch.
- ONE commit on worker branch; daemon finalize-merges.

## 3. Required reading

### 3.0 Plan + revision source (P0)

1. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §13 Task 8a (lines 1281-1323) — full task text.
2. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §10.8 (lines 485-543) — REVISED 2026-05-21; mechanism rationale.
3. `.claude/PRPs/briefs/brehon-conformance-audit-planning-2-revise.md` — DQ #311 plan-revision brief that produced the mechanism correction. Read §2.1 (defect statement), §2.2 (corrected mechanism), §2.3 (Tasks 8a/8/9 spec).

### 3.1 Prior cycle context (informational; explains WHY this revert is needed)

4. `git show c3aaba47f -- clippy.toml` — the prior broken Task 8 dispatch creating `clippy.toml` WITHOUT the matching `Cargo.toml [workspace.lints.clippy]` workspace-allow override.
5. `git show b00be611a` (fix-impl-1) + `git show 34f5cc567` (fix-impl-3) — net-positive cleanup commits that STAY (per §10.8 GOTCHA "fix-impl-1 + fix-impl-3 commits stay as cleanup payoff").

### 3.2 Rules (auto-loaded)

6. `.claude/rules/decision-queue.md` — `kind: "validate-pending-laptop"` shape for §5.3 DQ raise.
7. `.claude/rules/phase-branch.md` — worker branch push discipline.

## 4. Constraints

1. **Mid-task push discipline** (per `.claude/rules/decision-queue.md` "Mid-task visibility") — any DQ pushed to worker branch immediately after raise.
2. **Attribution integrity** — `from: "impl"`; never write `answered_by: "advisor"`/`"user"`.
3. **Single-file diff** — `git status --short` after the change must show ONLY `D clippy.toml`. Any other file in the diff is out of scope.
4. **No re-creation** — the worker MUST NOT recreate `clippy.toml` in a different form; Task 8 re-lands it.
5. **No `Cargo.toml` edit** — that is Task 8's job.
6. **No `--no-verify`** — never skip hooks (per `.claude/rules/no-destructive-defaults.md`).
7. **Workspace check stays green** — `cargo check --workspace --features full` STILL exits 0 after removal (toml deletion cannot break rustc; baseline at `4480a1bdb` had no `clippy.toml`).
8. **Workspace clippy may FAIL** post-revert without the workspace-allow override; that is EXPECTED per §10.8 GOTCHA. The pre-push gate (Constraint #11) is `cargo check`, NOT `cargo clippy`.
9. **Commit subject template (per plan §13 Task 8a)** — `chore(brehon-conformance-audit): revert prior broken Task 8 clippy.toml — DQ #311 mechanism revision`.
10. **Commit body** — cite DQ #311 + DQ #310 (user catch-fire) + §10.8 corrected mechanism + rustc lint-precedence rule 4. Reference prior broken Task 8 SHA `c3aaba47f` + fix-impl-1 `b00be611a` + fix-impl-3 `34f5cc567`.
11. **Pre-push cargo-check** (per `feedback_fix_impl_pre_push_cargo_check.md`) — run `bash scripts/brehon/cargo-check.sh --workspace --features full` LOCALLY before pushing the worker branch. Non-zero exit → file `kind: "blocker"` DQ; do not push the worker branch.

## 5. Validation gate (DoD per plan §15)

### 5.1 Pre-push workspace-check (worker runs locally before push)

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-task8a-prepush-check.log 2>&1
echo "exit: $?"
tail -10 .claude/PRPs/debug/brehon-conformance-audit-task8a-prepush-check.log
# EXPECT: exit 0
```

### 5.2 File-removal structural check

```bash
test ! -f clippy.toml
echo "exit: $?"
# EXPECT: exit 0

git status --short
# EXPECT: empty (clippy.toml removal already committed; no other changes)

# Confirm prior broken Task 8 SHA is the parent
git log -1 --format=%P HEAD
# EXPECT: equals a55f2b01f (current phase-branch tip)
```

### 5.3 §15 validate-pending-laptop DQ (raise after worker push)

After worker pushes the worker branch, raise a `kind: "validate-pending-laptop"` DQ entry naming the §15.1 cargo-check command verbatim. Advisor laptop runs the command and mutates the entry.

DQ entry shape (see `.claude/rules/decision-queue.md` "kind: validate-pending-laptop"):

```json
{
  "id": <next-id>,
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<UTC ISO 8601>",
  "branch": "<worker-branch-name>",
  "phase_task": "8a",
  "commands": [
    "bash scripts/brehon/cargo-check.sh --workspace --features full"
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
git commit -m "chore(decision-queue): impl raised DQ #<id> — task 8a validate-pending-laptop"
git push origin <worker-branch>
```

## 6. Self-resolve discipline

Per `.claude/rules/decision-queue.md` "Recipes" §"Recipe 2: self-resolve with kind: log" — if the worker finds a durable observation worth recording (e.g. "git rm reported file already gone; recovered by re-fetching origin"), write a `kind: "log"` entry directly to `resolved[]` with `answered_by: "impl-self-resolved"`. Do NOT raise as `kind: "blocker"` for things the worker can answer with evidence in-scope.

## 7. Failure-class mapping

| Symptom | Action |
|---|---|
| `git rm` reports "fatal: pathspec 'clippy.toml' did not match any files" | Tree already lacks the file → `kind: "log"` self-resolve; do NOT commit empty change. Surface to advisor as `kind: "blocker"` if uncertain. |
| `cargo check --workspace --features full` exits non-zero AFTER removal | `kind: "blocker"` DQ — removal cannot break rustc; non-zero implies a pre-existing breakage on the phase tip. Worker does NOT push. |
| Any non-`clippy.toml` file in `git status --short` after `git rm clippy.toml` | `kind: "blocker"` DQ — accidental contamination; worker does NOT commit. |
| Daemon finalize-merge conflict on `.claude/decision-queue.json` (concurrent advisor write race) | Standard finalize stash-pop conflict pattern; daemon resolves; advisor reconciles per `feedback_junior_finalize_merge_race_lossless_reconcile.md`. |

## 8. Out of scope

- Re-creating `clippy.toml` (Task 8).
- Editing `Cargo.toml` `[workspace.lints.clippy]` (Task 8).
- Per-module `#![deny(clippy::disallowed_methods)]` attributes (Task 9).
- Editing fix-impl-1 (`b00be611a`) or fix-impl-3 (`34f5cc567`) commits — they stay as net-positive cleanup payoff.
- Any `crates/**`, `migrations/**`, `tests/**` edit.
- Any `.claude/rules/*`, `.claude/skills/*`, `.claude/lessons/*` edit.
