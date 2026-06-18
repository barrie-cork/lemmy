---
name: Entry-kind phases — the Explore brief must ask for a runtime allowlist/validation array gating the consts
description: When planning a phase that registers new governance_log ENTRY_KIND_* consts, the Explore/recon brief MUST explicitly ask "is there a runtime allowlist or validation array (e.g. ROOM_KINDS) that gates these kinds and hardcodes a count?" A new const compiles clean but is runtime-rejected by emitters if a separate allowlist array isn't bumped in lockstep. A broad "find relevant code" prompt misses it; the targeted question catches it.
type: feedback
---

# Entry-kind phases — recon brief must hunt the runtime allowlist that gates the consts

When a Brehon sub-phase **registers new `governance_log` entry kinds** (a new
`pub const ENTRY_KIND_*: &str` in
`crates/db_schema/src/source/governance/governance_log.rs` + a shim re-export),
the planning recon must do more than confirm the const block and the shim. It
must specifically hunt for a **separate runtime allowlist / validation array**
that enumerates the permitted kinds and **hardcodes a count** — because adding a
const there is necessary but not sufficient: a new kind compiles clean but is
**runtime-rejected** by any emitter that checks the kind against that array
until the array is bumped in lockstep.

The canonical instance: `ROOM_KINDS` — a `#[cfg(feature = "full")]` runtime gate
in the `api` shim file that hardcodes "the 10 allowed room kinds" and rejects
anything else (it backs `append_room_event`'s ADR-008 integrity check). M3's 3
new chair/mute kinds would have compiled and passed the collision invariant
(`A == B` const-count vs literal-count) yet been runtime-rejected by the
Phase 3/4 bridge emitters if `ROOM_KINDS` hadn't been bumped 10→13 at the same
time. The PRD's own "crate(s) affected" list **omitted** the array — it named
the const block and the shim only. The recon Explore agent caught it because the
brief asked, verbatim, "is there any exhaustiveness mechanism over these consts."
A broad "find the relevant code" prompt would not have surfaced a second array in
a different file whose only link to the consts is a runtime string comparison.

## Why this matters

The const-count collision invariant (registry §"Adding a new kind requires"
step 4: `A == B`, no duplicate literals) proves the consts are internally
consistent — it does **not** prove every runtime allowlist that references those
consts agrees on the set. The allowlist is a *second* source of truth, in a
*different* file, coupled only by string value. Nothing in the compile path links
them: drop a kind from the const block and the build fails; add a kind to the
const block but not the allowlist and the build is green — the divergence only
surfaces at emit time, in the phase that *uses* the kind, which is often a
*later* sub-phase (registration and emission are deliberately split in M-track:
consts ship one phase, emitters ship the next). So the failure lands one or two
phases downstream of the phase that introduced it, where it's expensive to trace
back. The cheap catch is at recon time, with one targeted question.

## How to apply (planning recon, at brief-author time)

When the phase being planned registers new `ENTRY_KIND_*` consts (or any
analogous enum-of-string-constants the codebase validates at runtime):

1. The Explore/recon brief MUST include the literal question: **"Is there a
   runtime allowlist or validation array (e.g. `ROOM_KINDS`) that enumerates the
   permitted kinds and hardcodes a count or length? If so, name the file:line and
   the count, and confirm whether this phase must bump it."** Do not rely on the
   PRD's "crate(s) affected" list — that list under-counted the edit at least once
   (M3 entry-kinds; PRD omitted `ROOM_KINDS`).
2. If the recon finds such an array, the plan's §13 must include the array bump as
   an explicit task or step (not folded silently into the const task), and §4 must
   carry a watchpoint citing the array's file:line + old→new count (per
   `feedback_advisor_watchpoint_specificity.md` — name the array, not "watch the
   allowlist").
3. If the recon finds *no* such array, record that explicitly in the plan ("no
   runtime allowlist gates these kinds; const-block + shim + collision-invariant is
   the full set") so the absence is a verified finding, not an unchecked gap.

This is structural, not one-off: M3 has more entry-kind sub-phases coming, and the
v0/v1 governance-log surface uses the same const-plus-runtime-allowlist shape
elsewhere. The question costs one line in the recon brief; the miss costs a
runtime rejection discovered in a downstream emitter phase.

## Generalises to

Any "register a value in an enum of string constants, validated at runtime against
a separate hardcoded set" pattern — config-key seeds with an `EXPECTED_SEED_COUNT`
parity counter (registry §note flags this as a *different* domain but the
same shape), AP activity-type registries, feature-flag allowlists. The general
trap: a compile-checked declaration site and a runtime-checked allowlist site that
share only a string value, coupled by no type. The recon must enumerate **both**
sites; the plan must edit both in lockstep; the watchpoint must name the second
site so the impl-task can grep it.

## See also

- `.claude/rules/governance-log-entry-kind-registry.md` — the `A == B` collision
  invariant (proves const-internal consistency, NOT allowlist agreement) + the
  `ROOM_KINDS` runtime gate this lesson is about.
- `feedback_advisor_watchpoint_specificity.md` — the watchpoint must name the
  array file:line, not "watch the allowlist".
- `.claude/PRPs/reports/session-retro-2026-06-18-m3-entry-kinds-plan.md` — the
  origin retro (Change #1); the recon that caught `ROOM_KINDS` 10→13.
- `feedback_read_canonical_before_writing_spec.md` — same read-the-real-code-first
  family.
