//! SQLite storage for articles, sources, and reading history

use crate::{Article, Interests, ReaderConfig, Source, SourceType};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;

/// Database storage manager
pub struct Storage {
    conn: Connection,
}

impl Storage {
    /// Open or create the database
    pub fn open(path: Option<PathBuf>) -> Result<Self> {
        let path = path.unwrap_or_else(|| {
            let mut dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            dir.push("sovereign-reader");
            std::fs::create_dir_all(&dir).ok();
            dir.push("reader.db");
            dir
        });

        let conn = Connection::open(&path)
            .with_context(|| format!("Failed to open database at {:?}", path))?;

        let storage = Self { conn };
        storage.init_schema()?;

        Ok(storage)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sources (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                source_type TEXT NOT NULL,
                url TEXT NOT NULL,
                sections TEXT,
                enabled INTEGER NOT NULL DEFAULT 0,
                last_fetched TEXT
            );

            CREATE TABLE IF NOT EXISTS articles (
                id TEXT PRIMARY KEY,
                source_id TEXT NOT NULL,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                summary TEXT,
                content TEXT,
                authors TEXT,
                published TEXT,
                fetched_at TEXT NOT NULL,
                categories TEXT,
                read INTEGER NOT NULL DEFAULT 0,
                interest_score INTEGER,
                FOREIGN KEY (source_id) REFERENCES sources(id)
            );

            CREATE TABLE IF NOT EXISTS interests (
                id INTEGER PRIMARY KEY,
                topics TEXT,
                keywords TEXT,
                blocked_keywords TEXT,
                preferred_length TEXT
            );

            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_articles_source ON articles(source_id);
            CREATE INDEX IF NOT EXISTS idx_articles_read ON articles(read);
            CREATE INDEX IF NOT EXISTS idx_articles_published ON articles(published);
            "#,
        )?;

        Ok(())
    }

    // Source management

    /// Add or update a source
    pub fn save_source(&self, source: &Source) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO sources (id, name, source_type, url, sections, enabled, last_fetched)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                source.id,
                source.name,
                format!("{:?}", source.source_type),
                source.url,
                serde_json::to_string(&source.sections)?,
                source.enabled as i32,
                source.last_fetched.map(|d| d.to_rfc3339()),
            ],
        )?;

        Ok(())
    }

    /// Get all sources
    pub fn get_sources(&self) -> Result<Vec<Source>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, source_type, url, sections, enabled, last_fetched FROM sources",
        )?;

        let sources = stmt
            .query_map([], |row| {
                let source_type_str: String = row.get(2)?;
                let source_type = match source_type_str.as_str() {
                    "Feed" => SourceType::Feed,
                    "Web" => SourceType::Web,
                    "Wikipedia" => SourceType::Wikipedia,
                    _ => SourceType::Feed,
                };

                let sections_json: String = row.get(4)?;
                let sections: Vec<String> =
                    serde_json::from_str(&sections_json).unwrap_or_default();

                let last_fetched: Option<String> = row.get(6)?;
                let last_fetched = last_fetched.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|d| d.with_timezone(&Utc))
                });

                Ok(Source {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    source_type,
                    url: row.get(3)?,
                    sections,
                    enabled: row.get::<_, i32>(5)? != 0,
                    last_fetched,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sources)
    }

    /// Get enabled sources
    pub fn get_enabled_sources(&self) -> Result<Vec<Source>> {
        let sources = self.get_sources()?;
        Ok(sources.into_iter().filter(|s| s.enabled).collect())
    }

    /// Delete a source
    pub fn delete_source(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM sources WHERE id = ?1", params![id])?;
        Ok(())
    }

    // Article management

    /// Save an article
    pub fn save_article(&self, article: &Article) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO articles
            (id, source_id, title, url, summary, content, authors, published, fetched_at, categories, read, interest_score)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                article.id,
                article.source_id,
                article.title,
                article.url,
                article.summary,
                article.content,
                serde_json::to_string(&article.authors)?,
                article.published.map(|d| d.to_rfc3339()),
                article.fetched_at.to_rfc3339(),
                serde_json::to_string(&article.categories)?,
                article.read as i32,
                article.interest_score,
            ],
        )?;

        Ok(())
    }

    /// Get unread articles
    pub fn get_unread_articles(&self, limit: usize) -> Result<Vec<Article>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, source_id, title, url, summary, content, authors, published,
                   fetched_at, categories, read, interest_score
            FROM articles
            WHERE read = 0
            ORDER BY interest_score DESC, published DESC
            LIMIT ?1
            "#,
        )?;

        self.query_articles(&mut stmt, params![limit as i64])
    }

    /// Get articles by source
    pub fn get_articles_by_source(&self, source_id: &str, limit: usize) -> Result<Vec<Article>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, source_id, title, url, summary, content, authors, published,
                   fetched_at, categories, read, interest_score
            FROM articles
            WHERE source_id = ?1
            ORDER BY published DESC
            LIMIT ?2
            "#,
        )?;

        self.query_articles(&mut stmt, params![source_id, limit as i64])
    }

    /// Get recent articles for digest
    pub fn get_recent_articles(&self, days: u32, limit: usize) -> Result<Vec<Article>> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);

        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, source_id, title, url, summary, content, authors, published,
                   fetched_at, categories, read, interest_score
            FROM articles
            WHERE fetched_at >= ?1 AND read = 0
            ORDER BY interest_score DESC, published DESC
            LIMIT ?2
            "#,
        )?;

        self.query_articles(&mut stmt, params![cutoff.to_rfc3339(), limit as i64])
    }

    /// Mark article as read
    pub fn mark_read(&self, article_id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE articles SET read = 1 WHERE id = ?1",
            params![article_id],
        )?;
        Ok(())
    }

    /// Get all read article IDs (for filtering)
    pub fn get_read_article_ids(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT id FROM articles WHERE read = 1")?;
        let ids = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ids)
    }

    fn query_articles(
        &self,
        stmt: &mut rusqlite::Statement,
        params: impl rusqlite::Params,
    ) -> Result<Vec<Article>> {
        let articles = stmt
            .query_map(params, |row| {
                let authors_json: String = row.get(6)?;
                let authors: Vec<String> =
                    serde_json::from_str(&authors_json).unwrap_or_default();

                let categories_json: String = row.get(9)?;
                let categories: Vec<String> =
                    serde_json::from_str(&categories_json).unwrap_or_default();

                let published: Option<String> = row.get(7)?;
                let published = published.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|d| d.with_timezone(&Utc))
                });

                let fetched_at: String = row.get(8)?;
                let fetched_at = DateTime::parse_from_rfc3339(&fetched_at)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(Article {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    title: row.get(2)?,
                    url: row.get(3)?,
                    summary: row.get(4)?,
                    content: row.get(5)?,
                    authors,
                    published,
                    fetched_at,
                    categories,
                    read: row.get::<_, i32>(10)? != 0,
                    interest_score: row.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(articles)
    }

    // Interests management

    /// Save user interests
    pub fn save_interests(&self, interests: &Interests) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO interests (id, topics, keywords, blocked_keywords, preferred_length)
            VALUES (1, ?1, ?2, ?3, ?4)
            "#,
            params![
                serde_json::to_string(&interests.topics)?,
                serde_json::to_string(&interests.keywords)?,
                serde_json::to_string(&interests.blocked_keywords)?,
                format!("{:?}", interests.preferred_length),
            ],
        )?;

        Ok(())
    }

    /// Get user interests
    pub fn get_interests(&self) -> Result<Interests> {
        let mut stmt = self.conn.prepare(
            "SELECT topics, keywords, blocked_keywords, preferred_length FROM interests WHERE id = 1",
        )?;

        let result = stmt.query_row([], |row| {
            let topics_json: String = row.get(0)?;
            let keywords_json: String = row.get(1)?;
            let blocked_json: String = row.get(2)?;

            Ok(Interests {
                topics: serde_json::from_str(&topics_json).unwrap_or_default(),
                keywords: serde_json::from_str(&keywords_json).unwrap_or_default(),
                blocked_keywords: serde_json::from_str(&blocked_json).unwrap_or_default(),
                preferred_length: crate::ContentLength::default(),
            })
        });

        match result {
            Ok(interests) => Ok(interests),
            Err(_) => Ok(Interests::default()),
        }
    }

    // Config management

    /// Save config value
    pub fn set_config(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    /// Get config value
    pub fn get_config(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM config WHERE key = ?1")?;
        let result = stmt.query_row(params![key], |row| row.get(0));
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get reader config
    pub fn get_reader_config(&self) -> Result<ReaderConfig> {
        if let Some(json) = self.get_config("reader_config")? {
            Ok(serde_json::from_str(&json)?)
        } else {
            Ok(ReaderConfig::default())
        }
    }

    /// Save reader config
    pub fn save_reader_config(&self, config: &ReaderConfig) -> Result<()> {
        let json = serde_json::to_string(config)?;
        self.set_config("reader_config", &json)
    }

    /// Clean up old articles
    pub fn cleanup_old_articles(&self, days: u32) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        let deleted = self.conn.execute(
            "DELETE FROM articles WHERE fetched_at < ?1 AND read = 1",
            params![cutoff.to_rfc3339()],
        )?;
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_in_memory() {
        let storage = Storage {
            conn: Connection::open_in_memory().unwrap(),
        };
        storage.init_schema().unwrap();

        // Test saving and retrieving a source
        let source = Source {
            id: "test".to_string(),
            name: "Test Source".to_string(),
            source_type: SourceType::Feed,
            url: "https://example.com/feed".to_string(),
            sections: vec!["tech".to_string()],
            enabled: true,
            last_fetched: None,
        };

        storage.save_source(&source).unwrap();
        let sources = storage.get_sources().unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].name, "Test Source");
    }
}
