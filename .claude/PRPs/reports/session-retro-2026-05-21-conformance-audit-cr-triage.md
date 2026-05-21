# Session retro — 2026-05-21 — conformance-audit-cr-triage

**Harness:** claude-code
**Session window:** ~2026-05-21T11:30Z → 2026-05-21T18:15Z (~6h 45 min wall-clock, ~3h active engagement)
**Branch at start:** `055495923` (governance-v0)
**Branch at end:** `e42991879` (phase-brehon-conformance-audit, fix-impl-4 brief committed)
**Files touched:** 6 (canonical: 4 briefs + 1 handover + 1 PMD scratch; lane: 3 findings/comment artifacts)
**Commits:** 9 (advisor-authored on gov-v0: 4; advisor on phase: 3; BM Junior on phase: 2)

## TL;DR

Drove brehon-conformance-audit PR #141 through bm-pr → bm-poll-cr → bm-triage → fix-impl dispatch in one session. Two BM Junior workers self-reported success while producing incorrect/missing artifacts: #392 (bm-poll-cr) pushed to wrong branch (zero refs reached origin; recovered via cherry-pick of unreachable commit `dd01153b8`); #394 (bm-triage) never read its task brief and left every finding at the poll-cr placeholder bucket. The advisor post-condition checks caught both before they cascaded. **The highest-leverage finding:** the BM brief's `base_branch` parameter controls where the BM worker pushes its `chore(bm):` commit; setting it to `governance-v0` (correct per advisor-orchestrator.md §2.1 for BM verb briefs *committed* to trunk) is wrong for verbs whose *output* needs to land on the phase branch (poll-cr/triage/merge — all of these mutate `pr-N-findings.yaml` which lives on the phase branch). Propose a brief-shape clarification + verb-specific guidance.

---

## What surprised us

- **Two BM Junior false-successes in one session.** Per `feedback_bm_false_success_advisor_post_condition_catch.md` (single prior occurrence on v1-ship-1-r2 #322), this was a known one-off class. Two more in one session = **3 confirmed occurrences across the corpus** → promotion threshold met. Both #392 + #394 returned `done` status, retro evals at 0.78-0.82, and produced artifacts that fooled `mcp__junior-brehon__show_task` but failed real post-condition checks (origin push, findings YAML correctness). The BM model is Haiku 4.5 (per `feedback_brehon_subagent_model_effort_assignments.md`) — the cheap model + slug-merging of brief path into the worker branch name (truncation hides the actual brief path) is the read-miss vector for #394.
- **The bm-poll-cr brief's `base_branch: governance-v0` produced a "no-op push" silent failure.** Worker forked off governance-v0 (correct per the §2.1 BM-from-trunk rule) but the verb's Phase 6 script then pushed to `phase-brehon-conformance-audit` — a branch the worker never tracked. `git push origin phase-...` returned `Everything up-to-date`, the daemon finalize-merged the worker branch into local governance-v0 (creating merge `19b3541b5`), but daemon's governance-v0 never reached origin because origin's governance-v0 hadn't actually advanced. Net: 23-finding YAML existed only as a daemon-local unreachable commit (recoverable for ~14 days until next `git gc`). This is a brief-design defect, not a Junior bug.
- **CR posted 22 inline comments ~30 seconds after PR open.** Faster than any prior PR in the corpus. Suggested fast-mode pricing is in effect (post-Pro-trial repo private flip). Wakeup scheduled for 12 min was conservative — could have polled at 2 min and dispatched poll-cr immediately. Not load-bearing but worth noting cadence drift.
- **Advisor-direct triage was the recovery path that worked.** When the user asked how to recover from bm-triage #394's false-success, the AskUserQuestion options included "re-dispatch with hardened brief" — but the user picked "advisor-direct" (Recommended). Took ~20 min inline (8 CR comment reads via `gh api`, write a triage Python script, write a 4-bucket digest comment, write a fix-impl brief). Likely faster than dispatching another Junior cycle with the same Haiku read-miss risk.
- **The 8 majors split 7 fix-in-pr / 1 carry-forward cleanly.** No `rebut`. No CR-misread. This was a higher-quality CR pass than recent precedents (PR #138 had 1 critical + 1 major in 29 findings; PR #141 has 0 critical + 8 majors all real). The skill bundle (markdown + bash + Python + JSON schema) is the type of artifact CR reads well — small files, dense semantic content, clear contracts.
- **CR Pro features still active post-flip.** Repo went private 2026-04-25; Pro trial extended through 2026-05-09 per `reference_private_repo_constraints.md`. Today's auto-review depth (full schema-validity check, formula re-derivation, regex test) suggests the trial may have auto-renewed or the public-repo tier is now sufficient. Worth confirming next session.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add `base_branch:` guidance section to `.claude/PRPs/templates/bm-task-brief.template.md` (or create one if absent): "For bm-pr / bm-cut / bm-triage briefs — base_branch=`governance-v0` (worker forks from trunk, writes brief output to trunk paths). For bm-poll-cr / bm-merge / fix-impl impl-task briefs — base_branch=`phase-<phase>` (worker forks from phase branch, mutates files that LIVE on the phase branch like `pr-N-findings.yaml`)." | Eliminates the "no-op push" failure mode that cost #392 a cherry-pick recovery. | minor (~15-min edit) | 1× this session; new failure mode (not in PMD) |
| 2 | Promote `feedback_bm_false_success_advisor_post_condition_catch.md` from "single prior occurrence" to confirmed pattern — append the #392 + #394 evidence + extend the "auto-catch-fire" signal list: (a) BM Junior reports `done` but `git log` on the target branch shows zero new commits matching `chore(bm):` (b) findings YAML on phase branch unchanged or in placeholder shape (c) BM worker permission denials >0 in task log. | Future BM Junior cycles auto-detect false-success on the 3 mechanical signals; advisor stops trusting the `done` status alone. | minor | 3× total (v1-ship-1-r2 #322, this session #392, this session #394) — meets `feedback_principles_not_rules` 3+ canonical-promotion threshold |
| 3 | Add a Junior brief-read enforcement preamble to `.claude/PRPs/templates/bm-task-brief.template.md` (and impl-task-brief.template.md) §1: "READ THIS BRIEF FIRST. The first Read tool call in this task MUST be on the brief file at the path in your dispatch line. Failure to read the brief and act on its §2 Scope/§4 Constraints is a process breach surfaced at retro time per feedback_bm_false_success_advisor_post_condition_catch.md." | Cheap guard for the slug-truncation-hides-brief-path failure mode. Already added in fix-impl-4 brief constraint #11 as a one-off — promote to template so every future brief inherits it. | minor | 1× this session (Junior #394 + likely #392 root cause); the slug-truncation pattern is structural — every brief is at risk |
| 4 | Surface auto-mode classifier behaviour in `feedback_brehon_autonomy_goals.md` or a new lesson `feedback_auto_mode_classifier_blocks_visible_outbound.md`. When auto-mode is active, even pre-approved (via AskUserQuestion) `gh pr comment` / `gh issue create` calls are blocked pending per-action confirmation. Workflow: surface to user with `AskUserQuestion` listing the specific commands; user re-confirms; advisor proceeds. | Eliminates the friction of getting denied on a user-pre-approved action; user knows in advance that auto-mode adds a confirmation re-prompt for visible outbound. | minor | 1× this session; will repeat on every CR-triage user-gate-3 cycle under auto-mode |
| 5 | Update `.claude/commands/bm/bm-poll-cr.md` Phase 6 (push step) to add a post-push sanity check: `git log origin/<branch> -1 --grep "poll-cr #<PR#>"` should show the new commit OR exit 1 with diagnostic. Currently the verb assumes push success without verifying. | The "no-op push" failure mode would have been caught by the worker itself, not via 30-min advisor forensics. | medium (verb script edit + Junior subagent self-verification logic) | 1× this session; affects all 6 BM verbs that push (poll-cr, pr, triage, merge, prp-review, ping) |

## What to carry forward

- **Advisor-direct triage when the BM cycle is unreliable.** When two Junior cycles fail self-policing on the same PR, fall back to inline triage. Costs ~20-25 min but avoids the 3+ Junior cycles + verification rounds that the false-success class can produce.
- **Cherry-pick from unreachable commits as a recovery vector.** When daemon finalize-merge produced commits that exist in the object DB but didn't reach origin (typical when push targets the wrong branch), `git cherry-pick <unreachable-sha>` + push from the correct branch recovers the work cleanly. Used once this session; pattern is durable.
- **Triage Python script as one-shot tool.** Inline `python3 << EOF` heredocs collide with quote-heavy content; writing a `.triage-N.py` scratch file under `.claude/PRPs/reviews/.triage-<PR#>.py`, executing once, then deleting via `rm` is cleaner and re-runnable. Used this session for the 23-finding bulk-bucket.
- **CR's own diff suggestions are the contract for non-rebut findings.** Don't paraphrase. The fix-impl brief §2.1 table cites CR's verbatim before/after; the impl-task worker reads the CR comment via `gh api` if `notes` is truncated. This discipline saved ~10 min of "what did CR actually mean" interpretation cycles on cr-16 (schema patch), cr-19 (FP attribution), cr-21 (regex).
- **Read the live state before acting on stale wakeups.** Three stale wakeups fired this session (re-poll #387, re-poll #392, re-dispatch). Each was correctly identified via TaskList + `mcp__junior-brehon__show_task` + `git log` and ignored. Per `feedback_thin_wakeup_prompts_verify_live_state.md`, the verification cost is ~30 sec; the prevented cost is a duplicate Junior dispatch.
- **Brief shape: §6 "Commit subject" with §G4-style verbatim contract.** The fix-impl-4 brief's §2.1 table cited CR's diffs row-by-row + §6 stated three commit subjects in order. This is the same shape as `feedback_clippy_per_module_deny_requires_workspace_allow.md` (Worked example) but applied forward to fix-impl dispatch. Worth promoting to impl-task-brief template.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Junior #390 (bm-pr) | 8 | 0 | none | Clean ~1m35s runtime; opened PR #141 cleanly |
| Junior #392 (bm-poll-cr) | 0 | 25 | high | False-success; wrong-branch push; required cherry-pick recovery |
| Junior #394 (bm-triage) | 0 | 30 | high | False-success; never read brief; advisor-direct re-triage required |
| Advisor-direct triage (inline) | 25 | 0 | low | Replaced Junior #394 cleanly; 23 findings bucketed in ~20 min |
| Junior #395 (fix-impl-4) | TBD | TBD | TBD | Still running at retro author time (~9 min in); cr-21 confirmed applied; brief was read |
| `gh api repos/.../pulls/comments/<id>` | 15 | 0 | none | Direct CR-comment-body fetch; 8 majors read in parallel batches |
| AskUserQuestion (User Gate 3 + classifier override) | 5 | 0 | none | Clean architectural forks: triage recovery option + classifier confirmation |
| Cherry-pick from unreachable commit | 10 | 0 | medium | Junior #392 commits `dd01153b8` recovered as `b31040129` on phase branch cleanly |
| `general-purpose` subagent (task-log forensics) | 18 | 0 | none | 234KB + 342KB task logs analysed without main-context bloat; surfaced root causes verbatim |
| ScheduleWakeup (4 used) | 5 | 0 | low | Cadence calibrated for 5-15 min Junior cycles; 1 fired during active engagement (no work wasted) |
| Stale-wakeup verification | 2 | 0 | none | Three stale fires ignored per `feedback_thin_wakeup_prompts_verify_live_state.md` |
| Atomic protocol (canonical DQ write) | 3 | 0 | none | Multi-lane-worktree.md hard refusal #6 followed cleanly across 4 governance-v0 brief commits |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Junior #390 (bm-pr) | 1 (PR body via gh API) | 0 | 1.5 | <1 |
| Junior #392 (bm-poll-cr — false success) | 2 (pr-141-findings.yaml + bm-runlog.md) | 2 | 2.5 | <1 |
| Junior #394 (bm-triage — false success) | 1 (pr-141-findings.yaml stripped) | 2 | 2.8 | <1 |
| Advisor-direct triage | 3 (findings.yaml + comment.md + cr1-issue-body) | 1 | 22 | n/a (inline) |
| Junior #395 (fix-impl-4 — still running) | TBD (~7 files projected) | TBD (3 commits projected) | TBD | TBD |

**No watchdog risk this session.** The Junior failures were correctness, not envelope. Junior #394 ran 2m47s — well under any silence threshold; the issue was content, not duration.

## Decisions to revisit

- **DQ schema-v3 (cr-1 → issue #142)**: deferred carry-forward. When does the next planning sub-phase get authorised? The retro suggested a `v1-dq-schema-r1` follow-up but no commitment was made.
- **Auto-mode classifier policy**: should pre-approved (via AskUserQuestion) actions be exempt from per-action confirmation, or is the double-gate the right defence? Per `evaluation-calibration.md` "reliability > speed", probably keep the double-gate but document it as expected behaviour.
- **bm-poll-cr's "force-add + commit + push" Phase 6 step on a worker forked off governance-v0**: should the verb script detect base-branch mismatch with target-branch (where it intends to push) and refuse with a hard-error? Brief is one way; verb-script self-defence is another.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1 (base_branch guidance): promote to `.claude/PRPs/templates/bm-task-brief.template.md` (or create if absent) + cross-link in advisor-orchestrator.md §2.1
- [ ] Change #2 (BM false-success post-condition): extend `.claude/lessons/feedback_bm_false_success_advisor_post_condition_catch.md` (existing file) with 3-occurrence evidence + 3-signal auto-detect list
- [ ] Change #3 (brief-read enforcement preamble): promote to both `.claude/PRPs/templates/bm-task-brief.template.md` + `.claude/PRPs/templates/impl-task-brief.template.md`
- [ ] Change #4 (auto-mode classifier): write new `.claude/lessons/feedback_auto_mode_classifier_blocks_visible_outbound.md`
- [ ] Change #5 (bm-poll-cr push-verify): update `.claude/commands/bm/bm-poll-cr.md` Phase 6 (medium-cost) — defer unless #1 + #2 + #3 don't catch the failure mode in the next 2 sub-phases

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. /auto-phase artifacts not active this session — 10-category section skipped per Step 0.5 trigger._
