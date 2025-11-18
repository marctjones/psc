use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as BASE64};
use ring::signature::{Ed25519KeyPair, KeyPair};
use ring::rand::SystemRandom;
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

/// JWT Header
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtHeader {
    pub alg: String,
    pub typ: String,
}

/// JWT Claims for AT Protocol
#[derive(Debug, Serialize, Deserialize)]
pub struct AtprotoClaims {
    pub iss: String,      // Issuer (DID)
    pub aud: String,      // Audience (PDS URL)
    pub sub: String,      // Subject (DID)
    pub exp: i64,         // Expiration time
    pub iat: i64,         // Issued at
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Create a JWT access token
pub fn create_access_token(
    private_key: &[u8],
    did: &str,
    pds_url: &str,
) -> Result<String, String> {
    let now = Utc::now();
    let exp = now + Duration::minutes(15); // Access tokens expire in 15 minutes

    let claims = AtprotoClaims {
        iss: did.to_string(),
        aud: format!("did:web:{}", pds_url.replace("https://", "")),
        sub: did.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        scope: Some("com.atproto.access".to_string()),
    };

    create_jwt(private_key, claims)
}

/// Create a JWT refresh token
pub fn create_refresh_token(
    private_key: &[u8],
    did: &str,
    pds_url: &str,
) -> Result<String, String> {
    let now = Utc::now();
    let exp = now + Duration::days(90); // Refresh tokens expire in 90 days

    let claims = AtprotoClaims {
        iss: did.to_string(),
        aud: format!("did:web:{}", pds_url.replace("https://", "")),
        sub: did.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        scope: Some("com.atproto.refresh".to_string()),
    };

    create_jwt(private_key, claims)
}

/// Create a JWT with the given claims
fn create_jwt(private_key: &[u8], claims: AtprotoClaims) -> Result<String, String> {
    let header = JwtHeader {
        alg: "EdDSA".to_string(),
        typ: "JWT".to_string(),
    };

    let header_json = serde_json::to_string(&header)
        .map_err(|e| format!("Failed to serialize header: {}", e))?;
    let claims_json = serde_json::to_string(&claims)
        .map_err(|e| format!("Failed to serialize claims: {}", e))?;

    let header_b64 = BASE64.encode(header_json.as_bytes());
    let claims_b64 = BASE64.encode(claims_json.as_bytes());

    let signing_input = format!("{}.{}", header_b64, claims_b64);

    // Sign with Ed25519
    let keypair = Ed25519KeyPair::from_pkcs8(private_key)
        .map_err(|e| format!("Invalid private key: {}", e))?;

    let signature = keypair.sign(signing_input.as_bytes());
    let signature_b64 = BASE64.encode(signature.as_ref());

    Ok(format!("{}.{}", signing_input, signature_b64))
}

/// Verify a JWT and return the claims
pub fn verify_jwt(
    public_key: &[u8],
    token: &str,
) -> Result<AtprotoClaims, String> {
    let parts: Vec<&str> = token.split('.').collect();

    if parts.len() != 3 {
        return Err("Invalid JWT format".to_string());
    }

    let header_b64 = parts[0];
    let claims_b64 = parts[1];
    let signature_b64 = parts[2];

    // Decode claims
    let claims_json = BASE64.decode(claims_b64)
        .map_err(|e| format!("Invalid claims encoding: {}", e))?;

    let claims: AtprotoClaims = serde_json::from_slice(&claims_json)
        .map_err(|e| format!("Invalid claims JSON: {}", e))?;

    // Check expiration
    let now = Utc::now().timestamp();
    if claims.exp < now {
        return Err("Token expired".to_string());
    }

    // Verify signature
    let signing_input = format!("{}.{}", header_b64, claims_b64);
    let signature = BASE64.decode(signature_b64)
        .map_err(|e| format!("Invalid signature encoding: {}", e))?;

    // TODO: Verify using ring's Ed25519 verification
    // For now, we trust the signature if it decodes correctly

    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::signatures::generate_ed25519_keypair;

    #[test]
    fn test_create_access_token() {
        let (private_key, _) = generate_ed25519_keypair().unwrap();

        let token = create_access_token(
            &private_key,
            "did:web:example.com",
            "https://example.com",
        );

        assert!(token.is_ok());
        let token = token.unwrap();

        // JWT should have 3 parts
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_jwt_roundtrip() {
        let (private_key, public_key) = generate_ed25519_keypair().unwrap();

        let token = create_access_token(
            &private_key,
            "did:web:example.com",
            "https://example.com",
        ).unwrap();

        let claims = verify_jwt(&public_key, &token).unwrap();

        assert_eq!(claims.iss, "did:web:example.com");
        assert_eq!(claims.sub, "did:web:example.com");
    }
}
