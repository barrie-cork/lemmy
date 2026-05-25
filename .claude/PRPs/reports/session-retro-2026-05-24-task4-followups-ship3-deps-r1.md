# Session retro — 2026-05-24 — task4-followups-ship3-deps-r1

**Harness:** claude-code
**Session window:** ~2026-05-24T10:50Z → ~2026-05-24T12:35Z (~105 min)
**Branch at start:** `097a51d09` (`governance-v0`, post-T4a authoring)
**Branch at end:** `335628525` (`governance-v0`, +3 commits pushed; uncommitted dq fragments + roadmap edit)
**Files touched:** 4 committed (1 new lesson, 1 modified skill, 1 modified command, 1 hook + 1 lesson with 2 edits)
**Commits:** 3 (all explicit; no auto-commits)

## TL;DR

Continuation session that started executing the three T4a session-retro proposals from the previous compact-boundary handover, then expanded scope twice: first to mirror PMD lesson 525 (NSSM traps) for parity discipline, then to make the role-customization system genuinely useful for Ship-3 evaluation. The load-bearing finding was a **defect-class verification miss on the role-signal hook itself**: the previous session's "false-positive on advisor sessions" diagnosis was right that `head -50 | grep '[role:X]'` over-matched, but I missed the actual blocker the user was experiencing — the `retro-check.sh` exit-2 path, NOT role-signal-utilisation.sh perf cost. Caught only by reading `retro-bypass.jsonl` (7 fail-opens in 5 days, all 3 ESC presses) BEFORE patching anything. Falsifiable-hypothesis discipline (per the lesson extended in this very session) was the saver. End-of-session pivot to lane-hygiene fixes (ship-3 close + deps-r1 wrong-checkout fix) revealed a sibling pattern: **the user's typed status ("Clean trunk at 335628525, in sync with origin, 42 callsites across 31 files — cutting the phase branch") concealed a multi-lane-worktree violation** that empirical verification (`git worktree list`) surfaced in one tool call. Top change proposal: the brief-authoring template + `/check-role-health` consumer should both surface "what locality does the artifact constrain?" as an explicit first-class question, not just an exhortation buried in a lesson.

---

## What surprised us

- **The hook the user complained about was NOT the hook I had refactored.** User said "hooks are firing and causing the agent to get stuck until i press esc." My initial framing was "must be the role-signal hook, that's the one I changed proposal #2 to fix." But `retro-check.sh` is the one that exits 2 (blocks) — `role-signal-utilisation.sh` exits 0 always (advisory). Caught by reading `retro-bypass.jsonl` (7 fail-opens in 5 days, all attempt_count=3 — the ESC pattern) BEFORE patching anything. Generalises to: when the user reports an outcome ("got stuck"), don't pattern-match to the most-recently-touched code; verify which actual code path produces the outcome.

- **Role-tag false positive count in MY OWN transcript: 14.** When I synthesized the role-signal hook fix, I needed a discriminator that wouldn't fire on advisor transcripts discussing the four-role model. Counted occurrences in `/c/Users/barri/.claude/projects/.../8d330ff5-*.jsonl`: **14 `[role:X]` substrings** in a 1MB transcript. Every Stop event in the prior session had been running the full jq -rcs scan + git rev-parse + CLI invocation — measurably wasteful even though it always exit-0'd. The "false-positive on advisor session" diagnosis from session-4 retro was structurally right but I hadn't appreciated the magnitude until I counted.

- **Ship-3 lane state was already shipped before I touched it.** User said "ensure ship-3 lane has latest code" — MEMORY.md had ACTIVE entry pointing at the lane as if work were pending. Empirical check: PR #149 merged at 07:20Z (~5h before session start), local branch had only the post-merge runlog commit (1 ahead, 52 behind), worktree dir was empty and unregistered. The MEMORY.md entry was stale by ~5h. Asked the user via `AskUserQuestion` before deleting — confirmed close. Pattern: workflow-state markdown rots fast under PR-merge events because the merge happens on the BM side, not the advisor session.

- **The user's narrative description of "branch was cut correctly" hid a multi-lane-worktree violation.** User reported: "Clean trunk at 335628525, in sync with origin, 42 callsites across 31 files — cutting the phase branch." Sounded healthy. `git worktree list` showed `phase-v1-deps-r1` checked out IN the canonical `brehon-fork/` worktree — per Hard refusal #1 in `multi-lane-worktree.md`, that's the forbidden case. The narrative description gave no signal; the worktree list did. Cost: one extra tool call (~5 sec) to catch what was about to compound into DQ races + cross-lane merge conflicts. Per `feedback_falsifiable_hypothesis_before_structural_fix.md`: every status the user types is a hypothesis until checked against the authoritative source.

- **PowerShell `Remove-Item -Force` succeeded where `rmdir` + `rm -rf` + `cmd rmdir /S /Q` all failed with "device or resource busy".** The empty `brehon-fork-ship-3/` dir was watched by a VS Code file-watcher process (one of ~11 Code.exe subprocesses for the user's 2 windows). The Unix-shell deletion tools see the directory as held; PowerShell's `Remove-Item -Force` uses a different file-API path that handles watched-dir gracefully. Reproducible workaround for future "dir locked by VS Code watcher" situations.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Add an "outcome ≠ cause" verification step to advisor-orchestrator §5.4 DQ triage** AND to any new-feature/fix-impl brief authoring workflow. When the user reports an outcome ("stuck", "blocked", "wrong"), the advisor's FIRST tool call must be the cheapest authoritative-state check that confirms WHICH code path produced that outcome — NOT a patch to the most-recently-touched code. Specifically: read the observable log/journal (`retro-bypass.jsonl` for hook blocks; `git worktree list` for branch state; `gh pr view` for merge state) BEFORE proposing a fix path. | Prevents wrong-defect-class patches; structural-fix lesson recurrence already covers a sibling case but the bias toward "I just changed X, so it's X" is its own pattern | minor (1-paragraph addition to existing rule) | 1× this session (role-signal hook ≠ retro-check.sh confusion), 1× prior session (DQ #338 named-daemon-path-wrong-defect-site, per `feedback_falsifiable_hypothesis_before_structural_fix`) — **threshold met (2×)** |
| 2 | **Workflow-state file staleness signal** — MEMORY.md "Active workflow state" entries should embed a `last_verified_at:<ISO>` line OR be regenerated at session start. Stale entries from prior sessions (ship-3 ACTIVE while PR #149 was merged) waste 5-10 min of session start per occurrence (read stale claim → empirical check → reconcile). Cheap option: add a one-line check at SessionStart hook that flags any workflow-state entry whose embedded last-update timestamp is >12h old as "potentially-stale, verify against git". | Prevents "ACTIVE" entries from misleading the next session post-merge; the BM-side merge doesn't refresh the advisor-side MEMORY.md | minor (SessionStart hook + 1-line `last_verified_at` field convention in workflow_state_*.md) | 1× this session (ship-3 stale ACTIVE 5h post-merge), but the broader class "workflow-state files rot fast under merge events" is structural; deferred until 2nd observation |
| 3 | **Add a "PowerShell Remove-Item -Force fallback" entry to `pattern_cross_platform_divergences.md`.** The lesson currently has the rsync row added in this session but no fallback for "VS-Code-watcher-holds-dir on Windows". One sentence: "When `rm -rf` / `rmdir /S /Q` returns 'device or resource busy' on Windows, retry with `powershell -Command \"Remove-Item -LiteralPath <path> -Force\"` — different file-API path tolerates open watchers." | Prevents future "I can't delete this empty dir" incidents from spending tool calls on multiple shell variations | minor (1-line addition to existing pattern) | 1× this session, **possibly N× historically** — folded into existing pattern alongside the rsync entry per the same retro proposal style |
| 4 | **`/check-role-health` should NOT require user-scope vs project-scope choice from the user.** This session's session-4 retro promoted "scope-vs-locality" to the handover-assumption lesson, which is right. But a thicker fix: any new slash command that consumes project-scope artifacts (canonical PMD, role substrate, drain script) should hard-refuse user-scope authoring with a one-line check at write time: "command body references `.claude/roles/`, `.project-memory/`, or `scripts/brehon/` — must be project-scope or refuse to author." | Prevents the user-scope/project-scope class of mis-authoring without relying on the user catching it as falsifier | minor (a PostToolUse hook that checks any new `.claude/commands/*.md` OR `~/.claude/commands/*.md` for cross-scope path references and warns) | 1× session-4, 1× this session (but this session caught at compact-boundary, not authored fresh) — **threshold met (2×) — promotion candidate** |

## What to carry forward

- **Read the observable log/journal BEFORE patching the most-recently-touched code.** This session, `retro-bypass.jsonl` was a 30-second read that completely changed the patch shape (showed retro-check.sh was the blocker, not role-signal hook). The pattern is broader than DQ-RCA falsification — any time the user complains about an outcome, read the SIDE-EFFECT log first.

- **`AskUserQuestion` for state-classification ambiguity.** When ship-3 lane state was unclear (MEMORY.md said ACTIVE; git said MERGED), used `AskUserQuestion` to surface 3 options (close lane / re-cut fresh / verify only). User picked "close". 0 wasted minutes on guessing the user's intent. Cost: ~30s for the question round-trip. Saved: potentially 10-30 min of either wrong-direction cleanup or "are you sure?" back-and-forth.

- **Empirical verification of user-typed status.** "Branch cut correctly" → `git worktree list` → caught the canonical-checkout violation. The cost of the extra tool call (~5 sec) is negligible compared to the cost of compounding the violation through several follow-up tool calls. Especially load-bearing when the user is on mobile remote and giving narrative status updates without full context.

- **PowerShell `Remove-Item -Force` for Windows dir-lock situations.** Reproducible escape hatch when `rm -rf` / `rmdir` / `cmd rmdir /S /Q` all fail with "device or resource busy". Different file-API path tolerates active file watchers.

- **Refusing partial work to verify a hypothesis is cheap insurance.** When user said "branch cut correctly", I checked first (5 sec), found a violation, asked permission to fix (~30 sec), got the green light, executed cleanly. Cost: 35 sec. Alternative: pattern-match "cut correctly" as a closed case, start dispatching work from the wrong checkout, compound for 20+ min before the next worktree-list reveals the mess. Discipline pays.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Read `retro-bypass.jsonl` before patching | 25 | 0 | high | Inverted the defect-class diagnosis — without it, would have shipped role-signal hook fix only and missed the actual blocker class |
| Falsifiable-hypothesis check on user-typed branch-cut status | 15 | 0 | medium | One `git worktree list` revealed multi-lane violation hidden in narrative description |
| AskUserQuestion (ship-3 lane disposition) | 8 | 0 | none | Clean 3-option fork; user picked "close" immediately |
| Empirical drain-script smoke test (1 pending signal drained + embedded) | 3 | 0 | low | Confirmed pipeline operational + cleared the queue before Ship-3 starts collecting |
| Synthetic Junior-dispatch transcript for hook verification | 5 | 0 | none | `/tmp/test-junior-transcript.jsonl` + row-540 write-then-delete verified both positive + negative cases in one round |
| ToolSearch for TaskList/TaskUpdate (deferred tool loads) | 0 | 2 | low | Two separate ToolSearch round-trips to load TaskList then TaskUpdate; could have been batched in one query "select:TaskList,TaskUpdate" — minor inefficiency |
| PowerShell `Remove-Item -Force` fallback for locked dir | 5 | 3 | medium | Tried rmdir, rm -rf, cmd rmdir before PowerShell worked; ~3 min on Unix-tool variations could be skipped next time |
| /session-retro (current) | — | — | — | In flight |

**Net wall-clock:** ~105 min total; ~65 min "productive" execution against the 3 T4a proposals + NSSM mirror + Ship-3 evaluation-readiness; ~25 min on lane-hygiene fixes (ship-3 close + deps-r1 worktree fix); ~10 min on tool-loading + retry friction; ~5 min retro authoring overhead. No major dead ends.

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. No tasks crossed the heavy-task threshold (>55min runtime, >40min log silence, >8 files touched). Three multi-file commits, each sub-threshold:

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Implement T4a proposals 1+2 + retro #3 WATCH-item | 3 (hook + lesson + MEMORY.md) | 1 | ~25 | <1 (interactive) |
| Mirror PMD lesson 525 → NSSM lesson + sync to PMD | 1 (new) + 1 (existing pattern_cross_platform) | 1 | ~15 | <1 |
| Ship-3 eval-readiness (task_id tag + threshold override + retro-join) | 2 (skill + command) | 1 | ~20 | <1 |
| Ship-3 lane close + deps-r1 worktree fix | 0 commits (housekeeping) | 0 | ~25 | <1 |

No outliers; no carry-forward signals from complexity scores.

## Decisions to revisit

- **The 4 dq-frag-clarify-deps-r1-*.json files in canonical `brehon-fork/.claude/PRPs/debug/`** should be moved to the lane worktree and appended to the lane's DQ. Surfaced to user at session end; no decision recorded yet. Logical follow-up at next deps-r1 session start.

- **The leftover `origin/junior/role-impl-task-v1-ship-3-task-1...-438` branch** wasn't pruned during ship-3 close. Weekly-review's branch-accumulation health check will catch it; no immediate action needed.

- **11 zombie/normal VS Code subprocesses for 2 user windows** — confirmed normal subprocess fan-out (file watcher + GPU + extension host + language server per window). No action needed; flagged in conversation for completeness.

- **Cross-session DQ fragment activity during the session** (7 dq-frag files appeared mid-conversation) — likely from a parallel session driving deps-r1 clarify. Not investigated this session; will surface naturally at next session start via the session-start-multi-lane-check.sh WARN.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1 (outcome-vs-cause verification step): promote to `.claude/rules/advisor-orchestrator.md` §5.4 OR a new `.claude/lessons/feedback_outcome_not_cause_verify_log_first.md`. **Threshold met (2×).** Owner: next session.
- [ ] Change #4 (slash-command scope-vs-locality hard-refuse): promote to a PostToolUse hook OR a brief-author template note. **Threshold met (2×).** Owner: next session, or fold into existing brief-author template revisions.
- [ ] Change #2 (workflow-state staleness signal): single-occurrence — defer until 2nd observation. Recheck after next merge event.
- [ ] Change #3 (PowerShell Remove-Item fallback): single-occurrence within `pattern_cross_platform_divergences.md` — fold in alongside the rsync row from the same session-4 retro carry-forward.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
