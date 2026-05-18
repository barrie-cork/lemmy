---
phase: v1-federation-inbound-a
role: impl-task
kind: fix-impl
fix_impl_n: 1
authored: 2026-05-18
plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md
triggering_dq: 249
triggering_task: 7
classification: "NON-ALLOWLIST clippy::allow_attributes — user-authorised one-off fix-impl (allowlist NOT permanently broadened mid-phase; retro proposes the row). Per /auto-phase §G4 + user AskUserQuestion 2026-05-18."
base: "phase-v1-federation-inbound-a @ d19d97a5b (Task 7 merged tip)"
cap: "1 file edit (federation_inbox_nonce.rs ONLY)"
serial: "Cohort B strictly serial cap=1 — this fix-impl is the in-flight Task 7 §5.2 recovery; Task 8 dispatched only after Task 7 §5.2 result:pass"
---

# [role:impl-task] v1-federation-inbound-a fix-impl-1 — Task 7 clippy `allow_attributes` → `expect` — see .claude/PRPs/briefs/federation-inbound-a-fix-impl-1.md

> **Provenance:** Task 7 (#313) `validate-pending-laptop` DQ #249 CMD2 (`cargo-clippy --workspace --features full --no-deps -- -D warnings`) FAILED with exactly ONE lint error (CMD1 cargo-check PASSED clean). The Lemmy workspace `[workspace.lints.clippy]` denies `allow_attributes`, so the worker's `#[allow(dead_code)]` escape hatch is rejected. This is a single-line, single-file mechanical fix with clippy's own suggested replacement + an exact in-repo precedent. **Classification: NON-allowlist** (no `clippy::allow_attributes` row in the §G4 allowlist) — dispatched as a **user-authorised one-off** (user AskUserQuestion 2026-05-18; allowlist intentionally NOT broadened mid-phase, retro will propose adding the row).

## §0 Pre-flight (subagent runs this FIRST, before reading anything else)

- Confirm CWD branch is a Junior worktree branched off `phase-v1-federation-inbound-a` (base tip `d19d97a5b` — Task 7 merged). `git merge-base --is-ancestor d19d97a5b HEAD` MUST be true. If on `phase-v1-federation-inbound-a` itself or a non-`junior/*` branch → STOP, file `kind: "blocker"` DQ.
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — if inside a forbidden window exit non-zero `FORBIDDEN_WINDOW: <window>`, UNLESS the dispatch description carries `(user-authorised forbidden-window override per DQ #<id>)`. Shape G SUSPENDED (cargo on laptop) — window-cargo concern reduced; keep the check.
- Confirm the target line is present: `grep -n "allow(dead_code)" crates/db_schema/src/source/governance/federation_inbox_nonce.rs` MUST return the line at/near 34. If absent → STOP, file `kind: "blocker"` DQ (base mismatch — Task 7 not on this worktree's base).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-a fix-impl-1 — Task 7 clippy allow_attributes → expect`

Actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-a fix-impl-1 — see .claude/PRPs/briefs/federation-inbound-a-fix-impl-1.md
```

## §2 Scope

### 2.1 The exact failure (verbatim clippy output — the contract)

```
error: #[allow] attribute found
  --> crates\db_schema\src\source\governance\federation_inbox_nonce.rs:34:3
   |
34 | #[allow(dead_code)] // TODO(v1-federation-inbound-b): wired by replay-cleanup cron
   |   ^^^^^ help: replace it with: `expect`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.95.0/index.html#allow_attributes
   = note: requested on the command line with `-D clippy::allow-attributes`
```

### 2.2 The exact fix (the contract — implement THIS, do not paraphrase)

In `crates/db_schema/src/source/governance/federation_inbox_nonce.rs`, the single line currently reading:

```rust
#[allow(dead_code)] // TODO(v1-federation-inbound-b): wired by replay-cleanup cron
```

becomes:

```rust
#[expect(dead_code, reason = "wired by replay-cleanup cron in v1-federation-inbound-b")]
```

Rationale (cite in commit body): the Lemmy workspace clippy config sets `allow_attributes = deny`, so `#[allow(...)]` is rejected but `#[expect(...)]` is the sanctioned escape hatch. This matches:
- **Lesson `feedback_clippy_test_style.md`** Pattern 3 ("`#[expect(...)]` is allowed because `expect` attributes are different from `allow` attributes under `allow_attributes = deny`").
- **In-repo precedent** `crates/db_schema/src/source/governance/redaction.rs:38` (`#[expect(clippy::expect_used, reason = "static regex — infallible at startup")]`).

The original `// TODO(v1-federation-inbound-b): wired by replay-cleanup cron` intent is preserved inside the `reason = "..."` string (the `#[expect]` attribute's reason carries the same semantic; no separate trailing comment needed — a trailing `//` comment after the attribute is fine too if you prefer, but the `reason` string is the canonical form here).

**Boundaries:**
- Edit **ONLY** `crates/db_schema/src/source/governance/federation_inbox_nonce.rs`. Cap: **1 file**.
- Do **NOT** touch `remote_moderation_label.rs` (verified clippy-clean), `mod.rs`, `schema.rs`, any other crate, any `.claude/**` file (except the DQ raise per §4), any test, any migration.
- Do **NOT** add/remove/modify any other line in `federation_inbox_nonce.rs` — change ONLY the `#[allow(dead_code)]` attribute line. No reformat, no reorder, no import change.
- Do **NOT** `#[allow]`-spam elsewhere or suppress with a broader attribute. The fix is the single attribute swap above.

## §3 Required reading

- `.claude/lessons/feedback_clippy_test_style.md` — Pattern 3 (`#[expect]` is the legal escape hatch under `allow_attributes = deny`). **The governing lesson.**
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` — after the edit, you do NOT re-run clippy in-worker (Shape-G suspended; the advisor re-runs §5.2 on the laptop after finalize-merge). But heed the lesson's principle: the fix must actually resolve the lint, not mask it.
- `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` — pre-push local cargo-check discipline (see §4 Constraint).
- `crates/db_schema/src/source/governance/redaction.rs` lines 36-56 — the in-repo `#[expect(..., reason = "...")]` precedent to mirror verbatim in form.
- The triggering entry: `.claude/decision-queue.json` DQ #249 (Task 7 validate-pending-laptop, `from: "impl"`, the one you will NOT mutate — the advisor mutates it after re-validation).

## §4 Constraints

1. **One commit.** Subject: `fix(v1-federation-inbound-a): clippy allow_attributes → expect on federation_inbox_nonce delete_older_than (fix-impl 1)`. Commit body cites: triggering DQ #249, lesson `feedback_clippy_test_style.md` Pattern 3, precedent `redaction.rs:38`, and that this is a user-authorised one-off non-allowlist §G4 fix.
2. **Pre-push cargo-check discipline** (per `feedback_fix_impl_pre_push_cargo_check.md`): BEFORE pushing the worker branch, run `cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"` locally in-worker. Exit 0 expected (the fix is attribute-only, cannot break compilation). Non-zero → the fix is wrong; do NOT push, file `kind: "blocker"` DQ #<next> citing the cargo-check failure. Do NOT `#[allow]`-spam to make it pass. (Note: clippy itself is re-run by the advisor on the laptop post-finalize-merge per Shape-G-suspended §5.2 — you only run cargo-check pre-push, not clippy.)
3. **DQ raise** (per `.claude/refs/dq-recipes.md` Recipe — and pin `ensure_ascii=False` when writing `.claude/decision-queue.json`, per the recurring intermittent ascii-escape breach `retro_carryforward` items): raise a fresh `kind: "validate-pending-laptop"` DQ entry, `from: "impl"`, `phase_task: 7`, for THIS fix commit, with the SAME two commands as DQ #249 (cargo-check.bat + cargo-clippy.bat -D warnings) — the advisor re-runs both on the laptop after finalize-merge to confirm the fix. Compute next id as `max(all ids across pending+resolved+archives)+1`. Commit + push the DQ raise in the SAME commit as the code fix (or a second commit on the same branch — either is fine; the worker branch carries both). Do NOT mutate or touch DQ #249 (the advisor owns its resolution).
4. **MIRROR-ref discipline:** the fix form is dictated by `redaction.rs:38` precedent + clippy's own `help: replace it with: expect`. Do not invent a different escape (no `#![allow]` at module/crate level, no `#[cfg_attr]` games, no deleting the function). The attribute swap is the entire change.
5. **Attribution:** `from: "impl"`, `answered_by: null` on the DQ raise. NEVER write `answered_by: "advisor"` / `"user"` / `kind: "clarify"` (per `.claude/rules/decision-queue.md` hard refusals).
6. **Serial discipline:** this fix-impl is the in-flight Task 7 §5.2 recovery under Cohort B serial cap=1. Do not dispatch or reference any other task. One worker, one fix, terminal.

## §5 Acceptance

- `crates/db_schema/src/source/governance/federation_inbox_nonce.rs` line ~34 reads `#[expect(dead_code, reason = "wired by replay-cleanup cron in v1-federation-inbound-b")]` (was `#[allow(dead_code)] // ...`).
- No other line in that file changed (verify with `git diff --stat` = 1 file, ~1-2 lines).
- `remote_moderation_label.rs`, `mod.rs`, all other files unchanged.
- Local `cargo-check.bat --workspace --features full` exit 0 (run pre-push per §4.2).
- A fresh `kind: "validate-pending-laptop"` DQ entry raised (`from: "impl"`, `phase_task: 7`, 2 commands matching DQ #249), committed + pushed on the worker branch.
- DQ #249 NOT touched.
