# brehon-code-audit skill — design decisions (drafted 2026-05-14, body deferred)

**Status:** design captured; SKILL.md body intentionally deferred until the parallel session's audit report ships at `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md`. Authoring the skill body from real ground-truth output beats authoring from the prompt alone.

**Authored by:** governance-v0 advisor session 2026-05-14, companion to parallel session driving today's first-pass audit.

**Trigger for resumption:** the audit report exists at the path above (verify with `ls .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md`). When user next types `/skill-creator:skill-creator` or asks to "build the audit skill", this file is the design doc to follow.

---

## Decisions captured (user 2026-05-14)

### 1. Trigger phrasing

**Option chosen:** explicit invocation only.

Trigger phrases (encode in description):
- `/code-quality-audit`
- "run a code quality audit"
- "audit code quality"
- "check for spaghetti code"
- "review production-readiness of merged code"
- "find refactor candidates"
- "is shipped code clean"
- "audit Brehon code"

**Do NOT auto-trigger on:** new PRD planning, `/prp-core:prp-plan` invocation, phase start. User said the proactive trigger would be annoying when audit isn't needed. Discipline lives in the post-task-retro / phase-retro pattern (audit's findings get re-checked each retro), NOT in pre-emptive skill invocation.

### 2. Skill specificity

**Option chosen:** Brehon-specific.

- Skill path: `.claude/skills/brehon-code-audit/`
- Scope paths hardcoded:
  - `crates/api/api/src/governance/**/*.rs`
  - `crates/api/api_crud/src/governance/**/*.rs`
  - `crates/db_schema/src/source/governance/**/*.rs`
  - `crates/db_schema_file/src/source/governance/**/*.rs`
  - `crates/db_schema_file/src/enums.rs` (Brehon-added variants only)
  - `crates/db_schema_file/src/schema.rs` (Brehon-added sql_types + table! extensions)
  - `migrations/2026-*/{up,down}.sql`
  - `crates/server/tests/e2e.rs` (Brehon-authored fns; verify via `git blame`)
  - `crates/tools/seed_founders/`
  - `crates/api/api_common/src/governance/` (if exists)
- Lemmy upstream parity required (Lens 1 reads from `upstream/main`).
- 15 ADRs + 11 endpoints in CLAUDE.md are constraints; cite by number in findings.
- AGPL governance context: fork stays private pre-pilot; divergence-from-upstream is allowed but must be visible (Axis 4).

**Future-fork reuse:** if Brehon spawns a v2 fork or sibling fork (e.g. messaging crate becomes its own repo), build a generic variant then. Don't pre-pay the abstraction tax now.

### 3. Test cases

**Option chosen:** yes — test on today's audit output once it lands.

**Plan:**
1. Wait for `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md` to ship from parallel session.
2. Use it as the ground-truth reference for evals.
3. Test prompts simulate fresh sessions invoking the audit on Brehon governance code.
4. Assertions (objective, scriptable):
   - Report exists at the canonical path after invocation.
   - Report has §1 Method, §2 Headline counts, §3 Detailed findings, §4 Ranked backlog, §5 Cross-cutting, §6 Divergence catalog (6 H2 sections).
   - §1 records the three upstream-drift values (LOCAL_MAIN, UPSTREAM_TIP, DRIFT) verbatim.
   - §4 ranked backlog has ≥10 items.
   - Every finding in §3 has a `divergence axis tag` field populated (not blank).
   - Every finding has an effort estimate (XS/S/M/L).
   - Pre-audit `git fetch upstream` runs before subagent dispatch (verifiable via session transcript).
5. Subjective assertions skipped (which lens dominated, whether top refactor is "right") — those are user-judgement, captured in qualitative review.

### 4. Sequencing

**Option chosen:** wait for the audit to ship, then author from that experience.

**Why:** the parallel session's audit will surface unknowns (which subagent allocation worked best, where the bootstrap prompt was insufficient, what the actual report shape converged to vs the spec). Authoring from real-run experience produces a better skill body than authoring from the prompt alone. Cost: skill creation deferred by a few hours. Benefit: skill captures what actually worked, not what was supposed to work.

**Do not** dispatch this session as a parallel test-run of the audit. That would re-create the concurrent-session-collision pattern (`feedback_concurrent_advisor_session_collision.md`, PMD id 302) we just authored a lesson against.

---

## Source material when skill body is authored

When a future session picks this up:

1. **The bootstrap prompt** at `.claude/PRPs/briefs/v1-code-quality-audit-2026-05-14-prompt.md` (commit `43f8268a8`). The body below the `---PROMPT---` marker is the canonical instruction set; the skill body adapts it to skill format.

2. **The audit report** at `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md` (when it exists). Use as ground-truth output shape; assertions reference its structure.

3. **Concurrent-session discipline** from `.claude/lessons/feedback_concurrent_advisor_session_collision.md` (PMD id 302). Skill body must include the "single-purpose session, go idle when done" instruction.

4. **Companion lessons** to cite in the skill body:
   - `feedback_planner_enumerate_struct_callsites_for_addfield.md` (audit catches missing enumerations)
   - `feedback_fix_impl_pre_push_cargo_check.md` (audit catches missing pre-push checks)
   - `feedback_phase_retro_gate_enforcement.md` (audit findings inform retro gates)
   - `feedback_dq_historical_fail_sweep_at_bm_pr.md` (audit findings inform DQ discipline)
   - `feedback_bundle_means_one_worker_branch_not_one_commit.md` (audit catches per-commit anti-patterns)

5. **PRDs the audit informs** (per post-audit workflow §Step 6):
   - `v1-sponsor-liability.prd.md`
   - `v1-jury-mechanics.prd.md`
   - `v1-reputation-tuning.prd.md`
   - `v1-admin-dashboard.prd.md`
   - `v1-federation-inbound.prd.md`
   - `v1-ship-readiness.prd.md`

---

## Skill scaffolding to create when authored

```
.claude/skills/brehon-code-audit/
├── SKILL.md                  (to be written; body deferred)
├── DESIGN.md                 (this file)
├── evals/
│   └── evals.json            (test prompts; assertions added post-audit-ship)
└── references/
    └── audit-prompt.md       (copy of the bootstrap prompt for reuse)
```

`scripts/` and `assets/` directories are unlikely needed — the audit is read-only file analysis, no executable helpers or output templates.

---

## Hand-off note for the future-session

When the parallel session's audit report has shipped, run `/skill-creator:skill-creator` with input: "Resume the brehon-code-audit skill — design is at `.claude/skills/brehon-code-audit/DESIGN.md`, ground-truth audit report is at `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md`. Author SKILL.md per the design decisions captured in DESIGN.md, then write evals against the ground-truth report's shape."
