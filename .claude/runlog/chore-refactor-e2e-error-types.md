# Runlog — chore/refactor-e2e-error-types (PR-1, refactor-tier LAST lane — 5th of 5)

Audit-driven refactor-tier **PR-1 of 5** — the FINAL lane and the
HIGHEST-RISK one. Bundles audit findings **3.E.1 (CRIT) + 3.E.2
(CRIT) + 3.E.3 (MAJ) + 3.E.4 (MAJ)** (ranks 1, 2, 7, 8): unify
`crates/server/tests/e2e.rs` (8945 lines) Brehon-authored test
corpus to a uniform `LemmyResult<()>` shape (Case C → Case A per
`feedback_lemmy_error_no_std_error.md`), consolidate 4
phase-specific fixtures modules (~70% duplication) into shared
helpers, split the 447-line `phase1_migrations_round_trip` into 3
independent test fns, and replace `PHASE_1_MIGRATION_COUNT` with a
named `MIGRATIONS_TO_REVERT_PHASE_1` list.

Sequencing: PR-4 #128 + PR-5 #129 + PR-6 #130 (Step-2c parallel
cohort) shipped 2026-05-15; PR-2 #131 (TOCTOU create_report, Step-2b
serial) merged 2026-05-15 → strict-gate **4/5**. PR-1 is the LAST
(Step-2a serial dedicated lane). User chose **PR-2 first, then
PR-1** and explicitly user-gated the PR-1 dispatch (highest-risk,
e2e.rs worker-hang surface per `feedback_junior_worker_e2e_edit_hang`).
Strictly sequential. On PR-1 merge → strict-gate 5/5 → autonomous
loop STOPS → 4-role retro → user sign-off (carried critical
instruction; do NOT proceed to v1 PRD planning until 5/5 + retro +
sign-off).

---

## advisor: lane prepared + PR-1 dispatched — 2026-05-15

- **PR-2 #131 merged first** (squash `1fed06e16`, strict-gate 4/5).
  Post-merge governance-v0 ff-pulled; PR-2 worktree force-removed
  (submodule — `feedback_worktree_remove_force_for_submodules`);
  `chore/refactor-toctou` deleted.
- **L1 ROOT FIX re-applied (user option a):** governance-v0 advanced
  to `1fed06e16` (PR-2 squash + bm-pr brief `a998ecc03`), so the
  daemon `/srv/brehon-fork` base was stale again at `08a9069ff`.
  Ran `ssh homeserver: cd /srv/brehon-fork && git fetch origin &&
  git reset --hard origin/governance-v0` → daemon now at
  `1fed06e16` == laptop tip. PR-1 worker branches from the correct
  base; the #266-class stale-base DQ-collision is pre-empted (same
  proactive fix that kept PR-2's worker DQ #219 collision-free).
- **/precheck pre-queue git pre-flight — all green:** canonical
  branch `governance-v0` clean (only the pre-existing gitignored
  `memory.db.pre-migration-*` backup untracked); `governance-v0`
  `1fed06e16` == `origin/governance-v0` (no divergence — workers
  branch from the correct committed HEAD); daemon base `1fed06e16`
  (clean status); brief
  `.claude/PRPs/briefs/refactor-e2e-error-types-impl.md` present on
  governance-v0 (`9008a3d01`).
- Daemon health confirmed pre-dispatch: PID 258450, uptime 197h,
  0 active / 0 queued.
- Dispatched impl-task **Junior #268** with
  `base_branch=governance-v0` (NOT `chore/*` — L3, daemon cannot
  resolve chore refs) per
  `.claude/PRPs/briefs/refactor-e2e-error-types-impl.md`.
- **Risk surfaces being watched (LAST + highest-risk lane):**
  - e2e.rs is 8945 lines — `feedback_junior_worker_e2e_edit_hang`:
    anchor-Edit discipline mandatory; worker may hang on Edit. On a
    genuine hang DO NOT auto-cancel — surface to user (the brief
    enforces surgical `old_string` / no `replace_all` / no full Read
    / commit-every-~10-edits, but the hang class can still recur).
  - Case C → Case A uniformity is mandatory (no partial conversion —
    `feedback_lemmy_error_no_std_error.md` Case A). If the worker
    hits an unresolvable Case-C cascade it must DQ-block, not
    `#[allow]`/`#[ignore]`.
  - Brief §2.5 line-number tension: brief cites v1_sl_b at
    "11139-12086" but §2.2 states e2e.rs is 8945 lines total (audit
    line numbers stale). The brief itself instructs the worker to
    verify actual positions and DQ-block if drift >50 lines — this
    is a designed-in catch, not a dispatch blocker; advisor will
    surface any resulting DQ.
  - Test count must equal pre-refactor + 2 (round-trip split adds 2).
  - **Local e2e suite run (~26 min) is the LOAD-BEARING signal** —
    workspace-check (`cargo test --no-run`) is insufficient for a
    test refactor. Worker raises `validate-pending-laptop-e2e`
    (per brief §4 + advisor-orchestrator §5.2).
- Awaiting worker completion → verify scope (ONLY e2e.rs) + DQ →
  cut `chore/refactor-e2e-error-types` + lane worktree (L6 submodule
  init) → cherry-pick code commit → §5.2 advisor-laptop validate
  (Shape-G expected stuck per L2; e2e is load-bearing — run the full
  suite locally, not just compile) → bm-pr inline (L3) → CR → user
  gate 3 → user gate 5 → merge → **strict-gate 5/5 → STOP → 4-role
  retro → user sign-off**.
