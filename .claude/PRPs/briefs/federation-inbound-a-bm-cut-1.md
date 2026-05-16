---
phase: v1-federation-inbound-a
role: bm-task
task: bm-cut
brief_n: 1
authored: 2026-05-16
plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md
plan_approved: "user gate 1 — 2026-05-16 (DoD smoke PASS + watchpoint-specificity PASS + DQ #233 proceed user-ratified)"
---

# [role:bm-task] bm-cut v1-federation-inbound-a — see .claude/PRPs/briefs/federation-inbound-a-bm-cut-1.md

## §1 Role + dispatch

`[role:bm-task] bm-cut v1-federation-inbound-a — cut phase-v1-federation-inbound-a off governance-v0`

The actual create-task description (single line, <100 chars, per `.claude/rules/advisor-orchestrator.md` Junior task description template):

```
[role:bm-task] bm-cut v1-federation-inbound-a — see .claude/PRPs/briefs/federation-inbound-a-bm-cut-1.md
```

## §2 Scope

Cut `phase-v1-federation-inbound-a` off `governance-v0` (this is a PHASE branch, not a chore branch — it WILL be pushed).

- **Phase 0:** branch name `phase-v1-federation-inbound-a` (type: phase; matches `phase-v<N>-<area>-<letter>` per `.claude/rules/branch-manager.md` "Phase-branch discipline" — here `phase-v1-federation-inbound-a`).
- **Phase 1:** verify trunk (`governance-v0`) clean (`git status --short` empty modulo known runtime dirs) + synced with origin (`git fetch origin && git log governance-v0..origin/governance-v0 --oneline` empty). The approved plan file `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` MUST be present on trunk (it landed at commit `d6cd09fee`; DQ #233 resolved-proceed at `3ada2a658`).
- **Phase 2:** plan-presence check — `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` exists on `governance-v0` HEAD. (This is a phase branch with an approved plan — do NOT skip the plan check; that skip is chore-branch-only.)
- **Phase 3:** `git checkout -b phase-v1-federation-inbound-a governance-v0` THEN `git push -u origin phase-v1-federation-inbound-a` (phase branches are auto-push per `.claude/rules/branch-manager.md` autonomy table row "Push a `phase-*` branch to origin (`git push -u`) — Auto — No confirm"). The push is required so the laptop can create the lane-dedicated worktree off `origin/phase-v1-federation-inbound-a`.
- **Phase 4:** create `.claude/runlog/v1-federation-inbound-a-runlog.md` with the standard BM runlog header + a `## bm: cut phase-v1-federation-inbound-a off governance-v0 @ <trunk-sha>` first entry. Commit the runlog on the new phase branch and push.
- **Phase 5:** standard one-paragraph summary (branch created, pushed, runlog initialised, trunk SHA the branch was cut from).

## §3 Required reading

- `.claude/commands/bm/bm-cut.md` — the bm-cut verb script (follow step by step).
- `.claude/rules/branch-manager.md` — BM file-ownership + autonomy bounds + "Phase-branch discipline" + "Session-start ritual".
- `.claude/rules/phase-branch.md` — phase-branch + PR-into-`governance-v0` discipline (NOT `main`).
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` (relevant later at bm-pr, not bm-cut, but read for context).
- `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` — the approved plan this phase branch hosts (read §1 Summary + §6 Relationship + §13 task list for context; do NOT implement anything — bm-cut only creates the branch + runlog).

## §4 Constraints (hard rules — BM file-ownership)

- BM **NEVER** touches `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `Cargo.toml`. bm-cut creates ONLY the branch + `.claude/runlog/v1-federation-inbound-a-runlog.md`.
- Branch name MUST be exactly `phase-v1-federation-inbound-a`. Any other name → STOP and raise a `kind: "blocker"` DQ (`from: "bm"`).
- If trunk is NOT clean or NOT synced with origin → STOP and raise a `kind: "blocker"` DQ; do NOT force, do NOT stash, do NOT proceed.
- If the plan file is absent on `governance-v0` HEAD → STOP and raise a `kind: "blocker"` DQ (the plan landed at `d6cd09fee`; absence means trunk drifted — surface, do not improvise).
- Push is REQUIRED for this phase branch (per autonomy table — auto, no confirm). Do NOT skip the push (the laptop needs `origin/phase-v1-federation-inbound-a` to create the lane worktree).
- Do NOT open a PR (that is a later `bm-pr` task, post-impl). bm-cut is branch-creation + runlog only.
- Mid-task DQ writes (if any blocker) commit + push immediately to the current branch per `.claude/rules/decision-queue.md` "Mid-task visibility".
- Attribution: any DQ entry is `from: "bm"`, `answered_by: null` or `"bm-self-resolved"`. NEVER `"advisor"` / `"user"` / `"planner"`.

## §5 Concurrency note

Three CC advisor sessions run concurrently (this `v1-federation-inbound-a` lane + `v1-ship` + `v1-AD-e`). bm-cut here only creates a NEW branch off `governance-v0` and a NEW runlog file — zero overlap with the other lanes' phase branches or files. Per `.claude/rules/multi-lane-worktree.md`: after this bm-cut pushes `phase-v1-federation-inbound-a`, the user/advisor creates the lane-dedicated worktree `C:/Users/barri/Developer/brehon-fork-federation-inbound-a` off `origin/phase-v1-federation-inbound-a`; the lane's subsequent advisor session runs from THAT worktree (the canonical `brehon-fork` checkout stays meta-edit-only). bm-cut itself runs as a normal Junior task on the EliteDesk daemon (branches from `/srv/brehon-fork` `governance-v0` HEAD) — unaffected by the laptop worktree topology.
