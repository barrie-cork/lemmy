---
name: gh run watch --exit-status returns 0 in observed terminal scenarios on gh CLI 2.89.0
description: Empirical probes show gh run watch --exit-status returns exit 0 for completed-success, completed-failure, and in-progress→failure-watched-live runs despite docs saying otherwise. Always disambiguate via gh run view --json conclusion.
type: feedback
---

Empirical finding from v1-validate-agent Task 2 (2026-04-27): on gh CLI 2.89.0, `gh run watch <id> --exit-status` returns **exit 0** in all three observed terminal scenarios:

1. Run already completed with `conclusion: success` (probe target: 25017407689) — exit 0.
2. Run already completed with `conclusion: failure` (probe target: 25017407659) — exit 0.
3. Run in-progress at start, watched live until completion with `conclusion: failure` (probe target: 25017554049) — exit 0.

The flag's `--help` text says: "Exit with non-zero status if run fails." Observed behaviour contradicts that line.

## Why this matters

A polling agent that branches on `gh run watch --exit-status`'s exit code without disambiguation will classify every workflow run as success — false-green. v1-validate-agent's ci-watcher subagent depends on workflow-conclusion classification to write `validate-result` (pass) vs `validate-failed` (fail) DQ entries; getting this wrong means the advisor's polling loop reads pass for a failed cargo build and advances the §13-task pipeline against broken code.

## How to apply

Any subagent, script, or workflow that watches a GitHub Actions run via `gh run watch --exit-status`:

1. Run `gh run watch <id> --exit-status` and capture the exit code (advisory only).
2. **Always** follow with `gh run view <id> --json conclusion --jq '.conclusion'` and branch on the **conclusion string**, not the exit code.
3. Conclusion values to expect: `success | failure | cancelled | timed_out | action_required | neutral | skipped | stale`.
4. The exit code is informational; treat as advisory in case future gh CLI versions fix the bug.

```bash
gh run watch <id> --exit-status; status=$?
conclusion=$(gh run view <id> --json conclusion --jq '.conclusion')
case "$conclusion" in
  success) ;;  # pass path
  failure|cancelled|timed_out) ;;  # fail path
  *) ;;  # classifier-miss → catch-fire
esac
```

## Generalises to

Any version-pinned `gh` CLI behaviour. Documented behaviour and `--help` text are insufficient signals. Empirically validate on the gh CLI version the agent runs against before authoring classifier logic. The same caution applies to other CLIs whose `--exit-status`-style flags are documented but rarely tested under real terminal-state matrices.

## Symptom to recognise

A polling agent silently classifies a known-failed workflow run as pass, and downstream consumers (advisor stage-shape, retro reports) read green for a red run. The agent's logs show "exit 0" but the GitHub UI shows the run failed. If you see this divergence, the exit code is lying — switch to conclusion-string disambiguation.

## Where this came from

DQ #70 (impl-self-resolved 2026-04-27, v1-validate-agent Task 2). Probes captured in `.claude/PRPs/debug/v1-validate-agent-task2-gh-run-watch-probes.log` (gitignored as `*.log`). The plan's §4 watchpoint #1 mandated empirical validation precisely because public docs were silent on success/timeout/cancelled exit semantics; the empirical pass found the bug before ci-watcher.md shipped, so the subagent's body now treats `gh run view` conclusion as authoritative and the watch exit code as informational.
