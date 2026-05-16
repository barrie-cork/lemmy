---
name: retro-harvest
description: Sweep retros, triage open proposals for currency (STALE/LIVE/SUPERSEDED), emit one ranked actionable report. Read-only.
user-invocable: true
disable-model-invocation: true
---

# Retro harvest

Retros accumulate proposals faster than anyone acts on them. The
`session-retro` skill writes a "Promotion candidates" checklist with
boxes **unchecked by default** — the implied contract is "user checks
to authorise; a follow-up session executes." But nothing reads those
files: no PostToolUse hook globs `.claude/PRPs/reports/`, `weekly-review`
only sweeps PMD memories, and `memory-injection.md` loads `docs/memory/`
not the retro tree. So unless the user manually re-opens a specific
retro, every proposal in it silently drops.

This skill closes that gap. It is the harvester the retro corpus never
had: scan every retro, pull the open proposals, check each one against
the *current* codebase (because a proposal written 9 days ago may
already be shipped, stale, or done better by a later retro), and hand
the user one ranked report they can act on in a single sitting.

**This skill produces a report only — it never edits rules, lessons,
skills, or code.** The user reads the report and decides what to
action. Acting on proposals is a separate, explicit step (the report
ends with copy-pasteable next actions, not auto-applied changes).

## Why currency triage is the core of this skill

A raw list of "every unchecked box across 52 files" is noise — ~48
items, most already handled. The value is the *currency* judgment:

- **STALE** — already shipped. The lesson file exists in
  `.claude/lessons/`, the rule already has the change, the script
  already has the fix, or the problem no longer exists. Report it as
  closeable so the user can check the box and move on.
- **LIVE** — still relevant, still unimplemented. The proposed lesson
  file is absent, the rule lacks the change, the script gap remains.
  These are the real backlog.
- **SUPERSEDED** — a later retro proposed the same thing better, or
  the underlying decision was reversed by a subsequent ADR/DQ. Point
  at the superseding artifact so the user collapses duplicates.

Without this judgment the user re-reads 48 proposals to find the 12
that matter. With it, the skill does that filtering and shows its
evidence so the user can trust or override each call.

## Phase 0 — Environment + scope

Run first, before any heavy read:

1. `pwd && git branch --show-current` — confirm CWD. This skill reads
   `.claude/PRPs/reports/` which is the same on every branch (retros
   are direct-commit meta-work per `phase-branch.md`), so any branch
   is fine, but record it for the report header.
2. `git log --oneline -5` — capture the current trunk tip; the
   triage subagents use "is this commit/lesson present at HEAD" as
   the currency oracle, so the report must state which HEAD.
3. Glob `.claude/PRPs/reports/session-retro-*.md` and
   `.claude/PRPs/reports/*-retro.md`. Count them. Print a one-line
   scope snapshot: `scope: <N> session-retros + <M> phase-retros |
   branch <b> | HEAD <sha-short>`.
4. **Optional `--since <YYYY-MM-DD>` argument.** If the user passes a
   date, only triage retros whose filename date (or, for phase
   retros, file mtime) is on/after that date. Without it, default to
   **the last 21 days** — older proposals are almost always either
   shipped or deliberately deferred, and triaging the full historical
   corpus every run is wasteful. State the effective window in the
   snapshot line. The user can pass `--all` to force the full corpus.
5. **Phase-retro date oracle caveat.** Phase-retro files
   (`v1-*-retro.md`) often share an identical mtime from a bulk
   `git clone` / `git worktree add` (e.g. every file touched the same
   minute). mtime is therefore an unreliable date signal for the
   `--since` / last-21-day window. Before relying on mtime, sanity-
   check it: if ≥3 phase-retros share the same mtime minute, fall
   back to the file's **git authorship date**
   (`git log -1 --format=%ad --date=short -- <file>`) for the window
   decision, and note the override in the report header. Per the
   `integrator.md` "flag contradictory results" discipline — a clone
   artifact masquerading as a date would silently widen or narrow the
   window. (All three iteration-1 runs hit this; one caught it
   unprompted, which is why it is now explicit.)

## Delegation policy (applies to Phases 1 and 2)

Phases 1 and 2 are read-heavy: extracting from 20-50 retro files, then
grepping/globbing the live tree for ~30-70 currency checks. The
**preferred** execution is to delegate that read work to `Explore`
subagents so the parent keeps only the synthesis — the same pattern
`.claude/skills/harness-audit/SKILL.md` Phase 1 uses. Delegation is a
context-economy optimisation, **not a correctness requirement**: the
extraction rules and the currency-oracle discipline are identical
whether a subagent or the parent runs them.

**If the `Agent` / `Explore` tool is unavailable** (some harnesses do
not expose subagent dispatch), do NOT abort and do NOT improvise an
ad-hoc shape. Run the *same* Phase-1 and Phase-2 procedures **inline
in the parent**, against the same date-windowed file list, preserving
every discipline the subagent prompts below specify verbatim:
target-section-only extraction (Phase 1), Grep/Glob artifact
spot-checks with bias-toward-STALE-only-on-positive-evidence
(Phase 2), and absence-of-evidence ⇒ LIVE. The only difference is
*who* executes; the output contract (the structured proposal list,
then the verdict table) is unchanged.

When the inline path is taken, add one line to the report header:
`Delegation: inline (Agent/Explore unavailable in this harness)` so a
reader knows the parent did the read work and the token cost reflects
that. When delegation succeeds, the header line is
`Delegation: Explore subagents (Phase 1 ×1, Phase 2 ×<n> batches)`.
This is not cosmetic — it lets a future cadence decision
(ad-hoc vs folded into `weekly-review`) account for the real
context cost on the harness that will actually run it.

Lazy-load discipline either way: do not pre-read rule tables, lesson
corpora, or the full retro prose. Phase 1 reads only the named
sections; Phase 2 reads only what a grep hit needs for context. The
21-day window exists to bound the read; honour it.

## Phase 1 — Extract open proposals (preferred: one Explore subagent; inline fallback per Delegation policy)

The retro corpus is large (20-50 files in-window). The parent only
needs the *extracted proposals*, not the retro prose — so the
preferred path delegates this to one `Explore` subagent (per the
Delegation policy above). **If subagent dispatch is unavailable, run
the identical extraction inline** — same file list, same
target-section-only rule, same output contract.

Whether delegated or inline, the extraction prompt/procedure is the
same and MUST be followed verbatim — paraphrasing loses the
section-name precision that makes extraction reliable. When
delegating, the `Explore` prompt is self-contained (the subagent has
no conversation context):

> Working directory: `<repo-root>`. You are inventorying unprocessed
> proposals across retro files. Today is `<YYYY-MM-DD>`.
>
> For each retro file in this list: `<explicit file list from Phase 0
> glob, filtered to the effective date window>`
>
> Read the file and extract ONLY these sections (skip all other
> prose to conserve context):
> 1. **"Promotion candidates"** section — every checkbox line.
>    Record the checkbox state (`[ ]` unchecked vs `[x]`/`[X]`
>    checked) and the proposal text verbatim.
> 2. **"What to change"** table — every row. Record the change
>    text, expected effect, and any cost/recurrence column.
> 3. **"What to carry forward"** — only items phrased as a NEW
>    practice to adopt (skip pure observations).
> 4. **"Decisions to revisit"** — every bullet.
> 5. Any **"## Watch"** or **"Watch / promote-if-recurs"** items.
>
> A proposal is "open" if: a Promotion-candidates box is unchecked,
> OR a "What to change" row has no visible evidence in the same
> retro that it was actioned (no "SHIPPED", no commit SHA, no
> strikethrough). Checked boxes and explicitly-SHIPPED rows are
> "closed" — record them too but tag `closed`.
>
> Return a flat list, one entry per proposal:
> `{retro_file, retro_date, section, proposal_text (≤140 chars),
> state: open|closed, raw_evidence_in_retro}`. Group by retro file,
> newest first. Do NOT triage currency — that is a later step. Do
> NOT read files outside the provided list. Read-only.

The parent receives the structured list. Print a count line:
`extracted: <X> open proposals, <Y> already-closed, across <Z> retros`.

## Phase 2 — Currency triage (preferred: parallel Explore batches; inline fallback per Delegation policy)

Now each *open* proposal needs a currency verdict checked against the
**live codebase**, not against the retro that proposed it. This is the
skill's core judgment and the most error-prone if done from memory —
a proposal "add lesson X" is STALE only if `X` actually exists in
`.claude/lessons/` *now*.

**Preferred:** split the open-proposal list into batches of ~10 and
launch one `Explore` subagent per batch **in parallel** (single
message, multiple Agent calls). **If subagent dispatch is
unavailable** (per the Delegation policy): process the same batches
**sequentially inline in the parent**, applying the identical
verification recipe and evidence discipline below to each proposal.
Inline is slower and costs parent context but produces an identical
verdict table — the iteration-1 runs confirmed inline triage reaches
the same STALE/LIVE/SUPERSEDED calls when the discipline is held.

Each batch (delegated subagent prompt, or the inline checklist the
parent follows) is self-contained and carries its proposals inline:

> Working directory: `<repo-root>`. Trunk HEAD: `<sha-short>`. You are
> triaging the *currency* of retro proposals against the CURRENT
> codebase. For each proposal below, decide one of:
>
> - **STALE** — already implemented. Verify by checking the actual
>   artifact: if the proposal is "promote `feedback_foo.md`", Glob
>   `.claude/lessons/feedback_foo.md`. If "add rule X to
>   advisor-orchestrator.md §N", Grep that file for the rule text /
>   concept. If "fix script Y", Grep `Y` for the fix. If "the
>   problem is Z", check whether Z still exists. Cite the file:line
>   or glob hit as evidence.
> - **LIVE** — not yet implemented; the artifact is absent / the rule
>   lacks the change / the gap remains. Cite the negative check
>   (e.g. "Glob `.claude/lessons/feedback_foo.md` → no match").
> - **SUPERSEDED** — a later retro or a subsequent ADR/DQ addressed
>   the same thing differently or reversed the premise. Cite the
>   superseding artifact path.
>
> Proposals to triage (batch <i> of <n>):
> `<inline list of {retro_file, retro_date, proposal_text} for this batch>`
>
> Use Grep/Glob to spot-check — do NOT read whole files unless a
> grep hit needs context. Bias toward STALE only with positive
> evidence; if you cannot find the artifact, it is LIVE (absence of
> evidence = not yet done). Return a table:
> `{retro_file, proposal_text (≤120 chars), verdict:
> STALE|LIVE|SUPERSEDED, evidence (≤120 chars)}`. Read-only.

Collect all batch tables. If two subagents disagree on near-identical
proposals (same text, different retros), keep both rows but note the
duplication — that itself is a SUPERSEDED signal worth surfacing.

**Currency oracle discipline:** the verdict is only as trustworthy as
the evidence string. A STALE verdict with no file:line or glob hit is
not STALE — downgrade it to LIVE and flag "unverified STALE claim" in
the report. Per `.claude/lessons/feedback_runbook_audit_drift_post_event_check.md`
(every runbook/audit claim is a hypothesis until checked against
reality) and the user-memory discipline "verify retro substantive
supersede claims": every "already done" claim is a hypothesis until
the artifact is shown at HEAD.

## Phase 3 — Rank by leverage

Not all LIVE proposals are equal. Rank so the user acts on the
highest-value items first. Leverage = **recurrence × impact ÷ cost**,
estimated from the retro evidence (the skill does not re-derive
metrics, it reads what retros recorded):

- **Recurrence:** how many distinct retros raised the same or a
  near-identical proposal. ≥2 distinct retros = high; the
  `feedback_principles_not_rules.md` promotion bar is 2+ for a lesson,
  3+ for a canonical pattern — apply the same threshold here.
- **Impact:** from the retro's own cost/effect column where present
  ("saves 30 min per cycle", "3 phantom failures"). Absent → infer
  from scope words (a hard-refusal/catch-fire rule outranks a
  doc-wording tweak).
- **Cost:** the retro's cost tag (minor / medium / major) where
  present; a 1-line rule edit is minor, a new skill is major.

Produce three tiers:

- **Tier 1 — act now:** LIVE, recurrence ≥2 OR high impact, cost
  minor/medium. These pay back fast and the recurrence proves the
  pain is real.
- **Tier 2 — schedule:** LIVE, single-occurrence but real, OR
  high-cost-high-value (worth a dedicated session).
- **Tier 3 — closeable:** STALE + SUPERSEDED. The user just checks
  boxes / collapses duplicates; no implementation work.

## Phase 4 — Write the report

Output path:
`.claude/PRPs/reports/retro-harvest-<YYYY-MM-DD>.md` (UTC date). If
that file already exists from an earlier run today, append `-v2`
(then `-v3`) — never clobber; harvest reports are a journal of what
was open when, and overwriting destroys the "what did we know on
date D" audit trail (same discipline as the `session-retro` hard
refusal on clobbering retros).

Use this exact structure:

```markdown
# Retro harvest — <YYYY-MM-DD>

**Scope:** <N> session-retros + <M> phase-retros | window <since|last-21d|all>
**Branch / HEAD:** `<branch>` @ `<sha-short>`
**Delegation:** <`Explore subagents (Phase 1 ×1, Phase 2 ×<n> batches)` | `inline (Agent/Explore unavailable in this harness)`>
**Date oracle:** <`filename dates` | `git-author-date override — N phase-retros shared a clone-artifact mtime`>
**Extracted:** <X> open proposals, <Y> already-closed, across <Z> retros

## TL;DR

<One paragraph: how many LIVE Tier-1 items, the single highest-leverage
one, and how many STALE items are just sitting unchecked. Reads cleanly
cold.>

## Tier 1 — act now (LIVE, high recurrence or high impact, low cost)

| # | Proposal | Source retro(s) | Recurrence | Cost | Evidence it's still LIVE | Suggested next action |
|---|---|---|---|---|---|---|
| 1 | ... | <file> (+N more) | Nx | minor | <neg-check> | <1-line: edit file X / write lesson Y> |

## Tier 2 — schedule (LIVE, single-occurrence or high-cost-high-value)

| # | Proposal | Source retro | Cost | Evidence it's still LIVE | Note |
|---|---|---|---|---|---|

## Tier 3 — closeable (STALE / SUPERSEDED — just check the box)

| Proposal | Source retro | Verdict | Evidence | Action |
|---|---|---|---|---|
| ... | <file> | STALE | <file:line proving done> | tick `[x]` in source retro |
| ... | <file> | SUPERSEDED | <superseding artifact> | collapse into <newer> |

## Unverified STALE claims (downgraded to LIVE — needs a human look)

<Any proposal a triage subagent called STALE without citable evidence.
Listed separately so the user knows these are low-confidence.>

## Suggested action sequence

<Ordered, copy-pasteable. The skill does NOT execute these — it hands
them to the user. Example:
1. `git mv` / write `.claude/lessons/feedback_<x>.md` (Tier 1 #1)
2. Edit `.claude/rules/advisor-orchestrator.md` §5.3 to add <rule> (Tier 1 #2)
3. Tick the 6 STALE boxes in their source retros (Tier 3)>

---

_Generated by `.claude/skills/retro-harvest/SKILL.md`. Read-only sweep;
no rules/lessons/code modified. Currency verdicts are evidence-checked
against HEAD `<sha-short>` — re-run after acting to confirm closure._
```

The CRITICAL property: every Tier-1/2 row has a **negative-evidence
string** proving it's still LIVE, and every Tier-3 row has a
**positive-evidence string** proving it's done/superseded. A row
without its evidence is not actionable — the user can't trust a
verdict they can't see the basis for. If the triage subagents
returned verdicts without evidence, surface them in the "Unverified"
section rather than silently trusting them.

## Phase 5 — Surface to user

Return a compact closing summary in the conversation (NOT the whole
report — it's on disk):

```
Retro harvest written: .claude/PRPs/reports/retro-harvest-<date>.md

Tier 1 (act now): <count> — top: <one-line highest-leverage item>
Tier 2 (schedule): <count>
Tier 3 (closeable STALE/SUPERSEDED): <count> — just box-ticking
Unverified STALE downgraded: <count>

Highest-leverage next action: <the single thing worth doing first>
```

Then stop. The user decides what to action. Do NOT proactively start
implementing Tier-1 items — that is a separate explicit request, and
auto-acting on a triage you just produced repeats the exact
scope-violation class this skill exists to make visible (a retro
proposal is a *proposal*, not an instruction).

## Hard refusals

- **Never edit a rule, lesson, skill, script, or code file.** This
  skill writes exactly one file: the harvest report under
  `.claude/PRPs/reports/`. If you feel pressure to "just apply the
  one-line fix while I'm here," stop — that's the user's call, and
  conflating harvest with action is the failure mode this skill
  diagnoses.
- **Never tick a checkbox in a source retro.** Even for a verified
  STALE item, mutating the source retro is an action, not a harvest.
  The report tells the user which boxes to tick; the user (or a
  later explicit request) does it.
- **Never trust a STALE verdict without citable evidence.** Absence
  of proof that something shipped means it's LIVE. Downgrade
  unverified STALE → LIVE and list it in the "Unverified" section.
  Per `feedback_runbook_audit_drift_post_event_check.md` — claims
  are hypotheses until the artifact is shown.
- **Never clobber an existing harvest report.** Append `-v2`/`-v3`.
  The dated journal is the point.
- **Never re-derive retro metrics.** The skill reads recurrence/cost
  from what retros recorded; it does not re-run complexity scoring or
  re-time tasks. If a retro lacks a cost tag, infer coarsely from
  scope words and say "(inferred)" — don't fabricate a number.
- **Never widen scope to non-retro files.** `decision-queue.json`
  log-entries, runlogs, and commit `LESSON:` trailers have their own
  harvest path (the advisor at retro time per
  `feedback_retro_not_report.md`). This skill is retro-markdown only.

## When NOT to use this skill

- The user just finished a session and wants to record it → that's
  `session-retro` (writes a new retro). This skill reads existing
  ones.
- Sunday memory-health sweep → that's `weekly-review` (PMD prune +
  promote). This skill doesn't touch PMD.
- The user wants to action ONE specific known proposal they already
  have in mind → just do that directly; no need to sweep 50 files.
- Closing a sub-phase → the canonical phase-retro + the advisor's
  in-line `LESSON:` harvest per `feedback_retro_not_report.md`
  already covers that lane's proposals at ship time.

## Why this skill exists

Every retro is a small deposit of hard-won judgment. Without a
harvester, those deposits sit in `.claude/PRPs/reports/` earning no
interest — the proposals that would prevent the next repeat of a
known failure mode are written down and then never seen again. The
2026-05-16 analysis found ~48 unchecked proposals across 52 retro
files, including a `cycle_count ≥ 3` catch-fire rule proposed 7 days
earlier that would have prevented a ~123-min loss had it been
actioned. The checkbox UI in the retro template implied tracking
that never existed.

This skill makes the implicit explicit: it is the periodic (or
on-demand) reader the corpus assumed but never had. Run it ad hoc
when the retro backlog feels stale, or fold it into a cadence (e.g.
alongside `weekly-review`) so proposals get a human decision within
days of being written instead of never. ~10 minutes of triage in
exchange for not silently re-walking into failure modes a past
session already diagnosed.

## See also

- `.claude/skills/session-retro/SKILL.md` — writes the retros this
  skill harvests; its "Promotion candidates" checkbox section is the
  primary input.
- `.claude/skills/weekly-review/SKILL.md` — the PMD-memory analogue;
  this skill is its retro-markdown counterpart.
- `.claude/lessons/feedback_retro_not_report.md` — the harvest-step
  discipline for `LESSON:` trailers + DQ log entries (the non-retro
  harvest path this skill deliberately excludes).
- `.claude/lessons/feedback_runbook_audit_drift_post_event_check.md`
  — the "every claim is a hypothesis, verify against reality"
  discipline the currency oracle enforces.
- `.claude/skills/harness-audit/SKILL.md` — the read-heavy-sweep
  delegation pattern Phases 1-2 mirror (Explore subagent returns
  synthesis; parent context stays small).
