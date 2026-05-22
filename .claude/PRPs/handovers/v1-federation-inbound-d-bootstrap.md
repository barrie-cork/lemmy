---
phase: v1-federation-inbound-d
plan: (not yet authored — .claude/PRPs/plans/v1-federation-inbound-d.plan.md to be created via /prp-core:prp-plan after the planning Junior)
phase_branch: phase-v1-federation-inbound-d   # not yet created — cut at bm-cut
worktree: C:/Users/barri/Developer/brehon-fork-fed-in-d   # created at bm-cut; until then canonical C:/Users/barri/Developer/brehon-fork
authored: 2026-05-22
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the v1-federation-inbound-d advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon v1-federation-inbound-d.** This is a fresh session (or a resumed one). The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-fed-in-d` once `bm-cut` creates the lane worktree (per `.claude/rules/multi-lane-worktree.md`). The fed-in-c→fed-in-d transition is fully brehon-fork-native; do NOT read any `homeserver/.claude/advisor-context-*.md`.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane. Until bm-cut, you are on canonical `brehon-fork` / `governance-v0` (meta-edit lane: briefs + plan land on trunk). At handoff the lane worktree `C:/Users/barri/Developer/brehon-fork-fed-in-c` still exists with the deleted phase branch — user is expected to remove it post-transition (`git worktree remove ../brehon-fork-fed-in-c && git branch -d phase-v1-federation-inbound-c`); the canonical bm-cut for fed-in-d creates a fresh `brehon-fork-fed-in-d` worktree.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0 origin/governance-v0` — both must be at-or-ahead of `<HANDOFF-HASH>` (`a85b5bf2f` plus this transition's commit; see §"Git state at handoff"). If origin lags, push before any Junior dispatch (Junior workers branch from the committed+pushed trunk tip).
3. Read `.claude/decision-queue.json` (and `scripts/brehon/resolve-dq-canonical.sh v1-federation-inbound-d` once a phase branch exists) for pending entries since handoff; compare against §"Decision-queue snapshot" below. At handoff: pending may include DQ #326 (conformance-audit stale leftover — see §3 carry-forward; do NOT cross-lane-edit it).
4. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_v1_federation_inbound_d.md` is the running-state scratchpad (most-recent "Session handoff block" is authoritative on resume). Read `workflow_state_v1_federation_inbound_c.md` ONCE for carry-forward (it is the CLOSED record).

## Next concrete action

No plan exists yet. **Scope decision needed before authoring the planning brief.** The fed-in-c retro surfaced 4 options for the (b) Copilot DoS-hardening family (deferred from fed-in-c). User signalled "continue fed-in-d" at the phase-transition gate but did NOT pick an a/b/c/d option:

- **option-a:** ship (b) full next sub-phase (largest scope; ~5-8 tasks across LRU bounds, key-hash, TOCTOU eviction, e2e for each)
- **option-b:** ship narrow (b) subset (2 tasks: inbox-size cap + per-actor attestation cap, e2e tests for each) ← retro-favoured cut
- **option-c:** defer (b) for a different fed-in-* gap (e.g. ADR-014 fork-only AP types audit; rate-limit-counter cleanup scheduler hardening; additional federation-mod-roots Phase-6 coverage)
- **option-d:** defer (b) until pilot threshold

**Then:** surface the option choice to user via `AskUserQuestion` (gate 1 pre-plan); author `.claude/PRPs/briefs/v1-federation-inbound-d-planning-1.md` with the agreed scope; commit it to `governance-v0`; **run `/brehon-clarify .claude/PRPs/briefs/v1-federation-inbound-d-planning-1.md`** → resolve every clarify-DQ → queue the `[role:planning]` Junior task. Skipping `/brehon-clarify` on a planning brief is a process breach (advisor-orchestrator §3.1).

---

## 1. v1-federation-inbound-d in one paragraph

fed-in-d is the **federation-inbound DoS-hardening slice** of the federation-inbound v1 track. fed-in-a (PR #138, `7af873731`) shipped the additive schema/model/trust-state foundation. fed-in-b (PR #139, `413ef5899`) wired HTTP-path enforcement (`wrap_governance_inbound` + per-handler patches + `receive_remote_moderation_label` + rate-limit/replay + Phase-6 fixture + 5 handler e2e). fed-in-c (PR #144, `dc9bf17a2`) shipped the reader-side append-history fix on `get_inbound_config_int` + mirror in `publish_trust_attestation`, plus the e2e regression. fed-in-d addresses the **Copilot DoS-hardening family carried forward from fed-in-c retro §Carry-forward**: (b) the in-memory rate-limit maps that grow unboundedly under hostile peer load (`crates/apub/activities/src/governance/inbox.rs:473`), the raw-string HashMap keys that allow cheap memory inflation (`publish_trust_attestation.rs:165`), and the TOCTOU eviction race (`inbox.rs:698`) that can over-evict OR record `storage_cap_evicted` when DELETE affects 0 rows. DoD depends on the scope option chosen: option-a covers the full (b) family + e2e per axis; option-b limits to inbox-size cap + per-actor attestation cap + e2e for each. The exact slice depends on the planning Junior's scoping after the gate-1 option decision; advisor signs off at gate-1.

## 2. Why v1-federation-inbound-d is harder/easier than v1-federation-inbound-c

- **Harder — concurrency-safety design.** fed-in-c was mechanical (2-site `.order_by` add + 1 mirror e2e). fed-in-d's Copilot DoS-hardening requires real concurrency design: choosing between in-process LRU (low memory ceiling), Postgres-backed counters (per-instance scale, +1 DB write per inbound), or hash-keyed maps (cheap mitigation, leaks at sustained load). An LRU with N=10k keys × 24 bytes = 240KB is cheap; a hash-keyed map without bound still leaks; a Postgres-backed counter adds a DB write per inbound. **Gate-1 plan approval is load-bearing here** — don't auto-approve.
- **Harder — TOCTOU eviction concurrency.** `evict_oldest_unreviewed_if_needed` (`inbox.rs:698`) under multi-receive contention can over-evict AND record `storage_cap_evicted` when DELETE affects 0 rows. The fix is either (a) single atomic SQL with `RETURNING` (Diesel: `.returning(...)`) OR (b) transaction with row-level lock (`SELECT ... FOR UPDATE SKIP LOCKED` then DELETE). Both work; both have failure modes. Plan §4 watchpoint must name which.
- **Easier — established test patterns.** The Phase-2 e2e baseline is now 103/0/5 (fed-in-c's regression test held); the `v1_federation_inbound_b_fixtures` sibling module's `LemmyResult<()>` Case A shape is canonical for any new e2e edit. fed-in-c's mirror-pattern (mirror canonical sibling verbatim per advisor-orchestrator.md §3.6 canonical-schema-first gate) held.
- **Easier — Phase-2 e2e laptop-local path is bedded in.** DQ #229 (Shape G SUSPENDED until 2026-06-01) routes all cargo runs through validate-pending-laptop. fed-in-d may cross 2026-06-01 — re-check DQ #229 before assuming Shape G is back.
- **Harder — multi-task concurrent-write coordination.** If option-a (full (b) family) ships, the eviction fix + rate-counter rebuild + key-hash change touch overlapping data structures in `inbox.rs`. Plan §13 cohort `[P]` markers must reflect file overlap; advisor cohort-dispatch §4.1 step 4 YAML overlap check is the gate.

## 3. Lessons from v1-federation-inbound-c that apply to v1-federation-inbound-d

Reference by filename — do not duplicate. From the fed-in-c retro (`.claude/PRPs/reports/v1-federation-inbound-c-retro.md`) per-role signals + carry-forward:

**Advisor-side:**
- `feedback_bm_false_success_advisor_post_condition_catch.md` — **5× CONFIRMED** (now: #322, #392, #394, #404, #408). After EVERY BM Junior "done", verify the real-world effect (PR state, branch tip, file existence, runlog append), never the self-report. fed-in-c had 2 occurrences (#404 bm-pr missed phase-runlog + findings.yaml shell; #408 bm-poll-cr violated brief §4 bucket-blank constraint). Both recovered advisor-side. Continue: advisor post-condition is the only reliable catch.
- `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md` (REVISED 2026-05-18) — bm-merge first-try success on Junior #410 ✓. The post-merge runlog ordering + `bm-runlog.md` `merge=union` `.gitattributes` (`fd96360c9`) both held. Continue.
- `feedback_falsifiable_hypothesis_before_structural_fix.md` — fed-in-c surfaced DQ #338 (daemon-bug) RCA revision mid-session: original hypothesis (daemon finalize wrong-ref reset) was FALSIFIED by sub-agent forensics; true vector was advisor lane-agent ssh-reset against shared daemon checkout. Resolved option-b via PreToolUse hook (shipped by concurrent session). Carry: verify hypothesis in ≤30 min via sub-agent before committing to fix path; absence of structural evidence ≠ absence of vector.
- `feedback_cross_session_commit_attribution_collision.md` — fed-in-c had concurrent advisor sessions sharing canonical `.git/`; `git add` swept other session's staged files into a commit. Mitigation: `git status` between add and commit. Pattern recurs whenever ≥2 sessions write canonical; fed-in-d may face the same if RLS or audit lanes are concurrent.
- `feedback_thin_wakeup_prompts_verify_live_state.md` — `ScheduleWakeup` prompts must be ≤2 lines (wake trigger + resume-stage hint), not embedded decision trees. Decision logic lives in advisor-orchestrator.md, read at fire time.
- `feedback_junior_finalize_skips_when_worker_pre_pushes.md` — Junior daemon skips finalize-merge when worker pre-pushes. fed-in-c followed the DQ #338 pre-push mandate (cohort-1 manual finalize-merge worked cleanly). fed-in-d continues until daemon structural fix lands; verify the PreToolUse hook (option-b ship in concurrent session 2026-05-22) before relaxing the brief mandate.
- `feedback_default_local_testing.md` / `project_laptop_canonical_cargo_runner.md` — Shape G is SUSPENDED until 2026-06-01 (DQ #229, `project_shape_g_suspended_2026_05_16.md`). All cargo runs laptop-shape (`kind: "validate-pending-laptop"`, advisor-laptop mutates). fed-in-d may cross 2026-06-01 — re-check DQ #229.
- `feedback_background_task_notification_lies.md` — read the explicit `E2E_EXIT_0`/`CHECK_EXIT_0`/`CHAIN_DONE` marker; never trust the harness completion notification.

**Planning-side:**
- **Crate-qualify Lemmy-1.0 schema/file paths** (established fed-in-a, held fed-in-b/c). All paths: `crates/apub/activities/src/governance/inbox.rs:473` (rate-map), `:698` (TOCTOU eviction), `crates/api/api/src/governance/publish_trust_attestation.rs:165` (raw-string key). No bare "schema.rs".
- §2.4 mandatory file-class lesson injection: any `crates/server/tests/e2e.rs` edit → `feedback_lemmy_error_no_std_error.md` (Case A — mirror the `v1_federation_inbound_b_fixtures` sibling module's `LemmyResult<()>` shape verbatim) + `feedback_async_pool_test_pattern.md`; ≥2 e2e edits → + `feedback_junior_worker_e2e_edit_hang.md`. Any handler with 2+ DB writes (eviction fix; transactional rate-counter) → `feedback_multi_write_handlers_need_transactions.md`.
- **Complexity score gate** — `feedback_complexity_score_pre_split.md`. Concurrency-design plans should NOT be 2/10 (fed-in-c was; appropriate). fed-in-d full (b) should score ≥5/10; narrow (b) subset ≥4/10. If planner returns <4/10 for any (b) work, surface as DQ for planner revision.

**Impl-side:**
- `feedback_fix_impl_pre_push_cargo_check.md` + `feedback_fix_impl_enumerate_all_callsites.md` — before any struct/signature change brief, `rg "<Symbol>" crates/ tests/` enumerate the FULL N callsites; brief lists all N; file-edit cap is the count of distinct files. fed-in-c had 0 fix-impl cycles; fed-in-d concurrency design may introduce some. Plan ahead.
- `feedback_lemmy_error_no_std_error.md` Case A discipline (`LemmyResult<()>` outer + bare `?`) is canonical for the `v1_federation_inbound_b_fixtures` module style — extend to `v1_federation_inbound_d_fixtures` if new e2e fixtures land.

**BM-side:**
- **bm-merge first-try L14 REVISED success** (Junior #410 ✓ on PR #144). Continue.
- **bm-pr deliverable contract** — Junior #404 missed phase-runlog write + findings.yaml shell. Junior #408 violated bucket-blank constraint. Two patterns across two BM verbs in one phase. fed-in-d should consider tightening the brief contract OR accepting the gitignored-state risk (and continue advisor post-condition recovery).

## 4. v1-federation-inbound-d-specific watchlist

Each cites a specific file/table/line + a forward gate:

1. **`crates/apub/activities/src/governance/inbox.rs:473`** — rate-limit maps. Plan §4 watchpoint NAMES the chosen mitigation (LRU vs hash-key vs Postgres-backed counter) with concrete struct/type. Gate: plan §13 task body has the literal struct/type change; e2e asserts memory growth bounded across N unique peers (e.g. assert map.len() < bound after N inserts).
2. **`crates/api/api/src/governance/publish_trust_attestation.rs:165`** — raw-string HashMap key (subject_url). Plan §4 watchpoint NAMES the chosen mitigation (sha256 hash truncation? fixed-size key? bounded LRU?). Gate: plan §13 task enumerates BOTH the producer site (rate-map insert) and the consumer site (rate-map lookup); §G4 callsite-enumeration discipline applies (`rg "subject_url|rate_map_key" crates/` before authoring).
3. **`crates/apub/activities/src/governance/inbox.rs:698`** — `evict_oldest_unreviewed_if_needed` TOCTOU. Plan §4 watchpoint names whether single-atomic-SQL (Diesel `.returning(...)`) or transactional-row-lock (`SELECT FOR UPDATE SKIP LOCKED` + DELETE). Gate: e2e asserts no over-eviction under concurrent receives + no spurious `storage_cap_evicted` log on 0-row DELETE.
4. **Migrations** — fed-in-d is concurrency-side + reader-side. The DoS hardening is code-only (in-memory data structures) UNLESS the chosen mitigation is Postgres-backed counters (then a new `federation_rate_counter` table; +1 migration). **If the generated plan proposes a new migration under `migrations/**` without an explicit decision-point in §4 calling out the Postgres-backed choice, catch-fire — silent migration introduction is a scope violation.**
5. **DQ #326 stale leftover** — conformance-audit-task-8 entry that landed via gov-v0→fed-in-c forward-merge; mutated to pass on audit lane. Do NOT cross-lane-edit this in fed-in-d's DQ. Let audit lane resolve via DQ #307. If audit lane closes before fed-in-d ships, sync the DQ entry via natural forward-merge.
6. **DQ #338 daemon-bug** — concurrent session shipped option-b PreToolUse hook 2026-05-22. Verify hook is live (`.claude/hooks/` ls + `git log governance-v0 --grep "DQ #338"`) BEFORE relaxing the impl-task brief pre-push mandate. Until verified live, keep the §4 Constraints pre-push mandate in every impl-task brief.
7. **Concurrent sessions** — fed-in-c had ≥2 concurrent advisor sessions writing canonical `.git/` (causing one staged-file attribution collision recovered at `c858aa7ab`). Per `feedback_cross_session_commit_attribution_collision.md`: `git status` between `git add` and `git commit` is mandatory when other sessions are known active. fed-in-d may face same if v1-rls-r2 or audit-lane next sub-phase is concurrent.

## 5. Operational rules

Carry forward from fed-in-c; adjust per the retro's change bullets:

- **Polling cadence:** ~10 min `mcp__junior-brehon__list_tasks` (status only); on transition `show_task` + `git fetch` + read DQ; triage new pending entries; queue next per stage shape.
- **Brief discipline:** briefs at `.claude/PRPs/briefs/v1-federation-inbound-d-<role>-<n>.md`, committed to `governance-v0` first (planning/bm-cut/bm-pr/bm-merge briefs) OR to `phase-v1-federation-inbound-d` first (impl-task briefs after bm-cut). Pre-queue `memory_search_hybrid` (§2.3) + `/precheck` + §2.4 mandatory file-class lesson injection.
- **Concurrency-design plan must include explicit decision-point in §4 watchpoints** — name LRU-vs-hash-vs-Postgres for rate-map; name atomic-SQL-vs-row-lock for TOCTOU eviction. Concept-only watchpoints fail gate (§3.5 watchpoint specificity).
- **LemmyResult Case A override** for any e2e brief — mirror `v1_federation_inbound_b_fixtures` sibling module verbatim per `feedback_lemmy_error_no_std_error.md`.
- **Shape G status:** check DQ #229 / current state. SUSPENDED until 2026-06-01 means validate-pending-laptop per `advisor-orchestrator.md` §5.2. If fed-in-d crosses 2026-06-01, re-check DQ #229 before assuming Shape G is back.
- **Windows e2e:** bat-wrapper invocation + explicit-exit-marker reads per `feedback_windows_e2e_requires_bat_wrapper.md`. Never bare `cargo test`; never `-p lemmy_server --features full`.
- **Serial-cohort discipline** if hardware-constrained (pre-Shape-G; ≥2 cargo runs × 6 GB peak > 10 GB EliteDesk cap → degrade to serial per §4.1 step 5).
- **Model tiering:** Planning → Opus, Impl → Sonnet, BM/ci-watcher → Haiku (per CLAUDE.md Four-role model).
- **Clarify gate** (§3.3) before queueing planning Junior; six user gates (plan approval / judgment-heavy DQ / CR triage / Phase 2 e2e local-vs-dispatch / merge confirm / retro sign-off).
- **DQ attribution:** `chore|docs(advisor|decision-queue):` only from this session; non-advisor sessions cannot write `answered_by: "advisor"` or `kind: "clarify"`.
- **Memory headroom:** no bulk e2e.rs reads (8945+ lines); use Grep + targeted Reads with offset/limit per `feedback_junior_worker_e2e_edit_hang.md`.

## 6. What changed from v1-federation-inbound-c's rule set

- **DQ schema v3** (shipped on gov-v0 via v1-dq-schema-r1) — new ids are 12-hex composite (`<session-hex>-<seq>`); pre-v3 entries keep their integer id + gain `id_v1` alias. fed-in-d's DQ writes use `bash scripts/brehon/dq-v3-new-entry.sh`. Cross-lane id collision class is structurally eliminated for new entries.
- **Multi-lane atomic protocol hard refusal #6** (revised 2026-05-16) — atomic read-mutate-commit for any DQ write on canonical, especially when other sessions are concurrent. `git status` between `git add` and `git commit` is now mandatory belt-and-braces per `feedback_cross_session_commit_attribution_collision.md`.
- **DQ #338 PRE-PUSH MANDATE** — every impl-task brief involving finalize-merge MUST carry pre-push mandate in §4. Concurrent session shipped option-b PreToolUse hook 2026-05-22; verify live before relaxing.
- **L14 runlog COMPLETE entry POST-merge** (REVISED 2026-05-18) — bm-merge briefs use the post-merge runlog ordering per `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md`. Continue (fed-in-c Junior #410 first-try success).
- **brehon-conformance-audit skill is live** (shipped via PR #141) — `/brehon-conformance-audit` available for federation `mod.rs` Tier-1 audits at advisor pre-bm-merge. Run in `phase-diff` mode if fed-in-d touches `crates/apub/activities/src/governance/**.rs` or `crates/api/api/src/governance/**.rs`.

## 7. Catch-fire procedures

Universal triggers from `.claude/rules/advisor-orchestrator.md` §5.5 + phase-specific additions:

- Junior subagent ignores hard refusals (impl-task writes `crates/**` from non-authorising brief; BM writes `crates/**` from any brief) → surface breach + agent path.
- `answered_by: "advisor"` in commit whose subject is NOT `^(chore|docs)\((advisor|decision-queue)\)` → surface SHA + subject.
- Subagent commits to `governance-v0` or `main` directly (impl/bm should commit to phase branch or worker branch) → surface SHA + branch.
- `bm-task` opens PR into `main` instead of `governance-v0` → surface PR #.
- Plan introduces new migration under `migrations/**` WITHOUT explicit Postgres-backed-counter decision-point in §4 → catch-fire (silent scope violation).
- §G4 cycle-count meta-rule: ≥3 fails with same `(error_class, file_basename)` for cohort member → HARD REFUSAL catch-fire regardless of allowlist match.
- Conformance-audit Tier-1 finding on `crates/apub/activities/src/governance/**.rs` OR `crates/api/api/src/governance/**.rs` OR `crates/db_schema/src/source/governance/**.rs` → HARD REFUSAL catch-fire (NOT auto-fix; human-in-the-loop decides per `feedback_mirror_phase6_convention_in_same_file.md`).

## 8. Archive after v1-federation-inbound-d

The standard close: run `/brehon-phase-transition v1-federation-inbound-d v1-federation-inbound-e` (or next agreed slice — fed-in-e candidates per fed-in-c retro: admin REST endpoints + pseudonym rendering + step-up auth; OR if (b) was option-a full ship, the residual fed-in-* gap from retro option-c list). This skill will: close `workflow_state_v1_federation_inbound_d.md`, delete `workflow_state_v1_federation_inbound_c.md` (two-ago), create the next skeleton, write the next bootstrap file, update brehon-fork MEMORY.md, commit on governance-v0. The prior bootstrap file (this one) stays in `.claude/PRPs/handovers/` as its own archive — git history is the archive; no move.

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `a85b5bf2f` (captured 2026-05-22) — `chore(hooks): track allow-prp-deliverables.sh`
- Phase branch HEAD: not yet created (branch `phase-v1-federation-inbound-d` cut at bm-cut)
- Recent governance-v0 commits (`git -C C:/Users/barri/Developer/brehon-fork log --oneline -5 governance-v0`):

  ```
  a85b5bf2f chore(hooks): track allow-prp-deliverables.sh
  abfe5ac44 docs(retro): session-retro 2026-05-22 — carry-forward shipped
  180a0f9aa chore(advisor): promote 3 carry-forwards from parallel-subagent-dispatch retro
  45ef46a7f chore(bm): merge PR #144 complete — runlog
  a1b62280f feat(retro-followups-r1): four-role retro (task 4)
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff — verify on session start with scripts/brehon/resolve-dq-canonical.sh once a phase branch exists; pre-bm-cut, read .claude/decision-queue.json directly)
Note: DQ #326 (conformance-audit-task-8 validate-pending-laptop, result:pass-but-pending) may still be visible in fed-in-c's DQ tail until the audit lane's next sub-phase resolves DQ #307. Do NOT cross-lane-edit it from fed-in-d.
Note: DQ #338 (daemon-bug blocker) — concurrent session shipped option-b PreToolUse hook 2026-05-22; verify .claude/hooks/ + governance-v0 commit log before relaxing impl-task pre-push mandate.
Note: DQ schema is v3 (composite ids per session). New entries: bash scripts/brehon/dq-v3-new-entry.sh.
```

## Stop-and-ask tripwires

- **Stop and ask if:** the planning Junior's generated plan proposes a new migration under `migrations/**` WITHOUT a §4 watchpoint naming the Postgres-backed-rate-counter decision-point — silent migration introduction is a scope violation; fed-in-d is concurrency-side + reader-side, schema substrate is from fed-in-a/b.
- **Stop and ask if:** the planning Junior's plan §4 watchpoint for the rate-map mitigation is concept-only ("use a bounded data structure") without naming the concrete choice (LRU vs hash-key vs Postgres) — watchpoint-specificity gate fails per `feedback_advisor_watchpoint_specificity.md`.
- **Stop and ask if:** an impl-task brief targeting `crates/apub/activities/src/governance/**.rs` is queued WITHOUT running `/brehon-conformance-audit target_scope=file <brief-named-file>` first — §3.1.1 conformance-audit prevention checkpoint applies; Tier-1 findings fold into brief §3 / §4 before clarify-DQ entries.
- **Stop and ask if:** the §G4 cycle-count meta-rule fires (≥3 fails with same `(error_class, file_basename)` for cohort member) — HARD REFUSAL catch-fire regardless of allowlist match per `feedback_plan_stub_uniformity_with_canonical_sibling.md`. Do NOT auto-queue another fix-impl; surface to user for re-plan.
- **Stop and ask if:** DQ #338 PreToolUse hook is NOT verifiable live in `.claude/hooks/` at session start AND impl-task pre-push mandate is being relaxed — keep the mandate until structural fix is visibly present, per `feedback_falsifiable_hypothesis_before_structural_fix.md`.
