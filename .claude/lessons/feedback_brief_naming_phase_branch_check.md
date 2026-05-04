---
name: Brief naming source-of-truth is `git ls-tree origin/<phase-branch>`, not laptop `ls`
description: When picking the next-numbered filename for a phase-specific brief class (ci-watcher, fix-impl), `git ls-tree origin/<phase-branch> .claude/PRPs/briefs/` is the source-of-truth. Phase-only briefs are invisible from governance-v0 — picking by laptop `ls` collides with existing phase-only briefs and overwrites work.
type: feedback
---
**Rule:** when authoring a phase-specific brief (ci-watcher-N, fix-impl-N, impl-N for a phase that doesn't merge to trunk yet), the next-number lookup MUST be against `git ls-tree origin/<phase-branch> .claude/PRPs/briefs/`, not `ls .claude/PRPs/briefs/` on the laptop's checkout.

**Why this matters:**
- Brief files for phase-specific classes (ci-watcher-N especially) live ONLY on the phase branch — they're committed by the advisor on `phase-v1-XX` and never merged to `governance-v0` until the phase merges. The laptop's checkout of `governance-v0` doesn't see them.
- v1-SL-a's ci-watcher-12 collision incident: advisor authored `sl-a-ci-watcher-12.md` by counting briefs on the laptop's `governance-v0` checkout. The actual `phase-v1-SL-a` had ci-watcher-13 + ci-watcher-14 already authored; ci-watcher-12 already existed at `82bbee3b5` for DQ #131. Authoring "ci-watcher-12.md" overwrote the existing brief. Junior worker on phase branch read stale-on-phase content, mutated already-resolved DQ #131, wasted ~2 min runtime.
- Cost: 1 wasted Junior run + ~5 min advisor recovery (restore brief from git blob + re-author as ci-watcher-15).

**How to apply:**
- **Before authoring any phase-specific brief**, run `git ls-tree origin/phase-v1-<phase> .claude/PRPs/briefs/` and pick `max(N)+1`. The result includes all briefs committed on the phase branch (visible) and the laptop's view (already merged to trunk).
- **Quick check:** if you're authoring ci-watcher-N or fix-impl-N where the phase is not yet merged to `governance-v0`, ALWAYS use the git-ls-tree form. Laptop `ls` is correct only for briefs whose phase has merged.
- **Generalises:** any cross-branch artifact authoring where the destination branch may have content the laptop checkout doesn't. Same class as decision-queue.json mid-task push (an artifact lives on the phase branch and the laptop can't see it without `git fetch`).
- **Forward gate:** advisor brief-author checklist (proposed v1-SL-a §7 follow-up #6) — pre-write check `git ls-tree origin/<phase-branch> .claude/PRPs/briefs/`. Possibly automatable as a `.claude/hooks/` pre-write lint.

**Symptom to recognise:** a Junior worker on a phase branch reads a brief whose contents don't match what the advisor expected, OR a brief commit overwrites a prior brief's content silently. Catch via `git log --diff-filter=M -- .claude/PRPs/briefs/<file>` showing more than 1 commit (the advisor's brief should be a single create, never a modify).

**Retire when:** advisor brief-author tooling auto-checks against `origin/<phase-branch>` before write. Until then, mechanical pre-write check is the rule. Source: v1-SL-a retro §3.3.
