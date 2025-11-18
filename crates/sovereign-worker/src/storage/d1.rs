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
