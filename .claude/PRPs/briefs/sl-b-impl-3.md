---
role: impl-task
plan_task: 3
phase: v1-SL-b
created: 2026-05-04
related_plan: .claude/PRPs/plans/v1-sponsor-liability-b.plan.md §13 Task 3
---

# Brief — v1-SL-b Task 3 — Register `/endorsement/revoke` route + import handler

## 1. Role + dispatch line

`[role:impl-task] sl-b-impl-3 — see .claude/PRPs/briefs/sl-b-impl-3.md`

You are the **impl-task** subagent (Sonnet 4.6). Single-file edit on
`crates/api/routes/src/lib.rs`: extend the `governance::{...}` use block
with one new import, and register one new route.

## 2. Scope

**Produce:**

1. **One impl commit** with exactly two edits to
   `crates/api/routes/src/lib.rs`:
   a. Insert `revoke_endorsement::revoke_endorsement,` in the
      `governance::{...}` use block, after the existing
      `request_appeal::request_appeal,` line (~line 151).
   b. Insert
      `.route("/endorsement/revoke", post().to(revoke_endorsement))`
      immediately after the existing `/endorsement` line (~line 523).
2. **Push the worker branch** to `origin/junior/<task-slug>`.
3. **Write a `kind: "validate-pending"` DQ entry** capturing the
   workflow run id of `cargo-validate-workspace.yml` per Shape G
   Layer G2. This is a SECOND commit on the same worker branch.

**Do NOT** in this task:

- Touch any file other than `crates/api/routes/src/lib.rs` and
  `.claude/decision-queue.json`.
- Modify any other route registration or use-block entry.
- Refactor surrounding code or "improve" anything.
- Change indentation of unrelated lines.
- Add new tests (Tasks 4-12 ship the e2e tests).
- Run cargo locally — Shape G; cargo runs on GH-hosted runners.
- Modify `.claude/PRPs/plans/**` or `.claude/PRPs/briefs/**`.

## 3. Required reading

Read in this order before writing the Edits:

1. **`.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` §13 Task 3**
   (line 1863+) — canonical task spec, FILES YAML, MIRROR §10.6,
   GOTCHAs.
2. **`crates/api/routes/src/lib.rs` lines 145-160** — the
   `governance::{...}` use block. Confirm the alphabetical-after-
   `request_appeal::request_appeal,` placement (req < rev, so
   `revoke_endorsement::revoke_endorsement,` lands AFTER
   `request_appeal::`).
3. **`crates/api/routes/src/lib.rs` lines 519-530** — the
   `/governance` scope's `.route(...)` chain. Confirm
   `.route("/endorsement", post().to(create_endorsement))` exists
   (~line 523), and the new route inserts immediately after,
   before `/appeal`.
4. **`crates/api/api_crud/src/governance/revoke_endorsement.rs`** —
   confirm the `revoke_endorsement` function symbol exists (re-
   exported from `revoke_endorsement` module per Task 2's
   `mod.rs` wire). The use-block import path
   `revoke_endorsement::revoke_endorsement,` resolves to the
   handler fn under
   `lemmy_api_crud::governance::revoke_endorsement::revoke_endorsement`.
5. **`.claude/lessons/feedback_clippy_test_style.md`** and
   **`.claude/lessons/feedback_advisor_phase_branch_push_skips_workspace_check.md`**
   — clippy lint discipline; the workspace-check workflow only
   triggers on `junior/*` push (not `phase-v1-*`), so this task's
   worker-branch push WILL fire the workflow correctly.

## 3a. Handover from prior cohort

**From Tasks 1+2 + fix-impl chain (sl-b-impl-1, sl-b-impl-2,
sl-b-fix-impl-1, advisor-direct fix-impl-2):** SHIPPED.

- Task 1 (`8e1bec153`): `RevokeEndorsement` DTO extended with
  `reason: String`; new `RevokeEndorsementResponse` exported from
  `crates/api/api_common/src/governance.rs`.
- Task 2 (`fa9d1181b`): handler at
  `crates/api/api_crud/src/governance/revoke_endorsement.rs`
  (317 lines); module wired in `governance/mod.rs`.
- fix-impl-1 (`37a2c99d3`): clippy::indexing-slicing fix on
  line 302 → `as_object_mut().insert(...)`.
- fix-impl-2 (advisor-direct, `f8d39dd71`):
  clippy::collapsible-if fix → let-chain
  `if bypass_recorded && let Some(obj) = ...`.
- DQ #145, #146, #147 all resolved (#147 pass on local cargo
  check + clippy).

**Phase tip:** `26c3fe6ec chore(decision-queue): close DQ #145
#146 #147 — fix chain landed clean`.

The `revoke_endorsement::revoke_endorsement` symbol (handler
fn) is exported from
`lemmy_api_crud::governance::revoke_endorsement` and is in
scope for the routes crate (which depends on api_crud).

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branched off `phase-v1-SL-b`
  (tip `26c3fe6ec`).
- `git branch --show-current` should return a
  `junior/role-impl-task-...` branch (adapt to your actual
  worktree branch name).
- Two commits at task end:
  1. Impl: `feat(v1-SL-b): register POST /api/v4/governance/endorsement/revoke route (task 3)`
  2. DQ: `chore(decision-queue): impl raised DQ #<next-id> — sl-b-impl-3 validate-pending`

### Implementation discipline — the EXACT edits

**Edit 1 (use-block addition).** Anchor on lines 149-152 in
`crates/api/routes/src/lib.rs`. Current text (verify by reading):

```rust
    create_endorsement::create_endorsement,
    create_report::create_report,
    request_appeal::request_appeal,
  },
```

Replace with:

```rust
    create_endorsement::create_endorsement,
    create_report::create_report,
    request_appeal::request_appeal,
    revoke_endorsement::revoke_endorsement,
  },
```

(Insert one new line after the `request_appeal::` line; same
4-space indentation; trailing comma; preserve the closing `},`.)

**Edit 2 (route registration).** Anchor on lines 521-524. Current
text (verify by reading; line numbers may shift if Edit 1 lands
first — re-grep `'/endorsement"'` to confirm):

```rust
          .wrap(rate_limit.post())
          .route("/report", post().to(create_report))
          .route("/endorsement", post().to(create_endorsement))
          .route("/appeal", post().to(request_appeal))
```

Replace with:

```rust
          .wrap(rate_limit.post())
          .route("/report", post().to(create_report))
          .route("/endorsement", post().to(create_endorsement))
          .route("/endorsement/revoke", post().to(revoke_endorsement))
          .route("/appeal", post().to(request_appeal))
```

(Insert one new `.route(...)` line between `/endorsement` and
`/appeal`; preserve all other lines and indentation exactly.)

**Per plan §13 Task 3 GOTCHA (route line shift):** if `grep -n
'"/endorsement"' lib.rs` returns a line number ≠ 523, document
the actual line in the impl commit body. Edit 2's anchor is
the matched line, not a hard-coded number.

**Per plan §13 Task 3 GOTCHA (rate-limit middleware
inheritance):** the new route is inside the `/governance` scope
which wraps `rate_limit.post()` (line 521 above). The
`revoke_endorsement` handler inherits the post-rate-limit
middleware automatically. No explicit middleware wiring needed
on the new line.

### Hard refusals

- Do NOT modify any file other than the two named (lib.rs + DQ
  json).
- Do NOT change indentation of unrelated lines.
- Do NOT reorder existing use-block entries or route entries.
- Do NOT add a `.wrap(...)` to the new route (rate-limit is
  inherited from scope).
- Do NOT comment-out or remove the `request_appeal::` line —
  the new line is INSERTED AFTER it.
- Do NOT add new tests in this task (Tasks 4-12 ship them).
- Do NOT push the worker branch BEFORE both commits land.
- Do NOT mutate or resolve any existing DQ entry. Raise a new
  one for the workspace-check.

### Validate-pending DQ entry shape (Shape G Layer G2)

```json
{
  "id": <next-int>,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<NOW_ISO>",
  "subject": "sl-b-impl-3 workspace-check validate-pending",
  "branch": "junior/<task-slug>",
  "phase_task": "sl-b-impl-3",
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

If `gh run list` returns empty, wait up to 60 seconds and retry.

## 5. Acceptance

Task 3 passes if:

- `crates/api/routes/src/lib.rs` contains exactly the two new
  lines (use-block import + route registration).
- `git diff HEAD~1 -- crates/api/routes/src/lib.rs` shows
  exactly TWO `+` lines and ZERO `-` lines (pure additions, no
  modifications to existing code).
- One impl commit + one DQ-entry commit on the worker branch.
- DQ entry written with `kind: "validate-pending"`,
  `from: "impl"`, populated `workflow_run_id`.
- ci-watcher's later poll returns `conclusion: "success"` (no
  cargo check / clippy / test --no-run failures).
