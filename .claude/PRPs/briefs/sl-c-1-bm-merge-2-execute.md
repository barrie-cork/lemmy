# bm-merge brief (continuation) — execute merge PR #121 phase-v1-SL-c-1 → governance-v0

[role:bm-task] sl-c-1-bm-merge-2-execute — execute confirmed merge

## §0. Context

Junior bm-task #149 ran the bm-merge pre-merge gate (Phases 1–3 of `.claude/commands/bm/bm-merge.md`) on 2026-05-08. **All 6 gate checks passed:**

| Check | Result |
|---|---|
| PR state (OPEN, ready) | ✓ |
| Findings clean (zero CR findings) | ✓ |
| `mergeStateStatus` CLEAN | ✓ |
| CI checks all green | ✓ |
| DQ pending mentioning PR | 0 ✓ |
| CR re-poll since last commit | up-to-date ✓ |

Task #149 then surfaced the §4 final-confirm question to the user (per `branch-manager.md` autonomy rule: `gh pr merge` is "Manual: yes — confirm first").

**The user confirmed the merge** in the advisor session at 2026-05-08 (after task #149 exited). This continuation brief carries the relayed confirmation and instructs **Phases 5–9 only** (skip the gate; it already passed).

**§0-equivalent stricter constraints from sibling brief `sl-c-1-bm-merge-1.md` still apply:**
1. NEVER create a `temp-*` or `temp-bm-*` or any merge-staging branch.
2. NEVER push to `governance-v0` via `git push` other than the runlog commit at §8.
3. NEVER touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `.claude/PRPs/prds/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`.
4. NEVER execute `gh pr merge --squash` or `gh pr merge --rebase`. Use `--merge` only.
5. NEVER post a PR comment.
6. NEVER fire a Telegram ping.
7. NEVER re-run the gate. The gate already passed in task #149.

If you find yourself wanting to do any of items 1-7, **STOP, file a `kind: "blocker"` DQ entry from `from: "bm"` citing this brief's §0**, and exit non-zero.

## §1. Scope

Execute Phases 5–9 of `.claude/commands/bm/bm-merge.md`:

- §5 Execute merge command
- §6 Post-merge bookkeeping (fetch + checkout + ff-only pull)
- §7 Findings YAML — N/A (zero CR findings, no YAML exists; SKIP)
- §8 Append runlog entry
- §9 Output

## §2. Required reading

1. `.claude/rules/branch-manager.md` — autonomy bound (the user confirm has been collected by the advisor; this dispatch is the post-confirm execute).
2. `.claude/rules/phase-branch.md` — `--merge` not `--squash`; task-per-commit history.
3. `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory.
4. `.claude/commands/bm/bm-merge.md` — Phases 5–9.

## §3. Execute merge

```bash
gh pr merge 121 --repo barrie-cork/lemmy --merge --delete-branch
```

If non-zero exit → STOP and surface error. Do NOT retry, do NOT improvise alternative merge strategies.

Capture `gh` stdout + stderr; the merge SHA appears in the success message.

## §4. Post-merge bookkeeping

```bash
git fetch origin
git checkout governance-v0
git pull --ff-only origin governance-v0
git log -1 --oneline governance-v0
```

Verify the merge commit is on trunk. The merge commit subject will be `Merge pull request #121 from barrie-cork/phase-v1-SL-c-1`. Capture the merge SHA (first 9 chars).

If `git pull --ff-only` errors with "non-fast-forward" → STOP and surface (means EliteDesk-side state diverged). Do NOT force-pull.

## §5. Runlog append

Append to `.claude/runlog/bm-runlog.md`:

```markdown
## bm: merge — <ISO timestamp UTC>
- **PR:** #121 (Phase v1-SL-c-1 — sponsor_liability_grace scheduler module + clokwerk wiring)
- **base ← head:** governance-v0 ← phase-v1-SL-c-1
- **merge sha:** <merge-commit-SHA>
- **remote branch deleted?** yes (--delete-branch)
- **trunk position:** <new-trunk-SHA> (Merge pull request #121 from barrie-cork/phase-v1-SL-c-1)
- **findings YAML archived:** N/A (zero CR findings — clean merge)
- **stricter constraints honored:** yes per sl-c-1-bm-merge-1.md §0 + sl-c-1-bm-merge-2-execute.md §0 — no temp branches, no governance-v0 push beyond runlog commit, --merge only
- **gate ran in:** task #149 (gate-only); execute ran in: task #<this-task-id>
```

Commit + push the runlog edit on `governance-v0`. Commit subject (must match `^chore\(bm\)`):

```
chore(bm): bm-merge merged PR #121 (sl-c-1) at <merge-sha-short>
```

Single commit. No other writes to governance-v0.

## §6. What success looks like

After this dispatch:
- PR #121 state = `MERGED` per `gh pr view 121`.
- `origin/governance-v0` HEAD = the new merge commit `Merge pull request #121 from barrie-cork/phase-v1-SL-c-1`.
- `origin/phase-v1-SL-c-1` deleted.
- `.claude/runlog/bm-runlog.md` has a `## bm: merge` block appended.
- A single BM commit on `governance-v0` with subject `chore(bm): bm-merge merged PR #121 (sl-c-1) at <merge-sha-short>`, pushed to origin.
- Output line: `c-1 merged. Advisor can now bm-cut phase-v1-SL-c-2 from new governance-v0 tip per c-2 plan §0.`
- **NO temp branches created, NO direct governance-v0 push beyond the runlog commit, NO touching of crates/migrations/tests.**

## §7. Hard refusals (recap)

If you find yourself contemplating any of these, STOP + file `kind: "blocker"` DQ entry from `from: "bm"`:
- Creating a temp branch (any `temp-*`, `merge-*`, etc).
- `git push` to `governance-v0` directly other than the single runlog commit.
- Touching files outside `.claude/runlog/bm-runlog.md`.
- Using `--squash` or `--rebase` instead of `--merge`.
- Re-running `gh pr merge` after a non-zero exit.
- Posting a PR comment or Telegram ping.
- Re-running CR poll, gate checks, or AskUserQuestion (the user already confirmed).
