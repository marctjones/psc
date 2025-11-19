//! Sovereign Reader - Personal news aggregator and daily digest generator
//!
//! A Unix-philosophy tool that does one thing well: aggregates content from
//! your favorite websites and Wikipedia to create a personalized daily newspaper.

pub mod sources;
pub mod wikipedia;
pub mod digest;
pub mod storage;
pub mod llm;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A news/content source (website, RSS feed, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Source type
    pub source_type: SourceType,
    /// URL to fetch from
    pub url: String,
    /// Sections/categories to include (empty = all)
    pub sections: Vec<String>,
    /// Whether this source is enabled
    pub enabled: bool,
    /// Last fetch time
    pub last_fetched: Option<DateTime<Utc>>,
}

/// Type of content source
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceType {
    /// RSS/Atom feed
    Feed,
    /// Web scraping (one-time fetch)
    Web,
    /// Track a page for changes (stores hashes)
    Track,
    /// Wikipedia
    Wikipedia,
}

/// A fetched article/content item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    /// Unique identifier (usually URL hash)
    pub id: String,
    /// Source this came from
    pub source_id: String,
    /// Article title
    pub title: String,
    /// Article URL
    pub url: String,
    /// Brief summary/description
    pub summary: Option<String>,
    /// Full content (if available)
    pub content: Option<String>,
    /// Author(s)
    pub authors: Vec<String>,
    /// Publication date
    pub published: Option<DateTime<Utc>>,
    /// When we fetched it
    pub fetched_at: DateTime<Utc>,
    /// Categories/tags
    pub categories: Vec<String>,
    /// Has the user read this?
    pub read: bool,
    /// User's interest score (0-100)
    pub interest_score: Option<u8>,
}

/// User interests for content recommendation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Interests {
    /// Topics the user is interested in
    pub topics: Vec<String>,
    /// Keywords to boost
    pub keywords: Vec<String>,
    /// Keywords to suppress
    pub blocked_keywords: Vec<String>,
    /// Preferred content length (short/medium/long)
    pub preferred_length: ContentLength,
}

/// Content length preference
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum ContentLength {
    Short,
    #[default]
    Medium,
    Long,
    Any,
}

/// Configuration for the reader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaderConfig {
    /// How many articles per source in the digest
    pub articles_per_source: usize,
    /// How many Wikipedia articles to include
    pub wikipedia_articles: usize,
    /// Hours between fetches
    pub fetch_interval_hours: u32,
    /// Maximum article age in days
    pub max_article_age_days: u32,
    /// Output format for digest
    pub digest_format: DigestFormat,
}

impl Default for ReaderConfig {
    fn default() -> Self {
        Self {
            articles_per_source: 5,
            wikipedia_articles: 3,
            fetch_interval_hours: 6,
            max_article_age_days: 7,
            digest_format: DigestFormat::Markdown,
        }
    }
}

/// Output format for the daily digest
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum DigestFormat {
    #[default]
    Markdown,
    Html,
    Plain,
    Json,
}

/// Result of a fetch operation
#[derive(Debug)]
pub struct FetchResult {
    pub source_id: String,
    pub articles: Vec<Article>,
    pub errors: Vec<String>,
}

/// The daily digest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyDigest {
    /// When this digest was generated
    pub generated_at: DateTime<Utc>,
    /// Date this digest is for
    pub date: chrono::NaiveDate,
    /// Sections of content
    pub sections: Vec<DigestSection>,
    /// Total article count
    pub total_articles: usize,
}

/// A section in the daily digest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestSection {
    /// Section name (e.g., "Technology", "World News", "Wikipedia Discoveries")
    pub name: String,
    /// Articles in this section
    pub articles: Vec<Article>,
}
