---
phase: v1-federation-inbound-c
role: bm-task
task: bm-pr
brief_n: 1
authored: 2026-05-22
plan: .claude/PRPs/plans/v1-federation-inbound-c.plan.md
---

# [role:bm-task] v1-federation-inbound-c bm-pr — open PR for phase-v1-federation-inbound-c — see .claude/PRPs/briefs/v1-federation-inbound-c-bm-pr-1.md

## §1 Role + dispatch

`[role:bm-task] v1-federation-inbound-c bm-pr — open PR for phase-v1-federation-inbound-c into governance-v0`

Actual create-task description (single line, <100 chars):

```
[role:bm-task] v1-fed-in-c bm-pr — see .claude/PRPs/briefs/v1-federation-inbound-c-bm-pr-1.md
```

## §2 Scope

Run `bm-pr` for `phase-v1-federation-inbound-c`. fed-in-c is the **reader-side append-history fix slice** — adds `.order_by(governance_config::valid_from.desc())` to the two federation-inbound config reader call-sites (`get_inbound_config_int` in `inbox.rs` + the helper-duplicated mirror in `publish_trust_attestation.rs`) so the most-recent `valid_from` row wins under append-history semantics, plus a single e2e regression test (`appended_config_override_takes_effect_returns_429`) that asserts the override behaviour under live INSERT-with-newer-`valid_from`.

All 4 plan §13 tasks delivered + validated; 0 fix-impl chains; 1 Phase-2 e2e round (CLEAN first-try 103/0/5). Task 0 not present (plan is scope-(a)-only; small + mechanical).

**Phase branch:** `phase-v1-federation-inbound-c`
**Tip:** `dadda87e3` (`chore(decision-queue): migrate DQ #326 pending→resolved (audit-lane stale leftover)`)
**Base:** `governance-v0`
**Repo:** `barrie-cork/lemmy`

**Retro IS landed (Task 4 = retro authoring per plan §13):** at `ab877c889` + finalize-merge `3a5caa323`. The retro file is `.claude/PRPs/reports/v1-federation-inbound-c-retro.md` (496 lines, 6 H2 sections, 4 recurrence-class observations A-D, per-task complexity scores, §15.6 21-box cross-cutting verification all ✓, 5-item carry-forward). The PR body assembles from:
- the plan: `.claude/PRPs/plans/v1-federation-inbound-c.plan.md`
- the retro: `.claude/PRPs/reports/v1-federation-inbound-c-retro.md` (in-tree on phase branch)
- the commit log: `git log governance-v0..HEAD --oneline` (26 commits)
- the runlog: `.claude/runlog/v1-federation-inbound-c-runlog.md` (bm-cut entry + advisor notes — bm-pr Phase 6 appends "PR opened" entry)
- the §13 task table + the 1-round e2e summary below

**Open the PR only. Do NOT merge** (merge is a separate verb behind user gate 5/6).

## §3 Required reading

- `.claude/commands/bm/bm-pr.md` — bm-pr operational script (follow phases 1→7 verbatim)
- `.claude/rules/branch-manager.md` — BM autonomy bounds + file-ownership
- `.claude/rules/gh-pr-fork-target.md` — always `--repo barrie-cork/lemmy`
- `.claude/rules/phase-branch.md` — phase-branch + PR flow (base `governance-v0`, never `main`; not draft — CR skips drafts)
- `.claude/PRPs/plans/v1-federation-inbound-c.plan.md` — plan → title + plan reference + §13 task table content
- `.claude/PRPs/reports/v1-federation-inbound-c-retro.md` — retro (in-tree at phase tip; reference under "## Retro" in PR body)
- `.claude/runlog/v1-federation-inbound-c-runlog.md` — bm-cut entry + advisor entries; append "PR opened" entry per Phase 6
- `.claude/PRPs/briefs/v1-federation-inbound-b-bm-pr-1.md` — **canonical-sibling brief** (per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first); mirror its §1-§5 shape verbatim adjusting for fed-in-c's smaller, cleaner scope (4 tasks vs 9, 0 fix-impl vs 4, 1 e2e round CLEAN vs 2)

## §4 Constraints

- **`--repo barrie-cork/lemmy`** on every `gh` command (forks default to upstream `LemmyNet/lemmy` without it).
- Base: `governance-v0` (NOT `main`). Head: `phase-v1-federation-inbound-c`.
- **Not draft** (CodeRabbit skips drafts per `phase-branch.md`).
- **PR title:** `Phase v1-federation-inbound-c — reader-side append-history fix for federated governance inbound (.order_by(valid_from.desc()) + e2e regression)`
- **PR body** assembles from (in this order):
  - `## Summary` — one paragraph: fed-in-c ships a small reader-side correctness fix to federation-inbound governance config consumers. Both reader call-sites (`get_inbound_config_int` in `crates/apub/activities/src/governance/inbox.rs` + the helper-duplicated mirror in `crates/apub/activities/src/governance/publish_trust_attestation.rs`) now apply `.order_by(governance_config::valid_from.desc())` before `.first(...)` on the Diesel-typed base-table chain, so the most-recent row wins under the append-history semantics (per the canonical reader pattern at `config.rs:740-769`). Adds one e2e regression test `appended_config_override_takes_effect_returns_429` in `mod v1_federation_inbound_b_fixtures` asserting that an INSERT with a newer `valid_from` row immediately overrides per-peer rate limits. Scope is (a) reader-side only per fed-in-b retro §3 — the (b) Copilot DoS-hardening family is deferred to a future sub-phase (carry-forward decision-point in retro). Zero migrations, zero new helpers (PRECON-3 honoured), zero changes to `governance_config.rs` itself. 4 §13 tasks (Tasks 1+2 [P] cohort + Task 3 e2e + Task 4 retro); 0 fix-impl cycles; 1 Phase-2 e2e round CLEAN (103/0/5).
  - `## Plan reference` — `.claude/PRPs/plans/v1-federation-inbound-c.plan.md`
  - `## Retro` — `.claude/PRPs/reports/v1-federation-inbound-c-retro.md` (in-tree; 496 lines, 6 H2 sections incl. 4 recurrence-class observations + 5-item carry-forward)
  - `## Task table` — Tasks 1-4 with their commits + DQ entries (capture from the commit log):
    - Task 1 (Cohort 1 `[P]`) — `inbox.rs` `.order_by(valid_from.desc())` add in `get_inbound_config_int` — `0ed04fc71` — DQ #328 pass (validate-pending-laptop)
    - Task 2 (Cohort 1 `[P]`) — `publish_trust_attestation.rs` `.order_by(valid_from.desc())` add in `actor_cap` block — `01ed9241f` — DQ #329 pass (validate-pending-laptop)
    - Task 3 (Cohort 2 single-member, post-DQ #338) — `e2e.rs` new `appended_config_override_takes_effect_returns_429` test + workaround-comment swap — `9c7c43aff` — DQ #339 pass (validate-pending-laptop: check + clippy + test --no-run)
    - Task 4 (retro authoring) — `.claude/PRPs/reports/v1-federation-inbound-c-retro.md` (496 lines, 6 H2 sections) — `ab877c889`
  - `## Phase-2 e2e validation` —
    - **Round 1** (tip `e06e918e2`, 36m11s LOCAL): `test result: ok. 103 passed; 0 failed; 5 ignored; finished in 2171.42s` — new `appended_config_override_takes_effect_returns_429` PASS; all 6 v1_federation_inbound_b sibling fixtures (per_peer_rate_limit_returns_429 / allowlisted_happy_path / blocklisted_peer_returns_403 / replayed_activity_returns_409 / moderation_label_handler_persists_and_logs + this new one) PASS; 5 ignored are pre-existing v0-polish TODOs (GH #42/#43/#45) NOT phase regressions; zero regression to the 97 other Phase-1+ tests. Per DQ #340 (`kind: validate-pending-laptop-e2e`, `result: pass`, advisor-laptop 2026-05-21T22:33Z).
  - `## Commits` — `git log governance-v0..HEAD --oneline` output, one bullet per line (~26 commits — 3 feat + 1 retro + chore(advisor) brief-authoring + chore(decision-queue) DQ mutations + chore(merge) finalize-merges + 1 forward-merge from gov-v0 (108 commits subsumed)).
  - `## Validation` — DoD gates exit 0 across every §13 task; **zero §G4 classifier firings** (all 3 impl tasks first-try §15-green — confirms plan §5.1 complexity score 2/10 was accurate); §15.6 21-box cross-cutting verification walked at retro time (all ✓); the new federation-mod-roots `#![deny(clippy::disallowed_methods)]` attribute (landed via gov-v0 forward-merge from brehon-conformance-audit PR #141) produced **zero firings** on the new fed-in-c code — validates canonical-reader-pattern (`.order_by + .first`) is not on the disallowed list. Phase-2 e2e LOCAL (`feedback_default_local_testing.md` + `project_laptop_canonical_cargo_runner.md`); zero billed Actions minutes under Shape G SUSPENDED until 2026-06-01 per DQ #229.
  - `## Closes` — extract any `closes #N` / `fixes #N` from commit messages (likely none — plan-driven sub-phase; if none, write `(none — plan-driven sub-phase; carry-forward items tracked in retro)`).
- **Phase 1b (DQ historical-fail sweep):** run it. Sweep `kind: "validate-pending"` entries from `pending[]` whose `result` is in the failure enum. **Expected: no-op** — DQ pending=0 on tip `dadda87e3` (DQ #326 migrated to resolved[] at this commit; the leftover from cross-lane forward-merge `7f02254fe` was fully-answered and only stuck in pending[] as an array-transition bookkeeping miss). If sweep finds anything → STOP + `kind:"blocker"`.
- **Phase 1c (Phase 2 e2e gate):** the plan touches `crates/server/tests/e2e.rs` so the gate fires. It is **satisfied** — DQ #340 (`kind: validate-pending-laptop-e2e`, `branch: phase-v1-federation-inbound-c`, `result: pass`, 103/0/5) is in `resolved[]`. Gate Python passes. Do NOT surface user gate 4 — was already cleared at advisor `AskUserQuestion` 2026-05-21 (LOCAL chosen) and round-1 returned E2E_EXIT_0.
- Write `.claude/PRPs/reviews/pr-<N>-findings.yaml` shell per `.claude/PRPs/reviews/SCHEMA.md` for the later CR-triage cycle.
- Append the Phase 6 runlog entry to `.claude/runlog/v1-federation-inbound-c-runlog.md` (`## bm: PR opened — <ISO>`) AND to `.claude/runlog/bm-runlog.md` per `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md` (write the bm-pr "PR opened" runlog to the phase-branch runlog; the post-merge "COMPLETE" runlog goes on trunk per the L14 fix REVISED 2026-05-18 ordering).
- Do **NOT** post a Telegram ping (advisor handles outbound; no ping this phase unless the user asks).
- Do **NOT** touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**` (BM file-ownership HARD boundary per `branch-manager.md`).
- Do **NOT** mutate any DQ entry beyond Phase 1b sweep (no-op expected).
- **PRE-PUSH MANDATE (DQ #338 daemon-bug mitigation):** worker MUST pre-push the worker branch (`git push origin HEAD:$(git branch --show-current)`) after the runlog + findings.yaml commit, BEFORE the daemon's finalize step runs. Advisor will manually finalize-merge from `origin/junior/<worker-branch>` per the cohort-1 + Task-3 + Task-4 working template established this phase.

## §5 Validation gates

- Phase 1 pre-conditions: branch is `phase-v1-federation-inbound-c`, pushed, clean tree, >0 commits ahead of `governance-v0`, no existing PR (or skip to edit), retro-gate satisfied (retro file present in-tree at `.claude/PRPs/reports/v1-federation-inbound-c-retro.md` with 6 H2 sections).
- Phase 1b: no-op sweep (expected; if not, STOP + blocker).
- Phase 1c: Phase 2 e2e gate satisfied via DQ #340 pass; gate Python passes silently.
- Phase 2: PR body assembles cleanly from §4 ingredients.
- Phase 3: `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-v1-federation-inbound-c --title "<§4 title>" --body-file <tempfile>` returns PR URL.
- Phase 4: PR body matches the assembled content (`gh pr view --json body`).
- Phase 5: findings.yaml shell written at `.claude/PRPs/reviews/pr-<N>-findings.yaml` per SCHEMA.md.
- Phase 6: runlog appended on BOTH phase-branch runlog AND `bm-runlog.md` per L14 REVISED ordering.
- Phase 7: return PR URL + "Next suggested: bm-poll-cr after ~5-15min for CodeRabbit findings".

## §6 Expected output (return to advisor)

```
## bm-pr complete — PR #<N> opened on phase-v1-federation-inbound-c

**PR URL:** https://github.com/barrie-cork/lemmy/pull/<N>
**Title:** Phase v1-federation-inbound-c — reader-side append-history fix for federated governance inbound (...)
**Base:** governance-v0
**Head:** phase-v1-federation-inbound-c @ dadda87e3
**Body length:** ~<N> lines (~26 commits in §Commits section)
**findings.yaml:** .claude/PRPs/reviews/pr-<N>-findings.yaml (shell only — counters all 0)
**Runlog entries:** phase-branch + trunk bm-runlog.md per L14 ordering
**Phase 1b sweep:** no-op (DQ pending=0, expected after #326 migration at dadda87e3)
**Phase 1c e2e gate:** satisfied (DQ #340 result:pass, 103/0/5)
**Next suggested:** bm-poll-cr after ~5-15min for CodeRabbit findings
```

## §7 Why this brief differs from the canonical sibling

Mirrors `.claude/PRPs/briefs/v1-federation-inbound-b-bm-pr-1.md` schema verbatim per advisor-orchestrator.md §3.6 canonical-schema-first gate. Differences:
- **Scope:** fed-in-c is ~3× smaller (4 tasks vs 9, 0 fix-impl chains vs 4, 1 e2e round CLEAN vs 2).
- **Retro state:** fed-in-c retro IS in-tree at phase tip (Task 4 was retro authoring per plan §13); fed-in-b deferred retro to post-merge. PR body has a `## Retro` section vs fed-in-b's `## Completion report — Pending`.
- **Task table:** 4-row vs 10-row; each row carries impl commit + (Task 4) retro commit; no fix-impl commits anywhere.
- **Phase-2 e2e section:** single-round-clean summary; fed-in-b had two-round + fix-impl-7 narrative.
- **Validation section:** explicitly cites zero §G4 firings, confirms plan §5.1 complexity score 2/10 was accurate, notes the gov-v0-forward-merged federation-mod-roots `disallowed_methods` deny produced zero firings on new fed-in-c code.
- **L14 ordering:** writes runlog entries on BOTH phase-branch runlog AND `bm-runlog.md` per REVISED 2026-05-18 ordering (`feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md`); the COMPLETE entry on trunk comes POST-merge.
- **PRE-PUSH MANDATE added (§4 last bullet):** DQ #338 daemon-bug mitigation. fed-in-b's bm-pr brief predates DQ #338; fed-in-c's must include it explicitly so the BM worker pre-pushes + advisor manual finalize-merges from `origin/junior/<worker-branch>`. Working template from cohort-1 + Task-3 + Task-4 finalize-merges this phase.

---

_Brief author: advisor session (lane-dedicated CWD `C:/Users/barri/Developer/brehon-fork-fed-in-c` on `phase-v1-federation-inbound-c`). Brief committed on phase branch (NOT governance-v0 — multi-lane-worktree.md §"Layout" + bm-pr brief author convention for this lane), bm-pr Junior reads it from there and operates on the phase branch via the daemon worktree. User-gate-6 (retro sign-off — bm-pr dispatch approval) cleared 2026-05-22 via AskUserQuestion "Approve — queue bm-pr"._
