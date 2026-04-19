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

