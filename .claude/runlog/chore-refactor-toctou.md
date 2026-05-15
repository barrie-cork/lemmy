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
- Next: dispatch impl-task Junior with `base_branch=governance-v0`
  (NOT `chore/*` — L3) per `.claude/PRPs/briefs/refactor-toctou-impl.md`.
