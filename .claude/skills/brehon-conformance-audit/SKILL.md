---
name: brehon-conformance-audit
description: >
  Read-only audit catching the Phase-6 convention-divergence defect class: new federation
  handlers/helpers diverging from same-file canonical siblings across six fixed axes
  (conn-type, append reborrow, trait-bound, error idiom, conn acquisition, ADR-015 pseudonym).
  Invoked at brief-author time (prevention) and retro time (detection) via advisor-orchestrator
  stage-shape; emits a per-run audit report and a per-run metrics file.
allowed-tools: [LSP, Read, Grep, Glob, Bash]
user-invocable: true
---

# Brehon conformance audit

Phase-6 federation work (`v1-federation-inbound-a` + `-b`) produced a defect class where new
code in a Phase-6-bearing file diverges from same-file canonical siblings across six well-defined
axes. Three compile-caught divergences surfaced at fix-impl-1 (axes 1, 1, 3). One latent
footgun — `receive_remote_moderation_label` used `.unwrap_or_default()` where four lines away a
sibling hard-errors via `.domain().ok_or_else(|| LemmyErrorType::Unknown(...))?` — compiled
cleanly and was caught only by explicit advisor review. Existing tooling (`security-auditor`,
`code-audit`, `code-reviewer`) has no notion of diffing new handlers against in-repo siblings.
This skill provides that check: read-only, invokes `LSP, Read, Grep, Glob, Bash` only, and emits
a per-run audit report plus a per-run metrics file.

## When to run

- **Brief-author time (prevention):** when a brief targets a federation module root
  (`crates/apub/activities/src/governance/`, `crates/api/api/src/governance/`,
  `crates/db_schema/src/source/governance/`). Invoked by the advisor session’s §3.1
  stage-shape orchestration before queuing the impl-task.
- **Retro time (detection):** on `phase-diff <branch>` after impl-tasks complete. Confirms no
  latent footgun landed between authoring and merge. Invoked by the §3.9 verify gate.
- **Ad-hoc:** on `file <path>` or `fn-list <file:fn>,...` for spot-checks during implementation.

## Inputs

Three input modes:

- `phase-diff <branch>` — audit every changed file on `<branch>` since merge-base with
  `governance-v0`. Examines all new or modified functions against in-file siblings.
- `file <path>` — audit one file’s new symbols against in-file siblings.
- `fn-list <file:fn>,...` — audit one or more specific functions by name.

## Outputs

Exactly two output paths. The skill writes ONLY to these paths — no other `Edit`/`Write` calls.

- `.claude/PRPs/reports/conformance-audit-<scope-slug>-<YYYY-MM-DD>.md` — human-readable audit
  report: per-finding risk tier, sibling proof, evidence string, suggested action.
- `.claude/PRPs/audit-metrics/<scope-slug>.json` — machine-readable predictions + ground-truth
  journal (schema: `audit-metrics.schema.json`). Mutated post-§15 to record
  `compile_caught[]` / `runtime[]` ground truth for precision/recall computation.

## Six axes

The six axes are defined in sub-files under `axes/`. Each sub-file carries the detection
command(s), Trace-Up invariant citations, and an Error-code → design-question table (axes #1,
#3, #4). These files are authored in Task 2 (Cohort 2).

1. [`axes/1-conn-type.md`](axes/1-conn-type.md) — Conn-type / tx-boundary
2. [`axes/2-append-reborrow.md`](axes/2-append-reborrow.md) — Append reborrow shape
3. [`axes/3-trait-bound.md`](axes/3-trait-bound.md) — Trait-bound completeness
4. [`axes/4-error-idiom.md`](axes/4-error-idiom.md) — Error idiom at trust boundary
5. [`axes/5-conn-acquisition.md`](axes/5-conn-acquisition.md) — Conn acquisition idiom
6. [`axes/6-adr-015.md`](axes/6-adr-015.md) — ADR-015 pseudonym handling for remote actors

## Invocation discipline

- **Read-only:** uses `LSP, Read, Grep, Glob, Bash` only. No `Edit`/`Write` outside the two
  declared output paths (see `## Outputs`). No `Agent` dispatch.
- **No `cargo` invocation:** the skill reads history and diffs siblings; the compiler is the
  oracle, not a callee. No `cargo check`, `cargo test`, `cargo clippy`, or any other cargo call
  from within the skill body.
- **Hypotheses, not verdicts:** every flag raised is a hypothesis until (a) the compiler proves
  it via a §15 `cargo check` run outside this skill, or (b) a sibling diff demonstrates
  that new code is strictly weaker than an enforced sibling contract. Per
  `feedback_verify_automated_reviewer_claims_against_compiler.md`.
- **ADR-006 advisory-only preserved:** suggested actions must never convert an advisory inbound
  signal into an auto-apply. The skill surfaces findings; it does not enforce them.

## Risk tiers

- **Tier 1 — enforced-contract weakening with sibling proof.** New code uses a weaker pattern
  than a same-file sibling that already enforces the stronger contract. Action: fold into the
  brief scope (prevention) or raise as a catch-fire finding at retro (detection).
- **Tier 2 — sibling divergence without proof of weakening.** New code diverges from a
  sibling’s style but no contract enforcement is demonstrably weaker. Action: log to
  retro §3 “watch items” for the next sub-phase plan.
- **Tier 3 — stylistic.** Divergence is cosmetic (naming, comment style). Action: log to the
  audit report only; no escalation.
