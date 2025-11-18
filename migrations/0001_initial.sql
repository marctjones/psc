-- Initial schema for Sovereign
-- This migration creates all core tables for ActivityPub and AT Protocol support

-- Identity table (single user, but flexible for future)
CREATE TABLE IF NOT EXISTS identity (
    did TEXT PRIMARY KEY,
    handle TEXT UNIQUE NOT NULL,
    display_name TEXT,
    bio TEXT,
    avatar_cid TEXT,
    private_key_encrypted TEXT,
    public_key TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Posts table (stores posts from both protocols)
CREATE TABLE IF NOT EXISTS posts (
    uri TEXT PRIMARY KEY,
    protocol TEXT NOT NULL CHECK (protocol IN ('activitypub', 'atproto')),
    author_did TEXT NOT NULL,
    content TEXT NOT NULL,
    reply_to TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    raw_data JSON
);

CREATE INDEX IF NOT EXISTS idx_posts_author ON posts(author_did);
CREATE INDEX IF NOT EXISTS idx_posts_created ON posts(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_posts_protocol ON posts(protocol);

-- Follows table (social graph)
CREATE TABLE IF NOT EXISTS follows (
    follower TEXT NOT NULL,
    following TEXT NOT NULL,
    protocol TEXT NOT NULL CHECK (protocol IN ('activitypub', 'atproto')),
    accepted BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (follower, following)
);

CREATE INDEX IF NOT EXISTS idx_follows_follower ON follows(follower);
CREATE INDEX IF NOT EXISTS idx_follows_following ON follows(following);

-- Likes table
CREATE TABLE IF NOT EXISTS likes (
    uri TEXT PRIMARY KEY,
    post_uri TEXT NOT NULL,
    actor TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_likes_post ON likes(post_uri);
CREATE INDEX IF NOT EXISTS idx_likes_actor ON likes(actor);

-- Reposts/Announces table
CREATE TABLE IF NOT EXISTS reposts (
    uri TEXT PRIMARY KEY,
    post_uri TEXT NOT NULL,
    actor TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_reposts_post ON reposts(post_uri);
CREATE INDEX IF NOT EXISTS idx_reposts_actor ON reposts(actor);

-- Notifications table
CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL CHECK (type IN ('follow', 'like', 'repost', 'reply', 'mention')),
    actor TEXT NOT NULL,
    subject TEXT,
    read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_notifications_read ON notifications(read);
CREATE INDEX IF NOT EXISTS idx_notifications_created ON notifications(created_at DESC);

-- Delivery queue for ActivityPub federation
CREATE TABLE IF NOT EXISTS delivery_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity JSON NOT NULL,
    target_inbox TEXT NOT NULL,
    attempts INTEGER DEFAULT 0,
    next_retry TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_delivery_queue_retry ON delivery_queue(next_retry);
CREATE INDEX IF NOT EXISTS idx_delivery_queue_attempts ON delivery_queue(attempts);

-- Blocks table
CREATE TABLE IF NOT EXISTS blocks (
    blocker TEXT NOT NULL,
    blocked TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (blocker, blocked)
);

-- Mutes table
CREATE TABLE IF NOT EXISTS mutes (
    muter TEXT NOT NULL,
    muted TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (muter, muted)
);

-- Sessions table (for AT Protocol)
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    did TEXT NOT NULL,
    refresh_token_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_did ON sessions(did);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at);

-- Media blobs metadata
CREATE TABLE IF NOT EXISTS blobs (
    cid TEXT PRIMARY KEY,
    mime_type TEXT NOT NULL,
    size INTEGER NOT NULL,
    r2_key TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Actor cache (for remote ActivityPub actors)
CREATE TABLE IF NOT EXISTS actor_cache (
    actor_url TEXT PRIMARY KEY,
    actor_data JSON NOT NULL,
    public_key_pem TEXT,
    inbox_url TEXT,
    fetched_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_actor_cache_fetched ON actor_cache(fetched_at);
