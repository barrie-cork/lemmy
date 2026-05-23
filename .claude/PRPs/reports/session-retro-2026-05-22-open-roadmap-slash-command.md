# Session retro — 2026-05-22 — open-roadmap slash command

**Harness:** claude-code
**Session window:** 2026-05-22 ~21:00 UTC → ~22:30 UTC (~90 min)
**Branch at start:** `474d40227` (`governance-v0`)
**Branch at end:** `a73aa7ddd` (`governance-v0`) — but session-attributable end-state is `8ed010c6c` (roadmap refresh); the two commits after are from a concurrent session
**Files touched (this session):** 3 (1 commit + 2 user-scope files)
**Commits:** 1 explicit (`8ed010c6c` roadmap refresh) + 0 for the slash command (user-scope `~/.claude/*` is not in any repo)

## TL;DR

Short session with two deliverables: (1) refreshed `.claude/PRPs/v1-roadmap.json` to reflect quality-r1 + fed-in-d shipped earlier in the day + flipped fed-in lane to `partial` with sub_phases tracking (+85/-22, commit `8ed010c6c`); (2) shipped a new `/open-roadmap` slash command at `~/.claude/commands/open-roadmap.md` backed by `~/.claude/scripts/render-roadmap.py` (PRD-grouped HTML view, dark mode auto-switch, progress bars). Main finding: when the user said "I liked the last layout better", the search for prior artifacts was shallow (only checked `Temp/` + obvious script paths) and missed that an earlier session today had created a different roadmap HTML viewer (`session-retro-2026-05-22-roadmap-viewer-and-pmd-reference.md`) — the right move would have been a broader Glob across `.claude/PRPs/reports/` for any prior roadmap-HTML retros before pivoting to "describe the layout you remember." Resulted in ~10 min of re-deriving a layout the user had already approved in a prior session.

---

## What surprised us

- **An earlier session today shipped a similar HTML viewer that this session didn't find.** `session-retro-2026-05-22-roadmap-viewer-and-pmd-reference.md` exists; it documents a single-file kanban HTML viewer with inlined JSON. When the user said "I liked the last layout better", a Glob of `.claude/PRPs/reports/session-retro-*roadmap*` would have surfaced it instantly. Instead, the search was limited to disk paths (`Temp/`, `~/.claude/scripts/`) and the prior session's existence was invisible. The user accepted the "describe the layout you remember" path, but it cost ~10 min vs. just reading the prior retro + reusing the layout description.

- **F-string nested-quote bug surfaced on first script run.** Python 3.11 doesn't allow `\"` inside `f"..."` when the outer f-string is itself nested inside another f-string. The fix was mechanical (extract the conditional HTML to a variable). Surprising in the sense that the script "looked right" at write-time; only execution caught it. Mitigated by running the script before declaring "shipped" — the smoke-test discipline paid off.

- **User-scope `~/.claude/` is not under git anywhere.** When the user said "save and close", there was nothing to commit for the slash command — the files are on disk, full stop. This is fine, but it means the work has no git audit trail; if `~/.claude/` gets wiped, the slash command is lost. Worth flagging that user-scope scripts/commands should probably have a backup story (separate concern; not this session's scope).

- **MEMORY.md updated mid-session by a concurrent process (linter or another session).** Two `<system-reminder>` messages flagged MEMORY.md changes mid-turn — one updating fed-in-d → CLOSED + fed-in-e ACTIVE before the roadmap edits started. The surface-first ritual fired (lane status surfaced on first reply); but it could have been louder about "concurrent edits in flight on shared state — re-fetch before commits." Atomic-protocol step 1 fired correctly at the actual commit boundary, so no clobber occurred — discipline held.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When the user references a "last/previous/earlier" artifact, Glob `.claude/PRPs/reports/session-retro-*<topic>*` BEFORE searching disk paths. Add to the session-start ritual or as a lesson under `.claude/lessons/feedback_search_prior_retros_before_rebuilding.md` | Surfaces same-day session artifacts that aren't on disk but ARE in the retro corpus; avoids re-deriving layouts/approaches the user already approved | trivial (one Glob in the search sequence) | 1× this session; counts as ≥2 with the prior session that itself was searching for older roadmap context |
| 2 | When building a Python script with f-strings that may nest, write a minimal smoke-test invocation **before** declaring the script ready (already practiced this session — formalize as a checkpoint). The lesson `pattern_test_against_reality_not_syntax.md` already covers this in spirit; cite it explicitly in any future skill-creator output that involves Python scripts | Catches f-string / quote-nesting / encoding bugs at script-author time, not at first user invocation | trivial (one Bash invocation per script) | held this session; cite the existing pattern when scripts are involved |
| 3 | Roadmap viewer source-of-truth: this session's `/open-roadmap` reads `.claude/PRPs/v1-roadmap.json` at every invocation (live). The PRIOR session's viewer inlined the JSON at build-time (will drift). Note in the prior retro's "What to change" #2 that the live-fetch approach is now shipped — closes the gap they identified | Closes the loop: their change-table item #2 ("rewrite the HTML's data loading block") is structurally resolved by the new Python script approach. Worth a one-line note in either retro file so future searches surface both | trivial (one-line addendum cross-reference) | direct successor to a prior session's open item |

## What to carry forward

- **Atomic add+verify+commit+push as a single Bash command for shared-state files.** Used at commit `8ed010c6c` for the roadmap refresh per `multi-lane-worktree.md` hard refusal #6. Held cleanly under concurrent MEMORY.md edits and untracked debug files in the working tree (`.claude/PRPs/debug/dq-frag-clarify-009.json` left alone, no false stage).
- **Verify-then-edit on shared roadmap state.** Before flipping fed-in-d / quality-r1 status, ran `gh pr view` against the actual merge commits + checked plan/retro paths on disk. Surfaced one correction (fed-in-e plan file DOES exist, contrary to MEMORY's "plan unauthored" note) before the edit was committed — kept the JSON honest.
- **Read CLAUDE.md / advisor-orchestrator.md system-reminders carefully.** The session-start surface-first ritual triggered correctly on `git worktree list ≥2` and surfaced lane status as the first response, beating the inherited-context momentum to dive into work.
- **Python script + slash command split is the right shape for repeated rendering.** Renderer is deterministic (~0.5s, no LLM tokens), command body is thin (~30 lines, only the steps + failure modes). Reusable for similar "render JSON-as-HTML + open in browser" jobs (e.g. could clone for `/open-runlog`, `/open-dq` if useful later).

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/skill-creator:skill-creator` (the slash command itself) | 5 | 0 | none | Provided the scaffold; AskUserQuestion gates kept design choices visible. Useful for first-time skill design even when the user wants to skip the eval loop. |
| AskUserQuestion (4 rounds for roadmap refresh; 3 rounds for slash command design) | 15 | 0 | none | Clean fork on lane representation + design choices. No re-work after gates closed. The clarify-before-build pattern paid off. |
| Glob/Bash search for prior renderer | 0 | 10 | medium | Searched disk paths (`Temp/`, `~/.claude/scripts/`, `brehon-fork/`) but NOT `.claude/PRPs/reports/session-retro-*roadmap*`. Missed the prior session's artifact entirely. See "What to change" #1. |
| Atomic roadmap commit (fetch → add → status verify → commit → push) | 5 | 0 | none | Held under concurrent MEMORY.md activity. Discipline from `multi-lane-worktree.md` #6 worked as designed. |
| Python script smoke-test before declaring shipped | 5 | 0 | low | Caught the f-string nested-quote bug pre-handoff. Cost: 1 re-run; value: would have been a user-visible failure on first /open-roadmap. |
| `cmd //c start ""` browser-open mechanic | 2 | 0 | none | Worked first try on Windows. No fallback needed. |

## Complexity scores (heavy tasks only)

None this session — both deliverables were small. Roadmap refresh: 1 file / 1 commit / ~25 min / 0 min log-silence. Slash command: 2 files / 0 commits (user-scope) / ~30 min / 0 min log-silence. Neither approaches the >55min / >40min / >8 files thresholds.

## Decisions to revisit

- **User-scope `~/.claude/scripts/` and `~/.claude/commands/` have no git backup.** If lost, work is gone. Worth a separate clarify: should there be a periodic snapshot to a tracked location, or accept the ephemeral nature? Not urgent.
- **The PRD-grouped layout may be reusable for other JSON-as-HTML jobs.** The `render-roadmap.py` script's renderer functions (`render_lane_section`, `progress_bar`, `status_badge`, dark-mode CSS vars) could be factored into a shared module if 2+ similar renderers ever ship. Single-occurrence; not promoting.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

For each item from "What to change" that meets the threshold, the user may approve promotion. Boxes UNCHECKED by default; user checks to authorise; a follow-up session (or the user manually) executes the checked items.

- [ ] Change #1: promote to `.claude/lessons/feedback_search_prior_retros_before_rebuilding.md` — when the user says "last/previous/earlier" artifact, Glob the retro corpus first
- [ ] Change #3: cross-reference addendum on `session-retro-2026-05-22-roadmap-viewer-and-pmd-reference.md` noting that the live-fetch approach (their open item #2) is now structurally shipped via `/open-roadmap`
- [ ] PMD eval write: this retro's "search prior retros before rebuilding" finding (skipped this turn — see Step 5.5 note below)

---

## Step 5.5 note — PMD backfill

Skipped. No `memory_write_eval` was issued this turn (the promotion-candidates checkboxes are user-authorised, not auto-executed). If the user checks the PMD eval box above, a future session will write the eval AND run the canonical-DB backfill per the skill's Step 5.5 (`sync-lessons-to-pmd.sh --db <CANON_PMD> --strict` → `backfill.js` with absolute path → verify zero missing vectors).

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted: `feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`, `feedback_retro_task_complexity_score.md`. Auto-phase reliability section omitted per Step 0.5: session did not invoke `/auto-phase` or mutate any `.claude/auto-state/*.json` (existing files are from prior sessions for v1-SL-c-2 + v1-federation-inbound-a)._
