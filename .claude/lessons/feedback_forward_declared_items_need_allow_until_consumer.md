---
name: Forward-declared items + struct-field adds need full enumeration in the brief (fn + trait + struct, every literal)
description: When a task creates forward-declared items whose consumer lands later, EACH (free fn, trait method, struct) needs #[allow(dead_code)] until the consumer lands — not just the struct. When a task adds struct fields, EVERY literal constructor (production AND #[cfg(test)]) needs them. cargo test passing does NOT prove clippy -D warnings clean — #[cfg(test)] code counts a forward-declared item as used while the non-test bin-target build flags it dead-code. The impl-task brief must enumerate all of them or a fix-impl cycle results.
type: feedback
originSessionId: 18cf936c-2d67-4b17-bd15-f7fd4d96b9a7
---

# Forward-declared items + struct-field adds need full enumeration in the brief

**Rule (two triggers, both mechanical — pattern matches → the brief must satisfy the obligation):**

**Trigger A — the task ADDS or RENAMES struct fields.** Every existing literal
constructor of that struct (production AND `#[cfg(test)]`) needs the new field(s)
in the SAME commit, or it's an `E0063 missing-field` compile failure. The brief
MUST require the worker to `rg "<StructName>\s*\{" <path>/` to enumerate ALL
literals FIRST, then add the field to each. DoD: edited-literal count == `rg`
count. (This generalizes [[feedback_insertform_default_propagation]] beyond the
`*InsertForm`/derive(Default) case to any struct.)

**Trigger B — the task CREATES forward-declared items** (a `pub fn`, a `pub trait`
with methods, OR a `pub struct`/impl) whose only non-test consumer lands in a
LATER task. EACH such item that is unused in the non-test build needs
`#[allow(dead_code)]` until its consumer lands — **enumerate by kind, not just
the struct.** A free fn is a separate dead-code site from a trait, which is
separate from a struct.

**The masking trap (the reason this keeps recurring):** `cargo test` passing
does NOT prove `clippy -D warnings` clean. `#[cfg(test)]` code counts a
forward-declared item as *used*, so the test build is green while the non-test
`clippy` bin-target build flags it dead-code. The author who validates only with
`cargo test` (or trusts a worker's "tests pass") ships the gap; the
`validate-pending-laptop-linux` clippy `-D warnings` gate is what catches it —
one full fix-impl cycle later.

**Why:** m3-core-recording (M3 Phase 5) burned THREE fix-impl cycles, all this
same class:
- fix-impl-1 (`551bea68a`): Task 2 added 3 forward-declared `recording_config`
  helpers to `bridge_room.rs`; the brief mirrored a sibling helper pattern but
  those siblings WERE used (stage-mode) while these weren't until Task 4/5 →
  clippy dead_code. Needed `#[allow(dead_code)]`.
- fix-impl-1b (`d4bfd5968`): Task 2 added 4 `s3_*` fields to `BridgeConfig` but
  didn't update the `make_config` test-only literal in `sanction_handler.rs` →
  `E0063`. (Trigger A miss on a TEST literal — `cargo check --bin` is green;
  only `cargo test` compiles the fixture.)
- fix-impl-3 (`8a5431b8a`): Task 3 created `recording.rs` with
  `compute_content_sha256` (free fn) + `RecordingSink` trait + `LiveSink`
  struct; the brief put `#[allow(dead_code)]` on the *struct* only → the free fn
  and the trait were still dead-code under clippy. `cargo test` passed (the 2
  unit tests use them); the non-test clippy build did not. (Trigger B miss —
  enumerated the struct, not the fn + trait.)

Each cost ~one validate cycle. All three were preventable at brief-author time
with full enumeration.

**How to apply:**

- **Brief author (advisor):** the impl-task brief template §2.5 "Forward-declared-item
  + struct-field-propagation gate" fires on either trigger. For Trigger A: require
  the `rg "<StructName>\s*\{"` enumeration + a DoD count-match line. For Trigger
  B: enumerate every forward-declared `pub fn` / `pub trait` method / `pub struct`
  by kind and require `#[allow(dead_code)]` on each (mirror an existing in-file
  `#[allow]` verbatim), with a comment naming the consumer-task. Always include
  the masking-trap note in §4 (`cargo test` green ≠ clippy green).
- **When THIS task is the consumer** that wires a forward-declared item into the
  non-test build: remove its `#[allow(dead_code)]` — but ONLY for items genuinely
  now-used in the non-test build; LEAVE `#[allow]` on still-unused items. When
  unsure, LEAVE it: a stale `#[allow]` on a now-used item is harmless (not a
  clippy error); a still-unused item missing `#[allow]` IS an error. Err toward
  leaving — the clippy gate catches over-removal harmlessly.
- **Validation:** the `validate-pending-laptop-linux` DQ MUST run `clippy
  --no-deps -- -D warnings`, not just `check` + `test`. The clippy step is the
  only one of the three that catches the dead-code class on a forward-declared
  item. (See [[feedback_clippy_rerun_after_fix.md]] for the sibling "re-run
  clippy after a dead-code fix" discipline.)

Related: [[feedback_insertform_default_propagation]] (Trigger A, InsertForm-specific
origin), [[feedback_clippy_rerun_after_fix]] (dead-code fixes have sibling
findings), [[feedback_fix_impl_enumerate_all_callsites]] (E0063 callsite
enumeration for fix-impl briefs).
