# Session retro — 2026-05-29 — quality-r2 resume + Opus bump

**Harness:** claude-code
**Session window:** ~2026-05-29T15:00Z → ~2026-05-29T16:30Z (~90 min)
**Branch at start:** `cf3a410b9` (`governance-v0`)
**Branch at end:** `149a14b9d` (`governance-v0`)
**Files touched:** 16
**Commits:** 4 (advisor: 4, auto: 0)

## TL;DR

Short session covering three distinct tasks: (1) cold-resume after a previous session left quality-r2 Task 0 (#509) in-flight, (2) a complete rewrite of `docs/brehon-law-inspired-network/04-data-model-and-api.md` from live code as a side-track request, and (3) a daemon wiring check + planning role upgrade from Opus 4.7 → 4.8 including executor.ts patch, rebuild, and service restart. The 04-data-model rewrite was the largest artifact (~1100 net lines changed) and was authorship-delegated via subagent correctly. The main carry-forward is that the `restore-junior-server-patches.sh` drift check does not detect model-string changes (it only checks structural markers), requiring a manual SCP after patch-mirror update.

---

## What surprised us

**Advisor:**
- The restore script's drift-check passes even when the model string inside the patch is outdated — it only grep-checks structural marker strings (`childEnv.ANTHROPIC_MODEL = roleModel`), not the actual model value. So updating the patch mirror file on the laptop and running the restore script reports "all patches applied / no changes needed" while the daemon still has the old `'opus[1m]'` string. Required a manual SCP + rebuild. This is a silent-mismatch class that could persist across upstream pulls if not caught.
- `opus[1m]` was the actual string injected into `ANTHROPIC_MODEL` — this is not a real Claude CLI model alias. It was presumably working only because the CC CLI falls back to the subscription-tier default when `ANTHROPIC_MODEL` is unrecognised. Planning tasks may have been using the subscription default (Opus 4.7 on Max plan) rather than any explicit model, meaning the model injection was silently inoperative for this role for an unknown number of phases. Verified that `claude-opus-4-8` resolves correctly on the daemon.

**04-data-model rewrite side-track:**
- The plan file for the rewrite (`v1-data-model-doc-and-04-retire.plan.md`) was created as an untracked file by the subagent but not committed (the plan was scaffolded but the task was authorship-only — the advisor wrote the doc directly via a subagent, not via Junior). The plan file sits untracked. This is a minor loose end — either commit or delete.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add a model-value drift check to `restore-junior-server-patches.sh` — after the structural marker check, grep the `ROLE_MODEL_MAP` in the live `/opt/junior-src/src/daemon/executor.ts` and compare against the patch mirror's expected values. Exit non-zero if any value mismatches. | Prevents the "restore says OK but model is wrong" silent failure from persisting across upstream pulls. | minor | 1× this session; structurally inevitable on any model upgrade |
| 2 | When the model-value drift check fires, update PATCH-MARKER.md on the daemon automatically in the restore script (not just after manual SCP). The PATCH-MARKER.md is the EliteDesk's human-readable record — it should stay in sync with executor.ts, not require a separate manual edit. | Keeps PATCH-MARKER.md accurate without a separate step. | minor | 1× this session |
| 3 | Commit or delete `.claude/PRPs/plans/v1-data-model-doc-and-04-retire.plan.md` — currently untracked. If the 04-rewrite doc was the full deliverable (no impl tasks planned), the plan file is superfluous; delete it. If a future "retire 04 and redirect all refs" task is planned, commit it. | Removes filesystem noise / prevents future confusion about what's pending. | minor | 1× this session |
| 4 | Update `feedback_brehon_subagent_model_effort_assignments.md` memory entry — the planning row still describes 4.7 reasoning ("Opus 4.7 effort scale shifted upward", "xhigh on Opus 4.7 today is roughly what max on 4.6 was"). These rationale lines are now stale for 4.8. | Prevents a future session from consulting the memory and deriving a wrong effort recommendation based on 4.7 rationale. | minor | 1× (upgrade just happened) |

## What to carry forward

**Advisor:**
- Model-injection verification pattern: after any daemon patch that changes a model string, do NOT trust the restore script's drift check alone — manually grep the live executor.ts for the specific model value before declaring done. Command: `ssh homeserver "grep 'planning' /opt/junior-src/src/daemon/executor.ts"`.
- Pre-SCP backup pattern is already in the restore script (`cp ... .bak-$ts`). The script correctly backs up before overwriting — no changes needed there.
- `claude-code-guide` subagent lookup confirmed `claude-opus-4-8` is the correct no-date-suffix model ID. This is now the canonical reference; Haiku 4.5 has the dated suffix; Opus 4.8 and Sonnet 4.6 do not.
- The 04-data-model rewrite via a subagent (using the `general-purpose` agent to survey + write) saved ~30 min of manual cross-referencing. For reference-doc rewrites from live code, the pattern of "subagent with explicit file list + structural schema" worked cleanly on first attempt.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Cold-resume + runlog entry | 5 | 0 | none | Mechanical; resume state was clean from handover brief |
| 04-data-model rewrite via subagent | ~30 | 5 | low | Subagent produced accurate first-pass. 5 min wasted on scope clarification (user confirmed full rewrite vs patch) |
| Daemon wiring check (executor.ts grep + PATCH-MARKER read) | 10 | 0 | none | Clean discovery path |
| restore-junior-server-patches.sh --check | 0 | 8 | **high** | Reported all OK while model was wrong — silent false-positive on model-value drift |
| Manual SCP + rebuild + restart | 10 | 0 | none | Straightforward once drift was identified |
| `claude-code-guide` subagent for model ID confirmation | 3 | 0 | none | Fast authoritative lookup; correct answer on first call |
| Task 3 brief authorship (v1-quality-r2-impl-3.md) | 20 | 0 | none | Standard brief; mandatory file-class lesson table fired correctly |

## Complexity scores (heavy tasks only)

No Junior impl-tasks ran in this session. The 04-data-model rewrite was advisor/subagent-authored (no Junior dispatch).

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| 04-data-model rewrite (subagent) | 4 | 1 | ~25 | N/A (no watchdog; laptop subagent) |
| Daemon wiring check + model bump | 5 | 2 | ~30 | N/A |

## Decisions to revisit

- The untracked plan file `v1-data-model-doc-and-04-retire.plan.md` — if there's a future task to retire 04 and update all cross-references in the codebase, commit the plan. Otherwise delete. (See "What to change" #3.)
- The `feedback_brehon_subagent_model_effort_assignments.md` rationale lines about "Opus 4.7 effort scale shifted upward" are now stale. A future session doing a model-performance review should update the reasoning for 4.8. (See "What to change" #4.)
- Whether `'opus[1m]'` was ever being honoured by the CC CLI as an alias — if it was silently falling back to the subscription default for all prior planning tasks, the quality-r2 plan (authored as the first 4.8 task) is the first to genuinely use the configured model. Worth knowing as baseline context for the 4.8 calibration signal.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Model-value drift check in restore script**: add grep-based model-value verification to `restore-junior-server-patches.sh` after structural marker check. Promote to `.claude/lessons/feedback_junior_server_patch_model_drift.md` — the lesson is "restore script model-drift check must verify values, not just structural presence."
- [ ] **Post-SCP verify pattern**: after any model-string update via SCP, always grep the live file before building. Codify as a comment in the restore script and/or a brief line in `CLAUDE.md §Four-role model` "Patch health" note.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
