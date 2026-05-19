---
phase: v1-federation-inbound-b
role: impl-task
task: 6
brief_n: 1
authored: 2026-05-19
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
plan_task: "§13 Task 6 [P] — publish_trust_attestation.rs: wrap Activity::receive + impl GovernanceInboundActivity WITH per-actor rate-limit override"
parent_phase_tip: "Task 5 worktree merged + §15-green (Cohort B serial: Task 6 dispatched after Task 5 validates pass; phase tip = Task-5-merge sha, derive at dispatch time — NOT 4a60667c9)"
cohort: "Cohort B (Tasks 5-7 [P]) — user chose cap-≤2 serial (AskUserQuestion 2026-05-19 'Cap cohort ≤2 serial'); Task 6 dispatched SECOND, ALONE, after Task 5 §15-green, before Task 7. requires: task 4."
related_dq: "276 (proceed-as-one), 279 (serial-phase), 283 (Task-4 validate pass — requires:4 precondition), 235 (no .claude/** writes by Junior)"
---

# [role:impl-task] v1-federation-inbound-b Task 6 — publish_trust_attestation wrap receive + impl GovernanceInboundActivity with per-actor rate gate — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-6.md

> **Clarify provenance:** parent **planning** brief clarified via `/brehon-clarify` (DQ #273/#274/#275 resolved advisor-mode — all **option-a**, BINDING; DQ #276 split-or-proceed user-resolved **proceed-as-one**). PRECON-3 (BINDING) = §15 is `validate-pending-laptop` shape (Shape G SUSPENDED until 2026-06-01 per DQ #229). Impl-task brief; no clarify-DQ gates it directly. Cohort B serial-dispatch (5→6→7, one at a time) is the user's OOM-mitigation choice (AskUserQuestion 2026-05-19; rust-analyzer×3 ≈6.5 GB OOM-swap-killed the 15 GB EliteDesk in Cohort A — evals 424/425).

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is `phase-v1-federation-inbound-b` (you are on a Junior worktree branched off it). If `git branch --show-current` is anything else → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check (per `.claude/agents/impl-task.md` task-0 discipline): `date -u +"%a %H:%M UTC"` — if inside a forbidden window (Daily 02:55–04:15 / Sun 01:55–02:35 / Sun 03:55–04:30 / Wed 03:55–04:15 UTC) exit non-zero with `FORBIDDEN_WINDOW: <window>`. Shape G SUSPENDED (cargo on laptop); keep the self-check.
- **`requires: task 4` precondition self-verify:** Task 6 calls `wrap_governance_inbound` + impl's `GovernanceInboundActivity` + uses the `pub(crate)` helpers `rate_per_actor_counts()` / `current_hour_bucket()` / `log_inbox_drop` — ALL Task-4 deliverables, merged on the phase tip you branch from. Confirm: `grep -q 'pub(crate) async fn wrap_governance_inbound' crates/apub/activities/src/governance/inbox.rs && grep -q 'pub(crate) trait GovernanceInboundActivity' crates/apub/activities/src/governance/inbox.rs && grep -qE 'fn rate_per_actor_counts|fn current_hour_bucket' crates/apub/activities/src/governance/inbox.rs && grep -q 'fn log_inbox_drop' crates/apub/activities/src/governance/inbox.rs && echo PRECON-OK` — if any missing → STOP, file `kind: "blocker"` DQ (phase tip drifted / not branched off the Task-5-merge tip).
- **Task-5 sibling check** (Cohort B serial — Task 5 merged first): `git log phase-v1-federation-inbound-b --oneline --grep '(task 5)' -i | head -1` should be non-empty. Read the merged `publish_sanction_notice.rs` `impl GovernanceInboundActivity` block as the canonical sibling shape to mirror (Task 6 = same structure + a `check_per_actor_rate_limit` override). If empty → the serial order broke; file `kind: "blocker"` DQ (advisor dispatched Task 6 before Task 5 §15-green).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b Task 6 — publish_trust_attestation.rs: replace Activity::receive body with wrap_governance_inbound call + add impl GovernanceInboundActivity for PublishTrustAttestation WITH check_per_actor_rate_limit override (PRD §7.2 per-subject rate gate)`

The actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-b Task 6 — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-6.md
```

## §2 Scope

**Produce** (one commit) — **single file, `crates/apub/activities/src/governance/publish_trust_attestation.rs`** (plan §13 Task 6 FILES YAML: `modifies: [crates/apub/activities/src/governance/publish_trust_attestation.rs]`, `creates: []`):

Per plan **§10.5 + §10.4 verbatim** (the contract — copy exactly; §10.5 for the receive-wrap + trait-impl shell, §10.4 rate-limit cell pattern for the per-actor override body):

1. **REPLACE the existing `Activity::receive` body** (plan cites `:86-93` — verify by grep `fn receive` in this file) with a one-line `wrap_governance_inbound` call (delegating to `crate::governance::inbox::receive_remote_trust_attestation`).
2. **ADD `impl crate::governance::inbox::GovernanceInboundActivity for PublishTrustAttestation`** block with:
   - `activity_id`, `actor_domain`, `payload_size_bytes` (= `Ok(serde_json::to_vec(self)?.len())`), `payload_size_cap_key` (= `"federation.inbound.max_payload_bytes_trust_attestation"`) per §10.5.
   - **A `check_per_actor_rate_limit` OVERRIDE** (this is what makes Task 6 distinct from Task 5/7) — full body ~50 lines mirroring §10.4 wrapper step-4 (the per-peer-rate cell structure), adapted for per-ACTOR keyed by hashed subject URL. Per the plan §13 Task 6 IMPLEMENT block + PRD §7.2:
     - Read subject URL from `self.object.rest.get("subject")`.
     - Hash it (same hashing the §10.4 rate cell uses for the per-peer key — mirror that helper, do not invent a new hash).
     - Read `federation.inbound.per_actor_attestation_rate_per_hour` from `governance_config` via `lemmy_api::governance::config::get_int` against `Scope::Instance` (per DQ #274 option-a — the same config-read path Task 4's wrapper uses; do NOT direct-read env/const).
     - Increment the `crate::governance::inbox::rate_per_actor_counts()` cell entry at `(subject_hash, current_hour_bucket())`.
     - If the count `>` cap: call `crate::governance::inbox::log_inbox_drop(...)` with `ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_ACTOR`, then return `Err(LemmyErrorType::FederationActorRateLimitExceeded.into())`.
     - Else `Ok(())`.

**Do NOT** in this task:

- Touch `inbox.rs` (Task 4 — merged; you CALL `wrap_governance_inbound` + the `rate_per_actor_counts`/`current_hour_bucket`/`log_inbox_drop` helpers + impl the trait, you do NOT edit it), `publish_sanction_notice.rs` (Task 5 — merged separately), `publish_label.rs` (Task 7 — separate Cohort B task), `error.rs`/protocol/`governance_log.rs` (Tasks 1-3 merged), `scheduled_tasks.rs` (Task 8), `e2e.rs` (Task 9).
- Re-implement the rate-limit cell or `current_hour_bucket` — they are Task-4 `pub(crate)` deliverables; CALL them. Your override only does the per-actor key derivation + cap comparison + the drop-log-on-exceed, mirroring §10.4 wrapper step-4's per-peer structure.
- Invent a new hashing scheme or a new config-read path. Mirror §10.4's existing hash + use the DQ #274 option-a `get_int`/`Scope::Instance` path (the same one the merged wrapper uses for `federation.inbound.*`).

**Commit message** (exactly): `feat(v1-federation-inbound-b): publish_trust_attestation — wrap receive + impl GovernanceInboundActivity with per-actor rate gate (task 6)`

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #283 (Task 4 `validate-pending-laptop` resolved **pass** — the `requires: task 4` precondition), DQ #274 option-a (ALL `federation.inbound.*` config reads route through `lemmy_api::governance::config::get_int` vs `Scope::Instance` — your `per_actor_attestation_rate_per_hour` read MUST follow this), DQ #276 (proceed-as-one), DQ #279 (serial-phase).
2. **Plan §10.5** (receive-wrap + trait-impl shell) + **Plan §10.4** (the per-peer rate-limit cell structure your `check_per_actor_rate_limit` override mirrors for per-actor) — copy verbatim; these are the contract. The `:86-93` line cite is plan-time; verify with grep.
3. **Plan §13 "Task 6"** — step list + FILES YAML + the IMPLEMENT block (the ~50-line override skeleton with the inline `// ...` comments naming each step) + the `requires: task 4` reason.
4. **Task 4's merged `inbox.rs`** (READ-ONLY): `grep -n 'pub(crate) trait GovernanceInboundActivity' -A 12 crates/apub/activities/src/governance/inbox.rs` (trait method signatures — mirror EXACTLY) + `grep -n 'fn rate_per_actor_counts\|fn current_hour_bucket\|fn log_inbox_drop\|fn rate_per_peer_counts' -A 8 inbox.rs` (the helper signatures you call + the per-peer rate cell structure to mirror for per-actor) + `grep -n 'pub(crate) async fn wrap_governance_inbound' -A 4 inbox.rs` (wrapper sig you call). The per-peer rate-limit step inside `wrap_governance_inbound` (or its helper) is the EXACT structure your per-actor override mirrors — read it and adapt key=subject_hash instead of peer_domain.
5. **Task 5's merged `publish_sanction_notice.rs`** (READ-ONLY, canonical sibling — Cohort B serial means Task 5 merged before you): `grep -n 'impl.*GovernanceInboundActivity' -A 20 crates/apub/activities/src/governance/publish_sanction_notice.rs` — mirror its `impl` block STRUCTURE (Task 6 = same 4 metadata methods + ADD the `check_per_actor_rate_limit` override; Task 5 used the trait default). This is the canonical-schema-first source for the impl-block shape.
6. **MIRROR refs** — the existing Phase-6 `Activity::receive` body in `publish_trust_attestation.rs` (`grep -n 'fn receive' -A 10` for real position) + `self.object.rest.get("subject")` access shape (grep the struct def for how `rest` is typed — likely `serde_json::Map` or similar; the subject extraction must handle the `Option`/`Value` correctly without `unwrap`).
7. **Lessons** (MANDATORY per `.claude/rules/advisor-orchestrator.md` §2.4 — Task 6 runs `cargo-clippy.bat ... -D warnings`; the override is substantial new code with a config read + a DB write via `log_inbox_drop`):
   - `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — **Why:** `log_inbox_drop` (called by your override on rate-exceed) writes 2 rows in a `run_transaction` — that transaction boundary is INSIDE the Task-4 helper you call; do NOT re-wrap it. Your override calls the helper; it does not open its own tx.
   - `.claude/lessons/feedback_clippy_test_style.md` + `.claude/lessons/feedback_clippy_rerun_after_fix.md` — **Why:** the ~50-line override + trait-impl is substantial new code; workspace clippy denies `unwrap`/`expect`/`#[allow]` under `-D warnings`. The subject extraction (`Option` handling), the count comparison, the config read all must be clippy-clean. Re-run clippy after any fix.
   - `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Why:** `check_per_actor_rate_limit` returns `LemmyResult<()>`; the rate-exceed path returns `Err(LemmyErrorType::FederationActorRateLimitExceeded.into())`. Follow the canonical LemmyError shape (no `Box<dyn Error>` bridge); mirror the merged wrapper's per-peer-exceed error shape verbatim.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — capture-then-tail for the §5 commands; you do not run them yourself).

## §3a Handover from prior cohort

> **Handover state: DEGRADED** (per `.claude/rules/advisor-orchestrator.md` §4.3 — the Task-4 chain commits carry no `HANDOVER:` YAML trailer; Task-5's commit handover is read directly from its merged source per §3 reading #5, not a trailer). Synthesized by the advisor from the Task-4 public-API deliverable + the Task-5 canonical-sibling fact. The §0 precon self-verify + Task-5-sibling check grep the actual symbols on the phase tip as the authoritative checks.

```yaml
prior_cohort_tasks:
  - task: 4
    commit: 4a60667c9 (Task 4 enforcement core; §15-green per DQ #283 resolved pass 2026-05-19T20:44:53Z)
    filesModified: [crates/apub/activities/src/governance/inbox.rs]
    keyDecisions:
      - "pub(crate) trait GovernanceInboundActivity (activity_id/actor_domain/payload_size_bytes/payload_size_cap_key + DEFAULT check_per_actor_rate_limit). Task 6 impl's it AND OVERRIDES check_per_actor_rate_limit (per-subject rate gate, PRD §7.2)."
      - "pub(crate) helpers rate_per_actor_counts() (OnceLock<Mutex<HashMap<(String,i64),u32>>>), current_hour_bucket(), log_inbox_drop — Task 6's override calls all 3. The per-PEER rate-limit step inside wrap_governance_inbound is the structural template Task 6's per-ACTOR override mirrors (key=subject_hash not peer_domain)."
      - "DQ #274 option-a (BINDING): all federation.inbound.* config reads go through lemmy_api::governance::config::get_int vs Scope::Instance — Task 6's per_actor_attestation_rate_per_hour read MUST use this same path (the merged wrapper already does for its knobs; copy that call shape)."
      - "pub async fn receive_remote_trust_attestation — Phase-6 handler, surgically modified by Task 4 (signature unchanged); wrap_governance_inbound delegates to it. Task 6 does NOT touch it."
    notes: "Task 6 CONSUMES Task 4's wrapper + trait + 3 pub(crate) rate/log helpers. The per-actor override is the ONLY behavioural addition vs Task 5 (which used the trait default)."
  - task: 5
    commit: "(Task-5-merge sha — derive at dispatch via `git log phase-v1-federation-inbound-b --grep '(task 5)' -i`; Cohort B serial = Task 5 merged + §15-green BEFORE Task 6 dispatched)"
    filesModified: [crates/apub/activities/src/governance/publish_sanction_notice.rs]
    keyDecisions:
      - "Task 5 established the canonical Cohort-B impl-block shape: Activity::receive → one-line wrap_governance_inbound delegation + impl GovernanceInboundActivity with the 4 metadata methods, trait-default check_per_actor_rate_limit. Task 6 mirrors this STRUCTURE and ADDS the check_per_actor_rate_limit override."
    notes: "Read Task 5's merged publish_sanction_notice.rs impl block (§3 reading #5) as the canonical-schema-first sibling. Task 6 = Task-5 shape + per-actor override. If Task 5 not yet merged → serial order broke (§0 Task-5-sibling check files a blocker)."
```

## §4 Constraints (hard rules)

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-federation-inbound-b` (tip = Task-5-merge sha, Task 4 + Task 5 complete + §15-green). Finalize merges your worktree branch back; do NOT push to `phase-v1-federation-inbound-b` directly.
- One commit (single file). If clippy fails on first attempt, amend/fixup — do NOT split.
- Mid-task DQ visibility: if you raise a `pending` entry, **commit + push immediately** to your worktree branch per `.claude/rules/decision-queue.md` "Mid-task visibility".
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235 — interim escalation-and-transcribe)

If you need to write a DQ entry to `.claude/decision-queue.json` and the Claude Code sensitive-file gate blocks it (the same gate that blocked planning #330 + bm-cut #331 + Task-3 #335 + Task-4 #336): (a) write the intended DQ-entry JSON object to a worktree-root file `TASK6_BLOCKER_DQ.json` (or `TASK6_VALIDATE_PENDING.json` for §5), (b) write a short `TASK6_ESCALATION.md` naming the issue, (c) commit both at worktree root + push, (d) STOP. The advisor transcribes per `.claude/rules/escalation.md`. Task 6 has NO `.claude/` deliverable — the happy path's only `.claude/` write is the §5 `validate-pending-laptop` entry; if THAT is gated, use this path with `TASK6_VALIDATE_PENDING.json` (likely, given #335/#336's experience — be ready for it).

### Task-6 GOTCHAs (from plan §13 Task 6 + §10.4/§10.5 — all load-bearing)

- **Single-file, single-purpose:** ONLY `publish_trust_attestation.rs`. The wrapper + trait + rate/log helpers are Task-4 deliverables on the phase tip — CALL/impl them, never edit `inbox.rs`. If you find yourself editing `inbox.rs` → STOP, file `kind: "blocker"`.
- **The per-actor override is the ONLY thing distinct from Task 5/7.** Tasks 5 + 7 use the trait-default `check_per_actor_rate_limit`. Task 6 OVERRIDES it. Get the 4 metadata methods identical-in-shape to Task 5's merged sibling; the override is the delta.
- **Config read path is BINDING (DQ #274 option-a):** `federation.inbound.per_actor_attestation_rate_per_hour` is read via `lemmy_api::governance::config::get_int` against `Scope::Instance`. NOT a direct env/const read. The merged `wrap_governance_inbound` already reads `federation.inbound.*` knobs this way — copy that exact call shape from inbox.rs.
- **Reuse Task-4's hash + rate cell — do NOT invent.** `rate_per_actor_counts()` is the Task-4 `pub(crate)` cell; `current_hour_bucket()` is the Task-4 bucket fn. The subject-URL hash MUST be the same hashing the §10.4 per-peer step uses (mirror it; a divergent hash silently splits the rate namespace). Read §10.4's per-peer step structure (§3 reading #4) and adapt key=subject_hash.
- **payload_size_cap_key string is exact:** `"federation.inbound.max_payload_bytes_trust_attestation"` (per §10.5). Verbatim from §10.5, not abbreviated.
- **subject extraction without unwrap:** `self.object.rest.get("subject")` returns an `Option<&Value>` (verify the `rest` field type by grep). Handle the `None`/wrong-type case with a `LemmyResult` error (mirror how the wrapper handles a missing required field), NOT `.unwrap()`/`.expect()` (clippy-denied).
- **log_inbox_drop owns its own tx:** the Task-4 `log_inbox_drop` helper opens its own `run_transaction` (2 rows). Your override CALLS it on rate-exceed; do NOT wrap it in another tx. Per `feedback_multi_write_handlers_need_transactions.md`.

### Plan-cited line numbers may have drifted

§10.4/§10.5 are the contract for *content*. The `:86-93` (receive body) line cite + the Task-5 impl-block position are runtime-derived. `grep -n 'fn receive'` + `grep -n 'impl Activity'` in `publish_trust_attestation.rs`; `grep -n 'impl.*GovernanceInboundActivity'` in the merged `publish_sanction_notice.rs`. If the Phase-6 receive body or Task-5's impl shape diverges from §10.5's documented structure, file a DQ pending entry — the plan may have drifted.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

**Shape-G suspended until 2026-06-01** (DQ #229/#276). After committing + pushing your worktree branch, write a `kind: "validate-pending-laptop"` DQ entry (per `.claude/agents/impl-task.md` "Pre-Shape-G plans" + `.claude/rules/decision-queue.md` `kind: "validate-pending-laptop"`) with `from: "impl"`, `phase_task: 6`, `branch: <your-worktree-branch>`, and `commands[]` set **verbatim** to (per plan §13 Task 6 "Push and exit" — exactly 2 commands, NO `cargo-test --no-run` for Task 6):

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task6-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task6-clippy.log 2>&1"
```

Do NOT write `kind: "validate-pending"` and do NOT capture a `workflow_run_id` — there is no GitHub Actions run (workflows disabled until 2026-06-01). The advisor laptop session reads this entry, runs the 2 commands locally (serialized — one `target/`), and mutates the entry (`result: pass|fail`).

If the `.claude/decision-queue.json` write is gated by the sensitive-file gate, use the §4 harness-gap escalation path with `TASK6_VALIDATE_PENDING.json` at worktree root. (Per #335/#336's experience this gate is likely to fire — author the escalation files cleanly if so; it is NOT a blocker.)

Per `feedback_pipes_mask_exit_codes.md`: you do not run these commands yourself (the laptop advisor does) — your job is only to apply the §10.4/§10.5 implementation + raise the entry with the commands verbatim.

## §6 Expected output (return to advisor)

```
## Task 6 complete — v1-federation-inbound-b publish_trust_attestation wrap + trait-impl + per-actor rate gate

**Commit:** <sha> on <worktree-branch>
**File changed:**
  - crates/apub/activities/src/governance/publish_trust_attestation.rs (Activity::receive → one-line wrap_governance_inbound delegation to receive_remote_trust_attestation; +impl GovernanceInboundActivity for PublishTrustAttestation [4 metadata methods + check_per_actor_rate_limit OVERRIDE: subject-hash keyed, get_int/Scope::Instance config read, rate_per_actor_counts cell, log_inbox_drop on exceed → FederationActorRateLimitExceeded])
**PRECON self-check:** wrap_governance_inbound + trait + rate_per_actor_counts/current_hour_bucket/log_inbox_drop present in inbox.rs (confirmed — requires: task 4)
**Task-5 sibling:** publish_sanction_notice impl block read + mirrored for the 4 metadata methods (canonical-schema-first)
**Override fidelity:** per-actor key = same hash as §10.4 per-peer step (not invented); config via DQ #274 option-a get_int/Scope::Instance; log_inbox_drop called (not re-tx-wrapped)
**payload_size_cap_key:** "federation.inbound.max_payload_bytes_trust_attestation" (verbatim §10.5)
**No inbox.rs edit:** confirmed
**validate-pending-laptop DQ:** #<id> raised (commands: cargo-check.bat + cargo-clippy.bat -D warnings — 2 cmds, no e2e-norun for Task 6)
**Next:** advisor laptop runs §15 (2 cmds), mutates DQ #<id>; on pass → Task 7 dispatched (Cohort B serial, last task)
```

Plus any DQ #N references if you raised a blocker mid-task (NOT for a gated §5 DQ write — that's an escalation case per §4).

## §7 Why this brief differs from the plan

It does not — Task 6's scope is exactly plan §10.5 (receive-wrap + trait shell) + §10.4 (rate-cell pattern for the override) + §13 Task 6. This brief adds only: (a) §0 forbidden-window self-check + the `requires: task 4` precondition self-verify + the **Task-5-sibling check** (Cohort B serial means Task 5 merged + §15-green first; if not, the serial order broke — blocker), (b) §4 harness-gap escalation path + the explicit GOTCHAs incl. the BINDING DQ #274 config-read path + the "reuse Task-4 hash, do not invent" rule (a divergent hash silently splits the rate namespace — the highest-risk Task-6-specific footgun), (c) §5 explicit `validate-pending-laptop` shape per PRECON-3 with the **2-command** set (NO `cargo-test --test e2e --no-run` — Task-4-specific R7 checkpoint; Task 6's plan "Push and exit" lists only check + clippy), (d) the trait-signature-fidelity + subject-extraction-without-unwrap GOTCHAs. The receive-wrap + trait-impl + per-actor override are §10.5/§10.4/§13-Task-6 verbatim — do not deviate. Mirrors the canonical sibling `.claude/PRPs/briefs/v1-federation-inbound-b-impl-4.md` schema (same §0-§7 structure) AND uses Task-5's merged `publish_sanction_notice.rs` as the canonical-schema-first impl-block source per `.claude/rules/advisor-orchestrator.md` §3.6.

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; per DQ #279 the canonical session drives fed-in-b impl under serial-phase policy). Brief committed on `governance-v0` before the Junior Task-6 task is queued (queued only AFTER Task 5 validates §15-pass — Cohort B serial per the user's cap-≤2 OOM-mitigation choice); then propagated onto `phase-v1-federation-inbound-b` so the worker — which branches from the PHASE branch tip (Task-5-merge sha) — sees it. `/precheck` re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC before queue. Handover from Task 4 is DEGRADED (no HANDOVER: trailer) — synthesized in §3a; Task-5 handover is read directly from its merged source (§3 reading #5)._
