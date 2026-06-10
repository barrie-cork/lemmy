# Session retro — 2026-06-10 — pi-rls-pmd-parity

**Harness:** pi
**Session window:** 2026-06-10 interactive follow-up
**Branch at start:** `356b92bbc` approx (`work/governance-v0`)
**Branch at end:** `2bd8545ac` (`work/governance-v0`)
**Files touched:** RLS/PMD hooks, PMD CLI wrappers, pi harness docs/context, pmd skill
**Commits:** multiple auto(pi) commits plus explicit retro/coordination commits

## TL;DR

This session closed the biggest pi RLS parity gap: pi now has PMD reachability checks, lesson frontmatter reminders, lesson-to-PMD sync hooks, PMD CLI wrappers that work from Mac/P50 as well as EliteDesk, and a `harness-maintenance` mode for skill/lesson/retro metadata work. The main carry-forward is topology discipline: Brehon PMD is HTTP on the P50/Windows host, while other repos such as homeserver may use a different EliteDesk-centralized PMD shape. Future pi PMD work should start with `pmd-doctor.sh`, not by assuming one machine's topology applies everywhere.

---

## What surprised us

- **The PMD topology was easy to conflate across repos.** Homeserver memory documents describe an EliteDesk-centralized ssh-stdio PMD, while this Brehon repo uses the P50/Windows `pmd-http-mcp` service over Tailscale. The user had to correct the initial framing: check the existing Claude/P50/EliteDesk RLS docs and memories rather than infer from one repo.
- **The pi CLI wrappers were EliteDesk-biased.** `.pi/skills/pmd/SKILL.md` still pointed at `/srv/brehon-fork/scripts/...`; that works on EliteDesk but is wrong/noisy for Mac pi sessions. Switching to repo-relative `scripts/brehon/pmd-*.sh` made the skill portable.
- **Authentication failure looked like server failure until token sourcing was fixed.** A raw PMD query returned `unauthorized` because Mac pi did not source `/srv/brehon-fork/.env`. Updating wrappers to also source repo-local `.env` made hybrid PMD query succeed.
- **The new validator and doctor pattern paid off quickly.** `validate-skills.py --strict-style` caught the earlier skill-frontmatter class; `pmd-doctor.sh` gives the analogous one-command diagnostic for PMD endpoint/token/session issues.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Treat `bash scripts/brehon/pmd-doctor.sh` as the first diagnostic whenever pi PMD query/write or lesson sync fails. | Separates endpoint reachability, token sourcing, MCP initialize, and tool listing without leaking secrets. | done / minor | 2× this session (`unauthorized`, MCP session-id failure) |
| 2 | Keep PMD commands repo-relative in pi-facing docs/skills; avoid `/srv/brehon-fork/...` except when explicitly documenting EliteDesk. | Makes Mac/P50/EliteDesk pi sessions use the same command text from their current checkout. | done / minor | 2× this session (`pmd` skill and wrapper usage) |
| 3 | Add a short topology note when mentioning PMD in future cross-repo docs: Brehon = P50 HTTP service; homeserver = EliteDesk ssh-stdio central DB. | Prevents topology bleed between repos and machines. | minor | 1× strong correction this session + existing memory docs |
| 4 | Consider a single `/pmd-doctor` pi prompt/command wrapper around `scripts/brehon/pmd-doctor.sh`. | Makes the health check discoverable without remembering the shell path. | minor | 1× candidate |

## What to carry forward

- Pi should write durable learning artifacts to the same `.claude/PRPs/reports/` and `.claude/lessons/` surfaces as Claude Code, but use pi hooks/wrappers for PMD sync.
- `harness-maintenance` mode is the right mode for skill/lesson/retro/harness edits; it preserves the `.claude/` boundary while allowing explicit RLS work.
- PMD search from pi uses `scripts/brehon/pmd-query.sh`, which calls `memory_search_hybrid` by default. This is vector/semantic + FTS hybrid recall, not a plain keyword-only search.
- Local runtime files (`.pi/npm/`, `.pi/harness-factory/active.json`, SQLite WAL/SHM files) should stay ignored; durable profiles/hooks/scripts should stay tracked.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| PMD docs/memory search via `rg` + `pmd-query.sh` | 15 | 4 | medium | Found the Brehon-vs-homeserver topology split and confirmed hybrid recall. |
| `pmd-doctor.sh` / PMD wrapper hardening | 12 | 5 | medium | Converted `unauthorized`/session-id ambiguity into explicit token+endpoint checks. |
| Pi RLS hooks (`pmd-http-guard`, lesson reminder, lesson sync) | 20 | 3 | low | Brought pi closer to Claude Code's RLS loop for lesson authoring. |
| `validate-skills.py --strict-style` | 8 | 0 | none | Reconfirmed skill frontmatter stayed clean after updates. |
| Auto(pi) commits | 4 | 3 | low | Useful safety net, but created several small commits for one conceptual RLS parity pass. |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Pi RLS/PMD parity + PMD skill cleanup | ~10 | ~12 auto/manual | ~90 | 0 |

## Decisions to revisit

- Whether to add a first-class `/pmd-doctor` pi prompt for discoverability.
- Whether auto(pi) should suppress or batch commits in `harness-maintenance` mode, similar to CI-debug mode, to avoid fragmenting conceptual harness changes.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Add `/pmd-doctor` prompt/command that runs `bash scripts/brehon/pmd-doctor.sh` and summarizes safe output.
- [ ] Add a cross-repo PMD topology note/lesson: never copy PMD assumptions between Brehon, homeserver, phd-vault, and other repos without checking repo-local docs/config.
- [ ] Consider auto-commit suppression or batching for `/brehon-mode harness-maintenance`.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted: `feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`, `feedback_retro_task_complexity_score.md`._
