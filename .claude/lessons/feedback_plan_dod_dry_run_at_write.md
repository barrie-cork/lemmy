---
name: Plan generator should dry-run DoD commands at plan-write time
description: Plan agents should validate every DoD validation command is literally executable before committing the plan file. Phase 2b plan referenced cargo-check.bat clippy (nonexistent wrapper) — caught by pre-phase audit but should have been caught at plan-write time.
type: feedback
originSessionId: e31990ad-143c-4e12-9bfb-8b135b83d7e6
---
Plan generators should dry-run every validation command in the plan's DoD section against the current branch HEAD before committing the plan file.

**Why:** Phase 2b's plan referenced `scripts\brehon\cargo-check.bat clippy ...` as the clippy DoD command, but that wrapper only runs `cargo check`, not `cargo clippy`. The pre-phase harness audit at task 25 Step 1 caught it (5 minutes into the loop), but the plan agent could have caught it at plan-write time by running the command and observing the wrong output. This is the plan-generator-side complement to rule 11 (advisor runs DoD smoke test at plan review).

**How to apply:** Before committing a plan file, the plan agent runs each validation command from §11 (or equivalent DoD section) against the current HEAD. Commands targeting not-yet-created crates should fail with "crate not found" (expected red); commands targeting existing crates should succeed (expected green). Any command that fails for an unexpected reason (wrong wrapper, missing flag, unresolved path) blocks the plan commit until fixed. This discipline applies to all plan generators, not just Brehon phases.
