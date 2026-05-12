# Brief: sl-e-bm-cut-1 — cut phase-v1-SL-e off governance-v0

## 1. Role + dispatch line

`[role:bm-task] sl-e-bm-cut-1 — cut phase-v1-SL-e off governance-v0`

## 2. Scope

Run `/bm/cut` per `.claude/commands/bm/bm-cut.md` to create `phase-v1-SL-e`
branched from `governance-v0` HEAD `c61dc700c`. Push the new phase
branch to origin so subsequent `[role:impl-task]` workers can branch
from `phase-v1-SL-e` per advisor-orchestrator.md "Each impl-task
complete" Shape-G flow.

**Single deliverable:** the `phase-v1-SL-e` branch on origin pointing
at `c61dc700c` (or the current governance-v0 HEAD if it has advanced).

**Out of scope:**

- Do NOT add commits beyond what `/bm/cut` produces.
- Do NOT modify governance-v0.
- Do NOT modify the plan, decision-queue.json, or any handler/test
  files.
- Do NOT open a PR — that's `bm-pr` after impl tasks are done.

## 3. Required reading

1. `.claude/commands/bm/bm-cut.md` — canonical /bm/cut spec.
2. `.claude/rules/branch-manager.md` — BM mechanics + invariants.
3. `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md` §5 — confirms
   phase branch name (`phase-v1-SL-e`) + base (governance-v0).
4. `.claude/lessons/feedback_pr_per_phase.md` — phase-branch lifecycle
   discipline.

## 4. Constraints

- **Branch base must be `governance-v0` HEAD `c61dc700c`** at
  brief-write time. If governance-v0 has advanced when bm-cut runs,
  pull governance-v0 to the new HEAD first, then cut from there
  (latest-trunk discipline).
- **Branch name must be exactly `phase-v1-SL-e`** — match plan §5.
- **Push to origin immediately after cut.** Subsequent impl-task
  workers branch from `origin/phase-v1-SL-e`.
- **Commit subject for any cut-side commit (if /bm/cut writes a
  runlog):** must match `chore\(bm\):` per attribution-integrity in
  `.claude/rules/decision-queue.md`.
- **No DQ entries expected** — this is a mechanical cut. If the cut
  encounters a divergence (e.g. branch already exists on origin with
  different content), file a `kind: "blocker"` DQ with `from: "bm"`
  and stop.
