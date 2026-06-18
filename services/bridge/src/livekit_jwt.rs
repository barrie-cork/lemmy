use anyhow::Result;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(serde::Serialize, serde::Deserialize)]
struct VideoGrant {
    room: String,
    #[serde(rename = "roomJoin")]
    room_join: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Claims {
    iss: String,
    sub: String,
    nbf: u64,
    exp: u64,
    video: VideoGrant,
}

/// Mint a LiveKit HS256 access token.
///
/// `identity` is the opaque pseudonym string (ADR-015); the caller must supply
/// a pre-derived pseudonym. This function assigns it verbatim to the JWT `sub`.
pub fn mint_access_token(
    api_key: &str,
    api_secret: &str,
    room: &str,
    identity: &str,
    ttl_secs: u64,
) -> Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    let claims = Claims {
        iss: api_key.to_string(),
        sub: identity.to_string(),
        nbf: now,
        exp: now + ttl_secs,
        video: VideoGrant {
            room: room.to_string(),
            room_join: true,
        },
    };
    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(api_secret.as_bytes()),
    )?;
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

    #[test]
    fn mint_pseudonym_claims() {
        let token =
            mint_access_token("api-key", "secret", "room-1", "pseu_abc123", 3600).unwrap();
        let td = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(b"secret"),
            &Validation::new(Algorithm::HS256),
        )
        .unwrap();
        let claims = td.claims;
        assert_eq!(claims.sub, "pseu_abc123");
        assert_eq!(claims.iss, "api-key");
        assert_eq!(claims.video.room, "room-1");
        assert!(claims.video.room_join);
        assert!(claims.exp > claims.nbf);
    }
}
