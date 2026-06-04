---
name: Verify automated-reviewer (CodeRabbit/Copilot) trait/type claims against the compiler before acting
description: CR and Copilot trait/type/lifetime claims are hypotheses, not contracts — run cargo check --workspace before triaging such a finding as fix-in-pr; a confidently-worded review suggestion can break a compile
type: feedback
originSessionId: reconstructed-2026-06-04-v1-closeout-phase1d
---

> **Reconstruction note (2026-06-04, v1-closeout Phase 1d):** cited by the always-load rule `.claude/rules/advisor-orchestrator.md` §5.4 (the falsifiable-hypothesis CR-finding variant) and indexed in MEMORY.md, but absent on disk (the GAP-2 broken-citation class). Reconstructed from the §5.4 rule text + the PR#132 Diesel `LimitDsl` incident. Behaviour is unchanged — §5.4 already operationalises it; this file is the cited backing.

Rule: a CodeRabbit or Copilot finding that makes a **trait / type / lifetime / API-shape claim** ("use `.first()` instead of `.get(0)`", "this should be `&str` not `String`", "add `#[derive(Eq)]`", "this lifetime is redundant") is a **hypothesis, not a contract**. Before triaging it as `bucket: fix-in-pr` and queueing a fix, **compile-check the claim**: `cargo check --workspace` (or the narrowest crate that covers the cited line). If the suggested change doesn't compile, the finding is wrong — bucket it `rebut` with the compiler error as evidence, do NOT apply it.

**Why:** automated reviewers reason from pattern-matching over surface syntax, not from your crate's actual trait resolution. They are confidently right about generic idioms and confidently wrong about anything where a local trait impl, a Diesel DSL, a macro expansion, or a feature-gate changes what the types actually are. The wording carries no uncertainty, so a finding that would break the build reads identically to one that improves it — the only discriminator is the compiler.

Canonical incident (**PR#132**): CodeRabbit flagged `.get(0)` and recommended the idiomatic `.first()`. Applied blindly, it **broke the Diesel `LimitDsl` query** — in that context `.get(0)` was operating on a Diesel query-builder type where `.first()` resolves to a *different* trait method (`RunQueryDsl::first`, which executes the query) rather than `slice::first`. The "idiomatic cleanup" changed query semantics and failed to compile / changed behaviour. A `cargo check` before triage would have caught it instantly; instead it shipped into the fix-in-pr bucket and had to be reverted. (See also `feedback_audit_trait_derive_validate_field_types.md` for the derive variant: CR/Copilot suggesting `#[derive(Eq, Hash, Ord)]` on a struct containing an `f64` — doesn't compile, `f64` is not `Eq`/`Ord`.)

**How to apply:** during `bm-triage` / CR-finding triage (the four-bucket pass), for every finding tagged trait/type/lifetime/derive/API-shape:

1. Identify the narrowest cargo target covering the cited `file:line` (the crate, or `--workspace` if cross-crate).
2. Apply the suggested change on a throwaway basis (or reason it through against the actual types) and run `cargo check`.
3. Compiles + behaviour-preserving → `bucket: fix-in-pr`. Doesn't compile, or changes a trait method's meaning (Diesel DSL, `RunQueryDsl` vs `slice`, macro-generated impls) → `bucket: rebut`, paste the compiler error as the rebuttal evidence.
4. Pure-prose / style / naming findings with no type claim are exempt — they can't break a compile, triage them normally.

This is the same falsifiable-hypothesis discipline `advisor-orchestrator.md` §5.4 applies to structural-fix DQs, narrowed to the automated-reviewer surface: the reviewer's RCA is a claim to test, not an instruction to execute. The compiler is the oracle; the reviewer is a lead.

## See also

- `.claude/rules/advisor-orchestrator.md` §5.4 (falsifiable-hypothesis gate, CR-finding variant — the rule this backs)
- `feedback_falsifiable_hypothesis_before_structural_fix.md` — the parent discipline (verify a named-defect hypothesis before structural work)
- `feedback_audit_trait_derive_validate_field_types.md` — the derive-specific variant (`Eq`/`Hash`/`Ord` on `f64` doesn't compile)
- `feedback_advisor_cr_enum_drift.md` — a related CR-claims-vs-reality case for enum variants
- `.claude/rules/branch-manager.md` "What BM should refuse" + the four-bucket triage this slots into
