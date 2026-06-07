# Trial runbook: MiniMax M3 vs Sonnet 4.6 on impl-task — ARMED, fires on next qualifying phase

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
| `scripts/brehon/queue-minimax-task.sh` | ✅ ready; `MINIMAX_MODEL` default updated to `MiniMax-M3` (2026-06-07) |
| MiniMax endpoint (`https://api.minimax.io/anthropic/v1/messages`) | ✅ live; HTTP 200 verified 2026-05-31 (M2.7); M3 endpoint needs pre-flight verify |
| `MINIMAX_API_KEY` in `.env` | ⚠️ key present but needs rotation (see project_minimax_key_rotate_after_m1b_trial.md); M3 pre-flight verify needed before first arm |
| Target sub-phase | **rolling** — first phase with ≥5 qualifying tasks |
| Trial history | v1-RT-r4 NOT RUN (serial dispatch forced); v1-RT-r5 NOT RUN (only 1 qualifying task — Task 5 is e2e, excluded) |

**Decisive new evidence (2026-05-29) that this is worth running:** first-party
benchmark chart at `.claude/PRPs/reports/image.png` shows M2.7 lands at
near-Sonnet-parity on impl-shaped benchmarks (SWE-Bench-Pro: Sonnet 57.2 vs
M2.7 56.2; M2.7 ahead on Multi-SWE 52.7 vs 50.3 + Toolathlon 46.3 vs 44.8) at
**~10× cheaper** ($0.30/$1.20 per M vs Sonnet ~$3/$15). The trial answers the
one thing the chart cannot: does that parity hold **inside our harness** (MIRROR
refs, cargo gate, our DQ-blocker rate) on **Rust**, not SWE-bench Python.

**Why M3 and not M2.7:** upgraded per user instruction 2026-06-07; M3 is the
current flagship model per the MiniMax platform docs (`MiniMax-M3` model ID).
M2.7 remains available as a fallback override (`MINIMAX_MODEL=MiniMax-M2.7`).

## 1. The question this trial gates

Binary: **is MiniMax M3 good enough to replace Sonnet 4.6 on the impl-task
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
- **MiniMax arm (M3):** `scripts/brehon/queue-minimax-task.sh "<dispatch line>"`
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

### 2.3a MiniMax-arm prompt preamble (mandatory — prepend to every MiniMax-arm dispatch)

The MiniMax arm's task description (and any wrapper system prompt) MUST prepend the harness-tuned form of MiniMax's own best practices (full set: `reference_minimax_prompting_best_practices.md`). The non-negotiable three, ordered by the failure they prevent:

1. **"Explain WHY each constraint matters."** For any ADR/governance constraint in the brief, the dispatch states the rationale, not just the rule — MiniMax's #1 best practice, and the direct fix for the m1-b task-4 ADR-015 drop (`.claude/PRPs/reports/minimax-ab-m1b-task3-task4.md`). A constraint with no rationale is one the arm may trade away.
2. **"You may refuse / raise a blocker — do NOT silently scope out a constraint you cannot satisfy."** Explicit permission to refuse + DQ-blocker path instead of rationalising a gate to a later task.
3. **"The task is at the END of this prompt; context and constraints come first."** Long-context ordering — MiniMax's largest single quality lever.

Plus: labelled sections (Task/Context/Constraints/Output), cite the MIRROR ref as a concrete example (examples > rules), concrete output contract (the exact fn/struct shape expected). The §2.4a-style ADR clause (`advisor-orchestrator.md`) is doubly mandatory for the arm. Per `feedback_cheap_model_arm_drops_adr_constraints.md`.

### 2.4 Metrics per task (the comparison table)

Record into a results file `.claude/PRPs/reports/minimax-m3-trial-results.md`
(create at trial start):

| Metric | How measured | Proxy for |
|---|---|---|
| DQ-pending blockers raised | count `kind:"blocker"` entries `from:"impl"` on the arm's branch | reasoning quality |
| Cargo first-attempt pass/fail | did the arm's `validate-pending-laptop` pass on first push? (advisor runs the gate locally — Shape G suspended until 2026-06-01) | code correctness |
| Commit-shape compliance | `feat(scope): <title> (task N)`? HANDOVER trailer if `[P]`? | follows convention |
| Wall-clock | log mtime delta (task spawn → complete) | speed |
| (qualitative) diff quality | advisor eyeballs the diff vs the MIRROR ref | over/under-engineering |

### 2.5 Decision rule

- M3 **matches** Sonnet on cargo-first-pass AND DQ-blocker rate across the 5
  tasks → **switch impl-task to MiniMax M3** (see §4 for the permanent-cutover
  mechanics). The ~10× saving justifies it.
- M3 **raises** the DQ-blocker rate or fails the cargo gate more often →
  **stay Sonnet.** Recovery cycles cost more than the token saving buys.
- Mixed/ambiguous at n=5 → extend to n=8-10 before deciding, OR stay Sonnet
  (conservative default). Do NOT switch on a coin-flip.

**Cutover is task-shape-scoped, not all-or-nothing (revised 2026-06-07 per m1-b AB data).**
The m1-b A/B pair (n=2; `.claude/PRPs/reports/minimax-ab-m1b-task3-task4.md`) showed the
result is NOT binary: MiniMax reached parity on a pure pattern-follow DTO task (task 3) but
dropped a hard ADR-015 gate + used clippy-denied `.expect()` on a handler-with-embedded-
constraint task (task 4). So the decision rule is two-tier:
- **Safe MiniMax lane:** pure pattern-following — DTOs, boilerplate, MIRROR-ref-heavy, NO
  embedded ADR/governance logic. Switch these even on the current thin evidence.
- **Sonnet-or-hardened lane:** any handler that enforces an ADR gate or carries non-mechanical
  logic. Requires §2.3a preamble + §2.4a ADR clause + the §4b-adr verify gate — or just stay
  Sonnet. Do NOT route these to MiniMax on cost alone.

### 2.6 M2.7 vs M3 selection on EliteDesk (open — needs trialing)

`queue-minimax-task.sh` defaults to `MiniMax-M3` (2026-06-07); `MINIMAX_MODEL=MiniMax-M2.7`
is the fallback override. We do NOT yet have a calibrated threshold for *when* to prefer one
over the other on the daemon — the official docs (`reference_minimax_prompting_best_practices.md`)
do not version-differentiate. Candidate axes a future trial should measure (extend the §2.4
table with a `model_version` column and run M2.7 + M3 arms side-by-side on the SAME tasks):
- **Task complexity / token volume:** is M3's extra capability load-bearing only above some
  plan §5 complexity score, or on multi-file (>2) tasks? (M2.7 may suffice — and be cheaper/
  faster — for the simplest DTO lane.)
- **Constraint-following:** does M3 hold the ADR-015-class gate that M2.7 (or this M-arm)
  dropped? Re-run the task-4 shape against both versions.
- **Latency / EliteDesk resource cost:** wall-clock + peak RAM per version on the daemon —
  M3 may be slower; if M2.7 matches quality on the simple lane, prefer it there.
- **Cost delta:** confirm the per-M token pricing of M3 vs M2.7 (the ~10×-vs-Sonnet figure
  is for M2.7; M3 pricing must be re-checked before a cost-based default).
Until trialed, default `MiniMax-M3` for arms; treat the version choice as an explicit,
recordable decision per dispatch, not a silent default. Further trialing is expected and fine.

## 3. Task designation (rolling — auto-populated by advisor at plan approval)

The advisor runs §3.5a of `.claude/rules/advisor-orchestrator.md` at every plan approval (after the watchpoint gate, before surfacing to user). That step walks the §13 task list against §0.1 criteria and appends rows here. The running ✅ count is reported in the plan-approval surface as `MiniMax trial: N/5 qualifying tasks accumulated`.

**Do not fill this table manually** — the §3.5a step owns it.

| Phase | Task | File(s) | MIRROR ref | Qualifies? | Notes |
|---|---|---|---|---|---|
| v1-RT-r4 | Tasks 1–5 | see original designation 2026-05-29 | various | ✅ all 5 | NOT RUN — serial dispatch forced |
| v1-RT-r5 | Task 5 | `e2e.rs` | — | ❌ | e2e excluded |
| m1-b | Task 1 | `migrations/.../up.sql`+`down.sql` | `add_governance_config/up.sql` | ❌ | criterion 5: DoD is `migrate-roundtrip.sh`, not cargo-check/clippy |
| m1-b | Task 2 | model+mod.rs+newtypes.rs+schema.rs (4 files) | `governance_config.rs:21-60`; schema.rs:513/1487/1601 | ❌ | criterion 4: 4 files > 2 |
| m1-b | Task 3 | `api_common/src/governance.rs` (1 file) | `governance.rs:449-498` | ✅ | DTOs; single-file, cargo-gated |
| m1-b | Task 4 | `messaging_config.rs`+`mod.rs` (2 files) | `admin_config.rs:383/682/747` | ✅ | admin handler; single-write |
| m1-b | Task 5 | `messaging_config.rs`+`routes/lib.rs` (2 files) | `admin_config.rs:426`; `routes/lib.rs:478-512` | ✅ | validator + route reg |
| m1-b | Task 6 | `bridge_notify.rs`+`lib.rs`+`notify.rs` (3 files) | `notify.rs:305`; `plugins.rs:48` | ❌ | criterion 4: 3 files > 2 |
| m1-b | Task 7 | `e2e.rs` | `e2e.rs:6235/6504` | ❌ | criterion 2: e2e |
| m2-core-hook | Task 1 | `api_common/src/governance.rs` (1 file) | `db_views/site/src/api.rs:762-770` (tagged enum) | ✅ | DTO; single-file, cargo-gated |
| m2-core-hook | Task 2 | `bridge_notify.rs` (1 file) | `bridge_notify.rs:13-49` | ✅ | refactor PM path onto tagged union |
| m2-core-hook | Task 3 | `bridge_notify.rs` (1 file) | `bridge_notify.rs:13-49` | ✅ | add `governance_case_after_transition` sibling fn |
| m2-core-hook | Task 4 | db_schema log + api shim + registry doc (3 files) | `governance_log.rs:220-234` | ❌ | criterion 4: 3 files > 2 |
| m2-core-hook | Task 5 | `api/governance/governance_log.rs` (1 file) | `inbox.rs:721`; `governance_log.rs:255-314` | ✅ | `append_room_event` wrapper |
| m2-core-hook | Task 6 | 5 handler files | `notify.rs:285-306` | ❌ | criterion 4: 5 files > 2 |
| m2-core-hook | Task 7 | `appeal_window_expiry.rs`+`sponsor_liability_grace.rs` (2 files) | `notify.rs:285-306` + commit-boundary notes | ✅ | cron-site hook wiring |
| m2-core-hook | Task 8 | `e2e/governance.rs` | — | ❌ | criterion 2: e2e |
| m2-rooms-a | Task 1 | bridge_room.rs+config.rs+Cargo.toml+main.rs (4 files) | `config.rs:9-44` | ❌ | criterion 4: 4 files > 2 |
| m2-rooms-a | Task 2 | room_provisioner.rs+appservice.rs+main.rs (3 files) | `appservice.rs:144-191` | ❌ | criterion 4: 3 files > 2 |
| m2-rooms-a | Task 3 | room_provisioner.rs (1 file) | `appservice.rs:144-158`; `messaging_config.rs:69-83` | ✅ | single-file extend, MIRROR-heavy |
| m2-rooms-a | Task 4a | bridge_auth.rs+room_event_handler.rs+mod.rs+routes/lib.rs (4 files) | `messaging_config.rs:160-183`; `appservice.rs:52-90` | ❌ | criterion 4: 4 files > 2 |
| m2-rooms-a | Task 4b | bridge_read.rs+mod.rs+lib.rs (3 files) | `messaging_config.rs:160-183` | ❌ | criterion 4: 3 files > 2 |
| m2-rooms-a | Task 5 | room_provisioner.rs+soft_pause.rs (2 files) | `soft_pause.rs:34-41`; `bridge_notify.rs:38-45` | ✅ | 2-file, MIRROR-heavy, bearer-add |
| m2-rooms-a | Task 6 | room_provisioning.rs+dm_round_trip.rs (2 files) | `dm_round_trip.rs` `#[ignore]` pattern | ✅ | test-compile gated; `cargo test --no-run` |
| m2-late | Task 1 | 4 files (migration up/down + enums.rs + schema.rs) | `enums.rs:531-547`; `schema.rs:136-138` | ❌ | criterion 4: 4 files > 2 |
| m2-late | Task 2 | 4 files (sanction_event.rs + sanction_subscriber.rs + mod.rs + newtypes.rs) | `sanction.rs:1-48` | ❌ | criterion 4: 4 files > 2 |
| m2-late | Task 3 | 4 files (governance_log.rs + api shim + sanction_kind_map.rs + api mod.rs) | `governance_log.rs:117` | ❌ | criterion 4: 4 files > 2 |
| m2-late | Task 4 | 3 files (sanction_publisher.rs + api mod.rs + 1 create) | `bridge_auth.rs:6-18`; `governance_log.rs` append sig | ❌ | criterion 4: 3 files > 2 |
| m2-late | Task 5 | `submit_jury_vote.rs` (1 file) | `submit_jury_vote.rs:163-178`; `:452-480` | ✅ | 1-file, MIRROR-heavy, cargo-gated |
| m2-late | Task 6 | `crates/server/src/lib.rs` (1 file) | `lib.rs:340-410` | ✅ | 1-file, MIRROR-heavy, cargo-gated |
| m2-late | Task 7 | 3 files (sanction_handler.rs + appservice.rs + main.rs) | `room_provisioner.rs`; `appservice.rs::router()` | ❌ | criterion 4: 3 files; criterion 5: bridge in-task cargo only |
| m2-late | Task 8 | e2e/m2_late.rs + e2e.rs (2 files) | `e2e.rs:124-148` | ❌ | criterion 2: e2e |

**Running total: 13 ✅ qualifying** (m1-b 3,4,5 = 3; m2-core-hook 1,2,3,5,7 = 5; m2-rooms-a 3,5,6 = 3; m2-late T5,T6 = 2). Trial SUSPENDED — see MEMORY.md "MiniMax trial SUSPENDED (memory 827)"; do NOT dispatch arms until infra investigated and user resumes.

**TRIAL FIRES THIS PHASE — user override 2026-06-04** (`feedback_minimax_trial_run_below_threshold_on_user_override.md`): user waived the ≥5 cumulative gate (*"Run trial for eligble tasks, even if below 5/5… We need to get AB tests result"*). The 3 eligible tasks (m1-b 3,4,5) run BOTH arms per §2.3 on throwaway `ab-test/m1-b-t{3,4,5}-{sonnet,minimax}` branches off `phase-m1-b` AFTER real Tasks 1+2 land (Task 3 `requires:2`, 4 `requires:2,3`, 5 `requires:4`). Real pipeline stays Sonnet on `phase-m1-b`; trial never gates shipping. Results → `minimax-m27-trial-results.md`.

When cumulative ✅ count reaches ≥5 (absent an override) → proceed to §2.3 dispatch sequence alongside the real phase.

Results file: `.claude/PRPs/reports/minimax-m3-trial-results.md` (create at trial start; append a new `##` section per phase run).

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
  scope it then; do NOT bundle into the shipping phase.

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
- **NEVER block any phase's shipping on trial completion** — trial is parallel,
  optional, and abortable.
- **NEVER switch impl-task to MiniMax mid-phase** — cutover is its own
  follow-up sub-phase after the 5-task data is in + reviewed.
- **Forbidden-window awareness:** the MiniMax arm runs cargo-class work on the
  EliteDesk? NO — under Shape-G-suspended, cargo runs on the laptop advisor
  session for the validate gate. But the Junior impl-task arm itself runs on
  the daemon (no cargo, just edits + commit), so forbidden windows apply only
  to the advisor's local validate run, not the dispatch. Per
  `advisor-orchestrator.md` §5.1.
