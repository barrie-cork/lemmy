# BM-cut brief — v1-quality-r2

**Role:** `[role:bm-task]`
**Phase:** `v1-quality-r2`
**Authored:** 2026-05-28
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base:** `governance-v0` HEAD at task-execution time (current tip when bm-task runs)
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". No laptop-side phase worktree; advisor drives via Junior dispatch from canonical `brehon-fork`.

---

## 1. Role + dispatch line

```
[role:bm-task] v1-quality-r2 bm-cut — see .claude/PRPs/briefs/v1-quality-r2-bm-cut-1.md
```

---

## 2. Scope

### 2.1 What to produce

1. ff daemon-local `governance-v0` to `origin/governance-v0` via `git fetch origin governance-v0:governance-v0` (lane-safe per `feedback_daemon_local_trunk_stale_multi_lane.md`).
2. Verify ff'd HEAD is clean: `git status --short` MUST be empty.
3. Create branch `phase-v1-quality-r2` off `governance-v0` HEAD (post-ff).
4. Push `phase-v1-quality-r2` to `origin` with `git push -u origin phase-v1-quality-r2` (upstream tracking).
5. Verify the branch exists on origin via `gh api repos/barrie-cork/lemmy/branches/phase-v1-quality-r2 --jq '.name + " @ " + .commit.sha[0:9]'`.
6. Append one line to `.claude/runlog/bm-runlog.md` on `governance-v0`:
   ```
   ## bm-cut: phase-v1-quality-r2 off <SHA> — 2026-05-28
   ```
   where `<SHA>` is the short SHA of the cut base after ff.
7. Commit + push the runlog append to `governance-v0`. Commit subject: `chore(bm-task): log branch cut phase-v1-quality-r2 — PR #155 carry-forward bundle`.

### 2.2 Scope boundary

**IN scope:**
- `git fetch origin governance-v0:governance-v0`
- `git checkout -b phase-v1-quality-r2 governance-v0`
- `git push -u origin phase-v1-quality-r2`
- `.claude/runlog/bm-runlog.md` one-line append on `governance-v0`

**OUT of scope:**
- Any edit to `crates/**`, `migrations/**`, `tests/**`, `docs/**`
- Any PR creation (`bm-pr` is a later verb, after planning + impl complete)
- Any DQ entries unless a refusal case triggers (see §4)
- Any worktree creation on the laptop (Lane is Mode B — no laptop worktree by design)
- Any plan file authorship (`.claude/PRPs/plans/v1-quality-r2.plan.md` will be created by the planning Junior task on `phase-v1-quality-r2`, NOT on `governance-v0`)

### 2.3 Plan-file precondition is intentionally absent

Per `.claude/commands/bm/bm-cut.md` Phase 2: "A phase branch needs the corresponding plan file on trunk." This brief **explicitly exempts** v1-quality-r2 from that precondition because:

- The planning brief at `.claude/PRPs/briefs/v1-quality-r2-planning-1.md` is committed on `governance-v0`. The planning Junior worker forks from `phase-v1-quality-r2` after bm-cut and authors `.claude/PRPs/plans/v1-quality-r2.plan.md` on the phase branch.
- This is the Mode B convention (per `multi-lane-worktree.md` §"Brief location and trunk→phase sync"): planning brief on trunk + planning Junior on phase branch + plan file shipped on phase branch.
- The bm-cut.md Phase 2 paragraph is canonical for plans authored ahead of time; v1-quality-r2 authors plan after bm-cut.

Same precedent applies as Lane A: `.claude/PRPs/briefs/v1-redaction-r1-bm-cut-1.md` cut without a plan-file precondition.

---

## 3. Required reading

- `.claude/commands/bm/bm-cut.md` — canonical bm-cut procedure (read Phases 0-8 entirely)
- `.claude/rules/branch-manager.md` — BM file-ownership boundaries + autonomy bounds
- `.claude/rules/phase-branch.md` — phase-branch discipline (cut from `governance-v0`, never `main`)
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every `gh` command (forks default to upstream)
- `.claude/rules/multi-lane-worktree.md` §"Lane modes" + §"Brief location and trunk→phase sync" (background only; not load-bearing for bm-cut itself)
- `.claude/lessons/feedback_daemon_local_trunk_stale_multi_lane.md` — **MANDATORY:** ff daemon-local `governance-v0` to origin BEFORE cut; Task #482 first attempt failed this and exited as no-op
- `.claude/lessons/feedback_junior_finalize_merges_bm_cut_branch.md` — Phase 8 hazard (post-task finalize MUST NOT merge `phase-v1-quality-r2` back into daemon-local `governance-v0`)
- `.claude/lessons/feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` — runlog gate-block + advisor-relocate recovery

---

## 4. Constraints

- **PRE-CUT verification:** before cut, run `git fetch origin governance-v0:governance-v0` to ff the daemon-local trunk to origin (Task #482 first attempt: worker forked from stale daemon-local `governance-v0` @ `e5ee569bb` (3 commits behind origin) — root cause per `feedback_daemon_local_trunk_stale_multi_lane.md`). Then `git checkout governance-v0 && git status --short` MUST be empty AND `git rev-parse HEAD` MUST equal `git rev-parse origin/governance-v0` after the ff. Cut from the resulting HEAD (whatever its SHA). No SHA pin is asserted here because rebases across concurrent advisor sessions move the actual cut base.
- **Branch name:** exactly `phase-v1-quality-r2`. No deviation; no `phase-quality-r2` shorthand; no `v1-r2-quality` reordering.
- **Base MUST be `governance-v0`** (never `main` per `phase-branch.md`).
- **Push target:** `origin` (barrie-cork/lemmy). Use `--repo barrie-cork/lemmy` on any `gh` call.
- **Runlog append:** one line to `.claude/runlog/bm-runlog.md` on `governance-v0` (the GLOBAL BM runlog, NOT `.claude/runlog/v1-quality-r2-runlog.md`). Same convention as Lane A (`v1-redaction-r1-bm-cut-1.md`).
- **Attribution:** any DQ entry MUST set `from: "bm"`. **NEVER write `answered_by: "advisor"` or `answered_by: "user"`** per `.claude/rules/decision-queue.md` hard refusal #1. Self-resolve as `bm-self-resolved` only.
- **Mid-task push discipline:** if a DQ refusal fires, write the entry + commit + push the DQ in one atomic sequence (per `decision-queue.md` §"Mid-task visibility").
- **DO NOT** trigger the finalize-merge hazard described in bm-cut.md Phase 8. After the runlog commit + push completes, the bm-task should EXIT cleanly. The Junior daemon's generic post-job finalize agent may attempt to merge `phase-v1-quality-r2` back into daemon-local `governance-v0` — this is a known harness limitation, advisor will detect and recover per Phase 8 procedure.
- **DO NOT** create a laptop-side worktree. Mode B is by design.
- **DO NOT** author the plan file. That's the planning Junior's job, dispatched on `phase-v1-quality-r2` AFTER this bm-cut completes.

---

## 5. Context

- **Phase:** v1-quality-r2 (PR #155 carry-forward bundle — quality/cleanup sweep for issues #156, #157, #158, #159, #160)
- **Lane:** Q (second concurrent active lane; Lane A redaction is in-flight at `phase-v1-redaction-r1`)
- **Planning brief on trunk:** `.claude/PRPs/briefs/v1-quality-r2-planning-1.md` (committed in the same advisor commit as this bm-cut brief, OR in a prior advisor commit)
- **User authorization for this lane:** 2026-05-28 explicit ("Schedule the other postponed GT issue fixes too, if safe to do so" → user selected v1-quality-r2 via AskUserQuestion)
- **Concurrent lane activity:**
  - `phase-v1-redaction-r1` Lane A — IN-FLIGHT (planning Junior dispatched as Task #485). Zero file overlap with this lane (Lane A = `crates/db_schema/src/source/governance/redaction.rs`; this lane = `crates/server/tests/e2e.rs` + `crates/api/api/src/governance/{admin_emergency_remove,submit_jury_vote}.rs` + scripts + DQ JSON).
  - Daemon `.git/index.lock` contention: 2 concurrent lanes ≪ size-3 threshold per `feedback_cohort_shared_git_index_contention.md`. Safe.
  - Laptop e2e gate serialization: both lanes will eventually need laptop e2e (~26 min each). Stagger dispatches.
- **Hard precondition satisfied:** PR #155 (v1-RT-r3) merged via `5ebd8ae23`; all RT-r3 symbols this lane references (`EnvVarGuard`, `v1_rt_r3_fixtures`, `emit_reputation_event_local`) are on `governance-v0`.

---

## 6. After this bm-task completes

Advisor will:

1. Verify daemon worktree state post-bm-cut per bm-cut.md Phase 8 (`ssh homeserver 'cd /srv/brehon-fork && git symbolic-ref HEAD'` should return `refs/heads/phase-v1-quality-r2`).
2. Trunk→phase sync the planning brief so it's visible on `phase-v1-quality-r2` for the planning Junior to read. Per `multi-lane-worktree.md` §"Brief location and trunk→phase sync" Mode B procedure (SSH-merge from daemon's main worktree, which is on the phase branch post-bm-cut).
3. Dispatch the planning Junior with `base_branch=phase-v1-quality-r2` per `feedback_handover_assumptions_need_empirical_verification.md` discipline.

None of those steps are bm-task scope. The bm-task EXITS after the runlog commit + push.
