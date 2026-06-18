---
role: impl-task
phase: m3-core-infra
task: 1
parallel: true
cohort: [1, 2]
base_branch: phase-m3-core-infra
created: 2026-06-18
---

# Impl-task brief — m3-core-infra Task 1 [P]: `rtc_enabled` config-row seed + `BridgeStatus.rtc_enabled`

**Role:** `[role:impl-task]`
**Dispatch:** `[role:impl-task] m3-core-infra task1 rtc-enabled-seed — see .claude/PRPs/briefs/m3-core-infra-impl-1.md`
**Base branch:** `phase-m3-core-infra`
**Plan:** `.claude/PRPs/plans/m3-core-infra.plan.md` Task 1 + §10.1/§10.2/§10.3

---

## 1. Role + dispatch line

You are the **impl-task** subagent (Sonnet 4.6 — pattern-following from MIRROR refs). Execute plan Task 1, ONE commit, run the validation gate, write the `validate-pending-laptop` DQ, then **STOP**.

This is a `[P]` cohort member (cohort = Task 1 ∥ Task 2). Task 2 modifies `services/bridge/src/bridge_room.rs` only — **zero file overlap** with this task. Do not touch any bridge file.

---

## 2. Scope

Seed an `rtc_enabled=false` instance row into `governance_messaging_config` (DATA, not schema) and expose it on the bridge-read `BridgeStatus`.

**FILES (exhaustive — touch nothing else):**

```yaml
creates:
  - migrations/2026-06-18-000000-0000_seed_rtc_enabled_config/up.sql
  - migrations/2026-06-18-000000-0000_seed_rtc_enabled_config/down.sql
modifies:
  - crates/api/api/src/governance/bridge_read.rs   # add rtc_enabled to BridgeStatus + read in get_bridge_messaging_status
```

**IMPLEMENT (file 1 of 3) — `up.sql`** per §10.1, verbatim shape:

```sql
INSERT INTO governance_messaging_config (scope, key, value_type, value_int, value_bool, value_text, valid_from) VALUES
    ('instance', 'rtc_enabled', 'bool', NULL, false, NULL, '2026-06-18T00:00:00Z'::timestamptz)
ON CONFLICT (scope, key, valid_from) DO NOTHING;
```

**IMPLEMENT (file 2 of 3) — `down.sql`:**

```sql
DELETE FROM governance_messaging_config WHERE scope='instance' AND key='rtc_enabled' AND valid_from='2026-06-18T00:00:00Z'::timestamptz;
```

**IMPLEMENT (file 3 of 3) — `bridge_read.rs`** per §10.2:
- Add `pub rtc_enabled: bool,` to the `BridgeStatus` struct.
- In `get_bridge_messaging_status`, after the `oq009_reveal_threshold` read, add:
  ```rust
  let rtc_enabled =
    GovernanceMessagingConfig::read_current(pool, "instance", "rtc_enabled")
      .await?
      .and_then(|r| r.value_bool)
      .unwrap_or(false);   // absent → false (clean-posture default)
  ```
- Set `rtc_enabled` in the `BridgeStatus { … }` constructor.

**Commit:** `feat(governance): seed rtc_enabled config row + expose on BridgeStatus (task 1)`

**Do NOT:**
- Run `diesel print-schema` / regen `schema.rs` — this is a ROW seed, NOT a schema change. No `CREATE`/`ALTER`.
- Touch `services/bridge/**` (Task 2's lane), `governance_log.rs`, or any file not in the FILES list.
- Run cargo on the daemon (see §4).

---

## 3. Required reading

1. `.claude/PRPs/plans/m3-core-infra.plan.md` — Task 1 + §10.1, §10.2, §10.3 (the MIRROR refs)
2. `crates/api/api/src/governance/bridge_read.rs:15-46` — the BridgeStatus struct + read pattern to mirror
3. `crates/db_schema/src/source/governance/governance_messaging_config.rs:75-91` — `read_current` signature
4. `migrations/2026-06-03-000000-0000_add_governance_messaging_config/up.sql` (~line 52) — the seed block to mirror
5. `.claude/lessons/feedback_lemmy_migration_runner.md` — migration runner invocation (`cargo run -p lemmy_diesel_utils --features full`, NO sub-command args)
6. `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write the DQ, then STOP; do NOT run cargo on the daemon
7. `.claude/rules/decision-queue.md` "Mid-task visibility" — commit+push DQ on the worker branch

### 3a. Handover from prior cohort

(none — first cohort)

---

## 4. Constraints

- **Row-seed, not schema:** pin `valid_from` to the stable literal `'2026-06-18T00:00:00Z'` so `diesel migration redo` is idempotent. `ON CONFLICT DO NOTHING` must be a true no-op on rerun. Default-on-absent is `false` (clean-posture — RTC is off unless an admin flips it).
- **NO CARGO ON THE DAEMON.** Write a `validate-pending-laptop` DQ entry with:
  ```
  commands: [
    "cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full\"",
    "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings\""
  ]
  ```
  plus `branch: "<your worker branch>"`, `phase_task: 1`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Commit + push the DQ on your worker branch, then **STOP**. The laptop advisor runs the cargo + migration round-trip and mutates the entry. Do NOT run `cargo-check.bat`, `migrate-roundtrip.sh`, or any cargo yourself.
- **One commit** for the impl (3 files). The DQ commit is separate (`chore(decision-queue): impl raised validate-pending — rtc-enabled-seed`).
- **DQ mid-task discipline:** if blocked, write a `kind: "blocker"` DQ (`from: "impl"`), commit + push on the worker branch, stop.
- **LESSON trailer:** end the impl commit body with `LESSON: <one-line>` if you learn something durable; else omit.

---

## 5. Success signals (advisor verifies)

- `up.sql` + `down.sql` exist with the exact INSERT/DELETE shapes above.
- `bridge_read.rs` `BridgeStatus` has `pub rtc_enabled: bool` AND the constructor sets it from `read_current(...).unwrap_or(false)`.
- `validate-pending-laptop` DQ entry pushed on the worker branch (`phase_task: 1`).
- No cargo run on the daemon; no `schema.rs` regen; no bridge files touched.

---

## HANDOVER

```yaml
HANDOVER:
  task: m3-core-infra-task1
  filesCreated: [migrations/2026-06-18-000000-0000_seed_rtc_enabled_config/up.sql, migrations/2026-06-18-000000-0000_seed_rtc_enabled_config/down.sql]
  filesModified: [crates/api/api/src/governance/bridge_read.rs]
  keyDecisions:
    - "rtc_enabled seeded as instance row, value_bool=false, valid_from pinned 2026-06-18T00:00:00Z"
    - "BridgeStatus.rtc_enabled reads read_current(instance,rtc_enabled).value_bool.unwrap_or(false)"
    - "row-seed migration — NO schema.rs regen, NO CREATE/ALTER"
  notes: "<fill in: worker branch, validate-pending-laptop DQ id>"
```
