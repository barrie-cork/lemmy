# Session retro — 2026-05-16 — retro-harvest-skill-build

**Harness:** claude-code
**Session window:** ~19:30 UTC → ~22:30 UTC (~180 min) — continuous with the earlier pmd-backfill thread; this retro covers the skill-build arc that followed.
**Branch at start:** `50ec13495` (`governance-v0`)
**Branch at end:** `ea5e2ec7a` (`governance-v0`; `ea5e2ec7a` is a *concurrent-session* commit, not this session's)
**Files touched:** this session's commits = 2 (`b114937b8`, `50ec13495`); untracked new artifacts = retro-harvest skill + workspace + plan + 2 retros
**Commits:** 2 explicit (DQ #229 fix; sync-glob fix + earlier retro). The skill, eval workspace, plan, and this retro remain uncommitted.

## TL;DR

The session's spine: validate PMD → (out-of-scope DQ action, user-corrected) → session-retro → that retro surfaced a structural gap (retro proposals silently drop; nothing reads `.claude/PRPs/reports/` markdown) → user asked for a harvester skill → built `retro-harvest` through the full skill-creator eval loop (6 parallel runs, benchmark: 100% vs 58% with-skill vs baseline) → applied the one real defect fix (subagent-delegation had no inline fallback) → the skill's own first real run found a Tier-1 gap (5 infra lessons referenced live in MEMORY.md but never mirrored into the repo) → produced an executable remediation plan. The single most load-bearing finding: **a skill built to catch silently-dropped proposals immediately caught a silently-dropped class applied to the lesson corpus itself** — and the fix is a low-risk mirror, not authoring, because the content already exists in user-memory.

---

## What surprised us

- **The retro-harvest skill found a real bug on its first real run.** Not a contrived eval — all 3 with-skill runs independently discovered that `MEMORY.md` lines 63/66/71/139/173 point at `.claude/lessons/` files that were never committed (0 add-commits in git history). The skill's currency-oracle discipline ("a dangling index pointer is not artifact-at-HEAD proof") worked exactly as designed against the corpus that contains the skill.
- **The missing lessons aren't missing — they're misplaced.** Initial read of the retro finding implied 5 lessons needed *authoring* from retro prose (high-risk). Ground-truth check revealed all 5 exist as substantive content in the user-memory directory; they were just never mirrored into the repo-tracked `.claude/lessons/`. This flipped the remediation from "author 5 lessons" (judgement-heavy) to "copy + frontmatter-transform" (mechanical, reversible). The lesson: **verify where content lives before assuming it doesn't exist** — the retro-harvest skill's own §2-style ground-truth gate is what caught this.
- **The eval harness had no `Agent`/`Explore` tool.** All 6 test subagents had to run the skill's delegated Phases 1-2 inline. They succeeded — but only because each was a capable 1M-context model that improvised. This exposed that the SKILL.md *asserted* delegation with no fallback path, a latent correctness risk a constrained harness would have hit hard.
- **With-skill output was tighter than baseline, not just more correct.** Expected the skill to improve correctness; surprising that it also *shrank* output (68-82 lines vs baseline's 204) by constraining to the tier structure. Structure-as-compression was an unplanned benefit.
- **A concurrent advisor/meta session was active throughout.** `ea5e2ec7a` (DQ #240) and HEAD-advance mid-eval-run were not this session. The eval subagents correctly pinned currency triage to a fixed SHA and flagged the drift — the multi-lane discipline held without intervention.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Done this session** — added Delegation-policy block + inline-fallback to `retro-harvest` SKILL.md Phases 1-2 | Skill no longer improvises when subagent dispatch is absent; output contract preserved; token cost made explicit in report header | minor (done) | 1× this session (latent risk found via eval) |
| 2 | **Done this session** — added Phase-0 step 5 (phase-retro mtime vs git-author-date caveat) to `retro-harvest` | Date-window decisions no longer fooled by clone-artifact mtimes; all 3 eval runs hit this, only 1 caught it unprompted | minor (done) | 3× within this session's eval runs |
| 3 | Re-run the retro-harvest eval (iteration 2) **with `Agent`/`Explore` available** to get a true token delta — current +147k is inflated by forced-inline | Real cost number for the ad-hoc-vs-`weekly-review`-cadence decision | medium (one eval-loop iteration) | 1× (measurement gap, not a defect) |
| 4 | Fold `/retro-harvest` into a cadence (candidate: `weekly-review` step, OR a standalone monthly schedule) so proposals get a human decision within days, not never | Closes the root cause the skill addresses — the corpus had ~48 unactioned proposals + 5 dangling-index lessons because nothing periodically read it | medium (decide cadence + wire it) | structural — the gap recurs every session that writes a retro nobody re-reads |
| 5 | When a retro surfaces a "lesson X missing" finding, **always run the ground-truth check (does X exist in user-memory? in git history under any name?) before classifying remediation as authoring vs mirror** — codify in the retro-harvest currency-oracle section | Prevents over-scoping a mechanical mirror as judgement-heavy authoring (this session's near-miss) | minor (1 line in SKILL.md §3/§7) | 1× here + the broader `pattern_verify_before_trusting_shell_output` family (≥3 prior) |

## What to carry forward

- **Build-the-skill-then-let-it-run-on-itself is a high-signal validation.** The retro-harvest skill's first real target was the corpus it lives in, and it found a genuine Tier-1 gap. Dogfooding a meta-tool against its own domain surfaces real defects faster than synthetic evals.
- **Ground-truth-before-remediation.** The "5 lessons missing" finding could have become 5 authoring tasks. One `ls user-memory/ + git log --diff-filter=A` check reframed it as a mechanical mirror. Always locate where content actually lives before scoping the fix.
- **Plan-for-a-fresh-session discipline worked.** The mirror plan re-verifies ground truth at §5.1 and STOPS if a concurrent session already applied it — correct given a concurrent advisor was active all session. Self-contained plans with a re-verify gate survive the multi-lane reality.
- **One-line description + `disable-model-invocation` for explicit-only skills.** Matched the BM-command canonical pattern; no triggering-phrase padding wasted in the always-loaded catalogue. The right floor for a `/`-invoked-only skill.
- **The user's scope corrections are load-bearing signal.** The session opened with an out-of-scope DQ action (corrected), and that correction directly produced the `feedback_dq_hook_is_informational` lesson — which then showed up as one of the 5 dangling-index lessons the new skill found. The correction compounded into structural improvement.

---

## Three-signal scoring

Per `feedback_four_role_retro_signals.md`. Numbers defensible from the transcript, not exact.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/check-dq` (skill) | 0 | 12 | high | Triggered by hook context, not user ask → out-of-scope DQ #229 fix + push. User-corrected. Produced `feedback_dq_hook_is_informational` (net-positive via the correction, but the action itself was waste). |
| `/session-retro` (earlier, pmd thread) | 8 | 0 | none | Clean; surfaced the retro-proposal-drop gap that motivated the whole skill-build. High leverage. |
| Explore subagent (trace retro surfacing chain) | 15 | 0 | medium | Confirmed "proposals silently drop — no hook/cron/sweep reads PRPs/reports/" definitively; surprise that the gap was *total*, not partial. |
| Explore subagent (inventory 52 retros) | 20 | 0 | low | Compact inventory of ~48 unchecked items; correct delegation of read-heavy work. |
| Explore subagent (currency triage 7 retros) | 18 | 3 | low | Good STALE/LIVE calls; missed that this-session's glob fix was already shipped (minor — superseded by the with-skill eval run anyway). |
| `skill-creator` (skill authoring) | 40 | 5 | low | Strong scaffold for the eval loop. 5 min waste reconciling `aggregate_benchmark` schema (run-1/ subdir + summary.pass_rate not documented inline). |
| 6× eval subagents (3 with-skill + 3 baseline, parallel) | 60 | 10 | high | Parallel dispatch worked cleanly. Surprise: no `Agent`/`Explore` inside them forced inline Phases 1-2 (latent SKILL.md defect found). 10 min waste = the inline path wasn't documented so each improvised. |
| `aggregate_benchmark.py` | 5 | 8 | medium | +0.00 delta twice before discovering it reads `summary.pass_rate`/`failed`/`total` from a `run-1/` subdir. Schema undocumented in the skill body; reverse-engineered from source. |
| `generate_review.py` (viewer) | 3 | 6 | medium | cp1252 crash on box-drawing chars (the exact Windows-UTF-8 trap from this session's other thread). `--static` fallback worked. Harness friction, not skill defect. |

## Complexity scores (heavy tasks only)

Per `feedback_retro_task_complexity_score.md`. Format `<files>/<commits>/<runtime-min>/<max-log-silence-min>`.

| Task | Files | Commits | Runtime (min) | Max log silence (min) | Note |
|---|---:|---:|---:|---:|---|
| retro-harvest skill authoring + 2 citation-fix passes | 1 | 0 (uncommitted) | ~25 | ~3 | Self-contained; citation-verify caught 2 dangling lesson refs I'd written (dogfooded the skill's own discipline) |
| Full eval loop (evals.json → 6 parallel runs → grade → aggregate → viewer) | ~20 (workspace) | 0 | ~55 | ~9 | 6 subagents ran 388-527s each; parallel so wall-clock ≈ longest. Grading + aggregate schema friction ~13 min. |
| Subagent-fallback fix (SKILL.md edits ×4) | 1 | 0 (uncommitted) | ~12 | ~2 | Clean; DRY shared-policy block + per-phase pointers + header-template slot |
| Mirror-missing-infra-lessons plan authoring | 1 | 0 (uncommitted) | ~18 | ~3 | Ground-truth gathering (5 files × source-location verify) before drafting; self-contained plan |

No task exceeded the >55min / >40min-silence / >8-files watchdog-risk thresholds individually. The eval loop's ~55min was parallel-dispatched (6 subagents), not a single 55min worker — no watchdog risk.

## Decisions to revisit

- **Cadence for `/retro-harvest`** (What-to-change #4): ad-hoc only, or folded into `weekly-review`, or a standalone monthly `/schedule`? Best decided after iteration-2 gives a true token cost. Worth a dedicated decision.
- **The 2 genuinely-missing lessons** (`feedback_dq_collision_across_refs`, `feedback_planner_clippy_dryrun_implement_bodies`) flagged by some eval runs are NOT in user-memory — they need authoring from retro bodies. Separate plan, separate scope. Surface when the mirror plan is executed.
- **Commit hygiene:** the skill, eval workspace, plan, and 2 retros are all uncommitted. The eval workspace (`retro-harvest-workspace/`) is large — decide gitignore vs commit before the next session (workspace is iteration journal; arguably gitignored like `.claude/auto-state/`).

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] What-to-change #5 (ground-truth-before-remediation): add to `retro-harvest` SKILL.md §3/§7 as an explicit step — recurrence is this session + the broader `pattern_verify_before_trusting_shell_output` family (≥3 prior occurrences)
- [ ] What-to-change #4 (retro-harvest cadence): decide + wire — structural recurrence (every retro-writing session feeds the gap)
- [ ] `aggregate_benchmark.py` / `generate_review.py` Windows-friction: note in the retro-harvest workspace README so iteration 2 doesn't rediscover (cp1252 + run-1/ subdir schema) — recurrence is 2× this session + the cp1252 class is `pattern_cross_platform_divergences` (≥3 prior)
- [ ] Execute `.claude/PRPs/plans/mirror-missing-infra-lessons.plan.md` in a fresh session (the Tier-1 fix itself — user already requested this as a separate session)

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase section omitted —
session did not invoke `/auto-phase` or mutate auto-state (Step 0.5
trigger did not fire; the two auto-state JSONs are prior-session/other-lane)._
