---
phase: v1-RT-r1
role: impl-task
task: fix-impl-5
brief_n: 5
authored: 2026-05-12
parent_cr: cr-3
---

# [role:impl-task] RT-r1 fix-impl-5 — set source_event_type for jury-vote events in submit_jury_vote.rs — see .claude/PRPs/briefs/rt-r1-fix-impl-5.md

## §1 Role + dispatch

`[role:impl-task] RT-r1 fix-impl-5 — set source_event_type for jury-vote events in submit_jury_vote.rs`

## §2 Scope

### 2.1 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md` §G4 classifier table)

This is a CR fix-in-pr, not an allowlist §G4 entry.

**CR finding cr-3:**
> In `crates/api/api/src/governance/submit_jury_vote.rs` around line 944-945: The reputation event builder is leaving `source_event_type` as `None` (which falls back to DB default Endorsement) for jury-vote events; update the struct literal in submit_jury_vote (the place that sets `dedupe_key` and `source_event_type`) to set `source_event_type` to the correct enum variant for jury votes (e.g., `Some(RepEventSource::JuryVote)` or the project's equivalent) so jury-vote events are classified correctly. Ensure you import or qualify the enum variant used.

### 2.2 File edits

**File:** `crates/api/api/src/governance/submit_jury_vote.rs`

**Edit 1 — Fix source_event_type at line 945:**
- Current: `source_event_type: None,`
- Change to: `source_event_type: Some(ReputationEventSourceType::JuryVote),`
- The `ReputationEventSourceType` enum is imported at line 63 via `use lemmy_db_schema_file::enums::...` — verify `ReputationEventSourceType` is already imported; if not, add it to the existing use statement.
- The `JuryVote` variant exists in `ReputationEventSourceType` (defined in `crates/db_schema_file/src/enums.rs`).

**Touch only:** `crates/api/api/src/governance/submit_jury_vote.rs`

## §3 Required reading

1. `.claude/rules/decision-queue.md` — Recipe 1 if blocker found

## §4 Constraints

- **Touch only:** `crates/api/api/src/governance/submit_jury_vote.rs` + `.claude/decision-queue.json` (if blocker raised)
- **1 line change** — just the `source_event_type` field at line 945 + possibly 1 import line
- **Verify the import** — `ReputationEventSourceType` may already be imported via `lemmy_db_schema_file::enums`; check line ~63 before adding a duplicate import
- **Commit message:** `fix(v1-RT-r1): set source_event_type JuryVote in submit_jury_vote reputation event (cr-3)`
- **Shape G:** after committing, push worker branch; write `kind: "validate-pending"` DQ entry with `workflow_run_id` from `gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --json databaseId`. Commit + push DQ entry.
- **DQ atomic raise:** `git add .claude/decision-queue.json && git commit && git push origin <branch>` THEN end task.
- **Encoding:** Python `json.dump(..., ensure_ascii=False, indent=2)` for DQ writes.
- **next_id:** compute across `.claude/decision-queue.json` AND `.claude/decision-queue-archive-*.json`. Current max is 210 (fix-impl-4 will have claimed it) — next id is 211.
- **Attribution:** `from: "impl"`, never `from: "advisor"`.
- **Base branch:** `phase-v1-RT-r1`
