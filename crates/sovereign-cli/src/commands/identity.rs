use colored::Colorize;
use dialoguer::{Input, Password};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::config::{load_config, save_config, Result};

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareDnsRecord {
    id: Option<String>,
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
    proxied: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct CloudflareResponse<T> {
    success: bool,
    errors: Vec<CloudflareError>,
    result: Option<T>,
}

#[derive(Debug, Deserialize)]
struct CloudflareError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct CloudflareZone {
    id: String,
    name: String,
}

pub async fn setup(domain: &str) -> Result<()> {
    println!("{}", "Setting up Sovereign identity...".bold());
    println!();

    let mut config = load_config()?;

    // Get Cloudflare API token if not set
    let api_token = match &config.cloudflare_api_token {
        Some(token) => token.clone(),
        None => {
            let token: String = Password::new()
                .with_prompt("Cloudflare API Token")
                .interact()?;
            config.cloudflare_api_token = Some(token.clone());
            token
        }
    };

    // Get username/handle
    let handle: String = Input::new()
        .with_prompt("Username (handle)")
        .default(
            config
                .handle
                .clone()
                .unwrap_or_else(|| "user".to_string()),
        )
        .interact_text()?;

    config.handle = Some(handle.clone());
    config.domain = Some(domain.to_string());

    let client = Client::new();
    let pb = ProgressBar::new(4);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Step 1: Find zone ID
    pb.set_message("Finding Cloudflare zone...");
    let zone_id = find_zone_id(&client, &api_token, domain).await?;
    config.cloudflare_zone_id = Some(zone_id.clone());
    pb.inc(1);

    // Step 2: Create A record (pointing to workers)
    pb.set_message("Creating DNS records...");
    // For Workers, we use a CNAME to the workers.dev subdomain
    // For now, we'll create a placeholder
    create_dns_record(
        &client,
        &api_token,
        &zone_id,
        CloudflareDnsRecord {
            id: None,
            record_type: "CNAME".to_string(),
            name: domain.to_string(),
            content: "sovereign.workers.dev".to_string(), // Placeholder
            ttl: 1, // Auto
            proxied: Some(true),
        },
    )
    .await
    .ok(); // Ignore if already exists
    pb.inc(1);

    // Step 3: Create TXT record for AT Protocol
    pb.set_message("Creating AT Protocol verification...");
    let did = format!("did:web:{}", domain);
    create_dns_record(
        &client,
        &api_token,
        &zone_id,
        CloudflareDnsRecord {
            id: None,
            record_type: "TXT".to_string(),
            name: format!("_atproto.{}", domain),
            content: format!("did={}", did),
            ttl: 1,
            proxied: Some(false),
        },
    )
    .await
    .ok();
    pb.inc(1);

    // Step 4: Save configuration
    pb.set_message("Saving configuration...");
    config.server_url = Some(format!("https://{}", domain));
    save_config(&config)?;
    pb.inc(1);

    pb.finish_with_message("Done!");
    println!();
    println!("{}", "Identity setup complete!".green().bold());
    println!();
    println!("  {} {}", "Domain:".cyan(), domain);
    println!("  {} {}", "Handle:".cyan(), format!("@{}", handle));
    println!("  {} {}", "DID:".cyan(), did);
    println!();
    println!(
        "Next step: Run {} to deploy your server.",
        "sovereign server deploy".yellow()
    );

    Ok(())
}

pub async fn cleanup() -> Result<()> {
    println!("{}", "Cleaning up Sovereign identity...".bold());

    let config = load_config()?;

    let api_token = config
        .cloudflare_api_token
        .ok_or("No Cloudflare API token configured")?;
    let zone_id = config
        .cloudflare_zone_id
        .ok_or("No Cloudflare zone ID configured")?;
    let domain = config.domain.ok_or("No domain configured")?;

    let client = Client::new();

    // Delete DNS records
    println!("Deleting DNS records...");

    // Find and delete _atproto TXT record
    if let Ok(records) = list_dns_records(&client, &api_token, &zone_id, Some("TXT")).await {
        for record in records {
            if record.name == format!("_atproto.{}", domain) {
                if let Some(id) = record.id {
                    delete_dns_record(&client, &api_token, &zone_id, &id).await?;
                    println!("  {} TXT _atproto.{}", "Deleted".red(), domain);
                }
            }
        }
    }

    println!();
    println!("{}", "Cleanup complete!".green().bold());
    println!();
    println!(
        "Note: You may need to manually remove the CNAME record and Cloudflare Worker."
    );

    Ok(())
}

pub async fn export(output: Option<&str>) -> Result<()> {
    let config = load_config()?;

    let json = serde_json::to_string_pretty(&config)?;

    match output {
        Some(path) => {
            std::fs::write(path, &json)?;
            println!("{} {}", "Exported to".green(), path);
        }
        None => {
            println!("{}", json);
        }
    }

    Ok(())
}

pub async fn import(file: &str) -> Result<()> {
    let content = std::fs::read_to_string(file)?;
    let config: sovereign_core::Config = serde_json::from_str(&content)?;

    save_config(&config)?;
    println!("{} {}", "Imported from".green(), file);

    Ok(())
}

async fn find_zone_id(client: &Client, api_token: &str, domain: &str) -> Result<String> {
    // Extract root domain (e.g., "example.com" from "sub.example.com")
    let parts: Vec<&str> = domain.split('.').collect();
    let root_domain = if parts.len() >= 2 {
        format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1])
    } else {
        domain.to_string()
    };

    let url = format!(
        "https://api.cloudflare.com/client/v4/zones?name={}",
        root_domain
    );

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(30))
        .send()
        .await?;

    let result: CloudflareResponse<Vec<CloudflareZone>> = response.json().await?;

    if !result.success {
        let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
        return Err(format!("Cloudflare API error: {}", errors.join(", ")).into());
    }

    result
        .result
        .and_then(|zones| zones.into_iter().next())
        .map(|z| z.id)
        .ok_or_else(|| format!("Zone not found for domain: {}", domain).into())
}

async fn create_dns_record(
    client: &Client,
    api_token: &str,
    zone_id: &str,
    record: CloudflareDnsRecord,
) -> Result<()> {
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
        zone_id
    );

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .json(&record)
        .timeout(Duration::from_secs(30))
        .send()
        .await?;

    let result: CloudflareResponse<serde_json::Value> = response.json().await?;

    if !result.success {
        let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
        return Err(format!("Failed to create DNS record: {}", errors.join(", ")).into());
    }

    Ok(())
}

async fn list_dns_records(
    client: &Client,
    api_token: &str,
    zone_id: &str,
    record_type: Option<&str>,
) -> Result<Vec<CloudflareDnsRecord>> {
    let mut url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
        zone_id
    );

    if let Some(t) = record_type {
        url = format!("{}?type={}", url, t);
    }

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(30))
        .send()
        .await?;

    let result: CloudflareResponse<Vec<CloudflareDnsRecord>> = response.json().await?;

    if !result.success {
        let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
        return Err(format!("Failed to list DNS records: {}", errors.join(", ")).into());
    }

    Ok(result.result.unwrap_or_default())
}

async fn delete_dns_record(
    client: &Client,
    api_token: &str,
    zone_id: &str,
    record_id: &str,
) -> Result<()> {
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
        zone_id, record_id
    );

    let response = client
        .delete(&url)
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(30))
        .send()
        .await?;

    let result: CloudflareResponse<serde_json::Value> = response.json().await?;

    if !result.success {
        let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
        return Err(format!("Failed to delete DNS record: {}", errors.join(", ")).into());
    }

    Ok(())
}
