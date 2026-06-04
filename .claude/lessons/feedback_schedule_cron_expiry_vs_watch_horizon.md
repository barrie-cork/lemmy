---
name: /schedule and CronCreate auto-expire at 7 days — use a durable MEMORY.md/PMD watch for months-horizon triggers
description: A /schedule cron or CronCreate job is the wrong mechanism for a watch whose trigger is weeks-to-months away — recurring jobs auto-expire after 7 days (CronCreate) and session-scoped ones die with the session, so the watch silently lapses while reading as "we're watching". For a long-horizon trigger, write a durable MEMORY.md + PMD watch entry (re-read every session) with a mechanical re-check command instead.
type: feedback
---

When asked to "set a watch on X" (adopt-when-upstream-ships, revisit-when-condition-met, poll-until-Y), match the watch mechanism to the trigger's **horizon**. A `/schedule` cron or `CronCreate` job is correct only for short horizons; for a trigger that's weeks-to-months away, it silently lapses.

**Why:** v1-closeout Phase 5 (2026-06-04) needed a watch to "adopt upstream extism the moment it ships wasmtime ≥ 42" — a trigger horizon of months (extism's last wasmtime bump was 41, and ≥42 had no live upstream PR). The plan literally said "set a `/schedule` watch on extism PR #847". Two problems with that mechanism:

- **`CronCreate` recurring jobs auto-expire after 7 days** (they fire one final time, then delete — this bounds session lifetime by design). A months-horizon watch set as a 7-day cron lapses ~3 weeks before the soonest plausible trigger, with no signal that it lapsed.
- **Session-scoped crons die when the Claude session exits** (in-memory; `durable: true` persists to disk but still carries the 7-day recurring-expiry).
- The failure mode is the dangerous kind: the cron *reads as* "a watch is in place" right up until it silently expires. False confidence — the same class as a `ScheduleWakeup` prompt that goes stale (`feedback_thin_wakeup_prompts_verify_live_state.md`).

**How to apply:**

- **Estimate the trigger horizon first.** Days → a `/schedule`/`CronCreate` cron is fine (and re-create it weekly if it must outlive 7 days). Weeks-to-months, or "whenever upstream gets around to it" → do NOT use a cron.
- **For a long/unknown horizon, write a durable watch entry**, in two places (the cross-lane-shared layer, per `pmd-invariants.md`):
  1. A PMD memory file under `~/.claude/projects/<repo>/memory/watch_<slug>.md` (type: project), with a **mechanical re-check command** (the exact `gh api` / `curl crates.io` / `WebFetch` line that tests whether the trigger is met) and the **trigger-met action**.
  2. A one-line pointer in `MEMORY.md` under "Watch / promote-if-recurs", so every session reads it at start.
- The watch then survives indefinitely (re-read each session, never expires) and the re-check is a cheap mechanical step run "when convenient" — at a milestone boundary, a deps sweep, or any session that touches the area.
- **Canonical example:** `watch_extism_wasmtime_42_adopt.md` (the Phase 5 watch) — carries the `curl crates.io/api/v1/crates/extism` re-check + the bump-not-fork trigger-met action + the verified-facts so future sessions don't re-derive.

**Symptom to recognise:** a plan or prior session says "set a `/schedule` watch on X" where X is an upstream release, a dependency fix, or any event not under your control with no committed date. That's the trigger to swap the mechanism — a cron will lapse; a durable MEMORY.md/PMD entry won't.

**Generalises to:** any "watch for an external event with an unbounded horizon" task. The decision rule is mechanism-matches-horizon: ephemeral poller (cron/`ScheduleWakeup`) for hours-to-days; durable session-read entry for weeks-to-months-to-indefinite. Crons are pollers, not memory.

**Related lessons:**
- `feedback_thin_wakeup_prompts_verify_live_state.md` — same family (resume-context / poller written before the awaited event resolves; decays silently)
- `feedback_runbook_audit_drift_post_event_check.md` — the parent lesson; this `/schedule` footgun was 1 of the 5 stale-plan-facts that session
- `pmd-invariants.md` §1 — why the watch goes in the cross-lane-shared PMD, not a per-lane file
