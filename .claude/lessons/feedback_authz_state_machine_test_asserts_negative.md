---
name: Single-X-invariant tests must assert the negative, not just positive ordering
description: A state machine enforcing a single-X invariant (single presenter, single chair, single lock-holder, single active session) must have a test that asserts NO second holder remains — not merely that the correct holder was granted in the right order. An ordering-only assertion passes green while a concurrent-holder bug ships. cr-4 (PR #202) shipped a single-presenter gap that compiled, passed clippy, and passed its own marquee test.
type: feedback
---

When a state machine enforces a **single-X invariant** — single presenter, single chair, single mic-holder, single lock-owner, single active session — the test that guards it must assert the **negative side of the invariant** (no second holder remains active), not just the **positive side** (the right holder was granted, in the right order). An ordering-only assertion passes green while a concurrent-holder bug ships underneath it.

**Why:** the bug and the happy path produce the SAME positive trace. In Brehon m3-core-stage-mode (PR #202, CodeRabbit finding cr-4), the marquee `fifo_mic_pass_in_sequence` test drove 4 back-to-back `promote_next` → `on_activate` cycles and asserted the 4 `GrantPublish` commands fired in FIFO order (`["W1","W2","W3","W4"]`). That assertion passed. But `promote_next` and `chair_override(ForcePromote)` granted publish to the new participant **without revoking the prior `self.current` holder** — so 4 grants fired with 0 revokes, leaving up to 4 concurrent publishers. The single-presenter guarantee was broken, yet:

- `cargo check` passed (it compiles fine)
- `clippy -D warnings` passed (no lint)
- the marquee test passed (grant ORDER was correct)

The gap was invisible to every gate except CodeRabbit's human-grade review, because the missing revoke doesn't change the positive grant trace — it only changes what's left *un-revoked*, which the test never looked at. The grace path (`on_grace_expired`) DID revoke-before-promote; the asymmetry between the grace path and the direct-promote path was the trap.

**How to apply:** for any test guarding a single-X invariant, add an assertion over the FULL command/event sequence that confirms each handoff REVOKES (or releases / demotes / closes) the prior holder before granting the next. The cr-4 fix (commit `0258fef07`, `services/bridge/src/stage.rs`) added exactly this:

```rust
// Positive: grants still fire in FIFO order (the original assertion — keep it)
assert_eq!(grants, &["W1", "W2", "W3", "W4"], "GrantPublish must fire in FIFO order");

// NEGATIVE: each handoff revokes the prior holder BEFORE granting the next.
// windows(2) over the full command log catches the concurrent-holder gap.
assert!(
    sink.cmds.windows(2).any(|w|
        w[0] == GrantCmd::RevokePublish("W1".to_string())
        && w[1] == GrantCmd::GrantPublish("W2".to_string())),
    "handoff must RevokePublish(W1) before GrantPublish(W2)"
);
// ... repeat for W2→W3, W3→W4
```

The mechanical check: **does the test fail if you delete the revoke?** If deleting the invariant-enforcing operation leaves the test green, the test asserts the happy path, not the invariant. An invariant test must break when the invariant breaks.

This generalises beyond LiveKit grants — any "exactly one of X at a time" machine (a `pub fn acquire`/`release` lock, a `set_active`/`deactivate` session, a `promote`/`demote` chair) needs the negative assertion. Pair with `feedback_governance_type_state_handlers.md` (type-state guards centralise the WHO-can-transition check; this lesson covers the WHAT-must-also-happen-on-transition check — they're orthogonal).

**Recurrence:** 1× (cr-4, PR #202, 2026-06-19). Promoted at 1× because (a) the failure class is structural and recurring across any single-holder machine, (b) it survived three automated gates (cargo/clippy/own-test), and (c) m3-core-emergency-mute (Phase 4) ships another single-holder-adjacent control (`room_mute_all` drops all publishers) where the same negative-assertion discipline applies directly.
