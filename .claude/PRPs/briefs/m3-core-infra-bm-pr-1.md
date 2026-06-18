---
role: bm-task
verb: bm-pr
phase: m3-core-infra
base_branch: governance-v0
created: 2026-06-18
---

# bm-task brief — m3-core-infra bm-pr

**Role:** `[role:bm-task]`
**Verb:** `bm-pr`
**Phase:** `m3-core-infra`
**Base branch:** `governance-v0`

## Dispatch line

```
[role:bm-task] m3-core-infra bm-pr — see .claude/PRPs/briefs/m3-core-infra-bm-pr-1.md
```

## Scope

Open a PR from `phase-m3-core-infra` into `governance-v0` on `barrie-cork/lemmy`.

- Title: `feat(rtc,bridge): M3 RTC stack deployable+optional — rtc_enabled flag, LiveKit pseudonym-JWT, bridge_room cols, actor-pseudonym endpoint, profile-gated sidecars`
- NOT a draft (CodeRabbit must review)
- `--repo barrie-cork/lemmy` mandatory

## PR body

```
## Summary

Makes the M3 town-hall real-time-communication stack **deployable and fully optional**, and gives the bridge the ability to mint LiveKit access tokens whose participant identity is a **pseudonym** (ADR-015). Phase 1 stands up the stack + mints a JWT; it does NOT run a town hall (stage mode / chair / queue / mute / recording are Phases 3–5).

- **`rtc_enabled` config seed** (default `false`): new migration seeds the row into the existing `governance_messaging_config` typed-KV table; `BridgeStatus.rtc_enabled` read consumer surfaces it (Task 1). Zero schema change — data seed only.
- **`bridge_room` RTC-state columns** (`chair_id`, `queue_state`, `recording_config`) added to the bridge-local SQLite table via embedded-schema + idempotent ALTER guards (Task 2).
- **LiveKit HS256 JWT mint** (`services/bridge/src/livekit_jwt.rs::mint_access_token`): `sub` = pseudonym by contract (ADR-015) — no overload takes person_id; optional LiveKit config fields (Task 3).
- **Brehon-side `actor-pseudonym` endpoint** (`GET /bridge/actor-pseudonym`): calls the canonical `actor_pseudonym_helper::get_or_create` allocator — the load-bearing ADR-015 callsite for M3. Response `BridgeActorPseudonym { pseudonym: String }` ONLY (no person_id / username / email) (Task 4).
- **RTC Docker sidecars** (LiveKit, lk-jwt-service, Element Call) appended to `services/bridge/docker-compose.yml` under `profiles: ["rtc"]` — `docker compose up` (no profile) starts NONE (clean governance-only posture); `docker/docker-compose.yml` stays Lemmy-only (Task 5).
- **AGPL-NOTICE rows** for all three sidecars (ADR-011): LiveKit/lk-jwt Apache-2.0, Element Call AGPL-3.0 with §13 source-disclosure (Task 5).
- **`rtc_enabled=false` clean-posture e2e** proves a governance transition runs unchanged with the RTC stack absent + zero side-effects (Task 6).

## Validation

- **Windows cargo check** (`--workspace --features full`): EXIT 0 (Task 4, 18m06s).
- **Bridge Linux-compile gate** (`cargo-linux.sh check --manifest-path services/bridge/Cargo.toml`, Docker rust:1.95): EXIT 0 @ `9202664da` (Finished 5m17s; 4 benign dead-code warnings on `mint_access_token`/`Claims` — Phase-1 mints the capability, callers land Phase 3+).
- **e2e** (scoped, Windows nextest):
  - `m3_actor_pseudonym_endpoint_idempotent_opaque` PASS 1/1 (Task 4, after fix-impl-4a Crud import fix)
  - `m3_rtc_disabled_clean_posture_governance_unaffected` PASS 1/1 (Task 6)
- **Deploy-smoke** (Story 4, `docker compose --profile rtc up`): all 3 sidecars boot healthy — livekit Up (`--dev`), lk-jwt :8085/healthz HTTP 200, element-call :8086 HTTP 200; no-profile = tuwunel only (clean posture). Two placeholder bugs caught + fixed via fix-impl-5a (element-call tag `0.6.0`→`v0.6.0`; livekit `--config`→`--dev`).

## Plan reference

`.claude/PRPs/plans/m3-core-infra.plan.md` — Tasks 1–6.

## Commit log (phase diff)

See `git log governance-v0..phase-m3-core-infra --oneline`
```

## Required reading

- `.claude/rules/branch-manager.md`
- `.claude/rules/gh-pr-fork-target.md`
- `.claude/commands/bm/bm-pr.md`

## Constraints

- Base MUST be `governance-v0` (never `main`)
- `--repo barrie-cork/lemmy` on all `gh pr` commands
- NOT a draft (CodeRabbit skips drafts)
- Do NOT touch `crates/**`, `migrations/**`, `tests/**`, `services/bridge/**` — PR-open only, no code edits
- Body assembles from this brief + `git log governance-v0..phase-m3-core-infra --oneline`
