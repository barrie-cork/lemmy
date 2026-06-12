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
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `e2cca7d19` (see §"Git state at handoff"); if drifted, `git -C C:/Users/barri/Developer/brehon-fork log --oneline e2cca7d19..governance-v0` and update mental model before acting.
3. Read `workflow_state_m2_late.md` (CLOSED record) once for carry-forward context — especially the three carry-forward items listed there.
4. Read `.claude/decision-queue.json` for any pending entries since handoff (empty at handoff per snapshot below).
5. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_m2_late_2.md` is the running-state scratchpad.

## Next concrete action

> **STATE UPDATED 2026-06-12 (advisor session).** Planning is DONE — brief authored,
> `/brehon-clarify` complete (4 resolved DQ `a3d0e9941441-058`…`-061`), planning Junior
> #660 ran 10:48→11:00 UTC, plan authored + merged to `phase-m2-late-2` @ **`c6a22e072`**
> (`.claude/PRPs/plans/m2-late-2.plan.md`, 7 tasks, complexity 6/10). DoD smoke + watchpoint
> gate + MiniMax-designation (SUSPENDED) all run. **Plan-approval gate 1 is HELD pending
> user read** (user chose "Hold — read the plan first" 2026-06-12). DQ pending = 0.

**Next concrete action for the NEW session:**
1. **Verify Docker is in Linux-container mode** (`docker info --format '{{.OSType}}'` → `linux`) — required for bridge validation (see the LINUX-BRIDGE decision below).
2. Confirm the user has finished reading `.claude/PRPs/plans/m2-late-2.plan.md` and gives plan-approval (gate 1).
3. On approval → `/auto-phase m2-late-2 --unattended --start-from impl-cohort-0`.

### ⚠️ LINUX-BRIDGE validation (user-confirmed 2026-06-12 — load-bearing)

Bridge cargo (Tasks 3/4/5) does **NOT** run via the Windows-local `cd services/bridge && cargo …`
form — that **FAILS on the Windows host** (`ruma-common v0.19.0` E0119 conflicting-trait-impl
vs `time`, a host-toolchain quirk reproduced on the phase base). **All bridge cargo runs via
Docker** through `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml`
(no wrapper change needed — it's a scope-agnostic passthrough). The plan's §15.4 carries the
authoritative **LINUX-BRIDGE RIDER** with the exact commands; R9 + Task 0 Probe 4 point to it.
FIRST bridge container run is COLD (~10–20 min, full `ruma`/`matrix-sdk` compile); warms after
via the `brehon-cargo-registry` volume. Workspace Tasks 1/2 still use the Windows bat wrappers
(they compile fine).

**Before first bm-triage dispatch:** implement test-dogfood retro Change #4 — update `bm-task-brief.template.md` bm-triage row to specify `model: sonnet-4-6`. 5× recurrence threshold met; one-line template change before the first triage brief is authored.

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

5. **Bridge `Cargo.lock` drift** — **CORRECTED 2026-06-12:** `services/bridge/Cargo.lock` is **NOT tracked and does NOT exist in the repo** (it is generated on first `cargo` build, then cleaned). The earlier "tracked lockfile" claim here and in §4.5 was wrong (verified: `ls services/bridge/Cargo.lock` → No such file). Practical consequence: the recommended plan adds **no new bridge dependency** (reuses `reqwest`/`serde_json`/`tokio`/`rusqlite`), so no lockfile change and **no `validate-pending-laptop-linux` gate fires for that reason**. The Linux gate is instead the *normal* bridge-validation path here because the bridge cannot compile on Windows at all (ruma E0119) — see the LINUX-BRIDGE decision in the RESUME block: bridge cargo runs via `cargo-linux.sh --manifest-path services/bridge/Cargo.toml` regardless of whether deps changed.

## 5. Operational rules

**Polling:** ~10 min cadence, `mcp__junior-brehon__list_tasks`. On transition: `show_task` + `git fetch origin` + read `.claude/decision-queue.json`.

**Brief discipline:** briefs at `.claude/PRPs/briefs/m2-late-2-<role>-<n>.md`. Committed on `governance-v0` (planning/bm briefs) or on the phase branch (impl-task briefs, Mode A). Pre-queue: `memory_search_hybrid` + `/precheck` + §2.4 mandatory file-class lesson injection (bridge crate edits trigger `feedback_features_full_p_crate_incompatible.md`; `governance_log` edits trigger `feedback_multi_write_handlers_need_transactions.md`).

**Shape G:** SUSPENDED — validate-pending-laptop per `advisor-orchestrator.md` §5.2. All cargo runs on laptop.

**Bridge cargo isolation (R9):** never `--workspace` for bridge checks. Execute via Docker-Linux: `scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml`. The Windows-local `cd services/bridge && cargo check` FAILS (ruma E0119) — do not use it (LINUX-BRIDGE decision, RESUME block).

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

- governance-v0 HEAD: `e2cca7d19` (updated 2026-06-12 at test-dogfood phase transition) — `docs(retro): append bm-triage #658 misclassification finding to test-dogfood retro`
- Phase branch HEAD: `not yet created (branch phase-m2-late-2 cut at bm-cut)`
- Recent governance-v0 commits:

  ```
  e2cca7d19 docs(retro): append bm-triage #658 misclassification finding to test-dogfood retro
  0ee51f20a docs(lessons): promote PowerShell ASCII-only code-strings trap to a lesson
  36e546b07 chore(advisor): execute retro action items 1-3 (narrow-the-ask + user-skills snapshot)
  da8724bc7 chore(advisor): bm-merge done — PR #195 @ 946293cbd; gate 5 confirmed; auto-state bm-merge-done
  946293cbd Merge pull request #195 from barrie-cork/phase-test
  ```

_Prior handoff hash `443c9d8bc` (2026-06-08) superseded by test-dogfood phase (`test`) which ran 2026-06-11→12 and landed 20+ commits on governance-v0. Significant additions: `advisor-orchestrator.md` §5.2 Shape-G-disabled fast-path; `refs/auto-phase.md` Race-A fetch-before-write; `bm-task-brief.template.md` YAML recovery; lesson `feedback_powershell_ascii_only_code_strings.md`. See retro `.claude/PRPs/reports/session-retro-2026-06-12-test-dogfood-bm-triage-leg.md` for full change list._

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff — 2026-06-12)
```

## Stop-and-ask tripwires

- Stop and ask if: the plan includes any pseudonym→Matrix-account linkage task — B-actor is out of scope (user-confirmed 2026-06-07); this is a scope violation requiring explicit re-approval.
- Stop and ask if: the CR-A fix requires adding a new `conn`-accepting variant of `governance_log::append` — this is a signature change touching the shared `governance_log` module and needs advisor review before authoring the impl-task brief.
- Stop and ask if: the bridge power-level enforcement task proposes to build a pseudonym→rooms index that requires a new DB table or migration — m2-late-2 should be handler-only; a migration is a scope escalation.
- Stop and ask if: the pilot has no active `sanction_subscriber` row — verifying the subscriber seed before dispatching enforcement tests prevents a "delivery succeeded but nobody received it" false-green.
- Stop and ask if: any fix-impl brief targets `crates/server/tests/e2e.rs` with ≥2 edits — this is the historically fragile file; apply `feedback_fix_impl_pre_locate_e2e_anchors.md` discipline and surface the anchor list to the user before dispatch.

---

## STATUS: SHIPPED

- **PR:** #196 merged at 2026-06-12T22:09:32Z
- **Merge SHA:** e1d615ee33907fa0d9fdeecba00576039c05d448
- **Tombstoned by:** bm-merge Phase 8.5 post-condition
- **This handover is stale.** Do NOT use the RESUME block above as a basis for action — the phase is complete. Delete this file or archive it.
