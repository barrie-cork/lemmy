# [role:impl-task] sweep-2026-04-30 C1 — BM tooling CR findings (issue #88)

## 1. Dispatch line

`[role:impl-task] sweep-c1-bm-tooling — see .claude/PRPs/briefs/sweep-2026-04-30-c1-bm-tooling.md`

## 2. Scope

Address the 11 CR findings on `.claude/commands/bm/*.md` + `.claude/rules/branch-manager.md` from issue **#88** ([CR review on PR #87](https://github.com/barrie-cork/lemmy/pull/87)).

The 11 findings are doc/spec-only edits to BM tooling — no Rust code changes. Specifically:

- **cr-1** — `.claude/commands/bm/bm-merge.md:36` — Major
- **cr-2** — `.claude/commands/bm/bm-merge.md:82` — Major (schema-invalid `fix-in-pr` rows)
- **cr-3** — `.claude/commands/bm/bm-merge.md:113` — Major (do not allow merges with checks pending)
- **cr-4** — `.claude/commands/bm/bm-poll-cr.md:92` — Major (normalise timestamp before sequencing `cr-*` IDs)
- **cr-5** — `.claude/commands/bm/bm-poll-cr.md:176` — Major (do not key walkthrough findings solely by `cr_url`)
- **cr-6** — `.claude/commands/bm/bm-poll-cr.md:216` — Major
- **cr-7** — `.claude/commands/bm/bm-pr.md:*` — Major
- **cr-8** — `.claude/commands/bm/bm-cut.md:*` — Major
- **cr-9** — `.claude/commands/bm/bm-triage.md:*` — Major
- **cr-10** — `.claude/rules/branch-manager.md:*` — Low / Nit
- **cr-11** — `.claude/rules/branch-manager.md:*` — Low / Nit

Read the GitHub issue body via `gh issue view 88 --repo barrie-cork/lemmy --json body --jq .body` for the full CR thread URLs. Each finding's PR-discussion URL has the precise CR text and exact file/line.

**Out of scope:**
- Do NOT modify Rust code. Do NOT touch `crates/**`.
- Do NOT touch v1-AD-d feature code (PR #87's main diff).
- Do NOT regenerate any `.claude/PRPs/reviews/*.yaml` schemas.

**Boundaries:**
- Edit only `.claude/commands/bm/*.md` and `.claude/rules/branch-manager.md`.
- Each finding either fixes the issue (preferred) or is rebutted with explicit rationale committed in the same file.
- Group all 11 fixes into a single commit subject `chore(bm): address CR findings cr-1..cr-11 (closes #88)`.

## 3. Required reading

- **`.claude/rules/branch-manager.md`** (the rule the BM verbs implement)
- **`.claude/commands/bm/bm-merge.md`**, **`bm-poll-cr.md`**, **`bm-pr.md`**, **`bm-cut.md`**, **`bm-triage.md`**
- **GitHub issue body**: `gh issue view 88 --repo barrie-cork/lemmy --json body,title --jq '"\(.title)\n\n\(.body)"'`
- **PR #87 CR threads**: each cr-N URL inside the issue body resolves to the exact comment with full context. Read every URL before editing the corresponding line.
- **`.claude/lessons/feedback_branch_manager_pm_split.md`** (if present)

## 4. Constraints

**HARD FORBIDS — all the following will fail the task:**
- `cargo *` of any kind on the worker (no `cargo check`, no `cargo build`, no `cargo test`, no `cargo clippy`). The Junior daemon's host (EliteDesk) is forbidden from running cargo or any compilation.
- Editing any file under `crates/**` or `migrations/**`.
- Editing `.github/workflows/**`.
- Running e2e tests anywhere.

**Required behaviour:**
- Single commit at end with subject `chore(bm): address CR findings cr-1..cr-11 (closes #88)`.
- Trailer: `Closes: barrie-cork/lemmy#88`.
- Push branch and exit (worker pre-push pattern). Junior daemon's finalize agent will merge the worktree branch into `governance-v0`.
- DO NOT open a PR. DO NOT raise any decision-queue entry — this is a doc-only edit, no validation cycle needed.

**File-locality (this is the only cluster touching `.claude/commands/bm/*.md` and `.claude/rules/branch-manager.md`):** No conflict expected with other clusters in this batch.

**Cite findings inline:** every `<!-- cr-N -->` HTML comment placed at the edited site lets future audit trace the change back to its source thread. Use this convention.

**Mid-task DQ push:** if you find an ambiguity that requires advisor judgment, write a `kind: "clarify"`, `from: "impl"` entry in `.claude/decision-queue.json` and push immediately, do NOT block on it. Tell the user in your final comment that DQ #N is pending.
