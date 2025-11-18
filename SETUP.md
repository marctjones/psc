# Sovereign Setup Guide for Ubuntu 25.10

This guide walks you through setting up and testing Sovereign on your Ubuntu desktop.

## Build Status

- **sovereign-core**: ✅ Builds and tests pass (32 tests)
- **sovereign-cli**: ✅ Builds and tests pass (11 tests)
- **sovereign-worker**: ✅ Builds and tests pass (16 tests)

---

## Prerequisites

### 1. Install Rust

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow prompts, then reload your shell
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 2. Install Node.js (for Wrangler CLI)

```bash
# Using NodeSource repository
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Verify
node --version
npm --version
```

### 3. Install Wrangler (Cloudflare CLI)

```bash
npm install -g wrangler

# Verify
wrangler --version
```

### 4. Install Build Dependencies

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev
```

---

## Building the Project

### 1. Clone and Build

```bash
# Clone the repository
git clone <your-repo-url> sovereign
cd sovereign

# Build the CLI and core library
cargo build --release -p sovereign-core -p sovereign-cli

# The binary will be at:
# target/release/sovereign
```

### 2. Install the CLI (Optional)

```bash
# Install to ~/.cargo/bin (should be in your PATH)
cargo install --path crates/sovereign-cli

# Now you can run from anywhere:
sovereign --help
```

### 3. Run Tests

```bash
cargo test -p sovereign-core -p sovereign-cli
```

---

## Configuration

### 1. Create Cloudflare Account

1. Go to https://dash.cloudflare.com/sign-up
2. Create a free account
3. Add your domain (or use a free workers.dev subdomain for testing)

### 2. Generate API Token

1. Go to https://dash.cloudflare.com/profile/api-tokens
2. Click "Create Token"
3. Use "Edit zone DNS" template or create custom with:
   - Zone > DNS > Edit
   - Zone > Zone > Read
4. Copy the token

### 3. Configure Sovereign

```bash
# Set your Cloudflare API token
sovereign config set cloudflare_api_token YOUR_TOKEN_HERE

# View current configuration
sovereign config list
```

---

## Local Testing (Without Cloudflare)

You can test the CLI locally without deploying to Cloudflare:

### 1. Test CLI Commands

```bash
# View help
sovereign --help
sovereign identity --help
sovereign bsky --help
sovereign fedi --help

# Test configuration
sovereign config list
sovereign config set handle alice
sovereign config set domain localhost
sovereign config list

# Test identity commands (won't actually call Cloudflare)
sovereign whoami  # Under bsky or fedi subcommand
sovereign bsky whoami
sovereign fedi whoami
```

### 2. Test Post Formatting

```bash
# These will show placeholder output
sovereign bsky post "Hello from Sovereign!"
sovereign fedi post "Testing ActivityPub"
```

---

## Testing with a Local Server

To test the actual server endpoints, you'll need to run a local development server.

### Option A: Using Wrangler Dev Mode

**Note:** This requires the worker crate API to be updated first.

```bash
# Login to Cloudflare
wrangler login

# Create D1 database
wrangler d1 create sovereign-db

# Update wrangler.toml with the database ID

# Run local dev server
wrangler dev
```

### Option B: Mock Server for Testing

For browser testing without Cloudflare, you can create a simple mock server.

Create `mock_server.py`:

```python
#!/usr/bin/env python3
"""Simple mock server for testing Sovereign endpoints"""

from http.server import HTTPServer, BaseHTTPRequestHandler
import json

class SovereignHandler(BaseHTTPRequestHandler):
    def _send_json(self, data, status=200):
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())

    def do_GET(self):
        if self.path == '/health':
            self._send_json({'status': 'ok'})

        elif self.path.startswith('/.well-known/webfinger'):
            # WebFinger response
            self._send_json({
                'subject': 'acct:user@localhost:8787',
                'links': [{
                    'rel': 'self',
                    'type': 'application/activity+json',
                    'href': 'http://localhost:8787/users/user'
                }]
            })

        elif self.path == '/.well-known/nodeinfo':
            self._send_json({
                'links': [{
                    'rel': 'http://nodeinfo.diaspora.software/ns/schema/2.1',
                    'href': 'http://localhost:8787/nodeinfo/2.1'
                }]
            })

        elif self.path == '/nodeinfo/2.1':
            self._send_json({
                'version': '2.1',
                'software': {'name': 'sovereign', 'version': '0.1.0'},
                'protocols': ['activitypub'],
                'usage': {
                    'users': {'total': 1, 'activeMonth': 1, 'activeHalfyear': 1},
                    'localPosts': 0
                },
                'openRegistrations': False
            })

        elif self.path == '/users/user':
            # Actor document
            self._send_json({
                '@context': [
                    'https://www.w3.org/ns/activitystreams',
                    'https://w3id.org/security/v1'
                ],
                'id': 'http://localhost:8787/users/user',
                'type': 'Person',
                'preferredUsername': 'user',
                'name': 'Sovereign User',
                'summary': 'A self-hosted ActivityPub account',
                'inbox': 'http://localhost:8787/users/user/inbox',
                'outbox': 'http://localhost:8787/users/user/outbox',
                'followers': 'http://localhost:8787/users/user/followers',
                'following': 'http://localhost:8787/users/user/following',
                'publicKey': {
                    'id': 'http://localhost:8787/users/user#main-key',
                    'owner': 'http://localhost:8787/users/user',
                    'publicKeyPem': '-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----'
                }
            })

        elif self.path == '/.well-known/did.json':
            # DID document for AT Protocol
            self._send_json({
                '@context': [
                    'https://www.w3.org/ns/did/v1',
                    'https://w3id.org/security/multikey/v1'
                ],
                'id': 'did:web:localhost%3A8787',
                'alsoKnownAs': ['at://localhost:8787'],
                'verificationMethod': [{
                    'id': 'did:web:localhost%3A8787#atproto',
                    'type': 'Multikey',
                    'controller': 'did:web:localhost%3A8787',
                    'publicKeyMultibase': 'zDnaerDaTF5BXEavCrfRZEk316dpbLsfPDZ3WJ5hRTPFU2169'
                }],
                'service': [{
                    'id': '#atproto_pds',
                    'type': 'AtprotoPersonalDataServer',
                    'serviceEndpoint': 'http://localhost:8787'
                }]
            })

        elif self.path == '/users/user/outbox':
            self._send_json({
                '@context': 'https://www.w3.org/ns/activitystreams',
                'id': 'http://localhost:8787/users/user/outbox',
                'type': 'OrderedCollection',
                'totalItems': 0
            })

        elif self.path == '/users/user/followers':
            self._send_json({
                '@context': 'https://www.w3.org/ns/activitystreams',
                'id': 'http://localhost:8787/users/user/followers',
                'type': 'OrderedCollection',
                'totalItems': 0
            })

        elif self.path == '/users/user/following':
            self._send_json({
                '@context': 'https://www.w3.org/ns/activitystreams',
                'id': 'http://localhost:8787/users/user/following',
                'type': 'OrderedCollection',
                'totalItems': 0
            })

        else:
            self.send_error(404)

    def do_POST(self):
        if self.path == '/users/user/inbox':
            # Accept any activity
            self._send_json({'status': 'accepted'}, 202)
        else:
            self.send_error(404)

    def log_message(self, format, *args):
        print(f"[{self.log_date_time_string()}] {format % args}")

if __name__ == '__main__':
    server = HTTPServer(('localhost', 8787), SovereignHandler)
    print("Mock Sovereign server running at http://localhost:8787")
    print("\nTest URLs:")
    print("  Health:     http://localhost:8787/health")
    print("  WebFinger:  http://localhost:8787/.well-known/webfinger?resource=acct:user@localhost:8787")
    print("  NodeInfo:   http://localhost:8787/.well-known/nodeinfo")
    print("  Actor:      http://localhost:8787/users/user")
    print("  DID:        http://localhost:8787/.well-known/did.json")
    print("\nPress Ctrl+C to stop")
    server.serve_forever()
```

Run it:

```bash
python3 mock_server.py
```

---

## Testing with a Web Browser

### 1. Start the Mock Server

```bash
python3 mock_server.py
```

### 2. Test Endpoints in Browser

Open these URLs in your browser:

| Endpoint | URL | Description |
|----------|-----|-------------|
| Health | http://localhost:8787/health | Server status |
| WebFinger | http://localhost:8787/.well-known/webfinger?resource=acct:user@localhost:8787 | User discovery |
| NodeInfo | http://localhost:8787/.well-known/nodeinfo | Server metadata |
| Actor | http://localhost:8787/users/user | ActivityPub profile |
| DID | http://localhost:8787/.well-known/did.json | AT Protocol identity |
| Outbox | http://localhost:8787/users/user/outbox | User's posts |
| Followers | http://localhost:8787/users/user/followers | Follower list |

### 3. Test with curl

```bash
# Health check
curl http://localhost:8787/health

# WebFinger (pretty-printed)
curl -s "http://localhost:8787/.well-known/webfinger?resource=acct:user@localhost:8787" | jq

# Actor document
curl -s http://localhost:8787/users/user | jq

# DID document
curl -s http://localhost:8787/.well-known/did.json | jq
```

### 4. Test ActivityPub Federation (Manual)

To test that your server would be discoverable:

```bash
# WebFinger lookup (how Mastodon finds users)
curl -H "Accept: application/jrd+json" \
  "http://localhost:8787/.well-known/webfinger?resource=acct:user@localhost:8787"

# Actor document (how Mastodon gets profile info)
curl -H "Accept: application/activity+json" \
  http://localhost:8787/users/user
```

---

## Deployment Validation

Use the built-in test command to validate your deployment:

### Quick Connectivity Test

```bash
# Test if server is reachable
sovereign test quick http://localhost:8787

# Test production deployment
sovereign test quick https://yourdomain.com
```

### Full Validation Suite

```bash
# Run all validation tests against mock server
sovereign test validate http://localhost:8787

# With verbose output (shows response details)
sovereign test validate http://localhost:8787 --verbose

# Test production deployment
sovereign test validate https://yourdomain.com --verbose
```

The validation suite tests:
- Health endpoint
- WebFinger discovery
- NodeInfo metadata
- Actor document (ActivityPub)
- DID document (AT Protocol)
- Outbox collection
- AT Protocol server description

### Sample Output

```
Running deployment validation tests...
Target: http://localhost:8787

Test Results:
============================================================
[PASS] Health Check (12ms)
[PASS] WebFinger (8ms)
[PASS] NodeInfo (15ms)
[PASS] Actor Document (6ms)
[PASS] DID Document (5ms)
[PASS] Outbox (7ms)
[PASS] AT Protocol Server (9ms)
============================================================
Total: 7 passed, 0 failed (62 ms)

All tests passed!
```

### Remote Network Connectivity Tests

Test connectivity to remote networks to verify external services are accessible:

```bash
# Test connection to a Mastodon/ActivityPub instance
sovereign test remote-fedi mastodon.social

# Test other instances
sovereign test remote-fedi fosstodon.org
sovereign test remote-fedi hachyderm.io

# Test Bluesky network connectivity
sovereign test remote-bsky
```

### Test Connection to a Specific User

Test connectivity to a specific remote user without sending them any data (read-only):

```bash
# Test connection to a user (username@instance format)
sovereign test remote-user gargron@mastodon.social
sovereign test remote-user marcjones@mastodon.social
```

This performs read-only tests:
- WebFinger lookup (discover the user's profile URL)
- Actor document fetch (get their public profile)
- Public outbox fetch (their public posts)
- Collection info (followers/following counts if public)

**No data is sent to the user** - this only fetches publicly available information.

These tests help distinguish between "their system is down" vs "our code has bugs" by verifying:

**For ActivityPub/Mastodon (`remote-fedi`):**
- Instance reachability (HTTP 200 from root)
- NodeInfo availability (server metadata)
- WebFinger endpoint (user discovery)
- Actor fetching (profile retrieval)

**For Bluesky (`remote-bsky`):**
- Bluesky API accessibility
- Handle resolution service
- PDS server description
- Public feed access

### Sample Remote Test Output

```
Testing remote ActivityPub/Mastodon connectivity...
Instance: mastodon.social

Remote Connectivity Results:
============================================================
[PASS] Instance Reachable (245ms)
[PASS] NodeInfo Available (312ms)
[PASS] WebFinger Endpoint (198ms)
[PASS] Actor Fetching (287ms)
============================================================
Total: 4 passed, 0 failed (1042 ms)

mastodon.social is accessible!
```

---

## Deploying to Cloudflare

The worker crate is ready for deployment:

### 1. Setup Cloudflare Resources

```bash
# Login
wrangler login

# Create D1 database
wrangler d1 create sovereign-db

# Create R2 bucket
wrangler r2 bucket create sovereign-media
```

### 2. Update wrangler.toml

Edit `wrangler.toml` with your database and bucket IDs.

### 3. Run Migrations

```bash
wrangler d1 execute sovereign-db --file=migrations/0001_initial.sql
```

### 4. Deploy

```bash
wrangler deploy
```

### 5. Setup DNS

```bash
sovereign identity setup yourdomain.com
```

---

## Troubleshooting

### Build Errors

If you get OpenSSL errors:
```bash
sudo apt-get install libssl-dev pkg-config
```

If you get linking errors:
```bash
sudo apt-get install build-essential
```

### Wrangler Issues

If wrangler login fails:
```bash
# Clear credentials and try again
rm -rf ~/.wrangler
wrangler login
```

### Permission Errors

If npm install fails:
```bash
# Use local install instead of global
npm install wrangler
npx wrangler --version
```

---

## Next Steps

1. **Deploy to Cloudflare**: Use actual D1/R2 storage
2. **Test Federation**: Connect with real Mastodon/Bluesky instances
3. **Add UI**: Consider adding a simple web UI

---

## File Locations

After building:

- **CLI Binary**: `target/release/sovereign`
- **Config File**: `~/.sovereign/config.json`
- **Logs**: `~/.sovereign/logs/sovereign.log`

---

## Quick Reference

```bash
# Build
cargo build --release -p sovereign-cli

# Test
cargo test -p sovereign-core -p sovereign-cli

# Run CLI
./target/release/sovereign --help

# Configure
sovereign config set cloudflare_api_token YOUR_TOKEN
sovereign config set domain yourdomain.com
sovereign config set handle yourname
sovereign config list

# Identity
sovereign identity setup yourdomain.com
sovereign identity cleanup

# Bluesky commands
sovereign bsky whoami
sovereign bsky post "Hello!"
sovereign bsky timeline

# Fediverse commands
sovereign fedi whoami
sovereign fedi post "Hello!"
sovereign fedi timeline

# Deployment testing
sovereign test quick http://localhost:8787
sovereign test validate http://localhost:8787
sovereign test validate https://yourdomain.com --verbose

# Remote network testing
sovereign test remote-fedi mastodon.social
sovereign test remote-bsky
sovereign test remote-user gargron@mastodon.social
```
