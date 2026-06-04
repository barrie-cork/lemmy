# m1-b retro — governance messaging-config + bridge-notify wiring (M1 Tree B)

**Sub-phase:** m1-b (M1 Tree B — Tasks 1–7: `governance_messaging_config` table + admin config API + identity-policy validator + fire-and-forget bridge notify + e2e tests)
**Branch:** `phase-m1-b`
**Base:** `governance-v0` @ `9c7dac9c4` (trunk at init)
**Plan:** `.claude/PRPs/plans/m1.plan.md` (Tree B subset; M1 split into m1-b-first then m1-a per gate-1 user decision 07:00Z)
**Dates:** 2026-06-04 (single-day run: ~08:14Z init → ~16:39Z merge)
**Impl model:** four-role Junior orchestration via `/auto-phase M1` (planning pre-existing → impl-task ×7 → validate-pending-laptop ×7 → bm-pr → CR → fix-impl-1 → bm-merge), pre-Shape-G (all cargo/e2e on laptop)
**MiniMax A/B trial:** fired this phase (user override below 5/5 gate) — Tasks 3/4/5 dual-armed (Sonnet control + MiniMax-M2.7 trial); only Sonnet merged
**PR:** #177 (`phase-m1-b` → `governance-v0`) merged 2026-06-04 @ `d6d027794` (`--merge --admin`, advisory red-flag bypass)

---

## TL;DR for the advisor

**Plan delivered; the run was clean end-to-end but hit TWO §G4 catch-fires (both genuine, both 1-line fixes, both user-authorized advisor-direct edits) and surfaced FOUR new process/infra retro items.** All 3 in-scope Tree-B §16a stories shipped (stories 1/4/6 are Tree A, correctly deferred to m1-a). The MiniMax A/B trial reached 3 of 5 cumulative eligible tasks with a load-bearing finding (T4 divergence was brief-ambiguity-driven, not capability — T5 with a tighter brief converged byte-for-byte). CR was light (3 findings: 1 fix-in-pr, 2 rebut). Merge needed `--admin` to bypass the advisory adr-compliance red-flag (new-governance-route flag on plan-sanctioned M1 routes).

**Two §G4 catch-fires, both non-allowlist (correctly), both resolved cleanly:**
- **Task 6** — `E0599 .json() not found on reqwest_middleware::RequestBuilder`. The worker's code was idiomatic; the workspace just hadn't enabled the `json` feature on `reqwest-middleware`. 1-line Cargo.toml fix.
- **Task 7** — e2e test asserted `is_none()` for clean posture, but the migration *seeds* a `messaging_enabled=false` row, so `read_current` returns `Some(row)`. 1-assertion fix (test-expectation bug, not production).

Both were correctly classified non-allowlist → catch-fire (not auto-fixed), root-caused against the actual code/migration before surfacing, and the user authorized advisor-direct edits.

---

## 1. What worked — keep doing

### 1.1 Exit-marker verification caught two false "exit 0" harness notifications
Cmd-0 cycle-1 (Task 6) and the Task-7 e2e run BOTH had the background-task notification report "exit code 0" while the authoritative exit-marker said `CHECK_EXIT_NONZERO` / `E2E_RUN_EXIT_NONZERO`. Verifying the marker (not the notification) per `feedback_background_task_notification_lies.md` is what caught the real failures. **Keep:** never trust the harness completion notification's exit code; grep the exit-marker + `^error` count every time.

### 1.2 Compile-check-the-CR-claim before triaging
CR F2 (read_current tie-breaker) was verified against the schema (`id` Int4 column exists) + the Diesel idiom before classifying it `fix-in-pr`. CR F1 (is_admin) was checked against the plan (Task 4 mandated `is_admin` mirroring `admin_config.rs`) before rebutting. CR F3 (e2e assertion) was checked against the migration (`up.sql` seeds the row) before rebutting. Per `feedback_verify_automated_reviewer_claims_against_compiler.md`. **Keep:** every CR finding is a hypothesis; verify against code/plan/migration before bucketing.

### 1.3 §G4 non-allowlist discipline held under pressure
Both Task-6 and Task-7 failures were genuinely outside the allowlist (E0599-feature-gate; e2e test-logic). The classifier correctly catch-fired both rather than improvising an auto-fix, and each surfaced to the user WITH a root-caused, evidence-backed single-line recommendation. **Keep:** catch-fire + validated recommendation beats silent auto-fix on non-allowlist failures.

### 1.4 Scope-aware /brehon-verify (Tree B vs Tree A)
`/brehon-verify` correctly verified only the 3 in-scope Tree-B stories (2/3/5) and explicitly noted stories 1/4/6 as Tree-A out-of-scope, avoiding a false phantom. **Keep:** when a PR delivers a subset of a multi-tree plan, the verify must scope to the delivered tree's stories.

### 1.5 MiniMax A/B parallel-pair with comparison-only merge policy
The dual-arm dispatch (Sonnet + MiniMax off the same throwaway base, only Sonnet merging) produced clean comparison data without risking the canonical branch. **Keep:** the FF-promotion-of-control-arm-only mechanism.

---

## 2. Per-role signals

### 2.1 Advisor
**Worked:** brief series crisp; Mode-B trunk→phase single-file sync clean every cohort; exit-marker discipline; two §G4 root-causes (E0599 feature-gate, migration-seed-vs-absent-row) were verified against actual artifacts before surfacing, not guessed.
**Gap (retro item #4):** the bm-poll-cr findings YAML is gitignored → reaped with the worker worktree before the advisor could read it. The advisor re-derived the triage directly from the PR (which is the source of truth anyway). **Action:** treat the CR findings on the PR as canonical; do NOT depend on the ephemeral `pr-<N>-findings.yaml` surviving. Consider: bm-poll-cr should ALSO write a short findings summary to the (committed) runlog, or the advisor should always read CR findings from `gh pr view` directly.

### 2.2 Planning
**Worked:** the plan's MIRROR refs (admin_set_config_* sibling at e2e.rs:6235, the `Unknown(format!)` validator idiom, read_current accessor) were precise enough that the impl-task briefs could give exact anchors. The §16a Tree-A/Tree-B split was clear.
**Gap:** none specific to this run (plan pre-existed; the split decision was a gate-1 user call, not a planning defect).

### 2.3 Impl (Sonnet)
**Worked:** every task's code was idiomatic and matched the brief; the two failures were NOT bad code (T6 was a missing workspace feature flag; T7 was a wrong test expectation that the brief itself under-specified).
**Gap (retro item #3):** the Task-6 + Task-7 workers BOTH ran cargo on the daemon AFTER committing (cohort-6 hung in it; #582/#583 ran it redundantly post-commit). Single-arm impl-task briefs don't mandate the pre-push cargo-check that fix-impl briefs do, AND the NO-CARGO-ON-ELITEDESK rule was violated by the worker post-commit. **Action:** (a) extend the pre-push cargo-check §4 constraint to impl-task briefs that add NEW HTTP-client calls or NEW public API (would have caught T6's E0599 on the worker, not the laptop); (b) reinforce in the impl-task brief template that the worker must NOT run cargo on the daemon — write-DQ-and-stop is the contract.

### 2.4 BM (Haiku)
**Worked:** bm-pr opened PR #177 correctly (base gov-v0, not draft); bm-merge did the L14 runlog-AFTER-merge correctly (no belt-and-braces re-apply needed); L16 branch deletion clean; `--admin` bypass used only for the authorized advisory flag.
**Gap:** none — BM verbs executed to spec this phase.

---

## 3. New retro items (4) — promote candidates

1. **MiniMax dispatch findings A–D** (from `issue_note_minimax_dispatch_wrong_daemon_db.md`): A — `queue-minimax-task.sh` must `cd /srv/brehon-fork` first (FIXED, committed `10201285c`); B — always verify dispatch landed in the brehon DB specifically (`sqlite3 /srv/brehon-fork/.junior/junior.db`), never trust CLI echo; C — daemon-local gov-v0 can be stale in multi-lane; D — `cmd //c` redirect-path trap (backslash path + `cd /d` inside cmd). A+C+D folded into this retro per the note's removal condition.

2. **Worker post-task-retro `memory_write_eval` hang** (2× this phase, cohort 6 #580/#581): both workers hung in the post-task-retro MCP `memory_write_eval` tail AFTER committing their code. Benign (no index.lock, clean worktrees, code committed). Recovery: tar-preserve + push committed SHAs + cancel to free slots. **Promote-if-3rd:** if a third worker hangs in post-task-retro, investigate the MCP `memory_write_eval` call path on the daemon (timeout? HTTP PMD daemon reachability from worker subprocess?).

3. **Worker runs redundant cargo on daemon post-commit** (Task 6 + Task 7): see §2.3 gap. NO-CARGO-ON-ELITEDESK violated by the impl-task worker after it committed + raised the validate-pending DQ. Not harmful (single worker, no lock cascade), but it's wasted daemon CPU and a contract drift. **Action in §2.3.**

4. **Gitignored findings YAML reaped before advisor read** (CR triage): see §2.1 gap. **Action:** advisor reads CR findings from `gh pr view` directly; don't depend on `pr-<N>-findings.yaml` surviving worker reaping.

---

## 4. MiniMax A/B trial — running tally (3 of 5 cumulative, cross-phase)

| Task | Sonnet | MiniMax | Winner | Note |
|---|---|---|---|---|
| T3 (DTOs) | byte-identical struct defs | byte-identical (doc-comments differ) | TIE | wall-clock MiniMax-faster (1m39s vs 2m22s) |
| T4 (admin handler) | correct (bare ConfigValueWithProvenance GET DTO, .ok_or_else()?) | WRONG GET DTO + .expect() panics + 534-line gratuitous DQ rewrite + unbriefed scope field | **SONNET decisive** | T4 brief was under-specified on the GET DTO shape |
| T5 (validator + routes) | correct | byte-identical validator + same pub-struct fold-fix + identical routes | TIE | **KEY FINDING:** T5 brief pre-resolved the ambiguity (per T4 lesson) → MiniMax matched Sonnet exactly |

**Conclusion (n=3):** 1 decisive Sonnet win + 2 ties. **T4's divergence was BRIEF-AMBIGUITY-driven, not capability** — when the T5 brief was tightened, MiniMax converged byte-for-byte. MiniMax is viable for fully-specified MIRROR-ref tasks but is more sensitive to under-specified briefs than Sonnet. **Trial pauses at 3/5; resumes next MIRROR-heavy phase for the remaining 2 eligible tasks. Decision deferred to n=5.** Full data: `.claude/PRPs/reports/minimax-m27-trial-results.md`.

---

## 5. Per-task complexity (files / commits / runtime-min / max-log-silence-min)

| Task | files | commits | runtime (min) | §G4 cycles | notes |
|---|---|---|---|---|---|
| T1 migration | 2 | 1 | ~7 | 0 | migrate-roundtrip clean |
| T2 model+newtype | ~3 | 1 | ~6 | 0 | InsertForm omits AsChangeset (append-only) |
| T3 DTOs [AB] | 1 | 1 | ~2 | 0 | n=1 tie |
| T4 handler [AB] | ~2 | 2 | ~19 | 0 | n=2 Sonnet win |
| T5 validator+routes [AB] | 1 | 1 | ~1.5 | 0 | n=3 tie (warm target) |
| T6 bridge_notify | 4 | 2 | ~7 + §G4 | **1** | E0599 reqwest-middleware json feature |
| T7 e2e | 1 | 2 | ~13 + §G4 | **1** | test-expectation bug (is_none vs Some(false)) |
| fix-impl-1 (CR F2) | 1 | 1 | ~2 | 0 | read_current tie-breaker |

**Aggregate:** 8 impl tasks + 1 fix-impl; 2 §G4 catch-fires (both 1-line, both clean on cycle 2); 0 cycle-count-meta-rule triggers (no tuple hit ≥3); ~8.5h wall-clock single-day. Max log-silence: the cohort-6 worker hang (~40 min frozen `updated_at`) — benign, recovered.

---

## 6. End-of-phase actions

- **ROTATE MiniMax API key** — precondition MET (T3/T4/T5 all validated `pass`). The key was exposed in the advisor transcript 2026-06-04 via `list_tasks` (daemon job-row `envOverrides` returned plaintext). User chose rotate-after-trial at gate. **DO THIS NOW** (post-retro, pre-phase-transition or immediately after): rotate at the MiniMax console + update `.env`. Tracked in `pending_end_of_phase_actions` + `project_minimax_key_rotate_after_m1b_trial.md`.
- **Remove `issue_note_minimax_dispatch_wrong_daemon_db.md`** — Findings A+C+D are now folded into §3 item 1 of this retro (the note's removal condition is met).
- **m1-a (Tree A bridge crate, Tasks 8–13)** is the next sub-phase — the `services/bridge/` Matrix bridge, workspace-excluded. Stories 1/4/6 verify there.
