---
name: command-retro
description: >
  Per-slash-command retrospective. Aggregates friction signals for one or all
  `.claude/commands/*.md` (and `~/.claude/commands/*.md`) specs from PMD,
  `.claude/PRPs/reports/*-retro.md`, and `decision-queue.json`, scores each
  verb on a three-signal scale, auto-applies minor spec edits (typos, broken
  refs, dead links), and surfaces major edits as proposals for user review.
  DO use when: user says `/command-retro <verb>` or `/command-retro --all`,
  user asks "is this slash command still earning its keep", auditing a verb
  before a phase ship, weekly-review wants a command-level signal pass.
  Do NOT use for: per-Junior-task retros (use `post-task-retro`), session-wide
  retros (use `session-retro`), sub-phase retros (`.claude/PRPs/reports/v1-*-retro.md`),
  mid-invocation debugging of a verb that just failed (read the spec + transcript
  directly). Sibling to `post-task-retro` (per-task) and `session-retro` (per-session).
---

# Command Retrospective

You are reviewing one or more slash-command specs against the friction
signals they generated since their last retro. Your job is not to
re-design the verb — it's to surface whether the spec is still earning
its keep, and to apply small, defensible improvements when the evidence
is unambiguous.

This skill is the per-verb member of the retro family. Three-signal
scoring lifts directly from `session-retro`; eval-memory mechanics lift
from `post-task-retro`. The novel piece is the **command-spec edit
loop**: minor edits auto-apply, major edits surface as proposals in a
retro file the user reviews.

## When to run

- User invokes `/command-retro <verb>` (e.g. `/command-retro bm-merge`)
  to assess one verb.
- User invokes `/command-retro --all` to sweep every command spec,
  ranked by friction score.
- Weekly-review or session-retro pulls in command-level findings — they
  call this skill rather than re-implementing the aggregation.
- Before a sub-phase ship, advisor wants to know which BM verbs (the
  most-invoked class) shipped any silent friction this phase.

Skip if:
- The verb hasn't been invoked since its last retro (no new signal).
- The verb spec was committed less than 48 hours ago (no data yet).
- The user is mid-invocation of the verb and wants help finishing it
  (that's a debug, not a retro).

## First actions — every invocation

Read these three lessons before drafting anything. They encode hard-won
discipline; without them, the retro drifts into a report.

1. `.claude/lessons/feedback_retro_not_report.md` — A retro must say
   *what could be different*. The verb-level analogue: the proposal
   section must contain concrete diff suggestions (file:line + before
   → after), not generic complaints ("bm-merge feels heavy"). If your
   draft's proposals don't name file paths and exact text changes,
   it's a report, not a retro.
2. `.claude/lessons/feedback_four_role_retro_signals.md` — Three-signal
   scoring (saved / wasted / surprise) applied per-invocation of the
   verb. Aggregate across invocations to get a per-verb composite.
3. `.claude/lessons/feedback_principles_not_rules.md` — The recurrence
   threshold for promoting a proposed edit from "surfaced" to
   "auto-applied" is **3 invocations exhibiting the same friction**,
   matching the existing RLS pattern (3+ for canonical promotion). Two
   is a watch-list note. One is single-instance noise.

If any of those files is missing, surface that as the first finding
and stop — your context is incomplete and the retro will repeat past
mistakes.

## Step 0 — Resolve the verb list

**Per-verb mode** (`/command-retro <verb>`):
1. Resolve `<verb>` to a spec file. Search order:
   a. `.claude/commands/<verb>.md` (project scope)
   b. `.claude/commands/bm/<verb>.md` (BM sub-namespace)
   c. `~/.claude/commands/<verb>.md` (user scope)
2. If the verb has a leading `/`, strip it.
3. Zero matches → refuse with the searched paths; ask the user to
   confirm the verb name.
4. Multiple matches → ask via AskUserQuestion which to retro.

**Sweep mode** (`/command-retro --all`):
1. Glob both `.claude/commands/**/*.md` and `~/.claude/commands/*.md`.
2. For each spec, compute the friction score (Step 2). Sort
   descending. Take the top 5 by composite friction.
3. Run Steps 1-3 on each of the top 5 sequentially (not in parallel
   — the spec-edit phase is mutating and must be deterministic).
4. Step 4 writes a single sweep report listing all 5 verbs with their
   per-verb findings; the per-verb spec edits are applied inline as
   the sweep walks. Skip Step 5 (eval-memory) in sweep mode — write
   one composite eval at the end instead.

## Step 1 — Inventory the verb's invocations

Pull from three sources (the user picked "PMD + retros + DQ"). Run in
parallel where possible.

### 1a. PMD signals

```
memory_search_hybrid(query: "<verb> invocation OR usage OR friction OR bug",
                     tags: "<repo-name>", limit: 20)
memory_search_hybrid(query: "<verb> lesson OR pattern", tags: "lesson",
                     limit: 10)
```

Note any `qa-result` rows whose `source_ref` or content references the
verb. Note any `bug` rows whose tags or content reference the verb.
Note any `pattern` / `lesson` rows that cite the verb by name.

### 1b. Retro file signals

```bash
grep -l "/<verb>" .claude/PRPs/reports/*.md 2>/dev/null
grep -l "<verb>" .claude/PRPs/reports/session-retro-*.md 2>/dev/null
grep -l "<verb>" .claude/PRPs/reports/v1-*-retro.md 2>/dev/null
```

For each match, extract the surrounding paragraph(s) — the retro
mentions are the highest-signal source for friction patterns because
they were already pre-filtered through the retro-discipline.

### 1c. DQ signals

```bash
python -c "import json; d=json.load(open(r'.claude/decision-queue.json')); \
  hits=[e for e in d.get('resolved',[])+d.get('pending',[]) \
       if '<verb>' in json.dumps(e)]; \
  print(f'{len(hits)} DQ entries reference <verb>'); \
  [print(f'  #{e.get(\"id\")} ({e.get(\"kind\")}/{e.get(\"from\")}): {e.get(\"question\",\"\")[:80]}') for e in hits]"
```

Repeat for `.claude/decision-queue-archive-*.json` (the archived
entries often have the most-load-bearing historical signal).

### 1d. Spec-vintage check

```bash
git log --oneline --follow <spec-path> | head -10
git log -1 --format="%ci" -- <spec-path>   # last modification date
```

A spec last modified recently (<7 days) with high friction is a "did
we just regress?" signal. A spec last modified months ago with rising
friction is a "the spec has gone stale relative to the world it
operates in" signal. Both surface differently in Step 4.

## Step 2 — Score the verb (three signals + composite)

For every invocation surfaced in Step 1, score on the three-signal
scale per `feedback_four_role_retro_signals.md`. Then aggregate.

### 2a. Per-invocation scoring

| Signal | Meaning |
|---|---|
| **Saved (min)** | Wall-clock min the verb saved vs doing it manually |
| **Wasted (min)** | Wall-clock min spent on the verb misfiring, wrong path, re-execution, or fix-up |
| **Surprise** | Behaviour the user or advisor didn't expect (none / low / med / high) |

Score every invocation where the data exists. If a retro mention or
DQ entry references the verb but doesn't give enough detail to score,
note it as "uncatalogued" rather than guessing.

### 2b. Per-verb composite

| Metric | Value |
|---|---|
| **Invocations counted** | N |
| **Total saved (min)** | sum of saved |
| **Total wasted (min)** | sum of wasted |
| **Net (min)** | saved − wasted |
| **Surprise count (med+high)** | count of medium or high surprise events |
| **Friction score** | wasted ÷ (saved + wasted), 0.0-1.0 |
| **Composite score** | 1 − friction, 0.0-1.0 |

Apply the anti-inflation guard from `.claude/rules/evaluation-calibration.md`:
- Scores cluster 0.60-0.75. Above 0.85 = re-read the rubric, you're
  inflating.
- Below 0.60 = verb is net-negative and needs Step 3 attention.

If `invocations counted < 3`, the composite is provisional. Mark it
as "INSUFFICIENT SIGNAL — N invocations" and skip auto-apply in Step 3
(no minor edits applied on thin data).

## Step 3 — Classify proposals + apply minor edits

Pattern-match across the Step 1 signals. Three classes of proposed
edit, in increasing severity. The **minor / major split governs the
auto-apply gate**.

### Class A — Minor (auto-apply on recurrence ≥ 3)

Strictly mechanical, locally-scoped, reversible-by-edit changes:

1. **Typo fix.** Misspelling in the spec body. Diff = exact text
   replacement, no logic change.
2. **Broken file path.** Spec references `.claude/lessons/<file>.md`
   that no longer exists (file renamed, moved, or deleted). Diff =
   update path to new location (verified via `Glob`/`ls` before
   applying).
3. **Stale rule cross-ref.** Spec references a rule section by name
   that has been renamed (e.g. spec says "see §3.5" but the rule was
   restructured and the target is now §4.2). Diff = update reference
   (verified by reading the target rule file).
4. **Dead URL.** Spec contains a URL that returns 404 or has been
   replaced by a canonical form. Skip if the URL is external (rate-
   limited) unless the user explicitly confirmed.
5. **YAML frontmatter drift.** `description:` no longer matches the
   verb's actual scope (e.g. mentions a deprecated flag). Apply only
   if the verb scope has not changed — just the description text.

**Auto-apply gate (HARD)**:
- Friction signal must recur ≥ 3 times across invocations (per
  `feedback_principles_not_rules.md`).
- Or the issue must be objectively true (file doesn't exist, URL
  returns 404 — single observation suffices because the verb is
  broken right now).
- The diff must not change logic, control flow, branch behaviour, or
  any rule-cited contract (e.g. "the BM session NEVER touches
  `crates/**`" is contract; don't auto-edit contract text even if
  worded awkwardly).
- The spec file must not have an active git pending change (a `git
  diff --name-only HEAD <spec>` non-empty result blocks auto-apply
  — the user is mid-edit; don't clobber).

If the gate passes, **apply via Edit** (not Write — preserves the
rest of the file exactly). Then:
```bash
git add <spec-path>
# Do NOT commit yet — Step 4 batches all minor edits into one commit
# at the end of the verb's pass.
```

If the gate fails (logic-touching change, contract text, recurrence
< 3 without objective brokenness, file has pending changes), demote
the proposal to **Class B**.

### Class B — Major (always surface, never auto-apply)

Anything that changes logic, contract text, dispatch flow, file-
ownership boundaries, or behaviour the user might reasonably want to
ratify in advance. Examples from the existing corpus:

- Rephrasing a "MUST" / "MUST NOT" clause.
- Adding or removing a dispatch sub-agent.
- Changing a phase-stage ordering (e.g. L14 fix: runlog POST-merge
  vs PRE-merge).
- Adjusting the user-gate count (the six mandatory gates are
  load-bearing per CLAUDE.md "Mandatory user gates — never skip").
- Adding a new file-class to the file-ownership boundary in
  `.claude/rules/branch-manager.md` for BM verbs.
- Re-ordering the "First actions" section of a sibling retro skill.

Surface each Class B proposal as a row in the Step 4 retro file:

```markdown
| # | Proposal | Evidence | Recurrence | Suggested diff |
|---|---|---|---|---|
| B1 | <one-line description> | <PMD/retro/DQ citations> | <N invocations> | <verbatim before → after, with file:line> |
```

The user reviews; promotion happens manually.

### Class C — Watch (recurrence = 2)

The friction has been observed twice but not three times. Promote to
a watch-list row in `MEMORY.md` per the existing pattern (see
`MEMORY.md` "Watch / promote-if-recurs" section). The skill writes
this directly:

```bash
# Append a one-line entry to the watch section of MEMORY.md
# Path resolves via auto-memory: ~/.claude/projects/<encoded-cwd>/memory/MEMORY.md
```

Use a slug like `watch_<verb>_<friction-keyword>_<YYYYMMDD>.md` for
the linked file (write the watch file as well, brief — 5-10 lines on
the pattern, two citations).

## Step 4 — Write the retro file

**Per-verb mode**:
Output path: `.claude/PRPs/reports/command-retro-<verb>-<YYYY-MM-DD>.md`
(UTC date). Matches sibling retro filename conventions.

**Sweep mode**:
Output path: `.claude/PRPs/reports/command-retro-sweep-<YYYY-MM-DD>.md`.
The sweep file has a section per verb (the top-5 friction list)
ranked by composite score ascending.

Use the template at `.claude/skills/command-retro/template.md`. The
CRITICAL requirement (per `feedback_retro_not_report.md`) is that
**Class B proposals have concrete diff content** — not just "consider
revising Phase 5". A proposal without a before → after diff is
report-shaped; rework before saving.

If Class A edits were auto-applied during Step 3, list them in the
retro file's "Auto-applied edits" section with their full diff
content so the user can audit and revert if needed.

### Commit minor edits + retro file

```bash
# Stage the retro file
git add .claude/PRPs/reports/command-retro-<file>.md

# Stage any auto-applied spec edits (already added via git add in Step 3)
git status --short   # verify staging is what you expect

# Commit
git commit -m "$(cat <<'EOF'
chore(command-retro): <verb-or-sweep> — <one-line headline>

Auto-applied: <N> minor edits to <spec-path(s)>
Surfaced: <M> major proposals for user review
Composite score: <score>

Per .claude/skills/command-retro/SKILL.md
EOF
)"
```

**Subject convention**: `chore(command-retro):` — distinct from
`chore(advisor):` (advisor session writes) and `docs(retro):` (sub-
phase retro authorship). Forms a separate audit trail. The advisor
attribution rules (`.claude/rules/decision-queue.md` Attribution
integrity §Detection) do NOT apply to this commit class — this skill
runs in the laptop advisor session but writes to non-DQ artifacts;
attribution drift is not a risk here.

Do NOT push automatically. The user pushes after reviewing.

## Step 5 — Conditional: write a PMD eval

If the per-verb composite is below 0.60 (net-negative) OR ≥ 1 Class B
proposal surfaced OR ≥ 1 Class A edit auto-applied, write a single
eval memory:

```
memory_write_eval(
  title: "Command retro: <verb> — <one-line takeaway>"
  skill_or_tool: "command-retro"
  score: <composite>
  tags: "command-retro,<verb>,<repo-name>"
  source_ref: "<current branch>"
  content: |
    SCORE: <composite>
    CONFIDENCE: <pre-scoring estimate, 0.0-1.0>
    Invocations counted: <N>
    Net (min): <saved - wasted>
    Surprise events (med+high): <count>
    Auto-applied edits: <N>
    Class B proposals: <count>
    Class C watch entries: <count>
    Notable findings: <2-3 bullets>
)
```

Apply anti-inflation per `evaluation-calibration.md`: a verb that
worked cleanly with no proposals scores 0.65-0.70. A verb with
auto-applied minor fixes scores 0.55-0.65 (something WAS broken).

In sweep mode, write one eval per verb in the top-5, OR one aggregate
eval if the sweep was clean — your call based on what's
load-bearing.

## Step 5.5 — Backfill PMD embeddings (MANDATORY if Step 5 wrote any eval)

Per `feedback_pmd_backfill_after_write.md` + `feedback_pmd_cross_lane_canonical_db.md`.
Same canonical-DB discipline as `session-retro` Step 5.5 — the PMD
MCP has no write-time embedding, and a lane-worktree invocation must
target the canonical absolute path or the eval strands.

```bash
CANON_PMD="C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db"

# Precondition — Ollama reachable:
curl -s -m5 http://homeserver:11434/api/tags >/dev/null \
  && echo "ollama OK" || echo "OLLAMA UNREACHABLE — FTS5-degraded; note in §6"

OLLAMA_URL=http://homeserver:11434 \
PROJECT_MEMORY_DB="$CANON_PMD" \
PROJECT_ROOT=C:/Users/barri/Developer/brehon-fork \
  node C:/Users/barri/Developer/MCPs/project-memory-mcp/dist/scripts/backfill.js --verbose

# Verify 0 missing
python -c "import sqlite3; c=sqlite3.connect(r'C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db'); print('MISSING vectors:', c.execute('SELECT COUNT(*) FROM memories m WHERE NOT EXISTS (SELECT 1 FROM memory_vectors v WHERE v.memory_id=m.id)').fetchone()[0]); c.close()"
```

Skip ONLY if Step 5 was skipped (no eval written).

## Step 6 — Surface to user

Return a compact closing summary in the conversation:

**Per-verb mode**:
```
Command retro: /<verb>
Composite: <score> (<N> invocations, net <saved-wasted> min)

Auto-applied (Class A): <N> minor edits
  - <one-line each>

Surfaced for review (Class B): <M> major proposals
  - <one-line each>

Watch-list (Class C): <K> entries added to MEMORY.md

Full retro: .claude/PRPs/reports/command-retro-<verb>-<date>.md
```

**Sweep mode**:
```
Command retro sweep — top 5 by friction
1. /<verb-1> — <composite>, <N> invocations, <K> Class A applied, <M> Class B surfaced
2. /<verb-2> — ...
...
Full sweep report: .claude/PRPs/reports/command-retro-sweep-<date>.md
```

## Hard refusals

- **Never auto-apply a Class B edit.** The major/minor split is the
  load-bearing safety gate. If a proposal touches logic or contract
  text and the recurrence is 3+, that's a strong signal — still
  surface it as Class B and let the user ratify.
- **Never edit a contract clause** ("MUST" / "MUST NOT" / "NEVER" /
  hard-refusal text) under Class A even for a typo. Promote to
  Class B and surface; contract text is read by future sessions and
  even cosmetic edits there warrant user ratification.
- **Never edit a spec the user is mid-editing.** `git diff
  --name-only HEAD` showing the spec path → demote all proposals to
  Class B and note the conflict in the retro file.
- **Never sweep with `/command-retro --all` from a dirty working
  tree.** The sweep makes batched commits; an unrelated dirty state
  pollutes them. Refuse and ask the user to stash or commit first.
- **Never write a retro that's only Class A auto-applies** without
  also recording the friction signal that justified them. A retro
  file with no "Findings" section is a changelog, not a retro.
- **Never run on a verb committed in the last 48 hours** without
  user override. Specs need at least a couple of invocations under
  their new shape before retro signal is meaningful.

## When NOT to use this skill

- The verb just failed during the current session and you want to
  fix it — read the spec + transcript directly. A retro looks
  backward across many invocations; a debug looks forward at one
  failure.
- The user is asking "should we add a `/new-verb`" — that's a skill-
  design question, use the dogfood gate (`feedback_dogfood_slash_command_specs.md`)
  on the proposed spec.
- The slash command is `disable-model-invocation: true` AND has
  never been invoked manually — there's no signal corpus to score
  against. Note in the retro file and move on.

## Why this skill exists

Slash command specs are the most fan-out artifacts in the
RLS substrate: a single change to `bm-merge.md` affects every BM
session for the rest of the phase. Without a retro loop, friction
signals stay scattered across PMD evals, DQ entries, and session
retros — visible individually but never aggregated. The user notices
"bm-merge feels heavy" after the 12th invocation and re-engineers
under time pressure.

A command-level retro closes the loop on the same compounding-
interest deal the rest of the retro family operates on: ~10 minutes
of pattern-matching across pre-aggregated signals in exchange for
fewer rounds of "the spec drifted from the world" friction in future
phases. The auto-apply on minor edits keeps the loop cheap; the
major-edit surface keeps the user in control of contract text.

The position in the retro family:

| Scope | Skill / target | When |
|---|---|---|
| Per-Junior-task | `.claude/skills/post-task-retro/` → eval memory | End of every Junior task |
| Per-session (ad-hoc) | `.claude/skills/session-retro/` → `.claude/PRPs/reports/session-retro-*.md` | After a meaningful interactive session |
| **Per-slash-command** | **this skill** → `.claude/PRPs/reports/command-retro-<verb>-*.md` | **After enough invocations to score the spec; sweep at retro/weekly-review time** |
| Per-sub-phase | (manual) `.claude/PRPs/reports/v1-XX-Y-retro.md` | Closing a Brehon sub-phase |
| Weekly | `.claude/skills/weekly-review/` | Sunday cron |

All five feed `.claude/lessons/feedback_*.md` and `MEMORY.md` via
the same promotion pattern. A Class B proposal ratified by the user
in a command-retro becomes a spec edit; a Class C watch entry in
MEMORY.md becomes a candidate lesson on next recurrence.
