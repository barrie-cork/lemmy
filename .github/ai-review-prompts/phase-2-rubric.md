# Phase 2 rubric — Read models (db_views)

Phase 2a (`governance_case`, `jury_queue`) is shipped. Phase 2b
(`governance_modlog`) is pending. Phase 5 adds `reputation` views. All share
the same review focus.

Focus areas beyond the ADR rubric:

- **No `derive(Selectable)` on governance view structs.** Phase 2a
  established that bare-scalar fields (stubs, counts, placeholders with no
  source-table column) break the Diesel `Selectable` derive with an
  unresolvable `schema::<struct_name>_rows` import error. The rule is in
  `.claude/rules/view-crate-selectable-template.md`. Use tuple-load + map
  instead — see `crates/db_views/governance_case/src/impls.rs` for the
  reference pattern (`GovernanceCaseDetailRow`, `GovernanceCaseSummaryView`).
  Flag any PR that adds `#[derive(Selectable)]` to a governance view struct.

- **Modlog views return REDACTED text only** per ADR-015. No `SELECT
  person.name`, `person.email`, `local_user.email`, `display_name` joined
  into a public log row. The handler that calls the view is responsible for
  rendering — the view returns pseudonyms and scrubbed rationale text.

- **Permission shaping belongs in the handler, not the view.** The view
  hydrates everything the handler could need; the handler decides which
  fields to return based on whether the caller is a juror, admin, target,
  or public user. Flag views that branch internally on permission.

- **Query function names match the plan.** `list_open_cases_for_community`,
  `read_case_detail`, `list_cases_for_person`, `list_cases_needing_jury_selection`,
  `list_jury_assignments_for_person`, `list_available_jury_cases_for_person`,
  `count_unsubmitted_jury_assignments`, and for Phase 2b
  `read_modlog_entry`, `list_modlog_for_community`, `list_modlog_for_person`.
  Plan tasks 14–30 in `IMPLEMENTATION-PLAN-v0.md` §3.

- **Real-DB smoke tests per query.** Each public query function has at least
  one test under `crates/db_views/<crate>/src/tests.rs` or `tests/e2e.rs`
  that seeds via direct Diesel inserts and asserts the query shape.
  No mocks.
