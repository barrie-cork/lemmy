# Session retro — 2026-06-11 — junior-hook-bug-daemon-cleanup

**Harness:** claude-code
**Session window:** 2026-06-11 ~19:00 → ~19:40 UTC (~40 min active advisor wall-clock; investigation-heavy)
**Branch at start:** `f220f26f6` (`governance-v0`)
**Branch at end:** `50d150fc6` (`governance-v0`)
**Files touched:** 6 (1 handover ×4 revs, 3 runlog deletions, 2 foreign lesson files seen-not-touched)
**Commits:** 5 explicit (0 auto): `23798109e` (pushed concurrent session's lesson), `2bd989568`, `137fc1aad`, `835b8039a`, `50d150fc6`

## TL;DR

Resumed a handover to finish a PMD homeserver cutover + start an `/auto-phase` dogfood; the user redirected to "fix the Telegram completion hook first." That turned into a multi-hypothesis investigation that root-caused a **real junior-mcp binary bug** (`create_hook` hangs at "Extracting hook..." because the extraction subprocess spawns bare `claude -p` without `--mcp-config`, which stalls on MCP auto-init in any project dir) — filed as **barrie-cork/junior-mcp#1**. The investigation correctly falsified 4 wrong hypotheses before concluding (cosmetic warning, OAuth expiry, folder-trust, skip-perms flag), validating the falsifiable-hypothesis discipline. A follow-on "investigate other concerns" request surfaced and resolved daemon git-state drift (parked on stale `phase-m2-late-1`, 4.8 GB stale comparator WIP) — preserved the WIP to `/home/barrie/comparator-preserved/`, cleaned the daemon to a bm-cut-ready `governance-v0`. **Top change proposal:** add a session-start lesson + check that the daemon checkout is on `governance-v0` with a clean tree BEFORE relying on `base_branch` overrides — the daemon silently ignored my override because it was parked on a non-default branch.

---

## What surprised us

- **The daemon silently ignored the `base_branch` override on `create_task`.** I dispatched job-649 with `base_branch=smoke/pmd-roundtrip-20260611`; it branched off `phase-m2-late-1` instead — because the daemon checkout HEAD was parked on `phase-m2-late-1`, not `governance-v0`. No error, no warning. This is a latent footgun for any task dispatch that assumes `base_branch` is honored.
- **The "Junior MCP not configured" warning is a 4-year-long red herring.** It has fired on *every* `create_task`/`create_hook` since the project began (the EliteDesk `.mcp.json` never had an `mcpServers.junior` key), yet hooks were created successfully in the past (2026-05-30). It looks like the blocker but is purely cosmetic — `detectMcp` checks for a `junior` key and the binary continues past it. Nearly led me to edit the shared worker `.mcp.json` (the wrong fix).
- **`claude -p` hangs in project dirs but not in `/tmp`.** The decisive isolation: bare `claude -p` returns instantly in `/tmp` (no `.mcp.json`) but hangs (exit 124) in any `/srv/<project>` dir — even a *trusted* one. The variable is MCP auto-init at launch, plausibly aggravated by the PMD HTTP cutover. Workers are immune only because the daemon hands them an explicit `--mcp-config`.
- **The daemon's local comparator scripts had silently reverted a shipped safety guard.** The 5 modified `comparator-*.sh` files on the daemon weren't improvements — `comparator-judge.sh` had the `MIN_DIGEST` guard (shipped at `7ba8a20c7` to stop ghost scores) *removed*. A stale local checkout quietly undid a committed fix; would have re-bitten the next comparator run.
- **4.8 GB of untracked run artifacts sat in the daemon working tree.** The Pi comparator left planning-001..008 raw runs (4.8 GB) in `.claude/PRPs/comparator/runs/`, blocking a clean `git checkout`. Some were partially tracked, complicating the move.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Author `feedback_daemon_base_branch_override_ignored_when_parked.md`: before any `create_task` that relies on `base_branch`, run `ssh homeserver 'cd /srv/brehon-fork && git branch --show-current'` and confirm it equals the expected default (`governance-v0`) OR the exact override target exists as a daemon ref. The daemon branches from HEAD when the override target isn't a clean daemon-local ref. | Stops smoke/diagnostic tasks branching off a stale phase branch (contaminating isolation); stops impl tasks branching off the wrong base. | minor | 1× this session + ties to existing `feedback_daemon_local_trunk_stale_multi_lane.md` (≥2 with prior) |
| 2 | Add a session-start daemon-hygiene probe to the advisor ritual (`.claude/rules/advisor-orchestrator.md` §1 or a new hook): `ssh homeserver 'cd /srv/brehon-fork && git branch --show-current && git status --short \| head'` — surface if daemon is NOT on `governance-v0` or has a dirty tree, BEFORE any dispatch/bm-cut. | Catches parked-HEAD + stale-WIP drift at session start, not after a mis-dispatch. Would have caught both the base_branch issue and the 4.8 GB WIP up front. | minor | 1× this session; daemon-state drift is a recurring class (`feedback_daemon_local_trunk_stale_multi_lane.md`, `feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md`) |
| 3 | PMD issue-note id 941 already written; ensure the junior-mcp#1 fix lands (`--strict-mcp-config --mcp-config "{}"` for hook/schedule extraction). Until fixed, treat `create_hook`/`create_schedule` as non-functional — do NOT retry blindly each session (wastes ~5 min). Record in `feedback_daemon_telegram_completion_hook.md` "Known-broken until junior-mcp#1" note. | Stops every future session re-running the same hanging `create_hook` and re-investigating. | minor | filed; 1× here |
| 4 | When moving untracked artifacts out of a git tree on `/srv/`, remember `/srv/` is root-owned — `mkdir` under `/srv/<new>` needs sudo; use a barrie-owned dest (`/home/barrie/...`) and `set -e` so a failed `mkdir` aborts the chain BEFORE the `mv` (this session the `&&`-chain correctly aborted, but only by luck of ordering). Already in MEMORY.md cross-cutting patterns; reinforce in any cleanup brief. | Prevents a half-completed move that strands artifacts or (worse) a move into a dir that fails mid-way. | trivial | known pattern, re-confirmed |

## What to carry forward

- **The falsifiable-hypothesis discipline paid off hard this session.** I formed and *killed* 4 plausible root causes (cosmetic warning / OAuth / trust / skip-perms) before acting. The trust-toggle edit was the one speculative change I made — and I correctly flagged it as "did NOT fix the hang" rather than claiming victory. The PMD lesson search (prompted by the user's "did you check PMD?") surfaced id 675 proving a hook *was* created on 2026-05-30, which falsified my leading hypothesis before I edited the shared `.mcp.json`. **Keep: search PMD before any structural/config fix on shared infra.**
- **"Investigate other concerns you noticed" is a high-value prompt.** The daemon git-state cleanup (Thread D) was entirely from the user telling me to chase the loose threads I'd flagged in passing. Carrying small observations forward into an explicit investigation surfaced a reverted safety guard + 4.8 GB of stranded WIP that would have bitten the next session's bm-cut.
- **Atomic, my-files-only commits on the shared canonical checkout.** With 2-3 concurrent sessions on this checkout, I staged only my own files each commit (`git add <specific>` → `commit -F` → `git show --stat` verify) and left foreign WIP untouched. Worked cleanly across 5 commits with zero races. Per `feedback_canonical_checkout_foreign_wip_means_stop.md` — exactly the lesson a concurrent session had been editing live at session start.
- **`update-ref` not `reset --hard` for daemon trunk sync, and `mv` not `cp` for same-filesystem 4.8 GB moves.** Both lossless, both correct; both per existing lessons.
- **Preserve-before-discard for ambiguous WIP.** Saved the reverted-guard script edits as a 143-line patch and moved (not deleted) the 4.8 GB runs before cleaning the branch. No data lost even though the edits were almost certainly cruft.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `memory_search_hybrid` (PMD, prompted by user) | 20 | 0 | high | Surfaced id 675 proving a hook was created 2026-05-30 → falsified my "missing junior key blocks creation" hypothesis BEFORE I edited shared `.mcp.json`. The single highest-leverage call of the session. |
| Direct daemon SSH forensics (`strings` on binary, `journalctl`, `claude -p` isolation tests) | 30 | 8 | high | Isolated the real root cause (MCP-init hang). ~8 min wasted on the trust-toggle hypothesis that didn't pan out — but it was a cheap, reversible test. |
| `create_hook` (×2 via MCP + ×1 direct) | 0 | 6 | medium | Both MCP calls returned only the cosmetic warning; the direct daemon run revealed the EXIT 124 hang. The retries were necessary diagnosis, not waste, but ~6 min before the binary-level isolation. |
| GH issue creation (`gh issue create`) | 10 | 0 | none | Clean; precise root-cause writeup with isolation table + suggested fix. Durable artifact (junior-mcp#1). |
| Daemon git-state cleanup (Thread D) | 15 | 5 | high | Surprise: planning-001 was *partially tracked*, so the `mv` showed git deletions — needed a re-assessment before the final reset+checkout. 5 min on that detour; no harm (canonical results were in trunk). |
| Atomic canonical-checkout commits (×5) | — | 0 | none | Zero races across 5 commits with 2-3 concurrent sessions live. The discipline worked silently. |
| Foreign-WIP guard (session-start) | 5 | 0 | low | Fired correctly; the user confirmed the other session completed, unblocking work. Avoided a race. |

## Complexity scores (heavy tasks only)

No Junior impl-tasks, no cargo, no multi-file code edits this session — the complexity metric (`files/commits/runtime/log-silence`) is N/A. The session was investigation + git/SSH ops, not impl. The one heavy *advisor-side* task (daemon cleanup) touched 0 tracked files in the laptop repo (all ops were daemon-side SSH); its "complexity" is better captured by the 5-commit handover trail than the impl metric.

## Decisions to revisit

- **PMD homeserver round-trip is still unproven** — job-649 ran successfully but `homeserver max_id` did not increment. Next session must dispatch a *write-and-verify* task before retiring the laptop `pmd-http-mcp` daemon. (Open in handover Thread A.)
- **59 stale local branches on the daemon** (ab-cell/ab-test/planning-* cruft) — non-urgent cleanup candidate. Worth a one-off `git branch -D` sweep next quiet moment.
- **The daemon's `claude -p` MCP-init hang** is plausibly a side effect of the PMD HTTP cutover — if the homeserver HTTP PMD handshake is what's stalling bare `claude` launches, it could intermittently affect worker startup too. Watch worker spawn times; if they regress, the cutover is the suspect.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1 (base_branch-override-ignored-when-parked): promote to `.claude/lessons/feedback_daemon_base_branch_override_ignored_when_parked.md` (cross-references `feedback_daemon_local_trunk_stale_multi_lane.md`)
- [ ] Change #2 (session-start daemon-hygiene probe): update `.claude/rules/advisor-orchestrator.md` §1 polling-loop OR add a `.claude/hooks/` SessionStart probe
- [ ] Change #3 (create_hook known-broken until junior-mcp#1): add a "Known-broken" note to `.claude/lessons/feedback_daemon_telegram_completion_hook.md` (PMD id 941 already written as issue-note)

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
