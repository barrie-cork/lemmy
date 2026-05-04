---
description: |
  BM — final pre-merge gate; ASKS before gh pr merge --merge
argument-hint: |
  <PR#>
disable-model-invocation: true
---

# /bm-merge — final merge gate (no squash, no auto)

**Input**: $ARGUMENTS — PR number (e.g. `87`)

**Dispatcher → `branch-manager` subagent.** This command delegates
execution to the `branch-manager` subagent, which runs in its own
isolated context window. The subagent reads the operational script
below and follows it step by step.

Invoke:

> Use the `branch-manager` subagent to run `bm-merge`. Arguments:
> $ARGUMENTS (PR number). Follow the phases in
> `.claude/commands/bm/bm-merge.md` — run the full pre-merge gate
> (findings YAML counters clean, mergeStateStatus CLEAN, CI all green,
> no DQ mentioning this PR, re-poll CR if commits since last poll).
> ASK via AskUserQuestion before `gh pr merge --merge --delete-branch`.
> Use `--merge` not `--squash` (task-per-commit history). After merge,
> fast-forward local governance-v0, archive findings YAML with
> `merged_at`/`merge_commit`. Return the merge SHA + trunk position +
> "Next suggested" line.

The parent (impl) session should call `Agent(subagent_type="branch-manager", model="sonnet", prompt=<the above>)`. Pre-merge gate checks + final confirm: mostly mechanical (YAML counters, mergeStateStatus, CI, DQ scan), but the stakes are high (merge is irrevocable). Sonnet 4.6 — not Haiku (gate checks must be robust), not Opus (user's AskUserQuestion confirm is the final judgment gate).

---

## Operational script (for the subagent)

The BM agent runs the final pre-merge gate and, if all checks pass,
asks the user to confirm the merge. Always uses `--merge` (not
`--squash` — per `phase-branch.md`, the task-per-commit history is
load-bearing for retros).

**Reads:** `.claude/rules/branch-manager.md`,
`.claude/rules/phase-branch.md`,
`.claude/PRPs/reviews/SCHEMA.md`.

---

## Phase 1 — Validate input

```bash
gh pr view {N} --repo barrie-cork/lemmy \
  --json number,title,state,reviewDecision,mergeStateStatus,statusCheckRollup,mergeable
```

| State | Action |
|---|---|
| `MERGED` | INFO: "Already merged" then STOP |
| `CLOSED` | STOP: "Closed PR — cannot merge" |
| `OPEN` (draft) | STOP: "Draft PR — mark ready first" |
| `OPEN` (ready) | PROCEED |

---

## Phase 2 — Pre-merge gate

Run all of these and require all-green. Any FAIL → STOP and report.

### 2.1 Findings YAML clean

```bash
yq '.counters' .claude/PRPs/reviews/pr-{N}-findings.yaml
```

| Check | Required |
|---|---|
| `critical.open == 0` | YES (per `feedback_coderabbit_block_merge_critical.md`) |
| `major.open == 0` | YES (or all in `bucket: rebut|carry-forward|wont-fix` with rationale) |
| **No `fix-in-pr` rows remain (all `fix-in-pr` work has been moved to `bucket: done` with an `addressed_in` SHA)** | YES |
| `recommendation: approve` | YES |

<!-- cr-2 (closes #88): the findings schema reserves `addressed_in` for
     `bucket: done`, not for `bucket: fix-in-pr` (which is "still pending
     a code change"). The merge gate now requires zero remaining `fix-in-pr`
     rows; any addressed work must have moved to `bucket: done` carrying
     its `addressed_in` SHA. -->

If any check fails, STOP and print the exact rows blocking merge:

```bash
# Any unaddressed fix-in-pr work — must move to bucket: done before merge
yq '.findings[] | select(.bucket=="fix-in-pr")' \
  .claude/PRPs/reviews/pr-{N}-findings.yaml
# Any done-bucket row missing its commit SHA — schema-invalid
yq '.findings[] | select(.bucket=="done" and .addressed_in==null)' \
  .claude/PRPs/reviews/pr-{N}-findings.yaml
```

### 2.2 GitHub merge state

```bash
gh pr view {N} --repo barrie-cork/lemmy --json mergeStateStatus,mergeable
```

| `mergeStateStatus` | Action |
|---|---|
| `CLEAN` | OK |
| `BEHIND` | STOP: "Branch behind base — rebase first via separate ASK" |
| `BLOCKED` | STOP: "Branch protection blocks merge — check settings" |
| `DIRTY` | STOP: "Merge conflicts — resolve on phase branch first" |
| `UNKNOWN` | RETRY in 10s (GitHub mid-compute), then STOP if still UNKNOWN |
| `HAS_HOOKS` | OK |
| Other | STOP: "Unexpected merge state — investigate" |

### 2.3 CI checks all passing

```bash
gh pr view {N} --repo barrie-cork/lemmy \
  --json statusCheckRollup \
  --jq '.statusCheckRollup[] | select(.conclusion != "SUCCESS" and .conclusion != null) | {name, conclusion, detailsUrl}'
```

If any check is FAILURE / CANCELLED / TIMED_OUT: STOP and list them.

If any check is PENDING / null / still-running: STOP and wait. Re-poll
every 30-60 seconds until all required checks settle, then re-evaluate
the gate. Never offer "proceed anyway" — merging while a required check
is unsettled races CI and can land a PR before a blocking signal arrives.

<!-- cr-3 (closes #88): the prior "proceed anyway" branch was unsafe —
     a merge gate must wait for all required checks to settle, not race them. -->


### 2.4 No DQ pending that mentions this PR

```bash
yq '.pending[] | select(.question | contains("PR #{N}") or contains("pr-{N}") or contains("pr_{N}"))' \
  .claude/decision-queue.json
```

If any pending DQ entry references this PR: STOP, "Resolve DQ first."

### 2.5 Recent CR re-review absorbed

```bash
yq '.last_poll_at' .claude/PRPs/reviews/pr-{N}-findings.yaml
git log --since="{last_poll_at}" --oneline {head-branch} | head
```

If commits exist on the head branch SINCE the last CR poll: STOP,
"Re-poll CR before merge — `/bm-poll-cr {N}`." (Otherwise we may
miss a CR finding that landed after the last poll.)

---

## Phase 3 — Print pre-merge summary

```markdown
## /bm-merge — pre-merge summary for PR #{N}

**PR:** {title}
**Base ← Head:** {base} ← {head}
**Diff:** +{add}/-{del} ({changedFiles} files, {commits} commits)

### Findings (final state)
| Bucket | Critical | Major | Medium | Low | Nit |
|---|---|---|---|---|---|
| fix-in-pr | 0 | {N} (rebut/cf/wf) | … | … | … |
| done | {N} | {N} | … | … | … |
| rebut | {N} | {N} | … | … | … |
| carry-forward | {N} | {N} | … | … | … |
| wont-fix | {N} | {N} | … | … | … |

### Gate results
| Check | Result |
|---|---|
| Critical findings open | 0 ✓ |
| Major findings open in fix-in-pr | 0 ✓ |
| All fix-in-pr addressed | ✓ |
| `mergeStateStatus` | CLEAN ✓ |
| CI checks | all green ({N}/{N}) ✓ |
| DQ pending mentioning PR | 0 ✓ |
| CR re-poll since last commit | up to date ✓ |

### Merge command
```bash
gh pr merge {N} --repo barrie-cork/lemmy --merge --delete-branch
```

(`--merge` not `--squash` per `phase-branch.md` — task-per-commit
history is load-bearing for retros.)

(`--delete-branch` removes the remote `phase-*` branch after merge.
Local branch survives until you `git branch -D` it.)
```

---

## Phase 4 — ASK USER to confirm merge

This is the most-irreversible action BM takes. ASK explicitly:

```markdown
**Confirm merge of PR #{N}?**

This will:
1. Merge {head} into {base} via `--merge` (preserves task-per-commit history)
2. Delete the remote branch `origin/{head}` via `--delete-branch`
3. Local branch `{head}` survives — you can `git branch -D {head}` later

Reply `confirm` to merge, `dry-run` to print the gh command without
executing, or any other input to abort.
```

If user replies `dry-run`: print the `gh pr merge` command, do not
execute, log to runlog as "merge dry-run."

If user replies anything other than `confirm` or `dry-run`: ABORT,
log to runlog as "merge aborted by user."

---

## Phase 5 — Execute merge

If confirmed:

```bash
gh pr merge {N} --repo barrie-cork/lemmy --merge --delete-branch
```

Wait for the command to return. Do NOT continue if `gh` errors.

---

## Phase 6 — Post-merge bookkeeping

After successful merge:

```bash
git fetch origin
git checkout governance-v0
git pull --ff-only origin governance-v0
git log -1 --oneline
```

Verify the merge commit is on trunk now.

If the impl session is still on the merged head branch, leave it
alone — don't auto-checkout for them. Just print a note that the
local branch may be deleted when convenient.

---

## Phase 7 — Update findings YAML to archived state

Add to the YAML:

```yaml
merged_at: 2026-04-23T15:30:00Z
merge_commit: <sha>
final_recommendation: approve
```

Bucket promotion: every `fix-in-pr done` row's `bucket` stays `done`.
Counters refreshed. (The YAML stays gitignored — it's archival in the
local sense only.)

---

## Phase 8 — Append to runlog

```markdown
## bm: merge — {ISO timestamp}
- **PR:** #{N} ({title})
- **base ← head:** {base} ← {head}
- **merge sha:** {sha}
- **remote branch deleted?** yes
- **trunk position:** {trunk-sha} ({short-msg})
- **findings YAML archived:** .claude/PRPs/reviews/pr-{N}-findings.yaml
```

---

## Phase 9 — Output

```markdown
## /bm-merge complete

**PR #{N}:** MERGED
**Merge SHA:** {sha}
**Trunk now at:** {trunk-sha}

### Cleanup done
- Remote branch `origin/{head}` deleted ✓
- Findings YAML archived with `merged_at` + `merge_commit` ✓

### What you may want to do next
- `git branch -D {head}` — remove local branch (BM does NOT auto-delete local)
- Open the next sub-phase plan PR (out-of-scope for BM-merge)
- Send Telegram ping with `/bm-ping merge-ready` (asks first)
```

---

## Refusal cases

- Critical finding still open in `fix-in-pr` → STOP (per
  `feedback_coderabbit_block_merge_critical.md`).
- `mergeStateStatus` not CLEAN → STOP.
- CI checks failing → STOP.
- DQ pending mentions this PR → STOP.
- Commits since last CR poll → STOP, force re-poll.
- User reply other than `confirm` / `dry-run` → ABORT.

---

## What BM merge will NEVER do

- Use `--squash` (loses task-per-commit history per `phase-branch.md`).
- Force-merge past failing checks.
- Merge with open critical findings.
- Auto-merge without ASK (always confirms).
- Merge into `main` (only `governance-v0` is a valid base for v0/v1
  PRs; `main` is upstream-rebase only).
- Auto-`git branch -D` the local branch.

---

## See also

- `.claude/rules/branch-manager.md`
- `.claude/rules/phase-branch.md` — `--merge` not `--squash`
- memory `feedback_coderabbit_block_merge_critical.md` — critical = block
- memory `feedback_branch_manager_pm_split.md` — role split
- `.claude/PRPs/reviews/SCHEMA.md` — YAML schema
