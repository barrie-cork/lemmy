---
phase: v1-federation-inbound-b
role: bm-task
task: bm-pr
brief_n: 1
authored: 2026-05-20
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
---

# [role:bm-task] v1-federation-inbound-b bm-pr — open PR for phase-v1-federation-inbound-b — see .claude/PRPs/briefs/v1-federation-inbound-b-bm-pr-1.md

## §1 Role + dispatch

`[role:bm-task] v1-federation-inbound-b bm-pr — open PR for phase-v1-federation-inbound-b into governance-v0`

Actual create-task description (single line, <100 chars):

```
[role:bm-task] v1-federation-inbound-b bm-pr — see .claude/PRPs/briefs/v1-federation-inbound-b-bm-pr-1.md
```

## §2 Scope

Run `bm-pr` for `phase-v1-federation-inbound-b`. fed-in-b is the **HTTP-path enforcement layer for federated governance** — wrap-pattern (`wrap_governance_inbound`) over Phase-6 receive functions adding **per-peer trust gate (Allowlist/Blocklist/Unknown deny) + per-actor rate limit + replay-nonce gate + payload-cap gate** to incoming federation activities, plus a **periodic replay-cleanup cron**, plus a **handler-e2e test module** asserting all gate semantics.

All 9 plan §13 tasks delivered + validated; 4 fix-impl chains landed; 2 Phase-2 e2e rounds (round-2 CLEAN). Task 0 (pre-flight harness audit) was read-only no-commit by design.

**Phase branch:** `phase-v1-federation-inbound-b`
**Tip:** `90878d681` (`chore(decision-queue): advisor mutated DQ #290 — pass validate-pending-laptop-e2e`)
**Base:** `governance-v0`
**Repo:** `barrie-cork/lemmy`

**No completion report, no retro yet** — the v1-federation-inbound-b plan defers retro to **post-merge (user gate 6)**, same shape as v1-AD-e. The PR body assembles from:
- the plan: `.claude/PRPs/plans/v1-federation-inbound-b.plan.md`
- the commit log: `git log governance-v0..HEAD --oneline` (~69 commits)
- the runlog: `.claude/runlog/v1-federation-inbound-b-runlog.md` (bm-cut entry only — bm-pr Phase 6 appends "PR opened" entry)
- the §13 task table + the 2-round e2e summary below

**Open the PR only. Do NOT merge** (merge is a separate verb behind user gate 5/6).

## §3 Required reading

- `.claude/commands/bm/bm-pr.md` — bm-pr operational script (follow phases 1→7 verbatim)
- `.claude/rules/branch-manager.md` — BM autonomy bounds + file-ownership
- `.claude/rules/gh-pr-fork-target.md` — always `--repo barrie-cork/lemmy`
- `.claude/rules/phase-branch.md` — phase-branch + PR flow (base `governance-v0`, never `main`; not draft — CR skips drafts)
- `.claude/PRPs/plans/v1-federation-inbound-b.plan.md` — plan → title + plan reference + §13 task table content
- `.claude/runlog/v1-federation-inbound-b-runlog.md` — bm-cut entry; append "PR opened" entry per Phase 6
- `.claude/PRPs/briefs/v1-AD-e-bm-pr-1.md` — **canonical-sibling brief** (per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first); mirror its §1-§5 shape verbatim adjusting for fed-in-b's larger scope (9 tasks vs 6, 4 fix-impl chains vs 0, 2 e2e rounds vs 1)

## §4 Constraints

- **`--repo barrie-cork/lemmy`** on every `gh` command (forks default to upstream `LemmyNet/lemmy` without it).
- Base: `governance-v0` (NOT `main`). Head: `phase-v1-federation-inbound-b`.
- **Not draft** (CodeRabbit skips drafts per `phase-branch.md`).
- **PR title:** `Phase v1-federation-inbound-b — HTTP-path enforcement for federated governance (wrap + per-peer trust + per-actor rate + replay nonce + payload cap + replay-cleanup cron + handler-e2e)`
- **PR body** assembles from (in this order):
  - `## Summary` — one paragraph: fed-in-b ships `wrap_governance_inbound` as the unified HTTP-path enforcement wrapper for the 3 Phase-6 receive functions (`receive_remote_sanction_notice`, `receive_remote_trust_attestation`, `receive_remote_moderation_label`); enforces 4 gates inline (per-peer trust via `federation_peer.trust_level` Allowlist/Blocklist/Unknown-deny + per-actor hourly rate via `federation_inbox_actor_rate` keyed bucket + replay-nonce via `federation_inbox_nonce` with retention window + payload-cap via per-activity-type config keys); adds `scheduled_tasks::fed_replay_cleanup` cron (default 60min interval, 7d retention, `BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB` env-var test-disable); adds 5 handler-e2e tests in `mod v1_federation_inbound_b_fixtures` (blocked/rate/replay/happy/label) + augments Phase-6 `sanction_notice_round_trip` with `FederationPeerTrust::Allowlisted` fixture.
  - `## Plan reference` — `.claude/PRPs/plans/v1-federation-inbound-b.plan.md`
  - `## Completion report` — `Pending — retro deferred to post-merge (user gate 6)`
  - `## Task table` — Tasks 0-9 with their commits + DQ entries (capture from the commit log):
    - Task 0 — pre-flight harness audit — (no commit; read-only by plan design) — PASS
    - Task 1 (Cohort A `[P]`) — federation_peer schema + read accessor — `<sha>` — DQ #280 pass
    - Task 2 (Cohort A `[P]`) — federation_inbox_actor_rate schema + helper — `<sha>` — DQ #281 pass
    - Task 3 (Cohort A `[P]`) — federation_inbox_nonce schema + delete_older_than — `<sha>` — DQ #282 pass
    - Task 4 BARRIER — `wrap_governance_inbound` + 5 check helpers + `log_inbox_drop` + `GovernanceInboundActivity` trait + storage-cap + `receive_remote_moderation_label` shape (commits `53f8ba3bd` impl + finalize + `b9691c0ab` fix-impl-1 + `f01a1d44e` fix-impl-2 + `8b04e69a6` fix-impl-3) — DQ #283 pass
    - Task 5 (Cohort B serial-per-cap-≤2) — `publish_sanction_notice` wrap + GovernanceInboundActivity trait-default rate gate (`92605e907` impl + `3bd2cfa4e` fix-impl-4 HRTB `<'a>`) — DQ #285 pass
    - Task 6 (Cohort B serial-per-cap-≤2) — `publish_trust_attestation` wrap + per-actor rate override (`ec3b8643f` impl + `d32d2c3f2` fix-impl-5 .into() drop + `9656b4bc7` fix-impl-6 const-hoist) — DQ #286 pass
    - Task 7 (Cohort B serial-per-cap-≤2) — `publish_label` stub-fill receive (`fe9effd6b`) — DQ #287 pass (first-try)
    - Task 8 BARRIER — `scheduled_tasks` replay-cleanup cron wiring (`36d7bcd03`) — DQ #288 pass (first-try)
    - Task 9 BARRIER — `e2e.rs` Phase-6 fixture Allowlist + new `mod v1_federation_inbound_b_fixtures` (5 tests) (`d2e002cb0` impl + `7247d8126` fix-impl-7 test-fixture) — DQ #289 pass (impl §15 first-try; e2e round-2 pass after fix-impl-7)
  - `## Phase-2 e2e validation` —
    - **Round 1** (tip `95ee8274f`, 35m50s): `test result: FAILED. 100 passed; 2 failed; 5 ignored` — failures: `sanction_notice_round_trip` (Phase-6 regression: Hunk-1 fixture insert on wrong-DB), `v1_federation_inbound_b_fixtures::per_peer_rate_limit_returns_429` (Hunk-2 append-history INSERT collided with migration seed).
    - **fix-impl-7** (commit `7247d8126`, finalize `4efce35c8`): 1 file (`crates/server/tests/e2e.rs`) 2 hunks ~18 lines net; Hunk-1 = `Instance::read_or_create(&mut context_b.pool(), "instance-a.test")` mirrors line 5225 pattern; Hunk-2 = `UPDATE governance_config SET value_int=2 WHERE scope/key` instead of raw INSERT (per migration comment 'Do NOT use (scope, key) as conflict target').
    - **Round 2** (tip `4efce35c8`, 34m32s): `test result: ok. 102 passed; 0 failed; 5 ignored; finished in 2071.77s` — both fix-impl-7 hunks confirmed; all 4 other v1_federation_inbound_b fixtures (allowlisted_happy_path / blocklisted_peer_returns_403 / replayed_activity_returns_409 / moderation_label_handler_persists_and_logs) still passing; zero regression to the 100 other Phase-1+ tests.
  - `## Commits` — `git log governance-v0..HEAD --oneline` output, one bullet per line (~69 commits — feat + fix + chore(advisor) brief-authoring + chore(decision-queue) DQ mutations + chore(merge) finalize-merges; LARGEST PHASE BRANCH TO DATE).
  - `## Validation` — DoD gates exit 0 across every §13 task; §15 fail-rate inflection: Tasks 4/5/6 each needed 1+ fix-impl; Tasks 7/8/9 first-try §15-green (validates §2.4 worker-side pre-push cargo-check discipline per `feedback_fix_impl_pre_push_cargo_check.md`). Phase-2 e2e LOCAL twice (`feedback_default_local_testing.md` + `project_laptop_canonical_cargo_runner.md`); zero billed Actions minutes for fed-in-b validation under Shape G SUSPENDED until 2026-06-01 per DQ #229.
  - `## Closes` — extract any `closes #N` / `fixes #N` from commit messages (likely none — plan-driven sub-phase; if none, write `(none — plan-driven sub-phase)`).
- **Phase 1b (DQ historical-fail sweep):** run it. Sweep `kind: "validate-pending"` entries from `pending[]` whose `result` is in the failure enum. **Expected: no-op** — DQ pending=0 on tip `90878d681` (all DQ #280-#290 resolved). If sweep finds anything → STOP + `kind:"blocker"`.
- **Phase 1c (Phase 2 e2e gate):** the plan touches `crates/server/tests/e2e.rs` so the gate fires. It is **satisfied** — DQ #290 (`kind: validate-pending-laptop-e2e`, `branch: phase-v1-federation-inbound-b`, `result: pass`, 102/0/5) is in `resolved[]`. Gate Python passes. Do NOT surface user gate 4 — was already cleared at advisor `AskUserQuestion` 2026-05-20 (LOCAL chosen) and round-2 returned E2E_EXIT_0.
- Write `.claude/PRPs/reviews/pr-<N>-findings.yaml` shell per `.claude/PRPs/reviews/SCHEMA.md` for the later CR-triage cycle.
- Append the Phase 6 runlog entry to `.claude/runlog/v1-federation-inbound-b-runlog.md` (`## bm: PR opened — <ISO>`) AND to `.claude/runlog/bm-runlog.md` per `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md` (write the bm-pr "PR opened" runlog to the phase-branch runlog; the post-merge "COMPLETE" runlog goes on trunk per the L14 fix REVISED 2026-05-18 ordering).
- Do **NOT** post a Telegram ping (advisor handles outbound; no ping this phase unless the user asks).
- Do **NOT** touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**` (BM file-ownership HARD boundary per `branch-manager.md`).
- Do **NOT** mutate any DQ entry beyond Phase 1b sweep (no-op expected).

## §5 Validation gates

- Phase 1 pre-conditions: branch is `phase-v1-federation-inbound-b`, pushed, clean tree, >0 commits ahead of `governance-v0`, no existing PR (or skip to edit), retro-gate INFO (plan defers to post-merge).
- Phase 1b: no-op sweep (expected; if not, STOP + blocker).
- Phase 1c: Phase 2 e2e gate satisfied via DQ #290 pass; gate Python passes silently.
- Phase 2: PR body assembles cleanly from §4 ingredients.
- Phase 3: `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-v1-federation-inbound-b --title "<§4 title>" --body-file <tempfile>` returns PR URL.
- Phase 4: PR body matches the assembled content (gh pr view --json body).
- Phase 5: findings.yaml shell written at `.claude/PRPs/reviews/pr-<N>-findings.yaml` per SCHEMA.md.
- Phase 6: runlog appended on BOTH phase-branch runlog AND `bm-runlog.md` per L14 REVISED ordering.
- Phase 7: return PR URL + "Next suggested: bm-poll-cr after ~5-15min for CodeRabbit findings".

## §6 Expected output (return to advisor)

```
## bm-pr complete — PR #<N> opened on phase-v1-federation-inbound-b

**PR URL:** https://github.com/barrie-cork/lemmy/pull/<N>
**Title:** Phase v1-federation-inbound-b — HTTP-path enforcement for federated governance (...)
**Base:** governance-v0
**Head:** phase-v1-federation-inbound-b @ 90878d681
**Body length:** ~<N> lines (~<N> commits in §Commits section)
**findings.yaml:** .claude/PRPs/reviews/pr-<N>-findings.yaml (shell only — counters all 0)
**Runlog entries:** phase-branch + trunk bm-runlog.md per L14 ordering
**Phase 1b sweep:** no-op (DQ pending=0, expected)
**Phase 1c e2e gate:** satisfied (DQ #290 result:pass, 102/0/5)
**Next suggested:** bm-poll-cr after ~5-15min for CodeRabbit findings
```

## §7 Why this brief differs from the canonical sibling

Mirrors `.claude/PRPs/briefs/v1-AD-e-bm-pr-1.md` schema verbatim per advisor-orchestrator.md §3.6 canonical-schema-first gate. Differences:
- **Scope:** fed-in-b is ~3× larger (9 tasks vs 6, 4 fix-impl chains vs 0, 2 e2e rounds vs 1).
- **Task table:** 10-row vs 6-row; each row carries impl commit + fix-impl commits if any.
- **Phase-2 e2e section:** two-round summary explicitly cites fix-impl-7's role; v1-AD-e had one-round green.
- **Validation section:** explicitly cites the §15 fail-rate inflection between Tasks 4/5/6 and Tasks 7/8/9 (validates the §2.4 worker-side pre-push discipline — a load-bearing data point for fed-in-b retro carry-forward).
- **L14 ordering:** writes runlog entries on BOTH phase-branch runlog AND `bm-runlog.md` per REVISED 2026-05-18 ordering (`feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md`); the COMPLETE entry on trunk comes POST-merge.

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`). Brief committed on `governance-v0` only — bm-pr Junior reads it from there and operates on the phase branch via the daemon worktree. User-gate-5 (merge confirm — bm-pr dispatch approval) cleared 2026-05-20 via AskUserQuestion "APPROVE bm-pr"._
