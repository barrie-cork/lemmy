// Recording-fetch access control. The recording *capture* chain (LiveKit
// Egress trigger, S3 upload, room_recording_uploaded emit) was scaffold with
// no production trigger and was cut 2026-07-02 (chore/cut-dead-code-safe-set);
// re-add alongside the live town-hall-start wiring. The integration suite at
// tests/recording.rs exercises the live fetch + chain endpoints directly.

/// ADR-015 participant-floor: a recording-fetch requester MUST be a participant
/// of the room at recording time.  Cannot be zero under always_pseudonym (else a
/// pseudonymous town hall's recording leaks).  Both args are PSEUDONYMS.
pub fn is_participant(requester_pseudonym: &str, participants: &[String]) -> bool {
    participants.iter().any(|p| p == requester_pseudonym)
}

#[cfg(test)]
mod tests {
    use super::*;

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
