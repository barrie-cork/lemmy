# Retro: v1-SL-e — lane-wide e2e suite (revocation + window-expiry + backfill)

**Date:** 2026-05-13
**Sub-phase:** v1-SL-e
**Plan:** `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md`
**Phase branch tip (pre-PR):** `2df0afd8f` (post-RT-r1-merge + migration limit fix; DQ #200 phase-2 e2e pass)

---

## §1 Summary

**Shipped.** v1-SL-e is the **lane-closer** for the v1 sponsor-liability lane. Three lane-wide e2e tests prove the full producer→consumer path works as one coherent system.

| Task | Description | Commit | Phase-1 DQ | Phase-2 DQ |
|---|---|---|---|---|
| 0 | Pre-flight harness audit (Probes -1..18) | (no commit — verification only) | — | — |
| 1 | `revocation_during_window_escapes_full_lane` + `mod v1_sl_e_fixtures` shell + helpers | `5b1898051` | DQ #193 pass | DQ #194 pass (85 tests) |
| 2 | `window_expiry_fires_full_lane` (anchor-insert) | `842593ab6` | DQ #196 pass | DQ #197 pass (85 tests) |
| 3 | `backfill_of_mid_flight_v0_to_v1_deploy` + close `mod v1_sl_e_fixtures` | `6cf49ce50` | DQ #198 pass | DQ #199 **fail** → fix `2df0afd8f` → DQ #200 pass (88 tests) |
| 4 | Retrospective (this file) | this commit | — | — |

**Stories:** Stories 1 + 2 + 3 all completed (phase-2 e2e DQ #200: 88 passed, 0 failed, 3 ignored).

**Closes the v1 sponsor-liability lane** (modulo the restoration-during-window-escapes branch deferred to restorative-mechanics-v1 PRD per SL-c DQ #145). SL-lane-meta-retro follows after merge.

**Fix commit:** `2df0afd8f` — bump `revert_migrations` limit 8->12. Required because RT-r1 merged to `governance-v0` mid-SL-e, adding 4 new migrations; the old limit of 8 caused `v1_jm_a_backfill_populates_v0_snapshot` to fail on its DOWN/UP round-trip (DQ #199). Fix landed on `governance-v0 -> phase-v1-SL-e` merge commit `78d9ac539`.

---

## §2 Per-role signals

### Advisor

**What worked:**
- Three clarify-gate DQ entries (#190, #191, #192) before planning task dispatch caught three spec ambiguities: (a) direct-handler vs HTTP-layer invocation for SL-b revocation (resolved: direct-handler per SL-b e2e.rs:11190 precedent); (b) whether SL-d retro carry-forwards cr-3/cr-4 should fold into SL-e assertions (resolved: fold-into-assertions for cr-3; cr-4 deferred); (c) the source for the backfill UPDATE SQL (resolved: migration file verbatim). All three were self-resolved by advisor without user relay. Pre-planning clarify gate was justified.
- Shape G pipeline ran correctly for Tasks 1-3: each impl-task pushed, raised `validate-pending`, ci-watcher mutated, advisor ran phase-2 e2e locally. No manual per-task DQ intervention needed for the happy path.
- DQ #199 catch-fire (RT-r1 migration regression) was correctly classified as non-allowlist and surfaced to user. The fix was a narrow 1-line bump (`revert limit 8->12`) scoped to a test-harness constant, which the advisor authored directly as `fix(v1-SL-e)` commit.

**What surprised us:**
- **RT-r1 merge landed mid-SL-e**: `governance-v0` received 4 RT-r1 migrations (from PR #126 merge on 2026-05-12) while SL-e Tasks 2-3 were executing. The phase-branch had to absorb these via `78d9ac539`. This caused DQ #199 failure and one extra phase-2 e2e run (~30 min overhead).
- **Phase-2 e2e ran 4 times total** (DQ #194, #197, #199-fail, #200-pass) vs 3 planned. Each run ~30 min on laptop.

**What should change next:**
- When two lanes are executing concurrently, scan `gh pr list --repo barrie-cork/lemmy --state closed --json number,mergedAt,title` before each phase-2 e2e launch to detect whether any new PR merged to `governance-v0` since the last phase-branch sync. Diverged phase-branch = likely phase-2 e2e failure.

### Planning (Opus)

**What worked:**
- Plan quality: high. Tasks were concrete (file:line anchors, explicit mod placement, GOTCHA notes on `decided_at` UPDATE post-insert, env var safety with `prev_disable` pattern). The `mod v1_sl_e_fixtures` naming, anchor-Edit discipline for Tasks 2-3, and `LemmyResult<()>` Case A mandate all translated cleanly to impl.
- Complexity score 10 (plan §5.1) above the 8-threshold; planner DQ #190 self-resolved with a well-reasoned proceed rationale (e2e factor dominates; shared fixture helpers in one mod reduce duplication ~40% vs split). Advisor agreed. Result: Tasks 1-3 all ran in 1 commit each with no fix-impl cycles. The planner's self-resolve was vindicated.
- SL-d retro §5 carry-forwards (cr-3 + cr-4) were folded into the clarify gate (DQ #191) rather than reopened mid-planning. The clarify-before-plan discipline prevented mid-plan ambiguity discovery.

**What surprised us:**
- Nothing significant. Canonical Case A discipline (post-SL-c-2 amendment) successfully prevented any E0277 failures across all three tasks.

**What should change next:**
- The plan could have included an explicit watchpoint for "detect governance-v0 mid-phase merges before each phase-2 e2e launch" given that SL-e was concurrently executing with RT-r1. The `revert_migrations` limit would have been a natural §4 watchpoint.

### Impl (Sonnet)

**What worked:**
- Tasks 1-3: all landed on the correct file (`crates/server/tests/e2e.rs`) within scope. No production code contamination.
- All three tasks used `LemmyResult<()>` Case A shape throughout: `LemmyResult<T>` helpers, `LemmyResult<()>` test fn outer, bare `?`, zero `Box<dyn Error>`, zero `.map_err`. The 3-cycle catch-fire pattern from SL-c-2 was not repeated.
- Anchor-Edit discipline per `feedback_junior_worker_e2e_edit_hang.md` worked on a 13,854-line file (at Task 1 start). Task 1 appended 503 lines; Task 2 anchor-inserted 240 lines; Task 3 anchor-inserted 275 lines. No hang, no full-file Read.
- Task 1 HANDOVER trailer carried forward the canonical helper signatures and import list — Tasks 2-3 reused helpers without redeclaring.
- Task 1 proactively added `AsyncConnection` import (absent from SL-d Task 3 — the lesson that caused SL-d fix-impl-5). The lesson was applied.

**What surprised us:**
- DQ #195: Task 2 push failed twice with "HTTPS Broken pipe". Impl self-resolved after a background push succeeded on the third attempt. Overhead: ~50 min wall-clock between commit and validate-pending raise.

**What should change next:**
- Brief should include guidance for push-failure recovery: "on push failure, retry up to 2x; if still failing after 2x and the code committed, raise a DQ blocker immediately rather than waiting for more retries."

### BM (Haiku)

- bm-cut executed correctly: `phase-v1-SL-e` cut off `governance-v0` at `0b7616deb` (commit `0b7616debaf7bbad33adf49ae78dc0f316e04191`).
- No BM file-ownership breach detected; BM stayed within `.claude/`, `runlog/`, and DQ writes.
- bm-poll-cr and bm-pr pending (retro written before PR per `feedback_retro_not_report.md` GOTCHA).

### ci-watcher (Haiku)

- DQ #193, #196, #198: all mutated correctly on first ci-watcher run. No orphaned or double-mutated entries.
- No stop-hook/PMD isolation issues observed this phase (contrast with SL-d where DQ #195's mutation commit dropped due to stop-hook loop). Either the pattern improved or tasks were short enough to avoid the watchdog window.

---

## §3 Carry-forward (for v1-SL-lane-meta-retro)

1. **Lane-wide pseudonym-discipline coverage matrix**: SL-e Tests #1-#3 collectively assert ADR-015 pseudonym discipline on `sponsor_liability_pending`, `endorsement_revoked`, `sponsor_liability_escaped`, `sponsor_liability_applied`, `sponsor_liability_fired` payloads. A lane-meta-retro table should map each governance_log ENTRY_KIND to which sub-phase test covers its pseudonym assertion.

2. **Deferred-write semantics test pattern (negative+positive assertion pair)**: assert field is NULL before scheduler fires, then non-NULL after. Confirmed across SL-c-2 (5 tests), SL-d (unit test), SL-e (Tests #2 + #3). Promote to PMD as `feedback_deferred_write_semantics_test_pattern.md`.

3. **`BREHON_DISABLE_GRACE_CHECK_JOB` envelope + force-rewind `grace_expires_at` patterns**: Used in Tests #2 and #3. Pattern first appeared in SL-c-2. Promote to PMD as `feedback_force_rewind_grace_expires_at_test_technique.md` — canonical alternative to `tokio::time::sleep` for time-dependent scheduler tests.

4. **Open: restoration-during-window-escapes test plan**: Lives in restorative-mechanics-v1 PRD. The deferred branch (SL-c DQ #145 LOCKED) remains out of scope for v1-SL. The lane-meta-retro should ensure the restorative-mechanics-v1 PRD brief cites DQ #145 as the deferral source.

5. **`revert_migrations` limit tracking**: The RT-r1 merge mid-SL-e added 4 migrations, tripping the limit from 8 to require 12. At each sub-phase boundary that ships migrations, the advisor should check the cumulative migration count and bump the limit proactively.

6. **Multi-lane concurrency overhead**: SL-e executed concurrently with RT-r1 merge, adding 1 extra phase-2 e2e run (~30 min) and a fix commit. The lane-meta-retro should track total multi-lane overhead (DQ conflicts, mid-phase merges, additional e2e runs) as a cost model for concurrent lane execution.

---

## §4 Per-task complexity-score table

| Task | Files | Commits | Runtime (min, approx) | Max log-silence (min, est) | Watchdog risk |
|---|---|---|---|---|---|
| 0 (pre-flight) | 0 | 0 | ~13 | ~0 (bash probes only) | none |
| 1 (e2e #1: revocation + mod shell) | 1 (`e2e.rs`) | 1 | ~23 | ~8 | low |
| 2 (e2e #2: window-expiry) | 1 (`e2e.rs`) | 1 | ~25† | ~8 | low |
| 3 (e2e #3: backfill + close mod) | 1 (`e2e.rs`) | 1 | ~15 | ~6 | low |
| 4 (retro) | 1 (`v1-SL-e-retro.md`) | 1 | ~20 | ~0 (write-only) | none |

†Task 2 wall-clock was ~82 min (brief to finalize) due to DQ #195 push-failure recovery (~50 min overhead). Worker runtime (start to commit) was ~25 min.

**Median (impl tasks 1-3):** 1 file / 1 commit / ~23 min / ~8 min. All within the comfortable zone. No outliers; no watchdog risk. The planner's proceed-over-split decision (DQ #190) was correct.

**Plan §5 predicted:** complexity score 10 (above threshold), no watchdog risk expected. Actual: confirmed — no fix-impl cycles on test code, no E0277 catch-fires.

---

## §5 Lessons promotion

### New candidates

1. **`feedback_deferred_write_semantics_test_pattern.md`** — paired negative+positive assertion at producer vs consumer: assert deferred-write field is NULL before scheduler fires, non-NULL after. Confirmed across SL-c-2/SL-d/SL-e. Ready for PMD promotion.

2. **`feedback_force_rewind_grace_expires_at_test_technique.md`** — canonical alternative to `tokio::time::sleep` for time-dependent scheduler tests. Pattern: `diesel::sql_query("UPDATE moderation_case SET grace_expires_at = NOW() - INTERVAL '1 second' WHERE id = $1")...`. Used in SL-c-2 Tests #1-#5, SL-e Tests #2-#3. Well-confirmed.

3. **`feedback_revert_migrations_limit_cross_lane.md`** — when multiple lanes are concurrently shipping migrations, the `revert_migrations` limit in `v1_jm_a_backfill_populates_v0_snapshot` must be proactively bumped to `<total migration count>`. Check: `git log governance-v0 --oneline -- migrations/ | wc -l` before phase-2 e2e launch. The RT-r1 + SL-e concurrency caused a limit-8 failure on 12 migrations.

### Reinforced (already in PMD or lessons)

- `feedback_junior_worker_e2e_edit_hang.md` — anchor-Edit discipline confirmed across 3 anchor-inserts on a 14,000+ line file. No hang.
- `feedback_lemmy_error_no_std_error.md` (Case A) — all 3 tasks used Case A correctly; no E0277.
- `feedback_async_pool_test_pattern.md` — e2e fixture context pattern held throughout.

---

## §6 Acceptance (§17 checklist confirmation)

- [x] `rg -n 'mod v1_sl_e_fixtures' crates/server/tests/e2e.rs` -> 1 line (e2e.rs:13854).
- [x] `rg -n 'async fn revocation_during_window_escapes_full_lane|async fn window_expiry_fires_full_lane|async fn backfill_of_mid_flight_v0_to_v1_deploy' crates/server/tests/e2e.rs` -> 3 lines (e2e.rs:14013, :14357, :14596).
- [x] `git diff governance-v0..HEAD --stat` shows `crates/server/tests/e2e.rs | +1024` plus `.claude/` meta files. No production code, no migrations, no workflow YAML, no PRD/ADR/plan edits under `crates/**` (non-test).
- [x] DQ #200 (Phase-2 e2e): 88 passed, 0 failed, 3 ignored. All prior SL-a/b/c/d/JM tests preserved.
- [x] All 3 stories done:
  - Story 1: `revocation_during_window_escapes_full_lane` — confirmed (DQ #194 + DQ #200).
  - Story 2: `window_expiry_fires_full_lane` — confirmed (DQ #197 + DQ #200).
  - Story 3: `backfill_of_mid_flight_v0_to_v1_deploy` — confirmed (DQ #200 post-fix).
- [x] No new `ENTRY_KIND_*` consts shipped (SL-e is test-only).
- [x] No new migrations (`git diff governance-v0..HEAD -- migrations/` empty for impl commits).
- [x] Case A (`LemmyResult<()>` outer) throughout `mod v1_sl_e_fixtures`.
- [x] ADR-015 pseudonym discipline: all governance_log payload assertions use `is_string()` + `assert_ne!` against raw person_id (Tests #1-#3).
- [x] ADR-013 enum-exhaustiveness: no `match _ =>` wildcard in new test logic.
- [x] PRD §8.4 backfill SQL invoked verbatim in Test #3 (with comment citing PRD §8.4).
- [x] SL-lane-meta-retro flagged in §3 carry-forward (not authored here — separate artifact per plan GOTCHA).

---

*SL-e is the lane-closer. The v1-SL-lane-meta-retro follows after `bm-merge` ships this PR to `governance-v0`.*
