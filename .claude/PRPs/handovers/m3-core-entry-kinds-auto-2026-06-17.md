# auto-phase handover — m3-core-entry-kinds (auto-refreshed)

**Refreshed:** 2026-06-17T22:55:00Z · stage `impl-cohort-1-running`

## RESUME block (self-contained)
- **Sub-phase:** `m3-core-entry-kinds` (M3 town halls — Phase 2). Branch `phase-m3-core-entry-kinds`.
- **Stage:** `impl-cohort-1-running` — Junior #689 (`[role:impl-task]`) implementing the 3 chair/mute consts + shim + `ROOM_KINDS` 10→13. Single-task cohort (Tasks 1+2 consolidated), `is_last_cohort: true`.
- **Last commit on phase branch:** `33efbb27f` (trunk merged in for brief visibility; impl commit not yet landed).
- **Last commit on governance-v0:** `0e2425d16` — impl brief.
- **Gate 1 (plan approval):** APPROVED 22:48Z. Advisor owns the inline validation-command fix — the plan's Level-5/Task-3 invariant greps use `rg` (absent on laptop Bash + daemon) AND `rg '^\s+ENTRY_KIND_' | wc -l` counts LINES not consts. Underlying parity is fine (69/69/10 pre-impl).
- **Next concrete action (re-verify on resume):** poll #689 → on `done`, read the `validate-pending-laptop` DQ entry → run `./scripts/brehon/cargo-check.sh --workspace --features full` locally (advisor-laptop path; pre-Shape-G) + the CORRECTED invariant greps (use `grep -oE`, NOT `rg`): db_schema distinct consts == 72, shim-block distinct consts == 72, no dup literals, `ROOM_KINDS` len == 13. Mutate the DQ entry result. On pass → daemon finalize-merges → Task 4 (advisor registry reconcile on governance-v0) → bm-pr.
- **DQ pending ids:** none yet (#689 will raise one).
- **Concurrent activity:** none (single canonical worktree; daemon driving #689 only).

## Validation fix (advisor owns — gate-1 approved)
When running Task-3 / Level-5 invariants on the laptop, substitute the broken `rg` commands:
```bash
# A: db_schema distinct consts (expect 72)
grep -oE '^pub const ENTRY_KIND_[A-Z_]+' crates/db_schema/src/source/governance/governance_log.rs | sort -u | wc -l
# B: duplicate literals (expect empty)
grep -oE 'ENTRY_KIND_[A-Z_]+: &str = "[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs | grep -oE '"[a-z_]+"' | sort | uniq -d
# Shim parity (expect 72): distinct consts in the pub use block
awk '/pub use lemmy_db_schema/,/};/' crates/api/api/src/governance/governance_log.rs | grep -oE 'ENTRY_KIND_[A-Z_]+' | sort -u | wc -l
# ROOM_KINDS len (expect 13)
awk '/const ROOM_KINDS/,/];/' crates/api/api/src/governance/governance_log.rs | grep -cE 'ENTRY_KIND_ROOM_'
```
