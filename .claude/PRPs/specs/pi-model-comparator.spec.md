---
title: Pi-based multi-role model comparator — architecture spec
status: draft (design task #6)
authored: 2026-06-07
authored_by: advisor (canonical brehon-fork session)
supersedes: the impl-task-only MiniMax A/B trial (minimax-m27-trial-1.md) — generalised
run_location: EliteDesk (homeserver) — matches the "Pi on EliteDesk" user story
---

# Pi-based multi-role model comparator

## 0. One-paragraph statement

A **controlled experiment harness** that runs any model, in any development role
(planning / impl / bug-fix / review), under **identical conditions** inside the
Brehon harness — holding everything constant except the model — captures each
arm's trace + produced artifact, scores them against a per-role rubric, and emits
a **routing recommendation**: keep the role on its current (runner, model) pair,
or cut it over to a challenger. Built on the existing `.pi/` harness (multi-provider,
model-swappable cells already exist). The first experiment is *planning: Opus 4.8
max vs GPT-5.5 xhigh*. The driving user story: *"if MiniMax M3 is as good at
planning, all future planning tasks route through Pi Coding + MiniMax M3 on
EliteDesk."*

## 1. Why a comparator (not a one-off A/B)

The original ask was "Opus vs GPT-5.5 for planning." First principles reframed it:
the real object of value is a **reusable, model-agnostic environment** where any
candidate model can be measured against the incumbent for any role, with no
leakage (each arm sees exactly the same environment). The one-off A/B is just the
first run inside it.

Why the existing Junior path can't *host the challenger*: it is Claude-only
(`claude -p` + daemon `--model` injection). Every non-Claude arm needs bespoke
glue (the MiniMax SSH script, a proposed Codex wrapper). Pi is the clean way to
run a non-Claude challenger inside a harness.

**Framing nuance (per user 2026-06-07):** the comparison is deliberately
*runner-vs-runner* — control = Claude-Code+Opus (the production incumbent, tested
as-is), challenger = Pi+model. So it is NOT a single-variable "same environment,
swap only the model" experiment — model AND runner/harness vary together. That is
correct here because the *decision* is whether to switch the whole pipeline, not
to compare models in the abstract (see §2). The "no leakage" discipline still
applies *within* each runner: identical task input, the same dual-harness-mirrored
planning spec, the same base commit, the same cargo tooling — so any quality delta
is attributable to the (model+runner) bundle, not to one arm getting a richer
brief than the other.

Pi solves this natively: `@earendil-works/pi-ai` is a unified multi-provider API
(OpenAI, Anthropic, Google, openai-codex). A single Pi cell runs any provider by
swapping one field. The `.pi/` harness in this repo already proves the pattern
(`bm-pi.md: model: claude-haiku-4-5`, `settings.json: defaultModel: gpt-5.5`).

## 2. The arms — runner-vs-runner, not model-vs-model (per user, 2026-06-07)

**The Opus arm stays on Claude Code.** The incumbent planning runner is tested
*as it actually runs in production*: Opus 4.8 max via the existing Junior
`[role:planning]` path with the full `.claude/` harness. It is NOT re-run through
Pi. Running "Opus-in-Pi" would measure a configuration that doesn't exist in
production and never will — a dishonest baseline.

So the comparison is **runner-vs-runner**, and each arm differs in *two* bundled
variables (model AND runner/harness):

| Arm | Runner | Harness | Model | Dispatch |
|---|---|---|---|---|
| **control (incumbent)** | Claude Code / Junior | `.claude/` (auto-loaded CLAUDE.md, rules, lessons) | Opus 4.8, effort max | `mcp__junior-brehon__create_task("[role:planning] …")` |
| **challenger** | Pi (on EliteDesk) | `.pi/` (`PROJECT_CONTEXT.md` + `prp-plan.md`) | GPT-5.5 xhigh / MiniMax M3 / … | `run-comparator.sh` → headless Pi |

**Why bundling the runner difference is correct here** (it's not a clean
single-variable experiment): the decision is not "is GPT-5.5 better than Opus in
the abstract" — it is "should the whole planning pipeline switch from
Claude-Code+Opus to Pi+challenger." The runner/harness IS part of what's being
decided, so it belongs in the comparison. The user story confirms this framing:
*"stay with Pi Coding… all future planning routes through Pi."*

### What is held constant (to keep the comparison meaningful despite two runners)

| Held constant | How |
|---|---|
| Task input | the same brief / phase-ref, byte-identical to both runners |
| Role spec | `.pi/prompts/prp-plan.md` ⇔ `.claude/commands/prp-core/prp-plan.md` are the SAME planning spec, dual-harness-mirrored (the dual-harness contract keeps section headings stable) — verify they have not drifted before each experiment |
| Base commit | both fork from the same SHA |
| Cargo tooling | shared `scripts/brehon/cargo-*.sh` wrappers + `.claude/skills` (both harnesses reuse them) |
| Thinking budget | each runner at its max analogue (see §4) |
| Output target | a `plan.md` in the canonical 20-section schema |

### What varies (bundled, by design)

Model + provider + runner + harness flavour. The challenger cell is realised as a
Pi agent definition (mirror of `.pi/agents/bm-pi.md`) whose `model:`/`provider:`
line is the arm-specific field — OR a single def + a per-run override flag (build
task #5 confirms which Pi headless supports). The control cell is just the
existing Junior dispatch — no new plumbing.

## 3. Experiment config schema

One experiment = one role + one task input + N model arms. Proposed config
(`/.claude/PRPs/comparator/<experiment-id>.json`):

```json
{
  "experiment_id": "planning-opus-vs-gpt55-001",
  "role": "planning",
  "task_input": ".claude/PRPs/briefs/<the-real-planning-brief>.md",
  "base_commit": "<sha both arms fork from>",
  "arms": [
    {
      "id": "control", "is_incumbent": true,
      "runner": "claude-code",            // existing Junior [role:planning] path — NOT Pi
      "harness": ".claude/",
      "prompt": ".claude/commands/prp-core/prp-plan.md",
      "model": "claude-opus-4-8", "thinking": "max",
      "dispatch": "mcp__junior-brehon__create_task(\"[role:planning] …\")"
    },
    {
      "id": "challenger",
      "runner": "pi",                      // Pi on EliteDesk
      "harness": ".pi/",
      "prompt": ".pi/prompts/prp-plan.md",
      "provider": "openai", "model": "gpt-5.5", "thinking": "xhigh",
      "dispatch": "run-comparator.sh → headless Pi"
    }
  ],
  "rubric": "planning",
  "ships": "control"                       // control plan is canonical; challenger is compare-only, never merges
}
```

Note the asymmetry is intentional (per §2): control = Claude Code runner (no new
plumbing — it's the existing production path), challenger = Pi runner. Both read
the dual-harness-mirrored planning spec. Open/extensible: a MiniMax arm is
`{ "runner": "pi", "provider": "<minimax-pi-provider>", "model": "minimax-m3", … }`
once Pi's MiniMax provider + key are wired. Gemini likewise — challengers are
always Pi-side; the Claude-Code+Opus control is the fixed incumbent.

## 4. Thinking-budget parity (the fairness lever)

The incumbent planning role runs Opus at `effort: max` (`.pi`/`.claude`
planning.md frontmatter). To avoid handicapping either arm, the challenger runs
at the closest analogue:

| Arm | Knob | Value |
|---|---|---|
| Opus 4.8 (control) | Anthropic effort | max |
| GPT-5.5 (challenger) | `reasoning_effort` | xhigh |
| MiniMax M3 (future) | provider thinking | highest available |

Recorded per arm so the comparison report states the budget each arm got.

## 5. Isolation + sandboxing (mandatory — Pi has NO permission system)

Sandboxing applies asymmetrically — the two runners isolate differently:

- **Control (Claude Code):** already sandboxed by the Junior daemon — each
  `[role:planning]` task runs in its own `.junior/worktrees/job-<id>/` worktree,
  guarded by the (Claude-shaped) `worktree-guard.sh`. No new sandboxing needed;
  the control arm uses the production path verbatim.
- **Challenger (Pi):** Pi's docs are explicit — *"Pi does not include a built-in
  permission system for restricting filesystem, process, network, or credential
  access."* And `worktree-guard.sh` is Claude-Code-shaped (PreToolUse, `claude`
  event format) — it does NOT constrain Pi runs. So the **Pi arm's** sandboxing is
  the comparator's responsibility:

1. **One throwaway git worktree for the Pi arm**, off `base_commit`, named
   `ab-cell/<experiment-id>-challenger`. Created by `run-comparator.sh`, removed
   after capture. Never merged into any phase or trunk branch.
2. **EliteDesk coexistence:** the daemon's main worktree (`/srv/brehon-fork`) is
   live on a phase branch with active Junior workers; all share one `.git/`. The
   comparator must respect the **cross-lane cap** (max 2 concurrent worktree-
   writing agents, per `feedback_cohort_shared_git_index_contention.md`). Run
   arms **serially** by default (also avoids `.git/index.lock` contention) — the
   existing trial reached the same conclusion.
3. **Auto-commit hook:** `.pi/extensions/lemmy-hooks.ts` auto-commits every Pi
   edit (`auto(pi): update <basename>`). Fine for capturing the produced plan
   (each arm's worktree gets its own commit trail = the trace), but the runner
   should NOT let those commits reach a shared branch. Per-arm worktree isolates
   them.
4. **Sibling-worktree placement:** sandboxes live OUTSIDE
   `/srv/brehon-fork/.junior/worktrees/` so they don't trip the (Claude-side)
   guard or get reaped by the daemon's cancel handler.

## 6. Trace + output capture (per arm)

Two artifacts per arm, written to `.claude/PRPs/comparator/runs/<experiment-id>/<arm-id>/`:

| Artifact | What | Source |
|---|---|---|
| **trace** | the agent's full tool-call / reasoning transcript | Pi session log (headless run stdout/jsonl) + the worktree's `auto(pi)` commit trail |
| **output** | the produced artifact | the role's deliverable: `plan.md` (planning), `git diff` (impl/bugfix), findings (review) |
| **metrics** | token in/out, $cost, wall-clock, thinking budget | parsed from Pi run summary |

`observation-capture.sh` is a model — but it's Claude-Code-event-shaped; the
comparator needs a Pi-side capture (Pi run already emits a session log; the
runner tees it). No LLM in the capture path — pure observation.

## 7. Per-role scoring rubrics

Scoring is the science. Each role has objective + judged dimensions. The judged
dimensions are scored by a **separate neutral judge agent** (a third model, or
the advisor) reading both outputs blind to which arm produced which — never an arm
scoring itself.

### Planning rubric
| Dimension | Type | How measured |
|---|---|---|
| Plan completeness | objective | all 20 canonical §-sections present (`grep '^## '`) |
| Watchpoint specificity | objective | every §4 watchpoint cites a specific table/file/line (the cite-a-table gate) |
| ADR-constraint preservation | objective | ADR gates named in the brief appear in §13 with callsite + rationale (not just named) |
| §13 task decomposition | judged | atomic, dependency-ordered, MIRROR-ref'd tasks |
| §16a story coverage | objective | stories present + checkpoint commands executable |
| DoD smoke-test pass | objective | run every §15 command literally against HEAD — exit codes |
| Clarify-DQ quality | judged | did it surface real ambiguities vs invent noise |
| Token / $ cost | objective | from metrics |

### Impl rubric
cargo-first-pass (did validate-pending pass on first push) · DQ-blocker rate ·
diff quality vs MIRROR ref · commit-shape compliance · scope discipline · cost.
(Reuses the existing minimax-m27 rubric verbatim.)

### Bug-fix rubric
repro reproduced · fix resolves repro · no regression (test suite) · scope
(minimal diff) · root-cause vs symptom · cost.

### Review rubric
true-positive findings · false-positive rate · severity calibration · actionable
fixes · cost.

## 8. Decision rule → routing recommendation

The output is NOT just a score — it is a **routing decision** per the user story.

| Outcome (over n paired tasks) | Recommendation |
|---|---|
| Challenger ≥ incumbent on all objective gates AND judged-parity | **Cut over**: route this role to Pi + challenger model |
| Challenger worse on any hard gate (ADR-preservation, DoD-pass, cargo-first-pass) | **Stay** on incumbent |
| Mixed | Extend to larger n, or **task-shape-scoped** routing (e.g. challenger for low-complexity plans only) |

Statistical discipline (from the existing trial): paired data, n=5 default before
a binary call, conservative default = stay. Per-role, not global.

## 9. Cutover mechanics (the production flip — separate gated step)

If the recommendation is "cut over," adopting (Pi, challenger-model) as the
production runner for a role is a **separate, gated, reversible** change — never
done inside an experiment run:

1. **Routing flip:** the advisor's role-dispatch path changes from
   `mcp__junior-brehon__create_task("[role:<X>] …")` (Junior/`claude -p`) to a
   Pi-cell dispatch on EliteDesk. Recorded in `advisor-orchestrator.md` role
   table + the per-role model assignment lesson.
2. **Where recorded:** `v1-roadmap.json` (or a routing config) names the
   (runner, model) pair per role; a memory note captures the decision + evidence.
3. **Reversible:** the flip is a config change; reverting = point the role back at
   Junior+incumbent. The experiment artifacts are the audit trail.
4. **Scope it as its own sub-phase** (analogue of the planned `v1-minimax-cutover`):
   daemon/role-map edits, restore-script updates, `feedback_brehon_anthropic_only.md`
   reconciliation if a non-Anthropic model becomes a production runner.

## 10. EliteDesk provisioning (ground truth 2026-06-07)

| Need | Status | Action |
|---|---|---|
| Pi installed | ❌ not installed (node+npm present) | **user-local** install (npm prefix `~/.npm-global`, NO sudo — avoids PMD #998 sudo-on-daemon hazard): `npm i -g --ignore-scripts @earendil-works/pi-coding-agent` (canonical pkg per pi.dev; NOT `@mariozechner` author-scope, NOT macOS/homebrew). v0.78.1. |
| `subagent` tool | ❌ user-scope install needed | Linux npm-global symlink path (PROJECT_CONTEXT.md shows homebrew — adapt). Only needed if comparator cells dispatch Pi subagents; the planning cell may not. |
| OpenAI auth (challenger) | ⏳ needs interactive `/login` | **Codex OAuth (ChatGPT Plus/Pro)** per user 2026-06-07 — `pi` → `/login` → Codex on EliteDesk (interactive, user does it). Token → `~/.pi/agent/auth.json`, auto-refreshes. **NO `sk-proj` API key needed — sidesteps the chat-pasted-key rotation entirely** (pasted key stays dead). Matches the subscription cost model the "Pi-on-EliteDesk production" user story implies. |
| Anthropic auth (control) | ✅ OAuth (`~/.claude/.credentials.json`) drives the existing `claude -p` Junior path | **No action** — Opus arm stays on Claude Code (per user 2026-06-07); it never touches Pi, so no `ANTHROPIC_*` env needed for the comparator. (Note: Pi *also* supports Claude Pro/Max OAuth if a future Claude-on-Pi arm is wanted.) |
| MiniMax auth (future challenger) | ❌ suspended | Pi natively supports `MINIMAX_API_KEY` → provider `minimax` (no custom plumbing — confirms abandoning the old Junior SSH script was right). Re-provision the key when that arm runs. |

**Pi auth model (from EliteDesk `~/.npm-global/lib/.../docs/providers.md`):** subscriptions via `/login` (Codex / Claude Pro-Max / Copilot) store auto-refreshing tokens in `~/.pi/agent/auth.json`; API-key providers via env var or `auth.json`. Resolution order favours explicit `--api-key` > env > `auth.json`.
| Worktree base | ✅ `/srv/brehon-fork` on phase-m2-late-1, daemon live | sandboxes are siblings; serial runs; respect cross-lane cap |

## 11. Open questions for the build (task #5)

1. ~~Pi headless invocation~~ — **RESOLVED 2026-06-07 (pi.dev):** four modes —
   `pi -p "query"` (print) and `--mode json` (event stream) are the headless ones
   the runner uses. Model config via `models.json` (API keys / OAuth / custom).
   Confirm exact prompt+model+cwd flag shape with `pi --help` post-install.
2. **Per-run model override vs per-arm def** — pi.dev shows in-session `/model`
   switching + `models.json` config; confirm the *headless* `-p` flag for
   model/provider selection so one `comparator-<role>.md` + a `--model` flag can
   serve all arms (vs one def per arm).
3. ~~Anthropic OAuth-token extraction for the Opus arm~~ — **RESOLVED 2026-06-07:
   the Opus arm stays on Claude Code (existing Junior path, OAuth), never Pi. No
   `ANTHROPIC_*` env wiring needed.** Cost-fairness note: control rides the Claude
   subscription (OAuth), challenger pays OpenAI API $ — the comparison must report
   cost in comparable terms (e.g. $-equivalent or note the subscription/API
   asymmetry explicitly).
4. **GPT-5.5 provider choice** — `openai` (API key) vs `openai-codex` (Codex
   OAuth, the `.pi` default). API-key path is simpler for scripted dispatch.
5. **Pi-side sandbox enforcement** — Pi has no permission system; is throwaway-
   worktree isolation sufficient, or do we add a Pi extension equivalent to
   `worktree-guard.sh`?

## 12. What exists vs what to build (summary)

**Exists (reuse):** multi-provider Pi runtime; per-agent model slot; self-contained
`prp-plan.md` (+ impl/debug/issue-fix prompts); `PROJECT_CONTEXT.md` harness-as-
config; shared `.claude/skills` + cargo wrappers; subagent dispatch pattern;
auto-commit trace trail.

**Build (the comparator machinery):** comparator agent def(s) per role; the
`run-comparator.sh` runner (worktree-per-arm sandbox + headless Pi dispatch +
capture); experiment config schema; per-role rubric scoring + neutral-judge step;
comparison-report + routing-recommendation generator; EliteDesk provisioning
(Pi install + keys); the cutover sub-phase template.
