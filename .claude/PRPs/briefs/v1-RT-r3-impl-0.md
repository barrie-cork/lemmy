# Brief: impl-task 0 — v1-RT-r3 pre-flight harness audit

## 1. Role + dispatch

`[role:impl-task] v1-RT-r3 task 0 — pre-flight harness audit — see .claude/PRPs/briefs/v1-RT-r3-impl-0.md`

## 2. Scope

Pre-flight audit only. **No code edits. No commits.** Verify the environment is ready for v1-RT-r3 impl tasks.

Run the 15 probes (Probe 0 through Probe 14) from plan §13 Task 0 in order:

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — wrapper sanity (cargo-check honors -p)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/audit-cargo-check-p.log 2>&1"
tail -20 .claude/audit-cargo-check-p.log
# EXPECT: only lemmy_utils compiles

# Probe 2 — feature flag activation
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/audit-cargo-check-features.log 2>&1"
echo "exit: $?"
tail -20 .claude/audit-cargo-check-features.log
# EXPECT: workspace compiles with --features full; exit 0

# Probe 3 — cargo-test wrapper honors target selection
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --no-run --features full > .claude/audit-cargo-test.log 2>&1"
tail -20 .claude/audit-cargo-test.log
# EXPECT: e2e test target compiles workspace-wide

# Probe 4 — wrappers fail loud on cargo errors (exit-code propagation)
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --no-run --features nonexistent_xyz > .claude/audit-cargo-test-negative.log 2>&1"
echo "cargo-test.bat exit on bogus feature: $?"
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features nonexistent_xyz > .claude/audit-cargo-check-negative.log 2>&1"
echo "cargo-check.bat exit on bogus feature: $?"
# EXPECT: BOTH non-zero (typically 101)

# Probe 5 — clippy baseline against post-r2 HEAD
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/audit-clippy-baseline.log 2>&1"
echo "exit: $?"
tail -40 .claude/audit-clippy-baseline.log
# EXPECT: exit 0

# Probe 6 — current branch is phase-v1-RT-r3
git branch --show-current
# EXPECT: phase-v1-RT-r3

# Probe 7 — r1 + r2 deliverables landed on the base
git log governance-v0 --oneline | head -25
# EXPECT: r1 (PR #126) + r2 (PR #150) merge commits visible

# Probe 8 — r1 schema columns (dedupe_key + source_event_type) present
rg "dedupe_key" crates/db_schema/src/source/governance/reputation_event.rs | wc -l
# EXPECT: at least 3 (struct field decl + form field decl + doc-comment)
rg "source_event_type" crates/db_schema/src/source/governance/reputation_event.rs | wc -l
# EXPECT: at least 3
rg "ParticipationCron|DormancyCron|VoteOutcome|EvidenceQuality" crates/db_schema_file/src/enums.rs | wc -l
# EXPECT: 4

# Probe 9 — r1 ENTRY_KIND consts pre-landed
rg "ENTRY_KIND_PARTICIPATION_CRON_TICK|ENTRY_KIND_VOTE_OUTCOME_RECORDED|ENTRY_KIND_EVIDENCE_QUALITY_RECORDED" crates/db_schema/src/source/governance/governance_log.rs | wc -l
# EXPECT: 3
rg "ENTRY_KIND_PARTICIPATION_CRON_TICK|ENTRY_KIND_VOTE_OUTCOME_RECORDED|ENTRY_KIND_EVIDENCE_QUALITY_RECORDED" crates/api/api/src/governance/governance_log.rs | wc -l
# EXPECT: 3 (shim re-export)

# Probe 10 — r1+v1-AD seeded all 10 r3-relevant config keys
rg 'deltas\.participation_weekly_active|participation\.dormancy_window_days|deltas\.participation_dormant|deltas\.participation_juror_aligned|participation\.activity_threshold_comments|participation\.lookback_days|deltas\.evidence_cited|deltas\.evidence_bad_faith|participation\.evidence_cited_rationale_threshold_chars|job\.participation_interval_days' crates/api/api/src/governance/config.rs | wc -l
# EXPECT: at least 30 (each key appears in DEFAULT_* const, match arm, key-list array, seed-fragment — at least 3 occurrences per key × 10 keys)

# Probe 11 — partial unique index on dedupe_key present in migration
rg "reputation_event_dedupe_key_partial_idx" migrations/2026-05-10-000000-0000_add_reputation_event_v1_columns/up.sql | wc -l
# EXPECT: at least 1 (CREATE UNIQUE INDEX line)

# Probe 12 — submit_jury_vote.rs emit_reputation_event helper present
rg "^async fn emit_reputation_event" crates/api/api/src/governance/submit_jury_vote.rs | wc -l
# EXPECT: 1
rg "Some\(ReputationEventSourceType::JuryVote\)" crates/api/api/src/governance/submit_jury_vote.rs | wc -l
# EXPECT: at least 1 (the hard-coded source in the v0 helper body — Task 2 changes this)

# Probe 13 — concurrent-PR check (no other PR touches r3's IMPLEMENT files)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("participation_cron|submit_jury_vote|admin_emergency_remove|scheduled_tasks|api_common/src/governance|api/routes/src/lib")) | {number, title, headRefName}'
# EXPECT: empty output; if any other lane is touching r3 files, STOP

# Probe 14 — registry invariant: ENTRY_KIND const count == unique string-literal count
rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
# EXPECT: numeric count A
rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs \
  | awk -F: '/ENTRY_KIND_/ {print}' \
  | grep -oE '"[a-z_]+"' | sort -u | wc -l
# EXPECT: same count B == A
rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs \
  | awk -F: '/ENTRY_KIND_/ {print}' \
  | grep -oE '"[a-z_]+"' | sort | uniq -d
# EXPECT: empty
```

**EXPECT block:**
- Probes 0..3, 5..14 exit 0 (or as documented per probe)
- Probe 4 exits NON-ZERO (negative test confirms exit-code propagation)
- Probe 6 returns `phase-v1-RT-r3`
- Probes 8-12 confirm r1 + r2 deliverables intact (mechanical checks)
- Probe 13 returns empty (no concurrent PR overlap)
- Probe 14: count A == count B; `uniq -d` empty (registry invariant holds; r3 is consuming pre-landed consts, not adding new ones)

If any probe fails: write a `kind: "blocker"` DQ entry to `.claude/decision-queue.json`, commit + push it on the worker branch, and stop.

**Task 0 produces NO commit** if all probes pass — write results to task output only.

## 3. Required reading

- `.claude/PRPs/plans/v1-RT-r3.plan.md` §13 Task 0 (full probe list + EXPECT block — authoritative; this brief mirrors it)
- `.claude/rules/pre-phase-harness-audit.md` (R5: enumerate ALL probes explicitly)
- `.claude/rules/decision-queue.md` (DQ schema-v3 + Junior subagent attribution rules)

## 4. Constraints

- **No code edits** — this task is verification only.
- **No commit on success** — audit output goes to task output, not git.
- If any probe fails: file `kind: "blocker"` DQ (use `bash scripts/brehon/dq-v3-new-entry.sh` for the composite id; use `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending` to append), commit + push on the worker branch, then stop. Do not proceed to Task 1.
- Branch MUST be `phase-v1-RT-r3` (Probe 6 confirms this; if not, file a blocker DQ).
- Shape G is **SUSPENDED** per DQ #229 — this is a pre-Shape-G plan. Cargo runs on the laptop via `validate-pending-laptop`. Task 0 audit cargo probes (1, 2, 3, 5) are exceptions: they run inline on the EliteDesk worker because Task 0 has no commit/push and the per-task validate-pending pathway doesn't apply to verification-only tasks.

## 5. Forbidden-window check (advisor pre-queue)

Per `.claude/rules/advisor-orchestrator.md` §5.1: Shape G suspended → forbidden-window check IS binding for this task. Advisor checks at queue time; subagent's task-0 pre-flight refuses with `FORBIDDEN_WINDOW: <window>` if mis-queued.

## 6. Context

- Phase: v1-RT-r3
- Plan: `.claude/PRPs/plans/v1-RT-r3.plan.md` (on trunk `governance-v0`)
- Phase branch: `phase-v1-RT-r3` (cut from governance-v0; runlog entry at `dc9781a74`)
- Base branch for this task: `phase-v1-RT-r3`
- This is the first impl-task in v1-RT-r3. After it passes, advisor queues cohort-2 (Tasks 1+2+3 `[P]` parallel-eligible; per §4.1 step 5 budget check, pre-Shape-G cohort-of-3 with ~6 GB each ≫ 10 GB cap so the advisor may degrade to serial).
