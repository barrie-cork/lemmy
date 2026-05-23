---
axis: 1
scope: trunk (governance-v0 @ 7e6c4202f)
date: 2026-05-22
mode: file (all governance files in three roots)
---

# Conformance audit — axis 1: conn-type / tx-boundary

## Scope

Three federation/governance roots on `governance-v0` @ `7e6c4202f`:

- `crates/apub/activities/src/governance/` (5 files)
- `crates/api/api/src/governance/` (31 files)
- `crates/db_schema/src/source/governance/` (26 files)

Total: 62 governance .rs files.

## Detection

Per `axes/1-conn-type.md`:

1. Grepped `\.run_transaction\(` across all three roots → 17 callsites (excluding
   3 doc-comment mentions in `federation_outbox.rs`).
2. For each callsite, mapped the enclosing fn via awk-walk back to nearest
   `fn` signature.
3. Inspected each enclosing fn's `conn` receiver type.

## Inventory

17 `.run_transaction(` callsites:

| File:line | Enclosing fn | Receiver acquisition | Receiver type at call |
|---|---|---|---|
| `apub/activities/.../governance/inbox.rs:199` | `receive_remote_sanction_notice` | local `let conn = &mut get_conn(pool).await?` | `&mut DbConn<'_>` |
| `apub/activities/.../governance/inbox.rs:307` | `receive_remote_trust_attestation` | local `get_conn(pool)` | `&mut DbConn<'_>` |
| `apub/activities/.../governance/inbox.rs:622` | `log_inbox_drop` | param `conn: &mut DbConn<'_>` | `&mut DbConn<'_>` |
| `apub/activities/.../governance/inbox.rs:689` | `evict_oldest_unreviewed_if_needed` | param `conn: &mut DbConn<'_>` | `&mut DbConn<'_>` |
| `apub/activities/.../governance/inbox.rs:794` | `receive_remote_moderation_label` | local `get_conn(pool)` | `&mut DbConn<'_>` |
| `api/api/.../governance/accept_jury_assignment.rs:66` | `accept_jury_assignment` | local `get_conn(pool)` | `&mut DbConn<'_>` |
| `api/api/.../governance/admin_assign_jury.rs:113` | `admin_assign_jury` | local `get_conn(pool)` | `&mut DbConn<'_>` |
| `api/api/.../governance/admin_close_case.rs:45` | `admin_close_case` | local `get_conn(pool)` | `&mut DbConn<'_>` |
| `api/api/.../governance/admin_config.rs:522` | `admin_set_config` | local `get_conn(pool)` (verified) | `&mut DbConn<'_>` |
| `api/api/.../governance/admin_emergency_remove.rs:88` | `emergency_remove_open_case` | local `get_conn(pool)` | `&mut DbConn<'_>` |
| `api/api/.../governance/admin_rule_sets.rs:140` | `admin_create_rule_set` | local `get_conn(pool)` | `&mut DbConn<'_>` |
| `api/api/.../governance/admin_rule_sets.rs:296` | `admin_list_rule_sets` | local `get_conn(pool)` | `&mut DbConn<'_>` |
| `api/api/.../governance/admin_trigger_appeal_rejury.rs:51` | `admin_trigger_appeal_rejury` | local `get_conn(pool)` | `&mut DbConn<'_>` |
| `api/api/.../governance/decline_jury_assignment.rs:60` | `decline_jury_assignment` | local `get_conn(pool)` | `&mut DbConn<'_>` |
| `api/api/.../governance/sponsor_liability_grace.rs:173` | `run_grace_check_batch` | local `get_conn(pool)` | `&mut DbConn<'_>` |
| `api/api/.../governance/sponsor_liability_grace.rs:288` | `fire_or_escape_case` | param `conn: &mut DbConn<'_>` | `&mut DbConn<'_>` |
| `api/api/.../governance/submit_jury_vote.rs:148` | `submit_jury_vote` | local `get_conn(pool)` | `&mut DbConn<'_>` |
| `db_schema/.../governance/governance_log.rs:283` | `append` | local `let conn = &mut get_conn(pool).await?` | `&mut DbConn<'_>` |

## Helper fns taking `&mut AsyncPgConnection`

48 fns across the three roots take `&mut AsyncPgConnection`. **NONE call
`.run_transaction(`.** They are closure-body helpers invoked from inside an
outer `run_transaction(|conn| ...)` callback (the callback yields
`&mut AsyncPgConnection` — `diesel-async` convention). This matches the
canonical Phase-6 pattern documented in
`feedback_async_pool_test_pattern.md` and `axes/1-conn-type.md`'s "Trace Up"
section.

Sample (all OK per axis-1):

- `process_accept(conn: &mut diesel_async::AsyncPgConnection)` —
  `accept_jury_assignment.rs:75`
- `count_active_cases(conn: &mut AsyncPgConnection)` — `admin_dashboard.rs:125`
- `process_label_persist(conn: &mut AsyncPgConnection)` —
  `inbox.rs:391` (called inside `run_transaction` at line 794)
- `process_attestation_persist(conn: &mut AsyncPgConnection)` —
  `inbox.rs:405` (called inside `run_transaction` at line 307)
- All `reputation_snapshot.rs` helpers (11 callsites, lines 204…774) —
  invoked from inside an outer `run_transaction` higher up the call chain.

## Findings

**Tier 1:** 0
**Tier 2:** 0
**Tier 3:** 0

No divergences. The Phase-6 convention (run_transaction starter on
`&mut DbConn<'_>`, closure-body helpers on `&mut AsyncPgConnection`) is held
across all 17 callsites and all 48 helper fns in the three federation roots.

## Hypothesis discipline

All flags would have been hypotheses until §15 cargo check confirmed. With
zero flags raised, no compiler oracle is needed for axis-1.

## Evidence string

(None — no findings.)
