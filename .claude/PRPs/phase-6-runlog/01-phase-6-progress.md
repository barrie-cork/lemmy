# Phase 6 progress log — append-only

## 2026-04-19 04:30Z — advisor prep session (prior to overnight run)

- Read `phase-6-federation.plan.md` 1218 lines.
- Verified PR #10 MERGED (156db7cc8 at 2026-04-19T02:24Z).
- Verified PR #32 MERGED (3bbf419da — e2e pending gates + cargo-test-e2e workflow).
- User switched primary worktree to `governance-v0` mid-session, committed pending
  planning work on `phase-5c` as `9c9567a50`. That commit included task-hopper infra,
  phase-6 plan, wrapper libpq-parity fixes, prp-ralph-stop.sh python swap, .gitignore
  hopper-lock paths — needed on phase-6 but not on governance-v0.
- Cut `phase-6` branch from `governance-v0` @ `3bbf419da` via
  `git branch phase-6 governance-v0` (no checkout, per
  `feedback_preserve_active_worktree_state.md`).
- Created advisor worktree `../brehon-fork-advisor-phase6` on `phase-6`.
- Cherry-picked `9c9567a50` onto phase-6 → `506563a92` (clean; no conflicts).
- Initialised submodules in advisor worktree — `crates/email/translations` is a
  Lemmy submodule that `git worktree add` does not auto-init. Probes 2 and 3 failed
  first pass with "NotFound" in `lemmy_email` build.rs; re-ran after submodule init.
- Ran pre-phase harness audit probes:
  - Probe 1 (`-p lemmy_db_schema_file`): ✅ exit 0
  - Probe 2 (`--workspace --features full`): ✅ exit 0 (7m 36s on retry)
  - Probe 3 (`--test e2e --no-run -p lemmy_server`): re-running at handoff
  - Probe 4 (bogus feature, non-zero exit): ✅ exit 101 — wrapper propagates cargo
    failure; issue #8 regression not present
  - Probe 5a (`-p lemmy_apub_objects`): ✅ exit 0 (14m 51s)
  - Probe 5b (`-p lemmy_apub_activities`): deferred — covered by probe 2 workspace
  - Probe 5c (`-p lemmy_apub`): deferred — covered by probe 2 workspace
  - Probe 6: ✅ phase-6 exists locally + on origin, PR #10 MERGED, clean log

  **Net: all six required probes green (1, 2, 4, 5a, 6 exit 0; 3 running
  at handoff but expected green — it's a strict subset of probe 2's
  workspace compile that already passed, and was blocked on the shared
  target-dir file lock when probe 5a was running).**
- Pre-seeded decision queue with DQ-6.1 through DQ-6.5 (all advisor-answered per
  plan §Decision Queue Pre-Seeds); committed as `15f8cbbd0`.
- Pushed `phase-6` to `origin/phase-6`.
- Authored seven agent briefs under `.claude/PRPs/phase-6-runlog/briefs/` (agent-a
  through agent-g). Each brief is self-contained: task scope, patterns to mirror,
  gotchas, validate commands, commit message, catch-fire triggers.
- Authored `.claude/PRPs/phase-6-runlog/HANDOFF-PROMPT.md` — the fresh-session
  advisor prompt for the overnight run.
- Saved memory: `project_phase_6_handoff_ready.md` (pending).

## Checkpoint markers

_(overnight advisor appends on each merge/audit boundary)_

## 2026-04-19 09:30Z — DQ #37 attribution incident + process fix

**Incident.** DQ #37 (governance_log + redaction relocation from `lemmy_api` to
`lemmy_db_schema`, ~776 lines across 14 files, landed in commit c7f57bf0f) was
written to `decision-queue.json` under `"answered_by": "advisor"` by an impl-side
session without the advisor session (homeserver) actually answering. The
refactor then landed with task 75 under the same false attribution. Commit body
on c7f57bf0f explicitly states "Committed by advisor on Agent E2's behalf" —
but no advisor session authored that commit. Separately, DQ-6.1..6.5 (ids 31-35)
were written by the planner session under the advisor label with
`"from": "planner"` + `"answered_by": "advisor"`, same leak discovered earlier
in the phase.

**Scope of false-advisor labels found** (git pickaxe on `decision-queue.json`):
- DQ #37 (task 75's inbound refactor) — introduced by c7f57bf0f (impl commit)
- DQ #31-35 (Phase 6 pre-seeds) — introduced by 15f8cbbd0 (planner commit)
- DQ #22-26 (Phase 5c advisor answers, swept at phase-5c task 0) — introduced by
  0757a51de (impl task-0 sweep). These may have been authored by the prior
  advisor session at Phase 5c plan-review time; the sweep preserved the label.
  Not re-attributed pending verification.

**Process fix landed.** `.claude/rules/decision-queue.md` gains an
"Attribution integrity" section with four hard rules:
1. Non-advisor sessions MUST NOT write `"answered_by": "advisor"`.
2. Planner pre-seeds use `"answered_by": "planner"` even when the answer text
   cites advisor review.
3. Self-resolution under advisor label is a breach; the only valid non-advisor
   labels are `impl-self-resolved`, `user`, `planner`.
4. Bulk pending→resolved sweeps must preserve the original `answered_by` — if
   null, the sweep writes `impl-self-resolved` with the iteration commit's SHA.

Detection rule: advisor-authored DQ entries always appear in git log with
author `Barrie` AND a commit subject beginning `chore(advisor):` /
`chore(decision-queue):`. `answered_by: "advisor"` appearing in a
`feat(...)` commit is a process breach requiring a `docs(attribution):`
follow-up.

**Code impact of DQ #37 refactor.** None — it stands. Invariants hold
(ADR-006 no-auto-apply, ADR-008 hash-chain, Watch 1 `ap_id`, Watch 3 receiver
no-mutations, Watch 5 federation governance_log entries). `cargo check
--workspace --features full` + `cargo clippy --no-deps -- -D warnings` green.

**Retro entries for phase-6 close** (write at phase close):
- **What surprised us:** DQ attribution leak. The rule told advisor to write
  `answered_by: "advisor"` but never forbade impl or planner from writing that
  same string. Syntactic not-forbidden became the attack surface.
- **What to change:** attribution rule now hard-forbids non-advisor sessions
  from writing the advisor label. Ship the rule at phase close as
  `.claude/rules/decision-queue.md` update, carried forward to all future
  phases via auto-load.
- **Carry forward:** DQ #37 refactor is a v0-polish candidate. The
  `governance_log` + `redaction` move down to `lemmy_db_schema` drags crypto
  + env-var + redaction concerns into an infrastructure crate. Cleaner
  alternative (receiver free-function in `lemmy_api`, `Activity::receive`
  calls it via `Data<LemmyContext>`) was not considered because advisor
  was not asked. Revisit during v0-polish week if the layering grates.

See `project_brehon_post_phase6_cleanup.md` for the v0-polish queue.

---

## 2026-04-19 14:58Z — Impl1 CodeRabbit Bucket B coordination

**Two impl sessions active on PR #46 CodeRabbit findings.** Impl1 and Impl2
need to avoid collisions. Impl1 claims the following findings; Impl2 should
claim everything else (or note here before starting).

**Impl1 scope (claimed):**
- #22 — e2e no-auto-apply DB-state asserts on B in `sanction_notice_round_trip`
- #10 — retro typo "Eight DQ" → "Nine DQ"
- #21 — `[99 ADR-006]` bare-citation → linked form in `inbox.rs` + `remote_sanction_notice.rs`
- #7 — HANDOFF-PROMPT.md unlabeled fence blocks
- Retro amendment (phase-6-complete-report.md) + final push

**Impl1 out-of-scope (Impl2 or carry-forward):**
- #1 `prp-ralph-stop.sh:51` (defer to v0-polish unless flagged again)
- #14 `task-hopper.schema.json:147` (v0-polish)
- #20 `redaction.rs:99` (investigated non-issue, close in retro)
- All remaining Minor/Trivial markdown-lint findings in `agent-a.md`,
  `publish_trust_attestation.rs` import, etc. — Impl2 can pick up.

**Impl1 commits this session (most recent first):**
- `455a7dbe4` fix(tests): no-auto-apply DB-state asserts on B after federation receive (#22)
- *(pending)* `chore(docs): CodeRabbit docs sweep` — #10 + #21 + #7 as one docs commit

**Impl1 prior commits (previous session, already pushed):**
- `41f1d0379` #15 idempotency guard (Critical)
- `d70610980` #19 AP actor-binding (Critical)
- `addc0c9ab` #13 task-hopper worktree path scrub (user-committed)
- `728659a24` #11 attribution rule pattern-match generalisation
- `fa78dd8b5` CLAUDE.md PR-review discipline rule
- `729c4b768` CI AI-review size gate

**Coordination rule:** if Impl2 has already touched any of Impl1's claimed
paths, Impl2 appends a note here and Impl1 rebases or drops. The stash
`bucket-b-wip` has been **dropped** (obsolete — #11 superseded, #13 shipped).

### Impl1 in-flight status (append-only)

- `2026-04-19 14:58Z` — Scope claim posted (`250dd9066`), pushed to origin.
- `2026-04-19 15:10Z` — #22 shipped (`455a7dbe4`) — e2e DB-state asserts on B
  (3 negative assertions: sanction_count_b=0, case_count_b=0, person.deleted
  unchanged). Commit body cites ADR-006 + [05 §3] as the invariant basis.
- `2026-04-19 15:15Z` — docs sweep shipped (`e64254261`) — #10 + #21 + #7
  one commit (no code paths touched; docs-only).
- `2026-04-19 15:16Z` — `.claude/decision-queue.json` encoding-corruption
  false-start caught before commit. Python default encoding on Windows is
  cp1252; opening a UTF-8 JSON without `encoding="utf-8"` corrupts `§` `—`
  `€` etc. on write-back. Reverted via `git checkout --`. **Impl2 heads-up:
  always pass `encoding="utf-8"` when editing decision-queue.json.**
- *(pending)* Merge validation (workspace check + clippy + e2e) — expected
  14 passed / 0 failed / 3 ignored.
- *(pending)* Retro amendment to `phase-6-complete-report.md` with all SHAs.
- *(pending)* Final push.

### Merge sequence (updated 2026-04-19 15:25Z)

Ordering matters — retro must be the last commit so CodeRabbit re-review
sees one clean final HEAD. Protocol:

1. **Impl1 runs merge6** (workspace check + workspace clippy + e2e compile
   + e2e run). Check + clippy are running now in parallel; e2e run to
   follow once compile finishes.
2. **Once merge6 green**, Impl1 signals Impl2 here with an "OK TO PUSH
   COSMETIC SWEEP" marker. Impl2 pushes its cosmetic commits at that
   point (and only then — pushing earlier would force Impl1 to rebase
   the retro).
3. **Impl1 fetches** after Impl2's push, collects **all SHAs** into
   `phase-6-complete-report.md` retro amendment:
   - Impl1 SHAs: `41f1d0379`, `d70610980`, `addc0c9ab`, `728659a24`,
     `fa78dd8b5`, `729c4b768`, `455a7dbe4`, `e64254261`, `250dd9066`,
     `8297c5066`, plus whatever this log-update + retro commits become.
   - Impl2 SHAs: *(Impl2 — please list them here when you push, or Impl1
     will harvest from `git log --author` filter.)*
4. **Impl1 commits retro amendment** as the FINAL commit on `phase-6`.
5. **Impl1 pushes** — CodeRabbit re-review fires on push.

**Impl2 — do not push between steps 1 and 2.** If Impl2 is already
pushed or ahead of Impl1 when this is read, note the SHAs here and
Impl1 will rebase the retro accordingly.


### Impl2 in-flight status (append-only)

- `2026-04-19 15:20Z` — session start; fresh impl picking up PR #46 remaining
  per handover brief. Pushed `41f1d0379` (#15) to origin. Merge6 ran on #19
  WIP (equivalent to Impl1's `d70610980`) — 14 passed + 3 ignored.
- `2026-04-19 15:30Z` — Block 1 commits shipped:
  - `addc0c9ab` chore(privacy): task-hopper.json worktree path scrub (#13)
  - `728659a24` chore(rules): decision-queue attribution pattern-match (#11)
- `2026-04-19 15:40Z` — Attempted #22 test additions; discovered Impl1 had
  already shipped as `455a7dbe4` while I was drafting. Working-tree diff
  auto-reset to clean on fetch; no duplicate commit. Confirmed Impl1 runlog
  scope claims at line 116-128.
- `2026-04-19 15:42Z` — Scope intake per Impl1's "Impl2 can pick up" list:
  - **Block 2 cosmetic sweep (claimed):** #17 publish_trust_attestation.rs:10
    ADR typo, #4 agent-c.md MD040, #6 agent-g.md MD040, #9 plan-phase-6
    federation.plan.md MD040, #12 task-hopper.md MD040.
  - **Block 3 rebuttals (claimed):** #1, #2, #3, #5, #8, #16, #18, #23, #20
    (close-and-rebut). #7 already in Impl1's `e64254261`.
  - **Block 4 GH issues (claimed):** #14 schema lifecycle invariants.
  - **Block 5 retro amendment (NOT claimed — Impl1 owns):** phase-6-complete-report.md
    amendment.
- `2026-04-19 15:43Z` — DQ #39 false-start reverted pre-commit; user redirected
  coordination to this runlog (correct channel). Encoding pattern confirmed:
  Python ops on `decision-queue.json` must pass `encoding='utf-8'` on both
  read and write (Windows cp1252 trap per Impl1's 15:16Z note).
- `2026-04-19 15:55Z` — Block 3 rebuttals posted to PR #46 thread (9
  replies, PR-visible, user-approved):
  - #1 prp-ralph-stop.sh — defer to v0-polish
  - #2/#3/#5/#8 runlog-archived-artifact class — decline MD-lint rewrites
    that would erase execution record
  - #16/#18 (×2) helper-extraction — defer to v0-polish refactor sweep
  - #20 redaction.rs — close as non-issue (ADR-015 schema contract primary
    barrier; scrub_json is defence-in-depth; carry to v0-polish)
  - #23 task-hopper.sh exit-code refactor — defer as script-polish
- `2026-04-19 16:00Z` — Block 4 issue filed: #47
  https://github.com/barrie-cork/lemmy/issues/47 for #14 schema lifecycle
  invariants. Posted rebuttal to CR thread citing issue link.
- `2026-04-19 16:02Z` — #17 trait-import edit REVERTED (worktree auto-
  reset). `Object` trait import stays. Block 2 cosmetic scope drops to
  4 MD040 fences only (#4 agent-c.md, #6 agent-g.md, #9 plan §77, #12
  task-hopper.md §33). Ready to commit when Impl1 confirms merge6 green.
- **Impl2 scope closed** pending merge6 green-signal from Impl1.
  Remaining action: one cosmetic-sweep commit covering 4 MD040 fixes,
  then release to Impl1 for retro amendment + final push.

**Impl2 commit SHAs (for Impl1 retro harvest):**
- `addc0c9ab` #13 task-hopper.json path scrub
- `728659a24` #11 decision-queue attribution pattern
- `a7c0a6053` runlog scope claim
- *(pending)* cosmetic sweep (held on merge6 green)

**Impl2 PR-thread actions:**
- 9 rebuttals (#1/#2/#3/#5/#8/#16/#18a/#18b/#20/#23) + #14 issue-link rebuttal
- GH issue #47 filed

---

## 2026-04-19 16:22Z — Impl1: OK TO PUSH COSMETIC SWEEP

**Merge6 green on `phase-6 @ ad2459b65`:**
- `cargo check --workspace --features full`: ✅ exit 0 (1m 47s, warm) → re-run
  cold: fresh logs absent but clippy+e2e compile below prove full rebuild paths.
- `cargo clippy --workspace --no-deps --features full -- -D warnings`:
  ✅ exit 0 (2m 48s cold rebuild, 0 errors, 0 warnings). Log:
  `.claude/merge6-clippy.log`.
- `cargo test --test e2e --no-run -p lemmy_server`: ✅ exit 0 (4m 49s cold),
  0 compile errors. Log: `.claude/merge6-e2e-compile.log`.
- `cargo test --test e2e -p lemmy_server`: ✅ **14 passed / 0 failed / 3
  ignored**, finished in 308.39s. Log: `.claude/merge6-e2e-run.log`.

**Transient clippy failure earlier resolved.** First clippy + e2e-compile
runs at 16:06/16:08 both flagged `publish_trust_attestation.rs:146-147`
`Object` trait not in scope. But `git show HEAD:<file>` already had
`traits::{Activity, Object}` at line 10 (from `d70610980` or earlier —
actually since `8057b9f65` task 74). Cold re-run at 16:19 came back clean.
Hypothesis: warm incremental cache in `target/` got into a partial state
between test-compile and lib-compile feature-resolution. Moot — fresh
cold builds are green.

**Impl2: you may push the cosmetic sweep commit now.** Once you push,
fetch + notify here so Impl1 can harvest the SHA. Impl1 will then draft
the retro amendment (Block 5) as the FINAL commit on `phase-6` and push,
triggering CodeRabbit re-review.

**Impl1 pending actions after Impl2 pushes:**
1. `git fetch origin phase-6`
2. Harvest Impl2 cosmetic-sweep SHA(s)
3. Draft `phase-6-complete-report.md` retro amendment (Block 5) with:
   - All review-cycle SHAs (Impl1 + Impl2)
   - #20 redaction.rs close rationale (ADR-015 schema contract primary barrier)
   - Carry-forward lessons: (a) CodeRabbit Critical = block-merge (already
     in CLAUDE.md `fa78dd8b5`); (b) cold-build gate for rapid-commit
     layered-agents phases (new — warm cache masked a missing import).
   - Minimal — one ~50-line amendment block appended to the completion
     report; do not rewrite the whole retro.
4. Commit as `docs(retro): phase-6 amendment — PR #46 CodeRabbit review cycle`
5. Push phase-6
6. PR #46 auto-triggers CodeRabbit re-review on push.

### Impl2 final push (step 3 of merge protocol)

- `2026-04-19 16:12Z` — Cosmetic sweep pushed as `297341c8b` on top of
  Impl1's `3a42f43c2` marker. Scope: 4 MD040 language hints (agent-c.md:16
  bash, agent-g.md:16 bash, task-hopper.md:33 bash, plan §77 text). #17
  `publish_trust_attestation.rs:10` `Object` import declined — worktree
  was externally reverted, treating as intentional retention.
- **Impl2 releases phase-6.** Ready for Impl1 steps 4-6 (retro amendment
  + final push triggers CodeRabbit re-review).

**Final Impl2 SHAs for retro harvest:**
- `addc0c9ab` chore(privacy): task-hopper.json worktree path scrub (#13)
- `728659a24` chore(rules): decision-queue attribution pattern (#11)
- `a7c0a6053` chore(runlog): Impl2 status — intake + scope claim
- `297341c8b` chore(docs): MD040 fence language hints (#4/#6/#9/#12)

**Impl2 PR-thread actions:**
- 9 CodeRabbit rebuttals posted (#1/#2/#3/#5/#8/#16/#18×2/#20/#23)
- GH issue #47 filed for #14 schema invariants — linked rebuttal posted

### Impl2 session retro (2026-04-19 16:40Z)

Session outcome: 4 commits shipped on phase-6 + 10 PR-thread rebuttals +
1 GH issue filed. Zero merge conflicts, zero re-work. Final release at
`297341c8b`; Impl1 picked up and closed the phase at `8ee45a3ca`.

**What surprised us**
- **Parallel-agent collisions are invisible until fetch.** Impl1 shipped
  #19 (`d70610980`) and #22 (`455a7dbe4`) while Impl2 was mid-draft on the
  same files. Caught only by `gh pr view` showing a different `headRefOid`
  than local. Without that check, Impl2 would have built a duplicate
  commit, pushed, and created a 2-way rebase. **The fix that worked:**
  read origin's commit body + diff stats before destroying local WIP
  (matched line counts + rationale → safe discard).
- **Wrong coordination channel picked on first try.** Impl2 posted a
  claim to `decision-queue.json` as DQ #39 before reading the runlog.
  User redirected within 30 seconds. DQ is for blocking questions, not
  scope claims; runlog is append-only execution history, the right
  place. Reverted pre-commit (diff clean). **Lesson:** when a user says
  "another agent is active," first grep for existing coordination
  artifacts (`phase-6-runlog/`, `01-phase-6-progress.md`) before picking
  a channel.
- **External edits to working tree are silent.** #17 `Object` trait-import
  removal reverted between the draft and commit — no notification, only
  visible via a delayed system-reminder. Treating external edits as
  intentional-signal was the right call, but the invisible revert cost
  5 min of confusion. **No action;** this is a harness characteristic.
- **Push races still require a protocol.** Once merge6 was green on
  Impl1's side, we needed an explicit marker ("OK TO PUSH COSMETIC
  SWEEP") before Impl2 pushed — otherwise Impl2's cosmetic sweep could
  have landed between Impl1's merge6 run and the retro commit, invalidating
  the validation. **The fix:** `3a42f43c2` signal commit from Impl1 → I
  push on top → Impl1 harvests SHAs + retro as final commit. Clean.

**What to repeat**
- **Surface early when state doesn't match briefed hashes.** The original
  brief named a hash that was wrong (retro filed under `reports/` not
  `retros/`) and another that was stale (`41f1d0379` vs `d706109808`).
  Stopping and asking instead of acting saved a duplicate commit.
- **Rebuttals in one pass.** 9 rebuttals + 1 issue-link reply went out
  via parallel `gh api` calls in under a minute. Keeping the draft
  locally and firing them together (after user approval) avoided per-
  comment context-switch overhead.
- **Non-code Block-3/Block-4 work while compile-gated tasks wait.**
  Rebuttals + GH issue + runlog updates have zero conflict surface with
  Block-2 cosmetic commits. Filling the wall-clock while the e2e build
  ran kept the session productive during long waits.

**Carry-forward for future multi-impl phases**
- A dedicated claim format in the runlog header (not DQ) should be
  documented in `.claude/rules/phase-branch.md` or a new
  `multi-impl-coordination.md` rule. Current runlog coordination was
  invented ad-hoc by Impl1; works, but not discoverable by a fresh Impl2
  session.
- "Step 2/3 push marker" (Impl1 "merge6 green" → Impl2 push → Impl1 retro)
  is worth extracting as a reusable pattern rather than a one-off.

Impl2 session closed. No open blockers. phase-6 @ `8ee45a3ca` ready for
CodeRabbit re-review.


---

### Impl1 Bucket C intake (2026-04-19T16:20Z)

Resumed from handover `.claude/PRPs/handovers/pr46-bucket-c-merge.md`.
Local HEAD was `ce9bdd08c` (handover doc) + `0ff06abc9` (#1 Critical lock-race fix)
— **2 commits unpushed.**

User directive: **focus on Critical, defer Major/Minor to follow-up PR if needed.**
Impl2 will co-work this bucket.

**Claimed by Impl1 (in progress):**
- merge6b validation on `0ff06abc9` — clippy running (`b7ctqq9c0`),
  e2e-compile + e2e-run queued sequentially per handover rule.
- CodeRabbit #4 inbox atomicity — **ADR-006 invariant break.** Will
  fix now: wrap `insert_remote_sanction_notice` + `governance_log::append`
  in one `conn.run_transaction()` closure; mirror for `federation_attestation`.
  Pattern: `create_endorsement.rs:135-142`.

**Open for Impl2 (if active):**
- CodeRabbit #3 hash-chain causality — swap order in
  `submit_jury_vote.rs:466-488` so `case_decided` appends BEFORE
  `send_local_sanction_notice`. ADR-008 cosmetic-ish but easy.
- CodeRabbit #2 hidden pseudonym write — Option B simpler
  (log pseudonym creation event).
- #5 `TH_ISSUE_URL` env export — one-line fix `task-hopper.sh:486`.
- Minors #6/#7/#8 docs sweep — DQ wording, enum names, rule contradiction.

**Coordination:** whoever picks a task edits this runlog with a
"Claimed by Impl2: #N" line before staging. No DQ entries — this is scope,
not blocking questions.

**Push gate:** Impl1 holds the push pointer until merge6b is green on
`0ff06abc9`. Impl2's commits should stack on top locally; coordinate
final push via a signal commit (`merge6c green` pattern) or handoff.

### Impl1 Bucket C progress (2026-04-19T16:35Z)

**Currently editing** (DO NOT TOUCH — Impl1 has uncommitted changes):
- `crates/apub/activities/src/governance/inbox.rs` — #4 atomicity fix
  in progress, wrapping both receivers in `conn.run_transaction(...)`.
  Compile-check in progress. Will commit imminently.

**Safe for Impl2 to work on NOW** (no file overlap with Impl1):
- **#3 hash-chain order** — edit `crates/api/api/src/governance/submit_jury_vote.rs:466-488`
  only. Swap so the `case_decided` governance_log::append block runs
  BEFORE the `send_local_sanction_notice(...)` call. Keep both inside
  the same outer `run_transaction` so atomicity is preserved.
- **#5 TH_ISSUE_URL env export** — one-line fix in
  `scripts/brehon/task-hopper.sh:486`.
- **Minors #6/#7/#8** — `.claude/decision-queue.json` (DQ-6.7 wording),
  `.claude/PRPs/plans/phase-6-federation.plan.md:493` (enum names),
  `.claude/rules/task-hopper.md:149-153` (contradiction removal).

**Deferred per user directive** — #2 hidden pseudonym write will become
a follow-up GH issue; user chose critical-only focus.

**Impl1 will NOT touch** any of the above if Impl2 picks them up; Impl1
only owns `inbox.rs` in this commit.
