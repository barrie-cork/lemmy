---
description: Document a harness failure as RCA + session trace. Commits to current branch AND governance-v0 (cherry-pick if needed). Writes DQ kind:log entry and PMD bug entry for retro harvest.
argument-hint: [--slug <kebab-slug>] [--rca-only | --trace-only] [--phase <phase-id>] [--task <task-id>]
---

<objective>
`/doc-this` captures a post-failure signal — an RCA document + a session trace — immediately after an advisor session encounters a harness failure. It routes those documents into the retro-harvest pipeline (via a DQ `kind:"log"` entry) and into the PMD (as a `memory_type:"bug"` entry) so any future session can learn from the failure without grepping `debug/`.

The command runs **only in the advisor session** — never as a Junior task, never autonomously. It operates on the conversation context and live system state at the moment it's invoked; it does NOT re-run the failed commands.

Post-failure signal capture has been ad-hoc. This command makes harness improvement systematic.
</objective>

<usage>
**`$ARGUMENTS`** = optional flags:

```
/doc-this [--slug <kebab-slug>] [--rca-only | --trace-only]
          [--phase <phase-id>] [--task <task-id>]
```

- `--slug` — kebab-case identifier for the output files. If omitted, ask the user for a 3-5 word slug. This is the ONLY interactive step.
- `--rca-only` — write only the RCA document (skip trace + DQ log entry)
- `--trace-only` — write only the session trace (skip RCA + DQ log entry)
- `--phase` / `--task` — optional hints for PMD tagging (e.g. `--phase m2-late-1 --task 1`)

Examples:
```
/doc-this --slug m2-late-1-t1-validate-pending
/doc-this --slug validate-branch-mismatch --phase m2-late --task 2
/doc-this --slug bm-merge-dry-run-panic --rca-only
```
</usage>

<workflow>

## Phase 0 — Parse args + validate slug

1. Parse `$ARGUMENTS` for flags above.
2. Validate slug:
   - Pattern: `^[a-z0-9][a-z0-9-]*$`
   - Reject: spaces, uppercase, `/`, leading `-`
   - If omitted: ask user: "What 3-5 word kebab-case slug identifies this failure? (e.g. `validate-branch-mismatch`)"
3. Reject conflicting flags: `--rca-only` AND `--trace-only` together → STOP.
4. Compute output paths:
   - RCA:   `.claude/PRPs/debug/<slug>-rca.md`
   - Trace: `.claude/PRPs/debug/<slug>-trace.md`
5. If output files already exist → STOP: "Files already exist. Use `<slug>-2` or choose a different slug."
6. Verify `.claude/PRPs/debug/` exists: `ls .claude/PRPs/debug/`. If absent → STOP: "`.claude/PRPs/debug/` not found. Create it first."

---

## Phase 1 — Sample live system state

Run ALL of these before writing anything. Capture verbatim for the RCA "Confirmed state" section.

```bash
git branch --show-current
git worktree list
git log --oneline -6
git fetch origin
git status --short
```

DQ snapshot (Windows-safe — never `git show <branch-with-slashes>:<path>` directly):

```bash
SHA=$(git rev-parse origin/governance-v0)
git show ${SHA}:.claude/decision-queue.json > "$LOCALAPPDATA/Temp/dq-docthis.json"
python -c "
import io, json
d = json.load(io.open(r'C:\Users\barri\AppData\Local\Temp\dq-docthis.json', encoding='utf-8'))
pending = d.get('pending', [])
print('pending:', len(pending))
for e in pending:
    print(' ', e.get('id'), e.get('kind'), e.get('from'), str(e.get('question',''))[:80])
"
```

If a `validate-pending-laptop` entry is implicated in the failure, capture its full JSON.

Optional (migration failures): query docker dev DB for last applied migration:
```bash
docker exec docker-postgres-1 psql -U lemmy -d lemmy -c \
  "SELECT MAX(version) FROM __diesel_schema_migrations;"
```

Optional: `mcp__junior-brehon__list_tasks(status: "running")` — note any in-flight tasks.

---

## Phase 2 — Write RCA document (skip if `--trace-only`)

File: `.claude/PRPs/debug/<slug>-rca.md`

Section order — follow canonical exemplar `m2-late-1-t1-validate-pending-diagnosis.md`:

1. **Summary** — 2-3 sentences: what was attempted, what failed, what the root cause is
2. **Root cause** — one specific statement; multiple causes → numbered list
3. **Confirmed state** — verbatim Phase 1 output: branch, worktree list, git log, relevant remote branch tip + commits
4. **Dev DB state** — (omit entirely if not a migration or schema failure)
5. **Asset presence on branches** — what file/table/type existed on which branch; what was absent
6. **Failed attempts** — numbered list, each with:
   - Exact command (verbatim — never paraphrase)
   - One-line result
   - Root cause (one line)
7. **What needs to happen** — Option A (recommended) with concrete executable commands; Options B/C as alternatives if applicable
8. **Secondary issues** — (omit if none)
9. **DQ entry to mutate when validation passes** — full JSON of the pending entry + mutation spec: set `result: "pass"`, `answered_by: "advisor-laptop"`, `resolved_at`, move from `pending[]` to `resolved[]`. (Omit if no pending DQ is implicated.)

Content rules:
- Commands in "Failed attempts" = verbatim from conversation history; never paraphrased
- "Confirmed state" = verbatim Phase 1 output
- Mask credential/URL values: `postgres://<masked>@<host>:<port>/<db>`
- Never paste full cargo logs — reference the log file path; tail snippets (≤30 lines) acceptable

---

## Phase 3 — Write session trace document (skip if `--rca-only`)

File: `.claude/PRPs/debug/<slug>-trace.md`

Section order — follow canonical exemplar `m2-late-1-t1-validate-pending-advisor-session-trace.md`:

1. **What this is** — what failure, written for harness improvement, not for a human reader cold
2. **Preconditions** — worktree state at session start, pending DQ entries, session boundary events (compact, model switch, CWD check) — root causes often originate here
3. **Step-by-step trace** — every action the advisor took, in order:
   - `### Step N — <what advisor did>`
   - Tool/command invoked (verbatim or summarised — do NOT omit correct steps)
   - **Outcome:** ✅ Correct / ❌ Failed — one line
   - Root cause (failed steps only)
   - Missing check (if a follow-up was omitted)
   - **Harness gap label** (failed steps only): `G<N> — <short name>`
4. **Summary of harness gaps** — Markdown table: `| G# | Gap | Where | Fix |`
5. **What a correct `<handler>` for `<failure type>` should do** — numbered imperative steps, phrased concretely enough to drop into a rule file verbatim
6. **Related lessons / files** — cross-references: rules, lessons, DQ entry, other debug files

Content rules:
- Reconstruct from the actual conversation tool-call sequence — do NOT omit correct steps
- ✅ / ❌ symbols used consistently (retro harvester scans them)
- Harness-gap table is the primary harvest deliverable; do NOT inflate — if 2 gaps, that's 2 rows
- "What a correct handler should do" is imperative, step-numbered, prescriptive
- If the session crossed a compact/model boundary, that boundary is a step in the trace

---

## Phase 4 — Commit docs to current branch AND governance-v0

**Phase 4a — Commit to current branch** (wherever the advisor is when `/doc-this` runs):

```bash
git add .claude/PRPs/debug/<slug>-rca.md .claude/PRPs/debug/<slug>-trace.md
git commit -m "docs(advisor): RCA + trace for <slug>"
git push origin <current-branch>
```

This makes the documents visible to any worker or session on the same phase branch.

**Phase 4b — Cherry-pick to `governance-v0`** (only if current branch is NOT `governance-v0`):

```bash
COMMIT_SHA=$(git rev-parse HEAD)
git checkout governance-v0
git cherry-pick $COMMIT_SHA
git push origin governance-v0
git checkout <original-branch>   # return to starting branch
```

If current branch IS `governance-v0`: Phase 4a commit already lands on trunk — skip 4b.

**Why both targets matter:** Mode B lane means the advisor is often on `governance-v0`. But a failure that happened during an active impl session (on a phase branch) leaves debug docs only on the phase branch. Cherry-picking to trunk makes them reachable to ALL sessions and to the retro harvester's `governance-v0` scan. The phase branch keeps its own copy so the impl worker sees it without needing to read trunk.

If `git cherry-pick` conflicts (e.g. the debug file already exists on governance-v0 from a prior run): resolve by keeping the phase-branch version. If irresolvable without judgment, pause and ask user.

---

## Phase 5 — Write DQ `kind:"log"` entry to governance-v0

**Skip entirely if `--rca-only` or `--trace-only` was used** — the DQ log entry requires both documents for the harvester to be actionable.

Write fragment to `.claude/PRPs/debug/<slug>-dq-log-frag.json`:

```json
{
  "from": "advisor",
  "kind": "log",
  "timestamp": "<ISO8601 UTC>",
  "question": "Post-failure RCA + session trace written for <slug>",
  "options": ["promote-to-lesson", "fold-into-rule-amendment", "archive-only"],
  "context": "RCA: .claude/PRPs/debug/<slug>-rca.md. Trace: .claude/PRPs/debug/<slug>-trace.md. Harness gaps: <G1 short name>; <G2 short name>. Root cause: <one line from RCA §Root cause>.",
  "answer": "fold-into-rule-amendment",
  "answered_by": "advisor",
  "approved_by": null,
  "approved_at": null,
  "resolved_at": "<ISO8601 UTC>"
}
```

The `context` field's harness-gap list drives the retro harvest — be specific (e.g. "G1: ran commands from wrong branch; G2: DQ commands field contains Linux-style invocation").

Append to `decision-queue.json` via the helper (prevents backslash-path mangling class):
```bash
bash scripts/brehon/dq-v3-append-fragment.sh .claude/PRPs/debug/<slug>-dq-log-frag.json
```

Note: no `--pending` flag — `kind:"log"` goes directly to `resolved[]`.

Commit + push to `governance-v0` (separate commit from docs — different subjects, different reviewability):
```bash
git checkout governance-v0   # if not already there after Phase 4b
git add .claude/decision-queue.json
git commit -m "chore(decision-queue): advisor log DQ <new-id> — <slug> harness gaps"
git push origin governance-v0
```

---

## Phase 6 — Write PMD entry

Write a `memory_type:"bug"` entry so future advisor sessions can recall this failure class via `memory_search_hybrid` without needing to grep `debug/`.

```
memory_write(
  title: "RCA: <slug> — <one-sentence root cause>",
  content: "<Summary from RCA §Root cause>\n\nHarness gaps:\n<G#>: <Fix>\n<G#>: <Fix>\n\nDocs: .claude/PRPs/debug/<slug>-rca.md + <slug>-trace.md\nDQ log: <new-id>",
  memory_type: "bug",
  tags: "rca,harness<,phase-id if known>"
)
```

Tag discipline: always include `rca` and `harness`; add the phase slug if `--phase` was passed. Do NOT use `memory_type:"pattern"` (patterns require 3+ confirmed recurrences) or `memory_type:"lesson"` (lessons are authored at retro time after harvest). `"bug"` records the raw incident.

---

## Phase 7 — Output summary

```
## /doc-this complete

**Slug:** <slug>
**RCA:** .claude/PRPs/debug/<slug>-rca.md  [committed to <branch> + governance-v0]
**Trace:** .claude/PRPs/debug/<slug>-trace.md  [committed to <branch> + governance-v0]
**DQ log entry:** <new-id>  (resolved, kind:log — retro harvester will pick up)
**PMD entry:** written (memory_type:bug, tags:rca,harness,<phase>)

### Harness gaps identified
| G# | Gap | Fix |
|---|---|---|
<rows from trace §"Summary of harness gaps">

### Recommended next action
<one of:
  "Author lesson at .claude/lessons/feedback_<slug>.md"
  "Amend .claude/rules/<rule>.md §<section> — see G# Fix column"
  "Add to <phase> retro agenda item §<N>"
  — pick the most actionable>
```

If `--rca-only` or `--trace-only` was used, note which document was skipped and that the DQ log entry was not written.

</workflow>

<refusal-cases>

1. `.claude/PRPs/debug/` does not exist → STOP, ask user to create it or verify CWD.
2. Output files already exist → STOP, offer `<slug>-2` suffix.
3. Slug fails `^[a-z0-9][a-z0-9-]*$` (spaces, uppercase, leading `-`, contains `/`) → STOP, ask for valid slug.
4. `--rca-only` AND `--trace-only` both present → STOP: "Mutually exclusive flags."
5. `git cherry-pick` cannot complete without judgment calls → pause, surface to user with `git status` output; do NOT force-resolve with `-X theirs`.

</refusal-cases>

<hard-constraints>

**What NOT to include in documents:**
- Full cargo build logs (reference log file path + ≤30 line tail only)
- Credentials, token values, env-var values → mask as `<masked>`
- Full `decision-queue.json` → only the specific implicated entry
- Unrelated advisor polling steps from the same session that predate the failure

**Commit discipline:**
- Doc commit and DQ commit are always separate (Phase 4 ≠ Phase 5) — different subjects, different reviewability at retro time
- Cherry-pick to governance-v0 uses the Phase 4a SHA exactly — do NOT re-commit the files from governance-v0 (produces identical content but breaks the cherry-pick audit trail)
- Never amend a commit that has already been pushed

**DQ log attribution:**
- The `kind:"log"` entry always has `from:"advisor"`, `answered_by:"advisor"` — this runs in the advisor session
- It goes directly to `resolved[]` — never to `pending[]` (DQ invariant: `kind:"log"` in pending is a schema breach)

**Retro harvest integration:**
- The DQ entry's `context` field is the sole retro pipeline integration point
- The harvester scans entries with `kind:"log"`, `from:"advisor"`, where `context` starts with `"RCA: .claude/PRPs/debug/"`
- Debug files are evidence; they are NOT lessons. Lessons are authored at retro time by the advisor after reviewing the gaps table

</hard-constraints>

<reference-docs>

**Canonical exemplars (read before writing the documents):**
- `.claude/PRPs/debug/m2-late-1-t1-validate-pending-diagnosis.md` — canonical RCA shape
- `.claude/PRPs/debug/m2-late-1-t1-validate-pending-advisor-session-trace.md` — canonical trace shape

**Related rules + lessons:**
- `.claude/rules/multi-lane-worktree.md` — hard refusal #1 (never checkout phase branch in canonical)
- `.claude/rules/advisor-orchestrator.md` §5.2 — validate-pending-laptop handler
- `.claude/rules/decision-queue.md` — `kind:"log"` schema + append discipline
- `.claude/lessons/feedback_windows_e2e_requires_bat_wrapper.md` — Windows cargo wrapper requirement
- `.claude/lessons/feedback_lemmy_migration_runner.md` — migration runner invocation
- `.claude/lessons/feedback_windows_bash_python_git_show_tmp_traps.md` — DQ snapshot Windows recipe

**Integration with retro harvest:**
At retro time, the harvester finds DQ entries with `kind:"log"`, `from:"advisor"`, `context` prefix `"RCA: .claude/PRPs/debug/"` and:
- Promotes harness-gap `Fix` column items to `.claude/lessons/feedback_*.md` if lesson-worthy
- Routes rule-amendment items to the relevant `.claude/rules/*.md` section
- Debug files remain as supporting evidence and are NOT promoted to lessons

</reference-docs>
