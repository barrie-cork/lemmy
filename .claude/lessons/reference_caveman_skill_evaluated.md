---
name: Caveman skill — evaluated and parked
description: JuliusBrussee/caveman — output-token compression skill evaluated 2026-04-23, parked. Don't re-research unless token-axis problem changes.
type: reference
originSessionId: 9a67e495-30d6-43a2-a2ee-8be145286afd
---
JuliusBrussee/caveman (MIT, pushed 2026-04-18) is a Claude Code skill
that compresses **output tokens only** (~65-75%) via telegraphic
phrasing, plus three adjunct skills: caveman-commit (≤50-char commits),
caveman-review (one-line PR comments), caveman-compress (rewrites
CLAUDE.md into grunt-speak).

Evaluated 2026-04-23 for Brehon v1→v2 fit. **Parked — not adopted.**

Three reasons, all still applicable unless context changes:

1. Wrong token axis. Phase 1's 360k-token blowout was **input** side
   (cargo output paste, file re-reads, stream-of-thought diagnosis),
   already mitigated by no-cargo-output-paste.md + phase-splitting +
   cargo-output-capture.md (tail-20 + file redirect). Caveman's own
   README: "Caveman no make brain smaller. Caveman make mouth smaller."
   Output-only compression doesn't touch the axis that hurt.

2. Conflicts with existing conventions. Brehon has prescriptive commit
   style (feedback_commit_hygiene_lockfiles_and_task_labels), four-bucket
   PR triage (feedback_pr_review_triage_pattern), findings YAML schema,
   DQ attribution rules. caveman-commit's ≤50-char rule strips the
   `feat(scope): task N` anchor that's load-bearing for retros and CR
   review. caveman-compress rewriting CLAUDE.md would damage the
   "never re-litigate the 15 ADRs" hard-constraint block.

3. Opaque verification cost. Compressed output is harder to verify
   against silent-failure patterns, PM-hook stability, and DQ
   attribution "why" requirements.

**When to reconsider:** if a future output-volume issue emerges that
isn't fixable by tightening the "one-sentence per update" bar already
in the system prompt (e.g., a v2 PRD agent producing multi-screen
prose per tool call). Until then, skip.
