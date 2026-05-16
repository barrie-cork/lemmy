---
phase: v1-federation-inbound-a
role: impl-task
task: 0
brief_n: 1
authored: 2026-05-16
plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md
plan_task: "§13 Task 0 — Pre-flight harness audit + branch verification"
parent_phase_tip: dc484e2b9 (phase-v1-federation-inbound-a @ bm-cut #272)
cohort: "barrier (non-[P]) — dispatched ALONE before Cohort A (Tasks 1-5)"
---

# [role:impl-task] v1-federation-inbound-a Task 0 — pre-flight harness audit + branch verification — see .claude/PRPs/briefs/federation-inbound-a-impl-0.md

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is `phase-v1-federation-inbound-a` (you are on the Junior worktree branched off it). If `git branch --show-current` is anything else → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check (per `.claude/agents/impl-task.md` task-0 discipline): `date -u +"%a %H:%M UTC"` — if inside a forbidden window (Daily 02:55–04:15 / Sun 01:55–02:35 / Sun 03:55–04:30 / Wed 03:55–04:15 UTC) exit non-zero with `FORBIDDEN_WINDOW: <window>`. NOTE: Shape G is SUSPENDED (validate-pending-laptop mode per DQ #229/#231/#235) so cargo does NOT run on this worker — the forbidden-window cargo concern is reduced, but Task 0's probes still run here; keep the self-check.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-a Task 0 — pre-flight harness audit + branch verification (no commit)`

The actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-a Task 0 — see .claude/PRPs/briefs/federation-inbound-a-impl-0.md
```

## §2 Scope

Run **plan §13 Task 0's probe block VERBATIM** (probes 0..10 + Probe -1 submodule init). This is a **pure read-only harness audit — NO file creates, NO file modifies, NO commit**. The plan §13 Task 0 FILES YAML is `creates: []`, `modifies: []`.

### §2.1 The probe block (authoritative — read it from the plan, run it exactly)

Read `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` §13 "Task 0: Pre-flight harness audit + branch verification" and execute every probe in the fenced bash block exactly as written:

- **Probe 0** — Docker daemon (`docker ps`). DOCKER NOT RUNNING → exit 1 (testcontainers e2e needs it later; surface now).
- **Probe -1** — submodule init (if `git submodule status` shows uninitialised, `git submodule update --init --recursive`).
- **Probe 1** — branch is `phase-v1-federation-inbound-a`.
- **Probe 2** — baseline counts: `ENTRY_KIND_` in `crates/db_schema/src/source/governance/governance_log.rs` EXPECT **45**; `EXPECTED_SEED_COUNT` totals in `crates/api/api/src/governance/config.rs` EXPECT **127**.
- **Probe 3** — `schema.rs` clean of the 4 `-a` tables (`federation_peer`, `federation_inbox_dropped_log`, `federation_inbox_nonce`, `remote_moderation_label`). Present → contamination → exit 1.
- **Probe 4** — R9 InsertForm caller enumeration: `grep -rnE 'RemoteSanctionNoticeInsertForm\s*\{' crates/ tests/` + `FederationAttestationInsertForm\s*\{`. Capture both to `/tmp/fed-in-a-task0-rsn.log` + `/tmp/fed-in-a-task0-fa.log` and print. (This enumeration feeds Task 8's R9 callsite-padding scope — it MUST be in the Task 0 output the advisor reads.)
- **Probe 5** — trunk clean of `-a` enums in `migrations/`. Present → exit 1.
- **Probe 6** — DQ pending (informational; expect `[(229,'log')]` — the parked Shape-G reminder).
- **Probe 7** — PM-plugin-hooks-stable: 6 hook literals present in `crates/`.
- **Probe 8** — concurrent-PR check: `gh pr list` filtered for any open PR touching the shared files `-a` edits (schema.rs / config.rs / governance_log.rs ×2 / newtypes.rs / remote_sanction_notice.rs / federation_attestation.rs / e2e.rs / migration / registry). Print any matches (informational — surfaces cross-lane collision risk per DQ #232; v1-ship + v1-AD-e are the concurrent lanes).
- **Probe 9** — migration timestamp slot: `ls migrations/ | sort | tail -3`. EXPECT newest `2026-05-10-...`. **If `2026-05-17-000000-0000_add_federation_inbound_v1` already exists, report it (Task 1 picks `2026-05-18-...` instead).**
- **Probe 10** — wrapper availability: `scripts/brehon/cargo-check.sh` + `scripts/brehon/cargo-check.bat`.

### §2.2 Expected outcome

Probes 0-5, 7-10 exit 0; Probe 6 informational. On ANY contamination (Probe 3 / Probe 5 / Probe 9 collision / Probe 2 baseline mismatch), file a `kind: "blocker"` DQ (`from: "impl"`) per `.claude/rules/decision-queue.md` Recipe 1, commit + push it to the worker branch immediately (per "Mid-task visibility"), and STOP.

**No commit** if all probes pass — Task 0 produces only the probe output (returned to the advisor via the task-complete summary). The advisor reads Probe 4's InsertForm enumeration + Probe 8's concurrent-PR list + Probe 9's slot confirmation to gate Cohort A.

## §3 Required reading

- `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` §13 "Task 0" (the authoritative probe block — run it verbatim) + §0/§0.1/§0.2 (the 4 PRECONs + 3 clarify resolutions, for context on what the probes are guarding).
- `.claude/agents/impl-task.md` — your subagent contract (task-0 forbidden-window discipline, file-ownership, hard refusals).
- `.claude/PRPs/handovers/` — none yet (Task 0 is first; no prior-cohort handover).
- `.claude/rules/decision-queue.md` Recipe 1 (how to file a `kind: "blocker"` if contamination found) + "Mid-task visibility" (push the DQ immediately to the worker branch).
- `.claude/rules/pre-phase-harness-audit.md` (the R5 audit discipline Task 0 operationalises).

## §3a Handover from prior cohort

(none — Task 0 is the first task; no prior cohort.)

## §4 Constraints (hard rules)

- **NO file writes.** Task 0 `creates: []`, `modifies: []`. Pure probes. The ONLY write allowed is a `kind: "blocker"` DQ entry to `.claude/decision-queue.json` IF contamination is found (and that path is under `.claude/` — see harness-gap note below).
- **Harness-gap note (per DQ #235, resolved fix-daemon — interim escalation-and-transcribe):** if Task 0 needs to write a `kind: "blocker"` DQ entry to `.claude/decision-queue.json` and the Claude Code sensitive-file gate blocks it (the same gate that blocked planning Junior #271), DO NOT silently fail. Instead: (a) write the intended DQ-entry JSON object to a worktree-root file `TASK0_BLOCKER_DQ.json` (worktree-root writes succeed), (b) write a short `TASK0_ESCALATION.md` naming the contamination + the blocker entry, (c) commit both at worktree root + push, (d) STOP. The advisor transcribes per `.claude/rules/escalation.md` ("the escalation note IS the deliverable"). If all probes PASS, none of this applies — there is no write at all.
- **Branch:** `phase-v1-federation-inbound-a` (you are on a Junior worktree branched off it). NEVER commit to `governance-v0` / `main`.
- **No cargo.** Task 0 runs no cargo (Shape G suspended; validate-pending-laptop mode). The probes are `grep`/`ls`/`docker ps`/`gh`/`git` only.
- **Attribution:** any DQ entry is `from: "impl"`, `answered_by: null`. NEVER `"advisor"` / `"user"` / `"planner"`.
- **Probe 4 enumeration is load-bearing output.** The advisor uses the RemoteSanctionNoticeInsertForm + FederationAttestationInsertForm callsite enumeration to scope Task 8's R9 padding. Ensure both `/tmp/fed-in-a-task0-rsn.log` + `/tmp/fed-in-a-task0-fa.log` contents are in the returned task summary verbatim.

## §5 Validation gates

None (no code change, no cargo). Task 0's "validation" IS the probe block — all probes exit 0 (modulo Probe 6 informational) = pass.

## §6 Expected output (return to advisor)

```
## Task 0 complete — v1-federation-inbound-a pre-flight harness audit

Branch: phase-v1-federation-inbound-a (confirmed)
Probe 0 Docker: OK | NOT RUNNING
Probe 2 baselines: ENTRY_KIND_=<N> (expect 45); EXPECTED_SEED_COUNT totals=<...> (expect 127)
Probe 3 schema.rs -a tables: clean | CONTAMINATED
Probe 4 R9 callsites:
  RemoteSanctionNoticeInsertForm: <verbatim /tmp/fed-in-a-task0-rsn.log>
  FederationAttestationInsertForm: <verbatim /tmp/fed-in-a-task0-fa.log>
Probe 5 -a enums in migrations: clean | CONTAMINATED
Probe 6 DQ pending: <list>
Probe 8 concurrent-PR collisions: <list or none>
Probe 9 migration slot: newest=<dir>; 2026-05-17 slot <free | TAKEN -> Task 1 uses 2026-05-18>
Probe 10 wrappers: <present | missing>

VERDICT: <ALL PROBES PASS — Cohort A clear to dispatch | CONTAMINATION: blocker DQ filed #<id> — STOP>
```

## §7 Why this brief differs from the plan

It does not — Task 0's scope is exactly the plan §13 Task 0 probe block. This brief adds only: (a) the §0 forbidden-window self-check wording, (b) the §4 harness-gap interim escalation path for the contingency where a blocker DQ write is gated (per DQ #235; applies ONLY if contamination forces a DQ write — the happy path has zero writes), (c) emphasis that Probe 4's enumeration must be verbatim in the returned summary (the advisor needs it to scope Task 8's R9 padding).
