---
phase: v1-federation-inbound-b
role: impl-task
task: 7
brief_n: 1
authored: 2026-05-19
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
plan_task: "§13 Task 7 [P] — publish_label.rs: fill stub Activity::receive + impl GovernanceInboundActivity (trait-default rate gate)"
parent_phase_tip: "Task 6 worktree merged + §15-green (Cohort B serial: Task 7 dispatched LAST, after Task 6 validates pass; phase tip = Task-6-merge sha, derive at dispatch time)"
cohort: "Cohort B (Tasks 5-7 [P]) — user chose cap-≤2 serial (AskUserQuestion 2026-05-19 'Cap cohort ≤2 serial'); Task 7 dispatched THIRD/LAST, ALONE, after Task 6 §15-green. requires: task 4."
related_dq: "276 (proceed-as-one), 279 (serial-phase), 283 (Task-4 validate pass — requires:4 precondition), 235 (no .claude/** writes by Junior)"
---

# [role:impl-task] v1-federation-inbound-b Task 7 — publish_label fill stub receive + impl GovernanceInboundActivity — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-7.md

> **Clarify provenance:** parent **planning** brief clarified via `/brehon-clarify` (DQ #273/#274/#275 resolved advisor-mode — all **option-a**, BINDING; DQ #276 split-or-proceed user-resolved **proceed-as-one**). PRECON-3 (BINDING) = §15 is `validate-pending-laptop` shape (Shape G SUSPENDED until 2026-06-01 per DQ #229). Impl-task brief; no clarify-DQ gates it directly. Cohort B serial-dispatch (5→6→7, one at a time) is the user's OOM-mitigation choice (AskUserQuestion 2026-05-19; rust-analyzer×3 ≈6.5 GB OOM-swap-killed the 15 GB EliteDesk in Cohort A — evals 424/425). Task 7 is the LAST Cohort-B task → on its §15-pass, Cohort B (§16a Story 3) is complete.

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is `phase-v1-federation-inbound-b` (you are on a Junior worktree branched off it). If `git branch --show-current` is anything else → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check (per `.claude/agents/impl-task.md` task-0 discipline): `date -u +"%a %H:%M UTC"` — if inside a forbidden window (Daily 02:55–04:15 / Sun 01:55–02:35 / Sun 03:55–04:30 / Wed 03:55–04:15 UTC) exit non-zero with `FORBIDDEN_WINDOW: <window>`. Shape G SUSPENDED (cargo on laptop); keep the self-check.
- **`requires: task 4` precondition self-verify:** Task 7 calls `wrap_governance_inbound` + `receive_remote_moderation_label` + impl's `GovernanceInboundActivity` — ALL Task-4 deliverables (the label handler `receive_remote_moderation_label` was authored in Task 4 and hardened by fix-impl-3 for Finding 6.1). Confirm: `grep -q 'pub(crate) async fn wrap_governance_inbound' crates/apub/activities/src/governance/inbox.rs && grep -q 'pub(crate) trait GovernanceInboundActivity' crates/apub/activities/src/governance/inbox.rs && grep -q 'pub async fn receive_remote_moderation_label' crates/apub/activities/src/governance/inbox.rs && echo PRECON-OK` — if any missing → STOP, file `kind: "blocker"` DQ (phase tip drifted / not branched off the Task-6-merge tip).
- **Task-5/6 sibling check** (Cohort B serial — 5 then 6 merged before 7): `git log phase-v1-federation-inbound-b --oneline --grep '(task 5)\|(task 6)' -i | head -2` should show BOTH. Read the merged `publish_sanction_notice.rs` (Task 5) `impl GovernanceInboundActivity` block as the canonical sibling shape (Task 7 = same structure, trait-default rate gate like Task 5, but the receive delegates to `receive_remote_moderation_label` and the stub is REPLACED not modified). If either missing → serial order broke; file `kind: "blocker"` DQ.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b Task 7 — publish_label.rs: REPLACE stub Activity::receive body with wrap_governance_inbound call delegating to receive_remote_moderation_label + add impl GovernanceInboundActivity for PublishLabel (trait-default check_per_actor_rate_limit)`

The actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-b Task 7 — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-7.md
```

## §2 Scope

**Produce** (one commit) — **single file, `crates/apub/activities/src/governance/publish_label.rs`** (plan §13 Task 7 FILES YAML: `modifies: [crates/apub/activities/src/governance/publish_label.rs]`, `creates: []`):

Per plan **§10.5 verbatim** (the contract — copy exactly):

1. **REPLACE the existing STUB `Activity::receive` body** (plan cites `:27-30` — the `_context`-ignored Phase-6 stub; verify by grep `fn receive` in this file) with a one-line `wrap_governance_inbound` call delegating to `crate::governance::inbox::receive_remote_moderation_label` per §10.5. NOTE: this is a STUB REPLACE (the Phase-6 body is a no-op stub, not a real handler) — unlike Task 5/6 which replace a real Phase-6 receive body. The end shape is identical (one-line wrap delegation); the difference is only that there's no meaningful prior logic to preserve.
2. **ADD `impl crate::governance::inbox::GovernanceInboundActivity for PublishLabel`** block: `activity_id`, `actor_domain`, `payload_size_bytes`, `payload_size_cap_key` (= `"federation.inbound.max_payload_bytes_moderation_label"` per §10.5). Uses the trait-DEFAULT `check_per_actor_rate_limit` (no override — like Task 5; the per-actor override is Task-6/trust-attestation-only).

**Do NOT** in this task:

- Touch `inbox.rs` (Task 4 — merged; you CALL `wrap_governance_inbound` + `receive_remote_moderation_label` + impl the trait, you do NOT edit it. `receive_remote_moderation_label` already hard-errors on domainless actor per fix-impl-3 Finding 6.1 — do NOT re-touch it), `publish_sanction_notice.rs` (Task 5 — merged), `publish_trust_attestation.rs` (Task 6 — merged), `error.rs`/protocol/`governance_log.rs` (Tasks 1-3), `scheduled_tasks.rs` (Task 8), `e2e.rs` (Task 9).
- Add a `check_per_actor_rate_limit` override — Task 7 uses the trait DEFAULT (label has no per-actor rate gate; that is Task 6's trust-attestation-only behaviour).
- Re-implement label-handling logic — `receive_remote_moderation_label` (Task 4, fix-impl-3-hardened) owns it. Your job is the stub→wrap-delegation swap + the trait-impl metadata block.

**Commit message** (exactly): `feat(v1-federation-inbound-b): publish_label — fill stub receive + impl GovernanceInboundActivity (task 7)`

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #283 (Task 4 `validate-pending-laptop` resolved **pass** — the `requires: task 4` precondition; `receive_remote_moderation_label` is on the phase tip, fix-impl-3-hardened), DQ #276 (proceed-as-one), DQ #279 (serial-phase).
2. **Plan §10.5** — the authoritative `publish_label` stub-fill + trait-impl code block (copy verbatim; this is the contract). The `:27-30` (stub body) line cite is plan-time; verify with grep.
3. **Plan §13 "Task 7"** — step list + FILES YAML + the `requires: task 4` reason + the "REPLACES the Phase-6 stub" note (vs Task 5/6 which modify a real receive body).
4. **Task 4's merged `inbox.rs`** (READ-ONLY): `grep -n 'pub(crate) trait GovernanceInboundActivity' -A 12 crates/apub/activities/src/governance/inbox.rs` (trait method signatures — mirror EXACTLY) + `grep -n 'pub async fn receive_remote_moderation_label' -A 6 inbox.rs` (the handler you delegate to — confirm its signature; note it now hard-errors on domainless actor per fix-impl-3, which is fine — the wrapper/handler owns that, you just delegate) + `grep -n 'pub(crate) async fn wrap_governance_inbound' -A 4 inbox.rs` (wrapper sig you call).
5. **Task 5's merged `publish_sanction_notice.rs`** (READ-ONLY, canonical sibling — Cohort B serial means Task 5 + 6 merged before you): `grep -n 'impl.*GovernanceInboundActivity' -A 20 crates/apub/activities/src/governance/publish_sanction_notice.rs` — mirror its `impl` block STRUCTURE (Task 7 = Task-5 shape: 4 metadata methods + trait-default rate gate; the ONLY differences are `payload_size_cap_key` value + the receive delegates to `receive_remote_moderation_label`). This is the canonical-schema-first source.
6. **MIRROR refs** — the existing Phase-6 STUB `Activity::receive` body in `publish_label.rs` (`grep -n 'fn receive' -A 8` for its real position — it's a `_context`-ignored no-op stub; you REPLACE it entirely per §10.5).
7. **Lessons** (MANDATORY per `.claude/rules/advisor-orchestrator.md` §2.4 — Task 7 runs `cargo-clippy.bat ... -D warnings`; `publish_label.rs` is an apub activity handler):
   - `.claude/lessons/feedback_clippy_test_style.md` + `.claude/lessons/feedback_clippy_rerun_after_fix.md` — **Why:** the new trait-impl block + the filled receive must be clippy-clean under `-D warnings` (deny `unwrap`/`expect`/`#[allow]`). The Phase-6 stub likely had `_context` / `_` params that the real wrap call now USES — make sure no leftover `_`-prefixed unused bindings remain (clippy will flag a now-used param still named `_context`). Re-run clippy after any fix.
   - `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Why:** `actor_domain` returns `LemmyResult<String>`; the wrapped `receive` returns `LemmyResult<()>`. Follow the canonical LemmyError shape (no `Box<dyn Error>` bridge); mirror Task 5's merged sibling error shape verbatim.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — capture-then-tail for the §5 commands; you do not run them yourself).

## §3a Handover from prior cohort

> **Handover state: DEGRADED** (per `.claude/rules/advisor-orchestrator.md` §4.3 — the Task-4 chain commits carry no `HANDOVER:` YAML trailer; Task 5/6 handover is read directly from their merged sources per §3 reading #5, not trailers). Synthesized by the advisor from the Task-4 public-API deliverable + the Task-5 canonical-sibling fact. The §0 precon self-verify + Task-5/6-sibling check grep the actual symbols on the phase tip as the authoritative checks.

```yaml
prior_cohort_tasks:
  - task: 4
    commit: 4a60667c9 (Task 4 enforcement core + fix-impl-3 Finding 6.1; §15-green per DQ #283 resolved pass 2026-05-19T20:44:53Z)
    filesModified: [crates/apub/activities/src/governance/inbox.rs]
    keyDecisions:
      - "pub(crate) trait GovernanceInboundActivity (activity_id/actor_domain/payload_size_bytes/payload_size_cap_key + DEFAULT check_per_actor_rate_limit). Task 7 impl's it with the DEFAULT rate gate (no override — like Task 5; per-actor override is Task-6-only)."
      - "pub async fn receive_remote_moderation_label — the Task-4 label handler, HARDENED by fix-impl-3 (Finding 6.1: hard-errors on domainless actor via Phase-6 Sibling-B mirror). Task 7's filled receive delegates to it through wrap_governance_inbound. Task 7 does NOT touch receive_remote_moderation_label — it is complete + §15-green."
      - "pub(crate) async fn wrap_governance_inbound — the enforcement wrapper. Task 7's STUB receive body is REPLACED with a one-line delegation through it to receive_remote_moderation_label (per §10.5)."
    notes: "Task 7 CONSUMES Task 4's wrapper + trait + the (fix-impl-3-hardened) receive_remote_moderation_label. The ONLY structural difference vs Task 5: Task 7 REPLACES a no-op stub (not a real Phase-6 receive body) and delegates to the moderation-label handler; behaviourally identical end-shape (one-line wrap delegation + trait-default rate gate)."
  - task: 5
    commit: "(Task-5-merge sha — derive at dispatch via `git log phase-v1-federation-inbound-b --grep '(task 5)' -i`)"
    filesModified: [crates/apub/activities/src/governance/publish_sanction_notice.rs]
    keyDecisions:
      - "Canonical Cohort-B impl-block shape: Activity::receive → one-line wrap_governance_inbound delegation + impl GovernanceInboundActivity with 4 metadata methods, trait-default check_per_actor_rate_limit. Task 7 mirrors this EXACTLY (only payload_size_cap_key value + the receive's delegate-target differ)."
    notes: "Read Task 5's merged publish_sanction_notice.rs impl block (§3 reading #5) as the canonical-schema-first sibling. Task 7 is the closest twin of Task 5 (both trait-default rate gate); Task 6 was the outlier (per-actor override)."
  - task: 6
    commit: "(Task-6-merge sha — derive at dispatch via `git log phase-v1-federation-inbound-b --grep '(task 6)' -i`; Cohort B serial = Task 6 merged + §15-green BEFORE Task 7 dispatched)"
    filesModified: [crates/apub/activities/src/governance/publish_trust_attestation.rs]
    keyDecisions:
      - "Task 6 added the per-actor check_per_actor_rate_limit override (trust-attestation only). Task 7 does NOT replicate this — Task 7 uses the trait default. Listed only so Task 7 does not mistakenly copy Task 6's override into PublishLabel."
    notes: "If Task 5 OR Task 6 not yet merged → serial order broke (§0 Task-5/6-sibling check files a blocker). Task 7 is the LAST Cohort-B task; its §15-pass completes §16a Story 3."
```

## §4 Constraints (hard rules)

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-federation-inbound-b` (tip = Task-6-merge sha, Tasks 4+5+6 complete + §15-green). Finalize merges your worktree branch back; do NOT push to `phase-v1-federation-inbound-b` directly.
- One commit (single file). If clippy fails on first attempt, amend/fixup — do NOT split.
- Mid-task DQ visibility: if you raise a `pending` entry, **commit + push immediately** to your worktree branch per `.claude/rules/decision-queue.md` "Mid-task visibility".
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235 — interim escalation-and-transcribe)

If you need to write a DQ entry to `.claude/decision-queue.json` and the Claude Code sensitive-file gate blocks it (the same gate that blocked planning #330 + bm-cut #331 + Task-3 #335 + Task-4 #336): (a) write the intended DQ-entry JSON object to a worktree-root file `TASK7_BLOCKER_DQ.json` (or `TASK7_VALIDATE_PENDING.json` for §5), (b) write a short `TASK7_ESCALATION.md` naming the issue, (c) commit both at worktree root + push, (d) STOP. The advisor transcribes per `.claude/rules/escalation.md`. Task 7 has NO `.claude/` deliverable — the happy path's only `.claude/` write is the §5 `validate-pending-laptop` entry; if THAT is gated, use this path with `TASK7_VALIDATE_PENDING.json` (likely, given #335/#336's experience — be ready for it).

### Task-7 GOTCHAs (from plan §13 Task 7 + §10.5 — all load-bearing)

- **Single-file, single-purpose:** ONLY `publish_label.rs`. The wrapper + trait + `receive_remote_moderation_label` are Task-4 deliverables — CALL/impl them, never edit `inbox.rs`. `receive_remote_moderation_label` is fix-impl-3-hardened + §15-green; do NOT re-touch it. If you find yourself editing `inbox.rs` → STOP, file `kind: "blocker"`.
- **STUB REPLACE, not modify:** the Phase-6 `Activity::receive` is a `_context`-ignored no-op stub. You REPLACE it entirely with the §10.5 one-line wrap delegation. There's no prior logic to preserve (contrast Task 5/6 which modify real Phase-6 receive bodies). Watch for leftover `_`-prefixed params: the stub likely named the context `_context` (unused); the real wrap call USES the context — rename to `context` (clippy flags a now-used `_`-prefixed binding).
- **Trait-default rate gate (NO override):** `PublishLabel`'s `GovernanceInboundActivity` impl uses the DEFAULT `check_per_actor_rate_limit`. Do NOT copy Task 6's per-actor override. Label rate enforcement is the per-PEER gate inside `wrap_governance_inbound` (Task 4), not a per-actor override here.
- **payload_size_cap_key string is exact:** `"federation.inbound.max_payload_bytes_moderation_label"` (per §10.5). Verbatim, not abbreviated.
- **Delegate target is receive_remote_moderation_label:** the wrap call delegates to `crate::governance::inbox::receive_remote_moderation_label` (NOT `receive_remote_sanction_notice`/`receive_remote_trust_attestation` — those are Task 5/6's targets). Copy the delegate symbol exactly from §10.5.
- **Trait signature fidelity:** read the EXACT `GovernanceInboundActivity` method signatures from the merged `inbox.rs` (§3 reading #4) and mirror them verbatim. Mirror Task-5's merged impl-block shape for the 4 metadata methods (§3 reading #5). A signature mismatch is an E0xxx the §15 check/clippy catches.

### Plan-cited line numbers may have drifted

§10.5 is the contract for *content*. The `:27-30` (stub body) line cite + the Task-5 impl-block position are runtime-derived. `grep -n 'fn receive'` + `grep -n 'impl Activity'` in `publish_label.rs`; `grep -n 'impl.*GovernanceInboundActivity'` in the merged `publish_sanction_notice.rs`. If the Phase-6 stub or Task-5's impl shape diverges from §10.5's documented structure, file a DQ pending entry — the plan may have drifted.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

**Shape-G suspended until 2026-06-01** (DQ #229/#276). After committing + pushing your worktree branch, write a `kind: "validate-pending-laptop"` DQ entry (per `.claude/agents/impl-task.md` "Pre-Shape-G plans" + `.claude/rules/decision-queue.md` `kind: "validate-pending-laptop"`) with `from: "impl"`, `phase_task: 7`, `branch: <your-worktree-branch>`, and `commands[]` set **verbatim** to (per plan §13 Task 7 "Push and exit" — exactly 2 commands, NO `cargo-test --no-run` for Task 7):

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task7-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task7-clippy.log 2>&1"
```

Do NOT write `kind: "validate-pending"` and do NOT capture a `workflow_run_id` — there is no GitHub Actions run (workflows disabled until 2026-06-01). The advisor laptop session reads this entry, runs the 2 commands locally (serialized — one `target/`), and mutates the entry (`result: pass|fail`).

If the `.claude/decision-queue.json` write is gated by the sensitive-file gate, use the §4 harness-gap escalation path with `TASK7_VALIDATE_PENDING.json` at worktree root. (Per #335/#336's experience this gate is likely to fire — author the escalation files cleanly if so; it is NOT a blocker.)

Per `feedback_pipes_mask_exit_codes.md`: you do not run these commands yourself (the laptop advisor does) — your job is only to apply the §10.5 implementation + raise the entry with the commands verbatim.

## §6 Expected output (return to advisor)

```
## Task 7 complete — v1-federation-inbound-b publish_label stub-fill + trait-impl

**Commit:** <sha> on <worktree-branch>
**File changed:**
  - crates/apub/activities/src/governance/publish_label.rs (Phase-6 stub Activity::receive REPLACED → one-line wrap_governance_inbound delegation to receive_remote_moderation_label; +impl GovernanceInboundActivity for PublishLabel [4 metadata methods, trait-default check_per_actor_rate_limit])
**PRECON self-check:** wrap_governance_inbound + trait + receive_remote_moderation_label present in inbox.rs (confirmed — requires: task 4)
**Task-5/6 siblings:** both merged + read; Task-5 impl block mirrored for the 4 metadata methods (canonical-schema-first); Task-6 per-actor override NOT copied (Task 7 = trait default)
**Stub-replace:** Phase-6 no-op stub fully replaced; no leftover `_`-prefixed now-used params
**payload_size_cap_key:** "federation.inbound.max_payload_bytes_moderation_label" (verbatim §10.5)
**Delegate target:** receive_remote_moderation_label (correct — not sanction/attestation)
**No inbox.rs edit:** confirmed
**validate-pending-laptop DQ:** #<id> raised (commands: cargo-check.bat + cargo-clippy.bat -D warnings — 2 cmds, no e2e-norun for Task 7)
**Next:** advisor laptop runs §15 (2 cmds), mutates DQ #<id>; on pass → Cohort B COMPLETE (§16a Story 3 done); advisor advances to Task 8 (barrier — replay-cleanup cron)
```

Plus any DQ #N references if you raised a blocker mid-task (NOT for a gated §5 DQ write — that's an escalation case per §4).

## §7 Why this brief differs from the plan

It does not — Task 7's scope is exactly plan §10.5 + §13 Task 7. This brief adds only: (a) §0 forbidden-window self-check + the `requires: task 4` precondition self-verify + the **Task-5/6-sibling check** (Cohort B serial means 5 then 6 merged + §15-green first), (b) §4 harness-gap escalation path + the explicit GOTCHAs incl. the STUB-REPLACE-not-modify rule + the leftover-`_context`-param footgun (the highest-risk Task-7-specific clippy trap — the stub ignored the context; the real wrap call uses it) + "do NOT copy Task 6's override", (c) §5 explicit `validate-pending-laptop` shape per PRECON-3 with the **2-command** set (NO `cargo-test --test e2e --no-run` — Task-4-specific R7 checkpoint; Task 7's plan "Push and exit" lists only check + clippy), (d) the delegate-target + trait-signature-fidelity GOTCHAs. The stub-fill + trait-impl are §10.5 verbatim — do not deviate. Mirrors the canonical sibling `.claude/PRPs/briefs/v1-federation-inbound-b-impl-4.md` schema (same §0-§7 structure) AND uses Task-5's merged `publish_sanction_notice.rs` as the canonical-schema-first impl-block source per `.claude/rules/advisor-orchestrator.md` §3.6.

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; per DQ #279 the canonical session drives fed-in-b impl under serial-phase policy). Brief committed on `governance-v0` before the Junior Task-7 task is queued (queued only AFTER Task 6 validates §15-pass — Cohort B serial per the user's cap-≤2 OOM-mitigation choice); then propagated onto `phase-v1-federation-inbound-b` so the worker — which branches from the PHASE branch tip (Task-6-merge sha) — sees it. `/precheck` re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC before queue. Task 7 is the LAST Cohort-B task: its §15-pass completes §16a Story 3; the advisor then advances to Task 8 (barrier). Handover from Task 4 is DEGRADED (no HANDOVER: trailer) — synthesized in §3a; Task 5/6 handover is read directly from their merged sources (§3 reading #5)._
