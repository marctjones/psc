//! Web scraping fetcher for sites without RSS feeds

use crate::{Article, FetchResult, Source};
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use scraper::{Html, Selector};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::Fetcher;

/// Fetcher for web scraping
pub struct WebFetcher {
    client: reqwest::Client,
}

impl WebFetcher {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; sovereign-reader/0.1)")
            .build()
            .expect("Failed to build HTTP client");

        Self { client }
    }
}

impl Default for WebFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Fetcher for WebFetcher {
    async fn fetch(&self, source: &Source) -> Result<FetchResult> {
        let mut articles = Vec::new();
        let mut errors = Vec::new();

        // Fetch the page
        let response = self
            .client
            .get(&source.url)
            .send()
            .await
            .context("Failed to fetch page")?;

        if !response.status().is_success() {
            return Ok(FetchResult {
                source_id: source.id.clone(),
                articles: vec![],
                errors: vec![format!("HTTP {}", response.status())],
            });
        }

        let html = response.text().await.context("Failed to read response")?;
        let document = Html::parse_document(&html);

        // Try common article selectors
        let selectors = [
            "article",
            ".article",
            ".post",
            ".story",
            ".entry",
            "[data-testid='card']",
        ];

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    // Try to extract title
                    let title = extract_text(&element, &["h1", "h2", "h3", ".title", ".headline"])
                        .unwrap_or_default();

                    if title.is_empty() {
                        continue;
                    }

                    // Try to extract link
                    let url = element
                        .select(&Selector::parse("a").unwrap())
                        .next()
                        .and_then(|a| a.value().attr("href"))
                        .map(|href| resolve_url(&source.url, href))
                        .unwrap_or_default();

                    if url.is_empty() {
                        continue;
                    }

                    // Generate ID
                    let mut hasher = DefaultHasher::new();
                    url.hash(&mut hasher);
                    let id = format!("{:x}", hasher.finish());

                    // Try to extract summary
                    let summary = extract_text(&element, &["p", ".summary", ".excerpt", ".description"]);

                    let article = Article {
                        id,
                        source_id: source.id.clone(),
                        title,
                        url,
                        summary,
                        content: None,
                        authors: vec![],
                        published: None,
                        fetched_at: Utc::now(),
                        categories: vec![],
                        read: false,
                        interest_score: None,
                    };

                    articles.push(article);
                }

                // If we found articles with this selector, stop trying others
                if !articles.is_empty() {
                    break;
                }
            }
        }

        if articles.is_empty() {
            errors.push("No articles found with common selectors".to_string());
        }

        Ok(FetchResult {
            source_id: source.id.clone(),
            articles,
            errors,
        })
    }
}

/// Extract text from an element using multiple possible selectors
fn extract_text(element: &scraper::ElementRef, selectors: &[&str]) -> Option<String> {
    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(found) = element.select(&selector).next() {
                let text: String = found.text().collect::<Vec<_>>().join(" ");
                let text = text.trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
    }
    None
}

/// Resolve a potentially relative URL to absolute
fn resolve_url(base: &str, href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        return href.to_string();
    }

    if let Ok(base_url) = url::Url::parse(base) {
        if let Ok(resolved) = base_url.join(href) {
            return resolved.to_string();
        }
    }

    href.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_url_absolute() {
        let result = resolve_url("https://example.com", "https://other.com/page");
        assert_eq!(result, "https://other.com/page");
    }

    #[test]
    fn test_resolve_url_relative() {
        let result = resolve_url("https://example.com/news/", "/article/123");
        assert_eq!(result, "https://example.com/article/123");
    }
}
