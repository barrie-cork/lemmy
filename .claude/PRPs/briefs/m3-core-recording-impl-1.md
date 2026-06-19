# Brief: m3-core-recording impl-1 (Task 1)

## §1 Role + dispatch line

`[role:impl-task] m3-core-recording-task1-roomeventpayload-recording-fields — see .claude/PRPs/briefs/m3-core-recording-impl-1.md`

## §2 Scope

**Task 1 of plan `.claude/PRPs/plans/m3-core-recording.plan.md` (lines 477–502).** Add FIVE optional recording-metadata fields to the binary's `RoomEventPayload` DTO so the `room_recording_uploaded` chain entry can carry the Success-Criteria schema. This is the ONE in-scope `crates/**` touch — a trivial DTO extension (gate-1 ratified, planner DQ `1ed255d85572-001` — 5 optional fields on the OUTBOUND struct, mirroring emergency-mute's `federated` add; the uploader/chair actor rides the existing top-level `actor_pseudonym`, NOT a payload field).

**Produces (exactly 1 file edit, ONE commit):**
1. `crates/api/api/src/governance/governance_log.rs` — add 5 `#[serde(default, skip_serializing_if = "Option::is_none")]` recording fields to `RoomEventPayload` (place AFTER the existing `federated` field), plus extend the existing `#[cfg(test)]` test with a `room_recording_uploaded` case.

The 5 fields (exact types per plan §10.1 / §4(a)):
```rust
#[serde(default, skip_serializing_if = "Option::is_none")] pub media_url: Option<String>,
#[serde(default, skip_serializing_if = "Option::is_none")] pub content_sha256: Option<String>,
#[serde(default, skip_serializing_if = "Option::is_none")] pub duration_s: Option<i64>,
#[serde(default, skip_serializing_if = "Option::is_none")] pub speakers: Option<Vec<String>>,
#[serde(default, skip_serializing_if = "Option::is_none")] pub attendance_count: Option<i32>,
```

**The plan's Task 1 (lines 477–502) is the contract — follow its IMPLEMENT / MIRROR / GOTCHA exactly.** Key GOTCHA (plan line 493): `skip_serializing_if = "Option::is_none"` is **load-bearing** — without it, existing room emissions gain `"media_url":null` (etc), changing the hashed JSON byte-for-byte and breaking the m2/m3 chain-entry hashes. There is **no non-test `RoomEventPayload { … }` constructor** in `crates/` (grep returns only the test literals), so the `Option` fields are non-breaking.

**The test case must assert BOTH directions (mirror the `federated` test exactly):**
- A `room_recording_uploaded` payload `{ case_id, lifecycle_stage: "room_recording_uploaded", media_url: Some(..), content_sha256: Some(..), duration_s: Some(..), speakers: Some(vec![..pseudonyms..]), attendance_count: Some(..), .. all chair-action + federated fields None }` → `serde_json::to_value` → assert each of the 5 fields is present + correct, AND the chair-action + `federated` fields are OMITTED.
- A payload with all-None recording fields → assert each recording key `.is_none()` (proves `skip_serializing_if` keeps existing room kinds byte-identical).

**ADR-015:** `speakers` is `Vec<String>` of **pseudonyms** — never `person_id`/username/MXID. The uploader/chair actor travels via the existing top-level `actor_pseudonym` arg, NOT a payload field. Do NOT add `chair_pseudonym`/`uploader` to the payload.

**Do NOT touch:** `ROOM_KINDS` or any `ENTRY_KIND_*` const (already shipped — `room_recording_uploaded` is in `ROOM_KINDS` at `:227`, registry count is 72 and MUST stay 72); any `services/bridge/**` file; any migration; any file outside the 1 above.

**Branch:** forks from `phase-m3-core-recording` (current tip — the brief must be reachable there at task-spawn).

## §3 Required reading

- `.claude/PRPs/plans/m3-core-recording.plan.md` — Task 1 (477–502), §10.1 (`RoomEventPayload` recording-fields extension pattern), §4(a) (the DTO-extension rationale + outbound-vs-inbound `RoomEventPayload` distinction), §19 (1) (the gate-1-ratified mechanism — outbound struct, 5 optional fields, mirror `federated`).
- `crates/api/api/src/governance/governance_log.rs` — the existing `RoomEventPayload` struct (MIRROR — add the 5 new fields after `federated`) + its `#[cfg(test)]` module (MIRROR for the new test case).
- **Sibling precedent (canonical-schema-first):** the m3-core-emergency-mute Task 1 commit that added the `federated: Option<bool>` field to this same struct — `git log governance-v0 --grep "RoomEventPayload.federated" --oneline | head -1`, then `git show <sha> -- crates/api/api/src/governance/governance_log.rs`. The 5 recording fields are the EXACT same shape (`#[serde(default, skip_serializing_if = "Option::is_none")] pub <name>: Option<...>`) + the same two-direction test-extension pattern. Mirror it verbatim (one field → five fields).
- **Lessons (mandatory, §2.4 file-class injection):**
  - `feedback_postgres_jsonb_canonicalization.md` — the `skip_serializing_if` byte-identical-JSON concern; the existing room-emission hashes MUST stay byte-identical.
  - `feedback_clippy_test_style.md` — the `#[cfg(test)]` test uses `LemmyResult<()>` with `?`, no `unwrap`/`expect` (clippy denies them under `-D warnings`).
  - `feedback_lemmy_error_no_std_error.md` — if the test returns a Result, use `LemmyResult<()>` (the canonical room-test shape), not `Result<(), Box<dyn Error>>`.

## §4 Constraints

- **ONE commit, 1 file** — `feat(governance): RoomEventPayload recording fields for room_recording_uploaded (task 1)`.
- **ADR-016 (metadata-only, load-bearing):** the 5 fields are recording **metadata** (`media_url`, `content_sha256`, `duration_s`, `speakers`, `attendance_count`). No content/MP4-bytes field is added — the bytes live in S3; only the `content_sha256` (one hash) + metadata reach the chain.
- **ADR-015 (pseudonyms):** `speakers` carries pseudonym strings only. This DTO does NOT add any actor field — the uploader/chair travels via the existing top-level `actor_pseudonym` arg. Do NOT add `chair_pseudonym`/`uploader` to the payload.
- **`skip_serializing_if` is load-bearing** — the serde test MUST assert that all-None recording fields OMIT the keys (proving existing room kinds stay byte-identical) AND that the populated case emits each field. Both directions, or the hash-stability proof is incomplete.
- **registry count stays 72** — do NOT add any `ENTRY_KIND_*` const; do NOT touch `ROOM_KINDS`. This phase EMITS `room_recording_uploaded` (already registered), never re-registers.
- **NO daemon cargo.** Write a `validate-pending-laptop` DQ with `commands: ["cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full\"", "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings\""]` and `e2e_filter: null` (the new test is a unit test, runs under check). Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo yourself (laptop advisor validates). Per `feedback_validate_pending_laptop_write_then_stop.md`. (No `-linux` DQ — Task 1 touches no bridge/Cargo.toml/migration/cfg code.)
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push on the worker branch, stop.
- **Handover:** deliver the HANDOVER block inline in your task output (do NOT write a `.claude/PRPs/handovers/` file — that path is blocked by the sensitive-file guard in the Junior worktree context).
- End the commit body with a `LESSON:` trailer if you find anything durable.

## §3a Handover from prior cohort

(none — first cohort)

## §5 HANDOVER (worker fills at task end — return inline in task output)

```yaml
HANDOVER:
  task: m3-core-recording-task1
  filesModified: [crates/api/api/src/governance/governance_log.rs]
  keyDecisions:
    - "5 recording fields (media_url/content_sha256/duration_s/speakers/attendance_count) added after federated; skip_serializing_if keeps existing room kinds byte-identical"
    - "test asserts both directions (populated case emits all 5; all-None omits all 5)"
    - "speakers carries pseudonyms (ADR-015); uploader rides top-level actor_pseudonym; registry count UNCHANGED at 72; ROOM_KINDS untouched"
  validate_dq: <composite-id of the validate-pending-laptop DQ>
  notes: "<anything Task 4 must know — esp. exact field positions + the test fn name; Task 4's bridge room_event_client::RoomEventPayload mirror must match these 5 fields>"
```
