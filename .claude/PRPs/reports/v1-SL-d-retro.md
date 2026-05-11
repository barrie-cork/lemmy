# Retro: v1-SL-d — compute/fire split + submit_jury_vote pending transition + e2e/unit tests

**Date:** 2026-05-11
**Sub-phase:** v1-SL-d
**Plan:** `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md`
**Shipped:** `governance-v0` tip `6ae789d1d` (PR #123 merged)

---

## §1 Outcome

**Shipped.** Two halves delivered:

### Half 1 — `apply_sponsor_liability` compute/fire split
| Task | Description | Commit | Phase-1 DQ |
|---|---|---|---|
| 0 | Pre-flight harness audit | (no separate commit — pre-flight passed) | — |
| 1 | Split `apply_sponsor_liability` → `compute` + `fire` + wrapper | `9825a2d8a` | DQ #195 pass |
| fix-impl-1 | Suppress `dead_code` on Task 1 helpers | `cb8df2176` | — |
| fix-impl-2 | Replace `allow` with `expect` dead_code | `3302ceb75` | — |
| fix-impl-3 | Remove spurious `expect dead_code` on severity helper | `e889a8af7` | — |

### Half 2 — `submit_jury_vote` `Decided → SponsorLiabilityPending` transition
| Task | Description | Commit | Phase-1 DQ |
|---|---|---|---|
| 2 | `submit_jury_vote` pending transition + grace-window snapshot | `ad519f1ad` | DQ #196 pass |
| fix-impl-4 | Replace `expect()` with `ok_or_else` in Pending path | `9fcb2f381` | — |

### Half 3 — e2e tests + unit tests
| Task | Test fn | Commit | Phase-1 DQ | Impl runtime |
|---|---|---|---|---|
| 3 | `submit_jury_vote_transitions_decided_to_sponsor_liability_pending` | `aa4ee3238` | DQ #195 (shared) | ~18 min |
| 4 | `submit_jury_vote_no_sponsor_path_preserves_v0_immediate_decided` | `94c7f91aa` | DQ #196 (shared) | ~10 min |
| 5 | `submit_jury_vote_no_action_skips_liability_machinery` | `4e42830d3` | DQ #197 pass | ~9 min |
| fix-impl-5 | Add missing `AsyncConnection` import | `1c51bfb53` | — | — |
| 6 | `apply_sponsor_liability_wrapper_preserves_v0_outputs` | `83c811282` | DQ #198 pass | ~19 min |
| 7 | Unit tests: `compute_sponsor_liability_idempotent` + `grace_window_for_severity_reads_correct_config_key` | `6546b119e` | DQ #199 pass | ~10 min |

**Phase-1 workspace-checks:** DQ #195–#199 — all `result=pass`.
**Phase-2 e2e:** DQ #189 — 85 passed, 0 failed, 3 ignored (local laptop, run 2, log `e2e-v1-SL-d-78771349e-run2.log`).
**PR:** #123 opened 2026-05-10, CR reviewed, triage approved 2026-05-11, merged `6ae789d1d`.

---

## §2 Per-role signals

### Advisor

**What worked well:**
- Shape G pipeline ran cleanly for Tasks 3–7: each impl-task pushed, raised `validate-pending`, ci-watcher mutated, advisor advanced. No manual intervention per task after the fix-impl cycle.
- Triage inline (advisor-side gate, L15 fix): rebut/carry-forward split was clear with 0 criticals and 0 fix-in-pr blockers.
- Phase 2 e2e gate on correct branch (run 2): after wrong-branch incident on run 1 (`phase-v1-RT-r1` HEAD), run 2 explicitly verified SHA `78771349e` before launching. Lesson reinforced.

**Issues:**
- **ci-watcher #214 DQ mutation miss**: Task 7's ci-watcher mutated `result=pass` for workflow `25637289127` but never committed the DQ mutation. Advisor manually resolved via worktree direct edit + finalize-merge. Root cause: stop-hook blocking loop consumed all retries before the commit step.
- **bm-poll-cr #222 stop-hook loop**: Worker produced 9+ retro evals but hook couldn't locate them (PMD query isolation between bash hooks and MCP tools — known system issue). Task was functionally complete; findings YAML committed to worktree branch. Advisor manually finalize-merged.
- **Phase 2 e2e run 1 wrong branch**: Laptop was on `phase-v1-RT-r1` detached HEAD when cargo-test.bat launched. SL-d test fixtures absent. Root cause: `git checkout origin/phase-v1-SL-d` left detached HEAD and subsequent governance-v0 operations obscured the state. Fix: always verify `git rev-parse --short HEAD` before launching e2e.
- **DQ #194 stale pending from RT-r1**: Found during SL-d bm-merge gate scan. Migration validation fail for RT-r1 Task 3, fully answered by ci-watcher but never migrated to resolved. Cleaned up.
- **EliteDesk governance-v0 stray merge commit**: A merge-governance-v0-into-phase-v1-SL-d commit landed on governance-v0 on the EliteDesk. Reset to `origin/governance-v0` before queueing bm-merge.

### Planning (Opus)

- Plan quality: high. §13 tasks were concrete (file:line anchors, explicit mod placement, GOTCHA notes on `apply_sponsor_liability` visibility via `run_grace_check_batch`). The `pub(crate)` visibility GOTCHA for Task 6 was pre-called in the brief and plan — no impl surprise.
- Complexity score §5.1 was above threshold but planner self-resolved with correct proceed rationale (e2e tasks are serial anchor-Edits, not parallel; no meaningful reduction from splitting).
- Fix-impl briefs were authored correctly with verbatim §G4 blockquotes for allowlist matches.

### Impl (Sonnet)

- Tasks 1–7: all landed on correct files within scope. No crates/ contamination outside the owned files.
- Tasks 3–6 e2e tests: all used `LemmyResult<()>` Case A shape per the `feedback_plan_stub_uniformity_with_canonical_sibling.md` lesson — the 3-cycle c-2 catch-fire was avoided.
- Task 7 unit tests: used `AsyncPgConnection::establish` + `DATABASE_URL` skip pattern correctly; no testcontainer involvement.
- Fix cycle overhead: 5 fix-impl commits for Tasks 1–2 (dead_code/expect), 1 for Task 3 (missing import). Reasonable for a handler-level refactor.

### BM (Haiku)

- bm-cut, bm-pr, bm-poll-cr all executed without major issues.
- bm-poll-cr #222 stop-hook issue is a recurring system problem (PMD isolation) — not BM logic error.
- bm-merge executed cleanly: L14 git sequence followed (runlog commit before `gh pr merge`), branch deleted.

### ci-watcher (Haiku)

- DQ #195–#198: all mutated correctly on first run.
- DQ #199 (Task 7): mutation completed but commit step dropped due to stop-hook loop. Pattern matches the known PMD isolation issue.

---

## §3 Per-task complexity table

| Task | Files | Commits | Runtime (min, approx) | Max log silence (min) |
|---|---|---|---|---|
| 1 (split) | 1 (`sponsor_liability.rs`) | 4 (1 impl + 3 fix) | ~25 | ~8 |
| 2 (submit_jury_vote) | 1 (`submit_jury_vote.rs`) | 2 (1 impl + 1 fix) | ~20 | ~8 |
| 3 (e2e #1) | 1 (`e2e.rs`) | 1 | ~18 | ~10 |
| 4 (e2e #2) | 1 (`e2e.rs`) | 1 | ~10 | ~6 |
| 5 (e2e #3) | 1 (`e2e.rs`) | 1 | ~9 | ~5 |
| 6 (e2e #4) | 1 (`e2e.rs`) | 1 | ~19 | ~10 |
| 7 (unit tests) | 1 (`sponsor_liability.rs`) | 1 | ~10 | ~6 |

---

## §4 Lessons (new or reinforced)

1. **Verify branch before e2e launch (reinforced)**: `git rev-parse --short HEAD` + `grep -c <test_fn>` before every local e2e run. Two incidents in two sub-phases (RT-r1 + SL-d) confirm this is a load-bearing check, not a suggestion.

2. **ci-watcher stop-hook/PMD isolation (reinforced)**: When a ci-watcher task writes evals but shows `running` past expected completion, check task logs for "memory_write_eval" calls + stop-hook blocked entries. Mutation is likely complete; finalize-merge manually. This pattern appeared in SL-c-2 and SL-d.

3. **bm-poll-cr stop-hook loop (same root cause as ci-watcher)**: Functional completion (YAML committed, pushed) can be confirmed by reading task logs even when the task shows `running`. Advisor manually finalize-merges to pick up the artifact.

4. **Stale DQ entries in pending from prior phases**: At merge gate, always scan all pending entries — not just the current phase's. DQ #194 (RT-r1) was a false positive that added noise.

---

## §5 Watch items for next phase

- **cr-3 (carry-forward):** `e2e.rs:13128` — post-quorum idempotency on Pending path. File as brief anchor for next SL sub-phase.
- **cr-4 (carry-forward):** `e2e.rs:13490` — sanction table stays empty for NoAction. Same.
- **PMD stop-hook isolation**: Recurring pattern across ≥3 sub-phases. Worth surfacing as a project memory pattern if it recurs in SL-e.
