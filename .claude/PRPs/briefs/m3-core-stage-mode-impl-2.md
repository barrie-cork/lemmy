# Brief: m3-core-stage-mode impl-2 (Task 2)

## §1 Role + dispatch line

`[role:impl-task] m3-core-stage-mode-task2-livekit-can-publish — see .claude/PRPs/briefs/m3-core-stage-mode-impl-2.md`

## §2 Scope

**Task 2 of plan `.claude/PRPs/plans/m3-core-stage-mode.plan.md` (lines 457–486).** Add a `can_publish: bool` to `mint_access_token` and `canPublish` to `VideoGrant`, so stage mode can mint presenter (publish) vs watcher (no-publish) LiveKit tokens. This is the mic-grant mechanism (clarify `a3d0e9941441-067`: LiveKit publish-grant, NOT Matrix power-levels — those are Phase-4 emergency-mute only).

**Produces (exactly 1 file edit, ONE commit):**
1. `services/bridge/src/livekit_jwt.rs` — `#[serde(rename = "canPublish")] can_publish: bool` on `VideoGrant`; `can_publish: bool` as the final `mint_access_token` param; `video.can_publish = can_publish`. Update the existing `mint_pseudonym_claims` test to pass `can_publish: true` + assert `claims.video.can_publish == true`; add a second test minting a watcher token (`can_publish: false`) asserting `false` (per plan IMPLEMENT).

**The plan's Task 2 (lines 457–486) is the contract.** MIRROR `services/bridge/src/livekit_jwt.rs:5-76` (struct + mint + the existing decode-roundtrip test). GOTCHA (plan line 473): `identity` stays a pseudonym by contract — do NOT add a `person_id` overload (ADR-015). The presenter/watcher distinction is `can_publish`, NOT Matrix power-levels.

**Do NOT touch:** any `crates/**` file; any other `services/bridge/**` file (Task 2 is `livekit_jwt.rs` only); `Cargo.toml`/`Cargo.lock` (no new dep).

**Branch:** forks from `phase-m3-core-stage-mode` (current tip `b2f4d5e9a`).

## §3 Required reading

- `.claude/PRPs/plans/m3-core-stage-mode.plan.md` — Task 2 (457–486), §10.4 (LiveKit access-token `can_publish` pattern), §8 flow design (the presenter/watcher token distinction).
- `services/bridge/src/livekit_jwt.rs:5-76` — the WHOLE file: the `VideoGrant`/`Claims` structs, `mint_access_token`, and the existing `mint_pseudonym_claims` decode-roundtrip test (mirror it for the two new tests).
- **Lessons (mandatory, §2.4 file-class injection — `services/bridge/**`):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo runs on **Linux ONLY** via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Docker `rust:1.95`); the Windows-local `cd services/bridge && cargo` form fails with `ruma-common` E0119.
  - `feedback_linux_compile_proof_is_a_gate.md` — this bridge change writes a `validate-pending-laptop-linux` DQ; the laptop runs `cargo-linux.sh`; `bm-pr` gates on `result:pass`.
  - `feedback_clippy_test_style.md` — the two tests use `LemmyResult<()>`-style with `?`, no `unwrap`/`expect`.

## §4 Constraints

- **ONE commit, 1 file** — `feat(rtc): livekit can_publish presenter/watcher grant (task 2)`.
- **ADR-015 (load-bearing):** `identity` stays a pseudonym — do NOT add any `person_id`/username param or overload. The mic-grant distinction is `can_publish` only.
- **Mic-grant = LiveKit publish-grant, NOT Matrix power-levels** (clarify `a3d0e9941441-067`). Do NOT introduce any Matrix power-level call.
- **NO new dependency** — `Cargo.toml`/`Cargo.lock` unchanged. If you find yourself needing a new dep, STOP and raise a `kind: "blocker"` DQ (the plan asserts no new dep; a dep add is a scope surprise — watchpoint #6).
- **Bridge compiles on Linux ONLY.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml livekit_jwt"]`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself (laptop advisor validates). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push on the worker branch, stop.
- End the commit body with a `LESSON:` trailer if you find anything durable.

## §3a Handover from prior cohort

(none — first cohort)
