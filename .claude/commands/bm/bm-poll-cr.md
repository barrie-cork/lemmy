---
description: BM — poll CodeRabbit reviews on a PR; write/update findings YAML
argument-hint: <PR#>
---

# /bm-poll-cr — ingest CodeRabbit findings into YAML

**Input**: $ARGUMENTS — PR number (e.g. `87`)

**Dispatcher → `branch-manager` subagent.** This command delegates
execution to the `branch-manager` subagent, which runs in its own
isolated context window. The subagent reads the operational script
below and follows it step by step.

Invoke:

> Use the `branch-manager` subagent to run `bm-poll-cr`. Arguments:
> $ARGUMENTS (PR number). Follow the phases in
> `.claude/commands/bm/bm-poll-cr.md` — ground-truth the diff, pull CR
> comments from all three gh API endpoints, parse severity from CR
> header, assign stable `cr-<seq>` IDs, detect head-SHA change,
> merge into `.claude/PRPs/reviews/pr-<N>-findings.yaml` per SCHEMA.md.
> Return the counters table and "Next suggested" line.

The parent (impl) session should call `Agent(subagent_type="branch-manager", prompt=<the above>)`. The subagent handles the rest.

---

## Operational script (for the subagent)

The BM agent pulls all CodeRabbit review comments on the given PR
and writes them into `.claude/PRPs/reviews/pr-<N>-findings.yaml` per
the schema at `.claude/PRPs/reviews/SCHEMA.md`. Stable IDs across
re-polls. Auto, no prompt (writes to gitignored file only).

**Reads:** `.claude/rules/branch-manager.md`,
`.claude/PRPs/reviews/SCHEMA.md`.

---

## Phase 1 — Validate input + ensure dirs

```bash
mkdir -p .claude/PRPs/reviews .claude/runlog .claude/PRPs/reviews/.cr-cache

gh pr view {N} --repo barrie-cork/lemmy \
  --json number,state,baseRefName,headRefName,title,isDraft,createdAt,headRefOid
```

| State | Action |
|---|---|
| `MERGED` | INFO: "PR merged — last poll for archival, then stop" |
| `CLOSED` | WARN: "PR closed — poll anyway? confirm" |
| `OPEN` + `isDraft: true` | INFO: "Draft PR — CR skips drafts; expect zero CR findings" |
| `OPEN` + `isDraft: false` | PROCEED |

Capture `headRefOid` (the head commit SHA) — Phase 5 compares it
against the prior poll's `last_polled_head_sha` for change-detection.

---

## Phase 1.5 — Ground-truth the diff

Before trusting any CR walkthrough or "Removed file" claim, fetch the
actual file list in the PR diff and stash to a scratch file:

```bash
gh pr diff {N} --repo barrie-cork/lemmy --name-only \
  > .claude/PRPs/reviews/.cr-cache/pr-{N}-diff-files.txt
```

Phase 3 uses this list to (a) flag findings whose `file:` is not in
the diff as `notes: "outside-diff finding"` and (b) sanity-check
walkthrough claims. This guards against the "CR walkthrough says X
was removed but the diff shows Y" failure mode.

---

## Phase 2 — Pull CR comments via gh API (one query per source)

CodeRabbit posts in three places; all three are polled. Each query
returns **bodies inline** so no per-comment GET is needed (this
removes the N+1 round trip of the v1 design). Output is JSONL files
in `.cr-cache/` for Phase 3 to parse.

### 2.1 PR-level review summaries

```bash
gh api --paginate "repos/barrie-cork/lemmy/pulls/{N}/reviews" \
  --jq '.[] | select(.user.login == "coderabbitai[bot]")
        | {kind: "review", id, submitted_at, body, html_url, state}' \
  > .claude/PRPs/reviews/.cr-cache/pr-{N}-reviews.jsonl
```

### 2.2 In-line review comments (per file/line)

**List endpoint** is `pulls/{N}/comments` (returns bodies inline).
The by-id endpoint is `pulls/comments/{id}` (NOT
`pulls/{N}/comments/{id}` — that 404s; review comments are
repo-scoped, not PR-scoped).

```bash
gh api --paginate "repos/barrie-cork/lemmy/pulls/{N}/comments" \
  --jq '.[] | select(.user.login == "coderabbitai[bot]")
        | {kind: "inline", id, path, line, original_line, body,
           html_url, created_at, in_reply_to_id}' \
  > .claude/PRPs/reviews/.cr-cache/pr-{N}-inline.jsonl
```

### 2.3 Issue-level PR comments (CR walkthroughs + pre-merge-checks)

```bash
gh api --paginate "repos/barrie-cork/lemmy/issues/{N}/comments" \
  --jq '.[] | select(.user.login == "coderabbitai[bot]")
        | {kind: "issue", id, body, html_url, created_at}' \
  > .claude/PRPs/reviews/.cr-cache/pr-{N}-issue.jsonl
```

**Important:** `.cr-cache/` is gitignored (per `.gitignore`'s
`.claude/PRPs/reviews/pr-*-findings.yaml` line — extend that pattern
or add a sibling `.claude/PRPs/reviews/.cr-cache/` line). The cache
is per-poll scratch; safe to delete after Phase 6.

---

## Phase 3 — Parse severity + summary from CR body

CR's actionable findings open with a header line of the form:

```
_⚠️ Potential issue_ | _🟠 Major_
```

or with `🛠️ Refactor suggestion`, `🧩 Analysis chain`, `🧹 Nitpick`,
`💡 Verification agent` in the first slot. The **second token** is
authoritative for severity; the emoji is decoration that may drift.

| Second token in CR header | `severity` |
|---|---|
| `Critical` | `critical` |
| `Major` | `major` |
| `Medium` (rare) | `medium` |
| `Minor` | `low` |
| `Nitpick (assertive)` / no second token | `nit` |

If a comment body has NO header line matching `_.+_ \| _.+_` AND no
explicit `Critical|Major|Minor|Nitpick` token, it is **not an
actionable finding** — skip it (this is how CR posts metadata blocks
like `<details><summary>🤖 Prompt for AI Agents</summary>`).

**Summary** = the first `**bold heading**` line after the severity
header (CR's standard pattern). Fall back to the first sentence after
the header if no bold heading. Truncate to ~120 chars.

**Outside-diff annotation:** if `file:` from a finding is NOT present
in `.cr-cache/pr-{N}-diff-files.txt`, prepend `outside-diff: true`
to `notes:`. Triage uses this hint when deciding `carry-forward` vs
`fix-in-pr`.

**Fix suggestion** = any fenced code block CR proposes after the
summary, captured verbatim into `notes:` (do not reformat).

### Phase 3.5 — Walkthrough + pre-merge-check parsing (issue comments)

CR's walkthrough comment (one per PR; lives in `.cr-cache/pr-{N}-issue.jsonl`)
contains procedural signals worth ingesting. Two patterns to look for:

1. **Pre-merge-check failures.** Look for `### ❌ Failed checks` or
   `❌ Warning` rows in a markdown table. Emit one finding per row
   with:
   - `severity: low`
   - `summary: "CR pre-merge-check: <Check name> — <Explanation, ~60 chars>"`
   - `notes: "Resolution: <Resolution column text>"`
   - `cr_url: <html_url>#issuecomment-<id>` + `#walkthrough` anchor
   - `bucket: fix-in-pr` (default; triage may move to `wont-fix` if
     the user decides to skip the template requirement, etc.)

2. **Estimated review effort + change-cohort tables.** These are
   informational; do NOT emit findings. Log to runlog as
   `walkthrough_summary: "..."` for human reference.

If parser finds zero actionable findings on a non-draft PR open >30
min, log a warning to runlog AND emit a top-level `notes:` field on
the YAML (CR may have failed silently or the bot is throttled).

---

## Phase 4 — Stable IDs across re-polls

For each CR comment, the stable `id` is `cr-<seq>` where `<seq>` is
assigned in chronological order across all three sources (review +
inline + issue/walkthrough findings) on first poll. Re-polls:

- Match existing finding by `(source, cr_url)` pair.
- If found → update `posted_at`, `summary`, `notes` only. Preserve
  `bucket`, `addressed_in`, `rationale`.
- If new → assign next `cr-<seq+1>` and append.

Never renumber or reorder. The findings YAML is append-mostly; the
`id` is a stable handle the impl session can grep for.

---

## Phase 5 — Change-detection + addressed-finding matching

### 5.1 Skip-the-write short-circuit

Before any merge work, compare the PR's current `headRefOid`
(captured in Phase 1) against the YAML's `last_polled_head_sha` (if
the YAML exists). Then count the new CR comments.

| Head SHA changed? | New CR comments? | Action |
|---|---|---|
| No | No | **SKIP write.** Print "no change since poll #N" and STOP cleanly. Do not bump `poll_count`, do not update `last_poll_at`. |
| No | Yes | PROCEED (CR posted without a code change — common for late-arriving findings). |
| Yes | Either | PROCEED. |

This guards against poll-as-curiosity inflating the counter. The YAML
stores `last_polled_head_sha` after every PROCEED-write.

### 5.2 Addressed-finding detection (commit-SHA matching)

For each finding with `bucket: fix-in-pr`, scan commits since the
finding's `posted_at`:

```bash
git log --since="{posted_at}" --format="%H %s" governance-v0..HEAD
```

Heuristic: a commit message containing `cr-<seq>` or `CR #<comment-id>`
or matching the finding's `file:line` ref → mark `addressed_in:
<short-sha>`, leave `bucket: fix-in-pr` (triage promotes to `done`,
not poll).

### 5.3 Force-push detection

If a previously-recorded `addressed_in` SHA is no longer present in
`git log governance-v0..HEAD`, set `addressed_in: null` and append a
`notes:` line. Log warning to runlog.

---

## Phase 6 — Write/update YAML

Read existing YAML (if any), merge per Phase 4 rules, regenerate the
`counters` block from `findings[]`, advance `last_poll_at`,
`poll_count`, and `last_polled_head_sha`.

```python
# Pseudocode for the generator (BM runs as Python or bash+yq)
import yaml, datetime
existing = yaml.safe_load(open(yaml_path, encoding="utf-8")) if exists else {
    "schema_version": 1,
    "pr": pr_num,
    "title": pr_title,
    "head": pr_head,
    "base": pr_base,
    "opened_at": pr_opened_at_iso,
    "poll_count": 0,
    "last_polled_head_sha": None,
    "recommendation": "pending",
    "findings": [],
}
# ... merge per Phase 4 ...
existing["last_poll_at"] = datetime.datetime.utcnow().isoformat() + "Z"
existing["poll_count"] += 1
existing["last_polled_head_sha"] = head_ref_oid_from_phase_1
existing["counters"] = recompute_counters(existing["findings"])
with open(yaml_path, "w", encoding="utf-8") as f:
    yaml.safe_dump(existing, f, sort_keys=False, allow_unicode=True)
```

`encoding="utf-8"` is mandatory (per
`feedback_python_utf8_encoding_windows.md`). `allow_unicode=True`
preserves the CR severity emoji in `notes:` if any survived parse.

After successful write, optionally clean the cache:

```bash
rm .claude/PRPs/reviews/.cr-cache/pr-{N}-*.{jsonl,txt}
```

(Safe to keep for diagnosis if you want — they're gitignored.)

---

## Phase 7 — Append to runlog

```markdown
## bm: poll-cr — {ISO timestamp}
- **PR:** #{N}
- **head SHA:** {short-sha} ({changed since last poll? yes/no})
- **CR comments seen:** {total} ({R} review / {I} inline / {Q} issue)
- **Actionable findings ingested:** {N} ({M} from walkthrough/pre-merge)
- **New findings this poll:** {N}
- **Findings addressed since last poll:** {N} ({SHAs})
- **Counters:** critical {open/done/rebutted} | major {…} | medium {…} | low {…} | nit {…}
- **Recommendation:** {pending|approve|request-changes|block}
- **YAML:** .claude/PRPs/reviews/pr-{N}-findings.yaml
- **Notes:** {anything unusual — outside-diff count, walkthrough flags, force-push detected, etc.}
```

---

## Phase 8 — Output

```markdown
## /bm-poll-cr complete

**PR #{N}:** {title}
**Poll #{count}** at {ts}
**Head SHA:** {short-sha} {(unchanged since poll #N-1) | (advanced from {prev-sha})}

### New since last poll: {N}
{Bullet each new finding: id (severity) — summary. file:line.}
{If outside-diff: tag with `(outside-diff)`.}

### Addressed since last poll: {N}
{Bullet each finding now showing addressed_in: <sha>}

### CR findings counter (post-merge)
- Critical: {open}/{done}/{rebutted}
- Major: {open}/{done}/{rebutted}
- Medium: {open}/{done}/{rebutted}
- Low: {open}/{done}/{rebutted}
- Nit: {open}/{done}/{rebutted}

### Findings YAML
`.claude/PRPs/reviews/pr-{N}-findings.yaml` ({size} bytes)

### Suggested next
- If new critical/major: ping user via `/bm-ping cr-posted` (asks first)
- If PR opened >48h ago AND any major/critical still in `fix-in-pr`
  with `addressed_in: null`: consider `/bm-triage {N}` to bucket
  some as `carry-forward` (file as separate issue) — stale findings
  signal scope drift
- If all findings have `bucket: fix-in-pr` and impl session is active:
  impl reads YAML with `yq` and addresses
- If ready to triage: `/bm-triage {N}`
- If running ADR + cargo review: `/bm-prp-review {N}`
```

---

## Refusal cases

- PR not found → STOP, ask user to verify number.
- `gh api` rate-limited → retry once with backoff; if still limited,
  STOP and report `gh auth status` for diagnosis.
- YAML write fails → do NOT delete existing YAML; STOP and report.

---

## Edge cases

| Situation | Handling |
|---|---|
| CR posts then deletes a comment | Mark finding `bucket: wont-fix`, `notes: "withdrawn by CR"`. Do not delete the row. |
| CR re-posts same finding after force-push | Same `(source, cr_url)` may be reused or a new one issued; if URL differs, treat as new. |
| Finding references a file no longer in the diff | Keep finding; mark `notes: "file removed in {sha}"`. Triage handles disposition. |
| `gh` returns CR comments out of order | Sort by `created_at` before assigning `cr-<seq>` IDs. |

---

## See also

- `.claude/rules/branch-manager.md`
- `.claude/PRPs/reviews/SCHEMA.md` — YAML schema
- `.claude/commands/bm/bm-prp-review.md` — adds Claude/ADR findings
- `.claude/commands/bm/bm-triage.md` — bucket promotion + comment draft
- memory `feedback_pr_review_triage_pattern.md` — four-bucket triage
- memory `feedback_coderabbit_block_merge_critical.md` — CR Critical = block
