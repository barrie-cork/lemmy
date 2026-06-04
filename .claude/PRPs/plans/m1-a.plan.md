# Plan: m1-a — M1 Tree A (greenfield `services/bridge/` Matrix application-service bridge)

> **This is a pointer plan.** m1-a reuses the already-approved **`m1.plan.md` §13 Tasks 8–13** (Tree A) as its authoritative task list. There is no separate Tree-A task body here — duplicating §13 would risk drift. This file exists so `/auto-phase m1-a` resolves a single `m1-a*.plan.md` glob, and so the m1-a-scoped DoD + stories + guardrails are readable in one place.
>
> **Plan-path decision (2026-06-04):** user chose **reuse `m1.plan.md`** (not re-shape) at the m1-a bootstrap fork — "Reuse m1.plan.md (Recommended)" per the m1-a-bootstrap.md §"Next concrete action" path (a). The Tree-A subset was approved at M1 gate-1 (Tree B + Tree A planned together; Tree B shipped as m1-b). Authored_by: advisor (canonical brehon-fork / governance-v0 session).

## 1. Summary

m1-a ships **M1 Tree A**: the greenfield, **workspace-EXCLUDED** Rust application-service bridge daemon at `services/bridge/`. It depends on `matrix-sdk` 0.18 + `ruma-appservice-api` 0.16 + `axum`, owns the AS transaction server, puppet-on-first-contact, the 1:1 DM relay (text/image/voice), manual room provisioning, and soft-pause drain. It consumes Tree B's already-shipped `bridge_notify` HTTP contract + `governance_messaging_config` schema (m1-b, merged @ `d6d027794`) and never the reverse. The bridge is its own crate tree — `cargo build --workspace` pulls **zero** Matrix dependencies (story 6).

**Headline acceptance for m1-a:** the bridge compiles inside `services/bridge` (`cd services/bridge && cargo check`); `cargo build --workspace` pulls zero Matrix deps; and §16a stories **1, 4, 6** verify here (story 1 DM round-trip < 3s, story 4 soft-pause reversible without restart, story 6 workspace-exclusion holds).

## 2. Source — authoritative task body

**Tasks for m1-a are `m1.plan.md` §13 Tasks 8–13**, read verbatim from that file at dispatch time:

| Task | Title | Files (creates) | `requires:` |
|---|---|---|---|
| 8 | bridge crate skeleton + workspace exclusion | `services/bridge/{Cargo.toml,src/main.rs,src/config.rs}` (+ modifies root `Cargo.toml`: `exclude = ["services/bridge"]`) | — |
| 9 | AS transaction server (axum + ruma-appservice-api) | `services/bridge/src/appservice.rs` | task 8 |
| 10 | puppet-on-first-contact | `services/bridge/src/puppet.rs` | task 9 |
| 11 | 1:1 DM relay (text / image / voice) | `services/bridge/src/relay.rs` | task 10 |
| 12 | room provisioning + soft-pause | `services/bridge/src/{provision.rs,soft_pause.rs}` | task 11 |
| 13 | integration test + registration + docker-compose | `services/bridge/{registration.yaml,docker-compose.yml,tests/dm_round_trip.rs}` | task 12 |

> **Dispatch rule:** when authoring each impl-task brief, read the FULL §13 task block (FILES / IMPLEMENT / MIRROR / GOTCHA / VALIDATE) from `m1.plan.md` — do NOT rely on this table alone. The table is a navigation aid; the §13 block is the contract.
>
> **Cohort shape:** Tasks 8→13 form a strict `requires:` chain (8←9←10←11←12←13). **No `[P]` markers** — every task depends on its predecessor's crate state. Dispatch **serially**, one task at a time, each gated on the prior task's `validate-pending-laptop` reaching `result: pass`. Task 8 is the non-`[P]` entry (it also doubles as the Tree-A pre-flight: it establishes the crate + the `exclude` invariant before anything else).

## 5. Metadata + complexity

- **Phase branch:** `phase-m1-a` (cut at bm-cut).
- **Shape:** **pre-Shape-G** — cargo runs on the laptop (clarify `-045`, R9). NO `--features full` for Tree A (bridge has no such feature). Validate-pending-laptop commands run **from inside `services/bridge/`** (NOT `--workspace`), except the story-6 zero-Matrix-deps assertion which IS a workspace command.
- **Estimated tasks:** 6 (Tasks 8–13). Task 14 (Tree C docs/AGPL) + Task 15 (retro) are M1-level; per the m1-a-bootstrap close-out note, fold Task 14 into m1-a and author the m1-a retro at phase close.
- **Cargo budget:** ~3–4 GB peak (bridge crate alone, off-workspace). Not concurrent with any Tree-B work (m1-b already shipped).
- **Complexity caveat (m1.plan.md §18-R1):** the numeric complexity metric **under-counts** Tree A — large greenfield LOC (matrix-sdk + ruma, no in-repo MIRROR sibling) scoring low via "crates touched +1" + "tasks above 5" only. Treat Tasks 9/11/13 as higher-effort than their numeric contribution suggests.

## 7. Preflight guardrails (m1-a-binding subset of m1.plan.md §7)

- **R8 (Tree A is NOT the Lemmy harness):** `services/bridge/**` follows its own crate-local conventions — own error type, own `Cargo.toml`, external `matrix-sdk`/`ruma-appservice-api` examples (read via Ref MCP before writing). Do **NOT** apply Lemmy-workspace lessons (LemmyResult discipline, Diesel, `--features full`, the `crates/server/tests/e2e.rs` harness) to the bridge. Per `feedback_read_canonical_before_writing_spec.md` Tier-2 (read 1–2 external sibling examples first, since there is no in-repo sibling).
- **R9 (validate-pending-laptop, write-then-stop):** every impl task writes the `validate-pending-laptop` DQ entry (`commands` per the task's VALIDATE line), commits + pushes, then **STOPS** — it does NOT run cargo on the daemon. Per `feedback_validate_pending_laptop_write_then_stop.md` + `[[issue_note_worker_redundant_daemon_cargo]]`.

## 12. NOT building (m1-a scope boundaries)

- **NO Tree-B work** — Tasks 1–7 shipped in m1-b (merged `d6d027794`). m1-a consumes the `bridge_notify` POST seam + `governance_messaging_config` table; it never edits them.
- **NO governance-triggered rooms** — room provisioning is **manual only** (§12; OQ-V2-09).
- **NO portable/federated identity** — the puppet `messaging_user_id` map is bridge-local + ephemeral-to-M1 (M2 / ADR-016 replaces it with B-actor IDs). Do not design it as portable.
- **NO Matrix dep in the workspace `members`** — bridge stays in `exclude` (story 6 invariant; catch-fire if violated).

## 13. Step-by-step tasks

**See `m1.plan.md` §13 Tasks 8–13** (authoritative). Reproduced as a navigation table in §2 above; the FILES/IMPLEMENT/MIRROR/GOTCHA/VALIDATE blocks live only in `m1.plan.md` to avoid drift.

Plus M1-level tail (per m1-a-bootstrap §8):
- **Task 14 (Tree C):** `m1.plan.md` §13 Task 14 — chat-plane design docs + AGPL source-disclosure notice extension. Folded into m1-a.
- **Task 15 (retro):** authored at m1-a phase close (advisor, gate 6).

## 15. Validation (DoD) — m1-a subset

> Pre-Shape-G; cargo runs on the laptop (clarify `-045`). Advisor dry-runs the Tree-A DoD against HEAD at the relevant gates.

### 15.5 Tree A validation (laptop — inside `services/bridge/`)

```bash
cd services/bridge && cargo check ; echo "exit: $?"                       # EXPECT 0 (Tasks 8–12)
cd services/bridge && cargo clippy -- -D warnings ; echo "exit: $?"       # EXPECT 0
# Docker-compose-gated milestone (Task 13):
cd services/bridge && docker compose up -d && cargo test --test dm_round_trip -- --ignored ; echo "exit: $?" ; docker compose down   # EXPECT 0 — round-trip < 3s
```

### 15.6 Cross-cutting verification (m1-a-binding rows)

- [ ] `cargo tree --workspace 2>/dev/null | grep -c -E 'matrix-sdk|ruma'` == `0` (story 6 — exclusion holds).
- [ ] Root `Cargo.toml` has `exclude = ["services/bridge"]` (not just absence from `members`).
- [ ] `crates/apub/objects/src/protocol/person.rs` byte-unchanged (no AP person changes from the bridge).
- [ ] No new `plugin_hook_*` definition (bridge wires the existing notify seam only).
- [ ] R8: no Lemmy-workspace lesson applied to `services/bridge/**`.

> **Linux-compile gate:** likely **non-binding** for m1-a (per m1-a-bootstrap §5 + `feedback_linux_compile_proof_is_a_gate.md`) — the bridge adds NO migration and NO workspace `Cargo.toml` change beyond the `exclude` line, so the Option-2 trigger (Cargo.toml/Cargo.lock/migrations OR cfg(unix)/path-sep code) may not fire. **Re-evaluate at bm-pr** against the actual phase diff.

## 16a. Stories — m1-a verifies stories 1, 4, 6

> `/brehon-verify m1-a` iterates these three (the Tree-A-native subset; stories 2/3/5 are Tree-B-side, verified in m1-b). Each checkpoint runs against the `phase-m1-a` worktree; phantoms trigger catch-fire.

### Story 1: 1:1 DM text+image+voice round-trips < 3s

- **Composing tasks:** Tasks 8–13 (Tree A) + Task 6 notify seam (already on governance-v0).
- **Checkpoint:** `cd services/bridge && docker compose up -d && cargo test --test dm_round_trip -- --ignored && docker compose down`
- **Expected:** `1 passed; 0 failed`; round-trip latency assertion < 3s.
- **Brief-Scope outputs:** `services/bridge/src/{relay.rs,puppet.rs}`, `tests/dm_round_trip.rs`, `docker-compose.yml` exist + non-empty.

### Story 4: soft-pause reversible without restart

- **Composing tasks:** Task 12 (`soft_pause.rs`) + Task 13 (integration).
- **Checkpoint:** the enable→disable→enable assertion inside `dm_round_trip.rs`.
- **Expected:** relay drains on disable, resumes on enable; no restart, no data loss.
- **Brief-Scope outputs:** `services/bridge/src/soft_pause.rs` exists + drains-to-idle (OQ-V2-09 answers A/B/C/D).

### Story 6: `services/bridge/` workspace-excluded, zero Matrix deps

- **Composing tasks:** Task 8 (exclude + skeleton).
- **Checkpoint:** `cargo tree --workspace 2>/dev/null | grep -c -E 'matrix-sdk|ruma'`
- **Expected:** `0`.
- **Brief-Scope outputs:** root `Cargo.toml` has `exclude = ["services/bridge"]`; `services/bridge/Cargo.toml` exists with `matrix-sdk` dep.

## 18. Risks (m1-a-specific, from m1-a-bootstrap §4 + §7)

- **Tuwunel verify-items (Task 9), no compile signal:** #219 whoami response-code; #465 `ip_source` NOT set (loopback AS); federation-disabled (`allow_federation=true; forbidden_remote_server_names=[".*"]`); never-switch-fork. Each must be confirmed against the pinned Tuwunel image before Task 9 is "done". A verify-item that cannot be confirmed → surface to user (integration blocker, not a code defect) — NOT a §G4 auto-fix.
- **First greenfield-crate phase (R8):** no in-repo MIRROR sibling. Impl agents read external matrix-sdk/ruma quickstarts. Validation invocation shape differs (`cd services/bridge && cargo check`, not workspace cargo) — verify the bg-cargo exit marker, not the harness "exit 0" (`feedback_background_task_notification_lies.md`).
- **MiniMax A/B trial PAUSED at 3/5:** Tree-A tasks are greenfield (no MIRROR sibling) → likely **ineligible** per minimax-m27-trial-1.md §0.1 criteria. Do NOT fire the trial on Tree-A tasks unless one genuinely qualifies; the remaining 2 eligible tasks come from a future MIRROR-heavy phase.

## 19. Notes

- Entry-point handover: `.claude/PRPs/handovers/m1-a-bootstrap.md` (read RESUME block first).
- Running-state scratchpad: PMD `workflow_state_m1_a.md`.
- Carry-forward from m1-b: read `workflow_state_m1_b.md` ONCE.
- Pending obligation: rotate the MiniMax key (user-side, console) — precondition met (T3/T4/T5 validated); tracked in `project_minimax_key_rotate_after_m1b_trial.md`.

## 20. Confidence

High on the task body (it shipped through M1 gate-1 review as part of m1.plan.md and Tree B already validated the seam contract it consumes). Medium on the Tuwunel integration verify-items (Task 9) — those are the genuine unknowns with no compile-time signal and gate the integration test.
