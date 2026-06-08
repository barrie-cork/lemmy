---
phase: m2-late-2
plan: .claude/PRPs/plans/m2-late-2.plan.md   # not yet authored
phase_branch: phase-m2-late-2                  # not yet created until bm-cut
worktree: C:/Users/barri/Developer/brehon-fork-m2late2   # created at bm-cut; until then canonical brehon-fork
authored: 2026-06-08
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the m2-late-2 advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon m2-late-2** (bridge power-level enforcement + CR-A atomicity fix + pilot sanction-delivery verification). This is a fresh session (or a resumed one). The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-m2late2` once `bm-cut` creates the lane worktree (per `.claude/rules/multi-lane-worktree.md`).

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `443c9d8bc` (see §"Git state at handoff"); if drifted, `git -C C:/Users/barri/Developer/brehon-fork log --oneline 443c9d8bc..governance-v0` and update mental model before acting.
3. Read `workflow_state_m2_late.md` (CLOSED record) once for carry-forward context — especially the three carry-forward items listed there.
4. Read `.claude/decision-queue.json` for any pending entries since handoff (empty at handoff per snapshot below).
5. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_m2_late_2.md` is the running-state scratchpad.

## Next concrete action

Author `.claude/PRPs/briefs/m2-late-2-planning-1.md` (scope: power-level enforcement in bridge + CR-A atomicity fix; see §1 + carry-forward from `workflow_state_m2_late.md`). Then `/brehon-clarify` → queue planning Junior. **B-actor portable-ID linkage is OUT OF SCOPE** (user-confirmed 2026-06-07; OQ-ADR016-03 deferred).

---

## 1. m2-late-2 in one paragraph

m2-late-2 closes the two deliberate stubs left in m2-late-1: (a) the bridge power-level enforcement — `handle_sanction_event` returns `applied: false` because `services/bridge` lacks a pseudonym→rooms index; m2-late-2 builds that index (or a simpler case_id→rooms lookup) and calls Matrix `send_state_event` to apply the power-level change; (b) the CR-A atomicity gap — `enqueue_sanction_event`'s `sanction_event` INSERT and `governance_log::append` run on separate pool connections with no transaction, violating the ADR-008 "every governance write emits an audit log entry" invariant; fix: wrap both in a single fresh `conn.run_transaction()` inside `enqueue_sanction_event`. Additionally: pilot verification — confirm `BRIDGE_SANCTION_CALLBACK_URL` is set and `seed_sanction_subscriber` has seeded a live row in the pilot at `http://100.81.145.58:1236`. B-actor (portable-ID linkage) is out of scope.

## 2. Why m2-late-2 is harder than m2-late-1

**Harder:** The bridge power-level enforcement requires reading Matrix room state (which rooms is this pseudonym a member of?) — `services/bridge` is workspace-excluded (R9), has its own `Cargo.lock`, and uses `ruma`/`axum` not the Lemmy stack. The pseudonym→rooms index doesn't exist yet; either build a new in-memory map seeded from `bridge_room` + Matrix state, or use a simpler `case_id` lookup (the bridge room DB has `(case_id, room_type)` as the key — the sanction event carries `case_id` indirectly via `governance_log_entry_hash`). The right scope call is for the planner: full enforcement vs. a `case_id`-keyed partial enforcement.

**Not harder:** The CR-A atomicity fix is mechanical — `get_conn` + `run_transaction` wrapping two already-written inserts in `sanction_publisher.rs`. No new API surface, no migration, no e2e change. The pilot verification is a manual SSH + curl check.

## 3. Lessons from m2-late-1 that apply to m2-late-2

**Advisor-side:**
- `feedback_advisor_watchpoint_specificity.md` — watchpoints must cite specific files/tables. The bridge power-level path involves `services/bridge/src/sanction_handler.rs`, `services/bridge/src/appservice.rs` (AppState), and the Matrix client's `send_state_event` call. Name them in plan §4 watchpoints.
- `feedback_pre_phase_dod_smoke_test.md` — run every §15 DoD command at plan-approval time. The bridge crate is workspace-excluded; DoD commands must be scoped to `services/bridge/` separately (not `--workspace`).
- `feedback_linux_compile_proof_is_a_gate.md` — if bridge `Cargo.toml` or `Cargo.lock` changes, raise a `validate-pending-laptop-linux` DQ. The Option-2 scope trigger applies: bridge `Cargo.toml`/`Cargo.lock` changes = raise it.

**Impl-side:**
- `feedback_lemmy_error_no_std_error.md` — applies to any new `LemmyResult<()>` return in `sanction_publisher.rs`; use Case A (`.map_err(LemmyError::from)`).
- `feedback_multi_write_handlers_need_transactions.md` — the CR-A fix IS this pattern; the impl-task brief must explicitly cite this lesson.
- `feedback_features_full_workspace_only.md` + `feedback_features_full_p_crate_incompatible.md` — applies to `lemmy_api` cargo check; bridge is workspace-excluded so bridge checks must NOT use `--workspace --features full`.
- Bridge-crate is workspace-excluded (R9): all `cargo check`/`cargo build` for `services/bridge` must use `cargo check --manifest-path services/bridge/Cargo.toml` (or `cargo check` run from inside `services/bridge/`), never `--workspace`.

**BM-side:**
- `feedback_branch_manager_pm_split.md` — BM session is separate from impl; do not conflate.

## 4. m2-late-2-specific watchlist

1. **`services/bridge/src/sanction_handler.rs` `handle_sanction_event` stub**: currently returns `applied: false`. Enforcement requires: (a) a pseudonym→rooms lookup — check `services/bridge/src/` for any existing room-index structure (AppState fields, `bridge_room` table access, `matrix_client.get_joined_rooms()`); (b) `send_state_event` call with the correct `m.room.power_levels` content. Plan §13 must specify EXACTLY which Matrix state event shape and which power-level value for each `SanctionKind` variant.

2. **`crates/api/api/src/governance/sanction_publisher.rs` lines 178-207**: The CR-A fix wraps lines 185-203 (`insert_into(sanction_event_dsl::table)` + `governance_log::append`) in a single `conn.run_transaction(|conn| { ... })`. The transaction conn must be passed through to `governance_log::append` — check if `append` accepts `&mut AsyncPgConnection` or only `&mut DbPool<'_>`. If pool-only, the fix requires adding a `conn`-accepting variant of `append` or an inline insert. Plan must resolve this before task authorship.

3. **`governance_log::append` signature** (`crates/api/api/src/governance/governance_log.rs` + `crates/db_schema/src/source/governance/governance_log.rs`): verify the function signature accepts a transactional connection or only a pool. This is the pivotal question for CR-A scope — pool-only means a new helper; conn-accepting means a simple wrapper.

4. **Pilot `sanction_subscriber` row**: SSH to `http://100.81.145.58:1236` and verify `SELECT * FROM sanction_subscriber WHERE active = true` returns exactly one row with the bridge callback URL. If missing, `BRIDGE_SANCTION_CALLBACK_URL` was not set at startup — the pilot deployment config needs patching before end-to-end testing is possible.

5. **Bridge `Cargo.lock` drift**: if any `Cargo.toml` dependency changes in `services/bridge/`, the `Cargo.lock` will change. The `services/bridge/Cargo.lock` is tracked (R9: bridge is workspace-excluded, has its own lockfile). Any `Cargo.lock` change triggers the `validate-pending-laptop-linux` DQ gate (Option-2 scope per `feedback_linux_compile_proof_is_a_gate.md`).

## 5. Operational rules

**Polling:** ~10 min cadence, `mcp__junior-brehon__list_tasks`. On transition: `show_task` + `git fetch origin` + read `.claude/decision-queue.json`.

**Brief discipline:** briefs at `.claude/PRPs/briefs/m2-late-2-<role>-<n>.md`. Committed on `governance-v0` (planning/bm briefs) or on the phase branch (impl-task briefs, Mode A). Pre-queue: `memory_search_hybrid` + `/precheck` + §2.4 mandatory file-class lesson injection (bridge crate edits trigger `feedback_features_full_p_crate_incompatible.md`; `governance_log` edits trigger `feedback_multi_write_handlers_need_transactions.md`).

**Shape G:** SUSPENDED — validate-pending-laptop per `advisor-orchestrator.md` §5.2. All cargo runs on laptop.

**Bridge cargo isolation (R9):** never `--workspace` for bridge checks. Use `cargo check --manifest-path services/bridge/Cargo.toml` or `cd services/bridge && cargo check`.

**Windows e2e:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1"` + `run_in_background: true`. Read exit marker from log, not task notification. See `feedback_windows_e2e_requires_bat_wrapper.md`.

**Cohort cap:** ≤2 concurrent Junior tasks total (cross-lane). Pre-dispatch `list_tasks(status="running")` count.

**Model tiering:** Planning → Opus, Impl → Sonnet, BM/ci-watcher → Haiku.

**Clarify gate:** `/brehon-clarify` on planning brief before queueing planning Junior.

**Six user gates:** (1) plan approval, (2) judgment-heavy DQ, (3) CR triage, (4) Phase 2 e2e local vs dispatch, (5) merge confirm, (6) retro sign-off.

**DQ attribution:** advisor commits must match `^(chore|docs)\((advisor|decision-queue)\)`.

**Memory headroom:** no bulk e2e.rs reads; subagent delegation for multi-probe operations.

**R8 (CRITICAL):** `enqueue_sanction_event` runs OUTSIDE any vote transaction. The CR-A fix uses a FRESH transaction inside `enqueue_sanction_event` itself — never touches the vote transaction conn.

**R9 (CRITICAL):** `services/bridge` stays in root `Cargo.toml` `exclude` array. Never add to `members`.

## 6. What changed from m2-late-1's rule set

- **Bridge cargo isolation rule is now explicit** (was implicit from R9 in m2-late-1; now a named operational rule in §5 above).
- **CR-A atomicity fix is in scope** (was carry-forward in m2-late-1; now a primary deliverable).
- **B-actor is definitively out of scope** (user-confirmed 2026-06-07; OQ-ADR016-03 deferred). No pseudonym→Matrix-account linkage in m2-late-2.
- **Pilot verification is a new deliverable** (m2-late-1 wired the seed; m2-late-2 verifies it works against the live pilot instance).

## 7. Catch-fire procedures

Universal triggers from `.claude/rules/advisor-orchestrator.md` §5.6, plus:

- **impl-task edits `services/bridge/` and uses `--workspace`** for cargo → catch-fire (R9 violation; workspace check won't find bridge crates).
- **impl-task calls `enqueue_sanction_event` changes that touch the vote transaction's `conn`** → catch-fire (R8 violation; fire-and-forget must use a fresh pool connection).
- **plan introduces B-actor (pseudonym→Matrix-account linkage) in scope** → catch-fire (user-confirmed out of scope 2026-06-07; surface to user before planning completes).
- **`governance_log::append` is changed to accept a transactional conn AND the change is not atomic with the CR-A fix** → catch-fire (schema-retrofit without co-commit violates `pattern_spec_schema_co_commit.md`).
- **Junior subagent commits to `governance-v0` directly** → catch-fire (phase-branch discipline).
- **`bm-task` opens PR into `main`** → catch-fire (use `governance-v0`).

## 8. Archive after m2-late-2

Run `/brehon-phase-transition m2-late-2 <next-id>`. The skill will: close `workflow_state_m2_late_2.md`, delete `workflow_state_m2_late.md` (the two-ago at that point), create the next skeleton, write the next bootstrap file, update brehon-fork MEMORY.md, commit on governance-v0. This file (m2-late-2-bootstrap.md) stays in `.claude/PRPs/handovers/` — git history is the archive.

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `443c9d8bc` (captured 2026-06-08) — `docs(retro): m2-late-1 retro — B-publish sanction propagation`
- Phase branch HEAD: `not yet created (branch phase-m2-late-2 cut at bm-cut)`
- Recent governance-v0 commits:

  ```
  443c9d8bc docs(retro): m2-late-1 retro — B-publish sanction propagation
  be8134d0b Merge pull request #192 from barrie-cork/phase-m2-late-1
  991e35de5 fix(governance): CR round-2 fix-in-pr — reqwest timeout + sanction_handler doc (PR #192)
  35ad84565 fix(comparator): digest_plan regex matches unnumbered section headings
  5d0c28544 chore(merge): merge-forward governance-v0 into phase-m2-late-1 pre-merge (fix-impl-1 complete)
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff)
```

## Stop-and-ask tripwires

- Stop and ask if: the plan includes any pseudonym→Matrix-account linkage task — B-actor is out of scope (user-confirmed 2026-06-07); this is a scope violation requiring explicit re-approval.
- Stop and ask if: the CR-A fix requires adding a new `conn`-accepting variant of `governance_log::append` — this is a signature change touching the shared `governance_log` module and needs advisor review before authoring the impl-task brief.
- Stop and ask if: the bridge power-level enforcement task proposes to build a pseudonym→rooms index that requires a new DB table or migration — m2-late-2 should be handler-only; a migration is a scope escalation.
- Stop and ask if: the pilot has no active `sanction_subscriber` row — verifying the subscriber seed before dispatching enforcement tests prevents a "delivery succeeded but nobody received it" false-green.
- Stop and ask if: any fix-impl brief targets `crates/server/tests/e2e.rs` with ≥2 edits — this is the historically fragile file; apply `feedback_fix_impl_pre_locate_e2e_anchors.md` discipline and surface the anchor list to the user before dispatch.
