You are a Brehon advisor session about to be compacted, replaced, or swapped. Run this checkpoint so a resumed or new session can pick up without rebuilding context from scratch.

> **Modernized 2026-05-16:** rewritten from the legacy numbered-phase scheme (`advisor-context-phase-<N>.md`, `project_brehon_phase_<N>_notes.md`) to the current sub-phase / multi-lane-worktree scheme. The active lane is now derived from GIT (this session's CWD worktree branch), not from a context file that lags ship state. Per `.claude/rules/multi-lane-worktree.md` + `.claude/rules/advisor-orchestrator.md` §1.

This command is read-only against git. It writes one block to the active lane's phase-notes memory file (which is *not* version-controlled). It does **not** commit anything, push anywhere, or touch any working tree.

## Procedure

### 1. Detect the active phase/lane (from git, not a context file)

The active lane is whatever the **CWD worktree's branch** says it is — multiple concurrent lane worktrees are NORMAL (per `.claude/rules/multi-lane-worktree.md`), not an error condition. Do not stop-and-error on "multiple active files".

Run, using this session's actual CWD (it may be a `brehon-fork-<lane>` worktree, not always `C:\Users\barri\Developer\brehon-fork`):

- `pwd` — the worktree this session is in
- `git -C <cwd> rev-parse --abbrev-ref HEAD` — current branch
- `git -C <cwd> worktree list` — all active worktrees (context only)

Resolve the lane:

- **On a `phase-v1-<lane>` branch** (lane-dedicated worktree, e.g. `C:/Users/barri/Developer/brehon-fork-ship-1` on `phase-v1-ship-1`) → the lane is `<lane>` (e.g. `v1-ship-1`).
- **On `governance-v0` in the canonical `brehon-fork` checkout** → this is the meta/canonical session. The "active lane" for checkpoint purposes is whatever phase work this session was doing — derive it from the task list, recent commits (`git -C <cwd> log --oneline -10`), and the most-recent `.claude/PRPs/handovers/*-bootstrap.md`.

Only stop and tell the user if git state is **genuinely unreadable**: a detached HEAD with no resolvable branch AND no task-list / handover signal to infer the lane from. A partial-but-resolvable read is not a stop condition — checkpoint the lane this session's CWD is on.

### 2. Capture git state

Run these against this session's actual CWD (`<cwd>` from step 1 — may be a `brehon-fork-<lane>` worktree):

- `git -C <cwd> rev-parse --abbrev-ref HEAD` — current branch
- `git -C <cwd> rev-parse --short HEAD` — HEAD hash (short)
- `git -C <cwd> log --oneline -5` — recent commits on the current branch
- `git -C <cwd> rev-parse --short governance-v0` — `governance-v0` tip for drift comparison
- **When on/for a lane:** `git -C <cwd> rev-parse --short origin/phase-v1-<lane>` — the lane's phase-branch tip, so drift across BOTH refs is visible

If the repo is in a detached-HEAD state or a branch read fails, note it literally — don't fabricate a branch name.

### 3. Read the decision-queue

Read `<cwd>/.claude/decision-queue.json` (session-CWD-relative — per `.claude/rules/multi-lane-worktree.md` each worktree has its own DQ file; do **not** read the canonical checkout's copy for a lane session). Extract entries in the `pending` array (v2 schema uses `pending`/`resolved` arrays, not a per-entry `status` field). If the file is missing, record `(no decision-queue file found)`. If `pending` is empty, record `(empty)`.

### 4. Skim the lane's current state signal (best-effort)

Identify the most-recent of these two for the active lane and read its tail (~40 lines) for current impl/BM state (last task attempted, last error, last commit, what's queued next):

- `<cwd>/.claude/PRPs/handovers/<phase>-bootstrap.md` — lane handoff/bootstrap (if one exists for this lane)
- `<cwd>/.claude/runlog/<lane>-runlog.md` — the lane runlog (e.g. `.claude/runlog/v1-ship-1-runlog.md`)

If neither exists, skip this step silently (best-effort discipline — a partial checkpoint is more useful than none).

### 5. Reflect on your own state

Before writing the block, state to yourself (and the user in a short message) your answers to the four fields below. Don't pad — each is explicitly bounded:

- **Current focus** — one sentence.
- **Last decision and reasoning** — two sentences. What did you most recently decide, and why? If the session made no decisions, say `(no decisions this session)`.
- **Next expected artifact** — one sentence. What is the next thing you expect to review (a PR, a plan DoD, a completion report)?
- **Open tripwires** — bullet list, max 5 items. Phase-specific edge cases the next session should stop-and-ask about if they hit them.

### 6. Append the Session handoff block

Append (do not overwrite) to:

```
C:\Users\barri\.claude\projects\C--Users-barri-Developer-brehon-fork\memory\project_brehon_<sub-phase-slug>_notes.md
```

`<sub-phase-slug>` is the lane lowercased with `-`→`_` (e.g. lane `v1-ship-1` → `project_brehon_v1_ship_1_notes.md`; lane `v1-SL-e` → `project_brehon_v1_sl_e_notes.md`). This is the brehon-fork PMD dir (the advisor session CWD is the brehon-fork repo; per `brehon-fork/CLAUDE.md` `pmd_index`). If the file does **not** exist yet, CREATE it — this command may legitimately be the first writer for a new lane (matches the `project_brehon_<next>_notes.md` create-skeleton convention in `homeserver/.claude/advisor-context-<phase>.md` §8).

Use this exact structure, with today's UTC date/time and the values captured above:

```markdown

## Session handoff — <YYYY-MM-DD HH:MM UTC>

**Current focus:** <one sentence from step 5>

**Last decision and reasoning:** <two sentences from step 5>

**Next expected artifact:** <one sentence from step 5>

**Open tripwires:**
- <bullet 1>
- <bullet 2>
- ...

**Decision-queue pending at checkpoint:**
- <id / title> — <one-line summary>
- ...
(or `(empty)` / `(no decision-queue file found)`)

**Lane state (best-effort):** <one line from step 4, or `(no handover/runlog for this lane)`>

```git-state
worktree: <cwd from step 1>
branch: <current branch from step 2>
HEAD: <short hash>
governance-v0 tip: <short hash>
phase-v1-<lane> tip: <short hash, or n/a if on governance-v0 canonical>
recent commits:
<five-line git log --oneline output>
```
```

The block starts with a blank line (so it's visually separated from the prior content) and the header is an `## H2`. The triple-backtick `git-state` fence keeps the hashes copy-pasteable and now includes BOTH the `governance-v0` tip and the lane phase-branch tip.

If you created the notes file (first writer for this lane), also check `C:\Users\barri\.claude\projects\C--Users-barri-Developer-brehon-fork\memory\MEMORY.md` — if the new notes file is not yet indexed under the "Active workflow state" section, add ONE index line for it (the notes file is the artifact; MEMORY.md indexes it once). Do **not** add a noisy index line for every handoff.

### 7. Confirm

Tell the user in one short message:
- Which lane you checkpointed and the notes file path you wrote to
- The timestamp header you used
- The HEAD short-hash you captured

Do **not** print the full appended block back — the user can read it if they want. Keep the confirmation under two lines.

## Scope discipline

- Read-only git. No `git commit`, no `git push`, no `git fetch`, no working-tree modifications.
- Memory-file write only. Do **not** write to any repo working tree or the advisor-context file itself.
- Multi-lane aware: checkpoint ONLY the lane this session's CWD worktree is on. Do not write handoff blocks for other lanes' notes files. Per `.claude/rules/multi-lane-worktree.md`.
- Do **not** remove prior Session handoff blocks from the notes file. They accumulate — most-recent is always last.
- If any step fails (branch read, file missing), record the failure literally in the block and continue. A partial checkpoint is more useful than none.
