---
phase: v1-federation-inbound-b
role: impl-task
task: 1
brief_n: 1
authored: 2026-05-19
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
plan_task: "§13 Task 1 — add 6 new LemmyErrorType variants + extend status_code() mapper"
parent_phase_tip: 753289320 (phase-v1-federation-inbound-b @ Task-0 audit pass, origin tip)
cohort: "Cohort A (Tasks 1-3, 3-way [P]) — dispatched in parallel"
related_dq: "276 (proceed-as-one, user-resolved), 279 (serial-phase policy)"
---

# [role:impl-task] v1-federation-inbound-b Task 1 — LemmyErrorType variants + HTTP mapper — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-1.md

> **Clarify provenance:** this brief's parent **planning** brief was clarified via `/brehon-clarify` before the planning task was queued (DQ #273/#274/#275 resolved advisor-mode; DQ #276 split-or-proceed user-resolved **proceed-as-one**). PRECON-3 (BINDING) = §15 is `validate-pending-laptop` shape (Shape G SUSPENDED until 2026-06-01 per DQ #229). This is an impl-task brief; no clarify-DQ gates it directly — clarify gates planning briefs only.

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is `phase-v1-federation-inbound-b` (you are on a Junior worktree branched off it). If `git branch --show-current` is anything else → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check (per `.claude/agents/impl-task.md` task-0 discipline): `date -u +"%a %H:%M UTC"` — if inside a forbidden window (Daily 02:55–04:15 / Sun 01:55–02:35 / Sun 03:55–04:30 / Wed 03:55–04:15 UTC) exit non-zero with `FORBIDDEN_WINDOW: <window>`. NOTE: Shape G is SUSPENDED (validate-pending-laptop mode per DQ #229/#276) — cargo runs on the LAPTOP not this worker, so the forbidden-window cargo concern is reduced; keep the self-check anyway.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b Task 1 — 6 LemmyErrorType federation-inbound variants + status_code() HTTP mapper`

The actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-b Task 1 — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-1.md
```

## §2 Scope

**Produce** (one commit):

- `crates/utils/src/error.rs` — per plan §10.1 verbatim:
  - **6 new `LemmyErrorType` variants** inserted adjacent to the existing Federation cluster (plan cites line ~:114-122 — verify by grep): `FederationPeerBlocklisted`, `FederationPayloadTooLarge`, `FederationSchemaInvalid`, `FederationPeerRateLimitExceeded`, `FederationActorRateLimitExceeded`, `FederationActivityReplayed`. All unit variants; the enum's `#[serde(rename_all = "snake_case")]` (line :9) already applies (wire form `federation_peer_blocklisted` etc — do NOT add per-variant serde attrs).
  - **6 new `status_code()` match arms** added to the `impl ResponseError for LemmyError` match (plan cites line :223-230 — verify by grep), returning the PRD §5.3 codes verbatim: `FederationPeerBlocklisted → FORBIDDEN (403)`, `FederationPayloadTooLarge → PAYLOAD_TOO_LARGE (413)`, `FederationSchemaInvalid → BAD_REQUEST (400)`, `FederationPeerRateLimitExceeded → TOO_MANY_REQUESTS (429)`, `FederationActorRateLimitExceeded → TOO_MANY_REQUESTS (429)`, `FederationActivityReplayed → CONFLICT (409)`. Place the 6 arms BEFORE the existing `_ => ...BAD_REQUEST` catch-all arm.

**Do NOT** in this task:

- Touch the 3 protocol structs (Task 2), `governance_log.rs` / the registry (Task 3), `inbox.rs` (Task 4), any `publish_*.rs` (Cohort B Tasks 5-7), `scheduled_tasks.rs` (Task 8), or `e2e.rs` (Task 9).
- Add any caller-update edits. The enum is `#[non_exhaustive]` (line :10) so variant adds do NOT require any caller to be updated — if you find yourself editing a `match` on `LemmyErrorType` anywhere outside `error.rs`, STOP (you are out of scope).

**Commit message** (exactly): `feat(v1-federation-inbound-b): LemmyErrorType — 6 new federation inbound variants + HTTP status mapper (task 1)`

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #276 (BINDING — proceed-as-one, user-resolved; this is the single-plan path, no -b-1/-b-2 split), DQ #279 (serial-phase policy — single active phase; this canonical session drives impl).
2. **Plan §10.1** — the authoritative variant block + status-code mapper block (copy verbatim; this is the contract). Note the `:114-122` / `:223-230` line cites are plan-time; verify with grep before inserting.
3. **Plan §13 "Task 1"** — the step list + the cfg-select GOTCHA + the R9 `#[non_exhaustive]` GOTCHA.
4. **MIRROR ref** — `crates/utils/src/error.rs` existing Federation-prefixed variants (`FederationDisabled` cited at `:143`) for the naming convention + the existing `status_code()` arms (`IncorrectLogin → UNAUTHORIZED`, `NotFound → NOT_FOUND`) for the arm shape. **`grep -n 'FederationDisabled'` + `grep -n 'fn status_code'`** to find the real positions (line numbers may have drifted since plan-write).
5. **Lessons** (per `.claude/rules/advisor-orchestrator.md` §2.4 — Task 1 runs `cargo-clippy.bat ... -D warnings` on new code):
   - `.claude/lessons/feedback_clippy_test_style.md` — **Why:** the Lemmy workspace clippy config denies `unwrap`/`expect`/`#[allow]` escape-hatches; the new variants + match arms must be clippy-clean under `-D warnings` (the §5 validation runs `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings`).
   - `.claude/lessons/feedback_clippy_rerun_after_fix.md` — **Why:** if a clippy fix is needed, clippy must be re-run; a stale pass is not evidence the fix worked.
   - `.claude/lessons/feedback_features_full_workspace_only.md` — **Why:** the `ResponseError` impl is inside a `cfg_select! { feature = "full" => { ... } }` block (plan §13 Task 1 cfg-select GOTCHA); `--features full` on a `--workspace` build is what exercises it. (The §5 commands already use `--workspace --features full` — do not change them.)
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — capture-then-tail rule for the §5 validation commands).

## §3a Handover from prior cohort

(none — Cohort A is the first impl cohort; Task 0 was a pure read-only probe with no commit and no `HANDOVER:` trailer.)

## §4 Constraints (hard rules)

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-federation-inbound-b`. Finalize merges your worktree branch back; do NOT push to `phase-v1-federation-inbound-b` directly.
- One commit. (Task 1 is single-file; if clippy fails on first attempt, amend/fixup — do NOT split into a second commit.)
- Mid-task DQ visibility: if you raise a `pending` entry, **commit + push immediately** to your worktree branch per `.claude/rules/decision-queue.md` "Mid-task visibility".
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235 — interim escalation-and-transcribe)

If you need to write a DQ entry to `.claude/decision-queue.json` and the Claude Code sensitive-file gate blocks it (the same gate that blocked planning #330 + bm-cut #331): (a) write the intended DQ-entry JSON object to a worktree-root file `TASK1_BLOCKER_DQ.json`, (b) write a short `TASK1_ESCALATION.md` naming the issue, (c) commit both at worktree root + push, (d) STOP. The advisor transcribes per `.claude/rules/escalation.md`. **This applies ONLY if you must raise a DQ — Task 1's happy path has zero `.claude/` writes EXCEPT the §5 `validate-pending-laptop` entry; if THAT write is gated, use the same escalation path with `TASK1_VALIDATE_PENDING.json`.**

### Task-1 GOTCHAs (from plan §13 Task 1 — load-bearing)

- **cfg-select:** the `impl ResponseError for LemmyError` block is inside `cfg_select! { feature = "full" => { ... } }` (plan cites line :162-350). Insert the 6 status-code arms INSIDE that block (where the existing `IncorrectLogin`/`NotFound` arms live), keeping the build conditional. If you can't locate the existing arms inside a `cfg_select!`/`#[cfg(feature = "full")]` gate, STOP and file `kind: "blocker"` (the plan's structural assumption drifted).
- **`#[non_exhaustive]` (R9):** the enum is `#[non_exhaustive]` (line :10). Variant adds are forward-compatible — NO caller changes required anywhere. Do not "helpfully" update downstream matches.
- **Arm ordering:** the 6 new arms MUST precede the existing `_ => ...BAD_REQUEST` wildcard, else they are unreachable and clippy/compile behaviour differs from PRD §5.3 intent.

### Plan-cited line numbers may have drifted

§10.1 is the contract for *content*. The `:114-122` / `:223-230` / `:143` / `:162-350` line cites are plan-time snapshots — `grep -n` the actual anchors (`FederationDisabled`, `fn status_code`, `cfg_select`) to find real positions before inserting. If the enum no longer has a Federation cluster, or `status_code()` is no longer a `match self.error_type`, file a DQ pending entry — the plan may have drifted.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

**Shape-G suspended until 2026-06-01** (DQ #229/#276). After committing + pushing your worktree branch, write a `kind: "validate-pending-laptop"` DQ entry (per `.claude/agents/impl-task.md` "Pre-Shape-G plans" + `.claude/rules/decision-queue.md` `kind: "validate-pending-laptop"`) with `from: "impl"`, `phase_task: 1`, `branch: <your-worktree-branch>`, and `commands[]` set **verbatim** to:

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task1-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task1-clippy.log 2>&1"
```

Do NOT write `kind: "validate-pending"` and do NOT capture a `workflow_run_id` — there is no GitHub Actions run (workflows disabled until 2026-06-01). The advisor laptop session reads this entry, runs both commands locally, and mutates the entry (`result: pass|fail`).

If the `.claude/decision-queue.json` write is gated by the sensitive-file gate, use the §4 harness-gap escalation path with `TASK1_VALIDATE_PENDING.json` at worktree root.

Per `feedback_pipes_mask_exit_codes.md`: you do not run these commands yourself (the laptop advisor does) — your job is only to apply the §10.1 edits + raise the entry with the commands verbatim.

## §6 Expected output (return to advisor)

```
## Task 1 complete — v1-federation-inbound-b LemmyErrorType + HTTP mapper

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/utils/src/error.rs (+6 LemmyErrorType variants, +6 status_code() arms 403/413/400/429/429/409)
**Anchor verification:** Federation cluster found at line <N> (plan said ~114-122); status_code() match at line <M> (plan said ~223-230); both inside cfg_select!(feature="full") confirmed
**validate-pending-laptop DQ:** #<id> raised (commands: cargo-check.bat + cargo-clippy.bat -D warnings)
**Next:** advisor laptop runs §15 cargo, mutates DQ #<id>; Cohort A barrier waits on all 3 tasks (1+2+3)
```

Plus any DQ #N references if you raised a blocker mid-task.

## §7 Why this brief differs from the plan

It does not — Task 1's scope is exactly plan §10.1 + §13 Task 1. This brief adds only: (a) §0 forbidden-window self-check wording, (b) §4 harness-gap interim escalation path (applies only if a DQ write is gated) + the explicit arm-ordering / cfg-select / non_exhaustive reminders from §13 Task 1, (c) §5 explicit `validate-pending-laptop` shape per PRECON-3 (Shape G suspended), (d) explicit "plan line numbers may have drifted — trust grep for the anchors". The 6 variants + 6 status-code arms are §10.1 verbatim — do not deviate. Mirrors the canonical sibling `.claude/PRPs/briefs/federation-inbound-a-impl-1.md` per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate.

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; per DQ #279 the canonical session drives fed-in-b impl under serial-phase policy — single active phase, no lane race). Brief committed on `governance-v0` before the Junior Task-1 task is queued; then propagated onto `phase-v1-federation-inbound-b` (conflict-free trunk-merge in the lane worktree) so the worker — which branches from the PHASE branch — sees it. `/precheck` re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC before queue._
