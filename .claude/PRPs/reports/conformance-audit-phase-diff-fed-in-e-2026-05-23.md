# Conformance Audit — phase-diff v1-federation-inbound-e

**Run at:** 2026-05-23T09:15 UTC  
**Scope:** `phase-diff phase-v1-federation-inbound-e` (merge-base: `origin/governance-v0`)  
**Changed files (Rust/SQL):** `crates/apub/activities/src/governance/inbox.rs`, `crates/server/tests/e2e.rs`  
**Tier-1 scope files:** `inbox.rs` (yes), `e2e.rs` (test — not a Tier-1 governance file)

---

## Summary

**Findings: 0 Tier-1 | 0 Tier-2 | 0 Tier-3**

No conformance violations detected across all six axes. The phase-diff adds two new functions to `inbox.rs` (`acquire_evict_lock` + `evict_oldest_unreviewed_if_needed_in_tx`), refactors three existing callers, and appends a new e2e fixture module. All six axes pass cleanly against in-file siblings.

---

## Per-axis results

### Axis 1: Conn-type / tx-boundary — ✓ PASS

Detection: `grep -n "\.run_transaction(" inbox.rs` → lines 197, 318, 638, 820.

All four `.run_transaction` callers are in functions that acquire `conn` via `let conn = &mut get_conn(pool).await?;` — correct `DbConn`-equivalent entry point for `.run_transaction`.

`acquire_evict_lock` (new) takes `&mut AsyncPgConnection` — correct for an in-transaction helper (called inside the `run_transaction` closure where `conn` is already `AsyncPgConnection`). Does NOT call `.run_transaction` itself.

`evict_oldest_unreviewed_if_needed_in_tx` (renamed from `evict_oldest_unreviewed_if_needed`) also takes `&mut AsyncPgConnection` — correct. The rename removes the inner `run_transaction` wrapper, aligning with its callers' expectation of an in-tx helper.

**Evidence:** No new `.run_transaction` call site with wrong receiver type.

### Axis 2: Append reborrow shape — ✓ PASS

Detection: `grep -n "&mut (&mut \*conn)\.into()" inbox.rs` → lines 211, 330, 644, 731, 835.

All `governance_log::append` calls inside transactions use the canonical 4-token reborrow `&mut (&mut *conn).into()`. No deviation.

Line 644 — new `evict_oldest_unreviewed_if_needed_in_tx` body: `governance_log::append(&mut (&mut *conn).into(), ...)` ✓

**Evidence:** `axis-2: inbox.rs:644 new=&mut (&mut *conn).into() matches canonical`

### Axis 3: Trait-bound completeness — ✓ PASS

Detection: `wrap_governance_inbound<'a, F, Fut, A>` where clause includes `A: GovernanceInboundActivity + std::marker::Sync + 'a`.

No new generic functions added in this diff. `acquire_evict_lock` and `evict_oldest_unreviewed_if_needed_in_tx` are concrete (no generics). The existing `wrap_governance_inbound` Sync bound is intact.

**Evidence:** No new `#[async_trait]` generic or missing Sync bound.

### Axis 4: Error idiom at trust boundary — ✓ PASS

Detection: `grep -n "\.unwrap_or_default()\|\.unwrap_or(\"\|\.unwrap_or_else(String::new)" inbox.rs` → **0 hits**.

Diff-only lines (new code): zero `unwrap_or` variants. All trust-boundary field accesses in the new code use `?`-propagation or explicit error variants.

**Evidence:** 0 `.unwrap_or*` hits in target file and in phase diff.

### Axis 5: Conn acquisition idiom — ✓ PASS

Detection: `grep -n "let conn = &mut get_conn(" inbox.rs` → lines 189, 314, 444, 523, 792.

All main-tx connection acquisitions use the canonical form `let pool = &mut context.pool(); let conn = &mut get_conn(pool).await?;`. The `&mut context.pool()` pattern appears only at non-main-pool call sites (config reads, best-effort persist-failed appends outside the transaction).

**Evidence:** No `pool.get()` or improvised acquisition pattern in new code.

### Axis 6: ADR-015 pseudonym handling — ✓ PASS

Detection: `grep -n "actor_pseudonym" inbox.rs` → line 117 (doc comment only). No runtime `actor_pseudonym` field writes in the diff.

All `governance_log::append` calls pass `None` as the final (actor_pseudonym) argument, with explicit inline comment `// No local pseudonym for a remote actor; pass None per ADR-015.` on the primary in-tx call sites.

**Evidence:** All append calls: `actor_pseudonym = None`; diff adds zero non-None pseudonym writes.

---

## e2e.rs assessment

`crates/server/tests/e2e.rs` is a test file, not a Tier-1 governance module. No governance handler conventions apply. The new `mod v1_federation_inbound_e_fixtures` module follows the same `use super::*` + `LemmyResult<()>` pattern as sibling fixture modules. No audit findings.

---

## Required actions

None. Zero Tier-1/2/3 findings. No retro §3 actions required.

Advance to merge-confirm user gate (gate 5).
