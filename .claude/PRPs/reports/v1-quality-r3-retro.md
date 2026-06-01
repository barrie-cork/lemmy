# Retro — v1-quality-r3

**Phase:** v1-quality-r3 — EnvVarGuard C4 sweep (23 raw `set_var` sites)
**PR:** #169 merged `a23ac216f` at 2026-05-31T01:03:26Z
**Plan:** `.claude/PRPs/plans/v1-quality-r3.plan.md`

---

## TL;DR

Single-plane sweep of `e2e.rs`: 2 bootstrap sites annotated with SAFETY comments (option-b), 10 test-body sites wrapped with `EnvVarGuard` RAII guards. Two impl-tasks, clean e2e (126/0/5). Blocked at merge by an adr-compliance workflow infrastructure failure (runner picked up job then immediately crashed — not a real scan failure; local exit 0). Unblocked via owner-acknowledge bypass + `--admin` merge. One carry-forward lesson on audit look-back window sizing.

---

## §1. What surprised us

**Advisor:** The adr-compliance `mergeStateStatus: UNSTABLE` block was unexpected. The script exits 0 locally. The runner failure pattern (job allocated, zero steps executed, 3-second runtime) is consistent with GitHub Actions free-tier minutes exhaustion or a private-repo runner quota issue — not a code problem. Three re-trigger attempts all failed identically. The bypass path (owner-acknowledge comment + `--admin` merge) worked cleanly, but the investigation cost ~20 minutes.

**Planning:** (no planning subagent — plan was already authored). N/A.

**Impl-task:** Task 2 raised DQ `65b95cc574c8-001` (self-resolved log): the 400-character look-back window in the brief's audit script was too small to capture the full multi-line SAFETY comment blocks in task 1's output. The audit passed, but the impl agent noted the window should be wider for multi-line comment detection. Also raised a duplicate `validate-pending-laptop-e2e` entry (`65b95cc574c8-002`) when re-attempting after an apparent timeout — advisor resolved the duplicate by keeping `4b129a8b08f1-001` as authoritative.

**BM:** No surprises. bm-pr, bm-poll-cr, bm-triage, bm-merge all ran cleanly (bm-merge blocked correctly on UNSTABLE — correct behaviour per spec).

---

## §2. What to change

### 2.1 adr-compliance workflow runner reliability (DQ-level issue, carry forward)

The `workflow_dispatch` re-trigger path for adr-compliance does not work — jobs fail before steps execute. Root cause unclear (possible: private-repo minutes quota, `workflow_dispatch` from a feature branch not allowed, or runner resource limits). The `pull_request` event trigger (fired at PR open) DID execute correctly. The bypass path (owner-acknowledge + `--admin`) is functional but costs ~20 min investigation each time.

**Carry-forward action:** Add a note to the bm-merge brief template: if `mergeStateStatus: UNSTABLE` due to adr-compliance and the local scan exits 0, skip the re-trigger loop and go straight to acknowledge-comment + `--admin`. File a follow-up to investigate the workflow_dispatch issue separately.

### 2.2 Audit script look-back window (impl-task brief §4 constraint)

Brief audit scripts that validate multi-line comment blocks must use a look-back window sized to the block, not a fixed character count. The 400-char window was adequate for single-line safety comments but not for the 5-line SAFETY blocks task 1 produced.

**Carry-forward lesson:** `LESSON: Brief audit scripts with fixed look-back windows need to account for the full size of multi-line comment blocks above unsafe blocks.` — promoted from `bf3f7201d` LESSON trailer. Add to `.claude/lessons/feedback_envvarguard_audit_window.md`.

### 2.3 Duplicate validate-pending DQ entries from re-attempted tasks

Task 2 raised a second `validate-pending-laptop-e2e` DQ entry when it re-attempted after apparent timeout. The duplicate caused a merge-resolve commit (`08d7044ff`) and extra advisor time.

**Carry-forward:** impl-task brief §4 should cite `feedback_dq_v3_append_via_helper_script.md` to reinforce: check for existing `validate-pending` entries for this task before appending a new one.

---

## §3. What to carry forward

1. **Option-b (stay raw) for bootstrap sites** — when a guard would be dropped before the test body runs (fixture lifetime), SAFETY comment is the correct choice. This was the right call per `feedback_envvarguard_fixture_lifetime_footgun.md`. No change needed.

2. **Mode B works cleanly for single-plane quality sweeps** — the trunk→phase brief sync via daemon `git fetch origin governance-v0:governance-v0` pattern was smooth. No lane worktree needed for a 2-task sweep.

3. **bm-merge spec's UNSTABLE hard-block is correct** — BM correctly refused to merge. The problem was at the CI layer, not the BM. The spec's hard-block discipline saved us from merging a potentially broken CI state.

---

## §4. Per-role signals

### Advisor
- Brief sufficiency: ✓ both impl-task briefs were sufficient; no mid-task DQ blockers from impl
- DQ mid-task push: ✓ entries arrived promptly (duplicate noted in §2.3)
- User-gate interruptions: 2 (Gate 3 CR triage; Gate 5 merge confirm). Both clean one-round decisions
- adr-compliance block: ~20 min investigation + 3 failed re-triggers before bypass. See §2.1.
- bm-merge admin bypass: first use of `gh pr merge --admin` in this project — worked, documented

### Impl-task
- T1 (task 1, SAFETY comments): 1 file / 1 commit / 10 min / ~5 min max-silence. Clean.
- T2 (task 2, EnvVarGuard wraps): 1 file / 1 commit / 28 min / ~15 min max-silence. Clean, with one duplicate DQ.
- MIRROR refs: no MIRROR refs cited (pattern was mechanical; e2e.rs existing EnvVarGuard sites were the pattern)
- LESSON trailer harvested: `bf3f7201d` — audit window sizing

### BM-task
- bm-pr (#533): clean, no issues
- bm-poll-cr (#534): 1 finding (cr-1 low, Description check). Quick triage — done via PR body update
- bm-merge (#537): correctly blocked on UNSTABLE. Correct behavior per spec.
- CR triage: 0 critical/major; 1 low (template sections). No carry-forward findings.
- Runlog: BM wrote entries correctly; no attribution integrity issues

---

## §5. Quantified outcomes

| Task | Files | Commits | Runtime (min) | Max silence (min) | Complexity |
|------|-------|---------|---------------|-------------------|------------|
| T0 (harness audit) | — | — | 13 | — | 0/0/13/— |
| T1 (SAFETY comments) | 1 | 1 | 10 | ~5 | 1/1/10/5 |
| T2 (EnvVarGuard wraps) | 1 | 1 | 28 | ~15 | 1/1/28/15 |

All tasks within the comfortable envelope (no task >55 min runtime, no silence >40 min, no >8 files). Phase was correctly scoped for a 2-task sweep.

---

## §6. Lessons promoted this phase

| Signal | Source | Action |
|--------|--------|--------|
| Audit script look-back window must accommodate multi-line blocks | `bf3f7201d` LESSON trailer | Promote to `.claude/lessons/feedback_envvarguard_audit_window.md` |
| adr-compliance `workflow_dispatch` re-trigger doesn't work; use acknowledge + `--admin` for UNSTABLE blocks with clean local scan | This retro §2.1 | Note in bm-merge brief template §5 |
| Duplicate `validate-pending` DQ from re-attempted task — check for existing entry first | DQ `65b95cc574c8-002` incident | Add to impl-task brief §4 pre-push-DQ checklist |

---

## §7. Carry-forward to v1-quality-r3b

- Issue #167: `LemmyContext::create` — capture DB URL from context, not `LEMMY_DATABASE_URL` env
- `BREHON_DISABLE_*` sites (6+ locations in e2e.rs) — out of scope for C4 but worth a follow-on sweep if the pattern recurs
- adr-compliance `workflow_dispatch` investigation — separate from impl work
