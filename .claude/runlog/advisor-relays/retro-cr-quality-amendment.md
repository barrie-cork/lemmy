---
id: retro-cr-quality-amendment
from: advisor
to: impl
ts: 2026-04-24T02:30Z
relates_to: phase-v1-JM-a-retro.md, PR-92 CR findings, user observation 2026-04-24
decision: amend-retro-with-cr-quality-section
---

# Decision
Add a §"CR finding quality (PR #92)" section to `phase-v1-JM-a-retro.md`. User has observed that impl caught line-number drift and at least one hallucination in CR findings. Those observations are actionable signal — capture them in the retro so the four-bucket triage pattern (`feedback_pr_review_triage_pattern`) can be tuned over time.

Goes in the same retro amendment commit as the tool-use section (or a separate one — your call). Same commit subject can serve both: `docs(v1-JM-a): amend retro — tool-use + CR-quality self-assessment sections`.

# Instructions

Append the new section after (or adjacent to) the §"Tool-use self-assessment" section being added per `advisor-relays/retro-tool-use-amendment.md`.

## Section template

```markdown
## §N. CR finding quality — PR #92 (added 2026-04-24)

### §N.1. Hallucinations — findings that didn't match reality

<Bullet list. Each bullet: cr-ID + severity + what CR claimed + what you actually found + whether the underlying concern was valid anyway or fully bogus.>

<Example shape (fill with actual findings):>
<- **cr-X (severity)** — CR claimed `crates/foo/bar.rs:123` contained `<pattern>`, but line 123 has `<different thing>`. Actual location of the pattern: `crates/foo/bar.rs:456`. Underlying concern: VALID (same bug, different line) / PARTIALLY VALID (related concern exists but not the one described) / INVALID (CR invented the pattern).>

### §N.2. Line-number drift — findings with wrong line:col but right concern

<Bullet list. These are not hallucinations — the finding is real but CR's file:line anchor is off (usually by a small delta). Happens most often on generated files (schema.rs), files with recent upstream rebases, or when the PR diff context doesn't line up with current HEAD.>

<Example shape:>
<- **cr-Y (severity)** — CR pointed at `crates/foo/bar.rs:456`; actual location `crates/foo/bar.rs:462`. Off by 6 lines. File: schema.rs. Class: generated-file drift (schema.rs regenerates; line numbers shift on every Diesel regen).>

### §N.3. False-positive classes — findings valid in pattern but not in this codebase

<CR applies general Rust/Diesel/SQL idioms that don't always match Brehon conventions. Capture which patterns triggered this so the plan template can pre-empt them.>

<Example shape:>
<- **cr-Z (severity)** — CR recommended `<general idiom>`; Brehon pattern is `<specific convention>` established in phase/task `<ref>`. Not a bug in the code, but not something to adopt. Resolution: rebut or wont-fix.>

### §N.4. Correct + actionable findings (the bulk)

<Short paragraph counting how many findings were accurate and useful. This balances the negative sections above — CR is a high-value tool and the retro shouldn't overweight the failures.>

<Shape: "Of 10 findings ingested, N were exactly correct in file:line + concern (ratio %). Of those, M drove real fixes that improved the code beyond cosmetic. Particularly good catches: cr-9 (ADR-015 leak, was a real blindspot in the plan), cr-10 (non-idempotent seed, was actually buggy despite the commit message claiming idempotent).">

### §N.5. Per-severity accuracy rollup

<Fill in actual numbers from this phase. Simple table — no deep analysis, just a number line.>

| Severity | Findings ingested | Correct + actionable | Wrong location | Hallucinated | False-positive class |
|----------|-------------------|----------------------|----------------|--------------|----------------------|
| Critical | 0 | — | — | — | — |
| Major    | 4 | <?> | <?> | <?> | <?> |
| Medium   | 0 | — | — | — | — |
| Low      | 5 | <?> | <?> | <?> | <?> |
| Nit      | 1 | <?> | <?> | <?> | <?> |

### §N.6. Impact on four-bucket triage

<Two or three bullets on what the quality pattern means for future phases' triage decisions.>
<Candidate shapes:>
<- "CR's line-number drift on schema.rs is systematic — default to verify-before-fix for any schema.rs finding."
<- "CR's hallucination rate on this phase was <N>/<total>. Compare to prior phases once we have the data — if it's climbing, consider lowering CR's authority weight in the triage gate."
<- "None of CR's findings rebutted — all valid at some level. Confirms the `feedback_coderabbit_block_merge_critical` rule holds."

### §N.7. Recommendation: add to `feedback_pr_review_triage_pattern` memory

<If the patterns here are novel (not already in the memory note), draft a one-line update suggestion so the advisor or user can fold it into the memory. Otherwise note that the patterns confirm existing memory.>
```

## Scope guardrail

Only include findings **this phase** surfaced evidence for. Don't speculate about CR's general behaviour — stick to what cr-1..cr-10 actually showed. If a pattern only happened once, name it but don't generalise.

If you can't remember the specific drift/hallucination observations you mentioned to the user (line numbers in what file? which cr-ID?), pull from your current-session tool calls / Read outputs. Pin the evidence, even if it's brief.

## Validation

Docs-only. Same as the tool-use section — L1 check the commit compiles (trivial).

# Next

1. Slot this section into the retro amendment commit (combine with tool-use per `retro-tool-use-amendment.md`, or separate commits — your call).
2. Suggested combined commit subject: `docs(v1-JM-a): amend retro — tool-use + CR-quality self-assessment sections`.
3. After committing, continue with the lows batch per `pr92-lows-batch.md`.
4. When both retro amendment AND lows batch are landed, relay `impl-relays/pr92-lows-complete.md` listing all three commits.

# Why this matters (user's framing, for your section §N.6)

Over time the user wants a track record of CR's accuracy by finding class. Per the memory `feedback_pr_review_triage_pattern`, the four-bucket triage already separates "valid concern but not this PR" (carry-forward) from "valid + fixable here" (fix-in-pr) — but it doesn't separate "CR's file:line ref was right" from "CR's concern was right, but the location was wrong". That latter distinction is what your §N.2 captures. Accumulated across phases, it tells us:

- Which CR finding classes to trust directly (apply the fix)
- Which to verify first (follow the concern, look up the actual file:line)
- Which to probe for hallucination (re-read the file, see if the pattern CR claims actually exists)

Each phase's retro adds a row to this dataset.

# For the /handover skill design

Add to the skill's design spec:

> Handover briefs should NOT summarise CR findings as "N majors, M lows" — they should carry the finding-quality signal forward ("CR line-drift observed on schema.rs this phase; future phases should pre-verify schema.rs findings"). This keeps the four-bucket triage empirically tuned.

Add this to the retro's §"Handover skill design inputs" (same place the tool-use relay points to).

# Back-reference

Addresses user observation 2026-04-24 via advisor session: "Impl has identified a few errors in terms of line numbers and even a coderabbit hallucination. Will these be also captured in the retro?"
