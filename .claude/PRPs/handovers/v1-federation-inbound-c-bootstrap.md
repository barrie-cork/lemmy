---
phase: v1-federation-inbound-c
plan: (not yet authored — .claude/PRPs/plans/v1-federation-inbound-c.plan.md to be created via /prp-core:prp-plan after the planning Junior)
phase_branch: phase-v1-federation-inbound-c   # not yet created — cut at bm-cut
worktree: C:/Users/barri/Developer/brehon-fork-fed-in-c   # created at bm-cut; until then canonical C:/Users/barri/Developer/brehon-fork
authored: 2026-05-20
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the v1-federation-inbound-c advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon v1-federation-inbound-c.** This is a fresh session (or a resumed one). The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-fed-in-c` once `bm-cut` creates the lane worktree (per `.claude/rules/multi-lane-worktree.md`). This is the **second** fully-brehon-fork-native transition (the first was fed-in-b→fed-in-c which you are resuming into — fed-in-a's prior homeserver advisor-context chain has been retired; do NOT read any `homeserver/.claude/advisor-context-*.md`).

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane. Until bm-cut, you are on canonical `brehon-fork` / `governance-v0` (meta-edit lane: briefs + plan land on trunk). At handoff the lane worktree `C:/Users/barri/Developer/brehon-fork-fed-in-b` still exists with the deleted phase branch — user is expected to remove it post-transition (`git worktree remove ../brehon-fork-fed-in-b && git branch -d phase-v1-federation-inbound-b`); the canonical bm-cut for fed-in-c creates a fresh `brehon-fork-fed-in-c` worktree.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0 origin/governance-v0` — both must be `9ddc7d98d` after this transition's commit pushes (see §"Git state at handoff"). If origin lags, push before any Junior dispatch (Junior workers branch from the committed+pushed trunk tip).
3. Read `.claude/decision-queue.json` (and `scripts/brehon/resolve-dq-canonical.sh v1-federation-inbound-c` once a phase branch exists) for pending entries since handoff; compare against §"Decision-queue snapshot" below. At handoff: **pending=[] (empty)**.
4. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_v1_federation_inbound_c.md` is the running-state scratchpad (most-recent "Session handoff block" is authoritative on resume). Read `workflow_state_v1_federation_inbound_b.md` ONCE for carry-forward (it is the CLOSED record).

## Next concrete action

No plan exists yet. **Two scoping signals must be reconciled before authoring the planning brief:**

1. **fed-in-b retro carry-forwards** name 3 HIGH-priority scope candidates (a) reader-side append-history defect (`get_inbound_config_int` + mirror lack `.order_by(governance_config::valid_from.desc())`); (b) Copilot DoS-hardening family (rate-map LRU + key hash + TOCTOU eviction); (c) Phase-6 convention-divergence audit over fed-in-b's `inbox.rs` additions. (a) and (b) are code work; (c) is a meta-audit.
2. **`brehon-conformance-audit` skill/subagent is in flight** — concurrent advisor session authored a planning brief at commit `57ce4c322` on gov-v0 (`chore(advisor): author brehon-conformance-audit planning brief — supersedes 2026-05-20 guidance + integrates Rust best-practices research`). Per fed-in-b retro §3 the audit may be subsumed by this skill/subagent — DO NOT scope (c) into fed-in-c until you've read that planning brief and confirmed whether the audit is delivered standalone (skill/subagent) or as a sub-phase task. If the standalone path lands first, fed-in-c can focus narrowly on (a)+(b).

**Then author `.claude/PRPs/briefs/v1-federation-inbound-c-planning-1.md`** with the agreed scope, commit it to `governance-v0`, then **run `/brehon-clarify .claude/PRPs/briefs/v1-federation-inbound-c-planning-1.md`** → resolve every clarify-DQ → queue the `[role:planning]` Junior task. Skipping `/brehon-clarify` on a planning brief is a process breach (advisor-orchestrator §3.1).

---

## 1. v1-federation-inbound-c in one paragraph

fed-in-c is the **federation-inbound storage-hardening / read-correctness slice** of the federation-inbound v1 track. fed-in-a (PR #138, merged 2026-05-19, `7af873731`) shipped the additive schema/model/trust-state foundation. fed-in-b (PR #139, merged 2026-05-20, `413ef5899`) wired HTTP-path enforcement (`wrap_governance_inbound` + per-handler patches + `receive_remote_moderation_label` + rate-limit/replay + Phase-6 fixture + 5 handler e2e). fed-in-c addresses the **load-bearing carry-forwards from fed-in-b's retro §3/§5/§7**: (a) the reader-side append-history defect in `get_inbound_config_int` (`crates/apub/activities/src/governance/inbox.rs:421-438`) AND its mirror in `publish_trust_attestation.rs:142-152` — both lack `.order_by(governance_config::valid_from.desc())` and return arbitrary rows when `governance_config` accumulates admin-edit history; (b) the Copilot DoS-hardening family — unbounded in-memory rate-limit maps (`inbox.rs:473`) + raw-string HashMap keys (`publish_trust_attestation.rs:165`) + TOCTOU eviction (`inbox.rs:698`); (c) optionally the Phase-6 convention-divergence audit over fed-in-b's new code in `inbox.rs` (deferred-NOT-fixed per user-directive a 2026-05-19) — but only IF the `brehon-conformance-audit` skill/subagent (in flight at `57ce4c322`) hasn't landed standalone first. DoD: reader-side `valid_from.desc()` ordering on both call sites + LRU-bounded or hashed rate-limit maps + transactional or single-statement eviction with row lock + the 3 admin REST endpoints + pseudonym rendering + step-up auth (the originally-`-c` scope from fed-in-b PRD) — OR a narrower slice that defers admin-REST/pseudonym/step-up to fed-in-d. The exact slice depends on the planning Junior's scoping; advisor signs off at gate-1.

## 2. Why v1-federation-inbound-c is harder/easier than v1-federation-inbound-b

- **Easier in one respect:** the HTTP-path is already wired (fed-in-b) and Phase-2 e2e covers the enforcement gates (102/0/5 baseline). fed-in-c's reader-fix is mechanical (`.order_by(valid_from.desc())` on 2 call sites — `crates/apub/activities/src/governance/inbox.rs:421-438` + `publish_trust_attestation.rs:142-152`); the test fixture pattern is established (fix-impl-7 demonstrated `UPDATE governance_config` for append-history awareness).
- **Not easier — concurrency-safety design.** The Copilot DoS-hardening (copilot-1/2/3) requires real concurrency design: choosing between in-process LRU (low memory ceiling) vs Postgres-backed counters (per-instance scale) vs hash-keyed maps (cheap mitigation, leaks at sustained load). The Copilot finding bodies are well-described but the right answer is judgment-heavy: an LRU with N=10k keys × 24 bytes = 240KB is cheap; a hash-keyed map without bound still leaks; a Postgres-backed counter adds a DB write per inbound. **Gate-1 plan approval is load-bearing here** — don't auto-approve.
- **Not easier — TOCTOU eviction concurrency.** `evict_oldest_unreviewed_if_needed` (`inbox.rs:698`) under multi-receive contention can over-evict AND record `storage_cap_evicted` when DELETE affects 0 rows. The fix is either (a) single atomic SQL with `RETURNING` (Diesel: `.returning(...)`) OR (b) transaction with row-level lock (`SELECT ... FOR UPDATE SKIP LOCKED` then DELETE). Both work; both have failure modes. Plan §4 watchpoint must name which.
- **Easier — append-history reader pattern.** `crates/db_schema/src/source/governance/governance_config.rs` likely already has a "latest row per key" helper (`governance_config_current` view or similar). If yes, the reader fix is "use the existing helper"; if no, the fix is a `.order_by(valid_from.desc()).limit(1).get_result()` chain. Check the view existence first (`grep -rn "governance_config_current" crates/db_schema/src/`).
- **Harder for governance** — fed-in-b's user-directive Phase-6 convention-divergence audit is a meta-task. The audit-tooling generalization (user-directive b) lives in the `brehon-conformance-audit` planning brief on gov-v0 (`57ce4c322`). The next session should read that brief BEFORE scoping (c) — to know whether audit is delivered as `/brehon-conformance-audit` skill OR as a fed-in-c task.

## 3. Lessons from v1-federation-inbound-b that apply to v1-federation-inbound-c

Reference by filename — do not duplicate. From the fed-in-b retro (`.claude/PRPs/reports/v1-federation-inbound-b-retro.md`) per-role signals + carry-forward:

**Advisor-side:**
- `feedback_thin_wakeup_prompts_verify_live_state.md` — **PROMOTE-NOW** at fed-in-c start (≥4× this session segment). Wakeup prompts must be ≤2 lines (wake trigger + resume-stage hint), not embedded decision trees. Decision logic lives in this file + advisor-orchestrator.md, read at fire time.
- `feedback_junior_finalize_skips_when_worker_pre_pushes.md` — **PROMOTE-NOW** to a definite lesson + file upstream `barrie-cork/lemmy` issue. 3 occurrences this phase alone (Junior #348/#349/#350); recovery is `ssh homeserver 'cd /srv/brehon-fork && git push origin <branch>'`. Expect this to recur in fed-in-c; bake the recovery into impl-task / bm-task post-condition verification.
- `feedback_bm_false_success_advisor_post_condition_catch.md` — held across fed-in-a + fed-in-b. After EVERY Junior "done", verify the real-world effect (PR state, branch tip, file existence), never the self-report. fed-in-b had 2 BM Junior misses on bm-pr (phase-runlog absent + findings.yaml shell skipped — both low-blast non-blocking, but the discipline catches them).
- `feedback_background_task_notification_lies.md` — operational discipline; read the explicit `E2E_EXIT_0`/`CHECK_EXIT_0`/`CHAIN_DONE` marker; never trust the harness completion notification. fed-in-b verified twice (round-1 fail + round-2 pass).
- **CR poll-2 mis-attribution operating note** (fed-in-b retro §3) — when CR posts a 2nd review during a cherry-pick window, the review's `commit_id` shows the cherry-pick tip not the next post-fix-impl tip. Reliable signal: `gh pr view --json statusCheckRollup` (state SUCCESS/PENDING/FAILURE), not the review's commit_id alone.
- `feedback_default_local_testing.md` / `project_laptop_canonical_cargo_runner.md` — Shape G is SUSPENDED until 2026-06-01 (DQ #229, `project_shape_g_suspended_2026_05_16.md`). All cargo runs laptop-shape (`kind: "validate-pending-laptop"`, advisor-laptop mutates). If fed-in-c crosses 2026-06-01, re-check DQ #229 before assuming Shape G is back.

**Planning-side:**
- **Crate-qualify Lemmy-1.0 schema/file paths** (established in fed-in-a, held in fed-in-b). All paths: `crates/apub/activities/src/governance/inbox.rs`, `crates/apub/activities/src/governance/publish_trust_attestation.rs`, `crates/db_schema/src/source/governance/governance_config.rs`, `migrations/2026-04-18-000000-0000_add_governance_config/up.sql:38-50`. No bare "schema.rs".
- §2.4 mandatory file-class lesson injection: any `crates/server/tests/e2e.rs` edit → `feedback_lemmy_error_no_std_error.md` (Case A — mirror the existing `v1_federation_inbound_b_fixtures` sibling module's `LemmyResult<()>` shape verbatim) + `feedback_async_pool_test_pattern.md`; ≥2 e2e edits → + `feedback_junior_worker_e2e_edit_hang.md`. Any handler doing 2+ DB writes → `feedback_multi_write_handlers_need_transactions.md` (the eviction-fix or transactional rate-counter is a multi-write candidate).
- **§15 fail-rate inflection** (fed-in-b retro §2 Planning) — Tasks 4/5/6 each needed fix-impl; Tasks 7/8/9 first-try §15-GREEN. The §2.4 worker-side pre-push cargo-check discipline (`feedback_fix_impl_pre_push_cargo_check.md`) bedded in over Cohort B. Carry: insist on §2.4 from Task 1, not Task 4.

**Impl-side:**
- `feedback_fix_impl_pre_push_cargo_check.md` + `feedback_fix_impl_enumerate_all_callsites.md` — before any struct/signature change brief, `rg "<Symbol>" crates/ tests/` enumerate the FULL N callsites; brief lists all N; file-edit cap is the count of distinct files (NOT default ≤3). fed-in-b fix-impl-7 demonstrated this discipline (worker disambiguated brief-said-cap-1 vs reality-cap-11 `INSERT INTO governance_config` from codebase).
- `feedback_lemmy_error_no_std_error.md` — Case A discipline (`LemmyResult<()>` outer + bare `?`) is canonical for fed-in-b fixtures module. fed-in-c e2e additions mirror this verbatim.

**BM-side:**
- **bm-merge first-try L14 REVISED success** (Junior #351 ✓ — contrast with v1-ship-1-r2 Junior #322). The L14 REVISED rule (`feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md`, 2026-05-18) + `bm-runlog.md` `merge=union` `.gitattributes` (`fd96360c9`) both held. Continue.
- **bm-pr deliverable contract** — Junior #349 missed phase-runlog write + findings.yaml shell. Decide before fed-in-c's first bm-pr: tighten the brief contract OR accept the gitignored-state risk.

## 4. v1-federation-inbound-c-specific watchlist

Each cites a specific file/table/line + a forward gate:

1. **`crates/apub/activities/src/governance/inbox.rs:421-438`** — `get_inbound_config_int` reads via `.first::<Option<i64>>(conn)` WITHOUT `.order_by(governance_config::valid_from.desc())`. Plan §13 task must name this exact line range + the fix. Gate: plan §10 MIRROR ref shows the corrected pattern (mirror with `.order_by(valid_from.desc()).limit(1)` OR a `governance_config_current` view if one exists in `crates/db_schema/src/source/governance/`).
2. **`crates/apub/activities/src/governance/publish_trust_attestation.rs:142-152`** — the mirror site with the same defect. Gate: plan §13 enumerates BOTH file sites; §G4 callsite-enumeration discipline applies (`rg "get_inbound_config_int|governance_config::table" crates/` before authoring).
3. **`crates/apub/activities/src/governance/inbox.rs:473`** (rate-limit maps) AND **`publish_trust_attestation.rs:165`** (subject_url key) — Copilot copilot-2 + copilot-3 DoS hardening. Gate: plan §4 watchpoint NAMES the chosen mitigation (LRU vs hash-key vs Postgres-backed counter); plan §13 task body has the literal struct/type change; e2e asserts memory growth bounded across N unique peers.
4. **`crates/apub/activities/src/governance/inbox.rs:698`** — `evict_oldest_unreviewed_if_needed` TOCTOU. Gate: plan §4 watchpoint names whether single-atomic-SQL or transactional-row-lock; e2e asserts no over-eviction under concurrent receives + no spurious `storage_cap_evicted` log on 0-row DELETE.
5. **`crates/db_schema/src/source/governance/governance_config.rs`** — check FIRST whether a `governance_config_current` view OR a `latest_value_for(scope, key)` helper exists. If yes, the reader fix is "use the helper"; if no, the fix is the `.order_by(valid_from.desc()).limit(1)` chain (and possibly authoring the helper as part of fed-in-c). Gate: planning brief §3 Required reading includes the result of `grep -rn "governance_config_current" crates/db_schema/src/`.
6. **`brehon-conformance-audit` planning brief on gov-v0 (commit `57ce4c322`)** — read this BEFORE scoping the Phase-6 convention-divergence audit into fed-in-c. If the standalone skill/subagent lands first, fed-in-c can focus narrowly on (a) reader fix + (b) DoS hardening. Gate: planning brief §0 confirms which path is live.
7. **Migrations** — fed-in-c is reader-side + concurrency-side. The reader fix is code-only (no schema). The DoS hardening is code-only (in-memory data structures). The eviction fix is code-only or SQL-helper (no new tables). **If the generated plan proposes a new migration under `migrations/**`, catch-fire — that is a scope violation.** The schema substrate is fed-in-a's; fed-in-b added no schema; fed-in-c does not add schema.

## 5. Operational rules

Carry fed-in-b's rule set forward; adjust per the retro change bullets:

- **Polling cadence:** ~10 min `mcp__junior-brehon__list_tasks` (status only); on transition `show_task` + `git fetch` + read DQ + triage + queue next per advisor-orchestrator §3.1 stage-shape.
- **Brief discipline:** briefs at `.claude/PRPs/briefs/v1-federation-inbound-c-<role>-<n>.md`, committed to `governance-v0` BEFORE the Junior task. Dispatch string <100 chars: `[role:<role>] <slug> — see .claude/PRPs/briefs/<file>.md`. Pre-queue: `memory_search_hybrid` (limit 5) + `/precheck` + §2.4 mandatory file-class lesson injection (walk the file list against the table — no judgment call).
- **LemmyResult Case A override for any e2e brief:** the `v1_federation_inbound_b_fixtures` sibling module in `crates/server/tests/e2e.rs` uses `LemmyResult<()>`. Any fed-in-c e2e edit mirrors that shape verbatim (Case A per `feedback_lemmy_error_no_std_error.md`) — read the sibling at its line range BEFORE authoring the brief (canonical-schema-first gate).
- **Shape G status:** SUSPENDED until 2026-06-01 (DQ #229). fed-in-c runs `kind: "validate-pending-laptop"` per advisor-orchestrator §5.2. **If fed-in-c crosses 2026-06-01** (likely given storage-hardening can span days), re-check DQ #229 first thing in the new-day session — Shape G may be re-enabled and the validate-pending shape changes.
- **Windows e2e invocation:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true`. NEVER bare `cargo test` (libpq.dll needs the bat wrapper's vcpkg PATH); NEVER `-p lemmy_server --features full` (no `full` feature; use `--workspace`). Read the explicit exit marker, never the bg notification.
- **Cohort/serial discipline:** laptop is the canonical cargo runner (P50 64 GB); EliteDesk = Junior orchestration only. Per-task `[P]` cohort dispatch per advisor-orchestrator §4 (YAML overlap check + `requires:` dependency check + budget check). The reader fix on `inbox.rs:421-438` and the mirror on `publish_trust_attestation.rs:142-152` are likely `[P]` (different files); the DoS hardening on `inbox.rs:473`/`:698` is the SAME file as the reader fix → likely NON-`[P]` against it.
- **Model tiering:** Planning→Opus/xhigh, Impl→Sonnet/medium, BM/ci-watcher→Haiku/low (`feedback_brehon_subagent_model_effort_assignments.md`). Non-Anthropic in daemon logs = drift.
- **The 6 user gates (never skip):** plan approval (after DoD smoke + watchpoint-specificity), judgment-heavy DQ, CR triage, Phase-2 e2e local-vs-dispatch (never auto-pick — `feedback_e2e_local_or_dispatch_user_choice.md`; recommend local first per `feedback_default_local_testing.md`), merge confirm, retro sign-off.
- **DQ attribution:** advisor commits writing `answered_by` must have subject `^(chore|docs)\((advisor|decision-queue)\)`. Mid-task Junior DQ writes push to the worker branch immediately.
- **Memory headroom:** no bulk `crates/server/tests/e2e.rs` reads (it is ~15600+ lines post-fed-in-b — Junior workers hang on full-file Edit per `feedback_junior_worker_e2e_edit_hang.md`; advisor reads only targeted line ranges).
- **Multi-lane:** if another `phase-v1-*` lane is active (the `brehon-conformance-audit` thread may run concurrently), fed-in-c gets its own worktree `brehon-fork-fed-in-c` at bm-cut. The lane-dedicated session writes phase-branch DQ for fed-in-c only. PMD is cross-lane shared (canonical `brehon-fork/.project-memory/memory.db`); the worktree's `.mcp.json` MUST pin the absolute canonical `PROJECT_MEMORY_DB` (`feedback_pmd_cross_lane_canonical_db.md`).
- **Wakeup prompt discipline** (NEW from fed-in-b PROMOTE-NOW): wakeup prompts ≤2 lines. Wake trigger + resume-stage hint. Decision logic lives in advisor-orchestrator.md, read at fire time.
- **Finalize-push-skip recovery** (NEW from fed-in-b PROMOTE-NOW): after EVERY Junior "done" where the worker pre-pushed, verify `origin/<branch>` matches daemon-local `<branch>`. If daemon ahead, run `ssh homeserver 'cd /srv/brehon-fork && git push origin <branch>'` immediately. Bake into impl-task / bm-task post-condition verification.

## 6. What changed from v1-federation-inbound-b's rule set

- **fed-in-b was HTTP-path enforcement (handler-side wiring); fed-in-c is storage-hardening (reader-side correctness + concurrency-safety).** The §2.4 file-class injections that fire change: fed-in-b fired multi-write-handlers + e2e Case-A; fed-in-c fires `feedback_multi_write_handlers_need_transactions.md` (for the eviction fix if transactional) + possibly `feedback_pg_advisory_xact_lock_void_decode.md` (if SKIP LOCKED is used) + the e2e Case-A trio.
- **No new migrations expected.** fed-in-b's only schema-side change was the migration revert-list extension in fix-impl-1 + the seed-row UPDATE in fix-impl-7 (TEST-FIXTURE not production). fed-in-c is code-only. If the plan proposes a migration, that's a scope violation.
- **Two PROMOTE-NOW lessons land at fed-in-c start:** `feedback_thin_wakeup_prompts_verify_live_state.md` (definite) + `feedback_daemon_finalize_push_skip_load_bearing_recovery.md` (to author). Both should be referenced from the planning brief.
- **CR poll-2 mis-attribution** — new operating note added to bm-poll-cr discipline (statusCheckRollup is the truth, not the review commit_id).
- **bm-merge first-try L14 REVISED is the new baseline.** Do not regress to pre-merge runlog ordering.

## 7. Catch-fire procedures

Universal triggers from `.claude/rules/advisor-orchestrator.md` §5.5 PLUS fed-in-c-specific:

- **Plan proposes a new migration under `crates/db_schema/migrations/**` or `migrations/**`** → catch-fire: fed-in-c is reader-side + concurrency-side; the schema substrate is fed-in-a's. A new migration is a scope violation.
- **Plan proposes touching the `wrap_governance_inbound` enforcement order** → catch-fire: fed-in-b shipped the canonical enforcement order (trust → size → schema → rate → replay → handler) and Phase-2 e2e codifies it (102/0/5). fed-in-c hardens internals, NOT the enforcement contract.
- **Plan proposes removing fed-in-b's `ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED`** → catch-fire: best-effort persist-error logging is a load-bearing audit guarantee.
- **§G4 cycle-count meta-rule:** if a validate-pending fails 3× with the same `(error_class, file_basename)` for one cohort member → HARD REFUSAL catch-fire regardless of allowlist.
- **`get_inbound_config_int` reader still returns arbitrary rows after the planned fix lands** (Phase-2 e2e shows duplicate-emission or wrong-cap regression on a 2nd config row) → catch-fire: the fix is wrong-shaped; surface, do not auto-fix-forward.
- **BM "done" + PR still OPEN** → auto-catch-fire (fed-in-b precedent — `feedback_bm_false_success_advisor_post_condition_catch.md`).

## 8. Archive after v1-federation-inbound-c

The standard close: run `/brehon-phase-transition v1-federation-inbound-c <next-id>` (likely `v1-federation-inbound-d` if more federation-inbound work remains, OR a new lane if the storage-hardening completes the federation-inbound track). The skill will: mark `workflow_state_v1_federation_inbound_c.md` CLOSED, delete `workflow_state_v1_federation_inbound_b.md` (the two-phases-ago record in this lane chain — unambiguous, safe), create the next skeleton, write the next bootstrap file, update brehon-fork MEMORY.md, commit on governance-v0. This bootstrap file stays in `.claude/PRPs/handovers/` as its own archive — git history is the archive; no move.

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD (local): `9ddc7d98d` (captured 2026-05-20) — `docs(retro): v1-federation-inbound-b — federation-inbound HTTP-path enforcement shipped (PR #139 merged 413ef5899)`
- governance-v0 (origin): `9ddc7d98d` — same; pushed at retro commit. This transition's commit (the new bootstrap file) advances both to a NEW hash; verify origin == local + transition commit at session start.
- Phase branch HEAD: not yet created (branch `phase-v1-federation-inbound-c` cut at bm-cut)
- fed-in-b merge: PR #139 → `governance-v0`, merge commit `413ef5899`, merged 2026-05-20T13:45:40Z, head branch `phase-v1-federation-inbound-b` deleted on origin (L16 ✓)
- Recent governance-v0 commits (`git -C C:/Users/barri/Developer/brehon-fork log --oneline -6 governance-v0`):

  ```
  9ddc7d98d docs(retro): v1-federation-inbound-b — federation-inbound HTTP-path enforcement shipped (PR #139 merged 413ef5899)
  bbe4def06 chore(decision-queue): advisor clarify pass on brehon-conformance-audit-planning-1 — DQ #291-#294
  4480a1bdb chore(bm): merge PR #139 complete — runlog COMPLETE entry
  413ef5899 Merge pull request #139 from barrie-cork/phase-v1-federation-inbound-b
  a89eddb9c chore(advisor): author bm-merge brief for PR #139 — L14 REVISED post-merge runlog
  57ce4c322 chore(advisor): author brehon-conformance-audit planning brief — supersedes 2026-05-20 guidance + integrates Rust best-practices research
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff) — .claude/decision-queue.json pending=[] (resolved=91, schema_version=2). All fed-in-b validate-pending-laptop DQs (#285 task5 / #286 task6 / #287 task7 / #288 task8 / #289 task9 / #290 Phase-2 e2e) result:pass and resolved.
Archived-but-relevant: DQ #229 (advisor 2026-05-16, ADVISORY-LOG) — Shape G re-enable 2026-06-01. NOT pending (resolved earlier) but a LIVE forward reminder: if fed-in-c crosses 2026-06-01, re-check whether Shape G validation is back before assuming validate-pending-laptop. Concurrent advisor session's clarify pass DQ #291-#294 (commit bbe4def06) is on `brehon-conformance-audit-planning-1` — separate thread; not blocking fed-in-c. Read it before scoping the Phase-6 convention audit into fed-in-c.
```

## Stop-and-ask tripwires

- Stop and ask if: the generated plan introduces a new migration under `crates/db_schema/migrations/**` or `migrations/**` — fed-in-c is reader-side + concurrency-side; the schema substrate is fed-in-a's. A new migration is a scope violation.
- Stop and ask if: the plan proposes modifying `wrap_governance_inbound`'s gate ORDER (trust → size → schema → rate → replay → handler) — that's fed-in-b's contract; fed-in-c hardens internals, not the enforcement order.
- Stop and ask if: `scripts/brehon/resolve-dq-canonical.sh v1-federation-inbound-c` (once a phase branch exists) shows pending DQ entries that are NOT in the §"Decision-queue snapshot" above — a mid-flight blocker landed; triage before continuing the stage-shape.
- Stop and ask if: the `brehon-conformance-audit` planning brief (commit `57ce4c322` on gov-v0) has NOT been read before scoping the Phase-6 convention-divergence audit into fed-in-c — the audit may be delivered standalone (skill/subagent) per user-directive b; do not double-scope.
- Stop and ask if: a validate-pending entry fails a 3rd time with the same `(error_class, file_basename)` tuple for one cohort member — §G4 hard-refusal (the recipe family is wrong-shaped; do not auto-queue a 3rd fix-impl; this is a re-plan signal).
- Stop and ask if: fed-in-c session crosses 2026-06-01 UTC without re-checking DQ #229 — Shape G may be re-enabled and the validate-pending shape changes (`kind: "validate-pending"` on GH Actions vs `kind: "validate-pending-laptop"` on laptop).
