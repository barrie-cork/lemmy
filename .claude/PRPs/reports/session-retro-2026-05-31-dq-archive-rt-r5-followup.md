# Session retro — 2026-05-31 — dq-archive-rt-r5-followup

**Harness:** claude-code
**Session window:** ~14:06 IST → ~14:25 IST (~20 min)
**Branch at start:** `db839c300` (`governance-v0`) / RT-r5 `074571ac1`
**Branch at end:** `db839c300` (`governance-v0`) / RT-r5 `283f56ad5`
**Files touched:** 2 on RT-r5 (`.claude/decision-queue.json`, new archive file)
**Commits:** 2 on RT-r5 (dedup + archive pull); 0 new on governance-v0

*Continuation of `session-retro-2026-05-31-dq-archive-dedup.md`. That retro covered governance-v0 cleanup; this one covers the RT-r5 extension that surfaced new findings.*

## TL;DR

Applied the same DQ dedup to RT-r5 (505 KB → 266 KB, 245 → 141 resolved). Three new findings vs the prior retro: (1) the modified-entry check needed a substantive-fields filter, not a full-object comparison — `resolved_at`-only diffs are cosmetic; (2) a full `merge --no-edit` from governance-v0 conflicts on `v1-roadmap.json` because governance-v0 carries an older roadmap snapshot; the correct pattern is `git checkout FETCH_HEAD -- <file>` for surgical single-file pulls; (3) `git push origin governance-v0` must precede any cross-branch `git checkout FETCH_HEAD -- <file>` on a worktree that fetches from origin — the new archive file wasn't on origin yet when RT-r5 tried to pull it.

---

## What surprised us

- **`resolved_at`-only diff tripped the modified-entry guard** — the field comparison loop checked `answer`, `answered_by`, `resolved_at`, `bucket`, `approved_by`, `approved_at`. DQ #315 differed only on `resolved_at` (08:53 vs 10:45), which the loop flagged as a modification, causing the first dedup script to report "1 modified entry" and skip #315. The fix was narrowing the comparison to substantive fields (`answer`, `answered_by`, `result`, `question`) and treating `resolved_at` as cosmetic. This is a latent footgun for any future dedup script that copies the same field list.

- **Full merge-forward conflicts on roadmap** — `git merge origin/governance-v0` auto-merges DQ and most meta files cleanly, but `v1-roadmap.json` conflicted because governance-v0 carries a stale roadmap snapshot (from before RT-r4/quality-r3 shipped). A phase branch should always win on the roadmap. The correct primitive for "pull one file from governance-v0 into a phase branch" is `git checkout FETCH_HEAD -- <file>`, not a full merge that inherits all of governance-v0's diverged content.

- **Push-before-checkout ordering** — the first attempt to `git checkout FETCH_HEAD -- .claude/decision-queue-archive-2026-05-29-*` failed with "pathspec did not match any file known to git" because governance-v0 hadn't been pushed yet. The archive commit existed locally but `FETCH_HEAD` on RT-r5 was pointing at the pre-archive origin tip. Fix: `git push origin governance-v0` first, then re-fetch on RT-r5.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | In any DQ dedup script, compare only substantive fields (`answer`, `answered_by`, `result`, `question`, `kind`) — never `resolved_at`, `timestamp`, or approval timestamps | Prevents false "modified" classification for cosmetic timestamp diffs; keeps the dedup safe to auto-run | minor — 1-line filter change in future scripts | 1× here; likely every future dedup run |
| 2 | Document the `git checkout FETCH_HEAD -- <file>` pattern as the canonical way to pull a single file from governance-v0 into a phase branch — add a note to `.claude/rules/multi-lane-worktree.md` §"Mode A — trunk-authorship" or a new §"Single-file pull from trunk" | Prevents future merge conflict loops when a phase branch needs one new governance-v0 file without inheriting stale roadmap content | minor — 3-line addition to multi-lane-worktree.md | 1× here; will recur whenever a governance-v0 artifact needs propagating to an active phase branch |
| 3 | Add "push governance-v0 before cross-worktree file pulls" as an explicit ordering note in the DQ archive procedure | Prevents the pathspec-not-found failure class when the archive file exists locally but not on origin | minor — one bullet in the retro's "what to carry forward" or the decision-queue.md archive section | 1× here; will recur at every archive cycle |

## What to carry forward

- **Surgical `git checkout FETCH_HEAD -- <file>` pattern over full merge-forward** when the goal is to pull one governance-v0 artifact into a phase branch. Full merge inherits all diverged content including stale `v1-roadmap.json`; surgical checkout is lossless and conflict-free.

- **Push trunk first, fetch in worktree second** — any cross-worktree file pull via `FETCH_HEAD` requires the source branch to be current on origin. The local-only commit is invisible to another worktree's fetch. Sequence: commit on source branch → push source → fetch in target worktree → checkout file.

- **Substantive-vs-cosmetic field split for DQ comparison** — when comparing DQ entries across live and archive to detect "was this modified on the phase branch?", only the fields that gate decisions matter (`answer`, `answered_by`, `result`, `kind`, `question`). Timestamp fields (`resolved_at`, `timestamp`, `approved_at`) are cosmetic and should be excluded from the modified-entry guard.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Field-comparison dedup check | 3 | 4 | medium | `resolved_at` diff on #315 tripped the guard; needed a second pass to narrow field list |
| `git merge --abort` + surgical checkout | 2 | 3 | medium | Full merge-forward attempted first; conflict on roadmap required abort + rethink; surgical checkout worked cleanly on retry |
| Push-ordering discovery | 0 | 2 | low | Predictable in retrospect; resolved quickly once identified |
| AskUserQuestion (proceed with dedup?) | 2 | 0 | none | User interrupted the first Python write — correct; the modified-entry check needed to run first |

## Complexity scores (heavy tasks only)

No Junior tasks. Two interactive commits on RT-r5.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| RT-r5 DQ dedup + archive pull | 2 | 2 | 20 | 3 |

Low complexity. No watchdog risk.

## Decisions to revisit

- The `dq-archive.sh` script should gain a `--cutoff-ts` flag (proposed in prior retro) AND a substantive-fields-only comparison mode to prevent future false-modified classifications.
- Whether to add a "push trunk before cross-worktree pulls" PreToolUse hook guard — low priority, sequencing issue is easy to recover from.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Substantive-fields-only DQ comparison: add as a note in `homeserver/scripts/dq-archive.sh` inline comment + any future dedup script template
- [ ] Single-file pull pattern: add to `.claude/rules/multi-lane-worktree.md` §"Mode A — trunk-authorship from a phase lane" — already documents the SSH path, this is the simpler laptop-side analog

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted (loaded from prior invocation):
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
