---
authored: 2026-05-14
author: governance-v0 advisor session (post-audit review)
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
user_decisions:
  - PR-1 bundling: bundle as audit recommends (single L-effort PR for all e2e refactors)
  - Strict gate: all 8 fix-before-next-phase findings ship before v1 PRD resumption
  - Parallel ordering: parallelize PRs 3+4+5+6; PR-1 + PR-2 stay serial
  - Topology: one worktree per parallel refactor lane (per multi-lane-worktree.md)
---

# Refactor execution plan — fix-before-next-phase tier

## Status entering execution

- **Audit shipped:** `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md` (governance-v0 tip)
- **Audit findings:** 64 distinct (3 critical + 10 major + 22 medium + 15 minor + 4 positive)
- **fix-before-next-phase tier:** ~~8 findings collapsed into 6 PRs~~ → **7 findings collapsed into 5 PRs** after DQ #214 reject-finding (see §"Updates" below)
- **Parked work:** v1-ship-1 plan in parallel advisor session, awaiting approval gate
- **Strict gate:** all ~~6~~ **5** refactor PRs must merge to `governance-v0` before any v1 PRD planning resumes

## Updates

### 2026-05-14 — PR-3 dropped (audit-base error)

DQ #214 on `chore/refactor-eq-derive` (impl-raised, user-relayed via canonical advisor session):

Audit finding 3.C.1 prescribed adding `#[derive(Eq)]` to `GovernanceConfig`. The struct has `Option<f64> value_float` (intentional per schema CHECK constraint allowing one of int/float/bool/text non-NULL), which makes `Eq` structurally underivable (E0277: `f64: Eq` not satisfied). The audit treated `GovernanceConfig` as a parity outlier vs sibling governance models without validating that the prescribed one-line fix would compile.

**Decision:** reject the finding. Abandon `chore/refactor-eq-derive` (no commits to lose; branch was at trunk tip). Document audit-base error.

**Sibling outlier note:** impl pre-flight also found `crates/db_schema/src/source/governance/redaction.rs` missing `Eq` (cohort grep found 20, expected 21). Both outliers deserve **next-audit-cycle re-evaluation**, not action now.

**Cleanup:**
- Lane session deletes worktree `brehon-fork-refactor-eq-derive` and branch `chore/refactor-eq-derive`
- This canonical session removes PR-3 from the strict-gate list (now 5 PRs)
- `.claude/PRPs/briefs/refactor-eq-derive-*.md` files (bootstrap, bm-cut, impl) remain on trunk as audit-trail; do NOT delete

**Lesson candidate:** audit findings prescribing trait-derive additions should be validated against the struct's field types before promotion to `fix-before-next-phase` tier. Watchlist for next audit cycle.

## PR-to-finding map

| PR | Rank | Findings | Effort | Files touched | Strategy |
|---|---|---|---|---|---|
| PR-1 | 1, 2, 7, 8 | Case C error-type unification + 4-module fixture dedup + round-trip test split + named-migration revert | L (full day) | `crates/server/tests/e2e.rs` (single file, ~1500 lines edited) | Dedicated session, advisor-driven; mechanical sweep + structural reshape |
| PR-2 | 3 | TOCTOU race in `create_report.rs` (wrap SELECT-then-write in `run_transaction`) | M (half-day) | `crates/api/api_crud/src/governance/create_report.rs` | Dedicated session, advisor-driven; isolated single-file fix |
| ~~PR-3~~ | ~~11~~ | ~~Add `Eq` derive to `governance_config`~~ | **DROPPED** | n/a | **Rejected** via DQ #214 — audit-base error (f64 can't derive Eq); see §Updates |
| PR-4 | 6 | Pin `valid_from` on v0 + v1-AD-a seed migrations | S (30-120 min) | 2 migration files | **Parallel-lane** worktree |
| PR-5 | 13 | Propagate Diesel errors in `admin_audit_stream.rs:227,234` | S | `crates/api/api/src/governance/admin_audit_stream.rs` | **Parallel-lane** worktree |
| PR-6 | 17 | Add unit tests for `seed_founders::parse_founder_spec` | S | `crates/tools/seed_founders/src/main.rs` (add `#[cfg(test)] mod tests`) | **Parallel-lane** worktree |

## Topology — per-worktree-per-lane

Per `.claude/rules/multi-lane-worktree.md` to prevent the PMD #302 concurrent-session collision pattern.

### Canonical (governance-v0) checkout

`C:/Users/barri/Developer/brehon-fork` — this session. **Read-only during refactor execution.** Authors briefs, queues Junior bm-cut tasks, polls progress. Does NOT mutate any phase-branch DQ.

### Refactor worktrees (cut after bm-cut)

| Worktree path | Phase branch | Session | Concurrency |
|---|---|---|---|
| `brehon-fork-refactor-e2e` | `chore/refactor-e2e-error-types` | Dedicated advisor for PR-1 | Solo until PR-1 merges |
| `brehon-fork-refactor-toctou` | `chore/refactor-toctou` | Dedicated advisor for PR-2 | Solo until PR-2 merges |
| ~~`brehon-fork-refactor-eq-derive`~~ | ~~`chore/refactor-eq-derive`~~ | **DROPPED** (DQ #214 reject-finding) | n/a |
| `brehon-fork-refactor-valid-from` | `chore/refactor-valid-from` | Dedicated advisor for PR-4 | Parallel with PR-3/5/6 |
| `brehon-fork-refactor-diesel-errors` | `chore/refactor-diesel-errors` | Dedicated advisor for PR-5 | Parallel with PR-3/4/6 |
| `brehon-fork-refactor-seed-tests` | `chore/refactor-seed-tests` | Dedicated advisor for PR-6 | Parallel with PR-3/4/5 |

**Concurrent-session rules** (recap from PMD #302):
- One Claude Code session per worktree; never `cd` between worktrees in a single session.
- Phase-branch DQ entries (`validate-pending`, blockers) are mutated only by the lane-dedicated session for that phase.
- Briefs are authored on `governance-v0` from THIS session (canonical checkout); the lane-dedicated sessions just dispatch + monitor + mutate DQ.

### Sequencing

```
Phase 1 (this session, ~30 min): Author 6 bm-cut briefs + 6 impl-task briefs on governance-v0
Phase 2 (sequence-and-parallel):
  Step 2a (serial, ~1 day):  PR-1 e2e refactor in dedicated session
  Step 2b (serial, ~half day): PR-2 TOCTOU fix in dedicated session
                              (can start before PR-1 lands if no file overlap; verified: no overlap)
  Step 2c (parallel, ~half day): PRs 4, 5, 6 — 3 dedicated sessions, 3 worktrees, all running concurrently
                                  (PR-3 dropped via DQ #214 reject-finding)
Phase 3 (this session): All 5 PRs merged → trigger v1 PRD resumption per strict gate
```

PR-2 has zero file overlap with PR-1 (`create_report.rs` is in api_crud; PR-1 touches only `e2e.rs`). So PR-1 and PR-2 can ALSO run in parallel as 2 more worktrees. Doing this conservatively serial first to keep blast-radius small.

## Dependency analysis (verified no overlap across the parallel lanes)

- ~~PR-3: `crates/db_schema/src/source/governance/governance_config.rs` only~~ **DROPPED**
- PR-4: `migrations/2026-04-22-000300-0000_seed_v1_config_keys/` + `migrations/2026-04-18-000000-0000_add_governance_config/`
- PR-5: `crates/api/api/src/governance/admin_audit_stream.rs` only
- PR-6: `crates/tools/seed_founders/src/main.rs` only (adds `#[cfg(test)] mod tests`)

Zero file-path intersection across PR-4/5/6. Safe to run in parallel.

## Validation per refactor PR

Each PR runs the standard Shape-G validation chain:
- Worker push triggers `cargo-validate-workspace.yml`
- Migration touches (PR-4 only) also trigger `cargo-validate-migration.yml`
- e2e touches (PR-1 only) trigger Phase 2 e2e user gate per `feedback_phase_2_e2e_gate_enforcement.md` (PMD-indexed) — user picks local laptop run vs `gh workflow run`

Note: PR-1 is the only one that needs Phase 2 e2e. Others are non-e2e-touching → workspace check sufficient.

## Risk mitigations

- **Concurrent-session collision** (PMD #302): one worktree per lane; canonical checkout stays read-only during execution.
- **Bundled-attribution commits** (PMD #302 sub-lesson 1): each lane-dedicated session checks `git status` before any `git commit` to verify staged files match commit scope. Per the lesson's "How to apply" §1.
- **Citation breakage** (RT-r1 retro §5 watch-item 5 lesson): each refactor PR's body cites the audit finding ID (e.g. "Closes audit finding 3.E.1"). The audit itself stays unchanged on trunk; tier marks update post-merge.
- **Test regressions**: PR-1 is the highest-risk for test-stability fallout. Mitigation: run full e2e locally before opening PR-1 (~26 min, zero billed); only open PR after green local.

## v1-ship-1 plan handling

The parallel advisor session is parked on the v1-ship-1 plan-approval gate. That plan was authored by the parallel session as part of the v0-endpoint-coverage workflow before the user redirected to a code-quality audit.

**Decision for v1-ship-1**: defer plan approval until all 6 refactor PRs merge. The audit may surface (and the refactors may resolve) some of what v1-ship-1 was planning to address. After refactor tier ships, re-evaluate v1-ship-1 against the new trunk state.

Action: tell the parallel session to write a docs(advisor) note acknowledging the deferral; preserve the plan file on trunk; do not dispatch bm-cut for v1-ship-1 until refactor tier is clear.

## Resumption of v1 PRD work — after refactor tier merges

Once all 6 refactor PRs are on `governance-v0`:

1. Re-evaluate v1-ship-1 plan (parallel session output) against the refactored trunk.
2. Decide PRD ordering with user — candidates per audit §0 and v0-endpoint-coverage report:
   - v1-ship-1 (AGPL §13 + e2e backfill) per parallel session's planned work
   - v1-RT-r2 (critical-path: gates RT-r3 + RT-r5)
   - v1-JM-f, v1-RT-r4, v1-RT-r6 (parallel-eligible)
   - v1-admin-dashboard, v1-federation-inbound (status unknown; verify per §11 phase table)
3. First PRD planning Junior task must read `v1-code-quality-audit-2026-05-14.md` per audit §0 Step 6 hard rule.

## Open questions for user (before Phase 1 starts)

None as of 2026-05-14 — all decisions captured above. Proceeding to author the 6 briefs.

---

**Next concrete action:** I (this canonical-checkout session) author 6 bm-cut briefs + 6 impl-task briefs on governance-v0. Once committed, lane-dedicated sessions can be opened to dispatch each.
