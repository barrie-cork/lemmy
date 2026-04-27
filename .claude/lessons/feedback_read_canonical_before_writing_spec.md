---
name: Read a canonical example before writing a spec for that artifact class
description: When authoring a spec, template, or rule that prescribes the shape of an artifact (plan file, brief, retro, command), Glob + Read 1-2 existing canonical instances FIRST. Implicit schemas in a corpus are the single biggest source of plan-revision drift mid-implementation.
type: feedback
---

When authoring any spec, template, or rule that prescribes the shape of an artifact (plan files, briefs, retros, slash-command specs, agent contracts), `Glob` + `Read` 1-2 existing canonical instances **before** writing the spec — not after.

**Why:** 2026-04-27 spec-kit pattern adoption session. Wrote a plan that referenced "§15 task list, §16 stories" before reading any Brehon plan. The actual canonical 20-section schema is §13 = step-by-step tasks, §15 = validation commands DoD, §16 = acceptance criteria, §17 = completion checklist. The "§15 task" assumption was wrong. Caught only when I started extracting the template — at which point I had to Edit the plan file in-place to remap, and could have cascaded the drift through 4 dependent files (template, both agents, advisor-orchestrator) if not caught.

**How to apply:**

- Before writing `*.template.md`, `*.spec.md`, or any rule file that names section numbers/titles: `Glob` for the existing artifacts, `Read` 1-2 of them, `Grep '^## '` to extract the section schema.
- Cite the canonical instance(s) in the new spec ("based on the schema observed in `phase-v1-JM-a.plan.md`, `v1-jury-mechanics-c.plan.md`").
- If the existing schema looks ad-hoc rather than load-bearing, raise it explicitly: "the existing plans use schema X; this spec assumes Y; reconcile before commit."

**Generalises to:** any artifact-class authorship — Junior agent contracts, brief templates, decision-queue schemas, retro structures. The cheap 2-second `grep '^##'` is always worth it.

**Symptom to recognise in retrospect:** an in-place Edit on the plan file mid-implementation that renames section numbers or rewires schema references. If you find yourself doing that, the parent miss was "didn't read a canonical example before writing the spec."

**Brehon-specific application:**

- **Advisor session** authoring a new rule, command, lesson, or template under `.claude/`: before writing, `Glob .claude/{rules,commands,lessons,PRPs/templates}/` for sibling artifacts of the same class; Read 1-2; cite at the top of the new file. Per `.claude/rules/advisor-orchestrator.md` "Canonical-schema-first gate".
- **Planning subagent** authoring a plan: before §11 (Files to change) or §13 (Step-by-step tasks), Glob `.claude/PRPs/plans/` for the most-recent shipped plan in the same family; cite in §2 Source. Per `.claude/agents/planning.md` "Plan content discipline".
