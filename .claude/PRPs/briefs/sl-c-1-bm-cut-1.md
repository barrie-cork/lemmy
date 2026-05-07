# Brief: sl-c-1-bm-cut-1 — cut phase-v1-SL-c-1 off governance-v0

## 1. Role + dispatch line

`[role:bm-task] sl-c-1-bm-cut-1 — cut phase-v1-SL-c-1 off governance-v0`

## 2. Scope

Run `/bm-cut` per `.claude/commands/bm/bm-cut.md` to create
`phase-v1-SL-c-1` branched from `governance-v0` HEAD `fce34b893`.
Push the new phase branch to origin so subsequent `[role:impl-task]`
workers can branch from `phase-v1-SL-c-1` per
`advisor-orchestrator.md` "Each impl-task complete (under Shape G)"
flow.

**Single deliverable:** the `phase-v1-SL-c-1` branch on origin
pointing at `fce34b893`.

**Out of scope:**

- Do NOT add commits beyond what `/bm-cut` produces (a runlog append
  is the only allowed write per the bm-cut script Phase 4).
- Do NOT modify governance-v0.
- Do NOT modify the plan, decision-queue.json, or any
  `crates/**` / `migrations/**` / `tests/**` content.
- Do NOT open a PR — that's `bm-pr` after impl tasks land.

## 3. Required reading

1. `.claude/commands/bm/bm-cut.md` — canonical /bm-cut spec (Phases
   0-5; refusal cases).
2. `.claude/rules/branch-manager.md` — BM file-ownership + autonomy
   bounds + auto-vs-manual table.
3. `.claude/rules/phase-branch.md` — phase-branch + PR flow rules
   (this is a CODE phase, so PR flow applies).
4. `.claude/PRPs/plans/v1-sponsor-liability-c-1.plan.md` §5 Metadata
   — confirms phase branch name (`phase-v1-SL-c-1`) + base
   (`governance-v0`).
5. `.claude/lessons/feedback_pr_per_phase.md` — phase-branch
   lifecycle discipline.

## 4. Constraints

- **Branch base must be `governance-v0` HEAD `fce34b893`** at
  brief-write time. If governance-v0 has advanced when bm-cut runs,
  pull `governance-v0` to the new HEAD via `git checkout
  governance-v0 && git pull --ff-only origin governance-v0` first
  (latest-trunk discipline per bm-cut.md Phase 1 decision tree),
  then cut from the new HEAD.
- **Branch name must be exactly `phase-v1-SL-c-1`** — matches plan
  §5 Metadata. Validated by `bm-cut.md` Phase 0 against the regex
  `^phase-v\d+-[A-Z]+-[a-z]$` — `phase-v1-SL-c-1` matches.
- **Plan file must exist on trunk before cut** — bm-cut.md Phase 2
  verifies. Required path:
  `.claude/PRPs/plans/v1-sponsor-liability-c-1.plan.md`. Already
  committed at `7ccfee0af` and present on `governance-v0`.
- **Push to origin immediately after cut.** Subsequent impl-task
  workers branch from `origin/phase-v1-SL-c-1`. (The bm-cut.md
  default Phase 3 is local-only; this brief authorises the push as
  part of the same task because the next bm-task or impl-task needs
  the upstream tracking. This is the same pattern sl-b-bm-cut-1.md
  used at §4.)
- **Commit subject for the runlog append (Phase 4):** must match
  `^chore\(bm\):` per `.claude/rules/decision-queue.md`
  attribution-integrity (BM authoring on the phase branch — the
  runlog write at `.claude/runlog/v1-SL-c-1-runlog.md` is the only
  allowed in-task commit).
- **No DQ entries expected.** This is a mechanical cut. If the cut
  encounters a divergence (e.g. `phase-v1-SL-c-1` already exists on
  origin with different content, or governance-v0 has unpushed
  divergent commits), file a `kind: "blocker"` DQ with
  `from: "bm"` per `.claude/rules/decision-queue.md` Recipe 1, mid-
  task commit + push immediately to the cut branch, and stop.
- **No upstream-branch refusal:** `phase-v1-SL-c-1` does NOT exist
  on origin yet (verified at brief-write time). If it appears
  during the cut, treat as the divergence case above — do not
  silently proceed.

## 5. Hand-off

After bm-cut completes, the advisor's polling loop sees the new
`phase-v1-SL-c-1` branch on origin and proceeds to queue the first
impl-task per plan §13:

- **Task 0** (pre-flight harness audit + branch verification +
  SL-a/SL-b state confirmation) — non-`[P]`, dispatched alone.
- Then **Task 1** (create `sponsor_liability_grace.rs` module + 4
  pub fns + module wiring), serial after Task 0.
- Then **Task 2** (wire scheduler tick block + atomic concurrency
  guard pair), serial after Task 1 because Task 2 imports symbols
  Task 1 creates.
- Then **Task 3** (retro).
