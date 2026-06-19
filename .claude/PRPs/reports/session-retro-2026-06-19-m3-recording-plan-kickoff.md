# Session retro — 2026-06-19 — m3-recording-plan-kickoff

**Harness:** claude-code
**Session window:** ~21:05 → ~21:25 UTC (~20 min)
**Branch at start:** `d56f7ed4e` (`governance-v0`)
**Branch at end:** `3987f975b` (`governance-v0`)
**Files touched:** 3 (planning brief +98, decision-queue.json, workflow_state_m3_core_recording.md [PMD System-1, not in git diff])
**Commits:** 2 (auto: 0, explicit: 2 — `ad4f0958e` brief, `3987f975b` clarify)

## TL;DR

Standard advisor kickoff for M3 Phase 5 (m3-core-recording): session-start ritual → authored the planning brief → ran `/brehon-clarify` (advisor-mode, 1 Q resolved) → cleared pre-queue gates → queued planning Junior #734. The single load-bearing finding: **the bootstrap was authored from a pre-resolution snapshot and flagged OQ-V2-04 as a pending obligation ("must land this phase") when it had ALREADY resolved two days prior** (`99-decisions:655`, 2026-06-17). Verifying bootstrap claims against live repo state before acting on them caught three stale items (OQ-V2-04, `recording_config` column, the recording const/`ROOM_KINDS` membership) — all already shipped — which would otherwise have become wasted planner work or a false stop-and-ask tripwire. Top change proposal: bootstrap files should carry a `VERIFIED_AT: <SHA>` annotation on time-sensitive obligations so the resume session knows which claims to re-check (the lesson `feedback_bootstrap_handover_verified_at.md` already exists — this session is a clean recurrence confirming it).

---

## What surprised us

- **The bootstrap's own watchlist drifted from repo reality within 2 days of authorship.** Watchlist #6 + tripwire #3 both said the OQ-V2-04 resolution block "must land in this phase's commit" — but it landed 2026-06-17, and the bootstrap was authored 2026-06-19 from a snapshot that predated the merge of that resolution into `99-decisions.md`. Same drift on the `recording_config` column (already shipped by m3-core-infra) and the `ENTRY_KIND_ROOM_RECORDING_UPLOADED` const + `ROOM_KINDS` membership (count already 13, registry already 72). Surprising-but-good: the recon-first discipline caught all three before they reached the planner.
- **The recording emit-seam is more complete than expected.** The const, the shim re-export, `ROOM_KINDS` membership, the bridge→binary callback client, AND the `recording_config` gate column all already exist. The genuinely net-new surface this phase is narrow: the S3/Egress/MinIO dep layer + the fetch authz endpoint + a new `RoomEventPayload` variant. The phase is "wire an existing seam to a new external service," not "build a recording subsystem."
- **PMD MCP (`memory_search_hybrid`) was unreachable from this session** — `No such tool available`. The pre-queue lesson search (advisor-orchestrator §2.3) couldn't run. Fell back cleanly to the `.claude/lessons/` glob (which the bootstrap §3 already enumerated), so no lessons were missed — but the canonical search step silently no-op'd. This is the HTTP-daemon-on-homeserver topology (per `project_pmd_homeserver_http_topology.md`); the MCP wiring isn't present in this CWD's tool surface.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When resuming from a bootstrap, grep the bootstrap for any "must land this phase" / "pending" / OQ obligations and verify each against live repo state BEFORE authoring the brief — fold satisfied ones into a "NOTE on bootstrap divergence" section. (Done this session; codify as a standing step in the bootstrap-resume ritual.) | Stops stale bootstrap obligations from becoming wasted planner work or false tripwires | minor (5 min/phase) | 1× this session + matches existing `feedback_bootstrap_handover_verified_at.md` + `feedback_handover_assumptions_need_empirical_verification.md` → recurrence met |
| 2 | Add a `VERIFIED_AT: <SHA>` annotation to time-sensitive obligation lines in future bootstrap files (the bootstrap author stamps the SHA the claim was true at; the resumer re-checks anything whose `VERIFIED_AT` is behind current HEAD). The lesson `feedback_bootstrap_handover_verified_at.md` already prescribes this — apply it at `/brehon-phase-transition` bootstrap-authoring time. | Resume session knows exactly which claims to re-verify vs trust | minor | existing lesson, under-applied (this bootstrap had no VERIFIED_AT on the OQ-V2-04 obligation) |
| 3 | Confirm PMD HTTP MCP is wired into the canonical `brehon-fork` CWD's `.mcp.json` (the §2.3 hybrid search is a documented pre-queue step that silently failed). If the canonical checkout is intentionally PMD-less for advisor meta-work, note that in advisor-orchestrator §2.3 so the no-op is expected rather than a surprise. | Either restores the pre-queue search OR documents its absence so it's not re-discovered as friction each session | minor (investigate) | 1× this session; promote-if-recurs |

## What to carry forward

- **Recon-first, verify-the-handover discipline.** Reading the PRD Phase-5 scope, OQ-V2-04 resolution, the sibling brief, and grepping the actual const/column/seam anchors BEFORE writing the brief — this is exactly what caught the bootstrap drift. The 8-ish grep/read calls cost ~3 min and saved a planner round-trip.
- **Model the brief on the immediately-preceding sibling.** `m3-core-emergency-mute-planning-1.md` was a near-perfect template — same 4-section shell, same recon-anchor block, same load-bearing-constraint framing, same `--bins`-bare-fn and Linux-compile-gate constraints. Reusing its structure verbatim (canonical-schema-first gate) made the brief fast and consistent.
- **Self-resolve clarify questions with a citation; leave genuine design decisions for the planner's `kind:blocker`.** Only 1 of the candidate questions was a real coverage gap (flag-gate location → resolved with the `bridge_room.rs:15` citation). The S3-crate choice is a *planning* decision, not a brief gap — correctly left for the planner to raise during planning. Resisting the urge to over-generate clarify entries kept the queue tight.
- **Atomic stage-then-commit-then-push for `.claude/` meta-edits on the clean canonical checkout.** Tree was confirmed mine + clean at session start (multi-lane hard-refusal #7), so each commit was a single uninterrupted add→commit→show burst, then push + `ls-remote` verify. No races.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Bootstrap read + session-start ritual | 8 | 0 | none | clean; surfaced the gov-v0 SHA-ahead-of-snapshot (descendant, benign) |
| Recon greps (const/column/seam anchors) | 15 | 0 | medium | caught 3 stale bootstrap obligations before brief authoring |
| `/brehon-clarify` (advisor-mode) | 10 | 0 | low | 1 Q resolved with citation; correctly declined to over-generate |
| `/precheck` (5 parallel probes) | 5 | 0 | none | all PASS; daemon-sync + cap + window + memory clean |
| `memory_search_hybrid` (PMD §2.3) | 0 | 2 | medium | tool unreachable; fell back to lessons glob (see What-to-change #3) |
| `create_task` #734 | 3 | 0 | none | queued cleanly, base governance-v0, brief reachable on daemon-local |

## Complexity scores (heavy tasks only)

No heavy tasks completed this session. Planning Junior #734 was queued but is still `running` at retro time (created 21:16 UTC) — its complexity score will be recorded at the m3-core-recording phase retro, not here. Brief authoring was a single ~98-line write + 1 edit, well under any envelope threshold.

## Decisions to revisit

- PMD HTTP MCP availability in the canonical CWD (What-to-change #3) — worth a 5-min check before the next advisor session that relies on the §2.3 pre-queue search.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] What-to-change #1 (verify bootstrap obligations against live state at resume): the lessons `feedback_bootstrap_handover_verified_at.md` + `feedback_handover_assumptions_need_empirical_verification.md` already cover this — no new lesson needed; instead consider adding a one-line "verify time-sensitive bootstrap obligations" step to the bootstrap-resume section of `advisor-orchestrator.md` §1 (or the `/brehon-phase-transition` bootstrap-authoring step). User approves.
- [ ] What-to-change #3 (PMD MCP wiring confirm): investigate-then-document; no lesson promotion until it recurs.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted: `feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`, `feedback_retro_task_complexity_score.md`. Auto-phase reliability section omitted — this session did not invoke `/auto-phase` nor mutate any `.claude/auto-state/*.json` (Step 0.5 trigger did not fire)._
