# Complete `/auto-phase` Context Management Validation and Cleanup Plan

Status: handoff plan for local agent execution.

Scope: Claude Code harness only unless explicitly split into a separate Pi cleanup branch/commit. Do not modify `.pi/**` as part of this `/auto-phase` validation plan. Do not create a second harness.

## Current baseline

Recent committed `/auto-phase` context-management chain:

- `764dbd1e3 chore(auto-phase): add stage digest ring schema`
- `529aae6dc chore(auto-phase): add spill guard and auto-handover contract`
- `328991851 chore(auto-phase): wire compact resume to durable ledger`
- `5512e50e2 chore(auto-phase): implement compact ledger wiring`

Repo-tracked docs/schema now define:

- `stage_digests` + `digest_overflow_path`
- `spill_dir`
- `last_handover_path` + `last_handover_at`
- digest-first `/compact` and Phase 0.5 resume semantics
- `next_action_hypothesis` as re-verify-only

User-scope command files were intentionally not edited by the previous agent. Their exact insertion notes are in:

- `.claude/PRPs/reports/phase-2-auto-phase-spill-handover-command-notes.md`
- `.claude/PRPs/reports/phase-3-auto-phase-compact-ledger-plan.md`

## Goals

1. Apply the remaining user-scope command wiring safely.
2. Validate that the schema, docs, command bodies, runtime writes, and compact/resume behavior are hooked up correctly.
3. Confirm no `.pi/**` or unrelated local changes are included in any Claude `/auto-phase` commit.
4. Clean up remaining planning/research artifacts after validation, either by committing them deliberately, moving them to an archive location, or deleting them with explicit approval.

## Guardrails

- Do not edit `.pi/**` in this plan.
- Do not stage unrelated modified files such as `scripts/brehon/pmd-query.sh`, `scripts/brehon/pmd-write.sh`, or `.claude/PRPs/templates/plan.template.md` unless separately reviewed and explicitly declared in scope.
- Do not treat narrative digest content as routing, gate, cadence, catch-fire, or merge authority.
- Treat every `next_action_hypothesis` as a hypothesis to re-verify against live TaskList/DQ/PR/branch state before acting.
- Keep `.claude/auto-state/` gitignored.
- Preserve advisor-wide / junior-narrow split: the ledger is durable and wide; Junior briefs remain one-task scoped.

## Phase A — Preflight and inventory

Run:

```sh
git status --short
git show --no-patch --format='%H %s' HEAD
git log --oneline -6 -- .claude/refs/auto-phase.md .claude/refs/compact-prompt-approach.md .claude/PRPs/templates/auto-phase-state.template.json .claude/PRPs/reports/phase-2-auto-phase-spill-handover-command-notes.md .claude/PRPs/reports/phase-3-auto-phase-compact-ledger-plan.md
```

Confirm:

- HEAD includes `5512e50e2 chore(auto-phase): implement compact ledger wiring` or a later intentional successor.
- Existing unrelated local changes are noted and not staged.
- `.pi/**` changes are out of scope.

## Phase B — Apply user-scope command wiring

### B1. Backup user-scope command files

Before editing:

```sh
mkdir -p .claude/PRPs/reports/user-scope-command-backups
cp ~/.claude/commands/auto-phase.md .claude/PRPs/reports/user-scope-command-backups/auto-phase.$(date -u +%Y%m%dT%H%M%SZ).md
cp ~/.claude/commands/compact-phase.md .claude/PRPs/reports/user-scope-command-backups/compact-phase.$(date -u +%Y%m%dT%H%M%SZ).md
```

If either command file does not exist, record that fact in a report before creating or editing it.

### B2. Apply Phase 2 `/auto-phase` user-scope blocks

Use `.claude/PRPs/reports/phase-2-auto-phase-spill-handover-command-notes.md` as the source of truth.

Apply to `~/.claude/commands/auto-phase.md`:

- Phase 0.5 Step A schema-v3 additive backfill:
  - `spill_dir = .claude/auto-state/<phase>.spill/`
  - `last_handover_path = null`
  - `last_handover_at = null`
  - set `schema_version = 3`
- Phase 1 standing tool-output spill guard.
- Phase 1 auto-handover refresh immediately after the digest append/write step.

### B3. Apply Phase 3 `/compact` and resume blocks

Use `.claude/PRPs/reports/phase-3-auto-phase-compact-ledger-plan.md` as the source of truth.

Apply to `~/.claude/commands/compact-phase.md`:

- Priority 1 active-thread instruction reads `.claude/auto-state/<phase>.json` for `/auto-phase` sessions.
- It cites `stage_digests[-1]` and `last_handover_path`.
- It marks `next_action_hypothesis` as re-verify-only.

Apply to `~/.claude/commands/auto-phase.md`:

- Phase 0.5 Step E builds COMPACT resume report from `stage_digests[-3:]` and `last_handover_path`.
- It re-verifies `next_action_hypothesis` against live TaskList/DQ/PR/branch state.
- It preserves read-only-until-`continue` behavior.

### B4. Record user-scope patch notes in repo

Create or update a repo-tracked report, for example:

`.claude/PRPs/reports/auto-phase-user-scope-command-application-2026-06-11.md`

Include:

- command files edited
- backup paths
- exact sections changed
- whether files existed before editing
- validation commands run
- any deviations from the patch-note blocks

Commit only this report if repo-tracked evidence is desired. Do not commit the actual user-scope command files, because they live outside the repo.

## Phase C — Static validation

Run from repo root:

```sh
python3 - <<'PY'
import json
from pathlib import Path
p = Path('.claude/PRPs/templates/auto-phase-state.template.json')
data = json.loads(p.read_text())
assert data['schema_version'] == 3
for key in ['stage_digests', 'digest_overflow_path', 'spill_dir', 'last_handover_path', 'last_handover_at']:
    assert key in data, key
assert isinstance(data['stage_digests'], list)
print('schema ok')
PY

git check-ignore -v \
  .claude/auto-state/test-state.json \
  .claude/auto-state/v1-SL-c-2.spill/example.txt \
  .claude/auto-state/v1-SL-c-2.digests.jsonl

grep -R -F 'stage_digests[-1]' .claude/refs/compact-prompt-approach.md .claude/refs/auto-phase.md .claude/PRPs/reports/phase-3-auto-phase-compact-ledger-plan.md
grep -R -F 'stage_digests[-3:]' .claude/refs/auto-phase.md .claude/PRPs/reports/phase-3-auto-phase-compact-ledger-plan.md
grep -R -F 'last_handover_path' .claude/refs/compact-prompt-approach.md .claude/refs/auto-phase.md .claude/PRPs/reports/*.md
grep -R -F 'next_action_hypothesis' .claude/refs/compact-prompt-approach.md .claude/refs/auto-phase.md .claude/PRPs/reports/*.md
grep -R -F 're-verify' .claude/refs/compact-prompt-approach.md .claude/refs/auto-phase.md .claude/PRPs/reports/*.md
```

Confirm:

- JSON parses.
- Schema version is `3`.
- Phase 1 and Phase 2 fields are present.
- `.claude/auto-state/` covers state, spill, and digest overflow runtime files.
- Repo docs and command patch reports consistently mention digest-first compact/resume behavior.

## Phase D — Command-body validation

Because command files are user-scope, validate them explicitly:

```sh
grep -n -F 'spill_dir' ~/.claude/commands/auto-phase.md
grep -n -F 'last_handover_path' ~/.claude/commands/auto-phase.md
grep -n -F 'stage_digests[-3:]' ~/.claude/commands/auto-phase.md
grep -n -F 'next_action_hypothesis' ~/.claude/commands/auto-phase.md
grep -n -F 'stage_digests[-1]' ~/.claude/commands/compact-phase.md
grep -n -F 'last_handover_path' ~/.claude/commands/compact-phase.md
grep -n -F 're-verify' ~/.claude/commands/compact-phase.md
```

Confirm:

- `/auto-phase` command body has schema-v3 backfill.
- `/auto-phase` command body has spill guard and auto-handover refresh.
- `/auto-phase` Phase 0.5 Step E reads `stage_digests[-3:]` and `last_handover_path`.
- `/compact` priority 1 reads `stage_digests[-1]` and `last_handover_path` for `/auto-phase` runs.
- Both commands preserve `next_action_hypothesis` as re-verify-only.

## Phase E — Runtime simulation without touching live phase state

Use a disposable phase name such as `validation-auto-phase-context-20260611`.

Create a temporary validation state under `.claude/auto-state/` only. Since the directory is gitignored, do not commit it.

Suggested simulation checks:

1. Create or initialize a v1/v2-like state and run the resume/backfill logic, if the command supports a dry run or safe local invocation.
2. Confirm schema-v3 fields are added with defaults.
3. Simulate a stage transition and confirm:
   - one digest appends after the stage update
   - ring cap remains 12
   - overflow writes to `<phase>.digests.jsonl` when cap is exceeded
4. Simulate a large tool result over 16000 chars and confirm:
   - full output writes to `.claude/auto-state/<phase>.spill/<stage>-<tool>-<UTC-iso>.txt`
   - live context keeps only head + tail + path
5. Simulate auto-handover refresh and confirm:
   - `.claude/PRPs/handovers/<phase>-auto-<UTC-date>.md` is written or refreshed
   - ledger records `last_handover_path`
   - ledger records `last_handover_at`
6. Simulate resume report generation and confirm:
   - report uses `stage_digests[-3:]`
   - report cites `last_handover_path`
   - report marks `next_action_hypothesis` as re-verify-only
   - no state-changing action happens before user `continue`

Record results in:

`.claude/PRPs/reports/auto-phase-context-management-validation-2026-06-11.md`

## Phase F — Real dogfood validation

Run a real `/auto-phase` session only after static and simulation checks pass.

Minimum observations to record:

- On first transition, state file contains a new digest entry.
- On any large output, spill file is created and referenced.
- On transition, auto-handover refresh occurs and ledger points to it.
- On compact/resume, `/compact` summary uses `stage_digests[-1]` and `last_handover_path`.
- On resume, Phase 0.5 Step E uses `stage_digests[-3:]` and live re-verification before action.

If a real run is too risky, explicitly mark dogfood validation deferred and record what remains unverified.

## Phase G — Cleanup inventory

Current unrelated working-tree items to review separately:

Modified non-`.pi` files:

- `.claude/PRPs/templates/plan.template.md`
- `scripts/brehon/pmd-query.sh`
- `scripts/brehon/pmd-write.sh`

Untracked Claude/doc artifacts:

- `.claude/PRPs/plans/brehon_context_management_review.md`
- `.claude/PRPs/plans/claude_auto_phase_context_patch_plan.md`
- `.claude/PRPs/plans/pi-harness-context-injection.plan.md`
- `.claude/PRPs/reports/pi-harness-context-injection-retro.md`
- `.claude/PRPs/reports/session-retro-2026-06-10-pi-harness-context-injection.md`
- `.claude/lessons/feedback_pi_monolithic_context_dump.md`
- `.claude/lessons/reference_pi_progressive_disclosure_by_role.md`
- `docs/research/brehon_context_management_review.md`
- `docs/research/claude_auto_phase_context_patch_plan.md`

Modified `.pi/**` files:

- Treat as separate Pi harness work. Do not mix with Claude `/auto-phase` commits.

## Phase H — Cleanup decision tree

For each non-`.pi` artifact, decide one of:

1. Keep and commit as Claude harness context/research.
2. Move to an archive/report location and commit.
3. Delete as superseded scratch, after explicit confirmation.
4. Leave untracked with a clear reason.

Suggested grouping:

- Claude `/auto-phase` context-management artifacts:
  - `.claude/PRPs/plans/claude_auto_phase_context_patch_plan.md`
  - `docs/research/claude_auto_phase_context_patch_plan.md`
  - Consider whether one is canonical and the other duplicate.
- Brehon context-management research:
  - `.claude/PRPs/plans/brehon_context_management_review.md`
  - `docs/research/brehon_context_management_review.md`
  - Consider whether one belongs in docs/research and one should be removed.
- Pi harness artifacts:
  - `.claude/PRPs/plans/pi-harness-context-injection.plan.md`
  - `.claude/PRPs/reports/pi-harness-context-injection-retro.md`
  - `.claude/PRPs/reports/session-retro-2026-06-10-pi-harness-context-injection.md`
  - `.claude/lessons/feedback_pi_monolithic_context_dump.md`
  - `.claude/lessons/reference_pi_progressive_disclosure_by_role.md`
  - Handle only in a separate Pi cleanup commit if desired.

## Phase I — Final commit strategy

Recommended separate commits:

1. User-scope command application evidence, if a repo report is created:

```text
chore(auto-phase): record command wiring application
```

2. Validation report:

```text
chore(auto-phase): validate compact ledger wiring
```

3. Cleanup of Claude-only research/planning artifacts, if retained:

```text
docs(auto-phase): archive context management planning artifacts
```

4. Separate Pi cleanup, only if explicitly in scope:

```text
chore(pi): clean up context injection artifacts
```

Before each commit:

```sh
git diff --cached --name-status
if git diff --cached --name-only | grep -q '^.pi/'; then echo 'ERROR: .pi staged in Claude commit'; exit 1; fi
```

## Done criteria

- User-scope `/auto-phase` and `/compact` command files contain the Phase 2/3 blocks.
- Schema parses and remains version `3`.
- Runtime state/spill/digest paths are gitignored.
- A validation report records static checks and either simulated or real dogfood results.
- Any final commits exclude unrelated local changes and `.pi/**` unless deliberately handled in a separate Pi-scoped commit.
- Working tree is either clean or has documented, intentionally deferred changes.
