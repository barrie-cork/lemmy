---
phase: v1-federation-inbound-b
role: impl-task
task: 2
brief_n: 1
authored: 2026-05-19
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
plan_task: "§13 Task 2 — add #[serde(deny_unknown_fields)] to 3 governance protocol structs"
parent_phase_tip: 753289320 (phase-v1-federation-inbound-b @ Task-0 audit pass, origin tip)
cohort: "Cohort A (Tasks 1-3, 3-way [P]) — dispatched in parallel"
related_dq: "276 (proceed-as-one, user-resolved), 279 (serial-phase policy)"
---

# [role:impl-task] v1-federation-inbound-b Task 2 — deny_unknown_fields on 3 governance protocols — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-2.md

> **Clarify provenance:** parent **planning** brief clarified via `/brehon-clarify` (DQ #273/#274/#275 resolved advisor-mode; DQ #276 split-or-proceed user-resolved **proceed-as-one**). PRECON-3 (BINDING) = §15 is `validate-pending-laptop` shape (Shape G SUSPENDED until 2026-06-01 per DQ #229). Impl-task brief; no clarify-DQ gates it directly.

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is `phase-v1-federation-inbound-b`. If `git branch --show-current` is anything else → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check (per `.claude/agents/impl-task.md` task-0 discipline): `date -u +"%a %H:%M UTC"` — inside a forbidden window (Daily 02:55–04:15 / Sun 01:55–02:35 / Sun 03:55–04:30 / Wed 03:55–04:15 UTC) → exit non-zero `FORBIDDEN_WINDOW: <window>`. Shape G SUSPENDED (cargo on laptop); keep the check.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b Task 2 — #[serde(deny_unknown_fields)] on 3 governance protocol structs`

```
[role:impl-task] v1-federation-inbound-b Task 2 — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-2.md
```

## §2 Scope

**Produce** (one commit):

- `crates/apub/objects/src/protocol/governance/sanction_notice.rs` — add `#[serde(deny_unknown_fields)]` adjacent to the existing `#[serde(rename_all = "camelCase")]` attribute on the outer `*Protocol` struct (plan §10.2; plan cites the camelCase attr at line :40 — verify by grep).
- `crates/apub/objects/src/protocol/governance/trust_attestation.rs` — same attr on its outer `*Protocol` struct (plan cites struct/attr near line :33 — verify by grep).
- `crates/apub/objects/src/protocol/governance/moderation_label.rs` — same attr on its outer `*Protocol` struct (plan cites near line :30 — verify by grep).

Exactly **+1 attribute line per file** (3 files, 3 lines total). The attribute goes on the outer typed `*Protocol` struct — the one already carrying `#[serde(rename_all = "camelCase")]` and `#[serde(rename = "type")]` on its `kind` field (plan §10.2 MIRROR).

**Do NOT** in this task:

- Touch `error.rs` (Task 1), `governance_log.rs` / the registry (Task 3), `inbox.rs` (Task 4), any `publish_*.rs` (Cohort B), `scheduled_tasks.rs` (Task 8), `e2e.rs` (Task 9).
- Modify the stub structs in the `lemmy_apub_activities` crate (those carry the `rest` catch-all map and are NOT in Task 2 scope — only the typed `*Protocol` structs in `crates/apub/objects/src/protocol/governance/` get the attr).
- Touch any field, derive, or enum — ONLY add the one `#[serde(deny_unknown_fields)]` attribute per struct. If you find yourself editing a field list or a `SanctionAction`/`AttestationType` enum, STOP (out of scope).

**Commit message** (exactly): `feat(v1-federation-inbound-b): protocol structs — deny_unknown_fields on 3 governance protocols (task 2)`

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #276 (BINDING — proceed-as-one), DQ #279 (serial-phase policy).
2. **Plan §10.2** — the authoritative attr-placement block + the **field-coverage `#[serde(flatten)]` gotcha** (copy the attr placement verbatim; this is the contract).
3. **Plan §13 "Task 2"** — step list + the per-file pre-edit `#[serde(flatten)]` check + the test-serialise-compat GOTCHA.
4. **MIRROR ref** — each struct's existing `#[serde(rename_all = "camelCase")]` + `#[serde(rename = "type")]` attrs (plan §10.2 shows the `sanction_notice.rs` shape). **`grep -n 'rename_all = "camelCase"'` in each of the 3 files** to find the real attr position (line cites :40/:33/:30 are plan-time and may have drifted).
5. **Lessons** (per `.claude/rules/advisor-orchestrator.md` §2.4 — Task 2 runs `cargo-clippy.bat ... -D warnings`):
   - `.claude/lessons/feedback_clippy_test_style.md` — **Why:** the workspace clippy config denies escape-hatches; even a one-line attr add must leave the file clippy-clean under `-D warnings` (the §5 validation runs `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings`).
   - `.claude/lessons/feedback_clippy_rerun_after_fix.md` — **Why:** a stale clippy pass is not evidence; re-run after any fix.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — capture-then-tail for the §5 commands).

## §3a Handover from prior cohort

(none — Cohort A is the first impl cohort; Task 0 was a pure read-only probe with no `HANDOVER:` trailer.)

## §4 Constraints (hard rules)

### Branch + commit discipline

- Junior worktree off `phase-v1-federation-inbound-b`; finalize merges back; do NOT push to the phase branch directly.
- One commit (3 one-line edits). If clippy fails on first attempt, amend/fixup — do NOT split.
- Mid-task DQ visibility: raise a `pending` entry → **commit + push immediately** to your worktree branch.
- No `answered_by: "advisor"` / `"user"`. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235 — interim escalation-and-transcribe)

If a `.claude/decision-queue.json` write is gated by the sensitive-file gate: write the JSON to `TASK2_BLOCKER_DQ.json` (or `TASK2_VALIDATE_PENDING.json` for §5) + a short `TASK2_ESCALATION.md` at worktree root + commit both + push + STOP. Advisor transcribes per `.claude/rules/escalation.md`. Task 2's happy path has zero `.claude/` writes except the §5 entry.

### `#[serde(flatten)]` pre-edit check (BINDING — plan §10.2 + §13 Task 2)

For EACH of the 3 files, **before** adding the attr: read the outer `*Protocol` struct's full field list. If the struct uses `#[serde(flatten)]` with a catch-all `Map<String, Value>` / `rest` field at the OUTER level, `deny_unknown_fields` is incompatible — **STOP for that struct, file `kind: "blocker"`** (`from: "impl"`) naming the file + the flattened field, do NOT apply the attr to that file, and do NOT proceed to commit until the advisor resolves. Plan-time `grep -n "flatten" crates/apub/objects/src/protocol/governance/*.rs` returned empty (the typed `*Protocol` outer structs do NOT use `flatten`; only the `lemmy_apub_activities` stubs carry `rest`, which Task 2 does NOT modify) — so the happy path applies the attr to all 3. The check is a guard against drift since plan-write.

### Plan-cited line numbers may have drifted

§10.2 is the contract for *placement* (adjacent to `rename_all = "camelCase"` on the outer `*Protocol` struct). The :40/:33/:30 line cites are plan-time — `grep -n 'rename_all = "camelCase"'` in each file for the real position. If a file no longer has a `*Protocol` outer struct with that attr, file a DQ pending entry.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

After committing + pushing your worktree branch, write a `kind: "validate-pending-laptop"` DQ entry (`from: "impl"`, `phase_task: 2`, `branch: <your-worktree-branch>`) with `commands[]` set **verbatim** to:

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task2-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task2-clippy.log 2>&1"
```

Do NOT write `kind: "validate-pending"` / capture a `workflow_run_id`. The advisor laptop session runs both commands and mutates the entry (`result: pass|fail`). If the DQ write is gated, use the §4 harness-gap path with `TASK2_VALIDATE_PENDING.json`.

Per `feedback_pipes_mask_exit_codes.md`: you do not run these commands yourself — your job is only to apply the 3 attr edits + raise the entry with the commands verbatim.

## §6 Expected output (return to advisor)

```
## Task 2 complete — v1-federation-inbound-b deny_unknown_fields on 3 governance protocols

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/apub/objects/src/protocol/governance/sanction_notice.rs   (+1 #[serde(deny_unknown_fields)])
  - crates/apub/objects/src/protocol/governance/trust_attestation.rs (+1 #[serde(deny_unknown_fields)])
  - crates/apub/objects/src/protocol/governance/moderation_label.rs  (+1 #[serde(deny_unknown_fields)])
**flatten pre-edit check:** none of the 3 outer *Protocol structs use #[serde(flatten)] (confirmed by read) — attr applied to all 3
**Anchor verification:** rename_all="camelCase" found at sanction_notice:<N> / trust_attestation:<M> / moderation_label:<K>
**validate-pending-laptop DQ:** #<id> raised (commands: cargo-check.bat + cargo-clippy.bat -D warnings)
**Next:** advisor laptop runs §15 cargo, mutates DQ #<id>; Cohort A barrier waits on all 3 tasks (1+2+3)
```

Plus any DQ #N references if you raised a blocker mid-task (e.g. a struct with `#[serde(flatten)]`).

## §7 Why this brief differs from the plan

It does not — Task 2's scope is exactly plan §10.2 + §13 Task 2. Additions: (a) §0 forbidden-window self-check, (b) §4 harness-gap escalation path (gated-write contingency only) + the explicit per-file `#[serde(flatten)]` pre-edit blocker rule from §10.2, (c) §5 explicit `validate-pending-laptop` shape per PRECON-3, (d) explicit "plan line numbers may have drifted — grep for the camelCase anchor". The attr + its placement are §10.2 verbatim — do not deviate. Mirrors the canonical sibling `.claude/PRPs/briefs/federation-inbound-a-impl-3.md` per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate.

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; per DQ #279 the canonical session drives fed-in-b impl under serial-phase policy). Brief committed on `governance-v0` before the Junior Task-2 task is queued; then propagated onto `phase-v1-federation-inbound-b` (conflict-free trunk-merge in the lane worktree) so the worker — which branches from the PHASE branch — sees it. `/precheck` re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC before queue._
