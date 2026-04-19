# Phase 6 — Handoff prompt for overnight autonomous advisor session

Copy everything below the line into a fresh Claude Code session on
the Brehon fork (`C:\Users\barri\Developer\brehon-fork`). Session
should be on `governance-v0` branch; advisor operates from the
auxiliary worktree `../brehon-fork-advisor-phase6`.

---

You are the **advisor** for Brehon governance-fork Phase 6 (federation,
outbound-only). Execute the plan at
`.claude/PRPs/plans/phase-6-federation.plan.md` sequentially, spawning
seven agents (A–G) one at a time via the `Agent` tool, merging each
agent's worktree diff into `phase-6` between spawns. This is an
overnight autonomous run — take your time, verify each step, do not
ask for confirmation on routine decisions.

## Pre-flight state (ALREADY DONE — do not redo)

Performed 2026-04-19 in the prior session:

- PR #10 confirmed MERGED at `156db7cc8` on `governance-v0`.
- `phase-6` branch cut at `3bbf419da` (governance-v0 tip including
  PR #32 e2e pending gates + cargo-test-e2e workflow).
- Scaffolding commit `506563a92` cherry-picked onto `phase-6`:
  task-hopper infra (`.claude/task-hopper.json`, schema, helper,
  rule), Phase 6 plan, wrapper libpq-parity fixes
  (`cargo-check.bat` + `cargo-clippy.bat` now export `PQ_LIB_DIR`
  per `feedback_pq_sys_wrapper_env_propagation.md`), prp-ralph-stop
  hook jq→python3 swap, .gitignore hopper-lock paths.
- Decision-queue pre-seeds committed as `15f8cbbd0`: DQ-6.1 through
  DQ-6.5 all answered with advisor-expected answers
  (`.claude/decision-queue.json` phase bumped to "6").
- Advisor coordination worktree exists at
  `../brehon-fork-advisor-phase6` with submodules initialised.
- Pre-phase harness audit run:
  - Probe 1 (per-crate `-p` honored): ✅ exit 0
  - Probe 2 (`--workspace --features full`): ✅ exit 0 (7m 36s, on retry
    after submodule init)
  - Probe 3 (cargo-test target selection): re-running at handoff —
    check `.claude/audit-phase6-test.log` tail + background task
    `bvuicxwfu` status
  - Probe 4 (wrapper propagates non-zero on bogus feature): ✅ exit 101
  - Probe 5a (`-p lemmy_apub_objects`): in progress at handoff — check
    `.claude/audit-phase6-apub-obj.log`
  - Probe 5b (`-p lemmy_apub_activities`): not yet run
  - Probe 5c (`-p lemmy_apub`): not yet run
  - Probe 6 (phase-6 branch + PR #10 state + log empty vs governance-v0):
    ✅ phase-6 exists, PR #10 MERGED, only the scaffolding +
    DQ-pre-seed commits are ahead of governance-v0.
- `phase-6` branch pushed to `origin/phase-6` so every session sees it.

## Step 0 — verify environment

In the fresh session, run in order and confirm:

```bash
git branch --show-current
# Expected: governance-v0

git worktree list
# Expected: primary on governance-v0, ../brehon-fork-advisor-phase6 on phase-6

cd ../brehon-fork-advisor-phase6 && git log --oneline -3
# Expected top-most: 15f8cbbd0 chore(phase-6): decision-queue pre-seeds ...
# Then: 506563a92 chore(phase-6-prep): task-hopper infra + phase-6 plan ...
# Then: 3bbf419da fix(tests+ci): e2e pending gates + cargo-test-e2e workflow (#32)

ls crates/email/translations/backend/ | head -3
# Expected: several .json locale files — submodules are initialised.

tail -5 .claude/audit-phase6-features.log
# Expected: "Finished `dev` profile" + exit-code marker 0 in the task stdout.

tail -5 .claude/audit-phase6-test.log
# Expected: e2e compile finished; check exit via .output file.

cat .claude/task-hopper.json | python -c "import sys, json; d=json.load(sys.stdin); print('phase:', d['phase']); print('tasks so far:', len(d['tasks']))"
# Expected: phase: 6, tasks so far: 0
```

If any of these diverge from expected, STOP and read
`.claude/PRPs/phase-6-runlog/00-advisor-state.md` +
`.claude/PRPs/phase-6-runlog/01-phase-6-progress.md` to diagnose
drift before spawning any agent.

## Execution plan — sequential agents

All seven agents run **sequentially**. User directive: avoid parallel
execution. Spawn A, wait for A to commit + push, merge into phase-6,
validate, then spawn B. Repeat through G.

| # | Agent | Brief | Tasks | Typical duration |
|---|-------|-------|-------|-----------------|
| 1 | A | `.claude/PRPs/phase-6-runlog/briefs/agent-a.md` | 70, 71 | ~45 min |
| 2 | B | `.claude/PRPs/phase-6-runlog/briefs/agent-b.md` | 72 | ~60 min |
| 3 | C | `.claude/PRPs/phase-6-runlog/briefs/agent-c.md` | 73 | ~60 min |
| 4 | D | `.claude/PRPs/phase-6-runlog/briefs/agent-d.md` | 74 | ~60 min |
| 5 | E | `.claude/PRPs/phase-6-runlog/briefs/agent-e.md` | 75, 78 | ~75 min |
| 6 | F | `.claude/PRPs/phase-6-runlog/briefs/agent-f.md` | 76 | ~30 min |
| 7 | G | `.claude/PRPs/phase-6-runlog/briefs/agent-g.md` | 77 + SUBSCRIPTIONS.md | ~90 min |

Rough upper bound: ~7 hours. Comfortable within an overnight window.

## Spawn pattern

For each agent X (A through G):

### Step 1 — create agent's worktree

From the advisor worktree (primary or `../brehon-fork-advisor-phase6`
— either is fine since we're branching, not checkout-switching):

```bash
cd C:/Users/barri/Developer/brehon-fork-advisor-phase6
git fetch origin  # make sure we have the freshest phase-6
git branch agent-<X>-phase6 phase-6
git worktree add ../brehon-fork-agent-<X>-phase6 agent-<X>-phase6
```

Note: advisor worktree stays on `phase-6` for coordination. Do NOT
checkout-switch the primary worktree (it's on `governance-v0`) —
per `feedback_preserve_active_worktree_state.md`.

### Step 2 — spawn the agent

Use the `Agent` tool with:

- `subagent_type: "general-purpose"`
- `model: "opus"`
- `prompt`: the contents of
  `.claude/PRPs/phase-6-runlog/briefs/agent-<X>.md` verbatim,
  prefixed with:

  ```
  Run at maximum effort — deepest reasoning, most thorough
  exploration. You are continuing a Brehon governance-fork Phase 6
  layered-execution plan.

  ```

  then the brief file's contents.

- `description`: e.g. "Agent A — tasks 70+71 (schema + Diesel models)"

Example pseudocode:

```
prompt = "Run at maximum effort — deepest reasoning, most thorough exploration. You are continuing a Brehon governance-fork Phase 6 layered-execution plan.\n\n" + read(".claude/PRPs/phase-6-runlog/briefs/agent-a.md")
Agent(subagent_type="general-purpose", model="opus", description="Agent A — tasks 70+71", prompt=prompt)
```

The Agent tool blocks until the subagent returns. Its result is a
single message summarising what it did. Verify by inspecting the
worktree diff + task-hopper state + commit history, not by trusting
the summary prose.

### Step 3 — merge agent's branch into phase-6

Advisor runs, in advisor worktree:

```bash
cd ../brehon-fork-advisor-phase6
git fetch origin  # pull the agent's pushed branch
git merge --ff-only origin/agent-<X>-phase6
# or if not FF-able:
# git merge --no-ff origin/agent-<X>-phase6 -m "merge(phase-6): agent-<X> layer <N> — tasks ..."
```

Agent branches are one-shot feature branches; fast-forward is
expected if the agent is the only writer since the prior merge.

### Step 4 — validate merge point

```bash
cd ../brehon-fork-advisor-phase6
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/merge<N>-check.log 2>&1"
echo "check exit: $?"
tail -20 .claude/merge<N>-check.log
```

For merge points 2 and 3, also run clippy:

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/merge<N>-clippy.log 2>&1"
echo "clippy exit: $?"
tail -30 .claude/merge<N>-clippy.log
```

For merge point 3 (after Agent F), also e2e compile:

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/merge3-e2e-compile.log 2>&1"
echo "e2e-compile exit: $?"
tail -20 .claude/merge3-e2e-compile.log
```

For merge point 4 (after Agent G), run the **full** e2e test suite:

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > .claude/merge4-e2e-full.log 2>&1"
echo "full-e2e exit: $?"
tail -50 .claude/merge4-e2e-full.log
```

All must exit 0. If any are non-zero, STOP and investigate before
spawning the next agent.

### Step 5 — push + cleanup

```bash
git push origin phase-6
git worktree remove ../brehon-fork-agent-<X>-phase6
git branch -D agent-<X>-phase6  # local cleanup; remote branch stays
```

### Step 6 — log in progress file

Append a dated entry to
`.claude/PRPs/phase-6-runlog/01-phase-6-progress.md` with the agent's
commit SHAs + merge SHA + validation exit codes.

## After all seven agents merged

### Phase-6 completion report

Write `.claude/PRPs/reports/phase-6-complete-report.md` following the
Phase 5c template (`.claude/PRPs/reports/phase-5c-complete-report.md`).
Sections:

1. What shipped (9 tasks, 7 agents, 5 layers collapsed to 7 sequential
   worktrees)
2. What we learned (sequential-vs-parallel retrospective; did
   per-agent worktrees help or hurt?)
3. Carry-forwards — v1 items surfaced: peer allowlist table,
   trust-attestation wiring into endorsement handler, HTTP signature
   capture (DQ-6.2 deferred), env-var cleanup (DQ-6.4 deferred)
4. DQ resolutions summary
5. Reviewer notes for the PR

### Open PR

```bash
cd ../brehon-fork-advisor-phase6
gh pr create --repo barrie-cork/lemmy \
  --base governance-v0 --head phase-6 \
  --title "Phase 6 — Federation objects & activities, outbound-only" \
  --body "$(cat <<'EOF'
## Summary
- 11 MVP endpoints already live; Phase 6 closes the v0 federation DoD
- Two new tables: federation_attestation + remote_sanction_notice
- Three AP object types (SanctionNotice, TrustAttestation, ModerationLabel)
- Outbound publisher: send_local_sanction_notice + send_local_trust_attestation
- Inbound receiver: advisory-only, no auto-apply (ADR-006)
- submit_jury_vote wired to publish when scope == FederatedRecommendation
- Two-Postgres round-trip e2e (sanction_notice_round_trip)

## Completion report
`.claude/PRPs/reports/phase-6-complete-report.md`

## Plan reference
`.claude/PRPs/plans/phase-6-federation.plan.md`

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

Per `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy`
is mandatory on forks.

Per `.claude/rules/phase-branch.md` — do NOT open as draft
(CodeRabbit skips drafts).

### Save memory

After PR is open, save a memory entry
`project_brehon_phase_6_complete.md` with the PR URL, phase-6 commit
SHA range, agent-by-agent task assignments, and any surprises worth
remembering.

## Abort conditions (STOP and hand back to user)

- Any merge-point validation (check / clippy / e2e-compile) red.
- An agent's task-hopper escalates (retry cap hit — `status ==
  escalated` in `.claude/task-hopper.json`).
- An agent writes a DQ entry that isn't pre-seeded — surface for user
  input; do not answer your own agent's question.
- Cargo wrapper returns exit 0 on an obvious failure (regression of
  issue #8 / commit bb254e733). Pre-phase audit probe 4 passed, but
  if you see this during the run, STOP.
- Conflict between agent-<X>-phase6 and phase-6 on merge (non-FF and
  unclear which side is correct). Investigate.
- Discrepancy between a DQ answer and the plan text. Surface before
  applying.

## Reference state files

- `.claude/PRPs/plans/phase-6-federation.plan.md` — canonical plan
- `.claude/PRPs/phase-6-runlog/00-advisor-state.md` — live state
  snapshot
- `.claude/PRPs/phase-6-runlog/01-phase-6-progress.md` — append-only
  log (this session appends on each merge)
- `.claude/PRPs/phase-6-runlog/briefs/agent-<A..G>.md` — per-agent
  self-contained briefs
- `.claude/decision-queue.json` — DQ-6.1..6.5 pre-answered
- `.claude/task-hopper.json` — per-task execution ledger
- `scripts/brehon/task-hopper.sh` — helper CLI
- `.claude/rules/task-hopper.md` — hopper protocol
- `.claude/rules/phase-branch.md` — branch discipline
- `.claude/rules/cargo-output-capture.md` — exit-code safety
- `.claude/rules/no-cargo-output-paste.md` — context budget
- `CLAUDE.md` — fork-wide constraints, design-doc pointers

## Agent tool call templates (copy-paste ready)

### Agent A

```
Agent(
  subagent_type="general-purpose",
  model="opus",
  description="Agent A — Phase 6 Layer 1 tasks 70 + 71",
  prompt=<contents of .claude/PRPs/phase-6-runlog/briefs/agent-a.md verbatim>
)
```

### Agent B (after Agent A merged)

```
Agent(
  subagent_type="general-purpose",
  model="opus",
  description="Agent B — Phase 6 Layer 2 task 72 (AP objects)",
  prompt=<contents of agent-b.md verbatim>
)
```

… same pattern for C, D, E, F, G.

Do not try to batch multiple agents. One at a time, sequential.
