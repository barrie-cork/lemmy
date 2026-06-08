# Session retro — 2026-06-07 — m2-late T1 validation

**Harness:** claude-code  
**Session window:** ~2026-06-07T09:30 → ~2026-06-07T10:45 (~75 min)  
**Branch at start:** `4f1fda8b5` (`phase-m2-late-1`)  
**Branch at end:** `a33c0ee9b` (`phase-m2-late-1`)  
**Files touched:** 5 (schema.rs, decision-queue.json, 1 lesson, 2 handovers)  
**Commits:** 2 this session (schema.rs hand-extension + DQ mutation); 4 from prior Opus session also visible in the since-start log

## TL;DR

Session resumed a `validate-pending-laptop` DQ entry (`001f1c47c5dc-001`) left by the prior Opus session after a `/clear` model switch. The prior session had done substantial research (schema.rs regen strategy, diesel_ltree.patch analysis, pre-phase audit probes 1–3) and deposited a verified turnkey spec in `.claude/PRPs/handovers/m2-late-1-t1-validate-resume.md`. This session executed that spec mechanically: 4 schema.rs insertions, parallel migrate-roundtrip + workspace-check, Probe 4 audit completion, DQ resolution. Everything passed. The key finding: **a well-written resume handover with an explicit decision tree completely eliminates the "heavy reasoning required" excuse for keeping Opus on a mechanical execution session** — Sonnet executed the entire task without any judgment calls.

---

## What surprised us

- **The handover worked exactly as designed.** The prior Opus session explicitly noted "safe to /clear + switch to Sonnet" and spelled out a decision tree covering the one judgment call (schema.rs regen on Windows + patch staleness). No improvisation was needed. The mechanical execution finished in ~35 active minutes against ~75 min wall clock (most of that was cargo compile time). This is stronger evidence than any prior session that the "heavy reasoning" assumption for schema-level tasks is usually wrong — the reasoning is upfront in the planning/handover; execution is pattern-following.

- **Probe 4 (exit-code masking) passed cleanly on a fresh audit.** Prior sessions have found wrapper bugs here. The fact that it passed cleanly is worth noting: the `cargo-check.bat` wrapper correctly propagates exit 101 on a bogus feature. This baseline will matter if the wrapper is ever edited.

- **`migrate-roundtrip.sh` took ~11 minutes** for the first-ever run of a new diesel_utils binary (cold compile). The "re-forward" run took 1.19s (cached). Round-trip complete message appeared at the end of a long compile log — the log would look like a failure if you stopped reading before the end. The `echo "MIGRATE_EXIT: $?"` marker was essential.

- **`vswhere.exe is not recognized`** appeared mid-migrate-roundtrip log but the script continued without error. This is a known Windows noise line that the migrate-roundtrip.sh script handles internally. Not a bug, but it looks alarming. Not documented anywhere.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add a one-line comment to `migrate-roundtrip.sh` (or a README note) that `vswhere.exe is not recognized` is expected/harmless noise on systems without VS Developer Tools in PATH and does not indicate failure | Reduces per-session "did the script fail?" confusion; saves ~2 min diagnostic check | minor | 2× (this session + prior sessions that would have hit it) |
| 2 | The handover file's `RESUME first action` section says to also read `m2-late-1-t1-done.md` — that file does not exist. The reference should either be removed from the template or the file should be created at task completion | Eliminates a dead reference in the handover that causes a wasted Read attempt at session start | minor | 1× this session |
| 3 | Add `vswhere not recognized` to the pre-phase harness audit rule (`.claude/rules/pre-phase-harness-audit.md`) under a "Known noise patterns to ignore" callout — `INFO: Could not find files for the given pattern(s).` also appears | Prevents diagnostic dead-ends in future sessions running migrate-roundtrip for the first time | minor | 1× (but high confidence of future recurrence) |

## What to carry forward

- **Model-switch handover discipline is validated at this resolution level.** The Opus→Sonnet switch worked without friction because the handover contained: (1) explicit "safe to switch" statement with reasoning, (2) the one judgment call spelled out as a decision tree rather than prose, (3) anchor verification commands before the edits, (4) "if ANYTHING diverges → stop + raise blocker DQ" as the escape hatch. Future handovers from planning/research sessions to execution sessions should follow the same pattern.

- **Parallel Task B + Task C dispatch** (migrate-roundtrip and workspace-check running concurrently) saved ~11 minutes wall-clock vs sequential. Both are independent — this should be default when both are ready. Already in the handover spec; carry this as an impl discipline.

- **True-exit verification via echo marker, not notification summary.** All three background tasks were verified via the output file's `*_EXIT: N` line, not the notification. Notifications reported "exit code 0" for the workspace check — that one was actually correct this time, but the discipline is right. Notifications have lied 3× previously this phase.

- **Bottom-up insertion order for schema.rs edits** (A4→A3→A2→A1) keeps line-number anchors stable during the edit sequence. The handover recommended it; it worked without any anchor drift.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Resume handover spec (prior Opus session artifact) | ~30 | 0 | high | Eliminated all re-derivation; Sonnet executed without a single judgment call deviation |
| Parallel Task B + C dispatch | ~11 | 0 | low | Concurrency is always available here; handover already recommended it |
| Probe 4 negative exit-code test | 0 | 0 | low | Clean pass; baseline confirmed; ~3 min cost |
| DQ Python move pending→resolved | 2 | 0 | none | Standard pattern; Edit+Python combo worked cleanly |
| Background task echo-exit marker discipline | 2 | 0 | none | Saved potential diagnostic loop if exit was non-zero |

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`.

| Task | Files | Commits | Runtime (min) | Max log silence (min) | Watchdog risk |
|---|---:|---:|---:|---:|---|
| schema.rs hand-extension + DQ resolution | 2 | 2 | ~35 active / ~75 wall | ~11 (migrate-roundtrip compile) | low — laptop-side, no watchdog |

No Junior workers dispatched this session. No watchdog risk. The ~11 min migrate compile silence would be borderline on a daemon worker (threshold ~40 min) but is fine on laptop.

## Decisions to revisit

- The `m2-late-1-t1-done.md` handover reference: decide at T2 brief authoring whether the convention should be that a "done" handover is always written at task completion, or whether it's optional. The current gap (reference in resume handover to a file that doesn't exist) suggests the convention isn't enforced. One option: the DQ mutation commit IS the done marker — no separate file needed.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
