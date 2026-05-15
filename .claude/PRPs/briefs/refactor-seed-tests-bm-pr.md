---
phase: chore/refactor-seed-tests
role: bm-task
task: bm-pr
brief_n: 2
authored: 2026-05-15
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_findings: 3.E.20 (rank 17 — PR-6, LAST refactor lane)
note: "Run INLINE by advisor (L15 precedent + L3 — Junior bm-task base_branch=chore/* fails on daemon worktree-ref resolution, PR-4 #264 + confirmed PR-5). This brief documents intent regardless of executor. L5: this brief lives ONLY on governance-v0 — it is NOT cherry-picked onto chore/refactor-seed-tests (cherry-picking a governance-v0 brief onto a chore branch caused the PR #129 CONFLICTING incident when an advisor edit made the duplicate diverge)."
---

# [role:bm-task] bm-pr chore/refactor-seed-tests — see .claude/PRPs/briefs/refactor-seed-tests-bm-pr.md

## §1 Role + dispatch

`[role:bm-task] bm-pr chore/refactor-seed-tests — open PR into governance-v0`

## §2 Scope

Open a PR from `chore/refactor-seed-tests` into `governance-v0`
per `.claude/commands/bm/bm-pr.md` (Phase 1 → 7). Auto, no prompt
(autonomy table: open PR into governance-v0 is Auto).

**Branch state (prepared by advisor before bm-pr — do NOT re-cut):**

The advisor cuts `chore/refactor-seed-tests` off the CURRENT
`governance-v0` tip (`6cdbe5926` — post-PR-4 #128 + PR-5 #129 merge),
dispatches an impl-task Junior with `base_branch=governance-v0` (NOT
`base_branch=chore/*` — daemon cannot resolve chore refs, per PR-4
#264 + confirmed PR-5 #265), then rebases the worker's
`test(seed_founders): unit tests for parse_founder_spec validation
(audit 3.E.20)` commit onto `chore/refactor-seed-tests`, backfilling
the validate-pending DQ entry if the worker omitted it (PR-4 #263 +
PR-5 #265 — 2/2 process-miss precedent; expect 3/3, backfill
proactively). By bm-pr time the branch carries: the test-addition
commit + (if needed) a DQ-backfill commit. **The bm-pr brief is
NOT on this branch (L5) — it lives only on governance-v0.**

- Chore branch — no `.plan.md`. bm-pr retro gate + Phase 1c e2e
  gate correctly skip for chore branches (plan-aware script).
- Phase 1b historical-fail sweep: no-op if validate-pending passed
  cleanly (expected — pure additive test module, no production-code
  change).

**Title** (chore/test → `test(<scope>): <prose>`): `test(seed_founders): unit tests for parse_founder_spec validation (audit 3.E.20)`

**PR body** — per bm-pr Phase 3. No completion report, no plan. Body
MUST include:

- `## Summary` — Adds a `#[cfg(test)] mod tests` block at the end of
  `crates/tools/seed_founders/src/main.rs` with exactly 5 sync
  `#[test]` cases covering `parse_founder_spec` validation:
  `parse_founder_spec_valid` (happy path), `_negative_delta` (zero /
  negative reputation dimensions reject), `_exceeds_max` (values >
  `max_seed_delta` reject), `_wrong_segment_count` (≠4 colon segments
  reject), `_non_numeric` (non-i32 segments reject with parse-error
  message). `parse_founder_spec` is the CLI entry-point parser for the
  founder-seeding tool; before this change it had zero edge-case
  coverage (audit §3.E.20, rank 17, severity MED). PURE additive —
  no production code touched, no `Cargo.toml` change (`LemmyErrorType`
  already imported). Tests are sync (`#[test]`, not `#[tokio::test]`)
  — no DB / testcontainers; run in <1s.
- `## Plan reference` — `Ad-hoc — no plan file (chore branch;
  audit-driven refactor-tier PR-6 of 5 — LAST lane — per
  .claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md;
  PR-3 was dropped via DQ #214 so the gate is 5 PRs not 6)`
- `## Closes` — `Closes audit finding 3.E.20
  (v1-code-quality-audit-2026-05-14.md, rank 17, severity MED)`
- `## Validation` — Workspace check via Shape-G (DQ entry id +
  result). Note: Shape-G `cargo-validate-workspace` runs `cargo test
  --no-run` so it COMPILES but does NOT execute the new tests; the
  worker is required (impl brief §2.5 + §4) to run the 5 tests
  locally before push. If the GH `cargo-validate-workspace` run
  stuck (PR-4 ×2 + PR-5 ×1 precedent — persistent), state the
  advisor-laptop local cargo-check (+ local `cargo test -p
  seed_founders --tests`) fallback per advisor-orchestrator.md §5.2
  with the exit code + log path.

## §3 Required reading

- `.claude/commands/bm/bm-pr.md` (operational script — Phase 1→7)
- `.claude/rules/branch-manager.md` (file-ownership; autonomy table)
- `.claude/rules/phase-branch.md` (PR into governance-v0, not main; not draft)
- `.claude/rules/gh-pr-fork-target.md` (`--repo barrie-cork/lemmy` mandatory)
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md` (PR-6 of 5 — LAST; sequential; strict-gate)

## §4 Constraints

- **NEVER** touch `crates/**`, `migrations/**`, `tests/**`,
  `docs/brehon-law-inspired-network/**`.
- **NEVER** open PR into `main` — base `governance-v0`.
- **NEVER** open as draft (CR skips drafts).
- **NEVER** re-cut or force-push the branch.
- **NEVER** cherry-pick this brief onto `chore/refactor-seed-tests`
  (L5 — PR #129 CONFLICTING root cause). The brief is referenced
  by path; it stays on governance-v0 only.
- `--repo barrie-cork/lemmy` on EVERY `gh pr` subcommand.
- Do NOT post a PR comment / submit a review (Manual/ask per
  autonomy table — bm-pr only opens the PR; the triage digest
  comment is a separate user-gated step).
- Append to runlog `.claude/runlog/chore-refactor-seed-tests.md`
  (create if absent).
- Return PR number + URL in the final summary.

## §5 Concurrency note

Sequential lane — PR-4 (valid-from) merged (#128, strict-gate 1/5),
PR-5 (diesel-errors) merged (#129, strict-gate 2/5). This is the
LAST refactor lane (PR-6 of 5; PR-3 dropped via DQ #214). On PR-6
merge → strict-gate 5/5 satisfied → autonomous loop STOPS → 4-role
retro at `.claude/PRPs/reports/refactor-tier-retro.md` → user
sign-off gate. Zero cross-lane file overlap (this PR touches only
`crates/tools/seed_founders/src/main.rs`, a file no prior refactor
lane touched).
