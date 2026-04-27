# Handover — v1-validate-agent foreground execution paused mid-phase

**Author:** foreground impl session (laptop, brehon-fork CWD, branch `phase-v1-validate-agent`)
**When:** 2026-04-27, after Task 2 commit pushed at `000f86093`
**Mode:** foreground impl (Option A from session opening — not the four-role Junior orchestrator)
**Reason:** user requested handover after Task 2 to switch sessions

## Bootstrap prompt (paste into next session)

> You are picking up `v1-validate-agent` mid-phase as a foreground impl session in the brehon-fork CWD. The branch `phase-v1-validate-agent` has Tasks 0–5 shipped (6 commits ahead of `governance-v0`). Tasks 6 (retro) + PR + CR triage + merge remain. **First action:** read `.claude/PRPs/handovers/impl-2026-04-27-v1-validate-agent-tasks-3to5-and-2-shipped.md` (this file) end-to-end, then `.claude/PRPs/plans/v1-validate-agent.plan.md` §13 Task 6 + §17 Completion checklist + §16a Stories. **DO NOT run `git checkout` or branch ops until after reading this file.**

## Closing state assertions (verify these in next session before any write)

```bash
git branch --show-current
# EXPECT: phase-v1-validate-agent

git status --short
# EXPECT: empty (clean working tree)

git log governance-v0..HEAD --oneline
# EXPECT (in this order, oldest to newest):
#   1b8d0061d feat(ci): cargo-validate workflows for phase-v1-* + junior/* push triggers
#   ed049970b fix(ci): drop invalid --no-deps from cargo check step
#   70b3a4d08 feat(rules,agents): schema-additive validate-* kinds + impl-task push-and-exit + advisor validate-stage
#   97a7c99d6 docs(template): plan.template §15 DoD per workflow + §13 Shape G composition note
#   6e03249fa docs(briefs): jm-d-impl-2 §5 retrofit — push-and-exit per Shape G
#   000f86093 feat(agents): ci-watcher subagent + brief template (Haiku 4.5, low) + clippy fix

git log --oneline phase-v1-validate-agent..origin/phase-v1-validate-agent
# EXPECT: empty (everything pushed)
```

## What shipped (Tasks 0–5)

### Task 0 — Pre-flight harness audit
All 9 probes passed (branch, gh auth as `barrie-cork`, gh repo access, gh workflow `--yaml` flag, gh workflow help, gh run watch `--exit-status` flag, concurrent-PR check, DQ pending=0, four target files absent). No commit (verification only).

### Task 1 — Workflow YAMLs (`1b8d0061d` + `ed049970b`)
- `.github/workflows/cargo-validate-workspace.yml` — cargo check + clippy + test --no-run on push to `phase-v1-*` / `junior/*`. Cache prefix `cargo-validate-workspace-`. timeout 45 min.
- `.github/workflows/cargo-validate-migration.yml` — migration round-trip on push touching `migrations/**`. Cache prefix `cargo-validate-migration-`. timeout 30 min. Calls `bash scripts/brehon/migrate-roundtrip.sh`.
- `scripts/brehon/migrate-roundtrip.sh` — stub; exits non-zero if a real migration is detected vs `governance-v0`, forcing the next migration-authoring sub-phase to replace it. Logged as DQ #68 (planning-side phantom dependency).
- Fix-up commit `ed049970b`: dropped `--no-deps` from `cargo check` step (it's a clippy-only flag; the plan inherited it from clippy DoD wording). Logged as DQ #69.

### Task 2 — ci-watcher subagent (`000f86093`)
- `.claude/agents/ci-watcher.md` — Haiku 4.5, low effort, narrow tools (Read, Edit, Write, Bash). Frontmatter mirrors `bm-task.md`. Body has 6 sections (Before-you-start, Action-sequence, Empirical exit-code table, validate-result/failed entry shapes, Hard refusals, Output discipline).
- `.claude/PRPs/templates/ci-watcher-brief.template.md` — minimal three-field brief with classifier sequence. Citation-only Hard refusals (defers to ci-watcher.md body).
- **Empirical probe finding (DQ #70, load-bearing):** on gh CLI 2.89.0, `gh run watch <id> --exit-status` returns **exit 0** in all three observed terminal scenarios (success, completed-failure, in-progress→failure-watched-live). Despite `--help` text. ci-watcher MUST disambiguate via `gh run view <id> --json conclusion --jq '.conclusion'` post-watch. The exit code is unreliable for pass/fail; only the conclusion string is.
- Probes captured in `.claude/PRPs/debug/v1-validate-agent-task2-gh-run-watch-probes.log` (gitignored as `*.log`; the durable deliverable is the table inside ci-watcher.md).
- Cancelled + queued empirical probes deferred (conclusion-string fallback covers all eight enumerable conclusion values).
- Same commit also adds `rustup component add clippy --toolchain 1.95-x86_64-unknown-linux-gnu` step in cargo-validate-workspace.yml — `dtolnay/rust-toolchain@master` did not honour `components: clippy` directive when rust-toolchain.toml pins `1.95`.

### Task 3 — Schema-additive atomic commit (`70b3a4d08`)
Three files, one commit per `feedback_update_rule_doc_with_schema_additive.md`:
- `.claude/rules/decision-queue.md`: kind sub-section title extended to enumerate validate-pending/-result/-failed; new kind definitions; polling-loop routing per kind gains 6 new (kind, status) routes; Hard refusal #7 added (never write validate-result/failed from non-ci-watcher); Subagents-and-attribution gains ci-watcher as fifth subagent. impl-task continues writing as `from: "impl"` even with `kind: "validate-pending"` per DQ #63.
- `.claude/agents/impl-task.md`: H2 renamed `Per-task validation gate (out-of-band on GH Actions)`. Body rewritten to push-and-exit (capture workflow_run_id via gh run list, append validate-pending DQ entry, commit + push, exit). Pre-Shape-G plans (v1-JM-d and earlier) keep inline cargo per forward-only-immune rule. Hard refusals append: "Never invoke cargo for build/lint/test on Shape-G plans".
- `.claude/rules/advisor-orchestrator.md`: Stage-shape gains "Each impl-task complete (under Shape G)" + "ci-watcher complete" transitions. Forbidden execution windows gains Shape-G note (cargo no longer local for impl-task throughput; binding for advisor DoD smoke + pre-Shape-G dispatches + user-authorised diagnostics only). Catch-fire procedures gain 60-min ci-watcher cap + classifier-miss exit-code triggers. Cohort dispatch gains parallel-validate-pending clause. New "§G4 classifier" sub-section: allowlist (`clippy::doc_lazy_continuation`, missing imports `error[E0432]`, deprecated APIs) ≤3 file edits, non-allowlist failures catch-fire to user.

### Task 4 — Plan template extension (`97a7c99d6`)
- `.claude/PRPs/templates/plan.template.md`: appended §15.6 "DoD per workflow (Shape G plans — v1-JM-e onward)". §13 intro paragraph gained Shape G composition note. Existing §15.1–§15.5 unchanged (no renumbering — pre-existing plans cite by number).

### Task 5 — jm-d-impl-2.md §5 retrofit (`6e03249fa`)
- `.claude/PRPs/briefs/jm-d-impl-2.md` §5 only: switched body from inline cargo (cargo-check.sh + cargo-clippy.sh + cargo-test.sh) to Shape G push-and-exit. Heading retitled to "Validation gates (out-of-band on GH Actions per Shape G)". GOTCHAs preserved but reframed for the GH Actions failure surface. **No other JM-d brief edited** per DQ #61's "single-brief bounded retrofit" wording.

## What's pending

### Task 6 — Retro (next concrete action)
Author `.claude/PRPs/reports/v1-validate-agent-retro.md` per plan §13 Task 6:
- §1 Summary
- §2 Per-role signals (Advisor / Planning / Impl / BM) — but note this was **foreground impl, not four-role**, so §2 should reflect that explicitly (the planned four-role retro signals don't all apply)
- §3 Per-task complexity score table (`<files>/<commits>/<runtime-min>/<max-log-silence-min>` for Tasks 1..6)
- §4 Lessons promoted to `.claude/lessons/` — strong candidates listed below
- §5 Watch-items for next sub-phase (v1-JM-e first under Shape G)
- §6 Confidence score N/10

Commit subject: `docs(retro): v1-validate-agent retro + lessons promoted`

### PR + CR triage (BM-equivalent, foreground)
After retro commit:
1. `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-v1-validate-agent --title "v1-validate-agent — Out-of-Junior cargo validation via GH Actions" --body <heredoc-from-§17>`. NOT draft (CodeRabbit skips drafts).
2. Wait for CodeRabbit auto-review (per `.coderabbit.yaml`).
3. Run `/brehon-verify v1-validate-agent` to iterate the §16a Stories — expect Story 1 + Story 2 + Story 3 ✓ (Stories 4 + 5 are gated on a real failure surfacing during validate; if not exercised, mark `[deferred-to-retro]` per plan §16a). Story 6 ships only after bm-merge.
4. Surface CR triage to user with four-bucket categorization (fix-in-pr / rebut / carry-forward / done) + verify report.
5. Wait for user approval before any fix-in-PR commits and merge confirm.

### Merge + DQ #61 unblock confirmation
After user merge-confirm:
1. `gh pr merge --repo barrie-cork/lemmy --merge` (NOT --squash per `.claude/rules/phase-branch.md` "Do not squash the PR at merge — the task-per-commit history is load-bearing for retros").
2. Confirm `governance-v0` contains all 7 commits.
3. Phase branch retained per JM-b/c/d precedent.
4. Surface to user: "validate-agent bm-merge unblocks JM-d Task 2 queueing per DQ #61. Advisor on next homeserver-side polling tick can dispatch JM-d Task 2 against the now-Shape-G-compliant `jm-d-impl-2.md` brief."

## DQ entries opened during this session (all self-resolved as `kind: log`)

- **DQ #68** (`impl-self-resolved`, 21:20Z) — `migrate-roundtrip.sh` phantom dependency; shipped stub.
- **DQ #69** (`impl-self-resolved`, 21:35Z) — plan §13 Task 1 inherited invalid `--no-deps` flag for `cargo check`; dropped from workflow YAML.
- **DQ #70** (`impl-self-resolved`, 22:00Z) — `gh run watch --exit-status` returns exit 0 in all three observed terminal scenarios on gh CLI 2.89.0; ci-watcher MUST disambiguate via `gh run view <id> --json conclusion`. Load-bearing for ci-watcher.md design.

All three are in `resolved[]` (per `kind: log` discipline). The retro should harvest #69 + #70 into `.claude/lessons/` candidates (see "Lessons to promote" below).

## Workflow run state (latest 4 on phase-v1-validate-agent)

```
25017554049 cargo-validate-workspace failure (cargo check pass, clippy step missing toolchain — fixed in 000f86093)
25017407659 cargo-validate-workspace failure (cargo check --no-deps unknown flag — fixed in ed049970b)
25017407689 cargo-validate-migration success (stub script exit 0 — no migrations detected)
25017406623 claude-code-action.yml failure (unrelated workflow, not authored by us)
```

The Task 2 push (`000f86093`) at 22:00Z **should have triggered a fresh `cargo-validate-workspace` run** that may still be running or freshly completed when the next session picks up. Run `gh run list --repo barrie-cork/lemmy --branch phase-v1-validate-agent --limit 5 --json status,conclusion,databaseId,createdAt,workflowName` first thing.

If the latest run **still failed** despite the clippy fix, investigate before authoring the retro — the retro shouldn't claim "Story 1 ✓ workflow runs green on phase-v1-validate-agent" if it doesn't.

## Lessons to promote (candidates for `.claude/lessons/`)

Three lessons emerged during this session that survive past v1-validate-agent and should be filed as `.claude/lessons/feedback_*.md`:

1. **`feedback_gh_run_watch_exit_status_unreliable.md`** — empirical: gh CLI 2.89.0 `gh run watch --exit-status` returns exit 0 in all observed terminal scenarios. Always disambiguate via `gh run view <id> --json conclusion`. Body: Why (the flag's --help text is misleading; observed behaviour contradicts), How to apply (any subagent/script that watches a workflow run), Generalises to (any `gh` CLI version-pinned check needs empirical validation), Symptom to recognise (false-green workflow result).

2. **`feedback_dtolnay_rust_toolchain_components_with_toml.md`** — empirical: `dtolnay/rust-toolchain@master` does NOT honour `components: clippy` action input when `rust-toolchain.toml` pins a channel. Workaround: explicit `rustup component add clippy --toolchain <channel>-<target>` step. Body shape mirrors above.

3. **`feedback_planner_dod_dry_run_caught_partial.md`** — meta-lesson: the plan's §15 dry-run discipline catches YAML parse errors but cannot catch invalid cargo flags inside the YAML's `run:` body. Plan §13 Task 1 prescribed `cargo check --workspace --features full --no-deps`; YAML parses fine; first push fails on the runner. Pattern: plan-side dry-run is necessary but insufficient; first-push validation is the real gate. Generalises to: any DoD command embedded in a YAML/script that the planner doesn't actually invoke during plan-write.

## Foreground vs four-role notes for the retro

This session was **foreground impl, not four-role Junior**. Implications for retro §2:

- **Advisor signals** — N/A (no advisor session). The persistent advisor lives in `homeserver/` and didn't participate. The user gates (plan approval, CR triage, merge confirm) are still real but happened/will happen in-conversation rather than via polling-loop surfacing.
- **Planning signals** — apply normally. The plan's accuracy on §13 Task 1 missed the `--no-deps` flag issue and the `migrate-roundtrip.sh` phantom dependency; both surfaced during impl. Watchpoint accuracy was good (#1 about `gh run watch` was load-bearing and gated correctly).
- **Impl signals** — this session played impl. Note the foreground-vs-Junior delta: no `[role:impl-task]` dispatch line, no Junior worktree isolation, no automatic mid-task push (I had to remember to push manually). Cohort dispatch was sequential not parallel since I'm one session.
- **BM signals** — N/A this session; BM-equivalent work (PR creation + CR triage + merge) is still pending.

The retro should explicitly call out which signals were exercised under foreground vs which would have been exercised under four-role. This frames v1-JM-e (first sub-phase intended for fully four-role under Shape G) as the real test of the orchestration pieces.

## Open risks for next session

- **Task 2's empirical probes covered 3 of 4 plan-prescribed scenarios** (success / failure / in-progress→failure). Cancelled + queued were deferred. Plan §15.4 expected probes log present (✓) and exit-code table no-TBDs (✓ after rephrase). Story 4 in §16a — "ci-watcher classifies a failure workflow as validate-failed with the failing log slice attached" — was not exercised end-to-end (would require writing a fake `validate-pending` DQ entry and dispatching ci-watcher against it). Mark `[deferred-to-retro]` if not exercised before merge.
- **Story 5 (advisor §G4 classifier auto-queues fix-impl-task)** — gated on a real failure matching the allowlist. The clippy-toolchain-missing failure observed in 25017554049 is NOT allowlist-eligible (it's a workflow-config error, not a code-level lint), so even if the advisor classified it, it would catch-fire. Mark `[deferred-to-retro]`.
- **Story 6 (validate-agent bm-merge unblocks JM-d Task 2)** — only verifiable post-merge.
- The plan's §16a Story 1 acceptance is "first post-Task-1 push to phase-v1-validate-agent triggered cargo-validate-workspace". Three runs have triggered. **Whether any reached `conclusion: success`** depends on the latest run (post-`000f86093`). If still red, Story 1 is partially exercised (trigger ✓, run-green ✗); the retro must address.

## Files-changed summary (cumulative on phase branch)

```
.claude/agents/ci-watcher.md                                            (new, 134 lines)
.claude/agents/impl-task.md                                             (modified — H2 + body + 1 hard refusal)
.claude/decision-queue.json                                             (modified — 3 new resolved entries: #68, #69, #70)
.claude/PRPs/briefs/jm-d-impl-2.md                                      (modified — §5 only)
.claude/PRPs/templates/ci-watcher-brief.template.md                     (new, 30 lines)
.claude/PRPs/templates/plan.template.md                                 (modified — §15.6 + §13 note)
.claude/rules/advisor-orchestrator.md                                   (modified — Stage-shape, Forbidden, Catch-fire, Cohort, §G4)
.claude/rules/decision-queue.md                                         (modified — kinds, routing, refusals, attribution)
.github/workflows/cargo-validate-migration.yml                          (new, 53 lines)
.github/workflows/cargo-validate-workspace.yml                          (new, 76 lines)
scripts/brehon/migrate-roundtrip.sh                                     (new, 41 lines)
```

Total: 4 new files, 7 modified files, 6 commits, ~430 net new lines.

## See also

- `.claude/PRPs/plans/v1-validate-agent.plan.md` — the plan being executed (especially §13, §15, §16a, §17)
- `.claude/PRPs/handovers/advisor-2026-04-27-v1-validate-agent-planning.md` — prior session's handover (planning closure → impl entry)
- `.claude/PRPs/debug/v1-validate-agent-task2-gh-run-watch-probes.log` — local diagnostic log (gitignored)
- `.claude/decision-queue.json` — DQ #68/#69/#70 self-resolved entries from this session
