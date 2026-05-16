---
phase: v1-federation-inbound-a
role: impl-task
task: 5
brief_n: 1
authored: 2026-05-16
plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md
plan_task: "§13 Task 5 — UPDATE crates/db_schema/src/newtypes.rs (FederationInboxDroppedLogId + RemoteModerationLabelId)"
parent_phase_tip: e67e794cf (phase-v1-federation-inbound-a @ registry pre-write)
cohort: "Cohort A (Tasks 1-5, 5-way [P]) — dispatched in parallel"
related_dq: "232 (additive-only shared files), 234 (no FederationPeerId)"
---

# [role:impl-task] v1-federation-inbound-a Task 5 — newtypes.rs — see .claude/PRPs/briefs/federation-inbound-a-impl-5.md

> **Clarify provenance:** parent planning brief clarified via `/brehon-clarify` (DQ #230/#231/#232 resolved advisor-mode). DQ #232 (BINDING) = ALL shared-file edits strictly additive — `newtypes.rs` is a DQ #232-class shared file. DQ #234 (RESOLVED) = NO `FederationPeerId` newtype (reuse `InstanceId`). Impl-task brief; no clarify-DQ gates it directly.

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is `phase-v1-federation-inbound-a`. If not → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check: `date -u +"%a %H:%M UTC"` — inside a forbidden window → exit non-zero `FORBIDDEN_WINDOW: <window>`. Shape G SUSPENDED (cargo on laptop); keep the check.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-a Task 5 — newtypes.rs: FederationInboxDroppedLogId (i64) + RemoteModerationLabelId (i32)`

```
[role:impl-task] v1-federation-inbound-a Task 5 — see .claude/PRPs/briefs/federation-inbound-a-impl-5.md
```

## §2 Scope

**Produce** (one commit):

- `crates/db_schema/src/newtypes.rs` — **append AFTER the v1-RT-r1 block** (grep for the v1-RT-r1 newtype section marker to find the insertion point) the EXACT block from plan §13 Task 5:

```rust
// ========================================================================
// Federation inbound governance typed IDs (v1-federation-inbound-a)
// ========================================================================

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct FederationInboxDroppedLogId(pub i64);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct RemoteModerationLabelId(pub i32);
```

(Copy verbatim — the derive attribute set + the `i64` vs `i32` inner types are load-bearing.)

**Do NOT** in this task:

- Touch the migration (Task 1), `schema.rs` (Task 2), `config.rs` (Task 3), `governance_log.rs` (Task 4), any Diesel model (Cohort B), `e2e.rs` (Task 9).
- Add a `FederationPeerId` newtype — per DQ #234 there is NO such type (`federation_peer` keys on the existing `InstanceId`; federation_blocklist precedent). Add ONLY the 2 structs above.
- Add a newtype for `federation_inbox_nonce` — it has a composite PK (no `id` column), so no id newtype exists for it (per plan §13 Task 5 GOTCHA).
- Reorder/reformat any sibling-lane newtype block. **APPEND-ONLY** per DQ #232 — your block is a NEW section appended after the v1-RT-r1 block.

**Commit message** (exactly): `feat(v1-federation-inbound-a): newtypes.rs — FederationInboxDroppedLogId + RemoteModerationLabelId (task 5)`

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #232 (BINDING — `newtypes.rs` shared; new appended section only; NO reorder), DQ #234 (BINDING — no `FederationPeerId`; only the 2 structs above).
2. **Plan §13 "Task 5"** — the authoritative block (verbatim above) + the BIGSERIAL→i64 / SERIAL→i32 GOTCHA + the "no `FederationPeerId`" / "no nonce id" GOTCHAs.
3. **MIRROR ref** — **Phase 6 federation typed-IDs at `crates/db_schema/src/newtypes.rs` lines 295-330** (per plan §13 Task 5 MIRROR). Match the derive-attribute ordering + `cfg_attr` style exactly. Also grep the v1-RT-r1 newtype block marker for the append position.
4. **Lessons** (MANDATORY per `.claude/rules/advisor-orchestrator.md` §2.4 — "Any newtype under `crates/db_schema/src/newtypes`"):
   - `.claude/lessons/feedback_newtype_locations_lemmy_db_schema_vs_file.md` — **Why:** newtype authoritative definitions live in `lemmy_db_schema::newtypes` (`crates/db_schema/src/newtypes.rs` — THIS file). `lemmy_db_schema_file` re-exports only `PersonId` + `InstanceId`. Confirms these 2 new ids belong HERE (not in `db_schema_file`); confirms `federation_peer` correctly reuses `InstanceId` (already re-exported) per DQ #234 rather than a new type.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — capture-then-tail for §5).

## §3a Handover from prior cohort

(none — Cohort A is the first impl cohort; Task 0 was a pure read-only probe with no `HANDOVER:` trailer.)

## §4 Constraints (hard rules)

### Branch + commit discipline

- Junior worktree off `phase-v1-federation-inbound-a`; finalize merges back; do not push to the phase branch directly.
- One commit.
- Mid-task DQ visibility: raise a `pending` entry → **commit + push immediately** to your worktree branch.
- No `answered_by: "advisor"` / `"user"`. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235 — interim escalation-and-transcribe)

If a `.claude/decision-queue.json` write is gated: write the JSON to `TASK5_BLOCKER_DQ.json` (or `TASK5_VALIDATE_PENDING.json` for §5) + `TASK5_ESCALATION.md` at worktree root + commit both + push + STOP. Advisor transcribes per `.claude/rules/escalation.md`.

### DQ #232 APPEND-ONLY

`newtypes.rs` is edited by 3 lanes. Your 2 structs go in a NEW section appended after the v1-RT-r1 block (grep for its marker). Do NOT reorder/reformat existing newtype sections — a reformat touching a sibling-lane block is a cross-lane reconcile collision and a process breach.

### Inner-type correctness (load-bearing)

`FederationInboxDroppedLogId(pub i64)` — `federation_inbox_dropped_log.id` is **BIGSERIAL** → `i64`. `RemoteModerationLabelId(pub i32)` — `remote_moderation_label.id` is **SERIAL** → `i32`. Do NOT swap these — a Diesel type mismatch against Task 2's `schema.rs` `table!` block would surface at Cohort B Task 6/7 `cargo check` (those `requires: task 2` + `task 5`).

### Plan-cited line numbers may have drifted

§13 Task 5 cites Phase-6 newtypes at lines 295-330 (MIRROR) and "after the v1-RT-r1 block" (append position). `grep -n` the Phase-6 federation id structs + the v1-RT-r1 section marker to find the real positions. Append your section per the actual code.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

After committing + pushing, write a `kind: "validate-pending-laptop"` DQ entry (`from: "impl"`, `phase_task: 5`, `branch: <your-worktree-branch>`) with `commands[]` set **verbatim** to:

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task5-check.log 2>&1"
```

Do NOT write `kind: "validate-pending"` / capture a `workflow_run_id`. The advisor laptop session runs the command and mutates the entry. If the DQ write is gated, use the §4 harness-gap path with `TASK5_VALIDATE_PENDING.json`.

## §6 Expected output (return to advisor)

```
## Task 5 complete — v1-federation-inbound-a newtypes.rs

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/db_schema/src/newtypes.rs (+FederationInboxDroppedLogId(i64), +RemoteModerationLabelId(i32) — new appended section)
**Inner types:** FederationInboxDroppedLogId=i64 (BIGSERIAL), RemoteModerationLabelId=i32 (SERIAL) — confirmed
**No FederationPeerId / no nonce id:** confirmed per DQ #234 + composite-PK GOTCHA
**Append-only confirmed:** no sibling-lane block reordered/reformatted
**validate-pending-laptop DQ:** #<id> raised (command: cargo-check.bat --workspace --features full)
**Next:** advisor laptop runs §15 cargo, mutates DQ #<id>; Cohort A barrier waits on all 5 tasks
```

Plus any DQ #N references.

## §7 Why this brief differs from the plan

It does not — Task 5's scope is exactly plan §13 Task 5. Additions: (a) §0 forbidden-window self-check, (b) §4 harness-gap escalation path (gated-write contingency only), (c) §5 explicit `validate-pending-laptop` shape per DQ #231, (d) explicit BIGSERIAL/SERIAL inner-type emphasis (Cohort B Tasks 6/7 `requires` this). The 2 structs are §13 Task 5 verbatim — do not deviate.
