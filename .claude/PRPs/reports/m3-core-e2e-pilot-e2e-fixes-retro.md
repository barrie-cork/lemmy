# Retro — m3-core-e2e-pilot e2e-fixes (cycle-5, the 3 live-gate defects)

**Phase:** m3-core-e2e-pilot (M3-core Phase 6/6 — FINAL). Cycle-5 fix layer.
**Branch:** `phase-m3-core-e2e-pilot` @ `e6b250394`. Mode B.
**Window:** 2026-06-21 21:00 UTC → 2026-06-22 01:30 UTC (gate session → replan → cohort → validate → verify).
**Outcome:** all 3 e2e defects fixed + validated green; phase unblocked at e2e. Ready for bm-pr.

This is a retro, not a status report — signals + lessons per role, what to change next time.

## Per-task complexity (`files / commits / runtime-min / max-log-silence-min`)

| Task | Role | files | commits | runtime | notes |
|---|---|---|---|---|---|
| #768 planning e2e-fixes | planning (Opus) | 1 plan | 1 | ~12 | probe-grounded, complexity 3/10; found the api_common mirror + the seed_rtc_enabled_config precedent |
| #769 defect1 :3000 stub | impl (Sonnet) | 1 (compose) | 1 | ~4 | clean; caddy:2.8 no fallback needed |
| #770 defect3 mute fast-fail | impl (Sonnet) | 1 (test) | 1 | ~2 | clean; is_not_found arm retained |
| #771 defect2 rtc_enabled wire | impl (Sonnet) | 4 (2 crates + 2 bridge) | 1 | ~5 | atomic 4-file; byte-identical field both structs; obsolete env-guard correctly removed |

Zero fix-impl cycles. Zero catch-fires. Zero retries. Contrast: cycle-3 (the prior auto-fix loop) cost ~123 min over 3 cycles before a user-prompted re-plan.

## Advisor

**Signals:**
- **The replan-not-react discipline paid off measurably.** After cycle-3 catch-fire, the deliberate path — planning task → plan-approval gate → cohort — fixed all 3 defects in one pass, zero thrash. The cycle-count meta-rule (≥3 same-class fails = catch-fire) did its job: it forced the stop that made the clean replan possible.
- **Did not rubber-stamp an inbound "all fixes validated" framing.** The second session handed off "everything hinges on one validation run, all fixes in place." Verified against live test source first; the gate then exposed 3 genuinely new defects. The Tuwunel swap WORKED — it revealed the deeper layers precisely because it unblocked the previously-401'd path.
- **Falsifiable-hypothesis before flagging.** Twice avoided a false catch-fire: (1) suspected the defect2 worker left the `LIVEKIT_API_KEY` env-guard in — read the diff, the assert WAS removed, the refs are comments + a legit bridge-cred read; (2) defect2 e2e initially still R7-failed — traced it to stale DB state (not a code bug) before concluding. Both saved a wrong escalation.
- **Compile-proof-before-e2e ordering.** Ran the cheap `cargo check --workspace` gate before the ~30-min stack cycle. A wire-contract compile error would have been caught in 2m15s, not 30min in.

**Lessons (→ harvest):**
- **L1 (test-isolation, the cycle-5 finding):** a stale `bridge_room` row makes the provisioner's idempotency-skip fire BEFORE the new `rtc_enabled` gate, masking a correct fix as a failure. e2e suites that assert on persisted DB state need a fresh data dir per run. The advisor teardown (`down -v`) resets it; an in-suite reset would make per-run isolation independent of teardown discipline. **Candidate harness fix** — not blocking (the gate proved green on a clean DB), but worth a small follow-up.
- **L2 (verify-after-swap):** a swap/unblock fix (Conduit→Tuwunel) will surface downstream bugs the standalone check could not see (createRoom-standalone-200 ≠ full-flow-green). Plan for "the fix reveals the next layer" — the cycle-4→cycle-5 progression is the canonical example.
- **L3 (MEMORY.md index drift):** the index cites `feedback_build_what_tests_exercise.md` which has no backing file (planner flagged it in plan §19). The principle is sound (validate by observable behaviour — confirmed by reading `recording.rs:118-123`). ACTION: author the lesson OR correct the index line.

## Planning

**Signals:**
- **Probe-grounded, not guessed.** Each of the 3 options-per-defect was resolved on daemon-probe evidence (`:3000` curl-refused, the creds-gate code read, the timed-loop source). Picked the recommended option each time with cited rationale. Complexity 3/10 was accurate.
- **Found context the advisor brief missed:** the governance-side `CaseTransitionEvent` mirror (`api_common/governance.rs:889`), the already-shipped `seed_rtc_enabled_config` migration (so `rtc_enabled` was a real concept, not invented), and the test's env-guard-on-wrong-process bug (the test checked TEST-process env; the gate reads BRIDGE-process config).
- **Honest scope-boundary call on defect 2:** flagged the ≤2-crate Sonnet ceiling exception explicitly (4 files, atomic wire contract) and justified single-worker ownership to avoid a two-parallel-workers inconsistent-field-declaration risk. ADR-016 framing was correct (additive field to a reference integration, not a new ADR).

**Lessons:**
- The §10.x verbatim-block discipline (byte-identical field on both structs, pinned in the plan) is exactly what made the 4-file atomic change land in one worker pass with zero wire inconsistency. Keep for any future wire-contract field-add.

## Impl

**Signals:**
- All 3 workers shipped clean, one commit each, write-then-STOP honoured (no daemon cargo on the main crates). DoD greps all passed. The defect2 worker handled the trickiest part — removing the obsolete env-guard assert while keeping the load-bearing R7 assertion — correctly.
- The `[P]` cohort dispatched under the hard cap of 2 (2 then 1), not all 3 at once. No `.git/index.lock` contention.

**Lessons:**
- Pre-locating the verbatim Edit anchor (defect3 brief required `grep -n update_participant` first) avoided the e2e-edit-hang class. Worked.

## BM

Not yet exercised this cycle — bm-pr / CR triage / bm-merge are the next steps. (BM signals captured in the phase-close commit at merge time.)

## Conformance audit (§3.9.1)

N/A this cycle — the diff touches `crates/api/api_common`, `crates/api/api_utils`, and `services/bridge/**`; **no handler under `crates/api/api/src/governance/**` or `crates/apub/activities/src/governance/**`** changed, so there is no ADR-pinned-handler target for the conformance-audit skill. (defect2's `rtc_enabled` gate lives in `services/bridge/src/room_provisioner.rs`, not a Lemmy governance handler.)

## §3 actions / carry-forward

1. **L1 harness fix (candidate):** reset `services/bridge/.e2e-data` per e2e run (or assert-fresh in the suite) so per-run isolation doesn't depend on teardown discipline. Non-blocking.
2. **L3 index drift:** author `feedback_build_what_tests_exercise.md` or correct the MEMORY.md index line.
3. **DQ housekeeping:** 10 superseded older-cycle `validate-pending-laptop-*` entries to resolve/archive (their live results now in the 3 cycle-5 resolved/pass entries).
4. **Task 7 (D2 pilot runbook, NON-impl):** the D2 cross-instance + real-client paths (Element Call browser clients) are deferred to the pilot per DQ `3004b6625b83-001`; the runbook documents the manual D2 verification for the cross-instance <500ms timing + the 5 `todo!()` room_provisioning stubs.
