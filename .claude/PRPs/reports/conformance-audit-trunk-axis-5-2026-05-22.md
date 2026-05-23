---
axis: 5
scope: trunk (governance-v0 @ 7e6c4202f)
date: 2026-05-22
mode: file (all governance files in three roots)
---

# Conformance audit — axis 5: conn acquisition idiom

## Scope

Three federation/governance roots on `governance-v0` @ `7e6c4202f`:

- `crates/apub/activities/src/governance/` (5 files)
- `crates/api/api/src/governance/` (31 files)
- `crates/db_schema/src/source/governance/` (26 files)

Total: 62 governance .rs files.

## Detection

Per `axes/5-conn-acquisition.md`:

1. Grepped `get_conn|pool\.get|context\.pool` across all three roots → 103 lines.
2. Filtered for `get_conn(` invocations (actual pool acquisition, not import/use lines).
3. Identified three surface forms in use:
   - **Canonical A** (parameter hand-off): `let conn = &mut get_conn(pool).await?;` — where `pool`
     is already `&mut DbPool<'_>` received as a function parameter. Used in db_schema helpers and
     `config.rs` helpers throughout.
   - **Canonical B** (handler entry-point, two-line): `let pool = &mut context.pool(); let conn = &mut get_conn(pool).await?;` — `pool` bound via immutable reborrow `&mut`. 20 sites.
   - **Variant** (mutable binding): `let mut pool = context.pool(); let conn = &mut get_conn(&mut pool).await?;` — `pool` bound with `let mut` then passed as `&mut pool`. 6 sites.
4. One additional variant in `admin_audit_stream.rs`: error-recovery `let Ok(mut conn) = get_conn(&mut pool).await else { continue };` inside an async notification loop.
5. Compared each variant site against sibling acquisition patterns in the same or adjacent files.

## Inventory of all conn acquisition sites (handler entry-point pattern)

### Canonical form — `let pool = &mut context.pool();` (20 sites, all conformant)

| File | Lines | Notes |
|---|---|---|
| `apub/.../governance/inbox.rs` | 193–194, 301–302, 506–507, 764–765 | 4 handler entry-points |
| `apub/.../governance/publish_trust_attestation.rs` | 140–141, 172–173 | 2 entry-points |
| `api/.../governance/accept_jury_assignment.rs` | 62–63 | canonical |
| `api/.../governance/admin_assign_jury.rs` | 106–107 | canonical |
| `api/.../governance/admin_close_case.rs` | 38–39 | canonical |
| `api/.../governance/admin_config.rs` | 1185–1186 | canonical (GET /audit endpoint) |
| `api/.../governance/admin_emergency_remove.rs` | (pool param → line 82) | pool passed as param, `get_conn(pool)` canonical |
| `api/.../governance/admin_rule_sets.rs` | 130–131, 291–292 | 2 entry-points canonical |
| `api/.../governance/admin_trigger_appeal_rejury.rs` | 44–45 | canonical |
| `api/.../governance/appeal_window_expiry.rs` | 37–38 | canonical |
| `api/.../governance/decline_jury_assignment.rs` | 53–54 | canonical |
| `api/.../governance/reputation_snapshot.rs` | 362 + 390 | canonical (pool reused across loop iterations) |
| `api/.../governance/sponsor_liability_grace.rs` | 130 + 142 | canonical |
| `api/.../governance/submit_jury_vote.rs` | 135–136 | canonical |
| `db_schema/.../governance/governance_log.rs` | 260 | canonical (pool is fn param) |

### Variant form — `let mut pool = context.pool();` (6 sites, 5 distinct handler fns)

| File | Lines | Current form | Expected form | Function |
|---|---|---|---|---|
| `admin_dashboard.rs` | 68–69 | `let mut pool = context.pool(); let conn = &mut get_conn(&mut pool).await?;` | `let pool = &mut context.pool(); let conn = &mut get_conn(pool).await?;` | `admin_dashboard` |
| `admin_dashboard_html.rs` | 73–79 | `let mut pool = context.pool(); … get_conn(&mut pool)` | canonical two-line | `admin_dashboard_html` |
| `admin_dashboard_html.rs` | 247–253 | same variant form | canonical two-line | `admin_audit_html` |
| `admin_reputation_stats.rs` | 84–85 | `let mut pool = context.pool(); let conn = &mut get_conn(&mut pool).await?;` | canonical two-line | `admin_reputation_stats` |
| `get_my_reputation.rs` | 37–38 | `let mut pool = context.pool(); let conn = &mut get_conn(&mut pool).await?;` | canonical two-line | `get_my_reputation` |
| `admin_audit_stream.rs` | 226–227 | `let mut pool = context_for_stream.pool(); let Ok(mut conn) = get_conn(&mut pool).await else { continue };` | closest-canonical: `let pool = &mut …; let conn = &mut get_conn(pool).await?;` (different error-handling in loop context) | inner loop body |

## Findings

### Tier classification

**Tier 1** (enforced-contract weakening with sibling proof): 0
- The variant (`let mut pool` / `&mut pool`) compiles and produces the same `&mut DbConn<'_>`
  as the canonical form. Trunk passed cargo check at exit 0. No observed weakening; no sibling
  is strictly stronger in a way that the compiler enforces against the variant.

**Tier 2** (sibling divergence without proof of weakening): 5 findings
- Five handler functions in the `api/api/src/governance/` root use `let mut pool = context.pool()`
  where siblings in the same directory (in several cases the same file, `admin_dashboard_html.rs`)
  use `let pool = &mut context.pool()`. The mutable binding form (`let mut pool`) vs the immutable
  reborrow form (`let pool = &mut`) differ in intent signal: the canonical `&mut` form communicates
  "this pool borrow is consumed once and its lifetime is constrained to this scope"; the `let mut`
  form implies mutation of the binding, which is misleading when only one `get_conn` call is made.
  No runtime difference; sibling proof is strong (20 canonical sites in the same directory).

**Tier 3** (stylistic, no sibling contrast): 1 finding
- `admin_audit_stream.rs:226–227`: uses `let Ok(mut conn) = … await else { continue }` in a
  streaming notification loop. This is a justified divergence — the loop needs continue-on-error
  rather than `?`-propagation, so `let Ok(…) else { continue }` is the appropriate error-handling
  idiom. The `let mut pool = context_for_stream.pool()` part is still Variant-form (Tier 2 class),
  but the `let Ok(mut conn)` error recovery is contextually correct and not comparable to sibling
  handlers that propagate errors. Classified Tier 3 because the handler semantics justify the
  divergence.

## Finding detail

### F1 — admin_dashboard.rs (Tier 2)

```
// current (line 68–69)
let mut pool = context.pool();
let conn = &mut get_conn(&mut pool).await?;

// canonical sibling (e.g. accept_jury_assignment.rs:62–63)
let pool = &mut context.pool();
let conn = &mut get_conn(pool).await?;
```

Evidence: `axis-5: admin_dashboard.rs:68-69 new=let mut pool = context.pool(); get_conn(&mut pool) expected=let pool = &mut context.pool(); get_conn(pool)`

Sibling ref: `api/api/src/governance/accept_jury_assignment.rs:62-63`, `admin_assign_jury.rs:106-107`.

### F2 — admin_dashboard_html.rs fn `admin_dashboard_html` (Tier 2)

```
// current (lines 73–79)
let mut pool = context.pool();
let enabled = get_bool(&mut cache, &mut pool, …).await?;
…
let conn = &mut get_conn(&mut pool).await?;

// pattern note: pool is reused for get_bool *before* get_conn; the mut
// binding is load-bearing for the intermediate get_bool call
```

Evidence: `axis-5: admin_dashboard_html.rs:73-79 new=let mut pool (reused for get_bool then get_conn) expected=let pool = &mut context.pool() form`

Note: The `let mut pool` here is functionally necessary because `get_bool` also borrows `pool`
as `&mut` before `get_conn` is called. This is a stronger justification than F1/F3/F4/F5 —
the caller must pass the same pool binding to multiple functions. Tier 2 is retained because the
sibling contrast exists and the `&mut` reborrow form is syntactically achievable, but the
finding is low-priority.

Sibling ref: same file `admin_audit_html` at line 247 also uses this form — no within-file
sibling contrast, but contrast is strong across the broader directory.

### F3 — admin_dashboard_html.rs fn `admin_audit_html` (Tier 2)

Same pattern as F2 at lines 247–253: `let mut pool = context.pool(); get_bool(&mut cache, &mut pool, …); get_conn(&mut pool)`.

Evidence: `axis-5: admin_dashboard_html.rs:247-253 new=let mut pool (reused for get_bool then get_conn) expected=canonical form`

Sibling ref: `accept_jury_assignment.rs:62-63`.

### F4 — admin_reputation_stats.rs (Tier 2)

```
// current (lines 84–85)
let mut pool = context.pool();
let conn = &mut get_conn(&mut pool).await?;

// NOTE: pool is also passed as &mut to get_int calls on lines 89–107
// (cache.get_int uses &mut pool for config reads before conn is used)
```

Evidence: `axis-5: admin_reputation_stats.rs:84-85 new=let mut pool = context.pool(); get_conn(&mut pool) expected=let pool = &mut context.pool(); get_conn(pool)`

Sibling ref: `admin_reputation_stats.rs` has no internal sibling — cross-file: `admin_assign_jury.rs:106-107`.

### F5 — get_my_reputation.rs (Tier 2)

```
// current (lines 37–38)
let mut pool = context.pool();
let conn = &mut get_conn(&mut pool).await?;
```

This is the simplest Tier-2 case: `pool` is never reused before `get_conn`; the `let mut`
binding is unnecessary.

Evidence: `axis-5: get_my_reputation.rs:37-38 new=let mut pool = context.pool(); get_conn(&mut pool) expected=let pool = &mut context.pool(); get_conn(pool)`

Sibling ref: `accept_jury_assignment.rs:62-63` (identical handler shape, canonical form).

### F6 — admin_audit_stream.rs (Tier 3)

```
// current (lines 226–227, inside async notification receive loop)
let mut pool = context_for_stream.pool();
let Ok(mut conn) = get_conn(&mut pool).await else { continue };
```

The `let Ok(…) else { continue }` error-recovery is justified in a non-`?`-propagating loop
context. The `let mut pool` is Variant-form but the caller is `context_for_stream` (a clone),
not `context`, and the loop requires `continue`-on-error. Contextually appropriate.

Evidence: `axis-5: admin_audit_stream.rs:226-227 new=let Ok(mut conn) = get_conn(&mut pool).await else {continue} (stream loop) tier=3 contextually-justified`

## Pattern observation: justified `let mut pool` (F2, F3, F4)

Three of the five Tier-2 findings (F2/F3/F4) use the `let mut pool` form because the same `pool`
binding is passed to a config-accessor function (`get_bool`, `get_int`) **before** `get_conn`.
This is a functional justification absent from F1 and F5 (where `pool` is only used for `get_conn`).
The canonical `let pool = &mut context.pool()` pattern is still achievable here (the reborrow is
mutable), but the divergence is less egregious than F1/F5. Fix priority: F5 > F1 > F2/F3/F4 > F6.

## Hypothesis discipline

All six findings are hypotheses under the axis-5 spec. Trunk passed `cargo check` at exit 0.
The `let mut pool` variant and the canonical `let pool = &mut` form produce identical
`&mut DbPool<'_>` types at the `get_conn` callsite — the compiler sees no difference.
No Tier-1 (enforced weakening) finding is raised because no sibling enforces a
compile-checked contract that the variant violates.

## Summary

| Tier | Count | Files |
|---|---|---|
| Tier 1 | 0 | — |
| Tier 2 | 5 | `admin_dashboard.rs`, `admin_dashboard_html.rs` (×2), `admin_reputation_stats.rs`, `get_my_reputation.rs` |
| Tier 3 | 1 | `admin_audit_stream.rs` |
| Total | 6 | 5 distinct files |

All findings are in `crates/api/api/src/governance/`. Zero findings in `crates/apub/activities/src/governance/` or `crates/db_schema/src/source/governance/`.
