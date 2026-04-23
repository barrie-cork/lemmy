---
description: BM — re-classify findings into four buckets; draft digest comment; ASK before posting
argument-hint: <PR#>
---

# /bm-triage — four-bucket triage + digest comment

**Input**: $ARGUMENTS — PR number (e.g. `87`)

**Dispatcher → `branch-manager` subagent.** This command delegates
execution to the `branch-manager` subagent, which runs in its own
isolated context window. The subagent reads the operational script
below and follows it step by step.

Invoke:

> Use the `branch-manager` subagent to run `bm-triage`. Arguments:
> $ARGUMENTS (PR number). Follow the phases in
> `.claude/commands/bm/bm-triage.md` — apply four-bucket triage
> (fix-in-pr / rebut / carry-forward / wont-fix), update YAML with
> rationales, draft
> `.claude/PRPs/reviews/pr-<N>-comment.md`. ASK via AskUserQuestion
> before posting the comment with `gh pr comment`, and ASK per each
> carry-forward issue before `gh issue create`. Return the bucket
> counts + posted status + "Next suggested" line.

The parent (impl) session should call `Agent(subagent_type="branch-manager", model="opus", prompt=<the above>)`. Judgment-heavy: four-bucket classification is the merge-gate brain. Keep on Opus — misbucketing a critical fix-in-pr as rebut/wont-fix is the exact failure mode CodeRabbit-block-merge discipline (`feedback_coderabbit_block_merge_critical.md`, 3× caught) exists to prevent. Also applies the "if I revert this, does the symptom return?" test (`feedback_severity_labels_dont_imply_semantic.md`).

---

## Operational script (for the subagent)

The BM agent re-classifies all open findings into the four-bucket
triage from `feedback_pr_review_triage_pattern.md`, drafts a digest
comment, and **asks the user before posting**. The comment is the
BM's only outbound to GitHub on a PR (besides the merge).

**Reads:** `.claude/rules/branch-manager.md`,
`.claude/PRPs/reviews/SCHEMA.md`, memory
`feedback_pr_review_triage_pattern.md`,
`feedback_coderabbit_triage_four_buckets_confirmed.md`,
`feedback_severity_labels_dont_imply_semantic.md`.

---

## Phase 1 — Load findings YAML

```bash
yq '.' .claude/PRPs/reviews/pr-{N}-findings.yaml
```

If file missing: STOP, "Run `/bm-poll-cr {N}` and `/bm-prp-review {N}` first."

---

## Phase 2 — Apply four-bucket triage

For every open finding (i.e. not already `done` / `wont-fix`),
classify into one of:

| Bucket | Criteria |
|---|---|
| `fix-in-pr` | Critical or Major that's a real bug; clear fix; small scope; in scope of THIS PR |
| `rebut` | Reviewer is wrong (false positive, intended pattern, ADR-overridden, prior decision in DQ/runlog). MUST cite the rebuttal source in `rationale`. |
| `carry-forward` | Real issue but out-of-scope for this PR. MUST file a GH issue and link in `notes` (use `gh issue create --repo barrie-cork/lemmy`). Bucket auto-flips to `carry-forward` only after issue URL is captured. |
| `wont-fix` | Will not address (style preference, won't change for this fork, intentional). MUST cite reason in `rationale`. |

Apply per memory `feedback_severity_labels_dont_imply_semantic.md`:
ask "if I revert this CR-suggested change, does the symptom return?"
If no → likely `rebut` (semantic-neutral suggestion).

### Special-case: ADR violations

`source: claude` rows with `adr: ADR-<N>` and `severity: critical`
default to `fix-in-pr`. They are NEVER auto-bucketed to `wont-fix` or
`rebut` without an explicit override decision in DQ that cites a
superseding ADR.

### Special-case: cargo failures

`source: claude` rows that are cargo failures default to `fix-in-pr`.
Cannot be `rebut` or `wont-fix`.

---

## Phase 3 — Update YAML in place

For each newly-bucketed finding:

- Set `bucket: <new-bucket>`
- Set `rationale: "<one-sentence why>"` if bucket is `rebut` or `wont-fix`
- Set `notes:` to the GH issue URL if bucket is `carry-forward`
- Recompute `counters` block

For `carry-forward` findings, BM creates the GH issue **after** asking
user (next phase). It does NOT auto-create issues silently — those are
visible actions.

---

## Phase 4 — Draft the digest comment

Write `.claude/PRPs/reviews/pr-<N>-comment.md`:

```markdown
## Review digest — PR #{N}

**Recommendation:** {approve | request-changes | block}

### Findings disposition

| Bucket | Critical | Major | Medium | Low | Nit |
|---|---|---|---|---|---|
| fix-in-pr | {N} | {N} | {N} | {N} | {N} |
| rebut | {N} | {N} | {N} | {N} | {N} |
| carry-forward | {N} | {N} | {N} | {N} | {N} |
| done | {N} | {N} | {N} | {N} | {N} |
| wont-fix | {N} | {N} | {N} | {N} | {N} |

### fix-in-pr — to address before merge

{For each: id, severity, file:line, summary, fix recap}

### rebut — pushing back

{For each: id, severity, summary, rationale (citing rule/decision)}

### carry-forward — filing as separate issue

{For each: id, severity, summary, GH issue URL (if created)}

### done — already addressed

{For each: id, summary, addressed_in: <short-sha>}

### wont-fix

{For each: id, summary, rationale}

---

*Posted by branch-manager session.*
*Findings file: `.claude/PRPs/reviews/pr-{N}-findings.yaml` (local, gitignored).*
*Brehon review report: `.claude/PRPs/reviews/pr-{N}-review.md`.*
```

---

## Phase 5 — ASK USER before posting

This is a visible, outbound action. ASK explicitly:

```markdown
**Posting digest comment to PR #{N}?**

Action: `gh pr comment {N} --repo barrie-cork/lemmy --body-file .claude/PRPs/reviews/pr-{N}-comment.md`

Counts to be communicated:
- {open critical} critical / {open major} major still in fix-in-pr
- {N} findings rebutted (rationales attached)
- {N} carry-forward issues to be filed
- {N} findings already addressed in: {SHAs}

Reply `confirm` to post, `dry-run` to print body without posting,
or any other input to abort.
```

If the user replies `dry-run`, print the comment body to stdout but
do NOT call `gh pr comment`.

---

## Phase 6 — File carry-forward issues (if confirmed)

For each `carry-forward` row without a `notes:` URL yet, ASK before
each `gh issue create`:

```markdown
**File carry-forward issue for finding {id}?**

Title: "carry-forward from PR #{N}: {summary}"
Labels: carry-forward, source-{coderabbit|claude}
Body: refs to PR + finding YAML row + recommended next step

Reply `confirm` to file, or any other input to skip this one.
```

If confirmed: `gh issue create --repo barrie-cork/lemmy ...`, capture
URL, write back to finding's `notes:`, regenerate counters.

---

## Phase 7 — Post the digest (if confirmed)

```bash
gh pr comment {N} --repo barrie-cork/lemmy \
  --body-file .claude/PRPs/reviews/pr-{N}-comment.md
```

Capture the comment URL.

---

## Phase 8 — Append to runlog

```markdown
## bm: triage — {ISO timestamp}
- **PR:** #{N}
- **Buckets:** fix-in-pr {N} | rebut {N} | carry-forward {N} | done {N} | wont-fix {N}
- **Comment posted?** {yes (URL) | dry-run | aborted}
- **Carry-forward issues filed:** {N} ({URLs})
- **Recommendation:** {approve|request-changes|block}
```

---

## Phase 9 — Output

```markdown
## /bm-triage complete

**PR #{N}:** {title}
**Recommendation:** {approve | request-changes | block}

### Buckets after triage

| Bucket | Total | Critical | Major | Medium | Low | Nit |
|---|---|---|---|---|---|---|
| fix-in-pr | {N} | {N} | {N} | {N} | {N} | {N} |
| rebut | {N} | {N} | {N} | {N} | {N} | {N} |
| carry-forward | {N} | {N} | {N} | {N} | {N} | {N} |
| done | {N} | {N} | {N} | {N} | {N} | {N} |
| wont-fix | {N} | {N} | {N} | {N} | {N} | {N} |

### Outbound actions
- Comment posted: {yes (URL) | dry-run | aborted}
- Carry-forward issues filed: {N} ({URLs})

### Suggested next
- If recommendation = `block` and impl session active: impl reads YAML
  with `yq '.findings[] | select(.bucket=="fix-in-pr" and .severity=="critical")'`
  and addresses
- If `request-changes`: similar but for `major`
- If `approve`: `/bm-merge {N}` (asks before merging)
```

---

## Refusal cases

- Findings YAML missing → STOP, run poll/review first.
- ADR-violation finding being moved to `rebut` without superseding-ADR
  citation → STOP, file DQ entry asking for explicit override.
- Cargo-failure finding being moved to anything other than `fix-in-pr`
  → STOP.
- User reply other than `confirm` / `dry-run` → ABORT, do not post.

---

## See also

- `.claude/rules/branch-manager.md`
- memory `feedback_pr_review_triage_pattern.md`
- memory `feedback_coderabbit_triage_four_buckets_confirmed.md`
- memory `feedback_severity_labels_dont_imply_semantic.md`
- memory `feedback_coderabbit_block_merge_critical.md`
- `.claude/PRPs/reviews/SCHEMA.md`
- `.claude/commands/bm/bm-merge.md` — final merge gate
