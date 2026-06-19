---
role: bm-task
verb: bm-cut
phase: m3-core-emergency-mute
pr_number: null
related_dq: null
created: 2026-06-19
---

# BM-cut brief — m3-core-emergency-mute (M3 town halls — Phase 4)

**Role:** `[role:bm-task]`
**Phase:** `m3-core-emergency-mute` (branch `phase-m3-core-emergency-mute`)
**Authored:** 2026-06-19
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base:** `governance-v0` HEAD at task-execution time (`d943526fa` or later)
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". No laptop-side phase worktree; advisor drives via Junior dispatch from canonical `brehon-fork`. All cargo (crates Windows + bridge Linux) runs laptop-side via validate-pending-laptop[-linux] DQs regardless of mode.

> **Context:** m3-core-emergency-mute is M3 Phase 4 — federation-wide emergency
> mute-all for town-hall rooms (Matrix power-levels = cross-instance authority +
> local LiveKit revoke sweep for in-instance <500ms), emitting the FIRST
> `room_mute_all` chain entry (const already REGISTERED in Phase 2 — emit-only).
> LOW complexity (score 2, proceed-as-one), bridge-side logic on top of
> stage-mode's emit seam + m2-late-2's power-level machinery. **The governing plan
> `.claude/PRPs/plans/m3-core-emergency-mute.plan.md` ALREADY EXISTS** (on
> `governance-v0` at `ec5d8996c`, plan-approved at gate 1) — it is the bm-cut
> justification. The normal plan-presence check APPLIES. The plan must be
> reachable on `governance-v0` at task-execution time.

---

## 1. Role + dispatch line

```
[role:bm-task] m3-core-emergency-mute bm-cut — see .claude/PRPs/briefs/m3-core-emergency-mute-bm-cut-1.md
```

You are the **bm-task** subagent (Haiku 4.5 — git/yq/gh ops, no heavy reasoning). Execute the branch-manager verb `bm-cut` per `.claude/commands/bm/bm-cut.md`, step by step.

---

## 2. Scope

**Produce:** phase branch `phase-m3-core-emergency-mute` cut from `governance-v0` HEAD, pushed to `origin`, with upstream tracking set.

- **Phase:** `m3-core-emergency-mute` (branch `phase-m3-core-emergency-mute`)
- **PR number:** none (bm-cut — PR doesn't exist yet)
- **Trunk SHA:** `d943526fa` on `governance-v0` (or later — base is HEAD at execution time)

**Explicit boundaries:**
- Cut branch `phase-m3-core-emergency-mute` from `governance-v0` at its current HEAD
- Push with `git push -u origin phase-m3-core-emergency-mute`
- **Plan-presence check APPLIES (normal):** the governing plan `.claude/PRPs/plans/m3-core-emergency-mute.plan.md` IS present on `governance-v0` (commit `ec5d8996c` or later). Verify it is reachable: `git cat-file -e governance-v0:.claude/PRPs/plans/m3-core-emergency-mute.plan.md`. If absent, STOP and file a DQ.
- Write a one-line runlog entry to `.claude/runlog/m3-core-emergency-mute-runlog.md` (create if absent): `bm: bm-cut complete — phase-m3-core-emergency-mute cut from governance-v0 @ <SHA>`
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

- **Branch name must be exactly** `phase-m3-core-emergency-mute` (no variant)
- **Base must be** `governance-v0` (never `main`)
- **Pre-cut checks** (per `branch-manager.md`):
  1. `git status --short` on `governance-v0` — must be clean
  2. `git fetch origin && git log governance-v0..origin/governance-v0 --oneline` — must be empty (governance-v0 in sync)
  3. **Plan-presence check APPLIES:** `git cat-file -e governance-v0:.claude/PRPs/plans/m3-core-emergency-mute.plan.md` must succeed
- **Push with upstream tracking:** `git push -u origin phase-m3-core-emergency-mute`
- **Runlog:** create `.claude/runlog/m3-core-emergency-mute-runlog.md` if absent; append the bm-cut entry; commit + push on the **new branch** `phase-m3-core-emergency-mute` (per bm-cut.md Phase 5, the runlog commit is the first commit on the new branch)
- **DQ mid-task discipline:** if a blocker arises, write a `kind: "blocker"` DQ entry (use `bash scripts/brehon/dq-v3-new-entry.sh` for the id), commit + push, and stop

---

## 5. Success signals

- `git ls-remote origin refs/heads/phase-m3-core-emergency-mute` returns non-empty (branch exists on origin)
- `gh api repos/barrie-cork/lemmy/branches/phase-m3-core-emergency-mute --jq '.name + " @ " + .commit.sha[0:9]'` confirms the branch
- Runlog entry appended + committed + pushed on `phase-m3-core-emergency-mute`
- Push to origin succeeded with upstream tracking set

---

## 6. Out of scope

- Editing impl code, plan files, design docs, `services/bridge/**`, or any file in the never-touch list
- Opening a PR (that comes at phase close)
- Merging or rebasing anything
- Authoring or editing the plan (it already exists at `ec5d8996c`)

---

## 7. KNOWN harness limitation — bm-cut creates divergence (do NOT merge back)

**bm-cut creates a divergence point, NOT a feature branch.** `phase-m3-core-emergency-mute` must **NEVER** be merged back into `governance-v0` by the finalize agent. The daemon's generic post-job finalize may wrongly run `git merge --no-ff phase-m3-core-emergency-mute` INTO daemon-local `governance-v0`. Confirmed 4× now (v1-AD-e #282 + v1-ship-1, the bm-pr variant on m3-core-entry-kinds PR #200 auto-merge, and the `feedback_daemon_long_name_refspec_finalize.md` long-name refspec class 4× in stage-mode) — so the advisor's post-task daemon-trunk verification is MANDATORY this phase.

- **You (the bm-task worker):** just cut + push the branch + push the runlog. Do NOT merge anything. If the runlog write is blocked by the CC `.claude/**` sensitive-file gate, stop and report.
- **The advisor (post-task):** will verify daemon-local trunk as a ROUTINE step (`ssh homeserver 'cd /srv/brehon-fork && git log governance-v0 --oneline -1'`) and recover via `git update-ref` + push if a spurious merge landed. For the finalize-merge of this long-named worker branch, apply `feedback_daemon_long_name_refspec_finalize.md` (`git fetch origin <worker>:refs/heads/_fin<id>`) proactively.

---

## HANDOVER

```yaml
HANDOVER:
  task: m3-core-emergency-mute-bm-cut
  filesCreated: [.claude/runlog/m3-core-emergency-mute-runlog.md]
  filesModified: []
  keyDecisions:
    - "phase-m3-core-emergency-mute cut from governance-v0 @ d943526fa or later"
    - "Plan ALREADY EXISTS (.claude/PRPs/plans/m3-core-emergency-mute.plan.md @ ec5d8996c) — normal plan-presence check applies; gate-1 approved"
    - "LOW complexity (score 2, proceed-as-one); bridge-side logic composing stage-mode emit seam + m2-late-2 power-level machinery"
    - "5 tasks: Task0 barrier; Task1[P](crates Windows federated DTO) + Task2[P](bridge mute_handler); Task3 serial(requires:1); Task4 serial-docker; Task5 retro"
    - "FIRST emission of room_mute_all (const REGISTERED in Phase 2 — emit-only; do NOT re-register or bump count from 72)"
    - "Mechanism: Matrix power-levels = cross-instance authority (mirror sanction_handler) + LiveKit revoke sweep = local; NO new const/migration/dep/sidecar/emitter"
    - "Bridge compiles Linux-only (cargo-linux.sh --manifest-path); Tasks 2-4 write validate-pending-laptop-linux DQ"
    - "One in-scope crates/** touch: Task1 RoomEventPayload one optional `federated` field (trivial DTO); entry-kind registry UNTOUCHED; no migration"
  notes: "bm-cut only — no PR yet. Mode B lane (no laptop worktree). Next: /auto-phase m3-core-emergency-mute --start-from impl-cohort-1 (Cohort A = Task1[P]+Task2[P])."
```

Brief complete. Dispatch as:

```
[role:bm-task] m3-core-emergency-mute bm-cut — see .claude/PRPs/briefs/m3-core-emergency-mute-bm-cut-1.md
```

Base branch: `governance-v0`
