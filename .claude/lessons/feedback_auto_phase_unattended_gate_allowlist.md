---
name: /auto-phase --unattended gate allowlist (hard contract)
description: --unattended adds partial autonomy to /auto-phase by auto-clearing ONLY two no-judgment gates (4 e2e→local, 6 retro sign-off) and parking-and-pinging the four judgment gates (1 plan, 2 ADR/scope-DQ, 3 CR-triage, 5 merge). The allowlist is exhaustive and NOT runtime-extensible; no cheap-model agent is spun up; auto-clears log a distinct audit label, never "user".
type: feedback
---

# `/auto-phase --unattended` gate allowlist (hard contract)

`--unattended` is an opt-in partial-autonomy layer on `/auto-phase`. It exists so a multi-hour sub-phase does not sit dead for the hours the user is away — the mechanical stretches keep driving, the two no-judgment gates clear themselves, and the four judgment gates park-and-ping so the user returns to a batched queue of real decisions instead of a phase stalled at the first gate.

**It is NOT the rejected `--auto-all-gates` (L15).** That would have let a model clear plan-approval, ADR-DQ, CR-triage, or merge. `--unattended` never touches those four.

**Why this shape (the design rejection that produced it):** the original ask was "spin up a cheap (Haiku) model that monitors Junior tasks and tells the advisor when to progress." That framing was dissolved at design time because the monitor role is **already filled twice** — passively by the daemon's task-completion hook (Telegram ✅/❌ out-of-band), and actively by the Opus advisor's own `ScheduleWakeup` loop (near-free while sleeping; one cheap status-only `list_tasks` on wake). A cheap agent in the middle would **add a hop, not remove one** (the advisor must still wake to *author* the next brief / dispatch / run the gate — work Haiku is not authorised to do), **cost continuous tokens** across the phase (vs ~zero while sleeping), and **add a mis-classification surface** (a Haiku summary the advisor would have to verify against ground truth anyway — directly the `pattern_bm_false_success_advisor_post_condition_catch` failure, 5× confirmed). So the autonomy is a **fixed policy table**, not a model. No agent is spun up.

## The allowlist (exhaustive — do NOT extend without a new user-confirmed contract)

| Gate | `--unattended` behaviour | Why this side of the line |
|---|---|---|
| 1 Plan approval | **PARK + ping** | Judgment-heavy; a bad plan poisons the phase. |
| 2 Judgment-heavy DQ (ADR/scope/visible-to-others) | **PARK + ping** | The cheap-model-drops-ADR class ([[cheap-model-arm-drops-adr-constraints]], m1-b incident). |
| 3 CR triage | **PARK + ping** | Each finding is a compile-checkable judgment; mis-bucketing a `critical` ships a bug. |
| 4 e2e local-vs-dispatch | **AUTO: `local`** (or `dispatch` iff `--e2e dispatch`) | `local` is free + zero-billed + no public log. Auto-`local` only forecloses the *billed public* path — which a human opts INTO, never out of. |
| 5 Merge confirm | **PARK + ping** | Irreversible + outward-facing; `gh pr merge` NEVER fires unattended. |
| 6 Retro sign-off | **AUTO-CLEAR after a 3-check sanity gate** | Retro is internal, append-only, reversible, no code impact. |

Catch-fires are NEVER suppressed by `--unattended` — they park + ping like a gate, then wait for manual resume.

## How to apply

1. **Gate 6 auto-clear requires ALL THREE sanity checks** (existence checks, not a quality judgment — the advisor authored the retro one stage earlier): file exists; ≥800 bytes; required §-sections present (four-role retro signals per [[four-role-retro-signals]] + a "What to change"/§3-actions section + a per-task complexity line per [[retro-task-complexity-score]]). Any fail → **catch-fire**, not auto-clear. A malformed retro is a phantom-completion — auto-signing it off would launder a broken artifact past the audit.

2. **Audit honesty is load-bearing.** Auto-cleared gates write `decision: "auto-approve-unattended"` in `user_gate_history`, **NEVER** `"user"`. A `"user"` label on a policy auto-clear is an attribution breach in the same class as a non-advisor session forging `answered_by: "advisor"` (`decision-queue.md` "Attribution integrity") — catch-fire if detected on read. A retro reading `user_gate_history` must be able to report exactly which gates a human touched vs the policy cleared.

3. **Park-and-ping fires ONE Telegram notification per gate-reached event** (within Telegram scope — a `dq-blocking`/`merge-ready`-class ping per [[telegram-scope-notification-only]]; no diff content, no secrets, no file dumps; no batching). The `--unattended` opt-in IS the standing authorisation for that ping shape and ONLY that shape. MCP disconnected → silent skip + note in the auto-handover (pings are notifications, not gating signals — a missed ping never advances or blocks a gate).

4. **The flag is invocation-time, not sticky state.** A resume WITHOUT `--unattended` reverts to fully-gated — the safe direction. The schema-upgrade-in-place block re-derives `state['unattended']` from the current `$ARGUMENTS`, not from the prior state file.

5. **The allowlist is NOT runtime-extensible.** No flag widens it. Extending it requires a new user-confirmed contract edited into the Phase 7 table in `~/.claude/commands/auto-phase.md`. (At design time the user explicitly declined to add gate 5/merge to the allowlist — that decision is the boundary.)

## See also

- `~/.claude/commands/auto-phase.md` "Phase 7 — `--unattended` gate allowlist" — the canonical hard contract (this lesson is the durable record + rationale; the skill body is the executable spec).
- `.claude/refs/auto-phase.md` Hard refusal #11.
- `.claude/PRPs/templates/auto-phase-state.template.json` — `unattended` / `unattended_e2e_override` fields + the `user_gate_history` decision enum (`parked-pending-user`, `auto-approve-unattended`).
- [[cheap-model-arm-drops-adr-constraints]] — why gates 1/2/3 cannot be cheap-model-cleared.
- [[telegram-scope-notification-only]] — the ping scope the gate-waiting notification honours.
- `pattern_bm_false_success_advisor_post_condition_catch` (PMD) — why a cheap-model summary would have to be re-verified, dissolving the original monitor-agent framing.
