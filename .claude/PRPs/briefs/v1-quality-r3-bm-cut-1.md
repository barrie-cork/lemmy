---
role: bm-task
verb: bm-cut
phase: v1-quality-r3
pr_number: null
created: 2026-05-30
related_dq: null
---

# BM-cut brief — v1-quality-r3

**Role:** `[role:bm-task]`
**Phase:** `v1-quality-r3`
**Authored:** 2026-05-30
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base:** `governance-v0` HEAD at task-execution time (current tip `fa7bc3b0c` or later)
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". No laptop-side phase worktree; advisor drives via Junior dispatch from canonical `brehon-fork`.

---

## 1. Role + dispatch line

```
[role:bm-task] v1-quality-r3 bm-cut — see .claude/PRPs/briefs/v1-quality-r3-bm-cut-1.md
```

---

## 2. Scope

### 2.1 What to produce

1. ff daemon-local `governance-v0` to `origin/governance-v0` via `git fetch origin governance-v0:governance-v0` (lane-safe per `feedback_daemon_local_trunk_stale_multi_lane.md`).
2. Verify ff'd HEAD is clean: `git status --short` MUST be empty.
3. Create branch `phase-v1-quality-r3` off `governance-v0` HEAD (post-ff).
4. Push `phase-v1-quality-r3` to `origin` with `git push -u origin phase-v1-quality-r3` (upstream tracking).
5. Verify the branch exists on origin via `gh api repos/barrie-cork/lemmy/branches/phase-v1-quality-r3 --jq '.name + " @ " + .commit.sha[0:9]'`.
6. Append one line to `.claude/runlog/bm-runlog.md` on `governance-v0`:
   ```
   ## bm-cut: phase-v1-quality-r3 off <SHA> — 2026-05-30
   ```
   where `<SHA>` is the short SHA of the cut base after ff.
7. Commit + push the runlog append to `governance-v0`. Commit subject: `chore(bm-task): log branch cut phase-v1-quality-r3 — EnvVarGuard C4 sweep`.

### 2.2 Scope boundary

**IN scope:**
- `git fetch origin governance-v0:governance-v0` (or `git pull --ff-only` if checked out on governance-v0)
- `git checkout -b phase-v1-quality-r3 governance-v0`
- `git push -u origin phase-v1-quality-r3`
- `.claude/runlog/bm-runlog.md` one-line append on `governance-v0`

**OUT of scope:**
- Any edit to `crates/**`, `migrations/**`, `tests/**`, `docs/**`
- Any PR creation (`bm-pr` is a later verb, after impl complete)
- Any DQ entries unless a refusal case triggers (see §4)
- Any worktree creation on the laptop (Lane is Mode B — no laptop worktree by design)

### 2.3 Plan-file precondition — SATISFIED

The plan already exists on `governance-v0` at `fa7bc3b0c`: `.claude/PRPs/plans/v1-quality-r3.plan.md` (revised to e2e-sweep-only scope per split-DQ `630fc36b795c-001`). No planning Junior needed — bm-cut is the next action.

---

## 3. Required reading

- `.claude/commands/bm/bm-cut.md` — canonical bm-cut procedure (read Phases 0-8 entirely)
- `.claude/rules/branch-manager.md` — BM file-ownership boundaries + autonomy bounds
- `.claude/rules/phase-branch.md` — phase-branch discipline (cut from `governance-v0`, never `main`)
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every `gh` command
- `.claude/lessons/feedback_daemon_local_trunk_stale_multi_lane.md` — MANDATORY: ff daemon-local `governance-v0` to origin BEFORE cut
- `.claude/lessons/feedback_junior_finalize_merges_bm_cut_branch.md` — Phase 8 hazard (finalize MUST NOT merge `phase-v1-quality-r3` back into daemon-local `governance-v0`)
- `.claude/lessons/feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` — runlog gate-block + advisor-relocate recovery

---

## 4. Constraints

- **PRE-CUT verification:** before cut, ff daemon-local `governance-v0` to origin (refspec-fetch if on another branch; `git pull --ff-only` if checked out on `governance-v0`). Then `git status --short` MUST be empty AND `git rev-parse HEAD` MUST equal `git rev-parse origin/governance-v0`. Cut from the resulting HEAD.
- **Branch name:** exactly `phase-v1-quality-r3`. No deviation.
- **Base MUST be `governance-v0`** (never `main` per `phase-branch.md`).
- **Push target:** `origin` (barrie-cork/lemmy). Use `--repo barrie-cork/lemmy` on any `gh` call.
- **Runlog append:** one line to `.claude/runlog/bm-runlog.md` on `governance-v0` (the GLOBAL BM runlog).
- **Attribution:** any DQ entry MUST set `from: "bm"`. NEVER write `answered_by: "advisor"` or `answered_by: "user"`.
- **DO NOT** trigger the finalize-merge hazard (bm-cut.md Phase 8). EXIT cleanly after runlog commit + push.
- **DO NOT** create a laptop-side worktree. Mode B is by design.

---

## 5. Context

- **Phase:** v1-quality-r3 — EnvVarGuard C4 follow-on sweep (e2e.rs only, 23 raw set_var sites for LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS + GOVERNANCE_LOG_SIGNING_KEY). Split from original combined plan per user decision 2026-05-30.
- **Plan on trunk:** `.claude/PRPs/plans/v1-quality-r3.plan.md` at `fa7bc3b0c` (revised, score 7, T1+T2+retro only).
- **Issue #167** deferred to `v1-quality-r3b` (separate phase, begins post-r3 merge).
- **No concurrent active lanes** at bm-cut time (confirmed via `git worktree list` — canonical only).

---

## 6. After this bm-task completes

Advisor will:

1. Verify daemon worktree state post-bm-cut (`ssh homeserver 'cd /srv/brehon-fork && git symbolic-ref HEAD'` should return `refs/heads/phase-v1-quality-r3`).
2. Trunk→phase sync the plan + impl-task briefs via Mode B SSH-merge of `governance-v0` into `phase-v1-quality-r3` from the daemon's main worktree (per `multi-lane-worktree.md` §"Brief location and trunk→phase sync").
3. Author T1 / T2 impl-task briefs (with verbatim e2e.rs anchors per R11 from the actual phase-branch tip), commit to `governance-v0`, trunk→phase sync each before dispatch.
4. Dispatch **Task 0 (pre-flight harness audit)** with `base_branch=phase-v1-quality-r3`, then serial T1→T2 under validate-pending-laptop, then T3 retro.

None of those steps are bm-task scope. EXIT after the runlog commit + push.
