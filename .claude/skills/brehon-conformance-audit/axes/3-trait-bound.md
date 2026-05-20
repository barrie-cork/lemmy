---
axis: 3
title: "Trait-bound completeness"
allowed-tools: [LSP, Read, Grep, Glob, Bash]
---

# Axis 3: Trait-bound completeness

## What this catches

Generic functions annotated with `#[async_trait]` whose bodies execute code that requires
`Sync` (e.g. spawning tasks, crossing `await` points with a `Sync`-requiring bound) but
whose generic parameter lacks `+ Sync`. The compiler catches this at instantiation time —
the error appears when a concrete type (that happens to be `!Sync`) is used, not at the
definition site. Phase-6 instances were caught at fix-impl-1 because the function was
immediately instantiated in the same file with a concrete type. Latent versions (not
yet instantiated) would compile cleanly until a new non-`Sync` type is used.

## Trace Up ↑

Invariant defined in: PMD `project_phase6_convention_divergence_class.md` axis #3

Source lesson: `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A/B/C
enumeration; the `#[async_trait]` Sync + Send + 'static requirement is the analogue
of the error-shape uniformity invariant: get the bounds right at definition time.

Research doc §D2 — `#[async_trait]` methods with `Sync`-requiring bodies require
`A: Sync` on the generic. The bound is not inferred; it must be written explicitly.

Phase-6 instance: fix-impl-1 commit `cdff6f09d` — `wrap_governance_inbound<A: GovernanceInboundActivity>`
was missing `+ Sync`. The compiler caught this as `E0277` on two sites where a
concrete non-`Sync` type was instantiated.

## Trace Down ↓

Detection command(s):

1. Grep target file for `#[async_trait]` decorated generic functions:
   `grep -nB 2 "async fn.*<.*:.*Activity\|Policy\|Handler" <target_file> | grep -E "#\[async_trait\]|fn "`

2. For each generic function, check if the body has `await` points + uses the generic in a
   context that requires `Sync`:
   `grep -nA 30 "fn wrap_governance_inbound\|fn handle_governance" <target_file>`
   Look for: `tokio::spawn`, cross-`await` captures of `A`, or calls into `Sync`-requiring traits.

3. Compare the generic bounds against the in-file sibling:
   - Sibling: `pub async fn <name><A: GovernanceInboundActivity + Sync + ...>`
   - New fn:   `pub async fn <name><A: GovernanceInboundActivity>`   ← missing `+ Sync`

4. Compiler is the oracle — a `cargo check --workspace --features full` will surface any
   instantiation of the generic with a `!Sync` type. Planning-time MIRROR stubs should
   compile-check (`cargo check`) to catch this before impl-task dispatch.

## Error-code → design-question table

| Error / pattern | What it implies | Sibling pattern to mirror |
|---|---|---|
| `E0277: 'A' cannot be sent between threads safely` | Generic `A` lacks `Send` bound; async runtime requires `Send` for spawned futures. | Add `+ Send` to the generic bound: `<A: GovernanceInboundActivity + Send + Sync>`. |
| `E0277: the trait bound 'A: Sync' is not satisfied` | Generic `A` lacks `Sync` bound; the function body crosses an `await` point holding a reference to `A`. | Add `+ Sync` to the generic bound. Check sibling function for the complete bound set. |
| `E0277: 'A' cannot be shared between threads safely` (variant wording) | Same as `Sync` failure above; different Rust version may use different phrasing. | Add `+ Sync`. Mirror the sibling bound verbatim. |
| `E0277: future cannot be sent between threads safely` + note about generic | The enclosing `async fn` returns a non-`Send` future because `A` is not `Send`. | `<A: ... + Send + Sync + 'static>` is typically the full bound needed for async-trait dispatch. |

## Evidence string format (cap 120 chars)

`axis-3: <file>:<line> new=<generic-bounds> sibling=<sibling-file>:<line> sibling=<generic-bounds>`

Example:
`axis-3: inbox.rs:620 new=<A:GovernanceInboundActivity> sibling=inbox.rs:110 sibling=<A:GovernanceInboundActivity+Sync>`

## Hypothesis discipline (per `feedback_verify_automated_reviewer_claims_against_compiler.md`)

Every flag is a HYPOTHESIS until the compiler proves it (post-§15) OR a sibling diff shows new
code is strictly weaker than an enforced contract. Tier-1 = enforced-contract weakening with
sibling proof. Tier-2 = sibling divergence without proof of weakening. Tier-3 = stylistic.
