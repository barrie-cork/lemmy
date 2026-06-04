---
role: bm-task
verb: bm-cut
phase: m1-a
pr_number: null
created: 2026-06-04
related_dq: a192dbab1de8-001
---

# BM-cut brief — m1-a (M1 Tree A)

**Role:** `[role:bm-task]`
**Phase:** `m1-a` (branch `phase-m1-a`)
**Authored:** 2026-06-04
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base:** `governance-v0` HEAD at task-execution time (`ba263a76a` or later)
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". No laptop-side phase worktree; advisor drives via Junior dispatch from canonical `brehon-fork`. impl-task briefs author on `governance-v0` + trunk→phase sync.

> **Context:** M1 (chat infrastructure) was split into two phase branches per split-DQ `a192dbab1de8-001`. Tree B (`m1-b`, Tasks 1–7) **already shipped** — PR #177 merged @ `d6d027794`. This phase `m1-a` is **Tree A only**: the greenfield, workspace-EXCLUDED `services/bridge/` Matrix application-service bridge daemon (plan `m1.plan.md` §13 Tasks 8–13) plus Tree C docs (Task 14, folded in). m1-a consumes m1-b's already-shipped `bridge_notify` HTTP seam + `governance_messaging_config` table; it never edits them.

---

## 1. Role + dispatch line

```
[role:bm-task] m1-a bm-cut — see .claude/PRPs/briefs/m1-a-bm-cut-1.md
```

You are the **bm-task** subagent (Haiku 4.5 per your frontmatter — git/yq/gh ops, no heavy reasoning). Execute the branch-manager verb `bm-cut` per `.claude/commands/bm/bm-cut.md`, step by step.

---

## 2. Scope

**Produce:** phase branch `phase-m1-a` cut from `governance-v0` HEAD, pushed to `origin`, with upstream tracking set.

- **Phase:** `m1-a` (branch `phase-m1-a`)
- **PR number:** none (bm-cut — PR doesn't exist yet)
- **Trunk SHA:** `ba263a76a` on `governance-v0` (or later — base is HEAD at execution time)

**Explicit boundaries:**
- Cut branch `phase-m1-a` from `governance-v0` at its current HEAD
- Push with `git push -u origin phase-m1-a`
- Verify the plan file `.claude/PRPs/plans/m1-a.plan.md` is present on the new branch
- Write a one-line runlog entry to `.claude/runlog/m1-a-runlog.md` (create if absent): `bm: bm-cut complete — phase-m1-a cut from governance-v0 @ <SHA>`
- Commit + push the runlog entry on the new branch (per bm-cut.md Phase 5)

**Do NOT:**
- Open a PR (that comes at phase close after all impl tasks)
- Modify any file under `crates/`, `migrations/`, `docs/`, `services/bridge/`, `.claude/PRPs/plans/`, `.claude/PRPs/prds/`
- Merge or rebase anything

---

## 3. Required reading

1. `.claude/commands/bm/bm-cut.md` — the canonical bm-cut procedure; follow it step by step (note **Phase 8** finalize hazard)
2. `.claude/rules/branch-manager.md` — file-ownership boundaries and pre-cut checklist
3. `.claude/rules/phase-branch.md` — branch naming convention and pre-cut verifications
4. `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every `gh` call

---

## 4. Constraints

- **Branch name must be exactly** `phase-m1-a` (no variant)
- **Base must be** `governance-v0` (never `main`)
- **Pre-cut checks** (per `branch-manager.md`):
  1. `git status --short` on `governance-v0` — must be clean
  2. `git fetch origin && git log governance-v0..origin/governance-v0 --oneline` — must be empty (governance-v0 in sync)
  3. Plan file `.claude/PRPs/plans/m1-a.plan.md` must be present on `governance-v0`
- **Push with upstream tracking:** `git push -u origin phase-m1-a`
- **Runlog:** create `.claude/runlog/m1-a-runlog.md` if absent; append the bm-cut entry; commit + push on the **new branch** `phase-m1-a` (per bm-cut.md Phase 5, the runlog commit is the first commit on the new branch)
- **DQ mid-task discipline:** if a blocker arises, write a `kind: "blocker"` DQ entry (use `bash scripts/brehon/dq-v3-new-entry.sh` for the id), commit + push, and stop

---

## 5. Success signals

- `git ls-remote origin refs/heads/phase-m1-a` returns non-empty (branch exists on origin)
- `gh api repos/barrie-cork/lemmy/branches/phase-m1-a --jq '.name + " @ " + .commit.sha[0:9]'` confirms the branch
- Plan file `.claude/PRPs/plans/m1-a.plan.md` present on the new branch
- Runlog entry appended + committed + pushed on `phase-m1-a`
- Push to origin succeeded with upstream tracking set

---

## 6. Out of scope

- Editing impl code, plan files, design docs, `services/bridge/**`, or any file in the never-touch list
- Opening a PR (that comes at phase close)
- Merging or rebasing anything
- Tree B (`crates/**` governance-messaging-config) — shipped in `m1-b`; out of `m1-a`

---

## 8. KNOWN harness limitation — bm-cut creates divergence (do NOT merge back)

**bm-cut creates a divergence point, NOT a feature branch.** `phase-m1-a` is the deliverable; it must **NEVER** be merged back into `governance-v0`. The daemon's generic post-job finalize agent is feature-branch-shaped and may wrongly run `git merge --no-ff phase-m1-a` INTO daemon-local `governance-v0`, producing a spurious content-empty merge commit (`Merge phase-m1-a into governance-v0 (bm-cut task)`). Confirmed 2× (v1-AD-e #282 + v1-ship-1). See `.claude/lessons/feedback_junior_finalize_merges_bm_cut_branch.md`.

- **You (the bm-task worker):** just cut + push the branch + push the runlog. Do NOT merge anything. If the runlog write is blocked by the CC `.claude/**` sensitive-file gate, stop and report — the advisor will relocate it (`feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md`).
- **The advisor (post-task):** will verify daemon-local trunk as a ROUTINE step and recover via `git update-ref refs/heads/governance-v0 origin/governance-v0` (working-tree-safe) if the spurious merge appears.

---

## HANDOVER

```yaml
HANDOVER:
  task: m1-a-bm-cut
  filesCreated: [.claude/runlog/m1-a-runlog.md]
  filesModified: []
  keyDecisions:
    - "phase-m1-a cut from governance-v0 @ ba263a76a or later"
    - "m1-a = M1 Tree A only (services/bridge/, Tasks 8–13 + Task 14 docs); Tree B shipped as m1-b PR #177"
    - "plan reuses m1.plan.md §13 Tasks 8–13 via the m1-a.plan.md pointer"
  notes: "bm-cut only — no PR yet. Mode B lane (no laptop worktree)."
```

Brief complete. Dispatch as:

```
[role:bm-task] m1-a bm-cut — see .claude/PRPs/briefs/m1-a-bm-cut-1.md
```

Base branch: `governance-v0`
