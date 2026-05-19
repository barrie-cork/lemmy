---
phase: v1-federation-inbound-b
role: impl-task
kind: fix-impl
fix_impl_n: 2
authored: 2026-05-19
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
triggering_dq: 283
triggering_task: 4
classification: "§15 clippy-only failure (`-D warnings`) on Task-4's enforcement core AFTER fix-impl-1 cleared the 3 compile errors. 7 errors: 6× `dead-code` (EXPECTED pre-landed infra — Task 4 DEFINES the trait/statics/wrapper/log_inbox_drop; their callers arrive in Cohort B Tasks 5-7 which `requires: task 4` per plan §13) + 1× `clippy::redundant_closure_for_method_calls` @inbox.rs:546 (trivial mechanical; also pre-flagged by the 2026-05-19 conformance audit Finding 4.2). USER chose Option A + SEPARATE (2026-05-19, advisor surface): suppress the 6 pre-landed-infra dead-code via per-item `#[expect(dead_code, reason=...)]` (the codebase's sanctioned pattern — precedent crates/server/tests/e2e.rs:1923 `#[expect(dead_code, reason = ...)]`) + fix the redundant-closure. NOT bundled with Finding 6.1 (that is a SEPARATE fix-impl-3, gated behind this one's §15-green re-validation). NOT a §G4 mechanical-paste row, but tightly bounded + zero-design-ambiguity: every target is named with an exact line + exact text below."
base: "phase-v1-federation-inbound-b @ cdff6f09d (fix-impl-1 #337 done+merged-lossless: `chore(merge): finalize fix-impl-1 ... inbox helpers &mut DbConn + Sync bound`; the 3 compile errors cleared — §15 reval beleyxn8f: cargo-check CHECK_EXIT_0 ✓, cargo-test --test e2e --no-run TESTNORUN_EXIT_0 ✓, cargo-clippy CLIPPY_EXIT_NONZERO ✗ = the 7 errors this brief fixes; DQ #283 validate-pending-laptop result:null still pending)"
cap: "EXACTLY 7 edits, 1 file ONLY: crates/apub/activities/src/governance/inbox.rs. SIX `#[expect(dead_code, reason = \"pre-landed federation-inbound enforcement infra; callers wired by Cohort B Tasks 5-7 (impl GovernanceInboundActivity + wrap_governance_inbound call sites), which declare `requires: task 4` per plan §13\")]` attribute lines, EACH inserted on its OWN new line IMMEDIATELY ABOVE (no blank line between) the declaration, at these exact anchors: (1) above L443 `pub(crate) trait GovernanceInboundActivity: Sized {` — placed AFTER the existing `#[async_trait::async_trait]` line that precedes it (attribute order: `#[async_trait::async_trait]` then `#[expect(dead_code, reason=...)]` then the `pub(crate) trait`); (2) above L464 `pub(crate) fn rate_per_peer_counts() ...`; (3) above L470 `pub(crate) fn rate_per_actor_counts() ...`; (4) above L476 `pub(crate) fn current_hour_bucket() -> i64 {`; (5) above L484 `pub(crate) async fn wrap_governance_inbound<F, Fut, A>(`; (6) above L601 `pub(crate) async fn log_inbox_drop(`. PLUS ONE edit: L546 `      .unwrap_or_else(|e| e.into_inner());` → `      .unwrap_or_else(std::sync::PoisonError::into_inner);` (preserve the exact 6-space indent). NEVER add `#[expect(dead_code)]` to `evict_oldest_unreviewed_if_needed` (L653) — it is CALLED by log_inbox_drop, it is REACHABLE, clippy did NOT flag it; suppressing it would mask a future real dead-code signal. NEVER `#[allow]` (use `#[expect]` — the codebase convention; `#[allow]` would itself trip clippy::allow_attributes once the code is wired in Cohort B). NEVER change line numbers blind — grep the exact declaration text. NEVER touch any other file, crates/** elsewhere, schema_setup, Cargo.*, the plan, e2e.rs, any test, any caller. NEVER touch Finding 6.1 (receive_remote_moderation_label ~:735 .domain()...unwrap_or_default()) — that is fix-impl-3."
serial: "Single-task barrier fix (Task 4), strictly serial cap=1 — only in-flight Junior for this lane. §15 (DQ #283 — 3 cmds: cargo-check + cargo-clippy --no-deps -- -D warnings + cargo-test --test e2e --no-run) is re-run by the ADVISOR on the laptop AFTER this fix lands + finalize-merge. Task 4 advances only when ALL 3 §15 cmds pass. fix-impl-3 (Finding 6.1) is queued SERIALLY ONLY AFTER this fix-impl-2 §15-re-validates green (same file inbox.rs ⇒ serial mandatory regardless). Cohort B (Tasks 5-7 [P]) stays gated behind Task-4-complete (= fix-impl-2 §15-green AND fix-impl-3 §15-green) + the separate serena/rust-analyzer OOM mitigation (swap-death retro eval 424/425)."
---

# [role:impl-task] v1-federation-inbound-b fix-impl-2 — §15 mechanical: #[expect(dead_code)] ×6 pre-landed infra + redundant-closure @inbox.rs:546 — see .claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-2.md

> **Provenance:** fix-impl-1 (#337) merged to `phase-v1-federation-inbound-b` @ `cdff6f09d` and cleared the 3 compile errors (E0277 Sync + E0599 ×2 run_transaction). The advisor re-ran §15 on the laptop (bg `beleyxn8f`): `cargo-check` GREEN, `cargo-test --test e2e --no-run` GREEN, but `cargo-clippy --workspace --features full --no-deps -- -D warnings` **FAILED with 7 errors** (`crates/apub/activities/src/governance/inbox.rs`): **6× `dead-code`** + **1× `clippy::redundant_closure_for_method_calls`**. The 6 dead-code are **EXPECTED, NOT A BUG** — Task 4 *defines* the enforcement infra (`wrap_governance_inbound`, the `GovernanceInboundActivity` trait, the 3 rate-counter accessors, `log_inbox_drop`); its callers are **Cohort B Tasks 5-7**, each of which rewrites a `publish_*.rs` handler body to a one-line `wrap_governance_inbound(...).await` + adds `impl GovernanceInboundActivity for <type>` and **declares `requires: task 4`** (plan §13 lines 89/135/137/206/209/1178-1195/1447-1458). They are correctly unused until Cohort B wires them — the pre-landed-infra pattern. **User chose Option A + SEPARATE** (advisor surface 2026-05-19): suppress via per-item `#[expect(dead_code, reason=...)]` (codebase-sanctioned — sole precedent `crates/server/tests/e2e.rs:1923`) + fix the trivial redundant-closure; Finding 6.1 stays a **separate fix-impl-3** gated behind this one's §15-green. Classification: **§G4-adjacent mechanical, user-authorised, zero-design-ambiguity** — every target is named with an exact line + exact verbatim text in §2.3.

## §0 Pre-flight (subagent runs this FIRST, before reading anything else)

- Confirm CWD is a Junior worktree branched off `phase-v1-federation-inbound-b` (base tip `cdff6f09d`). `git merge-base --is-ancestor cdff6f09d HEAD` MUST be true. If on `phase-v1-federation-inbound-b` itself or a non-`junior/*` branch → STOP, file `kind: "blocker"` DQ into `.claude/decision-queue.json` (NOT a repo-root file unless the §4 harness-gap fires).
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — inside a forbidden window → exit non-zero `FORBIDDEN_WINDOW: <window>` unless dispatch carries `(user-authorised forbidden-window override per DQ #<id>)`. Shape G SUSPENDED (cargo on laptop) — keep the check.
- **Submodule init (MANDATORY before any in-worker cargo — per `feedback_worktree_submodules_not_auto_init`):** `git submodule update --init 2>&1` from worktree root. `crates/email/translations` is NOT auto-initialized in a fresh worktree; without it cargo fails pre-existing. Infra, not the fix.
- Confirm the 7 target anchors are present + unmodified at base. Run this grep block; the line numbers MUST match (if they drift by ±a few lines, grep the exact declaration text — do NOT blind-edit; if a declaration is ABSENT, STOP + `kind:"blocker"` base-mismatch):
  ```
  grep -nE "^pub\(crate\) trait GovernanceInboundActivity: Sized \{|^pub\(crate\) fn rate_per_peer_counts\(\)|^pub\(crate\) fn rate_per_actor_counts\(\)|^pub\(crate\) fn current_hour_bucket\(\) -> i64 \{|^pub\(crate\) async fn wrap_governance_inbound<F, Fut, A>\(|^pub\(crate\) async fn log_inbox_drop\(" crates/apub/activities/src/governance/inbox.rs
  ```
  Expected: L443 (trait), L464 (rate_per_peer_counts), L470 (rate_per_actor_counts), L476 (current_hour_bucket), L484 (wrap_governance_inbound), L601 (log_inbox_drop).
  Also confirm the redundant-closure anchor: `grep -n "unwrap_or_else(|e| e.into_inner())" crates/apub/activities/src/governance/inbox.rs` MUST return **exactly ONE** line (~L546). If it returns 0 or >1 → STOP + `kind:"blocker"` (premise changed — diagnosis missed a twin or fix-impl-1 already touched it).
- Confirm the `#[async_trait::async_trait]` precedes the trait: `grep -nB1 "^pub(crate) trait GovernanceInboundActivity: Sized {" crates/apub/activities/src/governance/inbox.rs` — the line immediately above L443 MUST be `#[async_trait::async_trait]`. The new `#[expect(dead_code, reason=...)]` goes BETWEEN that attribute and the trait keyword (attribute stacking; order: `#[async_trait::async_trait]` then `#[expect(...)]` then `pub(crate) trait`).
- Confirm no pre-existing dead-code suppression: `grep -cE "expect\(dead_code|allow\(dead_code" crates/apub/activities/src/governance/inbox.rs` MUST return **0**. If >0, the file already has suppressions — STOP + `kind:"blocker"` (premise changed; another fix touched it).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b fix-impl-2 — #[expect(dead_code)] ×6 pre-landed infra + redundant-closure`

Actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-b fix-impl-2 — see .claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-2.md
```

## §2 Scope

### 2.1 The failure being fixed (the contract — verbatim from the advisor's §15 reval clippy log, tip `cdff6f09d`)

```
error: trait `GovernanceInboundActivity` is never used
   --> crates\apub\activities\src\governance\inbox.rs:443:18
    = note: `-D dead-code` implied by `-D warnings`
    = help: to override `-D warnings` add `#[expect(dead_code)]` or `#[allow(dead_code)]`
error: function `rate_per_peer_counts` is never used
   --> crates\apub\activities\src\governance\inbox.rs:464:15
error: function `rate_per_actor_counts` is never used
   --> crates\apub\activities\src\governance\inbox.rs:470:15
error: function `current_hour_bucket` is never used
   --> crates\apub\activities\src\governance\inbox.rs:476:15
error: function `wrap_governance_inbound` is never used
   --> crates\apub\activities\src\governance\inbox.rs:484:21
error: function `log_inbox_drop` is never used
   --> crates\apub\activities\src\governance\inbox.rs:601:21
error: redundant closure
   --> crates\apub\activities\src\governance\inbox.rs:546:23
    | .unwrap_or_else(|e| e.into_inner());
    | help: replace the closure with the method itself: `std::sync::PoisonError::into_inner`
    = note: requested on the command line with `-D clippy::redundant-closure-for-method-calls`
error: could not compile `lemmy_apub_activities` (lib) due to 7 previous errors
CLIPPY_EXIT_NONZERO
```

### 2.2 Root cause (confirmed by reading plan §13 + the file)

The **6 dead-code are NOT defects** — they are the intended pre-landed enforcement infra. Plan §13 (lines 89, 135, 137, 206, 209, 1178-1195, 1447-1458): Cohort B **Tasks 5/6/7** each (a) replace a `publish_{sanction_notice,trust_attestation,label}.rs` handler body with a single `wrap_governance_inbound(self, context, |a, c| async move { … }).await` call and (b) add an `impl GovernanceInboundActivity for <Type>` block — and all three declare `requires: task 4`. Until Cohort B lands, the trait + wrapper + rate-counter accessors + `log_inbox_drop` are *defined but uncalled*. With `-D dead-code` (via `-D warnings`) that is a hard clippy error on Task-4-in-isolation. The codebase-sanctioned override is `#[expect(dead_code, reason = "…")]` (per the `-D warnings` help text + sole in-repo precedent `crates/server/tests/e2e.rs:1923` `#[expect(dead_code, reason = "struct fields accessed via Diesel QueryableByName reflection")]`). `#[expect]` (not `#[allow]`) is correct: when Cohort B wires the code in, `#[expect(dead_code)]` becomes a *fulfilled* expectation that clippy then flags as unfulfilled-removable — i.e. it self-documents the temporary nature and forces removal once the code is live (whereas a stale `#[allow]` would silently persist; and bare `#[allow]` itself trips `clippy::allow_attributes` in this workspace per `feedback_clippy_test_style`).

`evict_oldest_unreviewed_if_needed` (L653) is **NOT in the clippy list** — it is called by `log_inbox_drop`, so it is reachable through `log_inbox_drop`'s body even though `log_inbox_drop` itself is currently uncalled (clippy's dead-code analysis reports the *root* uncalled item, not its transitive callees). Adding `#[expect(dead_code)]` to `evict_oldest_unreviewed_if_needed` would be WRONG: once Cohort B wires `log_inbox_drop`, `evict_oldest_unreviewed_if_needed` becomes reachable and a stray `#[expect(dead_code)]` on it would be an unfulfilled expectation (clippy error) — and more importantly it was never flagged, so suppressing it masks a real future signal. Suppress EXACTLY the 6 the compiler named.

The **redundant-closure @L546** is a trivial mechanical lint: `.unwrap_or_else(|e| e.into_inner())` is a one-arg passthrough closure equivalent to the method path `std::sync::PoisonError::into_inner`. clippy's own `help:` gives the exact replacement. There is **exactly one** such site in the file (grep-confirmed: one `.unwrap_or_else(|e| e.into_inner())`, one `.lock()` chain, one `|x| x.method()` passthrough total — no twin, so no fix-impl-2b risk).

### 2.3 The fix (EXACTLY 7 edits, 1 file — grep the anchors, do NOT blind-edit line numbers)

All edits in `crates/apub/activities/src/governance/inbox.rs`. The `#[expect]` attribute text is **identical for all 6** (copy it verbatim):

```
#[expect(dead_code, reason = "pre-landed federation-inbound enforcement infra; callers wired by Cohort B Tasks 5-7 (impl GovernanceInboundActivity + wrap_governance_inbound call sites), which declare requires: task 4 per plan §13")]
```

1. **Above `trait GovernanceInboundActivity` (~L443).** This trait is preceded by `#[async_trait::async_trait]`. Insert the `#[expect(...)]` line BETWEEN `#[async_trait::async_trait]` and `pub(crate) trait GovernanceInboundActivity: Sized {` (so the order is: `#[async_trait::async_trait]` / `#[expect(dead_code, reason = "…")]` / `pub(crate) trait GovernanceInboundActivity: Sized {`). Preserve column 0 (no indent — these are module-level items).

2. **Above `pub(crate) fn rate_per_peer_counts() ...` (~L464).** Insert the `#[expect(...)]` line immediately above it (no blank line between). Check the 1-2 lines above L464 are a `// ----` comment / blank — the attribute goes on the line directly above the `pub(crate) fn`, after any doc/banner comment.

3. **Above `pub(crate) fn rate_per_actor_counts() ...` (~L470).** Same — attribute line immediately above the `pub(crate) fn`.

4. **Above `pub(crate) fn current_hour_bucket() -> i64 {` (~L476).** Same.

5. **Above `pub(crate) async fn wrap_governance_inbound<F, Fut, A>(` (~L484).** Same — attribute line immediately above; if a `// ----` banner comment precedes it, the attribute goes between the comment and the `pub(crate) async fn` (attributes must be adjacent to the item, comments may precede the attribute).

6. **Above `pub(crate) async fn log_inbox_drop(` (~L601).** Same.

7. **Redundant-closure @L546.** The line is exactly `      .unwrap_or_else(|e| e.into_inner());` (6-space indent, inside the `rate_per_peer_counts().lock()` chain). Change ONLY that line to:
   `      .unwrap_or_else(std::sync::PoisonError::into_inner);`
   (Preserve the exact 6-space leading indent and the trailing `;`. Verbatim the clippy `help:` replacement. Do NOT touch the `.lock()` on L545 or the surrounding block L543-548.)

**HARD: do NOT add `#[expect(dead_code)]` to `evict_oldest_unreviewed_if_needed` (~L653)** — it was NOT flagged (reachable via `log_inbox_drop`); suppressing it is wrong (see §2.2). **Do NOT touch `receive_remote_moderation_label` / its `.domain()...unwrap_or_default()` (~L735)** — that is Finding 6.1, a SEPARATE fix-impl-3. **Do NOT add any other attribute, change any signature, or touch any caller.**

### 2.4 Acceptance (the worker proves these before reporting done)

- `grep -c "expect(dead_code, reason = \"pre-landed federation-inbound" crates/apub/activities/src/governance/inbox.rs` returns **6** (the 6 new attributes, all identical text).
- `grep -nB1 "^pub(crate) trait GovernanceInboundActivity: Sized {" crates/apub/activities/src/governance/inbox.rs` shows the line directly above is the new `#[expect(dead_code, reason = "pre-landed federation-inbound…")]`, and ITS line above is `#[async_trait::async_trait]` (3-line stack intact).
- `grep -c "expect(dead_code" crates/apub/activities/src/governance/inbox.rs` returns **6** (NOT 7 — `evict_oldest_unreviewed_if_needed` untouched). `grep -c "allow(dead_code" ...` returns **0**.
- `grep -n "unwrap_or_else(std::sync::PoisonError::into_inner)" crates/apub/activities/src/governance/inbox.rs` shows **1** line (~L546); `grep -c "unwrap_or_else(|e| e.into_inner())" ...` returns **0**.
- §4.2 pre-push `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings` is **GREEN** (the 7 errors must clear and nothing new must trip). ALSO re-run `cargo-check.bat --workspace --features full` GREEN (sanity — attributes + a method-path swap must not perturb compilation). Capture + verify the EXPLICIT markers (not the bg notification — it has lied repeatedly this phase).

## §3 Required reading (IN ORDER before editing)

1. `crates/apub/activities/src/governance/inbox.rs` lines **440-490** (the trait + 3 rate-counter accessors + `wrap_governance_inbound` signature — the 5 module-level dead-code anchors, and the `#[async_trait::async_trait]` that precedes the trait) + **595-660** (`log_inbox_drop` ~601 and `evict_oldest_unreviewed_if_needed` ~653 — confirm the latter is called BY the former and is NOT in the clippy list) + **540-550** (the `rate_per_peer_counts().lock().unwrap_or_else(|e| e.into_inner())` chain — the redundant-closure site). This is the contract.
2. `crates/server/tests/e2e.rs:1923` — the **sole in-repo precedent** for `#[expect(dead_code, reason = "…")]`. Mirror its attribute shape (per-item, with a `reason = "…"` string).
3. `.claude/PRPs/plans/v1-federation-inbound-b.plan.md` §13 — read lines ~85-95, ~130-140, ~1175-1200, ~1445-1460 (Cohort B Tasks 5/6/7 each rewrite a `publish_*.rs` body to `wrap_governance_inbound(...).await` + add `impl GovernanceInboundActivity`, all `requires: task 4`). This is WHY the 6 are pre-landed infra, not dead code — it justifies the `reason = "…"` text.
4. `.claude/lessons/feedback_clippy_test_style.md` — the workspace clippy discipline: `#[allow]` itself trips `clippy::allow_attributes`; `#[expect]` is the sanctioned form; `--no-deps -- -D warnings` is the gate.
5. `.claude/lessons/feedback_clippy_rerun_after_fix.md` — clippy reports first-error-then-aborts per crate; after these 7 fixes, re-run the FULL clippy cmd to confirm no NEXT-layer lint surfaced (the §4.2 pre-push gate does this).
6. `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` — §4.2 mandatory pre-push cargo gate + the explicit-marker rule (bg notification unreliable).

## §4 Constraints (HARD — violation = STOP + kind:blocker DQ)

1. **One commit.** Subject: `fix(v1-federation-inbound-b): #[expect(dead_code)] on 6 pre-landed enforcement-infra items (Cohort-B-wired) + redundant-closure @inbox.rs:546 (fix-impl 2)`. Body: list the exactly-7 edits (6 anchors + the closure line) + the §2.4 grep-acceptance proof + the §4.2 pre-push clippy-green + cargo-check-green proof.
2. **Pre-push cargo gate (per `feedback_fix_impl_pre_push_cargo_check`):** before pushing the worker branch, run BOTH (clippy is the failing gate; check is the sanity gate), each with an explicit marker, and verify the marker in the FILE (the bg task-notification has lied repeatedly this phase — `feedback_background_task_notification_lies`):
   ```
   cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-fi2-clippy.log 2>&1 && echo FI2_CLIPPY_EXIT_0 >> .claude/PRPs/debug/fed-in-b-fi2-clippy.log || echo FI2_CLIPPY_EXIT_NONZERO >> .claude/PRPs/debug/fed-in-b-fi2-clippy.log"
   cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-fi2-check.log 2>&1 && echo FI2_CHECK_EXIT_0 >> .claude/PRPs/debug/fed-in-b-fi2-check.log || echo FI2_CHECK_EXIT_NONZERO >> .claude/PRPs/debug/fed-in-b-fi2-check.log"
   ```
   Either marker NONZERO → STOP, file `kind: "blocker"` DQ with the failing slice (≤120 lines). NEVER `#[allow]`-spam or add extra suppressions to force green — if clippy flags something OTHER than the 7 named, that is a NEW finding the advisor must triage (surface via blocker, do not absorb it into this fix).
3. **DQ writes go into `.claude/decision-queue.json`** (the lane file at the worktree `.claude/` path). Compute `next_id = max(all ids across pending+resolved)+1` from `.claude/decision-queue.json` + `.claude/decision-queue-archive-*.json` (current max is 283 → next 284). Commit + push the DQ entry on the worker branch immediately (mid-task visibility). **Harness-gap (per DQ #235):** IF the `.claude/decision-queue.json` write is blocked by the Claude Code sensitive-file gate, write the JSON to a worktree-root `FI2_BLOCKER_DQ.json` + a short `FI2_ESCALATION.md` + commit both + push + STOP; advisor transcribes. (This applies ONLY if you must raise a blocker — the happy path has NO `.claude/` write: you do NOT raise a validate-pending entry; DQ #283 already exists and the advisor re-runs §15 + mutates it post-merge.)
4. **File-ownership:** edits ONLY to `crates/apub/activities/src/governance/inbox.rs`, EXACTLY the 7 changes in §2.3. NEVER any other file, NEVER `schema_setup`, NEVER `Cargo.*`, NEVER `e2e.rs`/tests, NEVER the plan/ADRs, NEVER add `#[ignore]`, NEVER `#[allow]`, NEVER touch `evict_oldest_unreviewed_if_needed` or `receive_remote_moderation_label`.
5. **MIRROR-ref discipline:** the precedent `crates/server/tests/e2e.rs:1923` (`#[expect(dead_code, reason = "…")]` per-item shape) + the clippy `help:` text (`std::sync::PoisonError::into_inner`) are the authoritative shapes. If, after your 7 edits, clippy flags a NEW lint (e.g. an `#[expect]` placement that confuses `clippy::allow_attributes` or a doc-comment-vs-attribute ordering issue), fix MINIMALLY toward the precedent shape — but if it requires an 8th edit or touching a non-§2.3 item, STOP and file `kind: "blocker"` DQ with the new error (the cap is 7 by design; an 8th means the diagnosis missed something — surface, don't improvise).
6. **Attribution:** worker `from: "impl"`; never `answered_by: "advisor"|"user"`; never `kind: "clarify"|"validate-pending"|"validate-result"|"validate-failed"` (you raise NO validate entry — DQ #283 is the advisor's to re-validate).
7. **Serial:** cap=1 — only in-flight Junior for this lane. fix-impl-3 (Finding 6.1) is NOT yet queued; it is gated behind this fix-impl-2's §15-green re-validation.

## §5 What "done" looks like

One commit on a `junior/*` worktree branch off `cdff6f09d` making EXACTLY the 7 edits in §2.3 to `crates/apub/activities/src/governance/inbox.rs` (6 identical `#[expect(dead_code, reason = "pre-landed federation-inbound…")]` attributes on the 6 named items + the `.unwrap_or_else(std::sync::PoisonError::into_inner)` swap @~L546), with §2.4 grep-acceptance satisfied and §4.2 pre-push `cargo-clippy ... -D warnings` GREEN and `cargo-check` GREEN. No DQ entry raised on the happy path. The advisor then finalize-merge-reconciles the worker branch lane-safe (FF-verify into `phase-v1-federation-inbound-b`), re-runs the full §15 3-cmd chain (DQ #283's commands) on the laptop, and on all-three-green proceeds to author + dispatch fix-impl-3 (Finding 6.1). Task 4 is complete only when BOTH fix-impl-2 AND fix-impl-3 land with §15 green after each; DQ #283 → `result: "pass"` mutated only after fix-impl-3's §15 fully passes (the LAST §15 run for Task 4). Cohort B (Tasks 5-7 [P]) stays gated behind Task-4-complete + the serena/rust-analyzer OOM mitigation (swap-death retro eval 424/425).

## §6 Why this is mechanical + zero-design-ambiguity (not a re-litigation)

The 6 dead-code suppressions are NOT a judgment call: plan §13 explicitly schedules the callers in Cohort B with `requires: task 4` — the infra is *intended* to land before its callers. The `#[expect(dead_code, reason=...)]` is the codebase's sole sanctioned pattern for this (precedent e2e.rs:1923; `-D warnings` help text names it). The redundant-closure is clippy's own verbatim suggestion. Every target is named with an exact line + exact text. The ONLY way this goes wrong is a blind line-edit (mitigated: grep the declaration text, §0 + §2.3) or scope-creep onto `evict_oldest_unreviewed_if_needed`/Finding-6.1 (mitigated: explicit HARD exclusions). User chose Option A + SEPARATE after the advisor's catch-fire surface (2026-05-19); the Phase-6-conformance *interpretation* (whether the §10.4 plan stub should have pre-included the `#[expect]` + the `+Sync` bound — it did NOT, a plan-stub-uniformity defect) is a **retro deliverable** per `project_phase6_convention_divergence_class.md`, NOT this fix's concern. This fix just makes Task 4 independently §15-green.

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; per DQ #279 the canonical session drives fed-in-b under serial-phase policy). Mirrors the canonical fix-impl brief schema `.claude/PRPs/briefs/federation-inbound-a-fix-impl-5.md` + the phase-specific shape of `.claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-1.md` per `.claude/rules/advisor-orchestrator.md` §3.6. Anchors verified at the live fix-impl-1 tip `cdff6f09d` in the lane worktree `C:/Users/barri/Developer/brehon-fork-fed-in-b` before authoring (trait@443, rate_per_peer_counts@464, rate_per_actor_counts@470, current_hour_bucket@476, wrap_governance_inbound@484, log_inbox_drop@601, redundant-closure@546-single-site-no-twin, evict_oldest_unreviewed_if_needed@653-reachable-NOT-flagged, zero pre-existing dead_code suppressions). Committed on `governance-v0` before the Junior fix-impl task is queued; then propagated onto `phase-v1-federation-inbound-b` (conflict-free trunk-merge in the lane worktree) so the worker — branched from the phase tip `cdff6f09d` (fix-impl-1 merged) — sees it. /precheck re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC (= cdff6f09d-or-later) before queue. User-authorised Option A + SEPARATE (advisor surface 2026-05-19, §15 reval beleyxn8f clippy failure)._
