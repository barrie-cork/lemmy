---
session: 2026-06-01-redaction-r1-close
scope: v1-redaction-r1 phase close + v1-quality-r3b bootstrap
branch: governance-v0 (canonical brehon-fork)
authored: 2026-06-01
---

# Session Retro — 2026-06-01 — redaction-r1 close

## TL;DR

Phase v1-redaction-r1 closed cleanly (PR #173 merged `8b8c3f6fe`). Two lessons promoted. v1-quality-r3b bootstrapped via phase-transition skill. Main friction: stale bootstrap file from a prior 2026-05-31 transition attempt was silently on disk, and a `_new` filename for the workflow-state skeleton was immediately normalised by the PMD PostToolUse hook — creating a stale MEMORY.md pointer from the first write.

## What surprised us

1. **Stale bootstrap from 2026-05-31.** When the transition skill reached Step 3, a `v1-quality-r3b-bootstrap.md` already existed with governance-v0 HEAD `32aa45a60` (from a prior 2026-05-31 session that must have started the transition and then been abandoned). The correct HEAD was `49dd95685`. The skill didn't detect this discrepancy — it would have overwritten with a stale hash if the Read-before-Write constraint hadn't forced the advisor to read the file first and notice the mismatch. Needed 7 targeted Edit calls to patch the stale sections rather than a clean write.

2. **PMD PostToolUse hook immediately normalised `_new` filename.** The transition skill created `workflow_state_v1_quality_r3b_new.md` (to avoid colliding with the CLOSED record of the same slug). Immediately after creation, the PMD hook rewrote it to `workflow_state_v1_quality_r3b.md` with compact content. MEMORY.md's pointer to `_new` was stale from the moment it was written. The hook's file was correct; the process was wrong.

## What to change

| # | Change | File | Expected effect |
|---|--------|------|-----------------|
| 1 | **Phase-transition skill: Pre-Step-3 stale bootstrap check.** Before writing the bootstrap file, `ls` the target path. If it exists, compare its recorded governance-v0 HEAD against `git rev-parse --short governance-v0`. If they differ → flag as stale, overwrite with live data, note in commit message. | `~/.claude/skills/brehon-phase-transition/SKILL.md` — new "Pre-Step-3" section | Eliminates cold session starting with wrong ritual hash; surface the staleness explicitly rather than silently inheriting wrong state. |
| 2 | **Workflow-state naming: delete CLOSED record before creating skeleton.** Never use `_new` suffix. Check if `workflow_state_<next-slug>.md` exists; if so, delete it (it's a stale skeleton or a mislabelled CLOSED record that should have been removed at a prior transition). Then create the canonical-slug skeleton directly. | `~/.claude/skills/brehon-phase-transition/SKILL.md` — Step 2 naming discipline | Prevents PMD hook normalisation from immediately making MEMORY.md pointer stale. |
| 3 | **Bootstrap git-state block populated last.** The "Git state at handoff" block must use `git rev-parse` output from immediately before writing the file, not from earlier in the session. Even a 30-minute gap can have commits land. | `~/.claude/skills/brehon-phase-transition/SKILL.md` — Step 3 content guidelines | Next bootstrap always has the correct HEAD hash as its session-start verification anchor. |

**Status:** All three changes implemented in `~/.claude/skills/brehon-phase-transition/SKILL.md` this session (2026-06-01). See commit `chore(retro): session-retro-2026-06-01-redaction-r1-close + skill fixes`.

## What to carry forward

1. **Pre-flight `ls` before bootstrap writes is a cheap catch.** 30-second check vs re-editing 7 sections of a stale file. The same pattern applies to any skill that writes a file it might have written in a prior session — check mtime or a sentinel field (like governance-v0 HEAD) before trusting the file isn't stale.

2. **PMD hook normalisation is a feature, not a bug** — but name files for the hook from the start. If the hook normalises `foo_new.md` → `foo.md`, the correct process is to never create `foo_new.md` in the first place. Trust the hook; write the canonical name.

3. **Session retros written to lane worktrees are lost when the worktree is removed.** Write session retros to the canonical `brehon-fork` repo path (`.claude/PRPs/reports/`) so they survive worktree cleanup. This retro was re-authored from the conversation summary after the lane worktree was gone.

## Skill invocation scores

| Skill / command | Saved (min) | Wasted (min) | Surprise |
|---|---|---|---|
| `/brehon-phase-transition v1-redaction-r1 v1-quality-r3b` | ~25 | ~10 | medium — stale bootstrap + `_new` normalisation |
| `/session-retro` | ~10 | ~0 | low |

**Complexity score (transition task):** 3 files / 1 commit / ~15 min / ~5 min max-silence → `3/1/15/5`

## Recurrence note

Both findings are 1× this session (0× prior confirmed). Recorded as "What to change" items; not promoted to `feedback_*.md` lessons yet. Promote if either recurs in the next phase transition.

**Exception:** the "session retro to lane worktree" finding (§carry-forward #3) is a 1× process gap — addressed structurally by always writing retros to canonical brehon-fork path.
