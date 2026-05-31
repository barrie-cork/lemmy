---
role: bm-task
verb: bm-cut
phase: v1-quality-r3b
pr_number: null
created: 2026-05-31
related_dq: null
---

# BM-cut brief — v1-quality-r3b

**Role:** `[role:bm-task]`
**Phase:** `v1-quality-r3b`
**Authored:** 2026-05-31
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base:** `governance-v0` HEAD at task-execution time (`8e3c8146f` or later)
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". No laptop-side phase worktree; advisor drives via Junior dispatch from canonical `brehon-fork`.

---

## 1. Role + dispatch line

```
[role:bm-task] v1-quality-r3b bm-cut — see .claude/PRPs/briefs/v1-quality-r3b-bm-cut-1.md
```

---

## 2. Scope

**Produce:** phase branch `phase-v1-quality-r3b` cut from `governance-v0` HEAD, pushed to `origin`, with upstream tracking set.

**Explicit boundaries:**
- Cut branch `phase-v1-quality-r3b` from `governance-v0` at its current HEAD
- Push with `--set-upstream origin phase-v1-quality-r3b`
- Verify the plan file `.claude/PRPs/plans/v1-quality-r3b.plan.md` is present on the new branch
- Write a one-line runlog entry to `.claude/runlog/v1-quality-r3b-runlog.md` (create if absent): `bm: bm-cut complete — phase-v1-quality-r3b cut from governance-v0 @ <SHA>`
- Commit the runlog entry on `governance-v0`

**Do NOT:**
- Open a PR (that comes at phase close after all impl tasks)
- Modify any file under `crates/`, `migrations/`, `docs/`, `.claude/PRPs/plans/`, `.claude/PRPs/prds/`
- Merge or rebase anything

---

## 3. Required reading

1. `.claude/commands/bm/bm-cut.md` — the canonical bm-cut procedure; follow it step by step
2. `.claude/rules/branch-manager.md` — file-ownership boundaries and pre-cut checklist
3. `.claude/rules/phase-branch.md` — branch naming convention and pre-cut verifications
4. `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every `gh` call

---

## 4. Constraints

- **Branch name must be exactly** `phase-v1-quality-r3b` (no variant)
- **Base must be** `governance-v0` (never `main`)
- **Pre-cut checks** (per `branch-manager.md`):
  1. `git status --short` on `governance-v0` — must be clean
  2. `git fetch origin && git log governance-v0..origin/governance-v0 --oneline` — must be empty (governance-v0 in sync)
  3. Plan file `.claude/PRPs/plans/v1-quality-r3b.plan.md` must be present on `governance-v0`
- **Push with upstream tracking:** `git push -u origin phase-v1-quality-r3b`
- **Runlog:** create `.claude/runlog/v1-quality-r3b-runlog.md` if absent; append the bm-cut entry; commit on `governance-v0` with subject `chore(bm): v1-quality-r3b bm-cut complete — phase branch cut`
- **DQ mid-task discipline:** if a blocker arises, write a `kind: "blocker"` DQ entry (use `bash scripts/brehon/dq-v3-new-entry.sh` for the id), commit + push, and stop
