---
role: bm-task
verb: bm-cut
phase: v1-quality-r3c
pr_number: null
created: 2026-05-31
related_dq: null
---

# BM-cut brief — v1-quality-r3c

**Role:** `[role:bm-task]`
**Phase:** `v1-quality-r3c`
**Authored:** 2026-05-31
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base:** `governance-v0` HEAD at task-execution time (`07cada651` or later)
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". No laptop-side phase worktree; advisor drives via Junior dispatch from canonical `brehon-fork`.

---

## 1. Role + dispatch line

```
[role:bm-task] v1-quality-r3c bm-cut — see .claude/PRPs/briefs/v1-quality-r3c-bm-cut-1.md
```

---

## 2. Scope

**Produce:** phase branch `phase-v1-quality-r3c` cut from `governance-v0` HEAD, pushed to `origin`, with upstream tracking set.

**Explicit boundaries:**
- Cut branch `phase-v1-quality-r3c` from `governance-v0` at its current HEAD
- Push with `--set-upstream origin phase-v1-quality-r3c`
- Verify the plan file `.claude/PRPs/plans/v1-quality-r3c.plan.md` is present on the new branch
- Write a one-line runlog entry to `.claude/runlog/v1-quality-r3c-runlog.md` (create if absent): `bm: bm-cut complete — phase-v1-quality-r3c cut from governance-v0 @ <SHA>`
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

- **Branch name must be exactly** `phase-v1-quality-r3c` (no variant)
- **Base must be** `governance-v0` (never `main`)
- **Pre-cut checks** (per `branch-manager.md`):
  1. `git status --short` on `governance-v0` — must be clean
  2. `git fetch origin && git log governance-v0..origin/governance-v0 --oneline` — must be empty (governance-v0 in sync)
  3. Plan file `.claude/PRPs/plans/v1-quality-r3c.plan.md` must be present on `governance-v0`
- **Push with upstream tracking:** `git push -u origin phase-v1-quality-r3c`
- **Runlog:** create `.claude/runlog/v1-quality-r3c-runlog.md` if absent; append the bm-cut entry; commit on `governance-v0` with subject `chore(bm): v1-quality-r3c bm-cut complete — phase branch cut`
- **DQ mid-task discipline:** if a blocker arises, write a `kind: "blocker"` DQ entry (use `bash scripts/brehon/dq-v3-new-entry.sh` for the id), commit + push, and stop

---

## 5. Success signals

- `git ls-remote origin refs/heads/phase-v1-quality-r3c` returns non-empty (branch exists on origin)
- Plan file `.claude/PRPs/plans/v1-quality-r3c.plan.md` present on the new branch
- Runlog entry appended: `bm: bm-cut complete — phase-v1-quality-r3c cut from governance-v0 @ <SHA>`
- Runlog committed on `governance-v0` with subject `chore(bm): v1-quality-r3c bm-cut complete — phase branch cut`
- Push to origin succeeded with upstream tracking set

---

## 6. Out of scope

- Editing impl code, plan files, design docs, or any file in the never-touch list
- Opening a PR (that comes at phase close)
- Merging or rebasing anything

---

## HANDOVER

```yaml
HANDOVER:
  task: v1-quality-r3c-bm-cut
  filesCreated: [.claude/runlog/v1-quality-r3c-runlog.md]
  filesModified: []
  keyDecisions:
    - "phase-v1-quality-r3c cut from governance-v0 @ 07cada651 or later"
  notes: "bm-cut only — no PR yet"
```

Brief complete. Dispatch as:

```
[role:bm-task] v1-quality-r3c bm-cut — see .claude/PRPs/briefs/v1-quality-r3c-bm-cut-1.md
```

Base branch: `governance-v0`
