# Phase 5b — Complete Report

**Branch:** `phase-5b` (13 commits since branch-off from `governance-v0`)
**Closing commit:** `f183abfd9 test(governance): task 61 — bump PHASE_1_MIGRATION_COUNT 8 → 9 for Slice A migration` (2026-04-18)
**Plan:** `.claude/PRPs/plans/phase-5b-sponsor-liability-and-founder-bootstrap.plan.md`
**Design ref:** `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` §3 Phase 5b

---

## 1. Delivered vs plan

| Plan task | Status | Commit |
|---|---|---|
| 0 — pre-phase audit + branch cut | shipped | (plan commit 54fc8e99f) |
| 56 — sponsor_liability helper + Restoration variant + submit_jury_vote config reads + Scope::as_str refactor | shipped | 5aee34738 |
| 56 (fix) — align write plane to instance (decision-queue #16) | shipped | 61ddae110 |
| 57 — reputation gating + concurrent cap in admin_assign_jury | shipped | 11c1a2083 |
| 58 — OQ-006 threshold formula in create_report + is_finite() guard | shipped | 1682a544f |
| 59 — founder seeding CLI (crates/tools/seed_founders) | shipped | 77447ad8c |
| 59 (fix) — rename CLI error label to clear §12 Level 5 PII grep | shipped | f58da3d80 |
| 60 — sponsor_liability_with_founder_multiplier e2e (3 branches) | shipped | a56fe3ccb |
| 61 — phase-close validation + report + PR | in-progress | (this commit) |

Supporting commits (non-task): `2a5ad0caf` session wrap, `46e22a363` PM-hooks rule, `1e42f9d35` CI plan-drift, `461efad29` decision-queue resolution.

## 2. Commit list (chronological)

1. `54fc8e99f` docs(plan): phase-5b plan + narrow Level 2 parity DoD (decision-queue #15)
2. `5aee34738` feat(governance): task 56 — sponsor_liability + Restoration + submit_jury_vote + Scope::as_str
3. `2a5ad0caf` chore(session): phase-5b slice A wrap
4. `11c1a2083` feat(governance): task 57 — reputation gating + concurrent cap in admin_assign_jury
5. `46e22a363` docs(rules): pm-plugin-hooks-stable — preserve V2 messaging hooks
6. `1682a544f` feat(governance): task 58 — OQ-006 threshold formula + is_finite() guard
7. `1e42f9d35` fix(ci): plan-drift route extraction + oq-sweep verdict normalisation
8. `77447ad8c` feat(tools): task 59 — founder seeding CLI
9. `61ddae110` fix(governance): task 56 — sponsor_liability writes instance-scoped (decision-queue #16)
10. `a56fe3ccb` test(governance): task 60 — sponsor_liability_with_founder_multiplier e2e (3 branches)
11. `461efad29` chore(decision-queue): resolve #16 — split-plane bug fix per advisor Option B
12. `f58da3d80` fix(tools): task 59 — rename CLI error label to clear §12 Level 5 PII grep
13. `f183abfd9` test(governance): task 61 — bump PHASE_1_MIGRATION_COUNT 8 → 9 for Slice A migration

## 3. Deviations from plan

### 3.1 Split-plane bug in apply_sponsor_liability (decision-queue #16)

Task 56 shipped with `apply_sponsor_liability` reading the union plane (`community_id IS NULL OR = cid`, newest by `calculated_at`) for the clamp math but writing the `reputation_event` with `community_id = case.community_id`. Founder seeds live on the instance plane (`community_id = None`), so the write landed on a plane `load_live_events` filters out of the instance recompute. Consequence: the `+seed` and `−liability` events never composed at snapshot recompute time; founder capability evaluation became plane-dependent and non-deterministic across communities.

Surfaced by task 60's `FOUNDER_CHAIN_SURVIVAL` println showing `endorsement_strength=100` post-sanction (expected `0` per plan §11.5 line 1032). Advisor confirmed Option B (fix in slice C) over Option A (defer to Phase 5c): shipping Phase 5b with a known-broken capability path would let founders absorb arbitrarily many sanctions per-community. One-line fix at `sponsor_liability.rs:278-287` (`community_id: case.community_id` → `community_id: None`) landed at `61ddae110` before task 60's commit. `source_case_id` preserves audit linkage regardless of plane.

This deviation is the retro nomination for Phase 5b.

### 3.2 Decision-queue staleness at task 60 commit

Decision-queue #16 was recorded with `answer: null` in task 60's commit (`a56fe3ccb`) because the advisor's Option B resolution arrived after the commit landed. Resolved post-hoc via `chore(decision-queue): resolve #16` at `461efad29`. Retro note: advisor pipe is not real-time; commits land on the impl agent's cadence. The chore follow-up is the right pattern — no `--amend`, clean audit trail.

### 3.3 §12 Level 4 `PHASE_1_MIGRATION_COUNT` bump (GOTCHA-L4a)

Plan anticipated this; task 61 commit `f183abfd9` bumps the constant 8→9 to account for Slice A's `add_restoration_sanction_variant` migration. `phase1_migrations_round_trip` now reverts all 9 governance-bootstrap migrations LIFO and re-applies them cleanly.

### 3.4 §12 Level 2 "full e2e suite" requires `--test-threads=1`

Running `cargo test --test e2e` without `--test-threads=1` produces a race: tests share the `SETTINGS` (LazyLock) singleton, which caches the first `LEMMY_DATABASE_URL` set. When the suite parallelises, later tests read the cached URL and probe the wrong container. All 10 individual tests pass; full suite with `--test-threads=1` passes (10 passed, 0 failed, 424.62s).

This is a pre-existing harness property surfaced by slice C adding the second container-driven test. Task 60's `report_to_modlog_golden_path` already documents the assumption at line 794-801: "tests run with --test-threads=1 so no concurrent env mutation". The cargo-test wrapper does not enforce it, which is a wrapper gap.

Carry-forward for Phase 5c: either enforce `--test-threads=1` in the wrapper, or add a `.cargo/config.toml` `[test]` stanza for the e2e target. Not a Phase 5b blocker — every serial run is green.

### 3.5 §12 Level 5 Watch 3 variant arm count uses line-count heuristic

Plan line 1227-1228 asserts "≥8" variant arms named in `sponsor_liability.rs`. Actual grep count returns 7 because 3 variants share one line via `|` (line 116: `Label | VisibilityReduction | Restoration`). All 8 `SanctionAction` variants are named exhaustively — confirmed manually via `grep -n 'SanctionAction::'`. The line-count heuristic undercounts; the exhaustive-match invariant holds.

## 4. Decision-queue activity

### Closed in Phase 5b

- **#11** (OQ-004 juror cap at 3): resolved by planner 2026-04-17. Task 57 reads `config.jury.max_concurrent_assignments` from the Phase 5a seed.
- **#12** (sponsor-liability severity units): resolved by planner 2026-04-17. Task 56 maps `JuryDecision` → `SanctionAction` per `severity_for_action`; config keys `deltas.sponsor_liability_{minor,moderate,severe}` read from Phase 5a seed.
- **#13** (admin-config-write.sh wrapper): resolved by advisor — ship as Phase 5c sibling docs/scripts artefact, not an impl-plan task.
- **#14** (plan-drift `--features full` on `-p lemmy_server`): self-resolved by impl during Phase 5a close.
- **#15** (Level 2 parity DoD narrowing): resolved by advisor 2026-04-17. Plan edits at `54fc8e99f` dropped the `-p lemmy_api` parity invocation.
- **#16** (sponsor_liability split-plane bug): resolved by advisor 2026-04-18, Option B. See §3.1 above.

### Opened in Phase 5b

- **#16** (discovered + resolved within the phase).

### Carried forward

None open at phase close.

## 5. Carry-forward into Phase 5c

1. **Founder-chain-survival under default config (plan line 1035 / risk register R3).** Task 60 branch 2 asserts the math-faithful outcome: both founders clamp to `endorsement_strength=0` after a severe sanction with default `sponsor_liability_severe=-200` + `founder_multiplier=2.0` + `sponsor_liability_floor=0`. Founders do NOT retain the sponsorship capability under defaults. `FOUNDER_CHAIN_SURVIVAL` diagnostic preserved in the test body per advisor rule 12 (load-bearing observability, not debug cruft). V1 tuning concern: either raise `founder_multiplier` asymmetrically, raise the floor, or soften `severe` units. Not a 5c blocker; a design surface the pilot community will need resolved before live onboarding.

2. **Snapshot recompute does not re-apply the honour-price floor (OQ-025 candidate).** Surfaced during decision-queue #16 diagnosis. `apply_sponsor_liability` applies the floor at write time; `recompute_snapshot` sums events without re-clamping. A double-sanction-within-tick can drive snapshot below floor because the floor clamp in `apply_sponsor_liability` reads a pre-tick snapshot. Not exercised by task 60 (plan §11.5 branches are single-sanction). Log as OQ-025 candidate if the project owner decides the floor is a snapshot-level invariant rather than only a write-time clamp. Not a 5c blocker.

3. **`--test-threads=1` enforcement in cargo-test wrapper.** See §3.4. Optional Phase 5c chore.

4. **Post-target sponsor-liability path** (carry-forward from Phase 4b decision-queue #10). Task 60 uses Person-target cases. Post/Comment-target sponsor-liability is not exercised by any 5b test. Phase 5c task 57 target-inference fix or v1.

5. **V2/messaging.md uncommitted drift.** Pre-existing uncommitted edit in working tree since before slice C. Non-scope; next ops session should claim or revert.

## 6. Retro nomination

Per advisor rule 12 (long retro when a checkpoint fires), the split-plane bug (§3.1) is the 5b retro site. Three retro items worth the long-form treatment:

1. **Task 56 shipped without an end-to-end test exercising the helper.** The phase-close regression guards were green at task 56's commit (`5aee34738`) because no test drove sponsor-liability end-to-end until task 60. "Gates green" does NOT mean "bug-free"; future phases with cross-cutting helpers must ship with at least one integration test that composes read + write paths.

2. **Plan assertions are necessary-but-not-sufficient.** Task 60 branch 2's literal assertions passed while the `FOUNDER_CHAIN_SURVIVAL` println showed `endorsement_strength=100` — the diagnostic print was the load-bearing integration invariant. Keep those prints; they survive the next retro too.

3. **Scope-consistency watch gap.** Task 56's doc-comments advertise "cross-community sponsor-liability uses the (sponsor, community_id) snapshot when the case is community-scoped", which describes the intended semantics. The actual write uses `community_id = case.community_id` (matches the doc) but the **read** filter is a union — a latent plane asymmetry that no review caught. Watch items for future phases: any helper with `community_id`-parameterised reads AND writes should assert the two paths query the same plane under test.

## 7. §12 Validation results (at task-61 HEAD, commit `f183abfd9`)

| Level | Command | Result |
|---|---|---|
| L1 | `cargo-check.bat --features full --workspace` | exit 0 |
| L1 | `cargo-clippy.bat --features full --workspace --no-deps -- -D warnings` | exit 0 |
| L2 | `cargo-test.bat -p lemmy_server --test e2e -- config_parity_round_trip` | 1 passed |
| L2 | `cargo-test.bat -p lemmy_server --test e2e -- report_to_modlog_golden_path` | 1 passed (Watch 4) |
| L2 | `cargo-test.bat -p lemmy_server --test e2e -- sponsor_liability_with_founder_multiplier` | 1 passed |
| L2 | `cargo-test.bat -p lemmy_server --test e2e -- --test-threads=1` | 10 passed, 0 failed |
| L3 | `cargo-test.bat --test e2e --no-run -p lemmy_server` | compile ok |
| L3 | `cargo-check.bat -p brehon_seed_founders` | exit 0 |
| L4 | `head -1 migrations/2026-04-19-000000-0000_add_restoration_sanction_variant/up.sql` | `-- no-transaction` |
| L4 | `phase1_migrations_round_trip` (post-bump) | 1 passed |
| L5 | `lint-no-membership-read.sh` | pass |
| L5 | `lint-no-can-sponsor-read.sh` | pass |
| L5 | PII grep on `sponsor_liability.rs` + `seed_founders/main.rs` | exit 1 (no matches) |
| L5 | Watch 3 exhaustive match on `SanctionAction` variants | 8/8 named (line-count heuristic shows 7; see §3.5) |

## 8. File inventory (slice C only)

| File | Change |
|---|---|
| `Cargo.toml` | +1 workspace member (`crates/tools/seed_founders`) |
| `Cargo.lock` | +19 lines (clap + brehon_seed_founders + regex) |
| `crates/api/api/src/governance/sponsor_liability.rs` | +9 / −1 (split-plane fix at decision-queue #16) |
| `crates/server/Cargo.toml` | +1 dev-dep (regex) |
| `crates/server/tests/e2e.rs` | +533 (new test: `sponsor_liability_with_founder_multiplier`, 3 branches + PII grep) + 3 lines (migration count bump) |
| `crates/tools/seed_founders/Cargo.toml` | new file, 29 lines |
| `crates/tools/seed_founders/src/main.rs` | new file, 286 lines (founder CLI) |
| `.claude/decision-queue.json` | +1 entry (#16 resolved) |

## 9. Acceptance criteria (from plan §15)

- [x] All §12 Level 0–5 gates green at task-61 HEAD.
- [x] `sponsor_liability_with_founder_multiplier` test exists and passes (all 3 branches + PII grep).
- [x] `brehon_seed_founders` binary compiles and exposes `seed_founders` entry point.
- [x] Watch 4 `report_to_modlog_golden_path` passes at phase HEAD.
- [x] Decision-queue #11, #12 closed; #16 opened + closed within the phase.
- [x] Completion report written (this file).
- [ ] PR `phase-5b → governance-v0` opened (next step after this commit).
