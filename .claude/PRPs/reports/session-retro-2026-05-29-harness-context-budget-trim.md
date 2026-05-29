# Session retro — 2026-05-29 — harness-context-budget-trim

**Harness:** claude-code
**Session window:** ~2026-05-29 (spanned one `/compact`) → 2026-05-29 (~3–4h wall-clock across both halves)
**Branch at start:** `9ef77896d` (`governance-v0`)
**Branch at end:** `94b63a519` (`governance-v0`)
**Files touched:** 21 (this session's commits; excludes the other session's interleaved rt-r3-followup commits)
**Commits:** 5 mine (`8478f703b`, `4d8a643d7`, `f9982bd6e`, `43ef28f8c`, `94b63a519`), all explicit `chore/docs` — 2 others' (`1ff08b915`, `2e5da2ad9`) interleaved on shared trunk

## TL;DR

A harness-leanness session: ran `harness-audit` on the rules corpus, then applied user-approved
Pi-safe context-budget trims, fixed a quiet PMD-recall bug, and shipped one enforcement hook. The
most load-bearing finding inverted the going-in premise: the rules corpus is **already lean and
dual-harness shared** (`.pi/` cites the top rule files 68× by named section anchor), so one-time
extraction yields only ~1% (~1.8k tokens), not the 6–10k projected — and the chars/4 token estimate
under-counts markdown by ~1.6× (real ALWAYS-load ≈ 70.8k ≈ 35% of the 200K working budget). The
durable lever is **growth-discipline at author time**, now shipped as a lesson + a §3.6 rule clause +
a PostToolUse bloat hook. Top carry-forward: when auditing for context savings, the win on an
already-trimmed corpus is small — measure with the right token ratio *before* projecting, and invest
in prevention over one-time cuts.

---

## What surprised us

- **The chars/4 token heuristic under-counts markdown by ~1.6×.** Phase-1 audit math (chars/4) gave
  advisor-orchestrator = 11.9k tokens; live `/context` showed **19.8k**. Across the corpus the real
  ALWAYS-load is ~70.8k (~35% of 200K), not the ~42.7k/21% the audit concluded. My savings projection
  (6–10k) was built on the wrong ratio and was ~5× the actual outcome (~1.8k).
- **The rules corpus is a dual-harness shared contract, not Claude-Code-owned.** `.pi/` references the
  top rule files **68× across 18 files**, predominantly by *named section anchor* (`advisor-orchestrator.md
  "Stage-shape orchestration"`). This silently bounds every extraction: moving a heading breaks Pi. The
  going-in framing ("rules are the 86% lever, go cut them") didn't account for this at all.
- **`memory_search_hybrid` blew context twice — in a session about saving context.** Two broad recall
  queries each returned 85K–105K-char dumps auto-spilled to disk. Ironic, and a real cost: I switched to
  `grep`-ing the lessons dir for the coverage check instead.
- **8 lesson files had no YAML frontmatter and had been silently invisible to PMD recall for weeks**
  (`errors: 8` on every sync). They were authored in an H1+TL;DR style that the sync regex rejects. Real,
  promotable lessons that no `memory_search_hybrid` could ever surface.
- **A path-glob bug in the new hook passed `bash -n` while being logically wrong** (`*/.claude/rules/`
  required a leading slash; relative paths didn't match). Only the dogfood (test against reality) caught
  it — exactly the session's own recurring theme firing on my own code.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **DONE this session** — `harness-audit` now documents ~2.4× chars→token for markdown (report execute-pass actuals + new lesson calibration note). | Future audits project savings with the right ratio; no 5× over-projection. | minor (shipped) | 1× this session (projection miss) |
| 2 | **DONE this session** — new lesson `feedback_rule_narrative_to_refs_at_author_time.md` + paired §3.6 rule clause + PostToolUse hook `rule-narrative-bloat-reminder.sh`. | Converts growth-discipline from advice → mechanism; the corpus stops re-bloating one incident-narrative at a time. | medium (shipped) | 1× here + recurs against `feedback_lesson_must_pair_with_structural_fix` |
| 3 | **DONE this session** — `pmd-search-strategy.md` output-size footgun note (broad hybrid queries → 85K-char dumps; prefer tight queries or `grep` lessons dir for coverage). | Stops recall queries from costing more context than they save, esp. in budget sessions. | minor (shipped) | 2× this session |
| 4 | **DONE this session** — fixed 8 frontmatter-less lessons (now `imported: 8, errors: 0`). | 8 unfindable lessons become recallable; every future session's hybrid recall improves. | medium (shipped) | 1× (but 8 files = systemic) |
| 5 | **Carry-forward** — add a CI/pre-commit (or weekly-review step) lint that fails if any `.claude/lessons/feedback_*.md` lacks `^---` frontmatter. | Prevents the silent-skip class from recurring; the 8 broke over weeks with zero signal. | medium | 1× here (propose if a 9th appears) |
| 6 | **Carry-forward** — the scoring-matrix redundancy term over-counts on a shared-keyword grep (P3 self-rejected: "SessionStart" matched 4 distinct invariants). Add an execute-pass prose-diff confirmation before acting on a redundancy score. | Audit doesn't waste a cut proposal on coincidental keyword overlap. | minor | 1× here (noted in report + lesson; promote if 2nd audit hits it) |

## What to carry forward

- **Coverage-check before writing lessons.** This session I `grep`-ed the lessons dir first and confirmed
  the main themes were already captured — avoided 3+ redundant lesson writes. Cheap, prevents corpus noise.
- **Stage only my exact files; `git status` between add and commit.** Held across 3 commits on a trunk
  the other session was actively committing to (`1ff08b915`, `2e5da2ad9` interleaved). Zero attribution
  collisions. Per `feedback_cross_session_commit_attribution_collision.md` — proven again.
- **Dogfood hooks/scripts against crafted real inputs, not just `bash -n`.** Caught a logic bug that
  passed syntax check. 7 cases (fire / has-refs / scoped / non-rules / small / Write / rel-vs-abs path).
- **Trust-but-verify subagent output.** The frontmatter-fix subagent reported "all 8 OK"; I verified the
  *real* outcome (sync `imported: 8, errors: 0` + memory #627 + 529 vectors), not just its claim.
- **Self-reject cuts on execute-pass.** P3 (pmd-invariants redundancy) looked good in the audit but was
  coincidental keyword overlap on inspection — dropped it rather than force a bad cut. The memory-prune
  Step 3.5 "is-it-the-mechanism" check working as designed.
- **Honest reckoning over flattering numbers.** Reported the ~1.8k actual against my 6–10k projection
  plainly, with the root cause (wrong token ratio applied before measuring). The audit's value was the
  Pi-boundary finding, not the token count.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `harness-audit` (Phases 0–7) | 40 | 10 | high | Pi-boundary finding (68× refs) inverted the premise; chars/4 under-count surfaced only at apply-time, not in the audit math |
| Phase-1 Explore subagent (pre-compact) | 25 | 0 | low | inventory persisted to a resume artifact; survived `/compact` cleanly — no re-run needed |
| frontmatter-fix subagent (8 lessons) | 30 | 0 | medium | parallel read-and-synthesize kept out of main context; verified real outcome after |
| `memory_search_hybrid` ×2 | 0 | 8 | high | 85K–105K-char dumps spilled to disk; switched to `grep` lessons dir — the footgun is now §3 #3 |
| `rule-narrative-bloat-reminder.sh` dogfood | 15 | 5 | medium | caught a path-glob bug (`*/.claude` → `*.claude`) that `bash -n` passed; 7/7 after fix |
| AskUserQuestion (scope + depth) | 5 | 0 | none | clean fork on which levers to apply + conservative-vs-aggressive depth |
| `AskUserQuestion` (wrap-up items) | 3 | 0 | none | clean 3-item selection for the close-out |
| MEMORY.md byte-prune (manual edits) | 10 | 3 | low | over-edited slightly before re-measuring bytes; landed 210-byte headroom |

## Complexity scores (heavy tasks only)

No Junior impl-tasks ran this session (manual advisor-side meta-work), so the canonical
`files/commits/runtime/log-silence` impl metric is dormant. Advisor-side heavy-task proxy
(files-touched / commits):

| Task | Files | Commits | Notes |
|---|---:|---:|---|
| harness-audit apply (P1–P3 + skills) | 7 | 1 (`f9982bd6e`) | conservative extraction; P3 self-rejected |
| growth-discipline lesson + §3.6 + report actuals | 3 | 1 (`43ef28f8c`) | paired lesson+structural-fix |
| wrap-up (8 lessons + search note + hook) | 11 | 1 (`94b63a519`) | subagent for the 8; hook dogfooded |

None stressed any envelope (no watchdog — laptop session). Largest single commit = 11 files, all
small frontmatter prepends + one hook.

## Decisions to revisit

- **P4 (multi-lane-worktree Mode-A/B extract) was deferred**, gated on a per-section `.pi/` grep
  ("Lane modes" / "trunk→phase sync"). A future harness-leanness pass should run that grep and either
  extract or close it out — it's the one remaining high-confidence row left unactioned.
- **`tavily` MCP server** — if advisor sessions never web-search, dropping it from `.mcp.json` removes
  its schemas from the deferred pool. Left as the user's call; worth a one-line confirm next session.
- **Frontmatter lint (§3 #5)** — propose concretely if a 9th frontmatter-less lesson appears, or fold
  into weekly-review Step 1 as a cheap guard now.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [x] **DONE** — `feedback_rule_narrative_to_refs_at_author_time.md` written + synced + embedded (§3 #2).
- [x] **DONE** — `pmd-search-strategy.md` output-size note (§3 #3).
- [x] **DONE** — `rule-narrative-bloat-reminder.sh` hook + `.claude/settings.json` wiring (§3 #2).
- [ ] **§3 #5** — frontmatter-presence lint for `.claude/lessons/*.md`: add to weekly-review Step 1 OR a pre-commit hook. (1× here; promote on 2nd occurrence or fold into weekly-review now.)
- [ ] **§3 #6** — scoring-matrix redundancy over-count calibration: already noted in the audit report + the new lesson; promote to a `helpers/scoring-matrix.md` edit if a 2nd audit hits it.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. No `/auto-phase` artifacts active this session —
reliability section omitted per Step 0.5._
