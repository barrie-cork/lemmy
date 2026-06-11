# Session retro — 2026-06-11 — retro-check-session-age-gate

**Harness:** claude-code
**Session window:** ~2026-06-11 17:50Z → 18:05Z (~15 min)
**Branch at start:** `91918240c` (`governance-v0`)
**Branch at end:** `b19eb1806` (`governance-v0`)
**Files touched:** 1 (`.claude/hooks/retro-check.sh`)
**Commits:** 1 (auto: 0, explicit: 1)

## TL;DR

A brand-new CC session got nagged for a post-task retro within seconds of
starting. Root cause was a category error in `retro-check.sh`: the hook's
`WINDOW_MINUTES=60` is a *look-back window for finding an existing retro*, not
a *grace period before enforcement* — the hook had zero notion of session age,
so a fresh idle session (no retro yet) was blocked on its first turn-end.
Fixing it surfaced a second, latent bug: the hook referenced
`CLAUDE_SESSION_ID`, **an env var that does not exist** — `session_id` is only
delivered via stdin JSON, which the hook never read, so it always fell through
to the unstable `PPID`. The fix reads the real stdin `session_id`, adds a
proper session-age gate (no nag until ≥60 min old), exempts `junior/*` worker
branches, and self-prunes leaked markers. Top carry-forward: **a "this fires
too early/often" complaint about a time-windowed hook is a signal to check
whether the window is a look-back or an age-gate — they look identical in a
config but mean opposite things.**

---

## What surprised us

- **The "60-minute" the user expected and the "60-minute" in the code were
  semantically unrelated.** The user's mental model was "enforce after 60 min
  of session." The code's `WINDOW_MINUTES=60` was "look back 60 min for an
  existing retro." Same number, opposite meaning — which is exactly why the bug
  was invisible to anyone scanning the config. The hook never had an age concept
  at all; it just *looked* like it did because the look-back happened to be 60.
- **`CLAUDE_SESSION_ID` is not a real environment variable.** The hook used
  `${CLAUDE_SESSION_ID:-${PPID:-unknown}}` in two places as if it were the
  stable session key. It never resolved — Stop hooks receive `session_id` via
  **stdin JSON**, not env. The hook's own v1 design notes (lines 84–87) even
  documented that PPID is unstable across Stop invocations, yet the code was
  silently relying on PPID the whole time. The bug the notes warned about was
  never actually fixed; it was masked by a phantom env var.
- **The hook never consumed its own stdin.** Claude Code pipes the hook-input
  JSON to the command, but `retro-check.sh` ignored it entirely — all the
  session metadata it needed for a correct age gate was sitting unread on fd 0.
- **My first move was wrong and the user caught it.** I reached for `Glob` on
  the hook path before reading the file — the user rejected the tool call. The
  correction was right: read the artifact before acting on it.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When a user reports a time-gated hook "fires too early/often," **first classify the window: look-back vs age-gate** before touching logic. Add this as a one-line diagnostic habit (candidate lesson `feedback_time_window_lookback_vs_agegate.md`). | Avoids mis-diagnosing the trigger; the two are config-identical but opposite in meaning. | minor | 1× this session + 1× prior (`feedback_runbook_audit_drift_post_event_check.md` "every claim is a hypothesis" is the same class) |
| 2 | **Audit other hooks for phantom `CLAUDE_SESSION_ID` usage** — grep `.claude/hooks/*.sh` for `CLAUDE_SESSION_ID`; any consumer relying on it for correctness (not just logging) is silently degraded. | Catches the same latent bug elsewhere before it bites. | minor | 1× this session (see §Decisions to revisit for the grep result) |
| 3 | When reading session identity in a Stop/PostToolUse hook, **read stdin JSON for `session_id`/`transcript_path`** — never assume an env var. Encode the canonical stdin-read snippet in a lessons file or the hook-authoring reference. | Future hooks get a stable session key for free; no repeat of the PPID-fallback trap. | minor | 1× this session, recurrence-eligible if a 2nd hook needs session identity |
| 4 | The session-start marker dir `/tmp/cc-retro-sessions/` now holds two unrelated artifact kinds (retry-counter keyed on PPID, age-marker `*.start` keyed on session_id). **Consider namespacing** (`*.retry` vs `*.start`) so a future reader doesn't conflate them. | Prevents a future edit from pruning/clobbering the wrong file class. | minor | 1× this session |

## What to carry forward

- **Read the artifact before acting on it** — the rejected `Glob`-first move was
  the right thing to reject. Reading `retro-check.sh` first is what surfaced both
  the look-back/age-gate confusion *and* the phantom env var. (Matches
  `feedback_read_canonical_before_writing_spec.md` discipline applied to hooks.)
- **Verify the docs claim, don't infer it.** The `CLAUDE_SESSION_ID` finding
  came from WebFetching the live hooks docs, not from memory. The env-var list
  and the Stop-hook stdin schema were both load-bearing; guessing either would
  have produced a subtly-broken gate.
- **Empirically test every branch of a hook's decision tree before committing.**
  All four paths were exercised against real and synthetic PMDs: fresh→exit 0,
  young→exit 0, old+retro-present→exit 0, old+empty-PMD→exit 2. The exit-2 proof
  required pointing at a throwaway empty SQLite DB via `PROJECT_MEMORY_DB` — that
  isolation is what confirmed enforcement still works rather than was silently
  disabled. (Matches `pattern_test_against_reality_not_syntax.md`.)
- **Pre-commit canonical-checkout hygiene held** — `git worktree list` + `git
  status --short` confirmed single worktree on `governance-v0`, no foreign WIP,
  before the direct-commit (per `phase-branch.md` meta-work policy +
  multi-lane Hard refusal #7).

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| WebFetch (claude.com hooks docs) | 15 | 1 | high | Surfaced the non-existent `CLAUDE_SESSION_ID`; the 1 wasted min was the host-redirect round-trip. Decisive — the whole gate keys on the real stdin `session_id`. |
| AskUserQuestion (trigger-condition fork) | 5 | 0 | none | Clean 4-way fork; user picked session-age gate immediately. Avoided me guessing between age-gate / work-gate / disable-on-gov-v0. |
| Read `retro-check.sh` (full) | 10 | 0 | medium | Reading the whole 323-line hook surfaced both bugs at once; cheaper than iterating on a partial view. |
| Bash empirical 4-path verification | 8 | 2 | low | The 2 wasted min were scenario-3 first exiting 0 (legit recent retro in PMD) — had to add the empty-DB isolation to prove exit-2. Worth it. |
| Glob `retro-check.*` (rejected) | 0 | 1 | low | User-rejected; correct rejection. Logged as a course-correction, not a tool failure. |

## Complexity scores (heavy tasks only)

No heavy task this session — single-file hook edit. Complexity:
`1/1/~15/0` (1 file, 1 commit, ~15 min wall-clock, no Junior log-silence
dimension — interactive, not dispatched). Well inside the comfortable zone;
no carry-forward signal.

## Decisions to revisit

- **Audit grep result (run this session): clean.** `grep -rn CLAUDE_SESSION_ID
  .claude/hooks/` returns only (a) my new explanatory comments and (b) line ~347
  in `emit_retro_bypass_log`, which uses `${CLAUDE_SESSION_ID:-${PPID:-unknown}}`
  for the bypass-log `session_id` field — harmless (logging only, falls back to
  PPID fine). **No other hook is affected.** Worth replacing that line with the
  now-parsed `$SESSION_ID` for consistency in a future touch; left as-is to keep
  this commit minimal. §What-to-change #2 is therefore closed (no other consumer
  to fix).
- **Should the age-gate threshold be configurable** rather than hardcoded
  `ENFORCE_AFTER_MINUTES=60`? Not worth it yet — 60 matches the look-back
  window and the user's stated intent. Revisit only if a different cadence is
  requested.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] §What-to-change #1 (look-back vs age-gate classification): promote to `.claude/lessons/feedback_time_window_lookback_vs_agegate.md` — meets threshold (1× here + 1× prior "claims are hypotheses" class).
- [ ] §What-to-change #3 (read stdin for session_id in hooks): promote to `.claude/lessons/feedback_hook_session_id_from_stdin.md` once a 2nd hook needs session identity (recurrence-eligible, not yet met).
- [ ] §What-to-change #2 (audit other hooks for phantom env var): one-off action, not a lesson — run the grep next session, fix any consumer, then close.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
