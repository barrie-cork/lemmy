---
role: bm-task
verb: bm-cut
phase: m2-late-1
pr_number: null
created: 2026-06-07
related_dq: f58698282815-001
---

# BM-cut brief — m2-late-1 (B-publish additive machinery, T0–T4)

**Role:** `[role:bm-task]`
**Phase:** `m2-late-1` (branch `phase-m2-late-1`)
**Authored:** 2026-06-07
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base:** `governance-v0` HEAD at task-execution time (`df0ef53f4` or later)
**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". No laptop-side phase worktree; advisor drives via Junior dispatch from canonical `brehon-fork`. impl-task briefs author on `governance-v0` + trunk→phase sync.

> **Context:** m2-late (B-publish sanction propagation) was split per resolved
> DQ `f58698282815-001` (complexity 13 > Sonnet threshold 8, user-confirmed
> 2026-06-07). This is the FIRST sub-phase: **m2-late-1 = T0–T4**, the
> additive machinery — `SanctionKind` enum + migration + Diesel models +
> governance-log consts + exhaustive `SanctionAction→SanctionKind` map +
> the `sanction_publisher` module. **All additive: the publisher compiles
> but is never called; behaviour is unchanged.** m2-late-2 (T5–T9: wire the
> spawn site live + startup seed + bridge endpoint + e2e + retro) is cut as a
> separate branch AFTER m2-late-1 ships. The governing plan
> `.claude/PRPs/plans/m2-late.plan.md` covers BOTH sub-phases; the split
> boundary is documented in plan §19 + §5.1.

---

## 1. Role + dispatch line

```
[role:bm-task] m2-late-1 bm-cut — see .claude/PRPs/briefs/m2-late-1-bm-cut-1.md
```

You are the **bm-task** subagent (Haiku 4.5 — git/yq/gh ops, no heavy reasoning). Execute the branch-manager verb `bm-cut` per `.claude/commands/bm/bm-cut.md`, step by step.

---

## 2. Scope

**Produce:** phase branch `phase-m2-late-1` cut from `governance-v0` HEAD, pushed to `origin`, with upstream tracking set.

- **Phase:** `m2-late-1` (branch `phase-m2-late-1`)
- **PR number:** none (bm-cut — PR doesn't exist yet)
- **Trunk SHA:** `df0ef53f4` on `governance-v0` (or later — base is HEAD at execution time)

**Explicit boundaries:**
- Cut branch `phase-m2-late-1` from `governance-v0` at its current HEAD
- Push with `git push -u origin phase-m2-late-1`
- Verify the governing plan file `.claude/PRPs/plans/m2-late.plan.md` is present on the new branch (NOTE: the plan filename is `m2-late.plan.md`, NOT `m2-late-1.plan.md` — a single plan governs both split sub-phases; do NOT STOP/file-DQ for a missing `m2-late-1*.plan.md`)
- Write a one-line runlog entry to `.claude/runlog/m2-late-runlog.md` (create if absent): `bm: bm-cut complete — phase-m2-late-1 cut from governance-v0 @ <SHA>`
- Commit + push the runlog entry on the new branch (per bm-cut.md Phase 5)

**Do NOT:**
- Open a PR (that comes at phase close after all impl tasks)
- Modify any file under `crates/`, `migrations/`, `docs/`, `services/bridge/`, `.claude/PRPs/plans/`, `.claude/PRPs/prds/`
- Merge or rebase anything
- Cut `phase-m2-late-2` (that is a separate later bm-cut after this sub-phase ships)

---

## 3. Required reading

1. `.claude/commands/bm/bm-cut.md` — the canonical bm-cut procedure; follow it step by step (note **Phase 8** finalize hazard)
2. `.claude/rules/branch-manager.md` — file-ownership boundaries and pre-cut checklist
3. `.claude/rules/phase-branch.md` — branch naming convention and pre-cut verifications
4. `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every `gh` call

---

## 4. Constraints

- **Branch name must be exactly** `phase-m2-late-1` (no variant; NOT `phase-m2-late`)
- **Base must be** `governance-v0` (never `main`)
- **Pre-cut checks** (per `branch-manager.md`):
  1. `git status --short` on `governance-v0` — must be clean
  2. `git fetch origin && git log governance-v0..origin/governance-v0 --oneline` — must be empty (governance-v0 in sync)
  3. Governing plan file `.claude/PRPs/plans/m2-late.plan.md` must be present on `governance-v0` (the split sub-phase shares this plan — see §2 boundary note)
- **Push with upstream tracking:** `git push -u origin phase-m2-late-1`
- **Runlog:** create `.claude/runlog/m2-late-runlog.md` if absent; append the bm-cut entry; commit + push on the **new branch** `phase-m2-late-1` (per bm-cut.md Phase 5, the runlog commit is the first commit on the new branch)
- **DQ mid-task discipline:** if a blocker arises, write a `kind: "blocker"` DQ entry (use `bash scripts/brehon/dq-v3-new-entry.sh` for the id), commit + push, and stop

---

## 5. Success signals

- `git ls-remote origin refs/heads/phase-m2-late-1` returns non-empty (branch exists on origin)
- `gh api repos/barrie-cork/lemmy/branches/phase-m2-late-1 --jq '.name + " @ " + .commit.sha[0:9]'` confirms the branch
- Governing plan file `.claude/PRPs/plans/m2-late.plan.md` present on the new branch
- Runlog entry appended + committed + pushed on `phase-m2-late-1`
- Push to origin succeeded with upstream tracking set

---

## 6. Out of scope

- Editing impl code, plan files, design docs, `services/bridge/**`, or any file in the never-touch list
- Opening a PR (that comes at phase close)
- Merging or rebasing anything
- Cutting `phase-m2-late-2`

---

## 7. KNOWN harness limitation — bm-cut creates divergence (do NOT merge back)

**bm-cut creates a divergence point, NOT a feature branch.** `phase-m2-late-1` must **NEVER** be merged back into `governance-v0` by the finalize agent. The daemon's generic post-job finalize may wrongly run `git merge --no-ff phase-m2-late-1` INTO daemon-local `governance-v0`. Confirmed 2× (v1-AD-e #282 + v1-ship-1).

- **You (the bm-task worker):** just cut + push the branch + push the runlog. Do NOT merge anything. If the runlog write is blocked by the CC `.claude/**` sensitive-file gate, stop and report.
- **The advisor (post-task):** will verify daemon-local trunk as a ROUTINE step (`ssh homeserver 'cd /srv/brehon-fork && git log governance-v0 --oneline -1'`) and recover via `git update-ref` + push if a spurious merge landed.

---

## HANDOVER

```yaml
HANDOVER:
  task: m2-late-1-bm-cut
  filesCreated: [.claude/runlog/m2-late-runlog.md]
  filesModified: []
  keyDecisions:
    - "phase-m2-late-1 cut from governance-v0 @ df0ef53f4 or later"
    - "m2-late SPLIT per DQ f58698282815-001 (complexity 13>8, user-confirmed 2026-06-07): m2-late-1 = T0-T4 additive machinery (publisher compiles, never called, behaviour unchanged); m2-late-2 = T5-T9 wire-live + e2e + retro (separate later branch)"
    - "Single governing plan m2-late.plan.md covers both sub-phases; do NOT expect m2-late-1.plan.md"
    - "Pre-Shape-G (cargo on laptop via validate-pending-laptop DQ)"
  notes: "bm-cut only — no PR yet. Mode B lane (no laptop worktree). MiniMax trial T5,T6 designated but SUSPENDED (memory 827) — neither is in m2-late-1 anyway (both in m2-late-2)."
```

Brief complete. Dispatch as:

```
[role:bm-task] m2-late-1 bm-cut — see .claude/PRPs/briefs/m2-late-1-bm-cut-1.md
```

Base branch: `governance-v0`
