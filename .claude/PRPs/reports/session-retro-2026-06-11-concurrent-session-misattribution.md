# Session retro — 2026-06-11 — concurrent-session-misattribution

**Harness:** claude-code
**Session window:** ~2026-06-11 18:05Z → 18:50Z (~45 min; follow-up leg of the retro-check thread that began 16:54Z)
**Branch at start:** `b19eb1806` (`governance-v0`)
**Branch at end:** `bd5270225` (`governance-v0`)
**Files touched:** 4 (2 lesson files, `.gitignore`, 1 new memory + MEMORY.md pointer)
**Commits:** 1 explicit this leg (`bd5270225`); memory files are auto-memory (not git-committed)

## TL;DR

This leg promoted the two retro-check findings into lesson files, then chased an
apparent "session_id shifted mid-session" anomaly — which turned out to be a
**misread of a machine-shared marker directory**. The investigation revealed
that a *second, concurrent CC session* (`6db85786`, running `/auto-phase`
optimization) had been live in the same canonical `brehon-fork` checkout the
entire time, interleaving commits on `governance-v0` and rewriting `.mcp.json`
under me. Two of my earlier conclusions were wrong and got corrected: (1) the
`.mcp.json` localhost→Tailscale rewrite + secret-bearing `.bak` were the *other*
session's explicit user task, **not** my `lesson-pmd-sync.sh` hook; (2) there
was no id instability — two sessions, two correct ids. Top change: **when
`.mcp.json` changes under you or `governance-v0` HEAD advances without your
action, that is the signal of a concurrent live session — STOP and surface it
immediately (Hard refusal #7), don't absorb it as incidental.**

---

## What surprised us

- **Two CC sessions were live in the canonical checkout simultaneously, and I
  didn't notice until forced to.** The git log interleaves them plainly in
  hindsight — `cd7f9dff0 chore(auto-phase): …` (the other session) sits directly
  between my `b19eb1806` and `bd5270225`. I committed *around* another session's
  commits on the same branch without registering it as the multi-lane hazard my
  own rules exhaustively document.
- **The "session_id shift" was a phantom.** `/tmp/cc-retro-sessions/` is a fixed,
  machine-shared path — it holds one `.start` marker per *concurrent* session.
  Seeing two markers (`df821efb`, `6db85786`) read as "my id changed" when it was
  actually "a second session exists." The markers were both correct.
- **I mis-attributed the `.mcp.json` rewrite to my own hook — twice over.** First
  I claimed `lesson-pmd-sync.sh` repaired `.mcp.json` and created the `.bak`
  (it has no such logic — I even confirmed that and *still* drew the wrong
  conclusion). The real author was the concurrent session, whose 18:27 prompt was
  literally "repoint .mcp.json to homeserver, then backfill." I built a confident
  causal story on a coincidence of timing.
- **The hook firing at 83-min session age was the fix working correctly**, not a
  regression — pleasant confirmation. The age-gate passed (old session) and the
  retro-presence check found the 17:04 retro had aged out of the 60-min window.
  Exactly the designed behaviour.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Treat "shared artifact changed under me" as a concurrency signal, not noise.** When `.mcp.json` / `.env` / `governance-v0` HEAD / a tracked file changes without my action, STOP and run `git log --oneline -5` + check `~/.claude/projects/<proj>/*.jsonl` mtimes for a second live session BEFORE continuing. Per `feedback_canonical_checkout_foreign_wip_means_stop.md` Hard refusal #7 — which I had loaded and still under-applied. | Catches the concurrency hazard at first contact instead of 30 min into a false investigation. | minor | 2× this session (HEAD advanced silently + `.mcp.json` changed) + prior memory (cross-session-collision pattern) |
| 2 | **Before building a causal story from timing coincidence, falsify it.** The `.bak` appeared ~when my hook ran → I assumed my hook made it. The 30-sec check (`grep cp/.bak in the hook`; check the *other* transcript) would have refuted it immediately. Per `feedback_falsifiable_hypothesis_before_structural_fix.md` applied to attribution, not just structural fixes. | Stops confident-but-wrong attribution before it ships in a commit body / retro. | minor | 2× this session (both the id-shift and the `.mcp.json` mis-attribution were timing-coincidence stories) |
| 3 | **Lesson written this leg** (`reference_retro_check_marker_dir_machine_shared.md`, already shipped) — documents the machine-shared marker dir so the next session doesn't re-run this investigation. Verify it injects: it's a `reference_` memory, loaded via MEMORY.md pointer (line 92). | Future session sees ">1 marker = concurrent sessions, don't investigate" up front. | done | 1× (this incident) |

## What to carry forward

- **Investigate-before-sweeping caught a real secret-leak.** Even though I
  mis-attributed *who* created the `.mcp.json.bak`, the discipline of inspecting
  the untracked `.bak` (rather than `git add -A`-ing it) correctly prevented
  committing live API keys, and the `*.bak` gitignore + removal were the right
  fixes regardless of cause. The action was right even when the causal story was
  wrong — worth separating "did I do the safe thing?" (yes) from "did I explain
  it correctly?" (no, first pass).
- **Staged-set isolation verification held under concurrency.** `git diff
  --cached --name-only` before each commit confirmed only my intended files were
  staged, even with another session committing to the same branch. That check is
  exactly the Race-A/C mitigation from
  `feedback_cross_session_commit_attribution_collision.md` — it did its job; no
  foreign file was swept into my commits.
- **Empirical verification of the hook fire** — instead of assuming the 83-min
  nag was a regression, I checked the clocks (sqlite `now` vs `date -u` agreed),
  the markers, and the retro-window math. Confirmed designed behaviour in ~2 min.
- **Reading the actual transcripts settled the question definitively.** The
  discriminator was each session's first user prompt (`df821efb` = "I just
  started a new cc session…" = mine; `6db85786` = "Review the recent plan for
  optimising /auto-phase…" = theirs). Authoritative-source-over-memory, again.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Transcript forensics (read both `*.jsonl`) | 20 | 2 | high | Definitively resolved the "id shift" + the `.mcp.json` author. The 2 wasted min = the Windows `/c/...` path trap in the first python call (FileNotFoundError; fixed with `os.path.join` + raw Windows base). |
| `lesson-pmd-sync.sh` (PostToolUse auto-sync) | 8 | 0 | low | Both lessons auto-synced to HTTP PMD (938/939) with zero manual `memory_write`. Worked cleanly. |
| Lesson authoring (2 files, sibling-matched frontmatter) | 5 | 0 | none | `feedback_lesson_frontmatter_top_level_type.md` discipline held; harness normalized frontmatter on the memory file as expected. |
| Memory write (`reference_…marker_dir`) + MEMORY.md pointer | 3 | 0 | low | Dedup-checked first (no existing coverage); atomic edit on the shared MEMORY.md. |
| The false "id-shift" hypothesis itself | — | 8 | high | The whole leg's main waste: ~8 min before the marker-dir-is-shared realization. Mitigated for the future by change #1 + the new reference memory. |

## Complexity scores (heavy tasks only)

No heavy/Junior tasks this leg. Aggregate: `4/1/~45/0` (4 files, 1 git commit,
~45 min interactive wall-clock, no log-silence dimension — no dispatched work).
Well inside comfortable zone; no carry-forward complexity signal.

## Decisions to revisit

- **Should there be a SessionStart surface when a second CC session is already
  live on the same project?** The multi-lane hooks (`session-start-multi-lane-check.sh`)
  watch *worktree* drift, but two sessions on the *same* canonical checkout (not
  separate worktrees) may slip past. Worth checking whether that hook would have
  WARNed here, or whether a `~/.claude/projects/<proj>/*.jsonl` recent-mtime probe
  belongs in the session-start ritual. Not actioned — single occurrence at this
  scope; revisit if it recurs.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [x] Marker-dir-is-machine-shared: **already promoted** this leg to `reference_retro_check_marker_dir_machine_shared.md` (auto-memory) + MEMORY.md pointer. Done.
- [ ] §What-to-change #1 (shared-artifact-change = concurrency signal): consider amending `feedback_canonical_checkout_foreign_wip_means_stop.md` to add `.mcp.json` / `.env` / HEAD-advance to its trigger list explicitly (currently focuses on `git status` WIP). Meets threshold (2× this session + prior). User-authorise before editing that lesson.
- [ ] §What-to-change #2 (falsify timing-coincidence attribution before committing it): candidate amend to `feedback_falsifiable_hypothesis_before_structural_fix.md` to generalize from "structural fix" to "any causal attribution." Meets threshold (2× this session). User-authorise.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
