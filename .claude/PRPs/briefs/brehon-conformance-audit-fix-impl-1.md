# brehon-conformance-audit — fix-impl Task 1 brief (Task 8 remediation)

## 1. Role + dispatch line

`[role:impl-task] brehon-conformance-audit fix-impl-1 — remediate 6 Option::unwrap_or_default in lemmy_utils — see .claude/PRPs/briefs/brehon-conformance-audit-fix-impl-1.md`

## 2. Scope

Resolve DQ #303 per user-chosen option-a (2026-05-20 via AskUserQuestion). Replace 6 `Option::unwrap_or_default()` callsites in `crates/utils/` with semantically equivalent alternatives that do NOT trigger `core::option::Option::unwrap_or_default` under `disallowed-methods`. Then re-run the Task 8 probe to confirm zero violations and unblock Task 8 commit.

### 2.1 §G4 CANONICAL RECIPE — N/A (not an allowlist-matched failure)

DQ #303 is **NOT** a §G4 allowlist row (no error code; pre-existing-code violation under a new probe). User-chosen option-a is the canonical answer per the AskUserQuestion record.

### 2.2 Six callsites to remediate (enumerated; this IS the full set)

Per DQ #303 context (worker-363 cargo-clippy probe `cargo-clippy.sh --workspace --no-deps --features full -- -D warnings`):

| File | Line | Pattern | Replacement |
|---|---|---|---|
| `crates/utils/src/utils/markdown/image_links.rs` | 53 | `src.get(start..end).unwrap_or_default()` | `src.get(start..end).unwrap_or("")` |
| `crates/utils/src/utils/markdown/image_links.rs` | 105 | `self.title.as_ref().map(\|t\| t.len() + 3).unwrap_or_default()` | `self.title.as_ref().map(\|t\| t.len() + 3).unwrap_or(0)` |
| `crates/utils/src/utils/markdown/image_links.rs` | 113 | `self.title.as_ref().map(\|t\| t.len() + 3).unwrap_or_default()` | `self.title.as_ref().map(\|t\| t.len() + 3).unwrap_or(0)` |
| `crates/utils/src/utils/markdown/link_rule.rs` | 50 | `href.unwrap_or_default()` | `href.unwrap_or_else(String::new)` (or `.unwrap_or(String::new())` per the field type) |
| `crates/utils/src/utils/validation.rs` | 309 | `text.char_indices().last().unwrap_or_default()` | `text.char_indices().last().unwrap_or((0, '\0'))` |
| `crates/utils/src/utils/validation.rs` | 329 | `.unwrap_or_default()` (on `Option<&[(usize, &str)]>`) | `.unwrap_or(&[])` |

**File-edit cap:** 3 files (image_links.rs + link_rule.rs + validation.rs). Per advisor-orchestrator §5.3 callsite-enumeration discipline (`feedback_fix_impl_enumerate_all_callsites.md` 2026-05-11): enumerated full set; not a struct-shape change; 3 file edits ≤ default-3 cap.

### 2.3 What NOT to author

- Do NOT touch `crates/utils/src/error.rs` (lines 324/333/343 are `Result::unwrap_or_default`, not `Option`; out of scope of DQ #303 enumeration; the probe flagged 6 Option sites, not Result).
- Do NOT touch `clippy.toml` (Task 8 commit is GATED on this fix-impl landing; clippy.toml is authored by worker-363 still uncommitted on its worker branch — that worker branch is dead; Task 8 will be re-dispatched fresh after this fix-impl lands).
- Do NOT touch `crates/utils/Cargo.toml` (option-b was REJECTED by user 2026-05-20; do NOT add `[lints.clippy] disallowed-methods = allow` exemption).
- Do NOT add `#[allow(clippy::disallowed_methods)]` attributes to any function (per `feedback_fix_impl_pre_push_cargo_check.md`: `#[allow]`-spam to bypass is forbidden).
- Do NOT touch any other crate or test file.

**Branch context:**
- `base_branch` = `phase-brehon-conformance-audit` (at `20a8e3a7e`).
- Worker forks to its own `junior/...` worktree branch.
- ONE commit on worker branch; daemon finalize-merges into phase branch.

## 3. Required reading

### 3.0 Source DQ + plan (P0)

1. DQ #303 from `.claude/decision-queue.json` on phase tip — read `context` field verbatim (6-callsite enumeration source-of-truth).
2. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §10.8 (clippy.toml content) — the disallowed-methods seed entries that this fix-impl unblocks.

### 3.1 Lessons (mandatory)

3. `feedback_lemmy_error_no_std_error.md` — the recipe family DQ #303 came from (axis-4 footgun catch).
4. `feedback_fix_impl_pre_push_cargo_check.md` — pre-push cargo-check discipline (constraint §4.10).
5. `feedback_fix_impl_enumerate_all_callsites.md` — enumeration-first discipline (already executed: 6 sites enumerated above).
6. `feedback_clippy_test_style.md` — workspace clippy enforcement; `-D warnings` discipline.

### 3.2 Pre-fix verification reading (target files)

7. `crates/utils/src/utils/markdown/image_links.rs` lines 50-115 — confirm the three sites and their `Option<T>` inner types (T = `&str` at line 53; T = `usize` at lines 105 + 113).
8. `crates/utils/src/utils/markdown/link_rule.rs` lines 40-55 — confirm the `href` argument type (in markdown-it's full_link::add closure signature, `href` is `Option<String>` per markdown-it docs).
9. `crates/utils/src/utils/validation.rs` lines 300-340 — confirm line 309's `Option<(usize, char)>` and line 329's slice type.

### 3.3 Rules (auto-loaded)

10. `.claude/rules/decision-queue.md` — DQ #303 self-resolve shape on fix-impl landing (mutate to `answered_by: "impl-self-resolved"` after the fix commits + cargo-clippy probe shows exit 0).
11. `.claude/rules/phase-branch.md` — worker branch push.
12. `.claude/rules/cargo-output-capture.md` — capture cargo output to file, never pipe through tail.

## 4. Constraints

1. **Mid-task push discipline** — any DQ pushed immediately to worker branch.
2. **Attribution integrity** — `from: "impl"`. NEVER `answered_by: "advisor"` or `"user"`. Self-resolve DQ #303 with `answered_by: "impl-self-resolved"` ONLY after probe shows zero violations.
3. **Exactly 6 callsites edited** per §2.2 above. NOT 5, NOT 7. NO additional Option::unwrap_or_default sweep beyond these. Per `feedback_impl_task_enumerated_transform_all_or_blocker.md`.
4. **Replacements MUST be semantically equivalent** to the original `unwrap_or_default()` — same panic-vs-default behavior, same return type.
   - `Option<&str>::unwrap_or_default()` ≡ `.unwrap_or("")`.
   - `Option<usize>::unwrap_or_default()` ≡ `.unwrap_or(0)`.
   - `Option<String>::unwrap_or_default()` ≡ `.unwrap_or_else(String::new)` (lazy, allocates only on None branch) OR `.unwrap_or(String::new())` (eager — only acceptable on hot paths if profiling shows). Use `.unwrap_or_else(String::new)` by default.
   - `Option<(usize, char)>::unwrap_or_default()` ≡ `.unwrap_or((0, '\0'))`.
   - `Option<&[T]>::unwrap_or_default()` where `T: 'static` ≡ `.unwrap_or(&[])`.
5. **NO #[allow] spam** to bypass — per `feedback_fix_impl_pre_push_cargo_check.md`. If a replacement triggers a NEW lint, fix THAT lint properly (or DQ blocker if out of scope).
6. **Pre-push cargo-check + clippy probe** (mandatory per constraint §4.10 below) — run both BEFORE the push.
7. **NO test-behavior changes** — these 6 callsites are markdown parsing + validation; if existing tests reference the post-replacement behavior, run `cargo test -p lemmy_utils` and confirm zero new failures.
8. **NO new dependencies** — these are stdlib-only replacements.
9. **Commit subject template** — `fix(lemmy_utils): replace 6 Option::unwrap_or_default with explicit fallbacks (brehon-conformance-audit fix-impl-1)`.
10. **Pre-push cargo-check discipline** per `feedback_fix_impl_pre_push_cargo_check.md`: run `cargo-check.sh --workspace --features full` BEFORE push. Non-zero → patch in same commit (if in-scope) OR DQ blocker (if out-of-scope).
11. **Pre-push cargo-clippy probe** (replicates the Task 8 probe): `bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings`. EXPECT exit 0 (zero violations) — this is the success gate.
12. **No `--no-verify`** — never skip hooks.

## 5. Validation gate (DoD per plan §15)

### 5.1 Pre-push cargo-check (worker runs locally before push)

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-fix-impl-1-check.log 2>&1
status=$?
tail -20 .claude/PRPs/debug/brehon-conformance-audit-fix-impl-1-check.log
echo "exit: $status"
# EXPECT: exit 0 (semantically equivalent replacements should not break compile)
```

### 5.2 Pre-push cargo-clippy probe (THE SUCCESS GATE)

```bash
bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings > .claude/PRPs/debug/brehon-conformance-audit-fix-impl-1-clippy.log 2>&1
status=$?
tail -30 .claude/PRPs/debug/brehon-conformance-audit-fix-impl-1-clippy.log
echo "exit: $status"
# EXPECT: exit 0 — zero `Option::unwrap_or_default` violations in lemmy_utils.
# Non-zero → DO NOT commit; DQ blocker enumerating remaining violations.
```

### 5.3 Pre-push test (lemmy_utils only)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_utils --features full --no-run > .claude/PRPs/debug/brehon-conformance-audit-fix-impl-1-test-compile.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0 — semantically equivalent replacements should not break tests' compile
```

### 5.4 §15 validate-pending-laptop DQ (raise after worker push)

```json
{
  "id": "<next>",
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO>",
  "question": "validate phase-brehon-conformance-audit fix-impl-1 worker branch (6 unwrap_or_default replacements)",
  "branch": "<worker-branch>",
  "phase_task": "fix-impl-1",
  "commands": [
    "bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-fix-impl-1-check.log 2>&1; echo exit: $?",
    "bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings > .claude/PRPs/debug/brehon-conformance-audit-fix-impl-1-clippy.log 2>&1; echo exit: $?"
  ],
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

### 5.5 DQ #303 self-resolve (after probes pass + push)

After §5.1 + §5.2 + §5.3 all exit 0, worker mutates DQ #303 in place:

- `answer: "option-a executed per user 2026-05-20 — 6 Option::unwrap_or_default replaced in image_links.rs (3) + link_rule.rs (1) + validation.rs (2); cargo-clippy --workspace --no-deps --features full -- -D warnings exit 0 confirmed zero violations. clippy.toml ready to commit on next Task 8 worker re-dispatch."`
- `answered_by: "impl-self-resolved"`
- `resolved_at: "<ISO>"`

DQ entry committed + pushed BEFORE worker exits (mid-task push discipline).

## 6. KNOWN harness limitations

1. **CC v2.1.119 sensitive-file gate** — `crates/utils/**` should NOT be on the sensitive-file list (Rust source is the worker's normal write target). If Write is blocked, `/tmp + mv` fallback per plan §19.5 LESSON; but if specifically the Edit tool is blocked, raise blocker DQ — Junior's `crates/**` allow-list is presumed in the daemon settings.
2. **Daemon finalize-merge no-push** — advisor handles SSH-push.
3. **Concurrent-cargo serialization** — if other validate-pending-laptop entries are in flight when this fix-impl runs, advisor sequences them per advisor-orchestrator §5.2.

## 7. Next steps after this task

- Daemon finalize-merges fix-impl-1 commit onto phase branch.
- DQ #303 transitions to `resolved` via in-place mutation.
- Advisor re-dispatches Task 8 (Cohort 2 retry) — same brief, same probe; now exit 0 expected.
- After Task 8 lands clean clippy.toml, advisor proceeds to Cohort 4 (Task 9) — which simplifies given workspace lint config already does the deny escalation (planner may revise Task 9 scope at retro).

## 8. Commit subject template (verbatim)

```
fix(lemmy_utils): replace 6 Option::unwrap_or_default with explicit fallbacks (brehon-conformance-audit fix-impl-1)
```

Body cites:
- DQ #303 + plan path + SHA.
- Six callsites enumerated (image_links 53/105/113 + link_rule 50 + validation 309/329).
- Cargo-clippy probe exit 0 evidence.
- Co-Authored-By trailer.

## 9. DoD for this brief (advisor-side gate — completed at brief commit)

- [x] Brief committed on `phase-brehon-conformance-audit` BEFORE Junior task dispatch.
- [x] §2.2 enumerates all 6 callsites per `feedback_fix_impl_enumerate_all_callsites.md`.
- [x] §4 lists 12 constraints including pre-push cargo-check + clippy probe.
- [x] §5 names validate-pending-laptop DQ shape + DQ #303 self-resolve shape.
- [x] §G4 CANONICAL RECIPE block deferred (DQ #303 is NOT an allowlist row; user-relay supplied the answer).
