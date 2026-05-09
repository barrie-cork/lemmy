# Session retro — 2026-05-07 — minimax-ab-deferred

**Harness:** claude-code
**Session window:** ~14:10 UTC → ~14:25 UTC (~15 min)
**Branch at start:** `ec711ed1d` (`phase-v1-SL-b`)
**Branch at end:** `ec711ed1d` (`phase-v1-SL-b`) — no working-tree changes
**Files touched:** 2 (memory only — `project_minimax_ab_trial_deferred.md` created, `MEMORY.md` index updated)
**Commits:** 0 (auto: 0, explicit: 0)

## TL;DR

User asked whether to swap the impl-task subagent to MiniMax M2.7 to reduce Claude Code token spend, citing a prior A/B setup. Investigation found (a) no comparison data exists — the "MiniMax in daemon logs 2026-04-26" event was accidental config drift, not a deliberate A/B; (b) the tiering patch *already* has a per-job `envOverrides` mechanism built specifically for A/B trials, exposed via `junior task add --env-override KEY=VALUE` but **not** via the MCP `create_task` tool. Trial design landed in memory; user opted to defer execution to the first phase after v1-SL-b ships. Highest-leverage finding: the `envOverrides` infrastructure is more capable than the current memory corpus reflected, and the MiniMax decision (`feedback_brehon_anthropic_only.md`) was a reaction to drift — it didn't close out the A/B itself, it just removed the accidental drop-in.

---

## What surprised us

- **Existing `envOverrides` infrastructure was already A/B-shaped.** The tiering patch's executor.ts has explicit AB-test plumbing (`// AB test: per-task env overrides take precedence over role-model mapping`) and the CLI exposes `--env-override`. The memory corpus described model selection as "per-role tiering decided" but didn't mention that per-task A/B was already a first-class capability. Config-shape surprise, low cost — but worth recording so the next session doesn't re-investigate.
- **MCP `create_task` does NOT pass `env_override` through.** The MCP tool surface is narrower than the CLI surface. For an A/B trial, the only path is `ssh homeserver "junior task add ... --env-override ..."`. Worth noting because the advisor's polling-loop discipline assumes MCP-driven task creation.
- **The 2026-04-26 MiniMax incident was misread by the memory corpus.** `feedback_brehon_anthropic_only.md` records "Brehon stays on Anthropic" as a *decision*, but the underlying event was accidental drift (a stale systemd drop-in) — not a deliberative A/B that ran and lost. The decision sentence is more conclusive than the evidence supports. The user's framing ("if there is no data then proceed to configuring") caught this — they didn't accept the memory at face value.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add a one-line note to `feedback_brehon_anthropic_only.md` clarifying the 2026-04-26 event was config drift, not a failed A/B — the decision to stay Anthropic-only was provisional pending an actual trial | Future advisor sessions reading the memory don't conclude "we tried MiniMax and rejected it" when no trial ran | minor (1 file edit) | 1× this session, 1× memory was originally written 6 days ago — meets 2× threshold |
| 2 | When user asks "do we have comparison data on X" and a memory says "X was rejected", verify the memory's evidence base before answering. The memory may record a decision without recording that no data backed it | Avoids advisor-as-relay parroting of conclusive-sounding memory entries | minor (discipline, not file change) | 1× this session — single instance, note but don't promote |
| 3 | Document in `feedback_brehon_subagent_model_effort_assignments.md` (the canonical model-config memory) that `envOverrides` is the per-task A/B mechanism — name the CLI flag and the executor.ts injection point | Next time someone asks "can we A/B model X", the answer is one memory read away instead of a 20-tool-call investigation | minor (1 file edit) | 1× this session — single instance, propose but flag as low-priority |

## What to carry forward

- **Verify before recommending from memory.** This session's user prompt forced the verification ("if there is no data then proceed to configuring"). The advisor's first impulse was to cite `feedback_brehon_anthropic_only.md` — the user's framing exposed that the memory's conclusiveness exceeded its evidence. Pattern: a memory that names a decision is not the same as a memory that names data behind a decision.
- **Pattern-following deferral over execution-mid-flight.** User chose to defer the trial to a clean phase boundary rather than start it during v1-SL-b. Matches prior pattern (`feedback_temporal_isolation_brehon.md`) of preferring discrete-phase execution. Worth carrying forward as a reflexive default for any "should we add X to the active phase" question.
- **Memory-only sessions are valid retro targets.** Zero commits, two files written, ~15 min of investigation + design. The retro discipline applies regardless of commit count — the value was the memory write that survives the session boundary.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Memory pre-read (feedback_brehon_anthropic_only + feedback_brehon_subagent_model_effort_assignments) | 8 | 0 | medium | Surfaced the 2026-04-26 drift incident and the `envOverrides` design hint in one read pair |
| SSH probe of executor.ts + drop-ins | 5 | 2 | low | Confirmed clean Anthropic-only state; 2 min wasted on a wrong path searching for `mcp/server.ts` (turns out compiled binary, not src) |
| Strings-on-binary fallback | 1 | 1 | low | When grep on src failed, fell back to `strings dist/junior` — worked but produced large output that triggered the persisted-output cap once |
| AskUserQuestion (NOT used) | 0 | 0 | low | Could have used it for the local-vs-CLI question on trial mechanism, but user's "1" response made it unnecessary |
| Memory write (project_minimax_ab_trial_deferred.md + MEMORY.md edit) | 0 | 0 | none | Clean write, single Edit append to MEMORY.md |

## Complexity scores (heavy tasks only)

None. Session was short (~15 min), memory-only, no impl-class work. Per the lesson: only flag tasks >55min runtime, >40min log silence, or >8 files. None applicable.

## Decisions to revisit

- The MiniMax trial itself, when v1-SL-b ships. The deferral memory captures the trigger ("at the start of the first planning session after v1-SL-b retro is signed off"). The memory mechanism is auto-loaded into the next session's MEMORY.md, so this should self-surface.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

For each item from "What to change" that meets the threshold, the user may approve promotion. Boxes UNCHECKED by default.

- [ ] Change #1 (clarify drift-vs-trial in `feedback_brehon_anthropic_only.md`): edit the existing lesson in place, add one line under "Why" stating the underlying event was config drift not a deliberate A/B
- [ ] Change #3 (document `envOverrides` in `feedback_brehon_subagent_model_effort_assignments.md`): edit the existing lesson, add a "Per-task A/B mechanism" subsection with CLI flag + executor.ts pointer

Change #2 is discipline, not a file change — recorded but not promoted (single-occurrence noise per the recurrence rule).

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
