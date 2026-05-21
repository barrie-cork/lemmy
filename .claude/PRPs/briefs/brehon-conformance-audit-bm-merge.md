---
phase: brehon-conformance-audit
role: bm-task
task: bm-merge
brief_n: 1
authored: 2026-05-21
canonical_ref: .claude/PRPs/briefs/rt-r1-bm-merge-1.md
---

# [role:bm-task] brehon-conformance-audit bm-merge — merge PR #141 phase-brehon-conformance-audit into governance-v0 — see .claude/PRPs/briefs/brehon-conformance-audit-bm-merge.md

## §1 Role + dispatch

`[role:bm-task] brehon-conformance-audit bm-merge — merge PR #141 phase-brehon-conformance-audit into governance-v0`

## §2 Scope

Run `bm-merge` for PR #141 (`phase-brehon-conformance-audit` → `governance-v0`).

**User has explicitly confirmed merge** (User Gate 5 satisfied in advisor session 2026-05-21).

- PR #141: "Phase brehon-conformance-audit — Phase-6 convention-divergence skill + Clippy gate"
- Base: `governance-v0`
- Head: `phase-brehon-conformance-audit` (tip `ca46cfda1`)
- Repo: `barrie-cork/lemmy`
- ~95 commits; all CR fix-in-pr findings resolved (7 majors in `d2c7f9550`); cr-1 carry-forward → issue #142; CI mergeable=MERGEABLE

Merge with `--merge` (no squash, no rebase — task-per-commit history is load-bearing for retros per `phase-branch.md`).

## §3 Required reading

- `.claude/commands/bm/bm-merge.md` — full merge gate + execute script
- `.claude/rules/branch-manager.md` — autonomy bounds + L14/L15/L16 fixes
- `.claude/rules/gh-pr-fork-target.md` — always `--repo barrie-cork/lemmy`
- `.claude/runlog/bm-runlog.md` — append merge entry POST-merge per L14
- `.claude/lessons/feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md` — L14 runlog timing (CRITICAL)

## §4 Constraints

- **READ THIS BRIEF FIRST** (the first Read tool call MUST be on this file at the path in the dispatch line — per `feedback_bm_false_success_advisor_post_condition_catch.md`).
- `gh pr merge 141 --repo barrie-cork/lemmy --merge --delete-branch` — no squash, no rebase. The `--delete-branch` flag removes the remote phase branch on successful merge (L16 enforcement).
- Do NOT merge into `main` (main is upstream-rebase only per `phase-branch.md`).
- **L14 — runlog COMPLETE entry POST gh pr merge (NOT before).** Per `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr`: do NOT commit a `bm-runlog.md` entry to `governance-v0` BEFORE `gh pr merge`. The exact git sequence is:
  1. `gh pr merge 141 --repo barrie-cork/lemmy --merge --delete-branch` → wait for success (merge SHA captured)
  2. `git checkout governance-v0 && git pull --ff-only origin governance-v0` → pull the merge SHA locally
  3. Edit `.claude/runlog/bm-runlog.md` to append a `## bm: merge — <ISO ts>` block:
     ```
     ## bm: merge PR #141 — <ISO ts>
     - **PR:** #141 — Phase brehon-conformance-audit
     - **Merge SHA:** <merge-sha> (chronologically after gh pr merge returned)
     - **Phase branch:** phase-brehon-conformance-audit (deleted via --delete-branch)
     - **CR findings:** 23 total — 7 fix-in-pr (resolved d2c7f9550) / 1 carry-forward (cr-1 → issue #142) / 15 wont-fix
     - **Recommendation pre-merge:** approve
     ```
  4. `git add .claude/runlog/bm-runlog.md && git commit -m "chore(bm): merge PR #141 complete — runlog COMPLETE entry"`
  5. `git push origin governance-v0`
- **L16 — verify remote branch deletion.** After `gh pr merge --delete-branch`, run `git ls-remote origin refs/heads/phase-brehon-conformance-audit` — must return EMPTY. If non-empty (silent-skip), run `gh api -X DELETE -H "Accept: application/vnd.github+json" /repos/barrie-cork/lemmy/git/refs/heads/phase-brehon-conformance-audit` and re-verify.
- **L15** — the merge-gate's read-only checks (mergeable, recommendation, fix-in-pr addressed_in) ran INLINE in the advisor session at User Gate 5. The BM Junior should NOT re-run those gate checks; this brief assumes they are PASS.
- Do NOT edit `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**` (BM file-ownership boundary).
- User gate 5 already cleared — do NOT re-ask for merge confirmation via AskUserQuestion.
- Report in task output: merge SHA, deleted-branch confirmation, runlog commit SHA.

## §5 Out of scope

- Any fix-in-PR work (all 7 majors already in `d2c7f9550`).
- Any new DQ writes (the phase's outstanding pending DQ #326 is a historical artifact, advisor-laptop will mutate post-merge).
- Posting any PR comment or review (already done at User Gate 3).
- `/brehon-phase-transition` (separate verb, advisor-driven post-merge).
- PMD sync (`bash scripts/sync-lessons-to-pmd.sh`) — separate post-merge advisor-side step.

## §6 Commit subjects

Two commits expected:
1. The merge itself is created by `gh pr merge` (no manual commit subject — GH auto-generates `Merge pull request #141 from barrie-cork/phase-brehon-conformance-audit`).
2. `chore(bm): merge PR #141 complete — runlog COMPLETE entry` per `.claude/rules/branch-manager.md` autonomy + L14 timing.
