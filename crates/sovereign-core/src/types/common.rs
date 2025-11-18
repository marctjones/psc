use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// User identity across both protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub did: String,
    pub handle: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_cid: Option<String>,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
}

/// A post that can exist in either protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub uri: String,
    pub protocol: Protocol,
    pub author_did: String,
    pub content: String,
    pub reply_to: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Which protocol a resource belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    #[serde(rename = "activitypub")]
    ActivityPub,
    #[serde(rename = "atproto")]
    AtProto,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::ActivityPub => write!(f, "activitypub"),
            Protocol::AtProto => write!(f, "atproto"),
        }
    }
}

/// Follow relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Follow {
    pub follower: String,
    pub following: String,
    pub protocol: Protocol,
    pub accepted: bool,
    pub created_at: DateTime<Utc>,
}

/// Notification types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationType {
    Follow,
    Like,
    Repost,
    Reply,
    Mention,
}

/// A notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub notification_type: NotificationType,
    pub actor: String,
    pub subject: Option<String>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

/// Configuration for the sovereign instance
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub domain: Option<String>,
    pub cloudflare_api_token: Option<String>,
    pub cloudflare_zone_id: Option<String>,
    pub handle: Option<String>,
    pub server_url: Option<String>,
}

impl Config {
    pub fn is_configured(&self) -> bool {
        self.domain.is_some() && self.cloudflare_api_token.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_display() {
        assert_eq!(Protocol::ActivityPub.to_string(), "activitypub");
        assert_eq!(Protocol::AtProto.to_string(), "atproto");
    }

    #[test]
    fn test_protocol_serde() {
        let ap = Protocol::ActivityPub;
        let json = serde_json::to_string(&ap).unwrap();
        assert_eq!(json, "\"activitypub\"");

        let parsed: Protocol = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Protocol::ActivityPub);
    }

    #[test]
    fn test_config_is_configured() {
        let mut config = Config::default();
        assert!(!config.is_configured());

        config.domain = Some("example.com".to_string());
        assert!(!config.is_configured());

        config.cloudflare_api_token = Some("token".to_string());
        assert!(config.is_configured());
    }
}
