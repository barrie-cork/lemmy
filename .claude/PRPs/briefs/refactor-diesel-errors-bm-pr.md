---
phase: chore/refactor-diesel-errors
role: bm-task
task: bm-pr
brief_n: 2
authored: 2026-05-15
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_findings: 3.A.5 (rank 13 — PR-5)
note: "Likely run INLINE by advisor (L15 precedent) — Junior bm-task base_branch=chore/* fails on daemon worktree-ref resolution (PR-4 #264 incident). This brief documents intent regardless of executor."
---

# [role:bm-task] bm-pr chore/refactor-diesel-errors — see .claude/PRPs/briefs/refactor-diesel-errors-bm-pr.md

## §1 Role + dispatch

`[role:bm-task] bm-pr chore/refactor-diesel-errors — open PR into governance-v0`

## §2 Scope

Open a PR from `chore/refactor-diesel-errors` into `governance-v0`
per `.claude/commands/bm/bm-pr.md` (Phase 1 → 7). Auto, no prompt
(autonomy table: open PR into governance-v0 is Auto).

**Branch state (prepared by advisor before bm-pr — do NOT re-cut):**

The advisor cuts `chore/refactor-diesel-errors` off `governance-v0`,
dispatches an impl-task Junior with `base_branch=governance-v0` (NOT
`base_branch=chore/*` — daemon cannot resolve chore refs, per PR-4
#264), then rebases the worker's `chore(refactor): propagate Diesel
errors in admin_audit_stream (audit 3.A.5)` commit onto
`chore/refactor-diesel-errors`, backfilling the validate-pending DQ
entry if the worker omitted it (PR-4 #263 process-miss precedent).
By bm-pr time the branch carries: the refactor commit + (if needed)
a DQ-backfill commit + this bm-pr brief.

- Chore branch — no `.plan.md`. bm-pr retro gate + Phase 1c e2e
  gate correctly skip for chore branches (plan-aware script).
- Phase 1b historical-fail sweep: no-op if validate-pending passed
  cleanly (expected — pure 2-line Rust refactor).

**Title** (chore → `chore(<scope>): <prose>`): `chore(refactor): propagate Diesel errors in admin_audit_stream (audit 3.A.5)`

**PR body** — per bm-pr Phase 3. No completion report, no plan. Body
MUST include:

- `## Summary` — Replaces the `.optional().ok().flatten()` chain at
  2 sites in `crates/api/api/src/governance/admin_audit_stream.rs`
  (~lines 227, 234) with proper error propagation. The `.ok()`
  between `.optional()` and `.flatten()` silently reduced a DB query
  failure (pool exhaustion / syntax / connection) to `None`,
  indistinguishable from a legitimate empty result, in an
  observability-critical streaming handler. Fixed per audit §3.A.5
  recommendation (`.optional()?` if enclosing returns `LemmyResult`,
  else the `.map_err(|e| { tracing::warn!(...); e }).ok().flatten()`
  logging variant).
- `## Plan reference` — `Ad-hoc — no plan file (chore branch;
  audit-driven refactor-tier PR-5 of 5 per
  .claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md)`
- `## Closes` — `Closes audit finding 3.A.5
  (v1-code-quality-audit-2026-05-14.md, rank 13, severity MAJ)`
- `## Validation` — Workspace check via Shape-G (DQ entry id +
  result). If the GH `cargo-validate-workspace` run stuck (PR-4
  precedent ×2), state the advisor-laptop local cargo-check fallback
  per advisor-orchestrator.md §5.2 with the exit code + log path.
  Note which fix variant (`?` vs `.map_err`) the worker picked +
  the enclosing-function-shape rationale (from the worker's
  HANDOVER trailer if present).

## §3 Required reading

- `.claude/commands/bm/bm-pr.md` (operational script — Phase 1→7)
- `.claude/rules/branch-manager.md` (file-ownership; autonomy table)
- `.claude/rules/phase-branch.md` (PR into governance-v0, not main; not draft)
- `.claude/rules/gh-pr-fork-target.md` (`--repo barrie-cork/lemmy` mandatory)
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md` (PR-5 of 5; sequential; strict-gate)

## §4 Constraints

- **NEVER** touch `crates/**`, `migrations/**`, `tests/**`,
  `docs/brehon-law-inspired-network/**`.
- **NEVER** open PR into `main` — base `governance-v0`.
- **NEVER** open as draft (CR skips drafts).
- **NEVER** re-cut or force-push the branch.
- `--repo barrie-cork/lemmy` on EVERY `gh pr` subcommand.
- Do NOT post a PR comment / submit a review (Manual/ask per
  autonomy table — bm-pr only opens the PR; the triage digest
  comment is a separate user-gated step).
- Append to runlog `.claude/runlog/chore-refactor-diesel-errors.md`
  (create if absent).
- Return PR number + URL in the final summary.

## §5 Concurrency note

Sequential lane — PR-4 (valid-from) already merged (#128). PR-6
(seed-tests) PARKED until PR-5 PR is open + CR-triaged. Zero
cross-lane file overlap (this PR touches only
`admin_audit_stream.rs`).
