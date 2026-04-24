---
id: retro-r53-enum-drift-pattern
from: advisor
to: impl
ts: 2026-04-24T03:45Z
relates_to: phase-v1-JM-a-retro.md, R5.2, R5.3 (pr92-lows-complete.md cr-4 flag)
decision: amend-retro-with-r53-plus-pattern-meta-entry
---

# Decision
Add R5.3 to the retro drift list **and** roll R5.2+R5.3 into a pattern-level meta-observation in a new subsection. Second-strike is a pattern, not a one-off. Document explicitly so future sub-phases inherit the fix.

Combine with any other in-flight retro amendment commit (tool-use + CR-quality + R5.3 can all land in one commit). Suggested combined subject:

```
docs(v1-JM-a): amend retro — tool-use + CR-quality + R5.3 enum-drift pattern
```

# Instructions

## 1. Add R5.3 as a standalone retro entry

Insert R5.3 adjacent to R5.2 in the existing §"Plan drifts" section. Format it to match the R5.2 structure verbatim (same field labels, same tone):

```markdown
### R5.3 — CR + advisor both hallucinated CaseStatusTier variants (cr-4 lows batch)

**Symptom**: During the cr-4 docstring fix, CR's original finding proposed `Regular/Escalated/Maximum` as the accurate `CaseStatusTier` variants. Advisor relay `pr92-lows-batch.md` echoed that triplet without cross-checking. Impl verified against PRD §4.1 line 239-242 AND `crates/db_schema_file/src/enums.rs:697` and found the real variants are `Founder / Regular / Probation`.

**Caught by**: impl cross-check before writing code. Fixed in-commit with the correct variant names; commit body documents the drift.

**Root cause**: advisor accepted CR's claim at face value and propagated it in the relay. Did not cite PRD section + line in the relay body. Same failure mode as R5.2 — the advisor's enum-value proposals outran the underlying source-of-truth (PRD + enums.rs).

**Resolution**: commit `8ad8a3b56` uses the PRD-faithful variants. No separate commit needed; the fix is in-line with the cr-4 fix.

**Retro carry (plan-amendment recommendation)**: see §N.M below — pattern-level entry supersedes the per-instance recommendations from R5.2 and R5.3 individually.
```

## 2. Add the pattern-level meta-entry

After R5.3, add a new subsection rolling up both drifts. This is the load-bearing part — R5.2 alone looked like a one-off; R5.2+R5.3 is a pattern. Use this shape:

```markdown
### Pattern: advisor/CR enum-value drift (R5.2 + R5.3)

**Pattern**: Both R5.2 (cr-9 `JuryConstraintRelaxationReason` vocabulary) and R5.3 (cr-4 `CaseStatusTier` variants) share a common failure mode:

1. CR or advisor names specific enum-value strings in a finding/relay
2. The names sound plausible (`reputation_waiver`, `emergency_panel`, `Regular`, `Escalated`) because they match domain-adjacent vocabulary
3. The names are NOT the actual PRD + enums.rs variants
4. Impl detects by cross-checking PRD section + line before writing code

**Frequency**: 2 occurrences in v1-JM-a (one with CR as the source, one where CR's drift was echoed by advisor without verification). Both enum-related. Both caught by impl pre-write.

**Load-bearing observation**: **CR + advisor enum-value proposals are untrusted input until verified against PRD + the `enums.rs` or analogous source-of-truth file.** This is now a plan-authoring rule, not a retro curiosity.

**Plan-template amendments required** (for v1-JM-b and future sub-phases):

1. **For PRD-adjacent enum proposals in plans or relays**: every enum-value string named in a plan must cite `PRD §X.Y line Z` or `<file>:<line>` inline. Missing citation = reject at plan review.
2. **For advisor relays answering CR findings**: if the relay names concrete identifiers (enum values, column names, function names, const names), the relay MUST include a `# Source cross-check` section listing the PRD ref + code-path ref the advisor verified against. Missing section = impl treats the proposal as untrusted.
3. **For impl receiving a relay with named identifiers**: pre-write verification is mandatory — `grep -n <identifier> crates/` + PRD read. Impl's pattern in R5.2 (catch + file relay) and R5.3 (catch + fix in-commit) are both acceptable; the difference is whether the drift is large enough to block (R5.2, new vocabulary) or small enough to fix inline (R5.3, docstring).

**Memory note suggestion**: add a new `feedback_advisor_cr_enum_drift.md` memory entry with this pattern. Title: "Advisor/CR enum-value proposals require source-of-truth verification." Applicable to all v1+ sub-phases until the pattern stops recurring (three phases clean = pattern retired).

**Load-bearing quote for future planners**: "If the advisor or CR names a specific enum value string, the impl's default assumption should be: not in the PRD until proven otherwise. This defaults toward verification, not trust."
```

## 3. Cross-reference R5.2 + R5.3 from the pattern subsection

Add a "See also" line at the end of the pattern subsection pointing back to both individual drift entries and forward to the plan-amendment section (if that's where JM-b template changes land).

## 4. If combining with the tool-use + CR-quality amendments

If you haven't landed those yet, this is one commit. Section ordering suggestion:

1. §"Tool-use self-assessment" (per `retro-tool-use-amendment.md`)
2. §"CR finding quality — PR #92" (per `retro-cr-quality-amendment.md`) — R5.3 adds a clear row to the per-severity accuracy rollup in §N.5 (cr-4 goes in "False-positive class" column because the concern was valid but the variants proposed were wrong)
3. §"Plan drifts" — existing + R5.3 added + the new pattern subsection immediately after R5.3
4. §"Handoff notes" — unchanged

## 5. Retro section §N.5 (per CR-quality amendment) row update

The per-severity rollup table in the CR-quality section should now show cr-4 in the "False-positive class" column (valid concern, wrong proposed fix). Low row: "5 ingested, 4 correct + actionable, 0 wrong location, 0 hallucinated, 1 false-positive class (cr-4: docstring correction was valid but the proposed Regular/Escalated/Maximum variants were hallucinated by CR — impl fixed with real Founder/Regular/Probation)."

# Validation

Docs-only. L1 `cargo check --workspace` exit 0 (trivial; markdown changes only).

# Commit

If combining all three amendments:
```
docs(v1-JM-a): amend retro — tool-use + CR-quality + R5.3 enum-drift pattern
```

If R5.3 + pattern meta-entry only (tool-use + CR-quality already landed):
```
docs(v1-JM-a): amend retro — R5.3 + advisor/CR enum-drift pattern
```

Body should reference:
- R5.2 (prior enum-drift occurrence, cr-9 vocabulary)
- R5.3 (current enum-drift occurrence, cr-4 variants)
- cr-11 disposition: user bucketed wont-fix 2026-04-24 (for the retro's CR-quality cr-11 mention)

# Next

1. Land the amendment commit.
2. Relay `impl-relays/retro-amendments-complete.md` when done — BM will push + note in runlog.
3. After CI on `8ad8a3b56` (and post-amendment) goes green, PR #92 is merge-ready.

# Back-reference

Addresses:
- `impl-relays/pr92-lows-complete.md` §"cr-4 drift — advisor + CR both proposed wrong enum variants"
- User decision 2026-04-24T03:40Z: "write a relay for impl to add R5.3 + a pattern-level observation to the retro"
- Prior drift R5.2 (`advisor-relays/cr-9-enum-vocab-answer.md`)

# Why this matters (for the /handover skill design)

Add to the skill's design spec:

> Handover briefs from advisor → impl must include a `# Source cross-check` section for any answer naming concrete identifiers (enum values, column names, function names). Without the section, impl should default to verify-before-write. This codifies the R5.2 + R5.3 pattern as a schema-level guarantee, not a per-relay discipline the advisor can forget.

Add this to the retro's §"Handover skill design inputs" section (same place the tool-use + CR-quality relays point to).
