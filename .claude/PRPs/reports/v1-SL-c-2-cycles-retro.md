# Interim retro — v1-SL-c-2 Task 1 3-cycle catch-fire (mid-phase)

**Date:** 2026-05-09
**Scope:** v1-SL-c-2 Task 1 §G4 cycles 1-3 (workflow runs `25582548670`, `25595869651`, `25603848858`) and the resulting replan.
**This is interim** — covers the 3-cycle span only. The full v1-SL-c-2 sub-phase retro authors at end of c-2 ship and references this.

## §1 Wall-clock + per-role signals

### Wall-clock per cycle

| Cycle | Brief committed | Junior dispatched | Workflow done | Cycle wall-clock |
|---|---|---|---|---|
| 1 (impl-1) | ~22:55 UTC 2026-05-08 | #154 ~23:11 UTC 2026-05-08 (~16 min after brief) | 25582548670 ~17m | ~33 min total cycle |
| 2 (fix-impl-1) | ~05:00 UTC 2026-05-09 (advisor session) | #157 fail then #158 done at ~218s | 25595869651 ~17m | ~22 min done-time + ~6h advisor wall-clock waiting on resume cycles |
| 3 (fix-impl-2) | ~08:35 UTC 2026-05-09 | #160 ~10m | 25603848858 ~21m | ~31 min done-time |

**Aggregate:** ~3 cycles × ~25-30 min cargo wall-clock = ~80 min cargo-runner spend on the same compile error class. Plus advisor session resumes at each cycle boundary (5 sessions tracked in `auto-state.session_id` history; `resume_count: 5` post-replan). Plus the brief authoring + DQ mutation rounds at each cycle's start.

### Per-role signal

#### Planning role

**Signal: planner-side miss — plan §13 stub uniformity check is missing from gates.**

The v1-SL-c-2 plan §13 Task 1 stub prescribed `Result<(), Box<dyn Error>>` for the test fn AND `Result<T, Box<dyn Error>>` for the 4 helpers, contradicting the v1-SL-c-1 plan's §9.2 hand-off: *"the fixture-mod section [for `mod v1_sl_b_fixtures`] is non-binding for c-1 since c-1 ships no test code; it remains as documentation for c-2"*. The planner-side stub directly opposed the canonical sibling shape it was supposed to inherit.

**Why the plan-approval gate didn't catch it:** the DoD smoke test verifies §15 commands run, the watchpoint specificity gate verifies watchpoints cite specific tables/files, but neither gate cross-checks `§13 task stub return-type shape vs. canonical sibling fixtures module shape`. The advisor's session logs show plan approval at 21:49 UTC 2026-05-08 with no flag raised on the §13 stub.

**Carry-forward:** add a "stub-shape uniformity check across sibling fixtures modules" watchpoint OR require the planner to cite the canonical sibling line range in §13 task stubs that touch `crates/server/tests/e2e.rs`. Promote to PMD as `feedback_plan_stub_uniformity_with_canonical_sibling.md`.

#### Advisor role

**Signal: file-class lesson injection table fires the right lesson but doesn't verify the lesson body answers the question.**

The advisor's brief authoring at fix-impl-1 + fix-impl-2 correctly cited `feedback_lemmy_error_no_std_error.md` per the file-class table mandate (added at commit 558cef1a5, i.e. AFTER cycle 1 already failed — cycle 1's brief did not have the file-class table at the time it was authored, and that gap was the trigger for the rule).

But the lesson body itself was wrong-shaped: it prescribed exactly one recipe (`.map_err(|e| format!("{e}").into())` against `Result<(), Box<dyn Error>>` outer) and did NOT enumerate the case A / case B / case C distinction. So injecting the lesson into §3 of the fix-impl briefs gave Junior the wrong recipe twice in a row. Cycle 3's brief copied the recipe verbatim per the anti-paraphrase gate — and the recipe text itself failed E0283.

**Why the advisor missed it:** at fix-impl-1 brief authoring time, the advisor had no signal that "the lesson body might be too narrow for this code shape". The PMD search returned the lesson; the file-class table mandated it; the §G4 row cited it; nothing flagged that the lesson's recipe presumes a concrete outer error type rather than an abstract trait object.

**Carry-forward:** the lesson amendment (cases A/B/C) closes this. The §G4 row 4 split (4a/4b/4c) makes case selection mechanical at classification time. The file-class table parenthetical adds the canonical-sibling mirror as a structural correctness check.

**Second advisor signal: brief authorship at fix-impl-1 inverted the canonical recipe even while citing the lesson.** This was the original cycle-2 surfacing per `.claude/PRPs/debug/rca-sl-c-2-fix-impl-1-wrong-recipe.md`. The anti-paraphrase gate (commit 558cef1a5) closes this specific class of lapse going forward; cycle 3 demonstrated the gate works for paraphrase but does not detect "the canonical text is itself insufficient".

#### Impl role

**Signal: Junior workers followed each brief faithfully. Not at fault for any cycle.**

- Junior #154 (impl-1) — implemented Task 1 per plan §13 stub verbatim. No deviation.
- Junior #158 (fix-impl-1) — flipped test fn signature to `LemmyResult<()>` per brief. Did NOT touch helpers per brief explicit hard refusal. Followed instructions. The brief was wrong, not Junior.
- Junior #160 (fix-impl-2) — applied `.map_err` bridges at 5 Lemmy-native call sites verbatim per brief §G4 row recipe. Followed instructions. The recipe was wrong, not Junior.

**Junior daemon issues that surfaced (not cycle-causing but cycle-adjacent):**

- ci-watcher Juniors #155, #156, #157 failed on `git worktree add` with "fatal: invalid reference: junior/role-impl-task-...-md-154" because daemon-side local `refs/heads/junior/*` were not auto-fetched. Mitigated by manual force-fetch on the daemon. Lesson-candidate already in `auto-state.lesson_candidates_for_retro[]`.
- Daemon worker-branch creation: when fix-impl-2 was queued with `base_branch=impl-1 worker branch`, daemon created a NEW worker branch (md-160-suffixed) instead of fast-forwarding on impl-1's branch. Documented behavior, not a bug — lesson-candidate already in retro array.

#### BM role

Not active for v1-SL-c-2 yet (still in impl-cohort phase). Skip.

## §2 Root causes (ordered by load-bearing weight)

### RC-1 — Lesson recipe family was too narrow

**Pre-amendment lesson body:** "Mechanical fix: `.map_err(|e| format!("{e}").into())` on every Lemmy-native call." Single recipe, no case enumeration.

**Reality:** the recipe works only when the outer error type is concrete (e.g. a crate-local error enum at Phase 2b, where `From<String> for E` is unambiguous). When outer is abstract `Box<dyn Error>`, type inference fails. When the module mixes shapes, `?` propagation fails on Send/Sync/Sized cascades.

**Fix:** lesson body amended to enumerate cases A (uniform `LemmyResult<T>`, preferred), B (uniform `Box<dyn Error>` with annotated closures), C (mixed shapes — hard refusal). §G4 row 4 split into 4a/4b/4c per case.

### RC-2 — Plan §13 stub uniformity check was missing from gates

The plan §13 Task 1 stub prescribed Case C (mixed shapes) without realizing it. No gate checked the stub against the canonical sibling.

**Fix:** plan §19a carry-forward note added; lesson candidate for `feedback_plan_stub_uniformity_with_canonical_sibling.md`; future watchpoint specificity gate amendments.

### RC-3 — Advisor brief authorship at fix-impl-1 inverted canonical recipe

Already addressed by the anti-paraphrase gate at commit 558cef1a5 (fix-impl briefs MUST contain verbatim §G4 row block-quote in §2 before any file:line context). Did not prevent cycle 3 because cycle 3's row text was insufficient (RC-1).

**Fix:** anti-paraphrase gate stays; case enumeration in lesson + §G4 row split makes the verbatim text correct for the case at hand.

## §3 Carry-forward to full sub-phase retro

- Tasks 2-5 §13 stubs use `LemmyResult<()>` per c-1 §9.2 hand-off — verify at retro time by grep.
- Per-task complexity score for Task 1 in retro: max-cycle-count × per-cycle wall-clock, NOT effective LOC delta. The replan commit's LOC delta is small (~6 lines) but the real cost was 3 cycles + 5 advisor session resumes + 4 push grants.
- Lesson candidates promoted to PMD this retro:
  - `feedback_lemmy_error_no_std_error.md` (amended in place — not a new lesson; lesson re-sync after this commit lands).
  - `feedback_plan_stub_uniformity_with_canonical_sibling.md` (NEW — first surfacing).
  - The §G4 row 4 split + file-class table parenthetical are rule changes, not lesson promotions, but the rule changes' net effect should appear as a watch item: "next sub-phase that hits an E0277 LemmyError class — does §G4 row 4a/4b/4c routing pick the right rule? If yes, the case enumeration was successful."

## §4 Per-task complexity score (Task 1)

Per `feedback_retro_task_complexity_score.md`:

- **Files touched:** 1 (e2e.rs, post-replan)
- **Commits:** 1 (post-replan; pre-replan had 4 commits: impl-1 + fix-impl-1 + fix-impl-2 + 4 DQ raise/mutate, all wiped by hard-reset)
- **Runtime per cycle:** ~17-21 min (cargo-validate-workspace cold-cache on GH Actions)
- **Max-log-silence per cycle:** ~17-21 min (the cargo workflow is opaque from advisor's perspective until completion)
- **Cycles:** 3 catch-fired + 1 replan dispatch pending = 4 attempts on one task
- **Effective complexity (post-replan retroactive):** ~17 min cargo × 4 attempts = ~68 min direct cargo + ~30 min advisor brief authoring + ~5 min × 5 session resumes = **~123 min total task wall-clock for one ~6-line code change**.

Score: **9/10 complexity** (LOC says 1, cycle count + wall-clock says 9). The retro template should treat cycle-multiplier as a first-class signal, not an exceptional case.

## §5 Watch items (for next 2 sub-phases)

1. **Does §G4 row 4a/4b/4c routing pick the right case the first time on a fresh E0277?** If next E0277 catch-fire still requires a re-plan, the case enumeration is insufficient and needs another iteration.
2. **Does any future plan §13 stub for an e2e fixtures module match the canonical sibling shape on first pass?** If a planner still produces a wrong-shaped stub, escalate to a hard refusal in the planning gate.
3. **Does the canonical-sibling-mirror parenthetical on the file-class table get cited in the next e2e impl-task brief?** If yes, the rule landed; if no, the file-class table edit didn't propagate.
4. **Lesson PMD re-sync after this retro:** `bash scripts/sync-lessons-to-pmd.sh` should pick up the amended lesson body. Verify with `memory_search` for `LemmyError` returns the case-enumeration body, not the pre-amendment body.

## §6 Deferred to full c-2 retro

- Aggregate wall-clock for v1-SL-c-2 (Task 0 → Task 5 → bm-pr → cr → merge → retro).
- Net token spend across all advisor session resumes for c-2 (resume_count post-replan = 5+; full ship will have more).
- /auto-phase skill effectiveness: did the skill's cadence cut wall-clock vs c-1 baseline? Did the §G4 catch-fire surface vs auto-recover correctly?
- Comparison to c-1: c-1 shipped in 4h 21min with ~15 user touchpoints; how does c-2 compare given the 3-cycle detour?
