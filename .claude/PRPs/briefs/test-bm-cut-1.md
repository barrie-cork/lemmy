---
phase: test
role: bm-task
task: bm-cut
brief_n: 1
authored: 2026-06-11
plan: "(none — test dogfood sandbox; plan will be authored by planning Junior post-cut)"
plan_approved: "n/a — plan-file prereq waived by user 2026-06-11 (test sandbox)"
---

# [role:bm-task] bm-cut test — see .claude/PRPs/briefs/test-bm-cut-1.md

## §1 Role + dispatch

`[role:bm-task] bm-cut test — cut phase-test off governance-v0`

Actual create-task description (single line, <100 chars):

```
[role:bm-task] bm-cut test — see .claude/PRPs/briefs/test-bm-cut-1.md
```

## §2 Scope

Cut `phase-test` off `governance-v0` (PHASE branch — will be pushed).

- **Phase 0:** branch name `phase-test` (matches `^phase-[a-z0-9][a-z0-9-]*$` per `.claude/commands/bm/bm-cut.md` Phase 0).
- **Phase 1:** verify trunk (`governance-v0`) clean + synced with origin.
- **Phase 2 (OVERRIDE):** this is a sandbox dogfood phase. No plan file exists at `.claude/PRPs/plans/test*.plan.md` — **this is expected and intentional**. Skip the plan-present check. Do NOT raise a `kind: "blocker"` DQ for the missing plan file. The plan will be authored by the planning Junior AFTER the phase branch is cut.
- **Phase 3:** `git checkout -b phase-test governance-v0`
- **Phase 4:** `git push -u origin phase-test` — push is required so the advisor can dispatch planning Junior with `base_branch=phase-test` (workers fork from this branch).
- **Phase 5:** create `.claude/runlog/test-runlog.md` with standard BM runlog header + first entry. Commit the runlog on `phase-test` and push.

## §3 Required reading

- `.claude/commands/bm/bm-cut.md` — follow step by step (Phase 2 plan check is overridden above).
- `.claude/rules/branch-manager.md` — BM file-ownership + autonomy bounds + "Phase-branch discipline".
- `.claude/rules/phase-branch.md` — phase-branch + PR discipline.

## §4 Constraints (hard rules)

- BM **NEVER** touches `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `Cargo.toml`.
- Branch name MUST be exactly `phase-test`. Any other name → STOP and raise a `kind: "blocker"` DQ (`from: "bm"`).
- If trunk is NOT clean or NOT synced with origin → STOP and raise `kind: "blocker"` DQ. Do NOT stash, force, or proceed.
- **Phase 2 plan-check OVERRIDDEN** — do NOT STOP for missing `test*.plan.md`. (See §2 Phase 2 OVERRIDE above.)
- Push is REQUIRED. Do NOT skip — the advisor needs `origin/phase-test` to dispatch the planning Junior.
- Do NOT open a PR (that is a later `bm-pr` task). bm-cut = branch + runlog only.
- Attribution: any DQ entry is `from: "bm"`, `answered_by: null` or `"bm-self-resolved"`. NEVER `"advisor"` / `"user"`.

## §5 KNOWN harness limitations

**Finalize-merge hazard (Phase 8 of bm-cut.md):** the daemon finalize agent will attempt to merge `phase-test` back into `governance-v0`. This is WRONG for a phase branch (divergence is the deliverable, not a feature branch to merge back). This is a known daemon behavior — do NOT prevent it, the advisor will detect and recover post-bm-cut via `git update-ref refs/heads/governance-v0 origin/governance-v0` if needed.

**CC sensitive-file gate:** `.claude/**` writes (runlog at `.claude/runlog/test-runlog.md`) may be blocked. If `Write` to `.claude/runlog/test-runlog.md` is denied: write runlog content to `<worktree-root>/test-runlog.md` instead, complete Phase 4 (branch + push) normally, and state explicitly in output that the runlog was written to the worktree root due to the gate. The advisor will relocate.
