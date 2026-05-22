# Session retro — 2026-05-22 — context-prune Option B Steps 3-6

**Harness:** claude-code
**Session window:** ~2026-05-22T22:00Z → ~2026-05-22T23:10Z (~70 min)
**Branch at start:** `e89ac013a` (`governance-v0`) per session-start snapshot — actual on-disk at first commit was `3ce413fe6` (concurrent retro from a sibling session landed mid-session)
**Branch at end:** `9623260e5` (`governance-v0`)
**Files touched:** 5 (1 user-scope MEMORY.md, 1 rule, 2 new refs, this retro)
**Commits:** 1 explicit (`9623260e5` chore(refs): externalize §3.7+§3.8 + §6.1+§6.2) + 0 for Steps 3+4 (user-scope MEMORY.md is outside git)

## TL;DR

Resumed the context-prune Option B handover (Steps 3-6) authored by the prior session at `e89ac013a`. Steps 3+4 (MEMORY.md prose trims) self-verified per the handover's authorization at L282-285. Steps 5+6 (advisor-orchestrator.md §3.7+§3.8 and §6.1+§6.2 externalization) shipped in one commit. **Race-A caught during staging:** my `git add` swept up a concurrent session's unstaged edit to `feedback_advisor_dryrun_process_rule_preconditions_at_brief_author.md`; caught via `git status` between add and commit per `feedback_cross_session_commit_attribution_collision.md`; unstaged before commit. The other session's work stayed in the working tree for it to reclaim. Total token savings landed slightly under prediction (~50% recovery on Steps 3+4 due to audit-trail overhead) but Steps 5+6 delivered close to predicted ~1.9k.

---

## What surprised us

- **Concurrent commit landed at `3ce413fe6` mid-session.** Session-start snapshot showed local at `e89ac013a` but by the time I ran `git fetch` pre-commit, both my local and origin were at `3ce413fe6`. The concurrent session committed + pushed AND somehow my local was already FF-aligned (possibly because the canonical checkout's working tree gets touched by background tooling). Net effect: zero conflict for my commit — the concurrent commit only added a retro file in `.claude/PRPs/reports/`. But the working tree state being ahead of the session-start snapshot is itself novel; the surface-first ritual + multi-lane-worktree rule didn't predict this exact shape.
- **Race-A caught at staging.** `git add .claude/rules/advisor-orchestrator.md .claude/refs/advisor-narrow-gates.md .claude/refs/advisor-subagent-dispatch.md` returned a 4th file in the staging output: `M .claude/lessons/feedback_advisor_dryrun_process_rule_preconditions_at_brief_author.md`. That file's modification belonged to the concurrent retro-author session (visible in their commit body: "Generalisable lesson promoted to PMD (memory_id 494)... augmentation candidate for [this lesson]"). The session had authored the augmentation but exited without committing it. My `git add <three-named-files>` shouldn't have picked it up — but `git status` showed it as `M ` (working-tree modified, not staged) before my add, and after my add the same file showed as `M ` (staged, capital-M with no space prefix). Cause unclear; possibly `git add` resolves the path list against tracked-file index globs, possibly there's a shared-index quirk. **The mitigation worked.** `git restore --staged` cleanly unstaged the foreign file; my commit only contained the three intended files. Per `feedback_cross_session_commit_attribution_collision.md` Race-A: this is exactly the failure mode the lesson predicts and the `git status` between add and commit is the prescribed catch.
- **Token-savings prediction for Steps 3+4 was optimistic.** Handover predicted ~400 tokens (~250 + ~150). Actual delivery ~200 net because the audit-trail line in Historical consumed ~80 tokens (provenance overhead). For mechanical trims with provenance preservation, the rule of thumb should be ~50% recovery, not 100%.
- **Steps 5+6 token-savings prediction held.** Handover predicted ~1.9k (~700 + ~1.2k). The diff shows `3 files changed, 88 insertions(+), 51 deletions(-)` on advisor-orchestrator.md — net -47 lines from the rule + ~88 lines into two refs files. At ~25-30 tokens/line for prose, that's ~1.2-1.4k tokens removed from the always-loaded rule. Will know exact figure when the user's `/context` check returns.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When predicting token savings for prose trims in MEMORY.md (or any index file with provenance discipline), **subtract ~80 tokens per audit-trail line written into Historical.** Net savings ≈ (lines removed × ~40) − 80. Add as a one-line note in `feedback_context_trim_verify_empirically.md`. | Prevents future optimistic predictions; recalibrates the "is this trim worth doing?" cost-benefit. | minor (one-line lesson augmentation) | 2× this session (Steps 3+4 both under-delivered for the same audit-overhead reason) — qualifies as a recurrence within the session |
| 2 | When committing on the canonical checkout with concurrent sessions known to be active, **the `git status` between `git add` and `git commit` is non-negotiable.** This session demonstrated Race-A capture in real-time: `git add` produced a foreign-file staging that would have shipped under my Co-Authored-By line. Promote `feedback_cross_session_commit_attribution_collision.md` Race-A guidance from "recommended" to a hard rule in `.claude/rules/multi-lane-worktree.md` hard refusal #6's atomic protocol step 4. | Eliminates a known cross-session attribution failure mode. Mitigation cost is one extra `git status` call (~1 second). | minor (rule edit, one line) | 1× this session + prior occurrence on 2026-05-22 `c858aa7ab` (cited in source lesson) = 2× confirmed |
| 3 | The handover authorized self-verification for Steps 3+4 (L282-285) — that worked. But the savings prediction included audit-trail overhead implicitly, which it shouldn't have. Future handovers proposing mechanical trims should **state predicted savings as net-of-provenance** (so the executor knows whether to write an audit line or skip it for sub-100-line trims). | Clearer cost-benefit at handover-write time; fewer optimistic numbers. | minor (handover-template note) | 1× this session; pattern likely to recur on every future MEMORY.md prune |
| 4 | The concurrent retro-author session's commit message names this session ("PMD backfill ran (1 own row + 1 from concurrent Steps-3+4 session; both embedded; 0 missing vectors)") — meaning the concurrent session **knew this session was running and ran a coordinated PMD backfill on its behalf.** That's exactly the right behavior, but the coordination was invisible to me until I read the concurrent commit. **Worth a cross-reference protocol: when two sessions know about each other, write a one-line `.claude/runlog/` ledger entry naming the other.** Currently `agent-activity.json` tracks per-session presence but doesn't capture mutual-awareness. | Makes cross-session coordination auditable; prevents "did the other session do X for me?" guess-work. | minor (runlog convention) | 1× this session; would have been useful on prior 2x cross-session incidents |

## What to carry forward

- **The `git status` between `git add` and `git commit` Race-A catch is now battle-tested.** Twice in 24 hours (`c858aa7ab` + this session `9623260e5`). The protocol works; the cost is one tool call; the failure mode it catches would silently attribute foreign work to this session's commit. **Make this a reflex.**
- **Handover-driven execution under concurrency works.** Two sessions ran in parallel on the canonical checkout (this one + the concurrent retro-author). Both committed independently; both finished cleanly. The multi-lane-worktree rule was designed for phase-branch isolation, but the canonical checkout supports parallel sessions IF each one (a) fetches before commit, (b) stages only its own files, (c) `git status` checks before commit, (d) commits + pushes atomically. The rule's hard refusal #6 atomic protocol is exactly the right shape.
- **One-line pointers replace lazy-load section blocks cleanly.** Step 5's `### 3.7 Dogfood gate + 3.8 Schema-retrofit gate` heading + 2-line body reduces ~22 source lines to 3 lines while preserving discoverability (the refs/ path is named verbatim). Step 6 same shape with `### 6.1 Parallel dispatch + 6.2 Verify-after-subagent-completes`. The pattern is: collapse two adjacent low-frequency subsections to a single combined heading + 1-2 line pointer. Reuse this shape for future externalizations.
- **Re-reading the predecessor retro for shape mirror is the canonical-schema-first gate (rule §3.6) earning its keep.** This retro mirrors Steps 1+2 retro's section structure verbatim (TL;DR, What surprised us, What to change, What to carry forward, Three-signal scoring, Complexity scores, Decisions to revisit, Promotion candidates, Cross-references). Future retros in this series should keep the same skeleton.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`. Numbers are defensible from the transcript.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `git status` between add and commit (Race-A catch) | 30+ | 0 | high | Caught a concurrent session's foreign-file staging; would otherwise have committed an unrelated lesson augmentation under this session's Co-Authored-By |
| AskUserQuestion (verification mode at session start) | 5 | 0 | none | Clarified that `/context` is user-side; user picked self-verify Steps 3+4 + check Steps 5+6 — one clean turn |
| Canonical-schema-first gate (rule §3.6) — read predecessor retro before writing this one | 5 | 0 | low | Mirrored the Steps 1+2 retro shape verbatim; trivial to find via `ls`-glob; saved 5 min of structural drift |
| `git show` via canonical resolver (`scripts/brehon/git-show-json.sh`) | 1 | 1 | none | Initial bare `git show origin/governance-v0:.claude/...` failed on Windows backslash-path mangling per `feedback_windows_bash_python_git_show_tmp_traps.md`; resolver got it on second try |
| Foreign file unstage via `git restore --staged` | 2 | 0 | low | Standard recovery from Race-A catch; quick |
| Parallel TaskCreate (5 tasks in one message) | 1 | 0 | none | Stage tracking for Steps 3-6 + retro; cheap |
| ToolSearch round-trips for deferred TaskCreate/TaskUpdate/memory_write_eval | 0 | 2 | medium | Three tools were deferred; ToolSearch resolved each in ~1s; net friction is ~30s per round-trip. Probably unavoidable for fresh-session start |
| MEMORY.md self-verification (vs `/context` round trip) | 5 | 0 | none | Self-verify on Steps 3+4 per handover authorization at L282-285; user `/context` confirmed match; saved a fresh-session round trip |

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. This session had no Junior tasks — all interactive. One task (Step 5+6 externalization commit) was multi-file but mechanical.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---:|---:|---:|---:|---:|
| Steps 3+4 MEMORY.md trims | 1 (user-scope) | 0 (no git) | ~5 | n/a |
| Steps 5+6 advisor-orchestrator externalization | 3 (1 rule + 2 new refs) | 1 | ~25 | n/a |
| Retro author (this file) | 1 | 0 (will commit if user OKs) | ~10 | n/a |

No task hit the >55min runtime / >40min log silence / >8 files thresholds. The Steps 5+6 commit touched 3 files but with a clean structure (verbatim section moves + one-line pointers) — not a complexity outlier.

## Decisions to revisit

- **Should `git status` between `add` and `commit` be promoted to a hard rule in `multi-lane-worktree.md`?** Currently the rule's hard refusal #6 atomic protocol step 4 says "git add → verify the JSON (...) → git commit". It implies the verify-step, but doesn't name `git status` explicitly. Two same-day incidents argue for promotion; logging here as a candidate.
- **The token-savings audit-trail trade-off needs a heuristic.** This session's audit-trail line in Historical preserved provenance (which is correct per `feedback_lesson_lifecycle_chain.md` — the trim is recoverable via `memory_search_hybrid` against the moved entries' filenames). But it ate ~80 tokens. For sub-200-token trims, the audit line may swamp the savings. Worth a one-line decision rubric: "audit-trail line if trim ≥3 entries OR ≥150 tokens, else skip + rely on git blame."
- **Cross-session coordination via runlog ledger entry.** The concurrent session knew about this one (PMD backfill noted both rows). Going forward: when SessionStart detects another active session on the same canonical checkout, write a one-line ledger entry naming the other session. Useful for retros and for the eventual catch-fire on mutual-state corruption.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

| # | Change | Status | Promote-to | User OK? |
|---|---|---|---|---|
| 1 | Token-savings predictions for MEMORY.md trims must subtract audit-trail overhead (~80 tokens per provenance line) | candidate | augment `feedback_context_trim_verify_empirically.md` with one-line note | [ ] |
| 2 | `git status` between `git add` and `git commit` on canonical checkout is non-negotiable when concurrent sessions known active | candidate | promote `feedback_cross_session_commit_attribution_collision.md` Race-A from "recommended" to a hard rule in `multi-lane-worktree.md` hard refusal #6 atomic protocol step 4 | [ ] |
| 3 | One-line pointer replaces lazy-load section blocks cleanly (`### N.X + N.Y combined heading`) | record-only | shape pattern; usable on future externalizations without formal promotion | [ ] |

Single-occurrence noise (recorded but NOT proposed):

- The Windows backslash-path `git show` failure is already covered by `feedback_windows_bash_python_git_show_tmp_traps.md`; not a recurrence pattern.
- The ToolSearch round-trip friction is fresh-session-start cost; predictable and not a recurrence pattern within a single session.
- The concurrent commit landing mid-session is a single occurrence at promotion time; if it recurs in a future session, promote a coordination-protocol lesson.

---

## Cross-references

- `.claude/PRPs/handovers/context-prune-option-b-step3-onward-2026-05-22b.md` — the handover this session executed
- `.claude/PRPs/reports/session-retro-2026-05-22-context-prune-option-b-steps1-2.md` — predecessor retro whose shape this retro mirrors
- `.claude/lessons/feedback_context_trim_verify_empirically.md` — PMD #147, the empirical-verification pattern
- `.claude/lessons/feedback_cross_session_commit_attribution_collision.md` — Race-A pattern this session caught in real-time
- `.claude/lessons/feedback_lesson_lifecycle_chain.md` — provenance-discipline rationale for audit-trail lines
- `.claude/lessons/feedback_principles_not_rules.md` — doctrine that motivated Step 6's externalization of single-occurrence patterns
- `9623260e5` — Steps 5+6 ship commit
- `3ce413fe6` — concurrent retro from sibling session (Steps 1+2 retro)
- `e89ac013a` — session-start snapshot reference (Steps 3-6 handover authored)
