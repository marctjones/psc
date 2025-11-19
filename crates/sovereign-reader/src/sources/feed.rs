//! RSS/Atom feed fetcher

use crate::{Article, FetchResult, Source};
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use feed_rs::parser;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::Fetcher;

/// Fetcher for RSS/Atom feeds
pub struct FeedFetcher {
    client: reqwest::Client,
}

impl FeedFetcher {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("sovereign-reader/0.1")
            .build()
            .expect("Failed to build HTTP client");

        Self { client }
    }
}

impl Default for FeedFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Fetcher for FeedFetcher {
    async fn fetch(&self, source: &Source) -> Result<FetchResult> {
        let mut articles = Vec::new();
        let mut errors = Vec::new();

        // Fetch the feed
        let response = self
            .client
            .get(&source.url)
            .send()
            .await
            .context("Failed to fetch feed")?;

        if !response.status().is_success() {
            return Ok(FetchResult {
                source_id: source.id.clone(),
                articles: vec![],
                errors: vec![format!("HTTP {}", response.status())],
            });
        }

        let bytes = response.bytes().await.context("Failed to read response")?;

        // Parse the feed
        let feed = match parser::parse(&bytes[..]) {
            Ok(f) => f,
            Err(e) => {
                return Ok(FetchResult {
                    source_id: source.id.clone(),
                    articles: vec![],
                    errors: vec![format!("Failed to parse feed: {}", e)],
                });
            }
        };

        // Convert entries to articles
        for entry in feed.entries {
            // Skip if we're filtering by sections and this doesn't match
            if !source.sections.is_empty() {
                let entry_categories: Vec<String> = entry
                    .categories
                    .iter()
                    .map(|c| c.term.to_lowercase())
                    .collect();

                let matches_section = source.sections.iter().any(|s| {
                    entry_categories.iter().any(|c| c.contains(&s.to_lowercase()))
                });

                if !matches_section {
                    continue;
                }
            }

            // Generate a unique ID from the URL
            let url = entry
                .links
                .first()
                .map(|l| l.href.clone())
                .unwrap_or_else(|| entry.id.clone());

            let mut hasher = DefaultHasher::new();
            url.hash(&mut hasher);
            let id = format!("{:x}", hasher.finish());

            // Extract summary
            let summary = entry
                .summary
                .map(|s| s.content)
                .or_else(|| entry.content.as_ref().and_then(|c| c.body.clone()));

            // Extract content
            let content = entry.content.and_then(|c| c.body);

            // Extract authors
            let authors: Vec<String> = entry
                .authors
                .iter()
                .map(|a| a.name.clone())
                .collect();

            // Extract categories
            let categories: Vec<String> = entry
                .categories
                .iter()
                .map(|c| c.term.clone())
                .collect();

            let article = Article {
                id,
                source_id: source.id.clone(),
                title: entry.title.map(|t| t.content).unwrap_or_default(),
                url,
                summary,
                content,
                authors,
                published: entry.published.or(entry.updated),
                fetched_at: Utc::now(),
                categories,
                read: false,
                interest_score: None,
            };

            articles.push(article);
        }

        Ok(FetchResult {
            source_id: source.id.clone(),
            articles,
            errors,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceType;

    #[test]
    fn test_feed_fetcher_creation() {
        let fetcher = FeedFetcher::new();
        // Just verify it can be created
        assert!(true);
    }
}
