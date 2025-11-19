//! LLM integration for intelligent article scoring and summarization
//!
//! When running in an LLM assistant environment (like Claude Code), this module
//! uses file-based IPC to communicate with the assistant for intelligent features.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Check if we're running in an LLM assistant environment
pub fn is_llm_available() -> bool {
    // Check for common indicators
    std::env::var("CLAUDE_CODE").is_ok()
        || std::env::var("LLM_ASSISTANT").is_ok()
        || std::env::var("ANTHROPIC_CONTEXT").is_ok()
        // Also check if our IPC directory exists (assistant may have created it)
        || get_ipc_dir().exists()
}

/// Get the IPC directory for LLM communication
pub fn get_ipc_dir() -> PathBuf {
    let mut dir = dirs::runtime_dir()
        .or_else(|| dirs::cache_dir())
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    dir.push("sovereign-reader-llm");
    dir
}

/// Request types that can be sent to the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LlmRequest {
    /// Score articles based on user interests
    ScoreArticles {
        articles: Vec<ArticleForScoring>,
        interests: UserInterests,
    },
    /// Summarize an article
    Summarize {
        title: String,
        content: String,
        url: String,
    },
    /// Analyze reading patterns and suggest interests
    AnalyzePatterns {
        read_articles: Vec<ArticleForScoring>,
        current_interests: UserInterests,
    },
    /// Generate a personalized digest introduction
    DigestIntro {
        article_count: usize,
        sources: Vec<String>,
        top_topics: Vec<String>,
    },
    /// Suggest new sources based on interests
    SuggestSources {
        interests: UserInterests,
        current_sources: Vec<String>,
    },
}

/// Response types from the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LlmResponse {
    /// Scored articles with interest scores
    ScoredArticles {
        scores: Vec<ArticleScore>,
    },
    /// Article summary
    Summary {
        summary: String,
        key_points: Vec<String>,
    },
    /// Suggested interests based on reading patterns
    SuggestedInterests {
        topics: Vec<String>,
        keywords: Vec<String>,
        reasoning: String,
    },
    /// Personalized digest introduction
    DigestIntroText {
        intro: String,
    },
    /// Suggested sources
    SuggestedSources {
        sources: Vec<SourceSuggestion>,
    },
    /// Error response
    Error {
        message: String,
    },
}

/// Article data for scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleForScoring {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub source: String,
    pub categories: Vec<String>,
}

/// User interests for scoring context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInterests {
    pub topics: Vec<String>,
    pub keywords: Vec<String>,
    pub blocked_keywords: Vec<String>,
}

/// Score for an article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleScore {
    pub id: String,
    pub score: u8, // 0-100
    pub reason: String,
}

/// Suggested source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSuggestion {
    pub name: String,
    pub url: String,
    pub reason: String,
}

/// LLM client for file-based IPC communication
pub struct LlmClient {
    ipc_dir: PathBuf,
    timeout: Duration,
}

impl LlmClient {
    /// Create a new LLM client
    pub fn new() -> Result<Self> {
        let ipc_dir = get_ipc_dir();

        // Create IPC directory if it doesn't exist
        std::fs::create_dir_all(&ipc_dir)
            .context("Failed to create LLM IPC directory")?;

        Ok(Self {
            ipc_dir,
            timeout: Duration::from_secs(60),
        })
    }

    /// Set timeout for LLM responses
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Send a request to the LLM and wait for response
    pub async fn request(&self, request: LlmRequest) -> Result<LlmResponse> {
        let request_id = generate_request_id();
        let request_file = self.ipc_dir.join(format!("request-{}.json", request_id));
        let response_file = self.ipc_dir.join(format!("response-{}.json", request_id));

        // Write request file
        let request_json = serde_json::to_string_pretty(&request)?;
        std::fs::write(&request_file, &request_json)
            .context("Failed to write LLM request")?;

        // Also write a marker file that the assistant can watch
        let marker_file = self.ipc_dir.join("pending");
        std::fs::write(&marker_file, &request_id)
            .context("Failed to write marker file")?;

        // Wait for response
        let start = Instant::now();
        loop {
            if response_file.exists() {
                let response_json = std::fs::read_to_string(&response_file)
                    .context("Failed to read LLM response")?;

                // Clean up files
                let _ = std::fs::remove_file(&request_file);
                let _ = std::fs::remove_file(&response_file);
                let _ = std::fs::remove_file(&marker_file);

                let response: LlmResponse = serde_json::from_str(&response_json)
                    .context("Failed to parse LLM response")?;

                return Ok(response);
            }

            if start.elapsed() > self.timeout {
                // Clean up request file
                let _ = std::fs::remove_file(&request_file);
                let _ = std::fs::remove_file(&marker_file);

                return Err(anyhow::anyhow!(
                    "Timeout waiting for LLM response. Is the LLM assistant monitoring {}?",
                    self.ipc_dir.display()
                ));
            }

            // Poll every 100ms
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Score articles based on user interests
    pub async fn score_articles(
        &self,
        articles: Vec<ArticleForScoring>,
        interests: UserInterests,
    ) -> Result<Vec<ArticleScore>> {
        let response = self
            .request(LlmRequest::ScoreArticles {
                articles,
                interests,
            })
            .await?;

        match response {
            LlmResponse::ScoredArticles { scores } => Ok(scores),
            LlmResponse::Error { message } => Err(anyhow::anyhow!("LLM error: {}", message)),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    /// Summarize an article
    pub async fn summarize(
        &self,
        title: &str,
        content: &str,
        url: &str,
    ) -> Result<(String, Vec<String>)> {
        let response = self
            .request(LlmRequest::Summarize {
                title: title.to_string(),
                content: content.to_string(),
                url: url.to_string(),
            })
            .await?;

        match response {
            LlmResponse::Summary { summary, key_points } => Ok((summary, key_points)),
            LlmResponse::Error { message } => Err(anyhow::anyhow!("LLM error: {}", message)),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    /// Analyze reading patterns and suggest interests
    pub async fn analyze_patterns(
        &self,
        read_articles: Vec<ArticleForScoring>,
        current_interests: UserInterests,
    ) -> Result<(Vec<String>, Vec<String>, String)> {
        let response = self
            .request(LlmRequest::AnalyzePatterns {
                read_articles,
                current_interests,
            })
            .await?;

        match response {
            LlmResponse::SuggestedInterests {
                topics,
                keywords,
                reasoning,
            } => Ok((topics, keywords, reasoning)),
            LlmResponse::Error { message } => Err(anyhow::anyhow!("LLM error: {}", message)),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    /// Get the IPC directory path (for display to user)
    pub fn ipc_dir(&self) -> &PathBuf {
        &self.ipc_dir
    }
}

impl Default for LlmClient {
    fn default() -> Self {
        Self::new().expect("Failed to create LLM client")
    }
}

/// Generate a unique request ID
fn generate_request_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", time)
}

/// Instructions for the LLM assistant on how to process requests
pub fn get_assistant_instructions() -> String {
    let ipc_dir = get_ipc_dir();
    format!(
        r#"
## Sovereign Reader LLM Integration

The sovereign-reader tool can communicate with you for intelligent features.

### How it works:
1. Watch for files in: {}
2. When you see `pending` file, read the request-*.json file
3. Process the request based on its type
4. Write response to response-*.json (same ID)

### Request types:

**ScoreArticles**: Score articles 0-100 based on user interests
- Consider topic relevance, keyword matches
- Provide brief reason for each score

**Summarize**: Create a concise summary with key points
- 2-3 sentence summary
- 3-5 bullet point key takeaways

**AnalyzePatterns**: Suggest interests based on reading history
- Identify common themes
- Suggest new topics/keywords
- Explain reasoning

**DigestIntro**: Write personalized intro for daily digest
- Reference article count, sources
- Mention top topics
- Keep it brief and engaging

**SuggestSources**: Recommend news sources
- Based on user interests
- Include RSS feed URLs when possible
- Explain why each is relevant

### Example response format:
```json
{{
  "type": "ScoredArticles",
  "scores": [
    {{"id": "abc123", "score": 85, "reason": "Directly relates to Rust programming interest"}}
  ]
}}
```
"#,
        ipc_dir.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = LlmRequest::ScoreArticles {
            articles: vec![ArticleForScoring {
                id: "test".to_string(),
                title: "Test Article".to_string(),
                summary: Some("Summary".to_string()),
                source: "test-source".to_string(),
                categories: vec!["tech".to_string()],
            }],
            interests: UserInterests {
                topics: vec!["Rust".to_string()],
                keywords: vec!["programming".to_string()],
                blocked_keywords: vec![],
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("ScoreArticles"));
    }

    #[test]
    fn test_response_deserialization() {
        let json = r#"{
            "type": "ScoredArticles",
            "scores": [
                {"id": "test", "score": 85, "reason": "Good match"}
            ]
        }"#;

        let response: LlmResponse = serde_json::from_str(json).unwrap();
        match response {
            LlmResponse::ScoredArticles { scores } => {
                assert_eq!(scores.len(), 1);
                assert_eq!(scores[0].score, 85);
            }
            _ => panic!("Wrong response type"),
        }
    }
}
