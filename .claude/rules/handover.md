# Handover — shared invariants for `/handover-*` commands

This rule is referenced by `.claude/commands/handover/handover-advisor.md`
and `.claude/commands/handover/handover-impl.md`. Both commands write a
"resume brief" to `.claude/PRPs/handovers/` so a fresh Claude Code
session can pick up where the closing session left off.

The rule exists because the handover corpus pre-dates the commands:
six hand-written briefs under `.claude/PRPs/handovers/` and
`.claude/PRPs/reports/*-handover-*.md` (see §"Canonical exemplars")
already established the shape. The commands automate it. This rule
captures the invariants the commands must preserve so briefs remain
consistent with the existing corpus and with the role-ownership lines
drawn by `.claude/rules/branch-manager.md` and
`.claude/rules/decision-queue.md`.

## Role → command mapping

| Role | Command | Runs from |
|---|---|---|
| Advisor | `/handover-advisor` | Primary worktree on `governance-v0` |
| Impl | `/handover-impl` | Phase worktree on `phase-*` |
| BM | (none yet) | BM state reconstructable from `bm-runlog.md`; revisit after v1-JM-b |

A command run from the wrong worktree STOPS and asks (see §"Attribution
guards").

## Attribution guards

- The handover file's `**Author:**` field must match the session
  writing it. Advisor handovers preserve `answered_by: "advisor"`
  discipline; impl handovers NEVER self-attribute as advisor (per
  `.claude/rules/decision-queue.md:77-98`).
- The bootstrap prompt the brief emits names the role the next session
  should play. Never generate a cross-flavor prompt — wrong flavor on
  resume = wrong file-ownership boundaries = DQ attribution breach.
- `/handover-advisor` from a `phase-*` branch → STOP and ask. Advisor
  work lives on `governance-v0`.
- `/handover-impl` from `governance-v0` or without a plan file on the
  current branch → STOP and ask. Impl work lives on a phase branch
  with a committed plan.

## File ownership

Per `.claude/rules/branch-manager.md`:

### Advisor command WRITES

- `.claude/PRPs/handovers/advisor-*.md` — the brief
- `.claude/runlog/bm-runlog.md` — one-line append
- (existing) DQ pending entries, plan amendments — only if the command
  detects them uncommitted; it does not initiate them

### Advisor command NEVER WRITES

- `crates/**`, `migrations/**`, `tests/**`
- `.claude/PRPs/plans/*.plan.md` after initial commit (plan edits are
  a separate `docs(plan):` commit, not bundled with the handover)
- `.claude/PRPs/reviews/pr-*-findings.yaml` (BM-owned)

### Impl command WRITES

- `.claude/PRPs/handovers/impl-*.md` — the brief
- `.claude/runlog/<phase>-runlog.md` — one-line append, only if the
  file already exists on this branch; else skip (no file-creation
  during a handover)

### Impl command NEVER WRITES

- `.claude/PRPs/plans/*.plan.md` (plan files are advisor-owned once
  committed)
- `.claude/decision-queue.json` (DQ writes use `/bm-*` or a DQ
  draft relay, not a handover)
- `.claude/runlog/bm-runlog.md` (that file lives on `governance-v0`;
  cross-branch writes break phase discipline)
- `crates/**`, `migrations/**`, `tests/**` (always out-of-scope for a
  handover)

## What never goes in a brief

- Cargo log tails over ~30 lines. Reference a path
  (`.claude/build-*.log`, `.claude/audit-*.log`) and the exit code,
  not the body. Companion rule `.claude/rules/no-cargo-output-paste.md`.
- Raw CodeRabbit finding dumps. Reference the findings YAML at
  `.claude/PRPs/reviews/pr-<N>-findings.yaml` and cite the `id` field.
- Speculation about what the next session *will* do. Prescribe
  concrete actions with file:line refs, expected outputs, and a
  rollback if the action fails.
- Secrets, tokens, any `.env` content.
- Cross-role state that muddles ownership. An impl brief does not
  inventory pending DQ entries beyond flagging impl-side blockers;
  an advisor brief does not inventory phase-branch commits beyond
  noting merge readiness.

## When to write

### Advisor

- Before PC restart
- At DQ-batch closure (all pending advisor entries answered)
- At phase-transition boundaries (plan PR merged; phase PR merged)
- Before handing the session back to user for a decision requiring
  human-turnaround time

### Impl

- At task-N → task-N+1 boundary when the session is going to close
- Before PC restart
- When blocked on advisor input with no independent work available
- After a CI-driven regression is caught (see
  `pr92-cr-12-overreach-revert-ask.md` precedent — that relay became
  the de-facto handover for the session that picked up the fix)

**Do not write a handover mid-commit.** Finish or revert the current
commit first. Working-tree state in a brief is load-bearing for
resume — ambiguity between "committed" and "staged, not committed"
causes the resumer to either re-apply edits or miss them.

## Length target

150–300 lines. The existing corpus sits in this range:

- `v1-AD-c-advisor-resume-state.md` — 204 lines
- `phase-5a-handover-task-54-onward.md` — 200+ lines
- `v1-prep-2026-04-19.md` — 116 lines
- `pr46-bucket-c-merge.md` — 213 lines
- `v1-prd-edit-pass-2026-04-19.md` — 322 lines (complex multi-PRD
  in-flight edits; upper bound)
- `v1-prep-bootstrap.md` — 29 lines (bootstrap-only; not a full
  brief, companion to `v1-prep-2026-04-19.md`)

Under 150 lines = probably missing a "What's pending" or "Closing
state assertions" section. Over 350 lines = probably pasting cargo
output or CR dumps that belong in referenced files.

## Canonical exemplars (read when the rule is ambiguous)

- `.claude/PRPs/reports/v1-AD-c-advisor-resume-state.md` — advisor
  cold-resume brief at PR #81 CR round-2 stage (CR triage matrix,
  commit template, rebuttal reply texts, morning-session first-message
  template)
- `.claude/PRPs/reports/phase-5a-handover-task-54-onward.md` — impl
  handover at task 53 → 54 boundary (git state, plan deviations, DQ
  state, per-task specs)
- `.claude/PRPs/handovers/v1-prep-bootstrap.md` — bootstrap-prompt
  exemplar (29 lines; companion to `v1-prep-2026-04-19.md`)
- `.claude/runlog/advisor-relays/_README.md` — relay schema (fixed
  section-heading discipline the handover brief shares)
- `.claude/runlog/impl-relays/_README.md` — impl-relay schema

## See also

- `.claude/rules/branch-manager.md` — file ownership, autonomy bounds
- `.claude/rules/decision-queue.md` — attribution rules
- `.claude/rules/phase-branch.md` — branch topology
- `.claude/rules/no-cargo-output-paste.md` — no log-tail pasting
- `feedback_disable_model_invocation_for_user_only_commands.md`
  (memory) — why both commands set `disable-model-invocation: true`
