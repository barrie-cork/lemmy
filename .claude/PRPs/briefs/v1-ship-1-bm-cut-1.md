---
phase: v1-ship-1
role: bm-task
task: bm-cut
brief_n: 1
authored: 2026-05-16
plan: .claude/PRPs/plans/v1-ship-1-r1.plan.md
plan_approved: "user gate 1 — 2026-05-16 (DoD smoke 3/3 PASS local + watchpoint-specificity PASS; plan salvaged from blocked Junior #270, content verbatim Junior-authored, landed at commit 1e2171310)"
---

# [role:bm-task] bm-cut v1-ship-1 — see .claude/PRPs/briefs/v1-ship-1-bm-cut-1.md

## §1 Role + dispatch

`[role:bm-task] bm-cut v1-ship-1 — cut phase-v1-ship-1 off governance-v0`

The actual create-task description (single line, <100 chars, per `.claude/rules/advisor-orchestrator.md` Junior task description template):

```
[role:bm-task] bm-cut v1-ship-1 — see .claude/PRPs/briefs/v1-ship-1-bm-cut-1.md
```

## §2 Scope

Cut `phase-v1-ship-1` off `governance-v0` (this is a PHASE branch, not a chore branch — it WILL be pushed).

- **Phase 0:** branch name `phase-v1-ship-1` (type: phase; matches `phase-v<N>-<area>-<letter>`-class per `.claude/rules/branch-manager.md` "Phase-branch discipline" — here `phase-v1-ship-1`; the plan's §5 Metadata names `phase-v1-ship-1` as the branch).
- **Phase 1:** verify trunk (`governance-v0`) clean (`git status --short` empty modulo known runtime dirs) + synced with origin (`git fetch origin && git log governance-v0..origin/governance-v0 --oneline` empty). The approved plan file `.claude/PRPs/plans/v1-ship-1-r1.plan.md` MUST be present on trunk (it landed at commit `1e2171310`, now an ancestor of the current `governance-v0` tip).
- **Phase 2:** plan-presence check — `.claude/PRPs/plans/v1-ship-1-r1.plan.md` exists on `governance-v0` HEAD. (This is a phase branch with an approved plan — do NOT skip the plan check; that skip is chore-branch-only. NOTE: the plan file is `v1-ship-1-r1.plan.md` — the `-r1` re-plan suffix; the parked `v1-ship-1.plan.md` is the older audit-trail copy and is NOT this phase's plan. Confirm the `-r1` file specifically.)
- **Phase 3:** `git checkout -b phase-v1-ship-1 governance-v0` THEN `git push -u origin phase-v1-ship-1` (phase branches are auto-push per `.claude/rules/branch-manager.md` autonomy table row "Push a `phase-*` branch to origin (`git push -u`) — Auto — No confirm"). The push is required so the laptop can create the lane-dedicated worktree off `origin/phase-v1-ship-1`.
- **Phase 4:** create `.claude/runlog/v1-ship-1-runlog.md` with the standard BM runlog header + a `## bm: cut phase-v1-ship-1 off governance-v0 @ <trunk-sha>` first entry. Commit the runlog on the new phase branch and push.
- **Phase 5:** standard one-paragraph summary (branch created, pushed, runlog initialised, trunk SHA the branch was cut from).

## §3 Required reading

- `.claude/commands/bm/bm-cut.md` — the bm-cut verb script (follow step by step).
- `.claude/rules/branch-manager.md` — BM file-ownership + autonomy bounds + "Phase-branch discipline" + "Session-start ritual".
- `.claude/rules/phase-branch.md` — phase-branch + PR-into-`governance-v0` discipline (NOT `main`).
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` (relevant later at bm-pr, not bm-cut, but read for context).
- `.claude/PRPs/plans/v1-ship-1-r1.plan.md` — the approved plan this phase branch hosts (read §1 Summary + §6 Relationship + §13 task list for context; do NOT implement anything — bm-cut only creates the branch + runlog).

## §4 Constraints (hard rules — BM file-ownership)

- BM **NEVER** touches `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `Cargo.toml`. bm-cut creates ONLY the branch + `.claude/runlog/v1-ship-1-runlog.md`.
- Branch name MUST be exactly `phase-v1-ship-1`. Any other name → STOP and raise a `kind: "blocker"` DQ (`from: "bm"`).
- If trunk is NOT clean or NOT synced with origin → STOP and raise a `kind: "blocker"` DQ; do NOT force, do NOT stash, do NOT proceed.
- If the plan file `.claude/PRPs/plans/v1-ship-1-r1.plan.md` is absent on `governance-v0` HEAD → STOP and raise a `kind: "blocker"` DQ (the plan landed at `1e2171310`; absence means trunk drifted — surface, do not improvise). Do NOT substitute the parked `v1-ship-1.plan.md` — that is the older design-audit copy, not this phase's dispatchable plan.
- Push is REQUIRED for this phase branch (per autonomy table — auto, no confirm). Do NOT skip the push (the laptop needs `origin/phase-v1-ship-1` to create the lane worktree).
- Do NOT open a PR (that is a later `bm-pr` task, post-impl). bm-cut is branch-creation + runlog only.
- Mid-task DQ writes (if any blocker): per the KNOWN harness limitation below, `.claude/decision-queue.json` writes from the Junior worktree may be blocked by the CC v2.1.119 `.claude/**` sensitive-file gate (advisor task #10). If a `kind: "blocker"` DQ write is denied: write the blocker content to `<worktree-root>/v1-ship-1-bm-cut-BLOCKER.md` instead (worktree root is writable) and STOP with a clear escalation message naming that file — the advisor will relocate + action it (same Option-B relocate pattern proven for Junior #270). Do NOT silently proceed past a blocker because the DQ write failed.
- Attribution: any DQ entry (or escalation file) is `from: "bm"`, `answered_by: null` or `"bm-self-resolved"`. NEVER `"advisor"` / `"user"` / `"planner"`.

## §5 Concurrency note

Three CC advisor sessions run concurrently (this `v1-ship-1` lane + `v1-federation-inbound-a` + `v1-AD-e`). bm-cut here only creates a NEW branch off `governance-v0` and a NEW runlog file — zero overlap with the other lanes' phase branches or files. Per `.claude/rules/multi-lane-worktree.md`: after this bm-cut pushes `phase-v1-ship-1`, the user/advisor creates the lane-dedicated worktree `C:/Users/barri/Developer/brehon-fork-ship-1` off `origin/phase-v1-ship-1`; the lane's subsequent advisor session runs from THAT worktree (the canonical `brehon-fork` checkout stays meta-edit-only). bm-cut itself runs as a normal Junior task on the EliteDesk daemon (branches from `/srv/brehon-fork` `governance-v0` HEAD) — the daemon's main checkout may currently be on another lane's phase branch; that is irrelevant because the BM Junior task creates its own isolated worktree from `governance-v0`.

## §6 KNOWN harness limitation (advisor task #10 — read before any DQ write)

Claude Code v2.1.119 enforces a hardcoded `.claude/**` sensitive-file gate that is NOT overridden by `--dangerously-skip-permissions` (the Junior worker runs in `bypassPermissions` mode but the gate still fires) NOR by `settings.json permissions.allow`. **Consequence for this task:** bm-cut's primary deliverable is the branch + `.claude/runlog/v1-ship-1-runlog.md`. The runlog write to `.claude/runlog/` MAY be blocked by this gate. If `Write`/`Edit` to `.claude/runlog/v1-ship-1-runlog.md` is denied with a "sensitive file" error:

1. Write the runlog content to `<worktree-root>/v1-ship-1-runlog.md` (worktree root is writable — proven for Junior #270).
2. Complete Phase 3 (branch create + push) normally — branch creation is git, not a `.claude/**` write, so it is UNAFFECTED.
3. In the Phase 5 summary, state explicitly: "runlog written to worktree root `v1-ship-1-runlog.md` (not `.claude/runlog/`) due to CC v2.1.119 sensitive-file gate per advisor task #10; advisor must relocate."
4. The Junior finalize stage will commit + merge whatever is in the worktree (including the root-misplaced runlog); the advisor then relocates `v1-ship-1-runlog.md` → `.claude/runlog/v1-ship-1-runlog.md` on `governance-v0` (Option-B relocate pattern, same as the plan-file relocation already done for #270).

This is the EXPECTED path until the advisor installs the scoped PreToolUse hook (deferred until the user is at the laptop terminal). Do NOT treat the runlog-write block as a task failure — the branch + push are the load-bearing deliverables and are unaffected; the runlog is recoverable via relocate.
