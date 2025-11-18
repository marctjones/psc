use worker::*;
use sovereign_core::protocol::{build_actor, build_collection};
use sovereign_core::types::activitypub::activitypub_context;

use super::{activitypub_response, error_response, get_domain, get_username};
use crate::crypto::signatures;

/// Get actor document
pub async fn actor(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let username_param = ctx.param("username")
        .ok_or_else(|| Error::RustError("Missing username".to_string()))?;

    let domain = get_domain(&ctx.env, &req);
    let username = get_username(&ctx.env);

    // Single-user server - only respond for our user
    if username_param != &username {
        return Response::error("User not found", 404);
    }

    // TODO: Get actual profile data from D1
    let display_name = Some("Sovereign User");
    let summary = Some("A self-hosted ActivityPub account");

    // TODO: Get actual public key from storage
    let public_key_pem = "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...\n-----END PUBLIC KEY-----";

    let actor = build_actor(&username, &domain, display_name, summary, public_key_pem);

    activitypub_response(&actor)
}

/// Inbox GET (not typically used, but some implementations check it)
pub async fn inbox_get(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Response::error("Method not allowed", 405)
}

/// Inbox POST (receive activities from other servers)
pub async fn inbox_post(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let username_param = ctx.param("username")
        .ok_or_else(|| Error::RustError("Missing username".to_string()))?;

    let username = get_username(&ctx.env);

    if username_param != &username {
        return Response::error("User not found", 404);
    }

    // Get the activity body
    let body = req.text().await?;
    let activity: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| Error::RustError(format!("Invalid JSON: {}", e)))?;

    // TODO: Verify HTTP signature
    // let signature_valid = signatures::verify_request(&req, &ctx.env).await?;
    // if !signature_valid {
    //     return error_response(401, "Invalid signature");
    // }

    // Get activity type
    let activity_type = activity.get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("Unknown");

    let actor = activity.get("actor")
        .and_then(|a| a.as_str())
        .unwrap_or("unknown");

    console_log!("Received {} activity from {}", activity_type, actor);

    // Process the activity
    match activity_type {
        "Follow" => {
            // TODO: Store follow request in D1
            // TODO: Auto-accept and send Accept activity back
            console_log!("Follow request from {}", actor);
        }
        "Undo" => {
            // Check what's being undone
            if let Some(object) = activity.get("object") {
                let object_type = object.get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("Unknown");

                match object_type {
                    "Follow" => {
                        // TODO: Remove follower from D1
                        console_log!("Unfollow from {}", actor);
                    }
                    "Like" => {
                        // TODO: Remove like from D1
                        console_log!("Unlike from {}", actor);
                    }
                    _ => {}
                }
            }
        }
        "Like" => {
            // TODO: Store like in D1
            console_log!("Like from {}", actor);
        }
        "Announce" => {
            // TODO: Store boost in D1
            console_log!("Boost from {}", actor);
        }
        "Create" => {
            // Check if it's a reply to us
            if let Some(object) = activity.get("object") {
                if let Some(in_reply_to) = object.get("inReplyTo") {
                    // TODO: Store reply notification
                    console_log!("Reply from {}", actor);
                }
            }
        }
        "Delete" => {
            // TODO: Remove content from D1
            console_log!("Delete from {}", actor);
        }
        "Accept" => {
            // Our follow request was accepted
            // TODO: Update follow status in D1
            console_log!("Follow accepted by {}", actor);
        }
        "Reject" => {
            // Our follow request was rejected
            // TODO: Update follow status in D1
            console_log!("Follow rejected by {}", actor);
        }
        _ => {
            console_log!("Unhandled activity type: {}", activity_type);
        }
    }

    // Return 202 Accepted
    Response::ok("").map(|r| r.with_status(202))
}

/// Outbox (list of user's activities)
pub async fn outbox(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let username_param = ctx.param("username")
        .ok_or_else(|| Error::RustError("Missing username".to_string()))?;

    let domain = get_domain(&ctx.env, &req);
    let username = get_username(&ctx.env);

    if username_param != &username {
        return Response::error("User not found", 404);
    }

    // TODO: Get actual post count from D1
    let total_items = 0;

    let outbox_url = format!("https://{}/users/{}/outbox", domain, username);
    let collection = build_collection(&outbox_url, total_items, None);

    activitypub_response(&collection)
}

/// Followers collection
pub async fn followers(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let username_param = ctx.param("username")
        .ok_or_else(|| Error::RustError("Missing username".to_string()))?;

    let domain = get_domain(&ctx.env, &req);
    let username = get_username(&ctx.env);

    if username_param != &username {
        return Response::error("User not found", 404);
    }

    // TODO: Get actual follower count from D1
    let total_items = 0;

    let followers_url = format!("https://{}/users/{}/followers", domain, username);
    let collection = build_collection(&followers_url, total_items, None);

    activitypub_response(&collection)
}

/// Following collection
pub async fn following(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let username_param = ctx.param("username")
        .ok_or_else(|| Error::RustError("Missing username".to_string()))?;

    let domain = get_domain(&ctx.env, &req);
    let username = get_username(&ctx.env);

    if username_param != &username {
        return Response::error("User not found", 404);
    }

    // TODO: Get actual following count from D1
    let total_items = 0;

    let following_url = format!("https://{}/users/{}/following", domain, username);
    let collection = build_collection(&following_url, total_items, None);

    activitypub_response(&collection)
}
