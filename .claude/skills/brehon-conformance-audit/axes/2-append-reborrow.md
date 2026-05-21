---
axis: 2
title: "Append reborrow shape"
allowed-tools: [LSP, Read, Grep, Glob, Bash]
---

# Axis 2: Append reborrow shape

## What this catches

In-transaction calls to `governance_log::append` (and similar DB-append helpers) must
reborrow the connection as `&mut (&mut *conn).into()` — the canonical 4-token sequence
that adapts a `&mut AsyncPgConnection` (inside a transaction closure) into the expected
pool-like type. New code that passes the connection directly or constructs a different
reborrow expression diverges from the byte-conformant sibling pattern and may fail at
compile time or introduce a subtle lifetime error.

## Trace Up ↑

Invariant defined in: PMD `project_phase6_convention_divergence_class.md` axis #2

Source lesson: `.claude/lessons/feedback_multi_write_handlers_need_transactions.md`
(canonical transaction pattern; all DB writes inside `conn.run_transaction(|conn| async move { ... })`
use `conn` directly as `&mut AsyncPgConnection`; the reborrow is required when calling
helpers that take a pool type)

Phase-6 instance: Held — all existing Phase-6 `governance_log::append` callers in
`crates/apub/activities/src/governance/inbox.rs` and
`crates/db_schema/src/source/governance/` files use the canonical reborrow. No deviation
was observed at fix-impl-1 or fix-impl-3.

## Trace Down ↓

Detection command(s):

1. Grep target file for `governance_log::append` or similar in-tx append calls:
   `grep -n "governance_log::append\|::append(" <target_file>`

2. For each append call, check the connection argument:
   `grep -nA 3 "::append(" <target_file> | grep "conn"`
   Expected token sequence: `&mut (&mut *conn).into()`

3. Pattern-grep for the canonical 4-token sequence across the target file:
   `grep -n "&mut (&mut \*conn)\.into()" <target_file>`
   Any append call NOT producing this sequence is a divergence candidate.

4. Read the surrounding sibling calls to confirm the canonical form:
   `grep -nB 2 -A 2 "&mut (&mut \*conn)\.into()" <target_file>`

## Evidence string format (cap 120 chars)

`axis-2: <file>:<line> new=<token-sequence> expected=&mut (&mut *conn).into()`

Example:
`axis-2: inbox.rs:742 new=conn expected=&mut (&mut *conn).into()`

## Hypothesis discipline (per `feedback_verify_automated_reviewer_claims_against_compiler.md`)

Every flag is a HYPOTHESIS until the compiler proves it (post-§15) OR a sibling diff shows new
code is strictly weaker than an enforced contract. Tier-1 = enforced-contract weakening with
sibling proof. Tier-2 = sibling divergence without proof of weakening. Tier-3 = stylistic.
