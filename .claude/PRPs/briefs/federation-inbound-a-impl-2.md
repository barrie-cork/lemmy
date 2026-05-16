---
phase: v1-federation-inbound-a
role: impl-task
task: 2
brief_n: 1
authored: 2026-05-16
plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md
plan_task: "§13 Task 2 — UPDATE crates/db_schema_file/src/schema.rs (4 new tables + extend Phase-6 blocks)"
parent_phase_tip: e67e794cf (phase-v1-federation-inbound-a @ registry pre-write)
cohort: "Cohort A (Tasks 1-5, 5-way [P]) — dispatched in parallel"
related_dq: "232 (additive-only shared files), 234 (no FederationPeerId — use InstanceId)"
---

# [role:impl-task] v1-federation-inbound-a Task 2 — schema.rs — see .claude/PRPs/briefs/federation-inbound-a-impl-2.md

> **Clarify provenance:** parent planning brief clarified via `/brehon-clarify` (DQ #230/#231/#232 resolved advisor-mode). DQ #232 (BINDING) = ALL shared-file edits strictly additive/append-only — `schema.rs` is a DQ #232 shared file (3 concurrent CC lanes: v1-ship, v1-AD-e). This is an impl-task brief; no clarify-DQ gates it directly.

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is `phase-v1-federation-inbound-a`. If not → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check: `date -u +"%a %H:%M UTC"` — if inside a forbidden window exit non-zero with `FORBIDDEN_WINDOW: <window>`. Shape G SUSPENDED (cargo on laptop) — concern reduced; keep the check.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-a Task 2 — schema.rs: 4 new table! blocks + extend 2 Phase-6 blocks`

```
[role:impl-task] v1-federation-inbound-a Task 2 — see .claude/PRPs/briefs/federation-inbound-a-impl-2.md
```

## §2 Scope

**Produce** (one commit):

- `crates/db_schema_file/src/schema.rs` — per plan §10.2: **4 new `table!` blocks** (`federation_peer`, `federation_inbox_dropped_log`, `federation_inbox_nonce`, `remote_moderation_label`) + **extend the 2 existing Phase-6 `table!` blocks** (`remote_sanction_notice`, `federation_attestation` — add the new columns matching Task 1's ALTER) + **2 `joinable!` macro entries** + **4 entries in `allow_tables_to_appear_in_same_query!`**. Add `// v1-federation-inbound-a additions:` inline marker comments at each insertion point.

**Do NOT** in this task:

- Touch the migration (Task 1), `config.rs` (Task 3), `governance_log.rs` (Task 4), `newtypes.rs` (Task 5), any Diesel model file (Cohort B), `e2e.rs` (Task 9).
- Use a `FederationPeerId` type anywhere — per DQ #234, `federation_peer` keys on the existing `InstanceId` (federation_blocklist precedent). The `table!` block's PK column is `instance_id -> Int4` (or whatever §10.2 specifies), NOT a new id type.
- Reorder, reformat, or otherwise touch any sibling-lane block (v1-ship / v1-AD-e additions). **APPEND-ONLY** per DQ #232 — your edits are strictly new `table!` blocks at disjoint positions + 2 new column lines inside the 2 Phase-6 blocks (additive). Reformatting an existing block = process breach.

**Commit message** (exactly): `feat(v1-federation-inbound-a): schema.rs — 4 new tables + extend Phase-6 blocks (task 2)`

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #232 (BINDING — `schema.rs` is a shared file; edits strictly additive; new `table!` blocks + 2 additive column lines only), DQ #234 (BINDING — no `FederationPeerId`; `federation_peer` keys on `InstanceId`).
2. **Plan §10.2** — the authoritative schema.rs additions (4 `table!` blocks verbatim + the exact 2 Phase-6 block extensions + `joinable!` + `allow_tables_to_appear_in_same_query!` entries). This is the contract — copy verbatim.
3. **Plan §13 "Task 2"** — step list + GOTCHAs (APPEND-ONLY, no `FederationPeerId`).
4. **MIRROR refs** — the existing Phase-6 `table!` blocks for `remote_sanction_notice` + `federation_attestation` in `crates/db_schema_file/src/schema.rs` (grep them; §10.2 cites their shape). Match Diesel column-type conventions exactly (`Int4`, `Int8`, `Text`, `Nullable<...>`, `Timestamptz`, `Jsonb`).
5. **Lessons** (no §2.4 mandatory injection for `schema.rs` — not in the file-class table — but cross-cutting):
   - `.claude/lessons/feedback_cohort_validation_dependency_check.md` — **Why:** confirms Cohort A is validation-independent. Task 2's `cargo check --workspace --features full` compiles Rust against `schema.rs`; it does NOT run Task 1's migration. The new `table!` blocks must be self-consistent Diesel DSL (correct column types) — they do not need Task 1's SQL present to typecheck. (Cohort B Tasks 6/7/8 `requires: task 2`, not the reverse.)
   - `.claude/lessons/feedback_newtype_locations_lemmy_db_schema_vs_file.md` — **Why:** `schema.rs` lives in `lemmy_db_schema_file`; if a `table!` block needs to reference a newtype, `db_schema_file` re-exports ONLY `PersonId` + `InstanceId`. Use `InstanceId` for `federation_peer` (per DQ #234). Other ids are NOT available in this crate — but §10.2's `table!` blocks use raw Diesel column types (`Int8` etc.), so this is a guard, not an expected edit.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — for the §5 validation command capture).

## §3a Handover from prior cohort

(none — Cohort A is the first impl cohort; Task 0 was a pure read-only probe with no `HANDOVER:` trailer.)

## §4 Constraints (hard rules)

### Branch + commit discipline

- Junior worktree off `phase-v1-federation-inbound-a`; finalize merges back; do not push to the phase branch directly.
- One commit.
- Mid-task DQ visibility: raise a `pending` entry → **commit + push immediately** to your worktree branch.
- No `answered_by: "advisor"` / `"user"`. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235 — interim escalation-and-transcribe)

If a `.claude/decision-queue.json` write is gated by the sensitive-file gate: write the JSON object to `TASK2_BLOCKER_DQ.json` (or `TASK2_VALIDATE_PENDING.json` for the §5 entry) at worktree root + `TASK2_ESCALATION.md` + commit both + push + STOP. Advisor transcribes per `.claude/rules/escalation.md`.

### DQ #232 APPEND-ONLY (load-bearing)

`schema.rs` is edited by 3 concurrent lanes. Your ONLY allowed edits:
- New `table!` blocks for the 4 `-a` tables, placed at a disjoint position (end of the governance table cluster or wherever §10.2 specifies — pick a position no sibling lane is touching).
- 2 additive column lines inside the existing `remote_sanction_notice` + `federation_attestation` `table!` blocks (matching Task 1's ALTER columns).
- New `joinable!` + `allow_tables_to_appear_in_same_query!` entries (additive list extensions).

Do NOT: reorder existing tables, reformat sibling blocks, change whitespace outside your additions, or "tidy" anything. A reformat that touches a v1-ship / v1-AD-e block is a cross-lane reconcile collision and a process breach.

### Plan-cited content may have drifted

§10.2 is the contract. If a Phase-6 `table!` block's column list has drifted since plan-write (another lane extended it), `grep -n` the actual block and append your 2 columns AFTER whatever is currently there (still additive). If the block shape differs materially from §10.2, file a DQ pending entry.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

After committing + pushing, write a `kind: "validate-pending-laptop"` DQ entry (`from: "impl"`, `phase_task: 2`, `branch: <your-worktree-branch>`) with `commands[]` set **verbatim** to:

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-a-task2-check.log 2>&1"
```

Do NOT write `kind: "validate-pending"` / capture a `workflow_run_id`. The advisor laptop session runs the command and mutates the entry. If the DQ write is gated, use the §4 harness-gap path with `TASK2_VALIDATE_PENDING.json`.

## §6 Expected output (return to advisor)

```
## Task 2 complete — v1-federation-inbound-a schema.rs

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/db_schema_file/src/schema.rs (+4 table! blocks, +2 Phase-6 block extensions, +2 joinable!, +4 allow_tables entries)
**Append-only confirmed:** no sibling-lane block reordered/reformatted
**validate-pending-laptop DQ:** #<id> raised (command: cargo-check.bat --workspace --features full)
**Next:** advisor laptop runs §15 cargo, mutates DQ #<id>; Cohort A barrier waits on all 5 tasks
```

Plus any DQ #N references.

## §7 Why this brief differs from the plan

It does not — Task 2's scope is exactly plan §10.2 + §13 Task 2. Additions: (a) §0 forbidden-window self-check, (b) §4 harness-gap escalation path (gated-write contingency only), (c) §5 explicit `validate-pending-laptop` shape per DQ #231. The schema.rs DSL is §10.2 verbatim — do not deviate.
