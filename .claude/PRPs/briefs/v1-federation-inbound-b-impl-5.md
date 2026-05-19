---
phase: v1-federation-inbound-b
role: impl-task
task: 5
brief_n: 1
authored: 2026-05-19
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
plan_task: "§13 Task 5 [P] — publish_sanction_notice.rs: wrap Activity::receive + impl GovernanceInboundActivity"
parent_phase_tip: 4a60667c9 (phase-v1-federation-inbound-b @ Task 4 complete + §15-green: wrapper+trait+helpers+label-handler merged, DQ #283 resolved pass)
cohort: "Cohort B (Tasks 5-7 [P]) — user chose cap-≤2 serial (DQ-equivalent: AskUserQuestion 2026-05-19 'Cap cohort ≤2 serial'); Task 5 dispatched FIRST, ALONE, ahead of Task 6 then Task 7. requires: task 4."
related_dq: "276 (proceed-as-one), 279 (serial-phase), 283 (Task-4 validate pass — the requires:4 precondition), 235 (no .claude/** writes by Junior)"
---

# [role:impl-task] v1-federation-inbound-b Task 5 — publish_sanction_notice wrap receive + impl GovernanceInboundActivity — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-5.md

> **Clarify provenance:** parent **planning** brief clarified via `/brehon-clarify` (DQ #273/#274/#275 resolved advisor-mode — all **option-a**, BINDING; DQ #276 split-or-proceed user-resolved **proceed-as-one**). PRECON-3 (BINDING) = §15 is `validate-pending-laptop` shape (Shape G SUSPENDED until 2026-06-01 per DQ #229). Impl-task brief; no clarify-DQ gates it directly. Cohort B serial-dispatch order (5→6→7, one at a time) is the user's OOM-mitigation choice (AskUserQuestion 2026-05-19; rust-analyzer×3 ≈6.5 GB OOM-swap-killed the 15 GB EliteDesk in Cohort A — evals 424/425).

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is `phase-v1-federation-inbound-b` (you are on a Junior worktree branched off it). If `git branch --show-current` is anything else → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check (per `.claude/agents/impl-task.md` task-0 discipline): `date -u +"%a %H:%M UTC"` — if inside a forbidden window (Daily 02:55–04:15 / Sun 01:55–02:35 / Sun 03:55–04:30 / Wed 03:55–04:15 UTC) exit non-zero with `FORBIDDEN_WINDOW: <window>`. Shape G SUSPENDED (cargo on laptop); keep the self-check.
- **`requires: task 4` precondition self-verify:** Task 5 calls `wrap_governance_inbound` + impl's the `GovernanceInboundActivity` trait — BOTH land in Task 4, merged on the phase tip you branch from (`4a60667c9`, Task 4 complete + §15-green). Confirm: `grep -q 'pub(crate) async fn wrap_governance_inbound' crates/apub/activities/src/governance/inbox.rs && grep -q 'pub(crate) trait GovernanceInboundActivity' crates/apub/activities/src/governance/inbox.rs && echo PRECON-OK` — if either missing → STOP, file `kind: "blocker"` DQ (phase tip drifted / not branched off 4a60667c9+).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b Task 5 — publish_sanction_notice.rs: replace Activity::receive body with wrap_governance_inbound call + add impl GovernanceInboundActivity for PublishSanctionNotice`

The actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-b Task 5 — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-5.md
```

## §2 Scope

**Produce** (one commit) — **single file, `crates/apub/activities/src/governance/publish_sanction_notice.rs`** (plan §13 Task 5 FILES YAML: `modifies: [crates/apub/activities/src/governance/publish_sanction_notice.rs]`, `creates: []`):

Per plan **§10.5 verbatim** (the contract — copy it exactly; §10.5's first block is the `publish_sanction_notice` receive-wrap + trait-impl):

1. **REPLACE the existing `Activity::receive` body** (plan cites `:103-110` — verify by grep `fn receive` in this file) with a one-line `wrap_governance_inbound` call per §10.5 (delegating to `crate::governance::inbox::receive_remote_sanction_notice`).
2. **ADD `impl crate::governance::inbox::GovernanceInboundActivity for PublishSanctionNotice`** block (placed after the `impl Activity` block, plan cites ~`:112` — verify by grep). This impl uses the trait-DEFAULT `check_per_actor_rate_limit` (no override — sanction-notice has no per-actor rate gate; that's Task 6's trust-attestation only). Implement only: `activity_id`, `actor_domain`, `payload_size_bytes`, `payload_size_cap_key` (= `"federation.inbound.max_payload_bytes_sanction_notice"`) per §10.5.

**Do NOT** in this task:

- Touch `inbox.rs` (Task 4 — merged; you CALL its `wrap_governance_inbound` + impl its trait, you do NOT edit it), `publish_trust_attestation.rs` (Task 6 — separate Cohort B task), `publish_label.rs` (Task 7 — separate Cohort B task), `error.rs` / protocol structs / `governance_log.rs` (Tasks 1-3 merged), `scheduled_tasks.rs` (Task 8), `e2e.rs` (Task 9).
- Rewrite the `wrap_governance_inbound` wrapper or the `GovernanceInboundActivity` trait — they are Task-4 deliverables on the phase tip. Your job is the one-line receive-body swap + the trait-impl block per §10.5.
- Re-implement peer-trust / size / schema / rate / replay gates — those live IN `wrap_governance_inbound` (Task 4). The wrapper does the enforcement; this task only wires `publish_sanction_notice`'s `receive` through it + provides the trait-impl metadata.

**Commit message** (exactly): `feat(v1-federation-inbound-b): publish_sanction_notice — wrap receive + impl GovernanceInboundActivity (task 5)`

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #283 (Task 4 `validate-pending-laptop` resolved **pass** — this is the `requires: task 4` precondition; Task 4's wrapper+trait are §15-green on the phase tip), DQ #276 (proceed-as-one), DQ #279 (serial-phase). DQ #273/#274/#275 option-a shaped Task 4's wrapper internals — you do NOT re-apply them (they're already in the merged `wrap_governance_inbound`); read them only to understand what the wrapper enforces so your trait-impl metadata is consistent.
2. **Plan §10.5** — the authoritative `publish_sanction_notice` receive-wrap + trait-impl code block (copy verbatim; this is the contract). The `:103-110` (receive body) / `:112` (impl-block insertion point) line cites are plan-time; verify with grep.
3. **Plan §13 "Task 5"** — step list + FILES YAML + the `requires: task 4` reason.
4. **Task 4's merged `inbox.rs`** (READ-ONLY, for the trait signature you're impl'ing): `grep -n 'pub(crate) trait GovernanceInboundActivity' -A 12 crates/apub/activities/src/governance/inbox.rs` to read the EXACT trait method signatures (`activity_id`, `actor_domain`, `payload_size_bytes`, `payload_size_cap_key`, default `check_per_actor_rate_limit`) + `grep -n 'pub(crate) async fn wrap_governance_inbound' -A 4 inbox.rs` for the wrapper signature you call. Mirror these signatures EXACTLY in your impl block.
5. **MIRROR refs** — the existing Phase-6 `Activity::receive` body in `publish_sanction_notice.rs` (the swap target — `grep -n 'fn receive' -A 10 crates/apub/activities/src/governance/publish_sanction_notice.rs` for its real position; line cites may have drifted) + plan §10.5's first block as the exact replacement shape. **Also** read the sibling pattern if a Cohort-B peer already merged: `git log phase-v1-federation-inbound-b --oneline --grep '(task 6)\|(task 7)' -i` — if a peer task's `impl GovernanceInboundActivity` block already landed in `publish_trust_attestation.rs`/`publish_label.rs`, mirror its structure (Task 5 is dispatched FIRST so likely none yet — but check).
6. **Lessons** (MANDATORY per `.claude/rules/advisor-orchestrator.md` §2.4 — Task 5 runs `cargo-clippy.bat ... -D warnings`; `publish_sanction_notice.rs` is an apub activity handler):
   - `.claude/lessons/feedback_clippy_test_style.md` + `.claude/lessons/feedback_clippy_rerun_after_fix.md` — **Why:** the workspace clippy config denies `unwrap`/`expect`/`#[allow]`; the new trait-impl block + the wrapped receive must be clippy-clean under `-D warnings`. Re-run clippy after any fix (stale pass ≠ evidence).
   - `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Why:** `actor_domain` returns `LemmyResult<String>`; the wrapped `receive` returns `LemmyResult<()>`. Error propagation must follow the canonical LemmyError shape (no `Box<dyn Error>` bridge). Mirror the sibling apub activities' existing error shape verbatim.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — capture-then-tail for the §5 commands; you do not run them yourself).

## §3a Handover from prior cohort

> **Handover state: DEGRADED** (per `.claude/rules/advisor-orchestrator.md` §4.3 — the Task-4 chain commits carry no `HANDOVER:` YAML trailer). Synthesized by the advisor from the Task-4 public-API deliverable (the contract Cohort B consumes). This is NOT a catch-fire; the §0 precon self-verify greps the actual symbols on the phase tip as the authoritative check.

```yaml
prior_cohort_tasks:
  - task: 4
    commit: 4a60667c9 (Task 4 enforcement core; chain a3757eabe impl → cdff6f09d fix-impl-1 → f01a1d44e/45374097b fix-impl-2 → 8b04e69a6/4a60667c9 fix-impl-3; all on phase-v1-federation-inbound-b, §15-green per DQ #283 resolved pass 2026-05-19T20:44:53Z)
    filesModified: [crates/apub/activities/src/governance/inbox.rs]
    keyDecisions:
      - "pub(crate) trait GovernanceInboundActivity added to inbox.rs: methods activity_id() -> &Url, actor_domain() -> LemmyResult<String>, payload_size_bytes() -> LemmyResult<usize>, payload_size_cap_key() -> &'static str, + DEFAULT async fn check_per_actor_rate_limit(&self, &Data<LemmyContext>) -> LemmyResult<()> { Ok(()) }. Task 5 impl's this trait for PublishSanctionNotice using the default check_per_actor_rate_limit (no override)."
      - "pub(crate) async fn wrap_governance_inbound — the 7-step enforcement wrapper. Task 5's Activity::receive body becomes a one-line delegation through it to crate::governance::inbox::receive_remote_sanction_notice (per §10.5)."
      - "pub(crate) helpers rate_per_peer_counts(), rate_per_actor_counts(), current_hour_bucket(), log_inbox_drop — exposed for Cohort B (Task 6's per-actor override uses them; Task 5 does NOT — sanction-notice uses trait-default rate gate)."
      - "pub async fn receive_remote_sanction_notice — the Phase-6 handler, surgically modified by Task 4 (eviction call + best-effort persist_failed emit) but its signature is unchanged; wrap_governance_inbound delegates to it. Task 5 does NOT touch it."
      - "fix-impl-3 (Finding 6.1): receive_remote_moderation_label now hard-errors on domainless actor (Phase-6 Sibling-B mirror) — relevant to Task 7, not Task 5; noted for cohort consistency."
    notes: "Task 5 CONSUMES Task 4's wrapper + trait (calls/impl's, never edits inbox.rs). The §0 precon self-verify greps `wrap_governance_inbound` + `GovernanceInboundActivity` in inbox.rs on the phase tip as the authoritative existence check. Trait method signatures: read them verbatim from inbox.rs per §3 reading #4 and mirror EXACTLY (do not paraphrase the signatures)."
```

## §4 Constraints (hard rules)

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-federation-inbound-b` (tip `4a60667c9`+, Task 4 complete + §15-green). Finalize merges your worktree branch back; do NOT push to `phase-v1-federation-inbound-b` directly.
- One commit (single file). If clippy fails on first attempt, amend/fixup — do NOT split.
- Mid-task DQ visibility: if you raise a `pending` entry, **commit + push immediately** to your worktree branch per `.claude/rules/decision-queue.md` "Mid-task visibility".
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235 — interim escalation-and-transcribe)

If you need to write a DQ entry to `.claude/decision-queue.json` and the Claude Code sensitive-file gate blocks it (the same gate that blocked planning #330 + bm-cut #331 + Task-3 #335 + Task-4 #336): (a) write the intended DQ-entry JSON object to a worktree-root file `TASK5_BLOCKER_DQ.json` (or `TASK5_VALIDATE_PENDING.json` for §5), (b) write a short `TASK5_ESCALATION.md` naming the issue, (c) commit both at worktree root + push, (d) STOP. The advisor transcribes per `.claude/rules/escalation.md`. Task 5 has NO `.claude/` deliverable — the happy path's only `.claude/` write is the §5 `validate-pending-laptop` entry; if THAT is gated, use this path with `TASK5_VALIDATE_PENDING.json` (likely, given #335/#336's experience — be ready for it).

### Task-5 GOTCHAs (from plan §13 Task 5 + §10.5 — all load-bearing)

- **Single-file, single-purpose:** ONLY `publish_sanction_notice.rs`. The wrapper + trait are Task-4 deliverables on the phase tip — CALL/impl them, never edit `inbox.rs`. If you find yourself editing `inbox.rs` → STOP, file `kind: "blocker"`.
- **Trait-default rate gate:** `PublishSanctionNotice`'s `GovernanceInboundActivity` impl uses the DEFAULT `check_per_actor_rate_limit` (the `Ok(())` no-op). Do NOT add a per-actor override — that is Task 6's trust-attestation-specific behaviour. Sanction-notice's rate enforcement is the per-PEER gate inside `wrap_governance_inbound` (Task 4), not a per-actor override here.
- **payload_size_cap_key string is exact:** `"federation.inbound.max_payload_bytes_sanction_notice"` (per §10.5). Not a guessed/abbreviated key — copy it verbatim from §10.5.
- **Receive-body is a one-line delegation:** the new `Activity::receive` body is a single `wrap_governance_inbound(...)` call delegating to `crate::governance::inbox::receive_remote_sanction_notice` per §10.5's first block. Do NOT inline any enforcement logic — the wrapper owns it.
- **Trait signature fidelity:** read the EXACT `GovernanceInboundActivity` method signatures from the merged `inbox.rs` (§3 reading #4) and mirror them verbatim in your impl block. A signature mismatch (wrong return type, missing `async`, wrong `&Url` vs `Url`) is an E0xxx the §15 clippy/check will catch — get it right from the merged source, not from memory.

### Plan-cited line numbers may have drifted

§10.5 is the contract for *content*. The `:103-110` (receive body) / `:112` (impl-block insertion point) line cites are plan-time. `grep -n 'fn receive'` + `grep -n 'impl Activity'` in `publish_sanction_notice.rs` for real positions. If the Phase-6 receive body no longer matches §10.5's documented "before" shape, file a DQ pending entry — the plan may have drifted.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

**Shape-G suspended until 2026-06-01** (DQ #229/#276). After committing + pushing your worktree branch, write a `kind: "validate-pending-laptop"` DQ entry (per `.claude/agents/impl-task.md` "Pre-Shape-G plans" + `.claude/rules/decision-queue.md` `kind: "validate-pending-laptop"`) with `from: "impl"`, `phase_task: 5`, `branch: <your-worktree-branch>`, and `commands[]` set **verbatim** to (per plan §13 Task 5 "Push and exit" — exactly 2 commands, NO `cargo-test --no-run` for Task 5):

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task5-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task5-clippy.log 2>&1"
```

Do NOT write `kind: "validate-pending"` and do NOT capture a `workflow_run_id` — there is no GitHub Actions run (workflows disabled until 2026-06-01). The advisor laptop session reads this entry, runs the 2 commands locally (serialized — one `target/`), and mutates the entry (`result: pass|fail`).

If the `.claude/decision-queue.json` write is gated by the sensitive-file gate, use the §4 harness-gap escalation path with `TASK5_VALIDATE_PENDING.json` at worktree root. (Per #335/#336's experience this gate is likely to fire — author the escalation files cleanly if so; it is NOT a blocker.)

Per `feedback_pipes_mask_exit_codes.md`: you do not run these commands yourself (the laptop advisor does) — your job is only to apply the §10.5 implementation + raise the entry with the commands verbatim.

## §6 Expected output (return to advisor)

```
## Task 5 complete — v1-federation-inbound-b publish_sanction_notice wrap + trait-impl

**Commit:** <sha> on <worktree-branch>
**File changed:**
  - crates/apub/activities/src/governance/publish_sanction_notice.rs (Activity::receive body → one-line wrap_governance_inbound delegation to receive_remote_sanction_notice; +impl GovernanceInboundActivity for PublishSanctionNotice [activity_id/actor_domain/payload_size_bytes/payload_size_cap_key, trait-default check_per_actor_rate_limit])
**PRECON self-check:** wrap_governance_inbound + GovernanceInboundActivity present in inbox.rs on phase tip (confirmed — requires: task 4 satisfied)
**Trait signatures:** mirrored verbatim from merged inbox.rs (no paraphrase)
**payload_size_cap_key:** "federation.inbound.max_payload_bytes_sanction_notice" (verbatim from §10.5)
**No inbox.rs edit:** confirmed (wrapper/trait CALLED, not modified)
**validate-pending-laptop DQ:** #<id> raised (commands: cargo-check.bat + cargo-clippy.bat -D warnings — 2 cmds, no e2e-norun for Task 5)
**Next:** advisor laptop runs §15 (2 cmds), mutates DQ #<id>; on pass → Task 6 dispatched (Cohort B serial, user cap-≤2 choice)
```

Plus any DQ #N references if you raised a blocker mid-task (NOT for a gated §5 DQ write — that's an escalation case per §4, not a blocker).

## §7 Why this brief differs from the plan

It does not — Task 5's scope is exactly plan §10.5 (first block) + §13 Task 5. This brief adds only: (a) §0 forbidden-window self-check + the `requires: task 4` precondition self-verify (Task 4 merged + §15-green on `4a60667c9` per DQ #283), (b) §4 harness-gap escalation path (likely to fire for the §5 DQ write per #335/#336's experience) + the explicit GOTCHAs from §13 Task 5 + §10.5, (c) §5 explicit `validate-pending-laptop` shape per PRECON-3 with the **2-command** set (NO `cargo-test --test e2e --no-run` — that R7 checkpoint was Task-4-specific; Task 5's plan "Push and exit" lists only check + clippy), (d) explicit "plan line numbers may have drifted — grep the named symbols" + the trait-signature-fidelity GOTCHA. The receive-wrap + trait-impl are §10.5 verbatim — do not deviate. Mirrors the canonical sibling `.claude/PRPs/briefs/v1-federation-inbound-b-impl-4.md` schema per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate (same §0-§7 brief STRUCTURE; §10.5 content; Task 4 was the barrier authoring the wrapper/trait, Task 5 is the first Cohort-B consumer wiring one `publish_*.rs` through it).

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; per DQ #279 the canonical session drives fed-in-b impl under serial-phase policy). Brief committed on `governance-v0` before the Junior Task-5 task is queued; then propagated onto `phase-v1-federation-inbound-b` (conflict-free trunk-merge in the lane worktree) so the worker — which branches from the PHASE branch tip `4a60667c9` (Task 4 complete + §15-green) — sees it. `/precheck` re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC before queue. Cohort B dispatched SERIALLY (Task 5 → §15-green → Task 6 → §15-green → Task 7) per the user's OOM-mitigation choice (AskUserQuestion 2026-05-19 "Cap cohort ≤2 serial"; rust-analyzer×3 ≈6.5 GB OOM-swap-killed the 15 GB EliteDesk in Cohort A — evals 424/425). Handover from Task 4 is DEGRADED (no HANDOVER: trailer on the Task-4 chain) — synthesized in §3a from the Task-4 public-API deliverable; the §0 grep self-verify is the authoritative existence check._
