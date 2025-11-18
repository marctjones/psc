use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// ActivityPub Actor document
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    #[serde(rename = "@context")]
    pub context: serde_json::Value,
    pub id: String,
    #[serde(rename = "type")]
    pub actor_type: String,
    pub preferred_username: String,
    pub name: Option<String>,
    pub summary: Option<String>,
    pub inbox: String,
    pub outbox: String,
    pub followers: String,
    pub following: String,
    pub liked: Option<String>,
    pub public_key: ActorPublicKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<MediaAttachment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<MediaAttachment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<DateTime<Utc>>,
}

/// Public key embedded in Actor
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorPublicKey {
    pub id: String,
    pub owner: String,
    pub public_key_pem: String,
}

/// Media attachment (for icon, image)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAttachment {
    #[serde(rename = "type")]
    pub media_type: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type_mime: Option<String>,
}

/// WebFinger response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFingerResponse {
    pub subject: String,
    pub aliases: Vec<String>,
    pub links: Vec<WebFingerLink>,
}

/// WebFinger link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFingerLink {
    pub rel: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub link_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

/// Generic ActivityPub Activity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    #[serde(rename = "@context")]
    pub context: serde_json::Value,
    pub id: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub actor: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<DateTime<Utc>>,
}

/// ActivityPub Note object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: String,
    pub attributed_to: String,
    pub content: String,
    pub published: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<Vec<MediaAttachment>>,
}

/// NodeInfo response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
    pub version: String,
    pub software: NodeInfoSoftware,
    pub protocols: Vec<String>,
    pub usage: NodeInfoUsage,
    pub open_registrations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfoSoftware {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfoUsage {
    pub users: NodeInfoUsers,
    pub local_posts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfoUsers {
    pub total: u64,
    pub active_month: u64,
    pub active_halfyear: u64,
}

/// Collection (for outbox, followers, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    #[serde(rename = "@context")]
    pub context: serde_json::Value,
    pub id: String,
    #[serde(rename = "type")]
    pub collection_type: String,
    pub total_items: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<String>,
}

/// OrderedCollectionPage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderedCollectionPage {
    #[serde(rename = "@context")]
    pub context: serde_json::Value,
    pub id: String,
    #[serde(rename = "type")]
    pub collection_type: String,
    pub part_of: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
    pub ordered_items: Vec<serde_json::Value>,
}

/// Standard ActivityPub context
pub fn activitypub_context() -> serde_json::Value {
    serde_json::json!([
        "https://www.w3.org/ns/activitystreams",
        "https://w3id.org/security/v1"
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webfinger_serialization() {
        let wf = WebFingerResponse {
            subject: "acct:user@example.com".to_string(),
            aliases: vec!["https://example.com/@user".to_string()],
            links: vec![
                WebFingerLink {
                    rel: "self".to_string(),
                    link_type: Some("application/activity+json".to_string()),
                    href: Some("https://example.com/users/user".to_string()),
                    template: None,
                },
            ],
        };

        let json = serde_json::to_string(&wf).unwrap();
        assert!(json.contains("acct:user@example.com"));
    }

    #[test]
    fn test_actor_serialization() {
        let actor = Actor {
            context: activitypub_context(),
            id: "https://example.com/users/alice".to_string(),
            actor_type: "Person".to_string(),
            preferred_username: "alice".to_string(),
            name: Some("Alice".to_string()),
            summary: Some("Hello!".to_string()),
            inbox: "https://example.com/users/alice/inbox".to_string(),
            outbox: "https://example.com/users/alice/outbox".to_string(),
            followers: "https://example.com/users/alice/followers".to_string(),
            following: "https://example.com/users/alice/following".to_string(),
            liked: None,
            public_key: ActorPublicKey {
                id: "https://example.com/users/alice#main-key".to_string(),
                owner: "https://example.com/users/alice".to_string(),
                public_key_pem: "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----".to_string(),
            },
            icon: None,
            image: None,
            url: None,
            published: None,
        };

        let json = serde_json::to_string_pretty(&actor).unwrap();
        assert!(json.contains("@context"));
        assert!(json.contains("Person"));
    }
}
