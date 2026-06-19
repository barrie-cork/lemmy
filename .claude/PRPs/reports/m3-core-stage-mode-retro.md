# Retro — m3-core-stage-mode (M3 town halls, Phase 3, bridge-side)

**Shipped:** 2026-06-19 · PR #202 merged @ `a8713dd75` · gov-v0 @ `592a1d3bf`
**Scope:** chair-controlled stage mode — dual-sourced chair seat, FIFO raised-hand queue, mic-passing via LiveKit publish-grants (not Matrix power-levels), 30s grace auto-revoke + next-promote, chair transfer/override, Q&A sidebar (Matrix text timeline), and the FIRST emission of `room_chair_transferred`/`room_chair_override` governance-log chain entries.
**Shape:** 6 impl tasks (Cohort A = Tasks 1+2 `[P]`; Tasks 3-6 serial) + 5 fix-impls + 1 CR fix-impl. Bridge Linux-only (`cargo-linux.sh`). DoD = deterministic unit tests (NOT e2e).

---

## What surprised us

- **Advisor:** The daemon's **long-name `origin/junior/*` remote-tracking refs were never auto-created** — `git rev-parse origin/<worker>` resolved empty for every finalize-merge this phase (4 occurrences: Tasks 5, 6, cr-4 ×2). The branch names exceed some ref-creation threshold on the daemon's fetch config. Worked around every time by fetching into an explicit local ref (`git fetch origin <worker>:refs/heads/_tmp<id>`). 4× = promote to lesson. Also: the **daemon-local-first finalize pattern** fired three separate times (bm-poll-cr #721, cr-4 #722 code merge, bm-merge #723) — the daemon merges into daemon-local `governance-v0`/phase WITHOUT pushing to origin, leaving origin behind. Each needed an advisor `ssh push`. For cr-4 specifically the daemon merged the *code* at the worker's pre-DQ-resolution tip, so the advisor had to merge the DQ-resolution commit on top.
- **Planning:** Nothing surprised — the plan's emit-intent→drain seam (sync `pending_emits` accumulation in Tasks 3-5, async `drain_emits` in Task 6) was exactly right: it kept the stage state machine's unit tests synchronous/deterministic while deferring the async controller wiring. The "reachable-but-empty-drain satisfies clippy dead_code" prediction held — removing the 3 `#[allow(dead_code)]` in Task 6 compiled clean because `drain_emits` became the live caller.
- **Impl:** The Task-4 grace path **already** revoked-before-promote (`on_grace_expired`), but the direct-promote path (`promote_next`) and `chair_override(ForcePromote)` did not. This asymmetry was invisible until CR's cr-4 finding — the marquee `fifo_mic_pass_in_sequence` test asserted only grant *order*, never the revoke, so 4 back-to-back promotes fired 4 grants with 0 revokes and the test stayed green. A genuine single-presenter-invariant gap that compiled and passed its own test.
- **BM:** Copilot was **quota-limited** on PR #202 (zero findings) — CodeRabbit carried the whole review. The 4 CR findings split cleanly: 1 real correctness gap (cr-4), 1 intentional scaffold (cr-3), 2 doc-lint (cr-1/cr-2).

## What to change

- **Advisor:** Promote the **daemon long-name refspec** workaround to a lesson now (4× this phase, recurring across phases). The fetch-into-explicit-local-ref recipe (`git fetch origin <worker>:refs/heads/_fin<id>`) should be the *default* for any finalize-merge touching a `junior/role-*` branch, not a fallback discovered after `origin/<worker>` resolves empty. Also worth a one-line watch: `git worktree add` with a `cd … &&`-prefixed relative target nested a worktree *inside* canonical (the `cd` didn't persist before the relative path resolved) — always pass an **absolute** target path to `git -C <canonical> worktree add`.
- **Planning:** Nothing — the seam design and the Cohort-A file-disjoint `[P]` split (crates-Windows Task 1 vs bridge-Linux Task 2) worked without coordination friction.
- **Impl:** A marquee test that asserts an *invariant* (single-presenter) must assert the **negative** too (no second publisher remains granted), not just the positive ordering. The cr-4 fix added exactly this — `windows(2)` revoke-before-grant assertions. Carry that pattern: ordering-only assertions on authz-shaped state machines are insufficient.
- **BM:** None — triage was clean; the four-bucket split mapped directly to action.

## What to carry forward

- **Advisor:** The **fix-impl-skips-validate-DQ** recurrence-watch (MEMORY.md, 2× on m3-core-infra) **held this phase** — cr-4 fix-impl #722 *did* write its `validate-pending-laptop-linux` DQ (`8c61e18ffcea-001`). The advisor still verified-via-DoD-grep before trusting it (the right discipline regardless). Keep verifying; don't downgrade the watch yet (1 clean phase ≠ resolved).
- **Planning:** The deterministic-unit-test DoD (no e2e for a state machine) was the right call — the `#[tokio::test(start_paused)]` virtual-time grace boundary ran in 0.04s vs a real 30s wait, and the whole stage suite is 10 fast tests. Carry this for any future timing-boundary logic.
- **Impl:** The emit-intent seam (sync accumulate / async drain) is a reusable pattern for "keep the core testable, defer the I/O" — worth a lesson if a second subsystem adopts it.
- **BM:** The CR PR flow caught a real MAJOR correctness gap (cr-4) that cargo + the task's own test both passed. This is precisely the value the PR/CR flow exists for on bridge code — the bridge-only-no-e2e-fragility assumption held (all findings were doc/correctness/contract, zero e2e-edit-hang class).

---

## 5. Quantified outcomes (per-task complexity)

`complexity: <files>/<commits>/<runtime-min>/<silence-min>` — silence-min not instrumented this phase (telemetry CSV not captured); marked `—`.

| Task | Commit | complexity | Note |
|---|---|---|---|
| 1 — RoomEventPayload chair fields (crates) | `20f8357ac` | 4/1/~12/— | Cohort A `[P]`; additive DTO, no migration |
| 2 — livekit can_publish grant (bridge) | `ee630ad71`+fix `9760441ed` | 2/2/~10/— | Cohort A `[P]`; +1 fix-impl (dead_code scaffold) |
| 3 — stage core: FIFO + mic-pass machine | `77731ea73`+fix `5e20a2933` | 3/2/15/— | marquee; +1 fix-impl (unwrap_or_default→error-propagate) |
| 4 — 30s grace auto-revoke boundary | `53b8c5612`+fix `8c9f0ec56`+`08147c230` | 1/3/~14/— | +2 fix-impls (tokio time/test-util feature gate; doc-overindent clippy) |
| 5 — room-event client + emit-intent seam | `efb9c9d9d`+fix `f8df98092` | 6/2/~16/— | +1 fix-impl (dead_code scaffold pending Task 6) |
| 6 — stage-mode provisioning + drain + e2e stub | `425eb6a25` | 5/1/~18/— | removed 3 dead_code allows (drain made emitter live) |
| cr-4 — single-presenter revoke-before-grant | `0258fef07` | 1/1/2.5/— | CR fix-in-PR; the only post-PR fix |

- **Bundling:** No task exceeded the >55min/>8-files carry-forward thresholds. Largest was Task 5 (6 files). The serial Tasks 3-6 each landed under ~18 min — well within the Sonnet+watchdog envelope. No bundling drift.
- **Fix-impl density:** 5 fix-impls across 6 tasks (Tasks 2,3,4×2,5) — all clippy/feature-gate scaffold issues (dead_code ahead of caller, tokio feature gate, doc-lint, unwrap_or_default), none a logic regression. The dead_code-scaffold-ahead-of-caller pattern recurred 3× (Tasks 2,5 + Task-6-removal) — inherent to building a state machine before its async drain exists; the seam design made it benign.
- **CR findings:** 4 total (CR only; Copilot quota-limited). 1 fixed (cr-4), 1 rebutted (cr-3 intentional scaffold), 2 doc-fixed (cr-1/cr-2). 0 carry-forward.

---

## Lessons promoted this phase

Harvested from phase commits + DQ resolved entries:

1. **`feedback_daemon_long_name_refspec_finalize.md` (NEW — promote, 4× recurrence)** — daemon's `origin/junior/<long-name>` remote-tracking refs are not auto-created on fetch; finalize-merge must fetch into an explicit local ref. Recipe: `git fetch origin <worker>:refs/heads/_fin<id>`. Default-not-fallback for `junior/role-*` finalize-merges.
2. **Watch (1×, not yet promoted):** `git worktree add` absolute-target discipline — a `cd …&&`-prefixed relative target nests the worktree inside canonical. Always pass absolute target to `git -C <canonical> worktree add`. Captured in eval ID 1034. Promote if it recurs.
3. **Confirmed-held:** `feedback_fix_impl_workers_skip_validate_pending_dq.md` — the recurrence-watch held (cr-4 fix wrote its DQ). Advisor verify-via-DoD-grep retained as the durable mitigation regardless.
4. **Carry-candidate (1×):** emit-intent→drain seam (sync accumulate / async drain) as a "keep-core-testable, defer-I/O" pattern. Promote to a lesson if a second subsystem adopts it.

---

## Verdict

✅ **6/6 impl tasks + cr-4 fix shipped, validated, merged.** Single-pass-equivalent (5 fix-impls were all mechanical clippy/feature-gate, 0 logic re-plans). The one MAJOR correctness finding (cr-4) was caught by CR — exactly the PR flow's purpose — fixed and tested in one ~2.5-min cycle. All ADR-015 (pseudonyms-only) + ADR-016 (metadata-only) invariants held. M3 Phase 3 complete.
