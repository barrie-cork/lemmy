# Compact-prompt approach — design notes + optimisation log

**Status:** evolving. This is an optimisation log, NOT a confirmed lesson. It documents the
design rationale behind the custom `/compact` prompt so future sessions can keep tuning it
against real `/context` data.

**The artifact this describes:** `~/.claude/commands/compact-phase.md` (user-scope, NOT
repo-tracked — it lives in the user's `~/.claude/commands/` dir, manually pasted in when the
user runs `/compact <paste>`). Because it's user-scope it does not version with the repo; this
refs note is the tracked record of WHY it's shaped the way it is. If you edit the command file,
update this note in the same session.

---

## The core idea

Claude Code's default `/compact` produces a 9-section summary (Primary Request / Technical
Concepts / Files / Errors / Problem Solving / User messages / Pending / Current Work / Next
Step). That template is designed for a **cold resume with no other context**.

But a post-compact resume is NOT cold. The harness **auto-injects, in full, at session start**:

- `CLAUDE.md`
- **every** `.claude/rules/*.md` file (verbatim — ~45K tokens as of 2026-05-30)
- the `MEMORY.md` index
- re-injected skill bodies for any skill that ran earlier in the session
- the deferred-tools list, MCP server instructions, current date

So the summary runs **on top of** the entire standing-facts corpus. Any summary section that
restates the four-role model, governance/branch/PR conventions, PMD/DQ/backfill mechanics,
security constraints, canonical paths, or ADRs is **pure duplication** — the harness already
re-injected it. The default template can't know this, so its "Technical Concepts" and
constraint sections are structurally guaranteed to double-pay.

**The custom prompt's job is to suppress that duplication** and spend the summary budget only on
what the auto-load CANNOT reconstruct.

## What the auto-load CANNOT reconstruct (= what the summary MUST carry)

1. **The most recent active thread** — branch, last commits (SHA + one-liner), stage, in-flight
   state, and any *session-discovered facts that exist in no rule file*. For `/auto-phase`,
   priority 1 starts from the durable ledger: cite `.claude/auto-state/<phase>.json`
   `stage_digests[-1]` (stage, outcome, `next_action_hypothesis`) and `last_handover_path`
   instead of reconstructing branch/stage/in-flight state from conversation. Treat
   `next_action_hypothesis` as re-verify-only against live TaskList/DQ/PR/branch state on resume.
   (Deliberately NOT "lane activity" — see "Lane vocabulary trap" below.)
2. **The remaining task list, verbatim** — open TaskList items + exact next-action.
3. **The user's own messages, verbatim** — the highest-signal anti-drift anchor; nothing else
   in context reconstructs intent this faithfully.

## Lane vocabulary trap (why priority 1 is NOT "lane activity")

The first version of the prompt said "preserve the most recent **lane** activity." That assumes
the active thread is always Brehon phase/lane work. It often isn't — harness-infra sessions,
investigations, and meta-edits all run on `governance-v0` and are NOT lanes. When the session
isn't lane work, "most recent lane activity" and "most recent activity" **diverge**, and a
literal reading would make the summary hunt for lane state that doesn't exist while dropping the
thread that does. Fixed 2026-05-30: priority 1 now says "active thread — a phase/lane, or
harness/infra work, or an investigation; do not assume it was lane work."

## File-content compression rule

- **Committed files:** path + one-line outcome only. The diff is in git — reproducing content is
  waste.
- **Uncommitted in-flight edits:** reproduce content only when the resume would otherwise lose
  work not yet on disk.

## How to keep optimising this (the empirical loop)

The prompt can only be assessed *indirectly* until a real compaction runs against it. The loop:

1. **Before** the next `/compact`, run `/context` and note the "Memory files" bucket size + the
   active thread's actual state (branch, running task ids, open PRs).
2. Run `/compact` with the prompt pasted in.
3. **After**, inspect the produced summary against two tests:
   - **Priority test:** does the summary surface the real active thread (correct branch + tip +
     running task), the open task list, and the user messages? (Should be yes.)
   - **Duplication test:** does the summary restate anything already in the auto-loaded rules /
     MEMORY.md / CLAUDE.md? (Should be NO — every such line is wasted budget.)
4. Feed misses back into `~/.claude/commands/compact-phase.md` AND this note.

This is the `pattern_test_against_reality_not_syntax` discipline: the prompt's *wording* can look
right and still under/over-preserve in practice. Only a real compaction tells you.

## Optimisation history

- **2026-05-29** — Replaced the original phase-resume runbook (read auto-state JSON → DQ → git
  log → emit 5 status lines) with a generic two-priority compaction-focus message (lane activity
  + remaining tasks).
- **2026-05-30** — First effectiveness assessment after a real `/compact`. Findings: priority 2
  (task list) graded A; priority 1 (lane activity) graded C due to the lane-vocabulary trap +
  the session being non-lane work; ~300–500 tokens of duplication identified in the default
  template's Technical-Concepts + security sections (all re-injected by the harness anyway).
  Four fixes folded in: (1) explicit exclusion clause naming the auto-loaded corpus — the biggest
  lever, ~80% of available improvement; (2) de-laned priority 1; (3) committed-files-as-pointers;
  (4) verbatim-keep made explicit for user messages + next-action.
- **2026-05-30 (second assessment, real v1-quality-r3 compaction — `/context` run before).** Both
  tests PASS. **Priority test:** surfaced the active thread (v1-quality-r3, Mode B, in-flight),
  the exact next-action ("poll #528, gate-1 on completion"), and user messages verbatim.
  **Duplication test:** "correctly omitted the four-role model, DQ schema, branch conventions —
  all auto-loaded" → the exclusion clause works; the ~300–500-token duplication from the prior
  assessment is gone. **Partly answers open-question #1** (the exclusion clause DOES suppress the
  auto-loaded-facts restatement; whether a literal "Technical Concepts" *header* still emits is
  still unconfirmed — the assessment didn't mention one, weak evidence it's suppressed too).
  **One gap, NOT a prompt defect:** the next-action "poll #528" went stale because #528 finished
  *during* compaction — a fundamental point-in-time limit of any summary, unfixable by wording.
  Fix landed: priority 2 now marks the next-action as a **hypothesis to re-verify on resume**
  (re-check live TaskList/DQ/PR state before executing). This is a 2× recurrence with v1-RT-r2
  (PMD #502: "Resumed from compacted context; task #430 was already in-flight") → promoted into
  `feedback_thin_wakeup_prompts_verify_live_state.md` (compaction-resume is the same family as
  ScheduleWakeup-resume: a resume-context written before the awaited work resolves).
- **2026-06-11** — Digest-first `/compact` wiring for `/auto-phase` context-management Phase 3.
  Priority 1 now treats `.claude/auto-state/<phase>.json` as the authoritative active-thread
  source for `/auto-phase`: cite `stage_digests[-1]` plus `last_handover_path` rather than
  reconstructing branch/stage/in-flight state from conversation. The digest's
  `next_action_hypothesis` remains a hypothesis to re-verify against live TaskList/DQ/PR/branch
  state on resume; narrative digests are not gates, routers, or cadence inputs.

## Open questions for future tuning

- **(partly answered 2026-05-30)** Does the exclusion clause suppress the default template's
  Technical-Concepts section? The second assessment confirms the auto-loaded *facts* (four-role
  model, DQ schema, branch conventions) are no longer restated. STILL open: whether the harness
  emits a literal "Technical Concepts" *header* regardless of custom-prompt instructions (i.e.
  custom text appended-to vs replacing the default skeleton). Confirm by inspecting a future
  compaction's section headers directly.
- Is there a hard length ceiling the summary is compressed to? If so, suppressing duplication
  should free room for MORE active-thread detail — measure whether priority-1 fidelity improves
  after the exclusion clause lands.

## See also

- `~/.claude/commands/compact-phase.md` — the command file this note describes (user-scope).
- `.claude/lessons/feedback_context_trim_verify_empirically.md` — the parent discipline
  (documented behaviour ≠ observed behaviour; measure, don't assume).
- `.claude/lessons/feedback_session_start_inherited_context_anchors_action.md` — related:
  inherited session-start context shapes the first action.
