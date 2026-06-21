# Brief — m3-core-e2e-pilot fix-impl: emergency_mute accepted-error set

**Role + dispatch:** `[role:impl-task] em-mute-unavail — see .claude/PRPs/briefs/m3-core-e2e-pilot-impl-emergency-mute-fix.md`

Base branch: `phase-m3-core-e2e-pilot`. Single-file, ≤5-line test edit.

## Scope

Add LiveKit v1.7's psrpc-timeout error spelling to the **accepted-error set** in
`services/bridge/tests/emergency_mute.rs` so the base-e2e-stack run passes.

EDIT EXACTLY ONE FILE: `services/bridge/tests/emergency_mute.rs`.
Do NOT touch `src/`, compose files, or any other test.

The one Edit — extend the `is_not_found` predicate (currently 4 `||` arms ending
in `no participant`) with two more arms for the v1.7 psrpc-timeout signal:

```rust
                let is_not_found = err_str.contains("not found")
                    || err_str.contains("participant")
                    || err_str.contains("404")
                    || err_str.contains("no participant")
                    // LiveKit v1.7 routes UpdateParticipant via psrpc to the node
                    // owning the participant's media session. A never-connected
                    // publisher (no Element Call client in the base e2e stack) has
                    // no psrpc handler -> 3s timeout -> 503 "unavailable: no response
                    // from servers". This IS the zero-holder-by-absence state (same
                    // semantics as "not found"); the D2 pilot with real clients
                    // yields Ok(revoked). Verified via LiveKit twirp.go/psrpc logs
                    // 2026-06-21.
                    || err_str.contains("unavailable")
                    || err_str.contains("no response from servers");
```

The anchor block to replace (verbatim, lines ~200-203, confirmed unique — `grep -c`
on `no participant")` returns 1):

```rust
                let is_not_found = err_str.contains("not found")
                    || err_str.contains("participant")
                    || err_str.contains("404")
                    || err_str.contains("no participant");
```

## Required reading

- `services/bridge/tests/emergency_mute.rs` lines 160-230 (the revoke loop + the
  `match result { Ok(..) => .., Err(e) => { let is_not_found = ..; assert!(is_not_found, ..) } }`
  block you are editing — read it before editing so the comment above lands in context).
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate verbatim
  `old_string`/`new_string` anchors before the Edit (the anchor is given above; confirm
  it matches the file at HEAD before applying).
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — error-string handling idiom
  (this edit only matches on `e.to_string().to_lowercase()`, already present; no new
  error-type plumbing).

## Constraints

- This is a fix-impl under a **pre-Shape-G** flow. After the edit: do NOT run cargo
  yourself. Write a `validate-pending-laptop-e2e` DQ entry with
  `commands: ["./scripts/brehon/... (advisor runs the e2e gate)"]`, `e2e_filter: null`,
  `branch: phase-m3-core-e2e-pilot`, `phase_task: "emergency_mute-fix"`, commit + push,
  then **STOP**. The advisor runs the live e2e gate (gate-4 = LOCAL; 2-instance stack
  too heavy for hosted runners). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- Commit subject: `fix(bridge/e2e): accept LiveKit v1.7 psrpc 'unavailable' as zero-holder-by-absence (emergency_mute)`.
- Commit body ends with: `LESSON: LiveKit v1.7 UpdateParticipant on a never-connected participant returns psrpc-timeout '503 unavailable: no response from servers', not 'not found' — the base e2e stack (no WebRTC clients) needs this in the accepted-error set.`
- Push to `phase-m3-core-e2e-pilot` (mid-task DQ push discipline — advisor sees it on next fetch).
- DO NOT add `unavailable` to any NON-test code path — this is strictly the test's
  accepted-error tolerance for the client-less base stack. Real revoke failures in
  production must still surface.
