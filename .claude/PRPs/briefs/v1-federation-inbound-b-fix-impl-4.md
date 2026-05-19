---
phase: v1-federation-inbound-b
role: impl-task
kind: fix-impl
fix_impl_n: 4
authored: 2026-05-19
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
triggering_dq: 285
triggering_task: 5
classification: "Latent HRTB defect in Task-4 `wrap_governance_inbound` sig (inbox.rs:486 / plan §10.4 line 882), surfacing at first caller (Task 5 — publish_sanction_notice.rs:104). Compile error E0521-class: `lifetime may not live long enough` — `'1 must outlive '2`. Worker copied plan §10.5 verbatim (`|a, c| async move { receive(a,c).await }`); §10.5 is correct; §10.4 wrap-sig is the defect — missing HRTB tying `Fut`'s lifetime to the `&Data<LemmyContext>` borrow. Latent because Task-4's `#[expect(dead_code)]` shielded wrap from compiler call-site exercise through the Task-4 chain (fix-impl-1/2/3 GREEN; §15 cmd-3 e2e --no-run GREEN; DQ #283 resolved pass). Task 5 became the first caller → HRTB inference fails (Fut concretely typed before `'1` bound). USER chose Option A (2026-05-19): fix Task-4 wrap sig in inbox.rs. **Non-§G4-allowlist** (not unresolved-import/doc/map_err/E0277-LemmyError-bridge); hand-authored mechanical fix (canonical HRTB pattern, zero design ambiguity)."
base: "phase-v1-federation-inbound-b @ fcc0e7de0 (Task 5 worker-merge tip — publish_sanction_notice.rs wrap+trait-impl per §10.5 verbatim; DQ #285 raised pending; §15 cmd-1 FAILED on this tip with the HRTB error). The wrap-sig fix lands as a fix-impl on this tip; Task-5's publish_sanction_notice.rs commit (92605e907) stays UNCHANGED — it is correct per §10.5."
cap: "EXACTLY 1 hunk (≤8 lines), 1 file ONLY: crates/apub/activities/src/governance/inbox.rs. (a) Change the wrap-sig from `pub(crate) async fn wrap_governance_inbound<F, Fut, A>` to `pub(crate) async fn wrap_governance_inbound<'a, F, Fut, A>` (introduce explicit lifetime parameter `'a`). (b) Change the `context: &Data<LemmyContext>` parameter type to `context: &'a Data<LemmyContext>`. (c) Change the `where` clause bounds: `F: FnOnce(A, &'a Data<LemmyContext>) -> Fut`, `Fut: std::future::Future<Output = LemmyResult<()>> + 'a`, `A: GovernanceInboundActivity + std::marker::Sync + 'a`. (d) REMOVE the `#[expect(dead_code, reason = ...)]` line (485) immediately above `pub(crate) async fn wrap_governance_inbound` — Task 5 (publish_sanction_notice.rs:104) now calls it, so the dead_code expectation is unfulfilled and must be stripped (per workspace `allow_attributes = \"deny\"` + Rust dead_code chain-pruning per fix-impl-3 retro). NEVER touch the GovernanceInboundActivity trait, the other `#[expect(dead_code)]` sites (rate_per_actor_counts L470 — Cohort B Task 6 caller still pending, keep this annotation), receive_remote_*, log_inbox_drop, the gate-body, or any other line/fn/file. NEVER touch crates/** elsewhere, schema_setup, Cargo.*, the plan, e2e.rs, any test, the publish_*.rs callers, or .claude/PRPs/plans/v1-federation-inbound-b.plan.md. A 2nd hunk or any other-file edit → STOP + kind:blocker. **Override of Task-5 brief's 'no inbox.rs edit' guard:** explicit per this DQ — the guard exists to prevent worker scope creep into Task-4 territory during a Task-5 wiring; a planning-defect fix-impl on Task-4's sig surfaced post-merge is a different shape (advisor-scoped fix-impl, not worker drift)."
serial: "Single-task barrier fix (Task 5 §15 — the FIRST and ONLY fix-impl on this Cohort-B kickoff), strictly serial cap=1 — only in-flight Junior for this lane. §15 (DQ #285 — 2 cmds: cargo-check + cargo-clippy --no-deps -- -D warnings; NO e2e --no-run for Task 5) is re-run by the ADVISOR on the laptop AFTER this fix lands + finalize-merge. Task 5 is COMPLETE only when this fix's §15 passes both cmds; THEN advisor mutates DQ #285 result:pass. Cohort B's remaining serial dispatch (Task 6 → §15-green → Task 7) stays gated behind Task-5-complete per user's cap-≤2 OOM choice."
---

# [role:impl-task] v1-federation-inbound-b fix-impl-4 — wrap_governance_inbound HRTB lifetime parameter (`<'a>`) — see .claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-4.md

> **Provenance:** Task 5 (#340, worker commit 92605e907, finalize-merge fcc0e7de0) follows plan §10.5 VERBATIM — publish_sanction_notice.rs:104 receive body = `wrap_governance_inbound(self, context, |a, c| async move { receive_remote_sanction_notice(a, c).await }).await`. Advisor §15 cmd-1 `cargo-check.bat --workspace --features full` FAILED at this caller with `error: lifetime may not live long enough` (publish_sanction_notice.rs:104:77, `'1` must outlive `'2`). Root cause = plan §10.4 wrap-sig (inbox.rs:486) missing HRTB tying `Fut`'s lifetime to the `&Data<LemmyContext>` borrow; latent because Task-4's `#[expect(dead_code)]` shielded wrap from compiler call-site exercise. **USER chose Option A** (2026-05-19): fix Task-4 wrap sig in inbox.rs (single hunk, one file). DQ #285 stays pending until this fix's §15 (2 cmds) passes.

## §0 Pre-flight (subagent runs this FIRST, before reading anything else)

- Confirm CWD is a Junior worktree branched off `phase-v1-federation-inbound-b` (base tip `fcc0e7de0`). `git merge-base --is-ancestor fcc0e7de0 HEAD` MUST be true. If on `phase-v1-federation-inbound-b` itself or a non-`junior/*` branch → STOP, file `kind: "blocker"` DQ into `.claude/decision-queue.json` (NOT a repo-root file unless the §4 harness-gap fires).
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — inside a forbidden window → exit non-zero `FORBIDDEN_WINDOW: <window>` unless dispatch carries `(user-authorised forbidden-window override per DQ #<id>)`. Shape G SUSPENDED (cargo on laptop) — keep the check.
- **Submodule init (MANDATORY before any in-worker cargo — per `feedback_worktree_submodules_not_auto_init`):** `git submodule update --init 2>&1` from worktree root. `crates/email/translations` is NOT auto-initialized in a fresh worktree; without it cargo fails pre-existing. Infra, not the fix.
- Confirm the 2 anchors are present + at the expected shape (grep the text, line numbers approximate — base tip fcc0e7de0):
  ```
  grep -n "pub(crate) async fn wrap_governance_inbound" crates/apub/activities/src/governance/inbox.rs
  grep -n "wrap_governance_inbound" crates/apub/activities/src/governance/publish_sanction_notice.rs
  ```
  Expected: wrap sig declared once in inbox.rs ~L486; Task-5 caller in publish_sanction_notice.rs ~L104 (the §10.5-verbatim closure call). If wrap sig appears 0 or >1 times → STOP + `kind: "blocker"` (target moved). If publish_sanction_notice.rs has no caller → STOP + `kind: "blocker"` (Task-5 commit missing — base tip wrong).
- Confirm the HRTB defect is still present (un-fixed) at base: `grep -A 4 "pub(crate) async fn wrap_governance_inbound" crates/apub/activities/src/governance/inbox.rs | grep -E "<F, Fut, A>|<'a, F, Fut, A>"` — if it returns `<F, Fut, A>` (no `'a`) → defect present, proceed. If it returns `<'a, F, Fut, A>` → STOP + `kind: "blocker"` (premise changed — already fixed).
- Confirm the `#[expect(dead_code)]` on wrap is still present: `grep -B 1 "pub(crate) async fn wrap_governance_inbound" crates/apub/activities/src/governance/inbox.rs | grep -c "expect(dead_code"` MUST return `1`. If 0 → STOP + `kind: "blocker"` (already stripped — half-fixed state).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b fix-impl-4 — wrap_governance_inbound HRTB (<'a> lifetime param) — fix Task-5 §15 cmd-1 fail`

Actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-b fix-impl-4 — see .claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-4.md
```

## §2 Scope

### 2.1 The defect being fixed (the contract — Task 5 §15 cmd-1 fail, verified by advisor 2026-05-19)

In `crates/apub/activities/src/governance/inbox.rs`, `wrap_governance_inbound` (Task-4 wrapper, fn at ~L486) has a generic sig MISSING an explicit lifetime parameter binding the `&Data<LemmyContext>` borrow to the `Fut` return-type's lifetime:

```rust
#[expect(dead_code, reason = "pre-landed federation-inbound enforcement infra; callers wired by Cohort B Tasks 5-7 (impl GovernanceInboundActivity + wrap_governance_inbound call sites), which declare requires: task 4 per plan §13")]
pub(crate) async fn wrap_governance_inbound<F, Fut, A>(
  activity: A,
  context: &Data<LemmyContext>,
  inner: F,
) -> LemmyResult<()>
where
  F: FnOnce(A, &Data<LemmyContext>) -> Fut,
  Fut: std::future::Future<Output = LemmyResult<()>>,
  A: GovernanceInboundActivity + std::marker::Sync,
{
```

Task 5's caller in `publish_sanction_notice.rs:104` (per §10.5 verbatim — `|a, c| async move { crate::governance::inbox::receive_remote_sanction_notice(a, c).await }`) fails to compile because the closure-return-type `Fut` is concretely typed before the compiler binds `'1` (the borrow lifetime of `&Data<LemmyContext>` flowing into the closure's `c` arg), so the compiler cannot prove `Fut: '1`. Compile error verbatim (advisor §15 cmd-1 log tail):

```
error: lifetime may not live long enough
   --> crates\apub\activities\src\governance\publish_sanction_notice.rs:104:77
    |
104 |       crate::governance::inbox::wrap_governance_inbound(self, context, |a, c| async move {
    |  __________________________________________________________________________--_^
    | |                                                                          ||
    | |                                                                          |return type of closure `{async block@…:104:77: 104:87}` contains a lifetime `'2`
    | |                                                                          has type `&'1 activitypub_federation::config::Data<LemmyContext>`
105 | |       crate::governance::inbox::receive_remote_sanction_notice(a, c).await
106 | |     }).await
    | |_____^ returning this value requires that `'1` must outlive `'2`

error: could not compile `lemmy_apub_activities` (lib) due to 1 previous error
```

### 2.2 The fix (the contract — verbatim replacement; copy EXACTLY, do not paraphrase)

Replace the `wrap_governance_inbound` declaration + its `where` clause (currently lines ~485-495) with the HRTB-correct sig + STRIP the `#[expect(dead_code, ...)]` line. The new shape:

```rust
/// Apply the five-gate inbound enforcement policy then delegate to `inner`.
///
/// Called from each `Activity::receive` impl (Tasks 5-7) so the gate
/// sequence is identical across all governance activity types.
pub(crate) async fn wrap_governance_inbound<'a, F, Fut, A>(
  activity: A,
  context: &'a Data<LemmyContext>,
  inner: F,
) -> LemmyResult<()>
where
  F: FnOnce(A, &'a Data<LemmyContext>) -> Fut,
  Fut: std::future::Future<Output = LemmyResult<()>> + 'a,
  A: GovernanceInboundActivity + std::marker::Sync + 'a,
{
```

Diff (apply EXACTLY this, no other changes):

1. **DELETE** the line at `#[expect(dead_code, reason = "pre-landed federation-inbound enforcement infra; callers wired by Cohort B Tasks 5-7 (impl GovernanceInboundActivity + wrap_governance_inbound call sites), which declare requires: task 4 per plan §13")]` immediately above `pub(crate) async fn wrap_governance_inbound`. (Task 5 now calls wrap → the dead_code expectation is unfulfilled → workspace `allow_attributes = "deny"` would fail clippy. Rust dead_code chain-pruning per fix-impl-3 retro: stripping the root's annotation cascades to its callees. The `#[expect(dead_code)]` on `rate_per_actor_counts` L470 STAYS — that fn is only called by Task-6's per-actor override which is still pending.)
2. **CHANGE** `pub(crate) async fn wrap_governance_inbound<F, Fut, A>(` → `pub(crate) async fn wrap_governance_inbound<'a, F, Fut, A>(` (add `'a, ` at start of the type-param list).
3. **CHANGE** `context: &Data<LemmyContext>,` (the wrap-sig's own parameter, NOT the body's many `&Data` usages — that single line just inside the `(...)` between `activity: A,` and `inner: F,`) → `context: &'a Data<LemmyContext>,` (add `'a` to the borrow).
4. **CHANGE** `F: FnOnce(A, &Data<LemmyContext>) -> Fut,` → `F: FnOnce(A, &'a Data<LemmyContext>) -> Fut,` (add `'a` to the borrow in the bound).
5. **CHANGE** `Fut: std::future::Future<Output = LemmyResult<()>>,` → `Fut: std::future::Future<Output = LemmyResult<()>> + 'a,` (add `+ 'a` after the `>`).
6. **CHANGE** `A: GovernanceInboundActivity + std::marker::Sync,` → `A: GovernanceInboundActivity + std::marker::Sync + 'a,` (add `+ 'a` at end).

Nothing else in inbox.rs changes. The wrap body (lines 496 onward — `let peer_domain = activity.actor_domain()?;` etc.) is untouched. The `#[expect(dead_code)]` on `rate_per_actor_counts` (L470) is untouched.

### 2.3 What is NOT in scope (the fence)

- **NEVER touch `publish_sanction_notice.rs`.** Task-5's commit (92605e907) is correct per plan §10.5 verbatim — the closure shape `|a, c| async move { ... }` is the contract; this fix-impl makes the wrap-sig CORRECT FOR THAT CONTRACT.
- **NEVER touch the GovernanceInboundActivity trait** (L443-461). Trait sig is correct.
- **NEVER touch `rate_per_actor_counts` `#[expect(dead_code)]` L470.** That fn is consumed by Task-6's per-actor override (still pending dispatch) — premature strip would re-introduce a dead_code lint until Task 6 lands.
- **NEVER touch any `receive_remote_*` fn** (Phase-6 handlers + Task-4 modifications). Out of scope.
- **NEVER touch `log_inbox_drop`, `get_inbound_config_int`, `federation_inbox_check_peer_trust`, `current_hour_bucket`, `rate_per_peer_counts`, helper consts.** Out of scope.
- **NEVER touch plan/PRD/briefs.** §10.4 plan body retains its current text — that's a retro-scope edit (proposal #2 in the §G4 latent-defect retro chain), not a fix-impl scope.
- **NEVER touch `publish_trust_attestation.rs` or `publish_label.rs`.** Cohort-B Tasks 6/7 are pending; they will use the same wrap-sig (with HRTB) once this fix lands.
- **NEVER add `#[allow(...)]` to suppress any warning.** Per workspace `allow_attributes = "deny"`; only `#[expect(...)]` with reason is sanctioned and ONLY when it's the contractually-correct lint suppression.

### 2.4 Verification (run BEFORE committing — pre-push cargo-check discipline per `feedback_fix_impl_pre_push_cargo_check.md` 2026-05-13)

In the worker's worktree, after applying the §2.2 edit, run a LOCAL `cargo check` BEFORE pushing — the brief MUST verify the fix compiles before the worker exits, so the advisor doesn't burn a §15 cycle on a non-compiling fix-impl:

```
bash scripts/brehon/cargo-check.sh --workspace --features full 2>&1 | tail -50
```

Expected: `Finished \`dev\` profile [unoptimized] target(s) in <N>m<N>s` and exit 0. If the local cargo check fails:
- Compile error in inbox.rs (the worker mis-applied the §2.2 edit) → patch in same commit (re-read §2.2 verbatim and re-apply).
- Compile error in publish_sanction_notice.rs (HRTB still failing) → the §2.2 shape is wrong; STOP + `kind:"blocker"` with the error text.
- Unused-import / dead-code regression triggered elsewhere → STOP + `kind:"blocker"` with the offending file:line. (Per fix-impl-3 retro: stripping `#[expect(dead_code)]` on wrap may chain-cascade — but ONLY to wrap's transitive callees in the gate body; rate_per_peer_counts L464 is already non-annotated, current_hour_bucket L477 is already non-annotated, log_inbox_drop is already non-annotated, all GovernanceInboundActivity trait methods are consumed by the impl on publish_sanction_notice. Expected outcome: zero new dead-code reports.)

Then grep-verify the §2.2 edit landed cleanly:

```
# Expected: exactly 0 instances of the OLD sig
grep -c "pub(crate) async fn wrap_governance_inbound<F, Fut, A>" crates/apub/activities/src/governance/inbox.rs
# Expected: exactly 1 instance of the NEW sig
grep -c "pub(crate) async fn wrap_governance_inbound<'a, F, Fut, A>" crates/apub/activities/src/governance/inbox.rs
# Expected: exactly 0 instances of the dead_code annotation on wrap (the rate_per_actor_counts L470 one stays — that's a SEPARATE line)
grep -c "callers wired by Cohort B Tasks 5-7" crates/apub/activities/src/governance/inbox.rs
# (must equal 1 — the rate_per_actor_counts annotation. If 2 → did NOT strip the wrap annotation. If 0 → also stripped rate_per_actor_counts annotation, scope creep.)
# Expected: the `+ 'a` bounds present
grep -c "Future<Output = LemmyResult<()>> + 'a" crates/apub/activities/src/governance/inbox.rs
# (must be exactly 1)
```

If ANY grep returns the wrong count → patch + re-grep before committing.

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` pending entries** — DQ #285 (validate-pending-laptop, ptask=5, branch=junior worker for Task 5) is the gating entry; this fix-impl unblocks it. Read its `commands[]` — those are the §15 cmds the advisor will re-run AFTER this fix's finalize-merge.
2. **`.claude/PRPs/briefs/v1-federation-inbound-b-impl-5.md`** — the Task-5 brief — to confirm Task 5's contract (publish_sanction_notice.rs:104 caller shape per §10.5 verbatim) is what we're making the wrap-sig SUPPORT. Read §2 Scope + §5 Validation gates only.
3. **Plan §10.4** at `.claude/PRPs/plans/v1-federation-inbound-b.plan.md` lines 788-1100 (the `wrap_governance_inbound` block) — read the current sig text + body to confirm the §2.2 edit aligns with the §10.4 intent. Plan §10.4 body text retains its current text; only the in-repo `inbox.rs` sig changes. (Plan retrofit is a retro-scope edit, not in this fix's scope.)
4. **`crates/apub/activities/src/governance/inbox.rs:486-495`** — the current wrap-sig + where clause. Read it byte-for-byte before applying §2.2.
5. **`crates/apub/activities/src/governance/publish_sanction_notice.rs:103-110`** — the caller. Read it to confirm the new HRTB sig supports this exact caller shape (don't edit it; just read).
6. **Lessons** (MANDATORY per `.claude/rules/advisor-orchestrator.md` §2.4 + fix-impl-3 retro):
   - `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` — **Why:** mechanical fix-impl briefs MUST include the §2.4 pre-push cargo-check; without it, an HRTB-adjacent regression class costs another full ci-watcher cycle. Local cargo check ~30s warm.
   - `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Why:** `wrap_governance_inbound` returns `LemmyResult<()>`; the inner closure returns `LemmyResult<()>`. The HRTB sig fix MUST preserve the `Fut: Future<Output = LemmyResult<()>>` constraint exactly (don't drift to `Box<dyn Error>`).
   - `.claude/lessons/feedback_clippy_test_style.md` + `.claude/lessons/feedback_clippy_rerun_after_fix.md` — **Why:** the workspace clippy config denies `#[allow]`; only `#[expect(...)]` with reason is sanctioned. Stripping the wrap `#[expect(dead_code)]` is the correct outcome (it would become `unfulfilled_lint_expectations` once Task 5 calls wrap).
   - `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md` — **Why:** this is the same class as fix-impl-3 (latent-defect-via-#[expect(dead_code)] surfacing at first caller) but at the SIGNATURE level rather than the BODY level. Retro will pair them.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always) — capture-then-tail for the §2.4 pre-push cargo-check (you DO run it; this is unique among fix-impl briefs because the bug class — silent HRTB failures cascading at first caller — demands worker-side local-compile verification per the new pre-push discipline).

## §3a Handover from prior cohort

> **Handover state: DEGRADED + LATENT-DEFECT-CHAIN-AWARE** (per `.claude/rules/advisor-orchestrator.md` §4.3 — Task-4 chain commits and Task-5 commit carry no `HANDOVER:` YAML trailer). Synthesized by the advisor from the Task-4 + Task-5 public-API + Task-5-failure-context. This is NOT a catch-fire; the §0 grep self-verifies the actual symbols on the phase tip as the authoritative existence check.

```yaml
prior_tasks:
  - task: 4
    commits:
      - a3757eabe (impl)
      - cdff6f09d (fix-impl-1)
      - f01a1d44e (fix-impl-2 — also 45374097b finalize-merge)
      - 4a60667c9 (fix-impl-3 finalize-merge; Task 4 §15-green per DQ #283 resolved pass 2026-05-19T20:44:53Z)
    filesModified: [crates/apub/activities/src/governance/inbox.rs]
    keyDecisions:
      - "wrap_governance_inbound sig (L486) shipped with `<F, Fut, A>` (no HRTB) and `#[expect(dead_code)]` (L485) — protected from compiler call-site exercise; PASSED §15 e2e --no-run because no caller existed in lib AND no test caller exercised the closure shape."
      - "GovernanceInboundActivity trait (L443-461) shipped correctly; receive_remote_sanction_notice / receive_remote_trust_attestation / receive_remote_moderation_label all preserved their canonical sigs (Phase-6 compatibility)."
      - "rate_per_actor_counts (L470) shipped with `#[expect(dead_code)]` — STAYS until Task 6 (per-actor override) lands."
  - task: 5
    commits:
      - 92605e907 (impl — publish_sanction_notice.rs §10.5 verbatim wrap+trait-impl, 28-line diff)
      - 9f9ca9bc4 (DQ #285 mid-task push)
      - fcc0e7de0 (daemon finalize-merge — phase tip)
    filesModified: [crates/apub/activities/src/governance/publish_sanction_notice.rs, .claude/decision-queue.json]
    keyDecisions:
      - "publish_sanction_notice.rs:104 caller follows plan §10.5 VERBATIM — `|a, c| async move { receive_remote_sanction_notice(a, c).await }`. Worker correctly hard-followed contract; the §15 fail is a LATENT defect in Task-4 wrap-sig (not §10.5, not worker), surfaced because Task 5 is the FIRST caller of wrap and #[expect(dead_code)] previously shielded it. fix-impl-4 fixes the wrap-sig; publish_sanction_notice.rs stays unchanged."
      - "DQ #285 raised inline by worker (no harness-gap escalation needed — first Cohort-B task that worked without TASK<N>_*.json carriers since #335/#336 noise). Commands verbatim: cargo-check.bat + cargo-clippy.bat -D warnings (2 cmds, no e2e --no-run for Task 5)."
    notes: "fix-impl-4 CONSUMES (a) Task-4's wrapper signature (rewrites it minimally — HRTB + strip dead_code) and (b) Task-5's caller shape (preserves it byte-for-byte). The §0 grep self-verifies wrap symbol presence + HRTB-absence + dead_code-annotation-presence as the authoritative existence check. The §2.2 §15-precondition verification is the worker's contract: §2.4 pre-push cargo-check MUST pass before exit."
```

## §4 Constraints (hard rules)

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-federation-inbound-b` (tip `fcc0e7de0`, Task 5 worker-merge tip with the HRTB defect still present in wrap-sig). Finalize merges your worktree branch back; do NOT push to `phase-v1-federation-inbound-b` directly.
- ONE commit (single file, ≤8 lines changed in 1 hunk). Commit subject:

  ```
  fix(v1-federation-inbound-b): wrap_governance_inbound HRTB <'a> + strip dead_code (fix-impl-4 → Task 5 §15)
  ```

- Mid-task DQ visibility: if you raise a `pending` entry mid-task, **commit + push immediately** to your worktree branch per `.claude/rules/decision-queue.md` "Mid-task visibility".
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235)

If you need to write a DQ entry to `.claude/decision-queue.json` and the Claude Code sensitive-file gate blocks it: (a) write the intended DQ-entry JSON object to a worktree-root file `FIXIMPL4_BLOCKER_DQ.json`, (b) write a short `FIXIMPL4_ESCALATION.md` naming the issue, (c) commit both at worktree root + push, (d) STOP. The advisor transcribes per `.claude/rules/escalation.md`. fix-impl-4 has NO `.claude/` deliverable on the happy path — its only deliverable is the 1-hunk inbox.rs edit; there is NO `validate-pending` DQ write from this fix (the advisor mutates the EXISTING DQ #285 after re-running §15). The harness gap likely does NOT fire for this fix.

### fix-impl-4 GOTCHAs (load-bearing)

- **Strip the wrap `#[expect(dead_code)]` (L485 above the wrap fn).** Once Task 5 calls wrap, the dead_code expectation is UNFULFILLED → `unfulfilled_lint_expectations` is workspace-deny → clippy will fail. The strip is part of §2.2's contract.
- **Do NOT strip the `#[expect(dead_code)]` on `rate_per_actor_counts` L470.** That fn is consumed by Task-6 (per-actor override, still pending dispatch). Stripping it now → dead-code regression. The grep-verify at §2.4 counts `"callers wired by Cohort B Tasks 5-7"` and expects `1` (the rate_per_actor_counts annotation, NOT the wrap one).
- **HRTB syntax precision.** The `'a` lifetime parameter MUST appear BEFORE `F, Fut, A` in the generic list (`<'a, F, Fut, A>`, not `<F, Fut, A, 'a>` — Rust grammar requires lifetimes first). The `&'a` MUST appear in BOTH the parameter declaration AND the `F: FnOnce(A, &'a Data<LemmyContext>) -> Fut` bound (the bound's borrow must use the same lifetime name as the parameter). `Fut: ... + 'a` AND `A: ... + 'a` ensure the closure's return future and the activity outlive `'a`.
- **One commit, one file.** If §2.4 pre-push cargo-check surfaces a regression in a different file (unexpected), STOP + `kind:"blocker"` (do NOT add a second hunk to "fix" it; the fix-impl scope is the wrap-sig only).

### Plan-cited line numbers may have drifted

§2.2 cites "lines ~485-495" in `inbox.rs`. `grep -n "pub(crate) async fn wrap_governance_inbound" crates/apub/activities/src/governance/inbox.rs` for the real position. The edit is identified by SYMBOL + SIG-TEXT, not by line numbers.

## §5 Validation gates (advisor re-runs §15 — worker runs §2.4 pre-push cargo-check)

**Worker-side (this fix-impl):** §2.4 pre-push cargo-check is the only worker-side validation. Worker does NOT raise a NEW `validate-pending-laptop` DQ — the existing DQ #285 (raised by Task-5 worker, still pending) is the gate the advisor mutates after re-running §15 on the post-finalize-merge tip.

**Advisor-side (after this fix's finalize-merge):** advisor re-runs DQ #285's `commands[]` verbatim on the laptop lane:

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task5-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task5-clippy.log 2>&1"
```

On BOTH GREEN: advisor mutates DQ #285 result:"pass" + Task 5 complete. On either fail: §G4 classify + surface.

## §6 Expected output (return to advisor)

```
## fix-impl-4 complete — wrap_governance_inbound HRTB <'a> + strip dead_code

**Commit:** <sha> on <worktree-branch>
**File changed:** crates/apub/activities/src/governance/inbox.rs (1 hunk, ≤8 lines)
**PRECON self-check:** wrap sig had `<F, Fut, A>` (no HRTB) AND `#[expect(dead_code)]` (L485) — both verified present at base tip fcc0e7de0; new sig has `<'a, F, Fut, A>` with `&'a Data<LemmyContext>` + `Fut: ... + 'a` + `A: ... + 'a` bounds; wrap dead_code annotation STRIPPED.
**rate_per_actor_counts annotation:** STAYS (verified — grep count = 1 for the matching reason text).
**No other-file edit:** confirmed (only inbox.rs touched).
**§2.4 pre-push cargo-check:** GREEN (Finished dev profile in <N>m<N>s, exit 0).
**Grep-verify:** all 4 grep counts at expected values.
**No new DQ raised (the existing DQ #285 from Task-5 stays pending; advisor mutates after re-running §15).
**Next:** advisor laptop re-runs §15 (DQ #285 cmds = check + clippy -D warnings), mutates DQ #285 result:pass; Task 5 complete; Cohort B Task 6 dispatched serial.
```

Plus any DQ #N references if you raised a blocker mid-task (NOT for §2.4 cargo-check failure — that you patch in-commit per the §2.4 routing).

## §7 Why this brief differs from the plan

Plan §10.4 (line 882) prescribed the wrap-sig WITHOUT HRTB; that was a latent defect masked by `#[expect(dead_code)]`. This fix-impl revises the in-repo `inbox.rs` sig to add the HRTB lifetime parameter; plan §10.4 body text retrofit is a separate retro-scope edit (not in this fix's scope — the working source-of-truth on the phase tip is what the next caller compiles against). This brief differs from a standard impl-task brief in three ways: (a) §2.2 is the verbatim hunk-level diff specification (this is a mechanical sig fix — there is no "interpretation"); (b) §2.4 mandates pre-push cargo-check per `feedback_fix_impl_pre_push_cargo_check.md` — the bug class (silent HRTB-cascade-at-first-caller) demands worker-side verification before exit; (c) no new `validate-pending` DQ — the existing DQ #285 (raised by Task-5) is the gate, mutated by advisor after re-running §15 on the post-finalize-merge tip. Mirrors the canonical sibling `.claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-3.md` schema per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate (same §0-§7 brief STRUCTURE; same single-hunk single-file CAP discipline; same triggering-DQ-mutated-by-advisor-post-§15 pattern).

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; per DQ #279 the canonical session drives fed-in-b impl under serial-phase policy). Brief committed on `governance-v0` first; then cherry-picked / re-committed onto `phase-v1-federation-inbound-b` so the worker — which branches from the PHASE branch tip `fcc0e7de0` — sees it. `/precheck` re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC to the post-cherry-pick tip before queue. User-authorised waiver of the Task-5 brief's "no inbox.rs edit" guard per AskUserQuestion 2026-05-19 (Option A) — this is a Task-4-territory sig fix, scoped as a fix-impl (advisor-authored brief), not worker scope creep. fix-impl-4 is the FIRST fix-impl for Task 5 (continuing the fix-impl-N numbering across Task 4 chain 1/2/3 and now Task 5 chain 4). Non-§G4-allowlist; hand-authored mechanical fix._
