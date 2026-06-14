---
role: bm-task
verb: bm-pr
phase: m2-late-b-actor
base_branch: governance-v0
created: 2026-06-14
---

# bm-task brief — m2-late-b-actor bm-pr

**Role:** `[role:bm-task]`
**Verb:** `bm-pr`
**Phase:** `m2-late-b-actor`
**Base branch:** `governance-v0`

## Dispatch line

```
[role:bm-task] m2-late-b-actor bm-pr — see .claude/PRPs/briefs/m2-late-b-actor-bm-pr-1.md
```

## Scope

Open a PR from `phase-m2-late-b-actor` into `governance-v0` on `barrie-cork/lemmy`.

- Title: `feat(b-actor): portable actor-ID linkage — dual-signed link-claim + actor_app_link table (ADR-016 C3)`
- NOT a draft (CodeRabbit must review)
- `--repo barrie-cork/lemmy` mandatory

## PR body

```
## Summary

- Adds `actor_app_link` Postgres table + Diesel model: maps a Brehon `actor_pseudonym` UUID to an external app identity (`app_id` + `app_local_id`), with `revoked_at` for prospective revocation (ADR-016 component 3, B-actor portable actor-ID)
- Three new endpoints under `GET|POST /api/v4/governance/link`: `link_actor` (JWT-authed, mints dual-signed claim + POSTs to bridge), `link_confirm` (bearer-authed, verifies both ed25519 sigs before INSERT), `revoke_link` (JWT-authed, prospective unlink)
- Bridge SQLite inverse cache `app_actor_link` + `handle_link_claim` handler: verifies Brehon sig, countersigns `nonce\napp_local_id`, upserts cache, POSTs confirm back to Brehon
- 5 e2e integration tests (`actor_app_link_fixtures`): dual-sig flow, bad-sig rejection, bad-bearer rejection, prospective revoke, pseudonym-not-raw-identity (ADR-015)
- ADR-015 enforced throughout: `actor_pseudonym` UUID in all claim bytes and governance log payloads — never `person_id`, username, or email
- ADR-008 enforced: `link_confirm` and `revoke_link` call `governance_log::append` inside `run_transaction` before returning success

## Plan reference

`.claude/PRPs/plans/m2-late-b-actor.plan.md` — Task 9 (migration), Task 10 (routes), Task 11 (bridge cache), Task 12 (bridge handler), Task 13 (e2e tests)

## Validation

- Windows cargo check: EXIT 0 @ `a9315f37d`
- Linux compile (Docker rust:1.95): EXIT 0
- e2e tests: 5/5 PASS (`cargo test --test e2e actor_app_link`)
- governance-v0 merged forward @ `1542714a2`

## Commit log (phase diff)

See `git log governance-v0..phase-m2-late-b-actor --oneline`
```

## Required reading

- `.claude/rules/branch-manager.md`
- `.claude/rules/gh-pr-fork-target.md`
- `.claude/commands/bm/bm-pr.md`

## Constraints

- Base MUST be `governance-v0` (never `main`)
- `--repo barrie-cork/lemmy` on all `gh pr` commands
- NOT a draft
- Do NOT touch `crates/**`, `migrations/**`, `tests/**`
