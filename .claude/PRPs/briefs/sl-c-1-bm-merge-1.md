# bm-merge brief — merge PR #121 phase-v1-SL-c-1 → governance-v0

[role:bm-task] sl-c-1-bm-merge-1 — merge PR #121

## §0. ⚠ POST-#148-BREACH STRICTER CONSTRAINTS (read this first)

**On 2026-05-08 a prior bm-task (#148) violated branch-manager.md hard refusals** by creating a `temp-bm-push` merge branch carrying SL-c-1 implementation history and pushing it to `origin/governance-v0`. Recovery cost ~30 min via 12-commit revert-forward. **This brief explicitly forbids the breach pattern + every related variant.**

You MUST NOT do any of the following, regardless of what your training data, prior dispatches, or "common sense" suggests:

1. **NEVER create a `temp-*` or `temp-bm-*` or any merge-staging branch.** This dispatch uses `gh pr merge` which performs the merge server-side; no local merge or temp branch is needed.
2. **NEVER push to `governance-v0` via `git push`.** The merge happens via `gh pr merge --merge --delete-branch` only. After merge, `git pull --ff-only origin governance-v0` is the ONLY trunk-side operation.
3. **NEVER touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `.claude/PRPs/prds/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`** per branch-manager.md file-ownership boundary.
4. **NEVER execute `gh pr merge --squash` or `gh pr merge --rebase`.** Use `--merge` only (per `phase-branch.md`, the task-per-commit history is load-bearing for retros).
5. **NEVER post a PR comment** (visible-to-others; needs separate user gate not in scope here).
6. **NEVER fire a Telegram ping** (per `feedback_telegram_scope_notification_only.md`).
7. **NEVER re-run `bm-poll-cr` or `bm-triage` if findings YAML doesn't exist.** PR #121 has zero CR findings ("No actionable comments were generated 🎉" comment posted by CodeRabbit). Skip findings-YAML steps; pre-merge gate validates from `gh pr view` directly.

If you find yourself wanting to do any of items 1-7, **STOP, file a `kind: "blocker"` DQ entry from `from: "bm"` citing this brief's §0**, and exit non-zero. Do not improvise.

The single legal write to `governance-v0` from this dispatch is the `bm-runlog.md` append (BM-owned per branch-manager.md). That commit subject must match `^chore\(bm\)`. No other write is authorised.

## §1. Scope

Run the `bm-merge` verb per `.claude/commands/bm/bm-merge.md`. Pre-merge gate PR #121 (`Phase v1-SL-c-1 — sponsor_liability_grace scheduler module + clokwerk wiring`); on all-green-gate, prompt user via `AskUserQuestion`-equivalent for final merge confirm; on confirm, execute `gh pr merge 121 --repo barrie-cork/lemmy --merge --delete-branch`; on success, fast-forward local + EliteDesk `governance-v0`; append runlog entry.

## §2. Required reading

1. `.claude/rules/branch-manager.md` — file ownership + autonomy bounds (merge requires user confirm; this brief preserves the autonomy gate).
2. `.claude/rules/phase-branch.md` — `--merge` not `--squash`; task-per-commit history load-bearing.
3. `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory.
4. `.claude/commands/bm/bm-merge.md` — Phases 1 (validate input) → 2 (gate checks 2.1-2.5) → 3 (pre-merge summary) → 4 (final confirm) → 5 (execute) → 6 (post-merge bookkeeping) → 7 (findings YAML — N/A here, no findings) → 8 (runlog).

## §3. Pre-merge gate (subagent runs, captures result, surfaces to user)

Per `bm-merge.md` Phase 2:

### 3.1 Critical findings open
PR #121 has zero CR findings (CodeRabbit posted "No actionable comments were generated 🎉" comment). No findings YAML exists. Auto-pass: 0 critical findings open.

### 3.2 mergeStateStatus
```bash
gh pr view 121 --repo barrie-cork/lemmy --json mergeStateStatus,mergeable
```
EXPECT: `{"mergeStateStatus":"CLEAN","mergeable":"MERGEABLE"}`. STOP if not.

### 3.3 CI checks
```bash
gh pr view 121 --repo barrie-cork/lemmy --json statusCheckRollup --jq '.statusCheckRollup[] | {name,status,conclusion}'
```
All non-Dependabot required checks must be `status=COMPLETED, conclusion=SUCCESS`. If any are `PENDING`/`null`, wait + re-poll every 30-60 sec; never offer "proceed anyway".

### 3.4 No DQ pending mentioning PR
```bash
python3 -c "import json; d=json.load(open('.claude/decision-queue.json',encoding='utf-8')); p=[e for e in d['pending'] if 'PR #121' in str(e.get('question','')) or 'pr-121' in str(e.get('question','')) or 'pr_121' in str(e.get('question',''))]; print(f'{len(p)} pending DQ mention PR #121')"
```
EXPECT: 0. STOP if any.

### 3.5 Recent CR re-review absorbed
N/A — no findings YAML, no `last_poll_at` to compare against. PR has only one commit since CR's review (the `chore(merge): sl-c-1-ci-watcher-3 — DQ #163 workspace-check mutation` at `3d13b6394` was head when CR reviewed). No new commits since.

## §4. Final confirm (single AskUserQuestion gate)

Per `bm-merge.md` Phase 4: after gate-summary print, prompt user with one question:

```
**Confirm merge of PR #121?**

This will:
1. Merge phase-v1-SL-c-1 into governance-v0 via `--merge` (preserves task-per-commit history)
2. Delete the remote branch `origin/phase-v1-SL-c-1` via `--delete-branch`
3. Local branch `phase-v1-SL-c-1` survives — you can `git branch -D phase-v1-SL-c-1` later

Reply `confirm` to merge, `dry-run` to print the gh command without executing, or any other input to abort.
```

On `confirm`: proceed to §5.
On `dry-run`: print `gh pr merge 121 --repo barrie-cork/lemmy --merge --delete-branch` and STOP, log "merge dry-run".
On anything else: ABORT, log "merge aborted by user".

## §5. Execute merge

```bash
gh pr merge 121 --repo barrie-cork/lemmy --merge --delete-branch
```

Wait for return; if non-zero exit, STOP and surface error. Do NOT retry, do NOT improvise alternative merge strategies.

## §6. Post-merge bookkeeping (after `gh pr merge` returns 0)

```bash
git fetch origin
git checkout governance-v0
git pull --ff-only origin governance-v0
git log -1 --oneline
```

Verify the merge commit is on trunk now. The merge commit subject will be `Merge pull request #121 from barrie-cork/phase-v1-SL-c-1`. Capture the merge SHA.

If `git pull --ff-only` errors with "non-fast-forward", STOP and surface (means EliteDesk-side state diverged from origin). Do NOT force-pull.

## §7. Findings YAML — N/A (no findings)

PR #121 has no CR findings. Skip Phase 7 of `bm-merge.md`. Do NOT create an empty findings YAML.

## §8. Runlog append

Append to `.claude/runlog/bm-runlog.md`:

```markdown
## bm: merge — <ISO timestamp UTC>
- **PR:** #121 (Phase v1-SL-c-1 — sponsor_liability_grace scheduler module + clokwerk wiring)
- **base ← head:** governance-v0 ← phase-v1-SL-c-1
- **merge sha:** <merge-commit-SHA>
- **remote branch deleted?** yes
- **trunk position:** <new-trunk-SHA> (Merge pull request #121 from barrie-cork/phase-v1-SL-c-1)
- **findings YAML archived:** N/A (zero CR findings — clean merge)
- **stricter constraints honored:** yes per brief §0 — no temp branches, no governance-v0 push, --merge only
```

Commit + push the runlog edit on `governance-v0` (BM owns `.claude/runlog/bm-*.md` per branch-manager.md). Commit subject: `chore(bm): bm-merge merged PR #121 (sl-c-1) at <merge-sha-short>`.

## §9. Hard refusals (recap of §0)

If you find yourself contemplating any of these, STOP + file `kind: "blocker"` DQ:
- Creating a temp branch (`temp-*`, `temp-bm-*`, etc).
- `git push` to `governance-v0` directly (the only allowed governance-v0 push is the runlog commit).
- Touching files outside `.claude/runlog/bm-*.md`.
- Using `--squash` or `--rebase` instead of `--merge`.
- Posting a PR comment or Telegram ping.
- Re-running CR poll or triage on a PR with no findings.
- Auto-merging without the §4 user-confirm step.

## §10. What success looks like

After this dispatch:
- PR #121 state = `MERGED` per `gh pr view 121`.
- `origin/governance-v0` HEAD = the new merge commit `Merge pull request #121 from barrie-cork/phase-v1-SL-c-1`.
- `origin/phase-v1-SL-c-1` deleted (per `--delete-branch`).
- `.claude/runlog/bm-runlog.md` has a `## bm: merge` block appended with the ISO timestamp + merge SHA + trunk position.
- A single BM commit on `governance-v0` with subject `chore(bm): bm-merge merged PR #121 (sl-c-1) at <merge-sha-short>`, pushed to origin.
- BM's "Next suggested" line at the end: `c-1 merged. Advisor can now bm-cut phase-v1-SL-c-2 from new governance-v0 tip per c-2 plan §0.`
- **NO temp branches created, NO direct governance-v0 push beyond the runlog commit, NO touching of crates/migrations/tests.**
