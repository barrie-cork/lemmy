---
name: Verify automated-reviewer (CodeRabbit / Copilot) trait/type/lifetime claims against the compiler before triaging fix-in-pr
description: CodeRabbit and Copilot findings that assert a trait bound, type, lifetime, or API-shape claim are hypotheses, not contracts. Before triaging such a finding into bucket:fix-in-pr (i.e. before queueing a fix that follows the reviewer's recommendation), compile-check it (cargo check --workspace). A recommendation that compiles wrong is worse than the original — PR#132's CR-recommended .get(0)→.first() broke Diesel's LimitDsl resolution.
type: feedback
---

Automated code reviewers — CodeRabbit, GitHub Copilot — produce findings that *sound* authoritative, especially about Rust traits, types, lifetimes, and API shapes ("this should use `.first()` instead of `.get(0)`", "this lifetime is unnecessary", "this trait bound is redundant"). Treat any such claim as a **hypothesis to verify against the compiler**, not a contract to implement. Before triaging the finding into `bucket: fix-in-pr` — i.e. before you queue a fix that applies the reviewer's recommendation — **compile-check it**: `cargo check --workspace` (or the targeted crate). A reviewer recommendation that *compiles wrong*, or changes type-inference in a way the reviewer didn't model, is worse than the code it flagged, because the fix-in-pr commit then ships a regression with a clean-looking provenance ("addressed CR finding").

This is the **CR-finding variant** of the falsifiable-hypothesis gate (`.claude/rules/advisor-orchestrator.md` §5.4; `feedback_falsifiable_hypothesis_before_structural_fix.md`): a DQ or finding that names a specific code path as the defect site carries an RCA that is a hypothesis, and the cheap falsification runs *before* the fix path is chosen. For automated reviewers the falsification is a compile, and it's even cheaper than the structural-fix-DQ case — one `cargo check` settles it.

## Why this matters (PR#132 incident)

PR#132: CodeRabbit flagged a `.get(0)` call and recommended `.first()` — the idiomatic Rust swap, which is correct *in isolation* for slices/Vecs. But the call site was a **Diesel query builder**, where `.get(0)` and `.first()` resolve through different trait paths. Applying the CR recommendation broke Diesel's `LimitDsl` resolution — the "fix" did not compile / changed the query semantics. The recommendation was idiomatically reasonable and contextually wrong, and the only thing that would have caught it before it shipped as a fix-in-pr commit was a `cargo check` against the actual change — which is exactly the gate this lesson exists to enforce.

The trap is that automated-reviewer findings have a **veneer of correctness**: they cite the right lint, use the right vocabulary, and recommend the idiomatic form. That veneer is precisely why the claim slips past triage without verification — it *reads* like a settled fact. But the reviewer does not run the compiler against your tree; it pattern-matches. Trait resolution, type inference, and lifetime elision are context-sensitive in ways a pattern-matcher misses. The compiler is the authority; the reviewer is a lead to check.

This is the same class as `feedback_audit_trait_derive_validate_field_types.md` (an audit finding that recommended `Eq`/`Hash`/`Ord` derives on a struct with an `f64` field — doesn't compile; validate field types before promoting the finding). In both, an automated/audit recommendation about Rust trait machinery is plausible and wrong, and a compile-check is the discriminator.

## How to apply (advisor, at CR triage)

At `bm-triage` (the four-bucket CR triage, gate 3), when classifying a finding into a bucket:

1. **Identify trait/type/lifetime/API-shape claims.** Any finding that asserts "use trait X", "this type should be Y", "this lifetime/bound is unnecessary", "this method/derive is wrong" is a verifiable-against-compiler claim. (Findings about naming, formatting, comments, docs, or governance semantics are a different class — those don't get a compile-check, they get human judgment.)
2. **Before bucketing such a claim as `fix-in-pr`, compile-check it.** Apply the recommendation locally (or reason precisely about whether it compiles) and run `cargo check --workspace` (laptop, per `project_laptop_canonical_cargo_runner.md`). If it compiles clean AND preserves semantics → `fix-in-pr` is safe. If it breaks compilation or changes type-inference/query-semantics → **`bucket: rebut`** with the compiler error as the evidence, and post that in the triage comment so the bot is anchored and won't re-flag.
3. **Never queue a fix-impl that blindly applies a reviewer's trait/type recommendation.** The fix-impl brief must reflect the *verified* change, not the raw recommendation. If the recommendation didn't survive the compile-check, the finding is a rebuttal, not a fix.

The cost is one `cargo check` per trait/type claim (a subset of findings — most CR findings are about other things). The alternative — shipping a fix-in-pr commit that implements a wrong recommendation — costs a regression, a second CR pass, and a fix-the-fix cycle, with the added hazard that the regression carries the clean provenance "addressed CR finding #N" and is therefore *less* likely to be re-scrutinised.

## Generalises to

Any pipeline that ingests recommendations from a non-compiling source (automated reviewer, audit tool, LLM suggestion, a human reviewer reasoning from memory) and acts on them. The recommendation is a hypothesis; the authoritative oracle (here, the compiler; elsewhere, the test suite, the type-checker, the actual runtime) is the contract. Verify against the oracle before committing to the recommendation. Same family as `feedback_falsifiable_hypothesis_before_structural_fix.md` (falsify a structural-fix DQ's RCA before the fix), `feedback_audit_trait_derive_validate_field_types.md` (validate field types before promoting a derive finding), and `feedback_bm_false_success_advisor_post_condition_catch.md` (verify a tool's success claim against the actual post-condition) — in all of them, an authoritative-sounding claim is a lead, and the cheap verification against ground truth is the load-bearing safeguard.

## See also

- `.claude/rules/advisor-orchestrator.md` §5.4 (CR-finding variant of the falsifiable-hypothesis gate) — the rule that cites this file.
- `feedback_falsifiable_hypothesis_before_structural_fix.md` — the parent falsifiable-hypothesis discipline (structural-fix DQs).
- `feedback_audit_trait_derive_validate_field_types.md` — the sibling: validate field types before promoting an Eq/Hash/Ord derive finding.
- `feedback_pr_review_triage_pattern.md` + `feedback_coderabbit_triage_four_buckets_confirmed.md` — the four-bucket CR triage this gate plugs into (the compile-check decides `fix-in-pr` vs `rebut`).
- `feedback_coderabbit_block_merge_critical.md` — the separate rule that Critical findings always block merge (composes with this; this lesson governs how a finding is *verified*, that one governs Critical handling).
