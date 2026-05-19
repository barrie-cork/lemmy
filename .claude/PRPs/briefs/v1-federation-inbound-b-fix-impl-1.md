---
phase: v1-federation-inbound-b
role: impl-task
kind: fix-impl
fix_impl_n: 1
authored: 2026-05-19
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
triggering_dq: 283
triggering_task: 4
classification: "NON-ALLOWLIST §15 compile failure on Task-4's enforcement core, USER-AUTHORISED narrow fix-forward (Option A via advisor catch-fire surface 2026-05-19). NOT a §G4 mechanical-paste row (the 3 errors are E0277 Sync-bound + E0599 receiver-type-mismatch, neither matches an allowlist row verbatim). Hand-authored but TIGHTLY bounded: the canonical fix pattern is in the SAME FILE (Phase-6 sites @193/301 use `&mut get_conn(pool)` = `&mut DbConn` for run_transaction) + a codebase precedent (crates/api/api/src/governance/sponsor_liability_grace.rs:281 `conn: &mut DbConn<'_>`). 4 edits, 1 file, ZERO caller changes. feedback_plan_stub_uniformity_with_canonical_sibling class: Task-4's 2 new tx-starting helpers took `conn: &mut AsyncPgConnection` (raw, no run_transaction method) instead of the pooled `&mut DbConn<'_>` the working Phase-6 siblings use; + the wrapper generic A needs +Sync for the async_trait method (compiler-dictated verbatim)."
base: "phase-v1-federation-inbound-b @ a3757eabe (Task 4 #336 done+merged-lossless; all 7 §10.4 symbols present + Phase-6 surgical mods @195/303/761; §15 by4d7chba FAILED 3 compile errors; DQ #283 validate-pending-laptop result:null pending)"
cap: "EXACTLY 4 edits, 1 file ONLY: crates/apub/activities/src/governance/inbox.rs. (1) add `use lemmy_diesel_utils::connection::DbConn;` adjacent to the existing line-104 `use ...::DbPool;`. (2) line 491 `  A: GovernanceInboundActivity,` → `  A: GovernanceInboundActivity + std::marker::Sync,`. (3) line 606 (in `log_inbox_drop`) `  conn: &mut AsyncPgConnection,` → `  conn: &mut DbConn<'_>,`. (4) line 656 (in `evict_oldest_unreviewed_if_needed`) `  conn: &mut AsyncPgConnection,` → `  conn: &mut DbConn<'_>,`. NEVER change lines 390/404 (`insert_remote_sanction_notice`/`insert_federation_attestation` conn params — those are Phase-6 helpers called INSIDE a tx closure; `&mut AsyncPgConnection` is CORRECT there). NEVER change any caller (callers @195/303/761 evict, @510/525/552/581 log_inbox_drop already pass `&mut get_conn(pool).await?` = `&mut DbConn`). NEVER touch crates/** elsewhere, schema_setup, Cargo.*, the plan, e2e.rs, any test. NEVER add #[allow]."
serial: "Single-task barrier fix (Task 4). Strictly serial cap=1 — only in-flight Junior for this lane. §15 (DQ #283 — the 3 cmds cargo-check + cargo-clippy -D warnings + cargo-test --test e2e --no-run) is re-run by the ADVISOR on the laptop AFTER this fix lands + finalize-merge; Task 4 advances (DQ #283 → result:pass) only on that re-validation. Cohort B (Tasks 5-7 [P]) stays gated behind Task 4 + the separate serena/rust-analyzer OOM mitigation."
---

# [role:impl-task] v1-federation-inbound-b fix-impl-1 — inbox.rs DbConn receiver + Sync bound (Option A) — see .claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-1.md

> **Provenance:** Task 4 (#336) shipped the enforcement core to `phase-v1-federation-inbound-b` @ `a3757eabe` (verified lossless: all 7 §10.4 symbols + Phase-6 surgical mods). The advisor-run §15 on the laptop (3 cmds, bg by4d7chba) **FAILED** with 3 compile errors in `crates/apub/activities/src/governance/inbox.rs`, all the same root cause. **User chose Option A** (advisor catch-fire surface 2026-05-19): narrow advisor-authored fix-forward. Classification: **NON-ALLOWLIST, user-authorised** — hand-authored recipe, NOT a §G4 mechanical paste. But the fix is TIGHTLY bounded because the correct pattern is proven IN THE SAME FILE.

## §0 Pre-flight (subagent runs this FIRST, before reading anything else)

- Confirm CWD is a Junior worktree branched off `phase-v1-federation-inbound-b` (base tip `a3757eabe`). `git merge-base --is-ancestor a3757eabe HEAD` MUST be true. If on `phase-v1-federation-inbound-b` itself or a non-`junior/*` branch → STOP, file `kind: "blocker"` DQ into `.claude/decision-queue.json` (NOT a repo-root file unless the §4 harness-gap fires).
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — inside a forbidden window → exit non-zero `FORBIDDEN_WINDOW: <window>` unless dispatch carries `(user-authorised forbidden-window override per DQ #<id>)`. Shape G SUSPENDED (cargo on laptop) — keep the check.
- **Submodule init (MANDATORY before any in-worker cargo — per `feedback_worktree_submodules_not_auto_init`):** `git submodule update --init 2>&1` from worktree root. `crates/email/translations` is NOT auto-initialized in a fresh worktree; without it cargo fails pre-existing. Infra, not the fix.
- Confirm the 4 target anchors are present + unmodified at base: `grep -nE "use lemmy_diesel_utils::connection::DbPool|^  A: GovernanceInboundActivity,$|^pub\(crate\) async fn log_inbox_drop|^async fn evict_oldest_unreviewed_if_needed" crates/apub/activities/src/governance/inbox.rs` MUST show line 104 (DbPool import), line 491 (`  A: GovernanceInboundActivity,`), line 600 (`log_inbox_drop` fn), line 652 (`evict_oldest_unreviewed_if_needed` fn). If line numbers drift, grep the anchors (not blind line edits) — see §2.3. If `log_inbox_drop`/`evict_oldest_unreviewed_if_needed` are absent → STOP, file `kind: "blocker"` (base mismatch — Task 4 not on this tip).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b fix-impl-1 — inbox.rs DbConn receiver + Sync bound`

Actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-b fix-impl-1 — see .claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-1.md
```

## §2 Scope

### 2.1 The failure being fixed (the contract)

Advisor-run §15 (laptop, lane worktree, tip `a3757eabe`) — `cargo-check.bat --workspace --features full` FAILED, identical errors in `cargo-clippy` and `cargo-test --test e2e --no-run`:

```
error[E0277]: `A` cannot be shared between threads safely
   --> crates\apub\activities\src\governance\inbox.rs:565:12
565 |   activity.check_per_actor_rate_limit(context).await?;
note: required by a bound in `GovernanceInboundActivity::check_per_actor_rate_limit`
help: consider further restricting type parameter `A` with trait `Sync`
    |
491 |   A: GovernanceInboundActivity + std::marker::Sync,

error[E0599]: no method named `run_transaction` found for mutable reference `&mut AsyncPgConnection` in the current scope
   --> crates\apub\activities\src\governance\inbox.rs:620:6      (inside fn log_inbox_drop)
error[E0599]: no method named `run_transaction` found for mutable reference `&mut AsyncPgConnection` in the current scope
   --> crates\apub\activities\src\governance\inbox.rs:687:6      (inside fn evict_oldest_unreviewed_if_needed)

error: could not compile `lemmy_apub_activities` (lib) due to 3 previous errors
```

### 2.2 Root cause (confirmed by reading the working Phase-6 siblings in the SAME file)

`run_transaction` is a method on **`DbConn`** (`crates/diesel_utils/src/connection.rs`: `pub enum DbConn<'a>` line 53, `impl DbConn<'_>` line 67), NOT on raw `&mut AsyncPgConnection`. `DbConn` impls `Deref/DerefMut` → `AsyncPgConnection` (connection.rs:82/93).

The **working Phase-6 sites in the same inbox.rs** prove the pattern: `receive_remote_sanction_notice` (line ~192) does `let conn = &mut get_conn(pool).await?;` (→ `conn: &mut DbConn`) then `conn.run_transaction(|conn| {...}).await`. Every working `run_transaction` caller in this file (lines 193, 301, 421, 505, 758) holds `&mut DbConn` from `get_conn(pool)`.

Task-4's 2 NEW tx-starting helpers — `log_inbox_drop` (fn @600, param @606) and `evict_oldest_unreviewed_if_needed` (fn @652, param @656) — declared `conn: &mut AsyncPgConnection` (the RAW connection) and then call `conn.run_transaction(...)`. `run_transaction` is not in scope on the raw type → E0599. The callers ALL already pass `&mut get_conn(pool).await?` = `&mut DbConn` (evict callers @195/303/761; log_inbox_drop callers @510/525/552/581) — so **only the 2 helper param TYPES are wrong; callers are correct and need no change**. The `.execute(conn)` / `.get_result::<CountRow>(conn)` calls *inside* the helpers' transaction closures use the closure-provided `conn` (the tx's `&mut AsyncPgConnection`) and are unaffected by the param-type change (DbConn derefs anyway).

The E0277 is independent + compiler-dictated: `wrap_governance_inbound`'s generic `A` is bounded only `A: GovernanceInboundActivity` (line 491), but the `#[async_trait]` method `check_per_actor_rate_limit` (called @565) requires `A: Sync`. The compiler prints the exact patch.

**Codebase precedent for the helper signature**: `crates/api/api/src/governance/sponsor_liability_grace.rs:281` declares `conn: &mut DbConn<'_>` — the canonical signature for a helper that starts a transaction.

### 2.3 The fix (EXACTLY 4 edits, 1 file — grep the anchors, do not blind-edit line numbers)

All edits in `crates/apub/activities/src/governance/inbox.rs`:

1. **Add the `DbConn` import.** The file already has (grep to confirm exact line, ~104):
   `use lemmy_diesel_utils::connection::DbPool;`
   Add immediately adjacent (same module path, alphabetical with the existing `get_conn`/`DbPool` imports from `lemmy_diesel_utils::connection`):
   `use lemmy_diesel_utils::connection::DbConn;`
   (`get_conn` is imported at ~72, `DbPool` at ~104 — both from `lemmy_diesel_utils::connection`; add `DbConn` from the same module.)

2. **E0277 Sync bound.** Grep `^  A: GovernanceInboundActivity,$` (the `where`-clause line in `wrap_governance_inbound`, ~491). Change it to:
   `  A: GovernanceInboundActivity + std::marker::Sync,`
   (Verbatim the compiler's `help:` suggestion. Do NOT add `Send` or other bounds — only `+ std::marker::Sync`.)

3. **`log_inbox_drop` conn param.** In `pub(crate) async fn log_inbox_drop(` (fn ~600), the param line (~606) is `  conn: &mut AsyncPgConnection,`. Change ONLY that line to:
   `  conn: &mut DbConn<'_>,`

4. **`evict_oldest_unreviewed_if_needed` conn param.** In `async fn evict_oldest_unreviewed_if_needed(` (fn ~652), the param line (~656) is `  conn: &mut AsyncPgConnection,`. Change ONLY that line to:
   `  conn: &mut DbConn<'_>,`

**DO NOT** change the `conn: &mut AsyncPgConnection,` params at ~390 (`insert_remote_sanction_notice`) and ~404 (`insert_federation_attestation`) — those are Phase-6 helpers invoked INSIDE a `run_transaction` closure; `&mut AsyncPgConnection` is CORRECT for them (they receive the tx's connection, they do NOT start a tx). Changing them would break the working Phase-6 paths. There are exactly 4 `conn: &mut AsyncPgConnection,` lines in this file (390, 404, 606, 656); you change ONLY 606 and 656.

### 2.4 Acceptance (the worker proves these before reporting done)

- `grep -c "conn: &mut AsyncPgConnection," crates/apub/activities/src/governance/inbox.rs` returns **2** (only 390 + 404 remain — the Phase-6 in-tx helpers).
- `grep -n "conn: &mut DbConn<'_>," crates/apub/activities/src/governance/inbox.rs` shows **2** new lines (the 2 fixed helpers ~606/656).
- `grep -n "A: GovernanceInboundActivity + std::marker::Sync," crates/apub/activities/src/governance/inbox.rs` shows **1** line (~491).
- `grep -c "use lemmy_diesel_utils::connection::DbConn;" crates/apub/activities/src/governance/inbox.rs` returns **1**.
- §4.2 pre-push `cargo-check.bat --workspace --features full` is GREEN (the whole point — the 3 errors must clear and nothing new must break). Capture + verify the explicit marker (not the bg notification).

## §3 Required reading (IN ORDER before editing)

1. `crates/diesel_utils/src/connection.rs` lines **50-115** — `pub enum DbConn<'a>` (53), `get_conn` (58, returns `DbConn<'a>`), `impl DbConn<'_>` (67, where `run_transaction` lives), `Deref/DerefMut` (82/93). Confirms `run_transaction` requires `&mut DbConn`, and `DbConn` derefs to `AsyncPgConnection` (so inner `.execute(conn)` still works).
2. `crates/apub/activities/src/governance/inbox.rs` lines **185-215** (the WORKING Phase-6 `receive_remote_sanction_notice` `run_transaction` site: `let conn = &mut get_conn(pool).await?;` then `conn.run_transaction`) + **595-700** (the 2 BROKEN helpers `log_inbox_drop` ~600 / `evict_oldest_unreviewed_if_needed` ~652 and their `run_transaction` calls) + **485-495** (the `wrap_governance_inbound` `where` clause + the E0277 bound) + **560-570** (the `check_per_actor_rate_limit` call @565). This is the contract — mirror the Phase-6 receiver shape.
3. `crates/api/api/src/governance/sponsor_liability_grace.rs:281` — codebase precedent: `conn: &mut DbConn<'_>` helper signature.
4. `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md` — **PRIMARY**. New code must mirror the working sibling's type shape verbatim; the §10.4 stub diverged (raw `&mut AsyncPgConnection` where the sibling uses `&mut DbConn`).
5. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — error-shape discipline (the helpers return `LemmyResult<()>`; the param-type change must not perturb the `?`/error propagation — it won't, this is a receiver-type fix only).
6. `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` — §4.2 mandatory pre-push cargo-check + the explicit-marker rule.

## §4 Constraints (HARD — violation = STOP + kind:blocker DQ)

1. **One commit.** Subject: `fix(v1-federation-inbound-b): inbox.rs helpers take &mut DbConn for run_transaction + Sync bound on wrapper generic (fix-impl 1)`. Body: list the exactly-4 edits + the grep-acceptance proof + the pre-push cargo-check-green proof.
2. **Pre-push cargo-check (per `feedback_fix_impl_pre_push_cargo_check`):** before pushing the worker branch, run `cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-fi1-precheck.log 2>&1 && echo FI1_PRECHECK_EXIT_0 >> .claude/PRPs/debug/fed-in-b-fi1-precheck.log || echo FI1_PRECHECK_EXIT_NONZERO >> .claude/PRPs/debug/fed-in-b-fi1-precheck.log"`; verify the EXPLICIT `FI1_PRECHECK_EXIT_*` marker in the file (the bg task-notification has lied repeatedly this phase — `feedback_background_task_notification_lies`). Non-zero → STOP, file `kind: "blocker"` DQ with the failing slice. NEVER `#[allow]`-spam to force green.
3. **DQ writes go into `.claude/decision-queue.json`** (the lane file at the worktree `.claude/` path). Compute `next_id = max(all ids across pending+resolved)+1` from `.claude/decision-queue.json` + `.claude/decision-queue-archive-*.json` (current max is 283 → next 284). Commit + push the DQ entry on the worker branch immediately (mid-task visibility). **Harness-gap (per DQ #235):** IF the `.claude/decision-queue.json` write is blocked by the Claude Code sensitive-file gate, write the JSON to a worktree-root `FI1_BLOCKER_DQ.json` + a short `FI1_ESCALATION.md` + commit both + push + STOP; advisor transcribes. (This applies ONLY if you must raise a blocker — the happy path has NO `.claude/` write: you do NOT raise a validate-pending entry; DQ #283 already exists and the advisor re-runs §15 + mutates it post-merge.)
4. **File-ownership:** edits ONLY to `crates/apub/activities/src/governance/inbox.rs`, EXACTLY the 4 changes in §2.3. NEVER any other file, NEVER `schema_setup`, NEVER `Cargo.*`, NEVER `e2e.rs`/tests, NEVER the plan/ADRs, NEVER add `#[ignore]` or `#[allow]`.
5. **MIRROR-ref discipline:** the working Phase-6 `run_transaction` sites in the same file (~193/301) + `sponsor_liability_grace.rs:281` are the authoritative shape. If the compiler, after your 4 edits, still errors (e.g. a lifetime on `DbConn<'_>` needs naming), fix MINIMALLY toward what the working siblings do — do NOT introduce new abstractions or touch callers. If a 5th edit seems required, STOP and file `kind: "blocker"` DQ with the new error (the cap is 4 by design; a 5th means the diagnosis missed something — surface, don't improvise).
6. **Attribution:** worker `from: "impl"`; never `answered_by: "advisor"|"user"`; never `kind: "clarify"|"validate-pending"|"validate-result"|"validate-failed"` (you raise NO validate entry — DQ #283 is the advisor's to re-validate).
7. **Serial:** cap=1 — only in-flight Junior for this lane.

## §5 What "done" looks like

One commit on a `junior/*` worktree branch off `a3757eabe` making EXACTLY the 4 edits in §2.3 to `crates/apub/activities/src/governance/inbox.rs`, with §2.4 grep-acceptance satisfied and §4.2 pre-push `cargo-check --workspace --features full` GREEN (the 3 original errors cleared, nothing new broken). No DQ entry raised on the happy path. The advisor then finalize-merge-reconciles the worker branch lane-safe (FF-verify into `phase-v1-federation-inbound-b`), re-runs the full §15 3-cmd chain (DQ #283's commands) on the laptop, and on all-green mutates DQ #283 `result: "pass"` → Task 4 complete → Cohort B gate (separately gated on the serena/rust-analyzer OOM mitigation per swap-death retro eval 424/425).

## §6 Why this is non-allowlist but narrow

The 3 errors do not match a §G4 allowlist row verbatim (E0277 here is a generic `Sync` bound, not a LemmyError-conversion case A/B/C; E0599 is a receiver-type mismatch, not the allowlist's "missing `use` for a re-exported trait"). Per advisor-orchestrator.md §5.3 that classifies non-allowlist → catch-fire; the user reviewed the catch-fire surface and chose Option A (narrow fix-forward). It is hand-authored (this brief) NOT a §G4 mechanical paste — but it is tightly bounded: the correct pattern is proven in the same file (Phase-6 `run_transaction` receiver) + a codebase precedent, the diagnosis is complete, and the cap is exactly 4 edits in 1 file with zero caller changes. A 5th edit or any caller change is a hard STOP (the diagnosis would have missed something — surface, do not improvise).

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; per DQ #279 the canonical session drives fed-in-b under serial-phase policy). Mirrors the canonical fix-impl brief schema `.claude/PRPs/briefs/federation-inbound-a-fix-impl-5.md` per `.claude/rules/advisor-orchestrator.md` §3.6. Committed on `governance-v0` before the Junior fix-impl task is queued; then propagated onto `phase-v1-federation-inbound-b` (conflict-free trunk-merge in the lane worktree) so the worker — branched from the phase tip `a3757eabe` (Task 4 merged) — sees it. /precheck re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC (= a3757eabe-or-later) before queue. User-authorised Option A (catch-fire surface 2026-05-19, §15 by4d7chba failure)._
