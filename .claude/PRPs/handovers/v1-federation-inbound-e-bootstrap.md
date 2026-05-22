---
phase: v1-federation-inbound-e
plan: .claude/PRPs/plans/v1-federation-inbound-e.plan.md   # not yet authored
phase_branch: phase-v1-federation-inbound-e                  # not yet created — cut at bm-cut
worktree: C:/Users/barri/Developer/brehon-fork-fed-in-e     # created at bm-cut; until then canonical brehon-fork
authored: 2026-05-22
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the v1-federation-inbound-e advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon v1-federation-inbound-e.** This is a fresh session (or a resumed one). The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-fed-in-e` once `bm-cut` creates the lane worktree (per `.claude/rules/multi-lane-worktree.md`).

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane. Surface-first ritual: if ≥2 worktrees active or DQ-pending hook emits WARN, print a one-line lane status before anything else.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `2fcda11b7` or ahead (see §"Git state at handoff"). If drifted, run `git -C C:/Users/barri/Developer/brehon-fork log --oneline 2fcda11b7..governance-v0` and update your mental model before acting.
3. Read `.claude/decision-queue.json` for any pending entries since handoff; compare against §"Decision-queue snapshot" below. At handoff: empty.
4. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_v1_federation_inbound_e.md` is the running-state scratchpad. Read `workflow_state_v1_federation_inbound_d.md` ONCE for carry-forward (daemon stale-ref pattern, Task 2 deferral rationale, governance-v0 divergence check).
5. Lane cleanup (if not done): `cd C:/Users/barri/Developer/brehon-fork && git worktree remove ../brehon-fork-fed-in-d` (PR #146 already merged; branch deleted on origin). Then `git branch -d phase-v1-federation-inbound-d` if local tracking branch remains.

## Next concrete action

No plan exists yet. **Author `.claude/PRPs/briefs/v1-federation-inbound-e-planning-1.md`** scoped to Task 2 from the fed-in-d plan (per-peer bound + SHA-256 nonce) with an explicit concurrency-model section addressing the TOCTOU gap. Commit to `governance-v0`. Run `/brehon-clarify .claude/PRPs/briefs/v1-federation-inbound-e-planning-1.md` → resolve every clarify-DQ → queue the `[role:planning]` Junior task.

---

## 1. v1-federation-inbound-e in one paragraph

fed-in-e is the **continuation of the federation-inbound DoS-hardening track**, picking up Task 2 that fed-in-d explicitly deferred. fed-in-d (PR #146, `897d3f72d`) shipped the per-actor rate-map bound (`check_per_actor_rate_limit` + `OnceLock<Mutex<HashMap>>` + cap+eviction) in `publish_trust_attestation.rs`. Task 2 adds: **(a) per-peer bound** — an inbox-level cap keyed on the remote peer's instance domain (prevents one hostile instance from flooding the rate-map past `MAX_PER_ACTOR_RATE_ENTRIES`) — and **(b) SHA-256 nonce** — replacing the raw `subject_url` string key in the rate-map with a fixed-width hash (prevents cheap memory inflation via long actor URLs). The TOCTOU concern that triggered the deferral (the `contains_key` → `remove` race under concurrent tokio tasks holding the `Mutex` across await points) must be addressed in the plan's concurrency model section before the planning Junior is queued. DoD: per-peer cap enforced, nonce stored, existing 22 lib tests still passing, e2e for the new cap asserted, 0 new migrations unless Postgres-backed approach is chosen (watch: triggers scope review gate).

## 2. Why v1-federation-inbound-e is harder/easier than v1-federation-inbound-d

- **Harder — TOCTOU design gap must be resolved upfront.** fed-in-d's Task 2 deferral was explicitly because the `contains_key` → `remove` race under concurrent tokio tasks was not closed in the plan. fed-in-e's planning brief must include a concrete concurrency model choice: (a) hold the `Mutex` guard through the entire check-and-remove (avoids TOCTOU, may block concurrent tasks) OR (b) switch to `DashMap` (per-key locking, lower contention) OR (c) move rate-map to Postgres (eliminates in-process race, adds DB write per inbound). The plan's §4 watchpoint MUST name the chosen approach before advisor gate-1 approval.
- **Harder — SHA-256 nonce adds a crypto dep.** If `sha2` (already in the workspace from `lemmy_db_schema`) is re-used via `crates/apub/activities/Cargo.toml`, that's a one-line dep add. If a new crate is introduced, it's a Cargo.lock change + planner must verify workspace-level dep constraints. Plan must name which approach.
- **Easier — Task 1 foundation is in place.** The `OnceLock<Mutex<HashMap<String, (u32, Instant)>>>` singleton, the `MAX_PER_ACTOR_RATE_ENTRIES` constant, and the `check_per_actor_rate_limit` function exist and are tested. Task 2 extends the same file (`publish_trust_attestation.rs`) and possibly `inbox.rs`.
- **Easier — e2e baseline is 103/0/5.** The Phase-2 e2e baseline held through fed-in-d. Any regression from Task 2 will be visible immediately.
- **Same — daemon stale-ref class is still unresolved.** The structural fix (daemon `git fetch origin <branch>:<branch>` before `git worktree add`) has not shipped. PRE-PUSH MANDATE must remain in every brief until it does.

## 3. Lessons from v1-federation-inbound-d that apply to v1-federation-inbound-e

Reference by filename — do not duplicate content.

**Advisor-side:**

- `feedback_daemon_local_trunk_stale_multi_lane.md` — daemon stale-ref fired on BOTH Junior #415 (fix-in-pr) and #416 (bm-merge) in fed-in-d. The BM-verb variant (worker reads brief from `governance-v0`, daemon-local `governance-v0` lags origin, brief invisible) is not yet documented as a standalone lesson. Add `feedback_daemon_stale_bm_verb_brief_miss.md` if pattern recurs in fed-in-e. Until structural fix lands: pre-push ALL briefs AND run `ssh homeserver "cd /srv/brehon-fork && git fetch origin && git log --oneline origin/<branch> | head -3"` before each BM-verb dispatch to verify daemon-local ref is current.
- `feedback_cross_session_commit_attribution_collision.md` — `git status` between `git add` and `git commit` whenever another session is concurrent on the canonical `.git/`. fed-in-e may be concurrent with ship-2 or RT-r2.
- `feedback_bm_false_success_advisor_post_condition_catch.md` — 6× confirmed pattern. After every BM Junior "done", verify the real-world effect (PR state, branch tip, file existence, runlog append). Never trust the self-report.
- **Governance-v0 divergence check (new from fed-in-d — 1st occurrence, watch for 2nd):** before gate 5 (merge confirm), run `git log --oneline origin/governance-v0 ^phase-v1-<phase>` — if non-empty, review for reformatting/structural commits. Large reformats (like v1-quality-r1's `2f13ffb80`) will mark the PR CONFLICTING and require a merge-forward step on the phase branch before `gh pr merge`. Add this to your pre-gate-5 mental checklist.
- `feedback_thin_wakeup_prompts_verify_live_state.md` — `ScheduleWakeup` prompts ≤2 lines. Decision logic lives in advisor-orchestrator.md, read at fire time.
- `feedback_background_task_notification_lies.md` — read the explicit `E2E_EXIT_0`/`CHECK_EXIT_0` marker; never trust the harness completion notification.

**Planning-side:**

- **Concurrency model is mandatory in plan §4.** The TOCTOU gap was the specific reason Task 2 was deferred. The planning brief must require an explicit choice (hold-guard / DashMap / Postgres) with a concrete rationale. Concept-only watchpoint ("ensure thread-safety") fails the watchpoint-specificity gate per `feedback_advisor_watchpoint_specificity.md`.
- §2.4 mandatory file-class lesson injection: any `crates/server/tests/e2e.rs` edit → `feedback_lemmy_error_no_std_error.md` (Case A — mirror `v1_federation_inbound_b_fixtures` sibling `LemmyResult<()>` shape) + `feedback_async_pool_test_pattern.md`; ≥2 e2e edits → + `feedback_junior_worker_e2e_edit_hang.md`. Any handler with 2+ DB writes → `feedback_multi_write_handlers_need_transactions.md`.
- `feedback_complexity_score_pre_split.md` — Task 2 (per-peer bound + SHA-256 nonce + TOCTOU fix) should score ≥4/10. If planner returns <4/10, surface as DQ for revision.

**Impl-side:**

- `feedback_fix_impl_pre_push_cargo_check.md` — pre-push `cargo-check.sh` mandatory in brief §4. Structural change (nonce type switch, DashMap introduction) may cascade callsites — enumerate ALL with `rg` before authoring brief.
- `feedback_fix_impl_enumerate_all_callsites.md` — if Task 2 changes the rate-map key type (String → [u8; 32] or similar), enumerate ALL write sites and read sites in `publish_trust_attestation.rs` + any callers in `inbox.rs`. Brief lists all N; file-edit cap is distinct files.
- `feedback_lemmy_error_no_std_error.md` Case A — `LemmyResult<()>` outer + bare `?` is canonical for any new e2e fixtures module.

**BM-side:**

- PRE-PUSH MANDATE must be in every BM-verb brief until daemon structural fix is confirmed live. The fed-in-d pattern: bm-merge brief on `governance-v0` was invisible to the worker because daemon-local `governance-v0` lagged origin.
- Post-condition verification after every BM Junior "done" — check PR state, runlog append, branch tip. See `feedback_bm_false_success_advisor_post_condition_catch.md`.

## 4. v1-federation-inbound-e-specific watchlist

Each cites a specific file/table/line + a forward gate:

1. **`crates/apub/activities/src/governance/publish_trust_attestation.rs` — rate-map key.** Currently `String` (raw actor URL). Plan §4 watchpoint NAMES the replacement type (e.g. `[u8; 32]` SHA-256 truncated? `String` of hex? fixed-length `sha2::Sha256` digest?) AND names the `sha2` dep path (`crates/apub/activities/Cargo.toml` vs re-export from `lemmy_db_schema`). Gate: impl task body enumerates every `HashMap` write site (insert) and read site (get/contains_key) in the file; §G4 callsite-enumeration discipline applies.
2. **`crates/apub/activities/src/governance/publish_trust_attestation.rs` — TOCTOU concurrency model.** Plan §4 watchpoint NAMES the chosen model: (a) hold `MutexGuard` through check-and-insert (no await in between — verify with `tokio::task::block_in_place` or restructure so no `.await` between lock and drop), (b) `DashMap` (per-key locking), or (c) Postgres-backed. Gate: if (a), plan §13 task includes a `tokio::test` concurrent-insert probe; if (b), plan names the `dashmap` dep version; if (c), plan includes a migration.
3. **`crates/apub/activities/src/governance/inbox.rs` (optional, scope-dependent).** If per-peer cap is implemented here (inbox-level) rather than in `publish_trust_attestation.rs`, plan §4 must call this out explicitly. Gate: if `inbox.rs` is touched, conformance-audit `target_scope=file crates/apub/activities/src/governance/inbox.rs` runs BEFORE brief is committed (§3.1.1 prevention checkpoint).
4. **Migrations gate.** Task 2 is code-only UNLESS Postgres-backed rate counter is chosen. **Stop and ask if** the plan proposes a new migration under `migrations/**` without an explicit §4 decision-point naming the Postgres-backed choice — silent migration introduction is a scope violation.
5. **`crates/apub/activities/Cargo.toml` — sha2 dep.** If SHA-256 nonce requires adding `sha2` to this crate's Cargo.toml (not re-exported from workspace), plan must note it. Gate: plan §13 task body includes the `Cargo.toml` edit; advisor verifies no workspace-level version conflict before queueing.
6. **Daemon stale-ref pre-dispatch check.** Before every Junior task dispatch (impl AND BM-verb), verify daemon-local ref is current via `ssh homeserver "cd /srv/brehon-fork && git fetch origin && git log --oneline origin/<branch> | head -3"`. If daemon-local lags origin by the brief commit, either wait for daemon pull OR budget advisor inline execution as the fallback.

## 5. Operational rules

Carry forward from fed-in-d; adjust per the retro's change bullets:

- **Polling cadence:** ~10 min `mcp__junior-brehon__list_tasks`; on transition `show_task` + `git fetch` + read DQ; triage new pending; queue next per stage shape.
- **Brief discipline:** planning/bm-cut/bm-pr/bm-merge briefs committed to `governance-v0`; impl-task briefs committed to `phase-v1-federation-inbound-e` after bm-cut. Pre-queue `memory_search_hybrid` (§2.3) + `/precheck` + §2.4 mandatory file-class lesson injection.
- **Concurrency-model section is mandatory in planning brief** — names TOCTOU fix approach before brief is committed. Clarify gate runs before planning Junior is queued.
- **PRE-PUSH MANDATE in every brief** (impl AND BM-verb) until daemon structural fix confirmed live — `ssh homeserver "git fetch origin && git log --oneline origin/<branch> | head -3"` before each dispatch.
- **Governance-v0 divergence check before gate 5** — `git log --oneline origin/governance-v0 ^phase-v1-federation-inbound-e` — if non-empty and contains reformats, author merge-forward commit on phase branch first.
- **LemmyResult Case A override** for any e2e brief — mirror `v1_federation_inbound_b_fixtures` sibling module verbatim.
- **Shape G status:** SUSPENDED until 2026-06-01 (DQ #229). All cargo runs via validate-pending-laptop (`advisor-orchestrator.md` §5.2). Re-check DQ #229 if fed-in-e crosses 2026-06-01.
- **Windows e2e:** bat-wrapper invocation + explicit-exit-marker reads. Never bare `cargo test`; never `-p lemmy_server --features full`.
- **Serial-cohort discipline** if hardware-constrained (pre-Shape-G; ≥2 cargo × 6 GB > 10 GB cap → degrade to serial).
- **Model tiering:** Planning → Opus, Impl → Sonnet, BM/ci-watcher → Haiku.
- **Clarify gate** before planning Junior; six user gates (plan approval / judgment-heavy DQ / CR triage / Phase-2 e2e local-vs-dispatch / merge confirm / retro sign-off).
- **DQ attribution:** `chore|docs(advisor|decision-queue):` only from this session.
- **Memory headroom:** no bulk e2e.rs reads; use Grep + targeted Reads with offset/limit.

## 6. What changed from v1-federation-inbound-d's rule set

- **Governance-v0 divergence check added to pre-gate-5 checklist.** New from fed-in-d (1st occurrence of rustfmt-reformat CONFLICTING class). Not yet in `advisor-orchestrator.md` — carry as a manual habit until it is.
- **PRE-PUSH MANDATE extended to BM-verb briefs.** fed-in-d confirmed the daemon stale-ref fires on bm-merge briefs as well as impl-task briefs. The mandate (originally for impl-task finalize-merge) now covers every Junior task dispatch.
- **Daemon pre-dispatch ref check (new process step).** Before every Junior task creation (not just impl-task), verify daemon-local ref is current via ssh `git log --oneline origin/<branch> | head -3`. If it lags by the brief commit, either wait or budget advisor inline.
- **`feedback_daemon_stale_bm_verb_brief_miss.md` watch item.** If the BM-verb stale-ref pattern recurs in fed-in-e (3rd occurrence in the BM-verb class), promote to a named lesson file.

## 7. Catch-fire procedures

Universal triggers from `.claude/rules/advisor-orchestrator.md` §5.5 + phase-specific additions:

- Junior subagent writes `crates/**` / `migrations/**` / `tests/**` from non-authorising brief → surface breach + agent path.
- `answered_by: "advisor"` in commit whose subject is NOT `^(chore|docs)\((advisor|decision-queue)\)` → surface SHA + subject.
- Subagent commits to `governance-v0` or `main` directly → surface SHA + branch.
- `bm-task` opens PR into `main` → surface PR #.
- Plan introduces new migration under `migrations/**` WITHOUT explicit Postgres-backed-counter decision-point in §4 → catch-fire (silent scope violation; Task 2 is code-only unless Postgres path chosen).
- §G4 cycle-count meta-rule: ≥3 fails with same `(error_class, file_basename)` for cohort member → HARD REFUSAL catch-fire.
- Conformance-audit Tier-1 finding on `crates/apub/activities/src/governance/**.rs` or `crates/api/api/src/governance/**.rs` → HARD REFUSAL (NOT auto-fix; human-in-the-loop).
- Concurrency model watchpoint is concept-only in plan §4 ("ensure thread-safety" without naming the chosen approach) → catch-fire (watchpoint-specificity gate failure; task-2 TOCTOU is load-bearing).

## 8. Archive after v1-federation-inbound-e

Run `/brehon-phase-transition v1-federation-inbound-e <next>`. This skill will: close `workflow_state_v1_federation_inbound_e.md`, delete `workflow_state_v1_federation_inbound_d.md` (two-ago), create the next skeleton, write the next bootstrap file, update brehon-fork MEMORY.md, commit on governance-v0. The prior bootstrap file (this one) stays in `.claude/PRPs/handovers/` as its own archive — git history is the archive; no move.

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `2fcda11b7` (captured 2026-05-22) — `docs(retro): flip promotion candidate checkboxes to shipped (e2f44894b)`
- Phase branch HEAD: `phase-v1-federation-inbound-e` not yet created (branch cut at bm-cut)
- Recent governance-v0 commits (`git log --oneline -5 governance-v0`):

  ```
  2fcda11b7 docs(retro): flip promotion candidate checkboxes to shipped (e2f44894b)
  ff65533ee docs(retro): v1-federation-inbound-d retro — PR #146 merged 897d3f72d
  e2f44894b feat(skills+lessons): ship top-2 retro proposals from memory-prune-gate-ship
  8b08e8989 docs(briefs): v1-ship-2 planning brief — per-endpoint e2e backfill
  ebfcf7aaf chore(bm): merge PR #146 complete — 897d3f72d — federation-inbound-d shipped
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff)
Note: DQ #229 (Shape G re-enable) still pending — SUSPENDED until 2026-06-01. Validate-pending-laptop active for all cargo runs.
Note: DQ schema is v3 (composite ids per session). New entries: bash scripts/brehon/dq-v3-new-entry.sh.
```

## Stop-and-ask tripwires

- **Stop and ask if:** the planning Junior's generated plan §4 watchpoint for the TOCTOU concurrency model is concept-only ("ensure thread-safety", "use appropriate locking") without naming the concrete choice (hold-guard / DashMap / Postgres). This was the specific reason Task 2 was deferred from fed-in-d — the plan must resolve it before impl begins.
- **Stop and ask if:** the plan proposes a new migration under `migrations/**` without an explicit §4 decision-point naming the Postgres-backed-rate-counter choice. Task 2 is code-only unless that path is chosen; silent migration introduction is a scope violation.
- **Stop and ask if:** a `sha2` dep is introduced in `crates/apub/activities/Cargo.toml` at a version that conflicts with the workspace-level `sha2` used in `lemmy_db_schema` — workspace-level dep version uniformity is mandatory; check `grep -r "sha2" Cargo.toml crates/` before approving.
- **Stop and ask if:** a Junior BM-verb task (bm-merge, bm-pr) declares completion but the real-world effect (PR state, branch tip, runlog entry) is absent — apply `feedback_bm_false_success_advisor_post_condition_catch.md` and verify explicitly before advancing pipeline state.
- **Stop and ask if:** the daemon-local ref check (`ssh homeserver "cd /srv/brehon-fork && git log --oneline origin/<branch> | head -3"`) shows the daemon is behind the brief commit at task-dispatch time — do NOT dispatch; either wait for daemon pull or execute inline as in fed-in-d.
