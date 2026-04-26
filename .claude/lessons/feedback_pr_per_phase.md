---
name: Brehon phases ship via PR into governance-v0 (Phase 5 onwards, code only)
description: Brehon code deliveries from Phase 5 forward run on a phase branch and close via PR into governance-v0 for CodeRabbit review. Meta-work (.claude/, docs/, scripts/, infra) commits direct to governance-v0. Phases 1–4 grandfathered.
type: feedback
originSessionId: 50cf5c6f-6b81-45b0-b3b4-45ac46b352d8
---
Every Brehon **code delivery** from **Phase 5 onwards** runs on its own branch (`phase-5`, `phase-5a`, `phase-6`, sub-phase ids like `v1-JM-c`, …) and closes via a PR into `governance-v0`. Code commits land on the phase branch, never directly on `governance-v0`. **Meta-work** (`.claude/` lessons/agents/rules/commands/briefs, `docs/` retros/notes/research, `scripts/` tooling, `.gitignore`/`.mcp.json.example`/sync manifests) commits direct on `governance-v0` with no PR — see `.claude/rules/phase-branch.md` for the file-set litmus test.

**Why:** `.coderabbit.yaml` auto-reviews PRs whose base is `governance-v0` with phase-specific `path_instructions` (Phase 1 through Phase 6 all have dedicated rule blocks citing their ADRs). For code under `crates/**`, `migrations/**`, `tests/**`, that review is net-signal — CR catches Rust idiom drift, SQL footguns, missing tests. For meta-work (advisor-prompt prose, lesson markdown, hook scripts), CR review is net-noise. The fork is private (no AGPL §13 visibility trigger pre-pilot), so there's no external-audit reason to PR meta-work; the PR flow exists purely for CR review value, and that value is concentrated in code.

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
- Commit code (`crates/`, `migrations/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.coderabbit.yaml`) directly to `governance-v0` from Phase 5 onward
- Open a PR against `main` — `main` is reserved for upstream rebases
- Squash-merge the code PR — task-per-commit history is load-bearing for the retro + bisect workflow
- Skip CodeRabbit on a code PR because "the diff is small" — the Phase 4 DTO-only files are exactly the kind of thing it catches mistakes in
- Open a PR for pure meta-work (`.claude/`, `docs/`, `scripts/`, infra) — direct on `governance-v0` per `phase-branch.md`'s file-set policy. PR overhead with no CR signal is anti-discipline.

**Mixed diffs:** if a single commit/branch touches both code and meta-work, the code half wins — go through PR flow. CR will only inline-comment on code paths anyway.
