---
phase: v1-SL-d
role: impl-task
task: fix-3
brief_n: 1
authored: 2026-05-10
---

# [role:impl-task] v1-SL-d fix-impl-3 — remove spurious #[expect(dead_code)] from liability_severity_from_case_severity — see .claude/PRPs/briefs/sl-d-fix-impl-3.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-d fix-impl-3 remove spurious expect dead_code from liability_severity_from_case_severity`

## §2 Scope

### 2.1 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md` §G4 classifier table)

> | Failure signature | Auto-fix | Source lesson |
> | `warning: use of deprecated <api>` | replace with the suggested replacement | n/a (mechanical) |

The clippy lint `-D unfulfilled_lint_expectations` fires because `#[expect(dead_code)]` was placed on `liability_severity_from_case_severity` but that function IS called (by `grace_window_for_severity` in the same file). The `#[expect]` attribute is spurious — remove it.

### 2.2 Specific fix

In `crates/api/api/src/governance/sponsor_liability.rs`:

**REMOVE** the `#[expect(dead_code)]` attribute line that appears immediately before `fn liability_severity_from_case_severity`. The function body and signature stay unchanged.

**KEEP** the `#[expect(dead_code)]` attribute on `grace_window_for_severity` — that function has no callers yet (Task 2 will be the first caller), so dead_code fires legitimately and the expect is fulfilled.

**Do NOT touch** any other line.

### 2.3 Why only one removal

`liability_severity_from_case_severity` is called by `grace_window_for_severity` within the same file (line: `let liability_sev = liability_severity_from_case_severity(severity);`). The dead_code lint only fires on the unreachable entry-point (`grace_window_for_severity`), not on its callees. So:
- `grace_window_for_severity`: dead_code fires (no external caller yet) → `#[expect(dead_code)]` correct → KEEP
- `liability_severity_from_case_severity`: called within file → dead_code does NOT fire → `#[expect(dead_code)]` is unfulfilled → REMOVE

### 2.4 Validation

After editing, confirm:
```
rg -n 'expect(dead_code)' crates/api/api/src/governance/sponsor_liability.rs
```
Should return exactly ONE match (on `grace_window_for_severity`), not two.

## §3 Required reading

- `crates/api/api/src/governance/sponsor_liability.rs` (full file) — confirm the single call site of `liability_severity_from_case_severity` before removing the attribute
- `.claude/lessons/feedback_clippy_test_style.md` — `#[expect]` is correct form; `#[allow]` is denied; unfulfilled `#[expect]` is also denied
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` — Shape G push re-triggers validation after fix

## §3a Handover from prior cohort

- fix-impl-2 commit: `3302ceb75` — added `#[expect(dead_code)]` to BOTH functions
- DQ #190 result: `fail` — `unfulfilled_lint_expectations` on `liability_severity_from_case_severity` line 164 (the function IS called by grace_window_for_severity)
- Diagnosis: only `grace_window_for_severity` needs `#[expect(dead_code)]`; `liability_severity_from_case_severity` does not

## §4 Constraints

- **Files:** only `crates/api/api/src/governance/sponsor_liability.rs`. No other files.
- **1 line deletion only** — remove the `#[expect(dead_code)]` above `fn liability_severity_from_case_severity`. Nothing else.
- **Shape G:** after committing, push the worker branch; write a `kind: "validate-pending"` DQ entry with `workflow_run_id` from the triggered `cargo-validate-workspace.yml` run.
- **COMMIT MESSAGE:** `fix(v1-SL-d): remove spurious expect dead_code from liability_severity_from_case_severity (fix-impl-3)`
- **DQ mid-task push:** if a DQ blocker is filed, commit + push immediately per decision-queue.md.
