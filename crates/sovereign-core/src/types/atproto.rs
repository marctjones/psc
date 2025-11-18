use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// DID Document for AT Protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidDocument {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub also_known_as: Option<Vec<String>>,
    pub verification_method: Vec<VerificationMethod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assertion_method: Option<Vec<String>>,
    pub service: Vec<DidService>,
}

/// Verification method in DID Document
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationMethod {
    pub id: String,
    #[serde(rename = "type")]
    pub method_type: String,
    pub controller: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_multibase: Option<String>,
}

/// Service endpoint in DID Document
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidService {
    pub id: String,
    #[serde(rename = "type")]
    pub service_type: String,
    pub service_endpoint: String,
}

/// Session response from createSession
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub did: String,
    pub handle: String,
    pub access_jwt: String,
    pub refresh_jwt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// AT Protocol record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub uri: String,
    pub cid: String,
    pub value: serde_json::Value,
}

/// Bluesky post record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BskyPost {
    #[serde(rename = "$type")]
    pub record_type: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<ReplyRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub langs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<Facet>>,
}

/// Reply reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyRef {
    pub root: StrongRef,
    pub parent: StrongRef,
}

/// Strong reference to a record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrongRef {
    pub uri: String,
    pub cid: String,
}

/// Rich text facet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Facet {
    pub index: ByteSlice,
    pub features: Vec<FacetFeature>,
}

/// Byte slice for facet
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ByteSlice {
    pub byte_start: usize,
    pub byte_end: usize,
}

/// Facet feature (mention, link, tag)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum FacetFeature {
    #[serde(rename = "app.bsky.richtext.facet#mention")]
    Mention { did: String },
    #[serde(rename = "app.bsky.richtext.facet#link")]
    Link { uri: String },
    #[serde(rename = "app.bsky.richtext.facet#tag")]
    Tag { tag: String },
}

/// Bluesky profile record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BskyProfile {
    #[serde(rename = "$type")]
    pub record_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<BlobRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<BlobRef>,
}

/// Blob reference
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlobRef {
    #[serde(rename = "$type")]
    pub blob_type: String,
    #[serde(rename = "ref")]
    pub reference: BlobLink,
    pub mime_type: String,
    pub size: u64,
}

/// Link within blob reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobLink {
    #[serde(rename = "$link")]
    pub link: String,
}

/// Follow record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BskyFollow {
    #[serde(rename = "$type")]
    pub record_type: String,
    pub subject: String,
    pub created_at: DateTime<Utc>,
}

/// Like record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BskyLike {
    #[serde(rename = "$type")]
    pub record_type: String,
    pub subject: StrongRef,
    pub created_at: DateTime<Utc>,
}

/// Repost record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BskyRepost {
    #[serde(rename = "$type")]
    pub record_type: String,
    pub subject: StrongRef,
    pub created_at: DateTime<Utc>,
}

/// Profile view (from API)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileView {
    pub did: String,
    pub handle: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    pub followers_count: u64,
    pub follows_count: u64,
    pub posts_count: u64,
    pub indexed_at: DateTime<Utc>,
}

/// Feed view post
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedViewPost {
    pub post: PostView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<serde_json::Value>,
}

/// Post view
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostView {
    pub uri: String,
    pub cid: String,
    pub author: ProfileView,
    pub record: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<serde_json::Value>,
    pub reply_count: u64,
    pub repost_count: u64,
    pub like_count: u64,
    pub indexed_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_did_document_serialization() {
        let doc = DidDocument {
            context: vec![
                "https://www.w3.org/ns/did/v1".to_string(),
                "https://w3id.org/security/multikey/v1".to_string(),
            ],
            id: "did:web:example.com".to_string(),
            also_known_as: Some(vec!["at://example.com".to_string()]),
            verification_method: vec![VerificationMethod {
                id: "did:web:example.com#atproto".to_string(),
                method_type: "Multikey".to_string(),
                controller: "did:web:example.com".to_string(),
                public_key_multibase: Some("zDnaerx...".to_string()),
            }],
            authentication: None,
            assertion_method: None,
            service: vec![DidService {
                id: "#atproto_pds".to_string(),
                service_type: "AtprotoPersonalDataServer".to_string(),
                service_endpoint: "https://example.com".to_string(),
            }],
        };

        let json = serde_json::to_string_pretty(&doc).unwrap();
        assert!(json.contains("did:web:example.com"));
    }

    #[test]
    fn test_bsky_post_serialization() {
        let post = BskyPost {
            record_type: "app.bsky.feed.post".to_string(),
            text: "Hello, world!".to_string(),
            created_at: Utc::now(),
            reply: None,
            embed: None,
            langs: Some(vec!["en".to_string()]),
            facets: None,
        };

        let json = serde_json::to_string(&post).unwrap();
        assert!(json.contains("app.bsky.feed.post"));
        assert!(json.contains("Hello, world!"));
    }
}
