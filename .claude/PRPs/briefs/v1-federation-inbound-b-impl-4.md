---
phase: v1-federation-inbound-b
role: impl-task
task: 4
brief_n: 1
authored: 2026-05-19
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
plan_task: "§13 Task 4 — wrapper + 5 check helpers + GovernanceInboundActivity trait + storage-cap + label handler + Phase-6 receive_remote_* mods"
parent_phase_tip: f59b54ac6 (phase-v1-federation-inbound-b @ Cohort A complete: Tasks 1+2+3 merged + §15 pass, DQ #280/#281/#282 resolved)
cohort: "barrier (non-[P]) — dispatched ALONE; Cohort B (Tasks 5-7 [P]) requires: task 4"
related_dq: "276 (proceed-as-one), 279 (serial-phase), 273/274/275 (option-a binding), 235 (no .claude/** writes by Junior)"
---

# [role:impl-task] v1-federation-inbound-b Task 4 — wrapper + helpers + trait + label handler — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-4.md

> **Clarify provenance:** parent **planning** brief clarified via `/brehon-clarify` (DQ #273/#274/#275 resolved advisor-mode — all **option-a**, BINDING; DQ #276 split-or-proceed user-resolved **proceed-as-one**). PRECON-3 (BINDING) = §15 is `validate-pending-laptop` shape (Shape G SUSPENDED until 2026-06-01 per DQ #229). Impl-task brief; no clarify-DQ gates it directly.

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is `phase-v1-federation-inbound-b` (you are on a Junior worktree branched off it). If `git branch --show-current` is anything else → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check (per `.claude/agents/impl-task.md` task-0 discipline): `date -u +"%a %H:%M UTC"` — if inside a forbidden window (Daily 02:55–04:15 / Sun 01:55–02:35 / Sun 03:55–04:30 / Wed 03:55–04:15 UTC) exit non-zero with `FORBIDDEN_WINDOW: <window>`. Shape G SUSPENDED (cargo on laptop); keep the self-check.
- **`requires:` precondition self-verify:** Task 4 `requires: task 1` (LemmyErrorType variants) + `task 3` (ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED const). Both merged on the phase tip you branch from (`f59b54ac6`, Cohort A complete). Confirm: `grep -q 'FederationPeerBlocklisted' crates/utils/src/error.rs && grep -q 'ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED' crates/db_schema/src/source/governance/governance_log.rs && echo PRECON-OK` — if either missing → STOP, file `kind: "blocker"` DQ (phase tip drifted / not branched off f59b54ac6+).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b Task 4 — wrap_governance_inbound + 5 check helpers + log_inbox_drop + GovernanceInboundActivity trait + storage-cap + receive_remote_moderation_label`

The actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-b Task 4 — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-4.md
```

## §2 Scope

**Produce** (one commit) — **single file, `crates/apub/activities/src/governance/inbox.rs`** (plan §13 Task 4 FILES YAML: `modifies: [crates/apub/activities/src/governance/inbox.rs]`):

Per plan **§10.4 verbatim** (the contract — copy it exactly):

1. **APPEND at file tail** (after `insert_federation_attestation`, plan cites line ~:325 — verify by grep):
   - `pub(crate) trait GovernanceInboundActivity` — 4 methods + a default `check_per_actor_rate_limit` (per §10.4 + PRD §9.2).
   - `pub(crate) async fn wrap_governance_inbound` — the wrapper, 7-step body per PRD §9.2 verbatim.
   - 5 check helpers: the per-step gate fns (peer-trust / size / schema / per-peer-rate / replay) per §10.4.
   - `fn log_inbox_drop` — writes 2 rows (drop-log + governance_log) inside a `run_transaction`.
   - `fn evict_oldest_unreviewed_if_needed` — storage-cap eviction helper.
   - `fn rate_per_peer_counts`, `fn rate_per_actor_counts`, `fn current_hour_bucket` — rate-limit helpers.
   - 2 rate-limit static cells: `OnceLock<Mutex<HashMap<(String, i64), u32>>>` (one per-peer, one per-actor) per §10.4 + DQ #277.
   - `pub async fn receive_remote_moderation_label` — the label handler, PRD §9.4 body verbatim EXCEPT the resolved `peer_trust_level_at_receipt` fill-in per **DQ #273 option-a** (RE-QUERY `federation_inbox_check_peer_trust(peer_domain, conn)`, do NOT thread it from the wrapper).
2. **MODIFY 2 existing fns** (surgical — eviction call + best-effort emit ONLY, do NOT rewrite the fn bodies):
   - `receive_remote_sanction_notice` (plan cites `:105-179` — verify by grep): insert `evict_oldest_unreviewed_if_needed(...)` at fn head; wrap the existing `conn.run_transaction(...)` call in a best-effort `ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED` emit on rollback (per §10.4).
   - `receive_remote_trust_attestation` (plan cites `:195-250` — verify by grep): same 2 surgical changes.

**Do NOT** in this task:

- Touch `error.rs` (Task 1 — merged), the 3 protocol structs (Task 2 — merged), `governance_log.rs` / registry (Task 3 — merged), any `publish_*.rs` (Cohort B Tasks 5-7 — they `requires: task 4`), `scheduled_tasks.rs` (Task 8), `e2e.rs` (Task 9).
- Modify the wrapper home: it goes IN `crates/apub/activities/src/governance/inbox.rs` (the `lemmy_apub_activities` crate). **NOT** `crates/apub/apub/src/governance/inbox.rs` (the PRD §5.2/§9.1 literal path is a documented descriptor error / non-existent — PRECON-2 BINDING; see `inbox.rs:1-12` module doc-comment + DQ-6.6-inbound id 37 precedent). If you find yourself creating `crates/apub/apub/src/governance/inbox.rs` → STOP, file `kind: "blocker"`.
- Rewrite the existing `receive_remote_sanction_notice` / `receive_remote_trust_attestation` bodies wholesale. The changes are SURGICAL: (1) one eviction call at fn head, (2) wrap the existing `run_transaction` in a best-effort persist_failed emit. Preserve Phase-6's `run_transaction` shape + the `:166` mid-tx pool reborrow pattern verbatim (`governance_log::append(&mut (&mut *conn).into(), ...)`).

**Commit message** (exactly): `feat(v1-federation-inbound-b): wrap_governance_inbound + 5 check helpers + log_inbox_drop + GovernanceInboundActivity trait + storage-cap + receive_remote_moderation_label (task 4)`

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #276 (proceed-as-one), DQ #279 (serial-phase), **DQ #273 option-a** (label handler RE-QUERIES peer trust — do NOT thread from wrapper), **DQ #274 option-a** (all `federation.inbound.*` config reads route through `lemmy_api::governance::config::get_int` vs `Scope::Instance`), **DQ #275 option-a** (storage-cap eviction at INSERT time inside each `receive_remote_*`, NOT a 6th pre-`inner` reject-gate). These 3 option-a resolutions are BINDING and shape the wrapper/label-handler structure.
2. **Plan §10.4** — the authoritative wrapper + trait + 5 helpers + label-handler + storage-cap code blocks (copy verbatim; this is the contract). Note the `:325` / `:105-179` / `:195-250` line cites are plan-time; verify with grep.
3. **Plan §13 "Task 4"** — step list + the pre-task enumeration (`rg "fn receive" crates/apub/activities/src/governance/publish_*.rs` EXPECT 3) + all 7 GOTCHAs.
4. **PRD §9.2** (wrapper signature + 7-step body) + **PRD §9.4** (label handler body) — §10.4 cites these verbatim; read them for the exact body shape.
5. **MIRROR refs** — Phase-6 `receive_remote_sanction_notice` `run_transaction` shape at `inbox.rs:157-176` (the surgical-modify target — preserve its shape) + the `:166` mid-tx pool reborrow + the `inbox.rs:1-12` module doc-comment (PRECON-2 path correction). **`grep -n 'pub async fn receive_remote_sanction_notice'`, `grep -n 'run_transaction'`, `grep -n 'insert_federation_attestation'` in inbox.rs** for the real positions (line cites may have drifted).
6. **Lessons** (MANDATORY per `.claude/rules/advisor-orchestrator.md` §2.4 — `inbox.rs` is a handler doing 2+ DB writes; Task 4 runs `cargo-clippy.bat ... -D warnings` + `cargo-test --test e2e --no-run`):
   - `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — **Why:** `log_inbox_drop` writes 2 rows (drop-log + governance_log); `receive_remote_moderation_label` writes 2 rows; both REQUIRE `conn.run_transaction(...)`. The best-effort `_persist_failed` emit is deliberately OUTSIDE the tx (on rollback). This lesson is the contract for the transaction boundaries.
   - `.claude/lessons/feedback_pg_advisory_xact_lock_void_decode.md` — **Why:** if §10.4's eviction/replay helpers call any void-returning PG function, it needs `.execute()` not `.load()`. Apply if §10.4 uses one.
   - `.claude/lessons/feedback_clippy_test_style.md` + `.claude/lessons/feedback_clippy_rerun_after_fix.md` — **Why:** the workspace clippy config denies `unwrap`/`expect`/`#[allow]`; the wrapper + 5 helpers + trait are substantial new code that MUST be clippy-clean under `-D warnings`. Re-run clippy after any fix (stale pass ≠ evidence).
   - `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Why:** the wrapper returns `LemmyResult<()>` / `LemmyErrorType`; helper error propagation must follow the canonical LemmyError shape (no `Box<dyn Error>` bridge). Mirror the sibling `receive_remote_*` fns' existing error shape verbatim.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — capture-then-tail for the §5 commands).

## §3a Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1
    commit: (merged into phase tip f59b54ac6 via daemon finalize 9358d0df0)
    filesModified: [crates/utils/src/error.rs]
    keyDecisions: ["6 LemmyErrorType variants FederationPeerBlocklisted/FederationPayloadTooLarge/FederationSchemaInvalid/FederationPeerRateLimitExceeded/FederationActorRateLimitExceeded/FederationActivityReplayed + 6 status_code() arms 403/413/400/429/429/409 per §10.1; enum #[non_exhaustive]"]
    notes: "Task 4 wrapper returns these variants — they exist on the phase tip. Verified by §0 precon self-check."
  - task: 2
    commit: (merged into phase tip f59b54ac6)
    filesModified: [crates/apub/objects/src/protocol/governance/{sanction_notice,trust_attestation,moderation_label}.rs]
    keyDecisions: ["#[serde(deny_unknown_fields)] on all 3 outer *Protocol structs per §10.2 — schema strictness now enforced at serde layer"]
    notes: "Task 4's schema-check step relies on this serde-layer strictness (PRD §9.2 step 3); the wrapper does NOT re-implement field validation."
  - task: 3
    commit: (merged into phase tip f59b54ac6)
    filesModified: [crates/db_schema/src/source/governance/governance_log.rs, crates/api/api/src/governance/governance_log.rs]
    keyDecisions: ["ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED const + api shim re-export per §10.3; registry advisor-reconciled"]
    notes: "Task 4's best-effort emit on receive_remote_* tx-rollback uses this const — it exists on the phase tip. Verified by §0 precon self-check."
```

## §4 Constraints (hard rules)

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-federation-inbound-b` (tip `f59b54ac6`+, Cohort A complete). Finalize merges your worktree branch back; do NOT push to `phase-v1-federation-inbound-b` directly.
- One commit (single file). If clippy/e2e-norun fails on first attempt, amend/fixup — do NOT split.
- Mid-task DQ visibility: if you raise a `pending` entry, **commit + push immediately** to your worktree branch per `.claude/rules/decision-queue.md` "Mid-task visibility".
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235 — interim escalation-and-transcribe)

If you need to write a DQ entry to `.claude/decision-queue.json` and the Claude Code sensitive-file gate blocks it (the same gate that blocked planning #330 + bm-cut #331 + Task-3 #335): (a) write the intended DQ-entry JSON object to a worktree-root file `TASK4_BLOCKER_DQ.json` (or `TASK4_VALIDATE_PENDING.json` for §5), (b) write a short `TASK4_ESCALATION.md` naming the issue, (c) commit both at worktree root + push, (d) STOP. The advisor transcribes per `.claude/rules/escalation.md`. Task 4 has NO `.claude/` deliverable — the happy path's only `.claude/` write is the §5 `validate-pending-laptop` entry; if THAT is gated, use this path with `TASK4_VALIDATE_PENDING.json` (likely, given #335's experience — be ready for it).

### Task-4 GOTCHAs (from plan §13 Task 4 — all load-bearing)

- **PRECON-2 (wrapper home):** `crates/apub/activities/src/governance/inbox.rs` ONLY. NEVER `crates/apub/apub/src/governance/inbox.rs`. (Watchpoint #1.)
- **Multi-write atomicity:** `log_inbox_drop` (2 rows) + `receive_remote_moderation_label` (2 rows) MUST use `conn.run_transaction(...)`. The best-effort `_persist_failed` emit is OUTSIDE the tx (fires only on rollback). Per `feedback_multi_write_handlers_need_transactions.md`.
- **DQ #273 option-a:** `receive_remote_moderation_label` RE-QUERIES `federation_inbox_check_peer_trust(peer_domain, conn)` for `peer_trust_level_at_receipt` — do NOT thread it from the wrapper.
- **DQ #274 option-a:** ALL `federation.inbound.*` config reads route through `lemmy_api::governance::config::get_int` against `Scope::Instance`. No direct env/const reads for these knobs.
- **DQ #275 option-a:** storage-cap eviction happens at INSERT time INSIDE each `receive_remote_*` (via `evict_oldest_unreviewed_if_needed`), NOT as a 6th pre-`inner` reject-gate in the wrapper.
- **Rate-limit statics:** 2 `OnceLock<Mutex<HashMap<(String, i64), u32>>>` cells (per-peer, per-actor), co-located in `inbox.rs` per DQ #277. Not a new dep, not a new crate.
- **Replay via UNIQUE:** detect duplicate `(peer_instance, activity_id)` via `DatabaseError(UniqueViolation, _)` specifically; other DB errors propagate (do NOT swallow).
- **Mid-tx pool reborrow:** preserve Phase-6's `governance_log::append(&mut (&mut *conn).into(), ...)` reborrow pattern (plan cites `:166`) verbatim in the surgical modifications.

### Plan-cited line numbers may have drifted

§10.4 is the contract for *content*. The `:325` (append point), `:105-179` / `:195-250` (the 2 modify targets), `:157-176` / `:166` (run_transaction + reborrow MIRROR) line cites are plan-time. `grep -n` the named symbols (`insert_federation_attestation`, `receive_remote_sanction_notice`, `receive_remote_trust_attestation`, `run_transaction`) for real positions. If the Phase-6 receive fns no longer have the cited `run_transaction` shape, file a DQ pending entry — the plan may have drifted.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

**Shape-G suspended until 2026-06-01** (DQ #229/#276). After committing + pushing your worktree branch, write a `kind: "validate-pending-laptop"` DQ entry (per `.claude/agents/impl-task.md` "Pre-Shape-G plans" + `.claude/rules/decision-queue.md` `kind: "validate-pending-laptop"`) with `from: "impl"`, `phase_task: 4`, `branch: <your-worktree-branch>`, and `commands[]` set **verbatim** to:

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task4-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task4-clippy.log 2>&1"
cmd //c "scripts\brehon\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/fed-in-b-task4-test-norun.log 2>&1"
```

(The `cargo-test --test e2e --no-run` is the §16a Story 2 R7 checkpoint — it compiles the e2e harness against the new wrapper without running it; this is REQUIRED for Task 4 specifically.)

Do NOT write `kind: "validate-pending"` and do NOT capture a `workflow_run_id` — there is no GitHub Actions run (workflows disabled until 2026-06-01). The advisor laptop session reads this entry, runs the 3 commands locally (serialized — one `target/`), and mutates the entry (`result: pass|fail`).

If the `.claude/decision-queue.json` write is gated by the sensitive-file gate, use the §4 harness-gap escalation path with `TASK4_VALIDATE_PENDING.json` at worktree root. (Per #335's experience this gate is likely to fire — author the escalation files cleanly if so; it is NOT a blocker.)

Per `feedback_pipes_mask_exit_codes.md`: you do not run these commands yourself (the laptop advisor does) — your job is only to apply the §10.4 implementation + raise the entry with the commands verbatim.

## §6 Expected output (return to advisor)

```
## Task 4 complete — v1-federation-inbound-b wrapper + helpers + trait + label handler

**Commit:** <sha> on <worktree-branch>
**File changed:**
  - crates/apub/activities/src/governance/inbox.rs (+GovernanceInboundActivity trait, +wrap_governance_inbound, +5 check helpers, +log_inbox_drop, +evict_oldest_unreviewed_if_needed, +rate_per_peer/actor_counts, +current_hour_bucket, +2 OnceLock rate cells, +receive_remote_moderation_label; MODIFIED receive_remote_sanction_notice + receive_remote_trust_attestation surgically)
**PRECON self-check:** Task 1 variants + Task 3 const present on phase tip (confirmed)
**PRECON-2:** wrapper in lemmy_apub_activities/governance/inbox.rs (NOT apub/apub) — confirmed
**Phase-6 preservation:** receive_remote_* run_transaction shape + :166 reborrow preserved (surgical mods only)
**option-a compliance:** DQ #273 (label re-queries trust) / #274 (config via get_int Scope::Instance) / #275 (eviction at INSERT) — all honoured
**validate-pending-laptop DQ:** #<id> raised (commands: cargo-check.bat + cargo-clippy.bat -D warnings + cargo-test --test e2e --no-run)
**Next:** advisor laptop runs §15 (3 cmds), mutates DQ #<id>; Cohort B (Tasks 5-7 [P]) requires task 4 — gated until this validates pass
```

Plus any DQ #N references if you raised a blocker mid-task (NOT for a gated registry write — Task 4 has no registry deliverable; only a gated §5 DQ write is an escalation case).

## §7 Why this brief differs from the plan

It does not — Task 4's scope is exactly plan §10.4 + §13 Task 4. This brief adds only: (a) §0 forbidden-window self-check + the `requires: task 1/3` precondition self-verify (both merged on f59b54ac6 — Cohort A complete), (b) §4 harness-gap escalation path (likely to fire for the §5 DQ write per #335's experience) + the explicit 7 GOTCHAs from §13 Task 4 + the 3 option-a binding resolutions, (c) §5 explicit `validate-pending-laptop` shape per PRECON-3 INCLUDING the §16a Story 2 R7 `cargo-test --test e2e --no-run` checkpoint, (d) explicit "plan line numbers may have drifted — grep the named symbols". The wrapper + trait + 5 helpers + label handler + storage-cap + surgical Phase-6 mods are §10.4 verbatim — do not deviate. Mirrors the canonical sibling `.claude/PRPs/briefs/federation-inbound-a-impl-4.md` schema per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate (note: fed-in-a Task 4 was 9 consts; fed-in-b Task 4 is the enforcement core — same brief STRUCTURE, §10.4 content).

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; per DQ #279 the canonical session drives fed-in-b impl under serial-phase policy). Brief committed on `governance-v0` before the Junior Task-4 task is queued; then propagated onto `phase-v1-federation-inbound-b` (conflict-free trunk-merge in the lane worktree) so the worker — which branches from the PHASE branch tip f59b54ac6 (Cohort A complete) — sees it. `/precheck` re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC before queue. SINGLE non-[P] barrier — no cohort memory fan-out (the host swap-death root cause from Cohort A does not recur for a single task; the serena/rust-analyzer budget fix applies at Cohort B Tasks 5-7)._
