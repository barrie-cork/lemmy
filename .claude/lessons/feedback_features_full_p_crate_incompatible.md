---
name: --features full is incompatible with -p <crate> in Lemmy workspace
description: cargo check/test/clippy with -p lemmy_server (or any single crate) cannot also pass --features full — feature unification only works with --workspace. Plan DoD commands must not combine the two.
type: feedback
originSessionId: ab63cf5b-ba84-4a96-8382-b2d4d0b99b51
---
`--features full` only works with `--workspace` in the Lemmy workspace. Using `-p lemmy_server --features full` (or `-p lemmy_api --features full`) fails because partial feature unification doesn't propagate `full` to transitive deps that need it.

**Why:** This has caused plan-drift fixes in Phase 5a (task 0 dropped `--features full` from `-p lemmy_server`), Phase 5b (DQ #15 narrowed Level 2 parity DoD to remove `-p lemmy_api --features full`), and Phase 5c (DQ #14, same class). Three occurrences — promoted from "candidate feedback memory" per RLS auto-promote rule.

**How to apply:** At plan-write time, grep every DoD command for `-p <crate>` combined with `--features full`. If found, either (a) drop `--features full` from the `-p` invocation, or (b) switch to `--workspace --features full`. The planning agent and plan-review pass must both check this. The impl should not need to fix this at execution time — it should be caught before the plan is committed.
