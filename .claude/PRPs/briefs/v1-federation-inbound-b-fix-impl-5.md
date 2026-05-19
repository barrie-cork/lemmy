---
phase: v1-federation-inbound-b
role: impl-task
kind: fix-impl
fix_impl_n: 5
authored: 2026-05-19
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
triggering_dq: 286
triggering_task: 6
classification: "E0283 type-annotations-needed in Task-6 worker code (publish_trust_attestation.rs:155 `.into()`). Same LemmyError-bridge family as §G4 row 4a/4b (canonical lesson `feedback_lemmy_error_no_std_error.md`). The CANONICAL FIX is byte-identical with the SAME-FILE working sibling on lines 148-150 (4 lines above the defect): drop the spurious `.into()` and return `LemmyErrorType` directly, let `?` do the deterministic `From<LemmyErrorType> for LemmyError` conversion. The defect is the worker added a spurious `.into()` in the `.ok_or_else()` closure that introduced From-target ambiguity (compiler sees multiple `impl From<_> for LemmyError`). **Allowlist-equivalent §G4 mechanical fix** (verbatim canonical-sibling mirror; zero design ambiguity)."
base: "phase-v1-federation-inbound-b @ 5e6161a30 (Task 6 worker-merge tip — publish_trust_attestation.rs wrap+trait-impl+per-actor-override; DQ #286 raised pending; §15 cmd-1 FAILED on this tip with the E0283 ambiguity)."
cap: "EXACTLY 1 hunk (≤5 lines), 1 file ONLY: crates/apub/activities/src/governance/publish_trust_attestation.rs. Lines 151-156 currently read: `val.ok_or_else(|| { LemmyErrorType::Unknown(format!(\"governance_config.{CONFIG_KEY} has null value_int\"),) .into() })?`. CHANGE to: `val.ok_or_else(|| LemmyErrorType::Unknown(format!(\"governance_config.{CONFIG_KEY} has null value_int\")))?` (drop `.into()` + drop the inner `{ ... }` block since closure body is now a single expression). This mirrors the byte-identical .map_err sibling on lines 148-150 of the SAME fn (4 lines above): `.map_err(|_e| { LemmyErrorType::Unknown(format!(\"governance_config.{CONFIG_KEY} not seeded\")) })?` — note the .map_err sibling DOES NOT carry `.into()` and works. NEVER touch any other line, fn, or file. NEVER change crates/** elsewhere, schema_setup, Cargo.*, the plan, e2e.rs, any test, inbox.rs, publish_sanction_notice.rs, publish_label.rs, the wrap call site, the trait impl block, the rate-gate body, or the imports. A 2nd hunk or any other-file edit → STOP + kind:blocker."
serial: "Single-task barrier fix (Task 6 §15 — the FIRST fix-impl on Task 6), strictly serial cap=1 — only in-flight Junior for this lane. §15 (DQ #286 — 2 cmds: cargo-check + cargo-clippy --no-deps -- -D warnings; NO e2e --no-run for Task 6) is re-run by the ADVISOR on the laptop AFTER this fix lands + finalize-merge. Task 6 is COMPLETE only when this fix's §15 passes both cmds; THEN advisor mutates DQ #286 result:pass. Cohort B's remaining serial dispatch (Task 7) stays gated behind Task-6-complete per user's cap-≤2 OOM choice."
---

# [role:impl-task] v1-federation-inbound-b fix-impl-5 — publish_trust_attestation.rs:155 drop spurious .into() (verbatim same-fn sibling mirror)

> **Provenance:** Task 6 (#342, worker commit ec3b8643f, finalize-merge 5e6161a30) followed plan §10.5 / §10.6 (per-actor override) for publish_trust_attestation.rs. Advisor §15 cmd-1 `cargo-check.bat --workspace --features full` FAILED at publish_trust_attestation.rs:155 with `error[E0283]: type annotations needed` (2 occurrences, same line — both `cannot satisfy _: From<LemmyErrorType>` because multiple `impl From<_> for LemmyError` exist in `lemmy_utils`). Root cause = worker wrote `.ok_or_else(|| { LemmyErrorType::Unknown(...).into() })?` — the spurious `.into()` introduces target-type ambiguity (compiler can't tell whether the closure returns `LemmyError`, `anyhow::Error`, or something else). The byte-identical working sibling 4 lines above (line 148-150 `.map_err(|_e| { LemmyErrorType::Unknown(...) })?`) does NOT carry `.into()` and works. **§G4-allowlist-equivalent mechanical fix** per canonical lesson `feedback_lemmy_error_no_std_error.md` — same-file canonical-sibling mirror, ZERO design ambiguity. DQ #286 stays pending until this fix's §15 (2 cmds) passes.

## §0 Pre-flight (subagent runs this FIRST, before reading anything else)

- Confirm CWD is a Junior worktree branched off `phase-v1-federation-inbound-b` (base tip `5e6161a30`). `git merge-base --is-ancestor 5e6161a30 HEAD` MUST be true. If on `phase-v1-federation-inbound-b` itself or a non-`junior/*` branch → STOP, file `kind: "blocker"` DQ into `.claude/decision-queue.json` (NOT a repo-root file unless the §4 harness-gap fires).
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — inside a forbidden window → exit non-zero `FORBIDDEN_WINDOW: <window>` unless dispatch carries `(user-authorised forbidden-window override per DQ #<id>)`. Shape G SUSPENDED (cargo on laptop) — keep the check.
- **Submodule init (MANDATORY before any in-worker cargo — per `feedback_worktree_submodules_not_auto_init`):** `git submodule update --init 2>&1` from worktree root. `crates/email/translations` is NOT auto-initialized in a fresh worktree; without it cargo fails pre-existing. Infra, not the fix.
- Confirm the anchor is present at the expected shape (grep the text, line numbers approximate — base tip 5e6161a30):
  ```
  grep -n ".ok_or_else.*{" crates/apub/activities/src/governance/publish_trust_attestation.rs | head -3
  grep -n "has null value_int" crates/apub/activities/src/governance/publish_trust_attestation.rs
  ```
  Expected: line ~151 contains `val.ok_or_else(|| {` with line ~152-154 containing `LemmyErrorType::Unknown(format!("governance_config.{CONFIG_KEY} has null value_int"),)` and line ~155 containing `.into()`. If 0 → STOP + `kind:"blocker"` (already fixed). If multiple → STOP + `kind:"blocker"` (cap-1-hunk assumption invalid).
- Confirm the canonical-sibling working pattern is present at the expected shape (the reference shape — DO NOT EDIT IT):
  ```
  grep -B 1 -A 3 ".map_err(|_e|" crates/apub/activities/src/governance/publish_trust_attestation.rs | head -10
  ```
  Expected: line ~148-150 contains `.map_err(|_e| { LemmyErrorType::Unknown(format!("governance_config.{CONFIG_KEY} not seeded")) })?` (NO `.into()`). If absent → STOP + `kind:"blocker"` (the canonical reference changed).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b fix-impl-5 — publish_trust_attestation.rs:155 drop spurious .into() (Task 6 §15 E0283 fix)`

Actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-b fix-impl-5 — see .claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-5.md
```

## §2 Scope

### 2.1 The defect being fixed (the contract — Task 6 §15 cmd-1 fail, verified by advisor 2026-05-19)

In `crates/apub/activities/src/governance/publish_trust_attestation.rs`, around line 151-156 (inside the `let config_value = async { ... }` block that reads the per-actor rate cap from `governance_config`):

```rust
val.ok_or_else(|| {
  LemmyErrorType::Unknown(
    format!("governance_config.{CONFIG_KEY} has null value_int"),
  )
  .into()
})?
```

The spurious `.into()` on the `LemmyErrorType` makes the closure return type ambiguous. The compiler must choose a target for `Into::into()` but multiple `impl From<X> for LemmyError` exist (from `LemmyErrorType`, from `anyhow::Error`, and the blanket `impl<T> From<T> for T`). Without a target type annotation, E0283 fires.

Compile error verbatim (advisor §15 cmd-1 log tail, both E0283 occurrences point at the SAME line):

```
error[E0283]: type annotations needed
   --> crates\apub\activities\src\governance\publish_trust_attestation.rs:155:10
    |
155 |         .into()
    |          ^^^^
    |
    = note: cannot satisfy `_: From<LemmyErrorType>`
    = note: required for `LemmyErrorType` to implement `Into<_>`

error[E0283]: type annotations needed
   --> crates\apub\activities\src\governance\publish_trust_attestation.rs:155:10
    |
    = note: multiple `impl`s satisfying `LemmyError: From<_>` found in the following crates: `core`, `lemmy_utils`:
            - impl From<LemmyErrorType> for LemmyError;
            - impl From<lemmy_utils::error::UntranslatedError> for LemmyError;
            - impl<T> From<T> for LemmyError where T: Into<anyhow::Error>;
            - impl<T> From<T> for T;
```

### 2.2 The fix (the contract — verbatim canonical-sibling mirror; copy EXACTLY, do not paraphrase)

**§G4 CANONICAL RECIPE (verbatim from `feedback_lemmy_error_no_std_error.md` Case-B + the byte-identical SAME-FILE sibling `.map_err` on lines 148-150):**

> | Failure signature | Auto-fix | Source lesson |
> | E0283 `type annotations needed` on `.into()` of a `LemmyErrorType` inside an `.ok_or_else()` / `.map_err()` closure that returns to an `?` operator → multiple `impl From<_> for LemmyError` available; compiler cannot pick the target type | **Drop the `.into()` call** and return the `LemmyErrorType` directly from the closure. The `?` operator performs the `From<LemmyErrorType> for LemmyError` conversion deterministically (single-impl resolution path). Mirror the byte-identical working sibling pattern in the same file/fn that does NOT carry `.into()`. | `feedback_lemmy_error_no_std_error.md` Case-B |

REPLACE the existing lines 151-156 (the `val.ok_or_else(...)` block) with the canonical-sibling-mirror shape. The new lines:

```rust
val.ok_or_else(|| LemmyErrorType::Unknown(format!("governance_config.{CONFIG_KEY} has null value_int")))?
```

Diff (apply EXACTLY this, no other changes):

1. **DELETE** the 5 lines currently at 151-156 (the `val.ok_or_else(|| {` opening, the multi-line `LemmyErrorType::Unknown(format!(...))` argument, the `.into()`, and the `})?` closing).
2. **INSERT** the single line above as the replacement.

The closure body becomes a single expression (no `{ ... }` block needed), returning a `LemmyErrorType` directly. The `?` after the closing `)` performs the deterministic `From<LemmyErrorType> for LemmyError` conversion. This mirrors the `.map_err` sibling on lines 148-150 exactly (which uses the same `LemmyErrorType::Unknown(format!(...))` pattern without `.into()` and compiles cleanly).

### 2.3 What is NOT in scope (the fence)

- **NEVER touch `inbox.rs`.** Task-4 territory; fix-impl-4 already shipped the HRTB wrap-sig fix; no further inbox.rs edits.
- **NEVER touch `publish_sanction_notice.rs`** (Task-5 commit 92605e907 is complete + §15-green).
- **NEVER touch `publish_label.rs`** (Task-7 still pending — DO NOT pre-touch).
- **NEVER touch the wrap_governance_inbound call site** in publish_trust_attestation.rs (the `Activity::receive` body) — unchanged + working.
- **NEVER touch the impl GovernanceInboundActivity for PublishTrustAttestation trait-impl block** — unchanged + working.
- **NEVER touch the `check_per_actor_rate_limit` override fn body** beyond the single 1-hunk fix at lines 151-156 (the rest of the rate-gate logic is unchanged + correct).
- **NEVER touch the `.map_err` sibling at lines 148-150.** That is the CORRECT reference pattern; leave it byte-for-byte.
- **NEVER touch any other file** in `crates/` or any test, schema, Cargo, plan, brief, lesson.
- **NEVER add `#[allow(...)]` or `#[expect(...)]`** to suppress the E0283. The fix is structural (drop `.into()`), not a lint suppression.

### 2.4 Verification (run BEFORE committing — pre-push cargo-check discipline per `feedback_fix_impl_pre_push_cargo_check.md` 2026-05-13)

In the worker's worktree, after applying the §2.2 edit, run a LOCAL `cargo check` BEFORE pushing:

```
bash scripts/brehon/cargo-check.sh --workspace --features full 2>&1 | tail -30
```

Expected: `Finished \`dev\` profile [unoptimized] target(s) in <N>m<N>s` and exit 0. If the local cargo check fails:
- Compile error STILL at the SAME publish_trust_attestation.rs:line (E0283 not resolved) → STOP + `kind:"blocker"` with the new error text (the §2.2 fix shape is wrong; advisor will re-author).
- NEW compile error somewhere else (e.g. lifetime issue from the closure-body simplification) → STOP + `kind:"blocker"` with the offending file:line.
- Unused-import / dead-code regression → STOP + `kind:"blocker"` with the offending file:line.

Then grep-verify the §2.2 edit landed cleanly:

```
# Expected: exactly 0 instances of the OLD .into() that fired E0283
grep -c "LemmyErrorType::Unknown.*has null value_int" crates/apub/activities/src/governance/publish_trust_attestation.rs
# (must be exactly 1 — only the new single-line replacement contains this string. The OLD multi-line had it too, but we just deleted it.)
# Expected: exactly 0 lines in publish_trust_attestation.rs containing ".into()" on its own (the bare-.into() spurious pattern)
grep -n "^[[:space:]]*\.into()$" crates/apub/activities/src/governance/publish_trust_attestation.rs
# (must return NOTHING — no orphan .into() left)
# Expected: the canonical .map_err sibling UNCHANGED
grep -c "governance_config.{CONFIG_KEY} not seeded" crates/apub/activities/src/governance/publish_trust_attestation.rs
# (must be exactly 1 — the working sibling we mirrored)
```

If ANY grep returns the wrong count → patch + re-grep before committing.

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` pending entries** — DQ #286 (validate-pending-laptop, ptask=6, branch=junior worker for Task 6) is the gating entry; this fix-impl unblocks it. Read its `commands[]` — those are the §15 cmds the advisor will re-run AFTER this fix's finalize-merge.
2. **`crates/apub/activities/src/governance/publish_trust_attestation.rs:140-170`** — the entire config_value block. Read it byte-for-byte BEFORE applying §2.2 to confirm the canonical-sibling `.map_err` is on lines 148-150 (NOT carrying `.into()`) and the defect is on lines 151-156 (carrying `.into()`).
3. **`.claude/PRPs/briefs/v1-federation-inbound-b-impl-6.md`** — the Task-6 brief — to confirm Task 6's contract (single-file edit + cap key string + per-actor override + trait impl). Read §2 Scope only.
4. **Lessons** (MANDATORY per `.claude/rules/advisor-orchestrator.md` §2.4):
   - `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Why:** Case-B canonical recipe for the LemmyError-bridge family; the §G4 verbatim blockquote in §2.2 reproduces this lesson's mechanical fix.
   - `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` — **Why:** mechanical fix-impl briefs MUST include the §2.4 pre-push cargo-check; local cargo check ~30s warm vs a full ci-watcher cycle on regression.
   - `.claude/lessons/feedback_clippy_test_style.md` + `.claude/lessons/feedback_clippy_rerun_after_fix.md` — **Why:** clippy denies `#[allow]`; the §2.2 fix must NOT use a lint-suppression workaround.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always) — capture-then-tail for §2.4 pre-push cargo-check.

## §3a Handover from prior cohort

> **Handover state: DEGRADED + LEMMY-ERROR-BRIDGE-AWARE** (per `.claude/rules/advisor-orchestrator.md` §4.3 — Task-6 chain commits carry no `HANDOVER:` YAML trailer). Synthesized by the advisor from the Task-5 / Task-6 public-API + Task-6-failure-context. This is NOT a catch-fire; the §0 grep self-verifies the actual symbols on the phase tip as the authoritative existence check.

```yaml
prior_tasks:
  - task: 5
    commit: 92605e907 (impl) + finalize-merge fcc0e7de0 + fix-impl-4 finalize-merge cd9ae510b + DQ #285 mutate 88f0f03c8
    filesModified: [crates/apub/activities/src/governance/publish_sanction_notice.rs, crates/apub/activities/src/governance/inbox.rs]
    keyDecisions:
      - "Task 5 wraps Activity::receive via §10.5 verbatim closure + impl GovernanceInboundActivity (trait-default rate gate). fix-impl-4 added <'a> HRTB to wrap_governance_inbound + stripped its #[expect(dead_code)]. Task 5 §15-green at 88f0f03c8."
  - task: 6
    commits:
      - ec3b8643f (impl — publish_trust_attestation.rs wrap+trait-impl+per-actor-override, 124-line diff)
      - 96bbf96ec (DQ #286 mid-task push)
      - 5e6161a30 (daemon finalize-merge — phase tip)
    filesModified: [crates/apub/activities/src/governance/publish_trust_attestation.rs, .claude/decision-queue.json]
    keyDecisions:
      - "publish_trust_attestation.rs all 8 required symbols present exactly once: wrap_governance_inbound, impl GovernanceInboundActivity for PublishTrustAttestation, federation.inbound.max_payload_bytes_trust_attestation, check_per_actor_rate_limit (override), rate_per_actor_counts (Task-4 helper), current_hour_bucket, log_inbox_drop, FederationActorRateLimitExceeded. NO out-of-scope edits to inbox.rs, publish_sanction_notice.rs, publish_label.rs."
      - "DEFECT: line ~151-156 inside the config_value-read block, `val.ok_or_else(|| { LemmyErrorType::Unknown(format!(...)).into() })?` — the spurious `.into()` fires E0283 (target-type ambiguity). Canonical sibling 4 lines above (line ~148-150 `.map_err(|_e| { LemmyErrorType::Unknown(format!(...)) })?`) does NOT carry `.into()` and works."
      - "DQ #286 raised inline by worker (no harness-gap escalation). Commands verbatim: cargo-check.bat + cargo-clippy.bat -D warnings."
    notes: "fix-impl-5 CONSUMES Task-6's source file unchanged except for the 1-hunk at lines 151-156. The §0 grep self-verifies the defect symbol presence + canonical-sibling presence as the authoritative existence check. The §2.4 §15-precondition verification is the worker's contract: §2.4 pre-push cargo-check MUST pass before exit. NO test or other file is touched."
```

## §4 Constraints (hard rules)

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-federation-inbound-b` (tip `5e6161a30`, Task 6 worker-merge tip with the E0283 defect still present). Finalize merges your worktree branch back; do NOT push to `phase-v1-federation-inbound-b` directly.
- ONE commit (single file, ≤5 lines changed in 1 hunk). Commit subject:

  ```
  fix(v1-federation-inbound-b): publish_trust_attestation drop spurious .into() (fix-impl-5 → Task 6 §15)
  ```

- Mid-task DQ visibility: if you raise a `pending` entry mid-task, **commit + push immediately** to your worktree branch per `.claude/rules/decision-queue.md` "Mid-task visibility".
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235)

If you need to write a DQ entry and the Claude Code sensitive-file gate blocks it: (a) write the intended DQ-entry JSON to `FIXIMPL5_BLOCKER_DQ.json` at worktree root, (b) write `FIXIMPL5_ESCALATION.md` naming the issue, (c) commit both + push, (d) STOP. The advisor transcribes per `.claude/rules/escalation.md`. fix-impl-5 has NO `.claude/` deliverable on the happy path — its only deliverable is the 1-hunk publish_trust_attestation.rs edit; there is NO `validate-pending` DQ write from this fix (advisor mutates the EXISTING DQ #286 after re-running §15).

### fix-impl-5 GOTCHAs (load-bearing)

- **The canonical sibling is in the SAME file, SAME function, 4 lines above.** Read it (line 148-150) before applying §2.2. The fix is a verbatim mirror of that sibling's pattern.
- **Closure body becomes single-expression.** The new shape `|| LemmyErrorType::Unknown(format!(...))` (no `{ ... }` block) is valid Rust — closures with a single expression body don't need braces. If the worker insists on keeping `{ ... }` and a single returning expression, that's fine too — but no `.into()` and no trailing `;`.
- **Do NOT add `<LemmyErrorType as Into<LemmyError>>::into(...)` or `LemmyError::from(...)`** as alternative fixes. The §2.2 contract is the verbatim canonical-sibling mirror (drop `.into()`); alternative fixes would diverge from the sibling and re-introduce the same audit-class drift the canonical-sibling rule prevents.
- **One commit, one file.** If §2.4 pre-push cargo-check surfaces a regression in a different file (unexpected), STOP + `kind:"blocker"` (do NOT add a second hunk).

### Plan-cited line numbers may have drifted

§2.2 cites "lines 151-156" in `publish_trust_attestation.rs`. `grep -n "has null value_int" crates/apub/activities/src/governance/publish_trust_attestation.rs` for the real position. The edit is identified by the STRING `has null value_int` + the surrounding `.ok_or_else(|| { ... .into() })?` shape, not by line numbers.

## §5 Validation gates (advisor re-runs §15 — worker runs §2.4 pre-push cargo-check)

**Worker-side (this fix-impl):** §2.4 pre-push cargo-check is the only worker-side validation. Worker does NOT raise a NEW `validate-pending-laptop` DQ — the existing DQ #286 (raised by Task-6 worker, still pending) is the gate the advisor mutates after re-running §15 on the post-finalize-merge tip.

**Advisor-side (after this fix's finalize-merge):** advisor re-runs DQ #286's `commands[]` verbatim on the laptop lane:

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task6-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task6-clippy.log 2>&1"
```

On BOTH GREEN: advisor mutates DQ #286 result:"pass" + Task 6 complete. On either fail: §G4 classify + surface.

## §6 Expected output (return to advisor)

```
## fix-impl-5 complete — publish_trust_attestation drop spurious .into()

**Commit:** <sha> on <worktree-branch>
**File changed:** crates/apub/activities/src/governance/publish_trust_attestation.rs (1 hunk, ≤5 lines)
**PRECON self-check:** defect `.into()` present at line ~155; canonical sibling `.map_err` present at line ~148-150 — both verified at base tip 5e6161a30; new single-line shape mirrors the canonical sibling byte-for-byte.
**No other-file edit:** confirmed (only publish_trust_attestation.rs touched).
**§2.4 pre-push cargo-check:** GREEN (Finished dev profile in <N>m<N>s, exit 0).
**Grep-verify:** the 3 grep counts at expected values (`has null value_int`=1, orphan `.into()`=0, sibling `not seeded`=1).
**No new DQ raised** (the existing DQ #286 from Task-6 stays pending; advisor mutates after re-running §15).
**Next:** advisor laptop re-runs §15 (DQ #286 cmds = check + clippy -D warnings), mutates DQ #286 result:pass; Task 6 complete; Cohort B Task 7 dispatched serial.
```

Plus any DQ #N references if you raised a blocker mid-task (NOT for §2.4 cargo-check failure — that you patch in-commit per the §2.4 routing).

## §7 Why this brief differs from the plan

Plan §10.5 / §10.6 (per-actor override) prescribed the `publish_trust_attestation.rs` wrap + trait-impl + per-actor rate gate; the worker followed the contract correctly EXCEPT for the spurious `.into()` at line 155. This fix-impl is a 1-hunk byte-identical-sibling mirror per §G4-allowlist-equivalent canonical recipe; plan §10.5 / §10.6 retrofit is NOT needed (the plan body is correct; the worker's `.into()` is the divergence, not the plan). This brief mirrors the canonical sibling `.claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-4.md` schema per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate (same §0-§7 structure; same single-hunk single-file CAP discipline; same triggering-DQ-mutated-by-advisor-post-§15 pattern). Includes the VERBATIM §G4 blockquote in §2.2 per `.claude/rules/advisor-orchestrator.md` §G4 mandatory-verbatim-recipe rule (the LemmyError-bridge family is §G4-allowlist; canonical recipe text from `feedback_lemmy_error_no_std_error.md` Case-B + the same-file working sibling).

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; per DQ #279 the canonical session drives fed-in-b impl under serial-phase policy). Brief committed on `governance-v0` first; then cherry-picked / re-committed onto `phase-v1-federation-inbound-b` so the worker — which branches from the PHASE branch tip `5e6161a30` — sees it. `/precheck` re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC to the post-cherry-pick tip before queue. §G4-allowlist-equivalent mechanical fix (canonical-sibling mirror; zero design ambiguity); does NOT require AskUserQuestion — auto-queued per §G4 allowlist routing. fix-impl-5 is the FIRST fix-impl for Task 6 (continuing the fix-impl-N numbering: 1-3 = Task 4 chain, 4 = Task 5 wrap-sig HRTB, 5 = Task 6 .into() drop)._
