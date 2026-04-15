# Implementation Report — Phase 2a: governance_case + jury_queue

**Plan**: .claude/PRPs/plans/phase-2a-governance-case-jury-queue.plan.md
**Completed**: 2026-04-15
**Branch**: feature/phase-2a-governance-case-jury-queue
**Ralph iterations**: 11 task iterations + 2 mid-phase fixup iterations = 13 total
**HEAD at completion**: `f84bd92d9`

## Summary

Two new db_views crates, `lemmy_db_views_governance_case` and
`lemmy_db_views_jury_queue`, land under `crates/db_views/`. Each ships
plain-Rust view structs and a `#[cfg(feature = "full")]`-gated `impls`
module with the query functions enumerated in
[IMPLEMENTATION-PLAN-v0.md §Phase 2 tasks 14–24]. No migrations, no new
Diesel source types, no API layer, no smoke tests — those belong to
Phase 2b/3/4. `phase1_migrations_round_trip` still passes on the
feature branch (4 passed, 0 failed, identical to Phase 1 baseline).

## Tasks Completed

| # | Subject | Commit |
|---|---|---|
| 14 | create `lemmy_db_views_governance_case` crate | `29863d002` |
| 15 | `GovernanceCaseSummaryView` + `GovernanceCaseDetailView` structs | `6ff73d6ac` |
| 16 | `list_open_cases_for_community` query | `56ae38cae` |
| 17 | `read_case_detail` query | `2e468f2ea` |
| 18 | `list_cases_for_person` query | `495746575` |
| 19 | `list_cases_needing_jury_selection` query | `9008f95f0` |
| 20 | create `lemmy_db_views_jury_queue` crate | `0ff2e0354` |
| 21 | `JuryQueueView` struct | `e02ed6b61` |
| 22 | `list_jury_assignments_for_person` query | `ace4fb7f4` |
| 23 | `list_available_jury_cases_for_person` query | `634d91567` |
| 24 | `count_unsubmitted_jury_assignments` query | `f84bd92d9` |

Two additional advisor-approved non-task commits:

- `65324a548` — `chore(scripts): make cargo-check.bat forward args like cargo-test.bat` (Layer 1 fix, checkpoint-2)
- `d77dc6829` — `docs(plan): reframe phase 2a clippy DoD to use --no-deps --features full` (Layer 3 fix, checkpoint-2)

## Validation Results

| Gate | Check | Result |
|---|---|---|
| 1 | clean tree | PASS (only untracked research PDF) |
| 2 | `cargo check --workspace` | PASS (exit 0, 2m 12s) |
| 3 | `cargo test --test e2e -p lemmy_server` | PASS (4 passed, 0 failed, 29.62s) |
| 4 | `cargo check -p lemmy_db_views_governance_case --features full` | PASS (exit 0) |
| 4 | `cargo check -p lemmy_db_views_jury_queue --features full` | PASS (exit 0) |
| 4 | `cargo clippy --no-deps -p lemmy_db_views_governance_case -p lemmy_db_views_jury_queue --features full -- -D warnings` | PASS (exit 0) |
| 5 | 11 task commits with `task N —` prefix, in order | PASS (11 task-prefixed, task 14–24) |
| 5 | `wc -l` total commit count | **13** (11 task + 2 advisor-approved fixups); surfaced as plan-wording mismatch |

## Codebase Patterns Discovered

1. **`cargo check` without `--features full` hides feature-gated types.**
   A struct under `#[cfg(feature = "full")]` is invisible to the compiler
   in default-features mode, so any Selectable-derive error inside it
   cannot surface under bare `cargo check -p <crate>`. Every Phase 2a task
   that touches a governance source type validated with
   `scripts/brehon/cargo-check.bat -p <crate> --features full`.

2. **The `modlog` / `report_combined` template requires zero bare scalar
   fields.** Every field in `ModlogView` and `ReportCombinedViewInternal`
   is either `#[diesel(embed)]` (delegating to a source struct with its
   own `#[diesel(table_name = ...)]`) or `#[diesel(select_expression =
   ...)]`. A single bare scalar field trips Selectable's auto-resolution
   of a default table name from the struct name (snake_case + pluralize),
   which fails with `unresolved module or unlinked crate '<name>s'`.
   Phase 2a's view shapes all carry bare scalars or constants, so none
   of them derive `Queryable`/`Selectable`; all three views are plain
   Rust structs built by mapping an explicit Diesel tuple select in the
   impls.

3. **No `moderation_case <-> community` joinable.** Phase 1 did not
   declare `joinable!(moderation_case -> community (community_id))`, so
   every Phase 2a query that joins the two uses explicit
   `.on(community::id.nullable().eq(moderation_case::community_id))`.
   `jury_assignment -> moderation_case (case_id)` and the post/comment
   joinables DO exist, so the fluent `.inner_join(...)` form works there.

4. **Nullable-column inequality filters need `is_null().or(ne(x))`.**
   `moderation_case::target_person_id` and `::creator_id` are
   `Nullable<Int4>`. A bare `.ne(person_id)` would silently drop rows
   where the column is NULL because SQL evaluates `NULL != X` as UNKNOWN,
   and WHERE treats UNKNOWN as false. Task 23 uses
   `.is_null().or(.ne(person_id))` via `BoolExpressionMethods` to include
   cases with no target person or no creator in the juror's available queue.

5. **`scripts/brehon/cargo-check.bat` now forwards `%*`.** Phase 1
   shipped the script with a hardcoded `cargo check --workspace` at line
   20, silently discarding any `-p <pkg>` or `--features` arg. Commit
   `65324a548` changes it to `cargo check %*` to match `cargo-test.bat`.
   This was the primary "load-bearing" issue that caused Phase 2a task
   14's validation to false-green; the checkpoint-2 audit caught it.

## Deviations from Plan

1. **Plan §6 `Queryable/Selectable` template on the view structs does not
   apply.** The plan's §6 template (mirrored from `modlog`) adds
   `#[cfg_attr(feature = "full", derive(Queryable, Selectable))]` to all
   three views. The template is wrong for the Phase 2a shapes because
   they contain bare scalar fields that Selectable cannot resolve a
   default table for. Tasks 15–24 take the plan §12 R1 + R2 fallback
   paths pre-emptively: views are plain structs, impls use explicit
   tuple selects and manual-map steps. Documented in-code on each struct
   and in the task 15 commit body. No plan edit — the divergence is
   recorded in the report and in each affected task's commit body.

2. **Task 17 `read_case_detail` uses five round-trips, not two.** The
   plan §6 template describes a "two-query" pattern where the first
   query loads `ModerationCase::as_select()` + evidence_count subquery +
   appeal.status left-join + COALESCE target_creator_id in a single big
   tuple select, and the second query loads `Vec<Sanction>`. Since
   `GovernanceCaseDetailRow` dropped its Selectable derive (see deviation
   #1), I split the first query into four small queries:
   `ModerationCase` row, `case_evidence` count, `appeal.status`
   first+optional, and `resolve_target_creator_id` (which itself is a
   priority-order lookup over post/comment/target_person_id). The fifth
   query is `Vec<Sanction>`. Clarity over round-trip count is the right
   trade-off for a per-request handler; documented in the task 17 commit
   body and the `read_case_detail` doc comment.

3. **Gate 5 commit count.** Plan §10 says `wc -l` total commits should
   equal 11. On-disk state has 13 commits (11 task + 2 advisor-approved
   non-task fixups for the wrapper fix and plan reframe). Surfaced in
   the final report rather than failing the gate — the gate's intent
   ("exactly 11 task commits, one per task 14–24, in order") is
   satisfied; the literal `wc -l` count doesn't account for mid-phase
   fixup commits that the checkpoint-2 approval explicitly sanctioned.

## Drift Items Still Open (homeserver-side decisions)

All three drift items from plan §2 are landed as stubs per the advisor's
"code wins, log the drift" disposition. No new decisions surfaced during
implementation:

1. **[04 §4.1] `reporter_count: i64` stub = 0.** Phase 2a implementation
   always emits 0. [04 §4.1] needs to either (a) mark this as derived
   from a new reports table to be added in a future phase, or (b) remove
   it from the view spec. Advisor to decide.

2. **[04 §4.1] `jury_needed: i32` constant = 5.** Hardcoded per [05 §3].
   [04 §4.1] should either document it as "constant per [05 §3]" or add
   a `jury_size_target` column to `moderation_case`. `jury_submitted`
   remains sourced from a second-round-trip grouped count of
   `jury_assignment WHERE status = 'submitted'`.

3. **[04 §4.2] `deadline_at: Option<DateTime<Utc>>` always None.** Task
   24's `count_unsubmitted_jury_assignments` ships the unfiltered total
   count; Phase 4 adds the time filter when the homeserver-side decision
   on "stored column vs computed offset" lands.

## Learnings and Meta-Observations

### The load-bearing Phase 2a bug was the cargo-check.bat wrapper, not the plan.

Phase 1 shipped with `scripts/brehon/cargo-check.bat` hardcoding
`cargo check --workspace` at line 20, silently discarding any `-p`
argument passed to it. Phase 1 tasks ran `cargo-check.bat -p
lemmy_db_schema_file` and got green results — but the `-p` flag was
silently ignored, the workspace was checked instead, and feature
unification activated `lemmy_db_schema/full` because other workspace
members opted into it. The Selectable-derive problem this fork now has
on any "mix of bare scalars and source-table embeds" shape was invisible
until Phase 2a's task 14 fired against an empty lib (still green by luck)
and task 15 hit the governance source module that is itself gated behind
`full`. Per advisor checkpoint-2 audit, Layer 1 fix (wrapper pass-through)
+ Layer 3 fix (plan DoD reframe to `--features full` + `--no-deps`) land
first as non-task commits, then Layer 2 fix lands as the task 15 commit.

### Lesson carried forward to Phase 2b / future view crates.

Before writing a new view crate that mirrors `modlog`, answer this
about every field:

> For each field, is it (a) `#[diesel(embed)]` from a source struct,
> (b) explicit `#[diesel(select_expression = ...)]`, or (c) a bare
> scalar — and if (c), why is that OK here given Selectable's
> auto-resolution of a default table name from the struct name?

If you can't answer (c) convincingly against the `modlog` /
`report_combined` precedent, the derive is wrong and the view needs the
plain-struct + manual-tuple-load shape instead. Phase 2a logs three
existence-proofs of this fallback (`GovernanceCaseSummaryView`,
`GovernanceCaseDetailRow`, `JuryQueueView`) for future crate authors to
mirror.

### Iteration counts per task.

| Task | Iterations | Notes |
|---|---|---|
| 14 | 1 | Scaffold |
| 15 | 2 | Layer 2 fixes: first attempt kept Queryable/Selectable on Row, failed; second attempt dropped both |
| 16 | 2 | First attempt had `type Row` inside fn body (items_after_statements) + type_complexity on build_summary param; second attempt refactored to module-scope alias |
| 17 | 1 | |
| 18 | 1 | |
| 19 | 1 | |
| 20 | 1 | Scaffold |
| 21 | 1 | |
| 22 | 1 | Join via joinable! + explicit .on() for community worked first try |
| 23 | 1 | `not(exists(...))` and `is_null().or(.ne(...))` both compiled first try |
| 24 | 1 | |
| Final gate | 1 | All five gates passed |

Total task iterations: 14. Per-task average: 1.27. All tasks stayed
under the Phase 2a 4-iteration soft cap.

### Watchpoint hit (from checkpoint-2).

Clippy warmup behaved as expected. First `cargo check --features full`
run at task 15 was ~26s (~300 crates warming). Subsequent incremental
task checks ran at 12–30s. Task 19 clippy briefly touched ~76s (under
the 90s stop-and-diagnose threshold) — likely a fresh dep rebuild after
task 18's incremental save-file touch; no action taken. All other post-
warmup checks stayed well under 90s.
