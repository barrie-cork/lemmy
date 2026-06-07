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

### Task A — Hand-edit schema.rs (THE gating step) — VERIFIED TURNKEY 2026-06-07

**DO NOT run `diesel print-schema`.** Research (2026-06-07, two subagents, evidence below) established:
1. **The fork convention is HAND-EDIT, not regen.** Every prior migration phase (fed-in-a `0c9329da9`, v1-AD-a `dbaf58e4b` "schema.rs hand-edit", v1-SL-a `c7a977078`) hand-extended the `@generated` schema.rs. The `// v1-federation-inbound-a additions:` markers in the live file are the signature.
2. **`update_schema_file.sh` is Linux-only** (`start_dev_db.sh` = `pg_ctl`/`initdb`; no Windows pg install exists; `.env` has no DATABASE_URL). It has NEVER run on this laptop.
3. **The plan's prose is WRONG** — `m2-late.plan.md:224,443` says "the migration runner regenerates schema.rs". FALSE: `cargo run -p lemmy_diesel_utils` only applies migrations to a DB; it never writes schema.rs. (Plan-correction noted for retro; do not act on the false instruction.)
4. **`diesel_ltree.patch` is MOOT for this task.** It is referenced ONLY by `diesel.toml` (a print-schema input) — never at build time (`grep` of build.rs/src clean). Its effect is already baked into the committed schema.rs (the `diesel_ltree::sql_types::Ltree` import at schema.rs:199 + the person_actions/image_details allow_tables entries). Since we hand-edit and never run print-schema, **the patch and its "hunk #2 staleness" never engage.** (For the record, had print-schema run: diesel_cli 2.3.7 uses `diffy` 0.4.2 → a stale-context hunk HARD-ABORTS exit 1 and the `>` redirect truncates schema.rs to empty first. Another reason to hand-edit. Full detail: `feedback_diesel_ltree_patch_line_anchored_fragile.md` — note its "git apply" framing is slightly off; the real mechanism is diffy.)

**The edit — FOUR insertions into `crates/db_schema_file/src/schema.rs`** (line numbers are pre-edit; do each insertion top-down so earlier inserts shift later anchors — OR insert bottom-up to keep anchors stable; recommended bottom-up = allow_tables, then joinable, then table blocks, then sql_types). Verify each anchor with a `grep` before editing (lines drift if any prior commit touched the file).

**A1. sql_types struct** — insert between `SanctionAction` (currently schema.rs:132-134) and `SanctionScope` (136-138), preserving alphabetical order (`sanction_kind` < `sanction_scope`). 2-space indent (inside `pub mod sql_types`):
```rust
  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "sanction_kind"))]
  pub struct SanctionKind;
```

**A2. Two `table!` blocks** — insert AFTER the `sanction` block's closing (currently schema.rs:1385, the `}` that closes `diesel::table! { ... sanction (id) {...} }`) and BEFORE `secret` (1387). Mirror the `sanction` block's `use super::sql_types::...;` style. Column→Diesel-type mapping derived from `migrations/2026-06-07-000000-0000_add_sanction_event/up.sql` (verified against the `sanction` block at 1372-1384):
```rust
diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SanctionKind;

    sanction_event (id) {
        id -> Int4,
        sanction_id -> Int4,
        sanction_kind -> SanctionKind,
        subject_actor_pseudonym -> Text,
        effective_from -> Timestamptz,
        effective_until -> Nullable<Timestamptz>,
        governance_log_entry_hash -> Text,
    }
}

diesel::table! {
    sanction_subscriber (id) {
        id -> Int4,
        callback_url -> Text,
        active -> Bool,
        created_at -> Timestamptz,
    }
}
```
(`sanction_subscriber` has no enum/FK columns so no `use` block — mirror the `secret` table at 1387-1392 which is also bare.)

**A3. `joinable!`** — `sanction_event.sanction_id` FKs to `sanction(id)` (up.sql: `REFERENCES sanction(id)`). Insert after the `sanction ->` joinable group (currently schema.rs:1587-1591), keeping rough alpha order:
```rust
diesel::joinable!(sanction_event -> sanction (sanction_id));
```
(`sanction_subscriber` has no FK → no joinable.)

**A4. `allow_tables_to_appear_in_same_query!`** — add both tables to the FIRST macro (the big one at schema.rs:1599). Per fork convention (fed-in-a marker style), append a marker block before the closing `);` (currently ~schema.rs:1670, right after the `// v1-federation-inbound-a additions:` group):
```rust
  // m2-late-1 additions:
  sanction_event,
  sanction_subscriber,
```

**Then:** `cargo +nightly fmt --package lemmy_db_schema_file` (the fork's fmt step — note prior phases fmt'd; if `+nightly` toolchain absent, skip and let the workspace check confirm format-neutral compile). Then `git diff --stat crates/db_schema_file/src/schema.rs` should show ~+25 lines across the 4 regions. COMMIT on `phase-m2-late-1`: `feat(db_schema): hand-extend schema.rs for sanction_event + sanction_subscriber (task 1)`.

**If ANYTHING diverges from this spec** (anchor grep returns unexpected context, fmt errors, an extra column in up.sql you don't see mapped) → STOP, raise a `kind: blocker` DQ, surface to user. Do not improvise the schema edit.

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
