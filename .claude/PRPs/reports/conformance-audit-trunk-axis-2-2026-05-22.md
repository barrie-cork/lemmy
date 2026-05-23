---
axis: 2
scope: trunk (governance-v0 @ 7e6c4202f)
date: 2026-05-22
mode: file (all governance files in three roots + api_crud/governance + tools/seed_founders)
---

# Conformance audit — axis 2: append reborrow shape

## Scope

Five federation/governance roots on `governance-v0` @ `7e6c4202f`:

- `crates/apub/activities/src/governance/` (5 files)
- `crates/api/api/src/governance/` (31 files)
- `crates/api/api_crud/src/governance/` (several files)
- `crates/db_schema/src/source/governance/` (26 files)
- `crates/tools/seed_founders/` (1 file, uses governance_log)

## Detection

Per `axes/2-append-reborrow.md`:

1. Grepped all governance roots for `governance_log::append` → 49 production callsites
   (excluding test files, doc comments, comment-only lines).
2. For each callsite, checked the first argument (connection form):
   - `grep -rn -A 1 "governance_log::append" crates/ | grep "conn\|pool"`
3. Mapped non-canonical forms to their enclosing function to determine tx context
   (inside `run_transaction(|conn| async move { ... })` closure body vs outside).
4. Cross-referenced each non-canonical callsite against same-file siblings.

## Connection forms — inventory

### Form A: canonical 4-token `&mut (&mut *conn).into()`

Used in 23 callsites. All are inside a `run_transaction` closure or a helper called
from one (conn type `&mut AsyncPgConnection`):

- `inbox.rs`: lines 205, 311, 629, 700, 801 — 5 callsites
- `admin_assign_jury.rs`: line 1212 (`seat_appeal_panel`, added v1-JM-d)
- `accept_jury_assignment.rs`: line 195
- `appeal_window_expiry.rs`: line 65
- `decline_jury_assignment.rs`: lines 117, 192
- `federation_outbox.rs`: line 196
- `reputation_snapshot.rs`: line 297
- `sponsor_liability.rs`: lines 438, 459
- `sponsor_liability_grace.rs`: lines 312, 495, 527
- `create_endorsement.rs` (api_crud): line 321
- `create_report.rs` (api_crud): lines 291, 306
- `request_appeal.rs` (api_crud): line 163
- `revoke_endorsement.rs` (api_crud): lines 317, 363

### Form B: 3-token `&mut conn.into()` inside tx — NON-CANONICAL

Used in 20 callsites across 6 files, all inside a `run_transaction` closure body
(conn type `&mut AsyncPgConnection`). All are `governance_log::append` first-argument
positions:

| File | Lines | Count | Same-file canonical sibling? |
|---|---|---|---|
| `admin_assign_jury.rs` | 237, 276, 290, 982 | 4 | YES — line 1212 uses 4-token |
| `admin_close_case.rs` | 91 | 1 | NO — only append call in file |
| `admin_config.rs` | 587 | 1 | YES — see pool-form sibling below; 4-token files in same crate |
| `admin_emergency_remove.rs` | 244, 281, 296, 320 | 4 | NO — all appends in file use 3-token |
| `admin_rule_sets.rs` | 233 | 1 | NO — only in-tx append call in file |
| `submit_jury_vote.rs` | 251, 401, 453, 547, 680, 697, 773, 876, 925 | 9 | YES — same file uses 4-token for config/pseudonym helpers (lines 560, 567, 618, 662) |

Note: `submit_jury_vote.rs` shows a consistent INTRA-FILE split: `governance_log::append`
uses 3-token; `config::get_int` and `actor_pseudonym_helper::get_or_create` use 4-token.
This suggests the 3-token form was the working pattern at writing time and the 4-token was
adopted later for config/pseudonym helpers without back-filling the append calls.

### Form C: pool form — CORRECT outside-tx usage

Used in 6 callsites. All are explicitly OUTSIDE a transaction:

| File | Line | Context |
|---|---|---|
| `inbox.rs` | 219 | "Best-effort persist_failed emit outside the rollback" |
| `inbox.rs` | 325 | "Best-effort persist_failed emit outside the rollback" |
| `inbox.rs` | 815 | "Best-effort persist_failed emit outside the rollback" |
| `admin_config.rs` | 844 | `emit_denial_log(pool: &mut DbPool<'_>)` — called OUTSIDE run_transaction per comment at line 820 |
| `admin_rule_sets.rs` | 394 | `emit_rule_set_denial_log(pool: &mut DbPool<'_>)` — called OUTSIDE run_transaction |
| `seed_founders/main.rs` | 193 | Top-level CLI tool, no transaction |

All Form C callsites are architecturally correct: `governance_log::append` opens its own
internal `run_transaction` (visible at `governance_log.rs:282`), so calling it with a pool
outside an outer tx is the expected non-transactional-caller pattern.

## Type system analysis

`governance_log::append` signature (line 252): `pub async fn append(pool: &mut DbPool<'_>, ...)`.

`DbPool<'a>` is an enum (`diesel_utils/src/connection.rs:48`):
```rust
pub enum DbPool<'a> {
  Pool(&'a ActualDbPool),
  Conn(&'a mut AsyncPgConnection),
}
```

Two `From` impls are relevant:
- `From<&'a mut AsyncPgConnection> for DbPool<'a>` (line 104) — `DbPool::Conn` variant
- `From<&'a mut DbConn<'b>> for DbPool<'a>` (line 110) — also `DbPool::Conn` via deref

For a parameter `conn: &mut AsyncPgConnection`:

- `&mut conn.into()` — Rust applies an implicit reborrow on the method call receiver;
  `conn` is temporarily re-borrowed as `&mut AsyncPgConnection`, converted via `From` to
  `DbPool::Conn`. The mutable reference is NOT consumed (the implicit reborrow expires at
  the expression boundary). This compiles and produces `DbPool::Conn`.

- `&mut (&mut *conn).into()` — explicitly creates a shorter-lived reborrow `&mut *conn`,
  converts it to `DbPool::Conn`. Semantically identical to the 3-token form when `conn`
  is `&mut AsyncPgConnection`.

Both forms produce `DbPool::Conn` wrapping the same underlying connection object.
The comment in `governance_log.rs:272` documents the 4-token form as canonical but the
3-token form is semantically equivalent and compiles cleanly (trunk cargo check passed,
exit 0, 9m 13s).

**Tier classification follows:** because both forms produce identical runtime behaviour —
`DbPool::Conn` wrapping the same `&mut AsyncPgConnection` — neither form can be Tier 1
(enforced-contract weakening). The only distinguishable difference is:

1. **Readability / intent signalling.** The 4-token form explicitly signals "I am
   re-borrowing to allow continued use of `conn` after this call" (per the Rust reborrow
   semantics note). The 3-token form relies on implicit re-borrow rules which are correct
   but less explicit.
2. **Documentation contract.** `governance_log.rs:272` names the 4-token form as the
   expected calling convention for in-tx callers. The 3-token form deviates from that
   documented expectation while remaining functionally equivalent.

## Findings

### Tier 1 (enforced-contract weakening with sibling proof)

**Count: 0**

No callsite uses a form that weakens an enforced contract. All 20 Form-B callsites are
inside a `run_transaction` closure (conn type correct); all Form-C callsites are outside
(pool form appropriate). No case exists where a 3-token in-tx callsite is adjacent to a
4-token sibling in a way that demonstrates the 3-token form is contractually weaker —
they produce identical `DbPool::Conn` at runtime.

### Tier 2 (sibling divergence without proof of weakening)

**Count: 20** (all Form-B callsites)

Every 3-token `&mut conn.into()` callsite for `governance_log::append` in an in-tx
position diverges from the documented canonical pattern (`governance_log.rs:272`) and
from same-crate sibling files that use `&mut (&mut *conn).into()`. No runtime weakening
is proven, but the divergence from a documented pattern + the existence of 4-token siblings
in the same or adjacent files satisfies Tier-2.

**Most notable Tier-2 divergence:** `submit_jury_vote.rs` — 9 callsites using 3-token
for `governance_log::append` within the SAME file that uses 4-token for `config::get_int`
and `actor_pseudonym_helper::get_or_create`. The intra-file inconsistency is strongest
evidence of a systematic rather than intentional pattern.

**Second notable Tier-2 divergence:** `admin_assign_jury.rs` lines 237/276/290/982 use
3-token while line 1212 (added in a later sub-phase, v1-JM-d) uses 4-token — this is a
within-file regression where newer code corrected the pattern but older code was not
back-filled.

### Tier 3 (stylistic only)

**Count: 0**

No findings are purely stylistic in isolation; all Form-B cases have sibling evidence
meeting the Tier-2 threshold.

## Summary

| Tier | Count | Files affected |
|---|---|---|
| Tier 1 | 0 | — |
| Tier 2 | 20 | admin_assign_jury.rs, admin_close_case.rs, admin_config.rs, admin_emergency_remove.rs, admin_rule_sets.rs, submit_jury_vote.rs |
| Tier 3 | 0 | — |

**Total non-canonical in-tx callsites: 20 (all Tier 2)**
**Total correct outside-tx pool-form callsites: 6 (not findings)**
**Total canonical 4-token in-tx callsites: 23**

## Hypothesis discipline

Per `feedback_verify_automated_reviewer_claims_against_compiler.md`: every flag is a
hypothesis until the compiler proves it or a sibling diff shows new code is strictly
weaker. Trunk cargo check passed exit 0 (confirmed in evidence snapshot). The 20 Tier-2
flags are hypothesis-grade: both forms compile and produce identical `DbPool::Conn`.
The finding is "documented pattern not followed" rather than "enforced contract broken".

Promotion to Tier 1 would require demonstrating that `&mut conn.into()` in a specific
call site produces a different lifetime or connection identity than `&mut (&mut *conn).into()`
— no such difference was found in the trait impls.

## Evidence strings

```
axis-2: admin_assign_jury.rs:237  new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: admin_assign_jury.rs:276  new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: admin_assign_jury.rs:290  new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: admin_assign_jury.rs:982  new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: admin_close_case.rs:91    new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: admin_config.rs:587       new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: admin_emergency_remove.rs:244  new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: admin_emergency_remove.rs:281  new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: admin_emergency_remove.rs:296  new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: admin_emergency_remove.rs:320  new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: admin_rule_sets.rs:233    new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: submit_jury_vote.rs:251   new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: submit_jury_vote.rs:401   new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: submit_jury_vote.rs:453   new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: submit_jury_vote.rs:547   new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: submit_jury_vote.rs:680   new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: submit_jury_vote.rs:697   new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: submit_jury_vote.rs:773   new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: submit_jury_vote.rs:876   new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
axis-2: submit_jury_vote.rs:925   new=&mut conn.into()  expected=&mut (&mut *conn).into()  [T2]
```
