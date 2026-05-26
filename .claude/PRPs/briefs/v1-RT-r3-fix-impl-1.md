# fix-impl-task brief — v1-RT-r3 fix-impl-1 (clippy::too_many_arguments)

## 1. Role + dispatch

`[role:impl-task] v1-RT-r3 fix-impl-1 — silence clippy::too_many_arguments on emit_reputation_event helpers — see .claude/PRPs/briefs/v1-RT-r3-fix-impl-1.md`

## 2. Scope

Silence `clippy::too_many_arguments` (9/7) on the two emit-helper functions extended by tasks 2 + 3. Mechanical 2-line addition; ≤2 file edits; single commit.

**Failure context (verbatim from cargo-clippy log, advisor-laptop run against merged tip e4cc1e951):**

```
error: this function has too many arguments (9/7)
   --> crates\api\api\src\governance\admin_emergency_remove.rs:447:1
    |
447 | / async fn emit_reputation_event_local(
448 | |   conn: &mut AsyncPgConnection,
449 | |   person_id: PersonId,
    ...
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.95.0/index.html#too_many_arguments
    = note: `-D clippy::too-many-arguments` implied by `-D clippy::complexity`
    = help: to override `-D clippy::complexity` add `#[allow(clippy::too_many_arguments)]`

error: this function has too many arguments (9/7)
    --> crates\api\api\src\governance\submit_jury_vote.rs:1095:1
     |
1095 | / async fn emit_reputation_event(
1096 | |   conn: &mut diesel_async::AsyncPgConnection,
1097 | |   person_id: PersonId,
```

**Root cause:** plan §10.2 signature extension added `source_event_type: ReputationEventSourceType` + `dedupe_key: Option<String>` to `emit_reputation_event`, raising the arg count from 7 → 9. Clippy's `too_many_arguments` lint threshold is 7. Task 2 extended the canonical fn in `submit_jury_vote.rs`; task 3's mirrored helper `emit_reputation_event_local` inherited the 9-arg shape per brief 3's "mirroring submit_jury_vote.rs:968-994 shape verbatim" directive.

**FILES (per advisor-laptop validate-pending-laptop diagnosis):**

- creates: []
- modifies: `crates/api/api/src/governance/submit_jury_vote.rs` (one attribute line above `emit_reputation_event` at line 1095)
- modifies: `crates/api/api/src/governance/admin_emergency_remove.rs` (one attribute line above `emit_reputation_event_local` at line 447)
- requires: [] (no inter-task dependencies; tasks 2 + 3 already merged at phase tip e4cc1e951)

**IMPLEMENT (file 1 of 2 — submit_jury_vote.rs):**

Add the line `#[expect(clippy::too_many_arguments)]` IMMEDIATELY ABOVE the `async fn emit_reputation_event(` declaration at line 1095. The attribute MUST use `#[expect(...)]` not `#[allow(...)]` — workspace clippy denies `allow_attributes` (per `.claude/lessons/feedback_clippy_test_style.md`); `#[expect(...)]` is the only allowed escape hatch.

Resulting shape:

```rust
#[expect(clippy::too_many_arguments)]
async fn emit_reputation_event(
  conn: &mut diesel_async::AsyncPgConnection,
  person_id: PersonId,
  ...
```

**IMPLEMENT (file 2 of 2 — admin_emergency_remove.rs):**

Same edit, IMMEDIATELY ABOVE the `async fn emit_reputation_event_local(` declaration at line 447.

```rust
#[expect(clippy::too_many_arguments)]
async fn emit_reputation_event_local(
  conn: &mut AsyncPgConnection,
  person_id: PersonId,
  ...
```

**Pre-push cargo-check discipline (per `feedback_fix_impl_pre_push_cargo_check.md`):** before pushing the worker branch, run `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r3-fix-impl-1-precheck.log 2>&1"`. Exit 0 required. (Clippy can't run from Junior worker — Shape G suspended per DQ #229 — but cargo-check is fast and cheap and catches adjacent regressions like unused-import surfacing.)

**VALIDATE (deferred to laptop):**

Junior subagent MUST NOT execute clippy or e2e --no-run. Write `kind: "validate-pending-laptop"` DQ entry per `advisor-orchestrator.md` §5.2 with these 3 commands verbatim:

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r3-fix-impl-1-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r3-fix-impl-1-check.log
# EXPECT: exit 0
```

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r3-fix-impl-1-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-RT-r3-fix-impl-1-clippy.log
# EXPECT: exit 0
```

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-RT-r3-fix-impl-1-test-no-run.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r3-fix-impl-1-test-no-run.log
# EXPECT: exit 0
```

**Post-validate:** Write `kind: "validate-pending-laptop"` DQ entry. Required fields:
- `commands`: the 3 VALIDATE blocks above (verbatim)
- `branch`: `"phase-v1-RT-r3"`
- `phase_task`: `1` (fix-impl, scoped to fixing both tasks 2+3 in one commit)

Generate id via `bash scripts/brehon/dq-v3-new-entry.sh`. Append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Commit + push DQ to worker branch.

**Commit subject** (one commit only):
```
fix(governance): silence clippy::too_many_arguments on emit_reputation_event helpers (fix-impl-1)
```

Commit body MUST cite both file:line callsites + reference DQ entries `81719cf8ca8d-001` (task 3) and `1bb1ff7a00a8-001` (task 2) as the originating failures + `feedback_clippy_test_style.md` as the source lesson for `#[expect(...)]` over `#[allow(...)]`.

## 3. Required reading

- `.claude/lessons/feedback_clippy_test_style.md` (workspace `allow_attributes = deny` + the three legal escape hatches; `#[expect(...)]` is escape #3)
- `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` (mandatory pre-push cargo-check on every fix-impl)
- `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md` (verify the failure-cited callsites are the ONLY callsites; if more emit helpers exist with 9+ args, expand scope OR file kind:blocker DQ)
- `crates/api/api/src/governance/submit_jury_vote.rs:1090-1100` (read shape of fn before authoring patch)
- `crates/api/api/src/governance/admin_emergency_remove.rs:444-452` (same — read shape before patching)

## 4. Constraints

- **Exactly 2 file edits.** No other file may be touched. Cohort dispatch already done; this brief is narrow mechanical fix only.
- **One commit, conventional-commits style:** `fix(governance): silence clippy::too_many_arguments on emit_reputation_event helpers (fix-impl-1)`.
- **`#[expect(...)]` NOT `#[allow(...)]`.** Workspace clippy denies `allow_attributes`. Reference `feedback_clippy_test_style.md` paragraph "Two legal escape hatches" — `#[expect]` is escape hatch #3, the only one applicable here (the other two are for tests; this is library code).
- **Pre-push cargo-check is mandatory** (per `feedback_fix_impl_pre_push_cargo_check.md`). If pre-push surfaces an adjacent regression (e.g. unused import), patch in same commit IF in-scope OR raise `kind: "blocker"` DQ IF out-of-scope.
- **Callsite enumeration check** (per `feedback_fix_impl_enumerate_all_callsites.md`): before applying the patch, `rg "emit_reputation_event(_local)?" crates/` and confirm only the two cited file:line sites exist with 9-arg signatures. If a third 9-arg emit fn exists in the diff (e.g. some inherited helper), expand scope. If a 9-arg emit exists OUTSIDE the diff (i.e. already on phase branch from task 1), DO NOT EDIT — task 1's emit was 7-arg and isn't the issue.
- **DQ field discipline:** `phase_task: 1` since fix-impl scopes to fixing both task 2 + task 3 in one commit; `branch: "phase-v1-RT-r3"`; advisor-laptop will mutate result post-validation.
- **No spec-text edits.** This brief does NOT modify the plan, the test-style lesson, or any rule file. It modifies code only.

## 5. Forbidden-window check (advisor pre-queue)

N/A — Shape G suspended; cargo runs on laptop, not daemon. Daemon-side cohort window doesn't apply for this Junior task (Junior only runs Edit + cargo-check pre-push).

## 6. Context

This is a `fix-impl-task` originating from advisor-laptop's `validate-pending-laptop` run on merged tip `e4cc1e951` (task 2 + task 3 combined). cargo-check PASS (1m36s), cargo-clippy FAIL on the two 9-arg emit helpers, cargo-test --no-run NOT RUN (blocked by clippy gate). The failure is non-allowlist per `advisor-orchestrator.md` §5.3 §G4, but the user explicitly approved the `#[expect(clippy::too_many_arguments)]` fix path. Cycle count: 1 (first occurrence; meta-rule allows normal classification through cycle 2).

Plan §10.2 grew the signature to 9 args for legitimate reasons (PRD §5.3 emit-helper extensibility — `source_event_type` and `dedupe_key` are first-class concerns of the rep-event v2 design). Refactoring to a struct would be cleaner long-term but is out-of-scope for fix-impl-1; tracked as a future tech-debt item (not filed as a DQ entry — scope-creep risk).

The two emit helpers will keep their 9-arg shape; the `#[expect]` attribute documents that this is intentional and acknowledged.
