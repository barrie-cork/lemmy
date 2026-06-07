---
name: validate-pending-laptop runs from a worktree, never a bare checkout in the canonical tree
description: When the laptop advisor runs a validate-pending-laptop DQ whose branch differs from the canonical checkout's branch, it must resolve a WORKTREE on entry.branch (Mode A lane, or a Mode B throwaway), never `git checkout <phase-branch>` in canonical brehon-fork. A bare checkout violates multi-lane hard refusal #1 AND silently reverts to governance-v0 across a /compact boundary — working-tree state is conversation-invisible. The 2026-06-07 m2-late-1 T1 incident ran 8 cargo/migration steps against the wrong tree because a pre-compact checkout had reverted.
type: feedback
---

## TL;DR

A `validate-pending-laptop` DQ entry names the phase branch its commands must run
against (`entry.branch`). When that branch differs from the canonical `brehon-fork`
checkout's current branch (the common Mode B case), the advisor must run validation
from a **worktree on `entry.branch`** — the Mode A lane worktree if one exists, else a
**throwaway worktree** (`git worktree add ../brehon-fork-validate-<id> origin/<branch>`).
It must **never** do a bare `git checkout <phase-branch>` in the canonical tree.

Two independent reasons the bare checkout is wrong:

1. **It violates `multi-lane-worktree.md` hard refusal #1** ("Never `git checkout
   phase-*` inside `brehon-fork`") — a destructive cross-lane operation on the shared
   working tree.
2. **It is not durable across a `/compact` boundary.** A `git checkout` mutates
   *working-tree* state, which is invisible to the conversation. After a `/compact` (or
   any session-context reset), the resumed session has no record the checkout happened,
   the working tree has reverted to `governance-v0`, and the advisor proceeds believing
   it is on the phase branch while actually on trunk.

A worktree HEAD is **independent** of the canonical tree's HEAD, so it survives a
compact untouched. Combined with a per-command branch/SHA assertion, the
wrong-branch failure class becomes structurally impossible rather than merely
detectable.

## Why this matters (m2-late-1 T1 incident, 2026-06-07)

The T1 `validate-pending-laptop` DQ (`001f1c47c5dc-001`, branch `phase-m2-late-1`)
added a migration that existed only on the phase branch. The advisor:

- ran a plain `git checkout phase-m2-late-1` in the **canonical** `brehon-fork` tree
  (Mode B — no lane worktree existed),
- hit a `/compact` + `/model` swap, which reverted the working tree to `governance-v0`,
- then ran the migration runner, `migrate-roundtrip.sh`, `diesel print-schema`, and a
  dev-DB inspection — **all against `governance-v0`**, where the migration directory
  was absent.

Every downstream symptom was a consequence of the one wrong-branch error:
- `STATUS_DLL_NOT_FOUND` (wrapper axis — separate lesson, see cross-refs),
- `migrate-roundtrip.sh` → "no new migrations vs governance-v0; exit 0" (this is the
  **correct** output when run from the wrong branch — it diffs `entry.branch` HEAD vs
  `origin/governance-v0`; from `governance-v0` there is nothing new — and is therefore a
  **wrong-branch signal, not a no-op to route around**),
- regenerated schema with no `sanction_*` tables,
- dev DB missing the migration.

The root cause was undeniable only at step 11 (`git branch --show-current` →
`governance-v0`). Eight steps of cargo/migration forensics were spent on what one
correct step-1 worktree resolution would have prevented. Full trace:
`.claude/PRPs/debug/m2-late-1-t1-validate-pending-advisor-session-trace.md`.

The deeper finding: the handler spec *itself* used to prescribe `git checkout
origin/<branch>` in the canonical tree, which **contradicted** multi-lane hard refusal
#1. An agent following both rules faithfully was stuck. The fix (commit `24d457f61`)
rewrote the handler to use a worktree and carved out that path in the hard refusal, so
the two rules stop conflicting.

## When to apply

The gate fires whenever the advisor (laptop session) is about to run a
`validate-pending-laptop` / `*-laptop-e2e` / `*-laptop-linux` DQ entry whose
`entry.branch` differs from the canonical checkout's current branch.

Triggering signatures:
- The DQ `branch` field is a `phase-*` branch and the canonical `brehon-fork` is on
  `governance-v0` (the default Mode B situation).
- The session has crossed (or may cross) a `/compact`, session-end, or `/model` swap
  between checking out the branch and running the commands.

Does NOT fire when:
- A Mode A lane worktree already exists on `entry.branch` (`brehon-fork-<lane>`) and the
  advisor session's CWD **is** that worktree — there is nothing to check out; the lane
  worktree is the validation surface (still ff it to `origin/<branch>` first).
- The entry is a Shape-G `validate-pending` (runs on GitHub Actions via ci-watcher, not
  the laptop) — no local tree is involved.

## How to apply

1. **Resolve a worktree on `entry.branch`** (handler Sequence step 1, per
   `advisor-validation.md §"validate-pending-laptop handler"`):
   - **Mode A:** `cd` to `brehon-fork-<lane>`; ff to `origin/<branch>`
     (`git -C <lane> merge --ff-only origin/<branch>`).
   - **Mode B:** `git fetch origin <branch>` then
     `git worktree add ../brehon-fork-validate-<entry.id> origin/<branch>`, then run the
     **lane bootstrap** (else cargo fails on `lemmy_email`: `git -C <wt> submodule update
     --init --recursive` + copy `.mcp.json` / `.env` / `.claude/settings.local.json`).
     Remove the throwaway after (`git worktree remove`).
2. **Assert the branch/SHA before EACH command** (defends against mid-chain compact
   reversion):
   ```bash
   ACTUAL=$(git -C <wt> rev-parse HEAD); EXPECTED=$(git rev-parse origin/<entry.branch>)
   [ "$ACTUAL" = "$EXPECTED" ] || { echo "BRANCH MISMATCH — STOP"; exit 1; }
   ```
3. **For migration tasks, snapshot `SELECT MAX(version) FROM __diesel_schema_migrations`
   before+after** and confirm it advanced — a silent `exit 0` on "nothing to apply" is
   ambiguous (wrong branch OR already-applied); the delta disambiguates.

## Hard refusals

- **NEVER `git checkout <phase-branch>` in the canonical `brehon-fork` tree** to run
  validation. Use a worktree (Mode A lane or Mode B throwaway). The checkout is both a
  cross-lane hazard and compact-non-durable.

- **NEVER trust a pre-compact checkout.** If the session has crossed a `/compact`,
  `/model` swap, or session boundary, re-assert `HEAD == origin/<entry.branch>` before
  running anything — do not assume the working tree is where you left it.

- **NEVER read "no new migrations vs governance-v0; exit 0" as a no-op to work around.**
  From a phase-branch worktree it means the migration is genuinely absent (a real
  problem); from the wrong branch it means you are on `governance-v0` (a branch error).
  Either way it is a signal to investigate the branch, not a green light.

## Cross-references

- `.claude/refs/advisor-validation.md §"validate-pending-laptop handler"` — the handler
  Sequence (worktree step 1, per-command assertion step 2, wrapper substitution step 3)
  + §"Why a worktree, not a checkout" (the RCA inline).
- `.claude/rules/multi-lane-worktree.md` hard refusal #1 — the rule the bare checkout
  violated; now carves out the handler's throwaway-worktree path.
- `.claude/lessons/feedback_validate_pending_laptop_must_use_wrapper.md` — the **wrapper
  axis** (Windows libpq.dll / `STATUS_DLL_NOT_FOUND`); distinct from this **where-it-runs
  axis**. Both bit the T1 run; both are now handler steps.
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — the **who-runs
  axis** (impl-task writes the DQ + stops; laptop runs cargo).
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` — the bootstrap
  the Mode B throwaway must run (submodules + gitignored config copies) or cargo fails on
  `lemmy_email`.
- `.claude/lessons/feedback_falsifiable_hypothesis_before_structural_fix.md` — sibling
  discipline: each downstream T1 symptom (DLL, "no new migrations") was a locally
  plausible hypothesis diagnosed in isolation; the meta-lesson is to check "am I on the
  right branch?" before diagnosing per-command failures.
- `.claude/PRPs/debug/m2-late-1-t1-validate-pending-advisor-session-trace.md` — the
  step-by-step trace. Fix: commit `24d457f61` (S1+S2+S4b+S5).
