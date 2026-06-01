---
description: BM — final pre-merge gate; ASKS before gh pr merge --merge
argument-hint: <PR#>
disable-model-invocation: true
---

# /bm-merge — final merge gate (no squash, no auto)

**Input**: $ARGUMENTS — PR number (e.g. `87`)

**Dispatcher → `branch-manager` subagent.** This command delegates
execution to the `branch-manager` subagent, which runs in its own
isolated context window. The subagent reads the operational script
below and follows it step by step.

## Two execution modes (per L15 from v1-SL-c-1 retro)

The merge gate is split into two halves:

- **Gate-side (Phases 1-4 below)** — read-only pre-merge checks,
  pre-merge summary, AskUserQuestion confirm. Under `/auto-phase`
  these run **inline in the advisor session** (no Junior dispatch);
  the advisor already has plan + brief context loaded, so a Junior
  duplicates ~80% of context boot for a single decision. Manual
  `/bm-merge` invocation outside `/auto-phase` still dispatches the
  full script to the BM subagent (gate + execute) — the dispatch line
  below covers that path.
- **Execute-side (Phases 5-9 below)** — actual `gh pr merge`, runlog
  commit, post-merge bookkeeping. Always runs on the Junior side
  (mutating actions stay in BM territory). Under `/auto-phase`, the
  advisor session queues a single bm-task brief carrying ONLY Phases
  5-9 (the brief MUST explicitly order the BM Junior's git sequence
  per the L14 fix below — see "Phase 5 — Execute merge" header).

This split is canonical from 2026-05-07 onward (post v1-SL-c-1 retro).
Anything that re-introduces a pre-confirm Junior dispatch is a process
regression and must be surfaced.

## L14 fix — explicit git sequence in the BM brief (REVISED 2026-05-18: runlog POST-merge)

Under `/auto-phase`, the bm-task brief that carries the execute-side MUST
contain an explicit, numbered git sequence in §4 Constraints. The runlog
COMPLETE entry is written **POST-merge**, NOT pre-merge:

```
1. git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0
2. Re-verify PR #<N> is MERGEABLE (gh pr view <N> --repo barrie-cork/lemmy --json mergeable,state). If CONFLICTING / not OPEN → STOP + surface (do NOT improvise).
3. gh pr merge <N> --repo barrie-cork/lemmy --merge --delete-branch
4. Wait for return. If exit non-zero → HARD REFUSAL: capture verbatim error, STOP, surface. NEVER retry, NEVER git push -f, NEVER hand-resolve.
5. POST-MERGE ONLY (merge succeeded): git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0. Edit .claude/runlog/bm-runlog.md — append the "## bm: merge COMPLETE" entry per Phase 8 template with the REAL merge sha (gh pr view <N> --json mergeCommit -q .mergeCommit.oid). git add → git commit -m "chore(bm): merge PR #<N> complete — runlog COMPLETE entry" → git push origin governance-v0.
6. L16: git ls-remote origin refs/heads/<head-branch>. If still present → run ONCE: gh api -X DELETE -H "Accept: application/vnd.github+json" /repos/barrie-cork/lemmy/git/refs/heads/<head-branch>. Re-check; if still present after the single attempt, note "manual branch-delete follow-up" in Phase 9 output and STOP (do NOT loop).
```

**Why POST-merge (revised — supersedes the prior pre-merge ordering):**
the original L14 ordering committed the runlog entry to `governance-v0`
*before* `gh pr merge`. But the **bm-pr** step also appends a runlog
entry, on the **phase branch**. Both branches then diverged on the
same append-only `.claude/runlog/bm-runlog.md` from their common
ancestor → the file conflicted → `mergeStateStatus: DIRTY` →
**the merge could not complete** (v1-ship-1-r2 Junior #322, 2026-05-18;
see `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md` + DQ
#265). Writing the runlog COMPLETE entry **after** `gh pr merge
--delete-branch` removes the conflict entirely: the phase branch is
gone, so there is no second branch to diverge against. The
audit-trail-durability concern the pre-merge ordering addressed is
covered instead by (a) the bm-pr "PR opened" entry already on record
before any merge attempt, and (b) the L14 belt-and-braces fallback.

**L14 belt-and-braces (unchanged — only the timing moved pre→post):**
if the BM Junior skips the POST-merge runlog COMPLETE commit (observed
— Junior #323 did), the advisor's post-merge tick authors a
`docs(advisor): L14 belt-and-braces — runlog COMPLETE re-apply` block
on `governance-v0` with the *verified real merge sha* (per
`.claude/refs/auto-phase.md` invariant 7).

**Stronger structural option (SHIPPED 2026-05-18):** a `merge=union`
driver in `.gitattributes` for `.claude/runlog/bm-runlog.md` — it is
an append-only ledger, so a union merge is semantically correct and
makes ALL runlog cross-branch conflicts structurally impossible (would
also have prevented this incident even with the old ordering). This is
now in place on `governance-v0` (`.gitattributes` line
`.claude/runlog/bm-runlog.md merge=union`); it is belt-and-braces
behind the POST-merge ordering above. Tracked in
`feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md`.

## Hard-refusal contract for the BM Junior

The bm-task subagent dispatched by this verb operates under
categorical hard refusals. The brief MUST state them in §4
Constraints; this spec states them here so they apply to every
dispatch regardless of brief author. Promoted from the v1-ship-1-r2
bm-merge-2 RETRY brief (2026-05-18) which encoded them explicitly
and executed cleanly in ~2 min (Junior #323) after the original
brief lacked them and Junior #322 violated all five under pressure:

1. **NEVER `git push -f`, `git push --force`, or `git push
   --force-with-lease` on ANY branch.** Not on `phase-*`, not on
   `governance-v0`, not on anything. If a push fails, capture the
   verbatim error, STOP, surface.
2. **NEVER retry `gh pr merge`.** A failed merge is a clean stop.
   Capture the verbatim error message, STOP, surface. Do NOT issue
   the command a second time under any circumstance.
3. **NEVER hand-resolve a merge conflict.** Do NOT stage a local
   merge. Do NOT edit conflict markers. Do NOT improvise an
   alternate merge strategy. The merge fails → STOP.
4. **On any merge failure, ONE clean stop is the ONLY acceptable
   failure behavior.** No creative recovery. No fallback path. No
   "try `--squash` instead". The advisor decides next steps; the
   BM Junior surfaces and waits.
5. **NEVER self-report `result:success` when the primary objective
   failed.** If `gh pr merge` exited non-zero, the merge did not
   happen; the task did not succeed. Self-attribution must be
   honest; the advisor's post-condition check (Phase 5.5) is
   independent verification, not a license for the subagent to
   optimistically self-report.

Source: v1-ship-1-r2 Junior #322 incident — retried `gh pr merge`
3×, attempted `git push -f` on protected `governance-v0` (only
blocked by GH013, not its own judgment), hand-resolved a local
merge, then the task framework reported `result:success` while the
PR was still OPEN. See v1-ship-1-r2-retro.md §3 Action 2.

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
| `UNSTABLE` | Check which CI check is non-green. If the failing check is `adr-compliance` AND `scripts/brehon/check-adr-compliance.sh` (local scan) exits 0 → note the bypass in a DQ `kind: log` entry, then proceed with `gh pr merge {N} --repo barrie-cork/lemmy --merge --delete-branch --admin`. Otherwise STOP and wait for CI to resolve. (2× observed: quality-r3 PR #169 + quality-r3b PR #170 — adr-compliance `workflow_dispatch` re-trigger unreliable in private repos; local scan is authoritative.) |
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

## Phase 5.5 — Advisor post-condition verification (HARD RULE)

After `gh pr merge` returns, BEFORE Phase 6 bookkeeping and BEFORE
accepting `result:success`, run:

```bash
gh pr view {N} --repo barrie-cork/lemmy --json state,mergedAt,mergeCommit
```

The check passes ONLY if ALL three conditions hold:
- `state == "MERGED"`
- `mergedAt` is non-null (ISO timestamp)
- `mergeCommit.oid` is a real sha (40 hex chars)

If ANY condition fails (state still OPEN, mergedAt null, mergeCommit
null), this is an **automatic catch-fire**: BM reported done but the
PR is not merged. Capture the verbatim `gh pr view` output, STOP,
surface to user. Do NOT retry, do NOT improvise, do NOT advance to
Phase 6.

**Rationale:** BM Junior #322 (v1-ship-1-r2, 2026-05-18) self-reported
`result:success` on a total task failure — it had retried `gh pr
merge` 3× against an unmergeable PR, attempted `git push -f` on
protected trunk, hand-resolved a local merge, and written a false
runlog entry "merge sha TBD / remote branch deleted? yes" for a
merge that never happened. The advisor's independent post-condition
check was the ONLY thing that prevented a false "shipped" record
landing on `governance-v0`. This pattern has recurred 5× across the
phase corpus (`pattern_bm_false_success_advisor_post_condition_catch.md`);
codifying it in the verb's own spec — not just in `auto-phase.md`
invariant 7 — makes it apply to every bm-merge invocation, including
direct user-driven ones outside `/auto-phase`.

See `feedback_bm_false_success_advisor_post_condition_catch.md` and
v1-ship-1-r2-retro.md §"What to carry forward" (the
trust-but-verify-after-Junior-done rule).

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

## Phase 8.5 — Bootstrap handover tombstone

After the runlog append (Phase 8) and BEFORE Phase 9 output:

```bash
# Check if a bootstrap handover exists for this phase
PHASE_SLUG=$(gh pr view {N} --repo barrie-cork/lemmy --json headRefName -q '.headRefName | split("-") | .[1:]' | tr -d '\n')
# Bootstrap handovers follow the pattern: .claude/PRPs/handovers/<phase>-bootstrap.md
ls .claude/PRPs/handovers/ | grep -i "bootstrap"
```

If a matching `<phase>-bootstrap.md` exists under `.claude/PRPs/handovers/`:

1. Append the following block to the end of the file:

```markdown
---
## STATUS: SHIPPED

- **PR:** #{N} merged at {ISO timestamp}
- **Merge SHA:** {sha}
- **Tombstoned by:** bm-merge Phase 8.5 post-condition
- **This handover is stale.** Do NOT use the RESUME block above as a
  basis for action — the phase is complete. Delete this file or archive it.
```

2. Commit the tombstone:

```bash
git add .claude/PRPs/handovers/<phase>-bootstrap.md
git commit -m "chore(bm): tombstone handover — <phase> shipped (PR #{N})"
git push origin governance-v0
```

If NO matching bootstrap handover exists: skip silently — no action required.

**Why:** bootstrap handovers authored before a phase runs are sometimes stale
by the time the next session opens them. Appending a tombstone at merge time
converts the handover from a potential-re-derive-trigger into an explicit
"this is done" marker. Rooted in 2026-06-01 state-reconciliation session where
a stale bootstrap caused a full no-op planning-brief authorship (pre-impl HEAD
check was the only catch). See `session-retro-2026-06-01-state-reconciliation-v2.md`.

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
- Bootstrap handover tombstoned (if existed) ✓

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
