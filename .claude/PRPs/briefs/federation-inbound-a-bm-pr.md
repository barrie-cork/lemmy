---
phase: v1-federation-inbound-a
role: bm-task
task: bm-pr
brief_n: 1
authored: 2026-05-18
canonical_ref: .claude/PRPs/briefs/sl-e-bm-pr-1.md
---

# [role:bm-task] fed-in-a bm-pr — open PR for phase-v1-federation-inbound-a — see .claude/PRPs/briefs/federation-inbound-a-bm-pr.md

## §1 Role + dispatch

`[role:bm-task] fed-in-a bm-pr — open PR for phase-v1-federation-inbound-a into governance-v0`

## §2 Scope

Run `bm-pr` for `phase-v1-federation-inbound-a`. This sub-phase delivers
the **v1 federation-inbound foundation**: federation peer trust state,
inbox nonce + dropped-log models, remote moderation labels, the Phase-6
model extensions (UpdateForm + new columns), and the e2e phase1
round-trip + trust-state fixture probes. All 9 plan tasks (Cohort A +
serial Cohort B 4/4) shipped + §5.2 workspace-validated; Phase-2 e2e
GREEN.

**Phase branch:** `phase-v1-federation-inbound-a`
**Tip:** `c18d63fbd` (DQ #267 Phase-2 e2e pass mutation; daemon == origin == lane)
**Base:** `governance-v0`
**Repo:** `barrie-cork/lemmy`

**Validation status (for PR body):**
- Cohort A: Tasks 1–5 §5.2 validated (DQ #242/#244/#246 pass).
- Cohort B serial 4/4: T6 (DQ #248 pass), T7 (DQ #249+#251 pass — fix-impl-2), T8 (DQ #252/#264 pass — fix-impl-3), T9 (fix-impl-4 + fix-impl-5; DQ #266 workspace pass, DQ #267 Phase-2 e2e pass).
- **Phase-2 e2e:** local re-run #2 @ tip `122187ebd` GREEN — `91 passed; 0 failed; 5 ignored` (5 ignored = known GH#42/#43/#45 deflakes). Single regression `v1_jm_a_backfill_populates_v0_snapshot` resolved by fix-impl-5.

**No pre-PR retro** — the phase retro is authored at USER-GATE-6 *after*
merge (standard non-lane-closer flow). Do NOT expect a retro file on the
phase branch.

## §3 Required reading

- `.claude/commands/bm/bm-pr.md` — bm-pr verb
- `.claude/rules/branch-manager.md` — BM autonomy bounds + file-ownership
- `.claude/rules/gh-pr-fork-target.md` — always `--repo barrie-cork/lemmy`
- `.claude/rules/phase-branch.md` — phase-branch + PR flow (base governance-v0, NOT main; NOT draft)
- `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` — plan reference for PR body
- `.claude/lessons/feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md` — runlog timing (write runlog entry POST-merge only; do NOT commit a gov-v0 runlog entry before gh pr merge)

## §4 Constraints

- **--repo barrie-cork/lemmy** on every `gh` command (fork default is upstream LemmyNet/lemmy).
- Base: `governance-v0` (NOT `main` — main is upstream-rebase only).
- Head: `phase-v1-federation-inbound-a`.
- **Not draft** (CodeRabbit skips drafts).
- **PR title:** `Phase v1-federation-inbound-a — federation-inbound foundation (peer trust + inbox nonce + remote moderation labels + Phase-6 model ext + e2e probes)` (≤ keep under ~120 chars; trim if gh rejects).
- **PR body** assembles from:
  - 1-line summary (federation-inbound foundation).
  - Task summary: Cohort A (Tasks 1–5) + Cohort B serial (Tasks 6–9), with the validation-status bullets from §2.
  - Plan reference: `.claude/PRPs/plans/v1-federation-inbound-a.plan.md`.
  - Phase-2 e2e result line: `91 passed; 0 failed; 5 ignored` @ tip `122187ebd` (5 ignored = known GH#42/#43/#45 deflakes).
  - `git log governance-v0..phase-v1-federation-inbound-a --oneline` (58 commits — include as a collapsed `<details>` block or summarised; do NOT paste raw if it bloats the body excessively — a per-task summary table is preferable).
  - Test plan checklist (markdown checkboxes).
- **L14 runlog discipline:** do NOT commit a `bm-runlog.md` entry to
  `governance-v0` as part of this `bm-pr` verb in a way that will
  conflict with the post-merge runlog write. Per
  `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr`: if bm-pr
  writes a phase-branch runlog entry, that is fine (it merges with the
  PR); the trunk runlog entry is written COMPLETE POST-merge only.
- **Do NOT merge the PR** — this verb opens the PR only. Merge is
  USER-GATE-5 + a separate `bm-merge` dispatch.
- Write `.claude/PRPs/reviews/pr-<N>-findings.yaml` shell (per
  SCHEMA.md) so the later `bm-poll-cr` + `bm-triage` have a target.
- Report the opened PR number + URL in the task output.

## §5 Out of scope

- Merging the PR (USER-GATE-5 / `bm-merge`).
- CodeRabbit triage (separate `bm-poll-cr` + `bm-triage` cycle).
- The phase retro (USER-GATE-6, authored post-merge).
- Any edit to `crates/**`, `migrations/**`, `tests/**`,
  `docs/brehon-law-inspired-network/**` (BM file-ownership boundary —
  per `.claude/rules/branch-manager.md`).
