# Session retro — 2026-05-26 — pr-155-cr-triage-junior-479-fail

**Harness:** claude-code
**Session window:** 2026-05-26T17:26Z → ~19:10Z (~105 min wall-clock)
**Branch at start:** `87285bbfe` (`phase-v1-RT-r3`)
**Branch at end:** `065d9800e` (`phase-v1-RT-r3`)
**Files touched:** 5 (1 brief, 1 handover, 2 review artifacts gitignored, 0 crates)
**Commits:** 2 (auto: 0, explicit: 2 — `040730def` brief + `065d9800e` handover) + 1 by Junior bm-task #478 (mid-session, opened PR #155 from gov-side worktree)

## TL;DR

CR + Copilot ingest on PR #155 produced 12 findings; four-bucket triage filed cleanly + user-gate 3 approved. fix-impl-2 dispatch to Junior #479 failed `error_max_turns` after 38 min / $8.30 / 151 turns — **identical failure class** to v1-RT-r3 Task 4 cycle (#474–#477) that triggered the §5.3 cycle-count HARD REFUSAL + DQ -033 advisor-authorship carve-out. Worker found the correct JSON Edit recipe by its final turn (verified in `permission_denials`) but ran out of turns mid-application. The defect is brief-design, not recipe-correctness: a 246-line brief covering 3 file edits with helper-fn anchors NOT pre-located is too dense for Sonnet impl-task on a 17,000-line target. Top change: at brief-author time, **pre-locate exact `old_string` anchors for every e2e.rs Edit** and cap fix-impl briefs touching e2e.rs at ≤2 edits / ≤150 lines.

---

## What surprised us

- **Junior #479 had the right answer by turn 150** — the `permission_denials` array contained the EXACT correct JSON Edit `old_string`/`new_string` matching brief §2.2 verbatim. I had assumed `error_max_turns` meant the worker drifted off-brief; the log showed it was finding the correct answer but had burned 150 turns confirming context first. Re-read of the failure-class root cause: brief verbosity × file-fragility × Sonnet context-cost-per-Read.
- **CR's 🔴 Critical claim falsified incorrectly on first pass** — `python json.load` + `jq -e '[.resolved[].id] | unique | length'` returned valid + 207 unique IDs. I almost bucketed cr-1 as `rebut`. The duplicate-key collapse is silent under permissive parsers; only structural re-read of the lines 4085-4106 surfaced the indent + missing `},{` boundary. The falsification was incomplete because I used the wrong tooling (permissive parsers can't detect dup-key collapse).
- **Worker tried Linux `.sh` wrapper while brief specified Windows `.bat`** — `permission_denials` showed 4× `bash scripts/brehon/cargo-check.sh --workspace --features full` attempts blocked by Junior daemon's allowlist. The brief's `pre-phase-harness-audit.md` "OS-aware wrapper invocation" section gives the worker the right form, but the daemon's command allowlist didn't include `.sh` for that path. Net: worker correctly OS-adapted but daemon rejected.
- **`gh pr view --comments` exceeded read-tool token cap** — 44 KB JSON blob blocked Read. Had to re-fetch via `gh api /pulls/N/comments --paginate` + `jq`. Should default to `gh api`-based ingest from the start; the `view --comments` form is fine for human reading but bloats LLM context.
- **One Junior bm-task succeeded cleanly** — task #478 (bm-pr with inline body, lane-mode workaround per option ii) opened PR #155 cleanly + appended runlog. Mechanical bm-pr verbs are still reliable; the failure pattern is impl-task scope-specific.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | At brief-author time for any fix-impl touching `crates/server/tests/e2e.rs`, the advisor MUST pre-locate verbatim `old_string` anchors and paste them into the brief §2.2/§2.3 recipe. NEVER leave "use distinctive anchors per <lesson>" as guidance — the worker can't afford the Read budget. Add to `.claude/rules/advisor-orchestrator.md` §2.4 file-class lesson injection table as a hard precondition. | -100% recurrence of context-cost-driven `error_max_turns` on e2e.rs Edit tasks. Saves $8+ per dispatch + 38 min runtime. | minor (advisor pre-reads same lines either way) | 1× this session (#479) + 1× prior class (Task 4 cycle #474-#477, DQ -033) = **2× threshold met** |
| 2 | Cap fix-impl briefs at ≤150 lines AND ≤2 file edits AND ≤2 Edit calls per file when target is e2e.rs. Briefs exceeding this must split into 2 narrower briefs (e.g. fix-impl-2a + fix-impl-2b). Codify in `.claude/PRPs/templates/impl-task-brief.template.md` as a §2 Scope gate. | Forces brief decomposition before dispatch; each piece fits Sonnet's reliable envelope. Saves dispatch-retry cost; multiplies parallelism in cohort-eligible cases. | minor (template gate + advisor discipline) | 2× this session (the original fix-impl-2 + Task 4 carve-out) |
| 3 | Surface failure-class risk at fix-impl dispatch user-gate. When brief targets a known-fragile pattern (e2e.rs ≥2 edits, JSON syntax fix on a 4000-line DQ, multi-helper refactor), the AskUserQuestion at user-gate-3 MUST include "this brief matches the X failure class signature — N% retry rate observed" as one of the option descriptions. Add to `.claude/rules/advisor-orchestrator.md` §3.2 gate 3. | User authorises with full risk picture; advisor doesn't silently dispatch into known traps. | minor (one extra sentence in the gate prompt) | 1× this session — single recurrence; record as note, watch for 2× before promoting |
| 4 | Default CR triage ingest to `gh api /pulls/N/comments --paginate | jq` + `gh api /pulls/N/reviews | jq` instead of `gh pr view --comments`. Update `.claude/commands/bm/bm-poll-cr.md` Phase 1 to specify `gh api` as the primary fetch. | Avoids 44 KB JSON blob blowing the Read tool token cap; cuts ~5 min of re-fetching per CR poll. | minor (single-line update to a slash command) | 1× this session; not a new pattern (the token-cap behaviour was known) but never documented in the verb |
| 5 | Add a duplicate-key check to the JSON-validation recipe in `.claude/rules/decision-queue.md` "Verification" section. The permissive-parser trap (Python json.load + jq silently allow dup keys) means our standard validation lies on this defect class. Use `python -c "import json; json.loads(text, object_pairs_hook=lambda kvs: dict(kvs) if len(set(k for k,_ in kvs)) == len(kvs) else (_ for _ in ()).throw(ValueError('duplicate keys')))"`. | Catches dup-key collapse defects (like cr-1) at write-time, not at next CR review. | minor (one-line recipe in the rule body) | 1× this session — single recurrence; record + watch |
| 6 | When advisor falsifies a CR claim, the falsification tooling MUST match the claim's failure mode. CR's "strict parsers would reject" means the falsification needs strict-mode tooling (dup-key check, ordered-key check), not permissive `json.load`. Add to `.claude/rules/decision-queue.md` "Falsifiable-hypothesis gate (CR-finding variant)". | Avoids the "I checked it parses, therefore CR is wrong" pattern — that's not falsification, it's confirmation bias against CR. | minor (rule update) | 1× this session + 1× prior (`feedback_falsifiable_hypothesis_before_structural_fix.md` covers structural fixes; the CR-finding variant is adjacent but distinct) = **borderline 2×** |

## What to carry forward

- **Pre-compact handover discipline holds.** Authored + committed the handover BEFORE session-end signal without prompting; followed `advisor-orchestrator.md` §1 verbatim. The RESUME block is self-contained; next session can pick up cold in <5 tool calls. This is the second consecutive session with clean handover discipline.
- **Read worker's FINAL tool call before triaging Junior failure.** The `permission_denials` block in the task log told the WHOLE story — recipe correctness, OS adaptation, context exhaustion sequence — in 5 entries. Cost: 1 Bash + 1 Grep tool call. Reflexively read it on every failed Junior dispatch from now on.
- **Falsifiable-hypothesis gate fires on CR findings too** — when CR claims a defect, the falsification step is part of triage. Caught + corrected mid-session for cr-1; the recovery was inside ≤2 tool calls. Discipline is in place; just need to extend the tooling per §"What to change" #5/#6.
- **User-gate consolidation** — single AskUserQuestion call carried 3 paths (a/b/c) with clear option descriptions; user chose without re-clarify. Pattern: when surfacing a triage decision with N viable paths, batch them into one AskUserQuestion rather than asking sequential approve/deny.
- **Lane Mode A is still the right call for active phase-driven sessions** — no trunk→phase sync overhead; brief + handover both committed directly on phase branch. The canonical `brehon-fork` checkout was idle this session; no concurrent-session contention observed.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `gh pr view --comments` (initial CR fetch) | 0 | 5 | medium | 44 KB blob hit Read token cap; had to re-fetch via `gh api` — see "What to change" #4 |
| `gh api /pulls/N/comments --paginate \| jq` (re-fetch) | 5 | 0 | none | clean structured ingest of CR + Copilot findings |
| Four-bucket triage drafting (advisor inline) | 25 | 0 | none | bucketed 12 findings into pr-155-findings.yaml + pr-155-comment.md; user-gate 3 approved on first ask |
| Falsifiable-hypothesis check on cr-1 | 3 | 5 | high | permissive parsers (`json.load` + `jq -e`) returned PASS; structural Read confirmed defect — falsification was incomplete first pass — see "What to change" #5/#6 |
| AskUserQuestion (user-gate 3, 3-option triage) | 2 | 0 | none | option (a) chosen cleanly; no re-clarify needed |
| Brief authoring (fix-impl-2, 246 lines) | 20 | 0 | low | mirrored fix-impl-1 template; cited findings YAML + canonical recipe verbatim |
| Junior bm-task #478 (bm-pr inline body) | 15 | 0 | none | PR #155 opened cleanly; runlog appended; mechanical verb |
| Junior impl-task #479 (fix-impl-2) | 0 | 38 | high | `error_max_turns` after 151 turns; worker had right recipe by final turn; cost $8.30 — see "What to change" #1/#2/#3 |
| `mcp__junior-brehon__task_logs` (#479 triage) | 5 | 2 | medium | 542 KB log triggered file-pointer fallback; grep for stop_reason + permission_denials was the right cut |
| Handover authoring (130 lines, 3 paths) | 10 | 0 | none | self-contained; RESUME block + 3 retry paths recommended next session |
| AskUserQuestion (3 retry paths, interrupted) | 0 | 0 | none | user interrupted to close session before answering; paths preserved in handover |

**Net wasted vs saved:** ~50 wasted / ~85 saved = +35 min net positive. Junior #479's $8 + 38 min runtime is the dominant cost; almost all of it traces to one defect class (brief verbosity × file fragility) addressed by changes #1+#2.

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Junior bm-task #478 (bm-pr inline) | 2 (PR body composed + runlog appended) | 1 (`f0c2....` on `governance-v0`) | ~15 | ~3 |
| Junior impl-task #479 (fix-impl-2) | 0 (no commit) | 0 | 38 (FAIL `error_max_turns`) | ~5 (worker stayed active across 151 turns; no idle gaps until kill) |

**Flag:** #479 hit the failure envelope (runtime 38 min < 55 min threshold, but model-output-throughput exhausted before commit). This is **not** a watchdog kill — it's a tool-call budget exhaustion. The complexity metric's `max-log-silence-min` doesn't capture this failure mode well; the metric was designed for stdout-silent watchdog kills. Candidate: extend the metric to include `turn-count` for Sonnet impl-tasks.

## Decisions to revisit

- **§5.3 cycle-count meta-rule scope** — currently triggers HARD REFUSAL after 3 cycles same `(error_class, file_basename)`. Junior #479 was cycle 1 for fix-impl-2 but failure class IS identical to Task 4's 4-cycle catchfire (#474–#477). Should the meta-rule consider failure-class signature (`error_max_turns` on `e2e.rs`) across SUB-PHASE-TASKS, not just within a single task's cycle history? Worth a clarify pass next session — if pattern fires once more, this becomes a definite rule change.
- **fix-impl-2 retry path** — user interrupted before answering the 3-option AskUserQuestion. Path C (advisor JSON only + cr-3 → carry-forward) is recommended as the lowest-scope clean ship; Path B (advisor carve-out for both) extends the DQ -033 carve-out scope. Next session surfaces options + executes user choice.
- **Complexity-metric extension** — add `turn-count` (or `tool-call-count`) to the formula for Sonnet impl-task heavy tasks. The current `<files>/<commits>/<runtime>/<silence>` shape misses tool-call-budget failures.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1 (pre-locate verbatim anchors in e2e.rs fix-impl briefs): promote to `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` (cross-harness lesson) — **recurrence 2× threshold met** (#479 + Task 4 cycle).
- [ ] Change #2 (cap fix-impl brief length + edit count on e2e.rs): update `.claude/PRPs/templates/impl-task-brief.template.md` §2 Scope gate — recurrence 2× threshold met (same cycle).
- [ ] Change #3 (surface failure-class risk at user-gate 3): update `.claude/rules/advisor-orchestrator.md` §3.2 gate 3 — recurrence 1× this session, watch for 2× before promoting.
- [ ] Change #4 (default to `gh api` for CR ingest): update `.claude/commands/bm/bm-poll-cr.md` Phase 1 — recurrence 1× this session, low-cost so worth applying eagerly.
- [ ] Change #5 (duplicate-key JSON validation recipe): update `.claude/rules/decision-queue.md` Verification — recurrence 1× this session, watch for 2× before promoting.
- [ ] Change #6 (CR-finding falsifiable-hypothesis variant): update `.claude/rules/decision-queue.md` or extend `feedback_falsifiable_hypothesis_before_structural_fix.md` — borderline 2× recurrence (adjacent prior pattern).
- [ ] PMD eval write: ID 593 already written this session via Step 5 (session-end retro to PMD). Already covers the failure class.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._

---

## NEXT-SESSION ACTION — implement promotion candidates (user-authorised 2026-05-26)

User authorised at session close: **implement the 2 threshold-met
changes before any other work on PR #155**. The fix-impl-2 retry path
(per handover `.claude/PRPs/handovers/v1-RT-r3-fix-impl-2-recovery-2026-05-26.md`)
can wait — the brief shape that goes back to Junior MUST already
incorporate change #1 (verbatim anchors) and benefit from change #2
(length cap), so doing the lesson promotion FIRST is the correct order.

Execute in this order:

1. **Change #1 — promote to `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md`** (new lesson). Body: state the rule (advisor pre-locates verbatim `old_string` + `new_string` for every e2e.rs Edit in fix-impl briefs); cite Junior #479 + Task 4 cycle as the 2× recurrence evidence; cite `feedback_junior_worker_e2e_edit_hang.md` as the parent class lesson. Then cross-reference from `advisor-orchestrator.md` §2.4 file-class lesson injection table (add a new row: `crates/server/tests/e2e.rs` + fix-impl brief → also requires `feedback_fix_impl_pre_locate_e2e_anchors.md`).
2. **Change #2 — update `.claude/PRPs/templates/impl-task-brief.template.md`** §2 Scope gate. Add: "When `crates/server/tests/e2e.rs` appears in `modifies:` array AND brief is a fix-impl: brief length ≤150 lines, file edits ≤2, Edit calls per file ≤2. Briefs exceeding any cap MUST split (e.g. fix-impl-Na + fix-impl-Nb)." Also cite the new lesson from change #1.
3. **Commit both as `docs(lessons): promote fix-impl e2e.rs pre-located anchors lesson (2× recurrence) + template cap`** on `governance-v0` (lesson + template are trunk-direct per `phase-branch.md` "Direct on governance-v0" policy). Sync to PMD via `bash scripts/sync-lessons-to-pmd.sh --db <canonical> --strict` per the retro skill Step 5.5 recipe. Run the backfill per Step 5.5 since a new lesson was promoted.
4. **THEN return to fix-impl-2 retry per handover.** With change #1 in place, the next fix-impl-2 brief includes verbatim anchors; with change #2 in place, the cap forces split into 2a + 2b. The advisor's choice between Path A/B/C from the handover is unchanged by these promotions — but if Path A is chosen, the briefs are now properly shaped.

Estimated cost: ~25 min for both promotions (one lesson file authored
+ one template gate added + sync + backfill + commit + push). Saves
~$8 + 38 min per future e2e.rs fix-impl dispatch and prevents recurrence
of the Junior #479 failure class.

Changes #3-#6 (single-recurrence) stay in the watch list; promote when
they recur. User authorisation does NOT extend to those — only #1+#2.
