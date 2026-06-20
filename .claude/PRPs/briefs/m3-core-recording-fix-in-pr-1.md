## 1. Role + dispatch line

`[role:impl-task] m3-core-recording fix-in-pr 1 — CR cr-4 (LiveSink fail-closed) + cr-5 (non-empty actor_pseudonym guard)`

Dispatch string:
```
[role:impl-task] m3-core-recording-fix-in-pr-1-cr4-cr5 — see .claude/PRPs/briefs/m3-core-recording-fix-in-pr-1.md
```

## 2. Scope

Apply the TWO fix-in-pr CodeRabbit findings the advisor accepted at gate-3 for PR #205. Exactly **two files**, **one crate** (`brehon-bridge`, Linux-validated). Both are additive hardening; no behaviour change to the flag-gate or the happy path's logic, only fail-closed + an emit-time guard.

**File 1 of 2 — `services/bridge/src/recording.rs`** (modify — cr-4, fail-closed LiveSink):
The `LiveSink` impl's `trigger_egress` (≈:61-77) and `upload` (≈:79-110) are scaffold stubs that currently return `Ok(())` / a fabricated URL **without performing the external operation** — a fail-OPEN that would let `room_recording_uploaded` emit for media never egressed/uploaded once the path is wired. Make them **fail-closed**: replace the final `let _ = (...); Ok(())` / `Ok(format!(...))` with an `anyhow::bail!(...)` naming the method as scaffold-only-until-implemented. Keep the `.context(...)`-checked config reads above (they stay — the config validation is correct); only the **success-without-work tail** becomes a `bail!`. Mirror the suggested-fix diff in the CR thread on recording.rs:77.
- cr-4 exact intent: `trigger_egress` → `anyhow::bail!("LiveSink::trigger_egress is scaffold-only; refuse success until implemented")`; `upload` → `anyhow::bail!("LiveSink::upload is scaffold-only; refuse success until PUT is implemented")`. Preserve the Phase-6 comment blocks (the live-POST / live-PUT sketches) so the implementer has the recipe.

**File 2 of 2 — `services/bridge/src/stage.rs`** (modify — cr-5, non-empty actor_pseudonym guard):
`record_uploaded` (def ≈:373) currently uses `self.chair.clone()` for `actor_pseudonym`, so the `room_recording_uploaded` EmitIntent can carry `actor_pseudonym: None` — an unauditable governance entry that breaks the ADR-016 recording-event contract + the ADR-015 pseudonym binding. Change `record_uploaded` to **validate a non-empty chair pseudonym BEFORE pushing the EmitIntent**, and change its return type from `()` to `anyhow::Result<()>` (mirror the sibling `mute_all`/`record_uploaded` Result patterns if present; otherwise `anyhow::Result<()>`). When `self.chair` is `None` (or empty), return `Err` (`anyhow::bail!("record_uploaded requires a chair pseudonym; refusing to emit room_recording_uploaded with actor_pseudonym=None")`) — do NOT emit. When present, bind `actor_pseudonym = Some(chair)` and push as before, then `Ok(())`.

### 2.1 CALLSITE PROPAGATION (cr-5 signature change — MANDATORY, do all in this commit)

Changing `record_uploaded` `()` → `anyhow::Result<()>` is a callsite-propagating change. Update EVERY callsite in the SAME commit (else `unused_must_use` clippy `-D warnings` failure or `E0308`):

1. **`services/bridge/src/recording.rs:47`** (production) — `stage.record_uploaded(media_url, content_sha256, duration_s, speakers, attendance_count);` is inside `maybe_record` (which already returns `anyhow::Result<()>` and `?`-propagates `trigger_egress`/`upload`). Add `?`: `stage.record_uploaded(...)?;`.
2. **`services/bridge/src/stage.rs:793`** (the `record_uploaded_emits_recording_intent` `#[cfg(test)]` test, which already returns `Result<()>`) — add `?` to the `stage.record_uploaded(...)` call. If the test then needs the chair set to a non-empty pseudonym for the positive case to NOT bail, ensure the test's stage has `chair = Some("<pseudonym>")` before the call (it must already, since it asserts one EmitIntent is queued — verify and keep).
3. **Run `git grep -n 'record_uploaded' services/bridge/src/`** yourself FIRST and confirm exactly these 2 call-sites (def at stage.rs:373 + the doc-comment at :786 are NOT callsites). DoD: edited-callsite count == grep-callsite count (excluding the def + doc-comment lines).

### 2.2 Clean-posture test still holds (do NOT regress)

The `clean_posture_no_side_effects_when_disabled` test (`maybe_record(false, ..)` → zero side-effects) MUST still pass — cr-4/cr-5 only touch the `enabled=true` path + the sink impl + the emit guard. The `!enabled { return Ok(()) }` flag-gate in `maybe_record` is UNCHANGED. If your edit touches that gate, you've gone out of scope — revert.

### 2.3 Boundaries (do NOT)

- Do **NOT** touch cr-2 / cr-3 (requester-pseudonym spoofing + empty participant set) — those are **carry-forward to Phase-6** per gate-3; leave `is_participant` + `handle_recording_fetch` exactly as they are.
- Do **NOT** touch cr-1 (the stale DQ-timestamp finding — rebutted).
- Do **NOT** add a dep, a migration, a new const, or touch any file other than the two named.
- Do **NOT** run cargo on the daemon (R3). Write the `validate-pending-laptop-linux` DQ and **stop**.

## 3. Required reading

- The two CR finding threads on PR #205 (`gh api repos/barrie-cork/lemmy/pulls/205/comments` — cr-4 on `recording.rs:77`, cr-5 on `stage.rs:399`) — the suggested-fix diffs are the canonical intent.
- `services/bridge/src/recording.rs:38-110` (`maybe_record` + the `LiveSink` impl — the cr-4 target + the recording.rs:47 callsite).
- `services/bridge/src/stage.rs:373-405` (`record_uploaded` def — the cr-5 target) + `:786-805` (the `record_uploaded_emits_recording_intent` test — the cr-5 test callsite).
- `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md` (the callsite-enumeration discipline — cr-5's signature change MUST update all callsites in one commit).
- `.claude/lessons/feedback_bridge_validates_on_linux_not_windows.md` (R1 — bridge cargo is Linux-only via `cargo-linux.sh`).
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` (R3 — write the DQ, do NOT run cargo on the daemon).
- `.claude/lessons/feedback_clippy_rerun_after_fix.md` (re-run clippy after the change — the `?`-propagation + `Result` return are exactly the class clippy `-D warnings` catches via `unused_must_use`).

## 4. Constraints

### 4.1 ADR-016/015 contract is LOAD-BEARING for cr-5

The non-empty actor_pseudonym guard is the entire point of cr-5: a `room_recording_uploaded` chain entry with `actor_pseudonym: None` is unauditable (violates the ADR-016 metadata contract + the ADR-015 pseudonym binding). The guard MUST come BEFORE the EmitIntent push — when chair is absent, NOTHING is emitted (return Err, do not push a partial). DoD: `grep -n 'bail\|return Err\|actor_pseudonym' services/bridge/src/stage.rs` over the new `record_uploaded` body shows the guard precedes the `pending_emits.push`.

### 4.2 Validate-pending-laptop-linux DQ then STOP (R2 + R3)

After committing + pushing the worker branch, write a `validate-pending-laptop-linux` DQ entry (use `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`) with:
```json
{
  "from": "impl",
  "kind": "validate-pending-laptop-linux",
  "question": "Run cargo-linux.sh check + clippy + bridge unit tests for fix-in-pr cr-4+cr-5",
  "options": ["pass", "fail"],
  "context": "m3-core-recording fix-in-pr 1 — CR cr-4 (LiveSink fail-closed bail!) + cr-5 (record_uploaded non-empty actor_pseudonym guard, ()->Result<()> + 2 callsite ? propagations). Bridge Linux check + clippy -D warnings (unused_must_use class) + unit tests (record_uploaded_emits_recording_intent + clean_posture_no_side_effects_when_disabled must still pass).",
  "commands": [
    "scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml",
    "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings",
    "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml record_uploaded_emits_recording_intent",
    "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml clean_posture_no_side_effects_when_disabled"
  ],
  "branch": "<your worker branch>",
  "phase_task": "m3-core-recording fix-in-pr 1 (cr-4 + cr-5)",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```
Commit the fix (`fix(rtc): fail-closed LiveSink + non-empty actor_pseudonym guard (CR cr-4+cr-5, PR #205)`) + the DQ write (`chore(decision-queue): impl raised validate-pending-laptop-linux <id> — m3-core-recording fix-in-pr 1`), push, then **STOP**. Do **NOT** run cargo yourself.

### 4.3 Mid-task blocker discipline

If the `record_uploaded` body or the chair field shape doesn't match this brief (e.g. `self.chair` is already `String` not `Option`, so cr-5's None-case is unreachable), do **NOT** guess — raise a `kind: "blocker"` DQ (`from: "impl"`), commit + push, and stop. (If chair is already non-Option, cr-5 may reduce to an `is_empty()` guard — flag it rather than inventing.)

## 5. LESSON trailer

End the final commit body with a `LESSON:` line if you hit anything durable (e.g. the `record_uploaded` chair field was non-Option, narrowing cr-5).
