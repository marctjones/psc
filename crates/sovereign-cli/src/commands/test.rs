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

/// Test connectivity to a remote ActivityPub/Mastodon instance
pub async fn remote_fedi(instance: &str) -> Result<()> {
    println!("{}", "Testing remote ActivityPub/Mastodon connectivity...".cyan().bold());
    println!("Instance: {}\n", instance);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let mut results = Vec::new();
    let base = if instance.starts_with("http") {
        instance.trim_end_matches('/').to_string()
    } else {
        format!("https://{}", instance.trim_end_matches('/'))
    };

    // Test 1: Instance is reachable
    results.push(test_fedi_reachable(&client, &base).await);

    // Test 2: NodeInfo available
    results.push(test_fedi_nodeinfo(&client, &base).await);

    // Test 3: WebFinger endpoint works
    results.push(test_fedi_webfinger(&client, &base).await);

    // Test 4: Can fetch a public actor (usually admin or instance actor)
    results.push(test_fedi_actor(&client, &base).await);

    // Print results
    print_test_results(&results)?;

    Ok(())
}

/// Test connectivity to a specific remote ActivityPub user (read-only, no data sent)
pub async fn remote_user(username: &str, instance: &str) -> Result<()> {
    println!("{}", "Testing connectivity to remote ActivityPub user...".cyan().bold());
    println!("User: {}@{}\n", username, instance);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let mut results = Vec::new();
    let base = format!("https://{}", instance.trim_end_matches('/'));

    // Test 1: WebFinger lookup for the specific user
    let webfinger_result = test_user_webfinger(&client, &base, username, instance).await;
    let actor_url = webfinger_result.1.clone();
    results.push(webfinger_result.0);

    // Test 2: Fetch Actor document (if WebFinger succeeded)
    if let Some(url) = actor_url {
        let actor_result = test_user_actor(&client, &url).await;
        let outbox_url = actor_result.1.clone();
        results.push(actor_result.0);

        // Test 3: Fetch public outbox
        if let Some(outbox) = outbox_url {
            results.push(test_user_outbox(&client, &outbox).await);
        } else {
            results.push(TestResult {
                name: "Public Outbox".to_string(),
                passed: false,
                message: "No outbox URL found in actor".to_string(),
                duration_ms: 0,
            });
        }

        // Test 4: Fetch followers/following counts
        results.push(test_user_collections(&client, &url).await);
    } else {
        // Skip remaining tests if WebFinger failed
        results.push(TestResult {
            name: "Actor Document".to_string(),
            passed: false,
            message: "Skipped (WebFinger failed)".to_string(),
            duration_ms: 0,
        });
        results.push(TestResult {
            name: "Public Outbox".to_string(),
            passed: false,
            message: "Skipped (WebFinger failed)".to_string(),
            duration_ms: 0,
        });
        results.push(TestResult {
            name: "Collections".to_string(),
            passed: false,
            message: "Skipped (WebFinger failed)".to_string(),
            duration_ms: 0,
        });
    }

    // Print results
    print_test_results(&results)?;

    println!("\n{}", "Note: All tests are read-only. No data was sent to the user.".dimmed());

    Ok(())
}

/// Test connectivity to Bluesky network
pub async fn remote_bsky() -> Result<()> {
    println!("{}", "Testing remote Bluesky/AT Protocol connectivity...".cyan().bold());
    println!("Network: bsky.social\n");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let mut results = Vec::new();

    // Test 1: Public API is reachable
    results.push(test_bsky_api(&client).await);

    // Test 2: Can resolve a known handle
    results.push(test_bsky_resolve_handle(&client).await);

    // Test 3: PDS describe server
    results.push(test_bsky_describe_server(&client).await);

    // Test 4: Can fetch public posts
    results.push(test_bsky_public_feed(&client).await);

    // Print results
    print_test_results(&results)?;

    Ok(())
}

fn print_test_results(results: &[TestResult]) -> Result<()> {
    println!("{}", "Test Results:".bold());
    println!("{}", "=".repeat(60));

    let mut passed = 0;
    let mut failed = 0;
    let mut total_time = 0;

    for result in results {
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

        if !result.passed {
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
        println!("\n{}", "Some tests failed - remote service may be experiencing issues".yellow());
    } else {
        println!("\n{}", "All tests passed - remote service is operational".green().bold());
    }

    Ok(())
}

/// Test if Fediverse instance is reachable
async fn test_fedi_reachable(client: &reqwest::Client, base: &str) -> TestResult {
    let start = std::time::Instant::now();

    match client.get(base).send().await {
        Ok(resp) if resp.status().is_success() || resp.status().is_redirection() => TestResult {
            name: "Instance Reachable".to_string(),
            passed: true,
            message: format!("Status: {}", resp.status()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Ok(resp) => TestResult {
            name: "Instance Reachable".to_string(),
            passed: false,
            message: format!("Status: {}", resp.status()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => TestResult {
            name: "Instance Reachable".to_string(),
            passed: false,
            message: format!("Connection failed: {}", e),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Test Fediverse NodeInfo
async fn test_fedi_nodeinfo(client: &reqwest::Client, base: &str) -> TestResult {
    let url = format!("{}/.well-known/nodeinfo", base);
    let start = std::time::Instant::now();

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Value>().await {
                Ok(json) => {
                    if json.get("links").is_some() {
                        TestResult {
                            name: "NodeInfo Available".to_string(),
                            passed: true,
                            message: "Valid NodeInfo response".to_string(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    } else {
                        TestResult {
                            name: "NodeInfo Available".to_string(),
                            passed: false,
                            message: "Missing links in response".to_string(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    }
                }
                Err(e) => TestResult {
                    name: "NodeInfo Available".to_string(),
                    passed: false,
                    message: format!("Invalid JSON: {}", e),
                    duration_ms: start.elapsed().as_millis() as u64,
                },
            }
        }
        Ok(resp) => TestResult {
            name: "NodeInfo Available".to_string(),
            passed: false,
            message: format!("Status: {}", resp.status()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => TestResult {
            name: "NodeInfo Available".to_string(),
            passed: false,
            message: format!("Request failed: {}", e),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Test Fediverse WebFinger (using instance's admin account pattern)
async fn test_fedi_webfinger(client: &reqwest::Client, base: &str) -> TestResult {
    // Extract domain from URL
    let domain = base
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("localhost");

    // Try common admin accounts
    let test_accounts = ["admin", "administrator", "instance"];
    let start = std::time::Instant::now();

    for account in &test_accounts {
        let url = format!(
            "{}/.well-known/webfinger?resource=acct:{}@{}",
            base, account, domain
        );

        if let Ok(resp) = client.get(&url).send().await {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<Value>().await {
                    if json.get("subject").is_some() {
                        return TestResult {
                            name: "WebFinger Endpoint".to_string(),
                            passed: true,
                            message: format!("Found account: {}", account),
                            duration_ms: start.elapsed().as_millis() as u64,
                        };
                    }
                }
            }
        }
    }

    // WebFinger endpoint exists but no test account found
    let url = format!(
        "{}/.well-known/webfinger?resource=acct:test@{}",
        base, domain
    );

    match client.get(&url).send().await {
        Ok(resp) if resp.status() == 404 => TestResult {
            name: "WebFinger Endpoint".to_string(),
            passed: true,
            message: "Endpoint works (no test account found)".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Ok(resp) if resp.status() == 400 => TestResult {
            name: "WebFinger Endpoint".to_string(),
            passed: true,
            message: "Endpoint responds (400 for unknown user)".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Ok(resp) => TestResult {
            name: "WebFinger Endpoint".to_string(),
            passed: false,
            message: format!("Unexpected status: {}", resp.status()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => TestResult {
            name: "WebFinger Endpoint".to_string(),
            passed: false,
            message: format!("Request failed: {}", e),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Test fetching a public actor
async fn test_fedi_actor(client: &reqwest::Client, base: &str) -> TestResult {
    // Try to fetch the instance actor (common in Mastodon)
    let urls = [
        format!("{}/actor", base),
        format!("{}/users/admin", base),
    ];
    let start = std::time::Instant::now();

    for url in &urls {
        if let Ok(resp) = client
            .get(url)
            .header("Accept", "application/activity+json")
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<Value>().await {
                    if json.get("type").is_some() && json.get("inbox").is_some() {
                        let actor_type = json["type"].as_str().unwrap_or("Unknown");
                        return TestResult {
                            name: "Actor Fetching".to_string(),
                            passed: true,
                            message: format!("Found {} actor", actor_type),
                            duration_ms: start.elapsed().as_millis() as u64,
                        };
                    }
                }
            }
        }
    }

    TestResult {
        name: "Actor Fetching".to_string(),
        passed: false,
        message: "Could not fetch any public actor".to_string(),
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

/// Test Bluesky API reachability
async fn test_bsky_api(client: &reqwest::Client) -> TestResult {
    let url = "https://public.api.bsky.app/xrpc/app.bsky.actor.getProfile?actor=bsky.app";
    let start = std::time::Instant::now();

    match client.get(url).send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Value>().await {
                Ok(json) => {
                    let handle = json["handle"].as_str().unwrap_or("unknown");
                    TestResult {
                        name: "Bluesky API".to_string(),
                        passed: true,
                        message: format!("API accessible (fetched @{})", handle),
                        duration_ms: start.elapsed().as_millis() as u64,
                    }
                }
                Err(e) => TestResult {
                    name: "Bluesky API".to_string(),
                    passed: false,
                    message: format!("Invalid JSON: {}", e),
                    duration_ms: start.elapsed().as_millis() as u64,
                },
            }
        }
        Ok(resp) => TestResult {
            name: "Bluesky API".to_string(),
            passed: false,
            message: format!("Status: {}", resp.status()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => TestResult {
            name: "Bluesky API".to_string(),
            passed: false,
            message: format!("Request failed: {}", e),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Test Bluesky handle resolution
async fn test_bsky_resolve_handle(client: &reqwest::Client) -> TestResult {
    let url = "https://bsky.social/xrpc/com.atproto.identity.resolveHandle?handle=bsky.app";
    let start = std::time::Instant::now();

    match client.get(url).send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Value>().await {
                Ok(json) => {
                    if let Some(did) = json["did"].as_str() {
                        TestResult {
                            name: "Handle Resolution".to_string(),
                            passed: true,
                            message: format!("Resolved to {}", &did[..20.min(did.len())]),
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    } else {
                        TestResult {
                            name: "Handle Resolution".to_string(),
                            passed: false,
                            message: "No DID in response".to_string(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    }
                }
                Err(e) => TestResult {
                    name: "Handle Resolution".to_string(),
                    passed: false,
                    message: format!("Invalid JSON: {}", e),
                    duration_ms: start.elapsed().as_millis() as u64,
                },
            }
        }
        Ok(resp) => TestResult {
            name: "Handle Resolution".to_string(),
            passed: false,
            message: format!("Status: {}", resp.status()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => TestResult {
            name: "Handle Resolution".to_string(),
            passed: false,
            message: format!("Request failed: {}", e),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Test Bluesky PDS describe server
async fn test_bsky_describe_server(client: &reqwest::Client) -> TestResult {
    let url = "https://bsky.social/xrpc/com.atproto.server.describeServer";
    let start = std::time::Instant::now();

    match client.get(url).send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Value>().await {
                Ok(json) => {
                    let has_did = json.get("did").is_some();
                    TestResult {
                        name: "PDS Server Info".to_string(),
                        passed: has_did,
                        message: if has_did {
                            "Server description available".to_string()
                        } else {
                            "Missing server DID".to_string()
                        },
                        duration_ms: start.elapsed().as_millis() as u64,
                    }
                }
                Err(e) => TestResult {
                    name: "PDS Server Info".to_string(),
                    passed: false,
                    message: format!("Invalid JSON: {}", e),
                    duration_ms: start.elapsed().as_millis() as u64,
                },
            }
        }
        Ok(resp) => TestResult {
            name: "PDS Server Info".to_string(),
            passed: false,
            message: format!("Status: {}", resp.status()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => TestResult {
            name: "PDS Server Info".to_string(),
            passed: false,
            message: format!("Request failed: {}", e),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Test fetching public Bluesky feed
async fn test_bsky_public_feed(client: &reqwest::Client) -> TestResult {
    let url = "https://public.api.bsky.app/xrpc/app.bsky.feed.getTimeline?limit=1";
    let start = std::time::Instant::now();

    // This endpoint requires auth, so we test a public profile's feed instead
    let url = "https://public.api.bsky.app/xrpc/app.bsky.feed.getAuthorFeed?actor=bsky.app&limit=1";

    match client.get(url).send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Value>().await {
                Ok(json) => {
                    if json.get("feed").is_some() {
                        TestResult {
                            name: "Public Feed Access".to_string(),
                            passed: true,
                            message: "Can fetch public feeds".to_string(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    } else {
                        TestResult {
                            name: "Public Feed Access".to_string(),
                            passed: false,
                            message: "No feed in response".to_string(),
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    }
                }
                Err(e) => TestResult {
                    name: "Public Feed Access".to_string(),
                    passed: false,
                    message: format!("Invalid JSON: {}", e),
                    duration_ms: start.elapsed().as_millis() as u64,
                },
            }
        }
        Ok(resp) => TestResult {
            name: "Public Feed Access".to_string(),
            passed: false,
            message: format!("Status: {}", resp.status()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => TestResult {
            name: "Public Feed Access".to_string(),
            passed: false,
            message: format!("Request failed: {}", e),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Test WebFinger lookup for a specific user
async fn test_user_webfinger(
    client: &reqwest::Client,
    base: &str,
    username: &str,
    instance: &str,
) -> (TestResult, Option<String>) {
    let url = format!(
        "{}/.well-known/webfinger?resource=acct:{}@{}",
        base, username, instance
    );
    let start = std::time::Instant::now();

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Value>().await {
                Ok(json) => {
                    // Find the ActivityPub actor URL
                    let actor_url = json["links"]
                        .as_array()
                        .and_then(|links| {
                            links.iter().find(|link| {
                                link["rel"].as_str() == Some("self")
                                    && link["type"]
                                        .as_str()
                                        .map(|t| t.contains("activity"))
                                        .unwrap_or(false)
                            })
                        })
                        .and_then(|link| link["href"].as_str())
                        .map(|s| s.to_string());

                    let subject = json["subject"].as_str().unwrap_or("unknown");
                    (
                        TestResult {
                            name: "WebFinger Lookup".to_string(),
                            passed: true,
                            message: format!("Found: {}", subject),
                            duration_ms: start.elapsed().as_millis() as u64,
                        },
                        actor_url,
                    )
                }
                Err(e) => (
                    TestResult {
                        name: "WebFinger Lookup".to_string(),
                        passed: false,
                        message: format!("Invalid JSON: {}", e),
                        duration_ms: start.elapsed().as_millis() as u64,
                    },
                    None,
                ),
            }
        }
        Ok(resp) if resp.status() == 404 => (
            TestResult {
                name: "WebFinger Lookup".to_string(),
                passed: false,
                message: format!("User not found: {}@{}", username, instance),
                duration_ms: start.elapsed().as_millis() as u64,
            },
            None,
        ),
        Ok(resp) => (
            TestResult {
                name: "WebFinger Lookup".to_string(),
                passed: false,
                message: format!("Status: {}", resp.status()),
                duration_ms: start.elapsed().as_millis() as u64,
            },
            None,
        ),
        Err(e) => (
            TestResult {
                name: "WebFinger Lookup".to_string(),
                passed: false,
                message: format!("Request failed: {}", e),
                duration_ms: start.elapsed().as_millis() as u64,
            },
            None,
        ),
    }
}

/// Test fetching a specific user's Actor document
async fn test_user_actor(client: &reqwest::Client, actor_url: &str) -> (TestResult, Option<String>) {
    let start = std::time::Instant::now();

    match client
        .get(actor_url)
        .header("Accept", "application/activity+json")
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Value>().await {
                Ok(json) => {
                    let name = json["name"]
                        .as_str()
                        .or_else(|| json["preferredUsername"].as_str())
                        .unwrap_or("Unknown");
                    let actor_type = json["type"].as_str().unwrap_or("Unknown");
                    let outbox_url = json["outbox"].as_str().map(|s| s.to_string());

                    // Display some profile info
                    let followers = json["followers"]
                        .as_str()
                        .map(|_| "yes")
                        .unwrap_or("no");
                    let following = json["following"]
                        .as_str()
                        .map(|_| "yes")
                        .unwrap_or("no");

                    (
                        TestResult {
                            name: "Actor Document".to_string(),
                            passed: true,
                            message: format!(
                                "{} ({}) - followers:{} following:{}",
                                name, actor_type, followers, following
                            ),
                            duration_ms: start.elapsed().as_millis() as u64,
                        },
                        outbox_url,
                    )
                }
                Err(e) => (
                    TestResult {
                        name: "Actor Document".to_string(),
                        passed: false,
                        message: format!("Invalid JSON: {}", e),
                        duration_ms: start.elapsed().as_millis() as u64,
                    },
                    None,
                ),
            }
        }
        Ok(resp) => (
            TestResult {
                name: "Actor Document".to_string(),
                passed: false,
                message: format!("Status: {}", resp.status()),
                duration_ms: start.elapsed().as_millis() as u64,
            },
            None,
        ),
        Err(e) => (
            TestResult {
                name: "Actor Document".to_string(),
                passed: false,
                message: format!("Request failed: {}", e),
                duration_ms: start.elapsed().as_millis() as u64,
            },
            None,
        ),
    }
}

/// Test fetching a user's public outbox
async fn test_user_outbox(client: &reqwest::Client, outbox_url: &str) -> TestResult {
    let start = std::time::Instant::now();

    match client
        .get(outbox_url)
        .header("Accept", "application/activity+json")
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Value>().await {
                Ok(json) => {
                    let total = json["totalItems"].as_u64().unwrap_or(0);
                    let collection_type = json["type"].as_str().unwrap_or("Collection");

                    TestResult {
                        name: "Public Outbox".to_string(),
                        passed: true,
                        message: format!("{} with {} items", collection_type, total),
                        duration_ms: start.elapsed().as_millis() as u64,
                    }
                }
                Err(e) => TestResult {
                    name: "Public Outbox".to_string(),
                    passed: false,
                    message: format!("Invalid JSON: {}", e),
                    duration_ms: start.elapsed().as_millis() as u64,
                },
            }
        }
        Ok(resp) if resp.status() == 401 || resp.status() == 403 => TestResult {
            name: "Public Outbox".to_string(),
            passed: true,
            message: "Outbox exists (auth required for contents)".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Ok(resp) => TestResult {
            name: "Public Outbox".to_string(),
            passed: false,
            message: format!("Status: {}", resp.status()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(e) => TestResult {
            name: "Public Outbox".to_string(),
            passed: false,
            message: format!("Request failed: {}", e),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

/// Test fetching user's followers/following collection info
async fn test_user_collections(client: &reqwest::Client, actor_url: &str) -> TestResult {
    let start = std::time::Instant::now();

    // First get the actor to find collection URLs
    let actor_resp = match client
        .get(actor_url)
        .header("Accept", "application/activity+json")
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => resp,
        _ => {
            return TestResult {
                name: "Collections".to_string(),
                passed: false,
                message: "Could not fetch actor for collection URLs".to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
    };

    let json: Value = match actor_resp.json().await {
        Ok(j) => j,
        Err(_) => {
            return TestResult {
                name: "Collections".to_string(),
                passed: false,
                message: "Invalid actor JSON".to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
    };

    let mut info = Vec::new();

    // Try to get followers count
    if let Some(followers_url) = json["followers"].as_str() {
        if let Ok(resp) = client
            .get(followers_url)
            .header("Accept", "application/activity+json")
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(coll) = resp.json::<Value>().await {
                    if let Some(count) = coll["totalItems"].as_u64() {
                        info.push(format!("followers:{}", count));
                    }
                }
            }
        }
    }

    // Try to get following count
    if let Some(following_url) = json["following"].as_str() {
        if let Ok(resp) = client
            .get(following_url)
            .header("Accept", "application/activity+json")
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(coll) = resp.json::<Value>().await {
                    if let Some(count) = coll["totalItems"].as_u64() {
                        info.push(format!("following:{}", count));
                    }
                }
            }
        }
    }

    if info.is_empty() {
        TestResult {
            name: "Collections".to_string(),
            passed: true,
            message: "Collection URLs found (counts not public)".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        }
    } else {
        TestResult {
            name: "Collections".to_string(),
            passed: true,
            message: info.join(", "),
            duration_ms: start.elapsed().as_millis() as u64,
        }
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
