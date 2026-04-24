---
id: pr92-cr-findings
from: advisor
to: impl
ts: 2026-04-24T00:35Z
relates_to: PR-92
decision: fix-in-pr-all-four-majors
---

# Decision
CR posted 10 findings on PR #92. All 4 majors are `fix-in-pr` (no carry-forward, no rebut). Lows + nit triage happens separately. Start on cr-9 first (longest-path fix).

# Instructions

Read full findings at `C:/Users/barri/Developer/brehon-fork/.claude/PRPs/reviews/pr-92-findings.yaml`. Ingest with `yq '.findings[] | select(.severity == "major")'` for the four below.

**cr-9 — ADR-015 pseudonymisation violation (`relaxation_reason TEXT`)**. Highest priority; schema-design fix. The free-text `relaxation_reason` column on `jury_constraint_violation_log` can leak PII into the governance audit log. Replace with: `reason_code` (new Postgres enum: `reputation_waiver`, `emergency_panel`, `sponsor_vouched`, `admin_override`, `other` — draft enum values from PRD §10.7 if named there, else propose in impl relay for advisor sign-off) + `relaxation_metadata JSONB` (optional bounded-schema payload, not free text). Migration rewrite needed on `add_jury_mechanics_columns/up.sql:74` + `down.sql`. Diesel model on `crates/db_schema/src/source/governance/jury_constraint_violation_log.rs` mirrors. Add ADR-015 reference to migration SQL header.

**cr-7 + cr-8 — Missing ADR exception trail in migration SQL headers**. Both on `add_jury_mechanics_columns` migration (up.sql:36 + down.sql:15). Protected governance tables (`moderation_case`, `jury_assignment`) get new columns in up.sql and have those columns dropped in down.sql. Add a header comment block referencing the ADR that authorised the schema change (likely ADR-010 or similar — check `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` for the governing ADR number). Format:
```sql
-- ADR exception trail
-- Protected-table ALTER authorised by: ADR-<N> (jury mechanics v1 schema extensions)
-- Tables touched: moderation_case (6 columns added), jury_assignment (2 columns added)
-- Reversibility: down.sql drops in LIFO order; no data-preservation required (v0→v1 backfill is re-derivable)
```
Cheap fix — comment-only.

**cr-10 — Non-idempotent seed `ON CONFLICT`**. `seed_v1_jm_config_keys/up.sql:58` has `ON CONFLICT (scope, key, valid_from)` but the INSERT doesn't supply `valid_from` (defaults to `NOW()` which changes per run). Reruns insert duplicate active rows instead of deduping. Two fix paths, pick whichever matches AD-a's seed pattern:
- (a) Supply explicit `valid_from = CURRENT_DATE` (stable-per-day) or a fixed literal (e.g. `'2026-04-23'::date`) so the conflict target matches on rerun
- (b) Change conflict target to `(scope, key) WHERE valid_to IS NULL` (partial unique on active rows only) — requires adding a partial index if it doesn't exist
Check how v1-AD-a's `seed_v1_config_keys` migration handled this (trunk); mirror.

Validation after fix:
- L1 `cargo check --workspace` exit 0
- L2 `cargo check --workspace --features full` exit 0
- L4 e2e targeted: rerun `config_parity_round_trip` (must still pass 88 keys after cr-10 fix) + `v1_jm_a_backfill_populates_v0_snapshot` (cr-9 column rename doesn't break backfill test) + `phase1_migrations_round_trip` stays ignored
- L5 `cargo clippy --workspace --features full --no-deps -- -D warnings` exit 0
- L8 Idempotency probe: run seed migration TWICE against a fresh DB; second run should be a no-op (zero duplicate rows) — add as a new e2e test if not covered

# Retro carry (Task 11 retro already written — amend as needed)
- **Plan-drift addendum**: cr-9 is a plan-author gap — PRD §10.7 (jury constraint violation logging) should have flagged the ADR-015 collision when `relaxation_reason TEXT` was specified. Recommend plan-template addition: "For every new governance-log-adjacent free-text column, cross-check ADR-015 at PRD time; default shape is `reason_code` enum + `JSONB` metadata."
- cr-10 suggests AD-a's seed pattern may also have this bug — verify after JM-a lands. If yes, file a `chore(test): retrofit AD-a seed idempotency` issue.

# Next
1. Fix cr-9 first (longest path: migration + Diesel model + enum type).
2. Then cr-10 (medium: migration SQL + possible partial index).
3. Then cr-7/cr-8 (fastest: SQL header comments only).
4. Triage the 5 lows + 1 nit in a single batched commit after the 4 majors land.
5. File relay `impl-relays/pr92-cr-fix-complete.md` when all 4 majors have `addressed_in` SHAs; advisor will then run `/bm-poll-cr 92` again and `/bm-triage 92` to promote to `done`.

Commit subjects (per plan §task-per-commit + CR ref convention):
- `fix(v1-JM-a): cr-9 — ADR-015 pseudonymisation — replace relaxation_reason TEXT with reason_code + relaxation_metadata JSONB`
- `fix(v1-JM-a): cr-10 — idempotent seed via stable valid_from (mirrors AD-a pattern)`
- `fix(v1-JM-a): cr-7 cr-8 — ADR exception trail in protected-table migration SQL headers`
