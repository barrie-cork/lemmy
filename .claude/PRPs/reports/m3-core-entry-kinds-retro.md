# Retro: m3-core-entry-kinds — M3 town-hall chair/mute entry-kind consts

**Date:** 2026-06-18
**Phase branch:** `phase-m3-core-entry-kinds` (merged to `governance-v0` via daemon finalize footgun — see §3.1)
**PR:** #200 (`feat(db_schema,api): M3 town-hall chair/mute entry-kind consts + shim + ROOM_KINDS 10→13`) — auto-closed MERGED
**Tasks:** 1 consolidated impl-task (Tasks 1+2) + Task 4 advisor registry reconcile + bm-cut + bm-pr
**Trunk landing:** `a5fc60a2d` (M3 code) on `origin/governance-v0`; registry reconcile `35b16907a`; lesson `e8a7117e3`

---

## §1 What shipped

Three new governance-log entry-kind string consts for M3 town halls (Phase 2), mirroring the M2 room-kinds zero-migration pattern (`entry_kind` is TEXT):

| # | Task | Files | Outcome |
|---|---|---|---|
| 1+2 | 3 consts + api shim re-exports + `ROOM_KINDS` 10→13 + doc-comment bumps | `crates/db_schema/src/source/governance/governance_log.rs`, `crates/api/api/src/governance/governance_log.rs` | ✅ |
| 4 | Registry section + count bump 69→72 (advisor meta-work, direct on gov-v0) | `.claude/rules/governance-log-entry-kind-registry.md` | ✅ |

Consts: `ENTRY_KIND_ROOM_CHAIR_TRANSFERRED` (`room_chair_transferred`), `ENTRY_KIND_ROOM_CHAIR_OVERRIDE` (`room_chair_override`), `ENTRY_KIND_ROOM_MUTE_ALL` (`room_mute_all`). Pre-landed consts — no emitters this phase; call sites land bridge-side in M3 phase 3 (`_CHAIR_*`) and phase 4 (`_MUTE_ALL`).

**Validation:** `cargo check --workspace --features full` EXIT 0 (Finished 17m58s, lemmy_server + shim + `ROOM_KINDS(13)` gate compile-accept). Level-5 registry invariants: db_schema distinct consts = 72, shim parity = 72, no duplicate literals, `ROOM_KINDS` len = 13.

---

## §2 What went well

**Impl was clean and exact.** Junior #689 produced the diff exactly to spec on the first cycle — 3 consts at the right insertion point, alphabetical shim re-exports, `ROOM_KINDS` extended correctly, both doc comments bumped. Zero §G4 fix cycles. The consolidate-Tasks-1+2-into-one-impl decision (const + shim must land together for parity) matched the m2-late-1 precedent and avoided a needless cohort barrier.

**Validation discipline held under a session boundary.** The `cargo check --workspace --features full` survived a `/compact` mid-build (rustc PIDs persisted). The completion was verified three ways — log `Finished` marker, `CARGO_EXIT_0` sentinel, AND `lemmy_server` compiling clean — rather than trusting the (known-unreliable) task-notification exit summary. No false-green.

**Registry reconcile arithmetic was checked, not assumed.** Task 4's 69→72 bump was verified by summing all addends (`19+4+2+1+6+1+5+7+9+1+10+2+2+3 = 72`) before commit, not eyeballed.

**Gate 1 worked as designed.** The DoD smoke test caught that the plan's Level-5 validation greps used `rg` (absent on laptop Bash + daemon) AND counted lines not consts — surfaced to user, who approved the advisor-owns-inline-fix path. The substituted `grep -oE`/`awk` invariants ran correctly.

---

## §3 What was rough

### 3.1 Daemon finalize-merge bypassed gates 3 + 5 (the dominant signal) — CATCH-FIRE

**Root cause:** The bm-pr Junior task (#690) opened PR #200 correctly and deferred merge. But the worker's `/bm-pr` work had merged `governance-v0` into `phase-m3-core-entry-kinds` and pushed the phase branch, putting phase code on the worker's gov-v0-based feature-branch HEAD. The daemon's generic finalize-merge step then `git merge --no-ff`'d that feature branch into `governance-v0` **AND pushed to origin successfully**, landing the un-reviewed M3 code on `origin/governance-v0`. GitHub auto-closed PR #200 as MERGED. **CodeRabbit review FAILED mid-flight** ("Pull request was closed or merged during review"). Gate 3 (CR triage) and gate 5 (merge confirm) were both bypassed.

**Why it's the WORSE variant of a known bug:** This is recurrence #3 of `feedback_junior_finalize_merges_bm_cut_branch.md` (PMD #351), but the prior 2 incidents (2026-05-16 bm-cut) stayed daemon-local — the CC v2.1.119 gate blocked their push. Here the finalize pushed to origin freely; that containment is gone.

**Severity:** High process-severity, low outcome-severity. The landed code was correct + cargo-validated + invariant-checked this session, and a 3-string-const diff is exactly the low-signal case where CR adds little. But the gate-bypass itself is the real damage and would be catastrophic on a content-bearing PR that actually needed review.

**Decision (user, gate-driven):** Accept + harden. Code accepted as-is (validated); 3 preventatives filed (§3.1 fix below).

**Fix applied:** Updated `feedback_junior_finalize_merges_bm_cut_branch.md` (committed `e8a7117e3`) with the recurrence + 3 preventatives:
1. bm-pr worker must NOT merge `governance-v0` into the phase branch and push it (add refusal to `.claude/commands/bm/bm-pr.md`).
2. Advisor post-bm-pr verification is now mandatory: confirm PR is OPEN (not MERGED) AND origin trunk has not absorbed the phase code, before advancing to `cr-wait`. Strengthen the `/auto-phase` `bm-pr-running → cr-wait` "Verify PR exists" step.
3. Structural: daemon finalize for `[role:bm-task]` should skip the merge-to-trunk entirely (same `executor.ts` finalize-skip family as the planned impl-task fix).

### 3.2 No e2e / no Linux gate (correct, noted for completeness)

This was a pure-logic const-only diff (no Cargo.toml/migrations/cfg/path-sep code), so the Phase-2 e2e gate (gate 4) and the Linux-compile gate correctly did NOT fire. Windows `--features full` green == Linux green for this diff class. No action — recording the correct skip so a future auditor doesn't read it as an omission.

---

## §4 Four-role signals

- **Advisor:** Clean orchestration through validation; the catch-fire was caught on the resume tick by the post-bm-pr PR-state check (PR not in `--state open` → investigated → found MERGED). The falsifiable-hypothesis discipline paid off: did NOT jump to "BM hard-refusal breach" — read the task log, confirmed the BM deferred merge and the **daemon finalize** was the actual vector. `retro_bypass` rate: N/A (no Stop-hook fail-opens this phase).
- **Planning:** Plan logic was sound (parity held 69/69/10 pre-impl); the only defect was its Level-5/Task-3 validation command-form (`rg` + line-counting). Mechanical, caught at gate 1. Planning role ran in a prior session.
- **Impl (Junior #689):** Exact, first-cycle, zero fix loops. MIRROR discipline (m2-late-b-actor block) followed verbatim. Wrote the validate-pending-laptop DQ and STOPPED per NO-CARGO-ON-ELITEDESK.
- **BM (Junior #690):** The bm-pr verb itself was correct (opened non-draft PR, deferred merge). The fault was NOT the BM agent's reasoning — it was the daemon's generic finalize merging the worker branch. BM boundary held; daemon infrastructure did not.

---

## §5 Per-task complexity

| Task | Files | Commits | Runtime | Max log silence | Notes |
|---|---|---|---|---|---|
| impl 1+2 (#689) | 2 | 1 (+1 DQ) | ~4 min | <1 min | Trivial mechanical mirror; clean first cycle |
| Task 4 registry (advisor) | 1 | 1 | ~3 min | n/a | Direct-on-trunk meta-work; arithmetic verified |
| bm-cut (#688) | — | 1 | ~2 min | <1 min | Clean |
| bm-pr (#690) | — | 1 (PR + runlog) | ~4 min | <1 min | PR opened OK; daemon finalize footgun fired post-task |

Phase complexity: **LOW** (const-only SCHEMA). Wall-clock dominated by the 18-min cargo validation, not by reasoning.

---

## §6 Carry-forward actions

1. **(structural, cross-phase)** Implement the daemon finalize-skip for `[role:bm-task]` — the root fix for the §3.1 recurrence class. Until then, the mandatory advisor post-bm-pr OPEN-PR check (preventative #2) is the guard. **Owner: user/daemon-infra. Trigger: next bm-pr dispatch must run the OPEN-PR check.**
2. **(this phase, done)** Lesson #351 updated + 3 preventatives filed.
3. **(advisor-orchestrator.md)** Strengthen the `/auto-phase` `bm-pr-running → cr-wait` transition wording from "Verify PR exists" to "Verify PR exists AND is OPEN AND origin trunk does not contain the phase tip." — fold into the next advisor-orchestrator edit.
