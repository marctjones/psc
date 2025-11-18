use worker::*;
use sovereign_core::protocol::build_did_web_document;

use super::{json_response, error_response, get_domain, get_username};

/// Describe server endpoint
pub async fn describe_server(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let domain = get_domain(&ctx.env, &req);

    let response = serde_json::json!({
        "availableUserDomains": [domain],
        "inviteCodeRequired": false,
        "links": {
            "privacyPolicy": format!("https://{}/privacy", domain),
            "termsOfService": format!("https://{}/terms", domain)
        }
    });

    json_response(&response)
}

/// Create session (login)
pub async fn create_session(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let body = req.text().await?;
    let params: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| Error::RustError(format!("Invalid JSON: {}", e)))?;

    let identifier = params.get("identifier")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::RustError("Missing identifier".to_string()))?;

    let password = params.get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::RustError("Missing password".to_string()))?;

    let domain = get_domain(&ctx.env, &req);
    let username = get_username(&ctx.env);

    // TODO: Verify password against stored hash

    let did = format!("did:web:{}", domain);
    let handle = format!("{}@{}", username, domain);

    // TODO: Generate actual JWTs
    let access_jwt = "eyJ..."; // Placeholder
    let refresh_jwt = "eyJ..."; // Placeholder

    let response = serde_json::json!({
        "did": did,
        "handle": handle,
        "accessJwt": access_jwt,
        "refreshJwt": refresh_jwt
    });

    json_response(&response)
}

/// Refresh session
pub async fn refresh_session(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // TODO: Verify refresh token and issue new access token

    let domain = get_domain(&ctx.env, &req);
    let username = get_username(&ctx.env);

    let did = format!("did:web:{}", domain);
    let handle = format!("{}@{}", username, domain);

    let response = serde_json::json!({
        "did": did,
        "handle": handle,
        "accessJwt": "eyJ...",
        "refreshJwt": "eyJ..."
    });

    json_response(&response)
}

/// Create record
pub async fn create_record(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // TODO: Verify authentication

    let body = req.text().await?;
    let params: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| Error::RustError(format!("Invalid JSON: {}", e)))?;

    let collection = params.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::RustError("Missing collection".to_string()))?;

    let record = params.get("record")
        .ok_or_else(|| Error::RustError("Missing record".to_string()))?;

    let domain = get_domain(&ctx.env, &req);
    let did = format!("did:web:{}", domain);

    // Generate rkey
    let rkey = sovereign_core::protocol::generate_rkey();

    // TODO: Store record in D1
    // TODO: Calculate actual CID

    let uri = format!("at://{}/{}/{}", did, collection, rkey);
    let cid = "bafyreiabc123..."; // Placeholder

    console_log!("Created record: {}", uri);

    let response = serde_json::json!({
        "uri": uri,
        "cid": cid
    });

    json_response(&response)
}

/// Get record
pub async fn get_record(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let url = req.url()?;

    let repo = url.query_pairs()
        .find(|(k, _)| k == "repo")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| Error::RustError("Missing repo parameter".to_string()))?;

    let collection = url.query_pairs()
        .find(|(k, _)| k == "collection")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| Error::RustError("Missing collection parameter".to_string()))?;

    let rkey = url.query_pairs()
        .find(|(k, _)| k == "rkey")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| Error::RustError("Missing rkey parameter".to_string()))?;

    // TODO: Fetch record from D1

    // Return 404 for now
    error_response(404, "Record not found")
}

/// Resolve handle
pub async fn resolve_handle(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let url = req.url()?;

    let handle = url.query_pairs()
        .find(|(k, _)| k == "handle")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| Error::RustError("Missing handle parameter".to_string()))?;

    let domain = get_domain(&ctx.env, &req);
    let username = get_username(&ctx.env);

    // Check if handle matches our user
    let our_handle = format!("{}@{}", username, domain);
    let our_handle_alt = domain.clone(); // Handle could just be the domain

    if handle != our_handle && handle != our_handle_alt && handle != username {
        return error_response(404, "Handle not found");
    }

    let did = format!("did:web:{}", domain);

    let response = serde_json::json!({
        "did": did
    });

    json_response(&response)
}
