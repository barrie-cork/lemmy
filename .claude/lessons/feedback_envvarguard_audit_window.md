---
name: EnvVarGuard audit window must accommodate multi-line comment blocks
description: Brief audit scripts that verify SAFETY comments above unsafe blocks need a look-back window sized to the full multi-line comment, not a fixed short count.
metadata:
  type: feedback
---

When a brief's task-1 validation script checks for a SAFETY comment above an `unsafe` block, a fixed-character or fixed-line look-back window may miss the full block if the SAFETY comment is multi-line.

**Why:** v1-quality-r3 task 1 added 5-line SAFETY comment blocks above `set_var` calls in `bootstrap()` fixtures. The brief's audit script used a 400-character look-back window — sufficient for single-line comments but too small to capture the full block. The DQ log entry `cca087156` (DQ `65b95cc574c8-001`, self-resolved) flagged this at task runtime.

**How to apply:** When authoring a brief that asks the impl-task to add multi-line SAFETY/justification comments, set the audit look-back to at least `(expected lines × max-line-length)`. For typical 5-line SAFETY blocks in Rust: `800` characters is a safe minimum. Alternatively, pattern-match on the struct/function boundary rather than a fixed byte count.

**See also:** `feedback_envvarguard_fixture_lifetime_footgun.md` — the companion lesson on why bootstrap sites stay raw (guard dropped before test body runs).
