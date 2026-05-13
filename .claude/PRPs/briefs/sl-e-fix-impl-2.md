---
phase: v1-SL-e
role: impl-task
task: fix-impl-2
brief_n: 9
authored: 2026-05-13
---

# [role:impl-task] v1-SL-e fix-impl-2 — DQ #191 disclosure note + DQ #193 chronology fix (2 CR findings on decision-queue.json) — see .claude/PRPs/briefs/sl-e-fix-impl-2.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-e fix-impl-2 — DQ disclosure + chronology corrections (2 CR major findings on decision-queue.json)`

## §2 Scope

Address **2 CR triage-approved fix-in-pr findings** from PR #127 review, both on `.claude/decision-queue.json`. Single file, two surgical edits.

### Finding 1 — cr-2 on DQ #191: scope-decision should have been user-gated

- **Severity:** major
- **CR text (decision-queue.json:3333):** *"Surface these scope decisions to the user before resolving them"*
- **Diagnosis:** DQ #191 (`from: "advisor", kind: "clarify"`) was advisor-self-resolved at `2026-05-12T10:39:09Z` — same minute as the `timestamp` field. The question asked the planner whether to fold cr-3/cr-4 carry-forwards from SL-d retro into test #1 assertions OR add dedicated §13 tasks. This was a **scope-shaping decision** that per `.claude/rules/advisor-orchestrator.md` §3.2 "Mandatory user gates" gate 2 (judgment-heavy DQ — ADR-affecting / scope-changing / visible-to-others impact) should have surfaced to user via AskUserQuestion, not self-resolved.
- **Why it self-resolved instead:** during the planning-clarify pass `/brehon-clarify` was run in `--mode advisor`, which defaults to self-resolution for any question the advisor could answer from evidence. The fold-into-assertions answer was defensible (PRD §9.3 lists 3 SL-d contributions, no notification step; the cr-3/cr-4 idempotency assertions are bounded test-shape decisions, not scope expansions). But the line between "advisor-mode self-resolvable" and "judgment-heavy user-gate-required" is fuzzy — and CR's point is correct: when in doubt, surface.
- **Fix (durable, non-destructive):** add a `notes` field to DQ #191 with disclosure language acknowledging the process-miss. DO NOT change `answer` or `answered_by` (the work has shipped; rewriting the audit trail would breach the v2 forward-only rule per `.claude/rules/decision-queue.md` "Schema (v2)" first paragraph: *"Do not rewrite historical entries — pre-v2 idiosyncrasies stay as the audit trail. Forward-only consistency."*).

  Add this `notes` field to DQ #191, between `"resolved_at"` and the closing `}`:

  ```json
  "notes": "Process disclosure (added 2026-05-13 per CR cr-2 on PR #127): this scope-shaping clarify was self-resolved by advisor in --mode advisor without explicit user-gate per advisor-orchestrator.md §3.2 gate 2. The fold-into-assertions answer is defensible (cr-3/cr-4 are bounded assertion additions, not scope expansions), but the judgment call should have surfaced to user. Carry-forward for v1-SL-lane-meta-retro: when /brehon-clarify --mode advisor encounters a question whose answer would change the §13 task count (add tasks, split tasks, fold tasks together), upgrade to user-relay even if evidence supports self-resolution."
  ```

### Finding 2 — cr-3 on DQ #193: impossible chronology

- **Severity:** major
- **CR text (decision-queue.json:3369):** *"Fix the impossible chronology in DQ #193"*
- **Diagnosis:** DQ #193 has `"timestamp": "2026-05-12T14:30:00Z"` but `"resolved_at": "2026-05-12T13:48:09.735263+00:00"`. The entry was "resolved" 42 minutes BEFORE it was raised. Reading the entry: this is a ci-watcher mutation entry (validate-pending on Task 1 workspace check; `result: "pass"`). The ci-watcher subagent captured the workflow run's actual finish time as `resolved_at` (correctly — that's when the workflow concluded) but the `timestamp` field was set later when the entry was committed to the phase branch DQ. The two times are in the wrong order because the entry's `timestamp` represents "raised by impl-task post-push" but was filled in by ci-watcher's mutation step which ran AFTER reading the workflow.
- **Fix:** set `timestamp` to a value `<= resolved_at`. Use the workflow run's `created_at` (when impl-task triggered it, which is the natural "raise" moment for validate-pending). Read the actual creation time:

  ```bash
  gh run view 25737219926 --repo barrie-cork/lemmy --json createdAt
  ```

  Then update DQ #193's `timestamp` to that value (or any time strictly before `resolved_at: 2026-05-12T13:48:09.735263+00:00`). Reasonable inferred value: the impl-task push time is ~5-10 min before the workflow finish; if `gh run view` is unavailable, use `2026-05-12T13:43:00Z` (5 minutes before resolved_at — typical workspace-check duration).

  **Per the v2 forward-only rule** (decision-queue.md): this IS a permitted edit because the entry's `timestamp` is factually wrong (post-resolution time stamped as raise time). The fix corrects a data-entry error, not the substance of the decision. Add a `notes` field documenting the correction:

  ```json
  "notes": "Timestamp corrected 2026-05-13 per CR cr-3 on PR #127. Original timestamp value '2026-05-12T14:30:00Z' was post-resolution (after ci-watcher's mutation step). Corrected to <T> = workflow run 25737219926 createdAt (gh run view output), which is the natural raise moment for a validate-pending entry. resolved_at unchanged."
  ```

## §3 Required reading

**Mandatory file-class lessons (per `.claude/rules/advisor-orchestrator.md` §2.4 file-class table):**

1. `.claude/rules/decision-queue.md` — DQ schema v2; "Schema (v2)" §forward-only rule on historical edits; "Mid-task visibility" §commit + push from worker branch; Recipe 1 if blocker found.
2. `.claude/rules/advisor-orchestrator.md` §3.2 — Mandatory user gates (gate 2 is the canonical reference for "judgment-heavy DQ").
3. `.claude/rules/branch-manager.md` — file ownership boundaries (this fix is impl-task scope, not BM; bm-task cannot edit decision-queue.json contents).

**Plan + review sections:**

4. `.claude/PRPs/reviews/pr-127-findings.yaml` — find ids cr-2, cr-3.
5. `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md` §13 Task 1 — context for what DQ #191 was deciding (cr-3/cr-4 carry-forwards).
6. `.claude/lessons/feedback_json_dump_ensure_ascii_false.md` — mandatory `ensure_ascii=False` for JSON writes.

## §3a Handover from prior task

- **Phase tip:** `e967c8959`.
- **Sister task:** sl-e-fix-impl-1 (5 e2e.rs findings) ran (or is running) in parallel-or-serial-prior. Either way, this brief touches **only `.claude/decision-queue.json` + `.claude/PRPs/reviews/pr-127-findings.yaml`** — no overlap with fix-impl-1's e2e.rs edits. Worktree branches are isolated; if fix-impl-1 lands first, this task forks off the new tip.
- **DQ #191 location in current file:** entry starts at line ~3321, `resolved_at` at line 3333. CR's `discussion_r3231442349` cites line 3333 — the `resolved_at` line — the point being that self-resolution was wrong, not that the timestamp itself is malformed.
- **DQ #193 location in current file:** entry has `timestamp: 2026-05-12T14:30:00Z` and `resolved_at: 2026-05-12T13:48:09.735263+00:00`. Both are textually present in the JSON.
- **DO NOT modify any other DQ entry.** Only #191 and #193 get edits in this task. The `notes` field is OPTIONAL in the v2 schema — adding it is a non-breaking forward-only edit.

## §4 Constraints

- **Touch only:** `.claude/decision-queue.json` + `.claude/PRPs/reviews/pr-127-findings.yaml`. NO `crates/`, NO `migrations/`, NO `tests/`, NO other `.claude/` files.
- **NO `replace_all`** — `notes` field doesn't exist anywhere in the file yet, but `resolved_at` appears in every resolved entry. Use surgical Python `json.load` + mutation + `json.dump(..., ensure_ascii=False, indent=2)` to add the `notes` fields. NEVER use `Edit` with a `resolved_at` anchor — the line is non-unique.
- **JSON write pattern:**
  ```python
  import json, io
  with io.open('.claude/decision-queue.json', encoding='utf-8') as f:
      d = json.load(f)
  for e in d['resolved']:
      if e['id'] == 191:
          e['notes'] = "<text from §2 Finding 1>"
      elif e['id'] == 193:
          # First fetch workflow createdAt:
          # gh run view 25737219926 --repo barrie-cork/lemmy --json createdAt
          e['timestamp'] = "<corrected ISO 8601 from gh output>"
          e['notes'] = "<text from §2 Finding 2>"
  with io.open('.claude/decision-queue.json', 'w', encoding='utf-8') as f:
      json.dump(d, f, ensure_ascii=False, indent=2)
  ```
- **`gh run view` may fail** if the run was garbage-collected. If `gh run view 25737219926 --repo barrie-cork/lemmy --json createdAt` returns an error: use `2026-05-12T13:43:00Z` as the corrected timestamp (5 min before `resolved_at`). Document this fallback in the `notes` field.
- **`--repo barrie-cork/lemmy`** on all gh commands (per `.claude/rules/gh-pr-fork-target.md`).
- **Forward-only v2 rule preserved**: do NOT touch any other fields on #191 or #193 (`question`, `options`, `context`, `answer`, `answered_by`, `resolved_at`). Only `notes` is added (new field, non-breaking). Only #193's `timestamp` is corrected (because it's factually wrong — corrects a data-entry error per the rule's spirit; documented in `notes`).
- **Update findings YAML**: after commit, set `bucket: done` + `addressed_in: <commit-sha>` for cr-2 and cr-3 in `.claude/PRPs/reviews/pr-127-findings.yaml`. Regenerate `counters` block (major.open: 3→1; major.done: 0→2).
- **Watchpoint**: at task-end, run `git diff governance-v0..HEAD --stat` from worker branch tip. EXPECT: two files only: `.claude/decision-queue.json | +X -0` (notes additions + timestamp correction) and `.claude/PRPs/reviews/pr-127-findings.yaml | +2 -2`. NO `crates/` changes.
- **Commit message:** `chore(decision-queue): DQ #191 disclosure note + DQ #193 chronology correction (CR cr-2, cr-3 on PR #127)`. Footer: `Addresses CR cr-2 (DQ #191 scope-shaping should have been user-gated; notes field documents process-miss for retro carry-forward) and CR cr-3 (DQ #193 timestamp post-dated resolved_at; corrected to workflow createdAt with notes).`
- **Subject-line pattern matches** `^(chore|docs)\((advisor|decision-queue)\)` per `.claude/rules/decision-queue.md` "Attribution integrity §Detection" — this is a `chore(decision-queue):` commit and that's the correct scope.
- **Shape G**: NO validate-pending raise needed. This edit is meta-work (DQ edits + review YAML updates); it does NOT touch `crates/` and Phase 1 workspace-check is not triggered. Per `feedback_pr_per_phase.md` + `.claude/rules/phase-branch.md` "Direct on governance-v0" policy spirit, this is the kind of edit that would normally go direct on governance-v0 — but the audit trail belongs to PR #127, so commit on `phase-v1-SL-e` (worker branch forks off SL-e tip).
- **DQ atomic raise:** NONE in this task. No validate-pending. End task after commit + push.
- **Encoding:** `ensure_ascii=False` mandatory (per `feedback_json_dump_ensure_ascii_false.md`).
- **Attribution:** `from: "impl"` if any blocker is raised; the task DOES NOT itself write a new DQ entry — only mutates two existing entries' `notes` field (+ #193 `timestamp`). Per the v2 forward-only rule, the mutation is documented in-entry via the new `notes` field; commit subject does the rest.
- **DO NOT post a PR comment.** Same as fix-impl-1 — posting happens after both fix-impl tasks land + user gate clears (final summary comment).
