---
phase: m2-rooms-a
plan: .claude/PRPs/plans/m2-rooms-a.plan.md   # not yet authored — planning Junior must run first
phase_branch: phase-m2-rooms-a                  # not yet created until bm-cut
worktree: C:/Users/barri/Developer/brehon-fork-m2ra   # created at bm-cut; until then canonical brehon-fork
authored: 2026-06-06
authored_by: advisor (brehon-fork-m2 lane / governance-v0 session)
purpose: Bootstrap the m2-rooms-a advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon m2-rooms-a.** This is a fresh session (or a resumed one). The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-m2ra` once `bm-cut` creates the lane worktree (per `.claude/rules/multi-lane-worktree.md`). There is no homeserver session.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `b7fb90c4c` (see §"Git state at handoff"); if drifted, `git -C C:/Users/barri/Developer/brehon-fork log --oneline b7fb90c4c..governance-v0` and update your mental model before acting.
3. Read `.claude/decision-queue.json` (and `scripts/brehon/resolve-dq-canonical.sh m2-rooms-a` once a phase branch exists) for any pending entries since handoff; compare against §"Decision-queue snapshot" below (empty at handoff).
4. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_m2_rooms_a.md` is the running-state scratchpad. Read `workflow_state_m2_core_hook.md` ONCE for carry-forward (then don't re-read).
5. **Pilot server** — the fork is live at `http://100.81.145.58:1236` (admin `lemmy`/`lemmylemmy`). No governance `.wasm` plugins yet; the `governance_case_after_transition` hook from m2-core-hook fires into the bridge but the bridge has NO provisioning logic yet (that's what m2-rooms-a builds). A test case can be created and transitioned to confirm the hook fires (check bridge logs) but no rooms will be provisioned until m2-rooms-a lands.

## Next concrete action

Author `.claude/PRPs/briefs/m2-rooms-a-planning-1.md` (scope = M2-core Phases 3–5 per `.claude/PRPs/prds/m2-governance-triggered-rooms.prd.md` §"Implementation Phases": bridge-side provisioning service + `bridge_room` table + OQ-009 reveal + hash-chain emission + restart-idempotency + integration tests) → `/brehon-clarify` → queue planning Junior.

Run `/precheck` before any Junior dispatch. The m2-core-hook lane worktree (`brehon-fork-m2`) can be removed after this transition commit pushes: `git -C C:/Users/barri/Developer/brehon-fork worktree remove ../brehon-fork-m2 --force && git -C C:/Users/barri/Developer/brehon-fork branch -d phase-m2-core-hook`.

---

## 1. m2-rooms-a in one paragraph

m2-rooms-a delivers the **bridge-side room provisioning service** — the second half of M2-core. m2-core-hook (just shipped, PR #184) wired the binary-side `governance_case_after_transition` hook at all 11 `CaseStatus` transition sites and defined the 10 `ENTRY_KIND_ROOM_*` consts + the `append_room_event()` binary-side callback seam. m2-rooms-a adds: (Phase 3) the `services/bridge/` provisioning service that receives hook payloads and provisions/archives Matrix rooms for C2.1–C2.6 scenarios (jury / community-event / spin-out / appeal / emergency / membership-mirror), pins `always_pseudonym` for jury+appeal, implements OQ-009 graduated juror-handle reveal, and tracks state in a new `bridge_room` table (bridge-local store, not Brehon workspace); (Phase 4) the bridge's HTTP callback into `append_room_event()` to write `Room::*` entries onto the hash chain with restart-idempotency via last-seen row id; (Phase 5) the integration test suite asserting jury room <5s, emergency <2s, 10 hash-chain entries per lifecycle, no duplicate `Room::Created` on restart, and clean posture with `messaging_enabled=false`. DoD: all §Success Criteria in the PRD pass; `cargo build --workspace` still pulls zero Matrix deps; the pilot server at `http://100.81.145.58:1236` provisions a real jury room on a real `JurySelection` transition.

## 2. Why m2-rooms-a is easier/harder than m2-core-hook

- **Easier:** all the in-binary hook sites are already wired. No changes to any `crates/api/api/src/governance/*.rs` handler decision logic. The hash-chain `append()` writer is production-tested. `governance_messaging_config` KV table needs no schema change. Zero-migration for the Brehon workspace (the `bridge_room` table is bridge-local).
- **Not easier (harder):** the bridge lives in `services/bridge/` — workspace-excluded, different toolchain context, no Diesel, no Lemmy error types. The validate-pending-laptop command is `cd services/bridge && cargo check`, NOT `--workspace --features full`. OQ-009's graduated-reveal logic needs careful implementation (reveal `Juror-<suffix>` to fellow jurors only after ≥1 posted comment in the room — room-membership rendering at a threshold boundary). The `bridge_room` idempotency-by-`case_id` + last-seen-governance_log-row-id requires a restart-recovery path. Integration tests involve live Matrix API calls against Tuwunel (the pilot server) and testcontainers for the Brehon DB — richer test setup than the m2-core-hook e2e suite. The bridge-daemon Cargo conventions (OQ-V2-10: Tuwunel, ruma-appservice-api 0.16, matrix-sdk 0.18) are external to the Lemmy lesson corpus.

## 3. Lessons from m2-core-hook that apply to m2-rooms-a

Carry forward (reference by filename, do not duplicate):

- **Advisor-side:**
  - `feedback_verify_automated_reviewer_claims_against_compiler.md` — two CR findings were wrong on PR #184 (cr-7 falsified, cr-4 fix non-compiling); compile-check every trait/type/control-flow claim AND verify the proposed fix compiles, not just the finding.
  - `feedback_bm_merge_unstable_admin_bypass.md` — `UNSTABLE` on the unprotected `governance-v0` is expected; `--admin` bypass after local-scan is correct, not a catch-fire.
  - The **lossless DQ-merge reconcile discipline** — when a merge-forward touches `decision-queue.json`, scan both arrays for stale pending copies after resolving; the duplicate-pending shape recurs.
  - `feedback_cherry_pick_onto_restructured_file_reinjects_content.md` (PMD #831) — if a cherry-pick targets a file that has been decomposed into an `include!` host, verify the resulting diff is delta-only (not a re-injection). Always: `git show <sha> -- <file> | wc -l` before cherry-picking to a restructured target.

- **Impl-side:**
  - `feedback_validate_pending_laptop_write_then_stop.md` — workers write DQ `validate-pending-laptop` and STOP; do NOT run cargo on the daemon. For Tree-A (bridge crate), the command is `["cd services/bridge && cargo check"]`.
  - Scope impl `LESSON:` trailers to their producer context — a serialization choice that is correct for a log line may not be correct for an external wire payload (the cr-4 lesson). State the consumer.

- **Do NOT apply** the Lemmy-workspace lessons (`feedback_lemmy_error_no_std_error.md`, `feedback_multi_write_handlers_need_transactions.md`, Diesel/migration lessons, `feedback_features_full_workspace_only.md`) to `services/bridge/**` — wrong toolchain. Apply the m1-a bridge-context lessons instead (`feedback_async_pool_test_pattern.md` may apply for the bridge's own integration tests; verify).

## 4. m2-rooms-a-specific watchlist

- **`cargo build --workspace` zero-Matrix-deps invariant** (`crates/Cargo.toml` `exclude = ["services/bridge"]`): after every task that touches `services/bridge/`, run `cargo tree --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma'` — must be `0`. Gate for bm-pr. Story-6 invariant from M1; a regression here is a catch-fire.
- **`bridge_room` table idempotency key** (`services/bridge/` bridge-local store, schema to be designed at plan time): the `case_id` + `room_type` pair must be unique; `Room::Created` must fire exactly once per `case_id` even on bridge restart. The planner must spec the `last_seen_governance_log_row_id` recovery path in §13 VALIDATE before any impl task ships.
- **`crates/db_schema/src/source/governance/governance_log.rs:239-314`** (`append()` call signature): the `append_room_event()` wrapper from m2-core-hook is the binary-side seam. The bridge calls it via HTTP callback (the `notify_if_enabled` contract). Verify the callback URL + payload schema matches what the bridge sends before authoring any bridge-side provisioning impl-task brief.
- **OQ-009 graduated reveal** (`crates/api/api/src/governance/messaging_config.rs:69-83` + `actor_pseudonym_helper.rs:26-80`): `Juror-<suffix>` visible to fellow jurors only after ≥1 posted comment (admin-configurable threshold, default 1). The membership-rendering threshold check is bridge-side; the pseudonym allocator is binary-side (called via existing API). Plan must spec the threshold lookup path (from `governance_messaging_config` KV table).
- **Emergency-room timing** (`admin_emergency_remove.rs` hook site): the PRD requires <2s provisioning. The bridge provisioning path must be non-blocking to the caller and must not add latency to the `emergency_remove` HTTP response. Verify at plan time that the hook is fire-and-forget (it is in m2-core-hook — confirm the bridge-side consumer is async).

## 5. Operational rules

- Polling cadence ~10 min (`mcp__junior-brehon__list_tasks`). Briefs in `.claude/PRPs/briefs/m2-rooms-a-<role>-<n>.md`, committed to `governance-v0` first; impl-task briefs synced to `phase-m2-rooms-a` (Mode A direct in lane worktree `brehon-fork-m2ra`, or Mode B trunk→phase from canonical).
- **Shape: pre-Shape-G** (cargo on laptop). validate-pending-laptop command for bridge tasks = `["cd services/bridge && cargo check"]`; for workspace-scope tasks (if any) = `["./scripts/brehon/cargo-check.sh --workspace --features full"]`. Do NOT mix. Linux-compile gate: re-evaluate at bm-pr — if the phase diff adds no new workspace migration and no `cfg(unix)` code, the Option-2 trigger does not fire (same reasoning as m1-a).
- **Bridge toolchain context (R8):** `services/bridge/**` tasks use matrix-sdk 0.18 + ruma-appservice-api 0.16 external conventions, NOT Lemmy conventions. Mandatory-lesson-injection table mostly does NOT apply to `services/bridge/**` files. Lean on external-convention reading (Ref MCP: matrix-sdk/ruma docs) and read 1-2 sibling bridge files before authoring each brief (§3.6 Tier-2 source-file read discipline).
- **Pre-queue mandatory lesson check** (`memory_search_hybrid` + §2.3 hybrid PMD search + `/precheck`) before every brief. For `services/bridge/**` tasks, search PMD for "bridge", "matrix-sdk", "ruma", "room provisioning" — the m1-a session may have authored bridge-context lessons.
- Model tiering: Planning→Opus, Impl→Sonnet, BM/ci-watcher→Haiku. The 6 user gates apply. DQ attribution `chore|docs(advisor|decision-queue):`. Memory headroom: no bulk file reads.
- **Pilot server** (`http://100.81.145.58:1236`): can be used for smoke-testing after m2-rooms-a merges. Bridge process is running on homeserver (PID 1282402 at m2-core-hook ship). Integration tests use testcontainers (own Docker network), not the pilot server — keep them separate.
- **MiniMax trial:** SUSPENDED (`project_minimax_key_rotate_after_m1b_trial.md`). Do NOT dispatch MiniMax arms until infra investigated + key rotated.

## 6. What changed from m2-core-hook's rule set

- **Scope moves from in-binary to bridge-side.** The Lemmy crate lessons (Diesel, LemmyResult, `--features full`, `e2e.rs` edit discipline) are largely OUT of scope for m2-rooms-a tasks. `services/bridge/**` is the primary impl target.
- **validate-pending-laptop command is `cd services/bridge && cargo check`**, NOT `--workspace --features full`.
- **Linux-compile gate likely non-binding** — re-evaluate at bm-pr (no workspace migration expected).
- **Integration tests involve live Matrix API** — richer than the cargo-unit-only e2e suite in m2-core-hook. The plan must spec the testcontainers + Tuwunel setup for bridge integration tests.
- **No §16a retrofit yet** — plans on M2-track still predate the §16a stories convention; `/brehon-verify` runs manual reconciliation. Note in planning brief if the planner can add §16a blocks.

## 7. Catch-fire procedures

Universal triggers from `.claude/rules/advisor-orchestrator.md` §5.6 apply. m2-rooms-a additions:

- **A task adds `services/bridge` to the workspace `members` array** (instead of `exclude`) → catch-fire (breaks zero-Matrix-deps invariant, M1 story-6).
- **`cargo build --workspace` pulls any `matrix-sdk` or `ruma` dep** after a bridge-side task → catch-fire (verify with `cargo tree --workspace | grep -cE 'matrix-sdk|ruma'`; should be `0`).
- **`append_room_event()` callback URL or payload schema doesn't match what the bridge sends** → surface to user before dispatching any bridge provisioning impl-task (integration seam mismatch is harder to fix post-impl than pre).
- **`Room::Created` fires more than once for the same `case_id`** in a restart-idempotency test → catch-fire (the recovery path is broken; this is an acceptance-criterion failure, not a clippy nit).
- Non-allowlist §G4 failure (any `error[E*]` other than `E0432`, test failures, OOM, timeout) → catch-fire per advisor-orchestrator §5.3.

## 8. Archive after m2-rooms-a

Run `/brehon-phase-transition m2-rooms-a <next-id>` (next is likely `m2-rooms-b` if provisioning splits, or `m2-e2e` for the integration test phase, or `m2-late` — determine at m2-rooms-a ship based on remaining M2-core Phases 4–5 scope). This skill will: close `workflow_state_m2_rooms_a.md`, delete `workflow_state_m2_core_hook.md` (two-ago), create the next skeleton, write the next bootstrap, update brehon-fork MEMORY.md, commit on governance-v0. This bootstrap file stays in `.claude/PRPs/handovers/` as its own archive — git history is the archive; no move.

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `b7fb90c4c` (captured 2026-06-06) — `docs(retro): m2-core-hook phase retro — PR #184 shipped, trunk corruption cleared`
- Phase branch HEAD: not yet created (branch `phase-m2-rooms-a` cut at bm-cut)
- Recent governance-v0 commits (`git -C C:/Users/barri/Developer/brehon-fork log --oneline -5 governance-v0`):

  ```
  b7fb90c4c docs(retro): m2-core-hook phase retro — PR #184 shipped, trunk corruption cleared
  7468379b3 chore(advisor): m2-core-hook bm-merge complete — PR #184 merged @ 704a6ac45
  704a6ac45 Merge pull request #184 from barrie-cork/phase-m2-core-hook
  c48174638 fix(api): serialize CaseTransitionEvent.target_type as enum not Debug (CR cr-4)
  4d0164eed chore(advisor): m2-core-hook PR opened #184 (advisor-inline bm-pr)
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff)
```

## Stop-and-ask tripwires

- Stop and ask if: any task proposes adding `services/bridge` to the workspace `members` array in root `Cargo.toml` — it MUST stay in `exclude` (zero-Matrix-deps invariant from M1 story-6).
- Stop and ask if: the plan proposes any schema migration under `crates/db_schema/migrations/**` — m2-rooms-a should have zero workspace migrations; the `bridge_room` table lives in the bridge-local store only.
- Stop and ask if: the `append_room_event()` HTTP callback URL or payload schema in the bridge provisioning impl does not match the signature in `crates/api/api_utils/src/` from m2-core-hook — verify the seam before dispatching bridge-side impl tasks.
- Stop and ask if: a bridge-side task applies a Lemmy-workspace lesson (LemmyResult, Diesel, `cargo --features full`, the `e2e.rs` harness) to `services/bridge/**` — wrong toolchain (R8).
- Stop and ask if: the Tuwunel pilot server at `http://100.81.145.58:1236` is not reachable when integration tests require it — verify the bridge process is still running (`ssh homeserver 'ps aux | grep tuwunel'`) before dispatching integration-test impl tasks.
