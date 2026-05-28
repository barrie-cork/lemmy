# BM-cut brief — v1-redaction-r1

**Role:** `[role:bm-task]`
**Phase:** `v1-redaction-r1`
**Authored:** 2026-05-28
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base:** `governance-v0` @ `ec490cace`
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". No laptop-side phase worktree; advisor drives via Junior dispatch from canonical `brehon-fork`.

---

## 1. Role + dispatch line

```
[role:bm-task] v1-redaction-r1 bm-cut — see .claude/PRPs/briefs/v1-redaction-r1-bm-cut-1.md
```

---

## 2. Scope

### 2.1 What to produce

1. Create branch `phase-v1-redaction-r1` off `governance-v0` HEAD (`ec490cace`).
2. Push `phase-v1-redaction-r1` to `origin` with `git push -u origin phase-v1-redaction-r1` (upstream tracking).
3. Verify the branch exists on origin via `gh api repos/barrie-cork/lemmy/branches/phase-v1-redaction-r1 --jq '.name + " @ " + .commit.sha[0:9]'`.
4. Append one line to `.claude/runlog/bm-runlog.md`:
   ```
   ## bm-cut: phase-v1-redaction-r1 off ec490cace — 2026-05-28
   ```
5. Commit + push the runlog append to `governance-v0`. Commit subject: `chore(bm-task): log branch cut phase-v1-redaction-r1 post-triage authorisation`.

### 2.2 Scope boundary

**IN scope:**
- `git checkout -b phase-v1-redaction-r1 governance-v0`
- `git push -u origin phase-v1-redaction-r1`
- `.claude/runlog/bm-runlog.md` one-line append on `governance-v0`

**OUT of scope:**
- Any edit to `crates/**`, `migrations/**`, `tests/**`, `docs/**`
- Any PR creation (`bm-pr` is a later verb, after planning + impl complete)
- Any DQ entries unless a refusal case triggers (see §4)
- Any worktree creation on the laptop (Lane is Mode B — no laptop worktree by design)
- Any plan file authorship (`.claude/PRPs/plans/v1-redaction-r1.plan.md` will be created by the planning Junior task on `phase-v1-redaction-r1`, NOT on `governance-v0`)

### 2.3 Plan-file precondition is intentionally absent

Per `.claude/commands/bm/bm-cut.md` Phase 2: "A phase branch needs the corresponding plan file on trunk." This brief **explicitly exempts** Lane A from that precondition because:

- The planning brief at `.claude/PRPs/briefs/v1-redaction-r1-planning-1.md` is committed on `governance-v0` at `ec490cace`. The planning Junior worker forks from `phase-v1-redaction-r1` after bm-cut and authors `.claude/PRPs/plans/v1-redaction-r1.plan.md` on the phase branch.
- This is the Mode B convention (per `multi-lane-worktree.md` §"Brief location and trunk→phase sync"): planning brief on trunk + planning Junior on phase branch + plan file shipped on phase branch.
- The bm-cut.md Phase 2 paragraph is canonical for plans authored ahead of time; Lane A authors plan after bm-cut.

If the BM worker is uncertain about this exemption, refer to the precedent at `.claude/PRPs/briefs/v1-federation-inbound-e-bm-cut-1.md` which also cut without a plan-file precondition.

---

## 3. Required reading

- `.claude/commands/bm/bm-cut.md` — canonical bm-cut procedure (read Phases 0-8 entirely)
- `.claude/rules/branch-manager.md` — BM file-ownership boundaries + autonomy bounds
- `.claude/rules/phase-branch.md` — phase-branch discipline (cut from `governance-v0`, never `main`)
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every `gh` command (forks default to upstream)
- `.claude/rules/multi-lane-worktree.md` §"Lane modes" + §"Brief location and trunk→phase sync" (background only; not load-bearing for bm-cut itself)
- `.claude/lessons/feedback_junior_finalize_merges_bm_cut_branch.md` — Phase 8 hazard (post-task finalize MUST NOT merge `phase-v1-redaction-r1` back into daemon-local `governance-v0`)
- `.claude/lessons/feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` — runlog gate-block + advisor-relocate recovery

---

## 4. Constraints

- **PRE-CUT verification:** `git rev-parse --short governance-v0` MUST equal `ec490cace` before cut. If drift (e.g. concurrent advisor commit landed), STOP and file `kind: "blocker"` DQ.
- **PRE-CUT trunk cleanliness:** `git status --short` MUST be empty on `governance-v0` before cut. If non-empty, STOP and surface.
- **Branch name:** exactly `phase-v1-redaction-r1`. No deviation; no `phase-redaction-r1` shorthand; no `v1-r1-redaction` reordering.
- **Base MUST be `governance-v0`** (never `main` per `phase-branch.md`).
- **Push target:** `origin` (barrie-cork/lemmy). Use `--repo barrie-cork/lemmy` on any `gh` call.
- **Runlog append:** one line to `.claude/runlog/bm-runlog.md` on `governance-v0`. Per bm-cut.md Phase 5, the runlog commit is the first commit on the new branch.

  **IMPORTANT runlog clarification:** the canonical bm-cut.md Phase 5 says "Create or append to `.claude/runlog/<phase>-runlog.md`" (phase-specific). However, the precedent `.claude/PRPs/briefs/v1-federation-inbound-e-bm-cut-1.md` writes to `.claude/runlog/bm-runlog.md` (the global BM runlog). This brief follows the **global bm-runlog precedent** because (a) Lane A is Mode B (no laptop worktree to host a phase-specific runlog edit), (b) the global runlog is cross-lane visible, and (c) the canonical bm-cut.md command also says "Append to runlog" without disambiguating which file when both exist. Write to `.claude/runlog/bm-runlog.md` on `governance-v0`.
- **Attribution:** any DQ entry MUST set `from: "bm"`. **NEVER write `answered_by: "advisor"` or `answered_by: "user"`** per `.claude/rules/decision-queue.md` hard refusal #1. Self-resolve as `bm-self-resolved` only.
- **Mid-task push discipline:** if a DQ refusal fires, write the entry + commit + push the DQ in one atomic sequence (per `decision-queue.md` §"Mid-task visibility").
- **DO NOT** trigger the finalize-merge hazard described in bm-cut.md Phase 8. After the runlog commit + push completes, the bm-task should EXIT cleanly. The Junior daemon's generic post-job finalize agent may attempt to merge `phase-v1-redaction-r1` back into daemon-local `governance-v0` — this is a known harness limitation, advisor will detect and recover per Phase 8 procedure.
- **DO NOT** create a laptop-side worktree. Mode B is by design.
- **DO NOT** author the plan file. That's the planning Junior's job, dispatched on `phase-v1-redaction-r1` AFTER this bm-cut completes.

---

## 5. Context

- **Phase:** v1-redaction-r1 (P0 GDPR-critical redaction hardening — issue #58)
- **Lane:** A of 2 in the 2026-05-28 open-issue triage (Lane B closed as already-shipped per session work)
- **Planning brief on trunk:** `.claude/PRPs/briefs/v1-redaction-r1-planning-1.md` @ `ec490cace`
- **Conformance audit:** `.claude/PRPs/reports/conformance-audit-redaction-rs-2026-05-28.md` @ `ec490cace` (read by planning Junior, NOT by bm-task)
- **User authorization for this lane:** 2026-05-28 explicit ("dispatch Lane A bm-cut")
- **Concurrent lane activity:** `phase-v1-RT-r3` is in-flight at `brehon-fork-rt-r3` worktree (Task 4 advisor-side authorship). Lane A's bm-cut is `crates/`-free + plan-free; no `[P]` cohort, no `.git/index.lock` contention concern (single bm-task ≪ size-3 threshold per `feedback_cohort_shared_git_index_contention.md`).
- **Trunk SHA at brief-author time:** `ec490cace chore(advisor): v1-redaction-r1 planning brief + conformance audit (issue #58)`. If trunk has advanced by the time this bm-task runs, the constraint at §4 will catch the drift.

---

## 6. After this bm-task completes

Advisor will:

1. Verify daemon worktree state post-bm-cut per bm-cut.md Phase 8 (`ssh homeserver 'cd /srv/brehon-fork && git symbolic-ref HEAD'` should return `refs/heads/phase-v1-redaction-r1`).
2. Trunk→phase sync the planning brief + conformance audit report so they're visible on `phase-v1-redaction-r1` for the planning Junior to read. Per `multi-lane-worktree.md` §"Brief location and trunk→phase sync" Mode B procedure (SSH-merge from daemon's main worktree, which is on the phase branch post-bm-cut).
3. Dispatch the planning Junior with `base_branch=phase-v1-redaction-r1` per `feedback_handover_assumptions_need_empirical_verification.md` discipline (verify each named assumption before queueing).

None of those steps are bm-task scope. The bm-task EXITS after the runlog commit + push.
