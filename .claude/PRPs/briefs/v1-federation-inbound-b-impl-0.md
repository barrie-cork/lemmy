---
phase: v1-federation-inbound-b
role: impl-task
task: 0
brief_n: 1
authored: 2026-05-19
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
plan_task: "§13 Task 0 — Pre-flight harness audit + branch verification"
parent_phase_tip: 47d2fed30 (phase-v1-federation-inbound-b @ bm-cut #331 + advisor runlog reconcile)
cohort: "barrier (non-[P]) — dispatched ALONE before Cohort A (Tasks 1-3)"
---

# [role:impl-task] v1-federation-inbound-b Task 0 — pre-flight harness audit + branch verification — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-0.md

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is `phase-v1-federation-inbound-b` (you are on the Junior worktree branched off it). If `git branch --show-current` is anything else → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check (per `.claude/agents/impl-task.md` task-0 discipline): `date -u +"%a %H:%M UTC"` — if inside a forbidden window (Daily 02:55–04:15 / Sun 01:55–02:35 / Sun 03:55–04:30 / Wed 03:55–04:15 UTC) exit non-zero with `FORBIDDEN_WINDOW: <window>`. NOTE: Shape G is SUSPENDED (validate-pending-laptop mode per DQ #229/#276) so cargo does NOT run on this worker — the forbidden-window cargo concern is reduced, but Task 0's probes still run here; keep the self-check.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b Task 0 — pre-flight harness audit + branch verification (no commit)`

The actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-b Task 0 — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-0.md
```

## §2 Scope

Run **plan §13 Task 0's probe block VERBATIM** (Probe -1 submodule init + Probes 0..12). This is a **pure read-only harness audit — NO file creates, NO file modifies, NO commit**. The plan §13 Task 0 FILES YAML is `creates: []`, `modifies: []`.

### §2.1 The probe block (authoritative — read it from the plan, run it exactly)

Read `.claude/PRPs/plans/v1-federation-inbound-b.plan.md` §13 "Task 0: Pre-flight harness audit + branch verification" and execute every probe in the fenced bash block exactly as written:

- **Probe 0** — Docker daemon (`docker ps`). DOCKER NOT RUNNING → exit 1 (testcontainers e2e needs it later; surface now).
- **Probe -1** — submodule init (if `git submodule status` shows uninitialised `^-`, `git submodule update --init --recursive`).
- **Probe 1** — branch is `phase-v1-federation-inbound-b`.
- **Probe 2** — fed-in-a substrate present (PRECON-1 re-verify): `grep -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` EXPECT **54**; `grep -c "federation.inbound." crates/api/api/src/governance/config.rs` EXPECT **> 11**. A mismatch means fed-in-a substrate is absent/partial on this branch → contamination → file blocker.
- **Probe 3** — `inbox.rs` at the `lemmy_apub_activities` path (PRECON-2): `test -f crates/apub/activities/src/governance/inbox.rs` MUST pass; `test ! -f crates/apub/apub/src/governance/inbox.rs` MUST pass (the PRD §5.2/§9.1 descriptor-error path must NOT exist). Either assertion failing → exit 1, file blocker (PRECON-2 is BINDING — the wrapper home is the `lemmy_apub_activities` path, not the phantom `apub/apub` one).
- **Probe 4** — Phase-6 receivers + 3 `publish_*.rs` receive impls enumerated: `grep -nE 'pub async fn receive_remote_(sanction_notice|trust_attestation)' crates/apub/activities/src/governance/inbox.rs` EXPECT **2** matches (~:105, ~:195); `grep -nE 'async fn receive' crates/apub/activities/src/governance/publish_*.rs` EXPECT **3** matches (publish_sanction_notice.rs, publish_trust_attestation.rs, publish_label.rs). This enumeration is **load-bearing output** — the advisor uses it to scope Tasks 4-7 (the per-handler patch callsites). It MUST be verbatim in the returned summary.
- **Probe 5** — designed-breakage sweep (PRD §5.4 + watchpoint #3): `grep -nE 'PublishSanctionNotice::receive|PublishTrustAttestation::receive|PublishLabel::receive|ActivityTrait::receive' crates/server/tests/e2e.rs` EXPECT exactly **1** active direct-call site at ~:5244 inside `sanction_notice_round_trip` (other matches comments only). If MORE than one active direct-call site → a sibling test also breaks when the wrapper lands; this MUST be in the summary verbatim (the advisor scopes Task 9's fixture sweep from it).
- **Probe 6** — fed-in-a fixtures module present: `grep -n "mod v1_federation_inbound_a_fixtures" crates/server/tests/e2e.rs` EXPECT **1** match at ~:15093 (Task 9 mirrors this sibling's `LemmyResult<()>` Case-A shape verbatim).
- **Probe 7** — e2e.rs line count: `wc -l crates/server/tests/e2e.rs` EXPECT **15,482** (re-verify; baseline for Task 9's anchor-Edit append). Report the actual count regardless.
- **Probe 8** — Phase-6 receive bodies NOT already wrapped (idempotency): `grep -n "wrap_governance_inbound" crates/apub/activities/src/governance/publish_*.rs` EXPECT **no matches** (any match = trunk contamination / double-apply → file blocker).
- **Probe 9** — DQ pending state (informational): `python3 -c "import json; d=json.load(open('.claude/decision-queue.json')); print('pending:', [(e['id'], e.get('kind')) for e in d.get('pending',[])])"` — expect `pending: []` (DQ #276 user-resolved; #277/#278/#279 resolved). Report whatever is found.
- **Probe 10** — federation.inbound.* config keys landed (PRECON-1): `grep -nE "federation\.inbound\." migrations/2026-05-17-000000-0000_add_federation_inbound_v1/up.sql 2>&1 | wc -l` EXPECT **11** INSERTs. (The fed-in-a migration directory; confirms the 11 config-key seeds the wrapper reads exist.)
- **Probe 11** — concurrent-PR check: the `gh pr list` filter from the plan (any open PR touching the shared files `-b` edits: `crates/utils/src/error.rs` / `crates/apub/objects/src/protocol/governance/` / `crates/apub/activities/src/governance/` / `crates/db_schema/src/source/governance/governance_log.rs` / `crates/api/api/src/governance/governance_log.rs` / `crates/routes/src/utils/scheduled_tasks.rs` / `crates/server/tests/e2e.rs` / `.claude/rules/governance-log-entry-kind-registry.md`). Print any matches (informational — but per DQ #279 serial-phase policy NO other phase lane should be active; a match here is a surprise the advisor must triage).
- **Probe 12** — wrapper availability: `scripts/brehon/cargo-{check,clippy,test}.{sh,bat}` present.

### §2.2 Expected outcome

Probes 0-8, 10-12 exit 0; Probe 9 informational. On ANY contamination (Probe 2 baseline mismatch / Probe 3 PRECON-2 path violation / Probe 5 >1 direct-call site / Probe 8 already-wrapped / Probe 11 unexpected concurrent PR), file a `kind: "blocker"` DQ (`from: "impl"`) per `.claude/rules/decision-queue.md` Recipe 1, commit + push it to the worker branch immediately (per "Mid-task visibility"), and STOP.

**No commit** if all probes pass — Task 0 produces only the probe output (returned to the advisor via the task-complete summary). The advisor reads Probe 4's receiver/receive enumeration + Probe 5's designed-breakage sweep + Probe 7's line count + Probe 11's concurrent-PR list to gate Cohort A (Tasks 1-3).

## §3 Required reading

- `.claude/PRPs/plans/v1-federation-inbound-b.plan.md` §13 "Task 0" (the authoritative probe block — run it verbatim) + §1 Summary + §4 Solution + §0.x equivalents (the PRECON-1/PRECON-2 facts the probes guard; note this plan's §1-§10 carry the 4 binding clarify resolutions — DQ #273 option-2 / #274 option-a / #275 option-a / #250 — and the PRECON-2 path correction).
- `.claude/agents/impl-task.md` — your subagent contract (task-0 forbidden-window discipline, file-ownership, hard refusals).
- `.claude/rules/decision-queue.md` Recipe 1 (how to file a `kind: "blocker"` if contamination found) + "Mid-task visibility" (push the DQ immediately to the worker branch).
- `.claude/rules/pre-phase-harness-audit.md` (the R5 audit discipline Task 0 operationalises).

## §3a Handover from prior cohort

(none — Task 0 is the first task; no prior cohort.)

## §4 Constraints (hard rules)

- **NO file writes.** Task 0 `creates: []`, `modifies: []`. Pure probes. The ONLY write allowed is a `kind: "blocker"` DQ entry to `.claude/decision-queue.json` IF contamination is found (see harness-gap note below).
- **Harness-gap note (per DQ #235 — interim escalation-and-transcribe):** if Task 0 needs to write a `kind: "blocker"` DQ entry and the Claude Code `.claude/**` sensitive-file gate blocks it (the same gate that blocked planning #330 + bm-cut #331), DO NOT silently fail. Instead: (a) write the intended DQ-entry JSON object to a worktree-root file `TASK0_BLOCKER_DQ.json`, (b) write a short `TASK0_ESCALATION.md` naming the contamination + the blocker, (c) commit both at worktree root + push, (d) STOP. The advisor transcribes per `.claude/rules/escalation.md` ("the escalation note IS the deliverable"). If all probes PASS, none of this applies — zero writes.
- **Branch:** `phase-v1-federation-inbound-b` (you are on a Junior worktree branched off it). NEVER commit to `governance-v0` / `main`.
- **No cargo.** Task 0 runs no cargo (Shape G suspended; validate-pending-laptop mode). Probes are `grep`/`ls`/`docker ps`/`gh`/`git`/`python3` only.
- **Attribution:** any DQ entry is `from: "impl"`, `answered_by: null`. NEVER `"advisor"` / `"user"` / `"planner"`.
- **Probe 4 + Probe 5 + Probe 7 enumerations are load-bearing output.** The advisor uses Probe 4 (receiver fns + 3 publish_*.rs receive impls) to scope Tasks 4-7, Probe 5 (designed-breakage direct-call sites) to scope Task 9's fixture sweep, Probe 7 (e2e line count) for Task 9's anchor-Edit. All three MUST be in the returned task summary verbatim.

## §5 Validation gates

None (no code change, no cargo). Task 0's "validation" IS the probe block — all probes exit 0 (modulo Probe 9 informational) = pass.

## §6 Expected output (return to advisor)

```
## Task 0 complete — v1-federation-inbound-b pre-flight harness audit

Branch: phase-v1-federation-inbound-b (confirmed)
Probe 0 Docker: OK | NOT RUNNING
Probe 2 baselines: ENTRY_KIND_=<N> (expect 54); federation.inbound. config matches=<N> (expect >11)
Probe 3 PRECON-2 path: lemmy_apub_activities/governance/inbox.rs present | MISSING; apub/apub/governance/inbox.rs absent | PRESENT(violation)
Probe 4 receivers + receive impls (VERBATIM):
  receive_remote_*: <grep output — 2 lines ~:105 ~:195>
  publish_*.rs receive: <grep output — 3 lines>
Probe 5 designed-breakage sweep (VERBATIM): <grep output — expect 1 active site ~:5244>
Probe 6 fed-in-a fixtures module: present ~:<line> | MISSING
Probe 7 e2e.rs line count: <N> (expect 15482)
Probe 8 already-wrapped publish_*.rs: clean (no matches) | CONTAMINATED
Probe 9 DQ pending: <list>
Probe 10 migration config-key INSERTs: <N> (expect 11)
Probe 11 concurrent-PR collisions: <list or none>
Probe 12 wrappers: <present | missing>

VERDICT: <ALL PROBES PASS — Cohort A (Tasks 1-3) clear to dispatch | CONTAMINATION: blocker DQ filed #<id> — STOP>
```

## §7 Why this brief differs from the plan

It does not — Task 0's scope is exactly the plan §13 Task 0 probe block. This brief adds only: (a) the §0 forbidden-window self-check wording, (b) the §4 harness-gap interim escalation path for the contingency where a blocker DQ write is gated (per DQ #235; applies ONLY if contamination forces a DQ write — the happy path has zero writes), (c) emphasis that Probe 4/5/7 enumerations must be verbatim in the returned summary (the advisor needs them to scope Tasks 4-9). Mirrors the canonical sibling `.claude/PRPs/briefs/federation-inbound-a-impl-0.md` per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate.

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; per DQ #279 the canonical session drives fed-in-b impl under serial-phase policy — single active phase, no lane race). Brief committed on `governance-v0` before the Junior Task-0 task is queued. Junior branches from `phase-v1-federation-inbound-b` (the phase branch, tip `47d2fed30`) — `/precheck` re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC before queue (impl tasks branch from the PHASE branch, not trunk)._
