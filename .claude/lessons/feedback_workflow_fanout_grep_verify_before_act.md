---
name: workflow-fanout-grep-verify-before-act
description: For any workflow fan-out agent verdict classifying a code path as stale/delete/unused, grep-verify the claim against the live codebase before acting. Workflow agents reason about a snapshot that may not match HEAD.
metadata:
  type: feedback
---

For any workflow fan-out verdict that classifies a specific code path as **stale**, **can-delete**, **unused**, or **superseded**, run a targeted `grep` against the live codebase BEFORE making the edit. Workflow agents batch-classify against a snapshot of context; the codebase may have changed since they loaded it.

**Why:** Two occurrences in a single Phase 8 TODO sweep session:

1. **`v0-polish-ignore` false alarm** — workflow verdict suggested updating `#[ignore]` file paths. On grep inspection, the tests were already at the correct post-Phase-6 path (`crates/server/tests/e2e/governance.rs`). The workflow had reasoned about the pre-Phase-6 monolith structure.
2. **Federation cron TODO** — workflow classified `// TODO(v1-federation-inbound-b): wired by replay-cleanup cron` as stale. Grep confirmed the cron IS wired at `crates/routes/src/utils/scheduled_tasks.rs:405-465`. Delete was correct, but only after verification — not on the verdict alone.

**How to apply:**

- Before deleting a TODO comment: `grep -n '<the claim the TODO references>' crates/` to confirm the claim is satisfied (or unsatisfied).
- Before marking a code path as unused: `grep -rn '<identifier>' crates/` to confirm no live callers exist.
- Before acting on a "file already updated" verdict: `grep` or `Read` the file to confirm the stated state is real.

The grep takes 5–10 seconds. Acting on a stale verdict and having to revert costs 10+ minutes plus a dirty commit history.

**Scope:** applies to all workflow fan-out results where the verdict makes a factual claim about the current state of a specific file, identifier, or path. Does NOT apply to verdicts that are purely structural (e.g. "this TODO belongs in category v2-cleanup" — category classification doesn't need live verification, only factual "is this wired?" claims do).
