---
role: bm-task
verb: bm-cut
phase: m2-late-b-actor
pr_number: null
related_dq: null
created: 2026-06-13
---

# BM-cut brief — m2-late-b-actor (Phase 7 — portable actor-ID linkage)

**Role:** `[role:bm-task]`
**Phase:** `m2-late-b-actor` (branch `phase-m2-late-b-actor`)
**Authored:** 2026-06-13
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base:** `governance-v0` HEAD at task-execution time (`c74400740` or later)
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". No laptop-side phase worktree; advisor drives via Junior dispatch from canonical `brehon-fork`.

> **Context:** m2-late-b-actor implements ADR-016 component 3 (B-actor portable
> actor-ID linkage) — the deferred Phase 7. The 2026-06-07 out-of-scope decision
> was reversed by the user 2026-06-13, and the gating OQ-ADR016-03 was resolved
> the same day (lean adopted as-written, commit `c74400740`).
> **The governing plan `.claude/PRPs/plans/m2-late-b-actor.plan.md` ALREADY EXISTS**
> (authored this session, on `governance-v0` at `c74400740`) — it is the bm-cut
> justification. The normal plan-presence check APPLIES (do not relax it). The plan
> must be reachable on `governance-v0` at task-execution time.

---

## 1. Role + dispatch line

```
[role:bm-task] m2-late-b-actor bm-cut — see .claude/PRPs/briefs/m2-late-b-actor-bm-cut-1.md
```

You are the **bm-task** subagent (Haiku 4.5 — git/yq/gh ops, no heavy reasoning). Execute the branch-manager verb `bm-cut` per `.claude/commands/bm/bm-cut.md`, step by step.

---

## 2. Scope

**Produce:** phase branch `phase-m2-late-b-actor` cut from `governance-v0` HEAD, pushed to `origin`, with upstream tracking set.

- **Phase:** `m2-late-b-actor` (branch `phase-m2-late-b-actor`)
- **PR number:** none (bm-cut — PR doesn't exist yet)
- **Trunk SHA:** `c74400740` on `governance-v0` (or later — base is HEAD at execution time)

**Explicit boundaries:**
- Cut branch `phase-m2-late-b-actor` from `governance-v0` at its current HEAD
- Push with `git push -u origin phase-m2-late-b-actor`
- **Plan-presence check APPLIES (normal):** the governing plan `.claude/PRPs/plans/m2-late-b-actor.plan.md` IS present on `governance-v0` (commit `c74400740`). Verify it is reachable: `git cat-file -e governance-v0:.claude/PRPs/plans/m2-late-b-actor.plan.md`. If absent, STOP and file a DQ.
- Write a one-line runlog entry to `.claude/runlog/m2-late-b-actor-runlog.md` (create if absent): `bm: bm-cut complete — phase-m2-late-b-actor cut from governance-v0 @ <SHA>`
- Commit + push the runlog entry on the new branch (per bm-cut.md Phase 5)

**Do NOT:**
- Open a PR (that comes at phase close after all impl tasks)
- Modify any file under `crates/`, `migrations/`, `docs/`, `services/bridge/`, `.claude/PRPs/plans/`, `.claude/PRPs/prds/`
- Merge or rebase anything
- Author or edit the plan (it already exists)

---

## 3. Required reading

1. `.claude/commands/bm/bm-cut.md` — the canonical bm-cut procedure; follow it step by step (note **Phase 8** finalize hazard)
2. `.claude/rules/branch-manager.md` — file-ownership boundaries and pre-cut checklist
3. `.claude/rules/phase-branch.md` — branch naming convention and pre-cut verifications
4. `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every `gh` call

---

## 4. Constraints

- **Branch name must be exactly** `phase-m2-late-b-actor` (no variant)
- **Base must be** `governance-v0` (never `main`)
- **Pre-cut checks** (per `branch-manager.md`):
  1. `git status --short` on `governance-v0` — must be clean (pre-existing pilot-internal WIP is unrelated; do NOT stage or touch it)
  2. `git fetch origin && git log governance-v0..origin/governance-v0 --oneline` — must be empty (governance-v0 in sync)
  3. **Plan-presence check APPLIES:** `git cat-file -e governance-v0:.claude/PRPs/plans/m2-late-b-actor.plan.md` must succeed
- **Push with upstream tracking:** `git push -u origin phase-m2-late-b-actor`
- **Runlog:** create `.claude/runlog/m2-late-b-actor-runlog.md` if absent; append the bm-cut entry; commit + push on the **new branch** `phase-m2-late-b-actor` (per bm-cut.md Phase 5, the runlog commit is the first commit on the new branch)
- **DQ mid-task discipline:** if a blocker arises, write a `kind: "blocker"` DQ entry (use `bash scripts/brehon/dq-v3-new-entry.sh` for the id), commit + push, and stop

---

## 5. Success signals

- `git ls-remote origin refs/heads/phase-m2-late-b-actor` returns non-empty (branch exists on origin)
- `gh api repos/barrie-cork/lemmy/branches/phase-m2-late-b-actor --jq '.name + " @ " + .commit.sha[0:9]'` confirms the branch
- Runlog entry appended + committed + pushed on `phase-m2-late-b-actor`
- Push to origin succeeded with upstream tracking set

---

## 6. Out of scope

- Editing impl code, plan files, design docs, `services/bridge/**`, or any file in the never-touch list
- Opening a PR (that comes at phase close)
- Merging or rebasing anything
- Authoring or editing the plan (it already exists at `c74400740`)

---

## 7. KNOWN harness limitation — bm-cut creates divergence (do NOT merge back)

**bm-cut creates a divergence point, NOT a feature branch.** `phase-m2-late-b-actor` must **NEVER** be merged back into `governance-v0` by the finalize agent. The daemon's generic post-job finalize may wrongly run `git merge --no-ff phase-m2-late-b-actor` INTO daemon-local `governance-v0`. Confirmed 2× (v1-AD-e #282 + v1-ship-1).

- **You (the bm-task worker):** just cut + push the branch + push the runlog. Do NOT merge anything. If the runlog write is blocked by the CC `.claude/**` sensitive-file gate, stop and report.
- **The advisor (post-task):** will verify daemon-local trunk as a ROUTINE step (`ssh homeserver 'cd /srv/brehon-fork && git log governance-v0 --oneline -1'`) and recover via `git update-ref` + push if a spurious merge landed.

---

## HANDOVER

```yaml
HANDOVER:
  task: m2-late-b-actor-bm-cut
  filesCreated: [.claude/runlog/m2-late-b-actor-runlog.md]
  filesModified: []
  keyDecisions:
    - "phase-m2-late-b-actor cut from governance-v0 @ c74400740 or later"
    - "Plan ALREADY EXISTS (.claude/PRPs/plans/m2-late-b-actor.plan.md @ c74400740) — normal plan-presence check applies, do NOT relax"
    - "OQ-ADR016-03 resolved 2026-06-13 (lean adopted); Phase-7-out-of-scope decision reversed"
    - "13 tasks across db_schema + api + api_common + routes + services/bridge + e2e; HIGH complexity"
    - "Pre-Shape-G (cargo on laptop via validate-pending-laptop DQ); services/bridge workspace-excluded, Linux-only via cargo-linux.sh"
  notes: "bm-cut only — no PR yet. Mode B lane (no laptop worktree)."
```

Brief complete. Dispatch as:

```
[role:bm-task] m2-late-b-actor bm-cut — see .claude/PRPs/briefs/m2-late-b-actor-bm-cut-1.md
```

Base branch: `governance-v0`
</content>
