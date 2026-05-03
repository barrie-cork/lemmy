---
role: impl-task
plan_task: 0
phase: v1-SL-a
created: 2026-05-03
related_dq: 114
---

# Brief — v1-SL-a Task 0 — Pre-flight harness audit + branch verification + migrate-roundtrip.sh stub fix

## 1. Role + dispatch line

`[role:impl-task] v1-SL-a task 0 — see .claude/PRPs/briefs/sl-a-impl-0.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan Task 0 from `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` §13 (lines 1250-1414) — verification probes **plus** one file modification (`scripts/brehon/migrate-roundtrip.sh` stub replacement per DQ #114). Unlike JM-e Task 0 (verification-only), this Task 0 produces ONE commit.

## 2. Scope

**Produce:**

1. A written probe-result summary in your task output (Probes -1 through 10).
2. **One commit** on your Junior worktree branch replacing the stub body of `scripts/brehon/migrate-roundtrip.sh` with the real round-trip implementation specified in plan §13 Task 0 (lines 1341-1392).

**Do NOT** in this task:
- Touch any `crates/**`, `migrations/**`, `.claude/**`, or `docs/**` files. The ONLY editable file is `scripts/brehon/migrate-roundtrip.sh`.
- Run cargo locally — Shape G plan; cargo runs on GH-hosted runners after push.
- Push the worker branch yourself unless the task workflow requires it. Junior's daemon finalize-merges into the phase branch on completion.
- Proceed to Task 1 logic on any probe failure or stub-replacement failure.

## 3. Required reading

Read in this order before running any probe:

1. **Plan §13 Task 0** (`.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` lines 1250-1414) — the full probe list AND the migrate-roundtrip.sh real-implementation source. This brief mirrors it; the plan is canonical.
2. **DQ #114** (`.claude/decision-queue.json` resolved entries) — confirms the stub-fix scope landed under this Task 0 (Option A).
3. **`.claude/rules/pre-phase-harness-audit.md`** — the harness audit rule (R5: enumerate ALL probes); plan adapts the OS-aware wrapper guidance to Linux + the Junior daemon.
4. **`.claude/rules/pm-plugin-hooks-stable.md`** — load-bearing hook names Probe 7 checks.
5. **`scripts/brehon/migrate-roundtrip.sh`** (current stub) — read the existing 32-58 stub body + the header doc-comment + `origin/governance-v0` fetch-verify guard at lines 33-44 you must preserve.
6. **`crates/diesel_utils/src/main.rs`** (or equivalent) — verify the actual `lemmy_diesel_utils` binary sub-command surface BEFORE committing the script. Plan §13 GOTCHA at lines 1394-1401: if `revert` sub-command does not exist, the script should fall back to `redo` or replay-from-clean-container per the current diesel_utils CLI. **If the CLI surface is unclear, file a DQ pending entry rather than guess.**
7. **Lessons** (Glob `.claude/lessons/`, read any with keywords matching `harness`, `audit`, `probe`, `submodule`, `migration`, `roundtrip`, `diesel`):
   - `feedback_pipes_mask_exit_codes.md` — never pipe output through grep/head; capture to file + check `$?` directly.
   - `feedback_worktree_submodules_not_auto_init.md` — Probe -1 catches this on Linux/Junior worktrees.
   - `feedback_lemmy_migration_runner.md` — `forbid_diesel_cli` upstream trigger; only `cargo run -p lemmy_diesel_utils --features full -- ...` works because that path takes the `pg_advisory_lock(0)` per `crates/diesel_utils/src/schema_setup/mod.rs:214`.
   - `feedback_pre_phase_dod_smoke_test.md` — DoD smoke informs probe shapes (this brief inherits the rule's mandatory structure).

## 3a. Handover from prior cohort

(none — first task of phase-v1-SL-a, branch tip `ea322cd0a` cut by Junior task #82)

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branched off `phase-v1-SL-a` (tip `ea322cd0a` per `git log -1 --oneline phase-v1-SL-a`).
- `git branch --show-current` must return a `junior/role-impl-task-...` branch (NOT `phase-v1-SL-a` or `governance-v0` directly). Probe 1 in plan §13 reads `phase-v1-SL-a`; **adapt to your actual worktree branch name and confirm the worktree is downstream of `phase-v1-SL-a` HEAD via `git merge-base --is-ancestor phase-v1-SL-a HEAD || git log --oneline phase-v1-SL-a..HEAD`**.
- One commit at task end: the migrate-roundtrip.sh stub replacement. Junior's daemon finalize-merges into `phase-v1-SL-a`.

### Probe discipline

- Run ALL probes in the plan §13 order (Probe 0, -1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10).
- Capture probe output to `/tmp/sl-a-task0-probe-N.log` (writable temp on the EliteDesk Junior worktree).
- Check `$?` after each probe separately — do not pipe through `head`/`grep`. Per `feedback_pipes_mask_exit_codes.md`.
- On any Probe 0, 1, 2, 3, 7, or 9 FAIL: STOP immediately. Write a DQ pending entry (`from: "impl"`, `kind: "blocker"`) with the probe output + exit code. Commit + push the DQ entry to your worktree branch immediately per `.claude/rules/decision-queue.md` "Mid-task visibility".
- On Probe 6 (DQ advisory): list pending entries in your task output; advisor handles resolution. **Expect 0 pending — the SL-a planning DQ #116 was resolved before bm-cut.**
- On Probe 8 (concurrent-PR check): if output is non-empty, report to advisor via task output — do not STOP; advisor decides.
- On Probe 4 (ADR-013 match-site enumeration): if the count differs from the plan §10.5 expected 6 sites, file a DQ pending entry naming the new site(s) + recommend including in Task 5 — **do not silently expand Task 5 scope**.
- On Probe 10 (stub-fix status): if the stub is **already fixed** (someone got there first), STOP and file a DQ — do not double-edit.

### Stub-replacement discipline

- The stub body to replace is `scripts/brehon/migrate-roundtrip.sh` lines 32-58 (the existing `migrate-roundtrip.sh: no migrations/ change detected vs governance-v0; stub exiting 0` block + whatever follows). **Preserve** the header doc-comment + the `origin/governance-v0` fetch-verify guard at lines 33-44 of the existing file.
- The replacement body is the bash block at plan §13 Task 0 lines 1345-1392. Copy it verbatim. **Do not modify the cargo invocations** — they're written per `feedback_lemmy_migration_runner.md` exactly.
- **Validate `lemmy_diesel_utils revert` sub-command exists BEFORE committing**: `grep -E 'revert|run|redo' crates/diesel_utils/src/main.rs` (or wherever the binary `clap` parser lives — find it). If `revert` is absent and `redo` is present, adapt per plan §13 GOTCHA. If unclear, **file a DQ rather than guess** — the gotcha is a known unknown, not a license to improvise.
- After replacement, `bash -n scripts/brehon/migrate-roundtrip.sh` must exit 0 (syntax check). **Do not actually execute the script** — it spins up a Postgres container and runs cargo, which is workflow work, not impl-task work.

### DQ attribution

- No `answered_by: "advisor"` or `"user"` from this subagent.
- Self-resolve only as `"impl-self-resolved"`.
- Any pending DQ entry written from this task uses `from: "impl"`.
- Mid-task push required immediately after any DQ write per `.claude/rules/decision-queue.md`.

### Commit shape

- ONE commit only.
- Subject: `chore(scripts): replace migrate-roundtrip.sh stub with real round-trip logic (DQ #114; v1-SL-a task 0)` (verbatim per plan §13 line 1413).
- Body: include the `HANDOVER:` YAML trailer per `.claude/agents/impl-task.md` "Per-task commit shape" — this is the first task of a non-`[P]` cohort, but the trailer still aggregates downstream (the prior-cohort-handover aggregation step in cohort dispatch reads it).
- Files in commit: ONLY `scripts/brehon/migrate-roundtrip.sh`. Junior's auto-finalize will absorb the worktree branch into `phase-v1-SL-a`.

### No cargo locally

- This Task 0 does NOT run cargo. Plan is Shape G — workspace check + migration-roundtrip workflow + e2e all run on GH-hosted runners after Task 1's push.
- Do NOT raise a `kind: "validate-pending"` DQ entry from Task 0 — there's no cargo to validate. The first `validate-pending` entry of SL-a comes from Task 1 (migration directory creation) onward.

## 5. Probes to run (per plan §13 Task 0)

The probe set is verbatim from plan §13 lines 1264-1339. Run them in the order below; capture each to `/tmp/sl-a-task0-probe-N.log` per discipline above.

```bash
# Probe 0 — Docker daemon
docker ps > /tmp/sl-a-task0-probe-0.log 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe -1 — submodule init (Linux/Junior worktree quirk)
git submodule status > /tmp/sl-a-task0-submodule.log 2>&1
if grep -q '^-' /tmp/sl-a-task0-submodule.log; then
  git submodule update --init --recursive > /tmp/sl-a-task0-submodule-init.log 2>&1
  echo "submodule init exit: $?"
fi

# Probe 1 — branch verification (worker branch derived from phase-v1-SL-a)
echo "current: $(git branch --show-current)"
git merge-base --is-ancestor phase-v1-SL-a HEAD && echo "PASS: ancestor of phase-v1-SL-a" || echo "FAIL: not on phase-v1-SL-a lineage"

# Probe 2 — governance-v0 baseline counts
echo "ENTRY_KIND_ count (expect 33):"
grep -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs

echo "EXPECTED_SEED_COUNT total (expect 88 = 34 v0 + 27 AD + 27 JM):"
grep -nE '^pub const EXPECTED_SEED_COUNT' crates/api/api/src/governance/config.rs

echo "CaseStatus variant count (expect 9):"
sed -n '/^pub enum CaseStatus/,/^}/p' crates/db_schema_file/src/enums.rs | grep -cE '^\s*[A-Z][a-zA-Z]*,?$|#\[default\]'

# Probe 3 — schema baseline: moderation_case has no SL-a columns
grep -E 'grace_expires_at|liability_escape_reason' crates/db_schema_file/src/schema.rs > /tmp/sl-a-task0-probe-3.log 2>&1 && {
  echo "ERROR: SL-a columns already in schema.rs — branch contamination"; cat /tmp/sl-a-task0-probe-3.log; exit 1;
} || echo "schema.rs clean of SL-a columns"

# Probe 4 — ADR-013 match-site enumeration (live grep for §10.5 site list)
grep -rnE 'match\s+\w+\.status\s*\{' crates/api crates/api_crud --include='*.rs' > /tmp/sl-a-task0-probe-4.log 2>&1
echo "match sites found:"
cat /tmp/sl-a-task0-probe-4.log
# EXPECT: 6 sites matching plan §10.5. If new sites appeared since brief-write, file a DQ.

# Probe 5 — Postgres-side: case_status enum has 9 values
echo "case_status enum values (expect 9 currently):"
grep -B1 -A20 "CREATE TYPE case_status" migrations/2026-04-15-100000-0000_add_governance_enums/up.sql 2>/dev/null | head -15

# Probe 6 — DQ pending status (advisory)
python3 -c "import json; d=json.load(open('.claude/decision-queue.json', encoding='utf-8')); print('pending:', [(e['id'], e.get('kind')) for e in d.get('pending',[])])"

# Probe 7 — PM-plugin-hooks-stable check (REQUIRED to pass)
for h in \
  local_private_message_before_create \
  local_private_message_after_create \
  local_private_message_before_update \
  local_private_message_after_update \
  federated_private_message_before_receive \
  federated_private_message_after_receive; do
  grep -rqE "\"$h\"" crates/ || { echo "missing hook literal: $h"; exit 1; }
done
echo "PM hooks: all 6 present"

# Probe 8 — concurrent-PR check (advisory)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("moderation_case\\.rs|enums\\.rs|governance_log\\.rs|config\\.rs|migrations/2026-05-03")) | {number, title, headRefName}'
# EXPECT: empty output

# Probe 9 — Shape G workflow YAMLs lint-clean + carry the right flags
yamllint .github/workflows/cargo-validate-workspace.yml \
         .github/workflows/cargo-validate-migration.yml \
         .github/workflows/cargo-test-e2e.yml > /tmp/sl-a-task0-yamllint.log 2>&1
echo "yamllint exit: $?"
grep -E '\-\-features full|\-\-no-deps|\-D warnings' .github/workflows/cargo-validate-workspace.yml > /tmp/sl-a-task0-probe-9-flags.log 2>&1
cat /tmp/sl-a-task0-probe-9-flags.log

# Probe 10 — migrate-roundtrip.sh stub status (per DQ #114)
grep -q "migrate-roundtrip.sh: no migrations/ change detected vs governance-v0; stub exiting 0" \
  scripts/brehon/migrate-roundtrip.sh && echo "STUB STILL PRESENT — Task 0 must replace" || { echo "STUB ALREADY FIXED — verify via dry-run before continuing; if unexpected, file DQ"; exit 1; }
```

## 6. Stub replacement (post-probes)

After Probes 0-10 pass:

1. **Verify `lemmy_diesel_utils revert` sub-command exists** (gotcha-check):
   ```bash
   grep -rE 'revert|run|redo' crates/diesel_utils/src/ --include='*.rs' | head
   ```
   If `revert` sub-command is present per the binary's clap parser → use as-written. If absent and `redo` present → adapt per plan §13 GOTCHA (lines 1394-1401). If unclear → STOP and file DQ.

2. **Replace the stub body** (preserve lines 1-31 = doc-comment + fetch-verify guard; replace lines 32-58 = stub body):
   - Source for replacement: plan §13 Task 0 lines 1345-1392 (the bash block).
   - Use Edit (not Write) — preserve the file's existing header. Surgical replacement of the stub block only.

3. **Syntax check**: `bash -n scripts/brehon/migrate-roundtrip.sh` MUST exit 0. **Do not execute the script** itself — that's workflow-side work.

4. **Commit** with the verbatim subject from plan §13 line 1413; body includes `HANDOVER:` YAML trailer naming `filesCreated: []`, `filesModified: ["scripts/brehon/migrate-roundtrip.sh"]`, `keyDecisions: ["replaced stub per DQ #114"]`.

## 7. Expected output (return to advisor)

```
## Task 0 complete — v1-SL-a pre-flight audit + migrate-roundtrip.sh stub fix

**Branch:** junior/role-impl-task-...-N (descended from phase-v1-SL-a @ ea322cd0a)
**Probe results:**
  - Probe 0: DOCKER OK (or: FAIL — STOP)
  - Probe -1: submodule OK (or: init ran, exit N)
  - Probe 1: PASS (ancestor of phase-v1-SL-a)
  - Probe 2: ENTRY_KIND=33, EXPECTED_SEED_COUNT lines, CaseStatus=9 (PASS)
  - Probe 3: schema.rs clean of SL-a columns (PASS)
  - Probe 4: <N> match sites — list (expect 6 per §10.5)
  - Probe 5: case_status enum has 9 values (PASS)
  - Probe 6: pending DQ entries: <list or empty> — expected 0
  - Probe 7: PM hooks all 6 present (PASS)
  - Probe 8: concurrent-PR check: <empty or list — advisory>
  - Probe 9: yamllint exit 0; flag check — list (PASS)
  - Probe 10: STUB STILL PRESENT (pre-replacement)
**lemmy_diesel_utils revert sub-command:** <present | absent — used <fallback>>
**Stub replacement:** scripts/brehon/migrate-roundtrip.sh updated; bash -n PASS
**Commit:** <sha> chore(scripts): replace migrate-roundtrip.sh stub with real round-trip logic (DQ #114; v1-SL-a task 0)
**Next:** advisor queues Cohort A (Tasks 1+2+3 [P] — migration + enums + schema.rs)
```

If any blocking probe failed, replace the above with a DQ catch-fire entry (commit + push to worktree branch immediately).
