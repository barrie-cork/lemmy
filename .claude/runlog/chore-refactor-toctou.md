# Runlog — chore/refactor-toctou (PR-2, refactor-tier serial lane — FIRST of 2 remaining)

Audit-driven refactor-tier PR-2 of 5. Audit finding **3.B.1**
(rank 3, severity **CRITICAL**): wrap the SELECT-then-UPDATE/INSERT
block in `crates/api/api_crud/src/governance/create_report.rs` in
`run_transaction` (TOCTOU race; sibling `create_endorsement.rs` /
`revoke_endorsement.rs` already use the pattern — internal
inconsistency).

Sequencing: PR-4 #128 + PR-5 #129 + PR-6 #130 (the Step-2c parallel
cohort) shipped 2026-05-15. PR-2 + PR-1 are the Step-2a/2b serial
dedicated lanes. User chose **PR-2 first, then PR-1** (low-risk
isolated 1-file fix before the high-risk e2e.rs reshape). Strictly
sequential.

---

## advisor: lane prepared — 2026-05-15

- **L1 ROOT FIX applied first (tonight's retro lesson):** the daemon
  `/srv/brehon-fork` `governance-v0` was STALE at `ac52534e3d` (not
  even on the merged lineage — exactly the root cause of PR-6 #266's
  colliding DQ #214). Ran `ssh homeserver: cd /srv/brehon-fork &&
  git fetch origin && git reset --hard origin/governance-v0` →
  daemon now at `09db10847` (current tip incl. PR-4/5/6 + retro).
  PR-2 worker will branch from the correct base.
- bm-cut run **INLINE by advisor** (L3/L15 — Junior bm-task
  `base_branch=chore/*` fails on daemon worktree-ref resolution;
  proven PR-4 #264 + PR-5 #265).
- Trunk verified clean + synced (`09db10847` == origin/governance-v0).
- Cut `chore/refactor-toctou` off `governance-v0` @ `09db10847`
  (local-only, no push — per bm-cut brief Phase 3).
- Added lane-dedicated worktree
  `C:/Users/barri/Developer/brehon-fork-refactor-toctou`.
- **L6 FIX applied proactively (tonight's lesson):**
  `git submodule update --init crates/email/translations` in the new
  worktree (checked out `a3f9e466`) — re-pointed worktrees don't
  auto-init submodules; this prevents the PR-6 cargo-check
  `lemmy_email build.rs read_dir` failure before it can happen.
- Dispatched impl-task **Junior #267** with `base_branch=governance-v0`
  (NOT `chore/*` — L3) per `.claude/PRPs/briefs/refactor-toctou-impl.md`.
  Daemon health confirmed pre-dispatch (PID 258450, uptime 195h,
  0 active / 0 queued). Awaiting worker completion → Shape-G validate
  (or §5.2 advisor-laptop fallback if stuck-runner recurs per L2) →
  bm-pr inline (L3) → CR → user gate 3 → user gate 5 → merge.

## advisor: validated — 2026-05-15

- **Junior #267 done** (run #1 succeeded, 14:42→15:00Z ~17min). Worker
  raised DQ #219 (`from:impl`) CLEANLY off the L1-fixed base — NO id
  collision (contrast PR-6 #266 stale-base collision DQ #214). **The
  proactive L1 daemon-base root fix worked** — empirically confirmed
  the retro diagnosis. Worker code commit `511640ae9`:
  - touches ONLY `crates/api/api_crud/src/governance/create_report.rs`
    (+65/-11); subject `fix(api_crud): wrap create_report
    SELECT-then-write in run_transaction (audit 3.B.1 CRIT)`.
  - extracts `process_report` helper mirroring
    `create_endorsement::process_endorsement`
    (`#[allow(clippy::too_many_arguments)]`, returns
    `LemmyResult<CreateGovernanceReportResponse>`).
  - wraps the SELECT-then-UPDATE/INSERT block in
    `conn.run_transaction(|conn| async move {…}.scope_boxed())` —
    CRITICAL TOCTOU race closed.
  - hoists pseudonym fetch PRE-tx (mirrors `create_endorsement.rs` per
    ADR-015); rewires `governance_log::append` from pre-tx `pool_ref`
    to in-tx `&mut (&mut *conn).into()` (substantive: log appends now
    atomic with the case write). Zero behavior change.
- Cherry-picked ONLY `511640ae9` → `e57c20e9e` on chore (NOT the
  worker's GH-Shape-G-referencing DQ commit). `git diff --stat
  origin/governance-v0...HEAD` = ONLY `create_report.rs` + this runlog
  + DQ — no scope creep.
- Pushed `chore/refactor-toctou` (`be0d1eda5..e57c20e9e`).
  `cargo-validate-workspace.yml` triggers on `junior/*` ONLY (not
  `chore/*`); worker #267 `junior/*` push triggered run `25924783575`
  = **5th persistent stuck-runner** (PR-4 ×2 + PR-5 ×1 + PR-6 ×1 +
  PR-2 ×1 — frozen `in_progress`, zero job progress). Cancelled per
  advisor-orchestrator.md §5.2 (advisor-laptop fallback,
  user-authorised "Go with C" + overnight autonomy + sequential lane).
- **§5.2 advisor-laptop validation — PASS:** local
  `cargo-check.bat --workspace --features full` → `CHECK_EXIT_0`
  (12m14s, `lemmy_api_crud` clean); local `cargo-test.bat --workspace
  --features full --no-run` → `TESTCOMPILE_EXIT_0` (all test targets
  incl `e2e.rs` compiled clean). Logs at
  `C:/Users/barri/.claude/logs/pr2-check.log` +
  `pr2-testcompile.log`. Authoritative entry: **DQ #219**
  (`from:advisor`, `kind:validate-pending`, `result:pass`,
  `answered_by:advisor-laptop`). L6 submodule was init'd pre-emptively
  — no `lemmy_email build.rs` failure recurred.
- Next: bm-pr INLINE (L3/L15) per
  `.claude/PRPs/briefs/refactor-toctou-bm-pr.md` (authored on
  governance-v0 `a998ecc03` — L5: NOT cherry-picked here) → CR →
  user gate 3 → user gate 5 → merge → strict-gate 4/5 → surface PR-1
  for explicit user go-ahead (do NOT auto-start the highest-risk
  e2e.rs lane).
