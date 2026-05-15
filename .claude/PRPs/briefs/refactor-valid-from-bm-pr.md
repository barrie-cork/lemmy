---
phase: chore/refactor-valid-from
role: bm-task
task: bm-pr
brief_n: 2
authored: 2026-05-15
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_findings: 3.D.6 (rank 6 — PR-4)
---

# [role:bm-task] bm-pr chore/refactor-valid-from — see .claude/PRPs/briefs/refactor-valid-from-bm-pr.md

## §1 Role + dispatch

`[role:bm-task] bm-pr chore/refactor-valid-from — open PR into governance-v0`

## §2 Scope

Open a PR from `chore/refactor-valid-from` into `governance-v0` per
`.claude/commands/bm/bm-pr.md`. Execute Phase 1 → Phase 7 of that
script. Auto, no prompt (per branch-manager.md autonomy table — open
PR into governance-v0 is Auto).

**Branch state (already prepared by advisor — do NOT re-cut or
re-push):**

- `chore/refactor-valid-from` is at `af055f0fa` (pushed to origin):
  - `a2aa023ed` — `chore(migrations): pin valid_from literals on v0 + v1-AD-a seed migrations (audit §3.D.6)` (the refactor; 3 SQL files)
  - `af055f0fa` — `chore(decision-queue): advisor-laptop backfill DQ #215 #216 — PR-4 valid-from validate-pending pass`
- Working tree is clean; branch is pushed; remote == local.
- This is a **chore branch** — no `.plan.md` exists. bm-pr script's
  retro gate + Phase 1c e2e gate both correctly skip for chore
  branches (script is plan-aware). Phase 1b historical-fail sweep is
  a no-op (pending=0, no failed validate-pending entries).

**Title** (chore branch → `chore(<scope>): <prose>` from first
commit subject): `chore(migrations): pin valid_from literals on v0 + v1-AD-a seed migrations (audit 3.D.6)`

**PR body** — assemble per bm-pr Phase 3. No completion report, no
plan file. Use commit log + this context. Body MUST include:

- `## Summary` — Retrofits `valid_from` literal pinning on 2 seed
  migrations (`2026-04-18-...add_governance_config`,
  `2026-04-22-...seed_v1_config_keys`) + down.sql parity. Without the
  literal, `valid_from` defaults to `now()` and each migration rerun
  inserts a duplicate active row under the `(scope, key, valid_from)`
  unique index. Mirrors the JM-a cr-10 pattern from
  `2026-04-23-...seed_v1_jm_config_keys/up.sql`.
- `## Plan reference` — `Ad-hoc — no plan file (chore branch; one of
  the audit-driven refactor-tier PRs per
  .claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md)`
- `## Closes` — `Closes audit finding 3.D.6 (v1-code-quality-audit-2026-05-14.md, rank 6)`
- `## Validation` — Workspace check: local `cargo-check.sh
  --workspace --features full` PASS (DQ #215, advisor-laptop fallback
  after GH cargo-validate-workspace stuck-runner x2). Migration check:
  GH `cargo-validate-migration` run 25891289464 PASS (DQ #216).
  Reference the advisor-laptop backfill rationale (Junior #263
  omitted Recipe-1 DQ writes; recovery per advisor-orchestrator.md
  §5.2 validate-pending-laptop handler).

## §3 Required reading

- `.claude/commands/bm/bm-pr.md` (the operational script — follow Phase 1→7)
- `.claude/rules/branch-manager.md` (file-ownership; autonomy table)
- `.claude/rules/phase-branch.md` (PR into governance-v0, not main; not draft)
- `.claude/rules/gh-pr-fork-target.md` (`--repo barrie-cork/lemmy` mandatory on every `gh pr` call)
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md` (PR-4 of 5; sequential lane; strict-gate context)

## §4 Constraints

- **NEVER** touch `crates/**`, `migrations/**`, `tests/**`,
  `docs/brehon-law-inspired-network/**` (BM file-ownership boundary).
- **NEVER** open PR into `main` — base is `governance-v0`.
- **NEVER** open as draft (CR skips drafts per phase-branch.md).
- **NEVER** re-cut or force-push the branch — it is already prepared.
- `--repo barrie-cork/lemmy` on EVERY `gh pr` subcommand.
- Do NOT post a PR comment or submit a review (those are Manual/ask
  per autonomy table — bm-pr only opens the PR).
- Do NOT run `/bm-poll-cr` or `/bm-triage` — advisor dispatches those
  separately after CR posts.
- Append to runlog `.claude/runlog/chore-refactor-valid-from.md`
  (Phase 6) — create if absent (bm-cut created it).
- Return the PR number + URL in the final summary so the advisor can
  track it.

## §5 Concurrency note

Lane-dedicated worktree on the daemon side. PR-5 (`diesel-errors`) +
PR-6 (`seed-tests`) are NOT yet dispatched (advisor holds them
sequentially until this PR-4 PR is open + CR-triaged). No cross-lane
file overlap (this PR is the only one touching `migrations/`).
