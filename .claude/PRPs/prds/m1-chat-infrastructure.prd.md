# Sub-PRD: M1 — Chat Infrastructure (Bridge + Matrix + 1:1 DM)

**Parent**: `.claude/PRPs/prds/v2-messaging-rtc.prd.md` (umbrella) — this sub-PRD is the
detailed spec for the first cluster (M1, formerly V2a).
**Created**: 2026-06-03
**Status**: DRAFT — ready for `/prp-plan`
**Scheduled**: NOW — operator-demand trigger satisfied; user scheduled M1 this session.
**Naming**: "M1" per ADR-016 (renamed from V2a to end the ADR-010 v2-security naming collision).
The umbrella PRD body still uses V2a/V2b/V2c in places; M1 = V2a = chat infrastructure.

> **Scope decision (user, 2026-06-03):** M1 implements **chat infrastructure ONLY**.
> No ADR-016 backplane work (no B-fetch evidence adapter, no B-actor link flow, no
> B-publish/sanction propagation). The "first reference integration of the B-fetch/
> B-publish/B-actor contract" framing in ADR-016 is honoured at **M2**, where governance
> triggers first do real work. M1 proves the bridge architecture in isolation per
> Key-Hypothesis H1. See `99-decisions-and-open-questions.md` 2026-06-03 entry.

---

## §1. Vision and Goals

### 1.1 The problem M1 solves

A Brehon fork in operation has no real-time communication channel of any kind. Lemmy's
`to: [ObjectId; 1]` fixed-size array (`crates/apub/objects/src/protocol/private_message.rs:19-33`)
is a hard block on group messaging; there is no real-time transport, no audio/video, and
no rich media in private messages. Before any *governance-triggered* room can exist (M2) or
any town hall can run (M3), the underlying chat plane — a Matrix homeserver bridged into the
Brehon fork — must exist and be proven to work in isolation.

M1 stands up that plane: a Matrix homeserver (Tuwunel), a Brehon↔Matrix application-service
bridge daemon, puppeting of Brehon users into Matrix, 1:1 direct messages with rich media
(text + image + voice note), manually-provisioned community Matrix rooms, and an admin config
panel. Crucially, M1 also proves the **clean-disable posture**: with `messaging_enabled = false`,
the Brehon instance runs exactly as a v0/v1 governance-only platform with zero messaging
surface and zero behavioural change.

### 1.2 M1 goals

1. **Prove H1** — the mautrix-style application-service bridge pattern adapts to the Lemmy
   1.0-beta fork using only the existing Extism PM plugin hooks, with **zero** changes to the
   Lemmy binary's PM path, for text + image + voice-note 1:1 messaging.
2. **Bridge daemon exists** as a separate Rust process outside the Brehon workspace
   (`services/bridge/`), following the [07 §1.2](../../docs/brehon-law-inspired-network/07-operations-and-federation.md)
   external-services pattern — separate host capability, narrow credentials, no Brehon DB
   write access.
3. **Tuwunel homeserver deployed** as the sidecar, with the appservice registration and the
   containerised-non-host-network topology handled per OQ-V2-10's deployment caveat.
4. **Admin config panel** persists identity-policy and lifecycle-policy settings across restart.
5. **Clean-disable posture** — `messaging_enabled = false` preserves the v0 governance-only
   posture; the e2e governance suite passes unchanged.
6. **Soft-pause rollback story** (OQ-V2-09) is specified and implemented: disabling messaging
   after enabling it is a reversible soft pause, not a destructive teardown.

### 1.3 Non-goals (M1)

Named, not detailed — each belongs to a later phase or is permanently out of scope:

- **Governance-triggered rooms** (jury rooms, appeal rooms, emergency rooms) → **M2**. M1
  rooms are manually provisioned only.
- **`Room::*` governance-log entries** → **M2**. M1 writes nothing to the hash chain.
- **Town halls / MatrixRTC / LiveKit / Element Call / mic-passing** → **M3**.
- **ADR-016 backplane** (B-fetch evidence adapter, B-actor link flow, B-publish sanction
  events) → **M2+** per the 2026-06-03 scope decision. M1 users are puppeted Matrix accounts,
  NOT B-actor-linked portable identities.
- **Vanilla-Lemmy interop** — resolved OQ-V2-08 (a): Brehon↔Brehon only. No degraded-mode
  `Announce` mirror to non-Brehon peers.
- **Ad-hoc user-initiated rooms** (outside any governance context) → permanently out of scope
  per umbrella §"What We're NOT Building".
- **E2EE of governance chat against the Brehon operator** → out of scope (ADR-015: operators
  are trusted).
- **Voice notes as Lemmy PM content** → out of scope; Lemmy PM schema stays text-only, rich
  media lives Matrix-side.
- **Matrix reverse-proxied through the Brehon binary** → permanently out of scope; preserves
  "Brehon binary has no Matrix deps" forever.
- **Data-export from deleted rooms** → out of scope (V2-successor sub-PRD if ever).

---

## §2. Scope

### IN scope (M1)

1. **Tuwunel homeserver deployment** — containerised, with appservice registration, federation-
   disabled launch posture, and the `ip_source`/loopback handling per OQ-V2-10.
2. **`services/bridge/` Rust daemon** — Matrix application-service implementing: puppet-on-
   first-contact, 1:1 DM relay (Brehon↔Matrix), rich-media relay (image + voice note via Matrix
   media repo), manual community-room provisioning command surface.
3. **Bridge↔Brehon integration via existing PM plugin hooks** — the 7 hooks verified at the
   call sites in umbrella §Research-Summary Q1, used read-only / notification-only. No new hooks.
4. **Admin config panel** — `governance_messaging_config` table + the admin write path for:
   `messaging_enabled`, identity-policy (per-room-type default, with the jury/appeals pin
   non-overridable per ADR-015), lifecycle policy (`hard_delete_after_days`). Persists across
   restart.
5. **Clean-disable posture** — `messaging_enabled = false` is the default; when false, zero
   messaging surface, governance e2e unchanged.
6. **Soft-pause rollback** (OQ-V2-09 resolved below) — disable-after-enable is reversible.
7. **`messaging_user_id` mapping** — Brehon user ↔ puppet Matrix ID. **NOT** the ADR-016
   B-actor portable ID (that's M2+); this is a bridge-local puppet map.
8. **Deployment runbook + docker-compose** — side-by-side topology, the four "verify before
   committing" Tuwunel items, restart fixture for the admin-panel-persistence test.
9. **Design-doc updates ADR-016/umbrella defer to "M1 schedule time"** that are M1-relevant:
   [06 §2.2](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) plane-
   boundary extension (add chat plane), [06 §7](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md)
   threat rows for bridge/homeserver, [07 §5](../../docs/brehon-law-inspired-network/07-operations-and-federation.md)
   `messaging_enabled=false` posture row. (The ADR-016 §4 federated-app-planes architecture
   extension is M2-relevant — deferred with the backplane.)

### OUT of scope (deferred — named, not detailed)

See §1.3. The single most important deferral: **the ADR-016 backplane contract.** M1 does not
prove it; M2 does.

### Decision: where the bridge code lives

**Greenfield crate vs separate repo** (umbrella §Technical-Approach deferred this to "V2a
sub-PRD"): **separate directory `services/bridge/` inside the brehon-fork repo, but NOT a
Cargo workspace member.** Rationale: keeps the bridge versioned alongside the fork it integrates
with (single source of truth, single CI surface) while preserving the hard property that the
Brehon binary's `Cargo.toml` gains zero Matrix dependencies — the bridge has its own
`Cargo.toml` and its own dependency tree, excluded from the workspace `members` list and built
separately. A fully separate repo is rejected for M1: it splits the audit trail and doubles the
CI/secrets bootstrap for a solo dev with no second consumer yet. Revisit if a second app
integration (ADR-017+) wants to share bridge code.

> **Plan-author note:** confirm the workspace-exclusion mechanism against the live
> `Cargo.toml` `members`/`exclude` lists at plan time — `services/bridge/` must be in
> `exclude` (or simply outside any `members` glob) so `cargo build --workspace` never pulls
> Matrix deps. This is a watchpoint, not a settled line.

### Decision: Person AP `matrix_user_id` field

**Reuse the existing `matrix_user_id` field, do NOT add a new AP field, in M1.** The umbrella
deferred "extend Person with `brehon_matrix_room_advertisement` OR reuse `matrix_user_id`" to
the sub-PRD. M1 decision: **neither extension crosses the AP wire in M1.** The puppet mapping
is bridge-local (`messaging_user_id` map, §2 IN-scope #7); the legacy self-declared
`matrix_user_id` field (`crates/apub/objects/src/protocol/person.rs:49`) is left exactly as
upstream. Adding a Brehon actor-extension on the wire is an ADR-016 B-actor concern → deferred
with the backplane to M2+. This keeps M1's "zero changes to the Lemmy binary's federation
surface" property intact and avoids the "first Brehon-added actor-extension on the wire" risk
the umbrella flagged.

---

## §3. Why this PRD shape (and not larger)

M1 is deliberately the smallest cluster that proves H1. Three forces keep it small:

1. **Strict-sequential M1→M2→M3** (user Q10 2026-04-19) — M1 must smoke-test the bridge before
   any governance trigger layers on top. Pulling M2/M3 scope forward would couple bridge-
   architecture risk to governance-trigger risk.
2. **ADR-016 backplane deferred** (user 2026-06-03) — the contract details (B-fetch SPI,
   B-actor link flow) are M2+. M1 builds neither, so OQ-ADR016-01 and OQ-ADR016-03 do not gate
   it.
3. **Greenfield, no migration** — `governance_log.entry_kind` is TEXT (zero-migration for
   future `Room::*`), but M1 writes NOTHING to the hash chain anyway. The only new schema is
   `governance_messaging_config` (admin panel). Bridge state lives bridge-side, not in the
   Brehon DB.

Anything larger reintroduces the coupling H1 exists to isolate.

---

## §4. ADRs That Govern This

Inherited from umbrella §"ADRs That Govern This"; the M1-load-bearing ones:

| ADR | M1 constraint |
|---|---|
| ADR-001 (fork Lemmy) | Bridge is a separate service, not a Lemmy-binary change. Must not break upstream-rebase discipline. `services/bridge/` excluded from workspace. |
| ADR-004 (plane separation) | M1 adds the chat/RTC plane. [06 §2.2](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) plane-boundary wording extended this phase. |
| ADR-008 (append-only signed log) | M1 writes **nothing** to the hash chain (`Room::*` entries are M2). No new entry kinds in M1. |
| ADR-011 (AGPLv3) | Tuwunel (Apache-2.0) compatible. Bridge written in-house stays AGPL-3.0. AGPL notice extended to mention the bridge + Tuwunel. |
| ADR-012 (Extism plugin host) | The 7 PM plugin hooks are M1's only integration seam into the Brehon binary. All verified present (umbrella §Research-Summary Q1). |
| ADR-014 (vanilla-Lemmy interop) | **OQ-V2-08 resolved (a): Brehon↔Brehon only.** M1 messaging is fork-only; no degraded-mode interop. |
| ADR-015 (pseudonymised actor IDs) | M1 has no jury/appeals rooms (those are M2), but the admin panel's identity-policy validator MUST already reject jury/appeals identity-policy overrides (the pin is structural, enforced from M1 even though the rooms come in M2). |
| ADR-016 (cross-app backplane) | **Backplane deferred to M2+.** M1 builds no B-fetch/B-publish/B-actor. The ADR-016 §4 architecture/threat/ops doc extensions for federated app planes are deferred with it; only the chat-plane (not app-plane) doc extensions land in M1. |

**Contradiction check**: None. M1 supersedes no ADR. The ADR-015 identity-policy pin is enforced
structurally from M1 (validator), satisfied vacuously until M2 adds the rooms.

---

## §5. Open Questions This Touches

| OQ | Status for M1 |
|---|---|
| **OQ-V2-08** (vanilla-Lemmy interop) | ✅ Resolved 2026-06-03 → (a) Brehon↔Brehon only. M1 honours it (no degraded-mode paths). |
| **OQ-V2-09** (disable-after-enable rollback) | ✅ **Resolved in this sub-PRD, §6.** Soft-pause; four sub-questions answered below. |
| **OQ-V2-10** (Matrix homeserver) | ✅ Resolved → Tuwunel. M1 deployment section bakes in the four "verify before committing" items + containerised topology caveat. |
| OQ-ADR016-01 (B-fetch SPI) | Deferred — **no longer blocks M1** (M1 builds no evidence adapter). First-blocks the M-phase that introduces evidence-fetch. |
| OQ-ADR016-03 (B-actor link flow) | Deferred — **no longer blocks M1** (M1 builds no link flow; users are bridge-local puppets). First-blocks the phase that introduces the link flow. |
| OQ-ADR016-02 / -04 (B-publish / sanction translation) | M2-targeted, unchanged. |
| OQ-005 (juror notification UX) | Soft gate. M1 introduces a Matrix notification vector but no jury rooms; OQ-005 only bites at M2. Non-blocking for M1. |
| OQ-018 (admin config write endpoint) | Soft gate. M1's `governance_messaging_config` lands through M1's own admin path; aligns with OQ-018's `messaging_*` prefix reservation when that ships. Direct-psql config acceptable until then. |

---

## §6. OQ-V2-09 RESOLVED — disable-after-enable soft-pause mechanics

The 2026-06-01 lean established the **structural** decision (soft pause, not hard
decommission). This sub-PRD owns and answers the four open sub-questions:

**Sub-question A — bridge process shutdown signal.**
Resolution: `messaging_enabled = false` (admin panel write) flips a flag the bridge polls (or
is notified of) and the bridge **gracefully drains then idles** — it stops initiating new
Matrix syncs and stops relaying new PMs, but does NOT tear down the Tuwunel homeserver, does
NOT delete rooms, and does NOT deregister the appservice. The bridge process stays running in a
paused state (cheap — Tuwunel idle is <500 MB; bridge idle is negligible). Re-enabling flips the
flag back and the bridge resumes syncing. No process restart required for either direction.

**Sub-question B — in-flight room handling.**
Resolution: rooms remain on Tuwunel and remain accessible to their Matrix members directly (the
homeserver keeps running). What pauses is **Brehon-side relay** — a Brehon user's PM sent while
paused is NOT bridged (it stays a normal Lemmy PM, or is rejected with a "messaging paused"
notice — plan-author picks the cleaner UX). Messages sent Matrix-side during the pause are
queued by Tuwunel's normal sync and delivered to the bridge on resume. No message loss on the
Matrix side; Brehon→Matrix relay is the only direction that pauses.

**Sub-question C — active-jury-room-on-disable policy.**
Resolution for M1: **vacuously satisfied** — M1 has no jury rooms (those are M2). The policy is
stated here for M2 to inherit: a soft pause with an active jury room MUST NOT silently strand
jurors mid-deliberation; M2's policy (to be specified in the M2 sub-PRD) is the live concern.
For M1, the admin panel surfaces a warning if `messaging_enabled` is toggled off while any
manually-provisioned community room has had activity in the last N minutes (advisory, not
blocking).

**Sub-question D — media / GDPR artefact cleanup runbook.**
Resolution: soft pause does **NOT** trigger media cleanup (the homeserver keeps the media repo;
pause is reversible, so destroying media would break reversibility). Media/GDPR cleanup belongs
to the **hard-decommission** story (a separate, explicit operator action — out of M1 scope per
umbrella deferral) and to per-room `hard_delete_after_days(n)` lifecycle policy (which runs
independently of the enable/disable flag). The M1 runbook documents: (1) soft pause = no
cleanup; (2) `hard_delete_after_days` continues to run during pause (lifecycle is orthogonal to
enable state); (3) GDPR erasure requests during a pause are honoured via the normal lifecycle/
redaction path, not via the pause mechanism.

**Net:** soft pause is a reversible relay-pause. The homeserver, rooms, media, and appservice
registration all persist. Only Brehon→Matrix relay halts. This is the lowest-risk, fully-
reversible posture and matches the 2026-06-01 lean exactly.

---

## §7. Phase Details (for `/prp-plan`)

M1 is one sub-phase delivery (it may split into 1–2 phase branches at plan time depending on
complexity score; the planner decides). Indicative work breakdown for the plan author:

**Greenfield bridge daemon (`services/bridge/`)**
- Crate skeleton, excluded from workspace, own `Cargo.toml`. Sibling-read the existing external-
  service patterns before authoring.
- `matrix-sdk-appservice` integration (verify licence + maturity + the issue-#219 `whoami`
  interaction at plan time — pin a known-good version).
- Puppet-on-first-contact; 1:1 DM relay; rich-media relay (image + voice note).
- Manual community-room provisioning command surface.
- Bridge-local `messaging_user_id` puppet map (NOT B-actor).
- Soft-pause flag poll/notify + graceful drain-to-idle.

**Brehon-side (in-workspace, additive only)**
- `governance_messaging_config` table + admin write path (identity policy, lifecycle policy,
  `messaging_enabled`). Identity-policy validator rejects jury/appeals overrides (ADR-015 pin,
  structural from M1).
- Wire the existing 7 PM plugin hooks to the bridge (notification/read-only; no new hooks, no PM-
  path changes).
- `messaging_enabled = false` clean-posture guard (default false).

**Deployment / ops**
- docker-compose: Tuwunel + bridge side-by-side; federation-disabled launch posture
  (`allow_federation = true; forbidden_remote_server_names = [".*"]` for LiveKit OpenID — though
  LiveKit is M3, the posture is set now); `ip_source` NOT set on loopback; containerised non-
  host-network handling per OQ-V2-10.
- Restart fixture for the admin-panel-persistence acceptance test.
- AGPL notice extension (bridge + Tuwunel).

**Design-doc updates (chat-plane only; app-plane deferred with backplane)**
- [06 §2.2](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) — add chat plane to plane-boundary wording.
- [06 §7](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) — threat rows: (a) bridge process compromise, (b) hostile Matrix homeserver payload, (c) E2EE key mishandling in bridge, (d) media artefact leak via pict-rs/Matrix-media path confusion. (LiveKit SFU row is M3.)
- [07 §5](../../docs/brehon-law-inspired-network/07-operations-and-federation.md) — `messaging_enabled=false` / soft-pause posture row.

---

## §8. Success Criteria (verifiable — M1 only)

From umbrella §Success-Criteria, M1 rows only:

| Criterion | How verified |
|---|---|
| 1:1 DM text+image+voice round-trip < 3s | Integration test `services/bridge/tests/dm_round_trip.rs` (cross-instance fixture) |
| Admin panel identity-policy change persists across restart | Integration test; restart fixture via docker-compose |
| `messaging_enabled = false` preserves clean v0 governance-only posture | `cargo test --test e2e` governance flow passes unchanged |
| Soft pause is reversible (enable→disable→enable) without process restart or data loss | Integration test: enable, send DM, disable, verify relay paused + homeserver up, re-enable, verify relay resumes |
| Identity-policy validator rejects jury/appeals override | Unit test on the admin write path (ADR-015 pin, even before M2 rooms exist) |
| `services/bridge/` excluded from workspace; `cargo build --workspace` pulls zero Matrix deps | CI check / `cargo tree` assertion |

---

## §9. Cross-Cutting Impact

- [x] **Hash-chain governance log touched?** **NO** — M1 writes nothing to the chain. (`Room::*`
  entries are M2.) This is a deliberate narrowing from the umbrella's "YES" (which covered all
  three clusters).
- [x] **`actor_pseudonym` touched?** **NO in M1** — no jury/appeals room membership rendering yet
  (M2). The admin validator references ADR-015 but reads no pseudonym data.
- [x] **`CaseStatus::EmergencyRemove` affected?** **NO in M1** — emergency-room provisioning is M2.
- [x] **AGPLv3 notice affected?** **Minor** — extend to mention the bridge + Tuwunel.
- [x] **Plane boundary extension** — YES (chat plane added to [06 §2.2](../../docs/brehon-law-inspired-network/06-security-and-threat-model.md)).
- [x] **Threat-model rows** — YES (4 chat-plane rows; SFU/app-plane rows deferred).
- [x] **New schema** — `governance_messaging_config` only (admin panel). No other migration.
- [x] **Federation-disabled posture row** — YES ([07 §5](../../docs/brehon-law-inspired-network/07-operations-and-federation.md)).
- [x] **Workspace integrity** — `services/bridge/` MUST be workspace-excluded; Brehon binary
  `Cargo.toml` gains zero Matrix deps.

---

## §10. Backwards Compatibility

- **Default-off.** `messaging_enabled = false` is the shipped default. An instance that ignores
  M1 entirely runs identically to its pre-M1 self.
- **No PM-path change.** The 7 plugin hooks are existing seams; M1 adds consumers, not new
  hooks. Vanilla Lemmy PM behaviour is unchanged.
- **No AP-wire change.** Per the §2 Person-field decision, no new actor-extension crosses the
  wire in M1. Federation with vanilla Lemmy is byte-unchanged.
- **Upstream-rebase safe.** `services/bridge/` is outside the workspace; an upstream Lemmy
  rebase touches nothing in it.

---

## §11. Implementation Phases (for `/prp-plan`)

| # | Phase | Status | Depends | PRP Plan |
|---|---|---|---|---|
| 1 | M1 — chat infrastructure | pending | OQ-V2-08 ✅, OQ-V2-09 ✅, OQ-V2-10 ✅ (all resolved) | — (to be generated) |

The plan author runs `/prp-plan` against this sub-PRD. Complexity score will likely be high
(greenfield daemon + new external service + docker-compose); expect a split-or-proceed DQ. The
plan §13 task list should keep the greenfield bridge work and the in-workspace Brehon-side work
as distinct task groups (different file-ownership trees → cleaner cohorts).

---

## §12. Decisions Log (M1-specific, additive to umbrella §Decisions-Log)

| Decision | Choice | Rationale |
|---|---|---|
| M1 scope | Chat infrastructure ONLY; ADR-016 backplane deferred to M2+ | User 2026-06-03. Matches H1 (prove bridge in isolation) + strict-sequential Q10. |
| OQ-V2-08 (vanilla interop) | (a) Brehon↔Brehon only | User 2026-06-03. ADR-014 fork-only consistency; avoids degraded-mode paths. |
| OQ-V2-09 (rollback) | Soft pause = reversible relay-pause; homeserver/rooms/media/registration persist | This sub-PRD §6. Lowest-risk, fully reversible; matches 2026-06-01 lean. |
| Bridge code location | `services/bridge/` in-repo, workspace-EXCLUDED, own Cargo.toml | Single audit trail + single CI, while Brehon binary gains zero Matrix deps. Separate repo rejected (splits trail, doubles bootstrap, no second consumer yet). |
| Person AP field | Reuse legacy `matrix_user_id` unchanged; no new AP field in M1 | Puppet map is bridge-local; actor-extension-on-wire is a B-actor (ADR-016) concern → M2+. Keeps M1 AP-wire byte-unchanged. |
| Homeserver | Tuwunel (Synapse fallback) | OQ-V2-10 resolved 2026-06-01. Rust-native, MSC4143, fits solo-operator constraints. |

---

## §13. Research Summary (M1-relevant subset)

Condensed from umbrella §Research-Summary + `docs/research/matrix-homeserver-selection-2026.md`.

**PM plugin hooks (all 7 verified, umbrella Q1):** dispatcher at
`crates/api/api_utils/src/plugins.rs:40-51, 127-149`; 7 call sites across `private_message/
create.rs`, `update.rs`, `apub/objects/private_message.rs`, `reports/private_message_report/
create.rs`, `notify.rs`. Stability codified at `.claude/rules/pm-plugin-hooks-stable.md`.

**Greenfield confirmed (umbrella Q5):** zero Matrix/LiveKit/WebRTC/WebSocket/SSE/Synapse/
mautrix/Conduit deps in the workspace. Only surface refs: legacy `matrix_user_id` plumbing,
`is_valid_matrix_id` validator, localization strings.

**Tuwunel deployment (matrix research §Recommendation):** four "verify before committing" items —
(1) issue #465 `ip_source`/loopback (fixed v1.7.1; don't set `ip_source` on loopback bridges);
(2) issue #219 `whoami` response code (verify against `matrix-sdk-appservice` version; test
puppet registration end-to-end); (3) federation-disabled launch needs `allow_federation = true;
forbidden_remote_server_names = [".*"]`; (4) NEVER switch between Conduit-lineage forks (DB
corruption unrecoverable). RAM: Tuwunel idle <500 MB; LiveKit (M3) is the dominant consumer.
Containerised non-host-network topology requires explicit handling (bridge traffic is non-
loopback).

**No state-transition hooks fire today (umbrella Q3)** — irrelevant to M1 (M1 has no governance
triggers) but load-bearing for M2's log-tailing design.

---

## §14. Cross-References

- Umbrella: `.claude/PRPs/prds/v2-messaging-rtc.prd.md`
- Predecessor research: `docs/brehon-law-inspired-network/V2/messaging.md` §1–§11
- Homeserver research: `docs/research/matrix-homeserver-selection-2026.md`
- ADR-016: `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` §ADR-016
- OQ resolutions: `99-decisions-and-open-questions.md` 2026-06-03 entry (OQ-V2-08, M1 scope) +
  2026-06-01 entry (OQ-V2-09 lean, OQ-V2-10)
- Plugin-hook stability rule: `.claude/rules/pm-plugin-hooks-stable.md`
- External-services pattern: `docs/brehon-law-inspired-network/07-operations-and-federation.md` §1.2

---

*Generated: 2026-06-03 — M1 schedule time. Ready for `/prp-plan`.*
*Scope: chat infrastructure only. ADR-016 backplane deferred to M2+.*
