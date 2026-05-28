# fix-impl-task brief — v1-RT-r3 fix-impl-2 (DQ syntax + e2e iso-week guard hoist)

## 1. Role + dispatch

`[role:impl-task] v1-RT-r3 fix-impl-2 — fix DQ entry boundary at line 4091 + hoist iso-week guards in e2e fixtures — see .claude/PRPs/briefs/v1-RT-r3-fix-impl-2.md`

## 2. Scope

Two CR findings from PR #155, both mechanical, single commit:

- **cr-1 (🔴 Critical, outside-diff):** `.claude/decision-queue.json:4091` — DQ entry `a3d0e9941441-025` collapsed into the previous `validate-pending-laptop` entry (`c76792506538-001`) due to a missing `},\n    {` boundary. Python `json.load` + `jq` parse to 207 entries silently, but entry `-025` now carries both the previous entry's `branch`/`phase_task`/`result`/`log_slice`/`failed_commands` fields AND its own `kind:log`/`question`/`answer` fields in one object. Strict parsers (`yq -e`) would reject; the corruption is silent under Python/jq.
- **cr-3 (🟡 Minor):** `crates/server/tests/e2e.rs:17657-17660` + `e2e.rs:17740-17743` — ISO-week-boundary guard runs AFTER the `events_emitted == 0` assertion in both `participation_activity_cron_idempotent_across_same_iso_week` (line 17651) and `participation_dormancy_cron_idempotent_across_same_iso_week` (line 17737). If the ISO week flips mid-test, the assertion fires before the guard can exit cleanly as xfail. The guard must be hoisted BEFORE the assertion.

Three file edits total, single commit.

**FILES:**

- creates: []
- modifies:
  - `.claude/decision-queue.json` (3-line edit around line 4090-4091)
  - `crates/server/tests/e2e.rs` (two test functions, ~10 lines each)
- requires: [] (no inter-task dependencies; CR review on PR #155 already posted)

## 2.1 §G4 CANONICAL RECIPE — n/a (non-allowlist, user-approved fix-in-PR)

Both findings are NON-allowlist per `advisor-orchestrator.md` §5.3 §G4 (CR-surfaced findings, not a `validate-pending` fail). User-gate 3 (CR triage approval) was explicitly obtained at 2026-05-26T17:47Z; the user approved option (a) "Approve as drafted — queue fix-impl-2 then fix-impl-3". No verbatim §G4 row applies because this brief is the user-approved fix-in-PR action, not an auto-classifier dispatch. The recipe text in §2.2 below is the contract.

## 2.2 IMPLEMENT (file 1 of 2 — `.claude/decision-queue.json`)

The defective region at lines 4085-4106 (current state):

```
      ],
      "branch": "phase-v1-RT-r3",
      "phase_task": 1,
      "result": "pass",
      "log_slice": null,
      "failed_commands": null,
    "from": "advisor",
      "kind": "log",
      "timestamp": "2026-05-26T09:30:00Z",
      ...
      "id": "a3d0e9941441-025"
    },
    {
      "from": "advisor",
```

The fix:

1. After `"failed_commands": null,` on line 4090, the previous entry (`c76792506538-001`, a `validate-pending-laptop` for task 1) MUST close with `}` and a new entry `{` MUST open.
2. The current line 4091 (`    "from": "advisor",`) has 4-space indent instead of 6-space — that's the corruption signature.

Apply this edit (Edit tool, with file_path `.claude/decision-queue.json`):

```
old_string:
      "failed_commands": null,
    "from": "advisor",
      "kind": "log",
      "timestamp": "2026-05-26T09:30:00Z",

new_string:
      "failed_commands": null
    },
    {
      "from": "advisor",
      "kind": "log",
      "timestamp": "2026-05-26T09:30:00Z",
```

Note: `"failed_commands": null,` → `"failed_commands": null` (remove trailing comma), then `},\n    {` is the new entry boundary, then `"from": "advisor",` indented to 6 spaces.

After the edit, the previous validate-pending-laptop entry MUST also have a valid `id` field somewhere in its body (lines 4070-4090 — verify by `Read` lines 4060-4092 BEFORE applying the patch). If the previous entry's `id` (`c76792506538-001`) was eaten by the collapse, you must also restore it before the closing `}` — find the previous entry's id field by searching for `c76792506538-001` in the file BEFORE the patch.

**Verification (mandatory):**

```bash
python -c "import json, io; data = json.load(io.open('.claude/decision-queue.json', encoding='utf-8')); print('resolved=' + str(len(data['resolved'])) + ' unique_ids=' + str(len(set(e['id'] for e in data['resolved']))))"
# EXPECT: resolved=208 unique_ids=208 (one more than current — the orphaned entry now parses as its own)
```

```bash
git diff .claude/decision-queue.json | head -20
# EXPECT: shows the 3-line structural fix; no other change
```

## 2.3 IMPLEMENT (file 2 of 2 — `crates/server/tests/e2e.rs`)

**Edit A — `participation_activity_cron_idempotent_across_same_iso_week` (line 17651-17661):**

The current shape:

```rust
    let first = participation_cron::run_activity_batch(&context).await?;
    assert_eq!(first.events_emitted, 3, "first run emits 3 rows");
    let second = participation_cron::run_activity_batch(&context).await?;
    assert_eq!(
      second.events_emitted, 0,
      "second run in same iso_week emits 0 (dedupe_key on activity_cron:<community>:<person>:<iso_week>)"
    );

    let week_end_iso_week = Utc::now().iso_week();
    if week_start_iso_week != week_end_iso_week {
      eprintln!("xfail: ISO week boundary crossed mid-test ({week_start_iso_week:?} -> {week_end_iso_week:?})");
    }
    Ok(())
```

Fix: hoist the iso-week check BETWEEN the two `run_activity_batch` calls (after `first`, before `second`). On mismatch, return `Ok(())` early as xfail — the test cannot meaningfully assert idempotence across a week boundary.

```rust
    let first = participation_cron::run_activity_batch(&context).await?;
    assert_eq!(first.events_emitted, 3, "first run emits 3 rows");

    let week_end_iso_week = Utc::now().iso_week();
    if week_start_iso_week != week_end_iso_week {
      eprintln!("xfail: ISO week boundary crossed mid-test ({week_start_iso_week:?} -> {week_end_iso_week:?})");
      return Ok(());
    }

    let second = participation_cron::run_activity_batch(&context).await?;
    assert_eq!(
      second.events_emitted, 0,
      "second run in same iso_week emits 0 (dedupe_key on activity_cron:<community>:<person>:<iso_week>)"
    );

    Ok(())
```

(The trailing `Ok(())` stays; the duplicate week-end check after the assertion is removed.)

**Edit B — `participation_dormancy_cron_idempotent_across_same_iso_week` (line 17735-17744):**

Apply the EXACT same pattern to the dormancy test:

```rust
    let first = participation_cron::run_dormancy_batch(&context).await?;
    assert_eq!(first.events_emitted, 3, "first run emits 3 rows");

    let week_end_iso_week = Utc::now().iso_week();
    if week_start_iso_week != week_end_iso_week {
      eprintln!("xfail: ISO week boundary crossed mid-test ({week_start_iso_week:?} -> {week_end_iso_week:?})");
      return Ok(());
    }

    let second = participation_cron::run_dormancy_batch(&context).await?;
    assert_eq!(second.events_emitted, 0, "second run in same iso_week emits 0");

    Ok(())
```

**Two Edit tool calls maximum on `e2e.rs`** (one per test). DO NOT scan the rest of the 17000-line file; use exact-match `old_string` with sufficient surrounding context to make each edit unique. The two test bodies differ in seed counts (3 vs 3 — same), batch fn (activity vs dormancy), and outcome assertions — pick distinctive substrings to anchor each Edit.

## 2.4 Pre-push cargo-check discipline (mandatory, per `feedback_fix_impl_pre_push_cargo_check.md`)

Before pushing the worker branch:

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r3-fix-impl-2-precheck.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r3-fix-impl-2-precheck.log
```

Exit 0 required. The fix is structural-text-only on JSON + two test fns; no library code touched, so the only risk is a typo in the Rust edits surfacing as a compile error.

## 2.5 VALIDATE (deferred to laptop)

Junior subagent MUST NOT execute clippy or e2e --no-run. Write `kind: "validate-pending-laptop"` DQ entry per `advisor-orchestrator.md` §5.2 with these 3 commands verbatim:

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r3-fix-impl-2-check.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r3-fix-impl-2-clippy.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-RT-r3-fix-impl-2-test-no-run.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

**Post-validate:** Write `kind: "validate-pending-laptop"` DQ entry. Required fields:
- `commands`: the 3 VALIDATE blocks above (verbatim)
- `branch`: `"phase-v1-RT-r3"`
- `phase_task`: `2` (fix-impl-2, scoped to cr-1 + cr-3)

Generate id via `bash scripts/brehon/dq-v3-new-entry.sh`. Append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Commit + push DQ to worker branch.

## 2.6 Commit subject (one commit only)

```
fix(governance): repair DQ entry boundary at line 4091 + hoist iso-week guards in v1_rt_r3_fixtures idempotency tests (fix-impl-2)
```

Commit body MUST cite:
- CR finding cr-1 (URL: `https://github.com/barrie-cork/lemmy/pull/155#discussion_r4366042583`) for the DQ fix
- CR finding cr-3 (URL: `https://github.com/barrie-cork/lemmy/pull/155#discussion_r3305670145`) for the e2e fix
- Verification: `python -c "json.load..."` → `resolved=208 unique_ids=208` (paste the exact stdout)

## 3. Required reading

Mandatory file-class lessons (per `advisor-orchestrator.md` §2.4):

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` (e2e.rs edit — Case A/B/C error-shape; the existing `v1_rt_r3_fixtures` mod uses `LemmyResult<()>` outer with `LemmyResult<T>` helpers — DO NOT change error shape, just hoist the iso_week check)
- `.claude/lessons/feedback_async_pool_test_pattern.md` (e2e.rs edit — `AsyncPgConnection::establish` + `DbPool::Conn` pattern; this fix touches NEITHER, so just confirm no drift)
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` (e2e.rs ≥2 edits — anchor each Edit with distinctive surrounding context; never use full-file Read on this 17000-line file; cap at 2 Edit calls per test)
- `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` (mandatory pre-push cargo-check on every fix-impl)
- `.claude/lessons/feedback_json_dump_ensure_ascii_false.md` (DQ JSON edit — verify via `json.load` after edit; ASCII-only is fine here, no non-ASCII chars touched, but the verification recipe applies)

Pattern source:

- `.claude/PRPs/briefs/v1-RT-r3-fix-impl-1.md` (canonical fix-impl-N brief shape; this brief mirrors it)
- `.claude/PRPs/reviews/pr-155-findings.yaml` (cr-1 + cr-3 finding bodies, severity, file:line)
- `crates/server/tests/e2e.rs:17615-17710` (existing `v1_rt_r3_fixtures` shape — read for context BEFORE applying Edit A and Edit B)

## 4. Constraints

- **Exactly 2 file edits.** `.claude/decision-queue.json` + `crates/server/tests/e2e.rs` only. DO NOT touch any other file.
- **e2e.rs Edit cap: 2 Edit tool calls** (one per test). NEVER `Read` the full file. Use distinctive anchors per `feedback_junior_worker_e2e_edit_hang.md`.
- **One commit, conventional-commits style:** subject above. Body cites both CR finding URLs + verification stdout.
- **Pre-push cargo-check is mandatory** (per `feedback_fix_impl_pre_push_cargo_check.md`). If pre-push surfaces an adjacent regression (e.g. e2e fixtures import order broke), patch in same commit IF in-scope (i.e. inside the two test fns) OR raise `kind: "blocker"` DQ IF out-of-scope.
- **JSON verification mandatory** (per `feedback_json_dump_ensure_ascii_false.md` + cr-1 IS a JSON-corruption defect): after the Edit, run the `json.load` + `unique_ids` check. If `resolved` count differs from `208`, abort and raise `kind: "blocker"` DQ.
- **Atomic JSON edit** (per `multi-lane-worktree.md` Hard refusal #6): canonical-checkout DQ writes need read→mutate→verify→commit as a single shell sequence, BUT this is a phase-branch (Mode A lane) edit, not canonical — Hard refusal #6 applies only to canonical writes. Still: minimise window between Edit and commit; do not interleave with other tool calls.
- **DQ field discipline for the validate-pending-laptop entry:** `phase_task: 2` (fix-impl-2); `branch: "phase-v1-RT-r3"`; advisor-laptop will mutate result post-validation.
- **No spec-text edits.** No plan, no lesson, no rule file modification.
- **No EnvVarGuard scope changes** (out-of-scope per user gate 3 — cp-4/cp-5 are carry-forward).
- **No emit_reputation_event hoist** (out-of-scope per user gate 3 — cr-7 is carry-forward).

## 5. Forbidden-window check (advisor pre-queue)

N/A — Shape G suspended; cargo runs on laptop, not daemon. Daemon-side cohort window doesn't apply for this Junior task (Junior only runs Edit + cargo-check pre-push).

## 6. Context

CR (CodeRabbit) review on PR #155 posted at 2026-05-26T17:31:51Z. CR posted 1 review + 6 inline + 1 outside-diff comment; Copilot posted 1 review + 5 inline. Triage was completed by advisor at 2026-05-26T17:47:01Z (see `.claude/PRPs/reviews/pr-155-findings.yaml` + `.claude/PRPs/reviews/pr-155-comment.md`).

User-gate 3 (CR triage approval) was surfaced via `AskUserQuestion` and approved at the same time. User selected option (a) "Approve as drafted — queue fix-impl-2 then fix-impl-3". This brief is fix-impl-2 (the first of two queued fix-impl tasks).

After this commit lands + validate-pending-laptop PASS, fix-impl-3 will queue for cp-1/cp-2/cp-3 (config-input clamping). After both fix-impls land + re-validate, second `bm-poll-cr` then bm-merge gate.
