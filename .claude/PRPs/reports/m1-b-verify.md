# /brehon-verify — m1-b (phase-m1-b @ 5affadb05)

Verified 2026-06-04 against `phase-m1-b` tip `5affadb05` (post CR F2 fix).

## Scope note

`phase-m1-b` delivers **Tree B only** (plan Tasks 0–7: governance-side messaging config + bridge-notify wiring). Tree A (`services/bridge/`, Tasks 8–13) lands in a later sub-phase (`phase-m1-a`). The §16a stories that reference Tree A artifacts (Story 1 DM round-trip, Story 4 soft-pause, Story 6 workspace-exclude) are **out of scope** for this PR and are NOT verified here — verifying them against `phase-m1-b` would be a false phantom.

## Tree-B stories verified

| Story | Brief-Scope output | Result |
|---|---|---|
| **Story 2** — admin identity-policy change persists | `messaging_config.rs` contains `admin_set_messaging_config` + `admin_get_messaging_config`; route registered in `routes/src/lib.rs` | ✓ both handlers present (1+1); 4 `messaging` route refs in routes/lib.rs |
| **Story 3** — `messaging_enabled=false` preserves clean v0 posture | `bridge_notify.rs` early-returns on `messaging_enabled=false`; seed row `messaging_enabled=false` in `up.sql` | ✓ `if !enabled { return Ok(()) }` present; seed row INSERT present in up.sql |
| **Story 5** — identity-policy validator rejects jury/appeals override | `validate_identity_policy` present in `messaging_config.rs` and called before insert (ADR-015 pin) | ✓ `validate_identity_policy` defined; called as `validate_identity_policy(&data)?` before the typed-column insert |

## Validation evidence (all advisor-laptop, pre-Shape-G)

- `cargo check --workspace --features full`: ✓ 0 err 0 warn (Task 6 final: 13m07s; F2 fix: 1m27s)
- `cargo test --no-run -p lemmy_server --test e2e`: ✓ 0 err 0 warn (14m50s)
- `cargo test --test e2e -p lemmy_server <2 messaging fns>`: ✓ 2 passed (45.8s + 46.2s)
- **Linux-compile gate** `cargo-linux.sh check --workspace --features full` (Docker rust:1.95): ✓ 0 err 0 warn (23m28s)

## CR triage (PR #177)

3 CodeRabbit findings, user-approved triage (gate 3):
- **F2** (read_current nondeterministic order) → fix-in-pr ✓ (`.then_order_by(id.desc())`, validated)
- **F1** (is_admin vs capability check) → rebut (plan-mandated `is_admin`; posted on PR)
- **F3** (e2e Some(false) vs absent-row) → rebut (migration seeds the row; posted on PR)

## Outcome

**All in-scope (Tree-B) stories ✓. No phantom completions.** Advance to merge confirm (gate 5).
