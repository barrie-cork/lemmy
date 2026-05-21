# Session retro — 2026-05-22 — dq338-rca-revision-option-b-ship

**Harness:** claude-code
**Session window:** 2026-05-21T20:00Z → 2026-05-22T00:45Z (~285 min)
**Branch at start:** `75b437b54` (`governance-v0`)
**Branch at end:** `2484a9a10` (`governance-v0`)
**Files touched:** 8 (own commits) — see Step 1
**Commits:** 3 (auto: 0, explicit: 3 — `cf93b7ba6`, `8a392f217`, `2484a9a10`)

## TL;DR

The session opened with a `/clear`-inherited recommendation ("plan RT-r2 now") and a `UserPromptSubmit` hook reporting `DQ pending: 1 [#326]`. Instead of authoring RT-r2, the session burned all ~285 min on three nested investigations of a single defect: DQ #338 — the destructive `git reset --hard origin/phase-v1-federation-inbound-c` that orphaned plan-merge `2ad835aa4`. The first investigation falsified the DQ's own hypothesis (daemon-finalize bug); the second confirmed the correct vector (lane-agent ssh-reset against shared daemon checkout); the third shipped option-b (`PreToolUse` hook refusing the bug pattern). Highest-load-bearing finding: **structural-fix DQs anchor sessions into multi-hour investigations when the hypothesis is wrong** — a 30-min falsification gate before option-pick (the `feedback_falsifiable_hypothesis_before_structural_fix` lesson this session promoted) would have saved most of the displacement and bound the work better. Cross-cutting finding: shared `.git/` across multiple lane worktrees produced a near-attribution-collision (my staged work shipped under the other session's commit subject `c858aa7ab`).

---

## What surprised us

### Advisor

- **The DQ I had every reason to trust was authored on a wrong premise.** DQ #338's `context` field confidently named "daemon's finalize step issued an unintended reset" — I and a prior session both took it at face value. The user's option-a pick (investigate daemon executor.ts) inherited the same wrong premise. Only by doing the actual `grep -rn 'reset --hard' /opt/junior-src/` did the falsification surface. **The DQ is not a contract; it's a hypothesis.**
- **Auto-mode does not auto-bypass per-action user confirmations.** Even with "Auto Mode Active", `AskUserQuestion` still triggered three times mid-session (DQ #338 sequencing, revised RCA action, option-b decision). The auto-mode reminder shifts my bias toward continuing, but does not remove the gates — and shouldn't, given each was a judgment-heavy decision. The model was correct; the surprise was its calibration.
- **Cross-session commit-attribution collision is a real failure mode of `git worktree`.** `c858aa7ab chore(advisor): v1-dq-schema-r1 task 4 — four-role retro` shipped my hook + my rule edit + my checklist edit, none of which had anything to do with the retro that titled the commit. Two sessions sharing `/srv/brehon-fork/.git/` race on the shared index. The mitigation (per the new lesson) is mechanical: `git status` between `git add` and `git commit`. The surprise was that *no hook fires* on this defect class — it's a silent attribution breach.

### Planning

(N/A — no planning brief authored this session. The original ask was RT-r2 planning; it was deferred and never resumed.)

### Impl

- **Sub-agent forensics returned POSITIVE evidence, not just falsification.** The `general-purpose` sub-agent found a smoking-gun transcript path (`a6097edd-663b-4770-b73a-816a012b6d67.jsonl` uuid `cf9c79f1-a709-41ea-a920-dc643a2dd0a5` ts 19:58:17Z, 9 sec before the destructive reset). I expected the sub-agent to confirm "not daemon" by absence of evidence; instead it inverted the recommendation by *presence* of evidence. Time-budget for sub-agent: ~5 min. Net saved: would have been at least 60 min of laptop-side transcript spelunking I'd have been bad at (Windows path handling, jsonl parsing, ts-window correlation across multiple worktree transcripts).

### BM

(N/A — this session did not run a BM verb. No `bm-cut`/`bm-pr`/`bm-merge`/`bm-poll-cr`. The concurrent fed-in-c session handled all BM work for v1-dq-schema-r1.)

---

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Promote `feedback_falsifiable_hypothesis_before_structural_fix.md` into `.claude/rules/advisor-orchestrator.md` §5.4 (DQ triage decision tree).** Add a new sub-bullet: *"For DQs proposing a structural fix that names a specific code path (file/function/prompt) as the defect site, run a ≤30-min falsification pass (`grep` the named code path; read the relevant subagent's task log; check `who`/`last`) BEFORE picking option-a/b/c. If the named code path does not contain the suspect operation, surface the falsification to the user before structural work starts."* | Caps structural-fix-on-wrong-premise displacement (~3h saved per recurrence). | minor (a paragraph edit) | 1× this session, 0× prior — promotion candidate, not auto-promote |
| 2 | **Author a `feedback_session_start_inherited_context_anchors_action.md` lesson + advisor-orchestrator §1 cross-reference.** Inherited stdout from a `/clear`-cleared turn (the prior session's "plan RT-r2 now" recommendation) framed this session into action before the multi-lane CWD check ran. Surface-first ritual edit (shipped in `e6035f5e1` by the other session — independent of mine — also addresses this; the new lesson explicitly cites the failure mode and the ritual). | Reduces "inherited-context override session-start ritual" defect class. | minor | 1× this session (combined with prior surface-first edit in `e6035f5e1`, this is recurrence ≥2 across sessions) |
| 3 | **Generalise `refuse-ssh-reset-hard-shared-checkout.sh` to refuse ANY `ssh ...` writing to `/srv/brehon-fork` outside an allow-list pattern.** Current hook is narrow (catches one bug class). A broader hook that refuses `ssh.*homeserver.*/srv/brehon-fork.*git (reset|push --force|update-ref refs/heads/.*\s+\S{1,7}\b)` would catch the next bug class before the third occurrence. | Defends against the next still-unknown daemon-side defect class with one structural change. | medium (more regex testing; risk of false-positive friction on legitimate routines) | Speculative; do not ship until next occurrence demonstrates need (per `feedback_principles_not_rules.md` — grow allowlist on retro evidence) |
| 4 | **Add a PreToolUse hook on `Bash` matching `git commit` that compares `git diff --cached --name-only` against a session-scoped "intended files" list** (written at first `git add` time, reset at commit). Refuses commits whose staged file list contains entries unrelated to the session's expected scope. | Structurally prevents the cross-session attribution collision (`c858aa7ab` defect class). | medium (per-session state file; risk of false-positives on legitimate multi-file commits) | 1× this session — promotion candidate, not auto-ship; defer until 2nd occurrence |
| 5 | **Run `harness-audit` skill against `.claude/hooks/` to produce a complete inventory + latency + redundancy report.** The dir now has 15 scripts; README documents 6. Drift is real. The `watch_hook_dir_audit_pending.md` PMD memory is the pointer. | One-shot reduction of hook-bloat surface area; identifies WARN-not-FAIL hooks that could merge into one dispatcher. | medium (~30 min skill run) | Latent; trigger condition is the next session that touches `.claude/hooks/` |

## What to carry forward

- **`git status` between `git add` and `git commit` as a mandatory mitigation against attribution collisions.** Used in the final commit `2484a9a10`; confirmed only my 5 intended files were staged before commit fired. Cheap belt-and-braces; should become reflex on every commit that follows a `git add`.
- **Sub-agent dispatch for forensic investigations against large external artifacts** (~5 min, ~1 KB synthesis returned; ~60 min of my context preserved). The `general-purpose` sub-agent type with a self-contained prompt + explicit "what to investigate, what to report, in under 500 words" framing. Worked on first invocation.
- **Falsification-before-fix gate (the new lesson).** The 30-min falsification pass found the wrong premise; without it the option-a investigation would have cost much more before discovering the wrong code surface.
- **WARN-not-FAIL vs BLOCK-on-defect-class hook calibration.** This session shipped two new hooks: `session-start-multi-lane-check.sh` (WARN, exit 0; mirrors `pmd-canonical-guard.sh`) and `refuse-ssh-reset-hard-shared-checkout.sh` (BLOCK, exit 2; mirrors `check-cargo-pipe.sh`). The calibration test is "is the safer alternative mechanical?" If yes (e.g. `update-ref` vs `reset --hard`), BLOCK is correct. If no (e.g. `.mcp.json` config drift the user must hand-fix), WARN is correct.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `general-purpose` sub-agent (DQ #338 RCA forensics) | ~60 | 0 | high | Returned smoking-gun transcript citation; inverted the recommendation; ~5 min sub-agent runtime. Would have been ≥60 min inline given Windows path / jsonl parsing burden. |
| `AskUserQuestion` (DQ #338 v1 sequencing) | 5 | 0 | none | Clean three-way fork; user picked option-a. Mid-investigation re-decision (option-a falsified) also used AskUserQuestion — clean. |
| `AskUserQuestion` (DQ #338 v2 stabilise vs continue) | 5 | 0 | none | User picked "STOP and stabilize"; then re-decided to end session after I surfaced the multi-session reality. |
| `AskUserQuestion` (final option-b pick) | 3 | 0 | none | Single binary question; immediate answer. |
| `Bash` (manual investigations into daemon /opt/junior-src) | 30 | 30 | medium | Reproduced prior session's forensics independently (per the new lesson's "verify, don't trust" rule). Wasted ~30 min reading executor.ts + git.ts + buildFinalizePrompt before I realised the prior session had already done this thoroughly — should have read the existing `feedback_falsifiable_hypothesis_before_structural_fix.md` lesson FIRST. |
| `Write` (new lesson + memory files) | — | — | none | 4 new memory files + 1 rewritten lesson. Took ~25 min total; would have been ~10 min if I'd dictated them in one pass instead of authoring + revising. |
| `Edit` (`advisor-orchestrator.md` §1 surface-first ritual) | — | — | none | Edit displaced by concurrent session's `e6035f5e1` commit (which had its own surface-first edits). My edit shipped via attribution-collision under `c858aa7ab`. Net: change landed, but not under my commit. |
| `mcp__junior-brehon__show_task` | 2 | 0 | none | Confirmed Task 401 watchdog-killed (exit 143); Task 400 done; Task 402 done. |
| Cross-session attribution-collision mitigation (final commit) | 5 | 0 | none | `git status` between add/commit confirmed only my files staged; no surprise repeat. |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| DQ #338 investigation cycle (full session) | 8 | 3 | ~285 | n/a (interactive, not Junior) |
| Sub-agent forensic dispatch (within session) | 0 (read-only) | 0 | ~5 | n/a |

Flag against thresholds (>55min runtime, >40min log silence, >8 files):
- Whole session: 285 min on a single DQ → above the watchdog envelope, but I'm not a Junior subagent; this is just the session length. Flag retroactively: **the original task scope (RT-r2 planning) was abandoned in favour of an emergent investigation that should arguably have been its own scheduled session.** Carry-forward §"What to change" #1 addresses this (falsification gate).

## Decisions to revisit

- **Should the falsifiable-hypothesis lesson be promoted to a `.claude/rules/` rule** (always-loaded) rather than a lesson (loaded on PMD search hit)? The pattern is structural — any structural-fix DQ needs the gate. Worth a separate clarify pass.
- **RT-r2 planning brief is still un-authored.** This was the session's original purpose; defer to a fresh session that opens with the multi-lane CWD check ritual surfaced first.
- **Whether the cross-session attribution collision lesson (#4 above) should ship a hook now or wait.** Recurrence is 1; principle says defer. But the cost of an attribution breach (audit-trail wrong) is high; the cost of a false-positive on the hook is low (refuse and re-stage). Worth a clarify pass on threshold revision for high-blast-radius defect classes.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1: promote `feedback_falsifiable_hypothesis_before_structural_fix.md` to a `.claude/rules/advisor-orchestrator.md` §5.4 sub-bullet (cross-harness rule, always-loaded).
- [ ] Change #2: author `feedback_session_start_inherited_context_anchors_action.md` lesson + cross-reference from advisor-orchestrator §1 surface-first ritual. Recurrence: this session + the `e6035f5e1` concurrent-session ritual edit independently identifies the same defect class → ≥2 across sessions.
- [ ] Change #5: dispatch `harness-audit` skill against `.claude/hooks/` (recurrence: README drift documented + watch memory exists; trigger condition met).
- [ ] Change #4: defer commit-collision hook until second occurrence (per principle).
- [ ] Change #3: defer broader ssh-refusal hook until next occurrence (per principle).

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
