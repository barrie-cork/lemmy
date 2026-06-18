# Brief: m3-core-stage-mode impl-1 (Task 1)

## §1 Role + dispatch line

`[role:impl-task] m3-core-stage-mode-task1-roomeventpayload-chair-fields — see .claude/PRPs/briefs/m3-core-stage-mode-impl-1.md`

## §2 Scope

**Task 1 of plan `.claude/PRPs/plans/m3-core-stage-mode.plan.md` (lines 426–455).** Extend the binary's `RoomEventPayload` DTO with 5 optional chair-action metadata fields, and add the optional `chair_pseudonym` governance→bridge channel on `CaseTransitionEvent`. This is the ONE in-scope `crates/**` touch — a trivial DTO extension (gate-1 ratified, DQ `da838b8fc109-002`).

**Produces (exactly 3 file edits, ONE commit — do NOT split):**
1. `crates/api/api/src/governance/governance_log.rs` — 5 optional fields on `RoomEventPayload` (`action`, `target_pseudonym`, `from_pseudonym`, `to_pseudonym`, `at`), each `#[serde(default, skip_serializing_if = "Option::is_none")]` + a `#[cfg(test)]` serde roundtrip test (per plan IMPLEMENT file 1 of 3).
2. `crates/api/api_common/src/governance.rs` — `#[serde(default)] pub chair_pseudonym: Option<String>` on `CaseTransitionEvent` (plan file 2 of 3).
3. `crates/api/api_utils/src/bridge_notify.rs` — `chair_pseudonym: None` in the `CaseTransitionEvent { … }` constructor at `:136-143` (plan file 3 of 3).

**The plan's Task 1 (lines 426–455) is the contract — follow its IMPLEMENT / MIRROR / GOTCHA exactly.** Key GOTCHA (plan line 446): `skip_serializing_if = "Option::is_none"` is load-bearing — without it, existing m2-core-hook room emissions gain `"action":null` etc., changing the hashed JSON. The 3 files MUST land in ONE commit (splitting leaves `lemmy_api_utils` non-compiling — `bridge_notify.rs:136` needs `chair_pseudonym: None`).

**Do NOT touch:** the entry-kind registry / `ENTRY_KIND_*` consts (already shipped); any `services/bridge/**` file; any migration; any file outside the 3 above.

**Branch:** forks from `phase-m3-core-stage-mode` (current tip `b2f4d5e9a`).

## §3 Required reading

- `.claude/PRPs/plans/m3-core-stage-mode.plan.md` — Task 1 (426–455), §10.1 (`RoomEventPayload` extension pattern, mirror `governance_log.rs:91-97`), §10.2 (`CaseTransitionEvent` field, mirror the `juror_pseudonyms` shape), §11 (files to change), §19 note (2) (the DTO-extension rationale).
- `crates/api/api/src/governance/governance_log.rs:91-97` — the existing `RoomEventPayload` struct (mirror) + its `#[cfg(test)]` module gating (mirror for the new test).
- `crates/api/api/src/governance/room_event_handler.rs:19-23` — the `RoomEventRequest` consumer that deserialises the new fields (verify back-compat).
- `crates/api/api_utils/src/bridge_notify.rs:136-143` — the `CaseTransitionEvent` constructor that needs `chair_pseudonym: None`.
- **Lessons (mandatory, §2.4 file-class injection):**
  - `feedback_postgres_jsonb_canonicalization.md` — the `skip_serializing_if` byte-identical-JSON concern (PG `::text` vs serde compact); the m2-room-emission hash must stay byte-identical.
  - `feedback_clippy_test_style.md` — the `#[cfg(test)]` roundtrip test uses `LemmyResult<()>` with `?`, no `unwrap`/`expect` (clippy denies them).

## §4 Constraints

- **ONE commit, 3 files** — `feat(governance): RoomEventPayload chair-action fields + CaseTransitionEvent.chair_pseudonym (task 1)`. Splitting is forbidden (non-compiling intermediate).
- **ADR-015 (load-bearing):** the 5 new fields carry **pseudonyms** (`target_pseudonym`, `from_pseudonym`, `to_pseudonym`) — never `person_id`/username. They are `Option<String>` DTO fields; the bridge populates them with pseudonym strings (Task 5). This task only declares them; the pin is enforced where they're populated (Task 5) and asserted in the §15.5 grep.
- **ADR-016 (metadata-only):** these are chair-action METADATA fields — `{action, target_pseudonym}` / `{from_pseudonym, to_pseudonym, at}`. No content/speech/video field is added.
- **`skip_serializing_if` on ALL 5 fields** — verify the serde test asserts that a `room_chair_override` payload OMITS the transfer fields and vice-versa (per plan IMPLEMENT file 1). This proves the m2 emissions stay byte-identical.
- **NO daemon cargo.** Write a `validate-pending-laptop` DQ with `commands: ["cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full\"", "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings\""]` and `e2e_filter: null` (the new test is a unit test, runs under check). Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo yourself (laptop advisor validates). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push on the worker branch, stop.
- End the commit body with a `LESSON:` trailer if you find anything durable.

## §3a Handover from prior cohort

(none — first cohort)
