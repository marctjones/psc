# Project Sovereign Core (V2.2)

## Consolidated Handoff Prompt (Final Version)

We are beginning the creation of "Project Sovereign," a self-hosted digital ecosystem using only **commodity cloud services** (Cloudflare, AWS S3) and focusing entirely on **user freedom, simplicity, and low-cost maintenance**. Our goal is to make it easy for an individual to deploy their own digital presence.

We are specifically targeting services that currently trap users into proprietary single sign-on (SSO) systems (Google, Facebook, Amazon, Apple) and hosted social media. We are **not interested** in recreating existing, well-tested open-source tools like full email servers.

The current focus on the single-user ActivityPub/Bluesky server is **Step One**—a sample implementation to validate the architecture. The **long-term goal** is to brainstorm and prioritize other simple, cheap, and easily deployable services that enhance personal digital sovereignty.

---

## 1. Architectural Mandate & Stack

* **Goal:** A CLI-first, single-user system with "no-installer" portable binaries. We are **actively not interested in trying to make things scale**.
* **Core Logic (Appliance):** C#/.NET 9+ (using NativeAOT for main CLI, Server, and UI).
* **Security Layer (Integrity):** Rust (via FFI) for low-level cryptography only.
* **Extension Layer (Flexibility):** Python for high-level scripting, demos, and automation.

---

## 2. Python Isolation Mandate (CRITICAL)

Python scripts must **ALWAYS** be developed and executed within **isolated environments** (e.g., venv, pipx). Dependencies must be minimized and limited to stable, popular libraries (e.g., requests, click). Python must not populate the host system.

---

## 3. Required Functionality & Interactions

* **Identity Lifecycle:** The C# CLI must include `sovereign identity setup <domain>` and `cleanup` commands that automate DNS records (A, TXT) via the Cloudflare API.
* **Protocol Implementation:** AT Protocol uses C# libraries. ActivityPub **MUST** call the Rust bridge for all cryptographic signing operations.

---

## 4. Explicit Exclusions (DO NOT Implement)

* Do not use **Python for the core server/CLI/UI logic.**
* Do not use **Kubernetes, Docker Swarm, or complex scaling/load-balancing logic.**
* Do not attempt to implement **custom low-level crypto or signatures** in C# or Python; delegate this to the Rust bridge.
* Do not include **SMTP/Email hosting.**

---

## 5. Future Trajectory (The Next Steps)

Once the core server is stable, the project's focus will shift to **brainstorming and prioritizing** future components that are **simple to build, cheap and easy to host**, and align with personal interest and user freedom (e.g., decentralized contacts/calendar, private photo archive, lightweight personal search).

---

## 6. Bluesky / AT Protocol Implementation

### 6.1 Overview

The AT Protocol (Authenticated Transfer Protocol) powers Bluesky. Our implementation includes both a **Personal Data Server (PDS)** for hosting your identity and data, and a **client** for interacting with the Bluesky network.

### 6.2 Server Components (PDS)

#### Identity & Authentication
* **DID Document Hosting:** Serve `did:web` or `did:plc` documents at `/.well-known/did.json`
* **Handle Resolution:** DNS TXT record `_atproto.<domain>` pointing to DID
* **JWT Authentication:** Issue and validate access/refresh tokens
* **App Passwords:** Support for third-party client authentication

#### Data Repository
* **Repository Structure:** Merkle Search Tree (MST) for content-addressable storage
* **Record Types:** Posts, likes, reposts, follows, blocks, profile
* **Blob Storage:** Images and media stored in S3, referenced by CID
* **Repo Sync:** Export/import CAR files for data portability

#### Lexicon Endpoints (Required)
```
com.atproto.server.createSession
com.atproto.server.refreshSession
com.atproto.server.deleteSession
com.atproto.repo.createRecord
com.atproto.repo.deleteRecord
com.atproto.repo.getRecord
com.atproto.repo.listRecords
com.atproto.repo.putRecord
com.atproto.sync.getRepo
com.atproto.sync.getBlob
com.atproto.identity.resolveHandle
```

#### App View Endpoints (Required for Feed)
```
app.bsky.feed.getTimeline
app.bsky.feed.getAuthorFeed
app.bsky.feed.getPostThread
app.bsky.feed.getPosts
app.bsky.feed.getLikes
app.bsky.feed.getRepostedBy
app.bsky.actor.getProfile
app.bsky.actor.getProfiles
app.bsky.actor.searchActors
app.bsky.graph.getFollowers
app.bsky.graph.getFollows
app.bsky.graph.getBlocks
app.bsky.graph.getMutes
app.bsky.notification.listNotifications
app.bsky.notification.updateSeen
```

### 6.3 Client Features (CLI Commands)

#### Authentication
```bash
sovereign bsky login <handle> [--app-password]
sovereign bsky logout
sovereign bsky whoami
```

#### Posts & Content
```bash
sovereign bsky post "<text>" [--image <path>] [--alt "<alt-text>"]
sovereign bsky reply <post-uri> "<text>"
sovereign bsky quote <post-uri> "<text>"
sovereign bsky delete <post-uri>
sovereign bsky thread <post-uri>
```

#### Timeline & Feeds
```bash
sovereign bsky timeline [--limit <n>]
sovereign bsky feed <handle> [--limit <n>]
sovereign bsky notifications [--limit <n>]
sovereign bsky search "<query>" [--limit <n>]
```

#### Social Graph
```bash
sovereign bsky follow <handle>
sovereign bsky unfollow <handle>
sovereign bsky followers [<handle>] [--limit <n>]
sovereign bsky following [<handle>] [--limit <n>]
sovereign bsky block <handle>
sovereign bsky unblock <handle>
sovereign bsky mute <handle>
sovereign bsky unmute <handle>
```

#### Engagement
```bash
sovereign bsky like <post-uri>
sovereign bsky unlike <post-uri>
sovereign bsky repost <post-uri>
sovereign bsky unrepost <post-uri>
```

#### Profile Management
```bash
sovereign bsky profile [<handle>]
sovereign bsky profile update --name "<name>" --bio "<bio>" --avatar <path>
```

### 6.4 Federation with Bluesky Network

* **Relay Connection:** Subscribe to `bsky.network` firehose for global feed access
* **AppView Delegation:** Use `api.bsky.app` for aggregated views (likes, reposts, followers counts)
* **PDS Registration:** Register with Bluesky PLC directory for `did:plc` identifiers
* **Alternative:** Use `did:web` for fully self-sovereign identity (no PLC dependency)

---

## 7. ActivityPub / Mastodon Implementation

### 7.1 Overview

ActivityPub is the W3C standard powering Mastodon and the Fediverse. Our implementation includes a **single-user server** that can federate with any ActivityPub-compatible instance.

### 7.2 Server Components

#### Identity & Discovery
* **WebFinger:** `/.well-known/webfinger?resource=acct:user@domain`
* **Actor Document:** JSON-LD actor at `/users/<username>`
* **NodeInfo:** Server metadata at `/.well-known/nodeinfo`

#### Actor Endpoints
```
GET  /users/<username>           # Actor document
GET  /users/<username>/inbox     # Inbox (POST for federation)
POST /users/<username>/inbox     # Receive activities
GET  /users/<username>/outbox    # Public activities
GET  /users/<username>/followers # Followers collection
GET  /users/<username>/following # Following collection
GET  /users/<username>/liked     # Liked posts
```

#### HTTP Signatures (Rust Bridge - CRITICAL)
All federated requests **MUST** be signed using HTTP Signatures (RFC 9421):
* **Algorithm:** RSA-SHA256 or Ed25519
* **Headers Signed:** `(request-target)`, `host`, `date`, `digest`
* **Key Management:** RSA/Ed25519 keypair stored securely
* **Signature Verification:** Validate incoming requests from remote servers

#### Activity Types (Send & Receive)
```
Create    # New posts
Update    # Edit posts
Delete    # Remove content
Follow    # Follow request
Accept    # Accept follow
Reject    # Reject follow
Undo      # Undo like/follow/boost
Like      # Favorite a post
Announce  # Boost/reblog
Block     # Block user
```

#### Object Types
```
Note      # Standard post
Article   # Long-form content
Image     # Image attachment
Document  # File attachment
Question  # Poll
```

### 7.3 Client Features (CLI Commands)

#### Authentication
```bash
sovereign fedi login
sovereign fedi logout
sovereign fedi whoami
```

#### Posts & Content
```bash
sovereign fedi post "<text>" [--cw "<content-warning>"] [--image <path>]
sovereign fedi reply <post-url> "<text>"
sovereign fedi delete <post-url>
sovereign fedi thread <post-url>
sovereign fedi edit <post-url> "<new-text>"
```

#### Timelines
```bash
sovereign fedi timeline [--limit <n>]              # Home timeline
sovereign fedi local [--limit <n>]                 # Local timeline (own posts)
sovereign fedi notifications [--limit <n>]
sovereign fedi search "<query>" [--limit <n>]
```

#### Social Graph (Including Remote Users)
```bash
sovereign fedi follow <user@instance>
sovereign fedi unfollow <user@instance>
sovereign fedi followers [--limit <n>]
sovereign fedi following [--limit <n>]
sovereign fedi block <user@instance>
sovereign fedi unblock <user@instance>
sovereign fedi mute <user@instance>
sovereign fedi unmute <user@instance>
```

#### Engagement
```bash
sovereign fedi favorite <post-url>
sovereign fedi unfavorite <post-url>
sovereign fedi boost <post-url>
sovereign fedi unboost <post-url>
```

#### Profile Management
```bash
sovereign fedi profile [<user@instance>]
sovereign fedi profile update --name "<name>" --bio "<bio>" --avatar <path>
```

#### Polls
```bash
sovereign fedi poll "<question>" --option "<opt1>" --option "<opt2>" [--expires <duration>]
sovereign fedi vote <post-url> <option-index>
```

### 7.4 Federation Mechanics

#### Following Remote Users
1. **WebFinger Lookup:** Resolve `user@instance` to actor URL
2. **Fetch Actor:** GET actor document to obtain inbox URL
3. **Send Follow:** POST signed `Follow` activity to remote inbox
4. **Receive Accept:** Remote server sends `Accept` to your inbox
5. **Store Relationship:** Update local following list

#### Receiving Remote Posts
1. **Inbox Delivery:** Remote servers POST activities to your inbox
2. **Signature Verification:** Validate HTTP signature via Rust bridge
3. **Activity Processing:** Parse and store relevant activities
4. **Timeline Update:** Add posts from followed users to home timeline

#### Content Delivery
1. **Create Activity:** Wrap post in `Create` activity
2. **Recipient Resolution:** Determine followers' inboxes (with deduplication by shared inbox)
3. **Signed Delivery:** POST signed activity to each unique inbox
4. **Retry Logic:** Queue and retry failed deliveries

---

## 8. Unified CLI Structure

### 8.1 Top-Level Commands

```bash
sovereign identity setup <domain>     # Initialize identity and DNS
sovereign identity cleanup            # Remove DNS records and clean up
sovereign identity export             # Export identity/keys for backup
sovereign identity import <file>      # Import identity from backup

sovereign server start [--port <n>]   # Start the local server
sovereign server stop                 # Stop the server
sovereign server status               # Check server status

sovereign bsky <command>              # Bluesky/AT Protocol commands
sovereign fedi <command>              # ActivityPub/Fediverse commands

sovereign config get <key>            # Get configuration value
sovereign config set <key> <value>    # Set configuration value
sovereign config list                 # List all configuration
```

### 8.2 Output Formatting

All CLI output uses structured, readable text format:

```
┌─ Post by @alice.bsky.social ─────────────────────┐
│ This is a sample post with some content that     │
│ wraps nicely in the terminal.                    │
├──────────────────────────────────────────────────┤
│ ♡ 42  ⟳ 12  💬 5  │  2 hours ago                 │
└──────────────────────────────────────────────────┘
```

Options for output:
* `--json` - Machine-readable JSON output
* `--plain` - Simple text without formatting
* `--no-color` - Disable ANSI colors

### 8.3 Interactive Mode

```bash
sovereign shell                       # Enter interactive mode
```

Interactive mode provides:
* Tab completion for commands and handles
* Command history
* Real-time notifications
* Streaming timeline updates

---

## 9. Data Storage Architecture

### 9.1 Local Storage (SQLite)

```
~/.sovereign/
├── config.json              # User configuration
├── sovereign.db             # Main SQLite database
├── keys/
│   ├── signing.key          # Private signing key (encrypted)
│   └── signing.pub          # Public key
├── blobs/
│   └── <cid>/               # Cached media files
└── logs/
    └── sovereign.log        # Application logs
```

### 9.2 Database Schema (Core Tables)

```sql
-- Identity
CREATE TABLE identity (
    did TEXT PRIMARY KEY,
    handle TEXT UNIQUE,
    display_name TEXT,
    bio TEXT,
    avatar_cid TEXT,
    created_at TIMESTAMP
);

-- Posts (both protocols)
CREATE TABLE posts (
    uri TEXT PRIMARY KEY,
    protocol TEXT,           -- 'atproto' or 'activitypub'
    author_did TEXT,
    content TEXT,
    reply_to TEXT,
    created_at TIMESTAMP,
    raw_data JSON
);

-- Social Graph
CREATE TABLE follows (
    follower TEXT,
    following TEXT,
    protocol TEXT,
    accepted BOOLEAN,
    created_at TIMESTAMP,
    PRIMARY KEY (follower, following)
);

-- Engagements
CREATE TABLE likes (
    uri TEXT PRIMARY KEY,
    post_uri TEXT,
    created_at TIMESTAMP
);

CREATE TABLE reposts (
    uri TEXT PRIMARY KEY,
    post_uri TEXT,
    created_at TIMESTAMP
);

-- Notifications
CREATE TABLE notifications (
    id TEXT PRIMARY KEY,
    type TEXT,
    actor TEXT,
    subject TEXT,
    read BOOLEAN,
    created_at TIMESTAMP
);

-- Federation Queue
CREATE TABLE delivery_queue (
    id INTEGER PRIMARY KEY,
    activity JSON,
    target_inbox TEXT,
    attempts INTEGER,
    next_retry TIMESTAMP,
    created_at TIMESTAMP
);
```

### 9.3 Cloud Storage (S3)

* **Media Blobs:** Images, videos stored by content hash (CID)
* **Repository Backups:** Periodic CAR file exports
* **Static Assets:** Avatar, banner images served via Cloudflare CDN

---

## 10. Security Architecture

### 10.1 Rust Cryptography Bridge

The Rust bridge handles ALL cryptographic operations:

```rust
// Core signing interface
pub fn sign_message(key: &[u8], message: &[u8]) -> Vec<u8>;
pub fn verify_signature(pubkey: &[u8], message: &[u8], sig: &[u8]) -> bool;

// HTTP Signatures for ActivityPub
pub fn create_http_signature(
    key: &[u8],
    method: &str,
    path: &str,
    headers: &HashMap<String, String>
) -> String;

pub fn verify_http_signature(
    pubkey: &[u8],
    signature_header: &str,
    method: &str,
    path: &str,
    headers: &HashMap<String, String>
) -> bool;

// Key management
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>);
pub fn export_public_key_pem(pubkey: &[u8]) -> String;
```

### 10.2 Key Storage

* Private keys encrypted at rest using system keyring or passphrase
* Keys never exposed to Python or transmitted over network
* Separate keys for AT Protocol and ActivityPub (different algorithms supported)

### 10.3 Authentication Flows

#### AT Protocol
1. Create session with identifier + password
2. Receive JWT access token (short-lived) + refresh token
3. Use access token for API calls
4. Refresh when expired

#### ActivityPub
1. All server-to-server requests signed with HTTP Signatures
2. Local CLI authenticated via local socket or token file
3. No passwords transmitted during federation

---

## 11. Cloudflare Integration

### 11.1 DNS Management

```bash
sovereign identity setup mydomain.com
```

Automatically creates:
* `A` record pointing to server IP
* `TXT` record for AT Protocol handle verification: `_atproto.mydomain.com`
* `TXT` record for domain verification

### 11.2 Cloudflare Services Used

* **DNS:** Handle resolution, domain verification
* **CDN:** Cache static assets and media
* **SSL:** Automatic HTTPS certificates
* **Workers (Optional):** Edge caching for public endpoints

### 11.3 Required API Permissions

* `Zone.DNS` - Read and write DNS records
* `Zone.Zone` - Read zone information

---

## 12. Implementation Phases

### Phase 1: Foundation
- [ ] C# CLI scaffold with command parsing
- [ ] Rust FFI bridge for cryptography
- [ ] SQLite database initialization
- [ ] Configuration management
- [ ] Cloudflare DNS automation

### Phase 2: AT Protocol / Bluesky
- [ ] DID document generation and hosting
- [ ] JWT authentication
- [ ] Repository structure (MST)
- [ ] Core lexicon endpoints
- [ ] Client commands (post, timeline, follow)
- [ ] Media upload to S3
- [ ] Federation with bsky.network relay

### Phase 3: ActivityPub / Mastodon
- [ ] WebFinger and actor documents
- [ ] HTTP Signatures (Rust)
- [ ] Inbox/outbox endpoints
- [ ] Activity send/receive
- [ ] Client commands (post, timeline, follow)
- [ ] Remote user discovery and following
- [ ] Delivery queue with retry

### Phase 4: Polish & Features
- [ ] Interactive shell mode
- [ ] Notifications system
- [ ] Search functionality
- [ ] Polls (ActivityPub)
- [ ] Content warnings
- [ ] Media attachments with alt text
- [ ] Block/mute lists
- [ ] Data export/import

---

## 13. Testing Strategy

### 13.1 Unit Tests
* Cryptographic operations (Rust)
* Database operations
* Activity parsing/generation
* CLI command parsing

### 13.2 Integration Tests
* Full post creation flow
* Follow/unfollow cycle
* Federation delivery simulation

### 13.3 Federation Testing
* Test against Mastodon instance (docker)
* Test against Bluesky PDS sandbox
* Signature verification with real servers

---

## Current Status

**Design is now comprehensive. No code has been generated yet.**

### Ready for Implementation

The design now covers:
- Full Bluesky/AT Protocol client and server
- Full ActivityPub/Mastodon client and server
- Complete CLI command structure for all user features
- Federation mechanics for connecting to external servers
- Data storage and security architecture

**Next step: Proceed with code generation starting from Phase 1 (Foundation)?**
