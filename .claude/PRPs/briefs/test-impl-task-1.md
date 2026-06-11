---
phase: test
role: impl-task
n: 1
authored: 2026-06-11
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-test
task_number: 1
---

# [role:impl-task] test Task 1 — sandbox_clamp helper + unconditional module declaration

## 1. Role + dispatch line

```
[role:impl-task] test task-1 sandbox_clamp impl — see .claude/PRPs/briefs/test-impl-task-1.md
```

## 2. Scope

### What to produce

1. Create `crates/utils/src/sandbox.rs` with the verbatim content from plan §10.1.
2. Add `pub mod sandbox;` to `crates/utils/src/lib.rs` immediately after `pub mod error;`.
3. Commit, push to origin worker branch, write a `kind: "validate-pending"` DQ entry for `cargo-validate-workspace.yml`, then **STOP** — do NOT run cargo locally.

### Explicit boundaries

- **Only 2 files**: `crates/utils/src/sandbox.rs` (create) + `crates/utils/src/lib.rs` (modify).
- **No other files.** Do NOT touch `crates/server/tests/e2e.rs`, migrations, governance paths, or any file outside §11 of the plan.
- **Shape G (Layer G2 push-and-exit):** after commit + push, write the `validate-pending` DQ entry and stop. Do NOT run `cargo check`, `cargo clippy`, or any cargo command locally.
- Do NOT write `validate-pending-laptop` — this is a Shape G plan; GH Actions runs cargo.

## 3. Required reading (MANDATORY — read before first edit)

- **Plan §9 + §10**: `crates/utils/src/lib.rs:1-15` (module-declaration pattern) and `crates/utils/src/error.rs:1-30` (sibling unconditional module — convention reference). Read BOTH before writing `sandbox.rs`.
- **Plan §10.1**: verbatim `sandbox_clamp` fn + `#[cfg(test)] mod tests` block — copy exactly; do not paraphrase.
- **Plan §10.2**: verbatim `pub mod sandbox;` placement — outside `cfg_select!` block, immediately after `pub mod error;`.
- **Plan §7 guardrails**: R-clippy-1 (no `as` casts), R-clippy-2 (`#[test]` inside `#[cfg(test)] mod tests`), R-clippy-3 (no `.unwrap()`/`.expect()`), R-mod-decl (unconditional placement).
- `.claude/lessons/feedback_read_canonical_before_writing_spec.md` — read sibling module first.
- `.claude/lessons/feedback_clippy_test_style.md` — test-style lint shape.
- `.claude/rules/decision-queue.md` §"`kind: validate-pending`" — the DQ entry shape you must write post-push.

## 4. Constraints

### Implementation

- `sandbox.rs` content MUST be verbatim from plan §10.1 — no changes, no simplifications.
- `pub mod sandbox;` placement: immediately after `pub mod error;` line in `lib.rs`, OUTSIDE the `cfg_select! { feature = "full" => { … } }` block. Verify with `rg -n 'pub mod sandbox' crates/utils/src/lib.rs` after edit.
- No `as` casts anywhere — use `value.min(max)`.
- The two `#[test]` fns MUST be inside `#[cfg(test)] mod tests { … }`, not at module scope.
- No `.unwrap()` / `.expect()` in any code you write.

### Shape G push-and-exit (MANDATORY)

After `git commit` + `git push origin <worker-branch>`:

1. Capture the `workflow_run_id` via:
   ```bash
   sleep 15  # give GH Actions time to register the push
   gh run list --repo barrie-cork/lemmy --branch <worker-branch> --limit 1 --json databaseId,status --jq '.[0]'
   ```
   If no run appears within 60s: set `workflow_run_id` to `0` and note "no GH Actions run triggered (possible credits issue)" in the DQ `context` field — still write the DQ entry.

2. Write a `kind: "validate-pending"` DQ entry via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. The fragment must have:
   ```json
   {
     "from": "impl",
     "kind": "validate-pending",
     "timestamp": "<ISO8601 UTC>",
     "question": "Did cargo-validate-workspace.yml pass on Task 1 worker branch?",
     "options": ["pass", "fail"],
     "context": "Task 1 sandbox_clamp impl pushed to <worker-branch>. workflow_run_id: <id>",
     "workflow_run_id": <id or 0>,
     "branch": "<worker-branch>",
     "phase_task": 1,
     "result": null,
     "log_slice": null,
     "failed_jobs": null,
     "answer": null,
     "answered_by": null,
     "resolved_at": null,
     "approved_by": null,
     "approved_at": null
   }
   ```
3. `git add .claude/decision-queue.json && git commit -m "chore(decision-queue): impl raised validate-pending DQ — test task 1" && git push origin <worker-branch>`

4. **STOP.** Output "SHAPE-G EXIT: validate-pending DQ written, awaiting ci-watcher". Do NOT run cargo.

### DQ attribution

- `answered_by` on any DQ entry: `"impl-self-resolved"` or `null` — NEVER `"advisor"` / `"user"`.
- Use `bash scripts/brehon/dq-v3-new-entry.sh` to generate the composite id.

### Commit message

```
feat(test): sandbox_clamp dogfood helper in lemmy_utils (task 1)
```

### LESSON: trailer

End the commit body with:
```
LESSON: sandbox_clamp dogfood — Shape G push-and-exit; validate-pending DQ raised; ci-watcher polls cargo-validate-workspace.yml
```

## 5. Known harness notes

- **GH Actions credits may be low** (per advisor note 2026-06-11): if `gh run list` returns empty after 60s wait, set `workflow_run_id: 0` in the DQ entry and note it. The ci-watcher will classify `run_not_found`; advisor will handle via §G4. Do NOT block on this.
- **`dq-v3-new-entry.sh`**: generates composite id `<session_id>-<seq>`. Use it; never compute `max(all_ids)+1` (Hard refusal #9).
- **`dq-v3-append-fragment.sh`**: write the JSON fragment to a temp file first, then pass the path. Avoid inline heredoc — backslash paths mangle on Windows workers (per `feedback_windows_backslash_path_dq_via_write_fragment.md`). Write the fragment via the Write tool to `/tmp/test-task1-dq-fragment.json`, then call `bash scripts/brehon/dq-v3-append-fragment.sh /tmp/test-task1-dq-fragment.json --pending`.
