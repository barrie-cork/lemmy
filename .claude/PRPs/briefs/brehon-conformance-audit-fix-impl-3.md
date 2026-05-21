# brehon-conformance-audit — fix-impl Task 3 brief (diesel_utils + narrow-probe gate)

## 1. Role + dispatch line

`[role:impl-task] brehon-conformance-audit fix-impl-3 — diesel_utils 4-site fix + narrow-probe gate — see .claude/PRPs/briefs/brehon-conformance-audit-fix-impl-3.md`

## 2. Scope

Resolve DQ #309 per user option-a (2026-05-21 via AskUserQuestion): **"Narrow probe"** — keep `clippy.toml` workspace-wide, narrow Task 8/9 probe target to federation crates only. Net-positive carry-over: commit fix-impl-2's working-tree 4-site lemmy_diesel_utils fix (worker-369's working tree was cancelled before commit). Plan §10.8 misanalysis (workspace.lints.clippy.style does NOT activate clippy::disallowed_methods; clippy.toml IS the activation mechanism, and `disallowed-methods` is workspace-global, not per-module) — booked as Task 13 retro lesson; not in this brief's scope.

### 2.1 §G4 CANONICAL RECIPE — N/A (catch-fire path; not allowlist)

DQ #309 was catch-fire (judgment-heavy gate 2; user-relayed). User option-a is the canonical answer.

### 2.2 Four diesel_utils callsites to remediate (enumerated; full set)

Per DQ #307 context — same enumeration verified by worker-369 in its cancelled run:

| File | Line | Pattern | Inner type | Replacement |
|---|---|---|---|---|
| `crates/diesel_utils/src/pagination.rs` | 111 | `page_back.unwrap_or_default()` | `Option<bool>` | `page_back.unwrap_or(false)` |
| `crates/diesel_utils/src/pagination.rs` | 274 | `limit.try_into().unwrap_or_default()` | `Result<usize, _>` | `limit.try_into().unwrap_or(0)` |
| `crates/diesel_utils/src/schema_setup/mod.rs` | 73 | `TimeDelta::from_std(start_time.elapsed()).map(\|d\| d.to_string()).unwrap_or_default()` | `Result<String, _>` | `TimeDelta::from_std(start_time.elapsed()).map(\|d\| d.to_string()).unwrap_or_else(\|_\| String::new())` |
| `crates/diesel_utils/src/schema_setup/mod.rs` | 117 | (same shape as :73) | `Result<String, _>` | `TimeDelta::from_std(start_time.elapsed()).map(\|d\| d.to_string()).unwrap_or_else(\|_\| String::new())` |

**File-edit cap:** 2 files. Mechanical.

### 2.3 Narrow-probe validation (defines new success gate for Task 8 / Task 9)

After the 4 edits, run the **narrow probe** (federation crates only):

```bash
bash scripts/brehon/cargo-clippy.sh \
  -p lemmy_apub \
  -p lemmy_apub_activities \
  -p lemmy_apub_objects \
  -p lemmy_apub_send \
  --features full --no-deps -- -D warnings
```

EXPECT exit 0. The 4 apub crates ARE the federation surface; clippy.toml's `disallowed-methods` blocks new `unwrap_or_default` in NEW federation code (the brehon-conformance-audit intent). Already-clean apub crates → narrow probe is exit 0 today.

### 2.4 What NOT to author

- Do NOT touch `clippy.toml` (stays workspace-wide; semantics unchanged).
- Do NOT add `[lints.clippy] disallowed_methods = "deny"` to any Cargo.toml (user explicitly rejected the per-crate-Cargo.toml alternative when picking option-a).
- Do NOT add `#[allow(clippy::disallowed_methods)]` to non-federation crates (workspace-wide enforcement is desired; we just narrow the *probe* target, not the lint scope).
- Do NOT touch `crates/utils/`, `crates/db_schema/`, `crates/db_views/`, `crates/api/`, `crates/email/`, `crates/routes/`, `crates/server/` (out of scope; the workspace-wide `cargo clippy` outside the narrow probe will continue to fail at db_schema/etc — that's expected and intentional under option-a).
- Do NOT add `#[allow(clippy::disallowed_methods)]` to any function (per `feedback_fix_impl_pre_push_cargo_check.md`).
- Do NOT touch plan files (planner re-runs adjust §10.8 / §13 Task 9 scope).

**Branch context:**
- `base_branch` = `phase-brehon-conformance-audit` (currently at `e668cbb17`).
- Worker forks to its own `junior/...` worktree branch.
- ONE commit on worker branch; daemon finalize-merges into phase branch.

## 3. Required reading

### 3.0 Source DQ + plan (P0)

1. DQ #307 from `.claude/decision-queue.json` on phase tip — 4-callsite enumeration source-of-truth.
2. DQ #309 from worker-369 (commit `771317481`; in DQ pending at fetch time on phase tip after this Junior task commits) — the workspace-cascade finding that drove option-a.
3. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §13 Task 8 + §10.8 — for context; plan revision deferred.

### 3.1 Lessons (mandatory)

4. `feedback_lemmy_error_no_std_error.md` — recipe family (axis-4 footgun).
5. `feedback_fix_impl_pre_push_cargo_check.md` — pre-push discipline.
6. `feedback_fix_impl_enumerate_all_callsites.md` — enumeration discipline (4 sites enumerated).
7. `feedback_clippy_test_style.md` — clippy enforcement; `-D warnings`.
8. `.claude/PRPs/briefs/brehon-conformance-audit-fix-impl-1.md` §2.2 — precedent pattern for `unwrap_or_default` replacements.

### 3.2 Pre-fix verification reading

9. `crates/diesel_utils/src/pagination.rs` lines 105-115 + 268-280 — confirm sites.
10. `crates/diesel_utils/src/schema_setup/mod.rs` lines 65-80 + 110-125 — confirm sites.

### 3.3 Rules (auto-loaded)

11. `.claude/rules/decision-queue.md` — validate-pending-laptop DQ shape.
12. `.claude/rules/phase-branch.md` — worker branch push.
13. `.claude/rules/cargo-output-capture.md` — capture output to file.

## 4. Constraints

1. **Mid-task push discipline** — any DQ pushed immediately.
2. **Attribution integrity** — `from: "impl"`. NEVER `answered_by: "advisor"` or `"user"`.
3. **Exactly 4 callsites edited** per §2.2. NOT 3, NOT 5. NO additional sweep.
4. **Replacements MUST be semantically equivalent** to original `unwrap_or_default()`:
   - `Option<bool>::unwrap_or_default()` ≡ `.unwrap_or(false)`.
   - `Result<usize, _>::unwrap_or_default()` ≡ `.unwrap_or(0)`.
   - `Result<String, _>::unwrap_or_default()` ≡ `.unwrap_or_else(|_| String::new())`.
5. **NO #[allow] spam** — per `feedback_fix_impl_pre_push_cargo_check.md`.
6. **Pre-push narrow-probe gate** (§5.2 below). The workspace-wide probe will FAIL (expected per option-a; 100+ sites in non-federation crates); the **narrow probe is the success gate**.
7. **Pre-push cargo-check (workspace)** — `cargo-check.sh --workspace --features full` MUST exit 0 (compile-only; no clippy enforcement at this stage). Per `feedback_fix_impl_pre_push_cargo_check.md`.
8. **NO test-behavior changes** — 4 callsites are pagination + migration runner; existing tests should pass.
9. **NO new dependencies**.
10. **Commit subject template** — `fix(lemmy_diesel_utils): replace 4 unwrap_or_default with explicit fallbacks (brehon-conformance-audit fix-impl-3)`.
11. **No `--no-verify`**.
12. **DQ raise pre-push**: validate-pending-laptop entry references the **narrow probe command** (not workspace clippy).

## 5. Validation gate (DoD)

### 5.1 Pre-push cargo-check (compile validation)

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-fix-impl-3-check.log 2>&1
status=$?
tail -20 .claude/PRPs/debug/brehon-conformance-audit-fix-impl-3-check.log
echo "exit: $status"
# EXPECT: exit 0 (semantically equivalent replacements should not break compile)
```

### 5.2 Pre-push narrow-probe cargo-clippy (THE SUCCESS GATE)

```bash
bash scripts/brehon/cargo-clippy.sh \
  -p lemmy_apub \
  -p lemmy_apub_activities \
  -p lemmy_apub_objects \
  -p lemmy_apub_send \
  --features full --no-deps -- -D warnings \
  > .claude/PRPs/debug/brehon-conformance-audit-fix-impl-3-narrow-clippy.log 2>&1
status=$?
tail -30 .claude/PRPs/debug/brehon-conformance-audit-fix-impl-3-narrow-clippy.log
echo "exit: $status"
# EXPECT: exit 0 — federation crates clean under disallowed-methods.
# Non-zero → DO NOT commit; file kind: "blocker" DQ enumerating the federation-crate violations.
```

### 5.3 Pre-push test compile (lemmy_diesel_utils only)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_diesel_utils --features full --no-run > .claude/PRPs/debug/brehon-conformance-audit-fix-impl-3-test-compile.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 5.4 §15 validate-pending-laptop DQ (raise after worker push)

```json
{
  "id": "<next>",
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO>",
  "question": "validate phase-brehon-conformance-audit fix-impl-3 worker branch (4 diesel_utils edits + narrow-probe gate)",
  "branch": "<worker-branch>",
  "phase_task": "fix-impl-3",
  "commands": [
    "bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-fix-impl-3-check.log 2>&1; echo exit: $?",
    "bash scripts/brehon/cargo-clippy.sh -p lemmy_apub -p lemmy_apub_activities -p lemmy_apub_objects -p lemmy_apub_send --features full --no-deps -- -D warnings > .claude/PRPs/debug/brehon-conformance-audit-fix-impl-3-narrow-clippy.log 2>&1; echo exit: $?"
  ],
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

### 5.5 No DQ #309 self-resolve in this brief

DQ #309 is judgment-heavy; advisor resolved with `answered_by: "advisor"` referencing user option-a in a SEPARATE advisor commit BEFORE this Junior task dispatched. This brief does not self-resolve #309.

## 6. KNOWN harness limitations

1. **CC v2.1.119 sensitive-file gate** — `crates/diesel_utils/**` not on sensitive list.
2. **Daemon finalize-merge no-push** — advisor handles SSH-push.
3. **Workspace clippy will fail** under `--workspace --no-deps --features full -- -D warnings` after this commit; this is intentional per option-a. The narrow probe IS the new gate.

## 7. Next steps after this task

- Daemon finalize-merges fix-impl-3 commit onto phase branch.
- Advisor runs bundled narrow-probe locally to confirm success gate; mutates validate-pending-laptop to pass.
- DQ #308 (Task 8 workspace-wide validate-pending-laptop fail) stays at result=fail in `resolved[]` historical record once promoted; not retried.
- Cohort 2.5 (Task 6 compute-metrics.sh) ready to dispatch (gated only on Cohort 2 finalize-merges, which IS done).
- Task 9 (Cohort 4) scope revision deferred to planner DQ — current Task 9 wording assumes workspace-wide enforcement; planner re-decides whether to per-module `#![deny()]` annotate federation modules (probably yes for explicit-intent documentation).
- Task 13 retro lessons:
  - Plan §10.8 misanalysis of clippy::disallowed_methods activation mechanism (group-deny does NOT activate configurable lints).
  - Workspace-wide clippy.toml + narrow probe = simplest path for federation-only enforcement.

## 8. Commit subject template (verbatim)

```
fix(lemmy_diesel_utils): replace 4 unwrap_or_default with explicit fallbacks (brehon-conformance-audit fix-impl-3)
```

Body cites:
- DQ #307 + DQ #309 + user option-a + plan path + SHA.
- Four callsites enumerated (pagination 111/274 + schema_setup/mod 73/117).
- Narrow-probe exit 0 evidence.
- Note: workspace clippy will fail by design under option-a; narrow probe is the gate.
- Co-Authored-By trailer.

## 9. DoD for this brief (advisor-side gate — completed at brief commit)

- [x] Brief committed on `phase-brehon-conformance-audit` BEFORE Junior task dispatch.
- [x] §2.2 enumerates all 4 callsites.
- [x] §2.3 defines narrow probe (4 apub crates).
- [x] §4 lists 12 constraints including narrow-probe gate.
- [x] §5 names validate-pending-laptop DQ shape referencing the narrow probe command.
- [x] §G4 CANONICAL RECIPE block deferred (DQ #309 is catch-fire path).
