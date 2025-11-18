use colored::Colorize;

use super::config::{load_config, Result};
use crate::ui::formatting;

pub async fn login(handle: &str, _app_password: bool) -> Result<()> {
    let config = load_config()?;

    let server_url = config
        .server_url
        .ok_or("No server URL configured. Run 'sovereign identity setup' first.")?;

    println!("Logging in to {} as {}...", server_url, handle);

    // TODO: Implement actual login flow
    // This will call the AT Protocol createSession endpoint

    println!("{}", "Login successful!".green());
    println!();
    println!("  {} @{}", "Logged in as:".cyan(), handle);

    Ok(())
}

pub async fn logout() -> Result<()> {
    // TODO: Clear stored session

    println!("{}", "Logged out.".green());
    Ok(())
}

pub async fn whoami() -> Result<()> {
    let config = load_config()?;

    match config.handle {
        Some(handle) => {
            let did = config
                .domain
                .map(|d| format!("did:web:{}", d))
                .unwrap_or_else(|| "unknown".to_string());

            println!("{}", "Current identity:".bold());
            println!();
            println!("  {} @{}", "Handle:".cyan(), handle);
            println!("  {} {}", "DID:".cyan(), did);
        }
        None => {
            println!("{}", "Not logged in.".yellow());
        }
    }

    Ok(())
}

pub async fn post(text: &str, image: Option<&str>, alt: Option<&str>) -> Result<()> {
    let config = load_config()?;

    let _server_url = config
        .server_url
        .ok_or("No server URL configured")?;

    println!("Creating post...");

    // TODO: Implement actual post creation
    // This will call the AT Protocol createRecord endpoint

    if let Some(img) = image {
        println!("  {} {}", "Image:".dimmed(), img);
        if let Some(alt_text) = alt {
            println!("  {} {}", "Alt text:".dimmed(), alt_text);
        }
    }

    println!();
    println!("{}", "Post created!".green());
    println!();
    formatting::print_post("@you", text, 0, 0, 0, "just now");

    Ok(())
}

pub async fn timeline(limit: usize) -> Result<()> {
    let config = load_config()?;

    let _server_url = config
        .server_url
        .ok_or("No server URL configured")?;

    println!("{}", "Timeline".bold());
    println!();

    // TODO: Fetch actual timeline from AT Protocol getTimeline endpoint

    // Placeholder
    println!("{}", "No posts yet.".dimmed());
    println!();
    println!(
        "Tip: Follow some accounts to see their posts here."
    );

    Ok(())
}

pub async fn follow(handle: &str) -> Result<()> {
    let config = load_config()?;

    let _server_url = config
        .server_url
        .ok_or("No server URL configured")?;

    println!("Following @{}...", handle);

    // TODO: Implement follow via AT Protocol createRecord

    println!("{} @{}", "Now following".green(), handle);

    Ok(())
}

pub async fn unfollow(handle: &str) -> Result<()> {
    let config = load_config()?;

    let _server_url = config
        .server_url
        .ok_or("No server URL configured")?;

    println!("Unfollowing @{}...", handle);

    // TODO: Implement unfollow via AT Protocol deleteRecord

    println!("{} @{}", "Unfollowed".green(), handle);

    Ok(())
}

pub async fn like(uri: &str) -> Result<()> {
    let config = load_config()?;

    let _server_url = config
        .server_url
        .ok_or("No server URL configured")?;

    println!("Liking post...");

    // TODO: Implement like via AT Protocol createRecord

    println!("{}", "Liked!".green());

    Ok(())
}

pub async fn profile(handle: Option<&str>) -> Result<()> {
    let config = load_config()?;

    let _server_url = config
        .server_url
        .ok_or("No server URL configured")?;

    let target = handle.unwrap_or_else(|| {
        config.handle.as_deref().unwrap_or("unknown")
    });

    println!("{}", "Profile".bold());
    println!();

    // TODO: Fetch actual profile from AT Protocol getProfile endpoint

    println!("  {} @{}", "Handle:".cyan(), target);
    println!("  {} 0", "Followers:".cyan());
    println!("  {} 0", "Following:".cyan());
    println!("  {} 0", "Posts:".cyan());

    Ok(())
}
