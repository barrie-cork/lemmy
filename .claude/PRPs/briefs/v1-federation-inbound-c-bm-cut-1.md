---
phase: v1-federation-inbound-c
role: bm-task
task: bm-cut
brief_n: 1
authored: 2026-05-21
plan: .claude/PRPs/plans/v1-federation-inbound-c.plan.md
plan_approved: "user gate 1 — 2026-05-21 (DoD smoke PASS [§15.1 check exit0 2m23s / §15.2 clippy exit0 8m56s, both 0 err on governance-v0 @ 22f15bd9a (cherry-picked from worker 3b187627e)] + watchpoint-specificity PASS (file:line citations verified vs HEAD: inbox.rs:421/426/427, publish_trust_attestation.rs:139/145/146, config.rs:757, e2e.rs:15510/15600/15631) + dogfood PASS (Task 3 cites e2e.rs:15600-15628 sibling, Tasks 1+2 cite config.rs:740-769 canonical mirror))"
---

# [role:bm-task] bm-cut v1-federation-inbound-c — see .claude/PRPs/briefs/v1-federation-inbound-c-bm-cut-1.md

## §1 Role + dispatch

`[role:bm-task] bm-cut v1-federation-inbound-c — cut phase-v1-federation-inbound-c off governance-v0`

The actual create-task description (single line, <100 chars, per `.claude/rules/advisor-orchestrator.md` Junior task description template):

```
[role:bm-task] bm-cut v1-federation-inbound-c — see .claude/PRPs/briefs/v1-federation-inbound-c-bm-cut-1.md
```

## §2 Scope

Cut `phase-v1-federation-inbound-c` off `governance-v0` (this is a PHASE branch, not a chore branch — it WILL be pushed).

- **Phase 0:** branch name `phase-v1-federation-inbound-c` (type: phase; matches `phase-v<N>-<area>-<letter>` per `.claude/rules/branch-manager.md` "Phase-branch discipline" — here `phase-v1-federation-inbound-c`).
- **Phase 1:** verify trunk (`governance-v0`) clean (`git status --short` empty modulo known runtime dirs) + synced with origin (`git fetch origin && git log governance-v0..origin/governance-v0 --oneline` empty). The approved plan file `.claude/PRPs/plans/v1-federation-inbound-c.plan.md` MUST be present on trunk (it landed at commit `22f15bd9a`; cherry-picked from planning Junior #391 worker commit `3b187627e`; advisor finalize-merge of planner's commit 2026-05-21).
- **Phase 2:** plan-presence check — `.claude/PRPs/plans/v1-federation-inbound-c.plan.md` exists on `governance-v0` HEAD. (This is a phase branch with an approved plan — do NOT skip the plan check; that skip is chore-branch-only.)
- **Phase 3:** `git checkout -b phase-v1-federation-inbound-c governance-v0` THEN `git push -u origin phase-v1-federation-inbound-c` (phase branches are auto-push per `.claude/rules/branch-manager.md` autonomy table row "Push a `phase-*` branch to origin (`git push -u`) — Auto — No confirm"). The push is required so the laptop can create the lane-dedicated worktree off `origin/phase-v1-federation-inbound-c`.
- **Phase 4:** create `.claude/runlog/v1-federation-inbound-c-runlog.md` with the standard BM runlog header + a `## bm: cut phase-v1-federation-inbound-c off governance-v0 @ <trunk-sha>` first entry. Commit the runlog on the new phase branch and push.
- **Phase 5:** standard one-paragraph summary (branch created, pushed, runlog initialised, trunk SHA the branch was cut from).

## §3 Required reading

- `.claude/commands/bm/bm-cut.md` — the bm-cut verb script (follow step by step).
- `.claude/rules/branch-manager.md` — BM file-ownership + autonomy bounds + "Phase-branch discipline" + "Session-start ritual".
- `.claude/rules/phase-branch.md` — phase-branch + PR-into-`governance-v0` discipline (NOT `main`).
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` (relevant later at bm-pr, not bm-cut, but read for context).
- `.claude/PRPs/plans/v1-federation-inbound-c.plan.md` — the approved plan this phase branch hosts (read §1 Summary + §6 Relationship + §13 task list for context; do NOT implement anything — bm-cut only creates the branch + runlog).

## §4 Constraints (hard rules — BM file-ownership)

- BM **NEVER** touches `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `Cargo.toml`. bm-cut creates ONLY the branch + `.claude/runlog/v1-federation-inbound-c-runlog.md`.
- Branch name MUST be exactly `phase-v1-federation-inbound-c`. Any other name → STOP and raise a `kind: "blocker"` DQ (`from: "bm"`).
- If trunk is NOT clean or NOT synced with origin → STOP and raise a `kind: "blocker"` DQ; do NOT force, do NOT stash, do NOT proceed.
- If the plan file is absent on `governance-v0` HEAD → STOP and raise a `kind: "blocker"` DQ (the plan landed at `22f15bd9a`; absence means trunk drifted — surface, do not improvise).
- Push is REQUIRED for this phase branch (per autonomy table — auto, no confirm). Do NOT skip the push (the laptop needs `origin/phase-v1-federation-inbound-c` to create the lane worktree).
- Do NOT open a PR (that is a later `bm-pr` task, post-impl). bm-cut is branch-creation + runlog only.
- Mid-task DQ writes (if any blocker) commit + push immediately to the current branch per `.claude/rules/decision-queue.md` "Mid-task visibility".
- Attribution: any DQ entry is `from: "bm"`, `answered_by: null` or `"bm-self-resolved"`. NEVER `"advisor"` / `"user"` / `"planner"`.

## §5 Concurrency note

Check `git worktree list` at session start per `.claude/rules/multi-lane-worktree.md` "Session-start ritual". bm-cut here only creates a NEW branch off `governance-v0` and a NEW runlog file — zero overlap with any other active lane's phase branch or files (the brehon-conformance-audit lane runs from `brehon-fork-conformance-audit` on `phase-brehon-conformance-audit`; its files do not touch `phase-v1-federation-inbound-c` topology). After this bm-cut pushes `phase-v1-federation-inbound-c`, the user/advisor creates the lane-dedicated worktree `C:/Users/barri/Developer/brehon-fork-fed-in-c` off `origin/phase-v1-federation-inbound-c` (per `.claude/rules/multi-lane-worktree.md` + `feedback_phase_lane_worktree_bootstrap_checklist.md`); the lane's subsequent advisor session runs from THAT worktree (the canonical `brehon-fork` checkout stays meta-edit-only). bm-cut itself runs as a normal Junior task on the EliteDesk daemon (branches from `/srv/brehon-fork` `governance-v0` HEAD — `/precheck` re-verifies daemon-local sync before this task is queued, since the daemon-local trunk may lag the laptop tip; ff-via `git fetch origin gov:gov` is lane-safe per `feedback_daemon_local_trunk_stale_multi_lane.md`) — unaffected by the laptop worktree topology.

---

_Brief author: advisor session (laptop CWD `C:/Users/barri/Developer/brehon-fork`, canonical checkout on `governance-v0` @ `22f15bd9a`, 2026-05-21). Mirrors the canonical sibling `.claude/PRPs/briefs/v1-federation-inbound-b-bm-cut-1.md` per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate. Plan approved at User Gate 1 (DoD smoke PASS [check 2m23s + clippy 8m56s exit 0 on `22f15bd9a`] + watchpoint-specificity PASS + dogfood PASS). Brief committed on `governance-v0` before the Junior bm-cut task is queued._
