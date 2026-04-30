# [role:impl-task] sweep-2026-04-30 C2 — Harness CR findings (issue #85)

## 1. Dispatch line

`[role:impl-task] sweep-c2-harness-cr — see .claude/PRPs/briefs/sweep-2026-04-30-c2-harness-cr.md`

## 2. Scope

Address the 12 CR findings posted by CodeRabbit against the harness commit `66749bb71` from PR #81 (issue **#85**). The 12 findings target three sub-trees:

- `.claude/hooks/*` — 3 lifecycle hooks
- `.claude/channels/*` — Telegram + webhook channels
- `.claude/routines/*` — upstream-rebase routine

**The raw CR review** (per issue #85's "Reference" section) is at `.claude/cr-review-pr81-harness.txt` if that file is present at HEAD; if absent, fetch the CR comments from PR #81 via:

```
gh api repos/barrie-cork/lemmy/pulls/81/comments --jq '.[] | select(.path | startswith(".claude/hooks/") or startswith(".claude/channels/") or startswith(".claude/routines/"))' | head -200
```

For each of the 12 findings: either fix the issue (preferred) or post an explicit rebuttal in the file's commentary with rationale.

**Out of scope:**
- Do NOT expand to other harness files outside the `66749bb71` diff. Use `git show 66749bb71 --stat -- .claude/` to confirm the file list before editing.
- Do NOT modify any non-harness file.
- Do NOT touch any `.claude/commands/bm/*` files (issue #88, separate cluster C1).

**Boundaries:**
- Edit only files under `.claude/hooks/`, `.claude/channels/`, `.claude/routines/`.
- Single commit subject `chore(harness): address 12 CR findings on hooks/channels/routines (closes #85)`.

## 3. Required reading

- **`.claude/hooks/README.md`** (if present)
- **`.claude/hooks/check-cargo-pipe.sh`**, **`edit-readback-reminder.sh`**, **`inject-dq-state.sh`**, **`observation-capture.sh`**, **`pre-phase-audit.sh`**, **`prp-ralph-stop.sh`**, **`retro-check.sh`**, **`test-hooks.sh`**, **`validate-memory-search.sh`**, **`worktree-guard.sh`** (the hooks that exist at HEAD)
- **`.claude/channels/setup-telegram.md`** + **`.claude/channels/webhook/*`**
- **`.claude/routines/upstream-rebase.md`**
- **GitHub issue #85 body**: `gh issue view 85 --repo barrie-cork/lemmy --json body --jq .body`
- **`66749bb71` diff**: `git show 66749bb71 -- .claude/` (the source commit the CR is reviewing)

## 4. Constraints

**HARD FORBIDS:**
- `cargo *` of any kind on the worker. No compilation, no test runs, no clippy.
- Editing any file under `crates/**`, `migrations/**`, or `.github/workflows/**`.
- Running `bash` or `node` against any of the hooks to "test" them — do not execute the hooks. They are user-environment-specific (Telegram tokens, webhook URLs) and must not run in the worker.
- Modifying `66749bb71` itself or the active `.claude/commands/bm/*` (issue #88, separate cluster).

**Required behaviour:**
- Single commit subject: `chore(harness): address 12 CR findings on hooks/channels/routines (closes #85)`.
- Trailer: `Closes: barrie-cork/lemmy#85`.
- Push branch and exit. Junior daemon finalize-merges into `governance-v0`.
- DO NOT open a PR. DO NOT raise a validate-pending DQ entry — doc/script edits only, no cargo cycle.
- Inline `<!-- cr-N -->` comments at edited sites for audit traceability.

**File-locality:** `.claude/hooks/`, `.claude/channels/`, `.claude/routines/` are this cluster's exclusive territory in this batch — no conflict with C1, C3, C4.

**Mid-task DQ push:** ambiguities that need advisor judgment → `kind: "clarify"`, `from: "impl"` in `.claude/decision-queue.json`, push, continue.
