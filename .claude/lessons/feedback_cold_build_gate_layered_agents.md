---
name: Cold-build gate for rapid-commit layered-agents phases
description: Warm cargo incremental cache in target/ can land in a partial state between test-compile and lib-compile feature-resolution, producing transient clippy/compile false-reds. Gate commit-greenness on cold builds in rapid-commit phases.
type: feedback
originSessionId: 489fe821-667a-4d00-ba00-daf7f50e7584
---
In rapid-commit layered-agents phases (Phase 6 style — 7 agents, ~15 task
commits in ~hours), warm `target/` cache can land in a partial state
between test-compile and lib-compile feature-resolution. Symptom: a
clippy or check run false-reds on perfectly-correct code at HEAD; a cold
rebuild minutes later comes back clean with no intervening file change.

**Why:** Warm cache carries intermediate fingerprints across commits.
When a layered-agents phase pushes multiple crate-touching commits in
rapid succession, the fingerprint for a feature-gated path can get
stale while a downstream test-target compile has already baked the old
version into its metadata. Next clippy/check run picks one or the other
nondeterministically.

**How to apply:**

1. In rapid-commit phases (layered agents, ralph loops, multi-session
   parallel work), **do not trust warm-cache green**. Before marking a
   phase or PR ready for re-review, run a cold validation:
   ```bash
   cargo clean -p <crate>    # or cargo clean for the whole workspace
   scripts/brehon/cargo-check.bat --workspace --features full
   scripts/brehon/cargo-clippy.bat --workspace --no-deps --features full -- -D warnings
   scripts/brehon/cargo-test.bat --test e2e --no-run -p lemmy_server
   ```
2. If a clippy/check error points at a line you know is correct (and
   `git show HEAD:<file>` confirms it), don't chase phantom fixes. Run
   `cargo clean` and re-run before editing.
3. Reference incident: 2026-04-19 16:06Z clippy false-red on
   `publish_trust_attestation.rs:146-147` `Object` trait "not in scope"
   — file at HEAD had the import since `8057b9f65` (task 74). Cold
   rebuild at 16:19Z green with zero file change. See
   `project_pr46_phase_6_review_response.md`.

Memory candidate for phase-branch.md or a new pre-merge.md rule once the
pattern is seen twice more.
