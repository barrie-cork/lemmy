# Brief: m3-core-stage-mode fix-impl-1 (Task 2 clippy dead_code)

## §1 Role + dispatch line

`[role:impl-task] m3-core-stage-mode-fix-impl1-livekit-dead-code — see .claude/PRPs/briefs/m3-core-stage-mode-fix-impl-1.md`

## §2 Scope

**Fix the 4 `dead_code` clippy errors that failed Task 2's `validate-pending-laptop-linux` validation** (DQ `8f77bcfc296e-001`, CLIPPY_EXIT_101). `cargo check` already passes — the code is sound; the symbols are scaffolding ahead of their Task 3 (`stage.rs`) caller. The fix is to suppress the dead-code lint on each, mirroring the `#[allow(dead_code)]` idiom m3-core-infra already established at `services/bridge/src/config.rs:39` (`brehon_room_event_url`).

This is a **user-authorized** fix (catch-fire was non-allowlist; user chose fix-impl-now 2026-06-19). The recipe is the exact `#[allow(dead_code)]` annotation set below — apply verbatim, no logic changes.

**The 4 failing lints (verbatim from `cargo-linux.sh clippy -D warnings`):**
```
error: struct `VideoGrant` is never constructed   --> src/livekit_jwt.rs:6:8
error: struct `Claims` is never constructed        --> src/livekit_jwt.rs:15:8
error: function `mint_access_token` is never used  --> src/livekit_jwt.rs:27:8
error: fields `livekit_url`, `livekit_api_key`, and `livekit_api_secret` are never read
                                                   --> src/config.rs:56:9
```

**Produces (exactly 2 file edits, ONE commit):**
1. `services/bridge/src/livekit_jwt.rs` — add `#[allow(dead_code)]` to: `struct VideoGrant` (line ~6), `struct Claims` (line ~15), `pub fn mint_access_token` (line ~27). A short comment is welcome: `// wired by stage.rs (Task 3)`.
2. `services/bridge/src/config.rs` — add `#[allow(dead_code)]` to **each** of the 3 fields `livekit_url`, `livekit_api_key`, `livekit_api_secret` (lines ~56-60). Field-level `#[allow]` mirrors the existing `:39` `brehon_room_event_url` pattern exactly.

**Do NOT:**
- Touch any `crates/**` file (Task 1's domain; already validated).
- Touch `mint_access_token`'s signature or body, or the `can_publish` logic Task 2 added (it is correct).
- Add or remove any dependency. `Cargo.toml`/`Cargo.lock` unchanged.
- Use `#[allow(dead_code)]` at module/file scope — annotate the **specific** symbols only (per `feedback_fix_impl_pre_push_cargo_check.md` "NEVER `#[allow]`-spam to bypass" — here the allows are targeted and idiom-matched, NOT a bypass).

**Branch:** forks from the **Task 2 worker branch** `junior/role-impl-task-m3-core-stage-mode-task2-livekit-can-publish-see-claude-prps-briefs-m3-core-stage-mode-impl-2-md-709` (tip `a6d39dbb1`) — so the `#[allow]`s land on Task 2's commit lineage. The daemon will finalize-merge this into `phase-m3-core-stage-mode` alongside Task 2.

## §3 Required reading

- `services/bridge/src/livekit_jwt.rs:1-90` — the WHOLE file; the 3 symbols to annotate are `VideoGrant`, `Claims`, `mint_access_token`.
- `services/bridge/src/config.rs:37-60` — the existing `#[allow(dead_code)]` on `brehon_room_event_url` (line 39) is the mirror target; the 3 `livekit_*` fields are at 56-60.
- **Lessons (mandatory, §2.4 file-class injection — `services/bridge/**`):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo runs on **Linux ONLY** via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Docker `rust:1.95`).
  - `feedback_linux_compile_proof_is_a_gate.md` — write a `validate-pending-laptop-linux` DQ; the laptop runs `cargo-linux.sh`; `bm-pr` gates on `result:pass`.

## §4 Constraints

- **ONE commit, 2 files** — `fix(rtc): allow(dead_code) on livekit scaffold until stage.rs caller (fix-impl 1)`.
- **Targeted `#[allow(dead_code)]` ONLY** — per-symbol / per-field, mirroring `config.rs:39`. No file-scope or module-scope allow. The symbols go live at Task 3; the allow is the standard scaffold-ahead-of-caller idiom this crate already uses.
- **No logic change** — do NOT alter `mint_access_token`, `can_publish`, or any field type. This is annotation-only.
- **Bridge compiles on Linux ONLY.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml livekit_jwt"]`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself (laptop advisor validates). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push on the worker branch, stop.

## §3a Handover from prior cohort

Task 2 (#709) added `can_publish` to `mint_access_token`/`VideoGrant` + 2 tests; it passed `cargo check` but failed `cargo clippy -D warnings` on 4 `dead_code` lints because the runtime caller (`stage.rs`, Task 3) does not exist yet. The 4 symbols are pre-existing m3-core-infra scaffold (present at the phase base `d722cbc1d`), not introduced by Task 2. This fix-impl only suppresses the premature lint; Task 3 will make the symbols live.
