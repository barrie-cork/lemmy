---
name: L14 runlog-on-trunk-before-merge self-conflicts with bm-pr's phase-branch runlog entry
description: The L14 fix (commit a runlog entry to governance-v0 BEFORE `gh pr merge`) creates a guaranteed bm-runlog.md merge conflict whenever the earlier bm-pr step also appended a runlog entry on the phase branch. Both branches diverge on the same append-only file → PR goes DIRTY/CONFLICTING → merge impossible. Fix: write the runlog COMPLETE entry POST-merge (phase branch is gone after --delete-branch, so no conflict possible).
type: feedback
---

The **L14 fix** (`.claude/rules/auto-phase.md` invariant 7 + `.claude/commands/bm/bm-merge.md` "L14 fix" section) orders the bm-merge BM Junior to: `Edit .claude/runlog/bm-runlog.md → git add → git commit "chore(bm): merge PR #N — runlog entry" → git push origin governance-v0 → THEN gh pr merge`. The intent is a durable audit trail even if the merge errors mid-flight.

**The flaw:** the **bm-pr** step (earlier in the same sub-phase) ALSO appends a `## bm: PR opened — ...` entry to `.claude/runlog/bm-runlog.md`, but **on the phase branch** (commit `d0e52fdb4` in the incident). When bm-merge's L14 step then commits its `## bm: merge ...` entry to **governance-v0** (commit `32548e55f`) and runs `gh pr merge`, GitHub tries to merge the phase branch into governance-v0 — and `bm-runlog.md` has **diverged on both sides** from the common ancestor (`32f55344c`): the phase branch added the "PR opened" block, governance-v0 added the "bm: merge" block. The file conflicts → `mergeStateStatus: DIRTY`, `mergeable: CONFLICTING` → **the merge cannot complete.**

This is a **guaranteed** conflict on every bm-merge where bm-pr wrote a phase-branch runlog entry — not a flake. The L14 rule, as written, breaks the merge it is trying to make durable.

Confirmed: v1-ship-1-r2, 2026-05-18. bm-merge Junior #322 BLOCKED. `gh pr merge` failed verbatim: `X Pull request barrie-cork/lemmy#137 is not mergeable: the merge commit cannot be cleanly created.` Local `git merge --no-commit` showed `CONFLICT (content): Merge conflict in .claude/runlog/bm-runlog.md` — and ONLY that file (all code clean-merged). Cost: a full failed bm-merge cycle + ~30 min advisor inline recovery + a re-dispatch.

Compounding: a false durable record landed on governance-v0 (the `32548e55f` runlog block recorded `merge sha: TBD / remote branch deleted? yes / 2026-05-18T00:00:00Z` for a merge that never happened). The advisor had to correct it during the conflict-resolution union.

**The fix (proven — the corrected bm-merge-2 brief worked first try, Junior #323 ~2 min):**

Write the runlog COMPLETE entry **POST-merge**, not pre-merge. After `gh pr merge --delete-branch` succeeds, the phase branch is **gone** — there is no second branch to conflict with, so a `git checkout governance-v0 && Edit bm-runlog.md && commit && push` cannot conflict. The pre-merge L14 ordering is removed entirely.

**How to apply:**

- **bm-merge brief / `bm-merge.md` Phase 8:** the runlog write happens AFTER the successful `gh pr merge`, on governance-v0, committed as `chore(bm): merge PR #N complete — runlog COMPLETE entry`. Do NOT commit any runlog entry to governance-v0 BEFORE `gh pr merge`.
- **Belt-and-braces unchanged:** if the BM Junior skips the post-merge runlog commit (observed — Junior #323 did), the advisor authors a `docs(advisor): L14 belt-and-braces — runlog COMPLETE re-apply` block on governance-v0 with the *verified real merge sha* (per `.claude/rules/auto-phase.md` invariant 7, which still applies — only the timing moved from pre- to post-merge).
- **Stronger structural options (consider for the rule revision):** (a) add a `merge=union` driver in `.gitattributes` for `.claude/runlog/bm-runlog.md` — it is an append-only ledger, so union-merge is semantically correct and makes ALL runlog cross-branch conflicts impossible; (b) have bm-pr write its runlog entry on `governance-v0` (like bm-merge does) instead of the phase branch, so the two never diverge on a phase branch at all.
- **Files this lesson requires editing** (rule-doc change — surface at the retro user gate as ADR/process-affecting): `.claude/rules/auto-phase.md` (L14 invariant 7), `.claude/commands/bm/bm-merge.md` (the "L14 fix" section + Phase 8 ordering), the bm-merge brief template.

**Generalises to:** any two-step PR pipeline where an early step (PR-open) and a late step (pre-merge bookkeeping) both append to the same tracked file, one on the feature branch and one on the base branch. The base-branch write before merge guarantees a divergence conflict. Move all pre-merge bookkeeping to post-merge, or use a union merge driver for append-only ledgers, or keep all writes on one branch. See DQ #265 and `.claude/PRPs/reports/v1-ship-1-r2-retro.md` Action 1.
