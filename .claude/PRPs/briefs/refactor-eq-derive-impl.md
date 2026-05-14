---
phase: chore/refactor-eq-derive
role: impl-task
task: refactor
brief_n: 1
authored: 2026-05-14
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_finding: 3.C.1 (rank 11)
parent_phase_tip: <set by bm-cut — branch tip is governance-v0 HEAD at bm-cut time>
---

# [role:impl-task] chore/refactor-eq-derive — add Eq derive to GovernanceConfig — see .claude/PRPs/briefs/refactor-eq-derive-impl.md

## §1 Role + dispatch

`[role:impl-task] chore/refactor-eq-derive — add Eq derive to GovernanceConfig source model`

## §2 Scope

### §2.1 Driving audit finding

Audit §3.C.1 (rank 11, severity MAJ, effort XS, frequency 1):

> `governance_config.rs:10` · Lens 1+2 · Axis-4 quality-fail · **[MAJ]** `#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]` — missing `Eq`. All 20 other governance source models derive `PartialEq, Eq` together. Lone outlier breaks `HashSet<GovernanceConfig>` / `BTreeMap` key bounds · Lens 1: 100% parity violation with sibling files in the same directory; Lens 2: idiomatic Rust collection trait bound · add `Eq` to derive: `#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]` · **XS** · 1 file, one-line fix.

### §2.2 Pre-flight verification

```bash
# Verify the current shape is exactly what the audit reported
grep -n "^#\[derive" crates/db_schema/src/source/governance/governance_config.rs | head -3
```

Expected output line 10: `#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]` (no `Eq`).

If the line has already drifted (e.g. someone added Eq), STOP and file a DQ blocker — the refactor may already be done.

### §2.3 The edit

**File:** `crates/db_schema/src/source/governance/governance_config.rs`

**Line 10 — change from:**

```rust
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
```

**to:**

```rust
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
```

That is the only edit. No other file changes. No new tests required (existing derive-based tests, if any, will pick up Eq via `cargo check`).

### §2.4 Post-edit verification

```bash
# Confirm Eq was added
grep -n "^#\[derive(PartialEq, Eq" crates/db_schema/src/source/governance/governance_config.rs | head -1
# Expected: line 10 hit

# Spot-check sibling models still have Eq (no accidental regression)
grep -l "PartialEq, Eq" crates/db_schema/src/source/governance/*.rs | wc -l
# Expected: 21 (was 20 before this fix; governance_config now joins the cohort)
```

### §2.5 Validation gate (Shape G)

Push the worker branch. The push triggers `cargo-validate-workspace.yml` (path filter `crates/**` matches).

Capture `workflow_run_id` via `gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --json databaseId`.

Raise a `kind: "validate-pending"` DQ entry per `decision-queue.md` Recipe 1. **Atomic raise** per `feedback_dq_raise_before_ci_watcher_queue.md`: commit + push the DQ entry BEFORE the lane-dedicated session dispatches a ci-watcher.

## §3 Required reading

- `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md` §3.C.1 — driving finding + §5.1 internal-inconsistency framing
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md` — PR-3 of 6 context
- `.claude/agents/impl-task.md` — subagent contract (one task, one commit, one feature)
- `.claude/rules/decision-queue.md` Recipe 1 — DQ entry shape for validate-pending
- `.claude/rules/advisor-orchestrator.md` §5.3 §G4 classifier — Shape-G two-phase validation

## §4 Constraints

- **Files:** ONLY `crates/db_schema/src/source/governance/governance_config.rs`. NO other files.
- **Edits:** exactly 1 line edited (the derive attribute). NO additional deletes, NO reorders, NO new fields, NO documentation changes.
- **Branch:** `chore/refactor-eq-derive` (cut from `governance-v0` by sibling bm-cut brief).
- **Pre-push cargo-check (mandatory per `feedback_fix_impl_pre_push_cargo_check.md`):** before `git push origin chore/refactor-eq-derive`, run:

  ```bash
  bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/refactor-eq-derive-precheck.log 2>&1
  status=$?
  tail -20 .claude/PRPs/debug/refactor-eq-derive-precheck.log
  [ $status -eq 0 ] || exit $status
  ```

  Per `.claude/rules/cargo-output-capture.md` — capture-then-tail-separately pattern; never pipe cargo through tail/head. If non-zero exit: this is a one-line `derive` change that should never break compile — STOP and file a DQ blocker.

- **Shape G:** after pre-push cargo-check passes, push the worker branch; capture workflow_run_id; write `kind: "validate-pending"` DQ entry per Recipe 1.
- **DQ atomic raise:** commit + push DQ entry BEFORE lane session dispatches ci-watcher (per L3 from RT-r1 halt retro).
- **Encoding:** Python `json.dump(..., ensure_ascii=False, indent=2)` when writing the DQ entry.
- **next_id calculation:** span both `.claude/decision-queue.json` AND `.claude/decision-queue-archive-*.json` per decision-queue.md Hard refusal #2.
- **COMMIT MESSAGE:** `chore(refactor): add Eq derive to GovernanceConfig (audit 3.C.1)`

## §5 Out of scope

- All 19 other audit findings — each ships in its own refactor PR per execution plan.
- Adding `Eq` to non-Brehon files even if they're outliers (this audit scope was Brehon-authored only).
- Renaming or reordering any other derive attribute.
- Adding Hash derive (audit finding only required Eq; Hash is a separate concern).
- Touching `mod.rs`, `schema.rs`, or any other governance file.

## §6 HANDOVER trailer

Per `feedback_handover_trailer_cohort_propagation.md`: this refactor has no cohort siblings on the same worker branch (PRs 3, 4, 5, 6 run in parallel but each on its own branch). Trailer is optional; if added:

```yaml
HANDOVER:
  filesCreated: []
  filesModified:
    - crates/db_schema/src/source/governance/governance_config.rs
  keyDecisions:
    - Added Eq to PartialEq derive on GovernanceConfig per audit §3.C.1; aligns with sibling parity convention.
  notes: Audit finding tier was "fix-before-next-phase" (rank 11). One-line change, lowest-effort refactor in tier.
```
