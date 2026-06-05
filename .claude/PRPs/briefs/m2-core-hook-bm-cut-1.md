---
role: bm-task
verb: bm-cut
phase: m2-core-hook
pr_number: null
created: 2026-06-05
related_dq: null
---

# BM-cut brief — m2-core-hook (M2-core in-binary slice)

**Role:** `[role:bm-task]`
**Phase:** `m2-core-hook` (branch `phase-m2-core-hook`)
**Authored:** 2026-06-05
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base:** `governance-v0` HEAD at task-execution time (`a0ee41de3` or later)
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". No laptop-side phase worktree; advisor drives via Junior dispatch from canonical `brehon-fork`. impl-task briefs author on `governance-v0` + trunk→phase sync.

> **Context:** M2 (governance-triggered rooms) is split rooms-first. This phase `m2-core-hook` is the **in-binary slice only** of M2-core (plan `m2-core-transition-hook.plan.md`): the `governance_case_after_transition` hook at 11 transition sites, 10 `ENTRY_KIND_ROOM_*` consts, and the `append_room_event` library wrapper. The `services/bridge/` provisioning daemon, any HTTP callback route, and Matrix integration are OUT of this phase (separate bridge-side plan). All work is in `crates/`, fully cargo-workspace-gated; the workspace pulls zero Matrix deps.

---

## 1. Role + dispatch line

```
[role:bm-task] m2-core-hook bm-cut — see .claude/PRPs/briefs/m2-core-hook-bm-cut-1.md
```

You are the **bm-task** subagent (Haiku 4.5 per your frontmatter — git/yq/gh ops, no heavy reasoning). Execute the branch-manager verb `bm-cut` per `.claude/commands/bm/bm-cut.md`, step by step.

---

## 2. Scope

**Produce:** phase branch `phase-m2-core-hook` cut from `governance-v0` HEAD, pushed to `origin`, with upstream tracking set.

- **Phase:** `m2-core-hook` (branch `phase-m2-core-hook`)
- **PR number:** none (bm-cut — PR doesn't exist yet)
- **Trunk SHA:** `a0ee41de3` on `governance-v0` (or later — base is HEAD at execution time)

**Explicit boundaries:**
- Cut branch `phase-m2-core-hook` from `governance-v0` at its current HEAD
- Push with `git push -u origin phase-m2-core-hook`
- Verify the plan file `.claude/PRPs/plans/m2-core-transition-hook.plan.md` is present on the new branch
- Write a one-line runlog entry to `.claude/runlog/m2-core-hook-runlog.md` (create if absent): `bm: bm-cut complete — phase-m2-core-hook cut from governance-v0 @ <SHA>`
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

- **Branch name must be exactly** `phase-m2-core-hook` (no variant)
- **Base must be** `governance-v0` (never `main`)
- **Pre-cut checks** (per `branch-manager.md`):
  1. `git status --short` on `governance-v0` — must be clean
  2. `git fetch origin && git log governance-v0..origin/governance-v0 --oneline` — must be empty (governance-v0 in sync)
  3. Plan file `.claude/PRPs/plans/m2-core-transition-hook.plan.md` must be present on `governance-v0`
- **Push with upstream tracking:** `git push -u origin phase-m2-core-hook`
- **Runlog:** create `.claude/runlog/m2-core-hook-runlog.md` if absent; append the bm-cut entry; commit + push on the **new branch** `phase-m2-core-hook` (per bm-cut.md Phase 5, the runlog commit is the first commit on the new branch)
- **DQ mid-task discipline:** if a blocker arises, write a `kind: "blocker"` DQ entry (use `bash scripts/brehon/dq-v3-new-entry.sh` for the id), commit + push, and stop

---

## 5. Success signals

- `git ls-remote origin refs/heads/phase-m2-core-hook` returns non-empty (branch exists on origin)
- `gh api repos/barrie-cork/lemmy/branches/phase-m2-core-hook --jq '.name + " @ " + .commit.sha[0:9]'` confirms the branch
- Plan file `.claude/PRPs/plans/m2-core-transition-hook.plan.md` present on the new branch
- Runlog entry appended + committed + pushed on `phase-m2-core-hook`
- Push to origin succeeded with upstream tracking set

---

## 6. Out of scope

- Editing impl code, plan files, design docs, `services/bridge/**`, or any file in the never-touch list
- Opening a PR (that comes at phase close)
- Merging or rebasing anything

---

## 8. KNOWN harness limitation — bm-cut creates divergence (do NOT merge back)

**bm-cut creates a divergence point, NOT a feature branch.** `phase-m2-core-hook` is the deliverable; it must **NEVER** be merged back into `governance-v0`. The daemon's generic post-job finalize agent is feature-branch-shaped and may wrongly run `git merge --no-ff phase-m2-core-hook` INTO daemon-local `governance-v0`, producing a spurious content-empty merge commit. Confirmed 2× (v1-AD-e #282 + v1-ship-1). See `.claude/lessons/feedback_junior_finalize_merges_bm_cut_branch.md`.

- **You (the bm-task worker):** just cut + push the branch + push the runlog. Do NOT merge anything. If the runlog write is blocked by the CC `.claude/**` sensitive-file gate, stop and report — the advisor will relocate it (`feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md`).
- **The advisor (post-task):** will verify daemon-local trunk as a ROUTINE step and recover via `git update-ref refs/heads/governance-v0 origin/governance-v0` (working-tree-safe) if the spurious merge appears.

---

## HANDOVER

```yaml
HANDOVER:
  task: m2-core-hook-bm-cut
  filesCreated: [.claude/runlog/m2-core-hook-runlog.md]
  filesModified: []
  keyDecisions:
    - "phase-m2-core-hook cut from governance-v0 @ a0ee41de3 or later"
    - "m2-core-hook = M2-core in-binary slice only (hook + entry kinds + append_room_event wrapper); services/bridge/ daemon is a separate plan"
    - "plan = m2-core-transition-hook.plan.md; pre-Shape-G (cargo on laptop via validate-pending-laptop DQ)"
  notes: "bm-cut only — no PR yet. Mode B lane (no laptop worktree). MiniMax trial fires this phase (Tasks 1,2,3,5,7)."
```

Brief complete. Dispatch as:

```
[role:bm-task] m2-core-hook bm-cut — see .claude/PRPs/briefs/m2-core-hook-bm-cut-1.md
```

Base branch: `governance-v0`
