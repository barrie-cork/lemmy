# Implementation-guidance — `brehon-conformance-audit` skill

**Audience:** the `planning` subagent (Opus 4.7) that will be dispatched in a future session to author `.claude/PRPs/plans/brehon-conformance-audit.plan.md`.
**Written:** 2026-05-20 by advisor session on `governance-v0` @ `410fedb44`.
**Status:** advisory input to the future planning brief. The advisor will refine this into a proper planning brief at `.claude/PRPs/briefs/brehon-conformance-audit-planning-1.md` when the session opens; this document carries the substantive context so that brief can be short.

---

## 1. Why this skill exists (the design problem)

Phase-6 federation work (sub-phases `v1-federation-inbound-a`, `-b`) produced a defect class the existing tooling does not catch:

**New code (a new handler/helper/wrapper) is authored alongside a canonical Phase-6 sibling doing the same job in the same file/module, and the new code diverges across one or more of six axes — sometimes the compiler catches it, sometimes (worse) it compiles cleanly and ships a latent footgun.**

Concrete evidence from the 2026-05-19 conformance audit on `v1-federation-inbound-b` (recorded in PMD `project_phase6_convention_divergence_class.md` and `.claude/PRPs/reports/v1-federation-inbound-a-retro.md`):

- **3 compile-caught divergences** (axes "conn-type" and "trait-bound completeness") → fix-impl-1 closed `cdff6f09d`.
- **1 latent footgun, `cargo check`-clean** (axis "error idiom at trust boundary"): `receive_remote_moderation_label:~735` used `.domain().map(str::to_string).unwrap_or_default()` → would have persisted `source_instance = ''` for any domainless remote actor. Phase-6 siblings four lines away hard-error `.domain().ok_or_else(|| LemmyErrorType::Unknown(...))?`. Caught only because the user asked for "a thorough product-grade interpretation" and the advisor ran a bespoke read-only subagent. Without that ask, it would have shipped.

The reactive per-incident fixes (fix-impl-1, fix-impl-3) closed the specific instances. The defect *class* remains uncaught by the existing audit assets:

- `~/.claude/plugins/.../security-auditor.md` — generic OWASP/CWE checklist; no notion of "diff a new handler vs in-repo sibling".
- `.claude/skills/code-audit/SKILL.md` — language-agnostic static-analysis aggregator (line counts, complexity); no sibling-conformance technique.
- `code-reviewer`, `silent-failure-hunter`, `type-design-analyzer` (subagents) — adjacent but none does the canonical-sibling-divergence pass.

**The deliverable** is a new Brehon-specific skill at `.claude/skills/brehon-conformance-audit/SKILL.md` that runs at brief-author time (prevention) and at retro time (detection) against `(target_file, phase_diff)` and produces a divergence report across the six axes — including divergences that compile cleanly. Paired with a closed-loop metrics scheme that turns each run into training data for the next run, so the audit's precision and recall improve as the codebase grows.

---

## 2. Hard refusals the planner must respect

These are non-negotiable. Violating any is a process breach the advisor will catch at plan approval:

1. **The skill is read-only.** It never edits `crates/**`, `migrations/**`, `tests/**`, `docs/**`, plans, briefs, or lessons. It writes exactly two artifact paths: the per-run audit report at `.claude/PRPs/reports/conformance-audit-<phase>-<YYYY-MM-DD>.md` and the per-run metrics file at `.claude/PRPs/audit-metrics/<phase>.json` (gitignored — runtime journal).
2. **The six axes are the schema. Do NOT invent a seventh axis.** New divergence patterns are added by promoting existing PMD lessons (see §6 — self-improvement loop), not by hand-authoring axes. The schema is the contract retro-time + brief-author-time consumers both rely on; ad-hoc axes break that contract.
3. **STALE / LIVE / SUPERSEDED-style evidence discipline.** Every divergence flag must carry citable evidence (sibling file:line + new-code file:line + the diff between them). A flag without evidence is downgraded to "unverified" and surfaced separately — the user cannot trust a verdict they cannot see the basis for. Per `.claude/lessons/feedback_runbook_audit_drift_post_event_check.md`.
4. **The advisor decides verdicts, not the audit.** The audit *surfaces* divergences with risk-tier weighting; the human-in-the-loop (advisor / user gate) decides which are footguns and which are defensible context-specific divergences. False positives are confirmed only by human verdict at retro time. The audit must never auto-amend code, auto-edit briefs, or auto-promote lessons.
5. **No new ADR.** The skill respects existing ADRs (especially ADR-006 advisory-only, ADR-013 emergency-remove, ADR-014 vanilla-Lemmy interop, ADR-015 pseudonymisation). Axis #6 specifically encodes ADR-015. If the planner believes a new ADR is needed to express the audit's scope, STOP and file `kind: "blocker"`.
6. **Never trust automated-reviewer claims against the compiler.** Per `.claude/lessons/feedback_verify_automated_reviewer_claims_against_compiler.md` (PR #132 cr-2 precedent: CR's `.get(0)` → `.first()` claim broke Diesel `LimitDsl`; user's `#![expect(get_first)]` was right). The skill's outputs are hypotheses until the compiler proves them. Every "this is wrong" flag must be backed by either (a) a compile error the planner can reproduce, OR (b) a sibling diff that shows the new code is weaker than an enforced contract.
7. **Honour `feedback_principles_not_rules`.** The skill's design favours judgment with cited evidence over imperative `MUST`-stuffing. No "CRITICAL", no "NON-NEGOTIABLE", no all-caps stop-signs in the SKILL.md body. The discipline lives in the *evidence requirement* (every flag carries cited file:lines), not in tone.

---

## 3. What the skill produces (the contract)

### 3.1 Inputs

- **`target_scope`** (one of):
  - `phase-diff <phase-branch>` — diff `phase-<phase>..governance-v0` (or against advisor-named base), enumerate every new fn/handler in the diff.
  - `file <path>` — single-file audit (used at brief-author time for a specific target file).
  - `fn-list <file:fn>,<file:fn>,...` — targeted audit (used at retro time when the verify step found specific suspects).
- **`sibling-detection`** (default: `same-file`): `same-file` | `same-module` | `same-crate`. `same-file` is the highest-signal default — Phase-6 evidence showed siblings sit 4-50 lines away in the same file.

### 3.2 Outputs

**Per-run audit report** at `.claude/PRPs/reports/conformance-audit-<scope>-<YYYY-MM-DD>.md`:

```markdown
# Conformance audit — <scope> — <YYYY-MM-DD>

**Trunk HEAD:** <sha-short>
**Scope:** <phase-diff <branch> | file <path> | fn-list ...>
**Sibling detection:** <same-file | same-module | same-crate>
**Skill version:** <semver from .claude/skills/brehon-conformance-audit/VERSION>

## TL;DR

<One paragraph: total divergences flagged by axis, count by risk tier, the single highest-leverage finding to look at first.>

## Findings by risk tier

### Tier 1 — likely footgun (compile-clean + weakens an enforced contract)

| # | New code | Sibling | Axis | Divergence | Suggested action |
|---|---|---|---|---|---|
| 1 | inbox.rs:735 | inbox.rs:201 | #4 error idiom | new: .unwrap_or_default() / sibling: .ok_or_else(...)? | replace with .ok_or_else(LemmyErrorType::Unknown(...))? |

### Tier 2 — compile-caught (would fail §15 cargo-check)

[same shape as Tier 1]

### Tier 3 — defensible-but-flagged (advisor judgment required)

[same shape]

## Hard-invariant axes that held (true negatives)

<List of axes that were checked and found byte-conformant — keeps the report's "I looked at this and it was fine" signal explicit.>

## Methodology

<One paragraph: how siblings were located, which axes were checked, what counts as "enforced contract" weakening.>
```

**Per-run metrics file** at `.claude/PRPs/audit-metrics/<phase>.json` (gitignored):

```json
{
  "schema_version": 1,
  "scope": "<scope>",
  "head_sha": "<short>",
  "run_at": "<ISO 8601>",
  "skill_version": "<semver>",
  "predictions": [
    { "axis": "4", "risk_tier": 1, "target": "inbox.rs:735", "sibling": "inbox.rs:201", "evidence": "<≤120 chars>" }
  ]
}
```

Ground truth gets *appended* to this file post-§15 / post-CR-triage / post-30-days (per §6 below).

---

## 4. The six axes (paste verbatim into the plan §10)

Recorded in PMD `project_phase6_convention_divergence_class.md` and the retro-harvest report. These are the contract the skill detects against:

| Axis | What it means | Phase-6 instance | Detection method |
|---|---|---|---|
| **1. Conn-type / tx-boundary** | Does the new fn take `&mut DbConn<'_>` (starts tx via `.run_transaction`) or `&mut AsyncPgConnection` (must already be inside a tx)? | fix-impl-1 (#337): 2 helpers took `&mut AsyncPgConnection` but called `.run_transaction` (method on `&mut DbConn`); siblings 4 lines away took `&mut DbConn` | Grep file for `.run_transaction(` callers, check receiver type vs sibling receiver type |
| **2. Append reborrow shape** | In-tx `governance_log::append` calls must reborrow `&mut (&mut *conn).into()` exactly | Held (byte-conformant) | Pattern-grep against the canonical 4-token sequence |
| **3. Trait-bound completeness** | `#[async_trait]` methods with `Sync`-requiring bodies need `A: Sync` on the generic | fix-impl-1: `wrap_governance_inbound<A: GovernanceInboundActivity>` missing `+ Sync`; compile-caught | Compiler is the oracle, but planning-time MIRROR stub should compile-check |
| **4. Error idiom at trust boundary** | `.domain().ok_or_else(\|\| LemmyErrorType::Unknown(...))?` (hard-error) vs `.unwrap_or_default()` (silent empty-string) | **Finding 6.1**: `receive_remote_moderation_label:~735` used `.unwrap_or_default()` → persists `source_instance = ''`. Compiled cleanly. Latent data-integrity footgun. | **Sibling-diff**: locate same-file sibling doing same validation; flag every divergence where new code is *weaker* than sibling's enforced contract |
| **5. Conn acquisition idiom** | `let conn = &mut get_conn(pool).await?;` vs improvised variants | Held (byte-conformant) | Pattern-grep `let \w+ = &mut get_conn(` |
| **6. ADR-015 pseudonym handling for remote actors** | `None` for remote actors (no pseudonymisation possible) | Held | Pattern-grep `actor_pseudonym` + remote-actor type |

The plan's §10 lifts this table verbatim. The skill body itself encodes detection patterns per axis — Grep / Read / LSP calls — and produces evidence strings citing the canonical sibling's file:line.

---

## 5. Suggested §13 task decomposition (planner refines)

The planner authors §13 per their normal discipline — the §5.1 complexity-score computation, the FILES YAML per task, `[P]` markers only where genuinely disjoint, `requires:` arrays for cross-task dependencies. The following is the advisor's suggested shape; the planner may restructure with rationale in §10.

**Pre-estimate complexity score: ~6-8** (within Sonnet threshold; the planner verifies). The skill is a self-contained additive read-only deliverable — no migrations, no crate changes, no e2e changes.

### Suggested tasks

1. **Task 0 — pre-flight harness audit.** Standard Brehon pre-flight; `creates: []`, `modifies: []`.
2. **Task 1 — write `.claude/skills/brehon-conformance-audit/SKILL.md` (skeleton + axis table).** The skill body, frontmatter, six-axis detection table (§4 verbatim). `creates: [.claude/skills/brehon-conformance-audit/SKILL.md, .claude/skills/brehon-conformance-audit/VERSION]`.
3. **Task 2 — write the axis-detection sub-files.** One per axis: `.claude/skills/brehon-conformance-audit/axes/{1-conn-type,2-append-reborrow,3-trait-bound,4-error-idiom,5-conn-acquisition,6-adr-015}.md`. Each sub-file describes the detection method (Grep pattern / LSP call / Read pattern) and the evidence-string format. Per `.claude/lessons/reference_caveman_skill_evaluated.md` rationale (skills compose better when each axis is its own sub-file the parent skill loads on demand). `creates: [.claude/skills/brehon-conformance-audit/axes/*.md]`.
4. **Task 3 — write the sibling-detection helper.** A bash/python helper at `.claude/skills/brehon-conformance-audit/scripts/find-sibling.sh` that, given `(target_file, target_fn)`, locates the nearest in-file (then in-module, then in-crate) function with a similar signature. Uses LSP `documentSymbol` per `pattern_verify_before_trusting_shell_output` and Grep as fallback. `creates: [.claude/skills/brehon-conformance-audit/scripts/find-sibling.sh]`.
5. **Task 4 — write the metrics schema + ground-truth attribution rules.** The JSON schema at `.claude/skills/brehon-conformance-audit/audit-metrics.schema.json` + a markdown explainer at `.claude/skills/brehon-conformance-audit/METRICS.md` codifying §6's ground-truth attribution rules. `creates: [.claude/skills/brehon-conformance-audit/{audit-metrics.schema.json,METRICS.md}]`.
6. **Task 5 — write the calibration script.** `.claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh` reads `.claude/PRPs/audit-metrics/*.json`, computes precision/recall/lead-time/latent-rate per axis (per §6 below), writes a summary to stdout. Runs per-sub-phase at retro time; also runs every-3-sub-phases for trend detection. `creates: [.claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh]`.
7. **Task 6 — dogfood the skill against `v1-federation-inbound-b` (the source phase).** Run the skill against the closed-out fed-in-b diff. Confirm: (a) it re-finds the 3 compile-caught axes 1+1+3 divergences fix-impl-1 closed; (b) it re-finds Finding 6.1 (axis 4); (c) it produces a clean report at `.claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-dogfood-<date>.md`; (d) the metrics file captures the four findings as ground truth (compile-caught, axis 1/1/3; latent, axis 4). This task IS the brief's pre-commit dogfood gate (per `.claude/lessons/feedback_dogfood_slash_command_specs.md`). `creates: [.claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-dogfood-<date>.md]`.
8. **Task 7 — wire into the advisor stage-shape.** Edit `.claude/rules/advisor-orchestrator.md` §3.x to add a "Conformance-audit checkpoint" sub-section between brief-author and impl dispatch (prevention layer) and between bm-pr and bm-merge (detection layer). `modifies: [.claude/rules/advisor-orchestrator.md]`.
9. **Task 8 — wire into the §G4 classifier.** Edit `.claude/rules/advisor-orchestrator.md` §G4 — add a row pointing at the conformance-audit skill for "new code in a Phase-6-bearing file" briefs. `modifies: [.claude/rules/advisor-orchestrator.md]` (cohort-incompatible with Task 7 — both edit the same file; serial only).
10. **Task 9 — author the two paired lesson files.** `.claude/lessons/feedback_mirror_phase6_convention_in_same_file.md` (defect class; cross-link `feedback_plan_stub_uniformity_with_canonical_sibling`, `feedback_lemmy_error_no_std_error`) + `.claude/lessons/feedback_plan_mirror_stub_must_compile_and_annotate_prelanded.md` (planning-side prevention per PMD §1b). Per `.claude/lessons/feedback_one_system_memory_in_repo.md` + `feedback_lesson_mirror_check.md`. `creates: [.claude/lessons/{feedback_mirror_phase6_convention_in_same_file.md,feedback_plan_mirror_stub_must_compile_and_annotate_prelanded.md}]`.
11. **Task 10 — retro task.** Standard Brehon phase-close retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`. `creates: [.claude/PRPs/reports/brehon-conformance-audit-retro.md]`.

**§5.1 complexity factors** (planner verifies; this is a pre-estimate):
- §13 impl tasks above 5: +5 (Tasks 1-5)
- Migrations: 0
- Crates touched: 0 (skill is `.claude/skills/` only)
- e2e edits: 0
- ADR-affecting: 0
- Cargo budget: 0 (Shape-G suspended; this skill runs no cargo)
- **Estimated score: ~5-6**, comfortably below Sonnet's split threshold of 8.

**Cohort opportunities (`[P]`):** Tasks 2/3/4 likely [P] (disjoint sub-files / scripts / schemas); Task 5 [P] with them (disjoint script); Task 7 + Task 8 NOT [P] (both modify `advisor-orchestrator.md`); Task 9 [P] with Tasks 7+8 (disjoint lesson files). The planner sets `requires:` arrays rigorously.

---

## 6. The self-improving evaluation loop (the part you specifically asked for)

This is the part that turns each audit run into training data for the next run. The audit produces *predictions* (flagged divergences); reality produces *ground truth* (compile errors at §15, CR findings post-bm-pr, post-merge bugs). The system improves by comparing the two.

### 6.1 Three corpora to track

Persisted in `.claude/PRPs/audit-metrics/<phase>.json` (gitignored; summary lives in retro):

- **Predictions** — every divergence flagged by the audit run, tagged with `axis`, `risk_tier`, `target:file:line`, `sibling:file:line`, `evidence`.
- **Ground truth — compile-caught** — every §15 cargo-check / clippy / test-link failure, parsed from `.claude/runlog/` or DQ `kind: validate-pending` `result: fail`. Each failure tagged with its axis using the six-axis table.
- **Ground truth — runtime / human-caught** — every CR finding with `severity ≥ major` and `bucket: fix-in-pr`, every post-merge bug whose fix commit modifies an axis-relevant pattern in the same file within 30 days of merge, every Junior `kind: blocker` DQ raised for "compiles but wrong" issues.

### 6.2 The four metrics

| Metric | Formula | What it tells you | Improvement signal |
|---|---|---|---|
| **Precision per axis** | `true_positives / (true_positives + false_positives)` per axis | Which axes the audit flags reliably (high precision = trustworthy alerts) | Low precision on an axis → the detection rule is over-eager; tighten the pattern |
| **Recall per axis** | `true_positives / (true_positives + false_negatives)` per axis (false_negatives = ground-truth issues the audit missed) | Which axes the audit misses (low recall = blind spots) | Low recall on an axis → add new detection patterns; bring more siblings into the comparison set |
| **Lead time** | `audit_run_time - first_compile_failure_time` (negative = audit caught it before compiler; positive = audit missed it) | Whether the audit is genuinely preventive or post-hoc | Lead time near zero or positive → the audit is reactive; push it earlier in the workflow |
| **Latent-footgun catch rate** | `count(compile-clean issues caught by audit) / count(compile-clean issues found by any source)` | The Finding-6.1 metric — audit's value beyond what the compiler already provides | Low rate → axes #4 (error idiom) and #6 (ADR-015) need more patterns |

### 6.3 Ground-truth attribution rules (so the metrics don't drift)

1. **A §15 failure counts as ground truth only if its log slice's `error[E####]` code maps unambiguously to one of the six axes.** Multi-axis failures count once per axis. Failures outside the six axes (typos, plain logic bugs) are **out of scope** — they don't penalize recall.
2. **A CR finding counts as ground truth only if `severity ≥ major` AND `bucket = fix-in-pr` AND the finding text references a sibling-conformance issue.** CR's `nit`/`minor` findings are noise for this metric.
3. **A post-merge bug counts as ground truth only if the fix commit's diff modifies an axis-relevant pattern in the same file as the original commit, within 30 days of merge.** The 30-day window keeps the loop tight; older bugs have too many confounds.
4. **False positives are confirmed only by a human verdict during retro.** An audit-flagged divergence the advisor consciously accepts ("the `unwrap_or_default` IS the right call because the field is genuinely optional in this context") is a true-negative-of-divergence-class, not a false positive. The audit's job is to surface; the advisor's job is to decide.

### 6.4 Calibration cadence

| Cadence | What runs | Why |
|---|---|---|
| **Per-sub-phase** | Compute the four metrics; write to `audit-metrics/<phase>.json`; summarize in retro §X | Per-sub-phase grain matches existing retro discipline; cheap (~5 min) |
| **Every 3 sub-phases** | Aggregate across the last 3 sub-phases; if any metric crosses a threshold (e.g. precision <0.6 OR recall <0.5 on any axis), the next plan must include a calibration task | Three sub-phases is enough signal to avoid one-bad-sub-phase whiplash; cheap budget (~15 min/3-sub-phase) |
| **Every Brehon major version** | Full corpus review; retire axes that no longer fire (codebase grew past them); add new axes proposed by retros (via lesson promotion, not ad-hoc) | Keeps the audit growing with the codebase |

### 6.5 The first calibration data point (Task 6 dogfood)

The 2026-05-19 conformance subagent on fed-in-b produced:
- 3 compile-caught divergences (axes 1, 1, 3) — fix-impl-1 closed `cdff6f09d`.
- 1 latent footgun (axis 4) — Finding 6.1 — fix-impl-3.
- 2 minor idiom divergences judged defensible (true negatives).
- Hard-invariant axes (2, 5, 6) held byte-conformant — true negatives.

**If the skill had been running at brief-author time:**
- Axes 1+3 caught **before** Task 4 dispatch → -1 fix-impl cycle (~30 min saved).
- Axis 4 (Finding 6.1) flagged **before** the data-integrity issue made it to merge.
- Lead time per finding: **negative ~3 hours** (audit before brief vs. compiler at §15).
- Latent-footgun catch rate on this run: **1/1 = 100%** (Finding 6.1; only because the user asked).

**Metric to beat in fed-in-c:** **latent-footgun catch rate ≥ 1.0 without explicit user prompt** (i.e. the skill runs by default at retro time, not on-request).

---

## 7. Relevant approaches from the Rust best-practices research

`docs/research/Search for best practices when building with rust.md` ships 15 practices. The planner should research and incorporate the following six, all of which are directly relevant to a Rust-aware audit skill (the others are about agent-prompt hygiene that Brehon already handles via different mechanisms):

| Practice | How it informs the conformance audit | Applies to |
|---|---|---|
| **#4 — filter compiler output** | The skill's `cargo check`-based ground-truth scraping needs to handle Rust's verbose multi-line errors. The Brehon `scripts/brehon/cargo-check.sh` wrapper + tail-N capture already addresses this; the metrics script should reuse it, not reinvent. Per `.claude/lessons/feedback_cargo_output_capture.md` style. | Tasks 4, 5 (metrics scripts) |
| **#5 — `cargo check` not `cargo build`** | The compile-check axis (axes 1, 3) uses `cargo check --workspace --features full`, not `cargo build`. Already standard Brehon practice per `pattern_cargo_feature_flag_propagation`; reinforce in axis sub-files. | Task 2 (axis 1, 3) |
| **#6 — scope test runs to crate** | The skill never runs `cargo test --workspace`; the metrics script reads existing §15 outputs from runlogs/DQ. Axis sub-files do not invoke cargo at all — they Grep/Read. The compiler is consulted via existing ground-truth corpora, not re-invoked. | Tasks 2-5 (no cargo invocations from the skill body) |
| **#7 — full error messages not paraphrase** | Ground-truth corpus stores the verbatim `error[E####]` log slice, never paraphrased. The axis-mapping rule (§6.3 #1) reads the verbatim code, not a summary. Per `.claude/lessons/feedback_lemmy_error_no_std_error.md` Case-enumeration discipline. | Task 4 (metrics schema) |
| **#10 — full struct definitions for ownership/lifetime errors** | When the audit flags axis 3 (trait-bound completeness), the evidence string must include the *full generic constraint clause* of both new code and sibling — not just the trait name. A paraphrase of `<A: GovernanceInboundActivity>` vs `<A: GovernanceInboundActivity + Sync>` loses the load-bearing token. | Task 2 (axis 3 sub-file) |
| **#12 — stage migrations before Rust schema types** | Tangentially relevant: the audit MAY need a future axis for migration-LIFO-ordering vs schema.rs alignment (the v1-federation-inbound-a §13 revert-list-extension gap, per fed-in-a retro). NOT in scope for v1 of this skill (Tasks 1-10), but flag as a candidate axis #7 for "Every Brehon major version" calibration review (§6.4). | Future cycle (axis #7 candidate) |

The planner reads the full best-practices doc + cross-checks each numbered practice against the skill scope. Anything else worth lifting goes in plan §10 with rationale; anything *not* lifted is recorded in plan §12 "NOT building" with one-line "why excluded".

---

## 8. Three things to borrow selectively from `actionbook/rust-skills`

The repo at `github.com/actionbook/rust-skills` (1,159 stars, MIT, active) is the closest existing thing to a Rust-skills-for-Claude-Code project. The advisor's appraisal (read it in full at the prior conversation turn) is: **don't install the plugin** — the "meta-cognition framework" is theatre, the imperative "MUST invoke rust-router FIRST" prompt style conflicts with `feedback_principles_not_rules`, skill-name collisions break Brehon's auto-trigger behaviour, and the content is shallow compared to Brehon's codebase-grounded `.claude/lessons/`. But **three things are worth borrowing selectively as patterns** — copy the structural idea, not the literal files.

### 8.1 The five LSP-driven skills as a reference for the audit's LSP-orchestration layer

The repo's v2.0.8 release added five LSP-restricted skills with `allowed-tools: ["LSP", "Read", "Glob"]`:

- `rust-call-graph` — `incomingCalls` / `outgoingCalls` for tracing function call hierarchies.
- `rust-symbol-analyzer` — `documentSymbol` / `workspaceSymbol` for project structure.
- `rust-trait-explorer` — `goToImplementation` for trait impl location.
- `rust-code-navigator` — `goToDefinition` / `findReferences` for cross-file lookup.
- `rust-refactor-helper` — LSP-based refactoring impact analysis.

**What to borrow:** the **LSP-tool invocation patterns** for locating the canonical sibling. Task 3 (`find-sibling.sh`) needs exactly this capability — given `(target_file, target_fn)`, find the nearest function with a similar signature in the same file, then the same module, then the same crate. The repo's `rust-symbol-analyzer/SKILL.md` (~50-100 lines) demonstrates the LSP-call shape; lift the pattern into `find-sibling.sh`. Also lift the **`allowed-tools` restriction discipline** — the conformance-audit skill should explicitly set `allowed-tools: ["LSP", "Read", "Grep", "Glob", "Bash"]` to prevent ambient tool drift (no `Edit`, no `Write` beyond the two report paths in §3.2, no `Agent` dispatch beyond explicit subagent calls). Each axis sub-file inherits the parent's restriction.

**What NOT to borrow:** the marketing framing ("meta-cognition", "trace through cognitive layers"), the auto-trigger keyword stuffing in `description:` fields, the `rust-router` skill (it would fight Brehon's `feedback_principles_not_rules` + the four-role advisor model). Cite the LSP-tool patterns in plan §10 as the inspiration; do NOT install the repo as a dependency.

### 8.2 The error-code → design-question routing tables in `m01-m07`

The repo's Layer-1 skills (`m01-ownership`, `m02-resource`, `m03-mutability`, `m04-zero-cost`, `m05-type-driven`, `m06-error-handling`, `m07-concurrency`) each carry a small tabulated mapping of specific compiler error codes to "design question to ask before the surface fix". Coverage in the current state: E0382, E0597, E0277, E0308, E0499, E0502, E0596, E0425. The repo's own `REVIEW.md` flags missing codes: E0106, E0133, E0204, E0271, E0282, E0283.

**What to borrow:** the **table shape** as the structural template for axis #3 (trait-bound completeness) and axis #1 (conn-type) sub-files. The conformance audit's axis #3 detection rule needs to map `error[E####]` codes to the specific trait-bound or type-mismatch the new code is missing relative to the sibling. The repo's tables give a free starting catalogue — cross-check against Brehon's lesson corpus (`feedback_lemmy_error_no_std_error.md` already enumerates Case A/B/C for the LemmyError class; cross-reference) to find Brehon-specific gaps. The candidate axis #7 (migration LIFO) in §7 above would extend this catalogue further.

**What NOT to borrow:** the literal skill content. Brehon's `.claude/lessons/` covers most of this with codebase-grounded depth the rust-skills repo does not have. The table *shape* is reusable; the table *contents* should come from Brehon's lessons, not the repo's generic tutorial material.

### 8.3 The Trace Up ↑ / Trace Down ↓ composition pattern

Every Layer-1/Layer-2 skill in the repo has explicit **Trace Up ↑** (to higher-layer design questions) and **Trace Down ↓** (to lower-layer implementation specifics) sections that cross-reference other skills. Example from `m05-type-driven`:

```
Trace Up ↑
| Situation | Trace To | Question |
| What types to create | m09-domain | What's the domain model? |
| State machine design | m09-domain | What are valid transitions? |

Trace Down ↓
"Need type-safe wrapper for primitives" ↓ Newtype: struct UserId(u64);
"Need compile-time state validation"   ↓ Type State: Connection<Connected>
```

**What to borrow:** the **explicit cross-reference shape** for the six axis sub-files in Task 2. Each axis sub-file should have a "Trace Up" (to the PMD project memory or ADR that defines the invariant — e.g. axis #4 traces up to ADR-015 and `feedback_lemmy_error_no_std_error`) and a "Trace Down" (to the specific Grep pattern or LSP call that detects the divergence). Brehon's lessons already do this implicitly via PMD `[[name]]` links and "See also" footers; the explicit arrow-headed Trace Up/Down headings are a nicer reader experience for the audit-report-consumer (advisor at retro time) who is skimming, not deep-reading.

**What NOT to borrow:** the three-layer "Domain → Design → Mechanics" doctrine the repo wraps the Trace Up/Down pattern in. Brehon's existing ADR/lesson/rule hierarchy already covers that semantic layering — adopting the doctrine would introduce a parallel taxonomy that conflicts with `.claude/rules/`. Just adopt the arrows and the cross-reference tables; skip the meta-framing.

---

## 9. Wiring into the existing system (advisor stage-shape changes)

Task 7 + Task 8 above modify `.claude/rules/advisor-orchestrator.md`. The specifics:

### 9.1 Prevention layer — brief-author time

Add a sub-section to advisor-orchestrator §3.1 stage-shape orchestration:

> **Brief authored, planning task not yet queued → conformance-audit checkpoint.** If the brief targets a file matching `crates/apub/activities/src/governance/**.rs` OR `crates/api/api/src/governance/**.rs` (Phase-6-bearing files), run `.claude/skills/brehon-conformance-audit/SKILL.md` with `target_scope = file <brief-named-file>` BEFORE running `/brehon-clarify`. The audit report is committed at `.claude/PRPs/reports/conformance-audit-<phase>-<date>.md`. Findings flagged Tier-1 (likely footgun) are folded into the brief's §3 Required reading + §4 Constraints before clarify-DQ entries are written.

### 9.2 Detection layer — retro time

Add a sub-section to advisor-orchestrator §3.9 verify gate:

> **Conformance-audit pass at retro time.** Before `/brehon-verify`, run the conformance-audit skill with `target_scope = phase-diff <phase-branch>`. Findings flagged Tier-1 (likely footgun) become §3 actions in the retro file. The metrics file at `.claude/PRPs/audit-metrics/<phase>.json` is updated with predictions + per-§6.3 ground truth. Per-sub-phase calibration runs Task 5's `compute-metrics.sh`.

### 9.3 §G4 classifier extension

Add a row to the §G4 classifier table (Task 8):

| Failure signature | Auto-fix | Source lesson |
|---|---|---|
| Conformance audit Tier-1 finding on `crates/apub/activities/src/governance/**.rs` | Catch-fire to user with the audit report + suggested per-axis fix. NOT auto-fix; the human-in-the-loop decides. | `feedback_mirror_phase6_convention_in_same_file.md` (authored in Task 9) |

---

## 10. Anti-patterns to avoid (planner-side)

Lessons the planner should specifically guard against in this plan:

- **Don't over-parallelize `[P]`.** Tasks 7 + 8 both modify `advisor-orchestrator.md` — serial only. Task 3 (find-sibling helper) is `requires:` of Task 2 (axis sub-files) because the axis sub-files cite the helper.
- **Don't ship the metrics script without the metrics schema.** Task 4 (schema) is `requires:` of Task 5 (script). Order matters.
- **Don't dogfood (Task 6) before Tasks 1-5 are landed and §15-green.** Dogfooding against fed-in-b is the integration test; needs the full skill assembled.
- **Don't make the skill an auto-trigger.** `description:` field should NOT use the imperative MUST/CRITICAL pattern from `actionbook/rust-skills`. The skill is invoked explicitly from advisor-orchestrator stage-shape (Task 7) or from `/brehon-verify` (existing). The auto-trigger pattern would fight Brehon's principles-not-rules discipline.
- **Don't write a generic Rust-conformance audit.** This skill is Brehon-specific and Phase-6-grounded. The six axes are the contract. If the planner sees temptation to add "general code-quality" axes (long-functions, deep-nesting, etc.), STOP — those belong in `.claude/skills/code-audit/`, not here.
- **Don't depend on `cargo` invocation from the skill body.** Per Brehon's Shape-G discipline and `pattern_cargo_feature_flag_propagation` — cargo runs are advisor-orchestrated (laptop session OR Shape-G dispatch); the conformance-audit skill reads existing cargo output from runlogs/DQ, never invokes cargo itself.

---

## 11. Sources the planner reads in full before authoring

1. **This document** (this file) — the substantive context.
2. **PMD `project_phase6_convention_divergence_class.md`** — the user-directive memory recording the six-axis checklist, the latent-footgun evidence, the standing "audit-tooling generalization" directive.
3. **`.claude/PRPs/reports/retro-harvest-2026-05-20.md`** — the §★ Phase-6 spotlight section + §E self-improving evaluation loop (the substantive design source for §6 of this guidance).
4. **`.claude/PRPs/reports/v1-federation-inbound-a-retro.md`** — the parent retro's §3 actions + §5 watch-items, especially the "structural HIGH" gitignored bm-verb artifact issue (out of scope for this skill, but a sibling pattern of "ground-truth artifact lifecycle").
5. **`.claude/PRPs/reports/session-retro-2026-05-20-fed-in-b-impl-phase-close.md`** — change #6 ("Generalize audit-tooling into a Brehon audit skill") + the Phase-6 convention-divergence carry-forward.
6. **`docs/research/Search for best practices when building with rust.md`** — the 15 practices; §7 above lifts six relevant ones; planner cross-checks.
7. **`actionbook/rust-skills` repo (read-only via GitHub)** — §8 above identifies three patterns to borrow selectively. Do NOT install the repo as a dependency. Pull the LSP-tool patterns from `skills/rust-symbol-analyzer/SKILL.md`, `skills/rust-call-graph/SKILL.md`, `skills/rust-trait-explorer/SKILL.md`; the error-code table shape from `skills/m04-zero-cost/SKILL.md`; the Trace Up/Down composition pattern from `skills/m05-type-driven/SKILL.md`. Cite each in plan §10.
8. **`.claude/lessons/`** — Glob all `feedback_*.md` matching keywords: `principles_not_rules`, `read_canonical`, `dogfood_slash_command_specs`, `runbook_audit_drift`, `verify_automated_reviewer_claims`, `lemmy_error_no_std_error`, `plan_stub_uniformity_with_canonical_sibling`, `multi_write_handlers_need_transactions`, `one_system_memory_in_repo`, `lesson_mirror_check`, `four_role_retro_signals`, `retro_not_report`.
9. **`.claude/rules/advisor-orchestrator.md`** — §3.1 stage-shape, §3.6 canonical-schema-first gate, §3.9 verify gate, §5.3 §G4 classifier. Tasks 7+8 modify this file; read in full first.
10. **`.claude/PRPs/templates/plan.template.md`** — canonical 20-section schema, FILES YAML block, §5.1 complexity factor table.
11. **`.claude/PRPs/plans/v1-federation-inbound-b.plan.md`** — the canonical sibling plan (most recent, same domain context). Read in full per `feedback_read_canonical_before_writing_spec`.

---

## 12. Pre-resolved facts (binding inputs to the plan — do NOT re-derive)

These are advisor-decisions captured here so the planner does not re-litigate them:

- **The skill is a SKILL, not a subagent.** `.claude/skills/brehon-conformance-audit/SKILL.md` per the Brehon skill discipline (read-only, advisor-invocable, no Junior dispatch). The advisor invokes it inline at brief-author and retro time. (Rationale: a subagent would burn a Junior task slot per audit run; the skill is a thin orchestration over LSP+Grep that runs in the advisor session in ~2-5 min, no Junior needed.)
- **No cargo runs from the skill body.** The skill reads existing cargo output from runlogs/DQ. New ground truth comes from §15 runs the advisor already orchestrates; the skill's `compute-metrics.sh` is the only script, and it reads JSON, never invokes cargo. (Rationale: cargo-runs-on-laptop per Shape-G-suspended + the EliteDesk OOM history; per-audit cargo invocations would compete for the laptop's cargo target/ directory with active impl runs.)
- **The six axes are v1; no axis #7 in this plan.** Future axes are added via lesson promotion at "Every Brehon major version" calibration review (§6.4). The plan's §12 (NOT building) explicitly enumerates the candidate-axis-#7 list (migration LIFO, e2e fixture-vs-handler conformance, etc.) for future-cycle visibility, but they are out of scope for this plan. (Rationale: the user directive 2026-05-19 specifically named the six axes; expanding the schema before the v1 skill ships against ground truth would inflate scope without evidence.)
- **The skill ships under the existing `validate-pending-laptop` DoD shape.** No Shape-G workflows; no cargo on GH-Actions for this plan. Per PMD `project_shape_g_suspended_2026_05_16` + DQ #229 (re-check 2026-06-01). The §15 DoD names the existing read-only validation gates (the skill writes `.claude/skills/`, `.claude/lessons/`, `.claude/PRPs/audit-metrics/` schema files + `.claude/rules/advisor-orchestrator.md` — no compilation needed; §15 is `bash scripts/brehon/cargo-check.sh --workspace --features full` against the trunk tip as a no-op sanity check + the SKILL.md frontmatter linter if one exists).

---

## 13. Out of scope (the plan's §12 entries)

The planner enumerates these in §12 with one-line "why excluded" rationale:

- **Auto-fix of any flagged divergence.** The skill surfaces; the human decides.
- **Cargo invocation from the skill body.** Existing runlogs/DQ entries are the ground-truth source.
- **A `webauthn-rs` step-up gate before running the audit.** No write surface; no step-up needed.
- **Generic Rust-quality axes** (function-length, nesting, naming, etc.) — those belong in `.claude/skills/code-audit/`.
- **A subagent variant** — the skill is inline-invoked by the advisor. (Rationale per §12 above.)
- **Axis #7 candidates** — listed for visibility; out of plan scope; added by future lesson promotion.
- **Auto-trigger on every Rust question** — Brehon's principles-not-rules discipline; explicit invocation only.
- **A `rust-router`-style "must invoke first" gate** — same reason.
- **Bundle the `actionbook/rust-skills` repo as a dependency** — appraisal in §8; three patterns borrowed selectively, no dependency.
- **A migration of existing `.claude/lessons/` content into the audit-skill axis sub-files** — sub-files cite lessons by path; lessons stay where they are.

---

## 14. Plan file path + commit subject

The future planner Junior task will produce:

- **Plan file:** `.claude/PRPs/plans/brehon-conformance-audit.plan.md`
- **Commit subject** (from the planning subagent's finalize-merge): `feat(plan): brehon-conformance-audit skill plan (six-axis sibling-conformance + self-improving metrics loop)`
- **Brief commit subject** (from the advisor when authoring the brief from this guidance): `chore(advisor): author brehon-conformance-audit planning brief`

---

_Authored by advisor session 2026-05-20 against trunk HEAD `410fedb44`. This document is advisory input to the future planning brief at `.claude/PRPs/briefs/brehon-conformance-audit-planning-1.md`, not the planning brief itself. The brief will be authored when the implementation session opens; this document compresses the substantive context into one place so the brief can be ≤200 lines._
