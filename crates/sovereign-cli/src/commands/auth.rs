//! OAuth authentication commands for CLI

use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// Token storage for CLI authentication
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenStorage {
    pub server: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub expires_at: Option<String>,
}

impl TokenStorage {
    /// Load tokens from storage file
    pub fn load() -> Result<Self> {
        let path = get_token_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    /// Save tokens to storage file
    pub fn save(&self) -> Result<()> {
        let path = get_token_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Clear all tokens
    pub fn clear(&mut self) -> Result<()> {
        *self = Self::default();
        self.save()
    }
}

/// Get the path to token storage file
fn get_token_path() -> Result<PathBuf> {
    let mut path = dirs::config_dir()
        .ok_or("Could not find config directory")?;
    path.push("sovereign");
    path.push("auth.json");
    Ok(path)
}

/// Device authorization response from server
#[derive(Debug, Deserialize)]
struct DeviceAuthResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: String,
    expires_in: u32,
    interval: u32,
}

/// Token response from server
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u32,
    refresh_token: Option<String>,
    scope: Option<String>,
}

/// Token error response
#[derive(Debug, Deserialize)]
struct TokenError {
    error: String,
    error_description: Option<String>,
}

/// Log in to your Sovereign server using device authorization flow
pub async fn login(server: &str, scope: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let server = server.trim_end_matches('/');

    println!("{}", "Starting device authorization flow...".cyan());

    // Step 1: Request device authorization
    let device_auth_url = format!("{}/oauth/device", server);
    let response = client
        .post(&device_auth_url)
        .form(&[
            ("client_id", "sovereign-cli"),
            ("scope", scope),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("Device authorization failed: {}", error_text).into());
    }

    let device_auth: DeviceAuthResponse = response.json().await?;

    // Step 2: Show user the authorization instructions
    println!();
    println!("{}", "To complete login:".yellow().bold());
    println!();
    println!("  1. Open this URL in your browser:");
    println!("     {}", device_auth.verification_uri_complete.cyan());
    println!();
    println!("  2. Enter this code if prompted:");
    println!("     {}", device_auth.user_code.green().bold());
    println!();
    println!("  3. Authorize the device");
    println!();
    println!("Waiting for authorization (expires in {} seconds)...", device_auth.expires_in);

    // Step 3: Poll for token
    let token_url = format!("{}/oauth/token", server);
    let mut interval = std::time::Duration::from_secs(device_auth.interval as u64);
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(device_auth.expires_in as u64);

    loop {
        tokio::time::sleep(interval).await;

        if start.elapsed() > timeout {
            return Err("Authorization timed out".into());
        }

        let response = client
            .post(&token_url)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", &device_auth.device_code),
                ("client_id", "sovereign-cli"),
            ])
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            // Got the token!
            let token: TokenResponse = serde_json::from_str(&body)?;

            // Calculate expiration time
            let expires_at = chrono::Utc::now()
                + chrono::Duration::seconds(token.expires_in as i64);

            // Save tokens
            let storage = TokenStorage {
                server: Some(server.to_string()),
                access_token: Some(token.access_token),
                refresh_token: token.refresh_token,
                scope: token.scope,
                expires_at: Some(expires_at.to_rfc3339()),
            };
            storage.save()?;

            println!();
            println!("{} Successfully logged in!", "✓".green().bold());
            println!("  Server: {}", server);
            if let Some(scope) = &storage.scope {
                println!("  Scope: {}", scope);
            }
            return Ok(());
        }

        // Check for specific errors
        if let Ok(error) = serde_json::from_str::<TokenError>(&body) {
            match error.error.as_str() {
                "authorization_pending" => {
                    // Continue polling
                    print!(".");
                    use std::io::Write;
                    std::io::stdout().flush().ok();
                }
                "slow_down" => {
                    // Increase polling interval
                    interval += std::time::Duration::from_secs(5);
                }
                "expired_token" => {
                    return Err("Authorization expired. Please try again.".into());
                }
                "access_denied" => {
                    return Err("Authorization was denied.".into());
                }
                _ => {
                    let desc = error.error_description.unwrap_or_default();
                    return Err(format!("Authorization error: {} - {}", error.error, desc).into());
                }
            }
        }
    }
}

/// Log out and revoke tokens
pub async fn logout() -> Result<()> {
    let mut storage = TokenStorage::load()?;

    if storage.server.is_none() || storage.access_token.is_none() {
        println!("{} Not logged in", "!".yellow());
        return Ok(());
    }

    let server = storage.server.as_ref().unwrap();
    let client = reqwest::Client::new();

    // Revoke access token
    if let Some(token) = &storage.access_token {
        let revoke_url = format!("{}/oauth/revoke", server);
        let _ = client
            .post(&revoke_url)
            .form(&[("token", token)])
            .send()
            .await;
    }

    // Revoke refresh token
    if let Some(token) = &storage.refresh_token {
        let revoke_url = format!("{}/oauth/revoke", server);
        let _ = client
            .post(&revoke_url)
            .form(&[("token", token)])
            .send()
            .await;
    }

    // Clear local storage
    storage.clear()?;

    println!("{} Logged out successfully", "✓".green().bold());
    Ok(())
}

/// Show current authentication status
pub async fn status() -> Result<()> {
    let storage = TokenStorage::load()?;

    if storage.server.is_none() || storage.access_token.is_none() {
        println!("{} Not logged in", "!".yellow());
        println!();
        println!("Use {} to authenticate", "sovereign auth login <server>".cyan());
        return Ok(());
    }

    let server = storage.server.as_ref().unwrap();

    println!("{}", "Authentication Status".bold());
    println!();
    println!("  Server: {}", server.cyan());

    if let Some(scope) = &storage.scope {
        println!("  Scope: {}", scope);
    }

    if let Some(expires_at) = &storage.expires_at {
        if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(expires_at) {
            let now = chrono::Utc::now();
            if expires.with_timezone(&chrono::Utc) > now {
                let duration = expires.with_timezone(&chrono::Utc) - now;
                let minutes = duration.num_minutes();
                if minutes > 60 {
                    println!("  Expires: in {} hours", minutes / 60);
                } else {
                    println!("  Expires: in {} minutes", minutes);
                }
            } else {
                println!("  Expires: {} (expired)", "token expired".red());
                println!();
                println!("Use {} to get a new token", "sovereign auth refresh".cyan());
            }
        }
    }

    // Verify token with server
    let client = reqwest::Client::new();
    let introspect_url = format!("{}/oauth/introspect", server);

    if let Some(token) = &storage.access_token {
        let response = client
            .post(&introspect_url)
            .form(&[("token", token)])
            .send()
            .await;

        if let Ok(resp) = response {
            if let Ok(info) = resp.json::<serde_json::Value>().await {
                if info.get("active").and_then(|v| v.as_bool()).unwrap_or(false) {
                    println!("  Status: {}", "active".green().bold());
                    if let Some(username) = info.get("username").and_then(|v| v.as_str()) {
                        println!("  Username: {}", username);
                    }
                } else {
                    println!("  Status: {}", "inactive".red());
                }
            }
        }
    }

    Ok(())
}

/// Refresh the access token
pub async fn refresh() -> Result<()> {
    let mut storage = TokenStorage::load()?;

    if storage.server.is_none() || storage.refresh_token.is_none() {
        return Err("No refresh token available. Please log in again.".into());
    }

    let server = storage.server.as_ref().unwrap();
    let refresh_token = storage.refresh_token.as_ref().unwrap();

    let client = reqwest::Client::new();
    let token_url = format!("{}/oauth/token", server);

    let response = client
        .post(&token_url)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", "sovereign-cli"),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("Token refresh failed: {}", error_text).into());
    }

    let token: TokenResponse = response.json().await?;

    // Calculate expiration time
    let expires_at = chrono::Utc::now()
        + chrono::Duration::seconds(token.expires_in as i64);

    // Update storage
    storage.access_token = Some(token.access_token);
    if let Some(new_refresh) = token.refresh_token {
        storage.refresh_token = Some(new_refresh);
    }
    storage.expires_at = Some(expires_at.to_rfc3339());
    storage.save()?;

    println!("{} Token refreshed successfully", "✓".green().bold());
    Ok(())
}

/// Register a new OAuth client
pub async fn register_client(name: &str, redirect_uris: &[String], scope: &str) -> Result<()> {
    let storage = TokenStorage::load()?;

    let server = storage.server
        .ok_or("Not logged in. Please log in first with 'sovereign auth login <server>'")?;

    let client = reqwest::Client::new();
    let register_url = format!("{}/oauth/register", server);

    let body = serde_json::json!({
        "client_name": name,
        "redirect_uris": redirect_uris,
        "grant_types": ["authorization_code", "refresh_token"],
        "scope": scope,
    });

    let response = client
        .post(&register_url)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("Client registration failed: {}", error_text).into());
    }

    let client_info: serde_json::Value = response.json().await?;

    println!("{} Client registered successfully", "✓".green().bold());
    println!();
    println!("  Client ID:     {}", client_info.get("client_id")
        .and_then(|v| v.as_str())
        .unwrap_or("").cyan().bold());
    println!("  Client Secret: {}", client_info.get("client_secret")
        .and_then(|v| v.as_str())
        .unwrap_or("").yellow());
    println!();
    println!("{}", "Save these credentials! The client secret will not be shown again.".yellow());

    Ok(())
}

/// List registered OAuth clients
pub async fn list_clients(server: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let server = server.trim_end_matches('/');
    let list_url = format!("{}/oauth/clients", server);

    let response = client
        .get(&list_url)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("Failed to list clients: {}", error_text).into());
    }

    let clients: Vec<serde_json::Value> = response.json().await?;

    if clients.is_empty() {
        println!("No OAuth clients registered");
        return Ok(());
    }

    println!("{}", "Registered OAuth Clients".bold());
    println!();

    for client in clients {
        let id = client.get("client_id").and_then(|v| v.as_str()).unwrap_or("unknown");
        let name = client.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed");
        let scope = client.get("scope").and_then(|v| v.as_str()).unwrap_or("");
        let created = client.get("created_at").and_then(|v| v.as_str()).unwrap_or("");

        println!("  {} ({})", name.cyan().bold(), id);
        println!("    Scope: {}", scope);
        println!("    Created: {}", created);
        println!();
    }

    Ok(())
}

/// Get the current access token (for use by other commands)
pub fn get_access_token() -> Result<Option<String>> {
    let storage = TokenStorage::load()?;
    Ok(storage.access_token)
}

/// Get the current server URL (for use by other commands)
pub fn get_server() -> Result<Option<String>> {
    let storage = TokenStorage::load()?;
    Ok(storage.server)
}
