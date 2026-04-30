---
name: Re-run clippy after applying a clippy fix
description: Removing dead code can promote sibling bindings/assignments to also-stale state. The original lint was downstream of dead code; the fix can expose new linting findings on related lines. Always re-run clippy after applying any unused-* fix before pushing.
type: feedback
originSessionId: 18bed089-5b3f-400c-953f-ef5bf32cbbee
---
# Re-run clippy after applying a clippy fix

When a clippy fix changes the liveness, mutability, or reachability of a binding (or removes a write that previously justified `mut`/`let`/`const`), re-run clippy after the fix and before pushing. The original lint pointed at one site, but it was a downstream signal — the same dead-code pattern often has sibling findings nearby.

**Why:** Observed 2026-04-30 incident on `crates/api/api/src/governance/admin_assign_jury.rs`. Mac session committed `953a8360b` ("drop dead store + stale comment") which removed `current_geo_enabled = false;` on line 727 (the dead-store fix that closed clippy `unused-assignments`). At commit time, `mut` on the line-653 binding `let mut current_geo_enabled = geo_pref_enabled;` was *justified* — because line 727 *did* reassign it. The fix retroactively made `mut` stale. Mac did not re-run clippy after applying the fix.

The next workspace-check (run 25181904020) caught it on the GH runner — `unused-mut` on line 653, with `-D unused-mut` implied by `-D warnings`. Cost: one wasted GH workflow run (~21 min including queue), one diagnostic cycle by the advisor session, and one extra retrigger commit (`3f7dfec22`).

**How to apply:**
- After applying any clippy fix in the `unused-*` family (`unused-assignments`, `unused-variable`, `unused-mut`, `unused-import`, `dead-code`), re-run `cargo clippy -p <crate> --features <X> -- -D warnings` on the affected crate before staging the commit. Cost is the warm-cache reanalysis (~30s-2m for one crate).
- For broader hygiene fixes that touch multiple crates, run the full DoD validation that the workflow runs: `cargo clippy --workspace --features full --no-deps -- -D warnings`. ~4 min warm on the laptop, vs ~10-15 min on GH.
- This applies symmetrically to `cargo fix --workspace`: after a sweeping auto-fix run, re-run clippy without `--fix` to catch the second-order findings the first pass surfaced.
- For task-hopper bundles where multiple cluster fixes land in one commit (Mac's `953a8360b` style), add a final clippy gate as the last step before commit. Skipping it because "the previous step already ran clippy" misses the post-fix delta.
- This is also a §G4 classifier hint: when a `validate-pending` entry's `failed_jobs` reports `unused-*` warnings, the auto-queued narrow fix-impl-task brief should explicitly include "re-run `cargo clippy -p <crate>` locally after applying the fix and before pushing" in §3 Required reading.

**Related to:** the fact that GH workspace-check is ~3x slower than laptop clippy even on cache-hit — see `feedback_laptop_default_for_validate_pending.md`. The cost of a missed re-run amplifies because each retry round-trip is 10+ GH min.
