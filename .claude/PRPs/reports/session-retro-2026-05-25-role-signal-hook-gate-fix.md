# Session retro — 2026-05-25 — role-signal-hook-gate-fix

**Harness:** claude-code
**Session window:** 2026-05-25 ~15:30Z → ~15:55Z (~25 min)
**Branch at start:** `ea99b6acf` (`governance-v0`)
**Branch at end:** `8535ad496` (`governance-v0`)
**Files touched:** 3 (hook script, drain script, .gitignore)
**Commits:** 2 (auto: 0, explicit: 2)

## TL;DR

User asked `/check-role-health` while planning task #463 was running.
Health report showed all roles with `rules_read: []` and zero MCP
invocations — consistent with smoke-test-only signals. Then the user
prompted: "Ensure it is correctly wired up." Following that thread
uncovered that the role-detection gate in
`.claude/hooks/role-signal-utilisation.sh` had been silently exit-0
on every legitimate Junior worker for 24h+ since the T4a tightening
in `b80c16dcf` (2026-05-24 12:06Z): the post-T4a anchor `^\[role:` on
the first 200 bytes of the first user message could never match the
current Junior CLI dispatch shape, which prepends a ~552-byte
framework prefix before the `Task:\n[role:X]` line. Patched both the
hook (match the `Task:\n[role:X]` shape on bytes 0-2000) and an
adjacent drain-script bug (the discovery `find` filter missed every
worker-worktree queue file because scp renames them to
`worktree-<job>-queue.jsonl`). End-to-end verified live against task
#463's transcript: PMD row #564 written, then pruned as a dry-run.
Carry-forward: when a piece of plumbing's instrumentation hasn't
produced new data in a day, that itself is the signal to look at the
plumbing, not at the data.

---

## What surprised us

- **The corpus shape was the diagnostic clue, not the wiring.** The
  `/check-role-health` report initially read as "no production tasks
  have flowed yet; wait for v1-RT-r3 to generate real signals." That
  framing was wrong: only 4 production rows existed (561 / 562 /
  smoke-only) and 24h had passed since the T4a fix. The signal corpus
  not growing was the bug, not a waiting state. The user spotted this
  before me — the `/check-role-health` skill itself doesn't surface
  "your signal-emission rate looks like zero," so the diagnostic
  required outside-the-skill judgment.
- **Four sequential prior fixes (f6088a83d → feaa75db9 → 729b13312 →
  b80c16dcf) each replaced the previous root cause with a different
  root cause.** Branch gate → CLAUDE_PROMPT env → permissive grep →
  too-narrow anchor. The hook header documented each prior failure
  carefully — and yet failure #4 shipped because the test cases used
  during the T4a tightening were *advisor sessions* and *finalize
  agents* (the false-positive class), not the production Junior
  dispatch shape (a legit-worker case that was untestable from the
  laptop-side transcript corpus available at T4a time).
- **PMD rows 561 + 562 were finalize-agent false-positives from the
  pre-b80c16dcf permissive grep era, not real Junior workers.** Their
  source transcripts (`197534f7-...jsonl`,
  `6de3389c-...jsonl`) start with `You are a git finalize agent.`
  Both transcripts contain `[role:planning]` / `[role:impl-task]` as
  *quoted* text inside a "Task that was completed:" prose line, and
  the pre-T4a `head -50 | grep '[role:X]'` cheerfully matched them.
  The signals were never measuring what they claimed to measure.
- **The hook's failure shape was perfectly silent.** No `stderr`,
  no `exit 1`, no diagnostic queue entry. The combination "no rows
  written + no error" reads identically to "no Junior tasks ran" —
  which is the harness's quiet steady state. The defect had visible
  cost (24h of lost signal) but no visible symptom.
- **The drain script had its own latent bug from the same shipping
  window.** scp renames worker queue files
  to `worktree-<job>-queue.jsonl` (line 73 of drain), but the
  discovery `find` (line 90) matched only the literal
  `role-signal-queue.jsonl` filename. So worktree-prefixed files were
  fetched and then silently skipped during ingest. Even if the hook
  had been working, the worker signals would have been stranded in
  the laptop's drain staging dir, invisible to PMD.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add a one-line `_count` query to `/check-role-health` Step 5 that prints `(rows written in last 24h: N)` AND prints a WARN line when N=0 and there has been at least one Junior task dispatched in the same window. Implementation: shell out to `mcp__junior-brehon__list_tasks` for the count of `running\|completed\|failed` tasks in the 24h window, compare to row count from the existing search. | Converts "signal corpus looks small" from soft observation to mechanical surface. The current session would have caught the bug in 1 turn instead of 4. | minor | 1× this session; latent for 24h+ before user prompted; promote to skill | 
| 2 | Add a smoke-test step to `.claude/hooks/role-signal-utilisation.sh` review discipline: every time the role-detection logic changes, run the patched hook against ≥3 real transcripts (one real Junior worker, one finalize agent, one advisor session) BEFORE committing. The smoke harness I wrote at `.claude/role-signal-drain/test-role-detect.sh` is a working template; promote it under `.claude/hooks/test-role-signal-utilisation.sh` (tracked, gitignored signal-data deps remain ignored). | Catches the "test cases didn't include the production case" class that produced fixes #2/#3/#4. | medium | 4× recurrence on the same hook (f6088a83d / feaa75db9 / 729b13312 / b80c16dcf — promote-eligible) |
| 3 | Add a `.gitignore` entry sweep at retro/phase-transition: when a feature adds a runtime-artifact directory or file, `grep <feature-name> .gitignore` first. The MEMORY.md WATCH already flagged this (T4a 2026-05-24 with `.claude/role-signal-drain/`); this session re-hit it (was about to commit the drain dir). Recurrence threshold = 2. | Codify the pattern instead of relying on each session catching it ad-hoc. | minor | 2× now (T4a + this session); promote per MEMORY.md WATCH rule |

## What to carry forward

- **Read the file under the bug.** When the hook's role-gate failed
  the dry-run, I read every prior commit touching the file (4 of
  them) in chronological order before proposing a fix. Each commit's
  body documented a different root cause; aggregating them ruled out
  re-introducing any of the three prior failure modes. The proposed
  fix (`^Task:\s*\n\[role:X\]`) is structurally narrower than fix #3
  AND structurally broader than fix #4, hitting the exact contract
  shape and rejecting both prior false-positive classes by
  construction.
- **Verify across the negative cases too.** The smoke harness tested
  three transcript shapes: live Junior worker (must match) +
  finalize-agent transcripts that previously false-positive'd
  (must not match) + advisor session (must not match). All four
  cases passed before pushing.
- **Hot-patch live worktrees only when the patch is purely
  advisory.** Task #463 was hot-patched because the hook is exit-0
  always — no risk to the actual task. If the patched component had
  been load-bearing for task completion (e.g. a different Stop hook
  that blocked exit), the right answer would have been "let the task
  complete; future tasks pick up the fix."
- **Surface PMD pollution and clean it.** The dry-run wrote PMD row
  #564 with empty utilisation arrays. Kept-clean trumps slight
  data-completeness here: marked it `DRY_RUN` + `DELETE_ME`,
  expired-now, importance=1; pruned. The single real signal at task
  #463 exit will arrive on a clean dataset.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/check-role-health` skill | 5 | 0 | low | Worked exactly as written; the gap was in interpretation. No defect in the skill itself; §3 #1 is a feature enhancement. |
| Reading prior hook-commit bodies (4 commits) | 15 | 0 | medium | The bodies were the deciding evidence for the right regex shape. Without them, would have replayed fix #2 or fix #3. Carry-forward — invest in committing detailed bodies on hook fixes. |
| Live-transcript smoke harness (ad-hoc bash) | 10 | 2 | low | One false test expectation (job-462 expected `planning`, was `bm-task`) — 2 min to diagnose; trivial. Net +8. Promote per §3 #2. |
| Hot-patch into job-463 worktree via scp | 5 | 0 | low | Risk-free because hook is advisory exit-0. Saved the v1-RT-r3 planning-task signal that would otherwise have been lost. |
| Dry-run signal cleanup (PMD row #564 expire + prune) | 2 | 0 | none | Kept dataset clean for the real signal. |
| AskUserQuestion (3 calls: fix approach, in-flight task, dry-run cleanup) | 3 | 0 | none | Each one routed a real fork. Auto-mode classifier let all three through cleanly; no double-confirm friction. |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Diagnose + patch role-signal hook + drain | 3 | 2 | ~25 | <1 |

Below all flags (>55 / >40 / >8). Bounded surgical fix.

## Decisions to revisit

- **`/check-role-health` interpretation guidance is too permissive.**
  The current skill body says "All `rules_read: []` across every
  production task" matches "the expected pattern" for
  Haiku-backed workers — that framing absorbed the defect signal.
  Worth a single-line tightening: distinguish "empty arrays AND
  config_version is the live manifest SHA" (real worker that just
  didn't `Read` anything) from "empty arrays AND signal volume below
  expected dispatch rate" (instrumentation bug). §3 #1 covers the
  mechanical surface; this is the prose-side companion.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] §3 #2 (hook smoke-harness discipline, 4× recurrence on this
  one hook): promote to
  `.claude/lessons/feedback_role_detection_smoke_three_shapes.md`
  with the principle "every role/gate change runs against ≥3
  transcript shapes — real worker, finalize agent, advisor session —
  before commit." Cross-references the four sequential fixes by SHA.
- [ ] §3 #3 (gitignore audit at feature-author time, 2× recurrence):
  per MEMORY.md WATCH (already lifted 1× from T4a 2026-05-24). Now
  2×. Codify as
  `.claude/lessons/feedback_sibling_gitignore_at_feature_author.md`.
- [ ] §3 #1 (`/check-role-health` surface): update
  `~/.claude/skills/check-role-health/SKILL.md` (or
  `.claude/skills/check-role-health/SKILL.md` per skill registry) to
  add Step 5b "dispatch-vs-signal-rate check."
- [ ] PMD eval write — Step 5 below.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
