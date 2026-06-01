---
phase: v1-RT-r4
role: impl-task
task: 2
brief_n: 1
authored: 2026-05-29
parallel_cohort: A
---

# [role:impl-task] v1-RT-r4 task 2 admin sponsor-allowlist DTOs — see .claude/PRPs/briefs/rt-r4-impl-2.md

## §1 Role + dispatch

`[role:impl-task] v1-RT-r4 task 2 AddSponsorAllowlist + Remove DTOs in api_common/governance.rs`

## §2 Scope

MODIFY `crates/api/api_common/src/governance.rs` ONLY.

Add four DTOs mirroring the `AdminSetConfig`/`AdminSetConfigResponse` derive stack at `:444`/`:465`.

**IMPLEMENT:**

Add after the existing admin DTO block (after the last `*Response` struct before any non-DTO content):

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AddSponsorAllowlist {
  pub person_id: PersonId,
  pub community_id: Option<CommunityId>,
  pub note: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AddSponsorAllowlistResponse {
  pub allowlist_id: SponsorAllowlistId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct RemoveSponsorAllowlist {
  pub person_id: PersonId,
  pub community_id: Option<CommunityId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct RemoveSponsorAllowlistResponse {
  pub success: bool,
}
```

The derive stack MUST be verbatim — do NOT add or drop derives vs the `:444`/`:465` pattern.

**Scope boundary:** Do NOT edit any other file. No handler, no route, no migration.

## §3 Required reading

- `.claude/PRPs/plans/v1-RT-r4.plan.md` §13 Task 2 (FILES YAML, GOTCHA, MIRROR ref, VALIDATE)
- `.claude/PRPs/plans/v1-RT-r4.plan.md` §10 "Patterns to mirror" (DTO derive stack)
- `crates/api/api_common/src/governance.rs` lines 444–475 (the `AdminSetConfig` / `AdminSetConfigResponse` derive stack to mirror verbatim)
- `.claude/lessons/feedback_newtype_locations_lemmy_db_schema_vs_file.md` — `PersonId`, `CommunityId`, `SponsorAllowlistId` must come from their correct newtype homes
- `.claude/lessons/feedback_features_full_workspace_only.md` — `#[cfg(feature = "full")]` gate semantics (the file is behind the full feature)
- `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — per-crate `--features full` is valid for `lemmy_api_common`; workspace clippy uses `--workspace`
- `.claude/rules/decision-queue.md` — Recipe 2 (`kind: "log"`) for durable findings; Recipe 1 (`kind: "blocker"`) only if genuinely blocked

## §3a Handover from prior cohort

(none — first cohort)

Task 0 pre-phase harness audit was run by the advisor (not a Junior task) and passed all 4
probes (EXIT_0 on probes 1-3, EXIT_NONZERO on probe 4 — correct). Impl gate cleared.
Flag: `.claude/audit-phase-v1-RT-r4-complete.flag`.

Phase branch tip at `phase-v1-RT-r4` = `5afe86554` (MiniMax trial designation commit).
DQ pending: 0.

## §4 Constraints

- **`[P]` parallel:** This task is cohort-parallel with Task 1 (which modifies `sponsor_allowlist.rs`). Files are disjoint — no overlap.
- **Derive stack verbatim:** copy the exact derive stack from `:444`/`:465` — `#[skip_serializing_none]` + `#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]` + both `cfg_attr` lines. Do NOT invent derives or drop any.
- **Newtype imports:** `PersonId`, `CommunityId`, `SponsorAllowlistId` must be imported from their canonical homes. Read the existing `use` block at the top of the file and add any missing imports there.
- **No cargo runs.** Junior workers on the daemon do not invoke cargo (Shape G suspended — cargo runs via advisor laptop validate gate). After commit + push, write a `kind: "validate-pending-laptop"` DQ entry with `commands: ["cmd //c \"scripts\\\\brehon\\\\cargo-check.bat -p lemmy_api_common --features full > .claude/validate-t2.log 2>&1\"", "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat -p lemmy_api_common --features full --no-deps -- -D warnings > .claude/validate-t2-clippy.log 2>&1\""]`, `branch: <worker-branch>`, `phase_task: 2`. Commit + push the DQ entry immediately after writing it.
- **Attribution:** DQ entries use `from: "impl"`, `answered_by: null`. Never `from: "advisor"`.
- **Commit subject:** `feat(rt-r4): add AddSponsorAllowlist + Remove admin DTOs in api_common (task 2)`.
- **HANDOVER YAML trailer** in commit body: `filesCreated: []`, `filesModified: [crates/api/api_common/src/governance.rs]`, `keyDecisions: [derive stack mirrored verbatim from AdminSetConfig:444]`, `notes: <verbatim observation if any>`.

## §5 Validation (laptop, after advisor reads validate-pending DQ)

Laptop advisor runs:
1. `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_common --features full > .claude/validate-t2.log 2>&1"` — exit 0 expected
2. `cmd //c "scripts\\brehon\\cargo-clippy.bat -p lemmy_api_common --features full --no-deps -- -D warnings > .claude/validate-t2-clippy.log 2>&1"` — exit 0 expected

(Junior does not invoke these — validation is advisor-run via validate-pending-laptop DQ flow.)

## §6 Commit message

`feat(rt-r4): add AddSponsorAllowlist + Remove admin DTOs in api_common (task 2)`

Body: HANDOVER YAML trailer as described in §4.
