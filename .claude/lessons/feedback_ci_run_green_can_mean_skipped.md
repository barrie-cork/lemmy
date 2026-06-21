---
name: CI run green can mean the work step was skipped
description: A GitHub Actions run with conclusion=success can mean the load-bearing work step was SKIPPED by a gate (size cap, if-condition, path filter), not that it did its job. When verifying that an AI-review / gated workflow actually reviewed, read step-level conclusions, not just the run-level conclusion.
type: feedback
---

A GitHub Actions workflow run reporting `conclusion: success` does NOT mean the load-bearing step ran. A step gated by an `if:` condition (diff-size cap, `should_review` output, path filter, skip flag) that evaluates false is marked `skipped` — and **a run where the real step skipped still rolls up to `success`**. "Green" answers "did the run finish without error," not "did the workflow do its job."

**Why:** 2026-06-20 (minimax-m3-review-wiring session) — the `governance-ai-review` workflow had run twice that day and both showed `completed/success`. Asked "has MiniMax M3 review been firing today?", the run-level status said yes. But reading the step-level conclusions (`gh run view <id> --json jobs --jq '.jobs[].steps[]'`) showed the truth: `Assemble rubric`, `Review with MiniMax M3`, and `Post review comment` were all `skipped` — the diffs exceeded the 28KB gate and the runs posted a *skip notice* instead of a review. The model had reviewed nothing. The run-level green hid it completely; only the per-step read surfaced it. Same false-green class as `feedback_pipes_mask_exit_codes.md` (there: pipe hides the build's exit code; here: roll-up hides the work-step's skip).

**How to apply:**

- When verifying that a gated or AI-review workflow actually *did the thing* (reviewed, deployed, posted, validated) — not just that it finished — read the step-level conclusions:
  ```bash
  gh run view <run-id> --repo <owner/repo> --json jobs \
    --jq '.jobs[].steps[] | "\(.conclusion)\t\(.name)"'
  ```
  Look for the load-bearing step's conclusion being `success`, not `skipped`. A `skipped` work-step under a green run is the signal that a gate (size cap, `if:`, path filter) short-circuited the real work.
- For AI-review workflows specifically, confirm the outcome at the *destination*, not just the run: did a review comment actually land on the PR? `gh api repos/<o>/<r>/issues/<pr>/comments --jq '.[] | select(.body | test("<your review marker>"))'`. A run can be green, the review step green, and the comment still absent if a later post step had an `if:` you didn't expect.
- Generalises: any workflow with conditional steps (`if: steps.X.outputs.Y == '...'`) can report run-success while the conditional work skipped. Treat run-level `conclusion: success` as "no error," never as "work done." The work-done signal is the specific step's conclusion plus the observable side-effect.

## See also

- `.claude/lessons/feedback_pipes_mask_exit_codes.md` — sibling false-green: a pipe reports tail/head/grep's exit code, not the build's. Same lesson at the shell level; this one is the GitHub-Actions-roll-up level.
- `pattern_verify_before_trusting_shell_output` (PMD) — exit codes, pipes, wrappers, and now CI run-status all lie; verify the actual side-effect.
- `.claude/PRPs/reports/session-retro-2026-06-20-minimax-m3-review-wiring.md` — the incident in full retro context.
