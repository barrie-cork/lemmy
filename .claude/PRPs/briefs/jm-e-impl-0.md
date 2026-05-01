---
role: impl-task
plan_task: 0
phase: v1-JM-e
created: 2026-05-01
related_dq: null
---

# Brief — v1-JM-e Task 0 — Pre-flight harness audit + branch verification + JM-d state confirmation

## 1. Role + dispatch line

`[role:impl-task] v1-JM-e task 0 — see .claude/PRPs/briefs/jm-e-impl-0.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan Task 0 from `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` §13 — verification only. **No commit is produced at Task 0.** If any probe fails, STOP and surface to advisor via DQ before attempting Task 1.

## 2. Scope

**Produce:** a written probe-result summary in your task output (no commit, no file edits).

**Do NOT** in this task:
- Write any Rust, SQL, or YAML.
- Commit anything.
- Proceed to Task 1 logic if any probe exits non-zero.

## 3. Required reading

Read in this order before running any probe:

1. **Plan §13 Task 0** — the full probe list (Probes -1 through 9). This brief mirrors it verbatim; the plan is canonical.
2. **Plan §4.1** — architectural decisions locked in this sub-phase (understanding the before/after state).
3. **`.claude/rules/pre-phase-harness-audit.md`** — the harness audit rule (R5 enumerates all probes).
4. **`.claude/rules/pm-plugin-hooks-stable.md`** — load-bearing hook names Probe 7 checks.
5. **Lessons** (Glob `.claude/lessons/`, read any with keywords matching `harness`, `audit`, `probe`, `submodule`, `worktree`):
   - `feedback_pipes_mask_exit_codes.md` — never pipe output through grep/head; capture to file + check `$?`.
   - `feedback_worktree_submodules_not_auto_init.md` — Probe -1 catches this on Linux/Junior worktrees.
   - `feedback_pre_phase_dod_smoke_test.md` — DoD smoke informs probe shapes.

## 3a. Handover from prior cohort

(none — first cohort)

## 4. Constraints

### Branch + environment
- You start on a Junior worktree branched off `phase-v1-JM-e` (tip `ebb34bb41`).
- `git branch --show-current` MUST return `phase-v1-JM-e` (Probe 1 confirms). If it returns something else, file a DQ catch-fire entry.
- No commit at Task 0 — verification only. Finalize has nothing to merge.

### Probe discipline
- Run ALL probes in the order listed (Probe -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9).
- Capture probe output to `/tmp/jm-e-task0-probe-N.log` (or equivalent writable temp path on the EliteDesk).
- Check `$?` after each probe separately — do not pipe through `head`/`grep`.
- On any Probe 0 or Probe 1 FAIL: STOP immediately. Write a DQ pending entry (`from: "impl"`, `kind: "blocker"`) with the probe output and exit code. Commit + push the DQ entry to your worktree branch immediately.
- On Probe 6 (DQ advisory): list pending entries in your task output; advisor handles resolution.
- On Probe 8 (concurrent-PR check): if output is non-empty, report to advisor via task output — do not STOP; advisor decides whether to proceed.

### Line-number drift
- Plan §4.2 watchpoints cite specific line numbers (e.g. `submit_jury_vote.rs:140-184`). Probe 5 checks function presence by `rg -n`, not line numbers. If Probe 5 returns unexpected output (e.g. `process_appeal_vote` already present), file a DQ before proceeding — JM-e Task 1 may have been partially applied.

### DQ attribution
- No `answered_by: "advisor"` or `"user"` from this subagent.
- Self-resolve only as `"impl-self-resolved"`.

## 5. Probes to run (per plan §13 Task 0)

Run all probes. Capture output per the constraint above.

```bash
# Probe -1 — submodule init (Linux/Junior worktree quirk)
git submodule status > /tmp/jm-e-task0-submodule.log 2>&1
if grep -q '^-' /tmp/jm-e-task0-submodule.log; then
  git submodule update --init --recursive > /tmp/jm-e-task0-submodule-init.log 2>&1
  echo "submodule init exit: $?"
fi

# Probe 0 — Docker daemon running
docker ps > /tmp/jm-e-task0-docker.log 2>&1 && echo "DOCKER OK" || {
  echo "DOCKER NOT RUNNING"; exit 1;
}

# Probe 1 — branch verification
git branch --show-current
# EXPECT: phase-v1-JM-e

# Probe 2 — JM-d state confirmation: appeal schema columns present
rg -n 'decided_at -> Nullable<Timestamptz>' crates/db_schema_file/src/schema.rs | head
rg -n 'panel_size_snapshot -> Nullable<Int4>' crates/db_schema_file/src/schema.rs | head

# Probe 3 — JM-d state confirmation: helpers present
rg -n 'pub async fn select_appeal_panel|pub async fn seat_appeal_panel' crates/api/api/src/governance/admin_assign_jury.rs | head

# Probe 4 — JM-d state confirmation: ENTRY_KIND_APPEAL_DECIDED const declared
rg -n 'pub const ENTRY_KIND_APPEAL_DECIDED' crates/db_schema/src/source/governance/governance_log.rs | head

# Probe 5 — JM-c state confirmation: process_vote shape intact; process_appeal_vote absent
rg -n 'async fn process_vote|async fn process_appeal_vote' crates/api/api/src/governance/submit_jury_vote.rs | head
# EXPECT: process_vote present; process_appeal_vote NOT present

# Probe 6 — DQ pending status (advisory)
python3 -c "import json; d=json.load(open('.claude/decision-queue.json')); print('pending:', [(e['id'], e.get('kind')) for e in d.get('pending',[])])"

# Probe 7 — PM-plugin-hooks-stable check
for h in \
  local_private_message_before_create \
  local_private_message_after_create \
  local_private_message_before_update \
  local_private_message_after_update \
  federated_private_message_before_receive \
  federated_private_message_after_receive; do
  rg -q "\"$h\"" crates/ || { echo "missing hook literal: $h"; exit 1; }
done
echo "PM hooks: all 6 present"

# Probe 8 — concurrent-PR check (advisory)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("submit_jury_vote\\.rs|tests/e2e\\.rs")) | {number, title, headRefName}'
# EXPECT: empty output — advisor decides if non-empty

# Probe 9 — Shape G workflow YAMLs lint-clean
yamllint .github/workflows/cargo-validate-workspace.yml .github/workflows/cargo-test-e2e.yml > /tmp/jm-e-task0-yamllint.log 2>&1
echo "yamllint exit: $?"
```

## 6. Expected output (return to advisor)

```
## Task 0 complete — v1-JM-e pre-flight audit

**Branch:** phase-v1-JM-e (confirmed)
**Probe results:**
  - Probe -1: submodule OK (or: init ran, exit N)
  - Probe 0: DOCKER OK (or: FAIL — stop)
  - Probe 1: phase-v1-JM-e (PASS)
  - Probe 2: decided_at + panel_size_snapshot lines found at <lines>
  - Probe 3: select_appeal_panel + seat_appeal_panel at <lines>
  - Probe 4: ENTRY_KIND_APPEAL_DECIDED at <line>
  - Probe 5: process_vote at <line>; process_appeal_vote absent (PASS)
  - Probe 6: pending DQ entries: <list or empty>
  - Probe 7: PM hooks all 6 present (PASS)
  - Probe 8: concurrent-PR check: <empty or list — advisory>
  - Probe 9: yamllint exit 0 (PASS)
**No commit produced.**
**Next:** advisor queues Task 1 (appeal-vote tally branch + step_up_token DTO)
```

If any blocking probe failed, replace the above with a DQ catch-fire entry (commit + push to worktree branch immediately).
