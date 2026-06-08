---
paths:
  - ".pi/**"
  - ".claude/PRPs/comparator/**"
---

# Pi harness constraints — brehon-fork planning

Six non-negotiable rules for any plan produced by a Pi-hosted model on this project.
These match the constraints the Opus/Claude-Code planning session loads automatically
from `.claude/lessons/` and `.claude/rules/`. They exist here so Pi-hosted challengers
get the same operational context without needing the full Claude-Code session harness.

**Read this file once at the start of every planning run. It is the ground truth for
how impl workers execute plans on this project — a plan that violates these rules will
fail at impl time, not at plan-review time.**

---

## 1. No cargo on the daemon — validate-pending-laptop pattern

**All cargo commands run on the laptop (64 GB), NOT the EliteDesk daemon.**

The daemon has insufficient RAM for concurrent cargo. Multiple workers running cargo
simultaneously caused an OOM incident (load 39+, swap exhausted 0B). Workers that run
cargo themselves bypass the gate and can trigger a repeat.

Every §13 task that needs a cargo validation MUST:
1. Write a `validate-pending-laptop` DQ entry with the commands
2. Commit + push the DQ entry
3. **STOP** — do not execute the cargo command

**DQ entry shape** (copy this into each VALIDATE block):
```json
{
  "kind": "validate-pending-laptop",
  "commands": ["./scripts/brehon/cargo-check.sh --workspace --features full"],
  "branch": "<phase-branch>",
  "phase_task": <N>
}
```

For bridge-only tasks (`cd services/bridge && cargo check`), use that command in the
`commands` array instead.

§15 validation commands must spell out the DQ entry shape, not bare `cargo check`.
Source: `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md`

---

## 2. Cargo wrappers — never bare cargo on Windows paths

All workspace cargo invocations use the wrapper scripts in `scripts/brehon/`:

| Goal | Command |
|---|---|
| Check workspace | `./scripts/brehon/cargo-check.sh --workspace --features full` |
| Check single crate | `./scripts/brehon/cargo-check.sh -p <crate>` |
| Run e2e tests | `./scripts/brehon/cargo-test.bat --workspace --test e2e` (Windows `.bat`) |
| Migration runner | `cargo run -p lemmy_diesel_utils --features full` |

**`--features full` + `-p <crate>` together is a footgun.** `--features full` is a
workspace-level feature; it silently does nothing when combined with `-p`. Use
`--workspace --features full` for the full build; `-p <crate>` alone for per-crate checks.

Source: `.claude/lessons/feedback_features_full_workspace_only.md`,
`.claude/lessons/feedback_features_full_p_crate_incompatible.md`

---

## 3. Migration runner — never `diesel migration run` directly

Use:
```bash
cargo run -p lemmy_diesel_utils --features full
```
No sub-command arguments. `main.rs` handles migration execution. `diesel migration run`
bypasses the custom runner and fails.

Source: `.claude/lessons/feedback_lemmy_migration_runner.md`

---

## 4. e2e test return-type discipline

New e2e test functions in `crates/server/tests/e2e.rs` must return `LemmyResult<()>`,
NOT `Result<(), Box<dyn Error>>`. `LemmyError` does not implement `std::error::Error` —
using `Box<dyn Error>` causes **E0277** at compile time.

Correct pattern:
```rust
async fn test_name(pool: &DbPool) -> LemmyResult<()> {
    let result = some_op().await.map_err(|e| LemmyError::from(e))?;
    // ...
    Ok(())
}
```

DB connections in tests: `AsyncPgConnection::establish(&db_url).await?`
Source: `.claude/lessons/feedback_lemmy_error_no_std_error.md`,
`.claude/lessons/feedback_async_pool_test_pattern.md`

Before editing `e2e.rs`, pre-locate every `old_string` anchor:
```bash
grep -c '<your-anchor-text>' crates/server/tests/e2e.rs
# must return 1 — if > 1, revise the anchor to be unique
```
Source: `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md`

---

## 5. ADR-015 callsite requirement (load-bearing)

Any task touching a governance write path that identifies a subject (user/actor) MUST:

1. **Name ADR-015 explicitly** in the task body AND in the DoD line — not just in narrative.
2. **Cite the concrete callsite**: `actor_pseudonym::get_or_create(person_id, conn)`.
   Never write `person.name`, `local_user.email`, or raw `person_id` into the governance
   log or any outbound payload.
3. **Include a grep-verifiable DoD check**:
   ```
   DoD: grep actor_pseudonym <handler_file> returns both the definition AND a callsite
   in the write path
   ```

A plan that names ADR-015 in prose without a callsite in the DoD will produce an impl
that drops the constraint silently — this has happened before.

Source: `.claude/lessons/feedback_cheap_model_arm_drops_adr_constraints.md`

---

## 6. Canonical plan section numbering (§1–§20)

Every plan MUST use this exact numbered heading scheme. The downstream DoD smoke-test,
verify-gate, and impl tooling index sections by number. Unnumbered headings like
`## Summary` or `## Solution Statement` break the automation.

```
## 1. Summary
## 2. Source
## 3. Goal
## 4. Solution statement        ← watchpoints go here as named sub-sections
## 5. Metadata                  ← includes §5.1 complexity factor + §5.2 per-task ceiling
## 6. Out of scope
## 7. Open questions to escalate
## 8. Design decisions
## 9. Mandatory reading (for impl agent)
## 10. Patterns to mirror
## 11. Flow diagram
## 12. Files to change
## 13. Step-by-step tasks       ← Task 0 is always preflight; one commit per task
## 14. NOT building
## 15. Validation commands (DoD) ← validate-pending-laptop DQ entries here, not bare cargo
## 16a. Stories                 ← user-story + checkpoint command per story
## 17. Risks
## 18. Dependencies
## 19. Split-DQ (if §5.1 complexity score > 8)
## 20. Confidence score
```

§4 watchpoints must cite specific table names, file paths, or `schema.rs:line` — not
concepts. "Watch for type drift" without naming the type is a process miss.

§16a stories need a machine-runnable checkpoint command so `/brehon-verify` can confirm
each story mechanically.

**MIRROR reference — read before writing your plan:**
`.claude/PRPs/plans/m2-late.plan.md` is the most recently shipped plan. Use it as the
canonical shape reference. Check the actual section count and heading style before writing.

---

## Quick self-check before finalising the plan

- [ ] Every §13 task VALIDATE line specifies a DQ entry shape, not a bare `cargo` command
- [ ] No `-p <crate> --features full` combinations in any command
- [ ] Every ADR-015-touching task has a grep-verifiable callsite in its DoD
- [ ] All top-level headings use `## N.` numbering (spot-check §1, §13, §15, §16a)
- [ ] §15 references `validate-pending-laptop` DQ, not direct cargo execution
- [ ] §4 watchpoints cite file:line or table name (not abstract concepts)
