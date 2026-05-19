---
phase: v1-federation-inbound-b
role: impl-task
kind: fix-impl
fix_impl_n: 3
authored: 2026-05-19
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
triggering_dq: 283
triggering_task: 4
classification: "Conformance-audit Finding 6.1 (latent data-integrity footgun, NOT a §15/compile failure). USER chose Option A + SEPARATE (2026-05-19). Task-4's receive_remote_moderation_label (new code) extracts the remote actor domain with `.domain().map(str::to_string).unwrap_or_default()` → a domainless remote actor silently persists `peer_domain = \"\"` (and downstream `source_instance = ''`), whereas the TWO Phase-6 siblings in the SAME FILE (receive_remote_sanction_notice L149-159, receive_remote_trust_attestation L261-271) hard-error via `.domain().ok_or_else(|| LemmyErrorType::Unknown(format!(...)))?.to_string()`. Compiles cleanly; the divergence is a security/data-integrity inconsistency (sanctions/attestations from a domainless actor are REJECTED; labels are STORED under ''). This is the feedback_plan_stub_uniformity_with_canonical_sibling class again — new handler didn't copy-adapt the canonical sibling's trust-boundary validation. The fix is a verbatim sibling-mirror: EXACTLY 1 hunk (2 lines), 1 file, zero design ambiguity (the canonical idiom is byte-identical in both siblings)."
base: "phase-v1-federation-inbound-b @ 9ca42bd47 (fix-impl-2 COMPLETE: §15 cold-reval clippy GREEN [FI2COLD_CLIPPY_EXIT_0, 0 findings, 0 unfulfilled] + check GREEN [FI2COLD_CHECK_EXIT_0] + e2e --no-run GREEN [FI2COLD_TESTNORUN_EXIT_0] at tip cee6b917c; DQ #284 resolved advisor-answer Option A @ 9ca42bd47; DQ #283 validate-pending-laptop ptask=4 still pending — fix-impl-3 is Task 4's LAST fix; DQ #283 mutated result:pass only after THIS fix's §15 passes)"
cap: "EXACTLY 1 hunk, 1 file ONLY: crates/apub/activities/src/governance/inbox.rs. Replace the 2-line fallback at the `receive_remote_moderation_label` domain-extraction (the `.map(str::to_string)` line + the `.unwrap_or_default()` line, currently ~L742-743 — grep the anchor, do NOT blind-edit) with the Phase-6 sibling's hard-error tail (5 lines: `.ok_or_else(|| {` / `LemmyErrorType::Unknown(format!(` / the message line / `activity.actor.inner(),` / `))` / `})?` / `.to_string();` — see §2.3 for the EXACT verbatim block). The let-binding `let peer_domain = activity` / `.actor` / `.inner()` / `.domain()` lines (currently ~L738-741) stay UNCHANGED — only the 2 fallback lines after `.domain()` are replaced. Variable name `peer_domain` UNCHANGED. NEVER touch the 2 Phase-6 siblings (L149-159, L261-271 — they are the CORRECT reference, already conformant). NEVER touch any other line, fn, or file. NEVER change `crates/**` elsewhere, schema_setup, Cargo.*, the plan, e2e.rs, any test. NEVER touch the fix-impl-2 #[expect(dead_code)] sites or the closure. A 2nd hunk or any other-file edit → STOP + kind:blocker."
serial: "Single-task barrier fix (Task 4 — the LAST fix-impl for Task 4), strictly serial cap=1 — only in-flight Junior for this lane. §15 (DQ #283 — 3 cmds: cargo-check + cargo-clippy --no-deps -- -D warnings + cargo-test --test e2e --no-run) is re-run by the ADVISOR on the laptop AFTER this fix lands + finalize-merge. Task 4 is COMPLETE only when this fix's §15 passes all 3 cmds; THEN advisor mutates DQ #283 result:pass. Cohort B (Tasks 5-7 [P], requires task 4) stays gated behind Task-4-complete + the separate serena/rust-analyzer OOM mitigation (swap-death eval 424/425) which the advisor surfaces to the user BEFORE the 3-way [P] dispatch."
---

# [role:impl-task] v1-federation-inbound-b fix-impl-3 — Finding 6.1: receive_remote_moderation_label domain hard-error (verbatim Phase-6 sibling mirror) — see .claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-3.md

> **Provenance:** fix-impl-2 (#338, worker commit f01a1d44e) is COMPLETE — §15 cold-reval on the laptop lane GREEN on all 3 cmds at tip `cee6b917c` (clippy `FI2COLD_CLIPPY_EXIT_0` 0-findings, check `FI2COLD_CHECK_EXIT_0`, e2e --no-run `FI2COLD_TESTNORUN_EXIT_0`); DQ #284 resolved advisor-answer Option A (`9ca42bd47`). This fix-impl-3 addresses **conformance-audit Finding 6.1** — a LATENT data-integrity footgun that `cargo check`/`clippy` do NOT catch. **User chose Option A + SEPARATE** (2026-05-19): a dedicated fix-impl mirroring the Phase-6 hard-error pattern. Classification: **latent-divergence fix, user-authorised** — but a verbatim sibling-mirror with ZERO design ambiguity (the canonical idiom is byte-identical in the two Phase-6 siblings in the same file). **This is the LAST fix-impl for Task 4**; DQ #283 resolves after this fix's §15 passes.

## §0 Pre-flight (subagent runs this FIRST, before reading anything else)

- Confirm CWD is a Junior worktree branched off `phase-v1-federation-inbound-b` (base tip `9ca42bd47`). `git merge-base --is-ancestor 9ca42bd47 HEAD` MUST be true. If on `phase-v1-federation-inbound-b` itself or a non-`junior/*` branch → STOP, file `kind: "blocker"` DQ into `.claude/decision-queue.json` (NOT a repo-root file unless the §4 harness-gap fires).
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — inside a forbidden window → exit non-zero `FORBIDDEN_WINDOW: <window>` unless dispatch carries `(user-authorised forbidden-window override per DQ #<id>)`. Shape G SUSPENDED (cargo on laptop) — keep the check.
- **Submodule init (MANDATORY before any in-worker cargo — per `feedback_worktree_submodules_not_auto_init`):** `git submodule update --init 2>&1` from worktree root. `crates/email/translations` is NOT auto-initialized in a fresh worktree; without it cargo fails pre-existing. Infra, not the fix.
- Confirm the 3 anchors are present + at the expected shape (grep the text, the line numbers are approximate — base tip 9ca42bd47):
  ```
  grep -nE "pub async fn receive_remote_moderation_label\(|pub async fn receive_remote_sanction_notice\(|pub async fn receive_remote_trust_attestation\(" crates/apub/activities/src/governance/inbox.rs
  ```
  Expected: `receive_remote_sanction_notice` ~L136, `receive_remote_trust_attestation` ~L249, `receive_remote_moderation_label` ~L733.
- Confirm the Finding-6.1 divergence is still present (un-fixed) at base: `grep -n "unwrap_or_default()" crates/apub/activities/src/governance/inbox.rs` MUST return **exactly ONE** line (~L743), and the 2 lines above it must be `.domain()` (~L741) then `.map(str::to_string)` (~L742). If `unwrap_or_default()` returns 0 → STOP + `kind:"blocker"` (premise changed — already fixed or refactored). If it returns >1 → STOP + `kind:"blocker"` (a 2nd `unwrap_or_default()` appeared — the cap-1-hunk assumption is invalid; surface for re-scoping).
- Confirm the 2 Phase-6 siblings are present + conformant (the reference shape): `grep -n "has no domain" crates/apub/activities/src/governance/inbox.rs` MUST return **exactly TWO** lines (the sanction-notice ~L155 + trust-attestation ~L267 messages). If ≠2 → STOP + `kind:"blocker"` (the canonical reference changed — do not proceed against a moved target).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b fix-impl-3 — Finding 6.1 domain hard-error (Phase-6 sibling mirror)`

Actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-b fix-impl-3 — see .claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-3.md
```

## §2 Scope

### 2.1 The defect being fixed (the contract — conformance-audit Finding 6.1, verified by advisor 2026-05-19)

In `crates/apub/activities/src/governance/inbox.rs`, `receive_remote_moderation_label` (Task-4 new code, fn ~L733) extracts the remote actor's domain like this (current, ~L738-743):

```rust
  let peer_domain = activity
    .actor
    .inner()
    .domain()
    .map(str::to_string)
    .unwrap_or_default();
```

A remote actor with **no domain** (`.domain()` → `None`) → `unwrap_or_default()` → `peer_domain = ""` (empty string). The label is then persisted with `source_instance = ''` — a silent data-integrity corruption (a domainless/malformed federation actor's moderation label is STORED, attributed to no instance).

The **two Phase-6 siblings in the SAME FILE** that do the structurally-identical job (extract the remote actor domain for a governance activity) **hard-error** instead — they are the canonical, already-shipped, already-conformant reference:

### 2.2 The canonical reference (VERBATIM — both siblings are byte-identical except the variable name + message noun)

**Sibling A — `receive_remote_sanction_notice` (current L149-159):**

```rust
  let source_instance = activity
    .actor
    .inner()
    .domain()
    .ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "remote sanction notice actor {} has no domain",
        activity.actor.inner(),
      ))
    })?
    .to_string();
```

**Sibling B — `receive_remote_trust_attestation` (current L261-271):**

```rust
  let peer_domain = activity
    .actor
    .inner()
    .domain()
    .ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "remote trust attestation actor {} has no domain",
        activity.actor.inner(),
      ))
    })?
    .to_string();
```

The idiom is **identical**: `.domain()` → `.ok_or_else(|| { LemmyErrorType::Unknown(format!("remote <kind> actor {} has no domain", activity.actor.inner(),)) })?` → `.to_string()`. Sibling B is the closest match (same variable name `peer_domain`). `LemmyErrorType` is already imported + used in this file (both siblings use it). There is **no design decision** here — the fix is to make `receive_remote_moderation_label` use the same tail as its siblings.

### 2.3 The fix (EXACTLY 1 hunk, 1 file — grep the anchor, do NOT blind-edit line numbers)

In `crates/apub/activities/src/governance/inbox.rs`, in `receive_remote_moderation_label` (~L738-743), the binding is:

```rust
  let peer_domain = activity
    .actor
    .inner()
    .domain()
    .map(str::to_string)
    .unwrap_or_default();
```

**Replace ONLY the last 2 lines** (`    .map(str::to_string)` and `    .unwrap_or_default();`) with the Phase-6 sibling's hard-error tail (mirroring Sibling B verbatim, changing only the message noun to `moderation label`). The result MUST be exactly:

```rust
  let peer_domain = activity
    .actor
    .inner()
    .domain()
    .ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "remote moderation label actor {} has no domain",
        activity.actor.inner(),
      ))
    })?
    .to_string();
```

- The 4 lines `let peer_domain = activity` / `.actor` / `.inner()` / `.domain()` are **UNCHANGED** (do not retype them — edit only from `.map(str::to_string)` onward).
- Indentation: 4-space base for the `let`, continuation lines at 4 spaces, `LemmyErrorType::Unknown(format!(` body indented to match Sibling B exactly (6 spaces for `LemmyErrorType`, 8 for the string + `activity.actor.inner(),`). **Copy Sibling B's exact whitespace** (read L261-271 and mirror byte-for-byte, swapping only `trust attestation` → `moderation label`).
- The message string is `"remote moderation label actor {} has no domain"` (noun = `moderation label`, matching the handler's subject; mirrors the A/B pattern `"remote <kind> actor {} has no domain"`).
- `peer_domain` is later used as `&str`/`String` downstream (e.g. `source_instance`) — `.to_string()` preserves the `String` type the `unwrap_or_default()` produced, so **no downstream type change** and **no other edit** is needed. (If the compiler reports a type mismatch downstream after this change, that would mean the prior `unwrap_or_default()` produced a different type than `.to_string()` — it does not; both yield `String`. If somehow it does, STOP + `kind:"blocker"`, do NOT patch downstream.)

**DO NOT** touch Sibling A (L149-159) or Sibling B (L261-271) — they are correct. **DO NOT** touch the fix-impl-2 `#[expect(dead_code)]` sites (~L470, ~L485) or the `std::sync::PoisonError::into_inner` closure (~L548). **DO NOT** add a 2nd hunk. If the change appears to need more than the 2-line→7-line replacement in this one location, STOP + `kind:"blocker"` DQ (the diagnosis would have missed something — surface, do not improvise).

### 2.4 Acceptance (the worker proves these before reporting done)

- `grep -c "unwrap_or_default()" crates/apub/activities/src/governance/inbox.rs` returns **0** (the only occurrence — the Finding-6.1 site — is gone).
- `grep -c "has no domain" crates/apub/activities/src/governance/inbox.rs` returns **3** (the 2 pre-existing siblings + the new `moderation label` one).
- `grep -n "remote moderation label actor {} has no domain" crates/apub/activities/src/governance/inbox.rs` returns **exactly 1** line (~L743 area).
- `grep -c "remote sanction notice actor {} has no domain\|remote trust attestation actor {} has no domain" crates/apub/activities/src/governance/inbox.rs` returns **2** (the siblings are UNCHANGED — the fix did not perturb them).
- `git diff --stat` shows **1 file changed**, `crates/apub/activities/src/governance/inbox.rs`, with a small net-line delta (≈ +5 / −2; one hunk only). `git diff` shows the hunk is confined to the `receive_remote_moderation_label` domain-extraction.
- §4.2 pre-push `cargo-check.bat --workspace --features full` is **GREEN** AND `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings` is **GREEN** (the `?` operator + `LemmyErrorType` are already used by the siblings in this file — the fn returns `LemmyResult<()>` so `?` propagation is valid; no new clippy lint should surface). Capture + verify the EXPLICIT markers (not the bg notification).

## §3 Required reading (IN ORDER before editing)

1. `crates/apub/activities/src/governance/inbox.rs` lines **149-159** (Sibling A `receive_remote_sanction_notice` domain hard-error) + **261-272** (Sibling B `receive_remote_trust_attestation` domain hard-error — the CLOSEST match, same variable name `peer_domain`; copy its whitespace byte-for-byte) + **733-749** (the Finding-6.1 site `receive_remote_moderation_label`, fn signature confirms `-> LemmyResult<()>` so `?` is valid, and the divergent `.map(str::to_string).unwrap_or_default()` at ~L742-743). This is the contract — mirror Sibling B exactly, swapping only the message noun.
2. `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md` — **PRIMARY**. New code (Task-4 handler) must mirror the canonical sibling's shape — including trust-boundary error handling, not just the happy path. Finding 6.1 is this class: the new handler copied the structure but weakened `.ok_or_else(…)?` to `.unwrap_or_default()`.
3. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — error-shape discipline: this fn returns `LemmyResult<()>`; `LemmyErrorType::Unknown(...)` + `?` is the in-file-proven pattern (both siblings do it). No `.map_err` bridge needed (this is NOT a `Box<dyn Error>` test context — it's a handler returning `LemmyResult<()>`).
4. `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` — §4.2 mandatory pre-push cargo gates + the explicit-marker rule (bg notification unreliable).

## §4 Constraints (HARD — violation = STOP + kind:blocker DQ)

1. **One commit.** Subject: `fix(v1-federation-inbound-b): receive_remote_moderation_label hard-errors on domainless actor (Phase-6 sibling mirror, Finding 6.1, fix-impl 3)`. Body: the 1-hunk before/after + the §2.4 grep-acceptance proof + the §4.2 pre-push cargo-check+clippy-green proof + "mirrors receive_remote_trust_attestation L261-271 verbatim, message noun = moderation label".
2. **Pre-push cargo gate (per `feedback_fix_impl_pre_push_cargo_check`):** before pushing the worker branch, run BOTH (each with an explicit marker, verify the marker in the FILE — the bg task-notification has lied repeatedly this phase, `feedback_background_task_notification_lies`):
   ```
   cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-fi3-check.log 2>&1 && echo FI3_CHECK_EXIT_0 >> .claude/PRPs/debug/fed-in-b-fi3-check.log || echo FI3_CHECK_EXIT_NONZERO >> .claude/PRPs/debug/fed-in-b-fi3-check.log"
   cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-fi3-clippy.log 2>&1 && echo FI3_CLIPPY_EXIT_0 >> .claude/PRPs/debug/fed-in-b-fi3-clippy.log || echo FI3_CLIPPY_EXIT_NONZERO >> .claude/PRPs/debug/fed-in-b-fi3-clippy.log"
   ```
   (Run as TWO SEPARATE `cmd //c` invocations — NOT one bat-`&&`-bat chain inside a single `cmd //c` [that trap broke a chain earlier this phase: bat `goto :eof` clobbers errorlevel between chained bats per `feedback_batch_goto_eof_clobbers_errorlevel`].) Either marker NONZERO → STOP, file `kind: "blocker"` DQ with the failing slice (≤120 lines). NEVER `#[allow]`-spam.
3. **DQ writes go into `.claude/decision-queue.json`** (the lane file at the worktree `.claude/` path). Compute `next_id = max(all ids across pending+resolved)+1` from `.claude/decision-queue.json` + `.claude/decision-queue-archive-*.json` (current max is 284 → next 285). Commit + push the DQ entry on the worker branch immediately (mid-task visibility). **Harness-gap (per DQ #235):** IF the `.claude/decision-queue.json` write is blocked by the Claude Code sensitive-file gate, write the JSON to a worktree-root `FI3_BLOCKER_DQ.json` + a short `FI3_ESCALATION.md` + commit both + push + STOP; advisor transcribes. (Happy path has NO `.claude/` write: you do NOT raise a validate-pending entry; DQ #283 is the advisor's to re-validate post-merge.)
4. **File-ownership:** edits ONLY to `crates/apub/activities/src/governance/inbox.rs`, EXACTLY the 1 hunk in §2.3. NEVER any other file, NEVER `schema_setup`, NEVER `Cargo.*`, NEVER `e2e.rs`/tests, NEVER the plan/ADRs, NEVER add `#[ignore]`/`#[allow]`, NEVER touch the 2 Phase-6 siblings or the fix-impl-2 sites.
5. **MIRROR-ref discipline:** Sibling B (`receive_remote_trust_attestation` L261-271) is the authoritative verbatim shape — copy its `.ok_or_else(|| { LemmyErrorType::Unknown(format!(...)) })?.to_string()` tail byte-for-byte (whitespace included), swapping ONLY the message noun `trust attestation` → `moderation label`. Do NOT invent a different error type, a different message format, a `.map_err`, or a `match`. If mirroring Sibling B verbatim does not compile, STOP + `kind:"blocker"` (the siblings compile — a verbatim copy must too; a discrepancy means the diagnosis missed context).
6. **Attribution:** worker `from: "impl"`; never `answered_by: "advisor"|"user"`; never `kind: "clarify"|"validate-pending"|"validate-result"|"validate-failed"` (you raise NO validate entry — DQ #283 is the advisor's to re-validate).
7. **Serial:** cap=1 — only in-flight Junior for this lane. This is Task 4's LAST fix-impl.

## §5 What "done" looks like

One commit on a `junior/*` worktree branch off `9ca42bd47` making EXACTLY the 1 hunk in §2.3 to `crates/apub/activities/src/governance/inbox.rs` (`receive_remote_moderation_label`'s domain extraction now hard-errors via the verbatim Phase-6 sibling pattern, message noun `moderation label`), with §2.4 grep-acceptance satisfied and §4.2 pre-push `cargo-check` + `cargo-clippy -D warnings` BOTH GREEN. No DQ entry on the happy path. The advisor then finalize-merge-reconciles the worker branch lane-safe (FF-verify into `phase-v1-federation-inbound-b`), re-runs the full §15 3-cmd chain (DQ #283's commands) on the laptop lane, and on **all 3 green** mutates DQ #283 `result: "pass"` → **Task 4 COMPLETE** (Story 2 of plan §16a). Cohort B (Tasks 5-7 [P], requires task 4) then becomes eligible — gated on the separate serena/rust-analyzer OOM mitigation which the advisor surfaces to the user BEFORE the 3-way [P] dispatch (swap-death eval 424/425).

## §6 Why this is a verbatim mirror, not a judgment call

There is no design decision in Finding 6.1's fix: the two Phase-6 siblings in the same file already establish the canonical trust-boundary contract (`.domain().ok_or_else(…)?.to_string()`), byte-identical except the variable name and message noun. `receive_remote_moderation_label` is a sibling of those handlers (same trait of work: receive a remote governance activity, extract the actor domain). The fix copy-adapts Sibling B verbatim. The ONLY way this goes wrong is a paraphrase (inventing a different error type/message/structure — forbidden by §4.5) or a blind line-edit (mitigated: grep the `unwrap_or_default()` anchor, §0 + §2.3) or scope-creep (mitigated: explicit 1-hunk/1-file cap + HARD exclusions of the siblings and fix-impl-2 sites). The latent-footgun *interpretation* (whether the §10.4 plan stub should have specified the trust-boundary error-shape, and whether advisor brief-authoring + plan-authoring need a canonical-sibling diff gate) is a **retro deliverable** per `project_phase6_convention_divergence_class.md` — NOT this fix's concern. This fix just makes the handler conformant with its siblings.

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; per DQ #279 the canonical session drives fed-in-b under serial-phase policy). Mirrors the canonical fix-impl brief schema `.claude/PRPs/briefs/federation-inbound-a-fix-impl-5.md` + the phase shape of `.claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-{1,2}.md` per `.claude/rules/advisor-orchestrator.md` §3.6. **Brief-defect lesson applied (from fix-impl-2 DQ #284):** the binding contract is the EXACT VERBATIM Phase-6 sibling block (Sibling A L149-159 + Sibling B L261-271, read in the lane worktree at base tip 9ca42bd47 before authoring), cited line-precise with byte-for-byte whitespace instruction — NOT a paraphrase; the brief is self-consistent (§2.2 reference shape and §2.3 fix are the same idiom, no contradiction); the scope is the narrowest possible (1 hunk, 2-line→7-line replacement in 1 location, hard-capped). Finding-6.1 divergence verified present at base (single `.map(str::to_string).unwrap_or_default()` after `.domain()` ~L742-743; exactly 2 sibling "has no domain" hard-errors L155/L267). Committed on `governance-v0` before the Junior fix-impl task is queued; then cherry-picked onto `phase-v1-federation-inbound-b` (conflict-free) so the worker — branched from the phase tip `9ca42bd47` (fix-impl-2 merged + §15-green + DQ #284 resolved) — sees it. /precheck re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC (= 9ca42bd47-or-later) before queue. User-authorised Option A + SEPARATE (advisor conformance-audit surface 2026-05-19)._
