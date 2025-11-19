-- Sovereign Worker D1 Database Schema

-- Posts table for ActivityPub/AT Protocol content
CREATE TABLE IF NOT EXISTS posts (
    uri TEXT PRIMARY KEY,
    protocol TEXT NOT NULL,
    author_did TEXT NOT NULL,
    content TEXT NOT NULL,
    reply_to TEXT,
    created_at TEXT NOT NULL,
    raw_data TEXT
);

CREATE INDEX IF NOT EXISTS idx_posts_author ON posts(author_did);
CREATE INDEX IF NOT EXISTS idx_posts_created ON posts(created_at);

-- Follows table for relationships
CREATE TABLE IF NOT EXISTS follows (
    follower TEXT NOT NULL,
    following TEXT NOT NULL,
    protocol TEXT NOT NULL,
    accepted INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    PRIMARY KEY (follower, following)
);

CREATE INDEX IF NOT EXISTS idx_follows_follower ON follows(follower);
CREATE INDEX IF NOT EXISTS idx_follows_following ON follows(following);

-- Delivery queue for ActivityPub outbound
CREATE TABLE IF NOT EXISTS delivery_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity TEXT NOT NULL,
    target_inbox TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    next_retry TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_delivery_queue_retry ON delivery_queue(next_retry);

-- OAuth clients
CREATE TABLE IF NOT EXISTS oauth_clients (
    client_id TEXT PRIMARY KEY,
    client_secret TEXT,
    name TEXT NOT NULL,
    redirect_uris TEXT NOT NULL, -- JSON array
    grant_types TEXT NOT NULL,   -- JSON array
    scope TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- Device authorizations (RFC 8628)
CREATE TABLE IF NOT EXISTS device_authorizations (
    device_code TEXT PRIMARY KEY,
    user_code TEXT NOT NULL UNIQUE,
    client_id TEXT NOT NULL,
    scope TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending', -- pending, authorized, complete, expired
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id)
);

CREATE INDEX IF NOT EXISTS idx_device_auth_user_code ON device_authorizations(user_code);
CREATE INDEX IF NOT EXISTS idx_device_auth_status ON device_authorizations(status);

-- Access tokens
CREATE TABLE IF NOT EXISTS access_tokens (
    token TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    client_id TEXT NOT NULL,
    scope TEXT NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id)
);

CREATE INDEX IF NOT EXISTS idx_access_tokens_expires ON access_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_access_tokens_username ON access_tokens(username);

-- Refresh tokens
CREATE TABLE IF NOT EXISTS refresh_tokens (
    token TEXT PRIMARY KEY,
    access_token TEXT NOT NULL,
    username TEXT NOT NULL,
    client_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (client_id) REFERENCES oauth_clients(client_id)
);

CREATE INDEX IF NOT EXISTS idx_refresh_tokens_username ON refresh_tokens(username);

-- Insert default CLI client for device auth
INSERT OR IGNORE INTO oauth_clients (client_id, client_secret, name, redirect_uris, grant_types, scope, created_at)
VALUES (
    'sovereign-cli',
    NULL,
    'Sovereign CLI',
    '[]',
    '["urn:ietf:params:oauth:grant-type:device_code", "refresh_token"]',
    'read write follow admin',
    datetime('now')
);
