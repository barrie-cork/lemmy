---
name: fix-impl brief mandates pre-push cargo-check
description: Mechanical fix-impl-task briefs must include a pre-push `cargo check --workspace --features full` (via bat wrapper) gate. Catches unused-import + adjacent `-D warnings` regressions before they cost a workflow cycle.
type: feedback
---

# Fix-impl briefs must mandate pre-push cargo-check

When the §G4 classifier auto-dispatches a narrow mechanical fix-impl
task (struct-field pad, unused-import cleanup, clippy auto-fix), the
brief MUST include a **pre-push `cargo check`** instruction in §4
Constraints. Without it, the Junior worker pushes the worker branch
and the workspace-check workflow on GH Actions discovers regressions
that a 30-second local check would have caught — costing a full
ci-watcher cycle (~3-5 min) plus the fix-impl-(N+1) recovery.

**Why this lesson exists:** Per `.claude/PRPs/reports/v1-RT-r1-retro.md`
§2 Impl miss + §5 Watch-item 6. RT-r1 fix-impl-2 (8-callsite pad in
`crates/api/api_crud/src/governance/create_endorsement.rs`,
`crates/tools/seed_founders/src/main.rs`, `crates/server/tests/e2e.rs`)
landed cleanly — all 8 callsites padded correctly. But the `use
lemmy_db_schema::source::governance::reputation_event::ReputationEventInsertForm`
import in `seed_founders/main.rs` had been added during Task 7 to
support the new field references; after fix-impl-2's pad
materialized the field initializers, the import was no longer
referenced in non-test scopes (it WAS used inside `#[cfg(test)] mod
tests` but the workspace check builds without `--test`).

Workspace-check workflow `25729693838` failed on the unused-import
`-D warnings` rule. DQ #207 raised; fix-impl-3 dispatched
(`4952f88b0` — moved import into `mod tests`). One unforced cycle,
~10 min wallclock, one extra DQ entry, one extra ci-watcher dispatch.

**Mechanical:** had the fix-impl-2 worker run `cargo check --workspace
--features full` locally before push, the unused-import `-D warnings`
would have failed locally. Junior could have either (a) moved the
import in the same commit, OR (b) filed a DQ blocker citing the
scope-boundary tension — both are better outcomes than learning the
workflow result after a 5-min ci-watcher cycle.

**How to apply:** add the following snippet to §4 Constraints of every
mechanical fix-impl brief:

```markdown
- **Pre-push cargo-check (mandatory):** before `git push origin <branch>`,
  run `bash scripts/brehon/cargo-check.sh --workspace --features full >
  .claude/PRPs/debug/<phase>-fix-impl-<N>-precheck.log 2>&1` from the
  worker branch checkout. Exit code MUST be 0. On non-zero:
  - Inspect the log tail for new warnings/errors introduced by the
    fix-impl edits.
  - If the new finding is in-scope for this fix-impl brief (mechanical,
    same class as the §G4 recipe), patch in the same commit.
  - If the new finding is OUT of scope (e.g. unused-import surfacing
    after a struct-field pad — that's a different cleanup class),
    file a `kind: "blocker"` DQ entry with `from: "impl"` citing the
    boundary, push the DQ entry, end task. Advisor §G4-classifies the
    blocker and dispatches the appropriate next-fix-impl.
  - NEVER `#[allow]`-spam to bypass the gate. Per `feedback_default_local_testing.md`.

- **Windows worker note:** the bat wrapper (`scripts\brehon\cargo-check.bat`)
  sets vcpkg PATH for `libpq.dll` resolution. Bash invocation
  (`bash scripts/brehon/cargo-check.sh`) on the EliteDesk Linux daemon
  uses the equivalent `.sh` wrapper. Per `feedback_windows_e2e_requires_bat_wrapper.md`
  for the platform divergence.
```

The gate trades **30s of local cargo-check** (warm cache; cold is
~3-5 min) against a **3-5 min ci-watcher cycle** in the failure case.
Net positive on every cycle where the fix-impl introduces an
adjacent regression — and catches the cycle BEFORE the DQ
bookkeeping cost.

**Why not "always" add it to all fix-impl briefs:**

Some fix-impl briefs are workflow-cancellation recovery (e.g.
`result: cancelled` or `timed_out`) where the prior workflow was
killed before completing — local cargo-check has no value because
the worker branch already passed compile-time. The gate applies
specifically to mechanical edits where the brief introduces NEW
diffs that could affect the workspace-check workflow.

A safer heuristic: **add the gate by default; skip only when the
brief author explicitly documents why local cargo-check is
unnecessary** (e.g. "this is a workflow-rerun on an unchanged
branch; no local check needed").

**Edge cases:**

- **Daemon worker disk + memory constraints:** the EliteDesk daemon
  has `MemoryMax=10G`; a cold `cargo check --workspace --features full`
  uses ~6 GB peak. If the daemon is concurrently running another
  cargo invocation (e.g. cohort-parallel fix-impl tasks), the
  pre-push check can OOM. Mitigation: serialize fix-impl pre-push
  checks (one at a time) OR cache via the shared `target/` directory.
  Per `feedback_resource_budget_pre_queue.md`.
- **Shape G is "off-box" by design:** the whole point of Shape G is
  GH Actions does the heavy validation. Adding local cargo-check
  back into the fix-impl flow partially regresses on Shape G's
  off-box promise. The mitigation: the local check is `cargo check`
  (compile-time), NOT `cargo test` (runtime); compile-time check is
  fast and catches the dominant unused-import / `-D warnings` class
  without dragging in test infra.
- **Worker cancels mid-precheck:** if the daemon kills the cargo
  process before exit (out-of-time, OOM, user cancel), treat as
  inconclusive — file a DQ blocker, don't push.

**Companion lessons:**

- `feedback_fix_impl_enumerate_all_callsites.md` — L5 from RT-r1
  halt retro; this lesson (pre-push check) is the runtime backstop
  when enumeration alone misses an adjacent regression class.
- `feedback_default_local_testing.md` — runtime local testing > GH
  Actions for fast feedback. This lesson extends the principle to
  fix-impl pre-push.
- `feedback_pipes_mask_exit_codes.md` — capture full output, check
  exit code, then inspect file separately. Same pattern applies to
  the pre-push cargo-check.

**Where codified:**

- `.claude/rules/advisor-orchestrator.md` §5.3 §G4 classifier
  "Callsite-enumeration discipline" paragraph — extend to mention
  pre-push cargo-check.
- Every fix-impl brief authored after this lesson lands — inject
  the §4 Constraint snippet (mechanical paste).
- This lesson file.
