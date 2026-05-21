# brehon-conformance-audit — planning brief 2 (Task 8 + Task 9 mechanism revision)

## 1. Role + dispatch line

`[role:planning] brehon-conformance-audit plan-revise — Task 8 + Task 9 mechanism (federation-only enforcement) — see .claude/PRPs/briefs/brehon-conformance-audit-planning-2-revise.md`

## 2. Scope

Author an **in-place revision** to `.claude/PRPs/plans/brehon-conformance-audit.plan.md` that fixes the §10.8 + §13 Task 8 + §13 Task 9 mechanism mismatch surfaced by DQ #303 / #307 / #309 / #310 (three fix-impl cycles + user catch-fire 2026-05-21). The revision is targeted — touch only §10.8 + §13 Task 8 + §13 Task 9 + §4 (watchpoints if affected) + §5.1 (complexity score if task count changes) — and adds NEW remediation tasks to walk back the broken Task 8 and re-land it correctly. Tasks 6, 7, 9 stay structurally unchanged; Task 8 is rewritten + a new Task 8a "revert + re-land" precedes it; fix-impl-1 + fix-impl-3 commits stay as net-positive cleanups.

### 2.1 The plan-side defect

Plan §10.8 specifies:

```yaml
clippy.toml at repo root:
  disallowed-methods = [
    { path = "core::option::Option::unwrap_or_default", reason = "..." },
    { path = "core::result::Result::unwrap_or_default", reason = "..." }
  ]
```

Plan §13 Task 9 specifies: `#![deny(clippy::disallowed_methods)]` on 3 federation `mod.rs` files (`crates/apub/activities/src/governance/mod.rs`, `crates/api/api/src/governance/mod.rs`, `crates/db_schema/src/source/governance/mod.rs`).

Plan §10.8 comment claimed: *"non-federation code is unaffected"*. **Wrong.** The workspace `Cargo.toml [workspace.lints.clippy] style = { level = "deny", priority = -1 }` (line 100) escalates the `clippy::style` group (which contains `disallowed_methods`) to deny by default for the WHOLE workspace. Result: dropping `clippy.toml` activates the lint workspace-wide; the workspace immediately fails on 100+ pre-existing `unwrap_or_default` callsites across `db_schema`, `db_views`, `api_*`, `apub`, `routes`, `email`, `server/tests`, `utils`, `diesel_utils`, etc.

### 2.2 The correct mechanism (per rustc lint-precedence docs)

Per rustc's `src/doc/rustc/src/lints/levels.md` "Priority of lint level sources" rule 4: *"Within the source, attributes at a lower-level in the syntax tree take precedence over attributes at a higher level."*

Example from the docs: workspace-level `#![deny(unused_variables)]` + module-level `#[allow(unused_variables)]` → **allow wins** (lower syntax tree).

**Reverse direction (our case):** workspace-level `disallowed_methods = "allow"` + module-level `#![deny(clippy::disallowed_methods)]` → **deny wins** for items inside that module subtree. Federation-only enforcement achieved.

Required workspace edit (1 line):

```toml
[workspace.lints.clippy]
# ... existing entries ...
disallowed_methods = "allow"   # NEW: silence workspace-wide; per-module #![deny] re-enables for federation modules
```

The `clippy.toml` (defining WHICH methods are disallowed) stays workspace-wide as configuration. The LEVEL (`allow`/`deny`) is what's per-scope.

### 2.3 Required deliverables

- **NEW Task 8a**: revert `clippy.toml` commit `c3aaba47f` (or note it as "wipe-and-replace" — planner picks). Pre-Task-8.
- **REVISED Task 8**: SINGLE commit creating BOTH `clippy.toml` (same content as before) AND modifying root `Cargo.toml [workspace.lints.clippy]` to add `disallowed_methods = "allow"`. FILES YAML:
  ```yaml
  creates: ["clippy.toml"]
  modifies: ["Cargo.toml"]
  requires: ["8a"]
  ```
  §15 success gate = NARROW probe target = the four governance roots (apub_activities governance, api_api governance, db_schema_source governance). Probe command: `cargo clippy -p lemmy_apub_activities -p lemmy_api -p lemmy_db_schema --features full --no-deps -- -D warnings`. EXPECT exit 0 today (workspace-allow silences the activation; without Task 9's per-module deny, federation modules also pass).
- **Task 9 (UNCHANGED structurally)**: `#![deny(clippy::disallowed_methods)]` added to the 3 federation `mod.rs` files (per PRECON-4). §15 success gate = SAME narrow probe but now the federation modules ARE enforced (deny wins lower in syntax tree). EXPECT exit 0 (federation code in governance modules should already be axis-4 clean per fed-in-b fix-impl-3 evidence).
- **§4 Watchpoint #4 (probe ordering)**: stays applicable; planner verifies the wording still matches new Task 8 + Task 9 sequence.
- **§5.1 complexity score**: recompute if task count changed (Task 8a added → score +1 in `count(impl_tasks)`).
- **§13 Task 13 (retro)**: add bullet to "Lessons surfaced this sub-phase" enumerating the §10.8 mechanism misanalysis + cycle-3 catch-fire.

### 2.4 What NOT to touch

- Tasks 1-7 + 10-12 stay unchanged. Their commits already landed (fix-impl-1 cleanup + clippy.toml + 5 cohort-2 entries pass + DQ #303 resolved). Six-axis spine intact.
- fix-impl-1 (`b00be611a` — 6 lemmy_utils sites) + fix-impl-3 (`34f5cc567` — 4 lemmy_diesel_utils sites) commits stay. These are net-positive cleanups (axis-4 idiom: explicit fallbacks > silent `unwrap_or_default`). The corrected mechanism doesn't NEED them, but reverting them adds work + creates a different upstream-merge surface. Net-zero.
- DQ #308 (Task 8 workspace fail) stays in `pending[]` per §G4 fail-handling. Historical evidence of the cycle that drove this re-plan. Do NOT mutate.
- ADR-006/013/014/015 — none affected by the mechanism revision.
- PRECON-1..9 — all stay binding. PRECON-4 ("3 federation module roots") matches the corrected mechanism.

## 3. Required reading

### 3.0 P0 — defect evidence

1. DQ #303, #307, #309, #310 from `.claude/decision-queue.json` resolved[] — full chronology of the three cycles.
2. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §10.8 + §13 Task 8 + §13 Task 9 lines verbatim.
3. `.claude/PRPs/briefs/brehon-conformance-audit-fix-impl-1.md` + `-fix-impl-3.md` — the 10 sites already remediated (net-positive cleanups).
4. Root `Cargo.toml` lines 84-122 — the `[workspace.lints.clippy]` block (the missing `disallowed_methods = "allow"` line is THE plan-side defect).
5. `clippy.toml` (on phase tip, line 1-9) — currently committed; stays semantically (revised Task 8 re-creates with same content).

### 3.1 Authority

6. `.claude/PRPs/briefs/brehon-conformance-audit-planning-1.md` — original planning brief. §0.1 PRECONs stay binding. PRECON-4 ("3 federation module roots") IS the goal; the revision aligns the mechanism with that goal.
7. `.claude/PRPs/reports/brehon-conformance-audit-planning-guidance-2026-05-20.md` — prior advisory input, supplemented by the user clarifying conversation 2026-05-21 (this brief).

### 3.2 Mechanism-side reading

8. **rustc docs lint-precedence**: `https://github.com/rust-lang/rust/blob/main/src/doc/rustc/src/lints/levels.md#priority-of-lint-level-sources` (rule 4: lower-syntax-tree attributes win).
9. **clippy docs disallowed_methods**: configurable lint; `clippy.toml` defines WHICH methods; lint level is separate `[lints]` config.

### 3.3 Rules

10. `.claude/rules/branch-manager.md` — file ownership boundaries (advisor authors briefs; planner authors plans + .plan.md edits; impl edits `crates/**`).
11. `.claude/rules/decision-queue.md` — DQ #311 (this brief) is the trigger; on completion, planner self-resolves with `answered_by: "planner"`.
12. `.claude/PRPs/templates/plan.template.md` — §13 task shape (FILES YAML + `requires:` arrays + acceptance criteria).

## 4. Constraints

1. **Edit-only revision** — touch ONLY §10.8 + §13 Tasks 8a/8/9 + §4 (if Watchpoint #4 needs wording update) + §5.1 (recompute score) + §13 Task 13 (lessons bullet). Do NOT rewrite untouched sections.
2. **PRECONs binding** — all 9 PRECONs stay in force. PRECON-4 (3 federation module roots) is now mechanically backed; document the precondition match in §10.8.
3. **No new axes** — PRECON-9 stays (6 fixed axes; no axis #7).
4. **Cohort markers** — Task 8a is SERIAL (precedes Task 8; revert ordering matters). Task 8 stays in Cohort 2 OR moves to its own cohort depending on requires:; planner decides. Task 9 stays Cohort 4 SERIAL.
5. **Branch context** — base_branch=phase-brehon-conformance-audit (currently at `c78a39cb7` post-fix-impl-3). Planner forks to its own `junior/...` branch.
6. **Single commit** — plan revision is one commit on planner branch with subject `docs(plan): brehon-conformance-audit revise §10.8 + Task 8/8a/9 mechanism (DQ #311)`.
7. **DQ #311 self-resolve** — at planner-task completion, planner mutates DQ #311 to `answered_by: "planner"`, `answer` = "plan revision committed in <sha>; revised plan at <path>; new task count = N".
8. **No code edits** — planner authors plan only. Never touches `crates/**`, `Cargo.toml`, `clippy.toml`, mod.rs files. The plan REFERENCES the edits; Tasks 8a/8/9 implement.
9. **§15 DoD shapes** — each revised task has its own §15 validate-pending-laptop DoD shape per `.claude/rules/advisor-orchestrator.md` §5.2. Use cargo-check + narrow-probe pattern.
10. **§3.5 watchpoint specificity** — any new watchpoint cites a specific file/line.
11. **§3.6 canonical-schema-first** — if new section structure is needed, mirror an existing plan (v1-federation-inbound-a.plan.md is the canonical reference).
12. **§3.8 schema-changing-spec retrofit gate** — if revision changes the shape of an artifact class (e.g. new §15 sub-shape), ask user before committing. This is a TARGETED revision; not a schema change.

## 5. Validation gate (planner-side DoD)

### 5.1 Pre-commit dry-run

```bash
# Confirm the plan still parses + the new task FILES YAML schema validates
python3 -c "
import re
plan = open('.claude/PRPs/plans/brehon-conformance-audit.plan.md').read()
assert '## 10.8' in plan, '§10.8 still present'
assert 'Task 8a' in plan, 'Task 8a added'
assert 'disallowed_methods = \"allow\"' in plan, 'workspace-allow step documented'
assert '#![deny(clippy::disallowed_methods)]' in plan, 'per-module deny step documented'
print('OK')
"
```

### 5.2 §5.1 complexity score recompute

After the revision, planner re-runs the §5.1 complexity score with new `count(impl_tasks)` (was 12, now 13 with Task 8a). Score formula per `feedback_complexity_score_pre_split.md`. If new score crosses the >8 threshold, planner files a `kind: "log"` DQ noting the split-or-proceed analysis (likely proceed: revision is mechanical).

### 5.3 DQ #311 self-resolve

Planner mutates DQ #311 to `answered_by: "planner"`, `resolved_at: <ts>`, `answer: "Plan revised. §10.8 mechanism corrected (workspace `disallowed_methods = allow` + per-module `#![deny]`). Task 8a added (revert clippy.toml). Task 8 rewritten (single commit creates clippy.toml + edits Cargo.toml). Task 9 unchanged structurally; success gate clarified. fix-impl-1 + fix-impl-3 commits stay as net-positive cleanups (not load-bearing for the corrected mechanism)."`. Commit + push.

## 6. KNOWN harness limitations

1. **CC v2.1.119 sensitive-file gate** — `.claude/PRPs/plans/**.md` should NOT be on sensitive list (plan files are planner-write).
2. **Daemon finalize-merge no-push** — advisor handles SSH-push.
3. **Concurrent edit risk** — advisor session may be polling/mutating DQ in parallel. Planner reads + writes DQ #311 last, atomic per `.claude/rules/multi-lane-worktree.md` hard refusal #6.

## 7. Next steps after this task

- Daemon finalize-merges plan revision onto phase branch.
- Advisor pulls, reads revised §13 Tasks 8a + 8 + 9.
- Advisor dispatches Task 8a (revert clippy.toml).
- Advisor dispatches Task 8 (re-create clippy.toml + Cargo.toml workspace-allow).
- Advisor dispatches Task 9 (per-module deny on 3 mod.rs).
- Cohort 2.5 (Task 6 compute-metrics.sh) can then run; Cohort 3 (Task 7 dogfood) follows.
- Plan §10.8 corrected wording becomes a Task 13 retro lesson candidate (mechanism-vs-intent gap).

## 8. Commit subject template (verbatim)

```
docs(plan): brehon-conformance-audit revise §10.8 + Task 8/8a/9 mechanism (DQ #311)
```

Body cites:
- DQ #311 + chronology (DQ #303 → #307 → #309 → #310 cycles).
- Rustc lint-precedence rule 4 (lower-syntax-tree wins).
- §10.8 + §13 Task 8a/8/9 new shapes.
- Co-Authored-By trailer.

## 9. DoD for this brief (advisor-side gate — completed at brief commit)

- [x] Brief committed on `phase-brehon-conformance-audit` BEFORE planner Junior task dispatch.
- [x] §2 enumerates the plan-side defect + correct mechanism with rustc-docs citation.
- [x] §2.3 lists revised task shapes (Task 8a + revised Task 8 + Task 9).
- [x] §2.4 lists what NOT to touch (Tasks 1-7 + 10-12 + ADRs + PRECONs + fix-impl commits).
- [x] §4 lists 12 constraints (revision-only, no schema change).
- [x] §5 names planner DoD: plan-parse check + §5.1 score recompute + DQ #311 self-resolve.
- [x] §G4 N/A (planner-task; not impl-task allowlist).
