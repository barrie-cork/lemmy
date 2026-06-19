# Brief: m3-core-stage-mode fix-impl-cr4 (CR #202 cr-4 — single-presenter revoke-before-grant)

## §1 Role + dispatch line

`[role:impl-task] m3-core-stage-mode-fix-impl-cr4-single-presenter-revoke — see .claude/PRPs/briefs/m3-core-stage-mode-fix-impl-cr4.md`

## §2 Scope

**Fix CodeRabbit finding cr-4 (Major/critical) on PR #202** — the single-presenter invariant violation in `services/bridge/src/stage.rs`. User-approved fix-in-PR (gate-3, 2026-06-19).

**The defect (VERIFIED against code by advisor):** `promote_next` (`:105-113`) and `chair_override(Override::ForcePromote)` (`:237-260`) issue `GrantCmd::GrantPublish` for the new participant and overwrite `self.current` **WITHOUT** first issuing `GrantCmd::RevokePublish` for the prior `self.current` holder. So when a promote happens while a previous participant is still seated (the back-to-back mic-pass flow — `promote_next` → `on_activate` → `promote_next` again, with no intervening revoke), the prior publisher keeps LiveKit publish rights → two concurrent publishers → the single-presenter guarantee breaks. The 30s-grace path (`on_grace_expired`) DOES revoke before promoting (Task 4); the direct-promote path does not.

**Produces (exactly 1 file, ONE commit):** `services/bridge/src/stage.rs`

1. **`promote_next` (`:105-113`)** — before `sink.apply(GrantCmd::GrantPublish(head.clone()));`, revoke any seated holder:
   ```rust
   pub fn promote_next(&mut self, sink: &mut dyn GrantSink, conn: &Connection) -> Result<()> {
       let head = self
           .fifo
           .pop_front()
           .ok_or_else(|| anyhow!("promote_next: FIFO is empty"))?;
       // ponytail: single-presenter invariant — revoke the seated holder before granting the next.
       if let Some((prev, _)) = self.current.take() {
           sink.apply(GrantCmd::RevokePublish(prev));
       }
       sink.apply(GrantCmd::GrantPublish(head.clone()));
       self.current = Some((head, SeatState::Promoted));
       self.flush_queue(conn)?;
       Ok(())
   }
   ```
   (Take `self.current` so it's cleared before reassignment; the `pop_front` `?` stays FIRST so an empty-FIFO error does not revoke the current speaker.)

2. **`chair_override(Override::ForcePromote)` (`:237`)** — same revoke-before-grant before `sink.apply(GrantCmd::GrantPublish(target.to_string()));` (`:242`):
   ```rust
   Override::ForcePromote => {
       let case_id = self.case_id as i32;
       let lifecycle_stage = self.room_type.clone();
       let actor = self.chair.clone();
       self.fifo.retain(|p| p.as_str() != target);
       // ponytail: single-presenter invariant — revoke the seated holder before force-promoting.
       if let Some((prev, _)) = self.current.take() {
           sink.apply(GrantCmd::RevokePublish(prev));
       }
       sink.apply(GrantCmd::GrantPublish(target.to_string()));
       self.current = Some((target.to_string(), SeatState::Promoted));
       // ... (existing flush_queue + pending_emits.push EmitIntent UNCHANGED)
   }
   ```
   Do NOT touch the `EmitIntent` push or `flush_queue` — only insert the revoke before the grant.

3. **Update the `fifo_mic_pass_in_sequence` test (`:344-374`)** — the marquee test currently asserts only GrantPublish ORDER. With revoke-before-grant, the second+ `promote_next` calls now also emit `RevokePublish(prev)`. Update the assertion so it (a) still confirms the 4 grants fire in FIFO order, AND (b) confirms each handoff revokes the prior holder before granting the next. Per CR's proposed shape — assert the command SEQUENCE includes `RevokePublish(W1)` immediately before `GrantPublish(W2)` (and W2→W3, W3→W4). Keep the test deterministic (`Recorder` sink). A clean form:
   ```rust
   // Grants still fire in FIFO order:
   let grants: Vec<&str> = sink.cmds.iter().filter_map(|c| {
       if let GrantCmd::GrantPublish(p) = c { Some(p.as_str()) } else { None }
   }).collect();
   assert_eq!(grants, &["W1", "W2", "W3", "W4"], "GrantPublish must fire in FIFO order");
   // Single-presenter: each handoff revokes the prior holder before granting the next.
   assert!(
       sink.cmds.windows(2).any(|w|
           w[0] == GrantCmd::RevokePublish("W1".to_string())
           && w[1] == GrantCmd::GrantPublish("W2".to_string())),
       "handoff must RevokePublish(W1) before GrantPublish(W2)"
   );
   ```
   (`GrantCmd` already derives `PartialEq` (`:25` `#[derive(Debug, Clone, PartialEq)]`) — the `windows(2)` `==` comparison works as-is. No derive change needed.)

**Do NOT:**
- Touch any other method, the grace path (`run_grace`/`on_grace_expired` already revoke), `room_event_client.rs`, `room_provisioner.rs`, `config.rs`, or any other test (the other 9 stage tests + 2 room_event_client tests must stay green).
- Change the `EmitIntent` / `pending_emits` logic.
- Touch any `crates/**`, migration, `Cargo.toml`/`Cargo.lock`.
- Address CR cr-1/cr-2 (runlog doc-lint — advisor handles those separately) or cr-3 (rebutted, intentional scaffold).

**Branch:** forks from `phase-m3-core-stage-mode` (current tip `9152aa4d6` — Tasks 1-6 complete).

## §3 Required reading

- `services/bridge/src/stage.rs` (WHOLE file, phase tip `9152aa4d6`) — `promote_next` (`:105`), `chair_override` (`:183`, ForcePromote arm `:237`), `on_grace_expired` (the revoke-before-promote precedent — mirror it), `GrantCmd` enum (`:23`), `fifo_mic_pass_in_sequence` (`:344`), the `Recorder` test sink (`:239+`).
- PR #202 CR finding cr-4 (the proposed diff is the canonical fix shape).
- **Lessons (mandatory, §2.4 — `services/bridge/**`):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo runs **Linux ONLY** via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml`.
  - `feedback_linux_compile_proof_is_a_gate.md` — write a `validate-pending-laptop-linux` DQ; laptop runs `cargo-linux.sh`; merge gates on `result:pass`.
  - `feedback_clippy_test_style.md` — no `unwrap`/`expect`/`unwrap_or_default`; tests return `Result<()>` + `?`.

## §4 Constraints

- **ONE commit, 1 file** (`stage.rs`) — `fix(rtc): revoke prior publisher before granting next — single-presenter invariant (CR cr-4, PR #202)`.
- **Revoke-before-grant in BOTH `promote_next` AND `ForcePromote`** — the take()+revoke must run BEFORE the GrantPublish; in `promote_next` the FIFO `pop_front()?` stays first (empty-FIFO error must not revoke the seated speaker).
- **The 9 other stage tests + 2 room_event_client tests stay green** — only `fifo_mic_pass_in_sequence` changes (it gains the revoke-before-grant assertion).
- **No `unwrap`/`expect`/`unwrap_or_default`** (clippy `-D warnings`).
- **Bridge compiles on Linux ONLY.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml stage"]`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself (laptop advisor validates). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.

## §3a Handover from prior cohort

Tasks 1-6 shipped on `phase-m3-core-stage-mode` @ `9152aa4d6` (PR #202 open). CodeRabbit flagged cr-4: `promote_next` + `chair_override(ForcePromote)` grant publish to the new participant without revoking the prior `self.current` holder, breaking the single-presenter guarantee on back-to-back promotes. The grace path (`on_grace_expired`) already revokes-before-promote (Task 4) — this fix brings the direct-promote paths in line. User approved fix-in-PR at gate-3. The fix is ~6 lines (2 revoke-before-grant blocks) + updating the marquee `fifo_mic_pass_in_sequence` test to assert the revoke-before-grant handoff. cr-1/cr-2 (runlog doc-lint) + cr-3 (intentional scaffold, rebutted) are NOT this task.
