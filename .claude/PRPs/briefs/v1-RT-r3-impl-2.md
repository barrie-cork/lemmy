# Brief: impl-task 2 — v1-RT-r3 vote-outcome + evidence-cited emit in submit_jury_vote

## 1. Role + dispatch

`[role:impl-task] v1-RT-r3 task 2 — vote-outcome + evidence-cited emit in submit_jury_vote — see .claude/PRPs/briefs/v1-RT-r3-impl-2.md`

## 2. Scope

Implement §13 Task 2 of `.claude/PRPs/plans/v1-RT-r3.plan.md` verbatim.

**FILES (per plan §13 Task 2 FILES yaml):**

- creates: []
- modifies: `crates/api/api/src/governance/submit_jury_vote.rs` (emit helper signature + Sources 3 + 4a emit)
- requires: [] (Task 0 already done; no inter-task dependencies)

**IMPLEMENT (file 1 of 1):** Follow plan §13 Task 2 IMPLEMENT steps 1-5 verbatim:
1. Extend `emit_reputation_event` (lines 968-994) per §10.2 — add `source_event_type: ReputationEventSourceType` + `dedupe_key: Option<String>` parameters; populate matching fields in `ReputationEventInsertForm`; add `.on_conflict_do_nothing()`.
2. Update 2 existing JuryVote call sites (lines 583, 615): append `ReputationEventSourceType::JuryVote, None` as last two args.
3. Source 3 vote-outcome insert per §10.3 — inside existing juror loop (lines 577-597), after JuryReliability emit at line 583, only when `juror_decision == winning_decision`.
4. Source 4a evidence-cited insert per §10.4 — after reporter emit at line 625 (still inside `if path_kind == SLDPathKind::Decided`).
5. Add `ENTRY_KIND_VOTE_OUTCOME_RECORDED` + `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED` + `case_evidence` schema + `case_evidence::table` to `use` block at lines 42-86.

**VALIDATE (story-checkpoint feeds §16a Stories 3 + 5):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r3-task2-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r3-task2-check.log
# EXPECT: exit 0
```

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r3-task2-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-RT-r3-task2-clippy.log
# EXPECT: exit 0
```

**Note:** Shape G is SUSPENDED per DQ #229 — cargo runs on the laptop. Junior subagent must NOT execute VALIDATE locally.

**Post-validate:** Write `kind: "validate-pending-laptop"` DQ entry per `advisor-orchestrator.md` §5.2.
Required fields:
- `commands`: the 2 VALIDATE bash blocks above (verbatim)
- `branch`: `"phase-v1-RT-r3"`
- `phase_task`: `2`

Generate id via `bash scripts/brehon/dq-v3-new-entry.sh`. Append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Commit + push DQ to worker branch.

## 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1
    commit: b9876b7d8
    filesCreated:
      - crates/api/api/src/governance/participation_cron.rs
    filesModified:
      - crates/api/api/src/governance/mod.rs
      - crates/routes/src/utils/scheduled_tasks.rs
    keyDecisions:
      - sql_query for both discovery queries (activity GROUP BY HAVING + dormancy NOT EXISTS anti-join)
      - on_conflict_do_nothing untargeted (partial index; targeted form not supported)
      - ENTRY_KIND_PARTICIPATION_CRON_TICK used for both source-1 and source-2 governance_log entries
    notes: validate-pending-laptop DQ c76792506538-001 PASS (cargo-check + cargo-clippy --no-deps -- -D warnings clean, advisor-laptop SHA 7c1bcddab on phase-v1-RT-r3)
```

## 3. Required reading

- `.claude/PRPs/plans/v1-RT-r3.plan.md` §13 Task 2 (authoritative IMPLEMENT + MIRROR + GOTCHA + VALIDATE)
- `.claude/PRPs/plans/v1-RT-r3.plan.md` §10.2 (emit_reputation_event signature extension verbatim)
- `.claude/PRPs/plans/v1-RT-r3.plan.md` §10.3 (Source 3 vote-outcome emit block verbatim)
- `.claude/PRPs/plans/v1-RT-r3.plan.md` §10.4 (Source 4a evidence-cited emit block verbatim)
- `crates/api/api/src/governance/submit_jury_vote.rs:968-994` (current `emit_reputation_event` shape)
- `crates/api/api/src/governance/submit_jury_vote.rs:577-597` (juror loop where Source 3 lands)
- `crates/api/api/src/governance/submit_jury_vote.rs:600-625` (reporter emit AFTER which Source 4a lands)
- `crates/api/api/src/governance/submit_jury_vote.rs:687-699` (governance_log::append `&mut (&mut *conn).into()` syntax)
- `crates/api/api/src/governance/admin_emergency_remove.rs:276` (`actor_pseudonym_helper::get_or_create(&mut conn.into(), ...)` shape)
- `crates/db_schema/src/source/governance/jury_vote.rs:22+33` (`jury_vote.rationale` is `Option<String>`)
- `crates/db_schema/src/source/governance/case_evidence.rs:20` (`uploader_id` field — authoritative naming)
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — Source 3 + 4a emits land inside existing `run_transaction` at line 139; no new transaction wrap needed
- `.claude/lessons/feedback_features_full_workspace_only.md` + `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — cargo always uses `--workspace --features full`
- `.claude/lessons/feedback_clippy_test_style.md` + `.claude/lessons/feedback_clippy_rerun_after_fix.md` — Lemmy workspace test-style; clippy denies unwrap/expect/allow_attributes
- `.claude/lessons/feedback_laptop_default_for_validate_pending.md` — Shape G suspended; cargo runs on laptop
- `.claude/rules/decision-queue.md` (DQ schema-v3; impl-task writes `kind: "validate-pending-laptop"`; ALWAYS use `dq-v3-new-entry.sh`)

## 4. Constraints

- **No edits outside `submit_jury_vote.rs`.** Touching any other file is a process breach.
- **One commit** — `feat(governance): vote-outcome + evidence-cited emit in submit_jury_vote (task 2)`.
- **`winning_rationales` length check uses `r.chars().count()` not `r.len()`** — PRD threshold (`evidence_cited_rationale_threshold_chars`) is measured in UTF-8 characters.
- **`cache: &mut ConfigCache` borrow passed via `&mut` reborrow, NOT cloned** — mirror existing pattern at lines 547, 554.
- **`case_evidence::uploader_id` is the field name** — NOT `submitter_id` (the brief's clarify DQ -021 used "submitter_id (or equivalent person_id FK)"; the schema is authoritative).
- **`governance_log::append` calls use `&mut (&mut *conn).into()` syntax** — mirror `submit_jury_vote.rs:687-699`.
- **No clippy `#[allow]`** — use `#[expect(...)]` only after verifying lint is intentional.
- **`.on_conflict_do_nothing()` (untargeted) for the reputation_event INSERT** — partial unique index requires untargeted form.
- **Mid-task DQ push** — raise `kind: "blocker"` immediately if you find Source 3 / 4a callsite ambiguity or `winning_rationales` not in scope.
- **LESSON-trailer convention** — end commit body with `LESSON:` if a durable pattern emerges.

## 5. Forbidden-window check (advisor pre-queue)

Per `advisor-orchestrator.md` §5.1: Shape G suspended → forbidden-window check is binding for laptop validate-pending-laptop runs. Advisor checks at queue time AND at DQ mutation time.

Current UTC at queue: Mon 19:08 UTC (primary window 16:00–02:30 — OK).

## 6. Context

- Phase: v1-RT-r3
- Plan: `.claude/PRPs/plans/v1-RT-r3.plan.md` (on trunk + phase branch)
- Phase branch: `phase-v1-RT-r3` @ `0401709e0`
- Base branch for this task: `phase-v1-RT-r3`
- Cohort 2 — Tasks 1 + 2 + 3 dispatched in parallel (`[P]` per plan §13); YAML overlap check confirmed disjoint file sets.
- Prior cohort handover (Task 0): all 19 probes pass; submit_jury_vote.rs unchanged on phase branch.
