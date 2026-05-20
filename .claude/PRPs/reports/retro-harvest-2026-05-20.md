# Retro harvest — 2026-05-20

**Scope:** 25 session-retros + 16 phase-retros | window last-21d (≥ 2026-04-29)
**Branch / HEAD:** `governance-v0` @ `410fedb44`
**Delegation:** Explore subagents (Phase 1 ×1, Phase 2 ×4 batches)
**Date oracle:** git-author-date override — 15 phase-retros shared a clone-artifact mtime (`2026-05-10 21:46`)
**Extracted:** ~78 open proposals + ~17 already-closed, across 41 retros
**Special emphasis:** Phase-6 convention-divergence findings → audit-agent capability gap (per user request)

---

## TL;DR

Of ~78 open proposals across 41 retros, **22 are Tier-1 LIVE** (actionable now, recurrence ≥2 or high impact, low cost). The single highest-leverage item is **building a `brehon-conformance-audit` skill that detects same-file convention divergence at brief-author time** — Phase-6 evidence proved `cargo check`-clean code can ship latent data-integrity footguns (Finding 6.1: `unwrap_or_default()` on a federation domain field would have persisted `source_instance = ''`), caught only by a bespoke read-only subagent the user asked for explicitly. **20 STALE items are sitting unchecked** (lessons shipped, rules already amended) and can be closed by ticking boxes in source retros. The Phase-6 emphasis section (below) lays out (a) what the audit agents should detect, (b) which agents to extend or create, (c) a six-axis divergence checklist ready to encode, and (d) **a self-improving evaluation loop** with concrete metrics that turn each audit run into training data for the next run.

---

## ★ Phase-6 spotlight — what the audit agents should detect (and how they self-improve)

> Per user directive 2026-05-19 (PMD `project_phase6_convention_divergence_class.md`):
> the Task-4 §15 failures + Finding 6.1 share a single defect class — **new federation-inbound code diverging from the already-shipped, already-compiling Phase-6 convention living in the SAME file**. The fixes shipped per-incident are reactive; a comprehensive, reusable audit capability is the goal.

### A. The defect class in one sentence

A new function/handler/helper is authored alongside a canonical Phase-6 sibling doing the same job in the same file/module, and the new code diverges across one or more of six axes — sometimes the compiler catches it, sometimes (worse) it compiles cleanly and ships a latent footgun.

### B. The six-axis divergence checklist (ready-to-encode)

Recorded verbatim in the 2026-05-19 conformance audit; ready for direct paste into a SKILL.md or agent prompt:

| Axis | What it means | Phase-6 instance | Detection method |
|---|---|---|---|
| **1. Conn-type / tx-boundary** | Does the new fn take `&mut DbConn<'_>` (starts tx via `.run_transaction`) or `&mut AsyncPgConnection` (must already be inside a tx)? | fix-impl-1 (#337): 2 helpers took `&mut AsyncPgConnection` but called `.run_transaction` (method on `&mut DbConn`); siblings 4 lines away took `&mut DbConn` | Grep file for `.run_transaction(` callers, check receiver type vs sibling receiver type |
| **2. Append reborrow shape** | In-tx `governance_log::append` calls must reborrow `&mut (&mut *conn).into()` exactly | Held (byte-conformant) | Pattern-grep against the canonical 4-token sequence |
| **3. Trait-bound completeness** | `#[async_trait]` methods with `Sync`-requiring bodies need `A: Sync` on the generic | fix-impl-1: `wrap_governance_inbound<A: GovernanceInboundActivity>` missing `+ Sync`; compile-caught | Compiler is the oracle, but planning-time MIRROR stub should compile-check |
| **4. Error idiom at trust boundary** | `.domain().ok_or_else(\|\| LemmyErrorType::Unknown(...))?` (hard-error) vs `.unwrap_or_default()` (silent empty-string) | **Finding 6.1**: `receive_remote_moderation_label:~735` used `.unwrap_or_default()` → persists `source_instance = ''`. Compiled cleanly. Latent data-integrity footgun. | **Sibling-diff**: locate same-file sibling doing same validation; flag every divergence where new code is *weaker* than sibling's enforced contract |
| **5. Conn acquisition idiom** | `let conn = &mut get_conn(pool).await?;` vs improvised variants | Held (byte-conformant) | Pattern-grep `let \w+ = &mut get_conn(` |
| **6. ADR-015 pseudonym handling for remote actors** | `None` for remote actors (no pseudonymisation possible) | Held | Pattern-grep `actor_pseudonym` + remote-actor type |

### C. Three audit-agent extensions (concrete deliverables)

| Asset | Current state | Phase-6 gap | Proposed change | Effort |
|---|---|---|---|---|
| `~/.claude/plugins/.../security-auditor.md` (plugin agent) | Generic OWASP/CWE checklist; "input validation at trust boundaries" item exists | **Was not run on fed-in-a/b**. Has no notion of "diff new handler vs in-repo sibling"; checklist is target-agnostic | Add §"Canonical-sibling conformance mode" — given a phase diff, locate the nearest in-repo sibling per new fn/handler, diff across the 6 axes, flag every divergence where new code is *weaker* | medium — section append to existing agent body |
| `.claude/skills/code-audit/SKILL.md` | Language-agnostic static-analysis aggregator (line counts, linter violations, complexity scores) | Knows nothing about "same-file sibling conformance"; only checks shape-level metrics | Add Phase 4g technique: **Sibling Conformance Diff** — for any new fn in scope, locate same-file sibling by name-similarity + signature-shape; flag axis-divergences with risk-tier weighting | medium — new technique in existing Phase 4 |
| **NEW** `.claude/skills/brehon-conformance-audit/SKILL.md` | Does not exist | This is the recommended primary deliverable — Brehon-specific, runnable at brief-author time AND at retro time | Write a new skill that takes `(target_file, scope: phase-diff\|fn-list, sibling-detection: same-file\|same-module)` and produces a divergence report with the 6-axis checklist + evidence + risk tier | major — new skill |

**Recommendation:** ship the third (new Brehon skill) as the primary deliverable; the security-auditor extension is the generic safety net for non-Brehon repos. The code-audit Phase 4g is the cheap retrofit if effort is constrained.

### D. Two-layer protection (prevention + detection)

Per PMD memory §"product-grade deliverables still open" — both layers needed; the audit alone is insufficient.

| Layer | Lever | Phase-6 evidence |
|---|---|---|
| **Prevention** — at plan-authoring time | Plan §10.x MIRROR stubs MUST (i) compile as written against the named sibling's bounds, (ii) carry `#[expect(dead_code, reason="wired by Cohort B Task N")]` when §13 schedules callers in a later task | §1b: fed-in-b plan §10.4 declared wrapper bound without `+ Sync`; non-compilable as written. fix-impl-1 had to add it. Lesson candidate: `feedback_plan_mirror_stub_must_compile_and_annotate_prelanded` |
| **Prevention** — at brief-authoring time | Task-4-class briefs MUST mandate a §3 "canonical-sibling diff gate" — impl-task diffs new fn vs named sibling + justifies every deviating line, especially error idioms | fix-impl-5 (this week): worker added a 4-line `.into()` closure despite a working sibling `.map_err` 4 lines above doing it correctly. Sibling-mirror reading discipline must move to §2 (currently §3) |
| **Detection** — at brief-author time | Brehon conformance-audit skill runs against `(target_file, new_code_shape)` and reports divergence before worker dispatch | Catches the class before code is written |
| **Detection** — at retro time | Same skill runs against the full phase diff before sign-off; flags every latent footgun the compiler missed | Catches Finding-6.1-class data-integrity bugs that ship clean |

### E. Self-improving evaluation loop — closing the feedback cycle

> User directive: the workflow should self-improve as it learns more about brehon-fork and its requirements.

The audit produces findings; reality (compile errors, CR findings, post-merge bugs, fix-impl cycles) produces ground truth. The system improves by comparing predictions to ground truth and updating its priors.

#### E.1 Three corpora to track

Persisted in `.claude/PRPs/audit-metrics/<sub-phase>.json` (gitignored runtime artifact; the *summary* lives in retro §X):

- **Predictions** — every divergence flagged by the audit, tagged with `axis`, `risk_tier`, `target:file:line`, `sibling:file:line`.
- **Ground truth — compile-caught** — every §15 cargo-check / clippy / test-link failure parsed from `.claude/runlog/` or DQ `kind: validate-pending` `result: fail`. Tag each failure with its axis (using §B's table).
- **Ground truth — runtime / human-caught** — every CR finding, every post-merge bug, every Junior `kind: blocker` DQ raised for "compiles but wrong" issues.

#### E.2 The four metrics per audit run

| Metric | Formula | What it tells you | Improvement signal |
|---|---|---|---|
| **Precision per axis** | `true_positives / (true_positives + false_positives)` per of the 6 axes | Which axes the audit flags reliably (high precision = trustworthy alerts) | Low precision on an axis → the detection rule is over-eager; tighten the pattern |
| **Recall per axis** | `true_positives / (true_positives + false_negatives)` per axis (false_negatives = ground-truth issues the audit missed) | Which axes the audit misses (low recall = blind spots) | Low recall on an axis → add new detection patterns; bring more siblings into the comparison set |
| **Lead time** | `audit_run_time - first_compile_failure_time` (negative = audit caught it before compiler; positive = audit missed it) | Whether the audit is genuinely preventive or post-hoc | Lead time near zero or positive → the audit is reactive; push it earlier in the workflow |
| **Latent-footgun catch rate** | `count(compile-clean issues caught by audit) / count(compile-clean issues found by any source)` | The Finding-6.1 metric specifically — audit's value beyond what the compiler already provides | Low rate → the sibling-diff axes (#4 error idiom, #6 ADR-015) need more patterns |

#### E.3 The self-improvement workflow

```
   ┌──────────────────────────────────────────────────────────┐
   │                                                          │
   │   brief-author time                                      │
   │   ┌──────────────────────┐                               │
   │   │  brehon-conformance- │                               │
   │   │  audit (new skill)   │── flags 6 divergences ──┐     │
   │   └──────────────────────┘                         │     │
   │             │                                      │     │
   │             ▼                                      │     │
   │   advisor reads findings, amends brief             │     │
   │             │                                      │     │
   │             ▼                                      │     │
   │   worker runs §15                                  │     │
   │             │                                      │     │
   │             ▼                                      │     │
   │   ┌─── §15 fails (3 axes) ───┐                     │     │
   │   │                          │                     │     │
   │   ▼                          ▼                     │     │
   │ §15 pass               fix-impl + re-run           │     │
   │   │                          │                     │     │
   │   └────────────┬─────────────┘                     │     │
   │                ▼                                   │     │
   │   ┌──────────────────────┐                         │     │
   │   │ post-§15 audit pass  │── flags Finding 6.1 ────┤     │
   │   └──────────────────────┘   (latent footgun)      │     │
   │                │                                   │     │
   │                ▼                                   │     │
   │   advisor decides: fix-now / carry-forward         │     │
   │                │                                   │     │
   │                ▼                                   │     │
   │   ┌──────────────────────────┐                     │     │
   │   │ phase ship + CR triage   │                     │     │
   │   └──────────────────────────┘                     │     │
   │                │                                   │     │
   │                ▼                                   │     │
   │   ┌──────────────────────────┐                     │     │
   │   │ phase retro: compute     │◄────────────────────┘     │
   │   │ precision / recall /     │   (predictions corpus)    │
   │   │ lead-time / latent-rate  │                           │
   │   └──────────────────────────┘                           │
   │                │                                         │
   │                ▼                                         │
   │   ┌──────────────────────────┐                           │
   │   │ low precision on axis N? │                           │
   │   │  → tighten pattern       │                           │
   │   │ low recall on axis N?    │                           │
   │   │  → add new pattern       │                           │
   │   │ low latent-rate?         │                           │
   │   │  → expand sibling set    │                           │
   │   └──────────────────────────┘                           │
   │                │                                         │
   │                ▼                                         │
   │   commit updated skill body → next sub-phase             │
   │                                                          │
   └──────────────────────────────────────────────────────────┘
```

#### E.4 Ground-truth attribution rules (so metrics don't drift)

Without these, the loop measures noise.

1. **A §15 failure counts as ground truth only if its log slice's `error[E####]` code maps unambiguously to one of the six axes.** Multi-axis failures count once per axis. Failures outside the six axes (e.g. typos, plain logic bugs) are **out of scope** — they don't penalize recall.
2. **A CR finding counts as ground truth only if it's `severity ≥ major` AND the bucket is `fix-in-pr` AND the finding text references a sibling-conformance issue.** CR's `nit`/`minor` findings are noise for this metric.
3. **A post-merge bug counts as ground truth only if the fix commit's diff modifies an axis-relevant pattern in the same file as the original commit, within 30 days of merge.** The 30-day window keeps the loop tight; older bugs have too many confounds.
4. **False positives are confirmed only by a human verdict during retro.** An audit-flagged divergence the advisor consciously accepts (e.g. "the unwrap_or_default IS the right call because the field is genuinely optional in this context") is a true negative-of-divergence-class, not a false positive. The audit's job is to surface; the advisor's job is to decide.

#### E.5 Calibration cadence

| Cadence | What runs | Why |
|---|---|---|
| **Per-sub-phase** | Compute the four metrics, write to `audit-metrics/<sub-phase>.json`, summarize in retro §X | Per-sub-phase grain matches the existing retro discipline; cheap to compute (~5 min) |
| **Every 3 sub-phases** | Aggregate across the last 3 sub-phases; if any metric crosses a threshold (e.g. precision <0.6 OR recall <0.5 on any axis), the next plan must include a calibration task | Three sub-phases is enough signal to avoid one-bad-sub-phase whiplash; cheap budget (~15 min/3-sub-phase) |
| **Every Brehon major version** | Full corpus review; retire axes that no longer fire (codebase grew past them); add new axes proposed by retros | Keeps the audit growing with the codebase, not stuck in 2026 |

#### E.6 The first calibration data point

The 2026-05-19 conformance subagent on fed-in-b produced:
- **3 compile-caught divergences** (axes 1, 1, 3) — fix-impl-1 closed.
- **1 latent footgun** (axis 4) — Finding 6.1 — fix-impl-3 pending.
- **2 minor idiom divergences** judged defensible (not footguns).
- **Hard-invariant axes (2, 5, 6) held byte-conformant** — true negatives.

If we'd had the conformance-audit skill running at brief-author time:
- Axes 1+3 would have been caught **before** Task 4 dispatch → -1 fix-impl cycle (~30 min saved).
- Axis 4 (Finding 6.1) would have been flagged **before** the data-integrity issue made it to merge → uncertain savings; could have been weeks of debugging post-pilot.
- Lead time: **negative ~3 hours per finding** (audit before brief vs. compiler at §15).
- Latent-footgun catch rate on this run: **1/1 = 100%** (Finding 6.1 caught; only because the user asked).

The metric to beat in fed-in-c: **latent-footgun catch rate ≥ 1.0 without explicit user prompt** (i.e. the skill runs by default at retro time, not on-request).

### F. Where to write this down

1. **New skill at `.claude/skills/brehon-conformance-audit/SKILL.md`** — primary deliverable. Includes the six-axis checklist (§B verbatim), the two-layer protection model (§D), and the metrics scheme (§E).
2. **Two new lesson files:**
   - `.claude/lessons/feedback_mirror_phase6_convention_in_same_file.md` (the defect class; cross-link `feedback_plan_stub_uniformity_with_canonical_sibling`, `feedback_lemmy_error_no_std_error`).
   - `.claude/lessons/feedback_plan_mirror_stub_must_compile_and_annotate_prelanded.md` (the planning-side prevention; per PMD memory §1b).
3. **Rule amendment to `.claude/rules/advisor-orchestrator.md` §G4 classifier** — add a row pointing at the conformance-audit skill for "new code in a Phase-6-bearing file" briefs.
4. **Plan-authoring rule amendment** to plan template's §10.x MIRROR section: "Stub MUST compile as written against the named sibling's bounds + carry `#[expect(dead_code, reason=…)]` for pre-landed infra."

---

## Tier 1 — act now (LIVE, recurrence ≥2 or high impact, low cost)

| # | Proposal | Source retro(s) | Recurrence | Cost | Evidence still LIVE | Suggested next action |
|---|---|---|---|---|---|---|
| 1 | **Build `brehon-conformance-audit` skill** (six-axis divergence detector) | fed-in-b session-retro + fed-in-a-retro + PMD `project_phase6_convention_divergence_class.md` (standing user directive) | 2× phases + user directive | major | Glob `.claude/skills/brehon-conformance-audit/` → no match | Write new SKILL.md per §F above |
| 2 | **Enforce thin `ScheduleWakeup` prompts** (≤80 chars, decision logic in rules at fire time) | fed-in-b session-retro + v1-ship-1-r2-retro | 8× across 2 phases | minor | `.claude/skills/auto-phase/SKILL.md` body lacks ≤80 char policy | Edit auto-phase SKILL.md + cross-ref `feedback_thin_wakeup_prompts_verify_live_state.md` |
| 3 | **Move canonical-sibling-mirror BEFORE §G4 verbatim blockquote in fix-impl briefs** | fed-in-b session-retro (fix-impl-5) + v1-ship-1 (prior) | 2× | minor | impl-task-brief.template.md §2 ordering has §G4 first, sibling-mirror in §3 | Edit template §2 ordering + 1-line policy in advisor-orchestrator.md §G4 |
| 4 | **Author `feedback_3way_eof_splice_use_index_stages_not_markers.md`** | fed-in-a-retro §3 | 1× (high impact) | minor | Glob → no match | Write lesson per fed-in-a §3 evidence |
| 5 | **Author `feedback_dead_code_shields_latent_type_errors.md`** | fed-in-b session-retro change #3 | 2× across phases | minor | Glob → no match | Write lesson per change #3 |
| 6 | **Author `feedback_verify_automated_reviewer_claims_against_compiler.md`** | refactor-tier-final + MEMORY.md already cross-references it | 2× | minor | Glob → no match (MEMORY.md links to non-existent file) | Write lesson; existing memory link suggests partial scaffolding |
| 7 | **Add `clippy::as_conversions` → §G4 allowlist** (numeric-cast-needs-expect-block) | v1-SL-c-1-retro | 1× (planner gap) | minor | advisor-orchestrator.md §G4 lacks `as_conversions` row | Add row to §G4 classifier table |
| 8 | **Planner dry-run `clippy --workspace` against IMPLEMENT blocks with `as` casts** | v1-SL-c-1-retro | 1× (planner gap) | minor | No planner-side dry-run rule found | Add to plan-authoring rule or planning-task brief template |
| 9 | **Inline canonical Python recipe in ci-watcher brief template** | v1-SL-c-1-retro | 1× | minor | ci-watcher-brief.template.md has bash, no inline Python | Append Python recipe block to template |
| 10 | **Post-mutation diff inspection checklist** | v1-SL-c-1-retro | 1× | minor | Glob → no match | Lesson file or checklist appendix in ci-watcher brief template |
| 11 | **Fix structural HIGH: gitignored bm-verb artifacts lost in cleaned per-job worktrees** | fed-in-a-retro §3 (decide before next CR cycle) | 1× (HIGH structural) | medium | bm-poll-cr.md + bm-triage.md still write to gitignored paths | Pick option (a) commit to non-gitignored path, (b) echo to task output, or (c) daemon retains worktree |
| 12 | **DQ id collision via archive walk** — wire `resolve-dq-canonical.sh` to Junior task-0 pre-flight | v1-SL-lane-meta + v1-SL-c-2-retro | 2× | medium | impl-task.md + bm-task.md lack resolve-dq-canonical pre-flight | Edit agent definitions to invoke resolver |
| 13 | **Concurrent-lane pre-e2e merge scan** — check `gh pr list` before launching Phase-2 e2e | v1-SL-lane-meta | 1× | minor | advisor-orchestrator.md has no pre-e2e gh-pr-list gate | Add to §3.1 Phase-2 e2e stage-shape |
| 14 | **Stale-EliteDesk governance-v0 pre-flight** — `git fetch + pull --ff-only` before ci-watcher dispatch | v1-SL-c-2-retro (L18) | 3rd recurrence | minor | ci-watcher.md + impl-task.md pre-flight lacks L18 fix | Add pre-flight step to both agent definitions |
| 15 | **Multi-pass briefs: mandatory per-pass DoD** | refactor-tier-final | 1× | minor | Templates lack per-pass DoD | Edit impl-task-brief.template.md |
| 16 | **Advisor inline-author exception discipline** | refactor-tier-final | 1× | minor | branch-manager.md lacks advisor inline-author rule | Edit branch-manager.md autonomy table |
| 17 | **Brief-template pre-lint-debt baseline for `-D warnings` gates** | refactor-tier-final | 1× | minor | impl-task-brief.template.md lacks baseline criterion | Add to §2 VALIDATE block |
| 18 | **Brief-template "executes in default suite" criterion for audit-deliverable** | refactor-tier-final | 1× | minor | Same template lacks this criterion | Add to §2 |
| 19 | **`grep -c` zero-count exit-1 trap helper** (`scripts/brehon/grep-count.sh`) | fed-in-b session-retro change #4 | 2× | minor | Script does not exist | Write 5-line wrapper + brief template note |
| 20 | **Re-use EXACT validated command scope from prior passing DQ** | refactor-tier-final | 1× | minor | No advisor-orchestrator.md rule found | Add to §5 advisor discipline |
| 21 | **Author `feedback_advisor_clippy_before_commit.md`** | refactor-tier-final | 1× | minor | Glob → no match | Write lesson |
| 22 | **PMD lesson-query isolation between bash hooks and MCP tools** | session-retro-2026-05-18-pmd-stranding + session-retro-2026-05-16-pmd-backfill-dq-hook-scope | 2× | medium | `feedback_pmd_cross_lane_canonical_db.md` covers DB pinning but not hook-vs-MCP query isolation specifically | Extend existing lesson or write a new sibling lesson |

---

## Tier 2 — schedule (LIVE, single-occurrence or high-cost-high-value)

| # | Proposal | Source retro | Cost | Evidence still LIVE | Note |
|---|---|---|---|---|---|
| 1 | Restoration-during-window-escapes → restorative-mechanics-v1 PRD | v1-SL-lane-meta | major | Glob `.claude/PRPs/prds/restorative-mechanics*.md` → no match | DQ #145 deferred; PRD scoping needed before any further work |
| 2 | Daemon finalize-merge push patch upstream (17+ instances) | v1-SL-lane-meta + v1-RT-r1-halt L6 | major | Lesson candidate `feedback_junior_finalize_merge_race_lossless_reconcile.md` referenced but file Glob → no match; upstream `barrie-cork/lemmy#134` tracked | Upstream daemon fix vs lesson-only choice |
| 3 | PENDING-collapse-to-single-session (multi-lane overhead ~45 min/cross-merge) | v1-SL-lane-meta + v1-SL-c-2-retro | medium-high | Glob `feedback_single_session*.md` → no match | Worth a clarify session on the tradeoff before committing |
| 4 | Phase-6 convention-divergence as tracked sub-phase work item | fed-in-b session-retro change #5 | medium | Already user-deferred to retro 2026-05-19 | Convert deferred concern to named planner task slot |
| 5 | Deferred-write semantics test pattern (negative+positive assertion pair) | v1-SL-e-retro | minor | Glob `feedback_deferred_write*.md` → no match | Lesson plus test pattern documentation |
| 6 | Force-rewind `grace_expires_at` test technique | v1-SL-e-retro | minor | Glob → no match | Lesson |
| 7 | `revert_migrations` limit tracking across concurrent lanes | v1-SL-e-retro | medium | Glob `feedback_revert_migrations*.md` → no match | Cross-lane test discipline |
| 8 | Verify branch before e2e launch lesson | v1-SL-d-retro | minor | Glob → no match | Lesson capturing wrong-branch run1/correct-branch run2 incident |
| 9 | ci-watcher stop-hook/PMD isolation lesson | v1-SL-d-retro | minor | Glob → no match | Lesson capturing PMD isolation pattern |
| 10 | bm-poll-cr stop-hook loop lesson | v1-SL-d-retro | minor | Glob → no match | Lesson |
| 11 | Junior bm-task with `base_branch=chore/*` daemon worktree-ref bug | refactor-tier-retro | medium | No lessons documenting; bm-pr.md cites `chore/*` but not the bug | Lesson + possible daemon fix |
| 12 | Cherry-picking bm-pr brief causes CONFLICTING | refactor-tier-retro | medium | bm-pr.md has no cherry-pick guidance | Lesson + bm-pr.md amendment |
| 13 | Migration plans MUST cite ALL invariants — extend `feedback_migration_invariants_full_mirror.md` to additional invariant classes | v1-SL-a-retro | medium | Lesson exists but extension cases recur | Append-only revision to existing lesson |
| 14 | Three §G4 user-gate escalations = smell (allowlist extension review at retro) | v1-SL-a-retro | minor | Glob `.claude/PRPs/templates/retro.template.md` → no match (no canonical template) | Could codify as retro question; needs retro-template first |
| 15 | ADR red-flag scanner exhaustive-match exemption | v1-SL-a-retro + v1-SL-lane-meta | minor | Already STALE per batch-3 audit (`feedback_adr_scanner_false_positive_arm_split.md` exists) — moved to Tier 3 | (see Tier 3) |
| 16 | Defer next federation-outbound lane until post-restoration-mechanics finalization | session-retro-2026-05-20-fed-in-b | n/a | Standing user directive; not a tooling change | Roadmap-level decision |

---

## Tier 3 — closeable (STALE / SUPERSEDED — just check the box)

| Proposal | Source retro | Verdict | Evidence | Action |
|---|---|---|---|---|
| Post-condition-verify Junior "done" with real-world checks | v1-federation-inbound-a-retro | STALE | `.claude/lessons/feedback_bm_false_success_advisor_post_condition_catch.md` exists (lines 27-30 codify auto-catch-fire) | Tick `[x]` in source retro |
| Crate-qualify Lemmy-1.0 schema paths in planning briefs | v1-federation-inbound-a-retro | STALE | `.claude/PRPs/templates/plan.template.md:123` already cites `crates/db_schema/src/source/governance/...` pattern | Tick `[x]` |
| Harden BM hard-refusal language + advisor auto-catch-fire rule | v1-ship-1-r2-retro | STALE | bm-merge.md:49 + lesson 27-30 (shipped 2026-05-18) | Tick `[x]` |
| Daemon-local ref divergence recovery recipe | v1-ship-1-r2-retro | STALE | `feedback_junior_292_stale_base_recover_recipe.md` lines 28-36 encode full recipe | Tick `[x]` |
| #292 stale-base-self-merge recover recipe | v1-AD-e-retro | STALE | Same lesson — full procedure documented | Tick `[x]` |
| impl-task silent-partial on enumerated-transform briefs | v1-AD-e-retro | STALE | `feedback_impl_task_enumerated_transform_all_or_blocker.md` lines 24-27 codify all-N-or-blocker | Tick `[x]` |
| Windows DQ/JSON via Write-tool fragment | v1-AD-e-retro | STALE | `feedback_windows_backslash_path_dq_via_write_fragment.md` lines 20-37 encode pattern | Tick `[x]` |
| GH workflow stuck-runner: timeout-minutes + concurrency:cancel | refactor-tier-retro | STALE | `.github/workflows/cargo-validate-workspace.yml:33-44` has both | Tick `[x]` |
| Re-pointed worktree does NOT auto-init submodules | refactor-tier-retro | STALE | `feedback_worktree_submodules_not_auto_init.md` exists | Tick `[x]` |
| ScheduleWakeup crons accumulate → stale wakeup | refactor-tier-retro | STALE-but-RELATED | `feedback_thin_wakeup_prompts_verify_live_state.md` covers behavior. But Tier-1 #2 above is the structural fix (≤80 char prompt) | Tick `[x]` for the lesson; keep Tier-1 #2 |
| Plan §11 enumerate callsites when adding field to public struct | v1-RT-r1-retro | STALE | `feedback_fix_impl_enumerate_all_callsites.md` exists | Tick `[x]` |
| Phase retro gate (Task 11) not detection-enforced | v1-RT-r1-retro | STALE | `feedback_phase_retro_gate_enforcement.md` + bm-pr.md:56+60 pre-condition check | Tick `[x]` |
| DQ stale-fails cause bm-merge friction | v1-RT-r1-retro + v1-SL-d-retro | STALE | `feedback_dq_historical_fail_sweep_at_bm_pr.md` exists with full sweep procedure | Tick `[x]` |
| Fix-impl pre-push cargo-check discipline | v1-RT-r1-retro | STALE | `feedback_fix_impl_pre_push_cargo_check.md` exists | Tick `[x]` |
| DQ commits on phase branch cause PR DIRTY conflicts | v1-SL-c-2-retro | STALE | bm-merge.md L59 + `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md` covered; `.gitattributes` union-merge driver shipped | Tick `[x]` |
| PR DIRTY early-check with `mergeStateStatus` | v1-SL-c-2-retro | STALE | bm-merge.md §2.2 L169 has explicit mergeStateStatus check | Tick `[x]` |
| DQ id collision — archive-file walk | v1-SL-c-2-retro | STALE | `.claude/rules/decision-queue.md` L77-90 documents archive walk | Tick `[x]` |
| ci-watcher.md Hard refusal #7 first-line emphasis | v1-SL-c-1-retro | STALE | ci-watcher.md L166 has Hard refusal #7 (deprecated kinds) | Tick `[x]` |
| Migration plans MUST cite ALL invariants (base lesson) | v1-SL-a-retro | STALE | `feedback_migration_invariants_full_mirror.md` exists (extension cases moved to Tier 2 #13) | Tick `[x]` for base |
| Cross-check brief filenames against `git ls-tree origin/<phase>` | v1-SL-a-retro | STALE | `feedback_brief_naming_phase_branch_check.md` lines 6-14 | Tick `[x]` |
| Parallel-session phase-branch coordination (`.claude/agent-activity.json` check) | v1-SL-a-retro | STALE | session-awareness.md L20 + auto-phase.md L50 cross-reference agent-activity.json | Tick `[x]` |
| ADR red-flag scanner exhaustive-match exemption | v1-SL-a-retro + v1-SL-lane-meta | STALE | `feedback_adr_scanner_false_positive_arm_split.md` documents rule + workaround | Tick `[x]` |
| Subagent brief-compliance L1 emphasis (bm-task spec) | v1-SL-lane-meta | STALE | bm-task.md:1-30 enumerates Read rules/brief/verb-script + Glob lessons + git fetch | Tick `[x]` |
| Resolve-dq-canonical.sh wiring (canonical resolver exists, agent-pre-flight is Tier-1 #12) | v1-SL-lane-meta | STALE-partial | advisor-orchestrator.md + auto-phase.md cite the resolver as the Phase 0.5 mechanism; agent-side pre-flight is the LIVE half (Tier-1 #12) | Tick `[x]` for resolver existence; keep Tier-1 #12 |
| File-class table for e2e impl tasks | session-retro-2026-05-09 | STALE | advisor-orchestrator.md L45-66 contains the table with e2e.rs rows | Tick `[x]` |
| suspend auto-phase transitions, require explicit user gate | session-retro-2026-05-16-shape-g-suspension | STALE | auto-phase.md §"State-routing invariants" L124-128 enforces six non-skippable gates | Tick `[x]` |
| L14 rule revision (POST-merge runlog) | v1-ship-1-r2-retro | STALE | Applied 2026-05-18 `bade657f4` | Tick `[x]` |
| §G4 row 4 split (A/B/C) per Case A/B/C | v1-SL-c-2-cycles-retro | STALE | advisor-orchestrator.md §G4 row 4a/4b/4c exists | Tick `[x]` |
| Anti-paraphrase gate for fix-impl briefs (§G4 verbatim blockquote) | session-retro-2026-05-09-cycle-3-followup | STALE | advisor-orchestrator.md §G4 "Mandatory verbatim §G4 row" sub-section exists | Tick `[x]` |
| `feedback_lemmy_error_no_std_error.md` Cases A/B/C amendment | session-retro-2026-05-09-cycle-3-followup | STALE | Cases A/B/C present in lesson | Tick `[x]` |

---

## Unverified STALE claims (downgraded to LIVE — needs a human look)

None this run. Every STALE verdict above carries a citable file:line or glob hit; every LIVE verdict carries a negative check (`Glob → no match` or `Grep → no match`). The Phase-2 triage subagents held the discipline (`bias toward STALE only with positive evidence; absence of evidence ⇒ LIVE`).

---

## Suggested action sequence

The user reads the report and decides; below is the ordering I would recommend if asked.

1. **Write `.claude/skills/brehon-conformance-audit/SKILL.md`** (Tier-1 #1) — paste the six-axis table from §B verbatim, encode the self-improving evaluation loop from §E. This is the single highest-leverage item. Cost: ~2 hours; recurrent payback: catches latent Finding-6.1-class footguns at brief-author and retro time.
2. **Enforce thin `ScheduleWakeup` prompts** (Tier-1 #2) — 1-line edit to `.claude/skills/auto-phase/SKILL.md`; ~3-5k token savings per wakeup × 8× recurrence per phase.
3. **Move canonical-sibling-mirror to §2.1 in brief template** (Tier-1 #3) — 1-paragraph template edit; eliminates the fix-impl-5-class paraphrase failure.
4. **Tick the ~28 STALE boxes in Tier 3** — pure bookkeeping; ~10 minutes of `Edit` calls; closes the retro backlog so future harvests don't re-process them.
5. **Author the 4 Tier-1 lesson files** (#4, #5, #6, #21) — each ~30-min lesson; recurrence ≥2× justifies the cost.
6. **Edit `.claude/PRPs/templates/impl-task-brief.template.md`** (Tier-1 #15, #17, #18, #20) — bundle the four template-touching changes into one commit.
7. **Tier-2 items** — schedule after fed-in-c lands; the federation/restoration questions are roadmap-coupled.

---

_Generated by `.claude/skills/retro-harvest/SKILL.md`. Read-only sweep;
no rules/lessons/code modified. Currency verdicts are evidence-checked
against HEAD `410fedb44` — re-run after acting to confirm closure._
