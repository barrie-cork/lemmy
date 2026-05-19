---
phase: v1-federation-inbound-a
role: impl-task
task: 3
brief_n: 1
authored: 2026-05-16
plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md
plan_task: "§13 Task 3 — UPDATE crates/api/api/src/governance/config.rs (11 federation.inbound.* keys)"
parent_phase_tip: e67e794cf (phase-v1-federation-inbound-a @ registry pre-write)
cohort: "Cohort A (Tasks 1-5, 5-way [P]) — dispatched in parallel"
related_dq: "232 (additive-only shared files)"
---

# [role:impl-task] v1-federation-inbound-a Task 3 — config.rs — see .claude/PRPs/briefs/federation-inbound-a-impl-3.md

> **Clarify provenance:** parent planning brief clarified via `/brehon-clarify` (DQ #230/#231/#232 resolved advisor-mode). DQ #232 (BINDING) = ALL shared-file edits strictly additive — `config.rs` is a DQ #232 shared file (3 concurrent CC lanes). Impl-task brief; no clarify-DQ gates it directly.

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is `phase-v1-federation-inbound-a`. If not → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check: `date -u +"%a %H:%M UTC"` — inside a forbidden window → exit non-zero `FORBIDDEN_WINDOW: <window>`. Shape G SUSPENDED (cargo on laptop); keep the check.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-a Task 3 — config.rs: 11 federation.inbound.* keys + EXPECTED_SEED_COUNT + metadata + parity`

```
[role:impl-task] v1-federation-inbound-a Task 3 — see .claude/PRPs/briefs/federation-inbound-a-impl-3.md
```

## §2 Scope

**Produce** (one commit):

- `crates/api/api/src/governance/config.rs` — per plan §10.6:
  - **11 new `DEFAULT_FEDERATION_INBOUND_*` consts** (verbatim from §10.6 — 10 `i64` + 1 `&str`).
  - **11 match arms** (10 int + 1 text) in the config-key resolver, in the same style as the v1-RT-r1 block.
  - **11 `SEEDED_KEYS` tuples**, alphabetised within a NEW isolated block.
  - **`pub const EXPECTED_SEED_COUNT_V1_FED_IN: usize = 11;`** placed AFTER `EXPECTED_SEED_COUNT_V1_RT` (plan cites line 1576 — verify by grep).
  - **`pub const ENUM_FEDERATION_PEER_TRUST: &[&str] = &["unknown", "untrusted_receive", "blocklisted"];`** placed AFTER `ENUM_MULTI_SPONSOR_ESCAPE_RULE` (plan cites line 1601 — verify by grep).
  - **Parity-test sum extension** (add `EXPECTED_SEED_COUNT_V1_FED_IN` to the running parity total per §10.6) + **error-message extension** (the seed-count mismatch error string lists the new constant).
  - **11 `CONFIG_KEY_METADATA` entries** — one per key, exactly per the §10.6 table (value_type / valid_range / valid_enum / scope / apply_at). All `requires_re_jury: false`, `requires_step_up: false`, `doc_anchor: "v1-federation-inbound.prd.md§10"`.

**Do NOT** in this task:

- Touch the migration (Task 1), `schema.rs` (Task 2), `governance_log.rs` (Task 4), `newtypes.rs` (Task 5), any Diesel model (Cohort B), `e2e.rs` (Task 9).
- Reorder/reformat any sibling-lane const block (v1-ship / v1-AD-e). **APPEND-ONLY** per DQ #232 — your `SEEDED_KEYS` go in a NEW isolated block; `EXPECTED_SEED_COUNT_V1_FED_IN` + `ENUM_FEDERATION_PEER_TRUST` are NEW consts placed AFTER the named anchors; the parity sum is an additive `+ EXPECTED_SEED_COUNT_V1_FED_IN`.

**Commit message** (exactly): `feat(v1-federation-inbound-a): config.rs — 11 federation.inbound.* keys + EXPECTED_SEED_COUNT_V1_FED_IN + metadata + parity (task 3)`

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #232 (BINDING — `config.rs` shared; new isolated `SEEDED_KEYS` block + new consts after named anchors + additive parity sum; NO reorder of sibling blocks).
2. **Plan §10.6** — the authoritative 11 consts (verbatim) + match-arm count + SEEDED_KEYS rule + the exact `EXPECTED_SEED_COUNT_V1_FED_IN` / `ENUM_FEDERATION_PEER_TRUST` placement + the 11-row `CONFIG_KEY_METADATA` table. This is the contract.
3. **Plan §13 "Task 3"** — step list + GOTCHAs (APPEND-ONLY).
4. **MIRROR ref** — `crates/api/api/src/governance/config.rs:1498-1519` (the v1-RT-r1 `SEEDED_KEYS` block — mirror its tuple shape + alphabetisation exactly). Also grep the existing `EXPECTED_SEED_COUNT_V1_RT` + `ENUM_MULTI_SPONSOR_ESCAPE_RULE` consts for the placement anchors (plan-cited lines 1576/1601 may have drifted — trust grep).
5. **Lessons** (MANDATORY per `.claude/rules/advisor-orchestrator.md` §2.4 — Task 3 runs `cargo-clippy.bat ... -D warnings` on new code):
   - `.claude/lessons/feedback_clippy_test_style.md` — **Why:** the Lemmy workspace clippy config denies `unwrap`/`expect`/`#[allow]` escape-hatches; new const + match-arm code must be clippy-clean under `-D warnings`. (The §5 validation runs `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings`.)
   - `.claude/lessons/feedback_clippy_rerun_after_fix.md` — **Why:** if a clippy fix is needed, clippy must be re-run (a stale pass is not evidence the fix worked).
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — capture-then-tail for the §5 commands).

## §3a Handover from prior cohort

(none — Cohort A is the first impl cohort; Task 0 was a pure read-only probe with no `HANDOVER:` trailer.)

## §4 Constraints (hard rules)

### Branch + commit discipline

- Junior worktree off `phase-v1-federation-inbound-a`; finalize merges back; do not push to the phase branch directly.
- One commit. If clippy fails on first attempt, amend/fixup — do NOT split.
- Mid-task DQ visibility: raise a `pending` entry → **commit + push immediately** to your worktree branch.
- No `answered_by: "advisor"` / `"user"`. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235 — interim escalation-and-transcribe)

If a `.claude/decision-queue.json` write is gated: write the JSON to `TASK3_BLOCKER_DQ.json` (or `TASK3_VALIDATE_PENDING.json` for §5) + `TASK3_ESCALATION.md` at worktree root + commit both + push + STOP. Advisor transcribes per `.claude/rules/escalation.md`.

### DQ #232 APPEND-ONLY + clippy discipline

- `config.rs` is edited by 3 lanes. New `SEEDED_KEYS` go in a NEW isolated block (do NOT interleave into a sibling lane's block). `EXPECTED_SEED_COUNT_V1_FED_IN` + `ENUM_FEDERATION_PEER_TRUST` are new consts AFTER the named anchors. Parity sum is `... + EXPECTED_SEED_COUNT_V1_FED_IN` (additive term). Do NOT reorder/reformat existing blocks.
- Do NOT `#[allow]`-spam to silence clippy. Fix the root cause (per `feedback_clippy_test_style.md`). If a genuine clippy finding can't be resolved without an allow, file a DQ pending entry rather than spamming.

### Plan-cited line numbers WILL have drifted

`config.rs` is heavily edited by concurrent lanes. The plan cites line 1576 (`EXPECTED_SEED_COUNT_V1_RT`) and 1601 (`ENUM_MULTI_SPONSOR_ESCAPE_RULE`) as placement anchors — these are almost certainly stale. `grep -n 'EXPECTED_SEED_COUNT_V1_RT'` and `grep -n 'ENUM_MULTI_SPONSOR_ESCAPE_RULE'` to find the real positions and place your new consts immediately AFTER them. If the parity-sum expression or the seed-count error string has changed shape since plan-write, follow the actual code; if the semantics differ materially, file a DQ pending entry.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

After committing + pushing, write a `kind: "validate-pending-laptop"` DQ entry (`from: "impl"`, `phase_task: 3`, `branch: <your-worktree-branch>`) with `commands[]` set **verbatim** to:

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task3-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-a-task3-clippy.log 2>&1"
```

Do NOT write `kind: "validate-pending"` / capture a `workflow_run_id`. The advisor laptop session runs both commands and mutates the entry. If the DQ write is gated, use the §4 harness-gap path with `TASK3_VALIDATE_PENDING.json`.

## §6 Expected output (return to advisor)

```
## Task 3 complete — v1-federation-inbound-a config.rs

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/api/api/src/governance/config.rs (+11 DEFAULT_ consts, +11 match arms, +11 SEEDED_KEYS, +EXPECTED_SEED_COUNT_V1_FED_IN, +ENUM_FEDERATION_PEER_TRUST, +parity sum, +11 CONFIG_KEY_METADATA)
**Anchor verification:** EXPECTED_SEED_COUNT_V1_RT found at line <N> (plan said 1576); ENUM_MULTI_SPONSOR_ESCAPE_RULE at line <M> (plan said 1601)
**Append-only confirmed:** no sibling-lane block reordered/reformatted
**validate-pending-laptop DQ:** #<id> raised (commands: cargo-check.bat + cargo-clippy.bat -D warnings)
**Next:** advisor laptop runs §15 cargo, mutates DQ #<id>; Cohort A barrier waits on all 5 tasks
```

Plus any DQ #N references.

## §7 Why this brief differs from the plan

It does not — Task 3's scope is exactly plan §10.6 + §13 Task 3. Additions: (a) §0 forbidden-window self-check, (b) §4 harness-gap escalation path (gated-write contingency only) + explicit clippy-no-allow-spam reminder, (c) §5 explicit `validate-pending-laptop` shape per DQ #231, (d) explicit "plan line numbers WILL have drifted — trust grep for the placement anchors" (config.rs is the most concurrently-edited file). The 11 consts + metadata table are §10.6 verbatim — do not deviate.
