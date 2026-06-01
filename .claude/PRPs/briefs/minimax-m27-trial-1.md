# Trial runbook: MiniMax M2.7 vs Sonnet 4.6 on impl-task — ARMED, fires on next qualifying phase

> **Not a Junior dispatch brief.** This is an advisor-session runbook for an A/B
> trial, not a `[role:*]` task. It tells the advisor how to run a 5-task
> MiniMax-M2.7-vs-Sonnet-4.6 bake-off on throwaway branches, measure it, and
> decide whether to switch the impl-task role to MiniMax.
> Canonical-schema reference: there is no prior trial-runbook sibling; shape
> borrows §2/§3/§4 ordering from `.claude/PRPs/briefs/v1-RT-r3-planning-1.md`.

## 0. Status (2026-05-31 — ARMED, rolling trigger)

**ARMED — fires automatically on the next qualifying impl-task dispatch (rolling trigger, 2026-05-31).**

The trial no longer targets a single named phase. Instead it fires on the **first phase that has ≥5 qualifying impl-tasks** (§0.1 below). The advisor applies the trial criteria at brief-authorship time for every `[role:impl-task]` dispatch going forward; e2e tasks and judgment-heavy tasks self-exclude.

### 0.1 Qualifying task criteria (advisor checks at brief-authorship time)

A task qualifies for a MiniMax arm iff ALL of:
1. `[role:impl-task]` dispatch (not planning, bm, ci-watcher)
2. NOT an e2e task — brief slug does NOT contain `e2e` and the task's primary file is NOT `crates/server/tests/e2e.rs`
3. MIRROR-ref-heavy — plan §13 cites a specific `MIRROR:` line range in an existing file
4. ≤2 files modified (single/two-file handler shape)
5. Cargo-gated — plan task DoD names `cargo-check` or `cargo-clippy` as the validate step

When ≥5 qualifying tasks accumulate in a phase, run both arms. Fewer than 5 → accrue to the next phase (do not run a partial trial; insufficient n).

### 0.2 Precondition checklist (all green as of 2026-05-31)

| Precondition | State |
|---|---|
| Daemon `envOverrides` plumbing (`executor.ts:313`) | ✅ present; verified 2026-05-29 |
| `junior task add --env-override KEY=VALUE` | ✅ CLI v1.0.2 |
| `scripts/brehon/queue-minimax-task.sh` | ✅ ready; `MINIMAX_MODEL` default `MiniMax-M2.7` |
| MiniMax endpoint (`https://api.minimax.io/anthropic/v1/messages`) | ✅ live; HTTP 200 verified 2026-05-31 |
| `MINIMAX_API_KEY` in `.env` | ✅ funded (HTTP 200, 2026-05-31) |
| Target sub-phase | **rolling** — first phase with ≥5 qualifying tasks |
| Trial history | v1-RT-r4 NOT RUN (serial dispatch forced); v1-RT-r5 NOT RUN (only 1 qualifying task — Task 5 is e2e, excluded) |

**Decisive new evidence (2026-05-29) that this is worth running:** first-party
benchmark chart at `.claude/PRPs/reports/image.png` shows M2.7 lands at
near-Sonnet-parity on impl-shaped benchmarks (SWE-Bench-Pro: Sonnet 57.2 vs
M2.7 56.2; M2.7 ahead on Multi-SWE 52.7 vs 50.3 + Toolathlon 46.3 vs 44.8) at
**~10× cheaper** ($0.30/$1.20 per M vs Sonnet ~$3/$15). The trial answers the
one thing the chart cannot: does that parity hold **inside our harness** (MIRROR
refs, cargo gate, our DQ-blocker rate) on **Rust**, not SWE-bench Python.

**Why M2.7 and not M2.5:** the same chart shows M2.7 ≥ M2.5 on every panel at
identical price (MLE-Bench-lite 66.6 vs 51.5; GDPval-AA 50 vs 35). M2.5 is not
an arm. A dedicated M2.5-vs-M2.7 bake-off is a *contingent follow-up* only —
run it iff M2.7 clears the Sonnet bar AND you specifically distrust the
vendor-reported chart. (`MINIMAX_MODEL=MiniMax-M2.5 queue-minimax-task.sh ...`)

## 1. The question this trial gates

Binary: **is MiniMax M2.7 good enough to replace Sonnet 4.6 on the impl-task
role?** NOT "which MiniMax is better" (chart settles that). The cutover is
worth ~10× on the highest-token-volume Junior role; the risk is recovery
cycles from worse code. The trial measures whether the cheap model holds
quality on *our* Rust + MIRROR-ref + cargo-gated workflow.

**Scope of any eventual cutover = impl-task ONLY.** Not BM (Haiku, already
cheapest, runs `gh pr merge` — too much risk for trivial saving), not planning
(Opus, plan quality load-bearing), not advisor, not ci-watcher (negligible
volume). impl-task is the only role that is simultaneously high-token-volume +
pattern-following-from-MIRROR-refs (one valid output shape) + cargo-gated
(objective pass/fail catches degradation). That triad is why it's the only
candidate.

## 2. Trial mechanics

### 2.1 Arms

- **Control arm (Sonnet 4.6):** a *normal* `[role:impl-task]` dispatch via
  `mcp__junior-brehon__create_task`. The daemon's role-prefix patch pins
  `claude-sonnet-4-6`. We already have months of real Sonnet impl data, so the
  control baseline is essentially free — but run it on the SAME 5 designated
  tasks (on `ab-test/sonnet-<N>` branches) so the comparison is task-matched,
  not historical-average.
- **MiniMax arm (M2.7):** `scripts/brehon/queue-minimax-task.sh "<dispatch line>"`
  from a branch matching `ab-test/*`. The script injects the three env
  overrides; `executor.ts:313` makes them win over the Sonnet role-pin.

### 2.2 Hard collision-mitigation rules (from `project_minimax_ab_trial_deferred.md`)

1. Run on throwaway `ab-test/minimax-impl-<N>` + `ab-test/sonnet-impl-<N>`
   branches, branched from the v1-RT-r4 phase tip. **NEVER merge an ab-test
   branch into the phase branch or trunk.** The real v1-RT-r4 impl is a
   separate, normal dispatch that actually ships.
2. Run A/B tasks **serially**, never in parallel with the real impl task for
   the same work (avoids `.git/index.lock` contention + double-spend confusion).
3. Only designate tasks that are **pattern-following from MIRROR refs** — one
   valid output shape. Judgment-heavy tasks make the model variable
   un-isolable. (v1-RT-r4 sponsor-allowlist handlers are ideal: 3 prior shipped
   RT sub-phases to mirror admin-handler + capability-check + ENTRY_KIND-emit
   patterns from.)
4. `create_task` (MCP) does NOT pass `env_override` — the MiniMax arm MUST go
   via the SSH CLI (the script). Only the control arm uses MCP `create_task`.

### 2.3 Per-task dispatch sequence

For each of the 5 designated tasks N:
1. `git checkout -b ab-test/sonnet-impl-<N> <phase-tip>` + push; dispatch
   Sonnet control via `create_task` with `base_branch=ab-test/sonnet-impl-<N>`.
2. Wait for it to reach `complete`/`failed`. Record metrics (§2.4).
3. `git checkout -b ab-test/minimax-impl-<N> <phase-tip>` + push; dispatch
   MiniMax arm via `queue-minimax-task.sh` (it reads `JUNIOR_BASE_BRANCH` or
   defaults to `ab-test/minimax-trial` — set `JUNIOR_BASE_BRANCH=ab-test/minimax-impl-<N>`).
4. Wait; record metrics.
5. Both branches stay unmerged. The REAL task N ships via the normal v1-RT-r4
   pipeline (could reuse whichever arm's output is sound, but default: re-run
   clean on the phase branch so provenance is unambiguous).

### 2.4 Metrics per task (the comparison table)

Record into a results file `.claude/PRPs/reports/minimax-m27-trial-results.md`
(create at trial start):

| Metric | How measured | Proxy for |
|---|---|---|
| DQ-pending blockers raised | count `kind:"blocker"` entries `from:"impl"` on the arm's branch | reasoning quality |
| Cargo first-attempt pass/fail | did the arm's `validate-pending-laptop` pass on first push? (advisor runs the gate locally — Shape G suspended until 2026-06-01) | code correctness |
| Commit-shape compliance | `feat(scope): <title> (task N)`? HANDOVER trailer if `[P]`? | follows convention |
| Wall-clock | log mtime delta (task spawn → complete) | speed |
| (qualitative) diff quality | advisor eyeballs the diff vs the MIRROR ref | over/under-engineering |

### 2.5 Decision rule

- M2.7 **matches** Sonnet on cargo-first-pass AND DQ-blocker rate across the 5
  tasks → **switch impl-task to MiniMax M2.7** (see §4 for the permanent-cutover
  mechanics). The ~10× saving justifies it.
- M2.7 **raises** the DQ-blocker rate or fails the cargo gate more often →
  **stay Sonnet.** Recovery cycles cost more than the token saving buys.
- Mixed/ambiguous at n=5 → extend to n=8-10 before deciding, OR stay Sonnet
  (conservative default). Do NOT switch on a coin-flip.

## 3. Task designation (rolling — auto-populated by advisor at plan approval)

The advisor runs §3.5a of `.claude/rules/advisor-orchestrator.md` at every plan approval (after the watchpoint gate, before surfacing to user). That step walks the §13 task list against §0.1 criteria and appends rows here. The running ✅ count is reported in the plan-approval surface as `MiniMax trial: N/5 qualifying tasks accumulated`.

**Do not fill this table manually** — the §3.5a step owns it.

| Phase | Task | File(s) | MIRROR ref | Qualifies? | Notes |
|---|---|---|---|---|---|
| v1-RT-r4 | Tasks 1–5 | see original designation 2026-05-29 | various | ✅ all 5 | NOT RUN — serial dispatch forced |
| v1-RT-r5 | Task 5 | `e2e.rs` | — | ❌ | e2e excluded |

**Running total: 0 ✅ qualifying** (RT-r4 tasks were designated but not run; count resets to 0 for the rolling trigger going forward).

When cumulative ✅ count reaches ≥5 → proceed to §2.3 dispatch sequence alongside the real phase.

Results file: `.claude/PRPs/reports/minimax-m27-trial-results.md` (pre-exists; append a new `##` section per phase run).

**Do NOT let the trial gate any phase's shipping.** Trial runs ALONGSIDE the real phase on throwaway `ab-test/*` branches; real tasks dispatch normally.

## 4. If the trial passes — permanent cutover mechanics (NOT part of the trial)

The A/B trial uses env-override (wins over the role-pin). A **permanent** switch
of impl-task to MiniMax means editing the daemon patch, not just frontmatter:

- The impl-task model is injected by `/opt/junior-src/src/daemon/executor.ts`
  (`extractRoleModel(job.prompt)` → `childEnv.ANTHROPIC_MODEL = roleModel`)
  detecting the `[role:impl-task]` prefix. To make MiniMax the default, the
  role→model map (and `ANTHROPIC_BASE_URL` + `ANTHROPIC_AUTH_TOKEN`) for the
  impl role must change there.
- Mirror the change to `homeserver/scripts/junior-server-patches/` + the
  restore script (`restore-junior-server-patches.sh`) so an upstream pull
  doesn't revert it (per `.claude/agents/impl-task.md` "Model enforcement").
- Update `.claude/agents/impl-task.md` frontmatter `model:` + the enforcement
  note, AND `feedback_brehon_subagent_model_effort_assignments.md`, AND flip
  `feedback_brehon_anthropic_only.md` from "Anthropic-only" to "Anthropic +
  MiniMax-for-impl" (else a future advisor treats live MiniMax as drift).
- This is a follow-up sub-phase of its own (`v1-minimax-cutover` or similar) —
  scope it then; do NOT bundle into v1-RT-r4.

## 5. Required reading (for the advisor running the trial)

- `project_minimax_ab_trial_deferred.md` (PMD) — original trial design + the
  collision-mitigation hard rules this runbook operationalises.
- `.claude/PRPs/reports/image.png` — the M2.7-vs-M2.5-vs-Sonnet benchmark chart
  that decided M2.7-not-M2.5 + parity-not-superiority expectation.
- `feedback_brehon_subagent_model_effort_assignments.md` — per-role tiering +
  the verification-metrics list (DQ-rate-by-failure-class, cache hit rate).
- `feedback_brehon_anthropic_only.md` — the standing "Anthropic-only" decision
  this trial explicitly revisits (per its own "don't propose without revisit"
  clause). Flip it on cutover.
- `.claude/agents/planning.md` §5 + §5b — the non-Sonnet `target_model`
  thresholds (split-DQ at `score > 6`; per-task ceiling ≤3 files / ≤1 crate /
  dedicated e2e task). v1-RT-r4's plan must set `target_model: minimax-m2.7`
  IF the trial tasks are planned as MiniMax targets — but for the trial, the
  real plan stays Sonnet-targeted and the trial re-runs the same tasks under
  MiniMax separately.
- `scripts/brehon/queue-minimax-task.sh` — read the header; it has the
  pre-flight curl probe + the model-override usage.

## 6. Constraints (hard refusals for the advisor)

- **NEVER merge an `ab-test/*` branch.** Trial output is measurement, not ship.
- **NEVER put the key in any file I author.** It lives in `.env` only; the
  script reads it at runtime + redacts it from all output.
- **NEVER run the MiniMax arm via MCP `create_task`** (it drops env_override) —
  always the SSH CLI script.
- **NEVER block v1-RT-r4 shipping on trial completion** — trial is parallel,
  optional, and abortable.
- **NEVER switch impl-task to MiniMax mid-phase** — cutover is its own
  follow-up sub-phase after the 5-task data is in + reviewed.
- **Forbidden-window awareness:** the MiniMax arm runs cargo-class work on the
  EliteDesk? NO — under Shape-G-suspended, cargo runs on the laptop advisor
  session for the validate gate. But the Junior impl-task arm itself runs on
  the daemon (no cargo, just edits + commit), so forbidden windows apply only
  to the advisor's local validate run, not the dispatch. Per
  `advisor-orchestrator.md` §5.1.
