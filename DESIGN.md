# Project Sovereign Core (V2.3)

## Consolidated Handoff Prompt (Final Version)

We are beginning the creation of "Project Sovereign," a self-hosted digital ecosystem using only **commodity cloud services** (Cloudflare free tier) and focusing entirely on **user freedom, simplicity, and low-cost maintenance**. Our goal is to make it easy for an individual to deploy their own digital presence at **zero ongoing cost**.

We are specifically targeting services that currently trap users into proprietary single sign-on (SSO) systems (Google, Facebook, Amazon, Apple) and hosted social media. We are **not interested** in recreating existing, well-tested open-source tools like full email servers.

The current focus on the single-user ActivityPub/Bluesky server is **Step One**—a sample implementation to validate the architecture. The **long-term goal** is to brainstorm and prioritize other simple, cheap, and easily deployable services that enhance personal digital sovereignty.

---

## 1. Architectural Mandate & Stack

### 1.1 Goals
* CLI-first, single-user system with portable binaries
* **$0/month hosting** on Cloudflare free tier
* We are **actively not interested in trying to make things scale**

### 1.2 Technology Stack: Rust Only

| Component | Technology | Deployment |
|-----------|------------|------------|
| **CLI Tool** | Rust (native binary) | User's machine |
| **Server** | Rust → WebAssembly | Cloudflare Workers |
| **Database** | Cloudflare D1 | Cloudflare (SQLite) |
| **Blob Storage** | Cloudflare R2 | Cloudflare (S3-compatible) |
| **DNS/CDN** | Cloudflare | Cloudflare |

### 1.3 Why Rust Only?

* **Small WASM binaries** - 100KB-500KB vs 2-5MB for .NET
* **No runtime overhead** - Compiles directly to WASM
* **First-class Cloudflare support** - Official tooling and examples
* **Single language** - No FFI complexity between C# and Rust
* **Cryptography built-in** - Native access to ring/RustCrypto

### 1.4 Cloudflare Free Tier Limits

| Service | Free Limit | Sufficient For |
|---------|------------|----------------|
| Workers | 100K requests/day | ~1 req/sec sustained |
| D1 | 5GB, 5M reads/day | Years of posts |
| R2 | 10GB storage | Thousands of images |
| DNS | Unlimited | All needs |

---

## 2. Deployment Architecture

```
┌─────────────────────────────────────────────────────┐
│              CLOUDFLARE (Free Tier)                 │
│                                                     │
│  ┌─────────────────────────────────────────────┐   │
│  │         Rust → WebAssembly                  │   │
│  │                                             │   │
│  │  - ActivityPub endpoints                    │   │
│  │  - AT Protocol endpoints                    │   │
│  │  - HTTP Signature creation/verification     │   │
│  │  - JWT authentication                       │   │
│  │  - All business logic                       │   │
│  └─────────────────────────────────────────────┘   │
│         │              │              │             │
│         ▼              ▼              ▼             │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐        │
│  │    D1    │   │    R2    │   │    KV    │        │
│  │ (SQLite) │   │  (Blobs) │   │ (Cache)  │        │
│  └──────────┘   └──────────┘   └──────────┘        │
└─────────────────────────────────────────────────────┘
                         ▲
                         │ HTTPS
                         ▼
┌─────────────────────────────────────────────────────┐
│                  USER'S MACHINE                     │
│  ┌─────────────────────────────────────────────┐   │
│  │  sovereign CLI (Rust native binary)         │   │
│  │                                             │   │
│  │  - User-facing commands                     │   │
│  │  - Text-based UI                            │   │
│  │  - Calls Cloudflare-hosted API              │   │
│  └─────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

---

## 3. Python Isolation Mandate (For Scripting/Demos Only)

Python scripts for demos and automation must **ALWAYS** be developed and executed within **isolated environments** (e.g., venv, pipx). Dependencies must be minimized and limited to stable, popular libraries (e.g., requests, click). Python must not populate the host system.

**Python is NOT used for any core functionality.**

---

## 4. Required Functionality & Interactions

* **Identity Lifecycle:** The Rust CLI must include `sovereign identity setup <domain>` and `cleanup` commands that automate DNS records (A, TXT) via the Cloudflare API.
* **Protocol Implementation:** Both AT Protocol and ActivityPub are implemented in Rust, with cryptographic operations using the `ring` or `RustCrypto` crates.

---

## 5. Explicit Exclusions (DO NOT Implement)

* Do not use **any language other than Rust for core CLI/server logic**
* Do not use **Kubernetes, Docker Swarm, or complex scaling/load-balancing logic**
* Do not include **SMTP/Email hosting**
* Do not require **any paid services** - everything must work on free tiers

---

## 6. Future Trajectory (The Next Steps)

Once the core server is stable, the project's focus will shift to **brainstorming and prioritizing** future components that are **simple to build, cheap and easy to host**, and align with personal interest and user freedom (e.g., decentralized contacts/calendar, private photo archive, lightweight personal search).

---

## 7. Bluesky / AT Protocol Implementation

### 7.1 Overview

The AT Protocol (Authenticated Transfer Protocol) powers Bluesky. Our implementation includes both a **Personal Data Server (PDS)** for hosting your identity and data, and a **client** for interacting with the Bluesky network.

### 7.2 Server Components (PDS)

#### Identity & Authentication
* **DID Document Hosting:** Serve `did:web` or `did:plc` documents at `/.well-known/did.json`
* **Handle Resolution:** DNS TXT record `_atproto.<domain>` pointing to DID
* **JWT Authentication:** Issue and validate access/refresh tokens
* **App Passwords:** Support for third-party client authentication

#### Data Repository
* **Repository Structure:** Merkle Search Tree (MST) for content-addressable storage
* **Record Types:** Posts, likes, reposts, follows, blocks, profile
* **Blob Storage:** Images and media stored in R2, referenced by CID
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

### 7.3 Client Features (CLI Commands)

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

### 7.4 Federation with Bluesky Network

* **Relay Connection:** Subscribe to `bsky.network` firehose for global feed access
* **AppView Delegation:** Use `api.bsky.app` for aggregated views (likes, reposts, followers counts)
* **PDS Registration:** Register with Bluesky PLC directory for `did:plc` identifiers
* **Alternative:** Use `did:web` for fully self-sovereign identity (no PLC dependency)

---

## 8. ActivityPub / Mastodon Implementation

### 8.1 Overview

ActivityPub is the W3C standard powering Mastodon and the Fediverse. Our implementation includes a **single-user server** that can federate with any ActivityPub-compatible instance.

### 8.2 Server Components

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

#### HTTP Signatures
All federated requests **MUST** be signed using HTTP Signatures (RFC 9421):
* **Algorithm:** RSA-SHA256 or Ed25519 (using `ring` crate)
* **Headers Signed:** `(request-target)`, `host`, `date`, `digest`
* **Key Management:** RSA/Ed25519 keypair stored in D1/KV
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

### 8.3 Client Features (CLI Commands)

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

### 8.4 Federation Mechanics

#### Following Remote Users
1. **WebFinger Lookup:** Resolve `user@instance` to actor URL
2. **Fetch Actor:** GET actor document to obtain inbox URL
3. **Send Follow:** POST signed `Follow` activity to remote inbox
4. **Receive Accept:** Remote server sends `Accept` to your inbox
5. **Store Relationship:** Update D1 database

#### Receiving Remote Posts
1. **Inbox Delivery:** Remote servers POST activities to your inbox
2. **Signature Verification:** Validate HTTP signature
3. **Activity Processing:** Parse and store in D1
4. **Timeline Update:** Add posts from followed users to home timeline

#### Content Delivery
1. **Create Activity:** Wrap post in `Create` activity
2. **Recipient Resolution:** Determine followers' inboxes (with deduplication by shared inbox)
3. **Signed Delivery:** POST signed activity to each unique inbox
4. **Retry Logic:** Queue failed deliveries in D1 for retry

---

## 9. Unified CLI Structure

### 9.1 Top-Level Commands

```bash
sovereign identity setup <domain>     # Initialize identity and DNS
sovereign identity cleanup            # Remove DNS records and clean up
sovereign identity export             # Export identity/keys for backup
sovereign identity import <file>      # Import identity from backup

sovereign server deploy               # Deploy to Cloudflare Workers
sovereign server status               # Check deployment status
sovereign server logs                 # View recent logs

sovereign bsky <command>              # Bluesky/AT Protocol commands
sovereign fedi <command>              # ActivityPub/Fediverse commands

sovereign config get <key>            # Get configuration value
sovereign config set <key> <value>    # Set configuration value
sovereign config list                 # List all configuration
```

### 9.2 Output Formatting

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

### 9.3 Interactive Mode

```bash
sovereign shell                       # Enter interactive mode
```

Interactive mode provides:
* Tab completion for commands and handles
* Command history
* Real-time notifications
* Streaming timeline updates

---

## 10. Data Storage Architecture

### 10.1 Cloudflare D1 (SQLite)

All structured data stored in Cloudflare D1:

### 10.2 Database Schema (Core Tables)

```sql
-- Identity
CREATE TABLE identity (
    did TEXT PRIMARY KEY,
    handle TEXT UNIQUE,
    display_name TEXT,
    bio TEXT,
    avatar_cid TEXT,
    private_key_encrypted TEXT,
    public_key TEXT,
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

### 10.3 Cloudflare R2 (Blob Storage)

* **Media Blobs:** Images, videos stored by content hash (CID)
* **Repository Backups:** Periodic CAR file exports
* **Static Assets:** Avatar, banner images

### 10.4 Cloudflare KV (Optional Cache)

* **Session tokens**
* **Frequently accessed profiles**
* **Rate limiting counters**

---

## 11. Security Architecture

### 11.1 Cryptography (Rust Crates)

```rust
// Recommended crates
ring        // Fast, safe crypto primitives
ed25519-dalek  // Ed25519 signatures
rsa         // RSA signatures for ActivityPub
sha2        // SHA-256 for digests
base64      // Encoding
```

### 11.2 Key Storage

* Private keys encrypted with user passphrase
* Stored in D1 (encrypted) or Cloudflare Secrets
* Never transmitted - signing happens in Worker

### 11.3 Authentication Flows

#### AT Protocol
1. Create session with identifier + password
2. Receive JWT access token (short-lived) + refresh token
3. Use access token for API calls
4. Refresh when expired

#### ActivityPub
1. All server-to-server requests signed with HTTP Signatures
2. CLI authenticated via API token stored locally
3. No passwords transmitted during federation

---

## 12. Cloudflare Integration

### 12.1 DNS Management

```bash
sovereign identity setup mydomain.com
```

Automatically creates:
* `A` record pointing to Workers (or CNAME to workers.dev)
* `TXT` record for AT Protocol handle verification: `_atproto.mydomain.com`
* `TXT` record for domain verification

### 12.2 Worker Deployment

```bash
sovereign server deploy
```

Uses Wrangler CLI under the hood:
* Compiles Rust to WASM
* Deploys to Cloudflare Workers
* Binds D1, R2, KV resources

### 12.3 Required API Permissions

* `Zone.DNS` - Read and write DNS records
* `Zone.Zone` - Read zone information
* `Workers Scripts` - Deploy workers
* `D1` - Database access
* `R2` - Blob storage access

---

## 13. Rust Project Structure

```
sovereign/
├── Cargo.toml
├── Cargo.lock
├── wrangler.toml              # Cloudflare Worker config
│
├── crates/
│   ├── sovereign-cli/         # Native CLI binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── commands/
│   │       │   ├── mod.rs
│   │       │   ├── identity.rs
│   │       │   ├── bsky.rs
│   │       │   ├── fedi.rs
│   │       │   └── config.rs
│   │       └── ui/
│   │           ├── mod.rs
│   │           └── formatting.rs
│   │
│   ├── sovereign-worker/      # Cloudflare Worker (WASM)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── routes/
│   │       │   ├── mod.rs
│   │       │   ├── atproto.rs
│   │       │   ├── activitypub.rs
│   │       │   └── wellknown.rs
│   │       ├── crypto/
│   │       │   ├── mod.rs
│   │       │   ├── signatures.rs
│   │       │   └── jwt.rs
│   │       └── storage/
│   │           ├── mod.rs
│   │           ├── d1.rs
│   │           └── r2.rs
│   │
│   └── sovereign-core/        # Shared library
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── types/
│           │   ├── mod.rs
│           │   ├── atproto.rs
│           │   └── activitypub.rs
│           └── protocol/
│               ├── mod.rs
│               ├── lexicon.rs
│               └── activity.rs
│
├── migrations/                # D1 database migrations
│   └── 0001_initial.sql
│
└── scripts/                   # Python helper scripts (isolated)
    └── demo.py
```

---

## 14. Implementation Phases

### Phase 1: Foundation
- [ ] Rust workspace setup with three crates
- [ ] Cloudflare Worker scaffold (wrangler)
- [ ] D1 database schema and migrations
- [ ] Basic CLI with config management
- [ ] Cloudflare API integration (DNS)

### Phase 2: AT Protocol / Bluesky
- [ ] DID document generation and hosting
- [ ] JWT authentication
- [ ] Repository structure (MST)
- [ ] Core lexicon endpoints
- [ ] Client commands (post, timeline, follow)
- [ ] Media upload to R2
- [ ] Federation with bsky.network relay

### Phase 3: ActivityPub / Mastodon
- [ ] WebFinger and actor documents
- [ ] HTTP Signatures (ring crate)
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

## 15. Testing Strategy

### 15.1 Unit Tests
* Cryptographic operations
* Database operations
* Activity parsing/generation
* CLI command parsing

### 15.2 Integration Tests
* Full post creation flow
* Follow/unfollow cycle
* Federation delivery simulation

### 15.3 Federation Testing
* Test against Mastodon instance (docker)
* Test against Bluesky PDS sandbox
* Signature verification with real servers

### 15.4 Local Development
* Wrangler dev mode for local Worker testing
* Miniflare for D1/R2/KV simulation

---

## 16. Key Dependencies (Rust Crates)

### Worker (WASM)
```toml
[dependencies]
worker = "0.0.18"              # Cloudflare Workers SDK
serde = { version = "1", features = ["derive"] }
serde_json = "1"
ring = "0.17"                  # Cryptography
base64 = "0.21"
chrono = "0.4"
```

### CLI (Native)
```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
colored = "2"                  # Terminal colors
dialoguer = "0.11"             # Interactive prompts
indicatif = "0.17"             # Progress bars
```

---

## Current Status

**Design is now comprehensive. No code has been generated yet.**

### Architecture Summary

* **Language:** Rust only
* **Hosting:** Cloudflare free tier ($0/month)
* **CLI:** Native Rust binary on user's machine
* **Server:** Rust → WASM on Cloudflare Workers
* **Storage:** D1 (SQLite) + R2 (blobs)

### Ready for Implementation

The design covers:
- Full Bluesky/AT Protocol client and server
- Full ActivityPub/Mastodon client and server
- Complete CLI command structure for all user features
- Federation mechanics for connecting to external servers
- Data storage and security architecture
- Rust project structure and dependencies

**Next step: Proceed with code generation starting from Phase 1 (Foundation)?**
