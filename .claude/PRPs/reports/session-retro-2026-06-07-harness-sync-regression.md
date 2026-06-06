# Session retro — 2026-06-07 — harness-sync-regression

**Harness:** claude-code
**Session window:** 2026-06-07 (~session-length; harness-audit thread + m2-late planning prep)
**Branch at start:** `0001895f9` (`governance-v0`)
**Branch at end:** `7f4710c31` (`governance-v0`) + homeserver `6e833c3` (`main`)
**Files touched:** 15 (brehon-fork) + 1 (homeserver)
**Commits:** 7 (brehon-fork) + 1 (homeserver) — all explicit, 0 auto

## TL;DR

A `/harness-audit` run surfaced that the always-load rules corpus had **grown +7,835 chars since the prior audit** — and the cause was a *regression of a previously-shipped trim*. The 2026-05-31 consolidation of four rules into `universal-guards.md` was silently reverted on 2026-06-04 by the cross-repo sync tool (`sync-shared-skills.sh`), which still carries the four originals as canonical source and re-shipped them under a `chore: update skill sync manifest` auto-commit authored by a BM-task session. The most load-bearing finding: **the failure was invisible for 3 days because nothing re-checks that an applied context-budget trim stays applied.** The session fixed it at three layers — source-side (`RULE_EXCLUDE` per-repo exclusion), state-side (re-deleted the 4 files, ~3.16k tokens recovered), and detection-side (a new `harness-regression-guard.sh` SessionStart hook reading a tombstone ledger). The top carry-forward: when fixing a regression, the durable fix is at the *source* that re-introduces it, not the local symptom — and a guard against recurrence needs a ledger it reads, not a hardcoded list.

---

## What surprised us

- **The audit's headline was a regression, not drift.** Going in, the expectation was a routine "what's grown, what can be scoped" pass. Instead the +7,835-char growth traced to a single sync auto-commit undoing a shipped consolidation. A harness audit doubled as a regression detector — an unplanned but high-value use.
- **A BM-task session mutated `.claude/rules/**`** under a `chore: update skill sync manifest` subject. Per `branch-manager.md` HARD file-ownership, BM never touches rules — yet the *sync tooling* (not the BM agent reasoning) re-materialised them as a side effect of `git add -A .claude/`. The misleading commit subject masked a rules-corpus mutation. Surprising that the violation came through tooling, not agent judgment.
- **The prior audit (2026-05-31) predicted this exact trigger** ("Redundancy in a consolidated error-handling super-file") as the promotion condition for circuit-breaker/escalation — and then the super-file was built *and* the originals came back, and the next audit (this one) was the first to notice. The prediction was correct; the detection lag was the gap.
- **The Phase-1 Explore subagent's summary arithmetic was wrong** (claimed 204,213 ALWAYS chars; true value 118,673). Per-file class assignments were all correct (matched the deterministic grep), but the aggregate row was fabricated. Reinforces: trust subagent *data collection*, re-derive subagent *summaries*.
- **The prior audit's own table omitted `multi-lane-worktree.md`** (8.1k chars, ALWAYS) — the exact blind spot the skill's dogfood block warns about, recurring a third time.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **DONE this session** — add `harness-regression-guard.sh` SessionStart hook + `.claude/refs/harness-deleted-rules.txt` ledger | Silent re-add of a deleted always-load rule surfaces a WARN at next session start instead of going undetected for days | medium | 1× this session (the originating incident); guard now standing |
| 2 | **DONE this session** — `RULE_EXCLUDE` per-repo map in `homeserver/scripts/sync-shared-skills.sh` | Sync no longer re-ships brehon-fork's consolidated rules; self-cleans stale daemon copies on next run | medium | 1× (root cause); durable |
| 3 | **Audit summary rows must be parent-re-derived, never trusted from the Explore subagent.** Add an explicit instruction to `harness-audit/SKILL.md` Phase 1: "the subagent's summary totals are advisory; recompute ALWAYS-char totals parent-side with `wc -c` before scoring." | Stops fabricated aggregates (204k vs 118k this session) from reaching Phase 3 scoring | minor | 2× (this session + the 2026-05-31 subagent-drift note) → **meets promotion threshold** |
| 4 | **`multi-lane-worktree.md` keeps falling out of the audit's ALWAYS table.** The Phase-0 grep now catches it, but the report-writing step should cross-check the Phase-0 grep set against the rendered inventory table and flag omissions. | Closes the recurring blind spot (3rd occurrence) | minor | 3× across audits → **meets promotion threshold** |
| 5 | **Misleading commit subjects on sync auto-commits.** `sync-shared-skills.sh` line 295 hardcodes `chore: update skill sync manifest` even when it ships/removes rule *content*. Propose: include a one-line summary of what changed (e.g. `+N rules, -M rules, K hooks`) so a rules-corpus mutation is not masked as manifest housekeeping. | A future `git log` review catches content changes hidden under a housekeeping subject | minor | 1× this session, but directly caused the 3-day detection lag |

## What to carry forward

- **Fix the source, not the symptom.** When a regression is re-introduced by tooling, the durable fix is in the source that re-introduces it (the sync `RULE_EXCLUDE`), not the local deletion. Doing P1 alone would have been re-reverted next sync. Surfaced and applied cleanly — carry this as the default reflex for any "this came back" finding.
- **Falsifiable-hypothesis discipline paid off twice.** Before deleting the 4 files I grep-verified each rule's substantive content was present verbatim in `universal-guards.md` (one false-alarm "MISSING" turned out to be `Maximum`→`Max` abbreviation). Before recommending the sync fix I traced the actual re-add commit (`999317082`) rather than assuming. Both per `feedback_falsifiable_hypothesis_before_structural_fix.md`.
- **Surface outward-facing actions for confirmation, don't auto-run.** Held the two pushes + the `sync-shared-skills.sh` run for explicit user go-ahead (sync SCPs to 8 production repos). Per the autonomy bounds — worked cleanly, user authorised pushes, then sync separately.
- **Test guards against reality, not syntax.** The regression-guard was validated with a simulated reappearance (create file → WARN → cleanup → silent), not just `bash -n`. Per `pattern_test_against_reality_not_syntax`.
- **A ledger beats a hardcoded list for a guard's source of truth.** The guard reads `.claude/refs/harness-deleted-rules.txt`, so it stays correct as future audits add deletions — no code edit per new tombstone.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/harness-audit` skill | 40 | 5 | high | Found a regression a routine audit wasn't looking for. 5 min wasted re-deriving the subagent's wrong summary totals. |
| Phase-1 Explore subagent | 15 | 5 | medium | Correct per-file data; fabricated summary row (204k vs 118k true). Net positive but needed parent re-derivation. |
| Phase-0 Step-6 grep (deterministic class) | 10 | 0 | none | Caught all SCOPED/ALWAYS classes correctly; no class-mismatch. The mechanical-classification fix from a prior audit held. |
| Git-archaeology (commit trace `7f3056c9a`→`999317082`) | 25 | 0 | high | Pinpointed exact re-add commit + author + mechanism. The single most load-bearing investigation step. |
| `AskUserQuestion` (fix strategy) | 5 | 0 | none | Clean 3-way fork on per-repo-exclude vs consolidate-everywhere vs symptom-only. Right call to surface — blast radius spanned 8 repos. |
| Regression-guard authoring + live test | — | 0 | low | New standing detection; validated clean/regression/clean. |

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. Format: `<files>/<commits>/<runtime-min>/<max-log-silence-min>`.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| harness-audit run (read-only report) | 1 | 1 | ~25 | <2 |
| sync-tool fix + P1 (homeserver + brehon-fork) | 7 | 3 | ~30 | <2 |
| regression-guard (hook + ledger + skill + bootstrap + wiring) | 5 | 1 | ~20 | <2 |

No task exceeded the >55min runtime / >40min silence / >8 files flags. Healthy granularity throughout.

## Decisions to revisit

- **Should the regression-guard be a shared hook?** It is currently brehon-fork-canonical-scoped (guards the brehon-fork-only `universal-guards.md` consolidation). If another repo consolidates rules later, the guard + ledger pattern would need replicating. Defer until a 2nd repo consolidates — single-repo for now is correct.
- **The bigger always-load levers remain untouched.** `advisor-orchestrator.md` (17.3k tokens — 27% of the memory bucket) and `MEMORY.md` (11.3k, near the 195-line truncation ceiling) are the real heavyweight trims (P2/P3 + a `memory-prune` job). Worth a dedicated narrative→refs extraction session.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Change #3** (re-derive audit summary totals parent-side): update `.claude/skills/harness-audit/SKILL.md` Phase 1 — 2× recurrence (this session + 2026-05-31 subagent-drift note).
- [ ] **Change #4** (cross-check Phase-0 grep set vs rendered table): update `.claude/skills/harness-audit/SKILL.md` Phase 5 — 3× recurrence (multi-lane omission across audits).
- [ ] **Change #5** (descriptive sync commit subjects): edit `homeserver/scripts/sync-shared-skills.sh` line ~295 — 1× this session, directly caused the 3-day detection lag.
- [ ] **New lesson** `feedback_fix_regression_at_source_not_symptom.md` — "when a shipped change is silently reverted by tooling, the durable fix is source-side exclusion + a ledger-backed guard, not repeated local re-application." Captured in commit `f0b7cffcd` + `7f4710c31` LESSON trailers; promote to a standalone lesson if a 2nd source-side-revert incident recurs.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase section omitted —
no `/auto-phase` invocation or auto-state mutation this session (leftover
auto-state JSONs are from prior sessions; Step 0.5 trigger did not fire)._
