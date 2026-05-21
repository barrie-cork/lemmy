---
author: advisor (lane-dedicated brehon-fork-rls-r1 / phase-v1-rls-r1 session, post-PR-#140-merge)
next_role: advisor
next_session: canonical brehon-fork checkout on governance-v0
authored: 2026-05-21
purpose: Updates the pre-existing v1-federation-inbound-c-bootstrap.md (authored 2026-05-20) with v1-rls-r1's actual ship state + the user's scope-(a)-only decision. Read this brief FIRST; then read the full bootstrap; then start the session-start ritual.
---

# Advisor handover — fed-in-c entry, scope (a) only

## What changed since the bootstrap was authored (2026-05-20)

1. **v1-rls-r1 SHIPPED** 2026-05-21. PR #140 merged at `16ede83b6` on `governance-v0`. Post-merge runlog commit `073a34040`. Session retro on `governance-v0` at `f0dcc1275`. Three new lessons promoted in retro: `feedback_advisor_authoring_under_daemon_stress.md`, `feedback_worker_hang_post_dq_raise.md`, `feedback_cross_lane_daemon_ref_contamination.md`. One new lesson promoted mid-phase: `feedback_cohort_dq_id_collision.md`. The MEMORY.md "Active workflow state" block was updated post-merge (also picked up a concurrent edit from the conformance-audit advisor session — current canonical state is the merged form, see §"What canonical brehon-fork looks like now").

2. **brehon-conformance-audit progress.** Per MEMORY.md observed after merge: phase tip is now `6c7e80e29` (not `f9eb94969` as the bootstrap recorded); Phase 1 mechanism implementation (Tasks 8 + 8a + TOML fix + 9) DONE and all DQ-pass. Next: Cohort 3 Task 7 dogfood against fed-in-b snapshots. **Implication for fed-in-c scope (c):** the conformance-audit skill is closer to landing than the bootstrap assumed; if the user wants the Phase-6 convention-divergence audit, defer it until conformance-audit ships.

3. **User direction 2026-05-21:** fed-in-c **scope (a) only** — reader-side append-history fix on `get_inbound_config_int` + mirror. Defer (b) DoS-hardening and (c) Phase-6 convention-divergence audit to later sub-phases. This is the small bounded entry slice; (b) and (c) will be re-evaluated at fed-in-c retro time.

## What canonical brehon-fork looks like now

- CWD: `C:/Users/barri/Developer/brehon-fork`
- Branch: `governance-v0`
- Origin tip: `f0dcc1275` (`docs(retro): session retro for v1-rls-r1 ship close-out`)
- Local tip vs origin: **possibly stale** in the canonical checkout. Run `git fetch origin governance-v0 && git pull --ff-only` before any work.
- Worktrees present (`git worktree list`):
  - `C:/Users/barri/Developer/brehon-fork` — canonical, `governance-v0`
  - `C:/Users/barri/Developer/brehon-fork-conformance-audit` — active brehon-conformance-audit lane (separate advisor session — do NOT touch)
  - `C:/Users/barri/Developer/brehon-fork-rls-r1` — closed, awaiting teardown (see §"Lane teardown" below)
  - `C:/Users/barri/Developer/brehon-fork-tooling` — long-lived tooling lane, unrelated

## Lane teardown (manual, run from canonical session before fed-in-c bm-cut)

```bash
cd C:/Users/barri/Developer/brehon-fork
git worktree remove ../brehon-fork-rls-r1
git branch -d phase-v1-rls-r1
```

These are reversible (worktree teardown only removes the gitdir admin state + leaves the lane checkout dir; `git branch -d` only deletes the local ref since origin already deleted it). Run before authoring the fed-in-c planning brief — keeps the canonical session's `git worktree list` tidy.

## Scope (a) — reader-side append-history fix

Per the existing bootstrap §1 + §4 watchlist items 1 + 2 + 5:

- **File 1 of 2:** `crates/apub/activities/src/governance/inbox.rs:421-438` — `get_inbound_config_int` reads `.first::<Option<i64>>(conn)` WITHOUT `.order_by(governance_config::valid_from.desc())`. Returns arbitrary row when `governance_config` accumulates admin-edit history.
- **File 2 of 2:** `crates/apub/activities/src/governance/publish_trust_attestation.rs:142-152` — mirror site with the same defect.
- **Helper check FIRST:** `grep -rn "governance_config_current\|latest_value_for" crates/db_schema/src/` — if a "latest row per key" helper or view exists, the fix is "use the helper"; if not, the fix is the `.order_by(valid_from.desc()).limit(1)` chain (and possibly authoring the helper as part of fed-in-c, but that grows scope — push back if so).
- **e2e fixture pattern:** fed-in-b's `mod v1_federation_inbound_b_fixtures` in `crates/server/tests/e2e.rs` uses `LemmyResult<()>` outer shape. Any fed-in-c e2e additions mirror Case A per `feedback_lemmy_error_no_std_error.md`.
- **Expected migration count:** **zero**. fed-in-c scope (a) is reader-side code-only. If the planning Junior proposes a new migration under `migrations/**` or `crates/db_schema/migrations/**`, catch-fire (scope violation).

## Pre-planning gate sequence

Per advisor-orchestrator §3.1 stage-shape orchestration:

1. **Session-start ritual** — `pwd && git branch --show-current && git worktree list` from canonical CWD; `git fetch origin && git pull --ff-only`; read MEMORY.md (auto-loads); read `workflow_state_v1_federation_inbound_c.md` (already exists from bootstrap session).
2. **Read in order:**
   - This handover brief (this file).
   - `.claude/PRPs/handovers/v1-federation-inbound-c-bootstrap.md` in full (the 2026-05-20 base context — still load-bearing for §3 lessons + §4 watchlist + §5 ops + §7 catch-fires).
   - `.claude/PRPs/reports/v1-federation-inbound-b-retro.md` once (carry-forward).
3. **Author planning brief** at `.claude/PRPs/briefs/v1-federation-inbound-c-planning-1.md`. Commit on `governance-v0`. Scope: (a) only per user direction 2026-05-21. Explicitly de-scope (b) + (c) in the brief §2 with one-line justifications referencing this handover.
4. **Run `/brehon-clarify .claude/PRPs/briefs/v1-federation-inbound-c-planning-1.md`** (advisor-mode default; escalate to user-relay only for judgment-heavy ambiguities). Resolves all clarify-DQ entries against the brief BEFORE planning Junior dispatch.
5. **Queue planning Junior task** via `mcp__junior-brehon__create_task` with `base_branch=governance-v0` (canonical pre-bm-cut state).
6. On planning Junior complete:
   - Run §3.4 DoD smoke test (every §15 command literally against current HEAD).
   - Run §3.5 watchpoint specificity gate (every plan §4 watchpoint must cite a specific file:line or schema.rs line).
   - User Gate 1 (plan approval) via AskUserQuestion.

## Pre-cohort DQ id reservation (interim mitigation from v1-rls-r1)

Per `feedback_cohort_dq_id_collision.md` Option 3: **for any `[P]` cohort with N≥2 members in fed-in-c, advisor pre-authors N reserved DQ stubs in pending[] BEFORE dispatching the cohort, and each impl brief names its assigned DQ id explicitly in §4 Constraints**. This prevents the 5-way collision recovered from in v1-rls-r1 Cohort A. The lesson is not yet formalised in `advisor-orchestrator.md §4.1` — fed-in-c's planning brief should explicitly cite the mitigation in §0 PRECONs.

For scope (a) only, the likely cohort is 2 files (inbox.rs:421-438 + publish_trust_attestation.rs:142-152) — small enough that even pre-reservation is cheap.

## Lessons authored during v1-rls-r1 worth reading before fed-in-c

These are NEW and relevant to fed-in-c:

- `feedback_advisor_authoring_under_daemon_stress.md` — when 2+ daemon stresses fire in one phase, advisor switches to inline authoring. Bake into your planning brief: if scope (a)'s impl Junior hangs or contaminates, fall back early.
- `feedback_worker_hang_post_dq_raise.md` — the silent hang pattern + cancel/FF-merge/lane-cargo recovery template. Useful for fed-in-c's first impl-task dispatch.
- `feedback_cross_lane_daemon_ref_contamination.md` — pre-dispatch daemon-FF as a structural gate. Especially relevant given brehon-conformance-audit is still active in parallel.
- `feedback_cohort_dq_id_collision.md` — interim mitigation 3 for `[P]` cohorts (cited above).

## Decision-queue snapshot at fed-in-c entry

DQ pending = 0 at session start (verified post-merge). The session that authored this handover (lane v1-rls-r1) resolved DQ #303-#323. No DQs cross-cut into fed-in-c — fed-in-c starts from a clean slate.

Forward reminder still live: **DQ #229 — Shape G re-enable 2026-06-01.** If fed-in-c crosses 2026-06-01 (likely, storage-hardening can span days), re-check DQ #229 first thing in the new-day session; Shape G may be re-enabled and the validate-pending shape changes.

## Stop-and-ask tripwires (carry-forward from bootstrap §"Stop-and-ask")

Unchanged. Especially:

- Plan proposes a new migration under `migrations/**` → catch-fire (scope violation per scope (a) only).
- Plan proposes modifying `wrap_governance_inbound`'s gate order → catch-fire (fed-in-b contract).
- `brehon-conformance-audit` planning brief NOT read before scoping (c) into fed-in-c → catch-fire. User direction is **scope (a) only**, so this should not fire; but if a clarify-DQ or planning Junior tries to broaden the scope, surface to user.
- Validate-pending fails 3× with same `(error_class, file_basename)` → §G4 hard refusal.

## What this handover is NOT

- This handover is NOT a replacement for the existing `v1-federation-inbound-c-bootstrap.md`. It's a **delta + current-state update**. The bootstrap's §3 lessons + §4 watchlist + §5 operational rules + §7 catch-fires all still apply.
- This handover is NOT a planning brief. The planning brief is the next deliverable; this is the resume context to author it from.
- This handover is NOT impl-direction. The impl files + line ranges in §"Scope (a)" above are direction for the **planner**, not the impl-task worker. The planning Junior translates them into a §13 task table with full IMPLEMENT/MIRROR/GOTCHA/VALIDATE shape.

## See also

- `.claude/PRPs/handovers/v1-federation-inbound-c-bootstrap.md` — the comprehensive 2026-05-20 base context (this handover updates it; read both)
- `.claude/PRPs/reports/v1-federation-inbound-b-retro.md` — carry-forward source
- `.claude/PRPs/reports/v1-rls-r1-retro.md` — three new daemon-stress lessons born here
- `.claude/PRPs/reports/session-retro-2026-05-21-v1-rls-r1-ship.md` — fresh post-merge session retro with three carry-forward observations for v1-rls-r2 (but those are not blockers for fed-in-c)
- `MEMORY.md` (auto-loaded) — current Active workflow state block reflects the post-merge transition
- `workflow_state_v1_rls_r1.md` (just-shipped CLOSED record) — for any "what did v1-rls-r1 actually do" lookups
