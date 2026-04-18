---
iteration: 1
max_iterations: 6
plan_path: ".claude/PRPs/plans/phase-5b-sponsor-liability-and-founder-bootstrap.plan.md"
input_type: "plan"
slice: "A"
slice_tasks: "0,56"
started_at: "2026-04-17T00:00:00Z"
---

# PRP Ralph Slice — phase-5b-sponsor-liability-and-founder-bootstrap.plan.md — Slice A (resumed)

## Slice scope (hard gate for completion)

This loop implements these tasks:

- **Task 0** — Pre-phase harness audit + `phase-5b` branch cut + decision-queue #11/#12 close-out. ALREADY COMMITTED at `54fc8e99f` (task-0 produces zero commits per plan; the committed commit is the `docs(plan): phase-5b plan + narrow Level 2 parity DoD (decision-queue #15)` preflight mitigation for the L2 parity DoD breakage discovered during the audit).
- **Task 56** — Sponsor-liability helper + `SanctionAction::Restoration` variant + migration + `submit_jury_vote.rs` config reads + `Scope::as_str` → `Cow<'static, str>` refactor. ALL IN ONE COMMIT. **Done — landed at `5aee34738` (Slice A complete 2026-04-17).**

**Out of scope for this slice (slice B/C will handle):**

- Task 57 — admin_assign_jury reputation gating + concurrent-cap (slice B)
- Task 58 — create_report OQ-006 threshold formula (slice B)
- Task 59 — founder bootstrap CLI (slice C)
- Task 60 — three-branch e2e (slice C)
- Task 61 — PR + phase-close report (slice C)

## Completion criteria (all must be true before `<promise>COMPLETE</promise>`)

1. A `feat(governance): task 56 —` commit exists on `phase-5b` branch (task 0's commit `54fc8e99f` already satisfies task 0's "zero commits" contract-minus-the-preflight-mitigation).
2. `git diff --stat 54fc8e99f HEAD` shows (at minimum):
   - `migrations/2026-04-19-000000-0000_add_restoration_sanction_variant/up.sql` (new)
   - `migrations/2026-04-19-000000-0000_add_restoration_sanction_variant/down.sql` (new)
   - `crates/db_schema_file/src/enums.rs` (edited)
   - `crates/api/api/src/governance/config.rs` (Cow refactor)
   - `crates/api/api/src/governance/sponsor_liability.rs` (new)
   - `crates/api/api/src/governance/submit_jury_vote.rs` (edited — consts removed, cache reads, apply call)
   - `crates/api/api/src/governance/mod.rs` (`pub mod sponsor_liability;`)
3. Task 56 DoD (§11.1 lines 610-620):
   - [ ] Migration up.sql first line exactly `-- no-transaction`, second line `ALTER TYPE sanction_action ADD VALUE IF NOT EXISTS 'Restoration';`
   - [ ] `SanctionAction::Restoration` variant exists in enums.rs
   - [ ] `sponsor_liability.rs` compiles, `severity_for_action` exhaustive (no `_ =>`)
   - [ ] `config.rs:63-70` `Scope::as_str` returns `Cow<'static, str>` with all 6 call sites updated
   - [ ] 4 Phase 4 delta consts removed from submit_jury_vote.rs, ConfigCache reads in place
   - [ ] `apply_sponsor_liability` call inserted between step 8 and step 9 inside `if let Some((scope, action))`
4. Level 1 workspace validation exits 0: `cargo-check.bat --features full --workspace` + `cargo-clippy.bat --features full --workspace --no-deps -- -D warnings`.
5. Phase 4 regression: `report_to_modlog_golden_path` e2e exits 0 (Watch 4).
6. Phase 5a regression: `config_parity_round_trip` e2e exits 0 (Watch 1, narrowed per decision-queue #15).
7. Lint guards: `bash scripts/brehon/lint-no-membership-read.sh` and `bash scripts/brehon/lint-no-can-sponsor-read.sh` exit 0.
8. Watch 10 PII grep on `sponsor_liability.rs`: `grep -nE '\b(person_id|target_person_id|sponsored_id|sponsor_id)\b' crates/api/api/src/governance/sponsor_liability.rs | grep -v ':fn \|^crates/api/api/src/governance/sponsor_liability.rs:\d+:\s*//' ` — only type references (PersonId the newtype, field access in queries) allowed; no raw ids in `json!` payloads.

## Codebase Patterns (reusable across iterations)

### Pattern 1 — ConfigCache reads return concrete types
`config::get_int(cache, pool, Scope::Instance, "key") -> LemmyResult<i64>` signature takes `&mut DbPool<'_>`, so config reads cannot happen inside a `run_transaction` closure that holds `conn: &mut AsyncPgConnection`. Path B (pre-warm cache before transaction) is the conservative choice. The outer handler reads all keys via `cache.get_*()` calls BEFORE `run_transaction`; the closure only reads from the already-warm cache.

Critical: inside the helper `apply_sponsor_liability` which is called from within the transaction, config cache reads must ONLY hit already-warmed entries. The outer handler must pre-warm:
- `deltas.juror_aligned`, `deltas.juror_outlier`, `deltas.reporter_upheld`, `deltas.reporter_dismissed` (existing 4 consts replaced)
- `deltas.sponsor_liability_minor/moderate/severe` (all 3 — mapping depends on action chosen at vote-tally time)
- `liability.founder_multiplier`, `liability.regular_multiplier`
- `liability.sponsor_liability_floor`

### Pattern 2 — `run_transaction` shape (community/ban.rs:59-64)
```rust
conn.run_transaction(|conn| {
  async move {
    // multi-write body here
    Ok::<_, LemmyError>(())
  }.scope_boxed()
}).await?;
```

### Pattern 3 — Watch 10 PII: pseudonymise via `actor_pseudonym_helper::get_or_create`
Every `governance_log` payload touching a sponsor must use `sponsor_pseudonym: Uuid` (string form from `.to_string()`), never a raw `PersonId(i32)`. See Phase 4b `create_report` for reference.

## Current Task

Execute task 56 per §11.1 of the plan. ONE commit. Title: `feat(governance): task 56 — sponsor_liability helper + Restoration variant + submit_jury_vote config reads + Scope::as_str refactor`.

Task 0 is already complete (commit `54fc8e99f` landed the preflight mitigation; the task-0 audit artifacts live at `.claude/audit-*.log`).

## Instructions (per iteration)

1. Read this state file top to bottom.
2. Read §11.1 of the plan (`.claude/PRPs/plans/phase-5b-sponsor-liability-and-founder-bootstrap.plan.md`, lines 333-622).
3. Check `git log --oneline phase-5b --not 5a4a0f0a5` — task 0 should show the docs(plan) commit; task 56 is missing.
4. For task 56, implement per plan §11.1 in the order:
   a. Scope::as_str Cow refactor (config.rs) — smallest, validates Level 1 check
   b. Migration files (up.sql + down.sql)
   c. SanctionAction::Restoration variant (enums.rs) — check if `DbEnum` derive supports struct variant; if not, fall back to unit variant per GOTCHA-56b
   d. sponsor_liability.rs — full helper with severity mapping + integer math + founder check + floor clamp + two governance_log emits
   e. submit_jury_vote.rs — remove 4 consts, pre-warm cache, insert `apply_sponsor_liability` call
   f. mod.rs — `pub mod sponsor_liability;`
5. Capture EVERY cargo invocation to `.claude/build-task56-<level>.log` per cargo-output-capture.md; tail ≤20 lines.
6. On Level 1 green: commit with the required message form.
7. Post-commit: run §11.1 step 7 validation sweep (golden-path + parity + lint guards + Watch 10 grep).
8. If all completion criteria hold, append iteration log entry + emit `<promise>COMPLETE</promise>`.
9. Otherwise, log what was done + what's blocked; hook feeds the prompt back.

## What NOT to do (slice discipline)

- Do NOT touch task 57/58/59/60/61 code — those are slice B/C.
- Do NOT open a PR or write a `docs(report)` commit — slice C closes the phase.
- Do NOT nest `run_transaction` inside the apply_sponsor_liability helper — the outer `process_vote` transaction provides the conn.
- Do NOT emit raw `person_id` / `sponsor_id` in `governance_log` payloads — always pseudonymise (Watch 10).
- Do NOT use `_ =>` wildcard in `severity_for_action` match — exhaustive only (Watch 3).
- Do NOT paste cargo log bodies into the conversation — read file tails only (no-cargo-output-paste.md).

## Progress Log

### Iteration 1 — resumed from prior-session task 0 completion

**Status going in:** `phase-5b` branch at `54fc8e99f docs(plan): phase-5b plan + narrow Level 2 parity DoD (decision-queue #15)`. Task 0 audit artifacts at `.claude/audit-*.log` (all green after L2 narrowing per decision-queue #15). Task 56 not yet implemented.

**Plan:** implement task 56 end-to-end in one commit per §11.1.

**Actions:**

- **Scope::as_str → Cow<'static, str>** refactor in `crates/api/api/src/governance/config.rs` (5 call sites + fetch_value_at_scope signature updated; `.as_ref().to_string()` for cache-key lookups; `.into_owned()` for cache-key inserts and DB filter).
- **Migration** `migrations/2026-04-19-000000-0000_add_restoration_sanction_variant/{up,down}.sql` — `-- no-transaction` directive, `ADD VALUE IF NOT EXISTS 'Restoration'`.
- **SanctionAction::Restoration** — unit variant (GOTCHA-56b fallback: struct variant would break `Copy` which the enum derives). Doc comment cites OQ-003 amendment 2026-04-17 and GOTCHA-56a Minor severity bucket.
- **sponsor_liability.rs** new helper (332 LOC): `LiabilitySeverity` enum + `severity_for_action` exhaustive match + `multiply_and_round` (banker's rounding via `round_ties_even`; `#[expect(clippy::as_conversions, reason=...)]` on the helper only — workspace denies `as_conversions` globally). Body: raw_delta → sponsor_count divide → remainder_bump → founder exists check against `reputation_event.expires_at > now()` → multiplier → round → floor clamp against most-recent reputation_snapshot → `reputation_event` insert → two governance_log emits (`sponsor_liability_applied` always; `sponsor_liability_clamped` if floor fired). Watch 10: all `json!` payloads use `sponsor_pseudonym` (UUID string), zero raw person ids.
- **submit_jury_vote.rs** — removed 4 delta consts (replaced with config reads via `ConfigCache`, pattern `config::get_int(&mut cache, &mut (&mut *conn).into(), Scope::Instance, key)` — same as `reputation_snapshot.rs`); inserted `sponsor_liability::apply_sponsor_liability(...)` call at new step 8.5 inside `if let Some((scope, action))` block.
- **mod.rs** — `pub mod sponsor_liability;`.

**Validation (captured logs, tails only):**

- `build-task56-step0.log` — `cargo-check.bat -p lemmy_api --features full` after Cow refactor: exit 0 (1m 08s).
- `build-task56-l1.log` — `cargo-check.bat --features full --workspace`: exit 0 (3m 44s).
- Clippy iterations `build-task56-clippy{1..4}.log` — hit `redundant_type_annotations`, `as_conversions` (x3). Fixed via `let remainder_abs = ...` (drop type annotation), `u64::try_from(i)?` (replace `i as u64`), helper function `multiply_and_round` with `#[expect(clippy::as_conversions, reason=...)]`. Final `build-task56-clippy4.log` exit 0 (51.14s).
- `build-task56-parity.log` — `config_parity_round_trip` exit 0 (20.63s, 1 passed). Watch 1 regression guard green.
- `build-task56-golden.log` — `report_to_modlog_golden_path` exit 0 (33.69s, 1 passed). Watch 4 regression guard green (critical — task 56 mutated `submit_jury_vote`).
- Lint guards: `membership_state guard: pass`, `can_sponsor guard: pass`.
- Migration DoD: `head -1 migrations/.../up.sql` grep-matches `-- no-transaction` exactly.
- Watch 10 PII grep on sponsor_liability.rs: all `person_id`/`sponsor_id` hits are column accesses, type signatures, or struct fields of `ReputationEventInsertForm`; zero hits in `json!` payload values.

**Commit:** `5aee34738 feat(governance): task 56 — sponsor_liability helper + Restoration variant + submit_jury_vote config reads + Scope::as_str refactor` — 7 files changed, 450 insertions, 29 deletions.

**Slice A status:** complete. All 8 completion criteria hold:
1. ✅ Task 56 commit exists on phase-5b with required message form.
2. ✅ Diff covers all expected files (migration up/down, enums.rs, config.rs, sponsor_liability.rs new, submit_jury_vote.rs, mod.rs).
3. ✅ Task 56 DoD rows all met (migration first-line, Restoration variant, helper compiles, Cow refactor, consts removed, apply call inserted).
4. ✅ Level 1 check + clippy green.
5. ✅ Watch 4 golden-path regression green.
6. ✅ Watch 1 parity regression green (narrowed per decision-queue #15).
7. ✅ Both lint guards pass.
8. ✅ Watch 10 PII clean.

---
