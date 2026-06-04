---
phase: m1-a
plan: .claude/PRPs/plans/m1.plan.md   # Tree A = §13 Tasks 8–13; a dedicated m1-a.plan.md is NOT yet authored — confirm whether the existing m1.plan.md Tree-A subset suffices
phase_branch: phase-m1-a              # not yet created until bm-cut
worktree: C:/Users/barri/Developer/brehon-fork-m1a   # created at bm-cut; until then canonical brehon-fork (or drive Mode-B from canonical)
authored: 2026-06-04
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the m1-a advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon m1-a (M1 Tree A — the greenfield `services/bridge/` Matrix bridge crate, Tasks 8–13).** This is a fresh session. The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-m1a` once `bm-cut` creates the lane worktree. There is no homeserver session.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `a7d1112d0` (see §"Git state at handoff"); if drifted, `git -C ... log --oneline a7d1112d0..governance-v0` and update your mental model before acting.
3. Read `.claude/decision-queue.json` (and `scripts/brehon/resolve-dq-canonical.sh m1-a` once a phase branch exists) for any pending entries since handoff; compare against §"Decision-queue snapshot" below (was empty at handoff).
4. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_m1_a.md` is the running-state scratchpad. Read `workflow_state_m1_b.md` ONCE for carry-forward (then don't re-read).
5. **MiniMax key rotation** — check whether the user has rotated it (carried open from m1-b; precondition met). If not done, re-surface it. It's tracked in `pending_end_of_phase_actions` (m1-b) + `project_minimax_key_rotate_after_m1b_trial.md`.

## Next concrete action

**First decide: does m1-a need a fresh plan?** `m1.plan.md` ALREADY contains Tree A (§13 Tasks 8–13) with FILES/IMPLEMENT/MIRROR/GOTCHA/VALIDATE per task. Two paths:
- **(a)** If the existing `m1.plan.md` Tree-A subset is sufficient → skip planning, go straight to `bm-cut phase-m1-a`, then dispatch Task 8 (bridge crate skeleton + workspace-exclude).
- **(b)** If you want a dedicated `m1-a.plan.md` (e.g. to re-shape Tasks 8–13 with the matrix-sdk research fresh) → author `.claude/PRPs/briefs/m1-a-planning-1.md` → `/brehon-clarify` → queue planning Junior.

Recommend (a) unless the Tree-A tasks need re-shaping — the plan already exists and was approved at gate-1. Either way, **/auto-phase M1 with `--start-from bm-cut-running` (or a fresh m1-a invocation) is NOT the path** — M1's auto-state is at `stage: phase-transition` for Tree B; Tree A is a new run. Consider a fresh `/auto-phase m1-a` once a plan path is settled (it will init a new auto-state file), OR drive manually.

---

## 1. m1-a in one paragraph
M1 Tree A: the greenfield **`services/bridge/`** crate — a Matrix application-service bridge, **workspace-EXCLUDED** (not in the `members` glob; added to `exclude`). Tasks 8–13: (8) crate skeleton + `exclude` + `BridgeConfig`; (9) AS transaction server (axum + ruma-appservice-api 0.16 — `PUT /transactions`, `GET /users`, `GET /rooms`, hs_token auth); (10) puppet-on-first-contact (matrix-sdk register/login, bridge-local `messaging_user_id` map); (11) 1:1 DM relay (text/image/voice both directions, Matrix media repo); (12) manual room provisioning + soft-pause drain (OQ-V2-09); (13) Tree C docker-compose (Tuwunel + bridge) + chat-plane design docs. DoD: the bridge compiles inside `services/bridge` (`cd services/bridge && cargo check`), `cargo build --workspace` pulls ZERO Matrix deps, and §16a stories 1/4/6 verify here.

## 2. Why m1-a is easier/harder than m1-b
- **Easier:** no migrations, no Diesel, no `--features full` gymnastics, no governance-log/transaction concerns. Each task is an additive new file in an isolated crate; no risk to the Lemmy workspace.
- **Not easier (harder, actually):** this is the FIRST greenfield-crate phase — there is NO in-repo MIRROR sibling (R8). The impl agents must read matrix-sdk 0.18 + ruma-appservice-api 0.16 quickstarts (Ref MCP) and follow EXTERNAL conventions, not Lemmy ones. The 4 Tuwunel "verify before committing" items (Task 9) are real integration gotchas with no compile-time signal. Validation is `cd services/bridge && cargo check` (NOT workspace cargo) — different invocation shape.

## 3. Lessons from m1-b that apply to m1-a
Carry forward (reference by filename, do not duplicate):
- **Advisor-side:** exit-marker verification on every bg cargo (`feedback_background_task_notification_lies.md`) — STILL applies for `cd services/bridge && cargo check` runs; verify the marker, not the harness "exit 0". Read CR findings from `gh pr view` directly, NOT the ephemeral `pr-<N>-findings.yaml` ([[issue_note_worker_redundant_daemon_cargo]] retro item #4). Compile-check CR claims before triaging (`feedback_verify_automated_reviewer_claims_against_compiler.md`).
- **Impl-side:** the worker must NOT run cargo on the daemon after committing — write-DQ-and-stop (`feedback_validate_pending_laptop_write_then_stop.md` + [[issue_note_worker_redundant_daemon_cargo]]). For Tree A, the validate-pending-laptop command is `["cd services/bridge && cargo check"]` (inside-bridge) + a workspace zero-Matrix-deps assertion.
- **Do NOT apply** the Lemmy-workspace cargo lessons (`feedback_features_full_workspace_only.md`, `feedback_lemmy_error_no_std_error.md`, Diesel/migration lessons) — Tree A is a different toolchain context (R8). `feedback_read_canonical_before_writing_spec.md` Tier-2 (read 1-2 sibling files first) becomes "read matrix-sdk/ruma examples first" since there's no in-repo sibling.

## 4. m1-a-specific watchlist
- **`exclude = ["services/bridge"]` in root `Cargo.toml`** (Task 8) — MUST be the `exclude` array, not just absence from `members`. §16a story 6 gate: `cargo tree --workspace 2>/dev/null | grep -c -E 'matrix-sdk|ruma'` MUST be `0`. Verify with `cargo tree` post-add (m1.plan.md §13 Task 8 GOTCHA).
- **`services/bridge/src/appservice.rs` (Task 9)** — the 4 Tuwunel verify-items gate this task: #219 whoami response-code interaction; #465 `ip_source` NOT set (loopback AS); federation-disabled workaround (`allow_federation=true; forbidden_remote_server_names=[".*"]`); never-switch-fork. Each must be verified against the pinned Tuwunel image before the task is "done" — no compile signal.
- **`services/bridge/src/puppet.rs` (Task 10)** — the `messaging_user_id` map is bridge-local + ephemeral-to-M1; do NOT design it as a portable/federated identity (that's ADR-016/M2). Namespace puppets under the AS `registration.yaml` user namespace.
- **`services/bridge/src/relay.rs` (Task 11)** — voice flag via `org.matrix.msc3245.voice` or stable equiv; use the Matrix media repo for image/voice upload; the inbound Brehon→Matrix trigger is the HTTP POST that `bridge_notify::notify_if_enabled` (already shipped in m1-b, governance-v0) fires to `http://localhost:9009/brehon/notify`.
- **`services/bridge/src/soft_pause.rs` (Task 12)** — §16a story 4 verifies drains-to-idle (OQ-V2-09 answers A/B/C/D). Reversible without restart.

## 5. Operational rules
- Polling cadence ~10 min (`mcp__junior-brehon__list_tasks`). Briefs in `.claude/PRPs/briefs/m1-a-<role>-<n>.md`, committed to governance-v0 first; impl-task briefs synced to phase-m1-a (Mode A direct or Mode B trunk→phase). Pre-queue `memory_search_hybrid` + `/precheck` + §2.4 file-class lesson injection (but the Lemmy file-class table mostly does NOT apply to `services/bridge/**` — that's a new file class; lean on R8 external-convention reading instead).
- **Shape: pre-Shape-G** (cargo on laptop). Tree A validate-pending-laptop command = `["cd services/bridge && cargo check"]` + the workspace zero-Matrix-deps assertion. NO `--features full` (bridge has no such feature). NO Linux-compile gate unless the bridge crate's deps are Linux-sensitive (likely not for a check-only gate — re-evaluate at bm-pr per `feedback_linux_compile_proof_is_a_gate.md`: the bridge crate adds NO migration and NO workspace Cargo.toml change beyond the `exclude` line, so the Option-2 trigger may not fire — confirm at bm-pr).
- Model tiering: Planning→Opus, Impl→Sonnet, BM/ci-watcher→Haiku. The 6 user gates apply. DQ attribution `chore|docs(advisor|decision-queue):`. Memory headroom: no bulk file reads.
- **MiniMax A/B trial:** PAUSED at 3/5 (m1-b). Tree A tasks are NOT MIRROR-ref-heavy (greenfield, external conventions) → likely NOT MiniMax-eligible per the §0.1 criteria (no in-repo MIRROR sibling). Do NOT fire the trial on Tree-A tasks unless one genuinely qualifies. The remaining 2 eligible tasks come from a future MIRROR-heavy phase.

## 6. What changed from m1-b's rule set
- **No Lemmy-workspace cargo discipline** — Tree A uses `cd services/bridge && cargo check`, external matrix-sdk/ruma conventions, own error type. The entire "Cargo / Rust (Lemmy-specific)" lesson cluster is OUT of scope (R8).
- **No migration, no Diesel, no e2e.rs edits** — the e2e suite is the Lemmy workspace's; Tree A tests (if any) live under `services/bridge/tests/`.
- **Linux-compile gate likely non-binding** — re-evaluate at bm-pr (no migration, no workspace Cargo.toml change beyond `exclude`).
- **MiniMax trial paused** — Tree-A tasks likely ineligible (greenfield).

## 7. Catch-fire procedures
Universal triggers from `.claude/rules/advisor-orchestrator.md` §5.6 (subagent hard-refusal, attribution breach, non-allowlist §G4 fail, ci-watcher exit-code surprise, daemon down) apply. Tree-A additions:
- **A Tree-A task tries to ADD a Matrix dep to the workspace `members`** (instead of the isolated `services/bridge` crate) → catch-fire (breaks the zero-Matrix-deps invariant, §16a story 6).
- **A Tree-A impl applies a Lemmy-workspace lesson** (LemmyResult, Diesel, `--features full`) to `services/bridge/**` → catch-fire (R8 violation).
- **A Tuwunel verify-item (Task 9) cannot be confirmed against the pinned image** → surface to user (integration blocker, no auto-fix).

## 8. Archive after m1-a
The standard close: run `/brehon-phase-transition m1-a <next-id>` (next is likely the resumed `phase-v1-closeout` Phases 6–8, now that BOTH M1 trees will have shipped — confirm with user). This skill will: close `workflow_state_m1_a.md`, delete the two-ago record, create the next skeleton, write the next bootstrap, update MEMORY.md, commit on governance-v0. This bootstrap file stays in `.claude/PRPs/handovers/` as its own archive (git history).

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `a7d1112d0` (captured 2026-06-04) — `docs(retro): m1-b retro — M1 Tree B shipped (2 §G4 fixes, MiniMax 3/5, CR 1-fix/2-rebut)`
- Phase branch HEAD: not yet created (branch `phase-m1-a` cut at bm-cut)
- Recent governance-v0 commits (`git -C C:/Users/barri/Developer/brehon-fork log --oneline -5 governance-v0`):

  ```
  a7d1112d0 docs(retro): m1-b retro — M1 Tree B shipped (2 §G4 fixes, MiniMax 3/5, CR 1-fix/2-rebut)
  99ad26dc0 chore(bm): merge PR #177 complete
  d6d027794 Merge pull request #177 from barrie-cork/phase-m1-b
  599ed8f18 chore(advisor): m1-b bm-merge brief — merge PR #177 (--merge --admin --delete-branch)
  fa75d2fbb docs(reports): m1-b /brehon-verify — all 3 Tree-B stories pass, no phantoms
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff)
```

## Stop-and-ask tripwires

- Stop and ask if: a Tree-A task proposes adding `services/bridge` to the workspace `members` array (it MUST stay in `exclude` — §16a story 6 zero-Matrix-deps invariant).
- Stop and ask if: the plan or an impl applies a Lemmy-workspace convention (LemmyResult, Diesel, `cargo --features full`, the e2e.rs harness) to `services/bridge/**` — Tree A is a different toolchain (R8).
- Stop and ask if: a Tuwunel integration verify-item (#219 whoami / #465 ip_source / federation-disabled / never-switch-fork) cannot be confirmed against the pinned image — that's an integration blocker, not a code defect.
- Stop and ask if: the MiniMax A/B trial is about to fire on a Tree-A task — Tree A is greenfield (no MIRROR sibling) and likely ineligible per the §0.1 criteria.
- Stop and ask if: a separate `m1-a.plan.md` seems needed but `m1.plan.md` §13 Tasks 8–13 already cover Tree A — confirm the plan path (re-shape vs reuse) before authoring a planning brief.
