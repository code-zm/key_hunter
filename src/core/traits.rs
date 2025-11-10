use async_trait::async_trait;
use regex::Regex;
use std::time::Duration;

use super::error::Result;
use super::results::{DetectedKey, HuntResults, ReportFormat, SearchQuery, SearchResult, ValidationResult};

/// Trait for detecting potential API keys in text content
pub trait KeyDetector: Send + Sync {
    /// Name of the key type this detector handles (e.g., "shodan", "aws", "github")
    fn name(&self) -> &str;

    /// Detect potential keys in the given content
    fn detect(&self, content: &str, file_path: &str) -> Vec<DetectedKey>;

    /// Get the regex patterns used for detection
    fn patterns(&self) -> &[Regex];

    /// File extensions to prioritize when searching (e.g., [".env", ".py", ".js"])
    fn file_extensions(&self) -> &[&str];

    /// Search queries to use when searching for this key type
    fn search_queries(&self) -> Vec<String>;

    /// Additional filtering logic for detected keys (e.g., entropy checks)
    fn filter_key(&self, key: &str) -> bool {
        !key.is_empty()
    }
}

/// Trait for validating detected keys against their actual APIs
#[async_trait]
pub trait KeyValidator: Send + Sync {
    /// Validate a key by making an API request
    async fn validate(&self, key: &str) -> Result<ValidationResult>;

    /// The key type this validator handles
    fn key_type(&self) -> &str;

    /// Rate limit between validation requests
    fn rate_limit(&self) -> Duration {
        Duration::from_secs(1)
    }
}

/// Trait for searching code repositories for exposed keys
#[async_trait]
pub trait SearchProvider: Send + Sync {
    /// Search for files matching the query
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;

    /// Name of the search provider (e.g., "github", "gitlab")
    fn name(&self) -> &str;

    /// Download file content from a search result
    async fn get_file_content(&self, result: &SearchResult) -> Result<String>;

    /// Maximum results per query (API limitation)
    fn max_results_per_query(&self) -> usize {
        100
    }
}

/// Trait for generating reports from hunt results
pub trait Reporter: Send + Sync {
    /// Generate a report from the results
    fn generate(&self, results: &HuntResults) -> Result<String>;

    /// The format this reporter outputs
    fn format(&self) -> ReportFormat;
}
