---
role: impl-task
plan_task: 1
phase: v1-SL-b
created: 2026-05-04
related_dq: null
---

# Brief — v1-SL-b Task 1 — Extend RevokeEndorsement DTO + add RevokeEndorsementResponse

## 1. Role + dispatch line

`[role:impl-task] sl-b-impl-1 — see .claude/PRPs/briefs/sl-b-impl-1.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan Task 1 from
`.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` §13 — extend the
existing `RevokeEndorsement` DTO and add a new `RevokeEndorsementResponse`
DTO. **One file modification, one commit.**

## 2. Scope

**Produce:**

1. **One commit** modifying `crates/api/api_common/src/governance.rs`:
   - Extend existing `RevokeEndorsement` struct with `reason: String` field.
   - Drop `Copy` from its derive stack (`String` isn't `Copy`).
   - Keep `Default` (`String` defaults to `""`; handler will reject empty-after-trim).
   - Add new `RevokeEndorsementResponse` struct with three fields:
     `endorsement_id: EndorsementId`,
     `revoked_at: DateTime<Utc>`,
     `liability_chain_severed_for_cases: Vec<ModerationCaseId>`.
2. **Push the worker branch** to `origin/junior/<task-slug>`.
3. **Write a `kind: "validate-pending"` DQ entry** capturing the workflow
   run id of `cargo-validate-workspace.yml` per Shape G Layer G2.

**Do NOT** in this task:

- Touch any other file in `crates/**` (handler is Task 2; route is Task 3;
  e2e tests are Tasks 4-12).
- Run cargo locally — Shape G plan; cargo runs on GH-hosted runners after
  push.
- Modify `.claude/PRPs/plans/**` or `.claude/PRPs/briefs/**`.
- Add new ENTRY_KIND consts, CaseStatus variants, or schema columns —
  ALL substrate shipped in SL-a.
- Write to `decision-queue.json` outside the validate-pending entry +
  any blocker entries.

## 3. Required reading

Read in this order before writing the Edit:

1. **Plan §13 Task 1** (`.claude/PRPs/plans/v1-sponsor-liability-b.plan.md`)
   — the canonical IMPLEMENT block + GOTCHAs. This brief mirrors it; the
   plan is canonical.
2. **Plan §4.1 + §4.2** (architectural decisions + watchpoints) — confirms
   the `Copy`-drop reasoning and the `Default`-preservation rationale.
3. **`crates/api/api_common/src/governance.rs`** — read the file. Note:
   - Existing `RevokeEndorsement` at line ~311 with current derive
     `#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]`.
   - **Mirror reference:** `CreateEndorsement` + `CreateEndorsementResponse`
     at lines 287-305. Same derive pattern, same doc-comment style, same
     ts-rs gating.
   - Existing imports at top of file — `EndorsementId`, `ModerationCaseId`
     already imported (per plan §13 IMPORTS note).
4. **PRD §5.1** (`.claude/PRPs/prds/v1-sponsor-liability.prd.md`) — DTO
   field requirements (canonical source for `liability_chain_severed_for_cases`
   field name + type).
5. **`.claude/rules/decision-queue.md`** — schema-v2 for the
   `validate-pending` entry shape; `from: "impl"`, `kind: "validate-pending"`,
   `branch: <worker branch>`, `phase_task: "sl-b-impl-1"`,
   `workflow_run_id: <id>`.
6. **Lessons** (Glob `.claude/lessons/`):
   - `feedback_features_full_workspace_only.md` — workspace-check workflow
     uses `--workspace --features full`; SL-b's ts-rs derives activate
     under `full`.
   - `feedback_clippy_test_style.md` — R1, R3 sweep — after struct edit,
     `git grep -l 'RevokeEndorsement {'` to find construction sites
     needing `..Default::default()` propagation.
   - `feedback_pr_per_phase.md` — one commit per task discipline.
   - `feedback_explicit_file_arrays_on_tasks.md` — FILES YAML block from
     plan §13 binds the modify-set; Task 1 modifies exactly
     `crates/api/api_common/src/governance.rs`.

## 3a. Handover from prior cohort

**From Task 0 (sl-b-impl-0, task #115):** all 14 probes PASS. Task 0
made no commits (verification-only); phase-v1-SL-b tip remains at
`4827ea99c chore(advisor): brief sl-b-impl-0 — Task 0 pre-flight harness
audit`. SL-a substrate confirmed in place: 3 CaseStatus variants
(enums.rs:416/421/427); schema columns `grace_expires_at` + `liability_escape_reason`
(schema.rs:799/800); ENTRY_KIND consts at governance_log.rs:195/197 + shim
re-exports at governance_log.rs:51/74; 13 config keys seeded
(2 const decls + 2 match arms + 2 SEEDED_KEYS_WITH_CONSTS); existing
`RevokeEndorsement` DTO at api_common/src/governance.rs:311; existing
`/endorsement` route at routes/src/lib.rs:523. No concurrent PRs touch
SL-b target files.

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branched off `phase-v1-SL-b` (tip
  `4827ea99c`).
- `git branch --show-current` should return a `junior/role-impl-task-...-116`
  branch (or whatever the next task-id is — adapt to your actual worktree
  branch name).
- One commit at task end with subject:
  `feat(v1-SL-b): extend RevokeEndorsement DTO with reason + add RevokeEndorsementResponse (task 1)`
- After the impl commit, push the worker branch and write the
  `validate-pending` DQ entry as a SECOND commit on the same branch with
  subject: `chore(decision-queue): impl raised DQ #<next-id> — sl-b-impl-1 validate-pending`.
- Push the worker branch to origin AFTER both commits land. Junior's
  daemon finalize-merges into `phase-v1-SL-b` on completion.

### Implementation discipline

- **Edit-pattern:** anchor on `pub struct RevokeEndorsement` (the
  current 6-line struct). Replace the derive line + struct body verbatim
  per plan §13 IMPLEMENT.
- **Insert-pattern for `RevokeEndorsementResponse`:** anchor at the close
  of the new `RevokeEndorsement` struct (`}` on its own line); insert the
  new struct immediately after, separated by ONE blank line.
- **Imports:** confirm `DateTime` + `Utc` are in scope. If absent, add
  `use chrono::{DateTime, Utc};` to the file's existing `use` block (NOT
  inside any cfg-gated import block — plain). Note: `Vec` is std-prelude
  (always in scope); `EndorsementId` + `ModerationCaseId` already
  imported.
- **R3 sweep:** after the struct change, run
  `git grep -l 'RevokeEndorsement {'` (note the trailing space + brace)
  in the worktree. Pre-SL-b expected: zero matches. If matches appear
  (e.g. a test fixture or another caller), each must use
  `..Default::default()` syntax for forward compatibility. Per plan §13
  GOTCHA. If a match appears that needs more than `..Default::default()`
  to fix (e.g. it requires a `reason` value to be set explicitly), STOP
  and file a DQ blocker — that's a scope-expansion the advisor must
  resolve.
- **Do NOT touch the existing `CreateEndorsement` / `CreateEndorsementResponse`
  pair.** They're the canonical mirror reference — observation only.
- **Do NOT add new fields beyond the three named** in
  `RevokeEndorsementResponse`. PRD §5.1 names exactly these three.

### Validate-pending DQ entry shape

Per `.claude/rules/decision-queue.md` schema-v2 + Shape G Layer G2:

```json
{
  "id": <next-int>,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<NOW_ISO>",
  "subject": "sl-b-impl-1 workspace-check validate-pending",
  "branch": "junior/<task-slug>",
  "phase_task": "sl-b-impl-1",
  "workflow_run_id": <int from gh run list>,
  "commands": ["cargo check --workspace --features full", "cargo clippy --workspace --features full --no-deps -- -D warnings", "cargo test --no-run -p lemmy_server --test e2e"],
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

After pushing the impl commit, capture the `workflow_run_id`:

```bash
gh run list --repo barrie-cork/lemmy --branch <your-worker-branch> --workflow cargo-validate-workspace --limit 1 --json databaseId --jq '.[0].databaseId'
```

If `gh run list` returns empty (workflow hasn't started yet), wait up
to 60 seconds and retry — GH Actions ingestion lag.

### DQ attribution

- No `answered_by: "advisor"` or `"user"` from this subagent.
- Self-resolve only as `"impl-self-resolved"` (rare for impl-task; only
  for trivial questions that the impl can answer with file evidence).
- Any pending DQ entry written from this task uses `from: "impl"`.

### Hard refusals

- Do NOT modify any file outside `crates/api/api_common/src/governance.rs`
  + `.claude/decision-queue.json` (for the validate-pending entry).
- Do NOT add a `Copy` derive on `RevokeEndorsementResponse` (Vec + DateTime
  are non-Copy; would fail to compile).
- Do NOT remove `Default` from `RevokeEndorsement` (`Vec::new()` defaults
  fine; handler relies on `Default::default()` for backwards-compat
  fixture construction).
- Do NOT add scope creep — no handler logic, no route registration, no
  e2e test edits. Tasks 2/3/4-12 are separate.
- Do NOT push the worker branch BEFORE both commits (impl + DQ entry)
  land. Push is the Shape-G Layer G2 trigger.
- Do NOT touch `.claude/PRPs/plans/**` or `.claude/PRPs/briefs/**`.

## 5. Acceptance

Task 1 passes if:

- `crates/api/api_common/src/governance.rs` modifications:
  - `RevokeEndorsement` struct has 2 fields (`endorsement_id`, `reason`)
    AND derive line does NOT contain `Copy` AND derive line DOES contain
    `Default`.
  - New `RevokeEndorsementResponse` struct present with 3 fields
    (`endorsement_id`, `revoked_at`, `liability_chain_severed_for_cases`).
- One impl commit + one DQ-entry commit on the worker branch.
- Worker branch pushed to origin.
- DQ entry written with `kind: "validate-pending"`, `from: "impl"`,
  populated `workflow_run_id`.
- ci-watcher's later poll of `cargo-validate-workspace.yml` returns
  `conclusion: "success"`.

(The last bullet is verified by ci-watcher — not by this Task 1 worker.
Task 1 worker is done after writing the DQ entry + pushing.)
