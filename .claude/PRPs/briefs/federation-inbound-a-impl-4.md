---
phase: v1-federation-inbound-a
role: impl-task
task: 4
brief_n: 1
authored: 2026-05-16
plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md
plan_task: "§13 Task 4 — UPDATE governance_log.rs ×2 + entry-kind registry (crates/ ONLY — registry pre-written by advisor)"
parent_phase_tip: e67e794cf (phase-v1-federation-inbound-a @ registry pre-write)
cohort: "Cohort A (Tasks 1-5, 5-way [P]) — dispatched in parallel"
related_dq: "232 (additive-only shared files), 235 (harness-gap: Junior CANNOT write .claude/** — registry pre-landed by advisor)"
---

# [role:impl-task] v1-federation-inbound-a Task 4 — governance_log ×2 (crates/ ONLY) — see .claude/PRPs/briefs/federation-inbound-a-impl-4.md

> **Clarify provenance:** parent planning brief clarified via `/brehon-clarify` (DQ #230/#231/#232 resolved advisor-mode). DQ #232 (BINDING) = ALL shared-file edits strictly additive. DQ #235 (RESOLVED fix-daemon) = Junior workers CANNOT write `.claude/**` — so this Task 4 brief is SCOPED-DOWN vs the plan: Junior writes ONLY the 2 `crates/` `governance_log.rs` files. Impl-task brief; no clarify-DQ gates it directly.

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is `phase-v1-federation-inbound-a`. If not → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check: `date -u +"%a %H:%M UTC"` — inside a forbidden window → exit non-zero `FORBIDDEN_WINDOW: <window>`. Shape G SUSPENDED (cargo on laptop); keep the check.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-a Task 4 — governance_log.rs ×2: 9 new ENTRY_KIND consts + 9 pub use re-exports (crates/ ONLY)`

```
[role:impl-task] v1-federation-inbound-a Task 4 — see .claude/PRPs/briefs/federation-inbound-a-impl-4.md
```

## §2 Scope — SCOPED DOWN from plan §13 (registry is advisor-pre-landed)

**Produce** (one commit):

- `crates/db_schema/src/source/governance/governance_log.rs` — per plan §10.7: **9 new `ENTRY_KIND_FEDERATION_*` consts** (verbatim from §10.7 — `ENTRY_KIND_FEDERATION_INBOUND_BLOCKED` through `ENTRY_KIND_FEDERATION_PEER_TRUST_CHANGED`), as a NEW block AFTER line 218 (verify by grep), **alphabetical within the new block**.
- `crates/api/api/src/governance/governance_log.rs` — per plan §10.7: **9 alphabetical `pub use` re-exports** of those 9 consts, placed within the existing re-export block (plan cites lines 39-88 — verify by grep; insert alphabetically).

**DO NOT** in this task — CRITICAL SCOPE BOUNDARY:

- **DO NOT touch `.claude/rules/governance-log-entry-kind-registry.md`.** The plan §13 Task 4 FILES YAML lists this file, but it was **PRE-WRITTEN by the advisor and already landed on the phase branch** (commit `e67e794cf`, "docs(advisor): pre-write v1-federation-inbound-a registry section"). Per DQ #235, Junior workers cannot write `.claude/**` (Claude Code sensitive-file gate). The registry section (9 entry-kind rows + count 45→54) is ALREADY DONE. If you attempt to edit it you will hit the sensitive-file gate; do NOT try, do NOT escalate it — it is intentionally out of your scope. Your scope is the 2 `crates/` files ONLY.
- Do NOT touch the migration (Task 1), `schema.rs` (Task 2), `config.rs` (Task 3), `newtypes.rs` (Task 5), any Diesel model (Cohort B), `e2e.rs` (Task 9).
- Do NOT reorder/reformat any sibling-lane const or `pub use` block (v1-ship / v1-AD-e). **APPEND-ONLY** per DQ #232 — your 9 consts are a new block; your 9 `pub use` are alphabetical insertions into the existing list (additive).

**Commit message** (exactly): `feat(v1-federation-inbound-a): governance_log + entry-kind registry — 9 new federation_inbound_* consts (task 4)`

> Commit subject mentions "registry" for plan-§13-traceability even though the registry file itself was pre-landed by the advisor — the subject maps to plan §13 Task 4; the registry row content is already on the branch via `e67e794cf`. This is intentional and correct.

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #232 (BINDING — both `governance_log.rs` files are shared; new const block + alphabetical `pub use` insertions only; NO reorder), DQ #235 (RESOLVED fix-daemon — Junior cannot write `.claude/**`; registry pre-landed; Task 4 scoped to `crates/` only).
2. **Plan §10.7** — the authoritative 9 consts (verbatim) + the `pub use` re-export rule (alphabetical, lines 39-88). The registry markdown in §10.7 is FYI-only — it is already landed; do NOT reproduce it.
3. **Plan §13 "Task 4"** — step list. NOTE the FILES YAML lists the registry `.md` but your brief §2 overrides that (registry pre-landed).
4. **MIRROR ref** — the existing `ENTRY_KIND_*` const block in `crates/db_schema/src/source/governance/governance_log.rs` around line 218 (grep `^pub const ENTRY_KIND_` for the real boundary) + the existing `pub use` block in `crates/api/api/src/governance/governance_log.rs` lines 39-88 (grep `pub use` for the real range). Mirror the const + re-export shape exactly.
5. **Lessons** (no §2.4 mandatory injection for `governance_log.rs` — not a handler with DB writes, not in the file-class table — but cross-cutting):
   - `.claude/lessons/feedback_cohort_validation_dependency_check.md` — **Why:** confirms Cohort A is validation-independent. Task 4's `cargo check --workspace --features full` compiles the new consts + re-exports; these are pure `&str` consts with no dependency on Task 1's migration or any other Cohort A task.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — capture-then-tail for §5).

## §3a Handover from prior cohort

(none — Cohort A is the first impl cohort; Task 0 was a pure read-only probe with no `HANDOVER:` trailer. Note: the registry section was advisor-pre-landed at `e67e794cf`, not via a Junior cohort — it is not a "handover".)

## §4 Constraints (hard rules)

### Branch + commit discipline

- Junior worktree off `phase-v1-federation-inbound-a`; finalize merges back; do not push to the phase branch directly.
- One commit.
- Mid-task DQ visibility: raise a `pending` entry → **commit + push immediately** to your worktree branch.
- No `answered_by: "advisor"` / `"user"`. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235 — interim escalation-and-transcribe)

If a `.claude/decision-queue.json` write is gated: write the JSON to `TASK4_BLOCKER_DQ.json` (or `TASK4_VALIDATE_PENDING.json` for §5) + `TASK4_ESCALATION.md` at worktree root + commit both + push + STOP. Advisor transcribes per `.claude/rules/escalation.md`. **NOTE:** the registry `.md` write being gated is EXPECTED and is NOT an escalation case — it is intentionally out of scope (advisor pre-landed it). Only escalate a gated DQ write, never a gated registry write.

### DQ #232 APPEND-ONLY

Both `governance_log.rs` files are edited by 3 lanes. Your 9 consts go in a NEW block (placed after the last existing `ENTRY_KIND_` const, or wherever §10.7 specifies a disjoint position). Your 9 `pub use` are alphabetical insertions into the existing re-export list. Do NOT reorder/reformat existing consts or re-exports — a reformat that touches a sibling-lane block is a cross-lane reconcile collision and a process breach.

### Plan-cited line numbers may have drifted

§10.7 cites "after line 218" (db_schema consts) and "lines 39-88" (api re-exports). `grep -n '^pub const ENTRY_KIND_'` and `grep -n 'pub use'` in the respective files to find the real positions. Insert your block / re-exports per the actual code, alphabetically. If the existing const count differs materially from expectation, file a DQ pending entry.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

After committing + pushing, write a `kind: "validate-pending-laptop"` DQ entry (`from: "impl"`, `phase_task: 4`, `branch: <your-worktree-branch>`) with `commands[]` set **verbatim** to:

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task4-check.log 2>&1"
```

Do NOT write `kind: "validate-pending"` / capture a `workflow_run_id`. The advisor laptop session runs the command and mutates the entry. If the DQ write is gated, use the §4 harness-gap path with `TASK4_VALIDATE_PENDING.json`.

## §6 Expected output (return to advisor)

```
## Task 4 complete — v1-federation-inbound-a governance_log ×2

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/db_schema/src/source/governance/governance_log.rs (+9 ENTRY_KIND_FEDERATION_* consts, new block)
  - crates/api/api/src/governance/governance_log.rs (+9 alphabetical pub use re-exports)
**Registry:** NOT touched (advisor pre-landed at e67e794cf — intentionally out of Junior scope per DQ #235)
**Append-only confirmed:** no sibling-lane block reordered/reformatted
**validate-pending-laptop DQ:** #<id> raised (command: cargo-check.bat --workspace --features full)
**Next:** advisor laptop runs §15 cargo, mutates DQ #<id>; Cohort A barrier waits on all 5 tasks
```

Plus any DQ #N references.

## §7 Why this brief differs from the plan

**This brief intentionally SCOPES DOWN plan §13 Task 4.** Plan §13 Task 4 FILES YAML lists 3 files including `.claude/rules/governance-log-entry-kind-registry.md`. Per DQ #235 (RESOLVED fix-daemon — Junior workers cannot write `.claude/**` due to the Claude Code sensitive-file gate), the advisor PRE-WROTE and pushed the registry section (commit `e67e794cf`). Junior's scope is the **2 `crates/` files ONLY**. Other additions: (a) §0 forbidden-window self-check, (b) §4 harness-gap escalation path + explicit "gated registry write is EXPECTED, not an escalation case", (c) §5 explicit `validate-pending-laptop` shape per DQ #231. The 9 consts + 9 re-exports are §10.7 verbatim — do not deviate.
