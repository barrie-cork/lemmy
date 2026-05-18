---
phase: v1-federation-inbound-a
role: impl-task
kind: fix-impl
fix_impl_n: 3
authored: 2026-05-18
plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md
triggering_dq: 264
blocker_resolved: 265
triggering_task: 8
classification: "PLAN-SHAPE blocker resolved option-a by user. The Task-8 brief PROHIBITED editing e2e.rs on the FALSE Rust premise 'Option-able §10.3 fields keep e2e.rs:7451 compiling without a Task-8 e2e edit'. Option<T> does NOT make a struct-literal field optional (feedback_insertform_default_propagation.md documents this exact failure; it was a mandatory §3 lesson in the Task-8 brief — the planning error was asserting a scope boundary the cited lesson refutes). Task 8 (#318) correctly fixed the 2 in-scope inbox.rs callsites with ..Default::default(), did NOT touch out-of-scope e2e.rs, and raised a kind:blocker (DQ #265, transcribed from worker root-file TASK8_ESCALATION.md; worker pre-picked id 253 COLLIDED with sibling lane v1-ship-1 so advisor re-assigned 265). User selected option-a via advisor AskUserQuestion catch-fire surface 2026-05-18: authorize a narrow 1-line ..Default::default() fix at e2e.rs:7451. Advisor planning error — retro carry-forward (same lesson-class as fix-impl-1's #[expect(dead_code)] defect)."
base: "phase-v1-federation-inbound-a @ cf7fa1bdc (Task 8 impl 998fef033 + DQ #264/#265 transcription + root-debris removal; lane==origin==daemon-local synced)"
cap: "1 file edit (crates/server/tests/e2e.rs ONLY) — 1-line ..Default::default() insertion at a known anchor (line 7456→7457)"
serial: "Cohort B strictly serial cap=1 — this fix-impl completes Task 8's compile chain so its §5.2 cmd 3 (cargo-test --test e2e --no-run) can pass. Task 8 §5.2 (DQ #264) is run by the advisor on the laptop AFTER this fix lands + finalize-merge; Cohort B barrier reaches 3/3 only on DQ #264 result:pass."
---

# [role:impl-task] v1-federation-inbound-a fix-impl-3 — Task-8 e2e.rs:7451 add `..Default::default()` (option-a) — see .claude/PRPs/briefs/federation-inbound-a-fix-impl-3.md

> **Provenance:** Task 8 (#318) extended `FederationAttestationInsertForm` with 6 new `Option<_>` fields (plan §10.3). The Task-8 brief **prohibited Task 8 editing `crates/server/tests/e2e.rs`** (it is Task 9's file; `feedback_junior_worker_e2e_edit_hang`) on the stated premise that "Option-able §10.3 fields keep e2e.rs:7451 compiling without a Task-8 e2e edit". **That premise is false Rust:** `Option<T>` does NOT make a struct-literal field optional — a struct literal must specify all fields OR use `..Default::default()`. The callsite at `crates/server/tests/e2e.rs:7451` initializes `FederationAttestationInsertForm` with the **5 original fields and NO `..Default::default()`**, so after Task 8's 6 new fields it is `error[E0063]: missing fields source_instance, received_at, peer_trust_level_at_receipt, admin_reviewed_at, admin_action, dismissal_rationale in initializer of FederationAttestationInsertForm`. `feedback_insertform_default_propagation.md` documents this **exact** failure and was a **mandatory §3 lesson in the Task-8 brief** — the planning error was asserting a scope boundary the cited lesson directly contradicts. Task 8 (#318) did everything right: fixed the 2 in-scope `inbox.rs` callsites (138, 213) with `..Default::default()` per the lesson, did **not** touch out-of-scope `e2e.rs`, and raised a precise `kind: "blocker"` (DQ #265). **The user selected option-a** (advisor AskUserQuestion catch-fire surface, 2026-05-18; DQ #265 resolved `answered_by: "user"`): **authorize a narrow 1-line `..Default::default()` insertion at `e2e.rs:7451`.** `feedback_insertform_default_propagation.md` explicitly prescribes adding the spread to **all** caller sites; `FederationAttestationInsertForm` derives `#[derive(Clone, Default)]` so the spread compiles cleanly. Classification: **plan-shape blocker, user-authorised narrow fix** (advisor planning-error correction; retro carries the lesson).

## §0 Pre-flight (subagent runs this FIRST, before reading anything else)

- Confirm CWD branch is a Junior worktree branched off `phase-v1-federation-inbound-a` (base tip `cf7fa1bdc`). `git merge-base --is-ancestor cf7fa1bdc HEAD` MUST be true. If on `phase-v1-federation-inbound-a` itself or a non-`junior/*` branch → STOP, file `kind: "blocker"` DQ **into `.claude/decision-queue.json`** (NOT a repo-root file — see §4 Constraint 3).
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — if inside a forbidden window exit non-zero `FORBIDDEN_WINDOW: <window>`, UNLESS the dispatch description carries `(user-authorised forbidden-window override per DQ #<id>)`. Shape G SUSPENDED (cargo on laptop) — window-cargo concern reduced; keep the check.
- **Submodule init (MANDATORY before any in-worker cargo — per `feedback_worktree_submodules_not_auto_init`):** run `git submodule update --init 2>&1` from the worktree root. The `crates/email/translations` git submodule is NOT auto-initialized in a fresh worktree; without it `cargo-check` fails pre-existing (ENOENT on `translations/backend/`) — the same false-negative class that hit Cohort-A Task-5 / DQ #236 and fix-impl-1 #315. Infra, NOT part of the fix; initialize it so the §4.2 pre-push cargo-check is meaningful.
- Confirm the target callsite is present and unmodified: `grep -n "FederationAttestationInsertForm {" crates/server/tests/e2e.rs` MUST return line **7451**; `sed -n '7451,7457p' crates/server/tests/e2e.rs` MUST show the 5-field literal (`actor_url` / `subject_url` / `attestation_type` / `valid_until` / `signature`) with **NO** `..Default::default()` and the closing `})` at line 7457. If the line numbers differ or the literal already has `..Default::default()` → STOP, file `kind: "blocker"` DQ **into `.claude/decision-queue.json`** (base mismatch — Task 8 not on this worktree's base, or a concurrent edit moved it).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-a fix-impl-3 — Task-8 e2e.rs:7451 add ..Default::default() (option-a)`

Actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-a fix-impl-3 — see .claude/PRPs/briefs/federation-inbound-a-fix-impl-3.md
```

## §2 Scope

### 2.1 The failure being fixed (the contract)

After Task 8 (`998fef033`) added 6 new fields to `FederationAttestationInsertForm`, `cargo-test --workspace --features full --test e2e --no-run` (Task 8 §5.2 command 3) fails:

```
error[E0063]: missing fields `source_instance`, `received_at`, `peer_trust_level_at_receipt` and 3 other fields in initializer of `FederationAttestationInsertForm`
   --> crates/server/tests/e2e.rs:7451
    |
7451|    .values(&FederationAttestationInsertForm {
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `source_instance`, `received_at`, `peer_trust_level_at_receipt`, `admin_reviewed_at`, `admin_action` and `dismissal_rationale`
```

`FederationAttestationInsertForm` derives `#[derive(Clone, Default)]` (model file `crates/db_schema/src/source/governance/federation_attestation.rs:37`) and all 6 new fields are `Option<_>`, so the canonical fix per `feedback_insertform_default_propagation.md` is to add `..Default::default()` to this callsite (exactly as Task 8 already did to the 2 `inbox.rs` callsites in-scope).

### 2.2 The exact fix (the contract — implement THIS, do not paraphrase)

In `crates/server/tests/e2e.rs`, the callsite currently reads (lines 7451–7457):

```rust
    .values(&FederationAttestationInsertForm {
      actor_url: "https://test.invalid/u/seed-actor".to_string(),
      subject_url: "https://test.invalid/u/seed-subject".to_string(),
      attestation_type: AttestationType::TrustedReporter,
      valid_until: Some(future),
      signature: "seed-sig".to_string(),
    })
```

becomes (a single `..Default::default()` line inserted after the `signature:` line, before the closing `}`):

```rust
    .values(&FederationAttestationInsertForm {
      actor_url: "https://test.invalid/u/seed-actor".to_string(),
      subject_url: "https://test.invalid/u/seed-subject".to_string(),
      attestation_type: AttestationType::TrustedReporter,
      valid_until: Some(future),
      signature: "seed-sig".to_string(),
      ..Default::default()
    })
```

That is: **insert exactly one line `      ..Default::default()` (6-space indent to match the sibling fields' brace level) between the `signature: "seed-sig".to_string(),` line (7456) and the closing `})` line (7457).** Use an **anchor-Edit** keyed on the unique 2-line context (`signature: "seed-sig".to_string(),\n    })`) — do **NOT** rewrite, reformat, or re-read the whole 8900-line `e2e.rs` file (per `feedback_junior_worker_e2e_edit_hang`: large e2e.rs Edits hang the worker; this is a 1-line insertion at a unique anchor — anchor-Edit is safe, full-file ops are NOT).

**Boundaries:**
- Edit **ONLY** `crates/server/tests/e2e.rs`. Cap: **1 file**.
- Change **ONLY** the one callsite at line 7451 (the `FederationAttestationInsertForm` literal with `seed-actor`/`seed-sig`). Insert exactly the single `..Default::default()` line. Do **NOT** touch any other `InsertForm` callsite, any other test, any `use`, any other line. No reformat, no reorder, no import change. `git diff` must show **exactly one inserted line**, net +1.
- Do **NOT** touch `crates/db_schema/**`, `crates/apub/**`, `inbox.rs`, the model file, `schema.rs`, `Cargo.toml`, any migration, any `.claude/**` file (except the DQ raise per §4, written **into `.claude/decision-queue.json`**).
- Do **NOT** add any attribute, `#[allow]`, `#[expect]`, or any other field value. The fix is the single `..Default::default()` line insertion above.

## §3 Required reading

- `.claude/lessons/feedback_insertform_default_propagation.md` — **the load-bearing lesson.** `Option<T>` does NOT make a struct-literal field optional; a struct literal needs all fields OR `..Default::default()`. The lesson prescribes adding the `..Default::default()` spread to **every** caller site when an InsertForm gains fields. This callsite (`e2e.rs:7451`) is the one Task 8 could not touch (out-of-scope per its brief); this fix-impl completes the propagation the lesson requires.
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **why this is an anchor-Edit, not a full-file rewrite.** `crates/server/tests/e2e.rs` is ~8900 lines; a worker that Reads/rewrites the whole file (or does a multi-hunk Edit) hangs (SIGTERM). This fix is a **single-line insertion at a unique 2-line anchor** — use `Edit` with `old_string` = the exact `signature: "seed-sig".to_string(),\n    })` 2-line context and `new_string` = same with `..Default::default()` inserted. Do NOT Read the whole file; `sed -n '7448,7460p'` is enough to confirm the anchor pre-Edit.
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — e2e.rs error-shape discipline. **No code-shape change here** (you only add a struct-spread to an existing `.values(&...)` call inside an already-`?`-using async test fn); the test fn's `Result`/`?` shape is untouched. Read so you do NOT accidentally "fix" anything else in the file — the boundary is the single inserted line.
- `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` — pre-push local cargo-check discipline (see §4 Constraint 2). This discipline caught fix-impl-1's defective recipe; honour it again.
- `.claude/lessons/feedback_worktree_submodules_not_auto_init.md` — why §0 mandates `git submodule update --init` before in-worker cargo-check (CMD1 false-negative class).
- The triggering entry: `.claude/decision-queue.json` DQ #264 (Task 8 validate-pending-laptop, `from: "impl"`, the one you will NOT mutate — the advisor mutates it after re-validation on the fixed tip).
- The resolved blocker: `.claude/decision-queue.json` DQ #265 (`answered_by: "user"`, option-a) — the authority for this fix. Read its `answer` field; it is the contract.

## §4 Constraints

1. **One commit.** Subject: `fix(v1-federation-inbound-a): add ..Default::default() to e2e.rs:7451 FederationAttestationInsertForm callsite (Task-8 §10.3 propagation) (fix-impl 3)`. Commit body cites: triggering DQ #264, resolved blocker DQ #265 (option-a, user-selected), that the Task-8 brief's e2e.rs prohibition was an advisor planning error (false premise that `Option<T>` makes a struct-literal field optional; `feedback_insertform_default_propagation.md` refutes it), and that this completes the InsertForm-field propagation the lesson requires.
2. **Pre-push cargo-check discipline** (per `feedback_fix_impl_pre_push_cargo_check.md`): AFTER `git submodule update --init` (§0) and BEFORE pushing the worker branch, run `cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"` locally in-worker. Exit 0 expected (`..Default::default()` on a `#[derive(Default)]` form is sound). Non-zero → STOP, do NOT push, file `kind: "blocker"` DQ **into `.claude/decision-queue.json`** citing the cargo-check failure verbatim (full last ~80 lines). Do NOT `#[allow]`/`#[expect]`-spam to make it pass. (clippy + `--test e2e --no-run` are re-run by the advisor on the laptop post-finalize-merge per Shape-G-suspended §5.2 — you only run cargo-check pre-push.)
3. **DQ raise — write into the canonical `.claude/decision-queue.json`, NOT a repo-root file.** Task 8 (#318) wrote `TASK8_ESCALATION.md` + `TASK8_VALIDATE_PENDING.json` at repo root instead of canonical DQ entries — that is a **process miss** the advisor had to clean up (transcribe + rm). **Do NOT repeat it.** Per `.claude/refs/dq-recipes.md` Recipe + `.claude/rules/decision-queue.md` "Mid-task visibility": this fix-impl does NOT need a new validate-pending DQ (Task 8's validate is already DQ #264, which the advisor re-runs on the fixed tip). You only file a DQ if §0 or §4.2 forces a `kind: "blocker"` — and that blocker MUST be a properly-formed entry **appended to `.claude/decision-queue.json`'s `pending[]`** (computed `id = max(all ids across pending+resolved+archives)+1`; recompute, do not hardcode; **pin `ensure_ascii=False`** — the impl-task DQ-write ascii-escape breach recurred 5x this phase, write canonical UTF-8 not `\uXXXX`-escaped), committed + pushed on the worker branch in the same or a second commit. Do **NOT** mutate or touch DQ #264 or DQ #265 (the advisor owns #264's resolution; #265 is already resolved).
4. **MIRROR-ref discipline:** the fix form is dictated by the **user decision in DQ #265 (option-a)** + `feedback_insertform_default_propagation.md` — add `..Default::default()` to the callsite, exactly as Task 8 already did to the 2 `inbox.rs` callsites. Do NOT specify the 6 fields explicitly, do NOT add attributes, do NOT edit any other callsite or file. The single `..Default::default()` line insertion is the entire change.
5. **Attribution:** if a blocker DQ is forced, `from: "impl"`, `answered_by: null`. NEVER write `answered_by: "advisor"` / `"user"` / `kind: "clarify"` (per `.claude/rules/decision-queue.md` hard refusals).
6. **Serial discipline:** this fix-impl is the in-flight Task 8 §5.2 e2e-compile recovery under Cohort B serial cap=1. Do not dispatch or reference any other task. One worker, one fix, terminal.

## §5 Acceptance

- `crates/server/tests/e2e.rs` at the callsite that was lines 7451–7457: a single `..Default::default()` line is inserted after `signature: "seed-sig".to_string(),`, before the closing `})`. The 5 original field lines are byte-identical; only the one `..Default::default()` line is added.
- `git diff --stat` = 1 file (`crates/server/tests/e2e.rs`); `git diff` shows **exactly one inserted line** (net +1), nothing else.
- No other callsite, test, import, or line in `e2e.rs` changed. No other file changed.
- `git submodule update --init` ran in §0 (CMD1 cargo-check meaningful).
- Local `cargo-check.bat --workspace --features full` exit 0 (run pre-push per §4.2, after submodule init).
- No new DQ entry needed in the happy path (Task 8 validate is DQ #264, advisor-owned). If §0/§4.2 forced a `kind: "blocker"`, it is a properly-formed entry in `.claude/decision-queue.json` `pending[]` (`ensure_ascii=False` canonical) — **NOT a repo-root file**.
- DQ #264 NOT touched. DQ #265 NOT touched (already resolved). No repo-root `.md`/`.json` debris files created.
