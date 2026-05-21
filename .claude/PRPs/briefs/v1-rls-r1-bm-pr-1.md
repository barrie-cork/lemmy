---
phase: v1-rls-r1
role: bm-task
task: bm-pr
brief_n: 1
authored: 2026-05-21
plan: .claude/PRPs/plans/v1-rls-r1.plan.md
---

# [role:bm-task] v1-rls-r1 bm-pr — open PR for phase-v1-rls-r1 — see .claude/PRPs/briefs/v1-rls-r1-bm-pr-1.md

## §1 Role + dispatch

`[role:bm-task] v1-rls-r1 bm-pr — open PR for phase-v1-rls-r1 into governance-v0`

Actual create-task description (single line, <100 chars):

```text
[role:bm-task] v1-rls-r1 bm-pr — see .claude/PRPs/briefs/v1-rls-r1-bm-pr-1.md
```

## §2 Scope

Run `bm-pr` for `phase-v1-rls-r1`. v1-rls-r1 is **Recursive Learning System hardening Wave 1** — five top RLS-PMD review hardenings shipped across four tracks: (A) PMD foundation invariants + canonical-PMD SessionStart guard, (B) weekly-review Step 2c retro-harvest cadence + gitignored runtime journals + lane-bootstrap-checklist, (C) governance-log JSONL observability kinds doc + `retro_bypass` instrumentation in retro-check.sh + integration dogfood, (D) advisor-orchestrator §5.5 wiring + paired closure lessons + retro.

All 13 plan §13 tasks delivered + validated. **Zero Rust changes** — phase ships `.claude/**` rule/lesson/hook/spec + `docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md` + retro. Task 10 dogfood caught + fixed Task 7's `emit_retro_bypass_log` function-after-exit placement bug in the same commit.

**Phase branch:** `phase-v1-rls-r1`
**Tip:** `eaad3298d` (`docs(retro): v1-rls-r1 retrospective + 3 promoted lessons (task 13)`)
**Base:** `governance-v0`
**Repo:** `barrie-cork/lemmy`

**Retro:** SHIPPED at `.claude/PRPs/reports/v1-rls-r1-retro.md` (`eaad3298d`). Pre-bm-pr retro gate satisfies the file-presence check; plan §13 Task 13 names retro as terminal barrier (NOT pre-bm-pr blocker), so retro-gate logs INFO. CLAUDE.md user gate 6 (retro sign-off) is cleared by this retro at bm-pr time.

The PR body assembles from:
- the plan: `.claude/PRPs/plans/v1-rls-r1.plan.md`
- the commit log: `git log governance-v0..HEAD --oneline` (~32 commits)
- the retro: `.claude/PRPs/reports/v1-rls-r1-retro.md` (terminal § shape)
- the dogfood report: `.claude/PRPs/reports/v1-rls-r1-dogfood-2026-05-21.md`
- the §13 task table + complexity-score summary

**Open the PR only. Do NOT merge** (merge is a separate verb behind user gate 5/6).

## §3 Required reading

- `.claude/commands/bm/bm-pr.md` — bm-pr operational script (follow phases 1→7 verbatim)
- `.claude/rules/branch-manager.md` — BM autonomy bounds + file-ownership
- `.claude/rules/gh-pr-fork-target.md` — always `--repo barrie-cork/lemmy`
- `.claude/rules/phase-branch.md` — phase-branch + PR flow (base `governance-v0`, never `main`; not draft — CR skips drafts)
- `.claude/PRPs/plans/v1-rls-r1.plan.md` — plan → title + plan reference + §13 task table content
- `.claude/PRPs/reports/v1-rls-r1-retro.md` — retro deliverable (for §Completion report; replaces "Pending — retro deferred" with the actual retro reference)
- `.claude/PRPs/reports/v1-rls-r1-dogfood-2026-05-21.md` — dogfood report (cited in §Validation section)
- `.claude/PRPs/briefs/v1-federation-inbound-b-bm-pr-1.md` — **canonical-sibling brief** (per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first); mirror its §1-§7 shape verbatim adjusting for v1-rls-r1's smaller scope (13 tasks vs 9, zero fix-impl chains, no e2e rounds since zero Rust changes)

## §4 Constraints

- **`--repo barrie-cork/lemmy`** on every `gh` command (forks default to upstream `LemmyNet/lemmy` without it).
- Base: `governance-v0` (NOT `main`). Head: `phase-v1-rls-r1`.
- **Not draft** (CodeRabbit skips drafts per `phase-branch.md`).
- **PR title:** `Phase v1-rls-r1 — RLS hardening Wave 1: PMD invariants + canonical guard + retro_bypass JSONL + weekly Step 2c + paired closures (13 tasks, zero Rust)`
- **PR body** assembles from (in this order):
  - `## Summary` — one paragraph: v1-rls-r1 ships five RLS-PMD review hardenings as four tracks. Track A (PMD foundation) consolidates 5 invariants into `.claude/rules/pmd-invariants.md` (Task 2) + ships `.claude/hooks/pmd-canonical-guard.sh` (Task 3) for SessionStart cross-lane drift warning (WARN-not-FAIL) + adds MCP write-time embedding spec doc at `.claude/PRPs/specs/mcp-write-time-embedding.md` (Task 1, verification-deferred per PRECON-2). Track B (weekly cadence) inserts Step 2c retro-harvest sweep into `.claude/skills/weekly-review/SKILL.md` (Task 4, SURFACING-not-auto-promoting) + adds `.claude/harvest/` + `.claude/governance-log/` to `.gitignore` (Task 5) + authors `feedback_phase_lane_worktree_bootstrap_checklist.md` documenting per-worktree wiring (Task 6, DQ #301 dual-wire). Track C (retro observability) appends `emit_retro_bypass_log` to `.claude/hooks/retro-check.sh` fail-open path (Task 7) + creates `docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md` kind registry + 04 cross-link (Task 8) + integration dogfood (Task 10, caught + fixed Task 7 placement bug in-cycle). Track D (paired lessons + wire) ships `feedback_retro_bypass_governance_log.md` (Task 9) + wires `.claude/rules/advisor-orchestrator.md` §1 SessionStart guard bullet + new §5.5 retro-bypass-observability sub-section (Task 11) + paired closure lessons closing prior PENDING statuses (Task 12). Phase retro at `.claude/PRPs/reports/v1-rls-r1-retro.md` (Task 13) promotes 3 new lessons covering daemon-stress patterns observed during impl.
  - `## Plan reference` — `.claude/PRPs/plans/v1-rls-r1.plan.md`
  - `## Completion report` — `.claude/PRPs/reports/v1-rls-r1-retro.md` (terminal retro IS the completion report under this plan's shape)
  - `## Task table` — Tasks 1-13 with their commits + DQ entries (capture from the commit log):
    - Task 1 (Cohort A `[P]`) — `.claude/PRPs/specs/` README + MCP write-time embedding spec — `5b84ae751` — DQ #307 pass
    - Task 2 (Cohort A `[P]`) — `.claude/rules/pmd-invariants.md` (5 invariants) — `409f23960` — DQ #308 pass
    - Task 3 (Cohort A `[P]`) — `.claude/hooks/pmd-canonical-guard.sh` SessionStart guard — `59c5dd4ae` — DQ #309 pass
    - Task 4 (Cohort A `[P]`) — `.claude/skills/weekly-review/SKILL.md` Step 2c retro-harvest sweep — `595b9965a` — DQ #310 pass
    - Task 5 — `.gitignore` adds `.claude/harvest/` + `.claude/governance-log/` — `3b8140d93` — DQ #312 pass
    - Task 6 — `feedback_phase_lane_worktree_bootstrap_checklist.md` lesson — `1817bf9df` — DQ #313 pass (worker hung post-DQ-raise; advisor recovered via cancel + FF-merge + lane cargo)
    - Task 7 (Cohort A `[P]`) — `.claude/hooks/retro-check.sh` `emit_retro_bypass_log` append — `d55ddb797` — DQ #311 pass (initial placement bug fixed in Task 10 commit `3efa40b23`)
    - Task 8 — `docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md` + 04 cross-link — `fd3800175` — DQ #315 pass (worker hung post-DQ-raise; advisor recovered)
    - Task 9 — `feedback_retro_bypass_governance_log.md` paired lesson — `67f7bc885` — DQ #317 atomic pass (advisor-authored)
    - Task 10 — dogfood report + Task 7 placement fix — `3efa40b23` — DQ #320 atomic pass (advisor-driven dogfood; caught + fixed Task 7 regression in-cycle)
    - Task 11 — `.claude/rules/advisor-orchestrator.md` §1 SessionStart bullet + new §5.5 retro-bypass-observability sub-section (existing §5.5 Catch-fire renumbered to §5.6) — `a2ec16a3e` (bundled with Task 12) — DQ #318 atomic pass (advisor-authored)
    - Task 12 (`[P]`) — `feedback_pmd_canonical_guard_enforces_invariant.md` + `feedback_retro_harvest_weekly_cadence.md` paired closure lessons — `a2ec16a3e` (bundled with Task 11) — DQ #319 atomic pass (advisor-authored)
    - Task 13 — Retro + 3 promoted lessons (`feedback_advisor_authoring_under_daemon_stress.md`, `feedback_worker_hang_post_dq_raise.md`, `feedback_cross_lane_daemon_ref_contamination.md`) — `eaad3298d` — DQ #321 atomic pass (advisor-authored)
  - `## Validation` — Zero Rust changes; `bash scripts/brehon/cargo-check.sh --workspace --features full` exit 0 across **8** advisor-runs (one per task validate-pending-laptop entry mutation, except the Cohort A 5-way which consolidated into one cargo run since FILES YAML disjoint + zero Rust impact). All DoD §15.1-§15.8 gates exit 0. Zero Phase 2 e2e since zero Rust changes — no `crates/server/tests/e2e.rs` edits. **No billed GitHub Actions minutes** under Shape G SUSPENDED until 2026-06-01 per DQ #229; entire phase validated on lane laptop per `project_laptop_canonical_cargo_runner.md`.
  - `## Dogfood evidence` — `.claude/PRPs/reports/v1-rls-r1-dogfood-2026-05-21.md` summarises 3 integration probes (4.1 pmd-canonical-guard.sh canonical match + sentinel WARN; 4.6 weekly-review Step 2c surfaces ≥10 candidates from prior 30 days; 4.7 synthetic 3-attempt fail-open emits valid JSONL after Task 7 placement fix). All 3 probes PASS. The dogfood caught the Task 7 bug pre-merge.
  - `## Notable in-cycle finds` — (a) Cross-cohort DQ id collision (5 workers all picked id=307 forking from same phase tip); recovery via cherry-pick + atomic renumber; new lesson `feedback_cohort_dq_id_collision.md`. (b) Junior worker hang post-DQ-raise pattern (Tasks 6 + 8); recovery via cancel + FF-merge + lane cargo; new lesson `feedback_worker_hang_post_dq_raise.md`. (c) Cross-lane daemon-ref contamination (daemon-local phase-v1-rls-r1 reset to conformance-audit work 3+ times); recovery via detach + branch-f + checkout; new lesson `feedback_cross_lane_daemon_ref_contamination.md`. (d) Task 10 dogfood caught Task 7's function-after-exit placement; fix applied in same commit. All four findings + recoveries documented in `.claude/PRPs/reports/v1-rls-r1-retro.md`.
  - `## Commits` — `git log governance-v0..HEAD --oneline` output, one bullet per line (~32 commits — feat + chore(advisor) brief-authoring + chore(decision-queue) DQ mutations + 1 docs(retro)). Notably smaller than fed-in-b (~69 commits) due to zero Rust + zero fix-impl chains.
  - `## Closes` — `(none — plan-driven sub-phase; no GH issues referenced)`
- **Phase 1b (DQ historical-fail sweep):** run it. Sweep `kind: "validate-pending"` entries from `pending[]` whose `result` is in the failure enum. **Expected: no-op** — DQ pending=0 on tip `eaad3298d` (all DQ #307-#321 resolved). If sweep finds anything → STOP + `kind:"blocker"`.
- **Phase 1c (Phase 2 e2e gate):** the plan does NOT touch `crates/server/tests/e2e.rs` (zero Rust changes), so the gate does NOT fire. Skip phase 1c silently.
- Write `.claude/PRPs/reviews/pr-<N>-findings.yaml` shell per `.claude/PRPs/reviews/SCHEMA.md` for the later CR-triage cycle.
- Append the Phase 6 runlog entry to `.claude/runlog/v1-rls-r1-runlog.md` IF a phase-branch runlog exists (it MAY not — v1-rls-r1 was advisor-orchestrated without a dedicated runlog beyond brief commits) AND to `.claude/runlog/bm-runlog.md` per `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md` (write the bm-pr "PR opened" runlog to the phase-branch runlog or skip if absent; the post-merge "COMPLETE" runlog goes on trunk per the L14 fix REVISED 2026-05-18 ordering).
- Do **NOT** post a Telegram ping (advisor handles outbound; no ping this phase unless the user asks).
- Do **NOT** touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**` (BM file-ownership HARD boundary per `branch-manager.md`).
- Do **NOT** mutate any DQ entry beyond Phase 1b sweep (no-op expected).

## §5 Validation gates

- Phase 1 pre-conditions: branch is `phase-v1-rls-r1`, pushed, clean tree, ~32 commits ahead of `governance-v0`, no existing PR (or skip to edit), retro-gate INFO (plan §13 Task 13 names retro as terminal barrier, not pre-bm-pr; retro file IS present at `eaad3298d`).
- Phase 1b: no-op sweep (expected; if not, STOP + blocker).
- Phase 1c: skip (no `crates/server/tests/e2e.rs` edits this phase).
- Phase 2: PR body assembles cleanly from §4 ingredients.
- Phase 3: `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-v1-rls-r1 --title "<§4 title>" --body-file <tempfile>` returns PR URL.
- Phase 4: PR body matches the assembled content (gh pr view --json body).
- Phase 5: findings.yaml shell written at `.claude/PRPs/reviews/pr-<N>-findings.yaml` per SCHEMA.md.
- Phase 6: runlog appended on `bm-runlog.md` per L14 REVISED ordering (phase-branch runlog may not exist; create lightly if so).
- Phase 7: return PR URL + "Next suggested: bm-poll-cr after ~5-15min for CodeRabbit findings".

## §6 Expected output (return to advisor)

```text
## bm-pr complete — PR #<N> opened on phase-v1-rls-r1

**PR URL:** https://github.com/barrie-cork/lemmy/pull/<N>
**Title:** Phase v1-rls-r1 — RLS hardening Wave 1: ...
**Base:** governance-v0
**Head:** phase-v1-rls-r1 @ eaad3298d
**Body length:** ~<N> lines (~32 commits in §Commits section)
**findings.yaml:** .claude/PRPs/reviews/pr-<N>-findings.yaml (shell only — counters all 0)
**Runlog entries:** trunk bm-runlog.md per L14 ordering (phase-branch runlog created lightly if absent)
**Phase 1b sweep:** no-op (DQ pending=0, expected)
**Phase 1c e2e gate:** skipped (no crates/server/tests/e2e.rs edits this phase)
**Next suggested:** bm-poll-cr after ~5-15min for CodeRabbit findings
```

## §7 Why this brief differs from the canonical sibling

Mirrors `.claude/PRPs/briefs/v1-federation-inbound-b-bm-pr-1.md` schema verbatim per advisor-orchestrator.md §3.6 canonical-schema-first gate. Differences:

- **Scope:** v1-rls-r1 is ~1/3 the size (13 tasks vs 9 — but smaller per-task; zero Rust vs all-Rust; zero fix-impl chains vs 7; zero e2e rounds vs 2).
- **Task table:** 13-row vs 10-row; each row notes which Cohort A members vs serial chain vs advisor-authored Tasks 9-13.
- **Phase-2 e2e section:** OMITTED entirely (zero Rust changes; no `crates/server/tests/e2e.rs` edits; gate does not fire).
- **Dogfood evidence section:** ADDED (v1-rls-r1 Task 10's three integration probes are the integration-test equivalent for a meta-work phase).
- **Notable in-cycle finds section:** ADDED (v1-rls-r1 documented 4 daemon-stress recoveries in-cycle; canonical sibling had no such interventions).
- **Retro:** SHIPPED pre-bm-pr per plan §13 Task 13 terminal-barrier discipline (canonical sibling deferred retro to post-merge).
- **L14 ordering:** identical to canonical sibling (write bm-pr "PR opened" entry on bm-runlog.md trunk; COMPLETE entry post-merge).
- **Zero billed Actions minutes:** identical claim (Shape G SUSPENDED until 2026-06-01 per DQ #229).

---

_Brief author: advisor session (lane-dedicated CWD `C:/Users/barri/Developer/brehon-fork-rls-r1` on `phase-v1-rls-r1`). Brief committed on `phase-v1-rls-r1` (daemon BM worker reads from this branch). User-gate-5 (merge confirm — bm-pr dispatch approval) cleared 2026-05-21 via user reply "proceed"._
