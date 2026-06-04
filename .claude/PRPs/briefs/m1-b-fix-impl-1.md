# Brief: m1-b fix-impl-1 (CR F2 — read_current deterministic tie-breaker)

## 1. Role + dispatch

`[role:impl-task]` m1-b fix-impl-1 — add deterministic tie-breaker to `GovernanceMessagingConfig::read_current` (CR finding F2 on PR #177). Single Sonnet arm.

## 2. Scope

CodeRabbit finding F2 (PR #177): `read_current` in `crates/db_schema/src/source/governance/governance_messaging_config.rs` orders only by `valid_from DESC`, which is **nondeterministic on ties** (two rows with the same `valid_from` timestamp can return in arbitrary order). The table is append-only audit history, so same-timestamp ties are possible. Add a stable secondary sort on `id DESC`.

**The fix (exactly one edit):** in `read_current`, between the existing `.order_by(...)` and `.first::<Self>(conn)`, add a `.then_order_by(governance_messaging_config::id.desc())` line.

Current (around line 85-86):
```rust
        .order_by(governance_messaging_config::valid_from.desc())
        .first::<Self>(conn)
```
Becomes:
```rust
        .order_by(governance_messaging_config::valid_from.desc())
        .then_order_by(governance_messaging_config::id.desc())
        .first::<Self>(conn)
```

**Commit ONLY** `crates/db_schema/src/source/governance/governance_messaging_config.rs` + `.claude/decision-queue.json` (the validate-pending DQ you raise). Do NOT touch any other file.

**Boundaries:** this is a 1-line additive change. Do NOT refactor `read_current`, do NOT change the filters or the `.optional()?` handling, do NOT touch any other function. The `id` column exists on the table (`id -> Int4` in schema.rs) — `.then_order_by(governance_messaging_config::id.desc())` is the correct Diesel idiom.

## 3. Required reading

- `crates/db_schema/src/source/governance/governance_messaging_config.rs` — the `read_current` fn (Edit target). Read it first; confirm the `.order_by(governance_messaging_config::valid_from.desc())` line is unique (`grep -c` == 1) before editing.
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write the DQ, push, STOP. Do NOT run cargo on the daemon.

## 4. Constraints

1. **One additive line.** Only add the `.then_order_by(governance_messaging_config::id.desc())` line. No other change to the file.
2. **Uniqueness gate.** Before the Edit, `grep -c '.order_by(governance_messaging_config::valid_from.desc())' crates/db_schema/src/source/governance/governance_messaging_config.rs` must return `1`. (It is the only `read_current` order clause.)
3. **validate-pending-laptop, write-then-stop.** After committing, write a `kind: "validate-pending-laptop"` DQ entry with:
   ```
   commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]
   ```
   Set `branch` = your worker branch, `phase_task: "fix-1"`. Commit + push the DQ, then **STOP**. Do NOT run cargo yourself — validation is delegated to the laptop advisor. Per `feedback_validate_pending_laptop_write_then_stop.md`.
4. **DQ id via helper.** Generate the id with `bash scripts/brehon/dq-v3-new-entry.sh` + append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Never hand-compute.
5. **Commit message.** `fix(db_schema): deterministic tie-breaker in messaging-config read_current (CR F2)`.
