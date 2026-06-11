# Phase 3 `/auto-phase` Compact-Ledger Handoff Plan

Status: planning only. Do not apply in this change.

Scope: Claude Code harness only. No Pi Coding changes. Do not edit `.pi/**`. Do not edit user-scope `~/.claude/commands/*` directly; apply the patch notes below manually or via a separate explicit user-scope command update.

## Goal

Wire `/compact` and Phase 0.5 resume logic to the durable `/auto-phase` ledger so active-thread reconstruction starts from the surviving state file, not from disappearing conversation context.

Phase 3 depends on Phase 1 and Phase 2 fields:

- `stage_digests[-1]` for the latest compact stage/outcome/next-action narrative.
- `last_handover_path` for a self-contained handover refreshed at the last stage transition.
- `last_handover_at` to show freshness.
- `spill_dir` only as a pointer source when recent work spilled large tool output.

## Invariants to preserve

- `next_action_hypothesis` is re-verify-only. It must never be treated as permission to act before live TaskList/DQ/PR/branch state is checked.
- Narrative digests must not decide gates, routing, cadence, catch-fire, or merge behavior.
- Advisor context remains durable and wide: phase, stage, cohort, gate history, digest ring, spill references, handover path, verification status, and next-action hypotheses.
- Junior context remains narrow and brief-scoped: one task brief, explicit scope, write boundaries, validation expectations, and relevant error history only.
- Runtime state under `.claude/auto-state/` remains gitignored.
- Additive schema/backfill only; no second harness.

## Repo-tracked doc changes to apply

1. `.claude/refs/compact-prompt-approach.md`
   - Add an optimisation-history entry for digest-first `/compact` wiring.
   - Note that priority 1 should cite `.claude/auto-state/<phase>.json` and `last_handover_path` for `/auto-phase` runs.
   - Explicitly state that `next_action_hypothesis` is a hypothesis to re-verify on resume.

2. `.claude/refs/auto-phase.md`
   - Extend Phase 0.5 resume semantics so Step E builds the COMPACT resume report primarily from `stage_digests[-3:]` and `last_handover_path`.
   - Preserve the existing hard invariant: Phase 0.5 is read-only until the user replies `continue`.
   - Clarify that the digest is additional durable signal and not a new gate or router.

## User-scope command patch notes

### `~/.claude/commands/compact-phase.md`

Insert into priority 1, the active-thread instruction:

```text
If the active thread is a /auto-phase run (a .claude/auto-state/<phase>.json ledger exists for the
phase named in the session), treat the ledger as the authoritative active-thread source: cite
stage_digests[-1] (stage, outcome, next_action_hypothesis) and last_handover_path verbatim,
rather than reconstructing branch/stage/in-flight state from conversation. The ledger survives
compaction by design; the conversation does not. Keep the next-action marked a hypothesis to
re-verify on resume. For non-/auto-phase sessions, use the existing priority-1 wording unchanged.
```

### `~/.claude/commands/auto-phase.md`

Insert in Phase 0.5 Step E, where the COMPACT resume report is built:

```text
Build the COMPACT resume report (<=15 lines, <=500 tokens) primarily from the ledger, not from
conversation: read stage_digests[-3:] for the recent narrative and last_handover_path for the
self-contained handover. Surface the latest next_action_hypothesis as a HYPOTHESIS to re-verify
against live TaskList/DQ/PR state before any action (per feedback_thin_wakeup_prompts_verify_
live_state.md). Keep lazy-load + Steps-B-D subagent-delegation discipline unchanged; the digests
are additional durable signal, not a new preload.
```

## Suggested validation for Phase 3 implementation

- Confirm `/compact` priority-1 text mentions `stage_digests[-1]`, `last_handover_path`, and re-verification of `next_action_hypothesis`.
- Confirm `/auto-phase` Phase 0.5 Step E mentions `stage_digests[-3:]`, `last_handover_path`, and read-only-until-continue behavior.
- Confirm `.claude/refs/compact-prompt-approach.md` has a new optimisation-history entry.
- Confirm `.claude/refs/auto-phase.md` says digest/handover are resume sources only and not routing or gate inputs.
- Confirm no `.pi/**` paths are changed or staged.
- Confirm `.claude/auto-state/` is still covered by `.gitignore`.

## Suggested commit message for Phase 3

```text
chore(auto-phase): wire compact resume to durable ledger
```
