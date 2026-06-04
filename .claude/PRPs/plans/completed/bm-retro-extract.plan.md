# Plan — `/bm-retro-extract` command + post-merge retro-to-memory ritual

**Status:** draft plan · impl deferred to separate session · source: v1-JM-a cleanup-audit 2026-04-24
**Owner:** BM session (the command runs as BM)
**Scope:** add a single new command + rule amendments; ~1-2h impl including tests
**Trigger to execute:** whoever picks up "systemic retro-feedback-loop fix" next session

---

## 1. Background — why this exists

v1-JM-a's retro (`.claude/PRPs/reports/phase-v1-JM-a-retro.md`, merged in PR #92) identified 4 plan-authoring lessons — R3.2, R5.1, R5.2/R5.3 pattern, R10.1 — and **explicitly named** the memory entries that should be written to preserve them:

- R3.2 → `feedback_interim_failure_task_wording.md`
- R5.1 → `feedback_insertform_default_propagation.md`
- R5.2/R5.3 → `feedback_advisor_cr_enum_drift.md`
- R10.1 → `feedback_phase1_migration_count_lifo.md`

**None were written until 2026-04-24**, ~5h after the retro landed on trunk. The retro was archival (committed in git) but the lessons were not extracted into the per-session auto-loaded feedback surface.

Manual extraction this time took ~15 min (read retro, write 4 files, update MEMORY.md). That's fine once. It won't scale across v1-JM-b/c/d/e + v1-SL/RT/FI if every phase ends with 3-5 unextracted lessons sitting in retro files nobody re-reads.

The gap: retro → memory entry is a **recursive-learning step**, currently unowned and unenforced. Every missed extraction = a lesson that the next phase's planner and impl can't see.

This plan fixes that via one command + two rule amendments.

---

## 2. Goal

**After this plan lands, no retro ship without its lessons extracted.** Specifically:

1. A `/bm-retro-extract <phase>` command that the BM session runs post-merge. It reads the phase's retro file, proposes draft memory entries for every R-item, and asks the user to confirm before writing.
2. A rule amendment to `.claude/rules/branch-manager.md` marking retro-extract as a mandatory post-merge step (alongside archive, runlog append).
3. A rule amendment to the plan template requiring every plan's §3 "What to carry forward" section to name the memory entries the retro will produce (if any). Forces the retro writer to think about extraction up front, not post-hoc.

**Out of scope for this plan:**

- Automatic extraction (no AskUserQuestion gate). The draft-and-confirm flow is load-bearing because memory-entry wording has to be right-first-time — it's read by every future session.
- Extraction of older retros (v0 Phase 5a/5b/5c, v1-AD). Those are retroactive; handle separately if worth it.
- Memory-entry *removal* (the "retire when" clause). Defer to a future `/bm-retro-prune` command.

---

## 3. User-facing design

### 3.1 Command invocation

```
/bm-retro-extract v1-JM-b
```

Arg is the phase identifier matching `.claude/PRPs/reports/phase-<N>-retro.md` (or `phase-v1-<id>-retro.md` for sub-phases).

### 3.2 Command behavior

1. **Locate retro:** find `.claude/PRPs/reports/phase-<arg>-retro.md`. If missing, STOP with a message.
2. **Parse retro:** read the file. Look for sections named §2 ("What surprised"), §2.* sub-items with headings like "R<number>.<number>" or "Rule:" — these are the R-items.
3. **For each R-item, build a draft memory entry:**
   - Filename: inferred from the R-item's prescribed name in the retro (if the retro says "add a new `feedback_<name>.md` memory entry", use that); else generate from the R-item's title with the `feedback_` prefix.
   - Frontmatter: `name`, `description`, `type: feedback`, populated from the R-item's content.
   - Body: 3 sections per the memory-system rule in CLAUDE.md root config — lead rule line, **Why:** section, **How to apply:** section. Pull from the retro's R-item "Root cause" and "Fix" / "Resolution" subsections.
4. **Show draft to user:** for each proposed memory entry, print filename + a 15-line preview. Ask `confirm-all` / `edit <N>` / `skip <N>` / `abort`.
5. **On confirm:** write the files, update `MEMORY.md` with index lines in the appropriate section (user picks or auto-picks based on keywords — "plan" → plan-authoring section, "rust"/"diesel" → Rust patterns, etc.).
6. **Append to runlog:** `bm: retro-extract — <ISO>` with counts.
7. **No gh activity. No commits. Local-only.** Memory lives under `~/.claude/` so it's per-user anyway; not in-repo.

### 3.3 Refusal cases

- Retro file missing → STOP, suggest `/bm-merge` or "write the retro first".
- Retro has zero R-items detected → REPORT and ask user to point at the relevant section manually.
- MEMORY.md missing its standard topic headers → STOP, ask user to update MEMORY.md structure first.

### 3.4 Autonomy bounds (per branch-manager.md conventions)

- Read retro, read MEMORY.md, draft memory entries → **Auto** (local, reversible).
- Write memory files + update MEMORY.md → **Manual, asks first** (per-file confirm; once written, they affect every future session).
- Runlog append → **Auto**.

Rationale: memory entries are durable, auto-loaded, and wording-sensitive. Same risk profile as a PR comment (outbound/visible to every future session). Default to confirm.

---

## 4. Implementation outline

### 4.1 New command file

`.claude/commands/bm/bm-retro-extract.md`

Follows the template shape of other `bm-*.md` files. Dispatcher delegates to the `branch-manager` subagent with `model="opus"` (judgment-heavy; wording matters) and max-effort. Operational script has 7 phases:

1. Locate retro file.
2. Parse R-items (section heading regex: `^### \d+\.\d+\w* — ` or similar; confirm against v1-JM-a retro structure).
3. For each R-item, extract title + Why + How-to-apply sections.
4. Build draft memory entries (frontmatter + body per the system prompt schema).
5. **ASK USER** per memory entry: `confirm` / `edit` / `skip`. Use AskUserQuestion.
6. Write confirmed files; update MEMORY.md index lines.
7. Runlog append.

### 4.2 Rule amendments

**`.claude/rules/branch-manager.md`:** add a new section "Post-merge ritual" or extend the existing "Session-start ritual" section. List:

1. `/bm-merge` (gates merge).
2. `/bm-retro-extract <phase>` (extracts lessons — mandatory if a retro file exists).
3. (Optional, per "B" in this conversation — queued separately:) `/bm-archive <phase>` to commit gitignored review artifacts.

**Plan template** (`.claude/rules/plan-template.md` or wherever the §3 "Carry forward" schema lives — verify location before amending): add a sub-section "§3.5 Memory entries to write post-phase". Each R-item flagged during retro drafting names the memory filename it should produce. If the retro doesn't name them, the impl session is expected to add them during retro-drafting.

### 4.3 Tests

One manual sanity test: run `/bm-retro-extract v1-JM-a` against the already-extracted retro. Expected output: 4 drafts matching the 4 files already written (the ones from this conversation's "A"). Verify the drafts would produce byte-equivalent content to what's already on disk. If so, the parser + drafter are correct. If not, tune the parser.

---

## 5. Risks / edge cases

- **Retro structure drift across phases.** v1-JM-a's retro has §2.1/§2.2a/§2.2b/§2.2c/§2.3 structure. Phase 5c and v1-AD retros use different heading shapes. The parser needs to be forgiving — prefer "find any `### .+ — ` inside §2" over strict regex. Or standardise retro template structure as a related rule amendment.
- **Memory-entry duplication.** If `/bm-retro-extract` is re-run, it shouldn't overwrite existing entries silently. Check-then-skip on file existence; tell user "already extracted" per-file.
- **Topic assignment in MEMORY.md.** The auto-picker based on keywords might misclassify. Safe default: new entries go into a dated sub-section at the top (`## Pending triage — 2026-04-24 retro sweep`) and the user moves them during normal memory maintenance. Avoid clever auto-sorting; it's not worth the drift.
- **Retro "What worked" section items.** Those are not R-items per se — they're "keep doing" patterns. Extraction is optional. v1-JM-a retro §1.1-1.5 could each produce a feedback entry (e.g., "pattern snippets must cite file:line"), but they're often derivative of existing rules. Default: extract only §2 "What surprised" R-items; surface §1 items as "optional — want to extract?" at the end of the run.

---

## 6. Why this over alternatives

**Why a command + rule, not a skill?**
Skills are user-triggered and conversational; commands are scripted workflows with operational discipline. Retro extraction is a scripted workflow (parse → draft → confirm → write → log), not a conversational one. Command fits.

**Why not auto-extract (no confirm gate)?**
Memory wording is load-bearing for every future session. A bad entry (ambiguous, contradictory, overfit to one incident) silently wastes every session's context budget. Confirm gate costs ~2 min per phase; removing it saves nothing meaningful.

**Why the plan-template §3.5 amendment?**
Forces the retro author (impl session) to think about extraction while the details are fresh, not 3 weeks later when a BM session tries to parse a rotted retro. Same principle as "write the test first" — surface the downstream requirement at the point where the context is richest.

---

## 7. Acceptance criteria

- [ ] `.claude/commands/bm/bm-retro-extract.md` exists and matches the `bm-*.md` command-template shape.
- [ ] Running the command against `v1-JM-a` retro produces drafts for exactly the 4 R-items (R3.2, R5.1, R5.2/R5.3 pattern, R10.1) with filenames matching the retro's named prescriptions.
- [ ] Running against a retro with no §2 section produces a clean "no R-items detected" message, not an error.
- [ ] `.claude/rules/branch-manager.md` lists `/bm-retro-extract` as a post-merge mandatory step.
- [ ] Plan template has §3.5 "Memory entries to write post-phase" schema.
- [ ] Runlog entries for test runs present.

---

## 8. Handoff checklist for impl session

When picking this up:

1. Read this file + `.claude/PRPs/reports/phase-v1-JM-a-retro.md` (the test case) + `.claude/commands/bm/bm-merge.md` (closest analogous command for template) + `.claude/rules/branch-manager.md` (ownership rules this command must respect).
2. Verify the plan template location (§4.2 "wherever the §3 Carry forward schema lives — verify"). Might be in `.claude/commands/prp-core/prp-plan.md` or a rule file. Either way, the amendment lands there.
3. Write command + rule amendments in a single `chore(bm): add /bm-retro-extract + post-merge ritual` commit on trunk (`governance-v0`). This is a BM-lane change, not tied to any phase branch.
4. Manual sanity test against v1-JM-a retro before committing.
5. Post-impl: run `/bm-retro-extract v1-JM-a` once to confirm behaviour against already-extracted reality (should report "already extracted" on the 4 files).

Estimated effort: 1-2h for a fresh session. Most of the time is tuning the R-item parser against real retro structure.
