---
phase: v1-SL-d
role: impl-task
task: fix-1
brief_n: 1
authored: 2026-05-10
---

# [role:impl-task] v1-SL-d fix-impl-1 — dead_code clippy on Task 1 helpers — see .claude/PRPs/briefs/sl-d-fix-impl-1.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-d fix-impl-1 suppress dead_code on Task 1 helpers`

## §2 Scope

### 2.1 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md` §G4 classifier table)

> | Failure signature | Auto-fix | Source lesson |
> | `clippy::dead_code` on a function | add `#[expect(dead_code)]` or `#[allow(dead_code)]` per the clippy suggestion | n/a (mechanical) |

### 2.2 Specific fix

In `crates/api/api/src/governance/sponsor_liability.rs`, two functions added in Task 1 are flagged `-D dead_code` because Task 2 (their callers) hasn't shipped yet:

1. **Line 164:** `fn liability_severity_from_case_severity` — add `#[allow(dead_code)]` immediately above the `fn` line.
2. **Line 182:** `pub(crate) async fn grace_window_for_severity` — add `#[allow(dead_code)]` immediately above the `fn` line.

**Only these two attributes.** No other changes. The functions are correct — `compute_sponsor_liability` calls `liability_severity_from_case_severity` internally (verify by reading the file). If `liability_severity_from_case_severity` IS called inside `compute_sponsor_liability`, dead_code on it means `compute_sponsor_liability` itself isn't used yet either — check if it also needs `#[allow(dead_code)]`. Apply to any additional Task-1-introduced function that clippy flags dead_code on, but ONLY those.

**Do NOT modify:** function bodies, signatures, struct definitions, or any other file.

### 2.3 Validation

After editing, run the compute purity check from Task 1 brief to confirm no regressions:
```
rg -nE 'insert_into|update\(|append\(|persist' crates/api/api/src/governance/sponsor_liability.rs
```
Zero matches inside `compute_sponsor_liability` body = PASS.

## §3 Required reading

- `crates/api/api/src/governance/sponsor_liability.rs` (full file) — read before editing to identify ALL Task-1-introduced functions that may need `#[allow(dead_code)]`
- `.claude/lessons/feedback_clippy_test_style.md` — Lemmy clippy denies; `#[allow(dead_code)]` is the correct suppressor for transitional dead code (callers ship in Task 2)
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` — after applying the fix, the Shape G push re-triggers validation; no local cargo needed

## §3a Handover from prior cohort

Task 1 commit: `9825a2d8a` — `refactor(v1-SL-d): split apply_sponsor_liability into compute + fire + wrapper, add grace_window_for_severity (task 1)`
DQ #188 result: `fail` — failed_jobs: `['validate workspace']`
Error: `-D dead_code` on `liability_severity_from_case_severity` (line 164) and `grace_window_for_severity` (line 182).

## §4 Constraints

- **Files:** only `crates/api/api/src/governance/sponsor_liability.rs`. No other files.
- **≤3 file edits total** (this is a narrow fix — two attribute insertions).
- **Shape G:** after committing, push the worker branch; write a `kind: "validate-pending"` DQ entry with `workflow_run_id` from the triggered `cargo-validate-workspace.yml` run.
- **COMMIT MESSAGE:** `fix(v1-SL-d): suppress dead_code on Task 1 helpers pending Task 2 callers (fix-impl-1)`
- **DQ mid-task push:** if a DQ blocker is filed, commit + push immediately per decision-queue.md.
