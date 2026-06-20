use anyhow::Context;
use sha2::{Digest, Sha256};

/// Hex SHA-256 of the MP4 bytes.  This hash RIDES the governance chain via
/// append_room_event (R11) — computing it here with sha2 is correct; writing
/// it anywhere that BYPASSES append_room_event is an ADR-008/016 violation.
pub fn compute_content_sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

/// Applied by the recording controller; the real impl triggers LiveKit Egress
/// (reqwest + livekit_jwt token) and PUTs the MP4 to the generic-S3 store
/// (endpoint from config — R12, NEVER a hardcoded minio:9000).  Tests use the
/// `Recorder` spy.  Mirror of stage::GrantSink.
pub trait RecordingSink {
    /// Trigger LiveKit Egress for `room_id`.  Pseudonymous overlay only (ADR-015).
    fn trigger_egress(&mut self, room_id: &str) -> anyhow::Result<()>;
    /// Upload the MP4 bytes to the generic-S3 store; return the object URL.
    fn upload(&mut self, key: &str, bytes: &[u8]) -> anyhow::Result<String>;
}

/// Flag-gated recording controller.  When `enabled` is false, returns with
/// ZERO side-effects (no Egress, no upload, no EmitIntent) — the clean-posture
/// invariant (Success Criteria line 145).  When true, drives the sink and
/// pushes the room_recording_uploaded EmitIntent (drained by drain_emits →
/// append_room_event).  `speakers` are PSEUDONYMS (ADR-015).
#[allow(dead_code)] // live town-hall-start trigger lands Phase 6; reachable from the #[ignore] test now.
#[allow(clippy::too_many_arguments)] // 8 params intrinsic to the recording-emit contract (sink/stage/room/bytes/duration/speakers/attendance + flag)
pub fn maybe_record(
    enabled: bool,
    sink: &mut dyn RecordingSink,
    stage: &mut crate::stage::Stage,
    room_id: &str,
    mp4_bytes: &[u8],
    duration_s: i64,
    speakers: Vec<String>,
    attendance_count: i32,
) -> anyhow::Result<()> {
    if !enabled {
        return Ok(()); // <-- the load-bearing flag-gate (delete this → clean-posture test FAILS)
    }
    sink.trigger_egress(room_id)?;
    let content_sha256 = compute_content_sha256(mp4_bytes);
    let media_url = sink.upload(&format!("{room_id}.mp4"), mp4_bytes)?;
    stage.record_uploaded(media_url, content_sha256, duration_s, speakers, attendance_count);
    Ok(())
}

/// The real production sink — `#[allow(dead_code)]` until Phase 6 wires the
/// live town-hall-start trigger (the stage-mode/emergency-mute scaffold flow).
#[allow(dead_code)]
pub struct LiveSink<'a> {
    client: &'a reqwest::Client,
    config: &'a crate::config::BridgeConfig,
}

#[allow(dead_code)]
impl RecordingSink for LiveSink<'_> {
    fn trigger_egress(&mut self, room_id: &str) -> anyhow::Result<()> {
        let api_key = self.config.livekit_api_key.as_deref()
            .context("LIVEKIT_API_KEY not configured")?;
        let api_secret = self.config.livekit_api_secret.as_deref()
            .context("LIVEKIT_API_SECRET not configured")?;
        let livekit_url = self.config.livekit_url.as_deref()
            .context("LIVEKIT_URL not configured")?;
        let token = crate::livekit_jwt::mint_access_token(
            api_key, api_secret, room_id, "bridge-egress", 3600, false,
        )?;
        // Phase 6: live async POST →
        //   self.client
        //     .post(format!("{livekit_url}/twirp/livekit.proto.Egress/StartRoomCompositeEgress"))
        //     .bearer_auth(&token).json(&egress_request).send().await?
        let _ = (livekit_url, token, self.client);
        Ok(())
    }

    fn upload(&mut self, key: &str, bytes: &[u8]) -> anyhow::Result<String> {
        let endpoint = self.config.s3_endpoint.as_deref()
            .context("S3_ENDPOINT not configured")?;
        let bucket_name = self.config.s3_bucket.as_deref()
            .context("S3_BUCKET not configured")?;
        let access_key = self.config.s3_access_key.as_deref()
            .context("S3_ACCESS_KEY not configured")?;
        let secret_key = self.config.s3_secret_key.as_deref()
            .context("S3_SECRET_KEY not configured")?;
        let creds = s3::creds::Credentials::new(
            Some(access_key),
            Some(secret_key),
            None,
            None,
            None,
        )
        .map_err(|e| anyhow::anyhow!("S3 credentials error: {e}"))?;
        let region = s3::Region::Custom {
            region: "us-east-1".to_string(),
            endpoint: endpoint.to_string(),
        };
        let _bucket = s3::Bucket::new(bucket_name, region, creds)
            .map_err(|e| anyhow::anyhow!("S3 bucket error: {e}"))?;
        // Phase 6: live async PUT → _bucket.put_object(key, bytes).await?
        let _ = (key, bytes);
        Ok(format!(
            "{}/{}/{}",
            endpoint.trim_end_matches('/'),
            bucket_name,
            key
        ))
    }
}

/// ADR-015 participant-floor: a recording-fetch requester MUST be a participant
/// of the room at recording time.  Cannot be zero under always_pseudonym (else a
/// pseudonymous town hall's recording leaks).  Both args are PSEUDONYMS.
pub fn is_participant(requester_pseudonym: &str, participants: &[String]) -> bool {
    participants.iter().any(|p| p == requester_pseudonym)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test RecordingSink that records trigger_egress/upload calls in order.
    /// Mirror of stage.rs's Recorder GrantSink spy.
    struct Recorder {
        egress_calls: Vec<String>,
        upload_calls: Vec<(String, Vec<u8>)>,
    }

    impl Recorder {
        fn new() -> Self {
            Self {
                egress_calls: Vec::new(),
                upload_calls: Vec::new(),
            }
        }
    }

    impl RecordingSink for Recorder {
        fn trigger_egress(&mut self, room_id: &str) -> anyhow::Result<()> {
            self.egress_calls.push(room_id.to_string());
            Ok(())
        }

        fn upload(&mut self, key: &str, bytes: &[u8]) -> anyhow::Result<String> {
            self.upload_calls.push((key.to_string(), bytes.to_vec()));
            Ok(format!("https://s3.example.com/{key}"))
        }
    }

    /// Deterministic: compute_content_sha256(b"abc") == known SHA-256 hex of "abc".
    #[test]
    fn content_sha256_is_stable() {
        assert_eq!(
            compute_content_sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        );
    }

    /// Recorder spy compiles and records trigger_egress / upload calls.
    #[test]
    fn recorder_spy_records_calls() {
        let mut r = Recorder::new();
        r.trigger_egress("room-1").unwrap();
        r.trigger_egress("room-2").unwrap();
        let url = r.upload("key.mp4", b"bytes").unwrap();
        assert_eq!(r.egress_calls, vec!["room-1", "room-2"]);
        assert_eq!(r.upload_calls.len(), 1);
        assert_eq!(r.upload_calls[0].0, "key.mp4");
        assert_eq!(r.upload_calls[0].1, b"bytes");
        assert_eq!(url, "https://s3.example.com/key.mp4");
    }

    /// §16a Story 3 (clean-posture negative invariant, R7):
    /// `record_town_halls = false` → ZERO side-effects (no Egress, no S3 upload, no EmitIntent).
    ///
    /// Delete-the-gate check: removing the `if !enabled { return Ok(()) }` guard in
    /// `maybe_record` makes the false-case fire trigger_egress + upload + one EmitIntent,
    /// causing all three asserts below to FAIL.  This proves the test asserts the GATE,
    /// not a trivial always-skip.  See also: `maybe_record_enabled_records_sink_and_emits`.
    #[test]
    fn clean_posture_no_side_effects_when_disabled() {
        let mut recorder = Recorder::new();
        let conn = crate::bridge_room::open(":memory:").expect("open in-memory DB");
        let mut stage = crate::stage::Stage::load(&conn, 1, "governance").expect("load stage");

        maybe_record(
            false,
            &mut recorder,
            &mut stage,
            "room-1",
            b"mp4-data",
            60,
            vec!["speaker-pseudonym".to_string()],
            10,
        ).unwrap();

        assert!(
            recorder.egress_calls.is_empty(),
            "trigger_egress must NOT fire when recording is disabled (clean posture)"
        );
        assert!(
            recorder.upload_calls.is_empty(),
            "upload must NOT fire when recording is disabled (clean posture)"
        );
        assert!(
            stage.pending_emits.is_empty(),
            "pending_emits must be empty when recording is disabled (clean posture)"
        );
    }

    /// Positive companion to `clean_posture_no_side_effects_when_disabled`:
    /// `enabled = true` → sink called ≥1 time + exactly one room_recording_uploaded EmitIntent.
    /// Without this, the negative test could pass trivially if maybe_record were a no-op.
    #[test]
    fn maybe_record_enabled_records_sink_and_emits_intent() {
        let mut recorder = Recorder::new();
        let conn = crate::bridge_room::open(":memory:").expect("open in-memory DB");
        let mut stage = crate::stage::Stage::load(&conn, 2, "governance").expect("load stage");
        stage.chair = Some("chair-pseudonym".to_string());

        maybe_record(
            true,
            &mut recorder,
            &mut stage,
            "room-1",
            b"mp4-data",
            60,
            vec!["speaker-pseudonym".to_string()],
            10,
        ).unwrap();

        assert!(
            !recorder.egress_calls.is_empty(),
            "trigger_egress must fire at least once when recording is enabled"
        );
        assert!(
            !recorder.upload_calls.is_empty(),
            "upload must fire at least once when recording is enabled"
        );
        assert_eq!(
            stage.pending_emits.len(),
            1,
            "exactly one room_recording_uploaded EmitIntent must be queued"
        );
        assert_eq!(
            stage.pending_emits[0].entry_kind,
            "room_recording_uploaded",
            "EmitIntent entry_kind must be room_recording_uploaded"
        );
    }

    /// ADR-015 participant-floor (R9): MUST be a participant to fetch a recording.
    /// - participant pseudonym in the set → true
    /// - non-participant pseudonym → false
    /// - empty participant set → false (floor cannot be zero under always_pseudonym)
    #[test]
    fn is_participant_floor() {
        let set = vec!["alice-pseudo".to_string(), "bob-pseudo".to_string()];
        assert!(is_participant("alice-pseudo", &set), "member of the set → true");
        assert!(!is_participant("carol-pseudo", &set), "non-member → false");
        assert!(
            !is_participant("alice-pseudo", &[]),
            "empty set → false (floor cannot be zero — ADR-015)"
        );
    }
}
