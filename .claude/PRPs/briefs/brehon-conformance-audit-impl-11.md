[role:impl-task] brehon-conformance-audit Task 11 — wire skill + Clippy gate into advisor-orchestrator.md (Cohort 5)

## 1. Role + dispatch

`[role:impl-task]` — single file edit; surgical Edit-tool insertions into `.claude/rules/advisor-orchestrator.md`. No new code.

## 2. Scope

Per plan §13 Task 11 + plan §10.11. Make three surgical Edit-tool insertions into `.claude/rules/advisor-orchestrator.md` that wire the brehon-conformance-audit skill into advisor session orchestration:

1. **§3.1 prevention checkpoint** — new sub-section `### 3.1.1 Conformance-audit prevention checkpoint`, inserted AFTER existing `### 2.3 Pre-queue lesson check` sub-section and BEFORE existing `### 2.4 Mandatory file-class lesson injection` sub-section. (NOTE: numbering is `2.3/2.4` per the live file's sub-section indices — verify via `grep -n "^### " .claude/rules/advisor-orchestrator.md | head -20` BEFORE picking insertion anchor.)
2. **§3.9 detection checkpoint** — new sub-section `### 3.9.1 Conformance-audit detection checkpoint`, appended at the END of `## 3.9 Verify gate` section, BEFORE the start of `## 4. Cohort dispatch` (or whatever the next top-level section is — verify with `grep -n "^## " .claude/rules/advisor-orchestrator.md`).
3. **§G4 classifier table** — new row in the "Non-allowlist (catch-fire to user)" sub-section of §5.3, appended after the existing E0599 row but BEFORE the "Cycle-count meta-rule" paragraph.

The §G4 row goes into the **HARD REFUSAL** (non-allowlist) sub-section, NOT the auto-fix allowlist. Conformance-audit findings are human-in-the-loop catch-fires.

### 2.1 Verbatim content blocks (copy-paste from plan §10.11)

**Block 1 — §3.1.1 sub-section body:**

```markdown
### 3.1.1 Conformance-audit prevention checkpoint

**Skill invocation — conformance-audit (per `.claude/skills/brehon-conformance-audit/`)**:
If the brief targets a file matching `crates/apub/activities/src/governance/**.rs` OR
`crates/api/api/src/governance/**.rs` OR `crates/db_schema/src/source/governance/**.rs`,
invoke the skill with `target_scope = file <brief-named-file>` BEFORE `/brehon-clarify`.
Tier-1 findings fold into brief §3 / §4 before clarify-DQ entries.
```

**Block 2 — §3.9.1 sub-section body:**

```markdown
### 3.9.1 Conformance-audit detection checkpoint

**Conformance-audit detection** (per `.claude/skills/brehon-conformance-audit/`):
Before queueing `bm-merge`, after `/brehon-verify` returns ✓, run the skill with
`target_scope = phase-diff <phase-branch>`. Tier-1 findings become §3 actions in the retro.
Update the per-phase metrics file at `.claude/PRPs/audit-metrics/<phase>.json`. Run
`compute-metrics.sh` for per-sub-phase calibration.
```

**Block 3 — §G4 classifier table new row (verbatim, HARD REFUSAL non-allowlist row):**

```markdown
| Conformance-audit Tier-1 finding on `crates/apub/activities/src/governance/**.rs` OR `crates/api/api/src/governance/**.rs` OR `crates/db_schema/src/source/governance/**.rs` | **HARD REFUSAL — catch-fire to user** with audit report + suggested per-axis fix. NOT auto-fix; human-in-the-loop decides. | `feedback_mirror_phase6_convention_in_same_file.md` |
```

(Format note: the §G4 "Non-allowlist" sub-section in the live file uses a simpler 2-column table, NOT 3-column. Read the live file's existing non-allowlist table BEFORE inserting; adapt the row shape to match — likely just `| Trigger | Action |` shape, with the lesson citation embedded in the action prose.)

## 3. Required reading

Read each fully BEFORE editing:

1. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` — §10.11 (lines ~596-615) for the canonical wiring prose; §13 Task 11 (lines ~1547-1607) for the implement spec.
2. `.claude/rules/advisor-orchestrator.md` — entire file. Identify §3.1, §3.9, §5.3 structure BEFORE editing. **Critical:** the live file's sub-section indices may differ from plan §13 (which says "§3.1 sub-section after Pre-queue lesson check"). Use `grep -n "^### " .claude/rules/advisor-orchestrator.md` to find the exact insertion anchors.
3. `.claude/lessons/feedback_read_canonical_before_writing_spec.md` — read canonical structure before authoring; mandatory pre-Edit gate.
4. `.claude/PRPs/templates/impl-task-brief.template.md` — brief shape reference (this brief's structure).

**MIRROR refs:** existing `### X.Y.Z` sub-section style in `advisor-orchestrator.md` (markdown header + body block); existing §G4 table row shape (verify column count by reading the live file).

## 4. Constraints

1. **NEVER touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`.** This task modifies ONE file only: `.claude/rules/advisor-orchestrator.md`.
2. **Surgical Edit-tool insertions ONLY.** Do NOT rewrite or restructure other sections. Preserve all existing content verbatim.
3. **Read the live file FIRST** to confirm the §G4 table column-count + the `### 3.1 Pre-queue lesson check` / `### 3.9 Verify gate` exact anchor strings. Plan §10.11 prose may have drifted from the live file structure; trust the LIVE file over the plan when they disagree (per `feedback_read_canonical_before_writing_spec.md`).
4. **Verify markdown structure post-edit**: after Edit, run `python3 -c "content=open('.claude/rules/advisor-orchestrator.md').read(); import re; headers=re.findall(r'^(#{1,6})\\s+', content, re.M); assert len(headers) > 20, 'header count regression'; print('OK')"`. EXPECT: `OK`.
5. **Mid-task push discipline:** after Edit, raise `kind: "validate-pending-laptop"` DQ per the standard pattern. Single command:
   ```
   grep -c "Conformance-audit prevention checkpoint" .claude/rules/advisor-orchestrator.md && grep -c "Conformance-audit detection checkpoint" .claude/rules/advisor-orchestrator.md && grep -c "Conformance-audit Tier-1 finding" .claude/rules/advisor-orchestrator.md
   ```
   EXPECT: all three greps return ≥1. (No cargo invocation — this is a markdown-only change.)
6. **DQ raise atomic order:** commit + push the rule edit FIRST, THEN commit + push the DQ raise (atomic raise-before-dispatch per advisor-orchestrator.md §3.1).
7. **Branch:** fork from `phase-brehon-conformance-audit` tip (currently `4daccb4de`). Worker branch will be auto-generated by Junior.
8. **One commit** for the rule edit; **one commit** for the DQ raise. Two commits total, in that order.

## 5. Validation gate (worker-side)

Per plan §13 Task 11 VALIDATE block:

```bash
# §3.1.1 sub-section present
grep -c "Conformance-audit prevention checkpoint" .claude/rules/advisor-orchestrator.md
# EXPECT: >=1

# §3.9.1 sub-section present
grep -c "Conformance-audit detection checkpoint" .claude/rules/advisor-orchestrator.md
# EXPECT: >=1

# §G4 new row present
grep -c "Conformance-audit Tier-1 finding" .claude/rules/advisor-orchestrator.md
# EXPECT: >=1

# Markdown structure still valid
python3 -c "
content = open('.claude/rules/advisor-orchestrator.md').read()
import re
headers = re.findall(r'^(#{1,6})\s+', content, re.M)
assert len(headers) > 20, 'too few headers — possible truncation'
print('OK')
"
# EXPECT: OK
```

All four must pass before raising the DQ.

## 6. Commit subject

`feat(rules): wire brehon-conformance-audit skill + Clippy gate into advisor-orchestrator.md (task 11)`

LESSON trailer: include if you discover a durable observation (per `feedback_junior_pmd_write_convention.md`).
