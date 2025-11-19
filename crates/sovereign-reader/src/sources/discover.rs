//! RSS/Atom feed discovery and site analysis

use anyhow::{Context, Result};
use scraper::{Html, Selector};
use url::Url;

/// Discovered feed information
#[derive(Debug, Clone)]
pub struct DiscoveredFeed {
    pub url: String,
    pub title: Option<String>,
    pub feed_type: FeedType,
}

/// Type of feed discovered
#[derive(Debug, Clone, PartialEq)]
pub enum FeedType {
    Rss,
    Atom,
    Json,
    Unknown,
}

/// Discover RSS/Atom feeds on a website
pub async fn discover_feeds(url: &str) -> Result<Vec<DiscoveredFeed>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (compatible; sovereign-reader/0.1)")
        .build()?;

    let base_url = Url::parse(url).context("Invalid URL")?;
    let mut feeds = Vec::new();

    // Fetch the page
    let response = client.get(url).send().await?;
    let html = response.text().await?;
    let document = Html::parse_document(&html);

    // Method 1: Look for <link> tags in <head>
    let link_selector = Selector::parse("link[rel='alternate']").unwrap();
    for element in document.select(&link_selector) {
        let href = element.value().attr("href");
        let link_type = element.value().attr("type").unwrap_or("");
        let title = element.value().attr("title");

        if let Some(href) = href {
            let feed_type = match link_type {
                t if t.contains("rss") => FeedType::Rss,
                t if t.contains("atom") => FeedType::Atom,
                t if t.contains("json") => FeedType::Json,
                _ => continue, // Skip non-feed links
            };

            let feed_url = resolve_url(&base_url, href);
            feeds.push(DiscoveredFeed {
                url: feed_url,
                title: title.map(|s| s.to_string()),
                feed_type,
            });
        }
    }

    // Method 2: Look for common feed link patterns in <a> tags
    let a_selector = Selector::parse("a[href]").unwrap();
    for element in document.select(&a_selector) {
        if let Some(href) = element.value().attr("href") {
            let href_lower = href.to_lowercase();
            if href_lower.contains("rss")
                || href_lower.contains("feed")
                || href_lower.contains("atom")
                || href_lower.ends_with(".xml")
            {
                let feed_url = resolve_url(&base_url, href);

                // Avoid duplicates
                if !feeds.iter().any(|f| f.url == feed_url) {
                    // Try to determine type from URL
                    let feed_type = if href_lower.contains("atom") {
                        FeedType::Atom
                    } else if href_lower.contains("json") {
                        FeedType::Json
                    } else {
                        FeedType::Rss
                    };

                    let title = element.text().collect::<String>();
                    feeds.push(DiscoveredFeed {
                        url: feed_url,
                        title: if title.trim().is_empty() {
                            None
                        } else {
                            Some(title.trim().to_string())
                        },
                        feed_type,
                    });
                }
            }
        }
    }

    // Method 3: Try common feed paths
    let common_paths = [
        "/feed",
        "/feed/",
        "/rss",
        "/rss/",
        "/rss.xml",
        "/atom.xml",
        "/feed.xml",
        "/index.xml",
        "/feeds/posts/default",
        "/?feed=rss2",
        "/blog/feed",
        "/news/feed",
    ];

    for path in common_paths {
        let feed_url = format!("{}{}", base_url.origin().ascii_serialization(), path);

        // Skip if we already found this URL
        if feeds.iter().any(|f| f.url == feed_url) {
            continue;
        }

        // Check if the URL exists and is a valid feed
        if let Ok(resp) = client.head(&feed_url).send().await {
            if resp.status().is_success() {
                // Verify it's actually XML/feed content
                if let Some(content_type) = resp.headers().get("content-type") {
                    let ct = content_type.to_str().unwrap_or("");
                    if ct.contains("xml") || ct.contains("rss") || ct.contains("atom") {
                        feeds.push(DiscoveredFeed {
                            url: feed_url,
                            title: None,
                            feed_type: FeedType::Unknown,
                        });
                    }
                }
            }
        }
    }

    // Validate discovered feeds by trying to fetch them
    let mut valid_feeds = Vec::new();
    for feed in feeds {
        if let Ok(resp) = client.get(&feed.url).send().await {
            if resp.status().is_success() {
                valid_feeds.push(feed);
            }
        }
    }

    Ok(valid_feeds)
}

/// Analyze a website to determine the best way to track it
pub async fn analyze_site(url: &str) -> Result<SiteAnalysis> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (compatible; sovereign-reader/0.1)")
        .build()?;

    let base_url = Url::parse(url).context("Invalid URL")?;

    // Try to discover feeds
    let feeds = discover_feeds(url).await.unwrap_or_default();

    // Fetch the page for analysis
    let response = client.get(url).send().await?;
    let html = response.text().await?;
    let document = Html::parse_document(&html);

    // Count article-like elements
    let article_selectors = ["article", ".post", ".entry", ".story", ".news-item", ".blog-post"];
    let mut article_count = 0;

    for selector_str in article_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            article_count += document.select(&selector).count();
        }
    }

    // Detect site type
    let site_type = if !feeds.is_empty() {
        SiteType::HasFeed
    } else if article_count > 0 {
        SiteType::ScrapableArticles
    } else {
        // Check for common content patterns
        let has_links = document
            .select(&Selector::parse("a[href]").unwrap())
            .count()
            > 10;

        if has_links {
            SiteType::LinkList
        } else {
            SiteType::StaticPage
        }
    };

    // Find best selectors for scraping
    let recommended_selector = find_best_selector(&document);

    let supports_scraping = article_count > 0 || site_type == SiteType::LinkList;

    Ok(SiteAnalysis {
        url: url.to_string(),
        site_type,
        feeds,
        article_count,
        recommended_selector,
        supports_scraping,
    })
}

/// Site analysis result
#[derive(Debug)]
pub struct SiteAnalysis {
    pub url: String,
    pub site_type: SiteType,
    pub feeds: Vec<DiscoveredFeed>,
    pub article_count: usize,
    pub recommended_selector: Option<String>,
    pub supports_scraping: bool,
}

/// Type of website
#[derive(Debug, Clone, PartialEq)]
pub enum SiteType {
    /// Has RSS/Atom feed(s)
    HasFeed,
    /// Can scrape article elements
    ScrapableArticles,
    /// Page with list of links (like HN, Reddit)
    LinkList,
    /// Static page without regular updates
    StaticPage,
}

/// Find the best CSS selector for scraping articles
fn find_best_selector(document: &Html) -> Option<String> {
    let selectors_to_try = [
        ("article", "article"),
        (".post", ".post"),
        (".entry", ".entry"),
        (".story", ".story"),
        (".news-item", ".news-item"),
        (".blog-post", ".blog-post"),
        ("[data-testid='post']", "[data-testid='post']"),
        (".item", ".item"),
        ("li.news", "li.news"),
    ];

    for (selector_str, name) in selectors_to_try {
        if let Ok(selector) = Selector::parse(selector_str) {
            let count = document.select(&selector).count();
            if count >= 3 {
                return Some(name.to_string());
            }
        }
    }

    None
}

/// Resolve a potentially relative URL
fn resolve_url(base: &Url, href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else if let Ok(resolved) = base.join(href) {
        resolved.to_string()
    } else {
        href.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_url_absolute() {
        let base = Url::parse("https://example.com/page").unwrap();
        assert_eq!(
            resolve_url(&base, "https://other.com/feed"),
            "https://other.com/feed"
        );
    }

    #[test]
    fn test_resolve_url_relative() {
        let base = Url::parse("https://example.com/page").unwrap();
        assert_eq!(
            resolve_url(&base, "/feed.xml"),
            "https://example.com/feed.xml"
        );
    }
}
