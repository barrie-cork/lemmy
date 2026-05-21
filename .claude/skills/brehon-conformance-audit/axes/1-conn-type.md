---
axis: 1
title: "Conn-type / tx-boundary"
allowed-tools: [LSP, Read, Grep, Glob, Bash]
---

# Axis 1: Conn-type / tx-boundary

## What this catches

New federation handlers or helpers that take `&mut AsyncPgConnection` as their receiver but
call `.run_transaction(...)` — a method that lives only on `&mut DbConn<'_>`. Canonical
Phase-6 siblings four lines away take `&mut DbConn<'_>` and call `.run_transaction` correctly.
The divergence is a compile-caught bug class: Rust will not allow `.run_transaction` on the
wrong receiver type. The fix is to align the new function's parameter type with its sibling.

## Trace Up ↑

Invariant defined in: PMD `project_phase6_convention_divergence_class.md` axis #1

Source lesson: `.claude/lessons/feedback_multi_write_handlers_need_transactions.md`
(multi-write handlers must use `run_transaction`; any handler with 2+ Diesel writes wraps
them in `conn.run_transaction()`; receive type must be `&mut DbConn<'_>`)

Source lesson: `.claude/lessons/feedback_async_pool_test_pattern.md`
(`&mut AsyncPgConnection` is appropriate for already-inside-a-transaction callers;
`&mut DbConn<'_>` is the entry point that starts a transaction via `.run_transaction`)

Phase-6 instance: fix-impl-1 commit `cdff6f09d` — two helpers in
`crates/apub/activities/src/governance/inbox.rs` took `&mut AsyncPgConnection` but called
`.run_transaction`; same-file siblings four lines away took `&mut DbConn<'_>`.

## Trace Down ↓

Detection command(s):

1. Grep target file for `.run_transaction(` callers:
   `grep -n "\.run_transaction(" <target_file>`

2. For each `.run_transaction` caller, check the enclosing function signature:
   `grep -nB 20 "\.run_transaction(" <target_file> | grep -E "fn .*(conn|db|pool).*:"`

3. LSP hover to confirm receiver type at the `.run_transaction` call site:
   invoke `_hover` on the receiver expression in the function body.

4. Locate the in-file sibling that does the same operation correctly:
   `grep -n "fn receive_remote\|fn handle_governance\|fn publish_" <target_file>`
   then compare receiver types between new function and sibling.

## Error-code → design-question table

| Error / pattern | What it implies | Sibling pattern to mirror |
|---|---|---|
| `E0599: no method named 'run_transaction' found for mutable reference '&mut AsyncPgConnection'` | New fn takes `&mut AsyncPgConnection` but calls a `DbConn`-only method. Tx-boundary receiver is wrong type. | Change parameter to `&mut DbConn<'_>`. Callers pass `&mut get_conn(pool).await?` and call `conn.run_transaction(...)`. |
| `E0277: the trait bound 'AsyncPgConnection: DbPool' is not satisfied` | The function calls a method requiring `DbPool` (e.g. `get_conn`). `AsyncPgConnection` does not implement `DbPool`. | Use `&mut DbPool<'_>` (or `DbConn<'_>` alias) as the parameter type when the body needs pool-level methods. |
| `E0308: mismatched types` at a `.run_transaction` argument | The closure receives `&mut AsyncPgConnection` but was written expecting `&mut DbConn<'_>`. | Align closure argument type with what `.run_transaction` actually provides. |

## Evidence string format (cap 120 chars)

`axis-1: <file>:<line> new=<receiver-type> sibling=<sibling-file>:<line> sibling=<receiver-type>`

Example:
`axis-1: inbox.rs:412 new=&mut AsyncPgConnection sibling=inbox.rs:95 sibling=&mut DbConn<'_>`

## Hypothesis discipline (per `feedback_verify_automated_reviewer_claims_against_compiler.md`)

Every flag is a HYPOTHESIS until the compiler proves it (post-§15) OR a sibling diff shows new
code is strictly weaker than an enforced contract. Tier-1 = enforced-contract weakening with
sibling proof. Tier-2 = sibling divergence without proof of weakening. Tier-3 = stylistic.
