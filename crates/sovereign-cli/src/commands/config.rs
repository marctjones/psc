use colored::Colorize;
use sovereign_core::Config;
use std::fs;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".sovereign")
}

fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load_config() -> Result<Config> {
    let path = config_path();

    if !path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;

    let path = config_path();
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content)?;

    Ok(())
}

pub async fn get(key: &str) -> Result<()> {
    let config = load_config()?;

    let value = match key {
        "domain" => config.domain.clone(),
        "cloudflare_api_token" => config.cloudflare_api_token.clone().map(|_| "***".to_string()),
        "cloudflare_zone_id" => config.cloudflare_zone_id.clone(),
        "handle" => config.handle.clone(),
        "server_url" => config.server_url.clone(),
        _ => {
            return Err(format!("Unknown configuration key: {}", key).into());
        }
    };

    match value {
        Some(v) => println!("{}", v),
        None => println!("{}", "(not set)".dimmed()),
    }

    Ok(())
}

pub async fn set(key: &str, value: &str) -> Result<()> {
    let mut config = load_config()?;

    match key {
        "domain" => config.domain = Some(value.to_string()),
        "cloudflare_api_token" => config.cloudflare_api_token = Some(value.to_string()),
        "cloudflare_zone_id" => config.cloudflare_zone_id = Some(value.to_string()),
        "handle" => config.handle = Some(value.to_string()),
        "server_url" => config.server_url = Some(value.to_string()),
        _ => {
            return Err(format!("Unknown configuration key: {}", key).into());
        }
    };

    save_config(&config)?;
    println!("{} {} = {}", "Set".green(), key, value);

    Ok(())
}

pub async fn list() -> Result<()> {
    let config = load_config()?;

    println!("{}", "Configuration:".bold());
    println!();

    let items = [
        ("domain", config.domain.as_deref()),
        (
            "cloudflare_api_token",
            config.cloudflare_api_token.as_ref().map(|_| "***"),
        ),
        ("cloudflare_zone_id", config.cloudflare_zone_id.as_deref()),
        ("handle", config.handle.as_deref()),
        ("server_url", config.server_url.as_deref()),
    ];

    for (key, value) in items {
        let display = match value {
            Some(v) => v.to_string(),
            None => "(not set)".dimmed().to_string(),
        };
        println!("  {} = {}", key.cyan(), display);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.json");

        let config = Config {
            domain: Some("example.com".to_string()),
            cloudflare_api_token: Some("token123".to_string()),
            cloudflare_zone_id: Some("zone123".to_string()),
            handle: Some("alice".to_string()),
            server_url: Some("https://example.com".to_string()),
        };

        let content = serde_json::to_string_pretty(&config).unwrap();
        fs::write(&config_file, content).unwrap();

        let loaded: Config = serde_json::from_str(&fs::read_to_string(&config_file).unwrap()).unwrap();
        assert_eq!(loaded.domain, Some("example.com".to_string()));
        assert_eq!(loaded.handle, Some("alice".to_string()));
    }
}
