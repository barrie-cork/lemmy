---
phase: v1-RT-r1
role: impl-task
task: fix-impl-4
brief_n: 4
authored: 2026-05-12
parent_cr: cr-1, cr-2
---

# [role:impl-task] RT-r1 fix-impl-4 — ApplyAt semantics: add OnRestart variant + fix 10 ConfigKeyMetadata entries — see .claude/PRPs/briefs/rt-r1-fix-impl-4.md

## §1 Role + dispatch

`[role:impl-task] RT-r1 fix-impl-4 — ApplyAt semantics: add OnRestart variant + fix 10 ConfigKeyMetadata entries`

## §2 Scope

### 2.1 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md` §G4 classifier table)

This is a CR fix-in-pr, not an allowlist §G4 entry. Two CR findings bundled:

**CR finding cr-1 (job keys):**
> In `crates/api/api/src/governance/config.rs` around line 3105-3128: The two ConfigKeyMetadata entries for "job.participation_interval_days" and "job.rollup_interval_days" currently claim restart-delayed behavior but use `apply_at_default: ApplyAt::Immediate`; change their `apply_at_default` to a restart-specific variant (e.g., `ApplyAt::OnRestart` or `ApplyAt::Restart`) to reflect "takes effect at next server restart", and if that variant does not exist add it to the ApplyAt enum (and update any match arms/tests) so callers that read `apply_at_default` see the correct restart semantics.

**CR finding cr-2 (decay keys):**
> In `crates/api/api/src/governance/config.rs` around line 2835-2932: The ConfigKeyMetadata entries for the eight decay keys currently set `apply_at_default: ApplyAt::Immediate` but should be `ApplyAt::NextSnapshotJob`; update those ConfigKeyMetadata structs to use `ApplyAt::NextSnapshotJob`.

### 2.2 File edits

**File:** `crates/api/api/src/governance/config.rs`

**Edit 1 — Add `OnRestart` variant to ApplyAt enum (line 91-95):**
- Current enum body: `Immediate`, `NextJuryCycle`, `NextSnapshotJob`
- Add `OnRestart` after `NextSnapshotJob`
- Update the doc comment at lines 86-89 to mention `OnRestart` means "takes effect at next server restart (scheduler re-init)"
- Check for match arms on `ApplyAt` in this file; add `OnRestart` arm if any match is non-exhaustive (search with `match.*ApplyAt` or `ApplyAt::` patterns)

**Edit 2 — Fix 8 decay key entries (lines 2845, 2857, 2869, 2881, 2893, 2905, 2917, 2929):**
- Change `apply_at_default: ApplyAt::Immediate` → `apply_at_default: ApplyAt::NextSnapshotJob` for all 8 decay keys:
  - `decay.reporting_accuracy.positive_half_life_days` (line 2845)
  - `decay.reporting_accuracy.negative_half_life_days` (line 2857)
  - `decay.jury_reliability.positive_half_life_days` (line 2869)
  - `decay.jury_reliability.negative_half_life_days` (line 2881)
  - `decay.participation_consistency.positive_half_life_days` (line 2893)
  - `decay.participation_consistency.negative_half_life_days` (line 2905)
  - `decay.endorsement_strength.positive_half_life_days` (line 2917)
  - `decay.endorsement_strength.negative_half_life_days` (line 2929)

**Edit 3 — Fix 2 job key entries (lines 3113, 3125):**
- Change `apply_at_default: ApplyAt::Immediate` → `apply_at_default: ApplyAt::OnRestart` for:
  - `job.participation_interval_days` (line 3113)
  - `job.rollup_interval_days` (line 3125)

**Touch only:** `crates/api/api/src/governance/config.rs`

## §3 Required reading

1. `.claude/rules/decision-queue.md` — Recipe 1 if blocker found
2. `.claude/lessons/feedback_clippy_rerun_after_fix.md` — re-run clippy after adding enum variant

## §4 Constraints

- **Touch only:** `crates/api/api/src/governance/config.rs` + `.claude/decision-queue.json` (if blocker raised)
- **≤3 file edits** — all changes in one file
- **Add `OnRestart` variant** — do NOT repurpose `Immediate`; the new variant is semantically distinct
- **Check match exhaustiveness** — grep `ApplyAt` in the file for any `match` arm; add `OnRestart` to every match or the compiler will catch it as non-exhaustive
- **Commit message:** `fix(v1-RT-r1): ApplyAt semantics — add OnRestart variant + fix decay/job ConfigKeyMetadata apply_at (cr-1/cr-2)`
- **Shape G:** after committing, push worker branch; write `kind: "validate-pending"` DQ entry with `workflow_run_id` from `gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --json databaseId`. Commit + push DQ entry.
- **DQ atomic raise:** `git add .claude/decision-queue.json && git commit && git push origin <branch>` THEN end task.
- **Encoding:** Python `json.dump(..., ensure_ascii=False, indent=2)` for DQ writes.
- **next_id:** compute across `.claude/decision-queue.json` AND `.claude/decision-queue-archive-*.json`. Current max is 209 on this phase branch — next id is 210.
- **Attribution:** `from: "impl"`, never `from: "advisor"`.
- **Base branch:** `phase-v1-RT-r1`
