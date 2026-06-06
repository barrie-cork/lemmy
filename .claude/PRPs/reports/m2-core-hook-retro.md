# Retro — m2-core-hook (Governance Case Transition Hook, in-binary slice)

**Phase:** m2-core-hook · **PR:** #184 (merged @ `704a6ac45`) · **Trunk:** `governance-v0` @ `7468379b3`
**Authored:** 2026-06-06 (advisor, brehon-fork-m2 lane).
**Ran under:** four-role model (advisor + planning + impl + BM), Mode A dedicated lane worktree.

This retro covers the full lifecycle: Tasks 0–9 impl (prior sessions), the e2e-corruption diagnosis+fix (this session boundary), and the merge-back (verify → Linux gate → CR triage → merge). The factual record is the verify report (`m2-core-hook-verify.md`) + the debug report (`e2e-rs-botched-cherry-pick-corruption.md`); this interprets it.

---

## What surprised us

- **Advisor:** A **pre-existing trunk corruption** surfaced only when validating Task 8's e2e tests — `e2e.rs` had been uncompilable on `governance-v0` since 2026-06-05 13:43, two days before anyone noticed, because nothing had run the e2e suite against trunk in between. The botched BUG-1 cherry-pick (`6f4947b48`) re-injected ~2100 lines of pre-decomposition content into a file that had been decomposed to a 156-line `include!` host. **The corruption was invisible to `cargo check --workspace`** (the duplicate `async fn`s are in a test module that only the e2e target compiles) — it took a full e2e build to expose. Surprising that a green workspace check coexisted with an uncompilable test target for 2 days.
- **Advisor (merge-back):** The DQ merge-forward created a **silent duplicate** of entry `1dcd6a201003-002` — trunk carried a stale `pending` copy, the phase branch had the correct `resolved`/`pass` copy, and git's conflict region only covered one of them. A naive "keep HEAD" resolution would have left a phantom pending entry. The lossless-reconcile discipline (scan both arrays for dupes after resolving) caught it.
- **Advisor (CR):** **Two of CodeRabbit's higher-severity findings were wrong.** cr-7 (CR's only "Major") claimed `appeal_window_expiry.rs` "fires hooks for all cases regardless of update success — no error handling present" — but the loop uses `?` on every DB op, so it's false by construction. And cr-4's *valid* catch came with a *non-compiling* proposed fix (wrong type name + non-exhaustive match). The falsifiable-hypothesis discipline earned its keep twice in one triage.
- **Impl (carried, from commit trailers):** an earlier impl commit recorded `LESSON: format!("{:?}", variant) is the correct String conversion path` for `CaseTargetType` — which cr-4 then **reversed**: `Debug` repr is fine for a log line but is NOT a stability contract for an external wire payload. The "correct" path was producer-context-dependent, and the lesson over-generalized.
- **Planning:** the plan predates the §16a stories convention, so `/brehon-verify` could not run its mechanical FILES-YAML path — fell back to manual reconciliation. Worked (10-task phase, clean `feat(task N)` trail) but it's friction the §16a retrofit would remove.

## What to change

- **Advisor:** **A trunk compile-only gate would have caught the corruption at cherry-pick time, not 2 days later.** The retro of the diagnosis session already proposed a trunk compile-only CI check (gated against the Shape-G suspension). Re-surface as a concrete follow-up: a cheap `cargo check --workspace --tests` (or the e2e `--no-run`) on every `governance-v0` push would have turned the 2-day-latent corruption into an immediate red. **User-gated** (touches the Shape-G posture).
- **Planning:** **Retrofit §16a stories blocks onto M2-track plans** so `/brehon-verify` runs mechanically instead of by hand. Low priority but compounding — every future m2 sub-phase pays the manual-reconciliation tax otherwise.
- **Impl:** **Scope impl `LESSON:` trailers to their producer context.** The `format!("{:?}")` lesson was true for a log line, false for an external payload — but it was written as a general "correct path." Trailers that name a serialization choice should state the consumer (log vs wire vs DB).

## What to carry forward

- **Advisor:** The **falsifiable-hypothesis discipline for automated-reviewer claims** (`feedback_verify_automated_reviewer_claims_against_compiler.md`) is now 2-for-2 at catching confident-but-wrong CR findings on this PR alone. Keep compile-checking/code-reading every trait/type/control-flow claim before bucketing as fix-in-pr. The cr-4 case adds a corollary: **even a VALID finding can carry an INVALID fix** — verify the fix, not just the finding.
- **Advisor:** The **lossless DQ-merge reconcile** (dedupe-scan both arrays after resolving a `decision-queue.json` conflict) should be the default any time a merge-forward touches the DQ — the stale-pending-copy duplicate is a recurring shape, not a one-off.
- **Advisor:** **bm-merge UNSTABLE is expected, not a blocker** — the CR commit-status sits neutral (null) and there are no required checks on the unprotected `governance-v0`, so `--admin` bypass after a local-scan is correct (`feedback_bm_merge_unstable_admin_bypass.md` confirmed again).
- **Impl/BM:** cr-3 (carry-forward) — the `messaging_enabled` DB read on every hook call is a **bridge-side perf concern**; fold into the bridge-side plan's config-snapshot pass. Don't optimize it in the in-binary slice.

---

## Lessons promoted this phase

| Lesson | Status | Note |
|---|---|---|
| `feedback_cherry_pick_onto_restructured_file_reinjects_content.md` | **already authored** (PMD #831) | The corruption root-cause lesson; authored in the diagnosis session. Cited by the fix commit + debug report. |
| `LESSON: accumulate-then-fire pattern` (impl trailer) | skip (already idiomatic) | "collect tuples inside DB loop, fire HTTP after" — this is the existing `appeal_window_expiry`/`sponsor_liability_grace` pattern; no new lesson needed. |
| `LESSON: format!("{:?}", CaseTargetType) is the correct String path` (impl trailer) | **superseded by cr-4** | Reversed for external payloads — Debug repr is not a wire stability contract. The cr-4 fix commit body records the correction. No standalone lesson; the producer-context nuance is captured in this retro's "What to change" §Impl. |

---

## §5 Quantified outcomes

**CR triage (PR #184):** 8 findings, 0 critical, 0 blocking → 1 done (cr-4), 4 rebut (incl. Major cr-7, falsified), 2 wont-fix, 1 carry-forward. 1 of 8 actioned (12.5%) — high rebut rate driven by CR over-flagging correct-by-construction control flow.

**Gates:** 6 user gates — plan approval, clarify (prior sessions); CR triage (gate 3), merge confirm (gate 5), retro (gate 6, this). 0 catch-fires this session. 0 forbidden-window deferrals (laptop-local cargo, off-hours).

**Validation:** e2e 3/3 m2_ tests pass (Windows); `phase1_revert_list_matches_disk` pass; Linux `cargo check --workspace --features full` clean (16m55s); cr-4 re-validation: workspace check clean (10m07s) + e2e 3/3.

**DQ:** 10 m2 entries, all resolved/pass (7 validate-pending-laptop workspace checks + 1 log + 1 e2e + 1 Linux). 0 left pending.

**Per-task complexity** (impl ran in prior sessions; metrics from commit trail — files/commits): Tasks 1–3 (api_common DTO, bridge_notify refactor+fn) ~1–2 files each, 1 commit each; Tasks 4–5 (consts+wrapper) landed via Mode-B forward-merge; Tasks 6–7 (hook wiring) ~6 handler + 4 cron files, 1 commit each; Task 8 (e2e) 1 file, +corruption-fix follow-up. No task flagged >8 files / >55min / >40min-silence. The one anomaly was **not** a task — the cross-cutting corruption fix (this session) touched 2 files but required full-suite diagnosis.

---

## Outstanding (next session)

- **None blocking.** Phase shipped, trunk healthy.
- **Carry-forward follow-ups** (user-gated, non-urgent): (1) trunk compile-only CI gate vs Shape-G suspension; (2) §16a retrofit for M2-track plans; (3) cr-3 bridge-side perf (config-snapshot) — all noted above.
- Next: `/brehon-phase-transition` to close m2-core-hook + bootstrap the next M2 step (per MEMORY: M2-core unblocked; M2-late gated on OQ-ADR016-02/04).
