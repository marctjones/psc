use colored::Colorize;
use serde_json::Value;
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// Smoke test results
#[derive(Debug)]
struct TestResult {
    name: String,
    passed: bool,
    message: String,
    duration_ms: u64,
}

/// Run deployment validation tests against a live server
pub async fn validate(base_url: &str, verbose: bool) -> Result<()> {
    println!("{}", "Running deployment validation tests...".cyan().bold());
    println!("Target: {}\n", base_url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let mut results = Vec::new();

    // Test 1: Health check
    results.push(test_health(&client, base_url, verbose).await);

    // Test 2: WebFinger
    results.push(test_webfinger(&client, base_url, verbose).await);

    // Test 3: NodeInfo
    results.push(test_nodeinfo(&client, base_url, verbose).await);

    // Test 4: Actor endpoint
    results.push(test_actor(&client, base_url, verbose).await);

    // Test 5: DID document
    results.push(test_did_document(&client, base_url, verbose).await);

    // Test 6: Outbox
    results.push(test_outbox(&client, base_url, verbose).await);

    // Test 7: AT Protocol describe server
    results.push(test_describe_server(&client, base_url, verbose).await);

    // Print results
    println!("\n{}", "Test Results:".bold());
    println!("{}", "=".repeat(60));

    let mut passed = 0;
    let mut failed = 0;
    let mut total_time = 0;

    for result in &results {
        total_time += result.duration_ms;
        let status = if result.passed {
            passed += 1;
            "PASS".green().bold()
        } else {
            failed += 1;
            "FAIL".red().bold()
        };

        println!(
            "[{}] {} ({}ms)",
            status,
            result.name,
            result.duration_ms
        );

        if !result.passed || verbose {
            println!("     {}", result.message.dimmed());
        }
    }

    println!("{}", "=".repeat(60));
    println!(
        "Total: {} passed, {} failed ({} ms)",
        passed.to_string().green(),
        failed.to_string().red(),
        total_time
    );

    if failed > 0 {
        Err(format!("{} test(s) failed", failed).into())
    } else {
        println!("\n{}", "All tests passed!".green().bold());
        Ok(())
    }
}

/// Run quick connectivity test
pub async fn quick(base_url: &str) -> Result<()> {
    println!("Testing connection to {}...", base_url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let health_url = format!("{}/health", base_url.trim_end_matches('/'));

    match client.get(&health_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            println!("{} Server is reachable", "OK".green().bold());
            Ok(())
        }
        Ok(resp) => {
            Err(format!("Server returned status {}", resp.status()).into())
        }
        Err(e) => {
            Err(format!("Connection failed: {}", e).into())
        }
    }
}

/// Test health endpoint
async fn test_health(client: &reqwest::Client, base_url: &str, _verbose: bool) -> TestResult {
    let url = format!("{}/health", base_url.trim_end_matches('/'));
    let start = std::time::Instant::now();

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => TestResult {
            name: "Health Check".to_string(),
            passed: true,
            message: "Server is healthy".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Ok(resp) => TestResult {
            name: "Health Check".to_string(),
            passed: false,
            message: format!("Unexpected status: {}", resp.status()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => TestResult {
            name: "Health Check".to_string(),
            passed: false,
            message: format!("Request failed: {}", e),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Test WebFinger endpoint
async fn test_webfinger(client: &reqwest::Client, base_url: &str, verbose: bool) -> TestResult {
    let base = base_url.trim_end_matches('/');
    // Extract domain from URL
    let domain = base_url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("localhost");

    let url = format!(
        "{}/.well-known/webfinger?resource=acct:user@{}",
        base, domain
    );
    let start = std::time::Instant::now();

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Value>().await {
                Ok(json) => {
                    // Validate WebFinger structure
                    let has_subject = json.get("subject").is_some();
                    let has_links = json.get("links").and_then(|l| l.as_array()).is_some();

                    if has_subject && has_links {
                        TestResult {
                            name: "WebFinger".to_string(),
                            passed: true,
                            message: if verbose {
                                format!("Subject: {}", json["subject"])
                            } else {
                                "Valid WebFinger response".to_string()
                            },
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    } else {
                        TestResult {
                            name: "WebFinger".to_string(),
                            passed: false,
                            message: "Missing subject or links in response".to_string(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    }
                }
                Err(e) => TestResult {
                    name: "WebFinger".to_string(),
                    passed: false,
                    message: format!("Invalid JSON: {}", e),
                    duration_ms: start.elapsed().as_millis() as u64,
                },
            }
        }
        Ok(resp) => TestResult {
            name: "WebFinger".to_string(),
            passed: false,
            message: format!("Status: {}", resp.status()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => TestResult {
            name: "WebFinger".to_string(),
            passed: false,
            message: format!("Request failed: {}", e),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Test NodeInfo endpoint
async fn test_nodeinfo(client: &reqwest::Client, base_url: &str, verbose: bool) -> TestResult {
    let base = base_url.trim_end_matches('/');
    let url = format!("{}/.well-known/nodeinfo", base);
    let start = std::time::Instant::now();

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Value>().await {
                Ok(json) => {
                    let has_links = json.get("links").and_then(|l| l.as_array()).is_some();

                    if has_links {
                        // Follow the nodeinfo link
                        if let Some(href) = json["links"][0]["href"].as_str() {
                            match client.get(href).send().await {
                                Ok(ni_resp) if ni_resp.status().is_success() => {
                                    match ni_resp.json::<Value>().await {
                                        Ok(ni_json) => {
                                            let software = ni_json["software"]["name"]
                                                .as_str()
                                                .unwrap_or("unknown");
                                            TestResult {
                                                name: "NodeInfo".to_string(),
                                                passed: true,
                                                message: if verbose {
                                                    format!("Software: {}", software)
                                                } else {
                                                    "Valid NodeInfo response".to_string()
                                                },
                                                duration_ms: start.elapsed().as_millis() as u64,
                                            }
                                        }
                                        Err(e) => TestResult {
                                            name: "NodeInfo".to_string(),
                                            passed: false,
                                            message: format!("Invalid NodeInfo JSON: {}", e),
                                            duration_ms: start.elapsed().as_millis() as u64,
                                        },
                                    }
                                }
                                _ => TestResult {
                                    name: "NodeInfo".to_string(),
                                    passed: false,
                                    message: "Failed to fetch NodeInfo document".to_string(),
                                    duration_ms: start.elapsed().as_millis() as u64,
                                },
                            }
                        } else {
                            TestResult {
                                name: "NodeInfo".to_string(),
                                passed: false,
                                message: "No href in nodeinfo links".to_string(),
                                duration_ms: start.elapsed().as_millis() as u64,
                            }
                        }
                    } else {
                        TestResult {
                            name: "NodeInfo".to_string(),
                            passed: false,
                            message: "Missing links in well-known response".to_string(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    }
                }
                Err(e) => TestResult {
                    name: "NodeInfo".to_string(),
                    passed: false,
                    message: format!("Invalid JSON: {}", e),
                    duration_ms: start.elapsed().as_millis() as u64,
                },
            }
        }
        Ok(resp) => TestResult {
            name: "NodeInfo".to_string(),
            passed: false,
            message: format!("Status: {}", resp.status()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => TestResult {
            name: "NodeInfo".to_string(),
            passed: false,
            message: format!("Request failed: {}", e),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Test Actor endpoint
async fn test_actor(client: &reqwest::Client, base_url: &str, verbose: bool) -> TestResult {
    let base = base_url.trim_end_matches('/');
    let url = format!("{}/users/user", base);
    let start = std::time::Instant::now();

    match client
        .get(&url)
        .header("Accept", "application/activity+json")
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Value>().await {
                Ok(json) => {
                    let has_type = json.get("type").is_some();
                    let has_inbox = json.get("inbox").is_some();
                    let has_outbox = json.get("outbox").is_some();

                    if has_type && has_inbox && has_outbox {
                        TestResult {
                            name: "Actor Document".to_string(),
                            passed: true,
                            message: if verbose {
                                format!(
                                    "Type: {}, Inbox: {}",
                                    json["type"], json["inbox"]
                                )
                            } else {
                                "Valid Actor document".to_string()
                            },
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    } else {
                        TestResult {
                            name: "Actor Document".to_string(),
                            passed: false,
                            message: "Missing type, inbox, or outbox".to_string(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    }
                }
                Err(e) => TestResult {
                    name: "Actor Document".to_string(),
                    passed: false,
                    message: format!("Invalid JSON: {}", e),
                    duration_ms: start.elapsed().as_millis() as u64,
                },
            }
        }
        Ok(resp) => TestResult {
            name: "Actor Document".to_string(),
            passed: false,
            message: format!("Status: {}", resp.status()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => TestResult {
            name: "Actor Document".to_string(),
            passed: false,
            message: format!("Request failed: {}", e),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Test DID document endpoint
async fn test_did_document(client: &reqwest::Client, base_url: &str, verbose: bool) -> TestResult {
    let base = base_url.trim_end_matches('/');
    let url = format!("{}/.well-known/did.json", base);
    let start = std::time::Instant::now();

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Value>().await {
                Ok(json) => {
                    let has_id = json.get("id").is_some();
                    let has_verification = json.get("verificationMethod").is_some();

                    if has_id && has_verification {
                        TestResult {
                            name: "DID Document".to_string(),
                            passed: true,
                            message: if verbose {
                                format!("DID: {}", json["id"])
                            } else {
                                "Valid DID document".to_string()
                            },
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    } else {
                        TestResult {
                            name: "DID Document".to_string(),
                            passed: false,
                            message: "Missing id or verificationMethod".to_string(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    }
                }
                Err(e) => TestResult {
                    name: "DID Document".to_string(),
                    passed: false,
                    message: format!("Invalid JSON: {}", e),
                    duration_ms: start.elapsed().as_millis() as u64,
                },
            }
        }
        Ok(resp) => TestResult {
            name: "DID Document".to_string(),
            passed: false,
            message: format!("Status: {}", resp.status()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => TestResult {
            name: "DID Document".to_string(),
            passed: false,
            message: format!("Request failed: {}", e),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Test Outbox endpoint
async fn test_outbox(client: &reqwest::Client, base_url: &str, verbose: bool) -> TestResult {
    let base = base_url.trim_end_matches('/');
    let url = format!("{}/users/user/outbox", base);
    let start = std::time::Instant::now();

    match client
        .get(&url)
        .header("Accept", "application/activity+json")
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Value>().await {
                Ok(json) => {
                    let is_collection = json["type"]
                        .as_str()
                        .map(|t| t.contains("Collection"))
                        .unwrap_or(false);

                    if is_collection {
                        TestResult {
                            name: "Outbox".to_string(),
                            passed: true,
                            message: if verbose {
                                format!("Type: {}, Items: {}", json["type"], json["totalItems"])
                            } else {
                                "Valid Outbox collection".to_string()
                            },
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    } else {
                        TestResult {
                            name: "Outbox".to_string(),
                            passed: false,
                            message: "Not a valid Collection type".to_string(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    }
                }
                Err(e) => TestResult {
                    name: "Outbox".to_string(),
                    passed: false,
                    message: format!("Invalid JSON: {}", e),
                    duration_ms: start.elapsed().as_millis() as u64,
                },
            }
        }
        Ok(resp) => TestResult {
            name: "Outbox".to_string(),
            passed: false,
            message: format!("Status: {}", resp.status()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => TestResult {
            name: "Outbox".to_string(),
            passed: false,
            message: format!("Request failed: {}", e),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Test AT Protocol describe server endpoint
async fn test_describe_server(client: &reqwest::Client, base_url: &str, verbose: bool) -> TestResult {
    let base = base_url.trim_end_matches('/');
    let url = format!("{}/xrpc/com.atproto.server.describeServer", base);
    let start = std::time::Instant::now();

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Value>().await {
                Ok(json) => {
                    let has_did = json.get("did").is_some();
                    let has_available = json.get("availableUserDomains").is_some();

                    if has_did {
                        TestResult {
                            name: "AT Protocol Server".to_string(),
                            passed: true,
                            message: if verbose {
                                format!("DID: {}", json["did"])
                            } else {
                                "Valid server description".to_string()
                            },
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    } else {
                        TestResult {
                            name: "AT Protocol Server".to_string(),
                            passed: has_available, // Partial pass if has domains
                            message: if has_available {
                                "Server description present (no DID)".to_string()
                            } else {
                                "Missing did and availableUserDomains".to_string()
                            },
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    }
                }
                Err(e) => TestResult {
                    name: "AT Protocol Server".to_string(),
                    passed: false,
                    message: format!("Invalid JSON: {}", e),
                    duration_ms: start.elapsed().as_millis() as u64,
                },
            }
        }
        Ok(resp) => TestResult {
            name: "AT Protocol Server".to_string(),
            passed: false,
            message: format!("Status: {}", resp.status()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => TestResult {
            name: "AT Protocol Server".to_string(),
            passed: false,
            message: format!("Request failed: {}", e),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_display() {
        let result = TestResult {
            name: "Test".to_string(),
            passed: true,
            message: "OK".to_string(),
            duration_ms: 100,
        };
        assert!(result.passed);
        assert_eq!(result.duration_ms, 100);
    }
}
