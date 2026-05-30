# Session retro — 2026-05-31 — type-state-lesson-apollo-assessment

**Harness:** claude-code  
**Session window:** ~2026-05-30 23:30 IST → 2026-05-31 01:00 IST (~90 min)  
**Branch at start:** `71c4a6f2c` (`governance-v0`)  
**Branch at end:** `4e87daaa7` (`governance-v0`)  
**Files touched:** 8 (1 new lesson, 1 rule amendment, 6 handler annotations)  
**Commits:** 1 explicit, 0 auto

## TL;DR

Session started as an evaluation of whether to install Apollo's `rust-best-practices`
skill (answer: no — direct conflicts with LemmyError architecture and redundant with
workspace-wide deny lints). Pivoted to authoring a Brehon-specific type-state lesson
and retrofit annotations for the 6 existing governance handler state guards. The most
load-bearing finding: **two Explore subagents in parallel produced the full codebase
intelligence needed in ~4 min** — no main-context blowout, no repeated reads. The top
change proposal is to wire the new lesson into the PMD via sync + backfill so it's
reachable via `memory_search_hybrid`.

---

## What surprised us

- **Apollo's skill targeted a completely different error architecture.** The `rust-best-practices`
  skill recommends `thiserror` and `anyhow` as defaults. Brehon's workspace explicitly denies
  both (no external error crates; `LemmyError` enum + `LemmyResult<T>` are the canonical shapes).
  The mismatch wasn't guessable from the skill's description — it required reading the actual
  workspace `Cargo.toml` lints section to surface. Without the codebase check, the skill could
  have been installed and conflicted silently with every new handler brief.

- **The PMD had zero pre-existing entries for "type-state" or "state machine enforcement"**
  despite this being an obvious future-proofing concern for an 11→13-variant enum growing with
  every v1 sub-phase. The gap is not surprising in retrospect (the pattern hadn't hurt yet), but
  it confirms the lesson authoring was genuinely additive rather than duplicating prior work.

- **Plan mode correctly triggered the clarify question** before finalising scope. The user's
  original wording ("retrofit existing handlers FOR THE planned refactoring already documented
  in PMD") pointed at a non-existent PMD entry. AskUserQuestion resolved this into the correct
  scope (lesson + TODO annotations, not a structural Rust change) in one round-trip. Clean.

- **Comment-only `.rs` edits still need exact-line verification.** Two of the six TODO
  annotations were targeted by approximate line numbers from the Explore subagent report.
  Reading the actual guard lines before editing (not trusting the subagent's reported numbers)
  was the right call — `admin_close_case.rs` guard started at line 65, not 68 as initially
  estimated. Without the read step, the comment could have landed inside a different block.

---

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Sync the new lesson to PMD and run backfill: `bash scripts/sync-lessons-to-pmd.sh --db "$CANON_PMD" --strict && OLLAMA_URL=http://homeserver:11434 PROJECT_MEMORY_DB="$CANON_PMD" PROJECT_ROOT=C:/Users/barri/Developer/brehon-fork node .../backfill.js --verbose` | `memory_search_hybrid(query: "governance handler state guard")` returns the lesson; future Junior briefs via §2.3 pre-queue search pick it up automatically | minor (~5 min) | 1× this session (new lesson written); standard post-lesson step |
| 2 | Add `feedback_governance_type_state_handlers.md` to MEMORY.md index under "Cargo / Rust (Lemmy-specific)" section | Lesson is visible at session-start auto-load, not just via hybrid search | minor (~2 min) | 1× this session (lesson is orphaned from the index) |
| 3 | When evaluating third-party skills/tools for a Rust project, make workspace `[lints.clippy]` + error-type inventory the **first** read — not the skill's feature list. Add a one-line gate to `/review` or a standing advisor note: "check lints + error architecture before any `npx skills add`" | Prevents a silent adopt-and-conflict cycle where a skill's guidance contradicts workspace-enforced lints | minor (30-sec reminder in next relevant session) | 1× this session; adjacent failure mode seen 1× prior (CR triage recommending `.get(0)` → `.first()` without checking Diesel LimitDsl) |

---

## What to carry forward

- **Parallel Explore subagents for codebase intelligence.** Two agents dispatched simultaneously
  (one for PMD/roadmap, one for handler inventory) returned a complete picture in ~4 min with
  zero main-context blowout. The handler inventory agent returned 33 files, 6 handler deep-reads,
  4 existing helper functions, and a state-transition table — all without polluting the main
  thread with raw file content. Pattern: use Explore for research phases, keep main context for
  synthesis and writing.

- **Exact-line verification before comment injection.** Subagent reports cite approximate line
  numbers. Always `Read` the actual target range before `Edit` — even for comment-only changes.
  The verification step is 3 tool calls and takes ~30 seconds; skipping it risks a misplaced
  annotation that someone has to hunt later.

- **TODO annotation format `// TODO(type-state): <specific retrofit guidance> — see <lesson.md>`**
  is the right shape. The colon-keyword makes it `grep`-able, the specific guidance (not just
  "fix this") tells the next reader what to do, and the lesson reference closes the loop. Verify
  with `grep -rn "TODO(type-state)"` post-commit as the acceptance criterion.

- **Plan mode + AskUserQuestion for scope ambiguity is faster than guessing.** The clarify
  round-trip cost ~2 min. Proceeding on the wrong premise (that the PMD had a pre-existing
  refactoring plan) would have cost 15+ min of planning work against a non-existent target.

- **Comment-only + lesson + rule-amendment commits belong on `governance-v0` direct** (no
  phase branch, no cargo validation). The `phase-branch.md` litmus test ("would CR review be
  net-signal?") correctly routes these — CR is mediocre on advisor prose, excellent on Rust.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Explore subagent × 2 (parallel) — Apollo skill research + PMD search | 25 | 0 | low | Clean return; actionable synthesis on first read; Apollo findings complete including conflict analysis |
| Explore subagent × 2 (parallel) — handler inventory + roadmap read | 30 | 0 | low | 33-handler inventory, 6 deep reads, state-transition table — no main-context cost |
| AskUserQuestion (scope clarify) | 13 | 0 | none | One round-trip resolved "PMD has no pre-existing plan" ambiguity into correct scope |
| Plan mode + ExitPlanMode | 5 | 0 | none | Correctly gated execution; plan file is clean record of approved scope |
| Direct Edit × 6 (handler annotations) | 10 | 3 | low | 3 min lost to double-checking line numbers from subagent reports vs actuals; mitigated by Read-before-Edit |
| Bash validation suite (grep + python structural check) | 5 | 0 | none | All three verification checks passed first attempt; python check useful for lesson format validation |

**Net:** ~88 min saved / 3 min wasted. Surprise count: 0 high, 0 medium, 1 low (Apollo conflict depth).

---

## Complexity scores (heavy tasks only)

This session had no Junior tasks and no impl-task dispatches. The single commit was
advisor-authored, comment-only.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Type-state lesson + 6 annotations + rule row | 8 | 1 | ~25 | ~5 (longest Read batch) |

Well within envelope. No watchdog risk. No carry-forward on bundling.

---

## Decisions to revisit

- The planned v1 handler retrofit (adding `GovernanceCase<S>`, `state.rs`, `TryFrom` impls)
  is not yet a roadmap entry. Should be added to `v1-roadmap.json` as a future sub-phase
  (`v1-quality-r4` or a standalone `v1-typestate-r1`) with a note that the TODO annotations
  + lesson are the pre-work. Worth adding at next roadmap review session.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Workspace lints + error-architecture check before any third-party skill/tool install**:
  promote to a one-liner in MEMORY.md "Advisor session discipline" section (recurrence basis:
  1× this session + 1× prior CR/Diesel `.first()` mismatch — threshold met)

- [ ] **PMD sync + backfill after every new lesson file**: this is already documented in
  `feedback_pmd_backfill_after_write.md` and the session-retro skill body (Step 5.5) but
  was NOT executed this session. No new lesson needed — just execute the step (see §"What
  to change" #1 above).

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase section omitted: session did not
invoke `/auto-phase` or mutate auto-state JSON (leftover v1-RT-r3 artifact)._
