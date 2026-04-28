# Session close — v1-validate-agent shipped + advisor V1-additives in flight

**Author:** foreground impl session, brehon-fork CWD on `governance-v0` @ `2c93e11b0`
**When:** 2026-04-28 (session close)
**Mode:** save + close — next session resumes from this brief

## Bootstrap prompt (paste into next session opening brehon-fork)

> Resume the session that shipped v1-validate-agent (PR #104, merged 2026-04-27 at `ed4be2bd8`). Read `.claude/PRPs/handovers/session-close-2026-04-28-v1-validate-agent-shipped.md` (this file, on `governance-v0` @ `2c93e11b0`) end-to-end first. The advisor session has been busy in parallel post-merge; this brief lists what I did, what they did, and what's currently live. Do NOT touch any of the advisor's V1-additives WIP without checking — that work is mid-flight on `governance-v0`.

## What this session shipped (foreground impl)

**v1-validate-agent — PR #104 → MERGED at `ed4be2bd8`** (2026-04-27 21:49:34Z, 14 phase commits preserved, NOT squashed per `phase-branch.md`).

Phase commits on `governance-v0` (oldest → newest within the PR):

| SHA | Subject | Notes |
|---|---|---|
| `1b8d0061d` | feat(ci): cargo-validate workflows for phase-v1-* + junior/* push triggers | Task 1 |
| `ed049970b` | fix(ci): drop invalid --no-deps from cargo check step | Task 1 fix-up — DQ #69 |
| `70b3a4d08` | feat(rules,agents): schema-additive validate-* kinds + impl-task push-and-exit + advisor validate-stage | Task 3 atomic |
| `97a7c99d6` | docs(template): plan.template §15 DoD per workflow + §13 Shape G composition note | Task 4 |
| `6e03249fa` | docs(briefs): jm-d-impl-2 §5 retrofit — push-and-exit per Shape G | Task 5 |
| `000f86093` | feat(agents): ci-watcher subagent + brief template (Haiku 4.5, low) + clippy fix | Task 2 |
| `7bafb078c` | docs(handover): v1-validate-agent foreground impl paused after Task 2 | mid-phase pause |
| `da7e01358` | docs(retro): v1-validate-agent retro + lessons promoted | Task 6 + 3 lessons |
| `4b1d9a01c` | chore(pr-review): align validate-failed enum + classification source | cr-3 + cr-5 + cr-7 |
| `a17c1d0a4` | chore(pr-review): resolve impl-task Output discipline conflict under Shape G | cr-6 |
| `a1e9a20d9` | chore(pr-review): make migrate-roundtrip resilient to shallow CI checkout | cr-1 + cr-2 |
| `24c701562` | chore(pr-review): derive clippy toolchain from rustup show active-toolchain | cr-4 |
| `875073d29` | chore(pr-review): drop postgresql-client from stub migration workflow | cr-9 |
| `2f5fdbd99` | docs(verify): v1-validate-agent — all stories pass; safe-to-merge | verify report |

Then merge `ed4be2bd8`, then post-merge handover:

| `d715a1529` | docs(handover): v1-validate-agent post-merge follow-ups for homeserver session | direct on governance-v0 |

**Three lessons promoted in `da7e01358`:**
1. `feedback_gh_run_watch_exit_status_unreliable.md` — load-bearing; gh CLI 2.89.0 false-zero
2. `feedback_dtolnay_rust_toolchain_components_with_toml.md` — toolchain action ignores `components:` with rust-toolchain.toml; derive from `rustup show active-toolchain`
3. `feedback_planner_dod_dry_run_caught_partial.md` — meta-pattern; first-push validation is the real gate

**CR triage on PR #104:** 9 findings disposed — 7 done, 1 rebut (cr-8 markdownlint MD041 vs corpus convention; rationale documented in commit `875073d29` body), 1 fix-in-PR-now-done (cr-9). Findings YAML at `.claude/PRPs/reviews/pr-104-findings.yaml` (gitignored, runtime artifact).

**Workflow runs (final state on phase branch):**
- `cargo-validate-workspace` run [25020638716](https://github.com/barrie-cork/lemmy/actions/runs/25020638716) ✓ success on `24c701562` (cold ~28 min)
- `cargo-validate-migration` run 25021266534 ✓ success on `875073d29`

## Post-merge follow-ups (homeserver-side, completed by your advisor session)

Per the post-merge handover at `d715a1529`, three items needed action in `homeserver/`:

1. **`homeserver/library.yaml` — register 4 new artefacts.** ✓ done in homeserver commit `e4c51e6` (per your message during this session: "library.yaml: 4 v1-validate-agent artefacts registered (ci-watcher agent, 2 workflows, ci-watcher-brief template under new templates: kind)").
2. **`homeserver/CLAUDE.md` — ci-watcher disambiguation note.** ✓ done in homeserver commit `50cc944` (per your message: "CLAUDE.md: one-paragraph ci-watcher disambiguation note").
3. **Advisor flag — JM-d Task 2 unblocked per DQ #61.** Acknowledged in PMD (`workflow_state_2026_04_27_validate_agent_shipped.md` + `project_shape_g_adoption_2026_04_27.md` flipped to SHIPPED). The advisor will pick up the unblock on next polling tick — `git fetch origin` will reveal the merge.

**Stale PMD note to fix:** `project_shape_g_adoption_2026_04_27.md` says "a bounded retrofit of jm-d-impl-2.md §5 (inline cargo → push-and-exit + ci-watcher poll) is owed before the persistent advisor session queues Task 2". That retrofit ALREADY shipped as Task 5 of v1-validate-agent (commit `6e03249fa`, on `governance-v0` since the merge). The PMD entry should be updated to remove the "retrofit owed" line — it could mislead the next advisor session into thinking there's a pending step before Task 2 dispatch.

Verified locally:
```
$ grep -A2 -E '^## 5\.' .claude/PRPs/briefs/jm-d-impl-2.md
## 5. Validation gates (out-of-band on GH Actions per Shape G)

Retrofitted from inline cargo to push-and-exit per DQ #61 + v1-validate-agent.plan.md Task 5.
```

## What the advisor session shipped in parallel (NOT my work)

While I was finishing the v1-validate-agent merge, your advisor session committed 7 new commits on `governance-v0` that I should not modify without checking:

| SHA | Subject | What it touches |
|---|---|---|
| `e9170fbca` | chore(advisor): add 3 lessons for V1 spec additives (Opp 1+2+3) | `.claude/lessons/feedback_complexity_score_pre_split.md`, `feedback_explicit_file_arrays_on_tasks.md`, `feedback_handover_trailer_cohort_propagation.md` |
| `d094e0d42` | chore(advisor): plan + impl-task-brief templates for V1 spec additives | `.claude/PRPs/templates/impl-task-brief.template.md` (new), `.claude/PRPs/templates/plan.template.md` (extended) |
| `feb5c688c` | chore(advisor): planning + impl-task agent contracts for V1 additives | `.claude/agents/impl-task.md`, `.claude/agents/planning.md` |
| `ad5d1397b` | chore(advisor): advisor-orchestrator + brehon-verify wiring for V1 additives | `.claude/commands/brehon-verify.md`, `.claude/rules/advisor-orchestrator.md` |
| `72a7d170c` | chore(advisor): add synthetic-spec-trial-1 dogfood brief for V1 additives | `.claude/PRPs/briefs/synthetic-spec-trial-1.md` (new) |
| `902b279c4` | docs(agents): cite homeserver four-role tiering patch in subagent files | `.claude/agents/{bm-task,ci-watcher,impl-task,planning}.md` |
| `2c93e11b0` | chore(advisor): brief jm-d-ci-watcher-1 — dry-run validate-failed against run 25023186734 | `.claude/PRPs/briefs/jm-d-ci-watcher-1.md` (new) |

Two themes:

1. **V1 spec additives** (Opp 1+2+3) — appears to be a follow-up sub-phase the advisor is scoping. New lessons + new templates + agent-contract updates + a synthetic-spec-trial-1 dogfood brief. This is mid-flight; the next session should NOT change any of those files without re-reading the advisor's intent (read `.claude/PRPs/briefs/synthetic-spec-trial-1.md` first to understand the dogfood target).
2. **ci-watcher dry-run** — `jm-d-ci-watcher-1.md` brief targets workflow run **25023186734** for a dry-run `validate-failed` exercise. This is the FIRST real ci-watcher Junior dispatch — it'll exercise Stories 4 + 5 from the v1-validate-agent retro that were `[deferred-to-retro]`. The retro can be amended once those stories land successfully.

The advisor's `impl-task.md` and `planning.md` files have been edited TWICE post-merge:
- once in `feb5c688c` (V1 additives contract update)
- once in `902b279c4` (homeserver four-role tiering citation)

These changes coexist with my earlier `a17c1d0a4` edit to `impl-task.md` (CR cr-6 fix on Output discipline). Verify by reading the file post-resume — your additions should compose cleanly with the Shape G push-and-exit body I shipped.

## Current state at session close

```
Branch:     governance-v0
HEAD:       2c93e11b0 (advisor's latest, NOT mine)
Last mine:  d715a1529 (handover)
Tree:       clean, no pending edits
PR #104:    MERGED at ed4be2bd8
Phase br:   phase-v1-validate-agent retained (per phase-branch.md)
Routine:    trig_01TTdeDYw9forHBgWqPfmN4c (DISABLED — v1-validate-agent
            post-merge verification; manual run fired ~3× this session,
            scheduled 2026-04-28 22:00Z slot was disabled per user request)
```

## Still open (advisor's queue, not mine)

1. **ci-watcher dry-run against run 25023186734** — `jm-d-ci-watcher-1.md` brief committed at `2c93e11b0` is awaiting Junior dispatch. First real ci-watcher exercise. Its outcome will close Stories 4 + 5 from the v1-validate-agent retro (currently `[deferred-to-retro]`).
2. **V1 spec additives sub-phase** — Opp 1+2+3 lessons + templates + agent contracts shipped, but no plan file at `.claude/PRPs/plans/v1-spec-additives*.plan.md` yet. The advisor is mid-scoping; expect a clarify pass + plan write next.
3. **JM-d Task 2 dispatch** — fully unblocked per DQ #61 (validate-agent merged + jm-d-impl-2.md §5 retrofitted). Advisor's polling loop will queue when ready.
4. **Stale PMD note** — `project_shape_g_adoption_2026_04_27.md` claims "retrofit owed"; reality is the retrofit shipped. Worth updating in your next advisor-session sweep.

## Decision-queue state

`.claude/decision-queue.json` was last updated by my session at the v1-validate-agent retro time. Three resolved entries from this session:
- DQ #68 (impl-self-resolved) — `migrate-roundtrip.sh` phantom dependency
- DQ #69 (impl-self-resolved) — `cargo check --no-deps` flag inheritance
- DQ #70 (impl-self-resolved) — gh CLI 2.89.0 `--exit-status` false-zero (load-bearing)

DQ #61 (advisor 2026-04-27 plan-mode) is still resolved with the unblock condition now satisfied. No new pending entries. The advisor session may have added entries post-merge — check before assuming the queue is empty.

## What NOT to do in the next session

- **Don't touch the advisor's V1-additives WIP files** (`feedback_complexity_score_pre_split.md`, `feedback_explicit_file_arrays_on_tasks.md`, `feedback_handover_trailer_cohort_propagation.md`, `impl-task-brief.template.md`, agent contract changes from `feb5c688c`/`902b279c4`, `synthetic-spec-trial-1.md`, `jm-d-ci-watcher-1.md`) without first reading them and the advisor's intent.
- **Don't re-merge or re-PR v1-validate-agent.** It shipped. Phase branch retained but no further work belongs on it.
- **Don't run cargo locally.** v1-validate-agent established Shape G — cargo runs on GH Actions now. Local cargo is for advisor pre-plan-approval DoD smoke tests only.
- **Don't re-trigger or re-arm `trig_01TTdeDYw9forHBgWqPfmN4c`.** It's DISABLED. The follow-up verification was completed manually during this session; the routine served its purpose.

## See also

- `.claude/PRPs/plans/v1-validate-agent.plan.md` — the plan that just shipped (confidence 7/10)
- `.claude/PRPs/reports/v1-validate-agent-retro.md` — retro (confidence 8/10)
- `.claude/PRPs/reports/v1-validate-agent-verify.md` — /brehon-verify equivalent
- `.claude/PRPs/handovers/post-merge-2026-04-27-validate-agent-followups.md` — the homeserver-side handover (now mostly complete)
- `.claude/PRPs/briefs/jm-d-ci-watcher-1.md` — advisor's ci-watcher dry-run brief (newest)
- `.claude/PRPs/briefs/synthetic-spec-trial-1.md` — V1-additives dogfood brief
- PR #104: https://github.com/barrie-cork/lemmy/pull/104

---

_Session close 2026-04-28. Foreground impl session that shipped v1-validate-agent end-to-end + drafted post-merge follow-up brief. Advisor session running in parallel; this brief reflects the state at close, not the advisor's intent — read advisor briefs directly when resuming._
