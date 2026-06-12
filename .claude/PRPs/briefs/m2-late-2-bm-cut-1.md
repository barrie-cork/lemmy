---
role: bm-task
verb: bm-cut
phase: m2-late-2
pr_number: null
related_dq: null
created: 2026-06-12
---

# BM-cut brief — m2-late-2 (bridge power-level enforcement + CR-A atomicity fix + pilot verification)

**Role:** `[role:bm-task]`
**Phase:** `m2-late-2` (branch `phase-m2-late-2`)
**Authored:** 2026-06-12
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base:** `governance-v0` HEAD at task-execution time (`376c937a6` or later)
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". No laptop-side phase worktree; advisor drives via Junior dispatch from canonical `brehon-fork`.

> **Context:** m2-late-2 closes the two deliberate m2-late-1 carry-forward stubs
> (CR-A atomicity gap in `enqueue_sanction_event`; bridge power-level enforcement
> stub in `services/bridge/src/sanction_handler.rs`) and adds pilot verification.
> The clarified planning brief is `.claude/PRPs/briefs/m2-late-2-planning-1.md`
> (clarify pass complete 2026-06-12, DQ `a3d0e9941441-058..-061`).
> **The plan `.claude/PRPs/plans/m2-late-2.plan.md` does NOT exist yet** — the
> planning Junior authors it AFTER this bm-cut. Do NOT STOP or file-DQ for a
> missing plan file; that is expected at bm-cut time.

---

## 1. Role + dispatch line

```
[role:bm-task] m2-late-2 bm-cut — see .claude/PRPs/briefs/m2-late-2-bm-cut-1.md
```

You are the **bm-task** subagent (Haiku 4.5 — git/yq/gh ops, no heavy reasoning). Execute the branch-manager verb `bm-cut` per `.claude/commands/bm/bm-cut.md`, step by step.

---

## 2. Scope

**Produce:** phase branch `phase-m2-late-2` cut from `governance-v0` HEAD, pushed to `origin`, with upstream tracking set.

- **Phase:** `m2-late-2` (branch `phase-m2-late-2`)
- **PR number:** none (bm-cut — PR doesn't exist yet)
- **Trunk SHA:** `376c937a6` on `governance-v0` (or later — base is HEAD at execution time)

**Explicit boundaries:**
- Cut branch `phase-m2-late-2` from `governance-v0` at its current HEAD
- Push with `git push -u origin phase-m2-late-2`
- **Plan-presence check is RELAXED for this bm-cut:** the governing plan `.claude/PRPs/plans/m2-late-2.plan.md` does NOT exist yet (the planning Junior authors it AFTER this cut). Do NOT STOP or file a DQ for a missing plan file. The planning *brief* `.claude/PRPs/briefs/m2-late-2-planning-1.md` IS present on `governance-v0` — that is the only artifact that must exist.
- Write a one-line runlog entry to `.claude/runlog/m2-late-2-runlog.md` (create if absent): `bm: bm-cut complete — phase-m2-late-2 cut from governance-v0 @ <SHA>`
- Commit + push the runlog entry on the new branch (per bm-cut.md Phase 5)

**Do NOT:**
- Open a PR (that comes at phase close after all impl tasks)
- Modify any file under `crates/`, `migrations/`, `docs/`, `services/bridge/`, `.claude/PRPs/plans/`, `.claude/PRPs/prds/`
- Merge or rebase anything
- Author the plan (the planning Junior does that after this cut)

---

## 3. Required reading

1. `.claude/commands/bm/bm-cut.md` — the canonical bm-cut procedure; follow it step by step (note **Phase 8** finalize hazard)
2. `.claude/rules/branch-manager.md` — file-ownership boundaries and pre-cut checklist
3. `.claude/rules/phase-branch.md` — branch naming convention and pre-cut verifications
4. `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every `gh` call

---

## 4. Constraints

- **Branch name must be exactly** `phase-m2-late-2` (no variant)
- **Base must be** `governance-v0` (never `main`)
- **Pre-cut checks** (per `branch-manager.md`):
  1. `git status --short` on `governance-v0` — must be clean
  2. `git fetch origin && git log governance-v0..origin/governance-v0 --oneline` — must be empty (governance-v0 in sync)
  3. **Plan-presence check SKIPPED** — no `m2-late-2.plan.md` exists at bm-cut time (see §2). Verify the planning *brief* `.claude/PRPs/briefs/m2-late-2-planning-1.md` is present instead.
- **Push with upstream tracking:** `git push -u origin phase-m2-late-2`
- **Runlog:** create `.claude/runlog/m2-late-2-runlog.md` if absent; append the bm-cut entry; commit + push on the **new branch** `phase-m2-late-2` (per bm-cut.md Phase 5, the runlog commit is the first commit on the new branch)
- **DQ mid-task discipline:** if a blocker arises, write a `kind: "blocker"` DQ entry (use `bash scripts/brehon/dq-v3-new-entry.sh` for the id), commit + push, and stop

---

## 5. Success signals

- `git ls-remote origin refs/heads/phase-m2-late-2` returns non-empty (branch exists on origin)
- `gh api repos/barrie-cork/lemmy/branches/phase-m2-late-2 --jq '.name + " @ " + .commit.sha[0:9]'` confirms the branch
- Runlog entry appended + committed + pushed on `phase-m2-late-2`
- Push to origin succeeded with upstream tracking set

---

## 6. Out of scope

- Editing impl code, plan files, design docs, `services/bridge/**`, or any file in the never-touch list
- Opening a PR (that comes at phase close)
- Merging or rebasing anything
- Authoring the plan (the planning Junior does that)

---

## 7. KNOWN harness limitation — bm-cut creates divergence (do NOT merge back)

**bm-cut creates a divergence point, NOT a feature branch.** `phase-m2-late-2` must **NEVER** be merged back into `governance-v0` by the finalize agent. The daemon's generic post-job finalize may wrongly run `git merge --no-ff phase-m2-late-2` INTO daemon-local `governance-v0`. Confirmed 2× (v1-AD-e #282 + v1-ship-1).

- **You (the bm-task worker):** just cut + push the branch + push the runlog. Do NOT merge anything. If the runlog write is blocked by the CC `.claude/**` sensitive-file gate, stop and report.
- **The advisor (post-task):** will verify daemon-local trunk as a ROUTINE step (`ssh homeserver 'cd /srv/brehon-fork && git log governance-v0 --oneline -1'`) and recover via `git update-ref` + push if a spurious merge landed.

---

## HANDOVER

```yaml
HANDOVER:
  task: m2-late-2-bm-cut
  filesCreated: [.claude/runlog/m2-late-2-runlog.md]
  filesModified: []
  keyDecisions:
    - "phase-m2-late-2 cut from governance-v0 @ 376c937a6 or later"
    - "Plan does NOT exist at bm-cut time — planning Junior authors m2-late-2.plan.md AFTER this cut; plan-presence check relaxed"
    - "Clarify pass complete 2026-06-12 (DQ a3d0e9941441-058..-061): case_id payload extension, power-levels-only enforcement, pilot-verify as task T6, CR-A mechanical via reborrow"
    - "Pre-Shape-G (cargo on laptop via validate-pending-laptop DQ); services/bridge workspace-excluded (R9)"
  notes: "bm-cut only — no PR yet. Mode B lane (no laptop worktree). MiniMax trial SUSPENDED (memory 827)."
```

Brief complete. Dispatch as:

```
[role:bm-task] m2-late-2 bm-cut — see .claude/PRPs/briefs/m2-late-2-bm-cut-1.md
```

Base branch: `governance-v0`
