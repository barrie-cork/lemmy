---
role: bm-task
verb: bm-cut
phase: m3-core-infra
pr_number: null
related_dq: null
created: 2026-06-18
---

# BM-cut brief — m3-core-infra (M3 town halls — Phase 1)

**Role:** `[role:bm-task]`
**Phase:** `m3-core-infra` (branch `phase-m3-core-infra`)
**Authored:** 2026-06-18
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base:** `governance-v0` HEAD at task-execution time (`bbbae593e` or later)
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". No laptop-side phase worktree; advisor drives via Junior dispatch from canonical `brehon-fork`. All cargo (Postgres workspace, bridge Linux, e2e, deploy-smoke) runs laptop-side via validate-pending-laptop[-linux] DQs regardless of mode.

> **Context:** m3-core-infra is M3 Phase 1 — the deployable, fully-optional RTC stack
> (LiveKit/lk-jwt/Element Call sidecars + `rtc_enabled` flag default false + bridge
> LiveKit JWT minting with ADR-015 identity→pseudonym + `bridge_room` RTC-state
> columns). MEDIUM complexity (score 13, proceed-as-one per user 2026-06-18), 3
> validation surfaces (Postgres/Windows + bridge/Linux + Docker deploy-smoke).
> **The governing plan `.claude/PRPs/plans/m3-core-infra.plan.md` ALREADY EXISTS**
> (on `governance-v0` at `bbbae593e`, plan-approved at gate 1) — it is the bm-cut
> justification. The normal plan-presence check APPLIES. The plan must be reachable
> on `governance-v0` at task-execution time.

---

## 1. Role + dispatch line

```
[role:bm-task] m3-core-infra bm-cut — see .claude/PRPs/briefs/m3-core-infra-bm-cut-1.md
```

You are the **bm-task** subagent (Haiku 4.5 — git/yq/gh ops, no heavy reasoning). Execute the branch-manager verb `bm-cut` per `.claude/commands/bm/bm-cut.md`, step by step.

---

## 2. Scope

**Produce:** phase branch `phase-m3-core-infra` cut from `governance-v0` HEAD, pushed to `origin`, with upstream tracking set.

- **Phase:** `m3-core-infra` (branch `phase-m3-core-infra`)
- **PR number:** none (bm-cut — PR doesn't exist yet)
- **Trunk SHA:** `bbbae593e` on `governance-v0` (or later — base is HEAD at execution time)

**Explicit boundaries:**
- Cut branch `phase-m3-core-infra` from `governance-v0` at its current HEAD
- Push with `git push -u origin phase-m3-core-infra`
- **Plan-presence check APPLIES (normal):** the governing plan `.claude/PRPs/plans/m3-core-infra.plan.md` IS present on `governance-v0` (commit `bbbae593e` or earlier `2c6065bea`). Verify it is reachable: `git cat-file -e governance-v0:.claude/PRPs/plans/m3-core-infra.plan.md`. If absent, STOP and file a DQ.
- Write a one-line runlog entry to `.claude/runlog/m3-core-infra-runlog.md` (create if absent): `bm: bm-cut complete — phase-m3-core-infra cut from governance-v0 @ <SHA>`
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

- **Branch name must be exactly** `phase-m3-core-infra` (no variant)
- **Base must be** `governance-v0` (never `main`)
- **Pre-cut checks** (per `branch-manager.md`):
  1. `git status --short` on `governance-v0` — must be clean
  2. `git fetch origin && git log governance-v0..origin/governance-v0 --oneline` — must be empty (governance-v0 in sync)
  3. **Plan-presence check APPLIES:** `git cat-file -e governance-v0:.claude/PRPs/plans/m3-core-infra.plan.md` must succeed
- **Push with upstream tracking:** `git push -u origin phase-m3-core-infra`
- **Runlog:** create `.claude/runlog/m3-core-infra-runlog.md` if absent; append the bm-cut entry; commit + push on the **new branch** `phase-m3-core-infra` (per bm-cut.md Phase 5, the runlog commit is the first commit on the new branch)
- **DQ mid-task discipline:** if a blocker arises, write a `kind: "blocker"` DQ entry (use `bash scripts/brehon/dq-v3-new-entry.sh` for the id), commit + push, and stop

---

## 5. Success signals

- `git ls-remote origin refs/heads/phase-m3-core-infra` returns non-empty (branch exists on origin)
- `gh api repos/barrie-cork/lemmy/branches/phase-m3-core-infra --jq '.name + " @ " + .commit.sha[0:9]'` confirms the branch
- Runlog entry appended + committed + pushed on `phase-m3-core-infra`
- Push to origin succeeded with upstream tracking set

---

## 6. Out of scope

- Editing impl code, plan files, design docs, `services/bridge/**`, or any file in the never-touch list
- Opening a PR (that comes at phase close)
- Merging or rebasing anything
- Authoring or editing the plan (it already exists at `bbbae593e`)

---

## 7. KNOWN harness limitation — bm-cut creates divergence (do NOT merge back)

**bm-cut creates a divergence point, NOT a feature branch.** `phase-m3-core-infra` must **NEVER** be merged back into `governance-v0` by the finalize agent. The daemon's generic post-job finalize may wrongly run `git merge --no-ff phase-m3-core-infra` INTO daemon-local `governance-v0`. Confirmed 2× (v1-AD-e #282 + v1-ship-1), and the bm-pr variant of this footgun fired on m3-core-entry-kinds (PR #200 auto-merged) — so the advisor's post-task daemon-trunk verification is MANDATORY this phase.

- **You (the bm-task worker):** just cut + push the branch + push the runlog. Do NOT merge anything. If the runlog write is blocked by the CC `.claude/**` sensitive-file gate, stop and report.
- **The advisor (post-task):** will verify daemon-local trunk as a ROUTINE step (`ssh homeserver 'cd /srv/brehon-fork && git log governance-v0 --oneline -1'`) and recover via `git update-ref` + push if a spurious merge landed.

---

## HANDOVER

```yaml
HANDOVER:
  task: m3-core-infra-bm-cut
  filesCreated: [.claude/runlog/m3-core-infra-runlog.md]
  filesModified: []
  keyDecisions:
    - "phase-m3-core-infra cut from governance-v0 @ bbbae593e or later"
    - "Plan ALREADY EXISTS (.claude/PRPs/plans/m3-core-infra.plan.md @ bbbae593e) — normal plan-presence check applies; gate-1 approved"
    - "MEDIUM complexity (score 13, proceed-as-one per user 2026-06-18); 3 validation surfaces: Postgres/Windows + bridge/Linux + Docker deploy-smoke"
    - "7 tasks + retro: Task0 barrier; Task1[P](Postgres) + Task2[P](bridge); Task3 serial-after-2; Tasks4-6 barriers"
    - "ADR-015 seam SPLIT: allocator callsite binary-side (Task4), JWT sub=pseudonym bridge-side (Task3); end-to-end fetch is Phase 3"
    - "Bridge compiles Linux-only (cargo-linux.sh --manifest-path); Tasks 2,3 write validate-pending-laptop-linux DQ"
    - "rtc_enabled defaults false; row-seed migration (NOT schema); governance_log.rs UNTOUCHED"
  notes: "bm-cut only — no PR yet. Mode B lane (no laptop worktree). Next: /auto-phase m3-core-infra --start-from impl-cohort-1."
```

Brief complete. Dispatch as:

```
[role:bm-task] m3-core-infra bm-cut — see .claude/PRPs/briefs/m3-core-infra-bm-cut-1.md
```

Base branch: `governance-v0`
