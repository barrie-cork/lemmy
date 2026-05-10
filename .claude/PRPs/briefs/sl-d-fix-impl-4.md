---
phase: v1-SL-d
role: impl-task
task: fix-4
brief_n: 1
authored: 2026-05-10
---

# [role:impl-task] v1-SL-d fix-impl-4 — replace .expect() with ok_or_else in submit_jury_vote.rs — see .claude/PRPs/briefs/sl-d-fix-impl-4.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-d fix-impl-4 replace expect with ok_or_else in submit_jury_vote`

## §2 Scope

### 2.1 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md` §G4 classifier table)

> | Failure signature | Auto-fix | Source lesson |
> | `clippy::doc_lazy_continuation` warning | reword + mid-paragraph "and" | `feedback_clippy_doc_lazy_continuation_in_doc_comments.md` |

Lemmy's clippy config denies `-D clippy::expect-used`. Three `.expect()` calls in `submit_jury_vote.rs` must be replaced with `ok_or_else(|| LemmyErrorType::Unknown("...".into()))?`.

### 2.2 Specific fix

In `crates/api/api/src/governance/submit_jury_vote.rs`, inside the `if path_kind == SLDPathKind::Pending` block (around lines 691-694):

**Current code (approximate):**
```rust
if path_kind == SLDPathKind::Pending {
    let (target_pseudonym, severity_str, sponsors_pseudonyms) =
      sld_log_data.expect("set in 8.5 Pending branch");
    let grace_expires_at = sld_grace_expires_at.expect("set in 8.5 Pending branch");
```

**Replace with:**
```rust
if path_kind == SLDPathKind::Pending {
    let (target_pseudonym, severity_str, sponsors_pseudonyms) =
      sld_log_data.ok_or_else(|| LemmyErrorType::Unknown("sld_log_data missing on Pending path".into()))?;
    let grace_expires_at = sld_grace_expires_at.ok_or_else(|| LemmyErrorType::Unknown("sld_grace_expires_at missing on Pending path".into()))?;
```

There may be a third `.expect()` call nearby (the log slice shows 3 errors at lines ~687, 693, 694). Read the file to find ALL `.expect()` calls introduced by Task 2 and replace each one with `ok_or_else(|| LemmyErrorType::Unknown("...".into()))?`.

After fixing, confirm zero `.expect()` calls remain in the new Task 2 code:
```bash
rg -n '\.expect\(' crates/api/api/src/governance/submit_jury_vote.rs
```
Zero new occurrences = PASS (pre-existing `.expect()` in untouched v0 code is fine if any remain — check only the lines added by Task 2).

**Do NOT modify** any other logic, function bodies, or other files.

## §3 Required reading

- `crates/api/api/src/governance/submit_jury_vote.rs` (full file) — read to find all `.expect()` calls introduced by Task 2 (lines ~685-700 area)
- `.claude/lessons/feedback_clippy_test_style.md` — Lemmy clippy denies `.unwrap()`, `.expect()` in new code; use `ok_or_else` + `?` or `LemmyErrorType::Unknown`
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` — Shape G push re-triggers validation

## §3a Handover from prior cohort

- Task 2 commit: `ad519f1ad` — `feat(v1-SL-d): submit_jury_vote Decided→SponsorLiabilityPending transition + grace-window snapshot + deferred writes (task 2)`
- DQ #192 result: `fail` — `-D clippy::expect-used` on 3 `.expect()` calls at submit_jury_vote.rs lines ~687, 693, 694
- All three are inside `if path_kind == SLDPathKind::Pending` block; values are guaranteed Some on that path but clippy can't prove it

## §4 Constraints

- **Files:** only `crates/api/api/src/governance/submit_jury_vote.rs`. No other files.
- **≤3 line edits** (replace 2-3 `.expect()` calls with `ok_or_else`).
- **Shape G:** after committing, push the worker branch; write a `kind: "validate-pending"` DQ entry with `workflow_run_id` from the triggered `cargo-validate-workspace.yml` run.
- **COMMIT MESSAGE:** `fix(v1-SL-d): replace expect() with ok_or_else in submit_jury_vote Pending path (fix-impl-4)`
- **DQ mid-task push:** if a DQ blocker is filed, commit + push immediately per decision-queue.md.
