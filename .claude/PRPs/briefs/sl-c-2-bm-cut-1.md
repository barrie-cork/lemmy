# Brief: sl-c-2-bm-cut-1 — cut phase-v1-SL-c-2 off governance-v0

## 1. Role + dispatch line

`[role:bm-task] sl-c-2-bm-cut-1 — cut phase-v1-SL-c-2 off governance-v0`

## 2. Scope

Run `/bm-cut` per `.claude/commands/bm/bm-cut.md` to create
`phase-v1-SL-c-2` branched from `governance-v0` HEAD `d078f6a37`
(advisor commit `feat(advisor): /auto-phase skill —
single-trigger sub-phase orchestrator`). Push the new phase branch
to origin so subsequent `[role:impl-task]` workers can branch from
`phase-v1-SL-c-2` per `advisor-orchestrator.md` "Each impl-task
complete (under Shape G)" flow.

**Single deliverable:** the `phase-v1-SL-c-2` branch on origin
pointing at `d078f6a37` (or a later `governance-v0` HEAD if trunk
advances during cut, per `bm-cut.md` Phase 1 latest-trunk
discipline).

**Out of scope:**

- Do NOT add commits beyond what `/bm-cut` produces (a runlog
  append at `.claude/runlog/v1-SL-c-2-runlog.md` is the only
  allowed write per `bm-cut.md` Phase 4).
- Do NOT modify `governance-v0`.
- Do NOT modify the plan, decision-queue.json, or any
  `crates/**` / `migrations/**` / `tests/**` content.
- Do NOT open a PR — that's `bm-pr` after impl tasks land.
- Do NOT create temp-merge branches that touch `governance-v0`
  (per L11 from SL-c-1 retro / DQ #148 breach — `bm-task` MUST
  refuse any operation that pushes `temp-bm-push`/temp-merge
  branches to `governance-v0`).

## 3. Required reading

1. `.claude/commands/bm/bm-cut.md` — canonical /bm-cut spec
   (Phases 0-5; refusal cases).
2. `.claude/rules/branch-manager.md` — BM file-ownership +
   autonomy bounds + auto-vs-manual table. Pay attention to
   "What BM should refuse" — never push temp-merge branches to
   `governance-v0`.
3. `.claude/rules/phase-branch.md` — phase-branch + PR flow rules
   (this is a CODE phase, so PR flow applies — c-2 modifies
   `crates/server/tests/e2e.rs`).
4. `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md` §5
   Metadata — confirms phase branch name (`phase-v1-SL-c-2`) +
   base (`governance-v0`).
5. `.claude/PRPs/reports/v1-SL-c-1-retro.md` — L11 (temp-merge
   branch breach) + L14 (BM Junior runlog commit sequencing) + L15
   (gate-then-execute split duplicates context). Read so you do
   not repeat the SL-c-1 #148 hard-refusal breach.
6. `.claude/lessons/feedback_pr_per_phase.md` — phase-branch
   lifecycle discipline.

## 4. Constraints

- **Branch base must be `governance-v0` HEAD `d078f6a37`** at
  brief-write time. If `governance-v0` has advanced when bm-cut
  runs, pull `governance-v0` to the new HEAD via `git checkout
  governance-v0 && git pull --ff-only origin governance-v0` first
  (latest-trunk discipline per `bm-cut.md` Phase 1 decision tree),
  then cut from the new HEAD.
- **Branch name must be exactly `phase-v1-SL-c-2`** — matches plan
  §5 Metadata. Validated by `bm-cut.md` Phase 0 against the regex
  `^phase-v\d+-[A-Z]+-[a-z]$` — `phase-v1-SL-c-2` matches.
- **Plan file must exist on trunk before cut** — `bm-cut.md`
  Phase 2 verifies. Required path:
  `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md`. Already
  committed and present on `governance-v0`.
- **c-1 prerequisite verified at brief-write time:** PR #121
  (`phase-v1-SL-c-1 → governance-v0`) merged at `8bfc085dc`
  (2026-05-08T18:42 UTC). c-2's plan §6 lists c-1 as a MERGED
  upstream dep. No additional c-1 verification required at
  bm-cut time — Task 0 of c-2's §13 plan re-verifies via Probe 7.
- **Push to origin immediately after cut.** Subsequent impl-task
  workers branch from `origin/phase-v1-SL-c-2`. (The `bm-cut.md`
  default Phase 3 is local-only; this brief authorises the push as
  part of the same task because the next bm-task or impl-task
  needs the upstream tracking. Same pattern as
  `sl-c-1-bm-cut-1.md` §4.)
- **Commit subject for the runlog append (Phase 4):** must match
  `^chore\(bm\):` per `.claude/rules/decision-queue.md`
  attribution-integrity (BM authoring on the phase branch — the
  runlog write at `.claude/runlog/v1-SL-c-2-runlog.md` is the
  only allowed in-task commit).
- **No DQ entries expected.** This is a mechanical cut. If the cut
  encounters a divergence (e.g. `phase-v1-SL-c-2` already exists
  on origin with different content, or `governance-v0` has
  unpushed divergent commits), file a `kind: "blocker"` DQ with
  `from: "bm"` per `.claude/rules/decision-queue.md` Recipe 1,
  mid-task commit + push immediately to the cut branch, and stop.
- **No upstream-branch refusal:** `phase-v1-SL-c-2` does NOT exist
  on origin yet (verified at brief-write time:
  `gh pr list --repo barrie-cork/lemmy --state all --head
  phase-v1-SL-c-2` returns empty). If it appears during the cut,
  treat as the divergence case above — do not silently proceed.
- **Hard refusal — temp-merge branches:** as per L11/DQ #148,
  if you find yourself constructing a `temp-bm-push` or any
  intermediate merge branch that would touch `governance-v0`,
  STOP, file a `kind: "blocker"` DQ, and exit. The advisor will
  re-author this brief with stricter constraints rather than
  unwind a polluted trunk.

## 5. Hand-off

After bm-cut completes, the advisor's polling loop sees the new
`phase-v1-SL-c-2` branch on origin and proceeds with the planning
stage's planning Junior task per plan §13:

- **Task 0** (pre-flight harness audit + branch verification +
  SL-a/SL-b/c-1 state confirmation) — non-`[P]`, dispatched alone.
- Then **Task 1** (e2e test #1 — fire path + open
  `mod v1_sl_c_fixtures` shell + helpers), serial after Task 0.
- Then **Tasks 2-5** (each modifies
  `crates/server/tests/e2e.rs`; serial dispatch — YAML overlap
  rule refuses cohort).
- Then **Task 6** (retro).
