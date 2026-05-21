---
axis: 4
title: "Error idiom at trust boundary"
allowed-tools: [LSP, Read, Grep, Glob, Bash]
---

# Axis 4: Error idiom at trust boundary

## What this catches

New code at a federation trust boundary (receiving data from a remote actor) that uses
`.unwrap_or_default()` on `Option<String>` or `Result<String, _>` fields — silently
substituting an empty string when the field is absent — instead of the hard-error pattern
`.ok_or_else(|| LemmyErrorType::Unknown(...))?` (or equivalent) used by same-file siblings.
The canonical siblings guard against malformed or missing remote data by propagating errors;
the weaker pattern silently persists data-integrity footguns (e.g. `source_instance = ''`
in the DB). This is the Phase-6 Finding 6.1 defect class: **latent, compile-clean, only
caught by explicit sibling-diff review**.

**Cross-reference Track B (Clippy):** the per-module `#![deny(clippy::disallowed_methods)]`
in `crates/apub/activities/src/governance/mod.rs`,
`crates/api/api/src/governance/mod.rs`, and
`crates/db_schema/src/source/governance/mod.rs` catches the mechanical
`Option::unwrap_or_default` / `Result::unwrap_or_default` pattern at compile time in
those federation module roots. This axis (#4) catches the **sibling-divergence judgment
cases** the compiler cannot: a new function that uses `.ok_or_else(...)` but with a
weaker error type than its sibling, or that uses a different method to achieve the same
silent-discard effect.

## Trace Up ↑

Invariant defined in: PMD `project_phase6_convention_divergence_class.md` axis #4

Source lesson: `.claude/lessons/feedback_lemmy_error_no_std_error.md` — recipe source for
Clippy `disallowed_methods`; Case A/B/C enumeration for error-shape uniformity within a
module. The trust-boundary error idiom is the federation-domain application of the same
type-shape uniformity principle.

Research doc §D4 — federation trust-boundary `.unwrap_or_default()` detection; per-module
`disallowed_methods` deny as the compiler-mechanical layer.

ADR-013 — `CaseStatus::EmergencyRemove` mandatory; axis #4 enforcement preserves this
invariant by ensuring missing or malformed remote governance signals fail loudly rather
than silently persisting empty-string data that could mask a required `EmergencyRemove`
signal.

ADR-015 — `actor_pseudonym` for local actors; `None` for remote. Adjacent to axis #4:
a remote actor that provides unexpected `actor_pseudonym` data must be hard-rejected, not
silently accepted. (See axis #6 for the dedicated ADR-015 check.)

Phase-6 instance: Finding 6.1 — `receive_remote_moderation_label` at
`crates/apub/activities/src/governance/inbox.rs:~735` used
`.domain().map(str::to_string).unwrap_or_default()`, would have persisted
`source_instance = ''` for any domain-less remote actor. Siblings four lines away:
`receive_remote_sanction_notice` + `receive_remote_trust_attestation` use
`.domain().ok_or_else(|| LemmyErrorType::Unknown("...".to_string()))?`.
Closed at fix-impl-3 SHA `8b04e69a6`.

## Trace Down ↓

Detection command(s):

1. Grep target file for `.unwrap_or_default()` at trust-boundary call sites:
   `grep -n "\.unwrap_or_default()" <target_file>`

2. For each hit, check if it's on an `Option<String>` or `Result<_,_>` from a remote-actor
   field access (e.g. `.domain()`, `.id()`, `.actor_id()`, `.name()`):
   `grep -nB 5 "\.unwrap_or_default()" <target_file>`

3. Locate the in-file sibling doing the same field access with the hard-error pattern:
   `grep -n "\.ok_or_else(\|LemmyErrorType::" <target_file>`
   Compare: same field → sibling uses `.ok_or_else(|| LemmyErrorType::*)?` → flag divergence.

4. Also check for `.unwrap_or("")` (explicit empty-string default) and
   `.unwrap_or_else(String::new)` as equivalent silent-discard patterns:
   `grep -n "\.unwrap_or(\|\.unwrap_or_else(" <target_file>`

## Error-code → design-question table

| Error / pattern | What it implies | Sibling pattern to mirror |
|---|---|---|
| `clippy::disallowed_methods`: `use of disallowed method 'core::option::Option::unwrap_or_default'` | Trust-boundary `Option` silently replaced with empty. Compiler-caught in federation module roots via Track B. | Replace with `.ok_or_else(\|\| LemmyErrorType::Unknown("missing field".to_string()))?`. Mirror the exact sibling `LemmyErrorType` variant. |
| `clippy::disallowed_methods`: `use of disallowed method 'core::result::Result::unwrap_or_default'` | Trust-boundary `Result` silently replaced with default. Compiler-caught in federation module roots. | Replace with `?` propagation or `.map_err(\|e\| LemmyErrorType::Unknown(format!("{e}")))?`. |
| Pattern-grep hit: `.unwrap_or_default()` on `.domain()` / `.actor_id()` / `.name()` at a remote-actor handler | Latent data-integrity footgun (compile-clean). Not caught by Track B unless in a federation module root. Tier-1 when sibling uses `.ok_or_else()`. | Mirror sibling: `.domain().ok_or_else(\|\| LemmyErrorType::Unknown("expected domain for remote actor".to_string()))?`. |
| Pattern-grep hit: `.unwrap_or("")` or `.unwrap_or_else(String::new)` on a required remote field | Equivalent silent-discard pattern; not caught by `disallowed_methods` because it's not using the named method. | Same fix as `unwrap_or_default`. Mirror sibling's explicit error variant. |

## Evidence string format (cap 120 chars)

`axis-4: <file>:<line> new=.unwrap_or_default() sibling=<sibling-file>:<line> sibling=.ok_or_else(|| LemmyErrorType::*)?`

Example:
`axis-4: inbox.rs:735 new=.unwrap_or_default() sibling=inbox.rs:110 sibling=.ok_or_else(|| LemmyErrorType::Unknown(...))?`

## Hypothesis discipline (per `feedback_verify_automated_reviewer_claims_against_compiler.md`)

Every flag is a HYPOTHESIS until the compiler proves it (post-§15) OR a sibling diff shows new
code is strictly weaker than an enforced contract. Tier-1 = enforced-contract weakening with
sibling proof. Tier-2 = sibling divergence without proof of weakening. Tier-3 = stylistic.
