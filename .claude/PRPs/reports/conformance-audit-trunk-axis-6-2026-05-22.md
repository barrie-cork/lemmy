---
axis: 6
scope: trunk (governance-v0 @ 7e6c4202f)
date: 2026-05-22
mode: file (all governance files in three roots)
---

# Conformance audit — axis 6: ADR-015 pseudonym handling for remote actors

## Scope

Three federation/governance roots on `governance-v0` @ `7e6c4202f`:

- `crates/apub/activities/src/governance/` (5 files) — PRIMARY (remote-actor handlers)
- `crates/api/api/src/governance/` (31 files) — secondary
- `crates/db_schema/src/source/governance/` (26 files) — DB-write structs

Total: 62 governance .rs files.

## Detection

Per `axes/6-adr-015.md`:

1. Grepped `actor_pseudonym` across all three roots — full output captured.
2. Identified all `governance_log::append` callsites in remote-actor handlers
   (`inbox.rs` receive functions, `wrap_governance_inbound`, `log_inbox_drop`,
   `evict_oldest_unreviewed_if_needed`).
3. For each callsite, confirmed the value passed as the 4th parameter
   (`actor_pseudonym: Option<String>`).
4. Verified caller context for each `actor_pseudonym_helper::get_or_create`
   callsite across `api/governance/` to confirm only local `PersonId` values
   are passed.

## Inventory of `governance_log::append` callsites in remote-actor handlers

### `crates/apub/activities/src/governance/inbox.rs`

| Line | Enclosing fn | Context | 4th arg (`actor_pseudonym`) |
|---|---|---|---|
| 204-210 | `receive_remote_sanction_notice` | Remote actor inbound | `None` |
| 219-229 | `receive_remote_sanction_notice` | persist_failed error path | `None` |
| 310-316 | `receive_remote_trust_attestation` | Remote actor inbound | `None` |
| 324-334 | `receive_remote_trust_attestation` | persist_failed error path | `None` |
| 628-633 | `log_inbox_drop` | Drop-log helper (blocklisted/oversize/rate/replay) | `None` |
| 699-704 | `evict_oldest_unreviewed_if_needed` | Storage-cap eviction | `None` |
| 800-805 | `receive_remote_moderation_label` | Remote actor inbound (fed-in-b Task 4) | `None` |
| 814-824 | `receive_remote_moderation_label` | persist_failed error path | `None` |

**All 8 callsites pass `None` for `actor_pseudonym`.** Each is in an inbound
federation handler where the actor is a remote Lemmy instance; the comment at
line 125–127 explicitly cites ADR-015: "the actor is remote and has no local
pseudonym (per ADR-015 the pseudonym table only covers local Persons)."

### `crates/apub/activities/src/governance/publish_trust_attestation.rs`

No `governance_log::append` callsites. The `check_per_actor_rate_limit` impl
delegates to `log_inbox_drop` (which passes `None` — same helper as above).
No `actor_pseudonym` callsites at all in this file's receive/verify paths.

### `crates/apub/activities/src/governance/publish_sanction_notice.rs`

The only `actor_pseudonym` mention is a doc-comment (line 355) referencing
the outbound `send_local_sanction_notice` wrapper signature — not an active
callsite and not in a remote-actor context.

### `crates/api/api/src/governance/` — remote-actor context scan

No file in `api/governance/` handles inbound remote-actor data directly. The
federation receive path is exclusively in `apub/activities/`. The
`actor_pseudonym_helper::get_or_create` callsites in `api/governance/` all
operate on local `PersonId` values:

- `admin_emergency_remove.rs:81,279` — local admin / local person
- `submit_jury_vote.rs:133,483,487` — local juror / local target / local sponsor
- `sponsor_liability_grace.rs:234` — local revoker from `surety.sponsor_id`
- `reputation_snapshot.rs:294` — local person from batch-job or synchronous handler
- `admin_close_case.rs:36` — local admin
- `admin_assign_jury.rs:104,274,1195` — local admin / local persons
- `admin_trigger_appeal_rejury.rs:42` — local admin
- `decline_jury_assignment.rs:51,190` — local caller / local replacement
- `admin_config.rs:510,832` — local admin
- `admin_rule_sets.rs:127,384` — local admin
- `accept_jury_assignment.rs:60` — local caller
- `federation_outbox.rs:166` — local admin (outbound send, via `get` not
  `get_or_create`; confirmed by comment at line 155–165 citing ADR-015)

### `crates/db_schema/src/source/governance/governance_log.rs`

`GovernanceLogInsertForm` defines `actor_pseudonym: Option<String>` (line 102).
The `append` function's signature (line 256) takes `actor_pseudonym: Option<String>`
and passes it through. No callsites here — this is the write helper itself.
The doc-comment at lines 244–247 explicitly states: "`None` for system-generated
events (e.g. a scheduled job, or a remote actor whose pseudonym does not exist locally)."

### Inverse check: LOCAL actors passing `None`

Reviewed all `governance_log::append` callsites that pass `None` in
`api/governance/` to verify no local-actor action is incorrectly omitting its
pseudonym:

- `admin_assign_jury.rs:993` — `write_constraint_relaxation`: explicitly
  documented as system-level (R1/R2/R3 relaxations, not attributed to any admin).
  Comment at line 950–953 cites this distinction. Correct.
- All other `None` callsites in `inbox.rs` are remote-actor or infrastructure
  paths (drop logs, eviction logs, error-path persist-failed), all documented.

No local-actor handler was found passing `None` where a pseudonym should be
present.

## Findings

**Tier 1:** 0
**Tier 2:** 0
**Tier 3:** 0

No divergences. Every `governance_log::append` callsite in a remote-actor
handler passes `None` for `actor_pseudonym`. Every callsite in a local-actor
handler passes `Some(actor_pseudonym_helper::get_or_create(...))` or the
`get` variant (in `federation_outbox.rs`). The ADR-015 contract is uniformly
enforced across all 62 governance .rs files in the three roots.

The pattern is self-documenting: the comment at `inbox.rs:125–127` serves as
an explicit canonical sibling reference for the `None` contract, and the
`governance_log.rs:244–247` doc-comment codifies it at the schema level.

## Hypothesis discipline

All flags would have been hypotheses until §15 cargo check confirmed. With
zero flags raised, no compiler oracle is needed for axis-6.

## Evidence string

(None — no findings.)
