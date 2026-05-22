# Command retro — /<verb> — <YYYY-MM-DD>

**Spec path:** `<path-to-spec.md>`
**Spec last modified:** `<sha-short>` (`<YYYY-MM-DD>`)
**Invocations counted:** <N>
**Score sources:** PMD (<n> rows) · retros (<n> files) · DQ (<n> entries)
**Composite score:** <0.00-1.00> (anti-inflation per `.claude/rules/evaluation-calibration.md`)

## TL;DR

<One paragraph. The verb's main friction pattern (if any), the
highest-recurrence proposal, and the net-min number. Reads cleanly
out of context six months from now.>

---

## Findings

### What surprised us

<Per `.claude/lessons/feedback_retro_not_report.md`. Things this
verb did that the user or advisor didn't expect — surprising
friction OR surprising effectiveness. One bullet per item; each
must cite the source (PMD row id, retro file:section, or DQ #id).>

- <e.g. "bm-merge fired L14 fallback (advisor re-applied runlog
  COMPLETE entry post-merge) 4 times in 6 invocations — BM Junior
  systematically skips the post-merge runlog commit despite the
  brief ordering it. Cite: PMD #1042, session-retro-2026-05-22.">
- ...

### What's working

<Patterns the verb gets right. Worth recording so future sessions
don't undo them by accident.>

- <e.g. "L15 inline gate-side checks — zero pre-confirm Junior
  dispatches across the phase; ~80% context boot savings holds.">
- ...

---

## Three-signal scoring per invocation

Per `.claude/lessons/feedback_four_role_retro_signals.md`. Numbers
defensible from the cited source.

| # | Invocation context | Saved (min) | Wasted (min) | Surprise | Source |
|---|---|---:|---:|---|---|
| 1 | <e.g. PR #144 merge gate> | 8 | 12 | high | session-retro-2026-05-22.md §3 |
| 2 | <e.g. PR #141 merge gate> | 8 | 0 | none | PMD eval #1039 |
| ... | | | | | |

**Composite metrics:**

| Metric | Value |
|---|---|
| Invocations counted | <N> |
| Total saved (min) | <sum> |
| Total wasted (min) | <sum> |
| **Net (min)** | <saved - wasted> |
| Surprise events (med+high) | <count> |
| Friction score | <0.00-1.00> |
| **Composite score** | <1 - friction> |

---

## Class A — Minor edits auto-applied

<Per `.claude/skills/command-retro/SKILL.md` Step 3a. Each row had
recurrence ≥ 3 OR was objectively-broken (404, missing file, etc).
All diffs already applied and `git add`-staged for the commit at
end of run.>

| # | Issue | File:line | Before | After | Recurrence |
|---|---|---|---|---|---|
| A1 | <e.g. broken lesson ref> | `<spec>:42` | `feedback_runlog_post_merge.md` | `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md` | objective (file renamed 2026-05-18) |
| A2 | ... | | | | |

If no Class A edits were applied this run, replace this section with
"_No minor edits applied._".

---

## Class B — Major proposals (surfaced for user review)

<Per Step 3b. Each proposal must include a verbatim before → after
diff. The user reviews, then either applies via Edit, asks for a
different shape, or declines.>

| # | Proposal | Evidence | Recurrence | Suggested diff |
|---|---|---|---|---|
| B1 | <one-line description> | <cite PMD/retro/DQ> | <N invocations> | <see below> |

### B1 — <one-line> (detailed diff)

**File:** `<spec-path>`
**Lines:** <start>-<end>

**Before:**
```markdown
<verbatim current text>
```

**After:**
```markdown
<verbatim proposed text>
```

**Rationale:** <2-3 sentences explaining the change and citing the
recurrence pattern.>

**User decision:** ☐ Apply  ☐ Modify (note: ____)  ☐ Decline

---

## Class C — Watch-list entries added to MEMORY.md

<Per Step 3c. Each row recurs at count = 2; promoted to MEMORY.md
"Watch / promote-if-recurs" section. If it recurs a third time, it
escalates to a Class A or Class B proposal.>

| # | Pattern | Linked file in `~/.claude/projects/.../memory/` |
|---|---|---|
| C1 | <e.g. "bm-poll-cr returns 0 findings when CR rate-limited"> | `watch_bm_poll_cr_zero_on_rate_limit_20260522.md` |
| ... | | |

---

## Spec-vintage signal

| Signal | Value |
|---|---|
| Spec last modified | <YYYY-MM-DD> (`<sha-short>`) |
| Age in days | <N> |
| Vintage class | recent (< 7d) / mid (< 90d) / stale (> 90d) |
| Interpretation | <one of the canonical readings — see SKILL.md Step 1d> |

A spec in the "recent" class with high friction is a regression
signal — the recent change introduced the friction. A "stale" spec
with rising friction is a drift signal — the world moved around the
spec.

---

## Decisions to revisit

<Optional. List anything that warrants its own follow-up — a
separate retro session, a clarify pass, an architectural review of
the verb's place in the harness. One line per item.>

- <e.g. "If Class B #1 ratified, the L14 fix's pre-merge ordering
  in `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md`
  needs a follow-up paragraph noting the BM-skip pattern.">

---

## Promotion candidates (recurrence ≥ 2)

For each surfaced finding that the user wants promoted further, the
boxes below are checked manually. Unchecked by default.

- [ ] Class B proposal #__ → ratify and apply spec edit
- [ ] Class C watch entry #__ → promote to `.claude/lessons/feedback_<topic>.md`
- [ ] Surprise finding #__ → write a `pattern` PMD memory for cross-session reach
- [ ] Spec-vintage signal → propose a regular cadence (e.g. quarterly sweep of stale verbs)

---

_Generated by `.claude/skills/command-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_principles_not_rules.md`. Calibration:
`.claude/rules/evaluation-calibration.md`._
