# Session retro — 2026-06-10 — harness-maintenance-followup

**Harness:** pi
**Session window:** 2026-06-10 follow-up after skill-frontmatter cleanup
**Branch at start:** `356b92bbc` (`work/governance-v0`)
**Branch at end:** `356b92bbc` (`work/governance-v0`)
**Files touched:** retro artifact only in this pass; prior pass touched skill frontmatter and pi harness files
**Commits:** 0 in this pass

## TL;DR

The follow-up confirmed that the harness-maintenance changes solved the immediate skill-frontmatter failure class, but surfaced two cleanup-policy gaps: `pi-harness-factory` recreates local state under `.pi/npm/` and `.pi/harness-factory/active.json`, and those paths are not ignored, so a clean tree becomes dirty again after normal profile/tool use. The highest-leverage next change is to add explicit ignore rules for local Pi package/runtime state while keeping committed harness profiles and scripts tracked.

---

## What surprised us

- **Cleanup did not stay clean.** After removing `.pi/npm/` and restoring `.pi/rust-analyzer-check.jsonl`, the working tree later showed `.pi/npm/` again plus `.pi/harness-factory/active.json`. That means local package/runtime state is reproducible and should not depend on manual cleanup.
- **`active.json` is runtime state, not repo policy.** The committed authority is `.pi/extensions/lemmy-hooks.ts` plus `.pi/harness-factory/profiles/*.json`; `active.json` records the user’s current profile choice and should not be proposed as a durable repo artifact.
- **The validator paid off immediately.** `python3 .pi/scripts/validate-skills.py --strict-style` produced 0 errors / 0 warnings after refinement, which is exactly the small repeatable gate the previous retro proposed.
- **Auto-commit granularity is useful but noisy for harness work.** The auto(pi) hook produced several small commits for related harness-maintenance edits, then a manual bundling commit. This is safe, but less readable than a single intentional harness-maintenance commit.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add root `.gitignore` entries for `.pi/npm/` and `.pi/harness-factory/active.json`. | Keeps local Pi package/runtime state from reappearing as dirty worktree noise after factory/profile use. | minor | 2× this session (`.pi/npm/` reappeared after cleanup; `active.json` appeared after profile sync) |
| 2 | Add `.pi/rust-analyzer-check.jsonl` to the same ignore discussion, or decide explicitly that it remains tracked as a baseline artifact. | Prevents generated Rust analyzer JSON churn from recurring as accidental diff noise, or documents why this file is intentionally tracked. | minor | 1× concrete churn this session; prior commit history shows it was intentionally tracked once |
| 3 | For future harness-maintenance tasks, switch to `/brehon-mode harness-maintenance` first and consider temporarily disabling per-edit auto-commit if multiple coordinated files will change. | Produces fewer fragmented commits while preserving the `.claude/` boundary and app-code block. | minor | 1× this session, likely recurring for harness edits |
| 4 | Keep `.pi/scripts/validate-skills.py --strict-style` as the standard final gate after any skill metadata change. | Converts skill-loader failures into a fast local validation step. | already done | 3× original failures plus successful follow-up validation |

## What to carry forward

- Treat `.pi/harness-factory/profiles/*.json`, `.pi/harness-factory/README.md`, `.pi/extensions/lemmy-hooks.ts`, and `.pi/scripts/validate-skills.py` as durable repo artifacts.
- Treat `.pi/npm/` and `.pi/harness-factory/active.json` as local/runtime artifacts unless a future design explicitly says otherwise.
- Use the new validator before answering “skill conflicts” reports. It catches missing frontmatter, YAML parse failures, and description-length issues in one pass.
- Keep the dual-harness boundary intact: metadata fixes under `.claude/skills/` are legitimate only when explicitly requested or in `harness-maintenance` mode.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `.pi/scripts/validate-skills.py --strict-style` | 8 | 0 | low | Replaced repeated ad-hoc YAML checks with one command; confirmed 30 skill files clean. |
| `/brehon-mode harness-maintenance` design | 5 | 1 | medium | Solves the blocked `.claude/skills` edit path, but should be used proactively next time. |
| `pi-harness-factory` runtime state | 0 | 4 | medium | Recreated `.pi/npm/` and wrote `active.json`, causing dirty-tree noise after cleanup. |
| Auto(pi) per-edit commits | 3 | 3 | low | Preserved edits safely, but fragmented one conceptual harness-maintenance change across several commits. |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Harness-maintenance implementation + cleanup review | ~7 | 6 auto/manual | ~45 | 0 |

## Decisions to revisit

- Should `.pi/rust-analyzer-check.jsonl` remain tracked? It is generated JSON output and churned this session, but it was originally committed as a local memory/rust-analyzer artifact. Decide once, then either ignore it or document why it stays tracked.
- Should pi auto-commit be suppressible for all harness-maintenance work, not just CI-debug mode?

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Add `.pi/npm/` and `.pi/harness-factory/active.json` to `.gitignore` as local Pi runtime/package state.
- [ ] Add a short note to `.pi/harness-factory/README.md`: profiles are tracked, `active.json` is runtime-local.
- [ ] Decide `.pi/rust-analyzer-check.jsonl`: either add to `.gitignore` and remove from tracking in a deliberate cleanup, or document its purpose.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted: `feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`, `feedback_retro_task_complexity_score.md`._
