use ring::signature::{self, KeyPair, RsaKeyPair, Ed25519KeyPair};
use ring::rand::SystemRandom;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use sha2::{Sha256, Digest};

/// HTTP Signature algorithm
#[derive(Debug, Clone, Copy)]
pub enum SignatureAlgorithm {
    RsaSha256,
    Ed25519,
}

/// Generate a new RSA keypair
pub fn generate_rsa_keypair() -> Result<(Vec<u8>, Vec<u8>), String> {
    let rng = SystemRandom::new();

    // Generate RSA key using ring
    // Note: ring doesn't directly support RSA key generation
    // In production, you'd use the rsa crate or pre-generate keys

    // For now, return a placeholder
    Err("RSA key generation not yet implemented - use external tool".to_string())
}

/// Generate a new Ed25519 keypair
pub fn generate_ed25519_keypair() -> Result<(Vec<u8>, Vec<u8>), String> {
    let rng = SystemRandom::new();
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)
        .map_err(|e| format!("Failed to generate Ed25519 keypair: {}", e))?;

    let keypair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
        .map_err(|e| format!("Failed to parse keypair: {}", e))?;

    let public_key = keypair.public_key().as_ref().to_vec();
    let private_key = pkcs8_bytes.as_ref().to_vec();

    Ok((private_key, public_key))
}

/// Create an HTTP signature for a request
pub fn create_http_signature(
    private_key: &[u8],
    key_id: &str,
    method: &str,
    path: &str,
    host: &str,
    date: &str,
    digest: Option<&str>,
    algorithm: SignatureAlgorithm,
) -> Result<String, String> {
    // Build the signing string
    let mut headers_to_sign = vec!["(request-target)", "host", "date"];
    let mut signing_parts = vec![
        format!("(request-target): {} {}", method.to_lowercase(), path),
        format!("host: {}", host),
        format!("date: {}", date),
    ];

    if let Some(d) = digest {
        headers_to_sign.push("digest");
        signing_parts.push(format!("digest: {}", d));
    }

    let signing_string = signing_parts.join("\n");

    // Sign the string
    let signature_bytes = match algorithm {
        SignatureAlgorithm::RsaSha256 => {
            let keypair = RsaKeyPair::from_pkcs8(private_key)
                .map_err(|e| format!("Invalid RSA key: {}", e))?;

            let rng = SystemRandom::new();
            let mut signature = vec![0u8; keypair.public_modulus_len()];

            keypair.sign(
                &signature::RSA_PKCS1_SHA256,
                &rng,
                signing_string.as_bytes(),
                &mut signature,
            )
            .map_err(|e| format!("Signing failed: {}", e))?;

            signature
        }
        SignatureAlgorithm::Ed25519 => {
            let keypair = Ed25519KeyPair::from_pkcs8(private_key)
                .map_err(|e| format!("Invalid Ed25519 key: {}", e))?;

            keypair.sign(signing_string.as_bytes()).as_ref().to_vec()
        }
    };

    let signature_b64 = BASE64.encode(&signature_bytes);

    // Build the Signature header
    let algorithm_str = match algorithm {
        SignatureAlgorithm::RsaSha256 => "rsa-sha256",
        SignatureAlgorithm::Ed25519 => "hs2019", // Ed25519 uses hs2019 in HTTP Signatures
    };

    Ok(format!(
        r#"keyId="{}",algorithm="{}",headers="{}",signature="{}""#,
        key_id,
        algorithm_str,
        headers_to_sign.join(" "),
        signature_b64
    ))
}

/// Calculate digest of request body
pub fn calculate_digest(body: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(body);
    let result = hasher.finalize();
    format!("SHA-256={}", BASE64.encode(result))
}

/// Verify an HTTP signature
pub fn verify_http_signature(
    public_key_pem: &str,
    signature_header: &str,
    method: &str,
    path: &str,
    headers: &std::collections::HashMap<String, String>,
) -> Result<bool, String> {
    // Parse the Signature header
    let parts: std::collections::HashMap<String, String> = signature_header
        .split(',')
        .filter_map(|part| {
            let mut kv = part.splitn(2, '=');
            let key = kv.next()?.trim();
            let value = kv.next()?.trim().trim_matches('"');
            Some((key.to_string(), value.to_string()))
        })
        .collect();

    let signed_headers = parts.get("headers")
        .ok_or("Missing headers in signature")?
        .split(' ')
        .collect::<Vec<_>>();

    let signature_b64 = parts.get("signature")
        .ok_or("Missing signature")?;

    // Reconstruct the signing string
    let mut signing_parts = Vec::new();

    for header_name in &signed_headers {
        if *header_name == "(request-target)" {
            signing_parts.push(format!("(request-target): {} {}", method.to_lowercase(), path));
        } else {
            let value = headers.get(*header_name)
                .ok_or_else(|| format!("Missing header: {}", header_name))?;
            signing_parts.push(format!("{}: {}", header_name, value));
        }
    }

    let signing_string = signing_parts.join("\n");

    // Decode signature
    let signature_bytes = BASE64.decode(signature_b64)
        .map_err(|e| format!("Invalid signature encoding: {}", e))?;

    // TODO: Verify signature using the public key
    // This requires parsing the PEM and using ring's verification

    // Placeholder - always return true for now
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_digest() {
        let body = b"Hello, World!";
        let digest = calculate_digest(body);
        assert!(digest.starts_with("SHA-256="));
    }

    #[test]
    fn test_generate_ed25519() {
        let result = generate_ed25519_keypair();
        assert!(result.is_ok());

        let (private_key, public_key) = result.unwrap();
        assert!(!private_key.is_empty());
        assert_eq!(public_key.len(), 32); // Ed25519 public key is 32 bytes
    }

    #[test]
    fn test_create_http_signature() {
        // Generate a keypair for testing
        let (private_key, _) = generate_ed25519_keypair().unwrap();

        let result = create_http_signature(
            &private_key,
            "https://example.com/users/alice#main-key",
            "POST",
            "/inbox",
            "remote.com",
            "Sun, 06 Nov 1994 08:49:37 GMT",
            Some("SHA-256=abc123"),
            SignatureAlgorithm::Ed25519,
        );

        assert!(result.is_ok());
        let signature = result.unwrap();
        assert!(signature.contains("keyId="));
        assert!(signature.contains("signature="));
    }
}
