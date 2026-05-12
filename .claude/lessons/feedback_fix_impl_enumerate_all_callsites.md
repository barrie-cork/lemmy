---
name: fix-impl brief enumerate all callsites
description: When E0063/struct-shape errors point at K callsites, enumerate ALL N callsites via `rg` before authoring brief. Otherwise fix-impl-1 patches K, workflow FAILS again on N-K.
type: feedback
---

# fix-impl brief: enumerate all callsites before authoring

When the §G4 classifier dispatches a fix-impl-task in response to a struct-shape failure (E0063 missing-field on `<Type>` initializer; renamed/added/removed field; trait-impl signature change), the compile error log slice cites K specific `file:line` sites. The advisor's brief MUST run `rg "<Type>" crates/ tests/` BEFORE authoring the brief to enumerate the FULL N callsites — not just the K cited by the compiler.

**Why:** Per retro evidence at `.claude/PRPs/reports/v1-RT-r1-halt-retro.md` (commit `ffa2876e3`). v1-RT-r1 Cohort B-bundled (Tasks 6+7) added two new fields to `ReputationEventInsertForm`. The compile error log slice cited 2 sites: `sponsor_liability.rs:296` + `submit_jury_vote.rs:935`. fix-impl-1 brief covered only those 2 sites (well within the §G4 "≤3 file edits" cap). Junior #226 padded the 2 literals, pushed, workflow `25698028473` FAILED with the **same E0063 errors on 8 more callsites** — Junior had run `rg "ReputationEventInsertForm" crates/` itself and surfaced this via DQ #205 blocker (`crates/api/api_crud/src/governance/create_endorsement.rs:278+291`, `crates/tools/seed_founders/src/main.rs:173`, `crates/server/tests/e2e.rs:2902+2925+3935+5345+7986` — 8 sites in 3 additional files). The fix-impl-1 brief should have done that enumeration UPFRONT. Cost: ~10 min Junior cycle + DQ bookkeeping for what could have been a single bundled fix.

**Mechanical root cause:** the compiler's E0063 error only flags the call site that hit the type-check failure FIRST during compilation. Later call sites in different crates / modules are silenced by the cascade-stop. So `cargo check`'s log slice is a LOWER BOUND on affected callsites, not the full set. `rg "<Type>"` is the upper bound.

**How to apply:** When authoring a fix-impl-task brief in response to a struct-shape failure signature:

```bash
# (1) Run the enumeration grep BEFORE authoring the brief.
rg "<Type>" crates/ tests/  # the type name from the E0063 message
# - Capture ALL file:line hits
# - Include test files; some callsites may be in tests/, not just crates/api/

# (2) Sanity check the hit list against the failure-signature semantics:
#     - struct-literal initializer? Each `<Type> { ... }` block needs the new field added.
#     - function call with positional arg? Each call site needs the new arg added.
#     - trait-impl signature change? Each `impl <Trait> for <Type>` needs the matching method update.

# (3) The brief's edit-cap = count of distinct files containing those hits.
#     The ≤3-file-edits default from §G4 is for clippy/unused-import patterns, NOT struct-shape.

# (4) Author the brief listing ALL hits with explicit file:line. Junior's grep at task start
#     should match the brief's enumeration exactly.
```

**When N > 10 callsites or N > 5 files:** the change is no longer "narrow mechanical." Catch-fire to user with the full enumeration list. User decides whether to (a) extend the brief to cover all N sites in one Junior task, (b) split across multiple commits per crate / per file group, or (c) re-plan the struct shape itself.

**Edge cases:**

- Some callsites may be in test fixtures that are correctly using a stub/default constructor pattern. If `rg` matches turn out to be `<Type>::default()` or `<Type> { ..Default::default() }`, those callsites compile cleanly and don't need padding — verify before listing in the brief.
- Macro-expanded callsites won't show up in plain `rg "<Type>"`. If `<Type>` is constructed via a macro (e.g. `make_form!(...)`), search for the macro instead.
- Generated code (`schema.rs`, `*.pb.rs`) may reference the type but rebuild on every cargo run; these are usually safe to ignore in the brief.

**Detection (retro):** a `chore(decision-queue): impl raised DQ #<N> (blocker) — N more callsites missing fields` commit on a fix-impl worker branch is evidence that the brief under-enumerated. Retro flags it. The cost is a wasted fix-impl-1 cycle; the fix is brief-authoring discipline.

**Companion lessons:**
- `feedback_principles_not_rules.md` — when to extend rules vs. accept the cost.
- `feedback_lemmy_error_no_std_error.md` — Case-A/B/C enumeration is a similar discipline for trait-bound errors.

**Where codified:** `.claude/rules/advisor-orchestrator.md` §5.3 §G4 classifier "Callsite-enumeration discipline" paragraph.
