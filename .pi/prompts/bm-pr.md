---
description: |
  BM — open a PR from current phase/plan branch into governance-v0 (auto, no prompt)
argument-hint: |
  [--draft] (rare; CR skips drafts per phase-branch.md)
disable-model-invocation: true
---

# /bm-pr — open PR into governance-v0

**Input**: $ARGUMENTS

**Dispatcher → `branch-manager` subagent.** This command delegates
execution to the `branch-manager` subagent, which runs in its own
isolated context window. The subagent reads the operational script
below and follows it step by step.

Invoke:

> Use the `branch-manager` subagent to run `bm-pr`. Arguments:
> $ARGUMENTS. Follow the phases in `.claude/commands/bm/bm-pr.md` —
> verify pre-conditions, resolve title from plan file, assemble body
> from completion report + plan + commit log, open PR into
> governance-v0 via `gh pr create --repo barrie-cork/lemmy --base governance-v0`.
> Return a "PR opened" summary followed by a "Next suggested" line.

The parent (impl) session should call `Agent(subagent_type="branch-manager", model="sonnet", prompt=<the above>)`. Templated PR-body assembly + outbound PR-create (auto, no-ask) — Sonnet 4.6 balances cost with the prose-composition judgment. Not Haiku: body tone + ADR-constraint awareness matter for the PR description.

---

## Operational script (for the subagent)

The BM agent opens a PR from the current branch into
`governance-v0`. Auto, no prompt. Body assembled from completion
report (if any) + plan reference + commit log.

**Reads:** `.claude/rules/branch-manager.md`, `.claude/rules/phase-branch.md`,
`.claude/rules/gh-pr-fork-target.md`.

---

## Phase 1 — Verify pre-conditions

```bash
git branch --show-current
git status --short
git log governance-v0..HEAD --oneline
gh pr list --repo barrie-cork/lemmy --head $(git branch --show-current) --json number,state
```

| State | Action |
|---|---|
| On `governance-v0` or `main` | **STOP**: "Cannot open PR from trunk." |
| Branch not pushed (no `origin/<branch>`) | **STOP**: "Run `/bm-push` first." |
| Local ahead of remote | **STOP**: "Push pending — `/bm-push` first." |
| Working tree dirty | **STOP**: "Uncommitted changes — impl must commit first." |
| No commits ahead of `governance-v0` | **STOP**: "No commits to PR." |
| PR already exists for this branch | **EDIT** existing body via `gh pr edit` (skip to Phase 4) |

---

## Phase 2 — Resolve PR title

Branch-name → title pattern:

| Branch | Title shape |
|---|---|
| `phase-v<N>-<area>-<letter>` | `Phase v<N>-<area>-<letter> — <one-line goal>` |
| `plan/v<N>-<area>-<letter>` | `docs(plan): v<N>-<area>-<letter> — <one-line goal>` |
| `chore/<slug>` | `chore(<scope>): <slug-prose>` |

Resolve "one-line goal" by reading the plan file:

```bash
ls .claude/PRPs/plans/<phase-suffix>*.plan.md 2>/dev/null
# Read the H1 / first heading from the plan
```

If no plan file:

- **For phase implementation branches** (branch matches `^phase-v\d+-`)
  or **plan branches** (`^plan/`): STOP. A plan file is REQUIRED for
  phase/code work — `IMPLEMENTATION-PLAN-v0.md §2` mandates planning
  and implementation as separate phases. Tell the user to create the
  plan first under `.claude/PRPs/plans/<phase>*.plan.md` and re-run
  `/bm-pr`.
- **For chore/docs branches** (`^chore/`): fall back to the first
  commit's subject. Annotate the PR body with `Ad-hoc — no plan file
  (chore branch)` so reviewers see this branch was opened without a
  plan and that's intentional.

```bash
# Branch-class detection
branch="$(git rev-parse --abbrev-ref HEAD)"
case "$branch" in
  phase-v*|plan/*)
    if [ ! -f "$(ls .claude/PRPs/plans/${branch#phase-}*.plan.md 2>/dev/null | head -1)" ]; then
      echo "ERROR: phase/plan branches require a plan file in .claude/PRPs/plans/" >&2
      exit 1
    fi
    ;;
  chore/*)
    git log governance-v0..HEAD --oneline --reverse | head -1
    ;;
esac
```

<!-- cr-7 (closes #88): falling back to commit-subject for phase
     implementation branches let `/bm-pr` open unplanned implementation
     PRs labelled `Ad-hoc — no plan file`, violating
     IMPLEMENTATION-PLAN-v0.md §2. The fallback is now safe only for
     `chore/*` branches; phase/plan branches must STOP. -->


---

## Phase 3 — Assemble PR body

Source priority:

1. **Completion report** if present:
   `.claude/PRPs/reports/<phase-suffix>-complete-report.md` →
   include verbatim under `## Summary`.
2. **Plan file** if present:
   `.claude/PRPs/plans/<phase-suffix>*.plan.md` → reference path
   under `## Plan reference`.
3. **Commit log** always:
   `git log governance-v0..HEAD --oneline` → list under `## Commits`.

Body template:

```markdown
## Summary

{One-paragraph from completion report, OR derived from commit subjects}

## Plan reference

`.claude/PRPs/plans/{plan-file}` (or "Ad-hoc — no plan file")

## Completion report

`.claude/PRPs/reports/{report-file}` (or "Pending — to be added at merge")

## Commits

{git log governance-v0..HEAD --oneline output, one bullet per line}

## Closes

{From commit messages, extract `closes #N`, `fixes #N`, `relates #N`}

## Validation

Validation summary will be appended by `/bm-prp-review` once it runs.

---

*PR opened by branch-manager session. CodeRabbit review will follow
automatically (PR is not draft per `phase-branch.md`).*
```

---

## Phase 4 — Open PR (or edit existing)

### New PR

```bash
gh pr create \
  --repo barrie-cork/lemmy \
  --base governance-v0 \
  --head $(git branch --show-current) \
  --title "{title}" \
  --body "$(cat <<'EOF'
{body}
EOF
)"
```

`--draft` only if `$ARGUMENTS` contains it. Per `phase-branch.md`,
do NOT default to draft (CR skips drafts).

### Existing PR — edit body

```bash
gh pr edit {N} --repo barrie-cork/lemmy --body "$(cat <<'EOF'
{body}
EOF
)"
```

Do not change the title on edit unless explicitly told to.

---

## Phase 5 — Capture PR number + URL

```bash
gh pr view --repo barrie-cork/lemmy \
  --json number,url,title,state,baseRefName,headRefName
```

Store `pr_number` for downstream BM commands.

---

## Phase 6 — Append to runlog

```markdown
## bm: PR opened — {ISO timestamp}
- **PR:** #{N} — {title}
- **URL:** {url}
- **Base ← Head:** governance-v0 ← {branch}
- **Body source:** {completion-report | plan | commits-only}
- **Next:** wait ~5–10 min for CR; then `/bm-poll-cr {N}`
```

---

## Phase 7 — Output

```markdown
## /bm-pr complete

**PR #{N}:** {title}
**URL:** {url}
**Base ← Head:** governance-v0 ← {current-branch}
**Body source:** {completion-report | plan | commits-only}
**Draft?** No (CR-eligible)

### What happens next (automatic)

1. CodeRabbit will review within ~5–10 min.
2. Run `/bm-poll-cr {N}` to ingest findings.
3. Run `/bm-prp-review {N}` to add Brehon ADR + cargo review.
4. Triage with `/bm-triage {N}` → confirm before posting digest.
5. Merge with `/bm-merge {N}` → confirm before merging.

### Telegram ping

To send a "PR opened" ping to the channel, run `/bm-ping pr-ready` —
it will ASK before sending.
```

---

## Refusal cases

- On trunk → STOP.
- Working tree dirty → STOP.
- No commits to PR → STOP.
- Branch not pushed → STOP.
- Base resolved to `main` → **STOP**: "Cannot PR into main; trunk for
  v1 work is `governance-v0`."
- PR already exists AND user did not ask to edit → INFO, suggest
  `/bm-poll-cr {existing}`.

---

## See also

- `.claude/rules/branch-manager.md`
- `.claude/rules/phase-branch.md` — phase-branch + PR flow
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` rule
- `.claude/commands/bm/bm-poll-cr.md` — pick up CR findings
- `.claude/commands/bm/bm-prp-review.md` — Brehon ADR + cargo review
