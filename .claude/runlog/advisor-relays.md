# Advisor → impl relays

Copy-paste archive of answers and guidance the advisor session sent to impl
via user-relay. Append-only. Latest entry is always at the bottom.

Each entry is a dated markdown section; body between the `##` heading and the
`---` separator is the copy-paste-ready text for the impl session.

Companion files:
- `impl-relays.md` — impl → advisor direction (status reports, DQ drafts)
- `bm-runlog.md` — BM/advisor state-changes (git, PR, DQ answers, runlog)

---

## 2026-04-23T23:40Z — R10.1 answer (Task 10 PHASE_1_MIGRATION_COUNT)

**Decision**: Pick (c). Extend `PHASE_1_MIGRATION_COUNT` from 9 → 12. Do NOT file DQ #47 — answer accepted in-channel, `answered_by: "user"` if recorded anywhere.

### Instructions for Task 10's sub-edit 1

1. Set `const PHASE_1_MIGRATION_COUNT: u64 = 12;`

2. Update the comment block at `e2e.rs:311-321` to reflect the **real** arithmetic. The current comment is wrong even today — it names "6 Phase 1 + 2 Phase 5a + 1 Phase 5b Slice A = 9" but LIFO revert of 9 picks up the 2 `governance_log_notify` migrations + `federation_attestations` that the comment doesn't name. New comment should acknowledge the LIFO-positional nature of the count, not the semantic-set fiction.

3. Add an inline `TODO(v0-polish)` at the constant referencing two items:
   - Existing GH issue #43 (deflake reason in the `#[ignore]` attribute)
   - A NEW GH issue sketch to file in Task 11 retro: "`PHASE_1_MIGRATION_COUNT` is a LIFO count, not a semantic set — comment claims it reverts specific named migrations but the runner reverts top-N-by-timestamp, so any post-trunk migration added after the last bump silently takes the Nth slot. v1-AD-a's 4 migrations are currently uncounted and will be silently swapped into the revert list when the test is un-ignored."

4. Do NOT extend by 7. AD-a retrofit is out-of-scope for JM-a per plan §18 risk row 7 + §19 Notes. If/when someone un-ignores the test, they do AD-a retrofit in a separate `chore(test): retrofit v1-AD-a migrations into phase1_migrations_round_trip` commit.

### Task 11 retro carry

R10.1 gets a retro entry with:
- Root cause: "plan §13 Task 10 anticipated a single-number drift; reality is a semantic-model bug"
- The new GH issue sketch for the count-model
- Recommendation to replace the count-based revert with a named-migration list for future phases

### Proceed

Continue with the rest of Task 10 (table-list loop extensions + new `v1_jm_a_backfill_populates_v0_snapshot` test) once sub-edit 1 is in place.

---
