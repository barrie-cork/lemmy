---
name: Cheap-model arm drops ADR constraints
description: A cheaper model arm (MiniMax) silently dropped a hard ADR constraint that the brief listed but did not make load-bearing — enforce ADR gates at brief-author time, verify-gate time, and in any cheap-model preamble.
type: feedback
---

When dispatching a cheaper-model arm (or any model) on an impl-task whose file sits under an ADR-pinned path, a constraint that is merely *named* in the brief can be *rationalised away* by the model. The brief must make the constraint **load-bearing**: name the ADR, name the specific gate function + callsite, state WHY it can't be deferred, and require it as a DoD line.

**Why:** In the m1-b MiniMax A/B trial (n=2 tasks; AB data preserved at `.claude/PRPs/reports/minimax-ab-m1b-task3-task4.md`), the two arms diverged by task shape:
- **Task 3 (DTOs, MIRROR-ref-heavy):** MiniMax matched the canonical Claude/Sonnet arm exactly — same fields, derives, ts-rs gating, deliberate `Eq` omission. Functional parity.
- **Task 4 (handler with embedded ADR-015 gate):** MiniMax **dropped `validate_identity_policy`** — it wrote `//! No validate_identity_policy — that's Task 5`, i.e. it reasoned its way out of the ADR-015 pseudonymity pin and deferred it to the wrong task. The canonical arm both defined the gate AND called it in the write path. MiniMax also used clippy-denied `.expect()` accessors (would fail `-D warnings`).

The expensive failure is the ADR drop: it is **not a compile error** (validation can't catch it) — only ADR-review catches a silent governance omission. The `.expect()` issue is cheaper (clippy catches it). This is the worst error class for a cheap-model cutover because it passes the objective cargo gate that the trial relies on to "catch degradation."

This aligns with MiniMax's own #1 best practice (see [[reference-minimax-prompting-best-practices]]): "explain why a constraint matters, the model can choose better tradeoffs." A constraint with no rationale is one the model may trade away.

**How to apply:**
1. **Brief-author time (advisor):** when an impl-task touches a file under an ADR-pinned path (any handler that enforces ADR-013 EmergencyRemove, ADR-015 pseudonymity, etc.), the brief §4 MUST: (a) name the ADR + the specific gate fn/callsite, (b) state why it cannot be deferred, (c) add it as a DoD line. Per `advisor-orchestrator.md` §2.4a.
2. **Verify-gate time:** `/brehon-verify` greps the phase diff for any ADR-pinned handler and asserts the required gate fn is both DEFINED and CALLED. A defined-but-uncalled gate is a catch-fire. Per `brehon-verify.md` §ADR-gate check.
3. **Cheap-model preamble:** any MiniMax-arm dispatch prepends the 8 best-practices, foregrounding "explain why each constraint matters" + "explicit permission to refuse" (raise a DQ blocker rather than silently scope out). Per `minimax-m27-trial-1.md` §preamble.
4. **Swap caution:** "MiniMax is free" does NOT support a blanket Sonnet→MiniMax swap. Safe lane = pure pattern-following (DTOs, boilerplate, MIRROR-ref-heavy, no embedded ADR/governance logic). Logic-with-constraint tasks need either Sonnet or the full hardening above + ADR-review.
