---
name: harness-audit
description: >
  Evidence-based audit of the Claude Code harness's startup token load.
  Inventories auto-loaded rules, scores compression candidates, emits a ranked
  report at `.claude/PRPs/reports/harness-audit-<date>.md`. Read-only.
  DO use: "audit the harness", "review startup tokens", "what's loading at
  session start", or after a major rule/lesson restructure.
  Do NOT use: applying recommendations, Pi/MCP audits, or one-shot wc -l checks.
user-invocable: true
---

# Harness audit

Evidence-based audit of what loads at Claude Code session start in this repo. Produces a ranked report identifying compression candidates without ever editing source. The user reviews and decides what to trim.

**This skill produces a report only — it does NOT modify any harness file.**

## Phase 0: Detect environment

**Do this FIRST, before any other phase.**

1. **Confirm Claude Code session.** Read `AGENTS.md` (root) — if the file exists, log "Pi-Coding entry point present; this audit covers Claude Code only." If absent, log "single-harness repo." Do NOT audit `.pi/` regardless of platform — that's a sibling skill's job.
2. **Read `.claude/settings.json`.** Capture `skillListingBudgetFraction` (the per-turn skill catalogue cost cap; current minimum sane value is `0.005`). Note the value in the report so a future audit can detect regressions.
3. **Read `.mcp.json`.** Capture the list of configured MCP servers (just names, not configs). The report notes which are loaded at startup.
4. **Print one-line snapshot:** `env: claude-code | settings.skillListingBudgetFraction=<v> | mcp=<server1,server2,…> | pi=<present|absent>`.
5. **Concurrent-session check.** Run `git log governance-v0..HEAD --oneline --since="60 minutes ago"` (or `git log governance-v0 --oneline --since="60 minutes ago"` if already on governance-v0). If any commits appear that are not yours (author ≠ current git user), surface them in the env snapshot line: `⚠ concurrent-session activity: <N> commits in last 60 min (<sha-short> <subject>, …)`. This is observation only — do NOT stop the audit. Recurrence ≥3× (per `feedback_parallel_agents_one_worktree_per_agent`, `feedback_parallel_agent_diff_collision_detection`, 2026-05-09 empirical b8225be3e..479408a98) justifies the standing probe.
6. **Deterministic frontmatter classification (the SCOPED/ALWAYS source of truth).** Do NOT delegate classification to the Phase 1 subagent's reading judgment — it has misclassified files by prose-reading in TWO consecutive audits (2026-05-09 `governance-log-entry-kind-registry.md`; 2026-05-31 `multi-lane-worktree.md`, both reported SCOPED but ALWAYS-loading). A file is SCOPED **only** if line 1 is exactly `---` AND a `paths:` key appears inside the closing `---`. Anything else — including a file that merely mentions "paths" in prose, or whose line 1 is a `#` heading — is ALWAYS. Produce the canonical classification mechanically:

   ```bash
   for f in .claude/rules/*.md; do
     if [ "$(head -1 "$f")" = "---" ] && head -20 "$f" | awk '/^---$/{c++} c==1 && /^paths:/{found=1} END{exit !found}'; then
       echo "SCOPED  $f"
     else
       echo "ALWAYS  $f"
     fi
   done
   ```

   This table is the authority. Phase 1's subagent records line/char counts against it; Phase 3 scores only the files this step marks ALWAYS. If the subagent's reported class disagrees with this grep, the grep wins and the disagreement is logged as a subagent-drift note in the report.

## Phase 1: Inventory auto-load (delegated to Explore subagent)

Read-heavy probes belong in a subagent — keep parent context small. Per `.claude/lessons/feedback_subagent_delegation_for_multi_probe_commands.md`.

Launch ONE `Explore` agent with this self-contained prompt. **Paste the Phase 0 Step 6 classification table into the prompt where `<paste classification table>` appears** — the subagent records counts against that authority, it does NOT re-derive the class by reading (prose-reading mis-classified files in two prior audits):

> Working directory: `<repo-root>`. Inventory every file under `.claude/rules/` and report size against a pre-computed load classification.
>
> Here is the authoritative SCOPED/ALWAYS classification (computed mechanically by grep — treat it as fixed, do NOT override it by reading the file):
>
> ```
> <paste classification table>
> ```
>
> For each `.claude/rules/*.md`:
> 1. Record line count + character count.
> 2. Copy the `class` (ALWAYS or SCOPED) verbatim from the table above.
> 3. As a cross-check only: read the first 10 lines. If your reading suggests a different class than the table (e.g. you see `---` + `paths:` on a file the table marked ALWAYS, or vice-versa), do NOT change the class — instead add a `⚠ class-mismatch` note for that row. The grep table is authoritative; a mismatch means the grep or the file is malformed and a human must look.
>
> Also inventory:
> - `CLAUDE.md` (root) and `.claude/CLAUDE.md` — line count + character count.
> - User-scope MEMORY.md at `~/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/MEMORY.md` — line count (must be ≤195 to avoid 200-line truncation per the existing system warning).
> - `.claude/skills/*/SKILL.md` — count + total size (this is the skill listing budget input; capped by `skillListingBudgetFraction`).
>
> Return a ranked Markdown table sorted by char count descending: columns `class | path | lines | chars | est_tokens (chars/4)`. Append a 4-line summary: total ALWAYS files, total ALWAYS chars, total SCOPED files, total SCOPED chars. **Do NOT propose recommendations** — pure inventory only. Read-only.

The parent prints the table verbatim into the running report.

## Phase 2: Quantify cross-references

Some compression candidates depend on whether content is duplicated across files. Run these grep passes (parent, not subagent — they're 1-shot):

1. `Grep` for the names of the top-5 ALWAYS-load rule files (by char count) across all of `.claude/` — count external citations. Files cited from many places are harder to rename safely.
2. `Grep` for `# governance-log-entry-kind-registry`, `# decision-queue`, `# advisor-orchestrator`, `# branch-manager`, `# auto-phase` heading anchors. Each cross-citation is one line in the report's "external-citation count" column.
3. `Glob` `.claude/refs/*.md` — count files. The `refs/` directory is the project's "read-on-demand" convention (established by Pass 1b of the 2026-05-09 trim); a populated refs/ shows the harness has already done some progressive disclosure.

## Phase 3: Score

Read `helpers/scoring-matrix.md` just-in-time and apply the formula to every ALWAYS-load file. SCOPED files are NOT compression candidates — they don't auto-load in meta-work sessions. Use the helper's weights verbatim; do not improvise.

Output: ranked table, columns `path | always? | size_score | redundancy_score | citation_score | composite | bucket`.

## Phase 4: Recommend (three buckets)

Apply the scoring matrix's bucket thresholds to the ranked table:

- **High-confidence wins** (composite ≥ 6.0): explicit recommendation per file with estimated tokens saved. Example shapes: "add `paths:` frontmatter (Read-scope)"; "extract canonical-source-duplicate prose to `.claude/refs/<name>.md`"; "fold redundant entries into the parent pattern".
- **Watch items** (composite 3.0–5.9): recommend defer. Surface for retro promotion if it recurs in future audits. Cite the trigger that would promote (e.g. "if this file grows past 300 lines OR gains an additional canonical-source duplicate, escalate to high-confidence").
- **Out of scope** (any composite, but Pi-shared or already-minimum): explicitly list. Mandatory entries: `.pi/**`, `AGENTS.md`, `.claude/rules/branch-manager.md`, `.claude/rules/no-cargo-output-paste.md`, `.claude/rules/decision-queue.md` (schema), `.claude/lessons/`, `.claude/skills/` (Pi shares these per `.pi/PROJECT_CONTEXT.md` boundary). Also include `.claude/settings.json` if `skillListingBudgetFraction` is already at `0.005`.

For every "high-confidence" recommendation, the report includes:
- Source file path + current line count.
- The proposed change (one of the three shapes above).
- Estimated tokens saved (chars saved ÷ 4).
- Pi-Coding impact check (grep `.pi/` and `AGENTS.md` for the file's path; report hit count). Zero hits = Pi-safe; non-zero = surface for review before recommending.

**Delete-ledger note (regression-guard contract).** When a high-confidence recommendation is a **file deletion** (rule consolidated elsewhere, not a frontmatter-scope or extract), the report MUST state that applying it requires appending the deleted basename to `.claude/refs/harness-deleted-rules.txt` — the tombstone ledger read by the SessionStart hook `.claude/hooks/harness-regression-guard.sh`. Without the ledger entry, the guard cannot catch a silent re-add (the failure mode harness-audit-2026-06-07 found: the cross-repo sync tool re-shipped 4 consolidated rules; undetected 3 days). The audit itself stays read-only — it records the required ledger line in the report; the user appends it when they apply the deletion.

## Phase 5: Write the report

Read `helpers/report-template.md` just-in-time. Fill all `<…>` placeholders. Write to `.claude/PRPs/reports/harness-audit-<UTC-date>.md`. Do NOT commit — leave for the user to review and stage.

The report MUST include a "What changed since last audit" section: `Glob .claude/PRPs/reports/harness-audit-*.md`; if any prior reports exist, parse the most recent's "Total auto-load char count" and report the delta. First audit on this repo: write "no prior audit found".

## Phase 6: Memory write

Mirrors the `code-audit` and `post-task-retro` precedents. Call `memory_write_eval` with:

- `title: "Harness audit: <UTC-date> — <total ALWAYS-load chars> chars across <N> always-load files"`.
- `skill_or_tool: "harness-audit"`.
- `score`: 0.7 if any high-confidence wins; 0.55 if only watch items; 0.5 if no wins (a clean audit on a previously-trimmed harness is a successful run, not a failure).
- `tags: "harness,context-budget,brehon-fork,audit-only"`.
- `source_ref`: current branch.
- `content`: 5–8 lines. Top 3 wins by token impact, total estimated savings, the "what changed since last audit" delta line, one-line Pi-boundary check confirmation.

This eval also satisfies the Stop-hook retro requirement (per `.claude/rules/universal-guards.md` §4).

## Phase 7: Completion checklist + dogfood

- [ ] Phase 0 env snapshot printed
- [ ] Phase 0 Step 6 deterministic classification grep ran; table is the authority for Phase 1 + Phase 3
- [ ] Phase 1 Explore subagent ran with the classification table pasted in; ranked inventory table emitted; any `⚠ class-mismatch` rows surfaced
- [ ] Phase 2 cross-references counted
- [ ] Phase 3 composite scores computed via `helpers/scoring-matrix.md`
- [ ] Phase 4 three buckets emitted with explicit recommendations + Pi-impact grep per high-confidence row
- [ ] Phase 5 report written to `.claude/PRPs/reports/harness-audit-<date>.md`
- [ ] Phase 6 `memory_write_eval` written
- [ ] No source files outside the report were modified

### Dogfood

Per `.claude/lessons/feedback_dogfood_slash_command_specs.md`, every new skill ships a real-input dogfood note. This skill was authored after the 2026-05-09 trim pass (commits `59a184ef6..18e4a29c4`). Walked-through case at author time:

- Phase 1 would have correctly classified `governance-log-entry-kind-registry.md` as SCOPED (the prior audit subagent missed this; the verbatim frontmatter-detection instruction in this skill's Phase 1 prompt fixes it).
- Phase 2 would have detected that `decision-queue.md` Recipes 1–3 had no external citations to bodies (only to "Recipe 2 self-resolved" tags), making them a high-confidence extract candidate (matched the actual Pass 1b of that trim).
- Phase 4 "Out of scope" would have surfaced `.claude/rules/branch-manager.md` and `no-cargo-output-paste.md` as Pi-shared paths that must not be trimmed (matched the actual Pi-boundary check during that trim).

**Regression case (2026-05-31) — why Phase 0 Step 6 now exists.** The prose-reading frontmatter instruction in the Phase 1 subagent prompt was NOT sufficient: the 2026-05-31 audit's subagent classified `multi-lane-worktree.md` as SCOPED, but the file has no `---`/`paths:` frontmatter (line 1 is `# Multi-lane worktree discipline`) and ALWAYS-loads — confirmed at 5.5k tokens in a clean `/context`. It was the 3rd-largest always-load file and stayed invisible to compression scoring across BOTH the 2026-05-29 and 2026-05-31 runs. Root cause: classification was a model judgment call inside a read-heavy subagent. The fix moves classification to a deterministic parent-side grep (Phase 0 Step 6); the subagent now only records counts and flags mismatches. After this fix, the dogfood expectation is: a clean `head -1` + `paths:` grep over `.claude/rules/*.md` must reproduce the exact SCOPED set, and any file the subagent flags `⚠ class-mismatch` is a grep/file-malformation case for a human, never a silently-accepted reclassification.

If a future audit run does NOT match the prior trim's findings (e.g. classifies a SCOPED file as compression candidate, OR the Phase 0 grep set disagrees with the Phase 1 subagent's recorded classes without a logged mismatch note), the skill body has drifted — file a `.claude/decision-queue.json` entry citing this dogfood block.
