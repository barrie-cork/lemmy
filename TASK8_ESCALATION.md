# Task 8 Escalation — e2e.rs:7451 callsite will break (DQ #253)

## What was attempted

Task 8 added 6 new `Option<_>` fields to `FederationAttestationInsertForm` (per plan §10.3 + brief §2). The existing callsites in `inbox.rs` (lines 138, 213) were fixed by adding `..Default::default()` spread — this is in Task 8's scope.

## What fails

The callsite at `crates/server/tests/e2e.rs:7451` initializes `FederationAttestationInsertForm` with 5 fields explicitly, WITHOUT `..Default::default()`:

```rust
.values(&FederationAttestationInsertForm {
  actor_url: "https://test.invalid/u/seed-actor".to_string(),
  subject_url: "https://test.invalid/u/seed-subject".to_string(),
  attestation_type: AttestationType::TrustedReporter,
  valid_until: Some(future),
  signature: "seed-sig".to_string(),
})
```

After Task 8 adds 6 new fields, Rust requires either all fields specified OR the `..Default::default()` suffix. This will fail compilation with:
`missing fields 'source_instance', 'received_at', 'peer_trust_level_at_receipt', 'admin_reviewed_at', 'admin_action', 'dismissal_rationale' in initializer of FederationAttestationInsertForm`

**Result:** validate-pending-laptop command 3 (`--test e2e --no-run`) will fail until e2e.rs:7451 is fixed.

## Why the brief's assumption was incorrect

The brief states: "the §10.3 fields should be Option-able precisely so e2e.rs:7451 stays compiling without a Task-8 e2e.rs edit". This is incorrect Rust behavior. `Option<T>` type does NOT make a struct field optional in struct-literal syntax — see lesson `.claude/lessons/feedback_insertform_default_propagation.md` which explicitly documents this exact failure mode. The brief's prohibition on editing e2e.rs from Task 8 was therefore based on a false premise.

## What is needed

Advisor decision on one of:

- **option-a:** Authorize Task 8 to add `..Default::default()` to `crates/server/tests/e2e.rs:7451` — 1-line mechanical change. All 6 new InsertForm fields are `Option<_>` with `#[derive(Default)]`, so `..Default::default()` compiles cleanly. The `feedback_junior_worker_e2e_edit_hang` hazard is for LARGE edits (this is 1 line); cross-task file collision is not a risk since tasks are serial. This is the approach the lesson (`feedback_insertform_default_propagation.md`) prescribes.
- **option-b:** Remove `--test e2e --no-run` from Task 8's validate-pending-laptop commands (run only cargo-check + cargo-clippy for Task 8). Move the e2e compile validation to Task 9's validate scope. Task 9 will fix e2e.rs:7451 as part of its own changes.

## Suggested next steps

Prefer option-a: the lesson explicitly requires adding `..Default::default()` to ALL caller sites, and the Task 8 prohibition was a planning error. Option-a keeps Task 8 self-contained and validates the full compile chain immediately.

If option-b is chosen: run TASK8_VALIDATE_PENDING.json commands 1+2 only (skip command 3), then dispatch Task 9 with an explicit note to add `..Default::default()` to e2e.rs:7451 as well as its normal §10.8 scope.

## DQ entry to transcribe (DQ #253)

```json
{
  "id": 253,
  "from": "impl",
  "kind": "blocker",
  "timestamp": "2026-05-18T14:35:00Z",
  "question": "Task 8 adds 6 new Option<_> fields to FederationAttestationInsertForm. e2e.rs:7451 initializes it without ..Default::default() — will fail compilation. Brief prohibits editing e2e.rs from Task 8. Options: (a) authorize Task 8 fix of e2e.rs:7451 (1 line), (b) drop --test e2e --no-run from Task 8 validate and move to Task 9.",
  "options": [
    "option-a: authorize adding ..Default::default() to e2e.rs:7451 in Task 8 re-run",
    "option-b: drop --test e2e --no-run from Task 8 validate; Task 9 handles it"
  ],
  "context": "feedback_insertform_default_propagation.md lesson documents this exact failure: Option<T> does not make struct-literal fields optional. Brief said 'Option-able fields won't break e2e.rs:7451' — this is incorrect Rust. inbox.rs callsites (2) fixed with ..Default::default() per lesson; e2e.rs:7451 was out-of-scope per brief.",
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```
