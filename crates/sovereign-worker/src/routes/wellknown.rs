use worker::*;
use sovereign_core::types::activitypub::*;
use sovereign_core::protocol::{build_webfinger, build_nodeinfo, build_did_web_document};

use super::{json_response, get_domain, get_username};

/// WebFinger endpoint
pub async fn webfinger(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let url = req.url()?;
    let resource = url
        .query_pairs()
        .find(|(k, _)| k == "resource")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| Error::RustError("Missing resource parameter".to_string()))?;

    let domain = get_domain(&ctx.env, &req);
    let username = get_username(&ctx.env);

    // Check if the resource matches our user
    let expected_acct = format!("acct:{}@{}", username, domain);
    let expected_url = format!("https://{}/users/{}", domain, username);

    if resource != expected_acct && resource != expected_url {
        return Response::error("User not found", 404);
    }

    let webfinger = build_webfinger(&username, &domain);

    let mut headers = Headers::new();
    headers.set("Content-Type", "application/jrd+json")?;

    let json = serde_json::to_string(&webfinger)
        .map_err(|e| Error::RustError(e.to_string()))?;

    Ok(Response::ok(json)?.with_headers(headers))
}

/// NodeInfo redirect (/.well-known/nodeinfo)
pub async fn nodeinfo_redirect(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let domain = get_domain(&ctx.env, &req);

    let response = serde_json::json!({
        "links": [
            {
                "rel": "http://nodeinfo.diaspora.software/ns/schema/2.1",
                "href": format!("https://{}/nodeinfo/2.1", domain)
            }
        ]
    });

    json_response(&response)
}

/// NodeInfo endpoint
pub async fn nodeinfo(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // TODO: Get actual post count from D1
    let local_posts = 0;

    let nodeinfo = build_nodeinfo(local_posts);
    json_response(&nodeinfo)
}

/// DID Document for AT Protocol
pub async fn did_document(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let domain = get_domain(&ctx.env, &req);

    // TODO: Get actual public key from storage
    let public_key_multibase = "zDnaerDaTF5BXEavCrfRZEk316dpbLsfPDZ3WJ5hRTPFU2169";

    let doc = build_did_web_document(&domain, public_key_multibase);
    json_response(&doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webfinger_format() {
        let wf = build_webfinger("alice", "example.com");
        assert_eq!(wf.subject, "acct:alice@example.com");
    }

    #[test]
    fn test_webfinger_links() {
        let wf = build_webfinger("bob", "test.org");

        assert_eq!(wf.links.len(), 1);
        assert_eq!(wf.links[0].rel, "self");
        assert_eq!(wf.links[0].link_type.as_ref().unwrap(), "application/activity+json");
        assert_eq!(wf.links[0].href.as_ref().unwrap(), "https://test.org/users/bob");
    }

    #[test]
    fn test_nodeinfo_structure() {
        let ni = build_nodeinfo(10);

        assert_eq!(ni.version, "2.1");
        assert_eq!(ni.software.name, "sovereign");
        assert_eq!(ni.usage.local_posts, 10);
        assert_eq!(ni.usage.users.total, 1);
    }

    #[test]
    fn test_did_document_structure() {
        let doc = build_did_web_document("example.com", "zDnaertest123");

        // Convert to JSON to test structure
        let json = serde_json::to_value(&doc).unwrap();
        assert!(json["id"].as_str().unwrap().contains("did:web:example.com"));
        assert!(json["verificationMethod"].is_array());
        assert!(json["service"].is_array());
    }

    #[test]
    fn test_webfinger_aliases() {
        let wf = build_webfinger("user", "domain.com");

        assert!(!wf.aliases.is_empty());
        assert!(wf.aliases[0].contains("domain.com"));
    }
}
