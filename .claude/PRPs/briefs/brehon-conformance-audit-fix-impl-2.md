# brehon-conformance-audit — fix-impl Task 2 brief (Task 8 re-remediation)

## 1. Role + dispatch line

`[role:impl-task] brehon-conformance-audit fix-impl-2 — remediate 4 unwrap_or_default in lemmy_diesel_utils — see .claude/PRPs/briefs/brehon-conformance-audit-fix-impl-2.md`

## 2. Scope

Resolve DQ #307 per advisor option-a (2026-05-21 auto-mode; matches the precedent set by DQ #303 user-chosen option-a). Replace 4 `unwrap_or_default()` callsites in `crates/diesel_utils/` with semantically equivalent alternatives that do NOT trigger `disallowed-methods` under the clippy.toml seed authored by worker-368 (uncommitted on its worker branch). Then re-run the Task 8 probe to confirm zero violations and unblock Task 8 commit.

### 2.1 §G4 CANONICAL RECIPE — N/A (not an allowlist-matched failure)

DQ #307 is **NOT** a §G4 allowlist row (no error code; pre-existing-code violation under a new probe). Advisor option-a (auto-mode) is consistent with the DQ #303 precedent. Plan §10.8 + PRECON-4 misanalysis surfaced via DQ #307 — `workspace.lints.clippy.style={level=deny,priority=-1}` deny-escalates the `style` lint **group** but does NOT auto-enable `clippy::disallowed_methods` (a configurable lint that requires the `clippy.toml` `disallowed-methods = […]` array to function at all). Workspace lints config is necessary but not sufficient — `clippy.toml` IS the activation. fix-impl-2 remediates the 4 sites the activation surfaces.

### 2.2 Four callsites to remediate (enumerated; this IS the full set)

Per DQ #307 context (worker-368 cargo-clippy probe `cargo-clippy.sh --workspace --no-deps --features full -- -D warnings`):

| File | Line | Pattern | Inner type | Replacement |
|---|---|---|---|---|
| `crates/diesel_utils/src/pagination.rs` | 111 | `page_back.unwrap_or_default()` | `Option<bool>` | `page_back.unwrap_or(false)` |
| `crates/diesel_utils/src/pagination.rs` | 274 | `limit.try_into().unwrap_or_default()` | `Result<usize, _>` | `limit.try_into().unwrap_or(0)` |
| `crates/diesel_utils/src/schema_setup/mod.rs` | 73 | `TimeDelta::from_std(start_time.elapsed()).map(\|d\| d.to_string()).unwrap_or_default()` | `Result<String, _>` | `TimeDelta::from_std(start_time.elapsed()).map(\|d\| d.to_string()).unwrap_or_else(\|_\| String::new())` |
| `crates/diesel_utils/src/schema_setup/mod.rs` | 117 | (same shape as :73) | `Result<String, _>` | `TimeDelta::from_std(start_time.elapsed()).map(\|d\| d.to_string()).unwrap_or_else(\|_\| String::new())` |

**File-edit cap:** 2 files (`pagination.rs` + `schema_setup/mod.rs`). Per advisor-orchestrator §5.3 callsite-enumeration discipline (`feedback_fix_impl_enumerate_all_callsites.md` 2026-05-11): enumerated full set; not a struct-shape change; 2 file edits ≤ default-3 cap.

### 2.3 What NOT to author

- Do NOT touch `clippy.toml` (Task 8 re-dispatch will commit it — same brief at `.claude/PRPs/briefs/brehon-conformance-audit-impl-8.md`; worker-368's uncommitted `clippy.toml` work is the seed and is correct).
- Do NOT touch `crates/diesel_utils/Cargo.toml` (option-b was REJECTED — no per-crate allow exemption).
- Do NOT add `#[allow(clippy::disallowed_methods)]` attributes (per `feedback_fix_impl_pre_push_cargo_check.md`: `#[allow]`-spam to bypass is forbidden).
- Do NOT touch `crates/utils/` (those 6 sites were remediated by fix-impl-1; verify via cargo-clippy at §5.2 confirms).
- Do NOT touch any other crate.

**Branch context:**
- `base_branch` = `phase-brehon-conformance-audit` (at the tip after daemon finalize-merges worker-368, currently waiting).
- Worker forks to its own `junior/...` worktree branch.
- ONE commit on worker branch; daemon finalize-merges into phase branch.

## 3. Required reading

### 3.0 Source DQ + plan (P0)

1. DQ #307 from `.claude/decision-queue.json` on phase tip — read `context` field verbatim (4-callsite enumeration source-of-truth).
2. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §10.8 (clippy.toml content) — the disallowed-methods seed entries.
3. `.claude/PRPs/briefs/brehon-conformance-audit-fix-impl-1.md` §2.2 — pattern precedent for `unwrap_or_default` replacements (Option-typed examples).

### 3.1 Lessons (mandatory)

4. `feedback_lemmy_error_no_std_error.md` — recipe family DQ #307 came from (axis-4 footgun catch).
5. `feedback_fix_impl_pre_push_cargo_check.md` — pre-push cargo-check discipline (constraint §4.10).
6. `feedback_fix_impl_enumerate_all_callsites.md` — enumeration-first discipline (already executed: 4 sites enumerated above).
7. `feedback_clippy_test_style.md` — workspace clippy enforcement; `-D warnings` discipline.

### 3.2 Pre-fix verification reading (target files)

8. `crates/diesel_utils/src/pagination.rs` lines 105-115 + 268-280 — confirm both sites' inner types.
9. `crates/diesel_utils/src/schema_setup/mod.rs` lines 65-80 + 110-125 — confirm both sites' `Result<String, _>` type.

### 3.3 Rules (auto-loaded)

10. `.claude/rules/decision-queue.md` — DQ #307 self-resolve shape on fix-impl landing.
11. `.claude/rules/phase-branch.md` — worker branch push.
12. `.claude/rules/cargo-output-capture.md` — capture cargo output to file, never pipe through tail.

## 4. Constraints

1. **Mid-task push discipline** — any DQ pushed immediately to worker branch.
2. **Attribution integrity** — `from: "impl"`. NEVER `answered_by: "advisor"` or `"user"`. Self-resolve DQ #307 with `answered_by: "impl-self-resolved"` ONLY after probe shows zero violations.
3. **Exactly 4 callsites edited** per §2.2 above. NOT 3, NOT 5. NO additional sweep beyond these. Per `feedback_impl_task_enumerated_transform_all_or_blocker.md`.
4. **Replacements MUST be semantically equivalent** to the original `unwrap_or_default()`:
   - `Option<bool>::unwrap_or_default()` ≡ `.unwrap_or(false)`.
   - `Result<usize, _>::unwrap_or_default()` ≡ `.unwrap_or(0)`.
   - `Result<String, _>::unwrap_or_default()` ≡ `.unwrap_or_else(|_| String::new())` (lazy; matches fix-impl-1 precedent).
5. **NO #[allow] spam** to bypass — per `feedback_fix_impl_pre_push_cargo_check.md`. If a replacement triggers a NEW lint, fix THAT lint properly (or DQ blocker if out of scope).
6. **Pre-push cargo-check + clippy probe** (mandatory) — run both BEFORE the push.
7. **NO test-behavior changes** — these 4 callsites are pagination + migration runner; if existing tests reference the post-replacement behavior, run `cargo test -p lemmy_diesel_utils` and confirm zero new failures.
8. **NO new dependencies** — these are stdlib-only replacements.
9. **Commit subject template** — `fix(lemmy_diesel_utils): replace 4 unwrap_or_default with explicit fallbacks (brehon-conformance-audit fix-impl-2)`.
10. **Pre-push cargo-check discipline** per `feedback_fix_impl_pre_push_cargo_check.md`: run `cargo-check.sh --workspace --features full` BEFORE push. Non-zero → patch in same commit (if in-scope) OR DQ blocker (if out-of-scope).
11. **Pre-push cargo-clippy probe** (replicates the Task 8 probe with clippy.toml from worker-368 finalize-merged tip): `bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings`. EXPECT exit 0 (zero violations) — this is the success gate. **clippy.toml MUST be at repo root from base_branch tip** (it's the worker-368 merge that landed it).
12. **No `--no-verify`** — never skip hooks.

## 5. Validation gate (DoD per plan §15)

### 5.1 Pre-push cargo-check (worker runs locally before push)

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-fix-impl-2-check.log 2>&1
status=$?
tail -20 .claude/PRPs/debug/brehon-conformance-audit-fix-impl-2-check.log
echo "exit: $status"
# EXPECT: exit 0 (semantically equivalent replacements should not break compile)
```

### 5.2 Pre-push cargo-clippy probe (THE SUCCESS GATE)

```bash
bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings > .claude/PRPs/debug/brehon-conformance-audit-fix-impl-2-clippy.log 2>&1
status=$?
tail -30 .claude/PRPs/debug/brehon-conformance-audit-fix-impl-2-clippy.log
echo "exit: $status"
# EXPECT: exit 0 — zero `unwrap_or_default` violations workspace-wide (lemmy_utils + lemmy_diesel_utils both clean).
# Non-zero → DO NOT commit; DQ blocker enumerating remaining violations.
```

### 5.3 Pre-push test (lemmy_diesel_utils only)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_diesel_utils --features full --no-run > .claude/PRPs/debug/brehon-conformance-audit-fix-impl-2-test-compile.log 2>&1"
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
  "question": "validate phase-brehon-conformance-audit fix-impl-2 worker branch (4 unwrap_or_default replacements in lemmy_diesel_utils)",
  "branch": "<worker-branch>",
  "phase_task": "fix-impl-2",
  "commands": [
    "bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-fix-impl-2-check.log 2>&1; echo exit: $?",
    "bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings > .claude/PRPs/debug/brehon-conformance-audit-fix-impl-2-clippy.log 2>&1; echo exit: $?"
  ],
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

### 5.5 DQ #307 self-resolve (after probes pass + push)

After §5.1 + §5.2 + §5.3 all exit 0, worker mutates DQ #307 in place:

- `answer: "option-a executed per advisor auto-mode 2026-05-21 (matches DQ #303 precedent) — 4 unwrap_or_default replaced in pagination.rs (2) + schema_setup/mod.rs (2); cargo-clippy --workspace --no-deps --features full -- -D warnings exit 0 confirmed zero violations. Task 8 clippy.toml committed via worker-368 merge; this fix-impl-2 unblocks the probe."`
- `answered_by: "impl-self-resolved"`
- `resolved_at: "<ISO>"`

DQ entry committed + pushed BEFORE worker exits (mid-task push discipline).

## 6. KNOWN harness limitations

1. **CC v2.1.119 sensitive-file gate** — `crates/diesel_utils/**` should NOT be on the sensitive-file list. If Write is blocked, `/tmp + mv` fallback per plan §19.5 LESSON.
2. **Daemon finalize-merge no-push** — advisor handles SSH-push.

## 7. Next steps after this task

- Daemon finalize-merges fix-impl-2 commit onto phase branch.
- DQ #307 transitions to `resolved` via in-place mutation.
- Advisor re-dispatches Task 8 AGAIN (third dispatch with same impl-8.md brief) — clippy.toml + probe; now exit 0 expected (lemmy_utils + lemmy_diesel_utils both clean).
- After Task 8 lands clippy.toml clean, advisor proceeds to Cohort 2.5 (Task 6 compute-metrics.sh).
- **Retro signal (Task 13)**: plan §10.8 + PRECON-4 misanalyzed workspace.lints.clippy.style — group-deny does NOT activate configurable lints; only `clippy.toml` does. Planner clarify gate should have caught this; lesson candidate.

## 8. Commit subject template (verbatim)

```
fix(lemmy_diesel_utils): replace 4 unwrap_or_default with explicit fallbacks (brehon-conformance-audit fix-impl-2)
```

Body cites:
- DQ #307 + plan path + SHA.
- Four callsites enumerated (pagination 111/274 + schema_setup/mod 73/117).
- Cargo-clippy probe exit 0 evidence.
- Co-Authored-By trailer.

## 9. DoD for this brief (advisor-side gate — completed at brief commit)

- [x] Brief committed on `phase-brehon-conformance-audit` BEFORE Junior task dispatch.
- [x] §2.2 enumerates all 4 callsites per `feedback_fix_impl_enumerate_all_callsites.md`.
- [x] §4 lists 12 constraints including pre-push cargo-check + clippy probe.
- [x] §5 names validate-pending-laptop DQ shape + DQ #307 self-resolve shape.
- [x] §G4 CANONICAL RECIPE block deferred (DQ #307 is NOT an allowlist row; advisor auto-mode option-a per DQ #303 precedent).
