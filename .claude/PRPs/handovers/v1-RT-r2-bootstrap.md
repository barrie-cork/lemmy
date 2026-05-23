---
phase: v1-RT-r2
plan: .claude/PRPs/plans/v1-RT-r2.plan.md   # not yet authored
phase_branch: phase-v1-RT-r2                  # not yet created — cut at bm-cut
worktree: C:/Users/barri/Developer/brehon-fork-rt-r2   # created at bm-cut; until then use canonical brehon-fork
authored: 2026-05-23
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the v1-RT-r2 advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon v1-RT-r2.** The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-rt-r2` once `bm-cut` creates the lane worktree (per `.claude/rules/multi-lane-worktree.md`). There is no homeserver session.

**Parallel lane active:** `v1-ship-2` is running concurrently in worktree `brehon-fork-ship-2` (PR #147 open). Multi-lane discipline applies: both lanes share the canonical PMD (`C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db`) but have separate `.claude/decision-queue.json` files. Do NOT mutate ship-2's DQ entries from the RT-r2 session.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane; verify ship-2 worktree is separate.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `ece760ae6` (see §"Git state at handoff"); if drifted, log the delta.
3. Read `.claude/decision-queue.json` for any pending entries (0 at handoff — see §"Decision-queue snapshot").
4. `memory_search_hybrid(query: "reputation tuning decay chained halving", limit: 5)` — load relevant lessons before authoring the planning brief.

## Next concrete action

Author `.claude/PRPs/briefs/v1-RT-r2-planning-1.md` (scope: per-dimension chained-halving decay per PRD §11 phase 2). Run `/brehon-clarify v1-RT-r2-planning-1.md`. Queue planning Junior once all clarify-DQs resolved.

---

## 1. v1-RT-r2 in one paragraph

v1-RT-r2 ships the per-dimension chained-halving decay logic for the reputation subsystem. The PRD §11 phase 2 description: "Replace `compute_applied_delta` with per-dimension/per-direction calculator behind feature flag; add bounds clamping to `recompute_snapshot`." This is gated on v1-RT-r1 (PR #126, merged — schema foundation, 4 migrations, `ReputationEventSourceType` enum, 26 governance_config seed keys, `ENTRY_KIND_*` consts). DoD: the new calculator passes unit tests per-dimension, `recompute_snapshot` clamps correctly, feature-flag disable path restores prior behaviour, Phase-2 e2e green.

## 2. Why v1-RT-r2 is easier/harder than v1-federation-inbound-e

**Easier:** No concurrency race class (advisory-lock pattern resolved in fed-in-e; reputation decay is single-writer scheduled). No AP protocol boundary. No Postgres TEXT null-byte trap (no advisory-lock key construction). The codebase already has the reputation subsystem schema + feature-flag infrastructure from r1.

**Not easier:** The decay math itself is domain-logic-heavy — chained halving per dimension/direction with configurable half-lives from `governance_config`. The planner needs to read the PRD §5 proposed solution carefully. `compute_applied_delta` and `recompute_snapshot` are non-trivial Rust fns; any struct-shape change triggers E0063 across multiple callsites (see `feedback_fix_impl_enumerate_all_callsites.md`). Feature-flag gating must be verified both arms (enabled + disabled paths must be tested).

## 3. Lessons from v1-federation-inbound-e that apply to v1-RT-r2

**Advisor-side:**
- `feedback_postgres_text_null_byte_forbidden.md` — promoted this phase. If any new lock key or advisory-lock usage appears in r2, use `:` or `/` as separator. Not expected in r2 (no advisory locks) but lesson is fresh.
- `feedback_advisor_must_commit_brief_before_planning_dispatch.md` — second occurrence this phase. Commit the planning brief to `governance-v0` BEFORE dispatching the planning Junior. The pre-queue check in `advisor-orchestrator.md §2` gate: `git show governance-v0:.claude/PRPs/briefs/<brief>.md` must succeed before `create_task`.

**Planning-side:**
- `feedback_plan_dod_dry_run_at_write.md` — DoD commands must be executable. For RT-r2 the key DoD gate is a targeted `cargo test` of the reputation module. Verify the test invocation form against the workspace (`-p lemmy_db_queries` or `--workspace` — confirm which crate owns `compute_applied_delta`).
- `feedback_advisor_watchpoint_specificity.md` — every watchpoint must cite a specific file:fn:line, not a concept.

**Impl-side:**
- `feedback_lemmy_error_no_std_error.md` — Case A applies if any new e2e test is added alongside existing `v1_*_fixtures` modules. Mirror the sibling's `LemmyResult<()>` return type, not `Box<dyn Error>`.
- `feedback_fix_impl_enumerate_all_callsites.md` — if `compute_applied_delta` signature changes, `rg "compute_applied_delta" crates/` BEFORE authoring the impl brief to enumerate ALL callsites. Cap is the distinct-file count, not ≤3.
- `feedback_clippy_test_style.md` — Lemmy clippy denies `unwrap`/`expect` in tests; use `?` with `LemmyResult`.

**BM-side:**
- Nothing phase-specific. Standard bm-cut → bm-pr → bm-merge flow. Multi-lane: `--repo barrie-cork/lemmy` on every `gh pr` command; base branch `governance-v0` (not `main`, not `phase-v1-ship-2`).

## 4. v1-RT-r2-specific watchlist

1. **`compute_applied_delta` callsite count** — `crates/db_queries/src/governance/reputation.rs` (or similar; locate before planning brief). Enumerate ALL callers with `rg "compute_applied_delta" crates/` before the planning brief is authored. If >5 files, flag to planner to split impl tasks by file cluster.
2. **Feature-flag disable path** — `governance_config` key `feature.reputation_v1_decay_enabled` (per PRD §6 acceptance criteria). The impl must test both `true` and `false` arms. Plan §16a stories must include a story for the fallback path.
3. **`recompute_snapshot` bounds clamping** — PRD §5 requires clamping reputation scores to `[0.0, 1.0]` per dimension. Verify the Rust fn signature; `f64` field requires explicit `f64::clamp`. Do not use `unwrap_or_default()` on `Option<f64>` — per `feedback_error_idiom_at_trust_boundary` axis 4.
4. **Migration gating** — r2 should NOT add new migrations (r1 shipped the schema; r2 is compute-logic only). If the planner proposes a migration in r2, flag as scope violation and ask why.
5. **Parallel ship-2 lane** — `crates/server/tests/e2e.rs` is being touched by `v1-ship-2`. If r2 also needs an e2e test, coordinate: check what ship-2's latest phase-branch tip looks like before authoring r2's e2e brief to avoid divergent edit conflicts.

## 5. Operational rules

- **Polling cadence:** ~10 min `mcp__junior-brehon__list_tasks`.
- **Brief discipline:** briefs at `.claude/PRPs/briefs/v1-RT-r2-<role>-<n>.md`, committed to `governance-v0` BEFORE `create_task`. Pre-queue: `/precheck` + `memory_search_hybrid` + §2.4 mandatory file-class lesson injection.
- **Mandatory lesson injection (§2.4):** any edit to `crates/server/tests/e2e.rs` → inject `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md`; ≥2 e2e edits → also `feedback_junior_worker_e2e_edit_hang.md`; any `pg_advisory_xact_lock` call → inject `feedback_pg_advisory_xact_lock_void_decode.md` + `feedback_postgres_text_null_byte_forbidden.md`.
- **Shape G SUSPENDED** until 2026-06-01 (DQ #229). validate-pending-laptop pathway active per `advisor-orchestrator.md §5.2`. Cargo runs on laptop.
- **Windows e2e invocation:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true`. Never bare `cargo test` on Windows.
- **Model tiering:** Planning→Opus, Impl→Sonnet, BM/ci-watcher→Haiku.
- **Clarify gate:** `/brehon-clarify` before every planning brief dispatch.
- **6 user gates:** plan approval (gate 1), judgment-heavy DQ (gate 2), CR triage (gate 3), Phase-2 e2e local vs dispatch (gate 4), merge confirm (gate 5), retro sign-off (gate 6). Never skip.
- **DQ attribution:** `chore|docs(advisor|decision-queue):` subject pattern for any advisor DQ write.
- **Multi-lane PMD:** canonical `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` — never relative path in `.mcp.json`.

## 6. What changed from v1-federation-inbound-e's rule set

- **Domain shift:** federation/concurrency → reputation math. No AP protocol, no advisory-lock, no null-byte trap. Domain-specific watchpoints above (§4) replace fed-in-e's TOCTOU/race watchpoints.
- **New lesson active:** `feedback_postgres_text_null_byte_forbidden.md` (promoted this phase). Injected under `pg_advisory_xact_lock` row in §2.4 table.
- **Parallel lane added:** `v1-ship-2` is now the second concurrent active lane. Multi-lane CWD check ritual enforced at session-start.
- **Callsite-enumeration discipline raised:** after fed-in-a/RT-r1 pattern, any struct-shape change must pre-enumerate all callsites before brief authorship. Guard now explicit in §4 watchpoint 1.

## 7. Catch-fire procedures

From `.claude/rules/advisor-orchestrator.md §5.5`:
- Junior writes `crates/**` without authorising brief → catch-fire.
- `answered_by: "advisor"` in commit with non-`chore|docs(advisor|decision-queue):` subject → catch-fire.
- bm-task opens PR into `main` instead of `governance-v0` → catch-fire.
- Phase branch has uncommitted state when Junior reports complete → catch-fire.
- Conformance-audit Tier-1 finding on governance Rust files → HARD REFUSAL, surface to user.
- Cycle-count ≥3 with same `(error_class, file_basename)` → HARD REFUSAL catch-fire.
- `validate-pending` mutated to fail/cancelled/timed_out + non-allowlist → catch-fire.
- **Phase-specific:** if planner proposes a new DB migration in r2 → catch-fire (scope violation; r2 is compute-logic only, r1 owns the schema).

## 8. Archive after v1-RT-r2

Run `/brehon-phase-transition v1-RT-r2 v1-RT-r3`. This skill will: close `workflow_state_v1_RT_r2.md`, delete the two-ago record (`workflow_state_v1_federation_inbound_e.md`), create the `v1-RT-r3` skeleton, write the next bootstrap file, update MEMORY.md, commit on governance-v0. This bootstrap file stays in `.claude/PRPs/handovers/` as its own archive — git history is the archive.

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `ece760ae6` (captured 2026-05-23) — `chore(advisor): log bm-merge PR #148 v1-federation-inbound-e — 183d2753f`
- Phase branch HEAD: `phase-v1-RT-r2` not yet created (branch cut at bm-cut)
- Recent governance-v0 commits:

  ```
  ece760ae6 chore(advisor): log bm-merge PR #148 v1-federation-inbound-e — 183d2753f
  183d2753f Merge pull request #148 from barrie-cork/phase-v1-federation-inbound-e
  66688fced chore(advisor): author bm-merge brief for v1-federation-inbound-e
  12afb7fbc chore(bm): PR #148 opened for v1-federation-inbound-e
  004872cf5 chore(advisor): author bm-pr brief for v1-federation-inbound-e
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff — 0 pending entries)
Note: DQ #229 (Shape G re-enable) is in resolved[], dated reminder for 2026-06-01.
```

## Stop-and-ask tripwires

- Stop and ask if: the planner proposes a new DB migration under `crates/db_schema/migrations/**` — r2 is compute-logic only; a migration signals scope creep into r1 schema territory.
- Stop and ask if: `rg "compute_applied_delta" crates/` returns hits in more than 5 distinct files — the brief's impl task cap needs adjustment before dispatch.
- Stop and ask if: the Phase-2 e2e log shows a failure in any pre-existing `v1_*_fixtures` test — regression suspected; do not auto-queue a fix-impl before surfacing to user.
- Stop and ask if: the planning Junior proposes touching `crates/server/tests/e2e.rs` for an r2 test AND `v1-ship-2` has an open PR with e2e edits — coordinate before dispatching to avoid merge conflict class.
- Stop and ask if: the feature-flag `feature.reputation_v1_decay_enabled` is absent from the governance_config seed rows on `governance-v0` (would mean r1 config migration didn't land, blocking r2's feature-flag arm test).
