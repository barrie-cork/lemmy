# fix-impl-2 Escalation — Cross-Platform dead_code Discrepancy

**What was attempted:** Applied 7 edits to `crates/apub/activities/src/governance/inbox.rs` per brief §2.3: 6× `#[expect(dead_code, reason=...)]` on the items the advisor's laptop clippy reported as dead code + 1× redundant-closure fix at ~L546.

**What failed:** The worker's cargo-clippy (Linux/EliteDesk) reported `unfulfilled_lint_expectations` (error under `-D warnings`) for 4 of the 6 `#[expect(dead_code)]` attributes: `GovernanceInboundActivity`, `rate_per_peer_counts`, `current_hour_bucket`, `log_inbox_drop`. These items are NOT dead code on the worker because they are referenced inside `wrap_governance_inbound`'s body — Rust's dead_code lint fires only on the ROOT uncalled item in a call chain (i.e., `wrap_governance_inbound` itself), not on functions that are called FROM within dead code. The brief's §4.5 cap of 7 named edits was exceeded if an 8th edit to remove the 4 incorrect `#[expect]` is counted. The `allow_attributes = "deny"` workspace lint blocks `#[allow(dead_code)]` as an alternative.

**What is committed:** 3 edits total (not 7): 2× `#[expect(dead_code)]` on the 2 items that ARE dead code on the worker (`rate_per_actor_counts` — no callers anywhere; `wrap_governance_inbound` — no callers anywhere) + 1× redundant-closure fix. Worker pre-push gates: `FI2_CLIPPY_EXIT_0`, `FI2_CHECK_EXIT_0` (logs at `.claude/PRPs/debug/fed-in-b-fi2-clippy-v2.log` and `fed-in-b-fi2-check-v2.log`).

**What is needed:** Advisor decision on DQ #284 (see `FI2_BLOCKER_DQ.json`):

- **Option A:** Advisor re-runs §15 `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings` on the laptop from a **COLD** build (delete or clear the `target/` incremental cache first, or run in a fresh checkout). If the laptop's cold build ALSO shows only 2 dead_code items (consistent with the worker's analysis), the committed 3-edit fix is complete for fix-impl-2 and the advisor can re-validate DQ #283 as passing. The prior laptop run showing 6 may have been stale-cache-influenced.

- **Option B:** If the laptop's cold rebuild genuinely still shows 4 extra dead_code items for those 4 functions/trait, the advisor needs to author a fix-impl-2b with a different suppression strategy. Possible approaches: (b1) Move the 4 items into a `#[cfg(feature = "governance-inbound-enforcement")]` gate and add a stub `mod` so they compile in all builds but are conditionally "used"; (b2) Add a dummy caller of each item in a `#[cfg(test)]` block; (b3) Reorganize so the callers are in a live function body (not inside dead `wrap_governance_inbound`); none of these are mechanical like the original 7 edits.

**Suggested next step:** Option A first (free, 2 min). If cold build shows the same 2-item dead_code set as the worker, the discrepancy was cache-driven and the fix is done.

_DQ #284 transcription needed into `.claude/decision-queue.json` pending[] by advisor (harness-gap DQ #235 procedure)._
