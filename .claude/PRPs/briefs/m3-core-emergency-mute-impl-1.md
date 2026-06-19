# Brief: m3-core-emergency-mute impl-1 (Task 1)

## §1 Role + dispatch line

`[role:impl-task] m3-core-emergency-mute-task1-roomeventpayload-federated — see .claude/PRPs/briefs/m3-core-emergency-mute-impl-1.md`

## §2 Scope

**Task 1 of plan `.claude/PRPs/plans/m3-core-emergency-mute.plan.md` (lines 375–400).** Add ONE optional `federated: Option<bool>` field to the binary's `RoomEventPayload` DTO so the `room_mute_all` chain entry can carry the Success-Criteria `federated: true|false` metadata. This is the ONE in-scope `crates/**` touch — a trivial DTO extension (gate-1 ratified, planner DQ `78c6ad4bfb41-002` — payload carries `federated` only; `chair_pseudonym` is the existing top-level `actor_pseudonym`, NOT a new field).

**Produces (exactly 1 file edit, ONE commit):**
1. `crates/api/api/src/governance/governance_log.rs` — add `#[serde(default, skip_serializing_if = "Option::is_none")] pub federated: Option<bool>` to `RoomEventPayload` (place AFTER the existing chair-action fields), plus extend the existing `#[cfg(test)]` test (`:116-157`) with a `room_mute_all` case.

**The plan's Task 1 (lines 375–400) is the contract — follow its IMPLEMENT / MIRROR / GOTCHA exactly.** Key GOTCHA (plan line 391): `skip_serializing_if = "Option::is_none"` is **load-bearing** — without it, existing room emissions gain `"federated":null`, changing the hashed JSON byte-for-byte and breaking the m2/m3 chain-entry hashes. There is **no non-test `RoomEventPayload { … }` constructor** in `crates/` (grep returns only the test literals), so the `Option` field is non-breaking.

**The test case must assert BOTH directions:**
- A `room_mute_all` payload `{ case_id, lifecycle_stage: "room_mute_all", federated: Some(true), .. all chair-action fields None }` → `serde_json::to_value` → assert `v.get("federated").is_some()` AND `v["federated"] == true` AND the chair-action fields are OMITTED.
- A payload with `federated: None` → assert `v.get("federated").is_none()` (proves `skip_serializing_if` keeps existing room kinds byte-identical).

**Do NOT touch:** `ROOM_KINDS` or any `ENTRY_KIND_*` const (already shipped — `room_mute_all` is in `ROOM_KINDS` at `:174`, registry count is 72 and MUST stay 72); any `services/bridge/**` file; any migration; any file outside the 1 above.

**Branch:** forks from `phase-m3-core-emergency-mute` (current tip `8ffcef9ee`).

## §3 Required reading

- `.claude/PRPs/plans/m3-core-emergency-mute.plan.md` — Task 1 (375–400), §10.1 (`RoomEventPayload.federated` extension pattern), §4(a) (the DTO-extension rationale + why `chair_pseudonym` is NOT a payload field), §19 (1)+(2) (the gate-1-ratified mechanism + actor_pseudonym carries the chair).
- `crates/api/api/src/governance/governance_log.rs:91-158` — the existing `RoomEventPayload` struct (MIRROR — add the new field after the chair fields) + its `#[cfg(test)]` module (MIRROR for the new test case).
- **Sibling precedent (canonical-schema-first):** the m3-core-stage-mode Task 1 commit on `governance-v0` that added the 5 chair-action optional fields to this same struct — `git log governance-v0 --grep "RoomEventPayload chair-action" --oneline | head -1`, then `git show <sha> -- crates/api/api/src/governance/governance_log.rs`. The `federated` field is the EXACT same shape (`#[serde(default, skip_serializing_if = "Option::is_none")] pub <name>: Option<...>`) + the same test-extension pattern. Mirror it verbatim.
- **Lessons (mandatory, §2.4 file-class injection):**
  - `feedback_postgres_jsonb_canonicalization.md` — the `skip_serializing_if` byte-identical-JSON concern (PG `::text` vs serde compact); the existing room-emission hashes MUST stay byte-identical.
  - `feedback_clippy_test_style.md` — the `#[cfg(test)]` test uses `LemmyResult<()>` with `?`, no `unwrap`/`expect` (clippy denies them under `-D warnings`).

## §4 Constraints

- **ONE commit, 1 file** — `feat(governance): RoomEventPayload.federated field for room_mute_all (task 1)`.
- **ADR-016 (metadata-only, load-bearing):** `federated: Option<bool>` is a **metadata** flag (was the mute federation-wide?). No content/speech/video field is added. The `room_mute_all` chain entry carries `{ actor_pseudonym (= chair), federated }` — metadata only, per ADR-016 (content never hashed).
- **ADR-015 (pseudonyms):** this DTO does NOT add any actor field — the chair identity travels via the existing top-level `actor_pseudonym` arg (a pseudonym string), NOT a new payload field. Do not add `chair_pseudonym` to the payload.
- **`skip_serializing_if` is load-bearing** — the serde test MUST assert that `federated: None` OMITS the key (proving existing room kinds stay byte-identical) AND that `federated: Some(true)` emits `true`. Both directions, or the hash-stability proof is incomplete.
- **registry count stays 72** — do NOT add any `ENTRY_KIND_*` const; do NOT touch `ROOM_KINDS`. This phase EMITS `room_mute_all` (already registered), never re-registers.
- **NO daemon cargo.** Write a `validate-pending-laptop` DQ with `commands: ["cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full\"", "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings\""]` and `e2e_filter: null` (the new test is a unit test, runs under check). Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo yourself (laptop advisor validates). Per `feedback_validate_pending_laptop_write_then_stop.md`. (No `-linux` DQ — Task 1 touches no bridge/Cargo.toml/migration/cfg code.)
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push on the worker branch, stop.
- **Handover:** deliver the HANDOVER block inline in your task output (do NOT write a `.claude/PRPs/handovers/` file — that path is blocked by the sensitive-file guard in the Junior worktree context per PMD #857/#867).
- End the commit body with a `LESSON:` trailer if you find anything durable.

## §3a Handover from prior cohort

(none — first cohort)

## §5 HANDOVER (worker fills at task end — return inline in task output)

```yaml
HANDOVER:
  task: m3-core-emergency-mute-task1
  filesModified: [crates/api/api/src/governance/governance_log.rs]
  keyDecisions:
    - "federated: Option<bool> added after chair-action fields; skip_serializing_if keeps existing room kinds byte-identical"
    - "test asserts both directions (Some(true) emits true; None omits key)"
    - "registry count UNCHANGED at 72; ROOM_KINDS untouched"
  validate_dq: <composite-id of the validate-pending-laptop DQ>
  notes: "<anything the next cohort/Task 3 must know — esp. exact field position + the test fn name>"
```
