# Brief: m3-core-emergency-mute fix-impl-1 (CR #204 cr-9/cr-10/cr-11 — bridge logic fixes)

## §1 Role + dispatch line

`[role:impl-task] m3-core-emergency-mute-fix-impl-1-bridge-cr-fixes — see .claude/PRPs/briefs/m3-core-emergency-mute-fix-impl-1.md`

## §2 Scope

**Fix CodeRabbit findings cr-9, cr-10, cr-11 on PR #204** — three bridge logic correctness issues. User-approved fix-in-PR (gate-3, 2026-06-19).

**Produces (exactly 2 files, ONE commit):**
1. `services/bridge/src/mute_handler.rs` — fix cr-9 + cr-10
2. `services/bridge/src/stage.rs` — fix cr-11 (chair exclusion) + update marquee test

**The three defects (VERIFIED against code by advisor):**

### cr-9 — `compute_mute_all_override` unconditional insert may LOWER a pre-existing threshold (CRITICAL)

`mute_handler.rs` line ~33: `events.insert("m.call.member", serde_json::json!(mute_level))` is unconditional. If the room already has `events["m.call.member"] = 150` (i.e. the room was already partially locked), the insert LOWERS the threshold to 100 (the mute_level). Fix: only raise, never lower — use `max()`:

```rust
let current = events
    .get("m.call.member")
    .and_then(|v| v.as_i64())
    .unwrap_or(0);
if mute_level > current {
    events.insert("m.call.member", serde_json::json!(mute_level));
}
// Also apply the MSC3401 alias key with the same max() guard:
let current_alias = events
    .get("m.call.member#legacy")
    .and_then(|v| v.as_i64())
    .unwrap_or(0);
if mute_level > current_alias {
    events.insert("m.call.member#legacy", serde_json::json!(mute_level));
}
```

Mirror the max() guard for BOTH the primary and alias keys that `compute_mute_all_override` inserts.

### cr-10 — `as_object_mut()` silent no-op if `events` is non-object (MAJOR)

`mute_handler.rs` line ~34: `power_levels["events"].as_object_mut()` returns `None` silently if `events` is missing or non-object in the GET response. The outer `if let Some(events) = ...` suppresses the error. Fix: treat absence/non-object of `events` as an error (the Matrix spec guarantees the `events` key exists in a valid power-levels event):

```rust
let events = power_levels["events"]
    .as_object_mut()
    .ok_or_else(|| anyhow::anyhow!("power_levels.events missing or non-object"))?;
```

Use `?` propagation — `compute_mute_all_override` should return `anyhow::Result<()>` (check the current signature and adjust if needed).

### cr-11 — `mute_all` revokes the chair's own publish grant (MAJOR)

`stage.rs` line ~341: `mute_all` loops `publishers` and calls `RevokePublish` for ALL of them including the chair. The chair should remain a publisher after mute-all (only non-chair publishers are silenced). Fix: exclude the chair from the revoke sweep:

```rust
pub fn mute_all(&mut self, publishers: &[String], federated: bool, sink: &mut dyn GrantSink) {
    let chair = self.chair.clone();
    for p in publishers {
        if p != &chair {
            sink.apply(GrantCmd::RevokePublish(p.clone()));
        }
    }
    self.current = None;
    // ... existing EmitIntent push unchanged ...
}
```

**Update the `mute_all_revokes_all_publishers` marquee test:** the current test feeds a 4-publisher slice `["P1","P2","P3","P4"]` with no chair. After the fix, a slice that INCLUDES the chair pseudonym must NOT revoke the chair. Update the test to:
1. Keep the existing set-equality assertion for the 4 non-chair publishers (all still revoked).
2. ADD a second case: call `mute_all(&["P1", "P2", chair_pseudonym], ...)` and assert the chair pseudonym does NOT appear in the revoke set.

The `mute_all_revokes_all_publishers` test must still pass; add a second sub-assertion or a second `#[test]` fn `mute_all_skips_chair` (whichever is cleanest given the existing test structure — read the test first).

**Do NOT:**
- Touch `room_event_client.rs`, `mute_handler.rs`→`mute_all_power_levels` (the async fn), `sanction_handler.rs`, `emergency_mute.rs`, or any `crates/**` file.
- Address cr-2/cr-3/cr-7/cr-8 (those are fix-impl-2, separate task).
- Address cr-12 (rebutted — intentional todo!() stub).
- Change the `EmitIntent` / `pending_emits` push in `mute_all` — only add the chair-exclusion guard before the revoke loop.

**Branch:** forks from `phase-m3-core-emergency-mute` (current tip `78a7b2655`).

## §3 Required reading

- `services/bridge/src/mute_handler.rs` (WHOLE file, phase tip) — `compute_mute_all_override` fn signature + body; current return type; the `as_object_mut` pattern; both insert keys.
- `services/bridge/src/stage.rs` (WHOLE file, phase tip) — `mute_all` fn (~line 324+); `mute_all_revokes_all_publishers` test; `GrantCmd` derive list; `self.chair` field type.
- **Lessons (mandatory, §2.4 — `services/bridge/**`):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo runs **Linux ONLY** via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml`.
  - `feedback_linux_compile_proof_is_a_gate.md` — write a `validate-pending-laptop-linux` DQ; merge gates on `result:pass`.
  - `feedback_validate_pending_laptop_write_then_stop.md` — write the DQ and STOP; laptop advisor runs all cargo.
  - `feedback_clippy_test_style.md` — no `unwrap`/`expect`; tests return `anyhow::Result<()>` + `?`.

## §4 Constraints

- **ONE commit, 2 files** (`mute_handler.rs` + `stage.rs`) — `fix(rtc): mute_handler max-guard + events error + stage chair exclusion (CR cr-9/cr-10/cr-11, PR #204)`.
- **cr-9 max() guard:** both `m.call.member` and the MSC3401 alias key must use the max() guard (only raise, never lower).
- **cr-10 `?` propagation:** `as_object_mut()` failure must propagate as an error, not silently no-op. Adjust `compute_mute_all_override` return type to `anyhow::Result<()>` if not already.
- **cr-11 chair exclusion:** revoke loop MUST skip `p == &self.chair`. The `mute_all_revokes_all_publishers` test MUST be updated to assert the chair-skip invariant. No other test may break.
- **No `unwrap`/`expect`/`unwrap_or_default`** (clippy `-D warnings`).
- **Bridge compiles on Linux ONLY.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --bins mute_all_revokes_all_publishers"]`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP**. Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.
- **Handover:** deliver HANDOVER inline in task output (do NOT write to `.claude/PRPs/handovers/` — blocked by sensitive-file guard in Junior worktree).

## §3a Handover from impl tasks 1-4

- Task 2 shipped `mute_handler.rs` with `compute_mute_all_override` (pure) — the defect site for cr-9/cr-10.
- Task 3 shipped `stage.rs::mute_all` — the defect site for cr-11.
- Phase tip `78a7b2655` has both files. This fix brings bridge logic into correctness alignment before merge.
