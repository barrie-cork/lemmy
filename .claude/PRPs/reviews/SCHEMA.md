# `pr-<N>-findings.yaml` schema (v1)

The branch-manager (BM) agent writes a unified findings file per PR
at `.claude/PRPs/reviews/pr-<N>-findings.yaml`. This file is the
single source of truth for CodeRabbit + Brehon `/prp-review` +
manual findings on a PR. Other CC sessions (impl, /prp-issue-fix,
ad-hoc) read it via `yq` or `python -c "import yaml"`.

The file is **gitignored** (per `.gitignore`). It is a runtime
artifact of the BM session and does not ship to trunk.

The Markdown report at `.claude/PRPs/reviews/pr-<N>-review.md`
written by `/prp-review` is the human-readable canonical Brehon
review and IS tracked in git.

---

## Top-level shape

```yaml
schema_version: 1                          # int; bump on breaking schema change
pr: 87                                     # PR number (int)
title: "v1-AD-d — admin dashboard aggregate + audit SSE"
head: phase-v1-AD-d                        # head ref
base: governance-v0                        # base ref (always governance-v0 for v1)
opened_at: 2026-04-23T14:00:00Z            # ISO 8601 UTC, from gh pr view
last_poll_at: 2026-04-23T14:35:00Z         # ISO 8601, last /bm-poll-cr run
poll_count: 3                              # int, advances per /bm-poll-cr (only on PROCEED-write)
last_polled_head_sha: 0a1b2c3d4            # short SHA of head at last poll; change-detection
last_claude_run_at: 2026-04-23T14:33:00Z   # ISO 8601, last /bm-prp-review run
claude_run_count: 1                        # int, advances per /bm-prp-review
recommendation: pending                    # pending | approve | request-changes | block
findings: []                               # see Finding shape below
counters: {}                               # see Counters shape below
merged_at: null                            # set by /bm-merge on success
merge_commit: null                         # short SHA, set by /bm-merge
final_recommendation: null                 # frozen value of recommendation at merge
```

All timestamps are ISO 8601 UTC with `Z` suffix. The BM uses
`datetime.datetime.utcnow().isoformat() + "Z"` (Python) or
`date -u +"%Y-%m-%dT%H:%M:%SZ"` (bash).

---

## Finding shape

```yaml
- id: cr-1                                 # stable handle: cr-N | claude-N | user-N
  source: coderabbit                       # coderabbit | claude | user
  severity: critical                       # critical | major | medium | low | nit
  posted_at: 2026-04-23T14:30:00Z          # ISO 8601 UTC
  file: crates/api/api/src/governance/admin_dashboard.rs   # nullable for non-line findings
  line: 142                                # int or null
  summary: "Aggregate query missing actor_pseudonym join"  # ~120 char limit
  cr_url: https://github.com/barrie-cork/lemmy/pull/87#discussion_r123456  # only when source=coderabbit
  adr: ADR-008                             # only when source=claude AND ADR violation
  bucket: fix-in-pr                        # fix-in-pr | rebut | carry-forward | done | wont-fix
  addressed_in: null                       # short SHA when bucket=done; else null
  rationale: null                          # required when bucket in (rebut, wont-fix)
  notes: null                              # free-text; carry-forward stores GH issue URL here
```

### Field discipline

| Field | Required | Mutable across re-polls? | Notes |
|---|---|---|---|
| `id` | YES | NO | Stable handle. Never renumber, never reorder. |
| `source` | YES | NO | One of three values. |
| `severity` | YES | NO once set | Set on first ingest. Re-poll preserves. |
| `posted_at` | YES | YES | Updates if CR re-posts the same comment. |
| `file` | NO | NO | Null for PR-level (non-inline) findings. |
| `line` | NO | NO | Null when `file` is null OR when finding is file-level. |
| `summary` | YES | YES | Updates only if upstream summary changes. |
| `cr_url` | conditional | NO | Required for `source: coderabbit`; null otherwise. |
| `adr` | conditional | NO | Required for `source: claude` ADR violations; null otherwise. |
| `bucket` | YES | YES | The triage promotes this. |
| `addressed_in` | conditional | YES | Required when `bucket: done`; null otherwise. |
| `rationale` | conditional | YES | Required when `bucket in (rebut, wont-fix)`. |
| `notes` | NO | YES | Free text; carry-forward stores `gh issue create` URL. |

---

## Counters shape

Auto-regenerated on every YAML write from `findings[]`. Never
hand-edit.

```yaml
counters:
  critical:
    open: 2          # bucket in (fix-in-pr) and severity=critical
    done: 0          # bucket=done
    rebutted: 0      # bucket=rebut
    carry_forward: 0 # bucket=carry-forward
    wont_fix: 0      # bucket=wont-fix
  major:    {open: 0, done: 0, rebutted: 0, carry_forward: 0, wont_fix: 0}
  medium:   {open: 0, done: 0, rebutted: 0, carry_forward: 0, wont_fix: 0}
  low:      {open: 0, done: 0, rebutted: 0, carry_forward: 0, wont_fix: 0}
  nit:      {open: 0, done: 0, rebutted: 0, carry_forward: 0, wont_fix: 0}
  by_source:
    coderabbit: 6
    claude:     2
    user:       0
  total: 8
```

---

## Bucket semantics

| Bucket | Meaning | Required field |
|---|---|---|
| `fix-in-pr` | Real issue, in scope, will fix in this PR | (none) |
| `rebut` | Reviewer wrong / false positive / intended pattern / superseded by ADR | `rationale` |
| `carry-forward` | Real but out-of-scope; filed as separate GH issue | `notes:` (issue URL) |
| `done` | Addressed by a commit on the head branch | `addressed_in:` (SHA) |
| `wont-fix` | Will not address; style preference, fork-specific intent | `rationale` |

Critical ADR violations (`source: claude` + `adr: ADR-<N>` +
`severity: critical`) cannot move to `wont-fix` or `rebut` without
an explicit DQ override citing a superseding ADR.

Cargo-failure findings (`source: claude` + summary mentions cargo)
cannot move to anything other than `fix-in-pr`.

---

## Top-level recommendation logic

Computed by `/bm-prp-review` and `/bm-triage` after every change:

| Condition | `recommendation` |
|---|---|
| Any `severity: critical` AND `bucket: fix-in-pr` | `block` |
| Any `severity: major` AND `bucket: fix-in-pr` AND no critical fix-in-pr | `request-changes` |
| All findings in `bucket in (done, rebut, carry-forward, wont-fix)` | `approve` |
| Only `medium`/`low`/`nit` open in `fix-in-pr` | `approve` |
| Initial state (no review run yet) | `pending` |

`/bm-merge` requires `recommendation: approve` AND no
`bucket: fix-in-pr` rows with `addressed_in: null`.

---

## Reading the file from another session

### From bash + yq

```bash
# All open critical findings impl needs to address
yq '.findings[] | select(.bucket == "fix-in-pr" and .severity == "critical")' \
  .claude/PRPs/reviews/pr-87-findings.yaml

# Top-level recommendation
yq '.recommendation' .claude/PRPs/reviews/pr-87-findings.yaml

# Counts
yq '.counters' .claude/PRPs/reviews/pr-87-findings.yaml
```

### From Python

```python
import yaml
with open(".claude/PRPs/reviews/pr-87-findings.yaml", encoding="utf-8") as f:
    data = yaml.safe_load(f)

open_critical = [
    f for f in data["findings"]
    if f["bucket"] == "fix-in-pr" and f["severity"] == "critical"
]
print(f"{len(open_critical)} critical to address")
```

UTF-8 encoding is mandatory on Windows (per
`feedback_python_utf8_encoding_windows.md`).

---

## Lifecycle

1. **`/bm-pr` opens PR** — file does not exist yet.
2. **`/bm-poll-cr <N>`** (first run) — creates file with all CR rows
   in `bucket: fix-in-pr`.
3. **`/bm-prp-review <N>`** — appends Claude rows; recomputes
   `recommendation`.
4. **Impl reads YAML, fixes, commits** — `/bm-poll-cr` re-run picks
   up new commits via SHA matching, sets `addressed_in`.
5. **`/bm-triage <N>`** — promotes buckets per the four-bucket rules,
   drafts comment, asks before posting.
6. **`/bm-merge <N>`** — requires gates per Phase 2 of bm-merge.md;
   on success appends `merged_at`, `merge_commit`,
   `final_recommendation`.

---

## Schema versioning

`schema_version: 1` is the current version. Breaking changes (field
renames, removed fields, changed semantics) bump the version. The
BM is the only writer; readers check `schema_version` and refuse to
parse unknown versions rather than guessing.

To migrate an existing file across versions, BM writes a
`.claude/PRPs/reviews/pr-<N>-findings.v<N>.bak.yaml` snapshot before
the in-place migration. No silent rewrites.

---

## See also

- `.claude/rules/branch-manager.md` — operating rules
- `.claude/commands/bm/bm-poll-cr.md` — CR ingestion
- `.claude/commands/bm/bm-prp-review.md` — Claude/ADR ingestion
- `.claude/commands/bm/bm-triage.md` — bucket promotion
- `.claude/commands/bm/bm-merge.md` — final gate
- memory `feedback_pr_review_triage_pattern.md` — four-bucket pattern
