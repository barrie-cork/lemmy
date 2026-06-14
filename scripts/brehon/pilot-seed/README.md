# Brehon pilot-seed — standardized governance test-scenario seeding

Standardized, idempotent seeding so live governance test scenarios start from a
known, **correctly-scoped** state — instead of ad-hoc curl/SQL, which is
error-prone in a system this customizable (two traps bit phase-3 testing: the
`reputation_snapshot.community_id` scope mismatch, and re-triggering a handler
that doesn't fire the bridge hook).

**Canonical knowledge:** `.claude/lessons/feedback_pilot_governance_workflow_seeding_order.md`
(PMD pattern #990). Read it before extending these scripts.

## Where to run

These talk to the live pilot (`localhost:8536` + the `docker-postgres-1` /
`brehon-bridge` containers), so they run **on the homeserver**:

```bash
ssh homeserver "bash /srv/brehon-fork/scripts/brehon/pilot-seed/seed-appeal-ready.sh --appeal"
```

All knobs are env-overridable (see `lib.sh` header): `API`, `PG_CONTAINER`,
`COMMUNITY_ID`, `ADMIN_USER/PASS`, `TEST_PASS`.

## The contract every scenario script honors

- **Idempotent where possible** — re-running reuses existing accounts, re-flags
  eligibility, drives a *fresh* case (spent cases can't re-fire a past transition).
- **Machine-readable output** — `CASE_ID=..`, `POST_ID=..`, `STATUS=..`,
  `APPEAL_ROOM_ID=..`, `PANEL=..`, `RESULT=..`. Grep these, don't parse prose.
- **Fails loud** on a failed precondition (no token, case didn't open, panel
  empty, case not Decided). Never silently proceeds. `RESULT=PASS` is emitted
  ONLY when the scenario actually completed — never optimistically.
- **Config-aware — never hardcodes quorum/panel.** Governance config varies per
  community × severity_tier × status_tier (127 keys); a Severe case is panel 7 /
  quorum 5 / threshold 6, not the default 5/3/3. The win condition is
  `threshold_count`, NOT quorum. Scripts read the frozen `panel_size_snapshot` /
  `quorum_snapshot` / `threshold_count_snapshot` columns (set at assign-jury) via
  `lib.sh` helpers `read_panel_size` / `read_quorum` / `read_threshold_count`, and
  drive voting with `vote_to_threshold <case> <decision> [role]` over the ACTUAL
  `seated_panel`. Status assertions use `assert_status` (loud, not a bare WARN).
  Do NOT reintroduce a fixed `-lt 3` vote loop or a hardcoded `juror1..N` list.
- **Verifies three surfaces** where it asserts success: DB (`moderation_case` /
  `jury_assignment` / `governance_log` chain), bridge (`bridge_room`), Matrix
  (room exists). `lib.sh` has `bridge_rooms` + `verify_hash_chain` helpers.

## Scenarios

| Script | State produced | Status |
|---|---|---|
| `seed-jurors.sh [N] [prefix]` | N community-scoped, strictly jury-eligible accounts | ✅ VERIFIED (the eligible-juror INSERT used on juror6–10) |
| `seed-appeal-ready.sh [--appeal]` | fresh case → `Decided`; `--appeal` → `Appealed` + appeal room provisioned | ✅ VERIFIED as script (case 8, fix `52c0eb382`; case 6 first live verify) |
| `seed-deadlock.sh` | panel votes, no decision meets quorum → `AdminReview`, `jury_deadlock` | 🚧 STUB — flow not yet run live |
| `seed-emergency.sh` | post → `admin_emergency_remove` → `EmergencyRemove` + emergency room | 🚧 STUB — `admin_emergency_remove` route + payload TBD; <2s latency target |
| `seed-sanction-kind.sh KIND` | a provisioned-room case → sanction of `KIND` → power-level applied | 🚧 STUB — only `hide_content` exercised so far |

**STUB discipline:** a stub script documents the intended flow + the known
unknowns (exact route, payload shape, config knobs) and exits non-zero with a
`NOT_IMPLEMENTED` marker. Promote a stub to ✅ only after the flow is run live
and verified across the three surfaces — same bar as `seed-appeal-ready.sh`.
Do NOT fabricate an untested flow as if proven (the `#[ignore]`'d-test class of
bug — "looks wired, never ran" — is exactly what this harness exists to avoid).

## `lib.sh` building blocks (compose new scenarios from these)

- `login <user> <pass>` / `admin_jwt` — JWT (bare).
- `api_post <path> <json> [jwt]` — POST, echo body. `json_field <key>` reads a top-level field; `json_nested <k1> <k2> <k3>` reads 3 levels deep (e.g. `.post_view.post.id` on POST /post).
- `person_id <username>` — person.id.
- `register_and_approve <username>` — register (require_application) + admin-approve; idempotent; echoes person_id.
- `make_jury_eligible <person_id>` — the **community-scoped** `reputation_snapshot` INSERT (the scope trap, encoded once).
- `seed_eligible_jurors <prefix> <count>` — ensure a fresh strict-eligible pool; echoes person_ids.
- `case_status <case_id>` / `bridge_rooms` / `verify_hash_chain` — verification surfaces.

## Adding a scenario

1. Find the route that fires the transition you need: `grep -rn
   'governance_case_after_transition' crates/` → read the handler. (Not every
   handler that mutates the case fires the bridge hook — lesson §1.)
2. Compose the flow from `lib.sh` helpers; drive a **fresh** case.
3. Ensure the round's panel is seeded + eligible so the hook carries a non-empty
   pseudonym list (lesson §2/§4).
4. Assert across the three surfaces. Print machine-readable result lines.
5. Run it live, verify, then flip the table row to ✅.
