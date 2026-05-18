# v1-AD-e — plan-approval gate (continuation note for next session)

**Written:** 2026-05-16 by advisor at end of plan-authoring session.
**Purpose:** self-contained handoff so a fresh advisor session can run the mandatory gate-1 sequence before `/prp-implement` / `bm-cut` for v1-AD-e.

## State at handoff

- **Plan written + path-verified:** `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md` (committed to `governance-v0`). All §11 paths confirmed to exist; cited line anchors (`admin_dashboard.rs:38-46`, `lib.rs:544`, `config.rs:2318`, `governance.rs:652`) verified verbatim against HEAD.
- **Governance basis is clean:** OQ-V1-AD-01 was re-resolved 2026-05-16 (sequencing-only supersede; engine choice OPEN; 2026-04-20 deferral premise NOT overturned). The re-resolution + changelog entry were **authored by the advisor and applied by the user** to the sibling repo `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`. The plan only *cites* it — does not edit it. No ADR contradicted.
- **Keystone status:** v1-AD-a/b/c/d/wrap-up all SHIPPED (PRs #72/#76/#81/#87/#90). v1-AD-e is the final page-layer sub-phase; the 3 read-path-requiring PRD §6 pages are deferred to a future v1-AD-f via DQ #1.
- **Parallel session:** v1-ship-1 (AGPL §13 source-disclosure) runs in a separate CC session. Zero file overlap with v1-AD-e. v1-AD-e gets its own `brehon-fork-ad-e` worktree after bm-cut (per `.claude/rules/multi-lane-worktree.md`).
- **DQ pending at handoff:** `[#229]` only (unrelated — Shape-G-re-enable). The two v1-AD-e DQ pre-seeds are NOT yet written (they are step 1 below).
- **Shape G:** SUSPENDED until 2026-06-01 → v1-AD-e runs **validate-pending-laptop** (per `advisor-orchestrator.md` §5.2). Plan §15.6 covers the 2026-06-01 re-enable case (no plan change needed — command shapes are identical).

## Mandatory gate-1 sequence (next session runs this BEFORE bm-cut)

Per `.claude/rules/advisor-orchestrator.md` §3.1 ("Planning complete → §3.4 DoD smoke → §3.5 watchpoint specificity → user gate 1 → on approval, queue bm-cut") and CLAUDE.md "Mandatory user gates" #1.

### Step 1 — file the two DQ pre-seeds (advisor → user)

Both are fully specified in plan §19. Write them to `.claude/decision-queue.json` `pending[]` with `from: "advisor"`, `kind: "blocker"`, commit subject matching `^(chore|docs)\((advisor|decision-queue)\)`, push to `governance-v0`. Next id: compute `max(all ids across decision-queue.json + decision-queue-archive-*.json) + 1` (currently highest is #229 → next is #230, but RE-VERIFY across archives per `decision-queue.md` next-id rule).

- **DQ (scope cut):** Dashboard + Audit only vs all-5 vs literal-URL variant. Planner-recommended option (a) — Dashboard + Audit only; Config-editor/Single-key/Rule-set-manager → v1-AD-f. Verbatim question + 3 options in plan §19 "DQ #1".
- **DQ (engine):** maud vs askama. Planner-recommended (a) — maud (evidence table plan §5.2). Verbatim question + 2 options in plan §19 "DQ #2".

These are **judgment-heavy / scope-affecting** → user-relay (surface, wait for user reply, record `answered_by: "user"` verbatim). Plan Task 0 Probe 5 + Probe 6 gate on BOTH being in `resolved[]` before any impl runs.

### Step 2 — §15 DoD smoke test against current HEAD

Per `feedback_pre_phase_dod_smoke_test.md` + `feedback_plan_dod_dry_run_at_write.md`. Run **every** §15 command literally against current `governance-v0` HEAD; capture exit codes; surface as part of plan approval. The §15 commands (all bat-wrapper, `--features full`, clippy `--no-deps`):

- §15.1 `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > <log> 2>&1"` — EXPECT 0 (baseline; no v1-AD-e code yet so this just confirms HEAD is green).
- §15.2 `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > <log> 2>&1"` — EXPECT 0.
- §15.3 `cmd //c "scripts\\brehon\\cargo-test.bat --no-run -p lemmy_server --test e2e > <log> 2>&1"` — EXPECT 0 (NOTE: `-p lemmy_server` here is the test-target compile, NO `--features full` on this one — the `-p X --features full` incompat per `feedback_features_full_p_crate_incompatible` does NOT apply because `--features full` is absent here; this is intentional and correct).
- §15.4 e2e full-suite (the regression-guard baseline) — long-running (~26 min); `run_in_background: true`, bat wrapper, trailer-echo pattern. EXPECT current suite green (this is the pre-change baseline the Task 2 extraction must not regress).

All four are **executable as written** (verified: bat wrappers exist, `--features full` valid at workspace scope, `-p lemmy_server --test e2e --no-run` valid without `--features full`). No advisor-side DoD-rejection expected. If any HEAD command is unexpectedly red, that is a pre-existing-breakage signal unrelated to v1-AD-e → surface to user, do NOT proceed to bm-cut.

### Step 3 — watchpoint-specificity gate

Per `feedback_advisor_watchpoint_specificity.md`. Plan §10 patterns + §18 risks each cite a specific file:line (verified at authoring: `admin_dashboard.rs:38-64`, `admin_audit_stream.rs:123-126/245-260`, `config.rs:2318`, `governance.rs:652`, `lib.rs:544`). No concept-only watchpoints. Expected to PASS — but the next session must re-confirm (re-grep the cited anchors; HEAD may have moved if the parallel v1-ship-1 session merged something into `governance-v0` — none of v1-ship-1's files overlap, but re-verify the 5 anchors are still byte-accurate).

### Step 4 — surface to user for plan approval (user gate 1)

Bundle: (a) DoD smoke results (4 exit codes), (b) watchpoint gate ✓, (c) the two DQ resolutions from Step 1, (d) one-screen plan summary. Wait for explicit user approval. **On approval → queue `bm-cut` for `phase-v1-AD-e` off `governance-v0`** (then `git worktree add ../brehon-fork-ad-e phase-v1-AD-e` per multi-lane rule, open a lane-dedicated session).

## Quick-resume command for next session

```
/start-brehon v1-AD-e
```
Then read this note + `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md`, and execute Steps 1→4 above.
