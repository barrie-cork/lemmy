---
name: Brehon phases ship via PR into governance-v0 (Phase 5 onwards)
description: Every Brehon phase from Phase 5 forward runs on its own phase branch and closes via PR into governance-v0 so CodeRabbit reviews the phase. Phases 1–4 were grandfathered onto direct governance-v0 commits.
type: feedback
originSessionId: 50cf5c6f-6b81-45b0-b3b4-45ac46b352d8
---
Every Brehon phase from **Phase 5 onwards** runs on its own branch (`phase-5`, `phase-5a`, `phase-6`, …) and closes via a PR into `governance-v0`. Ralph commits land on the phase branch, never directly on `governance-v0`.

**Why:** `.coderabbit.yaml` auto-reviews PRs whose base is `governance-v0` with phase-specific `path_instructions` (Phase 1 through Phase 6 all have dedicated rule blocks citing their ADRs). Committing straight to `governance-v0` silently skips that review — which is what Phases 1–4 did. From Phase 5 on, every phase closes with a CodeRabbit-reviewed PR so the review layer actually fires before code lands on the integration branch.

**How to apply:**

1. **Phase start (done during the phase-transition handoff):**
   - `git checkout governance-v0 && git pull`
   - `git checkout -b phase-<N>` (or `phase-<N>a` if the phase was split per the 10–12-task rule)
   - Push the empty branch so CI and the advisor can see it: `git push -u origin phase-<N>`

2. **Phase body:** ralph commits land on `phase-<N>`. The `decision-queue.json`, commit message, and completion report conventions are unchanged.

3. **Phase close:**
   - `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-<N>` — the `--repo` flag is mandatory because `gh` defaults to the upstream `LemmyNet/lemmy` on forks (see `feedback_gh_pr_fork_repo_flag.md`)
   - PR title: `Phase <N> — <one-line goal>` (e.g. `Phase 5 — reputation snapshot + sponsor liability`)
   - PR body: link to the completion report in `.claude/PRPs/reports/phase-<N>-*-report.md` and the implementation-plan section
   - CodeRabbit auto-reviews within a few minutes (non-draft, non-WIP title)

4. **Review response:**
   - Advisor reviews CodeRabbit's summary and inline findings
   - Material findings get follow-up commits on `phase-<N>` — the PR updates in place, CodeRabbit re-reviews
   - Stylistic or opinionated findings can be declined with a short comment
   - When clean, merge (the fork uses straight merge to preserve the task-per-commit history; no squash)

5. **After merge:** `git checkout governance-v0 && git pull` on both homeserver advisor sessions and the brehon-fork impl side so subsequent phases branch from the updated tip.

**Grandfathering:** Phases 1, 2a, 2b, 3, 4a, 4b all shipped direct to `governance-v0`. They are NOT retroactively PR'd — the mechanical risk of moving 9+ in-flight commits (Phase 4b case) outweighs the review value at that point. If CodeRabbit flags something in a later phase that should have been caught earlier, treat it as a bug and handle it in the current phase.

**Rate-limit escape hatch:** if CodeRabbit's ~4 PRs/hour limit kicks in during a burst (e.g. Phase 5a + 5b + a follow-up in the same day), `.coderabbit.yaml` line 10 instructions apply: flip `reviews.auto_review.enabled` to `false` in a one-line PR, merge the burst, flip it back on.

**Edge case — sub-phase splits:** if a phase is split (e.g. 5a and 5b per the 10–12-task rule), each sub-phase gets its own branch and its own PR. Sub-phase A merges into `governance-v0`, then sub-phase B branches fresh from the updated `governance-v0`.

**Do not:**
- Commit directly to `governance-v0` from Phase 5 onward
- Open a PR against `main` — `main` is reserved for upstream rebases
- Squash-merge the PR — task-per-commit history is load-bearing for the retro + bisect workflow
- Skip CodeRabbit because "the diff is small" — the Phase 4 DTO-only files are exactly the kind of thing it catches mistakes in
