# Session retro — 2026-06-10 — pmd-verification-auth

**Harness:** pi
**Session window:** 2026-06-10T18:45Z → 2026-06-10T19:28Z (~43 min)
**Branch at start:** `89f0d36e9` (`work/governance-v0`)
**Branch at end:** `89f0d36e9` (`work/governance-v0`)
**Files touched:** 0 tracked files in the Lemmy repo (local PMD vector DBs were backfilled)
**Commits:** 0 during this retro-scoped session (auto: 0, explicit: 0)

## TL;DR

The session verified PMD health across three topologies: phd-vault's local `pmd_cli.py`, homeserver/server-side `.project-memory` DBs, and P50's HTTP MCP daemon for `work/governance-v0`. The load-bearing outcome was that the current Lemmy/governance-v0 PMD path is healthy: P50 is reachable over Tailscale, the HTTP daemon is alive, `.project-memory/memory.db` integrity is OK, vectors were backfilled from 756/810 to 810/810, and `memory_search_hybrid` returns relevant results. The main change proposal is to make the existing PMD guard verify *auth + initialize*, not merely endpoint reachability, because this session briefly confused “daemon reachable” with “query path authorised.”

---

## What surprised us

- The phd-vault and Lemmy PMD topologies are intentionally different. PhD vault's recent fix is local-first (`.claude/scripts/pmd_cli.py`, local SQLite, `PROJECT_MEMORY_DB`/`OLLAMA_URL` overrides), while Lemmy/governance-v0 uses P50's HTTP MCP daemon and token-bearing wrapper scripts. Looking only at one repo's fix can mislead the diagnosis in the other.
- P50 coming online turned the initial failure from a network outage into an auth/topology check. Before Tailscale was active, `pmd-query.sh` timed out. After P50 returned, the same path had to be checked for token loading and MCP initialize, not just TCP reachability.
- `.pi/hook-scripts/pmd-http-guard.sh` can report HTTP `400` as “reachable,” which is correct for daemon liveness but not sufficient for “PMD query/write is authorised.” The user had to prompt for the authorization layer explicitly.
- The P50 DB was operational but not fully vectorised: `810` memories and only `756` vectors before manual backfill. Hybrid search worked after backfill, but the gap shows that “server alive” and “semantic recall complete” are separate gates.
- The earlier commit `89f0d36e9 chore(pi): wire RLS lesson and PMD hooks` was directly relevant: it already carried the token/env fallback pattern for hook-side PMD checks and made the eventual auth success unsurprising once P50 was reachable.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Extend `.pi/hook-scripts/pmd-http-guard.sh` to run an authenticated MCP `initialize` when `PMD_HTTP_TOKEN` is available, and print separate states: `reachable`, `auth-ok`, `auth-failed`, `no-token`. | Prevents future sessions from treating HTTP `400/401` reachability as sufficient evidence that `pmd-query.sh`/lesson sync will work. | minor | 2× this session: reachability check plus later auth check |
| 2 | Add a small `scripts/brehon/pmd-doctor.sh` that runs the exact three checks used here: endpoint initialize via `pmd-query.sh`, DB integrity/vector counts for the active `.project-memory/memory.db`, and an optional P50 SSH backfill status probe. | Converts a multi-step, memory-heavy diagnostic into one command with a clear pass/fail matrix. | medium | 3× this session: local, homeserver, P50 checked separately |
| 3 | Update `.pi/skills/pmd/SKILL.md` to include a topology note: phd-vault uses local `pmd_cli.py`; Lemmy/governance-v0 uses HTTP MCP on P50; homeserver has independent server PMDs/backfill timer. | Avoids cross-repo overgeneralisation when the user says “check PMD config in phd-vault” during a Lemmy session. | minor | 1× this session + prior phd-vault migration context |
| 4 | Add vector coverage to the guard/doctor output, not only daemon reachability. Suggested check: `SELECT count(*) FROM memories`, `SELECT count(*) FROM memory_vectors GROUP BY model`. | Catches the P50 756/810 and agent-grey 186/206 vector gaps before a user notices weak semantic recall. | minor | 2× this session: P50 and agent-grey gaps |

## What to carry forward

- Treat PMD health as a four-layer stack: network/Tailscale → HTTP daemon/auth → SQLite integrity/counts → vector coverage/hybrid recall. Do not stop at the first green layer.
- Use phd-vault memories/config as a useful comparator, but check the repo-local topology before applying it. The vault's `pmd_cli.py` fix confirmed the “local DB + OLLAMA_URL” pattern, not the P50 HTTP token path.
- Backfill immediately after finding vector count drift. The P50 backfill was low-risk and converted `810/756` to `810/810` during the same diagnostic loop.
- Clean diagnostic scratch files before closing. Temporary `.pi/*.json`/`.pi/*.err`/`.pi/*.log` artifacts were removed, leaving `git status` clean before the retro write.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`. Numbers don't have to be exact; they have to be defensible from the transcript.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `pmd` skill | 5 | 0 | low | Pointed to the wrapper path and established that `memory_search_hybrid` is the expected tool. |
| phd-vault PMD memory/config check | 10 | 3 | medium | Confirmed the recent local-first fix; slight waste from initially trying to apply it too directly to Lemmy's HTTP topology. |
| P50 Tailscale/SSH/port probe | 15 | 2 | medium | Quickly separated offline host from auth/server problems once Tailscale came back. |
| P50 DB vector backfill | 20 | 3 | high | Fixed real vector drift (`756/810` → `810/810`) while verifying rather than deferring. |
| `pmd-http-guard.sh` manual run | 5 | 2 | medium | Useful liveness signal, but it currently cannot prove authorization. |
| Commit archaeology (`89f0d36e9`) | 8 | 0 | low | Found that the env/token fallback fix already existed in the hook-side code. |

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. Format: `<files>/<commits>/<runtime-min>/<max-log-silence-min>`.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Cross-topology PMD verification + P50 backfill | 0 tracked | 0 | 43 | 4 |

## Decisions to revisit

- Whether `pmd-http-guard.sh` should treat HTTP `401` as a warning rather than success when a token is expected. The current “401 proves daemon reachable” behaviour is useful for liveness but weak for PMD readiness.
- Whether homeserver's `/srv/sync/vault-live/.claude/memory/memory.db` is still active. It showed index-integrity warnings and no vectors; if retired, document it as retired so future audits do not chase it.
- Whether `agent-grey`'s `206/186` vector gap should be auto-backfilled by the server timer or explicitly excluded from the timer's target list.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Extend `.pi/hook-scripts/pmd-http-guard.sh` to verify authenticated MCP initialize, not only HTTP reachability.
- [ ] Add `scripts/brehon/pmd-doctor.sh` for the four-layer PMD health check.
- [ ] Update `.pi/skills/pmd/SKILL.md` with the cross-repo PMD topology matrix.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
