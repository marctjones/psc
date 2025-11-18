use colored::Colorize;
use std::process::Command;

use super::config::{load_config, Result};

pub async fn deploy() -> Result<()> {
    println!("{}", "Deploying Sovereign to Cloudflare Workers...".bold());
    println!();

    let config = load_config()?;

    if !config.is_configured() {
        return Err("Not configured. Run 'sovereign identity setup <domain>' first.".into());
    }

    // Check if wrangler is installed
    let wrangler_check = Command::new("wrangler").arg("--version").output();

    if wrangler_check.is_err() {
        println!("{}", "Wrangler CLI not found. Installing...".yellow());
        let status = Command::new("npm")
            .args(["install", "-g", "wrangler"])
            .status()?;

        if !status.success() {
            return Err("Failed to install wrangler. Please install it manually: npm install -g wrangler".into());
        }
    }

    // Run wrangler deploy
    println!("Running wrangler deploy...");
    let status = Command::new("wrangler").arg("deploy").status()?;

    if status.success() {
        println!();
        println!("{}", "Deployment successful!".green().bold());
        println!();
        if let Some(url) = config.server_url {
            println!("  {} {}", "Server URL:".cyan(), url);
        }
    } else {
        return Err("Deployment failed. Check wrangler output above.".into());
    }

    Ok(())
}

pub async fn status() -> Result<()> {
    let config = load_config()?;

    println!("{}", "Server Status".bold());
    println!();

    let server_url = config
        .server_url
        .ok_or("No server URL configured")?;

    // Try to reach the health endpoint
    let client = reqwest::Client::new();
    let health_url = format!("{}/health", server_url);

    println!("  {} {}", "URL:".cyan(), server_url);

    match client.get(&health_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                println!("  {} {}", "Status:".cyan(), "Online".green());
            } else {
                println!(
                    "  {} {} ({})",
                    "Status:".cyan(),
                    "Error".red(),
                    response.status()
                );
            }
        }
        Err(e) => {
            println!("  {} {} ({})", "Status:".cyan(), "Offline".red(), e);
        }
    }

    Ok(())
}

pub async fn logs(lines: usize) -> Result<()> {
    println!("{}", "Fetching logs...".bold());
    println!();

    // Run wrangler tail
    let status = Command::new("wrangler")
        .args(["tail", "--format", "pretty"])
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(_) => Err("Failed to fetch logs".into()),
        Err(e) => {
            println!(
                "{}",
                "Note: 'wrangler tail' requires wrangler to be logged in.".yellow()
            );
            println!("Run 'wrangler login' to authenticate.");
            Err(e.into())
        }
    }
}
