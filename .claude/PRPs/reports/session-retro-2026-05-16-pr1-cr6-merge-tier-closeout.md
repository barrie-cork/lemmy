# Session retro — 2026-05-16 — pr1-cr6-merge-tier-closeout

**Harness:** claude-code
**Session window:** ~2026-05-16T00:10Z → ~01:30Z (~80 min, plus a long-poll wait segment)
**Branch at start:** `116e87db3` (`governance-v0`) ; lane worktree on `69f88ee8f` (`chore/refactor-e2e-error-types`)
**Branch at end:** `ae52dc0cd` (`governance-v0`) ; lane worktree removed
**Files touched:** 2 explicit (`.claude/decision-queue.json` on chore branch via DQ #225; `.claude/PRPs/reports/refactor-tier-final-retro.md` on governance-v0) — plus the pre-existing cr-6 code commit `69f88ee8f` which was carried by this session's push but authored last session
**Commits:** 4 (chore-branch: `69f88ee8f` carried + `ba9177d46` DQ#225 ; governance-v0: `c4b09b65e` merge + `ae52dc0cd` FINAL retro)

## TL;DR

This session was the **tail of the PR-1 #132 CR-fix cycle** — the cr-6 sub-cycle (a CR re-review of its own cr-5 fix, which had shipped the audit-3.E.4 parity assert buried inside three `#[ignore]`d round-trip fns so it never ran). The session resumed post-compaction at "Gate 3 e2e in flight", confirmed the new non-ignored guard executed + passed (88→89, list matches disk), raised the re-validation-#4 DQ, then hit the most load-bearing surprise: **CodeRabbit had auto-paused reviews on the branch** ("influx of new commits") and would not re-review the cr-6 push unless explicitly triggered. The advisor correctly *detected and surfaced* this rather than silently skipping or silently triggering the CR gate; the user chose proceed-to-merge. PR #132 merged → strict-gate 5/5 → tier 4-role FINAL retro authored → loop stopped cleanly at the standing-instruction boundary (v1 PRD unblocked but NOT started). **Top change proposal: add a CR-auto-pause detection step to the CR-poll loop discipline so future sessions don't burn 4+ poll cycles interpreting `statusCheckRollup: None` as lag when it's actually a deliberate pause.**

---

## What surprised us

- **CodeRabbit auto-paused reviews on the branch, and the pause is indistinguishable from lag through the `gh pr view` status-check surface.** `statusCheckRollup` showed `CodeRabbit: None/None` across 4 consecutive polls (~40 min). The natural reading was "CR is slow on this large e2e diff" (and the prior session's CR review *had* taken ~45 min for incremental comments, reinforcing that hypothesis). It took inspecting the **CR summary issue-comment body** (which contained an explicit `## Reviews paused` notice + `@coderabbitai resume`/`review` commands) to discover the None state was a *deliberate auto-pause after the commit influx*, not in-progress work. The check-run API (`commits/<sha>/check-runs`) showed only 1 check-run ("Red-flag diff scan") — CR registers no status check-run at all on a paused branch, which is the same signature as "hasn't started yet."
- **The cr-5→cr-6 chain proved a CR-fix can itself be inert.** cr-5 added the parity helper; cr-6 caught that the helper was only called from `#[ignore]`d fns so it still never ran by default. This was empirically confirmed by the *prior* re-validation log showing those 3 fns as `ignored` and the suite count static at 88 — a nice case where the runtime evidence made the finding TRUE-by-observation (no compiler scratch-check needed, unlike cr-2).
- **`gh api` with a leading-slash endpoint got MSYS-path-rewritten on Git-Bash-for-Windows.** `gh api -X DELETE /repos/.../git/refs/heads/...` failed with `invalid API endpoint: "C:/Program Files/Git/repos/..."` — the shell rewrote the URL path as a filesystem path. Omitting the leading slash fixed it. Not in the standing Windows-traps list (which covers git-show slashed refs, /tmp, stdout encoding) — this is a *gh-api-specific* slash trap.
- **`gh pr merge` and `gh api -X DELETE` both returned empty stdout on success.** Verify-before-trust caught both (re-checked `gh pr view --json state` → MERGED; re-checked `git ls-remote` → empty). Expected per `pattern_verify_before_trusting_shell_output`, but worth noting both mutating gh commands were silent-on-success this session.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add a **CR-auto-pause detection step** to the CR-poll discipline (advisor-orchestrator.md §3.1 "bm-poll-cr" stage + the carried wakeup-prompt template): when `statusCheckRollup` CodeRabbit is `None` for >1 poll, **read the CR summary issue-comment body and grep for `Reviews paused` / `@coderabbitai resume`** before concluding "CR is lagging". If paused → surface the trigger-vs-proceed choice to the user immediately instead of polling. | Saves ~3-4 wasted poll cycles (~15-30 min wall + 4 cache-warm wakeups) per CR-heavy lane that trips the auto-pause threshold. Converts an ambiguous wait into an immediate decision point. | minor (one grep added to an existing poll step) | 1× this session; CR auto-pause will recur on any lane with ≥N fix-push commits — structural, not incidental |
| 2 | Add a **gh-api leading-slash trap** row to the Windows-traps lesson (`feedback_windows_bash_python_git_show_tmp_traps.md` or a sibling): `gh api` endpoints on Git-Bash-for-Windows must omit the leading `/` (MSYS rewrites `/repos/...` → `C:/Program Files/Git/repos/...`). Canonical form: `gh api repos/owner/repo/...` not `gh api /repos/...`. | Eliminates a 1-cycle self-correction on every `gh api` path call (branch-delete, ref ops) in future Windows advisor sessions. | minor (one lesson-table row) | 1× this session; recurs on every `gh api <path>` call with a leading slash on this platform — deterministic |
| 3 | The carried wakeup-prompt's "CR re-review COMPLETE" detection criteria listed `statusCheckRollup CodeRabbit conclusion SUCCESS/NEUTRAL, OR a NEW CR summary issue-comment created_at AFTER push`. **Neither fires under auto-pause** (CR posts no new check-run AND amends-in-place rather than creating a new comment — `updated_at` moved but `created_at` did not). Update the criteria to also treat "summary body contains `Reviews paused`" as a terminal "CR will not auto-review — escalate" state. | Closes the detection gap that made this session poll 4× before manually discovering the pause. Makes the wakeup-prompt self-sufficient for the auto-pause case. | minor (criteria-list edit in the prompt template / advisor-orchestrator.md) | folds into #1; same root cause |

## What to carry forward

- **Verify-before-trust on every mutating `gh` command.** `gh pr merge` and `gh api -X DELETE` both returned empty on success; both were re-verified (`gh pr view --json state` → MERGED; `git ls-remote` → empty) before proceeding. This is the discipline working exactly as `pattern_verify_before_trusting_shell_output` prescribes — keep it reflexive, never infer success from exit-code-0-and-empty-output alone.
- **Surface ambiguous external-system state to the user rather than guessing.** The CR auto-pause was a genuine fork (trigger `@coderabbitai review` — a visible PR comment, Manual per branch-manager.md — vs. proceed on local validation). The advisor surfaced it via AskUserQuestion with the full trade-off rather than either silently posting the trigger or silently skipping the gate. This is the correct handling of a Manual-class action behind an ambiguous signal.
- **Right standard of proof per finding.** cr-6 was bucketed fix-in-pr without a compiler scratch-check because its premise was *runtime-observable* (the prior re-validation log empirically showed the 3 fns `ignored` + count static at 88). cr-2 (prior session) required and failed a scratch-check because its premise was a trait-resolution *claim*. Carry the discrimination: observable-fact findings need observation; inference-claim findings need the compiler — don't over- or under-verify uniformly.
- **Honest stop-hook checkpoints mid-procedure; honest task-completion eval at the end.** IDs 327/328 were mid-procedure checkpoints (tagged not-task-complete); ID 329 was the genuine task-completion eval (score 0.66, clean-execution-no for the 3 self-corrected slips). No created_at forge, no hook edit, no raw SQL. Keep the checkpoint-vs-completion distinction explicit in the eval title + tags.
- **Stop the loop at the standing-instruction boundary, exactly.** The terminal action was the FINAL retro + surface + wait — no `/brehon-phase-transition`, no v1-PRD dispatch, no scheduled wakeup. The standing instruction was honoured to the letter (including the verbatim "v1 PRD planning is now UNBLOCKED but NOT started…" line).

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Gate-3 e2e poll (python log-slice, no full Read) | 8 | 0 | none | confirmed 89/0/5 + guard `ok` from a tail-slice; correctly did NOT Read the multi-MB log |
| DQ #225 next_id computation (span lane + canonical + archives + origin chore tip) | 5 | 0 | none | clean; MAX_ID=224→225 across all 3 live refs; no collision |
| CR re-poll loop (4 invocations) | 0 | ~20 | **high** | polled 4× interpreting `CodeRabbit: None` as lag; the 4th-cycle manual inspection of the summary body revealed the **auto-pause** — the wasted cycles are the direct evidence for "What to change" #1/#3 |
| `gh api` branch-delete (L16) | 2 | ~2 | medium | leading-slash MSYS rewrite cost one self-corrected retry → "What to change" #2 |
| `gh pr merge` + verify | 10 | 0 | low | empty stdout; verify-before-trust confirmed MERGED — discipline paid off |
| AskUserQuestion ×2 (CR gate, then merge confirm) | 3 | 0 | none | clean forks; user interrupted the first to redirect ("I meant proceed") — handled without re-asking |
| FINAL retro authoring | 25 | 0 | none | 4-role, references-not-duplicates the parallel-lane retro; per-task table + 4 CR-cycle lessons |
| memory_write_eval (IDs 327/328 checkpoints, 329 completion) | — | 0 | none | hook-required; honest 3-signal; no bypass |

## Complexity scores (heavy tasks only)

Per `feedback_retro_task_complexity_score.md`. This session had **no Junior impl-task** (the cr-6 code was authored last session; this session only carried its push + raised DQ + merged + wrote the retro). The one non-trivial advisor task:

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| PR-1 cr-6 tail: poll→DQ#225→CR-poll-loop→merge→tier-closeout→FINAL-retro | 2 | 4 | ~80 (excl. long-poll wait) | n/a (advisor session, not Junior worker — no watchdog) |

No task exceeded the >55min-runtime / >40min-silence / >8-files flags in the watchdog sense (this is an advisor session, not a Junior worker). The ~80 min wall is dominated by the CR-poll wait + the FINAL retro authoring, both expected.

## Decisions to revisit

- **CR auto-pause threshold interaction with multi-fix CR cycles.** Any lane that takes >N fix-push commits (cr-2..cr-6 + DQ pushes was clearly over the threshold) will trip CodeRabbit's `auto_pause_after_reviewed_commits`. Worth a clarify: should the CR-fix workflow batch fixes into fewer pushes, OR should the advisor proactively `@coderabbitai resume`/`review` (a Manual action) once per cycle after the last fix lands? Currently each fix pushed separately, which is what tripped the pause. Not urgent — folds into "What to change" #1 — but a structural question for the v1-PRD-era CR workflow.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

None of this session's three "What to change" items hit the 2×-this-session bar (each was 1× this session). However **#2 (gh-api leading-slash trap) is deterministic-on-platform** — it will recur on every `gh api <path>` call with a leading slash on Git-Bash-for-Windows, which is the "≥1 here AND structurally-certain-to-recur" case the threshold is meant to catch. **#1/#3 (CR auto-pause detection)** is structural (recurs on every CR-heavy lane), not incidental.

User may approve promotion (boxes UNCHECKED by default):

- [ ] #2 gh-api leading-slash trap: add a row to `.claude/lessons/feedback_windows_bash_python_git_show_tmp_traps.md` (deterministic platform recurrence — strong candidate)
- [ ] #1+#3 CR auto-pause detection: add a CR-paused-detection step to `.claude/rules/advisor-orchestrator.md` §3.1 (bm-poll-cr stage) AND the carried wakeup-prompt template; optionally a new `.claude/lessons/feedback_coderabbit_auto_pause_detection.md` (structural recurrence on CR-heavy lanes)
- [ ] (deferred — no action proposed) The CR-fix-batching vs proactive-resume clarify under "Decisions to revisit" — surface at v1-PRD CR-workflow design, not now

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase reliability section
omitted — this session did not invoke `/auto-phase` nor mutate any
`.claude/auto-state/*.json` (the leftover `v1-SL-c-2.json` is from a
prior session; Step 0.5 trigger did not fire). PMD eval skipped —
`PROJECT_MEMORY_DB` unset; the hook-required `memory_write_eval` (ID 329)
was written via the project-memory MCP tool separately._
