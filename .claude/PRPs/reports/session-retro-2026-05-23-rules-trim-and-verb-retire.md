# Session retro — 2026-05-23 — rules-trim and verb-retire

**Harness:** claude-code
**Session window:** 2026-05-22 ~22:00 UTC → 2026-05-23 ~01:00 UTC (~3 h wall-clock, ~5-7k tool-calls equivalent)
**Branch at start:** `9d4ae4099` (`governance-v0`)
**Branch at end:** `facae8450` (`governance-v0`)
**Files touched (my commits only):** 24
**Commits:** 2 explicit (`6ba1a50cd` rule-trim + `facae8450` verb-prune); 0 auto

## TL;DR

A two-pass context-budget cleanup in the canonical `brehon-fork` checkout while a concurrent fed-in-e task-1 advisor session ran in parallel. Pass 1 trimmed four auto-loaded rules (advisor-orchestrator, decision-queue, branch-manager, multi-lane-worktree) by extracting infrequently-fired procedural detail into four lazy-loaded `refs/` files, saving ~13 KB / ~3,300 tokens of session-start system-prompt load. Pass 2 retired 5 slash-command specs + 1 skill + 1 hook + their wiring (the prp-ralph trio + prp-pr + prp-commit), removing ~80 KB of slash-command surface that was already documented as "may drop later" in `brehon-reference.md`. The highest-leverage finding is procedural: a heading-anchor-preserving extraction pattern that lets refs/ absorb body text without breaking the ~30+ external citations (briefs, lessons, runlog, handovers) that reference rules by section anchor. The pattern is reusable for the still-open consolidation targets (deduplicate `/roadmap-next`, merge `retro-harvest` pair, fold `/precheck` into `/auto-phase`).

---

## What surprised us

- **The auto-loaded rule baseline was ~35K tokens, not 75K.** My initial estimate based on the "75K injected" framing in the user prompt was inflated because frontmatter-scoped rules (9 of 24 in `.claude/rules/`) don't load at session-start — they're path-gated and only trigger when matching files are read. The actual auto-load was ~140 KB / ~35K tokens; the remaining 40K is CLAUDE.md (6 KB) + MEMORY.md + system prompt + skills list + tool schemas. Re-reading the system-reminder injection at the top of the conversation would have caught this immediately.
- **Heading-anchor preservation was load-bearing, not cosmetic.** A grep sweep before the rule-trim pass found 8+ external citations referencing `branch-manager.md §"Telegram scope"`, `multi-lane-worktree.md §"Lifecycle"`, `multi-lane-worktree.md §"Worktree-aware DQ id discipline"`, etc — by anchor name, not by line number. Moving a section body to refs/ while keeping the section heading as a one-paragraph stub kept every citation resolving. This constraint shaped the extraction technique materially (couldn't just delete + replace; had to stub-and-forward-ref).
- **My initial "8 dead prp-core verbs" estimate was wrong.** A grep-for-citations sweep showed `prp-plan.md` is named load-bearing by `agents/planning.md:19`, `prp-implement.md` by `agents/impl-task.md:47`, `prp-review.md` by `bm-prp-review.md` + `branch-manager.md`, and the prp-ralph trio is wired to a settings.json Stop-hook + `prp-ralph-loop` skill + .gitignore entry. The actually-safe deletion set was 2 (prp-pr + prp-commit), with 3 more (ralph trio) requiring explicit user confirmation on wiring teardown. Original estimate was over-confident by 3×.
- **Surface-first ritual fired correctly when the sweep dirty-tree was discovered.** Per `multi-lane-worktree.md` 2026-05-22 surface-first ritual: when `git status` showed crates/* + bm-runlog.md + untracked debug/handover/retro files, I refused the `/command-retro --all` sweep per its hard refusal and surfaced the active fed-in-e task-1 implementation via AskUserQuestion instead of attempting a stash-and-restore that could have raced the concurrent validation. The fed-in-e session continued uninterrupted.
- **The `/command-retro --all` skill exists for exactly this question** but I drafted a 5-cluster qualitative consolidation map before invoking it. The skill would have produced an evidence-ranked report (PMD friction signals + retro mentions + DQ citations) — my map was structurally correct but un-prioritised against actual usage data. Per `feedback_principles_not_rules.md`: reach for the tool that's already there before drafting a one-off analysis.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When the user asks "audit X for trim/redundancy", check whether a `command-retro` / `harness-audit` / `code-audit` skill already covers the audit before drafting a qualitative analysis. Add a one-line check to `start-brehon` skill body: "before authoring a one-off audit, glob `.claude/skills/*/SKILL.md` + `~/.claude/skills/*/SKILL.md` for the audit domain". | Drops "I drafted a 5-cluster map before invoking the skill that produces a ranked report" pattern. Saves ~10-15 min per audit request. | minor (add 3-line check) | 1× this session, 0× prior — recorded, not promoted |
| 2 | The heading-anchor-preserving extraction pattern (rule section → refs/ file, with section heading stubbed as a one-paragraph forward-ref) belongs as a `.claude/lessons/feedback_rule_trim_anchor_preservation.md` lesson. The pattern is reusable across the remaining consolidation candidates (`/roadmap-next` dedup, `retro-harvest` pair merge, fold `/precheck` into `/auto-phase`). | Future rule-trim passes follow a documented procedure instead of re-deriving the constraint. | minor (~30-line lesson) | 1× this session, but the pattern applies to ≥3 known candidates. Threshold met. |
| 3 | Add an explicit "before any deletion sweep on slash commands, grep for citations in `.claude/agents/` + `.claude/rules/` + `.claude/hooks/` + `.claude/settings.json` FIRST" step to the `command-retro` skill body. My initial 8-verb estimate would have been corrected by this step before I told the user a number. | Prevents over-confident pre-grep estimates. ~5 min saved per delete pass. | minor (add to command-retro/SKILL.md before §"Step 1") | 1× this session (8 → 2 + 3) — promotion candidate if recurs. |
| 4 | The system-prompt injection size needs measurement, not estimation. Add a `harness-audit` skill invocation as the FIRST step of any rule-trim request: it produces empirical token counts per file. My 75K → 13K savings claim was useful but unverified against the actual context window post-trim. | Numeric claims about context savings become measurable instead of derived. | minor (skill exists; just invoke it) | 1× this session (75K initial estimate was 2× the truth) |
| 5 | When working in the canonical `brehon-fork` checkout while a fed-in-e (or any phase-lane) advisor session is running concurrently, NEVER use `git add -A` or `git add .` — always stage by explicit path. This already exists as `feedback_cross_session_commit_attribution_collision.md` but I almost reflexively typed `git add -A` once before catching it. Surface this as a SessionStart hook reminder when `git worktree list` shows ≥2 active phase-v1-* worktrees. | Mechanical defence against the cross-session add-sweep class. | medium (new hook script + settings.json registration) | 2× near-miss this session (rule-trim commit + verb-prune commit) — promotion candidate. |

## What to carry forward

- **Grep-for-citations before deletion.** Every slash command / rule section / skill file gets a `grep -rln "<name>" .claude/agents/ .claude/rules/ .claude/hooks/ .claude/settings.json .claude/skills/` sweep before any `git rm`. Caught 6 load-bearing refs the qualitative-only review would have missed.
- **JSON validation after settings.json edits.** Ran `python -c "import json; json.load(...)"` between editing the Stop-hook block and committing. The hook config is harness-critical; a malformed settings.json would have broken every future session opened in this CWD. Cheap (<1 sec) and load-bearing.
- **Fetch-before-push as standard pre-push step on the canonical checkout.** Per `multi-lane-worktree.md` §"Hard refusals" #6: `git fetch origin governance-v0 && git log HEAD..origin/governance-v0 -5` before every push to confirm no concurrent commit landed. Two successful applications this session.
- **AskUserQuestion to surface a hard refusal cleanly.** When the `/command-retro --all` sweep hit the dirty-tree refusal, I used AskUserQuestion to present three options (per-verb mode / stash-and-restore / abort) rather than picking one and surfacing afterward. The user chose abort, then immediately gave me the actionable redirect ("retire dead prp-core verbs"). Cleaner UX than "I can't do X, so I'll do Y".
- **Single-PR-with-all when the user asks for "single PR with all"** (and the underlying policy permits direct commit) — clarify before assuming. I correctly asked "PR vs direct commit per `phase-branch.md`" before committing, and the user picked direct-commit. Saved an unnecessary PR + CR cycle on meta-edits.
- **Heading-anchor preservation is the right invariant** for any rule-section extraction. Every section that's cited externally (by `§"Name"` anchor) keeps its heading as a stub. The refs/ file holds the body. External citations resolve to the stub which forwards to refs/. Applied 6 times this session without breaking a single external citation.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Edit (rule-trim, advisor-orchestrator.md §3.1 + §5.1 + §5.2 extractions) | 25 | 0 | none | Heading-anchor pattern worked cleanly; 5 sections extracted without breaking citations |
| Edit (rule-trim, decision-queue.md Archive policy + pre-v3 next-id) | 8 | 0 | none | Moderate-tier extraction per user preference |
| Edit (rule-trim, branch-manager.md Telegram + Findings YAML + Failure modes) | 6 | 0 | none | Three small stubs; refs/bm-mechanics.md new |
| Edit (rule-trim, multi-lane-worktree.md Lifecycle + Worktree-aware + Daemon side) | 6 | 0 | none | Heading anchors preserved; ~5 KB saved |
| Write (4 new refs/ files: stage-shape merged into auto-phase, advisor-validation, dq-mechanics, bm-mechanics, multi-lane-mechanics) | 10 | 0 | none | Followed `feedback_read_canonical_before_writing_spec.md`; cited existing refs precedent |
| AskUserQuestion (DQ extraction tier + session-start consolidation) | 8 | 0 | none | Forked cleanly on two design decisions before editing |
| Bash (grep -rln for citation sweep, pre-delete) | 12 | 0 | none | Caught 6 load-bearing prp-core refs the qualitative review missed |
| AskUserQuestion (sweep gate, dirty-tree fed-in-e conflict) | 5 | 0 | medium | Clean refusal + surface; user chose abort + redirect |
| `/command-retro --all` (skill invoked, then aborted) | 0 | 3 | low | Skill correctly refused dirty-tree per its hard refusal; ~3 min before abort decision |
| AskUserQuestion (PR vs direct-commit on rule-trim) | 5 | 0 | none | Policy-correct path chosen; avoided unnecessary PR+CR cycle |
| Bash (settings.json JSON validation pre-commit) | 0 | 0 | none | <1 sec; would have caught a malformed JSON if any |
| Bash (fetch-before-push, ×2) | 2 | 0 | none | Hard-refusal #6 discipline; clean both times |
| Initial qualitative consolidation map (pre-skill drafting) | 0 | 10 | low | Should have invoked `/command-retro --all` first; map was directionally correct but un-prioritised |
| Inflated 75K context estimate (pre-measurement) | 0 | 5 | medium | Re-reading system-reminder at session start would have shown ~35K, not 75K |

**Composite**: saved ~87 min vs wasted ~18 min → net saved ~69 min vs reading-and-editing the files manually. Friction score = 18 / (87 + 18) = 0.17. Composite (1 − friction) = **0.83**. Anti-inflation gate (per `evaluation-calibration.md`): >0.85 means re-read rubric — 0.83 is on the upper edge but defensible because (a) zero rework needed on either commit, (b) zero broken external citations post-edit, (c) zero settings.json malformations. Calibrating down to **0.70** per the rule "above 0.80 reserved for verified-with-all-outputs runs" — the trim claim is unverified against actual context window post-trim (Change #4 above).

**Final composite: 0.70.**

## Complexity scores

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. Two heavy commits:

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Rule-trim (`6ba1a50cd`) | 9 (5 modified rules + 4 new refs) | 1 | ~90 | ~6 |
| Verb-prune (`facae8450`) | 17 (5 deleted + 1 skill dir + 1 hook + 8 edited + others) | 1 | ~45 | ~5 |

Neither task hit the >55min / >40min log-silence / >8 files watchdog-risk zone substantively — the rule-trim commit's 9 files were all small targeted Edits (no Write-from-scratch except refs/). Both well within manual-session envelope.

## Decisions to revisit

- The qualitative 5-cluster consolidation map (proposals 1-8) is still actionable: deduplicate `/roadmap-next` (project vs user scope), merge `retro-harvest` + `retro-harvest-workspace` skills, audit `prp-prd` / `prp-debug` / `prp-issue-*` / `prp-codebase-question` for actual usage in the last 2-3 sub-phases (those are Tier-1/3 documented-as-kept but might still be net dead-weight). Cluster-A entry-point pruning (`/start-brehon` + `/check-dq` fold into one `/brehon` skill?) is also worth a clarify pass.
- The heading-anchor-preservation pattern (Change #2 above) deserves its own lesson file. Watch for one more occurrence to confirm threshold ≥2, then promote.
- Empirical token-count verification of the rule-trim claim: run `/harness-audit` next session-start to confirm the ~13K-saved claim against the actual context window injection.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #2 (heading-anchor extraction pattern): promote to `.claude/lessons/feedback_rule_trim_anchor_preservation.md` (recurrence: 1 this session × 6 extractions; threshold met if applied to remaining consolidation candidates)
- [ ] Change #5 (cross-session `git add -A` defence): promote to a SessionStart hook reminder when ≥2 active phase-v1-* worktrees detected. Pairs with existing `feedback_cross_session_commit_attribution_collision.md`. Recurrence: 2 near-misses this session + 1 confirmed prior incident (`c858aa7ab` 2026-05-22).
- [ ] Change #1 (skill-glob before audit drafting): update `start-brehon` skill body — single-line check
- [ ] Change #3 (pre-delete citation grep): update `command-retro` SKILL.md before §"Step 1"
- [ ] Change #4 (empirical token measurement before trim claims): invoke `/harness-audit` as Step 0 of rule-trim requests

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
