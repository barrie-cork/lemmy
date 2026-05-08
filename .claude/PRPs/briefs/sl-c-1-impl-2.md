---
role: impl-task
plan_task: 2
phase: v1-SL-c-1
created: 2026-05-08
related_plan: .claude/PRPs/plans/v1-sponsor-liability-c-1.plan.md §13 Task 2
related_dq: null
---

# Brief — v1-SL-c-1 Task 2 — Wire scheduler tick block + atomic concurrency guard pair in `scheduled_tasks.rs`

## 1. Role + dispatch line

`[role:impl-task] sl-c-1-impl-2 — see .claude/PRPs/briefs/sl-c-1-impl-2.md`

You are the **impl-task** subagent (Sonnet 4.6). Single-file edit on
`crates/routes/src/utils/scheduled_tasks.rs`: insert a third
concurrency-guard pair (after line 91) AND a new
`scheduler.every(...).run(...)` block inside `setup()` (after line
274) that calls the SL-c-1 Task 1 module's `run_grace_check_batch`
+ `check_grace_staleness`.

## 2. Scope

**Produce:**

1. **One impl commit** with two contiguous insertions in
   `crates/routes/src/utils/scheduled_tasks.rs`:
   a. New module-scope guard pair (`SPONSOR_LIABILITY_GRACE_RUNNING`
      + `GraceCheckRunningGuard` + `impl Drop`) — anchor at end of
      `AppealWindowExpiryRunningGuard`'s `impl Drop` block (line
      ~91).
   b. New `scheduler.every(CTimeUnits::minutes(...)).run(...)` block
      — anchor at end of the appeal-window-expiry scheduler block
      (line ~274; the closing `});` of that
      `scheduler.every(CTimeUnits::hour(1)).run(...)`).
2. **Push the worker branch** to `origin/junior/<task-slug>`.
3. **Write a `kind: "validate-pending"` DQ entry** capturing the
   `cargo-validate-workspace.yml` workflow run id per Shape G Layer
   G2. This is a SECOND commit on the same worker branch.

**Do NOT** in this task:

- Touch any file other than `crates/routes/src/utils/scheduled_tasks.rs`
  and `.claude/decision-queue.json`.
- Modify any other guard pair or scheduler block.
- Refactor surrounding code or "improve" anything (no rename, no
  reorder, no extracting helpers).
- Change indentation of unrelated lines.
- Add new tests (c-2 ships e2e tests).
- Run cargo locally — Shape G; cargo runs on GH-hosted runners.
- Modify `.claude/PRPs/plans/**` or `.claude/PRPs/briefs/**`.
- Touch the `sponsor_liability_grace.rs` module from Task 1 + fix-impl-1
  (it's already shipped on the phase tip; this task only consumes its
  public functions).

## 3. Required reading

Read in this order before writing the Edits:

1. **Plan §13 Task 2** in
   `.claude/PRPs/plans/v1-sponsor-liability-c-1.plan.md` (line ~1326)
   — c-1 anchors. The c-1 plan delegates the IMPLEMENT body to the
   trunk plan for verbatim Rust.
2. **Trunk plan §13 Task 2 IMPLEMENT block** in
   `.claude/PRPs/plans/v1-sponsor-liability-c.plan.md` (lines
   2422-2550) — VERBATIM Rust code blocks for both insertions.
   Copy the code exactly. Do NOT rewrite, rephrase, or "improve"
   the code.
3. **`crates/routes/src/utils/scheduled_tasks.rs` lines 71-91** —
   existing two guard pairs (REPUTATION_SNAPSHOT_RUNNING +
   APPEAL_WINDOW_EXPIRY_RUNNING). Confirms the canonical pattern
   you mirror.
4. **`crates/routes/src/utils/scheduled_tasks.rs` lines 189-245**
   (snapshot scheduler block) and lines 254-274 (appeal-window-
   expiry scheduler block). The new block lands AFTER the closing
   `});` of the appeal-window block.
5. **`crates/routes/src/utils/scheduled_tasks.rs` lines 1-62** —
   confirm imports in scope: `AtomicBool`, `Ordering` (std::sync),
   `Utc` (chrono), `CTimeUnits` (clokwerk), `get_conn` (lemmy_diesel_utils),
   `warn` (tracing). Per plan §13 Task 2 IMPORTS line: NO new imports
   required.
6. **`crates/api/api/src/governance/sponsor_liability_grace.rs`**
   public surface (the file shipped by Task 1 + fix-impl-1, current
   tip 94e72c926). Confirm:
   - `pub async fn run_grace_check_batch(context: &LemmyContext) -> LemmyResult<GraceCheckBatchOutcome>`
   - `pub async fn check_grace_staleness(conn: &mut AsyncPgConnection, max_grace_hours: i64, multiplier: f64, now: DateTime<Utc>) -> LemmyResult<()>`
7. **`.claude/lessons/feedback_clippy_test_style.md`** — clippy lint
   discipline; new clokwerk block may trip lints.
8. **`.claude/lessons/feedback_clippy_rerun_after_fix.md`** — second-
   stage lints can fire after compile clears (the c-1 fix-impl-1
   cycle hit this pattern; expect possible clippy follow-ups).
9. **`.claude/rules/decision-queue.md`** — schema-v2 for the new
   `validate-pending` entry (id collision avoidance, mid-task push
   discipline).

## 3a. Handover from prior cohort

**From Task 1 + fix-impl-1 (sl-c-1-impl-1 #142 + sl-c-1-fix-impl-1
#144 + advisor-laptop hand-fix at 827d932d5):** SHIPPED.

- `crates/api/api/src/governance/sponsor_liability_grace.rs` exists
  at ~400 lines on phase tip 94e72c926.
- `crates/api/api/src/governance/mod.rs` line 39 carries
  `pub mod sponsor_liability_grace;`.
- Workflow `25529116907` passed (DQ #162 mutated by advisor at
  94e72c926; cargo check + clippy + e2e --no-run all green).
- 4 pub fns + 1 pub enum + 1 pub struct exposed from the module.

**Phase tip:** `94e72c926` on `phase-v1-SL-c-1`
(`chore(decision-queue): advisor mutated DQ #162 — pass workspace
check (post-ci-watcher #145 hard-refusal breach)`).

**DQ pending on phase tip:** #156 (SL-b inert), #160 (resolved fail
by ci-watcher), #161 (resolved fail by advisor-laptop). #162 is in
resolved[] (just mutated by advisor; the green-gate close).

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branched off `phase-v1-SL-c-1` (tip
  `94e72c926`).
- `git branch --show-current` should return a `junior/role-impl-task-...`
  branch (adapt to your actual worktree branch name).
- Two commits at task end:
  1. Impl: `feat(v1-SL-c-1): wire sponsor_liability_grace scheduler block + atomic concurrency guard (task 2)` (matches plan §13 Task 2 COMMIT MESSAGE; minor adjustment from `v1-SL-c` → `v1-SL-c-1` for the c-1 split).
  2. DQ: `chore(decision-queue): impl raised DQ #<next-id> — sl-c-1-impl-2 validate-pending`

### Implementation discipline — the EXACT edits

**Insertion 1: Module-scope guard pair.** Anchor at the end of
`AppealWindowExpiryRunningGuard`'s `impl Drop` block (line 91 in the
worktree's `scheduled_tasks.rs` — verify with
`grep -n 'AppealWindowExpiryRunningGuard' crates/routes/src/utils/scheduled_tasks.rs`).

Insert IMMEDIATELY after the closing `}` of that `impl Drop` block:

```rust

// Concurrency guard for the 5-minute Brehon sponsor-liability
// grace-check tick. Mirrors REPUTATION_SNAPSHOT_RUNNING +
// RunningGuard (lines 71-79) and APPEAL_WINDOW_EXPIRY_RUNNING +
// AppealWindowExpiryRunningGuard (lines 83-91).
static SPONSOR_LIABILITY_GRACE_RUNNING: AtomicBool = AtomicBool::new(false);

struct GraceCheckRunningGuard;

impl Drop for GraceCheckRunningGuard {
  fn drop(&mut self) {
    SPONSOR_LIABILITY_GRACE_RUNNING.store(false, Ordering::Release);
  }
}
```

(The leading blank line is intentional — matches the spacing pattern
between the existing two guard pairs.)

**Insertion 2: Scheduler tick block.** Anchor at the end of the
appeal-window-expiry scheduler block. Use:

```bash
grep -n '"BREHON_DISABLE_APPEAL_WINDOW_JOB"' crates/routes/src/utils/scheduled_tasks.rs
```

to confirm the appeal-window block's location at task-time. The new
block lands AFTER the closing `});` of that
`scheduler.every(CTimeUnits::hour(1)).run(...)` (line ~274).

Insert IMMEDIATELY after that `});`:

```rust

// Brehon governance v1: sponsor-liability grace-check tick.
// Interval is read from `job.grace_check_interval_minutes` at
// scheduler setup (default 5). Find SponsorLiabilityPending cases
// past their grace_expires_at and transition them to Fired or
// Escaped per PRD §6.1 + §6.2.
//
// Restart-required tunability: clokwerk schedules pin at
// registration. Flipping `job.grace_check_interval_minutes` via
// POST /admin/config takes effect at next server restart.
// Mirrors v0 reputation-snapshot precedent (15-min interval
// hardcoded at scheduler setup).
//
// Disabled in tests via BREHON_DISABLE_GRACE_CHECK_JOB=1
// (mirrors BREHON_DISABLE_SNAPSHOT_JOB pattern at line 197).
//
// Concurrency guard mirrors REPUTATION_SNAPSHOT_RUNNING /
// APPEAL_WINDOW_EXPIRY_RUNNING. After the batch tick, a sibling
// staleness pass emits tracing::error! per stuck case (PRD §6.3).
let context_grace = context.reset_request_count();
let grace_pool = &mut context.pool();
let grace_interval_minutes_i64: i64 = lemmy_api::governance::config::get_int(
  &mut lemmy_api::governance::config::ConfigCache::new(),
  grace_pool,
  lemmy_api::governance::config::Scope::Instance,
  "job.grace_check_interval_minutes",
)
.await
.unwrap_or(5);
let grace_interval_minutes: u32 =
  u32::try_from(grace_interval_minutes_i64).unwrap_or(5);
scheduler.every(CTimeUnits::minutes(grace_interval_minutes)).run(move || {
  let context = context_grace.reset_request_count();
  async move {
    // Watchpoint #9: env-var check FIRST in closure body. Reversing
    // means tests that set BREHON_DISABLE_GRACE_CHECK_JOB still
    // consume an atomic-bool slot, leaking guards.
    if std::env::var("BREHON_DISABLE_GRACE_CHECK_JOB").as_deref() == Ok("1") {
      return;
    }
    if SPONSOR_LIABILITY_GRACE_RUNNING
      .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
      .is_err()
    {
      warn!("sponsor_liability_grace: previous batch still running, skipping this tick");
      return;
    }
    let _guard = GraceCheckRunningGuard;
    lemmy_api::governance::sponsor_liability_grace::run_grace_check_batch(&context)
      .await
      .inspect_err(|e| warn!("Failed to run grace_check batch: {e}"))
      .ok();

    // Staleness pass after the batch (per PRD §6.3 + DQ #146).
    // Mirrors snapshot pattern at scheduled_tasks.rs:213-244.
    let staleness_pool = &mut context.pool();
    let mut staleness_cache =
      lemmy_api::governance::config::ConfigCache::new();
    let max_grace_hours = lemmy_api::governance::config::get_int(
      &mut staleness_cache,
      staleness_pool,
      lemmy_api::governance::config::Scope::Instance,
      "liability.grace_window_maximum_hours",
    )
    .await
    .unwrap_or(720);
    let multiplier = lemmy_api::governance::config::get_float(
      &mut staleness_cache,
      staleness_pool,
      lemmy_api::governance::config::Scope::Instance,
      "job.grace_check_staleness_alert_multiplier",
    )
    .await
    .unwrap_or(2.0);
    match get_conn(staleness_pool).await {
      Ok(mut conn) => {
        if let Err(e) =
          lemmy_api::governance::sponsor_liability_grace::check_grace_staleness(
            &mut conn,
            max_grace_hours,
            multiplier,
            Utc::now(),
          )
          .await
        {
          warn!("grace staleness check failed: {e}");
        }
      }
      Err(e) => warn!("grace staleness check: get_conn failed: {e}"),
    }
  }
});
```

(Leading blank line intentional — matches the spacing between
existing scheduler blocks.)

**Watchpoint bindings (per plan §4):**

- **Watchpoint #3 (distinct guard pair):** must add a NEW static +
  struct, NOT reuse `REPUTATION_SNAPSHOT_RUNNING` or
  `APPEAL_WINDOW_EXPIRY_RUNNING`.
- **Watchpoint #5 (staleness reads INSIDE closure):** the
  `max_grace_hours` + `multiplier` reads MUST happen INSIDE the
  closure body (per-tick), NOT at `setup()` time. The
  `grace_interval_minutes` read happens at setup (only knob that's
  restart-required per PRD §6.4). Don't move config reads.
- **Watchpoint #9 (env-var FIRST):** the
  `std::env::var("BREHON_DISABLE_GRACE_CHECK_JOB")` check is the
  FIRST statement in the `async move` body, BEFORE
  `compare_exchange`. Reversing leaks guards.

### Validate-pending DQ entry shape (Shape G Layer G2)

```json
{
  "id": <next-int>,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<NOW_ISO>",
  "subject": "sl-c-1-impl-2 workspace-check validate-pending",
  "question": "Does cargo-validate-workspace pass for sl-c-1 task 2 (scheduler block + concurrency guard)?",
  "options": [
    "(A) pass — green-gate close; advance to Task 3 (retro)",
    "(B) fail — §G4 triage"
  ],
  "context": "impl-task pushed scheduler block + atomic guard pair at commit <impl-sha> on branch <worker-branch>. cargo-validate-workspace.yml triggered run <run-id>.",
  "branch": "<your-worker-branch>",
  "phase_task": 2,
  "workflow_run_id": <int from gh run list>,
  "commands": [
    "cargo check --workspace --features full",
    "cargo clippy --workspace --features full --no-deps -- -D warnings",
    "cargo test --no-run -p lemmy_server --test e2e"
  ],
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

**Compute next-id correctly per `.claude/rules/decision-queue.md`
Hard refusal #2:** `max(all_ids, default=0) + 1` across BOTH
`pending[]` and `resolved[]`. Last assigned id was 162 (advisor
mutated to pass at 94e72c926); next is 163.

**Use `ensure_ascii=False`** when re-serialising the JSON file per
`feedback_json_dump_ensure_ascii_false` — preserves UTF-8 (em-dashes,
section markers) cleanly. Do NOT escape `—` / `§` / other non-ASCII.

After pushing the impl commit, capture the `workflow_run_id`:

```bash
gh run list --repo barrie-cork/lemmy --branch <your-worker-branch> --workflow cargo-validate-workspace --limit 1 --json databaseId --jq '.[0].databaseId'
```

If `gh run list` returns empty, wait up to 60 seconds and retry.

### Hard refusals

- Do NOT modify any file other than the two named
  (`scheduled_tasks.rs` + `.claude/decision-queue.json`).
- Do NOT touch any guard pair other than the new one (do not edit
  REPUTATION_SNAPSHOT_RUNNING or APPEAL_WINDOW_EXPIRY_RUNNING).
- Do NOT edit any existing scheduler block.
- Do NOT add new imports — verify all 6 names are already in scope
  per plan §13 Task 2 IMPORTS section.
- Do NOT push the worker branch BEFORE both commits (impl + DQ
  entry) land.
- Do NOT reuse a DQ id; compute `max + 1`.
- Do NOT escape non-ASCII characters when writing decision-queue.json.

## 5. Acceptance

Task 2 passes if:

- `git diff HEAD~2 -- crates/routes/src/utils/scheduled_tasks.rs`
  shows ONLY two contiguous insertions: (a) the new guard pair
  (~10 lines) after line 91, (b) the new scheduler block (~70
  lines) after line 274. No other diff lines anywhere in `.rs`
  files.
- `git diff HEAD~1 -- .claude/decision-queue.json` shows ONE new
  entry appended to `pending[]` with the shape above, and no other
  changes to existing entries.
- One impl commit + one DQ-entry commit on the worker branch.
- DQ entry written with `kind: "validate-pending"`, `from: "impl"`,
  populated `workflow_run_id`.
- ci-watcher's later poll on the workflow run returns
  `conclusion: "success"`.
