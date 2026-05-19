---
phase: v1-federation-inbound-a
role: impl-task
kind: impl
task: 9
authored: 2026-05-18
plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md
base: "phase-v1-federation-inbound-a @ 0874634d0 (Cohort B 3/3 COMPLETE: T6 4032b5484 + T7 107f4bf1f + T8 998fef033/fix-impl-3 528f870ac all VALIDATED-PASS; DQ #264 resolved; lane==origin==daemon-local synced)"
requires:
  - { task: 1, present_on_phase_branch: true, why: "migration directory must exist for round-trip probe" }
  - { task: 2, present_on_phase_branch: true, why: "schema.rs new table! blocks for foundation-test imports" }
  - { task: 5, present_on_phase_branch: true, why: "newtypes (InstanceId etc.) for model imports" }
  - { task: 6, present_on_phase_branch: true, why: "federation_peer.rs + federation_inbox_check_peer_trust must exist" }
cap: "1 file edit (crates/server/tests/e2e.rs ONLY) — 4 anchored Edits per plan §10.8 (1 const-list prepend + 3 probe-block appends-before-Ok(()) + 1 NEW fixtures module at EOF)"
serial: "Cohort B strictly serial cap=1. Task 9 is the e2e.rs BARRIER (requires 1+2+5+6, all on phase branch). It is the LAST impl task before Task 10 (retro). On done + finalize-merge: advisor runs §5.2 (cargo-check + clippy + cargo-test --test e2e --no-run) on the laptop; on all-pass the phase advances to Phase-2 e2e user-gate-4 (local-vs-dispatch, never auto-pick)."
lesson_mirror_gap: "feedback_junior_worker_e2e_edit_hang.md is cited by name in advisor-orchestrator.md §2.4 + MEMORY.md but is ABSENT from .claude/lessons/ on this lane AND on origin/governance-v0 (it lives only in user-memory + is inlined into plan §10.8 GOTCHA + §'Lessons that bind §13'). The binding constraint (e2e.rs is huge — 14,863 lines — anchor-Edit ONLY, never full-file Read/rewrite/multi-hunk) is therefore quoted verbatim in §3 below from the plan + the lesson's own How-to-apply text. Retro carry-forward: author the lesson file from the user-memory copy + commit to governance-v0 .claude/lessons/."
---

# [role:impl-task] v1-federation-inbound-a Task 9 — e2e.rs phase-1 round-trip probes + trust-state foundation fixtures (BARRIER) — see .claude/PRPs/briefs/federation-inbound-a-impl-9.md

> **Provenance:** Task 9 is the e2e.rs **barrier** task of `v1-federation-inbound-a`. Cohort A (Tasks 1-5: migration + schema.rs + config.rs + entry-kinds + newtypes) and Cohort B (Tasks 6-8: 4 new Diesel model files + Phase-6 model extensions) are all merged + VALIDATED-PASS on `phase-v1-federation-inbound-a @ 0874634d0`. Task 9 extends the **existing** 3-test phase-1 migration round-trip suite with `-a` probes and appends a **NEW** `mod v1_federation_inbound_a_fixtures` (2 trust-state foundation tests) at end-of-file. Per plan §10.8 (concrete code, exact anchors) + §"Lessons that bind §13": **Task 9 uses single anchor-Edits** into `crates/server/tests/e2e.rs` (14,863 lines). This is the lesson-sanctioned narrow pattern, NOT the hang-prone full-file rewrite — and this exact phase has already proven it works (fix-impl-3 #320 just did a clean 1-line anchor-Edit into this same file with zero hang). Case A error shape per `feedback_lemmy_error_no_std_error.md` (uniform `LemmyResult<()>` outer, every helper `LemmyResult<T>`, bare `?`, one `map_err` on the testcontainer call) — mirrors the canonical sibling `mod v1_sl_b_fixtures` already in this file.

## §0 Pre-flight (subagent runs this FIRST, before reading anything else)

- Confirm CWD branch is a Junior worktree branched off `phase-v1-federation-inbound-a` (base tip `0874634d0`). `git merge-base --is-ancestor 0874634d0 HEAD` MUST be true. If on `phase-v1-federation-inbound-a` itself or a non-`junior/*` branch → STOP, file `kind: "blocker"` DQ **into `.claude/decision-queue.json`** (NOT a repo-root file — see §4 Constraint 4).
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — if inside a forbidden window exit non-zero `FORBIDDEN_WINDOW: <window>`, UNLESS the dispatch description carries `(user-authorised forbidden-window override per DQ #<id>)`. Shape G SUSPENDED (cargo on laptop) — keep the check.
- **Submodule init (MANDATORY before any in-worker cargo — per `feedback_worktree_submodules_not_auto_init`):** run `git submodule update --init 2>&1` from the worktree root. The `crates/email/translations` git submodule is NOT auto-initialized in a fresh worktree; without it `cargo-check` fails pre-existing (ENOENT on `translations/backend/`) — the CMD1 false-negative class that hit Cohort-A Task-5 / DQ #236 and fix-impl-1/3. Infra, NOT part of the task; initialize it so the §4.2 pre-push cargo-check is meaningful.
- Confirm the 4 anchor points are present + unmodified (read **only narrow `sed` windows**, NOT the whole file):
  - `grep -n "MIGRATIONS_TO_REVERT_PHASE_1" crates/server/tests/e2e.rs | head -1` → note the const declaration line.
  - `grep -n "fn test_phase1_migrations_forward\|fn test_phase1_migrations_revert\|fn test_phase1_migrations_reapply" crates/server/tests/e2e.rs` → note the 3 test fn line ranges.
  - `grep -n "^mod v1_sl_b_fixtures\|^mod v1_sl_e_fixtures" crates/server/tests/e2e.rs` → confirm the canonical sibling module exists (mirror target) and note EOF region.
  - `tail -5 crates/server/tests/e2e.rs` → confirm current EOF (the new module appends after the last existing line; do NOT displace any existing module).
  - If any anchor is absent or the const/test fns are missing → STOP, file `kind: "blocker"` DQ **into `.claude/decision-queue.json`** (base mismatch — required tasks not on this worktree's base).
- Confirm `requires:` are present on base: `for t in 1 2 5 6; do git log --grep "(task $t)" --oneline | head -1; done` — each MUST return a commit. (Advisor already verified pre-dispatch; this is the worker's defence-in-depth.)

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-a Task 9 — e2e.rs phase-1 round-trip probes + trust-state foundation fixtures (BARRIER)`

Actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-a Task 9 — see .claude/PRPs/briefs/federation-inbound-a-impl-9.md
```

## §2 Scope

### 2.1 What Task 9 produces (the contract)

Plan §10.8 is the authoritative spec. Read it (`.claude/PRPs/plans/v1-federation-inbound-a.plan.md` §10.8, lines ~1088–1182) — it has the concrete code, the exact anchor lines, and the Case-A error shape. The 5 changes, **all in `crates/server/tests/e2e.rs`, ALL via anchor-Edit**:

1. **`MIGRATIONS_TO_REVERT_PHASE_1` const** (anchor: the const's array literal): **prepend** `"2026-05-17-000000-0000_add_federation_inbound_v1",` as the first element + a group comment line `// v1-federation-inbound-a (1 migration, bump 18 → 19)` immediately above it. Anchor-Edit keyed on the first existing array element + its preceding comment — do NOT rewrite the whole const.

2. **`test_phase1_migrations_forward`** (anchor: the `Ok(())` at the END of this fn): **append a new probe block immediately before that `Ok(())`** asserting PRESENCE — 4 table-existence + 2 pg_type + (4+6) column + 10 index + 2 config-key probes per §10.8 item 2. Anchor-Edit keyed on the unique `Ok(())\n}` closing this specific fn (use a few lines of preceding context to disambiguate from the other two fns' `Ok(())`).

3. **`test_phase1_migrations_revert`** (anchor: the `Ok(())` at the END of this fn): **append a mirror block before that `Ok(())`** asserting ABSENCE (mirror of #2). Anchor-Edit, disambiguated by preceding context.

4. **`test_phase1_migrations_reapply`** (anchor: the `Ok(())` at the END of this fn): **append a small re-run block before that `Ok(())`** per §10.8 item 4. Anchor-Edit, disambiguated.

5. **NEW `mod v1_federation_inbound_a_fixtures`** at **end-of-file**: append the module **verbatim from plan §10.8 item 5** (the full `mod v1_federation_inbound_a_fixtures { ... }` block — `seed_federation_peer` helper + `federation_peer_trust_lookup_returns_seeded_state` + `federation_peer_trust_lookup_returns_unknown_for_first_seen`). Append after the current last line of the file (anchor-Edit keyed on the existing final module's closing `}` + EOF, OR use a trailing-append that does not displace any existing content). Do NOT modify any existing module.

### 2.2 Case A error shape (the contract — implement EXACTLY per plan §10.8 / canonical sibling)

The NEW module is **Case A** per `feedback_lemmy_error_no_std_error.md` (mirror the canonical sibling `mod v1_sl_b_fixtures` already in this file):

- Test fn signatures: `async fn <name>() -> LemmyResult<()>`.
- Helper `seed_federation_peer` signature: `-> LemmyResult<InstanceId>`.
- Every `?` propagation: **bare**, no `.map_err` — EXCEPT the one `governance_fixtures::start_postgres_with_migrations()` testcontainer call, which uses `.map_err(|e| LemmyErrorType::Unknown(format!("{e}")))?` exactly as written in plan §10.8 (anyhow→LemmyError bridge per v1-SL-b precedent).
- Module imports: exactly as listed in plan §10.8 item 5 (`AsyncPgConnection`, `RunQueryDsl`, `ExpressionMethods`, `QueryDsl`, `InstanceId`, `federation_inbox_check_peer_trust`, `FederationPeerInsertForm`, `FederationPeerTrust`, `federation_peer`, `instance`, `LemmyResult`).

Copy the module body **verbatim from plan §10.8 item 5**. Do not paraphrase, do not "improve", do not change field names or import paths.

### 2.3 Boundaries

- Edit **ONLY** `crates/server/tests/e2e.rs`. Cap: **1 file**, **5 anchored changes** (4 in existing items + 1 EOF module append).
- Do **NOT** touch `crates/db_schema/**`, `crates/apub/**`, `crates/api/**`, the model files, `schema.rs`, `config.rs`, `governance_log.rs`, `Cargo.toml`, any migration, any other `.claude/**` file (except a forced `kind:"blocker"` DQ per §4, written **into `.claude/decision-queue.json`**).
- Do **NOT** modify any **existing** test, module, helper, import, or const beyond the 5 changes in §2.1. `test_phase1_migrations_forward/revert/reapply` get **append-only** insertions before their final `Ok(())` (DQ #232 APPEND-ONLY discipline — never rewrite existing assertions). The `MIGRATIONS_TO_REVERT_PHASE_1` change is a **prepend** of one element + one comment (existing elements byte-identical).
- Do **NOT** add `#[allow]`, `#[expect]`, or any attribute not present in plan §10.8.

## §3 Required reading

- **`.claude/PRPs/plans/v1-federation-inbound-a.plan.md` §10.8** (lines ~1088–1182) — **the authoritative spec.** Concrete code, exact anchor lines, the verbatim `mod v1_federation_inbound_a_fixtures` block. This is the contract; implement it exactly.
- **`.claude/PRPs/plans/v1-federation-inbound-a.plan.md` §13 Task 9** (lines ~1657–1700) — FILES yaml (`modifies: [crates/server/tests/e2e.rs]`, `requires: 1,2,5,6`), the GOTCHA lines, the §15 push commands, the COMMIT subject.
- **`.claude/PRPs/plans/v1-federation-inbound-a.plan.md` §"Lessons that bind §13 decisions"** (lines ~90–135) — line 131-132: "`feedback_junior_worker_e2e_edit_hang.md` — Task 9 uses **single anchor-Edits**." This is the binding constraint.
- **e2e-edit-hang constraint (lesson file is NOT in this lane's `.claude/lessons/` — quoted here verbatim, this is the binding rule):** *"Junior workers reliably hang when issuing `Edit` calls into `crates/server/tests/e2e.rs` … the worker's log file stops growing but the process is alive with non-zero CPU — stuck mid-tool-result. … For any task whose DoD names edits to `crates/server/tests/e2e.rs`, never do the work via a full-file Read/rewrite or a multi-hunk Edit. Use clean small edits, well-separated lines (anchor-Edits)."* **Operationalised for Task 9:** (a) NEVER `Read` the whole `e2e.rs` (14,863 lines) — use narrow `sed -n 'A,Bp'` windows around each anchor to confirm context pre-Edit; (b) do the 5 changes as **5 separate small anchor-Edits** (one `Edit` call per change, each `old_string` = a tight unique 2–6 line context, `new_string` = same + the inserted block); (c) do NOT batch all 5 into one giant Edit; (d) if any single `Edit` does not return within a few minutes, do NOT retry blindly — STOP and file a `kind:"blocker"` DQ into `.claude/decision-queue.json` describing which anchor wedged (the advisor will pre-apply that hunk from the orchestrator session and re-dispatch the remainder).
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Case A** is the shape for the new module (uniform `LemmyResult<()>`/`LemmyResult<T>`, bare `?`, one `map_err` on the testcontainer). The canonical sibling to mirror is `mod v1_sl_b_fixtures` already in this same file (find it with `grep -n "^mod v1_sl_b_fixtures" crates/server/tests/e2e.rs`, read only that module's range with `sed`). Do NOT introduce `Box<dyn Error>` anywhere (that is Case C — hard refusal).
- `.claude/lessons/feedback_async_pool_test_pattern.md` — the trust helper takes `&mut AsyncPgConnection`; the fixtures use `governance_fixtures::async_conn(&db_url)`. Mirror the canonical sibling's connection-acquisition pattern exactly.
- `.claude/lessons/feedback_insertform_default_propagation.md` — context only: `FederationPeerInsertForm` in §10.8's `seed_federation_peer` is constructed with **all 4 of its fields explicitly** (`instance_id`, `trust_level: Some(...)`, `added_by_actor: None`, `notes: None`) per plan §10.8 — that is already complete, no `..Default::default()` needed there. Do not add one; do not change those field values.
- `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` — pre-push local cargo-check discipline (§4 Constraint 2).
- `.claude/lessons/feedback_worktree_submodules_not_auto_init.md` — why §0 mandates `git submodule update --init` before in-worker cargo-check.
- The base-state DQ: `.claude/decision-queue.json` — DQ #264 (Task 8, resolved pass — context only, do NOT touch) + #265 (resolved, option-a — context only).

## §4 Constraints

1. **One commit.** Subject **exactly**: `feat(v1-federation-inbound-a): e2e.rs — phase1 round-trip probes + trust-state foundation test module (task 9)`. Commit body cites: implements plan §10.8 (5 anchored changes), Case A error shape mirroring `mod v1_sl_b_fixtures`, `requires:` 1+2+5+6 all present on base `0874634d0`, APPEND-ONLY discipline honoured (DQ #232), and a `HANDOVER:` YAML trailer (filesModified: `[crates/server/tests/e2e.rs]`, keyDecisions: e.g. "5 anchor-Edits, no full-file rewrite", notes: pre-push cargo-check exit code).
2. **Pre-push cargo-check discipline** (per `feedback_fix_impl_pre_push_cargo_check.md`): AFTER `git submodule update --init` (§0) and BEFORE pushing the worker branch, run `cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"` locally in-worker. Exit 0 expected. Non-zero → STOP, do NOT push, file `kind:"blocker"` DQ **into `.claude/decision-queue.json`** with the cargo-check failure verbatim (last ~80 lines). Do NOT `#[allow]`/`#[expect]`-spam to make it pass. (clippy + `cargo-test --test e2e --no-run` are re-run by the **advisor on the laptop** post-finalize-merge per Shape-G-suspended §5.2 — you only run cargo-check pre-push. e2e EXECUTION is Phase-2 user-gate-4, NOT your job.)
3. **Post-push validate-pending-laptop DQ (REQUIRED — this is a normal impl-task, not a fix-impl).** After a clean pre-push cargo-check + push, raise a `kind: "validate-pending-laptop"` entry **into `.claude/decision-queue.json`** `pending[]` per `.claude/rules/decision-queue.md` + `.claude/refs/dq-recipes.md`: `from: "impl"`, `answered_by: null`, `branch: "phase-v1-federation-inbound-a"`, `phase_task: 9`, `commands:` = the **3 verbatim §15 commands from plan §13 Task 9 "Push and exit"** (cargo-check.bat + cargo-clippy.bat -D + cargo-test.bat --test e2e --no-run, all `--workspace --features full`, exact strings from the plan), `result: null`, `log_slice: null`, `failed_commands: null`. Compute `id = max(all ids across pending+resolved+ archives)+1` (recompute live, do NOT hardcode — sibling lanes share the id space; the worker-pre-picked-id collision bit Task 8). **Pin `ensure_ascii=False`** — the impl-task DQ-write ascii-escape breach recurred 5x this phase; write canonical UTF-8, NOT `\uXXXX`-escaped. Commit + push it on the worker branch (same commit as the feat, or an immediate follow-up `chore(decision-queue): impl raised DQ #<id> — task 9 validate-pending-laptop`). The advisor reads this on its next poll and runs the 3 commands on the laptop.
4. **DQ writes go into canonical `.claude/decision-queue.json`, NEVER a repo-root file.** Task 8 (#318) wrote `TASK8_ESCALATION.md` + `TASK8_VALIDATE_PENDING.json` at repo root — a process miss the advisor had to clean up. Do **NOT** repeat it. Every DQ entry (the §4.3 validate-pending OR any forced §0/§4.2 blocker) is a properly-formed entry appended to `.claude/decision-queue.json`, committed + pushed on the worker branch. Do **NOT** mutate/touch DQ #264 or #265.
5. **Attribution:** all DQ entries you write are `from: "impl"`, `answered_by: null`. NEVER write `answered_by: "advisor"` / `"user"` / `kind: "clarify"` / `kind: "validate-result"` / `kind: "validate-failed"` (per `.claude/rules/decision-queue.md` hard refusals). ci-watcher/advisor own mutations.
6. **MIRROR-ref discipline:** the new module is dictated by **plan §10.8 item 5 verbatim** + the canonical sibling `mod v1_sl_b_fixtures`. Do NOT invent test logic, do NOT change import paths, do NOT add tests beyond the 2 specified, do NOT alter the 3 existing round-trip tests beyond the append-only probe blocks in §2.1.
7. **Serial discipline:** Task 9 is the in-flight Cohort-B-serial barrier (cap=1). It is the last impl task. Do not dispatch or reference any other task. One worker, one impl commit (+ optional DQ follow-up commit), terminal.

## §5 Acceptance

- `git diff --stat` = **1 file** (`crates/server/tests/e2e.rs`); net change is **5 anchored insertions** (1 const prepend of element+comment; 3 append-only probe blocks before each round-trip test's final `Ok(())`; 1 new `mod v1_federation_inbound_a_fixtures` at EOF). NO existing test/module/import/const otherwise modified.
- `MIGRATIONS_TO_REVERT_PHASE_1` first element is `"2026-05-17-000000-0000_add_federation_inbound_v1",` with the `// v1-federation-inbound-a (1 migration, bump 18 → 19)` comment above it; all prior elements byte-identical, just shifted down.
- `grep -c "^mod v1_federation_inbound_a_fixtures" crates/server/tests/e2e.rs` = 1; the module body matches plan §10.8 item 5 (2 `#[tokio::test(flavor = "multi_thread")]` fns + `seed_federation_peer` helper, Case A shape, one `map_err` only on the testcontainer call).
- No `Box<dyn Error>` introduced anywhere (Case A only).
- `git submodule update --init` ran in §0.
- Local `cargo-check.bat --workspace --features full` exit 0 (run pre-push per §4.2, after submodule init).
- A `kind: "validate-pending-laptop"` DQ entry for `phase_task: 9` with the 3 verbatim §15 commands is appended to `.claude/decision-queue.json` `pending[]` (`from: "impl"`, `result: null`, `ensure_ascii=False` canonical, id recomputed live), committed + pushed on the worker branch.
- No repo-root `.md`/`.json` debris files created. DQ #264/#265 NOT touched.
- The 5 changes were each done as a **separate small anchor-Edit** (no full-file `e2e.rs` Read; no single mega-Edit) per the e2e-edit-hang binding constraint in §3.
