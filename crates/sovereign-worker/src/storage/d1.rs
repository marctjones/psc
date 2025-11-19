use worker::*;
use worker::d1::D1Database;

/// Database operations for D1

/// Get the D1 database binding
pub fn get_db(env: &Env) -> Result<D1Database> {
    env.d1("DB")
}

/// Initialize the database schema
pub async fn initialize_schema(_db: &D1Database) -> Result<()> {
    // This would normally be done via migrations
    // But we can also run it programmatically

    Ok(())
}

/// Store a post
pub async fn create_post(
    db: &D1Database,
    uri: &str,
    protocol: &str,
    author_did: &str,
    content: &str,
    reply_to: Option<&str>,
    raw_data: &serde_json::Value,
) -> Result<()> {
    let stmt = db.prepare(
        "INSERT INTO posts (uri, protocol, author_did, content, reply_to, created_at, raw_data)
         VALUES (?, ?, ?, ?, ?, datetime('now'), ?)"
    );

    stmt.bind(&[
        uri.into(),
        protocol.into(),
        author_did.into(),
        content.into(),
        reply_to.map(|s| s.to_string()).unwrap_or_default().into(),
        raw_data.to_string().into(),
    ])?
    .run()
    .await?;

    Ok(())
}

/// Get a post by URI
pub async fn get_post(db: &D1Database, uri: &str) -> Result<Option<serde_json::Value>> {
    let stmt = db.prepare("SELECT * FROM posts WHERE uri = ?");

    let result = stmt.bind(&[uri.into()])?
        .first::<serde_json::Value>(None)
        .await?;

    Ok(result)
}

/// List posts for timeline
pub async fn list_posts(
    db: &D1Database,
    limit: usize,
    offset: usize,
) -> Result<Vec<serde_json::Value>> {
    let stmt = db.prepare(
        "SELECT * FROM posts ORDER BY created_at DESC LIMIT ? OFFSET ?"
    );

    let results = stmt.bind(&[
        (limit as i64).into(),
        (offset as i64).into(),
    ])?
    .all()
    .await?;

    Ok(results.results()?)
}

/// Store a follow relationship
pub async fn create_follow(
    db: &D1Database,
    follower: &str,
    following: &str,
    protocol: &str,
    accepted: bool,
) -> Result<()> {
    let stmt = db.prepare(
        "INSERT OR REPLACE INTO follows (follower, following, protocol, accepted, created_at)
         VALUES (?, ?, ?, ?, datetime('now'))"
    );

    stmt.bind(&[
        follower.into(),
        following.into(),
        protocol.into(),
        accepted.into(),
    ])?
    .run()
    .await?;

    Ok(())
}

/// Delete a follow relationship
pub async fn delete_follow(
    db: &D1Database,
    follower: &str,
    following: &str,
) -> Result<()> {
    let stmt = db.prepare("DELETE FROM follows WHERE follower = ? AND following = ?");

    stmt.bind(&[follower.into(), following.into()])?
        .run()
        .await?;

    Ok(())
}

/// Get followers
pub async fn get_followers(
    db: &D1Database,
    user: &str,
    limit: usize,
) -> Result<Vec<String>> {
    let stmt = db.prepare(
        "SELECT follower FROM follows WHERE following = ? AND accepted = 1 LIMIT ?"
    );

    let results = stmt.bind(&[user.into(), (limit as i64).into()])?
        .all()
        .await?;

    let followers: Vec<String> = results.results::<serde_json::Value>()?
        .iter()
        .filter_map(|v| v.get("follower").and_then(|f| f.as_str()).map(|s| s.to_string()))
        .collect();

    Ok(followers)
}

/// Get following
pub async fn get_following(
    db: &D1Database,
    user: &str,
    limit: usize,
) -> Result<Vec<String>> {
    let stmt = db.prepare(
        "SELECT following FROM follows WHERE follower = ? AND accepted = 1 LIMIT ?"
    );

    let results = stmt.bind(&[user.into(), (limit as i64).into()])?
        .all()
        .await?;

    let following: Vec<String> = results.results::<serde_json::Value>()?
        .iter()
        .filter_map(|v| v.get("following").and_then(|f| f.as_str()).map(|s| s.to_string()))
        .collect();

    Ok(following)
}

/// Count posts
pub async fn count_posts(db: &D1Database) -> Result<u64> {
    let stmt = db.prepare("SELECT COUNT(*) as count FROM posts");

    let result = stmt.first::<serde_json::Value>(None).await?;

    Ok(result
        .and_then(|v| v.get("count").and_then(|c| c.as_u64()))
        .unwrap_or(0))
}

/// Count followers
pub async fn count_followers(db: &D1Database, user: &str) -> Result<u64> {
    let stmt = db.prepare(
        "SELECT COUNT(*) as count FROM follows WHERE following = ? AND accepted = 1"
    );

    let result = stmt.bind(&[user.into()])?
        .first::<serde_json::Value>(None)
        .await?;

    Ok(result
        .and_then(|v| v.get("count").and_then(|c| c.as_u64()))
        .unwrap_or(0))
}

/// Count following
pub async fn count_following(db: &D1Database, user: &str) -> Result<u64> {
    let stmt = db.prepare(
        "SELECT COUNT(*) as count FROM follows WHERE follower = ? AND accepted = 1"
    );

    let result = stmt.bind(&[user.into()])?
        .first::<serde_json::Value>(None)
        .await?;

    Ok(result
        .and_then(|v| v.get("count").and_then(|c| c.as_u64()))
        .unwrap_or(0))
}

/// Add to delivery queue
pub async fn queue_delivery(
    db: &D1Database,
    activity: &serde_json::Value,
    target_inbox: &str,
) -> Result<()> {
    let stmt = db.prepare(
        "INSERT INTO delivery_queue (activity, target_inbox, attempts, next_retry, created_at)
         VALUES (?, ?, 0, datetime('now'), datetime('now'))"
    );

    stmt.bind(&[activity.to_string().into(), target_inbox.into()])?
        .run()
        .await?;

    Ok(())
}

/// Get pending deliveries
pub async fn get_pending_deliveries(
    db: &D1Database,
    limit: usize,
) -> Result<Vec<serde_json::Value>> {
    let stmt = db.prepare(
        "SELECT * FROM delivery_queue
         WHERE next_retry <= datetime('now') AND attempts < 5
         ORDER BY created_at ASC LIMIT ?"
    );

    let results = stmt.bind(&[(limit as i64).into()])?
        .all()
        .await?;

    Ok(results.results()?)
}

// OAuth operations

/// Create an OAuth client
pub async fn create_oauth_client(
    db: &D1Database,
    client_id: &str,
    client_secret: &str,
    name: &str,
    redirect_uris: &[String],
    grant_types: &[String],
    scope: &str,
) -> Result<()> {
    let stmt = db.prepare(
        "INSERT INTO oauth_clients (client_id, client_secret, name, redirect_uris, grant_types, scope, created_at)
         VALUES (?, ?, ?, ?, ?, ?, datetime('now'))"
    );

    stmt.bind(&[
        client_id.into(),
        client_secret.into(),
        name.into(),
        serde_json::to_string(redirect_uris).unwrap_or_default().into(),
        serde_json::to_string(grant_types).unwrap_or_default().into(),
        scope.into(),
    ])?
    .run()
    .await?;

    Ok(())
}

/// Get an OAuth client by ID
pub async fn get_oauth_client(db: &D1Database, client_id: &str) -> Result<Option<serde_json::Value>> {
    let stmt = db.prepare("SELECT * FROM oauth_clients WHERE client_id = ?");

    let result = stmt.bind(&[client_id.into()])?
        .first::<serde_json::Value>(None)
        .await?;

    Ok(result)
}

/// List all OAuth clients
pub async fn list_oauth_clients(db: &D1Database) -> Result<Vec<serde_json::Value>> {
    let stmt = db.prepare("SELECT client_id, name, redirect_uris, grant_types, scope, created_at FROM oauth_clients");

    let results = stmt.all().await?;
    Ok(results.results()?)
}

/// Delete an OAuth client
pub async fn delete_oauth_client(db: &D1Database, client_id: &str) -> Result<()> {
    let stmt = db.prepare("DELETE FROM oauth_clients WHERE client_id = ?");
    stmt.bind(&[client_id.into()])?.run().await?;
    Ok(())
}

/// Create device authorization
pub async fn create_device_auth(
    db: &D1Database,
    device_code: &str,
    user_code: &str,
    client_id: &str,
    scope: &str,
) -> Result<()> {
    let stmt = db.prepare(
        "INSERT INTO device_authorizations (device_code, user_code, client_id, scope, status, created_at, expires_at)
         VALUES (?, ?, ?, ?, 'pending', datetime('now'), datetime('now', '+10 minutes'))"
    );

    stmt.bind(&[
        device_code.into(),
        user_code.into(),
        client_id.into(),
        scope.into(),
    ])?
    .run()
    .await?;

    Ok(())
}

/// Get device authorization status
pub async fn get_device_auth_status(db: &D1Database, device_code: &str) -> Result<String> {
    let stmt = db.prepare(
        "SELECT CASE
            WHEN expires_at < datetime('now') THEN 'expired'
            ELSE status
         END as status
         FROM device_authorizations WHERE device_code = ?"
    );

    let result = stmt.bind(&[device_code.into()])?
        .first::<serde_json::Value>(None)
        .await?;

    Ok(result
        .and_then(|v| v.get("status").and_then(|s| s.as_str()).map(|s| s.to_string()))
        .unwrap_or_else(|| "invalid".to_string()))
}

/// Get device auth info (client_id and scope)
pub async fn get_device_auth_info(db: &D1Database, device_code: &str) -> Result<(String, String)> {
    let stmt = db.prepare("SELECT client_id, scope FROM device_authorizations WHERE device_code = ?");

    let result = stmt.bind(&[device_code.into()])?
        .first::<serde_json::Value>(None)
        .await?;

    let client_id = result
        .as_ref()
        .and_then(|v| v.get("client_id").and_then(|s| s.as_str()))
        .unwrap_or("unknown")
        .to_string();

    let scope = result
        .and_then(|v| v.get("scope").and_then(|s| s.as_str()).map(|s| s.to_string()))
        .unwrap_or_else(|| "read".to_string());

    Ok((client_id, scope))
}

/// Authorize a device by user code
pub async fn authorize_device(db: &D1Database, user_code: &str) -> Result<bool> {
    let stmt = db.prepare(
        "UPDATE device_authorizations SET status = 'authorized'
         WHERE user_code = ? AND status = 'pending' AND expires_at > datetime('now')"
    );

    let _result = stmt.bind(&[user_code.into()])?.run().await?;

    // Check if device was authorized by querying
    let check_stmt = db.prepare("SELECT 1 FROM device_authorizations WHERE user_code = ? AND status = 'authorized'");
    let check_result = check_stmt.bind(&[user_code.into()])?
        .first::<serde_json::Value>(None)
        .await?;

    Ok(check_result.is_some())
}

/// Mark device auth as complete
pub async fn complete_device_auth(db: &D1Database, device_code: &str) -> Result<()> {
    let stmt = db.prepare("UPDATE device_authorizations SET status = 'complete' WHERE device_code = ?");
    stmt.bind(&[device_code.into()])?.run().await?;
    Ok(())
}

/// Create access token
pub async fn create_access_token(
    db: &D1Database,
    token: &str,
    username: &str,
    client_id: &str,
    scope: &str,
    expires_in: u32,
) -> Result<()> {
    let stmt = db.prepare(
        "INSERT INTO access_tokens (token, username, client_id, scope, created_at, expires_at)
         VALUES (?, ?, ?, ?, datetime('now'), datetime('now', '+' || ? || ' seconds'))"
    );

    stmt.bind(&[
        token.into(),
        username.into(),
        client_id.into(),
        scope.into(),
        expires_in.to_string().into(),
    ])?
    .run()
    .await?;

    Ok(())
}

/// Get access token info
pub async fn get_access_token_info(
    db: &D1Database,
    token: &str,
) -> Result<Option<(String, String, String, String)>> {
    let stmt = db.prepare(
        "SELECT username, client_id, scope, expires_at
         FROM access_tokens
         WHERE token = ? AND expires_at > datetime('now')"
    );

    let result = stmt.bind(&[token.into()])?
        .first::<serde_json::Value>(None)
        .await?;

    Ok(result.map(|v| {
        (
            v.get("username").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            v.get("client_id").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            v.get("scope").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            v.get("expires_at").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        )
    }))
}

/// Revoke access token
pub async fn revoke_access_token(db: &D1Database, token: &str) -> Result<()> {
    let stmt = db.prepare("DELETE FROM access_tokens WHERE token = ?");
    stmt.bind(&[token.into()])?.run().await?;
    Ok(())
}

/// Create refresh token
pub async fn create_refresh_token(
    db: &D1Database,
    token: &str,
    access_token: &str,
    username: &str,
    client_id: &str,
) -> Result<()> {
    let stmt = db.prepare(
        "INSERT INTO refresh_tokens (token, access_token, username, client_id, created_at)
         VALUES (?, ?, ?, ?, datetime('now'))"
    );

    stmt.bind(&[
        token.into(),
        access_token.into(),
        username.into(),
        client_id.into(),
    ])?
    .run()
    .await?;

    Ok(())
}

/// Validate refresh token and get associated data
pub async fn validate_refresh_token(
    db: &D1Database,
    token: &str,
) -> Result<Option<(String, String, String)>> {
    let stmt = db.prepare(
        "SELECT r.username, r.client_id, COALESCE(a.scope, 'read') as scope
         FROM refresh_tokens r
         LEFT JOIN access_tokens a ON r.access_token = a.token
         WHERE r.token = ?"
    );

    let result = stmt.bind(&[token.into()])?
        .first::<serde_json::Value>(None)
        .await?;

    Ok(result.map(|v| {
        (
            v.get("username").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            v.get("client_id").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            v.get("scope").and_then(|s| s.as_str()).unwrap_or("read").to_string(),
        )
    }))
}

/// Revoke refresh token
pub async fn revoke_refresh_token(db: &D1Database, token: &str) -> Result<()> {
    let stmt = db.prepare("DELETE FROM refresh_tokens WHERE token = ?");
    stmt.bind(&[token.into()])?.run().await?;
    Ok(())
}
