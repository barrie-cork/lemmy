# v1-federation-inbound-d retro

**Sub-phase**: v1-federation-inbound-d — per-actor rate-map bound on inbound federation trust attestations (DoS hardening)
**Branch**: `phase-v1-federation-inbound-d` (merged into `governance-v0` as `897d3f72d`, 2026-05-22T18:58:42Z)
**PR**: #146
**Wall-clock**: ~2 days elapsed (planning + bm-cut at `26c6badf2`; lane worktree `brehon-fork-fed-in-d` bootstrapped; Task 1 impl + fix-in-pr + merge). Active orchestration was a small fraction — most elapsed time was async waiting between stages.
**Outcome**: shipped (partial — Task 1 + fix-in-pr; Task 2 deferred). E2e: 103 passed / 0 failed / 5 ignored. Lib tests: 22/22. CR findings: 1 major fixed, 4 nits wont-fix.

This is a **retro, not a completion report** (`feedback_retro_not_report`): structured per-role (`feedback_four_role_retro_signals`), leading with process signals, not feature delivery.

---

## TL;DR for the next phase

The implementation was clean on the shipped scope (Task 1: `check_per_actor_rate_limit` function + `OnceLock` rate map with cap+eviction). **Two distinct recurring failures dominated the phase: daemon stale-ref (3rd recurrence in fed-in-d, firing on both fix-in-pr and bm-merge workers) and pre-merge CONFLICTING state due to a concurrent governance-v0 rustfmt reformat.** Both are known classes with documented recovery paths — but neither has a structural fix yet. The single highest-leverage fix for the next phase is a daemon-side `git fetch origin <branch>:<branch>` before `git worktree add` so workers branch from the live origin ref, not the stale daemon-local ref.

---

## Per-role signals

### ## Advisor

**What worked — keep:**

- **Conflict resolution inline on CONFLICTING PR.** When bm-merge missed (daemon stale-ref) and the PR was marked CONFLICTING (governance-v0 had received `2f13ffb80` — the v1-quality-r1 rustfmt reformat touching hundreds of files including `publish_trust_attestation.rs`), the advisor resolved by merging `governance-v0` into the phase branch from the lane worktree. Three conflict files: `.claude/decision-queue.json` (union), `.claude/runlog/bm-runlog.md` (append), and `crates/apub/activities/src/governance/publish_trust_attestation.rs` (accept phase-branch changes + take reformatted skeleton from governance-v0). All three resolved cleanly with zero data loss. This is the correct advisor posture: detect → diagnose → execute narrowly → verify → push.

- **Applying the fix-in-pr (cr-1) inline when Junior #415 missed.** Rather than re-queuing a second Junior cycle after the daemon stale-ref failure, the advisor applied the mutex-guard restructure directly from the lane worktree. The fix was mechanical (single test body: replace 4 separate lock acquisitions with one guard held through the entire test), audit-trailed via a `fix(bm): restructure per_actor_map_evicts_oldest_when_cap_reached to hold single mutex guard` commit, and did not touch any file outside the brief scope. Advisor never authors code — but when the Junior task demonstrably failed to land any change (worker declared "fix already applied" incorrectly), the inline apply is the correct next step to avoid wasting another Junior cycle on a known-miss.

- **Deferring Task 2 rather than stretching scope.** Task 2 (per-peer bound + SHA-256 nonce) carried a legitimate TOCTOU concern (the `contains_key` → `remove` race in the rate-map was not fully closed in the plan). The advisor deferred Task 2 to a future phase and shipped Task 1 cleanly, documenting the carry-forward explicitly. Scope discipline under pressure is a repeatable signal — deferral with a written rationale is better than partial/incorrect implementation.

- **E2e validated cleanly before merge.** Local Phase-2 e2e (103/0/5, 36 min via `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full ..."`) completed before the CONFLICTING resolution was pushed, confirming no regression on the rate-map logic. The 5 ignored are pre-existing v0-polish TODOs (GH issues #42/#43/#45), not phase regressions.

**What didn't — fix:**

- **Daemon stale-ref fires on BOTH fix-in-pr and bm-merge workers (#415, #416).** This is the 3rd+ recurrence of the pattern (also: fed-in-c bm-cut runlog, v1-AD-e 5× stale-base-self-merge). The daemon-local `phase-v1-federation-inbound-d` ref was at `37b4fe9b4` (pre-brief commit `759b52bda`) when the advisor had already committed the bm-merge-1 brief and the cr-1 fix onto the phase branch. Both workers saw the stale ref and incorrectly concluded "work already done" (a different failure mode from v1-AD-e's stale-base-self-merge mass-deletion — here the worker was confused about whether its change was needed, not about which tree to fork from). **→ See Cross-cutting §1. Carry-forward fix proposal: daemon-side `git fetch origin <branch>:<branch>` before `git worktree add`.**

- **Pre-merge CONFLICTING state not anticipated.** The advisor did not check whether governance-v0 had received major reformatting commits during the phase's elapsed time before queuing bm-merge. A 30-second `git log --oneline origin/governance-v0 ^phase-v1-federation-inbound-d` check at pre-merge time would have revealed `2f13ffb80` (rustfmt reformat) and prompted a merge-into-phase-branch step before the bm-merge dispatch. **→ Carry-forward: add a governance-v0-vs-phase-branch divergence check to the pre-merge checklist (advisor-orchestrator.md §3.1 "bm-pr complete" stage, before gate 5 "merge confirm").**

- **bm-merge-1 brief on governance-v0 not pulled by daemon before task creation.** The brief was committed to `governance-v0` at `759b52bda` but the daemon did not have that commit locally when Junior #416 was dispatched. This is the same root cause as the stale-ref issue (daemon-local ref lags origin), but manifesting as "brief invisible to worker" rather than "wrong tree to fork from". The structural fix is the same (daemon fetch before worktree create), but the symptom is distinct — worth tracking separately.

### ## Planning

The plan held cleanly for Task 1. The per-actor rate-map design (insertion-order eviction, `OnceLock` singleton, `MAX_PER_ACTOR_RATE_ENTRIES = 10_000` constant) was accurate and implementable in a single task. The §16a stories decomposition was sound.

**What went well:**

- Task 1 scope was well-bounded: one function (`check_per_actor_rate_limit`), one constant, one `OnceLock`-guarded `HashMap<String, (u32, Instant)>`, insertion-order eviction by iterating to the first key. The impl worker (Junior — task not separately enumerated; advisor applied the fix-in-pr inline) shipped all of these in scope.
- The complexity score estimate (low) was accurate: one file touched, one function, no migrations, no new crates, no multi-crate changes.
- Task 2 deferral was well-justified: the TOCTOU between `contains_key` and `remove` under concurrent load was a real design gap the plan surfaced. Better to defer than to ship an incorrect fix.

**What didn't — fix:**

- **No plan-level pre-flight check for governance-v0 divergence.** The plan §15 DoD commands were correct, but the plan did not include a "check governance-v0 distance at bm-merge time" step. Large reformatting PRs landing on governance-v0 mid-phase are a known class (v1-quality-r1 landed exactly this way). A plan-level §15.N "before bm-merge: verify `git log --oneline origin/governance-v0 ^phase-v1-...` is empty or contains only non-conflicting changes" would have surfaced the need for a merge-forward step before the CONFLICTING state was reached.

- **Task 2 was scoped alongside Task 1 without a clear pre-condition chain.** The plan listed Task 2 (per-peer bound + SHA-256 nonce) as a parallel peer of Task 1, but Task 2 carried a design dependency (the TOCTOU fix for the rate-map itself). A `requires: [task 1]` annotation and an explicit "Task 2 design-review gate" before queuing would have surfaced the dependency earlier.

### ## Impl

**What worked:**

- Task 1 implementation (the `check_per_actor_rate_limit` function + rate-map infrastructure) was correct on first pass. The eviction logic (insertion-order, retain-all-but-first when at cap), the `OnceLock<Mutex<...>>` singleton pattern, the `MAX_PER_ACTOR_RATE_ENTRIES` constant, and the integration into the trust-attestation handler were all plan-faithful.
- The rate-map test (`per_actor_map_evicts_oldest_when_cap_reached`) was mechanically correct — the four-separate-lock-acquisition structure was a code style issue (found by CR), not a logic error. The test covered the eviction semantics correctly.

**What didn't — fix:**

- **Junior #415 (fix-in-pr-1) false "fix already applied" conclusion.** The worker branched from the stale daemon-local ref (`37b4fe9b4`, pre-brief), which predated the cr-1 brief commit (`759b52bda`). The worker read the source file, saw no mutex guard restructure was present yet, and then failed to apply it — instead incorrectly reporting the fix was "already applied." This is a stale-ref confusion failure, not a comprehension failure; the worker reached the wrong conclusion because its working tree was correct (rate-map logic present) but the specific cr-1 fix (mutex guard hold) was correctly absent. **Root cause: daemon stale-ref, not impl worker quality.**

- **No explicit `pre_push_cargo_check` output in the Task 1 commit.** The plan brief should have required `bash scripts/brehon/cargo-check.sh --workspace --features full` before push and a `CARGO_EXIT_0` marker in the commit body per `feedback_fix_impl_pre_push_cargo_check.md`. This was not enforced in the brief — the validate-pending-laptop DQ caught any cargo regressions, but the in-brief guard would have given earlier signal.

### ## BM (Branch Manager)

**What worked:**

- **bm-pr brief and dispatch correctly cut the PR.** PR #146 was opened into `governance-v0` (not `main`) with `--repo barrie-cork/lemmy` per `gh-pr-fork-target.md`. PR body summarised Task 1 scope and the Task 2 deferral rationale.
- **CR triage was sound.** cr-1 (mutex lock restructure) correctly classified as `fix-in-pr` at `severity: major` — the four-separate-lock-acquisition pattern is a data-race latency footgun in async test context. cr-2 through cr-5 (nits) correctly classified as `wont-fix`: minor naming, comment, and const-visibility nits that do not affect correctness or ADR compliance.
- **Findings YAML populated correctly.** Buckets: 1 fix-in-pr (cr-1), 4 wont-fix (cr-2, cr-3, cr-4, cr-5). Addressed-in for cr-1 points to the advisor's inline fix-in-pr commit (since the Junior #415 missed it). No open critical or major findings at merge time.

**What didn't — fix:**

- **Junior #416 (bm-merge) missed the brief for the same stale-ref reason as #415.** The bm-merge brief (`759b52bda`) was visible on `origin/phase-v1-federation-inbound-d` but NOT on the daemon-local ref the Junior worker branched from. The worker attempted to merge without reading the brief's CONFLICTING-resolution instructions and declared success on a no-op. The advisor executed the merge inline.

- **bm-merge L14 runlog required advisor inline authorship.** The advisor executed `gh pr merge 146 --merge --delete-branch` inline. The runlog `COMPLETE` entry (merge sha `897d3f72d`, merged 2026-05-22T18:58:42Z) was committed by the advisor at `ebfcf7aaf` on `governance-v0` with subject `chore(bm): merge PR #146 complete — 897d3f72d — federation-inbound-d shipped`. L14 is closed.

- **bm-merge brief not pulled by daemon in time.** This is a co-symptom of the stale-ref pattern: the brief committed at `759b52bda` was a `governance-v0` commit, not a phase-branch commit. The daemon was not tracking `governance-v0` closely enough to pick it up. Both fix-in-pr and bm-merge tasks are BM-verb dispatches that read their briefs from trunk — and both missed for the same reason.

### ## Cross-cutting

#### §1 — HEADLINE: daemon stale-ref — 3rd+ recurrence; now a confirmed pattern requiring structural fix

**Pattern class:** The daemon's local ref for a branch (either the phase branch or `governance-v0`) lags `origin/` after the advisor has pushed new commits from the laptop side. Junior workers branch from the daemon-LOCAL ref. When the daemon-local ref is stale, the worker's working tree does not see the brief or the recent commits, leading to incorrect "already done" conclusions or silent scope gaps.

**Occurrences in this phase:**
- Junior #415 (fix-in-pr-1): daemon-local `phase-v1-federation-inbound-d` at `37b4fe9b4` (pre-brief `759b52bda`); worker saw rate-map in place but not the cr-1 brief; concluded "fix already applied"; advisor applied fix inline.
- Junior #416 (bm-merge): same daemon-local ref lag; worker saw branch state from before the cr-1 brief + CONFLICTING resolution instructions; attempted merge on stale tree; advisor executed merge inline.

**Prior occurrences:**
- v1-federation-inbound-c: bm-cut runlog deleted (index-only file, caused by daemon hash-object workaround with stale working-tree state).
- v1-AD-e: 5× stale-base-self-merge pattern (daemon-local phase ref lagged origin; workers saw stale merge-base and mass-deleted advisor files).
- `feedback_daemon_local_trunk_stale_multi_lane.md`: documented fix = `git fetch origin <branch>:<branch>` from the daemon side before task dispatch (lane-safe, no checkout switch required).

**Structural fix proposal (not yet shipped):** before each `git worktree add` for a task, the daemon MUST run `git fetch origin <branch>:<branch>` to fast-forward the daemon-local branch ref to origin. Alternatively, fork the worktree directly from `origin/<branch>` (bypassing the local ref entirely). Until this lands, **the advisor must pre-push all briefs, run a `git fetch` verification via `ssh homeserver "cd /srv/brehon-fork && git fetch origin && git log --oneline origin/<branch> | head -3"` before each task dispatch, and budget inline advisor execution as the expected fallback for any Junior task whose brief was committed in the last N hours.**

**Mitigation applied this phase:** none beyond advisor inline execution. The PRE-PUSH MANDATE (from fed-in-c §Carry-forward) was in place for Task 1 (worker pre-pushes before daemon finalize), which is why Task 1 landed cleanly. The mandate was NOT extended to the BM-verb briefs (fix-in-pr and bm-merge), which is where the failures hit.

**→ Action: extend the PRE-PUSH MANDATE to BM-verb briefs whenever the brief was committed to a branch within N hours of task dispatch. Add to `bm-cut.md`, `bm-merge.md` brief templates.**

#### §2 — Pre-merge governance-v0 divergence (first occurrence of rustfmt-reformat CONFLICTING class)

The v1-quality-r1 sub-phase landed `2f13ffb80` (rustfmt reformat of ~N files including `publish_trust_attestation.rs`) on `governance-v0` while `phase-v1-federation-inbound-d` was in flight. The PR was marked CONFLICTING. Advisor resolved via merge-into-phase-branch (3 conflict files, all auto-resolved cleanly after manual review).

**This is the first recorded occurrence of the "concurrent reformatting PR CONFLICTING" class.** It is not a process breach — reformatting PRs are a normal part of the v1-quality-* sub-phase family — but it was not anticipated in the pre-merge checklist.

**→ Action: add a governance-v0 divergence check to advisor-orchestrator.md §3.1 at the "bm-pr complete" stage (before gate 5). Concretely: `git log --oneline origin/governance-v0 ^phase-v1-<phase>` — if non-empty, review each commit for scope. Reformatting commits on shared files → author a merge-forward commit on the phase branch BEFORE gate 5. Structural commits (migrations, new crates) → surface to user.**

---

## Per-task complexity scores

Format: `<files-touched> / <commits> / <runtime-min> / <max-log-silence-min>` per `feedback_retro_task_complexity_score.md`. Junior task data derived from advisor observation + DQ records; exact daemon runtime not available post-phase.

| Task | Role | Files | Commits | Runtime (min) | Max silence | Notes |
|---|---|---|---|---|---|---|
| Task 1 (impl — publish_trust_attestation.rs) | impl | 1 | 1 | ~15 | n/a | Clean one-pass. `check_per_actor_rate_limit` fn + `OnceLock` map + `MAX_PER_ACTOR_RATE_ENTRIES`. |
| fix-in-pr cr-1 (advisor inline) | impl (advisor) | 1 | 1 | ~10 | n/a | Mutex guard restructure in `per_actor_map_evicts_oldest_when_cap_reached`. Junior #415 missed (stale-ref). Advisor applied. |
| bm-pr | bm | PR+runlog | 1 | ~5 | n/a | PR #146 opened correctly. |
| bm-poll-cr + bm-triage | bm | YAML | 1 | ~5 | n/a | CR: 1 major (cr-1), 4 nits. Triage sound. |
| bm-merge (advisor inline) | bm (advisor) | runlog | 1 | ~20 | n/a | Junior #416 missed (stale-ref + CONFLICTING). Advisor merged `governance-v0` into phase branch, then executed `gh pr merge 146 --merge --delete-branch` inline. |
| CONFLICTING resolution | advisor | 3 | 1 | ~15 | n/a | DQ, runlog, Rust file conflict resolution after governance-v0 divergence. All auto-resolved cleanly after manual review. |

**Aggregate:** 1 impl task (~15 min cargo runtime) + 2 advisor inline executions (fix-in-pr, bm-merge) + conflict resolution. **Both Junior BM-verb workers required stale-ref recovery, accounting for ~50% of the phase's active orchestration time.** The implementation itself was low-complexity; the phase cost was stale-ref overhead + conflict resolution.

Phase 2 e2e: 103 passed / 0 failed / 5 ignored, ~36 min local (`cargo-test.bat --workspace --test e2e --features full`). Lib tests: 22/22.

---

## Lessons promoted this phase

No new lesson files authored inline in this retro commit. The observations below meet or approach the promotion threshold; promote in a follow-up `chore(lessons):` commit on `governance-v0`.

1. **Daemon stale-ref — BM-verb briefs on `governance-v0` also affected, not only impl-task briefs on phase branch.** The existing `feedback_daemon_local_trunk_stale_multi_lane.md` + `feedback_junior_292_stale_base_recover_recipe.md` cover the impl-task variant. The BM-verb variant (worker reads `governance-v0` brief, daemon-local `governance-v0` lags origin, brief invisible) is a distinct symptom requiring a distinct entry in the lesson or a new lesson `feedback_daemon_stale_bm_verb_brief_miss.md`. **3rd+ class recurrence; promote.**

2. **Pre-merge governance-v0 divergence check (first occurrence — watch for 2nd).** When a large reformatting commit lands on governance-v0 mid-phase, the phase PR is marked CONFLICTING and the merge cannot proceed until the phase branch is updated. The check `git log --oneline origin/governance-v0 ^phase-v1-<phase>` before gate 5 catches this. **1st occurrence — do not promote yet; add to carry-forward watchlist.**

3. **PRE-PUSH MANDATE must extend to BM-verb briefs.** The mandate (from fed-in-c §Carry-forward, DQ #338 mitigation) was applied only to impl-task briefs. BM-verb briefs committed to `governance-v0` are equally vulnerable. Until the daemon-side structural fix lands, all briefs (impl + BM) committed within N hours of task dispatch should carry the mandate or the advisor should execute the merge inline as the default path. **Extend `feedback_junior_finalize_skips_when_worker_pre_pushes.md` or author a new entry targeting BM-verb briefs specifically.**

---

## Carry-forward to next sub-phase

### Task 2 deferred (per-peer bound + SHA-256 nonce)

Task 2 was deferred because:
- (a) The `contains_key` → `remove` TOCTOU race in the per-actor rate-map was not fully closed in the plan. A `Mutex<HashMap>` under concurrent tokio tasks can be held across await points — the design needed a clear concurrency model before implementation.
- (b) SHA-256 nonce adds scope (crypto dep, storage, expiry) that would have pushed complexity to the point of needing its own sub-phase planning.

**Recommendation for v1-federation-inbound-e:** plan Task 2 with an explicit concurrency model section (Tokio + `Mutex<HashMap>` hold discipline; `DashMap` alternative if per-key locking is needed; nonce storage schema if SHA-256 is in scope). The per-actor rate-map logic from Task 1 (`check_per_actor_rate_limit`) is the foundation — Task 2 builds on top.

### Daemon stale-ref structural fix (open — DQ #338 + recurrence class)

DQ #338 (`kind: "blocker"`, filed 2026-05-21) covers the daemon wrong-ref reset class. The fed-in-d stale-ref misses (#415, #416) are a related but distinct symptom (worker forks from stale daemon-local ref rather than the daemon issuing a `reset --hard`). Both point to the same structural gap: the daemon does not fetch origin before creating worktrees.

**Process invariant until fix lands: every impl-task brief AND every BM-verb brief (bm-cut, bm-merge, bm-pr) MUST carry the PRE-PUSH MANDATE in §4 Constraints.** Worker pre-pushes its commit to `origin/junior/<branch>` before the daemon's finalize step; advisor manually finalize-merges from `origin/junior/<branch>`. Checked against fed-in-c + fed-in-d evidence — the mandate prevents the stale-ref confuse; the advisor inline execution is the fallback when it fires anyway.

### L14 runlog COMPLETE — completed this session

The bm-merge was executed advisor-inline. The L14 runlog `COMPLETE` entry was authored by the advisor at `ebfcf7aaf` on `governance-v0` (`chore(bm): merge PR #146 complete — 897d3f72d — federation-inbound-d shipped`). No action outstanding.

### Governance-v0 divergence check (carry-forward watchlist)

Add to advisor pre-merge checklist (advisor-orchestrator.md §3.1, "bm-pr complete" → gate 5):

```
git log --oneline origin/governance-v0 ^phase-v1-<phase>
```

If non-empty: review each commit. Reformatting on shared files → merge-forward first. New structural work → surface to user.

### DQ #229 (Shape G re-enable) still pending

Shape G is SUSPENDED until 2026-06-01 (per `project_shape_g_suspended_2026_05_16.md`). All `validate-pending-laptop` entries in this phase used laptop-local cargo. No change to this posture from fed-in-d.

---

## Pipeline state at retro

- PR #146 **MERGED** `897d3f72d`; `phase-v1-federation-inbound-d` deleted from origin; governance-v0 trunk advanced.
- E2e: 103/0/5 (validate-pending-laptop result=pass, advisor-laptop). Lib tests: 22/22.
- CR findings: all resolved (1 fix-in-pr done, 4 nits wont-fix, 0 open).
- DQ pending: 0 (at retro authorship; DQ #229 Shape-G re-enable is a separate lane).
- Task 2 explicitly deferred (per-peer bound + SHA-256 nonce) — scope reduction recorded in PR body + this retro.
- L14 runlog COMPLETE committed `ebfcf7aaf` ✓.
- **Next:** user gate 6 (retro sign-off) → `/brehon-phase-transition` → plan v1-federation-inbound-e (Task 2 scope).
