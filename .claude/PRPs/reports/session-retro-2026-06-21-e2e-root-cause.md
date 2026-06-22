# Session retro — 2026-06-21 — e2e cycle-4 root-cause + AS-register

**Harness:** claude-code
**Session window:** ~18:30 → ~21:25 UTC (~175 min)
**Branch at start:** `56abd2aea` (`governance-v0`)
**Branch at end:** `a296bffe1` (`governance-v0`); phase `4df0ed991` (`phase-m3-core-e2e-pilot`)
**Files touched:** 7 on gov-v0 (briefs/plan/handover); 3 source on phase (emergency_mute.rs, docker-compose.yml, docker-compose.e2e.yml) + DQ
**Commits:** gov-v0 7 (all explicit advisor/plan); phase 6 new (2 fix + 2 merge + 2 sync) + the daemon finalize-merges

## TL;DR

After a cycle-3 catch-fire stopped the e2e auto-fix loop, this session re-ran the 3
remaining m3-core-e2e-pilot targets with corrected env and **diagnosed all three failures
empirically rather than reactively**. The headline finding: the "two big root causes" the
handover framed as larger-than-a-compose-tweak collapsed into **1 env-run-change
(recording), 1 two-line test edit (emergency_mute), and 1 real harness fix
(room_provisioning)**. The room_provisioning root cause — `401 M_UNKNOWN_TOKEN` because
`matrix-conduit:v0.6.0` never registers the appservice (mounted `registration.yaml`
inert; admin-room-only registration) — was the load-bearing discovery; the planner's live
probe then proved Option B (swap to the real Tuwunel already in the pilot) returns
createRoom 200. Top change proposal: a lightweight task-status triage agent — `list_tasks`
(no filter) dumped 65K chars and forced a grep-the-spill + per-id fallback for the trivial
"what's running / what's next" question.

---

## What surprised us

- **All three "big" e2e root causes were small.** The handover (and my own initial read)
  treated recording+room_provisioning as needing a whole `lemmy_server` on :3000 and
  emergency_mute as a possible LiveKit routing config issue. Empirically: recording was
  3 env vars, emergency_mute was 2 lines, room_provisioning was a compose image swap.
  Surprise: high — the framing inflated the work ~5×.
- **My own falsifiable-hypothesis pass caught my own wrong theory.** Mid-session I
  hypothesised the emergency_mute `unavailable` was a docker-internal-hostname leak
  (`LIVEKIT_URL=ws://livekit:7880` rewritten to an unresolvable host). The re-run with
  `LIVEKIT_ADMIN_URL=http://localhost:7880` explicit + `LIVEKIT_URL` unset STILL failed
  — falsifying it. The LiveKit `psrpc` server logs then showed the real cause (psrpc
  timeout for a never-connected participant). The handover's *original* instinct ("maybe
  `unavailable` belongs in the accepted set") — which I'd initially dismissed — was right.
  Surprise: medium — a good reminder that the first plausible RCA, even my own, is a
  hypothesis until the logs confirm.
- **The defect was the 3rd of a family.** `matrix-conduit:v0.6.0` ignoring a mounted
  `registration.yaml` is the same never-booted-base-placeholder class as the prior
  sled→rocksdb and CONDUIT_PORT 6167→8448 bugs this phase. The base compose was simply
  never exercised through a real appservice room-create until e2e. Surprise: low (pattern
  was already named) but confirms the family is real.
- **I reaped the incoming session's freshly-up e2e stack at wrap.** At session-end I ran a
  clean-end-state check, saw the stack up, and tore it down — THEN noticed it was "Up About
  a minute," i.e. the next session had just brought it up to run the bundled gate. Surprise:
  medium — the end-state ritual fired in the wrong order (decide-then-check instead of
  check-then-decide).

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Build a lightweight task-status triage agent (Haiku, read-only) — or a `/task-status` skill — that returns ONE screen: running tasks (id+title+elapsed), last N done/failed, advisor next-action inference. Runs the heavy `list_tasks` INSIDE the subagent so the 65K spill never hits main context. Decide extend-`brehon-state-status`-vs-new at build time. | Eliminates the grep-the-spill + per-id `show_task` dance for the most common mid-phase poll (cost it 3 tool calls this session). | medium (new agent or skill) | 1× this session (user-requested) + the overlap is with an existing agent |
| 2 | Add a "lead end-state checks with container age" guard — when wrapping a session that brought up (or might share) a docker stack, run `docker ps --format '{{.Names}} {{.Status}}'` and read the AGE before any teardown decision; a <2-min-old stack means another session is driving → surface, don't reap. Codify as a one-line addition to the advisor session-wrap ritual (or a lesson `feedback_docker_end_state_check_age_before_teardown.md`). | Prevents reaping another session's in-flight stack (cost this session: ~1 min of the next session's work). Extends the surface-first ritual from git-worktrees to docker state. | minor (lesson) | 1× this session; 1× prior-analog (surface-first ritual exists for worktrees) → threshold met |
| 3 | Consider a `list_tasks --limit N` / `--since <ts>` MCP param (root-cause fix for #1). `status=running` is already bounded, but "recently done" needs the full dump today. | Even with the triage agent, a bounded `list_tasks` would let the agent itself avoid the spill. Cheapest if the daemon MCP is easy to extend. | medium (MCP change, may be out of advisor's control) | 1× this session |

## What to carry forward

- **Empirical-probe-before-fix discipline held and paid off twice.** Every RCA was verified
  against source or a live daemon probe (LiveKit twirp/psrpc logs, raw `curl` to :7880 and
  the docker-internal hostname, pilot :1236 reachability, the bridge `M_UNKNOWN_TOKEN`
  reproduction). The two falsified/vindicated hypotheses both came from running the probe
  rather than reasoning. Keep leading with the probe.
- **Per-decision AskUserQuestion with concrete preview blocks** worked cleanly — each fork
  (which :3000 path, how to handle the LiveKit error, plan-approval, gate bundling) gave the
  user a real choice with the trade-off visible. No churn, no re-litigation.
- **Verify-by-diff before advancing** — both AS-register impl tasks were checked for R-NOSRC
  (only compose+DQ touched), BUG-15 distinct domains, and all guardrail env vars before being
  treated as "done," not trusting the worker's success report. Caught nothing wrong this time,
  but that's the point — the check is cheap and the failure it guards against is expensive.
- **DQ merge-conflict-by-union** on Mode-B sync: when the phase-branch DQ and gov-v0 DQ
  diverge, resolve by keeping all phase pending + appending gov-v0-only resolved entries (by
  id). No entries lost; reproducible.
- **The planner running its own live feasibility probe** (createRoom 200 vs Tuwunel, 401 vs
  Conduit; the R-REGTOKEN shutdown discovered live) is exactly the anti-guess-and-fail
  behaviour the brief mandated. Keep mandating a probe in recon when an option's viability
  is the load-bearing unknown.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Daemon empirical probes (curl/docker logs/SSH) | 60 | 5 | high | the core of the session; falsified 1 wrong RCA (the ~5 wasted), confirmed all 3 real ones. The LiveKit psrpc-log read was decisive. |
| planning subagent #765 (Opus) | 40 | 0 | medium | ran a LIVE probe, chose Option B with proof, caught the 6-vs-1-implemented-tests discrepancy I'd missed. Strong. |
| impl-task #764 (em-mute, Sonnet) | 8 | 0 | none | clean 2-line edit, exact match to brief |
| impl-task #766 (as-reg-base, Sonnet) | 10 | 0 | none | clean compose conversion, R-NOSRC clean |
| impl-task #767 (as-reg-overlay, Sonnet) | 10 | 0 | none | clean, BUG-15 domains preserved |
| AskUserQuestion (×5 forks) | 15 | 0 | none | clean decision forks; no re-litigation |
| `mcp__junior-brehon__list_tasks` (no filter) | — | 8 | medium | 65K-char/1727-line dump → spill file → grep-the-spill + per-id `show_task` fallback. The friction behind "What to change" #1. |
| Mode-B sync (×3 brief→phase merges) | 5 | 3 | low | one DQ merge conflict (union-resolved); otherwise mechanical |
| end-state check at wrap | — | 1 | medium | reaped a freshly-up stack; "What to change" #2 |

## Complexity scores (heavy tasks only)

Per `feedback_retro_task_complexity_score.md`. All comfortably in the comfort zone — no
watchdog risk, no outliers. (max-log-silence not separately telemetered this session; runtimes
were short enough that it's not load-bearing — marked `—`.)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| #764 emergency_mute fix | 2 (emergency_mute.rs + DQ) | 1 | ~1.5 | — |
| #765 planning (AS-register plan) | 2 (plan + DQ) | 1 | ~14 | — |
| #766 as-reg-base | 2 (docker-compose.yml + DQ) | 1 | ~3 | — |
| #767 as-reg-overlay | 2 (docker-compose.e2e.yml + DQ) | 1 | ~3.5 | — |

Median: 2/1/3/—. No task >55min, >40min silence, or >8 files. Planning at ~14min is normal for
Opus plan-shaping with a live probe folded in.

## Decisions to revisit

- The bundled e2e gate (the actual validation run) is deferred to the incoming session — the
  Tuwunel-swap flow (bridge→Tuwunel→createRoom→`bridge_room` write) has NOT yet been seen
  live in-stack, only createRoom-standalone. If it surprises, that's a new finding worth its
  own note.
- Whether to extend `brehon-state-status` vs author a new `/task-status` (the #1 proposal) —
  worth a quick clarify when the build is picked up, to avoid two agents doing 80% the same job.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] #2 (end-state check leads with container age): promote to `.claude/lessons/feedback_docker_end_state_check_age_before_teardown.md` (cross-harness lesson; extends the surface-first ritual to docker state). Threshold met (1× here + surface-first-ritual analog in prior memory).
- [ ] #1 (task-status triage agent): new project-scope subagent at `.claude/agents/task-status.md` OR extend `.claude/agents/brehon-state-status.md`. User-requested; decide shape at build.
- [ ] #1 root-cause variant: investigate `list_tasks --limit/--since` MCP param (daemon-side; may be out of advisor scope).

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
