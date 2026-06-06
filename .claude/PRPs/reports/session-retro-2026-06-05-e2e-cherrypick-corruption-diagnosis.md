# Session retro — 2026-06-05 — e2e-cherrypick-corruption-diagnosis

**Harness:** claude-code
**Session window:** ~2026-06-05 22:00 → 22:10 UTC (resumed post-compact; investigation+diagnosis ~90 min wall across the thread)
**Branch at start:** `91c646514` (`phase-m2-core-hook`, worktree `brehon-fork-m2`)
**Branch at end:** `1a086e631` (`phase-m2-core-hook`)
**Files touched:** 3 (1 reverted: `crates/server/tests/e2e.rs`; 2 committed: debug report + lesson)
**Commits:** 1 explicit (`1a086e631` docs(debug)), 0 auto

## TL;DR

Resumed mid-flow to validate m2-core-hook Task 8's e2e. The build came back red, and what looked like a "Task 8 broke e2e" turned out to be a **pre-existing trunk corruption**: the botched BUG-1 cherry-pick `6f4947b48` (2026-06-05 13:43) re-injected ~2,100 lines of pre-decomposition test content into `crates/server/tests/e2e.rs` after that file had been decomposed into a 156-line `include!` host — leaving `governance-v0` e2e-uncompilable for ~13 hours. The most load-bearing finding: **my first fix was incomplete and I launched re-validation before scanning for the full error class** — the user's "search for similar bugs" prompt is what surfaced the other 9 E0428s. Top change: after any structural fix to a large file, run a duplicate-definition + full-compile gate **before** declaring done, never after.

---

## What surprised us

- **The corruption was 3 layers deep, not 1.** The first symptom (orphaned `}` / `unexpected closing delimiter`) looked like the whole bug. Fixing it revealed 9 more `E0428` duplicate-definition errors, then a 10th-order issue: BUG-1's migration revert-list entry had landed in the *dead duplicate* and never reached the canonical `governance.rs`. A single botched cherry-pick produced four distinct breakages across two files.
- **The "passing" closeout e2e was stale evidence.** The v1-closeout retro recorded a green Phase-7 e2e at 00:10 — but the corrupting cherry-pick landed at 13:43. The green was real *then* and meaningless *now*. Trusting a retro's green-check without checking its timestamp vs subsequent commits would have masked a 13-hour trunk breakage.
- **The background cargo wrapper reported "exit code 0" while cargo exited 101.** The task-notification's exit summary reflected the `echo` after `||`, not cargo. This is exactly the `cargo-output-capture.md` / `feedback_task_notification_exit_summary_unreliable.md` failure mode — and it fired twice this session. The in-log marker (`E2E_T8_V2_EXIT_NONZERO`) was the truth.
- **A naive duplicate-fn regex over-reported (18 dups) vs the compiler's 9.** The within-file repeats (`seed_person` ×6 in governance.rs) are legitimate per-test nested fixture modules. The compiler is the authority on what's a real duplicate-in-scope; my grep was a starting signal, not a verdict.
- **My own first fix was a false-confidence moment.** I deleted lines 110-893, confirmed brace-balance (170/170), declared it correct, and **launched re-validation** — without running the duplicate-definition scan that would have shown the build was still 9 errors from compiling. A structural fix that "balances braces" can still be 9 errors short.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **After any structural fix to a large/`include!`-host file, run a duplicate-definition scan + `cargo … --no-run` (or at least `--no-run`) BEFORE declaring the fix done and launching re-validation.** Add to the §G4 / fix-impl discipline: "balance check is necessary but not sufficient — confirm zero duplicate defs across the include! scope and a clean `--no-run` compile." | Prevents the false-confidence launch (re-ran a doomed ~8-min e2e build). Catches multi-class corruption in one pass instead of N. | minor (a grep + a `--no-run`) | 1× this session (my partial fix) + 1× prior (the original lines-110-893 advisor fix the same day) = **2×** |
| 2 | **Promote the cherry-pick-onto-restructured-file lesson** (already authored: `feedback_cherry_pick_onto_restructured_file_reinjects_content.md`). Add a one-line trigger to the `bm-merge`/merge discipline: any cherry-pick/merge touching a decomposed `include!` host → run a duplicate-fn scan post-apply. | Catches this corruption class at apply-time, not 13 hours later at the next e2e run. | minor (lesson already written; needs the merge-discipline pointer) | 1× this session + the sibling `feedback_merge_forward_e2e_conflict_default_to_governance.md` (same victim file, different mechanism) = **2× pattern on e2e.rs** |
| 3 | **Add a trunk CI gate: `cargo test --no-run --workspace --test e2e` on every push to `governance-v0`.** This is the cheapest possible detector — the corruption would have been caught at 13:43 instead of surfacing in an unrelated lane's validation. (Note: tension with `project_shape_g_suspended` "keep cargo-validate-workspace disabled" — propose a *compile-only, no-test-run* job, which is fast and doesn't re-trigger the May auto-on-push burn.) | Trunk never sits e2e-uncompilable silently. Surfaces re-injection / merge corruption immediately. | medium (one workflow job; reconcile with Shape-G-suspended policy) | 1× this session; would have prevented a 13-hour trunk breakage |
| 4 | **When resuming on a thin ScheduleWakeup prompt, verify the precondition the prompt's branch is conditioned on FIRST.** Held this session (the "if green: commit/mutate/dispatch" wakeup fired against a red build + a user "don't apply fixes" redirect; I verified live state and no-op'd). Reinforce `feedback_thin_wakeup_prompts_verify_live_state.md` with this concrete green-precondition example. | Prevents acting on a stale decision tree (would have committed a non-existent fix + mutated a DQ to false-pass). | minor (lesson exists; add example) | 1× this session + the lesson's prior basis = **2×** |

## What to carry forward

- **Git-archaeology by bisecting a structural invariant** (here: brace-balance across the extraction commit chain `5dfcb00b5 → f2a836ebb → 611f0157c → 0f3531c81 → 6f4947b48`) pinpointed the exact corrupting commit in minutes. Reusable whenever "when did this file break" matters.
- **Compiler-as-authority for duplicate/structural claims.** The grep gave a candidate list; the `E0428` error list gave the verdict. Cross-checking the two caught the 18-vs-9 over-report. Same discipline as `feedback_verify_automated_reviewer_claims_against_compiler.md`, applied to my own analysis.
- **Surface judgment-heavy scope growth to the user before applying.** The fix grew from "delete 110-893" → "restore + revert-list + enum bug across two files." I re-gated to the user at each scope jump rather than silently expanding the approved action. The user's two redirects ("investigate further" → "save for dev team") were the right calls and only possible because I surfaced.
- **Leave a clean tree for a handoff.** Reverted my partial fix so the dev team starts from pristine corrupt HEAD; documented that revert in the report. A half-fixed working tree is a worse handoff than a documented broken one.
- **Trust the in-log marker, not the task-notification exit code** for any backgrounded cargo wrapper. Held twice this session.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| git-archaeology (brace-balance bisect of extraction chain) | ~40 | 0 | medium | pinpointed `6f4947b48` as the sole corrupting commit; confirmed 0 legit changes to e2e.rs since `0f3531c81` |
| duplicate-fn scan across include! scope (Python) | ~30 | ~3 | high | surfaced all 9 E0428s the partial fix missed; over-reported 18 (vs compiler 9) — 3 min reconciling nested-module false positives |
| structural delimiter scan (braces+parens+brackets, strings stripped) | ~10 | 0 | low | confirmed `crates/` clean repo-wide; necessary-but-insufficient (missed the balanced-duplicate class) |
| AskUserQuestion (×2: fix scope, expanded scope) | ~15 | 0 | none | clean re-gating at each scope jump; both answers redirected correctly |
| partial fix #1 (delete 110-893) + launched re-validation | 0 | ~10 | high | false-confidence: declared done + ran a doomed ~8-min e2e build before scanning for the full error class → **change proposal #1** |
| ScheduleWakeup (fired post-investigation) | 0 | ~1 | low | stale prompt; verified live state and no-op'd correctly per `feedback_thin_wakeup_prompts_verify_live_state.md` → **change #4** |
| findings doc + lesson + PMD #831 + memory pointer | ~20 | 0 | none | dev team can plan the fix from a self-contained report without re-deriving |

## Complexity scores (heavy tasks only)

N/A — no Junior impl-tasks ran this session. This was a manual interactive investigation + diagnosis + documentation thread; the complexity metric (`feedback_retro_task_complexity_score.md`) measures Junior worker envelopes, which don't apply here.

## Decisions to revisit

- **Should the e2e.rs corruption fix land directly on `governance-v0` (trunk hotfix) or only via the m2 PR merge?** Trunk is e2e-uncompilable for *all* lanes until one of these happens. Surfaced to the user; the dev-team plan owns the call. (Report §"Blast radius" note 1.)
- **Change #3 (trunk compile-only CI gate) vs `project_shape_g_suspended`.** The policy is "keep cargo-validate-workspace disabled." A `--no-run` compile-only job is a narrower thing than the suspended full-test workflow — worth a clarify on whether it's in-scope or violates the suspension intent.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1 (structural-fix → duplicate-def + `--no-run` gate before declaring done): add to `feedback_cherry_pick_onto_restructured_file_reinjects_content.md` "Detection" section AND the §G4/fix-impl discipline in `advisor-orchestrator.md` §5.3. **2× this session/day.**
- [x] Change #2 (cherry-pick-onto-restructured-file lesson): **already promoted** — `feedback_cherry_pick_onto_restructured_file_reinjects_content.md` authored + committed `1a086e631` + synced to PMD #831 this session.
- [ ] Change #3 (trunk compile-only CI gate): new `.github/workflows/` job OR a documented decision in `project_shape_g_suspended_2026_05_16.md` if rejected. Needs user call (Shape-G-suspension tension).
- [ ] Change #4 (thin-wakeup green-precondition example): append the concrete example to `feedback_thin_wakeup_prompts_verify_live_state.md`. **2× (this + prior basis).**

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
