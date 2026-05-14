# Retro: v1-RT-r1 — reputation-tuning substrate (post-ship)

**Date:** 2026-05-13 (post-ship; authored retrospectively — see §6 process miss)
**Sub-phase:** v1-RT-r1
**Plan:** `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` (11 tasks)
**PRD:** `.claude/PRPs/prds/v1-reputation-tuning.prd.md`
**Shipped:** PR [#126](https://github.com/barrie-cork/lemmy/pull/126) merged 2026-05-12 20:39 UTC → `governance-v0` at `1c8dc48a2`
**Phase branch (deletable):** `phase-v1-RT-r1` final tip `4a4ed354d` (pre-merge reconcile)
**Halt-retro:** `.claude/PRPs/reports/v1-RT-r1-halt-retro.md` (2026-05-11, mid-phase, `ffa2876e3`)

---

## §1 Outcome

**Shipped.** Substrate of the reputation-tuning lane — no handler emitters; r2/r3/r4/r5 land those.

| Deliverable | Commit | Notes |
|---|---|---|
| Task 1: `reputation_event` v1 columns + enum + partial unique index | `a153020a3` | Migration 2026-05-10-000000 |
| Task 2: `sponsor_allowlist` ALTER (community_id nullable + added_by_admin_id + note) | `b71781fb6` | Migration 2026-05-10-000100 |
| Task 3: backfill `reputation_event.source_event_type` via reason ILIKE | `488f4c743` | Migration 2026-05-10-000200; per DQ #184 |
| Task 4: seed 26 reputation-tuning `governance_config` rows | `4b259596e` | Migration 2026-05-10-000300; idempotent |
| Task 5: `ReputationEventSourceType` Rust enum (9 variants) | `5142dba54` | `crates/db_schema_file/src/enums.rs` |
| Tasks 6+7 (bundled): `schema.rs` extensions + Diesel structs | `03f6c670e` + `05cf5ae1d` | sql_types, table! changes, InsertForm |
| fix-impl-1 (E0063 pad): 2 callsites in `crates/api/api/src/governance/` | `d52f62124` | post-bundle compile fix |
| fix-impl-2 (E0063 pad): 8 callsites in api_crud + seed_founders + e2e | `cc11a91ab` | from halt-retro brief |
| fix-impl-3 (unused import): move into mod tests | `4952f88b0` | post-fix-impl-2 cleanup |
| Task 8: config.rs — 26 DEFAULT_* consts + SEEDED_KEYS + CONFIG_KEY_METADATA + parity | `198ab6d06` | as separate commit, not bundled |
| Task 9: 7 ENTRY_KIND consts (dual-file + registry) | `f2f9202b7` | as separate commit |
| Task 10: extend `phase1_migrations_round_trip` (14 → 18) | `bc2531346` | as separate commit |
| fix-impl-4 (CR cr-1+cr-2): ApplyAt::OnRestart variant + 10 metadata entries | `03f519f37` | CR triage |
| fix-impl-5 (CR cr-3): set `source_event_type = JuryVote` on emit | `69671f994` | CR triage |
| fix-impl-6 (CR cr-5): remove migration DEFAULT 1 sentinel | `3fd6f5877` | CR triage |
| Phase 1 workspace-checks | DQ #207, #208, #209, #210, #211, #212 | all `result=pass` after fix cycles |
| Phase 2 e2e | NOT raised as `validate-pending` | per phase commit log — appears to have been skipped |

**Counts:** ~30 phase-branch commits + 6 fix-impl cycles + 1 halt-retro + 12 ci-watcher cycles. From bm-cut `80e2e5d0c` (2026-05-10) to merge `1c8dc48a2` (2026-05-12) — **~52 hours wall-clock** (with halt-pause overnight).

---

## §2 Per-role signals

### Advisor

**Hits:**
- **Halt-retro called at the right moment.** Pausing Cohort B-final dispatch when the same isolation-validation bug class was about to recur (per ffa2876e3) prevented a 4th round of same-shape rework. Halt produced 5 durable lessons (L1-L5) shipped to rules + lessons before resume — and L1 (`requires:` field) plus L2 (serial ci-watcher) plus L3 (atomic raise-before-dispatch) plus L4 (multi-lane worktree) plus L5 (callsite enumeration) all hit in the resume cycle.
- **fix-impl-2 enumeration was complete and mechanical.** Brief listed 8 callsites by file:line; Junior #cc11a91ab patched all 8 in one task. L5 paid off immediately.
- **CR triage produced narrow fix-impl tasks.** cr-1+cr-2 bundled into fix-impl-4 (ApplyAt semantics fix); cr-3 → fix-impl-5; cr-5 → fix-impl-6. Three narrow fixes after the bundle, all clean.
- **Multi-lane discipline held.** Brief authoring on `governance-v0` (`610e538ea`), dispatch + DQ mutation on `phase-v1-RT-r1` worktree. Zero cross-lane race incidents in the resume cycle (vs 3 in the halt cycle).

**Misses:**
- **L1 (`requires:` field) did NOT prevent the Tasks 8+9+10 split.** Despite L1 being shipped and brief instructing "ONE commit, ONE workspace-check workflow", the bundle was actually committed as three separate `feat(v1-RT-r1):` commits (Tasks 8 → 9 → 10) then a fourth `chore(bm): merge tasks-8-9-10 bundle` workflow trigger. The workflow validated the combined tip, but the bundling instruction was not respected by Junior. Root cause: not investigated mid-phase; brief language was clear but Junior split the work. **Surface as a watch-item for the next bundled-commit task.**
- **fix-impl-3 was an unforced cycle.** fix-impl-2 left an unused `use ReputationEventSourceType` import (only used in `#[cfg(test)] mod tests`); workspace check failed on `-D warnings`. Pre-push `cargo check` on the fix-impl-2 worker branch would have caught it; brief did not require the worker to grep for new imports vs uses. **Add to §G4 brief authoring discipline: `rg "<NewType>" <touched-file>` upper-bound before commit.**
- **Phase 2 e2e was effectively skipped.** Plan §13 Task 10 says "Phase 2 e2e runs after Junior daemon finalize-merges into `phase-v1-RT-r1`" but no `validate-pending` entry with `cargo-test-e2e.yml` shows in DQ pending/resolved for RT-r1. CR review caught the substantive defects (ApplyAt semantics, JuryVote, DEFAULT 1) instead. **Watch-item: Phase 2 e2e dispatch is not happening automatically post-finalize-merge for migration+test phases.**
- **Task 11 (canonical phase retro before PR) was SKIPPED.** Plan §13 Task 11 + CLAUDE.md user gate 6 both require retro authored before `bm-pr`. Phase went bm-pr → CR → fix-impl-4/5/6 → bm-merge with no retro. This document is the post-hoc fix. **Root cause: no detection mechanism enforces the gate. Watch-item: pre-bm-pr gate must include "verify `.claude/PRPs/reports/<phase>-retro.md` exists with mtime > halt-retro mtime".**
- **DQ stale fails accumulated.** 5 historical-fail entries (#194, #203, #204, #206, #207) stayed in `pending[]` per option-2 semantics; bm-merge required a pre-merge reconcile commit (`4a4ed354d`) to resolve the DQ JSON conflict between phase + trunk. SL-c-2 retro called this out (lesson §3.1); same pattern repeated. **Mitigation candidate: bm-pr should sweep historical-fail validate-pending entries to `resolved[]` with explicit "superseded by PR merge" answer.**

### Planning

**Hits:**
- Plan §10 mirror skeletons (verbatim SQL + Rust) made every Junior task mechanical. Cohort A (Tasks 1-4) executed in ~4 min each despite being migration-class work.
- Plan §13 FILES YAML `creates: + modifies:` arrays caught the pre-bundle overlap correctly — no within-cohort file collision.
- Plan §5.2 complexity score (19/10) correctly identified RT-r1 as proceed-as-one rather than split-into-sub-phases.
- Plan §10.4 + §10.8 cumulative invariant accounting (V1_RT_NETNEW=26 + cumulative=127) correctly anticipated DQ #185/#187 dispute and resolved with audit-trail evidence pre-shipment.

**Misses:**
- **Plan did NOT specify Task 7 callsite enumeration.** Task 7 added two new fields to `ReputationEventInsertForm`; plan §11 "Files to change" listed `crates/db_schema/src/source/governance/reputation_event.rs` for Task 7 but did NOT list the 10 caller sites (sponsor_liability + submit_jury_vote + create_endorsement × 2 + seed_founders + e2e × 5). The 10 callsites were discovered AT compile-failure time, requiring fix-impl-1 (2 sites) and fix-impl-2 (8 sites). Plan-time `rg "ReputationEventInsertForm" crates/` would have surfaced the full set. **Promote to lesson:** `feedback_planner_enumerate_struct_callsites_for_addfield.md` — when a plan task adds a field to a public struct, plan §11 must enumerate all caller sites discovered via `rg`.
- **Plan §13 Task 7 brief scope was correct, fix-impl-1 brief scope was too narrow.** This was an advisor-side miss, not a planning miss — but the plan could have headed it off by listing all 10 sites in Task 7's `modifies:` array.
- **Plan §13 "Cohort B barrier. Sequential ordering required." marker.** L1 from halt-retro promoted the `requires:` field to the plan template, but RT-r1's own plan did not get retrofitted with that field. No incident from this miss (the Cohort B isolation was resolved at the brief layer), but **forward retrofit watch-item:** RT-r2/r3/r4/r5 plans must use the new `requires:` field.

### Impl (impl-task subagents)

**Hits:**
- Cohort A migrations (Tasks 1-4): mechanical SQL writing from §10 skeletons; 4 Junior tasks, ~4 min each. Once DQ id collisions resolved (advisor recovery c7ea59de3), workflow execution was clean.
- Task 5 (Rust enum), Tasks 6+7 (bundled), fix-impl-2 (8 callsites) all clean single-Junior-task executions.
- fix-impl-4 (multi-edit ApplyAt semantics) Junior correctly added the new enum variant + updated 10 metadata entries + ran the parametric reconciliation gate.

**Misses:**
- **Tasks 8+9+10 split despite bundle instruction.** Brief §2 was titled "BUNDLED (Tasks 8 + 9 + 10 in one Junior task, one commit)" and §6 commit message was explicit. Junior split into 3 commits with descriptive subject lines per task. Investigation deferred. **Watch-item:** confirm whether the Junior `impl-task` subagent prompt explicitly supports multi-objective bundles, or whether bundling needs a different role marker.
- **fix-impl-2 left unused import.** Junior padded 8 callsites correctly; the `use ReputationEventSourceType` import added during Task 7 was no longer needed in non-test scopes after the pad. fix-impl-3 cleaned up. **Mechanical:** add pre-push `cargo check` to fix-impl brief discipline OR add lesson: "after struct-field pad, grep new use-statements against new field-name occurrences."

### BM

**Hits:**
- bm-cut `45282da11` and bm-pr `ede9ffcdd` clean.
- bm-merge `e436d3fa5` handled the DQ JSON conflict via pre-merge reconcile (`4a4ed354d`) — clean recovery from the stale-fails accumulation.

**Misses:**
- Pre-merge DQ reconcile was reactive, not preventive. SL-c-2 retro flagged this same pattern. **Mitigation:** bm-pr brief should include the historical-fail sweep as a pre-PR step.

### ci-watcher

**Hits:**
- 12 ci-watcher cycles in this phase (DQ #189-#192, #195-#196 recovery + #199-#200 re-validate + #207-#212 mainline). Most resolved correctly per option-2 single-entry mutation.

**Misses:**
- **Resurrection bug (halt-retro DQ #189-#192, #195-#196).** 4 parallel ci-watchers off same phase tip → sequential finalize-merges → mutated entries reverted to pending. Documented as L2 (serial ci-watcher); applied successfully in resume cycle (post-#207 dispatched serially). **L2 working as designed in resume cycle.**

---

## §3 Lessons (durable — to promote)

1. **Plan §11 must enumerate all callsites when a task adds a field to a public struct.** Discovered via fix-impl-1+2 cycle on `ReputationEventInsertForm`. New lesson: `feedback_planner_enumerate_struct_callsites_for_addfield.md`. Complements L5 (advisor-side callsite enumeration at fix-impl brief time) by moving the enumeration upstream to plan authoring. **Promotion candidate.**

2. **Bundle instruction is not enforced by the impl-task subagent.** Tasks 8+9+10 brief specified ONE commit; Junior produced 3. Either the subagent prompt needs a "bundle: true" frontmatter that changes commit semantics, OR briefs should not assume bundling — should pre-author plan-task splits explicitly. **Investigation watch-item, not yet a lesson.**

3. **Phase 2 e2e dispatch did not fire automatically for RT-r1.** No `cargo-test-e2e.yml validate-pending` entry exists in DQ for the RT-r1 phase. CR review caught the substantive defects instead. **Watch-item:** verify auto-dispatch wiring for `cargo-test-e2e.yml` on `phase-v1-*` push; if intentional that e2e is advisor-driven for RT-class phases (no handler emitters → little e2e signal), document the carve-out.

4. **Phase retro gate (Task 11 / user gate 6) is not detection-enforced.** RT-r1 shipped without it; retro authored post-hoc. **Promotion candidate:** add a bm-pr brief pre-condition check `test -f .claude/PRPs/reports/<phase>-retro.md && [ "$(stat -c %Y .claude/PRPs/reports/<phase>-retro.md)" -gt "$(stat -c %Y .claude/PRPs/reports/<phase>-halt-retro.md 2>/dev/null || echo 0)" ]`. New lesson candidate: `feedback_phase_retro_gate_enforcement.md`.

5. **DQ stale-fails on phase branch cause bm-merge friction.** Pre-merge reconcile commit (`4a4ed354d`) needed to resolve DQ JSON conflict against governance-v0. Same pattern as SL-c-2 retro §3.1. **Promotion candidate:** add bm-pr brief instruction "sweep historical-fail validate-pending entries to resolved[] with `answer: superseded by PR merge` before bm-pr opens".

6. **Multi-lane worktree discipline (L4 from halt-retro) held in resume cycle.** Brief authoring on canonical `brehon-fork`, dispatch + DQ mutation on `brehon-fork-rt-r1`. Zero cross-lane race incidents in resume vs 3 in halt cycle. **Lesson already shipped; this is corroborating evidence.**

7. **Halt-retro pattern works.** Catching the recurring isolation-validation bug class mid-phase and shipping 5 rule/lesson changes BEFORE resume produced a clean resume cycle. Cost: ~1 day pause + retro authoring. Benefit: prevented at least 2 more cycles of same-shape rework. **Net positive. Use again when a recurring bug class is observed.**

---

## §4 Per-task complexity scores

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---|---|---|---|
| 0 (pre-flight) | 0 | 0 | ~5 | <1 |
| 1 (migration: reputation_event v1) | 2 | 1 + 1 DQ | ~4 | <1 |
| 2 (migration: sponsor_allowlist) | 2 | 1 + 1 DQ | ~4 | <1 |
| 3 (migration: backfill) | 2 | 1 + 1 DQ | ~4 | <1 |
| 4 (migration: seed 26 keys) | 2 | 1 + 1 DQ | ~4 | <1 |
| 5 (Rust enum) | 1 | 1 + 1 DQ | ~6 | <1 |
| 6+7 (bundled schema + structs) | 3 | 1 + 1 DQ | ~6 | <1 |
| fix-impl-1 (2 callsites) | 2 | 1 + 1 DQ | ~9 | <1 |
| **Halt-pause (overnight)** | n/a | n/a | **~16h** | n/a |
| fix-impl-2 (8 callsites) | 3 | 1 + 1 DQ | ~12 | <1 |
| fix-impl-3 (unused import) | 1 | 1 + 1 DQ | ~6 | <1 |
| Task 8 (config.rs 26 consts) | 1 | 1 + 1 DQ | ~10 | ~2 |
| Task 9 (governance-log dual-file + registry) | 3 | 1 + 1 DQ | ~8 | <1 |
| Task 10 (e2e extend) | 1 | 1 + 1 DQ | ~7 | <1 |
| fix-impl-4 (ApplyAt semantics, 10 entries) | 1 | 1 + 1 DQ | ~14 | ~3 |
| fix-impl-5 (JuryVote source_event_type) | 1 | 1 + 1 DQ | ~6 | <1 |
| fix-impl-6 (migration DEFAULT 1 removal) | 2 | 1 + 1 DQ | ~7 | <1 |
| **Advisor recovery work (halt cycle)** | 1 (decision-queue.json) | 7+ commits | **~4h** | n/a |
| **Phase total wall-clock** | — | ~30 phase commits + 6 fix-impl + bm-* | **~52h elapsed (incl. overnight)** | — |

**Headline:** mechanical impl-task work was fast (~4-14 min each, max log silence <3 min). Advisor wallclock was the bottleneck (4h recovery in halt cycle; halt-retro authoring added ~2-3h). Resume cycle was efficient (~6h from resume to merge).

---

## §5 Watch-items for next phase

- [ ] **Plan §11 callsite enumeration discipline** — when a planning task adds a field to a public struct, plan §11 lists all `rg`-discoverable callers in `modifies:` arrays. Author `feedback_planner_enumerate_struct_callsites_for_addfield.md`.
- [x] **Bundle-instruction enforcement** — RESOLVED 2026-05-13 via `feedback_bundle_means_one_worker_branch_not_one_commit.md` + impl-task brief template §8 Bundling appendix. Investigation finding: "bundle" canonical pattern is ONE worker branch + ONE workflow + N per-task commits. Junior's hard contract (impl-task.md L231 "one feature commit per plan task") outranks brief overrides. RT-r1 + earlier Tasks 6+7 bundle both produced per-task commits; brief asking for "one commit" was the outlier. No subagent prompt change needed.
- [x] **Phase 2 e2e auto-dispatch verification** — RESOLVED 2026-05-13. Investigation finding: `cargo-test-e2e.yml` is `workflow_dispatch`-only (intentional carve-out per 2026-04-28 minutes-budget audit). Empirical: 10/10 most recent runs are `event=workflow_dispatch`; last `event=push` run was 2026-04-28 JM-d. RT-r1 + SL-e both shipped without Phase 2 e2e (user gate 4 skipped) — procedural gap, not wiring bug. Encoded as: `feedback_phase_2_e2e_gate_enforcement.md` + bm-pr.md Phase 1c plan-aware gate. Future phases that touch `crates/server/tests/e2e.rs` are refused at bm-pr until a `validate-pending` entry with `result: pass` exists in resolved[].
- [ ] **Phase retro gate enforcement** — add pre-bm-pr file existence + mtime check on `.claude/PRPs/reports/<phase>-retro.md`. Author `feedback_phase_retro_gate_enforcement.md`.
- [ ] **DQ historical-fail sweep at bm-pr** — bm-pr brief instruction: resolve all `kind: validate-pending` entries with `result: fail` from the phase branch by adding `answer: "superseded by PR merge"`, `answered_by: "advisor"`, `resolved_at: <iso>`, move to `resolved[]`. Per SL-c-2 retro §3.1 + this retro §3.5.
- [ ] **Fix-impl pre-push cargo-check discipline** — fix-impl brief should include `cargo check --workspace --features full` (via the bat wrapper) as a pre-push gate to catch unused-import or downstream-deny-warnings issues. Avoids fix-impl-3-shape unforced cycles.
- [x] **PENDING: collapse to single-session** — attempted 2026-05-13, reverted; defer until concrete motivation. Phases B+C+D of the collapse-Junior plan landed (`16f9f4665`, `bcaa782d9`, `b648c11c2`) but were reverted (`5c0bac2d4`, `4aed5a51e`, `084300600`) after the migration revealed structural problems: ~132 citation paths across 24 rules + 12 agents + commands all need atomic update; concurrent-session race during execution produced bundled-intent commit `b004856df`. Junior model carried 5 lanes (SL-b through RT-r1) to retro without those costs. Trigger ("SL-b shipped + 4 lanes since") was satisfied 2026-05-05 but staleness alone is not motivation. Delta plan retained at `.claude/PRPs/plans/collapse-junior-2026-05-13.delta.md` as historical record. Revisit when (1) Junior friction produces a concrete next-phase blocker, or (2) one dedicated session can run the full B→I chain with no parallel writers. See `feedback_concurrent_advisor_session_collision.md` for the case study.
- [ ] **Halt-retro pattern documentation** — formalize the halt-retro vs phase-close-retro distinction. The halt-retro at `ffa2876e3` was load-bearing for this phase's recovery; promote the pattern.

---

## §6 Process miss — retro authored post-ship

Plan §13 Task 11 + CLAUDE.md user gate 6 both require the canonical phase retro to be authored BEFORE `bm-pr` opens. RT-r1 did not honor this: phase went `bm-pr` (`ede9ffcdd`) → CR review → fix-impl-4/5/6 → `bm-merge` (`1c8dc48a2`) with no retro.

This document is the post-hoc fix, authored 2026-05-13 from commit log + halt-retro + brief trail. Best-effort reconstruction, but **the live phase signals were lost** — no real-time per-task complexity wall-clock from each Junior task (estimated from commit timestamps), no advisor-during-CR-cycle observations recorded, no signal capture from the fix-impl-4/5/6 dispatches.

Surfaced as Watch-item §5.4 above. **Prevention:** add bm-pr pre-condition check for the retro file's existence + mtime.

Halt-retro at `ffa2876e3` is the closest live-signal capture for the first half of this phase. The resume-cycle signals are derived from this advisor session's brief commits and the rt-r1 worktree session's dispatch trail.

---

## §7 Sign-off

Author: governance-v0 advisor session (this session, on `C:/Users/barri/Developer/brehon-fork`).
Date: 2026-05-13.
Phase status: SHIPPED; branch `phase-v1-RT-r1` deletable on confirm.
