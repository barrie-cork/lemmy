---
phase: chore/refactor-toctou
role: bm-task
task: bm-pr
brief_n: 2
authored: 2026-05-15
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_findings: 3.B.1 (rank 3 — PR-2, CRITICAL; first of 2 remaining serial lanes)
note: "Run INLINE by advisor (L15 precedent + L3 — Junior bm-task base_branch=chore/* fails on daemon worktree-ref resolution, proven PR-4 #264 + PR-5 #265 + PR-6 #130). This brief documents intent regardless of executor. L5: this brief lives ONLY on governance-v0 — it is NOT cherry-picked onto chore/refactor-toctou (cherry-picking a governance-v0 brief onto a chore branch caused the PR #129 CONFLICTING incident when an advisor edit made the duplicate diverge)."
---

# [role:bm-task] bm-pr chore/refactor-toctou — see .claude/PRPs/briefs/refactor-toctou-bm-pr.md

## §1 Role + dispatch

`[role:bm-task] bm-pr chore/refactor-toctou — open PR into governance-v0`

## §2 Scope

Open a PR from `chore/refactor-toctou` into `governance-v0`
per `.claude/commands/bm/bm-pr.md` (Phase 1 → 7). Auto, no prompt
(autonomy table: open PR into governance-v0 is Auto).

**Branch state (prepared by advisor before bm-pr — do NOT re-cut):**

The advisor applied the L1 daemon-base root fix first (the daemon
`/srv/brehon-fork` `governance-v0` was STALE at `ac52534e3d` — not on
the merged lineage; reset to `origin/governance-v0` `09db10847`), cut
`chore/refactor-toctou` off `governance-v0` @ `09db10847` (post-PR-4
#128 + PR-5 #129 + PR-6 #130 merge + retro), dispatched an impl-task
Junior #267 with `base_branch=governance-v0` (NOT `base_branch=chore/*`
— daemon cannot resolve chore refs, per PR-4 #264 + PR-5 #265 + PR-6
#130), then cherry-picked the worker's `fix(api_crud): wrap
create_report SELECT-then-write in run_transaction (audit 3.B.1 CRIT)`
commit (`511640ae9` → `e57c20e9e` on chore) onto
`chore/refactor-toctou`. Worker #267 raised the validate-pending DQ
cleanly off the correct L1-fixed base (DQ #219, `from:impl`, NOT a
stale-base collision — the L1 proactive fix worked; this is the
"raised-clean" mode, contrast PR-6 #266 stale-collision). By bm-pr
time the branch carries: the `fix(api_crud)` commit + the
advisor-laptop validate-pending DQ-backfill commit + the lane runlog.
**The bm-pr brief is NOT on this branch (L5) — it lives only on
governance-v0.**

- Chore branch — no `.plan.md`. bm-pr retro gate + Phase 1c e2e
  gate correctly skip for chore branches (plan-aware script).
- Phase 1b historical-fail sweep: no-op if validate-pending passed
  cleanly (expected — single-file structural refactor, zero behavior
  change, sibling pattern already proven in `create_endorsement.rs`).

**Title** (chore/fix → `fix(<scope>): <prose>`): `fix(api_crud): wrap create_report SELECT-then-write in run_transaction (audit 3.B.1 CRIT)`

**PR body** — per bm-pr Phase 3. No completion report, no plan. Body
MUST include:

- `## Summary` — Wraps the SELECT-then-UPDATE/INSERT block in
  `crates/api/api_crud/src/governance/create_report.rs` in
  `conn.run_transaction(...)`, closing a CRITICAL TOCTOU race
  (audit §3.B.1, rank 3, severity CRITICAL): two concurrent
  `create_report` calls for the same target could both read "no
  existing case" then both INSERT, or interleave the case-version
  UPDATE with another writer. Extracts a `process_report` helper
  holding the transaction-closure body, mirroring
  `create_endorsement::process_endorsement` (named helper keeps the
  outer future under the workspace `large_futures` lint threshold).
  The pseudonym fetch is hoisted PRE-transaction (mirrors
  `create_endorsement.rs` per ADR-015); `governance_log::append` is
  rewired from the pre-tx `pool_ref` to the in-tx connection
  (`&mut (&mut *conn).into()`) so log appends are now atomic with the
  case write (substantive correctness improvement). Pre-tx validation
  (reason_code checks) intentionally left at pre-tx position per audit
  §3.B.6 positive exemplar. ZERO behavior change on the success path;
  no concurrency test added (out of scope per impl brief §5). Sibling
  `create_endorsement.rs` / `revoke_endorsement.rs` already use this
  pattern — this restores internal consistency across the governance
  write handlers.
- `## Plan reference` — `Ad-hoc — no plan file (chore branch;
  audit-driven refactor-tier PR-2 of 5 — serial dedicated lane
  Step-2b — per
  .claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md;
  PR-3 was dropped via DQ #214 so the gate is 5 PRs not 6;
  user-chosen sequencing PR-2 first then PR-1)`
- `## Closes` — `Closes audit finding 3.B.1
  (v1-code-quality-audit-2026-05-14.md, rank 3, severity CRITICAL)`
- `## Validation` — Workspace check via Shape-G. Note: Shape-G
  `cargo-validate-workspace` triggers on `junior/*` pushes only (NOT
  `chore/*`); the worker #267 `junior/*` push triggered run
  `25924783575`, which was the **5th persistent stuck-runner** (PR-4
  ×2 + PR-5 ×1 + PR-6 ×1 + PR-2 ×1 — `in_progress` frozen since
  creation, zero job progress). Cancelled per advisor-orchestrator.md
  §5.2 (advisor-laptop fallback, user-authorised "Go with C"
  2026-05-15 + overnight autonomy + sequential lane — covers PR-2).
  State the advisor-laptop local `cargo-check.bat --workspace
  --features full` exit code + `cargo-test.bat --workspace --features
  full --no-run` (test-target compile) exit code + DQ entry id +
  log paths. (Submodule `crates/email/translations` was init'd
  pre-emptively per L6 so no `lemmy_email build.rs read_dir`
  failure recurs.)

## §3 Required reading

- `.claude/commands/bm/bm-pr.md` (operational script — Phase 1→7)
- `.claude/rules/branch-manager.md` (file-ownership; autonomy table)
- `.claude/rules/phase-branch.md` (PR into governance-v0, not main; not draft)
- `.claude/rules/gh-pr-fork-target.md` (`--repo barrie-cork/lemmy` mandatory)
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md` (PR-2 of 5 — serial Step-2b; PR-1 is the LAST; strict-gate)

## §4 Constraints

- **NEVER** touch `crates/**`, `migrations/**`, `tests/**`,
  `docs/brehon-law-inspired-network/**`.
- **NEVER** open PR into `main` — base `governance-v0`.
- **NEVER** open as draft (CR skips drafts).
- **NEVER** re-cut or force-push the branch.
- **NEVER** cherry-pick this brief onto `chore/refactor-toctou`
  (L5 — PR #129 CONFLICTING root cause). The brief is referenced
  by path; it stays on governance-v0 only.
- `--repo barrie-cork/lemmy` on EVERY `gh pr` subcommand.
- Do NOT post a PR comment / submit a review (Manual/ask per
  autonomy table — bm-pr only opens the PR; the triage digest
  comment is a separate user-gated step).
- Append to runlog `.claude/runlog/chore-refactor-toctou.md`
  (already exists — lane was prepared).
- Return PR number + URL in the final summary.

## §5 Concurrency note

Sequential lane — PR-4 (valid-from) merged (#128, strict-gate 1/5),
PR-5 (diesel-errors) merged (#129, strict-gate 2/5), PR-6 (seed-tests)
merged (#130, strict-gate 3/5). PR-2 is the FIRST of the 2 remaining
serial dedicated lanes (Step-2b); PR-1 (e2e error-types reshape,
ranks 1/2/7/8, HIGHEST RISK) is the LAST (Step-2a, user-gated
explicitly before dispatch). On PR-2 merge → strict-gate 4/5; PR-2
done; surface PR-1 to user for explicit go-ahead, WAIT. Zero
cross-lane file overlap (PR-2 touches only
`crates/api/api_crud/src/governance/create_report.rs`; PR-1 touches
only `crates/server/tests/e2e.rs` — handover doc line 89). Do NOT
auto-start PR-1; do NOT proceed to v1 PRD planning until strict-gate
5/5 + the LAST-lane retro + user sign-off.
