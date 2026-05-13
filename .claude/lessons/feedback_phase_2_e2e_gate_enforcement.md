---
name: Phase 2 e2e gate enforcement at bm-pr
description: When a phase plan names an e2e task (Task N touches crates/server/tests/e2e.rs), bm-pr MUST refuse if no `kind: validate-pending` entry with `result: pass` exists for Phase 2 e2e. RT-r1 + SL-e both shipped without it.
type: feedback
---

# Pre-bm-pr Phase 2 e2e gate enforcement

Phase 2 e2e (testcontainers + Postgres + governance test suite) is
**explicitly advisor-driven** per 2026-04-28 minutes-budget audit
(`cargo-test-e2e.yml` is `workflow_dispatch`-only since then). User
gate 4 in `.claude/rules/advisor-orchestrator.md` §3.2 selects
between (a) local laptop run + advisor mutates `validate-pending` DQ
directly, or (b) `gh workflow run cargo-test-e2e.yml` + ci-watcher
polls. **Neither happens automatically** — the advisor must
proactively trigger Phase 2 e2e after the phase tip lands its last
e2e-touching commit.

The gap is procedural enforcement. RT-r1 (shipped 2026-05-12 via
PR #126) and SL-e (shipped 2026-05-13 via PR #127) both reached
bm-pr + bm-merge without Phase 2 e2e being run. RT-r1 added 163
lines to `crates/server/tests/e2e.rs` (Task 10 extension +
fix-impl-2 padding); SL-e added a whole `mod v1_sl_e_fixtures` test
module. None of this test surface was executed; only `cargo test
--no-run` (compile-only) ran via the workspace-check workflow.

**Why this lesson exists:** Per `.claude/PRPs/reports/v1-RT-r1-retro.md`
§3 Lesson 3 + §5 Watch-item 3. Two consecutive phases shipped with
Phase 2 e2e skipped. The wiring is correct — manual dispatch is
intentional carve-out — but the human-side discipline keeps slipping.
Encoding a bm-pr gate fixes the procedural gap.

**Verification of the wiring:**

```bash
gh run list --repo barrie-cork/lemmy --workflow cargo-test-e2e.yml \
  --limit 10 --json databaseId,headBranch,event,conclusion,createdAt
```

Result (2026-05-13): 10/10 most recent runs are `event=workflow_dispatch`.
Last `event=push` run was 2026-04-28 (JM-d, pre-audit). Confirmed —
no auto-trigger. RT-r1 + SL-e show zero runs of any kind.

**How to enforce — plan-aware bm-pr gate:**

The bm-pr operational script reads the plan file at
`.claude/PRPs/plans/<phase>.plan.md`; if the plan body mentions
`crates/server/tests/e2e.rs` (or names an "e2e" task in §13), bm-pr
refuses until a `kind: "validate-pending"` (or `*-laptop-e2e` /
`*-laptop`) DQ entry exists in `resolved[]` with:

- `branch: phase-<phase>` or a worker branch off it
- `phase_task` referencing an e2e task OR
- `local_log_path` referencing the local laptop e2e log
- `result: "pass"`

Skip the gate when the plan does NOT touch e2e.rs (rare for v1 lane
work — most phases extend the migration round-trip or add fixtures).

```bash
# Pre-condition (advisor inline, before authoring bm-pr brief)
PHASE_SLUG="<derive from current branch>"
PLAN_FILE=".claude/PRPs/plans/${PHASE_SLUG}.plan.md"

# Detect whether the plan touches e2e
E2E_PLAN=0
if [ -f "$PLAN_FILE" ] && grep -qE "crates/server/tests/e2e\.rs|cargo-test-e2e\.yml|phase1_migrations_round_trip|e2e (test|suite|task)" "$PLAN_FILE"; then
  E2E_PLAN=1
fi

if [ "$E2E_PLAN" = "1" ]; then
  # Scan DQ resolved[] for a passing e2e validate-pending entry on this phase
  python <<PYEOF
import json, io, sys
d = json.load(io.open('.claude/decision-queue.json', encoding='utf-8'))
phase = "phase-${PHASE_SLUG}"
ok = False
for e in d.get('resolved', []):
    if e.get('kind') in ('validate-pending', 'validate-pending-laptop', 'validate-pending-laptop-e2e') \
       and e.get('result') == 'pass' \
       and (
           (e.get('branch') or '').startswith(phase)
           or 'e2e' in (e.get('phase_task') or '').lower()
           or 'e2e' in (e.get('local_log_path') or '').lower()
       ):
        ok = True
        break
if not ok:
    print(f"STOP: Phase 2 e2e gate — no `kind: validate-pending` with `result: pass` for {phase} e2e found in resolved[]")
    print("User gate 4 (Phase 2 e2e — local vs dispatch) must run before bm-pr.")
    print("Options: (a) local cargo test (~26 min, zero billed via scripts/brehon/cargo-test.bat),")
    print("         (b) `gh workflow run cargo-test-e2e.yml --repo barrie-cork/lemmy --ref $phase`.")
    sys.exit(1)
PYEOF
else
  echo "INFO: plan does not touch crates/server/tests/e2e.rs — Phase 2 e2e gate skipped."
fi
```

**Detection heuristic for "plan touches e2e":**

- Literal path mention: `crates/server/tests/e2e.rs`
- Workflow mention: `cargo-test-e2e.yml`
- Test fn name: `phase1_migrations_round_trip` (the canonical extension target)
- Phrase: "e2e test" / "e2e suite" / "e2e task"

If ANY match, the gate fires. False positives (plan mentions e2e in
prose without changing e2e.rs) are tolerable — the user can dispatch
e2e cheaply (local laptop, ~26 min, zero billed) to clear the gate
even if the run is redundant.

False negatives (plan changes e2e.rs but doesn't mention it in body
prose) are rare — every Lemmy v1 plan since JM-a has used the canonical
"Task N: extend phase1_migrations_round_trip" pattern.

**Edge cases:**

- **Plan deliberately omits e2e** (e.g. pure rules/lessons phase): no
  match → gate silent → correct.
- **Multiple e2e validate-pending entries** (e2e ran multiple times,
  some pass + some fail): the gate finds the most recent pass and
  accepts. Add ordering by `resolved_at` if recency matters; current
  implementation accepts any pass.
- **e2e ran on a worker branch but not on phase tip:** ambiguous. The
  worker-branch tip may not equal the phase tip if subsequent
  fix-impl commits landed. Mitigation: gate compares e2e run's
  branch SHA against phase tip; if older than the latest e2e-
  touching commit on phase, gate stays closed. Future work — current
  implementation accepts any branch match.
- **Phase 2 e2e ran on the LOCAL laptop** (entry kind `validate-pending-
  laptop-e2e`): gate accepts. Both run targets satisfy user gate 4.
- **Retroactive enforcement** for already-shipped phases (RT-r1 +
  SL-e): impossible — phases shipped. The gate prevents recurrence;
  it cannot retroactively re-run e2e for shipped phases. Audit-trail
  note in retro is the historical fix.

**Companion lessons:**

- `feedback_phase_retro_gate_enforcement.md` — same pattern, different
  gate (retro file existence + mtime).
- `feedback_e2e_local_or_dispatch_user_choice.md` — user gate 4
  framing.
- `feedback_default_local_testing.md` — local-vs-dispatch preference.
- `feedback_windows_e2e_requires_bat_wrapper.md` — local laptop
  invocation discipline.

**Where codified:**

- `.claude/commands/bm/bm-pr.md` Phase 1 pre-conditions — add new
  row + inline gate script.
- This lesson file.
