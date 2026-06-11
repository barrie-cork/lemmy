# Session retro — 2026-06-11 — pmd-stdio-transport-fix

**Harness:** claude-code
**Session window:** 2026-06-11 ~19:40 → ~21:15 UTC (~95 min)
**Branch at start:** `62da1239d` (`governance-v0`)
**Branch at end:** `62da1239d` (`governance-v0`) — unchanged
**Files touched:** 0 tracked (all work in PMD System-1 `.md`, daemon `.mcp.json`, homeserver PMD DB)
**Commits:** 0 (auto: 0, explicit: 0)

## TL;DR

User invoked `/auto-phase m2-late-2`; the skill correctly hard-refused (no `m2-late-2.plan.md` exists), and the last recorded user intent (handover Thread B) was `/auto-phase test` (a dogfood sandbox) anyway — so the session never advanced the state machine. Instead it spent its full length on the **readiness gate**: re-syncing the daemon trunk (trivial) and *proving the PMD worker-write round-trip* (not trivial). The proof exposed a real, never-before-caught bug: **no Junior worker has ever successfully connected the post-2026-06-11 homeserver-HTTP PMD** — an async-HTTP-MCP connect race where a fast worker acts before the `type:http` MCP finishes its async handshake, finds the tool absent, and silently skips its instruction while reporting success. An RCA subagent reproduced the race directly; the fix (switch worker PMD to a **stdio transport** → synchronous connect) was applied and proven by a second smoke task (homeserver PMD `941→942`). The most load-bearing carry-forward: **never trust a worker's "success" for an out-of-band side effect — verify the DB/external state directly**, and the corollary that `task_logs` returns only the newest-run *tail* (it gave a false-negative here).

---

## What surprised us

- **`/auto-phase`'s honest hard-refusal was the right outcome, not an obstacle.** `m2-late-2` has no plan, so the skill couldn't run — and the actual recorded intent was `/auto-phase test`. The arg the user typed (`m2-late-2`) diverged from the last handover's direction (`test`). Surfacing that divergence (rather than silently starting either) was correct, but it's a reminder that the typed arg is not always ground truth — the handover is.
- **A Junior worker reported `done`/`succeeded` while doing the wrong work, twice.** Task #650 (and the prior session's #649) reported success but wrote nothing to PMD. The worker silently fell through to its git-finalize ritual because its required MCP tool wasn't present at the moment it acted. "Success" meant "I finished *a* job cleanly," not "I did the job in the brief."
- **The post-cutover homeserver-HTTP PMD had NEVER actually worked from a worker.** The 2026-06-11 cutover (`.mcp.json` → `http://100.81.145.58:11435/mcp`) was treated as done in handovers, but no worker write had ever been proven. It was a never-proven config exposed at the gate, not a regression — surprising because two handover revs and a concurrent session all assumed it worked.
- **A concurrent CC session edited the memory record to claim "#650 ✅ wrote one memory, BLOCKER #1 cleared" — which was false.** The RCA falsified it minutes later. Concurrent sessions writing optimistic conclusions into shared PMD files is a live hazard (this is the same shared-`.claude/` contention noted in prior retros, now manifesting in System-1 memory).
- **`task_logs` MCP returns only the newest run's TAIL, not the full task transcript.** For task #651 it showed only the git-finalize phase with zero `memory_write_eval` hits — a false-negative that briefly led to a wrong "the worker ignored the brief again" conclusion. The DB row (correct content, correct timestamp) was the actual proof. The worker also *paraphrased* the dictated title, so an exact-title `LIKE` query missed the row that was right there.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Worker MCP servers should use stdio transport, not `type:http`** (done for project-memory on daemon `.mcp.json`; check `Ref`/others). For any HTTP-transport worker MCP that must survive, add to the worker brief: "FIRST action — confirm `mcp__<server>__<tool>` is present; wait+retry tool discovery if absent." | Eliminates the async-connect-race class structurally; workers reliably see their tools at turn 1 | medium (config change, verified) | 2× this session (#650, #649 prior) |
| 2 | **Never trust a worker's `result:success` for an out-of-band side effect (PMD write, external API, file on another host).** Always independently verify the side effect landed (DB `id > baseline` + content substring; external state probe). Add as a standing advisor-orchestrator discipline. | Catches silent-wrong-work; would have caught #650/#649 immediately | minor (discipline) | 2× this session + matches `pattern_bm_false_success_advisor_post_condition_catch` (5× prior) — promotion-grade |
| 3 | **`task_logs` returns only the newest run's tail — do not conclude "tool not called" from it.** Verify worker side-effects against ground truth (the DB/artifact), and query by `id > baseline` + content, never an exact-title `LIKE` (workers paraphrase). Captured in the new lesson. | Avoids the false-negative I hit; saves a wrong-conclusion loop | minor (lesson) | 1× this session (new) |
| 4 | **Don't mark a blocker "cleared" in shared PMD before the proof lands.** The concurrent session marked #650 cleared optimistically. Blocker-cleared edits should cite the verifying probe (e.g. "max_id 941→942"). | Prevents false-green propagating across sessions | minor (discipline) | 1× this session; ties to `feedback_cross_session_commit_attribution_collision` |
| 5 | **Verify never-proven infra at the gate, with a *write-and-verify* task, not a read-only smoke.** The handover already said "make the next one write-and-verify" — honor that. A read-only smoke (#649) proved nothing. | Turns "assumed working" cutovers into proven ones before they gate real work | minor | 1× (handover already flagged it) |

## What to carry forward

- **The readiness-pass-before-dogfood discipline paid off.** Choosing "full readiness pass first" surfaced a latent infra bug that would have silently corrupted the dogfood (auto-phase workers leaning on PMD would have hit the same race). Cheap insurance; do it before any `/auto-phase` that depends on worker PMD writes.
- **RCA-via-subagent is the right shape for a multi-probe diagnostic.** The general-purpose RCA agent (self-contained brief, explicit read-only/no-mutate boundaries, "reproduce before concluding" instruction) returned a high-confidence reproduced root cause in ~13 min / 161k tokens without polluting parent context. The brief-like-a-smart-colleague framing (§6.3) worked.
- **Falsifiable-hypothesis discipline held.** The RCA brief gave two competing hypotheses (H1 protocol-shape, H2 worker-init) and instructed "test both, don't assume." The agent ruled out the obvious-but-wrong ones (auth, literal-vs-env-token, OAuth) by direct probe before landing the real cause. Per `feedback_falsifiable_hypothesis_before_structural_fix`.
- **scp-a-script-then-ssh for quote-heavy daemon work.** Nested quoting through ssh+bash failed once (the first diagnostic); writing the script to `/tmp` and `scp`-ing it then `ssh bash`-ing it was clean every time after. Carry this for any multi-line daemon probe.
- **WAL-mode check before sharing a SQLite DB across two writers.** Confirmed `journal_mode=wal` before trusting the stdio-child + HTTP-service concurrent-writer setup. Cheap, load-bearing.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/auto-phase` Phase-0 prereq check | 15 | 0 | low | correctly hard-refused missing-plan; surfaced typed-arg-vs-handover divergence |
| AskUserQuestion (target + readiness, then PMD-fix + memory-fix) | 8 | 0 | none | clean forks; "full readiness pass" choice surfaced the bug |
| RCA general-purpose subagent | 40 | 0 | high | reproduced the async-connect race directly; ruled out 4 wrong hypotheses; 161k tokens, off parent context |
| Junior task #650 (HTTP smoke) | 0 | 12 | high | false-success, wrote nothing — but the failure IS the finding that triggered the RCA |
| Junior task #651 (stdio smoke) | 0 | 0 | medium | proved the fix; `task_logs` tail false-negative cost ~3 min of wrong reading before DB check resolved it |
| `task_logs` MCP tool | 0 | 4 | medium | newest-run-tail-only → false negative; DB row was ground truth |
| session-retro skill | — | — | — | this retro |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| RCA subagent (af3bf5ea) | 0 (read-only) | 0 | ~13.5 | n/a (interactive subagent) |
| Junior #650 (HTTP smoke) | 0 | 0 | <1 (27s wall) | n/a |
| Junior #651 (stdio smoke) | 0 | 0 | <1 (17s wall) | n/a |

No task stressed the watchdog envelope — all smoke tasks were sub-minute. The heavy cost was the RCA's 161k subagent tokens (deliberate, off parent context) and the human-loop diagnostic depth, neither of which the complexity metric is designed to capture.

---

## Lessons promoted this session

- **NEW:** `issue_note_async_http_mcp_connect_race_fast_worker.md` (System-1 PMD) — the async-HTTP-MCP connect race + the `task_logs` tail false-negative + "never trust worker success for out-of-band side effects." Recurrence-grade (2× this session, matches the 5× `pattern_bm_false_success_advisor_post_condition_catch`).
- **CORRECTED:** `project_auto_phase_test_dogfood_active.md` — fixed the concurrent session's false "#650 cleared" claim to reflect the real RCA finding + the proven stdio fix.

## Carry-forward state for the next session

- `/auto-phase test` is **clear to launch** — both readiness legs green (daemon trunk ff'd to `62da1239d`; PMD stdio write-round-trip proven via #651, `941→942`).
- Daemon `.mcp.json` project-memory is now **stdio** (backup `.mcp.json.bak-stdio-20260611T201010Z`). If the homeserver-HTTP PMD service is later wanted for *interactive* use, that's independent of the worker stdio config.
- Laptop `pmd-http-mcp` NSSM retirement (Thread A) is now genuinely re-evaluable but **out of scope** for the dogfood — workers no longer depend on the homeserver HTTP endpoint.
- Non-gating still-open: Telegram completion hook broken (junior-mcp#1 — poll, don't stall); stale `OLLAMA_URL=192.168.1.157` in `project-memory-http.service` (degrades hybrid-search embeddings only).
