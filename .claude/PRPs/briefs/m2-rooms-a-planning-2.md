# Planning-Revision Brief — m2-rooms-a: T2 juror-sourcing seam (DQ a3d0e9941441-055, option-a)

**Phase:** m2-rooms-a
**Branch:** phase-m2-rooms-a (planner finalize-merges the amended plan onto this branch)
**Authored:** 2026-06-06
**Authored by:** advisor (canonical brehon-fork / governance-v0 session)
**Plan target (AMEND, do not rewrite):** `.claude/PRPs/plans/m2-rooms-a.plan.md`
**Supersedes nothing** — this is an incremental amendment brief layered on `m2-rooms-a-planning-1.md` (the original full brief). Read that first for full context; this brief changes ONE seam.

---

## 0. Role + dispatch line

`[role:planning]` — amend the m2-rooms-a plan to close the T2 juror-sourcing gap (DQ a3d0e9941441-055, option-a): the bridge cannot learn which jurors to invite because `CaseTransitionEvent` carries no identities. Augment the event payload at the binary producer.

---

## 1. Why this brief exists (the gap the original plan shipped)

The m2-rooms-a plan §13 **Task 2** requires the bridge provisioner to *"resolve
each assigned juror via `PuppetMap::ensure_puppet` and invite as
`Juror-<suffix>`"* (plan §13 T2 step (d), confirmed at plan line ~608). The
headline acceptance criterion (plan §1 line 19, §16a Story 1 line ~867) is
*"exactly the 5 assigned jurors appear as `Juror-<suffix>`"*.

**This is unimplementable as scoped.** Verified against `phase-m2-rooms-a` HEAD
`56de6da77`:

- `CaseTransitionEvent` (`crates/api/api_common/src/governance.rs:860-866`)
  carries **only** `case_id`, `old_status`, `new_status`, `community_id`,
  `target_type` — all integer/enum. Its doc-comment is explicit: *"Integer +
  enum fields only — no usernames/emails (the bridge owns pseudonym
  resolution)."* The bridge receives this and nothing else for a transition.
- The bridge has **no workspace-DB access** — `services/bridge/Cargo.toml` has
  no diesel/sqlx/postgres/lemmy_db dependency (R8 toolchain isolation). It
  *cannot* run the `jury_assignment ⨝ actor_pseudonym` join itself.
- The only source of `Juror-<suffix>` handles is
  `jury_assignment(case_id → person_id)` ⨝ `actor_pseudonym(person_id →
  pseudonym: String)` — both workspace-side
  (`crates/db_schema/src/source/governance/jury_assignment.rs:21-24`,
  `actor_pseudonym.rs:20-21`).
- None of the 4 resolved clarify-DQs (-051/-052/-053/-054) addressed juror
  sourcing. The original brief's T2 row (planning-1 §7 line 162) said "member
  invite loop with `Juror-<suffix>` puppets" but never specified the source.

So `PuppetMap::ensure_puppet(brehon_user: &str)` needs a per-juror identity
string the bridge has no way to obtain. The acceptance criterion cannot be met.

## 2. The decision (already made — option-a, user-directed planner round)

**DQ a3d0e9941441-055 resolves to option-a: augment `CaseTransitionEvent` with
a `juror_pseudonyms` field, populated at notify-fire time from the
`jury_assignment ⨝ actor_pseudonym` join in the binary producer.**

This matches the **M1 DM-relay precedent** (verified
`services/bridge/src/relay.rs:28-36`): `BridgeNotifyPayload` carries
`brehon_sender`/`brehon_recipient` as `String` — **the binary computes the
identities and pushes them into the payload; the bridge never GETs identities
back.** Option-a is the precedent-consistent shape. Option-b (a new GET-back
route) was rejected: it adds a 3rd binary route + a network round-trip on the
provisioning path and *contradicts* the no-GET-back M1 model.

The producer site already supports this trivially — verified
`crates/api/api_utils/src/bridge_notify.rs:53-90`,
`governance_case_after_transition`:
- It already holds `let pool = &mut context.pool();` (line 60).
- It already performs an async DB read before constructing the payload
  (`GovernanceMessagingConfig::read_current`, lines 60-66).
- Adding a conditional `jury_assignment ⨝ actor_pseudonym` fetch there and
  setting `juror_pseudonyms` on the event is a natural extension, not a new
  pattern.

## 3. What the planner must do (the amendment scope)

This is an **incremental amendment**, not a re-plan. Change only what option-a
requires; leave T0, T1, T3, T4a, T4b, T5, T6 structurally intact except where
they consume the new field.

### 3.1 New payload field (workspace-side type change)

- Add `pub juror_pseudonyms: Vec<String>` (or `Option<Vec<String>>` — planner's
  call; prefer `Vec<String>` defaulting empty for non-jury transitions, which
  serde-defaults cleanly and keeps the bridge's match arms simple) to
  `CaseTransitionEvent` in `crates/api/api_common/src/governance.rs:860-866`.
  Update the doc-comment: the field carries *pre-resolved pseudonymous handles*
  (NOT usernames/emails) — so the "no usernames/emails; bridge owns pseudonym
  resolution" invariant is preserved in spirit (the binary resolves to
  *pseudonyms*, never to real identities; ADR-015 holds).
- This is a pure type/serde change — `PartialEq`, `Serialize`, `Deserialize`,
  `Debug`, `Clone` all derive trivially on `Vec<String>`.

### 3.2 Producer population (workspace-side logic change)

- In `crates/api/api_utils/src/bridge_notify.rs`
  `governance_case_after_transition` (lines 53-90): **conditionally** populate
  `juror_pseudonyms`. The planner MUST specify *which* `new_status` values
  warrant the fetch — jury-bound transitions only (the C2.1 jury path:
  `JurySelection` and any status where a jury room is provisioned/active per
  the PRD C2.1 scenario). For non-jury transitions the field is empty — no
  fetch, no cost.
- The fetch is `jury_assignment ⨝ actor_pseudonym` keyed by `case.id`,
  returning `Vec<String>` of pseudonyms. **Check first** whether
  `crates/api/api/src/governance/actor_pseudonym_helper.rs` already exposes a
  by-case juror-pseudonym fetch (it is the existing pseudonym-resolution
  helper); if so, reuse it; if not, add a thin read method (planner decides
  home: a new method on `JuryAssignment`, or a helper fn). Do NOT duplicate an
  existing query.
- **Crate-location caution:** `bridge_notify.rs` is in `api_utils`;
  `actor_pseudonym_helper.rs` is in `api`. Verify the dependency direction
  before reusing the helper (api_utils → api may not be a legal dep edge). If
  the helper isn't reachable from `api_utils`, the read belongs on the
  `db_schema` source model (`JuryAssignment` / `ActorPseudonym`), which both
  crates can call. The planner MUST resolve this dep-direction question in the
  amended task's MIRROR/GOTCHA notes — it determines where the fetch fn lives.

### 3.3 Bridge consumer (already in T2 — now actually implementable)

- T2 step (d) becomes implementable as written: the provisioner reads
  `event.juror_pseudonyms` and loops `PuppetMap::ensure_puppet(&pseudonym)` →
  invite as `Juror-<suffix>`. The MIRROR is `relay.rs`'s use of
  `payload.brehon_sender`/`brehon_recipient`. Update the T2 IMPLEMENT text to
  source the loop from the payload field (remove the implicit "the bridge
  knows the jurors" assumption).

### 3.4 Plan sections to amend (enumerate explicitly — R5 discipline)

1. **§13 Task 2** — rewrite step (d) to source jurors from
   `event.juror_pseudonyms`; add the producer-population as a new IMPLEMENT
   file or a new sub-task (see §3.5 split question below).
2. **§13 — producer task** — the workspace-side change (3.1 + 3.2) is NEW work
   not in any current task. The planner decides: fold into a re-scoped T2, OR
   add a dedicated task (e.g. **T1.5 / T2a** "augment `CaseTransitionEvent` +
   populate at producer"). See §3.5.
3. **§5.2 per-task complexity ceiling** — recompute the file/crate counts for
   whatever task now owns the workspace change. T2 was "3 files, 1 crate";
   adding 2 workspace files (`governance.rs`, `bridge_notify.rs`) + possibly a
   `db_schema` read method would push it to ~5 files / 2-3 crates — **over the
   ≤4-file / ≤2-crate Sonnet ceiling**. This is the structural reason to split
   (§3.5). Update the §5.2 per-task table to reflect the new shape.
4. **§15 Validation commands** — the workspace-side task's validate-pending
   command changes from `["cd services/bridge && cargo check"]` to **also**
   `["./scripts/brehon/cargo-check.sh --workspace --features full"]` (the
   payload + producer change is workspace Rust). The bridge consumer task keeps
   `cd services/bridge && cargo check`. **Do NOT mix the two on one task** —
   plan §15 / R8 forbids it (bridge has no `full` feature). This is the
   toolchain-boundary crossing the advisor declined to make in a brief.
5. **§16a Story 1 / §16 acceptance criteria** — no wording change to the
   acceptance criterion itself ("exactly the 5 assigned jurors as
   `Juror-<suffix>`"), but add a checkpoint that the producer populates
   `juror_pseudonyms` for jury transitions (so the criterion is now traceable
   to an implementation path).
6. **§7 R-guardrails** — the zero-Matrix-deps invariant (R1) is UNAFFECTED
   (the new field is pure `Vec<String>`, no Matrix types; `cargo tree
   --workspace | grep -cE 'matrix-sdk|ruma'` still 0). State this explicitly so
   the reader knows the boundary held. R2 (zero workspace migrations) also
   UNAFFECTED — no schema change (jury_assignment + actor_pseudonym already
   exist from v1-JM-b).

### 3.5 The split question the planner must resolve (and document)

The workspace-side change (payload + producer) and the bridge-side consumer
change **cross the toolchain boundary** (workspace `--features full` vs bridge
`cargo check`). Per §5.2 ceiling + R8, they likely belong in **separate
tasks**:

- **Workspace task** (new, e.g. T2a): augment `CaseTransitionEvent`
  (`governance.rs`) + populate at producer (`bridge_notify.rs`) + the read
  method if needed (`db_schema` or `api`). Validation: `--workspace --features
  full`. This task ships *before* the bridge consumer so the field exists.
- **Bridge task** (re-scoped T2, or T2b): consume `event.juror_pseudonyms` in
  the provisioner. Validation: `cd services/bridge && cargo check`.

The planner decides the exact numbering/split and MUST update the
`requires:`/dependency edges (the bridge consumer `requires:` the workspace
task, since it deserializes the new field). Document the split rationale in
§5.2 (the same way the original split T4 → T4a/T4b for the ceiling).

## 4. Constraints (stop-and-ask tripwires + hard rules)

- **Do NOT add a GET-back route for juror sourcing.** Option-b was rejected
  (contradicts the no-GET-back M1 model). The juror list travels IN the
  payload, computed by the binary producer. (The separate
  `GET /governance/bridge/messaging-status` read route in T4b is a *different*
  thing — that's for `messaging_enabled` + reveal-threshold config, not juror
  identities; leave it as planned.)
- **ADR-015 (pseudonymisation) is load-bearing here.** The producer resolves
  to **pseudonyms** (`actor_pseudonym.pseudonym`), NEVER to real
  usernames/emails/person display names. The bridge must never receive a real
  identity. State this in the amended task's GOTCHA. If the planner finds any
  path that would leak a real identity into `juror_pseudonyms`, STOP and raise
  a blocker DQ.
- **STOP if** the only available join requires a crate-dependency edge that
  doesn't exist (api_utils → api). Resolve by moving the read to `db_schema`
  (callable from both) — but if that's also blocked, raise a `kind: blocker`
  DQ rather than inventing a new dep edge.
- **STOP if** populating `juror_pseudonyms` would require making the
  fire-and-forget notify path blocking or slow (it adds one indexed DB read on
  `case_id` for jury transitions only — acceptable; but if the planner sees a
  fan-out or N+1, surface it). R3 (non-blocking notify) still holds.
- **§2.4 mandatory lesson injection re-runs for the NEW workspace files.** The
  workspace task now touches `crates/api/api_common/src/governance.rs` and
  `crates/api/api_utils/src/bridge_notify.rs`. Walk the §2.4 file-class table:
  the producer does a single DB read (not 2+ writes → no transaction lesson);
  it's not e2e.rs, not a migration, not a `#[cfg(feature="full")]` gate add.
  The injection that DOES apply: `feedback_features_full_workspace_only.md`
  (the workspace task uses `--features full`; per-crate `-p` is invalid). The
  bridge consumer task keeps the bridge-context injection set from planning-1
  §6 (NOT the Lemmy-workspace lessons — R8). The planner lists which lessons
  fire per amended task in the plan's task notes.
- **Do NOT touch** T0, T1, T3, T4a, T4b, T5, T6 except: (a) T5's chain-emission
  is unaffected; (b) any `requires:` edge that must now point at the new
  workspace task. Keep the amendment surgical.
- **MIRROR discipline:** the producer-population MIRROR is
  `bridge_notify.rs:60-66` (the existing async DB read before payload
  construction) + `relay.rs:28-36` (payload-carries-identities precedent).
  Cite both file:line in the amended task.

## 5. Required reading (planner must read these first)

- `.claude/PRPs/briefs/m2-rooms-a-planning-1.md` — the original full brief (all
  WPs, scope constraints, DoD, lesson injections). This brief layers on it.
- `.claude/PRPs/plans/m2-rooms-a.plan.md` — the plan being amended. Read §13
  Task 2, §5.2, §15, §16/§16a, §7 R1-R8 before editing.
- `crates/api/api_common/src/governance.rs:840-880` (on `phase-m2-rooms-a`) —
  `CaseTransitionEvent`, `PrivateMessagePayload`, `BridgeNotifyPayload` (the
  type to augment + the precedent struct).
- `crates/api/api_utils/src/bridge_notify.rs:53-90` (on `phase-m2-rooms-a`) —
  `governance_case_after_transition` (the producer site to populate).
- `services/bridge/src/relay.rs:28-36` (on `phase-m2-rooms-a`) —
  `BridgeNotifyPayload`/`brehon_sender`/`brehon_recipient` (the M1
  payload-carries-identities precedent).
- `crates/db_schema/src/source/governance/jury_assignment.rs:21-24` +
  `actor_pseudonym.rs:20-21` — the join source models.
- `crates/api/api/src/governance/actor_pseudonym_helper.rs` — check for an
  existing by-case juror-pseudonym fetch before adding one.
- DQ `a3d0e9941441-055` in `.claude/decision-queue.json` (resolved) — the full
  question/options/context the advisor verified.

## 6. Deliverable

An **amended** `.claude/PRPs/plans/m2-rooms-a.plan.md` on `phase-m2-rooms-a`
(planner finalize-merges as usual) that:
1. Adds the workspace-side payload+producer change as a task (split per §3.5),
   with full IMPLEMENT/MIRROR/GOTCHA/VALIDATE blocks matching the plan's
   existing task shape.
2. Re-scopes T2's juror loop to source from `event.juror_pseudonyms`.
3. Updates §5.2 (complexity ceiling recompute), §15 (per-task validation
   commands — workspace task gets `--features full`), §16a (traceability
   checkpoint), §7 (state R1/R2 unaffected), and the `requires:` edges.
4. Lists which §2.4 lessons fire per amended task.

Do NOT author implementation code. Do NOT touch `crates/**` /
`services/bridge/**` source files — only the plan markdown. The amendment goes
through the normal plan-approval gate (advisor runs §3.4 DoD smoke + §3.5
watchpoint gates on the amended §15/§4 before re-approving).

---

_Authored by advisor. Read-only reference for planning Junior. Do not modify
during the planning run. DQ a3d0e9941441-055 (option-a) is the source of
authority for this amendment._
