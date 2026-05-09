---
name: Plan generator should dry-run DoD commands at plan-write time
description: Plan agents should validate every DoD validation command is literally executable before committing the plan file. Phase 2b plan referenced cargo-check.bat clippy (nonexistent wrapper) — caught by pre-phase audit but should have been caught at plan-write time.
type: feedback
originSessionId: e31990ad-143c-4e12-9bfb-8b135b83d7e6
---
Plan generators should dry-run every validation command in the plan's DoD section against the current branch HEAD before committing the plan file.

**Why:** Phase 2b's plan referenced `scripts\brehon\cargo-check.bat clippy ...` as the clippy DoD command, but that wrapper only runs `cargo check`, not `cargo clippy`. The pre-phase harness audit at task 25 Step 1 caught it (5 minutes into the loop), but the plan agent could have caught it at plan-write time by running the command and observing the wrong output. This is the plan-generator-side complement to rule 11 (advisor runs DoD smoke test at plan review).

**How to apply:** Before committing a plan file, the plan agent runs each validation command from §11 (or equivalent DoD section) against the current HEAD. Commands targeting not-yet-created crates should fail with "crate not found" (expected red); commands targeting existing crates should succeed (expected green). Any command that fails for an unexpected reason (wrong wrapper, missing flag, unresolved path) blocks the plan commit until fixed. This discipline applies to all plan generators, not just Brehon phases.

**Extended scope (2026-05-09): plan-mode Verification lists.** The same dry-run discipline applies to plan files written in Claude Code plan-mode (`ExitPlanMode`-bound plans under `~/.claude/plans/<slug>.md`), not just Brehon implementation-plan §11 DoD sections. Before calling `ExitPlanMode`, walk the plan's "Verification" / "End-to-end checks" list against the plan's "Files to modify" delta and assert each verification step is consistent with the proposed changes. Two failure modes:

1. **Verification asserts a positive that the plan's delta removes.** Example (advisor-orchestrator rewrite, 2026-05-09): plan said "drop the mirror note" but verification step #3 said "Mirror note still present at top: `head -5 ... | grep 'Mirror note'`". Caught at runtime by inverting the check; would have been caught at plan-write time by reading the verification step against the delta.
2. **Verification asserts an exact count or shape that the plan's delta changes.** Example: plan says "562 → ~280 lines (50% reduction)" but verification says "wc -l target 250-300". 280 falls outside 250-300; the plan's target and the verification's range disagree. Caught only when the executed file lands at 400 and the plan's reduction estimate is exposed as over-promise.

**How to apply at plan-write time:** before `ExitPlanMode`, list every concrete assertion in the verification section. For each one, ask: "is this consistent with what 'Files to modify' will produce?" If no, fix the verification step (or fix the delta) before committing the plan to `ExitPlanMode`. The cost is ~30 seconds per verification step; the cost of a runtime contradiction is one inverted check + a moment of doubt about which spec is authoritative.

This generalises the original DoD-side rule: **every executable or grep-able assertion in a plan body is a hypothesis the plan author should have tested before shipping the plan**. DoD commands at plan-author time, verification steps at plan-mode-author time. Same discipline, different surface.
