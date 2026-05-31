---
phase: v1-quality-r3c
plan: .claude/PRPs/plans/v1-quality-r3c.plan.md   # not yet authored
phase_branch: phase-v1-quality-r3c                  # not yet created until bm-cut
worktree: C:/Users/barri/Developer/brehon-fork-quality-r3c   # created at bm-cut; until then canonical brehon-fork
authored: 2026-05-31
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the v1-quality-r3c advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon v1-quality-r3c.** This is a fresh session (or a resumed one). The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-quality-r3c` once `bm-cut` creates the lane worktree (per `.claude/rules/multi-lane-worktree.md`). There is no homeserver session.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane. Note: stale worktrees `brehon-fork-quality-r2-validate`, `brehon-fork-quality-r3b`, `brehon-fork-rt-r4` may still be listed — those are the user's to remove, not yours to act on.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `228857f9c` (see §"Git state at handoff"); if drifted, check `git log --oneline 228857f9c..governance-v0` and update mental model before acting.
3. Read `.claude/decision-queue.json` for any pending entries since handoff. At handoff: 0 pending entries.
4. Check `cat .claude/governance-log/retro-bypass.jsonl 2>/dev/null | tail -5` — per `feedback_outcome_not_cause_check_retro_bypass.md` (this was skipped at r3b session start; do it here first).
5. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_v1_quality_r3c.md` is the running-state scratchpad. Read `workflow_state_v1_quality_r3b.md` once at session start for carry-forward (CLOSED record), then do not re-read.

## Next concrete action

**Scope selection (user-steered quality lane):** v1-quality-r3c scope is not yet defined. Candidate issues for the next quality sweep:

1. **Issue #165** — fix stale `.coderabbit.yaml` v0 "exactly 11 endpoints" rule (ADR-010 false-positive source; caused the UNSTABLE merge gate at r3). Low effort, high leverage for CI noise.
2. **Issue #166** — e2e tests bypass Actix route wiring; add HTTP-path coverage suite-wide. Medium effort; touches `e2e.rs` broadly — plan must use `feedback_fix_impl_pre_locate_e2e_anchors.md` discipline.
3. **BREHON_DISABLE_* sites sweep** — enumerate and audit `BREHON_DISABLE_` env-var gates in `crates/`; ensure each has a test that exercises the disabled path. Low-medium effort.
4. **Issue #158** (emit_reputation_event dedup) — premature-DRY gate: re-check `grep -rn "emit_reputation_event" crates/ --include="*.rs" -l`; if ≥3 callers, scope is unblocked. If still 2, defer again.
5. **Issue #167 close** — verify GitHub auto-closed #167 on PR #170 merge; if not, `gh issue close 167 --repo barrie-cork/lemmy`.

Ask the user which issue(s) to target before authoring the planning brief.

Once scope is confirmed: author `.claude/PRPs/briefs/v1-quality-r3c-planning-1.md` → `/brehon-clarify` → queue planning Junior.

---

## 1. v1-quality-r3c in one paragraph

v1-quality-r3c is the next user-steered sweep in the quality meta-lane. v1-quality-r3b shipped Issue #167 (DB URL capture at `LemmyContext::create`, removing env-var re-read in `admin_audit_stream`). v1-quality-r3c's scope is not yet fixed — the quality lane is driven out-of-band per user instruction 2026-05-22. The most actionable candidates are: #165 (`.coderabbit.yaml` false-positive ADR-010 rule), #166 (HTTP-path e2e coverage), and the `BREHON_DISABLE_*` audit. DoD follows the quality-lane pattern: workspace cargo gates exit 0, full e2e baseline maintained (126/0/5 or better), targeted test coverage added.

## 2. Why v1-quality-r3c is easier/harder than v1-quality-r3b

**Easier:** v1-quality-r3b crossed into production code (`crates/api/`) with a cross-crate accessor, requiring careful pub-visibility reasoning. If r3c targets #165 (config file edit) or the `BREHON_DISABLE_*` audit (env-var gate checks), it stays in infrastructure/test territory — lower impl complexity, faster CR cycle.

**Not easier:** Issue #166 (HTTP-path e2e coverage) is broad — it touches `e2e.rs` at many points and requires enumerating Actix route wiring across all governance endpoints. That's a high-complexity task with anchor-collision risk. If r3c includes #166, the brief must fire `feedback_fix_impl_pre_locate_e2e_anchors.md` and the uniqueness gate for every anchor before dispatch.

## 3. Lessons from v1-quality-r3b that apply to v1-quality-r3c

**Advisor discipline:**

- **State lane mode explicitly in the bootstrap.** r3b's bootstrap didn't say Mode A or B; the session inferred it. This bootstrap says: Mode B is the default for single-task quality sweeps (no dedicated lane worktree needed until e2e is mandatory). Switch to Mode A only if validate-pending-laptop is the gate. Per `feedback_mode_b_trunk_phase_sync.md`.
- **Check retro-bypass.jsonl at session start.** r3b skipped this. Catch-fire: run `cat .claude/governance-log/retro-bypass.jsonl 2>/dev/null | tail -5` in the session-start ritual (Step 4 above). Per `feedback_outcome_not_cause_check_retro_bypass.md`.
- **Merge-forward before bm-merge is routine.** governance-v0 moves fast during active sessions. After `/brehon-verify`, run `git log origin/governance-v0 ^phase-v1-quality-r3c --oneline` proactively; if non-empty, merge-forward before queuing bm-pr. DQ conflict pattern: theirs (archived DQ) + carry (active validate-pending-laptop).
- `feedback_falsifiable_hypothesis_before_structural_fix.md` — any DQ proposing a structural fix names a hypothesis; verify ≤30 min before acting.

**For production code edits (if r3c touches crates/):**

- `feedback_multi_write_handlers_need_transactions.md` — 2+ DB writes → `conn.run_transaction()`.
- `feedback_lemmy_error_no_std_error.md` — all new test/handler code uses `LemmyResult<()>` with `?`. See `feedback_async_pool_test_pattern.md` for e2e test pattern.
- `feedback_governance_type_state_handlers.md` — if any handler loads `ModerationCase` and matches `case.status`.

**For any e2e.rs edits:**

- `feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate verbatim `old_string`/`new_string` anchors BEFORE authoring the brief.
- Uniqueness gate: `grep -c '<anchor>' crates/server/tests/e2e.rs` must return `1` for every Edit target. Per `advisor-orchestrator.md` §2.4 row for `e2e.rs` ≥2 edits.

**Shape G still SUSPENDED:** validate-pending-laptop path active. Shape G re-enable date was 2026-06-01 — check current DQ state at session start; if a DQ lifted the suspension, update accordingly.

## 4. v1-quality-r3c-specific watchlist

1. **`.coderabbit.yaml` ADR-010 rule (if r3c = #165):** `grep -n "exactly 11" .coderabbit.yaml` — the false-positive rule lives here. Any edit must be tested against the actual endpoint count (`grep -rn "ApiEndpoint" crates/api/api/src/governance/` | wc -l). Fix must not suppress valid ADR-010 checks.

2. **HTTP-path e2e coverage (if r3c = #166):** `crates/server/tests/e2e.rs` currently bypasses Actix routing — all governance tests call handler functions directly. Plan must enumerate the 11 v0-scope endpoints from `docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md §2` and specify per-endpoint HTTP-path test shape. Pre-locate anchors before dispatch.

3. **BREHON_DISABLE_* audit (if r3c = DISABLE sweep):** `grep -rn "BREHON_DISABLE_" crates/ --include="*.rs"` — enumerate all gate sites. Plan must assert that each disabled path has at least one e2e test that toggles the gate.

4. **Issue #158 gate (emit_reputation_event):** before including #158 in scope, verify caller count: `grep -rn "emit_reputation_event" crates/ --include="*.rs" -l`. ≥3 files → premature-DRY gate open. Still 2 files → defer; do NOT add to r3c scope.

5. **DQ `a3d0e9941441-042` harvest:** this `kind: log` entry from r3b notes §16a spec drift ("exactly ONE → exactly 14"). At r3c plan-authoring time, the planner should correct the §16a wording in the r3b plan if it ever gets referenced. Low priority — harvest at retro.

## 5. Operational rules

Polling cadence: ~10 min `mcp__junior-brehon__list_tasks`. Pre-queue: run `/precheck` (5-gate; covers Tailscale, daemon, trunk sync, forbidden window, memory headroom) + `memory_search_hybrid` (limit 5) + §2.4 mandatory file-class lesson injection. Mode B default for single-task quality sweeps; lane worktree only when validate-pending-laptop requires it.

**Shape G status:** SUSPENDED as of 2026-05-16 (commit `9bd933fe2` re-enabled tentatively; per MEMORY.md note, monitor first run). At this bootstrap authorship (2026-05-31), validate-pending-laptop remains the path — check `.claude/decision-queue.json` at session start for any resolved suspension-lift DQ.

**Windows e2e:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true` (~26 min). Never bare `cargo test` on Windows (libpq.dll missing). Per `feedback_windows_e2e_requires_bat_wrapper.md`.

Model tiering: Planning → Opus 4.8, Impl → Sonnet 4.6, BM/ci-watcher → Haiku 4.5. Six user gates (plan approval, judgment-heavy DQ, CR triage, Phase-2 e2e local vs dispatch, merge confirm, retro sign-off). DQ attribution: `chore|docs(advisor|decision-queue):`.

**Telegram completion hook:** check `mcp__junior-brehon__list_hooks` at session start — if hook ID 1 absent (daemon restart wipes hooks), recreate per `feedback_daemon_telegram_completion_hook.md`.

## 6. What changed from v1-quality-r3b's rule set

- **Lane mode explicit.** r3b left mode implicit; r3c bootstrap states Mode B as default for single-task quality sweeps.
- **Retro-bypass.jsonl added to session-start ritual.** Missing from r3b session start; now Step 4.
- **Merge-forward discipline made proactive.** r3b caught it reactively (bm-pr flagged CONFLICTING). r3c: run the check after `/brehon-verify`, before queueing bm-pr.
- **Issue #167 closed.** r3b shipped the fix — that carry-forward is resolved. Remaining candidates: #165, #166, #158, BREHON_DISABLE_* sweep.

## 7. Catch-fire procedures

Universal triggers from `.claude/rules/advisor-orchestrator.md` §5.5 plus:

- **Stop and ask if:** the scope selection results in >3 files in `crates/server/tests/e2e.rs` — that size implies a broad sweep that warrants its own plan-complexity review.
- **Stop and ask if:** #158 (emit_reputation_event) is included and caller count is still 2 — that's a premature-DRY violation per the original deferral gate.
- **Stop and ask if:** Shape G suspension is unclear — do not auto-dispatch validate-pending (GH Actions) without confirming the suspension status from current DQ.
- **Stop and ask if:** bm-pr shows `mergeStateStatus: CONFLICTING` before attempting merge — merge-forward is required; do not merge blind.

## 8. Archive after v1-quality-r3c

Run `/brehon-phase-transition v1-quality-r3c v1-quality-r3d` (or next appropriate id). This skill will: close `workflow_state_v1_quality_r3c.md`, delete `workflow_state_v1_quality_r3b.md`, create the next skeleton, write the next bootstrap file, update brehon-fork MEMORY.md, commit on governance-v0. This bootstrap file (`.claude/PRPs/handovers/v1-quality-r3c-bootstrap.md`) stays in place as its own archive — git history is the archive.

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `228857f9c` (captured 2026-05-31) — `fix(harness-audit): deterministic frontmatter classification (Phase 0 Step 6)`
- Phase branch HEAD: not yet created (branch `phase-v1-quality-r3c` cut at bm-cut)

Recent governance-v0 commits:

```
228857f9c fix(harness-audit): deterministic frontmatter classification (Phase 0 Step 6)
5e68b858e docs(rules): strip multi-lane-worktree.md narrative → refs (-55%)
b8a8eaf72 docs(retro): v1-quality-r3b retro — Issue #167 fix shipped, merge-forward lesson
04a772528 Merge remote-tracking branch 'origin/governance-v0' into governance-v0
41fb7fe12 docs(advisor): handover governance-v0 harness-cleanup-2026-05-31
```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty — 0 pending entries at handoff)
Note: DQ kind:log a3d0e9941441-042 exists in resolved[] — §16a spec drift "exactly ONE → exactly 14" EnvVarGuard count; harvest at r3c plan authoring.
```

## Stop-and-ask tripwires

- Stop and ask if: scope includes Issue #166 (HTTP-path e2e coverage) AND the plan does not pre-locate verbatim `old_string` anchors for every `e2e.rs` edit — that is the anchor-collision risk class from r3b.
- Stop and ask if: Issue #158 (emit_reputation_event) is proposed for r3c AND `grep -rn "emit_reputation_event" crates/ --include="*.rs" -l` still returns only 2 files — premature-DRY gate not yet open.
- Stop and ask if: validate-pending-laptop DQ `be6ddc108436-001` is referenced again — it was for r3b; a new entry is needed for r3c's task if validate-pending-laptop is the validation path.
- Stop and ask if: the session's governance-v0 HEAD differs from `228857f9c` by more than 5 commits — that implies a significant trunk advance during a context gap; rebuild mental model from `git log` before dispatching any Junior task.
- Stop and ask if: `bm-pr` shows `mergeStateStatus: CONFLICTING` — governance-v0 may have advanced again; merge-forward before proceeding.
