---
name: evaluate-run
description: "Post-run evaluation of skills. Capture issues, improvements, lessons learned."
---

# Evaluate Run Skill

Create a structured evaluation after running any skill.

---

## Why Evaluations Matter

**Skills improve through feedback.** Without structured evaluation, the same mistakes repeat. An evaluation that notes "missed 3 implicit decisions" leads to skill refinement.

**Input quality vs execution quality must be separate.** A skill that correctly processes garbage input isn't failing. An evaluation that blames the skill for bad data leads to wrong fixes.

**Specificity enables action.** "Improve validation" is useless. "Add check for transcript < 50% expected word count" is implementable. Evaluations must be concrete enough to become code.

**Tracking prevents regression.** When recommendations are marked "applied," future evaluations can verify the fix worked. Untracked improvements get lost or reverted.

---

## Quick Start

```bash
# 1. Identify what to evaluate
SKILL="calendar-sync"
SOURCE="Google Calendar API"

# 2. Check for existing evaluations
ls .claude/docs/post-run-evaluations/*${SKILL}*

# 3. Create evaluation file
# Filename: YYYY-MM-DD__Evaluation__{Skill}_{Source}.md
# Location: .claude/docs/post-run-evaluations/
```

Then fill in the template with:
- **TL;DR** (3 lines: status, key finding, action taken)
- **What happened** (bullet points)
- **Input quality** (separate from execution)
- **What worked well** (preserve strengths)
- **Issues** (table: severity, description)
- **Recommendations** (numbered, specific)
- **Next steps** (checkboxes)

---

## When to Use

- After any skill run with issues
- To document improvements discovered
- For skill maintenance
- After significant skill runs (even successful ones) to capture learnings

---

## Evaluation Categories

When evaluating a skill run, assess these dimensions:

### 1. Completeness

Did the skill extract/process everything it should have?

| Question | Good | Bad |
|----------|------|-----|
| All items found? | "Extracted 4/4 decisions" | "Missed 2 implicit decisions" |
| All sections populated? | "All template sections filled" | "Resources section empty" |
| Edge cases handled? | "Single-presenter workshop handled" | "Crashed on empty transcript" |

**How to check:** Compare output against source. Count items. Verify all template sections.

### 2. Accuracy

Are the extractions/outputs correct?

| Question | Good | Bad |
|----------|------|-----|
| Content faithful to source? | "Decisions match transcript" | "Paraphrased changed meaning" |
| YAML valid? | "All fields parse correctly" | "Date format wrong" |
| Links resolve? | "All wikilinks valid" | "3 broken links" |

**How to check:** Spot-check extractions against source. Validate YAML. Test wikilinks.

### 3. Efficiency

Was the approach sensible?

| Question | Good | Bad |
|----------|------|-----|
| Reasonable time? | "Processed in 10 min" | "Took 45 min for simple note" |
| No redundant work? | "Single pass sufficient" | "Re-read file 5 times" |
| Appropriate scope? | "Focused on changed files" | "Read entire codebase for a typo fix" |

**How to check:** Review approach taken. Note processing time. Check for repeated operations.

### 4. User Experience

Was the output useful and usable?

| Question | Good | Bad |
|----------|------|-----|
| Output well-organised? | "Clear section structure" | "Wall of text" |
| Actionable? | "Tasks have clear next steps" | "Vague action items" |
| Appropriate detail? | "Summary is 15 lines" | "Summary is 3 pages" |

**How to check:** Read output as user would. Check if actions are actionable.

### 5. Side Effects

Any unintended changes?

| Question | Good | Bad |
|----------|------|-----|
| Only expected files changed? | "Created 4 planned files" | "Modified unrelated note" |
| No data loss? | "Original preserved" | "Overwrote existing content" |
| State consistent? | "All flags updated" | "Forgot to update flags" |

**How to check:** Review all file changes. Verify no overwrites. Check flag states.

---

## Input Quality Assessment

Always assess input quality separately from skill execution:

| Factor | Questions to Ask |
|--------|------------------|
| Source completeness | Is the input complete? Missing sections? Truncated? |
| Source quality | Transcription errors? Garbled text? Missing speakers? |
| YAML correctness | Correct type? Required fields present? Valid values? |
| File structure | Expected location? Correct naming? Dependencies available? |

**Rating scale:**
- **9-10:** Excellent input, no issues
- **7-8:** Good input, minor issues
- **5-6:** Adequate input, some gaps
- **3-4:** Poor input, significant issues
- **1-2:** Unusable input, skill cannot succeed

**Key principle:** A skill that produces mediocre output from poor input may be working correctly. Don't recommend skill changes when input quality is the problem.

---

## Writing Good Recommendations

### Be Specific

| Bad | Good |
|-----|------|
| "Improve validation" | "Add check that `type: workshop` not `type: meeting`" |
| "Handle edge cases" | "Fall back to single-presenter format when `presenters:` has one entry" |
| "Better error messages" | "When transcript file missing, list which files exist in directory" |

### Include Context

```markdown
1. **Add transcript existence check** -- Before starting pipeline, verify all
   files listed in `transcripts:` YAML field exist. Prompt user about missing
   files rather than failing silently mid-processing.
   - Triggered by: P2.md missing during JP Byrne workshop processing
   - Impact: Would have caught issue before Pass 1 started
```

### Track Application Status

```markdown
1. **Add transcript existence check** -- [description]
   - Applied 2026-01-27: Added to Pre-flight Validation section

2. **Support subfolder structure** -- [description]
   - Pending: Needs template update first

3. **Improve error messages** -- [description]
   - Rejected: Too complex for current use case
```

---

## Example Evaluation Snippets

### Good "What Worked Well" Section

```markdown
## What Worked Well

1. **3-pass pipeline effective** -- Segmentation > Extraction > Verification
   caught issues that single-pass would miss
2. **Discussion capture was rich** -- Q&A back-and-forth preserved with
   speaker attribution
3. **Derived notes properly linked** -- All tasks/questions/ideas have
   `source_note:` pointing to workshop
```

### Good "Issues" Table

```markdown
## Issues

| Severity | Description |
|----------|-------------|
| warning | YAML used `type: meeting` instead of `type: workshop` -- auto-corrected |
| warning | P2 transcript missing -- user confirmed to proceed without |
| info | Single presenter but template assumes multiple -- adapted appropriately |
| info | Breakout outcomes incomplete due to missing P2 |
```

### Good "Input Quality Assessment"

```markdown
## Input Quality Assessment

**Input quality: Good (7/10)**

| Factor | Rating | Notes |
|--------|--------|-------|
| Transcript completeness | Partial | P2 missing (breakout session) |
| Transcript quality | Good | Clear speaker attribution, minimal errors |
| YAML accuracy | Partial | Wrong type field; missing themes |
| File structure | Good | Subfolder pattern with transcripts/ |

**Conclusion:** Input was adequate. Missing P2 was input limitation, not skill failure.
```

---

## Template

Use the following structure for evaluation files saved to `.claude/docs/post-run-evaluations/`:

```markdown
# Evaluation: {Skill} -- {Source}

## TL;DR
- **Status**: [Pass/Partial/Fail]
- **Key finding**: [One sentence]
- **Action taken**: [One sentence]

## What Happened
- [Bullet points]

## Input Quality Assessment
**Input quality: [Rating] ([N]/10)**

## What Worked Well
1. [Strength to preserve]

## Issues
| Severity | Description |
|----------|-------------|
| [critical/warning/info] | [Description] |

## Recommendations
1. **[Title]** -- [Description with context]

## Next Steps
- [ ] [Concrete action]
```

## Output Location

`.claude/docs/post-run-evaluations/YYYY-MM-DD__Evaluation__{Skill}_{Slug}.md`

---

## Checklist Before Completing Evaluation

- [ ] TL;DR captures the essence in 3 lines
- [ ] Input quality assessed separately from execution
- [ ] "What Worked Well" documents strengths to preserve
- [ ] Issues table uses consistent severity levels (critical/warning/info)
- [ ] Recommendations are specific and actionable
- [ ] Each recommendation explains context/trigger
- [ ] Next steps are concrete checkboxes
- [ ] File count in YAML matches actual issues/recommendations
