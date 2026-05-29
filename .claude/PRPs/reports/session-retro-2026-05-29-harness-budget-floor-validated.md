# Session retro — 2026-05-29 — harness-budget-floor-validated

**Harness:** claude-code (Opus 4.8 1M for the work session; Opus 4.6 for the closing `/context` validation)
**Session window:** 2026-05-29 (resume-from-handover) → 2026-05-29 (`/context` validation) (~spanned 2 harness sessions)
**Branch at start:** `7d0d9230a` (`governance-v0`)
**Branch at end:** `a6c55e41d` (`governance-v0`)
**Files touched:** 4 (decision-queue.md, multi-lane-worktree.md, refs/multi-lane-mechanics.md, the redesign handover) + MEMORY.md (user-scope, not repo-tracked)
**Commits:** 4 explicit (`d52fd5811` B3, `22418bad6` B5a, `8ea254a1a` handover close-out, `a6c55e41d` tokenizer correction); 0 auto

## TL;DR

Continued the harness context-budget redesign (B3 multi-lane relocation + MEMORY.md prune + B5a stale-trim) and investigated two follow-on levers via parallel subagents. The most load-bearing finding is twofold and self-correcting: (1) **the whole effort was sized against a wrong tokenizer ratio** — the predecessor report's `2.4 chars/token` over-counts; the real ratio measured against the harness's own `/context` per-file numbers is `~3.3 chars/token`, so the true always-load is **45k (22.5%)**, not the ~62k I reported mid-session; and (2) **profile-scoping (the hoped-for ~15-20k lever) is structurally impossible** in this harness — verified, not assumed. Top change: augment `feedback_context_trim_verify_empirically.md` with the chars/token correction so no future harness-audit re-propagates 2.4. The redesign is at the defensible floor; the only thing that shrinks the corpus further is v1 closing (retires the orchestration apparatus).

---

## What surprised us

- **The freeze map went stale *during* the effort that consumed it.** The redesign report §2 listed multi-lane `§"Lane modes"` and `§"Brief location and trunk→phase sync"` as relocatable (0 citations). When I ran `verify-rule-anchors.sh` before B3, both were FROZEN — because **B1 (earlier this same redesign) had added Claude-side citations to them** in `refs/auto-phase.md:586,588`. The two largest movable chunks became un-movable mid-stream. The guard caught it; if I'd trusted the report I'd have created 2 new dangling citations. Sharp reminder that a freeze map is a snapshot, and any multi-step relocation can invalidate its own map.
- **My own token math was off by ~1.4× the entire time.** I reported "71.5k → 62.2k" using the predecessor's `2.4 chars/tok`. The fresh-session `/context` (Opus 4.6) reported Memory files at **45k**, not 62k. Measuring the harness's per-file numbers against char counts gave the real ratio: **~3.3 chars/tok** (3.29 / 3.40 / 3.35 / 3.23 / 3.43 across the five biggest files — tight cluster, not noise). Every "≈ Δ tok @2.4×" figure I'd written was inflated. The *char* deltas were always right; only the token translation was wrong.
- **The big second lever didn't exist.** "Profile-scope the rules so impl/BM sessions skip advisor-orchestrator.md" sounded like the ~15-20k win. It's structurally impossible: the only scope primitive is `paths:`-on-Read, which can't fire on a session-role condition; no role-based loading config exists; and `branch-manager.md` is Pi-pinned resident-by-inheritance. The subagent investigation turned a plausible-big-lever into a verified dead end in one pass.
- **MEMORY.md was already mostly one-liners** — the handover's premise ("move CLOSED-lane detail to topic files") only partly applied. The real byte weight was (a) the 5-line Historical archive ledger (~3k chars of 2026-05-22 provenance) and (b) a handful of 350-835-char ACTIVE entries whose detail duplicated their linked workflow_state files. The biggest single entry (v1-quality-r2) was 835 chars — 6× the ~200 budget.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Augment `.claude/lessons/feedback_context_trim_verify_empirically.md` with the chars/token correction: **the `2.4 chars/tok` figure over-counts; real Opus-4.6 ratio is ~3.3; never propagate 2.4 into a harness-audit — read `/context` directly or measure per-file.** | Future harness-budget work reports honest token numbers; no inflated "we're at 62k" when the harness says 45k | minor | **2× (1 here + the lesson's own existing "documented≠observed" pattern, which this is a fresh instance of)** |
| 2 | Update `.claude/skills/harness-audit/` (helpers/scoring-matrix or report-template) to **read the harness's own `/context` per-file tokens as ground truth, and if estimating from chars, use ~3.3 not 2.4.** | The audit skill stops emitting 1.4×-inflated projections that mislead the budget decision | minor | 1× here (the redesign report that seeded 2.4 is the upstream artifact) — propose, don't force |
| 3 | Add to the redesign report / any freeze-map doc a standing note: **"a multi-step relocation can invalidate its own freeze map — run `verify-rule-anchors.sh` before EACH step, never once at the start."** (Already practiced this session; codify it.) | Prevents a future session trusting a stale freeze map and silently breaking Pi/Claude citations | minor | 1× here (but the guard already enforces it mechanically — this is doc-belt-and-suspenders) |
| 4 | When the next session is tempted to chase the `/context` "save ~13.5k" suggestion on the big rule files, **read the "DO NOT re-chase" block in `.claude/PRPs/handovers/harness-context-budget-redesign-2026-05-29.md` first** — profile-scoping is proven impossible; the remaining mass is the verified floor. | Saves a future session from re-deriving the B5b dead end (~3 hrs of subagent + investigation this session) | free (already written) | 1× here |

## What to carry forward

- **The guard (`verify-rule-anchors.sh`) is ground truth over any freeze-map document.** Run it before each relocation step and read the MISSING list against the known-benign baseline (3 dangles). It caught the stale-freeze-map this session before any damage. Used 3× cleanly (B3, B5a, sanity).
- **Parallel read-only subagents are the right tool for "is this lever real?" investigations.** Two `Agent` dispatches (B5a admissibility audit + B5b feasibility) ran concurrently, returned evidence-backed reports, and I verified their load-bearing claims against source before acting. Killed a mirage and confirmed a safe cut in one round. Brief them like colleagues (full context, what's ruled out) per advisor-orchestrator §6.3, and trust-but-verify the conclusions.
- **`memory-prune` skill's surface-then-confirm + bytes-bind-first discipline worked.** Measured both dimensions, led with the byte overage, got user confirmation via AskUserQuestion, applied, verified under budget. Clean.
- **When the user reframes the goal mid-session ("remove v0 / trim v1"), ground the answer in what the terms actually mean before acting.** "v0" turned out to be mostly the live base (trunk branch name, binding ADRs, shipped governance-log kinds) — surfacing that distinction prevented a destructive misread.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `verify-rule-anchors.sh` guard (run 3×) | 45 | 0 | high | caught the stale freeze map before B3 broke 2 citations; the single highest-leverage tool this session |
| B5a admissibility-audit subagent (Agent) | 30 | 0 | low | returned a verified per-cut table with grep evidence; the line-143 dangle catch was exactly right; I re-verified and it held |
| B5b feasibility subagent (Agent) | 90 | 0 | medium | turned "maybe a 15-20k lever" into "structurally impossible, here's the proof" — saved chasing a dead end; claims verified against source |
| `memory-prune` skill | 15 | 0 | low | bytes-first discipline + surface-then-confirm; landed under budget first pass |
| AskUserQuestion (×3) | 10 | 0 | none | clean forks: prune plan, direction (both-sequenced), close-out |
| `/context` fresh-session read (Opus 4.6) | 20 | 0 | high | the validation that surfaced the 2.4-vs-3.3 tokenizer error — without it I'd have left an inflated number in the record |
| 2.4-chars/tok estimate (predecessor-inherited) | — | 25 | high | every mid-session token figure I reported was wrong by ~1.4×; re-derivation + handover correction was the cost |

## Complexity scores (heavy tasks only)

No Junior impl-tasks ran this session (pure laptop-side advisor harness-infra work). The complexity metric (`files/commits/runtime/log-silence`) is designed for Junior worker tasks and doesn't apply to interactive advisor edits. Closest heavy item for reference:

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| B3 + MEMORY + B5a + 2 subagent investigations + close-out | 5 | 4 | ~spanned 2 sessions | N/A (interactive, no Junior log) |

Not a watchdog-envelope concern — interactive, no worker stalls.

## Decisions to revisit

- The predecessor `harness-redesign-session-profiles-2026-05-29.md` report carries the stale 2.4 ratio AND the now-disproven profile-scoping estimate (~15-22k). It's a scratch report, not a rule — but if any future session cites it, those two numbers will mislead. Worth a one-line "SUPERSEDED: see handover §DO-NOT-re-chase + tokenizer correction" banner at its top. (Low priority; the handover already carries the corrections.)
- The `harness-audit` skill itself was not run this session, but it's the artifact that would re-propagate 2.4 if invoked. Change #2 targets it. Verify the skill's actual estimation method when it's next touched.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1 (chars/token correction): **augment** existing `.claude/lessons/feedback_context_trim_verify_empirically.md` (do NOT create a new file — this is the same documented≠observed pattern the lesson already owns; it already has a 2026-05-29 MEMORY.md-bytes addition, this is the chars/token sibling). Then re-sync + backfill PMD.
- [ ] Change #2 (harness-audit skill uses /context ground-truth, 3.3 not 2.4): update `.claude/skills/harness-audit/` when next touched.
- [ ] Change #3 (run guard before each relocation step): one-line note in the redesign report or `verify-rule-anchors.sh` header.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase reliability section omitted —
no `/auto-phase` invocation or auto-state mutation this session (leftover
`v1-RT-r3.json` does not trigger per Step 0.5 revised rule)._
