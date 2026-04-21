# Upstream-rebase routine

A weekly cloud routine that monitors `LemmyNet/lemmy` for new commits and
opens a draft PR with a rebase impact analysis. Replaces the Monday-morning
manual check from CLAUDE.md.

## Why a routine

- Runs on Anthropic cloud infrastructure → laptop doesn't need to be on
- Persistent across machine restarts (unlike `/loop`)
- Posts a structured analysis instead of dumping a `git log` you read manually
- Costs subscription tokens, not your local time

Fits one of the explicit risks in
[`IMPLEMENTATION-PLAN-v0.md`](../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md)
§7.1 — "rebase weekly onto upstream/main." A routine is the smallest
mechanism that turns "weekly" from a discipline into an automatic event.

## Setup

Routines are created via `/schedule` in any Claude Code session, or via
[claude.ai/code/routines](https://claude.ai/code/routines). The CLI is the
quickest path:

```text
/schedule weekly Monday 09:00 — Brehon upstream-rebase impact analysis
```

When prompted, paste the prompt from the next section. Configure:

- **Repositories**: `barrie-cork/lemmy` (this fork)
- **Environment**: any environment with `git` installed (the default works)
- **Connectors**: GitHub (required to open a draft PR). Slack optional if
  you want a notification when the analysis is ready.
- **Schedule**: weekly, Mondays at 09:00 local. Cron equivalent: `0 9 * * 1`
- **Branch push**: enable "Allow unrestricted branch pushes" so the routine
  can push the analysis branch (`claude/upstream-rebase-YYYY-MM-DD`).

## The prompt

Paste this verbatim into `/schedule`. It is self-contained — routines run
without conversation history, so every constraint must be stated.

```text
You are auditing this Brehon fork against upstream LemmyNet/lemmy and
producing a rebase impact analysis.

## Steps

1. Fetch the upstream remote:
   git fetch https://github.com/LemmyNet/lemmy.git main:upstream-main

2. Find the pinned upstream commit from CLAUDE.md (`@ d1975776a` or
   whatever the current pin is — read CLAUDE.md to confirm).

3. List commits since the pin:
   git log --oneline upstream-main ^<pinned-sha>

4. If the count is ZERO, exit silently — do not open a PR. Post nothing.

5. If the count is non-zero, classify each commit:
   - **governance-touching** if it modifies any path under:
     - crates/db_schema/src/source/governance/
     - crates/db_views/governance_*/
     - crates/api/api/src/governance/
     - crates/api/api_crud/src/governance/
     - crates/api/api_common/src/governance.rs
     - crates/apub/objects/src/governance/
     - crates/apub/activities/src/governance/
     - crates/server/src/governance.rs
     - migrations/*governance*/
     - crates/api/api_utils/src/plugins.rs (Extism host)
     - crates/api/api_crud/src/private_message/ (PM hooks per pm-plugin-hooks-stable.md)
   - **schema-touching** if it modifies migrations/ or crates/db_schema/
     outside the governance subtree
   - **other** for everything else

6. For each governance-touching commit, dump:
   - SHA + subject
   - Full diff stat (`git show --stat <sha>`)
   - Whether it touches files the fork has modified — check with
     `git log --all --oneline -- <touched-paths>` filtered to fork commits

7. Open a draft PR on barrie-cork/lemmy:
   - Branch: `claude/upstream-rebase-<YYYY-MM-DD>`
   - Base: `governance-v0`
   - Title: `chore(rebase): upstream impact analysis <YYYY-MM-DD> — <N> commits, <M> governance-touching`
   - Body: a markdown summary with three sections:
     - "Summary" — counts by category, recommended action (rebase soon / can defer)
     - "Governance-touching commits" — the dumps from step 6, in chronological order
     - "Schema-touching commits" — SHA + subject for each
     - "AGPL-NOTICE update" — the new pin SHA + `git describe` output, ready to paste
   - Do NOT actually rebase. Do NOT push to governance-v0. The PR is
     analysis only.

8. If a Slack connector is available, post a one-line summary to
   #brehon-alerts (or whichever channel is connected): "Upstream impact
   <date>: <N> new commits, <M> governance-touching. Analysis: <PR-URL>"

## Constraints

- Never modify governance-v0 directly.
- Never run `cargo` — this is analysis only.
- Never push to or close existing PRs.
- If the GitHub connector is missing, skip step 7 and instead leave the
  analysis as a Markdown comment on the routine session — do not crash.
- If the AGPL-NOTICE pin matches the current upstream HEAD, exit silently
  (no work to do).
```

## Cost estimate

A typical week sees 5–30 upstream commits. The routine reads `git log`,
diff-stats each, and writes one PR — roughly 30k–80k tokens per run.
Weekly cadence is 4 runs/month. Budget ~250k tokens/month at sane bounds.

## Tuning

To run nightly instead of weekly, change the schedule to `0 9 * * *`. The
prompt is unchanged. Step 4 (silent-on-zero-commits) ensures nights with no
upstream activity are free.

To pause: visit [claude.ai/code/routines](https://claude.ai/code/routines),
toggle "Repeats" off. Re-enable when ready.

## Why not a GitHub Action?

A GH Action would also work and is technically cheaper (no Claude tokens).
A routine wins on three dimensions for *this specific use case*:

1. **Variable analysis depth** — the routine can decide whether a commit
   touching `lemmy_diesel_utils::pagination` matters to governance crates
   that import it. A GH Action would need that logic hardcoded.
2. **Natural-language summary** — the PR body reads like a hand-written
   note ("3 of these 12 commits touch the apub layer; one rewrites the
   inbox dispatcher in a way that breaks our `receive_remote_sanction_notice`
   handler"), not a template.
3. **Fits existing PR review flow** — the analysis lands as a draft PR
   that CodeRabbit auto-reviews per `.coderabbit.yaml`. You read it the
   same way you read any other PR.

A GH Action is the right answer if you want zero-LLM cost and accept a
simpler "here's a list of SHAs" output.

## See also

- [Routines docs](https://code.claude.com/docs/en/routines)
- [Implementation plan §7.1 — top risks](../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md)
- [`AGPL-NOTICE.md`](../../AGPL-NOTICE.md) — the pin gets bumped on every accepted rebase
- Companion plan: `~/.claude/plans/are-we-fully-utilising-majestic-planet.md` (gap "Routines")
