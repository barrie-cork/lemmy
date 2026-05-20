# Session retro — 2026-05-20 — fed-in-b impl phase close

**Harness:** claude-code
**Session window:** 2026-05-19 ~12:00 UTC → 2026-05-20 ~10:31 UTC (~22 h wall, mostly long-poll idle; ~3-4 h active advisor work)
**Branch at start:** governance-v0 (canonical) + `phase-v1-federation-inbound-b` (lane, ~`8b04e69a6`)
**Branch at end:** `aa4caaf68` (phase-v1-federation-inbound-b) — Phase-2 e2e bg running, DQ #290 pending
**Files touched (worker-side aggregate):** ~12 crates files + DQ + briefs (gov-v0)
**Commits (this session window, gov-v0):** ~6 advisor briefs + 6 DQ-mutation commits; phase branch ~65 commits ahead of trunk

## TL;DR

v1-federation-inbound-b impl phase closed cleanly through 9 §13 tasks under user's "cap cohort ≤2 serial" OOM-mitigation directive — Cohort B (Tasks 5/6/7 [P]) dispatched one-at-a-time, not in parallel. Three fix-impl chains (HRTB lifetime, spurious `.into()`, items-after-statements + dead_code) resolved without escalation past §G4-allowlist-equivalent classification. Tasks 7/8/9 §15-green first try, validating cumulative GOTCHA-accumulation discipline. The session's most load-bearing finding: **stale `ScheduleWakeup` prompts re-fired ~7× across long-polls, each carrying a 300-500-line verbatim decision tree that went stale on stage completion** — already-recorded watch-item per `feedback_thin_wakeup_prompts_verify_live_state.md` recurred ≥4× this session alone. Top change proposal: enforce thin (1-2 line) wakeup prompts that delegate decision-tree consultation to rules read at fire time. Phase-2 e2e currently running locally (Phase-6 sanction_notice_round_trip + 5 new fixtures); marker poll deferred to next wakeup.

---

## What surprised us

- **Worker §2.4 pre-push cargo discipline paid off cleanly on Tasks 7/8/9.** Per `feedback_fix_impl_pre_push_cargo_check.md`, worker-side `cargo-check.bat --workspace --features full` before pushing caught regression risk locally; net result was three consecutive first-try §15 passes on Tasks 7/8/9. The §15 fail-rate dropped from 100% on Tasks 4/5/6 (each needed fix-impl) to 0% on Tasks 7/8/9 — a clear inflection point coinciding with the lesson landing in Brief §2.4.
- **HRTB lifetime fail at Task 5 was latent for Task 4** (`feedback_lemmy_error_no_std_error.md`-adjacent class). The wrap-sig in inbox.rs was authored at Task 4 but its lifetime correctness was **shielded by `#[expect(dead_code)]`** — the compiler never exercised the call-site discipline until Task 5 became the first caller. Lesson: when an "infra" task lands a function that no caller exercises, the dead_code suppression hides type-level errors that would have caught at write-time.
- **E0283 spurious `.into()` in `.ok_or_else()` at Task 6** (`feedback_lemmy_error_no_std_error.md` Case-B). The worker added a 4-line multi-line closure with `.into()` despite a working SAME-FILE sibling `.map_err` 4 lines above doing it correctly. Suggests the worker did not consult the immediate sibling when reaching for the canonical recipe — canonical-sibling-mirror discipline (read the working sibling FIRST, byte-identical) needs to be moved EARLIER in the brief §2 than the §G4 verbatim blockquote.
- **clippy::items-after-statements at Task 6 R2.** const declaration sat mid-function after let-statements; the §15 R2 clippy fail surfaced this. Both items-after-statements and the unfulfilled lint expectation on `rate_per_actor_counts` (Task 6 became the first caller of `rate_per_actor_counts`, falsifying its `#[expect(dead_code)]`) were §G4-allowlist-equivalent mechanical fixes — auto-queued via fix-impl-6 without user gate.
- **Stale wakeup polling overhead recurred ≥4× this session.** Each `ScheduleWakeup` prompt arriving with a 300-500-line verbatim decision tree wasted ~3-5k tokens parsing instructions for stages already completed (e.g. polling Task 7 markers after Task 7 was already merged). Already a watch-item per `feedback_thin_wakeup_prompts_verify_live_state.md` from v1-ship-1-r2; recurred multiple times here.
- **User "Cap cohort ≤2 serial" directive worked first-time on Cohort B.** Three potentially parallel `[P]` tasks dispatched serially — no parallel-dispatch race detected, no DQ id collisions, no daemon-local trunk staleness incidents (per the v1-fed-in-a daemon-local-trunk-stale class). Resource-budget OOM-mitigation correctly identified before-dispatch.
- **`grep -c <pattern> <file>` exits 1 when count is 0**, killing `&&` chains. Hit during §15 R7 grep-verification of new mod boundaries. Fixed inline with `|| echo "0"` fallback, but worth a generalized lesson — `grep -c` exit-1-on-zero-count is a recurring trap (recurrence ≥2 across recent sessions).

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Enforce thin wakeup prompts.** Update `.claude/skills/auto-phase/SKILL.md` (and the advisor self-discipline doc) — `ScheduleWakeup.prompt` must be ≤80 chars (wake-trigger + resume-stage only); decision logic lives in `.claude/rules/advisor-orchestrator.md` §3.1 stage-shape, read at fire time. Verify-then-act explicit: "check `mcp__junior-brehon__list_tasks` + `git fetch` + DQ read BEFORE any action." Already a watch-item, not yet applied. | Stops ~3-5k token waste per wakeup; eliminates stale-tree re-execution risk; ~4× wakeups this session alone | minor (1-line change to skill body + rule cross-ref) | ≥4× this session + ≥4× v1-ship-1-r2 = 8× across 2 phases — **promote now** |
| 2 | **Move canonical-sibling-mirror BEFORE §G4 verbatim blockquote in fix-impl briefs.** Update `.claude/PRPs/templates/impl-task-brief.template.md` so §2 ordering is: §2.1 sibling-mirror citation (file:line range of working SAME-FILE sibling) → §2.2 §G4 verbatim blockquote → §2.3 §G4 acceptance criteria. Today §G4 blockquote is §2.1; sibling-mirror only appears in §3 Required reading. Worker reads §2 first and sometimes misses §3. | Eliminates the worker-paraphrases-canonical-recipe-while-ignoring-sibling failure mode (fix-impl-5 was exactly this) | minor (template edit + 1-line policy in advisor-orchestrator.md §G4) | 1× this session (fix-impl-5) + 1× v1-ship-1 (per memory) = 2× — **promote** |
| 3 | **Add lesson on dead_code-shielded latent type errors.** Author `.claude/lessons/feedback_dead_code_shields_latent_type_errors.md` — when an "infra" function ships with `#[expect(dead_code, reason="callers wired by future tasks")]`, the compiler never exercises call-site HRTB / Send / Sync bounds until a caller exists. Counter-discipline: planner authoring a multi-task plan whose Task N adds infra and Task N+1 first-calls it MUST include a temporary unit test that calls the infra fn with the expected closure-shape, so the compiler-exercises the HRTB at Task N's §15. Strip the test in the same task as stripping `#[expect(dead_code)]`. | Catches HRTB / lifetime bounds at the task that authors the wrapper, not the next task. ~30 min savings × N future infra-then-caller chains | medium (new lesson + plan-template note) | 1× this session (fix-impl-4, Task 4→Task 5 HRTB chain) + at least 1× prior (v1-RT-r1 had a similar class per memory) = 2× — **promote** |
| 4 | **Generalize `grep -c` zero-count exit-1 trap to a lesson** OR a single-line shell helper `grep-count` wrapping the fallback. Helper variant: `scripts/brehon/grep-count.sh <pattern> <file>` echoes the count and always exits 0. Brief authors switch from `grep -c "X" file` to `bash scripts/brehon/grep-count.sh "X" file`. | Stops `&&` chain breakage in §15 grep-verification sweeps | minor (single-script helper + brief template note) | 1× this session + ≥1× prior across recent sessions per memory = 2× — **promote** |
| 5 | **Surface Phase-6 convention-divergence class as a sub-phase work item** (already user-deferred to retro at 2026-05-19). The fed-in-b new code in `crates/apub/activities/src/governance/inbox.rs` and `publish_*` siblings introduced wrap-sig and trait-impl shapes that diverge from Phase-6's pre-existing conventions in the same file. fix-impl-1 only patches compiler-caught sites per user directive 2026-05-19; the divergence-class audit is fed-in-b carry-forward to retro. Convert this into a tracked carry-forward item with a named "thorough product-grade interpretation" task — not a bug fix, a design alignment pass. | Closes the user-flagged thread; converts deferred concern to actionable plan slot | medium (new sub-phase task, planner-side) | 1× this session, plus standing user directive — **carry forward** |
| 6 | **Generalize audit-tooling into a Brehon audit skill** (already standing user directive). The conformance-audit pattern used informally this session — "does new code in file X match existing same-file conventions for trait-impl, error-bridge, type-shape" — recurred at Tasks 4/5/6/7. Propose `.claude/skills/brehon-conformance-audit/SKILL.md` OR a `security-auditor.md`-style subagent that runs at brief-author time against a target file + pattern type. | Catches same-file convention divergence at brief-author time, before worker writes contrary code | medium (new skill) | standing user directive — **carry forward** |

## What to carry forward

- **User cap-≤2-serial OOM-mitigation directive.** Reapply mechanically on any future cohort whose plan §5 complexity score >8 OR whose member-count >2. Don't second-guess the directive — Cohort B paid this dividend cleanly with zero parallel-dispatch incidents.
- **Brief §2.4 worker-side pre-push cargo validation.** Tasks 7/8/9 first-try §15-green is the proof. Keep §2.4 in every impl-task and fix-impl-task brief going forward; this is not optional.
- **Atomic raise-before-dispatch for DQ entries** (advisor-orchestrator.md §3.1). All ci-watcher-equivalent dispatches in this session committed+pushed the DQ entry BEFORE creating Junior task. Zero contract-violations. Continue.
- **Atomic DQ-mutate protocol** (fetch → read-fresh → mutate → verify → commit → push → re-verify). Used 6× this session (DQ #285/286/287/288/289 mutations + DQ #290 raise). Zero clobbers; zero "git add reported nothing" incidents. Continue.
- **Canonical-sibling-mirror reading discipline.** When applied (fix-impl-5 after-the-fact, Tasks 7/8/9 pre-emptively), it produced correct code first-try. Bake into brief §2 ordering per "What to change" #2.
- **§G4-allowlist-equivalent auto-queue (no AskUserQuestion) for mechanical recipes.** fix-impl-5 (spurious .into()) and fix-impl-6 (items-after-statements + dead_code-strip) shipped without user-gate friction. Conservative-by-design allowlist held — non-allowlist fix-impl-4 (HRTB) correctly went through user-gate.
- **Live-state verify-then-act before any wakeup-triggered action.** Even with stale wakeup prompts arriving, the explicit verify pattern (read TaskList + git fetch + DQ scan first) caught every stale-tree mismatch in this session before any wrong action shipped. Until wakeup prompts get thin (change #1), this discipline is the catch-net.
- **Lane-worktree CWD enforcement.** Lane-dedicated CWD `brehon-fork-fed-in-b` was used for all phase-branch operations; canonical `brehon-fork` for governance-v0 advisor briefs + DQ mutations. Zero cross-lane DQ writes; multi-lane-worktree.md §"Hard refusals" #2 held. Continue.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `mcp__junior-brehon__create_task` (9 impl + 3 fix-impl dispatches) | ~480 (8×60min equivalents of manual impl) | ~10 | low | All workers landed; no daemon-local-trunk-stale incidents under serial discipline |
| Atomic DQ mutate protocol (5× pass mutations + 1× raise) | ~15 (avoided clobber-recovery) | 0 | none | Zero "git add reported nothing"; protocol holds |
| AskUserQuestion (Task 5 HRTB recovery + user-gate-4 Phase-2 mode) | ~20 (correct architectural fork) | 0 | none | Clean single-letter "a" + clean "Local" pick |
| Canonical-sibling-mirror discipline (Tasks 7/8/9 pre-emptive) | ~90 (3 first-try §15 vs ~30min/fix-impl each) | 0 | medium-high | Inflection from fix-impl-rate 3/3 (Tasks 4/5/6) → 0/3 (Tasks 7/8/9) |
| §G4-allowlist-equivalent auto-queue (fix-impl-5, fix-impl-6) | ~15 (saved 2× user-gate friction) | 0 | none | Conservative allowlist held; right calls |
| ScheduleWakeup long-poll cadence | ~10 (kept compute idle during e2e/Junior runs) | ~25 (stale-tree re-parses) | high | Net positive but stale-tree wakeup prompts cost real tokens — change #1 |
| User cap-≤2-serial directive | ~30 (avoided likely OOM cascade on Cohort B) | 0 | none | Followed mechanically; zero parallel-dispatch incidents |
| `feedback_lemmy_error_no_std_error.md` Case-B recipe | ~12 (fix-impl-5 single-line replace, no debug round-trip) | 0 | none | §G4 verbatim blockquote worked; sibling-mirror would have prevented entirely |
| `feedback_junior_worker_e2e_edit_hang.md` (Task 9 TWO Edits ONLY) | ~30 (avoided watchdog hang on 15k-line e2e.rs) | 0 | none | 1 in-place + 1 APPEND landed first-try, no retries |
| post-task-retro stop-hook (5 fires) | ~5 (forced retro discipline) | ~10 (5× retro authoring) | none | Net positive; minor authoring overhead |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Task 4 BARRIER (wrap + helpers + trait + label-handler) | 8 | 1 | ~55 | ~25 |
| Task 5 (publish_sanction_notice) | 1 | 1 | ~22 | ~10 |
| Task 6 (publish_trust_attestation + per-actor override) | 1 | 1 | ~35 | ~18 |
| fix-impl-4 (HRTB <'a> + strip dead_code) | 1 | 1 | ~14 | ~8 |
| fix-impl-5 (drop spurious .into()) | 1 | 1 | ~12 | ~6 |
| fix-impl-6 (hoist const + strip dead_code) | 2 | 1 | ~18 | ~10 |
| Task 7 (publish_label) | 1 | 1 | ~20 | ~9 |
| Task 8 BARRIER (scheduled_tasks replay-cleanup cron) | 1 | 1 | ~10 | ~5 |
| Task 9 BARRIER (e2e.rs Phase-6 fixture + new module) | 1 | 1 | **~52** | **~28** |
| **Sub-phase median** | 1 | 1 | **20** | **10** |
| **Sub-phase outliers** | Task 4 (8 files) | — | Task 9 (52min, near 55min flag), Task 4 (55min, at flag) | Task 9 (28min), Task 4 (25min) |

**Outlier analysis:** Task 4 (8 files, 55min, 25min silence) was correctly flagged as BARRIER and was an infra-shipping task per plan; the file-count was inherent to the shape. Task 9 (1 file, 52min, 28min silence) was driven by `crates/server/tests/e2e.rs` size (~15k lines) — the 28-min silence window aligned with the worker reading the file's context before composing the two edits. Both within envelope; no planning bug.

## Decisions to revisit

- **Phase-6 convention-divergence class** (carry-forward to fed-in-b retro per user directive 2026-05-19). Convert to an explicit planner task or audit-skill scope.
- **Audit-tooling generalization to `.claude/skills/brehon-conformance-audit/`** (standing user directive). Scope: brief-author-time audit pass over `(target_file, new_code_shape, pattern_type)` against existing same-file conventions. Worth a clarify pass.
- **DQ #235 harness-gap reconsider.** Workers escalate via worktree-root `TASK<N>_*.json` files — this session relied on it once during Task 4 fix-impl chain. Whether to formalize this pattern in a Junior daemon update vs keep ad-hoc is worth a separate scope.
- **PMD MCP `.mcp.json` per-lane wiring** (per `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`). Lane worktree `brehon-fork-fed-in-b/.mcp.json` was confirmed pointing at canonical PMD this session; mechanism still doc-only — promote to a tracked SessionStart hook is on the v1-ship-1-r2 retro carry-forward list, not closed.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1 (thin wakeup prompts): update `.claude/skills/auto-phase/SKILL.md` body + cross-ref `feedback_thin_wakeup_prompts_verify_live_state.md`. Recurrence: 8× across 2 phases.
- [ ] Change #2 (canonical-sibling-mirror BEFORE §G4 blockquote): edit `.claude/PRPs/templates/impl-task-brief.template.md` §2 ordering + 1-line policy in `.claude/rules/advisor-orchestrator.md` §G4 classifier. Recurrence: 2× across 2 phases.
- [ ] Change #3 (dead_code-shields-latent-type-errors lesson): new `.claude/lessons/feedback_dead_code_shields_latent_type_errors.md` + plan-template GOTCHA. Recurrence: 2× across recent phases.
- [ ] Change #4 (grep-count helper or lesson): `scripts/brehon/grep-count.sh` wrapper OR new lesson. Recurrence: ≥2× across recent sessions.
- [ ] Change #5 (Phase-6 convention-divergence as carry-forward): tracked in fed-in-b retro at phase close, plus a follow-up sub-phase candidate. Standing user directive.
- [ ] Change #6 (Brehon conformance-audit skill): new `.claude/skills/brehon-conformance-audit/SKILL.md`. Standing user directive.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. No `/auto-phase` artifacts active
this session — section 10-category dormant._
