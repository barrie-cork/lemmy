---
role: bm-task
verb: bm-cut
phase: v1-deps-r2
pr_number: null
created: 2026-06-05
related_dq: null
---

# BM-cut brief — v1-deps-r2 (security alert sweep — webmention inline)

**Role:** `[role:bm-task]`
**Phase:** `v1-deps-r2` (branch `phase-v1-deps-r2`)
**Authored:** 2026-06-05
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base:** `governance-v0` HEAD at task-execution time (`659232391` or later)
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". No laptop-side phase worktree; advisor drives via Junior dispatch from canonical `brehon-fork`. impl is laptop-local (advisor writes Rust inline in this session).

> **Context:** v1-deps-r2 is the deferred Phase 4 from the v1 close-out — security alert sweep. Scope collapsed to 1 impl task: inline the W3C Webmention sender, drop `webmention 0.6.0`, eliminating `rustls-webpki 0.101.7` → `rustls 0.21.12` → `hyper-rustls 0.24.2` → `reqwest 0.11.27` → `webmention 0.6.0` chain (3 Dependabot alerts #49/#50/#55, one HIGH). Plan: `.claude/PRPs/plans/v1-deps-r2.plan.md`. Impl: advisor-inline (laptop-local, no EliteDesk).

---

## 1. Role + dispatch line

```
[role:bm-task] v1-deps-r2 bm-cut — see .claude/PRPs/briefs/v1-deps-r2-bm-cut-1.md
```

You are the **bm-task** subagent (Haiku 4.5 per your frontmatter — git/yq/gh ops, no heavy reasoning). Execute the branch-manager verb `bm-cut` per `.claude/commands/bm/bm-cut.md`, step by step.

---

## 2. Scope

**Produce:** phase branch `phase-v1-deps-r2` cut from `governance-v0` HEAD, pushed to `origin`, with upstream tracking set.

- **Phase:** `v1-deps-r2` (branch `phase-v1-deps-r2`)
- **PR number:** none (bm-cut — PR doesn't exist yet)
- **Trunk SHA:** `659232391` on `governance-v0` (or later — base is HEAD at execution time)

**Explicit boundaries:**
- Cut branch `phase-v1-deps-r2` from `governance-v0` at its current HEAD
- Push with `git push -u origin phase-v1-deps-r2`
- Verify the plan file `.claude/PRPs/plans/v1-deps-r2.plan.md` is present on the new branch
- Write a one-line runlog entry to `.claude/runlog/v1-deps-r2-runlog.md` (create if absent): `bm: bm-cut complete — phase-v1-deps-r2 cut from governance-v0 @ <SHA>`
- Commit + push the runlog entry on the new branch (per bm-cut.md Phase 5, the runlog commit is the first commit on the new branch)

**Do NOT:**
- Open a PR (that comes at phase close after impl)
- Modify any file under `crates/`, `migrations/`, `docs/`, `.claude/PRPs/plans/`, `.claude/PRPs/prds/`
- Merge or rebase anything

---

## 3. Required reading

1. `.claude/commands/bm/bm-cut.md` — the canonical bm-cut procedure; follow it step by step (note **Phase 8** finalize hazard)
2. `.claude/rules/branch-manager.md` — file-ownership boundaries and pre-cut checklist
3. `.claude/rules/phase-branch.md` — branch naming convention and pre-cut verifications
4. `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every `gh` call
5. `.claude/PRPs/plans/v1-deps-r2.plan.md` — plan committed before this task (per bm-cut verb — bm-cut reads the plan to verify it's present)

---

## 4. Constraints

- **Branch name must be exactly** `phase-v1-deps-r2` (no variant)
- **Base must be** `governance-v0` (never `main`)
- **Pre-cut checks** (per `branch-manager.md`):
  1. `git status --short` on `governance-v0` — must be clean
  2. `git fetch origin && git log governance-v0..origin/governance-v0 --oneline` — must be empty (governance-v0 in sync)
  3. Plan file `.claude/PRPs/plans/v1-deps-r2.plan.md` must be present on `governance-v0`
- **Push with upstream tracking:** `git push -u origin phase-v1-deps-r2`
- **Runlog:** create `.claude/runlog/v1-deps-r2-runlog.md` if absent; append the bm-cut entry; commit + push on the **new branch** `phase-v1-deps-r2`
- **DQ mid-task discipline:** if a blocker arises, write a `kind: "blocker"` DQ entry (use `bash scripts/brehon/dq-v3-new-entry.sh` for the id), commit + push, and stop
- **Hard refusals apply:** never touch `crates/`, `migrations/`, `tests/`, `Cargo.toml`, `Cargo.lock`. Never merge, rebase, or open a PR. Never post PR comments without confirmation. Never send Telegram pings without confirmation.

---

## 5. Success signals

- `git ls-remote origin refs/heads/phase-v1-deps-r2` returns non-empty (branch exists on origin)
- `gh api repos/barrie-cork/lemmy/branches/phase-v1-deps-r2 --jq '.name + " @ " + .commit.sha[0:9]'` confirms the branch
- Plan file `.claude/PRPs/plans/v1-deps-r2.plan.md` present on the new branch
- Runlog entry appended + committed + pushed on `phase-v1-deps-r2`
- Push to origin succeeded with upstream tracking set

---

## 6. Out of scope

- Editing impl code, plan files, design docs, or any file in the never-touch list
- Opening a PR (that comes at phase close)
- Merging or rebasing anything
- Any impl work — advisor does that inline after bm-cut completes

---

## KNOWN harness limitation — bm-cut creates divergence (do NOT merge back)

**bm-cut creates a divergence point, NOT a feature branch.** `phase-v1-deps-r2` must **NEVER** be merged back into `governance-v0`. The daemon's generic post-job finalize agent may wrongly run `git merge --no-ff phase-v1-deps-r2` INTO daemon-local `governance-v0`. Confirmed 2× prior phases. See `.claude/lessons/feedback_junior_finalize_merges_bm_cut_branch.md`.

- **You (the bm-task worker):** just cut + push the branch + push the runlog. Do NOT merge anything. If the runlog write is blocked by the CC `.claude/**` sensitive-file gate, stop and report.
- **The advisor (post-task):** will verify daemon-local trunk as a ROUTINE step and recover via `git update-ref refs/heads/governance-v0 origin/governance-v0` if the spurious merge appears.

---

## HANDOVER

```yaml
HANDOVER:
  task: v1-deps-r2-bm-cut
  filesCreated: [.claude/runlog/v1-deps-r2-runlog.md]
  filesModified: []
  keyDecisions:
    - "phase-v1-deps-r2 cut from governance-v0 @ 659232391 or later"
    - "v1-deps-r2 = security alert sweep; 1 impl task (webmention inline + drop dep)"
    - "impl is advisor-inline laptop-local; no EliteDesk dispatch for impl"
  notes: "bm-cut only — no PR yet. Mode B lane (no laptop worktree)."
```

Brief complete. Dispatch as:

```
[role:bm-task] v1-deps-r2 bm-cut — see .claude/PRPs/briefs/v1-deps-r2-bm-cut-1.md
```

Base branch: `governance-v0`
