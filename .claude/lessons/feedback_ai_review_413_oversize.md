---
name: AI review workflow 413 oversize on phase-size PRs
description: actions/ai-inference@v2 with gpt-4o caps at 8000 tokens; Phase-size PRs (~1500+ lines) will 413-fail
type: feedback
originSessionId: 07131c85-9c72-4cef-a469-5051bc6e326a
---
# AI review workflow 413 oversize

The `.github/workflows/governance-ai-review.yml` (or similar) running `actions/ai-inference@v2` with `gpt-4o` has a hard cap at **8000 tokens** for the request body. Any Brehon phase PR with ~1500+ lines of diff will fail with HTTP 413 "Request body too large".

**Why:** gpt-4o's default context is 128k but the `ai-inference@v2` wrapper caps the request payload regardless. Observed on Phase 5a PR #4 (2026-04-17, ~1500-line diff).

**How to apply:** If you see `AI review: fail 7s` status check on a phase PR with a long diff, do NOT treat it as a blocking review finding. It's infrastructure. Confirm by checking the action log for `413 Request body too large`. Then:

1. Acknowledge to user it's infra, not substance.
2. Do NOT try to shrink the PR to fit.
3. Flag as a cross-phase infra TODO — either cap `diff_size` in the workflow, switch to a larger-context provider, or chunk the diff before sending.

**CodeRabbit is separate** — it uses a different pipeline (Pro plan) and handles phase-size PRs fine. Wait for CodeRabbit regardless of AI-review status.

**Red-flag diff scan** is also separate — it's a regex scan for forbidden patterns (new routes, new migrations, etc.) that require maintainer ACK. Failing red-flag doesn't block merge if an ACK comment is posted on the PR.

## Status-check triage reference

| Check | Owner | Fails mean |
|-------|-------|-----------|
| `AI review` | actions/ai-inference workflow | Likely 413 on phase PRs; infra, not a review finding |
| `Red-flag diff scan` | regex scan workflow | New route/migration detected; post ACK comment |
| `CodeRabbit` | CodeRabbit Pro | Real review; read the actionable comments |
