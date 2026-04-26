---
name: Junior subagents promote lessons via LESSON commit trailer
description: Junior subagents on the EliteDesk can't write to laptop-side PMD. Convention: end commit-message body with a LESSON line; advisor harvests at retro time and promotes to PMD + .claude/lessons.
type: feedback
---

PMD lives at `~/.claude/projects/.../memory/` on the laptop only. Junior subagents (planning / impl-task / bm-task) run on the EliteDesk in worktrees and **cannot write directly to PMD** — different machine, no SSH-back path designed. Without a discipline, lessons learned during a Junior task have nowhere to land:

- Junior commit messages capture the *what* but not retrospective framing
- DQ resolved entries capture *answers* but not why they generalise
- Worktrees get reaped after task completion — anything not committed is lost
- The advisor on the laptop reads commit messages and DQ entries but has no signal saying "this one is lesson-worthy"

**Why:** during a v1 sub-phase the cumulative PMD gap could be 10–30 lessons that should have been promoted but weren't. The c-inherited-dragon plan §F.2 deferred this; the gaps doc 2026-04-26 (Drift 3) re-opened it.

**How to apply — two-part convention:**

**Part 1 — Junior side (`LESSON:` trailer in commit-message body).**

When a Junior subagent (planning / impl-task / bm-task) discovers a lesson worth promoting, end the commit-message body with one or more `LESSON:` lines:

```
feat(rep_tuning): cap appeal-window calc at i32::MAX (task 12)

Files: crates/db_views/src/appeal.rs
Validation: cargo test -p lemmy_db_views appeal -- --nocapture (passed)

LESSON: appeal-window expiry uses FOR UPDATE NOWAIT not bare UPDATE — the sanction-cleanup pattern's lock-wait deadlocks under jury concurrency. Reference: crates/db_views/src/appeal.rs:142.
LESSON: i32::MAX overflow guard is needed because the wall-clock now+window math sits inside the .map() before the saturating_add. Without the guard, large window configs produce panic in release.
```

Conventions:
- One `LESSON:` line per discrete lesson — don't merge multiple lessons into one line
- Include enough context that the advisor can write it as a standalone PMD entry without re-reading the commit diff
- Reference specific files/lines when the lesson cites code
- Don't paste cargo output, secrets, or large diffs (per `feedback_no_cargo_output_paste`)
- Don't write `LESSON:` for routine progress — the bar is "future me would have wanted to know this before starting"

The trailer is optional. Most Junior commits won't need one. If you're tempted to write more than two `LESSON:` lines on a single commit, the lessons are probably finer-grained than they need to be — pick the most durable one.

**Part 2 — Advisor side (harvest at retro time).**

At sub-phase retro authoring (after the retro gate in `/brehon-phase-transition` Step 0 passes), the advisor scans for `LESSON:` trailers and DQ resolved entries and promotes lesson-worthy signals to PMD + `.claude/lessons/`. The harvest workflow:

1. **Gather:** within the phase branch's commit range:
   ```
   git -C C:/Users/barri/Developer/brehon-fork log <prior-tip>..<current-tip> --pretty=full | grep -B 2 '^LESSON:'
   ```
   Read each lesson trailer + the commit subject for context. Cross-reference against `.claude/decision-queue.json` resolved entries — DQ resolutions sometimes encode lessons that the impl subagent didn't promote to a `LESSON:` trailer.

2. **Cluster:** if multiple commits surfaced the same lesson (e.g. three `LESSON:` trailers about pq-sys cache invalidation), the lesson is pattern-strength — promote with higher importance and reference all three commits in the PMD entry.

3. **Decide:** for each lesson signal, decide between three outcomes:
   - **Promote:** durable cross-phase pattern → write a new PMD entry (`feedback_*` for behavioural rules, `reference_*` for resource pointers, `project_*` for active-state notes). Mirror to `.claude/lessons/` per the one-system principle.
   - **Skip:** narrow to this sub-phase, captured adequately in the retro carry-forward section, no PMD value.
   - **Augment:** the lesson belongs in an existing PMD entry — use `supersedes: <old_id>` to update the existing memory rather than write a duplicate.

4. **Cite source in body:** PMD entries promoted from Junior commits should name the originating commit SHA in the body (`Source: <SHA> (impl-task subagent on phase-v1-JM-c worktree)`), so future advisors can re-read the original context.

5. **Mirror to lessons corpus:** per `feedback_one_system_memory_in_repo`, every new `feedback_*` or `reference_*` PMD entry that applies to brehon-fork work gets a parallel write to `brehon-fork/.claude/lessons/<filename>.md`. Same content. Commit on `governance-v0` so future Junior subagents read it via their start-of-task `Glob .claude/lessons/`.

6. **Update MEMORY.md index:** one-line entry for each new memory, per `memory-hygiene` rules.

**Where the harvest lives in the retro file:** the retro file at `.claude/PRPs/retros/<phase-id>-retro.md` should include a fourth informational H2 (in addition to the three required ones from `feedback_retro_not_report`):

```markdown
## Lessons promoted this phase

- `feedback_appeal_window_lock_wait.md` — sourced from <SHA>, impl-task, v1-JM-c task 12
- `feedback_i32_max_saturation_guard.md` — sourced from <SHA>, impl-task, v1-JM-c task 12
- (none) → if no lessons surfaced, write "(none — no LESSON: trailers in this phase's commits)"
```

This makes the harvest visible and auditable. A retro with `(none)` and a phase that had ≥5 commits is a signal the convention is being underused — the advisor should check whether the impl-task subagent prompts (`.claude/agents/impl-task.md`) actually mention the convention.

**What this lesson does NOT cover:**

- The retro file's three required H2 sections (still governed by `feedback_retro_not_report`)
- Per-role retro structure under the four-role model (governed by `feedback_four_role_retro_signals`)
- Cross-machine memory sync (option (b) in the gaps doc — deferred indefinitely; this lesson encodes options (a)+(c) only)
