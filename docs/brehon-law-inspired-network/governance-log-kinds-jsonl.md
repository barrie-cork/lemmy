# Governance-log JSONL kinds (hook-emitted observability)

## 1. Scope

Harness-internal observability events emitted by
`.claude/hooks/*.sh` as JSONL records to sidecar files under
`.claude/governance-log/`. **NOT the v1 product governance-
log** — that lives in the PG `governance_log` table per
`docs/brehon-law-inspired-network/03-architecture.md` §6
"The append-only governance log (interface)" and is
referenced (in the redaction-service-contract block) by
`docs/brehon-law-inspired-network/04-data-model-and-api.md`.
These two systems are DIFFERENT THINGS by design: the PG
table is hash-chained product surface; this sidecar is
observability tooling. Different consumers, different write
paths, different durability guarantees.

First registered kind: `retro_bypass` (fail-open bypass trail emitted by `.claude/hooks/retro-check.sh` — see §2 Kind registry).

## 2. Kind registry

| kind | originating hook | sidecar path | JSONL field schema | consumer |
|---|---|---|---|---|
| `retro_bypass` | `.claude/hooks/retro-check.sh` (fail-open path; `emit_retro_bypass_log` function) | `.claude/governance-log/retro-bypass.jsonl` | `{timestamp: ISO 8601 UTC, session_id: string, attempt_count: integer, prompt_hash: 16-hex SHA-256 of CLAUDE_PROMPT, branch_at_fail_open: string, kind: "retro_bypass"}` | future audit reads (a dedicated rate-trend audit step would be added if `retro_bypass` rate becomes load-bearing; weekly-review Step 2c is the retro-corpus sweep over `.claude/PRPs/reports/*.md`, not this JSONL trail) |

## 3. Adding a new kind

To register a new hook-emitted observability kind:

(a) Define the sidecar path under `.claude/governance-log/<kind>.jsonl`.
(b) Confirm `.gitignore` includes `.claude/governance-log/` (already gitignored — added in v1-rls-r1 Task 5; verify on lane bootstrap).
(c) Author the emit shell function in the originating hook — append a `emit_<kind>_log` function that writes a single JSONL line to the sidecar path.
(d) Add a row to the §2 table above with all five columns populated.
(e) Document the consumer (weekly-review Step 2c / a future skill / ad-hoc audit reads) in the table's `consumer` column.
(f) Add a paired lesson at `.claude/lessons/feedback_<kind>_governance_log.md` cross-linking the instrumentation back to this doc. Per `feedback_lesson_must_pair_with_structural_fix_when_fixable.md`.
