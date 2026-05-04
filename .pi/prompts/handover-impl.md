---
description: |
  Handover — write a resume brief for the next impl session
argument-hint: |
  <optional-slug, e.g. "task5-boundary" or "pre-pc-restart"> [--no-write]
disable-model-invocation: true
---

# /handover-impl — write impl resume brief

**Input**: $ARGUMENTS

The impl session is closing (task-N → task-N+1 boundary, blocked on
advisor, PC restart, or explicit park). This command samples impl-
relevant state, detects what's pending for the next impl session,
writes a self-contained resume brief to
`.claude/PRPs/handovers/impl-<ISO-date>-<slug>.md`, and emits a
bootstrap prompt the user pastes into a fresh session.

**Reads:** `.claude/rules/handover.md`, `.claude/rules/phase-branch.md`,
`.claude/rules/cargo-output-capture.md`,
`.claude/rules/no-cargo-output-paste.md`,
`.claude/rules/pre-phase-harness-audit.md`.

**Never writes:** `crates/**`, `migrations/**`, `tests/**`,
`.claude/PRPs/plans/*.plan.md`, `.claude/decision-queue.json`,
`.claude/runlog/bm-runlog.md`. See `.claude/rules/handover.md`
§"File ownership".

---

## Phase 0 — Parse arguments

| Input | Slug | Mode |
|---|---|---|
| `task5-boundary` | `task5-boundary` | write |
| `pre-pc-restart --no-write` | `pre-pc-restart` | dry-run (print to conversation) |
| (empty) | auto: `task<N>` from HEAD commit subject, or `HEAD-<short-SHA>` | write |

If the arg contains `--no-write`, print the brief + bootstrap prompt
to the conversation and skip Phase 3.5 (file write) and Phase 4
(runlog append).

---

## Phase 1 — Attribution guard

```bash
git branch --show-current
git rev-parse HEAD
ls .claude/PRPs/plans/*.plan.md 2>/dev/null
```

**Decision tree:**

| Current branch | Plan file present on branch? | Action |
|---|---|---|
| `phase-*` | yes | Proceed. |
| `phase-*` | no | STOP: "Impl handover expects a committed plan on the phase branch. None found at `.claude/PRPs/plans/*.plan.md`. Clarify — is this branch pre-plan-commit?" |
| `governance-v0` | n/a | STOP: "Impl handover must be written from a phase worktree on `phase-*`. You are on `governance-v0`. If you meant to write an advisor handover, run `/handover-advisor`." |
| `plan/*` | n/a | STOP: "Plan branches do not run impl; no handover needed. If you meant to park plan work, that is advisor-side — run `/handover-advisor`." |
| Anything else | n/a | STOP: "Impl handover expects `phase-*`. Current branch `{branch}`. Clarify." |

Also verify: if `git status --short` shows files staged but not
committed, STOP and advise: "Do not write an impl handover mid-commit.
Commit or revert the staged changes first — ambiguity between committed
and staged state is the most common resume failure mode." (Per
`.claude/rules/handover.md` §"When to write".)

---

## Phase 2 — Sample state

Run in this order (all read-only):

```bash
git rev-parse HEAD
git status --short
git log governance-v0..HEAD --oneline
git log origin/$(git branch --show-current)..HEAD --oneline 2>/dev/null
git stash list
```

Then read (files, not full bodies unless small):

- Active plan file at `.claude/PRPs/plans/*.plan.md` — filename + Task
  header list only (e.g. `grep '^## Task ' <plan>`); do not re-read
  full body
- `.claude/runlog/<phase>-runlog.md` if it exists — tail (last 10
  entries)
- `.claude/runlog/impl-relays/` — most recent 3 files (check whether
  each has a corresponding `advisor-relays/*-answer.md` or
  resolving commit)
- `.claude/audit-*.log` and `.claude/build-*.log` — list filenames
  + last-modified timestamps; do NOT paste bodies (per
  `no-cargo-output-paste.md`)
- `.claude/PRPs/reviews/pr-<N>-findings.yaml` if the current branch
  has an open PR — note the findings count by bucket + severity

**Cap total read volume.** Do not paste any log tail over 20 lines
into the brief. Reference paths + exit codes; the next session can
read the logs themselves if needed.

---

## Phase 3 — Detect what's pending

Build an inventory of impl-relevant open items:

### 3.1 Current task number

Match the latest commit subject against plan Task headers:

```bash
git log -1 --pretty=%s                                       # e.g. "feat(v1-JM-a): task 8 — endorsement chain check"
grep -n '^## Task ' .claude/PRPs/plans/*.plan.md | head -20  # task header list
```

If the commit subject is `feat(*): task N — ...` or `fix(*): task N — ...`,
the next task is `N+1`. If the commit is a `chore(lint):` or
`chore(pr-review):`, walk back to find the last `feat/fix task-N` and
infer N from there.

If no match, note "Current task unclear — last commit `<subject>` — next
session must re-read plan to find current position."

### 3.2 Working tree

From `git status --short`:

- `M` (modified, unstaged) — work-in-progress; list paths + one-line
  `git diff --stat` per file
- `A`/`M` (staged) — should be empty per Phase 1 guard; if present,
  the guard should have STOPPED
- `??` (untracked) — list; flag any under `crates/`, `migrations/`,
  `tests/` as "uncommitted new files — likely task-N artifacts"
- `U` (unmerged) — STOP per Phase 1 if seen

### 3.3 Unpushed commits

From `git log origin/<branch>..HEAD`, list commits not yet pushed.
These are typically a BM concern (BM will push via `/bm-push`), but
the impl brief records them so the resuming session knows what's
already committed locally vs what the remote has.

### 3.4 Background jobs

List `.claude/build-*.log` and `.claude/audit-*.log` files modified
in the last 2 hours. For each, tail -5 and note the exit-code line
(if present). If any log shows no exit code and was modified recently,
flag as "possibly still running at session close."

### 3.5 Relay backlog

For each recent `impl-relays/*.md`, check whether:

- There is an `advisor-relays/<id>-answer.md` → resolved
- There is a commit referencing the relay id in its message → resolved
- Neither → still pending advisor response

List unresolved ones with their `blocking: true|false` frontmatter flag.

### 3.6 CR / PR findings

If an open PR exists on this branch, read
`.claude/PRPs/reviews/pr-<N>-findings.yaml` `counters` block. Note
counts in buckets `fix-in-pr`, `carry-forward`, `rebut`, and by
severity. Do NOT copy finding bodies — reference the YAML path + id.

---

## Phase 3.5 — Compose + write the brief

File path (if `--no-write` not set):

```
.claude/PRPs/handovers/impl-<ISO-date>-<slug>.md
```

- `<ISO-date>` = `YYYY-MM-DD` from current UTC date
- `<slug>` = arg 1, or `task<N>` inferred from §3.1, else `HEAD-<short-SHA>`

### Brief template

Fill each section from the Phase 2 + 3 samples. Omit a section if
truly empty (e.g. no unpushed commits → write "(clean)" under
§"Unpushed").

```markdown
# Impl handover — <ISO-date> — <slug>

**Written:** <ISO UTC timestamp, minute precision>
**Author:** impl session (<worktree-path from `git rev-parse --show-toplevel`>)
**Branch:** <phase-branch> @ <short-SHA>
**Plan:** <path to plan file>
**Purpose:** Self-contained brief for next-session resume.

## TL;DR

- <one line: what task just finished, or what's in flight>
- <one line: working-tree state + unpushed-count>
- <one line: what's blocking, if anything (relay pending, CR open)>
- <one line: next task number + one-line description from plan>
- <one line: any running/pending cargo job or external gate>

## Cold-resume sequence

1. Read `CLAUDE.md` + `.claude/rules/*.md` (auto-loaded in `-p` mode).
2. Read this file in full.
3. Read `.claude/PRPs/plans/<plan-file>` §"Task <next-N>" for the next task spec.
4. Read `.claude/runlog/<phase>-runlog.md` tail (if exists) — last 10 events.
5. Verify state:
   ```bash
   git branch --show-current        # expect <branch>
   git rev-parse HEAD               # expect <short-SHA>
   git status --short               # expect <inventory>
   git log governance-v0..HEAD --oneline | wc -l   # expect <commit-count>
   ```
6. If `.claude/runlog/<phase>-runlog.md` exists, append cold-resume event:
   ```
   <ISO-UTC> | i | t<N> | cold-resume | handover=<file> drift=<none|<detail>>
   ```

## State at handover

### Git
- Branch: `<phase-branch>` (tracking `origin/<phase-branch>`)
- HEAD: `<SHA>` (`<commit-subject>`)
- Commits ahead of `governance-v0`: <N> (one per task so far, per `phase-branch.md`)
- Unpushed to `origin/<branch>`: <N commits OR "clean">
- Working tree: <inventory from `git status --short` OR "clean">
- Stash: <N entries OR "clean">

### Plan position
- Active plan: `<path>`
- Tasks completed: <list of task numbers>
- Current task: <N> (`<subject line from plan>`) — <status: not-started | in-progress | done-but-not-committed>
- Next task: <N+1> (`<subject line from plan>`)

### Relays
- Impl → advisor awaiting response: <list with `blocking: true|false`>
- Advisor → impl not yet applied: <list>

### Background jobs
- `<log-path>` — modified <time>, exit <code|"(no exit line found — may be running)">
  (one entry per recent `.claude/build-*.log` / `.claude/audit-*.log`)

### PR / CR (if open PR on this branch)
- PR <N>: <URL>, mergeState <status>
- Findings YAML: `.claude/PRPs/reviews/pr-<N>-findings.yaml`
- Open `fix-in-pr` critical: <N>
- Open `fix-in-pr` major: <N>
- Open `carry-forward`: <N>

## What's pending (ordered)

<Numbered list. Each item:
 - **N. <action verb> <concrete target>**
   - Context: <one sentence>
   - File:line or command: <ref>
   - Expected outcome: <one sentence>
   - Rollback if it fails: <one sentence>
>

For the next task, cite the plan §"Task <N>" verbatim rather than
paraphrasing — the plan is the source of truth.

## What NOT to touch

Per `.claude/rules/handover.md` and `.claude/rules/branch-manager.md`:

- `.claude/PRPs/plans/*.plan.md` — advisor-owned; do not amend without
  a separate `docs(plan):` commit coordinated via DQ.
- `.claude/decision-queue.json` — answers are advisor-written; impl may
  only self-resolve via `answered_by: "impl-self-resolved"` per
  `decision-queue.md:77-98`.
- `.claude/runlog/bm-runlog.md` — BM-owned; impl appends to phase runlog
  `.claude/runlog/<phase>-runlog.md` if one exists.
- `.claude/PRPs/reviews/**` — BM-owned.
- `crates/`, `migrations/`, `tests/` paths OUTSIDE the scope of the
  next task (cross-reference the plan's Task <N> files list).

## Bootstrap prompt (paste into next session)

```
I'm resuming impl work on the Brehon governance fork. Previous session
wrote a handover at `.claude/PRPs/handovers/impl-<ISO-date>-<slug>.md`.

First steps:
1. Read CLAUDE.md + .claude/rules/*.md (auto-loaded in -p mode).
2. Read `.claude/PRPs/handovers/impl-<ISO-date>-<slug>.md` in full.
3. Execute its "Cold-resume sequence" in order.
4. Report the TL;DR summary plus any state drift you detected back to me.

Do not take any state-changing actions until I confirm.
```

## Closing state assertions (verify on resume)

- `git branch --show-current` → `<phase-branch>`
- `git rev-parse HEAD` → `<full-SHA>`
- `git rev-parse origin/<phase-branch>` → `<full-SHA>` (or different if unpushed)
- `git status --short` → <exact inventory>
- `git log governance-v0..HEAD --oneline | wc -l` → <commit-count>
- Active handover file exists: `test -f .claude/PRPs/handovers/impl-<ISO-date>-<slug>.md`
- Plan file present: `test -f <plan-path>`
```

Write this template to the file (or print if `--no-write`).

---

## Phase 4 — Append to runlog (conditional)

(Skip if `--no-write`.)

```bash
test -f .claude/runlog/<phase>-runlog.md && echo "exists" || echo "absent"
```

- If the phase runlog file exists, append one line:
  ```
  <ISO-UTC> | i | meta | handover-written | file=.claude/PRPs/handovers/impl-<ISO-date>-<slug>.md head=<short-SHA>
  ```
- If the phase runlog file does NOT exist, **skip the append**. The
  handover file itself IS the ledger entry. Do not auto-create
  `<phase>-runlog.md` — file creation is a separate intentional action
  (impl opens the runlog at task 0 of a new phase per precedent, not
  mid-phase as a side effect of handover).

**Never** write to `.claude/runlog/bm-runlog.md` from an impl worktree.
That file lives on `governance-v0` and cross-branch writes break
phase-branch discipline (`.claude/rules/phase-branch.md`).

---

## Phase 5 — Output

```markdown
## /handover-impl complete

**Handover:** .claude/PRPs/handovers/impl-<ISO-date>-<slug>.md (<N> lines)
**Branch:** <phase-branch> @ <short-SHA>
**Plan:** <path>
**Task position:** completed <last-N>, next <N+1>
**Pending inventory:**
- Working tree modified: <N files>
- Unpushed commits: <N>
- Relays awaiting advisor: <N>
- Background jobs: <N> (<count-running> possibly running)
- Open CR fix-in-pr: <N critical / N major>

**Runlog appended:** <phase-runlog-path if exists, else "skipped (no phase runlog)">

### Bootstrap prompt (copy-paste into fresh Claude Code session)

```
I'm resuming impl work on the Brehon governance fork. Previous session
wrote a handover at `.claude/PRPs/handovers/impl-<ISO-date>-<slug>.md`.

First steps:
1. Read CLAUDE.md + .claude/rules/*.md (auto-loaded in -p mode).
2. Read `.claude/PRPs/handovers/impl-<ISO-date>-<slug>.md` in full.
3. Execute its "Cold-resume sequence" in order.
4. Report the TL;DR summary plus any state drift you detected back to me.

Do not take any state-changing actions until I confirm.
```

### Hand-off

Close this session. The next impl session picks up via the bootstrap
prompt above. Consider whether to `/bm-push` from the BM worktree
first if there are unpushed commits on this branch.
```

If `--no-write`, replace the first line with:

```
## /handover-impl complete (DRY RUN — no file written, no runlog append)
```

and print the full brief inline in place of the file-path reference.

---

## Refusal cases

- Current branch is `governance-v0`, `plan/*`, or `main` → STOP (Phase 1).
- Current branch `phase-*` but no plan file present → STOP (Phase 1).
- Working tree has staged-but-uncommitted files → STOP (Phase 1).
- `.claude/PRPs/handovers/` directory missing → STOP and ask.
- Slug contains `/` or whitespace → STOP and ask for a kebab-case slug.
- Target file already exists with same `<ISO-date>-<slug>` → STOP and
  ask whether to overwrite or add a counter suffix.

---

## See also

- `.claude/rules/handover.md` — shared invariants
- `.claude/rules/phase-branch.md` — branch topology (the guard in
  Phase 1 enforces this)
- `.claude/rules/decision-queue.md` — attribution rules (impl never
  self-attributes as advisor)
- `.claude/rules/no-cargo-output-paste.md` — why logs are referenced,
  not pasted
- `.claude/commands/handover/handover-advisor.md` — advisor-side flavor
- `.claude/PRPs/reports/phase-5a-handover-task-54-onward.md` — canonical
  impl exemplar — mirror its section shape (§1 git state, §2 done/left,
  §3 plan deviations, §4 DQ state, §5-N task specs)
