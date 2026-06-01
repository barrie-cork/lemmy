---
role: bm-task
verb: <bm-cut|bm-pr|bm-poll-cr|bm-triage|bm-merge|bm-push|bm-ping>
phase: <phase-slug>      # e.g. v1-quality-r2a
pr_number: <N-or-null>   # null for bm-cut (PR doesn't exist yet)
created: <YYYY-MM-DD>
related_dq: <id-or-null>
---

# Brief: <phase> bm-<verb> — <one-line summary>

> **Canonical sibling reference:** this template was promoted 2026-05-29 after the
> third recurrence (post bm-cut, bm-pr, bm-poll-cr, bm-merge, bm-triage briefs across
> ~30 sub-phases). Per `feedback_read_canonical_before_writing_spec.md`, also `Glob`
> + `Read` 1-2 sibling instances of the SAME verb (e.g. prior `*-bm-pr-*.md` if
> authoring a bm-pr brief) before filling this template — the per-verb shape varies
> within the bm-task family.

## 1. Role + dispatch line

`[role:bm-task] <phase> bm-<verb> — see .claude/PRPs/briefs/<file>.md`

You are the **bm-task** subagent (Haiku 4.5 per your frontmatter — git/yq/gh ops, no
heavy reasoning). Execute the branch-manager verb `bm-<verb>` per
`.claude/commands/bm/bm-<verb>.md`.

## 2. Scope

<One-paragraph summary of what this verb does in this specific phase / for this
specific PR. Include the load-bearing IDs:>

- **Phase:** `<phase-slug>` (branch `phase-<phase-slug>`)
- **PR number:** #<N> on `barrie-cork/lemmy` (omit for bm-cut)
- **Trunk SHA:** `<sha>` on `governance-v0` (relevant for bm-cut, bm-merge gate)
- **Phase tip:** `<sha>` on `phase-<phase>` (relevant for bm-pr, bm-poll-cr, bm-merge)

**Produce:**

- `<artifact>` — <what changes; cite the verb script's expected output>
- `<artifact>` — <what changes>
- Runlog entry in `.claude/runlog/<phase>-runlog.md` noting key SHAs and timestamps

**Do NOT (verb-level scope boundaries):**

- <out-of-scope action — e.g. "Open the PR" if this is a bm-cut>
- <out-of-scope action — e.g. "Merge the PR" if this is a bm-poll-cr>
- <out-of-scope action — e.g. "Post any PR comment" if this is anything but a confirmed bm-comment>

### 2.1 Per-verb expected scope (cheat sheet — fill in only the row that applies)

| Verb | Scope summary | Produces |
|---|---|---|
| `bm-cut` | Branch `phase-<phase>` from `governance-v0` HEAD; push with `-u`; runlog entry. **No PR.** | new branch on origin |
| `bm-pr` | Open PR `phase-<phase> → governance-v0` (not draft, `--repo barrie-cork/lemmy`); record PR number. | PR opened |
| `bm-poll-cr` | Poll CodeRabbit + Copilot on PR #N; ingest findings into `.claude/PRPs/reviews/pr-N-findings.yaml`; advance `last_poll_at` + `poll_count`. | findings YAML updated |
| `bm-triage` | Apply advisor-approved bucket decisions to `pr-N-findings.yaml`; regenerate counters. | findings YAML updated |
| `bm-merge` | After all gates pass, `gh pr merge N --merge --delete-branch`; runlog entry. | PR merged |
| `bm-push` | Force-push correction or recovery push to phase / worker branch (REQUIRES advisor user-gate confirm). | branch tip advanced |
| `bm-ping` | Telegram notification ping for one of the 5 event shapes (REQUIRES advisor user-gate confirm). | Telegram message sent |

## 3. Required reading

These are mandatory for **every** bm-task brief:

- `.claude/commands/bm/bm-<verb>.md` — the verb script (canonical procedure)
- `.claude/rules/branch-manager.md` — file-ownership boundaries, autonomy bounds, session-start ritual
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every `gh` command
- `.claude/rules/phase-branch.md` — base MUST be `governance-v0`, no squash, no draft

Add verb-specific reading from this table — fill in only the row(s) that apply:

| Verb | Additional required reading |
|---|---|
| `bm-cut` | `.claude/PRPs/plans/<phase>.plan.md` (committed before this task) |
| `bm-pr` | `.claude/PRPs/reports/<phase>-complete-report.md` (if exists; pulled into PR body) |
| `bm-poll-cr` | `.claude/PRPs/reviews/SCHEMA.md` (findings YAML schema), `.claude/PRPs/reviews/pr-<N>-findings.yaml` (existing artifact to update in place), `.claude/lessons/feedback_pr_review_triage_pattern.md` (4-bucket), `.claude/lessons/feedback_severity_labels_dont_imply_semantic.md`, `.claude/lessons/feedback_verify_automated_reviewer_claims_against_compiler.md` |
| `bm-triage` | `.claude/PRPs/reviews/SCHEMA.md`, prior `bm-poll-cr` findings YAML for the same PR |
| `bm-merge` | `.claude/PRPs/reports/<phase>-verify.md` (advisor verify-gate output), all gates pre-cleared by advisor |

## 4. Constraints

**Hard refusals — apply to EVERY bm-task verb:**

1. **NEVER touch `crates/`, `migrations/`, `tests/`, `crates/server/tests/`, `docs/brehon-law-inspired-network/`, `.claude/PRPs/plans/**`, `.claude/PRPs/prds/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`** (per `.claude/rules/branch-manager.md` §"File ownership boundaries").
2. **NEVER post a PR comment** (`gh pr comment`) without explicit advisor / user confirmation. Draft only into `.claude/PRPs/reviews/pr-<N>-comment.md`.
3. **NEVER submit a PR review** (`gh pr review --approve|--request-changes`). Draft only.
4. **NEVER merge a PR** outside an explicit `bm-merge` task with advisor pre-cleared gates + user merge-confirm.
5. **NEVER force-push** (`git push --force`, `--force-with-lease`) without explicit advisor / user confirmation (separate `bm-push` task).
6. **NEVER delete a branch** (local or remote) without explicit confirmation. The exception is `gh pr merge --delete-branch` inside a confirmed `bm-merge`.
7. **NEVER send a Telegram ping** without explicit confirmation (separate `bm-ping` task; one of 5 allowed event shapes per `feedback_telegram_scope_notification_only.md`).
8. **NEVER write `answered_by: "advisor"` or `approved_by`** in any DQ entry. The only valid `answered_by` labels for a bm-task are `"bm-self-resolved"` (per `decision-queue.md` Attribution integrity). Hard refusals #1 + #8 in `decision-queue.md`.
9. **NEVER use the abolished `next_id = max+1` recipe** if you write a DQ entry — use `bash scripts/brehon/dq-v3-new-entry.sh` + `bash scripts/brehon/dq-v3-append-fragment.sh` (Hard refusal #9 in `decision-queue.md`).
10. **`--repo barrie-cork/lemmy` MANDATORY** on every `gh` command (per `gh-pr-fork-target.md`). Without it, `gh` defaults to upstream `LemmyNet/lemmy` because this is a fork.
11. **Base branch MUST be `governance-v0`** (never `main` — `main` is for upstream rebases only, per `phase-branch.md`).
12. **PR MUST NOT be draft** (CodeRabbit skips drafts, per `phase-branch.md`).
13. **Do NOT squash** the PR at merge (per `phase-branch.md` — task-per-commit history is load-bearing for retros).
14. **Refuse silently and raise a `kind: "blocker"` DQ** if a required artifact is missing (e.g. `pr-<N>-findings.yaml` for a `bm-poll-cr` re-poll). Do NOT author the artifact from scratch unless explicitly instructed.

**Verb-specific constraints** — add the row(s) that apply:

| Verb | Verb-specific constraints |
|---|---|
| `bm-cut` | Branch name MUST be `phase-<phase>` exactly. Plan file MUST exist on trunk before this task. No code commits — topology only. |
| `bm-pr` | Body MUST include §Summary + §Validation + §Plan reference + §Issues addressed (closes-N where applicable). Title under 70 chars. **Phase-1d Linux gate:** if the diff touches `Cargo.toml`/`Cargo.lock`/`migrations/**` or adds `cfg(unix)`/`cfg(target_os)`, STOP unless a `validate-pending-laptop-linux` DQ is at `result:pass` (per `bm-pr.md` Phase-1d). bm-task only checks — never runs cargo/Docker. |
| `bm-poll-cr` | Update findings YAML IN PLACE preserving stable finding IDs + four-bucket layout. Advance `last_poll_at` (ISO 8601 UTC) + `poll_count`. Counters regenerate on every write. For new findings, assign fresh `id` slugs (`cr-new-3`, `cr-new-4`, etc.). |
| `bm-triage` | Apply ONLY the advisor-approved bucket decisions listed in §3 of the brief. Do NOT improvise additional bucket assignments. Counters regenerate. |
| `bm-merge` | Pre-merge gate verification (§3 of the brief lists what to verify). If any gate fails, raise `kind: "blocker"` DQ and stop. |

**Commit subject discipline:**

- `bm-cut`, `bm-pr` (topology / PR open) → no special commit subject (the verb either creates a branch with no commit, or opens a PR remotely).
- `bm-poll-cr`, `bm-triage` (findings YAML updates) → `chore(reviews): bm-<verb> — PR #<N> — <one-line summary>`.
- `bm-merge` → no commit subject (the merge happens server-side via `gh pr merge`).
- **NEVER** `chore(advisor):` or `chore(decision-queue):` — those are advisor-exclusive commit-subject patterns per `decision-queue.md` Attribution integrity §Detection (subject pattern `^(chore|docs)\((advisor|decision-queue)\)`).

**HANDOVER trailer** (mandatory on commits that produce artifacts — bm-poll-cr / bm-triage / and ad-hoc bm-task variants that write files):

```yaml
HANDOVER:
  task: <phase>-bm-<verb>
  filesCreated: [<list>]
  filesModified: [<list>]
  keyDecisions:
    - "<one-line decision>"
  notes: "<counters or status — e.g. 'done=3, fix-in-pr=0, rebut=4, carry-forward=2, wont-fix=0'>"
```

**Push discipline:** push to the worker branch (`junior/role-bm-task-...`); the daemon
finalize-merges into `phase-<phase>` (for bm-poll-cr / bm-triage / similar on-phase
writes). For `bm-cut`, push the new phase branch directly to origin with `-u` (that's
the verb's purpose). For `bm-pr`, the verb opens a PR remotely (no push). For
`bm-merge`, no push (server-side merge).

## 5. Success signals

**Verb-specific success criteria** — fill in for this brief:

- <signal 1 — e.g. "PR #N opened on barrie-cork/lemmy, state=OPEN, mergeable=MERGEABLE">
- <signal 2 — e.g. "findings YAML counters block: done=3, fix-in-pr=0, rebut=4, carry-forward=2, wont-fix=0">
- <signal 3 — e.g. "runlog entry appended with verb + SHA + timestamp">
- <signal 4 — verb-specific final commit subject literal>
- HANDOVER trailer present (where applicable)
- Push to worker branch / origin succeeds

## 6. Out of scope

**Common across all bm-task verbs:**

- Editing impl code, plan files, design docs, or any file in the never-touch list.
- Cutting a new branch outside the verb's scope (e.g. a `bm-poll-cr` brief never cuts a branch).
- Posting PR comments / reviews / Telegram pings without explicit confirmation.
- Triggering CI workflow runs (CI triggers on push by the verb that actually pushes).

**Verb-specific exclusions** — fill in for this brief:

- <e.g. "Bm-merge or bm-cut work for separate lanes (separate sessions)">

## 7. Done

Worker exits with all §5 success signals satisfied, the chore commit (where
applicable) pushed to its worker branch, and the daemon finalize-merge (where
applicable) advancing `phase-<phase>` tip. Advisor session resumes at the next
stage-shape transition per `.claude/rules/advisor-orchestrator.md` §3.1.

**Advisor post-condition (mandatory, within 1 poll tick):** run the verb's 3-signal
check from the table in `advisor-orchestrator.md` §3.1 "BM post-dispatch 3-signal
check". Do NOT advance the stage on BM `done` alone — `result:success` is a
hypothesis. Catch-fire to user if any signal fails.

---

## Template usage notes (delete this section when authoring a real brief)

**When to use this template:**

- Authoring any `bm-task` brief (any of the 7 verbs).
- 80+ prior bm-task briefs followed this shape; this template encodes the recurring
  invariants. Sibling-pattern check (per `feedback_read_canonical_before_writing_spec.md`)
  is still required — read 1-2 recent briefs for the SAME verb before filling this in,
  because per-verb scope varies within the family.

**Promotion provenance:** promoted 2026-05-29 after explicit user request, third
recurrence threshold met (bm-cut + bm-pr + bm-poll-cr + bm-triage + bm-merge briefs
across ~30 sub-phases). See conversation context + `feedback_lesson_lifecycle_chain.md`.

**Sections to ALWAYS include in a real brief:**

- §1 Role + dispatch line (verbatim format — the matching subagent reads it)
- §2 Scope (with the load-bearing IDs: phase, PR number, trunk SHA, phase tip)
- §3 Required reading (always 4 common files + verb-specific additions)
- §4 Constraints (always all 14 hard refusals + verb-specific row + commit-subject discipline + HANDOVER trailer schema)
- §5 Success signals (verb-specific)
- §6 Out of scope (verb-specific exclusions)
- §7 Done (one paragraph)

**Sections to OMIT (the impl-task template has them; bm-task does NOT):**

- §0 Pre-flight (impl-task agent runs forbidden-window check; bm-task doesn't run cargo)
- §2.0 Scope gate — e2e.rs fix-impl size cap (impl-task only; bm-task never touches `crates/`)
- §2.5 Worker self-tests (impl-task only; bm-task verifies via the verb script's own output)
- §3a Handover from prior cohort (impl-task cohort dispatch only; bm-task is never `[P]`-cohort)
- §4.3 validate-pending-laptop entry shape (impl-task only; bm-task never runs cargo gates)

**Sibling pattern lookups (do these BEFORE filling in this template):**

```bash
# For a bm-poll-cr brief, find the 2 most recent canonical examples:
ls -t .claude/PRPs/briefs/*-bm-poll-cr-*.md | head -2

# For a bm-cut brief:
ls -t .claude/PRPs/briefs/*-bm-cut-*.md | head -2

# For a bm-merge brief:
ls -t .claude/PRPs/briefs/*-bm-merge-*.md | head -2
```

Read them, copy the verb-specific section shapes, then fill in this template's
remaining structure.
