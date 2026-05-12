---
phase: v1-SL-d
role: impl-task
task: fix-5
brief_n: 1
authored: 2026-05-10
---

# [role:impl-task] v1-SL-d fix-impl-5 — add missing use diesel_async::AsyncConnection in mod v1_sl_d_fixtures — see .claude/PRPs/briefs/sl-d-fix-impl-5.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-d fix-impl-5 add missing AsyncConnection import in e2e mod`

## §2 Scope

### 2.1 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md` §G4 classifier table)

> | Failure signature | Auto-fix | Source lesson |
> | `error[E0432]: unresolved import` | add the missing `use` per the suggestion | n/a (mechanical) |

Compiler error `E0599: no function or associated item named \`establish\` found for struct \`AsyncPgConnection\`` at e2e.rs:12911 and 13021. Compiler suggestion: add `use diesel_async::AsyncConnection;` at line 12822 (the start of `mod v1_sl_d_fixtures`).

### 2.2 Specific fix

In `crates/server/tests/e2e.rs`, inside `mod v1_sl_d_fixtures`, add `use diesel_async::AsyncConnection;` to the use block near the top of the mod (around line 12822).

The compiler points to line 12822 as the insertion point. Read the file to find the exact use block in `mod v1_sl_d_fixtures` and insert the missing import there.

**Only this one line addition.** No other changes.

### 2.3 Validation

After editing, confirm:
```
grep -n 'use diesel_async::AsyncConnection' crates/server/tests/e2e.rs
```
Should return at least one match inside `mod v1_sl_d_fixtures`.

## §3 Required reading

- `crates/server/tests/e2e.rs:12820-12830` — the use block at the top of `mod v1_sl_d_fixtures` to find the exact insertion point
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` requires `AsyncConnection` trait in scope
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` — Shape G push re-triggers validation

## §3a Handover from prior cohort

- Task 3 commit: `aa4ee3238` — `test(v1-SL-d): e2e test #1 — Decided → SponsorLiabilityPending transition + mod v1_sl_d_fixtures shell (task 3)`
- DQ #194 result: `fail` — E0599 at lines 12911 + 13021: `AsyncPgConnection::establish` not found; missing `use diesel_async::AsyncConnection;`
- Compiler suggestion: add at line 12822

## §4 Constraints

- **Files:** only `crates/server/tests/e2e.rs`. No other files.
- **1 line addition only** — `use diesel_async::AsyncConnection;` inside `mod v1_sl_d_fixtures` use block.
- **Shape G:** after committing, push the worker branch; write a `kind: "validate-pending"` DQ entry with `workflow_run_id` from the triggered `cargo-validate-workspace.yml` run.
- **COMMIT MESSAGE:** `fix(v1-SL-d): add missing AsyncConnection import in mod v1_sl_d_fixtures (fix-impl-5)`
- **DQ mid-task push:** if a DQ blocker is filed, commit + push immediately per decision-queue.md.
