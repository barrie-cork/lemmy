---
description: BM — show current branch, unpushed commits, PR state, CR count, DQ pending
argument-hint: (none)
---

# /bm-status — read-only branch + PR snapshot

**Dispatcher → `branch-manager` subagent.** This command delegates
execution to the `branch-manager` subagent, which runs in its own
isolated context window and shares the working-directory filesystem.
The subagent reads the operational script below (the same file you're
reading) and follows it step by step.

Invoke:

> Use the `branch-manager` subagent to run `bm-status`. Arguments:
> $ARGUMENTS. Follow the phases in `.claude/commands/bm/bm-status.md`.
> Return a one-screen summary followed by a "Next suggested" line per
> your return-format rule.

The parent (impl) session should call `Agent(subagent_type="branch-manager", model="haiku", prompt=<the above>)`. Mechanical read-only verb — Haiku 4.5 is sufficient; no semantic judgment required.

---

## Operational script (for the subagent)

The BM agent prints a one-screen summary of what's happening on the
current branch and any open PR for it. Read-only, auto, no prompt.

**Reads:** `.claude/rules/branch-manager.md` for boundaries.

---

## Phase 1 — Local git state

```bash
git fetch origin
git branch --show-current
git status --short
git log governance-v0..HEAD --oneline | head -20
git log HEAD..origin/$(git branch --show-current) --oneline 2>/dev/null | head -5
git log origin/$(git branch --show-current)..HEAD --oneline 2>/dev/null | head -5
```

Capture:

- `current_branch`
- `dirty_files` count
- `commits_ahead_of_trunk` count
- `commits_behind_remote` count (if remote exists)
- `commits_ahead_of_remote` count (unpushed)

---

## Phase 2 — Open PR state (if any)

```bash
gh pr list --repo barrie-cork/lemmy \
  --head $(git branch --show-current) \
  --state open \
  --json number,title,state,reviewDecision,mergeStateStatus,baseRefName \
  | head -50
```

If a PR exists:

```bash
gh pr view {N} --repo barrie-cork/lemmy \
  --json number,title,state,reviewDecision,mergeStateStatus,reviews,comments,statusCheckRollup,additions,deletions,changedFiles
```

Capture:

- `pr_number`, `pr_state`, `pr_review_decision`, `pr_merge_state`
- CR review count (CodeRabbit author = `coderabbitai[bot]`)
- Failing checks count
- Total `+/-` lines

---

## Phase 3 — Findings YAML state

```bash
ls .claude/PRPs/reviews/pr-${PR_NUM}-findings.yaml 2>/dev/null
```

If exists, read the `counters` block (and only that block — no full
file dump per `no-cargo-output-paste.md` discipline):

```bash
yq '.counters' .claude/PRPs/reviews/pr-${PR_NUM}-findings.yaml
yq '.last_poll_at, .poll_count' .claude/PRPs/reviews/pr-${PR_NUM}-findings.yaml
```

If `yq` is unavailable, fall back to `python -c 'import yaml; ...'`.

---

## Phase 4 — Decision queue + runlog

```bash
yq '.pending | length' .claude/decision-queue.json 2>/dev/null \
  || python -c "import json; print(len(json.load(open('.claude/decision-queue.json'))['pending']))"

ls -t .claude/runlog/*.md 2>/dev/null | head -1
```

Capture:

- `dq_pending` count
- Most recent runlog file path + last modified timestamp

---

## Phase 5 — Print one-screen summary

```markdown
## /bm-status — {ISO timestamp}

### Branch
- **Current:** `{current_branch}`
- **Working tree:** {clean | {N} dirty files}
- **vs trunk:** {ahead by N} | {at trunk}
- **vs remote:** {ahead by N (unpushed)} | {behind by N (need fetch)} | {in sync} | {no remote}

### PR (if any)
- **#{pr_number}:** {title} → {pr_state}
- **Review:** {pr_review_decision} (CodeRabbit reviews: {N})
- **Merge state:** {pr_merge_state}
- **Failing checks:** {N}
- **Diff:** +{add}/-{del} ({changedFiles} files)

### Findings YAML
- **File:** {path or "not yet polled"}
- **Last poll:** {ts}, count {N}
- **Counters:** critical {open/done/rebutted} | major {…} | medium {…} | low {…}

### Coordination
- **DQ pending:** {N}
- **Last runlog:** {path} ({mtime})

### Suggested next BM action
{computed recommendation — see logic below}
```

### Recommendation logic

| Observed state | Suggested next |
|---|---|
| On `phase-*`, no PR, commits ahead of remote | `/bm-push` then `/bm-pr` |
| PR open, no findings YAML | `/bm-poll-cr {N}` |
| PR open, findings YAML stale (>30 min since last poll) | `/bm-poll-cr {N}` |
| PR open, no Claude review yet | `/bm-prp-review {N}` |
| PR open, all findings done, all checks green | `/bm-merge {N}` |
| `dq_pending > 0` | "Read DQ — answer or surface to user before proceeding" |
| On trunk (`governance-v0`), no current sub-phase | `/bm-cut <next-suffix>` (after plan PR merges) |

---

## Refusal cases

None — `/bm-status` is read-only.

---

## See also

- `.claude/rules/branch-manager.md`
- `.claude/PRPs/reviews/SCHEMA.md`
- `.claude/commands/bm/bm-poll-cr.md`
- `.claude/commands/bm/bm-pr.md`
