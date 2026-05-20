# brehon-conformance-audit — bm-cut brief

## 1. Role + dispatch line

`[role:bm-task] bm-cut phase-brehon-conformance-audit — see .claude/PRPs/briefs/brehon-conformance-audit-bm-cut-1.md`

## 2. Scope

Cut local branch `phase-brehon-conformance-audit` off `governance-v0` tip `7d97c90e5` per `.claude/commands/bm/bm-cut.md`. Do NOT push (per the spec: "Do NOT push yet. The branch sits local until impl makes its first commit").

**Commit only:**
- `.claude/runlog/brehon-conformance-audit-runlog.md` (new file — Phase 4 append per bm-cut.md).

**Do NOT touch:**
- The plan file (`.claude/PRPs/plans/brehon-conformance-audit.plan.md`) — already authored at `a16a4e2e1`.
- Any file under `crates/`, `migrations/`, `tests/`, `docs/brehon-law-inspired-network/`.
- `.claude/PRPs/plans/**`, `.claude/PRPs/prds/**`.

## 3. Required reading

1. `.claude/commands/bm/bm-cut.md` — operational script in full. Phase 0 regex was widened 2026-05-20 (`7d97c90e5`) to accept `^phase-[a-z0-9][a-z0-9-]*$`; `phase-brehon-conformance-audit` matches.
2. `.claude/rules/branch-manager.md` — file ownership boundaries; the BM session OWNS `.claude/runlog/bm-*.md` and the branch creation, NEVER touches `crates/`, `migrations/`, `tests/`, `docs/brehon-law-inspired-network/`, `.claude/PRPs/plans/`.
3. `.claude/rules/phase-branch.md` — phase-branch + PR flow. The plan ships meta-tooling PLUS three federation `mod.rs` attribute additions — crosses the PR-flow threshold; PR closes via `bm-pr` into `governance-v0` (NOT `main`).
4. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` — read §5 Metadata + §13 Task 0 verification probes ONLY (so the runlog entry's "next" line is accurate). Do NOT read the full plan; impl session does that.
5. `.claude/lessons/feedback_junior_finalize_merges_bm_cut_branch.md` — KNOWN harness limitation: the Junior daemon's finalize agent will WRONGLY run `git merge --no-ff phase-brehon-conformance-audit INTO daemon-local governance-v0`, producing a spurious content-empty merge commit. The advisor verifies daemon-local trunk POST-bm-cut as a routine step and recovers via `git update-ref refs/heads/governance-v0 origin/governance-v0`. Do NOT attempt to "fix" this from inside the bm-cut task.
6. `.claude/lessons/feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` — runlog-write gate-block hazard + advisor-relocate recovery.

## 4. Constraints

1. **Phase 0 sanity:** branch name `phase-brehon-conformance-audit` MUST match the broadened regex (`^phase-[a-z0-9][a-z0-9-]*$`). It does (lowercase + dashes only). If the regex check refuses, STOP and file `kind: "blocker"` DQ.
2. **Phase 1 trunk state:**
   - Working tree clean (verify; if not, STOP — do NOT auto-stash).
   - `governance-v0` synced with `origin/governance-v0` (currently `7d97c90e5` on both per the dispatch).
   - If `governance-v0` is behind origin, run `git checkout governance-v0 && git pull --ff-only origin governance-v0` (per bm-cut.md Phase 1 decision tree).
3. **Phase 2 plan verification:** confirm `.claude/PRPs/plans/brehon-conformance-audit.plan.md` exists on `governance-v0`. It does (at `a16a4e2e1`).
4. **Phase 3 cut:** `git checkout -b phase-brehon-conformance-audit governance-v0`. Do NOT push.
5. **Phase 4 runlog:** create `.claude/runlog/brehon-conformance-audit-runlog.md` with the bm-cut.md Phase 4 template — `## bm: branch cut — <ISO timestamp>` + 4 bullets (branch / off / plan / next).
6. **Phase 5 output:** print the bm-cut.md Phase 5 markdown surface verbatim (Branch cut, Off, Plan file on trunk, Pushed? No, Runlog).
7. **Phase 6 KNOWN harness limitation:** state in the runlog under a `## bm: KNOWN harness limitation` block that:
   - bm-cut creates divergence from trunk (the phase branch is the deliverable, NEVER merged back into governance-v0).
   - The daemon finalize agent is feature-branch-shaped and will run `git merge --no-ff phase-brehon-conformance-audit INTO daemon-local governance-v0` post-task. This is a known hazard (per `feedback_junior_finalize_merges_bm_cut_branch.md`), advisor handles post-task.
   - The CC v2.1.119 runlog gate-block (per `feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md`) may block the BM Junior's runlog write. If the runlog cannot be written, STOP and file `kind: "blocker"` DQ with the error message.
8. **Commit:** ONE commit. Subject: `chore(bm): cut phase-brehon-conformance-audit from governance-v0 @ 7d97c90e5`. Body cites the plan file by path + the brief by path.
9. **Attribution integrity:** the BM Junior NEVER writes `answered_by: "advisor"` or `answered_by: "user"`. Self-attribution under `bm-self-resolved` only if absolutely necessary (no DQ entries expected from this task — it's mechanical).
10. **No push:** the branch is local-only at bm-cut. Push deferred to first impl-task commit + `/bm-push`.

## 5. KNOWN harness limitation (verbatim — copy into the runlog)

> bm-cut creates divergence from `governance-v0`. The phase branch `phase-brehon-conformance-audit` is the deliverable; it must NEVER be merged back into `governance-v0`. The Junior daemon's generic post-job finalize agent is feature-branch-shaped and will WRONGLY run `git merge --no-ff phase-brehon-conformance-audit INTO daemon-local governance-v0`, producing a spurious content-empty merge commit. The advisor verifies daemon-local trunk POST-bm-cut as a ROUTINE step (not an exception path) and recovers via `git update-ref refs/heads/governance-v0 origin/governance-v0` (working-tree-safe — NOT `git reset --hard`).

## 6. Next steps after this task

- Advisor verifies daemon-local trunk via `ssh homeserver "cd /srv/brehon-fork && git log governance-v0 --oneline -1"` and recovers if the finalize-agent-merge bug fired.
- Advisor dispatches Cohort 1 (Task 1 alone — SKILL.md skeleton) per plan §13 cohort plan.
- The phase branch gets pushed to origin via `bm-push` (or implicitly by the first impl-task commit's push).

## 7. DoD for this brief (advisor-side gate)

- [ ] BM Junior task created with this brief path in the description line.
- [ ] Brief content matches `.claude/commands/bm/bm-cut.md` operational steps verbatim.
- [ ] No reference to `cargo`, no reference to crate code.
- [ ] Subject line for the BM Junior commit specified.
- [ ] KNOWN harness limitation §5 carried verbatim per `feedback_junior_finalize_merges_bm_cut_branch.md`.

## 8. Commit subject for this brief

`chore(advisor): author bm-cut brief for phase-brehon-conformance-audit`
