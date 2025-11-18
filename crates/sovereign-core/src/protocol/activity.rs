use crate::types::activitypub::*;
use chrono::Utc;

/// Activity type constants
pub const ACTIVITY_CREATE: &str = "Create";
pub const ACTIVITY_UPDATE: &str = "Update";
pub const ACTIVITY_DELETE: &str = "Delete";
pub const ACTIVITY_FOLLOW: &str = "Follow";
pub const ACTIVITY_ACCEPT: &str = "Accept";
pub const ACTIVITY_REJECT: &str = "Reject";
pub const ACTIVITY_UNDO: &str = "Undo";
pub const ACTIVITY_LIKE: &str = "Like";
pub const ACTIVITY_ANNOUNCE: &str = "Announce";
pub const ACTIVITY_BLOCK: &str = "Block";

/// Public addressing constant
pub const PUBLIC_COLLECTION: &str = "https://www.w3.org/ns/activitystreams#Public";

/// Build a Create activity wrapping a Note
pub fn build_create_activity(
    id: &str,
    actor: &str,
    note: Note,
) -> Activity {
    Activity {
        context: activitypub_context(),
        id: id.to_string(),
        activity_type: ACTIVITY_CREATE.to_string(),
        actor: actor.to_string(),
        object: Some(serde_json::to_value(note).unwrap()),
        target: None,
        to: Some(vec![PUBLIC_COLLECTION.to_string()]),
        cc: Some(vec![format!("{}/followers", actor)]),
        published: Some(Utc::now()),
    }
}

/// Build a Follow activity
pub fn build_follow_activity(
    id: &str,
    actor: &str,
    target: &str,
) -> Activity {
    Activity {
        context: activitypub_context(),
        id: id.to_string(),
        activity_type: ACTIVITY_FOLLOW.to_string(),
        actor: actor.to_string(),
        object: Some(serde_json::Value::String(target.to_string())),
        target: None,
        to: Some(vec![target.to_string()]),
        cc: None,
        published: Some(Utc::now()),
    }
}

/// Build an Accept activity (for accepting follows)
pub fn build_accept_activity(
    id: &str,
    actor: &str,
    follow_activity: Activity,
) -> Activity {
    Activity {
        context: activitypub_context(),
        id: id.to_string(),
        activity_type: ACTIVITY_ACCEPT.to_string(),
        actor: actor.to_string(),
        object: Some(serde_json::to_value(follow_activity).unwrap()),
        target: None,
        to: None,
        cc: None,
        published: Some(Utc::now()),
    }
}

/// Build a Like activity
pub fn build_like_activity(
    id: &str,
    actor: &str,
    object_id: &str,
) -> Activity {
    Activity {
        context: activitypub_context(),
        id: id.to_string(),
        activity_type: ACTIVITY_LIKE.to_string(),
        actor: actor.to_string(),
        object: Some(serde_json::Value::String(object_id.to_string())),
        target: None,
        to: Some(vec![PUBLIC_COLLECTION.to_string()]),
        cc: None,
        published: Some(Utc::now()),
    }
}

/// Build an Announce (boost/reblog) activity
pub fn build_announce_activity(
    id: &str,
    actor: &str,
    object_id: &str,
) -> Activity {
    Activity {
        context: activitypub_context(),
        id: id.to_string(),
        activity_type: ACTIVITY_ANNOUNCE.to_string(),
        actor: actor.to_string(),
        object: Some(serde_json::Value::String(object_id.to_string())),
        target: None,
        to: Some(vec![PUBLIC_COLLECTION.to_string()]),
        cc: Some(vec![format!("{}/followers", actor)]),
        published: Some(Utc::now()),
    }
}

/// Build an Undo activity
pub fn build_undo_activity(
    id: &str,
    actor: &str,
    activity_to_undo: Activity,
) -> Activity {
    Activity {
        context: activitypub_context(),
        id: id.to_string(),
        activity_type: ACTIVITY_UNDO.to_string(),
        actor: actor.to_string(),
        object: Some(serde_json::to_value(activity_to_undo).unwrap()),
        target: None,
        to: None,
        cc: None,
        published: Some(Utc::now()),
    }
}

/// Build a Delete activity
pub fn build_delete_activity(
    id: &str,
    actor: &str,
    object_id: &str,
) -> Activity {
    Activity {
        context: activitypub_context(),
        id: id.to_string(),
        activity_type: ACTIVITY_DELETE.to_string(),
        actor: actor.to_string(),
        object: Some(serde_json::json!({
            "id": object_id,
            "type": "Tombstone"
        })),
        target: None,
        to: Some(vec![PUBLIC_COLLECTION.to_string()]),
        cc: None,
        published: Some(Utc::now()),
    }
}

/// Build a Note object
pub fn build_note(
    id: &str,
    actor: &str,
    content: &str,
    reply_to: Option<&str>,
) -> Note {
    Note {
        context: Some(activitypub_context()),
        id: id.to_string(),
        object_type: "Note".to_string(),
        attributed_to: actor.to_string(),
        content: content.to_string(),
        published: Utc::now(),
        to: Some(vec![PUBLIC_COLLECTION.to_string()]),
        cc: Some(vec![format!("{}/followers", actor)]),
        in_reply_to: reply_to.map(|s| s.to_string()),
        url: Some(id.to_string()),
        sensitive: None,
        summary: None,
        attachment: None,
    }
}

/// Build a WebFinger response
pub fn build_webfinger(
    username: &str,
    domain: &str,
) -> WebFingerResponse {
    let actor_url = format!("https://{}/users/{}", domain, username);

    WebFingerResponse {
        subject: format!("acct:{}@{}", username, domain),
        aliases: vec![actor_url.clone()],
        links: vec![
            WebFingerLink {
                rel: "self".to_string(),
                link_type: Some("application/activity+json".to_string()),
                href: Some(actor_url),
                template: None,
            },
        ],
    }
}

/// Build an Actor document
pub fn build_actor(
    username: &str,
    domain: &str,
    display_name: Option<&str>,
    summary: Option<&str>,
    public_key_pem: &str,
) -> Actor {
    let actor_url = format!("https://{}/users/{}", domain, username);

    Actor {
        context: activitypub_context(),
        id: actor_url.clone(),
        actor_type: "Person".to_string(),
        preferred_username: username.to_string(),
        name: display_name.map(|s| s.to_string()),
        summary: summary.map(|s| s.to_string()),
        inbox: format!("{}/inbox", actor_url),
        outbox: format!("{}/outbox", actor_url),
        followers: format!("{}/followers", actor_url),
        following: format!("{}/following", actor_url),
        liked: Some(format!("{}/liked", actor_url)),
        public_key: ActorPublicKey {
            id: format!("{}#main-key", actor_url),
            owner: actor_url.clone(),
            public_key_pem: public_key_pem.to_string(),
        },
        icon: None,
        image: None,
        url: Some(actor_url),
        published: Some(Utc::now()),
    }
}

/// Build NodeInfo response
pub fn build_nodeinfo(local_posts: u64) -> NodeInfo {
    NodeInfo {
        version: "2.1".to_string(),
        software: NodeInfoSoftware {
            name: "sovereign".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        protocols: vec!["activitypub".to_string()],
        usage: NodeInfoUsage {
            users: NodeInfoUsers {
                total: 1,
                active_month: 1,
                active_halfyear: 1,
            },
            local_posts,
        },
        open_registrations: false,
    }
}

/// Build an empty OrderedCollection
pub fn build_collection(
    id: &str,
    total_items: u64,
    first_page: Option<&str>,
) -> Collection {
    Collection {
        context: activitypub_context(),
        id: id.to_string(),
        collection_type: "OrderedCollection".to_string(),
        total_items,
        first: first_page.map(|s| s.to_string()),
        last: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_note() {
        let note = build_note(
            "https://example.com/posts/1",
            "https://example.com/users/alice",
            "Hello, world!",
            None,
        );

        assert_eq!(note.object_type, "Note");
        assert_eq!(note.content, "Hello, world!");
        assert!(note.in_reply_to.is_none());
    }

    #[test]
    fn test_build_create_activity() {
        let note = build_note(
            "https://example.com/posts/1",
            "https://example.com/users/alice",
            "Test post",
            None,
        );

        let activity = build_create_activity(
            "https://example.com/posts/1/activity",
            "https://example.com/users/alice",
            note,
        );

        assert_eq!(activity.activity_type, "Create");
        assert!(activity.object.is_some());
    }

    #[test]
    fn test_build_follow_activity() {
        let activity = build_follow_activity(
            "https://example.com/follows/1",
            "https://example.com/users/alice",
            "https://other.com/users/bob",
        );

        assert_eq!(activity.activity_type, "Follow");
        assert_eq!(activity.object.unwrap().as_str().unwrap(), "https://other.com/users/bob");
    }

    #[test]
    fn test_build_webfinger() {
        let wf = build_webfinger("alice", "example.com");

        assert_eq!(wf.subject, "acct:alice@example.com");
        assert_eq!(wf.links.len(), 1);
        assert_eq!(wf.links[0].href.as_ref().unwrap(), "https://example.com/users/alice");
    }

    #[test]
    fn test_build_actor() {
        let actor = build_actor(
            "alice",
            "example.com",
            Some("Alice"),
            Some("Hello!"),
            "-----BEGIN PUBLIC KEY-----\ntest\n-----END PUBLIC KEY-----",
        );

        assert_eq!(actor.id, "https://example.com/users/alice");
        assert_eq!(actor.preferred_username, "alice");
        assert_eq!(actor.inbox, "https://example.com/users/alice/inbox");
    }

    #[test]
    fn test_build_nodeinfo() {
        let ni = build_nodeinfo(42);

        assert_eq!(ni.software.name, "sovereign");
        assert_eq!(ni.usage.local_posts, 42);
        assert!(!ni.open_registrations);
    }
}
