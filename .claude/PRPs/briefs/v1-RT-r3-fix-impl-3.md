---
role: impl-task
plan_task: cr-triage-fixes-3
phase: v1-RT-r3
created: 2026-05-28
related_dq: pr-155-findings.yaml (cp-1, cp-2, cp-3, cr-12)
---

# Brief — v1-RT-r3 fix-impl-3 — cp-1/cp-2/cp-3 config-clamp group + cr-12 retro table escape

> **Clarify provenance:** this is a fix-impl brief addressing 4 binding `fix-in-pr` findings from PR #155's `/bm-triage` (committed `9f6024b95`, posted `pullrequestcomment-4565869493`). No clarify-DQ on this brief: triage already resolved bucket + scope.

## 1. Role + dispatch line

`[role:impl-task] v1-RT-r3 fix-impl-3 — see .claude/PRPs/briefs/v1-RT-r3-fix-impl-3.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute 4 fix-in-pr findings from `.claude/PRPs/reviews/pr-155-findings.yaml`. **Two commits**, one per logical group (per `feedback_bundle_means_one_worker_branch_not_one_commit.md`).

## 2. Scope — bundled (cp-1+cp-2+cp-3 clamp group + cr-12 retro escape in one Junior task)

**Why bundled:** ONE worker branch, ONE workspace-check workflow, ONE ci-watcher cycle. Two commits per the impl-task contract.

**Risk class:** LOW. Edits target `participation_cron.rs` + `scheduled_tasks.rs` + one markdown retro line. **NOT** `crates/server/tests/e2e.rs` (avoids Junior #479 failure class — context-cost-driven `error_max_turns` on 8945-line e2e). Edits per file: 1-3 small clamps + 1 markdown line. Total brief: ≤200 lines.

### 2.1 Commit 1 — Config-input clamping (cp-1 + cp-2 + cp-3)

**Files:**
- `crates/routes/src/utils/scheduled_tasks.rs:471-482` (cp-1: `participation_interval_days`)
- `crates/api/api/src/governance/participation_cron.rs:86-102` (cp-2: `lookback_days` + `activity_threshold`)
- `crates/api/api/src/governance/participation_cron.rs:233-240` (cp-3: `dormancy_window_days`)

**Canonical mirror to copy verbatim** — `scheduled_tasks.rs:427-440` already implements this exact pattern for `replay_window_days`:

```rust
let raw_window_days_i64 = lemmy_api::governance::config::get_int(
  &mut cache,
  pool,
  lemmy_api::governance::config::Scope::Instance,
  "federation.inbound.replay_window_days",
)
.await
.unwrap_or(7);
let window_days_i64 = raw_window_days_i64.max(1);
if raw_window_days_i64 < 1 {
  warn!(
    "federation_inbox_nonce cleanup: invalid replay_window_days={raw_window_days_i64}; clamped to 1"
  );
}
```

**Recipe — apply identical shape per finding:**

#### cp-1: `scheduled_tasks.rs:471-482` — `participation_interval_days`

Current code (line 471-482):

```rust
let context_participation = context.reset_request_count();
let participation_pool = &mut context.pool();
let participation_interval_days_i64: i64 = lemmy_api::governance::config::get_int(
  &mut lemmy_api::governance::config::ConfigCache::new(),
  participation_pool,
  lemmy_api::governance::config::Scope::Instance,
  "job.participation_interval_days",
)
.await
.unwrap_or(7);
let participation_interval_days: u32 =
  u32::try_from(participation_interval_days_i64).unwrap_or(7);
```

Replace with (insert clamp + warn between `unwrap_or(7)` and the `u32::try_from`):

```rust
let context_participation = context.reset_request_count();
let participation_pool = &mut context.pool();
let raw_participation_interval_days_i64: i64 = lemmy_api::governance::config::get_int(
  &mut lemmy_api::governance::config::ConfigCache::new(),
  participation_pool,
  lemmy_api::governance::config::Scope::Instance,
  "job.participation_interval_days",
)
.await
.unwrap_or(7);
let participation_interval_days_i64 = raw_participation_interval_days_i64.max(1);
if raw_participation_interval_days_i64 < 1 {
  warn!(
    "participation_cron: invalid participation_interval_days={raw_participation_interval_days_i64}; clamped to 1"
  );
}
let participation_interval_days: u32 =
  u32::try_from(participation_interval_days_i64).unwrap_or(7);
```

#### cp-2: `participation_cron.rs:86-102` — `lookback_days` + `activity_threshold`

Current code (line 86-102):

```rust
let lookback_days = config::get_int(
  &mut cache,
  pool,
  Scope::Instance,
  "participation.lookback_days",
)
.await
.unwrap_or(config::DEFAULT_PARTICIPATION_LOOKBACK_DAYS);

let activity_threshold = config::get_int(
  &mut cache,
  pool,
  Scope::Instance,
  "participation.activity_threshold_comments",
)
.await
.unwrap_or(config::DEFAULT_PARTICIPATION_ACTIVITY_THRESHOLD_COMMENTS);
```

Replace with (clamp+warn after each `unwrap_or`):

```rust
let raw_lookback_days = config::get_int(
  &mut cache,
  pool,
  Scope::Instance,
  "participation.lookback_days",
)
.await
.unwrap_or(config::DEFAULT_PARTICIPATION_LOOKBACK_DAYS);
let lookback_days = raw_lookback_days.max(1);
if raw_lookback_days < 1 {
  warn!(
    "participation_activity_cron: invalid lookback_days={raw_lookback_days}; clamped to 1"
  );
}

let raw_activity_threshold = config::get_int(
  &mut cache,
  pool,
  Scope::Instance,
  "participation.activity_threshold_comments",
)
.await
.unwrap_or(config::DEFAULT_PARTICIPATION_ACTIVITY_THRESHOLD_COMMENTS);
let activity_threshold = raw_activity_threshold.max(1);
if raw_activity_threshold < 1 {
  warn!(
    "participation_activity_cron: invalid activity_threshold_comments={raw_activity_threshold}; clamped to 1"
  );
}
```

#### cp-3: `participation_cron.rs:233-240` — `dormancy_window_days`

Current code (line 233-240):

```rust
let dormancy_window_days = config::get_int(
  &mut cache,
  pool,
  Scope::Instance,
  "participation.dormancy_window_days",
)
.await
.unwrap_or(config::DEFAULT_PARTICIPATION_DORMANCY_WINDOW_DAYS);
```

Replace with:

```rust
let raw_dormancy_window_days = config::get_int(
  &mut cache,
  pool,
  Scope::Instance,
  "participation.dormancy_window_days",
)
.await
.unwrap_or(config::DEFAULT_PARTICIPATION_DORMANCY_WINDOW_DAYS);
let dormancy_window_days = raw_dormancy_window_days.max(1);
if raw_dormancy_window_days < 1 {
  warn!(
    "participation_dormancy_cron: invalid dormancy_window_days={raw_dormancy_window_days}; clamped to 1"
  );
}
```

**Imports check (cp-2 + cp-3):** verify `warn!` is in scope in `participation_cron.rs`. If not, add `use tracing::warn;` at the top. The file already uses `tracing` per existing `info!` calls at line 215.

**Commit message:** `fix(governance): clamp participation cron config inputs to >=1 with warn-on-invalid (cp-1, cp-2, cp-3, PR #155)`

### 2.2 Commit 2 — Retro markdown table escape (cr-12)

**File:** `.claude/PRPs/reports/session-retro-2026-05-26-pr-155-cr-triage-junior-479-fail.md:31`

**Current line 31** (broken — literal `|` inside the `--paginate | jq` code spans splits row into 7 columns instead of 5):

```
| 4 | Default CR triage ingest to `gh api /pulls/N/comments --paginate | jq` + `gh api /pulls/N/reviews | jq` instead of `gh pr view --comments`. Update `.claude/commands/bm/bm-poll-cr.md` Phase 1 to specify `gh api` as the primary fetch. | Avoids 44 KB JSON blob blowing the Read tool token cap; cuts ~5 min of re-fetching per CR poll. | minor (single-line update to a slash command) | 1× this session; not a new pattern (the token-cap behaviour was known) but never documented in the verb |
```

**Replace with** (HTML-entity-escape both pipes inside code spans — `&#124;`):

```
| 4 | Default CR triage ingest to `gh api /pulls/N/comments --paginate &#124; jq` + `gh api /pulls/N/reviews &#124; jq` instead of `gh pr view --comments`. Update `.claude/commands/bm/bm-poll-cr.md` Phase 1 to specify `gh api` as the primary fetch. | Avoids 44 KB JSON blob blowing the Read tool token cap; cuts ~5 min of re-fetching per CR poll. | minor (single-line update to a slash command) | 1× this session; not a new pattern (the token-cap behaviour was known) but never documented in the verb |
```

**Commit message:** `docs(retro): escape pipes inside code spans on row 4 (cr-12, PR #155)`

### 2.3 Do NOT in this task

- Address cr-2, cr-4, cr-7, cp-4, cp-5 (carry-forward; GH issues #156-#160 already filed for v1-quality-r2).
- Address cr-5, cr-6, cr-8, cr-9, cr-10, cr-11, cr-13 (wont-fix; rationales already in YAML).
- Touch `crates/server/tests/e2e.rs` — that's the v1-RT-r3 Junior #479 failure class. No e2e edits in this task.
- Reword the retro's other rows (cr-12 is row 4 ONLY — leave rows 1/2/3/5/6 untouched).

## 3. Required reading

In this order:

1. **`.claude/PRPs/reviews/pr-155-findings.yaml`** — the source of truth. Read `cp-1`, `cp-2`, `cp-3`, `cr-12` entries to confirm severity + rationale match this brief.
2. **`.claude/decision-queue.json`** — no clarify-DQ on this brief; no pending blockers gate this dispatch.
3. **MIRROR ref — `crates/routes/src/utils/scheduled_tasks.rs:427-440`** — the canonical clamp pattern (already shipped for `replay_window_days`). Copy this shape into each of the three cp-* fixes.
4. **Lessons** (Glob `.claude/lessons/`):
   - `feedback_pipes_mask_exit_codes.md` (always — capture-then-tail rule for §5 cargo gates)
   - `feedback_clippy_test_style.md` (always — workspace denies escape-hatches; no `unwrap`/`expect`/`allow`)
   - `feedback_lemmy_error_no_std_error.md` (defensive — this brief doesn't add `Result`-returning helpers but the touched files are LemmyResult-shaped; if a clippy lint fires on existing `.unwrap_or(...)` it's pre-existing, not in scope)

## 3a. Handover from prior cohort

(none — fix-impl-3 follows fix-impl-2 which shipped at `90cd0183d`; not a `[P]` cohort.)

## 4. Constraints

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-RT-r3`. Finalize merges your worktree branch back; do not push to `phase-v1-RT-r3` directly.
- **Two commits**, one per logical group (Commit 1 = cp-1/cp-2/cp-3, Commit 2 = cr-12). Order is binding: Commit 1 first (Rust code changes that validate via cargo), then Commit 2 (markdown-only). If clippy/check fails after Commit 1, amend or fixup that commit; do not collapse into Commit 2.
- Mid-task DQ visibility: if you raise a `pending` entry, **commit + push immediately** to your worktree branch.
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Brief verbosity limits (per session-retro-2026-05-26 What-to-change #2)

This brief is ≤200 lines and ≤2 file groups. Within limits. **Do NOT pre-fetch the YAML or canonical mirror at task start** — both are excerpted verbatim above. Read them only if Edit fails on anchor mismatch.

### Plan-cited line numbers may have drifted

Line numbers in §2 were captured at phase tip `1e2f536a3` (2026-05-28). If `grep -n` shows the function signature has moved by ±5 lines, follow the grep output. If the function shape itself has changed (e.g. `unwrap_or` replaced with something else), file a DQ pending entry — do not re-author the clamp differently.

### Lesson trailer (encouraged)

If you discover a non-obvious constraint touching `participation_cron.rs` or the clamp pattern, append `LESSON:` to your commit body per `feedback_junior_pmd_write_convention.md`.

<!-- SHAPE-G-SUSPENDED until 2026-06-01: impl-task writes validate-pending-laptop (not validate-pending). Advisor laptop runs cargo. -->

## 5. Validation gates (LAPTOP — Shape-G suspended)

After both commits land, push the worker branch, write a `kind: "validate-pending-laptop"` DQ entry with `commands[]` populated below. Advisor laptop session mutates the entry on completion.

`commands[]` for the DQ entry:

1. `bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-RT-r3-fix-impl-3-check.log 2>&1` → exit 0.
2. `bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r3-fix-impl-3-clippy.log 2>&1` → exit 0.

Per `feedback_pipes_mask_exit_codes.md`: never pipe cargo through tail/head/grep when checking success — capture full output to a log, check exit code, tail the log separately.

**No e2e gate required** — this fix-impl is clamp-only + markdown-only. The cp-* clamps are inside cron handlers that are disabled in tests via `BREHON_DISABLE_PARTICIPATION_JOB=1` (per `scheduled_tasks.rs:488`); cr-12 is markdown that doesn't compile. cp-* behaviour is exercised by existing v1-RT-r3 idempotency tests at the next full e2e run (validates default-value paths; clamping is observed-only via the `warn!`).

If a gate fails, **STOP and surface to advisor via DQ.** Do not `#[allow]`-spam clippy denials.

## 6. Expected output (return to advisor)

```
## Task fix-impl-3 complete — v1-RT-r3 cp-1+cp-2+cp-3 clamp group + cr-12 retro escape

**Commits:** <sha-1> (clamp) + <sha-2> (retro) on <worktree-branch>
**Files changed:**
  - crates/routes/src/utils/scheduled_tasks.rs (cp-1 clamp + warn block)
  - crates/api/api/src/governance/participation_cron.rs (cp-2 lookback_days + activity_threshold; cp-3 dormancy_window_days)
  - .claude/PRPs/reports/session-retro-2026-05-26-pr-155-cr-triage-junior-479-fail.md (cr-12 row 4 pipe escape)
**Validation:** check / clippy both exit 0
**DQ entry:** `kind: validate-pending-laptop` with workflow_run_id null (Shape-G suspended)
**Next:** advisor laptop runs cargo, mutates DQ, then `/bm-poll-cr 155` (promote cp-1/2/3/cr-12 to done) → `/bm-merge 155`
```

## 7. Why this brief differs from the plan (if applicable)

This is a fix-impl brief, not a §13-plan task. No plan-step substitution. Sources: PR #155 CR triage + Copilot findings as bucketed in `.claude/PRPs/reviews/pr-155-findings.yaml` and approved by user at `/bm-triage 155` outbound user-gate (2026-05-28 ~15:50 UTC).

## 8. Bundling

Bundle of 2 commits in one Junior task. Per `feedback_bundle_means_one_worker_branch_not_one_commit.md`: ONE worker branch, ONE workspace-check workflow, ONE ci-watcher cycle, **2 commits** (one per logical group). Per-commit subjects per §2.1 / §2.2.

**Compile-success between bundled commits:** Commit 1 (Rust clamps) is compile-clean in isolation; the `warn!` macro is already in scope (verify import in `participation_cron.rs` — add `use tracing::warn;` if absent). Commit 2 is markdown-only, no compile impact. Order is non-negotiable: Commit 1 → Commit 2.
