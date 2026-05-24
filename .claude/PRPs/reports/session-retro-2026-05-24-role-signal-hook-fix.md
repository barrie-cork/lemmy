# Session retro — 2026-05-24 — role-signal-hook-fix

**Harness:** claude-code
**Session window:** 2026-05-24T09:00:00Z → 2026-05-24T10:30:00Z (~90 min)
**Branch at start:** `eca94497e` (`governance-v0`)
**Branch at end:** `097a51d09` (`governance-v0`)
**Files touched:** 5 (hook, drain script, smoke brief, handover, runlog)
**Commits:** 5 (all explicit, all on governance-v0 → forward-merged to phase-v1-RT-r3)

## TL;DR

Session 4 of role-customization. Goal: ship the Junior-side role-signal Stop hook fix per session-3 handover §3.3. Shipped the fix in two iterations — the first patch (`feaa75db9`) was built on the handover's named-but-untested assumption that `CLAUDE_PROMPT` env exists in `claude -p` mode; smoke task #448 revealed it doesn't (Junior daemon doesn't set it; Claude Code's `-p` mode is env-minimal by design). Pivoted to parse `transcript_path` JSONL for the role tag (`729b13312`); verified end-to-end via smoke #449 with both rows (sentinel 534 + role-signal sibling 535) landing in canonical PMD. **Top change proposal: when a handover's "smallest change" is built on an untested assumption about a system property, the next session MUST empirically verify the assumption BEFORE the patch — 2x recurrence with `feedback_falsifiable_hypothesis_before_structural_fix.md`.**

---

## What surprised us

- **`CLAUDE_PROMPT` env does NOT exist in `claude -p` mode.** Session-3 handover §3.3 confidently proposed Option A as the smallest-change form, gating role detection on `[[ "$CLAUDE_PROMPT" =~ \[role: ]]`. The env var simply doesn't exist — Claude Code's `-p` mode is env-minimal by design (only `CLAUDE_PROJECT_DIR` + plugin paths). The handover never tested the assumption, so the patch encoded it and shipped broken. Smoke #448 was the first reality check (sentinel row 533 landed via HTTP MCP, but no queue file, no role-signal sibling, no signal anywhere). The third claude-code-guide query confirmed the docs say the prompt is ONLY available via the `transcript_path` JSONL.

- **Stop hooks DO fire in `-p` mode but events are invisible.** Per `--include-hook-events` flag docs, hook events are suppressed from the JSONL stream-json log by default. The Junior daemon doesn't pass the flag, so worker logs show zero `"hook_event":"Stop"` entries despite hooks actually running. Side effects (file writes, DB inserts) still happen — absence of hook events in the log is NOT evidence the hook didn't fire. Initially mis-diagnosed the smoke #448 failure as "hook didn't fire" before realizing it fired but exited early on the empty `CLAUDE_PROMPT`.

- **EliteDesk's `/home/barrie/MCPs/project-memory-mcp/` is NOT a git checkout.** Session-3 handover §3.3 step 1 confidently said "`ssh homeserver "cd /home/barrie/MCPs/project-memory-mcp && git pull"`" — that immediately exited 128 with "not a git repository". The directory is a scp'd deployment from March. Took ~3 min to diagnose + decide on scp-instead-of-clone approach. The handover didn't verify the deploy shape.

- **Drain script's rsync doesn't exist on Git-Bash Windows.** Wrote drain script using rsync (handover §3.3 phrasing said "rsync queue file back"); failed instantly with `rsync: command not found` on first invocation. Should have caught this at write-time given the existing `pattern_cross_platform_divergences.md`. Wasted ~5 min on commit + push of the rsync version before realizing.

- **Architecture mismatch surfaced via `AskUserQuestion` on EliteDesk write path.** Realized mid-flight that `write-role-signal.js` opens local SQLite via `better-sqlite3` and can't write to a remote DB. The HTTP MCP transport (T1b's whole purpose) is for MCP-tool calls, not for shell-side CLIs. This forced a sub-decision: queue+drain pipeline vs add HTTP transport to the CLI. Pleasant surprise: the queue+drain pipeline is genuinely lossless and async — better than a synchronous HTTP CLI would have been.

- **End-to-end verification was decisive once it ran.** Smoke #449 took 28s, both rows landed, drain script ingested correctly. The 5-min verification loop made fix #2 trustable in a way that fix #1 (which the laptop-side `bash -x` test passed) was not.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When a handover proposes a "smallest change" gated on an untested system property (env var existence, CLI behavior, file presence on a remote machine), verify the assumption BEFORE writing the patch. One curl/ssh/grep beats a smoke task that ships in 25 min wasted. Promote to lesson: `feedback_handover_assumptions_need_empirical_verification.md` (cross-harness). | Catches "fix #1 was wrong-shaped" before commit; saves ~25 min/incident | minor (add a §2 verification line to handover-advisor template) | 2× (this session = fix #1, also `feedback_falsifiable_hypothesis_before_structural_fix.md` DQ #338) |
| 2 | Add a row to `pattern_cross_platform_divergences.md` noting Git-Bash on Windows lacks rsync — use scp for cross-machine sync scripts. Reference the drain script (`scripts/brehon/drain-role-signal-queue.sh`) as the canonical pattern. | Prevents the next sync-script author from writing rsync; the existing pattern doc covered other divergences but missed this one. | minor (pattern-file edit) | 1× this session, 0× confirmed prior — borderline (still worth promoting because the pattern doc exists for exactly this purpose) |
| 3 | Add `bash scripts/brehon/drain-role-signal-queue.sh` as a pre-step in any future `/check-role-health` skill body (T4a from handover §4.1). Without it, the analysis is only as fresh as the last manual drain — defeats the point of the queue. Also wire it as Step ~2c in weekly-review (T4b). | T4a consumer always has freshest Junior signals; weekly-review automatically catches accumulated queue drift. | minor (drain script ship is done; T4a/T4b authoring still pending) | 1× anticipated (drain pattern is new) |
| 4 | When debugging a hook in `claude -p` mode, ALWAYS check side effects (queue files, DB rows) before assuming the hook didn't fire. Add a note to a new lesson `feedback_stop_hook_fires_silent_in_p_mode.md` if recurrence hits 2x. | Stop the "hook never fired → diagnose-from-logs → reach wrong conclusion" loop. | minor (lesson at second occurrence) | 1× this session — single-instance, WATCH for second occurrence |
| 5 | Update the role-signal hook header comment + `.claude/PRPs/handovers/role-customization-2026-05-24-session4.md` §2.2 are already in place — but the lesson note for `feedback_claude_p_mode_env_minimal.md` should be authored next session at second occurrence (after T4a authoring may surface another env-related surprise). | Codifies the "no `CLAUDE_PROMPT` env in -p mode" trap for future hook authors. | minor | 1× this session — WATCH |

## What to carry forward

- **The `claude-code-guide` agent is an excellent docs-truth-checker.** Used 2× this session (Stop hook fire behavior, `CLAUDE_PROMPT` env existence) — both queries returned cited, actionable answers in <30s that decisively settled wrong assumptions. Net positive in both invocations.
- **`AskUserQuestion` on architectural forks > guessing.** Used 5× this session — branch field source, deploy method, write path, push gate, hook revision plan. All produced clean decisions with no wasted work downstream. Architectural ambiguity surfaced to user, mechanical work done by advisor.
- **Smoke task as integration test.** Smoke #448 + #449 were the deciding signal — wall-clock cost ~30 sec each, but the failure of #448 was the catch that prevented a "looks-working-locally" patch from being declared done. Adopt as pattern: when shipping infrastructure that interacts with Junior daemon, the first verification MUST be a Junior task, not a laptop-side `bash -x`.
- **Multi-lane forward-merge discipline kept rt-r3 in sync.** rt-r3 was cut at start of session; every governance-v0 commit got fast-forward merged + pushed within minutes. No conflicts, no drift.
- **Cross-machine canonical PMD with HTTP MCP + drain pipeline works.** This session shipped the missing piece (Stop hook fix) that lets EliteDesk Junior workers contribute to canonical laptop PMD via TWO paths: (a) MCP tool calls via HTTP transport (synchronous), (b) hook-side queue + drain (async). Architecture is sound — neither path is a hack.
- **Atomic read-fetch-mutate-commit-push on canonical checkout per `multi-lane-worktree.md` Hard refusal #6.** Used every push this session. Zero collisions despite the concurrent `6cdee0caa` federation-inbound-e brief authoring session running in parallel.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `claude-code-guide` agent (Stop hook fire in -p mode) | 15 | 0 | medium | confirmed hooks fire silently; ruled out "hook didn't fire" wrong-path diagnosis |
| `claude-code-guide` agent (CLAUDE_PROMPT env existence) | 20 | 0 | high | overturned core handover assumption that fix #1 was built on |
| Smoke task #448 (under fix #1) | 0 | 25 | high | sentinel landed but no role-signal; revealed env-gate was always-empty |
| Smoke task #449 (under fix #2) | proves fix end-to-end | 0 | low | both rows landed exactly as designed |
| `AskUserQuestion` ×5 (architectural forks) | 14 | 0 | none | all clean decisions; architecture decisions surfaced to user, mechanical work done by advisor |
| `ToolSearch` for junior-brehon schemas | 1 | 0 | none | clean |
| Drain script (rsync version, committed in `feaa75db9`) | 0 | 5 | medium | rsync doesn't exist on Git-Bash Windows; should have caught at write-time |
| Drain script (scp fix in `59d3b3414`) | works end-to-end | 0 | none | clean Windows-compatible recipe |
| Hook fix #1 (`feaa75db9` — drop branch gate, derive WORKER_DIR) | partial (WORKER_DIR resolution IS correct) | 0 | none | necessary but not sufficient; the env-gate replacement was the wrong path |
| Hook fix #2 (`729b13312` — parse transcript_path) | full | 0 | low | matched session-3 handover Option B verbatim |
| Session-4 handover write | 30+ next-session | 0 | none | self-contained per `.claude/rules/handover.md` |

## Complexity scores (heavy tasks only)

No heavy Junior tasks this session — only two ~28s smoke tasks, well under the threshold.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Smoke task #448 (smoke-v1) | 0 | 0 | 1 | 0 |
| Smoke task #449 (smoke-v2) | 0 | 0 | 1 | 0 |
| Advisor-side hook fix iteration (3 commits, multi-machine) | 4 | 3 | 90 | n/a (interactive) |

No watchdog risk. The advisor-side iteration's 90-min wall-clock was the bulk; well-distributed across debug → assumption-overturn → fix → verify.

## Decisions to revisit

- **Should the drain script run on a cron schedule** (handover §4.5 option c)? Current proposal is to invoke from `/check-role-health` (T4a) + weekly-review (T4b) only — pull-on-demand. May need a periodic drain (~15 min) if Junior dispatch volume rises and `role-signal-queue.jsonl` grows fast. Worth a clarify once T4a ships and we have a real signal-rate.
- **Should the role-signal hook also fire for non-bm-task roles** (impl-task, planning, ci-watcher)? The transcript-parsing fix supports all four role tags, but verification only ran for bm-task. First impl-task or planning Junior task under `729b13312` will reveal whether WORKER_DIR resolution still works correctly (transcript_path lives under per-task project dir for those roles too — confirmed mechanically, not empirically).
- **The `rules_read` + `mcp_tools_invoked` content fields came out as `[]`** in row 535. Need a longer-running worker (impl-task or planning) to confirm the jq parsing works on a richer transcript. Worth verifying before T4a authoring.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Change #1** (handover assumptions need empirical verification): promote to `.claude/lessons/feedback_handover_assumptions_need_empirical_verification.md` — cross-harness; 2× recurrence with existing `feedback_falsifiable_hypothesis_before_structural_fix.md`. Meets threshold.
- [ ] **Change #2** (Git-Bash on Windows lacks rsync): add a row to existing `.claude/lessons/pattern_cross_platform_divergences.md` (do not create a new lesson — fold into pattern).
- [ ] **Change #3** (drain script as pre-step for T4a/T4b): update `.claude/skills/weekly-review/SKILL.md` Step 2c (when authored) AND author drain pre-step into T4a skill body (which doesn't exist yet — author at T4a creation time).
- [ ] **Change #4** (Stop hook side-effects in -p mode): WATCH for second occurrence before promoting to `feedback_stop_hook_fires_silent_in_p_mode.md`.
- [ ] **Change #5** (no CLAUDE_PROMPT env in -p mode): WATCH for second occurrence before promoting to `feedback_claude_p_mode_env_minimal.md`.
- [ ] PMD eval write — appropriate (recurrence ≥ 2 on Change #1).

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
