# Brief: m3-core-entry-kinds Task 1+2 — 3 chair/mute consts + api shim + ROOM_KINDS extend

## 1. Role + dispatch

`[role:impl-task] m3-core-entry-kinds-consts-shim — see .claude/PRPs/briefs/m3-core-entry-kinds-impl-1.md`

Pre-Shape-G plan. Workers write `validate-pending-laptop` DQ and STOP; do NOT run workspace cargo on the daemon.

> **Consolidation note:** the plan lists Task 1 (db_schema consts) and Task 2 (shim +
> `ROOM_KINDS`) separately. They are bundled here into ONE impl task because a const
> added to db_schema without its shim re-export leaves the registry shim-parity invariant
> failing, and `--features full` needs `ROOM_KINDS` extended for the gate to compile-accept
> the new kinds. This matches the m2-late-1 task-3 precedent (db_schema const + shim in one
> task). Same files, same edits, same single validate cycle.

## 2. Scope

**Produce:**

1. **Modify** `crates/db_schema/src/source/governance/governance_log.rs` — after the M2 room-kinds block (after `ENTRY_KIND_ROOM_LIFECYCLE_EVENT` on line 248, BEFORE the blank line + `// m2-late-1 additions` comment on line 250), insert a new section comment + 3 consts:
   ```rust

   // M3 town-hall chair/mute kinds (3) — zero-migration; entry_kind is TEXT.
   // Call sites land bridge-side in M3 phase 3 (_CHAIR_TRANSFERRED / _CHAIR_OVERRIDE)
   // and phase 4 (_MUTE_ALL) via append_room_event. Pre-landed-const exemption.
   pub const ENTRY_KIND_ROOM_CHAIR_TRANSFERRED: &str = "room_chair_transferred";
   pub const ENTRY_KIND_ROOM_CHAIR_OVERRIDE: &str = "room_chair_override";
   pub const ENTRY_KIND_ROOM_MUTE_ALL: &str = "room_mute_all";
   ```

2. **Modify** `crates/api/api/src/governance/governance_log.rs` — TWO edits in this one file:

   **(a) `pub use` shim** (the block at lines 39-72) — insert the 3 consts at their alphabetical positions:
   - `ENTRY_KIND_ROOM_CHAIR_OVERRIDE` and `ENTRY_KIND_ROOM_CHAIR_TRANSFERRED` go AFTER `ENTRY_KIND_ROOM_BRIDGE_ERROR` and BEFORE `ENTRY_KIND_ROOM_CREATED` (alphabetical: `BRIDGE_ERROR` < `CHAIR_OVERRIDE` < `CHAIR_TRANSFERRED` < `CREATED`). Line 60 currently reads `ENTRY_KIND_ROLLUP_RECOMPUTED, ENTRY_KIND_ROOM_ARCHIVED, ENTRY_KIND_ROOM_BRIDGE_ERROR,` and line 61 begins `ENTRY_KIND_ROOM_CREATED, ...`.
   - `ENTRY_KIND_ROOM_MUTE_ALL` goes AFTER `ENTRY_KIND_ROOM_MEMBER_REMOVED` and BEFORE `ENTRY_KIND_ROOM_RECORDING_UPLOADED` (alphabetical: `MEMBER_REMOVED` < `MUTE_ALL` < `RECORDING_UPLOADED`). Line 62-63 currently: `ENTRY_KIND_ROOM_MEMBER_ADDED, ENTRY_KIND_ROOM_MEMBER_REMOVED,` then `ENTRY_KIND_ROOM_RECORDING_UPLOADED, ENTRY_KIND_ROOM_TRANSCRIPT_READY,`.

   **(b) `ROOM_KINDS` array** (lines 100-111) — add all 3 consts in sorted order matching the shim:
   - `ENTRY_KIND_ROOM_CHAIR_OVERRIDE` + `ENTRY_KIND_ROOM_CHAIR_TRANSFERRED` after `ENTRY_KIND_ROOM_BRIDGE_ERROR` (line 102), before `ENTRY_KIND_ROOM_CREATED` (line 103).
   - `ENTRY_KIND_ROOM_MUTE_ALL` after `ENTRY_KIND_ROOM_MEMBER_REMOVED` (line 108), before `ENTRY_KIND_ROOM_RECORDING_UPLOADED` (line 109).
   - Bump the doc comment on line 97 from `The 10 allowed` → `The 13 allowed`.
   - Bump the doc comment on line 115 from `the 10 \`ENTRY_KIND_ROOM_*\` kinds` → `the 13 \`ENTRY_KIND_ROOM_*\` kinds`.

Then write the `validate-pending-laptop` DQ entry and STOP.

**Explicit boundaries (do NOT touch):**
- Do NOT edit `.claude/rules/governance-log-entry-kind-registry.md` — advisor meta-work (the harness DQ #235 blocks Junior writes to `.claude/rules/**` anyway). Leave a NOTE in your impl commit body.
- Do NOT edit any other file under `crates/db_schema/**` or `crates/api/**`.
- Do NOT edit `crates/server/**`, `services/bridge/**`, or any e2e file.
- Do NOT add call sites / emitters — those land bridge-side in M3 phase 3/4 (pre-landed-const exemption).
- Do NOT run `cargo check --workspace` on the daemon (NO-CARGO-ON-ELITEDESK).

## 3. Required reading

**Before your first edit, Read these files:**

- `crates/db_schema/src/source/governance/governance_log.rs:236-261` — the M2 room-kinds block (the MIRROR — same const syntax, same section-comment style) + the insertion point (after line 248, before line 250's `// m2-late-1 additions`).
- `crates/api/api/src/governance/governance_log.rs:39-72` — the `pub use` shim block (alphabetical insertion).
- `crates/api/api/src/governance/governance_log.rs:97-132` — the `ROOM_KINDS` array + `append_room_event` gate + the two doc-comment counts to bump (10→13).
- `.claude/rules/governance-log-entry-kind-registry.md` — READ ONLY. The M3 registry section + count bump (69→72) is advisor meta-work. Leave the NOTE in your commit body.
- `feedback_validate_pending_laptop_write_then_stop.md` — write the DQ entry and STOP; no daemon cargo.

## 3a. Handover from prior cohort

(none — first cohort.)

## 4. Constraints

- **MIRROR discipline:** the 3 db_schema consts replicate the M2 room-kinds block syntax verbatim — `pub const ENTRY_KIND_<NAME>: &str = "<snake_case>";`, section comment above, NOT alphabetized within the file (grouped by milestone; append as a new dated section).
- **`&str` values are snake_case = const-name suffix lowercased:** `_CHAIR_TRANSFERRED` → `"room_chair_transferred"`, `_CHAIR_OVERRIDE` → `"room_chair_override"`, `_MUTE_ALL` → `"room_mute_all"`. A literal colliding with any existing value fails the acceptance invariant — these three are new (no prior chair/mute kind), so no collision.
- **Two-part edit in `api/governance_log.rs` is MANDATORY:** both the `pub use` block AND the `ROOM_KINDS` array. Missing `ROOM_KINDS` would compile fine but leave `append_room_event` rejecting the new kinds at runtime — the M3 phase-3/4 emitters would fail. `ROOM_KINDS` is `#[cfg(feature = "full")]`, so `--features full` is the only way to compile-check it.
- **`ROOM_KINDS` final length MUST be 13** and both doc comments must read "13". The count parity is enforced by the advisor's Task-3 invariant checks, not the compiler.
- **R7 / NO-CARGO-ON-ELITEDESK:** Do NOT run `./scripts/brehon/cargo-check.sh` or any `cargo` command on the daemon. After committing, write the `validate-pending-laptop` DQ entry then STOP immediately.
- **validate-pending-laptop DQ:** write it with `commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]`, `phase_task: 1`, `branch: phase-m3-core-entry-kinds`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id (or `dq-v3-append-fragment.sh`).
- **Registry NOTE in commit body:** `NOTE: advisor must add M3 section + bump count 69→72 in .claude/rules/governance-log-entry-kind-registry.md (advisor meta-work, harness DQ #235 blocks Junior).`
- **Impl commit subject:** `feat(db_schema,api): add 3 M3 chair/mute entry-kind consts + shim + ROOM_KINDS 10→13 (task 1+2)`.
- **DQ commit subject:** `chore(decision-queue): impl raised validate-pending-laptop for m3-core-entry-kinds task-1`.
- One impl commit + one DQ commit, then STOP. **Mid-task push:** after EACH commit, `git push origin phase-m3-core-entry-kinds` immediately so the laptop advisor sees the DQ entry.
- **DQ mid-task discipline:** if a blocker arises (e.g. line numbers drifted, an anchor isn't unique), write a `kind: "blocker"` DQ entry, commit + push, and stop — do NOT guess.

**Mandatory lesson fired (file-class table match):**
- `feedback_validate_pending_laptop_write_then_stop.md` — pre-Shape-G plan, `crates/db_schema/**` + `crates/api/**` edits → worker writes the validate-pending-laptop DQ and STOPS.

## 5. Success signals

- 3 new consts present in `crates/db_schema/src/source/governance/governance_log.rs` with the exact snake_case values.
- All 3 re-exported in the `api` shim `pub use` block (alphabetical).
- `ROOM_KINDS` array has 13 entries; both doc comments read "13".
- `validate-pending-laptop` DQ entry written + pushed; impl commit + DQ commit both pushed to `phase-m3-core-entry-kinds`.

## HANDOVER

```yaml
HANDOVER:
  task: m3-core-entry-kinds-consts-shim
  filesModified:
    - crates/db_schema/src/source/governance/governance_log.rs
    - crates/api/api/src/governance/governance_log.rs
  keyDecisions:
    - "3 M3 chair/mute consts: room_chair_transferred / room_chair_override / room_mute_all"
    - "ROOM_KINDS extended 10 → 13; doc comments bumped 10→13 (gate accepts the new kinds)"
    - "Tasks 1+2 consolidated into one impl task (db_schema const + shim must land together for parity)"
    - "Registry doc + count bump 69→72 is advisor meta-work (left a NOTE in commit body)"
  notes: "Pre-landed consts; no emitters (M3 phase 3/4 bridge-side). validate-pending-laptop DQ raised for laptop cargo --features full."
```
