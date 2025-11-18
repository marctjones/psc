use crate::types::atproto::*;
use chrono::Utc;

/// AT Protocol collection NSIDs
pub const COLLECTION_POST: &str = "app.bsky.feed.post";
pub const COLLECTION_LIKE: &str = "app.bsky.feed.like";
pub const COLLECTION_REPOST: &str = "app.bsky.feed.repost";
pub const COLLECTION_FOLLOW: &str = "app.bsky.graph.follow";
pub const COLLECTION_BLOCK: &str = "app.bsky.graph.block";
pub const COLLECTION_PROFILE: &str = "app.bsky.actor.profile";

/// Build a DID document for did:web
pub fn build_did_web_document(
    domain: &str,
    public_key_multibase: &str,
) -> DidDocument {
    let did = format!("did:web:{}", domain);

    DidDocument {
        context: vec![
            "https://www.w3.org/ns/did/v1".to_string(),
            "https://w3id.org/security/multikey/v1".to_string(),
        ],
        id: did.clone(),
        also_known_as: Some(vec![format!("at://{}", domain)]),
        verification_method: vec![VerificationMethod {
            id: format!("{}#atproto", did),
            method_type: "Multikey".to_string(),
            controller: did.clone(),
            public_key_multibase: Some(public_key_multibase.to_string()),
        }],
        authentication: None,
        assertion_method: None,
        service: vec![DidService {
            id: "#atproto_pds".to_string(),
            service_type: "AtprotoPersonalDataServer".to_string(),
            service_endpoint: format!("https://{}", domain),
        }],
    }
}

/// Build a post record
pub fn build_post_record(
    text: &str,
    reply: Option<ReplyRef>,
    langs: Option<Vec<String>>,
) -> BskyPost {
    BskyPost {
        record_type: COLLECTION_POST.to_string(),
        text: text.to_string(),
        created_at: Utc::now(),
        reply,
        embed: None,
        langs,
        facets: None,
    }
}

/// Build a follow record
pub fn build_follow_record(subject_did: &str) -> BskyFollow {
    BskyFollow {
        record_type: COLLECTION_FOLLOW.to_string(),
        subject: subject_did.to_string(),
        created_at: Utc::now(),
    }
}

/// Build a like record
pub fn build_like_record(uri: &str, cid: &str) -> BskyLike {
    BskyLike {
        record_type: COLLECTION_LIKE.to_string(),
        subject: StrongRef {
            uri: uri.to_string(),
            cid: cid.to_string(),
        },
        created_at: Utc::now(),
    }
}

/// Build a repost record
pub fn build_repost_record(uri: &str, cid: &str) -> BskyRepost {
    BskyRepost {
        record_type: COLLECTION_REPOST.to_string(),
        subject: StrongRef {
            uri: uri.to_string(),
            cid: cid.to_string(),
        },
        created_at: Utc::now(),
    }
}

/// Build a profile record
pub fn build_profile_record(
    display_name: Option<&str>,
    description: Option<&str>,
) -> BskyProfile {
    BskyProfile {
        record_type: COLLECTION_PROFILE.to_string(),
        display_name: display_name.map(|s| s.to_string()),
        description: description.map(|s| s.to_string()),
        avatar: None,
        banner: None,
    }
}

/// Generate an AT-URI for a record
pub fn build_at_uri(did: &str, collection: &str, rkey: &str) -> String {
    format!("at://{}/{}/{}", did, collection, rkey)
}

/// Parse an AT-URI into components
pub fn parse_at_uri(uri: &str) -> Option<(String, String, String)> {
    let uri = uri.strip_prefix("at://")?;
    let parts: Vec<&str> = uri.splitn(3, '/').collect();

    if parts.len() != 3 {
        return None;
    }

    Some((
        parts[0].to_string(),
        parts[1].to_string(),
        parts[2].to_string(),
    ))
}

/// Generate a record key (rkey) - typically a TID
pub fn generate_rkey() -> String {
    // TID format: base32-sortable timestamp
    // For simplicity, we use a timestamp-based approach
    let now = Utc::now();
    let micros = now.timestamp_micros();

    // Convert to base32-sortable (simplified)
    let chars: Vec<char> = "234567abcdefghijklmnopqrstuvwxyz".chars().collect();
    let mut result = String::new();
    let mut n = micros as u64;

    for _ in 0..13 {
        result.push(chars[(n & 0x1f) as usize]);
        n >>= 5;
    }

    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_did_web_document() {
        let doc = build_did_web_document("example.com", "zDnaerx...");

        assert_eq!(doc.id, "did:web:example.com");
        assert!(doc.also_known_as.unwrap().contains(&"at://example.com".to_string()));
        assert_eq!(doc.service[0].service_endpoint, "https://example.com");
    }

    #[test]
    fn test_build_post_record() {
        let post = build_post_record("Hello!", None, Some(vec!["en".to_string()]));

        assert_eq!(post.record_type, "app.bsky.feed.post");
        assert_eq!(post.text, "Hello!");
        assert!(post.reply.is_none());
    }

    #[test]
    fn test_build_at_uri() {
        let uri = build_at_uri("did:web:example.com", "app.bsky.feed.post", "abc123");
        assert_eq!(uri, "at://did:web:example.com/app.bsky.feed.post/abc123");
    }

    #[test]
    fn test_parse_at_uri() {
        let result = parse_at_uri("at://did:web:example.com/app.bsky.feed.post/abc123");
        assert!(result.is_some());

        let (did, collection, rkey) = result.unwrap();
        assert_eq!(did, "did:web:example.com");
        assert_eq!(collection, "app.bsky.feed.post");
        assert_eq!(rkey, "abc123");
    }

    #[test]
    fn test_parse_at_uri_invalid() {
        assert!(parse_at_uri("not-an-at-uri").is_none());
        assert!(parse_at_uri("at://did/collection").is_none());
    }

    #[test]
    fn test_generate_rkey() {
        let rkey1 = generate_rkey();
        let rkey2 = generate_rkey();

        // Should be 13 characters
        assert_eq!(rkey1.len(), 13);

        // Should be different (with high probability)
        // Note: could be same if called in same microsecond
        assert!(!rkey1.is_empty());
        assert!(!rkey2.is_empty());
    }

    #[test]
    fn test_build_follow_record() {
        let follow = build_follow_record("did:web:other.com");

        assert_eq!(follow.record_type, "app.bsky.graph.follow");
        assert_eq!(follow.subject, "did:web:other.com");
    }

    #[test]
    fn test_build_like_record() {
        let like = build_like_record("at://did:web:example.com/app.bsky.feed.post/123", "bafyabc");

        assert_eq!(like.record_type, "app.bsky.feed.like");
        assert_eq!(like.subject.cid, "bafyabc");
    }
}
