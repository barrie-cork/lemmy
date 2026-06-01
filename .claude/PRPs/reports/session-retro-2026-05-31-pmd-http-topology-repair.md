# Session retro — 2026-05-31 — pmd-http-topology-repair

**Harness:** claude-code
**Session window:** ~2026-05-31 00:00 → 03:00 (~3h, spanning the type-state work + PMD repair)
**Branch at start:** `71c4a6f2c` (`governance-v0`)
**Branch at end:** `7bde047cd` (`governance-v0`, pushed)
**Files touched:** ~18 across two threads (type-state lesson+annotations; PMD hooks/skills/rules/scripts)
**Commits this session:** 5 authored (`4e87daaa7`, `a6643c689`, `10c14bb25`, `7bde047cd`, 2 retros) + interleaved concurrent-session commits on the same branch

## TL;DR

Started as "assess the Apollo rust-best-practices skill" (rejected — conflicts with LemmyError
architecture), pivoted to authoring a Brehon type-state handler lesson + retrofit annotations,
then the user's "this should happen automatically: sync the new lesson to PMD" opened a deep
thread: the PMD had silently switched to an HTTP-daemon topology, breaking three separate
infrastructure legs. The session repaired all three (lesson auto-sync hook, weekly-review
backfill target, hosts-resolution) and root-caused the highest-impact finding: **semantic
search had been silently degraded to FTS5-only for the entire HTTP era** because the server
couldn't resolve `homeserver` to reach Ollama for query-embedding. The recurring lesson:
every "it's working" claim about the PMD was a hypothesis that needed a live probe — the
write path looked fine while the search path was blind.

---

## What surprised us

- **The HTTP topology had quietly broken three things at once, all silent.** The PMD switched
  from SQLite-via-env-var to an HTTP daemon (2026-05-30), and nothing surfaced the fallout:
  (1) `sync-lessons-to-pmd.sh` wrote to a dead daemon-local store, (2) weekly-review Step 1b
  backfilled the wrong DB and reported "MISSING: 0" (false confidence), (3) the server couldn't
  resolve `homeserver` so every hybrid search degraded to FTS5. Each failure mode emitted at
  most one stderr line. None blocked anything. The system *looked* healthy.

- **Semantic search was broken for ~24h and would have stayed broken indefinitely.** The
  `stderr.log` line `memory_search_hybrid: falling back to FTS5 (ollama error: This operation
  was aborted)` was the only signal, in a log nobody reads. Keyword queries kept working, so
  the degradation was invisible to anyone not running a deliberately non-keyword-overlapping
  semantic probe.

- **`${VAR@Q}` is not valid Python.** Building the lesson-sync hook, the bash parameter-transform
  quoting (`'it'\''s'`) broke the python payload builder on any lesson with an apostrophe.
  Caught only because I deliberately tested with apostrophe-laden prose. The fix (pass via
  `os.environ`) is obvious in hindsight but the failure was silent until the live test.

- **`memory_search` (FTS5) can't find a row by its filename.** The first idempotency design
  queried the basename and got "No memories found" — so it spawned duplicates instead of
  updating. `memory_get_file_context` (lookup BY file_path) was the right primitive.

- **The investigation collapsed a planned Junior task.** I was about to dispatch a Junior task
  to "investigate the HTTP server DB location," but reading `start-pmd-http-server.ps1` answered
  the only open question in 30 seconds (the server starts with the canonical laptop path). The
  task would have burned a dispatch to rediscover a one-line fact.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add a SessionStart probe that runs ONE non-keyword-overlap `memory_search_hybrid` and WARNs if it returns 0 hits OR if `stderr.log` shows a recent FTS5-fallback line | Surfaces semantic-search degradation within seconds of session start instead of never. This exact failure (server can't embed queries) was invisible for ~24h. | medium (new hook) | 1× this session; the class (silent PMD degrade) is 3× now (this + 2 prior backfill-gap incidents) — **threshold met** |
| 2 | `pmd-canonical-guard.sh` already checks HTTP reachability — extend it to also curl the server's configured `OLLAMA_URL` and WARN if unreachable | Catches the `homeserver`-unresolvable class at session start (the root cause here), not just server-down | minor (extend existing hook) | 1× this session |
| 3 | Document the `homeserver` → Tailscale-IP hosts requirement in the lane bootstrap checklist (`feedback_phase_lane_worktree_bootstrap_checklist.md`) — a fresh Windows machine / new lane won't have the hosts entry and will silently FTS5-degrade | Prevents the next machine-setup from rediscovering this 3-hour debug | minor (one checklist line) | 1× this session; pairs with the committed `add-homeserver-hosts-entry.ps1` |
| 4 | When building any shell→python bridge, pass values via `os.environ`, never interpolate (`${VAR@Q}` or `$VAR`) into python source | Eliminates the apostrophe/quote/backslash injection-break class | minor (discipline) | 1× this session; generalizes — worth a one-liner in a shared lesson |

## What to carry forward

- **Every "the PMD is working" claim is a hypothesis — probe the specific path.** The write path
  worked while the search path was blind. A single end-to-end probe (write → embed → semantic
  recall with non-overlapping vocabulary) is what distinguished "embedded" from "searchable."
  Per `feedback_verify_automated_reviewer_claims_against_compiler.md`, extended here to
  infrastructure claims.

- **Read the start script / config before dispatching an investigation.** `start-pmd-http-server.ps1`
  answered "where does the live store live" instantly. Gathering a brief's facts often resolves
  the question the brief was going to ask.

- **Live-test infrastructure hooks before trusting them.** The lesson-sync hook had two distinct
  bugs (`@Q`, FTS5-filename) that bash `-n` syntax checks could never catch — only an
  end-to-end run against the real server surfaced them. Per `pattern_test_against_reality_not_syntax`.

- **Idempotency by stable key.** The hook keys on `file_path` via `memory_get_file_context` and
  picks the lowest id on duplicates, so repeated edits converge on one canonical row instead of
  fanning out. Verified: 4 consecutive runs → 1 row.

- **PowerShell-vs-bash paste discipline for elevated commands.** The hosts-file one-liner failed
  3× through bash (`! ` prefix routes to bash; backticks + nested quotes break it). The robust
  path for the user: a pipe-into-`Add-Content` single statement run in their already-elevated
  PowerShell, not a `Start-Process -Verb RunAs` with embedded quoting.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Explore ×2 parallel (Apollo + handler inventory) | 30 | 0 | low | Clean synthesis; rejected Apollo skill with evidence |
| Explore ×2 parallel (sync mechanism + HTTP topology audit) | 40 | 0 | medium | Surfaced the 3-leg breakage; the topology audit was the pivot of the session |
| lesson-pmd-sync.sh build + live test | 25 | 15 | high | 2 silent bugs (`@Q`, FTS5-filename) cost ~15 min of debug, all caught by live testing — worth it |
| Reading start-pmd-http-server.ps1 (instead of Junior dispatch) | 45 | 0 | high | Collapsed a planned Junior task to a 30-sec read |
| Root-cause of semantic-search degrade (stderr.log + curl IP test) | — | 10 | high | ~10 min ruling out "ranking too low" before finding the resolution failure; the highest-value find |
| Hosts-file elevation (user-run) | 5 | 10 | medium | 3 failed paste attempts (bash/PowerShell quoting) before the pipe-into-Add-Content form worked |

**Net:** the session's leverage was the topology audit + the resolution root-cause — both would have compounded silently for weeks otherwise.

---

## Complexity scores (heavy tasks only)

No Junior tasks dispatched. All work advisor-authored, laptop-side.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Type-state lesson + 6 annotations + injection row | 8 | 1 | ~25 | ~5 |
| PMD HTTP-topology repair (hook + skill + rules + script + hosts) | ~10 | 3 | ~120 | n/a (interactive) |

---

## Decisions to revisit

- **Items #1 + #2 (semantic-search-degrade probe) should ship soon** — the failure is invisible
  by construction and the recurrence threshold is met. Worth a small follow-up session or
  folding into the next harness-audit.
- The `feedback_pmd_retro_check_http_store_split.md` lesson now carries a 2026-05-31 CORRECTION
  (the daemon-local store is a *lagged replica* via `pmd-snapshot.timer`, not a separate store —
  per the concurrent session's forensics). The HTTP-first fixes remain correct regardless.

---

## Promotion candidates (recurrence ≥ 2)

- [ ] **Silent PMD degradation needs a SessionStart probe** (items #1/#2): promote to a new hook
  + a lesson `feedback_pmd_semantic_degrade_silent.md`. Recurrence: 3× (this + 2 prior
  backfill-gap incidents) — threshold met.
- [ ] **shell→python bridges must pass via os.environ, never interpolate** (item #4): fold into
  an existing windows/bash lesson or a new one-liner. Recurrence: 1× here, but it's a sharp
  general-purpose footgun.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase section omitted: no `/auto-phase`
invocation or auto-state mutation this session._
