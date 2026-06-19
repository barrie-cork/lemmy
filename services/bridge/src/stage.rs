// Task 4/5/6 wire Stage into request handlers; allow dead_code until then.
#![allow(dead_code)]

use std::collections::VecDeque;
use anyhow::{anyhow, Context, Result};
use rusqlite::Connection;
use crate::bridge_room;

/// Seat state for the participant currently at the mic position.
/// Participants in the FIFO queue are implicitly in Watcher status.
#[derive(Debug, Clone, PartialEq)]
pub enum SeatState {
    Watcher,
    /// Granted publish; awaiting activation (Task 4 adds the 30s grace boundary).
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
    /// Note: the 30s grace timer is Task 4; this leaves the Promoted participant awaiting
    /// on_activate with no timeout boundary.
    pub fn promote_next(&mut self, sink: &mut dyn GrantSink, conn: &Connection) -> Result<()> {
        let head = self
            .fifo
            .pop_front()
            .ok_or_else(|| anyhow!("promote_next: FIFO is empty"))?;
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

    /// Stub signature — Task 4 adds the tokio::select! grace-timer wiring.
    pub fn on_grace_expired(
        &mut self,
        _p: &str,
        _sink: &mut dyn GrantSink,
        _conn: &Connection,
    ) -> Result<()> {
        // Task 4: RevokePublish(_p) + promote_next(_sink, _conn)
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
                    sink.apply(GrantCmd::RevokePublish(target.to_string()));
                    self.current = None;
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
                self.fifo.retain(|p| p.as_str() != target);
                sink.apply(GrantCmd::GrantPublish(target.to_string()));
                self.current = Some((target.to_string(), SeatState::Promoted));
                self.flush_queue(conn)?;
                Ok(())
            }
        }
    }

    /// Transfer the chair to `to`. Returns `(from, to)` for the chain entry (Task 5 emits).
    /// Persists the new chair_id immediately.
    pub fn transfer_chair(&mut self, to: &str, conn: &Connection) -> Result<(String, String)> {
        let from = self
            .chair
            .take()
            .ok_or_else(|| anyhow!("transfer_chair: no current chair"))?;
        self.chair = Some(to.to_string());
        bridge_room::write_chair_id(conn, self.case_id, &self.room_type, to)?;
        Ok((from, to.to_string()))
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

        let grants: Vec<&str> = sink.cmds.iter().filter_map(|c| {
            if let GrantCmd::GrantPublish(p) = c { Some(p.as_str()) } else { None }
        }).collect();
        assert_eq!(
            grants,
            &["W1", "W2", "W3", "W4"],
            "GrantPublish must fire in FIFO order"
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
}
