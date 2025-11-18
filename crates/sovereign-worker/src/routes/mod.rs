pub mod wellknown;
pub mod activitypub;
pub mod atproto;

use worker::*;

/// Helper to return JSON response with correct content type
pub fn json_response<T: serde::Serialize>(data: &T) -> Result<Response> {
    let json = serde_json::to_string(data)
        .map_err(|e| Error::RustError(e.to_string()))?;

    let mut headers = Headers::new();
    headers.set("Content-Type", "application/json")?;

    Ok(Response::ok(json)?.with_headers(headers))
}

/// Helper to return ActivityPub JSON-LD response
pub fn activitypub_response<T: serde::Serialize>(data: &T) -> Result<Response> {
    let json = serde_json::to_string(data)
        .map_err(|e| Error::RustError(e.to_string()))?;

    let mut headers = Headers::new();
    headers.set("Content-Type", "application/activity+json")?;

    Ok(Response::ok(json)?.with_headers(headers))
}

/// Helper to return error response
pub fn error_response(status: u16, message: &str) -> Result<Response> {
    let body = serde_json::json!({
        "error": message
    });

    let mut headers = Headers::new();
    headers.set("Content-Type", "application/json")?;

    let response = Response::ok(body.to_string())?
        .with_headers(headers)
        .with_status(status);

    Ok(response)
}

/// Get the domain from environment or request
pub fn get_domain(env: &Env, req: &Request) -> String {
    // Try environment variable first
    if let Ok(domain) = env.var("DOMAIN") {
        return domain.to_string();
    }

    // Fall back to request host
    req.url()
        .ok()
        .and_then(|url| url.host_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "localhost".to_string())
}

/// Get username from environment (single-user server)
pub fn get_username(env: &Env) -> String {
    env.var("USERNAME")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "user".to_string())
}
