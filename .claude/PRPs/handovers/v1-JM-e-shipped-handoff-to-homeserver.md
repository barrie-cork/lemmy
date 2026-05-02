---
author: advisor-brehon-fork-session
written: 2026-05-02T16:31Z
purpose: Handoff to fresh homeserver-CWD session to run /brehon-phase-transition v1-JM-e v1-SL-d
target_session: fresh CC at C:/Users/barri/Developer/homeserver
---

# v1-JM-e SHIPPED — handoff for /brehon-phase-transition

This advisor session (CWD `C:\Users\barri\Developer\brehon-fork`) drove v1-JM-e end-to-end and merged PR #107 into `governance-v0`. The skill `/brehon-phase-transition v1-JM-e v1-SL-d` writes its 5 deliverables into the **homeserver repo + homeserver PMD**, which is a different write surface than the brehon-fork advisor work this session has been doing. The user chose to pause and run the skill from a fresh CC session in the homeserver CWD.

## Use this file by

1. Opening fresh CC at `C:\Users\barri\Developer\homeserver`
2. Reading this handoff file (read-only)
3. Running `/brehon-phase-transition v1-JM-e v1-SL-d`

The skill at `~/.claude/skills/brehon-phase-transition/SKILL.md` will then execute Steps 0-7. This handoff pre-fills enough state that the skill's Step 0 reads run cleanly.

---

## Final state of v1-JM-e

**`governance-v0` HEAD on `origin/barrie-cork/lemmy`:** `c629a5590`

```
c629a5590 docs(retro): add canonical three-H2-header anchors to v1-JM-e retro
7e18701fc chore(bm): record PR-merge for phase-v1-JM-e (PR #107)
ea9e0d309 Merge pull request #107 from barrie-cork/phase-v1-JM-e
fb72867fb chore(advisor): brief jm-e-bm-merge-1 — merge PR #107 phase-v1-JM-e → governance-v0
9ac1e9143 fix(v1-JM-e): CR round-3 — cr-19 add H1 to satisfy MD041
f9023702f chore(decision-queue): advisor-laptop mutated DQ #113 — pass validate-pending-laptop-e2e
537bd2f0f fix(v1-JM-e): CR round-2 — cr-16 + cr-18 + cr-9-dup-2
aec57544a chore(decision-queue): advisor-laptop mutated DQ #112 — pass validate-pending-laptop-e2e
```

- **Phase branch tip merged:** `9ac1e9143` (pre-merge tip on `phase-v1-JM-e`)
- **Merge commit:** `ea9e0d309`
- **PR #107 merged via merge commit** (NOT squash, NOT rebase) per phase-branch.md convention; preserves 60+ task-by-task commits

## What v1-JM-e shipped (per plan §13)

6 plan tasks, all complete on `governance-v0`:

| Task | Slug | Commit |
|---|---|---|
| 1 | Appeal-vote tally + step_up_token DTO | `e9dbc044a` (impl) + 3 fix-impls + ci-watcher cycle |
| 2 | Cross-sub-phase capstone (full appeal lifecycle) | `a2be76a49` + 2 fix-impls (`360c24cb9`, `263a08f00`) |
| 3 | Audit-log invariant test (governance_log sequence) | `53e1b9ef8` |
| 4 | Config-churn regression (appeal.window_days) | `c83aaec05` |
| 5 | §12 security cluster (admin-visibility + spoofing) | `7726f8cf4` |
| 6 | Retro + governance-log registry marker flip | `4915a655b` |

**Plus `/brehon-verify` report at `d4b6dfa3b`** (all 4 §16a stories ✓ no phantoms).

**Plus 3 CR-driven fix-in-pr commits** during the PR cycle (`5e008a576`, `04fc66a32`, `537bd2f0f`, `9ac1e9143`).

## CR + redflag results

**4 CR review rounds; 27 findings total:**
- 15 done (all 10 round-1 fix-in-pr + 3 round-2 + 1 round-3 + 1 dedup)
- 5 rebut (cr-10/cr-11/cr-17/cr-20/redflag-1, all with code-citation rationales in pr-107-findings.yaml)
- 7 wont-fix (audit-trail principle on DQ historical entries — cr-1/4/5/6/14/15/21)
- 0 open

**Red-flag scanner:** failed 4 times advisory-only on the `accept_jury_assignment.rs` `EmergencyRemove` arm role-dispatch refactor. ADR-013 was strengthened (variant preserved in both Original + Appeal arms + new terminal-state guard in `process_appeal_vote`); the regex scanner can't see the refactor. Maintainer-ack posted at issuecomment-4363778416.

## Registry state at merge

- `ENTRY_KIND_APPEAL_DECIDED` flipped `(pending)` → `(active)` (handler `submit_jury_vote.rs::process_appeal_vote`)
- Count check: **33** (unchanged — JM-e added zero new consts, was handler + tests + 1 DTO field + 1 marker flip per plan)
- `cargo run -p lemmy_diesel_utils` migrations: round-trip passes (e2e `phase1_migrations_round_trip` is ignored per pre-existing GH #43)
- **Lessons promoted this sub-phase:** `feedback_brehon_config_micros_scaled.md` (Task 3 ship). 11 governance config keys are micros-scaled (× 1,000,000) with strict `>` comparisons; reputation_snapshot rows feed weight formulas.

## DQ state at handoff

```
pending: 0
resolved: 110
```

All in-flight DQ entries from the JM-e cycle (#100–#113) are in `resolved[]`. No carry-forward DQs. Audit-trail-flagged DQ #102 / #105 historical timestamp drift remains as-is per the wont-fix rationale.

## Carry-forward to v1-SL-d (from §7 of the retro)

1. **decline_jury_assignment role-dispatch gap** (DQ #108, `kind: log`). Mirror of accept_jury_assignment Task 2 fix-impl. No appeal-jury-decline test path exists yet, so not a regression today; would surface when (and if) v1.1+ test exercises the path. Open as low-priority issue post-merge.
2. **Audit-log invariant test plan-prefix drift verification** (retro §3.1). One-off lint at retro-time. Open as tooling/automation issue.
3. **Branch-switch contamination canary** (retro §3.2). Pre-commit hook that checks `git rev-parse --abbrev-ref HEAD` matches a session-pinned branch file. Open as `.claude/hooks/` enhancement.

## SL-d-specific watchpoints (for the new advisor-context file)

These are the watchpoints the homeserver session should encode into `advisor-context-v1-SL-d.md` §4:

1. **`submit_jury_vote.rs:782` ALL_JURY_DECISIONS const** is the pre-cleanup landing spot for SL-d's compute branch. The TODO comment says `TODO(sponsor-liability-v1)` — the v1-JM-c plan §13 Task 7 left this stub intentionally for SL-d to graft onto. SL-d MUST read this site, NOT pattern-match by skimming.
2. **PRD §17.1 sequencing:** `v1-JM-c must merge before v1-SL-d starts impl` — SATISFIED (JM-c shipped at `873b35958`, v1-JM-d at the same, JM-e at `c629a5590`). All three JM sub-phases are on trunk; SL-d is now unblocked.
3. **PRD §12.4 spoofing-protection assertion** — JM-e's Task 5 test (`constraint_relaxation_visible_to_community_admin_orphan_case_blocks_spoofing` at e2e.rs:9989+) now uses the strengthened `matches!(LemmyErrorType::NotFound)` assertion. SL-d should mirror this pattern when adding new spoof-test coverage rather than `is_err()` alone.
4. **Sponsor-liability cascade entry kinds** are reserved per registry §"sponsor-liability-v1 (reserved — §17 of PRD enumerates 5 new kinds)": `sponsor_liability_pending`, `sponsor_liability_fired`, `sponsor_liability_escaped`, `endorsement_revoked`, `restoration_completed`. SL-d's plan §17 will expand the registry from 33 → 38.
5. **`feedback_brehon_config_micros_scaled.md`** (newly promoted in JM-e Task 3) applies directly to SL-d — sponsor-liability config keys (`sponsor_liability.*`) are also micros-scaled. Tests driving the SL-d compute branch must seed reputation_snapshot rows.

## Skill-input arguments

The skill expects `<completing-id> <next-id>`:
- **completing-id:** `v1-JM-e`
- **next-id:** `v1-SL-d` (user choice — see PRD §17.1; v1-rep-tuning-r3 is the parallel-safe alternative if user reconsiders)

Phase-id grammar resolution:
- v1-JM-e: display `v1-JM-e`, slug `v1_JM_e`
- v1-SL-d: display `v1-SL-d`, slug `v1_SL_d`

## Skill writes the homeserver session must perform

1. **Step 1 archive:** `git mv homeserver/.claude/advisor-context-v1-JM-e.md homeserver/.claude/archive/advisor-context-v1-JM-e-archive.md`
2. **Step 2a rename:** `project_brehon_v1_jm_e_notes.md` → `project_brehon_v1_jm_e_complete.md` (frontmatter update + CLOSED banner)
3. **Step 2b two-ago delete:** Delete `project_brehon_v1_jm_d_complete.md` (the two-phases-ago rule walks the JM lane chain since both JM-d and JM-e are JM-lane sub-phases). v1-AD-c's complete file should already be gone from the JM-d → JM-e transition.
4. **Step 3 fresh notes:** Create `project_brehon_v1_sl_d_notes.md` with the canonical skeleton.
5. **Step 4 fresh advisor-context:** Create `homeserver/.claude/advisor-context-v1-SL-d.md` per the skill template (5 watchpoints, operational rules carry-forward from JM-e archive).
6. **Step 5 bootstrap prompt:** 7-section template with the homeserver-resolved paths.
7. **Step 6 indexes:** Update MEMORY.md + `project_brehon_governance_platform.md`.
8. **Step 7 commit:** `chore(brehon): archive v1-JM-e advisor state, bootstrap v1-SL-d context` on the homeserver repo's branch.

## Retro location

`C:\Users\barri\Developer\brehon-fork\.claude\PRPs\reports\v1-JM-e-retro.md`

Skill Step 0 retro-gate **passes** (verified at `c629a5590`): three canonical H2 headers (`## What surprised us`, `## What to change`, `## What to carry forward`) committed as anchor sections. Deeper structure preserved (§1 What worked / §2 Per-role signals / §3 What didn't / §4 Per-task complexity / §5 Lessons / §6 Confidence / §7 Follow-up).

## Cross-repo context the homeserver session needs

- **brehon-fork is at branch `governance-v0`** with the JM-e merge integrated (HEAD `c629a5590`).
- **DQ pending in brehon-fork:** 0.
- **Tailscale ACL still blocks Junior daemon SSH** from the laptop. Throughout JM-e CR cycles, BM verbs (bm-pr, bm-merge) were dispatched via foreground `branch-manager` subagent (Agent tool, model=sonnet). The homeserver session should know this so it doesn't try Junior dispatch first.
- **Retro signals are four-role partial.** Tasks 1+2 were Junior four-role; Tasks 3-5 were advisor-laptop-only (per PMD #117 e2e.rs Junior-hang); Task 6 + retro authored by advisor-laptop. The retro reflects this in §2.

## Rough sketch of what advisor-context-v1-SL-d.md §4 watchlist should contain

(Detailed enough that the homeserver session can adopt verbatim or refine.)

```
## 4. v1-SL-d-specific watchlist

1. **submit_jury_vote.rs:782 graft point.** SL-d's compute branch
   replaces the `TODO(sponsor-liability-v1)` stub at the location
   immediately after the new ALL_JURY_DECISIONS const. The const itself
   is NEW in JM-e (cr-8 fix-in-pr) — read both before authoring SL-d's
   plan §10 mirror references.
2. **registry count: 33 → 38.** SL-d adds 5 new ENTRY_KIND_*: per PRD
   §17 reserved section "sponsor-liability-v1". Plan §10.x must
   include the dual-file edit (db_schema definition + api shim
   re-export) per v1-AD-a precedent.
3. **endorsement-revocation cascade** must respect the orphan-case
   guard — if a sponsor's endorsement is revoked while one of their
   endorsed users has an open Decided case, the cascade does NOT
   retroactively flip the case. Test pattern: e2e.rs:9989+ orphan-case
   ModerationCaseInsertForm (creator_id=NULL) + winning_decision = NoAction.
4. **micros-scaled sponsor-liability config keys** per
   `feedback_brehon_config_micros_scaled.md`: sponsor_liability.*
   thresholds are × 1_000_000 with strict `>`. Tests seed
   reputation_snapshot accuracy=100 + jury_eligible per the established
   pattern.
5. **PRD §11 v0-compat regression coverage:** SL-d must extend the JM-c
   `v0_case_completes_under_v0_rules_after_v1_config_flip` pattern
   with a sponsor-liability-knob churn variant. Same shape as JM-e
   Task 4 (config-churn regression for appeal.window_days).
```

---

**Ready when you are.** Run `/brehon-phase-transition v1-JM-e v1-SL-d` from the fresh homeserver-CWD session.
