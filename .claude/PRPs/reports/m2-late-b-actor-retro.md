# Retro: m2-late-b-actor — B-actor Portable Actor-ID Linkage (ADR-016 C3)

**Date:** 2026-06-14
**Phase branch:** `phase-m2-late-b-actor`
**PR:** #197 (`feat(b-actor): portable actor-ID linkage — dual-signed link-claim + actor_app_link table`)
**Tasks:** 13 impl-tasks + fix-impl-1 (bridge CR fixes) + bm-cut + bm-pr + bm-poll-cr + bm-merge

---

## §1 What shipped

**B-actor** (ADR-016 Component 3): a portable Brehon actor ID that an external app links to its native user identity via a dual-signed link-claim (Brehon ed25519 + app countersignature).

**Delivered artifacts:**

| # | Task | Files | Outcome |
|---|---|---|---|
| 1 | Migration: `actor_app_link` table | `migrations/2025-05-18-000001_actor_app_link/` | ✅ |
| 2 | schema.rs regen | `crates/db_schema/src/schema.rs` | ✅ |
| 3 | Model + InsertForm | `crates/db_schema/src/source/governance/actor_app_link.rs` | ✅ |
| 4 | `ActorAppLinkId` newtype | `crates/db_schema/src/newtypes/mod.rs` | ✅ |
| 5 | `pub mod actor_app_link` export | `crates/db_schema/src/source/governance/mod.rs` | ✅ |
| 6 | `ENTRY_KIND_*` consts + `sign_link_claim` | `crates/db_schema/src/source/governance/governance_log.rs` | ✅ |
| 7 | Shim re-export for entry kinds | `crates/api/api/src/governance/mod.rs` | ✅ |
| 9 | DTOs + `link_actor`/`link_confirm`/`revoke_link` handlers | `crates/api/api/src/governance/actor_app_link.rs` + `crates/api/api_common/src/governance.rs` | ✅ |
| 10 | Route wiring `/link`, `/link/confirm`, `/link/revoke` | `crates/api/routes/src/governance.rs` | ✅ |
| 11 | Bridge SQLite cache `app_actor_link` | `services/bridge/src/app_actor_link.rs` | ✅ |
| 12 | Bridge `handle_link_claim` handler + route wiring | `services/bridge/src/link_handler.rs` + `services/bridge/src/appservice.rs` | ✅ |
| 13 | E2E: 5 integration tests (dual-sig, bad-sig, bad-bearer, pseudonym check, revoke) | `crates/server/tests/e2e/actor_app_link.rs` | ✅ |
| fix-1 | Bridge CR fixes: composite PK, log redaction, non-2xx confirm | `services/bridge/src/app_actor_link.rs` + `services/bridge/src/link_handler.rs` | ✅ |

**No task 8** — plan skipped it (tasks jump 7→9 by design; 8 was merged into 9 scope).

---

## §2 What went well

**Role discipline held.** All 13 impl-tasks ran on the daemon as `[role:impl-task]`; advisor stayed meta-only. The BM verb chain (`bm-cut → bm-pr → bm-poll-cr → bm-merge`) ran as planned with no BM/impl boundary violations.

**Validate-pending-laptop gate worked cleanly.** 13+ DQ entries, every one resolved without §G4 classifier escalation. The bridge Linux compile gate (`cargo-linux.sh check --manifest-path services/bridge/Cargo.toml`) caught that the bridge is workspace-excluded and validated correctly via Docker Linux.

**CR caught a real composite-key bug (cr-6).** The original `app_actor_link` SQLite schema had `app_local_id TEXT PRIMARY KEY` — two different apps sharing the same local user ID would silently collide. CR's fix was correct and was fixed clean in fix-impl-1. This is exactly what the CR gate is for.

**ADR-015 pseudonym discipline held.** All handler payloads use `actor_pseudonym` UUID (`brehon_actor_id`), never `person_id`/username/email. The ADR-015 load-bearing clause in the brief § 4 was enforced; `grep brehon_actor_id` in e2e confirmed the pseudonym (not person_id) flows through all 5 tests.

**Bridge crate isolation worked.** The `services/bridge` workspace exclusion handled without incident — impl-tasks correctly wrote `validate-pending-laptop-linux` DQ entries and stopped; advisor ran `cargo-linux.sh` from a throwaway worktree.

---

## §3 What was rough

### 3.1 DQ merge conflicts (×4 across phase)

**Root cause:** Laptop pushed briefs/answers to `origin/governance-v0` without syncing the daemon; daemon workers wrote DQ entries on stale base; finalize-merge diverged from origin. Happened on bm-poll-cr brief (most costly), two validate-pending-laptop answers, and DQ cleanup after fix-impl-1.

**Fix applied:** Added `feedback_daemon_sync_before_dispatch.md` lesson (committed `21df8cc01`). The daemon sync step is now part of `advisor-orchestrator.md` §4 "Daemon trunk sync" hard gate.

**Severity:** Medium — each instance cost ~10 min of Python union-script resolution. No data was lost.

### 3.2 Throwaway worktree cleanup

The detached-HEAD `brehon-fork-validate-fix1` worktree was used for the fix-impl-1 bridge validation. The DQ mutation written there had to be manually copied to the phase-branch worktree (`brehon-fork-validate-13`) before committing. This is awkward but correct — the throwaway is detached and can't push to the phase branch directly.

**No lesson filed** — this is inherent to the throwaway-worktree pattern. The copy step is the correct procedure.

### 3.3 PMD HTTP MCP session drop (session start)

At session resume (post-compaction), `memory_write_eval` failed ×3 with "missing session id and not initialize". Root cause: MCP HTTP session handshake not completing after context restart. Fix: user ran `/mcp` to re-initialize. This is a known PMD HTTP topology edge case, not a Brehon process issue.

**Mitigation already tracked:** `issue_note_pmd_http_service_needs_execstop_wal_checkpoint.md` — RESOLVED 2026-06-13.

---

## §4 Per-task complexity scores

| Task | Files | Commits | Runtime-min (est) | Notes |
|---|---|---|---|---|
| 1 (migration) | 2 | 1 | ~5 | Clean |
| 2 (schema regen) | 1 | 1 | ~5 | Clean |
| 3 (model) | 1 | 1 | ~5 | Clean |
| 4 (newtype) | 1 | 1 | ~5 | Clean |
| 5 (mod export) | 1 | 1 | ~5 | Clean |
| 6 (entry-kind consts + sign_link_claim) | 1 | 1 | ~10 | Clean |
| 7 (shim re-export) | 1 | 1 | ~5 | Clean |
| 9 (DTOs + handlers) | 2 | 1 | ~20 | Heaviest non-e2e task |
| 10 (routes) | 1 | 1 | ~10 | Clean |
| 11 (bridge SQLite cache) | 1 | 1 | ~15 | Clean |
| 12 (bridge handler) | 2 + clippy follow-up | 2 | ~25 | Clippy -D warnings needed follow-up commit |
| 13 (e2e tests) | 2 | 2 | ~30 | Import fixes needed (E0599 + unused import) |
| fix-impl-1 (bridge CR) | 2 | 1 | ~20 | Linux validate gate correct |

Aggregate: 13 tasks / ~165 min impl runtime (est) / no §G4 classifier fires.

---

## §5 ADR compliance

| ADR | Gate | Result |
|---|---|---|
| ADR-016 (B-actor backplane) | B-actor seam delivered; dual-signed claim; composite `UNIQUE` on `actor_app_link` | ✅ |
| ADR-015 (pseudonymity) | All payloads carry `brehon_actor_id` (UUID from `actor_pseudonym`); no person_id/email in any log or payload | ✅ |
| ADR-008 (append-only log) | `link_actor`: `actor_app_link_created` appended via `governance_log::append()` before returning; `revoke_link`: `actor_app_link_revoked` appended; hash chain not rewritten | ✅ |
| ADR-012 (fire-and-forget bridge) | `link_actor` fires-and-forgets the bridge POST (log-and-swallow on error); CR cr-3 rebutted on this basis | ✅ |
| ADR-011 (AGPLv3) | No new deps added outside workspace; bridge uses existing `axum`/`reqwest`/`rusqlite`/`ed25519-dalek`; source disclosure unchanged | ✅ |
| ADR-013 (EmergencyRemove) | Not touched by this phase | N/A |

---

## §6 CR findings summary

| Finding | Severity | Bucket | Outcome |
|---|---|---|---|
| cr-1 (DQ commands mismatch) | Major | rebut | DQ is meta-work; commands field advisory |
| cr-2 (workflow_run_id: 0) | Low | rebut | 0 is documented Shape-G-disabled sentinel |
| cr-3 (/link fire-and-forget) | Major | rebut | ADR-012 by design; doc comment says log-and-swallow |
| cr-4 (sig strengthening) | Critical | carry-forward | Valid v1 hardening; nonce+app_local_id covered by app countersig |
| cr-5 (governance_log non-write assertions) | Nit | carry-forward | Nice-to-have; not blocking v0 |
| cr-6 (composite cache key) | Major | fix-in-pr ✅ | Fixed `dcb5d3723`; Linux-validated |
| cr-7 (bridge config fail-fast) | Major | carry-forward | Bridge is v0-stub; deferred to v1 |
| cr-8 (raw actor ID in logs) | Major | fix-in-pr ✅ | Fixed `dcb5d3723`; redacted to first 8 chars |
| cr-9 (non-2xx confirm) | Major | fix-in-pr ✅ | Fixed `dcb5d3723`; BAD_GATEWAY on non-2xx |
| cr-10 (meta summary) | Nit | done | Not actionable |

---

## §7 Carry-forward obligations

| ID | Item | Priority | Target |
|---|---|---|---|
| cf-1 | **cr-4**: Strengthen sig binding — cover all claim fields + verify expiry and Brehon sig before DB writes | High | v1 hardening sub-phase |
| cf-2 | **cr-5**: Add governance_log non-write assertions to rejection tests in `actor_app_link.rs` | Low | Next e2e-expanding sub-phase |
| cf-3 | **cr-7**: Bridge config fail-fast at boot (validate key hex length + URL format) | Medium | v1 bridge hardening |
| cf-4 | **MiniMax key rotation** | Carried | User rotates self |
| cf-5 | **role-customization T4a** `/check-role-health` | Carried | Next available session |

---

## §8 Lessons filed this phase

- `feedback_daemon_sync_before_dispatch.md` — daemon-local trunk stale causes cascading DQ merge conflicts; sync before every Junior dispatch (committed `21df8cc01`)

---

## §9 State after merge

- `governance-v0` HEAD: `b72fa1c2e` (bm-merge brief)
- Phase branch `phase-m2-late-b-actor`: merged into `governance-v0` (PR #197)
- Pilot server: continues to run at `http://100.81.145.58:1236`
- B-actor seam: available on `governance-v0`; pilot testing can proceed with the actor-linking flow
- ADR-016 C3: DONE. Remaining ADR-016 components: C4 (PeerTube), C5 (cross-app reputation rollup) — not yet scheduled
