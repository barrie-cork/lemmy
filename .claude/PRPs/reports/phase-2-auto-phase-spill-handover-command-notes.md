# Phase 2 `/auto-phase` User-Scope Command Patch Notes

Status: Phase 2 implementation notes only. Do not apply automatically from this report.

Scope: Claude Code harness only. These notes are for `~/.claude/commands/auto-phase.md`, which is user-scope and intentionally not edited by this repo change.

## Context

Phase 2 adds schema-v3 fields to `.claude/PRPs/templates/auto-phase-state.template.json`:

- `spill_dir`
- `last_handover_path`
- `last_handover_at`

The repo reference contract is documented in `.claude/refs/auto-phase.md` and `.claude/rules/advisor-orchestrator.md`. Runtime files under `.claude/auto-state/` remain gitignored.

## Required insertion block: Phase 0.5 schema backfill

Insert in `~/.claude/commands/auto-phase.md` Phase 0.5 Step A, where older ledgers are schema-upgraded before reconciliation.

```text
Schema-v3 additive backfill: if schema_version < 3, set:
- spill_dir = ".claude/auto-state/<phase>.spill/" when absent or null
- last_handover_path = null when absent
- last_handover_at = null when absent
Then set schema_version = 3 and write the ledger before continuing Phase 0.5 reconciliation.
This is forward-only and must not reinterpret v1/v2 fields.
```

## Required insertion block: spill-guard wrapper

Insert near the top of Phase 1 as a standing rule applied to all tool results during the tick.

```text
Tool-output spill guard: when any tool result exceeds 16000 chars, write the full result to
auto_state.spill_dir/<stage>-<tool>-<UTC-iso>.txt (default .claude/auto-state/<phase>.spill/),
and retain only the first ~40 and last ~40 lines plus the spill path in context. For broad PMD
semantic queries, prefer running inside a subagent (per pmd-search-strategy.md) so the dump never
enters main context; spill is the fallback when a large result was not anticipated.
```

## Required insertion block: auto-handover refresh

Insert in Phase 1 immediately after the Phase 1 digest append/write step that was added in Phase 1.

```text
Auto-handover refresh: after the digest append, write/overwrite
.claude/PRPs/handovers/<phase>-auto-<UTC-date>.md with: current sub-phase, stage, last commit on
the relevant branch (SHA + subject), next concrete action (the digest's next_action_hypothesis,
marked "re-verify on resume"), DQ pending ids, and a one-line concurrent-activity note. Set
auto_state.last_handover_path + last_handover_at. Commit on governance-v0 with subject
"chore(advisor): auto-phase handover refresh <phase> <stage>". This is the floor that satisfies
the advisor-orchestrator.md §1 pre-compact handover discipline automatically.
```

## Non-goals

- Do not edit `.pi/**`.
- Do not edit `~/.claude/commands/compact-phase.md` for Phase 2; that is Phase 3.
- Do not make routing, gates, cadence, or catch-fire depend on narrative digest content.
- Do not include `stage_digests` content in Junior brief bodies; the advisor ledger stays wide and Junior context stays narrow.
