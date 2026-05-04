---
description: BM — run Brehon /prp-review (ADR + cargo) and merge findings into the same YAML
argument-hint: <PR#>
disable-model-invocation: true
---

# /bm-prp-review — Brehon ADR + cargo review, ingested into findings YAML

**Input**: $ARGUMENTS — PR number (e.g. `87`)

**Dispatcher → `branch-manager` subagent.** This command delegates
execution to the `branch-manager` subagent, which runs in its own
isolated context window. The subagent reads the operational script
below and follows it step by step.

**Note on subagent limits:** subagents cannot invoke other subagents,
so the BM agent cannot call `/prp-core:prp-review` itself. If the
parent wants a full `/prp-review` run, the parent invokes it first
and then runs `/bm-prp-review <N>` so BM ingests the resulting
`.claude/PRPs/reviews/pr-<N>-review.md`.

**The markdown report is REQUIRED.** If `pr-<N>-review.md` is missing,
BM STOPS and tells the user to run `/prp-review` first. BM does NOT
proceed with a partial `source: claude` review derived only from cargo
output, because that path skips the ADR / cross-cutting / API-shape
checks that the full `/prp-review` performs. Emitting an incomplete
`source: claude` set would let blocking ADR violations slip past the
merge gate.

<!-- cr-8 (closes #88): the prior fallback path described only cargo
     capture but the command still labelled findings as `source: claude`,
     which the schema treats as ADR-/cross-cutting-aware. Make the full
     `/prp-review` mandatory rather than emit incomplete claude-* rows. -->


Invoke:

> Use the `branch-manager` subagent to run `bm-prp-review`. Arguments:
> $ARGUMENTS (PR number). Follow the phases in
> `.claude/commands/bm/bm-prp-review.md` — capture cargo logs to
> `.claude/build-bm-pr<N>-{check,clippy,test}.log`, tail each,
> parse findings from `pr-<N>-review.md` if it exists (otherwise write
> claude-* findings from your own cargo and ADR analysis), assign
> stable `claude-<seq>` IDs, merge into findings YAML with
> `source: claude`, set top-level `recommendation`. Return the cargo
> exit codes + ADR summary + "Next suggested" line.

The parent (impl) session should call `Agent(subagent_type="branch-manager", model="opus", prompt=<the above>)`. Judgment-heavy: interprets cargo failures, writes severity/bucket, cross-checks ADRs. Keep on Opus — misclassifying a cargo failure or ADR violation as non-blocking is the `feedback_coderabbit_block_merge_critical.md` failure mode.

---

## Operational script (for the subagent)

The BM agent runs the existing `/prp-core:prp-review` flow (ADR
compliance + cargo validation + cross-cutting greps) and merges its
findings into `.claude/PRPs/reviews/pr-<N>-findings.yaml` with
`source: claude`. Auto, no prompt. The underlying `/prp-review`
itself does the cargo runs; BM is the wrapper that captures, parses,
and unifies output into the YAML schema.

**Reads:** `.claude/rules/branch-manager.md`,
`.claude/PRPs/reviews/SCHEMA.md`,
`.claude/commands/prp-core/prp-review.md`.

---

## Phase 1 — Validate input

```bash
gh pr view {N} --repo barrie-cork/lemmy \
  --json number,state,baseRefName,headRefName,title,additions,deletions,changedFiles
```

| State | Action |
|---|---|
| `MERGED` | INFO: "Skip Claude review — PR merged" then STOP |
| `CLOSED` | WARN: "Closed PR — review anyway? confirm" |
| `OPEN` | PROCEED |

---

## Phase 2 — Run /prp-review under capture

`/prp-review` is the canonical Brehon review (see
`.claude/commands/prp-core/prp-review.md`). BM invokes it but does
NOT let it post to GitHub itself — BM owns the GitHub-comment surface
via `/bm-triage`.

The standard `/prp-review` writes:
- `.claude/PRPs/reviews/pr-<N>-review.md` — canonical Markdown report
  (kept; tracked in git)

BM additionally:
- Reads `pr-<N>-review.md` after the run
- Extracts each issue (Critical / High / Medium / Low / Suggestions)
  → schema-mapped finding rows
- Appends to `.claude/PRPs/reviews/pr-<N>-findings.yaml` with
  `source: claude`

### Cargo capture discipline

`/prp-review` runs cargo. Per `.claude/rules/cargo-output-capture.md`,
ALL cargo invocations must be captured to `.claude/build-bm-pr<N>-{step}.log`
with exit code preserved. Per `.claude/rules/no-cargo-output-paste.md`,
only the tail of each log enters the conversation.

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/build-bm-pr{N}-check.log 2>&1"
echo "check exit: $?"
tail -20 .claude/build-bm-pr{N}-check.log

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/build-bm-pr{N}-clippy.log 2>&1"
echo "clippy exit: $?"
tail -20 .claude/build-bm-pr{N}-clippy.log

cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/build-bm-pr{N}-test.log 2>&1"
echo "test compile exit: $?"
tail -20 .claude/build-bm-pr{N}-test.log
```

---

## Phase 3 — Severity mapping (Claude → YAML)

| `/prp-review` level | YAML `severity` |
|---|---|
| Critical (BLOCK) | `critical` |
| High | `major` |
| Medium | `medium` |
| Low | `low` |
| Suggestion | `nit` |

ADR violations are ALWAYS `critical`. Set `adr: ADR-<N>` field on
the finding row.

Cargo failures are ALWAYS `critical`. Set `bucket: fix-in-pr` and
`notes:` carries the tail of the failing log path (NOT the log
content — per `no-cargo-output-paste.md`).

---

## Phase 4 — Stable IDs

Claude findings use `claude-<seq>` IDs. Sequence advances per-PR,
across re-runs. Match by `(source, file, line, summary[:60])` tuple
on re-run; matched rows update `posted_at` only.

If `/prp-review` is re-run and a finding is no longer present in the
new report (Claude considered it resolved), promote bucket:

- If `bucket: fix-in-pr` and finding gone → `bucket: done`,
  `addressed_in: <HEAD-sha-at-rerun>`.
- If `bucket: rebut` and finding gone → leave; finding may have
  reappeared as a new ID; let triage decide.

---

## Phase 5 — Write/update YAML

Same merge discipline as `/bm-poll-cr` (Phase 4). Regenerate
`counters` block. Advance `last_poll_at` and `poll_count` for the
Claude side via separate counters: `claude_run_count`,
`last_claude_run_at`.

If the YAML doesn't exist yet (no CR poll first), create it from
scratch with the same top-level shape, `findings:` containing only
the Claude rows. The CR poll later will append `cr-*` rows.

---

## Phase 6 — Top-level recommendation

After merging, set the YAML's top-level `recommendation:`:

| Condition | `recommendation` |
|---|---|
| Any `critical` open in `fix-in-pr` | `block` |
| Any `major` open in `fix-in-pr` AND no critical | `request-changes` |
| Only `medium`/`low`/`nit` open in `fix-in-pr` | `approve` |
| All remaining findings in `bucket: rebut` / `carry-forward` / `done` / `wont-fix` (no `fix-in-pr` rows of any severity) | `approve` |

<!-- cr-9 (closes #88): a PR with no open `fix-in-pr` rows could
     previously keep a non-`approve` recommendation if all surviving
     findings were `rebut`/`carry-forward`/`done`/`wont-fix` only,
     blocking `/bm-merge`. The fourth row makes that explicit. -->


This is the BM's recommendation, not auto-posted. `/bm-triage` reads
this when drafting the digest comment.

---

## Phase 7 — Append to runlog

```markdown
## bm: prp-review — {ISO timestamp}
- **PR:** #{N}
- **cargo check:** {PASS|FAIL exit N}, log .claude/build-bm-pr{N}-check.log
- **cargo clippy:** {PASS|FAIL exit N}, log .claude/build-bm-pr{N}-clippy.log
- **cargo test --no-run:** {PASS|FAIL exit N}, log .claude/build-bm-pr{N}-test.log
- **ADR violations:** {N} ({list ADRs})
- **Issues by severity:** critical {N} / major {N} / medium {N} / low {N} / nit {N}
- **Recommendation:** {approve|request-changes|block}
- **Markdown report:** .claude/PRPs/reviews/pr-{N}-review.md
- **Findings YAML updated:** .claude/PRPs/reviews/pr-{N}-findings.yaml
```

---

## Phase 8 — Output

```markdown
## /bm-prp-review complete

**PR #{N}:** {title}
**Recommendation:** {approve | request-changes | block}

### Cargo validation
| Check | Result | Log |
|---|---|---|
| `cargo check --workspace --features full` | {PASS/FAIL} | .claude/build-bm-pr{N}-check.log |
| `cargo clippy ...` | {PASS/FAIL} | .claude/build-bm-pr{N}-clippy.log |
| `cargo test --test e2e --no-run -p lemmy_server` | {PASS/FAIL} | .claude/build-bm-pr{N}-test.log |

### ADR compliance
{Bullet each ADR check from /prp-review with PASS/FAIL}

### Findings added (source: claude)
- Critical: {N}
- Major: {N}
- Medium: {N}
- Low: {N}
- Nit: {N}

### Artifacts
- Markdown report: `.claude/PRPs/reviews/pr-{N}-review.md` (tracked)
- Unified findings: `.claude/PRPs/reviews/pr-{N}-findings.yaml` (gitignored)

### Suggested next
- Triage + draft digest: `/bm-triage {N}`
- Ping user if recommendation = block: `/bm-ping cr-posted` (asks first)
```

---

## Refusal cases

- Cargo fails to compile (build env issue, not code) → STOP, do NOT
  add findings, ask user to check `scripts/brehon/` wrappers per
  `pre-phase-harness-audit.md`.
- `/prp-review` Markdown malformed → log warning, attempt parse,
  flag missing sections in `notes:`.
- Cargo log file already exists from a prior run → overwrite (this is
  a fresh review).

---

## See also

- `.claude/rules/branch-manager.md`
- `.claude/commands/prp-core/prp-review.md` — the underlying review
- `.claude/rules/cargo-output-capture.md`
- `.claude/rules/no-cargo-output-paste.md`
- `.claude/PRPs/reviews/SCHEMA.md`
