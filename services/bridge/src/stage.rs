// Task 4/5/6 wire Stage into request handlers; allow dead_code until then.
#![allow(dead_code)]

use std::collections::VecDeque;
use anyhow::{anyhow, Context, Result};
use rusqlite::Connection;
use crate::bridge_room;
use crate::room_event_client::RoomEventPayload;

/// Grace period before a promoted-but-not-activated participant is auto-revoked.
/// Timer is tokio::time (virtual-time-controllable) so tests drive it deterministically.
const GRACE_SECS: u64 = 30;

/// Seat state for the participant currently at the mic position.
/// Participants in the FIFO queue are implicitly in Watcher status.
#[derive(Debug, Clone, PartialEq)]
pub enum SeatState {
    Watcher,
    /// Granted publish; awaiting activation (30s grace — run_grace drives the boundary).
    Promoted,
    Speaking,
}

/// Commands emitted by the state machine; the GrantSink adapter applies them to LiveKit.
#[derive(Debug, Clone, PartialEq)]
pub enum GrantCmd {
    GrantPublish(String),   // pseudonym (ADR-015: opaque strings only)
    RevokePublish(String),  // pseudonym
}

/// Applied by the state machine; the real impl re-mints LiveKit tokens (Task 4+ wiring).
/// Tests use the `Recorder` impl below.
pub trait GrantSink {
    fn apply(&mut self, cmd: GrantCmd);
}

/// Chair-override action variants.
pub enum Override {
    ForceDemote,
    ForcePromote,
}

/// Pending room-event POST intent produced by sync state-mutation methods.
/// The async bridge controller (Task 6) drains `Stage::pending_emits` and
/// calls `post_room_event` for each. Fire-and-forget: transport failure is
/// logged and swallowed at the callsite (best-effort chain entry).
pub struct EmitIntent {
    pub entry_kind: &'static str,
    pub payload: RoomEventPayload,
    pub actor_pseudonym: Option<String>,
}

/// In-memory stage state with persisted FIFO queue and chair seat.
///
/// ADR-015: all stored strings are pseudonyms (opaque), never person_id or username.
/// FIFO is single-writer (the chair's bridge controller) — no lock needed.
pub struct Stage {
    case_id: i64,
    room_type: String,
    pub chair: Option<String>,
    /// Raised-hand FIFO; persisted to bridge_room.queue_state on every mutation.
    pub fifo: VecDeque<String>,
    /// Currently promoted or speaking participant. Not persisted (reconnect derives it
    /// from LiveKit state; Task 6 wires the dual-source). None = seat empty.
    pub current: Option<(String, SeatState)>,
    /// Queued room-event POST intents from sync state mutations. Drained by the
    /// async controller (Task 6). In-memory only — cleared on reload (best-effort).
    pub pending_emits: Vec<EmitIntent>,
}

impl Stage {
    /// Load Stage from persisted bridge_room columns. Returns an empty Stage when no
    /// row exists yet (first raise_hand will UPSERT the row via flush_queue).
    pub fn load(conn: &Connection, case_id: i64, room_type: &str) -> Result<Self> {
        let chair = bridge_room::read_chair_id(conn, case_id, room_type)?;
        let fifo = match bridge_room::read_queue_state(conn, case_id, room_type)? {
            Some(json) => serde_json::from_str::<Vec<String>>(&json)
                .map(|v| v.into_iter().collect())
                .context("corrupt bridge_room.queue_state")?,
            None => VecDeque::new(),
        };
        Ok(Stage {
            case_id,
            room_type: room_type.to_string(),
            chair,
            fifo,
            current: None,
            pending_emits: Vec::new(),
        })
    }

    /// Append `p` to the raised-hand FIFO (no dup). Persists the queue immediately.
    pub fn raise_hand(&mut self, p: &str, conn: &Connection) -> Result<()> {
        let owned = p.to_string();
        if !self.fifo.contains(&owned) {
            self.fifo.push_back(owned);
            self.flush_queue(conn)?;
        }
        Ok(())
    }

    /// Pop the FIFO head, emit GrantPublish, and move that participant to Promoted.
    /// Returns Err if the FIFO is empty (invalid transition — type-state guard).
    /// Call `run_grace` after this to start the 30s activation window (Task 4).
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

    /// Transition the current Promoted participant to Speaking.
    /// Returns Err if `p` is not the currently Promoted participant (type-state guard).
    pub fn on_activate(&mut self, p: &str) -> Result<()> {
        match &mut self.current {
            Some((name, state)) => {
                if name.as_str() != p {
                    return Err(anyhow!(
                        "on_activate: {} is not the current participant (current: {})",
                        p,
                        name
                    ));
                }
                if *state != SeatState::Promoted {
                    return Err(anyhow!(
                        "on_activate: {} is in {:?} state, expected Promoted",
                        p,
                        state
                    ));
                }
                *state = SeatState::Speaking;
                Ok(())
            }
            None => Err(anyhow!("on_activate: no participant in seat")),
        }
    }

    /// Auto-revoke the grace-expired Promoted participant and promote the next FIFO head.
    ///
    /// Idempotent: returns Ok without side-effects if `p` is no longer in Promoted state
    /// (activation or chair-demote already happened before the timer fired).
    /// ADR-015: `p` is a pseudonym string — never person_id or username.
    pub fn on_grace_expired(
        &mut self,
        p: &str,
        sink: &mut dyn GrantSink,
        conn: &Connection,
    ) -> Result<()> {
        match &self.current {
            Some((name, SeatState::Promoted)) if name.as_str() == p => {}
            _ => return Ok(()), // already activated or demoted; idempotent guard
        }
        sink.apply(GrantCmd::RevokePublish(p.to_string()));
        self.current = None;
        if !self.fifo.is_empty() {
            self.promote_next(sink, conn)?;
        }
        Ok(())
    }

    /// Drive the 30s grace window for the Promoted participant `p` using virtual time.
    ///
    /// Races `tokio::time::sleep(GRACE_SECS)` against `cancel` (the activation signal).
    /// On sleep expiry without activation: fires `on_grace_expired` (RevokePublish + next-promote).
    /// On `cancel` resolving first: exits cleanly — no revoke.
    ///
    /// The timer is `tokio::time` (never wall-clock) so tests control it with
    /// `tokio::time::advance` and `#[tokio::test(start_paused = true)]` (R7).
    ///
    /// Callers supply the cancel Future:
    /// - Tests: `std::future::pending::<()>()` for the boundary case;
    ///   `std::future::ready(())` when activation already happened.
    /// - Production (Task 5+): a `oneshot::Receiver<()>` that `on_activate` fires.
    pub async fn run_grace<F: std::future::Future + Unpin>(
        &mut self,
        p: &str,
        sink: &mut dyn GrantSink,
        conn: &Connection,
        cancel: F,
    ) -> Result<()> {
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(GRACE_SECS)) => {
                self.on_grace_expired(p, sink, conn)?;
            }
            _ = cancel => {}
        }
        Ok(())
    }

    /// Force-demote or force-promote a participant out of FIFO order.
    ///
    /// - `ForceDemote`: revoke the current seat holder (must be `target`), clear seat.
    /// - `ForcePromote`: remove `target` from FIFO (if present), grant publish, set as Promoted.
    pub fn chair_override(
        &mut self,
        action: Override,
        target: &str,
        sink: &mut dyn GrantSink,
        conn: &Connection,
    ) -> Result<()> {
        match action {
            Override::ForceDemote => match &self.current {
                Some((name, _)) if name.as_str() == target => {
                    let case_id = self.case_id as i32;
                    let lifecycle_stage = self.room_type.clone();
                    let actor = self.chair.clone();
                    sink.apply(GrantCmd::RevokePublish(target.to_string()));
                    self.current = None;
                    self.pending_emits.push(EmitIntent {
                        entry_kind: "room_chair_override",
                        payload: RoomEventPayload {
                            case_id,
                            matrix_room_id: None,
                            lifecycle_stage,
                            member_count: None,
                            action: Some("force_demote".to_string()),
                            target_pseudonym: Some(target.to_string()),
                            from_pseudonym: None,
                            to_pseudonym: None,
                            at: None,
                            federated: None,
                        },
                        actor_pseudonym: actor,
                    });
                    Ok(())
                }
                Some((name, _)) => Err(anyhow!(
                    "force_demote: {} is not the current participant (current: {})",
                    target,
                    name
                )),
                None => Err(anyhow!("force_demote: seat is empty")),
            },
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
                self.flush_queue(conn)?;
                self.pending_emits.push(EmitIntent {
                    entry_kind: "room_chair_override",
                    payload: RoomEventPayload {
                        case_id,
                        matrix_room_id: None,
                        lifecycle_stage,
                        member_count: None,
                        action: Some("force_promote".to_string()),
                        target_pseudonym: Some(target.to_string()),
                        from_pseudonym: None,
                        to_pseudonym: None,
                        at: None,
                        federated: None,
                    },
                    actor_pseudonym: actor,
                });
                Ok(())
            }
        }
    }

    /// Transfer the chair to `to`. Returns `(from, to)` for callers; also pushes a
    /// `room_chair_transferred` EmitIntent to `pending_emits` (Task 5 — first emission).
    /// Persists the new chair_id immediately. ADR-015: `from`/`to` are pseudonyms.
    pub fn transfer_chair(&mut self, to: &str, conn: &Connection) -> Result<(String, String)> {
        let from = self
            .chair
            .take()
            .ok_or_else(|| anyhow!("transfer_chair: no current chair"))?;
        self.chair = Some(to.to_string());
        bridge_room::write_chair_id(conn, self.case_id, &self.room_type, to)?;
        let case_id = self.case_id as i32;
        let lifecycle_stage = self.room_type.clone();
        let now = time::OffsetDateTime::now_utc();
        let at = format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            now.year(),
            now.month() as u8,
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        );
        self.pending_emits.push(EmitIntent {
            entry_kind: "room_chair_transferred",
            payload: RoomEventPayload {
                case_id,
                matrix_room_id: None,
                lifecycle_stage,
                member_count: None,
                action: None,
                target_pseudonym: None,
                from_pseudonym: Some(from.clone()),
                to_pseudonym: Some(to.to_string()),
                at: Some(at),
                federated: None,
            },
            actor_pseudonym: Some(from.clone()),
        });
        Ok((from, to.to_string()))
    }

    /// Emergency mute-all: revoke publish for EVERY locally-known publisher
    /// (instant in-instance effect — option (b)) and push the room_mute_all
    /// EmitIntent. The cross-instance authority is mute_handler::mute_all_power_levels
    /// (Matrix power-levels); this is the LOCAL belt-and-suspenders.
    ///
    /// `publishers` is the explicit locally-known publisher set (the caller enumerates
    /// it from LiveKit room state). ADR-015: every string is a pseudonym.
    /// Zero-holder invariant: after this returns, NO listed publisher retains a grant.
    pub fn mute_all(&mut self, publishers: &[String], federated: bool, sink: &mut dyn GrantSink) {
        for p in publishers {
            sink.apply(GrantCmd::RevokePublish(p.clone()));   // the load-bearing sweep
        }
        self.current = None;
        self.pending_emits.push(EmitIntent {
            entry_kind: "room_mute_all",
            payload: RoomEventPayload {
                case_id: self.case_id as i32,
                matrix_room_id: None,
                lifecycle_stage: self.room_type.clone(),
                member_count: None,
                action: None, target_pseudonym: None,
                from_pseudonym: None, to_pseudonym: None, at: None,
                federated: Some(federated),
            },
            actor_pseudonym: self.chair.clone(),   // chair_pseudonym (ADR-015 pin)
        });
    }

    fn flush_queue(&self, conn: &Connection) -> Result<()> {
        let v: Vec<&str> = self.fifo.iter().map(String::as_str).collect();
        let json = serde_json::to_string(&v)?;
        bridge_room::write_queue_state(conn, self.case_id, &self.room_type, &json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge_room;

    /// Test GrantSink that records commands in order.
    struct Recorder {
        cmds: Vec<GrantCmd>,
    }

    impl Recorder {
        fn new() -> Self {
            Self { cmds: Vec::new() }
        }
    }

    impl GrantSink for Recorder {
        fn apply(&mut self, cmd: GrantCmd) {
            self.cmds.push(cmd);
        }
    }

    fn open_stage(case_id: i64) -> (Stage, rusqlite::Connection) {
        let conn = bridge_room::open(":memory:").expect("open in-memory DB");
        let stage = Stage::load(&conn, case_id, "governance").expect("load stage");
        (stage, conn)
    }

    /// §16a Story 1 — marquee DoD: four raise_hand + four promote_next/on_activate cycles;
    /// GrantPublish must fire in FIFO order for all four participants.
    #[test]
    fn fifo_mic_pass_in_sequence() -> Result<()> {
        let (mut stage, conn) = open_stage(1);
        let mut sink = Recorder::new();

        stage.raise_hand("W1", &conn)?;
        stage.raise_hand("W2", &conn)?;
        stage.raise_hand("W3", &conn)?;
        stage.raise_hand("W4", &conn)?;

        stage.promote_next(&mut sink, &conn)?;
        stage.on_activate("W1")?;

        stage.promote_next(&mut sink, &conn)?;
        stage.on_activate("W2")?;

        stage.promote_next(&mut sink, &conn)?;
        stage.on_activate("W3")?;

        stage.promote_next(&mut sink, &conn)?;
        stage.on_activate("W4")?;

        // Grants still fire in FIFO order.
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
        assert!(
            sink.cmds.windows(2).any(|w|
                w[0] == GrantCmd::RevokePublish("W2".to_string())
                && w[1] == GrantCmd::GrantPublish("W3".to_string())),
            "handoff must RevokePublish(W2) before GrantPublish(W3)"
        );
        assert!(
            sink.cmds.windows(2).any(|w|
                w[0] == GrantCmd::RevokePublish("W3".to_string())
                && w[1] == GrantCmd::GrantPublish("W4".to_string())),
            "handoff must RevokePublish(W3) before GrantPublish(W4)"
        );
        Ok(())
    }

    /// force_promote a watcher out of FIFO order (before the FIFO head),
    /// then force_demote the speaker; exercises the guarded-error path on double-demote.
    #[test]
    fn chair_override_reorder() -> Result<()> {
        let (mut stage, conn) = open_stage(2);
        let mut sink = Recorder::new();

        stage.raise_hand("W1", &conn)?;
        stage.raise_hand("W2", &conn)?;
        stage.raise_hand("W3", &conn)?;

        // Force-promote W3 out of order — W3 is granted before W1 (the FIFO head).
        stage.chair_override(Override::ForcePromote, "W3", &mut sink, &conn)?;
        assert_eq!(
            sink.cmds.last(),
            Some(&GrantCmd::GrantPublish("W3".to_string())),
            "force_promote must emit GrantPublish before the FIFO head"
        );
        // W3 removed from FIFO; [W1, W2] remain.
        assert_eq!(
            stage.fifo.iter().collect::<Vec<_>>(),
            &["W1", "W2"],
            "force_promote must remove target from FIFO"
        );

        // Activate W3 as the speaker.
        stage.on_activate("W3")?;

        // Force-demote W3.
        stage.chair_override(Override::ForceDemote, "W3", &mut sink, &conn)?;
        let revokes: Vec<&str> = sink.cmds.iter().filter_map(|c| {
            if let GrantCmd::RevokePublish(p) = c { Some(p.as_str()) } else { None }
        }).collect();
        assert_eq!(revokes, &["W3"], "force_demote must emit RevokePublish");

        // Guarded-error path: force_demote with no current speaker must fail.
        let err = stage.chair_override(Override::ForceDemote, "W1", &mut sink, &conn);
        assert!(err.is_err(), "force_demote on empty seat must return an error");
        Ok(())
    }

    #[test]
    fn promote_next_on_empty_fifo_errors() -> Result<()> {
        let (mut stage, conn) = open_stage(3);
        let mut sink = Recorder::new();
        let err = stage.promote_next(&mut sink, &conn);
        assert!(err.is_err(), "promote_next on empty FIFO must return an error");
        Ok(())
    }

    #[test]
    fn on_activate_wrong_participant_errors() -> Result<()> {
        let (mut stage, conn) = open_stage(4);
        let mut sink = Recorder::new();
        stage.raise_hand("W1", &conn)?;
        stage.promote_next(&mut sink, &conn)?;
        // W1 is Promoted; activating a different participant must fail.
        let err = stage.on_activate("W2");
        assert!(err.is_err(), "on_activate with wrong participant must return an error");
        Ok(())
    }

    #[test]
    fn raise_hand_no_dup() -> Result<()> {
        let (mut stage, conn) = open_stage(5);
        stage.raise_hand("W1", &conn)?;
        stage.raise_hand("W1", &conn)?;
        assert_eq!(stage.fifo.len(), 1, "raise_hand must not add duplicate entries");
        Ok(())
    }

    #[test]
    fn stage_load_roundtrip() -> Result<()> {
        let conn = bridge_room::open(":memory:").expect("open DB");
        let mut stage = Stage::load(&conn, 10, "governance")?;
        let mut sink = Recorder::new();
        stage.raise_hand("A", &conn)?;
        stage.raise_hand("B", &conn)?;
        // promote_next pops "A"; FIFO should be ["B"] after.
        stage.promote_next(&mut sink, &conn)?;

        // Reload from persisted state.
        let stage2 = Stage::load(&conn, 10, "governance")?;
        assert_eq!(
            stage2.fifo.iter().collect::<Vec<_>>(),
            &["B"],
            "FIFO must survive a reload (queue_state round-trip)"
        );
        Ok(())
    }

    /// §16a Story 2 DoD — the load-bearing 30s boundary (R7):
    /// promote W1 (W2 queued), advance virtual clock 30s WITHOUT activating W1,
    /// assert RevokePublish(W1) THEN GrantPublish(W2).
    ///
    /// Requires tokio features: `time` (sleep) + `test-util` (start_paused / advance).
    #[tokio::test(start_paused = true)]
    async fn grace_no_activate_auto_revokes_and_promotes_next() -> Result<()> {
        let (mut stage, conn) = open_stage(7);
        let mut sink = Recorder::new();
        stage.raise_hand("W1", &conn)?;
        stage.raise_hand("W2", &conn)?;
        stage.promote_next(&mut sink, &conn)?;

        // Spawn the advance task BEFORE awaiting run_grace; otherwise the single-threaded
        // runtime would have nothing to fire the sleep (time is paused).
        let advance_task = tokio::spawn(async {
            tokio::time::advance(std::time::Duration::from_secs(30)).await;
        });
        // cancel = pending (never resolves): only the sleep branch can fire.
        stage
            .run_grace("W1", &mut sink, &conn, std::future::pending::<()>())
            .await?;
        advance_task.await?;

        // cmds[0] = GrantPublish(W1) from promote_next
        // cmds[1] = RevokePublish(W1) from on_grace_expired
        // cmds[2] = GrantPublish(W2) from next promote_next
        assert_eq!(sink.cmds.len(), 3, "promote + grace-revoke + next-promote expected");
        assert_eq!(
            sink.cmds[1],
            GrantCmd::RevokePublish("W1".to_string()),
            "grace expiry must emit RevokePublish(W1)"
        );
        assert_eq!(
            sink.cmds[2],
            GrantCmd::GrantPublish("W2".to_string()),
            "grace expiry must promote next queued watcher"
        );
        Ok(())
    }

    /// §16a Story 3 DoD — transfer_chair pushes a room_chair_transferred EmitIntent
    /// with from_pseudonym, to_pseudonym, at (timestamp), and actor_pseudonym set.
    #[test]
    fn transfer_chair_produces_emit_intent() -> Result<()> {
        let (mut stage, conn) = open_stage(9);
        stage.chair = Some("old_chair".to_string());
        let (from, to) = stage.transfer_chair("new_chair", &conn)?;
        assert_eq!(from, "old_chair");
        assert_eq!(to, "new_chair");
        assert_eq!(stage.pending_emits.len(), 1, "exactly one EmitIntent must be queued");
        let emit = &stage.pending_emits[0];
        assert_eq!(emit.entry_kind, "room_chair_transferred");
        assert_eq!(
            emit.payload.from_pseudonym,
            Some("old_chair".to_string()),
            "from_pseudonym must be the old chair (ADR-015: pseudonym only)"
        );
        assert_eq!(
            emit.payload.to_pseudonym,
            Some("new_chair".to_string()),
            "to_pseudonym must be the new chair"
        );
        assert!(emit.payload.at.is_some(), "at timestamp must be present");
        assert_eq!(
            emit.actor_pseudonym,
            Some("old_chair".to_string()),
            "actor_pseudonym must be the transferring chair"
        );
        Ok(())
    }

    /// §16a Story 4 DoD — chair_override (ForceDemote) pushes a room_chair_override
    /// EmitIntent with action + target_pseudonym; transfer fields absent.
    #[test]
    fn chair_override_produces_emit_intent() -> Result<()> {
        let (mut stage, conn) = open_stage(10);
        let mut sink = Recorder::new();
        stage.chair = Some("chair_pseudonym".to_string());
        stage.raise_hand("W1", &conn)?;
        stage.promote_next(&mut sink, &conn)?;
        stage.on_activate("W1")?;
        stage.chair_override(Override::ForceDemote, "W1", &mut sink, &conn)?;
        assert_eq!(stage.pending_emits.len(), 1, "exactly one EmitIntent must be queued");
        let emit = &stage.pending_emits[0];
        assert_eq!(emit.entry_kind, "room_chair_override");
        assert_eq!(
            emit.payload.action,
            Some("force_demote".to_string()),
            "action must be force_demote"
        );
        assert_eq!(
            emit.payload.target_pseudonym,
            Some("W1".to_string()),
            "target_pseudonym must be the demoted participant"
        );
        assert!(
            emit.payload.from_pseudonym.is_none(),
            "from_pseudonym must be absent in override emit (transfer-only field)"
        );
        assert!(
            emit.payload.to_pseudonym.is_none(),
            "to_pseudonym must be absent in override emit (transfer-only field)"
        );
        assert_eq!(
            emit.actor_pseudonym,
            Some("chair_pseudonym".to_string()),
            "actor_pseudonym must be the chair who issued the override"
        );
        Ok(())
    }

    /// Contrast: activate W1 before grace fires, advance 30s — no revoke fires.
    /// cancel = ready() resolves immediately, selecting the cancel branch over the sleep.
    #[tokio::test(start_paused = true)]
    async fn grace_activate_before_expiry_no_revoke() -> Result<()> {
        let (mut stage, conn) = open_stage(8);
        let mut sink = Recorder::new();
        stage.raise_hand("W1", &conn)?;
        stage.raise_hand("W2", &conn)?;
        stage.promote_next(&mut sink, &conn)?;
        stage.on_activate("W1")?;

        // cancel = ready (resolves immediately): activation already happened, no revoke.
        stage
            .run_grace("W1", &mut sink, &conn, std::future::ready(()))
            .await?;
        tokio::time::advance(std::time::Duration::from_secs(30)).await;

        let revoke_count = sink
            .cmds
            .iter()
            .filter(|c| matches!(c, GrantCmd::RevokePublish(_)))
            .count();
        assert_eq!(
            revoke_count,
            0,
            "activation before grace must not fire RevokePublish"
        );
        Ok(())
    }

    /// §16a Story 1 — mute_all zero-holder negative invariant (cr-4):
    /// call mute_all with an EXPLICIT 4-publisher slice and assert EVERY publisher
    /// receives a RevokePublish command (set-equality, NOT "≥1 revoke fired").
    ///
    /// cr-4 failure mode: a single-presenter path leaves N-1 holders behind in a
    /// multi-publisher scenario. Taking the EXPLICIT `publishers` slice (NOT from
    /// `self.current`, which is single-presenter) exercises N>1 — the exact cr-4 gap.
    ///
    /// Mechanical delete-the-revoke check: deleting the `for p in publishers { sink.apply(RevokePublish) }` loop
    /// makes the set-equality assert fail — this proves the test asserts the zero-holder
    /// invariant, not the happy path.
    #[test]
    fn mute_all_revokes_all_publishers() -> Result<()> {
        let (mut stage, _conn) = open_stage(11);
        let mut sink = Recorder::new();
        // Seat a chair (ADR-015: pseudonym string, not person_id or username).
        stage.chair = Some("chair-pseudonym".to_string());

        // EXPLICIT 4-publisher slice — NOT derived from self.current (single-presenter).
        // N>1 is what makes this the cr-4 failure-mode exercise.
        let publishers: Vec<String> = ["P1", "P2", "P3", "P4"].iter().map(|s| s.to_string()).collect();
        stage.mute_all(&publishers, true, &mut sink);

        // Collect ALL RevokePublish pseudonyms from the recorder.
        let revoked: std::collections::HashSet<String> = sink.cmds.iter().filter_map(|c| {
            if let GrantCmd::RevokePublish(p) = c { Some(p.clone()) } else { None }
        }).collect();
        let expected: std::collections::HashSet<String> =
            ["P1", "P2", "P3", "P4"].iter().map(|s| s.to_string()).collect();
        // Set-equality: EVERY listed publisher must be revoked — no surviving holder.
        assert_eq!(
            revoked, expected,
            "mute_all must revoke EVERY listed publisher (zero-holder invariant, cr-4)"
        );

        // Assert exactly ONE room_mute_all EmitIntent with federated == Some(true) and actor == chair.
        assert_eq!(stage.pending_emits.len(), 1, "exactly one EmitIntent must be queued for mute_all");
        let emit = &stage.pending_emits[0];
        assert_eq!(emit.entry_kind, "room_mute_all", "entry_kind must be room_mute_all");
        assert_eq!(emit.payload.federated, Some(true), "payload.federated must be Some(true)");
        assert_eq!(
            emit.actor_pseudonym,
            Some("chair-pseudonym".to_string()),
            "actor_pseudonym must be the chair pseudonym (ADR-015 pin)"
        );
        Ok(())
    }
}
