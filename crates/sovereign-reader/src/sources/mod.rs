//! Content source fetching and parsing

mod feed;
mod web;

pub use feed::FeedFetcher;
pub use web::WebFetcher;

use crate::{Article, FetchResult, Source, SourceType};
use anyhow::Result;
use async_trait::async_trait;

/// Trait for content fetchers
#[async_trait]
pub trait Fetcher: Send + Sync {
    /// Fetch articles from a source
    async fn fetch(&self, source: &Source) -> Result<FetchResult>;
}

/// Fetch articles from any source type
pub async fn fetch_source(source: &Source) -> Result<FetchResult> {
    match source.source_type {
        SourceType::Feed => {
            let fetcher = FeedFetcher::new();
            fetcher.fetch(source).await
        }
        SourceType::Web => {
            let fetcher = WebFetcher::new();
            fetcher.fetch(source).await
        }
        SourceType::Wikipedia => {
            // Wikipedia is handled separately
            Ok(FetchResult {
                source_id: source.id.clone(),
                articles: vec![],
                errors: vec!["Use wikipedia module for Wikipedia sources".to_string()],
            })
        }
    }
}

/// Well-known news sources with their configurations
pub fn get_known_sources() -> Vec<Source> {
    vec![
        Source {
            id: "lwn".to_string(),
            name: "LWN.net".to_string(),
            source_type: SourceType::Feed,
            url: "https://lwn.net/headlines/rss".to_string(),
            sections: vec![],
            enabled: false,
            last_fetched: None,
        },
        Source {
            id: "ars".to_string(),
            name: "Ars Technica".to_string(),
            source_type: SourceType::Feed,
            url: "https://feeds.arstechnica.com/arstechnica/index".to_string(),
            sections: vec![],
            enabled: false,
            last_fetched: None,
        },
        Source {
            id: "hackernews".to_string(),
            name: "Hacker News".to_string(),
            source_type: SourceType::Feed,
            url: "https://hnrss.org/frontpage".to_string(),
            sections: vec![],
            enabled: false,
            last_fetched: None,
        },
        Source {
            id: "nytimes".to_string(),
            name: "New York Times".to_string(),
            source_type: SourceType::Feed,
            url: "https://rss.nytimes.com/services/xml/rss/nyt/HomePage.xml".to_string(),
            sections: vec![],
            enabled: false,
            last_fetched: None,
        },
        Source {
            id: "wapo".to_string(),
            name: "Washington Post".to_string(),
            source_type: SourceType::Feed,
            url: "https://feeds.washingtonpost.com/rss/national".to_string(),
            sections: vec![],
            enabled: false,
            last_fetched: None,
        },
        Source {
            id: "bbc".to_string(),
            name: "BBC News".to_string(),
            source_type: SourceType::Feed,
            url: "https://feeds.bbci.co.uk/news/rss.xml".to_string(),
            sections: vec![],
            enabled: false,
            last_fetched: None,
        },
        Source {
            id: "guardian".to_string(),
            name: "The Guardian".to_string(),
            source_type: SourceType::Feed,
            url: "https://www.theguardian.com/world/rss".to_string(),
            sections: vec![],
            enabled: false,
            last_fetched: None,
        },
        Source {
            id: "reuters".to_string(),
            name: "Reuters".to_string(),
            source_type: SourceType::Feed,
            url: "https://www.reutersagency.com/feed/".to_string(),
            sections: vec![],
            enabled: false,
            last_fetched: None,
        },
    ]
}
