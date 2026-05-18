---
phase: v1-AD-e
role: bm-task
task: bm-cut
brief_n: 1
authored: 2026-05-16
plan: .claude/PRPs/plans/v1-admin-dashboard-e.plan.md
plan_approved: "user gate 1 — 2026-05-16 (DoD smoke 4/4 PASS local: check 2m42s / clippy 3m13s / test-no-run 2m45s / e2e 89 passed 0 failed 5 ignored 34.7min; watchpoint-specificity PASS; DQ #237=(a) Dashboard+Audit-only + DQ #238=(a) maud user-resolved at 7d8f84dfc)"
canonical_sibling: ".claude/PRPs/briefs/v1-ship-1-bm-cut-1.md (schema-first gate — this brief mirrors its structure + the multi-lane push pattern + the CC v2.1.119 sensitive-file fallback)"
---

# [role:bm-task] bm-cut v1-AD-e — see .claude/PRPs/briefs/v1-AD-e-bm-cut-1.md

## §1 Role + dispatch

`[role:bm-task] bm-cut v1-AD-e — cut phase-v1-AD-e off governance-v0`

The actual create-task description (single line, <100 chars, per `.claude/rules/advisor-orchestrator.md` Junior task description template):

```
[role:bm-task] bm-cut v1-AD-e — see .claude/PRPs/briefs/v1-AD-e-bm-cut-1.md
```

## §2 Scope

Cut `phase-v1-AD-e` off `governance-v0` (this is a PHASE branch, not a chore branch — it WILL be pushed).

- **Phase 0:** branch name `phase-v1-AD-e` (type: phase; matches `^phase-v\d+-[A-Z]+-[a-z]$` per `.claude/rules/branch-manager.md` "Phase-branch discipline" and `.claude/commands/bm/bm-cut.md` Phase 0; the plan's §5 Metadata names `phase-v1-AD-e` as the branch).
- **Phase 1:** verify trunk (`governance-v0`) clean (`git status --short` empty modulo known runtime dirs) + synced with origin (`git fetch origin && git log governance-v0..origin/governance-v0 --oneline` empty; `git log origin/governance-v0..governance-v0 --oneline` empty). The approved plan file `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md` MUST be present on trunk (it landed at commit `d466d93d7`, now an ancestor of the current `governance-v0` tip).
- **Phase 2:** plan-presence check. **The plan file is `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md` — NOT `v1-AD-e.plan.md`.** The bm-cut.md Phase 2 glob hint (`grep '<phase-suffix>'` → would look for `AD-e`) will NOT match `admin-dashboard-e`. Verify the literal path `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md` exists on `governance-v0` HEAD with `git checkout governance-v0 && ls .claude/PRPs/plans/v1-admin-dashboard-e.plan.md`. (This is a phase branch with an approved plan — do NOT skip the plan check; that skip is chore-branch-only.)
- **Phase 3:** `git checkout -b phase-v1-AD-e governance-v0` THEN `git push -u origin phase-v1-AD-e`. **NOTE — deliberate deviation from bm-cut.md Phase 3 "Do NOT push yet":** phase branches are auto-push per `.claude/rules/branch-manager.md` autonomy table row "Push a `phase-*` branch to origin (`git push -u`) — Auto — No". The push is REQUIRED here so the laptop can create the lane-dedicated worktree off `origin/phase-v1-AD-e` per `.claude/rules/multi-lane-worktree.md`. This matches the canonical sibling `v1-ship-1-bm-cut-1.md` §2 Phase 3 (the multi-lane-aware pattern supersedes the older local-only bm-cut.md text for phase branches). Pushing an empty-but-named phase branch is correct here — the lane worktree, not a PR, is the immediate consumer.
- **Phase 4:** create `.claude/runlog/v1-AD-e-runlog.md` with the standard BM runlog header + a `## bm: cut phase-v1-AD-e off governance-v0 @ <trunk-sha>` first entry (use the bm-cut.md Phase 4 markdown shape: branch / off / plan / next). Commit the runlog on the new phase branch and push.
- **Phase 5:** standard one-paragraph summary (branch created, pushed, runlog initialised, trunk SHA the branch was cut from). Use the bm-cut.md Phase 5 output block.

## §3 Required reading

- `.claude/commands/bm/bm-cut.md` — the bm-cut verb script (follow step by step; note the §2 Phase 3 push deviation above is intentional and brief-authorised).
- `.claude/rules/branch-manager.md` — BM file-ownership + autonomy bounds + "Phase-branch discipline" + "Session-start ritual".
- `.claude/rules/phase-branch.md` — phase-branch + PR-into-`governance-v0` discipline (NOT `main`).
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` (relevant later at bm-pr, not bm-cut, but read for context).
- `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md` — the approved plan this phase branch hosts (read §1 Summary + §6 Relationship + §13 task list for context; do NOT implement anything — bm-cut only creates the branch + runlog).
- `.claude/rules/multi-lane-worktree.md` — why the push is mandatory (lane worktree consumes `origin/phase-v1-AD-e`).

## §4 Constraints (hard rules — BM file-ownership)

- BM **NEVER** touches `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `Cargo.toml`. bm-cut creates ONLY the branch + `.claude/runlog/v1-AD-e-runlog.md`.
- Branch name MUST be exactly `phase-v1-AD-e`. Any other name → STOP and raise a `kind: "blocker"` DQ (`from: "bm"`).
- If trunk is NOT clean or NOT synced with origin (ahead OR behind) → STOP and raise a `kind: "blocker"` DQ; do NOT force, do NOT stash, do NOT `git pull` to "fix" an ahead-state, do NOT proceed. (A behind-state may be ff-pulled per bm-cut.md Phase 1 decision tree; an ahead-state STOPS.)
- If the plan file `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md` is absent on `governance-v0` HEAD → STOP and raise a `kind: "blocker"` DQ (the plan landed at `d466d93d7`; absence means trunk drifted — surface, do not improvise). Do NOT substitute any other plan file.
- Push is REQUIRED for this phase branch (per autonomy table — auto, no confirm). Do NOT skip the push (the laptop needs `origin/phase-v1-AD-e` to create the lane worktree). This overrides bm-cut.md Phase 3's local-only text for phase branches.
- Do NOT open a PR (that is a later `bm-pr` task, post-impl). bm-cut is branch-creation + runlog only.
- Attribution: any DQ entry (or escalation file) is `from: "bm"`, `answered_by: null` or `"bm-self-resolved"`. NEVER `"advisor"` / `"user"` / `"planner"`.

## §5 Concurrency note

Three CC advisor sessions run concurrently (this `v1-AD-e` lane + `v1-federation-inbound-a` + `v1-ship-1`). bm-cut here only creates a NEW branch off `governance-v0` and a NEW runlog file — zero overlap with the other lanes' phase branches or files. Per `.claude/rules/multi-lane-worktree.md`: after this bm-cut pushes `phase-v1-AD-e`, the user/advisor creates the lane-dedicated worktree `C:/Users/barri/Developer/brehon-fork-ad-e` off `origin/phase-v1-AD-e`; the lane's subsequent advisor session runs from THAT worktree (the canonical `brehon-fork` checkout stays meta-edit-only). bm-cut itself runs as a normal Junior task on the EliteDesk daemon (branches from `/srv/brehon-fork` `governance-v0` HEAD via its own isolated worktree) — the daemon's main checkout may currently be on another lane's phase branch; that is irrelevant because the BM Junior task creates its own isolated worktree from `governance-v0`.

> **Daemon-local trunk staleness (per `feedback_daemon_local_trunk_stale_multi_lane.md`):** the EliteDesk `/srv/brehon-fork` local `governance-v0` may lag `origin/governance-v0` if another lane's daemon task last left it behind. The BM Junior MUST `git fetch origin` and branch `phase-v1-AD-e` off the **freshest** `governance-v0` (i.e. confirm `git log governance-v0..origin/governance-v0` is empty after fetch; if the daemon-local trunk is behind origin, `git fetch origin governance-v0:governance-v0` to fast-forward the local ref WITHOUT a checkout switch — lane-safe — before `git checkout -b phase-v1-AD-e governance-v0`). The plan-approval gate's DoD smoke + DQ resolutions landed on origin at `7d8f84dfc`; the phase branch must be cut from a trunk that includes them.

## §6 KNOWN harness limitation (CC v2.1.119 — read before any DQ or runlog write)

Claude Code v2.1.119 enforces a hardcoded `.claude/**` sensitive-file gate that is NOT overridden by `--dangerously-skip-permissions` (the Junior worker runs in `bypassPermissions` mode but the gate still fires) NOR by `settings.json permissions.allow`. **Consequence for this task:** bm-cut's deliverables are the branch + `.claude/runlog/v1-AD-e-runlog.md`. The runlog write to `.claude/runlog/` MAY be blocked by this gate, as MAY a `kind: "blocker"` DQ write to `.claude/decision-queue.json`.

**If the runlog `Write`/`Edit` to `.claude/runlog/v1-AD-e-runlog.md` is denied with a "sensitive file" error:**

1. Write the runlog content to `<worktree-root>/v1-AD-e-runlog.md` (worktree root is writable — proven for Junior #270 and the v1-ship-1 bm-cut).
2. Complete Phase 3 (branch create + push) normally — branch creation + push is git, not a `.claude/**` write, so it is UNAFFECTED and is the load-bearing deliverable.
3. In the Phase 5 summary, state explicitly: "runlog written to worktree root `v1-AD-e-runlog.md` (not `.claude/runlog/`) due to CC v2.1.119 sensitive-file gate; advisor must relocate."
4. The Junior finalize stage commits + merges whatever is in the worktree (including the root-misplaced runlog); the advisor then relocates `v1-AD-e-runlog.md` → `.claude/runlog/v1-AD-e-runlog.md` on `governance-v0` (Option-B relocate pattern, same as v1-ship-1).

**If a `kind: "blocker"` DQ write to `.claude/decision-queue.json` is denied:** write the blocker content to `<worktree-root>/v1-AD-e-bm-cut-BLOCKER.md` instead and STOP with a clear escalation message naming that file — the advisor will relocate + action it (same Option-B relocate pattern). Do NOT silently proceed past a blocker because the DQ write failed.

This is the EXPECTED path until the advisor installs the scoped PreToolUse hook. Do NOT treat the runlog-write block as a task failure — the branch + push are the load-bearing deliverables and are unaffected; the runlog is recoverable via relocate.
