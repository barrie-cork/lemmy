---
axis: 5
title: "Conn acquisition idiom"
allowed-tools: [LSP, Read, Grep, Glob, Bash]
---

# Axis 5: Conn acquisition idiom

## What this catches

New handlers or helpers that acquire a database connection using an improvised variant
instead of the canonical Lemmy pattern `let conn = &mut get_conn(pool).await?;`. Variants
to flag: calling `pool.get()` directly, using `context.pool()` without `get_conn()`,
using `context.pool().get()`, or any other indirect acquisition. The canonical form is
byte-conformant across all Phase-6 governance handlers. Deviation may introduce subtle
lifetime mismatches or bypasses the async pool's error handling.

## Trace Up ↑

Invariant defined in: PMD `project_phase6_convention_divergence_class.md` axis #5

Source lesson: `.claude/lessons/feedback_async_pool_test_pattern.md`
(the `&mut get_conn(pool).await?` pattern; `AsyncPgConnection::establish` is for tests;
`get_conn(pool).await?` is the canonical production pattern via `lemmy_diesel_utils::connection::get_conn`)

Phase-6 instance: Held — all Phase-6 handlers in
`crates/apub/activities/src/governance/inbox.rs` use `let conn = &mut get_conn(pool).await?;`
verbatim. No deviation was observed at fix-impl-1, fix-impl-2, or fix-impl-3.

## Trace Down ↓

Detection command(s):

1. Grep target file for all connection acquisitions:
   `grep -n "get_conn\|pool\.get\|context\.pool" <target_file>`

2. For each hit, check if it matches the canonical pattern:
   `grep -n "let \w\+ = &mut get_conn(" <target_file>`
   Flag any hit that is NOT in this form.

3. Read the surrounding context for flagged hits to understand if it's an in-transaction
   `conn` reuse (acceptable) or a new pool acquisition:
   `grep -nB 5 -A 5 "pool\.get\|context\.pool" <target_file>`

4. Compare against in-file sibling acquisition pattern:
   `grep -n "let conn = &mut get_conn" <target_file>`
   Expected: `let conn = &mut get_conn(pool).await?;` (exactly this form).

## Evidence string format (cap 120 chars)

`axis-5: <file>:<line> new=<variant> expected=let conn = &mut get_conn(...)`

Example:
`axis-5: inbox.rs:820 new=let conn = pool.get().await? expected=let conn = &mut get_conn(pool).await?`

## Hypothesis discipline (per `feedback_verify_automated_reviewer_claims_against_compiler.md`)

Every flag is a HYPOTHESIS until the compiler proves it (post-§15) OR a sibling diff shows new
code is strictly weaker than an enforced contract. Tier-1 = enforced-contract weakening with
sibling proof. Tier-2 = sibling divergence without proof of weakening. Tier-3 = stylistic.
