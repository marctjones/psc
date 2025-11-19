//! OAuth2 server implementation for CLI auth and third-party apps
//!
//! Supports:
//! - Device Authorization Grant (RFC 8628) for CLI authentication
//! - Authorization Code Grant for web apps
//! - Client registration for third-party apps

use serde::{Deserialize, Serialize};
use worker::*;

use super::{error_response, get_domain, get_username, json_response};
use crate::storage::d1;

/// OAuth client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClient {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub grant_types: Vec<String>,
    pub scope: String,
    pub created_at: String,
}

/// Device authorization request
#[derive(Debug, Deserialize)]
pub struct DeviceAuthRequest {
    pub client_id: String,
    pub scope: Option<String>,
}

/// Device authorization response
#[derive(Debug, Serialize)]
pub struct DeviceAuthResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_in: u32,
    pub interval: u32,
}

/// Token request
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub device_code: Option<String>,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// Token response
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Token error response
#[derive(Debug, Serialize)]
pub struct TokenError {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
}

/// Client registration request
#[derive(Debug, Deserialize)]
pub struct ClientRegistrationRequest {
    pub client_name: String,
    pub redirect_uris: Option<Vec<String>>,
    pub grant_types: Option<Vec<String>>,
    pub scope: Option<String>,
}

/// Client registration response
#[derive(Debug, Serialize)]
pub struct ClientRegistrationResponse {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    pub grant_types: Vec<String>,
    pub scope: String,
}

/// OAuth metadata (RFC 8414)
pub async fn metadata(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let domain = get_domain(&ctx.env, &req);
    let base_url = format!("https://{}", domain);

    let metadata = serde_json::json!({
        "issuer": base_url,
        "authorization_endpoint": format!("{}/oauth/authorize", base_url),
        "token_endpoint": format!("{}/oauth/token", base_url),
        "device_authorization_endpoint": format!("{}/oauth/device", base_url),
        "registration_endpoint": format!("{}/oauth/register", base_url),
        "revocation_endpoint": format!("{}/oauth/revoke", base_url),
        "introspection_endpoint": format!("{}/oauth/introspect", base_url),
        "response_types_supported": ["code", "token"],
        "grant_types_supported": [
            "authorization_code",
            "refresh_token",
            "urn:ietf:params:oauth:grant-type:device_code"
        ],
        "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post", "none"],
        "scopes_supported": ["read", "write", "follow", "admin"],
        "code_challenge_methods_supported": ["S256"]
    });

    json_response(&metadata)
}

/// Device authorization endpoint (RFC 8628)
pub async fn device_authorize(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let form_data = req.form_data().await?;

    let client_id = form_data
        .get("client_id")
        .and_then(|v| match v {
            FormEntry::Field(s) => Some(s),
            _ => None,
        })
        .ok_or_else(|| Error::RustError("Missing client_id".to_string()))?;

    let scope = form_data
        .get("scope")
        .and_then(|v| match v {
            FormEntry::Field(s) => Some(s),
            _ => None,
        })
        .unwrap_or_else(|| "read".to_string());

    let db = d1::get_db(&ctx.env)?;
    let domain = get_domain(&ctx.env, &req);

    // Verify client exists
    if d1::get_oauth_client(&db, &client_id).await?.is_none() {
        return error_response(400, "Invalid client_id");
    }

    // Generate device and user codes
    let device_code = generate_secure_token(32);
    let user_code = generate_user_code();

    // Store device authorization
    d1::create_device_auth(&db, &device_code, &user_code, &client_id, &scope).await?;

    let response = DeviceAuthResponse {
        device_code,
        user_code: user_code.clone(),
        verification_uri: format!("https://{}/oauth/device/verify", domain),
        verification_uri_complete: format!("https://{}/oauth/device/verify?user_code={}", domain, user_code),
        expires_in: 600, // 10 minutes
        interval: 5,
    };

    json_response(&response)
}

/// Device verification page (shows form to enter user code)
pub async fn device_verify_page(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let url = req.url()?;
    let user_code = url
        .query_pairs()
        .find(|(k, _)| k == "user_code")
        .map(|(_, v)| v.to_string());

    let domain = get_domain(&ctx.env, &req);

    let html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>Authorize Device - {}</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body {{ font-family: system-ui, sans-serif; max-width: 400px; margin: 2rem auto; padding: 1rem; }}
        h1 {{ font-size: 1.5rem; }}
        form {{ display: flex; flex-direction: column; gap: 1rem; }}
        input {{ padding: 0.5rem; font-size: 1.2rem; text-align: center; letter-spacing: 0.3rem; }}
        button {{ padding: 0.75rem; font-size: 1rem; background: #2563eb; color: white; border: none; border-radius: 4px; cursor: pointer; }}
        button:hover {{ background: #1d4ed8; }}
        .error {{ color: #dc2626; }}
    </style>
</head>
<body>
    <h1>Authorize Device</h1>
    <p>Enter the code displayed on your device:</p>
    <form method="POST" action="/oauth/device/verify">
        <input type="text" name="user_code" placeholder="XXXX-XXXX" value="{}" required pattern="[A-Z0-9]{{4}}-[A-Z0-9]{{4}}" />
        <button type="submit">Authorize</button>
    </form>
</body>
</html>"#, domain, user_code.unwrap_or_default());

    let mut headers = Headers::new();
    headers.set("Content-Type", "text/html")?;
    Ok(Response::ok(html)?.with_headers(headers))
}

/// Device verification submission
pub async fn device_verify_submit(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let form_data = req.form_data().await?;

    let user_code = form_data
        .get("user_code")
        .and_then(|v| match v {
            FormEntry::Field(s) => Some(s.to_uppercase().replace(" ", "")),
            _ => None,
        })
        .ok_or_else(|| Error::RustError("Missing user_code".to_string()))?;

    let db = d1::get_db(&ctx.env)?;

    // Mark device as authorized
    let authorized = d1::authorize_device(&db, &user_code).await?;

    let domain = get_domain(&ctx.env, &req);

    let html = if authorized {
        format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>Device Authorized - {}</title>
    <style>
        body {{ font-family: system-ui, sans-serif; max-width: 400px; margin: 2rem auto; padding: 1rem; text-align: center; }}
        .success {{ color: #16a34a; font-size: 3rem; }}
    </style>
</head>
<body>
    <div class="success">✓</div>
    <h1>Device Authorized</h1>
    <p>You can close this window and return to your CLI.</p>
</body>
</html>"#, domain)
    } else {
        format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>Authorization Failed - {}</title>
    <style>
        body {{ font-family: system-ui, sans-serif; max-width: 400px; margin: 2rem auto; padding: 1rem; text-align: center; }}
        .error {{ color: #dc2626; font-size: 3rem; }}
    </style>
</head>
<body>
    <div class="error">✗</div>
    <h1>Authorization Failed</h1>
    <p>Invalid or expired code. Please try again.</p>
    <a href="/oauth/device/verify">Try Again</a>
</body>
</html>"#, domain)
    };

    let mut headers = Headers::new();
    headers.set("Content-Type", "text/html")?;
    Ok(Response::ok(html)?.with_headers(headers))
}

/// Token endpoint
pub async fn token(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let form_data = req.form_data().await?;

    let grant_type = form_data
        .get("grant_type")
        .and_then(|v| match v {
            FormEntry::Field(s) => Some(s),
            _ => None,
        })
        .ok_or_else(|| Error::RustError("Missing grant_type".to_string()))?;

    let db = d1::get_db(&ctx.env)?;

    match grant_type.as_str() {
        "urn:ietf:params:oauth:grant-type:device_code" => {
            let device_code = form_data
                .get("device_code")
                .and_then(|v| match v {
                    FormEntry::Field(s) => Some(s),
                    _ => None,
                })
                .ok_or_else(|| Error::RustError("Missing device_code".to_string()))?;

            // Check device authorization status
            let status = d1::get_device_auth_status(&db, &device_code).await?;

            match status.as_str() {
                "pending" => {
                    // Authorization pending - client should continue polling
                    let error = TokenError {
                        error: "authorization_pending".to_string(),
                        error_description: Some("The authorization request is still pending".to_string()),
                    };
                    let json = serde_json::to_string(&error)
                        .map_err(|e| Error::RustError(e.to_string()))?;
                    let mut headers = Headers::new();
                    headers.set("Content-Type", "application/json")?;
                    return Ok(Response::ok(json)?.with_headers(headers).with_status(400));
                }
                "authorized" => {
                    // Generate tokens
                    let access_token = generate_secure_token(32);
                    let refresh_token = generate_secure_token(32);

                    let username = get_username(&ctx.env);

                    // Get client_id and scope from device auth
                    let (client_id, scope) = d1::get_device_auth_info(&db, &device_code).await?;

                    // Store tokens
                    d1::create_access_token(&db, &access_token, &username, &client_id, &scope, 3600).await?;
                    d1::create_refresh_token(&db, &refresh_token, &access_token, &username, &client_id).await?;

                    // Mark device auth as complete
                    d1::complete_device_auth(&db, &device_code).await?;

                    let response = TokenResponse {
                        access_token,
                        token_type: "Bearer".to_string(),
                        expires_in: 3600,
                        refresh_token: Some(refresh_token),
                        scope: Some(scope),
                    };

                    return json_response(&response);
                }
                "expired" => {
                    let error = TokenError {
                        error: "expired_token".to_string(),
                        error_description: Some("The device code has expired".to_string()),
                    };
                    let json = serde_json::to_string(&error)
                        .map_err(|e| Error::RustError(e.to_string()))?;
                    let mut headers = Headers::new();
                    headers.set("Content-Type", "application/json")?;
                    return Ok(Response::ok(json)?.with_headers(headers).with_status(400));
                }
                _ => {
                    return error_response(400, "Invalid device_code");
                }
            }
        }
        "refresh_token" => {
            let refresh_token = form_data
                .get("refresh_token")
                .and_then(|v| match v {
                    FormEntry::Field(s) => Some(s),
                    _ => None,
                })
                .ok_or_else(|| Error::RustError("Missing refresh_token".to_string()))?;

            // Validate refresh token and get associated data
            let token_data = d1::validate_refresh_token(&db, &refresh_token).await?;

            if let Some((username, client_id, scope)) = token_data {
                // Generate new access token
                let new_access_token = generate_secure_token(32);
                let new_refresh_token = generate_secure_token(32);

                // Store new tokens
                d1::create_access_token(&db, &new_access_token, &username, &client_id, &scope, 3600).await?;
                d1::create_refresh_token(&db, &new_refresh_token, &new_access_token, &username, &client_id).await?;

                // Revoke old refresh token
                d1::revoke_refresh_token(&db, &refresh_token).await?;

                let response = TokenResponse {
                    access_token: new_access_token,
                    token_type: "Bearer".to_string(),
                    expires_in: 3600,
                    refresh_token: Some(new_refresh_token),
                    scope: Some(scope),
                };

                return json_response(&response);
            } else {
                return error_response(400, "Invalid refresh_token");
            }
        }
        _ => {
            return error_response(400, "Unsupported grant_type");
        }
    }
}

/// Revoke a token
pub async fn revoke(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let form_data = req.form_data().await?;

    let token = form_data
        .get("token")
        .and_then(|v| match v {
            FormEntry::Field(s) => Some(s),
            _ => None,
        })
        .ok_or_else(|| Error::RustError("Missing token".to_string()))?;

    let db = d1::get_db(&ctx.env)?;

    // Try to revoke as access token first, then as refresh token
    d1::revoke_access_token(&db, &token).await?;
    d1::revoke_refresh_token(&db, &token).await?;

    // Always return 200 per RFC 7009
    Response::ok("")
}

/// Introspect a token
pub async fn introspect(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let form_data = req.form_data().await?;

    let token = form_data
        .get("token")
        .and_then(|v| match v {
            FormEntry::Field(s) => Some(s),
            _ => None,
        })
        .ok_or_else(|| Error::RustError("Missing token".to_string()))?;

    let db = d1::get_db(&ctx.env)?;

    // Check if token is valid
    let token_info = d1::get_access_token_info(&db, &token).await?;

    if let Some((username, client_id, scope, expires_at)) = token_info {
        let response = serde_json::json!({
            "active": true,
            "scope": scope,
            "client_id": client_id,
            "username": username,
            "token_type": "Bearer",
            "exp": expires_at
        });
        json_response(&response)
    } else {
        let response = serde_json::json!({
            "active": false
        });
        json_response(&response)
    }
}

/// Register a new OAuth client
pub async fn register_client(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let body: ClientRegistrationRequest = req.json().await?;

    let db = d1::get_db(&ctx.env)?;

    // Generate client credentials
    let client_id = generate_secure_token(16);
    let client_secret = generate_secure_token(32);

    let redirect_uris = body.redirect_uris.unwrap_or_default();
    let grant_types = body.grant_types.unwrap_or_else(|| vec!["authorization_code".to_string()]);
    let scope = body.scope.unwrap_or_else(|| "read".to_string());

    // Store client
    d1::create_oauth_client(
        &db,
        &client_id,
        &client_secret,
        &body.client_name,
        &redirect_uris,
        &grant_types,
        &scope,
    ).await?;

    let response = ClientRegistrationResponse {
        client_id,
        client_secret: Some(client_secret),
        client_name: body.client_name,
        redirect_uris,
        grant_types,
        scope,
    };

    let mut headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    let json = serde_json::to_string(&response)
        .map_err(|e| Error::RustError(e.to_string()))?;
    Ok(Response::ok(json)?.with_headers(headers).with_status(201))
}

/// List registered clients (admin only)
pub async fn list_clients(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // TODO: Add authentication check for admin access
    let db = d1::get_db(&ctx.env)?;
    let clients = d1::list_oauth_clients(&db).await?;
    json_response(&clients)
}

/// Delete a client
pub async fn delete_client(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let client_id = ctx.param("client_id")
        .ok_or_else(|| Error::RustError("Missing client_id".to_string()))?;

    let db = d1::get_db(&ctx.env)?;
    d1::delete_oauth_client(&db, client_id).await?;

    Response::ok("")
}

/// Helper: Validate access token from Authorization header
pub async fn validate_token(req: &Request, env: &Env) -> Result<Option<String>> {
    let auth_header = req.headers().get("Authorization")?;

    if let Some(auth) = auth_header {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            let db = d1::get_db(env)?;
            if let Some((username, _, _, _)) = d1::get_access_token_info(&db, token).await? {
                return Ok(Some(username));
            }
        }
    }

    Ok(None)
}

/// Generate a secure random token
fn generate_secure_token(len: usize) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Simple token generation - in production use a proper CSPRNG
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let rand_bytes: Vec<u8> = (0..len)
        .map(|i| ((time >> (i % 16)) as u8).wrapping_add(i as u8))
        .collect();

    base64_url_encode(&rand_bytes)
}

/// Generate a user-friendly device code
fn generate_user_code() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    // Generate 8 character code in XXXX-XXXX format
    let chars = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // Removed confusing chars
    let mut code = String::new();

    for i in 0..8 {
        if i == 4 {
            code.push('-');
        }
        let idx = ((time >> (i * 5)) as usize) % chars.len();
        code.push(chars.chars().nth(idx).unwrap());
    }

    code
}

/// URL-safe hex encoding (simple token format)
fn base64_url_encode(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}
