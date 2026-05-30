---
phase: v1-RT-r4
role: impl-task
task: 1
brief_n: 1
authored: 2026-05-29
parallel_cohort: A
---

# [role:impl-task] v1-RT-r4 task 1 sponsor_allowlist db-helpers — see .claude/PRPs/briefs/rt-r4-impl-1.md

## §1 Role + dispatch

`[role:impl-task] v1-RT-r4 task 1 sponsor_allowlist inline db-helpers (insert/delete/exists)`

## §2 Scope

MODIFY `crates/db_schema/src/source/governance/sponsor_allowlist.rs` ONLY.

Add three inline `#[cfg(feature = "full")] pub async fn` helpers at the end of the file,
mirroring the inline-helper convention in `federation_peer.rs:55` / `:70`.

**IMPLEMENT:**

1. `pub async fn sponsor_allowlist_insert(form: &SponsorAllowlistInsertForm, conn: &mut AsyncPgConnection) -> LemmyResult<SponsorAllowlist>`
   — `diesel::insert_into(sponsor_allowlist::table).values(form).returning(SponsorAllowlist::as_returning()).get_result(conn).await?`

2. `pub async fn sponsor_allowlist_delete(allowlist_id: SponsorAllowlistId, conn: &mut AsyncPgConnection) -> LemmyResult<usize>`
   — `diesel::delete(sponsor_allowlist::table.find(allowlist_id)).execute(conn).await?`

3. `pub async fn sponsor_allowlist_exists(person_id: PersonId, community_id: Option<CommunityId>, conn: &mut AsyncPgConnection) -> LemmyResult<bool>`
   — filter `person_id.eq(person_id)` AND either `community_id.eq(c)` (when `Some(c)`) OR `community_id.is_null()` (when `None`). Returns `.first::<SponsorAllowlist>(conn).await.optional()?.is_some()`. Build the community predicate with a branch on `community_id` (diesel `BoxedQuery` or two-arm `match`) — NOT a single `.eq(community_id)` (that matches NULL-vs-NULL incorrectly).

**Scope boundary:** Do NOT edit any other file. No migration. No schema.rs. No impls/ directory.
These helpers live inline in `source/governance/sponsor_allowlist.rs` per governance crate convention.

## §3 Required reading

- `.claude/PRPs/plans/v1-RT-r4.plan.md` §13 Task 1 (FILES YAML, GOTCHA, MIRROR ref, VALIDATE)
- `.claude/PRPs/plans/v1-RT-r4.plan.md` §10 "Patterns to mirror" (inline db-helper shape)
- `crates/db_schema/src/source/governance/federation_peer.rs` lines 55–94 — the canonical inline-helper convention to mirror (`.optional()?` on read, `.get_result(conn)` on insert)
- `crates/db_schema/src/source/governance/sponsor_allowlist.rs` — read the full file first; understand the existing struct layout before editing
- `.claude/lessons/feedback_newtype_locations_lemmy_db_schema_vs_file.md` — `SponsorAllowlistId`, `PersonId`, `CommunityId` imports must come from their correct newtype homes
- `.claude/lessons/feedback_features_full_workspace_only.md` — `#[cfg(feature = "full")]` gate semantics
- `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — per-crate `--features full` is valid for `lemmy_db_schema` (it defines `full`); but workspace clippy uses `--workspace`
- `.claude/rules/decision-queue.md` — Recipe 2 (`kind: "log"`) for durable findings; Recipe 1 (`kind: "blocker"`) only if genuinely blocked

## §3a Handover from prior cohort

(none — first cohort)

Task 0 pre-phase harness audit was run by the advisor (not a Junior task) and passed all 4
probes (EXIT_0 on probes 1-3, EXIT_NONZERO on probe 4 — correct). Impl gate cleared.
Flag: `.claude/audit-phase-v1-RT-r4-complete.flag`.

Phase branch tip at `phase-v1-RT-r4` = `5afe86554` (MiniMax trial designation commit).
DQ pending: 0.

## §4 Constraints

- **`[P]` parallel:** This task is cohort-parallel with Task 2 (which modifies `api_common/governance.rs`). Files are disjoint — no overlap.
- **Inline convention:** helpers go in `source/governance/sponsor_allowlist.rs`, NOT in a new `impls/` file. The governance crate convention is inline helpers in the source file.
- **`community_id IS NULL` arm:** the `sponsor_allowlist_exists` community predicate MUST be a two-arm branch (Some(c) → `.eq(c)`, None → `.is_null()`). A single `.eq(community_id)` passes `None` as SQL NULL which fails to match NULL rows via `=`.
- **No cargo runs.** Junior workers on the daemon do not invoke cargo (Shape G suspended — cargo runs via advisor laptop validate gate). After commit + push, write a `kind: "validate-pending-laptop"` DQ entry with `commands: ["cmd //c \"scripts\\\\brehon\\\\cargo-check.bat -p lemmy_db_schema --features full > .claude/validate-t1.log 2>&1\"", "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat -p lemmy_db_schema --features full --no-deps -- -D warnings > .claude/validate-t1-clippy.log 2>&1\""]`, `branch: <worker-branch>`, `phase_task: 1`. Commit + push the DQ entry immediately after writing it.
- **Attribution:** DQ entries use `from: "impl"`, `answered_by: null`. Never `from: "advisor"`.
- **Commit subject:** `feat(rt-r4): add sponsor_allowlist inline db-helpers (task 1)`.
- **HANDOVER YAML trailer** in commit body: `filesCreated: []`, `filesModified: [crates/db_schema/src/source/governance/sponsor_allowlist.rs]`, `keyDecisions: [community_id branch on is_null vs eq]`, `notes: <verbatim observation if any>`.

## §5 Validation (laptop, after advisor reads validate-pending DQ)

Laptop advisor runs:
1. `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/validate-t1.log 2>&1"` — exit 0 expected
2. `cmd //c "scripts\\brehon\\cargo-clippy.bat -p lemmy_db_schema --features full --no-deps -- -D warnings > .claude/validate-t1-clippy.log 2>&1"` — exit 0 expected

(Junior does not invoke these — validation is advisor-run via validate-pending-laptop DQ flow.)

## §6 Commit message

`feat(rt-r4): add sponsor_allowlist inline db-helpers (task 1)`

Body: HANDOVER YAML trailer as described in §4.
