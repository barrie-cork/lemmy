---
phase: v1-SL-d
role: impl-task
task: fix-2
brief_n: 1
authored: 2026-05-10
---

# [role:impl-task] v1-SL-d fix-impl-2 — replace #[allow] with #[expect] on Task 1 helpers — see .claude/PRPs/briefs/sl-d-fix-impl-2.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-d fix-impl-2 replace allow with expect dead_code`

## §2 Scope

### 2.1 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md` §G4 classifier table)

> | Failure signature | Auto-fix | Source lesson |
> | `warning: use of deprecated <api>` | replace with the suggested replacement | n/a (mechanical) |

The clippy lint `-D clippy::allow-attributes` rejects `#[allow(...)]` and requires `#[expect(...)]` instead. This is the Lemmy workspace's `allow_attributes` deny rule. The fix is a direct replacement per the clippy `help: replace it with: \`expect\`` suggestion.

### 2.2 Specific fix

In `crates/api/api/src/governance/sponsor_liability.rs`:

1. **Line 164:** Change `#[allow(dead_code)]` → `#[expect(dead_code)]`
2. **Line 183:** Change `#[allow(dead_code)]` → `#[expect(dead_code)]`

**ONLY these two substitutions.** No other changes to the file. Do not change function bodies, signatures, struct definitions, or any other attribute.

Verify the exact current line numbers by reading the file first — the line numbers from the log (164, 183) are from the compiled artifact; the source may differ by a line or two after prior edits.

### 2.3 Validation

After editing, confirm with:
```
rg -n 'allow(dead_code)' crates/api/api/src/governance/sponsor_liability.rs
```
Should return zero matches (all `allow` replaced with `expect`).

## §3 Required reading

- `crates/api/api/src/governance/sponsor_liability.rs` (full file) — read before editing to find the exact line numbers of the two attributes
- `.claude/lessons/feedback_clippy_test_style.md` — Lemmy clippy denies `#[allow]`; always use `#[expect]` per `-D clippy::allow-attributes`
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` — after fix, Shape G push re-triggers validation

## §3a Handover from prior cohort

- fix-impl-1 commit: `cb8df2176` — added `#[allow(dead_code)]` (wrong form)
- DQ #189 result: `fail` — `-D clippy::allow_attributes` fires on `#[allow(dead_code)]` at lines 164 + 183
- Clippy suggestion: `help: replace it with: \`expect\``
- This is cycle 2 on `(clippy::allow_attributes, sponsor_liability.rs)`. If this fix also fails = catch-fire (cycle 3 rule).

## §4 Constraints

- **Files:** only `crates/api/api/src/governance/sponsor_liability.rs`. No other files.
- **≤3 file edits** (two attribute replacements only).
- **Shape G:** after committing, push the worker branch; write a `kind: "validate-pending"` DQ entry with `workflow_run_id` from the triggered `cargo-validate-workspace.yml` run.
- **COMMIT MESSAGE:** `fix(v1-SL-d): replace allow with expect dead_code on Task 1 helpers (fix-impl-2)`
- **DQ mid-task push:** if a DQ blocker is filed, commit + push immediately per decision-queue.md.
