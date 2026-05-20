# brehon-conformance-audit planning brief

**Written**: 2026-05-20 by advisor session (laptop, brehon-fork CWD `C:/Users/barri/Developer/brehon-fork`, canonical checkout on `governance-v0` @ `432fc2679`).
**Subagent target**: `planning` (Opus 4.7 — see `.claude/agents/planning.md`).
**Worktree**: Junior cuts `junior/brehon-conformance-audit-planning-1` from `governance-v0` committed HEAD `432fc2679`. Plan file commits + pushes back to `governance-v0` at finalize.
**Authority anchor**: this brief SUPERSEDES `.claude/PRPs/reports/brehon-conformance-audit-planning-guidance-2026-05-20.md` (the prior advisory-input doc). User decisions 2026-05-20 captured in §0.1 are BINDING and pre-empt the guidance doc's open questions.
**Sub-phase target**: `brehon-conformance-audit` — a Brehon skill that catches the **Phase-6 convention-divergence defect class** (new code in a Phase-6-bearing file diverges from same-file canonical siblings across six well-defined axes, sometimes compile-clean and latent). Read-only deliverable; no `crates/**` writes; no migration; no Shape-G workflows; ships under `validate-pending-laptop` DoD.

---

## 0. Why this sub-phase exists (the design problem — read first)

Phase-6 federation work (sub-phases `v1-federation-inbound-a`, `-b`) produced a defect class the existing tooling does not catch:

**New code (a new handler / helper / wrapper) is authored alongside a canonical Phase-6 sibling doing the same job in the same file/module, and the new code diverges across one or more of six axes — sometimes the compiler catches it, sometimes (worse) it compiles cleanly and ships a latent footgun.**

Concrete evidence on `v1-federation-inbound-b` (PMD `project_phase6_convention_divergence_class.md`, retro `.claude/PRPs/reports/session-retro-2026-05-20-fed-in-b-impl-phase-close.md`, parent retro `.claude/PRPs/reports/v1-federation-inbound-a-retro.md`):

- **3 compile-caught divergences** (axes 1, 1, 3) → fix-impl-1 closed `cdff6f09d`.
- **1 latent footgun, `cargo check`-clean** (axis 4): `receive_remote_moderation_label:~735` used `.domain().map(str::to_string).unwrap_or_default()` → would have persisted `source_instance = ''` for any domainless remote actor. Phase-6 siblings four lines away hard-error `.domain().ok_or_else(|| LemmyErrorType::Unknown(...))?`. Caught only because the user explicitly asked for "a thorough product-grade interpretation" and the advisor ran a bespoke read-only subagent. Without that ask, it would have shipped.
- Phase-6's own retro (`.claude/PRPs/reports/phase-6-complete-report.md` §"What to change") independently identified that **layer merge gates ran only `cargo check + clippy + test --no-run`**, missing 3 contamination flakes until Layer 5 — adjacent failure class (gate insufficient to catch sibling-divergence at the right cadence). E2e gating is OUT of `-b` scope per §0.1.5 below; the conformance-audit skill addresses the static-divergence half of that retro lesson.

The reactive per-incident fixes closed the specific instances. The defect *class* remains uncaught by the existing audit assets:

- `~/.claude/plugins/.../security-auditor.md` — generic OWASP/CWE checklist; no notion of "diff a new handler vs in-repo sibling".
- `.claude/skills/code-audit/SKILL.md` — language-agnostic static-analysis aggregator (line counts, complexity); no sibling-conformance technique.
- `code-reviewer`, `silent-failure-hunter`, `type-design-analyzer` subagents — adjacent but none does the canonical-sibling-divergence pass.

**The deliverable** is a new Brehon-specific skill at `.claude/skills/brehon-conformance-audit/SKILL.md` that runs at brief-author time (prevention) and at retro time (detection), augmented by **per-module Clippy `disallowed_methods`/`disallowed_types` gates in federation paths** that mechanically catch the axis-4-class footgun at the compiler level. Paired with a closed-loop metrics scheme that turns each run into training data for the next run.

---

## 0.1 Pre-resolved facts (READ BEFORE §1 — these are BINDING; do NOT re-derive or file blockers for them)

The advisor resolved these in `AskUserQuestion` exchanges on 2026-05-20. They pre-empt round-trips the planner would otherwise have to make. They SUPERSEDE the 2026-05-20 planning-guidance doc's open questions.

### 0.1.1 — PRECON-1 — Plan shape: SKILL (six axes + metrics loop) + parallel CLIPPY ENFORCEMENT TRACK

**User decision 2026-05-20 (Q1 — Plan shape):** "Extend the 6-axis skill with Clippy/Dylint enforcement (Recommended)". Specifically: keep the skill as the orchestration spine (sibling-diff + 6 axes + metrics loop), AND add a parallel static-analysis track. Dylint is DEFERRED (PRECON-7 below). Other Rust-research items (cargo-nextest, rust-analyzer MCP wider scope, CodeRabbit CLI, `ENABLE_PROMPT_CACHING_1H=1`) are DEFERRED to follow-up plans EXCEPT `rust-analyzer-mcp` install (PRECON-3 — the skill's sibling-detection layer needs it).

**Binding consequence for the plan:** §13 has TWO sub-tracks fused into one plan:

- **Track A — the SKILL (sibling-diff orchestration):** `.claude/skills/brehon-conformance-audit/SKILL.md` + six axis sub-files + `find-sibling.sh` helper + metrics schema + `compute-metrics.sh` + dogfood + stage-shape wiring + lesson files. The skill is read-only, invokes no cargo, runs in the advisor session.
- **Track B — the CLIPPY ENFORCEMENT (mechanical disallowed_methods/types):** a single `clippy.toml` at repo root (or extend if one exists) + per-module `#![deny(clippy::disallowed_methods)]` attributes in three federation modules ONLY (per PRECON-4). The compiler catches the axis-4-class footgun mechanically; the skill catches the judgment/sibling-diff classes the compiler cannot.

The two tracks are independent and can ship as `[P]`-able task cohorts where FILES YAML disjointness holds. They are mutually reinforcing: Clippy gates the well-known mechanical patterns (workspace-wide CI signal); the skill catches new patterns the compiler does not yet have rules for (axis schema extends via lesson promotion, not ad-hoc).

### 0.1.2 — PRECON-2 — E2e gating is OUT of scope

**User decision 2026-05-20 (Q2 — E2e gating):** "Lets defer this to the follow up implementation plan that will happen after the first run evaluation." Phase-6's "layer gates missed 3 e2e flakes" lesson is real but it is `cargo test --test e2e` plumbing, NOT static-divergence detection. Mixing it into the conformance-audit plan dilutes the audit's read-only contract.

**Binding consequence:** the plan's §12 "NOT building" MUST explicitly enumerate "no e2e pre-merge gate changes" and note the follow-up trigger ("after the first calibration cycle's metrics land, a separate plan reconsiders e2e-gating tightening"). The skill body never invokes `cargo test`; the metrics loop READS existing §15 e2e outputs from `.claude/runlog/` and DQ `validate-pending-laptop` entries, never re-runs them.

### 0.1.3 — PRECON-3 — LSP integration: rust-analyzer-mcp INSTALL + actionbook patterns BORROWED selectively

**User decision 2026-05-20 (Q3 — LSP integration):** "rust-analyzer MCP install + actionbook patterns selectively borrowed (Recommended)".

**Binding consequences:**

- A §13 task installs `rust-analyzer-mcp` via `cargo install rust-analyzer-mcp` and wires it into `.mcp.json.example` (the committed template) with an entry pointing at the canonical `rust-analyzer` binary path (`rustup component add rust-analyzer` is a prerequisite documented in the task body). Per `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md` + multi-lane-worktree §"PMD is cross-lane shared", each lane's gitignored `.mcp.json` MUST be bootstrapped from `.mcp.json.example` AND keep its canonical absolute paths (PROJECT_MEMORY_DB, etc.) verbatim — only `PROJECT_ROOT` varies per worktree. The new `rust-analyzer` MCP server entry's command is the rustup-resolved binary path; no per-lane variance needed.
- Tasks 2-3 (axis sub-files + `find-sibling.sh`) USE real LSP calls (`mcp__rust-analyzer__definition`, `_references`, `_hover`, `_workspace_diagnostics`, `_documentSymbol`) for sibling detection where available. Grep is the documented fallback (per zeenix/rust-analyzer-mcp known limitation: code actions may return empty arrays before indexing finishes).
- actionbook/rust-skills patterns BORROWED structurally (NOT installed as a dependency):
  - **`allowed-tools: [LSP, Read, Grep, Glob, Bash]` restriction** on the SKILL.md frontmatter + every axis sub-file. No `Edit` or `Write` beyond the two declared output paths (§3.2). No `Agent` dispatch.
  - **Trace Up ↑ / Trace Down ↓ cross-reference shape** in every axis sub-file (Trace Up → PMD/ADR/lesson that defines the invariant; Trace Down → the specific Grep pattern or LSP call detection method).
  - **Error-code → design-question table shape** for axis #3 (trait-bound completeness) and axis #1 (conn-type) sub-files — the table SHAPE is the actionbook pattern; the CONTENT comes from Brehon's `.claude/lessons/feedback_lemmy_error_no_std_error.md` and the Phase-6 evidence.
- actionbook patterns NOT borrowed: the "meta-cognition framework" framing; the auto-trigger keyword-stuffing in `description:` fields; the `rust-router` skill; the imperative-MUST tone. Brehon's `feedback_principles_not_rules` discipline overrides.

### 0.1.4 — PRECON-4 — Clippy scope: per-module `#![deny()]` in federation modules ONLY

**User decision 2026-05-20 (Q1-followup — Clippy scope):** "Per-module #![deny()] in federation modules only (Recommended)".

**Binding consequences:** the Clippy enforcement track (Track B per PRECON-1) applies `#![deny(clippy::disallowed_methods)]` (and where relevant `#![deny(clippy::disallowed_types)]`) to **exactly these three module roots**:

1. `crates/apub/activities/src/governance/` — the wrapper + receivers + per-handler patches (the PRD-named home for axis-4 footguns).
2. `crates/api/api/src/governance/` — the Phase-6 admin/governance API handlers (sanction-vote, jury-quorum, etc.) where multi-write transactions live.
3. `crates/db_schema/src/source/governance/` — the federation_peer + federation_inbox_* + remote_* + governance_log models where insert-form validation and `governance_log::append` live.

**`clippy.toml` ships disallowed entries that apply ONLY when a downstream module opts in via `#![deny(clippy::disallowed_methods)]`** (Clippy's `disallowed_methods` is path-resolved globally, but the *enforcement level* is per-module via the deny-attribute). Per-module deny — NOT workspace-wide warn — keeps noise out of non-federation paths.

**The seed `clippy.toml` entries** (the planner refines exact `path:` resolution against the workspace; cite the lesson `feedback_lemmy_error_no_std_error.md` Case-A/B/C enumeration as the recipe-source):

```toml
# clippy.toml (repo root or extend existing)
disallowed-methods = [
  { path = "core::option::Option::unwrap_or_default",
    reason = "Use .ok_or_else(|| LemmyErrorType::*) for required federation fields. See feedback_lemmy_error_no_std_error.md and project_phase6_convention_divergence_class.md axis #4." },
  { path = "core::result::Result::unwrap_or_default",
    reason = "Use ? or .map_err with explicit error type. See feedback_lemmy_error_no_std_error.md." }
  # The planner adds further entries per axis-1 (diesel sync RunQueryDsl misuse) and axis-3 (Send-bound stripping) as the dogfood (Task 6) confirms each pattern's path resolves cleanly.
]

# disallowed-types entries follow same shape; the planner adds AsyncPgConnection-direct-use bans per axis #1
# if the dogfood finds them necessary beyond what the skill catches.
```

**Workspace-wide CI:** existing CI already runs `cargo clippy --workspace --no-deps --features full -- -D warnings` per `.claude/lessons/feedback_clippy_test_style.md`; the per-module `#![deny]` attributes turn the clippy.toml entries into hard errors within the federation module roots. No new CI workflow required.

### 0.1.5 — PRECON-5 — Plan-authoring: this brief authors the full planning brief; Junior planning task authors the actual plan file

**User decision 2026-05-20 (Q-followup — Plan authoring):** "Author full brief now; queue Junior planning task for the actual plan". Standard Brehon four-role discipline preserved.

**Binding consequence:** this brief at `.claude/PRPs/briefs/brehon-conformance-audit-planning-1.md` is the COMPLETE planning input. The advisor commits + pushes this brief to `governance-v0`. The advisor then runs `/brehon-clarify` against this brief, resolves clarify-DQs, and queues the planning Junior task. The Junior planning subagent (Opus 4.7) authors `.claude/PRPs/plans/brehon-conformance-audit.plan.md` per the §2 contract below.

### 0.1.6 — PRECON-6 — Metrics: full §6 schema + cadence as specced

**User decision 2026-05-20 (Q4 — Metrics loop):** "Ship full section-6 metrics + cadence as specced (Recommended)".

**Binding consequence:** §13 ships:

- The metrics JSON schema (`.claude/skills/brehon-conformance-audit/audit-metrics.schema.json`).
- The four metrics formulas (precision/recall per axis; lead time; latent-footgun catch rate) verbatim per the 2026-05-20 guidance §6.2.
- The four ground-truth attribution rules verbatim per the 2026-05-20 guidance §6.3.
- The three calibration cadences (per-sub-phase; every 3 sub-phases; every Brehon major version) per 2026-05-20 guidance §6.4.
- The first calibration data point against fed-in-b (the dogfood; per 2026-05-20 guidance §6.5).

This is the **single biggest novel contribution** of the plan — no published Claude Code precision/recall harness exists per the Rust research §E1. The plan ships it from day one rather than deferring it for ~2 months of data accumulation. New metrics are added only via lesson promotion at the "every Brehon major version" calibration review (not ad-hoc).

### 0.1.7 — PRECON-7 — Dylint deferred

**User decision 2026-05-20 (Q-followup — Dylint):** "Clippy only for v1; Dylint deferred (Recommended)".

**Binding consequence:** §12 "NOT building" enumerates Dylint with the rationale: Clippy `disallowed_methods` covers the axis-4 mechanical recipe; the skill itself (sibling-diff via LSP+Grep) handles axis-1 conn-type walks; Dylint adds a Rust nightly tooling dep + custom-lint authoring overhead exceeding v1 value. Reconsider at the "every Brehon major version" calibration review.

### 0.1.8 — PRECON-8 — Dogfood: both historical (Finding 6.1 retroactive) + current tip

**User decision 2026-05-20 (Q-followup — Dogfood scope):** "Both — re-find historical Finding 6.1 + new divergences (Recommended)".

**Binding consequence:** the dogfood task (Task 6 in the suggested decomposition; planner refines) runs the skill against TWO snapshots of `v1-federation-inbound-b`:

1. **Pre-fix-impl-3 tip** (the SHA before fix-impl-3 landed — the advisor identifies the exact SHA via `git log --oneline phase-v1-federation-inbound-b | grep -B1 "fix-impl-3"` at plan time and pins it in §13's Task-6 brief). This snapshot still has Finding 6.1 (axis-4 latent `unwrap_or_default`). The skill MUST flag it. Confirms axis-4 detection works against known ground truth.
2. **Current merged tip** (`governance-v0`'s most-recent fed-in-b merge SHA). This snapshot has Finding 6.1 closed; the skill should NOT flag axis-4 on `receive_remote_moderation_label`. Confirms zero false-positives on legitimate closed work.

The dogfood report at `.claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-dogfood-<date>.md` documents both runs. The metrics file at `.claude/PRPs/audit-metrics/v1-federation-inbound-b.json` captures both runs as historical ground-truth data — this IS the first calibration data point (§6.5 in the 2026-05-20 guidance).

### 0.1.9 — PRECON-9 — Six axes are FIXED in v1

The six axes from `project_phase6_convention_divergence_class.md` are the v1 schema contract. The planner does NOT invent a seventh axis. Candidate axis-#7s (migration LIFO, e2e fixture-vs-handler conformance) are recorded in §12 "NOT building" with one-line "why excluded" rationale and the trigger ("every Brehon major version" calibration review). New axes are added via lesson promotion, NOT ad-hoc.

The six axes (the plan's §10 lifts this table VERBATIM):

| Axis | What it means | Phase-6 instance | Detection method |
|---|---|---|---|
| **1. Conn-type / tx-boundary** | Does the new fn take `&mut DbConn<'_>` (starts tx via `.run_transaction`) or `&mut AsyncPgConnection` (must already be inside a tx)? | fix-impl-1 (#337): 2 helpers took `&mut AsyncPgConnection` but called `.run_transaction` (method on `&mut DbConn`); siblings 4 lines away took `&mut DbConn` | Grep file for `.run_transaction(` callers; check receiver type vs sibling receiver type. LSP: `_hover` on receiver to confirm type. |
| **2. Append reborrow shape** | In-tx `governance_log::append` calls must reborrow `&mut (&mut *conn).into()` exactly | Held (byte-conformant) | Pattern-grep against the canonical 4-token sequence |
| **3. Trait-bound completeness** | `#[async_trait]` methods with `Sync`-requiring bodies need `A: Sync` on the generic | fix-impl-1: `wrap_governance_inbound<A: GovernanceInboundActivity>` missing `+ Sync`; compile-caught | Compiler is the oracle; planning-time MIRROR stub should compile-check (`cargo check --workspace --features full` reads existing runlog/DQ, never invoked by the skill) |
| **4. Error idiom at trust boundary** | `.domain().ok_or_else(\|\| LemmyErrorType::Unknown(...))?` (hard-error) vs `.unwrap_or_default()` (silent empty-string) | **Finding 6.1**: `receive_remote_moderation_label:~735` used `.unwrap_or_default()` → persists `source_instance = ''`. Compiled cleanly. Latent data-integrity footgun. | **Sibling-diff**: locate same-file sibling doing same validation; flag every divergence where new code is *weaker* than sibling's enforced contract. Compiler-mechanical via Clippy `disallowed_methods` (Track B) at the trust-boundary modules. |
| **5. Conn acquisition idiom** | `let conn = &mut get_conn(pool).await?;` vs improvised variants | Held (byte-conformant) | Pattern-grep `let \w+ = &mut get_conn(` |
| **6. ADR-015 pseudonym handling for remote actors** | `None` for remote actors (no pseudonymisation possible) | Held | Pattern-grep `actor_pseudonym` + remote-actor type |

---

## 1. Role + dispatch line

`[role:planning] brehon-conformance-audit plan — six-axis sibling-conformance skill + federation Clippy track + self-improving metrics loop`

The actual create-task description (single line, <100 chars, per `.claude/rules/advisor-orchestrator.md` Junior task description template):

```
[role:planning] brehon-conformance-audit plan — see .claude/PRPs/briefs/brehon-conformance-audit-planning-1.md
```

Everything else lives in this brief.

---

## 2. Scope — what to produce

Produce **one plan file** at **`.claude/PRPs/plans/brehon-conformance-audit.plan.md`** following `.claude/PRPs/templates/plan.template.md`'s 20-section schema literally.

### 2.1 What this sub-phase ships (the deliverable surface)

The plan's §13 task list MUST cover exactly the following two tracks fused into one plan, and NOTHING beyond it (cross-check every task against §12 "NOT building" — anything not listed below is OUT):

**Track A — the SKILL (orchestration spine):**

1. **`.claude/skills/brehon-conformance-audit/SKILL.md`** — frontmatter + body. Frontmatter declares `allowed-tools: [LSP, Read, Grep, Glob, Bash]` (PRECON-3 — no `Edit`/`Write` beyond declared outputs; no `Agent` dispatch). Body declares the three input modes (`phase-diff <branch>`, `file <path>`, `fn-list <file:fn>,...`), the two output paths (per-run audit report + per-run metrics file), and the six-axis schema (lift §0.1.9 table VERBATIM). `description:` field is PLAIN-LANGUAGE — no auto-trigger keyword-stuffing, no imperative MUST/CRITICAL tone (PRECON-3 + `feedback_principles_not_rules`).

2. **Six axis sub-files** at `.claude/skills/brehon-conformance-audit/axes/{1-conn-type,2-append-reborrow,3-trait-bound,4-error-idiom,5-conn-acquisition,6-adr-015}.md`. Each sub-file declares the detection method (Grep pattern / LSP call / Read pattern), the evidence-string format (≤120 chars per the metrics schema), the Trace Up ↑ section (PMD/ADR/lesson defining the invariant), the Trace Down ↓ section (specific detection command). The axis sub-files inherit the parent SKILL.md's `allowed-tools` restriction. Per the 2026-05-20 guidance §8.1 (LSP restriction discipline) + §8.3 (Trace Up/Down).

3. **`.claude/skills/brehon-conformance-audit/scripts/find-sibling.sh`** — given `(target_file, target_fn)`, locates the nearest in-file (then in-module, then in-crate) function with a similar signature. Uses `mcp__rust-analyzer__documentSymbol` and `_workspace_diagnostics` where available; Grep + Read as documented fallback (PRECON-3 known limitation). Output: stdout one-line `(sibling_file, sibling_line, sibling_signature)` triple OR `NO_SIBLING_FOUND` on miss. Per `pattern_verify_before_trusting_shell_output` — exit codes are validated, not trusted.

4. **`.claude/skills/brehon-conformance-audit/audit-metrics.schema.json`** — JSON schema for per-run predictions file. Schema MUST encode: `schema_version: 1`, `scope`, `head_sha`, `run_at` (ISO 8601), `skill_version` (semver), `predictions[]` (each: `axis`, `risk_tier`, `target` (file:line), `sibling` (file:line), `evidence` ≤120 chars). Append-mode for `ground_truth_compile_caught[]` (populated post-§15) + `ground_truth_runtime[]` (populated post-CR-triage / post-merge bug fixes within 30 days). Per the 2026-05-20 guidance §3.2.

5. **`.claude/skills/brehon-conformance-audit/METRICS.md`** — explainer codifying the four ground-truth attribution rules verbatim from the 2026-05-20 guidance §6.3:
   - Rule 1: §15 failures count as ground truth only if `error[E####]` maps unambiguously to one of the six axes. Multi-axis count once per axis. Out-of-axis failures don't penalize recall.
   - Rule 2: CR findings count as ground truth only if `severity ≥ major` AND `bucket = fix-in-pr` AND finding text references sibling-conformance.
   - Rule 3: Post-merge bugs count only if fix-commit diff modifies an axis-relevant pattern in the same file within 30 days of merge.
   - Rule 4: False positives confirmed only by human verdict at retro.

6. **`.claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh`** — reads `.claude/PRPs/audit-metrics/*.json`, computes the four metrics (precision/recall per axis; lead time; latent-footgun catch rate) per the 2026-05-20 guidance §6.2, writes summary to stdout. Runs per-sub-phase at retro time AND every-3-sub-phases for trend detection. NO cargo invocation — reads JSON only. Per the 2026-05-20 guidance §12 + §10 anti-pattern #6 ("Don't depend on `cargo` invocation from the skill body").

7. **Dogfood task** — run the skill against TWO snapshots of `v1-federation-inbound-b` (PRECON-8): (a) pre-fix-impl-3 tip → must flag Finding 6.1 (axis-4); (b) current merged tip → must NOT flag axis-4. Writes the dogfood report at `.claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-dogfood-<YYYY-MM-DD>.md` AND seeds the first metrics data point at `.claude/PRPs/audit-metrics/v1-federation-inbound-b.json`.

**Track B — the CLIPPY ENFORCEMENT (mechanical static analysis):**

8. **`clippy.toml` at repo root** (or extend if one exists) — declares `disallowed-methods` and `disallowed-types` entries per PRECON-4. The planner reads any existing `clippy.toml` and merges; the planner does NOT clobber existing entries. Cite the source lesson (`feedback_lemmy_error_no_std_error.md` for axis-4 recipe; `feedback_clippy_test_style.md` for the existing workspace clippy discipline; `feedback_lemmy_migration_runner.md` for the migration-runner-disallowance precedent if relevant for axis-1 candidates).

9. **Per-module `#![deny(clippy::disallowed_methods)]` attributes** in EXACTLY these three module roots (PRECON-4):
   - `crates/apub/activities/src/governance/mod.rs` (or the crate's `lib.rs` if the governance module is `pub mod`-exported and the deny attribute should propagate). The planner reads the current `mod.rs` to choose the correct attribute location (module-scope vs crate-scope).
   - `crates/api/api/src/governance/mod.rs` (same decision shape).
   - `crates/db_schema/src/source/governance/mod.rs` (same decision shape).

**Track C — wiring + lessons + retro (mandatory closeout):**

10. **rust-analyzer-mcp install** — a §13 task installs `rust-analyzer-mcp` via `cargo install rust-analyzer-mcp` (the planner verifies installation availability before the dogfood task) AND updates `.mcp.json.example` (the committed template) with a `rust-analyzer` server entry. Per multi-lane-worktree.md §"PMD is cross-lane shared" — `.mcp.json.example` is the source of truth; per-lane `.mcp.json` (gitignored) is bootstrapped from it AND keeps canonical absolute paths verbatim. The rust-analyzer entry uses the rustup-resolved binary path (`rustup which rust-analyzer` resolves it; if `rustup component add rust-analyzer` is required, the task body documents the prereq command — non-blocking on the workstation since the laptop is the canonical cargo runner per `project_laptop_canonical_cargo_runner.md`).

11. **Wire into the advisor stage-shape** — edit `.claude/rules/advisor-orchestrator.md` adding:
    - **§3.1 prevention checkpoint** (between brief-author and impl dispatch): if the brief targets a file matching `crates/apub/activities/src/governance/**.rs` OR `crates/api/api/src/governance/**.rs` OR `crates/db_schema/src/source/governance/**.rs` (the three module roots from PRECON-4), run the skill with `target_scope = file <brief-named-file>` BEFORE `/brehon-clarify`. Tier-1 findings fold into brief §3 / §4 before clarify-DQ entries.
    - **§3.9 detection checkpoint** (at retro time, before `/brehon-verify`): run the skill with `target_scope = phase-diff <phase-branch>`. Tier-1 findings become §3 actions in the retro. Update the per-phase metrics file. Run `compute-metrics.sh` for per-sub-phase calibration.
    - **§G4 classifier extension** — add one row: "Conformance audit Tier-1 finding on `crates/apub/activities/src/governance/**.rs` (and the other two roots) → catch-fire to user with audit report + suggested per-axis fix; NOT auto-fix; human-in-the-loop decides." Cite the new lesson file from Task 13 below.

12. **Paired lesson files** — per `feedback_one_system_memory_in_repo.md` + `feedback_lesson_mirror_check.md` + `feedback_lesson_must_pair_with_structural_fix_when_fixable.md` (the structural fix IS the skill + the Clippy gate — the lesson cross-links them):
    - `.claude/lessons/feedback_mirror_phase6_convention_in_same_file.md` — defect class lesson. Cross-link `[[feedback_plan_stub_uniformity_with_canonical_sibling]]`, `[[feedback_lemmy_error_no_std_error]]`, `[[feedback_multi_write_handlers_need_transactions]]`, `[[project_phase6_convention_divergence_class]]`, `[[feedback_read_canonical_before_writing_spec]]`. Body: Phase-6 evidence (the 4 fed-in-b incidents); the six axes (lifted from §0.1.9); the structural fix (this skill + the Clippy gate); the brief-author checklist ("if your task creates a fn under one of the three module roots: read the same-file sibling first; cite it in §3 Required reading; rerun the skill at brief-time").
    - `.claude/lessons/feedback_plan_mirror_stub_must_compile_and_annotate_prelanded.md` — planning-side prevention. Planner authoring a multi-task plan whose Task N adds infra and Task N+1 first-calls it MUST include a temporary unit test (or assert call site) that exercises Task N's signature so the compiler proves call-site discipline at Task N's §15, NOT Task N+1's. Strip the assert in the same task that strips `#[expect(dead_code)]`. Cross-link `[[feedback_dead_code_shields_latent_type_errors]]` (if/when that lesson lands per the 2026-05-20 session retro change-#3).

13. **Standard final §13 retro task** per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`.

### 2.2 Scope boundary — what is explicitly NOT in this sub-phase

Per the 2026-05-20 user decisions §0.1.1–0.1.9. The plan's §12 "NOT building" MUST enumerate these (each with one-line "why excluded" rationale and the future trigger if any):

- **No e2e pre-merge gate changes** (PRECON-2). Phase-6 retro lesson is real but separate scope. Trigger: post-first-calibration-cycle metrics review.
- **No cargo-nextest adoption** (deferred per PRECON-1). Trigger: follow-up plan after first calibration cycle.
- **No CodeRabbit CLI integration** (deferred per PRECON-1). Trigger: same.
- **No `ENABLE_PROMPT_CACHING_1H=1` worker-env rollout** (deferred per PRECON-1). Trigger: separate token-economy plan.
- **No Dylint custom-lint adoption** (PRECON-7). Trigger: every-Brehon-major-version calibration review.
- **No seventh axis** (PRECON-9). Candidate axis-#7s explicitly listed:
  - Migration LIFO discipline (the fed-in-a §13 revert-list-extension gap pattern; per `feedback_lemmy_migration_runner.md`).
  - E2e fixture-vs-handler conformance (the Phase-6 `sanction_notice_round_trip` Allowlist-fixture pattern from fed-in-b §11.4).
  - Schema-rs alignment with applied migrations.
  - `#[cfg(feature = "full")]` gate consistency (per `feedback_features_full_workspace_only.md`).
  Each is recorded with one-line rationale ("not v1; promote via lesson-then-axis at every-major-version review").
- **No auto-trigger on every Rust question** (PRECON-3 + `feedback_principles_not_rules`). The skill is explicitly invoked from advisor-orchestrator stage-shape OR from `/brehon-verify`. Trigger: never (anti-pattern).
- **No `rust-router`-style "must invoke first" gate** (PRECON-3). Same rationale.
- **No actionbook/rust-skills as a dependency** (PRECON-3). Three patterns BORROWED structurally; no install. Trigger: never (anti-pattern).
- **No subagent variant** (the skill is inline-invoked by the advisor — burning a Junior task slot per audit run defeats the cost model). Trigger: never (anti-pattern).
- **No `webauthn-rs` step-up gate before audit invocation** (no write surface; no step-up needed).
- **No auto-fix of any flagged divergence.** The skill surfaces; the human decides. Trigger: never (anti-pattern per `feedback_principles_not_rules`).
- **No generic Rust-quality axes** (function-length, nesting, naming, etc.). Those belong in `.claude/skills/code-audit/`, not here. Trigger: never (anti-pattern).
- **No cargo invocation from the skill body.** Existing runlogs/DQ entries are the ground-truth source; compiler is the oracle, NOT a callee. Trigger: never (anti-pattern per `pattern_cargo_feature_flag_propagation` + Shape-G suspended state).
- **No migration of existing `.claude/lessons/` content into axis sub-files.** Sub-files cite lessons by path; lessons stay where they are.
- **No new ADR.** The skill respects existing ADRs (especially ADR-006 advisory-only, ADR-013 emergency-remove, ADR-014 vanilla-Lemmy interop, ADR-015 pseudonymisation). Axis #6 specifically encodes ADR-015. Trigger: planner files `kind: "blocker"` if a new ADR appears necessary.

**Hard out-of-scope (per the broader v1 Brehon platform):** auto-apply (v3 / ADR-006), reputation portability (v2/v3), cross-instance jury (v3), OPA federation policy (v2), federation discovery (v2). The audit reads federation code; it does NOT change federation behaviour.

### 2.3 Open questions — clarify-gate inputs

The 2026-05-20 user decisions resolved the major open questions. Residual ambiguities the planner MUST surface (the advisor will run `/brehon-clarify` against this brief and raise `kind: "clarify"` DQs as needed):

1. **`clippy.toml` path-resolution edge cases.** The `path = "core::option::Option::unwrap_or_default"` entry depends on Clippy's path-resolution rules — re-exported types (`std::option::Option` vs `core::option::Option` vs Lemmy's own `Option` re-exports) may or may not be caught. The planner verifies at plan time via a small probe (read existing `clippy.toml` workspace-wide entries; `grep -rn "disallowed_methods\|disallowed-methods" crates/`) and proposes either: (a) ship the seed entries as-is and rely on the dogfood (Task 7) to confirm coverage; (b) ship a broader path-pattern that catches all Option re-exports. **Lean (NOT binding):** option (a) — dogfood is the integration test; over-tuning paths before evidence is premature.

2. **`rust-analyzer-mcp` availability + `.mcp.json` lane-bootstrap UX.** Per `feedback_phase_lane_worktree_bootstrap_checklist.md`, `.mcp.json` is gitignored per-lane; the committed template is `.mcp.json.example`. Adding rust-analyzer to the example template requires every active lane (currently `governance-v0` canonical + `phase-v1-federation-inbound-b` if still active + `tooling-local-validation`) to re-bootstrap. The planner names this in §10 (the path-bootstrap cost) and either: (a) ships the example template update + a one-line bootstrap-note in the lane-checklist lesson; (b) defers rust-analyzer-mcp install to a separate follow-up plan and lets v1 of the skill use Grep/Read only. **Lean (NOT binding):** option (a) — the install is `cargo install` (one command) + a one-line `.mcp.json.example` edit; the bootstrap cost is per-lane, paid once, and the lane checklist already requires `.mcp.json` re-bootstrap on lane creation.

3. **Storage location for per-run audit metrics.** The 2026-05-20 guidance §3.2 names `.claude/PRPs/audit-metrics/<phase>.json` (gitignored). The planner confirms whether `.claude/PRPs/audit-metrics/` should be added to `.gitignore` (likely YES — it is runtime journal data like `.claude/runlog/`'s gitignored sub-paths) AND whether the per-sub-phase metrics summary lands in the retro file (visible in git) — that's the 2026-05-20 guidance §6's intent (summary in retro; full per-prediction journal gitignored). The planner explicitly states this in §13's Task-4 description (`.gitignore` adds `.claude/PRPs/audit-metrics/`).

4. **Compute-metrics.sh language choice.** The 2026-05-20 guidance says "bash/python". The planner picks one — likely bash + jq (per `feedback_postgres_jsonb_canonicalization.md` jq usage precedent) OR python (per `feedback_python_utf8_encoding_windows.md` + Windows-cross-platform considerations). **Lean (NOT binding):** python with `json.load(open(...))` and `ensure_ascii=False` on writes (per `feedback_json_dump_ensure_ascii_false.md`) — bash + jq has cross-platform quirks on Windows that the planner has hit before; python is portable across the laptop (Windows) + EliteDesk (Linux daemon) without per-platform shims.

5. **First-run baseline for the metrics file shape.** The dogfood (Task 7) is also the first metrics data point per PRECON-8. The planner specifies in §13's Task-7 description what fields are populated on a "dogfood run" vs a "production run" — they ARE the same shape, but the dogfood seeds `ground_truth_compile_caught` from the fed-in-b fix-impl-1 commit metadata (already on `governance-v0` history) RATHER than a live §15 invocation. The planner names the exact git commands to extract ground truth from history.

The advisor (not the planner) files `kind: "clarify"` DQs at the clarify-gate. Planner ambiguities are `kind: "blocker"` (for genuine open decisions) or `kind: "log"` (for choices the planner has made with rationale). Per `.claude/rules/decision-queue.md` hard refusals #6.

### 2.4 Plan file deliverable shape

- **One file:** `.claude/PRPs/plans/brehon-conformance-audit.plan.md`.
- **Follow `.claude/PRPs/templates/plan.template.md` 20-section schema literally.** §16a Stories mandatory.
- **PRECON-2 DoD shape — `validate-pending-laptop`, NOT Shape-G.** §15 names the cargo DoD commands verbatim with `--workspace --features full`. Do NOT author the Shape-G `.github/workflows/*.yml` per-workflow §15.6 section (dormant until 2026-06-01 per PMD DQ #229). Mirror `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` §15 as the laptop-shape exemplar (fed-in-a shipped under Shape-G suspension).
- **§15 expected commands** (the planner refines exact incantations against current `.gitignore` + scripts):
  - `bash scripts/brehon/cargo-check.sh --workspace --features full` — confirms the per-module `#![deny(clippy::disallowed_methods)]` attributes compile without breaking existing federation code (the Clippy track sanity check; if existing code violates a newly-added disallowed-method, that is a finding the plan §13 includes a remediation task for — NOT a §15 catch-fire; the plan §13 makes the disallowed entries land AFTER any existing-code remediation).
  - `bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings` — confirms the disallowed-methods entries fire as expected in the federation modules and stay silent elsewhere (the Clippy track's primary signal).
  - `cargo test --test e2e --no-run -p lemmy_server` — sanity: skill changes do not break test-link.
  - `bash .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh .claude/PRPs/audit-metrics/v1-federation-inbound-b.json` — confirms the metrics script runs end-to-end on the dogfood-seeded data without panic / non-zero exit.
  - Per `feedback_clippy_rerun_after_fix.md` — if clippy surfaces existing-code violations, the §13 remediation task lands BEFORE the disallowed-methods entries; the §15 sequence holds.

- **§5 complexity-factor breakdown table** per `feedback_complexity_score_pre_split.md`. **Pre-estimate: ~7-9** — moderately complex but contained:
  - §13 impl tasks above 5: +5 (Tasks 1-7 of Track A + Tasks 8-9 of Track B + Tasks 10-13 of Track C = ~13 tasks; the planner's exact count drives the +1/+2 increment).
  - Migrations: 0.
  - Crates touched: 0 (skill is `.claude/skills/` only; Clippy track touches `clippy.toml` + 3 per-module attribute additions — these are crate-attribute edits, not crate-internal logic changes).
  - e2e edits: 0 (the dogfood does NOT change e2e; it READS history).
  - ADR-affecting: 0.
  - Cargo budget: 0 (Shape-G suspended; this skill runs no cargo; §15 cargo runs are the existing laptop-mode commands).
  - **Estimated score: ~7-9**, comfortably below Sonnet's split threshold of 8 (border-line; the planner computes honestly via FILES-YAML-per-task heuristic).
  - If the score crosses 8, the planner files the split-or-proceed DQ per the template's §5 threshold. **Expected:** if it crosses, the natural split SEAM is Track A vs Track B vs Track C (the skill body / the Clippy enforcement / the wiring + lessons + retro). A staged-delivery split would land Track A first (the new file artifacts; lowest blast radius), Track B second (the federation-module-attribute edits; potentially flags existing-code violations needing remediation), Track C third (the advisor-orchestrator.md edits + lessons + retro). The planner proposes this split in the DQ; the advisor resolves at plan-approval.

- **§13 per-task `creates:` / `modifies:` FILES YAML block** mandatory (load-bearing for cohort dispatch + the §5 complexity heuristic). `[P]` markers only where FILES-YAML disjointness genuinely holds. **Critical dependency structure for `[P]`:**
  - **Track A tasks 2 (axis sub-files), 3 (find-sibling.sh), 4 (metrics schema), 5 (METRICS.md), 6 (compute-metrics.sh)** — likely `[P]` among themselves (all disjoint paths under `.claude/skills/brehon-conformance-audit/`). All depend on Task 1 (SKILL.md skeleton) so `requires: [1]`.
  - **Track A Task 7 (dogfood)** — depends on Tasks 1-6 landing; NOT `[P]` with them.
  - **Track B Task 8 (clippy.toml)** — likely `[P]` with Track A tasks (disjoint files); confirm via FILES YAML intersection. `requires: [0]` (Task 0 pre-flight only).
  - **Track B Task 9 (per-module deny attributes)** — touches `crates/apub/activities/src/governance/mod.rs` + `crates/api/api/src/governance/mod.rs` + `crates/db_schema/src/source/governance/mod.rs` (three files). `requires: [8]` (clippy.toml entries must exist before deny-attributes deny against them; if deny precedes disallowed-methods entries, the cargo build is briefly broken between Task 8 and Task 9 — order matters).
  - **Track C Task 10 (rust-analyzer-mcp install + .mcp.json.example edit)** — disjoint from Tracks A/B; `[P]` candidate; `requires: [0]`.
  - **Track C Task 11 (advisor-orchestrator.md wiring)** — single file edit; NOT `[P]` with Task 7 (dogfood report references advisor-orchestrator §3.9 detection-checkpoint shape) or Task 13 (retro task may reference the wiring as a shipped change). `requires: [1, 8]` (the skill and clippy.toml must exist before wiring references them).
  - **Track C Task 12 (lesson files)** — two new lesson files; disjoint from everything else; `[P]` candidate; `requires: [7]` (cite the dogfood report).
  - **Track C Task 13 (retro)** — last; serial.

  The planner sets `requires:` arrays on every task with a cross-task dependency (per `.claude/rules/advisor-orchestrator.md` §4.1 step 4a — load-bearing to prevent the Cohort-A/Cohort-B isolation bug class).

- **§16a Stories** — every story names composing §13 tasks + the (laptop-mode) DoD checkpoint + Brief-Scope outputs. Likely **5 stories**:
  - **Story 1 — skill skeleton compiles**: SKILL.md + axis sub-files + find-sibling.sh + metrics schema + METRICS.md + compute-metrics.sh exist and pass a structural lint (frontmatter valid YAML; axis sub-files reference declared `allowed-tools`).
  - **Story 2 — Clippy federation gate fires**: clippy.toml entries + three per-module `#![deny]` attributes land; `cargo clippy --workspace --no-deps --features full -- -D warnings` passes (existing federation code is conformant OR a remediation task landed first); the disallowed-methods entries demonstrably fire when a probe-edit introduces a violation (the planner names the probe in Task 8's body).
  - **Story 3 — rust-analyzer MCP wired**: `cargo install rust-analyzer-mcp` succeeds; `.mcp.json.example` updated; `mcp__rust-analyzer__documentSymbol` callable from a probe (the planner runs the probe at Task 10 against a federation file and confirms LSP returns symbols).
  - **Story 4 — dogfood passes**: skill run against pre-fix-impl-3 fed-in-b SHA flags Finding 6.1 (axis-4); skill run against current merged tip does NOT flag axis-4 on `receive_remote_moderation_label`; report committed; metrics seed committed.
  - **Story 5 — wiring + lessons + retro shipped**: advisor-orchestrator.md edited (§3.1, §3.9, §G4); two lesson files exist and cross-link correctly; retro file authored per `feedback_retro_not_report.md`.

- **§4 watchpoints** — every entry cites a SPECIFIC file:line / function name / table at CURRENT HEAD (per `feedback_advisor_watchpoint_specificity.md`). The 6-watchpoint seed list is in §4.1 below.

- **§6 "Relationship to other v1 sub-phases"** — confirm: the audit is independent of any active impl sub-phase; it is a META-tool that audits OTHER sub-phases' code. It does not block fed-in-c (the next federation sub-phase) nor any active lane. The audit's first production use will be at fed-in-c brief-author time (and at fed-in-b retro retro-actively if the metrics data point lands in time).

**Commit only the plan file** (and any planner DQ entries — pushed immediately per `.claude/rules/decision-queue.md` "Mid-task visibility"). Plan-file commit pushes to `governance-v0` after the advisor's DoD smoke + watchpoint-specificity gates pass and the user approves (User Gate 1).

---

## 3. Required reading (in order, before drafting any plan section)

1. **This brief in full** — the substantive context + the PRECON decisions.
2. `.claude/agents/planning.md` — the planning subagent contract (model-enforcement, §13 FILES YAML, §5 split threshold, §16a Stories, hard refusals).
3. `.claude/PRPs/templates/plan.template.md` — canonical 20-section plan schema.
4. **`.claude/PRPs/reports/brehon-conformance-audit-planning-guidance-2026-05-20.md`** — the prior advisory-input doc this brief SUPERSEDES. Read it for the substantive design discussion (the six axes, the four metrics formulas, the three calibration cadences, the actionbook-pattern appraisals, the LSP-tooling rationale). PRECON decisions in §0.1 above OVERRIDE the open questions in the guidance doc; everything else in the guidance is correct context.
5. **PMD `project_phase6_convention_divergence_class.md`** — the user-directive memory recording the six-axis checklist, the latent-footgun evidence, the standing "audit-tooling generalization" directive.
6. **`.claude/PRPs/reports/session-retro-2026-05-20-fed-in-b-impl-phase-close.md`** — §"What to change" #6 ("Generalize audit-tooling into a Brehon audit skill") + the Phase-6 convention-divergence carry-forward.
7. **`.claude/PRPs/reports/v1-federation-inbound-a-retro.md`** — parent retro §3 actions + §5 watch-items.
8. **`.claude/PRPs/reports/phase-6-complete-report.md`** — the original Phase-6 retrospective. Read §"What surprised us" (Agent F's signature deviation; Agent G's token-budget degradation), §"What to change" (layer-gates ran only `cargo check + clippy + test --no-run`, missing 3 contamination flakes until Layer 5 — the adjacent failure class), §"What to carry forward" (the DQ #37 attribution incident → `decision-queue.md` Attribution-integrity rule, which the audit must respect when writing `answered_by`), §"Amendment — PR #46 CodeRabbit review cycle" (3 Critical findings caught by CR that local advisor + e2e missed — the cross-validator pattern the conformance-audit complements, not replaces).
9. **`docs/research/brehon-claude-code-rust-best-practices.md`** — the May 2026 best-practices research report. Read in full; specifically:
    - §"TL;DR Top 5" item #5 (Clippy `disallowed_methods` + Dylint for D4 federation trust-boundary lints) — the source of the Clippy track design.
    - §D1 (Same-file sibling conformance) — the canonical published evidence that no Claude-Code-specific audit skill for intra-file sibling walks exists; cite this as the "novel contribution" justification.
    - §D2 (`#[async_trait]` Send + Sync + 'static bounds) — the axis-3 mechanical recipe.
    - §D3 (Diesel + diesel-async connection type discipline) — the axis-1 mechanical recipe.
    - §D4 (Federation / ActivityPub trust-boundary `.unwrap_or_default()` detection) — the axis-4 mechanical recipe + the per-module deny-attribute pattern.
    - §E1 (precision/recall metrics for AI audits) — the harness rationale; the published precision/recall lacuna; the Sherlock-style severity-discipline cite.
    - §E2 (eval rubrics for Rust agents) — the calibration-discipline cite.
    - §G2 (rust-analyzer via MCP) — the `zeenix/rust-analyzer-mcp` install + tool inventory.
    - §G4 (CodeRabbit / Greptile) — deferred per PRECON-1; read for context only.
    - §B1-B4 (subagent orchestration) — the skill-vs-subagent rationale (the audit is a skill not a subagent per PRECON-1 + the 2026-05-20 guidance §12).
10. **`docs/research/Search for best practices when building with rust.md`** — the 15-practice generic Rust-Pi research doc. Read for context only; the Brehon-specific best-practices doc (#9 above) is the authoritative source. Practices #4 (filter compiler output), #5 (cargo check not cargo build), #6 (scope test runs to crate), #7 (full error messages not paraphrase), #10 (full struct definitions), #12 (stage migrations before Rust types) are the six the 2026-05-20 guidance §7 lifts as relevant.
11. **`.claude/PRPs/plans/v1-federation-inbound-a.plan.md`** — the structural-skeleton + discipline mirror for §1-§4/§5/§6/§15-laptop-DoD/§16a/§19. Read in full per `feedback_read_canonical_before_writing_spec.md` + `.claude/rules/advisor-orchestrator.md` §3.6 (canonical-schema-first gate).
12. **`.claude/PRPs/plans/phase-6-federation.plan.md`** — the §13 task-decomposition precedent for a "new artifact" sub-phase (Phase-6 shipped new federation artifacts; this skill ships new audit artifacts). Read for the multi-task layered-delivery shape.
13. **`.claude/rules/advisor-orchestrator.md`** — §3.1 stage-shape (Track C Task 11 edits this section), §3.6 canonical-schema-first gate, §3.7 dogfood gate (Task 7 satisfies this), §3.8 schema-changing-spec retrofit (the planner asks `AskUserQuestion` at the §3.8 trigger — this plan adds a NEW marker in `.claude/lessons/` and a NEW section in advisor-orchestrator.md, both forward-only; retrofit not needed; the planner CONFIRMS this in §10), §3.9 verify gate, §4.1 cohort dispatch (the planner sets `requires:` arrays rigorously), §5.3 §G4 classifier (Task 11 extends).
14. **`.claude/rules/decision-queue.md`** — schema-v2 (Tasks 1+ may file `kind: "blocker"` and `kind: "log"` entries; the dogfood Task-7 ground-truth seeding reads `validate-pending` + `validate-pending-laptop` historical entries from fed-in-b's `decision-queue.json` + archives).
15. **`.claude/rules/branch-manager.md`** — file ownership boundaries (the BM Junior task that opens the PR for this sub-phase MUST NOT touch the skill content; only PR title + body + runlog entries).
16. **`.claude/rules/multi-lane-worktree.md`** — §"PMD is cross-lane shared" + §"Worktree-aware DQ id discipline" (the lane this sub-phase runs in MUST use the canonical PMD path; the rust-analyzer MCP entry MUST be added to `.mcp.json.example` not a lane-local `.mcp.json`).
17. **`.claude/rules/pmd-search-strategy.md`** — Track A Task 1's SKILL.md `description:` field is a hybrid-search-friendly natural-language sentence describing WHAT the skill does (not auto-trigger keywords); the planner phrases it per this rule.
18. **`.claude/skills/code-audit/SKILL.md`** — the existing language-agnostic audit skill. Read in full to confirm the conformance-audit does NOT duplicate it; the conformance-audit is Brehon-Phase-6-specific and intra-file-sibling-driven; code-audit is language-agnostic and produces aggregate metrics. Cite the distinction in §6 of the plan.
19. **`.claude/skills/harness-audit/SKILL.md`** + `scoring-matrix.md` (if present) — context only; the harness-audit is a different scope (Brehon-platform-wide harness audit, not per-file conformance) but its scoring-matrix discipline (calibrated weights, anti-inflation) informs the four-metrics design.
20. **Phase-6 federation code at CURRENT HEAD** (`432fc2679`):
    - `crates/apub/activities/src/governance/inbox.rs` — `wrap_governance_inbound`, `receive_remote_moderation_label` (~line 735), the 5 check helpers, `receive_remote_sanction_notice` (~line 105), `receive_remote_trust_attestation` (~line 195), the `log_inbox_drop` helper. **Re-read the file in full at plan time** to confirm Finding 6.1 is closed at current HEAD (the planner uses this read as the dogfood Task-7 baseline).
    - `crates/apub/activities/src/governance/publish_{sanction_notice,trust_attestation,label}.rs` — the three per-handler patches.
    - `crates/db_schema/src/source/governance/{federation_peer,federation_inbox_dropped_log,federation_inbox_nonce,remote_moderation_label,remote_sanction_notice,federation_attestation,governance_log}.rs` — the model files the receivers write to. Confirm axis-2 (append reborrow shape) is byte-conformant across all `governance_log::append` call sites.
    - `crates/utils/src/error.rs` — the LemmyErrorType enum + the 6 federation error variants from fed-in-b. Confirm axis-4 sibling patterns (`.ok_or_else(|| LemmyErrorType::*)`) are byte-conformant across the federation receivers.
21. **`Glob .claude/lessons/feedback_*.md`** and Read every file whose name keyword matches: `lemmy_error_no_std_error`, `multi_write_handlers_need_transactions`, `async_pool_test_pattern`, `clippy_test_style`, `clippy_rerun_after_fix`, `read_canonical`, `dogfood_slash_command_specs`, `runbook_audit_drift`, `verify_automated_reviewer_claims`, `plan_stub_uniformity_with_canonical_sibling`, `one_system_memory_in_repo`, `lesson_mirror_check`, `lesson_must_pair_with_structural_fix_when_fixable`, `four_role_retro_signals`, `retro_not_report`, `principles_not_rules`, `advisor_watchpoint_specificity`, `pre_phase_dod_smoke_test`, `plan_dod_dry_run_at_write`, `cohort_validation_dependency_check`, `complexity_score_pre_split`, `mcp_canonical_pmd_path_enforce_at_session_start`, `phase_lane_worktree_bootstrap_checklist`, `verify_files_with_read`, `commit_aggressively_in_shared_repos`, `dead_code_shields_latent_type_errors` (if landed by then per session-retro-2026-05-20 change-#3), `json_dump_ensure_ascii_false`, `python_utf8_encoding_windows`, `postgres_jsonb_canonicalization`, `fix_impl_enumerate_all_callsites`, `fix_impl_pre_push_cargo_check`.

The §2.4 file-class mandatory-lesson injection (per `.claude/rules/advisor-orchestrator.md` §2.4) does NOT auto-fire on this PLANNING brief (the brief authors a plan, not impl code). The §2.4 table WILL auto-fire when the advisor later authors the IMPL briefs from this plan — the plan should make that injection mechanical by naming the file classes per task in the FILES YAML block.

---

## 4. Constraints — what the planner MUST enforce

### 4.1 Seed watchpoint list (the planner extends with file:line citations at plan time)

| # | Watchpoint | What to watch | Where to cite (current HEAD) |
|---|---|---|---|
| 1 | **The skill writes ONLY two output paths.** Per the 2026-05-20 guidance §3 + PRECON-3 `allowed-tools` restriction. Any task that writes outside the declared paths (`.claude/PRPs/reports/conformance-audit-*.md` and `.claude/PRPs/audit-metrics/*.json`) is a scope breach. | The two paths are the contract — the SKILL.md frontmatter declares them; no axis sub-file may write elsewhere. | `.claude/skills/brehon-conformance-audit/SKILL.md` frontmatter (Task 1); enforced per axis sub-file (Task 2). |
| 2 | **The six axes are FIXED.** Per PRECON-9. No axis #7 in this plan. Candidate axis-#7s recorded in §12 with "every-major-version" trigger. | Axis sub-file count = 6 (Task 2); the metrics schema's `axis` field enum is `["1","2","3","4","5","6"]` (Task 4). | `project_phase6_convention_divergence_class.md` PMD memory (the user-directive source). |
| 3 | **No cargo invocation from the skill body.** Per PRECON-2 + 2026-05-20 guidance §12 + the multiple anti-pattern citations. The skill READS existing cargo output from runlogs/DQ history; it never invokes cargo. | The SKILL.md body explicitly states this; `compute-metrics.sh` reads JSON only; `find-sibling.sh` uses LSP+Grep+Read, not `cargo check`. | `pattern_cargo_feature_flag_propagation.md` PMD pattern + `project_shape_g_suspended_2026_05_16` PMD project memory. |
| 4 | **Clippy `disallowed_methods` deny-attribute lands AFTER any existing-code remediation.** Per `feedback_clippy_rerun_after_fix.md`. If the planner's Task-8 probe (a probe-run of `cargo clippy --workspace --no-deps --features full -- -D warnings` with the proposed clippy.toml entries) surfaces existing federation-code violations, the plan §13 inserts a remediation task BEFORE Task 9 (the per-module deny attribute additions). The §15 sequence holds without breaking the build mid-plan. | Probe step inside Task 8 verifies no pre-existing violations OR enumerates them for a remediation task. | `crates/apub/activities/src/governance/`, `crates/api/api/src/governance/`, `crates/db_schema/src/source/governance/` module roots at HEAD `432fc2679`. |
| 5 | **No auto-trigger keyword-stuffing in `description:` field.** Per `feedback_principles_not_rules.md` + PRECON-3. The SKILL.md `description:` is one plain-language sentence describing what the skill does. The advisor invokes it explicitly via `/brehon-verify`, `/brehon-clarify`, or the stage-shape checkpoints from Task 11. | Frontmatter review at plan-approval. | `.claude/skills/brehon-conformance-audit/SKILL.md` frontmatter (Task 1). |
| 6 | **The dogfood (Task 7) is the integration test.** Per PRECON-8. Do NOT dogfood before Tasks 1-6 land and §15-green; do NOT skip the dogfood (it IS the calibration baseline). Two snapshots: pre-fix-impl-3 (must flag axis-4) + current merged (must NOT flag axis-4). | Task 7 `requires: [1, 2, 3, 4, 5, 6]`. | `.claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-dogfood-<date>.md` + `.claude/PRPs/audit-metrics/v1-federation-inbound-b.json`. |
| 7 | **Trust automated-reviewer claims against the compiler.** Per `feedback_verify_automated_reviewer_claims_against_compiler.md` (PR #132 cr-2 precedent). The skill's outputs are HYPOTHESES until the compiler proves them. Every "this is wrong" flag must be backed by either (a) a compile error the planner can reproduce OR (b) a sibling diff that shows the new code is weaker than an enforced contract. | Axis sub-files (Task 2) explicitly state this discipline in the Trace Up ↑ section of each axis. | Lesson cited verbatim in METRICS.md (Task 5). |
| 8 | **ADR-006 advisory-only is preserved.** The skill audits federation inbound code; it must NEVER suggest a "fix" that turns advisory-only inbound into auto-apply. Any divergence flag whose suggested-action would violate ADR-006 is a planner-side process breach and STOPs. | Axis sub-file for axis #4 (error idiom) + axis #6 (ADR-015) explicitly cross-link ADR-006. | `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` ADR-006. |

The planner expands this list with file:line citations at plan-time (per `feedback_advisor_watchpoint_specificity.md` — concept-only watchpoints are insufficient; every watchpoint cites a SPECIFIC table, file, function, or schema.rs line).

### 4.2 Behavioural constraints

- **The planning subagent NEVER writes Rust code, NEVER writes axis sub-file content, NEVER writes the SKILL.md body, NEVER writes the metrics scripts.** Those are impl-task deliverables, NOT plan deliverables. The plan SPECIFIES what each artifact contains (the contract) but does NOT author the artifact. Per `.claude/agents/planning.md`.
- **The planning subagent runs `/brehon-clarify` on this brief is the ADVISOR's job, not the planner's.** Planner ambiguities are `kind: "blocker"` (genuine open decisions) or `kind: "log"` (decisions made with rationale). Never `kind: "clarify"`. Per `.claude/rules/decision-queue.md` hard refusal #6.
- **Per-module deny-attribute additions are TINY edits** (`#![deny(clippy::disallowed_methods)]` is one line at the top of a mod.rs file). Track B Task 9's FILES YAML must reflect this (low blast-radius). The planner does NOT lump them into a "Track B megatask" with the clippy.toml authoring.
- **Mid-task DQ push discipline** per `.claude/rules/decision-queue.md` "Mid-task visibility" — every DQ entry the planner files MUST be committed AND pushed to the worker branch (the Junior task's worktree branch) immediately. The advisor's `git fetch` picks it up.
- **Attribution integrity** per `.claude/rules/decision-queue.md` Attribution-integrity — planner DQ entries use `from: "planner"`. The planner NEVER writes `answered_by: "advisor"` (a hard refusal).
- **Pre-push cargo discipline per `feedback_fix_impl_pre_push_cargo_check.md`** — the plan's §13 task instructions for Tasks 8+9 (Clippy track) MUST require the impl-task worker to run `bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings` LOCALLY before pushing the worker branch. Without this, the federation-module-deny lands but the worker doesn't notice it broke existing code until §15 — the same cycle-cost as the v1-RT-r1 fix-impl-3 incident.
- **Canonical-sibling-mirror discipline per `feedback_read_canonical_before_writing_spec.md`** — the plan's §13 task instructions MUST require any worker authoring an axis sub-file (Task 2) to FIRST Read the existing `.claude/skills/code-audit/SKILL.md` AND `.claude/skills/harness-audit/SKILL.md` AND `.claude/skills/post-task-retro/SKILL.md` as canonical Brehon-skill exemplars. The new skill's structural shape mirrors theirs; deviation requires rationale in plan §10.

### 4.3 Hard refusals (the planner MUST stop on these)

1. **NEVER propose a §13 task that adds a seventh axis to the schema.** PRECON-9 is binding.
2. **NEVER propose a §13 task that wires `cargo test` or `cargo check` invocation INTO the skill body.** Per multiple anti-pattern citations. Compiler is the oracle (post-§15), NOT a callee.
3. **NEVER propose a §13 task that ships an `Edit` or `Write` from the skill body outside the two declared output paths.** `allowed-tools` restriction violation.
4. **NEVER propose a §13 task that makes the skill auto-trigger on every Rust question.** PRECON-3 anti-pattern.
5. **NEVER propose a §13 task that installs actionbook/rust-skills as a dependency.** PRECON-3 — patterns BORROWED structurally; no install.
6. **NEVER propose a §13 task that re-runs §15 cargo to produce ground truth.** Ground truth comes from EXISTING runlog/DQ history.
7. **NEVER propose a §13 task that converts the skill into a subagent.** The advisor invokes inline; subagent variant is anti-pattern per cost model.
8. **NEVER propose a §13 task that touches `crates/**` business logic.** Only `mod.rs` ATTRIBUTE-ADDITION edits in the three federation-module roots are sanctioned (Track B Task 9); business logic stays untouched.
9. **NEVER propose a §13 task that touches Cargo.toml, Cargo.lock, or rust-toolchain.toml.** The Clippy track adds `clippy.toml` entries (one new file or one existing file extended); no manifest changes.
10. **NEVER propose adding the conformance-audit skill to ANY auto-loaded chain (CLAUDE.md import, agent definition import, hook).** Per PRECON-3 + the actionbook anti-pattern — explicit invocation only, via advisor-orchestrator stage-shape Task 11.

If the planner sees a contradiction between this brief and any other source (PRD, ADR, prior plan, lesson), STOP and file `kind: "blocker"`. Do NOT silently resolve.

---

## 5. Dogfood — pre-commit walkthrough (per `feedback_dogfood_slash_command_specs.md`)

This brief was dogfooded against `v1-federation-inbound-b` at HEAD `432fc2679` before commit, per `.claude/rules/advisor-orchestrator.md` §3.7 Dogfood gate (planning-stage class).

**What worked:**

- The six-axis schema (PRECON-9) cleanly maps each of the four fed-in-b incidents (the 3 compile-caught at fix-impl-1 + the 1 latent Finding 6.1) to one of axes 1/1/3/4. Zero incidents fell outside the schema.
- The Clippy `disallowed_methods` recipe for axis #4 (PRECON-4 + research doc §D4) directly catches `.unwrap_or_default()` on `Option<String>` at federation trust boundaries — the exact Finding 6.1 pattern. The per-module `#![deny]` discipline (NOT workspace-wide warn) keeps noise out of non-federation paths per PRECON-4 lean.
- The two-snapshot dogfood (PRECON-8) gives the metrics file a real ground-truth seed (pre-fix-impl-3 flagged + closed) without inventing data.
- The §13 task dependency structure (Track A serial-by-default + Track B `requires` Track A's SKILL.md + Track C wires last) prevents the Cohort-A/Cohort-B isolation bug class per `feedback_cohort_validation_dependency_check.md`.

**What didn't (and how the brief responds):**

- Path resolution for `clippy.toml`'s `path = "core::option::Option::unwrap_or_default"` is not pre-verified — re-exports may dodge the disallow. §2.3 ambiguity #1 surfaces this as a clarify-DQ candidate; the dogfood (Task 7) is the integration test that proves coverage.
- The §15 sequence for Track B (clippy.toml → deny attributes) creates a transient build-broken window if existing code violates a disallowed-method. Watchpoint #4 + the §15 sequence + the remediation-task-before-deny ordering (Track B Task 8 internal probe) handles this; the planner refines exact task-ordering at plan time.
- The metrics schema's `evidence` field is capped at 120 chars per the 2026-05-20 guidance §3.2; the planner verifies this is enough to convey "new: `.unwrap_or_default()` / sibling: `.ok_or_else(...)?`" + file:lines. If 120 is tight, the planner proposes 200 with rationale; minor schema change.

The dogfood completed in ~12 min (walkthrough of the brief against the four fed-in-b incidents + the federation-module structure at HEAD). Cost-benefit confirmed: ~12 min dogfood vs ~30-60 min remediation cost of a planning miss caught at plan-approval.

---

## 6. After the planner ships

1. **Junior planning task completes** — `.claude/PRPs/plans/brehon-conformance-audit.plan.md` is on `governance-v0` via Junior's finalize-merge.
2. **Advisor runs §3.4 DoD smoke test** — every command in plan §15 against current HEAD; non-cargo commands (the `compute-metrics.sh` invocation against a fixture) confirm executability; cargo commands run laptop-mode (per PRECON-2).
3. **Advisor runs §3.5 watchpoint-specificity gate** — every watchpoint in plan §4 cites a SPECIFIC file:line / function / table. Concept-only entries (e.g. "watch for axis drift" without naming the axis sub-file path) trigger a DQ for revision before plan approval.
4. **Advisor runs §3.7 dogfood gate** — confirms plan §1's pre-commit dogfood section is present.
5. **Advisor runs §3.8 schema-changing-spec retrofit gate** — this plan adds new artifact CLASSES (`.claude/skills/brehon-conformance-audit/`, `.claude/PRPs/audit-metrics/`, `clippy.toml` may be new). It does NOT change the shape of an existing artifact class. No retrofit needed; the planner explicitly states this in §10.
6. **User Gate 1 (plan approval)** — advisor surfaces the plan + the four gate outcomes (DoD smoke / watchpoint specificity / dogfood / schema-retrofit) to the user. User approves or requests revisions.
7. **On approval — bm-cut** — cut `phase-brehon-conformance-audit` from `governance-v0` HEAD via the `bm-cut` Junior task per `.claude/commands/bm/bm-cut.md` + `.claude/rules/branch-manager.md`.
8. **Impl tasks dispatch** — per the plan §13 cohort structure. Tracks A/B/C respect their `requires:` graph; cohort dispatch goes through the §4.1 sequence with YAML overlap check + `requires:` dependency check + budget check (Shape-G suspended, so budget binding only against laptop OOM; per PRECON-2 this skill ships zero new cargo budget so budget is non-binding).
9. **§15 laptop-mode validation per impl-task** — `validate-pending-laptop` DQ entries; advisor mutates per `.claude/rules/advisor-orchestrator.md` §5.2.
10. **Track A first-call discipline (per `feedback_dead_code_shields_latent_type_errors`)** — if any axis sub-file references a `find-sibling.sh` function or `compute-metrics.sh` entry-point that has no caller until a later task, the planner adds a temporary test/assert that exercises it at the authoring task's §15. PRECON-7 + Task 12's `feedback_plan_mirror_stub_must_compile_and_annotate_prelanded.md` lesson formalizes this for FUTURE plans; this plan ALSO obeys it.
11. **Dogfood (Task 7) is the user-visible gate** — the dogfood report + the metrics seed file land on the phase branch; advisor confirms (a) Finding 6.1 was flagged at the pre-fix-impl-3 snapshot, (b) current tip is clean, (c) the metrics file is JSON-valid.
12. **bm-pr → CR → triage → fix-in-PR → bm-merge** — standard Brehon pipeline per `.claude/rules/branch-manager.md` + the BM verb scripts. User Gates 3-5 fire normally.
13. **Retro task (Task 13)** — authored per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`. User Gate 6.
14. **`/brehon-phase-transition`** — closes the sub-phase, archives DQ entries per the archive policy at `.claude/rules/decision-queue.md`.

---

## 7. Definition-of-done for THIS planning brief

This brief is DONE when:

- This file at `.claude/PRPs/briefs/brehon-conformance-audit-planning-1.md` is committed to `governance-v0`.
- A clarify-gate pass has run (`/brehon-clarify .claude/PRPs/briefs/brehon-conformance-audit-planning-1.md`) and all clarify-DQs are resolved.
- The user has approved (or modified-then-approved) this brief's scope.
- The Junior planning task is queued with the matching dispatch line in §1.

The plan file itself is the Junior planning task's deliverable; that completion is downstream of this brief.

---

## 8. Commit + push

This brief commits as: `chore(advisor): author brehon-conformance-audit planning brief — supersedes 2026-05-20 guidance + integrates Rust best-practices research`

Per `.claude/rules/decision-queue.md` Attribution-integrity §Detection: the subject pattern matches `^(chore|docs)\((advisor|decision-queue)\)`.

Push to `governance-v0`. Then run `/brehon-clarify .claude/PRPs/briefs/brehon-conformance-audit-planning-1.md`.

---

_Authored by advisor session 2026-05-20 against trunk HEAD `432fc2679`. Supersedes `.claude/PRPs/reports/brehon-conformance-audit-planning-guidance-2026-05-20.md`. The four PRECON-1/2/3/4/5/6/7/8/9 decisions in §0.1 captured the user's 2026-05-20 directives; the planner does NOT re-derive them._
