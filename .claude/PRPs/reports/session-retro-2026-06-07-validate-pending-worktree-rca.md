# Session retro — 2026-06-07 — validate-pending-worktree-rca

**Harness:** claude-code
**Session window:** 2026-06-07 ~11:30 → ~12:10 UTC (~40 min)
**Branch at start:** `23ee5cb71` (`governance-v0`)
**Branch at end:** `1c9fdb7a2` (`governance-v0`)
**Files touched:** 6 (5 harness edits in commit `24d457f61` + 1 lesson in `1c9fdb7a2`)
**Commits:** 2 explicit (`24d457f61`, `1c9fdb7a2`); 0 auto. (A third, `578cf812d` /doc-this, is a concurrent session's — not this thread.)

## TL;DR

User asked to (1) create a Mode A worktree for `phase-m2-late-1` to unblock a stuck T1 validation, then (2) trace the RCA of that stuck validation and propose harness fixes. The load-bearing finding: the `validate-pending-laptop` handler spec *prescribed* a bare `git checkout origin/<branch>` in the canonical tree, which **directly contradicted** `multi-lane-worktree.md` hard refusal #1 — a harness self-contradiction, not an agent error. A `/compact` then silently reverted that checkout to `governance-v0`, producing 8 failed validation steps against the wrong tree. Top change (shipped this session): rewrite the handler to use a worktree (compact-immune) + per-command branch assertion, and carve out the contradiction in hard refusal #1. The generalizable lesson reached the PMD only because we explicitly checked — a commit `LESSON:` trailer alone would NOT have fed the RLS loop.

---

## What surprised us

- **The harness contradicted itself.** The handler ref (`advisor-validation.md`) said "checkout the phase branch in canonical"; the always-loaded rule (`multi-lane-worktree.md` #1) said "never checkout a phase branch in canonical." An agent obeying both was stuck. The original trace framed this as an agent error at step 3; it's really a spec defect. Surprising because both files are mature and have been read many times without the conflict being noticed — the on-demand ref and the always-on rule were never in context *together* at decision time.
- **`git checkout` is conversation-invisible and compact-non-durable.** Working-tree HEAD is not part of the summarized conversation state. A `/compact` reverted the checkout to `governance-v0` with zero signal; the resumed session "knew" (from the summary) it was on the phase branch while the working tree said otherwise. This is a general trap for *any* working-tree mutation that spans a compact, not just this handler.
- **The file-path PMD sync DOES reach the live HTTP daemon.** Per pmd-invariants #1's "HTTP topology supersedes env-var/sync" note, I expected `sync-lessons-to-pmd.sh` (a `sqlite3` CLI writer) to write a *separate* store the daemon never reads. Reality: the daemon reads the same `memory.db` the CLI writes — the sync inserted my lesson as #882, fully searchable. My defensive `memory_write` (#883) was therefore a redundant duplicate I had to prune. The invariants note overstated the split for the file-path case.
- **The local `phase-m2-late-1` branch was 4 commits behind origin** — and silently. Creating the worktree from it would have reproduced the very "missing migration" bug being diagnosed. Caught only by an explicit `rev-list HEAD..origin` check before `git worktree add`.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **[SHIPPED `24d457f61`]** Handler Sequence step 1 → worktree (Mode A lane / Mode B throwaway), never bare checkout; + per-command `HEAD==origin/<branch>` assertion; + Windows wrapper substitution table; + pre/post `__diesel_schema_migrations` snapshot. | wrong-branch validation becomes structurally impossible (worktree HEAD compact-immune) instead of detected-at-step-11 | major | 1× this session, 0× prior (new failure mode) |
| 2 | **[SHIPPED `24d457f61`]** `multi-lane-worktree.md` hard refusal #1 carves out the handler's throwaway-worktree path + notes compact non-durability + extends to `phase-m2-*`/`phase-*`. | the two rules stop contradicting; an agent can obey both | minor | 1× this session |
| 3 | **[SHIPPED `24d457f61`]** `session-start-multi-lane-check.sh`: branch-pattern fix (`m1/m2/m3` were silently skipped) + declared-vs-actual `lane_mode` drift check; `handover.md` makes `lane_mode: A\|B` a required frontmatter field. | the *originating* drift (Mode A declared, Mode B operated) gets a session-start WARN; M-track lanes stop being invisible to the hook | medium | 1× this session; the branch-pattern miss is a latent bug that affected every M-track lane since the V2→M rename |
| 4 | **[SHIPPED `1c9fdb7a2` + PMD #882]** New lesson `feedback_validation_uses_worktree_not_checkout.md` (where-it-runs axis) + synced to live PMD. | future validation/brief work hits it via `memory_search_hybrid` pre-queue gate | minor | the "lesson must pair with structural fix" discipline — fix without searchable lesson is half-done |
| 5 | **[PROPOSAL]** Amend pmd-invariants #1's HTTP-topology note: the file-path `memory.db` sync (`sync-lessons-to-pmd.sh`) DOES reach the live daemon (same file); only the *env-var* mechanism is superseded. Current wording implies the whole file-path approach writes a dead store. | stops future sessions from doing redundant `memory_write` after a sync (the #883 dupe I had to prune) | minor | 1× here; worth a targeted edit, not a lesson |
| 6 | **[PROPOSAL — separate impl-task]** S6 from the RCA: `diesel_ltree.patch` hunk #2 is line-anchored and rots when migrations add tables. Replace with table-name-keyed idempotent post-processing OR a build-time drift-detection codegen test. | the next migration that adds tables won't hit "error applying hunk #2" | medium | already an independent PMD lesson #881; orthogonal to this session's branch fix — belongs on schema tooling |

## What to carry forward

- **Verify the local branch tip against origin before `git worktree add`.** `rev-list HEAD..origin/<branch>` is ~1s and caught a 4-commit-stale local branch that would have reproduced the bug under investigation. Now a standing pre-worktree check.
- **Read the actual harness mechanics, don't trust the report's own fix list.** The trace file proposed 6 parallel gaps; reading the four real layers (DQ data / handler ref / always-on rule / wrapper scripts) collapsed them to ~3 root fixes + 1 orthogonal bug, and revealed the harness *contradiction* the trace had mis-attributed as agent error. The falsifiable-hypothesis discipline applied to a debug artifact, not just a DQ.
- **Run the lesson→PMD loop explicitly and verify searchability.** Don't assume a commit `LESSON:` trailer feeds RLS — it doesn't until harvested. `memory_search_hybrid` for a distinctive term after writing confirms the brief-injection path actually works.
- **Stage only your own files when the working tree has pre-existing changes.** Opening status had staged lessons + `schema.rs` + tools from a prior session; explicit `git add <paths>` + `git reset HEAD <pre-existing>` kept the commit clean. (Cross-session shared-`.git/` discipline.)
- **Surface-first lane status when ≥2 worktrees exist.** Did this at session start; flagged the stale `brehon-fork-m2rooms-a` worktree (m2-rooms-a closed) for cleanup.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Manual harness reading (advisor-validation.md, multi-lane, scripts) | 30 | 0 | high | reading the 4 real layers found the spec self-contradiction the trace missed; collapsed 6 proposed gaps → 3 roots |
| Pre-worktree `rev-list HEAD..origin` check | 15 | 0 | medium | caught 4-commit-stale local branch before it reproduced the bug |
| `memory_write` (MCP) | 0 | 5 | medium | redundant dupe #883 — the file-path sync had already written #882; cost a prune cycle |
| `sync-lessons-to-pmd.sh` (background) | 5 | 0 | medium | DID reach the live daemon (surprise — contradicted my read of pmd-invariants #1) |
| `memory_prune` (dry-run then real) | 3 | 0 | none | clean dedup of #883 + 13 expired qa-results |
| `memory_search_hybrid` verify | 2 | 0 | none | confirmed #882 searchable + ranked #2 for realistic query |
| session-retro skill | — | — | — | in progress (this file) |

## Complexity scores (heavy tasks only)

Per `feedback_retro_task_complexity_score.md`. Format: `<files>/<commits>/<runtime-min>/<max-log-silence-min>`.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Worktree create + Mode A bootstrap | 0 (config copies, no tracked edits) | 0 | 5 | 0 |
| 4-solution harness fix (S1+S2+S4b+S5) | 5 | 1 | 20 | 0 |
| Lesson author + PMD sync + dedup | 1 | 1 | 12 | 0 |

No task exceeded the >55min / >40min-silence / >8-files flags.

## Decisions to revisit

- **Stale `brehon-fork-m2rooms-a` worktree + local branch** (m2-rooms-a closed per MEMORY.md, PR #191 merged). Lifecycle step 3 says tear down at ship; it wasn't. Surfaced to user; cleanup pending user OK.
- **S6 (diesel_ltree.patch staleness)** — independent bug (PMD #881), should be a scoped schema-tooling impl-task, not folded into a handler change. Flag for the next m2-late validation cycle (it WILL bite when the sanction_* migration regenerates schema.rs).
- **Concurrent session activity** — the m2late lane tip advanced (`e7eedeee8`) and `/doc-this` landed on trunk during this session. No conflict with the `governance-v0` meta-edits, but a reminder that multi-session-on-shared-`.git/` is the live operating mode.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [x] Change #4: lesson `feedback_validation_uses_worktree_not_checkout.md` — **DONE** (committed `1c9fdb7a2`, PMD #882).
- [ ] Change #5: amend `pmd-invariants.md` #1 HTTP-topology note to clarify file-path sync DOES reach the live daemon (only env-var mechanism superseded). Minor edit; user to approve.
- [ ] Change #6 / S6: new schema-tooling impl-task to replace line-anchored `diesel_ltree.patch` with table-name-keyed post-processing or a drift-detection codegen test. Already PMD #881; needs a plan/brief, not a lesson.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase section omitted (no `/auto-phase`
invocation or auto-state mutation this session; leftover JSONs are prior-phase)._
