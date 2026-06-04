# Phase 7 — Type-state retrofit RECON (read-only, 2026-06-04)

> Done while waiting on the Phase 6 golden-baseline e2e. Read-only; no files touched.
> Canonical pattern: `.claude/lessons/feedback_governance_type_state_handlers.md`.
> Plan section: `v1-closeout.plan.md` §"Phase 7". Branch blob baseline: post-M1 merge `060bcac15`.

## TL;DR

6 `TODO(type-state)` sites confirmed on the branch, each a distinct guard shape. The phantom
`GovernanceCase<S>` scaffold does **NOT** exist yet (the existing `GovernanceCase*` types in
`crates/db_views/governance_case/src/lib.rs` are unrelated **read-model view structs** — name-adjacent,
not the wrapper). Task 0 creates the scaffold; the 6 retrofits follow. **All behaviour-preserving:
keep returning `LemmyErrorType::NotFound` (NOT a new `InvalidCaseState`) so callers can't infer
internal state — exactly today's behaviour.**

## Scaffold placement decision (RECOMMENDED)

Put the scaffold in **`crates/api/api/src/governance/state.rs`** (new file, sibling to the existing
`jury_common.rs` shared-helper module), declared via `pub mod state;` in
`crates/api/api/src/governance/mod.rs`. Rationale:

- The 6 consuming handlers all live in `crates/api/api/src/governance/` and **already import
  `ModerationCase`** from `lemmy_db_schema::source::governance::moderation_case` — zero new cross-crate
  deps.
- The lesson suggests `crates/api_common/src/governance/state.rs`, BUT api_common's governance is a
  **flat file** (`api_common/src/governance.rs`, 844 lines) that imports `CaseStatus`/`ModerationCaseId`
  but **NOT** the full `ModerationCase` struct, and is **not** `#[cfg(feature="full")]`-gated. Putting
  the wrapper there forces (a) converting the flat file to a dir module, and (b) adding a full-gated
  `ModerationCase` import to api_common. Higher churn, more conflict surface. The api-crate placement
  is lower-churn and conventional (mirrors `jury_common.rs`).
- **`#[cfg(feature = "full")]` is MANDATORY on the scaffold** — `ModerationCase` lives in a module
  compiled only under `full` (confirmed via the view-crate doc comment). The api crate's `governance`
  handlers are already full-context, so this is consistent.

## CaseStatus ground truth (12 variants — lesson says 13, STALE)

`crates/db_schema_file/src/enums.rs:393` — `pub enum CaseStatus` has **exactly 12** variants:
`Open, ThresholdMet, JurySelection, InReview, Decided, Appealed, Closed, EmergencyRemove, AdminReview,
SponsorLiabilityPending, SponsorLiabilityFired, SponsorLiabilityEscaped`.
(The lesson's "13 variants (v1 will add more)" predates the settled count — **stale-metric flag** per
`feedback_stale_metric_lesson_guard.md`; update the lesson at Phase 7 close. `Founder/Regular/Probation`
belong to the SEPARATE `CaseStatusTier` enum, not CaseStatus.)

Every retrofit's `TryFrom` allow/deny match must enumerate all 12 exhaustively (no `_ =>`) — the sites
already do this, so the scaffold's `TryFrom` impls inherit the same exhaustive lists verbatim.

## The 6 sites (each → its state marker + TryFrom shape)

| # | Site | Guard shape today | Type-state model | Accepts (allow-list) |
|---|---|---|---|---|
| 1 | `accept_jury_assignment.rs:110` | **dual-role** nested match: dispatch on `JuryAssignmentRole` FIRST, then per-role status match | `GovernanceCase<JurySelection>` for Original arm (accepts JurySelection\|InReview), `GovernanceCase<Appealed>` for Appeal arm. Dispatch on role first (existing pattern), then `try_from` per arm. | Original: `JurySelection, InReview`; Appeal: `Appealed` |
| 2 | `admin_assign_jury.rs:122` | single match, 3-variant allow | `GovernanceCase<PreJuryAssignable>` | `Open, ThresholdMet, EmergencyRemove` |
| 3 | `admin_close_case.rs:65` | **inverted** guard (reject `Closed` only, accept other 11) | `GovernanceCase<NotYetClosed>` via exhaustive allow-list TryFrom covering 11 variants | all EXCEPT `Closed` |
| 4 | `admin_trigger_appeal_rejury.rs:68` | single-variant allow (**cleanest candidate**) | `GovernanceCase<Appealed>` | `Appealed` only |
| 5 | `sponsor_liability_grace.rs:135` | **filter-query** pattern (`.filter(status.eq(SponsorLiabilityPending))`), NOT an exhaustive match — runs in a batch for-loop | harden to exhaustive match first, then wrap per-case `GovernanceCase<SponsorLiabilityPending>` INSIDE the for-loop (the batch filter stays; the per-case revalidation gets the wrapper) | `SponsorLiabilityPending` |
| 6 | `submit_jury_vote.rs:271` | **terminal-state idempotency** guard via `matches!(... terminal-list ...)` → early-return success (NOT an error) | `GovernanceCase<Active>` where `Active` excludes terminal variants via a `CanReceiveVote` sealed trait. **NB: this site returns `Ok(SubmitJuryVoteResponse{case_decided:true})` on the terminal branch — NOT an error.** The type-state model must preserve the early-return-success semantics, not convert it to a `TryFrom` error. The trickiest of the 6. | non-terminal: `Open, ThresholdMet, JurySelection, InReview` (terminal short-circuit: `Decided, Closed, Appealed, EmergencyRemove, AdminReview, SponsorLiability{Pending,Fired,Escaped}`) |

## State markers needed (zero-sized structs in state.rs)

`JurySelection`, `Appealed`, `PreJuryAssignable`, `NotYetClosed`, `SponsorLiabilityPending`, `Active`
(6 markers; `Appealed` shared by sites 1-Appeal-arm + 4). Plus the `CanReceiveVote` sealed trait for
site 6's `Active`.

## Behaviour-preservation invariants (CRITICAL)

1. **Keep `LemmyErrorType::NotFound`** as the `TryFrom::Error` mapping for sites 1–5 (today's behaviour;
   `InvalidCaseState` does NOT exist in `LemmyErrorType` and adding it would CHANGE the HTTP status +
   leak state). Do NOT add `InvalidCaseState`.
2. **Site 6 is NOT a TryFrom-error site** — it early-returns `Ok(...case_decided:true...)`. Model it so
   the terminal branch still returns that exact success response. A naive `TryFrom<…> -> Err` would flip
   a 200-success into a 404 — a behaviour regression. Consider: `GovernanceCase::<Active>::try_from`
   returning a sentinel the caller maps to the success early-return, OR keep site 6 as a `matches!`
   guard and only retrofit sites 1–5 (surface this as a Phase 7 sub-decision).
3. **Exhaustive matches stay exhaustive** (no `_ =>`) per ADR-013 — the scaffold's TryFrom impls carry
   the same 12-variant enumeration the sites have today.
4. Each retrofit must keep the surrounding logic (config cache, exclusion-ID reads, idempotency checks
   that follow the guard) byte-identical — only the guard block changes.

## Execution shape (four-role OR direct; Sonnet per the model decision)

- **Task 0 (non-`[P]`):** create `crates/api/api/src/governance/state.rs` with the 6 markers + the
  `CanReceiveVote` sealed trait + `GovernanceCase<S>` struct + one `TryFrom<ModerationCase>` impl per
  marker (returning `NotFound`). Add `pub mod state;` to `governance/mod.rs`. `cargo check -p lemmy_api
  --features full` must pass. The 6 retrofits then `[P]`-parallel-eligible BUT all touch sibling files
  under one dir (low collision); they share only the `state.rs` definition from Task 0.
- **Tasks 1–6:** one site each. Each: replace the guard block with `let case = GovernanceCase::<Marker>
  ::try_from(case)?;` (sites 1–5) and use `case.inner` downstream. Site 6 needs the success-preserving
  treatment (see invariant 2).
- **Validation:** `cargo check -p lemmy_api --features full` per task; full e2e at the end (the 6 sites
  have e2e coverage: `report_to_modlog_golden_path`, `submit_jury_vote_*`, `grace_check_*`,
  `appeal_*`, `admin_assign_jury_*`). Cargo laptop-local.

## Open sub-decisions to surface at Phase 7 start

1. **Site 6 modelling** — retrofit it (with success-preserving sentinel) vs leave it as `matches!` and
   retrofit only 5. (Recommend: surface to user; site 6's success-not-error semantics make it the one
   genuine judgement call.)
2. **`[P]` parallelism** — the 6 retrofits could run as a cohort, but cargo is laptop-only + serial; if
   driven by this session directly, do them sequentially (5 min each) rather than dispatching a cohort.
3. **MIRROR ref** — there is no existing in-tree `GovernanceCase<S>` to mirror; the lesson's code block
   IS the canonical pattern. Cite the lesson verbatim in the Task 0 brief.

## Status

RECON COMPLETE. Phase 7 is execution-ready pending Phase 6 completion (Phase 7 shares the
`governance/` dir conceptually but touches DIFFERENT files than the e2e split, so it could even run
before Phase 6 finishes — but plan order is 6→7→8). Nothing edited.
