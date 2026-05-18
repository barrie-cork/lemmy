---
phase: v1-ship-1
role: impl-task
task: 0
brief_n: 0
plan: .claude/PRPs/plans/v1-ship-1-r2.plan.md
created: 2026-05-18
related_dq: null
---

# [role:impl-task] v1-ship-1-r2 Task 0 pre-flight — see .claude/PRPs/briefs/v1-ship-1-impl-0.md

> **Clarify provenance:** this brief's parent **planning** brief
> (`v1-ship-1-r2-planning-1.md`) was clarified before the planning task
> (#309) was queued. DQ #261 (`from: "planner"`, `kind: "log"`,
> RESOLVED) records the Option(b) mechanism-precision correction baked
> into the r2 plan. Task 0 authors no code — read DQ #261 only for
> orientation; it does not gate this task.

## 0. Pre-flight (subagent runs this before reading anything else)

The `impl-task` agent already runs the forbidden-window check from
`.claude/agents/impl-task.md` "Task-0 pre-flight". This brief inherits
that — do not duplicate the bash. No `forbidden-window-override` on the
dispatch line (advisor confirmed Mon ~12:2x UTC is inside the secondary
window 04:30–14:59).

## 1. Role + dispatch line

`[role:impl-task] v1-ship-1-r2 Task 0 pre-flight — see .claude/PRPs/briefs/v1-ship-1-impl-0.md`

You are the **impl-task** subagent (Sonnet 4.6 per your frontmatter).
Execute plan Task 0 from `.claude/PRPs/plans/v1-ship-1-r2.plan.md` §13
(Task 0, lines ~479-577). **Verification only** — environment +
topology + merged-surface sanity for the AGPL §13 source-disclosure
e2e rebuild.

## 2. Scope

Run **Probes 0–15** from plan §13 Task 0 **in order**. Report each
probe result (PASS / FAIL / WARN) with its actual output. **No commit
at Task 0 — verification only. No files changed. No code authored.**

The plan §13 Task 0 fenced "Probes" block (lines ~493-569) is the
**contract**. Run those exact commands. The bullet list below is
orientation + the WARN-vs-blocker classification.

**TWO PROBES ARE REWORDED for worktree-awareness (Probes 1 + 3).** You
run on a Junior worktree branch (`junior/role-impl-task-...`), NOT on
`phase-v1-ship-1` directly, and a local `governance-v0` ref may be
absent or moved on the daemon checkout. The plan's literal Probe 1
(`= phase-v1-ship-1`) and Probe 3 (`--is-ancestor governance-v0 HEAD`)
would spuriously fail on a worktree → you would self-downgrade them to
WARN (defensible, but a literal-contract deviation). **Use the
reworded forms below — they ARE the contract for this brief and
override the plan's literal text for Probes 1 and 3 ONLY.** All other
probes (0, 2, 4–15) run verbatim from the plan block.

- **Probe 0** — `docker ps > /dev/null 2>&1`. Docker daemon running
  (Shape G suspended → e2e runs locally later, so Docker presence
  matters). FAIL → **blocker (STOP)**.
- **Probe 1 (REWORDED — worktree-aware)** — branch is the phase branch
  OR a worktree of it. Run:
  ```bash
  b="$(git branch --show-current)"
  if [ "$b" = "phase-v1-ship-1" ] || echo "$b" | grep -qE '^junior/.*ship-1'; then
    echo "BRANCH OK ($b)"
  else
    echo "WRONG BRANCH ($b)"; exit 1
  fi
  ```
  FAIL → **blocker (STOP)**.
- **Probe 2** — working tree clean (`git status --porcelain` empty).
  FAIL → **WARN** (a finalize/daemon step may be mid-flight; report
  the dirty paths, continue).
- **Probe 3 (REWORDED — pinned bm-cut SHA, not moving governance-v0)**
  — phase branch descends from the recorded bm-cut commit. Run:
  ```bash
  git merge-base --is-ancestor 8c271285e HEAD && echo "BASE OK (descends bm-cut 8c271285e)" || { echo "BASE DRIFT"; exit 1; }
  ```
  `8c271285e` = the v1-ship-1 bm-cut commit (`docs(advisor): v1-ship-1
  bm-cut brief — cut phase-v1-ship-1 off governance-v0`), verified by
  the advisor to be an ancestor of the current phase tip. FAIL →
  **blocker (STOP)**.
- **Probe 4** — `AGPL-NOTICE.md` at repo root, > 1000 bytes (Task 3's
  `include_str!` target). FAIL → **blocker (STOP)**.
- **Probe 5** — `source_disclosure: SourceDisclosure {` present in
  `crates/api/api_crud/src/site/read.rs` (merged Task 1+2 surface;
  EXPECT one line, currently ~:74; drift OK — symbol presence is the
  contract). Absent → **blocker (STOP)**.
- **Probe 6** — `crates/api/api_crud/build.rs` present AND contains
  `BREHON_FORK_COMMIT` (merged Task 2). FAIL → **blocker (STOP)**.
- **Probe 7** — merged Task 3 surface: `crates/api/api/src/site/source.rs`
  present + `pub async fn get_source` in it + `pub mod source;` in
  `crates/api/api/src/site/mod.rs` + `"/source", get().to(get_source)`
  in `crates/api/routes/src/lib.rs`. Any missing → **blocker (STOP)**.
- **Probe 8** — failed test fn present:
  `^async fn agpl_source_disclosure_surface_returns_notice\(\) -> lemmy_utils::error::LemmyResult<\(\)>`
  in `crates/server/tests/e2e.rs` (Task 6's Edit target; EXPECT one
  line near :14865, drift ±50 OK — symbol presence is the contract).
  Absent → **blocker (STOP)**.
- **Probe 9** — canonical mirror anchors at `crates/server/src/lib.rs`:
  (a) `let context: LemmyContext = federation_config.deref().clone();`
  near :364; (b) `.app_data(Data::new(context.clone()))` near :379;
  (c) `.wrap(FederationMiddleware::new(federation_config.clone()))`
  near :380. Report each line. Any anchor absent → **blocker (STOP)**
  (Task 6's byte-mirror source would be invalid).
- **Probe 10** — `context.rs` fixture anchors:
  `pub async fn init_test_federation_config() -> FederationConfig<LemmyContext>`
  near :65 + `pub async fn init_test_context() -> Data<LemmyContext>`
  near :97 in `crates/api/api_utils/src/context.rs`. Report lines.
  Absent → **WARN** (informational; plan §10 mirrors `lib.rs`, not
  `context.rs`).
- **Probe 11** — exactly **2** `^  pub async fn bootstrap() -> LemmyResult<(`
  in `crates/server/tests/e2e.rs` (DQ #226 sanity:
  `governance_fixtures` + `admin_config_fixtures`); first hit near :801
  (the chosen `governance_fixtures::bootstrap`). Count ≠ 2 →
  **blocker (STOP)**.
- **Probe 12** — `activitypub_federation` in `Cargo.lock` still
  `version = "0.7.0-beta.11"` (DQ #258 sanity). Drifted → **blocker
  (STOP)** (re-verify DQ #258 against new version before Task 6).
- **Probe 13** — concurrent-PR check: the `gh pr list --repo
  barrie-cork/lemmy` jq from the plan block (any open PR touching
  `crates/server/tests/e2e.rs`). Non-empty → **blocker (STOP)** —
  file-ownership conflict; surface the colliding PR number.
- **Probe 14** — wrapper-script positive: `cmd //c
  "scripts\\brehon\\cargo-check.bat -p lemmy_server > .claude/audit-cargo-check-p.log 2>&1"`.
  EXPECT exit 0. Non-zero → **blocker (STOP)** (the e2e validation
  wrapper is broken; Task 6's later cargo gate would be unreliable).
  Report exit + last 5 log lines.
- **Probe 15** — wrapper-script negative (exit-code propagation):
  `cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-check-negative.log 2>&1"`.
  EXPECT **non-zero** (typically 101). If **0** → wrapper masks exit
  codes → **blocker (STOP)** (per `feedback_pipes_mask_exit_codes.md` /
  `feedback_batch_goto_eof_clobbers_errorlevel.md`). Report the exit.

**Blocker probes (STOP + file `kind: "blocker"` DQ, `from: "impl"`):**
0, 1 (reworded), 3 (reworded), 4, 5, 6, 7, 8, 9, 11, 12, 13, 14, 15.
**WARN-only (report, continue):** Probe 2 (dirty tree), Probe 10
(`context.rs` informational anchors). Rationale: every probe except 2
and 10 verifies a load-bearing precondition for the Task 6 e2e
rebuild; Probe 2 may legitimately be non-empty during a daemon
finalize, and Probe 10's anchors are informational (plan §10 mirrors
`lib.rs`, not `context.rs`).

Note Probes 14-15 write `.claude/audit-cargo-check-*.log` — these are
gitignored audit logs (the `.claude/audit-*.log` debug class), NOT
tracked `.claude/` config. Writing them is expected and not a
sensitive-file violation.

## 3. Required reading

In this order:

1. **`.claude/PRPs/plans/v1-ship-1-r2.plan.md` §13 Task 0** (lines
   ~479-577) — the verbatim Probes 0–15 fenced block. Run those exact
   commands **except Probes 1 and 3**, which use the reworded forms in
   §2 of this brief (worktree-awareness; §7 explains why).
2. **`.claude/rules/pre-phase-harness-audit.md`** — R5 audit shape
   (enumerate ALL probes explicitly; no silent skips; report every
   probe even on early blocker).
3. **`.claude/rules/decision-queue.md`** — Recipe 1 (blocker DQ) +
   "Mid-task visibility" (commit + push the DQ immediately if filed) +
   the next-id-spans-archives rule (DQ #50 collision lesson).

## 3a. Handover from prior cohort

(none — Task 0 is the first task of the r2 cycle; no prior cohort.
Tasks 1–4 are MERGED carry-forward context, NOT re-executed — Probes
5/6/7/8 confirm their presence on the phase branch.)

## 4. Constraints

- **No commit.** Task 0 is verification only — no files changed (the
  Probe 14/15 audit logs are gitignored and not committed), no
  `git commit`, no worktree tip movement beyond what the daemon cuts.
- **No code authoring.** Do not edit `crates/**`, `migrations/**`,
  `tests/**`, `docs/**`, or any tracked `.claude/**` file. The ONLY
  permitted `.claude/` write is `.claude/decision-queue.json` IF a
  blocker DQ is filed (the mid-task-visibility exception).
- **DQ mid-task push:** if a blocker DQ is filed:
  ```bash
  git add .claude/decision-queue.json
  git commit -m "chore(decision-queue): impl raised DQ #<id> — <slug>"
  git push origin <worktree-branch>
  ```
  immediately per `decision-queue.md` "Mid-task visibility". Compute
  `next_id` across `.claude/decision-queue.json` +
  `.claude/decision-queue-archive-*.json` (DQ #50 collision lesson).
- **Attribution:** DQ entries use `from: "impl"`, `answered_by: null`,
  `kind: "blocker"`. NEVER `from: "advisor"` / `answered_by:
  "advisor"` / `answered_by: "user"`. NEVER `kind: "clarify"`
  (advisor-only).
- **Report ALL 16 probes even after an early blocker.** Per R5: do not
  short-circuit the report. Run remaining probes where mechanically
  possible; for probes that cannot run because an earlier blocker
  invalidated a precondition, mark `SKIPPED (blocked by Probe <N>)`.
- **Shape G note:** Task 0 has NO `cargo check`/`clippy`/`test`
  invocation that gates the task (Probes 14/15 are wrapper-sanity
  one-offs, not the §15 DoD gate). The repo-wide Shape-G suspension
  (DQ #229, until 2026-06-01) does not affect Task 0 — it only affects
  Task 6's validation handoff (`kind: "validate-pending-laptop"`, not
  this task). Task 0 writes NO `validate-pending*` entry.

## 4.1 CANONICAL CASE OVERRIDE

Not applicable — Task 0 authors no e2e test. No `LemmyResult` /
`Box<dyn Error>` Case A/B/C decision required (that is Task 6's
concern; plan §10.5/§10.6/§10.7 pre-specify the canonical `lib.rs`
byte-mirror idiom).

## 5. Validation gates

**None.** Task 0 is pure verification — no §15 DoD cargo gate, no
`validate-pending*` DQ entry. Probes 14/15 are internal
wrapper-sanity probes whose results are part of the §6 report, not a
separate validation handoff.

## 6. Expected output (return to advisor)

```
## Task 0 complete — v1-ship-1-r2 pre-flight harness audit

**Probes 0-15:**
  - Probe 0  (Docker daemon):                 PASS/FAIL
  - Probe 1  (branch = phase OR junior/*ship-1): PASS/FAIL — <branch>
  - Probe 2  (tree clean):                    PASS/WARN — <dirty paths if WARN>
  - Probe 3  (descends bm-cut 8c271285e):     PASS/FAIL
  - Probe 4  (AGPL-NOTICE.md >1000B):         PASS/FAIL — <byte count>
  - Probe 5  (read.rs source_disclosure):     PASS/FAIL — <line>
  - Probe 6  (api_crud/build.rs BREHON_FORK): PASS/FAIL
  - Probe 7  (Task 3 surface: source.rs+mod+route): PASS/FAIL
  - Probe 8  (failed e2e fn present):         PASS/FAIL — <line ~14865>
  - Probe 9  (lib.rs mirror anchors a/b/c):   PASS/FAIL — <3 lines>
  - Probe 10 (context.rs fixture anchors):    PASS/WARN — <lines>
  - Probe 11 (exactly 2 bootstraps):          PASS/FAIL — <count>, first ~:801
  - Probe 12 (activitypub_federation 0.7.0-beta.11): PASS/FAIL — <version>
  - Probe 13 (concurrent-PR check):           PASS/FAIL — <colliding PRs or empty>
  - Probe 14 (wrapper positive exit 0):       PASS/FAIL — <exit>
  - Probe 15 (wrapper negative non-zero):     PASS/FAIL — <exit>
**Verdict:** ALL PASS (proceed to Task 6) | BLOCKER on Probe <N> (DQ #<id>)
**No commit** (Task 0 verification only).
**Next:** advisor queues Task 6 (e2e App-construction rebuild on the
canonical lib.rs:228-241/364/379-382 idiom).
```

Plus the DQ #N reference if a blocker was filed.

## 7. Why this brief differs from the plan

**Two probes reworded (Probes 1 + 3); no other deviations.**

1. **Probe 1 — branch check made worktree-aware.** The plan's literal
   `test "$(git branch --show-current)" = "phase-v1-ship-1"` assumes
   the probe runs on the phase branch. Junior workers run on a
   per-task worktree branch (`junior/role-impl-task-...`), so the
   literal form fails spuriously and the worker self-downgrades it to
   WARN — a defensible but literal-contract-violating deviation
   (observed in the r1 cycle; carried lesson per the v1-ship-1
   reconciled retro orig-id-1). The reworded form accepts the phase
   branch OR any `junior/*ship-1` worktree branch, keeping it a true
   blocker without false positives.

2. **Probe 3 — ancestry pinned to the bm-cut SHA, not the moving
   `governance-v0` ref.** The plan's literal `git merge-base
   --is-ancestor governance-v0 HEAD` is fragile: a Junior worktree (or
   the daemon checkout parked on another lane) may have no local
   `governance-v0` ref, or a moved one, so the probe fails spuriously
   and is self-downgraded to WARN. The advisor verified `8c271285e`
   (the v1-ship-1 bm-cut commit) is an ancestor of the current phase
   tip `c9036d24c`; pinning that immutable SHA makes Probe 3 a stable
   true blocker. (Same false-blocker class as Probe 1.)

The WARN-vs-blocker split in §2/§4 codifies the plan's intent (the
plan marks blockers via `exit 1` and soft probes via comment-only
handling); this brief makes the split unambiguous for the subagent.
The Shape-G-suspension context is session state from the bootstrap,
not a plan deviation — Task 0 runs no §15 cargo gate so the suspension
is immaterial here.
