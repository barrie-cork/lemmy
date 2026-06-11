# Session retro — 2026-06-12 — unattended-gate-allowlist

**Harness:** claude-code
**Session window:** 2026-06-11 ~23:30 → 2026-06-12 ~00:50 UTC+1 (~80 min)
**Branch at start:** `a1447e139` (`governance-v0`)
**Branch at end:** `5ac7d891a` (`governance-v0`)
**Files touched:** 4 (3 in-repo committed + 1 user-scope skill, not git-tracked here)
**Commits:** 1 (auto: 0, explicit: 1) — `5ac7d891a feat(advisor): /auto-phase --unattended gate allowlist`

## TL;DR

The user asked to "add Haiku model agents to /auto-phase that monitor Junior tasks
and tell the advisor when to progress." The session's main thread was *not*
building that — it was **reshaping the ask through four AskUserQuestion gates until
the real problem surfaced** (true unattended progression), then recognising that a
cheap-model monitor agent was the wrong fix entirely: the monitor role is already
filled twice (daemon completion hook + advisor's own ScheduleWakeup loop), so a
Haiku agent would *add* a hop and a mis-classification surface, not remove one. The
delivered feature is a `--unattended` policy allowlist (no agent): auto-clear the 2
no-judgment gates (e2e→local, retro sign-off), park-and-ping the 4 judgment gates.
The most load-bearing finding: **when a user asks for a specific mechanism, the
highest-value move is to find the friction underneath it before building** — here it
turned a "spin up a cheap agent" project into a ~67-line policy edit that's strictly
safer. Top change proposal: the AskUserQuestion-to-narrow-the-ask pattern worked so
well it should be the default opening move for any "add a mechanism X" request.

---

## What surprised us

- **The literal ask was the wrong fix, and four clarifying questions revealed it.**
  The user wanted a Haiku monitor agent. Reading the actual architecture (daemon
  hook + advisor ScheduleWakeup) showed the monitor role was already doubly filled.
  Surprising in a good direction: the design pushback was *welcomed*, not resisted —
  each AskUserQuestion narrowed the real requirement (faster wake? context? mid-task
  signal? → "true unattended progression").
- **"True unattended progression" collided head-on with an already-rejected design
  (`--auto-all-gates`, L15).** The user's stated goal initially read as exactly the
  thing the harness deliberately forbids. The resolution wasn't "refuse" — it was
  finding the narrow safe subset (2 of 6 gates carry no judgment/code/ADR/billing
  risk). Surprising that the safe allowlist was so small (gate 6 always; gate 4 only
  as a fixed `local` default, never a model choice).
- **The user *refined my proposal to be simpler and safer than what I offered.***
  I proposed gate 4 use a "pre-set default you choose." The user said "gate 4 default
  is always local" — which removed a configuration surface AND removed the only
  money/exposure risk (local is free/unbilled). A rare case where the user's
  constraint was tighter and cleaner than the assistant's.
- **A concurrent session was driving the `/auto-phase` `test` sandbox in the same
  repo during this session.** Commits `479525f2d` + `6f1275d63` (test-bm-triage)
  landed at 00:46–00:47, one minute after my feature commit at 00:45, same author
  (solo-dev). My `git status` foreign-WIP check at commit time was clean (they
  committed *after* mine), so no collision — but it's a live reminder that the
  canonical checkout had concurrent activity.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Codify the "narrow-the-ask-before-building-a-mechanism" pattern as a lesson `feedback_narrow_the_ask_before_building_mechanism.md`. When a user requests a specific mechanism ("add X agent / X hook / X script"), the first move is 1-4 AskUserQuestion gates to surface the *friction underneath* the requested mechanism — the requested mechanism is a hypothesis about the fix, not the fix. | Turns over-engineered "build a new agent" projects into minimal policy/config edits; prevents shipping infra the user didn't actually need | minor (one lesson) | 1× strongly this session; cross-ref prior `feedback_falsifiable_hypothesis_before_structural_fix.md` (DQ premise ≠ contract) — same family, 2nd in family |
| 2 | The user-scope skill body `~/.claude/commands/auto-phase.md` is NOT git-tracked in brehon-fork, so the executable half of this feature has no version history / no CR review / no rollback point. Propose: a tracked mirror or a `scripts/sync-user-skills.sh` snapshot under brehon-fork so user-scope skill edits get a durable diff trail. | Restores audit trail + rollback for the load-bearing executable spec; today only the in-repo 1/3 of the feature is recoverable from git | medium | 1× here, but structural — affects every `~/.claude/commands/*.md` edit |
| 3 | Add a `--dry-run --unattended` dogfood trace to the next actual phase that ships, and a retro check that the gate-6 3-check sanity gate fires correctly on a real (non-truncated) retro. The feature shipped with *mental* dogfood only (3 traces), no live exercise. | Converts the unverified-spec risk into evidence before the first unattended phase relies on it | minor (one dry-run + observe) | 1× — feature is brand new, never executed |

## What to carry forward

- **AskUserQuestion as the opening move for ambiguous mechanism requests.** Four gates
  this session, each one load-bearing (real problem → gate behaviour → allowlist scope
  → notifier impl). Zero wasted questions; each changed the design. This is the
  cross-harness habit worth keeping: don't build, narrow first.
- **Read the actual architecture before accepting the premise.** Reading
  `auto-phase.md` + the daemon-hook lesson + the state template *before* designing
  revealed the monitor role was already filled — which dissolved the whole original
  framing. The ~4 file reads paid for themselves many times over.
- **Triple-surface enforcement for a safety contract.** The allowlist is enforced in
  three independent places (routing-table rows, Phase 7 hard-refusals, failure-mode
  table) that were cross-checked against each other via 3 mental traces. For a
  load-bearing safety boundary, redundant enforcement that's *verified consistent* is
  the right discipline — a single point would be one edit away from silent breach.
- **Audit-honesty label discipline.** Auto-cleared gates log `auto-approve-unattended`,
  never `user`. Carrying forward the principle: any automation that stands in for a
  human decision must record *that it did so* in a distinguishable way — same family as
  the `answered_by: "advisor"` attribution rule and the `bm false-success` pattern.
- **Verify-before-trusting on the PMD write.** Searched for a duplicate before writing
  (none), confirmed the live store was homeserver-served (recent rows present), then
  FTS5-verified the write landed (ID 948). Surfaced the cosmetic `repo_name: "unknown"`
  quirk of the HTTP `memory_write` path honestly rather than glossing it.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`. This was a single-role
(advisor authoring) session, not a four-role orchestration — scoring is per
tool/skill batch, not per Junior role.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| AskUserQuestion ×4 (real-problem / gate-behaviour / allowlist / notifier) | 90+ | 0 | high | Each gate reshaped the design; collectively turned a "build a Haiku agent" project into a ~67-line policy edit. Highest-leverage tool of the session. |
| ToolSearch (junior hooks, memory_write, memory_search) | 3 | 2 | low | Two misses on `select:` syntax for tools not yet deferred-listed; fell back to keyword search cleanly. Minor friction. |
| Architecture reads (auto-phase.md, daemon-hook lesson, state template) | 40 | 0 | medium | Dissolved the original premise (monitor already filled). The read that mattered most. |
| Edit ×8 + Write ×2 (skill body, refs, template, lesson) | — | 5 | low | One placeholder-style inconsistency in the schema-backfill block caught + fixed on re-read (the `<...>` pseudo-code didn't match surrounding concrete Python). |
| memory_write + verify (PMD ID 948) | 5 | 0 | low | Clean; surfaced `repo_name: "unknown"` HTTP-path quirk honestly. |
| 3 mental dogfood traces (gate 5 / gate 6 fail / resume-without-flag) | 15 | 0 | none | Confirmed triple-surface consistency before commit. No live exercise though — see What-to-change #3. |

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. This session ran no
Junior impl-tasks (advisor-authored throughout, interactive), so the watchdog-envelope
metric (`max-log-silence`) is N/A. Recording the one heavy authoring task for shape:

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| --unattended spec authoring (skill body + refs + template + lesson) | 4 | 1 | ~50 | N/A (interactive, no Junior worker) |

No task exceeded the >8-files / >55-min / >40-min-silence flags. Comfortable zone.

## Decisions to revisit

- **Gate 5 (merge) auto-clear stays rejected — confirm it stays rejected.** The user
  was offered "also auto-clear gate 5" and declined. That boundary is the load-bearing
  one; a future session feeling pressure to "make it more autonomous" must not quietly
  cross it. The lesson + Phase 7 hard-refusal #2 guard this, but it's worth a conscious
  re-affirm at the first real unattended phase.
- **Sticky-to-restart caveat:** `--unattended` only takes effect next `/auto-phase`
  invocation (skill body loads once at session start). Worth confirming the user
  internalised this — it's the kind of deferred-effect that surprises at use time.
- **User-scope skill versioning (What-to-change #2)** warrants its own small follow-up —
  it's a structural gap affecting all `~/.claude/commands/` edits, not just this one.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

Boxes UNCHECKED by default; user checks to authorise.

- [ ] What-to-change #1: promote to `.claude/lessons/feedback_narrow_the_ask_before_building_mechanism.md` (cross-harness lesson). Meets threshold via family-recurrence with `feedback_falsifiable_hypothesis_before_structural_fix.md` (premise-is-a-hypothesis, not a contract — 2nd in family).
- [ ] What-to-change #2: new `scripts/sync-user-skills.sh` snapshot OR tracked mirror for `~/.claude/commands/*.md` edits — restores git audit trail for user-scope skill bodies. (Structural; single-instance but high blast-radius.)
- [ ] What-to-change #3: live `--dry-run --unattended` exercise at the next shipping phase (execution item, not a lesson — flip when done).
- [x] PMD eval write — DONE this session (ID 948, the feature lesson itself, not a retro eval). A *retro* eval is optional below.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase reliability section omitted:
session edited the /auto-phase **spec** but did not invoke it or mutate any
auto-state JSON — trigger condition (revised 2026-05-09) does not fire._
