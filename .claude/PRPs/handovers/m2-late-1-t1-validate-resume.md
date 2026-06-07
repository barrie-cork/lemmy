# m2-late-1 T1 validation — RESUME handover (advisor laptop session)

**Author:** advisor (Opus 4.8 laptop session), 2026-06-07
**For:** next advisor session resuming this DQ validation (safe to run on Sonnet — see "Model note")
**Lane:** `C:/Users/barri/Developer/brehon-fork-m2late` (Mode A), branch `phase-m2-late-1`

---

## One-line state

Validating DQ `001f1c47c5dc-001` (`validate-pending-laptop`): T1 `sanction_event` migration round-trip + workspace `cargo-check --features full`. Pre-phase audit ~80% done. **schema.rs NOT yet regenerated** — that is the gating next action and the workspace check CANNOT pass until it is done.

## VERIFIED_AT

- Branch tip: `e7eedeee8` (docs(lessons) commit — see below). Code tip from T1: `057be173a`. Merge HEAD was `4f1fda8b5`.
- Working tree: CLEAN (the lesson file is committed). `git status --short` → empty.
- DQ pending: 1 → `001f1c47c5dc-001` only.

## Model note (safe to /clear + switch to Sonnet?)

**YES, safe.** `/clear` resets conversation context only — filesystem + git are untouched, nothing uncommitted is at risk (working tree clean). The remaining work is mechanical GIVEN this handover. The ONE judgment call (schema.rs regen on Windows + patch staleness) is spelled out as an explicit decision tree below so Sonnet does not have to derive it. If the regen path hits something not covered here, raise a `kind: blocker` DQ and surface to user rather than guessing.

---

## What is DONE

1. **Lesson written + committed** (`e7eedeee8`): `.claude/lessons/feedback_diesel_ltree_patch_line_anchored_fragile.md` — the diesel_ltree.patch line-anchored fragility finding (user-requested follow-up). Retro-harvest candidate.
2. **Pre-phase audit probes** (`.claude/rules/pre-phase-harness-audit.md`):
   - Probe 0 (Docker): ✅ OK
   - Probe 1 (`cargo-check -p lemmy_utils`): ✅ exit 0, `-p` honored, no scope leak. Log `.claude/audit-cargo-check-p.log`.
   - Probe 2 (`cargo-check -p lemmy_db_schema --features full`): exit **101** (notification falsely said 0 — verified true code via output-file marker). Failure is the EXPECTED pre-regen gap: `E0425 cannot find type SanctionKind in crate::schema::sql_types` (enums.rs added the enum; schema.rs not regenerated). Wrapper integrity OK (compiled all deps with full features active; failed only at the known gap). Log `.claude/audit-cargo-check-features.log`.
   - Probe 3 (`cargo-test --test e2e --no-run -p lemmy_server`): exit **101**, SAME SanctionKind gap (transitive dep). Log `.claude/audit-cargo-test.log`.
   - **Probe 4 (negative exit-code-masking) NOT cleanly run** — deferred. With the SanctionKind gap present, a bogus-feature probe fails on EITHER cause, muddying the masking signal. RUN PROBE 4 AFTER schema.rs regen (clean baseline). Until then the audit flag is NOT yet touched.

## CRITICAL CORRECTIONS to the DQ entry (do not run the literal commands)

- DQ command 1 literal text: `cargo run -p lemmy_diesel_utils --features full -- migration run` is **WRONG**. `crates/diesel_utils/src/main.rs:4` does `bail!` on ANY argument. Correct form: bare `cargo run -p lemmy_diesel_utils --features full` with `LEMMY_DATABASE_URL` env set. (Confirmed by source read + PMD `feedback_lemmy_migration_runner.md`.)
- `diesel print-schema` is read-only/safe (forbid-trigger doesn't apply).

---

## REMAINING TASKS (in order)

### Task A — Regenerate schema.rs (THE gating step)

`diesel print-schema` AUTO-APPLIES the ltree patch — `diesel.toml` declares `patch_file = "crates/db_schema_file/diesel_ltree.patch"`. So patch application is NOT a separate `git apply`; diesel does it. If hunk #2 is stale (line offsets shifted by the new sanction_* tables), `diesel print-schema` emits a patch-apply WARNING — watch stderr for it.

The canonical regen script is `scripts/update_schema_file.sh`:
```
source scripts/start_dev_db.sh          # ← Linux/local-pg path (pg_ctl/initdb/$PGDATA)
cargo run --package lemmy_diesel_utils --features full   # apply migrations
diesel print-schema >crates/db_schema_file/src/schema.rs # regen + auto-apply ltree patch
cargo +nightly fmt --package lemmy_db_schema_file
```

**WINDOWS DIVERGENCE (decision point):** `start_dev_db.sh` uses `pg_ctl`/`initdb`/`$PGDATA` — a LOCAL Postgres install, which the Windows laptop may not have. The rest of the harness uses Docker containers. Options:

- **Option A1 (recommended):** stand up a migrated Postgres in Docker, point `diesel print-schema` at it manually:
  1. `docker run -d --rm -e POSTGRES_PASSWORD=password -e POSTGRES_DB=lemmy -p 5433:5432 pgautoupgrade/pgautoupgrade:18-alpine` (the user's step-1 `localhost:5433` DB).
  2. `export LEMMY_DATABASE_URL=postgres://postgres:password@localhost:5433/lemmy` (verify user/pwd/db match the container).
  3. Apply migrations: invoke the runner WITH Windows vcpkg PATH — reuse `scripts/brehon/migrate-roundtrip-cargo.bat` mechanism OR `cmd //c` wrapping `cargo run -p lemmy_diesel_utils --features full` after setting vcpkg PATH (libpq.dll). DO NOT run bare `cargo run` in git-bash (STATUS_DLL_NOT_FOUND — see `feedback_windows_e2e_requires_bat_wrapper.md`).
  4. `diesel print-schema > crates/db_schema_file/src/schema.rs` (also needs UCRT/vcvars PATH — diesel.exe built in vcvars shell; run from cmd+vcvars, NOT bare git-bash, per `feedback_lemmy_migration_runner.md` Windows gotcha).
  5. `cargo +nightly fmt --package lemmy_db_schema_file`.
- **Option A2:** check if the user's pre-existing `localhost:5433` DB (step 1 of their instructions) is already running + migrated; if so skip the docker-run + migrate, just print-schema against it. NOTE the user's step-1 URL was `postgres://lemmy:password@localhost:5433/lemmy` (user `lemmy`, not `postgres`) — reconcile credentials with whatever is actually running.

**Patch-staleness handling (the user's explicit concern):** if `diesel print-schema` warns the patch doesn't apply (hunk #2 offset stale due to sanction_* tables inserting above line ~1366), regenerate the patch from the new baseline rather than force-applying. Procedure: produce the print-schema output WITHOUT the patch, hand-apply the two semantic fixups (Ltree import on `comment` table → `diesel_ltree::sql_types::Ltree`; add `person_actions` + `image_details` + the federation-inbound-a tables + the new sanction tables as appropriate to `allow_tables_to_appear_in_same_query!`), then `git diff` to regenerate `diesel_ltree.patch`. See `.claude/lessons/feedback_diesel_ltree_patch_line_anchored_fragile.md` for the full reasoning. **If this gets ambiguous → raise a `kind: blocker` DQ and surface to user.**

After regen: `git diff --stat crates/db_schema_file/src/schema.rs` should show the new `sql_types::SanctionKind` block + `sanction_event` + `sanction_subscriber` table DSL. If schema.rs changed, COMMIT it on `phase-m2-late-1`: `feat(db_schema): regen schema.rs for sanction_event (task 1 validation)`.

### Task B — Migration round-trip proof

Run `bash scripts/brehon/migrate-roundtrip.sh` (spins its OWN throwaway containers, diffs vs origin/governance-v0, applies the new migration forward twice on fresh DBs; Windows-aware via migrate-roundtrip-cargo.bat). Capture: `bash scripts/brehon/migrate-roundtrip.sh > .claude/m2-late-1-t1-migrate.log 2>&1; echo "exit: $?"`. Expect exit 0 + "round-trip complete for 1 new migration(s)". (This is INDEPENDENT of Task A — it doesn't need schema.rs; it validates the SQL applies. Can run before or after A.)

### Task C — Workspace check (the real positive signal)

After Task A regen resolves the SanctionKind reference: `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/m2-late-1-t1-check.log 2>&1"; echo "exit: $?"`. Verify TRUE exit via the printed marker, NOT the task-notification (notifications lie — confirmed 3× this session). Expect exit 0. NOTE: `--workspace --features full` is correct; never `-p <crate> --features full` (incompatible per `feedback_features_full_p_crate_incompatible`).

### Task D — Finish the audit

Run Probe 4 (negative) now that baseline is clean:
`cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-check-negative.log 2>&1"; echo "PROBE4_EXIT: $?"` — expect NON-ZERO (101) + "does not contain this feature: nonexistent_xyz". If it prints 0 → exit-code masking bug, STOP + fix wrapper. On pass: `touch .claude/audit-phase-m2-late-1-complete.flag`.

### Task E — Mutate the DQ entry

ONLY if Tasks B + C both pass. Set on `001f1c47c5dc-001`: `result:"pass"`, `answered_by:"advisor-laptop"`, fill `answer` (cite migrate-roundtrip exit + workspace-check exit + schema.rs regen SHA), `resolved_at`, move pending[]→resolved[]. Author the mutation via a Write-tool JSON fragment + read-merge (avoid Windows backslash-path mangling per `feedback_windows_backslash_path_dq_via_write_fragment.md`), or edit decision-queue.json directly with the Edit tool. Commit: `chore(decision-queue): advisor-laptop validated DQ 001f1c47c5dc-001 — T1 sanction_event pass`. Push `origin phase-m2-late-1`.
On FAIL of B or C: leave entry in pending[], populate `result:"fail"` + `log_slice` (last 100 lines of failing cmd) + `failed_commands`, `answered_by:"advisor-laptop"`, then §G4 triage / surface to user.

---

## Watchpoints

- **task-notifications lie** — every cargo background task this session reported "exit 0" while truly exit 101. ALWAYS verify via the `echo "...EXIT: $?"` marker in the output file, never the notification summary. (`feedback_task_notification_exit_summary_unreliable`)
- **PHASE_1_MIGRATION_COUNT (LIFO)** — `feedback_phase1_migration_count_lifo.md`: if the e2e round-trip test (`phase1_migrations_round_trip`) is later un-ignored / in scope, the new migration may need a count bump. NOT triggered by THIS validation task (migration-run + workspace-check only). Flag for the planner if e2e is added.
- **m2-rooms-a worktree still present** (`brehon-fork-m2rooms-a` @ `a3192d3c9`) though MEMORY says it CLOSED 2026-06-06 (PR #191 merged). Not blocking; clean up at convenience via `git worktree remove`.

## RESUME first action

Read this file + `.claude/PRPs/handovers/m2-late-1-t1-done.md` + the lesson `feedback_diesel_ltree_patch_line_anchored_fragile.md`. Then START at **Task A** (schema.rs regen) — it gates Task C. Tasks A and B are independent and can run concurrently.
