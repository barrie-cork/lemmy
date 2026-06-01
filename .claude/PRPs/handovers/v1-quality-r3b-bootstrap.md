---
phase: v1-quality-r3b
plan: .claude/PRPs/plans/v1-quality-r3b.plan.md   # not yet authored — GH #167 LemmyContext::database_url() refactor
phase_branch: phase-v1-quality-r3b                  # not yet created until bm-cut
worktree: C:/Users/barri/Developer/brehon-fork-quality-r3b   # created at bm-cut; until then canonical brehon-fork
authored: 2026-06-01
authored_by: advisor (canonical brehon-fork / governance-v0 session — updated at v1-redaction-r1 transition)
purpose: Bootstrap the v1-quality-r3b advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon v1-quality-r3b.** This is a fresh session (or a resumed one). The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-quality-r3b` once `bm-cut` creates the lane worktree (per `.claude/rules/multi-lane-worktree.md`). There is no homeserver session.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `49dd95685` (see §"Git state at handoff"); if drifted, `git -C C:/Users/barri/Developer/brehon-fork log --oneline 49dd95685..governance-v0` and update your mental model before acting.
3. Read `.claude/decision-queue.json` for any pending entries since handoff; compare against the §"Decision-queue snapshot" below (empty at handoff).
4. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_v1_quality_r3b_new.md` is the running-state scratchpad. Read `workflow_state_v1_redaction_r1.md` once at this session's start for carry-forward context, then do not re-read.
5. Check `mcp__junior-brehon__list_hooks` — hook ID 1 must exist; recreate if absent per `feedback_daemon_telegram_completion_hook.md`.
6. Check `cat .claude/governance-log/retro-bypass.jsonl 2>/dev/null | tail -5` for hook fail-open events.

## Next concrete action

Author `.claude/PRPs/briefs/v1-quality-r3b-planning-1.md` (scope: Issue #167 — refactor `admin_audit_stream.rs:125` to derive the Postgres LISTEN URL from the injected `context` or `context.settings()` path rather than re-reading a live env var, ensuring the connection is not coupled to the process-global `LEMMY_DATABASE_URL`) → `/brehon-clarify` → queue planning Junior.

Before authoring the brief, promote the nominated lesson from v1-quality-r3 retro:
1. `feedback_envvarguard_audit_window.md` — audit scripts with fixed look-back windows must be sized to the full multi-line block they're checking, not a fixed character count. *(Verify: `.claude/lessons/feedback_envvarguard_audit_window.md` exists; if so, run `scripts/sync-lessons-to-pmd.sh` to confirm it's indexed.)*

Also check: run `grep -rn "emit_reputation_event" crates/ --include="*.rs" -l` from governance-v0 HEAD — if ≥ 3 files, issue #158 may now qualify for inclusion; if still 2, it remains deferred (premature-DRY gate).

---

## 1. v1-quality-r3b in one paragraph

v1-quality-r3b delivers **Issue #167**: refactor the `admin_audit_stream` handler so it derives its Postgres LISTEN connection URL from the `LemmyContext` object rather than reading `LEMMY_DATABASE_URL` from the process-global environment. This was the deferred architectural fix from v1-quality-r2's regression cycle — the handler used `Settings::get_database_url()` lazily, which caused test failures when `EnvVarGuard` dropped the env var before the handler's lazy re-read. The v1-quality-r2 fix was a band-aid (keeping the var live in 3 test bodies); r3b does the proper fix at the handler level. Current state on governance-v0 shows `admin_audit_stream.rs:125` uses `context.settings().get_database_url()` — the fix must verify whether `context.settings()` reads the live env var or a baked value, and if the former, replace it with a path that does NOT read the env. DoD: workspace cargo gates exit 0, full e2e suite passes locally (no regression vs v1-quality-r3 baseline: 126/0/5 or better), `admin_audit_stream.rs` no longer reads `LEMMY_DATABASE_URL` from the process-global environment. Issue #158 (`emit_reputation_event` helper extraction) remains deferred (2 callers; premature-DRY gate, re-check before brief authorship).

## 2. Why v1-quality-r3b is easier/harder than v1-quality-r3

**Easier:** v1-quality-r3b is a single-handler fix (one file: `crates/api/api/src/governance/admin_audit_stream.rs`). The scope is narrow and well-defined by Issue #167. No e2e.rs sweep required — the e2e test changes are limited to verifying the handler works without `LEMMY_DATABASE_URL` being set explicitly. The EnvVarGuard machinery is already in place; the fix should reduce the places where guards are needed, not add more.

**Not easier:** This crosses into production code (`crates/api/`), not just `e2e.rs` — it gets a full CR semantic review. The shape of the fix needs the planner to decide: does `context.settings().get_database_url()` actually read the live env var, or is it a baked value from startup? If it reads live env, the fix must substitute a pool-derived URL or a settings field cached at startup. That decision is load-bearing and must be explicit in the plan. The e2e test for `admin_audit_stream` must be updated to assert the LISTEN connection works without the env var set — the fix is only complete if the test validates the new path.

## 3. Lessons from v1-quality-r3/v1-redaction-r1 that apply to v1-quality-r3b

**From v1-redaction-r1 (new since prior bootstrap authorship):**
- `feedback_merge_forward_clippy_debt_from_trunk.md` — after any merge-forward pulling quality-r* commits, run `cargo clippy --workspace --features full --no-deps -- -D warnings` before queueing the next Junior task.
- `feedback_merge_forward_e2e_conflict_default_to_governance.md` — for e2e.rs merge-forward conflicts where the phase branch made no test-logic changes, take governance-v0 side.
- bm-merge pre-dispatch: run `git ls-remote origin governance-v0` immediately before `create_task` and compare to the brief's `base_sha`. If SHA differs, update the brief first.

**Advisor discipline:**
- `feedback_envvarguard_audit_window.md` — any audit script checking multi-line blocks must be sized to accommodate the full block height, not a fixed char count. Apply when authoring the DoD audit step.
- adr-compliance `mergeStateStatus: UNSTABLE` with clean local scan → skip re-trigger loop, go straight to owner-acknowledge comment + `--admin` merge. First use documented at v1-quality-r3; the protocol is now known. Note in the bm-merge brief §5 when authoring.
- Duplicate `validate-pending` DQ from re-attempted task: impl-task brief §4 must cite `feedback_dq_v3_append_via_helper_script.md` — check for existing `validate-pending` entry for this task before appending new one.

**For production code edits (crates/api/):**
- `feedback_multi_write_handlers_need_transactions.md` — if the #167 fix introduces any new DB writes alongside the LISTEN connection (unlikely, but confirm at plan review).
- `feedback_lemmy_error_no_std_error.md` — any new or modified handler code must use `LemmyResult<()>` with `?` (not `Box<dyn Error>`).
- `feedback_governance_type_state_handlers.md` — if the handler loads `ModerationCase` and matches on `case.status`, the type-state pattern applies. Check at plan review.

**For any e2e.rs edits (the test update for #167):**
- `feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate verbatim anchors before writing the brief; even a small e2e.rs edit must have exact old_string confirmed from the phase-branch tip.
- `feedback_lemmy_error_no_std_error.md` — e2e error-shape conventions.

**Mode B discipline (v1-quality-r3 ran cleanly in Mode B):**
- Trunk→phase brief sync via daemon `git fetch origin governance-v0 && git merge origin/governance-v0` pattern — clean for single-plane quality sweeps. Use the same pattern for r3b unless the phase touches >2 files and warrants a dedicated worktree.

## 4. v1-quality-r3b-specific watchlist

1. **`crates/api/api/src/governance/admin_audit_stream.rs:125`** — the `context.settings().get_database_url()` call that Issue #167 must fix. Verify at plan-review: does `Settings::get_database_url()` read `LEMMY_DATABASE_URL` from the live process env, or from a startup-baked `Settings` struct? If live-env: the fix must substitute a pool URL or settings-cached value. Plan §13 IMPLEMENT file must list this file; the DoD checkpoint must assert `grep -n "get_database_url\|LEMMY_DATABASE_URL" crates/api/api/src/governance/admin_audit_stream.rs` exits non-zero (i.e., the call is gone).
2. **`crates/server/tests/e2e.rs` — the `admin_audit_stream` test** — the fix is only architecturally complete if the e2e test verifies the LISTEN connection works WITHOUT `LEMMY_DATABASE_URL` being explicitly set. The plan must include a brief §13 task that edits the test to verify this. Pre-locate the test's `EnvVarGuard` blocks from the phase-branch tip before authoring the impl brief.
3. **`emit_reputation_event` call-site count (premature-DRY gate for #158)** — run `grep -rn "emit_reputation_event" crates/ --include="*.rs" -l` from governance-v0 HEAD before brief authorship. If count < 3 files, exclude #158 from scope and note in the plan §3 deferred list. Do NOT include #158 if the count is still 2.
4. **`feedback_envvarguard_audit_window.md` indexing** — verify the lesson is indexed in PMD via `scripts/sync-lessons-to-pmd.sh` before dispatching any impl-task whose brief mentions audit scripts. If not indexed, the pre-queue hybrid search won't surface it.
5. **adr-compliance workflow_dispatch re-trigger still broken** — if mergeStateStatus: UNSTABLE appears at bm-merge and local scan exits 0, do NOT enter the 3-re-trigger loop. Immediate path: owner-acknowledge comment + `--admin`. See v1-quality-r3 retro §2.1. Note this in the bm-merge brief §5 at authorship time.

## 5. Operational rules

- **Polling cadence:** ~10 min, `mcp__junior-brehon__list_tasks`.
- **Brief discipline:** briefs at `.claude/PRPs/briefs/v1-quality-r3b-<role>-<n>.md`, committed to governance-v0 before dispatch (Mode B) OR directly on phase branch (Mode A); pre-queue `memory_search_hybrid` + `/precheck` + §2.4 mandatory file-class lesson injection. Impl-task briefs touching e2e.rs MUST include `feedback_fix_impl_pre_locate_e2e_anchors.md` + `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md` in §3 Required reading. Impl-task briefs touching `crates/api/**/src/**` handlers MUST include `feedback_multi_write_handlers_need_transactions.md`.
- **Shape G status:** Shape G was RE-ENABLED 2026-05-30 (commit `9bd933fe2`). GH Actions billing resets ~2026-06-01. **Check at session start:** if Actions minutes are restored, validate-pending uses Shape G (ci-watcher via `gh run watch`); if still exhausted, use validate-pending-laptop (laptop bat-wrapper). This phase may be the first to run full Shape G — monitor the first run's minutes carefully against the 3000-cap.
- **Windows e2e bat-wrapper:** `cmd //c "scripts\brehon\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true`. Never bare `cargo test` on Windows (libpq.dll missing). See `feedback_windows_e2e_requires_bat_wrapper.md`.
- **Serial cohort:** if both `admin_audit_stream.rs` AND `e2e.rs` are in the same task, no `[P]` on that task (YAML overlap rule). If planner separates handler fix and e2e test fix into separate tasks, they may be `[P]` only if no file overlap.
- **Model tiering:** Planning → Opus 4.8, Impl → Sonnet 4.6, BM/ci-watcher → Haiku 4.5.
- **Clarify gate:** run `/brehon-clarify` on each planning brief before dispatch.
- **Six user gates:** plan approval (gate 1), judgment-heavy DQ (gate 2), CR triage (gate 3), Phase-2 e2e local-vs-dispatch (gate 4), merge confirm (gate 5), retro sign-off (gate 6). Never skip.
- **DQ attribution:** advisor-side commits must match `^(chore|docs)\((advisor|decision-queue)\)`.
- **Lesson promotion:** promote `feedback_envvarguard_audit_window.md` (if not yet indexed) before authoring the planning brief; then run `scripts/sync-lessons-to-pmd.sh`.
- **Duplicate validate-pending guard:** impl-task briefs §4 must include: "Before appending a validate-pending DQ entry, check `.claude/decision-queue.json` for any existing validate-pending entry with the same phase_task number. If one exists, do NOT append a duplicate — the existing entry is authoritative. See `feedback_dq_v3_append_via_helper_script.md`."

## 6. What changed from v1-quality-r3's rule set

- **Production code in scope** — `crates/api/api/src/governance/admin_audit_stream.rs` is a production handler, not a test file. Full CR semantic review applies. The mandatory file-class lesson for `crates/api/**/src/**` handlers applies.
- **Shape G may be fully active** — v1-quality-r3 ran validate-pending-laptop throughout (Shape G billing exhausted). r3b may be the first quality-lane phase to use ci-watcher dispatch. Check at session start.
- **New carry-forward guard on duplicate validate-pending DQ** — added to impl-task brief §4 constraint template following the DQ `65b95cc574c8-002` incident in r3.
- **adr-compliance bypass path is now documented** — no more 3-re-trigger loop; go straight to acknowledge-comment + `--admin` when local scan exits 0 and UNSTABLE blocks. Note in bm-merge brief §5 at authorship time.
- **Two lessons now indexed from quality lane:** `feedback_envvarguard_fixture_lifetime_footgun.md` (from r2 retro, promoted at r3 session start) + `feedback_envvarguard_audit_window.md` (from r3 LESSON trailer `bf3f7201d`). Both are mandatory injections for any e2e.rs brief.

## 7. Catch-fire procedures

Universal triggers (per `.claude/rules/advisor-orchestrator.md` §5.5):
- Junior subagent commits to `governance-v0` or `main` directly.
- `bm-task` opens PR into `main`.
- Phase branch has uncommitted state when Junior reports complete.
- ci-watcher exit code not in the empirical table.
- Cancelling a Junior task whose worker log shows uncommitted code → SSH tar before cancel.

Phase-specific additions:
- **Stop if: the #167 fix removes the `context.settings().get_database_url()` call but does NOT update the e2e test to verify the LISTEN connection works without the env var** — the fix is architecturally incomplete without the test verification. Surface and add the test update before merge.
- **Stop if: plan includes issue #158 (`emit_reputation_event` extraction) but the call-site count is still < 3** — premature-DRY gate must be re-run at plan-time from the phase-branch tip; never assume the count from r3b authorship time.
- **Stop if: gate-4 e2e shows > 0 failed** — no merge until 0 failed confirmed. Baseline is 126/0/5 (v1-quality-r3).
- **Stop if: the bm-merge brief does NOT include the adr-compliance bypass note (acknowledge + `--admin` for UNSTABLE + clean local scan)** — add it before dispatch. Three re-triggers in v1-quality-r3 cost ~20 min for a known infrastructure failure.
- **Stop if: a duplicate validate-pending DQ entry appears for the same phase_task** — the first entry is authoritative; resolve the duplicate immediately via the advisor DQ answer commit rather than leaving both in pending.

## 8. Archive after v1-quality-r3b

Run `/brehon-phase-transition v1-quality-r3b <next-quality-slice>`. This skill will: close `workflow_state_v1_quality_r3b_new.md`, delete `workflow_state_v1_redaction_r1.md` (the two-ago record at that point), create the next skeleton, write the next bootstrap file, update brehon-fork MEMORY.md, commit on governance-v0. This bootstrap file stays in `.claude/PRPs/handovers/` as its own archive — git history is the archive; no move.

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `49dd95685` (captured 2026-06-01) — `docs(retro): v1-redaction-r1 — GDPR scrubber hardening retro + 2 lessons`
- Phase branch HEAD: not yet created (branch `phase-v1-quality-r3b` cut at bm-cut)
- Recent governance-v0 commits:

  ```
  49dd95685 docs(retro): v1-redaction-r1 — GDPR scrubber hardening retro + 2 lessons
  e66fd1a38 chore(runlog): commit bm-triage runlog + stray untracked artifacts (v1-quality-r3c closeout)
  beee2095c chore(brehon): close v1-quality-r3c, bootstrap v1-redaction-r1
  851ce382d docs(templates): bm-merge verb-constraint — daemon-local ref freshness check (v1-quality-r3c CF-3)
  df7a76de1 docs(lessons): bm-merge daemon-local ref staleness + update-ref vs reset-hard (v1-quality-r3c CF-1/CF-4)
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff — 0 pending entries on governance-v0 at 2026-06-01)
```

## Stop-and-ask tripwires

- `Stop and ask if:` the plan includes any migration under `crates/db_schema/migrations/` — this phase is a single handler refactor only; a new migration is a scope violation.
- `Stop and ask if:` the issue #167 fix shape involves adding a new function parameter to `admin_audit_stream()` that changes its public signature visible to AP federation callers — confirm scope is confined to the internal LISTEN channel only; an AP-visible signature change would need an ADR review.
- `Stop and ask if:` issue #158 scope is included in the plan but `emit_reputation_event` call-site count is < 3 — run `grep -rn "emit_reputation_event" crates/ --include="*.rs" -l` from the phase-branch tip and count files before confirming.
- `Stop and ask if:` gate-4 full-e2e shows any test count regression below 126 passed (not just > 0 failed) — even a passing-but-fewer count means a test moved to ignored unexpectedly; verify before merge.
- `Stop and ask if:` the planner's #167 fix shape requires the handler to own a new `PgPool` / `tokio_postgres::Client` that bypasses the injected `LemmyContext.pool` — a second independent pool in the same handler is an architectural anti-pattern; surface and confirm before plan approval.
