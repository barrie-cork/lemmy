---
description: Handover — write a resume brief for the next advisor session
argument-hint: <optional-slug, e.g. "pre-pc-restart" or "pr92-merged"> [--no-write]
disable-model-invocation: true
---

# /handover-advisor — write advisor resume brief

**Input**: $ARGUMENTS

The advisor session is closing (PC restart, DQ-batch closure,
phase-transition boundary, or explicit park). This command samples
advisor-relevant state, detects what's pending for the next advisor
session, writes a self-contained resume brief to
`.claude/PRPs/handovers/advisor-<ISO-date>-<slug>.md`, and emits a
bootstrap prompt the user pastes into a fresh session.

**Reads:** `.claude/rules/handover.md`, `.claude/rules/branch-manager.md`,
`.claude/rules/decision-queue.md`.

**Never writes:** `crates/**`, `migrations/**`, `tests/**`,
`.claude/PRPs/plans/*.plan.md`, `.claude/PRPs/reviews/**`. See
`.claude/rules/handover.md` §"File ownership".

---

## Phase 0 — Parse arguments

| Input | Slug | Mode |
|---|---|---|
| `pre-pc-restart` | `pre-pc-restart` | write |
| `pr92-merged --no-write` | `pr92-merged` | dry-run (print to conversation) |
| (empty) | auto: derived from HEAD short-SHA + timestamp | write |

If the arg contains `--no-write`, print the brief + bootstrap prompt
to the conversation and skip Phase 3.5 (file write) and Phase 4
(runlog append).

---

## Phase 1 — Attribution guard

```bash
git branch --show-current
```

**Decision tree:**

| Current branch | Action |
|---|---|
| `governance-v0` | Proceed. |
| `phase-*` or `plan/*` | STOP: "Advisor handover must be written from primary worktree on `governance-v0`. You are on `{branch}`. If you meant to write an impl handover, run `/handover-impl`. If you meant to park advisor state from a phase worktree, switch worktrees first." |
| Anything else (detached HEAD, `main`) | STOP: "Advisor handover expects `governance-v0`. Current branch `{branch}`. Clarify." |

---

## Phase 2 — Sample state

Run in this order (all read-only):

```bash
git fetch origin
git status --short
git rev-parse HEAD
git log origin/governance-v0..governance-v0 --oneline
git worktree list
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,baseRefName,mergeStateStatus,reviewDecision
```

Then read (files, not full bodies unless small):

- `.claude/decision-queue.json` — note entries with `answer: null`
  and note entries resolved since the previous handover file's timestamp
- `.claude/runlog/bm-runlog.md` — tail (last 10 entries; not full file)
- `.claude/runlog/advisor-relays/` — most recent 3 files (check
  whether each has a corresponding impl-side response in
  `.claude/runlog/impl-relays/` or a downstream commit)
- Active plan file, if any: `.claude/PRPs/plans/*.plan.md` —
  list paths, do not read bodies
- Most recent handover file under `.claude/PRPs/handovers/advisor-*.md`
  — filename + TL;DR only (to avoid repeating context)

Cap total read volume per `.claude/rules/no-cargo-output-paste.md` —
do not paste any log tail over 20 lines into the brief.

---

## Phase 3 — Detect what's pending

Build an inventory of advisor-relevant open items:

### 3.1 DQ pending

From `.claude/decision-queue.json`, list entries where
`answer == null`. Group by `from:` (impl, bm, planner). Note:

- Entries from impl with `blocking: true` — highest priority
- Entries from bm — usually topology/ADR concerns
- Entries raised by advisor itself (self-reminders)

### 3.2 PR advisor-gate

From `gh pr list` output, list PRs where **any** of:

- `mergeStateStatus != CLEAN`
- `reviewDecision == CHANGES_REQUESTED`
- findings YAML at `.claude/PRPs/reviews/pr-<N>-findings.yaml` has
  open `severity: critical` entries in `bucket: fix-in-pr`

### 3.3 Plan / PRD amendments in flight

```bash
git status --short -- '.claude/PRPs/plans/' '.claude/PRPs/prds/' 'docs/brehon-law-inspired-network/'
```

Any modified files here indicate uncommitted advisor work. List them
(path + one-line diff summary per `git diff --stat`).

### 3.4 Relay backlog

For each recent `advisor-relays/*.md`, check whether there is either
(a) an impl-side commit whose message references the relay id, or
(b) an `impl-relays/*.md` with `relates_to:` pointing at the relay.
If neither, the relay is pending impl response — list it.

---

## Phase 3.5 — Compose + write the brief

File path (if `--no-write` not set):

```
.claude/PRPs/handovers/advisor-<ISO-date>-<slug>.md
```

- `<ISO-date>` = `YYYY-MM-DD` from current UTC date
- `<slug>` = arg 1, or `HEAD-<short-SHA>` if arg 1 omitted

### Brief template

Fill each section from the Phase 2 + 3 samples. Omit a section if
truly empty (e.g. no PRs open → omit §"PR advisor-gate" header; write
"(none)" if the section structure needs preserving).

```markdown
# Advisor handover — <ISO-date> — <slug>

**Written:** <ISO UTC timestamp, minute precision>
**Author:** advisor session (C:\Users\barri\Developer\brehon-fork)
**Branch:** governance-v0 @ <short-SHA>
**Purpose:** Self-contained brief for next-session resume.

## TL;DR

- <one line: what happened in this session>
- <one line: what's pending (DQ/PR/relay counts)>
- <one line: what's blocked, if anything>
- <one line: what the next session should do first>
- <one line: any deadline or external gate (e.g. "CI e2e running on PR #N")>

## Cold-resume sequence

1. Read `CLAUDE.md` + `.claude/rules/*.md` (auto-loaded in `-p` mode).
2. Read this file in full.
3. Read `.claude/decision-queue.json` — note pending entries listed below.
4. Read `.claude/runlog/bm-runlog.md` tail — last 10 entries for state context.
5. Verify state:
   ```bash
   git fetch origin
   git rev-parse HEAD              # expect <short-SHA>
   git status --short              # expect <inventory from §State at handover>
   gh pr list --repo barrie-cork/lemmy --state open --json number,title,mergeStateStatus
   ```
6. **Verify untested assumptions BEFORE writing any patch.** For each numbered item under §"What's pending" that lists `ASSUMES: <property> — VERIFY: <check>`, run the named VERIFY check (≤5 min per assumption). If any check falsifies the assumption, surface to user via `AskUserQuestion` with the failed assumption + revised options — do NOT proceed with the prescribed "smallest change". Per `.claude/lessons/feedback_handover_assumptions_need_empirical_verification.md`.
7. Append cold-resume event to runlog:
   ```
   <ISO-UTC> | advisor | meta | cold-resume | handover=<file> drift=<none|<detail>>
   ```

## State at handover

### Git
- Branch: `governance-v0`
- HEAD: `<SHA>` (`<commit-subject>`)
- Working tree: <inventory from `git status --short` OR "clean">
- Unpushed: <N commits from `git log origin/..governance-v0` OR "clean">

### Worktrees
<List from `git worktree list` with purpose of each>

### Open PRs
<Table from `gh pr list`. Include columns: PR#, title, branch, mergeState, reviewDecision>

### Decision queue
- Pending requiring advisor: <ids with one-line framing>
- Pending requiring impl (impl-self-resolvable): <ids>
- Recently resolved (since <prev-handover-date>): <ids>

### Relays
- Advisor → impl awaiting impl response: <list of filenames>
- Impl → advisor awaiting advisor answer: <list of filenames>

### Plan / PRD / ADR in-flight
<List of uncommitted files under `.claude/PRPs/plans/`, `.claude/PRPs/prds/`, `docs/brehon-law-inspired-network/` with one-line diff summary>

## What's pending (ordered by priority)

<Numbered list. Each item:
 - **N. <action verb> <concrete target>**
   - Context: <one sentence>
   - File/command: <path or command>
   - Expected outcome: <one sentence>
   - Rollback if it fails: <one sentence>
   - **Untested assumptions** (env vars, CLI tools, remote dir shapes, hook events the prescription depends on but THIS session did NOT empirically verify): <list each as "ASSUMES: <property> — VERIFY: <one-line cheap check>"; OR "(none — all dependencies verified in this session)". Per `feedback_handover_assumptions_need_empirical_verification.md` — next session MUST run each VERIFY check BEFORE the patch.
>

## What NOT to touch

Per `.claude/rules/branch-manager.md` and `.claude/rules/handover.md`:

- `crates/**`, `migrations/**`, `tests/**` — impl-owned.
- Active plan/PRD files unless this session is explicitly editing them:
  <list any plan/PRD the next session MUST leave alone until impl finishes>.
- PR findings YAML at `.claude/PRPs/reviews/**` — BM-owned; read only.

## Bootstrap prompt (paste into next session)

```
I'm resuming advisor work on the Brehon governance fork. Previous session
wrote a handover at `.claude/PRPs/handovers/advisor-<ISO-date>-<slug>.md`.

First steps:
1. Read CLAUDE.md + .claude/rules/*.md (auto-loaded in -p mode).
2. Read `.claude/PRPs/handovers/advisor-<ISO-date>-<slug>.md` in full.
3. Execute its "Cold-resume sequence" in order.
4. Report the TL;DR summary plus any state drift you detected back to me.

Do not take any state-changing actions until I confirm.
```

## Closing state assertions (verify on resume)

- `git branch --show-current` → `governance-v0`
- `git rev-parse HEAD` → `<full-SHA>`
- `git rev-parse origin/governance-v0` → `<full-SHA>` (same if no unpushed)
- `git status --short` → <exact inventory>
- `.claude/decision-queue.json` pending count → <N>
- Active handover file exists: `test -f .claude/PRPs/handovers/advisor-<ISO-date>-<slug>.md`
```

Write this template to the file (or print if `--no-write`).

---

## Phase 4 — Append to runlog

(Skip if `--no-write`.)

Append one line to `.claude/runlog/bm-runlog.md`:

```
<ISO-UTC> | advisor | meta | handover-written | file=.claude/PRPs/handovers/advisor-<ISO-date>-<slug>.md branch=governance-v0 head=<short-SHA>
```

Per `.claude/rules/handover.md`, this is the single cross-role ledger
the advisor command writes to. Impl handovers append to their own
phase runlog, not this file.

---

## Phase 5 — Output

```markdown
## /handover-advisor complete

**Handover:** .claude/PRPs/handovers/advisor-<ISO-date>-<slug>.md (<N> lines)
**Branch:** governance-v0 @ <short-SHA>
**Pending inventory:**
- DQ requiring advisor: <N>
- Open PRs advisor-gate: <N>
- Relays awaiting response: <N>
- Plan/PRD in-flight files: <N>

**Runlog appended:** .claude/runlog/bm-runlog.md (1 new line)

### Bootstrap prompt (copy-paste into fresh Claude Code session)

```
I'm resuming advisor work on the Brehon governance fork. Previous session
wrote a handover at `.claude/PRPs/handovers/advisor-<ISO-date>-<slug>.md`.

First steps:
1. Read CLAUDE.md + .claude/rules/*.md (auto-loaded in -p mode).
2. Read `.claude/PRPs/handovers/advisor-<ISO-date>-<slug>.md` in full.
3. Execute its "Cold-resume sequence" in order.
4. Report the TL;DR summary plus any state drift you detected back to me.

Do not take any state-changing actions until I confirm.
```

### Hand-off

Close this session. The next advisor session picks up via the bootstrap
prompt above. No further action here.
```

If `--no-write`, replace the first line with:

```
## /handover-advisor complete (DRY RUN — no file written, no runlog append)
```

and print the full brief inline in place of the file-path reference.

---

## Refusal cases

- Current branch is `phase-*` or `plan/*` → STOP (Phase 1).
- Current branch is anything other than `governance-v0` → STOP (Phase 1).
- `.claude/PRPs/handovers/` directory missing → STOP and ask
  (never auto-create tracked top-level dirs).
- Slug contains `/` or whitespace → STOP and ask for a kebab-case slug.
- Target file already exists with same `<ISO-date>-<slug>` → STOP and
  ask whether to overwrite or add a counter suffix.

---

## See also

- `.claude/rules/handover.md` — shared invariants
- `.claude/rules/branch-manager.md` — file ownership + autonomy bounds
- `.claude/rules/decision-queue.md` — attribution rules
- `.claude/commands/handover/handover-impl.md` — impl-side flavor
- `.claude/PRPs/reports/v1-AD-c-advisor-resume-state.md` — canonical
  exemplar (204 lines) — mirror its section shape
