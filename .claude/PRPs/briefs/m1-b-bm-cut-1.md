---
role: bm-task
verb: bm-cut
phase: m1-b
pr_number: null
created: 2026-06-04
related_dq: a192dbab1de8-001
---

# BM-cut brief — m1-b (M1 Tree B)

**Role:** `[role:bm-task]`
**Phase:** `m1-b` (branch `phase-m1-b`)
**Authored:** 2026-06-04
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base:** `governance-v0` HEAD at task-execution time (`d7b913578` or later)
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". No laptop-side phase worktree; advisor drives via Junior dispatch from canonical `brehon-fork`. impl-task briefs author on `governance-v0` + trunk→phase sync.

> **Context:** M1 (chat infrastructure) was split into two phase branches per split-DQ `a192dbab1de8-001`. This phase `m1-b` is **Tree B only** (in-workspace governance-messaging-config + admin handler + identity-policy validator + read-only bridge notify wiring + e2e — plan §13 Tasks 0–7). Tree A (`services/bridge/` greenfield daemon, Tasks 8–13) ships later as `phase-m1-a`. Tree C docs (Task 14) bundles with `phase-m1-a`.

---

## 1. Role + dispatch line

```
[role:bm-task] m1-b bm-cut — see .claude/PRPs/briefs/m1-b-bm-cut-1.md
```

You are the **bm-task** subagent (Haiku 4.5 per your frontmatter — git/yq/gh ops, no heavy reasoning). Execute the branch-manager verb `bm-cut` per `.claude/commands/bm/bm-cut.md`, step by step.

---

## 2. Scope

**Produce:** phase branch `phase-m1-b` cut from `governance-v0` HEAD, pushed to `origin`, with upstream tracking set.

- **Phase:** `m1-b` (branch `phase-m1-b`)
- **PR number:** none (bm-cut — PR doesn't exist yet)
- **Trunk SHA:** `d7b913578` on `governance-v0` (or later — base is HEAD at execution time)

**Explicit boundaries:**
- Cut branch `phase-m1-b` from `governance-v0` at its current HEAD
- Push with `--set-upstream origin phase-m1-b`
- Verify the plan file `.claude/PRPs/plans/m1.plan.md` is present on the new branch
- Write a one-line runlog entry to `.claude/runlog/m1-b-runlog.md` (create if absent): `bm: bm-cut complete — phase-m1-b cut from governance-v0 @ <SHA>`
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

- **Branch name must be exactly** `phase-m1-b` (no variant)
- **Base must be** `governance-v0` (never `main`)
- **Pre-cut checks** (per `branch-manager.md`):
  1. `git status --short` on `governance-v0` — must be clean
  2. `git fetch origin && git log governance-v0..origin/governance-v0 --oneline` — must be empty (governance-v0 in sync)
  3. Plan file `.claude/PRPs/plans/m1.plan.md` must be present on `governance-v0`
- **Push with upstream tracking:** `git push -u origin phase-m1-b`
- **Runlog:** create `.claude/runlog/m1-b-runlog.md` if absent; append the bm-cut entry; commit on `governance-v0` with subject `chore(bm): m1-b bm-cut complete — phase branch cut`
- **DQ mid-task discipline:** if a blocker arises, write a `kind: "blocker"` DQ entry (use `bash scripts/brehon/dq-v3-new-entry.sh` for the id), commit + push, and stop

---

## 5. Success signals

- `git ls-remote origin refs/heads/phase-m1-b` returns non-empty (branch exists on origin)
- Plan file `.claude/PRPs/plans/m1.plan.md` present on the new branch
- Runlog entry appended: `bm: bm-cut complete — phase-m1-b cut from governance-v0 @ <SHA>`
- Runlog committed on `governance-v0` with subject `chore(bm): m1-b bm-cut complete — phase branch cut`
- Push to origin succeeded with upstream tracking set

---

## 6. Out of scope

- Editing impl code, plan files, design docs, or any file in the never-touch list
- Opening a PR (that comes at phase close)
- Merging or rebasing anything
- Tree A (`services/bridge/`) — entirely out of `m1-b`; ships as `phase-m1-a`

---

## HANDOVER

```yaml
HANDOVER:
  task: m1-b-bm-cut
  filesCreated: [.claude/runlog/m1-b-runlog.md]
  filesModified: []
  keyDecisions:
    - "phase-m1-b cut from governance-v0 @ d7b913578 or later"
    - "m1-b = M1 Tree B only (Tasks 0-7); Tree A ships as phase-m1-a"
  notes: "bm-cut only — no PR yet"
```

Brief complete. Dispatch as:

```
[role:bm-task] m1-b bm-cut — see .claude/PRPs/briefs/m1-b-bm-cut-1.md
```

Base branch: `governance-v0`
