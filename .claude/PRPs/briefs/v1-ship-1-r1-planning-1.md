# v1-ship-1-r1 planning brief (RE-PLAN against post-refactor-tier HEAD)

**Written**: 2026-05-16 by advisor session (laptop, brehon-fork CWD `C:/Users/barri/Developer/brehon-fork`, canonical checkout on `governance-v0` @ `27982bf22`).
**Subagent target**: `planning` (Opus 4.7 — see `.claude/agents/planning.md`).
**Worktree**: Junior cuts `junior/v1-ship-1-r1-planning-1` from `governance-v0` committed HEAD `27982bf22`. Plan file commits + pushes back to `governance-v0` at finalize.
**Authority anchor**: `.claude/PRPs/prds/v1-ship-readiness.prd.md` §7.1 (Phase v1-ship-1 — AGPL §13 surface). ADR-011 (AGPLv3 inherited + source-disclosure required) is the load-bearing constraint; this is its user-visible surfacing. v0-endpoint-coverage audit (`reports/v0-endpoint-coverage-2026-05-14.md` §8 Tier-1 gap #1) is the substrate evidence.

---

## 0. Why this is a RE-PLAN, not a fresh plan (read this first)

A complete plan already exists at **`.claude/PRPs/plans/v1-ship-1.plan.md`** (authored 2026-05-14, parked at the plan-approval gate, never approved/dispatched). Its **design is sound and UNCHANGED** — the AGPL §13 surface (extend `GetSiteResponse` with a `source_disclosure` block + add `GET /api/v4/source` returning the `AGPL-NOTICE.md` body) is exactly right, ADR-011 still drives it, no ADR contradiction, scope unchanged.

**What changed:** between 2026-05-14 and now, a 5-PR refactor tier merged (PR-4 #128, PR-5 #129, PR-6 #130, PR-2 #131, PR-1 #132). PR-1 #132 was a **LemmyResult-unification refactor of `crates/server/tests/e2e.rs`** that rewrote error-handling across the whole file AND added a Phase-1 migration-revert-list guard. The advisor has empirically confirmed the v1-ship-1 plan's `e2e.rs` MIRROR refs are now **all stale by 30-175 lines**:

| Plan §9/§10 cited anchor | Plan line | Actual location now | Drift |
|---|---|---|---|
| `report_to_modlog_golden_path` (golden-path outer-shape mirror) | `e2e.rs:2077-2275` | **`e2e.rs:2251`** | ~+174 lines |
| `all_mvp_endpoints_return_non_404` (in-process HTTP mirror) | `e2e.rs:3700-3880` | **`e2e.rs:3780`** | ~+80 lines |
| `governance_fixtures::bootstrap()` signature | `e2e.rs:767-809` | **`e2e.rs:801`** | ~+34 lines |
| `mod governance_fixtures` | `e2e.rs:88` | **`e2e.rs:118`** | ~+30 lines |
| e2e.rs total size | plan assumed ~14k | **14,862 lines** | grew |
| **DANGEROUS:** `bootstrap()` is no longer unique | plan says "the new test uses this verbatim" | there are now **TWO** `pub async fn bootstrap()` — `e2e.rs:801` (governance_fixtures) AND `e2e.rs:5665` (a sibling fixtures module added by the refactor tier) | ambiguity the 2026-05-14 plan could not anticipate |

The `crates/db_views/site/src/api.rs` and `crates/api/api_crud/src/site/read.rs` MIRROR refs (the DTO + handler-population anchors) are **less likely touched** (the refactor tier was test-focused) but are **UNVERIFIED** and MUST be re-confirmed against current HEAD — do not trust the 2026-05-14 line numbers.

**Your job:** produce a refreshed plan whose design is carried forward verbatim from `v1-ship-1.plan.md` but whose every `file:line` MIRROR ref, §11 caller-enumeration, §10 verbatim-code-block, and §15 DoD command is **re-derived and re-verified against `governance-v0` @ `27982bf22`**, and which **resolves the dual-`bootstrap()` ambiguity explicitly**.

---

## 1. Role + dispatch line

`[role:planning] v1-ship-1-r1 re-plan — AGPL §13 surface, refresh MIRROR refs post-refactor-tier`

The actual create-task description (single line, <100 chars, per `.claude/rules/advisor-orchestrator.md` Junior task description template):

```
[role:planning] v1-ship-1-r1 re-plan — see .claude/PRPs/briefs/v1-ship-1-r1-planning-1.md
```

Everything else lives in this brief.

---

## 2. Scope — what to produce

Produce **one plan file** at **`.claude/PRPs/plans/v1-ship-1-r1.plan.md`** for sub-phase **v1-ship-1** (the `-r1` suffix marks the re-plan; do NOT overwrite `v1-ship-1.plan.md` — retros/journals matter, per `.claude/rules/no-destructive-defaults.md` discipline; the original parked plan stays as the audit trail).

### 2.1 Carry forward verbatim (design is unchanged — do NOT re-litigate)

These are LOCKED from `v1-ship-1.plan.md`. Copy the design intent; only refresh the citations:

- **Two surfaces, both public, no auth, no rate-limit:**
  1. `GetSiteResponse` gains a `source_disclosure: SourceDisclosure` field (license SPDX, canonical repo URL, build-time-injected fork commit SHA, relative `/api/v4/source` disclosure URL). Populated in `read_site` from compile-time constants. Cache-safe (values are build-time-stable).
  2. New endpoint `GET /api/v4/source` returns `GetSourceResponse { notice: String, license: String }` where `notice` is the verbatim `AGPL-NOTICE.md` via `include_str!` at compile time (zero DB, zero auth).
- **Fork-commit SHA** read at compile time via `env!("BREHON_FORK_COMMIT")`, populated by a `build.rs` in the route-registering crate (shells `git rev-parse HEAD`, or reads `BREHON_FORK_COMMIT` env when CI/Docker sets it; missing env + missing `.git/` → emit `"unknown"` non-fatally).
- **No new crate, no new migration, no new ADR.** Field-add + handler-population are ONE atomic task (cache invariant breaks transiently otherwise — this was plan §13 Task 2's rationale; preserve it).
- **Scope boundary:** v1-ship-1 is ONLY the §13 surface. v1-ship-2 (per-endpoint e2e backfill) and v1-ship-3 (tactical polish) are SEPARATE later sub-phases per PRD §7.2/§7.3 — explicitly OUT of scope here. Release-pipeline / SBOM / signed binaries are OUT (moved to a future `v2-release-pipeline.prd.md` per PRD §2 + §3).
- **Acceptance (PRD §7.1):** fresh client (no auth) → `GET /api/v4/site` returns 200 with `source_disclosure.license == "AGPL-3.0"` and `source_disclosure.disclosure_url == "/api/v4/source"`; `GET /api/v4/source` resolves to the notice body. One e2e test asserts BOTH surfaces.

### 2.2 What you MUST re-derive (the actual re-plan work)

For EVERY `file:line` reference, verbatim code block, and caller-enumeration in the new plan:

1. **Open the cited file at current HEAD and READ the actual lines.** Do not transcribe the 2026-05-14 numbers. The plan's §9 "Mandatory reading" and §10 "Patterns to mirror" line ranges are ALL suspect.
2. **`crates/server/tests/e2e.rs` anchors (confirmed drifted):** locate the CURRENT line of every test/helper the plan mirrors — `report_to_modlog_golden_path`, `all_mvp_endpoints_return_non_404`, `governance_fixtures` module, `bootstrap()`. Use `grep -n` for the symbol, then READ the surrounding block to confirm the outer-shape (return type, `#[tokio::test]` flavor, `.map_err` bridge) the plan's §10.5/§10.6 mirror MUST reflect **post-LemmyResult-unification** (the refactor tier changed the error-bridge idiom — verify what the CURRENT canonical e2e error-shape is by reading a sibling test; cite it per `feedback_lemmy_error_no_std_error.md` case A/B/C enumeration).
3. **Resolve the dual-`bootstrap()` ambiguity (HARD — do not guess):** there are now two `pub async fn bootstrap()` (`e2e.rs:801` in `governance_fixtures`, and `e2e.rs:5665` in a sibling module). The plan's new e2e test must call exactly ONE. Determine which is the correct fixture for an HTTP-surface test that needs `(container, Data<LemmyContext>, db_url)` by reading BOTH signatures + their module context. State the chosen one explicitly in §10 with its current line + full signature, and add a §4 watchpoint citing the exact `mod` path + line so the impl-task cannot pick the wrong one. **If the two bootstraps are genuinely ambiguous for this use case (e.g. neither cleanly returns what an AGPL-surface HTTP test needs), file a `kind: "blocker"` DQ** — do not pick arbitrarily.
4. **`crates/db_views/site/src/api.rs` (DTO anchor — UNVERIFIED):** re-confirm the CURRENT line of the `GetSiteResponse` struct definition, its exact derive set + ts-rs `cfg_attr` shape, and its file-level imports. The plan's §10.1 verbatim block MUST match current HEAD byte-for-byte.
5. **`crates/api/api_crud/src/site/read.rs` (handler anchor — UNVERIFIED):** re-confirm the CURRENT lines of `get_site` + `read_site` + the `CacheLock<GetSiteResponse>` `LazyLock` + the single `GetSiteResponse { ... }` constructor site. Per `feedback_planner_enumerate_struct_callsites_for_addfield.md` (R9): **`grep -rn "GetSiteResponse" crates/` and enumerate EVERY constructor site** in §11 — confirm whether the refactor tier introduced any new construction site (the 2026-05-14 plan claimed exactly one at `read.rs:64-71`; re-verify that count holds).
6. **`crates/api/routes/src/lib.rs` (route-registration anchor — UNVERIFIED):** re-confirm the CURRENT lines of the site scope + the route-registration shape the new `/api/v4/source` route mirrors.
7. **§15 DoD commands:** the plan ships under **Shape G** (per the original §15). Re-confirm the workflow paths (`.github/workflows/cargo-validate-workspace.yml`, `cargo-test-e2e.yml`) still exist and the per-workflow DoD shape per `.claude/PRPs/templates/plan.template.md` §15.6 is current. Preserve the §15.4 e2e test-name filter but re-confirm the test fn name the plan proposes is unique (grep it in current e2e.rs to avoid a name collision with anything the refactor tier added).

### 2.3 Plan file deliverable shape

- **One file:** `.claude/PRPs/plans/v1-ship-1-r1.plan.md`.
- **Follow `.claude/PRPs/templates/plan.template.md` 20-section schema literally.** §16a Stories mandatory.
- **Shape G applies.** §15 DoD uses the per-workflow shape; inline cargo invocations forbidden in §15.
- **§5 complexity score breakdown table** per `feedback_complexity_score_pre_split.md`. The original scored **3/10** (Sonnet 4.6 target; split-DQ threshold `>8`). Re-derive the score against the refreshed §11/§13; it should remain low (single deliverable, ~60-line e2e append, one DTO field, one new small handler+route), but compute it honestly — if the dual-bootstrap resolution or a discovered new constructor site materially raises it, say so.
- **§16a Stories** — name composing §13 tasks + Shape-G checkpoint + Brief-Scope outputs. Likely **2 stories** (story 1: `source_disclosure` on `GetSiteResponse` discovery surface; story 2: `/api/v4/source` body + the e2e asserting both). `[P]` cohort marker is NOT applicable (single coherent deliverable; field-add + handler-population are atomic; the e2e depends on both — per the original plan §13's "not applicable" note; preserve unless your re-derivation finds genuine file-disjoint parallelism, which is unlikely).
- **§4 watchpoints** — every entry cites a SPECIFIC file:line / handler / `schema.rs` line at CURRENT HEAD (per `feedback_advisor_watchpoint_specificity.md`). The dual-`bootstrap()` resolution MUST be one watchpoint. The 6-watchpoint seed list is in §4.1 below.
- **§6 "Relationship to v1-ship-2 / v1-ship-3"** — confirm v1-ship-1 ships ONLY the §13 surface; e2e backfill + polish are later sub-phases.

**Commit only the plan file** (and any planner DQ entries). Plan-file commit pushes to `governance-v0` after the advisor's DoD smoke + watchpoint-specificity gates pass and the user approves (User Gate 1).

---

## 3. Required reading (in order, before drafting any plan section)

1. `.claude/agents/planning.md` — your subagent contract.
2. `.claude/PRPs/templates/plan.template.md` — canonical 20-section plan schema.
3. **`.claude/PRPs/plans/v1-ship-1.plan.md`** — the parked plan you are refreshing. Read in FULL. Its §1-§8 + §16-§20 design narrative carries forward; its §9/§10/§11/§13/§15 citations are what you re-derive. **Do not change the design; only refresh the evidence.**
4. `.claude/PRPs/prds/v1-ship-readiness.prd.md` — read in full, specifically §1 (ship-gate problem), §2 (IN/OUT scope), §3 (why this PRD shape), §4 (ADR-011 governs), §7.1 (Phase v1-ship-1 detail — the authority anchor), §7.2 + §7.3 (what is DEFERRED to later sub-phases — to keep your scope tight).
5. `reports/v0-endpoint-coverage-2026-05-14.md` §8 (Tier-1 gap #1) + §10 (sequencing recommendation) — the substrate evidence.
6. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — **ADR-011** (AGPLv3 + source-disclosure required — the load-bearing constraint), and confirm no other ADR contradicts a public unauthenticated `/api/v4/source` endpoint.
7. `AGPL-NOTICE.md` (repo root) — the verbatim body the new endpoint returns via `include_str!`. Confirm its current byte count + that the `include_str!` relative path the plan proposes resolves from the route-registering crate's root at CURRENT HEAD (the refactor tier did not touch repo-root files, but verify the path arithmetic).
8. **`crates/db_views/site/src/api.rs`** — locate the CURRENT `GetSiteResponse` definition + its derive/cfg-attr shape + file-level imports. (Plan §10.1 mirror — re-derive.)
9. **`crates/api/api_crud/src/site/read.rs`** — locate CURRENT `get_site` + `read_site` + the `CacheLock` + the constructor site(s). (Plan §10.2 mirror + §11 R9 enumeration — re-derive; `grep -rn "GetSiteResponse" crates/`.)
10. **`crates/api/routes/src/lib.rs`** — locate the CURRENT site scope + route-registration shape. (Plan §10.4 mirror — re-derive.)
11. **`crates/server/tests/e2e.rs`** — 14,862 lines at HEAD `27982bf22`. Locate (via `grep -n`, then READ the block) the CURRENT lines of: `mod governance_fixtures`, BOTH `pub async fn bootstrap()` (govern­ance_fixtures @ ~801 and the sibling @ ~5665), `report_to_modlog_golden_path` (~2251), `all_mvp_endpoints_return_non_404` (~3780). Read the CURRENT canonical e2e error-shape from a recent sibling test (post-LemmyResult-unification — the refactor tier's PR-1 #132 changed this) and cite it per `feedback_lemmy_error_no_std_error.md` case A/B/C. (Plan §10.5/§10.6 mirror + the dual-bootstrap resolution — re-derive.)
12. `crates/api/api_utils/src/context.rs` — confirm CURRENT `LemmyContext` definition + `settings()` accessor lines (plan §9 cited `12-59`; re-verify).
13. **Glob `.claude/lessons/feedback_*.md` and Read every file whose name keyword matches:** `lemmy_error_no_std_error`, `junior_worker_e2e_edit_hang`, `e2e_filter`, `async_pool_test_pattern`, `planner_enumerate_struct_callsites`, `insertform_default_propagation`, `complexity_score`, `pre_phase_dod`, `plan_dod_dry_run`, `advisor_watchpoint_specificity`, `read_canonical`, `features_full`, `features_full_p_crate`, `clippy_test_style`, `shape_g`, `validate_pending`, `dogfood`, `principles_not_rules`, `plan_baseline_self_reference`, `build_what_tests_exercise`, `test_target_compile_validation`, `verify_files_with_read`. That is the lessons-corpus discipline (per `.claude/rules/advisor-orchestrator.md` §2.4 — this is a planning brief so the impl-file-class table does not auto-fire, but these are the lessons the PLAN must bake into its §4/§10/§13).
14. **Glob `.claude/lessons/reference_*.md` and Read** any matching: `phase_branch`, `branch_manager`, `worktree`, `prp_commands`.
15. `.claude/PRPs/briefs/sl-e-planning-1.md` — exemplar planning brief (canonical brief shape, per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate).
16. `.github/workflows/cargo-validate-workspace.yml` + `cargo-test-e2e.yml` — confirm they exist + their current trigger shape (Shape-G §15 DoD references them by path + expected `conclusion`).
17. The 5 refactor-tier retros for context on WHAT changed in e2e.rs (read for awareness, do not transcribe): `.claude/PRPs/reports/refactor-tier-retro.md` + `.claude/PRPs/reports/refactor-tier-final-retro.md` (the latter documents the PR-1 #132 LemmyResult-unification + the cr-5/cr-6 migration-revert-list guard that inserted the drift).

---

## 4. Constraints (hard rules — violating any is a process breach)

### 4.1 Plan-content discipline

- **Design is LOCKED; only evidence is refreshed.** Do NOT change the AGPL §13 surface design, the two-endpoint shape, the cache-safety reasoning, the `build.rs` fork-commit approach, or the scope boundary. If you believe the design itself needs to change (not just its citations), STOP and file a `kind: "blocker"` DQ — that is an advisor/user decision, not a planner re-derivation.
- **Every `file:line` in the new plan MUST be verified by an actual Read at HEAD `27982bf22`.** Per `feedback_verify_files_with_read.md` + `pattern_verify_before_trusting_shell_output` — a `grep -n` hit is a pointer; READ the block to confirm the outer shape. Per `feedback_plan_baseline_self_reference.md`: cite anchors by symbol + ancestry, and where the plan needs a stable reference, prefer a grep-able symbol over a bare line number so the plan self-heals if minor drift recurs before dispatch.
- **Dual-`bootstrap()` resolution is mandatory and explicit** (§2.2 step 3). One §4 watchpoint MUST name the chosen `mod`-path + current line. Wrong-bootstrap selection by the impl-task would be a silent test-harness failure.
- **Shape G DoD shape (forward-only).** §15 per `plan.template.md` §15.6. Inline cargo forbidden.
- **§16a Stories mandatory.** ~2 stories. Each names composing §13 tasks + Shape-G checkpoint + Brief-Scope outputs.
- **§4 watchpoints cite specific file:line / handler / `schema.rs` line at CURRENT HEAD**, never abstract concepts (per `feedback_advisor_watchpoint_specificity.md`). **Seed list (≥6):**
  1. **Dual `bootstrap()`** — the new e2e test calls exactly one; cite the chosen `mod`+line; impl-task must not pick the other.
  2. **e2e canonical error-shape post-LemmyResult-unification** — cite the current sibling test + its case (A/B/C per `feedback_lemmy_error_no_std_error.md`); the new test's outer return + `?`-bridge MUST mirror it verbatim (the refactor tier changed this idiom).
  3. **`GetSiteResponse` constructor enumeration** — §11 lists every `GetSiteResponse { .. }` site from `grep -rn`; field-add must update all; cite each current line (R9 / `feedback_planner_enumerate_struct_callsites_for_addfield.md`).
  4. **`read_site` cache shape** — adding a field changes the cached `GetSiteResponse`; confirm the `CacheLock`/`LazyLock` current lines + that values stay build-time-stable (cache-safe); cite `read.rs:<current>`.
  5. **`include_str!` path arithmetic** — the relative path from the route-registering crate root to repo-root `AGPL-NOTICE.md` must resolve at compile time; cite the exact crate + the path; note "breaks the build loudly if the file moves" (acceptable failure mode).
  6. **e2e Edit-size discipline** — the new test is a ~60-line append (single anchor-Edit at file end); e2e.rs is 14,862 lines; per `feedback_junior_worker_e2e_edit_hang.md` keep it one append, well under the hang threshold. Cite the anchor pattern the impl-task appends after.
- **§5 complexity score breakdown table mandatory.** Re-derive; was 3/10.
- **R-rule inheritance** from prior retros (R1-R9) applies; R9 (struct-field-add caller enumeration) is load-bearing here.

### 4.2 Decision-queue discipline

- **Attribution integrity.** `from: "planner"` or `null`. NEVER `"advisor"`, `"user"`, `"impl-self-resolved"`, `"clarify"` (clarify is advisor-only).
- **Mid-task DQ commits push immediately** to the worker branch, not at finalize (per `.claude/rules/decision-queue.md` "Mid-task visibility").
- **Boundary-of-judgment — STOP and file a `kind: "blocker"` DQ rather than guess when:**
  - The two `bootstrap()` fns are genuinely ambiguous for an AGPL-surface HTTP test (neither cleanly returns the needed tuple) — §2.2 step 3.
  - `grep -rn "GetSiteResponse" crates/` reveals MORE than the one constructor site the original plan assumed (the refactor tier may have added one) — surface the count + locations, ask whether the field-add task scope changes.
  - The design itself (not just citations) appears to need a change — §4.1.
  - Any §9-cited file no longer exists / was renamed by the refactor tier.
- **Do NOT file `kind: "clarify"`** — that kind is advisor-only (the advisor runs `/brehon-clarify` on THIS brief after it commits; you only ever write `blocker` or `log`).

### 4.3 File ownership

- **Touch only:**
  - `.claude/PRPs/plans/v1-ship-1-r1.plan.md` (CREATE — new file, do NOT modify the parked `v1-ship-1.plan.md`).
  - `.claude/decision-queue.json` (APPEND new pending or planner-resolved entries; push immediately).
- **Do NOT edit anything under** `crates/`, `migrations/`, `tests/`, `docs/`, `.claude/PRPs/prds/`, `.claude/PRPs/reports/`, `.claude/agents/`, `.claude/rules/`, `.claude/commands/`, `.github/workflows/`, the parked `v1-ship-1.plan.md`, or any other plan file.
- **One commit at finalize:** `feat(plan): v1-ship-1-r1 sub-phase plan (MIRROR refs refreshed post-refactor-tier)`.

### 4.4 Schema-first discipline

- **Read the relevant struct/handler/route source at HEAD BEFORE designing §13.** Confirm at planning time: `GetSiteResponse` exists + its current derive shape; `read_site`/`get_site` exist + the constructor count; the route-registration scope exists. If any baseline assumption fails, `kind: "blocker"` DQ.

### 4.5 Cross-cutting from PMD-promoted patterns

- **`pattern_verify_before_trusting_shell_output`** — every grep hit confirmed by a direct Read of the block.
- **`pattern_test_against_reality_not_syntax`** — the plan's §10 verbatim blocks must match current-HEAD source exactly, not the 2026-05-14 transcription.
- **`pattern_cargo_feature_flag_propagation`** — §15 uses `--workspace --features full` only.
- **`feedback_read_canonical_before_writing_spec`** — mirror the CURRENT canonical e2e test + DTO patterns, not the stale ones.

### 4.6 Attribution integrity reminder

The only valid `answered_by` labels for a Junior planning subagent are `"planner"` or `null`. Never `"advisor"`, `"user"`, `"impl-self-resolved"`.

---

**Lean / advisor-side tip (not a constraint):** the entire value of this re-plan is *citation freshness + the dual-bootstrap resolution*. The design was already approved-in-principle (it's why the plan was parked at the gate, not rejected). A high-quality re-plan reads the actual current source, fixes every line number, picks the right `bootstrap()`, re-confirms the single `GetSiteResponse` constructor (or surfaces if there are now more), and changes nothing else. Resist the temptation to "improve" the design — that re-opens an approval decision that isn't yours to re-open.

A second observation: complexity should stay ~3/10. If your re-derivation pushes it materially higher, the most likely cause is a discovered second `GetSiteResponse` constructor site or a genuinely-ambiguous bootstrap — both are `kind: "blocker"` DQ moments, not silent absorptions into a bigger plan.

A third observation: this is the **hard ship-gate** — the fork ships out of AGPL compliance without it, from the first external byte. The plan's correctness matters more than its speed. A wrong MIRROR ref here costs an impl-task that reads the wrong code and a full validate cycle to discover it.

---

_Brief author: advisor session (laptop CWD `C:/Users/barri/Developer/brehon-fork`, canonical checkout on `governance-v0` @ `27982bf22`, 2026-05-16). Brief committed on `governance-v0` before the Junior planning task is queued. Per `.claude/rules/advisor-orchestrator.md` §3.3 clarify gate, the advisor runs `/brehon-clarify .claude/PRPs/briefs/v1-ship-1-r1-planning-1.md` after this brief commits + pushes; the planning task only queues after every clarify-DQ entry on this brief is resolved._
