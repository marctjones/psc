//! Wikipedia article discovery and fetching

use crate::{Article, Interests};
use anyhow::{Context, Result};
use chrono::Utc;
use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Wikipedia API client
pub struct WikipediaClient {
    client: reqwest::Client,
    base_url: String,
}

impl WikipediaClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("sovereign-reader/0.1 (personal news aggregator)")
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url: "https://en.wikipedia.org/w/api.php".to_string(),
        }
    }

    /// Get the featured article of the day
    pub async fn get_featured_article(&self) -> Result<Option<Article>> {
        let today = Utc::now().format("%Y/%m/%d").to_string();
        let url = format!(
            "https://en.wikipedia.org/api/rest_v1/feed/featured/{}",
            today
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch featured content")?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let data: serde_json::Value = response.json().await?;

        if let Some(tfa) = data.get("tfa") {
            let title = tfa["title"].as_str().unwrap_or("Featured Article");
            let extract = tfa["extract"].as_str().unwrap_or("");
            let page_url = format!(
                "https://en.wikipedia.org/wiki/{}",
                title.replace(' ', "_")
            );

            let mut hasher = DefaultHasher::new();
            page_url.hash(&mut hasher);
            let id = format!("{:x}", hasher.finish());

            return Ok(Some(Article {
                id,
                source_id: "wikipedia".to_string(),
                title: format!("Featured: {}", title),
                url: page_url,
                summary: Some(extract.to_string()),
                content: None,
                authors: vec![],
                published: Some(Utc::now()),
                fetched_at: Utc::now(),
                categories: vec!["Featured".to_string()],
                read: false,
                interest_score: Some(90),
            }));
        }

        Ok(None)
    }

    /// Search for articles based on user interests
    pub async fn search_by_interests(
        &self,
        interests: &Interests,
        limit: usize,
        read_articles: &[String],
    ) -> Result<Vec<Article>> {
        let mut articles = Vec::new();

        // Search for each topic
        for topic in &interests.topics {
            let results = self.search(topic, 5).await?;
            for article in results {
                // Skip if already read
                if read_articles.contains(&article.id) {
                    continue;
                }
                // Skip if contains blocked keywords
                if interests.blocked_keywords.iter().any(|k| {
                    article.title.to_lowercase().contains(&k.to_lowercase())
                }) {
                    continue;
                }
                articles.push(article);
            }
        }

        // Search for keywords too
        for keyword in &interests.keywords {
            let results = self.search(keyword, 3).await?;
            for article in results {
                if read_articles.contains(&article.id) {
                    continue;
                }
                if !articles.iter().any(|a| a.id == article.id) {
                    articles.push(article);
                }
            }
        }

        // Sort by interest score and take top N
        articles.sort_by(|a, b| {
            b.interest_score
                .unwrap_or(0)
                .cmp(&a.interest_score.unwrap_or(0))
        });
        articles.truncate(limit);

        Ok(articles)
    }

    /// Get random articles (for discovery)
    pub async fn get_random_articles(&self, count: usize) -> Result<Vec<Article>> {
        let params = [
            ("action", "query"),
            ("format", "json"),
            ("list", "random"),
            ("rnnamespace", "0"),
            ("rnlimit", &count.to_string()),
        ];

        let response = self
            .client
            .get(&self.base_url)
            .query(&params)
            .send()
            .await
            .context("Failed to fetch random articles")?;

        let data: RandomResponse = response.json().await?;
        let mut articles = Vec::new();

        for item in data.query.random {
            // Fetch summary for each
            if let Ok(Some(article)) = self.get_article_summary(&item.title).await {
                articles.push(article);
            }
        }

        Ok(articles)
    }

    /// Search Wikipedia
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<Article>> {
        let params = [
            ("action", "query"),
            ("format", "json"),
            ("list", "search"),
            ("srsearch", query),
            ("srlimit", &limit.to_string()),
            ("srprop", "snippet|titlesnippet"),
        ];

        let response = self
            .client
            .get(&self.base_url)
            .query(&params)
            .send()
            .await
            .context("Failed to search Wikipedia")?;

        let data: SearchResponse = response.json().await?;
        let mut articles = Vec::new();

        for result in data.query.search {
            let url = format!(
                "https://en.wikipedia.org/wiki/{}",
                result.title.replace(' ', "_")
            );

            let mut hasher = DefaultHasher::new();
            url.hash(&mut hasher);
            let id = format!("{:x}", hasher.finish());

            // Clean up snippet (remove HTML tags)
            let summary = result
                .snippet
                .replace("<span class=\"searchmatch\">", "")
                .replace("</span>", "");

            articles.push(Article {
                id,
                source_id: "wikipedia".to_string(),
                title: result.title,
                url,
                summary: Some(summary),
                content: None,
                authors: vec![],
                published: None,
                fetched_at: Utc::now(),
                categories: vec![],
                read: false,
                interest_score: Some(50),
            });
        }

        Ok(articles)
    }

    /// Get summary for a specific article
    pub async fn get_article_summary(&self, title: &str) -> Result<Option<Article>> {
        let url = format!(
            "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
            urlencoding::encode(title)
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let data: SummaryResponse = response.json().await?;

        let page_url = format!(
            "https://en.wikipedia.org/wiki/{}",
            data.title.replace(' ', "_")
        );

        let mut hasher = DefaultHasher::new();
        page_url.hash(&mut hasher);
        let id = format!("{:x}", hasher.finish());

        Ok(Some(Article {
            id,
            source_id: "wikipedia".to_string(),
            title: data.title,
            url: page_url,
            summary: Some(data.extract),
            content: None,
            authors: vec![],
            published: None,
            fetched_at: Utc::now(),
            categories: vec![],
            read: false,
            interest_score: Some(50),
        }))
    }

    /// Get "On This Day" articles
    pub async fn get_on_this_day(&self) -> Result<Vec<Article>> {
        let now = Utc::now();
        let url = format!(
            "https://en.wikipedia.org/api/rest_v1/feed/onthisday/selected/{}/{}",
            now.format("%m"),
            now.format("%d")
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let data: serde_json::Value = response.json().await?;
        let mut articles = Vec::new();

        if let Some(selected) = data.get("selected").and_then(|s| s.as_array()) {
            for event in selected.iter().take(3) {
                if let Some(pages) = event.get("pages").and_then(|p| p.as_array()) {
                    if let Some(page) = pages.first() {
                        let title = page["title"].as_str().unwrap_or("Unknown");
                        let extract = page["extract"].as_str().unwrap_or("");
                        let year = event["year"].as_i64().unwrap_or(0);

                        let page_url = format!(
                            "https://en.wikipedia.org/wiki/{}",
                            title.replace(' ', "_")
                        );

                        let mut hasher = DefaultHasher::new();
                        page_url.hash(&mut hasher);
                        let id = format!("{:x}", hasher.finish());

                        articles.push(Article {
                            id,
                            source_id: "wikipedia".to_string(),
                            title: format!("On This Day ({}): {}", year, title),
                            url: page_url,
                            summary: Some(extract.to_string()),
                            content: None,
                            authors: vec![],
                            published: Some(Utc::now()),
                            fetched_at: Utc::now(),
                            categories: vec!["On This Day".to_string()],
                            read: false,
                            interest_score: Some(70),
                        });
                    }
                }
            }
        }

        Ok(articles)
    }
}

impl Default for WikipediaClient {
    fn default() -> Self {
        Self::new()
    }
}

// Response types for Wikipedia API

#[derive(Deserialize)]
struct RandomResponse {
    query: RandomQuery,
}

#[derive(Deserialize)]
struct RandomQuery {
    random: Vec<RandomItem>,
}

#[derive(Deserialize)]
struct RandomItem {
    title: String,
}

#[derive(Deserialize)]
struct SearchResponse {
    query: SearchQuery,
}

#[derive(Deserialize)]
struct SearchQuery {
    search: Vec<SearchResult>,
}

#[derive(Deserialize)]
struct SearchResult {
    title: String,
    snippet: String,
}

#[derive(Deserialize)]
struct SummaryResponse {
    title: String,
    extract: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wikipedia_client_creation() {
        let client = WikipediaClient::new();
        assert_eq!(client.base_url, "https://en.wikipedia.org/w/api.php");
    }
}
