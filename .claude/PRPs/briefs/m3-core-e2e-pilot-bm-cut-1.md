---
role: bm-task
verb: bm-cut
phase: m3-core-e2e-pilot
pr_number: null
related_dq: null
created: 2026-06-20
---

# BM-cut brief — m3-core-e2e-pilot (M3 town halls — Phase 6 of 6, FINAL)

**Role:** `[role:bm-task]`
**Phase:** `m3-core-e2e-pilot` (branch `phase-m3-core-e2e-pilot`)
**Authored:** 2026-06-20
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base:** `governance-v0` HEAD at task-execution time (`486f62283` or later)
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". No laptop-side phase worktree; advisor drives via Junior dispatch from canonical `brehon-fork`. All cargo (bridge Linux via `cargo-linux.sh`) + the live `-e2e` stack run laptop-side via validate-pending-laptop[-linux][-e2e] DQs regardless of mode.

> **Context:** m3-core-e2e-pilot is M3 Phase 6 — the FINAL M3-core sub-phase:
> full town-hall acceptance e2e + a real operator-run D2 pilot. It is NOT a
> code-add phase — registry frozen at 72, no migration, no new prod dep
> (one DQ-gated `[dev-dependencies]` livekit-client line possible in Task 4).
> Two halves: Half A (Tasks 1–6 — e2e harness bring-up [MinIO + 2nd federated
> instance, both net-new] + recording cr-2/cr-3 carry-forwards + 4 acceptance
> test files turned live) + Half B (Task 7 — operator-run D2 pilot, NON-impl,
> no cargo DoD). **The governing plan
> `.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` ALREADY EXISTS** (on
> `governance-v0` at `486f62283`, plan-approved at gate 1 on 2026-06-20) — it is
> the bm-cut justification. The normal plan-presence check APPLIES. The plan
> must be reachable on `governance-v0` at task-execution time.

---

## 1. Role + dispatch line

```
[role:bm-task] m3-core-e2e-pilot bm-cut — see .claude/PRPs/briefs/m3-core-e2e-pilot-bm-cut-1.md
```

You are the **bm-task** subagent (Haiku 4.5 — git/yq/gh ops, no heavy reasoning). Execute the branch-manager verb `bm-cut` per `.claude/commands/bm/bm-cut.md`, step by step.

---

## 2. Scope

**Produce:** phase branch `phase-m3-core-e2e-pilot` cut from `governance-v0` HEAD, pushed to `origin`, with upstream tracking set.

- **Phase:** `m3-core-e2e-pilot` (branch `phase-m3-core-e2e-pilot`)
- **PR number:** none (bm-cut — PR doesn't exist yet)
- **Trunk SHA:** `486f62283` on `governance-v0` (or later — base is HEAD at execution time)

**Explicit boundaries:**
- Cut branch `phase-m3-core-e2e-pilot` from `governance-v0` at its current HEAD
- Push with `git push -u origin phase-m3-core-e2e-pilot`
- **Plan-presence check APPLIES (normal):** the governing plan `.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` IS present on `governance-v0` (commit `486f62283` or later). Verify it is reachable: `git cat-file -e governance-v0:.claude/PRPs/plans/m3-core-e2e-pilot.plan.md`. If absent, STOP and file a DQ.
- Write a one-line runlog entry to `.claude/runlog/m3-core-e2e-pilot-runlog.md` (create if absent): `bm: bm-cut complete — phase-m3-core-e2e-pilot cut from governance-v0 @ <SHA>`
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

- **Branch name must be exactly** `phase-m3-core-e2e-pilot` (no variant)
- **Base must be** `governance-v0` (never `main`)
- **Pre-cut checks** (per `branch-manager.md`):
  1. `git status --short` on `governance-v0` — must be clean
  2. `git fetch origin && git log governance-v0..origin/governance-v0 --oneline` — must be empty (governance-v0 in sync)
  3. **Plan-presence check APPLIES:** `git cat-file -e governance-v0:.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` must succeed
- **Push with upstream tracking:** `git push -u origin phase-m3-core-e2e-pilot`
- **Runlog:** create `.claude/runlog/m3-core-e2e-pilot-runlog.md` if absent; append the bm-cut entry; commit + push on the **new branch** `phase-m3-core-e2e-pilot` (per bm-cut.md Phase 5, the runlog commit is the first commit on the new branch)
- **DQ mid-task discipline:** if a blocker arises, write a `kind: "blocker"` DQ entry (use `bash scripts/brehon/dq-v3-new-entry.sh` for the id), commit + push, and stop

---

## 5. Success signals

- `git ls-remote origin refs/heads/phase-m3-core-e2e-pilot` returns non-empty (branch exists on origin)
- `gh api repos/barrie-cork/lemmy/branches/phase-m3-core-e2e-pilot --jq '.name + " @ " + .commit.sha[0:9]'` confirms the branch
- Runlog entry appended + committed + pushed on `phase-m3-core-e2e-pilot`
- Push to origin succeeded with upstream tracking set

---

## 6. Out of scope

- Editing impl code, plan files, design docs, `services/bridge/**`, or any file in the never-touch list
- Opening a PR (that comes at phase close)
- Merging or rebasing anything
- Authoring or editing the plan (it already exists at `486f62283`)

---

## 7. KNOWN harness limitation — bm-cut creates divergence (do NOT merge back)

**bm-cut creates a divergence point, NOT a feature branch.** `phase-m3-core-e2e-pilot` must **NEVER** be merged back into `governance-v0` by the finalize agent. The daemon's generic post-job finalize may wrongly run `git merge --no-ff phase-m3-core-e2e-pilot` INTO daemon-local `governance-v0`. Confirmed repeatedly (v1-AD-e #282, v1-ship-1, the bm-pr variant on m3-core-entry-kinds PR #200 auto-merge, the long-name refspec class in stage-mode/emergency-mute/recording) — so the advisor's post-task daemon-trunk verification is MANDATORY this phase.

- **You (the bm-task worker):** just cut + push the branch + push the runlog. Do NOT merge anything. If the runlog write is blocked by the CC `.claude/**` sensitive-file gate, stop and report.
- **The advisor (post-task):** will verify daemon-local trunk as a ROUTINE step (`ssh homeserver 'cd /srv/brehon-fork && git log governance-v0 --oneline -1'`) and recover via `git update-ref` + push if a spurious merge landed. For the finalize-merge of this long-named worker branch, apply `feedback_daemon_long_name_refspec_finalize.md` (`git fetch origin <worker>:refs/heads/_fin<id>`) proactively.

---

## HANDOVER

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-bm-cut
  filesCreated: [.claude/runlog/m3-core-e2e-pilot-runlog.md]
  filesModified: []
  keyDecisions:
    - "phase-m3-core-e2e-pilot cut from governance-v0 @ 486f62283 or later"
    - "Plan ALREADY EXISTS (.claude/PRPs/plans/m3-core-e2e-pilot.plan.md @ 486f62283) — normal plan-presence check applies; gate-1 approved 2026-06-20"
    - "Complexity 7/10 (proceed-as-one, high end of M3-core); NO migration, NO new const (registry frozen at 72), NO new prod dep (one DQ-gated dev-dep possible Task 4); zero crates/** change planned"
    - "9 tasks: Task0 barrier; Task1(harness — MinIO + 2nd federated instance override compose + reach-smoke, net-new infra, Docker DoD not cargo); Task2(bridge src — RecordingSink sync->async + cr-2/cr-3, requires:1 at -e2e); Tasks3-6[P at edit layer](4 acceptance test files turned live, each requires:1, laptop-serial at -e2e); Task7(D2 pilot runbook, NON-impl, no cargo); Task8 retro"
    - "MARQUEE = Task4 emergency_mute.rs cross-instance <500ms PUBLISHER-CLIENT (DQ 3004b6625b83-001 resolved option-B by user 2026-06-20: automated in-instance + cross-instance to D2 pilot per PRD fallback). Server-side measurement = FALSE GREEN catch-fire."
    - "Bridge compiles Linux-only (cargo-linux.sh --manifest-path); Tasks 2-6 write validate-pending-laptop-linux DQ; Tasks 3-6 also write validate-pending-laptop-e2e (needs full RTC stack from Task 1). gate-4 (e2e local-vs-dispatch) is LIVE — advisor surfaces ONCE."
    - "Half B (Task 7) is operational/NON-impl: DoD = retro-recorded pilot outcome (did a real town hall run? chair passed mic? recording landed?), NOT a test pass."
  notes: "bm-cut only — no PR yet. Mode B lane (no laptop worktree). Next: advisor dispatches impl cohort starting Task 0 (barrier) then Task 1 (harness, hard requires: of every acceptance test). Cross-lane cap = 2 running; bridge cold build ~10-20 min first run."
```

Brief complete. Dispatch as:

```
[role:bm-task] m3-core-e2e-pilot bm-cut — see .claude/PRPs/briefs/m3-core-e2e-pilot-bm-cut-1.md
```

Base branch: `governance-v0`
