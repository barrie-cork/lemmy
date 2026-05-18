---
phase: v1-federation-inbound-a
role: impl-task
kind: fix-impl
fix_impl_n: 2
authored: 2026-05-18
plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md
triggering_dq: 249
supersedes_fix_impl: 1
blocker_resolved: 250
triggering_task: 7
classification: "NON-ALLOWLIST clippy::allow_attributes — user-authorised CORRECTED recipe after fix-impl-1's recipe was found DEFECTIVE (DQ #250). fix-impl-1 prescribed #[allow]->#[expect]; that fails CMD2 differently because delete_older_than is a pub fn in a lib crate (dead_code never fires => unfulfilled_lint_expectations => -D warnings error). User selected option-a (remove the attribute entirely) via advisor AskUserQuestion catch-fire surface 2026-05-18. Allowlist NOT broadened mid-phase; retro proposes the row + this recipe-correctness lesson."
base: "phase-v1-federation-inbound-a @ d67f34397 (DQ #250 resolved tip; lane==origin==daemon-local synced)"
cap: "1 file edit (federation_inbox_nonce.rs ONLY)"
serial: "Cohort B strictly serial cap=1 — this fix-impl is the in-flight Task 7 §5.2 recovery (supersedes fix-impl-1); Task 8 dispatched only after Task 7 §5.2 result:pass on the fixed tip"
---

# [role:impl-task] v1-federation-inbound-a fix-impl-2 — Task 7 clippy `allow_attributes`: REMOVE the attribute (option-a) — see .claude/PRPs/briefs/federation-inbound-a-fix-impl-2.md

> **Provenance:** Supersedes fix-impl-1 (#315), whose prescribed recipe (`#[allow(dead_code)]` → `#[expect(dead_code)]`) was **defective**. fix-impl-1 correctly refused to push it (per its §4.2 pre-push-cargo-check constraint) and filed `kind: "blocker"` DQ #250. **Why the old recipe was wrong:** `delete_older_than` is a `pub async fn` in **library crate** `lemmy_db_schema`. Rust does **not** emit the `dead_code` lint for `pub` items in a lib crate (they are public API surface). So `#[expect(dead_code)]` would be an *unfulfilled expectation* → `unfulfilled_lint_expectations` warning (on by default) → with `-D warnings` in CMD2, a **hard error**. The fix would merely trade one clippy error for another. **The user selected option-a** (advisor AskUserQuestion catch-fire surface, 2026-05-18; DQ #250 resolved `answered_by: "user"`): **remove the `#[allow(dead_code)]` attribute entirely, keeping only the line comment.** A `pub fn` never trips `dead_code`, so **no suppression attribute is needed at all** — this eliminates the `clippy::allow_attributes` violation without introducing `unfulfilled_lint_expectations`. **Classification: NON-allowlist**, user-authorised corrected one-off (allowlist intentionally NOT broadened mid-phase; retro proposes both the row and the recipe-correctness lesson).

## §0 Pre-flight (subagent runs this FIRST, before reading anything else)

- Confirm CWD branch is a Junior worktree branched off `phase-v1-federation-inbound-a` (base tip `d67f34397` — DQ #250 resolved). `git merge-base --is-ancestor d67f34397 HEAD` MUST be true. If on `phase-v1-federation-inbound-a` itself or a non-`junior/*` branch → STOP, file `kind: "blocker"` DQ.
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — if inside a forbidden window exit non-zero `FORBIDDEN_WINDOW: <window>`, UNLESS the dispatch description carries `(user-authorised forbidden-window override per DQ #<id>)`. Shape G SUSPENDED (cargo on laptop) — window-cargo concern reduced; keep the check.
- **Submodule init (MANDATORY before any in-worker cargo — per `feedback_worktree_submodules_not_auto_init`):** run `git submodule update --init 2>&1` from the worktree root. The `crates/email/translations` git submodule is NOT auto-initialized in a fresh worktree; without it, `cargo-check` fails pre-existing (ENOENT on `translations/backend/`) — this is the same false-negative class that hit Cohort-A Task-5 / DQ #236 and that fix-impl-1 #315 surfaced. This is infra, NOT part of the fix; initialize it so the §4.2 pre-push cargo-check is meaningful.
- Confirm the target line is present: `grep -n "allow(dead_code)" crates/db_schema/src/source/governance/federation_inbox_nonce.rs` MUST return the line at/near 34. If absent → STOP, file `kind: "blocker"` DQ (base mismatch — Task 7 not on this worktree's base, or fix-impl-1 partially applied).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-a fix-impl-2 — Task 7 clippy allow_attributes: REMOVE attribute (option-a)`

Actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-a fix-impl-2 — see .claude/PRPs/briefs/federation-inbound-a-fix-impl-2.md
```

## §2 Scope

### 2.1 The failure being fixed (verbatim clippy output — the contract)

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

> **NOTE on clippy's `help: replace it with: expect`:** clippy's mechanical suggestion is **wrong for this site** and MUST NOT be followed. `#[expect(dead_code)]` produces `unfulfilled_lint_expectations` here because `delete_older_than` is a `pub fn` in a lib crate (`dead_code` never fires on it). This is exactly why fix-impl-1 was blocked (DQ #250). The correct fix is **removal**, per the user decision below — do NOT apply `#[expect]`.

### 2.2 The exact fix (the contract — implement THIS, do not paraphrase, do not follow clippy's `expect` suggestion)

In `crates/db_schema/src/source/governance/federation_inbox_nonce.rs`, the single attribute line currently reading (at/near line 34):

```rust
#[allow(dead_code)] // TODO(v1-federation-inbound-b): wired by replay-cleanup cron
```

becomes (attribute removed, the `// TODO` comment preserved on its own line):

```rust
// TODO(v1-federation-inbound-b): wired by replay-cleanup cron
```

That is: **delete the `#[allow(dead_code)]` attribute entirely**; keep the `// TODO(v1-federation-inbound-b): wired by replay-cleanup cron` text as a plain line comment immediately above `pub async fn delete_older_than`. Do **NOT** add `#[expect(...)]`, `#[allow(...)]`, `#[cfg_attr(...)]`, `#[used]`, `#![allow]`, or any other attribute. The function body, signature, `#[cfg(feature = "full")]` line above it, and all other lines stay byte-identical.

Rationale (cite in commit body): `delete_older_than` is a `pub async fn` in **library crate** `lemmy_db_schema`. Rust does not emit the `dead_code` lint for `pub` items in a lib crate (public API surface), so **no dead-code suppression attribute is needed**. Removing `#[allow(dead_code)]`:
- eliminates the `clippy::allow_attributes` violation (no `#[allow]` present), AND
- does **not** introduce `unfulfilled_lint_expectations` (no `#[expect]` present),

leaving CMD2 clippy `-D warnings` clean on this line. This is **option-a**, user-selected via advisor AskUserQuestion catch-fire surface 2026-05-18 (DQ #250 resolved, `answered_by: "user"`). Option-b (force `#[expect]` + relax shared `[workspace.lints]`) was rejected — it collides with the DQ #232 additive-only-shared-files constraint across 3 concurrent lanes.

**Boundaries:**
- Edit **ONLY** `crates/db_schema/src/source/governance/federation_inbox_nonce.rs`. Cap: **1 file**.
- Do **NOT** touch `remote_moderation_label.rs`, `mod.rs`, `schema.rs`, `Cargo.toml`, `[workspace.lints]`, any other crate, any test, any migration, any `.claude/**` file (except the DQ raise per §4).
- Do **NOT** add/remove/modify any other line in `federation_inbox_nonce.rs` — change ONLY the `#[allow(dead_code)]` attribute line (delete the attribute, keep the comment). No reformat, no reorder, no import change.
- Do **NOT** `#[allow]`/`#[expect]`-spam anywhere or suppress with any attribute. The fix is the single attribute **removal** above.

## §3 Required reading

- `.claude/lessons/feedback_clippy_test_style.md` — Pattern 3 governs `allow_attributes = deny`. **Important nuance this brief adds (retro will promote):** the legal escape-hatch under `allow_attributes = deny` is `#[expect(...)]` *only where the lint actually fires*. For an intentionally-unused **`pub` API fn in a lib crate**, `dead_code` never fires, so the correct fix is to **remove the suppression entirely**, NOT `#[expect(dead_code)]` (which then trips `unfulfilled_lint_expectations`). The `redaction.rs:38` precedent (`#[expect(clippy::expect_used)]` on a static regex) is a *firing* lint and does not generalize to this case.
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` — the fix must actually resolve the lint, not mask it. (You do NOT re-run clippy in-worker — Shape-G suspended; the advisor re-runs §5.2 on the laptop after finalize-merge. But heed the principle.)
- `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` — pre-push local cargo-check discipline (see §4 Constraint 2). This is what correctly caught fix-impl-1's defective recipe — honour it again.
- `.claude/lessons/feedback_worktree_submodules_not_auto_init.md` — why §0 mandates `git submodule update --init` before in-worker cargo-check (CMD1 false-negative class).
- The triggering entry: `.claude/decision-queue.json` DQ #249 (Task 7 validate-pending-laptop, `from: "impl"`, the one you will NOT mutate — the advisor mutates it after re-validation on the fixed tip).
- The resolved blocker: `.claude/decision-queue.json` DQ #250 (`answered_by: "user"`, option-a) — the authority for this corrected recipe. Read its `answer` field; it is the contract.

## §4 Constraints

1. **One commit.** Subject: `fix(v1-federation-inbound-a): remove #[allow(dead_code)] on federation_inbox_nonce delete_older_than (pub fn never trips dead_code) (fix-impl 2)`. Commit body cites: triggering DQ #249, resolved blocker DQ #250 (option-a, user-selected), why `#[expect]` was wrong here (pub-fn-in-lib-crate → `unfulfilled_lint_expectations`), and that this is a user-authorised corrected non-allowlist §G4 fix superseding fix-impl-1.
2. **Pre-push cargo-check discipline** (per `feedback_fix_impl_pre_push_cargo_check.md`): AFTER `git submodule update --init` (§0) and BEFORE pushing the worker branch, run `cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"` locally in-worker. Exit 0 expected (the fix is attribute-removal only, cannot break compilation; the submodule was just initialized so the prior CMD1 ENOENT is gone). Non-zero → STOP, do NOT push, file `kind: "blocker"` DQ #<next> citing the cargo-check failure verbatim (full last ~80 lines). Do NOT `#[allow]`/`#[expect]`-spam to make it pass. (Note: clippy itself is re-run by the advisor on the laptop post-finalize-merge per Shape-G-suspended §5.2 — you only run cargo-check pre-push, not clippy.)
3. **DQ raise** (per `.claude/refs/dq-recipes.md` Recipe — and **pin `ensure_ascii=False`** when writing `.claude/decision-queue.json`; the impl-task DQ-write ascii-escape breach has recurred 4x this phase [#283/#305/#308/#310 escaped; #307 canonical] — write canonical UTF-8, NOT `\uXXXX`-escaped, so the advisor does not have to whole-file re-encode on mutation): raise a fresh `kind: "validate-pending-laptop"` DQ entry, `from: "impl"`, `phase_task: 7`, for THIS fix commit, with the SAME two commands as DQ #249 (`scripts\brehon\cargo-check.bat --workspace --features full` + `scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings`) — the advisor re-runs both on the laptop after finalize-merge to confirm the fix. Compute next id as `max(all ids across pending+resolved+archives)+1` (current max is 250 → next is 251 unless a concurrent lane bumped it; recompute, do not hardcode). Commit + push the DQ raise in the SAME commit as the code fix (or a second commit on the same branch — either is fine; the worker branch carries both). Do **NOT** mutate or touch DQ #249 or DQ #250 (the advisor owns #249's resolution; #250 is already resolved).
4. **MIRROR-ref discipline:** the fix form is dictated by the **user decision in DQ #250 (option-a)** — *remove* the attribute. Do NOT follow clippy's `help: replace it with: expect` (that is the defective path that blocked fix-impl-1). Do NOT invent any other escape (`#![allow]`, `#[cfg_attr]`, deleting the function, `#[used]`). The single attribute **removal** is the entire change.
5. **Attribution:** `from: "impl"`, `answered_by: null` on the DQ raise. NEVER write `answered_by: "advisor"` / `"user"` / `kind: "clarify"` (per `.claude/rules/decision-queue.md` hard refusals).
6. **Serial discipline:** this fix-impl is the in-flight Task 7 §5.2 recovery under Cohort B serial cap=1, superseding fix-impl-1. Do not dispatch or reference any other task. One worker, one fix, terminal.

## §5 Acceptance

- `crates/db_schema/src/source/governance/federation_inbox_nonce.rs` at/near line 34: the `#[allow(dead_code)]` attribute is **removed**; the line now reads only `// TODO(v1-federation-inbound-b): wired by replay-cleanup cron` (a plain comment), immediately above `pub async fn delete_older_than`. No `#[allow]`, no `#[expect]`, no other attribute on that function.
- No other line in that file changed (verify with `git diff --stat` = 1 file; `git diff` shows only the attribute removal, ~1 line net).
- `remote_moderation_label.rs`, `mod.rs`, `Cargo.toml`, all other files unchanged.
- `git submodule update --init` ran in §0 (CMD1 cargo-check meaningful).
- Local `cargo-check.bat --workspace --features full` exit 0 (run pre-push per §4.2, after submodule init).
- A fresh `kind: "validate-pending-laptop"` DQ entry raised (`from: "impl"`, `phase_task: 7`, 2 commands matching DQ #249, `ensure_ascii=False` canonical), committed + pushed on the worker branch.
- DQ #249 NOT touched. DQ #250 NOT touched (already resolved).
