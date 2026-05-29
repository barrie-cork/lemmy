---
name: session-retro
description: Session-level retrospective after a meaningful pi/Claude Code session — sections (What surprised us / What to change / What to carry forward), three-signal scoring of skills/agents/commands, automation proposals, optional promotion to .claude/lessons/. Use whenever the user says /reflect, "write a retro", "post-mortem", "what did we learn", "how did that go", or wraps up a non-trivial session — even without the word "retro." Cross-harness (Claude Code + pi). Distinct from `post-task-retro` (per-Junior-task eval + hook) and sub-phase retros at .claude/PRPs/reports/v1-*-retro.md (four-role, heavier). The in-between case: an ad-hoc session that touched several files, dispatched subagents, or surfaced a friction pattern.
---

# Session Retrospective

You are reviewing the session you just completed. Your job is not to
summarise what happened — the user can read the transcript. Your
job is to surface what's worth carrying forward: friction patterns,
wins worth repeating, and concrete change proposals that a future
session (in either harness) can act on.

This skill is the cross-harness, ad-hoc entry point. Both Claude
Code sessions and pi sessions invoke it via `/reflect`, "write a
retro", or skill-match against the description above. Its output
joins the existing `.claude/PRPs/reports/*-retro.md` corpus, so a
retro written by pi today is visible to a Claude Code session
tomorrow (and vice versa) via the same memory-injection rule that
loads the rest of the lesson library.

## First actions — every invocation

Read these three lessons before drafting anything. They encode
hard-won discipline; without them, retros drift into reports.

1. `.claude/lessons/feedback_retro_not_report.md` — A retro must
   say *what could be different*. Three canonical H2 headers:
   **What surprised us / What to change / What to carry forward**.
   These are required, not optional. If your draft doesn't have
   all three with substantive content, it's not a retro.
2. `.claude/lessons/feedback_four_role_retro_signals.md` —
   Three-signal scoring (saved / wasted / surprise) per skill or
   agent invocation. Numbers don't have to be exact; they have to
   be defensible from the transcript.
3. `.claude/lessons/feedback_retro_task_complexity_score.md` —
   per-task complexity metric (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`)
   for any heavy task in the session. Aggregate in §5 below.

If any of those files is missing, surface that as the first finding
and stop — your context is incomplete and the retro will repeat
past mistakes.

## Step 0.5 — Detect /auto-phase artifacts (load 4th lesson if present)

Before Step 1 inventory, check whether the session involved a
`/auto-phase` skill invocation. Run:

```bash
ls .claude/auto-state/*.json 2>/dev/null
ls .claude/auto-state/*-archived-*.json 2>/dev/null
ls .claude/auto-state/*-catchfire-*.md 2>/dev/null
```

**Trigger condition (revised 2026-05-09):** Auto-phase reliability
section is required ONLY when the **session actively interacted**
with `/auto-phase` — either invoked it directly OR mutated
`.claude/auto-state/<phase>.json` during the session. A leftover
JSON from a prior session does NOT trigger the section by itself.

To decide:

1. Did the transcript contain an explicit `/auto-phase <args>`
   invocation? → **trigger fires**.
2. Did the session edit, write, or `Bash` mutate any
   `.claude/auto-state/*.json` file? → **trigger fires**.
3. Did the session author a brief, run a Junior task, mutate a
   `validate-pending` DQ, or otherwise advance the state machine
   for the phase whose JSON exists? → **trigger fires**.
4. None of the above; JSON is on disk only because a prior session
   left it → **trigger does NOT fire; skip the 10-category section**.

When the trigger fires:

4. **Also read `.claude/lessons/feedback_auto_phase_retro_signals.md`**
   — 10-category automation-reliability discipline. The canonical
   3-lesson stack (above) captures friction in *manual* sessions; it
   does NOT capture failure modes specific to *automated*
   orchestration (stage-transition correctness, cadence calibration,
   catch-fire FP/FN, §G4 classifier accuracy, etc).

If this lesson exists and any auto-state artifact is present, the
retro template's optional **Auto-phase reliability** section
(`.claude/skills/session-retro/template.md`) MUST be filled. The 10
categories cover: stage-transition correctness, cadence calibration,
auto-state integrity, touchpoint count vs target, catch-fire FP/FN
rate, §G4 classifier accuracy, L14/L15/L16 fixes still holding,
subagent offload effectiveness, plan §13 fidelity vs cohort dispatch,
resume-cycle pain points.

Per the user's framing 2026-05-08: *"reliability and accuracy is more
important than speed, especially when the system offers automation"*.
Skipping this section under an `/auto-phase` retro is the same class
of failure as omitting "What to change" — the retro becomes a report.

If no auto-state artifacts and no `/auto-phase` invocation in the
transcript, this 4th lesson is dormant. Skip it; the section in the
template should be omitted entirely from the final retro file.

## Step 1 — Inventory the session

Don't trust memory; pull from authoritative sources. Run these in
parallel where possible:

```bash
# What changed in the repo this session
git log --oneline --since="<session-start ISO>"
git log --oneline --since="<session-start>" --grep "auto(pi)"   # pi auto-commits
git diff --stat <session-start-sha>..HEAD                        # files touched

# What the harness did (pi only)
ls -la ~/.pi/agent/sessions/ 2>/dev/null | tail -3              # latest sessions
# Read the most recent session file under sessionDir for the transcript

# What CI ran
gh run list --repo <repo> --limit 10 --json databaseId,workflowName,status,conclusion,event,createdAt

# What's left in flight
git status --short

# /auto-phase artifacts (only if Step 0.5 detected them)
cat .claude/auto-state/<phase>.json 2>/dev/null      # final state file
ls .claude/auto-state/*-catchfire-*.md 2>/dev/null   # catch-fire dumps
git log --oneline --grep "auto-phase\|chore(advisor)" --since="<session-start>"
```

Build a compact session timeline: ordered list of "what the user
asked → what was done → what surfaced." Skip noise (file reads,
tool acks); keep the inflection points (decisions, branch switches,
failed attempts, course corrections).

**Under `/auto-phase`:** the auto-state JSON is the authoritative
timeline source. Read its `user_gate_history`, `junior_tasks`,
`resume_count`, `last_known_phase_tip` history; cross-reference
against the git log of advisor commits. The advisor commits (matching
`^chore\(advisor\):` per attribution-integrity) are the durable
record of every state-machine transition; the JSON is the
machine-readable form. Disagreement between the two is itself a
finding (auto-state JSON drifted from git reality — surface as
category §3 "Auto-state integrity" issue).

## Step 2 — Score every skill / agent / command invocation

For every skill call, subagent dispatch, slash command, or major
tool batch, score it on the three-signal scale per
`feedback_four_role_retro_signals.md`:

| Signal | Meaning |
|---|---|
| **Saved (min)** | Wall-clock minutes saved vs. doing it manually or from scratch |
| **Wasted (min)** | Wall-clock minutes spent on dead ends, wrong premises, or re-derivation |
| **Surprise** | Behaviour the user or harness didn't expect, in either direction (none / low / medium / high) |

Be specific — "saved 30 min by isolating GH Actions context in a
ci-debug subagent" beats "subagent worked well." Numbers don't have
to be exact; they have to be defensible from the transcript.

For complexity-class tasks (impl, planning, BM verb, anything
multi-file or multi-step), record the complexity metric per
`feedback_retro_task_complexity_score.md`:
`<files-touched>/<commits>/<runtime-min>/<max-log-silence-min>`.

## Step 3 — Identify automation opportunities

Pattern-match across signals from Step 2. Three classes of
opportunity, in increasing order of investment:

1. **Bundle repeated work into a script.** If 2+ tasks in the
   session independently wrote a similar helper script, propose
   adding it to the relevant skill's `scripts/` dir so it's reused
   next time.
2. **Codify repeated friction as a lesson or extension hook.** If
   the same wrong-premise / surprising-behaviour appeared 2+ times,
   propose a new `.claude/lessons/feedback_<topic>.md`, a
   `pi.registerCommand` / event-handler change in
   `.pi/extensions/lemmy-hooks.ts`, or a Claude Code hook in
   `.claude/settings.local.json`.
3. **Isolate context-heavy roles into a subagent.** If the harness
   had to load a large body of detail it didn't need 80% of the
   time, propose a project-scope subagent (`.pi/agents/<name>.md`
   for pi, `.claude/agents/<name>.md` for Claude Code) with focused
   harness. Reference example: `.pi/agents/ci-debug.md` (added
   2026-05-06 after the adr-compliance loop).

The recurrence threshold is 2+ in this session, OR 1 here AND 1+
in prior memory. Single-occurrence frictions are noted, not
promoted. This rule comes from the existing RLS pattern: 3+ for
canonical promotion to `## Learned Patterns`, 2+ for a lesson file,
1 for a session note. Stay disciplined; over-promotion poisons the
lesson corpus with one-off noise.

## Step 4 — Write the retro file

Output path: `.claude/PRPs/reports/session-retro-<YYYY-MM-DD>-<slug>.md`
(UTC date; slug = a 3-5 word kebab-case description of the session's
main thread). Matches the existing `session-retro-2026-04-25-advisor-jm-c-plan.md`
precedent so retros aggregate naturally.

Use the template at `.claude/skills/session-retro/template.md`
verbatim. The CRITICAL requirement is that all three canonical
headers (What surprised us / What to change / What to carry
forward) have substantive content. If "What to change" is empty or
vague, you've written a report; rework before saving.

The template is structured to match the existing retro format
under `.claude/PRPs/reports/` so a future weekly-review or
phase-retro can lift sections / cite findings without reformatting.

## Step 5 — Conditional: write a PMD eval

If `PROJECT_MEMORY_DB` is exported (verify with
`echo "${PROJECT_MEMORY_DB:-unset}"`) AND a clear pattern emerged
from Step 3 (recurrence ≥ 2), write a `memory_write_eval` entry so
future sessions can find this learning via PMD search.

The eval entry's `source_ref` should be the session's working
branch or `session-retro-<slug>` if no branch is meaningful. The
`title` should be `Session retro: <one-line takeaway>`.

Skip this step if PMD is unset (`start-pi.sh` may not have wired it
on pi side; `.claude/scripts/setup-memory.sh` may not have run on
Claude Code side). Don't fabricate the path.

## Step 5.5 — Backfill PMD embeddings (MANDATORY if Step 5 wrote an eval OR §3 promoted a new lesson)

**The PMD MCP server has NO write-time embedding** (verified by
source inspection of `dist/index.js` — zero embed/ollama refs). A
`memory_write_eval` from Step 5, AND any new
`.claude/lessons/feedback_*.md` a §3 promotion created + synced via
`scripts/sync-lessons-to-pmd.sh`, land as **text-only rows**.
`memory_search_hybrid` silently degrades to FTS5 for unembedded rows
— the semantic-recall advantage (+62.7% R@10) is absent until
`backfill.js` embeds them. This recurred 2026-05-16 (8 unembedded
rows incl. entries from concurrent sessions that skipped the
backfill — see `feedback_pmd_backfill_after_write.md`).

If Step 5 wrote an eval, OR §3 promoted a new lesson that was synced
to PMD, run the backfill before Step 6:

> **CANONICAL DB — MANDATORY (per `feedback_pmd_cross_lane_canonical_db.md`
> + the 2026-05-18 stranding incident).** The PMD is cross-lane shared;
> the single canonical store is
> `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db`. A
> **relative** `.project-memory/memory.db` resolves against the
> per-worktree `PROJECT_ROOT`, so when this retro runs from a *lane
> worktree* (`brehon-fork-<lane>`) the sync + backfill + verify all hit
> a **lane-local** DB the Stop hook and `memory_search_hybrid` never
> read — the retro discipline then *becomes* the stranding vector it
> exists to close (this exact regression: v1-ship-1-r2, 5 lessons +
> 21 retros stranded). Always pass the **explicit absolute canonical
> path** below — `--db <canonical>` to the sync, `PROJECT_MEMORY_DB=
> <canonical>` to the backfill, the canonical path to the verify —
> NEVER the relative form, regardless of which checkout the retro runs
> from. Set once and reuse:

```bash
CANON_PMD="C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db"

# 1. (only if a new lesson was promoted) sync it into PMD first.
#    --db <canonical> is mandatory from a lane worktree (the script's
#    legacy fallback is lane-local and WILL strand — see 53b1a52c1).
#    --strict makes a lane-local resolution a HARD FAIL (exit 3) rather
#    than a non-fatal WARN this automated retro step might not surface
#    — defence-in-depth on the explicit --db (per the same incident's
#    "What to change" #3). If this exits 3, the --db path above is
#    wrong; fix it and re-run — do NOT proceed to backfill.
bash scripts/sync-lessons-to-pmd.sh --db "$CANON_PMD" --strict

# 2. Precondition check — Ollama reachable (silent-degrade trap):
curl -s -m5 http://homeserver:11434/api/tags >/dev/null \
  && echo "ollama OK" || echo "OLLAMA UNREACHABLE — backfill will FTS5-degrade; note in §6"

# 3. Backfill (idempotent on (memory_id, model) — only unembedded rows).
#    PROJECT_MEMORY_DB MUST be the absolute canonical path, NOT relative.
OLLAMA_URL=http://homeserver:11434 \
PROJECT_MEMORY_DB="$CANON_PMD" \
PROJECT_ROOT=C:/Users/barri/Developer/brehon-fork \
  node C:/Users/barri/Developer/MCPs/project-memory-mcp/dist/scripts/backfill.js --verbose

# 4. Verify 0 missing — against the CANONICAL DB (not a lane-local copy):
python -c "import sqlite3; c=sqlite3.connect(r'C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db'); print('MISSING vectors:', c.execute('SELECT COUNT(*) FROM memories m WHERE NOT EXISTS (SELECT 1 FROM memory_vectors v WHERE v.memory_id=m.id)').fetchone()[0]); c.close()"
```

Skip ONLY if Step 5 was skipped AND §3 promoted no new lesson (nothing
new to embed). If Ollama is unreachable, the rows stay FTS5-only —
note it in the Step 6 surface so the user knows semantic search is
degraded until the next backfill. Per
`.claude/lessons/feedback_pmd_backfill_after_write.md` +
`.claude/lessons/feedback_pmd_cross_lane_canonical_db.md`.

## Step 6 — Surface to user

Return a compact closing summary in the conversation:

```
Retro written: .claude/PRPs/reports/session-retro-<filename>

Highest-leverage proposals (top 3):
1. <change> — <expected effect>
2. <change> — <expected effect>
3. <change> — <expected effect>

Recurrence threshold met (≥ 2): <count> items proposed for promotion
Single-instance noise: <count> items recorded but not proposed
```

Include only the top 3 in chat; the full list lives in the retro
file.

## Hard refusals

- **Never overwrite a phase retro** at `.claude/PRPs/reports/v1-*-retro.md`.
  Those are sub-phase advisor retros with a different scope and
  per-role structure (per `feedback_four_role_retro_signals.md`).
  This skill writes only `session-retro-<date>-<slug>.md` files.
- **Never auto-promote a finding to a global location**
  (`~/.pi/agent/`, `~/.claude/`) without explicit user confirmation.
  Surface promotion candidates in §3 with the concrete command
  needed; the user runs them.
- **Never write a "report" instead of a "retro".** If "What to
  change" has fewer than 2 concrete items — each with a file path,
  command, or skill reference — the draft is incomplete. Rework
  before writing the file.
- **Never assess a session you didn't witness.** If the user asks
  /reflect on a session whose transcript you can't access (e.g. a
  prior pi session in `~/.pi/agent/sessions/` you can't read),
  refuse and ask the user to invoke /reflect from the session
  being retro'd.
- **Never overwrite an existing retro file silently.** If the
  target filename exists, append `-v2` (or `-v3` etc.) to the slug
  rather than clobbering. Retros are journals; revision history
  matters.

## When NOT to use this skill

- The user is mid-task and asks "how's it going?" — that's a
  status check, not a retro. Answer directly without invoking the
  skill.
- The session was 1-3 tool calls of routine code reading. Retro
  overhead is greater than the lesson value. Skip with a one-line
  "session too short to retro" reply.
- Just finished a Junior task on a `junior/*` branch — use
  `post-task-retro` instead. That skill is hook-enforced and tied
  to `memory_write_eval` with `source_ref` matching.
- Closing a sub-phase (v1-XX-Y) — use the canonical phase-retro
  shape under `.claude/PRPs/reports/v1-XX-Y-retro.md`. Per-role
  structure and TL;DR are required at that scope; this skill's
  template is too thin.
- A prior retro's "What to change" table already has concrete file
  paths + commands + skill refs — the current session is just
  shipping that table. Don't wrap the execution leg in a new retro
  proposal. Execute directly against the table; flip checkboxes;
  surface the execution as a small note in the original retro OR
  write a separate retro only if the execution surfaced new findings
  worth carrying forward. Two-session recurrence basis: 2026-05-22
  (memory-prune-gate-ship leg) + 2026-05-21 (retro-action-items-
  execution, PMD #451).

## Why this skill exists

Sessions accumulate friction patterns that no single iteration
sees clearly: a recurring wrong-premise loop, a skill that quietly
saves time, an extension hook that fires too often. Without a
disciplined retro habit, those patterns compound silently across
many future sessions in both harnesses.

A retro is the moment where the harness pays back its compounding
interest: ~10 minutes of thoughtful review in exchange for hours
of friction avoided across the next dozen sessions. That trade
only works if the retro produces *concrete change proposals* (§3)
— without those, the time is spent and nothing improves.

This skill is the cross-harness, ad-hoc complement to the existing
retro discipline:

| Scope | Skill / target | When |
|---|---|---|
| Per-Junior-task | `.claude/skills/post-task-retro/` → eval memory | End of every Junior task |
| Per-session (ad-hoc) | **this skill** → `.claude/PRPs/reports/session-retro-*.md` | After a meaningful interactive session |
| Per-sub-phase | (manual) `.claude/PRPs/reports/v1-XX-Y-retro.md` | Closing a Brehon sub-phase |
| Weekly | `.claude/skills/weekly-review/` | Sunday cron |

All four feed into `.claude/lessons/feedback_*.md` via the same
promotion pattern; the lessons get loaded by both pi (via PMD
search once `PROJECT_MEMORY_DB` is wired) and Claude Code (via the
`.claude/rules/memory-injection.md` rule). A finding promoted from
a pi /reflect session is reachable from the next Claude Code
session, and vice versa.
