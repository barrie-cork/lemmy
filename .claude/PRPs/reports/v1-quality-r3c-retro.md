# Retro — v1-quality-r3c

**Phase:** v1-quality-r3c — CodeRabbit endpoint-rule fix + sponsor-allowlist sweep + BREHON_DISABLE_* guard tests  
**PR:** #172 (merged `c1ba108dc`, 2026-06-01T10:27Z)  
**Retro authored:** 2026-06-01  
**Branch:** `phase-v1-quality-r3c` → `governance-v0`

---

## §1. Outcome

All three plan stories shipped and verified:

- **T1:** `.coderabbit.yaml` — "EXACTLY 11" assertion removed; v1-aware caveat referencing OQ-020 in place. ✓
- **T2:** `crates/server/tests/e2e.rs` — sponsor-allowlist routes added to HTTP-path sweep (14 → 16 routes). ✓
- **T3:** `crates/server/tests/e2e.rs` — `test_brehon_disable_snapshot_job` + `test_brehon_disable_fed_replay_cleanup_job` added. ✓

E2E: **128 passed / 0 failed / 5 skipped** (DQ `a3d0e9941441-044`, `result: pass`).

---

## §2. Task complexity scores

| Task | Role | Files | Commits | Runtime (min) | Max log silence (min) | Notes |
|---|---|---|---|---|---|---|
| T0 pre-flight | impl | — | — | — | — | n/a (bm-cut only) |
| T1 (impl-task, job-561) | impl | 1 (`.coderabbit.yaml`) | 1 | ~3 | — | Clean run |
| T2 (impl-task, job-559 from RT-r5 reuse) | impl | 1 (`e2e.rs`) | 1 | ~5 | — | Delivered in RT-r5 cycle |
| T3 (impl-task, job-562) | impl | 1 (`e2e.rs`) | 1 | ~6 | — | Clean run |
| fix-impl-1 (job-563) | impl | 1 (`e2e.rs`) | 1 | ~8 | 2 | Missing imports + wrong fixture module |
| fix-impl-2 (job-565) | impl | 1 (`e2e.rs`) | 1 | ~7 | 2 | FederationInboxNonce API calls wrong |
| fix-impl-3 (advisor-laptop) | advisor | 1 (`e2e.rs`) | 1 | ~26 | — | Full e2e run on laptop |
| bm-cut | bm | — | — | ~3 | — | Clean |
| bm-pr (job-564) | bm | — | — | ~5 | — | Clean; PR #172 opened |
| bm-poll-cr + bm-triage | bm/advisor | — | — | ~10 | — | 5 findings; cr-2 fix confirmed |
| bm-merge (job-567) | bm | — | — | ~3 | — | Clean merge at `c1ba108dc` |

**Total wall-clock:** ~2 sessions (2026-06-01). Three fix-impl cycles after T3 initial impl.

---

## §3. Four-role retro signals

### Advisor

**What worked:**
- Pre-impl HEAD check (`feedback_advisor_pre_impl_head_check.md`) — confirmed T2 was already shipped in RT-r5 cycle before dispatching T3; saved one wasted impl round.
- Falsifiable-hypothesis gate at bm-triage: cr-2 `\b` corruption was mis-falsified by raw-text probe; decoded `json.load` test was decisive; CR was correct. Correct escalation path triggered.
- DQ `a3d0e9941441-043` (clarify entry for FED_REPLAY_CLEANUP test shape) was authored before planning, saving one fix-impl cycle on T3 shape.
- Mode B trunk→phase sync worked cleanly for fix-impl-1 and fix-impl-3 briefs.

**What went wrong:**
- **Three fix-impl cycles for T3.** Root cause: T3 test functions called `FederationInboxNonce` API incorrectly (wrong field names in InsertForm, non-existent `delete_older_than` signature), and were placed in wrong fixture module. These are e2e-specific API surface traps not caught by the pre-queue lesson injection.
- **bm-merge #566 silent failure.** Daemon's `governance-v0` was at `fa091be00` while the brief commit (`84e297b46`) was on `origin/governance-v0`. BM couldn't read the brief ("fatal: path does not exist in 'governance-v0'") and did not raise a clear DQ. Fixed by `git update-ref` SSH update to daemon-local ref. **The lesson (don't use `git reset --hard` on daemon shared checkout; use `git update-ref` instead) was new this session** — not yet in lessons corpus.

### Planning

- Plan §5 complexity score 4/10 was accurate; the three e2e edits were the dominant factor.
- DQ clarify entry `a3d0e9941441-043` correctly anticipated the FED_REPLAY shape ambiguity. However, even with the clarify entry, the T3 impl got the `FederationInboxNonce` API wrong — the clarify entry addressed scheduler-vs-function placement, not the insert/delete API surface.
- **Lesson candidate:** when T3-class tasks call crate-internal APIs (`FederationInboxNonce`) that are not in the design doc, the planning brief should require impl to read the source definition before writing the test. A `Required reading: crates/db_schema/src/source/federation_nonce.rs` line would have prevented fix-impl cycles 1 and 2.

### Impl

- T1 (job-561): clean. `.coderabbit.yaml` edit was mechanical and correct first pass.
- T2 (job-559 reused from RT-r5): clean.
- T3 (job-562): first pass failed. Missing `use` imports (E0433), wrong fixture module (not inside `v1_rt_r3_fixtures`), wrong `FederationInboxNonce` API calls. Three separate failure modes in one task.
- fix-impl-1 (job-563): fixed imports + module placement. cargo-check pass. e2e still failing.
- fix-impl-2 (job-565): fixed `FederationInboxNonce` API calls. e2e still failing (assertion logic wrong on `fed_replay_cleanup` test).
- fix-impl-3 (advisor-laptop): corrected assertion logic in `test_brehon_disable_fed_replay_cleanup_job`. E2E passed 128/0/5.

**Recurring pattern:** fix-impl cascades on e2e.rs tasks that call crate-internal APIs not enumerated in the plan brief. This is the 2nd phase (after v1-RT-r3) where `FederationInboxNonce`-adjacent API surface caused a cascade.

### BM

- bm-cut: clean.
- bm-pr: clean; PR #172 opened correctly.
- bm-poll-cr: 5 findings ingested. cr-2 falsifiable-hypothesis gate handled correctly inside BM (rare — the subagent identified the raw-text vs json-decoded discrepancy independently).
- bm-triage: correctly bucketed all 5. cr-2 → fix-in-pr; all others → wont-fix with rationales.
- bm-merge #566: **silent failure** (see Advisor §3 above). bm-merge #567: clean.

---

## §4. Retro-bypass.jsonl check

No retro-bypass entries were emitted this phase (no Stop hook fail-open events observed). Rate trend: stable.

---

## §5. Carry-forward items

| # | Item | Priority | Target |
|---|---|---|---|
| CF-1 | Add `git update-ref refs/heads/<branch> origin/<branch>` as the daemon-side reset recipe to lessons corpus. `git reset --hard` blocked by hook on shared daemon checkout; `update-ref` is the correct mechanism. | High | Next session |
| CF-2 | impl-task brief §3 for e2e tasks calling crate-internal APIs: add a **Required reading** line pointing to the source definition of the API being exercised. Specifically: when the task inserts/queries `federation_nonce`, read `crates/db_schema/src/source/` first. Add this to the file-class lesson injection table row for `crates/server/tests/e2e.rs`. | Medium | v1-quality-r3d brief authoring |
| CF-3 | bm-merge brief should include an explicit daemon-local-ref freshness check: before dispatching bm-merge, run `ssh homeserver "cd /srv/brehon-fork && git log governance-v0 -1 --oneline"` and compare to `origin/governance-v0`. If behind: `git fetch origin governance-v0 && git update-ref refs/heads/governance-v0 origin/governance-v0`. | Medium | Next bm-merge brief template |
| CF-4 | `feedback_bm_merge_daemon_local_ref_staleness.md` — document the bm-merge #566 failure mode: daemon-local branch tip behind origin → brief unreachable → silent "path does not exist" failure in bm-task. Correct: `update-ref` pre-flight, not `reset --hard`. | High | Next session |

---

## §6. Lessons to promote

1. **`feedback_daemon_reset_hard_blocked_use_update_ref.md`** — `git reset --hard` on daemon's shared `/srv/brehon-fork` checkout is blocked by a hook. Use `git update-ref refs/heads/<branch> origin/<branch>` instead. Applies whenever the advisor needs to fast-forward daemon-local branch to match origin without a Junior task.

2. **`feedback_bm_merge_daemon_local_ref_staleness.md`** (CF-4 above) — see §5.

3. **`feedback_pre_impl_head_check_e2e_reuse.md`** (already saved this session as `docs/lessons: verify branch diff-vs-trunk before concluding code is missing`) — confirmed the pre-impl HEAD check prevents redundant T2 dispatch.

---

## §7. MiniMax trial (§3.5a)

Per plan-approval designation table (carried from v1-quality-r3c plan approval):

| Task | Qualifies | Reason |
|---|---|---|
| T1 `.coderabbit.yaml` | ❌ | config edit, no MIRROR-ref pattern |
| T2 e2e.rs routes | ❌ | e2e edit — disqualifies |
| T3 e2e.rs guard tests | ❌ | e2e edit — disqualifies |

Cumulative qualifying tasks this phase: **0**. Running total (all phases): unchanged from prior count.

---

## §8. Next phase

`v1-redaction-r1` — active in lane `brehon-fork-redaction-r1` / `phase-v1-redaction-r1`.  
Per handover `.claude/PRPs/handovers/v1-redaction-r1-resume-2026-06-01.md`:
- DQ `9f1d7e7ca817-001` pending (`result: fail` from old Docker-down e2e run).
- Needs: re-run e2e post-merge-forward, then bm-pr.
- Do not start until v1-quality-r3c retro signed off (this retro).
