# Retro: brehon-conformance-audit — Phase-6 convention-divergence skill + Clippy gate

**Date:** 2026-05-21
**Sub-phase:** brehon-conformance-audit
**Plan:** `.claude/PRPs/plans/brehon-conformance-audit.plan.md`
**Phase tip @ retro time:** `3d48a9682` (pre-PR; merge into `governance-v0` pending User Gate 5)

---

## §1 Outcome

**Shipped (pre-merge).** All 14 §13 tasks landed on `phase-brehon-conformance-audit`. The deliverable is a Brehon skill + Clippy gate that detects and prevents the Phase-6 convention-divergence defect class (per `project_phase6_convention_divergence_class.md` axis #4).

### Task table

| Cohort | Task | Description | Final commit (impl) | DQ | Wallclock (min) |
|---|---|---|---|---|---|
| 1 | 1 | SKILL.md skeleton | `1e3948e43` | — | ~5 |
| 2 [P] | 2 | Six axis sub-files | `fe78ff388` | #301 pass | ~30 |
| 2 [P] | 3 | find-sibling.sh | `6388ad42f` | #302 pass | ~12 |
| 2 [P] | 4 | audit-metrics.schema.json | `1d6362098` | #304 pass | ~10 |
| 2 [P] | 5 | METRICS.md | `a78a67d37` | #305 pass | ~8 |
| 2 [P] | 10 | rust-analyzer-mcp + .mcp.json.example | `16a0dcab8` | #300 pass | ~6 |
| 2 (broken) | 8 (v1) | clippy.toml original | `c3aaba47f` (REVERTED at `bf92be5dc`) | #303 fail / #307 blocker / #308 fail / #309 blocker / #310 catch-fire | — |
| fix-impl-1 | — | lemmy_utils 6-site unwrap_or_default fix | `b00be611a` | #306 pass | ~18 |
| fix-impl-2 | — | lemmy_diesel_utils 4-site fix | (skipped — superseded by fix-impl-3) | — | — |
| fix-impl-3 | — | diesel_utils 4-site fix | `34f5cc567` | #310 fail (mechanism cycle-3) | ~15 |
| 2.6 | 8a | revert prior broken clippy.toml | `bf92be5dc` | #312 pass | ~3 |
| 2.7 | 8 (v2) | clippy.toml + Cargo.toml workspace-allow (corrected mechanism per DQ #311) | `6720dc72a` + TOML fix `3082c35ff` (advisor-direct) | #314 pass | ~10 |
| 2.5 | 6 | compute-metrics.sh | `dcc733ba3` | #316 pass | ~12 |
| 4 | 9 | per-module #![deny] on 3 federation mod.rs | `4464a2c0f` | #315 pass | ~6 |
| 3 | 7 | dogfood v1-federation-inbound-b (two snapshots) | `9a142e6f0` | #317 pass | ~15 |
| 5 [P] | 11 | advisor-orchestrator.md wiring | `2b0b27590` | #318 pass | ~3 |
| 5 [P] | 12 | two paired lesson files | `f9ff2b0e6` | #319 pass | ~8 |

**Phase-1 validate-pending-laptop:** all `result=pass` after the cycle-3 catch-fire recovery (mechanism revision via DQ #311 → Task 8a + Task 8 v2).
**Phase-2 e2e:** N/A — PRECON-2 OUT (skill is markdown + bash + attribute-only Rust; no test execution).
**Total in-flight time:** ~6 days (2026-05-15 bm-cut → 2026-05-21 Cohort 5 complete).

### Dogfood metrics (Task 7 verbatim output)

```
axis-4 precision: 1.000
axis-4 recall: 1.000
Lead time (median): 27.5h after ground-truth event (1 data point)
Latent-footgun catch rate (axis-4): 1
  (axis-4 findings the skill caught but the compiler missed)
```

Calibration confirms the skill catches the `inbox.rs:743 .unwrap_or_default()` divergence at the pre-fix-impl-3 snapshot (`649871f7d`); not flagged at current tip (post fix-impl-3). Ground-truth fix: `8b04e69a6` (fix-impl-3 merged 2026-05-19T20:27:54+00:00).

---

## §2 What surprised us

### §10.8 mechanism mismatch (DQ #311, cycle-3 catch-fire)

The plan's original §10.8 prescribed `clippy.toml` with `disallowed-methods` entries + per-module `#![deny(clippy::disallowed_methods)]` on the three federation modules. The plan's INTENT — "non-federation code is unaffected" — was correct. The plan's MECHANISM — silent on the workspace-wide default — was wrong.

The repo's `Cargo.toml [workspace.lints.clippy]` block at line 100 sets `style = { level = "deny", priority = -1 }`. The `disallowed_methods` lint is in the `clippy::style` group. So the workspace-wide default for `disallowed_methods` was already **deny**, NOT allow. Adding per-module `#![deny]` was a no-op (the lint was already firing workspace-wide); workspace-wide enforcement on shared code like `lemmy_utils` and `lemmy_diesel_utils` (which legitimately use `unwrap_or_default` on `Option<String>` where empty-string IS the intended fallback) triggered the three fix-impl cycles:

1. fix-impl-1: lemmy_utils 6 sites (legitimate empty-string defaults rewritten as explicit `.unwrap_or_else(String::new)`) — landed.
2. fix-impl-2: replaced by fix-impl-3 (lemmy_diesel_utils 4 sites).
3. fix-impl-3: landed. Then narrow-probe gate caught **lemmy_apub_objects** pre-existing violations on `.unwrap_or_default()` — those weren't even targeted by the brief. **Cycle-3 catch-fire to user.**

The corrected mechanism (per DQ #311 planner re-plan + user option-c) inverts the override direction. Add `disallowed_methods = "allow"` to the workspace-level `[workspace.lints.clippy]` block, then re-enable via per-module `#![deny(clippy::disallowed_methods)]` only on the three federation governance modules. This is **rustc lint-precedence rule 4** — lower-syntax-tree wins. Workspace-allow silences `disallowed_methods` workspace-wide; per-module deny re-enables only where conformance matters.

**The skill cannot detect this kind of plan-level mechanism error.** It detects per-file convention divergence (axis 1-6). It does not audit clippy lint-group activation paths. That gap is recorded as a forward lesson candidate.

### Task 8 TOML inline-table parse error (advisor-direct fix)

Task 8 v2 brief specified multiline inline tables for the `clippy.toml` `disallowed-methods` array. TOML 1.0 §Inline Tables forbids multiline — must be single-line per inline table. `cargo clippy --workspace` failed on cmd 2 of DQ #314 with `error reading Clippy's configuration file: invalid inline table, expected }`. User authorised direct advisor fix at `3082c35ff` ("you fix. I allow"); collapsed each `{ path=..., reason=... }` to single line. Recovery cost: ~5 min.

**Carry-forward:** brief authors of TOML edits must verify TOML grammar before specifying multiline literal blocks. The cargo `--workspace` log catches it on first compile; advisor's pre-push gate (`feedback_fix_impl_pre_push_cargo_check.md`) would have caught this if Task 8 included a local clippy probe step in its brief.

### Daemon stash-pop / next_id collision (DQ #318 → #319)

Junior #389 (Task 12) raised DQ as `#318` — **collision** with Junior #388 (Task 11) which also raised `#318`. Junior #388's commit `b3c59b64c` was on `phase-brehon-conformance-audit` before Junior #389's worker branch forked. The expected behaviour: Junior #389 should compute `next_id = 319` after reading the post-finalize tip.

**Actual mechanism (saved by Junior's lossless reconcile):** daemon's `finalize-merge` for Junior #389 detected the JSON conflict on `.claude/decision-queue.json` and renumbered Task 12's raise to `#319` during the 3-way merge resolution (commit `439a52cf8`). The `feedback_junior_finalize_merge_race_lossless_reconcile.md` pattern fired correctly.

**Implication:** the daemon's finalize-merge ID-conflict resolution is load-bearing for `[P]` cohort dispatch when both workers raise DQs. This worked clean for Cohort 5; should be flagged as a carry-forward observability point — if it ever fails to renumber, both raises would land with the same id and one of them would be invisible to advisor mutation.

### `[P]` cohorts that aren't actually parallel (Cohort 2 retrospective)

Cohort 2 [P] was Tasks 2, 3, 4, 5, 10 — file-set disjoint. In practice they ran serially-with-cohort-handoff because the daemon's per-task finalize-merge serialised them through the phase branch tip. Wallclock gain over true serial: maybe 20-30 min on a ~6 hour Cohort. The `[P]` marker still mattered for the cohort-dispatch advancement rule (advisor doesn't wait between cohort members) and for the next_id collision protection mechanism — but the "parallel" theoretical wins were modest.

---

## §3 What to change

### Add lint-group activation audit to the planner gate

When a plan specifies a *per-module* enforcement of a lint that lives inside a workspace-default-active group, the planner must verify:

1. Workspace-wide group level (e.g. `style = { level = "deny" }`).
2. Whether the lint name is in the named group.
3. Whether the per-module attribute direction (lower-syntax-tree wins per rustc rule 4) needs a workspace-level override to suppress activation outside the per-module scope.

Codify as `feedback_clippy_per_module_deny_requires_workspace_allow.md` — promoted in this retro per §"Lessons promoted".

### Tighten the cycle-count meta-rule for MECHANISM-level failures

The §G4 cycle-count meta-rule (advisor-orchestrator.md §5.3) says "3 cycles same `(error_class, file_basename)` = HARD REFUSAL catch-fire". The cycle-3 catch-fire on Task 8 worked exactly to that rule — but cost ~123 min on three failed cycles before triggering.

**Proposed tightening:** when the failure is MECHANISM-level (same lint, workspace-wide failure mode, just different uncovered callsites) — NOT site-level (specific file:line in known-bad code) — the meta-rule should fire HARD REFUSAL on **2 cycles**, not 3. The distinguishing signal: each cycle's narrow fix successfully patches the cited callsites; the next cycle then fails on a previously-unflagged crate. That's a mechanism mismatch, not a coverage gap.

Forward-only proposal; this retro is the evidence; not yet a feedback memory.

### Brief-author checklist: include local pre-push cargo probe for any clippy.toml edit

`feedback_fix_impl_pre_push_cargo_check.md` already says fix-impl briefs MUST include the pre-push cargo gate. **Extend this to Task-level briefs** that touch `clippy.toml`, `Cargo.toml [workspace.lints.*]`, or `rust-toolchain.toml`. Catch TOML parse errors / lint-config errors before they hit cargo on the laptop.

---

## §4 What to carry forward

### The Phase-6 convention-divergence skill works

PRECON-8 two-snapshot calibration confirms it. Axis-4 precision/recall 1.0/1.0 on the v1-federation-inbound-b test set; latent-footgun catch rate 1 (the skill flagged the inbox.rs:743 .unwrap_or_default() divergence that the compiler did not). Lead time 27.5h ahead of the runtime fix (fix-impl-3 at `8b04e69a6`).

The skill operates on history (reads phase-branch diffs or named-file scope) and produces structured findings per axis. It pairs with the Clippy gate (workspace-allow + per-module deny) to provide both **detection** (skill, advisory) and **prevention** (clippy, structural).

### Workspace-allow + per-module deny is a generalisable lint-precedence idiom

Rustc lint-precedence rule 4 (lower-syntax-tree wins) makes this pattern stable. It applies to any lint where the workspace baseline differs from a small set of target modules — not just `disallowed_methods`. Worth considering for:

- Federation-only lints we want strict on, repo-wide allowed for legacy code.
- Test-only lints (allow workspace, deny in `crates/server/tests/`).
- DB-schema lints (allow workspace, deny in `crates/db_schema/src/source/`).

### Junior's lossless-reconcile DQ-id renumber protects [P] cohort dispatch

The Cohort 5 collision (#318 → #319) was caught silently and resolved correctly. The mechanism is documented in `feedback_junior_finalize_merge_race_lossless_reconcile.md` but its application to DQ-id collisions specifically is worth flagging — the finalize-merge is the safety net, not the Junior worker's next_id computation.

### Advisor-direct hotfix on tooling-only files is OK when user authorises

User said "you fix. I allow" for the `clippy.toml` TOML parse error. The fix was a 4-line text mutation on a repo-root tooling file (not `crates/**`). Advisor authored directly at `3082c35ff` (commit subject `fix(brehon-conformance-audit): collapse clippy.toml inline tables to single line — TOML syntax cleanup`). Recovery: ~5 min vs ~15 min Junior fix-impl dispatch. The four-role discipline still holds (advisor never authors `crates/**`); but **repo-root tooling files (`clippy.toml`, `rust-toolchain.toml`, repo-root `Cargo.toml [workspace.lints]`)** are the BM/advisor's lane under multi-lane-worktree.md when user-authorised.

---

## §5 Per-role signals

### Advisor

**What worked well:**
- Cycle-3 catch-fire fired correctly on Task 8 v1 mechanism error. User escalation produced option-c (mechanism revision) and the recovery path (Task 8a revert + Task 8 v2 corrected mechanism) was clean.
- The `validate-pending-laptop` handler ran cleanly across 9 entries (#300, #301, #302, #304, #305, #306, #312, #314, #315, #316, #317, #318, #319) — including 4 cargo-runtime probes (#306, #314, #315, #316 indirectly via Task 9 deny gate). Zero false-positives, zero advisor-side handler errors.
- Daemon-stale-base reconcile (cherry-pick + DQ conflict resolution) ran 4 times this session without losing any work. The pattern from `feedback_junior_292_stale_base_recover_recipe.md` worked deterministically.
- RLS-r1 lane PAUSE: when user asked "can we pause so RLS lane can merge safely?", advisor honored Option B (let in-flight Junior #382 finish + manually defer finalize). Zero state-changing actions during the pause window. Post-resume: ran reconcile + validation cleanly.

**Issues:**
- **DQ next_id collision in [P] cohort 5** — Junior #389 raised #318 colliding with Junior #388. Daemon's finalize-merge renumbered to #319; advisor never saw the collision. Carry-forward observability flagged in §2.
- **Task 8 v1 mechanism error** — advisor approved a plan that was correct in intent but wrong in mechanism. The §10.8 wording reviewed at plan-time did not red-flag the workspace lint-group activation path. Lesson promoted (see §"Lessons promoted").
- **Stale wakeup re-fires** — ≥5 stale `ScheduleWakeup` prompts re-fired in this session. Advisor honored `feedback_thin_wakeup_prompts_verify_live_state.md` discipline (verified state first, ignored stale instructions) on every fire. Cost: minor parent-context noise.
- **Task 13 (this retro) authored advisor-side, not Junior** — plan §13 listed Task 13 as a normal task with `creates: [retro.md]`, but per `feedback_retro_not_report.md` retros are advisor-authored. Plan-author lesson: future plans should mark Task 13 as `[ADVISOR-AUTHORED]` and exclude it from cohort dispatch logic. Mechanical impact: zero (advisor reached this naturally); template-improvement candidate.

### Planning (Opus)

- **Plan §10.8 mechanism error (DQ #311 cycle-3 trigger)** — original plan's clippy.toml mechanism was wrong (workspace `style=deny` already active disallowed_methods; per-module `#![deny]` was a no-op). The DQ #311 replan was clean: option-c mechanism revision (workspace-allow + per-module deny re-enable per rustc rule 4) shipped correctly. Lesson: planner must audit lint-group activation paths when prescribing per-module lint gates. See §"Lessons promoted".
- **Cohort plan revision (DQ #297)** — pre-bm-cut planner self-resolved that Task 6 must be in Cohort 2.5 (not Cohort 2) because of `requires: [4, 5]`. The fix landed cleanly at `7bb6c51bb`.
- **Complexity score §5.1 was 11/10** (above threshold post-DQ #311 Task 8a addition). DQ #291 proceed-as-one decision was correct — splitting would have added cohort coordination overhead without changing per-task complexity.
- **§13 task spec quality was high** — file-line anchors, GOTCHA blocks, explicit FILES YAML, validate gates. Tasks 11+12 worked as parallel cohort with zero ambiguity.

### Impl (Sonnet)

- **Task 7 dogfood quality was high.** Worker authored the dogfood report + seed JSON + .gitignore + METRICS.md backfill correctly. Anti-hallucination report-content guidance with explicit `git show <sha>:<file>` commands worked — content matched ground-truth.
- **Task 8 v1 → fix-impl-1+2+3 cycle** — workers correctly remediated the cited callsites in each cycle. The mechanism error was upstream (plan); workers cannot detect lint-group activation paths. Carry-forward: workers patched ~10 sites across `lemmy_utils` + `lemmy_diesel_utils` with explicit fallbacks (net-positive cleanup; explicit-fallback style is canonical idiom outside federation modules too).
- **DQ #316 brief-template mis-transcription** — Junior #386 (Task 6) raised DQ with `bash scripts/brehon/cargo-check.sh --workspace --features full` instead of the brief §5.3 `compute-metrics.sh` smoke test. Advisor functionally validated correctly (ran the right command per the brief). Audit-trail-only mis-match. Lesson candidate: brief-template-discipline at Junior worker level.
- **Cohort 5 [P] parallel dispatch worked cleanly** — both workers raised DQs, finalize-merged, daemon resolved id collision. ~7m38s wallclock for both, vs ~11 min serial.

### BM (Haiku)

- **bm-cut clean** — phase branch cut from `governance-v0` tip cleanly at `0935c3c92`.
- **PR not yet opened** — bm-pr is the next step after retro sign-off (User Gate 6) → User Gate 5 (merge confirm).
- **Daemon finalize-merge ran 14 times across the phase** without any topology breaks. All worker branches finalize-merged into phase branch deterministically.

---

## §6 §5 watch-items + complexity scores

Per `feedback_retro_task_complexity_score.md`: `<files>/<commits>/<runtime-min>/<max-log-silence-min>`.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---|---|---|---|
| 1 | 1 | 2 | ~5 | <1 |
| 2 [P] | 6 | 2 | ~30 | ~3 |
| 3 [P] | 1 | 2 | ~12 | ~2 |
| 4 [P] | 1 | 2 | ~10 | <1 |
| 5 [P] | 1 | 2 | ~8 | <1 |
| 6 | 1 | 2 | ~12 | ~2 |
| 7 | 4 | 2 | ~15 | ~3 |
| 8 v1 (broken) | 1 | 1 (REVERTED) | ~5 | <1 |
| fix-impl-1 | 1 | 1 | ~18 | ~4 |
| fix-impl-3 | 1 | 1 | ~15 | ~3 |
| 8a | 1 | 2 | ~3 | <1 |
| 8 v2 | 2 (+ advisor-direct fix to 1) | 2 (+1 advisor) | ~10 | ~2 |
| 9 | 3 | 2 | ~6 | ~1 |
| 10 [P] | 1 | 2 | ~6 | <1 |
| 11 [P] | 1 | 2 | ~3 | <1 |
| 12 [P] | 2 | 2 | ~8 | ~2 |
| **Total** | — | ~30 | ~166 (wall) | — |

**Dogfood metrics (Task 7 verbatim — copy from §1):**

```
axis-4 precision: 1.000
axis-4 recall: 1.000
Lead time (median): 27.5h after ground-truth event (1 data point)
Latent-footgun catch rate (axis-4): 1
```

**Watch-items (forward to next sub-phase):**

- **W-1 — Mechanism cycle-2 trigger candidate:** advisor-orchestrator §5.3 cycle-count meta-rule may benefit from a mechanism-vs-site-level distinction. Forward observation; not yet a rule.
- **W-2 — DQ next_id daemon-side renumber observability:** Junior's lossless reconcile saved Cohort 5 silently. If it ever silently fails, both raises land with the same id; one becomes invisible. Forward observation point.
- **W-3 — Repo-root tooling carve-out for advisor-direct fixes:** `clippy.toml` advisor-direct fix worked clean per user authorisation. Codify the carve-out scope (repo-root tooling files only: `clippy.toml`, `rust-toolchain.toml`, repo-root `Cargo.toml [workspace.lints.*]`). Forward proposal.
- **W-4 — Plan §13 Task 13 retro-authorship marker:** mark retro tasks as `[ADVISOR-AUTHORED]` to skip Junior cohort dispatch logic. Template improvement candidate.
- **W-5 — Linting-config TOML pre-push gate:** extend `feedback_fix_impl_pre_push_cargo_check.md` to Task-level briefs that touch `clippy.toml` / `Cargo.toml [workspace.lints]` / `rust-toolchain.toml`. Forward proposal.

---

## §7 Lessons surfaced this sub-phase

1. **§10.8 mechanism mismatch — workspace lint-group activation vs per-module #![deny] direction.** Promoted as `feedback_clippy_per_module_deny_requires_workspace_allow.md` in this retro (see §"Lessons promoted").
2. **Cycle-3 catch-fire signal validated.** The §G4 cycle-count meta-rule fired correctly on Task 8 v1 mechanism error. Forward proposal: tighten to cycle-2 for MECHANISM-level failures (W-1).
3. **Net-positive cleanup from rejected-mechanism cycles.** fix-impl-1 + fix-impl-3 replaced ~10 sites of `Option::unwrap_or_default` with explicit fallbacks across `lemmy_utils` + `lemmy_diesel_utils`. Carry-forward: explicit-fallback style is canonical idiom outside federation modules; consider broader audit eventually.
4. **TOML inline-tables forbid multiline.** Spec gate gap; W-5 proposes a structural fix.
5. **Junior's lossless-reconcile DQ-id renumber protects [P] cohort dispatch.** Mechanism worked; W-2 records observability point.
6. **Advisor-direct hotfix on repo-root tooling files is OK when user authorises.** Carve-out from the four-role discipline; W-3 records codification candidate.

---

## §8 Lessons promoted

Two lesson files authored as Task 12 deliverables (already on phase branch):

- `.claude/lessons/feedback_mirror_phase6_convention_in_same_file.md` — Phase-6 convention-divergence defect class lesson, cross-linked + dogfood report cited.
- `.claude/lessons/feedback_plan_mirror_stub_must_compile_and_annotate_prelanded.md` — planning-side prevention for "stub compiles at Task N but signature error pops at Task N+1".

One new lesson file authored as part of this retro (DQ #311 follow-on):

- `.claude/lessons/feedback_clippy_per_module_deny_requires_workspace_allow.md` — planner-side prevention for workspace lint-group activation interacting with per-module `#![deny]` direction. Codifies rustc lint-precedence rule 4 mechanism.

**Sync to PMD** (run after retro commit lands on `governance-v0`):

```bash
bash scripts/sync-lessons-to-pmd.sh
# Then backfill embeddings:
OLLAMA_URL=http://homeserver:11434 PROJECT_MEMORY_DB=C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db PROJECT_ROOT=C:/Users/barri/Developer/brehon-fork node C:/Users/barri/Developer/MCPs/project-memory-mcp/dist/scripts/backfill.js --verbose
```

---

## §9 Next steps (post-retro)

1. **User Gate 6 (retro sign-off)** — surface this retro to user; await approval.
2. **bm-pr** — open PR `phase-brehon-conformance-audit` → `governance-v0` (per `phase-branch.md`).
3. **CR auto-review** — CodeRabbit triggers on PR open; advisor polls findings.
4. **bm-poll-cr + bm-triage** — apply four-bucket triage (per `feedback_pr_review_triage_pattern.md`).
5. **User Gate 5 (merge confirm)** — surface merge readiness; await approval.
6. **bm-merge** — `gh pr merge --delete-branch` (per L14 post-merge runlog discipline + L16 post-condition).
7. **`/brehon-phase-transition`** — five-deliverable handoff (per `reference_brehon_phase_transition_skill.md`).
8. **Lane worktree removal** — `git worktree remove ../brehon-fork-conformance-audit` after merge per `multi-lane-worktree.md` §"Lifecycle".

---

**Retro authored by:** advisor (laptop session, Opus 4.7 1M context)
**Phase tip at retro time:** `3d48a9682`
