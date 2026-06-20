# Session retro — 2026-06-20 — minimax-m3-review-wiring

**Harness:** claude-code
**Session window:** 2026-06-20 ~08:15 → ~17:40 UTC (intermittent; ~active 90 min)
**Branch at start:** `2a400a6ab` (`governance-v0`)
**Branch at end:** `38e0c1845` (`governance-v0`)
**Files touched:** 2 (`.github/workflows/governance-ai-review.yml` created + tuned; `.disabled` removed)
**Commits:** 2 explicit by me reaching origin (`38e0c1845`, plus the repoint that got swept into `e4ebac81e`); 0 auto

## TL;DR

User asked to "git pull to fix MiniMax CR code reviewing." The premise was false — there was nothing to pull and no MiniMax review wiring existed. The real gap: lemmy was the **one fleet repo** whose AI-review workflow (`governance-ai-review.yml.disabled`) never got migrated off the old Claude action, so governance PRs had only CodeRabbit after Copilot was disabled fleet-wide. Falsified the premise first (checked PR reviewers, grepped `.github/`, read the homeserver `ci-ai-workflows-inventory.md` playbook), then repointed the workflow to the canonical MiniMax M3 `curl` step copied verbatim from an already-migrated sibling repo, added the `MINIMAX_API_KEY` secret, and proved it end-to-end (M3 posted a real ADR review on a throwaway PR). Top carry-forward: **starting from the homeserver inventory doc instead of the conversation's stated premise** saved building the wrong thing — a "git pull" that would have been a no-op.

---

## What surprised us

- **The user's stated task was a false premise, but the underlying intent was real.** "git pull to fix minimax CR reviewing" — local `governance-v0` was already even with origin (0 behind), and no MiniMax review path existed to fix. Falsifying before acting (per `feedback_falsifiable_hypothesis_before_structural_fix.md`) turned a no-op into the actual fix.
- **A concurrent session's commit silently swept my staged workflow change into its own commit.** My `git commit` (heredoc) never ran; commit `e4ebac81e` (`docs(retro): m3-core-recording…` from another session) carried BOTH the retro file AND my staged `+governance-ai-review.yml` / `-…yml.disabled` under a misleading subject — then pushed it to origin. My change went live, correctly, but under the wrong commit message and bundled with unrelated work. Exactly the cross-session attribution collision `feedback_cross_session_commit_attribution_collision.md` warns about.
- **The auto-mode classifier blocked the workflow push as data-exfiltration** — correctly flagging that the workflow uploads private governance diffs to a third-party LLM (api.minimax.io). The *config* push later went through fine; only the push that first *armed* the pathway was gated. The boundary fired on intent (new external data recipient), not on file content.
- **Two successful runs had been silently hitting the skip path, not reviewing anything.** "Has it fired today?" → yes, green ×2 — but both skipped on the 28KB gate and posted skip-notices. "Green" ≠ "reviewed." Reading the step-level conclusions (not just the run conclusion) surfaced this. Classic `pattern_verify_before_trusting_shell_output` at the CI-run level.
- **The 28KB diff gate was ~2000× too small for M3.** M3 has a 1M-token context and a 64MB request-body cap; the gate was a GitHub-Models-8K-era artifact that threw away review on exactly the phase-size PRs that need it most.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When committing to canonical `brehon-fork` `governance-v0`, do the `git add → commit → verify HEAD subject` as ONE atomic burst and **verify `git log -1` shows MY subject before pushing** (per Hard refusal #6/#7 + `feedback_cross_session_commit_attribution_collision.md`). I skipped the post-commit verify and a concurrent session clobbered my commit. | No more silent absorption of my staged changes into a foreign commit with a wrong subject | minor | 1× this session + ≥1× prior (the lesson exists) → **meets threshold** |
| 2 | Add a `feedback_ci_run_green_can_mean_skipped.md` lesson: a workflow run's top-level `conclusion: success` can mean the real work step was `skipped` by a gate. Always read step-level conclusions (`gh run view --json jobs --jq '.jobs[].steps[]'`) when verifying an AI-review/gated workflow actually did its job. | Future "is X firing?" checks read steps, not just run status | minor | 1× this session; 0× prior as a named lesson → record, propose |
| 3 | Note in the workflow file (done) that `max_tokens: 2000` caps review depth on big diffs even though the gate now allows 400KB input — flag for bump if big-PR reviews read thin. | Avoids "why is the review so short on a huge PR" confusion later | done | 1× — recorded inline |

## What to carry forward

- **Falsify the stated premise before executing, especially on "just do X" infra asks.** Checked PR reviewers + grepped `.github/` + read the canonical inventory doc before touching anything. Turned a no-op "git pull" into the correct fix. Used once, decisively.
- **Start from the fleet playbook, not from scratch.** `homeserver/docs/ci-ai-workflows-inventory.md` had the exact migration recipe (endpoint, model, secret, thinking-disabled) and named lemmy as the one unmigrated repo. Copying the M3 `curl` step **verbatim from an already-migrated sibling** (`web-archive/pr-review.yml` pulled from origin, not the stale local copy) avoided inventing the API shape.
- **Prove the real path, not just "it's wired."** Direct API smoke-test → live `workflow_dispatch` → throwaway PR exercising the actual review path → re-test against the new gate on a real >28KB PR. Each step caught something the prior wouldn't have (the skip-path illusion would have survived "it's green").
- **Read the model docs for limits before tuning a guard.** The 1M-context / 64MB-body / recommended-params facts came from MiniMax docs + model card, not memory — which is what made the gate change defensible rather than a guess.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Premise falsification (grep `.github/`, read inventory) | 20 | 0 | high | Stopped a no-op "git pull"; found the real gap (unmigrated workflow) |
| Verbatim copy of M3 curl step from migrated sibling | 15 | 0 | low | `gh api` pull of merged remote `pr-review.yml`; avoided inventing payload shape. Local copy was stale (pre-migration) — 2 min lost reading it first |
| Direct M3 API smoke-test (curl + jq) | 5 | 0 | none | Confirmed thinking-disabled returns clean content, 0 `<think>`, before committing |
| `gh run view --json jobs` step-level read | 10 | 0 | high | Revealed "green" runs were skip-path, not review-path. Load-bearing |
| Throwaway PR #207 end-to-end test | 8 | 0 | none | Proved the actual M3 review path posts a real comment; cleaned up after |
| WebFetch/WebSearch M3 docs | 8 | 2 | low | Authoritative context/param numbers; one fetch returned "not specified" before the model card gave the real values |
| AskUserQuestion (push auth ×2, temperature) | 4 | 0 | none | Clean forks on outbound-data decision + off-spec temp choice |

## Complexity scores (heavy tasks only)

No task crossed the heavy-task thresholds (>55min runtime / >40min log silence / >8 files). The session touched 2 files across ~90 active minutes with no Junior dispatch. Single workflow-authoring task: `2/2/~90/n-a` — well inside envelope; no carry-forward signal.

## Decisions to revisit

- `max_tokens: 2000` on review output vs 400KB allowed input — review depth is now the binding constraint on large PRs, not input size. Revisit if a real phase-size governance PR gets a review that reads thin.
- The repoint commit lives inside `e4ebac81e` with a wrong (`docs(retro)`) subject. Not worth a history rewrite (it's pushed + merged-forward), but worth knowing if anyone `git log`s for the M3 wiring and can't find it. Recorded here as the durable pointer.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1 (atomic commit + verify-HEAD-subject before push on canonical checkout): the lesson `feedback_cross_session_commit_attribution_collision.md` already exists — this session is a fresh recurrence. Consider adding the "verify `git log -1` subject is MINE before push" step explicitly to that lesson's How-to-apply if not already there.
- [ ] Change #2 (`feedback_ci_run_green_can_mean_skipped.md`): new cross-harness lesson — run `conclusion: success` can hide a skipped work-step; verify step-level conclusions for gated/AI-review workflows.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
