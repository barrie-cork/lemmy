# Session retro — 2026-06-04 — v1-closeout plan + isolated-lane bootstrap

**Harness:** claude-code
**Session window:** 2026-06-04 ~12:50 → ~14:30 UTC (~100 min)
**Branch at start:** `10201285c` (`governance-v0`)
**Branch at end:** `83694fed2` (`governance-v0`) + `0f9cdb407` (`phase-v1-closeout`)
**Files touched:** 6 (1 tracked plan, 1 handover, 1 plan-mode scratch, 3 gitignored harness-wiring copies) + 1 worktree created
**Commits:** 2 (auto: 0, explicit: 2)

## TL;DR

User asked for a "post-shipping major phase close-out plan" for v1 cleanup/refactor, then steered the shape across **seven mid-work messages** (isolate from M1 → optimise for `/workflows` → make it hybrid → check the CC guide → user-will-execute → full-harness → "is it a good approach?"). The session produced an approved 8-phase close-out plan (phased by *conflict-risk* against the live M1 lane, with hybrid `workflow`/`four-role`/`manual` execution-lane tags) and fully bootstrapped an isolated `brehon-fork-closeout` worktree (PMD HTTP MCP + 3 hooks verified) ready for the user to drive. **Most load-bearing finding:** incremental mid-work steering on a planning task is *high-value* (each message sharpened the plan) but the right response is to **fold-and-continue, not restart** — I absorbed all seven without re-deriving, because the recon was front-loaded and the plan file was the single mutable surface. **Top change proposal:** the AskUserQuestion-then-execute-then-AskUserQuestion cadence cost two extra round-trips that a single up-front 3-question batch (scope + isolation + deliverable, which I *did* run first) could have anticipated by also asking "who executes" and "/workflows?" — but those two only emerged from the user's own messages, so the lesson is narrower: *when a request says "create a plan," proactively ask the execution-ownership + execution-tooling questions in the first batch.*

---

## What surprised us

- **Seven separate user messages arrived mid-work**, each a genuine refinement rather than a correction: "separate worktree in isolation" → "optimise for /workflows" → "perhaps hybrid" → "check claude code guide" → "I will continue in a different worktree" → "set up with full harness" → "is it a good approach?" → "so it is in isolation." None contradicted prior ones; they *layered*. Surprising in a good way — the user was co-designing in real time, and the plan got materially better for it (conflict-risk phasing, hybrid lanes, full-harness handoff all came from this).
- **The `/workflows` mechanics had a load-bearing constraint I would have gotten subtly wrong without the guide check:** the workflow *script itself* has no file/shell access — only the subagents it spawns do — and a workflow **cannot pause for mid-run user input** (only permission prompts). That directly determined the hybrid boundary: anything needing a CR-triage / merge-confirm / ADR-risk gate *cannot* live inside a workflow. Verifying against `claude-code-guide` before baking assumptions paid off.
- **The orphaned worktree dirs were all `.git`-less and empty** (0–1 entries, 8–64 KB), and `git worktree prune` was already clean — so they're pure filesystem leftovers, NOT registered worktrees. Inspecting before assuming saved a wrong `git worktree remove` recommendation; they need plain `rm -rf` (with inspect-then-confirm) instead.
- **The live Dependabot banner (19 vulns: 2 critical/1 high/11 moderate/5 low) exactly matched the plan's deps-r2 (6) + deps-r3 (13) split** — an unplanned cross-check that confirmed the plan's security numbers were live-accurate, not stale from the brief.
- **PMD topology surprise (benign):** the bootstrap checklist still says "verify `PROJECT_MEMORY_DB` shows the canonical path," but under the 2026-05-30+ HTTP-daemon topology there is no such env line — the `.mcp.json` uses `http://localhost:11435/mcp` and the absence is *correct*. The checklist step 5 wording is stale relative to the current topology (noted, not yet fixed).

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When a user request is "create a plan / design X," the **first AskUserQuestion batch should include execution-ownership ("who runs it — you or me?") and execution-tooling ("normal pipeline, /workflows, or hybrid?")** alongside scope/isolation/deliverable. | Anticipates the 2 extra round-trips this session spent on "I'll execute in another worktree" + "optimise for /workflows" — both arrived as separate mid-work messages. | minor (add 1–2 options to an existing question batch) | 1× this session + 1× prior (the "local vs dispatch" Phase-2 gate is the same class — execution-ownership surfaced late) → meets ≥2 promotion threshold |
| 2 | **Update bootstrap-checklist step 5** (`feedback_phase_lane_worktree_bootstrap_checklist.md`) to state that under the HTTP-daemon topology (2026-05-30+) the PMD verification is `grep '"url".*11435' .mcp.json` (HTTP endpoint), NOT `grep PROJECT_MEMORY_DB` — and that the absence of a `PROJECT_MEMORY_DB` line is expected, not a failure. | Stops the next lane-bootstrap operator from chasing a phantom "missing PMD path" finding (I hit this exact false-alarm this session). | minor (one paragraph edit) | 1× this session + the topology note already exists in `pmd-invariants.md` #1 → ≥2 |
| 3 | Add a one-line **"who executes this plan" field to the plan-mode plan template header** (advisor-drives vs user-drives vs four-role-pipeline). | Makes the handoff model explicit at plan-write time instead of emerging from mid-work clarification; the handover doc then derives from it mechanically. | minor | 1× this session (single-instance — recorded, proposed at lesson-tier only if it recurs) |

## What to carry forward

- **Fold-and-continue over restart for mid-work steering.** Seven layering messages were absorbed without re-deriving any prior work because (a) reconnaissance was front-loaded into the first ~10 tool calls, and (b) the plan file was the single mutable surface — each refinement was an edit, not a rewrite. This is the right default when steering *layers* rather than *contradicts*. (Used cleanly 7× this session.)
- **Verify framework/tool mechanics against the authoritative guide before baking them into a plan.** The `claude-code-guide` dispatch corrected the workflow file-access + no-mid-run-gate model that the hybrid design hinged on. Cheap insurance (~65s subagent) for a plan whose core was the execution-lane split.
- **Inspect-before-assume on filesystem cleanup.** Classifying the 6 orphaned dirs (`.git` presence + entry count + `git worktree prune --dry-run`) before recommending an action distinguished "registered worktree → `git worktree remove`" from "loose dir → `rm -rf`." Per `no-destructive-defaults.md`; held.
- **Front-load parallel reconnaissance.** Two background Explore agents (carry-patch inventory + deps-r2/r3 scope) ran concurrently with the plan drafting, returning structured tables that dropped straight into Phase 3 and Phase 4. Net-positive: the plan's two thinnest areas got hard numbers without blocking the write.
- **Surface-first lane status** when ≥2 worktrees are active (stated `lanes: …closeout:phase-v1-closeout active; other-active: …validate:phase-m1-b (M1, live)` in the handover + responses). Per `advisor-orchestrator.md` surface-first ritual; correct call given the live M1 lane.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `claude-code-guide` subagent (/workflows mechanics) | 20 | 0 | medium | corrected the script-has-no-file-access + no-mid-run-gate model; the API-primitives detail was unavailable in docs but the constraints I needed were confirmed. Directly shaped the hybrid boundary. |
| Explore ×2 (carry-patch + deps scope, parallel bg) | 25 | 0 | low | both returned clean structured tables; folded verbatim into Phase 3/4. Parallel dispatch kept them off the critical path. |
| AskUserQuestion ×3 batches | 12 | 6 | low | first batch (scope/isolation/deliverable) was high-value; the 2nd (deps/wasmtime/promote) + 3rd (handoff-depth/PMD) were necessary but partly forced by mid-work messages that a fuller first batch could have pre-empted (→ "What to change" #1). |
| EnterPlanMode / ExitPlanMode | 8 | 0 | none | clean gate; plan approved first pass, no rework loop. |
| Bash recon batches (worktree/DQ/M1-touch-set) | 30 | 2 | none | front-loaded; the 2 wasted min were a too-narrow grep for SessionStart hooks that returned empty (file had content) — re-ran wider. |
| Worktree bootstrap + 3-hook verification | 15 | 0 | low (PMD-path stale wording) | submodule init + wiring copy + programmatic hook checks all passed; the only friction was the stale `PROJECT_MEMORY_DB` checklist wording (→ "What to change" #2). |
| TaskCreate/Update (4 tasks) | 2 | 1 | none | lightweight tracking; one stale-id update ("Task not found") cost a beat. |

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. No Junior impl-tasks ran this session (manual advisor session). The one heavy advisor task:

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Plan authoring + Phase-0 bootstrap | 6 | 2 | ~100 | n/a (interactive, no Junior log) |

No flags (interactive session; the >55min runtime is wall-clock-with-user-in-loop, not unattended worker time — the watchdog-envelope concern doesn't apply).

## Decisions to revisit

- **The `/schedule` watch on extism PR #847** (Phase 5 deps-r3) was left as a ready-to-run step for the user's close-out session rather than created here — correct (it's user-gated + belongs in that session), but worth confirming it actually gets set so the deferred wasmtime fix isn't forgotten.
- **Phases 6–8 are authored-but-blocked behind M1 merge.** The gate check (`git log origin/phase-m1-b ^origin/governance-v0` must be empty) is documented in both plan + handover; revisit when M1 ships to un-block the e2e split + type-state retrofit.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Change #1** (execution-ownership + tooling in first plan-question batch): promote to `.claude/lessons/feedback_plan_request_ask_execution_ownership_first.md` (cross-harness lesson). Basis: 1× this session + the Phase-2 local-vs-dispatch late-surfacing class (prior).
- [ ] **Change #2** (bootstrap-checklist HTTP-PMD verification wording): update `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` step 5 directly (it's a checklist, not a lesson-to-promote — just fix the stale step).
- [ ] **Change #3** (plan-template "who executes" header field): single-instance — record only; promote if it recurs.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase section omitted —
no `/auto-phase` invocation or auto-state mutation this session (leftover
`m1-b.json` / `v1-RT-r3.json` are prior-session artifacts; trigger did not fire)._
