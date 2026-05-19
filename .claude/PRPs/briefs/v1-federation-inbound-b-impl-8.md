---
phase: v1-federation-inbound-b
role: impl-task
task: 8
brief_n: 1
authored: 2026-05-20
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
plan_task: "§13 Task 8 BARRIER — scheduled_tasks.rs: APPEND replay-cleanup cron block per §10.8"
parent_phase_tip: 31cf49164 (Cohort B Tasks 5/6/7 ALL COMPLETE + §15-green: publish_sanction_notice/publish_trust_attestation/publish_label all wrapped+trait-impl'd; DQ #285/286/287 all resolved pass; fix chain: fix-impl-4 HRTB + fix-impl-5 E0283 + fix-impl-6 clippy items-after-statements+unfulfilled-expectation)
cohort: "BARRIER (not [P]) — Task 8 stands alone between Cohort B (Tasks 5-7) and Task 9 (handler-e2e). requires: task 4 (replay-cleanup target = federation_inbox_nonce rows, which Task 4's wrap_governance_inbound writes via Phase-6 receive_remote_* handlers calling the existing replay-check helper)."
related_dq: "229 (Shape G suspended), 250 (delete_older_than is pub async fn — NO dead_code suppression needed), 285/286/287 (Cohort B §15 pass), 235 (no .claude/** writes by Junior — harness gap)"
---

# [role:impl-task] v1-federation-inbound-b Task 8 — scheduled_tasks.rs replay-cleanup cron wiring — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-8.md

> **Provenance:** Cohort B (Tasks 5/6/7) complete + §15-green at 31cf49164; fix chain history fix-impl-4/5/6 (HRTB wrap sig + E0283 .into() drop + clippy items-after-statements + dead_code strip). Task 8 is the next BARRIER per plan §13 Task 8 / §16a Story 4 — appends a `federation_inbox_nonce` replay-cleanup cron tick to `scheduled_tasks.rs` mirroring the existing `appeal_window_expiry` hourly pattern. Single file, append-only block, no compile/lint risk classes pre-identified.

## §0 Pre-flight (subagent runs this FIRST, before reading anything else)

- Confirm CWD branch is `phase-v1-federation-inbound-b` (you are on a Junior worktree branched off it). If `git branch --show-current` is anything else → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — inside a forbidden window → exit non-zero `FORBIDDEN_WINDOW: <window>`. Shape G SUSPENDED (cargo on laptop).
- **Submodule init (MANDATORY before any in-worker cargo — per `feedback_worktree_submodules_not_auto_init`):** `git submodule update --init 2>&1` from worktree root. `crates/email/translations` is NOT auto-initialized in a fresh worktree.
- **`requires: task 4` precondition self-verify:** Task 8 cleans up `federation_inbox_nonce` rows written by Task-4's `wrap_governance_inbound` (via Phase-6 receive_remote_* handlers' replay-check). Confirm Task 4 + Cohort B all merged: `git log phase-v1-federation-inbound-b --oneline --grep '(task 4)' --grep '(task 5)' --grep '(task 6)' --grep '(task 7)' -i | head -10` — must show feat/fix commits for tasks 4-7. If task 4 missing → STOP + `kind: "blocker"`.
- Confirm the two mirror anchors are present (the reference patterns):
  ```
  grep -n "appeal_window_expiry\|APPEAL_WINDOW_EXPIRY_RUNNING\|SPONSOR_LIABILITY_GRACE_RUNNING\|sponsor_liability_grace" crates/routes/src/utils/scheduled_tasks.rs | head -10
  ```
  Expected: `APPEAL_WINDOW_EXPIRY_RUNNING` ~L260, `SPONSOR_LIABILITY_GRACE_RUNNING` ~L308, both blocks in the file. The append point is AFTER the `sponsor_liability_grace` closing block (~line :360, just after its `});` scheduler chain).
- Confirm `delete_older_than` is present in lemmy_db_schema (per DQ #250 GOTCHA):
  ```
  grep -n "pub async fn delete_older_than" crates/db_schema/src/source/governance/federation_inbox_nonce.rs
  ```
  Must return ≥1. If missing → STOP + `kind: "blocker"` (Task-1/2 chain incomplete).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b Task 8 — scheduled_tasks.rs: APPEND replay-cleanup cron block per §10.8`

Actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-b Task 8 — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-8.md
```

## §2 Scope

**Produce** (one commit) — **single file, `crates/routes/src/utils/scheduled_tasks.rs`** (plan §13 Task 8 FILES YAML: `modifies: [crates/routes/src/utils/scheduled_tasks.rs]`, `creates: []`):

Per plan **§10.8 verbatim** (the contract — copy it exactly):

1. **APPEND a new scheduler block** AFTER the existing `sponsor_liability_grace` block (which ends ~line :360 with its closing `});` for the scheduler chain).
2. The new block contains:
   - A static `AtomicBool` guard `FED_REPLAY_CLEANUP_RUNNING` (concurrency sentinel mirroring `APPEAL_WINDOW_EXPIRY_RUNNING` / `SPONSOR_LIABILITY_GRACE_RUNNING`).
   - A `FedReplayCleanupRunningGuard` struct + `Drop` impl that releases the AtomicBool on drop.
   - A `let context_fed_replay = context.reset_request_count();` clone for the scheduler closure.
   - Reading `federation.inbound.replay_cleanup_cron_interval_minutes` from `governance_config` (default 60) via `lemmy_api::governance::config::get_int` against `Scope::Instance`.
   - A `scheduler.every(CTimeUnits::minutes(fed_replay_interval_minutes)).run(move || async move { ... });` block.
   - Inside the async move:
     - Check `BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB=1` env var (test-disable, mirror of `BREHON_DISABLE_GRACE_CHECK_JOB`); early-return if set.
     - `compare_exchange` on `FED_REPLAY_CLEANUP_RUNNING` (skip if a prior tick still running, warn log).
     - Bind `_guard = FedReplayCleanupRunningGuard;`.
     - Read `federation.inbound.replay_window_days` from `governance_config` (default 7).
     - `get_conn` from pool.
     - Call `lemmy_db_schema::source::governance::federation_inbox_nonce::delete_older_than(window_days_i64, &mut conn)`.
     - Log info if `deleted > 0`; log warn on get_conn error.

3. **Apply plan §10.8 VERBATIM** for the actual code text — do NOT paraphrase the closure body, the config keys (`federation.inbound.replay_cleanup_cron_interval_minutes` + `federation.inbound.replay_window_days`), the env var name (`BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB`), or the `delete_older_than` call path.

**Do NOT** in this task:

- Touch `inbox.rs` (Tasks 4 / fix-impl-4/6 territory), `publish_sanction_notice.rs` (Task 5), `publish_trust_attestation.rs` (Task 6/fix-impl-5/6), `publish_label.rs` (Task 7), `federation_inbox_nonce.rs` (Tasks 1-2 created it — `delete_older_than` is already there per DQ #250), `e2e.rs` (Task 9).
- Modify or re-define `lemmy_api::governance::config::get_int`, `Scope::Instance`, `ConfigCache`, or any `lemmy_diesel_utils::connection::get_conn` API. They exist on the phase tip; the block USES them.
- Add a `#[allow(dead_code)]` or `#[expect(dead_code)]` anywhere — `delete_older_than` is `pub async fn` in lemmy_db_schema (lib crate); Rust does NOT emit `dead_code` for `pub` items in lib crates, so no suppression is needed (per DQ #250 GOTCHA in plan).
- Re-read or alter the `appeal_window_expiry` or `sponsor_liability_grace` blocks — they are MIRROR references, do NOT touch them.

**Commit message** (exactly): `feat(v1-federation-inbound-b): scheduled_tasks — replay-cleanup cron wiring (task 8)`

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #229 (Shape G suspended; advisor runs §15 on laptop), DQ #250 (`delete_older_than` is `pub async fn` in lib crate — NO `#[expect(dead_code)]` needed), DQ #285/286/287 (Cohort B §15 pass — confirms wrap call sites compile + lint clean against the merged Task-4 enforcement core, so the cron block can call `delete_older_than` from a separate file without additional setup).
2. **Plan §10.8** — the authoritative replay-cleanup cron block (copy verbatim; this is the contract). The plan-cited line `:360` for the append point is plan-time; verify with grep.
3. **Plan §13 Task 8** — step list + FILES YAML + the GOTCHAs (DQ #250 + test-isolation env var).
4. **`crates/routes/src/utils/scheduled_tasks.rs:260-360`** (READ-ONLY, for the mirror patterns):
   - `appeal_window_expiry` block ~L260-288 (closest mirror — hourly default, atomic-bool guard, env-var test-disable).
   - `sponsor_liability_grace` block ~L308-360 (append-AFTER target — appends after its closing `});`).
   - The static `APPEAL_WINDOW_EXPIRY_RUNNING` + the `*RunningGuard` struct shape — your `FED_REPLAY_CLEANUP_RUNNING` + `FedReplayCleanupRunningGuard` mirror these exactly.
5. **`crates/db_schema/src/source/governance/federation_inbox_nonce.rs`** (READ-ONLY, just to confirm `delete_older_than` exists with the expected signature: `pub async fn delete_older_than(window_days: i64, conn: &mut AsyncPgConnection) -> Result<usize, DbError>` or similar — Tasks 1-2 created this fn).
6. **Lessons** (MANDATORY per `.claude/rules/advisor-orchestrator.md` §2.4 — Task 8 runs `cargo-clippy.bat ... -D warnings`; `scheduled_tasks.rs` is a routes/util file):
   - `.claude/lessons/feedback_clippy_test_style.md` + `.claude/lessons/feedback_clippy_rerun_after_fix.md` — **Why:** clippy denies `unwrap`/`expect`/`#[allow]`; the new cron block must be clippy-clean under `-D warnings`. The plan §10.8 block uses `.unwrap_or(60)` / `.unwrap_or(7)` / `.unwrap_or(0)` on already-converted i64 — these are NOT `Result::unwrap()` (which would fail clippy) but `Result::unwrap_or(default)` and `i64::try_from(...).unwrap_or(default)` which return defaults on conversion failure — clippy-safe.
   - `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Why:** §10.8 uses `.await.unwrap_or(60)` etc on config reads (so no `?` propagation through the closure — no LemmyError bridging). The closure body uses `match` + `warn!` for errors. No `.into()` on `LemmyErrorType` (which was the fix-impl-5 defect in Task 6) — verify by grep before commit.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — capture-then-tail for §5 commands).

## §3a Handover from prior cohort

> **Handover state: DEGRADED** (per `.claude/rules/advisor-orchestrator.md` §4.3 — Cohort B chain commits carry no `HANDOVER:` YAML trailer). Synthesized by the advisor from the Cohort B public-API deliverable. This is NOT a catch-fire; the §0 grep self-verifies the actual symbols on the phase tip.

```yaml
prior_cohort_tasks:
  - task: 4 (chain)
    commits: [a3757eabe, cdff6f09d, f01a1d44e, 4a60667c9, fix-impl-4 = 3bd2cfa4e]
    filesModified: [crates/apub/activities/src/governance/inbox.rs]
    keyDecisions:
      - "wrap_governance_inbound + GovernanceInboundActivity trait + 5 helpers (rate_per_peer_counts, rate_per_actor_counts, current_hour_bucket, log_inbox_drop, get_inbound_config_int) landed; fix-impl-4 added <'a> HRTB to wrap sig + stripped its #[expect(dead_code)]."
  - task: 5
    commit: 92605e907 (impl + DQ #285 mutate 88f0f03c8)
    filesModified: [crates/apub/activities/src/governance/publish_sanction_notice.rs]
    keyDecisions:
      - "publish_sanction_notice §10.5 wrap + trait-impl (trait-default rate gate); §15-green at 88f0f03c8."
  - task: 6 (chain)
    commits: [ec3b8643f (impl), d32d2c3f2 (fix-impl-5 .into() drop), 9656b4bc7 (fix-impl-6 const hoist + dead_code strip), 065a24ce7 (finalize-merge), b29b18874 (DQ #286 mutate)]
    filesModified: [crates/apub/activities/src/governance/publish_trust_attestation.rs, crates/apub/activities/src/governance/inbox.rs (annotation strip only)]
    keyDecisions:
      - "publish_trust_attestation §10.5/§10.6 wrap + trait-impl with per-actor override calling rate_per_actor_counts/current_hour_bucket/log_inbox_drop; cap key federation.inbound.max_payload_bytes_trust_attestation; per-actor cap key federation.inbound.per_actor_attestation_rate_per_hour. inbox.rs rate_per_actor_counts #[expect(dead_code)] stripped by fix-impl-6."
  - task: 7
    commits: [fe9effd6b (impl), 4c18c12d9 (finalize-merge), 31cf49164 (DQ #287 mutate)]
    filesModified: [crates/apub/activities/src/governance/publish_label.rs]
    keyDecisions:
      - "publish_label stub-fill: REPLACE Phase-6 STUB Activity::receive with one-line wrap_governance_inbound delegation to crate::governance::inbox::receive_remote_moderation_label + impl GovernanceInboundActivity for PublishLabel with trait-default rate gate; cap key federation.inbound.max_payload_bytes_moderation_label. §15-green first try (cleanest Cohort-B task)."
    notes: "Cohort B closed; §16a Story 3 complete. Task 8 BARRIER scope: append replay-cleanup cron block to scheduled_tasks.rs per §10.8. CALLS the existing lemmy_db_schema::source::governance::federation_inbox_nonce::delete_older_than (created in Tasks 1-2 per DQ #250 — `pub async fn` in lib crate, no dead_code suppression needed). Reads governance_config keys via existing lemmy_api::governance::config::get_int + Scope::Instance. Uses test-isolation env var BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB mirror of BREHON_DISABLE_GRACE_CHECK_JOB. Mirror patterns are appeal_window_expiry (~L260-288, closest hourly pattern) + sponsor_liability_grace (~L308-360, append-AFTER target)."
```

## §4 Constraints (hard rules)

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-federation-inbound-b` (tip `31cf49164`, Cohort B complete). Finalize merges your worktree branch back; do NOT push to `phase-v1-federation-inbound-b` directly.
- ONE commit (single file). If clippy fails first attempt, amend/fixup — do NOT split.
- Mid-task DQ visibility: if you raise a `pending` entry mid-task, **commit + push immediately**.
- No `answered_by: "advisor"` or `"user"`. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235)

If you need to write the §5 `validate-pending-laptop` DQ entry and the sensitive-file gate blocks it: write `TASK8_VALIDATE_PENDING.json` + `TASK8_ESCALATION.md` at worktree root, commit both, STOP. The advisor transcribes. Task-5/6/7 workers RAISED DQ inline OK (no harness-gap fired since Task-4 chain) — so likely OK here too.

### Task-8 GOTCHAs (from plan §13 Task 8 + §10.8 — all load-bearing)

- **Append-only** AFTER the `sponsor_liability_grace` block's closing `});` (~L360). The two prior blocks (appeal_window_expiry L260-288, sponsor_liability_grace L308-360) STAY UNCHANGED. Do NOT modify them.
- **Static `FED_REPLAY_CLEANUP_RUNNING: AtomicBool` + `FedReplayCleanupRunningGuard` struct + `Drop` impl** mirror the existing patterns exactly (function/scope-local to the cron-wiring function — likely the `setup` or `start_scheduled_tasks` fn that contains the other blocks). Their placement: just BEFORE the new `let context_fed_replay = context.reset_request_count();` line, INSIDE the same fn body, at module/fn scope as the other guards.
- **Config keys are EXACT strings** (per §10.8): `"federation.inbound.replay_cleanup_cron_interval_minutes"` (interval; default 60) and `"federation.inbound.replay_window_days"` (window; default 7). Do NOT paraphrase / abbreviate.
- **Env var name is EXACT**: `"BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB"` (mirror of `BREHON_DISABLE_GRACE_CHECK_JOB`). The test-disable check is `std::env::var("BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB").as_deref() == Ok("1")` — verbatim per §10.8.
- **`delete_older_than` call path**: `lemmy_db_schema::source::governance::federation_inbox_nonce::delete_older_than(window_days_i64, &mut conn)`. NO `dead_code` annotation needed (DQ #250 — `pub async fn` in lib crate; Rust does NOT emit `dead_code` for `pub` items in lib crates).
- **NO `.into()` on `LemmyErrorType`** anywhere in the closure (canonical-sibling-mirror discipline carries forward from fix-impl-5). The closure body uses `.await.unwrap_or(N)` on config reads (returns the default on Err) and `match conn_result { Ok(...) => ..., Err(e) => warn!(...) }` for connection errors. No `?` propagation.
- **`u32::try_from(i64).unwrap_or(60)`** is clippy-safe (it's `Result::unwrap_or(default)`, not `Result::unwrap()`).

### Plan-cited line numbers may have drifted

§10.8 is the contract for *content*. The `:360` (append-AFTER) / `:260-288` (appeal_window_expiry mirror) / `:308-360` (sponsor_liability_grace mirror) line cites are plan-time. `grep -n "appeal_window_expiry\|sponsor_liability_grace\|APPEAL_WINDOW_EXPIRY_RUNNING\|SPONSOR_LIABILITY_GRACE_RUNNING" crates/routes/src/utils/scheduled_tasks.rs` for real positions. If the file no longer matches §10.8's documented "mirror" shape, file a DQ pending entry — the plan may have drifted.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

**Shape-G suspended until 2026-06-01** (DQ #229). After committing + pushing your worktree branch, write a `kind: "validate-pending-laptop"` DQ entry with `from: "impl"`, `phase_task: 8`, `branch: <your-worktree-branch>`, and `commands[]` set **verbatim** to (per plan §13 Task 8 "Push and exit" — exactly 2 commands, NO `cargo-test --no-run` for Task 8):

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task8-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task8-clippy.log 2>&1"
```

Do NOT write `kind: "validate-pending"`. Do NOT capture a `workflow_run_id`. The advisor laptop session reads this entry and runs the 2 commands locally; mutates the entry on completion.

If the sensitive-file gate fires, use the §4 harness-gap path with `TASK8_VALIDATE_PENDING.json`.

## §6 Expected output (return to advisor)

```
## Task 8 complete — v1-federation-inbound-b scheduled_tasks replay-cleanup cron wiring

**Commit:** <sha> on <worktree-branch>
**File changed:** crates/routes/src/utils/scheduled_tasks.rs (APPEND-only after sponsor_liability_grace block ~L360; +<N> lines, 0 deletions)
**PRECON self-check:** appeal_window_expiry + sponsor_liability_grace blocks present at ~L260-288 + L308-360; delete_older_than present in federation_inbox_nonce.rs (per DQ #250)
**§10.8 verbatim:** static FED_REPLAY_CLEANUP_RUNNING + FedReplayCleanupRunningGuard + scheduler.every(CTimeUnits::minutes(...)).run(...) block appended; config keys "federation.inbound.replay_cleanup_cron_interval_minutes" + "federation.inbound.replay_window_days"; env var "BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB"; delete_older_than call path verbatim
**No #[expect(dead_code)] added** (per DQ #250 GOTCHA — pub async fn in lib crate)
**No .into() on LemmyErrorType** (per fix-impl-5 canonical-sibling-mirror discipline)
**No edits to inbox.rs / publish_*.rs / federation_inbox_nonce.rs / e2e.rs:** confirmed
**validate-pending-laptop DQ:** #<id> raised (commands: cargo-check.bat + cargo-clippy.bat -D warnings — 2 cmds, no e2e-norun)
**Next:** advisor laptop runs §15 (2 cmds), mutates DQ #<id>; on pass → Task 9 dispatched (handler-e2e barrier).
```

Plus any DQ #N references if you raised a blocker mid-task.

## §7 Why this brief differs from the plan

It does not — Task 8's scope is exactly plan §10.8 (verbatim block) + §13 Task 8. This brief adds only: (a) §0 forbidden-window + `requires: task 4` precondition self-verifies + mirror-anchor + delete_older_than grep, (b) §4 harness-gap escalation path (likely NOT to fire since Task-5/6/7 workers raised inline OK; but covered for safety) + the explicit GOTCHAs from §13 Task 8 + §10.8 + lessons from fix-impl-5/6 (no spurious `.into()`; no items-after-statements; if the new block declares a const, it must precede statements), (c) §5 explicit `validate-pending-laptop` shape with the 2-command set (NO `cargo-test --test e2e --no-run` — Task 9 handles e2e), (d) explicit "plan line numbers may have drifted — grep the named symbols" GOTCHA. The replay-cleanup cron block is §10.8 verbatim — do not deviate. Mirrors the canonical sibling `.claude/PRPs/briefs/v1-federation-inbound-b-impl-7.md` schema per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate (same §0-§7 brief STRUCTURE; same single-file BARRIER discipline as Task 8 is BARRIER post-Cohort-B).

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`). Brief committed on `governance-v0` first; then cherry-picked / re-committed onto `phase-v1-federation-inbound-b` so the worker — which branches from the PHASE branch tip `31cf49164` (Cohort B complete + §15-green) — sees it. `/precheck` re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC before queue. Task 8 BARRIER (not [P]); cohort cap=1 still enforced under user cap-≤2 OOM choice. Handover from Cohort B is DEGRADED (no HANDOVER: trailer on the chain) — synthesized in §3a from the Cohort B public-API deliverable._
